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


import ast
import runpy
from unittest import mock


class SourceContractData(unittest.TestCase):
    def texts(self):
        return {relative: (ROOT / relative).read_text() for relative in evidence.SOURCE_CONTRACTS}

    def changed_function(self, text, name, before, after):
        selected = [node for node in ast.parse(text).body
                    if isinstance(node, ast.FunctionDef) and node.name == name]
        self.assertEqual(len(selected), 1)
        body = ast.get_source_segment(text, selected[0])
        self.assertIn(before, body)
        return text.replace(body, body.replace(before, after, 1), 1)

    def test_canonical_contracts_allow_comments_and_whitespace(self):
        for relative, text in self.texts().items():
            with self.subTest(relative=relative):
                evidence.recognize_source_contract(text, relative)
                evidence.recognize_source_contract("\n# source formatting\n" + text + "\n# end\n", relative)

    def test_changed_graphs_and_referenced_constants_refuse(self):
        texts = self.texts()
        producer = "scripts/spdx_sbom.py"
        changed = self.changed_function(texts[producer], "linux_graphs",
                                        '"lock": "server"', '"lock": "client"')
        candidates = [
            (producer, changed),
            (producer, texts[producer].replace('"x86_64": "x86_64-unknown-linux-gnu"',
                                               '"x86_64": "other-target"', 1)),
            ("scripts/check-installed-release-gate.py",
             texts["scripts/check-installed-release-gate.py"].replace(
                 'EDITIONS = ("essentials", "platform")', 'EDITIONS = ("essentials",)', 1)),
            ("scripts/check-release-attestation.py",
             texts["scripts/check-release-attestation.py"].replace(
                 'ACTION_SHA = "1e69f48acb82d1966a394da916b4c1698aa569d6"',
                 'ACTION_SHA = "0000000000000000000000000000000000000000"', 1)),
        ]
        for relative, text in candidates:
            with self.subTest(relative=relative):
                self.assertNotEqual(text, texts[relative])
                with self.assertRaises(evidence.AuditError):
                    evidence.recognize_source_contract(text, relative)

    def test_selected_decorators_and_defaults_refuse(self):
        relative = "scripts/spdx_sbom.py"
        text = self.texts()[relative]
        for header in ("@staticmethod\ndef linux_graphs(arch):", "def linux_graphs(arch=None):"):
            with self.subTest(header=header):
                changed = text.replace("def linux_graphs(arch):", header, 1)
                with self.assertRaises(evidence.AuditError):
                    evidence.recognize_source_contract(changed, relative)

    def test_duplicate_alias_augmented_and_conditional_bindings_refuse(self):
        relative = "scripts/spdx_sbom.py"
        text = self.texts()[relative]
        declarations = (
            "linux_graphs = None\n",
            "LINUX_TARGETS += {}\n",
            "import math as linux_graphs\n",
            "from math import floor as linux_graphs\n",
            "def linux_graphs(arch):\n    pass\n",
            "if True:\n    def linux_graphs(arch):\n        pass\n",
        )
        for declaration in declarations:
            with self.subTest(declaration=declaration):
                with self.assertRaises(evidence.AuditError):
                    evidence.recognize_source_contract(text + "\n" + declaration, relative)

    def test_relevant_writes_in_other_bodies_refuse(self):
        relative = "scripts/spdx_sbom.py"
        text = self.texts()[relative]
        bodies = (
            "global linux_graphs\n    linux_graphs = None\n    return payload",
            "import math as linux_graphs\n    return payload",
            "if False:\n        def linux_graphs(arch):\n            pass\n    return payload",
            "LINUX_TARGETS['additional'] = 'other'\n    return payload",
        )
        for body in bodies:
            with self.subTest(body=body):
                changed = self.changed_function(text, "sha256_bytes",
                                                "return hashlib.sha256(payload).hexdigest()", body)
                with self.assertRaises(evidence.AuditError):
                    evidence.recognize_source_contract(changed, relative)

    def test_unrecognized_declarations_and_loader_ownership_refuse(self):
        texts = self.texts()
        relative = "scripts/check-installed-release-gate.py"
        candidates = (
            texts[relative] + "\nDECLARED_NOTE = object()\n",
            texts[relative] + "\nclass AdditionalDeclaration:\n    pass\n",
            texts[relative] + "\nif True:\n    pass\n",
            texts[relative].replace(' / "spdx_sbom.py"', ' / "other_document.py"', 1),
        )
        for text in candidates:
            with self.subTest(text_length=len(text)):
                self.assertNotEqual(text, texts[relative])
                with self.assertRaises(evidence.AuditError):
                    evidence.recognize_source_contract(text, relative)

    def test_owned_workflow_policy_keeps_permissions_actions_and_subjects(self):
        workflow = (ROOT / ".github/workflows/release.yml").read_text()
        result = evidence.require_release_workflow(workflow)
        self.assertFalse(result["signed_artifact"])
        self.assertEqual(result["action"], "actions/attest@1e69f48acb82d1966a394da916b4c1698aa569d6 # v4.2.2")
        candidates = (
            workflow.replace("id-token: write", "id-token: read", 1),
            workflow.replace("actions/attest@1e69f48acb82d1966a394da916b4c1698aa569d6", "actions/attest@other", 1),
            workflow.replace("all-subjects.txt", "different-subjects.txt", 1),
            workflow + "\npackages: write\n",
        )
        for text in candidates:
            with self.subTest(text_length=len(text)):
                self.assertNotEqual(text, workflow)
                with self.assertRaises(evidence.AuditError):
                    evidence.require_release_workflow(text)

    def test_source_read_and_parse_are_bounded(self):
        relative = "scripts/spdx_sbom.py"
        with self.assertRaises(evidence.AuditError):
            evidence.recognize_source_contract("x" * (evidence.MAX_SOURCE_BYTES + 1), relative)
        with self.assertRaises(evidence.AuditError):
            evidence.recognize_source_contract("def incomplete(", relative)
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            path = root / "source.py"
            path.write_bytes(b"x" * (evidence.MAX_SOURCE_BYTES + 1))
            with self.assertRaises(evidence.AuditError):
                evidence.read_text(root, path.name)
            path.unlink()
            path.mkdir()
            with self.assertRaises(evidence.AuditError):
                evidence.read_text(root, path.name)

    def test_source_ast_normalizes_only_empty_type_parameters(self):
        import copy

        absent = ast.parse(
            'def ordinary():\n    return "type_params=[]"\n'
            'async def asynchronous():\n    return "type_params=[]"\n'
            'class Declaration:\n    note = "type_params=[]"\n'
        )
        for node in absent.body:
            if "type_params" not in node._fields:
                node._fields = (*node._fields, "type_params")
            if hasattr(node, "type_params"):
                delattr(node, "type_params")
        empty = copy.deepcopy(absent)
        for node in empty.body:
            node.type_params = []
        before = ast.dump(empty, include_attributes=True)
        self.assertEqual(evidence.source_ast_dump(absent), evidence.source_ast_dump(empty))
        self.assertEqual(evidence.source_declarations_sha256(absent),
                         evidence.source_declarations_sha256(empty))
        self.assertEqual(ast.dump(empty, include_attributes=True), before)
        self.assertTrue(all(not hasattr(node, "type_params") for node in absent.body))
        self.assertTrue(all(node.type_params == [] for node in empty.body))
        for index in range(len(empty.body)):
            with self.subTest(node_type=type(empty.body[index]).__name__):
                nonempty = copy.deepcopy(empty)
                nonempty.body[index].type_params = [ast.Name(id="T", ctx=ast.Load())]
                self.assertNotEqual(evidence.source_ast_dump(nonempty),
                                    evidence.source_ast_dump(empty))
                self.assertNotEqual(evidence.source_declarations_sha256(nonempty),
                                    evidence.source_declarations_sha256(empty))
                self.assertEqual(len(nonempty.body[index].type_params), 1)
        literal = ast.Constant(value="type_params=[]")
        self.assertEqual(evidence.source_ast_dump(literal),
                         ast.dump(literal, include_attributes=False))
        self.assertEqual(literal.value, "type_params=[]")
        self.assertEqual([node.value for node in ast.walk(empty)
                          if isinstance(node, ast.Constant)], ["type_params=[]"] * 3)

    def test_inspected_contracts_never_use_execution_or_module_loading(self):
        texts = self.texts()
        refused = AssertionError("unexpected execution or module loading")
        with mock.patch("importlib.util.spec_from_file_location", side_effect=refused), \
                mock.patch("builtins.exec", side_effect=refused), \
                mock.patch("builtins.eval", side_effect=refused), \
                mock.patch("runpy.run_path", side_effect=refused):
            for relative, text in texts.items():
                evidence.recognize_source_contract(text, relative)
            evidence.require_package_contract(texts)
            evidence.require_release_workflow((ROOT / ".github/workflows/release.yml").read_text())


if __name__ == "__main__":
    unittest.main()
