# D01 independent Essentials password browser checkpoint

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; original D01 task
`a96a1977-3210-4284-8f7d-645793369301`; existing worktree
`7c85f5ef-3fac-4f72-aaed-08474d7fb454`. Observation date: 2026-10-02.
Only this report was reserved. The root owns integration and task status.

Result: **the bounded password browser path passed**. A fresh synthetic
administrator signed in through the printed Essentials section 2 form,
reached the signed-in portal, opened and closed Sign-in and security,
signed out, and signed in again. Final cleanup signed out again. No product
or printed-guide failure was observed in these steps, and no corrective
retry was performed. This is an independent worker checkpoint against the
guide, not a recruited inexperienced human's usability study or a completion
claim for every D01 task. D05's independent user/operator acceptance gate
remains a root decision. O06 remains DONE and closed.

## Exact inputs and provenance

The published guide was read from Git objects at
`6b4db4f0317e4427c187f55e063989ccba124217`, also the observed local main ref.
No merge or guide edit was performed. The authorized prebuilt Essentials
artifacts were the root's c01-source build, with source pin
`c01c39ab4e092423d5522bedc50fff87656d8c0a`. Artifact bytes were independently
hashed before execution; this worker did not reproduce their build.

| Input | Exact observation |
| --- | --- |
| Server | `riauth`, 183038592 bytes, SHA-256 `7abf745c10691a012a1918c83089168d0e0d88b42764dbf8ed538b90c828b606` |
| Maintenance | `riauth-maintenance`, 56499296 bytes, SHA-256 `86490c7f71de6b7ae9d4dabdf9060a8757100f3aedbdaa670284d9e1206a2a95` |
| Binary directory | `/Users/dominik/orca/projects/riAuth-public-preview-local-module-boundaries-wave27/target/d01-essentials-c01c39a/aarch64-apple-darwin/debug` |
| Published Essentials guide | `docs/essentials-guide.md`, blob `e48371d3a34a7b6441cbcfce700daf954aa5f2ae`, SHA-256 `9df3c2861984751d48b28d284cfb47c07801fd3934072e2ccb829edeb35579a0` |
| Published portal guidance | `docs/PORTAL.md`, blob `7c529fcf698e0e07b083d4cfb520bf340f32857e`, SHA-256 `386fa4ca53f4607133b3f30071b7e2714ff4504cb0879b514fe2b52d86ddc015` |
| Older c01 guide, distinct input | Blob `2b8408f90d82bb35d49fc1ebd3512d9a1a853962`, SHA-256 `80f54441cd5cca1cd3c8521ba91b125db0ac8a86c8ee4055986b8f69402c67ee`; its hash was read, its historical observations were not refreshed |
| Live GUI provider | RiWork Cua.ai Driver MCP, daemon 0.30.4, `com.trycua.driver`, `/Applications/CuaDriver.app/Contents/MacOS/cua-driver`; health `ok` on macOS 26.2 arm64 |
| Permission readback | Accessibility and Screen Recording granted; read-only checks did not run the prompt-capable direct ScreenCaptureKit probe |

The local cua-driver skill pack identifies itself as 0.29.1, distinct from
the live daemon. SKILL, MACOS, BROWSER and RUNTIME instructions were consulted,
and advertised tool descriptions/current state were read before browser
input. Instruction-file SHA-256 pins:

| File under `/Users/dominik/.agents/skills/cua-driver/` | SHA-256 |
| --- | --- |
| `SKILL.md` | `593466f73db9e063b3d18a7a939fec79e7e171fe1f2da486f0b780e223426abe` |
| `MACOS.md` | `c3fe476a0321fef996e82cbe0ab06d75a44cf1298daaec24ed681efb9a3b2cc5` |
| `BROWSER.md` | `865da3b26f49f9a4dcdcdcfd245783e604ba81563b19e566579a17a1b46c3ba7` |
| `RUNTIME.md` | `ad032aecfbee63812a2fda201aeb3c8b0ebc56dc8418690c71d2fc2c7850fe20` |

These are instruction and artifact hash checks, not additional body reviews
or runtime tests. The guide's section 2 operator/password path and browser
instructions, section 4 navigation entry, and PORTAL sign-in, sign-out and
security-panel guidance support the recorded actions. Selected c01
`src/cli/local.rs` init and `src/cli.rs` hidden-password code were also read
to deliver the printed interactive setup without adding noninteractive flags.
No broader source audit was performed.

## Disposable operator setup

The ownership ledger's `wave30_D01_independent_user_browser` reservation was
read: approved, runtime released, this exact worktree/shell, one report,
RiWork Cua only, at most 20 minutes and no other desktop controller.

Initial free space was 10.829 GiB and `lsof` found no listener on port 9000.
A new owner-private directory under this worktree's ignored
`deployment-private/d01-user-browser-isnden7n` and a private `xdg` directory
were created with umask 077. A newly generated synthetic password was kept
out of the report and visible command arguments. The printed interactive
command was executed with absolute authorized binary/config paths and the
fresh XDG directory:

```text
riauth-maintenance --config <fresh-private-lab>/riauth.toml init
  --issuer http://localhost:9000 --listen 127.0.0.1:9000
  --data-dir data --admin admin
```

Both hidden password prompts completed; init exited 0 and reported
`initialized: true`. No `--password-stdin`, CLI login, session approval,
HTTP authentication substitute or previous operator fixture was used.

`riauth --config <fresh-private-lab>/riauth.toml serve` owned PID 9986.
The listener readback was exactly PID 9986 on `127.0.0.1:9000`.
The guide's `curl --fail http://127.0.0.1:9000/readyz` entry check, bounded
with `--max-time 5`, exited 0: status `ok`, role `integrated`, version
`0.1.1`. This HTTP read was only the printed readiness prerequisite.
The browser performed all user authentication and navigation.

## Actual browser observations

Driver session `wave30-d01-user-browser` used one explicitly authorized
`browser_prepare` with `allow_launch: true`, profile `isolated_new`.
It returned `launched_isolated_browser`, prepared/owned PID 6760, and
reported no personal profile copying, changed preferences, permission
prompt, existing-browser termination, foregrounding or global input.
`list_windows` returned ordinary window 102201 titled `about:blank`.
Binding reported `status: ok`, `binding_quality: exact`,
`mutation_allowed: true`, endpoint access `driver_owned`:

```text
target: bt-c792e95c-4d86-4c75-b477-134900f71f89
tab:    tab-d0cab2bb-6bed-4ce4-8fa9-6739d7380e6f
```

These were returned session capabilities, not guessed raw CDP identifiers.
Every ref action used a fresh snapshot from this same target/tab/session.
Typing used `browser_type`. Button clicks explicitly selected
`input_route: dom_event`, the documented macOS background route. Those
dispatches report `unverifiable`; their application effects were independently
checked by subsequent page snapshots. They do not prove physical mouse input
or a trusted-click browser ceremony. No raw CDP, Playwright, AppleScript,
legacy page script, personal-profile attachment or provider switch was used.

| Step | Fresh observed evidence |
| --- | --- |
| Open printed `http://localhost:9000/apps` | Snapshot `p845`: “Not signed in”, Username, Password, optional Authenticator or recovery code, and instruction to leave it empty |
| Enter credentials | Username entered through `p845:6`, confirmed as synthetic `admin` in `p846`; password typed through `p846:8`; `p847` confirmed the authenticator field remained empty; no secret value was printed or captured |
| Submit Sign in | `p847:53`; `p848` showed submission pending, then `p849` showed “Signed in as admin (@admin)”, “Connected”, “Welcome back, admin”, and 0 applications |
| Open Sign-in and security | `p849:55`; `p850` showed the named dialog, Password/Change password, Passkeys with “You have no passkeys yet”, and Authenticator app “Off” |
| Close security dialog | `p850:22`; `p851` returned to the signed-in portal |
| Sign out | `p851:92`; `p852` showed “Not signed in”, “You’re signed out”, and the password sign-in form |
| Sign in again | Username remained `admin`; fresh password input via `p852:8`, empty code checked at `p853`, submit `p853:53`; pending `p854`, then `p855` restored signed-in identity, “Connected” and the empty catalogue |
| Final UI cleanup sign-out | `p855:92`; `p856` again confirmed “Not signed in” and “You’re signed out” |

An empty catalogue is expected: this fresh lab had no registered application.
The portal also displayed the missing-email and extra-verification notices;
the latter remained informational here and no application access was attempted.
No passkey, Touch ID, authenticator enrollment, password change or recovery
button was activated. Sessions and consent was visible as a link but was not
opened; session-list/consent management is not claimed.

No screenshot or video was requested at any point, including secret entry.
Raw passwords, cookies, session tokens, private signing material and browser
storage were not copied into durable evidence. Browser snapshots were used
for page state, not an accessibility conformance test.

## Cleanup, release and limitations

Finally cleanup used Driver `kill_app` only on the proven driver-owned
PID 6760; it reported SIGKILL sent. `end_session` reported inactive.
Subsequent Driver `list_windows` returned zero windows for that PID and
`list_apps` no longer listed it. No personal browser process was terminated.
Driver-managed profile-file erasure was not separately inspected or claimed.

The owned server/controller received Ctrl-C/SIGINT and exited 0. The private
lab, store, logs and XDG directory were removed in the controller's finally
block. Server elapsed time was **186.997 seconds**; minimum monitored free
space **9.598 GiB**, above the 8.5 GiB stop margin and 8 GiB floor. Cleanup
readback at `2026-10-02T12:26:34.616383+00:00` found the lab absent, owned
server PID absent, no listener on 9000 (`lsof` exit 1), and 9.904 GiB free.
The complete checkpoint stayed within the authorized 20-minute bound; an
exact overall browser/preflight duration was not separately instrumented.
Immediate result and sole desktop/runtime resource release were sent through
the explicit project orchestrator before this report was committed.

Preflight discovery initially checked a guessed `riauth-server` filename;
the authorized server is `riauth`, which was located and hash-verified before
execution. A read-only implicit session-state query returned
`session_not_started`; starting the declared named lifecycle then succeeded.
Neither was a product/guide failure or a permission/binding refusal. All
actual product commands and required page transitions above succeeded.

This checkpoint leaves install/download/release provenance execution, an
ordinary non-administrator user, an independent recruited human, application
OIDC redirect/consent, physical/synced/phone passkeys, Safari/Firefox/mobile,
invitation/mail, password lifecycle, session-list/consent management,
LDAP, SCIM, PostgreSQL, HTTPS/proxy deployments, backup/recovery and the
other Essentials/Platform tasks untested here. It supplies bounded password
browser evidence only and does not imply all-D01, all-D05, current CI,
release, HA, interoperability or deployment acceptance.

No Cargo, build, other product binary, existing source/test/config/guide edit, merge/reset,
main/push/status operation, new worker/task/worktree, service installation,
settings/security change or other-worker contact occurred. The only tracked
change is this reserved report. Existing accepted receipt-secret, header,
PAM, held Group and nonrenewed-lease/paused-I/O limits remain undisturbed.

Report verification: `python3 scripts/check-docs.py` exited 0 with
“Markdown links and build-directory layout checked”; `git diff --check`
passed. These documentation checks are separate from the browser checkpoint.
