"""Isolated source allocation checks; no validator or service is executed."""

import ast
import hashlib
import importlib.util
import os
import pathlib
import re
import stat
import sys
import tempfile
import unittest
from unittest import mock


WORKFLOW = pathlib.Path(__file__).resolve().parent.parent / ".github/workflows/check-local-shared-handoff.yml"
NAMES = (
    "check-exact-edition-matrix.py",
    "check-installed-release-gate.py",
    "check-local-edition-transition-postgres.py",
    "check-local-encrypted-edition-transition.py",
    "spdx_sbom.py",
)


class SealedValidatorSources(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        text = WORKFLOW.read_text()
        start = text.index("          # Proposal only:")
        end = text.index("\n          PY", start)
        source = "\n".join(line[10:] for line in text[start:end].splitlines()) + "\n"
        tree = ast.parse(source)
        names = {"Refusal", "require", "regular", "seal_validator_sources"}
        selected = [node for node in tree.body
                    if isinstance(node, (ast.ClassDef, ast.FunctionDef)) and node.name in names]
        if len(selected) != len(names):
            raise AssertionError("allocation definitions missing or ambiguous")
        namespace = {"hashlib": hashlib, "os": os, "re": re, "stat": stat}
        # Evaluate only the allocation definitions when this focused test is run.
        exec(compile(ast.Module(body=selected, type_ignores=[]), str(WORKFLOW), "exec"), namespace)
        cls.seal = staticmethod(namespace["seal_validator_sources"])
        cls.refusal = namespace["Refusal"]

    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="validator-source-test-")
        self.addCleanup(self.temporary.cleanup)
        self.root = pathlib.Path(self.temporary.name)
        self.source = self.root / "source"
        self.source.mkdir(mode=0o700)
        self.destination = self.root / "validator-scripts"
        self.payloads = {name: f"VALUE = {index}\n".encode("ascii") for index, name in enumerate(NAMES)}
        self.pins = {}
        for name, payload in self.payloads.items():
            (self.source / name).write_bytes(payload)
            self.pins[name] = hashlib.sha256(payload).hexdigest()

    def test_only_verified_sources_are_copied_with_private_modes(self):
        (self.source / "notes.txt").write_text("ordinary sibling")
        (self.source / "unrelated.py").write_text("VALUE = 9\n")
        (self.source / "__pycache__").mkdir()
        self.seal(self.source, self.destination, self.pins)
        self.assertEqual({path.name for path in self.destination.iterdir()}, set(NAMES))
        self.assertEqual(stat.S_IMODE(self.destination.stat().st_mode), 0o700)
        for name in NAMES:
            path = self.destination / name
            self.assertEqual(path.read_bytes(), self.payloads[name])
            self.assertEqual(hashlib.sha256(path.read_bytes()).hexdigest(), self.pins[name])
            self.assertEqual(stat.S_IMODE(path.stat().st_mode), 0o600)
            self.assertEqual(path.stat().st_uid, os.getuid())

    def test_missing_source_refuses(self):
        (self.source / NAMES[0]).unlink()
        with self.assertRaises(FileNotFoundError):
            self.seal(self.source, self.destination, self.pins)

    def test_wrong_retained_buffer_hash_refuses(self):
        (self.source / NAMES[0]).write_bytes(b"VALUE = 9\n")
        with self.assertRaises(self.refusal) as caught:
            self.seal(self.source, self.destination, self.pins)
        self.assertEqual(caught.exception.code, "validator_import_mismatch")
        self.assertFalse((self.destination / NAMES[0]).exists())

    def test_nonregular_sources_refuse(self):
        for kind in ("directory", "symlink", "fifo"):
            with self.subTest(kind=kind):
                path = self.source / NAMES[0]
                path.unlink()
                if kind == "directory":
                    path.mkdir()
                elif kind == "symlink":
                    path.symlink_to(self.source / NAMES[1])
                else:
                    os.mkfifo(path, mode=0o600)
                with self.assertRaises(self.refusal) as caught:
                    self.seal(self.source, self.root / kind, self.pins)
                self.assertEqual(caught.exception.code, "regular_file_required")
                if kind == "directory":
                    path.rmdir()
                else:
                    path.unlink()
                path.write_bytes(self.payloads[NAMES[0]])

    def test_source_size_is_bounded(self):
        (self.source / NAMES[0]).write_bytes(b"x" * (1024 ** 2 + 1))
        with self.assertRaises(self.refusal) as caught:
            self.seal(self.source, self.destination, self.pins)
        self.assertEqual(caught.exception.code, "validator_source_size_limit")

    def test_existing_output_is_never_reused(self):
        self.destination.mkdir(mode=0o700)
        marker = self.destination / "marker"
        marker.write_bytes(b"retained")
        with self.assertRaises(FileExistsError):
            self.seal(self.source, self.destination, self.pins)
        self.assertEqual(marker.read_bytes(), b"retained")
        self.assertEqual({path.name for path in self.destination.iterdir()}, {"marker"})

    def test_manifest_names_and_hashes_are_closed(self):
        for replacement in ("../outside.py", "missing-suffix", "bad-hash"):
            with self.subTest(replacement=replacement):
                pins = dict(self.pins)
                if replacement == "bad-hash":
                    pins[NAMES[0]] = "invalid"
                else:
                    pins[replacement] = pins.pop(NAMES[0])
                with self.assertRaises(self.refusal) as caught:
                    self.seal(self.source, self.destination, pins)
                self.assertEqual(caught.exception.code, "validator_source_manifest")
                self.assertFalse(self.destination.exists())

    def test_exact_copied_synthetic_sources_supply_relative_imports(self):
        helper = NAMES[2]
        payload = (
            "import importlib.util, pathlib\n"
            "RESULT = []\n"
            f"for name in {tuple(name for name in NAMES if name != helper)!r}:\n"
            "    spec = importlib.util.spec_from_file_location('allocation_peer', pathlib.Path(__file__).parent / name)\n"
            "    peer = importlib.util.module_from_spec(spec)\n"
            "    spec.loader.exec_module(peer)\n"
            "    RESULT.append(peer.VALUE)\n"
        ).encode("ascii")
        (self.source / helper).write_bytes(payload)
        self.pins[helper] = hashlib.sha256(payload).hexdigest()
        self.seal(self.source, self.destination, self.pins)
        with mock.patch.object(sys, "dont_write_bytecode", True):
            spec = importlib.util.spec_from_file_location("allocation_entry", self.destination / helper)
            entry = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(entry)
        self.assertEqual(entry.RESULT, [0, 1, 3, 4])
        self.assertEqual({path.name for path in self.destination.iterdir()}, set(NAMES))
        self.assertEqual((self.destination / helper).read_bytes(), payload)


if __name__ == "__main__":
    unittest.main()
