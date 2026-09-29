# Q08 exact edition bundle matrix

Observation: 2026-09-29 UTC. Source revision
`4ca7558ee8563a0c5eedc1f13d4a9a6faa6a8d4a`; `Cargo.lock` SHA-256
`f3c9de0e5c5bd7b5bea6aa1f7ce1e2fc827815b94838f1aec2f8231e1f9fc347`.
Host: ARM64 macOS (`aarch64-apple-darwin`), Rust 1.98.1. The full observed
result, including every compiled and excluded capability, is in
[q08-exact-edition-matrix.json](q08-exact-edition-matrix.json). Both native
builds used that one revision with explicit features and an isolated Cargo
target. These are local source builds; no Linux release bundle was tested.
The matrix JSON SHA-256 is
`fe6cf68d59d8f71f2bc0fd9f60404e7162f3f953bd4bdbb8545d422f78b19a88`;
its embedded script SHA-256 is
`d7b9315b894905c8e033f81eeeb69a68d3016259558a59dde9fb0e237b744b9e`.

## Reproduce

From the repository root, with Rust 1.98.1, Python 3, and the usual native
build dependencies:

```sh
python3 scripts/check-exact-edition-matrix.py \
  --target-dir "$PWD/target/q08-exact-bundles" \
  --out "$PWD/target/q08-exact-bundles/matrix.json"
```

The script runs `cargo build --locked --no-default-features --features
essentials --bins` and the corresponding `platform` build, with
`CARGO_TARGET_DIR=$PWD/target/q08-exact-bundles`,
`CARGO_PROFILE_DEV_DEBUG=0`, and `CARGO_BUILD_JOBS=2`. It copies both server
and maintenance binaries after each build, before the next feature set can
replace `target/debug` binaries. It checks the binary capability documents,
the direct normal dependency trees, two Platform-only configuration cases, and
fresh redb and PostgreSQL instances. A disposable loopback PostgreSQL cluster
is started only when `initdb`, `pg_ctl`, and `createdb` are installed. A missing
cluster tool is reported as unavailable, not as PostgreSQL evidence. The
script removes test instances and stops its cluster. It also inventories the
expected Linux release files under `target/dist` (override with
`--release-dir`); presence is reported without claiming provenance verification.

## Observed local matrix

| Check | Essentials | Platform |
| --- | --- | --- |
| Explicit Cargo build | Passed; `build_features=[essentials]` | Passed; `build_features=[essentials,platform]` |
| Compiled capability catalog | 60 compiled; 26 excluded | 86 compiled; none excluded |
| Direct normal dependencies | 38 | 52; 14 additional optional direct dependencies |
| `rate_limits.saml = 10` | Rejected, exit 1: `rate_limits.saml requires the Platform build` | Accepted; server ready and login 200 |
| `capabilities.disabled = ["identity.device_trust"]` | Rejected, exit 1: `Disabled capability identity.device_trust requires the Platform build` | Accepted; server ready and login 200 |
| Fresh redb | Init passed; `/readyz`, `/api/capabilities`, admin login all 200 | Same |
| Fresh PostgreSQL | Init passed; `/readyz`, `/api/capabilities`, admin login all 200 | Same |

The 26 excluded Essentials features are exactly the Platform-only set in the
machine-readable report, including SAML, RADIUS, proxy, inbound SCIM, cloud
directory, device trust, temporary access, Windows login, and SSF features.
The direct-dependency delta is `flate2`, `hmac`, `hyper`, `hyper-util`,
`ldap3_proto`, `md-5`, `psl`, `risaml`, `roxmltree`, `sha1`, `time`,
`tokio-rustls`, `tokio-util`, and `x509-parser`. The direct dependency check
does not assert their absence from Essentials' transitive closure.
Artifact-scope capability `usable` fields are null; local readiness and login
prove only the exercised instances, not external peers or all features.

Native binary SHA-256:

| Edition | `riauth` | `riauth-maintenance` |
| --- | --- | --- |
| Essentials | `59676422a1d7597daf4a41a6e3544d4740a61291362bd68f39e0908f4553d220` | `135ebfbdd172491adbab1744035e752306e939e99bcdcaefb655bf4f7a1853d7` |
| Platform | `a520836f2258e9416c898047ed6f9773a21e2fb3ece7bd4b9844729e91c3e4cb` | `5bb13a25b887e15811964a6b8da9de117ffd740af33912c5f9ab5c2ddb9cfd22` |

## Architecture and release artifact gate

The local `target/dist` had none of the expected nine files per Linux
architecture. Linux x86-64 and Linux ARM64 native archives, maintenance
archives, `riauthctl` archives, container archives, checksum files, and build
provenance for this revision are **unavailable** in this worktree. Native
Linux ARM64 execution and complete release artifact verification remain open.
The release workflow specifies both Linux architectures and
`scripts/check-release-bundle.py` requires a complete, same-revision set;
neither is evidence that those jobs ran for this revision.

Read-only GitHub release inspection used:

```sh
gh release list --repo Rhein-Industries/riAuth --limit 20 \
  --json tagName,isDraft,publishedAt,createdAt
gh release view v0.1.1 --repo Rhein-Industries/riAuth \
  --json assets,createdAt,isDraft,tagName
gh release download v0.1.1 --repo Rhein-Industries/riAuth \
  --pattern build-provenance.json --dir target/q08-exact-bundles/remote-release
```

The sole listed release was draft `v0.1.1`. Its four assets were a generic
`riauth-linux-x86_64.tar.gz`, generic x86-64 Docker archive,
`build-provenance.json`, and `SHA256SUMS`; there were no edition-specific or
ARM64 assets. Its provenance identified commit
`5ef0261e8b6d54e3532b69041511c84d9887648a` and Rust 1.93.0, so those
assets cannot verify the tested revision. A draft release is not a published
artifact. Recheck release state when running the final release gate.

The remaining acceptance work is to build and execute both editions from one
tag on native Linux x86-64 and ARM64, run the packaged archive/image smoke
checks, validate complete provenance and checksums with
`scripts/check-release-bundle.py`, and inspect any newly uploaded assets.
