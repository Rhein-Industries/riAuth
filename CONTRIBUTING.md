# Contributing to riAuth

riAuth v0.1.1 is an early public release. Bug reports, documentation corrections, interoperability results, and focused patches are welcome. Check the [release limitations](docs/limitations.md) and [existing issues](https://github.com/Rhein-Industries/riAuth/issues) before starting a larger change.

For a vulnerability, follow the [security policy](SECURITY.md). Keep exploit details, credentials, and personal data out of public issues and pull requests.

## Repository layout

| Path | What belongs there |
| --- | --- |
| `src/` | The server, CLI, protocols, storage, and embedded portal assets in `src/portal/` |
| `tests/` | Rust integration tests and shared fixtures; larger identity suites are split under `tests/identity/` |
| `docs/` | User, protocol, and operator guides; focused advanced-feature guides live in `docs/enterprise/` |
| `examples/` | Sample configuration and synthetic example data |
| `deploy/` | Deployment and integration templates |
| `scripts/` | Local checks, integration setup, and release helpers |
| `tools/browser/` | Browser test project and its locked JavaScript dependencies |
| `fuzz/` | Parser fuzz targets and seed corpora |

Link new guides from the [documentation index](docs/README.md). Put generated build output and local credentials outside tracked source; the [getting-started guide](docs/getting-started.md) shows the private output directory used in examples.

## Build and check changes

Use the pinned Rust 1.93.0 toolchain. The [getting-started guide](docs/getting-started.md#prerequisites) lists native build prerequisites; Linux builds need a C/C++ compiler, CMake, pkg-config, OpenSSL development headers, and udev development headers.

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --features test-support,fuzzing --locked -- -D warnings
cargo test --all-targets --features test-support,fuzzing --locked
python3 scripts/check-repo-hygiene.py
python3 scripts/check-docs.py
python3 scripts/generate-third-party-notices.py --check
```

Run the tests most relevant to your change while developing, then run the full local set above before submitting. Tests that need a browser, Docker, PostgreSQL, OpenLDAP, Traefik, XMLsec, hardware, or external tenants are described in the relevant guides and [testing guide](docs/testing.md). State which of those environments you actually used; a local fixture does not establish compatibility with a live peer.

If Cargo.lock or the Linux release dependency graph changes, run `python3 scripts/generate-third-party-notices.py` and commit the updated [third-party notices](THIRD_PARTY_NOTICES.md). The generator checks the locked versions and source licenses; do not edit the generated file by hand.

## Submit a pull request

- Explain the behavior changed, why it matters, and any compatibility or migration effect. Link the issue or specification when relevant.
- Add a regression test for a bug or protocol change. Keep test credentials synthetic and fixtures free of live secrets.
- Update the user guide, API contract, and release notes when behavior or operator steps change.
- Include the local commands you ran and their results. Identify any external acceptance that remains open.

Provide the local results with your pull request. Maintainers may ask for an integration run against the affected protocol or deployment environment before merging.

The project source is [MIT licensed](LICENSE). Dependencies retain their own terms and notices; preserve required attribution when adding third-party material.
