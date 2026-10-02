#!/usr/bin/env python3
"""Disposable redb recovery drill using the real riauth CLI and HTTP service.

Only selected, non-secret observations enter the evidence file. All credentials,
archives, sessions and stores live in a private temporary directory.
"""

import argparse
import hashlib
import importlib.util
import json
import os
import secrets
import socket
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path


class DrillFailure(RuntimeError):
    """Only the drill's own credential-free assertions may enter evidence."""


def require(condition, message):
    if not condition:
        raise DrillFailure(message)


def load_oidc():
    path = Path(__file__).with_name("recovery-drill-oidc.py")
    spec = importlib.util.spec_from_file_location("recovery_drill_oidc", path)
    require(spec is not None and spec.loader is not None, "RP helper unavailable")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def source_provenance():
    repository = Path(__file__).resolve().parent.parent
    environment = os.environ.copy()
    for name in ("GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_COMMON_DIR"):
        environment.pop(name, None)

    def git(*args):
        result = subprocess.run(["git", "-C", str(repository), *args], capture_output=True,
                                text=True, timeout=5, env=environment)
        require(result.returncode == 0, "source provenance unavailable")
        return result.stdout.strip()

    require(not git("status", "--porcelain", "--untracked-files=no"),
            "drill requires a clean tracked source tree")
    # Observed source provenance; the operator must correlate the binary hash
    # with an actual build of this tree. A cached executable is not attested here.
    return {"repository_head": git("rev-parse", "HEAD"),
            "production_src_tree": git("rev-parse", "HEAD:src"),
            "tracked_tree_clean": True}


def probe(url):
    try:
        with urllib.request.urlopen(url, timeout=2) as response:
            body = response.read()
            try:
                return response.status, json.loads(body)
            except json.JSONDecodeError:
                return response.status, {}
    except urllib.error.HTTPError as error:
        return error.code, {}
    except (urllib.error.URLError, TimeoutError, ConnectionError):
        return None, {}


def start(binary, config):
    return subprocess.Popen([str(binary), "--config", str(config), "serve"],
                            stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL,
                            stderr=subprocess.DEVNULL, start_new_session=True)


def stop(process):
    if process is not None and process.poll() is None:
        process.terminate()
        try:
            process.wait(timeout=8)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=3)


def wait_ready(process, url):
    until = time.monotonic() + 20
    while time.monotonic() < until:
        status, _ = probe(url + "/readyz")
        if status == 200:
            return
        require(process.poll() is None, "service exited before readiness")
        time.sleep(0.1)
    raise DrillFailure("service readiness timed out")


def write_evidence(path, evidence):
    payload = (json.dumps(evidence, indent=2, sort_keys=True) + "\n").encode()
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(descriptor, "wb") as output:
        output.write(payload)


def run(binary, evidence, oidc):
    with tempfile.TemporaryDirectory(prefix="riauth-recovery-drill-") as workspace:
        root = Path(workspace)
        config = root / "riauth.toml"
        admin_session = root / "admin-session.json"
        user_session = root / "user-session.json"
        backup_key = root / "backup.key"
        database_key = root / "database.key"
        wrong_key = root / "wrong.key"
        archive = root / "backup.riauth"
        restored = root / "restored"
        with socket.socket() as listener:
            listener.bind(("127.0.0.1", 0))
            port = listener.getsockname()[1]
        issuer = f"http://127.0.0.1:{port}"
        ready = issuer + "/readyz"
        admin_password = secrets.token_urlsafe(28)
        user_password = secrets.token_urlsafe(28)
        service = None
        relying_party = None

        def cli(args, *, session=admin_session, password=None, remote=False, timeout=60):
            argv = [str(binary), "--config", str(config), "--session-file", str(session),
                    "--json", "--non-interactive"]
            if remote:
                argv += ["--server", issuer]
            argv += args
            environment = os.environ.copy()
            for key in ("RIAUTH_SERVER", "RIAUTH_CONFIG", "RIAUTH_SESSION_FILE", "RIAUTH_AGENT_FILE",
                        "RIAUTH_PASSWORD", "RIAUTH_OTP", "RIAUTH_RUN_ID"):
                environment.pop(key, None)
            result = subprocess.run(argv, input=password, text=True, capture_output=True,
                                    timeout=timeout, env=environment)
            try:
                envelope = json.loads(result.stdout)
            except json.JSONDecodeError:
                envelope = {}
            return result.returncode, envelope

        def passed(identifier, **observed):
            evidence["checks"].append({"id": identifier, "result": "passed", "observed": observed})

        def success(args, **options):
            exit_code, envelope = cli(args, **options)
            require(exit_code == 0 and envelope.get("ok") is True,
                    f"{' '.join(args[:2])}: exit {exit_code}, code {envelope.get('error', {}).get('code')}")
            return envelope["data"]

        def refusal(args, **options):
            exit_code, envelope = cli(args, **options)
            require(exit_code != 0 and envelope.get("ok") is False,
                    f"{' '.join(args[:2])}: expected a structured refusal")
            return {"exit_code": exit_code, "error_code": envelope["error"]["code"]}

        try:
            capabilities = success(["capabilities"])
            passed("binary_capabilities", edition=capabilities.get("edition"))
            success(["keygen", "--out", str(backup_key)])
            success(["keygen", "--out", str(database_key)])
            success(["keygen", "--out", str(wrong_key)])
            require(backup_key.read_bytes() != database_key.read_bytes(), "keys are equal")
            success(["init", "--issuer", issuer, "--listen", f"127.0.0.1:{port}",
                     "--data-dir", str(root / "data"), "--database-key-file", str(database_key),
                     "--password-stdin"], password=admin_password + "\n", timeout=90)
            service = start(binary, config)
            wait_ready(service, issuer)
            passed("source_ready", http_status=200, storage="encrypted_redb")

            denied_archive = root / "denied.riauth"
            denied = refusal(["backup", "--key-file", str(backup_key), "--out", str(denied_archive)],
                             session=root / "no-session.json", remote=True)
            require(not denied_archive.exists(), "denied backup created an archive")
            passed("backup_without_session_denied", **denied, output_absent=True)

            success(["login", "admin", "--password-stdin"],
                    password=admin_password + "\n", remote=True)
            revision = success(["revision"], remote=True)["revision"]
            success(["user", "create", "drill-user", "--password-stdin",
                     "--if-revision", str(revision), "--idempotency-key", secrets.token_hex(16)],
                    password=user_password + "\n", remote=True)
            success(["login", "drill-user", "--password-stdin"], session=user_session,
                    password=user_password + "\n", remote=True)
            identity = success(["whoami"], session=user_session, remote=True)
            require(identity["user"]["username"] == "drill-user", "source user identity mismatch")
            passed("representative_login_before_backup", username="drill-user",
                   login_kind="identity_service")

            relying_party = oidc.LocalRelyingParty(root, issuer, "recovery-drill-rp")
            revision = success(["revision"], remote=True)["revision"]
            client = success(["client", "create", relying_party.client_id,
                              "--redirect-uri", relying_party.redirect_uri, "--scope", "openid,profile",
                              "--if-revision", str(revision), "--idempotency-key", secrets.token_hex(16)],
                             remote=True)
            require(client["client"]["confidential"] is False and client["client_secret"] is None
                    and client["client"]["redirect_uris"] == [relying_party.redirect_uri],
                    "RP public client registration mismatch")
            passed("local_rp_registered", public_client=True, exact_loopback_callback=True,
                   **relying_party.provider())
            passed("application_login_before_backup", **relying_party.login(user_session))

            backup = success(["backup", "--key-file", str(backup_key), "--out", str(archive)],
                             remote=True, timeout=120)
            require(backup["verified"] is True and archive.stat().st_size > 0, "backup unverified")
            passed("authenticated_backup", format=backup["api_version"], verified=True,
                   bytes=archive.stat().st_size)

            stop(service)
            service = None
            outage_status, _ = probe(ready)
            require(outage_status is None, "stopped source still accepts readiness")
            outage_login = refusal(["login", "drill-user", "--password-stdin"],
                                   session=root / "outage-session.json",
                                   password=user_password + "\n", remote=True)
            passed("source_outage", ready_http_status=None, login=outage_login)

            wrong_target = root / "wrong-key-restore"
            wrong = refusal(["restore", "--backup", str(archive), "--key-file", str(wrong_key),
                             "--out", str(wrong_target), "--database-key-file", str(database_key)])
            require(not wrong_target.exists(), "wrong key created restore target")
            passed("wrong_backup_key_rejected", **wrong, output_absent=True)

            tampered = root / "tampered.riauth"
            altered = bytearray(archive.read_bytes())
            altered[len(altered) // 2] ^= 1
            tampered.write_bytes(altered)
            tamper_target = root / "tampered-restore"
            tamper = refusal(["restore", "--backup", str(tampered), "--key-file", str(backup_key),
                              "--out", str(tamper_target), "--database-key-file", str(database_key)])
            require(not tamper_target.exists(), "tampered archive created restore target")
            passed("tampered_archive_rejected", **tamper, output_absent=True)

            occupied = root / "occupied"
            occupied.mkdir()
            marker = occupied / "marker"
            marker.write_text("retain")
            collision = refusal(["restore", "--backup", str(archive), "--key-file", str(backup_key),
                                 "--out", str(occupied), "--database-key-file", str(database_key)])
            require(marker.read_text() == "retain", "existing target modified")
            passed("existing_target_preserved", **collision, marker_unchanged=True)

            result = success(["restore", "--backup", str(archive), "--key-file", str(backup_key),
                              "--out", str(restored), "--database-key-file", str(database_key)],
                             timeout=120)
            require(result["verified"] is True and result["serving_allowed"] is False,
                    "restore verification or gate mismatch")
            require(result["encrypted_at_rest"] is True and result["storage"] == "redb",
                    "restored storage mismatch")
            recovery_id = result["recovery"]["id"]
            require(result["recovery"]["invalidated"]["sessions"] >= 2,
                    "restored sessions were not invalidated")
            passed("restore_verified_and_gated", verified=True, serving_allowed=False,
                   encrypted_at_rest=True, storage="redb", recovery_id=recovery_id,
                   invalidated_sessions=result["recovery"]["invalidated"]["sessions"])

            config = restored / "riauth.toml"
            status = success(["recovery", "status"])
            require(status["pending"]["id"] == recovery_id and status["serving_allowed"] is False,
                    "pending recovery status mismatch")
            blocked = subprocess.run([str(binary), "--config", str(config), "--json", "serve"],
                                     capture_output=True, text=True, timeout=12)
            require(blocked.returncode != 0, "gated service started")
            require(probe(ready)[0] is None, "gated service opened listener")
            passed("gated_serve_denied", exit_code=blocked.returncode, listener_closed=True)

            wrong_id = refusal(["recovery", "complete", "--recovery-id", "wrong-recovery",
                                "--persistent-credentials-reconciled"])
            require(success(["recovery", "status"])["pending"]["id"] == recovery_id,
                    "wrong ID changed the recovery gate")
            passed("wrong_recovery_id_denied", **wrong_id, gate_preserved=True)

            no_attestation = refusal(["recovery", "complete", "--recovery-id", recovery_id])
            require(success(["recovery", "status"])["pending"]["id"] == recovery_id,
                    "missing attestation changed the recovery gate")
            passed("missing_reconciliation_attestation_denied", **no_attestation,
                   gate_preserved=True)

            # This attestation is justified only for the synthetic fixture: both
            # persistent passwords were just issued and verified above, and the
            # public fixture client has no secret or post-snapshot policy changes.
            completed = success(["recovery", "complete", "--recovery-id", recovery_id,
                                 "--persistent-credentials-reconciled"])
            require(completed["serving_allowed"] is True, "completion did not open gate")
            require(success(["recovery", "status"])["pending"] is None,
                    "pending gate survived completion")
            passed("recovery_completed", serving_allowed=True, synthetic_credentials_reviewed=True)

            service = start(binary, config)
            wait_ready(service, issuer)
            denied_session = refusal(["whoami"], session=user_session, remote=True)
            passed("restored_session_rejected", **denied_session)
            success(["login", "drill-user", "--password-stdin"], session=user_session,
                    password=user_password + "\n", remote=True)
            identity = success(["whoami"], session=user_session, remote=True)
            require(identity["user"]["username"] == "drill-user", "restored user identity mismatch")
            success(["login", "admin", "--password-stdin"],
                    password=admin_password + "\n", remote=True)
            doctor = success(["doctor"], remote=True)
            require(doctor["healthy"] is True and doctor["encrypted_at_rest"] is True
                    and doctor["storage"] == "redb" and doctor["users"] >= 2,
                    "restored doctor checks failed")
            discovery_status, discovery = probe(issuer + "/.well-known/openid-configuration")
            keys_status, keys = probe(issuer + "/oauth/jwks")
            require(discovery_status == 200 and discovery["issuer"] == issuer,
                    "restored discovery failed")
            require(keys_status == 200 and len(keys["keys"]) > 0, "restored JWKS failed")
            passed("restored_service_and_login", ready_http_status=200, username="drill-user",
                   login_kind="identity_service",
                   doctor_healthy=True, doctor_encrypted_at_rest=True,
                   discovery_http_status=discovery_status,
                   jwks_http_status=keys_status, jwks_key_count=len(keys["keys"]))
            passed("application_login_after_restore", **relying_party.login(user_session, restored=True))
        finally:
            try:
                if relying_party is not None:
                    relying_party.close()
            finally:
                stop(service)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True, help="built riauth binary")
    parser.add_argument("--evidence", type=Path, required=True, help="new JSON evidence file")
    args = parser.parse_args()
    binary = args.binary.resolve(strict=True)
    require(args.evidence.parent.is_dir(), "evidence parent directory does not exist")
    require(not args.evidence.exists(), "evidence output already exists")
    oidc = load_oidc()
    evidence = {"schema_version": "riauth.recovery-drill/v1",
                "started_at": datetime.now(timezone.utc).isoformat(),
                "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
                "script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                "oidc_helper_sha256": hashlib.sha256(Path(oidc.__file__).read_bytes()).hexdigest(),
                "scope": "disposable localhost redb, real CLI/HTTP service and synthetic local OIDC RP",
                "checks": [], "result": "failed",
                "external_gates": ["lost backup key", "lost encrypted database key without archive",
                                   "missing configured secret files", "Vault Transit and other external services",
                                   "PostgreSQL PITR and multi-node failover",
                                   "deployed external OIDC or SAML relying party"]}
    try:
        evidence["observed_source"] = source_provenance()
        run(binary, evidence, oidc)
        require(hashlib.sha256(binary.read_bytes()).hexdigest() == evidence["binary_sha256"],
                "binary changed during drill")
        evidence["result"] = "passed"
    except Exception as error:
        # Foreign exception strings can echo arguments, tokens or callback URLs.
        message = (str(error) if isinstance(error, (DrillFailure, oidc.DrillFailure))
                   else "drill operation failed; private exception details suppressed")
        evidence["failure"] = {"type": type(error).__name__, "message": message}
    evidence["finished_at"] = datetime.now(timezone.utc).isoformat()
    write_evidence(args.evidence, evidence)
    print(json.dumps({"result": evidence["result"], "checks": len(evidence["checks"]),
                      "evidence": str(args.evidence)}))
    return 0 if evidence["result"] == "passed" else 1


if __name__ == "__main__":
    sys.exit(main())
