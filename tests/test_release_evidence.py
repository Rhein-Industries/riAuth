"""Source-contract fixtures for the Q11 release-evidence audit."""

import contextlib
import importlib.util
import io
import json
import pathlib
import shutil
import subprocess
import sys
import tempfile
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/check-release-evidence.py"
spec = importlib.util.spec_from_file_location("release_evidence", SCRIPT)
evidence = importlib.util.module_from_spec(spec)
spec.loader.exec_module(evidence)


class ReleaseEvidenceAudit(unittest.TestCase):
    def copy_checkout(self, directory):
        dest = pathlib.Path(directory)
        for relative in evidence.AUDITED:
            target = dest / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(ROOT / relative, target)
        return dest

    def test_checkout_contract_stays_negative(self):
        report = evidence.audit(ROOT)
        self.assertEqual(report["schema"], "riauth.release-evidence/v1")
        self.assertEqual(report["vulnerability_intake"]["security_emails_in_policy"], [])
        self.assertEqual(
            report["vulnerability_intake"]["response_deadline"],
            "none promised for this initial release",
        )
        self.assertFalse(report["vulnerability_intake"]["reports_received_by_this_check"])
        self.assertEqual(report["signing"]["mechanism"], "github-artifact-attestations")
        self.assertEqual(report["signing"]["sigstore_instance"], "public-good")
        self.assertEqual(
            report["signing"]["action"],
            "actions/attest@1e69f48acb82d1966a394da916b4c1698aa569d6",
        )
        self.assertTrue(report["signing"]["present_in_release_path"])
        self.assertFalse(report["signing"]["signed_artifact"])
        self.assertFalse(report["signing"]["bundle_produced"])
        self.assertFalse(report["sbom"]["spdx_or_cyclonedx_document"])
        self.assertTrue(report["sbom"]["notices_include_spdx_license_expressions"])
        self.assertFalse(report["sbom"]["notices_are_an_sbom"])
        self.assertEqual(report["sbom"]["source_producer"], "scripts/spdx_sbom.py")
        self.assertFalse(report["sbom"]["source_producer_in_release_workflow"])
        self.assertTrue(report["sbom"]["packager_source_calls_producer"])
        self.assertTrue(report["sbom"]["asset_checks_require_package_spdx"])
        self.assertFalse(report["sbom"]["release_sbom_produced"])
        self.assertIn("scripts/spdx_sbom.py", report["files"])
        self.assertIn("scripts/check-installed-release-gate.py", report["files"])
        self.assertFalse(report["independent_review_record"])
        self.assertFalse(report["provenance"]["cryptographic_attestation"])
        self.assertEqual(report["provenance"]["schema"], "riauth.build/v4")
        self.assertFalse(report["publication"]["published_by_this_check"])
        self.assertEqual(report["linux_arm64_execution"], "not run")
        self.assertFalse(report["release_executed"])
        self.assertFalse(report["bundle_checker"]["ran_against_assets"])
        self.assertFalse(report["checker_host"]["runs_release_packager"])
        self.assertEqual(report["release_assets"]["status"], "not_requested")
        self.assertEqual(report["release_assets"]["verification"], "not performed")
        for relative in evidence.AUDITED:
            self.assertEqual(report["files"][relative], evidence.digest(ROOT / relative))

    def test_cli_matches_audit(self):
        completed = subprocess.run(
            [sys.executable, str(SCRIPT)],
            cwd=ROOT,
            check=True,
            capture_output=True,
            text=True,
        )
        report = evidence.audit(ROOT)
        self.assertEqual(json.loads(completed.stdout), report)
        with contextlib.redirect_stdout(io.StringIO()) as captured:
            self.assertEqual(evidence.main(["--root", str(ROOT)]), 0)
        self.assertEqual(json.loads(captured.getvalue()), report)

    def test_missing_deadline_fails(self):
        with tempfile.TemporaryDirectory() as directory:
            root = self.copy_checkout(directory)
            policy = root / "SECURITY.md"
            policy.write_text(policy.read_text().replace(
                "No response or remediation deadline is promised for this initial release.",
                "Reports are welcome.",
            ))
            with self.assertRaisesRegex(evidence.AuditError, "missing intake text"):
                evidence.audit(root)

    def test_signing_producer_fails_closed(self):
        with tempfile.TemporaryDirectory() as directory:
            root = self.copy_checkout(directory)
            workflow = root / ".github/workflows/release.yml"
            workflow.write_text(workflow.read_text() + "\ncosign sign artifact\n")
            with self.assertRaisesRegex(evidence.AuditError, "signing or SBOM producer"):
                evidence.audit(root)

    def test_procedure_cannot_claim_a_signed_artifact(self):
        with tempfile.TemporaryDirectory() as directory:
            root = self.copy_checkout(directory)
            procedure = root / "docs/roadmap/q11-release-evidence.md"
            procedure.write_text(procedure.read_text() + "\nA signed artifact was produced.\n")
            with self.assertRaisesRegex(evidence.AuditError, "signed artifact"):
                evidence.audit(root)

    def test_procedure_cannot_invent_a_contact(self):
        with tempfile.TemporaryDirectory() as directory:
            root = self.copy_checkout(directory)
            procedure = root / "docs/roadmap/q11-release-evidence.md"
            procedure.write_text(procedure.read_text() + "\nContact security@example.com.\n")
            with self.assertRaisesRegex(evidence.AuditError, "contact that is not in SECURITY.md"):
                evidence.audit(root)

    def test_dist_names_are_not_verification(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            present = root / "present"
            present.mkdir()
            (present / "build-provenance-linux-aarch64.json").write_text("{}\n")
            report = evidence.audit(ROOT, present)
            self.assertEqual(report["release_assets"]["status"], "names_only")
            self.assertEqual(
                report["release_assets"]["present"],
                ["build-provenance-linux-aarch64.json"],
            )
            self.assertEqual(report["release_assets"]["verification"], "not performed")
            self.assertEqual(report["linux_arm64_execution"], "not run")
            self.assertFalse(report["signing"]["signed_artifact"])
            self.assertFalse(report["release_executed"])
            self.assertFalse(report["sbom"]["spdx_or_cyclonedx_document"])
            self.assertFalse(report["sbom"]["release_sbom_produced"])

        missing = ROOT / "target" / "dist-not-created"
        report = evidence.audit(ROOT, missing)
        self.assertEqual(report["release_assets"]["status"], "unavailable")
        self.assertEqual(report["release_assets"]["verification"], "not performed")
        self.assertFalse(report["release_executed"])

    def test_dist_spdx_name_is_not_a_release_sbom(self):
        with tempfile.TemporaryDirectory() as directory:
            present = pathlib.Path(directory)
            (present / "riauth.spdx.json").write_text("{}\n")
            report = evidence.audit(ROOT, present)
        self.assertEqual(report["release_assets"]["present"], ["riauth.spdx.json"])
        self.assertEqual(report["release_assets"]["verification"], "not performed")
        self.assertFalse(report["sbom"]["spdx_or_cyclonedx_document"])
        self.assertFalse(report["sbom"]["notices_are_an_sbom"])
        self.assertFalse(report["sbom"]["source_producer_in_release_workflow"])
        self.assertTrue(report["sbom"]["packager_source_calls_producer"])
        self.assertTrue(report["sbom"]["asset_checks_require_package_spdx"])
        self.assertFalse(report["sbom"]["release_sbom_produced"])
        self.assertFalse(report["release_executed"])

    def test_release_path_cannot_name_the_source_producer(self):
        for relative in evidence.RELEASE_PATHS:
            with self.subTest(relative=relative):
                with tempfile.TemporaryDirectory() as directory:
                    root = self.copy_checkout(directory)
                    path = root / relative
                    path.write_text(path.read_text() + "\npython3 scripts/spdx_sbom.py\n")
                    with self.assertRaisesRegex(evidence.AuditError, "source producer"):
                        evidence.audit(root)

    def test_packager_must_keep_the_exact_package_call(self):
        with tempfile.TemporaryDirectory() as directory:
            root = self.copy_checkout(directory)
            path = root / "scripts/package-release.sh"
            path.write_text(path.read_text().replace(
                evidence.PACKAGE_CALL,
                "python3 scripts/spdx_sbom.py\n",
            ))
            with self.assertRaisesRegex(evidence.AuditError, "packager must call the source producer once"):
                evidence.audit(root)

    def test_procedure_must_keep_the_source_producer_boundary(self):
        with tempfile.TemporaryDirectory() as directory:
            root = self.copy_checkout(directory)
            procedure = root / "docs/roadmap/q11-release-evidence.md"
            procedure.write_text(procedure.read_text().replace(
                "No release SBOM was produced in this slice.",
                "The producer is documented in this file.",
            ))
            with self.assertRaisesRegex(evidence.AuditError, "no release SBOM was produced"):
                evidence.audit(root)

    def test_procedure_cannot_call_the_output_a_release_sbom(self):
        with tempfile.TemporaryDirectory() as directory:
            root = self.copy_checkout(directory)
            procedure = root / "docs/roadmap/q11-release-evidence.md"
            procedure.write_text(procedure.read_text() + "\nThis output is a release SBOM.\n")
            with self.assertRaisesRegex(evidence.AuditError, "procedure claims a release SBOM"):
                evidence.audit(root)

    def test_producer_script_cannot_claim_a_release_sbom(self):
        with tempfile.TemporaryDirectory() as directory:
            root = self.copy_checkout(directory)
            script = root / "scripts/spdx_sbom.py"
            script.write_text(script.read_text() + "\nThis output is a release SBOM.\n")
            with self.assertRaisesRegex(evidence.AuditError, "source producer claims a release SBOM"):
                evidence.audit(root)

    def test_producer_script_must_stay_locked_and_offline(self):
        with tempfile.TemporaryDirectory() as directory:
            root = self.copy_checkout(directory)
            script = root / "scripts/spdx_sbom.py"
            script.write_text(script.read_text().replace("--locked", "--frozen"))
            with self.assertRaisesRegex(evidence.AuditError, "does not pass --locked"):
                evidence.audit(root)


if __name__ == "__main__":
    unittest.main()
