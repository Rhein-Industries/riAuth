#!/usr/bin/env python3
"""Disposable PG16 base-backup recovery drill; never operates on an existing cluster.

Build `riauth` and `--example recovery_native_artifacts` in a private Cargo target.
The output directory contains only redacted reports. All credential fixtures and
the cluster live in a new mode-0700 temporary directory and are reaped on exit.
"""

import argparse
import hashlib
import json
import os
import secrets
import shutil
import socket
import subprocess
import tempfile
import time
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path


def require(value, message):
    if not value:
        raise RuntimeError(message)


def free_port():
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        return listener.getsockname()[1]


def port_closed(port):
    with socket.socket() as probe:
        probe.settimeout(0.5)
        return probe.connect_ex(("127.0.0.1", port)) != 0


def http_status(url):
    try:
        with urllib.request.urlopen(url, timeout=1) as response:
            return response.status
    except urllib.error.HTTPError as error:
        return error.code
    except (urllib.error.URLError, TimeoutError, ConnectionError):
        return None


def private_json(path, content):
    with os.fdopen(os.open(path, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600), "w") as output:
        json.dump(content, output, indent=2, sort_keys=True)
        output.write("\n")


def private_text(path, content):
    with os.fdopen(os.open(path, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600), "w") as output:
        output.write(content)


def redact(text, sensitive):
    for value in sensitive:
        if value:
            text = text.replace(value, "[redacted]")
    return text


def postgres_env(password=None):
    environment = {key: value for key, value in os.environ.items()
                   if not key.startswith("PG") and key != "PSQLRC"}
    if password is not None:
        environment["PGPASSWORD"] = password
    return environment


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main(binary, fixture, pg_bin, out):
    require(not out.exists(), "output directory must not exist")
    require(binary.is_file() and fixture.is_file(), "private Cargo binaries required")
    require(all((pg_bin / name).is_file() for name in
                ("initdb", "pg_ctl", "psql", "pg_basebackup", "pg_verifybackup", "createdb")),
            "PG16 tooling incomplete")
    require("16.14" in subprocess.check_output([pg_bin / "pg_ctl", "--version"], text=True,
                                               env=postgres_env()),
            "exact PG16.14 tooling required")
    require(shutil.disk_usage(out.parent).free >= 8 * 1024**3, "8 GiB free-space floor not met")
    out.mkdir(mode=0o700)
    root = Path(tempfile.mkdtemp(prefix="riauth-native-physical-"))
    os.chmod(root, 0o700)
    report = {"scope": {"date_utc": datetime.now(timezone.utc).isoformat(),
                        "operation": "physical_base_backup", "postgres": "16.14",
                        "isolated_loopback": True, "external_host": False,
                        "escrow_or_relying_party_proof": False,
                        "binary_sha256": sha256(binary),
                        "fixture_sha256": sha256(fixture)},
              "checks": [], "outcome": "running", "open_gates": []}
    pg_port, http_port = free_port(), free_port()
    require(pg_port != http_port, "port collision")
    issuer = f"http://127.0.0.1:{http_port}"
    source, copy = root / "source", root / "copy"
    config, session = root / "riauth.toml", root / "admin-session.json"
    running_pg = None
    service = None
    holder = None
    cleanup_ok = True
    sensitive = []
    pg_password = secrets.token_hex(24)
    sensitive.append(pg_password)
    password_file = root / "pg-password"

    def flush():
        path = out / "report.json"
        temporary = out / "report.json.tmp"
        temporary.write_text(redact(json.dumps(report, indent=2, sort_keys=True), sensitive) + "\n")
        os.chmod(temporary, 0o600)
        temporary.replace(path)

    def check(name, **observed):
        report["checks"].append({"id": name, "result": "passed", "observed": observed})
        flush()

    def pg(name, *args, ok=True, timeout=90):
        result = subprocess.run([str(pg_bin / name), *map(str, args)],
                                text=True, capture_output=True, timeout=timeout,
                                env=postgres_env(pg_password))
        result.stderr = redact(result.stderr, sensitive)
        if ok:
            require(result.returncode == 0,
                    f"{name} exit {result.returncode}")
        return result

    def sql(query, database="riauth_drill"):
        return pg("psql", "--no-password", "-X", "-A", "-t", "-h", "127.0.0.1", "-p", pg_port,
                  "-U", "riauth_drill", "-d", database, "-c", query).stdout.strip()

    def cli(args, cfg=config, password=None, remote=False, timeout=60):
        env = os.environ.copy()
        for key in ("RIAUTH_SERVER", "RIAUTH_CONFIG", "RIAUTH_SESSION_FILE", "RIAUTH_AGENT_FILE",
                    "RIAUTH_PASSWORD", "RIAUTH_OTP", "RIAUTH_RUN_ID"):
            env.pop(key, None)
        argv = [str(binary), "--config", str(cfg), "--session-file", str(session),
                "--json", "--non-interactive"]
        if remote:
            argv += ["--server", issuer]
        result = subprocess.run(argv + list(map(str, args)), input=password, text=True,
                                capture_output=True, timeout=timeout, env=env)
        try:
            envelope = json.loads(result.stdout)
        except json.JSONDecodeError:
            envelope = {}
        return result.returncode, envelope

    def success(args, **options):
        code, envelope = cli(args, **options)
        require(code == 0 and envelope.get("ok") is True,
                f"CLI {args[:2]} exit {code}, error {envelope.get('error', {}).get('code')}")
        return envelope["data"]

    def refusal(args, **options):
        code, envelope = cli(args, **options)
        require(code != 0 and envelope.get("ok") is False,
                f"CLI {args[:2]} did not give structured refusal (exit {code})")
        return {"exit_code": code, "error_code": envelope["error"]["code"]}

    def fixture_run(*args):
        result = subprocess.run([str(fixture), *map(str, args)], text=True,
                                capture_output=True, timeout=60)
        require(result.returncode == 0,
                f"fixture {args[0]} exit {result.returncode}")

    def pg_start(data):
        nonlocal running_pg
        require(running_pg is None, "another drill PostgreSQL directory is running")
        pg("pg_ctl", "-D", data, "-l", root / "postgres.log", "start", "-w")
        running_pg = data

    def pg_stop():
        nonlocal running_pg
        if running_pg is not None:
            pg("pg_ctl", "-D", running_pg, "stop", "-m", "immediate", "-w")
            running_pg = None

    def stop_service():
        nonlocal service
        if service is not None and service.poll() is None:
            service.terminate()
            try:
                service.wait(timeout=8)
            except subprocess.TimeoutExpired:
                service.kill()
                service.wait(timeout=4)
        service = None

    def fence():
        require(service is None or service.poll() is not None,
                "fence refuses physical restore while a riAuth writer is running")

    def lineage():
        return {"system_identifier": sql("select system_identifier from pg_control_system()"),
                "database_oid": sql("select oid from pg_database where datname='riauth_drill'", "postgres"),
                "records_oid": sql("select 'riauth_store.records_v1'::regclass::oid")}

    try:
        private_text(password_file, pg_password + "\n")
        pg("initdb", "-D", source, "-U", "riauth_drill", "--auth=scram-sha-256",
           "--pwfile", password_file,
           "--encoding=UTF8", "--no-locale")
        escaped = str(root).replace("'", "''")
        with (source / "postgresql.conf").open("a") as config_file:
            config_file.write(f"\nlisten_addresses = '127.0.0.1'\nport = {pg_port}\n"
                              f"unix_socket_directories = '{escaped}'\n"
                              "wal_level = replica\nmax_wal_senders = 4\n")
        pg_start(source)
        pg("createdb", "--no-password", "-h", "127.0.0.1", "-p", pg_port,
           "-U", "riauth_drill", "riauth_drill")
        pg("createdb", "--no-password", "-h", "127.0.0.1", "-p", pg_port,
           "-U", "riauth_drill", "empty_schema")
        connection = root / "connection"
        private_text(connection, (f"host=127.0.0.1 port={pg_port} dbname=riauth_drill "
                                  f"user=riauth_drill password={pg_password} "
                                  "sslmode=disable connect_timeout=3\n"))
        postgres_config = root / "postgres.json"
        private_json(postgres_config, {"connection_file": str(connection),
                                       "local_unencrypted": True, "pool_size": 4})
        database_key, wrong_key = root / "database.key", root / "wrong.key"
        success(["keygen", "--out", database_key])
        sensitive.append(database_key.read_text().strip())
        success(["keygen", "--out", wrong_key])
        sensitive.append(wrong_key.read_text().strip())
        admin_password = secrets.token_urlsafe(30)
        input_file = root / "input.json"
        inputs = {"admin_password": admin_password,
                  "user_password": secrets.token_urlsafe(30),
                  "new_user_password": secrets.token_urlsafe(30),
                  "late_user_password": secrets.token_urlsafe(30)}
        sensitive.extend(inputs.values())
        private_json(input_file, inputs)
        success(["init", "--issuer", issuer, "--listen", f"127.0.0.1:{http_port}",
                 "--postgres-config", postgres_config, "--database-key-file", database_key,
                 "--password-stdin"], password=admin_password + "\n")
        artifacts, post, verified = root / "artifacts.json", root / "post.json", root / "verified.json"
        fixture_run("issue", config, input_file, artifacts)
        sensitive.extend(json.loads(artifacts.read_text())[key] for key in
                         ("session", "access", "refresh", "code", "verifier", "proof"))
        check("issued_fixture", session=True, bearer=True, refresh=True,
              authorization_code=True, one_use_proof=True)
        service = subprocess.Popen([str(binary), "--config", str(config), "serve"],
                                   stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL,
                                   stderr=subprocess.DEVNULL)
        until = time.monotonic() + 25
        while time.monotonic() < until and http_status(issuer + "/readyz") != 200:
            require(service.poll() is None, "source service exited before readiness")
            time.sleep(0.1)
        require(http_status(issuer + "/readyz") == 200, "source readiness timed out")
        success(["login", "admin", "--password-stdin"], remote=True,
                password=admin_password + "\n")
        check("source_representative_login", readiness=200, username="admin")
        try:
            fence()
            raise RuntimeError("fence wrongly accepted a live writer")
        except RuntimeError as error:
            require(str(error).startswith("fence refuses"), "unexpected fence result")
        check("live_writer_fence_refusal", writer_running=True)
        stop_service()
        require(port_closed(http_port), "source listener survived writer stop")
        snapshot_count = int(sql("select count(*) from riauth_store.records_v1"))
        snapshot_lineage = lineage()
        occupied = root / "occupied"
        occupied.mkdir()
        (occupied / "marker").write_text("preserve")
        denied = pg("pg_basebackup", "-D", occupied, "-h", "127.0.0.1", "-p", pg_port,
                    "-U", "riauth_drill", "-X", "stream", "-c", "fast", "--no-password",
                    ok=False)
        require(denied.returncode != 0 and (occupied / "marker").read_text() == "preserve",
                "occupied target was not refused intact")
        check("occupied_target_refusal", exit_code=denied.returncode, marker_preserved=True)
        fence()
        pg("pg_basebackup", "-D", copy, "-h", "127.0.0.1", "-p", pg_port,
           "-U", "riauth_drill", "-X", "stream", "-c", "fast", "--no-password", timeout=180)
        require((copy / "backup_manifest").is_file(), "backup manifest missing")
        pg("pg_verifybackup", copy)
        check("physical_base_backup", verified=True, snapshot_records=snapshot_count,
              lineage=snapshot_lineage)
        fixture_run("post", config, input_file, post)
        post_count = int(sql("select count(*) from riauth_store.records_v1"))
        require(post_count > snapshot_count, "post-snapshot mutations did not increase records")
        check("lost_timeline_mutations", post_records=post_count,
              password_rotated=True, signing_key_rotated=True, late_user_created=True)
        fence()
        pg_stop()
        require(pg("pg_ctl", "-D", source, "status", ok=False).returncode != 0,
                "source postmaster still running")
        require(port_closed(pg_port), "source PostgreSQL listener still occupied")
        check("source_stopped_and_fenced", source_postmaster_stopped=True, port_closed=True)
        pg_start(copy)
        require(int(sql("select count(*) from riauth_store.records_v1")) == snapshot_count,
                "restored record count differs from base backup")
        require(lineage() == snapshot_lineage, "physical restore lineage unexpectedly changed")
        before = success(["recovery", "status"])
        require(before["serving_allowed"] is True and before["pending"] is None,
                "expected same-lineage blind spot was not observed")
        private_json(out / "status-before-invalidation.json", before)
        check("same_lineage_blind_spot", restored_records=snapshot_count,
              lineage_equal=True, status_says_serving_allowed=True,
              listener_started=False)
        wrong_config = root / "wrong-key.toml"
        original_config = config.read_text()
        require(original_config.count(str(database_key)) == 1, "database key path ambiguous")
        wrong_config.write_text(original_config.replace(str(database_key), str(wrong_key)))
        os.chmod(wrong_config, 0o600)
        wrong = refusal(["recovery", "status"], cfg=wrong_config)
        check("wrong_database_key_refusal", **wrong)
        empty_connection = root / "empty.connection"
        empty_connection.write_text(connection.read_text().replace("dbname=riauth_drill",
                                                                 "dbname=empty_schema"))
        os.chmod(empty_connection, 0o600)
        require(original_config.count(str(connection)) == 1, "connection-file path ambiguous")
        empty_config = root / "empty.toml"
        empty_config.write_text(original_config.replace(str(connection), str(empty_connection)))
        os.chmod(empty_config, 0o600)
        empty_status = success(["recovery", "status"], cfg=empty_config)
        require(empty_status["initialized"] is False and empty_status["serving_allowed"] is False,
                "empty schema was not fail closed")
        check("empty_schema_refusal", initialized=False, serving_allowed=False)
        ready = root / "holder.ready"
        holder = subprocess.Popen([str(fixture), "hold", str(config), str(ready)],
                                  stdin=subprocess.PIPE, stdout=subprocess.DEVNULL,
                                  stderr=subprocess.DEVNULL, text=True)
        until = time.monotonic() + 15
        while time.monotonic() < until and not ready.exists():
            require(holder.poll() is None, "holder exited before ready")
            time.sleep(0.05)
        require(ready.exists(), "holder readiness timed out")
        held = refusal(["recovery", "invalidate", "--database-restored"])
        require(held["error_code"] == "conflict", "current writer refusal was not conflict")
        require(success(["recovery", "status"])["pending"] is None,
                "denied invalidate changed the gate")
        holder.stdin.write("release\n")
        holder.stdin.flush()
        require(holder.wait(timeout=10) == 0, "holder did not release")
        holder = None
        check("current_writer_refusal", **held, gate_unchanged=True)
        invalidated = success(["recovery", "invalidate", "--database-restored"])
        private_json(out / "invalidate.json", invalidated)
        after = success(["recovery", "status"])
        private_json(out / "status-after-invalidation.json", after)
        require(after["serving_allowed"] is False and after["pending"] is not None,
                "invalidation did not close gate")
        require(after["pending"]["reconcile"]["signing_keys"] >= 1,
                "restored signing key absent from reconciliation")
        check("offline_invalidation", serving_allowed=False,
              pending_id=after["pending"]["id"],
              invalidated=after["pending"]["invalidated"],
              reconcile=after["pending"]["reconcile"])
        fixture_run("verify", config, artifacts, post, verified)
        artifact_result = json.loads(verified.read_text())
        private_json(out / "artifact-refusals.json", artifact_result)
        check("restored_artifact_refusals", **artifact_result)
        service = subprocess.Popen([str(binary), "--config", str(config), "serve"],
                                   stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL,
                                   stderr=subprocess.DEVNULL)
        require(service.wait(timeout=12) != 0 and port_closed(http_port),
                "unreconciled service opened a listener")
        stop_service()
        check("unreconciled_serve_refusal", listener_closed=True)
        wrong_id = refusal(["recovery", "complete", "--recovery-id", "wrong-recovery-id",
                            "--persistent-credentials-reconciled"])
        no_attestation = cli(["recovery", "complete", "--recovery-id", after["pending"]["id"]])
        require(no_attestation[0] != 0, "missing attestation was accepted")
        final = success(["recovery", "status"])
        require(final["pending"]["id"] == after["pending"]["id"]
                and final["serving_allowed"] is False and port_closed(http_port),
                "gate reopened without reconciliation")
        private_json(out / "status-final.json", final)
        check("completion_gates", wrong_id=wrong_id, missing_attestation_exit=no_attestation[0],
              gate_closed=True, listener_closed=True)
        reports = "\n".join(path.read_text() for path in out.glob("*.json"))
        require(all(value not in reports for value in sensitive),
                "a generated credential or key appeared in a durable report")
        check("report_redaction", generated_credentials_and_keys_absent=True)
        report["outcome"] = "passed_gate_closed"
        report["open_gates"] = [
            "Post-snapshot password and signing-key changes cannot be reconstructed from this base backup; no recovery completion attestation was made.",
            "External credential/secret escrow, real host and relying-party login, and deployed HA/outage behavior require separate authorized setup and evidence.",
        ]
    except Exception as error:
        report["outcome"] = "failed"
        report["failure"] = redact(str(error), sensitive)[:500]
        raise RuntimeError(report["failure"]) from None
    finally:
        if holder is not None:
            try:
                holder.stdin.write("release\n")
                holder.stdin.flush()
                holder.wait(timeout=5)
            except Exception:
                holder.kill()
                holder.wait(timeout=5)
        stop_service()
        if running_pg is not None:
            try:
                pg_stop()
            except Exception as error:
                cleanup_ok = False
                report["cleanup_error"] = redact(str(error), sensitive)[:300]
        for data in (source, copy):
            if data.exists() and pg("pg_ctl", "-D", data, "status", ok=False).returncode == 0:
                try:
                    pg("pg_ctl", "-D", data, "stop", "-m", "immediate", "-w")
                except Exception as error:
                    cleanup_ok = False
                    report["cleanup_error"] = redact(str(error), sensitive)[:300]
        if cleanup_ok:
            shutil.rmtree(root)
        else:
            report["private_workspace_preserved"] = str(root)
        report["cleanup"] = {"postgres_stopped": cleanup_ok,
                             "private_workspace_removed": cleanup_ok,
                             "service_stopped": True}
        if not cleanup_ok:
            report["outcome"] = "failed_cleanup"
        flush()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--fixture", type=Path, required=True)
    parser.add_argument("--pg-bin", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    arguments = parser.parse_args()
    main(arguments.binary.resolve(), arguments.fixture.resolve(),
         arguments.pg_bin.resolve(), arguments.out.resolve())
