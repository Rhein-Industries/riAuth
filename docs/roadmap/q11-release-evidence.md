# Q11 vulnerability intake and release evidence

Status: source check, a source SPDX producer wired to the Linux packager's
exact archive and image names, and a pinned GitHub artifact-attestation
request on the package job. Q11 remains open. This page does not record a
completed signature, a release SBOM, a separate review record, a GitHub
release created by this work, or a Linux ARM64 release run. No release SBOM
was produced in this slice. No attestation bundle was produced in this slice.

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

The release workflow file does not name this producer. It does not run
`package-linux`. The packager names the producer once, in the command above.
The bundle checker and the installed gate each load it so their exact asset
sets include the three document names and compare each recorded checksum with
the file bytes. Those two checkers do not run `package-linux`.

The packager, the bundle checker, and the installed gate still fail this
audit if they name `cosign`, `syft`, `cyclonedx`, the word `spdx`, `in-toto`,
`gpg`, `minisign`, `ssh-keygen`, or `sigstore`, or if they contain `.sbom`,
`.spdx`, `.cdx`, `.sig`, or `cyclonedx`. The `.spdx.json` filenames stay in
the producer, so those three files do not contain the `.spdx` marker.
`spdx_sbom.py` does not match the word-boundary denylist. The workflow must
contain no `spdx_sbom.py` reference. The packager must contain the one
command above. Each asset checker must contain one `spdx_sbom.py` reference,
one `linux_spdx_names` call, and one `require_linux_package_bytes` call.

The workflow is checked separately. After the three package-document
filenames are removed from that scan, the same producer words and output
markers still fail it. The attestation text it may contain is the pinned
action below and those three filenames. The workflow file does not name
Sigstore. Sigstore Public Good is the visibility rule for this public
repository, recorded here rather than as a string in the workflow.

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

`package-linux` is the binding the packager uses. It validates the named inputs
and prepares all three documents before publishing each file separately.
Preparation failures write no documents; an I/O failure during publication can
leave a partial set and must stop the release job. `--server-manifest` and `--client-manifest` run the locked offline
metadata commands. The fixtures pass `--essentials-metadata`,
`--platform-metadata`, and `--client-metadata` instead, so those fixtures do
not run cargo. A second write of the same inputs is byte-identical. Changed
or missing archive bytes fail `require_linux_package_bytes` even when a
checksum file has been rewritten to the new digest. This output is not a
release SBOM.

`--local-unsigned` is only for a caller who already has the named file bytes
and the same locked graph. It appends two sentences to the creation comment:
"This document is a local unsigned inventory of the named file bytes." and
"It is not an official attestation." Without the flag those sentences are
absent. `verify` accepts the same flag. A labeled document does not match
`verify` when the flag is absent. `package-linux` does not pass the flag. A
local inventory is not a release SBOM, and it is not an official attestation.

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

## Attestation request

The package job in [release.yml](../../.github/workflows/release.yml) requests
GitHub artifact attestations for this public repository. Public repositories
use the Sigstore Public Good instance. The Actions OIDC issuer is
`https://token.actions.githubusercontent.com`. The four steps use
`actions/attest@1e69f48acb82d1966a394da916b4c1698aa569d6`. That commit is tag
`v4.2.2` of `actions/attest`, checked on 2026-09-29: the tag points at that
commit, the commit signature verification reason is `valid`, and the GitHub
release is immutable, not a draft, published `2026-08-04T20:36:29Z`, with
`targetCommitish` `main`. At that commit the action's file-provenance
permission example is `id-token: write`, `contents: read`, and
`attestations: write`.

The package job uses that permission set. The publish job keeps
`contents: write` and does not request `id-token` or `attestations`. The
workflow does not request `packages: write`, `artifact-metadata`, or
`push-to-registry`. It does not set a `github-token` input. The Windows
device-host note is a separate placeholder. It is not this Linux release
trust root, and this page does not invent another trust root.

The provenance step attests every regular file `sha256sum` lists in
`target/dist`. The checksum file is written under the runner temp directory,
so it is not itself a release asset. Three further steps attest the
Essentials, Platform, and riauthctl package documents. Each names only the
files that document lists and passes that document as `sbom-path`. The
provenance step does not pass `sbom-path`. No attestation bundle was produced
in this slice.

`signing.mechanism` is `github-artifact-attestations`.
`signing.sigstore_instance` is `public-good`. `signing.present_in_release_path`
is true because the workflow contains this request. `signing.signed_artifact`
and `signing.bundle_produced` stay false. `provenance.cryptographic_attestation`
stays false because `riauth.build/v4` is still build metadata. The request is
source text. It is not a bundle.

### Offline check

After a release run that is not this slice, verify one subject with a bundle
saved from that run and a trust root fetched with `gh attestation trusted-root`.
The trust root is not stored in this repository. It rotates, so this checkout
does not vendor a copy. The checker command is:

```sh
python3 scripts/check-release-attestation.py verify \
  --artifact PATH \
  --bundle BUNDLE.json \
  --trusted-root TRUSTED_ROOT.jsonl \
  --tag TAG \
  --source SOURCE \
  --run-id RUN_ID \
  --run-attempt RUN_ATTEMPT \
  --predicate https://slsa.dev/provenance/v1
```

`python3 scripts/check-release-attestation.py` builds `gh attestation verify`
for repository `Rhein-Industries/riAuth` and certificate identity
`https://github.com/Rhein-Industries/riAuth/.github/workflows/release.yml@refs/tags/<tag>`.
`gh` 2.96.0 rejects a command that sets both `--cert-identity` and
`--signer-workflow`, so the checker passes only the certificate identity.
That identity contains the workflow path and the tag. The command also passes
issuer `https://token.actions.githubusercontent.com`, source ref
`refs/tags/<tag>`, and the same 40-character source commit for both
`--source-digest` and `--signer-digest`. The signer digest is the riAuth
workflow commit, not the `actions/attest` pin. The command also passes
`--deny-self-hosted-runners`, `--bundle`, `--custom-trusted-root`, and the
predicate type. Provenance uses `https://slsa.dev/provenance/v1`. An SPDX
inventory attestation uses `https://spdx.dev/Document/v2.3`. The command does
not pass `--no-public-good`.

The flags were read from `gh` 2.96.0 on 2026-09-29. The certificate JSON
fields were read from `cli/cli` `v2.96.0`, which parses `sigstore-go` v1.2.1
`VerificationResult.signature.certificate`. After `gh` exits 0, the checker
requires `subjectAlternativeName`, `issuer`, `buildSignerDigest`,
`sourceRepositoryDigest`, `sourceRepositoryRef`, `sourceRepositoryURI`,
`runnerEnvironment`, and `runInvocationURI` to equal the pin, and it requires one witnessed timestamp.
For both provenance and SPDX, the certificate `runInvocationURI` must equal
`https://github.com/Rhein-Industries/riAuth/actions/runs/<run-id>/attempts/<run-attempt>`.
Strings inside the predicate do not satisfy that pin. For provenance, the
predicate `invocationId` must then be
`https://github.com/Rhein-Industries/riAuth/actions/runs/<run-id>/attempts/<run-attempt>`.
That field is workflow-controlled and is read only after the certificate
check. For an SPDX predicate, the document must be SPDX 2.3 and must record
the subject file name and its SHA-256. That predicate is also
workflow-controlled. An unverified bundle parse is not success.

Exit 0 from `verify` means the supplied file and bundle met those pins.
`signed_artifact` in its JSON stays false. This checkout did not produce the
bundle. This command was not run against a riAuth release bundle in this
slice. The 2026-09-30 integration review added the certificate run
and attempt pin for both predicates and strengthened the existing negative
fixtures. These fixtures were not rerun during integration. No actual
release bundle was verified.

### Public observation on 2026-09-29

A read-only GitHub API read on 2026-09-29 recorded these facts.
`Rhein-Industries/riAuth` was public, not archived, and its default branch
was `main`. Draft `v0.1.1` listed only `build-provenance.json`,
`riauth-linux-x86_64.docker.tar.gz`, `riauth-linux-x86_64.tar.gz`, and
`SHA256SUMS`. Public CI run `36342719277` at
`96e23e2dedf84db4e39091a004cbd6591acbec97` concluded success and uploaded
zero artifacts. That draft is not a current accepted complete release.
`python3 scripts/check-release-attestation.py live-public` classifies a fresh
read and fails closed when the response no longer matches this observation.
A match still leaves `signed_artifact` and `release_executed` false. The bare
repository attestations route returning 404 was not treated as a count of
attestations.

## Checker

From the repository root:

```sh
python3 scripts/check-release-evidence.py
python3 scripts/check-release-attestation.py static
python3 -m unittest discover -s tests -p 'test_release_evidence.py' -v
python3 -m unittest discover -s tests -p 'test_release_attestation.py' -v
```

Exit 0 means the checkout still matches this contract: the intake sentences,
the single draft release command, provenance schema `riauth.build/v4`, the
attestation disclaimer, the third-party notice output name, the pinned attest
action above and no other signing producer on that release path, the source
SPDX producer named above, one packager call to that producer, and asset
checks that name the package documents. The JSON report uses schema
`riauth.release-evidence/v1`. It keeps `sbom.spdx_or_cyclonedx_document`,
`sbom.notices_are_an_sbom`, `sbom.source_producer_in_release_workflow`, and
`sbom.release_sbom_produced` false. `sbom.packager_source_calls_producer` and
`sbom.asset_checks_require_package_spdx` are true because this source text
contains that call and those checks. They do not record a packager run or a
release SBOM. `signing.signed_artifact`, `signing.bundle_produced`, an
independent review record, publication, release execution, and Linux ARM64
execution stay negative. `signing.present_in_release_path` is true.
Exit 0 is not a release result.

`release_assets.status` is `not_requested` unless `--dist DIR` is set.
`--dist` lists regular filenames in that directory and sets `verification` to
`not performed`. A missing directory is `unavailable`. A filename does not
verify a checksum, and it does not become a signature or an SBOM. A name
ending in `.spdx.json` stays a filename. `release_sbom_produced` stays false.

The public check workflow runs this script, `check-release-attestation.py static`,
these unit tests, `test_release_attestation.py`, and `test_spdx_sbom.py`.
That run still does not build a release bundle.

## Local unsigned inventory

On 2026-09-29 this checkout's `Cargo.lock` and `crates/riauthctl/Cargo.lock`
were byte-identical to `f3aba63ac3b824843a40b99623f1619ef8edc19f`. Their
SHA-256 values are `b5c9d11c001244b8017303ce8c20516e02946483845eee40720b0910758d4426`
and `6f546c966c70505b8ef204cea811f7300b1b359de63f01a8389caadc3164b964`.
`Cargo.toml` and `crates/riauthctl/Cargo.toml` had no diff from that commit.
`python3 scripts/spdx_sbom.py produce --local-unsigned` then hashed preserved
regular files from the Q08 directory
`a08-native-linux-arm64-f3aba63-20260929`. The command used
`--locked --offline --no-default-features`, `CARGO_INCREMENTAL=0`,
`CARGO_BUILD_JOBS=1`, and the producer's private temporary target. It did not
rebuild or repackage those files, and it did not rename them to release asset
names. The output is `target/q11-local-unsigned-f3aba63/`, which is not source.
Available space on `/System/Volumes/Data` was 26477700 KiB before the command
and 26476936 KiB after it.

| Document | SHA-256 | Packages |
| --- | --- | --- |
| `local-riauth-essentials-aarch64.spdx.json` | `c64455270ca2a928ff1dde73a7586152c667bcf8c2626c4652aa0b9fd1d8b289` | 301 |
| `local-riauth-platform-aarch64.spdx.json` | `9e1520c898251ba0bf116f1ddc3994c9f7e6ce8ff21fb3888ab5967b0ee184a9` | 353 |
| `local-riauthctl-aarch64.spdx.json` | `23a62085c911d5e84d532a2e17aeabe04a562f25d234d18e4c7d0dfaa6700e35` | 121 |

Essentials records feature `essentials`, the server lock, and three files:
`local-riauth-essentials-aarch64.tar.gz`
`b330e3e8a66f697df297b962042220dbdb0234aeb251f17a4293c35d2f3835ef`,
`local-riauth-maintenance-essentials-aarch64.tar.gz`
`58f144ac3f9470e4011e65e65333d876d4f9afce726743409c8f0755a590a83a`, and
`local-riauth-essentials-aarch64.docker.tar.gz`
`5f4347c9748a6b9b92bdf3f426282a082ab9aacf694514de43abdb2759b8ca15`.
Its 301 packages are 297 registry packages, the path root, and those three
files. Platform records features `essentials,platform`, the same server lock,
and `local-riauth-platform-aarch64.tar.gz`
`441e6ef7bf9a99c439441c1c440ec10045bc877182f925109555dd08d0eac0f5`,
`local-riauth-maintenance-platform-aarch64.tar.gz`
`df6d7e61b4648cc666b1421024cbf3357e17342ac4d1db02b458cc25bffb8292`, and
`local-riauth-platform-aarch64.docker.tar.gz`
`8ef3774017201248cd2cc1aa0d69b992d6e6990e8dc6f0981caa4de8d03a7604`.
Its 353 packages are 349 registry packages, the path root, and those three
files. riauthctl records no features, the client lock, and
`local-riauthctl-aarch64.tar.gz`
`562bf354e97b0f119785d40d54d479eb139a367ba12312424619d505a445ca8b`.
Its 121 packages are 119 registry packages, the path root, and that one file.
Each document is SPDX 2.3. `verify --local-unsigned` rebuilt each document and
printed `SPDX document matches the locked metadata and input files`.

The preserved Q08 closure file records different `normal_packages` counts,
280, 329, and 106, and tree hashes this producer did not compute. That file
has no per-crate license or checksum, so it was not the graph source. These
documents do not claim those counts or hashes. Each creation comment includes
the local unsigned inventory sentence and "It is not an official attestation."
This output is not a release SBOM. No release SBOM was produced in this slice.
The preserved files are not Linux ARM64 release execution.

## Boundary for this slice

The checker was added on top of accepted commit `57d49d3`. No `target/dist`
was present in this worktree when the slice started. The packager was not
executed. Assets were not downloaded. `origin/main` was not updated. The
checker records its own `checker_host`. For that slice the host was Darwin
`arm64`, and `runs_release_packager` was false. A Darwin host field is not
Linux ARM64 release evidence. A later public-check run of the same script is
still not a release job.

The earlier source check on this branch is `063096e`. Accepted history
records that same check as `95d1720`. The branch
`roadmap/q11-release-gate-wave21` is aligned to accepted commit
`eabb48b70dc38647acf9368429a0ebdbef51ddcc`. The previous tip
`1c507e25f59fcb7a4ad400cf9dd63b74d23a0960` remains at the local ref
`refs/recovery/q11-before-accepted-eabb48b-1c507e2`. That ref was not pushed.
Accepted history and `main` were not edited by this alignment.

The producer was not added to the release workflow. The Linux packager source
calls it through `package-linux`, and the asset checkers require the resulting
document bytes. The release workflow now contains the pinned attestation
request above.
This slice did not execute the packager. No release SBOM was produced in this slice.
No attestation bundle was produced in this slice.

A checked-tag run, the bundles from that run, offline verification of those
bundles, a release SBOM from that run, a separate review record, and a
published asset set whose checksums and provenance were checked for a named
tag remain open. Linux ARM64 release execution remains evidence for the
native release job to produce. A preserved local ARM64 archive is not that
execution.
