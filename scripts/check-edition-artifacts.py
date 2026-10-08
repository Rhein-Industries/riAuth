#!/usr/bin/env python3
"""Focused smoke checks against the exact release archives and loaded images."""

import argparse
import json
import pathlib
import socket
import subprocess
import tarfile
import tempfile
import time
import urllib.error
import urllib.request


PASSWORD = "a05-artifact-smoke-password"
PLATFORM_ROUTES = (
    ("/scim/v2/ServiceProviderConfig", 200),
    ("/.well-known/ssf-configuration", 200),
    ("/events", 200),
    ("/api/admin/access/requests", 403),
    ("/api/admin/access/grants", 403),
)
SHARED_ROUTES = (
    ("/portal/assets/admin.js", 200),
    ("/api/admin/clients/smoke/diagnostics", 403),
    ("/api/admin/client-checks", 405),
    ("/api/portal/password", 405),
    ("/api/portal/account/reset", 405),
)


def run(*args, input=None, timeout=30):
    result = subprocess.run(args, input=input, text=True, capture_output=True, timeout=timeout)
    if result.returncode:
        raise RuntimeError(f"{args[0]} failed ({result.returncode}): {result.stderr.strip()}")
    return result.stdout


def port():
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        return listener.getsockname()[1]


def request(base, path, body=None, token=None, extra_headers=None):
    headers = {"Content-Type": "application/json"} if body is not None else {}
    if token:
        headers["Authorization"] = f"Bearer {token}"
    if extra_headers:
        headers.update(extra_headers)
    payload = json.dumps(body).encode() if body is not None else None
    call = urllib.request.Request(base + path, data=payload, headers=headers)
    try:
        with urllib.request.urlopen(call, timeout=5) as response:
            return response.status, response.read()
    except urllib.error.HTTPError as error:
        return error.code, error.read()


def wait_ready(base, alive):
    deadline = time.monotonic() + 20
    while time.monotonic() < deadline:
        if not alive():
            raise RuntimeError("server exited before readiness")
        try:
            if request(base, "/readyz")[0] == 200:
                return
        except (OSError, TimeoutError):
            pass
        time.sleep(0.1)
    raise RuntimeError("server did not become ready")


def capabilities(binary, edition):
    envelope = json.loads(run(str(binary), "--json", "capabilities"))
    data = envelope["data"]
    assert envelope["ok"] and data["edition"] == edition
    assert data["build_features"] == (["essentials"] if edition == "essentials" else ["essentials", "platform"])
    return data


def check_routes(base, edition):
    for route, expected in SHARED_ROUTES:
        actual, _ = request(base, route)
        assert actual == expected, f"{edition} {route}: expected {expected}, got {actual}"
    for route, platform_status in PLATFORM_ROUTES:
        expected = 404 if edition == "essentials" else platform_status
        actual, _ = request(base, route)
        assert actual == expected, f"{edition} {route}: expected {expected}, got {actual}"


def token(base):
    status, body = request(base, "/api/login", {"username": "admin", "password": PASSWORD})
    assert status == 200, f"login returned {status}"
    return json.loads(body)["session_token"]


def post_agent(base, bearer, identifier, permission, parent=None):
    revision_status, revision_body = request(base, "/api/state/revision", token=bearer)
    assert revision_status == 200, f"Agent revision read returned {revision_status}"
    revision = json.loads(revision_body)["revision"]
    body = {"id": identifier, "ttl": 3600, "permissions": [permission]}
    if parent is not None:
        body["parent"] = parent
    headers = {"If-Match": f'"{revision}"',
               "Idempotency-Key": f"a05-agent-create-{identifier}"}
    return request(base, "/api/agents", body, bearer, headers)[0]


def check_agent_boundary(base):
    bearer = token(base)
    cases = (
        ("workspace", {"action": "directory.read", "resource": "workspace/example"}, None),
        ("entra", {"action": "directory.sync", "resource": "entra/example"}, None),
        ("wildcard", {"action": "directory.read", "resource": "*"}, None),
    )
    for identifier, permission, parent in cases:
        status = post_agent(base, bearer, f"a05-{identifier}", permission, parent)
        assert status == 400, f"Essentials accepted {identifier} agent ({status})"


def create_platform_state(base):
    status = post_agent(
        base, token(base), "a05-platform-state", {"action": "user.offboard", "resource": "*"}
    )
    assert status == 200, f"Platform agent creation returned {status}"


def archive_binary(archive, edition, directory):
    destination = directory / edition
    destination.mkdir()
    with tarfile.open(archive, "r:gz") as source:
        member = source.getmember("riauth")
        assert member.isfile(), f"{archive} has no server binary"
        with source.extractfile(member) as payload, (destination / "riauth").open("wb") as output:
            while chunk := payload.read(1024 * 1024):
                output.write(chunk)
    binary = destination / "riauth"
    binary.chmod(0o700)
    capabilities(binary, edition)
    return binary


def check_tool_archive(archive, name, directory, label=None):
    destination = directory / (label or name)
    destination.mkdir()
    with tarfile.open(archive, "r:gz") as source:
        member = source.getmember(name)
        assert member.isfile(), f"{archive} has no {name} binary"
        with source.extractfile(member) as payload, (destination / name).open("wb") as output:
            while chunk := payload.read(1024 * 1024):
                output.write(chunk)
    binary = destination / name
    binary.chmod(0o700)
    assert name in run(str(binary), "--version")
    return binary


def stop(process):
    process.terminate()
    try:
        process.wait(timeout=5)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait(timeout=5)


def archive_server(binary, config, log):
    with log.open("wb") as output:
        process = subprocess.Popen([str(binary), "--config", str(config), "serve"], stdout=output, stderr=output)
    return process


def assert_downgrade_rejection(downgrade, preflight):
    assert downgrade.returncode, downgrade.stdout + downgrade.stderr
    direct = json.loads(downgrade.stdout)
    assert direct["ok"] is False and "Platform" in direct["error"]["message"], direct
    assert preflight.returncode == 5, preflight.stdout + preflight.stderr
    report = json.loads(preflight.stdout)["data"]
    assert report["ready"] is False, report
    blockers = report["blockers"]
    assert any(item["resource"] == "meta/edition_provenance" for item in blockers), blockers
    assert any(item["resource"].startswith("agents/") and
               "Platform build" in item["reason"] for item in blockers), blockers


def check_archives(essentials, platform, maintenance, directory):
    binaries = {
        edition: archive_binary(archive, edition, directory)
        for edition, archive in (("essentials", essentials), ("platform", platform))
    }
    maintenance_binaries = {
        edition: check_tool_archive(archive, "riauth-maintenance", directory, f"{edition}-maintenance")
        for edition, archive in maintenance.items()
    }
    for edition, binary in binaries.items():
        root = directory / f"{edition}-instance"
        root.mkdir()
        config = root / "riauth.toml"
        listen = port()
        base = f"http://127.0.0.1:{listen}"
        run(str(maintenance_binaries[edition]), "--config", str(config), "--json", "--non-interactive", "init",
            "--issuer", base, "--listen", f"127.0.0.1:{listen}",
            "--data-dir", str(root / "data"), "--password-stdin", input=PASSWORD + "\n")
        log = root / "server.log"
        server = archive_server(binary, config, log)
        try:
            wait_ready(base, lambda: server.poll() is None)
            check_routes(base, edition)
            if edition == "essentials":
                check_agent_boundary(base)
            else:
                create_platform_state(base)
        except Exception as error:
            raise RuntimeError(f"{error}; server log: {log.read_text(errors='replace')[-4000:]}") from error
        finally:
            stop(server)
        if edition == "platform":
            downgrade = subprocess.run(
                [str(binaries["essentials"]), "--config", str(config), "--json", "serve"],
                text=True, capture_output=True, timeout=15,
            )
            preflight = subprocess.run(
                [str(maintenance_binaries["platform"]), "--config", str(config), "--json",
                 "transition-preflight", "--target", "essentials"],
                text=True, capture_output=True, timeout=15,
            )
            assert_downgrade_rejection(downgrade, preflight)
    print("Native archives: edition, route, agent issuance and downgrade checks passed")
    return maintenance_binaries["platform"]


def docker(*args, input=None, timeout=30):
    return run("docker", *args, input=input, timeout=timeout)


def check_image(edition, image, volume, listen):
    base = f"http://127.0.0.1:{listen}"
    issuer = f"https://127.0.0.1:{listen}"
    docker("run", "--rm", "-i", "-v", f"{volume}:/data", image,
           "--json", "--non-interactive", "init", "--issuer", issuer,
           "--listen", "0.0.0.0:9000", "--data-dir", "/data", "--password-stdin",
           input=PASSWORD + "\n", timeout=60)
    container = f"riauth-a05-{edition}-{listen}"
    docker("run", "-d", "--name", container, "-p", f"127.0.0.1:{listen}:9000",
           "-v", f"{volume}:/data", image, "serve")
    try:
        wait_ready(base, lambda: docker("inspect", "--format", "{{.State.Running}}", container).strip() == "true")
        check_routes(base, edition)
        if edition == "essentials":
            check_agent_boundary(base)
        else:
            create_platform_state(base)
    except Exception as error:
        raise RuntimeError(f"{error}; container log: {docker('logs', container)[-4000:]}") from error
    finally:
        docker("rm", "-f", container)


def check_images(essentials, platform, platform_maintenance):
    images = {"essentials": essentials, "platform": platform}
    volumes = {}
    try:
        for edition, image in images.items():
            volume = f"riauth-a05-{edition}-{port()}"
            volumes[edition] = volume
            docker("volume", "create", volume)
            label = docker("image", "inspect", "--format", '{{index .Config.Labels "org.riauth.edition"}}', image).strip()
            assert label == edition, f"{image} edition label is {label}"
            check_image(edition, image, volume, port())
        downgrade = subprocess.run(
            ["docker", "run", "--rm", "-v", f"{volumes['platform']}:/data", essentials,
             "--json", "serve"], text=True, capture_output=True, timeout=20,
        )
        pathlib.Path(platform_maintenance).chmod(0o755)
        preflight = subprocess.run(
            ["docker", "run", "--rm", "--entrypoint", "/usr/local/bin/riauth-maintenance",
             "-v", f"{volumes['platform']}:/data",
             "-v", f"{pathlib.Path(platform_maintenance).resolve()}:/usr/local/bin/riauth-maintenance:ro",
             platform, "--config", "/data/riauth.toml", "--json",
             "transition-preflight", "--target", "essentials"],
            text=True, capture_output=True, timeout=20,
        )
        assert_downgrade_rejection(downgrade, preflight)
    finally:
        for volume in volumes.values():
            docker("volume", "rm", "-f", volume)
    print("Loaded images: edition, route, agent issuance and downgrade checks passed")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--essentials-archive", type=pathlib.Path, required=True)
    parser.add_argument("--platform-archive", type=pathlib.Path, required=True)
    parser.add_argument("--essentials-maintenance-archive", type=pathlib.Path, required=True)
    parser.add_argument("--platform-maintenance-archive", type=pathlib.Path, required=True)
    parser.add_argument("--riauthctl-archive", type=pathlib.Path, required=True)
    parser.add_argument("--essentials-image")
    parser.add_argument("--platform-image")
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix="riauth-a05-") as temporary:
        directory = pathlib.Path(temporary)
        platform_maintenance = check_archives(
            args.essentials_archive, args.platform_archive,
            {"essentials": args.essentials_maintenance_archive,
             "platform": args.platform_maintenance_archive}, directory)
        check_tool_archive(args.riauthctl_archive, "riauthctl", directory)
        if args.essentials_image or args.platform_image:
            if not (args.essentials_image and args.platform_image):
                parser.error("both image names are required")
            check_images(args.essentials_image, args.platform_image, platform_maintenance)


if __name__ == "__main__":
    main()
