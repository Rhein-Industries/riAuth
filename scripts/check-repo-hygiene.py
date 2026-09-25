#!/usr/bin/env python3
"""Reject private files and generated output accidentally added to Git."""

import pathlib
import re
import subprocess


ROOT = pathlib.Path(__file__).resolve().parents[1]
GENERATED_DIRS = {
    "target", "node_modules", "build", "dist", "coverage", "htmlcov",
    "playwright-report", "test-results", "__pycache__", ".pytest_cache",
    ".mypy_cache", ".ruff_cache", ".idea", ".next", ".svelte-kit",
    ".terraform",
}
PRIVATE_DIRS = {"deployment-private", ".archive", "secrets"}
PRIVATE_SUFFIXES = (
    ".pem", ".key", ".p8", ".p12", ".pfx", ".jks", ".keystore",
    ".crt", ".cer", ".redb", ".db", ".sqlite", ".sqlite3",
    ".backup", ".bak", ".dump", ".tfstate", ".tfvars", ".tfvars.json",
)
GENERATED_SUFFIXES = (".log", ".pyc", ".pyo", ".pyd", ".swp", ".swo", "~")
PRIVATE_KEY_HEADER = (
    "-----BEGIN (RSA |EC |DSA |OPENSSH |ENCRYPTED )?PRIVATE KEY-----"
)


def blocked_reason(name: str) -> str | None:
    parts = pathlib.PurePosixPath(name).parts
    basename = parts[-1]
    if any(part in PRIVATE_DIRS for part in parts[:-1]) or parts[0] in {"data", "migration"}:
        return "private deployment data"
    if any(part in GENERATED_DIRS for part in parts[:-1]) or (
        len(parts) > 1 and parts[0].startswith("target-")
    ) or parts[:2] == ("fuzz", "artifacts"):
        return "generated output"
    if basename in {
        ".DS_Store", "Thumbs.db", "backup.json", "deployer.json", "reader.json",
        "directory-agent.json", "replacement.json", "source-transaction.json",
        "authentik-import.json",
    } or basename.endswith(("-credential.json", "-session.json")):
        return "local or private file"
    if basename.endswith(".env") or (
        basename.startswith(".env")
        and basename not in {".env.example", ".env.sample", ".env.template"}
    ) or (
        ".env." in basename
        and not basename.endswith((".env.example", ".env.sample", ".env.template"))
    ):
        return "local environment configuration"
    if basename in {
        ".netrc", ".npmrc", ".pypirc", ".pgpass", "credentials.json",
        "id_rsa", "id_ed25519", "terraform.tfvars", "terraform.tfvars.json",
    } or (basename.startswith("service-account") and basename.endswith(".json")):
        return "local credentials"
    if basename.endswith(("-password", "-secret", "-token")):
        return "local credentials"
    if len(parts) == 1 and (
        basename == "riauth.toml"
        or basename == ".riauth-session.json"
        or basename == "session.json"
        or basename.endswith("-session.json")
    ):
        return "local configuration or session"
    if basename.endswith(PRIVATE_SUFFIXES) or re.search(
        r"\.(?:redb|db|sqlite|sqlite3)-|\.tfstate\.", basename
    ):
        return "credential, database, or backup file"
    if basename.endswith(GENERATED_SUFFIXES):
        return "generated output"
    return None


raw_names = subprocess.check_output(
    ["git", "ls-files", "--cached", "-z"], cwd=ROOT
).split(b"\0")
names = [name.decode(errors="surrogateescape") for name in raw_names if name]
errors = []
for name in names:
    reason = blocked_reason(name)
    if reason:
        errors.append(f"{name}: {reason}")

# Search index blobs: local edits to a staged file cannot hide staged material.
key_search = subprocess.run(
    ["git", "grep", "--cached", "-l", "-z", "-E", "-e", PRIVATE_KEY_HEADER, "--", "."],
    cwd=ROOT,
    capture_output=True,
    check=False,
)
if key_search.returncode not in {0, 1}:
    raise SystemExit(key_search.stderr.decode(errors="replace"))
for name in key_search.stdout.decode(errors="surrogateescape").rstrip("\0").split("\0"):
    if name:
        errors.append(f"{name}: private key material")

if errors:
    raise SystemExit("Tracked-file hygiene check failed:\n" + "\n".join(errors))
print(f"Tracked-file hygiene checked ({len(names)} files)")
