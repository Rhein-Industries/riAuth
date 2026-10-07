#!/usr/bin/env python3
"""Validate one native publication image, smoke its persisted state, then save it.

Only the publication workflow invokes this script. Product output, credentials
and HTTP bodies stay in the private fixture; artifacts contain fixed outcomes,
image/source identities and the tested image archive, never runtime volumes.
"""

import argparse
import hashlib
import json
import os
import pathlib
import platform
import re
import shutil
import signal
import subprocess
import tempfile
import time
import urllib.error
import urllib.request
import uuid


GIB = 1024**3
OWNER = "org.riauth.publish-fixture"
SOURCE_URL = "https://github.com/Rhein-Industries/riAuth"
VERSION = "0.1.3"
ARCHES = {"amd64": ("x86_64", "X64"), "arm64": ("aarch64", "ARM64")}


class Refusal(Exception):
    pass


def require(condition, code):
    if not condition:
        raise Refusal(code)


def digest(path):
    value = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            value.update(block)
    return value.hexdigest()


class Fixture:
    def __init__(self, args, private):
        self.args, self.private = args, private
        self.owner = uuid.uuid4().hex
        self.prefix = "riauth-publish-" + self.owner
        self.containers, self.volumes = {}, {}
        self.deadline = time.monotonic() + 300
        self.receipt = {"schema": "riauth.container-publication/v1", "ok": False,
                        "edition": args.edition, "architecture": args.architecture,
                        "version": VERSION, "source_sha": args.source_sha,
                        "workflow_sha": args.workflow_sha, "checked_ci_run_id": args.ci_run,
                        "checks": [], "failure": None, "cleanup": [],
                        "unexpected_exception": None, "helper_failure_line": None}

    def capacity(self):
        require(min(shutil.disk_usage(p).free for p in
                    (self.private, "/var/lib/docker")) >= 10 * GIB, "disk_stop_10GiB")

    def command(self, args, *, input_data=None, timeout=30, cleanup=False, cap=1024**2, watch=None):
        if not cleanup:
            self.capacity()
            require(time.monotonic() < self.deadline, "fixture_deadline")
            timeout = min(timeout, self.deadline - time.monotonic())
        # No command/output printing: even successful initialization is private.
        with tempfile.TemporaryFile(dir=self.private) as out, tempfile.TemporaryFile(dir=self.private) as err:
            process = subprocess.Popen(args, stdin=subprocess.PIPE if input_data is not None else subprocess.DEVNULL,
                                       stdout=out, stderr=err, start_new_session=True)
            try:
                if input_data is not None:
                    require(len(input_data) <= 65536, "stdin_cap")
                    process.stdin.write(input_data)
                    process.stdin.close()
                end = time.monotonic() + timeout
                while process.poll() is None:
                    require(time.monotonic() < end, "command_deadline")
                    require(os.fstat(out.fileno()).st_size <= cap and
                            os.fstat(err.fileno()).st_size <= cap, "command_output_cap")
                    if not cleanup:
                        self.capacity()
                    if watch is not None and watch.exists():
                        require(watch.stat().st_size <= 2 * GIB, "archive_cap")
                    time.sleep(0.1)
                code = process.wait()
                require(time.monotonic() < end, "terminal_command_deadline")
                require(os.fstat(out.fileno()).st_size <= cap and
                        os.fstat(err.fileno()).st_size <= cap, "command_output_cap")
                out.seek(0)
                return code, out.read(cap + 1)
            finally:
                if process.poll() is None:
                    os.killpg(process.pid, signal.SIGTERM)
                    try:
                        process.wait(timeout=3)
                    except subprocess.TimeoutExpired:
                        os.killpg(process.pid, signal.SIGKILL)
                        process.wait(timeout=5)

    def docker(self, *args, **kwargs):
        return self.command(["docker", *map(str, args)], **kwargs)

    def inspect(self, kind, name, cleanup=False):
        code, data = self.docker(kind, "inspect", name, cleanup=cleanup)
        require(code == 0, "inspect_failed")
        values = json.loads(data)
        require(isinstance(values, list) and len(values) == 1, "inspect_shape")
        return values[0]

    def absent(self, kind, name, cleanup=False):
        # Listing is a positive absence observation; arbitrary inspect errors
        # must never be mistaken for permission to create/delete a resource.
        code, data = self.docker(kind, "ls", "--format", "{{.Name}}" if kind == "volume" else "{{.Names}}",
                                 *([] if kind == "volume" else ["--all"]), cleanup=cleanup)
        require(code == 0 and name not in data.decode().splitlines(), "resource_not_absent")

    def volume(self, suffix):
        name = self.prefix + "-" + suffix
        self.absent("volume", name)
        self.volumes[name] = None  # Retain ambiguous creation for fail-closed cleanup.
        code, _ = self.docker("volume", "create", "--label", OWNER + "=" + self.owner, name)
        require(code == 0, "volume_create_failed")
        item = self.inspect("volume", name)
        require(item["Name"] == name and item.get("Labels", {}).get(OWNER) == self.owner,
                "volume_owner")
        self.volumes[name] = item["CreatedAt"]
        return name

    def container(self, argv, mounts, *, port=False, stdin=False):
        name = self.prefix + "-c" + str(len(self.containers)) + "-" + uuid.uuid4().hex[:8]
        self.absent("container", name)
        self.containers[name] = None
        command = ["container", "create", "--name", name, "--label", OWNER + "=" + self.owner,
                   "--user", "10001:10001", "--read-only", "--cap-drop", "ALL",
                   "--security-opt", "no-new-privileges:true", "--pids-limit", "128",
                   "--memory", "512m", "--cpus", "1", "--network", "bridge" if port else "none",
                   "--log-driver", "none", "--tmpfs", "/tmp:rw,nosuid,nodev,size=16m"]
        if port:
            command += ["--publish", "127.0.0.1::9000"]
        if stdin:
            command += ["--interactive"]
        for mount in mounts:
            command += ["--mount", mount]
        if not any("dst=/data" in mount for mount in mounts):
            command += ["--tmpfs", "/data:rw,nosuid,nodev,size=16m"]
        command += ["--entrypoint", argv[0], self.image, *argv[1:]]
        code, _ = self.docker(*command)
        require(code == 0, "container_create_failed")
        item = self.inspect("container", name)
        require(item["Name"] == "/" + name and item["Image"] == self.image and
                item["Config"].get("Labels", {}).get(OWNER) == self.owner, "container_owner")
        self.containers[name] = item["Id"]
        return name

    def owned_container(self, name, cleanup=False):
        item = self.inspect("container", name, cleanup)
        require(item["Name"] == "/" + name and item["Config"].get("Labels", {}).get(OWNER) == self.owner
                and self.containers[name] in (None, item["Id"]), "container_owner_changed")
        return item

    def remove(self, name, cleanup=False):
        item = self.owned_container(name, cleanup)
        if item["State"]["Running"]:
            code, _ = self.docker("container", "stop", "--time", "5", item["Id"], cleanup=cleanup, timeout=20)
            require(code == 0, "owned_stop_failed")
        code, _ = self.docker("container", "rm", item["Id"], cleanup=cleanup)
        require(code == 0, "owned_remove_failed")
        self.absent("container", name, cleanup)
        del self.containers[name]

    def tool(self, argv, mounts=(), input_data=None, timeout=30):
        name = self.container(argv, mounts, stdin=input_data is not None)
        failed = False
        try:
            code, data = self.docker("container", "start", "--attach", "--interactive", name,
                                     input_data=input_data, timeout=timeout)
            item = self.owned_container(name)
            require(not item["State"]["Running"] and code == item["State"]["ExitCode"] == 0,
                    "image_tool_failed")
            return data
        except BaseException:
            failed = True
            raise
        finally:
            try:
                self.remove(name, cleanup=True)
            except BaseException:
                if not failed:
                    raise
                self.receipt["cleanup"].append("tool_cleanup_failed")

    def http(self, port, path, *, body=None, token=None):
        headers = {"Accept": "application/json"}
        if body is not None:
            headers["Content-Type"] = "application/json"
        if token is not None:
            headers["Authorization"] = "Bearer " + token
        request = urllib.request.Request("http://127.0.0.1:" + str(port) + path,
                                         data=None if body is None else json.dumps(body).encode(), headers=headers)
        # Avoid host proxies and redirects, so fixture credentials stay local.
        class NoRedirect(urllib.request.HTTPRedirectHandler):
            def redirect_request(self, req, fp, code, msg, headers, newurl):
                return None
        opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), NoRedirect())
        with opener.open(request, timeout=min(3, max(0.1, self.deadline - time.monotonic()))) as reply:
            data = reply.read(1024**2 + 1)
            require(len(data) <= 1024**2 and reply.status == 200, "http_response")
            return data, reply.headers

    def start(self, mounts):
        name = self.container(["/usr/local/bin/riauth", "--config", "/config/riauth.toml", "serve"], mounts, port=True)
        code, _ = self.docker("container", "start", name)
        require(code == 0, "server_start")
        item = self.owned_container(name)
        require(item["State"]["Running"], "server_exited")
        require(item["Config"]["User"] == "10001:10001" and item["HostConfig"]["ReadonlyRootfs"]
                and not item["HostConfig"]["Privileged"] and "ALL" in item["HostConfig"]["CapDrop"]
                and "no-new-privileges:true" in item["HostConfig"]["SecurityOpt"], "server_controls")
        observed = {m["Destination"]: m["RW"] for m in item["Mounts"]}
        require(observed["/config"] is False and observed["/theme"] is False and observed["/data"] is True,
                "mount_controls")
        ports = item["NetworkSettings"]["Ports"]["9000/tcp"]
        require(len(ports) == 1 and ports[0]["HostIp"] == "127.0.0.1", "loopback_only")
        port = int(ports[0]["HostPort"])
        end = min(self.deadline, time.monotonic() + 30)
        while time.monotonic() < end:
            require(self.owned_container(name)["State"]["Running"], "server_exited")
            try:
                data, _ = self.http(port, "/readyz")
                ready = json.loads(data)
                require(ready["status"] == "ok" and ready["version"] == VERSION and
                        ready["issuer"] == "https://container-smoke.invalid", "readiness_identity")
                return name, port
            except (urllib.error.URLError, TimeoutError, ConnectionError):
                time.sleep(0.2)
        raise Refusal("readiness_deadline")

    def run(self):
        a = self.args
        require((platform.system(), platform.machine(), os.environ.get("RUNNER_ARCH"),
                 os.environ.get("RUNNER_ENVIRONMENT")) ==
                ("Linux", *ARCHES[a.architecture], "github-hosted"), "native_host")
        code, data = self.command(["git", "-C", str(a.product), "rev-parse", "HEAD"])
        require(code == 0 and data.decode().strip() == a.source_sha, "source_checkout")
        item = self.inspect("image", a.image)
        labels = item["Config"].get("Labels") or {}
        require(item["Os"] == "linux" and item["Architecture"] == a.architecture and
                0 < item["Size"] <= GIB and
                item["Config"]["User"] == "10001:10001" and
                labels.get("org.riauth.edition") == a.edition and
                labels.get("org.opencontainers.image.revision") == a.source_sha and
                labels.get("org.opencontainers.image.source") == SOURCE_URL and
                labels.get("org.opencontainers.image.version") == VERSION, "image_identity")
        self.image = item["Id"]
        require(re.fullmatch(r"sha256:[0-9a-f]{64}", self.image), "image_id")
        caps = json.loads(self.tool(["/usr/local/bin/riauth", "--json", "capabilities"]))
        require(caps["ok"] is True and caps["data"]["edition"] == a.edition and
                caps["data"]["version"] == VERSION and
                caps["data"]["target"] == {"os": "linux", "arch": ARCHES[a.architecture][0]},
                "native_binary_identity")
        require(self.tool(["/usr/local/bin/riauth", "--version"]).strip() == b"riauth 0.1.3", "binary_version")
        licenses = {}
        for name in ("LICENSE", "THIRD_PARTY_NOTICES.md"):
            data = self.tool(["/bin/cat", "/usr/share/doc/riauth/" + name])
            require(data == (a.product / name).read_bytes(), "license_source_equality")
            licenses[name] = hashlib.sha256(data).hexdigest()
        self.tool(["/usr/bin/test", "-r", "/usr/share/doc/libssl3t64/copyright"])
        self.receipt["checks"] += ["native_host_image_binary", "licenses"]
        config, state = self.volume("config"), self.volume("data")
        for volume in (config, state):
            output = self.tool(["/usr/bin/stat", "-c", "%u:%g:%a", "/data"],
                               ["type=volume,src=" + volume + ",dst=/data"])
            require(output.strip() == b"10001:10001:700", "volume_uid_mode")
        rw = ["type=volume,src=" + config + ",dst=/config", "type=volume,src=" + state + ",dst=/data"]
        self.tool(["/usr/local/bin/riauth", "keygen", "--out", "/config/database.key"], rw)
        password = "Disposable-" + uuid.uuid4().hex
        initialized = json.loads(self.tool(["/usr/local/bin/riauth", "--config", "/config/riauth.toml",
            "--json", "--non-interactive", "init", "--issuer", "https://container-smoke.invalid",
            "--listen", "0.0.0.0:9000", "--data-dir", "/data", "--database-key-file",
            "/config/database.key", "--password-stdin"], rw, (password + "\n").encode(), timeout=90))
        require(initialized["ok"] is True and initialized["data"]["initialized"] is True, "encrypted_init")
        self.tool(["/bin/sh", "-c", "cat >> /config/riauth.toml"], rw,
                  b'\n[frontend]\ntheme_dir = "/theme"\n')
        theme = self.private / "theme"
        (theme / "assets").mkdir(parents=True, mode=0o755)
        theme.chmod(0o755)
        (theme / "assets").chmod(0o755)
        css = b"/* publication readonly theme */\n:root { --publication-smoke: 1; }\n"
        (theme / "assets" / "app.css").write_bytes(css)
        (theme / "assets" / "app.css").chmod(0o644)
        mounts = [rw[0] + ",readonly", rw[1], "type=bind,src=" + str(theme) + ",dst=/theme,readonly"]
        server, port = self.start(mounts)
        served, headers = self.http(port, "/portal/assets/app.css")
        require(served == css and headers.get("X-Content-Type-Options") == "nosniff" and
                headers.get("Cache-Control") == "no-store", "theme_response")
        login = json.loads(self.http(port, "/api/login", body={"username": "admin", "password": password})[0])
        token = login["session_token"]
        require(isinstance(token, str) and token, "login_session")
        me = self.http(port, "/api/me", token=token)[0]
        jwks = self.http(port, "/oauth/jwks")[0]
        require(json.loads(jwks)["keys"], "signing_keys")
        self.remove(server)  # Actual stop/close, then a fresh container on the same volumes.
        server, port = self.start(mounts)
        require(self.http(port, "/api/me", token=token)[0] == me and
                self.http(port, "/oauth/jwks")[0] == jwks and
                self.http(port, "/portal/assets/app.css")[0] == css, "persistent_restart")
        self.receipt["checks"] += ["encrypted_initialization", "readyz", "readonly_theme", "persistent_restart"]
        self.remove(server)
        archive = a.output / (a.edition + "-" + a.architecture + ".docker.tar")
        require(not archive.exists(), "archive_exists")
        code, _ = self.docker("image", "save", "--output", archive, self.image, timeout=90, watch=archive)
        require(code == 0 and archive.is_file() and 0 < archive.stat().st_size <= 2 * GIB, "archive_cap")
        archive.chmod(0o600)
        self.receipt.update(image_id=self.image, licenses=licenses,
                            archive={"name": archive.name, "bytes": archive.stat().st_size, "sha256": digest(archive)})
        require(time.monotonic() < self.deadline, "terminal_fixture_deadline")
        self.capacity()

    def cleanup(self):
        for name in list(self.containers):
            try:
                self.remove(name, cleanup=True)
            except BaseException:
                self.receipt["cleanup"].append("container_cleanup_failed")
        for name, created in list(self.volumes.items()):
            try:
                item = self.inspect("volume", name, cleanup=True)
                require(item["Name"] == name and item.get("Labels", {}).get(OWNER) == self.owner
                        and created in (None, item["CreatedAt"]), "volume_owner_changed")
                code, _ = self.docker("volume", "rm", name, cleanup=True)
                require(code == 0, "volume_remove_failed")
                self.absent("volume", name, cleanup=True)
                del self.volumes[name]
            except BaseException:
                self.receipt["cleanup"].append("volume_cleanup_failed")
        self.receipt["remaining_owned_containers"] = len(self.containers)
        self.receipt["remaining_owned_volumes"] = len(self.volumes)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--product", type=pathlib.Path, required=True)
    parser.add_argument("--output", type=pathlib.Path, required=True)
    parser.add_argument("--image", required=True)
    parser.add_argument("--edition", choices=("essentials", "platform"), required=True)
    parser.add_argument("--architecture", choices=tuple(ARCHES), required=True)
    parser.add_argument("--source-sha", required=True)
    parser.add_argument("--workflow-sha", required=True)
    parser.add_argument("--ci-run", required=True)
    args = parser.parse_args()
    require(all(re.fullmatch(r"[0-9a-f]{40}", s) for s in (args.source_sha, args.workflow_sha)), "source_sha")
    require(re.fullmatch(r"[1-9][0-9]{0,19}", args.ci_run), "ci_run_id")
    args.product, args.output = args.product.resolve(), args.output.resolve(strict=True)
    signal.signal(signal.SIGCHLD, signal.SIG_DFL)
    require(signal.getsignal(signal.SIGCHLD) == signal.SIG_DFL, "waitable_child_required")
    def interrupted(signum, frame):
        raise Refusal("fixture_interrupted")
    signal.signal(signal.SIGTERM, interrupted)
    os.umask(0o077)
    with tempfile.TemporaryDirectory(prefix="runtime-", dir=args.output.parent) as temporary:
        fixture = Fixture(args, pathlib.Path(temporary))
        try:
            fixture.run()
        except Refusal as error:
            fixture.receipt["failure"] = error.args[0]
        except BaseException as error:
            fixture.receipt["failure"] = "unexpected_smoke_failure"
            classes = {KeyError: "key_error", TypeError: "type_error", ValueError: "value_error",
                       json.JSONDecodeError: "json_decode_error", BrokenPipeError: "broken_pipe",
                       PermissionError: "permission_error", FileNotFoundError: "file_not_found",
                       FileExistsError: "file_exists", OSError: "os_error", TimeoutError: "timeout"}
            fixture.receipt["unexpected_exception"] = classes.get(type(error), "unclassified")
            trace = error.__traceback__
            for _ in range(64):
                if trace is None:
                    break
                if trace.tb_frame.f_code.co_filename == __file__ and 1 <= trace.tb_lineno <= 10000:
                    fixture.receipt["helper_failure_line"] = trace.tb_lineno
                trace = trace.tb_next
        finally:
            fixture.cleanup()
        fixture.receipt["ok"] = fixture.receipt["failure"] is None and not fixture.receipt["cleanup"]
        receipt = args.output / (args.edition + "-" + args.architecture + ".json")
        with receipt.open("x") as stream:
            json.dump(fixture.receipt, stream, sort_keys=True, indent=2)
            stream.write("\n")
            stream.flush()
            os.fsync(stream.fileno())
        print("container smoke passed" if fixture.receipt["ok"] else "container smoke refused; see fixed receipt")
        return 0 if fixture.receipt["ok"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
