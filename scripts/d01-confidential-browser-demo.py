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
OPENSSL_VERSION = "OpenSSL 3.6.4 25 Aug 2026"
MAX_FILE = 256 * 1024
MAX_REQUEST_LINE = 8192
MAX_HEADERS = 8192
MAX_HEADER_LINE = 4096
MAX_HEADER_COUNT = 32
MAX_COOKIE = 4096
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
        self.demo.failure, self.demo.done = "unexpected_failure", True


class Handler(http.server.BaseHTTPRequestHandler):
    timeout = 5
    protocol_version = "HTTP/1.0"

    def log_message(self, format, *args):
        pass

    def send_error(self, code, message=None, explain=None):
        self.server.demo.failure, self.server.demo.done = "request_invalid", True
        self.reply(code, "Local demo could not complete this request.")

    def handle_one_request(self):
        demo = self.server.demo
        self.requestline, self.request_version, self.command = "", "HTTP/1.0", None
        self.close_connection = True
        try:
            demo.stage = "request"
            with demo.budget.limit(5, "request_timeout"):
                self.raw_requestline = self.rfile.readline(MAX_REQUEST_LINE + 1)
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
                require(self.headers.get_all("Host") == [AUTHORITY]
                        and self.headers.get_all("Authorization") is None
                        and self.headers.get_all("Transfer-Encoding") is None
                        and self.headers.get_all("Expect") is None
                        and self.headers.get_all("Content-Length") in (None, ["0"]),
                        "request_invalid")
                target = urllib.parse.urlsplit(self.path)
                require(not target.scheme and not target.netloc and not target.fragment,
                        "request_invalid")
            if self.command == "GET":
                self.get(target)
            elif self.command == "POST":
                require(target.path == "/login" and not target.query
                        and self.headers.get_all("Origin") == [ORIGIN]
                        and self.headers.get_all("Content-Type") == ["application/x-www-form-urlencoded"],
                        "request_invalid")
                self.cookies()
                location, cookie = demo.begin()
                self.reply(303, "", location=location, cookie=(FLOW_COOKIE, cookie))
                demo.statuses["authorization_redirect"] = 303
                demo.checks["authorization_redirect_issued"] = True
            else:
                raise Failure("request_invalid")
        except Failure as failure:
            demo.failure, demo.done = failure.tag, True
            self.reply(400, "Local demo could not complete this request.")
        except Exception:
            demo.failure, demo.done = "unexpected_failure", True
            self.reply(400, "Local demo could not complete this request.")

    def cookies(self):
        headers = self.headers.get_all("Cookie", [])
        require(len(headers) <= 1, "request_invalid")
        raw = headers[0] if headers else ""
        require(len(raw) <= MAX_COOKIE, "request_limit")
        pieces = raw.split(";") if raw else []
        require(len(pieces) <= MAX_HEADER_COUNT, "request_limit")
        own = {}
        for piece in pieces:
            name, separator, value = piece.strip().partition("=")
            if name in (FLOW_COOKIE, APP_COOKIE):
                require(separator and name not in own
                        and re.fullmatch(r"[A-Za-z0-9_-]{43}", value) is not None,
                        "request_invalid")
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
