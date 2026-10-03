# D01 bounded request-line timeout rejection: source-only plan

Date: 2026-10-03. Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
Reservation `wave30_D01_bounded_request_read_timeout_plan`; existing worktree
`ed9ac424-59f4-4520-905b-919aea3521eb`, parent
`d88ece174746e3354e2022b8d852fba8050eeac0`.

**Propose only the four helper hunks below for independent root review.** They
reject and discard an exact initial-read TimeoutError on an entirely preflow
connection, record it, and allow at most three such refusals before terminating
on the fourth. They accept no partial request. This is a viable bounded design,
not an implementation, memory pass, or successful browser journey. All local
runtime remains HELD; remote A09 `37097066234` owns validation. No slot was taken.

## Fixed source and actual failed evidence read

Read the entire 757-line helper at
`6f1c2940de4b6f456354812f7d8c24a93100d3fb:scripts/d01-confidential-browser-demo.py`:
35,749 bytes, SHA-256
`75bfb8a63d68aee6ee6b2e500959c0e7d80a6331f1c690022c4300134c7d34f1`.
Read the complete newly accepted 309-line appendix at
`6202ee1d90368895223e649199849a13769d6b46:docs/roadmap/local-wave30-d01-user-browser-review.md`:
17,869 appended bytes, SHA-256
`95cee91aeb8adf8e6493c5fa2ad8d9f61234fd5b5e60b1255904418f57848d87`.
Its whole prior prefix was compared as data, not all reread here. Read the full
66-line safe receipt at fixed `6f1c2940`:
`docs/roadmap/evidence/wave30-d01-a270-runtime-cleanup-root-review.json`, 1,638
bytes, SHA-256 `7561321eb839f96727fccad204912869bbda8934144e6158f802e523cab02f6b`.
No private artifact, header, request bytes, password, or protocol file was read.

The actual composed a270 checkpoint remains **FAILED**: helper1, controller1,
server0, five setup CLI0. The finite observer identifies exact TimeoutError at
`Handler.handle_one_request`, original line468, `self.rfile.readline(...)`.
Protected-before403 plus a fresh semantic page and credential/provider/discovery
validation were observed. Authorization redirect, exchange, callback, native
verification, UserInfo, and authenticated protected access remained false/null.
The two preflow Authorization refusals remain distinct; their sender and that
of the timed-out connection remain UNKNOWN. No timeout counter existed in that
executed source, and no retrospective value of the proposed counter is inferred.

The named read may have buffered partial bytes. Neither that observation nor
this plan establishes an empty connection, browser preconnect, sender, or causal
attribution. Actual absence and owned-child join were verified. The first true
event and whole60s cleanup remain unproven. Historical failures, lost values,
unknown causes, and memory-only successes are not rewritten. This report does
not duplicate Sol3's separately owned retrospective diagnosis.

## Exact prospective claim and behavior

Prospective source scope is only `scripts/d01-confidential-browser-demo.py`:
`Demo.__init__` initializes one private integer, `Handler.handle_one_request`
wraps only its existing initial readline, `main` initializes the corresponding
private record field and copies that integer during its original finally. No
product/Core, guide, controller, verifier, provider, crypto, parser, reply,
cleanup, native binary, or harness change is proposed.

The catch body requires exact built-in TimeoutError identity, `attempted is
False`, all of pending/cookie/subject exactly None, done exactly False, failure
exactly None, and an exact integer counter in 0..3 (bool excluded). Any nonexact
state or subclass uses bare raise into the unchanged original terminal handler.
An exact guarded timeout increments the count once. Counts1–3 leave done/failure
unchanged. Count4 sets `failure="request_timeout"` and done True, then returns.
The main loop cannot start a fifth connection after that terminal assignment;
a manually inconsistent count4 also fails the catch guard. No count exceeds4
through the proposed writes. Authorization's original independent refusal
counter and fourth-refusal rule remain unchanged; neither counter grants access.

Return happens inside the existing request budget, before line-length checking,
HeaderReader installation, parse_request, Host/Authorization/transport/content
checks, target parsing, cookies, get/begin/callback, or reply. The timed-out
request is never normalized, inspected, resumed, retried on the same stream,
parsed, accepted, or routed. No status, checks flag, pending/cookie/subject,
Location, response body, or secret is read/written by the new rejection branch.
The exception variable is not retained or formatted. The existing finite
observer reads only its previously approved traceback/class projection.

`close_connection=True` is already set before the read. Returning leaves it
true, so the handler cannot reenter its request loop on the failed stream.
The original `DemoServer.process_request` finally calls shutdown_request and
clears active on both normal return and exceptions. Its original handler finish
path disposes of the file buffers. This discards any partial request bytes with
the entire connection; no bytes from it are carried into the next accepted
request. A cleanup exception remains an original terminal refusal. This is
source control-flow reasoning; actual close/absence is still an unrun oracle.

A later accepted request starts on a new connection and passes every original
parser, Host, Authorization, transport/content, method, origin, and cookie guard.
The unchanged /login path alone can call begin; nonce/state/S256, confidential
exchange, native signature/token validation, UserInfo, and fresh-cookie protected
access still supply every original success predicate. The new integer is outside
checks; it cannot satisfy any journey predicate or confer authorization.

## Observer and actual controller semantics

First three strict preflow timeouts are explicitly classified as **handled
request rejections**, evidenced by the new private count, not as unexpected
terminal failures. They do not call, clear, reset, or replace the original
unexpected observer. This is a declared classification change for this one
operation, not silent loss of the incident: the count is initialized/finally
copied, persists alongside all original results, and never implies an empty
connection or sender. On the fourth, failure/done become terminal BEFORE calling
the original observer with its existing fallback. All original handler/server/
main unexpected-observation calls and the observer itself remain unchanged.

An initial draft called the observer on every recovered timeout. Static review
rejected that variant: the observer's own BaseException fallback can swallow a
Budget.interrupt Halt while observing. In the old path that always ended in a
terminal failure; on a newly recoverable path it could permit continuation after
an interruption. No such draft executed. Restricting the new observer call to
the already-terminal fourth refusal avoids that new continuation risk without
changing Budget/observer or deferring signals. If root requires observed=true
on every first-three rejection as well, that additional requirement is blocked
by this existing fallback under the permitted scope; reserve a separate policy
change, rather than adopting the unsafe draft or masking interrupts.

For the proposed first three, a later unexpected exception can still become the
original observer's first diagnostic. No rejection overwrites an earlier true
observation if one is already present. A fourth TimeoutError is observed as the
first terminal event when no prior diagnostic exists. The distinction is explicit
counter/result/exit state, not a fabricated diagnostic cause or sender.

Read the full225-line, 17,326-byte original controller command at immutable
`2e29c30d01ccd41a2aac3914e3ac20afa38d324f` in the same user-browser report; SHA-256
`5eb6ab2d3d23b49fd431c41f1932082fe0c6779ad724ac77b7fded10926b26b0`.
The accepted actual appendix independently binds that template. Its
finite_observation/retain_observation sanitize the diagnostic without changing
helper_result, helper_exit, or result. The browser loop refuses a nonzero helper
exit. It does not turn a valid observed=true diagnostic into a failed exit.
The complete helper_result retains the new safe integer in private outer evidence;
existing public summaries do not expose this new count or raw connection data.

Also extracted the exact 113,395-byte composed source at `251a29c` (SHA
`484bd441ef0f3db547d75f6138f2943b07a3ef090c30271a9cdc3d7914dda02e`) as data,
and read its complete finiteObservation/diagnostic, latch/observeController,
related readback collector, and final outcome branches. Lines811–820 diagnostic
store the finite projection without first-failure mutation. Lines829–835 latch
only an actual failure label; lines870–878 gate completion by numeric helper
exit and the accepted final protected-page phase. This suffix is preserved by
the separately accepted two-literal composition; it is not run here.

Therefore a preflow refusal cannot manufacture a successful helper or page.
If a later real exception/failure occurs, the original helper terminal handling
and nonzero exit still produce helper_failed at the controller's first-failure
latch; a diagnostic does not replace that latch or its recorded timing. If the
full journey later succeeds, original checks, zero exit, and final-page predicates
are still mandatory. Even if a retained diagnostic is already true, the unchanged
controller separates it from failure/result and never treats it as a success
predicate. Counts1–3 mean specifically these handled read refusals; count4 marks
the new terminal threshold, subject to independent budget failures. They do not
identify any later failure's cause. Count0 means none of these newly handled
rejections. No sender/empty-connection inference is permitted. Observer projection
failure remains its original bounded fallback, not new journey credit.

**Source-binding prerequisite:** the frozen a270 launcher/controller still pins
helper75b. It would reject a different helper hash. This plan does not edit that
source or silently compute a trusted pin from mutable bytes. Root must separately
review any exact source-pin adoption/composed release before a future journey;
the new helper is not runnable under the old immutable binding unchanged.

## Budgets and preservation proof

`Handler.timeout=5`, the enclosing `budget.limit(5,"request_timeout")`, absolute
helper maximum600, pending180, and all original budget/alarm logic remain exact.
Returning unwinds that context and its original final tick; the listener loop
then performs its original absolute/disk check. There is no new start/deadline,
sleep, request retry, read, listener, or extra budget. Three guarded refusals may
leave the listener available, subject to existing budgets and process health.
Signal-driven Halt/request_timeout or any other BaseException is not caught by
the dedicated TimeoutError clause; no observer call can swallow one during a
newly continuing first-three rejection. Header/body/parse/crypto/reply failures remain
terminal through the original paths. If an independent budget expires while
unwinding the fourth rejection, its original Halt tag takes precedence; the
request_timeout/done terminal assignment is not converted into continuation.
This preserves the original deadline outcome, rather than overriding it to make
an unconditional fixed-tag claim during an already-expired fixture.

Preparation180, active840/inclusive900, outer cleanup60 and their observational
limits are untouched. There is no guarantee of hard cancellation of every native
or GUI operation, nor new proof of whole60 cleanup. No GUI/Driver call was made.

Static assembly used exactly four single-occurrence replacements. Full byte
reversal restored the entire 35,749-byte original; an independent AST reversal
removed the new initialization/dict key/final copy and replaced the new one-read
try with its original assignment, recovering the full original AST. The catch's
try body contains exactly that one assignment/call, with no else/finally. Its
first condition fails via bare raise; its handled path ends via value-less
return. It contains no parser, route, cookie, reply, secret/header/path, additional
read, or exception-content access. All top-level constants/imports and main
success/exception/cleanup logic are restored by the same exact inverse.

Thirty-two of35 functions/methods remain byte/AST-identical; no function is added:

```
Failure.__init__ Halt.__init__ require private_directory private_bytes load_verifier
client_secret Budget.__init__ Budget.__enter__ Budget.interrupt Budget.tick
Budget.limit Budget.close observe_unexpected_failure Demo.invoke Demo.setup
Demo.begin Demo.callback Demo.clear HeaderReader.__init__ HeaderReader.readline
DemoServer.__init__ DemoServer.process_request DemoServer.handle_error
Handler.log_message Handler.record_request_reason Handler.require_request
Handler.send_error Handler.cookies Handler.get Handler.reply QuietParser.error
```

The other three change only as mapped above. Existing accepted-request and all
outer handler exception arms remain exact after removing the nested read try.
No complete private collection, security assertion, receipt, removal, or audit
contract is weakened. This is parser/source proof, not an execution assertion.

## Prospective focused checks; all UNRUN and separately reserved

One small pure-memory validation may be proposed to root after full source review.
It must extract only approved bodies, use synthetic file-reader/close/observer
sinks, never import/main/run the helper, touch a socket, use private input, or
call a provider/native verifier. No harness is authored by this slice. Retain full
finite results/numeric exit before grading, no raw synthetic sentinel contents.
The following are meaningful checks, not a claim of executed case counts:

| Check | Required oracle |
| --- | --- |
| Buffered partial request prefix then exact TimeoutError | Counter increments; no parse/route/reply/header/cookie/secret access; close flag and original shutdown/active-clear path; no byte reuse or content serialization. A buffer prefix is not called an empty connection. |
| First/second/third guarded timeout on fresh handlers | Counts1/2/3, same Demo/absolute start, no done/failure/check/status mutation; no observer call or diagnostic reset; original streams each discarded. |
| Fourth guarded timeout / loop boundary | Count4 and fixed request_timeout/done BEFORE observer; first-only diagnostic retained even on projector error; no reply/dispatch or fifth listener-loop continuation. |
| Each nonexact state independently | attempted=True or0; each pending/cookie/subject non-None; done=True or0; prior failure; counter bool/negative/4: original terminal refusal, no recovery increment. |
| TimeoutError subclass or another exception at initial read | Original exact-class projection and terminal handling; no new allowance. |
| Timeout during header/parse/body or reply | Outside the one-operation catch; original terminal behavior and assertions remain; no partial route acceptance. |
| Budget Halt on context entry, read, or exit | Not swallowed; original deadlines/tag and terminal cleanup win; no reset or continuation at an expired absolute/pending/phase limit. |
| Later genuine fatal failure after a handled timeout | Fatal event reaches original observer (or keeps any prior latch); actual terminal failure/nonzero exit and controller first-failure latch preserved, never relabeled as the handled timeout or a pass. |
| Normal guarded request after a refused connection | Full original successful-request guard chain retained; refusal earned no journey flag, cookies, nonce, pending state, or protected access. |
| Private final counter copy / startup-before-Demo failure | Exact int0..4 field, initial0 if Demo never exists, no public/secret/URL data added; existing record/result/cleanup checks unchanged. |
| Finite controller interpretation | observed=true alone does not latch failure; helper nonzero still does; helper0 still needs exact final-page/phase and full checks. No controller rewrite. |

A real Driver journey is separately HELD and needs root's source/runtime/desktop
reservation, exact helper/composition pins and immutable reviewed setup. It must
still prove sign-in, consent/callback/native verification and protected-after
access under original predicates, with truthful failure/cleanup evidence. No
new probe, anonymous HTTP readiness traffic, sender instrumentation, browser
preconnect assumption, or workaround provider is proposed. A memory result alone
would not close original D01/D05 or rewrite the actual failed checkpoint.

## Exact candidate identities and report-only checks

Full proposed helper: 36,884 bytes /777 lines, SHA-256
`37d32db6fbd8c2c9b676f691d115ecfd1af893724144361971e86f73f573b406`.
Complete four-hunk zero-context forward diff: 1,486 bytes /28 lines, SHA-256
`11f7af8c63652869b4dcdbc446527c96aa7fda378ed9ba0f04af242cb001ca85`.
New dedicated try occupies candidate lines469–486; Handler method462–547.
The zero-context patch requires an exact original-blob check before any later
application (and unidiff-zero support); it is not applied by this report.
The complete source fence below ends with its original LF immediately before the
closing Markdown fence, with no extra source LF. It is public source data only.

Actual checks: fixed Git objects/prefix/body reads; full candidate/diff hashes;
Python AST parse; complete byte/AST inverse;32 protected functions and narrow
control-flow assertions. All passed without candidate execution. Repository
CONTRIBUTING was reread; no applicable ancestor AGENTS was present.
`python3 scripts/check-docs.py` exited0. Immutable witness references, exact
four-hunk forward/reverse application, prior-report preservation and sole-report
scope passed; `git diff --cached --check` is required before this one-file commit.
The first cached whitespace check exited2 on a single space that represented a
blank context line in the initially embedded unified diff. The complete patch
was re-encoded with zero context, retaining all four hunks and identical candidate
bytes; this changes no proposed helper behavior. Final docs, patch inverses and
cached whitespace checks were rerun for this concrete report-format correction.
No build/typecheck, helper import/main/function/case/harness,
CLI/native/provider/HTTP/socket/listener/Driver/browser/version/process inspection,
network query/download/dispatch, private input, deletion, alignment, worker
contact/new worker/task/WT, main/push or task-status change occurred. Previous
reports and all actual failures remain unchanged. Root alone reserves any source
implementation/runtime and decides original D01/D05 disposition.

## Complete proposed diff (data; not applied)

```diff
--- a/scripts/d01-confidential-browser-demo.py
+++ b/scripts/d01-confidential-browser-demo.py
@@ -282,0 +283 @@
+        self.preflow_request_timeouts = 0
@@ -468 +469,18 @@
-                self.raw_requestline = self.rfile.readline(MAX_REQUEST_LINE + 1)
+                try:
+                    self.raw_requestline = self.rfile.readline(MAX_REQUEST_LINE + 1)
+                except TimeoutError as timeout:
+                    if not (type(timeout) is TimeoutError
+                            and demo.attempted is False and demo.pending is None
+                            and demo.cookie is None and demo.subject is None
+                            and demo.done is False and demo.failure is None
+                            and type(demo.preflow_request_timeouts) is int
+                            and 0 <= demo.preflow_request_timeouts < 4):
+                        raise
+                    demo.preflow_request_timeouts += 1
+                    if demo.preflow_request_timeouts == 4:
+                        demo.failure, demo.done = "request_timeout", True
+                        try:
+                            observe_unexpected_failure(demo.unexpected_failure_observation, "handler")
+                        except BaseException:
+                            pass
+                    return
@@ -615,0 +634 @@
+              "preflow_request_timeouts": 0,
@@ -709,0 +729 @@
+            record["preflow_request_timeouts"] = demo.preflow_request_timeouts
```

## Complete proposed helper (data; not executed)

```python
#!/usr/bin/env python3
"""One disposable confidential local-demo application; browser authentication only.

Python 3.11+, POSIX alarms, the pinned recovery verifier and native OpenSSL are
prerequisites. Import starts nothing. The external owner supplies a private lab,
registers the printed client, drives the browser through RiWork Cua.ai Driver,
and owns the IdP/browser lifecycle and the overall 15-minute cleanup deadline.
This helper neither logs in to riAuth nor approves an authorization request.
"""

import argparse
import contextlib
import hashlib
import http.server
import json
import os
import re
import secrets
import shutil
import signal
import stat
import subprocess
import sys
import time
import types
import urllib.parse
from pathlib import Path


ISSUER = "http://localhost:9000"
CLIENT_ID = "local-demo"
ORIGIN = "http://localhost:3000"
AUTHORITY = "localhost:3000"
CALLBACK = ORIGIN + "/callback"
FLOW_COOKIE = "d01_demo_flow"
APP_COOKIE = "d01_local_demo"
VERIFIER_COMMIT = "9cefe7a56425bb73c17753e8766d92320b77da3b"
VERIFIER_BLOB = "3be747d03146f1bcaa3ec012ee8d173b61fa737d"
VERIFIER_SHA256 = "f6dd1aa0b71de4793c9b86ee799012bb31a8fc2d6182976094b44df6ef04de3d"
OPENSSL_SHA256 = "67a83dd6d6d747d50c5d296dffb23e32bae9a2c588c93ae2d77e4c607b455c72"
OPENSSL_VERSION = "OpenSSL 3.6.4 25 Aug 2026 (Library: OpenSSL 3.6.4 25 Aug 2026)"
MAX_FILE = 256 * 1024
MAX_REQUEST_LINE = 8192
MAX_HEADERS = 8192
MAX_HEADER_LINE = 4096
MAX_HEADER_COUNT = 32
MAX_COOKIE = 4096
REQUEST_INVALID_REASONS = {
    "http_parse", "host", "authorization", "transfer_encoding", "expect",
    "content_length", "target_scheme", "target_netloc", "target_fragment",
    "method", "post_target", "origin", "content_type", "cookie_header_count",
    "own_cookie_shape",
}
STOP_FREE_BYTES = 17 * 1024**3 // 2
PENDING_SECONDS = 180
FAILURE_TAGS = {
    "arguments_invalid", "paths_invalid", "private_file_invalid",
    "credential_invalid", "verifier_hash_mismatch", "verifier_import_failed",
    "provider_failed", "discovery_failed", "listener_failed",
    "request_invalid", "request_limit", "request_timeout", "response_timeout",
    "flow_already_started", "callback_invalid", "callback_already_consumed",
    "token_exchange_failed", "token_validation_failed", "userinfo_failed",
    "protected_before_unobserved", "fixture_deadline", "pending_deadline",
    "disk_margin", "disk_observation_failed", "interrupted", "cleanup_failed",
    "evidence_write_failed", "unexpected_failure",
}


class Failure(Exception):
    def __init__(self, tag):
        self.tag = tag if tag in FAILURE_TAGS else "unexpected_failure"
        super().__init__(self.tag)


class Halt(BaseException):
    """Pass through HTTP/verifier Exception handlers to owned finally cleanup."""

    def __init__(self, tag):
        self.tag = tag if tag in FAILURE_TAGS else "unexpected_failure"


def require(condition, tag):
    if not condition:
        raise Failure(tag)


def private_directory(path):
    path = Path(os.path.abspath(path))
    fd = os.open(path, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        info = os.fstat(fd)
        require(stat.S_ISDIR(info.st_mode) and info.st_uid == os.getuid()
                and stat.S_IMODE(info.st_mode) == 0o700, "paths_invalid")
    finally:
        os.close(fd)
    return path


def private_bytes(path, limit=MAX_FILE):
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    try:
        info = os.fstat(fd)
        require(stat.S_ISREG(info.st_mode) and info.st_uid == os.getuid()
                and stat.S_IMODE(info.st_mode) == 0o600
                and 0 < info.st_size <= limit, "private_file_invalid")
        with os.fdopen(fd, "rb", closefd=False) as source:
            raw = source.read(limit + 1)
        require(0 < len(raw) <= limit, "private_file_invalid")
        return raw
    finally:
        os.close(fd)


def load_verifier(path):
    raw = private_bytes(path)
    require(hashlib.sha256(raw).hexdigest() == VERIFIER_SHA256,
            "verifier_hash_mismatch")
    # Execute precisely the verified bytes, without rereading or writing pycache.
    module = types.ModuleType("d01_pinned_recovery_verifier")
    module.__file__ = str(path)
    try:
        exec(compile(raw, str(path), "exec"), module.__dict__)
    except Exception:
        raise Failure("verifier_import_failed") from None
    return module


def client_secret(module, path):
    try:
        saved = module.json_object(private_bytes(path))
    except Failure:
        raise
    except Exception:
        raise Failure("credential_invalid") from None
    client = saved.get("client")
    require(isinstance(client, dict), "credential_invalid")
    settings, scopes = client.get("settings"), client.get("scopes")
    require(client.get("client_id") == CLIENT_ID
            and client.get("confidential") is True
            and client.get("service") is False and client.get("enabled") is True
            and client.get("redirect_uris") == [CALLBACK]
            and isinstance(scopes, list) and len(scopes) == 2
            and all(isinstance(value, str) for value in scopes)
            and set(scopes) == {"openid", "profile"}
            and isinstance(settings, dict)
            and settings.get("token_endpoint_auth_method") is None,
            "credential_invalid")
    secret = saved.get("client_secret")
    require(isinstance(secret, str) and 0 < len(secret) <= 4096
            and all(32 <= ord(value) <= 126 for value in secret),
            "credential_invalid")
    saved.clear()
    return secret


class Budget:
    """One main-thread alarm bounds blocking work and samples free space."""

    def __init__(self, lab, started, seconds):
        self.lab, self.deadline = lab, started + seconds
        self.phase_deadline = self.pending_deadline = None
        self.phase_tag = "fixture_deadline"
        self.minimum_free = None
        self.samples = 0
        self.previous = {}

    def __enter__(self):
        require(hasattr(signal, "setitimer") and hasattr(signal, "SIGALRM")
                and signal.getitimer(signal.ITIMER_REAL) == (0.0, 0.0),
                "arguments_invalid")
        for name in (signal.SIGALRM, signal.SIGINT, signal.SIGTERM):
            self.previous[name] = signal.getsignal(name)
            signal.signal(name, self.tick if name == signal.SIGALRM else self.interrupt)
        self.tick()
        return self

    def interrupt(self, signum, frame):
        raise Halt("interrupted")

    def tick(self, signum=None, frame=None):
        try:
            free = shutil.disk_usage(self.lab).free
        except Exception:
            raise Halt("disk_observation_failed") from None
        self.samples += 1
        self.minimum_free = free if self.minimum_free is None else min(self.minimum_free, free)
        require_free = free >= STOP_FREE_BYTES
        if not require_free:
            raise Halt("disk_margin")
        now = time.monotonic()
        deadlines = [(self.deadline, "fixture_deadline")]
        if self.phase_deadline is not None:
            deadlines.append((self.phase_deadline, self.phase_tag))
        if self.pending_deadline is not None:
            deadlines.append((self.pending_deadline, "pending_deadline"))
        deadline, tag = min(deadlines)
        if now >= deadline:
            raise Halt(tag)
        signal.setitimer(signal.ITIMER_REAL, min(1.0, deadline - now))

    @contextlib.contextmanager
    def limit(self, seconds, tag):
        old = self.phase_deadline, self.phase_tag
        deadline = time.monotonic() + seconds
        if old[0] is None or deadline < old[0]:
            self.phase_deadline, self.phase_tag = deadline, tag
        try:
            self.tick()
            yield
        finally:
            self.phase_deadline, self.phase_tag = old
            # Do not replace an in-flight Halt with another alarm during unwinding.
            if sys.exc_info()[0] is None:
                self.tick()

    def close(self):
        if self.previous:
            signal.setitimer(signal.ITIMER_REAL, 0)
            for name, handler in self.previous.items():
                signal.signal(name, handler)
            self.previous.clear()


def observe_unexpected_failure(holder, site):
    """First-only finite private observation; never replace the original outcome."""
    try:
        if holder["observed"] or site not in ("handler", "server", "main"):
            return
        holder["observed"] = True
        error = sys.exc_info()[1]
        label = "other"
        for kind, name in ((AttributeError, "AttributeError"),
                           (TypeError, "TypeError"), (ValueError, "ValueError"),
                           (KeyError, "KeyError"), (OSError, "OSError"),
                           (BrokenPipeError, "BrokenPipeError"),
                           (ConnectionResetError, "ConnectionResetError"),
                           (TimeoutError, "TimeoutError")):
            if type(error) is kind:
                label = name
                break
        own_function = own_line = None
        codes = (
            (HeaderReader.readline.__code__, "HeaderReader.readline"),
            (DemoServer.process_request.__code__, "DemoServer.process_request"),
            (Demo.begin.__code__, "Demo.begin"),
            (Demo.callback.__code__, "Demo.callback"),
            (Demo.invoke.__code__, "Demo.invoke"),
            (Handler.handle_one_request.__code__, "Handler.handle_one_request"),
            (Handler.send_error.__code__, "Handler.send_error"),
            (Handler.get.__code__, "Handler.get"),
            (Handler.reply.__code__, "Handler.reply"),
            (main.__code__, "main"),
        )
        trace = BaseException.__traceback__.__get__(error, BaseException) if isinstance(error, BaseException) else None
        for _ in range(64):
            if trace is None:
                break
            for code, name in codes:
                if trace.tb_frame.f_code is code and type(trace.tb_lineno) is int and 1 <= trace.tb_lineno <= 1024:
                    own_function, own_line = name, trace.tb_lineno
            trace = trace.tb_next
        if trace is not None:
            own_function = own_line = None
        holder["diagnostic"] = {"site": site, "exception_class": label,
                                "own_function": own_function, "own_line": own_line}
    except BaseException:
        # Projection/storage failures retain the first latch and original handler.
        pass


class Demo:
    def __init__(self, module, workspace, secret, budget, checks, statuses):
        self.module, self.workspace, self.secret = module, workspace, secret
        self.issuer, self.client_id = ISSUER, CLIENT_ID
        self.budget, self.checks = budget, checks
        self.statuses = statuses
        self.stage = "setup"
        self.pending = self.cookie = self.subject = None
        self.attempted = self.done = False
        self.failure = None
        self.request_invalid_reason = None
        self.preflow_authorization_refusals = 0
        self.preflow_request_timeouts = 0
        self.unexpected_failure_observation = {"observed": False, "diagnostic": None}

    def invoke(self, stage, seconds, function, *args, **kwargs):
        self.stage = stage
        try:
            with self.budget.limit(seconds, stage + "_failed"):
                return function(*args, **kwargs)
        except Failure:
            raise
        except Exception:
            # Imported failures are never formatted or copied into the record.
            raise Failure(stage + "_failed") from None

    def setup(self, executable):
        def provider():
            self.openssl = Path(executable).resolve(strict=True)
            info = self.openssl.stat()
            require(str(Path(executable)) == "/opt/homebrew/bin/openssl"
                    and stat.S_ISREG(info.st_mode) and 0 < info.st_size <= 32 * 1024 * 1024,
                    "provider_failed")
            with self.openssl.open("rb") as source:
                require(hashlib.file_digest(source, "sha256").hexdigest() == OPENSSL_SHA256,
                        "provider_failed")
            self.openssl_environment = {key: value for key, value in os.environ.items()
                                        if key in {"PATH", "HOME", "TMPDIR", "LANG", "LC_ALL"}}
            result = subprocess.run([str(self.openssl), "version"], capture_output=True,
                                    timeout=5, env=self.openssl_environment)
            require(result.returncode == 0
                    and len(result.stdout) <= 256 and len(result.stderr) <= 4096
                    and result.stdout.decode("ascii").strip() == OPENSSL_VERSION,
                    "provider_failed")
        self.invoke("provider", 5, provider)
        self.checks["provider_identity_verified"] = True
        document = self.invoke("discovery", 5,
                               self.module.LocalRelyingParty.discovery, self)
        methods = document.get("token_endpoint_auth_methods_supported")
        require(isinstance(methods, list) and "client_secret_post" in methods
                and all(isinstance(value, str) for value in methods), "discovery_failed")
        self.discovery_document = document
        self.checks["discovery_verified"] = True

    def begin(self):
        self.stage = "flow"
        require(not self.attempted, "flow_already_started")
        self.attempted = True
        self.pending = {"state": secrets.token_urlsafe(32), "nonce": secrets.token_urlsafe(32),
                        "verifier": secrets.token_urlsafe(48),
                        "flow_cookie": secrets.token_urlsafe(32),
                        "started_at": int(time.time())}
        self.budget.pending_deadline = time.monotonic() + PENDING_SECONDS
        self.budget.tick()
        fields = {"response_type": "code", "client_id": CLIENT_ID,
                  "redirect_uri": CALLBACK, "scope": "openid profile",
                  "state": self.pending["state"], "nonce": self.pending["nonce"],
                  "code_challenge": self.module.b64url(
                      hashlib.sha256(self.pending["verifier"].encode("ascii")).digest()),
                  "code_challenge_method": "S256"}
        return (self.discovery_document["authorization_endpoint"] + "?"
                + urllib.parse.urlencode(fields), self.pending["flow_cookie"])

    def callback(self, query, flow_cookie):
        self.stage = "callback"
        require(self.pending is not None, "callback_already_consumed")
        pending, self.pending = self.pending, None
        self.budget.pending_deadline = None
        raw = tokens = access = subject = code = form = None
        try:
            require(self.module.equal(flow_cookie, pending["flow_cookie"]), "callback_invalid")
            try:
                code = self.module.LocalRelyingParty.callback_fields(self, query, pending["state"])
            except Exception:
                raise Failure("callback_invalid") from None
            self.checks["state_issuer_flow_cookie_verified"] = True
            form = {"grant_type": "authorization_code", "client_id": CLIENT_ID,
                    "client_secret": self.secret, "redirect_uri": CALLBACK,
                    "code": code, "code_verifier": pending["verifier"]}
            status, _, raw = self.invoke("token_exchange", 5, self.module.request,
                                         self.discovery_document["token_endpoint"],
                                         method="POST", form=form)
            self.statuses["token_exchange"] = status
            require(status == 200, "token_exchange_failed")
            tokens = self.module.json_object(raw)
            access, scope = tokens.get("access_token"), tokens.get("scope")
            require(isinstance(access, str) and 0 < len(access) <= self.module.MAX_TOKEN
                    and re.fullmatch(r"[A-Za-z0-9_.-]+", access) is not None
                    and tokens.get("token_type") == "Bearer" and isinstance(scope, str)
                    and set(scope.split()) == {"openid", "profile"}, "token_exchange_failed")
            self.checks["confidential_s256_exchange_verified"] = True
            # Keep the pinned implementation; cap combined JWKS/signature work
            # at five real seconds as well as its existing socket/process caps.
            subject = self.invoke("token_validation", 5,
                                  self.module.LocalRelyingParty.verify_id_token,
                                  self, tokens.get("id_token"), access, pending,
                                  self.discovery_document)
            self.checks["rs256_jwks_issuer_audience_nonce_time_access_hash_verified"] = True
            status, _, raw = self.invoke("userinfo", 5, self.module.request,
                                         self.discovery_document["userinfo_endpoint"], bearer=access)
            self.statuses["userinfo"] = status
            require(status == 200 and self.module.equal(self.module.json_object(raw).get("sub"), subject),
                    "userinfo_failed")
            self.checks["userinfo_subject_verified"] = True
            self.cookie, self.subject = secrets.token_urlsafe(32), subject
            return self.cookie
        finally:
            pending.clear()
            if form is not None:
                form.clear()
            raw = tokens = access = subject = code = form = None
            self.secret = None

    def clear(self):
        if self.pending is not None:
            self.pending.clear()
        self.pending = self.cookie = self.subject = self.secret = None
        self.budget.pending_deadline = None


class HeaderReader:
    def __init__(self, source):
        self.source, self.remaining, self.lines = source, MAX_HEADERS, 0

    def readline(self, size=-1):
        line = self.source.readline(min(MAX_HEADER_LINE + 1, self.remaining + 1))
        self.remaining -= len(line)
        self.lines += 1
        require(len(line) <= MAX_HEADER_LINE and self.remaining >= 0
                and self.lines <= MAX_HEADER_COUNT + 1, "request_limit")
        return line


class DemoServer(http.server.HTTPServer):
    def __init__(self, demo):
        self.demo, self.active = demo, None
        super().__init__(("127.0.0.1", 3000), Handler)
        self.timeout = 0.2

    def process_request(self, request, address):
        self.active = request
        try:
            self.finish_request(request, address)
        finally:
            self.shutdown_request(request)
            self.active = None

    def handle_error(self, request, address):
        try:
            observe_unexpected_failure(self.demo.unexpected_failure_observation, "server")
        except BaseException:
            pass
        self.demo.failure, self.demo.done = "unexpected_failure", True


class PreflowAuthorizationRefusal(Exception):
    pass


class Handler(http.server.BaseHTTPRequestHandler):
    timeout = 5
    protocol_version = "HTTP/1.0"

    def log_message(self, format, *args):
        pass

    def record_request_reason(self, reason):
        demo = self.server.demo
        if demo.request_invalid_reason is None and reason in REQUEST_INVALID_REASONS:
            demo.request_invalid_reason = reason

    def require_request(self, condition, reason):
        if not condition:
            self.record_request_reason(reason)
            raise Failure("request_invalid")

    def send_error(self, code, message=None, explain=None):
        self.record_request_reason("http_parse")
        self.server.demo.failure, self.server.demo.done = "request_invalid", True
        self.reply(code, "Local demo could not complete this request.")

    def handle_one_request(self):
        demo = self.server.demo
        self.requestline, self.request_version, self.command = "", "HTTP/1.0", None
        self.close_connection = True
        try:
            demo.stage = "request"
            with demo.budget.limit(5, "request_timeout"):
                try:
                    self.raw_requestline = self.rfile.readline(MAX_REQUEST_LINE + 1)
                except TimeoutError as timeout:
                    if not (type(timeout) is TimeoutError
                            and demo.attempted is False and demo.pending is None
                            and demo.cookie is None and demo.subject is None
                            and demo.done is False and demo.failure is None
                            and type(demo.preflow_request_timeouts) is int
                            and 0 <= demo.preflow_request_timeouts < 4):
                        raise
                    demo.preflow_request_timeouts += 1
                    if demo.preflow_request_timeouts == 4:
                        demo.failure, demo.done = "request_timeout", True
                        try:
                            observe_unexpected_failure(demo.unexpected_failure_observation, "handler")
                        except BaseException:
                            pass
                    return
                if not self.raw_requestline:
                    return
                require(len(self.raw_requestline) <= MAX_REQUEST_LINE, "request_limit")
                original, self.rfile = self.rfile, HeaderReader(self.rfile)
                try:
                    if not self.parse_request():
                        return
                finally:
                    self.rfile = original
                self.close_connection = True
                self.require_request(self.headers.get_all("Host") == [AUTHORITY], "host")
                if (self.headers.get_all("Authorization") is not None
                        and demo.attempted is False and demo.pending is None
                        and demo.cookie is None and demo.subject is None
                        and demo.preflow_authorization_refusals < 4):
                    self.record_request_reason("authorization")
                    self.require_request(self.headers.get_all("Transfer-Encoding") is None, "transfer_encoding")
                    self.require_request(self.headers.get_all("Expect") is None, "expect")
                    self.require_request(self.headers.get_all("Content-Length") in (None, ["0"]), "content_length")
                    target = urllib.parse.urlsplit(self.path)
                    self.require_request(not target.scheme, "target_scheme")
                    self.require_request(not target.netloc, "target_netloc")
                    self.require_request(not target.fragment, "target_fragment")
                    demo.preflow_authorization_refusals += 1
                    raise PreflowAuthorizationRefusal
                self.require_request(self.headers.get_all("Authorization") is None, "authorization")
                self.require_request(self.headers.get_all("Transfer-Encoding") is None, "transfer_encoding")
                self.require_request(self.headers.get_all("Expect") is None, "expect")
                self.require_request(self.headers.get_all("Content-Length") in (None, ["0"]), "content_length")
                target = urllib.parse.urlsplit(self.path)
                self.require_request(not target.scheme, "target_scheme")
                self.require_request(not target.netloc, "target_netloc")
                self.require_request(not target.fragment, "target_fragment")
            if self.command == "GET":
                self.get(target)
            elif self.command == "POST":
                self.require_request(target.path == "/login" and not target.query, "post_target")
                self.require_request(self.headers.get_all("Origin") == [ORIGIN], "origin")
                self.require_request(self.headers.get_all("Content-Type") == ["application/x-www-form-urlencoded"], "content_type")
                self.cookies()
                location, cookie = demo.begin()
                self.reply(303, "", location=location, cookie=(FLOW_COOKIE, cookie))
                demo.statuses["authorization_redirect"] = 303
                demo.checks["authorization_redirect_issued"] = True
            else:
                self.record_request_reason("method")
                raise Failure("request_invalid")
        except PreflowAuthorizationRefusal:
            if demo.preflow_authorization_refusals == 4:
                demo.failure, demo.done = "request_invalid", True
            self.reply(403, "Local demo could not complete this request.")
        except Failure as failure:
            demo.failure, demo.done = failure.tag, True
            self.reply(400, "Local demo could not complete this request.")
        except Exception:
            try:
                observe_unexpected_failure(demo.unexpected_failure_observation, "handler")
            except BaseException:
                pass
            demo.failure, demo.done = "unexpected_failure", True
            self.reply(400, "Local demo could not complete this request.")

    def cookies(self):
        headers = self.headers.get_all("Cookie", [])
        self.require_request(len(headers) <= 1, "cookie_header_count")
        raw = headers[0] if headers else ""
        require(len(raw) <= MAX_COOKIE, "request_limit")
        pieces = raw.split(";") if raw else []
        require(len(pieces) <= MAX_HEADER_COUNT, "request_limit")
        own = {}
        for piece in pieces:
            name, separator, value = piece.strip().partition("=")
            if name in (FLOW_COOKIE, APP_COOKIE):
                self.require_request(separator and name not in own
                                     and re.fullmatch(r"[A-Za-z0-9_-]{43}", value) is not None,
                                     "own_cookie_shape")
                own[name] = value
        return own

    def get(self, target):
        demo, cookies = self.server.demo, self.cookies()
        if target.path == "/callback":
            cookie = demo.callback(target.query, cookies.get(FLOW_COOKIE))
            self.reply(303, "", location="/protected", cookie=(APP_COOKIE, cookie), clear_flow=True)
            demo.statuses["callback"] = 303
        elif target.path == "/protected" and not target.query:
            demo.stage = "protected"
            authenticated = (demo.subject is not None and demo.cookie is not None
                             and demo.module.equal(cookies.get(APP_COOKIE), demo.cookie))
            self.reply(200 if authenticated else 403,
                       "Signed in. Protected application access is available." if authenticated
                       else "Sign in required.", protected=authenticated)
            if authenticated:
                demo.checks["protected_with_fresh_cookie_accepted"] = True
                demo.statuses["protected_after"] = 200
                if not demo.checks["protected_without_cookie_denied"]:
                    demo.failure = "protected_before_unobserved"
                demo.done = True
            elif not demo.attempted and APP_COOKIE not in cookies:
                demo.checks["protected_without_cookie_denied"] = True
                demo.statuses["protected_before"] = 403
        elif target.path == "/" and not target.query:
            self.reply(200, "Use Sign in to open this local application.", start=True)
        else:
            self.reply(404, "This page is unavailable.")

    def reply(self, status, text, *, location=None, cookie=None, clear_flow=False,
              start=False, protected=False):
        heading = "Protected application access" if protected else "Local demo"
        form = ('<form method="post" action="/login"><button type="submit">Sign in</button></form>'
                if start else "")
        raw = ('<!doctype html><html lang="en"><meta charset="utf-8">'
               '<title>Local demo</title><h1>' + heading + '</h1><p>' + text + '</p>'
               + form + '</html>').encode("utf-8")
        with self.server.demo.budget.limit(5, "response_timeout"):
            self.send_response(status)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(raw)))
            self.send_header("Connection", "close")
            self.send_header("Cache-Control", "no-store")
            self.send_header("Referrer-Policy", "no-referrer")
            self.send_header("Content-Security-Policy", "default-src 'none'; form-action 'self'; frame-ancestors 'none'")
            if location is not None:
                self.send_header("Location", location)
            if cookie is not None:
                self.send_header("Set-Cookie", cookie[0] + "=" + cookie[1]
                                 + "; HttpOnly; SameSite=Lax; Path=/")
            if clear_flow:
                self.send_header("Set-Cookie", FLOW_COOKIE + "=; Max-Age=0; HttpOnly; SameSite=Lax; Path=/")
            self.end_headers()
            self.wfile.write(raw)
            self.wfile.flush()
        self.close_connection = True


class QuietParser(argparse.ArgumentParser):
    def error(self, message):
        raise Failure("arguments_invalid")


def main():
    started = time.monotonic()
    budget = demo = server = evidence_fd = workspace = secret = None
    stage = "arguments"
    record = {"schema": "riauth.d01-confidential-browser/v1", "result": "failed",
              "failure_stage": None, "failure_tag": None, "checks": {}, "cleanup": {},
              "request_invalid_reason": None, "preflow_authorization_refusals": 0,
              "preflow_request_timeouts": 0,
              "unexpected_failure_observation": {"observed": False, "diagnostic": None},
              "source": {"verifier_commit": VERIFIER_COMMIT, "verifier_blob": VERIFIER_BLOB,
                         "verifier_expected_sha256": VERIFIER_SHA256},
              "provider": None,
              "http_statuses": {name: None for name in
                                ("authorization_redirect", "callback", "token_exchange",
                                 "userinfo", "protected_before", "protected_after")}}
    try:
        parser = QuietParser(add_help=False, allow_abbrev=False)
        for name in ("workspace", "secret-file", "verifier-helper", "openssl", "evidence"):
            parser.add_argument("--" + name, required=True)
        parser.add_argument("--deadline-seconds", type=int, default=600)
        args = parser.parse_args()
        require(sys.version_info >= (3, 11) and 1 <= args.deadline_seconds <= 600,
                "arguments_invalid")
        stage = "paths"
        workspace = private_directory(args.workspace)
        lab = private_directory(workspace.parent)
        require(not any(workspace.iterdir()), "paths_invalid")
        secret_path, verifier_path, evidence_path = [Path(os.path.abspath(value)) for value in
                                                   (args.secret_file, args.verifier_helper, args.evidence)]
        require(private_directory(secret_path.parent).parent == lab
                and verifier_path.parent == lab and evidence_path.parent == lab
                and len({secret_path, verifier_path, evidence_path}) == 3, "paths_invalid")
        budget = Budget(lab, started, args.deadline_seconds)
        budget.__enter__()
        evidence_fd = os.open(evidence_path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
        os.fchmod(evidence_fd, 0o600)
        stage = "verifier"
        module = load_verifier(verifier_path)
        record["source"]["verifier_sha256"] = VERIFIER_SHA256
        record["source"]["helper_sha256"] = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
        stage = "credential"
        secret = client_secret(module, secret_path)
        for name in ("credential_private_validated", "provider_identity_verified", "discovery_verified",
                     "protected_without_cookie_denied", "authorization_redirect_issued",
                     "state_issuer_flow_cookie_verified", "confidential_s256_exchange_verified",
                     "rs256_jwks_issuer_audience_nonce_time_access_hash_verified",
                     "userinfo_subject_verified", "protected_with_fresh_cookie_accepted"):
            record["checks"][name] = name == "credential_private_validated"
        demo = Demo(module, workspace, secret, budget, record["checks"], record["http_statuses"])
        record["unexpected_failure_observation"] = demo.unexpected_failure_observation
        secret = None
        demo.setup(args.openssl)
        record["provider"] = {"sha256": OPENSSL_SHA256, "version": OPENSSL_VERSION}
        demo.stage = "listener"
        try:
            server = DemoServer(demo)
        except Exception:
            raise Failure("listener_failed") from None
        print(json.dumps({"ready": True, "port": 3000, "pid": os.getpid()}), flush=True)
        while not demo.done:
            budget.tick()
            server.handle_request()
        if demo.failure is not None:
            raise Failure(demo.failure)
        require(all(record["checks"].values()), "unexpected_failure")
        record["result"] = "passed"
    except (Failure, Halt) as failure:
        record["failure_tag"] = failure.tag
        record["failure_stage"] = demo.stage if demo is not None else stage
    except KeyboardInterrupt:
        record["failure_tag"], record["failure_stage"] = "interrupted", stage
    except Exception:
        try:
            observe_unexpected_failure(record["unexpected_failure_observation"], "main")
        except BaseException:
            pass
        record["failure_tag"] = "unexpected_failure"
        record["failure_stage"] = demo.stage if demo is not None else stage
    finally:
        cleanup_failed = False
        if budget is not None:
            try:
                budget.close()
            except BaseException:
                cleanup_failed = True
            record["minimum_free_bytes"], record["disk_samples"] = budget.minimum_free, budget.samples
        if server is not None:
            try:
                if server.active is not None:
                    server.shutdown_request(server.active)
                    server.active = None
            except BaseException:
                cleanup_failed = True
            try:
                server.server_close()
            except BaseException:
                cleanup_failed = True
            record["cleanup"]["listener_closed"] = server.socket.fileno() == -1
            record["cleanup"]["connection_closed"] = server.active is None
        if demo is not None:
            record["request_invalid_reason"] = demo.request_invalid_reason
            record["preflow_authorization_refusals"] = demo.preflow_authorization_refusals
            record["preflow_request_timeouts"] = demo.preflow_request_timeouts
            try:
                demo.clear()
                record["cleanup"]["private_references_cleared"] = True
            except BaseException:
                cleanup_failed = True
                record["cleanup"]["private_references_cleared"] = False
        secret = None
        try:
            if workspace is not None:
                record["cleanup"]["verifier_temporaries_removed"] = not any(workspace.iterdir())
            if cleanup_failed or (record["result"] == "passed" and not all(record["cleanup"].values())):
                raise Failure("cleanup_failed")
        except BaseException:
            record["result"] = "failed"
            record["cleanup"]["failure_tag"] = "cleanup_failed"
            if record["failure_tag"] is None:
                record["failure_tag"], record["failure_stage"] = "cleanup_failed", "cleanup"
        record["elapsed_seconds"] = round(time.monotonic() - started, 3)
    written = False
    if evidence_fd is not None:
        try:
            with os.fdopen(evidence_fd, "wb") as output:
                output.write((json.dumps(record, sort_keys=True) + "\n").encode("ascii"))
                output.flush()
                os.fsync(output.fileno())
            written = True
        except Exception:
            record["result"] = "failed"
            record["evidence_failure_tag"] = "evidence_write_failed"
            if record["failure_tag"] is None:
                record["failure_tag"], record["failure_stage"] = "evidence_write_failed", "evidence"
        finally:
            try:
                os.close(evidence_fd)
            except OSError:
                pass
    try:
        print(json.dumps({"result": record["result"], "failure_tag": record["failure_tag"],
                          "cleanup_failure_tag": record["cleanup"].get("failure_tag"),
                          "evidence_failure_tag": record.get("evidence_failure_tag"),
                          "evidence_written": written}), flush=True)
    except Exception:
        return 1
    return 0 if record["result"] == "passed" and written else 1


if __name__ == "__main__":
    raise SystemExit(main())
```
