#!/usr/bin/env python3
"""Build one revision's two server editions and record a local, executable matrix.

The report distinguishes observed native binaries from absent release assets. It
never treats a Cargo target or a workflow definition as a published artifact.
"""

import argparse
import datetime
import hashlib
import json
import os
import pathlib
import platform
import re
import shutil
import socket
import subprocess
import tempfile
import time
import urllib.error
import urllib.request


ROOT = pathlib.Path(__file__).resolve().parent.parent
PLATFORM_ONLY = frozenset({
    "access.temporary_entitlements", "agents.certificate_bindings",
    "audit.self_hosted_event_map",
    "directory.entra_sync", "directory.ldap_provider",
    "directory.scim_inbound", "directory.workspace_sync",
    "identity.device_trust", "identity.https_client_certificates",
    "identity.saml_sources", "identity.scheduled_offboarding",
    "identity.windows_device_login", "operations.vault_transit_signing",
    "proxy.forward_auth_sso", "proxy.reverse_proxy",
    "proxy.shared_domain_sso", "radius.eap_tls", "radius.pap",
    "radius.radsec", "saml.assertion_encryption",
    "saml.idp_signed_browser_sso", "saml.logout_fanout",
    "saml.sp_initiated_logout", "saml.upstream_logout", "ssf.push",
    "workflow.controlled_extensions",
})
SHARED_SENTINELS = frozenset({
    "identity.passkeys", "identity.oidc_sources", "directory.ldap_sync",
    "directory.scim_outbound", "operations.postgresql", "oidc.code.pkce_s256",
    "operations.encrypted_backup_restore", "portal.user_applications",
    "agents.parent_ownership",
})
PLATFORM_DIRECT_DEPS = frozenset({
    "flate2", "hmac", "hyper", "hyper-util", "ldap3_proto", "md-5", "psl",
    "risaml", "roxmltree", "sha1", "time", "tokio-rustls", "tokio-util",
    "wasmi", "x509-parser",
})
RELEASE_NAMES = (
    "riauth-essentials-linux-{arch}.tar.gz",
    "riauth-platform-linux-{arch}.tar.gz",
    "riauth-maintenance-essentials-linux-{arch}.tar.gz",
    "riauth-maintenance-platform-linux-{arch}.tar.gz",
    "riauthctl-linux-{arch}.tar.gz",
    "riauth-essentials-linux-{arch}.docker.tar.gz",
    "riauth-platform-linux-{arch}.docker.tar.gz",
    "build-provenance-linux-{arch}.json",
    "SHA256SUMS-linux-{arch}",
)


def digest(path):
    checksum = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            checksum.update(chunk)
    return checksum.hexdigest()


def command(args, *, input=None, env=None, timeout=120):
    return subprocess.run(args, cwd=ROOT, input=input, text=True,
                          capture_output=True, env=env, timeout=timeout)


def require(condition, detail):
    if not condition:
        raise AssertionError(detail)


def successful(args, **kwargs):
    result = command(args, **kwargs)
    require(result.returncode == 0,
            f"{args!r}: exit {result.returncode}: {result.stdout[-1000:]} {result.stderr[-1000:]}")
    return result.stdout


def envelope(binary, *args, input=None):
    result = command([str(binary), "--json", *args], input=input)
    try:
        document = json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise AssertionError(f"{binary} {args}: no JSON: {result.stdout[-1000:]} {result.stderr[-1000:]}") from error
    return result.returncode, document


def free_port():
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        return listener.getsockname()[1]


def get(url):
    try:
        with urllib.request.urlopen(url, timeout=2) as response:
            return response.status, response.read()
    except urllib.error.HTTPError as error:
        return error.code, error.read()


def post_json(url, body):
    request = urllib.request.Request(url, data=json.dumps(body).encode(),
                                     headers={"Content-Type": "application/json"})
    try:
        with urllib.request.urlopen(request, timeout=5) as response:
            return response.status, response.read()
    except urllib.error.HTTPError as error:
        return error.code, error.read()


def ready(process, base):
    until = time.monotonic() + 20
    while time.monotonic() < until:
        require(process.poll() is None, "server exited before readiness")
        try:
            if get(base + "/readyz")[0] == 200:
                return
        except OSError:
            pass
        time.sleep(0.1)
    raise AssertionError("server did not become ready")


def serve(binary, config, base):
    process = subprocess.Popen([str(binary), "--config", str(config), "serve"],
                               cwd=ROOT, stdout=subprocess.DEVNULL,
                               stderr=subprocess.PIPE, text=True)
    try:
        ready(process, base)
        status, body = get(base + "/api/capabilities")
        require(status == 200, f"capabilities HTTP status {status}")
        document = json.loads(body)
        require(document["scope"] == "instance", "instance capability scope missing")
        login_status, login_body = post_json(base + "/api/login", {"username": "admin",
                                                              "password": "q08-disposable-password"})
        require(login_status == 200 and json.loads(login_body).get("session_token"),
                f"login failed: {login_status}")
        return {"readyz": 200, "capabilities": 200,
                "login": 200, "edition": document["edition"], "storage": "opened"}
    except Exception:
        process.terminate()
        process.wait(timeout=5)
        raise AssertionError(f"serve failed: {process.stderr.read()[-2000:]}")
    finally:
        if process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=5)
        process.stderr.close()


def init_instance(binary, root, postgres_config=None):
    root.mkdir()
    port = free_port()
    base = f"http://127.0.0.1:{port}"
    config = root / "riauth.toml"
    args = ["--config", str(config), "--non-interactive", "init",
            "--issuer", base, "--listen", f"127.0.0.1:{port}",
            "--data-dir", str(root / "data"), "--password-stdin"]
    if postgres_config:
        args += ["--postgres-config", str(postgres_config)]
    status, document = envelope(binary, *args, input="q08-disposable-password\n")
    require(status == 0 and document["ok"], f"init failed: {document}")
    return config, base


def postgres_cluster(root):
    programs = {name: shutil.which(name) for name in ("initdb", "pg_ctl", "createdb")}
    if not all(programs.values()):
        return None, f"missing PostgreSQL programs: {sorted(k for k, v in programs.items() if not v)}"
    cluster = root / "postgres-cluster"
    port = free_port()
    successful([programs["initdb"], "-D", str(cluster), "-U", "riauth_test",
                "--auth=trust", "--encoding=UTF8", "--no-locale"], timeout=60)
    with (cluster / "postgresql.conf").open("a") as config:
        config.write(f"\nlisten_addresses = '127.0.0.1'\nport = {port}\nunix_socket_directories = ''\n")
    successful([programs["pg_ctl"], "-D", str(cluster), "-l", str(root / "postgres.log"),
                "start", "-w"], timeout=60)
    return (programs, cluster, port), None


def check_capabilities(binaries):
    documents = {}
    for edition, binary in binaries.items():
        status, document = envelope(binary, "capabilities")
        require(status == 0 and document["ok"], f"{edition} capabilities failed: {document}")
        data = document["data"]
        require(data["edition"] == edition and data["scope"] == "artifact", f"{edition} identity")
        require(data["build_features"] == (["essentials"] if edition == "essentials" else ["essentials", "platform"]),
                f"{edition} feature flags")
        host_os = {"Darwin": "macos"}.get(platform.system(), platform.system().lower())
        require(data["target"] == {"os": host_os, "arch": platform.machine().replace("arm64", "aarch64")},
                f"{edition} target: {data['target']}")
        documents[edition] = data
    essentials = set(documents["essentials"]["compiled_features"])
    full = set(documents["platform"]["compiled_features"])
    require(PLATFORM_ONLY <= full and not PLATFORM_ONLY & essentials,
            "Platform-only compiled feature boundary differs")
    require(SHARED_SENTINELS <= essentials, "shared compiled capability missing")
    require(full - essentials == PLATFORM_ONLY, f"unexpected edition feature delta: {full - essentials ^ PLATFORM_ONLY}")
    for edition, data in documents.items():
        states = data["feature_states"]
        require(set(states) == full, f"{edition} incomplete capability states")
        require({name for name, state in states.items() if state["compiled"]} == set(data["compiled_features"]),
                f"{edition} compiled states disagree")
        require(all(state["usable"] is None for state in states.values()),
                "artifact capabilities asserted runtime usability")
    return {edition: {"compiled_count": len(data["compiled_features"]),
                      "compiled_features": data["compiled_features"],
                      "excluded_features": sorted(full - set(data["compiled_features"]))}
            for edition, data in documents.items()}


def direct_dependencies():
    dependencies = {}
    for edition in ("essentials", "platform"):
        output = successful(["cargo", "tree", "--locked", "--no-default-features",
                             "--features", edition, "--edges", "normal", "--depth", "1",
                             "--prefix", "none"])
        dependencies[edition] = {match.group(1) for line in output.splitlines()[1:]
                                 if (match := re.match(r"^(\S+) v\S+", line))}
    require(dependencies["platform"] - dependencies["essentials"] == PLATFORM_DIRECT_DEPS,
            f"unexpected direct dependency delta: {dependencies['platform'] - dependencies['essentials'] ^ PLATFORM_DIRECT_DEPS}")
    return {"platform_only": sorted(PLATFORM_DIRECT_DEPS),
            "essentials_direct_count": len(dependencies["essentials"]),
            "platform_direct_count": len(dependencies["platform"])}


def release_assets(root):
    result = {}
    for arch in ("x86_64", "aarch64"):
        files = {}
        for template in RELEASE_NAMES:
            name = template.format(arch=arch)
            path = root / name
            files[name] = {"status": "present", "sha256": digest(path)} if path.is_file() else {"status": "unavailable"}
        result[arch] = {"status": "present_unverified" if all(v["status"] == "present" for v in files.values())
                        else "unavailable_incomplete", "files": files}
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target-dir", type=pathlib.Path, default=ROOT / "target" / "q08-exact-bundles")
    parser.add_argument("--release-dir", type=pathlib.Path, default=ROOT / "target" / "dist")
    parser.add_argument("--out", type=pathlib.Path, required=True)
    args = parser.parse_args()
    target = args.target_dir.resolve()
    revision = successful(["git", "rev-parse", "HEAD"]).strip()
    source_status = successful(["git", "status", "--porcelain", "--untracked-files=all",
                                "--", "Cargo.toml", "Cargo.lock", "src", "crates"]).strip()
    require(not source_status, f"Cargo build source differs from HEAD: {source_status}")
    report = {"schema": "riauth.exact-edition-matrix/v1", "source_revision": revision,
              "captured_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
              "script_sha256": digest(pathlib.Path(__file__)),
              "build_source_dirty": bool(source_status), "host": {"os": platform.system(),
              "machine": platform.machine(), "rustc": successful(["rustc", "--version"]).strip()},
              "cargo_lock_sha256": digest(ROOT / "Cargo.lock"), "builds": {}, "checks": {},
              "release_assets": release_assets(args.release_dir.resolve())}
    copies = {}
    for edition in ("essentials", "platform"):
        folder = target / "exact-binaries" / edition
        folder.mkdir(parents=True, exist_ok=True)
        build = ["cargo", "build", "--locked", "--no-default-features", "--features", edition, "--bins"]
        environment = dict(os.environ, CARGO_TARGET_DIR=str(target), CARGO_PROFILE_DEV_DEBUG="0", CARGO_BUILD_JOBS="2")
        successful(build, env=environment, timeout=3600)
        for name in ("riauth", "riauth-maintenance"):
            shutil.copy2(target / "debug" / name, folder / name)
        copies[edition] = {name: folder / name for name in ("riauth", "riauth-maintenance")}
        for path in copies[edition].values():
            require(path.is_file(), f"missing copied binary: {path}")
        report["builds"][edition] = {"command": " ".join(build),
            "environment": {"CARGO_TARGET_DIR": str(target), "CARGO_PROFILE_DEV_DEBUG": "0", "CARGO_BUILD_JOBS": "2"},
            "status": "passed",
            "binaries": {name: {"sha256": digest(path), "bytes": path.stat().st_size}
                         for name, path in copies[edition].items()}}
    report["checks"]["capabilities"] = check_capabilities({k: v["riauth"] for k, v in copies.items()})
    report["checks"]["direct_dependencies"] = direct_dependencies()
    with tempfile.TemporaryDirectory(prefix="riauth-q08-") as temporary:
        scratch = pathlib.Path(temporary)
        report["checks"]["storage"] = {}
        instances = {}
        for edition in ("essentials", "platform"):
            config, base = init_instance(copies[edition]["riauth-maintenance"], scratch / f"{edition}-redb")
            report["checks"]["storage"][f"{edition}/redb"] = serve(copies[edition]["riauth"], config, base)
            instances[edition] = (config, base)
        # Each valid Platform setting must be rejected by Essentials before a listener starts.
        # Explicit SAML 30 equals the initialized default; the capability change
        # below remains a genuine initialized-agreement mismatch.
        report["checks"]["config_rejection"] = {}
        for label, addition, message in (
            ("saml_rate_limit", "\n[rate_limits]\nsaml = 30\n",
             "rate_limits.saml requires the Platform build"),
            ("device_trust_activation", "\n[capabilities]\ndisabled = [\"identity.device_trust\"]\n",
             "Disabled capability identity.device_trust requires the Platform build"),
        ):
            for edition in ("essentials", "platform"):
                original, base = instances[edition]
                candidate = original.parent / f"{label}.toml"
                candidate.write_text(original.read_text() + addition)
                if edition == "essentials":
                    status, document = envelope(copies[edition]["riauth"], "--config", str(candidate), "serve")
                    require(status != 0 and message in document["error"]["message"],
                            f"Essentials accepted incompatible config: {document}")
                    report["checks"]["config_rejection"][label] = {"exit_code": status,
                        "message": document["error"]["message"], "platform_same_setting": "pending"}
                else:
                    if label == "saml_rate_limit":
                        serve(copies[edition]["riauth"], candidate, base)
                        result = "accepted_and_ready"
                    else:
                        status, document = envelope(copies[edition]["riauth"],
                                                    "--config", str(candidate), "serve")
                        require(status != 0 and
                                "Configured active capabilities do not match the initialized instance"
                                in document["error"]["message"],
                                f"Platform changed initialized security agreement: {document}")
                        result = "rejected_after_init_by_security_agreement"
                    report["checks"]["config_rejection"][label]["platform_same_setting"] = result
        cluster, reason = postgres_cluster(scratch)
        if cluster is None:
            report["checks"]["storage"]["postgresql"] = {"status": "unavailable", "reason": reason}
        else:
            programs, data_dir, port = cluster
            try:
                for edition in ("essentials", "platform"):
                    database = f"riauth_q08_{edition}"
                    successful([programs["createdb"], "-h", "127.0.0.1", "-p", str(port),
                                "-U", "riauth_test", database])
                    root = scratch / f"{edition}-postgres"
                    root.mkdir()
                    connection = root / "connection"
                    connection.write_text(f"host=127.0.0.1 port={port} dbname={database} user=riauth_test sslmode=disable\n")
                    connection.chmod(0o600)
                    pg_config = root / "postgres.json"
                    pg_config.write_text(json.dumps({"connection_file": str(connection),
                                                      "ca_file": None, "local_unencrypted": True}))
                    config, base = init_instance(copies[edition]["riauth-maintenance"], root / "instance", pg_config)
                    report["checks"]["storage"][f"{edition}/postgresql"] = serve(copies[edition]["riauth"], config, base)
            finally:
                successful([programs["pg_ctl"], "-D", str(data_dir), "stop", "-m", "immediate", "-w"])
    require(successful(["git", "rev-parse", "HEAD"]).strip() == revision,
            "source revision changed during matrix")
    require(successful(["git", "status", "--porcelain", "--untracked-files=all",
                        "--", "Cargo.toml", "Cargo.lock", "src", "crates"]).strip() == source_status,
            "build source changed during matrix")
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    print(f"Exact edition matrix passed for {revision}; report: {args.out}")


if __name__ == "__main__":
    main()
