# A08 local native ARM64 transition run, 2026-09-29

Private local evidence only. Binary source commit:
`f3aba63ac3b824843a40b99623f1619ef8edc19f`. Final validator tip:
`730d373078c4b128373d7afa5f3e278f63b065fd`. The only post-build
changes are in `scripts/`; `src`, Cargo manifests, lockfiles, and the
standalone client source did not change. The worktree is clean. Main was not
changed, merged, or pushed. Recovery refs and all prior Q08 evidence remain.

Set these paths to replay the local checks:

```sh
REPO=/Users/dominik/orca/projects/riAuth-public-preview-roadmap-q08-exact-bundles-wave16
EVIDENCE=$REPO/target/a08-native-linux-arm64-f3aba63-20260929
BUILD_IMAGE=riauth-q08-build:6ca4779-aarch64-local
RUNTIME_IMAGE=riauth-q08-runtime:6ca4779-aarch64-local
PG_IMAGE=riauth-q08-postgres-runtime:6ca4779-aarch64-local
```

The builder image ID is
`sha256:09ca0afee35d39cadc6e33ce89abc4b530bccadaf7ecace2bd586544ffdfed42`.
Its Dockerfile and the runtime Dockerfiles use pinned Rust 1.98.1 Trixie and
Debian Trixie slim base digests. Docker reported Linux `aarch64`. The build
script used `--locked`, exact toolchain, `CARGO_INCREMENTAL=0`, a private
Cargo target, and at most four native build jobs. It copied the previous
cache into a separate target, then compiled the changed server crate.

## Source checks

All commands used `CARGO_TARGET_DIR=target/a08-cross-edition-20260929/source`,
`CARGO_BUILD_JOBS=2`, and `CARGO_INCREMENTAL=0`:

```sh
cargo test --locked --no-default-features --features platform --test edition_transition_preflight -- --test-threads=1
cargo test --locked --no-default-features --features platform --test edition_transition_cross_build -- --ignored --test-threads=1
cargo test --locked --no-default-features --features essentials --test edition_transition_cross_build -- --ignored --test-threads=1
cargo test --locked --no-default-features --features platform --lib node_security::tests -- --test-threads=1
cargo fmt --all -- --check
python3 -m py_compile scripts/check-edition-artifacts.py scripts/check-installed-release-gate.py scripts/check-local-edition-transition-postgres.py
```

Results: preflight 3/3, Platform cross-build 1/1, Essentials cross-build
1/1, node-security 4/4, formatting and Python compilation passed. The
cross-build test checks policy/active-set mismatch, absent/old/future
agreements, full-store/config stale tokens, both switch directions, retained
records, old-build refusal, and Essentials session/login preservation.

## Native build and checks

The exact native Cargo commands and ELF proof are in `container/build.sh`
and `logs/build.log`:

```sh
docker run --rm --platform linux/arm64 --name riauth-a08-f3aba63-builder --label org.riauth.local-evidence=a08-native-linux-arm64-f3aba63-20260929 -v "$REPO:/src:ro" -v "$EVIDENCE:/evidence" -w /src "$BUILD_IMAGE" bash /evidence/container/build.sh > "$EVIDENCE/logs/build.log" 2>&1
```

Result: exit 0; five native ELF64 AArch64 binaries. Build log reported
`rustc 1.98.1`, both exact lockfile hashes, `uname -m=aarch64`, five
`readelf` class/machine checks and five SHA-256 values.

```sh
docker run --rm --platform linux/arm64 --name riauth-a08-f3aba63-installed --label org.riauth.local-evidence=a08-native-linux-arm64-f3aba63-20260929 -v "$REPO:/src:ro" -v "$EVIDENCE:/evidence" -w /src "$RUNTIME_IMAGE" python3 /evidence/container/local-installed-smoke.py > "$EVIDENCE/logs/local-installed-smoke.log" 2>&1
docker run --rm --platform linux/arm64 --name riauth-a08-f3aba63-matrix --label org.riauth.local-evidence=a08-native-linux-arm64-f3aba63-20260929 -v "$REPO:/src:ro" -v "$EVIDENCE:/evidence" -w /src "$RUNTIME_IMAGE" python3 /evidence/container/local-edition-matrix.py > "$EVIDENCE/logs/local-edition-matrix.log" 2>&1
docker run --rm --platform linux/arm64 --name riauth-a08-f3aba63-recovery --label org.riauth.local-evidence=a08-native-linux-arm64-f3aba63-20260929 -v "$REPO:/src:ro" -v "$EVIDENCE:/evidence" -w /src "$RUNTIME_IMAGE" python3 /evidence/container/same-edition-recovery.py > "$EVIDENCE/logs/same-edition-recovery.log" 2>&1
docker run --rm --platform linux/arm64 --name riauth-a08-f3aba63-closure --label org.riauth.local-evidence=a08-native-linux-arm64-f3aba63-20260929 -v "$REPO:/src:ro" -v "$EVIDENCE:/evidence" -w /src "$BUILD_IMAGE" python3 /evidence/container/check-closures.py > "$EVIDENCE/logs/dependency-closure.log" 2>&1
```

All exited 0. Installed drill: Essentials→Platform→Essentials, verified v3
encrypted backup, isolated restore, recovery refusal until completion,
direct downgrade rejection, signing keys unchanged. Edition matrix: 60
Essentials and 87 Platform compiled capabilities; 27 excluded from Essentials;
redb bootstrap/login and incompatible config checks passed. Same-edition
recovery passed for both editions. Locked dependency closure: Essentials 280,
Platform 329, standalone client 106 normal packages.

PostgreSQL used the separate maintenance binary mounted from the native
artifact set. The first run preserved in
`logs/postgres-transition-attempt1-client-drain.log` confirmed the other-client
refusal, then retried before the PostgreSQL backend had drained. The harness
wait was committed in `d33a245`; the product gate was not relaxed. The
second command exited 0:

```sh
docker run --platform linux/arm64 --name riauth-a08-f3aba63-postgres --label org.riauth.local-evidence=a08-native-linux-arm64-f3aba63-20260929 --user postgres -e PATH=/usr/lib/postgresql/17/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin -v "$REPO:/src:ro" -v "$EVIDENCE/artifacts:/artifacts:ro" -w /src "$PG_IMAGE" python3 /src/scripts/check-local-edition-transition-postgres.py --artifacts /artifacts --evidence /tmp/transition.json --source-revision f3aba63ac3b824843a40b99623f1619ef8edc19f > "$EVIDENCE/logs/postgres-transition.log" 2>&1
docker cp riauth-a08-f3aba63-postgres:/tmp/transition.json "$EVIDENCE/local-postgres-transition.json"
docker rm riauth-a08-f3aba63-postgres
```

PostgreSQL 17.11: Essentials→Platform→Essentials, direct wrong-build refusal,
other-client refusal, 22 prior records byte-for-byte preserved at upgrade and
25 at downgrade, and all three server opens/login succeeded.

`730d373` strengthened the PostgreSQL validator to read the stored issuer,
format-2 security agreement, provenance and version activation after each
handoff. The supplemental native rerun passed with those assertions and left
the earlier passing report untouched:

```sh
docker run --platform linux/arm64 --name riauth-a08-f3aba63-postgres-v2 --label org.riauth.local-evidence=a08-native-linux-arm64-f3aba63-20260929 --user postgres -e PATH=/usr/lib/postgresql/17/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin -v "$REPO:/src:ro" -v "$EVIDENCE/artifacts:/artifacts:ro" -w /src "$PG_IMAGE" python3 /src/scripts/check-local-edition-transition-postgres.py --artifacts /artifacts --evidence /tmp/transition-v2.json --source-revision f3aba63ac3b824843a40b99623f1619ef8edc19f > "$EVIDENCE/logs/postgres-transition-v2.log" 2>&1
docker cp riauth-a08-f3aba63-postgres-v2:/tmp/transition-v2.json "$EVIDENCE/local-postgres-transition-v2.json"
docker rm riauth-a08-f3aba63-postgres-v2
```

The v2 report SHA-256 is
`f49a9462d27e952074df79624ec978264501ba9409322bee559d8ba9a99958dd`;
the validator SHA-256 is
`d6b0d023164756b2fcabe1a89794400d5ee70b43b6b4874abed3fd2f6a2cce7d`.
The report confirms issuer and authentication policy unchanged, active sets
switched to the target and back, and edition/version markers coordinated.

Five local archives were assembled by `container/assemble-local-archives.sh`.
The assembly command exited 0:

```sh
docker run --rm --platform linux/arm64 --name riauth-a08-f3aba63-archives --label org.riauth.local-evidence=a08-native-linux-arm64-f3aba63-20260929 -v "$REPO:/src:ro" -v "$EVIDENCE:/evidence" -w /src "$BUILD_IMAGE" bash /evidence/container/assemble-local-archives.sh > "$EVIDENCE/logs/assemble-local-archives.log" 2>&1
```

The native archive validator passed after `d594b25` required both direct
edition refusal and separate retained-agent/provenance preflight evidence.
The earlier provenance-order failure is preserved in
`logs/archive-smoke-attempt1-provenance-order.log`. The final archive command:

```sh
docker run --rm --platform linux/arm64 --name riauth-a08-f3aba63-archive-smoke --label org.riauth.local-evidence=a08-native-linux-arm64-f3aba63-20260929 -v "$REPO:/src:ro" -v "$EVIDENCE:/evidence" -w /src "$RUNTIME_IMAGE" python3 scripts/check-edition-artifacts.py --essentials-archive /evidence/local-archives/local-riauth-essentials-aarch64.tar.gz --platform-archive /evidence/local-archives/local-riauth-platform-aarch64.tar.gz --essentials-maintenance-archive /evidence/local-archives/local-riauth-maintenance-essentials-aarch64.tar.gz --platform-maintenance-archive /evidence/local-archives/local-riauth-maintenance-platform-aarch64.tar.gz --riauthctl-archive /evidence/local-archives/local-riauthctl-aarch64.tar.gz > "$EVIDENCE/logs/archive-smoke.log" 2>&1
```

`container/verify-local-archives.py` passed exact three-member, binary-byte,
license and notice checks for all five archives. Two local images were built
with `container/Dockerfile.local-edition` over the pinned runtime base, then
saved to `local-images/`. `container/local-image-smoke.py` passed image
bootstrap/routes/agents, direct downgrade refusal and separate Platform
maintenance preflight after the local wrapper mounted the maintenance binary.
The earlier image harness failure is preserved in
`logs/local-image-smoke-attempt1-maintenance-boundary.log`. Image archive
gzip and embedded-binary checks passed in `logs/image-archive-integrity.log`.
Only labeled disposable containers and volumes were removed; no global prune.

```sh
docker build --platform linux/arm64 -f "$EVIDENCE/container/Dockerfile.local-edition" --build-arg RIAUTH_EDITION=essentials -t riauth-q08-essentials:f3aba63-aarch64-local "$EVIDENCE/image-context" > "$EVIDENCE/logs/essentials-image.log" 2>&1
docker build --platform linux/arm64 -f "$EVIDENCE/container/Dockerfile.local-edition" --build-arg RIAUTH_EDITION=platform -t riauth-q08-platform:f3aba63-aarch64-local "$EVIDENCE/image-context" > "$EVIDENCE/logs/platform-image.log" 2>&1
python3 "$EVIDENCE/container/local-image-smoke.py" > "$EVIDENCE/logs/local-image-smoke.log" 2>&1
docker save riauth-q08-essentials:f3aba63-aarch64-local | gzip -c > "$EVIDENCE/local-images/local-riauth-essentials-aarch64.docker.tar.gz"
docker save riauth-q08-platform:f3aba63-aarch64-local | gzip -c > "$EVIDENCE/local-images/local-riauth-platform-aarch64.docker.tar.gz"
```

## Evidence and open gates

`native-arm64-evidence.json` is the `d594b25` snapshot. It has full paths,
bytes and SHA-256 for five
binaries, five archives, two image archives, 54 preserved files in total,
all raw logs, local reports and validators present at that snapshot. Manifest SHA-256:
`d401a88c15a5cda1a3fe568c5f65be47f948e5b909ec593e69506e5c6a727926`.
The later `local-postgres-transition-v2.json` and `logs/postgres-transition-v2.log`
are supplemental preserved evidence from validator tip `730d373`.
`native-arm64-evidence-v2.json` is the final manifest for current file hashes;
it includes the supplemental result and this runbook. The earlier manifest is
retained unchanged as a historical snapshot.
The earlier `6ca4779` private evidence tree and recovery refs were not changed.
This is local evidence and does not assert a shipped release.

`GITHUB_RUN_ID`, `GITHUB_RUN_ATTEMPT`, and `GITHUB_REPOSITORY` were unset;
`target/dist` did not exist. Exact official x86_64/aarch64 asset sets,
GitHub-run provenance, package/installed release entrypoints, CI, signatures,
attestations, publication, deployment, previous-version binary migration and
rollback, PostgreSQL restore and deployed recovery remain open. Keep
A08/Q08/Q10 in progress until those independent gates have real evidence.
The final disk observation was above 33 GiB free, exceeding the 8 GiB floor.
