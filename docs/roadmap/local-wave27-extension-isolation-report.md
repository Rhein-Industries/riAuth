# W07 local wave 27 extension isolation report

Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`
Assigned task: `63ab3917-0fe0-4d02-819e-fc0c09b72ca6` — W07 controlled extensions
Worktree: `e1b4399a-8c0d-46b8-880c-a71a4ebf53e7`
Path: `/Users/dominik/orca/projects/riAuth-public-preview-local-extension-isolation-wave27`
Branch: `roadmap/local-extension-isolation-wave27`
Base: `4cc1c8bf82f48d9561f1f61c7fb13487b8d610b0`
Date: 2026-10-01

## Delivered slice and commits

Implementation commit: `6421a1e79392fe483130f727a365066cbdb88ee6` —
`Isolate installed extension guests with fail-closed macOS sandbox`.
This report is a separate evidence-only commit; its own hash is available
from the branch log under `Record wave27 extension isolation evidence`.

The slice adds macOS kernel confinement and fail-closed inherited-descriptor
admission to the existing killable installed-server guest. It does not complete
the whole W07 task. Its RiWork status remains `in_progress`; arbitrary custom
graphs and external/platform acceptance remain open.

The repository has no applicable `AGENTS.md` in this worktree or its ancestor
directories. Read `CONTRIBUTING.md`, `SECURITY.md`, the integration verdict,
the existing guest implementation, workflow documentation, and the assigned
RiWork task details. The task record's older scheduling hold is superseded by
the user's explicit current assignment to implement. No model settings were
changed and no workers were spawned.

The project orchestrator confirmed ownership of `extension_gate.rs`,
`extension_gate/**`, this report, and only the **Controlled extensions** and
**Isolated guest** sections of `docs/workflows.md`. It recorded the remaining
Linux integration-test adjustments. No SAML executor/assembly, activation,
version, accepted checkout, or main-branch source was edited. No merge, push,
external message, or real cloud mutation was performed.

## Files and behavior

| File | Change |
| --- | --- |
| `src/workflow/extension_gate.rs` | Launch through the isolation backend; refuse unsupported executable discovery; check native isolation before reading IPC; scope execution tests to the supported backend while retaining portable admission/binding assertions. |
| `src/workflow/extension_gate/isolation.rs` | Fixed macOS sandbox launcher, exact image/library parameters, descriptor audit, direct unsandboxed-entry refusal, and unsupported-host refusal. |
| `src/workflow/extension_gate/guest-macos.sb` | Default-deny Seatbelt profile; bounded loader read exceptions, selected CPU/page-size sysctls, and metadata/descriptor inspection. |
| `src/workflow/extension_gate/native-probe.c` | Test-only native probes of file/network/child creation denial, inheritable file/socket refusal, and kernel kqueue closure across exec. |
| `docs/workflows.md` | Correct W07 platform scope, loader exceptions, descriptor refusal, and sandbox dependencies within the two assigned sections. |
| `docs/roadmap/local-wave27-extension-isolation-report.md` | This evidence and integration handoff. |

The macOS parent launches `/usr/bin/sandbox-exec`, with the fixed embedded
profile and argv parameters, then the canonical installed/renamed server
image and its existing internal guest token. No shell or environment override
selects the executable or policy. Missing sandbox setup fails closed.

The profile denies ungranted operations, including network access, filesystem
writes, Mach service lookup, fork, and child spawn. Only the selected image
may exec. Loader reads include `/System/Library`, `/usr/lib`, that exact image,
and the two resolved conventional Homebrew OpenSSL dylibs. File metadata,
root-directory reads, `/dev/fd` enumeration, selected CPU/page-size sysctl reads,
and `/dev/null` writes are allowed. This is not a claim that all host metadata
is hidden. Other dependency locations receive no fallback directory read.

Before reading a request, the native helper enumerates `/dev/fd`, requires
that the directory iterator is the sole descriptor above stderr, drops it,
and verifies its descriptor is absent. Extra descriptors and inspection
errors deny. Only `ENOENT` or Darwin `EBADF` after dropping the sole iterator
is accepted. Requiring one iterator also prevents a non-vnode descriptor
from being mistaken for a closed descriptor solely because stat fails.
Extras are refused rather than closed through unsafe raw-fd ownership hooks.

Native entry probes must also observe permission denial for opening
`/etc/passwd` and binding loopback. They do not read file contents or contact
a peer. These guard checks reject an ordinary direct argv invocation; they
are not an attestation of arbitrary caller-provided sandbox policies.

The existing bounded request/result buffers, module/function/local/stack/fuel
limits, startup compilation deadline, per-step deadline before acceptance and
after joined IO, empty environment, and process-bound `stage_binding` remain
in place. No new imports, network permission, proof, claim, transaction,
idempotency, authorization, or audit semantics were added. The parent still
needs to be scheduled to enforce a deadline: a paused parent, OS spawn/reap
delays, and thread scheduling have no exact kernel wall-time guarantee.

## Verification actually performed

Native environment: macOS 26.2, build `25C56`, Apple Silicon;
Rust `1.98.1 (48a229cea 2026-09-01)`.

All Cargo build and test invocations used the private
`CARGO_TARGET_DIR=$PWD/.cargo-target-wave27`, `CARGO_BUILD_JOBS=1`,
`CARGO_INCREMENTAL=0`, and `CARGO_PROFILE_DEV_DEBUG=0`. No accepted/shared target
was used. Build output and probe logs are retained locally under the assigned
worktree, relocated after verification to ignored `target/wave27/` and its
`evidence/` directory. Disk was checked during builds and stayed well above
the 8 GiB stop threshold (approximately 56 GiB free in the last sample).

| Check | Result |
| --- | --- |
| `pwd && git status --short --branch` | Runtime worked; correct worktree and initially clean assigned branch. |
| `cargo build --locked --bin riauth` | Passed; final link completed in 56.93 s, with the compact-unwind warning noted below. |
| `cargo test --locked --lib workflow::extension_gate::tests -- --test-threads=1` | Passed: 19 tests, 0 failures, 95 filtered; final run 33.06 s after compilation. |
| `cargo fmt --all -- --check` | Passed after formatting owned Rust files. |
| `git diff --check` and staged diff check | Passed. |
| Link validation for the two owned Markdown files | Passed; local Markdown targets resolve. |
| Native profile/loader/entry probes | Final profile returned the exact bounded `allow` response from the installed server; early probes refused until required loader files, page-size sysctl, and the closed iterator's `EBADF` were handled. |

The focused suite covers the copied/renamed installed image, empty/sentinel
environments, startup validation, late finished-child observation, late joined
IO, kill/reap of a nonreturning child, capped flooding output, missing/corrupt
images, identifier withholding, stack/fuel/module/output caps, and request
binding that leaves Wasmi compilation to the child. New native regressions
cover file reads/writes, loopback binding, fork and same-image `posix_spawn`
denial, inherited file/socket refusal with a valid request, independently
verified Darwin kqueue closure across exec, and ordinary
unsandboxed entry refusal. The C fixtures compile with
`cc -Wall -Wextra -Werror`; they are never part of the production binary.

Initial focused runs failed closed on ungranted Homebrew dylib reads and on
Darwin's closed-descriptor stat result. These were diagnosed and corrected;
an additional probe showed that Darwin itself closes kqueues across exec.
The regression now proves that with `fcntl` before accepting a kqueue-origin
guest launch; it does not falsely credit the helper audit with that closure.
Only the final source/run result is the slice's acceptance evidence. No broad
test campaign, release build, live identity flow, accessibility test, or
cross-platform native run was performed.

## Residual gaps and integration dependencies

- Linux, Windows, and other hosts have no implemented kernel backend. Source
  paths refuse with `external_runtime_required` before spawning; their new
  refusal test was not executed on this macOS host. No native Linux/Windows
  sandbox or compatibility completion is claimed.
- Intel macOS, other macOS versions, and installed dependency layouts beyond
  this host were not exercised. The conventional Intel Homebrew library path
  is source handling, not a compatibility result.
- Apple's `sandbox-exec` is deprecated. Absence, policy compilation/application
  failure, ungranted loader requirements, and descriptor-audit failures deny;
  there is no unsandboxed retry. Packaging must retain that dependency or
  implement another supported backend before widening platform support.
- Server dependencies or operators that deliberately leave inheritable
  descriptors open will cause safe refusal. Automated descriptor closing
  remains unimplemented; future work must preserve the crate's
  `unsafe_code = forbid` contract or explicitly review any changed boundary.
- The orchestrator must adapt workflow/configuration integration tests on
  unsupported hosts to explicit refusal, while retaining portable bounds,
  permissions, approval, and stage-binding checks. Only macOS-dependent guest
  execution tests are platform-scoped here; no unrelated tests were edited.
- Parent suspension/OS scheduling limits remain. Blocking spawn/reap and
  joined IO are not an exact real-time kernel deadline; late results are
  discarded once observed. Child-creation denial prevents guest descendants
  from retaining IPC pipes.
- The debug server link emits the macOS linker warning about the large
  `__eh_frame` section/compact unwind encoding. The link succeeds; this slice
  does not fix that existing packaging/toolchain concern.
- Arbitrary custom workflow graphs, additional guest ABIs, WASI/imports,
  network extensions, independent operator acceptance, and whole W07
  completion remain outside this delivered slice.

## Primary references consulted

The installed Apple `man sandbox-exec` states its deprecation and that the
command enters the specified sandbox before executing the command. The
installed Apple `man fd` documents `/dev/fd` descriptor handles. Native probes
above establish this host's behavior; neither manual establishes Linux or
Windows behavior.

Apple's [posix_spawn manual](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/posix_spawn.2.html)
and [XNU exec implementation](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/kern/kern_exec.c)
describe descriptor inheritance and special close-on-exec spawn handling.
Rust's [upstream Unix process implementation](https://github.com/rust-lang/rust/blob/master/library/std/src/sys/process/unix/unix.rs)
was reviewed as a reference, not proof of every detail in the pinned local
toolchain. The implementation therefore does not assume that `Command`
closes all inheritable descriptors. The independent native audit and
descriptor regressions enforce the admission contract.
