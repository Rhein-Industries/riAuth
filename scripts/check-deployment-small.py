#!/usr/bin/env python3
"""Opt-in, disposable small-template mechanics check; never a release/HA gate.

--check-source only reads pinned Git objects and renders Compose locally. It
does not contact the Docker engine. Normal mode starts local fixture containers;
run it only after the exact command/source has been reviewed and authorized.
No raw child output, credentials, config contents or sessions enter evidence.
"""

import argparse
import hashlib
import json
import os
import re
import secrets
import selectors
import shutil
import signal
import socket
import subprocess
import sys
import tempfile
import time
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PROJECT = "891e7443-8dac-4c1b-897f-9e53cb59c7ee"
TASK = "6c981199-62dd-464d-a12a-4ce4e27f428f"
TEMPLATE_REVISION = "60437b59933cadd40a1f5fbbb91ba153aee56456"
EQUIVALENT_REVISION = "c01c39ab4e092423d5522bedc50fff87656d8c0a"
TEMPLATE_BLOB = "a75307cde81782907014e97fb607884c2d0e731b"
TEMPLATE_SHA256 = "2db9060a4bbd8dd895401f5e58f2653ca3b774abf8e1867b9a41a52c0ae0cea6"
IMAGE = "sha256:ae84172a41d2dbe607a581d48fcde79cd1d293e00244994e257a757fd6def842"
IMAGE_REVISION = "f3aba63ac3b824843a40b99623f1619ef8edc19f"
PROBE_IMAGE = "sha256:9d2e5553305c7c7b0097999bb17187c69b921ccd6bc9d40e4bb5ebe652c00285"
MANIFEST = "docs/roadmap/evidence/a08-native-linux-arm64-2026-09-29/native-arm64-evidence-v2.json"
OUTPUT_LIMIT = 262144  # combined stdout/stderr, including discarded diagnostics
INPUT_LIMIT = 65536
LABEL = "org.riauth.o07"
INTERRUPTED = False


class Refusal(Exception):
    """Fixed, secret-free failure identifier; never embeds subprocess output."""


def require(condition, code):
    if not condition:
        raise Refusal(code)


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


def digest(data):
    return hashlib.sha256(data).hexdigest()


def decode(data):
    def unique(pairs):
        result = {}
        for key, value in pairs:
            require(key not in result, "duplicate_json_key")
            result[key] = value
        return result

    try:
        return json.loads(data, object_pairs_hook=unique)
    except (ValueError, UnicodeError):
        raise Refusal("invalid_json") from None


def interrupted(_signum, _frame):
    global INTERRUPTED
    INTERRUPTED = True


class Runner:
    def __init__(self, seconds, environment=None):
        self.end = time.monotonic() + seconds
        self.environment = environment

    def call(self, argv, *, input_bytes=b"", timeout=15, cleanup=False):
        require(len(input_bytes) <= INPUT_LIMIT, "child_input_limit")
        require(cleanup or shutil.disk_usage(ROOT / "target").free >= 8 * 1024**3, "disk_floor")
        deadline = min(self.end, time.monotonic() + timeout)
        require(deadline > time.monotonic(), "deadline")
        require(cleanup or not INTERRUPTED, "cancelled")
        try:
            process = subprocess.Popen(
                argv, cwd=ROOT, env=self.environment, stdin=subprocess.PIPE,
                stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                start_new_session=True,
            )
        except OSError:
            raise Refusal("child_unavailable") from None
        selector = selectors.DefaultSelector()
        output = bytearray()
        total = 0
        pending = memoryview(input_bytes)
        try:
            for stream in (process.stdout, process.stderr):
                os.set_blocking(stream.fileno(), False)
                selector.register(stream, selectors.EVENT_READ)
            if pending:
                os.set_blocking(process.stdin.fileno(), False)
                selector.register(process.stdin, selectors.EVENT_WRITE)
            else:
                process.stdin.close()
            while selector.get_map():
                require(cleanup or not INTERRUPTED, "cancelled")
                remaining = deadline - time.monotonic()
                require(remaining > 0, "child_deadline")
                for key, event in selector.select(min(remaining, 0.1)):
                    stream = key.fileobj
                    if event & selectors.EVENT_WRITE:
                        try:
                            written = os.write(stream.fileno(), pending[:16384])
                            pending = pending[written:]
                        except BrokenPipeError:
                            pending = pending[:0]
                        except BlockingIOError:
                            continue
                        if not pending:
                            selector.unregister(stream)
                            stream.close()
                    else:
                        try:
                            chunk = os.read(stream.fileno(), 16384)
                        except BlockingIOError:
                            continue
                        if not chunk:
                            selector.unregister(stream)
                            stream.close()
                            continue
                        total += len(chunk)
                        require(total <= OUTPUT_LIMIT, "child_output_limit")
                        if stream is process.stdout:
                            output.extend(chunk)
            remaining = deadline - time.monotonic()
            require(remaining > 0, "child_deadline")
            try:
                status = process.wait(timeout=remaining)
            except subprocess.TimeoutExpired:
                raise Refusal("child_deadline") from None
            require(time.monotonic() < deadline, "child_deadline_after_join")
            require(cleanup or not INTERRUPTED, "cancelled")
            return status, bytes(output)
        finally:
            # Also kill escaped pipe holders in the CLI's process group on error.
            if process.poll() is None or selector.get_map():
                try:
                    os.killpg(process.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
            for stream in (process.stdin, process.stdout, process.stderr):
                stream.close()
            selector.close()
            try:
                process.wait(timeout=0.5)
            except subprocess.TimeoutExpired:
                pass  # caller fails; remote fixture cleanup is separate below

    def success(self, argv, **options):
        status, output = self.call(argv, **options)
        require(status == 0, "child_command_failed")
        return output


def private_file(path, data):
    descriptor = os.open(path, os.O_CREAT | os.O_EXCL | os.O_WRONLY | os.O_NOFOLLOW, 0o600)
    with os.fdopen(descriptor, "wb") as output:
        output.write(data)


def source_proof(runner):
    path = "deploy/compose-small.yml"
    blobs = []
    for revision in (TEMPLATE_REVISION, EQUIVALENT_REVISION):
        blob = runner.success(["git", "rev-parse", f"{revision}:{path}"]).decode().strip()
        require(blob == TEMPLATE_BLOB, "template_blob_changed")
        blobs.append(blob)
    template = runner.success(["git", "show", f"{TEMPLATE_REVISION}:{path}"])
    require(digest(template) == TEMPLATE_SHA256, "template_digest_changed")
    manifest_bytes = runner.success(["git", "show", f"{TEMPLATE_REVISION}:{MANIFEST}"])
    manifest = decode(manifest_bytes)
    image = manifest["local_images"]["essentials"]
    require(manifest["binary_source_commit"] == IMAGE_REVISION
            and image["image_id"] == IMAGE and image["revision"] == IMAGE_REVISION
            and image["platform"] == "linux/arm64" and image["edition"] == "essentials"
            and manifest["release_gate_result"] is False, "local_image_manifest_mismatch")
    return template, {
        "harness_sha256": digest(Path(__file__).read_bytes()),
        "harness_git_blob": runner.success(["git", "hash-object", str(Path(__file__))]).decode().strip(),
        "execution_head": runner.success(["git", "rev-parse", "HEAD"]).decode().strip(),
        "template_revision": TEMPLATE_REVISION, "equivalent_revision": EQUIVALENT_REVISION,
        "template_blob": blobs[0], "template_sha256": digest(template),
        "manifest_sha256": digest(manifest_bytes), "image": IMAGE,
        "image_source_revision": IMAGE_REVISION, "edition": "essentials",
        "probe_image": PROBE_IMAGE, "artifact_kind": "local-source",
        "current_source_binary_verified": False, "release_verified": False,
    }


def labels(token):
    return {LABEL + ".project": PROJECT, LABEL + ".task": TASK, LABEL + ".run": token}


def overlay(config, issuer, token):
    services = {}
    for service in ("riauth", "init"):
        services[service] = {
            "image": IMAGE, "pull_policy": "never", "labels": labels(token),
            "volumes": [{"type": "bind", "source": str(config), "target": "/config",
                         "read_only": service == "riauth",
                         "bind": {"create_host_path": False}}],
        }
    services["riauth"]["healthcheck"] = {"test": [
        "CMD", "/usr/local/bin/riauth", "--server", issuer, "--session-file",
        "/tmp/riauth-health-session", "--request-timeout", "5", "status",
    ]}
    return {"services": services, "volumes": {"riauth-data": {"labels": labels(token)}},
            "networks": {"default": {"labels": labels(token)}}}


def render_proof(runner, root, project, config, issuer, token, template):
    base = root / "template.yml"
    patch = root / "fixture.json"
    empty_env = root / "empty.env"
    private_file(base, template)
    private_file(patch, canonical(overlay(config, issuer, token)))
    private_file(empty_env, b"")
    prefix = ["docker", "compose", "--project-name", project, "--project-directory",
              str(root), "--env-file", str(empty_env), "--profile", "tools", "-f", str(base)]
    # Compose config is a local render, not an engine operation. Empty env file
    # and private project directory prevent accidental checkout .env ingestion.
    original = decode(runner.success(prefix + ["config", "--format", "json"]))
    rendered = decode(runner.success(prefix + ["-f", str(patch), "config", "--format", "json"]))
    require(set(original["services"]) == {"riauth", "init"}
            and set(original["volumes"]) == {"riauth-data"}
            and set(original["networks"]) == {"default"}, "template_topology_changed")
    expected = decode(canonical(original))
    for name, service in expected["services"].items():
        require(service["user"] == "10001:10001"
                and service["entrypoint"] == ["/usr/local/bin/riauth", "--config", "/config/riauth.toml"],
                "template_execution_changed")
        mounts = {mount["target"]: mount for mount in service["volumes"]}
        require(set(mounts) == {"/config", "/data"}
                and mounts["/data"]["type"] == "volume"
                and mounts["/data"]["source"] == "riauth-data"
                and mounts["/config"]["type"] == "bind"
                and mounts["/config"]["bind"]["create_host_path"] is False,
                "template_mount_changed")
        mounts["/config"]["source"] = str(config)
        service["labels"] = labels(token)
        service["pull_policy"] = "never"
    service = expected["services"]["riauth"]
    require(service["network_mode"] == "host" and service["read_only"] is True
            and service["security_opt"] == ["no-new-privileges:true"]
            and service["command"] == ["serve"]
            and service["restart"] == "unless-stopped"
            and service["stop_grace_period"] == "30s"
            and service["healthcheck"]["timeout"] == "8s"
            and service["healthcheck"]["interval"] == "15s"
            and service["healthcheck"]["start_period"] == "30s"
            and service["healthcheck"]["retries"] == 3
            and service["tmpfs"] == ["/tmp:rw,nosuid,nodev,size=64m"], "template_safety_changed")
    require(expected["services"]["init"]["profiles"] == ["tools"], "tools_profile_changed")
    require(next(m for m in service["volumes"] if m["target"] == "/config")["read_only"] is True
            and not next(m for m in expected["services"]["init"]["volumes"]
                         if m["target"] == "/config").get("read_only", False), "config_mode_changed")
    service["healthcheck"]["test"][3] = issuer
    expected["volumes"]["riauth-data"]["labels"] = labels(token)
    expected["networks"]["default"]["labels"] = labels(token)
    require(rendered == expected, "unapproved_rendered_change")
    # Replace only generated identity/path/port fields for a reproducible digest.
    structural = canonical(rendered).decode().replace(str(config), "<FIXTURE>")
    structural = structural.replace(issuer, "<LOOPBACK_ISSUER>").replace(project, "<PROJECT>")
    structural = structural.replace(token, "<RUN>")
    proof = {"rendered_structural_sha256": digest(structural.encode()),
             "allowed_changes": ["fixture_bind_source", "healthcheck_loopback_url",
                                 "project_name", "ownership_labels", "image_pull_never"],
             "host_network_preserved": True, "one_volume": True,
             "uid_10001_preserved": True, "read_only_preserved": True,
             "tools_profile_preserved": True, "no_new_privileges_preserved": True}
    return prefix + ["-f", str(patch)], rendered, proof


PROBE = r'''
import hashlib,json,shutil,socket,sys,urllib.error,urllib.request
data=json.loads(sys.stdin.buffer.read(65537))
if data["kind"]=="port":
    with socket.socket() as s:
        s.bind(("127.0.0.1",0)); print(json.dumps({"port":s.getsockname()[1],"free_bytes":shutil.disk_usage("/").free}))
else:
    class NoRedirect(urllib.request.HTTPRedirectHandler):
        def redirect_request(self,*args,**kwargs): return None
    opener=urllib.request.build_opener(urllib.request.ProxyHandler({}),NoRedirect())
    path=data["path"]
    if path not in ["/livez","/readyz","/healthz","/.well-known/openid-configuration","/oauth/jwks"]:
        raise RuntimeError("unapproved_probe_path")
    try:
        with opener.open(data["issuer"]+path,timeout=2) as r:
            raw=r.read(65537)
            if len(raw)>65536: raise RuntimeError("probe_response_limit")
            body=json.loads(raw)
            out={"status":r.status,"issuer_matches":body.get("issuer")==data["issuer"]}
            if path=="/oauth/jwks":
                out["jwks_count"]=len(body["keys"])
                out["jwks_sha256"]=hashlib.sha256(json.dumps(body,sort_keys=True,separators=(",",":")).encode()).hexdigest()
    except urllib.error.HTTPError as e: out={"status":e.code}
    except (urllib.error.URLError,TimeoutError,ConnectionError): out={"status":None}
    print(json.dumps(out))
'''

SETUP_EMPTY = r'''
import json,os,sys
if os.listdir("/fixture"): raise RuntimeError("nonempty_fixture_refused")
os.chown("/fixture",10001,10001);os.chmod("/fixture",0o700)
s=os.stat("/fixture");print(json.dumps({"uid":s.st_uid,"gid":s.st_gid,"mode":s.st_mode&0o777}))
'''

FILE_MODES = 'test "$(stat -c "%u:%g:%a" /config/riauth.toml)" = 10001:10001:600 && test "$(stat -c "%u:%g:%a" /config/database.key)" = 10001:10001:600'
STORE_HASH = "sha256sum /config/riauth.toml /data/riauth.redb"
REMOVE_FIXTURE = 'rm -f /config/riauth.toml /config/database.key /config/database.key.held; test -z "$(ls -A /config)"; chmod 0777 /config'


class Deployment:
    def __init__(self, runner, root, token, evidence):
        self.runner, self.root, self.token, self.evidence = runner, root, token, evidence
        self.project = "riauth-o07-" + token
        self.config = root / "config"
        self.config.mkdir(mode=0o700)
        self.docker = ["docker"]
        self.compose = None
        self.volume = None
        self.network = None
        self.model = None
        self.helper_number = 0
        self.resources_possible = False

    def passed(self, name, **observed):
        self.evidence["checks"].append({"id": name, "result": "passed", "observed": observed})

    def engine(self):
        # Pin the user's current local endpoint; never silently switch context.
        context = self.runner.success(["docker", "context", "show"]).decode().strip()
        require(re.fullmatch(r"[A-Za-z0-9_.-]{1,128}", context), "unsupported_context")
        endpoint = self.runner.success(["docker", "context", "inspect", context, "--format",
                                        '{{.Endpoints.docker.Host}}']).decode().strip()
        require(endpoint.startswith("unix://") and os.path.isabs(endpoint[7:]), "nonlocal_engine_refused")
        require(not os.environ.get("DOCKER_HOST") or os.environ["DOCKER_HOST"] == endpoint,
                "endpoint_override_refused")
        require(not os.environ.get("DOCKER_TLS_VERIFY"), "tls_endpoint_override_refused")
        self.docker = ["docker", "--host", endpoint]
        for key in ("DOCKER_CONTEXT", "DOCKER_HOST", "DOCKER_TLS_VERIFY", "DOCKER_CERT_PATH"):
            self.runner.environment.pop(key, None)
        version = decode(self.runner.success(self.docker + ["version", "--format",
                             '{"os":{{json .Server.Os}},"arch":{{json .Server.Arch}},"version":{{json .Server.Version}}}']))
        require(version["os"] == "linux" and version["arch"] == "arm64", "unsupported_engine_platform")
        self.evidence["engine"] = {"os": "linux", "arch": "arm64", "version": version["version"],
                                   "context": context, "host_visibility_test_required": True}
        for image, expected_user in ((IMAGE, "10001:10001"), (PROBE_IMAGE, None)):
            fields = ('{"id":{{json .Id}},"os":{{json .Os}},"arch":{{json .Architecture}},'
                      '"user":{{json (index .Config "User")}},'
                      '"volumes":{{json (index .Config "Volumes")}},'
                      '"edition":{{if index .Config "Labels"}}{{json (index .Config.Labels "org.riauth.edition")}}{{else}}null{{end}},'
                      '"revision":{{if index .Config "Labels"}}{{json (index .Config.Labels "org.opencontainers.image.revision")}}{{else}}null{{end}}}')
            item = decode(self.runner.success(self.docker + ["image", "inspect", "--format", fields, image]))
            require(item["id"] == image and item["os"] == "linux" and item["arch"] == "arm64",
                    "installed_image_mismatch")
            if expected_user:
                require(item["user"] == expected_user and item["edition"] == "essentials"
                        and item["revision"] == IMAGE_REVISION
                        and item["volumes"] == {"/data": {}}, "installed_provenance_mismatch")
            else:
                require(not item["volumes"], "helper_anonymous_volume_refused")
        self.passed("installed_images_match_local_manifest")

    def helper(self, code, data=None, *, empty_mount=False):
        self.helper_number += 1
        name = self.project + "-helper-" + str(self.helper_number)
        argv = self.docker + ["run", "--rm", "--pull", "never", "--name", name,
                              "--read-only", "--cap-drop", "ALL", "--security-opt", "no-new-privileges:true",
                              "--pids-limit", "32", "--memory", "64m", "--user", "10001:10001",
                              "--network", "host" if not empty_mount else "none", "--interactive"]
        for key, value in labels(self.token).items():
            argv += ["--label", key + "=" + value]
        if empty_mount:
            # The only helper mount is checked EMPTY, before any key/config exists.
            require(not list(self.config.iterdir()), "nonempty_helper_mount_refused")
            argv[argv.index("10001:10001")] = "0:0"
            argv += ["--cap-add", "CHOWN", "--cap-add", "FOWNER", "--cap-add", "DAC_OVERRIDE",
                     "--mount", f"type=bind,source={self.config},target=/fixture"]
        argv += ["--entrypoint", "python3", PROBE_IMAGE, "-c", code]
        self.resources_possible = True  # also covers a timed-out/uncertain create
        try:
            return decode(self.runner.success(argv, input_bytes=canonical(data or {}), timeout=8))
        except Refusal as error:
            if empty_mount and str(error) == "child_command_failed":
                raise Refusal("uid_mapping_unsupported") from None
            raise

    def cli(self, args, *, password=None, name="init", timeout=20):
        argv = self.compose + ["run", "--rm", "--no-deps", "--pull", "never", "--name",
                               self.project + "-" + name, "-T", "init", "--json", "--non-interactive"] + args
        return self.runner.call(argv, input_bytes=(password + "\n").encode() if password else b"", timeout=timeout)

    def tool(self, command, *, cleanup=False):
        # This is the declared application init service, not the Python helper.
        return self.runner.success(self.compose + ["run", "--rm", "--no-deps", "--pull", "never", "--name",
                                   self.project + "-tool", "-T", "--entrypoint", "/bin/sh", "init", "-ec", command],
                                   timeout=8, cleanup=cleanup)

    def inspect(self, kind, identifier, *, cleanup=False):
        fields = ('{"labels":{{json .Labels}}}' if kind in ("volume", "network") else
                  '{"labels":{{json .Config.Labels}},"user":{{json .Config.User}},'
                  '"read_only":{{json .HostConfig.ReadonlyRootfs}},"security":{{json .HostConfig.SecurityOpt}},'
                  '"network":{{json .HostConfig.NetworkMode}},"mounts":{{json .Mounts}},"tmpfs":{{json .HostConfig.Tmpfs}},'
                  '"state":{{json .State.Status}},'
                  '"health":{{if index .State "Health"}}{{json .State.Health.Status}}{{else}}null{{end}}}')
        return decode(self.runner.success(self.docker + [kind, "inspect", "--format", fields, identifier],
                                          timeout=4, cleanup=cleanup))

    def owns(self, item):
        return all(item.get("labels", {}).get(key) == value for key, value in labels(self.token).items())

    def container_ids(self, *, cleanup=False):
        argv = self.docker + ["ps", "--all", "--quiet"]
        for key, value in labels(self.token).items():
            argv += ["--filter", "label=" + key + "=" + value]
        ids = self.runner.success(argv, timeout=4, cleanup=cleanup).decode().split()
        require(len(ids) <= 8 and all(re.fullmatch(r"[0-9a-f]{12,64}", identifier) for identifier in ids),
                "owned_container_inventory_invalid")
        return ids

    def owned_resources(self, kind, *, cleanup=False):
        argv = self.docker + [kind, "ls", "--format", "{{.Name}}"]
        for key, value in labels(self.token).items():
            argv += ["--filter", "label=" + key + "=" + value]
        names = self.runner.success(argv, timeout=4, cleanup=cleanup).decode().split()
        require(len(names) <= 1 and all(re.fullmatch(r"[A-Za-z0-9_.-]{1,128}", name) for name in names),
                "owned_resource_inventory_invalid")
        return names

    def service_id(self):
        ids = self.runner.success(self.compose + ["ps", "--all", "--quiet", "riauth"]).decode().split()
        require(len(ids) == 1 and re.fullmatch(r"[0-9a-f]{12,64}", ids[0]), "own_service_missing")
        return ids[0]

    def service_ready(self):
        deadline = min(self.runner.end, time.monotonic() + 35)
        while time.monotonic() < deadline:
            item = self.inspect("container", self.service_id())
            require(self.owns(item), "service_ownership_mismatch")
            require(item["state"] == "running", "service_exited")
            if item["health"] == "healthy":
                return item
            time.sleep(0.2)
        raise Refusal("healthcheck_deadline")

    def probe(self, path):
        return self.helper(PROBE, {"kind": "http", "issuer": self.issuer, "path": path})

    def remote(self, command, *, password=None):
        output = self.runner.success(self.compose + ["exec", "-T", "riauth", "/usr/local/bin/riauth",
                                     "--json", "--non-interactive", "--server", self.issuer,
                                     "--session-file", "/tmp/o07-admin-session", "--request-timeout", "5"] + command,
                                     input_bytes=(password + "\n").encode() if password else b"", timeout=10)
        value = decode(output)
        require(value.get("ok") is True, "remote_cli_refused")
        return value["data"]

    def run(self, template):
        self.engine()
        setup = self.helper(SETUP_EMPTY, empty_mount=True)
        require(setup == {"uid": 10001, "gid": 10001, "mode": 0o700}, "uid_mapping_unsupported")
        port_info = self.helper(PROBE, {"kind": "port"})
        require(port_info["free_bytes"] >= 8 * 1024**3, "engine_disk_floor")
        port = port_info["port"]
        require(isinstance(port, int) and 1024 <= port <= 65535, "invalid_fixture_port")
        # Refuse a conflicting controller-host port without probing that service.
        try:
            with socket.socket() as listener:
                listener.bind(("127.0.0.1", port))
        except OSError:
            raise Refusal("controller_port_conflict") from None
        self.issuer = f"http://127.0.0.1:{port}"
        self.compose, self.model, proof = render_proof(self.runner, self.root, self.project,
                                                        self.config, self.issuer, self.token, template)
        self.compose = self.docker + self.compose[1:]
        self.evidence["render"] = proof
        volume = self.model["volumes"]["riauth-data"]["name"]
        existing = self.runner.success(self.docker + ["volume", "ls", "--quiet", "--filter",
                                      "name=^" + re.escape(volume) + "$"])
        require(not existing.strip(), "existing_volume_refused")
        network = self.model["networks"]["default"]["name"]
        existing = self.runner.success(self.docker + ["network", "ls", "--quiet", "--filter",
                                      "name=^" + re.escape(network) + "$"])
        require(not existing.strip(), "existing_network_refused")
        self.volume = volume
        self.network = network
        password = secrets.token_urlsafe(32)
        for args, stdin in ((["keygen", "--out", "/config/database.key"], None),
                            (["init", "--issuer", self.issuer, "--listen", f"127.0.0.1:{port}",
                              "--data-dir", "/data", "--database-key-file", "/config/database.key",
                              "--password-stdin"], password)):
            status, output = self.cli(args, password=stdin, timeout=25)
            require(status == 0 and decode(output).get("ok") is True, "fixture_init_failed")
        require(self.owns(self.inspect("volume", self.volume)), "volume_ownership_mismatch")
        try:
            self.tool(FILE_MODES)
        except Refusal as error:
            if str(error) == "child_command_failed":
                raise Refusal("uid_mapping_unsupported") from None
            raise
        self.passed("uid_mapping_private_init_one_volume")
        self.runner.success(self.compose + ["up", "--detach", "--no-build", "--pull", "never", "riauth"])
        state = self.service_ready()
        require(state["user"] == "10001:10001" and state["read_only"] is True
                and "no-new-privileges:true" in state["security"] and state["network"] == "host",
                "live_container_safety_mismatch")
        mounts = {mount["Destination"]: mount for mount in state["mounts"]}
        tmpfs_options = set((state.get("tmpfs") or {}).get("/tmp", "").split(","))
        require(set(mounts) in ({"/config", "/data"}, {"/config", "/data", "/tmp"})
                and mounts["/config"]["RW"] is False
                and mounts["/data"]["Name"] == self.volume
                and {"rw", "nosuid", "nodev"} <= tmpfs_options
                and bool({"size=64m", "size=67108864"} & tmpfs_options), "live_mount_mismatch")
        require(len(self.container_ids()) == 1, "tools_profile_started_unexpectedly")
        for path in ("/livez", "/readyz", "/healthz", "/.well-known/openid-configuration"):
            result = self.probe(path)
            require(result["status"] == 200, "fixture_http_not_ready")
            if path.endswith("openid-configuration"):
                require(result["issuer_matches"], "fixture_discovery_mismatch")
        jwks = self.probe("/oauth/jwks")
        require(jwks["status"] == 200 and jwks["jwks_count"] > 0, "fixture_jwks_missing")
        # Engine-loopback probes alone cannot prove Desktop host visibility.
        host = decode(self.runner.success([sys.executable, "-c", PROBE], timeout=5,
                      input_bytes=canonical({"kind": "http", "issuer": self.issuer, "path": "/oauth/jwks"})))
        require(host["status"] == 200 and host.get("jwks_sha256") == jwks["jwks_sha256"],
                "host_network_visibility_unsupported")
        self.passed("host_network_visibility_and_live_probes", public_tls_verified=False)
        self.remote(["login", "admin", "--password-stdin"], password=password)
        doctor = self.remote(["doctor"])
        require(doctor["encrypted_at_rest"] is True and doctor["storage"] == "redb"
                and doctor["healthy"] is True, "fixture_doctor_mismatch")
        self.passed("administrator_login_encrypted_store")
        status, output = self.cli(["serve"], name="second-owner", timeout=8)
        denial = decode(output)
        require(status == 5 and denial.get("ok") is False
                and denial.get("error", {}).get("code") == "storage_owned", "second_owner_not_refused")
        self.passed("second_redb_owner_refused", error_code="storage_owned", exit_code=5)
        self.runner.success(self.compose + ["stop", "--timeout", "5", "riauth"], timeout=8)
        require(self.probe("/readyz")["status"] is None, "stopped_fixture_still_serves")
        before = self.tool(STORE_HASH)
        self.tool("mv /config/database.key /config/database.key.held")
        status, _ = self.cli(["serve"], name="missing-key", timeout=8)
        require(status != 0 and self.probe("/readyz")["status"] is None, "missing_key_not_refused")
        require(before == self.tool(STORE_HASH), "missing_key_changed_store_or_config")
        self.tool("mv /config/database.key.held /config/database.key")
        self.passed("outage_missing_key_refusal_preserves_store")
        self.runner.success(self.compose + ["up", "--detach", "--no-build", "--pull", "never", "riauth"])
        self.service_ready()
        self.remote(["login", "admin", "--password-stdin"], password=password)
        require(self.probe("/readyz")["status"] == 200, "restarted_fixture_not_ready")
        self.passed("same_volume_read_only_restart_login")

    def cleanup(self):
        # Caller reserves the last 20 seconds of the whole 180s deadline.
        failures = []
        for identifier in self.container_ids(cleanup=True) if self.resources_possible else []:
            try:
                item = self.inspect("container", identifier, cleanup=True)
                require(self.owns(item), "cleanup_ownership_mismatch")
                self.runner.success(self.docker + ["rm", "--force", identifier], timeout=3, cleanup=True)
            except Refusal:
                failures.append("container_cleanup_failed")
        volumes = self.owned_resources("volume", cleanup=True) if self.resources_possible else []
        if volumes:
            try:
                require(volumes == [self.volume] and self.compose is not None, "cleanup_volume_not_created_here")
                require(self.owns(self.inspect("volume", self.volume, cleanup=True)), "cleanup_volume_not_owned")
                self.tool(REMOVE_FIXTURE, cleanup=True)
                # Recheck after the tool, never remove a volume now in use.
                require(self.owns(self.inspect("volume", self.volume, cleanup=True)), "cleanup_volume_not_owned")
                self.runner.success(self.docker + ["volume", "rm", self.volume], timeout=3, cleanup=True)
            except Refusal:
                failures.append("volume_or_private_file_cleanup_failed")
        # Catch a tool whose CLI timed out but left a remote container behind.
        remaining = self.container_ids(cleanup=True) if self.resources_possible else []
        if remaining:
            failures.append("owned_containers_remaining")
        networks = self.owned_resources("network", cleanup=True) if self.resources_possible else []
        if networks:
            try:
                require(networks == [self.network] and self.owns(self.inspect("network", self.network, cleanup=True)),
                        "cleanup_network_not_created_here")
                self.runner.success(self.docker + ["network", "rm", self.network], timeout=3, cleanup=True)
            except Refusal:
                failures.append("network_cleanup_failed")
        try:
            self.config.rmdir()  # works for an empty UID10001 directory via its owning parent
        except OSError:
            failures.append("private_fixture_remaining")
        return {"ok": not failures, "failures": failures}


def target_directory():
    target = ROOT / "target"
    require(not target.is_symlink(), "symlink_target_refused")
    target.mkdir(exist_ok=True)
    require(target.is_dir() and target.resolve() == target, "private_target_invalid")
    free = shutil.disk_usage(target).free
    require(free >= 8 * 1024**3, "disk_floor")
    return target


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--template-revision", required=True)
    parser.add_argument("--image", required=True)
    parser.add_argument("--image-source-revision", required=True)
    parser.add_argument("--edition", required=True, choices=["essentials"])
    parser.add_argument("--artifact-kind", required=True, choices=["local-source"])
    parser.add_argument("--probe-image", required=True)
    parser.add_argument("--timeout-seconds", type=int, default=180, choices=[180])
    parser.add_argument("--evidence", type=Path)
    parser.add_argument("--check-source", action="store_true",
                        help="Git + Compose render only; no engine/container/service calls")
    args = parser.parse_args()
    require((args.template_revision, args.image, args.image_source_revision, args.probe_image)
            == (TEMPLATE_REVISION, IMAGE, IMAGE_REVISION, PROBE_IMAGE), "unreserved_source_or_image")
    require(args.check_source or args.evidence is not None, "new_evidence_path_required")
    require(not args.check_source or args.evidence is None, "static_mode_has_no_runtime_evidence")
    target = target_directory()
    evidence_path = None
    if args.evidence is not None:
        evidence_path = args.evidence.absolute()
        require(evidence_path.parent == target and not evidence_path.exists()
                and not evidence_path.is_symlink(), "evidence_path_not_new_private_target")
    token = secrets.token_hex(12)
    root = Path(tempfile.mkdtemp(prefix="o07-small-", dir=target))
    os.chmod(root, 0o700)
    environment = {key: os.environ[key] for key in (
        "PATH", "HOME", "LANG", "DOCKER_CONFIG", "DOCKER_HOST", "DOCKER_CONTEXT",
        "DOCKER_TLS_VERIFY", "DOCKER_CERT_PATH",
    ) if key in os.environ}
    environment["RIAUTH_IMAGE"] = IMAGE
    runner = Runner(args.timeout_seconds, environment)
    try:
        template, source = source_proof(runner)
    except Exception:
        shutil.rmtree(root)
        raise
    if args.check_source:
        try:
            runner.environment["DOCKER_HOST"] = "unix://" + str(root / "render-engine-disabled.sock")
            for key in ("DOCKER_CONTEXT", "DOCKER_TLS_VERIFY", "DOCKER_CERT_PATH"):
                runner.environment.pop(key, None)
            _, _, proof = render_proof(runner, root, "riauth-o07-" + token, root / "config",
                                      "http://127.0.0.1:49100", token, template)
            print(json.dumps({"mode": "source-only", "source": source, "render": proof,
                              "engine_endpoint_disabled": True, "engine_contacted": False,
                              "product_runtime_executed": False}, sort_keys=True))
            return 0
        finally:
            shutil.rmtree(root)
    signal.signal(signal.SIGINT, interrupted)
    signal.signal(signal.SIGTERM, interrupted)
    # SIGHUP is untouched: inherited nohup ignore remains intact. Cleanup is
    # best effort; no atomic signal/publication or forced-termination promise.
    evidence = {"schema": "riauth.deployment-small-local/v1", "project": PROJECT, "task": TASK,
                "source": source, "checks": [], "result": "failed", "scope": "local-template-mechanics",
                "public_tls_verified": False, "distributed_verified": False,
                "ha_verified": False, "whole_o07_complete": False}
    deployment = Deployment(runner, root, token, evidence)
    overall_end = runner.end
    runner.end -= 20
    try:
        deployment.run(template)
        evidence["result"] = "passed"
    except Refusal as error:
        evidence["failure"] = str(error)
    except Exception:
        evidence["failure"] = "unexpected_failure"  # never stringify secret-bearing exceptions
    finally:
        runner.end = overall_end
        try:
            evidence["cleanup"] = deployment.cleanup()
        except Exception:
            evidence["cleanup"] = {"ok": False, "failures": ["cleanup_incomplete"]}
        if not evidence["cleanup"]["ok"]:
            evidence["result"] = "failed"
        if INTERRUPTED or time.monotonic() >= overall_end:
            evidence["result"] = "failed"
            evidence["failure"] = "cancelled" if INTERRUPTED else "deadline"
        if not (root / "config").exists():
            shutil.rmtree(root)
    private_file(evidence_path, json.dumps(evidence, sort_keys=True, indent=2).encode() + b"\n")
    print(json.dumps({"result": evidence["result"], "checks": len(evidence["checks"]),
                      "cleanup_ok": evidence["cleanup"]["ok"], "evidence_written": True}))
    return 0 if evidence["result"] == "passed" else 1


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Refusal as failure:
        print(json.dumps({"result": "refused", "failure": str(failure)}))
        sys.exit(2)
    except Exception:
        print(json.dumps({"result": "refused", "failure": "unexpected_failure"}))
        sys.exit(2)
