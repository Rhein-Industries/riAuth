"""Preflight and redacted-evidence boundaries for the pinned OIDF runner."""

import importlib.util
import json
import os
import pathlib
import subprocess
import sys
import tempfile
import unittest
from unittest import mock


SCRIPT = pathlib.Path(__file__).resolve().parent.parent / "scripts/run-conformance.py"
spec = importlib.util.spec_from_file_location("run_conformance", SCRIPT)
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)


class ConformancePreflight(unittest.TestCase):
    def test_server_requires_safe_explicit_url(self):
        self.assertEqual(runner.validate_server("https://pilot.example.test")["scheme"], "https")
        self.assertEqual(runner.validate_server("http://127.0.0.1:8443")["scheme"], "http")
        for url in ("http://pilot.example.test", "https://user:secret@pilot.example.test",
                    "https://pilot.example.test/?token=secret", "https://pilot.example.test/#fragment"):
            with self.subTest(url=url), self.assertRaises(ValueError) as caught:
                runner.validate_server(url)
            self.assertNotIn("secret", str(caught.exception))

    def test_private_config_requires_owner_only_regular_file(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            config = root / "pilot.json"
            config.write_text("{}")
            config.chmod(0o600)
            self.assertEqual(runner.validate_config(config), config.resolve())
            config.chmod(0o644)
            with self.assertRaisesRegex(ValueError, "owner-only"):
                runner.validate_config(config)
            config.chmod(0o600)
            link = root / "link.json"
            link.symlink_to(config)
            with self.assertRaisesRegex(ValueError, "symlink"):
                runner.validate_config(link)

    def test_suite_pin_rejects_other_revision(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            (root / "scripts").mkdir()
            (root / "scripts" / "run-test-plan.py").write_text("pass\n")
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            subprocess.run(["git", "add", "scripts/run-test-plan.py"], cwd=root, check=True)
            subprocess.run(["git", "-c", "user.name=Fixture", "-c", "user.email=fixture@example.test",
                            "commit", "-qm", "fixture"], cwd=root, check=True)
            with self.assertRaisesRegex(ValueError, "Expected independent suite commit"):
                runner.validate_suite(root)
            revision = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip()
            with mock.patch.object(runner, "PIN", revision):
                self.assertEqual(runner.validate_suite(root)[1], revision)
                (root / "scripts" / "run-test-plan.py").write_text("# dirty\n")
                with self.assertRaisesRegex(ValueError, "local changes"):
                    runner.validate_suite(root)

    def test_report_is_private_and_cannot_be_replaced(self):
        with tempfile.TemporaryDirectory() as directory:
            output = pathlib.Path(directory) / "evidence"
            runner.write_private_report(output, {"scope": "fixture"})
            self.assertEqual(output.stat().st_mode & 0o777, 0o700)
            report = output / "run.json"
            self.assertEqual(report.stat().st_mode & 0o777, 0o600)
            self.assertEqual(json.loads(report.read_text()), {"scope": "fixture"})
            with self.assertRaisesRegex(ValueError, "new directory"):
                runner.write_private_report(output, {})

    def test_export_symlink_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            (root / "output").write_text("fixture")
            (root / "link").symlink_to(root / "output")
            with self.assertRaisesRegex(ValueError, "symlink"):
                runner.export_digest(root)

    def test_missing_private_inputs_prevent_execution(self):
        with tempfile.TemporaryDirectory() as directory:
            output = pathlib.Path(directory) / "evidence"
            environment = dict(os.environ)
            environment.pop("CONFORMANCE_SERVER", None)
            environment.pop("CONFORMANCE_TOKEN", None)
            result = subprocess.run(
                [sys.executable, str(SCRIPT), "--suite", str(pathlib.Path(directory) / "suite"),
                 "--config", str(pathlib.Path(directory) / "config"), "--plan", "fixture",
                 "--output", str(output)],
                env=environment, text=True, capture_output=True, check=False)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("CONFORMANCE_SERVER is missing", result.stderr)
            self.assertFalse(output.exists())


if __name__ == "__main__":
    unittest.main()
