#!/usr/bin/env python3
"""Build or verify an SPDX document from locked Cargo metadata and exact files.

The document is a pure function of the lockfile, the resolved graph, and the
bytes of the named input files. This output is not a release SBOM. This
document is not a build attestation. Crate archives were not fetched.
"""

import argparse
import hashlib
import json
import os
import pathlib
import re
import subprocess
import sys
import tempfile
import tomllib


HONESTY = "This output is not a release SBOM."
NOT_ATTESTATION = "This document is not a build attestation."
NOT_FETCHED = "Crate archives were not fetched."
CREATED = "1970-01-01T00:00:00Z"
CREATED_REASON = "created is 1970-01-01T00:00:00Z because the document is a pure function of its inputs, not a build time."
DOCUMENT_LICENSE = "dataLicense CC0-1.0 applies to this SPDX document only, not to the named packages."
TOOL = "Tool: riauth-spdx-sbom-1"
if CREATED not in CREATED_REASON:
    raise RuntimeError("SPDX created timestamp does not match its reason")
CRATES_IO_SOURCES = {
    "registry+https://github.com/rust-lang/crates.io-index",
    "sparse+https://index.crates.io/",
}
NAME_RE = re.compile(r"[A-Za-z0-9._+-]+")
TARGET_RE = re.compile(r"[A-Za-z0-9._-]+")
FEATURE_RE = re.compile(r"[A-Za-z0-9_-]+")
HEX64 = re.compile(r"[0-9a-f]{64}")
LICENSE_ID = re.compile(r"(?:LicenseRef-[A-Za-z0-9.\-+]+|[A-Za-z0-9][A-Za-z0-9.\-+]*)")


class SpdxError(ValueError):
    """The inputs cannot produce or confirm an SPDX document."""


def canonical_bytes(value):
    return (json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True) + "\n").encode()


def sha256_bytes(payload):
    return hashlib.sha256(payload).hexdigest()


def parse_features(values):
    features = []
    for value in values:
        for part in value.split(","):
            feature = part.strip()
            if not feature or FEATURE_RE.fullmatch(feature) is None:
                raise SpdxError(f"invalid feature name: {value!r}")
            features.append(feature)
    return tuple(sorted(set(features)))


def parse_named_paths(values):
    parsed = []
    seen = set()
    for value in values:
        name, separator, raw_path = value.partition("=")
        if not separator or not raw_path:
            raise SpdxError(f"expected NAME=PATH, got {value!r}")
        if NAME_RE.fullmatch(name) is None:
            raise SpdxError(f"invalid input name: {name!r}")
        if name in seen:
            raise SpdxError(f"duplicate input name: {name}")
        seen.add(name)
        parsed.append((name, pathlib.Path(raw_path)))
    return parsed


def parse_expectations(values):
    expected = {}
    for value in values or []:
        name, separator, digest = value.partition("=")
        if not separator or NAME_RE.fullmatch(name) is None or HEX64.fullmatch(digest) is None:
            raise SpdxError(f"expected NAME=SHA256, got {value!r}")
        if name in expected:
            raise SpdxError(f"duplicate expected checksum: {name}")
        expected[name] = digest
    return expected


def read_input(name, path):
    if path.is_symlink():
        raise SpdxError(f"input is a symlink: {name}")
    if not path.is_file():
        raise SpdxError(f"input is not a regular file: {name}")
    try:
        payload = path.read_bytes()
    except OSError as error:
        raise SpdxError(f"input is unreadable: {name}: {error}") from error
    return {"name": name, "sha256": sha256_bytes(payload), "size": len(payload)}


def load_lock(path):
    try:
        payload = pathlib.Path(path).read_bytes()
        data = tomllib.loads(payload.decode())
    except (OSError, UnicodeError, tomllib.TOMLDecodeError) as error:
        raise SpdxError(f"lockfile is unreadable: {error}") from error
    entries = data.get("package", [])
    if not isinstance(entries, list):
        raise SpdxError("lockfile package list is invalid")
    index = {}
    for package in entries:
        if not isinstance(package, dict):
            raise SpdxError("lock package is not an object")
        name = package.get("name")
        version = package.get("version")
        if not isinstance(name, str) or not name or not isinstance(version, str) or not version:
            raise SpdxError("lock package is missing a name or version")
        source = package.get("source") or "path"
        if not isinstance(source, str) or not source:
            raise SpdxError(f"lock package source is invalid: {name}")
        key = (name, version, source)
        if key in index:
            raise SpdxError(f"duplicate lock package: {name} {version} {source}")
        checksum = package.get("checksum")
        if checksum is not None and (
            not isinstance(checksum, str) or HEX64.fullmatch(checksum) is None
        ):
            raise SpdxError(f"lock checksum is not lowercase sha256: {name}")
        index[key] = checksum
    return payload, index


def license_declared(value):
    if not value or not str(value).strip():
        return "NOASSERTION", ""
    text = " ".join(str(value).strip().split())
    position = 0
    while position < len(text):
        if text[position] == " ":
            position += 1
            continue
        if text[position] in "()":
            position += 1
            continue
        operator = next((
            item for item in ("AND", "OR", "WITH")
            if text.startswith(item, position) and (position + len(item) == len(text) or text[position + len(item)] in " ()")
        ), None)
        if operator is not None:
            position += len(operator)
            continue
        ident = LICENSE_ID.match(text, position)
        if ident is None or ident.start() != position or (
            ident.end() != len(text) and text[ident.end()] not in " ()"
        ):
            return "NOASSERTION", text
        position = ident.end()
    return text, ""


def cargo_metadata_command(manifest, target, features, no_default_features):
    if TARGET_RE.fullmatch(target) is None:
        raise SpdxError(f"invalid target: {target}")
    command = [
        "cargo", "metadata", "--format-version", "1", "--locked", "--offline",
        "--filter-platform", target, "--manifest-path", str(manifest),
    ]
    if no_default_features:
        command.append("--no-default-features")
    if features:
        command.extend(["--features", ",".join(features)])
    return command


def run_cargo_metadata(manifest, target, features, no_default_features):
    if not no_default_features:
        raise SpdxError(
            "pass --no-default-features with --manifest; the root package's default features are not selected"
        )
    manifest = pathlib.Path(manifest)
    if not manifest.is_file():
        raise SpdxError(f"manifest is not a regular file: {manifest}")
    command = cargo_metadata_command(manifest, target, features, no_default_features)
    env = os.environ.copy()
    env["CARGO_INCREMENTAL"] = "0"
    env["CARGO_NET_OFFLINE"] = "true"
    with tempfile.TemporaryDirectory(prefix="riauth-spdx-target-") as target_dir:
        env["CARGO_TARGET_DIR"] = target_dir
        try:
            completed = subprocess.run(
                command, text=True, capture_output=True, env=env, timeout=180,
            )
        except subprocess.TimeoutExpired as error:
            raise SpdxError("cargo metadata timed out") from error
    if completed.returncode != 0:
        detail = [line.strip() for line in completed.stderr.splitlines() if line.strip()]
        message = next((line for line in reversed(detail) if line.startswith("error:")), None)
        if message is None:
            message = detail[-1] if detail else "cargo metadata failed"
        raise SpdxError(f"cargo metadata failed: {message}")
    try:
        return json.loads(completed.stdout)
    except json.JSONDecodeError as error:
        raise SpdxError(f"cargo metadata returned invalid JSON: {error}") from error


def load_metadata(path):
    try:
        metadata = json.loads(pathlib.Path(path).read_text())
    except (OSError, json.JSONDecodeError) as error:
        raise SpdxError(f"metadata is unreadable: {error}") from error
    if not isinstance(metadata, dict) or "resolve" not in metadata or "packages" not in metadata:
        raise SpdxError("metadata has no resolve graph")
    return metadata


def identity_of(package):
    if not isinstance(package, dict):
        raise SpdxError("metadata package is not an object")
    name = package.get("name")
    version = package.get("version")
    if not isinstance(name, str) or not name or not isinstance(version, str) or not version:
        raise SpdxError("metadata package is missing a name or version")
    source = package.get("source") or "path"
    if not isinstance(source, str) or not source:
        raise SpdxError(f"metadata package source is invalid: {name}")
    return {"name": name, "version": version, "source": source}


def identity_label(identity):
    return f"{identity['name']}@{identity['version']}#{identity['source']}"


def normal_edge(dep):
    if not isinstance(dep, dict) or not isinstance(dep.get("pkg"), str) or not dep["pkg"]:
        raise SpdxError("dependency entry is missing a package id")
    kinds = dep.get("dep_kinds")
    if kinds is None:
        return True
    if not isinstance(kinds, list):
        raise SpdxError(f"dependency kind list is invalid: {dep['pkg']}")
    if not kinds:
        raise SpdxError(f"dependency kind list is empty: {dep['pkg']}")
    for item in kinds:
        if not isinstance(item, dict):
            raise SpdxError(f"dependency kind is not an object: {dep['pkg']}")
        if item.get("kind") in (None, "normal"):
            return True
    return False


def resolved_graph(metadata, lock_index):
    resolve = metadata.get("resolve") if isinstance(metadata, dict) else None
    if not isinstance(resolve, dict):
        raise SpdxError("metadata resolve root is missing")
    root_id = resolve.get("root")
    if not isinstance(root_id, str) or not root_id:
        raise SpdxError("metadata resolve root is missing")
    raw_packages = metadata.get("packages")
    raw_nodes = resolve.get("nodes", [])
    if not isinstance(raw_packages, list) or not isinstance(raw_nodes, list):
        raise SpdxError("metadata resolve graph is invalid")
    packages = {}
    for package in raw_packages:
        if not isinstance(package, dict) or not isinstance(package.get("id"), str) or not package["id"]:
            raise SpdxError("metadata package is missing an id")
        if package["id"] in packages:
            raise SpdxError("duplicate metadata package id")
        packages[package["id"]] = package
    nodes = {}
    for node in raw_nodes:
        if not isinstance(node, dict) or not isinstance(node.get("id"), str) or not node["id"]:
            raise SpdxError("metadata resolve node is missing an id")
        if node["id"] in nodes:
            raise SpdxError("duplicate metadata resolve node")
        nodes[node["id"]] = node
    if root_id not in packages:
        raise SpdxError("metadata resolve root is not a package")
    if root_id not in nodes:
        raise SpdxError("metadata resolve root has no node")
    included = []
    edges = []
    seen = set()
    stack = [root_id]
    while stack:
        current = stack.pop()
        if current in seen:
            continue
        if current not in packages or current not in nodes:
            raise SpdxError(f"resolved id is missing from metadata: {current}")
        seen.add(current)
        included.append(packages[current])
        deps = nodes[current].get("deps", [])
        if not isinstance(deps, list):
            raise SpdxError("dependency list is invalid")
        for dep in deps:
            if normal_edge(dep):
                edges.append((current, dep["pkg"]))
                stack.append(dep["pkg"])
    by_id = {}
    crates = []
    for package in included:
        identity = identity_of(package)
        label = identity_label(identity)
        if label in by_id:
            raise SpdxError(f"two packages share one identity: {label}")
        key = (identity["name"], identity["version"], identity["source"])
        if key not in lock_index:
            raise SpdxError(
                f"package is not in the lockfile: {identity['name']} {identity['version']} {identity['source']}"
            )
        lock_checksum = lock_index[key]
        metadata_checksum = package.get("checksum")
        if metadata_checksum is not None and (
            not isinstance(metadata_checksum, str) or metadata_checksum != lock_checksum
        ):
            raise SpdxError(f"metadata checksum does not match the lockfile: {label}")
        if identity["source"] != "path" and lock_checksum is None:
            raise SpdxError(f"non-path package has no sha256 checksum in the lockfile: {label}")
        declared, raw_license = license_declared(package.get("license"))
        by_id[label] = identity
        crates.append({
            "label": label,
            "identity": identity,
            "checksum": lock_checksum or "",
            "license": declared,
            "raw_license": raw_license,
            "edges": [],
        })
    label_by_raw = {}
    for package in included:
        label_by_raw[package["id"]] = identity_label(identity_of(package))
    edge_pairs = []
    for parent, child in edges:
        if child not in label_by_raw:
            raise SpdxError(f"dependency is not in the resolved graph: {child}")
        edge_pairs.append((label_by_raw[parent], label_by_raw[child]))
    children = {}
    for parent, child in edge_pairs:
        children.setdefault(parent, []).append(child)
    for crate in crates:
        crate["edges"] = tuple(sorted(set(children.get(crate["label"], []))))
    crates.sort(key=lambda item: item["label"])
    return label_by_raw[root_id], crates


def digest_payload(root, crates, files, target, features, no_default_features, lock_sha256):
    return {
        "crates": [
            {
                "checksum": crate["checksum"],
                "edges": list(crate["edges"]),
                "label": crate["label"],
                "license": crate["license"],
                "raw_license": crate["raw_license"],
            }
            for crate in crates
        ],
        "features": list(features),
        "files": files,
        "lock_sha256": lock_sha256,
        "no_default_features": no_default_features,
        "root": root,
        "target": target,
    }


def creation_comment(target, features, no_default_features, lock_sha256):
    feature_text = ",".join(features) if features else "none"
    return (
        f"{HONESTY} {NOT_ATTESTATION} {NOT_FETCHED} {DOCUMENT_LICENSE} {CREATED_REASON} "
        f"target={target}; features={feature_text}; no_default_features={str(no_default_features).lower()}; "
        f"lock_sha256={lock_sha256}."
    )


def download_location(source, name, version):
    if source in CRATES_IO_SOURCES:
        return f"https://crates.io/api/v1/crates/{name}/{version}/download"
    return "NOASSERTION"


def build_document(metadata, lock_bytes, lock_index, target, features, no_default_features, files):
    if TARGET_RE.fullmatch(target) is None:
        raise SpdxError(f"invalid target: {target}")
    if not files:
        raise SpdxError("at least one exact input file is required")
    features = tuple(sorted(features))
    root, crates = resolved_graph(metadata, lock_index)
    root_matches = [crate for crate in crates if crate["label"] == root]
    if len(root_matches) != 1:
        raise SpdxError("resolved root is not in the package closure")
    root_version = root_matches[0]["identity"]["version"]
    root_name = root_matches[0]["identity"]["name"]
    files = sorted(files, key=lambda item: item["name"])
    lock_sha256 = sha256_bytes(lock_bytes)
    payload = digest_payload(root, crates, files, target, features, no_default_features, lock_sha256)
    digest = sha256_bytes(canonical_bytes(payload))
    crate_ids = {crate["label"]: f"SPDXRef-Crate-{index:04d}" for index, crate in enumerate(crates, start=1)}
    input_ids = {item["name"]: f"SPDXRef-Input-{index:04d}" for index, item in enumerate(files, start=1)}
    packages = []
    for item in files:
        packages.append({
            "SPDXID": input_ids[item["name"]],
            "name": item["name"],
            "versionInfo": root_version,
            "downloadLocation": "NOASSERTION",
            "filesAnalyzed": False,
            "licenseConcluded": "NOASSERTION",
            "licenseDeclared": "NOASSERTION",
            "copyrightText": "NOASSERTION",
            "checksums": [{"algorithm": "SHA256", "checksumValue": item["sha256"]}],
            "comment": (
                f"checksum is the SHA-256 of the exact input file bytes; bytes={item['size']}. "
                "The caller associated this file with the locked graph. "
                f"{NOT_ATTESTATION} {HONESTY}"
            ),
        })
    for crate in crates:
        identity = crate["identity"]
        comment = (
            f"cargo-name={identity['name']}; cargo-version={identity['version']}; "
            f"cargo-source={identity['source']}"
        )
        if identity["source"] == "path":
            comment += ". Path package; no registry archive was hashed."
        elif identity["source"] in CRATES_IO_SOURCES:
            comment += ". checksum is the Cargo.lock registry checksum. " + NOT_FETCHED
        else:
            comment += ". checksum is the Cargo.lock checksum. " + NOT_FETCHED
        if crate["raw_license"]:
            comment += f" cargo-license={crate['raw_license']}."
        package = {
            "SPDXID": crate_ids[crate["label"]],
            "name": identity["name"],
            "versionInfo": identity["version"],
            "downloadLocation": download_location(identity["source"], identity["name"], identity["version"]),
            "filesAnalyzed": False,
            "licenseConcluded": "NOASSERTION",
            "licenseDeclared": crate["license"],
            "copyrightText": "NOASSERTION",
            "comment": comment,
        }
        if crate["checksum"]:
            package["checksums"] = [{"algorithm": "SHA256", "checksumValue": crate["checksum"]}]
        if identity["source"] != "path":
            package["externalRefs"] = [{
                "referenceCategory": "PACKAGE-MANAGER",
                "referenceType": "purl",
                "referenceLocator": f"pkg:cargo/{identity['name']}@{identity['version']}",
            }]
        packages.append(package)
    relationships = []
    described = [input_ids[item["name"]] for item in files] + [crate_ids[root]]
    for spdx_id in described:
        relationships.append({
            "spdxElementId": "SPDXRef-DOCUMENT",
            "relationshipType": "DESCRIBES",
            "relatedSpdxElement": spdx_id,
        })
    for item in files:
        relationships.append({
            "spdxElementId": input_ids[item["name"]],
            "relationshipType": "OTHER",
            "relatedSpdxElement": crate_ids[root],
            "comment": "Caller associated this exact file with the locked graph.",
        })
    for crate in crates:
        for child in crate["edges"]:
            relationships.append({
                "spdxElementId": crate_ids[crate["label"]],
                "relationshipType": "DEPENDS_ON",
                "relatedSpdxElement": crate_ids[child],
            })
    relationships.sort(key=lambda item: (
        item["relationshipType"], item["spdxElementId"], item["relatedSpdxElement"], item.get("comment", ""),
    ))
    return {
        "spdxVersion": "SPDX-2.3",
        "dataLicense": "CC0-1.0",
        "SPDXID": "SPDXRef-DOCUMENT",
        "name": f"{root_name}-{target}",
        "documentNamespace": f"https://github.com/Rhein-Industries/riAuth/spdx/input/{digest}",
        "creationInfo": {
            "created": CREATED,
            "creators": [TOOL],
            "comment": creation_comment(target, features, no_default_features, lock_sha256),
        },
        "documentDescribes": described,
        "packages": packages,
        "relationships": relationships,
    }


def collect_files(named_paths, expectations):
    files = [read_input(name, path) for name, path in named_paths]
    names = {item["name"] for item in files}
    missing = sorted(set(expectations) - names)
    if missing:
        raise SpdxError("expected checksum names an absent input: " + ", ".join(missing))
    for item in files:
        expected = expectations.get(item["name"])
        if expected is not None and expected != item["sha256"]:
            raise SpdxError(f"input sha256 mismatch: {item['name']}")
    return files


def write_bytes(path, payload):
    path = pathlib.Path(path)
    if path.is_symlink():
        raise SpdxError("output path is a symlink")
    if path.exists() and path.is_dir():
        raise SpdxError("output path is a directory")
    parent = path.parent
    if str(parent) not in ("", ".") and not parent.exists():
        raise SpdxError(f"output directory does not exist: {parent}")
    temporary = path.with_name(path.name + ".tmp")
    try:
        temporary.write_bytes(payload)
        os.replace(temporary, path)
    except OSError as error:
        if temporary.exists() and temporary.is_file() and not temporary.is_symlink():
            temporary.unlink()
        raise SpdxError(f"output was not written: {error}") from error


def assemble(metadata, lock_path, target, features, no_default_features, named_paths, expectations):
    lock_bytes, lock_index = load_lock(lock_path)
    files = collect_files(named_paths, expectations)
    return build_document(
        metadata, lock_bytes, lock_index, target, tuple(features), no_default_features, files,
    )


def metadata_from_args(args):
    if args.metadata:
        return load_metadata(args.metadata)
    return run_cargo_metadata(args.manifest, args.target, parse_features(args.features), args.no_default_features)


def command_args(parser):
    source = parser.add_mutually_exclusive_group(required=True)
    source.add_argument("--metadata", help="cargo metadata JSON; do not also build")
    source.add_argument("--manifest", help="Cargo.toml passed to cargo metadata --locked --offline")
    parser.add_argument("--lock", required=True)
    parser.add_argument("--target", required=True)
    parser.add_argument("--features", action="append", default=[])
    parser.add_argument("--no-default-features", action="store_true")
    parser.add_argument("--file", action="append", required=True, help="NAME=PATH regular file to hash")
    parser.add_argument("--expect", action="append", default=[], help="NAME=SHA256 that the file must match")


def main(argv=None):
    parser = argparse.ArgumentParser(
        description="Build or verify input-bound SPDX JSON. " + HONESTY,
    )
    commands = parser.add_subparsers(dest="command", required=True)
    produce = commands.add_parser("produce")
    command_args(produce)
    produce.add_argument("--out", required=True)
    verify = commands.add_parser("verify")
    command_args(verify)
    verify.add_argument("--document", required=True)
    args = parser.parse_args(argv)
    try:
        features = parse_features(args.features)
        named_paths = parse_named_paths(args.file)
        expectations = parse_expectations(args.expect)
        if args.command == "produce":
            output = pathlib.Path(args.out)
            for _name, path in named_paths:
                if path.resolve(strict=False) == output.resolve(strict=False):
                    raise SpdxError("output path is an input file")
        metadata = metadata_from_args(args)
        document = assemble(
            metadata, args.lock, args.target, features, args.no_default_features, named_paths, expectations,
        )
        payload = canonical_bytes(document)
        if args.command == "verify":
            try:
                actual = pathlib.Path(args.document).read_bytes()
            except OSError as error:
                raise SpdxError(f"document is unreadable: {error}") from error
            if actual != payload:
                raise SpdxError("SPDX document does not match the locked metadata and input files")
            print("SPDX document matches the locked metadata and input files")
            return 0
        write_bytes(args.out, payload)
        print(
            f"wrote {args.out} packages={len(document['packages'])} "
            f"files={len(named_paths)} namespace={document['documentNamespace']}"
        )
        return 0
    except SpdxError as error:
        print(f"spdx sbom failed: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
