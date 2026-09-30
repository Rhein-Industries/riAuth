# A08/Q08/Q10 local encrypted-storage supplement, 2026-09-29

This is local native Linux ARM64 evidence, not a shipped release. Installed
server and maintenance binaries are the unchanged `f3aba63ac3b824843a40b99623f1619ef8edc19f`
copies in `../artifacts/`. The test-only Store API probe was compiled from
`tests/edition_transition_store_probe.rs`; it never replaced those binaries.
The encrypted transition validator is committed at
`9f8efaf0372e02b0bc0042251d591f1facc0f783`, and the installed
PostgreSQL logical restore adaptation at
`082563b362acb218f276e4dbcaf476e303b561b7`.

Set `REPO=/Users/dominik/orca/projects/riAuth-public-preview-roadmap-q08-exact-bundles-wave16`
and `EVIDENCE=$REPO/target/a08-native-linux-arm64-f3aba63-20260929`.
The builder image ID is
`sha256:09ca0afee35d39cadc6e33ce89abc4b530bccadaf7ecace2bd586544ffdfed42`;
the runtime and PostgreSQL image IDs are respectively
`sha256:91724daedf800ad357defe83f6c8a299d50b718968d53b6274ca9281611aeacc`
and `sha256:0d8ccb03408814693c5a18cd550788084be549c24b78a9acc748e1a77fa4b8c3`.
The Dockerfiles and pinned parent digests are in the preceding immutable
`../native-arm64-evidence-v2.json` manifest. Docker reported native
Linux `aarch64`; `file` identified the probe and five product binaries as
ELF64 AArch64. Rust 1.98.1, exact Cargo.lock, `--locked`, `-j 4`, and
`CARGO_INCREMENTAL=0` were used. Host source checks used `-j 2` in the
private `target/a08-cross-edition-20260929/source` target. Available disk
stayed above 25 GiB, exceeding the 8 GiB floor.

The final source command was:

```sh
cd "$REPO"
CARGO_TARGET_DIR=target/a08-cross-edition-20260929/source CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 sh tests/edition_transition_cross_build.sh > "$EVIDENCE/logs/encrypted-storage/source-cross-build-final.log" 2>&1
```

It passed four ignored integration executions: Platform and Essentials on
plaintext redb, then Platform and Essentials on encrypted redb. Each retained
two real users, a product API auditor grant, credential records and a revoked
session. Native probe compilation used:

```sh
docker run --rm --platform linux/arm64 --name riauth-a08-encrypted-probe-build --label org.riauth.local-evidence=a08-encrypted-f3aba63 --env CARGO_HOME=/evidence/cargo-home --env CARGO_TARGET_DIR=/evidence/cargo-target/server --env CARGO_INCREMENTAL=0 --env CARGO_BUILD_JOBS=4 -v "$REPO:/src:ro" -v "$EVIDENCE:/evidence" -w /src riauth-q08-build:6ca4779-aarch64-local bash -c 'test "$(uname -m)" = aarch64 && test "$(df -Pk /evidence | awk "NR==2 {print \$4}")" -ge 8388608 && rustc -Vv && cargo test --release --locked --no-default-features --features platform --test edition_transition_store_probe --no-run --target aarch64-unknown-linux-gnu -j 4' > "$EVIDENCE/logs/encrypted-storage/probe-build-attempt2.log" 2>&1
```

It exited 0. The test binary was copied to
`encrypted-storage/edition-transition-store-probe`; SHA-256
`58b1aac1fa628a8f038e7029aedb406357d8e85dce95e0580f4d895e6a03b3f4`.
The first build attempt used a login shell that removed Rust from PATH; its
failure log remains at `logs/encrypted-storage/probe-build.log`.

The final installed encrypted transition commands were:

```sh
docker run --platform linux/arm64 --name riauth-a08-encrypted-redb-v5 --label org.riauth.local-evidence=a08-encrypted-f3aba63 -v "$REPO:/src:ro" -v "$EVIDENCE/artifacts:/artifacts:ro" -v "$EVIDENCE/encrypted-storage:/probe:ro" -w /src riauth-q08-runtime:6ca4779-aarch64-local python3 /src/scripts/check-local-encrypted-edition-transition.py --backend redb --artifacts /artifacts --probe /probe/edition-transition-store-probe --evidence /tmp/encrypted-redb-v4.json --source-revision f3aba63ac3b824843a40b99623f1619ef8edc19f > "$EVIDENCE/logs/encrypted-storage/redb-v4.log" 2>&1
docker cp riauth-a08-encrypted-redb-v5:/tmp/encrypted-redb-v4.json "$EVIDENCE/encrypted-storage/local-encrypted-redb-transition-v4.json"
docker rm riauth-a08-encrypted-redb-v5
docker run --platform linux/arm64 --name riauth-a08-encrypted-postgres-v4 --label org.riauth.local-evidence=a08-encrypted-f3aba63 --user postgres -e PATH=/usr/lib/postgresql/17/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin -v "$REPO:/src:ro" -v "$EVIDENCE/artifacts:/artifacts:ro" -v "$EVIDENCE/encrypted-storage:/probe:ro" -w /src riauth-q08-postgres-runtime:6ca4779-aarch64-local python3 /src/scripts/check-local-encrypted-edition-transition.py --backend postgresql --artifacts /artifacts --probe /probe/edition-transition-store-probe --evidence /tmp/encrypted-postgres-v4.json --source-revision f3aba63ac3b824843a40b99623f1619ef8edc19f > "$EVIDENCE/logs/encrypted-storage/postgres-v4.log" 2>&1
docker cp riauth-a08-encrypted-postgres-v4:/tmp/encrypted-postgres-v4.json "$EVIDENCE/encrypted-storage/local-encrypted-postgres-transition-v4.json"
docker rm riauth-a08-encrypted-postgres-v4
```

Both exited 0. Redb preserved 30 logical rows on Essentials to Platform
and 33 on Platform to Essentials. PostgreSQL preserved 36 then 39 logical
rows, plus 36 raw ciphertext rows at upgrade and 39 at downgrade (the final
report records the upgrade count; the code checks both). Both used a real
non-admin user and active auditor grant created through installed CLI/HTTP,
plus revoked sessions, and checked their decrypted Store API hashes. The
encrypted PostgreSQL `storage_format` was read as `aes256gcm-v1`; SQL
metadata values were verified as ciphertext, while logical metadata was
decoded only through the Store API. Redb's encrypted format was checked by
the product's missing-key format refusal and wrong-key authentication
refusal. Both directions checked issuer and authentication policy unchanged,
target active capabilities with edition/version/provenance/revision,
no automatic agreement adoption, wrong-build direct-open refusal, a stale
whole-store plan, a compatible config token change, authentication-policy
drift, and absent/old/new/shared-capability agreement drift. A live redb
writer and a connected PostgreSQL client each blocked activation.

Earlier local attempts and their immutable logs and reports are retained
in this supplement directory. The first redb attempt failed a validator
comparison against an absent initial revision; the validator was corrected.
Later reruns strengthened exact key-error and live-grant assertions. No
product binary was rebuilt.

The installed encrypted PostgreSQL logical backup/restore commands were:

```sh
docker run --platform linux/arm64 --name riauth-a08-encrypted-pg-restore-platform-v2 --label org.riauth.local-evidence=a08-encrypted-f3aba63 --user postgres -e PATH=/usr/lib/postgresql/17/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin -v "$REPO:/src:ro" -v "$EVIDENCE/artifacts:/artifacts:ro" -w /src riauth-q08-postgres-runtime:6ca4779-aarch64-local python3 /src/scripts/recovery-drill-postgres.py --binary /artifacts/platform/riauth --maintenance-binary /artifacts/platform/riauth-maintenance --pg-bin /usr/lib/postgresql/17/bin --evidence /tmp/pg-restore-platform.json > "$EVIDENCE/logs/encrypted-storage/pg-restore-platform-attempt2.log" 2>&1
docker cp riauth-a08-encrypted-pg-restore-platform-v2:/tmp/pg-restore-platform.json "$EVIDENCE/encrypted-storage/local-pg-restore-platform.json"
docker rm riauth-a08-encrypted-pg-restore-platform-v2
docker run --platform linux/arm64 --name riauth-a08-encrypted-pg-restore-essentials --label org.riauth.local-evidence=a08-encrypted-f3aba63 --user postgres -e PATH=/usr/lib/postgresql/17/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin -v "$REPO:/src:ro" -v "$EVIDENCE/artifacts:/artifacts:ro" -w /src riauth-q08-postgres-runtime:6ca4779-aarch64-local python3 /src/scripts/recovery-drill-postgres.py --binary /artifacts/essentials/riauth --maintenance-binary /artifacts/essentials/riauth-maintenance --pg-bin /usr/lib/postgresql/17/bin --evidence /tmp/pg-restore-essentials.json > "$EVIDENCE/logs/encrypted-storage/pg-restore-essentials-attempt1.log" 2>&1
docker cp riauth-a08-encrypted-pg-restore-essentials:/tmp/pg-restore-essentials.json "$EVIDENCE/encrypted-storage/local-pg-restore-essentials.json"
docker rm riauth-a08-encrypted-pg-restore-essentials
```

Each edition passed all 16 checks: authenticated backup, outage
fail-closed behavior, wrong-key and occupied-target refusal, isolated
encrypted PostgreSQL restore, serving gate, wrong recovery ID and missing
attestation refusal, synthetic credential reconciliation, old-session
rejection, fresh user and admin login, doctor, discovery and JWKS. The
first Platform attempt failed because the pre-existing drill omitted
user-creation revision and idempotency inputs; its failed report/log are
preserved, and the corrected rerun passed. This is a local logical restore,
not PostgreSQL physical/PITR evidence.

The previous `../native-arm64-evidence-v2.json`, all archives, images,
reports and binaries are unchanged. `native-arm64-encrypted-evidence.json`
contains absolute paths, byte counts and SHA-256 for every new report/log,
the probe, source validators, these instructions, and the previous manifest.
Official exact CI assets and GitHub run identity, signatures, attestations,
SBOM, publication, deployment, previous-version ARM64 rollback, external
key escrow, TLS PostgreSQL, and physical/PITR recovery remain open. No CI
trigger, tag, push, merge or publication occurred.
