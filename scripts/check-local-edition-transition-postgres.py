#!/usr/bin/env python3
"""Exercise local native PostgreSQL edition handoffs; never certify release assets.

Run as an unprivileged Linux user with initdb, pg_ctl, createdb and psql on PATH.
The artifact directory contains essentials/ and platform/ server and maintenance
binaries. The disposable cluster, connection file and sessions stay in a private
temporary directory. Only hashes and redacted results leave that directory.
"""

import argparse
import base64
import hashlib
import importlib.util
import json
import os
import pathlib
import platform
import re
import subprocess
import tempfile
import time
import tomllib
import urllib.error
import urllib.request


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


ROOT = pathlib.Path(__file__).resolve().parent
matrix = load("exact_matrix", ROOT / "check-exact-edition-matrix.py")
gate = load("installed_gate", ROOT / "check-installed-release-gate.py")
encrypted_fixture = load("encrypted_fixture", ROOT / "check-local-encrypted-edition-transition.py")
RATE_NAMES = frozenset(("portal_start", "portal_approve", "login", "passkey", "account",
                        "source_start", "source_callback", "saml", "mfa", "device_start",
                        "device_verify", "browser_decision", "browser_state",
                        "forward_auth", "outpost_start", "general"))
EXCLUDED = {
    "meta/revision", "meta/version_activation", "meta/edition_provenance",
    "meta/node_security",
}
TRANSITION_METADATA = {
    "meta/issuer", "meta/node_security", "meta/version_activation",
    "meta/edition_provenance",
}


def rows(programs, port, database):
    output = subprocess.check_output([
        programs["psql"], "-h", "127.0.0.1", "-p", str(port),
        "-U", "riauth_test", "-d", database, "-At", "-F", "|", "-c",
        "SELECT encode(key,'hex'),encode(value,'hex') FROM riauth_store.records_v1 ORDER BY key",
    ], text=True)
    result = {}
    for line in output.splitlines():
        key_hex, value_hex = line.split("|", 1)
        name = bytes.fromhex(key_hex).decode()
        if name not in EXCLUDED and not name.startswith("meta/edition_transition_history/"):
            result[name] = hashlib.sha256(bytes.fromhex(value_hex)).hexdigest()
    return result


def transition_metadata(programs, port, database):
    output = subprocess.check_output([
        programs["psql"], "-h", "127.0.0.1", "-p", str(port),
        "-U", "riauth_test", "-d", database, "-At", "-F", "|", "-c",
        "SELECT encode(key,'hex'),encode(value,'hex') FROM riauth_store.records_v1 ORDER BY key",
    ], text=True)
    result = {}
    for line in output.splitlines():
        key_hex, value_hex = line.split("|", 1)
        name = bytes.fromhex(key_hex).decode()
        if name in TRANSITION_METADATA:
            result[name] = json.loads(bytes.fromhex(value_hex))
    matrix.require(set(result) == TRANSITION_METADATA, "transition metadata is incomplete")
    matrix.require(result["meta/node_security"]["format"] == 3,
                   "transition requires a current format-3 security agreement")
    rates = result["meta/node_security"]["effective_rate_limits"]
    matrix.require(set(rates) == RATE_NAMES and
                   all(type(value) is int and 1 <= value <= 100_000 for value in rates.values()),
                   "format-3 effective rate map is incomplete or invalid")
    return result


def require_target_metadata(before, after, target):
    matrix.require(after["meta/issuer"] == before["meta/issuer"],
                   "transition changed the issuer")
    for field in ("issuer", "authentication", "effective_rate_limits"):
        matrix.require(after["meta/node_security"][field] == before["meta/node_security"][field],
                       f"transition changed security agreement {field}")
    matrix.require(after["meta/node_security"]["active_capabilities"] !=
                   before["meta/node_security"]["active_capabilities"],
                   "target active capabilities were not switched")
    matrix.require(after["meta/version_activation"]["edition"] == target and
                   after["meta/edition_provenance"]["last_activated_edition"] == target,
                   "target edition metadata was not coordinated")


def require_preserved(before, after, direction):
    changed = sorted(set(before).symmetric_difference(after) |
                     {key for key, value in before.items() if after.get(key) != value})
    matrix.require(not changed, f"{direction} changed or removed stored rows: {changed}")


def authenticated_status(base, path, token):
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
    request = urllib.request.Request(base + path, headers={"Authorization": "Bearer " + token})
    try:
        with opener.open(request, timeout=5) as response:
            response.read()
            return response.status
    except urllib.error.HTTPError as error:
        error.read()
        return error.code


def ordinary_fixture(server, config, base, scratch):
    # Reuse only the live fixture; never the historical format-2 metadata/run/probe.
    encrypted_fixture.live_identity_and_grant(server, config, base, scratch)
    session = scratch / "group-admin-session.json"
    with gate.serving(server, config, base, scratch / "group-fixture.log"):
        gate.remote(server, base, session, "login", "admin", "--password-stdin",
                    input="q08-disposable-password\n")
        for command in (("group", "create", "shared-fixture"),
                        ("group", "add-member", "shared-fixture", "delegate")):
            revision = gate.remote(server, base, session, "revision")["revision"]
            gate.cli(server, "--server", base, "--session-file", session, "--non-interactive",
                     "--if-revision", revision, "--idempotency-key", os.urandom(16).hex(), *command)
        gate.remote(server, base, session, "logout")


def refusal_snapshot_counts(before, after):
    labels = ("http_rates", "http_rate_expiry", "http_rate_count",
              "maintenance_cursors", "maintenance_bounds", "protected_or_other")
    fields = ("before", "after", "added", "changed", "removed")

    def read(raw):
        if type(raw) is not bytes or len(raw) > 8 * 1024 ** 2 or raw and not raw.endswith(b"\n"):
            raise ValueError("refusal_snapshot_shape")
        result, cursor = {}, 0
        while cursor < len(raw):
            end = raw.find(b"\n", cursor)
            split = raw.find(b"|", cursor, end)
            key_size = split - cursor
            value_size = end - split - 1
            if (end < 0 or split < 0 or not 2 <= key_size <= 8192 or key_size % 2
                    or not 0 <= value_size <= 2 * 1024 ** 2 or value_size % 2
                    or len(result) >= 1_000_000):
                raise ValueError("refusal_snapshot_shape")
            key_hex, value_hex = raw[cursor:split], raw[split + 1:end]
            if re.fullmatch(rb"[0-9a-f]+", key_hex) is None or re.fullmatch(rb"[0-9a-f]*", value_hex) is None:
                raise ValueError("refusal_snapshot_shape")
            key = bytes.fromhex(key_hex.decode("ascii"))
            key.decode("utf-8")
            bucket, separator, _ = key.partition(b"/")
            if not bucket or not separator or b"\x00" in key or key in result:
                raise ValueError("refusal_snapshot_shape")
            result[key] = value_hex
            cursor = end + 1
        return result

    def category(key):
        if key.startswith(b"http_rates/"):
            return "http_rates"
        if key.startswith(b"index_expiry_http_rates/"):
            return "http_rate_expiry"
        if key == b"index_counts/http_rates":
            return "http_rate_count"
        if key.startswith(b"maintenance_cursors/"):
            return "maintenance_cursors"
        if key.startswith(b"maintenance_bounds/"):
            return "maintenance_bounds"
        return "protected_or_other"

    before_rows, after_rows = read(before), read(after)
    result = {label: {field: 0 for field in fields} for label in labels}
    for key, value in before_rows.items():
        counts = result[category(key)]
        counts["before"] += 1
        if key not in after_rows:
            counts["removed"] += 1
        elif after_rows[key] != value:
            counts["changed"] += 1
    for key in after_rows:
        counts = result[category(key)]
        counts["after"] += 1
        if key not in before_rows:
            counts["added"] += 1
    return result


def refusal_admission_snapshots(before, lower_ns, upper_ns, elapsed_ns, limit):
    # Derive complete expected records from BEFORE only; never interpret AFTER.
    if (any(type(value) is not int for value in (lower_ns, upper_ns, elapsed_ns, limit))
            or not 0 <= lower_ns <= upper_ns <= (2 ** 64 - 1) * 1_000_000_000
            or not 0 <= elapsed_ns <= 5_000_000_000
            or upper_ns - lower_ns > 6_000_000_000
            or abs(upper_ns - lower_ns - elapsed_ns) > 250_000_000
            or not 1 <= limit <= 100_000):
        raise ValueError("refusal_admission_prerequisite")
    refusal_snapshot_counts(before, before)  # Existing bounded, strict hex/duplicate parser.
    rows = {}
    for line in before.split(b"\n")[:-1]:
        key_hex, value_hex = line.split(b"|", 1)
        rows[bytes.fromhex(key_hex.decode("ascii"))] = value_hex

    def pack(records):
        raw = b"".join(key.hex().encode("ascii") + b"|" + records[key] + b"\n"
                       for key in sorted(records))
        if len(raw) > 8 * 1024 ** 2:
            raise ValueError("refusal_admission_prerequisite")
        return raw

    if pack(rows) != before:
        raise ValueError("refusal_admission_prerequisite")
    rate_id = base64.urlsafe_b64encode(hashlib.sha256(b"127.0.0.1\x00general").digest()).rstrip(b"=")
    rate_key = b"http_rates/" + rate_id
    rate_hex = rows.get(rate_key)
    if rate_hex is None:
        raise ValueError("refusal_admission_prerequisite")
    rate = re.fullmatch(rb"\[(0|[1-9][0-9]{0,19}),(0|[1-9][0-9]{0,9})\]",
                        bytes.fromhex(rate_hex.decode("ascii")))
    if rate is None:
        raise ValueError("refusal_admission_prerequisite")
    start, count = int(rate[1]), int(rate[2])
    lower, upper = lower_ns // 1_000_000_000, upper_ns // 1_000_000_000
    if not 0 <= start <= lower or not 1 <= count <= 2 ** 32 - 1:
        raise ValueError("refusal_admission_prerequisite")
    total = sum(key.startswith(b"http_rates/") for key in rows)
    if (not 1 <= total <= 100_000
            or rows.get(b"index_counts/http_rates") != str(total).encode("ascii").hex().encode("ascii")):
        raise ValueError("refusal_admission_prerequisite")
    index_id = base64.urlsafe_b64encode(hashlib.sha256(rate_id).digest()).rstrip(b"=")
    id_hex = (b'"' + rate_id + b'"').hex().encode("ascii")

    def expiry_key(at):
        return (b"index_expiry_http_rates/"
                + f"{min(at + 60, 2 ** 64 - 1):020}/".encode("ascii") + index_id)

    old_expiry = expiry_key(start)
    if (rows.get(old_expiry) != id_hex
            or sum(key.startswith(b"index_expiry_http_rates/") and value == id_hex
                   for key, value in rows.items()) != 1):
        raise ValueError("refusal_admission_prerequisite")
    seen = set()
    for at in range(lower, upper + 1):  # At most seven independently bracketed Unix seconds.
        next_start, prior_count = (start, count) if min(start + 60, 2 ** 64 - 1) > at else (at, 0)
        if prior_count >= limit:  # Such a transaction returns 429, never the required 403.
            continue
        next_count = min(prior_count + 1, 2 ** 32 - 1)
        if (next_start, next_count) in seen:
            continue
        seen.add((next_start, next_count))
        new_expiry = expiry_key(next_start)
        if new_expiry != old_expiry and new_expiry in rows:
            raise ValueError("refusal_admission_prerequisite")
        expected = rows.copy()
        expected[rate_key] = f"[{next_start},{next_count}]".encode("ascii").hex().encode("ascii")
        del expected[old_expiry]
        expected[new_expiry] = id_hex
        yield pack(expected)


def shared_probe(server, config, base, scratch, revoked_token=None):
    session = scratch / f"delegate-{time.monotonic_ns()}.json"
    admin = scratch / f"admin-{time.monotonic_ns()}.json"
    with gate.serving(server, config, base, scratch / f"shared-{time.monotonic_ns()}.log"):
        if revoked_token is not None:
            matrix.require(authenticated_status(base, "/api/me", revoked_token) == 401,
                           "previously logged-out session became valid")
        gate.remote(server, base, session, "login", "delegate", "--password-stdin",
                    input="q08-delegate-disposable-password\n")
        me = gate.remote(server, base, session, "whoami")
        matrix.require(me["user"]["username"] == "delegate" and not me["user"]["admin"],
                       "ordinary credential did not identify the same non-admin user")
        token = json.loads(session.read_text())["token"]
        matrix.require(authenticated_status(base, "/api/audit?limit=1", token) == 200,
                       "active auditor grant stopped authorizing audit read")
        # Collection reads filter visible users; creation is the administration boundary.
        settings = tomllib.loads(config.read_text())
        rates = settings.get("rate_limits", {})
        limit = rates.get("general", 600) if type(rates) is dict else None
        matrix.require(settings.get("database_key_file") is None
                       and settings.get("trusted_proxies", []) == []
                       and re.fullmatch(r"http://127\.0\.0\.1:([0-9]+)", base) is not None
                       and settings.get("issuer") == base
                       and settings.get("listen") == base.removeprefix("http://")
                       and type(limit) is int and 1 <= limit <= 100_000,
                       "unsupported user-create admission fixture")
        connection = config.parent / settings["postgres"]["connection_file"]
        target = re.fullmatch(r"host=127\.0\.0\.1 port=([0-9]+) dbname=riauth_transition "
                              r"user=riauth_test sslmode=disable", connection.read_text().strip())
        matrix.require(target is not None and 0 < int(target[1]) <= 65535,
                       "unexpected user-create refusal snapshot target")

        def refusal_rows():
            result = subprocess.run([
                "psql", "-h", "127.0.0.1", "-p", target[1], "-U", "riauth_test", "-d", "riauth_transition",
                "-X", "--no-password", "-v", "ON_ERROR_STOP=1", "-At", "-F", "|", "-c",
                "SELECT encode(key,'hex'),encode(value,'hex') FROM riauth_store.records_v1 ORDER BY key",
            ], capture_output=True, timeout=5)
            matrix.require(result.returncode == 0 and result.stdout,
                           "full user-create refusal snapshot unavailable")
            return result.stdout

        revision = gate.remote(server, base, session, "revision")["revision"]
        before_refusal = refusal_rows()
        lower_ns, tick = time.time_ns(), time.monotonic_ns()
        denied = gate.remote(server, base, session, "--if-revision", revision,
                             "--idempotency-key", os.urandom(16).hex(),
                             "user", "create", "shared-refused-user", "--password-stdin",
                             input="q08-refused-disposable-password\n", expected=4)
        elapsed_ns, upper_ns = time.monotonic_ns() - tick, time.time_ns()
        matrix.require(denied["error"]["http_status"] == 403
                       and denied["error"]["code"] == "access_denied"
                       and denied["exit_code"] == 4,
                       "ordinary auditor gained user administration")
        after_refusal = refusal_rows()
        try:
            try:
                admitted = any(after_refusal == expected for expected in refusal_admission_snapshots(
                    before_refusal, lower_ns, upper_ns, elapsed_ns, limit))
            except (ValueError, OverflowError):
                admitted = False  # Unsupported admission evidence remains a full-snapshot failure.
            matrix.require(admitted,
                           "refused user creation changed durable records")
        except AssertionError as error:
            if type(error) is AssertionError:
                try:
                    error.store_snapshot_counts = refusal_snapshot_counts(before_refusal, after_refusal)
                except BaseException:
                    pass  # Diagnostics must not replace the original complete-snapshot failure.
            raise
        time.sleep(0.2)
        again = gate.remote(server, base, session, "whoami")
        matrix.require(again["expires_at"] == me["expires_at"], "session expiry was renewed")
        gate.remote(server, base, session, "logout")
        matrix.require(authenticated_status(base, "/api/me", token) == 401,
                       "logout failed to revoke the session")
        gate.remote(server, base, admin, "login", "admin", "--password-stdin",
                    input="q08-disposable-password\n")
        grants = gate.remote(server, base, admin, "grants", "get", "delegate")
        matrix.require(grants["grants"] == [{"role": "auditor", "scope": "audit/events",
                                            "target_id": "events"}], "auditor grant changed")
        group = gate.remote(server, base, admin, "get", "group", "shared-fixture")
        matrix.require(me["groups"], "ordinary membership missing")
        gate.remote(server, base, admin, "logout")
        return {"user": me["user"], "groups": me["groups"], "group": group,
                "grants": grants, "audit_status": 200, "users_status": 403}, token


def shared_config_refusals(programs, port, database, server, maintenance, config, base, scratch):
    original = config.read_text()
    parsed = tomllib.loads(original)
    baseline = rows(programs, port, database)
    metadata = transition_metadata(programs, port, database)
    candidates = {}
    authentication, count = re.subn(r"(?m)^(session_ttl\s*=\s*)\d+$",
                                   lambda m: m[1] + str(parsed["session_ttl"] + 1), original)
    matrix.require(count == 1, "session_ttl configuration field missing")
    candidates["authentication"] = (authentication, "Configured token lifetimes or password policy")
    value = metadata["meta/node_security"]["effective_rate_limits"]["general"] + 1
    if "general" in parsed.get("rate_limits", {}):
        rate, count = re.subn(r"(?m)^(general\s*=\s*)\d+$", lambda m: m[1] + str(value), original)
        matrix.require(count == 1, "general rate configuration field missing")
    elif re.search(r"(?m)^\[rate_limits\]\s*$", original):
        rate, count = re.subn(r"(?m)^\[rate_limits\]\s*$",
                             lambda m: m[0] + f"\ngeneral = {value}", original)
        matrix.require(count == 1, "duplicate rate_limits table")
    else:
        rate = original + f"\n[rate_limits]\ngeneral = {value}\n"
    matrix.require(tomllib.loads(rate)["rate_limits"]["general"] == value, "rate fixture invalid")
    candidates["rate"] = (rate, "Configured HTTP rate limit for general")
    for label, (text, message) in candidates.items():
        candidate = scratch / f"refuse-{label}.toml"
        candidate.write_text(text)
        blocked = gate.cli(maintenance, "--config", candidate,
                           "transition-plan", "--target", "platform", expected=5)
        matrix.require(any(item["resource"] == "meta/node_security"
                           for item in blocked["data"]["blockers"]), "shared configuration was ignored")
        refused = gate.cli(server, "--config", candidate, "serve", expected=None)
        matrix.require(message in refused["error"]["message"], "shared configuration startup refusal changed")
        matrix.require(rows(programs, port, database) == baseline and
                       transition_metadata(programs, port, database) == metadata,
                       "configuration refusal changed stored rows or security agreement")
        try:
            status = gate.get_status(base + "/readyz")
        except OSError:
            status = None
        matrix.require(status != 200, "refused process left a serving listener")


def main():
    os.umask(0o077)
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--artifacts", type=pathlib.Path, required=True)
    parser.add_argument("--evidence", type=pathlib.Path, required=True)
    parser.add_argument("--source-revision", required=True)
    args = parser.parse_args()
    matrix.require(re.fullmatch(r"[0-9a-f]{40}", args.source_revision) is not None,
                   "source revision must be a full Git commit")
    matrix.require(platform.system() == "Linux" and platform.machine() == "aarch64",
                   "native Linux aarch64 required")
    programs = {name: matrix.shutil.which(name) for name in ("initdb", "pg_ctl", "createdb", "psql")}
    matrix.require(all(programs.values()), f"PostgreSQL tools missing: {programs}")
    artifacts = args.artifacts.resolve()
    essentials = artifacts / "essentials"
    platform_bins = artifacts / "platform"
    binaries = {edition: {name: artifacts / edition / name
                          for name in ("riauth", "riauth-maintenance")}
                for edition in ("essentials", "platform")}
    for edition, pair in binaries.items():
        matrix.require(all(path.is_file() for path in pair.values()),
                       f"{edition} installed binary missing")
    args.evidence.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="local-a08-postgres-", dir=args.evidence.parent) as temporary:
        scratch = pathlib.Path(temporary)
        cluster, reason = matrix.postgres_cluster(scratch)
        matrix.require(cluster is not None, reason or "PostgreSQL cluster unavailable")
        cluster_programs, data_dir, port = cluster
        programs.update(cluster_programs)
        database = "riauth_transition"
        try:
            matrix.successful([programs["createdb"], "-h", "127.0.0.1", "-p", str(port),
                               "-U", "riauth_test", database])
            root = scratch / "instance"
            root.mkdir()
            connection = root / "connection"
            connection.write_text(f"host=127.0.0.1 port={port} dbname={database} user=riauth_test sslmode=disable\n")
            connection.chmod(0o600)
            pg_config = root / "postgres.json"
            pg_config.write_text(json.dumps({"connection_file": str(connection),
                                             "ca_file": None, "local_unencrypted": True}))
            config, base = matrix.init_instance(essentials / "riauth-maintenance",
                                                root / "config", pg_config)
            first = matrix.serve(essentials / "riauth", config, base)
            ordinary_fixture(essentials / "riauth", config, base, scratch)
            shared_before, revoked_token = shared_probe(essentials / "riauth", config, base, scratch)
            shared_config_refusals(programs, port, database, essentials / "riauth",
                                   platform_bins / "riauth-maintenance", config, base, scratch)
            before = rows(programs, port, database)
            before_metadata = transition_metadata(programs, port, database)
            matrix.require(any(key.startswith("users/") for key in before), "no identities stored")
            refused = gate.cli(platform_bins / "riauth", "--config", config, "serve", expected=None)
            matrix.require("active capabilities" in refused["error"]["message"],
                           "direct Platform open did not reject source agreement")
            plan = gate.cli(platform_bins / "riauth-maintenance", "--config", config,
                            "transition-plan", "--target", "platform")
            matrix.require(plan["ready"] and plan["transition_token"], "upgrade plan unavailable")
            connected = subprocess.Popen([
                programs["psql"], "-h", "127.0.0.1", "-p", str(port), "-U", "riauth_test",
                "-d", database, "-c", "SELECT pg_sleep(15)",
            ], env={**os.environ, "PGAPPNAME": "riauth"}, stdout=subprocess.DEVNULL,
               stderr=subprocess.DEVNULL)
            try:
                for _ in range(50):
                    count = subprocess.check_output([
                        programs["psql"], "-h", "127.0.0.1", "-p", str(port), "-U", "riauth_test",
                        "-d", database, "-At", "-c", "SELECT count(*) FROM pg_stat_activity WHERE application_name='riauth'",
                    ], text=True).strip()
                    if int(count) > 0:
                        break
                    time.sleep(0.1)
                else:
                    raise AssertionError("other riAuth PostgreSQL client did not connect")
                blocked = gate.cli(platform_bins / "riauth-maintenance", "--config", config,
                                   "transition-activate", "--target", "platform",
                                   "--token", plan["transition_token"], expected=5)
                matrix.require("Stop every riAuth process" in blocked["error"]["message"],
                               "connected client did not block activation")
            finally:
                connected.terminate()
                try:
                    connected.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    connected.kill()
                    connected.wait(timeout=5)
            for _ in range(200):
                remaining = subprocess.check_output([
                    programs["psql"], "-h", "127.0.0.1", "-p", str(port), "-U", "riauth_test",
                    "-d", database, "-At", "-c", "SELECT count(*) FROM pg_stat_activity WHERE application_name='riauth'",
                ], text=True).strip()
                if int(remaining) == 0:
                    break
                time.sleep(0.1)
            else:
                raise AssertionError("other riAuth PostgreSQL client remained connected")
            upgrade = gate.cli(platform_bins / "riauth-maintenance", "--config", config,
                               "transition-activate", "--target", "platform",
                               "--token", plan["transition_token"])
            matrix.require(upgrade["activated_edition"] == "platform", "upgrade failed")
            upgraded_rows = rows(programs, port, database)
            require_preserved(before, upgraded_rows, "upgrade")
            upgraded_metadata = transition_metadata(programs, port, database)
            require_target_metadata(before_metadata, upgraded_metadata, "platform")
            second = matrix.serve(platform_bins / "riauth", config, base)
            shared_platform, _ = shared_probe(platform_bins / "riauth", config, base, scratch, revoked_token)
            matrix.require(shared_platform == shared_before, "upgrade changed shared identity/authorization")
            refused = gate.cli(essentials / "riauth", "--config", config, "serve", expected=None)
            matrix.require("Platform" in refused["error"]["message"] or
                           "compiled capability" in refused["error"]["message"],
                           "direct Essentials downgrade did not reject Platform activation")
            before_down = rows(programs, port, database)
            plan = gate.cli(platform_bins / "riauth-maintenance", "--config", config,
                            "transition-plan", "--target", "essentials")
            matrix.require(plan["ready"] and plan["transition_token"], "downgrade plan unavailable")
            downgrade = gate.cli(platform_bins / "riauth-maintenance", "--config", config,
                                 "transition-activate", "--target", "essentials",
                                 "--token", plan["transition_token"])
            matrix.require(downgrade["activated_edition"] == "essentials", "downgrade failed")
            require_preserved(before_down, rows(programs, port, database), "downgrade")
            returned_metadata = transition_metadata(programs, port, database)
            require_target_metadata(upgraded_metadata, returned_metadata, "essentials")
            matrix.require(returned_metadata["meta/node_security"]["active_capabilities"] ==
                           before_metadata["meta/node_security"]["active_capabilities"],
                           "Essentials active capabilities were not restored")
            third = matrix.serve(essentials / "riauth", config, base)
            shared_return, _ = shared_probe(essentials / "riauth", config, base, scratch, revoked_token)
            matrix.require(shared_return == shared_before, "downgrade changed shared identity/authorization")
            report = {"schema": "riauth.local-native-postgres-transition/v1",
                      "release_gate_result": False, "architecture": "linux/aarch64",
                      "backend": "postgresql", "postgres_version": subprocess.check_output(
                          [programs["psql"], "--version"], text=True).strip(),
                      "binary_sha256": {edition: {name: gate.digest(path) for name, path in pair.items()}
                                        for edition, pair in binaries.items()},
                      "validator_sha256": gate.digest(pathlib.Path(__file__)),
                      "source_revision": args.source_revision,
                      "baseline_rows": len(before), "upgrade_preserved_rows": len(before),
                      "downgrade_preserved_rows": len(before_down),
                      "other_client_refused": True,
                      "issuer_and_authentication_preserved": True,
                      "agreement_format": 3, "all_effective_rates_preserved": True,
                      "shared_configuration_refusals": ["authentication", "general_rate"],
                      "shared_identity_authorization_sample": "passed",
                      "shared_sample_sha256": hashlib.sha256(json.dumps(
                          shared_before, sort_keys=True).encode()).hexdigest(),
                      "full_shared_gate": "not_certified",
                      "active_capabilities_switched": True,
                      "edition_and_version_metadata_coordinated": True,
                      "editions": [first["edition"], second["edition"], third["edition"]]}
            with args.evidence.open("x") as destination:
                json.dump(report, destination, indent=2, sort_keys=True)
                destination.write("\n")
            print(json.dumps({"result": "passed", "evidence": str(args.evidence),
                              "editions": report["editions"]}, sort_keys=True))
        finally:
            matrix.successful([programs["pg_ctl"], "-D", str(data_dir),
                               "stop", "-m", "immediate", "-w"])


if __name__ == "__main__":
    main()
