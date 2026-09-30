"""Fail-closed checks for the release attestation pin.

These fixtures do not call gh and do not produce a bundle. A passing
verification still reports signed_artifact false.
"""

import base64
import hashlib
import importlib.util
import json
import pathlib
import subprocess
import sys
import tempfile
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/check-release-attestation.py"
spec = importlib.util.spec_from_file_location("release_attestation", SCRIPT)
attest = importlib.util.module_from_spec(spec)
spec.loader.exec_module(attest)

TAG = "v1.2.3"
SOURCE = "a" * 40
RUN_ID = "4242"
RUN_ATTEMPT = "1"
PREDICATE = attest.PROVENANCE_PREDICATE


def bundle_for(statement, **overrides):
    payload = base64.b64encode(json.dumps(statement).encode()).decode()
    bundle = {
        "mediaType": attest.BUNDLE_MEDIA_TYPE,
        "verificationMaterial": {"certificate": "present"},
        "dsseEnvelope": {
            "payload": payload,
            "signatures": [{"sig": "AAAA"}],
        },
    }
    bundle.update(overrides)
    return bundle


def verification(artifact_name, artifact_sha, predicate_type=PREDICATE, **overrides):
    certificate = attest.expected_certificate(TAG, SOURCE, RUN_ID, RUN_ATTEMPT)
    certificate.update(overrides.get("certificate", {}))
    if predicate_type == attest.PROVENANCE_PREDICATE:
        predicate = {
            "runDetails": {
                "metadata": {"invocationId": attest.invocation_id(RUN_ID, RUN_ATTEMPT)},
            },
        }
    else:
        predicate = {
            "spdxVersion": "SPDX-2.3",
            "packages": [{
                "name": artifact_name,
                "checksums": [{"algorithm": "SHA256", "checksumValue": artifact_sha}],
            }],
        }
    if isinstance(predicate, dict):
        predicate["decoy"] = "".join((
            attest.certificate_identity(TAG),
            attest.OIDC_ISSUER,
            SOURCE,
            f"refs/tags/{TAG}",
            f"https://github.com/{attest.REPOSITORY}",
            "github-hosted",
            attest.invocation_id(RUN_ID, RUN_ATTEMPT),
        ))
    predicate = overrides.get("predicate", predicate)
    statement = {
        "predicateType": overrides.get("predicate_type", predicate_type),
        "subject": overrides.get(
            "subject",
            [{"name": artifact_name, "digest": {"sha256": artifact_sha}}],
        ),
        "predicate": predicate,
    }
    return {
        "verificationResult": {
            "statement": statement,
            "signature": {"certificate": certificate},
            "verifiedTimestamps": overrides.get(
                "timestamps",
                [{"type": "rekor", "uri": "https://rekor.sigstore.dev"}],
            ),
        },
    }


class ReleaseWorkflowSource(unittest.TestCase):
    def workflow(self):
        return (ROOT / attest.WORKFLOW).read_text()

    def test_real_workflow_is_pinned_and_unsigned(self):
        report = attest.require_release_workflow(self.workflow())
        self.assertEqual(report["repository"], attest.REPOSITORY)
        self.assertEqual(report["workflow"], attest.WORKFLOW)
        self.assertEqual(report["action"], attest.ATTEST_USES)
        self.assertEqual(report["sigstore_instance"], "public-good")
        self.assertEqual(report["oidc_issuer"], attest.OIDC_ISSUER)
        self.assertFalse(report["signed_artifact"])
        self.assertNotIn("sigstore", self.workflow().lower())

    def test_static_cli_matches(self):
        completed = subprocess.run(
            [sys.executable, str(SCRIPT), "static", "--root", str(ROOT)],
            check=True,
            capture_output=True,
            text=True,
        )
        self.assertEqual(json.loads(completed.stdout), attest.require_release_workflow(self.workflow()))
        self.assertNotIn("signature verified", completed.stdout.lower())

    def test_producer_words_and_markers_fail(self):
        mutations = {
            "cosign": self.workflow() + "\ncosign sign artifact\n",
            "spdx word": self.workflow() + "\nspdx document\n",
            "sbom marker": self.workflow() + "\nextra.sbom.json\n",
        }
        for name, text in mutations.items():
            with self.subTest(name=name):
                with self.assertRaisesRegex(attest.AttestationError, "signing or SBOM producer"):
                    attest.require_release_workflow(text)

    def test_unexpected_action_pin_fails(self):
        text = self.workflow().replace(attest.ACTION_SHA, "b" * 40, 1)
        with self.assertRaisesRegex(attest.AttestationError, "four official attest actions"):
            attest.require_release_workflow(text)

    def test_registry_write_and_publish_oidc_fail(self):
        cases = {
            "packages": (self.workflow() + "\npackages: write\n", "package registry write"),
            "publish oidc": (
                self.workflow().replace(
                    "    permissions:\n      contents: write\n",
                    "    permissions:\n      contents: write\n      id-token: write\n",
                    1,
                ),
                "id-token write is not limited",
            ),
        }
        for name, (text, message) in cases.items():
            with self.subTest(name=name):
                with self.assertRaisesRegex(attest.AttestationError, message):
                    attest.require_release_workflow(text)

    def test_provenance_step_cannot_carry_an_inventory_predicate(self):
        essentials = "          sbom-path: target/dist/riauth-essentials-linux-${{ matrix.arch }}.spdx.json\n"
        anchor = "          subject-checksums: ${{ runner.temp }}/riauth-subjects/all-subjects.txt\n"
        text = self.workflow().replace(essentials, "", 1).replace(anchor, anchor + essentials, 1)
        with self.assertRaisesRegex(attest.AttestationError, "provenance step also carries an inventory predicate"):
            attest.require_release_workflow(text)


class BundleAndPolicy(unittest.TestCase):
    def materials(self, directory, payload=b"artifact-bytes", statement=None, bundle=None, root_text="trusted-root\n"):
        directory = pathlib.Path(directory)
        artifact = directory / "sample.bin"
        artifact.write_bytes(payload)
        digest = hashlib.sha256(payload).hexdigest()
        if statement is None:
            statement = {"subject": [{"name": artifact.name, "digest": {"sha256": digest}}]}
        if bundle is None:
            bundle = bundle_for(statement)
        bundle_path = directory / "bundle.json"
        bundle_path.write_text(json.dumps(bundle))
        trusted_root = directory / "trusted_root.jsonl"
        trusted_root.write_text(root_text)
        return artifact, bundle_path, trusted_root, digest

    def verify(self, directory, runner, predicate_type=PREDICATE, **kwargs):
        artifact, bundle_path, trusted_root, digest = self.materials(directory, **kwargs)
        return attest.verify_attestation(
            artifact, bundle_path, trusted_root, TAG, SOURCE, RUN_ID, RUN_ATTEMPT, predicate_type,
            runner=runner,
        ), digest

    def test_schema_and_missing_proof_fail_before_gh(self):
        with tempfile.TemporaryDirectory() as directory:
            artifact, bundle_path, trusted_root, digest = self.materials(directory)
            statement = {"subject": [{"name": "sample.bin", "digest": {"sha256": digest}}]}
            payload = bundle_for(statement)["dsseEnvelope"]["payload"]
            cases = {
                "media type": (bundle_for(statement, mediaType="application/json"), "bundle schema is invalid"),
                "no certificate": (
                    {
                        "mediaType": attest.BUNDLE_MEDIA_TYPE,
                        "verificationMaterial": {},
                        "dsseEnvelope": {"payload": payload, "signatures": [{"sig": "AAAA"}]},
                    },
                    "missing proof",
                ),
                "empty signature": (
                    {
                        "mediaType": attest.BUNDLE_MEDIA_TYPE,
                        "verificationMaterial": {"certificate": "present"},
                        "dsseEnvelope": {"payload": payload, "signatures": [{"sig": ""}]},
                    },
                    "missing proof",
                ),
            }
            calls = []

            def runner(command):
                calls.append(command)
                raise AssertionError("gh must not run")

            for name, (body, message) in cases.items():
                with self.subTest(name=name):
                    bundle_path.write_text(json.dumps(body))
                    with self.assertRaisesRegex(attest.AttestationError, message):
                        attest.verify_attestation(
                            artifact, bundle_path, trusted_root, TAG, SOURCE, RUN_ID, RUN_ATTEMPT,
                            PREDICATE, runner=runner,
                        )
            self.assertEqual(calls, [])

    def test_subject_substitution_fails_before_gh(self):
        with tempfile.TemporaryDirectory() as directory:
            artifact, bundle_path, trusted_root, digest = self.materials(directory)
            artifact.write_bytes(b"replaced-bytes")
            calls = []

            def runner(command):
                calls.append(command)
                raise AssertionError("gh must not run")

            with self.assertRaisesRegex(attest.AttestationError, "subject digest mismatch"):
                attest.verify_attestation(
                    artifact, bundle_path, trusted_root, TAG, SOURCE, RUN_ID, RUN_ATTEMPT, PREDICATE,
                    runner=runner,
                )
            self.assertEqual(calls, [])
            self.assertNotEqual(hashlib.sha256(artifact.read_bytes()).hexdigest(), digest)

    def test_wrong_certificate_identity_fails_after_gh(self):
        with tempfile.TemporaryDirectory() as directory:
            artifact, bundle_path, trusted_root, digest = self.materials(directory)

            def runner(_command):
                wrong = verification(artifact.name, digest)
                wrong["verificationResult"]["signature"]["certificate"]["subjectAlternativeName"] = (
                    "https://github.com/other/repo/.github/workflows/release.yml@refs/tags/v1.2.3"
                )
                return [wrong]

            with self.assertRaisesRegex(attest.AttestationError, "certificate identity mismatch"):
                attest.verify_attestation(
                    artifact, bundle_path, trusted_root, TAG, SOURCE, RUN_ID, RUN_ATTEMPT, PREDICATE,
                    runner=runner,
                )

    def test_predicate_decoy_does_not_supply_identity(self):
        with tempfile.TemporaryDirectory() as directory:
            artifact, _bundle_path, _trusted_root, digest = self.materials(directory)
            mutations = (
                ("runnerEnvironment", "self-hosted"),
                ("runInvocationURI", attest.invocation_id("99", RUN_ATTEMPT)),
                ("runInvocationURI", attest.invocation_id(RUN_ID, "99")),
                ("runInvocationURI", None),
            )
            for predicate_type in attest.PREDICATES:
                for key, value in mutations:
                    with self.subTest(predicate_type=predicate_type, field=key, value=value):
                        result = verification(artifact.name, digest, predicate_type=predicate_type)
                        certificate = result["verificationResult"]["signature"]["certificate"]
                        if value is None:
                            certificate.pop(key)
                        else:
                            certificate[key] = value
                        with self.assertRaisesRegex(attest.AttestationError, "certificate identity mismatch"):
                            attest.require_verification_result(
                                [result], artifact.name, digest, TAG, SOURCE, RUN_ID, RUN_ATTEMPT,
                                predicate_type,
                            )

    def test_missing_timestamp_is_missing_proof(self):
        with tempfile.TemporaryDirectory() as directory:
            artifact, _bundle_path, _trusted_root, digest = self.materials(directory)
            result = verification(artifact.name, digest, timestamps=[])
            with self.assertRaisesRegex(attest.AttestationError, "missing proof"):
                attest.require_verification_result(
                    [result], artifact.name, digest, TAG, SOURCE, RUN_ID, RUN_ATTEMPT, PREDICATE,
                )

    def test_provenance_run_pin_and_spdx_checksum(self):
        with tempfile.TemporaryDirectory() as directory:
            artifact, bundle_path, trusted_root, digest = self.materials(directory)

            def wrong_run(_command):
                return [verification(
                    artifact.name,
                    digest,
                    predicate={
                        "runDetails": {"metadata": {"invocationId": attest.invocation_id("99", "1")}},
                        "decoy": attest.invocation_id(RUN_ID, RUN_ATTEMPT),
                    },
                )]

            with self.assertRaisesRegex(attest.AttestationError, "workflow run mismatch"):
                attest.verify_attestation(
                    artifact, bundle_path, trusted_root, TAG, SOURCE, RUN_ID, RUN_ATTEMPT, PREDICATE,
                    runner=wrong_run,
                )

            def wrong_spdx(_command):
                return [verification(
                    artifact.name,
                    digest,
                    predicate_type=attest.SPDX_PREDICATE,
                    predicate={
                        "spdxVersion": "SPDX-2.3",
                        "packages": [{
                            "name": artifact.name,
                            "checksums": [{"algorithm": "SHA256", "checksumValue": "c" * 64}],
                        }],
                    },
                )]

            with self.assertRaisesRegex(attest.AttestationError, "subject digest mismatch"):
                attest.verify_attestation(
                    artifact, bundle_path, trusted_root, TAG, SOURCE, RUN_ID, RUN_ATTEMPT,
                    attest.SPDX_PREDICATE, runner=wrong_spdx,
                )

    def test_success_keeps_signed_artifact_false(self):
        with tempfile.TemporaryDirectory() as directory:
            artifact, bundle_path, trusted_root, digest = self.materials(directory)
            calls = []

            def runner(command):
                calls.append(command)
                return [verification(artifact.name, digest)]

            report = attest.verify_attestation(
                artifact, bundle_path, trusted_root, TAG, SOURCE, RUN_ID, RUN_ATTEMPT, PREDICATE,
                runner=runner,
            )
            self.assertFalse(report["signed_artifact"])
            self.assertEqual(report["subject_sha256"], digest)
            self.assertEqual(report["source"], SOURCE)
            command = calls[0]
            self.assertEqual(command[0:3], ["gh", "attestation", "verify"])
            self.assertNotIn("--no-public-good", command)
            self.assertNotIn("--signer-workflow", command)
            self.assertIn("--deny-self-hosted-runners", command)
            self.assertEqual(command[command.index("--signer-digest") + 1], SOURCE)
            self.assertEqual(command[command.index("--source-digest") + 1], SOURCE)
            self.assertEqual(command[command.index("--source-ref") + 1], f"refs/tags/{TAG}")
            self.assertEqual(command[command.index("--cert-oidc-issuer") + 1], attest.OIDC_ISSUER)
            self.assertEqual(
                command[command.index("--cert-identity") + 1],
                attest.certificate_identity(TAG),
            )
            self.assertNotIn(attest.ACTION_SHA, command)
            self.assertIn(str(bundle_path), command)
            self.assertIn(str(trusted_root), command)

            def spdx_runner(command):
                calls.append(command)
                return [verification(artifact.name, digest, predicate_type=attest.SPDX_PREDICATE)]

            spdx_report = attest.verify_attestation(
                artifact, bundle_path, trusted_root, TAG, SOURCE, RUN_ID, RUN_ATTEMPT,
                attest.SPDX_PREDICATE, runner=spdx_runner,
            )
            self.assertFalse(spdx_report["signed_artifact"])
            self.assertEqual(spdx_report["predicate_type"], attest.SPDX_PREDICATE)

            def wrong_predicate(_command):
                return [verification(artifact.name, digest, predicate_type="https://example.invalid/v1")]

            with self.assertRaisesRegex(attest.AttestationError, "predicate type mismatch"):
                attest.verify_attestation(
                    artifact, bundle_path, trusted_root, TAG, SOURCE, RUN_ID, RUN_ATTEMPT, PREDICATE,
                    runner=wrong_predicate,
                )

    def test_empty_trust_root_and_symlink_fail(self):
        with tempfile.TemporaryDirectory() as directory:
            directory = pathlib.Path(directory)
            artifact, bundle_path, trusted_root, _digest = self.materials(directory)
            trusted_root.write_text("")
            with self.assertRaisesRegex(attest.AttestationError, "trusted root is missing"):
                attest.verify_attestation(
                    artifact, bundle_path, trusted_root, TAG, SOURCE, RUN_ID, RUN_ATTEMPT, PREDICATE,
                    runner=lambda _command: [],
                )
            trusted_root.write_text("trusted-root\n")
            linked = directory / "linked.bin"
            linked.symlink_to(artifact)
            with self.assertRaisesRegex(attest.AttestationError, "artifact is not a regular file"):
                attest.verify_attestation(
                    linked, bundle_path, trusted_root, TAG, SOURCE, RUN_ID, RUN_ATTEMPT, PREDICATE,
                    runner=lambda _command: [],
                )

    def test_cli_rejects_a_bad_bundle_without_claiming_success(self):
        with tempfile.TemporaryDirectory() as directory:
            directory = pathlib.Path(directory)
            artifact = directory / "sample.bin"
            artifact.write_bytes(b"artifact-bytes")
            bundle = directory / "bundle.json"
            bundle.write_text("{}\n")
            trusted_root = directory / "trusted_root.jsonl"
            trusted_root.write_text("trusted-root\n")
            completed = subprocess.run(
                [
                    sys.executable, str(SCRIPT), "verify",
                    "--artifact", str(artifact),
                    "--bundle", str(bundle),
                    "--trusted-root", str(trusted_root),
                    "--tag", TAG,
                    "--source", SOURCE,
                    "--run-id", RUN_ID,
                    "--run-attempt", RUN_ATTEMPT,
                    "--predicate", PREDICATE,
                ],
                capture_output=True,
                text=True,
            )
        self.assertEqual(completed.returncode, 1)
        self.assertIn("bundle schema is invalid", completed.stderr)
        self.assertEqual(completed.stdout, "")
        self.assertNotIn("signature verified", completed.stderr.lower())


class PublicObservation(unittest.TestCase):
    def observed(self, **changes):
        repository = {
            "full_name": attest.REPOSITORY,
            "private": False,
            "archived": False,
            "visibility": "public",
            "default_branch": "main",
        }
        release = {
            "tagName": "v0.1.1",
            "isDraft": True,
            "assets": [{"name": name} for name in attest.OBSERVED_DRAFT_ASSETS],
        }
        run = {
            "databaseId": attest.OBSERVED_RUN_ID,
            "headSha": attest.OBSERVED_RUN_SHA,
            "conclusion": "success",
            "workflowName": "Public CI",
        }
        artifact_count = 0
        repository.update(changes.get("repository", {}))
        release.update(changes.get("release", {}))
        run.update(changes.get("run", {}))
        if "artifact_count" in changes:
            artifact_count = changes["artifact_count"]
        return repository, release, run, artifact_count

    def test_recorded_observation_is_not_a_release(self):
        report = attest.classify_public_observation(*self.observed())
        self.assertTrue(report["historical_draft_incomplete"])
        self.assertFalse(report["current_accepted_release"])
        self.assertFalse(report["signed_artifact"])
        self.assertFalse(report["release_executed"])
        self.assertEqual(report["public_ci_run"], 36342719277)
        self.assertEqual(report["public_ci_artifact_count"], 0)
        self.assertEqual(report["draft_assets"], list(attest.OBSERVED_DRAFT_ASSETS))

    def test_changed_observation_fails_closed(self):
        cases = {
            "private": {"repository": {"private": True}},
            "archived": {"repository": {"archived": True}},
            "branch": {"repository": {"default_branch": "develop"}},
            "published": {"release": {"isDraft": False}},
            "extra asset": {"release": {"assets": [{"name": name} for name in attest.OBSERVED_DRAFT_ASSETS] + [{"name": "extra.tar.gz"}]}},
            "other sha": {"run": {"headSha": "b" * 40}},
            "artifacts": {"artifact_count": 1},
        }
        for name, changes in cases.items():
            with self.subTest(name=name):
                with self.assertRaises(attest.AttestationError):
                    attest.classify_public_observation(*self.observed(**changes))


if __name__ == "__main__":
    unittest.main()
