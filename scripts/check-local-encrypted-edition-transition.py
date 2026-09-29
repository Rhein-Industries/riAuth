#!/usr/bin/env python3
"""Disposable encrypted-storage handoff of installed native binaries.

The separately compiled test probe uses riAuth Store::inspect to decrypt rows.
PostgreSQL ciphertext is checked independently through SQL. This is local
source/artifact evidence, never official release provenance.
"""
import argparse
import hashlib
import importlib.util
import json
import os
import pathlib
import platform
import re
import secrets
import subprocess
import tempfile
import time


HERE = pathlib.Path(__file__).resolve().parent


def module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    loaded = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(loaded)
    return loaded


matrix = module("exact_matrix", HERE / "check-exact-edition-matrix.py")
gate = module("installed_gate", HERE / "check-installed-release-gate.py")
MUTABLE = {"meta/revision", "meta/version_activation", "meta/edition_provenance",
           "meta/node_security"}


def shared(hashes):
    return {key: value for key, value in hashes.items()
            if key not in MUTABLE and not key.startswith("meta/edition_transition_history/")}


def probe(binary, config, root, action):
    output = root / f"probe-{action}-{time.monotonic_ns()}.json"
    env = {**os.environ, "RIAUTH_PROBE_CONFIG": str(config),
           "RIAUTH_PROBE_AGREEMENT": str(root / "original-agreement.json"),
           "RIAUTH_PROBE_OUTPUT": str(output), "RIAUTH_PROBE_ACTION": action}
    result = subprocess.run([str(binary), "--exact", "isolated_store_probe", "--ignored"],
                            env=env, capture_output=True, text=True, timeout=120)
    matrix.require(result.returncode == 0 and output.is_file(),
                   f"Store API probe {action} failed: {result.stdout[-500:]} {result.stderr[-500:]}")
    return json.loads(output.read_text())


def metadata(before, after, target):
    a, b = before["metadata"], after["metadata"]
    matrix.require(a["issuer"] == b["issuer"], "issuer changed")
    for field in ("issuer", "authentication"):
        matrix.require(a["node_security"][field] == b["node_security"][field],
                       f"security agreement {field} changed")
    matrix.require(a["node_security"]["format"] == b["node_security"]["format"] == 2,
                   "security agreement format changed")
    matrix.require(a["node_security"]["active_capabilities"] !=
                   b["node_security"]["active_capabilities"], "active set did not switch")
    matrix.require(b["version_activation"]["edition"] == target and
                   b["edition_provenance"]["last_activated_edition"] == target,
                   "version/provenance not coordinated")
    matrix.require(b["revision"] > (a["revision"] or 0), "revision did not advance")


def changed_config(config, scratch, change):
    source = config.read_text()
    if change == "policy":
        revised, count = re.subn(r"(?m)^(session_ttl\s*=\s*)(\d+)$",
                                 lambda match: match[1] + str(int(match[2]) + 1), source)
    elif change == "compatible":
        revised, count = re.subn(r"(?m)^(reviewed_client_creation\s*=\s*)(true|false)$",
                                 lambda match: match[1] + ("false" if match[2] == "true" else "true"),
                                 source)
    elif change == "no-key":
        revised, count = re.subn(r"(?m)^database_key_file\s*=.*\n", "", source)
    elif change.startswith("key:"):
        revised, count = re.subn(r'(?m)^(database_key_file\s*=\s*)"[^"]+"',
                                 lambda match: match[1] + json.dumps(change[4:]), source)
    else:
        raise ValueError(change)
    matrix.require(count == 1, f"config {change} field missing")
    destination = scratch / f"config-{change.split(':')[0]}.toml"
    destination.write_text(revised)
    return destination


def expect_blocked(binary, config, target, resource=None):
    result = gate.cli(binary, "--config", config, "transition-plan", "--target", target,
                      expected=5)
    if resource:
        matrix.require(any(issue["resource"] == resource for issue in result["data"]["blockers"]),
                       f"missing blocker {resource}: {result}")
    return result


def pg_encoding(programs, port, database):
    output = matrix.successful([
        programs["psql"], "-h", "127.0.0.1", "-p", str(port), "-U", "riauth_test",
        "-d", database, "-At", "-c",
        "SELECT encoding FROM riauth_store.storage_format WHERE singleton",
    ])
    matrix.require(output.strip() == "aes256gcm-v1", "PostgreSQL store is not encrypted")
    return output.strip()


def pg_ciphertext(programs, port, database):
    output = matrix.successful([
        programs["psql"], "-h", "127.0.0.1", "-p", str(port), "-U", "riauth_test",
        "-d", database, "-At", "-F", "|", "-c",
        "SELECT encode(key,'hex'),encode(value,'hex') FROM riauth_store.records_v1 ORDER BY key",
    ])
    hashes = {}
    encrypted_metadata = False
    for line in output.splitlines():
        key_hex, value_hex = line.split("|", 1)
        key = bytes.fromhex(key_hex).decode()
        value = bytes.fromhex(value_hex)
        if key == "meta/node_security":
            try:
                json.loads(value)
            except (UnicodeDecodeError, json.JSONDecodeError):
                encrypted_metadata = True
        hashes[key] = hashlib.sha256(value).hexdigest()
    matrix.require(encrypted_metadata, "security metadata appears to be SQL plaintext")
    return hashes


def connected_pg_refusal(programs, port, database, maintenance, config, token):
    connected = subprocess.Popen([
        programs["psql"], "-h", "127.0.0.1", "-p", str(port), "-U", "riauth_test",
        "-d", database, "-c", "SELECT pg_sleep(15)",
    ], env={**os.environ, "PGAPPNAME": "riauth"}, stdout=subprocess.DEVNULL,
       stderr=subprocess.DEVNULL)
    try:
        for _ in range(100):
            count = matrix.successful([
                programs["psql"], "-h", "127.0.0.1", "-p", str(port),
                "-U", "riauth_test", "-d", database, "-At", "-c",
                "SELECT count(*) FROM pg_stat_activity WHERE application_name='riauth'",
            ]).strip()
            if int(count) > 0:
                break
            time.sleep(0.1)
        else:
            raise AssertionError("other PostgreSQL client did not connect")
        denied = gate.cli(maintenance, "--config", config, "transition-activate",
                          "--target", "platform", "--token", token, expected=5)
        matrix.require("Stop every riAuth process" in denied["error"]["message"],
                       "connected writer was not refused")
    finally:
        connected.terminate()
        connected.wait(timeout=5)
    for _ in range(200):
        count = matrix.successful([
            programs["psql"], "-h", "127.0.0.1", "-p", str(port),
            "-U", "riauth_test", "-d", database, "-At", "-c",
            "SELECT count(*) FROM pg_stat_activity WHERE application_name='riauth'",
        ]).strip()
        if int(count) == 0:
            return
        time.sleep(0.1)
    raise AssertionError("other PostgreSQL client remained connected")


def live_redb_refusal(server, maintenance, config, base, token):
    process = subprocess.Popen([str(server), "--config", str(config), "serve"],
                               stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    try:
        matrix.ready(process, base)
        refused = gate.cli(maintenance, "--config", config, "transition-activate",
                           "--target", "platform", "--token", token, expected=None)
        matrix.require(refused["error"], "live redb writer was accepted")
    finally:
        process.terminate()
        process.wait(timeout=5)


def live_identity_and_grant(server, config, base, scratch):
    """Create a real non-admin credential and active low-risk grant through CLI/HTTP."""
    session = scratch / "admin-session.json"
    grants = scratch / "auditor-grant.json"
    grants.write_text(json.dumps([{"role": "auditor", "scope": "audit/events"}]))
    prefix = ("--config", config, "--session-file", session, "--server", base,
              "--non-interactive")
    process = subprocess.Popen([str(server), "--config", str(config), "serve"],
                               stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    try:
        matrix.ready(process, base)
        gate.cli(server, *prefix, "login", "admin", "--password-stdin",
                 input="q08-disposable-password\n")
        revision = gate.cli(server, *prefix, "revision")["revision"]
        gate.cli(server, *prefix, "--if-revision", revision,
                 "--idempotency-key", secrets.token_hex(16),
                 "user", "create", "delegate", "--password-stdin",
                 input="q08-delegate-disposable-password\n")
        gate.cli(server, *prefix, "grants", "set", "delegate", "--file", grants)
        assigned = gate.cli(server, *prefix, "grants", "get", "delegate")
        matrix.require(assigned["grants"] == [{"role": "auditor", "scope": "audit/events",
                                               "target_id": "events"}],
                       "active auditor grant was not assigned")
        gate.cli(server, *prefix, "logout")
    finally:
        process.terminate()
        process.wait(timeout=5)


def run(args):
    os.umask(0o077)
    matrix.require(platform.system() == "Linux" and platform.machine() == "aarch64",
                   "native Linux aarch64 required")
    matrix.require(re.fullmatch(r"[0-9a-f]{40}", args.source_revision) is not None,
                   "source revision must be exact")
    bins = {edition: {name: args.artifacts / edition / name
                      for name in ("riauth", "riauth-maintenance")}
            for edition in ("essentials", "platform")}
    for pair in bins.values():
        matrix.require(all(path.is_file() for path in pair.values()), "installed binary missing")
    matrix.require(args.probe.is_file(), "native Store API test probe missing")
    matrix.require(not args.evidence.exists(), "evidence already exists")
    with tempfile.TemporaryDirectory(prefix=f"a08-encrypted-{args.backend}-", dir=args.evidence.parent) as temp:
        scratch = pathlib.Path(temp)
        cluster = None
        programs = {}
        port = None
        database = "riauth_encrypted_transition"
        try:
            if args.backend == "postgresql":
                cluster, reason = matrix.postgres_cluster(scratch)
                matrix.require(cluster is not None, reason or "PostgreSQL unavailable")
                cluster_programs, data_dir, port = cluster
                programs = {name: matrix.shutil.which(name)
                            for name in ("initdb", "pg_ctl", "createdb", "psql")}
                programs.update(cluster_programs)
                matrix.successful([programs["createdb"], "-h", "127.0.0.1", "-p", str(port),
                                   "-U", "riauth_test", database])
                connection = scratch / "connection"
                connection.write_text(f"host=127.0.0.1 port={port} dbname={database} user=riauth_test sslmode=disable\n")
                pg_config = scratch / "postgres.json"
                pg_config.write_text(json.dumps({"connection_file": str(connection),
                                                 "ca_file": None, "local_unencrypted": True}))
            else:
                pg_config = None
            key = scratch / "database.key"
            wrong = scratch / "wrong.key"
            gate.cli(bins["essentials"]["riauth-maintenance"], "keygen", "--out", key)
            gate.cli(bins["essentials"]["riauth-maintenance"], "keygen", "--out", wrong)
            matrix.require(key.read_bytes() != wrong.read_bytes(), "test keys equal")
            root = scratch / "instance"
            root.mkdir()
            config = root / "riauth.toml"
            http_port = matrix.free_port()
            base = f"http://127.0.0.1:{http_port}"
            init = ["--config", config, "--non-interactive", "init", "--issuer", base,
                    "--listen", f"127.0.0.1:{http_port}", "--data-dir", root / "data",
                    "--database-key-file", key, "--password-stdin"]
            if pg_config:
                init += ["--postgres-config", pg_config]
            gate.cli(bins["essentials"]["riauth-maintenance"], *init,
                     input="q08-disposable-password\n")
            first = matrix.serve(bins["essentials"]["riauth"], config, base)
            probe(args.probe, config, scratch, "fixture")
            live_identity_and_grant(bins["essentials"]["riauth"], config, base, scratch)
            before = probe(args.probe, config, scratch, "snapshot")
            matrix.require(all(before["counts"][name] > 0 for name in
                               ("identities", "credentials", "grants", "revocations")),
                           f"fixture coverage incomplete: {before['counts']}")
            matrix.require(before["counts"]["identities"] >= 2 and
                           before["counts"]["grants"] >= 2,
                           "live identity or grant fixture missing")
            if args.backend == "postgresql":
                encoding = pg_encoding(programs, port, database)
                raw_before = pg_ciphertext(programs, port, database)
            else:
                encoding = "aes256gcm-v1"
                raw_before = None
            no_key = changed_config(config, scratch, "no-key")
            wrong_key = changed_config(config, scratch, f"key:{wrong}")
            for candidate, expected in ((no_key, "Database encryption configuration does not match"),
                                        (wrong_key, "Encrypted data authentication failed")):
                refused = gate.cli(bins["platform"]["riauth-maintenance"], "--config", candidate,
                                   "transition-plan", "--target", "platform", expected=None)
                matrix.require(expected in refused["error"]["message"],
                               f"missing/wrong storage key refusal differed: {refused['error']['code']}")
            for mutation in ("missing-agreement", "old-agreement", "new-agreement", "shared-drift"):
                probe(args.probe, config, scratch, mutation)
                expect_blocked(bins["platform"]["riauth-maintenance"], config, "platform",
                               "meta/node_security")
                probe(args.probe, config, scratch, "restore-agreement")
            maintenance = bins["platform"]["riauth-maintenance"]
            refused = gate.cli(bins["platform"]["riauth"], "--config", config,
                               "serve", expected=None)
            matrix.require("active capabilities" in refused["error"]["message"],
                           "direct wrong build open did not refuse")
            plan = gate.cli(maintenance, "--config", config, "transition-plan", "--target", "platform")
            token = plan["transition_token"]
            matrix.require(plan["ready"] and token, "upgrade plan unavailable")
            probe(args.probe, config, scratch, "store-drift")
            gate.cli(maintenance, "--config", config, "transition-activate", "--target", "platform",
                     "--token", token, expected=5)
            probe(args.probe, config, scratch, "remove-drift")
            changed = changed_config(config, scratch, "compatible")
            stale_config = gate.cli(maintenance, "--config", changed, "transition-activate",
                                    "--target", "platform", "--token", token, expected=5)
            matrix.require("token" in stale_config["error"]["message"].lower(),
                           "compatible config change did not invalidate the plan token")
            policy = changed_config(config, scratch, "policy")
            policy_blocked = gate.cli(maintenance, "--config", policy, "transition-activate",
                                      "--target", "platform", "--token", token, expected=5)
            matrix.require("meta/node_security" in policy_blocked["error"]["message"],
                           "authentication-policy drift did not block activation")
            if args.backend == "postgresql":
                connected_pg_refusal(programs, port, database, maintenance, config, token)
            else:
                live_redb_refusal(bins["essentials"]["riauth"], maintenance, config, base, token)
            upgrade = gate.cli(maintenance, "--config", config, "transition-activate",
                               "--target", "platform", "--token", token)
            matrix.require(upgrade["activated_edition"] == "platform", "upgrade activation failed")
            upgraded = probe(args.probe, config, scratch, "snapshot")
            matrix.require(shared(before["row_hashes"]) == shared(upgraded["row_hashes"]),
                           "logical rows changed on upgrade")
            metadata(before, upgraded, "platform")
            if raw_before is not None:
                raw_upgraded = pg_ciphertext(programs, port, database)
                matrix.require(shared(raw_before) == shared(raw_upgraded),
                               "ciphertext rows changed on upgrade")
            second = matrix.serve(bins["platform"]["riauth"], config, base)
            before_down = probe(args.probe, config, scratch, "snapshot")
            raw_before_down = (pg_ciphertext(programs, port, database)
                               if args.backend == "postgresql" else None)
            refused = gate.cli(bins["essentials"]["riauth"], "--config", config,
                               "serve", expected=None)
            matrix.require(refused["error"], "direct downgrade was accepted")
            down_plan = gate.cli(maintenance, "--config", config,
                                 "transition-plan", "--target", "essentials")
            matrix.require(down_plan["ready"] and down_plan["transition_token"],
                           "downgrade plan unavailable")
            down = gate.cli(maintenance, "--config", config, "transition-activate",
                            "--target", "essentials", "--token", down_plan["transition_token"])
            matrix.require(down["activated_edition"] == "essentials", "downgrade activation failed")
            returned = probe(args.probe, config, scratch, "snapshot")
            matrix.require(shared(before_down["row_hashes"]) == shared(returned["row_hashes"]),
                           "logical rows changed on downgrade")
            metadata(before_down, returned, "essentials")
            matrix.require(before["metadata"]["node_security"]["active_capabilities"] ==
                           returned["metadata"]["node_security"]["active_capabilities"],
                           "original active set not restored")
            if raw_before is not None:
                matrix.require(shared(raw_before_down) == shared(pg_ciphertext(programs, port, database)),
                               "ciphertext rows changed on downgrade")
            third = matrix.serve(bins["essentials"]["riauth"], config, base)
            report = {
                "schema": "riauth.local-encrypted-edition-transition/v1",
                "result": "passed", "release_gate_result": False,
                "source_revision": args.source_revision, "architecture": "linux/aarch64",
                "backend": args.backend, "storage_encoding": encoding,
                "binary_sha256": {edition: {name: gate.digest(path) for name, path in pair.items()}
                                  for edition, pair in bins.items()},
                "probe_sha256": gate.digest(args.probe),
                "validator_sha256": gate.digest(pathlib.Path(__file__)),
                "fixture_counts": before["counts"],
                "logical_preserved_rows_upgrade": len(shared(before["row_hashes"])),
                "logical_preserved_rows_downgrade": len(shared(before_down["row_hashes"])),
                "postgres_ciphertext_preserved_rows": len(shared(raw_before)) if raw_before else None,
                "checks": ["missing_and_wrong_database_key_refused", "missing_old_new_and_shared_drift_agreement_refused",
                           "stale_full_store_plan_refused", "compatible_config_token_refused",
                           "authentication_policy_config_drift_refused",
                           "wrong_build_direct_open_refused", "target_metadata_atomic",
                           "issuer_auth_policy_preserved", "logical_identity_credential_grant_revocation_preserved",
                           "live_user_credential_auditor_grant_and_logout",
                           "postgres_raw_ciphertext_preserved" if raw_before else "redb_encrypted_store_api_verified",
                           "postgres_other_client_refused" if raw_before else "redb_live_writer_refused"],
                "editions": [first["edition"], second["edition"], third["edition"]],
            }
            with args.evidence.open("x") as output:
                json.dump(report, output, sort_keys=True, indent=2)
                output.write("\n")
            print(json.dumps({"result": "passed", "backend": args.backend,
                              "evidence": str(args.evidence)}))
        finally:
            if cluster is not None:
                matrix.successful([programs["pg_ctl"], "-D", str(data_dir),
                                   "stop", "-m", "immediate", "-w"])


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--backend", choices=("redb", "postgresql"), required=True)
    parser.add_argument("--artifacts", type=pathlib.Path, required=True)
    parser.add_argument("--probe", type=pathlib.Path, required=True)
    parser.add_argument("--evidence", type=pathlib.Path, required=True)
    parser.add_argument("--source-revision", required=True)
    run(parser.parse_args())
