#!/usr/bin/env python3
"""One local session-read measurement for a riAuth binary.

The measured operation is GET /api/me (Core::me) on a fresh loopback instance.
One invocation is one binary and one backend, redb or PostgreSQL. A second pass
creates empty groups during the same read so writer overlap and the server's
background counters are both recorded. Optional --directory-users and
--directory-groups add a bounded directory and verify the administrator's
group index before measurement. Both default to zero. --pace-ms sleeps
between measured reads so a pass can outlast the 60-second maintenance cadence.

python3 scripts/q09_benchmark_slice.py --self-check
python3 scripts/q09_benchmark_slice.py --binary PATH --backend redb --out FILE
python3 scripts/q09_benchmark_slice.py --binary PATH --directory-users 2 --directory-groups 2
python3 scripts/q09_benchmark_slice.py --binary PATH --secure --backend postgresql --out FILE

Build the binary in a private CARGO_TARGET_DIR with CARGO_INCREMENTAL=0.
Do not use another worktree's target directory. A relative --binary is
resolved from the caller's directory before any subprocess. --self-check
does not start riAuth and does not publish timings.
"""

import argparse
import base64
import contextvars
import hashlib
import json
import os
import platform
import re
import secrets
import shutil
import signal
import socket
import ssl
import subprocess
import sys
import tempfile
import threading
import time
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = Path(__file__).resolve()
SCHEMA = "riauth.benchmark-slice/v1"
GENERAL_LIMIT = 600
BUDGET_MAX = 500
INDEX_PAGE_ROWS = 128
DIRECTORY_USERS_MAX = 32
DIRECTORY_GROUPS_MAX = 32
PACE_MS_MAX = 15000
MAINTENANCE_CADENCE_SECONDS = 60
BACKGROUND_JOBS = (
    "reconciliation",
    "provisioning",
    "mail",
    "logout_ssf",
    "maintenance",
    "alerts",
    "manual_connector",
    "deactivation",
)
INSECURE_LISTENER = "The listener is loopback HTTP without TLS. Database encryption is off."
INSECURE_POSTGRES = (
    "PostgreSQL uses one disposable loopback cluster with trust auth, sslmode=disable, and local_unencrypted."
)
SECURE_LISTENER = (
    "The listener is loopback HTTPS. The client trusts one fresh private CA and checks the certificate hostname. "
    "The certificate names DNS:localhost and IP:127.0.0.1. A fresh database_key_file encrypts records as aes256gcm-v1."
)
SECURE_POSTGRES = (
    "PostgreSQL is one disposable loopback cluster with scram-sha-256 on hostssl and local_unencrypted false. "
    "libpq preflight uses sslmode=verify-full and the private CA. The measured binary connection file uses "
    "sslmode=require with host localhost and hostaddr 127.0.0.1. This binary parser accepts disable, prefer, "
    "and require, then sets SslMode::Require. rustls checks that hostname against the webpki roots and ca_file. "
    "An unrelated CA is refused by libpq and by product init, and sslmode=disable is refused."
)
SECURE_REDB = "This run uses the redb backend."
SECURE_SIGNER = "This observation records no external signer and no published release identity."
SECURE_DOCKER_VM = "This observation ran inside Docker's Linux VM."
TLS_CONTEXT = contextvars.ContextVar("q09_tls_context", default=None)
HOSTSSL_HBA = (
    "hostssl all all 127.0.0.1/32 scram-sha-256\n"
    "hostssl all all ::1/128 scram-sha-256\n"
)
LIMITATIONS = (
    "One run measures one binary and one backend with one sequential session-read client.",
    "Essentials and Platform both serve Core::me. An edition comparison is two runs at the same settings.",
    "redb and PostgreSQL are selected with --backend. A backend comparison is two runs at the same settings.",
    INSECURE_LISTENER,
    INSECURE_POSTGRES,
    "The default dataset is a fresh init: one administrator and no extra users or group memberships.",
    "Optional directory flags add users and groups, put the administrator and every extra user in each of those groups, and verify that index before measurement.",
    "Directory flags are capped at 32 and by the shared general-request budget. That stays inside one 128-row group-index page.",
    "Latency is the client clock around the HTTP exchange, including loopback.",
    "Percentiles are nearest-rank ceil(n*q) over every attempt in the pass, including errors.",
    "Successful throughput is successful attempts divided by that pass's wall time.",
    "RSS and CPU are ps samples of the serve process. The interval is 50 ms.",
    "Host load average is recorded beside the server samples and is not removed from them.",
    "The interference pass is concurrent POST /api/groups with an empty member set, Idempotency-Key, and If-Match.",
    "Interference creates empty groups and does not change the administrator memberships recorded for the session read.",
    "Server background counters come from /api/operations/metrics. The closing metrics read is inside the delta.",
    "The default general rate limit is 600 requests per minute. Estimated general requests must stay at or below 500.",
    "Build profile, toolchain, and feature set are recorded only from the operator flags. The script does not recover them from the binary. The artifact sha256 is of the measured file.",
    "This report is an observation of the recorded run. It assigns no load, recovery, or capacity target.",
    "Optional --pace-ms sleeps between measured reads. The sleep is outside each latency sample and inside the pass wall clock. Maintenance cadence overlap requires that sleep to make the pass longer than 60 seconds and the maintenance finished counter to increase during the pass.",
)


class SliceError(RuntimeError):
    pass


def digest(path):
    checksum = hashlib.sha256()
    with Path(path).open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            checksum.update(chunk)
    return checksum.hexdigest()


def scrub(text, hidden):
    value = str(text)
    for secret in hidden:
        if secret:
            value = value.replace(secret, "[redacted]")
    return value


def fail(message, hidden):
    raise SliceError(scrub(message, hidden))


def percentile_us(samples, numerator, denominator):
    ordered = sorted(samples)
    count = len(ordered)
    if count == 0:
        raise ValueError("empty samples")
    index = (count * numerator + denominator - 1) // denominator
    index = min(max(index, 1), count) - 1
    return ordered[index]


def distribution(samples):
    if not samples:
        return {"count": 0, "population": "all_attempts", "unit": "microseconds"}
    return {
        "count": len(samples),
        "population": "all_attempts",
        "unit": "microseconds",
        "method": "nearest-rank ceil(n*q)",
        "min_us": min(samples),
        "p50_us": percentile_us(samples, 1, 2),
        "p95_us": percentile_us(samples, 95, 100),
        "p99_us": percentile_us(samples, 99, 100),
        "max_us": max(samples),
    }


def parse_cputime(text):
    value = text.strip()
    days = 0
    if "-" in value:
        day_text, value = value.split("-", 1)
        days = int(day_text)
    parts = value.split(":")
    if len(parts) == 2:
        hours = 0
        minutes = int(parts[0])
        seconds = float(parts[1])
    elif len(parts) == 3:
        hours = int(parts[0])
        minutes = int(parts[1])
        seconds = float(parts[2])
    else:
        raise ValueError(f"unrecognized ps time {text!r}")
    return ((days * 24 + hours) * 60 + minutes) * 60 + seconds


def directory_plan(users, groups):
    """Users and memberships the administrator session read will walk."""
    if not isinstance(users, int) or not isinstance(groups, int) or users < 0 or groups < 0:
        raise SliceError("directory sizes must be integers >= 0")
    if users > DIRECTORY_USERS_MAX or groups > DIRECTORY_GROUPS_MAX:
        raise SliceError(
            f"directory sizes are capped at {DIRECTORY_USERS_MAX} users and {DIRECTORY_GROUPS_MAX} groups"
        )
    extra = [f"q09-user-{index:04d}" for index in range(1, users + 1)]
    names = [f"q09-dir-{index:04d}" for index in range(1, groups + 1)]
    memberships = [
        {"group": name, "username": username}
        for name in names
        for username in ("admin", *extra)
    ]
    return {
        "extra_users": users,
        "extra_usernames": extra,
        "groups": names,
        "memberships": memberships,
        "writes": users + groups + len(memberships),
    }


def dataset_name(plan):
    if plan["writes"] == 0:
        return "fresh-init"
    return f"q09-{plan['extra_users']}u-{len(plan['groups'])}g"


def build_note(profile="unrecorded", toolchain="unrecorded", features="unrecorded"):
    """Operator-supplied build identity. This does not inspect the binary."""
    note = {}
    for key, value in (("profile", profile), ("toolchain", toolchain), ("features", features)):
        if value is None:
            value = "unrecorded"
        if (
            not isinstance(value, str)
            or not value
            or len(value) > 64
            or any(character.isspace() or not character.isprintable() for character in value)
        ):
            raise SliceError(f"build {key} must be one short token")
        note[key] = value
    return note


def request_budget(iterations, warmup, interference_cap, directory_users=0, directory_groups=0):
    plan = directory_plan(directory_users, directory_groups)
    # One revision read, one write, and one conflict refresh per directory
    # mutation, plus the administrator read, user list, group list, and the
    # pre-write session read used to learn the administrator id.
    directory_revision_reads = (1 + plan["writes"]) if plan["writes"] else 0
    directory_verification = 4 if plan["writes"] else 0
    terms = {
        "runtime_capabilities": 1,
        "metrics": 4,
        "warmup_reads": 2 * warmup,
        "measured_reads": 2 * iterations,
        "revision_reads": 1 + interference_cap,
        "group_creates": interference_cap,
        "directory_revision_reads": directory_revision_reads,
        "directory_writes": plan["writes"],
        "directory_verification": directory_verification,
    }
    return {
        "general_limit_per_minute": GENERAL_LIMIT,
        "estimated_general_requests": sum(terms.values()),
        "terms": terms,
        "login_requests": 1,
        "directory": {
            "users": directory_users,
            "groups": directory_groups,
            "users_max": DIRECTORY_USERS_MAX,
            "groups_max": DIRECTORY_GROUPS_MAX,
            "index_page_rows": INDEX_PAGE_ROWS,
            "memberships": len(plan["memberships"]),
        },
    }


def pace_seconds(pace_ms):
    if isinstance(pace_ms, bool) or not isinstance(pace_ms, int) or pace_ms < 0 or pace_ms > PACE_MS_MAX:
        raise SliceError(f"pace-ms must be an integer from 0 through {PACE_MS_MAX}")
    return pace_ms / 1000


def maintenance_cadence_overlap(finished_delta, elapsed_seconds):
    """True when this pass both outlasts the maintenance period and sees a finish."""
    return (
        isinstance(finished_delta, int)
        and not isinstance(finished_delta, bool)
        and finished_delta >= 1
        and isinstance(elapsed_seconds, (int, float))
        and elapsed_seconds > MAINTENANCE_CADENCE_SECONDS
    )


def validate_shape(iterations, warmup, interference_cap, directory_users=0, directory_groups=0):
    if iterations < 1 or warmup < 0 or interference_cap < 1:
        raise SliceError("iterations must be >= 1, warmup >= 0, and interference cap >= 1")
    budget = request_budget(
        iterations, warmup, interference_cap, directory_users, directory_groups,
    )
    if budget["estimated_general_requests"] > BUDGET_MAX:
        raise SliceError(
            f"estimated general requests {budget['estimated_general_requests']} exceed {BUDGET_MAX}"
        )
    if directory_groups > INDEX_PAGE_ROWS:
        raise SliceError("directory group count exceeds one group-index page for the administrator")
    return budget


def ensure_redacted(report, hidden):
    encoded = json.dumps(report, sort_keys=True)
    for secret in hidden:
        if secret and secret in encoded:
            raise SliceError("report included a credential")
    assert_report_text_clean(encoded)
    return encoded


def assert_report_text_clean(encoded):
    lowered = encoded.lower()
    for needle in ("-----begin private", "-----begin rsa private", "password="):
        if needle in lowered:
            raise SliceError("report included secret material")


def remember_secret(hidden, value):
    text = str(value)
    if len(text) < 12:
        raise SliceError("refusing to track a short credential")
    if text not in hidden:
        hidden.append(text)


def assert_secrets_absent(blob, hidden, label):
    data = blob if isinstance(blob, (bytes, bytearray)) else str(blob).encode()
    for secret in hidden:
        if secret and secret.encode() in data:
            raise SliceError(f"{label} contains a credential")


def linuxkit_vm(host):
    uname = host.get("uname") if isinstance(host, dict) else None
    if not isinstance(uname, dict):
        return False
    blob = " ".join(str(uname.get(key, "")) for key in ("system", "node", "release", "version"))
    return "linuxkit" in blob.lower()


def secure_limitations(host, backend):
    if backend not in ("redb", "postgresql"):
        raise SliceError("backend must be redb or postgresql")
    lines = []
    replaced_listener = False
    replaced_postgres = False
    for line in LIMITATIONS:
        if line == INSECURE_LISTENER:
            lines.append(SECURE_LISTENER)
            replaced_listener = True
        elif line == INSECURE_POSTGRES:
            lines.append(SECURE_POSTGRES if backend == "postgresql" else SECURE_REDB)
            replaced_postgres = True
        else:
            lines.append(line)
    if not replaced_listener or not replaced_postgres:
        raise SliceError("secure limitations could not find the plaintext lines")
    lines.append(SECURE_SIGNER)
    if linuxkit_vm(host):
        lines.append(SECURE_DOCKER_VM)
    return lines


def completion_code(report):
    limited = 0
    errors = 0
    for name in ("quiet", "interference"):
        limited += int(report["passes"][name]["error_statuses"].get("429", 0))
        errors += int(report["passes"][name]["errors"])
    if limited:
        return 3
    if errors:
        return 4
    if not report["interference"]["overlap"]:
        return 2
    for name in ("quiet", "interference"):
        passage = report["passes"][name]
        if (
            passage.get("elapsed_seconds", 0) > MAINTENANCE_CADENCE_SECONDS
            and passage.get("maintenance_cadence_overlap") is False
        ):
            return 5
    return 0


def command_output(args, *, cwd, env, input_text=None, timeout=60, hidden=()):
    try:
        result = subprocess.run(
            args,
            cwd=cwd,
            env=env,
            input=input_text,
            text=True,
            capture_output=True,
            timeout=timeout,
            check=False,
        )
    except subprocess.TimeoutExpired:
        fail(f"timeout: {args[0]} {args[-1]}", hidden)
    except OSError as error:
        fail(str(error), hidden)
    tail = scrub((result.stderr or "")[-800:], hidden)
    return result.returncode, result.stdout, tail


def git_text(args):
    result = subprocess.run(args, cwd=ROOT, text=True, capture_output=True, check=False)
    if result.returncode != 0:
        raise SliceError(f"{args} failed: {result.stderr.strip()}")
    return result.stdout.strip()


def source_identity():
    commit = git_text(["git", "rev-parse", "HEAD"])
    status = git_text([
        "git", "status", "--porcelain", "--untracked-files=all", "--",
        "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "src", "crates",
        "scripts/q09_benchmark_slice.py",
    ])
    channel = ""
    for line in (ROOT / "rust-toolchain.toml").read_text().splitlines():
        if line.strip().startswith("channel"):
            channel = line.split("=", 1)[1].strip().strip('"')
    return {
        "commit": commit,
        "dirty_paths": [line[3:] for line in status.splitlines() if line],
        "checkout_rust_toolchain": channel,
        "binary_compiler": "unrecorded",
    }


def hardware():
    info = {
        "uname": platform.uname()._asdict(),
        "python": platform.python_version(),
        "cpu_count": os.cpu_count(),
        "loadavg": list(os.getloadavg()) if hasattr(os, "getloadavg") else None,
    }
    if sys.platform == "darwin":
        for key, args in (
            ("cpu_brand", ["sysctl", "-n", "machdep.cpu.brand_string"]),
            ("memory_bytes", ["sysctl", "-n", "hw.memsize"]),
            ("logical_cpu", ["sysctl", "-n", "hw.ncpu"]),
            ("macos_version", ["sw_vers", "-productVersion"]),
        ):
            result = subprocess.run(args, text=True, capture_output=True, check=False)
            info[key] = result.stdout.strip() if result.returncode == 0 else None
        if str(info.get("memory_bytes") or "").isdigit():
            info["memory_bytes"] = int(info["memory_bytes"])
    return info


def read_ps(pid):
    result = subprocess.run(
        ["ps", "-p", str(pid), "-o", "rss=", "-o", "time="],
        text=True, capture_output=True, check=False,
    )
    if result.returncode != 0 or not result.stdout.strip():
        raise OSError(result.stderr.strip() or f"ps exited {result.returncode}")
    rss_text, cpu_text = result.stdout.split()
    return {"rss_kib": int(rss_text), "cpu_seconds": parse_cputime(cpu_text)}


class ProcessSampler:
    def __init__(self, pid):
        self.pid = pid
        self.samples = []
        self.read_errors = 0
        self._lock = threading.Lock()
        self._stop = threading.Event()
        self._thread = threading.Thread(target=self._run, name="q09-ps", daemon=True)

    def start(self):
        self.capture()
        self._thread.start()

    def capture(self):
        try:
            sample = read_ps(self.pid)
        except (OSError, ValueError):
            with self._lock:
                self.read_errors += 1
            return
        with self._lock:
            self.samples.append(sample)

    def stop(self):
        self._stop.set()
        self._thread.join(timeout=1)
        self.capture()

    def _run(self):
        while not self._stop.wait(0.05):
            self.capture()

    def summary(self):
        with self._lock:
            samples = list(self.samples)
            read_errors = self.read_errors
        if not samples:
            return {
                "available": False,
                "samples": 0,
                "read_errors": read_errors,
                "sampler": "ps -p PID -o rss= -o time=",
            }
        rss = [sample["rss_kib"] for sample in samples]
        cpu = [sample["cpu_seconds"] for sample in samples]
        return {
            "available": True,
            "samples": len(samples),
            "read_errors": read_errors,
            "rss_kib_max": max(rss),
            "rss_kib_min": min(rss),
            "rss_kib_start": rss[0],
            "rss_kib_end": rss[-1],
            "cpu_seconds": round(max(0.0, cpu[-1] - cpu[0]), 6),
            "sampler": "ps -p PID -o rss= -o time=",
            "rss_unit": "kibibyte",
            "interval_seconds": 0.05,
        }


def http_exchange(method, url, body=None, headers=None, timeout=5):
    payload = None if body is None else json.dumps(body).encode()
    request = urllib.request.Request(url, data=payload, headers=headers or {}, method=method)
    context = TLS_CONTEXT.get()
    started = time.perf_counter()
    try:
        if context is None:
            response_cm = urllib.request.urlopen(request, timeout=timeout)
        else:
            response_cm = urllib.request.urlopen(request, timeout=timeout, context=context)
        with response_cm as response:
            raw = response.read()
            status = response.status
    except urllib.error.HTTPError as error:
        try:
            raw = error.read()
            status = error.code
        finally:
            error.close()
    except (urllib.error.URLError, TimeoutError, ssl.SSLError, ConnectionError, OSError):
        elapsed = int(round((time.perf_counter() - started) * 1_000_000))
        return elapsed, None, None, True
    elapsed = int(round((time.perf_counter() - started) * 1_000_000))
    try:
        parsed = json.loads(raw) if raw else None
        unparsed = False
    except json.JSONDecodeError:
        parsed = None
        unparsed = True
    return elapsed, status, parsed, unparsed


def classify_me(status, parsed, unparsed, username, groups=None):
    if status is None:
        return False, "transport"
    if status == 200 and isinstance(parsed, dict):
        user = parsed.get("user")
        if isinstance(user, dict) and user.get("username") == username:
            found = parsed.get("groups")
            if groups is not None and (not isinstance(found, list) or sorted(found) != sorted(groups)):
                return False, "unexpected_groups"
            return True, None
        return False, "unexpected_body"
    if unparsed:
        return False, "unparsed_body"
    if isinstance(parsed, dict) and isinstance(parsed.get("error"), str):
        return False, parsed["error"]
    return False, "http_error"


def auth_headers(token, extra=None):
    headers = {"Authorization": f"Bearer {token}"}
    if extra:
        headers.update(extra)
    return headers


def require_document(status, parsed, label, hidden):
    if status != 200 or not isinstance(parsed, dict):
        fail(f"{label} failed with status {status}", hidden)
    return parsed


def counter_snapshot(metrics):
    runtime = metrics.get("runtime") if isinstance(metrics, dict) else None
    jobs = runtime.get("background", {}).get("jobs") if isinstance(runtime, dict) else None
    if not isinstance(jobs, dict):
        raise SliceError("metrics response has no runtime.background.jobs")
    missing = [name for name in BACKGROUND_JOBS if name not in jobs]
    if missing:
        raise SliceError(f"metrics missing background jobs: {', '.join(missing)}")
    try:
        snapshot = {
            "requests_total": int(metrics["requests_total"]),
            "responses_error_total": int(metrics["responses_error_total"]),
            "rate_limited_total": int(metrics["rate_limited_total"]),
            "worker_rejections_total": int(metrics["worker_rejections_total"]),
            "write_wait_count": int(runtime["write_wait"]["count"]),
            "write_hold_count": int(runtime["write_hold"]["count"]),
            "cleanup_count": int(runtime["cleanup"]["count"]),
            "cleanup_errors": int(runtime["cleanup_errors"]),
            "background_jobs": {
                name: {"finished": int(job["finished"]), "failed": int(job["failed"])}
                for name, job in jobs.items()
            },
        }
    except (KeyError, TypeError, ValueError) as error:
        raise SliceError(f"metrics counters are incomplete: {error}") from error
    return snapshot


def subtract_counters(after, before):
    delta = {}
    for key, value in after.items():
        if key == "background_jobs":
            delta[key] = {
                name: {
                    field: value[name][field] - before[key][name][field]
                    for field in ("finished", "failed")
                }
                for name in value
                if name in before[key]
            }
        else:
            delta[key] = value - before[key]
    return delta


def wait_ready(process, base, hidden):
    deadline = time.monotonic() + 20
    while time.monotonic() < deadline:
        if process.poll() is not None:
            fail("server exited before readiness", hidden)
        _elapsed, status, _parsed, _unparsed = http_exchange("GET", base + "/readyz", timeout=1)
        if status == 200:
            return
        time.sleep(0.05)
    fail("server did not become ready", hidden)


def start_process(args, cwd, env):
    process = subprocess.Popen(
        args,
        cwd=cwd,
        env=env,
        stdin=subprocess.DEVNULL,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE,
        text=True,
        start_new_session=True,
    )
    bucket = []

    def drain():
        try:
            for line in process.stderr:
                bucket.append(line)
                if sum(len(item) for item in bucket) > 8000:
                    del bucket[:-20]
        except (OSError, ValueError):
            return

    thread = threading.Thread(target=drain, name="q09-stderr", daemon=True)
    thread.start()
    process.q09_stderr = bucket
    process.q09_stderr_thread = thread
    return process


def stop_process(process, hidden):
    if process is None:
        return ""
    if process.poll() is None:
        try:
            os.killpg(process.pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            process.wait(timeout=5)
    thread = getattr(process, "q09_stderr_thread", None)
    if thread is not None:
        thread.join(timeout=1)
    if process.stderr is not None:
        process.stderr.close()
    return scrub("".join(getattr(process, "q09_stderr", []))[-800:], hidden)


def absolute_binary(binary):
    """Resolve a supplied executable from the caller's current directory."""
    path = Path(binary).expanduser()
    if not path.is_absolute():
        path = Path.cwd() / path
    path = path.resolve()
    if not path.is_file():
        raise SliceError(f"binary is not a file: {path}")
    return path


def argv_for(binary, args, fixture):
    if fixture:
        return [sys.executable, str(SCRIPT), "--fixture-mode", *args]
    return [str(binary), *args]


def parse_envelope(stdout, stderr, label, hidden):
    try:
        document = json.loads(stdout)
    except json.JSONDecodeError:
        fail(f"{label} did not return JSON: {stderr}", hidden)
    if not isinstance(document, dict) or document.get("ok") is not True or not isinstance(document.get("data"), dict):
        fail(f"{label} did not return an ok data envelope: {stderr}", hidden)
    return document["data"]


def artifact_record(binary, fixture):
    path = SCRIPT if fixture else Path(binary)
    if not path.is_file():
        raise SliceError(f"binary is not a file: {path}")
    return {
        "kind": "fixture" if fixture else "file",
        "path": str(path),
        "sha256": digest(path),
        "bytes": path.stat().st_size,
    }


def free_port():
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        return listener.getsockname()[1]


def require_programs(names):
    programs = {name: shutil.which(name) for name in names}
    missing = sorted(name for name, path in programs.items() if not path)
    if missing:
        raise SliceError(f"missing programs: {', '.join(missing)}")
    return programs


def database_key_text():
    text = base64.urlsafe_b64encode(secrets.token_bytes(32)).decode("ascii").rstrip("=")
    padded = text + ("=" * ((4 - len(text) % 4) % 4))
    if "=" in text or len(base64.urlsafe_b64decode(padded)) != 32:
        raise SliceError("database key encoding is not 32-byte base64url without padding")
    return text


def write_private_text(path, text):
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL
    fd = os.open(path, flags, 0o600)
    with os.fdopen(fd, "w") as handle:
        handle.write(text)
    if path.stat().st_mode & 0o077:
        raise SliceError("private file is readable by the group or others")


def toml_basic(value):
    return '"' + str(value).replace("\\", "\\\\").replace('"', '\\"') + '"'


def openssl_text(args, cwd, hidden):
    code, stdout, tail = command_output(
        ["openssl", *args], cwd=str(cwd), env=os.environ.copy(), timeout=30, hidden=hidden,
    )
    if code != 0:
        raise SliceError(f"openssl {' '.join(args[:2])} failed: {tail}")
    return stdout


def private_ca_context(ca_file):
    context = ssl.create_default_context(cafile=str(ca_file))
    context.check_hostname = True
    context.verify_mode = ssl.CERT_REQUIRED
    return context


def tls_probe(url, context, timeout=5):
    request = urllib.request.Request(url, method="GET")
    try:
        with urllib.request.urlopen(request, timeout=timeout, context=context) as response:
            response.read()
            return response.status
    except urllib.error.HTTPError as error:
        try:
            error.read()
        finally:
            error.close()
        return error.code
    except (urllib.error.URLError, TimeoutError, ssl.SSLError, ConnectionError, OSError):
        return None


def make_server_material(root, hidden):
    require_programs(("openssl",))
    certs = root / "certs"
    certs.mkdir(mode=0o700)
    ca_key = certs / "ca.key"
    ca_crt = certs / "ca.crt"
    server_key = certs / "server.key"
    server_csr = certs / "server.csr"
    server_crt = certs / "server.crt"
    server_ext = certs / "server.ext"
    unrelated_key = certs / "unrelated.key"
    unrelated_crt = certs / "unrelated.crt"
    ca_config = certs / "ca.cnf"
    unrelated_config = certs / "unrelated.cnf"
    for path, name in ((ca_config, "q09-private-ca"), (unrelated_config, "q09-unrelated-ca")):
        path.write_text(
            "[req]\n"
            "distinguished_name=dn\n"
            "x509_extensions=v3_ca\n"
            "prompt=no\n"
            "[dn]\n"
            f"CN={name}\n"
            "[v3_ca]\n"
            "basicConstraints=critical,CA:TRUE\n"
            "keyUsage=critical,keyCertSign,cRLSign\n"
            "subjectKeyIdentifier=hash\n"
        )
    openssl_text(
        ["req", "-x509", "-newkey", "rsa:2048", "-nodes", "-keyout", str(ca_key),
         "-out", str(ca_crt), "-days", "2", "-config", str(ca_config)],
        certs, hidden,
    )
    ca_key.chmod(0o600)
    openssl_text(
        ["req", "-newkey", "rsa:2048", "-nodes", "-keyout", str(server_key),
         "-out", str(server_csr), "-subj", "/CN=localhost"],
        certs, hidden,
    )
    server_key.chmod(0o600)
    server_ext.write_text(
        "basicConstraints=CA:FALSE\n"
        "subjectAltName=DNS:localhost,IP:127.0.0.1\n"
        "extendedKeyUsage=serverAuth\n"
        "keyUsage=digitalSignature,keyEncipherment\n"
        "subjectKeyIdentifier=hash\n"
        "authorityKeyIdentifier=keyid,issuer\n"
    )
    openssl_text(
        ["x509", "-req", "-in", str(server_csr), "-CA", str(ca_crt), "-CAkey", str(ca_key),
         "-CAcreateserial", "-out", str(server_crt), "-days", "2", "-extfile", str(server_ext)],
        certs, hidden,
    )
    verified = openssl_text(["verify", "-CAfile", str(ca_crt), str(server_crt)], certs, hidden)
    if "OK" not in verified.split():
        raise SliceError("private CA did not verify the server certificate")
    san = openssl_text(
        ["x509", "-in", str(server_crt), "-noout", "-ext", "subjectAltName"], certs, hidden,
    )
    if "DNS:localhost" not in san or "127.0.0.1" not in san:
        raise SliceError("server certificate SAN does not match the loopback issuer")
    openssl_text(
        ["req", "-x509", "-newkey", "rsa:2048", "-nodes", "-keyout", str(unrelated_key),
         "-out", str(unrelated_crt), "-days", "2", "-config", str(unrelated_config)],
        certs, hidden,
    )
    unrelated_key.chmod(0o600)
    key_text = database_key_text()
    key_path = certs / "database.key"
    write_private_text(key_path, key_text + "\n")
    for secret_path in (ca_key, server_key, unrelated_key):
        for line in secret_path.read_text().splitlines():
            if len(line) >= 12:
                remember_secret(hidden, line)
    remember_secret(hidden, key_text)
    return {
        "ca": ca_crt,
        "server_cert": server_crt,
        "server_key": server_key,
        "unrelated_ca": unrelated_crt,
        "database_key": key_path,
        "san": ["DNS:localhost", "IP:127.0.0.1"],
    }


def pg_hba_is_hostssl_scram(text):
    lines = []
    for line in text.splitlines():
        stripped = line.strip()
        if stripped and not stripped.startswith("#"):
            lines.append(stripped)
    return lines == [
        "hostssl all all 127.0.0.1/32 scram-sha-256",
        "hostssl all all ::1/128 scram-sha-256",
    ]


def require_connection_file(text, port):
    single = text.strip()
    if "\n" in single:
        raise SliceError("connection file has more than one line")
    parts = single.split()
    required = [
        "host=localhost",
        "hostaddr=127.0.0.1",
        f"port={port}",
        "dbname=riauth_q09",
        "user=riauth_test",
        "sslmode=require",
    ]
    for item in required:
        if item not in parts:
            raise SliceError("connection file is missing a required field")
    for banned in ("sslmode=verify-full", "sslmode=disable", "sslmode=prefer", "sslmode=allow"):
        if banned in single:
            raise SliceError("connection file uses a rejected sslmode")
    if any(part == "trust" or part.startswith("trust=") for part in parts):
        raise SliceError("connection file names trust")
    passwords = [part.split("=", 1)[1] for part in parts if part.startswith("password=")]
    if len(passwords) != 1 or re.fullmatch(r"[0-9a-f]{32,}", passwords[0]) is None:
        raise SliceError("connection credential is not hex")
    return passwords[0]


def libpq_conninfo(port, sslmode, ca_file):
    if sslmode not in ("verify-full", "disable", "require"):
        raise SliceError("libpq preflight sslmode is not one of the checked values")
    return (
        f"host=localhost hostaddr=127.0.0.1 port={port} dbname=riauth_q09 "
        f"user=riauth_test sslmode={sslmode} sslrootcert={ca_file}"
    )


def postgres_env(password):
    env = os.environ.copy()
    env["PGPASSWORD"] = password
    return env


def psql_text(psql, conninfo, password, sql, hidden, cwd):
    code, stdout, tail = command_output(
        [psql, conninfo, "-v", "ON_ERROR_STOP=1", "-tA", "-c", sql],
        cwd=str(cwd), env=postgres_env(password), timeout=30, hidden=hidden,
    )
    return code, stdout.strip(), tail


def attach_https_material(config_path, cert_file, key_file):
    text = config_path.read_text()
    if "tls_cert_file" in text or "tls_key_file" in text:
        raise SliceError("config already names TLS files")
    addition = (
        f"tls_cert_file = {toml_basic(cert_file)}\n"
        f"tls_key_file = {toml_basic(key_file)}\n"
    )
    # Root keys must precede the first table. A line after [postgres] would belong to that table.
    config_path.write_text(addition + text)
    config_path.chmod(0o600)
    if config_path.stat().st_mode & 0o077:
        raise SliceError("config file is readable by the group or others")


def parse_cli_document(stdout, stderr, label, hidden):
    document = None
    for line in reversed(stdout.splitlines()):
        stripped = line.strip()
        if stripped.startswith("{"):
            try:
                document = json.loads(stripped)
            except json.JSONDecodeError:
                document = None
            break
    if not isinstance(document, dict):
        fail(f"{label} did not return JSON: {stderr}", hidden)
    return document


def feature_facts(runtime, names):
    states = runtime.get("feature_states")
    if not isinstance(states, dict):
        raise SliceError("runtime capabilities have no feature_states")
    facts = {}
    for name in names:
        state = states.get(name)
        if not isinstance(state, dict):
            raise SliceError(f"runtime capabilities omit {name}")
        fact = {
            "compiled": state.get("compiled") is True,
            "configured": state.get("configured") is True,
            "usable": state.get("usable") is True,
        }
        if not all(fact.values()):
            raise SliceError(
                f"{name} compiled={fact['compiled']} configured={fact['configured']} usable={fact['usable']}"
            )
        facts[name] = fact
    return facts


def start_postgres(root):
    programs = require_programs(("initdb", "pg_ctl", "createdb"))
    cluster = root / "postgres-cluster"
    port = free_port()
    code, _stdout, tail = command_output(
        [programs["initdb"], "-D", str(cluster), "-U", "riauth_test", "--auth=trust",
         "--encoding=UTF8", "--no-locale"],
        cwd=root, env=os.environ.copy(), timeout=60,
    )
    if code != 0:
        raise SliceError(f"initdb failed: {tail}")
    with (cluster / "postgresql.conf").open("a") as config:
        config.write(
            f"\nlisten_addresses = '127.0.0.1'\nport = {port}\nunix_socket_directories = ''\n"
        )
    started = False
    try:
        code, _stdout, tail = command_output(
            [programs["pg_ctl"], "-D", str(cluster), "-l", str(root / "postgres.log"), "start", "-w"],
            cwd=root, env=os.environ.copy(), timeout=60,
        )
        if code != 0:
            raise SliceError(f"pg_ctl start failed: {tail}")
        started = True
        code, _stdout, tail = command_output(
            [programs["createdb"], "-h", "127.0.0.1", "-p", str(port), "-U", "riauth_test", "riauth_q09"],
            cwd=root, env=os.environ.copy(), timeout=60,
        )
        if code != 0:
            raise SliceError(f"createdb failed: {tail}")
        connection = root / "connection"
        connection.write_text(
            f"host=127.0.0.1 port={port} dbname=riauth_q09 user=riauth_test sslmode=disable\n"
        )
        connection.chmod(0o600)
        config_path = root / "postgres.json"
        config_path.write_text(json.dumps({
            "connection_file": str(connection),
            "ca_file": None,
            "local_unencrypted": True,
            "pool_size": 8,
        }))
        return {"programs": programs, "cluster": cluster, "root": root, "config": config_path, "port": port}
    except Exception:
        if started or (cluster / "postmaster.pid").exists():
            stop_postgres(programs, cluster, root)
        raise


def stop_postgres(programs, cluster, root):
    command_output(
        [programs["pg_ctl"], "-D", str(cluster), "stop", "-m", "immediate", "-w"],
        cwd=root, env=os.environ.copy(), timeout=60,
    )


def start_postgres_verified(root, material, hidden):
    """Disposable PostgreSQL with a private CA. TCP auth is hostssl scram only."""
    programs = require_programs(("initdb", "pg_ctl", "createdb", "psql"))
    cluster = root / "postgres-cluster"
    socket_dir = root / "pg-socket"
    socket_dir.mkdir(mode=0o700)
    if "'" in str(socket_dir):
        raise SliceError("socket directory path is not usable in postgresql.conf")
    port = free_port()
    password = secrets.token_hex(24)
    remember_secret(hidden, password)
    password_file = root / "pg-password"
    write_private_text(password_file, password + "\n")
    code, _stdout, tail = command_output(
        [programs["initdb"], "-D", str(cluster), "-U", "riauth_test",
         "--auth=scram-sha-256", "--pwfile", str(password_file),
         "--encoding=UTF8", "--no-locale"],
        cwd=root, env=os.environ.copy(), timeout=60, hidden=hidden,
    )
    password_file.unlink(missing_ok=True)
    if code != 0:
        raise SliceError(f"initdb failed: {tail}")
    shutil.copyfile(material["server_cert"], cluster / "server.crt")
    shutil.copyfile(material["server_key"], cluster / "server.key")
    (cluster / "server.key").chmod(0o600)
    with (cluster / "postgresql.conf").open("a") as config:
        config.write(
            "\nlisten_addresses = '127.0.0.1'\n"
            f"port = {port}\n"
            f"unix_socket_directories = '{socket_dir}'\n"
            "ssl = on\n"
            "ssl_cert_file = 'server.crt'\n"
            "ssl_key_file = 'server.key'\n"
            "ssl_min_protocol_version = 'TLSv1.2'\n"
            "password_encryption = 'scram-sha-256'\n"
        )
    (cluster / "pg_hba.conf").write_text(
        "local all all scram-sha-256\n" + HOSTSSL_HBA
    )
    started = False
    try:
        code, _stdout, tail = command_output(
            [programs["pg_ctl"], "-D", str(cluster), "-l", str(root / "postgres.log"), "start", "-w"],
            cwd=root, env=os.environ.copy(), timeout=60, hidden=hidden,
        )
        if code != 0:
            raise SliceError(f"pg_ctl start failed: {tail}")
        started = True
        code, _stdout, tail = command_output(
            [programs["createdb"], "-h", str(socket_dir), "-p", str(port),
             "-U", "riauth_test", "riauth_q09"],
            cwd=root, env=postgres_env(password), timeout=60, hidden=hidden,
        )
        if code != 0:
            raise SliceError(f"createdb failed: {tail}")
        (cluster / "pg_hba.conf").write_text(HOSTSSL_HBA)
        if not pg_hba_is_hostssl_scram((cluster / "pg_hba.conf").read_text()):
            raise SliceError("pg_hba is not hostssl scram")
        code, _stdout, tail = command_output(
            [programs["pg_ctl"], "-D", str(cluster), "reload"],
            cwd=root, env=os.environ.copy(), timeout=30, hidden=hidden,
        )
        if code != 0:
            raise SliceError(f"pg_ctl reload failed: {tail}")
        connection = root / "connection"
        write_private_text(connection, (
            f"host=localhost hostaddr=127.0.0.1 port={port} dbname=riauth_q09 "
            f"user=riauth_test password={password} sslmode=require\n"
        ))
        require_connection_file(connection.read_text(), port)
        config_path = root / "postgres.json"
        config_path.write_text(json.dumps({
            "connection_file": str(connection),
            "ca_file": str(material["ca"]),
            "local_unencrypted": False,
            "pool_size": 8,
        }))
        verified = libpq_preflight(
            programs["psql"], port, password, material["ca"], material["unrelated_ca"], hidden, root,
        )
        return {
            "programs": programs,
            "cluster": cluster,
            "root": root,
            "config": config_path,
            "connection": connection,
            "port": port,
            "password": password,
            "preflight": verified,
        }
    except Exception:
        if started or (cluster / "postmaster.pid").exists():
            stop_postgres(programs, cluster, root)
        raise


def libpq_preflight(psql, port, password, ca_file, unrelated_ca, hidden, cwd):
    ok_info = libpq_conninfo(port, "verify-full", ca_file)
    code, stdout, tail = psql_text(psql, ok_info, password, "SELECT current_setting('ssl')", hidden, cwd)
    if code != 0 or stdout != "on":
        raise SliceError(f"libpq verify-full preflight failed: {tail}")
    bad_info = libpq_conninfo(port, "verify-full", unrelated_ca)
    code, _stdout, _tail = psql_text(psql, bad_info, password, "SELECT 1", hidden, cwd)
    if code == 0:
        raise SliceError("libpq accepted an unrelated CA")
    disabled = libpq_conninfo(port, "disable", ca_file)
    code, _stdout, _tail = psql_text(psql, disabled, password, "SELECT 1", hidden, cwd)
    if code == 0:
        raise SliceError("libpq accepted sslmode=disable")
    return {
        "libpq_sslmode_verify_full": True,
        "server_ssl": True,
        "libpq_unrelated_ca_refused": True,
        "libpq_sslmode_disable_refused": True,
    }


def refuse_unrelated_postgres(binary, root, cluster, material, hidden, env):
    unrelated_root = root / "unrelated-instance"
    data = unrelated_root / "data"
    data.mkdir(parents=True)
    config = unrelated_root / "riauth.toml"
    pg_json = unrelated_root / "postgres.json"
    pg_json.write_text(json.dumps({
        "connection_file": str(cluster["connection"]),
        "ca_file": str(material["unrelated_ca"]),
        "local_unencrypted": False,
        "pool_size": 8,
    }))
    port = free_port()
    password = secrets.token_urlsafe(18)
    remember_secret(hidden, password)
    key_text = database_key_text()
    key_path = unrelated_root / "database.key"
    write_private_text(key_path, key_text + "\n")
    remember_secret(hidden, key_text)
    init = [
        "--json", "--config", str(config), "--non-interactive", "init",
        "--issuer", f"https://127.0.0.1:{port}", "--listen", f"127.0.0.1:{port}",
        "--data-dir", str(data), "--admin", "admin", "--password-stdin",
        "--database-key-file", str(key_path),
        "--postgres-config", str(pg_json),
    ]
    code, stdout, tail = command_output(
        [str(binary), *init], cwd=unrelated_root, env=env,
        input_text=password + "\n", timeout=60, hidden=hidden,
    )
    if config.exists():
        raise SliceError("unrelated CA init published a config")
    document = parse_cli_document(stdout, tail, "unrelated CA init", hidden)
    error = document.get("error") if isinstance(document.get("error"), dict) else {}
    result = {
        "exit_code": code,
        "error_code": error.get("code"),
        "http_status": error.get("http_status"),
        "config_published": False,
    }
    if code != 6 or error.get("code") != "storage_unavailable" or error.get("http_status") != 503:
        raise SliceError(
            "unrelated CA init exited "
            f"{code} code={error.get('code')} status={error.get('http_status')}: {tail}"
        )
    if document.get("ok") is not False:
        raise SliceError("unrelated CA init returned ok")
    return result


def read_storage_format(backend, instance, cluster, hidden):
    if backend == "redb":
        path = instance / "data" / "riauth.redb"
        blob = path.read_bytes()
        if blob.count(b"aes256gcm-v1") < 1 or blob.count(b"plain-v1") != 0:
            raise SliceError("redb storage format marker is not aes256gcm-v1")
        assert_secrets_absent(blob, hidden, "redb store")
        return "aes256gcm-v1"
    password = cluster["password"]
    info = libpq_conninfo(cluster["port"], "verify-full", json.loads(cluster["config"].read_text())["ca_file"])
    code, stdout, tail = psql_text(
        cluster["programs"]["psql"], info, password,
        "SELECT encoding FROM riauth_store.storage_format WHERE singleton",
        hidden, cluster["root"],
    )
    if code != 0 or stdout != "aes256gcm-v1":
        raise SliceError(f"postgres storage format is {stdout or 'unread'}: {tail}")
    code, hex_rows, tail = psql_text(
        cluster["programs"]["psql"], info, password,
        "SELECT encode(value, 'hex') FROM riauth_store.records_v1",
        hidden, cluster["root"],
    )
    if code != 0:
        raise SliceError(f"postgres record read failed: {tail}")
    try:
        blob = bytes.fromhex("".join(hex_rows.split()))
    except ValueError as error:
        raise SliceError("postgres record hex is unreadable") from error
    if b"plain-v1" in blob:
        raise SliceError("postgres records include plain-v1")
    assert_secrets_absent(blob, hidden, "postgres records")
    return "aes256gcm-v1"


def measured_pass(
    base, token, username, iterations, warmup, pid, hidden, expected_groups,
    interfere=None, pace_ms=0,
):
    pace = pace_seconds(pace_ms)
    for _ in range(warmup):
        _elapsed, status, parsed, unparsed = http_exchange(
            "GET", base + "/api/me", headers=auth_headers(token), timeout=5,
        )
        ok, code = classify_me(status, parsed, unparsed, username, expected_groups)
        if not ok:
            fail(f"warmup session read failed: {code or status}", hidden)
    _before_elapsed, before_status, before_body, _before_unparsed = http_exchange(
        "GET", base + "/api/operations/metrics", headers=auth_headers(token), timeout=5,
    )
    before = require_document(before_status, before_body, "metrics", hidden)
    sampler = ProcessSampler(pid)
    load_start = list(os.getloadavg()) if hasattr(os, "getloadavg") else None
    sampler.start()
    if interfere is not None:
        interfere["start"].set()
    samples = []
    success = 0
    statuses = {}
    codes = {}
    started = time.perf_counter()
    try:
        for index in range(iterations):
            elapsed, status, parsed, unparsed = http_exchange(
                "GET", base + "/api/me", headers=auth_headers(token), timeout=5,
            )
            samples.append(elapsed)
            ok, code = classify_me(status, parsed, unparsed, username, expected_groups)
            if ok:
                success += 1
            else:
                key = "transport" if status is None else str(status)
                statuses[key] = statuses.get(key, 0) + 1
                if code:
                    codes[code] = codes.get(code, 0) + 1
            if pace and index + 1 < iterations:
                time.sleep(pace)
    finally:
        elapsed_seconds = time.perf_counter() - started
        sampler.stop()
        if interfere is not None:
            interfere["stop"].set()
            interfere["thread"].join(timeout=30)
    load_end = list(os.getloadavg()) if hasattr(os, "getloadavg") else None
    _after_elapsed, after_status, after_body, _unparsed = http_exchange(
        "GET", base + "/api/operations/metrics", headers=auth_headers(token), timeout=5,
    )
    after = require_document(after_status, after_body, "metrics", hidden)
    delta = subtract_counters(counter_snapshot(after), counter_snapshot(before))
    finished = delta["background_jobs"]["maintenance"]["finished"]
    elapsed_rounded = round(elapsed_seconds, 6)
    return {
        "clients": 2 if interfere else 1,
        "warmup": warmup,
        "pace_ms": pace_ms,
        "attempts": iterations,
        "success": success,
        "errors": iterations - success,
        "error_statuses": statuses,
        "error_codes": codes,
        "elapsed_seconds": elapsed_rounded,
        "throughput_success_per_second": round(success / elapsed_seconds, 6) if elapsed_seconds else None,
        "throughput_attempts_per_second": round(iterations / elapsed_seconds, 6) if elapsed_seconds else None,
        "latency_us": distribution(samples),
        "process": sampler.summary(),
        "maintenance_finished_delta": finished,
        "maintenance_cadence_overlap": maintenance_cadence_overlap(finished, elapsed_rounded),
        "server_counter_delta": delta,
        "server_counter_delta_includes": "the closing /api/operations/metrics read and interference requests that completed before it",
        "host_loadavg": {"start": load_start, "end": load_end},
    }


def group_writer(base, token, cap, start, stop, outcome):
    start.wait()
    _elapsed, status, parsed, _unparsed = http_exchange(
        "GET", base + "/api/state/revision", headers=auth_headers(token), timeout=5,
    )
    if status != 200 or not isinstance(parsed, dict) or not isinstance(parsed.get("revision"), int):
        outcome["error"] += 1
        return
    revision = parsed["revision"]
    posts = 0
    refreshes = 0
    while not stop.is_set() and outcome["success"] < cap and posts < cap:
        name = f"q09-bench-{posts + 1:04d}"
        headers = auth_headers(token, {
            "Content-Type": "application/json",
            "Idempotency-Key": f"q09-{posts + 1}-{secrets.token_hex(8)}",
            "If-Match": f'"{revision}"',
        })
        _elapsed, status, _parsed, _unparsed = http_exchange(
            "POST", base + "/api/groups", {"name": name}, headers, timeout=5,
        )
        posts += 1
        if status == 200:
            outcome["success"] += 1
        elif status == 409 and refreshes < cap:
            outcome["conflict"] += 1
            _elapsed, status, parsed, _unparsed = http_exchange(
                "GET", base + "/api/state/revision", headers=auth_headers(token), timeout=5,
            )
            refreshes += 1
            if status == 200 and isinstance(parsed, dict) and isinstance(parsed.get("revision"), int):
                revision = parsed["revision"]
        else:
            outcome["error"] += 1
            if outcome["error"] >= 5:
                break
    outcome["posts"] = posts
    outcome["revision_refreshes"] = refreshes


def expect_status(status, parsed, label, hidden, kind):
    if status != 200 or not isinstance(parsed, kind):
        fail(f"{label} failed with status {status}", hidden)
    return parsed


def current_revision(base, token, hidden):
    _elapsed, status, parsed, _unparsed = http_exchange(
        "GET", base + "/api/state/revision", headers=auth_headers(token), timeout=15,
    )
    document = expect_status(status, parsed, "revision", hidden, dict)
    revision = document.get("revision")
    if not isinstance(revision, int):
        fail("revision response did not include an integer revision", hidden)
    return revision


def bound_write(base, token, method, path, body, revision, hidden):
    """One idempotent mutation. A single 409 refreshes the revision and retries."""
    for attempt in (1, 2):
        headers = auth_headers(token, {
            "Idempotency-Key": f"q09-dir-{secrets.token_hex(8)}",
            "If-Match": f'"{revision}"',
        })
        if body is not None:
            headers["Content-Type"] = "application/json"
        _elapsed, status, parsed, _unparsed = http_exchange(
            method, base + path, body, headers, timeout=60,
        )
        if status == 200 and isinstance(parsed, dict):
            return revision + 1, parsed
        if status == 409 and attempt == 1:
            revision = current_revision(base, token, hidden)
            continue
        fail(f"{method} {path} failed with status {status}", hidden)
    fail(f"{method} {path} failed after a revision refresh", hidden)


def dataset_shell(plan, verification):
    return {
        "name": dataset_name(plan),
        "kind": "fresh-init" if plan["writes"] == 0 else "session-read-directory",
        "bootstrap_administrator": "admin",
        "extra_users": plan["extra_users"],
        "extra_usernames": list(plan["extra_usernames"]),
        "groups": list(plan["groups"]),
        "memberships": list(plan["memberships"]),
        "session_read_groups": list(plan["groups"]),
        "session_read_index": "index_user_groups for the bootstrap administrator",
        "index_page_rows": INDEX_PAGE_ROWS,
        "within_one_index_page": len(plan["groups"]) <= INDEX_PAGE_ROWS,
        "verified": True,
        "verification": verification,
    }


def fresh_dataset():
    return dataset_shell(
        directory_plan(0, 0),
        "no directory requests; dataset is the bootstrap administrator",
    )


def install_directory(base, token, plan, hidden):
    """Create the directory and require the session read to show it."""
    _elapsed, status, parsed, _unparsed = http_exchange(
        "GET", base + "/api/me", headers=auth_headers(token), timeout=15,
    )
    me = expect_status(status, parsed, "administrator session", hidden, dict)
    user = me.get("user")
    if not isinstance(user, dict) or user.get("username") != "admin" or not isinstance(user.get("id"), str) or not user["id"]:
        fail("session read did not identify the administrator", hidden)
    admin_id = user["id"]
    if me.get("groups") not in ([], None):
        fail("administrator already has group memberships before directory setup", hidden)
    revision = current_revision(base, token, hidden)
    ids = {"admin": admin_id}
    for username in plan["extra_usernames"]:
        password = secrets.token_urlsafe(18)
        hidden.append(password)
        revision, created = bound_write(
            base, token, "POST", "/api/users",
            {"username": username, "password": password, "admin": False},
            revision, hidden,
        )
        if created.get("username") != username or not isinstance(created.get("id"), str) or not created["id"]:
            fail("user create response did not identify the user", hidden)
        ids[username] = created["id"]
    for name in plan["groups"]:
        revision, created = bound_write(
            base, token, "POST", "/api/groups", {"name": name}, revision, hidden,
        )
        if created.get("name") != name:
            fail(f"group create response did not name {name}", hidden)
    expected_members = {name: set() for name in plan["groups"]}
    for item in plan["memberships"]:
        username = item["username"]
        group = item["group"]
        revision, created = bound_write(
            base, token, "PUT",
            f"/api/groups/{group}/members/{username}",
            None, revision, hidden,
        )
        members = created.get("members")
        if not isinstance(members, list) or ids[username] not in members:
            fail(f"membership response for {username} in {group} did not contain that user", hidden)
        expected_members[group].add(ids[username])
    _elapsed, status, parsed, _unparsed = http_exchange(
        "GET", base + "/api/me", headers=auth_headers(token), timeout=15,
    )
    observed = expect_status(status, parsed, "directory session read", hidden, dict)
    found = observed.get("groups")
    if not isinstance(found, list) or sorted(found) != sorted(plan["groups"]):
        fail("session read groups do not match the directory", hidden)
    if not isinstance(observed.get("user"), dict) or observed["user"].get("id") != admin_id:
        fail("directory session read lost the administrator id", hidden)
    _elapsed, status, parsed, _unparsed = http_exchange(
        "GET", base + "/api/users", headers=auth_headers(token), timeout=15,
    )
    listed_users = expect_status(status, parsed, "user list", hidden, list)
    usernames = []
    for entry in listed_users:
        if not isinstance(entry, dict) or not isinstance(entry.get("username"), str):
            fail("user list entry did not include a username", hidden)
        usernames.append(entry["username"])
    if sorted(usernames) != sorted(["admin", *plan["extra_usernames"]]):
        fail("user list does not match the directory", hidden)
    _elapsed, status, parsed, _unparsed = http_exchange(
        "GET", base + "/api/groups", headers=auth_headers(token), timeout=15,
    )
    listed_groups = expect_status(status, parsed, "group list", hidden, list)
    by_name = {}
    for entry in listed_groups:
        if not isinstance(entry, dict) or not isinstance(entry.get("name"), str) or not isinstance(entry.get("members"), list):
            fail("group list entry did not include members", hidden)
        by_name[entry["name"]] = set(entry["members"])
    if set(by_name) != set(plan["groups"]):
        fail("group list does not match the directory", hidden)
    member_counts = {}
    for name, expected in expected_members.items():
        if by_name[name] != expected:
            fail(f"members of {name} do not match the directory", hidden)
        member_counts[name] = len(expected)
    return dataset_shell(plan, {
        "method": "GET /api/me, GET /api/users, and GET /api/groups before measurement",
        "session_groups": sorted(found),
        "usernames": sorted(usernames),
        "member_counts": member_counts,
    })


def latency_delta(interference, quiet):
    delta = {}
    for key in ("min_us", "p50_us", "p95_us", "p99_us", "max_us"):
        left = quiet["latency_us"].get(key)
        right = interference["latency_us"].get(key)
        delta[key] = None if left is None or right is None else right - left
    return delta


def run_slice(
    binary, backend, iterations, warmup, interference_cap,
    fixture=False, directory_users=0, directory_groups=0,
    build_profile="unrecorded", build_toolchain="unrecorded", build_features="unrecorded",
    pace_ms=0, secure=False,
):
    if secure and fixture:
        raise SliceError("--secure does not use the fixture server")
    if backend not in ("redb", "postgresql"):
        raise SliceError("backend must be redb or postgresql")
    pace_seconds(pace_ms)
    budget = validate_shape(
        iterations, warmup, interference_cap, directory_users, directory_groups,
    )
    plan = directory_plan(directory_users, directory_groups)
    build = build_note(build_profile, build_toolchain, build_features)
    binary = absolute_binary(binary)
    hidden = []
    source = source_identity()
    host = hardware()
    artifact = artifact_record(binary, fixture)
    env = os.environ.copy()
    if fixture:
        env["Q09_FIXTURE_ME_DELAY"] = "0.02"
    code, stdout, tail = command_output(
        argv_for(binary, ["--json", "capabilities"], fixture),
        cwd=ROOT, env=env, timeout=30, hidden=hidden,
    )
    if code != 0:
        fail(f"capabilities exited {code}: {tail}", hidden)
    capabilities = parse_envelope(stdout, tail, "capabilities", hidden)
    if not fixture and capabilities.get("schema_version") != "riauth.capabilities/v2":
        raise SliceError("binary capabilities schema is not riauth.capabilities/v2")
    product_run = capabilities.get("schema_version") == "riauth.capabilities/v2"
    username = "admin"
    root = Path(tempfile.mkdtemp(prefix="riauth-q09-"))
    server = None
    cluster = None
    material = None
    tls_reset = None
    security = None
    try:
        if secure:
            material = make_server_material(root, hidden)
        if backend == "postgresql":
            cluster = start_postgres_verified(root, material, hidden) if secure else start_postgres(root)
        port = free_port()
        base = f"{'https' if secure else 'http'}://127.0.0.1:{port}"
        instance = root / "instance"
        instance.mkdir()
        config = instance / "riauth.toml"
        password = secrets.token_urlsafe(18)
        hidden.append(password)
        refusal = None
        if secure and cluster is not None:
            refusal = refuse_unrelated_postgres(binary, root, cluster, material, hidden, env)
        init = [
            "--json", "--config", str(config), "--non-interactive", "init",
            "--issuer", base, "--listen", f"127.0.0.1:{port}",
            "--data-dir", str(instance / "data"), "--admin", username, "--password-stdin",
        ]
        if secure:
            init.extend(["--database-key-file", str(material["database_key"])])
        if cluster is not None:
            init.extend(["--postgres-config", str(cluster["config"])])
        code, stdout, tail = command_output(
            argv_for(binary, init, fixture),
            cwd=instance, env=env, input_text=password + "\n", timeout=120, hidden=hidden,
        )
        if code != 0:
            fail(f"init exited {code}: {tail}", hidden)
        parse_envelope(stdout, tail, "init", hidden)
        if secure:
            storage_format = read_storage_format(backend, instance, cluster, hidden)
            attach_https_material(config, material["server_cert"], material["server_key"])
            assert_secrets_absent(config.read_bytes(), hidden, "config")
            if (material["database_key"].stat().st_mode & 0o777) != 0o600:
                raise SliceError("database key mode is not 0600")
            tls_reset = TLS_CONTEXT.set(private_ca_context(material["ca"]))
        server = start_process(argv_for(binary, ["--config", str(config), "serve"], fixture), instance, env)
        try:
            wait_ready(server, base, hidden)
        except SliceError:
            fail(f"server failed: {stop_process(server, hidden)}", hidden)
        if secure:
            matching = tls_probe(base + "/readyz", TLS_CONTEXT.get())
            unrelated = tls_probe(base + "/readyz", private_ca_context(material["unrelated_ca"]))
            system_trust = tls_probe(base + "/readyz", ssl.create_default_context())
            if matching != 200 or unrelated is not None or system_trust is not None:
                raise SliceError(
                    f"https trust probe matching={matching} unrelated={unrelated} system={system_trust}"
                )
        _elapsed, status, parsed, _unparsed = http_exchange(
            "POST", base + "/api/login",
            {"username": username, "password": password},
            {"Content-Type": "application/json"},
            timeout=30,
        )
        login = require_document(status, parsed, "login", hidden)
        token = login.get("session_token")
        if not isinstance(token, str) or not token:
            fail("login response did not include a session token", hidden)
        hidden.append(token)
        _elapsed, status, parsed, _unparsed = http_exchange("GET", base + "/api/capabilities", timeout=5)
        runtime = require_document(status, parsed, "runtime capabilities", hidden)
        if runtime.get("edition") != capabilities.get("edition"):
            fail("runtime edition does not match the binary capabilities", hidden)
        if runtime.get("storage_backend") != backend:
            fail(f"runtime storage_backend is {runtime.get('storage_backend')}", hidden)
        if runtime.get("scope") != "instance":
            fail("runtime capabilities scope is not instance", hidden)
        if secure:
            names = ["operations.native_tls", "operations.encrypted_storage"]
            if backend == "postgresql":
                names.append("operations.postgresql")
            security = {
                "https": {
                    "matching_ca_readyz": True,
                    "unrelated_ca_refused": True,
                    "system_trust_refused": True,
                    "san": list(material["san"]),
                    "hostname_check": True,
                    "client_ca": "fresh private CA",
                },
                "database_key_file": {
                    "present": True,
                    "decoded_bytes": 32,
                    "encoding": "base64url-nopad",
                    "file_mode": "0600",
                },
                "storage_format": storage_format,
                "credential_bytes_in_store": False,
                "features": feature_facts(runtime, names),
            }
            if cluster is not None:
                security["postgres"] = {
                    "local_unencrypted": False,
                    "auth": "scram-sha-256",
                    "hba": "hostssl 127.0.0.1/32 and ::1/128",
                    "product_connection_sslmode": "require",
                    "product_parser": "disable, prefer, and require; SslMode::Require; rustls hostname and ca_file",
                    "product_unrelated_ca_init": refusal,
                }
                security["postgres"].update(cluster["preflight"])
        if plan["writes"]:
            dataset = install_directory(base, token, plan, hidden)
        else:
            dataset = fresh_dataset()
        expected_groups = dataset["session_read_groups"]
        quiet = measured_pass(
            base, token, username, iterations, warmup, server.pid, hidden, expected_groups,
            pace_ms=pace_ms,
        )
        outcome = {"success": 0, "conflict": 0, "error": 0, "posts": 0, "revision_refreshes": 0}
        start = threading.Event()
        stop = threading.Event()
        writer_context = contextvars.copy_context()
        writer = threading.Thread(
            target=writer_context.run,
            args=(group_writer, base, token, interference_cap, start, stop, outcome),
            name="q09-groups",
            daemon=True,
        )
        writer.start()
        interference = measured_pass(
            base, token, username, iterations, warmup, server.pid, hidden, expected_groups,
            interfere={"start": start, "stop": stop, "thread": writer},
            pace_ms=pace_ms,
        )
        report = {
            "schema": SCHEMA,
            "observations_only": True,
            "performance_claim": False,
            "product_run": product_run,
            "operation": {
                "name": "session_read",
                "method": "GET",
                "route": "/api/me",
                "core": "Core::me",
                "success": "HTTP 200, user.username equals the bootstrap administrator, and groups equal the recorded session-read index",
            },
            "dataset": dataset,
            "protocol": {
                "script": "scripts/q09_benchmark_slice.py",
                "sha256": digest(SCRIPT),
            },
            "source": source,
            "build": build,
            "artifact": artifact,
            "capabilities": {
                "schema_version": capabilities.get("schema_version"),
                "edition": capabilities.get("edition"),
                "build_features": capabilities.get("build_features"),
                "version": capabilities.get("version"),
                "target": capabilities.get("target"),
            },
            "runtime": {
                "edition": runtime.get("edition"),
                "build_features": runtime.get("build_features"),
                "storage_backend": runtime.get("storage_backend"),
                "scope": runtime.get("scope"),
            },
            "hardware": host,
            "settings": {
                "backend": backend,
                "iterations": iterations,
                "warmup": warmup,
                "interference_cap": interference_cap,
                "pace_ms": pace_ms,
                "maintenance_cadence_seconds": MAINTENANCE_CADENCE_SECONDS,
                "directory_users": directory_users,
                "directory_groups": directory_groups,
                "username": username,
                "database_encryption": bool(secure),
                "tls": bool(secure),
                "issuer": "loopback https base" if secure else "loopback http base",
                "listen": "127.0.0.1 ephemeral",
                "rate_limits": "process defaults",
                "postgres_local_unencrypted": False if secure else cluster is not None,
                "postgres_pool_size": 8 if cluster is not None else None,
                "budget": budget,
            },
            "passes": {"quiet": quiet, "interference": interference},
            "interference": {
                "kind": "concurrent POST /api/groups",
                "memberships": "empty",
                "writer": outcome,
                "overlap": outcome["success"] > 0,
                "interference_minus_quiet_us": latency_delta(interference, quiet),
            },
            "limitations": secure_limitations(host, backend) if secure else list(LIMITATIONS),
        }
        if security is not None:
            report["security"] = security
        ensure_redacted(report, hidden)
        return report
    finally:
        if tls_reset is not None:
            TLS_CONTEXT.reset(tls_reset)
        stderr = stop_process(server, hidden)
        if cluster is not None:
            stop_postgres(cluster["programs"], cluster["cluster"], cluster["root"])
        shutil.rmtree(root, ignore_errors=True)
        del stderr


def fixture_main(argv):
    if "capabilities" in argv:
        print(json.dumps({
            "schema_version": "riauth.cli/v1",
            "ok": True,
            "data": {
                "schema_version": "q09.fixture/v1",
                "edition": "essentials",
                "build_features": ["essentials"],
                "version": "fixture",
                "target": {"os": "fixture", "arch": "fixture"},
                "interface": "server",
            },
        }))
        return 0
    if "init" in argv:
        values = {}
        cursor = 0
        while cursor < len(argv) - 1:
            if argv[cursor].startswith("--") and not argv[cursor + 1].startswith("--"):
                values[argv[cursor][2:]] = argv[cursor + 1]
            cursor += 1
        Path(values["config"]).write_text(f"listen={values['listen']}\n")
        sys.stdin.read()
        print(json.dumps({"schema_version": "riauth.cli/v1", "ok": True, "data": {"initialized": True}}))
        return 0
    if "serve" in argv:
        config = Path(argv[argv.index("--config") + 1])
        listen = dict(line.split("=", 1) for line in config.read_text().splitlines() if "=" in line)["listen"]
        host, port = listen.rsplit(":", 1)
        serve_fixture(host, int(port))
        return 0
    raise SliceError("fixture command is not capabilities, init, or serve")


def serve_fixture(host, port):
    from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

    state = {
        "requests": 0,
        "groups": 0,
        "metrics_calls": 0,
        "revision": 1,
        "admin_id": "admin-id",
        "accounts": {"admin": "admin-id"},
        "directory": {},
        "lock": threading.Lock(),
    }
    delay = float(os.environ.get("Q09_FIXTURE_ME_DELAY", "0.02"))
    token = "q09-fixture-token"

    class Handler(BaseHTTPRequestHandler):
        protocol_version = "HTTP/1.1"

        def log_message(self, fmt, *args):
            return

        def body(self):
            length = int(self.headers.get("Content-Length", "0") or 0)
            if not length:
                return b""
            return self.rfile.read(length)

        def send_json(self, status, payload):
            raw = json.dumps(payload).encode()
            self.send_response(status)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(raw)))
            self.end_headers()
            self.wfile.write(raw)

        def authorized(self):
            return self.headers.get("Authorization") == f"Bearer {token}"

        def json_body(self, raw):
            if not raw:
                return {}
            try:
                document = json.loads(raw.decode())
            except (UnicodeDecodeError, json.JSONDecodeError):
                return None
            return document if isinstance(document, dict) else None

        def commit_mutation(self, apply):
            if not self.authorized():
                self.send_json(401, {"error": "unauthorized", "error_description": "unauthorized"})
                return
            if not self.headers.get("Idempotency-Key") or self.headers.get("If-Match") is None:
                self.send_json(428, {
                    "error": "precondition_required",
                    "error_description": "Idempotency-Key and If-Match required",
                })
                return
            with state["lock"]:
                if self.headers.get("If-Match") != f'"{state["revision"]}"':
                    status, payload = 409, {"error": "conflict", "error_description": "revision changed"}
                else:
                    status, payload = apply()
                    if status == 200:
                        state["revision"] += 1
            self.send_json(status, payload)

        def do_GET(self):
            path = self.path.split("?", 1)[0]
            with state["lock"]:
                state["requests"] += 1
                revision = state["revision"]
                session_groups = sorted(
                    name for name, members in state["directory"].items() if state["admin_id"] in members
                )
                users = [
                    {"id": user_id, "username": username}
                    for username, user_id in sorted(state["accounts"].items())
                ]
                groups = [
                    {"name": name, "members": sorted(members)}
                    for name, members in sorted(state["directory"].items())
                ]
                if path == "/api/operations/metrics":
                    state["metrics_calls"] += 1
                    metrics_calls = state["metrics_calls"]
                    group_count = state["groups"]
                    request_count = state["requests"]
            if path == "/readyz":
                self.send_json(200, {"ok": True})
                return
            if path == "/api/capabilities":
                self.send_json(200, {
                    "scope": "instance",
                    "edition": "essentials",
                    "storage_backend": "redb",
                    "build_features": ["essentials"],
                })
                return
            if not self.authorized():
                self.send_json(401, {"error": "unauthorized", "error_description": "unauthorized"})
                return
            if path == "/api/me":
                time.sleep(delay)
                self.send_json(200, {
                    "user": {"id": state["admin_id"], "username": "admin"},
                    "groups": session_groups,
                })
                return
            if path == "/api/state/revision":
                self.send_json(200, {"revision": revision})
                return
            if path == "/api/users":
                self.send_json(200, users)
                return
            if path == "/api/groups":
                self.send_json(200, groups)
                return
            if path == "/api/operations/metrics":
                jobs = {
                    name: {"finished": 0, "failed": 0, "active": 0}
                    for name in BACKGROUND_JOBS
                }
                jobs["logout_ssf"]["finished"] = metrics_calls
                jobs["maintenance"]["finished"] = metrics_calls
                self.send_json(200, {
                    "requests_total": request_count,
                    "responses_error_total": 0,
                    "rate_limited_total": 0,
                    "worker_rejections_total": 0,
                    "runtime": {
                        "write_wait": {"count": group_count, "seconds": 0},
                        "write_hold": {"count": group_count, "seconds": 0},
                        "cleanup": {"count": 0, "seconds": 0},
                        "cleanup_errors": 0,
                        "background": {"jobs": jobs},
                    },
                })
                return
            self.send_json(404, {"error": "not_found", "error_description": "not found"})

        def do_POST(self):
            raw = self.body()
            path = self.path.split("?", 1)[0]
            with state["lock"]:
                state["requests"] += 1
            if path == "/api/login":
                self.send_json(200, {"session_token": token})
                return
            if path == "/api/users":
                document = self.json_body(raw)
                username = None if document is None else document.get("username")
                if not isinstance(username, str) or not username:
                    self.send_json(400, {"error": "invalid_request", "error_description": "username required"})
                    return

                def apply(username=username):
                    if username in state["accounts"]:
                        return 409, {"error": "conflict", "error_description": "user exists"}
                    user_id = f"id-{username}"
                    state["accounts"][username] = user_id
                    return 200, {"id": user_id, "username": username}

                self.commit_mutation(apply)
                return
            if path == "/api/groups":
                document = self.json_body(raw)
                name = None if document is None else document.get("name")
                if not isinstance(name, str) or not name:
                    self.send_json(400, {"error": "invalid_request", "error_description": "name required"})
                    return

                def apply(name=name):
                    if name in state["directory"]:
                        return 409, {"error": "conflict", "error_description": "group exists"}
                    state["directory"][name] = set()
                    state["groups"] += 1
                    return 200, {"name": name, "members": []}

                self.commit_mutation(apply)
                return
            self.send_json(401, {"error": "unauthorized", "error_description": "unauthorized"})

        def do_PUT(self):
            self.body()
            path = self.path.split("?", 1)[0]
            parts = path.split("/")
            with state["lock"]:
                state["requests"] += 1
            if len(parts) != 6 or parts[1:3] != ["api", "groups"] or parts[4] != "members":
                self.send_json(404, {"error": "not_found", "error_description": "not found"})
                return
            group, username = parts[3], parts[5]

            def apply(group=group, username=username):
                if group not in state["directory"] or username not in state["accounts"]:
                    return 404, {"error": "not_found", "error_description": "not found"}
                state["directory"][group].add(state["accounts"][username])
                return 200, {"name": group, "members": sorted(state["directory"][group])}

            self.commit_mutation(apply)

    server = ThreadingHTTPServer((host, port), Handler)
    server.serve_forever()


def main(argv=None):
    argv = list(sys.argv[1:] if argv is None else argv)
    if argv and argv[0] == "--fixture-mode":
        return fixture_main(argv[1:])
    parser = argparse.ArgumentParser(description="Measure one local riAuth session-read slice.")
    parser.add_argument("--self-check", action="store_true")
    parser.add_argument("--binary")
    parser.add_argument("--backend", choices=("redb", "postgresql"), default="redb")
    parser.add_argument("--iterations", type=int, default=80)
    parser.add_argument("--warmup", type=int, default=5)
    parser.add_argument("--interference-cap", type=int, default=30)
    parser.add_argument("--directory-users", type=int, default=0)
    parser.add_argument("--directory-groups", type=int, default=0)
    parser.add_argument("--pace-ms", type=int, default=0)
    parser.add_argument("--secure", action="store_true")
    parser.add_argument("--build-profile", default="unrecorded")
    parser.add_argument("--build-toolchain", default="unrecorded")
    parser.add_argument("--build-features", default="unrecorded")
    parser.add_argument("--out")
    args = parser.parse_args(argv)
    if args.self_check:
        if (
            args.binary or args.out or args.backend != "redb"
            or args.directory_users or args.directory_groups
            or args.build_profile != "unrecorded" or args.build_toolchain != "unrecorded"
            or args.build_features != "unrecorded" or args.pace_ms or args.secure
        ):
            raise SliceError("--self-check takes no binary, backend, directory size, pace, build note, secure mode, or output file")
        report = run_slice(sys.executable, "redb", 12, 1, 4, fixture=True)
        code = completion_code(report)
        if code != 0 or report["product_run"] or not report["interference"]["overlap"]:
            raise SliceError(f"self-check failed with completion code {code}")
        if (
            report["dataset"]["kind"] != "fresh-init"
            or report["dataset"]["name"] != "fresh-init"
            or report["dataset"]["extra_users"] != 0
            or report["build"] != {"profile": "unrecorded", "toolchain": "unrecorded", "features": "unrecorded"}
        ):
            raise SliceError("self-check baseline dataset was not a fresh init")
        directory = run_slice(
            sys.executable, "redb", 4, 0, 1, fixture=True, directory_users=2, directory_groups=2,
        )
        directory_code = completion_code(directory)
        observed = directory["dataset"]
        counts = observed["verification"]["member_counts"]
        if (
            directory_code != 0
            or directory["product_run"]
            or not directory["interference"]["overlap"]
            or observed["kind"] != "session-read-directory"
            or observed["name"] != "q09-2u-2g"
            or not observed["verified"]
            or observed["extra_users"] != 2
            or observed["session_read_groups"] != ["q09-dir-0001", "q09-dir-0002"]
            or len(observed["memberships"]) != 6
            or counts != {"q09-dir-0001": 3, "q09-dir-0002": 3}
            or directory["passes"]["quiet"]["success"] != 4
            or directory["passes"]["interference"]["success"] != 4
            or directory["passes"]["quiet"]["pace_ms"] != 0
            or directory["passes"]["quiet"]["maintenance_cadence_overlap"]
        ):
            raise SliceError(f"self-check directory failed with completion code {directory_code}")
        paced = run_slice(
            sys.executable, "redb", 4, 0, 1, fixture=True, pace_ms=40,
        )
        paced_code = completion_code(paced)
        quiet_paced = paced["passes"]["quiet"]
        interference_paced = paced["passes"]["interference"]
        if (
            paced_code != 0
            or paced["product_run"]
            or not paced["interference"]["overlap"]
            or paced["settings"]["pace_ms"] != 40
            or quiet_paced["pace_ms"] != 40
            or quiet_paced["success"] != 4
            or quiet_paced["elapsed_seconds"] < 0.10
            or quiet_paced["maintenance_finished_delta"] < 1
            or quiet_paced["maintenance_cadence_overlap"]
            or interference_paced["elapsed_seconds"] < 0.10
            or interference_paced["maintenance_finished_delta"] < 1
            or interference_paced["maintenance_cadence_overlap"]
        ):
            raise SliceError(f"self-check pace failed with completion code {paced_code}")
        print(
            "q09 benchmark slice self-check passed product_run=false overlap=true "
            "directory_verified=true pace_stretched=true maintenance_delta=true "
            "cadence_overlap=false"
        )
        return 0
    if not args.binary:
        raise SliceError("--binary is required unless --self-check is set")
    report = run_slice(
        args.binary, args.backend, args.iterations, args.warmup, args.interference_cap,
        directory_users=args.directory_users, directory_groups=args.directory_groups,
        build_profile=args.build_profile, build_toolchain=args.build_toolchain,
        build_features=args.build_features, pace_ms=args.pace_ms, secure=args.secure,
    )
    code = completion_code(report)
    encoded = json.dumps(report, indent=2, sort_keys=True) + "\n"
    assert_report_text_clean(encoded)
    if args.out:
        destination = Path(args.out)
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(encoded)
    sys.stdout.write(encoded)
    print(
        f"q09 benchmark slice completion={code} product_run={str(report['product_run']).lower()} "
        f"backend={report['settings']['backend']} edition={report['runtime']['edition']} "
        f"dataset={report['dataset']['name']} "
        f"directory_users={report['dataset']['extra_users']} "
        f"directory_groups={len(report['dataset']['groups'])} "
        f"verified={str(report['dataset']['verified']).lower()} "
        f"pace_ms={report['settings']['pace_ms']} "
        f"secure={str(report['settings']['tls']).lower()} "
        f"quiet_maintenance_overlap={str(report['passes']['quiet']['maintenance_cadence_overlap']).lower()} "
        f"interference_maintenance_overlap={str(report['passes']['interference']['maintenance_cadence_overlap']).lower()}",
        file=sys.stderr,
    )
    return code


if __name__ == "__main__":
    try:
        sys.exit(main())
    except SliceError as error:
        print(f"q09 benchmark slice failed: {error}", file=sys.stderr)
        sys.exit(1)
