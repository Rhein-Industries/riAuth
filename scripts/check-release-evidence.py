#!/usr/bin/env python3
"""Audit source-backed vulnerability intake and release-evidence boundaries.

Reads a checkout. Does not build, sign, publish, download assets, or decide
that a release succeeded. --dist lists filenames only. scripts/spdx_sbom.py
is a source producer. The Linux packager source calls it, and the asset
checkers require its package documents. A successful audit still reports no
release SBOM.
"""

import argparse
import ast
import copy
import hashlib
import inspect
import json
import os
import pathlib
import platform
import re
import stat
import subprocess
import sys


MAX_SOURCE_BYTES = 1024 ** 2
MAX_SOURCE_NODES = 50_000
MAX_SOURCE_DEPTH = 128

# Canonical source declarations and selected complete bodies; locations/comments are excluded.
SOURCE_CONTRACTS = {
    "scripts/spdx_sbom.py": {
        "functions": {
            "linux_graphs": "62c78c739b601205ac7950fef5a81fd3b6af3eabf3508017953cd2a8cfcd94ca",
            "linux_spdx_names": "4101b935efa4aee88b095378b3adb7cefc63a237782531bb6eaf7809063f714c"
        },
        "bindings": {
            "LINUX_TARGETS": "0b9558092d70f2f9c1fe07a087d50b0fc17ab98f4e4480bff8508be2ae8508ba",
            "SpdxError": "a76c42a8775627c5b6856d674263dfc30b04b0f95bfdb37532cd2079631263bb"
        },
        "declarations_sha256": "4dd26aea753a2973e227fff1bff4a6114ec2710973247913ada6ba2c3a53e6d0"
    },
    "scripts/check-installed-release-gate.py": {
        "functions": {
            "_package_documents": "c1622ae977a8f52dc4539a6c1ae4c0e9626ff2cc0f99e2edfd93d5ab89d79dbe",
            "expected_names": "c285fea9eaee2d5b45accdb6b4e298e67152c0e309bdcdccf0ff8241e9c9daff"
        },
        "bindings": {
            "pathlib": "fd65a776d7fef693633776fffe83ef87a35c5cd3d1c6f522b4bb47ab2568a1e1",
            "importlib": "866a97c0a29d398dc2bf8a74409548dccd9bb5221f4bf0f58ad1a37ef83e5260",
            "EDITIONS": "0c0c081b5d2157eea6266260c3f3f8bcb0e3d88a14958f3cb16df42a02d383df",
            "package_documents": "f61ad47aaee354faeb76d632d9bdc6af37d4659229f591541120883a3f93a51b"
        },
        "declarations_sha256": "19c2a947facb5460c60201ac5bcf27062d16c3e8a936e958d946bfb66697b0b0"
    },
    "scripts/check-release-attestation.py": {
        "functions": {
            "require": "17455206cd1189919470aecc2963643d195cef8e82fbf46b6deb9c2ad02fc8ad",
            "workflow_producer_hits": "d8ae004b186020c7a403fd2cb820ca425282da85c46925178d880f8ac3f14462",
            "require_release_workflow": "7cca1c076f227812d9177315232b219f2611da440dce54986391eb4ddacc175f"
        },
        "bindings": {
            "re": "6b81b4ca19c88ec507aad63f1f49291209368a9490113655a68e994c91d092c1",
            "AttestationError": "7139504770cebe7378a4b7259a3d0d8bbf7a14024e842063a19d51b0e0d9c4dc",
            "REPOSITORY": "e65e3090a05381ee2a971d3728ca5a6eda2bd87b3b7b4180cc7deecf816e8ad9",
            "WORKFLOW": "2783fc0f88de22aff1d41ff3d3fc88a7554c0b1d3fdc2bbd7b216d622791aec5",
            "ACTION_SHA": "5e88555d534e54e9e64a6a3a375c434b9627821f8e8fd8792d87a6fcc1689dc7",
            "ATTEST_USES": "0f9549ebc30865d12f8c43dfd49f41415c407caa10ab9bb7b231d8aaedf1bdd2",
            "OIDC_ISSUER": "4814a6aac4e2a9374ea0344081c8eaa4826e26620f09b5dfaeffd41ad701c869",
            "PACKAGE_DOCUMENTS": "be2eb77d4b90ef2d4ca3950dba85099ef8ba785898e0c85c23392ee0e01c9246",
            "SUBJECT_LISTS": "5a958e13d19ac6bb790ca1d8fb4ce7376f9e1c0256778f3b593bffd1d5c9fa8e",
            "PRODUCER_PATTERNS": "e92ca9bd4cbfcb27192f907e36fdc997f00b2468cd24a33f5d71c2ad2e50e10c",
            "OUTPUT_MARKERS": "f3ed2acb358b6096f203b4156475a32a51e4d70ce45ffe56b70872c1b0bccb75"
        },
        "declarations_sha256": "a8b00976f2d08a75144738bfb923d2dcf37e5f99ad3ff5dd94ce277cff3e98e9"
    }
}


# Auditor-owned policy pins, independent of the checkout being inspected.
ATTEST_REPOSITORY = "Rhein-Industries/riAuth"
ATTEST_WORKFLOW = ".github/workflows/release.yml"
ACTION_SHA = "1e69f48acb82d1966a394da916b4c1698aa569d6"
ATTEST_USES = f"actions/attest@{ACTION_SHA} # v4.2.2"
ATTEST_OIDC_ISSUER = "https://token.actions.githubusercontent.com"
ATTEST_PACKAGE_DOCUMENTS = (
    "riauth-essentials-linux-${{ matrix.arch }}.spdx.json",
    "riauth-platform-linux-${{ matrix.arch }}.spdx.json",
    "riauthctl-linux-${{ matrix.arch }}.spdx.json",
)
ATTEST_SUBJECT_LISTS = (
    "all-subjects.txt",
    "essentials-subjects.txt",
    "platform-subjects.txt",
    "riauthctl-subjects.txt",
)
ATTEST_PRODUCER_PATTERNS = (
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

PACKAGE_TARGETS = {
    "x86_64": "x86_64-unknown-linux-gnu",
    "aarch64": "aarch64-unknown-linux-gnu",
}
PACKAGE_EDITIONS = ("essentials", "platform")


SCHEMA = "riauth.release-evidence/v1"
AUDITED = (
    "SECURITY.md",
    "docs/release-notes.md",
    "docs/roadmap/q11-release-evidence.md",
    ".github/workflows/release.yml",
    "scripts/package-release.sh",
    "scripts/check-release-bundle.py",
    "scripts/check-installed-release-gate.py",
    "scripts/generate-third-party-notices.py",
    "scripts/spdx_sbom.py",
    "scripts/check-release-attestation.py",
)
SOURCE_PRODUCER = "scripts/spdx_sbom.py"
PACKAGE_CALL = "\n".join((
    "python3 scripts/spdx_sbom.py package-linux \\",
    '  --arch "$RIAUTH_ARCH" \\',
    '  --dist "$riauth_dist" \\',
    "  --server-manifest Cargo.toml \\",
    "  --server-lock Cargo.lock \\",
    "  --client-manifest crates/riauthctl/Cargo.toml \\",
    "  --client-lock crates/riauthctl/Cargo.lock",
))
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
    "scripts/check-installed-release-gate.py",
)


class AuditError(ValueError):
    """The checkout no longer matches the source evidence contract."""


def compacted(text):
    return re.sub(r"\s+", " ", text).strip()


def require(condition, message):
    if not condition:
        raise AuditError(message)


def source_bytes(path, relative):
    nofollow = getattr(os, "O_NOFOLLOW", None)
    nonblock = getattr(os, "O_NONBLOCK", None)
    require(type(nofollow) is int and nofollow != 0
            and type(nonblock) is int and nonblock != 0,
            "source read protections unavailable")
    try:
        before = path.lstat()
    except FileNotFoundError as error:
        raise AuditError(f"missing {relative}") from error
    except OSError as error:
        raise AuditError(f"could not read {relative}") from error
    require(stat.S_ISREG(before.st_mode) and before.st_nlink == 1,
            f"nonregular source {relative}")
    require(before.st_size <= MAX_SOURCE_BYTES, f"source size limit {relative}")
    try:
        descriptor = os.open(path, os.O_RDONLY | nofollow | nonblock)
        with os.fdopen(descriptor, "rb") as stream:
            opened = os.fstat(stream.fileno())
            require(stat.S_ISREG(opened.st_mode) and opened.st_nlink == 1,
                    f"nonregular source {relative}")
            payload = stream.read(MAX_SOURCE_BYTES + 1)
            after = os.fstat(stream.fileno())
        current = path.lstat()
    except OSError as error:
        raise AuditError(f"could not read {relative}") from error
    identity = lambda info: (info.st_dev, info.st_ino, info.st_size,
                             info.st_mtime_ns, info.st_ctime_ns)
    require(identity(before) == identity(opened) == identity(after) == identity(current)
            and len(payload) == before.st_size and len(payload) <= MAX_SOURCE_BYTES,
            f"source changed {relative}")
    return payload


def digest(path):
    return hashlib.sha256(source_bytes(path, path.name)).hexdigest()


def read_text(root, relative):
    try:
        return source_bytes(root / relative, relative).decode("utf-8")
    except UnicodeError as error:
        raise AuditError(f"invalid source text {relative}") from error


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


def release_denylist(relative, text):
    hits = []
    for pattern in PRODUCER_PATTERNS:
        if pattern.search(text):
            hits.append(f"{relative}: {pattern.pattern}")
    lowered = text.lower()
    for marker in OUTPUT_MARKERS:
        if marker in lowered:
            hits.append(f"{relative}: output marker {marker}")
    if "spdx-sbom" in lowered:
        hits.append(f"{relative}: spdx-sbom")
    return hits


def require_source_wiring(texts):
    hits = []
    for relative in RELEASE_PATHS:
        if relative == ".github/workflows/release.yml":
            continue
        hits.extend(release_denylist(relative, texts[relative]))
    require(not hits, "release path names a signing or SBOM producer: " + ", ".join(hits))
    workflow = texts[".github/workflows/release.yml"]
    require(
        workflow.count("spdx_sbom.py") == 0 and "package-linux" not in workflow,
        "release workflow names the source producer",
    )
    packager = texts["scripts/package-release.sh"]
    require(
        packager.count(PACKAGE_CALL) == 1 and packager.count("spdx_sbom.py") == 1,
        "packager must call the source producer once",
    )
    asset_checks = (
        (
            "scripts/check-release-bundle.py",
            "bundle checker",
            "package_documents.require_linux_package_bytes(root, arch)",
        ),
        (
            "scripts/check-installed-release-gate.py",
            "installed gate",
            "package_documents.require_linux_package_bytes(dist, arch)",
        ),
    )
    for relative, label, byte_call in asset_checks:
        text = texts[relative]
        require(
            text.count("spdx_sbom.py") == 1
            and text.count("package_documents.linux_spdx_names(arch)") == 1
            and text.count(byte_call) == 1
            and "package-linux" not in text,
            f"{label} source producer references changed",
        )


def source_binding_names(node):
    if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
        return (node.name,)
    if isinstance(node, ast.Import):
        return tuple(alias.asname or alias.name.split(".")[0] for alias in node.names)
    if isinstance(node, ast.ImportFrom):
        return tuple(alias.asname or alias.name for alias in node.names)
    if isinstance(node, ast.Assign):
        return tuple(target.id for target in node.targets if isinstance(target, ast.Name))
    if isinstance(node, ast.AnnAssign) and isinstance(node.target, ast.Name):
        return (node.target.id,)
    return ()


def source_ast_dump(node):
    normalized = copy.deepcopy(node)
    for item in ast.walk(normalized):
        parameters = getattr(item, "type_params", None)
        if type(parameters) is list and not parameters:
            delattr(item, "type_params")
    options = {"include_attributes": False}
    if "show_empty" in inspect.signature(ast.dump).parameters:
        options["show_empty"] = True
    return ast.dump(normalized, **options)


def source_declarations_sha256(tree):
    rows = []
    for node in tree.body:
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            rows.append([
                type(node).__name__, node.name, source_ast_dump(node.args),
                [source_ast_dump(item) for item in node.decorator_list],
                None if node.returns is None else source_ast_dump(node.returns),
                node.type_comment,
                [source_ast_dump(item) for item in getattr(node, "type_params", [])],
            ])
        else:
            rows.append(["declaration", source_ast_dump(node)])
    return hashlib.sha256(json.dumps(rows, separators=(",", ":"), ensure_ascii=True).encode("ascii")).hexdigest()


def recognize_source_contract(text, relative):
    contract = SOURCE_CONTRACTS.get(relative)
    require(contract is not None, f"unsupported source contract {relative}")
    require(type(text) is str and len(text.encode("utf-8")) <= MAX_SOURCE_BYTES,
            f"source size limit {relative}")
    try:
        tree = ast.parse(text, filename=relative)
        pending, count = [(tree, 0)], 0
        while pending:
            node, depth = pending.pop()
            count += 1
            require(count <= MAX_SOURCE_NODES and depth <= MAX_SOURCE_DEPTH,
                    f"source AST limit {relative}")
            pending.extend((child, depth + 1) for child in ast.iter_child_nodes(node))
        expected = contract["functions"] | contract["bindings"]
        protected = set(expected) | {"__file__", "__builtins__"}
        bindings, allowed_stores, allowed_declarations = {}, set(), set()
        for node in tree.body:
            for name in source_binding_names(node):
                if name in protected:
                    require(name not in bindings, f"rebound source contract {relative}")
                    bindings[name] = node
                    allowed_declarations.add(id(node))
                    if isinstance(node, ast.Assign):
                        allowed_stores.update(id(target) for target in node.targets
                                              if isinstance(target, ast.Name) and target.id == name)
                    elif isinstance(node, ast.AnnAssign):
                        allowed_stores.add(id(node.target))
        require(set(bindings) == set(expected), f"missing source contract {relative}")
        for node in ast.walk(tree):
            if isinstance(node, ast.Name) and isinstance(node.ctx, (ast.Store, ast.Del)):
                require(node.id not in protected or id(node) in allowed_stores,
                        f"rebound source contract {relative}")
            elif isinstance(node, (ast.Attribute, ast.Subscript)) and isinstance(node.ctx, (ast.Store, ast.Del)):
                base = node.value
                while isinstance(base, (ast.Attribute, ast.Subscript)):
                    base = base.value
                require(not isinstance(base, ast.Name) or base.id not in protected,
                        f"rebound source contract {relative}")
            elif isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef,
                                   ast.Import, ast.ImportFrom)):
                require(not isinstance(node, ast.ImportFrom)
                        or all(alias.name != "*" for alias in node.names),
                        f"unsupported source declarations {relative}")
                require(not protected.intersection(source_binding_names(node))
                        or id(node) in allowed_declarations, f"rebound source contract {relative}")
            elif isinstance(node, (ast.Global, ast.Nonlocal)):
                require(not protected.intersection(node.names), f"rebound source contract {relative}")
            elif isinstance(node, ast.arg):
                require(node.arg not in protected, f"rebound source contract {relative}")
        for name, expected_hash in expected.items():
            actual = hashlib.sha256(source_ast_dump(bindings[name]).encode()).hexdigest()
            require(actual == expected_hash, f"source contract changed {relative}: {name}")
        require(source_declarations_sha256(tree) == contract["declarations_sha256"],
                f"unsupported source declarations {relative}")
    except (SyntaxError, ValueError, TypeError, RecursionError, MemoryError) as error:
        if isinstance(error, AuditError):
            raise
        raise AuditError(f"could not parse {relative}") from error


def workflow_producer_hits(text):
    """Reject signing producers. Package-document filenames are the SPDX subjects."""
    scanned = text
    for name in ATTEST_PACKAGE_DOCUMENTS:
        scanned = scanned.replace(name, "package-document")
    hits = []
    for pattern in ATTEST_PRODUCER_PATTERNS:
        if re.search(pattern, scanned, re.I):
            hits.append(f"{ATTEST_WORKFLOW}: {pattern}")
    lowered = scanned.lower()
    for marker in OUTPUT_MARKERS:
        if marker in lowered:
            hits.append(f"{ATTEST_WORKFLOW}: output marker {marker}")
    if "spdx-sbom" in lowered:
        hits.append(f"{ATTEST_WORKFLOW}: spdx-sbom")
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
    for name in ATTEST_PACKAGE_DOCUMENTS:
        require(text.count(name) == 1, f"workflow subject is missing {name}")
    for name in ATTEST_SUBJECT_LISTS:
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
        "repository": ATTEST_REPOSITORY,
        "workflow": ATTEST_WORKFLOW,
        "action": ATTEST_USES,
        "sigstore_instance": "public-good",
        "oidc_issuer": ATTEST_OIDC_ISSUER,
        "signed_artifact": False,
    }

def package_graphs(arch):
    """Exact packaged files for each locked graph. Names only; no build is run."""
    target = PACKAGE_TARGETS.get(arch)
    if target is None:
        raise AuditError(f"unsupported package architecture: {arch}")
    essentials = f"riauth-essentials-linux-{arch}"
    platform = f"riauth-platform-linux-{arch}"
    return (
        {
            "id": "essentials",
            "output": f"{essentials}.spdx.json",
            "features": ("essentials",),
            "no_default_features": True,
            "lock": "server",
            "files": (
                f"{essentials}.tar.gz",
                f"riauth-maintenance-essentials-linux-{arch}.tar.gz",
                f"{essentials}.docker.tar.gz",
            ),
        },
        {
            "id": "platform",
            "output": f"{platform}.spdx.json",
            "features": ("essentials", "platform"),
            "no_default_features": True,
            "lock": "server",
            "files": (
                f"{platform}.tar.gz",
                f"riauth-maintenance-platform-linux-{arch}.tar.gz",
                f"{platform}.docker.tar.gz",
            ),
        },
        {
            "id": "riauthctl",
            "output": f"riauthctl-linux-{arch}.spdx.json",
            "features": (),
            "no_default_features": True,
            "lock": "client",
            "files": (f"riauthctl-linux-{arch}.tar.gz",),
        },
    )

def package_spdx_names(arch):
    return {graph["output"] for graph in package_graphs(arch)}

def installed_package_names(arch):
    return {f"riauth-{edition}-linux-{arch}.tar.gz" for edition in PACKAGE_EDITIONS} | {
        f"riauth-maintenance-{edition}-linux-{arch}.tar.gz" for edition in PACKAGE_EDITIONS
    } | {f"riauth-{edition}-linux-{arch}.docker.tar.gz" for edition in PACKAGE_EDITIONS} | {
        f"riauthctl-linux-{arch}.tar.gz", f"build-provenance-linux-{arch}.json",
        f"SHA256SUMS-linux-{arch}",
    } | package_spdx_names(arch)

def expected_package_graphs(arch):
    essentials = f"riauth-essentials-linux-{arch}"
    platform = f"riauth-platform-linux-{arch}"
    return (
        {
            "id": "essentials",
            "output": f"{essentials}.spdx.json",
            "features": ("essentials",),
            "no_default_features": True,
            "lock": "server",
            "files": (
                f"{essentials}.tar.gz",
                f"riauth-maintenance-essentials-linux-{arch}.tar.gz",
                f"{essentials}.docker.tar.gz",
            ),
        },
        {
            "id": "platform",
            "output": f"{platform}.spdx.json",
            "features": ("essentials", "platform"),
            "no_default_features": True,
            "lock": "server",
            "files": (
                f"{platform}.tar.gz",
                f"riauth-maintenance-platform-linux-{arch}.tar.gz",
                f"{platform}.docker.tar.gz",
            ),
        },
        {
            "id": "riauthctl",
            "output": f"riauthctl-linux-{arch}.spdx.json",
            "features": (),
            "no_default_features": True,
            "lock": "client",
            "files": (f"riauthctl-linux-{arch}.tar.gz",),
        },
    )


def require_package_contract(texts):
    """Recognize source as data, then compare the auditor-owned package policy."""
    recognize_source_contract(texts[SOURCE_PRODUCER], SOURCE_PRODUCER)
    recognize_source_contract(texts["scripts/check-installed-release-gate.py"],
                              "scripts/check-installed-release-gate.py")
    for arch in ("x86_64", "aarch64"):
        graphs = package_graphs(arch)
        require(graphs == expected_package_graphs(arch), f"{arch} package graphs changed")
        names = package_spdx_names(arch)
        require(names <= installed_package_names(arch), "installed gate omits package document names")


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

    require_source_wiring(texts)
    recognize_source_contract(texts["scripts/check-release-attestation.py"],
                              "scripts/check-release-attestation.py")
    require_release_workflow(workflow)

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
    require(
        "python3 scripts/spdx_sbom.py package-linux" in procedure,
        "procedure does not name the packager binding",
    )
    require(
        "The release workflow file does not name this producer." in procedure,
        "procedure no longer keeps the producer out of the release workflow",
    )
    require(
        "This slice did not execute the packager." in procedure,
        "procedure claims the packager ran",
    )
    require("packager_source_calls_producer" in procedure, "procedure omits the packager source field")
    require("asset_checks_require_package_spdx" in procedure, "procedure omits the asset-check field")
    require(
        "actions/attest@1e69f48acb82d1966a394da916b4c1698aa569d6" in procedure,
        "procedure does not name the pinned attest action",
    )
    require(
        "https://token.actions.githubusercontent.com" in procedure,
        "procedure does not name the Actions OIDC issuer",
    )
    require("Sigstore Public Good" in procedure, "procedure does not name the public-good instance")
    require(
        "python3 scripts/check-release-attestation.py" in procedure,
        "procedure does not name the attestation checker",
    )
    require(
        "No attestation bundle was produced in this slice." in procedure,
        "procedure claims an attestation bundle",
    )
    require("gh attestation trusted-root" in procedure, "procedure does not name the offline trust root command")
    require("gh attestation verify" in procedure, "procedure does not name the offline verify command")
    require("36342719277" in procedure, "procedure does not record the observed public CI run")
    require(
        "not a current accepted complete release" in procedure,
        "procedure does not keep the old draft out of the accepted release",
    )
    producer_text = texts[SOURCE_PRODUCER]
    for sentence in PRODUCER_HONESTY:
        require(sentence in producer_text, f"source producer is missing honesty text: {sentence}")
    require("--locked" in producer_text, "source producer does not pass --locked")
    require("--offline" in producer_text, "source producer does not pass --offline")
    require("This output is a release SBOM." not in producer_text, "source producer claims a release SBOM")
    require(
        "This document is a local unsigned inventory of the named file bytes." in producer_text,
        "source producer is missing the local unsigned label",
    )
    require(
        "It is not an official attestation." in producer_text,
        "source producer is missing the official-attestation boundary",
    )
    require_package_contract(texts)

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
        "signing": {
            "mechanism": "github-artifact-attestations",
            "sigstore_instance": "public-good",
            "repository": "Rhein-Industries/riAuth",
            "workflow": ".github/workflows/release.yml",
            "action": "actions/attest@1e69f48acb82d1966a394da916b4c1698aa569d6",
            "oidc_issuer": "https://token.actions.githubusercontent.com",
            "present_in_release_path": True,
            "signed_artifact": False,
            "bundle_produced": False,
        },
        "sbom": {
            "spdx_or_cyclonedx_document": False,
            "third_party_notices": "THIRD_PARTY_NOTICES.md",
            "notices_include_spdx_license_expressions": True,
            "notices_are_an_sbom": False,
            "source_producer": SOURCE_PRODUCER,
            "source_producer_in_release_workflow": False,
            "packager_source_calls_producer": True,
            "asset_checks_require_package_spdx": True,
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
        "files": {relative: hashlib.sha256(texts[relative].encode("utf-8")).hexdigest() for relative in AUDITED},
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
