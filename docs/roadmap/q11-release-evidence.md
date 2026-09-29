# Q11 vulnerability intake and release evidence

Status: source check only. Q11 remains open. This page does not record a
signature, an SBOM, a separate review record, a GitHub release created by
this work, or a Linux ARM64 run.

The check reads the files named below. It does not build a bundle, download
assets, or call the edition-matrix, installed-release, or bundle checkers.

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

[check-release-bundle.py](../../scripts/check-release-bundle.py) is the command
the publish job runs on the combined artifact directory. It fails closed unless
both architectures' filenames, checksums, and provenance fields match. Its
success line is printed only by that command. This procedure does not run it,
and the workflow file is not that success.

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
generator does not write a CycloneDX SBOM.

The release workflow, the packager, and the bundle checker do not name a
signing or SBOM producer. The denylist checked in those three files is
`cosign`, `syft`, `cyclonedx`, `spdx`, `in-toto`, `gpg`, `minisign`,
`ssh-keygen`, `sigstore`, and the output markers `.sbom`, `.spdx`, `.cdx`,
`.sig`, and `cyclonedx`. The [Q08 note](q08-exact-edition-bundles.md) and the
[Q10 gate](q10-installed-release-gate.md) remain the edition-matrix and
installed-artifact records. This check does not call those scripts and does
not use their results.

## Checker

From the repository root:

```sh
python3 scripts/check-release-evidence.py
python3 -m unittest discover -s tests -p 'test_release_evidence.py' -v
```

Exit 0 means the checkout still matches this contract: the intake sentences,
the single draft release command, provenance schema `riauth.build/v4`, the
attestation disclaimer, the third-party notice output name, and no signing or
SBOM producer on that release path. The JSON report uses schema
`riauth.release-evidence/v1`. It keeps signing, an SBOM, an independent review
record, publication, release execution, and Linux ARM64 execution negative.
Exit 0 is not a release result.

`release_assets.status` is `not_requested` unless `--dist DIR` is set.
`--dist` lists regular filenames in that directory and sets `verification` to
`not performed`. A missing directory is `unavailable`. A filename does not
verify a checksum, and it does not become a signature or an SBOM.

The public check workflow runs this script and these unit tests. That run
still does not build a release bundle.

## Boundary for this slice

The checker was added on top of accepted commit `57d49d3`. No `target/dist`
was present in this worktree when the slice started. The packager was not
executed. Assets were not downloaded. `origin/main` was not updated. The
checker records its own `checker_host`. For this slice that host was Darwin
`arm64`, and `runs_release_packager` was false. A Darwin host field is not
Linux ARM64 release evidence. A later public-check run of the same script is
still not a release job.

Signatures, an SBOM, a separate review record, and a published asset set whose
checksums and provenance were checked for a named tag remain open. Linux ARM64
execution remains evidence for the native release job to produce.
