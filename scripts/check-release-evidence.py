#!/usr/bin/env python3
"""Audit source-backed vulnerability intake and release-evidence boundaries.

Reads a checkout. Does not build, sign, publish, download assets, or decide
that a release succeeded. --dist lists filenames only. scripts/spdx_sbom.py
is a source producer; a successful audit still reports no release SBOM.
"""

import argparse
import hashlib
import json
import pathlib
import platform
import re
import subprocess
import sys


SCHEMA = "riauth.release-evidence/v1"
AUDITED = (
    "SECURITY.md",
    "docs/release-notes.md",
    "docs/roadmap/q11-release-evidence.md",
    ".github/workflows/release.yml",
    "scripts/package-release.sh",
    "scripts/check-release-bundle.py",
    "scripts/generate-third-party-notices.py",
    "scripts/spdx_sbom.py",
)
SOURCE_PRODUCER = "scripts/spdx_sbom.py"
PRODUCER_HONESTY = (
    "This output is not a release SBOM.",
    "This document is not a build attestation.",
    "Crate archives were not fetched.",
    "created is 1970-01-01T00:00:00Z because the document is a pure function of its inputs, not a build time.",
)
INTAKE_SENTENCES = (
    "Security fixes are considered for the latest v0.1.x release.",
    "https://github.com/Rhein-Industries/riAuth/security/advisories",
    "without sharing vulnerability details",
    "We will acknowledge reports and coordinate a fix and disclosure with the reporter.",
    "No response or remediation deadline is promised for this initial release.",
)
NOTES_SENTENCES = (
    "Maintainers review the tagged commit, checks, smoke tests, and draft assets before publication.",
    "The provenance records build metadata; it is not a cryptographic attestation.",
)
PRODUCER_PATTERNS = tuple(
    re.compile(pattern, re.I)
    for pattern in (
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
)
OUTPUT_MARKERS = (".sbom", ".spdx", ".cdx", ".sig", "cyclonedx")
PROCEDURE_PHRASES = (
    "response within",
    "sla",
    "signed artifact",
    "signature verified",
    "sbom generated",
    "sbom attached",
    "independent review completed",
    "independently reviewed",
    "release succeeded",
    "release published",
    "linux arm64 passed",
    "arm64 evidence recorded",
)
EMAIL = re.compile(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}")
MAILTO = re.compile(r"mailto:([^\s)>]+)", re.I)
RELEASE_PATHS = (
    ".github/workflows/release.yml",
    "scripts/package-release.sh",
    "scripts/check-release-bundle.py",
)


class AuditError(ValueError):
    """The checkout no longer matches the source evidence contract."""


def compacted(text):
    return re.sub(r"\s+", " ", text).strip()


def require(condition, message):
    if not condition:
        raise AuditError(message)


def digest(path):
    checksum = hashlib.sha256()
    checksum.update(path.read_bytes())
    return checksum.hexdigest()


def read_text(root, relative):
    path = root / relative
    if not path.is_file():
        raise AuditError(f"missing {relative}")
    return path.read_text()


def contacts(text):
    found = set(EMAIL.findall(text))
    found.update(MAILTO.findall(text))
    return sorted(found)


def worktree_state(root):
    try:
        commit = subprocess.check_output(
            ["git", "-C", str(root), "rev-parse", "HEAD"],
            text=True,
            stderr=subprocess.DEVNULL,
        ).strip()
        status = subprocess.check_output(
            ["git", "-C", str(root), "status", "--porcelain"],
            text=True,
            stderr=subprocess.DEVNULL,
        )
    except (OSError, subprocess.CalledProcessError):
        return {"commit": None, "dirty": None}
    return {"commit": commit, "dirty": bool(status.strip())}


def inventory(dist):
    if dist is None:
        return {"status": "not_requested", "verification": "not performed"}
    if not dist.exists():
        return {"status": "unavailable", "verification": "not performed"}
    require(dist.is_dir(), f"release directory is not a directory: {dist}")
    names = sorted(path.name for path in dist.iterdir() if path.is_file())
    return {
        "status": "names_only",
        "present": names,
        "verification": "not performed",
    }


def audit(root, dist=None):
    """Return the evidence report or raise AuditError."""
    root = pathlib.Path(root)
    texts = {relative: read_text(root, relative) for relative in AUDITED}
    security = texts["SECURITY.md"]
    notes = texts["docs/release-notes.md"]
    workflow = texts[".github/workflows/release.yml"]
    packager = texts["scripts/package-release.sh"]
    bundle = texts["scripts/check-release-bundle.py"]
    notices = texts["scripts/generate-third-party-notices.py"]
    procedure = texts["docs/roadmap/q11-release-evidence.md"]

    for sentence in INTAKE_SENTENCES:
        require(sentence in compacted(security), f"SECURITY.md is missing intake text: {sentence}")
    for sentence in NOTES_SENTENCES:
        require(sentence in compacted(notes), f"release notes are missing: {sentence}")

    create_lines = [line.strip() for line in workflow.splitlines() if "gh release create" in line]
    require(len(create_lines) == 1, "release workflow must contain one gh release create command")
    for token in ("--draft", "--verify-tag", "--notes-file docs/release-notes.md"):
        require(token in create_lines[0], f"draft release command is missing {token}")
    require("check-release-bundle.py" in workflow, "release workflow does not run check-release-bundle.py")
    for token in ("runner: ubuntu-24.04", "runner: ubuntu-24.04-arm", "arch: x86_64", "arch: aarch64"):
        require(token in workflow, f"release workflow is missing declared target text: {token}")

    producers = []
    for relative in RELEASE_PATHS:
        lowered = texts[relative].lower()
        for pattern in PRODUCER_PATTERNS:
            if pattern.search(texts[relative]):
                producers.append(f"{relative}: {pattern.pattern}")
        for marker in OUTPUT_MARKERS:
            if marker in lowered:
                producers.append(f"{relative}: output marker {marker}")
        for token in ("spdx_sbom.py", "spdx-sbom"):
            if token in lowered:
                producers.append(f"{relative}: {token}")
    require(not producers, "release path names a signing or SBOM producer: " + ", ".join(producers))

    for token in (
        "'schema': 'riauth.build/v4'",
        "SHA256SUMS-linux-",
        "build-provenance-linux-",
        'test "$(uname -s)" = Linux',
    ):
        require(token in packager, f"packager is missing {token}")
    require(
        'print("Complete x86_64 and aarch64 release bundle verified")' in bundle,
        "bundle checker success text changed; this audit is not that result",
    )
    require(
        'OUTPUT = ROOT / "THIRD_PARTY_NOTICES.md"' in notices,
        "third-party notice generator does not name THIRD_PARTY_NOTICES.md",
    )
    spdx_lines = [line for line in notices.splitlines() if re.search(r"spdx", line, re.I)]
    require(len(spdx_lines) == 2, "notice generator SPDX mentions changed")
    require(
        any("A crate's SPDX expression describes its available" in line for line in spdx_lines)
        and any("| Crate | Version | SPDX expression |" in line for line in spdx_lines),
        "notice generator SPDX mentions are not the license-expression column",
    )
    require(
        re.search(r"\b(sbom|cyclonedx)\b", notices, re.I) is None,
        "third-party notice generator now refers to an SBOM document",
    )

    procedure_text = compacted(procedure).lower()
    procedure_hits = [
        phrase for phrase in PROCEDURE_PHRASES
        if re.search(rf"(?<!\w){re.escape(phrase)}(?!\w)", procedure_text)
    ]
    require(
        not procedure_hits,
        "procedure claims evidence this audit does not have: " + ", ".join(procedure_hits),
    )
    require("scripts/check-release-evidence.py" in procedure, "procedure does not name the verify-only checker")
    require("SECURITY.md" in procedure, "procedure does not cite SECURITY.md")
    require("Exit 0 is not a release result." in procedure, "procedure no longer says exit 0 is not a release result")
    require(SOURCE_PRODUCER in procedure, "procedure does not name the source SPDX producer")
    require(
        "This output is not a release SBOM." in procedure,
        "procedure no longer says this output is not a release SBOM",
    )
    require(
        "No release SBOM was produced in this slice." in procedure,
        "procedure no longer says no release SBOM was produced",
    )
    require("This output is a release SBOM." not in procedure, "procedure claims a release SBOM")
    producer_text = texts[SOURCE_PRODUCER]
    for sentence in PRODUCER_HONESTY:
        require(sentence in producer_text, f"source producer is missing honesty text: {sentence}")
    require("--locked" in producer_text, "source producer does not pass --locked")
    require("--offline" in producer_text, "source producer does not pass --offline")
    require("This output is a release SBOM." not in producer_text, "source producer claims a release SBOM")

    policy_contacts = contacts(security)
    invented = [item for item in contacts(procedure) if item not in policy_contacts]
    require(
        not invented,
        "procedure names a contact that is not in SECURITY.md: " + ", ".join(invented),
    )

    return {
        "schema": SCHEMA,
        "worktree": worktree_state(root),
        "checker_host": {
            "system": platform.system(),
            "machine": platform.machine(),
            "runs_release_packager": False,
        },
        "vulnerability_intake": {
            "policy": "SECURITY.md",
            "advisory_url": "https://github.com/Rhein-Industries/riAuth/security/advisories",
            "public_fallback": "issue requesting a private contact, without vulnerability details",
            "security_emails_in_policy": policy_contacts,
            "response_deadline": "none promised for this initial release",
            "reports_received_by_this_check": False,
        },
        "signing": {"present_in_release_path": False, "signed_artifact": False},
        "sbom": {
            "spdx_or_cyclonedx_document": False,
            "third_party_notices": "THIRD_PARTY_NOTICES.md",
            "notices_include_spdx_license_expressions": True,
            "notices_are_an_sbom": False,
            "source_producer": SOURCE_PRODUCER,
            "source_producer_in_release_workflow": False,
            "release_sbom_produced": False,
        },
        "independent_review_record": False,
        "provenance": {
            "schema": "riauth.build/v4",
            "cryptographic_attestation": False,
            "checksum_files": "SHA256SUMS-linux-*",
        },
        "publication": {
            "command": "gh release create --draft --verify-tag",
            "published_by_this_check": False,
        },
        "linux_targets_declared_in_workflow": ["x86_64", "aarch64"],
        "linux_arm64_execution": "not run",
        "release_executed": False,
        "bundle_checker": {
            "script": "scripts/check-release-bundle.py",
            "ran_against_assets": False,
        },
        "release_assets": inventory(None if dist is None else pathlib.Path(dist)),
        "files": {relative: digest(root / relative) for relative in AUDITED},
    }


def main(argv=None):
    parser = argparse.ArgumentParser(description="Verify source release-evidence boundaries.")
    parser.add_argument("--root", default=".", help="checkout to read")
    parser.add_argument("--dist", help="optional directory whose filenames are listed and not verified")
    args = parser.parse_args(argv)
    try:
        report = audit(args.root, args.dist)
    except AuditError as error:
        print(f"release evidence audit failed: {error}", file=sys.stderr)
        return 1
    json.dump(report, sys.stdout, indent=2)
    print()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
