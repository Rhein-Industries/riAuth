"""Reproducibility and negative checks for the source SPDX producer."""

import hashlib
import importlib.util
import json
import os
import pathlib
import subprocess
import sys
import tempfile
import unittest
from unittest import mock


ROOT = pathlib.Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/spdx_sbom.py"
spec = importlib.util.spec_from_file_location("spdx_sbom", SCRIPT)
spdx = importlib.util.module_from_spec(spec)
spec.loader.exec_module(spdx)

TARGET = "x86_64-unknown-linux-gnu"
REG = "registry+https://github.com/rust-lang/crates.io-index"
SPARSE = "sparse+https://index.crates.io/"
VERSIONS = {
    "libc": "0.2.1",
    "plain": "3.0.0",
    "shared": "5.0.0",
    "transitive": "1.0.0",
    "devonly": "9.0.0",
    "hidden": "8.0.0",
    "buildonly": "7.0.0",
    "unused": "4.0.0",
    "orphan": "6.0.0",
}
CRATE_MANIFEST = """[package]
name = "spdx-fixture"
version = "0.0.0"
edition = "2021"
license = "MIT OR Apache-2.0"
"""


def digest_char(char):
    return char * 64


def render_lock(packages):
    lines = ["version = 4", ""]
    for name, version, source, checksum in packages:
        lines.append("[[package]]")
        lines.append(f'name = "{name}"')
        lines.append(f'version = "{version}"')
        if source != "path":
            lines.append(f'source = "{source}"')
        if checksum is not None:
            lines.append(f'checksum = "{checksum}"')
        lines.append("")
    return "\n".join(lines).encode()


def package(pid, name, version, source=None, checksum=None, license_text=None):
    item = {"id": pid, "name": name, "version": version, "source": source}
    if checksum is not None:
        item["checksum"] = checksum
    if license_text is not None:
        item["license"] = license_text
    return item


def node(pid, deps):
    return {"id": pid, "deps": deps}


def edge(pkg, kinds=None):
    if kinds is None:
        return {"pkg": pkg}
    return {"pkg": pkg, "dep_kinds": [{"kind": kind} for kind in kinds]}


def closure_fixture(root_id="path+file:///tmp/marker-root#riauth@0.1.1"):
    names = (
        "libc", "plain", "shared", "transitive", "devonly",
        "hidden", "buildonly", "unused", "orphan",
    )
    chars = {
        "libc": "a", "plain": "b", "shared": "c", "transitive": "d",
        "devonly": "e", "hidden": "1", "buildonly": "2", "unused": "3", "orphan": "4",
    }
    checksums = {name: digest_char(chars[name]) for name in names}
    ids = {name: f"{REG}#{name}@{VERSIONS[name]}" for name in names}
    ids["riauth"] = root_id
    packages = [
        package(root_id, "riauth", "0.1.1", license_text="MIT OR Apache-2.0"),
        package(ids["libc"], "libc", VERSIONS["libc"], REG, checksums["libc"], "MIT/Apache-2.0"),
        package(ids["plain"], "plain", VERSIONS["plain"], REG, checksums["plain"], "OREILLY"),
        package(ids["shared"], "shared", VERSIONS["shared"], REG, checksums["shared"], "(MIT OR Apache-2.0)"),
        package(ids["transitive"], "transitive", VERSIONS["transitive"], REG, checksums["transitive"]),
        package(ids["devonly"], "devonly", VERSIONS["devonly"], REG, checksums["devonly"], "MIT"),
        package(ids["hidden"], "hidden", VERSIONS["hidden"], REG, checksums["hidden"], "MIT"),
        package(ids["buildonly"], "buildonly", VERSIONS["buildonly"], REG, checksums["buildonly"], "MIT"),
        package(ids["unused"], "unused", VERSIONS["unused"], REG, checksums["unused"], "MIT"),
        package(ids["orphan"], "orphan", VERSIONS["orphan"], REG, checksums["orphan"], "MIT"),
    ]
    nodes = [
        node(root_id, [
            {"pkg": ids["libc"], "dep_kinds": [{"kind": None}]},
            edge(ids["plain"]),
            edge(ids["shared"], ["dev", "normal"]),
            edge(ids["devonly"], ["dev"]),
            edge(ids["buildonly"], ["build"]),
        ]),
        node(ids["libc"], [edge(ids["transitive"], ["normal"])]),
        node(ids["plain"], []),
        node(ids["shared"], []),
        node(ids["transitive"], []),
        node(ids["devonly"], [edge(ids["hidden"], ["normal"])]),
        node(ids["hidden"], []),
        node(ids["buildonly"], []),
        node(ids["orphan"], []),
    ]
    metadata = {"packages": packages, "resolve": {"root": root_id, "nodes": nodes}}
    lock_packages = [("riauth", "0.1.1", "path", None)]
    lock_packages.extend((name, VERSIONS[name], REG, checksums[name]) for name in names)
    return metadata, lock_packages, ids, checksums


def root_and_dep(source, checksum):
    root_id = "path+file:///tmp/root-marker#app@0.1.0"
    dep_id = f"{source}#libc@0.2.1"
    metadata = {
        "packages": [
            package(root_id, "app", "0.1.0"),
            package(dep_id, "libc", "0.2.1", source, checksum, "MIT"),
        ],
        "resolve": {
            "root": root_id,
            "nodes": [
                node(root_id, [edge(dep_id, ["normal"])]),
                node(dep_id, []),
            ],
        },
    }
    return metadata, [
        ("app", "0.1.0", "path", None),
        ("libc", "0.2.1", source, checksum),
    ]


class SpdxProducer(unittest.TestCase):
    def cli(self, args):
        return subprocess.run(
            [sys.executable, str(SCRIPT), *args],
            capture_output=True,
            text=True,
            timeout=120,
            check=False,
        )

    def assemble(self, directory, metadata, lock_packages, files, **kwargs):
        directory = pathlib.Path(directory)
        directory.mkdir(parents=True, exist_ok=True)
        lock_path = directory / "Cargo.lock"
        payload = kwargs["lock_bytes"] if "lock_bytes" in kwargs else render_lock(lock_packages)
        lock_path.write_bytes(payload)
        named = []
        for name, body in files:
            path = directory / name
            path.write_bytes(body)
            named.append((name, path))
        return spdx.assemble(
            metadata,
            lock_path,
            kwargs.get("target", TARGET),
            kwargs.get("features", ()),
            kwargs.get("no_default_features", False),
            named,
            kwargs.get("expectations", {}),
        )

    def assert_failed(self, completed, message):
        self.assertEqual(completed.returncode, 1, completed.stdout + completed.stderr)
        self.assertIn(message, completed.stderr)
        self.assertNotIn("Traceback", completed.stderr)
        self.assertEqual(completed.stdout, "")

    def assert_honest(self, document, serialized=None):
        comment = document["creationInfo"]["comment"]
        for sentence in (
            spdx.HONESTY,
            spdx.NOT_ATTESTATION,
            spdx.NOT_FETCHED,
            spdx.CREATED_REASON,
            spdx.DOCUMENT_LICENSE,
        ):
            self.assertIn(sentence, comment)
        self.assertEqual(document["spdxVersion"], "SPDX-2.3")
        self.assertEqual(document["dataLicense"], "CC0-1.0")
        self.assertEqual(document["creationInfo"]["created"], spdx.CREATED)
        self.assertEqual(document["creationInfo"]["creators"], [spdx.TOOL])
        self.assertNotIn("release", document["name"])
        serialized = spdx.canonical_bytes(document).decode() if serialized is None else serialized
        self.assertNotIn("This output is a release SBOM.", serialized)
        self.assertIn("This output is not a release SBOM.", serialized)
        self.assertNotIn("GENERATED_FROM", serialized)
        self.assertNotIn("packageVerificationCode", serialized)
        self.assertNotIn('"supplier"', serialized)
        self.assertNotIn("file://", serialized)
        self.assertNotIn("path+file://", serialized)
        self.assertNotRegex(serialized, r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}")
        self.assertRegex(
            document["documentNamespace"],
            r"^https://github.com/Rhein-Industries/riAuth/spdx/input/[0-9a-f]{64}$",
        )
        for item in document["packages"]:
            self.assertIs(item["filesAnalyzed"], False)
            self.assertEqual(item["licenseConcluded"], "NOASSERTION")
            self.assertEqual(item["copyrightText"], "NOASSERTION")
            self.assertNotIn("supplier", item)

    def assert_assemble_fails(self, metadata, lock_packages, message):
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaisesRegex(spdx.SpdxError, message):
                self.assemble(directory, metadata, lock_packages, [("sample.bin", b"sample-bytes")])

    def write_minimal(self, directory, payload=b"sample-bytes"):
        directory = pathlib.Path(directory)
        root_id = "path+file:///tmp/abs-marker#spdx-fixture@0.0.0"
        metadata = {
            "packages": [package(root_id, "spdx-fixture", "0.0.0", license_text="MIT OR Apache-2.0")],
            "resolve": {"root": root_id, "nodes": [node(root_id, [])]},
        }
        (directory / "metadata.json").write_text(json.dumps(metadata), encoding="utf-8")
        (directory / "Cargo.lock").write_bytes(render_lock([
            ("spdx-fixture", "0.0.0", "path", None),
        ]))
        sample = directory / "sample.bin"
        sample.write_bytes(payload)
        return {
            "metadata": directory / "metadata.json",
            "lock": directory / "Cargo.lock",
            "sample": sample,
        }

    def produce_args(self, fixture, out, extra=()):
        return [
            "produce",
            "--metadata", str(fixture["metadata"]),
            "--lock", str(fixture["lock"]),
            "--target", TARGET,
            "--file", f"sample.bin={fixture['sample']}",
            "--out", str(out),
            *extra,
        ]

    def verify_cli(self, fixture, document, extra=()):
        return self.cli([
            "verify",
            "--metadata", str(fixture["metadata"]),
            "--lock", str(fixture["lock"]),
            "--target", TARGET,
            "--file", f"sample.bin={fixture['sample']}",
            "--document", str(document),
            *extra,
        ])

    def test_license_tokenizer_bounds_operators(self):
        self.assertEqual(spdx.license_declared("MIT OR Apache-2.0"), ("MIT OR Apache-2.0", ""))
        self.assertEqual(spdx.license_declared("  MIT   OR   Apache-2.0 "), ("MIT OR Apache-2.0", ""))
        self.assertEqual(spdx.license_declared("OREILLY"), ("OREILLY", ""))
        self.assertEqual(spdx.license_declared("MIT/Apache-2.0"), ("NOASSERTION", "MIT/Apache-2.0"))
        self.assertEqual(spdx.license_declared(""), ("NOASSERTION", ""))
        self.assertEqual(spdx.license_declared(None), ("NOASSERTION", ""))
        self.assertEqual(spdx.license_declared("Apache-2.0 WITH LLVM-exception"), ("Apache-2.0 WITH LLVM-exception", ""))

    def test_names_and_features_are_bounded(self):
        with self.assertRaisesRegex(spdx.SpdxError, "invalid feature"):
            spdx.parse_features(["../x"])
        with self.assertRaisesRegex(spdx.SpdxError, "duplicate input name"):
            spdx.parse_named_paths(["a=/tmp/one", "a=/tmp/two"])
        with self.assertRaisesRegex(spdx.SpdxError, "invalid input name"):
            spdx.parse_named_paths(["a/b=/tmp/one"])
        with self.assertRaisesRegex(spdx.SpdxError, "invalid target"):
            spdx.cargo_metadata_command("Cargo.toml", "../target", (), False)
        self.assertEqual(spdx.parse_features(["b,a", "a"]), ("a", "b"))

    def test_cargo_metadata_command_is_locked_and_offline(self):
        command = spdx.cargo_metadata_command("Cargo.toml", TARGET, ("a", "b"), True)
        self.assertEqual(command[:8], [
            "cargo", "metadata", "--format-version", "1", "--locked", "--offline",
            "--filter-platform", TARGET,
        ])
        self.assertIn("--manifest-path", command)
        self.assertIn("--no-default-features", command)
        self.assertEqual(command[command.index("--features") + 1], "a,b")

    def test_lock_checksum_must_be_lowercase_sha256(self):
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "Cargo.lock"
            path.write_bytes(render_lock([("libc", "0.2.1", REG, "A" * 64)]))
            with self.assertRaisesRegex(spdx.SpdxError, "not lowercase sha256"):
                spdx.load_lock(path)

    def test_requires_an_exact_input_file(self):
        metadata, lock_packages, _, _ = closure_fixture()
        with tempfile.TemporaryDirectory() as directory:
            lock_path = pathlib.Path(directory) / "Cargo.lock"
            lock_path.write_bytes(render_lock(lock_packages))
            _, index = spdx.load_lock(lock_path)
            with self.assertRaisesRegex(spdx.SpdxError, "at least one exact input file"):
                spdx.build_document(
                    metadata, lock_path.read_bytes(), index, TARGET, (), False, [],
                )

    def test_graph_closure_and_license_boundaries(self):
        metadata, lock_packages, _, checksums = closure_fixture()
        with tempfile.TemporaryDirectory() as directory:
            document = self.assemble(
                directory, metadata, lock_packages, [("sample.bin", b"sample-bytes")],
            )
        serialized = spdx.canonical_bytes(document).decode()
        self.assert_honest(document, serialized)
        self.assertEqual(document["name"], f"riauth-{TARGET}")
        self.assertEqual(
            [item["name"] for item in document["packages"]],
            ["sample.bin", "libc", "plain", "riauth", "shared", "transitive"],
        )
        self.assertEqual(
            document["documentDescribes"],
            ["SPDXRef-Input-0001", "SPDXRef-Crate-0003"],
        )
        for name in ("devonly", "hidden", "buildonly", "unused", "orphan", "marker-root"):
            self.assertNotIn(name, serialized)
        by_name = {item["name"]: item for item in document["packages"]}
        sample = by_name["sample.bin"]
        self.assertEqual(sample["versionInfo"], "0.1.1")
        self.assertEqual(sample["downloadLocation"], "NOASSERTION")
        self.assertEqual(sample["licenseDeclared"], "NOASSERTION")
        self.assertEqual(
            sample["checksums"],
            [{"algorithm": "SHA256", "checksumValue": hashlib.sha256(b"sample-bytes").hexdigest()}],
        )
        self.assertIn("bytes=12", sample["comment"])
        self.assertIn("The caller associated this file with the locked graph.", sample["comment"])
        libc = by_name["libc"]
        self.assertEqual(libc["licenseDeclared"], "NOASSERTION")
        self.assertIn("cargo-license=MIT/Apache-2.0.", libc["comment"])
        self.assertIn("checksum is the Cargo.lock registry checksum.", libc["comment"])
        self.assertEqual(
            libc["downloadLocation"],
            "https://crates.io/api/v1/crates/libc/0.2.1/download",
        )
        self.assertEqual(
            libc["checksums"],
            [{"algorithm": "SHA256", "checksumValue": checksums["libc"]}],
        )
        self.assertEqual(libc["externalRefs"], [{
            "referenceCategory": "PACKAGE-MANAGER",
            "referenceType": "purl",
            "referenceLocator": "pkg:cargo/libc@0.2.1",
        }])
        plain = by_name["plain"]
        self.assertEqual(plain["licenseDeclared"], "OREILLY")
        self.assertNotIn("cargo-license=", plain["comment"])
        root = by_name["riauth"]
        self.assertEqual(root["licenseDeclared"], "MIT OR Apache-2.0")
        self.assertNotIn("checksums", root)
        self.assertNotIn("externalRefs", root)
        self.assertIn("Path package; no registry archive was hashed.", root["comment"])
        self.assertNotIn("cargo-license=", root["comment"])
        self.assertEqual(by_name["shared"]["licenseDeclared"], "(MIT OR Apache-2.0)")
        self.assertEqual(by_name["transitive"]["licenseDeclared"], "NOASSERTION")
        self.assertNotIn("cargo-license=", by_name["transitive"]["comment"])
        self.assertIn("features=none", document["creationInfo"]["comment"])
        self.assertIn("no_default_features=false", document["creationInfo"]["comment"])
        self.assertIn(
            "lock_sha256=" + hashlib.sha256(render_lock(lock_packages)).hexdigest(),
            document["creationInfo"]["comment"],
        )
        self.assertEqual(
            [
                (
                    item["relationshipType"],
                    item["spdxElementId"],
                    item["relatedSpdxElement"],
                    item.get("comment", ""),
                )
                for item in document["relationships"]
            ],
            [
                ("DEPENDS_ON", "SPDXRef-Crate-0001", "SPDXRef-Crate-0005", ""),
                ("DEPENDS_ON", "SPDXRef-Crate-0003", "SPDXRef-Crate-0001", ""),
                ("DEPENDS_ON", "SPDXRef-Crate-0003", "SPDXRef-Crate-0002", ""),
                ("DEPENDS_ON", "SPDXRef-Crate-0003", "SPDXRef-Crate-0004", ""),
                ("DESCRIBES", "SPDXRef-DOCUMENT", "SPDXRef-Crate-0003", ""),
                ("DESCRIBES", "SPDXRef-DOCUMENT", "SPDXRef-Input-0001", ""),
                (
                    "OTHER",
                    "SPDXRef-Input-0001",
                    "SPDXRef-Crate-0003",
                    "Caller associated this exact file with the locked graph.",
                ),
            ],
        )

    def test_absolute_ids_and_input_order_are_stable(self):
        with tempfile.TemporaryDirectory() as directory:
            directory = pathlib.Path(directory)
            left_meta, left_lock, _, _ = closure_fixture("path+file:///var/a/proj#riauth@0.1.1")
            right_meta, right_lock, _, _ = closure_fixture("path+file:///private/tmp/b/proj#riauth@0.1.1")
            files = [("beta.bin", b"beta"), ("alpha.bin", b"alpha")]
            left = self.assemble(
                directory / "left", left_meta, left_lock, files, features=("b", "a"),
            )
            right = self.assemble(
                directory / "right",
                right_meta,
                right_lock,
                list(reversed(files)),
                features=("a", "b"),
            )
            self.assertEqual(spdx.canonical_bytes(left), spdx.canonical_bytes(right))
            text = spdx.canonical_bytes(left).decode()
            self.assertNotIn("/var/a/proj", text)
            self.assertNotIn("/private/tmp/b/proj", text)
            self.assertEqual(
                [item["name"] for item in left["packages"] if item["SPDXID"].startswith("SPDXRef-Input")],
                ["alpha.bin", "beta.bin"],
            )
            spaced = json.loads(json.dumps(left_meta))
            for item in spaced["packages"]:
                if item["name"] == "riauth":
                    item["license"] = "MIT   OR   Apache-2.0"
            same = self.assemble(
                directory / "spaced", spaced, left_lock, files, features=("a", "b"),
            )
            self.assertEqual(spdx.canonical_bytes(left), spdx.canonical_bytes(same))
            self.assertTrue(spdx.canonical_bytes(left).startswith(b'{"SPDXID":"SPDXRef-DOCUMENT"'))
            self.assertTrue(spdx.canonical_bytes(left).endswith(b"\n"))

    def test_namespace_changes_when_inputs_change(self):
        metadata, lock_packages, _, _ = closure_fixture()
        files = [("sample.bin", b"sample-bytes")]
        with tempfile.TemporaryDirectory() as directory:
            directory = pathlib.Path(directory)
            base = self.assemble(directory / "base", metadata, lock_packages, files)
            changed_file = self.assemble(
                directory / "file", metadata, lock_packages, [("sample.bin", b"sample-byteS")],
            )
            self.assertNotEqual(base["documentNamespace"], changed_file["documentNamespace"])
            changed_lock = self.assemble(
                directory / "lock",
                metadata,
                lock_packages,
                files,
                lock_bytes=render_lock(lock_packages) + b"\n",
            )
            self.assertNotEqual(base["documentNamespace"], changed_lock["documentNamespace"])
            changed_target = self.assemble(
                directory / "target",
                metadata,
                lock_packages,
                files,
                target="aarch64-unknown-linux-gnu",
            )
            self.assertNotEqual(base["documentNamespace"], changed_target["documentNamespace"])
            self.assertIn("target=aarch64-unknown-linux-gnu", changed_target["creationInfo"]["comment"])
            self.assertIn(spdx.HONESTY, changed_target["creationInfo"]["comment"])
            changed_flag = self.assemble(
                directory / "flag", metadata, lock_packages, files, no_default_features=True,
            )
            self.assertNotEqual(base["documentNamespace"], changed_flag["documentNamespace"])

    def test_expected_checksum_must_match_the_file(self):
        metadata, lock_packages, _, _ = closure_fixture()
        payload = b"sample-bytes"
        good = hashlib.sha256(payload).hexdigest()
        with tempfile.TemporaryDirectory() as directory:
            directory = pathlib.Path(directory)
            self.assemble(
                directory / "ok",
                metadata,
                lock_packages,
                [("sample.bin", payload)],
                expectations={"sample.bin": good},
            )
            with self.assertRaisesRegex(spdx.SpdxError, "input sha256 mismatch"):
                self.assemble(
                    directory / "bad",
                    metadata,
                    lock_packages,
                    [("sample.bin", payload)],
                    expectations={"sample.bin": "a" * 64},
                )
            with self.assertRaisesRegex(spdx.SpdxError, "absent input"):
                self.assemble(
                    directory / "miss",
                    metadata,
                    lock_packages,
                    [("sample.bin", payload)],
                    expectations={"other.bin": good},
                )

    def test_empty_dependency_kind_list_fails(self):
        metadata, lock_packages, ids, _ = closure_fixture()
        metadata["resolve"]["nodes"][0]["deps"].append({"pkg": ids["libc"], "dep_kinds": []})
        self.assert_assemble_fails(metadata, lock_packages, "dependency kind list is empty")

    def test_non_path_package_without_a_lock_checksum_fails(self):
        metadata, lock_packages, _, _ = closure_fixture()
        for item in metadata["packages"]:
            if item["name"] == "libc":
                item.pop("checksum", None)
        lock_packages = [
            (name, version, source, None if name == "libc" else checksum)
            for name, version, source, checksum in lock_packages
        ]
        self.assert_assemble_fails(metadata, lock_packages, "no sha256 checksum")

    def test_package_absent_from_the_lock_fails(self):
        metadata, lock_packages, _, _ = closure_fixture()
        lock_packages = [row for row in lock_packages if row[0] != "libc"]
        self.assert_assemble_fails(metadata, lock_packages, "not in the lockfile")

    def test_metadata_checksum_mismatch_fails(self):
        metadata, lock_packages, _, _ = closure_fixture()
        for item in metadata["packages"]:
            if item["name"] == "libc":
                item["checksum"] = digest_char("f")
        self.assert_assemble_fails(metadata, lock_packages, "does not match the lockfile")

    def test_duplicate_lock_package_fails(self):
        metadata, lock_packages, _, checksums = closure_fixture()
        lock_packages = [*lock_packages, ("libc", "0.2.1", REG, checksums["libc"])]
        self.assert_assemble_fails(metadata, lock_packages, "duplicate lock package")

    def test_duplicate_identity_fails(self):
        first = "path+file:///tmp/one#demo@0.0.0"
        second = "path+file:///tmp/two#demo@0.0.0"
        metadata = {
            "packages": [
                package(first, "demo", "0.0.0"),
                package(second, "demo", "0.0.0"),
            ],
            "resolve": {
                "root": first,
                "nodes": [
                    node(first, [edge(second, ["normal"])]),
                    node(second, []),
                ],
            },
        }
        self.assert_assemble_fails(
            metadata,
            [("demo", "0.0.0", "path", None)],
            "two packages share one identity",
        )

    def test_git_source_without_a_checksum_fails(self):
        metadata, lock_packages = root_and_dep("git+https://example.com/libc.git#abcdef", None)
        self.assert_assemble_fails(metadata, lock_packages, "no sha256 checksum")

    def test_download_location_follows_the_source(self):
        cases = (
            (REG, "https://crates.io/api/v1/crates/libc/0.2.1/download", "registry checksum"),
            (SPARSE, "https://crates.io/api/v1/crates/libc/0.2.1/download", "registry checksum"),
            (
                "git+https://example.com/libc.git#abcdef",
                "NOASSERTION",
                "checksum is the Cargo.lock checksum.",
            ),
        )
        with tempfile.TemporaryDirectory() as directory:
            directory = pathlib.Path(directory)
            for index, (source, location, comment) in enumerate(cases):
                metadata, lock_packages = root_and_dep(source, digest_char("a"))
                metadata["packages"][1].pop("checksum")
                document = self.assemble(
                    directory / str(index),
                    metadata,
                    lock_packages,
                    [("sample.bin", b"sample-bytes")],
                )
                libc = {item["name"]: item for item in document["packages"]}["libc"]
                self.assertEqual(libc["downloadLocation"], location)
                self.assertIn(comment, libc["comment"])
                self.assertIn(spdx.NOT_FETCHED, libc["comment"])
                self.assertEqual(libc["checksums"][0]["checksumValue"], digest_char("a"))
                self.assertEqual(
                    libc["externalRefs"][0]["referenceLocator"],
                    "pkg:cargo/libc@0.2.1",
                )
                if location == "NOASSERTION":
                    self.assertNotIn("registry checksum", libc["comment"])

    def test_metadata_file_whitespace_is_not_hashed(self):
        metadata, lock_packages, _, _ = closure_fixture()
        with tempfile.TemporaryDirectory() as directory:
            directory = pathlib.Path(directory)
            lock = directory / "Cargo.lock"
            lock.write_bytes(render_lock(lock_packages))
            sample = directory / "sample.bin"
            sample.write_bytes(b"sample-bytes")
            compact = directory / "compact.json"
            pretty = directory / "pretty.json"
            compact.write_text(json.dumps(metadata, separators=(",", ":"), sort_keys=True))
            pretty.write_text(json.dumps(metadata, indent=2, sort_keys=True))
            outputs = []
            for meta, name in ((compact, "a.json"), (pretty, "b.json")):
                out = directory / name
                completed = self.cli([
                    "produce",
                    "--metadata", str(meta),
                    "--lock", str(lock),
                    "--target", TARGET,
                    "--file", f"sample.bin={sample}",
                    "--out", str(out),
                ])
                self.assertEqual(completed.returncode, 0, completed.stderr)
                self.assertEqual(completed.stderr, "")
                outputs.append(out.read_bytes())
            self.assertEqual(outputs[0], outputs[1])

    def test_usage_errors_exit_2(self):
        cases = (
            ["--claim-release"],
            ["produce", "--claim-release"],
            ["produce", "--metadata", "m.json", "--lock", "l.lock", "--target", TARGET, "--out", "out.json"],
        )
        for args in cases:
            with self.subTest(args=args):
                completed = self.cli(args)
                self.assertEqual(completed.returncode, 2, completed.stderr)
                self.assertNotIn("Traceback", completed.stderr)
                self.assertNotIn("This output is a release SBOM.", completed.stdout)

    def test_cli_refusals_leave_the_output_in_place(self):
        with tempfile.TemporaryDirectory() as directory:
            directory = pathlib.Path(directory)
            fixture = self.write_minimal(directory)
            out = directory / "out.json"
            sentinel = b"sentinel-not-a-sbom\n"
            out.write_bytes(sentinel)

            def refuse(file_arg, message, tail=()):
                completed = self.cli([
                    "produce",
                    "--metadata", str(fixture["metadata"]),
                    "--lock", str(fixture["lock"]),
                    "--target", TARGET,
                    "--file", file_arg,
                    "--out", str(out),
                    *tail,
                ])
                self.assert_failed(completed, message)
                self.assertEqual(out.read_bytes(), sentinel)
                self.assertFalse((directory / "out.json.tmp").exists())

            refuse(f"sample.bin={directory / 'missing.bin'}", "not a regular file")
            folder = directory / "subdir"
            folder.mkdir()
            refuse(f"bundled={folder}", "not a regular file")
            link = directory / "link.bin"
            link.symlink_to(fixture["sample"])
            refuse(f"link.bin={link}", "input is a symlink")
            refuse(
                f"sample.bin={fixture['sample']}",
                "input sha256 mismatch",
                ("--expect", "sample.bin=" + ("0" * 64)),
            )
            refuse(
                f"sample.bin={fixture['sample']}",
                "expected NAME=SHA256",
                ("--expect", "sample.bin=" + ("A" * 64)),
            )
            refuse(f"a/b={fixture['sample']}", "invalid input name")
            completed = self.cli([
                "produce",
                "--metadata", str(fixture["metadata"]),
                "--lock", str(fixture["lock"]),
                "--target", TARGET,
                "--file", f"sample.bin={fixture['sample']}",
                "--file", f"sample.bin={fixture['sample']}",
                "--out", str(out),
            ])
            self.assert_failed(completed, "duplicate input name")
            self.assertEqual(out.read_bytes(), sentinel)

            before = fixture["sample"].read_bytes()
            collided = self.cli([
                "produce",
                "--metadata", str(fixture["metadata"]),
                "--lock", str(fixture["lock"]),
                "--target", TARGET,
                "--file", f"sample.bin={fixture['sample']}",
                "--out", str(fixture["sample"]),
            ])
            self.assert_failed(collided, "output path is an input file")
            self.assertEqual(fixture["sample"].read_bytes(), before)
            self.assertFalse(pathlib.Path(str(fixture["sample"]) + ".tmp").exists())

            spelled = self.cli([
                "produce",
                "--metadata", "no.json",
                "--lock", "no.lock",
                "--target", TARGET,
                "--file", "sample.bin=./missing.bin",
                "--out", "missing.bin",
            ])
            self.assert_failed(spelled, "output path is an input file")
            self.assertNotIn("not a regular file", spelled.stderr)

            target = directory / "real-out.json"
            target.write_bytes(sentinel)
            linked_out = directory / "linked-out.json"
            linked_out.symlink_to(target)
            linked = self.cli(self.produce_args(fixture, linked_out))
            self.assert_failed(linked, "output path is a symlink")
            self.assertEqual(target.read_bytes(), sentinel)

            missing_dir = self.cli(self.produce_args(fixture, directory / "missing-dir" / "out.json"))
            self.assert_failed(missing_dir, "output directory does not exist")
            self.assertFalse((directory / "missing-dir").exists())

    def test_attacker_tmp_symlink_is_untouched(self):
        with tempfile.TemporaryDirectory() as directory:
            directory = pathlib.Path(directory)
            fixture = self.write_minimal(directory)
            victim = directory / "victim"
            victim_bytes = b"victim-not-the-document\n"
            victim.write_bytes(victim_bytes)
            victim_ino = victim.stat().st_ino
            planted = directory / "out.json.tmp"
            planted.symlink_to(victim)
            planted_target = os.readlink(planted)
            out = directory / "out.json"
            sentinel = b"sentinel-not-a-sbom\n"
            out.write_bytes(sentinel)
            names = sorted(path.name for path in directory.iterdir())

            refused = self.cli(self.produce_args(
                fixture, out, ("--expect", "sample.bin=" + ("0" * 64)),
            ))
            self.assert_failed(refused, "input sha256 mismatch")
            self.assertEqual(out.read_bytes(), sentinel)
            self.assertEqual(victim.read_bytes(), victim_bytes)
            self.assertEqual(victim.stat().st_ino, victim_ino)
            self.assertTrue(planted.is_symlink())
            self.assertEqual(os.readlink(planted), planted_target)
            self.assertEqual(sorted(path.name for path in directory.iterdir()), names)

            produced = self.cli(self.produce_args(fixture, out))
            self.assertEqual(produced.returncode, 0, produced.stderr)
            self.assertEqual(produced.stderr, "")
            self.assertEqual(victim.read_bytes(), victim_bytes)
            self.assertEqual(victim.stat().st_ino, victim_ino)
            self.assertTrue(planted.is_symlink())
            self.assertEqual(os.readlink(planted), planted_target)
            self.assertFalse(out.is_symlink())
            self.assertTrue(out.is_file())
            document = out.read_bytes()
            self.assertIn(b"This output is not a release SBOM.", document)
            self.assertNotIn(b"victim-not-the-document", document)
            self.assertNotEqual(document, sentinel)
            self.assertEqual(sorted(path.name for path in directory.iterdir()), names)

    def test_replace_failure_keeps_output_and_tmp_symlink(self):
        with tempfile.TemporaryDirectory() as directory:
            directory = pathlib.Path(directory)
            victim = directory / "victim"
            victim_bytes = b"victim-not-the-document\n"
            victim.write_bytes(victim_bytes)
            victim_ino = victim.stat().st_ino
            planted = directory / "out.json.tmp"
            planted.symlink_to(victim)
            planted_target = os.readlink(planted)
            out = directory / "out.json"
            sentinel = b"sentinel-not-a-sbom\n"
            out.write_bytes(sentinel)
            names = sorted(path.name for path in directory.iterdir())
            seen = {}

            def refuse_replace(source, destination):
                temporary = pathlib.Path(source)
                seen["name"] = temporary.name
                seen["parent"] = temporary.parent.resolve()
                seen["regular"] = temporary.is_file() and not temporary.is_symlink()
                seen["bytes"] = temporary.read_bytes()
                seen["destination"] = pathlib.Path(destination)
                raise OSError("replace refused")

            with mock.patch.object(spdx.os, "replace", side_effect=refuse_replace):
                with self.assertRaisesRegex(spdx.SpdxError, "output was not written"):
                    spdx.write_bytes(out, b"complete-document")
            self.assertEqual(seen["bytes"], b"complete-document")
            self.assertTrue(seen["regular"])
            self.assertNotEqual(seen["name"], "out.json.tmp")
            self.assertTrue(seen["name"].startswith(".riauth-write-"))
            self.assertTrue(seen["name"].endswith(".tmp"))
            self.assertEqual(seen["parent"], directory.resolve())
            self.assertEqual(seen["destination"], out)
            self.assertFalse((directory / seen["name"]).exists())
            self.assertEqual(out.read_bytes(), sentinel)
            self.assertFalse(out.is_symlink())
            self.assertEqual(victim.read_bytes(), victim_bytes)
            self.assertEqual(victim.stat().st_ino, victim_ino)
            self.assertTrue(planted.is_symlink())
            self.assertEqual(os.readlink(planted), planted_target)
            self.assertEqual(sorted(path.name for path in directory.iterdir()), names)

    def test_verify_accepts_only_the_canonical_bytes(self):
        with tempfile.TemporaryDirectory() as directory:
            directory = pathlib.Path(directory)
            fixture = self.write_minimal(directory)
            out = directory / "out.json"
            other = directory / "other.json"
            features = ["--features", "b", "--features", "a,b"]
            produced = self.cli(self.produce_args(fixture, out, features))
            self.assertEqual(produced.returncode, 0, produced.stderr)
            self.assertEqual(produced.stderr, "")
            self.assertFalse((directory / "out.json.tmp").exists())
            again = self.cli(self.produce_args(
                fixture, other, ["--features", "a", "--features", "b"],
            ))
            self.assertEqual(again.returncode, 0, again.stderr)
            self.assertEqual(out.read_bytes(), other.read_bytes())
            document = json.loads(out.read_text())
            self.assert_honest(document, out.read_text())
            self.assertIn("features=a,b", document["creationInfo"]["comment"])
            self.assertIn(document["documentNamespace"], produced.stdout)
            self.assertIn("packages=2", produced.stdout)
            self.assertIn("files=1", produced.stdout)
            self.assertNotIn(str(directory), out.read_text())
            verified = self.verify_cli(fixture, out, features)
            self.assertEqual(verified.returncode, 0, verified.stderr)
            self.assertEqual(
                verified.stdout.strip(),
                "SPDX document matches the locked metadata and input files",
            )
            original = out.read_bytes()
            digest = hashlib.sha256(fixture["sample"].read_bytes()).hexdigest()
            extra = json.loads(original)
            extra["packages"].append({"SPDXID": "SPDXRef-Crate-9999", "name": "extra"})
            mutations = [
                original.replace(digest.encode(), b"0" * 64, 1),
                original.replace(
                    b"This output is not a release SBOM.",
                    b"This note was removed from the document.",
                ),
                spdx.canonical_bytes(extra),
                (json.dumps(json.loads(original), indent=2) + "\n").encode(),
                original + b"\n",
            ]
            for payload in mutations:
                self.assertNotEqual(payload, original)
                out.write_bytes(payload)
                failed = self.verify_cli(fixture, out, features)
                self.assert_failed(
                    failed,
                    "SPDX document does not match the locked metadata and input files",
                )
            missing = self.verify_cli(fixture, directory / "absent.json", features)
            self.assert_failed(missing, "document is unreadable")
            flagged = directory / "flagged.json"
            completed = self.cli(self.produce_args(fixture, flagged, ["--no-default-features"]))
            self.assertEqual(completed.returncode, 0, completed.stderr)
            self.assertNotEqual(flagged.read_bytes(), original)
            self.assertIn("no_default_features=true", flagged.read_text())

    def cargo(self, args, directory, target_dir):
        env = os.environ.copy()
        env["CARGO_INCREMENTAL"] = "0"
        env["CARGO_NET_OFFLINE"] = "true"
        env["CARGO_TARGET_DIR"] = str(target_dir)
        return subprocess.run(
            ["cargo", *args],
            cwd=directory,
            env=env,
            capture_output=True,
            text=True,
            timeout=120,
            check=False,
        )

    def write_crate(self, directory, extra=""):
        directory = pathlib.Path(directory)
        (directory / "src").mkdir(parents=True, exist_ok=True)
        (directory / "src" / "lib.rs").write_text("")
        (directory / "Cargo.toml").write_text(CRATE_MANIFEST + extra)
        return directory

    def test_manifest_requires_no_default_features(self):
        with tempfile.TemporaryDirectory() as directory:
            directory = pathlib.Path(directory)
            sample = directory / "sample.bin"
            sample.write_bytes(b"exact-bytes")
            out = directory / "out.json"
            sentinel = b"sentinel-not-a-sbom\n"
            out.write_bytes(sentinel)
            completed = self.cli([
                "produce",
                "--manifest", str(directory / "missing-Cargo.toml"),
                "--lock", str(directory / "missing.lock"),
                "--target", TARGET,
                "--file", f"sample.bin={sample}",
                "--out", str(out),
            ])
            self.assert_failed(completed, "pass --no-default-features")
            self.assertNotIn("cargo metadata failed", completed.stderr)
            self.assertEqual(out.read_bytes(), sentinel)
            self.assertFalse((directory / "out.json.tmp").exists())

    def test_z_cargo_locked_roundtrip_is_reproducible(self):
        with tempfile.TemporaryDirectory() as directory:
            directory = pathlib.Path(directory)
            self.write_crate(directory)
            target = directory / "cargo-target"
            target.mkdir()
            generated = self.cargo(
                ["generate-lockfile", "--offline", "--manifest-path", str(directory / "Cargo.toml")],
                directory,
                target,
            )
            self.assertEqual(generated.returncode, 0, generated.stderr)
            sample = directory / "sample.bin"
            sample.write_bytes(b"exact-bytes")
            lock = directory / "Cargo.lock"
            manifest = directory / "Cargo.toml"

            def args(out=None, document=None):
                command = [
                    "produce" if out is not None else "verify",
                    "--manifest", str(manifest),
                    "--lock", str(lock),
                    "--target", TARGET,
                    "--no-default-features",
                    "--file", f"sample.bin={sample}",
                ]
                if out is not None:
                    command.extend(["--out", str(out)])
                else:
                    command.extend(["--document", str(document)])
                return command

            out_a = directory / "a.json"
            out_b = directory / "b.json"
            first = self.cli(args(out=out_a))
            self.assertEqual(first.returncode, 0, first.stderr)
            self.assertEqual(first.stderr, "")
            second = self.cli(args(out=out_b))
            self.assertEqual(second.returncode, 0, second.stderr)
            self.assertEqual(out_a.read_bytes(), out_b.read_bytes())
            self.assertFalse((directory / "a.json.tmp").exists())
            document = json.loads(out_a.read_text())
            self.assert_honest(document, out_a.read_text())
            self.assertNotIn(str(directory), out_a.read_text())
            self.assertEqual(document["name"], f"spdx-fixture-{TARGET}")
            self.assertEqual(
                [item["name"] for item in document["packages"]],
                ["sample.bin", "spdx-fixture"],
            )
            self.assertEqual(document["packages"][0]["versionInfo"], "0.0.0")
            self.assertIn("bytes=11", document["packages"][0]["comment"])
            crate = document["packages"][1]
            self.assertEqual(crate["licenseDeclared"], "MIT OR Apache-2.0")
            self.assertNotIn("checksums", crate)
            self.assertNotIn("externalRefs", crate)
            self.assertIn("Path package; no registry archive was hashed.", crate["comment"])
            self.assertIn(
                "lock_sha256=" + hashlib.sha256(lock.read_bytes()).hexdigest(),
                document["creationInfo"]["comment"],
            )
            self.assertIn("no_default_features=true", document["creationInfo"]["comment"])
            self.assertIn("packages=2", first.stdout)
            self.assertIn("files=1", first.stdout)
            self.assertIn(document["documentNamespace"], first.stdout)
            verified = self.cli(args(document=out_a))
            self.assertEqual(verified.returncode, 0, verified.stderr)
            self.assertEqual(
                verified.stdout.strip(),
                "SPDX document matches the locked metadata and input files",
            )
            sample.write_bytes(b"exact-byteS")
            changed = self.cli(args(document=out_a))
            self.assert_failed(changed, "SPDX document does not match the locked metadata and input files")

    def test_z_cargo_stale_lock_does_not_replace_output(self):
        with tempfile.TemporaryDirectory() as directory:
            directory = pathlib.Path(directory)
            self.write_crate(directory)
            target = directory / "cargo-target"
            target.mkdir()
            generated = self.cargo(
                ["generate-lockfile", "--offline", "--manifest-path", str(directory / "Cargo.toml")],
                directory,
                target,
            )
            self.assertEqual(generated.returncode, 0, generated.stderr)
            lock = directory / "Cargo.lock"
            before = lock.read_bytes()
            self.write_crate(directory, '\n[dependencies]\nserde = "1"\n')
            sample = directory / "sample.bin"
            sample.write_bytes(b"exact-bytes")
            out = directory / "out.json"
            sentinel = b"sentinel-not-a-sbom\n"
            out.write_bytes(sentinel)
            completed = self.cli([
                "produce",
                "--manifest", str(directory / "Cargo.toml"),
                "--lock", str(lock),
                "--target", TARGET,
                "--no-default-features",
                "--file", f"sample.bin={sample}",
                "--out", str(out),
            ])
            self.assert_failed(completed, "cargo metadata failed:")
            self.assertIn("spdx sbom failed:", completed.stderr)
            self.assertEqual(out.read_bytes(), sentinel)
            self.assertFalse((directory / "out.json.tmp").exists())
            self.assertEqual(lock.read_bytes(), before)
            self.assertNotIn("This output is a release SBOM.", completed.stderr)
