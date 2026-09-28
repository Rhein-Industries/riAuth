#!/usr/bin/env python3
"""Check A03 source boundaries and report explicit crate-reference edges.

This is a source-level inventory, not a Rust call graph: `super` imports, macros,
trait dispatch and runtime calls are outside its counts. A counted edge means one
source file explicitly names a crate-root module in code or a grouped `use`.
"""

import argparse
import collections
import json
from pathlib import Path
import re


ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "src"
MANAGEMENT = {
    "agent", "context", "lifecycle", "offboarding", "operations",
    "provisioning", "registration", "reports", "resource", "state",
}
API_SERVER = {"api"}
CLIENT = {"cli", "main"}
STORAGE = {"store", "postgres_store"}
PROTOCOL = {
    "assurance", "authenticator", "authorization", "browser", "claims",
    "cloud_directory", "device_trust", "directory", "dpop", "event_map",
    "exchange", "issuer", "jose", "keyring", "ldap_server", "logout",
    "mtls", "oidc", "outpost", "pam", "passkey", "password_history",
    "portal", "provider", "proxy_server", "radius", "response", "saml",
    "scim", "session_protocol", "signin", "source", "ssf", "windows_login",
}
IDENTITY_ALLOWED = {"crypto", "error", "jose", "model", "identity"}
STORAGE_FORBIDDEN = {"core", "agent", "windows_login", "logout", "ssf"}
MODEL_FORBIDDEN = {"portal", "saml", "radius", "ldap_server", "outpost"}
CONTEXT_FORBIDDEN = {"core"}


def root_module(path: Path) -> str:
    return path.relative_to(SRC).parts[0].removesuffix(".rs")


def group(module: str) -> str:
    if module == "identity":
        return "identity"
    if module == "core":
        return "core_engine"
    if module == "model":
        return "model"
    if module in STORAGE:
        return "storage"
    if module in MANAGEMENT:
        return "management"
    if module in API_SERVER:
        return "api_server"
    if module in CLIENT:
        return "client"
    if module in PROTOCOL:
        return "protocol"
    return "shared_support"


def grouped_roots(source: str) -> list[str]:
    roots = []
    for match in re.finditer(r"crate::\s*\{", source):
        start = match.end()
        depth = 0
        field_start = start
        for pos in range(start, len(source)):
            char = source[pos]
            if char == "{":
                depth += 1
            elif char == "}":
                if depth == 0:
                    field = source[field_start:pos].strip()
                    if field:
                        root = re.match(r"([A-Za-z_]\w*)", field)
                        if root:
                            roots.append(root.group(1))
                    break
                depth -= 1
            elif char == "," and depth == 0:
                field = source[field_start:pos].strip()
                root = re.match(r"([A-Za-z_]\w*)", field)
                if root:
                    roots.append(root.group(1))
                field_start = pos + 1
    return roots


def references(path: Path) -> set[str]:
    source = path.read_text()
    source = re.sub(r"(?m)^\s*//[^\n]*$", "", source)
    source = re.sub(r"/\*.*?\*/", "", source, flags=re.S)
    refs = set(re.findall(r"\bcrate::([A-Za-z_]\w*)", source))
    refs.update(grouped_roots(source))
    refs.discard("self")
    refs.discard("super")
    return refs


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", type=Path, help="write the measured graph to this path")
    args = parser.parse_args()
    paths = sorted(SRC.rglob("*.rs"))
    module_groups = {root_module(path): group(root_module(path)) for path in paths}
    edges = collections.defaultdict(set)
    errors = []
    for path in paths:
        source_module = root_module(path)
        source_group = module_groups[source_module]
        refs = references(path)
        for target in refs:
            if target in module_groups:
                edges[(source_group, module_groups[target])].add(path.relative_to(ROOT).as_posix())
        if source_group == "identity":
            forbidden = refs - IDENTITY_ALLOWED
            if forbidden:
                errors.append(f"{path.relative_to(ROOT)}: identity refers to {sorted(forbidden)}")
            if re.search(r"\bimpl\s+Core\b", path.read_text()):
                errors.append(f"{path.relative_to(ROOT)}: identity implements Core")
        if source_group == "storage" and refs & STORAGE_FORBIDDEN:
            errors.append(f"{path.relative_to(ROOT)}: storage refers to {sorted(refs & STORAGE_FORBIDDEN)}")
        if source_group == "model" and refs & MODEL_FORBIDDEN:
            errors.append(f"{path.relative_to(ROOT)}: model refers to {sorted(refs & MODEL_FORBIDDEN)}")
        if source_module == "context" and refs & CONTEXT_FORBIDDEN:
            errors.append(f"{path.relative_to(ROOT)}: context refers to {sorted(refs & CONTEXT_FORBIDDEN)}")
    graph = {
        "method": "Distinct Rust source files with explicit crate-root references, including grouped use; excludes super imports, macros and runtime dispatch",
        "source_files": len(paths),
        "module_groups": dict(sorted(module_groups.items())),
        "group_edges": [
            {"from": left, "to": right, "files": len(files), "paths": sorted(files)}
            for (left, right), files in sorted(edges.items())
        ],
        "boundary_checks": {
            "identity_allowed_crate_roots": sorted(IDENTITY_ALLOWED),
            "identity_forbidden_reference_files": sum(
                bool(references(path) - IDENTITY_ALLOWED)
                for path in paths if group(root_module(path)) == "identity"
            ),
            "identity_storage_reference_files": sum(
                "store" in references(path)
                for path in paths if group(root_module(path)) == "identity"
            ),
            "storage_adapter_reference_files": sum(
                bool(references(path) & STORAGE_FORBIDDEN)
                for path in paths if group(root_module(path)) == "storage"
            ),
            "model_adapter_reference_files": sum(
                bool(references(path) & MODEL_FORBIDDEN)
                for path in paths if group(root_module(path)) == "model"
            ),
            "context_core_reference_files": sum(
                "core" in references(path)
                for path in paths if root_module(path) == "context"
            ),
        },
    }
    if args.json:
        args.json.parent.mkdir(parents=True, exist_ok=True)
        args.json.write_text(json.dumps(graph, indent=2) + "\n")
    for name, count in graph["boundary_checks"].items():
        if name.endswith("_files"):
            print(f"{name}: {count}")
    print(f"Explicit crate-reference graph checked across {len(paths)} Rust source files")
    if errors:
        raise SystemExit("Module boundary check failed:\n" + "\n".join(errors))


if __name__ == "__main__":
    main()
