"""Fixture checks for the Q09 session-read measurement script."""

import base64
import importlib.util
import json
import os
import pathlib
import shutil
import socket
import ssl
import tempfile
import threading
import unittest


SCRIPT = pathlib.Path(__file__).resolve().parents[1] / "scripts/q09_benchmark_slice.py"
spec = importlib.util.spec_from_file_location("q09_benchmark_slice", SCRIPT)
benchmark = importlib.util.module_from_spec(spec)
spec.loader.exec_module(benchmark)


class Percentiles(unittest.TestCase):
    def test_nearest_rank_matches_ceil(self):
        samples = [1, 2, 3, 4, 100]
        self.assertEqual(benchmark.percentile_us(samples, 1, 2), 3)
        self.assertEqual(benchmark.percentile_us(samples, 95, 100), 100)
        self.assertEqual(benchmark.percentile_us(samples, 99, 100), 100)
        self.assertEqual(benchmark.percentile_us([7], 99, 100), 7)
        self.assertEqual(benchmark.percentile_us([10, 20], 1, 2), 10)
        spread = benchmark.distribution(samples)
        self.assertLessEqual(spread["p50_us"], spread["p95_us"])
        self.assertLessEqual(spread["p95_us"], spread["p99_us"])
        self.assertEqual(spread["max_us"], 100)
        self.assertEqual(benchmark.distribution([])["count"], 0)

    def test_cpu_time_parser(self):
        self.assertEqual(benchmark.parse_cputime("0:00.48"), 0.48)
        self.assertEqual(benchmark.parse_cputime("1:02:03"), 3723)
        self.assertEqual(benchmark.parse_cputime("2-01:02:03"), 2 * 86400 + 3723)

    def test_budget_stays_under_the_general_limit(self):
        budget = benchmark.validate_shape(80, 5, 30)
        self.assertEqual(budget["estimated_general_requests"], 236)
        self.assertEqual(budget["directory"]["memberships"], 0)
        self.assertLess(budget["estimated_general_requests"], budget["general_limit_per_minute"])
        wide = benchmark.validate_shape(80, 5, 30, 8, 8)
        self.assertEqual(wide["directory"]["memberships"], 72)
        self.assertLessEqual(wide["estimated_general_requests"], benchmark.BUDGET_MAX)
        with self.assertRaises(benchmark.SliceError):
            benchmark.validate_shape(400, 5, 30)
        with self.assertRaises(benchmark.SliceError):
            benchmark.validate_shape(80, 5, 30, 32, 32)
        with self.assertRaises(benchmark.SliceError):
            benchmark.directory_plan(-1, 1)
        with self.assertRaises(benchmark.SliceError):
            benchmark.directory_plan(benchmark.DIRECTORY_USERS_MAX + 1, 0)
        plan = benchmark.directory_plan(2, 2)
        self.assertEqual(plan["extra_usernames"], ["q09-user-0001", "q09-user-0002"])
        self.assertEqual(
            [item["username"] for item in plan["memberships"] if item["group"] == "q09-dir-0001"],
            ["admin", "q09-user-0001", "q09-user-0002"],
        )
        self.assertEqual(plan["writes"], 10)
        self.assertEqual(benchmark.dataset_name(plan), "q09-2u-2g")
        self.assertEqual(benchmark.dataset_name(benchmark.directory_plan(8, 8)), "q09-8u-8g")
        self.assertEqual(benchmark.dataset_name(benchmark.directory_plan(0, 0)), "fresh-init")
        self.assertEqual(
            benchmark.build_note("debug", "1.98.1", "essentials"),
            {"profile": "debug", "toolchain": "1.98.1", "features": "essentials"},
        )
        with self.assertRaises(benchmark.SliceError):
            benchmark.build_note("debug profile", "1.98.1", "essentials")

    def test_redaction_and_completion_codes(self):
        clean = {"passes": {"quiet": {"errors": 0, "error_statuses": {}},
                            "interference": {"errors": 0, "error_statuses": {}}},
                 "interference": {"overlap": True}}
        benchmark.ensure_redacted(clean, ["secret-value"])
        with self.assertRaises(benchmark.SliceError):
            benchmark.ensure_redacted({"leak": "secret-value"}, ["secret-value"])
        self.assertEqual(benchmark.completion_code(clean), 0)
        limited = json.loads(json.dumps(clean))
        limited["passes"]["quiet"]["errors"] = 1
        limited["passes"]["quiet"]["error_statuses"] = {"429": 1}
        self.assertEqual(benchmark.completion_code(limited), 3)
        failed = json.loads(json.dumps(clean))
        failed["passes"]["quiet"]["errors"] = 1
        self.assertEqual(benchmark.completion_code(failed), 4)
        quiet_only = json.loads(json.dumps(clean))
        quiet_only["interference"]["overlap"] = False
        self.assertEqual(benchmark.completion_code(quiet_only), 2)
        missed = json.loads(json.dumps(clean))
        missed["passes"]["quiet"]["elapsed_seconds"] = 70
        missed["passes"]["quiet"]["maintenance_cadence_overlap"] = False
        missed["passes"]["interference"]["elapsed_seconds"] = 1
        missed["passes"]["interference"]["maintenance_cadence_overlap"] = False
        self.assertEqual(benchmark.completion_code(missed), 5)
        self.assertFalse(benchmark.maintenance_cadence_overlap(0, 70))
        self.assertFalse(benchmark.maintenance_cadence_overlap(1, 60))
        self.assertTrue(benchmark.maintenance_cadence_overlap(1, 60.001))
        self.assertTrue(benchmark.maintenance_cadence_overlap(2, 75))
        self.assertEqual(benchmark.pace_seconds(0), 0)
        self.assertEqual(benchmark.pace_seconds(6500), 6.5)
        with self.assertRaises(benchmark.SliceError):
            benchmark.pace_seconds(-1)
        with self.assertRaises(benchmark.SliceError):
            benchmark.pace_seconds(benchmark.PACE_MS_MAX + 1)

    def test_session_read_classification(self):
        self.assertEqual(benchmark.classify_me(200, {"user": {"username": "admin"}}, False, "admin"), (True, None))
        self.assertEqual(
            benchmark.classify_me(
                200, {"user": {"username": "admin"}, "groups": ["q09-dir-0001"]}, False, "admin", ["q09-dir-0001"],
            ),
            (True, None),
        )
        self.assertEqual(
            benchmark.classify_me(
                200, {"user": {"username": "admin"}, "groups": []}, False, "admin", ["q09-dir-0001"],
            )[1],
            "unexpected_groups",
        )
        self.assertEqual(benchmark.classify_me(200, {"user": {"username": "other"}}, False, "admin")[1], "unexpected_body")
        self.assertEqual(benchmark.classify_me(429, {"error": "rate_limited"}, False, "admin")[1], "rate_limited")
        self.assertEqual(benchmark.classify_me(None, None, True, "admin")[1], "transport")


class FixtureRun(unittest.TestCase):
    def test_self_check_records_a_complete_fixture_observation(self):
        report = benchmark.run_slice(benchmark.sys.executable, "redb", 12, 1, 4, fixture=True)
        encoded = json.dumps(report)
        self.assertEqual(benchmark.completion_code(report), 0)
        self.assertFalse(report["product_run"])
        self.assertFalse(report["performance_claim"])
        self.assertTrue(report["observations_only"])
        self.assertEqual(report["schema"], "riauth.benchmark-slice/v1")
        self.assertEqual(report["artifact"]["kind"], "fixture")
        self.assertNotIn("q09-fixture-token", encoded)
        self.assertNotIn("password", encoded)
        quiet = report["passes"]["quiet"]
        interference = report["passes"]["interference"]
        self.assertEqual(quiet["success"], 12)
        self.assertEqual(quiet["errors"], 0)
        self.assertEqual(interference["success"], 12)
        self.assertEqual(quiet["server_counter_delta"]["write_wait_count"], 0)
        self.assertGreater(interference["server_counter_delta"]["write_wait_count"], 0)
        self.assertEqual(interference["server_counter_delta"]["write_wait_count"], report["interference"]["writer"]["success"])
        self.assertTrue(report["interference"]["overlap"])
        self.assertIn("maintenance", quiet["server_counter_delta"]["background_jobs"])
        self.assertIn("logout_ssf", quiet["server_counter_delta"]["background_jobs"])
        for passage in (quiet, interference):
            latency = passage["latency_us"]
            self.assertLessEqual(latency["p50_us"], latency["p95_us"])
            self.assertLessEqual(latency["p95_us"], latency["p99_us"])
            self.assertGreater(passage["throughput_success_per_second"], 0)
            self.assertTrue(passage["process"]["available"])
            self.assertGreater(passage["process"]["rss_kib_max"], 0)
            self.assertGreaterEqual(passage["process"]["cpu_seconds"], 0)
        self.assertTrue(report["limitations"])
        self.assertEqual(report["settings"]["database_encryption"], False)
        self.assertEqual(report["settings"]["tls"], False)
        self.assertEqual(report["dataset"]["kind"], "fresh-init")
        self.assertEqual(report["dataset"]["name"], "fresh-init")
        self.assertEqual(report["build"]["profile"], "unrecorded")
        self.assertEqual(report["dataset"]["extra_users"], 0)
        self.assertEqual(report["dataset"]["session_read_groups"], [])
        self.assertEqual(report["settings"]["directory_users"], 0)
        self.assertEqual(report["settings"]["directory_groups"], 0)
        self.assertEqual(report["settings"]["pace_ms"], 0)
        self.assertEqual(quiet["pace_ms"], 0)
        self.assertGreaterEqual(quiet["maintenance_finished_delta"], 1)
        self.assertFalse(quiet["maintenance_cadence_overlap"])
        self.assertEqual(
            quiet["maintenance_finished_delta"],
            quiet["server_counter_delta"]["background_jobs"]["maintenance"]["finished"],
        )
        with self.assertRaises(benchmark.SliceError):
            benchmark.main(["--self-check", "--out", "fixture.json"])
        with self.assertRaises(benchmark.SliceError):
            benchmark.main(["--self-check", "--directory-users", "2"])

    def test_directory_fixture_verifies_memberships_and_redacts_secrets(self):
        secrets = []
        real = benchmark.secrets.token_urlsafe

        def capture(nbytes=16):
            value = "q09-secret-" + real(6)
            secrets.append(value)
            return value

        benchmark.secrets.token_urlsafe = capture
        try:
            report = benchmark.run_slice(
                benchmark.sys.executable, "redb", 4, 0, 1,
                fixture=True, directory_users=2, directory_groups=2,
            )
        finally:
            benchmark.secrets.token_urlsafe = real
        encoded = json.dumps(report)
        self.assertGreaterEqual(len(secrets), 3)
        for secret in secrets:
            self.assertNotIn(secret, encoded)
        self.assertNotIn("password", encoded)
        data = report["dataset"]
        self.assertEqual(data["kind"], "session-read-directory")
        self.assertEqual(data["name"], "q09-2u-2g")
        self.assertEqual(report["build"]["profile"], "unrecorded")
        self.assertTrue(data["verified"])
        self.assertTrue(data["within_one_index_page"])
        self.assertEqual(data["extra_users"], 2)
        self.assertEqual(data["session_read_groups"], ["q09-dir-0001", "q09-dir-0002"])
        self.assertEqual(len(data["memberships"]), 6)
        self.assertEqual(
            data["verification"]["usernames"],
            ["admin", "q09-user-0001", "q09-user-0002"],
        )
        self.assertEqual(
            data["verification"]["member_counts"],
            {"q09-dir-0001": 3, "q09-dir-0002": 3},
        )
        self.assertEqual(report["passes"]["quiet"]["success"], 4)
        self.assertEqual(report["passes"]["quiet"]["errors"], 0)
        self.assertEqual(report["passes"]["interference"]["success"], 4)
        self.assertEqual(report["passes"]["quiet"]["server_counter_delta"]["write_wait_count"], 0)
        self.assertGreater(report["interference"]["writer"]["success"], 0)
        self.assertEqual(
            report["passes"]["interference"]["server_counter_delta"]["write_wait_count"],
            report["interference"]["writer"]["success"],
        )
        self.assertFalse(report["product_run"])
        self.assertFalse(report["performance_claim"])
        self.assertEqual(report["passes"]["quiet"]["pace_ms"], 0)
        self.assertFalse(report["passes"]["quiet"]["maintenance_cadence_overlap"])

    def test_paced_fixture_stretches_the_pass_without_claiming_cadence_overlap(self):
        report = benchmark.run_slice(
            benchmark.sys.executable, "redb", 4, 0, 1, fixture=True, pace_ms=40,
        )
        self.assertEqual(benchmark.completion_code(report), 0)
        self.assertEqual(report["settings"]["pace_ms"], 40)
        self.assertEqual(report["settings"]["maintenance_cadence_seconds"], 60)
        quiet = report["passes"]["quiet"]
        interference = report["passes"]["interference"]
        for passage in (quiet, interference):
            self.assertEqual(passage["pace_ms"], 40)
            self.assertEqual(passage["success"], 4)
            self.assertEqual(passage["errors"], 0)
            self.assertGreaterEqual(passage["elapsed_seconds"], 0.10)
            self.assertGreaterEqual(passage["maintenance_finished_delta"], 1)
            self.assertFalse(passage["maintenance_cadence_overlap"])
            self.assertEqual(
                passage["maintenance_finished_delta"],
                passage["server_counter_delta"]["background_jobs"]["maintenance"]["finished"],
            )
        self.assertTrue(report["interference"]["overlap"])
        self.assertFalse(report["product_run"])


PROBE_BINARY = """#!/usr/bin/env python3
import json, os, sys
record = {"argv0": sys.argv[0], "cwd": os.getcwd(), "args": sys.argv[1:]}
with open(__LOG_PATH__, "a", encoding="utf-8") as handle:
    handle.write(json.dumps(record) + "\\n")
    handle.flush()
if "capabilities" in sys.argv:
    data = {
        "schema_version": "riauth.capabilities/v2",
        "edition": "platform",
        "build_features": ["essentials", "platform"],
        "version": "relative-path-probe",
        "target": {"os": "probe", "arch": "probe"},
        "interface": "server",
    }
elif "init" in sys.argv:
    sys.stdin.read()
    data = {"initialized": True}
elif "serve" in sys.argv:
    raise SystemExit(0)
else:
    raise SystemExit(2)
json.dump({"schema_version": "riauth.cli/v1", "ok": True, "data": data}, sys.stdout)
sys.stdout.write("\\n")
"""


class RelativeBinary(unittest.TestCase):
    def test_missing_relative_binary_names_the_absolute_path(self):
        relative = "target/debug/riauth-q09-missing"
        self.assertFalse((pathlib.Path.cwd() / relative).exists())
        with self.assertRaises(benchmark.SliceError) as caught:
            benchmark.absolute_binary(relative)
        message = str(caught.exception)
        self.assertIn("binary is not a file:", message)
        recorded = pathlib.Path(message.split(": ", 1)[1])
        self.assertTrue(recorded.is_absolute())
        self.assertEqual(recorded, (pathlib.Path.cwd() / relative).resolve())

    def test_relative_binary_executes_from_the_instance_directory(self):
        checkout = pathlib.Path(tempfile.mkdtemp(prefix="q09-relative-binary-"))
        previous = os.getcwd()
        try:
            binary = checkout / "target" / "debug" / "riauth"
            binary.parent.mkdir(parents=True)
            log = checkout / "invocations.jsonl"
            binary.write_text(PROBE_BINARY.replace("__LOG_PATH__", json.dumps(str(log))))
            binary.chmod(0o755)
            os.chdir(checkout)
            resolved = benchmark.absolute_binary("target/debug/riauth")
            self.assertEqual(resolved, binary.resolve())
            self.assertTrue(resolved.is_absolute())
            with self.assertRaises(benchmark.SliceError) as caught:
                benchmark.run_slice("target/debug/riauth", "redb", 12, 1, 4)
            message = str(caught.exception)
            self.assertNotIn("No such file or directory", message)
            self.assertNotIn("target/debug/riauth", message)
            self.assertTrue(log.is_file(), message)
            lines = [json.loads(line) for line in log.read_text().splitlines() if line]
            phases = []
            for item in lines:
                phase = next((name for name in ("capabilities", "init", "serve") if name in item["args"]), "?")
                phases.append(phase)
                argv0 = pathlib.Path(item["argv0"])
                self.assertTrue(argv0.is_absolute(), item)
                self.assertEqual(argv0.resolve(), resolved)
            self.assertEqual(phases, ["capabilities", "init", "serve"], message)
            self.assertEqual(pathlib.Path(lines[0]["cwd"]).resolve(), benchmark.ROOT.resolve())
            instance = pathlib.Path(lines[1]["cwd"]).resolve()
            self.assertNotEqual(instance, checkout.resolve())
            self.assertFalse((instance / "target" / "debug" / "riauth").exists())
            self.assertEqual(pathlib.Path(lines[2]["cwd"]).resolve(), instance)
        finally:
            os.chdir(previous)
            shutil.rmtree(checkout)


class SecureMode(unittest.TestCase):
    def test_private_ca_rejects_unrelated_and_system_trust(self):
        hidden = []
        root = pathlib.Path(tempfile.mkdtemp(prefix="q09-ca-"))
        stop = threading.Event()
        started = []
        ready = threading.Event()

        def serve():
            context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
            context.load_cert_chain(
                certfile=str(material["server_cert"]), keyfile=str(material["server_key"]),
            )
            sock = socket.socket()
            sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            sock.bind(("127.0.0.1", 0))
            sock.listen(16)
            sock.settimeout(0.2)
            started.append(sock.getsockname()[1])
            ready.set()
            while not stop.is_set():
                try:
                    client, _addr = sock.accept()
                except socket.timeout:
                    continue
                try:
                    with context.wrap_socket(client, server_side=True) as tls:
                        tls.settimeout(2)
                        try:
                            tls.recv(4096)
                        except OSError:
                            pass
                        body = b'{"ok":true}'
                        tls.sendall(
                            b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: "
                            + str(len(body)).encode()
                            + b"\r\nConnection: close\r\n\r\n"
                            + body
                        )
                except ssl.SSLError:
                    pass
                finally:
                    try:
                        client.close()
                    except OSError:
                        pass
            sock.close()

        thread = None
        token = None
        try:
            material = benchmark.make_server_material(root, hidden)
            key_text = (root / "certs" / "database.key").read_text().strip()
            decoded = base64.urlsafe_b64decode(key_text + "=")
            self.assertEqual(len(decoded), 32)
            self.assertNotIn("=", key_text)
            self.assertEqual(material["database_key"].stat().st_mode & 0o777, 0o600)
            self.assertIn(key_text, hidden)
            thread = threading.Thread(target=serve, name="q09-tls-probe", daemon=True)
            thread.start()
            self.assertTrue(ready.wait(5))
            url = f"https://127.0.0.1:{started[0]}/readyz"
            matching = benchmark.private_ca_context(material["ca"])
            self.assertEqual(matching.verify_mode, ssl.CERT_REQUIRED)
            self.assertTrue(matching.check_hostname)
            self.assertEqual(benchmark.tls_probe(url, matching), 200)
            token = benchmark.TLS_CONTEXT.set(matching)
            _elapsed, status, parsed, _unparsed = benchmark.http_exchange("GET", url, timeout=5)
            self.assertEqual(status, 200)
            self.assertEqual(parsed, {"ok": True})
            self.assertIsNone(benchmark.tls_probe(url, benchmark.private_ca_context(material["unrelated_ca"])))
            self.assertIsNone(benchmark.tls_probe(url, ssl.create_default_context()))
        finally:
            if token is not None:
                benchmark.TLS_CONTEXT.reset(token)
            stop.set()
            if thread is not None:
                thread.join(timeout=2)
            shutil.rmtree(root, ignore_errors=True)

    def test_connection_file_hba_and_redaction(self):
        password = "ab" * 16
        text = (
            "host=localhost hostaddr=127.0.0.1 port=5432 dbname=riauth_q09 "
            f"user=riauth_test password={password} sslmode=require\n"
        )
        self.assertEqual(benchmark.require_connection_file(text, 5432), password)
        for rejected in ("sslmode=verify-full", "sslmode=disable", "sslmode=prefer"):
            with self.assertRaises(benchmark.SliceError):
                benchmark.require_connection_file(text.replace("sslmode=require", rejected), 5432)
        self.assertTrue(benchmark.pg_hba_is_hostssl_scram(benchmark.HOSTSSL_HBA))
        self.assertFalse(benchmark.pg_hba_is_hostssl_scram("local all all trust\n" + benchmark.HOSTSSL_HBA))
        self.assertFalse(benchmark.pg_hba_is_hostssl_scram("host all all 127.0.0.1/32 trust\n"))
        info = benchmark.libpq_conninfo(5432, "verify-full", "/tmp/ca.crt")
        self.assertIn("sslmode=verify-full", info)
        self.assertIn("host=localhost", info)
        self.assertIn("hostaddr=127.0.0.1", info)
        self.assertNotIn("password=", info)
        key = benchmark.database_key_text()
        banner = "-----BEGIN " + "PRIVATE KEY-----"
        hidden = [key, banner]
        with self.assertRaises(benchmark.SliceError):
            benchmark.ensure_redacted({"leak": key}, hidden)
        with self.assertRaises(benchmark.SliceError):
            benchmark.ensure_redacted({"leak": banner}, hidden)
        with self.assertRaises(benchmark.SliceError):
            benchmark.assert_report_text_clean('{"stored":"password=hidden"}')
        benchmark.ensure_redacted({"storage_format": "aes256gcm-v1"}, hidden)
        host = {
            "uname": {
                "system": "Linux",
                "node": "example",
                "release": "7.0.12-linuxkit",
                "version": "#1 SMP linuxkit",
            },
        }
        secure = "\n".join(benchmark.secure_limitations(host, "postgresql"))
        self.assertNotIn("Database encryption is off", secure)
        self.assertNotIn("trust auth", secure)
        self.assertIn("sslmode=verify-full", secure)
        self.assertIn("sslmode=require", secure)
        self.assertIn("aes256gcm-v1", secure)
        self.assertIn("external signer", secure)
        self.assertIn("Docker's Linux VM", secure)
        self.assertNotIn("password", secure.lower())
        redb = "\n".join(benchmark.secure_limitations({"uname": {}}, "redb"))
        self.assertIn("This run uses the redb backend.", redb)
        self.assertNotIn("Docker's Linux VM", redb)
        self.assertNotIn("Database encryption is off", redb)
        insecure = "\n".join(benchmark.LIMITATIONS)
        self.assertIn(benchmark.INSECURE_LISTENER, insecure)
        self.assertIn(benchmark.INSECURE_POSTGRES, insecure)
        with self.assertRaises(benchmark.SliceError):
            benchmark.main(["--self-check", "--secure"])
        with self.assertRaises(benchmark.SliceError):
            benchmark.run_slice(
                benchmark.sys.executable, "redb", 1, 0, 1, fixture=True, secure=True,
            )
