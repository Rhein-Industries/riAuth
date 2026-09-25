#!/usr/bin/env python3
"""Build the version-locked Linux release dependency notices from Cargo.lock.

The source archive links let recipients obtain the exact source of the
MPL-2.0-covered crates in executable distributions. License texts absent from
some published .crate archives are copied from the same upstream revision in
third-party-license-sources/.
"""

import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys


ROOT = Path(__file__).resolve().parent.parent
SOURCES = ROOT / "scripts" / "third-party-license-sources"
OUTPUT = ROOT / "THIRD_PARTY_NOTICES.md"
TARGET = "x86_64-unknown-linux-gnu"
LICENSE_FILE = re.compile(r"^(?:LICEN[CS]E|COPYING|NOTICE|COPYRIGHT)(?:[._-]|$)", re.I)
TREE_PACKAGE = re.compile(r"^([^ ]+) v([^ ]+)")

OVERRIDE_URLS = {
    "ribergshamra-LICENSE": "https://raw.githubusercontent.com/Rhein-Industries/ribergshamra/33927f40a480d05284fa56749800b157540ee2d7/LICENSE",
    "webauthn-LICENSE.md": "https://raw.githubusercontent.com/kanidm/webauthn-rs/d2c10d53ca5ef033d37ee6462e936e9eb72ad98c/LICENSE.md",
    "ldap3-LICENSE.md": "https://raw.githubusercontent.com/kanidm/ldap3/a41de11d4331bb8977b7bc373aba78bd153674ea/LICENSE.md",
    "asn1-rs-LICENSE-MIT": "https://raw.githubusercontent.com/rusticata/asn1-rs/a20e5f7319c896737ad0f2557037817b91ad854f/LICENSE-MIT",
    "asn1-rs-LICENSE-APACHE": "https://raw.githubusercontent.com/rusticata/asn1-rs/a20e5f7319c896737ad0f2557037817b91ad854f/LICENSE-APACHE",
    "yasna-LICENSE-MIT": "https://raw.githubusercontent.com/qnighy/yasna.rs/b7e65f9a4c317494cce2d18ea02b3d6eaaea7985/LICENSE-MIT",
    "yasna-LICENSE-APACHE": "https://raw.githubusercontent.com/qnighy/yasna.rs/b7e65f9a4c317494cce2d18ea02b3d6eaaea7985/LICENSE-APACHE",
    "Apache-2.0.txt": "https://www.apache.org/licenses/LICENSE-2.0.txt",
}

WEBAUTHN_CRATES = {
    "base64urlsafedata",
    "fido-hid-rs",
    "webauthn-attestation-ca",
    "webauthn-authenticator-rs",
    "webauthn-rs",
    "webauthn-rs-core",
    "webauthn-rs-proto",
}

EXTRA_CRATE_FILES = {
    "aws-lc-sys": ("aws-lc/LICENSE", "aws-lc/third_party/fiat/LICENSE"),
    "regex-syntax": ("src/unicode_tables/LICENSE-UNICODE",),
    "ring": (
        "src/polyfill/once_cell/LICENSE-APACHE",
        "src/polyfill/once_cell/LICENSE-MIT",
        "third_party/fiat/LICENSE",
    ),
    "tracing-core": ("src/spin/LICENSE",),
}


def command(*args):
    return subprocess.check_output(args, cwd=ROOT, text=True)


def linux_dependencies():
    metadata = json.loads(command("cargo", "metadata", "--format-version", "1", "--locked"))
    by_name_version = {}
    for package in metadata["packages"]:
        key = (package["name"], package["version"])
        if key in by_name_version:
            raise ValueError(f"ambiguous package name/version: {key}")
        by_name_version[key] = package

    tree = command(
        "cargo", "tree", "--locked", "--target", TARGET,
        "--edges", "normal", "--prefix", "none", "--format", "{p}",
    )
    keys = set()
    for line in tree.splitlines():
        match = TREE_PACKAGE.match(line)
        if not match:
            raise ValueError(f"unrecognized cargo tree line: {line}")
        keys.add(match.groups())

    packages = []
    for key in sorted(keys):
        package = by_name_version[key]
        if package["id"] == metadata["resolve"]["root"]:
            continue
        if not package["source"].startswith("registry+https://github.com/rust-lang/crates.io-index"):
            raise ValueError(f"unhandled package source: {key}: {package['source']}")
        if not package.get("license"):
            raise ValueError(f"missing license declaration: {key}")
        packages.append(package)
    return packages


def override_files(package):
    name = package["name"]
    if name.startswith("ribergshamra"):
        names = ("ribergshamra-LICENSE",)
    elif name in WEBAUTHN_CRATES:
        names = ("webauthn-LICENSE.md",)
    elif name == "ldap3_proto":
        names = ("ldap3-LICENSE.md",)
    elif name == "asn1-rs-impl":
        names = ("asn1-rs-LICENSE-MIT", "asn1-rs-LICENSE-APACHE")
    elif name == "yasna":
        names = ("yasna-LICENSE-MIT", "yasna-LICENSE-APACHE")
    elif name == "cms":
        # The published crate and matching upstream revision have no LICENSE
        # file. Its README explicitly offers Apache-2.0 or MIT; elect Apache.
        names = ("Apache-2.0.txt",)
    else:
        raise ValueError(f"no license file and no reviewed override: {name} {package['version']}")
    return [(SOURCES / name, OVERRIDE_URLS[name]) for name in names]


def license_material(package):
    directory = Path(package["manifest_path"]).parent
    files = [(p, f"{package['name']}-{package['version']}/{p.name}")
             for p in directory.iterdir() if p.is_file() and LICENSE_FILE.match(p.name)]
    for relative in EXTRA_CRATE_FILES.get(package["name"], ()):
        files.append((directory / relative, f"{package['name']}-{package['version']}/{relative}"))
    if not files:
        files.extend(override_files(package))
    if package["name"] == "cms":
        readme = (directory / "README.md").read_text(encoding="utf-8")
        section = readme.split("## License\n", 1)[1].split("\n## ", 1)[0]
        files.append((section.encode("utf-8"), f"cms-{package['version']}/README.md, License section"))
    for source, label in sorted(files, key=lambda x: x[1]):
        data = source if isinstance(source, bytes) else source.read_bytes()
        if not data or b"\x00" in data:
            raise ValueError(f"invalid license material: {label}")
        yield label, data


def fence(text):
    longest = max((len(m.group()) for m in re.finditer(r"`+", text)), default=0)
    return "`" * max(4, longest + 1)


def render():
    recorded_sources = dict(
        line.split("  ", 1)
        for line in (SOURCES / "SOURCES.txt").read_text(encoding="utf-8").splitlines()[1:]
    )
    if recorded_sources != OVERRIDE_URLS:
        raise ValueError("third-party-license-sources/SOURCES.txt disagrees with the generator")
    packages = linux_dependencies()
    notices = {}
    rows = []
    for package in packages:
        refs = []
        for label, data in license_material(package):
            identifier = "N-" + hashlib.sha256(data).hexdigest()[:16]
            existing = notices.setdefault(identifier, {"data": data, "origins": set()})
            if existing["data"] != data:
                raise ValueError(f"notice hash collision: {identifier}")
            existing["origins"].add(label)
            refs.append(f"{identifier} ({label.rsplit('/', 1)[-1]})")
        name = package["name"]
        version = package["version"]
        source = f"https://static.crates.io/crates/{name}/{name}-{version}.crate"
        rows.append(f"| {name} | {version} | {package['license']} | {source} | {', '.join(refs)} |")

    lock_sha = hashlib.sha256((ROOT / "Cargo.lock").read_bytes()).hexdigest()
    output = [
        "# Third-party notices for riAuth Linux x86_64\n\n",
        "Generated by `scripts/generate-third-party-notices.py` from the locked Cargo normal-dependency graph. "
        "Do not edit this file by hand. It covers the Rust crates and bundled native code used by the "
        "Linux release binary. Development and explicit build-only edges are excluded; "
        "procedural macro packages may still appear through normal dependency edges. The container "
        "also contains Debian packages outside this inventory; inspect their copyright records "
        "under `/usr/share/doc` in the built image.\n\n",
        f"Cargo.lock SHA-256: `{lock_sha}`. Target: `{TARGET}`.\n\n",
        "Each source URL below is a versioned source archive. Recipients of binaries containing "
        "MPL-2.0-covered crates can obtain the corresponding Source Code Form at those URLs; "
        "the covered source remains under MPL-2.0. Notice text IDs below point to exact license "
        "and attribution text copied from the published crate or, when absent there, the "
        "identified primary upstream or licensor source. A crate's SPDX expression describes its available "
        "license options; including more than one option's text does not change that choice.\n\n",
        "## Dependency inventory\n\n",
        "| Crate | Version | SPDX expression | Source archive | License and notice texts |\n",
        "| --- | --- | --- | --- | --- |\n",
        *[row + "\n" for row in rows],
        "\n## License and notice texts\n\n",
    ]
    for identifier, entry in sorted(notices.items()):
        body = entry["data"].decode("utf-8")
        marker = fence(body)
        output.extend((
            f"### {identifier}\n\n",
            "Source files: " + "; ".join(sorted(entry["origins"])) + "\n\n",
            f"{marker}text\n",
            body,
            "" if body.endswith("\n") else "\n",
            f"{marker}\n\n",
        ))
    return "".join(output).encode("utf-8")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="fail when the checked-in notice differs")
    args = parser.parse_args()
    content = render()
    if args.check:
        if not OUTPUT.exists() or OUTPUT.read_bytes() != content:
            raise SystemExit("THIRD_PARTY_NOTICES.md is stale; run scripts/generate-third-party-notices.py")
        print("Third-party notices match Cargo.lock and the Linux dependency graph")
    else:
        OUTPUT.write_bytes(content)
        print(f"Wrote {OUTPUT.relative_to(ROOT)} ({len(content)} bytes)")


if __name__ == "__main__":
    try:
        main()
    except (ValueError, subprocess.CalledProcessError) as exc:
        raise SystemExit(str(exc)) from exc
