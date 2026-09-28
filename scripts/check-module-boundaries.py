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
SERVER_ASSEMBLY = {"assembly"}
PROTOCOL = {
    "assurance", "authenticator", "authorization", "browser", "claims",
    "cloud_directory", "device_trust", "directory", "dpop", "event_map",
    "exchange", "issuer", "jose", "keyring", "ldap_server", "logout",
    "mtls", "oidc", "outpost", "pam", "passkey", "password",
    "portal", "provider", "proxy_server", "radius", "response", "saml",
    "scim", "session_protocol", "signin", "source", "ssf", "windows_login",
}
IDENTITY_ALLOWED = {"crypto", "error", "model", "identity"}
STORAGE_FORBIDDEN = {"core", "agent", "windows_login", "logout", "ssf", "assembly", "identity"}
MODEL_FORBIDDEN = {"portal", "saml", "radius", "ldap_server", "outpost", "jose", "encryption"}
MODEL_CONFIG_LEGACY = {
    "jose": {"ClientAuthMethod", "MachineTrust"},
    "exchange": {"ExchangePolicy"},
    "encryption": {"EncryptionKey"},
}
CONTEXT_FORBIDDEN = {"core"}
RAW_STRING = re.compile(r'(?:br|r)(?P<hashes>#{0,255})"')
CHAR_LITERAL = re.compile(r"(?:b)?'(?:\\(?:u\{[0-9A-Fa-f_]+\}|x[0-9A-Fa-f]{2}|.)|[^'\\\n])'")


def masked_rust_source(source: str) -> str:
    """Mask comments and literals, preserving line breaks and import punctuation."""
    def mask(fragment: str) -> str:
        return "".join(char if char in "\r\n" else " " for char in fragment)

    result = []
    pos = 0
    while pos < len(source):
        start = pos
        if source.startswith("//", pos):
            newline = source.find("\n", pos)
            pos = len(source) if newline == -1 else newline
        elif source.startswith("/*", pos):
            depth = 1
            pos += 2
            while pos < len(source) and depth:
                if source.startswith("/*", pos):
                    depth += 1
                    pos += 2
                elif source.startswith("*/", pos):
                    depth -= 1
                    pos += 2
                else:
                    pos += 1
        else:
            token_start = pos == 0 or not (source[pos - 1].isalnum() or source[pos - 1] == "_")
            raw = RAW_STRING.match(source, pos) if token_start else None
            char = CHAR_LITERAL.match(source, pos) if token_start else None
            if raw:
                closing = '"' + raw.group("hashes")
                end = source.find(closing, raw.end())
                pos = len(source) if end == -1 else end + len(closing)
            elif char:
                pos = char.end()
            elif source[pos] == '"':
                pos += 1
                while pos < len(source):
                    if source[pos] == "\\":
                        pos += 2
                    elif source[pos] == '"':
                        pos += 1
                        break
                    else:
                        pos += 1
            else:
                result.append(source[pos])
                pos += 1
                continue
        result.append(mask(source[start:pos]))
    return "".join(result)


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
    if module in SERVER_ASSEMBLY:
        return "server_assembly"
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
    for match in re.finditer(r"crate\s*::\s*\{", source):
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


def split_use_branches(tree: str) -> list[str]:
    branches = []
    depth = 0
    start = 0
    for pos, char in enumerate(tree):
        if char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
        elif char == "," and depth == 0:
            branches.append(tree[start:pos])
            start = pos + 1
    branches.append(tree[start:])
    return branches


def expanded_use_paths(tree: str, prefix: tuple[str, ...] = ()) -> list[tuple[str, ...]]:
    paths = []
    for branch in split_use_branches(tree):
        branch = branch.strip()
        if not branch:
            continue
        brace = branch.find("{")
        if brace != -1 and branch.endswith("}"):
            head = branch[:brace].strip().removesuffix("::")
            parts = tuple(part.strip() for part in head.split("::") if part.strip())
            paths.extend(expanded_use_paths(branch[brace + 1:-1], prefix + parts))
        else:
            head = re.split(r"\s+as\s+", branch, maxsplit=1)[0]
            parts = tuple(part.strip() for part in head.split("::") if part.strip())
            paths.append(prefix + parts)
    return paths


def legacy_client_config_references(source: str) -> set[str]:
    source = masked_rust_source(source)
    paths = set(re.findall(r"\bcrate\s*::\s*(\w+)\s*::\s*(\w+)\b", source))
    for match in re.finditer(r"\buse\s+crate\s*::\s*(.*?);", source, flags=re.S):
        paths.update(
            path[:2] for path in expanded_use_paths(match.group(1)) if len(path) >= 2
        )
    return {
        f"{root}::{leaf}" for root, leaf in paths
        if leaf in MODEL_CONFIG_LEGACY.get(root, ())
    }


def references(path: Path) -> set[str]:
    source = masked_rust_source(path.read_text())
    refs = set(re.findall(r"\bcrate\s*::\s*([A-Za-z_]\w*)", source))
    refs.update(grouped_roots(source))
    refs.discard("self")
    refs.discard("super")
    return refs


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", type=Path, help="write the measured graph to this path")
    args = parser.parse_args()
    paths = sorted(SRC.rglob("*.rs"))
    legacy_password_history = SRC / "password_history.rs"
    module_groups = {root_module(path): group(root_module(path)) for path in paths}
    edges = collections.defaultdict(set)
    errors = []
    if legacy_password_history.exists():
        errors.append("src/password_history.rs: password history policy belongs in identity")
    for path in paths:
        source_module = root_module(path)
        source_group = module_groups[source_module]
        refs = references(path)
        for target in refs:
            if target in module_groups:
                edges[(source_group, module_groups[target])].add(path.relative_to(ROOT).as_posix())
        if source_group == "identity":
            protocol_refs = refs & PROTOCOL
            if protocol_refs:
                errors.append(f"{path.relative_to(ROOT)}: identity refers to protocol {sorted(protocol_refs)}")
            forbidden = refs - IDENTITY_ALLOWED - PROTOCOL
            if forbidden:
                errors.append(f"{path.relative_to(ROOT)}: identity refers to {sorted(forbidden)}")
            if re.search(r"\bimpl\s+Core\b", path.read_text()):
                errors.append(f"{path.relative_to(ROOT)}: identity implements Core")
        if source_group == "storage" and refs & STORAGE_FORBIDDEN:
            errors.append(f"{path.relative_to(ROOT)}: storage refers to {sorted(refs & STORAGE_FORBIDDEN)}")
        if source_group == "storage" and refs & PROTOCOL:
            errors.append(f"{path.relative_to(ROOT)}: storage refers to protocol {sorted(refs & PROTOCOL)}")
        if source_module == "dpop" and refs & STORAGE:
            errors.append(f"{path.relative_to(ROOT)}: DPoP refers directly to storage")
        if source_module == "jose" and refs & STORAGE:
            errors.append(f"{path.relative_to(ROOT)}: JOSE refers directly to storage")
        if source_module == "issuer" and refs & (STORAGE | {"core"}):
            errors.append(f"{path.relative_to(ROOT)}: issuer refers directly to storage or Core")
        if source_module in {"keyring", "response"} and refs & (STORAGE | {"core"}):
            errors.append(f"{path.relative_to(ROOT)}: {source_module} refers directly to storage or Core")
        if source_module == "claims" and refs & (STORAGE | {"core"}):
            errors.append(f"{path.relative_to(ROOT)}: claims refers directly to storage or Core")
        if source_module == "event_map" and refs & (STORAGE | {"core"}):
            errors.append(f"{path.relative_to(ROOT)}: event_map refers directly to storage or Core")
        if source_module == "device_trust" and refs & (STORAGE | {"core"}):
            errors.append(f"{path.relative_to(ROOT)}: device_trust refers directly to storage or Core")
        if source_module == "session_protocol" and refs & (STORAGE | {"core"}):
            errors.append(f"{path.relative_to(ROOT)}: session_protocol refers directly to storage or Core")
        if source_module == "oidc" and refs & (STORAGE | {"core"}):
            errors.append(f"{path.relative_to(ROOT)}: OIDC refers directly to storage or Core")
        if source_module == "authorization" and refs & (STORAGE | {"core"}):
            errors.append(f"{path.relative_to(ROOT)}: authorization refers directly to storage or Core")
        if source_module == "exchange" and refs & (STORAGE | {"core"}):
            errors.append(f"{path.relative_to(ROOT)}: exchange refers directly to storage or Core")
        if source_module == "ssf" and refs & (STORAGE | {"core"}):
            errors.append(f"{path.relative_to(ROOT)}: SSF refers directly to storage or Core")
        if path == SRC / "saml.rs" and refs & (STORAGE | {"core"}):
            errors.append("src/saml.rs: SAML browser SSO refers directly to storage or Core")
        if path == SRC / "saml/logout.rs" and refs & (STORAGE | {"core"}):
            errors.append("src/saml/logout.rs: SAML logout refers directly to storage or Core")
        if path == SRC / "passkey.rs" and refs & (STORAGE | {"core", "assembly"}):
            errors.append("src/passkey.rs: passkey protocol refers directly to storage, Core or assembly")
        if path == SRC / "authenticator.rs" and refs & (STORAGE | {"core", "assembly"}):
            errors.append("src/authenticator.rs: authenticator protocol refers directly to storage, Core or assembly")
        if path == SRC / "password.rs" and refs & (STORAGE | {"core", "assembly"}):
            errors.append("src/password.rs: password protocol refers directly to storage, Core or assembly")
        if path == SRC / "logout.rs" and refs & (STORAGE | {"core", "assembly"}):
            errors.append("src/logout.rs: logout protocol refers directly to storage, Core or assembly")
        if path == SRC / "mtls.rs" and refs & (STORAGE | {"core", "assembly"}):
            errors.append("src/mtls.rs: client-certificate protocol refers directly to storage, Core or assembly")
        if path == SRC / "portal/self_service.rs" and refs & (STORAGE | {"core", "assembly"}):
            errors.append("src/portal/self_service.rs: browser self-service refers directly to storage, Core or assembly")
        if path == SRC / "portal/mfa.rs" and refs & (STORAGE | {"core", "assembly"}):
            errors.append("src/portal/mfa.rs: browser MFA refers directly to storage, Core or assembly")
        if path == SRC / "portal/sources.rs" and (
            refs & (STORAGE | {"core", "assembly"})
            or re.search(r"\bimpl\s+Core\b|\.\s*store\b", masked_rust_source(path.read_text()))
        ):
            errors.append("src/portal/sources.rs: browser source adapter refers directly to storage, Core or assembly")
        if path == SRC / "proxy_server.rs" and (
            refs & STORAGE or re.search(r"\.\s*store\b", masked_rust_source(path.read_text()))
        ):
            errors.append("src/proxy_server.rs: proxy transport refers directly to storage")
        if path == SRC / "windows_login.rs" and re.search(
            r"\.\s*store\s*\.\s*read\s*\(", masked_rust_source(path.read_text())
        ):
            errors.append("src/windows_login.rs: Windows protocol directly reads storage")
        if path == SRC / "windows_login.rs" and re.search(
            r"\.\s*store\s*\.\s*write\s*\(", masked_rust_source(path.read_text())
        ):
            errors.append("src/windows_login.rs: Windows protocol directly writes storage")
        if path == SRC / "ldap_server.rs" and (
            refs & STORAGE or re.search(r"\.\s*store\b", masked_rust_source(path.read_text()))
        ):
            errors.append("src/ldap_server.rs: LDAP protocol refers directly to storage")
        if path == SRC / "radius.rs":
            radius_source = masked_rust_source(path.read_text())
            if "core" in refs or re.search(r"\bCore\b", radius_source):
                errors.append("src/radius.rs: RADIUS protocol refers directly to Core")
            if re.search(r"\.\s*store\s*\.\s*read\s*\(", radius_source) or re.search(
                r"\bfn\s+(?:client|radius_identity)\s*\(", radius_source
            ):
                errors.append("src/radius.rs: RADIUS client or identity read belongs in assembly")
            if re.search(r"\.\s*store\s*\.\s*write\s*\(", radius_source):
                errors.append("src/radius.rs: RADIUS replay or close write belongs in assembly")
            if refs & STORAGE or re.search(r"\.\s*store\b", radius_source):
                errors.append("src/radius.rs: RADIUS protocol refers directly to storage")
        if path == SRC / "radius/eap.rs":
            eap_source = masked_rust_source(path.read_text())
            if (
                refs & (STORAGE | {"core"})
                or re.search(r"\bfn\s+validate_identity\s*\(", eap_source)
                or re.search(r"\.\s*store\s*\.\s*(?:read|write)\s*\(", eap_source)
                or re.search(
                    r"\.\s*mutation\s*\(|\btx\s*\.\s*(?:get|put|delete|list|maintenance_page)\s*(?:<|\()",
                    eap_source,
                )
            ):
                errors.append("src/radius/eap.rs: EAP Core or storage access belongs in assembly")
        if source_group == "model" and refs & PROTOCOL:
            errors.append(f"{path.relative_to(ROOT)}: model refers to protocol {sorted(refs & PROTOCOL)}")
        if source_group == "model" and refs & MODEL_FORBIDDEN:
            errors.append(f"{path.relative_to(ROOT)}: model refers to {sorted(refs & MODEL_FORBIDDEN)}")
        if source_group == "model":
            legacy_config = legacy_client_config_references(path.read_text())
            if legacy_config:
                errors.append(f"{path.relative_to(ROOT)}: model refers to legacy client configuration types {sorted(legacy_config)}")
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
            "identity_protocol_reference_files": sum(
                bool(references(path) & PROTOCOL)
                for path in paths if group(root_module(path)) == "identity"
            ),
            "storage_adapter_reference_files": sum(
                bool(references(path) & STORAGE_FORBIDDEN)
                for path in paths if group(root_module(path)) == "storage"
            ),
            "storage_identity_reference_files": sum(
                "identity" in references(path)
                for path in paths if group(root_module(path)) == "storage"
            ),
            "storage_protocol_reference_files": sum(
                bool(references(path) & PROTOCOL)
                for path in paths if group(root_module(path)) == "storage"
            ),
            "dpop_storage_reference_files": sum(
                bool(references(path) & STORAGE)
                for path in paths if root_module(path) == "dpop"
            ),
            "jose_storage_reference_files": sum(
                bool(references(path) & STORAGE)
                for path in paths if root_module(path) == "jose"
            ),
            "issuer_storage_reference_files": sum(
                bool(references(path) & STORAGE)
                for path in paths if root_module(path) == "issuer"
            ),
            "issuer_core_reference_files": sum(
                "core" in references(path)
                for path in paths if root_module(path) == "issuer"
            ),
            "keyring_storage_reference_files": sum(
                bool(references(path) & STORAGE)
                for path in paths if root_module(path) == "keyring"
            ),
            "keyring_core_reference_files": sum(
                "core" in references(path)
                for path in paths if root_module(path) == "keyring"
            ),
            "response_storage_reference_files": sum(
                bool(references(path) & STORAGE)
                for path in paths if root_module(path) == "response"
            ),
            "response_core_reference_files": sum(
                "core" in references(path)
                for path in paths if root_module(path) == "response"
            ),
            "claims_storage_reference_files": sum(
                bool(references(path) & STORAGE)
                for path in paths if root_module(path) == "claims"
            ),
            "claims_core_reference_files": sum(
                "core" in references(path)
                for path in paths if root_module(path) == "claims"
            ),
            "event_map_storage_reference_files": sum(
                bool(references(path) & STORAGE)
                for path in paths if root_module(path) == "event_map"
            ),
            "event_map_core_reference_files": sum(
                "core" in references(path)
                for path in paths if root_module(path) == "event_map"
            ),
            "device_trust_storage_reference_files": sum(
                bool(references(path) & STORAGE)
                for path in paths if root_module(path) == "device_trust"
            ),
            "device_trust_core_reference_files": sum(
                "core" in references(path)
                for path in paths if root_module(path) == "device_trust"
            ),
            "session_protocol_storage_reference_files": sum(
                bool(references(path) & STORAGE)
                for path in paths if root_module(path) == "session_protocol"
            ),
            "session_protocol_core_reference_files": sum(
                "core" in references(path)
                for path in paths if root_module(path) == "session_protocol"
            ),
            "oidc_storage_reference_files": sum(
                bool(references(path) & STORAGE)
                for path in paths if root_module(path) == "oidc"
            ),
            "oidc_core_reference_files": sum(
                "core" in references(path)
                for path in paths if root_module(path) == "oidc"
            ),
            "authorization_storage_reference_files": sum(
                bool(references(path) & STORAGE)
                for path in paths if root_module(path) == "authorization"
            ),
            "authorization_core_reference_files": sum(
                "core" in references(path)
                for path in paths if root_module(path) == "authorization"
            ),
            "exchange_storage_reference_files": sum(
                bool(references(path) & STORAGE)
                for path in paths if root_module(path) == "exchange"
            ),
            "exchange_core_reference_files": sum(
                "core" in references(path)
                for path in paths if root_module(path) == "exchange"
            ),
            "ssf_storage_reference_files": sum(
                bool(references(path) & STORAGE)
                for path in paths if root_module(path) == "ssf"
            ),
            "ssf_core_reference_files": sum(
                "core" in references(path)
                for path in paths if root_module(path) == "ssf"
            ),
            "saml_browser_storage_reference_files": int(
                bool(references(SRC / "saml.rs") & STORAGE)
            ),
            "saml_browser_core_reference_files": int(
                "core" in references(SRC / "saml.rs")
            ),
            "saml_logout_storage_reference_files": int(
                bool(references(SRC / "saml/logout.rs") & STORAGE)
            ),
            "saml_logout_core_reference_files": int(
                "core" in references(SRC / "saml/logout.rs")
            ),
            "passkey_storage_reference_files": int(
                bool(references(SRC / "passkey.rs") & STORAGE)
            ),
            "passkey_core_reference_files": int(
                "core" in references(SRC / "passkey.rs")
            ),
            "authenticator_storage_reference_files": int(
                bool(references(SRC / "authenticator.rs") & STORAGE)
            ),
            "authenticator_core_reference_files": int(
                "core" in references(SRC / "authenticator.rs")
            ),
            "password_storage_reference_files": int(
                bool(references(SRC / "password.rs") & STORAGE)
            ),
            "password_core_reference_files": int(
                "core" in references(SRC / "password.rs")
            ),
            "logout_storage_reference_files": int(
                bool(references(SRC / "logout.rs") & STORAGE)
            ),
            "logout_core_reference_files": int(
                "core" in references(SRC / "logout.rs")
            ),
            "mtls_storage_reference_files": int(
                bool(references(SRC / "mtls.rs") & STORAGE)
            ),
            "mtls_core_reference_files": int(
                "core" in references(SRC / "mtls.rs")
            ),
            "portal_self_service_storage_reference_files": int(
                bool(references(SRC / "portal/self_service.rs") & STORAGE)
            ),
            "portal_self_service_core_reference_files": int(
                "core" in references(SRC / "portal/self_service.rs")
            ),
            "model_protocol_reference_files": sum(
                bool(references(path) & PROTOCOL)
                for path in paths if group(root_module(path)) == "model"
            ),
            "model_adapter_reference_files": sum(
                bool(references(path) & MODEL_FORBIDDEN)
                for path in paths if group(root_module(path)) == "model"
            ),
            "model_legacy_client_config_reference_files": sum(
                bool(legacy_client_config_references(path.read_text()))
                for path in paths if group(root_module(path)) == "model"
            ),
            "context_core_reference_files": sum(
                "core" in references(path)
                for path in paths if root_module(path) == "context"
            ),
            "password_history_adapter_files": int(legacy_password_history.exists()),
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
