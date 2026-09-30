#!/usr/bin/env python3
"""Check release attestation source and verify one offline bundle.

The release workflow requests GitHub artifact attestations for the public
repository Rhein-Industries/riAuth. Public repositories use the Sigstore
Public Good instance. This script does not sign, publish, fetch a trust
root, or decide that a release ran. A passing verification still belongs to
the bundle the caller supplied.
"""

import argparse
import base64
import binascii
import hashlib
import json
import pathlib
import re
import subprocess
import sys


REPOSITORY = "Rhein-Industries/riAuth"
WORKFLOW = ".github/workflows/release.yml"
WORKFLOW_FILE = "release.yml"
ACTION_SHA = "1e69f48acb82d1966a394da916b4c1698aa569d6"
ATTEST_USES = f"actions/attest@{ACTION_SHA} # v4.2.2"
OIDC_ISSUER = "https://token.actions.githubusercontent.com"
PROVENANCE_PREDICATE = "https://slsa.dev/provenance/v1"
SPDX_PREDICATE = "https://spdx.dev/Document/v2.3"
PREDICATES = (PROVENANCE_PREDICATE, SPDX_PREDICATE)
BUNDLE_MEDIA_TYPE = "application/vnd.dev.sigstore.bundle.v0.3+json"
TAG_RE = re.compile(r"v[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?")
SOURCE_RE = re.compile(r"[0-9a-f]{40}")
RUN_RE = re.compile(r"[1-9][0-9]*")
PACKAGE_DOCUMENTS = (
    "riauth-essentials-linux-${{ matrix.arch }}.spdx.json",
    "riauth-platform-linux-${{ matrix.arch }}.spdx.json",
    "riauthctl-linux-${{ matrix.arch }}.spdx.json",
)
SUBJECT_LISTS = (
    "all-subjects.txt",
    "essentials-subjects.txt",
    "platform-subjects.txt",
    "riauthctl-subjects.txt",
)
PRODUCER_PATTERNS = (
    r"\bcosign\b",
    r"\bsyft\b",
    r"\bcyclonedx\b",
    r"\bspdx\b",
    r"\bin-toto\b",
    r"\bgpg\b",
    r"\bminisign\b",
    r"\bssh-keygen\b",
    r"\bsigstore\b",
)
OUTPUT_MARKERS = (".sbom", ".spdx", ".cdx", ".sig", "cyclonedx")
OBSERVED_DRAFT_ASSETS = (
    "build-provenance.json",
    "riauth-linux-x86_64.docker.tar.gz",
    "riauth-linux-x86_64.tar.gz",
    "SHA256SUMS",
)
OBSERVED_RUN_ID = 36342719277
OBSERVED_RUN_SHA = "96e23e2dedf84db4e39091a004cbd6591acbec97"


class AttestationError(ValueError):
    """The workflow source or the supplied bundle does not meet the pin."""


def require(condition, message):
    if not condition:
        raise AttestationError(message)


def sha256_file(path):
    checksum = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            checksum.update(chunk)
    return checksum.hexdigest()


def certificate_identity(tag):
    return f"https://github.com/{REPOSITORY}/{WORKFLOW}@refs/tags/{tag}"


def invocation_id(run_id, run_attempt):
    return f"https://github.com/{REPOSITORY}/actions/runs/{run_id}/attempts/{run_attempt}"


def workflow_producer_hits(text):
    """Reject signing producers. Package-document filenames are the SPDX subjects."""
    scanned = text
    for name in PACKAGE_DOCUMENTS:
        scanned = scanned.replace(name, "package-document")
    hits = []
    for pattern in PRODUCER_PATTERNS:
        if re.search(pattern, scanned, re.I):
            hits.append(f"{WORKFLOW}: {pattern}")
    lowered = scanned.lower()
    for marker in OUTPUT_MARKERS:
        if marker in lowered:
            hits.append(f"{WORKFLOW}: output marker {marker}")
    if "spdx-sbom" in lowered:
        hits.append(f"{WORKFLOW}: spdx-sbom")
    return hits


def require_release_workflow(text):
    """Fail closed unless the workflow requests only the pinned public-good action."""
    hits = workflow_producer_hits(text)
    require(not hits, "release path names a signing or SBOM producer: " + ", ".join(hits))
    require(
        "\npermissions:\n  contents: read\n\nconcurrency:" in text,
        "workflow permissions are not contents read",
    )
    require(text.count("id-token: write") == 1, "id-token write is not limited to the package job")
    require(text.count("attestations: write") == 1, "attestations write is not limited to the package job")
    require(text.count("contents: write") == 1, "contents write is not limited to the draft publish job")
    require("packages: write" not in text, "workflow requests package registry write")
    require("artifact-metadata" not in text, "workflow requests artifact metadata write")
    require("push-to-registry" not in text, "workflow requests registry publication")
    require("github-token:" not in text, "workflow overrides the attestation token")
    require(
        text.count(
            "    permissions:\n"
            "      id-token: write\n"
            "      contents: read\n"
            "      attestations: write\n"
        ) == 1,
        "package job permissions are not the attestation set",
    )
    publish = text.split("\n  publish:\n", 1)
    require(len(publish) == 2, "publish job is missing")
    require("id-token:" not in publish[1], "publish job requests an OIDC token")
    require("attestations:" not in publish[1], "publish job requests attestations")
    require(text.count(ATTEST_USES) == 4, "workflow does not pin four official attest actions")
    require(
        "actions/attest@" not in text.replace(ATTEST_USES, ""),
        "release workflow pins an unexpected attest action",
    )
    require(text.count("sbom-path:") == 3, "workflow does not request three package inventory attestations")
    require(
        text.count("subject-checksums:") == 4,
        "workflow does not pin four subject digest lists",
    )
    for name in PACKAGE_DOCUMENTS:
        require(text.count(name) == 1, f"workflow subject is missing {name}")
    for name in SUBJECT_LISTS:
        require(
            text.count("${{ runner.temp }}/riauth-subjects/" + name) == 1,
            f"subject digest list is missing {name}",
        )
    require(
        'sha256sum -- * > "$tmp/all-subjects.txt"' in text,
        "provenance subjects are not the packaged directory bytes",
    )
    provenance = text.split("- name: Attest packaged file provenance", 1)
    require(len(provenance) == 2, "provenance attestation step is missing")
    provenance_step = provenance[1].split("\n      - name:", 1)[0]
    require("sbom-path:" not in provenance_step, "provenance step also carries an inventory predicate")
    require("spdx_sbom.py" not in text and "package-linux" not in text, "release workflow names the source producer")
    return {
        "repository": REPOSITORY,
        "workflow": WORKFLOW,
        "action": ATTEST_USES,
        "sigstore_instance": "public-good",
        "oidc_issuer": OIDC_ISSUER,
        "signed_artifact": False,
    }


def statement_from_bundle(bundle):
    if not isinstance(bundle, dict) or bundle.get("mediaType") != BUNDLE_MEDIA_TYPE:
        raise AttestationError("bundle schema is invalid")
    material = bundle.get("verificationMaterial")
    if not isinstance(material, dict) or "certificate" not in material:
        raise AttestationError("missing proof")
    envelope = bundle.get("dsseEnvelope")
    if not isinstance(envelope, dict):
        raise AttestationError("missing proof")
    signatures = envelope.get("signatures")
    if not isinstance(signatures, list) or not any(
        isinstance(item, dict) and isinstance(item.get("sig"), str) and item["sig"]
        for item in signatures
    ):
        raise AttestationError("missing proof")
    payload = envelope.get("payload")
    if not isinstance(payload, str) or not payload:
        raise AttestationError("bundle schema is invalid")
    try:
        decoded = base64.b64decode(payload, validate=True)
        statement = json.loads(decoded)
    except (binascii.Error, json.JSONDecodeError, ValueError) as error:
        raise AttestationError("bundle schema is invalid") from error
    if not isinstance(statement, dict) or not isinstance(statement.get("subject"), list):
        raise AttestationError("bundle schema is invalid")
    return statement


def require_subject(statement, artifact_name, artifact_sha):
    digests = []
    for subject in statement["subject"]:
        if not isinstance(subject, dict) or not isinstance(subject.get("digest"), dict):
            raise AttestationError("bundle schema is invalid")
        digest = subject["digest"].get("sha256")
        if not isinstance(digest, str) or re.fullmatch(r"[0-9a-f]{64}", digest) is None:
            raise AttestationError("bundle schema is invalid")
        digests.append(digest)
        if subject.get("name") == artifact_name and digest == artifact_sha:
            return
    if artifact_sha not in digests:
        raise AttestationError("subject digest mismatch")
    raise AttestationError("subject name mismatch")


def require_spdx_predicate(predicate, artifact_name, artifact_sha):
    if not isinstance(predicate, dict) or predicate.get("spdxVersion") != "SPDX-2.3":
        raise AttestationError("predicate type mismatch")
    packages = predicate.get("packages")
    if not isinstance(packages, list):
        raise AttestationError("bundle schema is invalid")
    for package in packages:
        if not isinstance(package, dict) or package.get("name") != artifact_name:
            continue
        checksums = package.get("checksums")
        if not isinstance(checksums, list):
            continue
        for item in checksums:
            if (
                isinstance(item, dict)
                and item.get("algorithm") == "SHA256"
                and item.get("checksumValue") == artifact_sha
            ):
                return
    raise AttestationError("subject digest mismatch")


def expected_certificate(tag, source, run_id, run_attempt):
    """Certificate fields gh parses from signature.certificate. Not the predicate."""
    return {
        "subjectAlternativeName": certificate_identity(tag),
        "issuer": OIDC_ISSUER,
        "buildSignerDigest": source,
        "sourceRepositoryDigest": source,
        "sourceRepositoryRef": f"refs/tags/{tag}",
        "sourceRepositoryURI": f"https://github.com/{REPOSITORY}",
        "runnerEnvironment": "github-hosted",
        "runInvocationURI": invocation_id(run_id, run_attempt),
    }


def require_certificate(result, tag, source, run_id, run_attempt):
    """Pin the non-forgeable certificate summary and a witnessed timestamp."""
    signature = result.get("signature")
    if not isinstance(signature, dict):
        raise AttestationError("missing proof")
    certificate = signature.get("certificate")
    if not isinstance(certificate, dict):
        raise AttestationError("missing proof")
    timestamps = result.get("verifiedTimestamps")
    if not isinstance(timestamps, list) or not timestamps:
        raise AttestationError("missing proof")
    for item in timestamps:
        if (
            not isinstance(item, dict)
            or not isinstance(item.get("type"), str)
            or not item["type"]
            or not isinstance(item.get("uri"), str)
            or not item["uri"]
        ):
            raise AttestationError("missing proof")
    for key, value in expected_certificate(tag, source, run_id, run_attempt).items():
        if certificate.get(key) != value:
            raise AttestationError("certificate identity mismatch")


def require_verification_result(results, artifact_name, artifact_sha, tag, source, run_id,
                                run_attempt, predicate_type):
    """Apply pins to a GitHub CLI verification result. This does not call gh.

    Certificate fields are checked before the predicate. gh documents the
    predicate as workflow-controlled, so an invocation id or SPDX checksum
    match does not establish the signer.
    """
    if not isinstance(results, list) or not results:
        raise AttestationError("missing proof")
    for item in results:
        if not isinstance(item, dict):
            raise AttestationError("bundle schema is invalid")
        result = item.get("verificationResult")
        if not isinstance(result, dict):
            raise AttestationError("missing proof")
        require_certificate(result, tag, source, run_id, run_attempt)
        statement = result.get("statement")
        if not isinstance(statement, dict):
            raise AttestationError("bundle schema is invalid")
        require(statement.get("predicateType") == predicate_type, "predicate type mismatch")
        require_subject(statement, artifact_name, artifact_sha)
        if predicate_type == PROVENANCE_PREDICATE:
            predicate = statement.get("predicate")
            run_details = predicate.get("runDetails") if isinstance(predicate, dict) else None
            metadata = run_details.get("metadata") if isinstance(run_details, dict) else None
            expected = invocation_id(run_id, run_attempt)
            if not isinstance(metadata, dict) or metadata.get("invocationId") != expected:
                raise AttestationError("workflow run mismatch")
        else:
            require_spdx_predicate(statement.get("predicate"), artifact_name, artifact_sha)


def gh_verify_command(artifact, bundle, trusted_root, tag, source, predicate_type):
    return [
        "gh", "attestation", "verify", str(artifact),
        "--repo", REPOSITORY,
        "--cert-identity", certificate_identity(tag),
        "--cert-oidc-issuer", OIDC_ISSUER,
        "--source-ref", f"refs/tags/{tag}",
        "--source-digest", source,
        "--signer-digest", source,
        "--deny-self-hosted-runners",
        "--bundle", str(bundle),
        "--custom-trusted-root", str(trusted_root),
        "--predicate-type", predicate_type,
        "--format", "json",
    ]


def verify_attestation(artifact, bundle, trusted_root, tag, source, run_id, run_attempt,
                       predicate_type, runner=None):
    artifact = pathlib.Path(artifact)
    bundle = pathlib.Path(bundle)
    trusted_root = pathlib.Path(trusted_root)
    require(TAG_RE.fullmatch(tag) is not None, "release tag is missing or invalid")
    require(SOURCE_RE.fullmatch(source) is not None, "source commit is missing or invalid")
    require(RUN_RE.fullmatch(run_id) is not None and RUN_RE.fullmatch(run_attempt) is not None,
            "workflow run identity is missing or invalid")
    require(predicate_type in PREDICATES, "predicate type is not a release predicate")
    require(artifact.is_file() and not artifact.is_symlink(), "artifact is not a regular file")
    require(bundle.is_file() and not bundle.is_symlink(), "missing proof")
    require(
        trusted_root.is_file() and not trusted_root.is_symlink() and trusted_root.stat().st_size > 0,
        "trusted root is missing",
    )
    try:
        parsed = json.loads(bundle.read_text())
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise AttestationError("bundle schema is invalid") from error
    statement = statement_from_bundle(parsed)
    artifact_sha = sha256_file(artifact)
    require_subject(statement, artifact.name, artifact_sha)
    command = gh_verify_command(artifact, bundle, trusted_root, tag, source, predicate_type)
    if runner is None:
        completed = subprocess.run(command, text=True, capture_output=True)
        if completed.returncode != 0:
            detail = completed.stderr.strip().splitlines()
            message = detail[-1] if detail else "attestation rejected"
            raise AttestationError(f"attestation rejected: {message}")
        try:
            results = json.loads(completed.stdout)
        except json.JSONDecodeError as error:
            raise AttestationError("attestation rejected: verification result is not JSON") from error
    else:
        results = runner(command)
    require_verification_result(
        results, artifact.name, artifact_sha, tag, source, run_id, run_attempt, predicate_type,
    )
    return {
        "repository": REPOSITORY,
        "workflow": WORKFLOW,
        "tag": tag,
        "source": source,
        "run_id": run_id,
        "run_attempt": run_attempt,
        "predicate_type": predicate_type,
        "subject_sha256": artifact_sha,
        "signed_artifact": False,
    }


def classify_public_observation(repository, release, run, artifact_count):
    """Classify one read-only GitHub response. It is not a release result."""
    require(isinstance(repository, dict), "repository response schema is invalid")
    require(repository.get("full_name") == REPOSITORY, "repository response is a different repository")
    require(repository.get("private") is False, "repository is not public")
    require(repository.get("archived") is False, "repository is archived")
    require(repository.get("visibility") == "public", "repository visibility is not public")
    require(repository.get("default_branch") == "main", "default branch is not main")
    require(isinstance(release, dict), "release response schema is invalid")
    require(release.get("tagName") == "v0.1.1", "observed draft tag changed")
    require(release.get("isDraft") is True, "observed v0.1.1 is not a draft")
    assets = release.get("assets")
    require(isinstance(assets, list), "release asset schema is invalid")
    names = sorted(item.get("name") for item in assets if isinstance(item, dict))
    require(names == sorted(OBSERVED_DRAFT_ASSETS), "observed draft assets changed")
    require(isinstance(run, dict), "workflow run schema is invalid")
    require(run.get("databaseId") == OBSERVED_RUN_ID, "observed workflow run changed")
    require(run.get("headSha") == OBSERVED_RUN_SHA, "observed workflow run commit changed")
    require(run.get("conclusion") == "success", "observed public CI run did not succeed")
    require(run.get("workflowName") == "Public CI", "observed run is not the public CI workflow")
    require(artifact_count == 0, "observed public CI run uploaded artifacts")
    return {
        "repository": REPOSITORY,
        "visibility": "public",
        "archived": False,
        "default_branch": "main",
        "draft_tag": "v0.1.1",
        "draft_assets": list(OBSERVED_DRAFT_ASSETS),
        "historical_draft_incomplete": True,
        "current_accepted_release": False,
        "public_ci_run": OBSERVED_RUN_ID,
        "public_ci_sha": OBSERVED_RUN_SHA,
        "public_ci_artifact_count": 0,
        "signed_artifact": False,
        "release_executed": False,
    }


def fetch_public_observation():
    repository = json.loads(subprocess.check_output(
        ["gh", "api", f"repos/{REPOSITORY}"], text=True,
    ))
    release = json.loads(subprocess.check_output(
        ["gh", "release", "view", "v0.1.1", "--repo", REPOSITORY, "--json", "tagName,isDraft,assets"],
        text=True,
    ))
    run = json.loads(subprocess.check_output(
        [
            "gh", "run", "view", str(OBSERVED_RUN_ID), "--repo", REPOSITORY,
            "--json", "databaseId,headSha,conclusion,workflowName",
        ],
        text=True,
    ))
    artifacts = json.loads(subprocess.check_output(
        ["gh", "api", f"repos/{REPOSITORY}/actions/runs/{OBSERVED_RUN_ID}/artifacts"],
        text=True,
    ))
    return classify_public_observation(repository, release, run, artifacts.get("total_count"))


def read_workflow(root):
    path = pathlib.Path(root) / WORKFLOW
    if not path.is_file():
        raise AttestationError(f"missing {WORKFLOW}")
    return path.read_text()


def main(argv=None):
    parser = argparse.ArgumentParser(description="Check release attestation pins. Exit 0 is not a release result.")
    commands = parser.add_subparsers(dest="command", required=True)
    static = commands.add_parser("static")
    static.add_argument("--root", default=".")
    verify = commands.add_parser("verify")
    verify.add_argument("--artifact", required=True)
    verify.add_argument("--bundle", required=True)
    verify.add_argument("--trusted-root", required=True)
    verify.add_argument("--tag", required=True)
    verify.add_argument("--source", required=True)
    verify.add_argument("--run-id", required=True)
    verify.add_argument("--run-attempt", required=True)
    verify.add_argument("--predicate", required=True, choices=PREDICATES)
    commands.add_parser("live-public")
    args = parser.parse_args(argv)
    try:
        if args.command == "static":
            report = require_release_workflow(read_workflow(args.root))
        elif args.command == "live-public":
            report = fetch_public_observation()
        else:
            report = verify_attestation(
                args.artifact, args.bundle, args.trusted_root, args.tag, args.source,
                args.run_id, args.run_attempt, args.predicate,
            )
    except AttestationError as error:
        print(f"release attestation check failed: {error}", file=sys.stderr)
        return 1
    except (OSError, subprocess.CalledProcessError, json.JSONDecodeError) as error:
        print(f"release attestation check failed: {error}", file=sys.stderr)
        return 1
    json.dump(report, sys.stdout, indent=2)
    print()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
