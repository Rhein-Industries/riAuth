#!/usr/bin/env python3
"""Run a disposable PostgreSQL archive recovery and storage-outage drill."""

import argparse
import hashlib
import json
import os
import secrets
import shutil
import socket
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def private_file(path, content):
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(descriptor, "wb") as output:
        output.write(content)


def port():
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        return listener.getsockname()[1]


def probe(url):
    try:
        with urllib.request.urlopen(url, timeout=3) as response:
            body = response.read()
            try:
                return response.status, json.loads(body)
            except json.JSONDecodeError:
                return response.status, {}
    except urllib.error.HTTPError as error:
        return error.code, {}
    except (urllib.error.URLError, TimeoutError, ConnectionError):
        return None, {}


def stop_service(service):
    if service is not None and service.poll() is None:
        service.terminate()
        try:
            service.wait(timeout=8)
        except subprocess.TimeoutExpired:
            service.kill()
            service.wait(timeout=3)


def wait_ready(service, issuer):
    until = time.monotonic() + 25
    while time.monotonic() < until:
        if probe(issuer + "/readyz")[0] == 200:
            return
        require(service.poll() is None, "service exited before readiness")
        time.sleep(0.1)
    raise RuntimeError("service readiness timed out")


def run(binary, pg_bin, evidence):
    with tempfile.TemporaryDirectory(prefix="riauth-pg-recovery-drill-") as workspace:
        root = Path(workspace)
        pg_data = root / "postgres"
        source_config = root / "riauth.toml"
        config = source_config
        admin_session = root / "admin-session.json"
        user_session = root / "user-session.json"
        backup_key = root / "backup.key"
        database_key = root / "database.key"
        wrong_key = root / "wrong.key"
        archive = root / "backup.riauth"
        restored = root / "restored"
        pg_port = port()
        http_port = port()
        require(pg_port != http_port, "port reservation collided")
        issuer = f"http://127.0.0.1:{http_port}"
        admin_password = secrets.token_urlsafe(28)
        user_password = secrets.token_urlsafe(28)
        service = None
        pg_running = False

        def pg(command, *args, timeout=60, allow_failure=False):
            result = subprocess.run([str(pg_bin / command), *map(str, args)],
                                    capture_output=True, text=True, timeout=timeout)
            if not allow_failure:
                require(result.returncode == 0,
                        f"{command} failed with exit {result.returncode}: {result.stderr[-300:]}")
            return result

        def sql(database, query):
            result = pg("psql", "-X", "-A", "-t", "-h", "127.0.0.1", "-p", pg_port,
                        "-U", "riauth_drill", "-d", database, "-c", query)
            return result.stdout.strip()

        def start_pg():
            nonlocal pg_running
            pg("pg_ctl", "-D", pg_data, "-l", root / "postgres.log", "start", "-w")
            pg_running = True

        def stop_pg():
            nonlocal pg_running
            if pg_running:
                pg("pg_ctl", "-D", pg_data, "stop", "-m", "immediate", "-w")
                pg_running = False

        def cli(args, *, session=admin_session, password=None, remote=False, timeout=60):
            argv = [str(binary), "--config", str(config), "--session-file", str(session),
                    "--json", "--non-interactive"]
            if remote:
                argv += ["--server", issuer]
            argv += list(map(str, args))
            environment = os.environ.copy()
            for key in ("RIAUTH_SERVER", "RIAUTH_CONFIG", "RIAUTH_SESSION_FILE", "RIAUTH_AGENT_FILE",
                        "RIAUTH_PASSWORD", "RIAUTH_OTP", "RIAUTH_RUN_ID"):
                environment.pop(key, None)
            result = subprocess.run(argv, input=password, text=True, capture_output=True,
                                    timeout=timeout, env=environment)
            try:
                envelope = json.loads(result.stdout)
            except json.JSONDecodeError:
                envelope = {"_stderr": result.stderr[-1000:]}
            return result.returncode, envelope

        def passed(identifier, **observed):
            evidence["checks"].append({"id": identifier, "result": "passed", "observed": observed})

        def success(args, **options):
            exit_code, envelope = cli(args, **options)
            require(exit_code == 0 and envelope.get("ok") is True,
                    f"{' '.join(map(str, args[:2]))}: exit {exit_code}, "
                    f"code {envelope.get('error', {}).get('code')}")
            return envelope["data"]

        def refusal(args, **options):
            exit_code, envelope = cli(args, **options)
            require(exit_code != 0 and envelope.get("ok") is False,
                    f"{' '.join(map(str, args[:2]))}: expected a structured refusal; "
                    f"exit {exit_code}, ok {envelope.get('ok')}, "
                    f"code {envelope.get('error', {}).get('code')}, "
                    f"stderr {envelope.get('_stderr', '')}")
            return {"exit_code": exit_code, "error_code": envelope["error"]["code"]}

        def pg_config(database):
            connection = root / f"{database}.connection"
            settings = root / f"{database}.json"
            private_file(connection, (f"host=127.0.0.1 port={pg_port} dbname={database} "
                                      "user=riauth_drill sslmode=disable connect_timeout=3\n").encode())
            private_file(settings, json.dumps({"connection_file": str(connection),
                                               "local_unencrypted": True,
                                               "pool_size": 4}).encode())
            return settings

        try:
            version = pg("pg_ctl", "--version").stdout.strip()
            capabilities = success(["capabilities"])
            passed("local_tooling", postgres_version=version, edition=capabilities.get("edition"))

            pg("initdb", "-D", pg_data, "-U", "riauth_drill", "--auth=trust",
               "--encoding=UTF8", "--no-locale", timeout=90)
            escaped_root = str(root).replace("'", "''")
            with (pg_data / "postgresql.conf").open("a") as output:
                output.write(f"\nlisten_addresses = '127.0.0.1'\nport = {pg_port}\n"
                             f"unix_socket_directories = '{escaped_root}'\n")
            start_pg()
            for database in ("riauth_source", "riauth_target"):
                pg("createdb", "-h", "127.0.0.1", "-p", pg_port, "-U", "riauth_drill", database)
            source_pg = pg_config("riauth_source")
            target_pg = pg_config("riauth_target")
            require(sql("riauth_target", "select count(*) from pg_namespace "
                        "where nspname='riauth_store'") == "0", "target is not empty")
            passed("isolated_cluster_ready", source_database=True, target_database_empty=True,
                   loopback_only=True)

            success(["keygen", "--out", backup_key])
            success(["keygen", "--out", database_key])
            success(["keygen", "--out", wrong_key])
            require(backup_key.read_bytes() != database_key.read_bytes(), "keys are equal")
            success(["init", "--issuer", issuer, "--listen", f"127.0.0.1:{http_port}",
                     "--postgres-config", source_pg, "--database-key-file", database_key,
                     "--password-stdin"], password=admin_password + "\n", timeout=90)
            service = subprocess.Popen([str(binary), "--config", str(config), "serve"],
                                       stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL,
                                       stderr=subprocess.DEVNULL)
            wait_ready(service, issuer)
            success(["login", "admin", "--password-stdin"], remote=True,
                    password=admin_password + "\n")
            source_doctor = success(["doctor"], remote=True)
            require(source_doctor["healthy"] is True and source_doctor["storage"] == "postgresql"
                    and source_doctor["encrypted_at_rest"] is True,
                    "source PostgreSQL doctor checks failed")
            passed("source_ready", ready_http_status=200, storage="encrypted_postgresql",
                   doctor_healthy=True)
            success(["user", "create", "drill-user", "--password-stdin"], remote=True,
                    password=user_password + "\n")
            success(["login", "drill-user", "--password-stdin"], session=user_session,
                    remote=True, password=user_password + "\n")
            require(success(["whoami"], session=user_session, remote=True)["user"]["username"]
                    == "drill-user", "source login identity mismatch")
            passed("representative_source_login", username="drill-user")

            backup = success(["backup", "--key-file", backup_key, "--out", archive],
                             remote=True, timeout=120)
            require(backup["verified"] is True and archive.stat().st_size > 0,
                    "PostgreSQL backup unverified")
            passed("postgres_backup", format=backup["api_version"], verified=True,
                   bytes=archive.stat().st_size)

            stop_pg()
            require(service.poll() is None, "service exited during database outage")
            require(probe(issuer + "/livez")[0] == 200, "liveness failed during outage")
            until = time.monotonic() + 20
            while time.monotonic() < until and probe(issuer + "/readyz")[0] != 503:
                time.sleep(0.2)
            require(probe(issuer + "/readyz")[0] == 503,
                    "readiness did not fail closed during database outage")
            outage_login = refusal(["login", "drill-user", "--password-stdin"],
                                   session=root / "outage-session.json", remote=True,
                                   password=user_password + "\n", timeout=25)
            passed("database_outage_fail_closed", live_http_status=200, ready_http_status=503,
                   login=outage_login)
            stop_service(service)
            service = None
            start_pg()
            require(sql("riauth_source", "select count(*) from pg_namespace "
                        "where nspname='riauth_store'") == "1", "source store did not recover")
            passed("database_restarted", source_store_present=True)

            wrong_target = root / "wrong-key-restore"
            wrong = refusal(["restore", "--backup", archive, "--key-file", wrong_key,
                             "--out", wrong_target, "--database-key-file", database_key,
                             "--postgres-config", target_pg])
            require(not wrong_target.exists(), "wrong key created output directory")
            require(sql("riauth_target", "select count(*) from pg_namespace "
                        "where nspname='riauth_store'") == "0", "wrong key changed target database")
            passed("wrong_key_restore_rejected", **wrong, output_absent=True, target_empty=True)

            occupied_target = root / "occupied-restore"
            occupied = refusal(["restore", "--backup", archive, "--key-file", backup_key,
                                "--out", occupied_target, "--database-key-file", database_key,
                                "--postgres-config", source_pg])
            require(not occupied_target.exists(), "occupied database created output directory")
            require(sql("riauth_source", "select count(*) from pg_namespace "
                        "where nspname='riauth_store'") == "1", "occupied source database changed")
            source_status = success(["recovery", "status"])
            require(source_status["initialized"] is True and source_status["serving_allowed"] is True
                    and source_status["pending"] is None,
                    "occupied restore changed source serving state")
            passed("occupied_database_restore_rejected", **occupied, output_absent=True,
                   source_store_preserved=True, source_serving_allowed=True)

            result = success(["restore", "--backup", archive, "--key-file", backup_key,
                              "--out", restored, "--database-key-file", database_key,
                              "--postgres-config", target_pg], timeout=120)
            require(result["verified"] is True and result["storage"] == "postgresql"
                    and result["encrypted_at_rest"] is True and result["serving_allowed"] is False,
                    "PostgreSQL restore verification or gate mismatch")
            recovery_id = result["recovery"]["id"]
            require(result["recovery"]["invalidated"]["sessions"] >= 2,
                    "restored sessions were not invalidated")
            require(sql("riauth_target", "select count(*) from pg_namespace "
                        "where nspname='riauth_store'") == "1", "target store missing")
            passed("postgres_restore_verified_and_gated", verified=True, storage="postgresql",
                   encrypted_at_rest=True, serving_allowed=False, recovery_id=recovery_id,
                   invalidated_sessions=result["recovery"]["invalidated"]["sessions"])

            config = restored / "riauth.toml"
            status = success(["recovery", "status"])
            require(status["pending"]["id"] == recovery_id and status["serving_allowed"] is False,
                    "PostgreSQL recovery status mismatch")
            blocked = subprocess.run([str(binary), "--config", str(config), "--json", "serve"],
                                     capture_output=True, text=True, timeout=15)
            require(blocked.returncode != 0 and probe(issuer + "/readyz")[0] is None,
                    "gated PostgreSQL service started")
            passed("gated_serve_denied", exit_code=blocked.returncode, listener_closed=True)

            wrong_id = refusal(["recovery", "complete", "--recovery-id", "wrong-recovery",
                                "--persistent-credentials-reconciled"])
            require(success(["recovery", "status"])["pending"]["id"] == recovery_id,
                    "wrong ID changed PostgreSQL gate")
            passed("wrong_recovery_id_denied", **wrong_id, gate_preserved=True)
            no_attestation = refusal(["recovery", "complete", "--recovery-id", recovery_id])
            require(success(["recovery", "status"])["pending"]["id"] == recovery_id,
                    "missing attestation changed PostgreSQL gate")
            passed("missing_attestation_denied", **no_attestation, gate_preserved=True)

            # The fixture's two passwords were generated and verified in this run.
            completed = success(["recovery", "complete", "--recovery-id", recovery_id,
                                 "--persistent-credentials-reconciled"])
            require(completed["serving_allowed"] is True, "PostgreSQL gate did not open")
            passed("recovery_completed", serving_allowed=True,
                   synthetic_credentials_reviewed=True)

            service = subprocess.Popen([str(binary), "--config", str(config), "serve"],
                                       stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL,
                                       stderr=subprocess.DEVNULL)
            wait_ready(service, issuer)
            old_session = refusal(["whoami"], session=user_session, remote=True)
            passed("restored_session_rejected", **old_session)
            success(["login", "drill-user", "--password-stdin"], session=user_session,
                    remote=True, password=user_password + "\n")
            require(success(["whoami"], session=user_session, remote=True)["user"]["username"]
                    == "drill-user", "restored login identity mismatch")
            success(["login", "admin", "--password-stdin"], remote=True,
                    password=admin_password + "\n")
            doctor = success(["doctor"], remote=True)
            require(doctor["healthy"] is True and doctor["storage"] == "postgresql"
                    and doctor["encrypted_at_rest"] is True and doctor["users"] >= 2,
                    "restored PostgreSQL doctor checks failed")
            discovery_status, discovery = probe(issuer + "/.well-known/openid-configuration")
            keys_status, keys = probe(issuer + "/oauth/jwks")
            require(discovery_status == 200 and discovery["issuer"] == issuer,
                    "restored discovery failed")
            require(keys_status == 200 and len(keys["keys"]) > 0, "restored JWKS failed")
            passed("restored_login_and_health", ready_http_status=200, username="drill-user",
                   doctor_healthy=True, doctor_encrypted_at_rest=True,
                   discovery_http_status=discovery_status, jwks_http_status=keys_status,
                   jwks_key_count=len(keys["keys"]))
        finally:
            stop_service(service)
            if pg_running:
                stop_pg()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True, help="built riauth binary")
    parser.add_argument("--pg-bin", type=Path, help="directory with initdb, pg_ctl, createdb, psql")
    parser.add_argument("--evidence", type=Path, required=True, help="new JSON evidence file")
    args = parser.parse_args()
    binary = args.binary.resolve(strict=True)
    pg_bin = args.pg_bin or Path(shutil.which("initdb") or "").parent
    require(pg_bin.is_dir(), "PostgreSQL tool directory is unavailable")
    require(args.evidence.parent.is_dir(), "evidence parent directory does not exist")
    require(not args.evidence.exists(), "evidence output already exists")
    for tool in ("initdb", "pg_ctl", "createdb", "psql"):
        require((pg_bin / tool).is_file(), f"PostgreSQL tool unavailable: {tool}")
    evidence = {"schema_version": "riauth.postgres-recovery-drill/v1",
                "started_at": datetime.now(timezone.utc).isoformat(),
                "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
                "scope": "disposable loopback PostgreSQL cluster, real CLI and HTTP service",
                "checks": [], "result": "failed",
                "external_gates": ["lost backup or database key escrow", "referenced secret-file recovery",
                                   "PostgreSQL PITR and multi-node failover", "TLS PostgreSQL connection",
                                   "external signers, mail, directories and provisioning",
                                   "real OIDC or SAML relying party"]}
    try:
        run(binary, pg_bin, evidence)
        evidence["result"] = "passed"
    except Exception as error:
        evidence["failure"] = {"type": type(error).__name__, "message": str(error)}
    evidence["finished_at"] = datetime.now(timezone.utc).isoformat()
    private_file(args.evidence, (json.dumps(evidence, indent=2, sort_keys=True) + "\n").encode())
    print(json.dumps({"result": evidence["result"], "checks": len(evidence["checks"]),
                      "evidence": str(args.evidence)}))
    return 0 if evidence["result"] == "passed" else 1


if __name__ == "__main__":
    sys.exit(main())
