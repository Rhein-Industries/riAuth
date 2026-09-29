"""Fixture checks for the Q09 session-read measurement script."""

import importlib.util
import json
import pathlib
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
        self.assertLessEqual(budget["estimated_general_requests"], benchmark.BUDGET_MAX)
        self.assertLess(budget["estimated_general_requests"], budget["general_limit_per_minute"])
        with self.assertRaises(benchmark.SliceError):
            benchmark.validate_shape(400, 5, 30)

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
        with self.assertRaises(benchmark.SliceError):
            benchmark.main(["--self-check", "--out", "fixture.json"])
