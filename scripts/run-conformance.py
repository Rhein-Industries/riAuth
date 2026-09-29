#!/usr/bin/env python3
"""Run one pinned OIDF plan; retain only metadata, never raw suite output."""

import argparse
import hashlib
import json
import os
import pathlib
import stat
import subprocess
import sys
import tempfile
import urllib.parse


PIN = "440eec8bac7b12b7389d7ca9cbc459b53507a443"
ROOT = pathlib.Path(__file__).resolve().parent.parent


def require(condition, message):
    if not condition:
        raise ValueError(message)


def git(*args, cwd):
    result = subprocess.run(["git", *args], cwd=cwd, text=True, capture_output=True, check=False)
    require(result.returncode == 0, "Git inspection failed for conformance source")
    return result.stdout.strip()


def validate_suite(path):
    require(path.is_dir(), "Independent suite directory is missing")
    suite = path.resolve(strict=True)
    require(suite.is_dir(), "Independent suite directory is missing")
    require(pathlib.Path(git("rev-parse", "--show-toplevel", cwd=suite)) == suite,
            "Suite path must be its Git checkout root")
    revision = git("rev-parse", "HEAD", cwd=suite)
    require(revision == PIN, f"Expected independent suite commit {PIN}")
    require(not git("status", "--porcelain", "--untracked-files=all", cwd=suite),
            "Pinned suite checkout has local changes or untracked files")
    entry = suite / "scripts" / "run-test-plan.py"
    require(entry.is_file() and not entry.is_symlink(), "Pinned suite runner is missing")
    return suite, revision


def validate_config(path):
    require(not path.is_symlink(), "Private suite configuration cannot be a symlink")
    require(path.is_file(), "Private suite configuration is missing")
    config = path.resolve(strict=True)
    info = config.stat()
    require(stat.S_ISREG(info.st_mode), "Private suite configuration must be a regular file")
    require(info.st_uid == os.geteuid() and info.st_mode & 0o077 == 0
            and info.st_mode & stat.S_IRUSR,
            "Private suite configuration must be owner-readable and owner-only")
    require(info.st_size <= 2 * 1024 * 1024, "Private suite configuration exceeds 2 MiB")
    return config


def validate_server(value):
    require(value and value == value.strip() and not any(ord(char) < 32 for char in value),
            "CONFORMANCE_SERVER is missing or malformed")
    parsed = urllib.parse.urlsplit(value)
    require(parsed.scheme in ("https", "http") and parsed.hostname
            and not parsed.username and not parsed.password
            and not parsed.query and not parsed.fragment,
            "CONFORMANCE_SERVER must be a URL without credentials, query or fragment")
    try:
        parsed.port
    except ValueError as error:
        raise ValueError("CONFORMANCE_SERVER port is invalid") from error
    require(parsed.scheme == "https" or parsed.hostname in ("localhost", "127.0.0.1", "::1"),
            "CONFORMANCE_SERVER requires HTTPS except on loopback")
    return {"scheme": parsed.scheme,
            "url_sha256": hashlib.sha256(value.encode()).hexdigest()}


def validate_plan(value):
    require(value and value == value.strip() and len(value) <= 200
            and not any(ord(char) < 32 or ord(char) == 127 for char in value),
            "Plan must be one exact, nonempty upstream name/variant string")
    return value


def file_digest(path):
    checksum = hashlib.sha256()
    size = 0
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            checksum.update(block)
            size += len(block)
    return {"sha256": checksum.hexdigest(), "bytes": size}


def export_digest(directory):
    checksum = hashlib.sha256()
    count = 0
    total = 0
    for path in sorted(directory.rglob("*")):
        require(not path.is_symlink(), "Suite export contains a symlink")
        if path.is_dir():
            continue
        require(path.is_file(), "Suite export contains a non-file entry")
        record = file_digest(path)
        checksum.update(path.relative_to(directory).as_posix().encode() + b"\0")
        checksum.update(record["sha256"].encode() + b"\0")
        count += 1
        total += record["bytes"]
    return {"sha256": checksum.hexdigest(), "files": count, "bytes": total}


def write_private_report(output, report):
    output.parent.mkdir(parents=True, exist_ok=True)
    require(not output.exists() and not output.is_symlink(),
            "Evidence output must be a new directory")
    output.mkdir(mode=0o700)
    descriptor = os.open(output / "run.json", os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(descriptor, "w") as destination:
        json.dump(report, destination, indent=2, sort_keys=True)
        destination.write("\n")


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--suite", type=pathlib.Path, required=True)
    parser.add_argument("--config", type=pathlib.Path, required=True)
    parser.add_argument("--plan", required=True, help="Exact upstream plan name and variants")
    parser.add_argument("--output", type=pathlib.Path, default=pathlib.Path("target/conformance"))
    args = parser.parse_args(argv)
    server = validate_server(os.environ.get("CONFORMANCE_SERVER", ""))
    token = os.environ.get("CONFORMANCE_TOKEN", "")
    require(token and not any(ord(char) < 32 or ord(char) == 127 for char in token),
            "CONFORMANCE_TOKEN must be configured for the pilot")
    plan = validate_plan(args.plan)
    suite, revision = validate_suite(args.suite)
    config = validate_config(args.config)
    config_sha256 = file_digest(config)["sha256"]
    require(not args.output.is_symlink(), "Evidence output cannot be a symlink")
    output = args.output.resolve()
    require(not output.is_relative_to(suite), "Evidence output must be outside the suite checkout")
    require(not output.exists() and not output.is_symlink(),
            "Evidence output must be a new directory")
    riauth_commit = git("rev-parse", "HEAD", cwd=ROOT)
    require(not git("status", "--porcelain", "--untracked-files=no", cwd=ROOT),
            "riAuth tracked source has local changes")

    with tempfile.TemporaryDirectory(prefix="riauth-oidf-private-") as temporary:
        private = pathlib.Path(temporary)
        exports = private / "exports"
        exports.mkdir(mode=0o700)
        stdout = private / "stdout"
        stderr = private / "stderr"
        command = [sys.executable, str(suite / "scripts" / "run-test-plan.py"),
                   "--no-parallel", "--export-dir", str(exports), plan, str(config)]
        with stdout.open("wb") as out, stderr.open("wb") as err:
            result = subprocess.run(command, cwd=suite, stdout=out, stderr=err, check=False)
        require(file_digest(config)["sha256"] == config_sha256,
                "Private suite configuration changed during the pilot")
        exported = export_digest(exports)
        evidence_complete = exported["files"] > 0
        report = {
            "schema": "riauth.conformance-pilot/v2",
            "scope": "one_local_pilot_plan",
            "certification_claim": False,
            "suite_commit": revision,
            "riauth_commit": riauth_commit,
            "plan": plan,
            "server": server,
            "config_sha256": config_sha256,
            "exit_code": result.returncode,
            "passed": result.returncode == 0 and evidence_complete,
            "evidence_complete": evidence_complete,
            "stdout": file_digest(stdout),
            "stderr": file_digest(stderr),
            "exports": exported,
            "runner_managed_raw_output_retained": False,
        }
    write_private_report(output, report)
    print(f"Pilot exit {result.returncode}; metadata-only evidence: {output / 'run.json'}")
    return result.returncode or (0 if evidence_complete else 1)


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError) as error:
        raise SystemExit(f"conformance preflight failed: {error}") from error
