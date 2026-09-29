# Q10 installed release gate: first bounded slice

The native `package` job now runs
[`check-installed-release-gate.py`](../../scripts/check-installed-release-gate.py)
once on Linux x86-64 and once on Linux ARM64, after packaging and the existing
archive/image smoke. It takes the checked tag's commit, repository, run ID and
run attempt from the release job. The gate reads the nine expected assets for
that architecture. It rejects missing or extra files, bad SHA256SUMS entries,
wrong provenance or lockfile hashes, wrong target/toolchain/feature metadata,
and an installed maintenance binary whose hash differs from provenance. It
extracts only named regular binaries from the checked archives into a private
temporary install directory. No Cargo build or checkout binary supplies a
result.

The installed server and maintenance binaries must report one version, their
exact edition features and Linux target architecture. The installed `riauthctl`
must report that version. The gate then uses a fresh, isolated redb instance to
exercise the following supported path:

1. Essentials maintenance `keygen` and `init`; Essentials server readiness,
   JWKS, administrator login and a verified encrypted v3 backup through the
   installed server CLI.
2. Platform maintenance read-only preflight records the store schema, index
   version and source edition. The installed Platform server opens the same
   store, becomes ready, preserves JWKS and accepts administrator login.
3. A direct return to Essentials must fail with independent edition provenance
   and build activation blockers. Platform maintenance must produce a ready
   transition plan and activate its exact token. Essentials must then reopen
   with preserved JWKS and a working administrator login.
4. Essentials maintenance restores the pre-transition backup into a new redb
   directory. Serving must stay closed until explicit restored-state recovery
   completion. The old session must fail, while a fresh administrator login and
   the original JWKS work after completion. This is an isolated same-version
   rollback rehearsal; no traffic or live deployment changes.

The release step is:

```sh
python3 ../../scripts/check-installed-release-gate.py . \
  "$RIAUTH_COMMIT" "$GITHUB_REPOSITORY" "$GITHUB_RUN_ID" "$GITHUB_RUN_ATTEMPT"
```

It runs after `cd target/dist` in `.github/workflows/release.yml`. The script
prints a JSON result with installed binary hashes and checked outcomes only
after every assertion passes. Its disposable password, key, store, backup and
session files are removed with the temporary directory. Failed assertions
exit nonzero and prevent a release artifact upload.

## Evidence on the Q10 worktree

At accepted head `679f2927885d9dc4dcc1c881fd972bcade70e07a`, the host was
ARM64 macOS and `target/dist` did not exist. This exact command exited 1:

```sh
python3 scripts/check-installed-release-gate.py target/dist \
  679f2927885d9dc4dcc1c881fd972bcade70e07a \
  Rhein-Industries/riAuth 1 1
```

Its result was `installed release gate failed: exact release asset directory
unavailable: .../target/dist`. The `1 1` run fields are local placeholders;
the gate stopped before provenance checks or binary execution. No installed
release-artifact result is claimed from this host.

`python3 -m unittest discover -s tests -p 'test_installed_release_gate.py' -v`
passed four small metadata fixtures: complete exact manifest, missing asset
rejection, checksum/revision rejection, and missing run identity. They test
the preflight's failure behavior, not release binaries or recovery behavior.
The public CI check job
runs the same small fixture tests before any release tag is packaged.

## Remaining gates and Q08 boundary

The native Linux x86-64 and Linux ARM64 release jobs must run this gate and the
existing archive/image smoke on assets from one checked tag. The publish job
must still pass `check-release-bundle.py` across both architectures. Neither a
native run nor a complete release asset set exists locally for the accepted
head. Docker images, deployed services, PostgreSQL restore, external identity
peers, and actual previous-version schema migration or rollback remain outside
this slice. A version rollback needs a compatible previous release binary and
backup format; the current package job supplies only one version.

The accepted [Q08 evidence](q08-exact-edition-bundles.md) covers local macOS
source binaries built from `4ca7558ee8563a0c5eedc1f13d4a9a6faa6a8d4a`.
Those hashes do not identify artifacts built from the later accepted head.
Q08's exact Linux ARM64 and release archive/image gaps remain open until the
native release jobs supply and verify them.
