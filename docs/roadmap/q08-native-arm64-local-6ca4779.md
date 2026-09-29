# Q08 native Linux ARM64 local artifact run at `6ca4779`

This is **local build and smoke evidence**, not a shipped release. Five binaries,
five local native archives, and two local container archives are preserved under
`target/q08-native-linux-arm64-6ca4779-20260929/`. The absolute evidence root is
`/Users/dominik/orca/projects/riAuth-public-preview-roadmap-q08-exact-bundles-wave16/target/q08-native-linux-arm64-6ca4779-20260929`.
Its private, redacted `native-arm64-evidence.json` records full artifact and log
paths, SHA-256 hashes, byte counts, local image IDs, script hashes, and results.
That manifest's SHA-256 is
`c7063655e4ba24802913a5c945060966a40922664a1d5fd93140bc2e083bcb5a`.
The evidence root is mode `0700`; logs and JSON are mode `0600`. Disposable
passwords, keys, sessions, stores, and PostgreSQL cluster data were removed.

The Cargo source was clean at accepted commit
`6ca4779b68cf41e29af490d2aca6ea3d59e1f2bd` before the build. Subsequent
validator-only commits `d498c13ee17f6971654768912a8d0516d4c8d62c` and
`f12257906c5b000dd2b4d3f342abd46e68cf4b44` changed no `src`, Cargo
manifest, lockfile, or toolchain file. The binary source remains exactly the
accepted commit. The previous tip is recoverable at
`refs/recovery/q08-before-native-arm64-20260929`.

## Native build and provenance

Docker 29.7.2 reported Linux `aarch64` with 16 CPUs. The source Dockerfile's
pinned Rust base was
`rust:1.98.1-trixie@sha256:a8a5f0a1e5fe7dfe1d352591e4a1c7dd2c08fd70475cae872cf3458ba0df0546`;
the runtime base was
`debian:trixie-slim@sha256:a99cfc517144bc59b1978475ec53b46ecabec7e43635402ee5b77cc54cd1b20a`.
Private local build/runtime images used those exact bases. The native builder
reported `rustc 1.98.1 (48a229cea 2026-09-01)` and
`uname -m = aarch64`. `Cargo.lock` SHA-256 was
`b5c9d11c001244b8017303ce8c20516e02946483845eee40720b0910758d4426`;
the standalone client lock SHA-256 was
`6f546c966c70505b8ef204cea811f7300b1b359de63f01a8389caadc3164b964`.
The build container's apt package versions are in `logs/build.log`; pinned
base images do not pin later apt repository contents.

`container/build.sh` in the evidence root ran these exact Cargo commands with
`CARGO_BUILD_JOBS=4`, `CARGO_INCREMENTAL=0`,
`CARGO_HOME=/evidence/cargo-home`, and a fresh
`/evidence/cargo-target` separate from all accepted targets:

```sh
CARGO_TARGET_DIR=/evidence/cargo-target/server cargo build --release --locked --no-default-features --features essentials --bins --target aarch64-unknown-linux-gnu -j 4
CARGO_TARGET_DIR=/evidence/cargo-target/server cargo build --release --locked --no-default-features --features platform --bins --target aarch64-unknown-linux-gnu -j 4
CARGO_TARGET_DIR=/evidence/cargo-target/client cargo build --manifest-path crates/riauthctl/Cargo.toml --release --locked --no-default-features --target aarch64-unknown-linux-gnu -j 4
```

It copied each server and edition-matched maintenance binary before the next
feature build. `file` identified every binary as an ARM aarch64 GNU/Linux ELF;
`readelf -h` reported `ELF64` and `AArch64` five times. The build log and the
preserved binaries have these SHA-256 hashes:

| Local binary path under `artifacts/` | SHA-256 |
| --- | --- |
| `essentials/riauth` | `5feac36b7ad5d292cafb192b673f4353f8fa424e90a35d667d8fa7f41cac74eb` |
| `essentials/riauth-maintenance` | `90b2056e8ad679de1c34e236ccf1fa9f5d1719aa0880d9bfe36bdb81b02241be` |
| `platform/riauth` | `606a2ce5d72de16fe8749efdb3504fd81471c88118008180667a013ccffea79d` |
| `platform/riauth-maintenance` | `413b3fb6d0b3f94c83bb5d3ba04d2bc9fc316dbd8ea3103438d4690e8904072a` |
| `client/riauthctl` | `573de0718e3301f2553145eb3d8fd01ce82bc69525a2176f075abc612c02a1e1` |

`container/assemble-local-archives.sh` assembled five named **local** archives
under `local-archives/`. `archive-integrity.json` verifies three regular
members per archive, byte-for-byte source license and notices, and an exact
match to each preserved binary. SHA-256 hashes in the same order as the
Essentials server, Essentials maintenance, Platform server, Platform
maintenance, and client rows above are
`d3039b777fb6268242eb9f3872aecce9a18004685b0b967c54bdd3bacde5c8ea`,
`9b9e2a2cf8d20ef5feb847b4bb3d12e16f90af178cf0a4069743eb0a719b0a0c`,
`e167c628b9a509231a353979a98e796e09e21a4d42e7c27f067d102d4e9a5f3c`,
`ce094917e3543d1e1fc06317251de86d570e203c4af07cb7eae8365d6cc7f864`,
and `804727728e1458a57f74e8924dba6db1e0af2c8d2144277387ca437349b04573`.
The two saved local image archives under `local-images/` have SHA-256
`344dc217157fb34fdb09c1c90f4eb439ecc775ad5624a3648e738ae9aa845b07`
(Essentials) and
`1a4baf092002696d127f6e69d0997fd0edeb9d27cbf158ef29883fb0b06959db`
(Platform). `docker image inspect` reported both as `linux/arm64`, with the
accepted revision and edition labels. Each image's server binary hash matched
the corresponding standalone artifact. These local images use a small private
Dockerfile over the pinned runtime base; they are not outputs of the release
workflow's multi-stage Dockerfile.

## Focused execution results

| Command or evidence file under the evidence root | Result |
| --- | --- |
| `container/local-edition-matrix.py` → `local-edition-matrix.json` | Passed in native ARM64 Linux: Essentials 60 compiled and 27 excluded capabilities; Platform 87 compiled. Both fresh redb instances reached `/readyz`, capabilities, and admin login (HTTP 200). Essentials rejected SAML rate-limit and device-trust overrides. Platform accepted the SAML rate limit; changing its active capability set after initialization was rejected by the stored security agreement. |
| `container/local-postgres-smoke.py` → `local-postgres-smoke.json` | Passed with native PostgreSQL 17.11: both editions initialized separate disposable databases, opened them, and returned HTTP 200 for readiness, capabilities, and admin login. |
| `container/check-closures.py` → `dependency-closure.json` | Passed locked ARM64 normal dependency checks: Essentials 280 packages, Platform 329, standalone `riauthctl` 106. No terminal USB dependencies in either server; no server, store, or USB dependencies in the base client. The direct optional Platform dependency delta is 15, including `wasmi`. |
| `container/same-edition-recovery.py` → `same-edition-recovery.json` | Passed independently for both editions on redb: maintenance keygen/init, server JWKS/login, verified encrypted v3 backup, isolated maintenance restore, serving blocked pending recovery, explicit completion, old session rejection, new administrator login, and preserved JWKS. |
| `python3 scripts/check-edition-artifacts.py` on all five local archives → `logs/archive-smoke.log` | **Exit 1.** Edition archive bootstrap, route, and agent checks advanced through Platform agent creation after the focused header fix. The final direct downgrade assertion expected a stored-agent blocker; the binary refused earlier with `Configured active capabilities do not match the initialized instance`. The release smoke gate remains failing. |
| `container/local-image-smoke.py` → `logs/local-image-smoke.log` | **Exit 1.** Both local image bootstrap, route, and agent checks passed. The final downgrade assertion failed for the same earlier active-capability refusal. Only labeled disposable volumes and containers were removed. |
| `container/local-installed-smoke.py` using accepted `check-installed-release-gate.py` helpers → `logs/local-installed-smoke.log` | **Exit 1.** Installed binary metadata and initial Essentials backup passed. Platform could not open that Essentials store: `Configured active capabilities do not match the initialized instance`. An exploratory config that disabled the compiled Platform-only delta failed read-only preflight because `access.temporary_entitlements` cannot be disabled. The official Q10 gate was left unchanged. |

The local same-edition restore result is not an Essentials→Platform→Essentials
recovery result. The active-capability disagreement blocks that full transition
before Platform serves; it needs a reviewed product/transition design. The
archive and image smoke keep their stricter expected downgrade blocker. No
release gate assertion was loosened to turn the failures into passes.

## Remaining A09, Q08, Q10, and Q11 gates

`GITHUB_RUN_ID`, `GITHUB_RUN_ATTEMPT`, and `GITHUB_REPOSITORY` were unset, and
`target/dist` contained no official exact asset set. Consequently
`package-release.sh`, the official `check-installed-release-gate.py` entry
point, and `check-release-bundle.py` were not run with invented identity or
CI provenance. The local `drill` helper above ran directly against preserved
native binaries without passing the official asset verifier.

- **A09/Q08:** Native ARM64 local binaries and images now exist, with redb and
  PostgreSQL smoke. The tag-checked native x86-64 and ARM64 CI package jobs,
  exact twelve-file per-architecture release sets, source SPDX package
  documents, checksums, GitHub run provenance, both archive/image smoke gates,
  and combined-bundle verification remain open. The local assets are not
  uploaded, signed, or published.
- **Q10:** The installed cross-edition recovery gate fails at Essentials to
  Platform open. Previous-version schema migration, rollback with an actual
  older binary, PostgreSQL restore, and deployed-service recovery also lack
  evidence here. Same-edition isolated restore passed as described above.
- **Q11:** No official release SBOM, signature or attestation, separate
  vulnerability/remediation review record, draft-asset review, or publication
  evidence was produced by this local run. Source notices and local archive
  contents were verified only for the local assets.

Host free space was about 80 GiB before the build and 58.4 GiB at the manifest
update; every observation exceeded the 8 GiB floor. Builds used four jobs at
most. No accepted Cargo target, main, remote branch, or release was changed;
no global Docker prune was run. The local images and preserved artifacts remain
available for review.
