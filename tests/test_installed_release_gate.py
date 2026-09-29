"""Small manifest fixtures for the fail-closed Q10 package preflight."""

import importlib.util
import json
import pathlib
import tempfile
import unittest


SCRIPT = pathlib.Path(__file__).resolve().parent.parent / "scripts/check-installed-release-gate.py"
spec = importlib.util.spec_from_file_location("installed_release_gate", SCRIPT)
gate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gate)
COMMIT = "a" * 40
REPOSITORY = "Rhein-Industries/riAuth"
RUN_ID = "123"
RUN_ATTEMPT = "1"


def lone_graph(name):
    root_id = f"path+file:///tmp/q10-gate#{name}@0.1.1"
    return {
        "packages": [{
            "id": root_id,
            "name": name,
            "version": "0.1.1",
            "license": "MIT OR Apache-2.0",
        }],
        "resolve": {"root": root_id, "nodes": [{"id": root_id, "deps": []}]},
    }


def lone_lock(name):
    return f'version = 4\n\n[[package]]\nname = "{name}"\nversion = "0.1.1"\n'


class AssetPreflight(unittest.TestCase):
    def fixture(self, directory):
        base = pathlib.Path(directory)
        root = base / "dist"
        root.mkdir()
        arch = "aarch64"
        names = gate.expected_names(arch)
        provenance_name = f"build-provenance-linux-{arch}.json"
        sums_name = f"SHA256SUMS-linux-{arch}"
        document_names = gate.package_documents.linux_spdx_names(arch)
        provenance = {
            "schema": "riauth.build/v4", "commit": COMMIT,
            "repository": REPOSITORY, "run_id": RUN_ID,
            "run_attempt": RUN_ATTEMPT, "target_triple": "aarch64-unknown-linux-gnu",
            "oci_platform": "linux/arm64", "build_os": {"name": "Linux", "architecture": arch},
            "rustc": "rustc 1.98.1 (fixture)",
            "cargo_lock_sha256": gate.digest(gate.ROOT / "Cargo.lock"),
            "riauthctl_cargo_lock_sha256": gate.digest(gate.ROOT / "crates/riauthctl/Cargo.lock"),
            "riauthctl_features": "no-default-features",
            "server_builds": {}, "maintenance_builds": {},
        }
        for edition, features in (("essentials", ["essentials"]),
                                  ("platform", ["essentials", "platform"])):
            provenance["server_builds"][edition] = {
                "features": features, "no_default_features": True,
                "docker_image_id": "sha256:" + "b" * 64,
            }
            provenance["maintenance_builds"][edition] = {
                "features": features, "no_default_features": True,
                "binary_sha256": "c" * 64,
            }
        for name in names - {sums_name, provenance_name} - document_names:
            (root / name).write_bytes(name.encode())
        inputs = base / "inputs"
        inputs.mkdir()
        server_lock = inputs / "Cargo.lock"
        client_lock = inputs / "riauthctl.lock"
        server_lock.write_text(lone_lock("riauth"))
        client_lock.write_text(lone_lock("riauthctl"))
        gate.package_documents.package_linux(
            root, arch, lone_graph("riauth"), lone_graph("riauth"), server_lock,
            lone_graph("riauthctl"), client_lock,
        )
        (root / provenance_name).write_text(json.dumps(provenance))
        self.write_sums(root, arch)
        return root, provenance

    def write_sums(self, root, arch):
        names = gate.expected_names(arch) - {f"SHA256SUMS-linux-{arch}"}
        sums = "".join(f"{gate.digest(root / name)}  {name}\n" for name in sorted(names))
        (root / f"SHA256SUMS-linux-{arch}").write_text(sums)

    def verify(self, root):
        return gate.verify_assets(root, "aarch64", COMMIT, REPOSITORY, RUN_ID, RUN_ATTEMPT)

    def test_complete_exact_manifest(self):
        with tempfile.TemporaryDirectory() as directory:
            root, provenance = self.fixture(directory)
            self.assertEqual(self.verify(root), provenance)

    def test_missing_asset_fails_before_install(self):
        with tempfile.TemporaryDirectory() as directory:
            root, _ = self.fixture(directory)
            (root / "riauth-platform-linux-aarch64.tar.gz").unlink()
            with self.assertRaisesRegex(ValueError, "assets unavailable"):
                self.verify(root)

    def test_checksum_and_revision_mismatch_fail(self):
        with tempfile.TemporaryDirectory() as directory:
            root, _ = self.fixture(directory)
            with self.assertRaisesRegex(ValueError, "provenance commit mismatch"):
                gate.verify_assets(root, "aarch64", "d" * 40, REPOSITORY, RUN_ID, RUN_ATTEMPT)
            (root / "riauth-platform-linux-aarch64.tar.gz").write_bytes(b"tampered")
            with self.assertRaisesRegex(ValueError, "checksum mismatch"):
                self.verify(root)

    def test_rewritten_checksum_does_not_hide_changed_bytes(self):
        with tempfile.TemporaryDirectory() as directory:
            root, _provenance = self.fixture(directory)
            archive = root / "riauth-platform-linux-aarch64.tar.gz"
            archive.write_bytes(b"tampered-bytes")
            self.write_sums(root, "aarch64")
            with self.assertRaisesRegex(ValueError, "packaged file sha256 mismatch"):
                self.verify(root)

    def test_missing_package_document_fails_the_exact_set(self):
        with tempfile.TemporaryDirectory() as directory:
            root, _provenance = self.fixture(directory)
            (root / "riauth-essentials-linux-aarch64.spdx.json").unlink()
            with self.assertRaisesRegex(ValueError, "assets unavailable"):
                self.verify(root)

    def test_missing_run_identity_fails(self):
        with tempfile.TemporaryDirectory() as directory:
            root, _ = self.fixture(directory)
            with self.assertRaisesRegex(ValueError, "run identity is missing"):
                gate.verify_assets(root, "aarch64", COMMIT, REPOSITORY, "", RUN_ATTEMPT)


if __name__ == "__main__":
    unittest.main()
