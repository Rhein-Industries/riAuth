#!/usr/bin/env python3
"""Fail closed unless both release architectures have complete, matching assets."""

import hashlib
import importlib.util
import json
import pathlib
import re
import sys
import tarfile


def _package_documents():
    path = pathlib.Path(__file__).resolve().parent / "spdx_sbom.py"
    spec = importlib.util.spec_from_file_location("riauth_spdx_sbom", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


package_documents = _package_documents()


ARCHITECTURES = {"x86_64": "amd64", "aarch64": "arm64"}


def digest(path):
    checksum = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            checksum.update(chunk)
    return checksum.hexdigest()


def require(condition, message):
    if not condition:
        raise ValueError(message)


def check(root, commit, repository, run_id, run_attempt):
    expected = set()
    common = None
    for arch, oci_arch in ARCHITECTURES.items():
        assets = {
            f"riauth-{edition}-linux-{arch}.tar.gz" for edition in ("essentials", "platform")
        } | {
            f"riauth-maintenance-{edition}-linux-{arch}.tar.gz" for edition in ("essentials", "platform")
        } | {
            f"riauth-{edition}-linux-{arch}.docker.tar.gz" for edition in ("essentials", "platform")
        } | {f"riauthctl-linux-{arch}.tar.gz", f"build-provenance-linux-{arch}.json"} | (
            package_documents.linux_spdx_names(arch)
        )
        sums_name = f"SHA256SUMS-linux-{arch}"
        expected.update(assets | {sums_name})
        lines = (root / sums_name).read_text().splitlines()
        entries = {}
        for line in lines:
            match = re.fullmatch(r"([0-9a-f]{64})  ([A-Za-z0-9_.-]+)", line)
            require(match is not None, f"invalid checksum line in {sums_name}: {line!r}")
            checksum, name = match.groups()
            require(name not in entries, f"duplicate checksum entry: {name}")
            entries[name] = checksum
        require(set(entries) == assets, f"{sums_name} has missing or unexpected entries: {set(entries) ^ assets}")
        for name, checksum in entries.items():
            require(digest(root / name) == checksum, f"checksum mismatch: {name}")
        package_documents.require_linux_package_bytes(root, arch)

        provenance = json.loads((root / f"build-provenance-linux-{arch}.json").read_text())
        for field, value in {
            "schema": "riauth.build/v4", "commit": commit, "repository": repository,
            "run_id": run_id, "run_attempt": run_attempt,
            "target_triple": f"{arch}-unknown-linux-gnu", "oci_platform": f"linux/{oci_arch}",
        }.items():
            require(provenance.get(field) == value, f"{arch} provenance {field} mismatch")
        require(provenance.get("build_os", {}).get("name") == "Linux", f"{arch} build OS mismatch")
        require(provenance["build_os"].get("architecture") == arch, f"{arch} build architecture mismatch")
        for edition, features in (("essentials", ["essentials"]), ("platform", ["essentials", "platform"])):
            build = provenance.get("server_builds", {}).get(edition, {})
            require(build.get("features") == features and build.get("no_default_features") is True,
                    f"{arch} {edition} feature mismatch")
            require(str(build.get("docker_image_id", "")).startswith("sha256:"),
                    f"{arch} {edition} image ID missing")
        maintenance_builds = provenance.get("maintenance_builds", {})
        require(isinstance(maintenance_builds, dict) and set(maintenance_builds) == {"essentials", "platform"},
                f"{arch} maintenance builds missing or unexpected")
        for edition, features in (("essentials", ["essentials"]), ("platform", ["essentials", "platform"])):
            build = maintenance_builds[edition]
            require(isinstance(build, dict), f"{arch} {edition} maintenance build is invalid")
            require(build.get("features") == features and build.get("no_default_features") is True,
                    f"{arch} {edition} maintenance feature mismatch")
            binary_digest = build.get("binary_sha256")
            require(isinstance(binary_digest, str) and re.fullmatch(r"[0-9a-f]{64}", binary_digest) is not None,
                    f"{arch} {edition} maintenance binary digest missing")
            archive_name = f"riauth-maintenance-{edition}-linux-{arch}.tar.gz"
            with tarfile.open(root / archive_name, "r:gz") as archive:
                member = archive.getmember("riauth-maintenance")
                require(member.isfile(), f"{archive_name} has no maintenance binary")
                binary = archive.extractfile(member)
                require(binary is not None, f"{archive_name} cannot read maintenance binary")
                with binary:
                    checksum = hashlib.sha256()
                    for chunk in iter(lambda: binary.read(1024 * 1024), b""):
                        checksum.update(chunk)
                require(checksum.hexdigest() == binary_digest,
                        f"{arch} {edition} maintenance binary digest mismatch")
        require(provenance.get("riauthctl_features") == "no-default-features",
                f"{arch} riauthctl feature mismatch")
        shared = tuple(provenance.get(key) for key in
                       ("commit", "rustc", "cargo_lock_sha256", "riauthctl_cargo_lock_sha256"))
        require(all(shared), f"{arch} shared provenance incomplete")
        if common is None:
            common = shared
        else:
            require(shared == common, "architectures differ in revision, toolchain, or lockfiles")
    actual = {path.name for path in root.iterdir() if path.is_file()}
    require(actual == expected, f"release asset set differs: {actual ^ expected}")
    require(not any(path.is_dir() for path in root.iterdir()), "unexpected release directory")


if __name__ == "__main__":
    if len(sys.argv) != 6:
        raise SystemExit("usage: check-release-bundle.py DIST COMMIT REPOSITORY RUN_ID RUN_ATTEMPT")
    try:
        check(pathlib.Path(sys.argv[1]), *sys.argv[2:])
    except (OSError, ValueError, KeyError, tarfile.TarError, json.JSONDecodeError) as error:
        raise SystemExit(f"incomplete release bundle: {error}") from error
    print("Complete x86_64 and aarch64 release bundle verified")
