#!/usr/bin/env python3
"""ONE manual native LOCAL container cohort, not a release or full shared gate.

No imports from other gate helpers. All product calls use existing public CLI
or HTTP contracts. Secrets and application replies stay in bounded memory.
The workflow requires separate root review and runtime release before dispatch.
"""

import argparse
import datetime
import gzip
import hashlib
import json
import os
import pathlib
import platform
import re
import selectors
import shutil
import signal
import socket
import stat
import subprocess
import tarfile
import threading
import time
import tomllib
import urllib.error
import urllib.parse
import urllib.request
import uuid
import zipfile


PROJECT = "891e7443-8dac-4c1b-897f-9e53cb59c7ee"
REPOSITORY = "Rhein-Industries/riAuth"
SOURCE = "b619fe25269ccc150e473bbcde47cdb3623ef810"
TREE = "a627df2ce21a4d255b8c1914d4f543e32f40f4de"
REVIEW = "66c814a339665e6b3f8e6c22bc59d4f6f0aa224c"
NATIVE_RECEIPT = "docs/roadmap/evidence/wave30-a09-native-x86-root-review.json"
BUILDKIT_RECEIPT = "docs/roadmap/evidence/wave30-container-buildkit-source-pin.json"
RECEIPT_HASHES = {
    NATIVE_RECEIPT: "ccf7c3fb6c0bd85816097c32a24a1c1b0a67b9597346d571cd68975a4680b1af",
    BUILDKIT_RECEIPT: "4d263888b2bdc5a2eb922b29c85535e8321dc2cba7c246b5b6b355cef57e57b0",
}
BUILDKIT = "docker.io/moby/buildkit@sha256:98cc6a3fc46220d00f8224ae483f3274fc874e9be8d7dd1e2e2c5481209228b5"
ARM_BUILDKIT_RECEIPT = "docs/roadmap/evidence/wave30-container-buildkit-arm64-source-pin.json"
PLATFORMS = {
    "x86_64": {
        "architecture": "x86_64", "machine": "x86_64", "runner_arch": "X64",
        "daemon_arch": "x86_64", "oci_arch": "amd64", "platform": "linux/amd64",
        "elf_machine": 62, "target": "x86_64-unknown-linux-gnu",
        "source": SOURCE, "tree": TREE, "review": REVIEW,
        "native_receipt": NATIVE_RECEIPT, "receipt_hashes": RECEIPT_HASHES,
        "native_schema": "riauth.wave30.native-x86-root-review/v1",
        "run": 37046857550, "job": 110970324302, "artifact": 11246575279,
        "buildkit": BUILDKIT, "buildkit_receipt": BUILDKIT_RECEIPT,
        "tool_manifest": "linux_amd64_manifest", "cohort_limit": "native x86 LOCAL cohort only",
    },
    "arm64": {
        "architecture": "arm64", "machine": "aarch64", "runner_arch": "ARM64",
        "daemon_arch": "aarch64", "oci_arch": "arm64", "platform": "linux/arm64",
        "elf_machine": 183, "target": "aarch64-unknown-linux-gnu",
        "source": "9a819317efb3a13fa27cd86f884be2be00898fc0",
        "tree": "1528b61ba463d9262d6252d54174748a176f313b",
        "review": "b71b7b0041a549793233e8c7a81bbb61797e20f3",
        "native_receipt": "docs/roadmap/evidence/wave30-a09-native-arm64-37016520583.json",
        "receipt_hashes": {
            "docs/roadmap/evidence/wave30-a09-native-arm64-37016520583.json":
                "2cbf8ee46dbdd8b2ab43ad933913dd0a16c20b3183497c2d3b9cf3d1287da227",
            BUILDKIT_RECEIPT: "4d263888b2bdc5a2eb922b29c85535e8321dc2cba7c246b5b6b355cef57e57b0",
            ARM_BUILDKIT_RECEIPT: "e234b809df9212785c818ace0a287eb356de0b9d5ca8560aab0818e5111d0de2",
        },
        "native_schema": "riauth.root-reviewed-artifact-receipt/v1",
        "run": 37016520583, "job": 110868629053, "artifact": 11232871527,
        "buildkit": "docker.io/moby/buildkit@sha256:3ad6bb9bc8c78c0069d03247adb9a59b3b43d68e55353e876e558b888c6c1768",
        "buildkit_receipt": ARM_BUILDKIT_RECEIPT,
        "tool_manifest": "linux_arm64_manifest", "cohort_limit": "native ARM64 LOCAL cohort only",
    },
}
DOCKERFILE_HASH = "458ebb247170c6d4af2ff45b22e5a36e0a5380612140a43448d2ddcebc5d83cb"
WORKFLOW = ".github/workflows/check-local-container-cohort.yml"
OWNER_LABEL = "org.riauth.local.owner"
GiB = 1024**3
NATIVE_CAP = 512 * 1024**2
IMAGE_CAP = 2 * GiB
LOG_CAP = 8 * 1024**2
RATE_DEFAULTS = {"portal_start": 10, "portal_approve": 20, "login": 20,
                 "passkey": 30, "account": 10, "source_start": 30,
                 "source_callback": 30, "saml": 30, "mfa": 10,
                 "device_start": 30, "device_verify": 20,
                 "browser_decision": 60, "browser_state": 1200,
                 "forward_auth": 6000, "outpost_start": 30, "general": 600}
USER_FIELDS = {"id", "username", "email", "display_name", "enabled", "admin",
               "mfa_enabled", "password_available", "created_at", "attributes",
               "email_verified", "subjects"}


class Refusal(Exception):
    """Only fixed public phase/reason identifiers, never external output."""


def require(condition, code):
    if not condition:
        raise Refusal(code)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def file_hash(path):
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def private_write(path, data):
    with path.open("xb") as out:
        os.chmod(path, 0o600)
        out.write(data)


def json_bytes(data):
    return (json.dumps(data, sort_keys=True, indent=2) + "\n").encode()


def closed_platform(architecture):
    require(architecture in PLATFORMS, "unsupported_native_architecture")
    return PLATFORMS[architecture]


def native_receipt_view(record, selected):
    if selected["architecture"] == "x86_64":
        return record
    require(record["schema"] == selected["native_schema"]
            and record["runner"]["label"] == "ubuntu-24.04-arm"
            and record["runner"]["system"] == "Linux"
            and record["runner"]["machine"] == selected["machine"]
            and record["runner"]["arch"] == selected["runner_arch"]
            and record["runner"]["environment"] == "github-hosted"
            and record["native_archive_slice"] == "passed"
            and record["official_release"] is False
            and record["root_local_artifact_execution"] is False
            and all(p["target"] == selected["target"] for p in record["products"]),
            "root_arm_native_scope")
    # Only adapt fixed root metadata; preserve products, inputs, logs and raw attribution.
    return {**record, "repository": REPOSITORY, "product_source": record["product_sha"],
            "run_id": record["run"], "job_id": record["job"], "attempt": 1,
            "workflow_source": record["workflow_sha"],
            "workflow_run_conclusion": record["conclusion"],
            "selected_platform": {"architecture": selected["architecture"],
                "elf_machine": selected["elf_machine"], "target": selected["target"]},
            "artifact": {"id": record["artifact"]["id"], "name": record["artifact"]["name"],
                "size_in_bytes": 49177062, "digest": record["artifact"]["github_reported_digest"]},
            "files": {
                "evidence.json": {"bytes": 60299, "sha256": record["downloaded_evidence_sha256"]},
                "resources.jsonl": {"bytes": 278604, "sha256": record["resource_log_sha256"]},
            }}


def arm_tool_pin(tool, parent):
    descriptor = {"mediaType": "application/vnd.oci.image.manifest.v1+json",
                  "digest": "sha256:3ad6bb9bc8c78c0069d03247adb9a59b3b43d68e55353e876e558b888c6c1768",
                  "size": 2261, "platform": {"architecture": "arm64", "os": "linux"}}
    config = "sha256:f27f9c00a3aca2c219642d1500610eade3ddcb6b873ea8847852ab156663d32f"
    require(tool["schema"] == "riauth.wave30-buildkit-arm64-source-pin/v1"
            and parent["index_digest"]
                == "sha256:cec9f139f45e93c5c69c60f8b07cfad9f43f4ef6b6a6cd917527fea5ff2e3dea"
            and parent["source_reference"] == BUILDKIT
            and tool["verified_parent_index_digest"] == parent["index_digest"]
            and tool["selected_descriptor"] == descriptor
            and descriptor in parent["index"]["manifests"]
            and tool["response_bytes"] == descriptor["size"]
            and tool["response_sha256"] == descriptor["digest"].split(":")[1]
            and tool["response_digest_header"] == descriptor["digest"]
            and tool["linux_arm64_manifest"]["schemaVersion"] == 2
            and tool["linux_arm64_manifest"]["mediaType"] == descriptor["mediaType"]
            and tool["linux_arm64_manifest"]["config"]
                == {"mediaType": "application/vnd.oci.image.config.v1+json", "digest": config, "size": 2668}
            and tool["linux_arm64_config_digest"] == config
            and tool["registry_read_only"] is True
            and tool["public_auth_token_retained"] is False,
            "root_arm_buildkit_identity")


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


class Cohort:
    def __init__(self, args):
        self.selected = closed_platform(os.environ.get("ARCHITECTURE"))
        self.args = args
        self.root = args.root.resolve(strict=True)
        expected_root = pathlib.Path(os.environ["RUNNER_TEMP"]).resolve() / (
            "riauth-container-" + os.environ["GITHUB_RUN_ID"] + "-" + os.environ["GITHUB_RUN_ATTEMPT"])
        require(self.root == expected_root, "exclusive_workflow_root")
        self.evidence = self.root / "evidence"
        require(self.root.is_dir() and not args.root.is_symlink()
                and stat.S_IMODE(self.root.stat().st_mode) == 0o700,
                "private_root_required")
        self.started = time.monotonic()
        self.deadline = self.started + 7200
        self.fixture_deadline = None
        self.cleanup_deadline = None
        self.phase = "source"
        self.owner = "a09-" + uuid.uuid4().hex
        self.builder = self.owner + "-builder"
        self.builder_container = "buildx_buildkit_" + self.builder + "0"
        self.builder_volume = self.builder_container + "_state"
        self.builder_attempted = False
        self.daemon_seen = False
        self.builder_id = None
        self.builder_volume_record = None
        self.builder_created_after = None
        self.builder_instance = None
        self.containers = {}
        self.volumes = {}
        self.images = {}
        self.tags = {e: "riauth-local/" + self.owner + ":" + e
                     for e in ("essentials", "platform")}
        self.active = {}
        self.lock = threading.Lock()
        self.stop_event = threading.Event()
        self.monitor_end = threading.Event()
        self.resource_error = None
        self.storage_paths = {"host": pathlib.Path("/"), "private": self.root,
                              "workspace": pathlib.Path(os.environ["GITHUB_WORKSPACE"]),
                              "docker_storage": pathlib.Path("/var/lib/docker")}
        self.home = self.root / "home"
        self.home.mkdir(mode=0o700)
        self.config = self.root / "docker-config"
        self.config.mkdir(mode=0o700)
        private_write(self.config / "config.json", b"{}\n")
        # Explicit allowlist: no CI transport token or operator Docker/riAuth env.
        self.child_env = {"PATH": os.environ["PATH"], "HOME": str(self.home),
                          "LANG": "C.UTF-8", "LC_ALL": "C.UTF-8",
                          "DOCKER_HOST": "unix:///var/run/docker.sock",
                          "DOCKER_CONFIG": str(self.config),
                          "BUILDX_CONFIG": str(self.config / "buildx")}
        self.receipt = {"schema": "riauth.local-container-cohort/v1", "project": PROJECT,
                        "repository": REPOSITORY, "official_release": False,
                        "shared_full_gate": "not_run", "source": self.selected["source"],
                        "source_tree": self.selected["tree"], "root_review": self.selected["review"],
                        "architecture": self.selected["architecture"],
                        "steps": [], "checks": [], "cleanup_errors": [],
                        "images": {}, "resources": {}, "result": "not_run",
                        "limits": [self.selected["cohort_limit"],
                                   "resource-only Dockerfile variant",
                                   "format3 enforcement through pinned public entrypoints",
                                   "private complete logical snapshot; no raw rows in evidence",
                                   "b619 native notices were later found stale; retained, not regenerated",
                                   "sampling is not a hard quota",
                                   "no registry/tag/release/tenant/device claim"]}
        self.receipt.update(owner=self.owner, expected_builder={"name": self.builder,
            "container": self.builder_container, "volume": self.builder_volume},
            planned_image_tags=self.tags, creation_records={"containers": {}, "volumes": {}},
            uid_probes=[])
        self.receipt["github"] = {k: os.environ.get(k) for k in (
            "GITHUB_REPOSITORY", "GITHUB_RUN_ID", "GITHUB_RUN_ATTEMPT", "GITHUB_JOB",
            "GITHUB_WORKFLOW_REF", "GITHUB_WORKFLOW_SHA", "GITHUB_SHA", "GITHUB_REF",
            "GITHUB_EVENT_NAME")}
        self.capacity(30 * GiB)
        self.monitor = threading.Thread(target=self.sample_loop, daemon=True)
        self.monitor.start()

    def save(self):
        target = self.evidence / "container-cohort.json"
        temporary = self.evidence / "container-cohort.tmp"
        self.receipt["pending_owned_containers"] = {name: {k: value[k] for k in
            ("pending", "after", "image") if k in value} for name, value in self.containers.items()
            if value.get("pending")}
        self.receipt["pending_owned_volumes"] = {name: {"after": value["after"]}
            for name, value in self.volumes.items() if value.get("pending")}
        payload = json_bytes(self.receipt)
        require(len(payload) <= LOG_CAP, "evidence_cap")
        with temporary.open("wb") as out:
            os.chmod(temporary, 0o600)
            out.write(payload)
        temporary.replace(target)

    def capacity(self, threshold=10 * GiB):
        with self.lock:
            paths = dict(self.storage_paths)
        free = {name: shutil.disk_usage(path).free for name, path in paths.items()}
        require(min(free.values()) >= threshold, "capacity_refusal")
        return free

    def sample_loop(self):
        path = self.evidence / "resources.jsonl"
        count, previous, maximum_gap = 0, None, 0.0
        try:
            with path.open("xb") as stream:
                os.chmod(path, 0o600)
                while not self.monitor_end.is_set():
                    now = time.monotonic()
                    gap = 0 if previous is None else now - previous
                    previous = now
                    maximum_gap = max(gap, maximum_gap)
                    free = self.capacity(0)
                    sample = {"elapsed": round(now - self.started, 3), "free_bytes": free,
                              "phase": self.phase, "gap_seconds": round(gap, 3)}
                    stream.write(json.dumps(sample).encode() + b"\n")
                    stream.flush()
                    count += 1
                    require(stream.tell() <= LOG_CAP, "resource_log_cap")
                    self.receipt["resources"] = {"samples": count,
                        "maximum_gap_seconds": maximum_gap,
                        "minimum_free_bytes": {k: min(v, self.receipt.get("resources", {})
                            .get("minimum_free_bytes", {}).get(k, v)) for k, v in free.items()}}
                    if min(free.values()) < 10 * GiB and not self.stop_event.is_set():
                        self.stop_event.set()
                        with self.lock:
                            for group in list(self.active):
                                self.kill_group(group)
                    self.monitor_end.wait(2)
        except BaseException:
            self.resource_error = "resource_monitor_refused"
            self.stop_event.set()
            with self.lock:
                for group in list(self.active):
                    self.kill_group(group)

    def check_budget(self):
        require(not self.stop_event.is_set(), "resource_monitor_refused")
        require(time.monotonic() < self.deadline, "controller_deadline")
        require(self.fixture_deadline is None or time.monotonic() < self.fixture_deadline,
                "fixture_deadline")

    @staticmethod
    def process_identity(pid):
        try:
            text = pathlib.Path("/proc/" + str(pid) + "/stat").read_text()
        except FileNotFoundError:
            return None
        fields = text[text.rfind(")") + 2:].split()
        return {"state": fields[0], "group": int(fields[2]), "session": int(fields[3]),
                "start_ticks": int(fields[19])}

    def kill_group(self, group, sig=signal.SIGTERM):
        # Caller holds lock. The leader remains unreaped until the last signal,
        # so its PID/session cannot be reused for an unrelated process group.
        owned = self.active.get(group)
        current = self.process_identity(group)
        if owned is None or current is None:
            return False
        original = owned["identity"]
        if any(current[k] != original[k] for k in ("group", "session", "start_ticks")):
            return False
        try:
            os.killpg(group, sig)
        except ProcessLookupError:
            pass
        return True

    def command(self, name, argv, *, timeout=60, input_data=None, public=False,
                cleanup=False, consumer=None, output_cap=LOG_CAP):
        require(re.fullmatch(r"[a-z0-9_-]+", name) is not None, "unsafe_phase_name")
        if not cleanup:
            self.check_budget()
        start = time.monotonic()
        end = start + timeout
        if not cleanup:
            end = min(end, self.deadline, self.fixture_deadline or self.deadline)
        elif self.cleanup_deadline is not None:
            end = min(end, self.cleanup_deadline)
        log = None
        if public:
            log = (self.evidence / (name + ".log")).open("xb")
            os.chmod(log.name, 0o600)
        outputs = {"stdout": bytearray(), "stderr": bytearray()}
        process = subprocess.Popen([str(x) for x in argv], cwd=self.args.product,
                                   env=self.child_env, stdin=subprocess.PIPE,
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                   start_new_session=True)
        try:
            with self.lock:
                identity = self.process_identity(process.pid)
                require(identity is not None and identity["group"] == identity["session"] == process.pid,
                        "owned_group_creation_identity")
                self.active[process.pid] = {"process": process, "identity": identity}
        except BaseException:
            # No group proof: signal/reap only the Popen child, never a guessed group.
            process.terminate()
            try:
                process.wait(timeout=2)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=5)
            for stream in (process.stdin, process.stdout, process.stderr):
                stream.close()
            if log is not None:
                log.close()
            self.receipt["cleanup_errors"].append("group_creation_identity_unproven")
            raise
        status = None
        try:
            require(input_data is None or len(input_data) <= 65536, "stdin_cap")
            if input_data:
                process.stdin.write(input_data)
            process.stdin.close()
            with selectors.DefaultSelector() as selector:
                for field in ("stdout", "stderr"):
                    stream = getattr(process, field)
                    os.set_blocking(stream.fileno(), False)
                    selector.register(stream, selectors.EVENT_READ, field)
                total, logged = 0, 0
                while selector.get_map() or self.process_identity(process.pid)["state"] != "Z":
                    if not cleanup:
                        self.check_budget()
                    require(time.monotonic() < end, "command_timeout")
                    for key, _ in selector.select(0.2):
                        chunk = os.read(key.fileobj.fileno(), 65536)
                        if not chunk:
                            selector.unregister(key.fileobj)
                            continue
                        if key.data == "stdout" and consumer is not None:
                            consumer(chunk)
                        else:
                            total += len(chunk)
                            require(total <= output_cap, "command_output_cap")
                            outputs[key.data].extend(chunk)
                        if log is not None:
                            logged += len(chunk)
                            require(logged <= LOG_CAP, "public_log_cap")
                            log.write(chunk)
        finally:
            # Reap the direct process and eliminate descendants in this exact group.
            with self.lock:
                self.kill_group(process.pid)
            grace = time.monotonic() + 2
            while time.monotonic() < grace:
                current = self.process_identity(process.pid)
                if current is None or current["state"] == "Z":
                    break
                time.sleep(0.05)
            with self.lock:
                self.kill_group(process.pid, signal.SIGKILL)
                status = process.wait(timeout=5)  # No group signal after this reap.
            empty = False
            group_deadline = time.monotonic() + 2
            while time.monotonic() < group_deadline:
                try:
                    os.killpg(process.pid, 0)
                except ProcessLookupError:
                    empty = True
                    break
                time.sleep(0.05)
            with self.lock:
                if empty:
                    self.active.pop(process.pid, None)
            for stream in (process.stdout, process.stderr):
                stream.close()
            if log is not None:
                log.close()
            item = {"name": name, "exit": status, "elapsed": round(time.monotonic() - start, 3),
                    "output_recorded": public, "owned_child_reaped": process.returncode is not None,
                    "owned_group_empty": empty}
            if public:
                item.update(log_bytes=pathlib.Path(log.name).stat().st_size,
                            log_sha256=file_hash(pathlib.Path(log.name)))
            self.receipt["steps"].append(item)
            require(empty, "owned_cli_group_remains")
        return status, bytes(outputs["stdout"]), bytes(outputs["stderr"])

    def docker(self, name, *args, **options):
        return self.command(name, ["docker", *args], **options)

    def inspect(self, kind, identifier, cleanup=False):
        code, out, _ = self.docker("inspect-" + kind, kind, "inspect", identifier,
                                   timeout=20, cleanup=cleanup)
        require(code == 0, "owned_inspection_failed")
        value = json.loads(out)
        require(isinstance(value, list) and len(value) == 1, "inspection_shape")
        return value[0]

    def absent(self, kind, name):
        self.prove_absent(kind, name)

    def prove_absent(self, kind, name, cleanup=False):
        code, _, error = self.docker("check-absent-" + kind, kind, "inspect", name,
                                      timeout=20, cleanup=cleanup)
        markers = {"container": (b"No such container", b"No such object"),
                   "image": (b"No such image", b"No such object"),
                   "volume": (b"no such volume",)}
        require(code == 1 and any(marker in error for marker in markers[kind]),
                "resource_preexists_or_daemon_unavailable")
        code, _, _ = self.docker("absence-daemon-check", "info", "--format", "{{.OSType}}",
                                  timeout=20, cleanup=cleanup)
        require(code == 0, "absence_not_proven_daemon_unavailable")

    def git(self, path, *args):
        code, out, _ = self.command("git-source", ["git", "-C", path, *args], timeout=20)
        require(code == 0, "immutable_git_read_failed")
        return out

    def source_check(self):
        require((platform.system(), platform.machine(), os.environ.get("RUNNER_ARCH"),
                 os.environ.get("RUNNER_ENVIRONMENT")) == ("Linux", self.selected["machine"], self.selected["runner_arch"], "github-hosted"),
                "unsupported_native_host")
        require(os.environ.get("GITHUB_REPOSITORY") == REPOSITORY
                and os.environ.get("GITHUB_EVENT_NAME") == "workflow_dispatch"
                and os.environ.get("GITHUB_REF", "").startswith("refs/heads/"), "manual_repository")
        controller = self.git(self.args.controller, "rev-parse", "HEAD").decode().strip()
        require(re.fullmatch(r"[0-9a-f]{40}", controller) is not None
                and controller == os.environ.get("GITHUB_SHA")
                and controller == os.environ.get("GITHUB_WORKFLOW_SHA"), "controller_identity")
        for path, pin in ((self.args.product, self.selected["source"]), (self.args.review, self.selected["review"]),
                          (self.args.controller, controller)):
            require(not path.is_symlink() and self.git(path, "rev-parse", "HEAD").decode().strip() == pin
                    and self.git(path, "status", "--porcelain") == b"", "clean_exact_checkout")
        require(self.git(self.args.product, "rev-parse", "HEAD^{tree}").decode().strip() == self.selected["tree"],
                "product_tree")
        self.native = json.loads((self.args.review / self.selected["native_receipt"]).read_bytes())
        for relative, expected in self.selected["receipt_hashes"].items():
            require(file_hash(self.args.review / relative) == expected, "root_receipt_hash")
        tool = json.loads((self.args.review / self.selected["buildkit_receipt"]).read_bytes())
        require(tool["source_reference"] == self.selected["buildkit"] and tool["layers_downloaded"] is False
                and tool["native_image_executed"] is False, "tool_pin_scope")
        if self.selected["architecture"] == "arm64":
            arm_tool_pin(tool, json.loads((self.args.review / BUILDKIT_RECEIPT).read_bytes()))
        self.native = native_receipt_view(self.native, self.selected)
        r = self.native
        require(r["schema"] == self.selected["native_schema"]
                and r["project"] == PROJECT and r["repository"] == REPOSITORY
                and r["product_source"] == self.selected["source"] and r["product_tree"] == self.selected["tree"]
                and r["run_id"] == self.selected["run"] and r["attempt"] == 1
                and r["job_id"] == self.selected["job"] and r["workflow_run_conclusion"] == "success"
                and r["selected_platform"] == {"architecture": self.selected["architecture"], "elf_machine": self.selected["elf_machine"],
                    "target": self.selected["target"]} and r["shared_full_gate"] == "not_run",
                "root_native_identity")
        require(len(r["products"]) == 5 and len(r["steps"]) == 11
                and all(x["exit_code"] == 0 for x in r["steps"]), "root_native_outcome")
        for path, expected in r["inputs"].items():
            require(file_hash(self.args.product / path) == expected, "product_input_identity")
        original = (self.args.product / "Dockerfile").read_bytes()
        require(digest(original) == DOCKERFILE_HASH, "dockerfile_identity")
        needle = b'cargo build --release --locked --no-default-features --features "$RIAUTH_EDITION" --bin riauth'
        prefix = (b'CARGO_HOME=/build/cargo-home CARGO_TARGET_DIR=/build/target '
                  b'CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 '
                  b'CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_RELEASE_DEBUG=0 ')
        require(original.count(needle) == 1, "serial_recipe_span")
        recipe = original.replace(needle, prefix + needle, 1)
        require(recipe.replace(prefix + needle, needle, 1) == original, "recipe_reversal")
        self.recipe = self.root / "Dockerfile.serial"
        private_write(self.recipe, recipe)
        self.receipt.update(controller=controller,
            helper_sha256=file_hash(pathlib.Path(__file__)),
            workflow_sha256=file_hash(self.args.controller / WORKFLOW),
            native_input={"receipt_sha256": self.selected["receipt_hashes"][self.selected["native_receipt"]],
                          "run": r["run_id"], "artifact": r["artifact"]},
            tool_pin={"reference": self.selected["buildkit"], "receipt_sha256": self.selected["receipt_hashes"][self.selected["buildkit_receipt"]]},
            recipe={"variant": "private-serial-resource-prefix", "original_sha256": digest(original),
                    "derived_sha256": digest(recipe), "exact_reverse": True})
        self.receipt["native_input"]["raw_schema"] = r["schema"]
        if self.selected["architecture"] == "arm64":
            self.receipt["native_input"]["root_outer_zip_rehashed"] = False
        # Credentials are outside product; only the Dockerfile's unchanged COPY set is sent.
        require(self.root not in self.args.product.resolve().parents
                and self.args.product.resolve() not in self.root.parents
                and self.root != self.args.product.resolve(), "private_context_overlap")
        self.save()

    def http(self, url, *, token=None, data=None, method="GET", cap=LOG_CAP, extra_headers=None):
        self.check_budget()
        headers = {"Accept": "application/json"}
        if extra_headers:
            require(set(extra_headers) <= {"If-Match", "Idempotency-Key"}, "closed_route_headers")
            headers.update(extra_headers)
        if token is not None:
            headers["Authorization"] = "Bearer " + token
        if data is not None:
            headers["Content-Type"] = "application/json"
            data = json.dumps(data).encode()
        request = urllib.request.Request(url, data=data, headers=headers, method=method)
        opener = urllib.request.build_opener(NoRedirect())
        try:
            response = opener.open(request, timeout=15)
        except urllib.error.HTTPError as error:
            response = error
        with response:
            body = bytearray()
            end = min(time.monotonic() + 60, self.deadline, self.fixture_deadline or self.deadline)
            while True:
                self.check_budget()
                require(time.monotonic() < end, "http_deadline")
                block = response.read1(65536)
                if not block:
                    break
                body.extend(block)
                require(len(body) <= cap, "http_body_cap")
            self.check_budget()
            return response.status, bytes(body), dict(response.headers)

    def github_json(self, suffix):
        url = "https://api.github.com/repos/" + REPOSITORY + suffix
        code, body, _ = self.http(url, token=os.environ["A09_GH_TOKEN"])
        require(code == 200, "github_metadata_refused")
        return json.loads(body)

    def native_transport(self):
        self.phase = "native-transport"
        self.capacity(30 * GiB)
        r = self.native
        run = self.github_json("/actions/runs/" + str(self.selected["run"]) + "/attempts/1")
        artifact = self.github_json("/actions/artifacts/" + str(self.selected["artifact"]))
        require(run["id"] == r["run_id"] and run["run_attempt"] == 1
                and run["conclusion"] == "success" and run["status"] == "completed"
                and run["head_sha"] == r["workflow_source"]
                and run["repository"]["full_name"] == REPOSITORY
                and run["path"] == ".github/workflows/check-local-artifacts.yml", "native_run_metadata")
        expected = r["artifact"]
        require(all(artifact.get(k) == v for k, v in expected.items())
                and artifact["expired"] is False
                and artifact["workflow_run"]["id"] == r["run_id"]
                and artifact["workflow_run"]["head_sha"] == r["workflow_source"], "native_artifact_metadata")
        # Authenticated API redirect only; signed storage GET has no bearer header.
        code, _, headers = self.http("https://api.github.com/repos/" + REPOSITORY
            + "/actions/artifacts/" + str(self.selected["artifact"]) + "/zip", token=os.environ["A09_GH_TOKEN"], cap=65536)
        location = headers.get("Location") or headers.get("location")
        require(code == 302 and isinstance(location, str), "artifact_redirect")
        target = urllib.parse.urlsplit(location)
        require(target.scheme == "https" and target.username is None and target.password is None
                and target.port in (None, 443) and target.hostname is not None
                and (target.hostname.endswith(".blob.core.windows.net")
                     or target.hostname.endswith(".githubusercontent.com")), "artifact_storage_origin")
        outer = self.root / "native.zip"
        limit = time.monotonic() + 180
        opener = urllib.request.build_opener(NoRedirect())
        with opener.open(location, timeout=15) as response, outer.open("xb") as out:
            os.chmod(outer, 0o600)
            require(response.status == 200, "artifact_storage_status")
            count = 0
            while True:
                self.check_budget()
                require(time.monotonic() < limit, "artifact_download_deadline")
                block = response.read1(65536)
                if not block:
                    break
                count += len(block)
                require(count <= NATIVE_CAP, "artifact_compressed_cap")
                out.write(block)
        require(count == expected["size_in_bytes"]
                and "sha256:" + file_hash(outer) == expected["digest"], "outer_zip_digest")
        self.receipt["native_input"]["outer_zip_rehashed"] = True
        unpacked = self.root / "native"
        unpacked.mkdir(mode=0o700)
        allowed = {"evidence.json", "resources.jsonl"}
        allowed |= {x["name"] + ".log" for x in r["steps"]}
        allowed |= {"archives/" + x["archive"] for x in r["products"]}
        with zipfile.ZipFile(outer) as source:
            files, size = set(), 0
            for info in source.infolist():
                require(info.filename == "archives/" or info.filename in allowed, "zip_member_path")
                require(not (info.flag_bits & 1), "encrypted_zip_member")
                mode = info.external_attr >> 16
                require(stat.S_IFMT(mode) in (0, stat.S_IFDIR if info.is_dir() else stat.S_IFREG),
                        "zip_member_type")
                if info.is_dir():
                    require(info.filename == "archives/", "zip_directory")
                    continue
                require(info.filename not in files, "duplicate_zip_member")
                files.add(info.filename)
                size += info.file_size
                require(size <= NATIVE_CAP, "artifact_expanded_cap")
                destination = unpacked / info.filename
                destination.parent.mkdir(mode=0o700, exist_ok=True)
                with source.open(info) as incoming, destination.open("xb") as out:
                    os.chmod(destination, 0o600)
                    copied = 0
                    while block := incoming.read(1024 * 1024):
                        self.check_budget()
                        copied += len(block)
                        require(copied <= info.file_size, "zip_payload_size")
                        out.write(block)
                    require(copied == info.file_size, "zip_payload_truncated")
            require(files == allowed, "native_zip_inventory")
        for name, declared in r["files"].items():
            p = unpacked / name
            require(p.stat().st_size == declared["bytes"] and file_hash(p) == declared["sha256"],
                    "native_evidence_identity")
        native_evidence = json.loads((unpacked / "evidence.json").read_bytes())
        require(native_evidence["source_sha"] == self.selected["source"] and native_evidence["source_tree"] == self.selected["tree"]
                and native_evidence["native_archive_slice"] == "passed"
                and [{k: v for k, v in p.items() if k != "observed_server_capabilities"}
                     for p in native_evidence["products"]] == r["products"], "native_evidence_outcome")
        for step in r["steps"]:
            log = unpacked / (step["name"] + ".log")
            require(log.stat().st_size == step["log_bytes"] and file_hash(log) == step["log_sha256"],
                    "native_log_identity")
        self.bins = {}
        expanded = 0
        for product in r["products"]:
            archive = unpacked / "archives" / product["archive"]
            require(archive.stat().st_size == product["archive_bytes"]
                    and file_hash(archive) == product["archive_sha256"], "native_archive_identity")
            directory = unpacked / (product["edition"] + "-" + product["binary"])
            directory.mkdir(mode=0o700)
            with tarfile.open(archive, "r:gz") as source:
                members = source.getmembers()
                require(len(members) == 3 and {m.name for m in members}
                    == {product["binary"], "LICENSE", "THIRD_PARTY_NOTICES.md"}
                    and all(m.isfile() for m in members), "native_tar_members")
                for member in members:
                    expanded += member.size
                    require(expanded <= NATIVE_CAP, "native_tar_expanded_cap")
                    with source.extractfile(member) as incoming:
                        payload = incoming.read(member.size + 1)
                    require(len(payload) == member.size, "native_tar_payload")
                    destination = directory / member.name
                    private_write(destination, payload)
                    if member.name != product["binary"]:
                        require(digest(payload) == r["inputs"][member.name], "native_license_identity")
                binary = directory / product["binary"]
                require(binary.stat().st_size == product["binary_bytes"]
                        and file_hash(binary) == product["binary_sha256"], "native_binary_identity")
                self.elf(binary, self.selected["elf_machine"])
                binary.chmod(0o755)  # Public executable; enclosing host directories stay private.
                self.bins[(product["edition"], product["binary"])] = binary
        self.native_caps = {}
        for edition in ("essentials", "platform"):
            code, output, _ = self.command("native-capabilities", [self.bins[(edition, "riauth")],
                "--json", "capabilities"])
            envelope = json.loads(output)
            require(code == 0 and envelope["ok"] is True, "native_capabilities_status")
            recorded = json.loads((unpacked / ("capabilities-" + edition + ".log")).read_bytes())
            require(envelope == recorded, "native_capabilities_exact")
            observed = next(p["observed_server_capabilities"] for p in native_evidence["products"]
                            if p["edition"] == edition and p["binary"] == "riauth")
            require(observed == envelope["data"], "native_product_capability_record")
            self.capabilities(envelope["data"], edition)
            self.native_caps[edition] = envelope["data"]
        self.save()

    @staticmethod
    def elf(path, machine):
        with path.open("rb") as stream:
            header = stream.read(20)
        require(len(header) == 20 and header[:6] == b"\x7fELF\x02\x01"
                and header[18:20] == machine.to_bytes(2, "little"), "native_elf" + str(machine))

    def capabilities(self, data, edition):
        require(data["edition"] == edition and data["build_features"]
                == (["essentials"] if edition == "essentials" else ["essentials", "platform"]),
                "capability_features")
        version = tomllib.loads((self.args.product / "Cargo.toml").read_text())["package"]["version"]
        require(data["version"] == version and data["target"] == {"arch": self.selected["machine"], "os": "linux"},
                "capability_native_target_version")
        return data

    def daemon_setup(self):
        self.phase = "builder-setup"
        self.capacity(30 * GiB)
        require(stat.S_ISSOCK(pathlib.Path("/var/run/docker.sock").stat().st_mode), "native_socket")
        code, out, _ = self.docker("daemon-info", "info", "--format", "{{json .}}", timeout=30)
        require(code == 0, "native_daemon_unavailable")
        info = json.loads(out)
        require(info["OSType"] == "linux" and info["Architecture"] == self.selected["daemon_arch"]
                and "desktop" not in info["OperatingSystem"].lower(), "native_daemon_required")
        self.daemon_seen = True
        storage = pathlib.Path(info["DockerRootDir"])
        require(storage.is_absolute() and storage.is_dir(), "daemon_storage_unmeasurable")
        with self.lock:
            self.storage_paths["docker_storage"] = storage
        self.capacity(30 * GiB)
        self.receipt["daemon"] = {k: info[k] for k in
            ("OSType", "Architecture", "DockerRootDir", "ServerVersion", "OperatingSystem")}
        self.absent("container", self.builder_container)
        self.absent("volume", self.builder_volume)
        code, out, _ = self.docker("builder-list", "buildx", "ls", "--format", "{{json .}}")
        require(code == 0 and self.builder.encode() not in out, "builder_preexists")
        # Pull a content-addressed tool, never a mutable tag. No registry writes.
        code, _, _ = self.docker("pull-pinned-buildkit", "image", "pull", self.selected["buildkit"],
                                  timeout=300, public=True)
        require(code == 0, "buildkit_pull_failed")
        tool = self.inspect("image", self.selected["buildkit"])
        pin = json.loads((self.args.review / self.selected["buildkit_receipt"]).read_bytes())
        require(tool["Id"] == pin[self.selected["tool_manifest"]]["config"]["digest"]
                and tool["Os"] == "linux" and tool["Architecture"] == self.selected["oci_arch"], "buildkit_actual_identity")
        self.receipt["tool_pin"]["actual_config_id"] = tool["Id"]
        self.builder_created_after = int(datetime.datetime.now(datetime.timezone.utc).timestamp())
        self.builder_attempted = True
        self.save()  # Expected exclusive names retained even if setup loses its response.
        code, _, _ = self.docker("builder-create", "buildx", "create", "--name", self.builder,
            "--driver", "docker-container", "--driver-opt", "image=" + self.selected["buildkit"],
            "--driver-opt", "memory=8g", "--driver-opt", "cpu-period=100000",
            "--driver-opt", "cpu-quota=300000", timeout=60, public=True)
        require(code == 0, "builder_create_failed")
        instance = self.config / "buildx" / "instances" / self.builder
        require(instance.is_file() and not instance.is_symlink(), "private_builder_instance")
        self.builder_instance = file_hash(instance)
        code, _, _ = self.docker("builder-bootstrap", "buildx", "inspect", self.builder,
                                  "--bootstrap", timeout=180, public=True)
        require(code == 0, "builder_bootstrap_failed")
        self.recover_builder()
        require(self.builder_id is not None, "bootstrapped_builder_identity_missing")
        self.save()

    def recover_builder(self):
        """Creation interval + exclusive name + private instance + actual mounts.

        Also used on a failed bootstrap, before any cleanup of daemon children.
        Failure to prove these identities blocks deletion, never triggers prune.
        """
        if not self.builder_attempted:
            return
        code, out, error = self.docker("recover-builder-inspect", "container", "inspect",
                                      self.builder_container, timeout=20, cleanup=True)
        if code == 1 and any(x in error for x in (b"No such container", b"No such object")):
            self.prove_absent("container", self.builder_container, cleanup=True)
            self.prove_absent("volume", self.builder_volume, cleanup=True)
            # No daemon child was created. The private builder instance can be removed later.
            return
        require(code == 0, "builder_recovery_unavailable")
        instance = self.config / "buildx" / "instances" / self.builder
        require(instance.is_file() and not instance.is_symlink(), "builder_identity_unavailable")
        config = json.loads(instance.read_bytes())
        require(config["Name"] == self.builder and config["Driver"] == "docker-container"
                and len(config["Nodes"]) == 1
                and config["Nodes"][0]["Endpoint"] in {"default", "unix:///var/run/docker.sock"}
                and config["Nodes"][0]["DriverOpts"]["image"] == self.selected["buildkit"],
                "private_builder_binding")
        if self.builder_instance is not None:
            require(file_hash(instance) == self.builder_instance, "builder_instance_changed")
        self.builder_instance = file_hash(instance)
        item = json.loads(out)[0]
        created = datetime.datetime.fromisoformat(item["Created"].replace("Z", "+00:00")).timestamp()
        require(item["Name"] == "/" + self.builder_container
                and item["Config"]["Image"] == self.selected["buildkit"]
                and created >= self.builder_created_after
                and (self.builder_id is None or item["Id"] == self.builder_id), "builder_creation_identity")
        mounts = [m for m in item["Mounts"] if m["Type"] == "volume"]
        require(len(mounts) == 1 and mounts[0]["Name"] == self.builder_volume
                and mounts[0]["Destination"] == "/var/lib/buildkit", "builder_state_mount")
        volume = self.inspect("volume", self.builder_volume, cleanup=True)
        volume_created = datetime.datetime.fromisoformat(volume["CreatedAt"].replace("Z", "+00:00")).timestamp()
        require(volume_created >= self.builder_created_after, "builder_volume_creation_identity")
        self.builder_id = item["Id"]
        self.builder_volume_record = volume
        self.receipt["builder"] = {"name": self.builder, "container_id": item["Id"],
            "created": item["Created"], "state_volume": volume["Name"],
            "state_created": volume["CreatedAt"], "instance_sha256": self.builder_instance,
            "privileged_tool_only": item["HostConfig"]["Privileged"]}

    def create_volume(self, suffix):
        name = self.owner + "-" + suffix
        self.absent("volume", name)
        created_after = int(datetime.datetime.now(datetime.timezone.utc).timestamp())
        self.volumes[name] = {"pending": True, "after": created_after}
        self.save()
        code, _, _ = self.docker("volume-create", "volume", "create", "--label",
                                  OWNER_LABEL + "=" + self.owner, name)
        require(code == 0, "volume_create_failed")
        self.confirm_volume(name)
        return name

    def confirm_volume(self, name, missing_ok=False):
        code, out, error = self.docker("volume-owner-inspect", "volume", "inspect", name,
                                      timeout=20, cleanup=True)
        if missing_ok and code == 1 and b"no such volume" in error:
            self.prove_absent("volume", name, cleanup=True)
            return None
        require(code == 0, "volume_inspection_failed")
        item = json.loads(out)[0]
        require(item["Name"] == name and (item.get("Labels") or {}).get(OWNER_LABEL) == self.owner,
                "volume_owner_mismatch")
        old = self.volumes[name]
        if not old.get("pending"):
            require(item["CreatedAt"] == old["CreatedAt"] and item["Mountpoint"] == old["Mountpoint"],
                    "volume_replaced")
        else:
            created = datetime.datetime.fromisoformat(item["CreatedAt"].replace("Z", "+00:00")).timestamp()
            require(created >= old["after"], "volume_creation_interval")
        self.volumes[name] = item
        self.receipt["creation_records"]["volumes"][name] = {k: item[k]
            for k in ("Name", "CreatedAt", "Mountpoint", "Driver")}
        return item

    def create_container(self, image, suffix, entrypoint, args, *, mounts=(), network="none"):
        name = self.owner + "-" + suffix + "-" + uuid.uuid4().hex[:12]
        self.absent("container", name)
        before = int(datetime.datetime.now(datetime.timezone.utc).timestamp())
        self.containers[name] = {"pending": True, "after": before, "image": image}
        self.save()
        argv = ["container", "create", "--name", name, "--label", OWNER_LABEL + "=" + self.owner,
                "--user", "10001:10001", "--network", network, "--read-only", "--cap-drop", "ALL",
                "--security-opt", "no-new-privileges", "--pids-limit", "256", "--memory", "2g",
                "--cpus", "2", "--log-driver", "none", "--tmpfs", "/tmp:rw,nosuid,noexec,size=16m",
                "--interactive", "--entrypoint", entrypoint]
        for mount in mounts:
            argv += ["--mount", mount]
        if not any("dst=/data" in m for m in mounts):
            argv += ["--mount", "type=tmpfs,dst=/data,tmpfs-size=16777216"]
        argv += [image, *args]
        code, _, _ = self.docker("container-create", *argv)
        require(code == 0, "container_create_failed")
        self.confirm_container(name)
        return name

    def confirm_container(self, name, missing_ok=False):
        code, out, error = self.docker("container-owner-inspect", "container", "inspect", name,
                                      timeout=20, cleanup=True)
        if missing_ok and code == 1 and any(x in error for x in (b"No such container", b"No such object")):
            self.prove_absent("container", name, cleanup=True)
            return None
        require(code == 0, "container_inspection_failed")
        item = json.loads(out)[0]
        require(item["Name"] == "/" + name
                and (item["Config"].get("Labels") or {}).get(OWNER_LABEL) == self.owner,
                "container_owner_mismatch")
        old = self.containers[name]
        if not old.get("pending"):
            require(item["Id"] == old["Id"] and item["Created"] == old["Created"], "container_replaced")
        else:
            created = datetime.datetime.fromisoformat(item["Created"].replace("Z", "+00:00")).timestamp()
            require(created >= old["after"] and item["Image"] == old["image"], "container_creation_identity")
        require(item["Config"]["User"] == "10001:10001"
                and item["HostConfig"]["ReadonlyRootfs"] and not item["HostConfig"]["Privileged"]
                and "ALL" in item["HostConfig"]["CapDrop"]
                and not any(m.get("Source") == "/var/run/docker.sock" for m in item["Mounts"]),
                "application_controls")
        self.containers[name] = item
        self.receipt["creation_records"]["containers"][name] = {k: item[k]
            for k in ("Id", "Created", "Image", "Name")}
        return item

    def remove_container(self, name, cleanup=False):
        item = self.confirm_container(name, missing_ok=cleanup)
        if item is None:
            del self.containers[name]
            return
        if item["State"]["Running"]:
            code, _, _ = self.docker("owned-container-stop", "container", "stop", "--time", "5",
                                      item["Id"], timeout=25, cleanup=cleanup)
            require(code == 0 and not self.confirm_container(name)["State"]["Running"],
                    "owned_writer_stop_failed")
        code, _, _ = self.docker("container-remove", "container", "rm", "--force", item["Id"],
                                  timeout=30, cleanup=cleanup)
        require(code == 0, "owned_container_remove_failed")
        self.prove_absent("container", item["Id"], cleanup=cleanup)
        del self.containers[name]

    def tool(self, image, entrypoint, args, *, mounts=(), input_data=None, timeout=60):
        name = self.create_container(image, "tool", entrypoint, args, mounts=mounts)
        try:
            result = self.docker("container-tool", "container", "start", "--attach", "--interactive",
                                 name, input_data=input_data, timeout=timeout)
            state = self.confirm_container(name)["State"]
            require(not state["Running"] and result[0] in (0, state["ExitCode"]), "tool_exit_identity")
            # Product exit is State.ExitCode, independently of attach transport status.
            return state["ExitCode"], result[1], result[2]
        finally:
            self.remove_container(name, cleanup=True)

    def image_identity(self, edition, identifier):
        item = self.inspect("image", identifier)
        labels = item["Config"].get("Labels") or {}
        require(re.fullmatch(r"sha256:[0-9a-f]{64}", item["Id"]) is not None
                and item["Os"] == "linux" and item["Architecture"] == self.selected["oci_arch"]
                and item["Config"]["User"] == "10001:10001"
                and item["Config"]["Entrypoint"] == ["riauth", "--config", "/data/riauth.toml"]
                and item["Config"]["Cmd"] == ["serve"]
                and item["Config"]["WorkingDir"] == "/data"
                and item["Config"]["Volumes"] == {"/data": {}}
                and labels.get(OWNER_LABEL) == self.owner
                and labels.get("org.riauth.edition") == edition
                and labels.get("org.opencontainers.image.revision") == self.selected["source"]
                and labels.get("org.opencontainers.image.source") == "https://github.com/" + REPOSITORY,
                "image_product_identity")
        return item

    def build_images(self):
        self.phase = "images"
        for edition, tag in self.tags.items():
            self.capacity(30 * GiB)
            self.absent("image", tag)
            iid = self.root / (edition + ".iid")
            code, _, _ = self.docker("build-" + edition, "buildx", "build", "--builder", self.builder,
                "--platform", self.selected["platform"], "--load", "--progress", "plain", "--file", self.recipe,
                "--build-arg", "RIAUTH_EDITION=" + edition, "--build-arg", "RIAUTH_COMMIT=" + self.selected["source"],
                "--label", OWNER_LABEL + "=" + self.owner, "--tag", tag, "--iidfile", iid,
                self.args.product, timeout=1800, public=True)
            require(code == 0 and iid.is_file(), "image_build_failed")
            identifier = iid.read_text().strip()
            item = self.image_identity(edition, identifier)
            require(tag in item["RepoTags"], "owned_image_tag")
            self.images[edition] = item["Id"]
            self.receipt["images"][edition] = {"id": item["Id"], "tag": tag,
                "os": item["Os"], "architecture": item["Architecture"],
                "labels": item["Config"]["Labels"], "recipe": self.receipt["recipe"]}
            self.save()
        for edition, image in self.images.items():
            self.capacity(30 * GiB)
            archive = self.evidence / ("local-" + edition + "-" + self.selected["machine"] + ".docker.tar.gz")
            size = 0
            with archive.open("xb") as raw:
                os.chmod(archive, 0o600)
                with gzip.GzipFile(fileobj=raw, mode="wb", mtime=0, filename="") as out:
                    def consume(block):
                        nonlocal size
                        size += len(block)
                        require(size <= IMAGE_CAP, "image_archive_expanded_cap")
                        out.write(block)
                        require(raw.tell() <= IMAGE_CAP, "image_archive_compressed_cap")
                    code, _, _ = self.docker("save-" + edition, "image", "save", self.tags[edition],
                                              timeout=300, consumer=consume)
                    require(code == 0, "image_save_failed")
            self.validate_image_tar(archive, edition, image)
            self.receipt["images"][edition].update(archive=archive.name,
                archive_bytes=archive.stat().st_size, archive_sha256=file_hash(archive))
        for edition, image in self.images.items():
            self.image_identity(edition, image)
            code, _, _ = self.docker("remove-for-reload", "image", "rm", "--no-prune", self.tags[edition])
            require(code == 0, "image_remove_for_reload_failed")
        for edition, image in self.images.items():
            self.capacity(30 * GiB)
            archive = self.evidence / self.receipt["images"][edition]["archive"]
            require(file_hash(archive) == self.receipt["images"][edition]["archive_sha256"], "image_archive_changed")
            self.validate_image_tar(archive, edition, image)
            code, _, _ = self.docker("load-" + edition, "image", "load", "--input", archive, timeout=300)
            require(code == 0 and self.image_identity(edition, self.tags[edition])["Id"] == image,
                    "loaded_image_identity")
            self.inspect_product(edition, image)
        self.save()

    def validate_image_tar(self, archive, edition, image):
        require(archive.stat().st_size <= IMAGE_CAP, "image_archive_size")
        members, hashes, payloads = {}, {}, {}
        total = 0
        with tarfile.open(archive, "r:gz") as source:
            for member in source:
                self.check_budget()
                name = member.name.rstrip("/")
                path = pathlib.PurePosixPath(name)
                require(name and not path.is_absolute() and ".." not in path.parts
                        and name == str(path) and name not in members
                        and (member.isfile() or member.isdir()), "image_tar_member")
                members[name] = member
                if member.isdir():
                    continue
                total += member.size
                require(total <= IMAGE_CAP, "image_tar_expanded_cap")
                h, copied = hashlib.sha256(), 0
                small = bytearray()
                with source.extractfile(member) as incoming:
                    while block := incoming.read(1024 * 1024):
                        self.check_budget()
                        copied += len(block)
                        h.update(block)
                        if member.size <= LOG_CAP:
                            small.extend(block)
                require(copied == member.size, "image_tar_truncated")
                hashes[name] = h.hexdigest()
                if member.size <= LOG_CAP:
                    payloads[name] = bytes(small)
        require("manifest.json" in payloads, "docker_archive_manifest")
        manifest = json.loads(payloads["manifest.json"])
        require(isinstance(manifest, list) and len(manifest) == 1
                and manifest[0]["RepoTags"] == [self.tags[edition]], "image_tar_exact_tag")
        config, layers = manifest[0]["Config"], manifest[0]["Layers"]
        require(config in payloads and "sha256:" + hashes[config] == image
                and isinstance(layers, list) and layers and len(set(layers)) == len(layers)
                and all(layer in hashes for layer in layers), "image_tar_config_layers")
        settings = json.loads(payloads[config])
        require(settings["os"] == "linux" and settings["architecture"] == self.selected["oci_arch"]
                and settings["config"]["Labels"][OWNER_LABEL] == self.owner
                and settings["config"]["Labels"]["org.riauth.edition"] == edition
                and settings["config"]["Labels"]["org.opencontainers.image.revision"] == self.selected["source"],
                "saved_config_identity")
        allowed = {"manifest.json", config, *layers}
        # Docker save can include OCI index/layout or legacy per-layer metadata.
        allowed |= {"index.json", "oci-layout", "repositories"}
        if "index.json" in payloads:
            index = json.loads(payloads["index.json"])
            require(index["schemaVersion"] == 2 and len(index["manifests"]) == 1, "saved_oci_index")
            descriptor = index["manifests"][0]
            ref = descriptor["digest"]
            require(re.fullmatch(r"sha256:[0-9a-f]{64}", ref) is not None, "oci_manifest_digest")
            name = "blobs/sha256/" + ref.split(":")[1]
            require(name in payloads and hashes[name] == ref.split(":")[1], "oci_manifest_payload")
            oci = json.loads(payloads[name])
            require(oci["schemaVersion"] == 2 and oci["config"]["digest"] == image
                    and ["blobs/sha256/" + x["digest"].split(":")[1] for x in oci["layers"]] == layers,
                    "oci_image_manifest_binding")
            allowed.add(name)
        if (self.receipt.get("daemon", {}).get("ServerVersion") == "28.0.4"
                and all(type(x) is str and re.fullmatch(r"blobs/sha256/[0-9a-f]{64}", x)
                        for x in layers)):
            allowed |= self.expected_v28_legacy_members(
                settings, layers, members, hashes, payloads, edition, allowed)
        for layer in layers:
            parent = str(pathlib.PurePosixPath(layer).parent)
            if parent != "." and not layer.startswith("blobs/sha256/"):
                allowed |= {parent + "/VERSION", parent + "/json"}
        require(set(hashes) <= allowed, "image_tar_unreferenced_member")
        for name, actual in hashes.items():
            if name.startswith("blobs/sha256/"):
                require(name == "blobs/sha256/" + actual, "image_blob_digest")

    def expected_v28_legacy_members(self, settings, layers, members, hashes, payloads,
                                   edition, ordinary_allowed):
        """Derive only the fixed recipe's v28.0.4 legacy blobs, never classify extras."""
        self.check_budget()
        require(type(settings) is dict and set(settings) == {
                    "architecture", "config", "created", "history", "os", "rootfs"}
                and settings["architecture"] == self.selected["oci_arch"]
                and settings["os"] == "linux" and type(settings["history"]) is list
                and edition in {"essentials", "platform"}
                and re.fullmatch(r"a09-[0-9a-f]{32}", self.owner) is not None,
                "image_legacy_profile")
        rootfs = settings["rootfs"]
        require(type(rootfs) is dict and set(rootfs) == {"type", "diff_ids"}
                and rootfs["type"] == "layers" and type(rootfs["diff_ids"]) is list
                and 1 <= len(rootfs["diff_ids"]) <= 128
                and all(type(x) is str and re.fullmatch(r"sha256:[0-9a-f]{64}", x)
                        for x in rootfs["diff_ids"]), "image_legacy_rootfs")
        diff_ids = rootfs["diff_ids"]
        require(len(set(diff_ids)) == len(diff_ids)
                and layers == ["blobs/sha256/" + x[7:] for x in diff_ids]
                and all(name in members and members[name].isfile()
                        and hashes.get(name) == diff_id[7:]
                        for name, diff_id in zip(layers, diff_ids)), "image_legacy_layers")
        expected_config = {
            "User": "10001:10001", "Cmd": ["serve"],
            "Entrypoint": ["riauth", "--config", "/data/riauth.toml"],
            "WorkingDir": "/data", "Volumes": {"/data": {}},
            "ExposedPorts": {"9000/tcp": {}},
            "Env": ["PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"],
            "ArgsEscaped": True,
            "Labels": {OWNER_LABEL: self.owner, "org.riauth.edition": edition,
                "org.opencontainers.image.revision": self.selected["source"],
                "org.opencontainers.image.source": "https://github.com/" + REPOSITORY},
        }

        def exact(value, expected):
            if type(value) is not type(expected):
                return False
            if type(expected) is dict:
                return value.keys() == expected.keys() and all(
                    exact(value[key], item) for key, item in expected.items())
            if type(expected) is list:
                return len(value) == len(expected) and all(
                    exact(item, wanted) for item, wanted in zip(value, expected))
            return value == expected

        require(exact(settings["config"], expected_config), "image_legacy_config")
        created = settings["created"]
        require(type(created) is str and re.fullmatch(
            r"[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}"
            r"(?:\.[0-9]{0,8}[1-9])?Z", created) is not None, "image_legacy_created")
        try:
            datetime.datetime.strptime(created[:19], "%Y-%m-%dT%H:%M:%S")
        except ValueError:
            raise Refusal("image_legacy_created") from None
        # Pinned Go container.Config declaration order, including mandatory zero fields.
        zero_config = {
            "Hostname": "", "Domainname": "", "User": "",
            "AttachStdin": False, "AttachStdout": False, "AttachStderr": False,
            "Tty": False, "OpenStdin": False, "StdinOnce": False,
            "Env": None, "Cmd": None, "Image": "", "Volumes": None,
            "WorkingDir": "", "Entrypoint": None, "OnBuild": None, "Labels": None,
        }
        runtime_config = {}
        for key, value in zero_config.items():
            runtime_config[key] = expected_config.get(key, value)
            if key == "AttachStderr":
                runtime_config["ExposedPorts"] = expected_config["ExposedPorts"]
            if key == "Cmd":
                runtime_config["ArgsEscaped"] = expected_config["ArgsEscaped"]
        runtime_config["Labels"] = dict(sorted(expected_config["Labels"].items()))

        def encode(value):
            # Only constructed ASCII strings/bools/nulls/maps/lists reach this encoder.
            # Preserve struct order; sorted map fields were constructed explicitly above.
            return json.dumps(value, separators=(",", ":"), allow_nan=False).encode("ascii")

        chain, previous = None, None
        expected_members = set()
        for ordinal, diff_id in enumerate(diff_ids):
            self.check_budget()
            chain = diff_id if chain is None else "sha256:" + digest(
                (chain + " " + diff_id).encode("ascii"))
            top = ordinal + 1 == len(diff_ids)
            pre_id = {"created": created if top else "1970-01-01T00:00:00Z",
                      "container_config": zero_config}
            if top:
                pre_id.update(config=runtime_config, architecture=self.selected["oci_arch"], os="linux")
            id_map = {**pre_id, "layer_id": chain}
            if previous is not None:
                id_map["parent"] = "sha256:" + previous
            # CreateID sorts only its top-level RawMessage map, retaining nested struct order.
            legacy_id = digest(encode(dict(sorted(id_map.items()))))
            saved = {"id": legacy_id}
            if previous is not None:
                saved["parent"] = previous
            saved.update(pre_id)
            # Intermediate OS is added after CreateID; the top already contained it.
            saved["os"] = "linux"
            expected = encode(saved)
            expected_hash = digest(expected)
            name = "blobs/sha256/" + expected_hash
            require(name not in expected_members and name not in ordinary_allowed
                    and name in members and members[name].isfile()
                    and type(members[name].size) is int
                    and members[name].size == len(expected) <= LOG_CAP
                    and hashes.get(name) == expected_hash and payloads.get(name) == expected,
                    "image_legacy_metadata_binding")
            expected_members.add(name)
            previous = legacy_id
        return expected_members

    def inspect_product(self, edition, image):
        # Copy public product bytes from a never-started, controlled container.
        name = self.create_container(image, "product-copy", "/bin/true", [])
        try:
            paths = {"server": "/usr/local/bin/riauth", "license": "/usr/share/doc/riauth/LICENSE",
                     "notices": "/usr/share/doc/riauth/THIRD_PARTY_NOTICES.md"}
            for kind, source in paths.items():
                destination = self.root / (edition + "-image-" + kind)
                code, _, _ = self.docker("copy-product", "container", "cp", name + ":" + source, destination)
                require(code == 0 and destination.is_file() and not destination.is_symlink(), "image_public_file")
                destination.chmod(0o600)
                if kind == "server":
                    self.elf(destination, self.selected["elf_machine"])
                    self.receipt["images"][edition]["server_sha256"] = file_hash(destination)
                else:
                    source_name = "LICENSE" if kind == "license" else "THIRD_PARTY_NOTICES.md"
                    require(file_hash(destination) == self.native["inputs"][source_name], "image_notices_identity")
        finally:
            self.remove_container(name, cleanup=True)
        code, out, _ = self.tool(image, "/usr/local/bin/riauth", ["--json", "capabilities"])
        envelope = json.loads(out)
        require(code == 0 and envelope["ok"] is True, "image_capabilities_status")
        self.capabilities(envelope["data"], edition)
        require(envelope["data"] == self.native_caps[edition], "native_image_capability_parity")
        self.receipt["images"][edition]["capabilities"] = envelope["data"]
        code, server_version, _ = self.tool(image, "/usr/local/bin/riauth", ["--version"])
        require(code == 0 and re.fullmatch(rb"riauth [0-9]+\.[0-9]+\.[0-9]+\s*", server_version), "server_version")
        self.version = server_version.decode().split()[1]
        for product in (self.bins[(edition, "riauth-maintenance")], self.bins[("client", "riauthctl")]):
            code, output, _ = self.command("native-tool-version", [product, "--version"])
            require(code == 0 and output.decode().split() == [product.name, self.version], "cohort_tool_version")

    def build_reader(self):
        """One separate source-pinned release/no-run target; no product image change."""
        self.phase = "reader-build"
        self.capacity(30 * GiB)
        source = self.args.product / "tests/edition_transition_store_probe.rs"
        require(not source.is_symlink() and source.is_file()
                and source.stat().st_size == 5722
                and file_hash(source) == "da486cb8cd7c9d6dfcd78da2704e27688b43a8dd40c7fa58e5b4574f678aed32",
                "reader_source_identity")
        inputs = {
            "Cargo.toml": "58e5ef824ed96290179c9f76fea208dc37173caeee21b6ce8d37f8a6dd1abcb8",
            "Cargo.lock": "b5c9d11c001244b8017303ce8c20516e02946483845eee40720b0910758d4426",
            "rust-toolchain.toml": "887f9be066a15585a2c583578e84b0fcb541126d81546276bad3d2ff00d61167",
            "Dockerfile": DOCKERFILE_HASH,
            ".dockerignore": "4fbd9472316442bdd8b72e268feb1140701e87ac28dd02295c687b6d379eb093",
            "tests/edition_transition_store_probe.rs": file_hash(source),
        }
        for relative, expected in inputs.items():
            require(file_hash(self.args.product / relative) == expected, "reader_input_identity")
        context = self.root / "reader-test"
        context.mkdir(mode=0o700)
        private_write(context / source.name, source.read_bytes())
        # Main .dockerignore stays unchanged. This named context contains exactly one pinned file.
        require(list(context.iterdir()) == [context / source.name], "reader_context_allowlist")
        stage = b"""
FROM build AS reader-build
COPY --from=reader-test /edition_transition_store_probe.rs /build/tests/edition_transition_store_probe.rs
RUN set -eu; test "$RIAUTH_EDITION" = platform; \
    for file in /build/target/release/deps/edition_transition_store_probe-*; do \
      test ! -L "$file"; if test -f "$file" && test -x "$file"; then exit 2; fi; \
    done; \
    CARGO_HOME=/build/cargo-home CARGO_TARGET_DIR=/build/target CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_RELEASE_DEBUG=0 \
      cargo test --release --locked --no-default-features --features platform,test-support --test edition_transition_store_probe --no-run; \
    found=0; picked=; \
    for file in /build/target/release/deps/edition_transition_store_probe-*; do \
      test ! -L "$file"; \
      if test -f "$file" && test -x "$file"; then found=$((found + 1)); picked="$file"; fi; \
    done; \
    test "$found" -eq 1; mkdir -m 0700 /reader-output; \
    cp -- "$picked" /reader-output/riauth-store-probe; chmod 0755 /reader-output/riauth-store-probe

FROM scratch AS reader-export
COPY --from=reader-build /reader-output/riauth-store-probe /riauth-store-probe
"""
        recipe = self.root / "Dockerfile.reader"
        private_write(recipe, self.recipe.read_bytes() + stage)
        exported = self.root / "reader-export"
        exported.mkdir(mode=0o700)
        code, _, _ = self.docker("build-store-reader", "buildx", "build", "--builder", self.builder,
            "--platform", self.selected["platform"], "--progress", "plain", "--file", recipe,
            "--target", "reader-export", "--build-context", "reader-test=" + str(context),
            "--build-arg", "RIAUTH_EDITION=platform",
            "--output", "type=local,dest=" + str(exported), self.args.product,
            timeout=1800)
        require(code == 0 and list(exported.iterdir()) == [exported / "riauth-store-probe"],
                "reader_build_or_export_failed")
        binary = exported / "riauth-store-probe"
        metadata = binary.lstat()
        require(stat.S_ISREG(metadata.st_mode) and not binary.is_symlink()
                and metadata.st_nlink == 1 and metadata.st_uid == os.getuid()
                and 0 < metadata.st_size <= 128 * 1024**2, "reader_artifact_shape")
        # Public source artifact; only the explicitly bound file is accessible to UID10001.
        binary.chmod(0o755)
        self.elf(binary, self.selected["elf_machine"])
        self.reader = binary
        self.reader_hash = file_hash(binary)
        retained = self.evidence / ("local-store-reader-" + self.selected["architecture"] + ".bin")
        private_write(retained, binary.read_bytes())
        require(file_hash(retained) == self.reader_hash
                and stat.S_IMODE(retained.stat().st_mode) == 0o600, "reader_retention_identity")
        self.receipt["reader"] = {
            "schema": "riauth.local-snapshot-reader/v1", "source": self.selected["source"],
            "source_tree": self.selected["tree"], "inputs": inputs,
            "architecture": self.selected["architecture"], "target": self.selected["target"],
            "elf_machine": self.selected["elf_machine"], "features": ["platform", "test-support"],
            "profile": "release", "locked": True, "default_features": False, "build_jobs": 1,
            "incremental": 0, "debug": 0, "compile_no_run": True,
            "test_target": "edition_transition_store_probe", "recipe_sha256": file_hash(recipe),
            "binary_sha256": self.reader_hash, "binary_bytes": metadata.st_size,
            "retained_artifact": retained.name, "native_transport_member": False,
            "action": "snapshot", "verified_physical_observations": 0,
        }
        for relative, expected in inputs.items():
            require(file_hash(self.args.product / relative) == expected, "reader_input_changed")
        require(file_hash(context / source.name) == inputs["tests/edition_transition_store_probe.rs"],
                "reader_context_changed")
        self.save()

    def reader_offline(self, fixture):
        require(fixture["app"] is None, "logical_reader_requires_stopped_owner")
        volumes = {fixture["config_volume"], fixture["data_volume"]}
        for volume in volumes:
            self.confirm_volume(volume)  # Existing owner/CreatedAt/Mountpoint identity.
        for name in list(self.containers):
            item = self.confirm_container(name)
            if any(m["Type"] == "volume" and m["Name"] in volumes for m in item["Mounts"]):
                require(not item["State"]["Running"], "logical_reader_competing_owned_process")

    def reader_io(self, fixture, script):
        self.reader_offline(fixture)
        code, out, _ = self.tool(self.images["platform"], "/bin/sh", ["-ceu", script],
            mounts=[fixture["mounts"][0] + ",readonly", fixture["mounts"][1]], timeout=20)
        require(code == 0, "private_reader_io_failed")
        return out

    def reader_file_identity(self, fixture, filename, empty=False):
        require(re.fullmatch(r"/data/\.a09-reader-[0-9a-f]{32}/snapshot\.json", filename) is not None,
                "reader_closed_output_path")
        out = self.reader_io(fixture, "stat -c '%u:%g:%a:%i:%s:%h:%f' " + filename)
        fields = out.strip().split(b":")
        require(len(fields) == 7 and all(re.fullmatch(rb"[0-9]+", f) for f in fields[:6])
                and re.fullmatch(rb"[0-9a-f]+", fields[6]) is not None, "reader_output_stat")
        uid, gid, mode, inode, size, links = (int(f) for f in fields[:6])
        require(uid == gid == 10001 and mode == 600 and inode > 0 and links == 1
                and stat.S_ISREG(int(fields[6], 16)) and 0 <= size <= LOG_CAP
                and ((size == 0) if empty else (size > 0)), "private_reader_output_shape")
        return uid, gid, mode, inode, links

    @staticmethod
    def parse_reader_snapshot(data):
        require(0 < len(data) <= LOG_CAP, "logical_snapshot_input_cap")

        def pairs(items):
            result = {}
            for key, value in items:
                require(key not in result, "logical_snapshot_duplicate_key")
                result[key] = value
            return result

        def integer(text):
            require(len(text) <= 20, "logical_snapshot_integer_cap")
            return int(text)

        def no_number(text):
            raise Refusal("logical_snapshot_unsupported_number")

        value = json.loads(data, object_pairs_hook=pairs, parse_int=integer,
                           parse_float=no_number, parse_constant=no_number)
        pending, visited = [(value, 0)], 0
        while pending:
            item, depth = pending.pop()
            visited += 1
            require(depth <= 32 and visited <= 100000, "logical_snapshot_structure_cap")
            if type(item) is dict:
                require(all(type(key) is str and len(key.encode("utf-8")) <= 4096 for key in item),
                        "logical_snapshot_key_cap")
                pending.extend((child, depth + 1) for child in item.values())
            elif type(item) is list:
                pending.extend((child, depth + 1) for child in item)
            elif type(item) is str:
                require(len(item.encode("utf-8")) <= 1024**2, "logical_snapshot_value_cap")
            else:
                require(item is None or type(item) in (int, bool), "logical_snapshot_value_type")
        require(type(value) is dict and set(value) == {"row_hashes", "metadata", "counts"},
                "logical_snapshot_schema")
        rows, metadata, counts = value["row_hashes"], value["metadata"], value["counts"]
        require(type(rows) is dict and 0 < len(rows) <= 100000
                and all(type(v) is str and re.fullmatch(r"[A-Za-z0-9_-]{43}", v) for v in rows.values()),
                "logical_snapshot_rows")
        require(type(metadata) is dict and set(metadata) ==
                {"issuer", "node_security", "version_activation", "edition_provenance", "revision"},
                "logical_snapshot_metadata")
        require(type(counts) is dict and set(counts) ==
                {"identities", "credentials", "grants", "revocations"}
                and all(type(v) is int and 0 <= v <= 1000000 for v in counts.values()),
                "logical_snapshot_counts")
        return value

    def logical_snapshot(self, fixture):
        self.reader_offline(fixture)
        require(file_hash(self.reader) == self.reader_hash, "reader_binary_changed")
        before_physical = self.offline_hashes(fixture)
        directory = "/data/.a09-reader-" + uuid.uuid4().hex
        filename = directory + "/snapshot.json"
        self.reader_io(fixture, "umask 077; mkdir -m 0700 " + directory
                       + "; set -C; : > " + filename)
        initial = self.reader_file_identity(fixture, filename, empty=True)
        host_directory = self.root / ("logical-observation-" + uuid.uuid4().hex)
        host_directory.mkdir(mode=0o700)
        host_identity = host_directory.stat()
        destination = host_directory / "snapshot.json"
        require(not destination.exists() and not destination.is_symlink(), "reader_copy_destination_exists")
        mount = "type=bind,src=" + str(self.reader) + ",dst=/cohort/riauth-store-probe,readonly"
        name = self.create_container(self.images["platform"], "snapshot-reader", "/usr/bin/env",
            ["-i", "PATH=/usr/local/bin:/usr/bin:/bin", "HOME=/tmp", "LANG=C.UTF-8",
             "LC_ALL=C.UTF-8", "RUST_BACKTRACE=0",
             "RIAUTH_PROBE_CONFIG=/config/riauth.toml", "RIAUTH_PROBE_ACTION=snapshot",
             "RIAUTH_PROBE_OUTPUT=" + filename, "RIAUTH_PROBE_AGREEMENT=/tmp/unused-agreement",
             "/cohort/riauth-store-probe", "--exact", "isolated_store_probe", "--ignored",
             "--test-threads=1"],
            mounts=[fixture["mounts"][0] + ",readonly", fixture["mounts"][1], mount])
        try:
            result = self.docker("snapshot-reader", "container", "start", "--attach", "--interactive",
                                 name, timeout=90)
            state = self.confirm_container(name)["State"]
            require(not state["Running"] and result[0] == state["ExitCode"] == 0,
                    "snapshot_reader_failed")
            # fs::write truncates the caller-created file; it must not replace its inode.
            require(self.reader_file_identity(fixture, filename) == initial,
                    "reader_output_identity_changed")
            # No --follow-link; copy only the known regular file from the exited owner.
            # Body is file-to-file, never CLI/stdout/public evidence; raw tool errors stay private.
            code, _, _ = self.docker("copy-logical-snapshot", "container", "cp",
                                    name + ":" + filename, destination, timeout=20)
            require(code == 0 and self.reader_file_identity(fixture, filename) == initial,
                    "reader_output_copy_failed")
            parent = host_directory.lstat()
            require(stat.S_ISDIR(parent.st_mode) and parent.st_uid == os.getuid()
                    and stat.S_IMODE(parent.st_mode) == 0o700
                    and (parent.st_dev, parent.st_ino) ==
                    (host_identity.st_dev, host_identity.st_ino), "reader_copy_parent_changed")
            observed = destination.lstat()
            require(stat.S_ISREG(observed.st_mode) and observed.st_uid == os.getuid()
                    and observed.st_gid == os.getgid()
                    and stat.S_IMODE(observed.st_mode) == 0o600 and observed.st_nlink == 1
                    and 0 < observed.st_size <= LOG_CAP, "private_reader_copy_shape")
            fd = os.open(destination, os.O_RDONLY | os.O_NOFOLLOW)
            try:
                opened = os.fstat(fd)
                require((opened.st_dev, opened.st_ino) == (observed.st_dev, observed.st_ino),
                        "reader_copy_open_identity")
                data = bytearray()
                while block := os.read(fd, 65536):
                    self.check_budget()
                    data.extend(block)
                    require(len(data) <= LOG_CAP, "reader_copy_input_cap")
                final = os.fstat(fd)
                current = destination.lstat()
                parent_final = host_directory.lstat()
                require(all(getattr(parent_final, field) == getattr(parent, field)
                            for field in ("st_dev", "st_ino", "st_uid", "st_gid", "st_mode")),
                        "reader_copy_parent_read_identity")
                require(all(getattr(state, field) == getattr(observed, field)
                            for state in (final, current) for field in
                            ("st_dev", "st_ino", "st_uid", "st_gid", "st_mode", "st_nlink", "st_size"))
                        and len(data) == observed.st_size, "reader_copy_read_identity")
            finally:
                os.close(fd)
            value = self.parse_reader_snapshot(bytes(data))
        finally:
            self.remove_container(name, cleanup=True)
        after_physical = self.offline_hashes(fixture)
        self.receipt["reader"]["last_physical_observation"] = {
            "config_equal": before_physical.splitlines()[0] == after_physical.splitlines()[0],
            "key_equal": before_physical.splitlines()[1] == after_physical.splitlines()[1],
            "redb_equal": before_physical.splitlines()[2] == after_physical.splitlines()[2],
        }
        require(after_physical == before_physical, "reader_changed_physical_state")
        self.receipt["reader"]["verified_physical_observations"] += 1
        return value

    def preservation_before(self, fixture, config="riauth.toml"):
        self.reader_offline(fixture)
        require(config in {"riauth.toml", "candidate-issuer.toml", "candidate-policy.toml",
                           "candidate-rate.toml"}, "refusal_closed_config")
        directory = "/data/.a09-preservation-" + uuid.uuid4().hex
        # Exact private byte copies; nothing from config/key is emitted to stdout.
        script = "umask 077; mkdir -m 0700 " + directory + "; set -C; "
        for source, name in (("/config/riauth.toml", "config"), ("/config/database.key", "key"),
                             ("/config/" + config, "candidate")):
            script += "test -f " + source + "; test ! -L " + source + "; cat " + source
            script += " > " + directory + "/" + name + "; "
        self.reader_io(fixture, script)
        modes = self.reader_io(fixture, "stat -c '%u:%g:%a:%h:%F' "
            + directory + "/config " + directory + "/key " + directory + "/candidate")
        require(modes.splitlines() == [b"10001:10001:600:1:regular file"] * 3,
                "private_preservation_copy_modes")
        logical = self.logical_snapshot(fixture)
        return {"directory": directory, "config": config, "logical": logical,
                "physical": self.offline_hashes(fixture)}

    def preservation_after(self, fixture, before, failure):
        directory, config = before["directory"], before["config"]
        require(re.fullmatch(r"/data/\.a09-preservation-[0-9a-f]{32}", directory) is not None
                and config in {"riauth.toml", "candidate-issuer.toml", "candidate-policy.toml",
                               "candidate-rate.toml"}, "preservation_closed_paths")
        # No key/config values in transport: only three fixed cmp exit booleans.
        script = ""
        for source, name in (("/config/riauth.toml", "config"), ("/config/database.key", "key"),
                             ("/config/" + config, "candidate")):
            script += "if cmp -s " + source + " " + directory + "/" + name
            script += "; then printf '1'; else printf '0'; fi; "
        exact = self.reader_io(fixture, script)
        require(re.fullmatch(rb"[01]{3}", exact) is not None, "preservation_cmp_shape")
        physical = self.offline_hashes(fixture)
        a, b = before["logical"], self.logical_snapshot(fixture)
        rows_a, rows_b = a["row_hashes"], b["row_hashes"]
        added, removed = len(rows_b.keys() - rows_a.keys()), len(rows_a.keys() - rows_b.keys())
        changed = sum(rows_a[key] != rows_b[key] for key in rows_a.keys() & rows_b.keys())
        fields_a, fields_b = before["physical"].splitlines(), physical.splitlines()
        require(len(fields_a) == len(fields_b) == 3
                and all(re.fullmatch(rb"[0-9a-f]{64}  /(?:config/riauth\.toml|config/database\.key|data/riauth\.redb)", v)
                        for v in fields_a + fields_b), "preservation_physical_shape")
        packet = {"schema": "riauth.offline-preservation/v1",
            "config_equal": exact[0:1] == b"1", "key_equal": exact[1:2] == b"1",
            "candidate_config_equal": exact[2:3] == b"1",
            "physical_config_equal": fields_a[0] == fields_b[0],
            "physical_key_equal": fields_a[1] == fields_b[1],
            "physical_redb_equal": fields_a[2] == fields_b[2],
            "row_hashes_equal": rows_a == rows_b,
            "metadata_equal": json_bytes(a["metadata"]) == json_bytes(b["metadata"]),
            "counts_equal": a["counts"] == b["counts"],
            "added": added, "removed": removed, "changed": changed}
        packets = self.receipt.setdefault("offline_preservation", [])
        require(len(packets) < 32, "preservation_packet_cap")
        packets.append(packet)
        require(exact == b"111" and packet["physical_config_equal"] and packet["physical_key_equal"]
                and packet["row_hashes_equal"] and packet["metadata_equal"] and packet["counts_equal"],
                failure)

    def maintenance(self, fixture, edition, command, *, config="riauth.toml", input_data=None):
        mount = "type=bind,src=" + str(self.bins[(edition, "riauth-maintenance")])
        mount += ",dst=/cohort/riauth-maintenance,readonly"
        mounts = list(fixture["mounts"])
        if command[0] not in {"init", "keygen"}:
            mounts[0] += ",readonly"
        return self.tool(self.images[edition], "/cohort/riauth-maintenance",
                         ["--config", "/config/" + config, "--json", "--non-interactive", *command],
                         mounts=mounts + [mount], input_data=input_data, timeout=120)

    def shell(self, fixture, script, *, input_data=None):
        # Scripts are fixed controller literals; no user/record content is interpolated.
        code, out, _ = self.tool(self.images["essentials"], "/bin/sh", ["-c", script],
                                 mounts=fixture["mounts"], input_data=input_data)
        require(code == 0, "uid10001_fixture_io_failed")
        return out

    def new_fixture(self, suffix, edition):
        config = self.create_volume(suffix + "-config")
        data = self.create_volume(suffix + "-data")
        for volume in (config, data):
            code, out, _ = self.tool(self.images[edition], "/usr/bin/stat",
                ["-c", "%u:%g:%a", "/data"],
                mounts=["type=volume,src=" + volume + ",dst=/data"])
            match = re.fullmatch(rb"([0-9]+):([0-9]+):([0-7]{3,4})", out.strip())
            if code == 0 and match:
                self.receipt["uid_probes"].append({"volume": volume, "uid": int(match[1]),
                    "gid": int(match[2]), "mode": match[3].decode(), "before_credentials": True})
                self.save()
            require(code == 0 and out.strip() == b"10001:10001:700", "uid_mapping_unsupported")
        with socket.socket() as listener:
            listener.bind(("127.0.0.1", 0))
            port = listener.getsockname()[1]
        fixture = {"base": "http://127.0.0.1:" + str(port), "port": port,
                   "config_volume": config, "data_volume": data,
                   "mounts": ["type=volume,src=" + config + ",dst=/config",
                              "type=volume,src=" + data + ",dst=/data"], "app": None,
                   "password": "Disposable-" + uuid.uuid4().hex}
        code, out, _ = self.maintenance(fixture, edition, ["keygen", "--out", "/config/database.key"])
        require(code == 0 and json.loads(out)["data"]["created"] is True, "keygen_status")
        code, out, _ = self.maintenance(fixture, edition, ["init", "--issuer", fixture["base"],
            "--listen", "127.0.0.1:" + str(port), "--data-dir", "/data",
            "--database-key-file", "/config/database.key", "--password-stdin"],
            input_data=(fixture["password"] + "\n").encode())
        require(code == 0 and json.loads(out)["data"]["initialized"] is True, "first_init_status")
        original = self.shell(fixture, "cat /config/riauth.toml")
        parsed = tomllib.loads(original.decode())
        require(parsed["issuer"] == fixture["base"] and parsed["data_dir"] == "/data"
                and parsed["database_key_file"] == "/config/database.key"
                and parsed.get("postgres") is None, "encrypted_redb_config")
        rates = {**RATE_DEFAULTS, **parsed.get("rate_limits", {})}
        require(set(rates) == set(RATE_DEFAULTS) and all(isinstance(v, int) and v > 0 for v in rates.values()),
                "sixteen_effective_rates")
        modes = self.shell(fixture, "stat -c '%u:%g:%a' /config /data /config/riauth.toml /config/database.key /data/riauth.redb")
        require(modes.splitlines() == [b"10001:10001:700", b"10001:10001:700",
            b"10001:10001:600", b"10001:10001:600", b"10001:10001:600"], "private_fixture_modes")
        fixture["original_config"] = original
        return fixture

    def start_app(self, fixture, edition):
        require(fixture["app"] is None, "writer_already_active")
        with socket.socket() as listener:
            listener.bind(("127.0.0.1", fixture["port"]))  # Never stop a competing listener.
        mounts = [fixture["mounts"][0] + ",readonly", fixture["mounts"][1]]
        name = self.create_container(self.images[edition], "server", "/usr/local/bin/riauth",
                                    ["--config", "/config/riauth.toml", "serve"],
                                    mounts=mounts, network="host")
        fixture["app"] = name
        code, _, _ = self.docker("server-start", "container", "start", name)
        require(code == 0, "server_start_failed")
        end = min(time.monotonic() + 45, self.fixture_deadline)
        while time.monotonic() < end:
            self.check_budget()
            require(self.confirm_container(name)["State"]["Running"], "server_exited_before_ready")
            try:
                status, _, _ = self.http(fixture["base"] + "/readyz", cap=65536)
                if status == 200:
                    return
            except (urllib.error.URLError, TimeoutError, ConnectionError):
                pass
            time.sleep(0.2)
        raise Refusal("server_readiness_deadline")

    def stop_app(self, fixture):
        if fixture["app"] is not None:
            self.remove_container(fixture["app"], cleanup=True)
            fixture["app"] = None

    def client(self, fixture, session, command, *, input_data=None, expected=0):
        argv = [self.bins[("client", "riauthctl")], "--server", fixture["base"],
                "--session-file", session, "--json", "--non-interactive", "--request-timeout", "30", *command]
        code, out, _ = self.command("base-client", argv, input_data=input_data, timeout=60)
        envelope = json.loads(out)
        require(code == expected and envelope["schema_version"] == "riauth.cli/v1", "client_status")
        if expected == 0:
            require(envelope["ok"] is True, "client_success_envelope")
            return envelope["data"]
        require(envelope["ok"] is False and envelope["error"]["http_status"]
                == {3: 401, 4: 403}[expected] and envelope["exit_code"] == expected,
                "client_refusal_is_server_status")
        return envelope

    def login(self, fixture, session, username, password):
        return self.client(fixture, session, ["login", username, "--password-stdin"],
                           input_data=(password + "\n").encode())

    def mutate(self, fixture, session, command, *, input_data=None, expected=0):
        revision = self.client(fixture, fixture["admin_session"], ["revision"])["revision"]
        require(isinstance(revision, int) and revision >= 0, "revision_type")
        return self.client(fixture, session, ["--if-revision", str(revision),
            "--idempotency-key", "a09-" + uuid.uuid4().hex, *command],
            input_data=input_data, expected=expected)

    def token(self, fixture, session):
        value = json.loads(session.read_bytes())
        require(stat.S_IMODE(session.stat().st_mode) == 0o600
                and value["issuer"] == fixture["base"] and value["api_base"] == fixture["base"]
                and isinstance(value["token"], str) and value["token"], "issuer_bound_private_session")
        return value["token"]

    def revoked(self, fixture, session):
        self.client(fixture, session, ["whoami"], expected=3)
        status, _, _ = self.http(fixture["base"] + "/api/me", token=self.token(fixture, session))
        require(status == 401, "revoked_session_http401")

    def snapshot(self, fixture):
        session = fixture["delegate_session"]
        self.login(fixture, fixture["admin_session"], "admin", fixture["password"])
        self.login(fixture, session, "delegate", fixture["delegate_password"])
        discovery = self.client(fixture, session, ["discovery"])
        require(discovery["issuer"] == fixture["base"] and discovery["jwks_uri"]
                == fixture["base"] + "/oauth/jwks", "exact_discovery")
        status, jwks, _ = self.http(discovery["jwks_uri"])
        require(status == 200, "jwks_status")
        me = self.client(fixture, session, ["whoami"])
        require(set(me["user"]) == USER_FIELDS and me["user"]["admin"] is False
                and me["user"]["password_available"] is True, "complete_ordinary_user")
        user = self.client(fixture, fixture["admin_session"], ["user", "get", "delegate"])
        group = self.client(fixture, fixture["admin_session"], ["group", "get", "local-team"])
        grants = self.client(fixture, fixture["admin_session"], ["grants", "get", "delegate"])
        require(user == me["user"] and group["members"] == [user["id"]]
                and "local-team" in me["groups"] and grants["grants"]
                == [{"role": "auditor", "scope": "audit/events", "target_id": "events"}],
                "group_grant_subject_binding")
        status, _, _ = self.http(fixture["base"] + "/api/audit?limit=1", token=self.token(fixture, session))
        require(status == 200, "audit_events_allowed")
        users_before = self.client(fixture, fixture["admin_session"], ["user", "list"])
        self.mutate(fixture, session, ["user", "create", "forbidden-child", "--password-stdin"],
            input_data=("Denied-" + uuid.uuid4().hex + "\n").encode(), expected=4)
        require(self.client(fixture, fixture["admin_session"], ["user", "list"]) == users_before,
                "denied_user_creation_wrote_nothing")
        for saved in fixture.get("revoked", []):
            self.revoked(fixture, saved)
        # All 12 UserView fields and every me field except fresh session ID/expiry.
        stable_me = {k: v for k, v in me.items() if k not in {"session_id", "expires_at"}}
        return {"me": stable_me, "user": user, "group": group, "grants": grants,
                "discovery": {k: discovery[k] for k in ("issuer", "jwks_uri")}, "jwks": json.loads(jwks)}

    def revoke_copy(self, fixture):
        session = self.root / ("revoking-session-" + uuid.uuid4().hex + ".json")
        saved = self.root / ("revoked-session-" + uuid.uuid4().hex + ".json")
        self.login(fixture, session, "delegate", fixture["delegate_password"])
        private_write(saved, session.read_bytes())
        self.client(fixture, session, ["logout"])
        require(not session.exists(), "logout_session_removed")
        fixture.setdefault("revoked", []).append(saved)
        self.revoked(fixture, saved)

    def offline_hashes(self, fixture):
        require(fixture["app"] is None, "offline_writer_required")
        # Comparison only in memory; neither key hash nor raw data is evidence.
        return self.shell(fixture, "sha256sum /config/riauth.toml /config/database.key /data/riauth.redb")

    def direct_refusal(self, fixture, edition, config="riauth.toml", marker=None):
        require(fixture["app"] is None, "direct_refusal_requires_stopped_writer")
        before = self.preservation_before(fixture, config)
        code, out, _ = self.tool(self.images[edition], "/usr/local/bin/riauth",
            ["--config", "/config/" + config, "--json", "serve"],
            mounts=[fixture["mounts"][0] + ",readonly", fixture["mounts"][1]], timeout=20)
        envelope = json.loads(out)
        require(code == 2 and envelope["ok"] is False, "direct_open_refusal")
        if marker is not None:
            require(marker in envelope["error"]["message"], "specific_direct_refusal")
        self.preservation_after(fixture, before, "refused_open_changed_store_or_config")

    def plan(self, fixture, target, *, config="riauth.toml", expected=0):
        code, out, _ = self.maintenance(fixture, "platform", ["transition-plan", "--target", target],
                                        config=config)
        envelope = json.loads(out)
        require(code == expected and envelope["schema_version"] == "riauth.cli/v1"
                and envelope["ok"] == (expected == 0) and envelope["exit_code"] == expected
                and envelope["data"]["ready"] == (expected == 0)
                and envelope["data"]["read_only"] is True, "transition_plan_envelope")
        return envelope["data"]

    def config_refusals(self, fixture):
        original = fixture["original_config"]
        parsed = tomllib.loads(original.decode())
        specifications = (
            ("issuer", "meta/issuer", "Configured issuer does not match the initialized instance"),
            ("policy", "meta/node_security", "Configured token lifetimes or password policy do not match the initialized instance"),
            ("rate", "meta/node_security", "Configured HTTP rate limit for login"),
        )
        before = self.preservation_before(fixture)
        for name, resource, reason in specifications:
            text = original.decode()
            wanted = json.loads(json.dumps(parsed))
            if name == "issuer":
                wanted["issuer"] = fixture["base"] + "/different"
                text, count = re.subn(r'(?m)^issuer = "[^"\n]+"$',
                    "issuer = " + json.dumps(wanted["issuer"]), text)
            elif name == "policy":
                wanted["session_ttl"] += 1
                text, count = re.subn(r"(?m)^session_ttl = [0-9]+$",
                    "session_ttl = " + str(wanted["session_ttl"]), text)
            else:
                wanted.setdefault("rate_limits", {})["login"] = RATE_DEFAULTS["login"] + 1
                require("login" not in parsed.get("rate_limits", {}), "fresh_default_rate_expected")
                if "rate_limits" not in parsed:
                    text += "\n[rate_limits]\nlogin = 21\n"
                    count = 1  # Empty maps are omitted by Config serialization.
                else:
                    text, count = re.subn(r"(?m)^\[rate_limits\]$", "[rate_limits]\nlogin = 21", text)
            require(count == 1 and tomllib.loads(text) == wanted, "candidate_exact_one_field")
            filename = "candidate-" + name + ".toml"
            # Closed filenames above; exclusive creation as the same unprivileged owner.
            self.shell(fixture, "umask 077; set -C; cat > /config/" + filename,
                       input_data=text.encode())
            plan_before = self.offline_hashes(fixture)
            report = self.plan(fixture, "platform", config=filename, expected=5)
            require(any(b["resource"] == resource and reason in b["reason"] for b in report["blockers"]),
                    "specific_config_plan_blocker")
            require(self.offline_hashes(fixture) == plan_before, "read_only_config_plan_changed_store")
            self.direct_refusal(fixture, "essentials", config=filename, marker=reason)
            self.preservation_after(fixture, before, "config_refusal_changed_original")
        require(self.shell(fixture, "cat /config/riauth.toml") == original, "original_config_preserved")

    def handoff(self, fixture, target):
        require(fixture["app"] is None, "handoff_writer_required_stopped")
        before = self.offline_hashes(fixture)
        report = self.plan(fixture, target)
        require(report["target_edition"] == target and isinstance(report["transition_token"], str)
                and report["transition_token"], "exact_plan_token")
        require(self.offline_hashes(fixture) == before, "read_only_plan_changed_store")
        code, out, _ = self.maintenance(fixture, "platform", ["transition-activate", "--target", target,
            "--token", report["transition_token"]])
        require(code == 0 and json.loads(out)["ok"] is True, "explicit_activation_status")
        require(self.shell(fixture, "cat /config/riauth.toml") == fixture["original_config"],
                "activation_changed_config")
        require(self.offline_hashes(fixture).splitlines()[:2] == before.splitlines()[:2],
                "activation_changed_config_or_key")

    def platform_refusal_fixture(self):
        fixture = self.new_fixture("platform-state", "platform")
        self.start_app(fixture, "platform")
        session = self.root / "platform-state-admin.json"
        try:
            self.login(fixture, session, "admin", fixture["password"])
            bearer = self.token(fixture, session)
            status, body, _ = self.http(fixture["base"] + "/api/state/revision", token=bearer)
            require(status == 200, "agent_revision_status")
            revision = json.loads(body)["revision"]
            status, _, _ = self.http(fixture["base"] + "/api/agents", method="POST", token=bearer,
                data={"id": "a09-platform-state", "ttl": 3600,
                      "permissions": [{"action": "user.offboard", "resource": "*"}]},
                extra_headers={"If-Match": '"' + str(revision) + '"',
                               "Idempotency-Key": "a09-agent-" + uuid.uuid4().hex})
            require(status == 200, "platform_agent_status")
        finally:
            self.stop_app(fixture)
        before = self.preservation_before(fixture)
        self.direct_refusal(fixture, "essentials", marker="Platform")
        plan_before = self.offline_hashes(fixture)
        code, out, _ = self.maintenance(fixture, "platform", ["transition-preflight", "--target", "essentials"])
        envelope = json.loads(out)
        require(code == 5 and envelope["ok"] is False and envelope["data"]["ready"] is False
                and any(b["resource"].startswith("agents/") and "Platform build" in b["reason"]
                        for b in envelope["data"]["blockers"])
                and any(b["resource"] == "meta/edition_provenance" for b in envelope["data"]["blockers"]),
                "platform_state_downgrade_blockers")
        require(self.offline_hashes(fixture) == plan_before, "read_only_preflight_changed_store")
        self.preservation_after(fixture, before, "platform_refusal_changed_store")

    def fixture_gate(self):
        self.phase = "fixture"
        self.capacity(30 * GiB)
        self.fixture_deadline = time.monotonic() + 1200
        fixture = self.new_fixture("clean", "essentials")
        fixture.update(admin_session=self.root / "admin-session.json",
                       delegate_session=self.root / "delegate-session.json",
                       delegate_password="Delegate-" + uuid.uuid4().hex)
        self.start_app(fixture, "essentials")
        try:
            self.login(fixture, fixture["admin_session"], "admin", fixture["password"])
            self.mutate(fixture, fixture["admin_session"], ["user", "create", "delegate", "--password-stdin"],
                         input_data=(fixture["delegate_password"] + "\n").encode())
            self.mutate(fixture, fixture["admin_session"], ["group", "create", "local-team"])
            self.mutate(fixture, fixture["admin_session"], ["group", "add-member", "local-team", "delegate"])
            grants = self.root / "grants.json"
            private_write(grants, json_bytes([{"role": "auditor", "scope": "audit/events"}]))
            self.mutate(fixture, fixture["admin_session"], ["grants", "set", "delegate", "--file", grants])
            self.revoke_copy(fixture)
            baseline = self.snapshot(fixture)
        finally:
            self.stop_app(fixture)
        self.config_refusals(fixture)
        self.direct_refusal(fixture, "platform", marker="Configured active capabilities")
        self.handoff(fixture, "platform")
        self.start_app(fixture, "platform")
        try:
            require(self.snapshot(fixture) == baseline, "platform_identity_authorization_config_changed")
            self.revoke_copy(fixture)
        finally:
            self.stop_app(fixture)
        self.direct_refusal(fixture, "essentials", marker="Platform")
        self.handoff(fixture, "essentials")
        self.start_app(fixture, "essentials")
        try:
            require(self.snapshot(fixture) == baseline, "returned_identity_authorization_config_changed")
        finally:
            self.stop_app(fixture)
        self.platform_refusal_fixture()
        self.receipt["checks"] = ["uid10001_copy_up_private_modes", "encrypted_redb_key_configured",
            "pinned_format3_startup_and_sixteen_rates", "exact_discovery_and_jwks",
            "all12_userview_fields_and_group_grant_subject", "audit_events200_user_create403_client4",
            "two_logout401_client3_across_handoffs", "issuer_policy_loginrate_specific_offline_refusals",
            "read_only_plan_exact_token_EPE", "wrong_direct_build_refused",
            "isolated_platform_agent_and_downgrade_refusal"]
        self.fixture_deadline = None

    def cleanup(self):
        """Only verified exact creation identities, including daemon children."""
        self.phase = "cleanup"
        self.fixture_deadline = None
        cleanup_deadline = time.monotonic() + 240
        self.cleanup_deadline = cleanup_deadline
        # Commands below have independent small cleanup bounds, not a runtime retry.
        for name in list(self.containers):
            try:
                require(time.monotonic() < cleanup_deadline, "cleanup_deadline")
                self.remove_container(name, cleanup=True)
            except BaseException:
                self.receipt["cleanup_errors"].append("owned_container_cleanup_blocked")
        if self.builder_attempted:
            try:
                self.recover_builder()
                require(time.monotonic() < cleanup_deadline, "cleanup_deadline")
                if self.builder_id is not None:
                    code, _, _ = self.docker("builder-stop-remove", "container", "rm", "--force",
                                              self.builder_id, timeout=30, cleanup=True)
                    require(code == 0, "builder_container_cleanup_failed")
                    current = self.inspect("volume", self.builder_volume, cleanup=True)
                    require(all(current.get(k) == self.builder_volume_record.get(k)
                                for k in ("Name", "Driver", "Mountpoint", "CreatedAt", "Labels", "Options")),
                            "builder_volume_replaced")
                    code, _, _ = self.docker("builder-volume-remove", "volume", "rm", self.builder_volume,
                                              timeout=30, cleanup=True)
                    require(code == 0, "builder_volume_cleanup_failed")
                    # No global builder/cache operation; the instance is bound to private config.
                    code, _, _ = self.docker("builder-instance-remove", "buildx", "rm", "--keep-state",
                                              "--force", self.builder, timeout=30, cleanup=True)
                    require(code == 0, "builder_instance_cleanup_failed")
                    self.prove_absent("container", self.builder_id, cleanup=True)
                    self.prove_absent("volume", self.builder_volume, cleanup=True)
                else:
                    self.prove_absent("container", self.builder_container, cleanup=True)
                    self.prove_absent("volume", self.builder_volume, cleanup=True)
                self.receipt["builder_children_removed"] = True
            except BaseException:
                self.receipt["cleanup_errors"].append("builder_cleanup_identity_blocked")
        for name in list(self.volumes):
            try:
                require(time.monotonic() < cleanup_deadline, "cleanup_deadline")
                if self.confirm_volume(name, missing_ok=True) is None:
                    del self.volumes[name]
                    continue
                code, _, _ = self.docker("volume-remove", "volume", "rm", name, timeout=30, cleanup=True)
                require(code == 0, "volume_cleanup_failed")
                self.prove_absent("volume", name, cleanup=True)
                del self.volumes[name]
            except BaseException:
                self.receipt["cleanup_errors"].append("owned_volume_cleanup_blocked")
        for edition, tag in (self.tags.items() if self.daemon_seen else []):
            try:
                require(time.monotonic() < cleanup_deadline, "cleanup_deadline")
                code, out, _ = self.docker("cleanup-image-inspect", "image", "inspect", tag,
                                          timeout=20, cleanup=True)
                if code == 0:
                    item = json.loads(out)[0]
                    require((item["Config"].get("Labels") or {}).get(OWNER_LABEL) == self.owner
                            and item["Config"]["Labels"]["org.riauth.edition"] == edition
                            and item["Config"]["Labels"]["org.opencontainers.image.revision"] == self.selected["source"]
                            and item["RepoTags"] == [tag], "image_cleanup_identity")
                    code, _, _ = self.docker("owned-image-remove", "image", "rm", "--no-prune", tag,
                                              timeout=60, cleanup=True)
                    require(code == 0, "owned_image_remove_failed")
                self.prove_absent("image", tag, cleanup=True)
                if edition in self.images:
                    self.prove_absent("image", self.images[edition], cleanup=True)
            except BaseException:
                self.receipt["cleanup_errors"].append("owned_image_cleanup_blocked")
        try:
            if not self.daemon_seen:
                require(not self.containers and not self.volumes and not self.builder_attempted,
                        "unverified_daemon_resources")
                self.receipt["docker_cleanup"] = "no_docker_resource_creation_attempted"
            else:
                code, out, _ = self.docker("owned-image-inventory", "image", "ls", "--quiet", "--no-trunc",
                    "--filter", "label=" + OWNER_LABEL + "=" + self.owner, timeout=20, cleanup=True)
                require(code == 0 and not out.strip(), "owned_image_inventory_not_empty")
                self.receipt["owned_image_inventory_empty"] = True
        except BaseException:
            self.receipt["cleanup_errors"].append("owned_image_inventory_unproven")
        with self.lock:
            groups = list(self.active)
        with self.lock:
            for group in groups:
                self.kill_group(group)  # Refuses a missing/reused creation identity.
        self.receipt["remaining_owned_containers"] = len(self.containers)
        self.receipt["remaining_owned_volumes"] = len(self.volumes)
        self.receipt["remaining_owned_cli_groups"] = len(self.active)
        if self.active or self.containers or self.volumes:
            self.receipt["cleanup_errors"].append("owned_resources_remain")
        self.receipt["retained_inputs"] = "content-addressed tool/base image layers; no shared deletion"
        self.monitor_end.set()
        self.monitor.join(timeout=5)
        if self.monitor.is_alive() or self.resource_error:
            self.receipt["cleanup_errors"].append("resource_monitor_failed")
        # Remove only private inputs/sessions/logless fixture state; public evidence remains.
        for child in self.root.iterdir():
            if child == self.evidence:
                continue
            if child.is_dir() and not child.is_symlink():
                shutil.rmtree(child)
            else:
                child.unlink()
        self.save()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("controller", "product", "review", "root"):
        parser.add_argument("--" + name, type=pathlib.Path, required=True)
    args = parser.parse_args()
    os.umask(0o077)
    gate = None
    failure = None

    def interrupted(signum, frame):
        raise Refusal("controller_signal")

    for sig in (signal.SIGTERM, signal.SIGINT):
        signal.signal(sig, interrupted)
    try:
        gate = Cohort(args)
        gate.source_check()
        gate.native_transport()
        gate.daemon_setup()
        gate.build_images()
        gate.build_reader()
        gate.fixture_gate()
        # Recheck committed source and manifests after every product operation.
        require(gate.git(args.product, "status", "--porcelain") == b"", "product_checkout_changed")
        for relative, expected in gate.native["inputs"].items():
            require(file_hash(args.product / relative) == expected, "product_input_changed")
        gate.receipt["result"] = "passed"
    except BaseException as error:
        # Never format an external exception: it could contain a signed URL/token/reply.
        failure = str(error) if isinstance(error, Refusal) else "unexpected_" + type(error).__name__
        if gate is not None:
            gate.receipt.update(result="failed_or_refused", failure=failure, failed_phase=gate.phase)
            if type(error) is OSError:
                try:
                    sites = {
                        Cohort.start_app.__code__: "start_app",
                        Cohort.create_container.__code__: "container_create",
                        Cohort.command.__code__: "owned_cli_transport",
                    }
                    node, site = error.__traceback__, "other"
                    for _ in range(16):
                        if node is None:
                            break
                        site = sites.get(node.tb_frame.f_code, "other")
                        node = node.tb_next
                    if node is not None:
                        site = "other"
                    number = error.errno
                    gate.receipt["os_error"] = {
                        "site": site,
                        "errno": number if type(number) is int and 0 <= number <= 4095 else None,
                    }
                except BaseException:
                    pass  # Optional projection cannot replace the original failure.
    finally:
        if gate is not None:
            try:
                gate.cleanup()
            except BaseException:
                gate.receipt["cleanup_errors"].append("private_cleanup_failed")
                failure = failure or "private_cleanup_failed"
            if gate.receipt["cleanup_errors"]:
                gate.receipt["result"] = "failed_or_refused"
                failure = failure or "owned_cleanup_failed"
            try:
                gate.save()
            except BaseException:
                failure = failure or "receipt_write_failed"
    print(json.dumps({"result": "failed_or_refused" if failure else "passed",
                      "failure": failure, "official_release": False}))
    return 1 if failure else 0


if __name__ == "__main__":
    raise SystemExit(main())
