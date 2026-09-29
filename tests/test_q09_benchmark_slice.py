"""Fixture checks for the Q09 session-read measurement script."""

import importlib.util
import json
import os
import pathlib
import shutil
import tempfile
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
