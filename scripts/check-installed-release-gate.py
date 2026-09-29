#!/usr/bin/env python3
"""Gate one native Linux release asset set on installed-binary recovery behavior.

The package job runs this once per architecture. No Cargo or checkout binary is
executed. Missing, mixed, or unverified release assets fail before installation.
"""

import argparse
import contextlib
import hashlib
import importlib.util
import json
import pathlib
import platform
import re
import socket
import subprocess
import tarfile
import tempfile
import time
import urllib.error
import urllib.request


def _package_documents():
    path = pathlib.Path(__file__).resolve().parent / "spdx_sbom.py"
    spec = importlib.util.spec_from_file_location("riauth_spdx_sbom", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


package_documents = _package_documents()


ROOT = pathlib.Path(__file__).resolve().parent.parent
EDITIONS = ("essentials", "platform")
ARCH_TO_OCI = {"x86_64": "amd64", "aarch64": "arm64"}
PASSWORD = "q10-disposable-artifact-password"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(path):
    checksum = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            checksum.update(block)
    return checksum.hexdigest()


def run(args, *, input=None, timeout=120):
    return subprocess.run(args, input=input, text=True, capture_output=True, timeout=timeout)


def success(args, *, input=None, timeout=120):
    result = run(args, input=input, timeout=timeout)
    require(result.returncode == 0,
            f"{args[0]} exited {result.returncode}: {result.stdout[-700:]} {result.stderr[-700:]}")
    return result.stdout


def cli(binary, *args, input=None, expected=0):
    result = run([str(binary), "--json", *map(str, args)], input=input)
    try:
        envelope = json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise ValueError(f"{binary.name} returned no JSON: {result.stdout[-700:]} {result.stderr[-700:]}") from error
    require((result.returncode != 0 if expected is None else result.returncode == expected),
            f"{binary.name} {args}: exit {result.returncode}: {envelope}")
    require(envelope.get("schema_version") == "riauth.cli/v1", f"{binary.name} CLI schema changed")
    require(envelope.get("ok") is (result.returncode == 0), f"{binary.name} {args}: unexpected outcome")
    return envelope.get("data") if expected == 0 else envelope


def expected_names(arch):
    return {f"riauth-{edition}-linux-{arch}.tar.gz" for edition in EDITIONS} | {
        f"riauth-maintenance-{edition}-linux-{arch}.tar.gz" for edition in EDITIONS
    } | {f"riauth-{edition}-linux-{arch}.docker.tar.gz" for edition in EDITIONS} | {
        f"riauthctl-linux-{arch}.tar.gz", f"build-provenance-linux-{arch}.json",
        f"SHA256SUMS-linux-{arch}",
    } | package_documents.linux_spdx_names(arch)


def verify_assets(dist, arch, commit, repository, run_id, run_attempt):
    require(arch in ARCH_TO_OCI, f"unsupported architecture {arch}")
    require(re.fullmatch(r"[0-9a-f]{40}", commit) is not None, "release commit is missing or invalid")
    require(re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repository) is not None,
            "release repository is missing or invalid")
    require(re.fullmatch(r"[1-9][0-9]*", run_id) is not None
            and re.fullmatch(r"[1-9][0-9]*", run_attempt) is not None,
            "release run identity is missing or invalid")
    require(dist.is_dir(), f"exact release asset directory unavailable: {dist}")
    expected = expected_names(arch)
    actual = {path.name for path in dist.iterdir()}
    require(actual == expected, f"exact {arch} release assets unavailable or unexpected: {sorted(expected ^ actual)}")
    require(all(path.is_file() and not path.is_symlink() for path in dist.iterdir()),
            "release assets must be regular files")
    sums_name = f"SHA256SUMS-linux-{arch}"
    entries = {}
    for line in (dist / sums_name).read_text().splitlines():
        match = re.fullmatch(r"([0-9a-f]{64})  ([A-Za-z0-9_.-]+)", line)
        require(match is not None, f"invalid checksum line: {line!r}")
        checksum, name = match.groups()
        require(name not in entries, f"duplicate checksum: {name}")
        entries[name] = checksum
    require(set(entries) == expected - {sums_name}, "release checksum manifest differs from exact asset set")
    for name, checksum in entries.items():
        require(digest(dist / name) == checksum, f"release checksum mismatch: {name}")
    package_documents.require_linux_package_bytes(dist, arch)
    provenance = json.loads((dist / f"build-provenance-linux-{arch}.json").read_text())
    required = {"schema": "riauth.build/v4", "commit": commit, "repository": repository,
                "run_id": run_id, "run_attempt": run_attempt,
                "target_triple": f"{arch}-unknown-linux-gnu",
                "oci_platform": f"linux/{ARCH_TO_OCI[arch]}",
                "cargo_lock_sha256": digest(ROOT / "Cargo.lock"),
                "riauthctl_cargo_lock_sha256": digest(ROOT / "crates/riauthctl/Cargo.lock"),
                "riauthctl_features": "no-default-features"}
    for field, value in required.items():
        require(provenance.get(field) == value, f"release provenance {field} mismatch")
    require(provenance.get("build_os", {}).get("name") == "Linux"
            and provenance["build_os"].get("architecture") == arch,
            "release build OS/architecture mismatch")
    require(provenance.get("rustc", "").startswith("rustc 1.98.1 "), "release Rust toolchain mismatch")
    for edition, features in (("essentials", ["essentials"]), ("platform", ["essentials", "platform"])):
        build = provenance.get("server_builds", {}).get(edition, {})
        maintenance = provenance.get("maintenance_builds", {}).get(edition, {})
        require(build.get("features") == features and build.get("no_default_features") is True,
                f"{edition} server feature provenance mismatch")
        require(maintenance.get("features") == features and maintenance.get("no_default_features") is True,
                f"{edition} maintenance feature provenance mismatch")
        require(re.fullmatch(r"sha256:[0-9a-f]{64}", build.get("docker_image_id", "")) is not None,
                f"{edition} image ID missing")
        require(re.fullmatch(r"[0-9a-f]{64}", maintenance.get("binary_sha256", "")) is not None,
                f"{edition} maintenance binary hash missing")
    return provenance


def install_member(archive, name, destination):
    with tarfile.open(archive, "r:gz") as source:
        members = source.getmembers()
        require(len(members) == 3 and {member.name for member in members} ==
                {name, "LICENSE", "THIRD_PARTY_NOTICES.md"}
                and all(member.isfile() for member in members),
                f"{archive.name} has missing or unexpected members")
        member = source.getmember(name)
        require(member.size <= 256 * 1024 * 1024, f"{archive.name} {name} exceeds install bound")
        with source.extractfile(member) as payload, destination.open("wb") as output:
            require(payload is not None, f"{archive.name} cannot extract {name}")
            for block in iter(lambda: payload.read(1024 * 1024), b""):
                output.write(block)
    destination.chmod(0o700)
    return destination


def install(dist, arch, root, provenance):
    root.mkdir()
    binaries = {}
    for edition in EDITIONS:
        directory = root / edition
        directory.mkdir()
        server = install_member(dist / f"riauth-{edition}-linux-{arch}.tar.gz",
                                "riauth", directory / "riauth")
        maintenance = install_member(dist / f"riauth-maintenance-{edition}-linux-{arch}.tar.gz",
                                     "riauth-maintenance", directory / "riauth-maintenance")
        require(digest(maintenance) == provenance["maintenance_builds"][edition]["binary_sha256"],
                f"{edition} installed maintenance hash mismatch")
        binaries[edition] = {"server": server, "maintenance": maintenance,
                             "server_sha256": digest(server), "maintenance_sha256": digest(maintenance)}
    client_dir = root / "client"
    client_dir.mkdir()
    client = install_member(dist / f"riauthctl-linux-{arch}.tar.gz", "riauthctl", client_dir / "riauthctl")
    return binaries, client


def verify_installed_metadata(binaries, client, arch):
    capabilities = {}
    for edition in EDITIONS:
        server = binaries[edition]["server"]
        data = cli(server, "capabilities")
        require(data.get("schema_version") == "riauth.capabilities/v2"
                and data.get("scope") == "artifact" and data.get("interface") == "server",
                f"{edition} installed capability metadata changed")
        require(data.get("edition") == edition
                and data.get("build_features") == (["essentials"] if edition == "essentials" else ["essentials", "platform"]),
                f"{edition} installed build metadata mismatch")
        require(data.get("target") == {"os": "linux", "arch": arch},
                f"{edition} installed target mismatch")
        compiled = data.get("compiled_features", [])
        states = data.get("feature_states", {})
        require(compiled and len(compiled) == len(set(compiled))
                and {name for name, state in states.items() if state.get("compiled") is True} == set(compiled)
                and all(state.get("usable") is None for state in states.values()),
                f"{edition} installed compiled catalog/states disagree")
        require(data.get("permissions") and data.get("schemas")
                and data.get("cli_result_schema") == "riauth.cli/v1",
                f"{edition} installed API metadata missing")
        version = data["version"]
        require(success([str(server), "--version"]).strip() == f"riauth {version}",
                f"{edition} server version differs from capability metadata")
        require(success([str(binaries[edition]["maintenance"]), "--version"]).strip() ==
                f"riauth-maintenance {version}", f"{edition} maintenance version mismatch")
        capabilities[edition] = data
    require(capabilities["essentials"]["version"] == capabilities["platform"]["version"],
            "installed editions have different versions")
    essential = set(capabilities["essentials"]["compiled_features"])
    extra = set(capabilities["platform"]["compiled_features"]) - essential
    require(essential < set(capabilities["platform"]["compiled_features"]) and len(extra) >= 20,
            "installed Platform build lacks expected capability addition")
    require(success([str(client), "--version"]).strip() ==
            f"riauthctl {capabilities['essentials']['version']}", "installed client version mismatch")
    return {"version": capabilities["essentials"]["version"],
            "essentials_compiled": len(essential), "platform_compiled": len(capabilities["platform"]["compiled_features"])}


def free_port():
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        return listener.getsockname()[1]


def get_json(url):
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
    try:
        with opener.open(url, timeout=2) as response:
            return response.status, json.load(response)
    except urllib.error.HTTPError as error:
        return error.code, json.load(error)


def get_status(url):
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
    try:
        with opener.open(url, timeout=2) as response:
            response.read()
            return response.status
    except urllib.error.HTTPError as error:
        return error.code


@contextlib.contextmanager
def serving(binary, config, base, log):
    with log.open("w") as output:
        process = subprocess.Popen([str(binary), "--config", str(config), "serve"],
                                   stdout=output, stderr=output)
    try:
        deadline = time.monotonic() + 25
        while time.monotonic() < deadline:
            require(process.poll() is None, f"{binary} exited before readiness: {log.read_text()[-1000:]}")
            try:
                if get_status(base + "/readyz") == 200:
                    break
            except (OSError, ValueError):
                pass
            time.sleep(0.1)
        else:
            raise ValueError(f"{binary} did not become ready: {log.read_text()[-1000:]}")
        yield
    finally:
        if process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=5)


def remote(server, base, session, *args, input=None, expected=0):
    return cli(server, "--server", base, "--session-file", session,
               "--non-interactive", *args, input=input, expected=expected)


def drill(binaries, root):
    essential = binaries["essentials"]
    platform_binary = binaries["platform"]
    instance = root / "instance"
    instance.mkdir()
    port = free_port()
    base = f"http://127.0.0.1:{port}"
    config = instance / "riauth.toml"
    key = root / "backup.key"
    require(cli(essential["maintenance"], "keygen", "--out", key)["created"] is True,
            "installed maintenance keygen failed")
    initialized = cli(essential["maintenance"], "--config", config, "--non-interactive",
                      "init", "--issuer", base, "--listen", f"127.0.0.1:{port}",
                      "--data-dir", instance / "data", "--password-stdin", input=PASSWORD + "\n")
    require(initialized is not None and config.is_file(), "installed maintenance init failed")
    session = root / "session.json"
    backup = root / "before-platform.riauth"
    with serving(essential["server"], config, base, root / "essentials-first.log"):
        jwks_status, jwks = get_json(base + "/oauth/jwks")
        require(jwks_status == 200 and jwks.get("keys"), "installed Essentials JWKS missing")
        remote(essential["server"], base, session, "login", "admin", "--password-stdin",
               input=PASSWORD + "\n")
        # Backup is streamed by the installed server through the installed CLI.
        result = remote(essential["server"], base, session, "backup", "--key-file", key,
                        "--out", backup)
        require(result.get("verified") is True and result.get("api_version") == "riauth.backup/v3"
                and result.get("issuer") == base and backup.is_file(),
                "installed backup was not verified")
    before = cli(platform_binary["maintenance"], "--config", config,
                 "transition-preflight", "--target", "platform", expected=5)
    before = before["data"]
    require(before.get("ready") is False and before.get("store_schema") is not None
            and before.get("store_index_version") is not None
            and before.get("last_activated_edition") == "essentials"
            and "meta/node_security" in {item["resource"] for item in before["blockers"]},
            "Essentials store metadata did not preflight for Platform")
    upgrade = cli(platform_binary["maintenance"], "--config", config,
                  "transition-plan", "--target", "platform")
    require(upgrade.get("ready") is True and upgrade.get("transition_token"),
            "supported clean upgrade handoff unavailable")
    upgraded = cli(platform_binary["maintenance"], "--config", config,
                   "transition-activate", "--target", "platform",
                   "--token", upgrade["transition_token"])
    require(upgraded.get("activated_edition") == "platform"
            and upgraded.get("target_revision", 0) > upgraded.get("source_revision", 0),
            "installed maintenance upgrade handoff failed")
    with serving(platform_binary["server"], config, base, root / "platform.log"):
        require(get_json(base + "/oauth/jwks") == (200, jwks),
                "Platform open changed signing keys")
        remote(platform_binary["server"], base, root / "platform-session.json",
               "login", "admin", "--password-stdin", input=PASSWORD + "\n")
    rejected = cli(essential["server"], "--config", config, "serve", expected=None)
    require("Platform" in rejected["error"]["message"] or
            "compiled capability" in rejected["error"]["message"],
            "Essentials did not reject direct downgrade")
    blocked = cli(platform_binary["maintenance"], "--config", config,
                  "transition-preflight", "--target", "essentials", expected=5)
    resources = {item["resource"] for item in blocked["data"]["blockers"]}
    require({"meta/version_activation", "meta/edition_provenance"} <= resources,
            "direct downgrade blockers missing")
    plan = cli(platform_binary["maintenance"], "--config", config,
               "transition-plan", "--target", "essentials")
    require(plan.get("ready") is True and plan.get("transition_token"),
            "supported clean edition handoff unavailable")
    activated = cli(platform_binary["maintenance"], "--config", config,
                    "transition-activate", "--target", "essentials",
                    "--token", plan["transition_token"])
    require(activated.get("activated_edition") == "essentials"
            and activated.get("target_revision", 0) > activated.get("source_revision", 0),
            "installed maintenance handoff failed")
    with serving(essential["server"], config, base, root / "essentials-return.log"):
        require(get_json(base + "/oauth/jwks") == (200, jwks),
                "supported edition return changed signing keys")
        remote(essential["server"], base, root / "returned-session.json",
               "login", "admin", "--password-stdin", input=PASSWORD + "\n")
    restored = root / "restored"
    result = cli(essential["maintenance"], "restore", "--backup", backup,
                 "--key-file", key, "--out", restored)
    require(result.get("restored") is True and result.get("verified") is True
            and result.get("storage") == "redb" and result.get("serving_allowed") is False,
            "installed maintenance restore did not verify isolated redb")
    restored_config = restored / "riauth.toml"
    status = cli(essential["server"], "--config", restored_config, "recovery", "status")
    require(status.get("initialized") is True and status.get("serving_allowed") is False
            and isinstance(status.get("pending"), dict) and status["pending"].get("id"),
            "restored-state gate missing")
    denied = cli(essential["server"], "--config", restored_config, "serve", expected=None)
    require("recover" in denied["error"]["message"].lower(),
            "restored store served before reconciliation")
    complete = cli(essential["server"], "--config", restored_config, "recovery", "complete",
                   "--recovery-id", status["pending"]["id"],
                   "--persistent-credentials-reconciled")
    require(complete is not None, "restored-state completion failed")
    require(cli(essential["server"], "--config", restored_config,
                "recovery", "status")["serving_allowed"] is True,
            "restored-state serving gate stayed closed")
    with serving(essential["server"], restored_config, base, root / "restored.log"):
        require(get_json(base + "/oauth/jwks") == (200, jwks),
                "isolated restore changed signing keys")
        remote(essential["server"], base, session, "whoami", expected=3)
        remote(essential["server"], base, root / "restored-session.json",
               "login", "admin", "--password-stdin", input=PASSWORD + "\n")
    return {"backend": "redb", "schema": before["store_schema"],
            "index_version": before["store_index_version"],
            "edition_build_transition": "essentials-platform-essentials",
            "direct_downgrade": "rejected", "explicit_handoff": "passed",
            "backup_format": "riauth.backup/v3", "restore": "verified_isolated",
            "restored_state_gate": "closed_until_explicit_completion",
            "old_session_after_restore": "rejected", "signing_keys": "preserved"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("dist", type=pathlib.Path)
    parser.add_argument("commit")
    parser.add_argument("repository")
    parser.add_argument("run_id")
    parser.add_argument("run_attempt")
    args = parser.parse_args()
    arch = {"arm64": "aarch64"}.get(platform.machine(), platform.machine())
    dist = args.dist.resolve()
    require(success(["git", "rev-parse", "HEAD"]).strip() == args.commit,
            "checked-out revision differs from requested release commit")
    provenance = verify_assets(dist, arch, args.commit, args.repository,
                               args.run_id, args.run_attempt)
    require(platform.system() == "Linux", "native installed-artifact drill requires Linux")
    with tempfile.TemporaryDirectory(prefix="riauth-q10-") as temporary:
        root = pathlib.Path(temporary)
        binaries, client = install(dist, arch, root / "installed", provenance)
        metadata = verify_installed_metadata(binaries, client, arch)
        checks = drill(binaries, root)
        print(json.dumps({"schema": "riauth.release-gate/v1", "commit": args.commit,
                          "architecture": arch, "provenance_sha256": digest(dist / f"build-provenance-linux-{arch}.json"),
                          "installed": {edition: {"server_sha256": binaries[edition]["server_sha256"],
                                                   "maintenance_sha256": binaries[edition]["maintenance_sha256"]}
                                        for edition in EDITIONS},
                          "metadata": metadata, "checks": checks}, sort_keys=True))


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, KeyError, IndexError, subprocess.TimeoutExpired,
            tarfile.TarError, json.JSONDecodeError) as error:
        raise SystemExit(f"installed release gate failed: {error}") from error
