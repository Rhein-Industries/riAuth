# Q11 vulnerability intake and release evidence

Status: source check, plus a source SPDX producer wired to the Linux
packager's exact archive and image names. Q11 remains open. This
page does not record a signature, a release SBOM, a separate review record,
a GitHub release created by this work, or a Linux ARM64 run. No release SBOM
was produced in this slice.

The check reads the files named below, including the installed release gate.
It does not build a bundle, download assets, or run the edition-matrix,
installed-release, or bundle checkers against release files.

## Intake

Follow [SECURITY.md](../../SECURITY.md). In this checkout that file is the
whole intake policy:

- Security fixes are considered for the latest v0.1.x release.
- When private vulnerability reporting is enabled, use Security → Advisories →
  Report a vulnerability at
  <https://github.com/Rhein-Industries/riAuth/security/advisories>. Include the
  affected version or commit, the impact, and steps to reproduce. Keep exploit
  details, credentials, and personal data out of public issues.
- If that private reporting button is unavailable, open a public issue asking
  maintainers for a private contact method without sharing vulnerability
  details.
- The policy says: "We will acknowledge reports and coordinate a fix and
  disclosure with the reporter."
- No response or remediation deadline is promised for this initial release.

SECURITY.md names no email address and no other contact. This procedure does
not add one. Nothing here records that a vulnerability report was received,
acknowledged, fixed, or disclosed.

A code change that addresses a report, if one is made, is an ordinary commit.
Shipping it still depends on the release workflow below. This page does not
claim that such a commit exists.

## What the release path emits

These statements describe the current source. They are not a result from a
release job.

The [release workflow](../../.github/workflows/release.yml) runs when a `v*`
tag is pushed. Its validate job requires that tag to be the current `main`
commit. The package matrix text names `ubuntu-24.04` for `x86_64` and
`ubuntu-24.04-arm` for `aarch64`. That text is a workflow declaration, not a
completed runner log for this commit.

[package-release.sh](../../scripts/package-release.sh) runs only when `uname`
is Linux and matches `RIAUTH_ARCH` (`x86_64` or `aarch64`). For that
architecture it writes the Essentials and Platform server archives, the
edition-matched maintenance archives, the riauthctl archive, the two container
archives, `build-provenance-linux-$ARCH.json` with schema `riauth.build/v4`,
and `SHA256SUMS-linux-$ARCH`. The provenance object records the commit,
repository, run id, run attempt, target, OS, feature sets, image ids,
maintenance binary digests, rustc, and both Cargo lockfile digests. It has no
signature field.

Before that provenance object and `SHA256SUMS-linux-$ARCH`, the packager runs:

```sh
python3 scripts/spdx_sbom.py package-linux \
  --arch "$RIAUTH_ARCH" \
  --dist "$riauth_dist" \
  --server-manifest Cargo.toml \
  --server-lock Cargo.lock \
  --client-manifest crates/riauthctl/Cargo.toml \
  --client-lock crates/riauthctl/Cargo.lock
```

The command resolves Essentials with feature `essentials`, Platform with
features `essentials,platform`, and riauthctl with no features. Each
resolution passes `--no-default-features`, `--locked`, and `--offline`.
Essentials and Platform share `Cargo.lock`. riauthctl uses
`crates/riauthctl/Cargo.lock`. The documents are
`riauth-essentials-linux-$ARCH.spdx.json` (Essentials server archive,
Essentials maintenance archive, and Essentials container archive),
`riauth-platform-linux-$ARCH.spdx.json` (the Platform counterparts), and
`riauthctl-linux-$ARCH.spdx.json` (the riauthctl archive only). Provenance
is not an input. `SHA256SUMS-linux-$ARCH` is written afterward, so its lines
include the three documents. Generated documents go to the packager dist
directory, `target/dist` in the workflow, and are not source. This output is
not a release SBOM. Quoting the command is not a run of the packager.

[check-release-bundle.py](../../scripts/check-release-bundle.py) is the command
the publish job runs on the combined artifact directory. It fails closed unless
both architectures' filenames, checksums, package-document bytes, and
provenance fields match. A checksum line rewritten to a new archive digest
still fails when the document records the previous digest. A missing document
fails the exact filename set. Its success line is printed only by that
command. This procedure does not run it, and the workflow file is not that
success. The installed gate uses the same filename set and the same byte
comparison for one architecture. This procedure does not run that gate.

The publish job then runs `gh release create` with `--draft` and
`--verify-tag`, using the [release notes](../release-notes.md) as the draft
notes. A draft command in source is not a GitHub release created here. The
release notes say maintainers review the tagged commit, checks, smoke tests,
and draft assets before publication, and that provenance records build
metadata rather than a cryptographic attestation. Those sentences are the
written rule. They are not a review record for any commit, including this one.

[generate-third-party-notices.py](../../scripts/generate-third-party-notices.py)
checks and writes [THIRD_PARTY_NOTICES.md](../../THIRD_PARTY_NOTICES.md) from
the locked dependency graph. Its table has an SPDX expression column for each
crate's license options. That column is not an SPDX document, and the
generator does not write a CycloneDX SBOM. The source producer below is a
separate tool. The release workflow does not call it.

The release workflow file does not name this producer. It also does not name
a signing tool. The packager names the producer once, in the command above.
The bundle checker and the installed gate each load it so their exact asset
sets include the three document names and compare each recorded checksum with
the file bytes. Those two checkers do not run `package-linux`.

The workflow, packager, bundle checker, and installed gate still fail this
audit if they name `cosign`, `syft`, `cyclonedx`, the word `spdx`, `in-toto`,
`gpg`, `minisign`, `ssh-keygen`, or `sigstore`, or if they contain `.sbom`,
`.spdx`, `.cdx`, `.sig`, or `cyclonedx`. The `.spdx.json` filenames stay in
the producer, so those four release files do not contain the `.spdx` marker.
`spdx_sbom.py` does not match the word-boundary denylist. The workflow must
contain no `spdx_sbom.py` reference. The packager must contain the one
command above. Each asset checker must contain one `spdx_sbom.py` reference,
one `linux_spdx_names` call, and one `require_linux_package_bytes` call.

The [Q08 note](q08-exact-edition-bundles.md) `RELEASE_NAMES` list is
unchanged. It reports that older name list as present or unavailable. It is
not the publish exact-set, and it does not list the three documents. The
[Q10 gate](q10-installed-release-gate.md) remains the installed-artifact
record. This check does not run either script against assets and does not
use their results.

## Source SPDX producer

[spdx_sbom.py](../../scripts/spdx_sbom.py) writes SPDX 2.3 JSON from one
locked Cargo graph and the exact files named on the command line.
This output is not a release SBOM.

`--metadata` reads a `cargo metadata --format-version 1` document the caller
already has. `--manifest` is the other source. It runs `cargo metadata` with
`--format-version 1`, `--locked`, `--offline`, `--filter-platform`, and
`--manifest-path`. `--metadata` and `--manifest` cannot be combined. `--manifest` requires
`--no-default-features`. The root package's default features are not
selected. `--features` names any additional features to resolve and record.
The command records the caller's `--target`. It does not re-evaluate `cfg`
expressions. The resolved graph in the metadata is the graph that is
recorded. Crate archives were not fetched. The producer does not extract
archives and does not build riAuth.

Each `--file NAME=PATH` hashes one regular file. `NAME` uses only
`A-Za-z0-9._+-` and is the only file name stored. The producer refuses a
directory, a symlink, a missing path, a duplicate name, an empty file list,
and an output path that is also an input. `--expect NAME=SHA256` refuses a
file whose bytes hash to a different value. The write creates a uniquely
named regular file in the output directory and replaces the destination only
after the document is complete, so a refusal leaves an existing output file
unchanged. A pre-existing `.tmp` symlink beside the destination is not opened.

Path packages stay source `path`. They do not get a registry checksum or a
`pkg:cargo` purl. Any other source needs a lowercase SHA-256 in `Cargo.lock`.
For the crates.io sources
`registry+https://github.com/rust-lang/crates.io-index` and
`sparse+https://index.crates.io/`, `downloadLocation` is the crates.io
download URL and the recorded checksum is the lockfile checksum. Every other
source uses `NOASSERTION` for `downloadLocation`. `licenseConcluded` is
`NOASSERTION`. `licenseDeclared` keeps a Cargo license only when it
tokenizes as SPDX identifiers, `LicenseRef-*`, `AND`, `OR`, `WITH`, and
parentheses. `MIT/Apache-2.0` stays `NOASSERTION`, with the raw string in
the package comment. `filesAnalyzed` is false. `dataLicense` `CC0-1.0`
applies to this SPDX document only, not to the named packages. Creators are
`Tool: riauth-spdx-sbom-1`. `created` is `1970-01-01T00:00:00Z` because the
document is a pure function of its inputs, not a build time.

The document describes each named file and the root crate. Each file has an
`OTHER` relationship to that root because the caller associated the file
with the locked graph. Normal dependencies are `DEPENDS_ON`. Dev-only and
build-only edges are omitted. The producer does not use `GENERATED_FROM`.
Naming a file does not mean the release workflow built it. The absolute
Cargo package id, including a `file://` path, is not stored. Two writes of
the same inputs are byte-identical. `verify` rebuilds the document and
requires those same bytes. It prints `SPDX document matches the locked
metadata and input files` only in that case. A changed checksum, a removed
honesty sentence, or an extra package fails closed.

From the repository root, with metadata and a lockfile the caller chooses:

```sh
python3 scripts/spdx_sbom.py produce \
  --metadata METADATA.json \
  --lock Cargo.lock \
  --target x86_64-unknown-linux-gnu \
  --file sample-bytes=PATH \
  --out OUTPUT.json
python3 scripts/spdx_sbom.py verify \
  --metadata METADATA.json \
  --lock Cargo.lock \
  --target x86_64-unknown-linux-gnu \
  --file sample-bytes=PATH \
  --document OUTPUT.json
```

`PATH` is an exact file. A local sample records that sample. It does not
create a release SBOM.

`package-linux` is the binding the packager uses. It writes no document until
every named archive and image is a regular file, then writes all three or
none. `--server-manifest` and `--client-manifest` run the locked offline
metadata commands. The fixtures pass `--essentials-metadata`,
`--platform-metadata`, and `--client-metadata` instead, so those fixtures do
not run cargo. A second write of the same inputs is byte-identical. Changed
or missing archive bytes fail `require_linux_package_bytes` even when a
checksum file has been rewritten to the new digest. This output is not a
release SBOM.

This slice did not execute the packager. No release-built binary or image
was hashed. No release SBOM was produced in this slice. The producer does
not call the [Q08 note](q08-exact-edition-bundles.md) scripts. Success from
`produce`, `verify`, or `package-linux` is not a release result.

The producer fixtures hash named sample bytes, one no-dependency crate
resolved with `cargo metadata --locked --offline`, and temporary archive
bytes supplied to `package-linux`. They do not run `package-release.sh`.
Run them with:

```sh
python3 -m unittest discover -s tests -p 'test_spdx_sbom.py' -v
```

## Checker

From the repository root:

```sh
python3 scripts/check-release-evidence.py
python3 -m unittest discover -s tests -p 'test_release_evidence.py' -v
```

Exit 0 means the checkout still matches this contract: the intake sentences,
the single draft release command, provenance schema `riauth.build/v4`, the
attestation disclaimer, the third-party notice output name, no signing
producer on that release path, the source SPDX producer named above, one
packager call to that producer, and asset checks that name the package
documents. The JSON report uses schema `riauth.release-evidence/v1`. It keeps
`sbom.spdx_or_cyclonedx_document`, `sbom.notices_are_an_sbom`,
`sbom.source_producer_in_release_workflow`, and `sbom.release_sbom_produced`
false. `sbom.packager_source_calls_producer` and
`sbom.asset_checks_require_package_spdx` are true because this source text
contains that call and those checks. They do not record a packager run or a
release SBOM. Signing, an independent review record, publication, release
execution, and Linux ARM64 execution stay negative.
Exit 0 is not a release result.

`release_assets.status` is `not_requested` unless `--dist DIR` is set.
`--dist` lists regular filenames in that directory and sets `verification` to
`not performed`. A missing directory is `unavailable`. A filename does not
verify a checksum, and it does not become a signature or an SBOM. A name
ending in `.spdx.json` stays a filename. `release_sbom_produced` stays false.

The public check workflow runs this script, these unit tests, and
`test_spdx_sbom.py`. That run still does not build a release bundle.

## Boundary for this slice

The checker was added on top of accepted commit `57d49d3`. No `target/dist`
was present in this worktree when the slice started. The packager was not
executed. Assets were not downloaded. `origin/main` was not updated. The
checker records its own `checker_host`. For that slice the host was Darwin
`arm64`, and `runs_release_packager` was false. A Darwin host field is not
Linux ARM64 release evidence. A later public-check run of the same script is
still not a release job.

The earlier source check on this branch is `063096e`. Accepted history
records that same check as `95d1720`. This worktree continues from `063096e`
and leaves the accepted commit unchanged. The producer was not added to the
release workflow. The Linux packager source calls it through `package-linux`,
and the asset checkers require the resulting document bytes.
This slice did not execute the packager. No release SBOM was produced in this slice.

Signatures, a release SBOM from a real packaged artifact run, a separate
review record, and a published asset set whose checksums and provenance were
checked for a named tag remain open. Linux ARM64 execution remains evidence
for the native release job to produce.
