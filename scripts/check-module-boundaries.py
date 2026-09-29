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


def rust_function_body(source: str, name: str) -> str | None:
    """Find a named function body in masked Rust source for a narrow boundary check."""
    match = re.search(rf"\bfn\s+{re.escape(name)}\s*\(", source)
    if match is None:
        return None
    start = source.find("{", match.end())
    if start == -1:
        return None
    depth = 0
    for pos in range(start, len(source)):
        if source[pos] == "{":
            depth += 1
        elif source[pos] == "}":
            depth -= 1
            if depth == 0:
                return source[start + 1:pos]
    return None


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
        if path == SRC / "proxy_server.rs" and (
            "core" in refs or re.search(r"\bCore\b", masked_rust_source(path.read_text()))
        ):
            errors.append("src/proxy_server.rs: proxy transport refers directly to Core")
        if path == SRC / "proxy_server.rs" and re.search(
            r"\bApp\b|\.\s*run\s*\(", masked_rust_source(path.read_text())
        ):
            errors.append("src/proxy_server.rs: proxy request Core dispatch belongs in assembly")
        if path == SRC / "windows_login.rs" and (
            "core" in refs or re.search(r"\bCore\b", masked_rust_source(path.read_text()))
        ):
            errors.append("src/windows_login.rs: Windows protocol refers directly to Core")
        if path == SRC / "windows_login.rs" and re.search(
            r"\.\s*store\s*\.\s*read\s*\(", masked_rust_source(path.read_text())
        ):
            errors.append("src/windows_login.rs: Windows protocol directly reads storage")
        if path == SRC / "windows_login.rs" and re.search(
            r"\.\s*store\s*\.\s*write\s*\(", masked_rust_source(path.read_text())
        ):
            errors.append("src/windows_login.rs: Windows protocol directly writes storage")
        if path == SRC / "pam.rs" and (
            refs & (STORAGE | {"core"})
            or re.search(r"\bCore\b|\bTx\b|\.\s*store\b", masked_rust_source(path.read_text()))
        ):
            errors.append("src/pam.rs: temporary-access protocol refers directly to Core or storage")
        if path == SRC / "provider.rs" and (
            refs & (STORAGE | {"core"})
            or re.search(r"\bCore\b|\bTx\b|\.\s*store\b", masked_rust_source(path.read_text()))
        ):
            errors.append("src/provider.rs: provider policy refers directly to Core or storage")
        if path == SRC / "signin.rs" and (
            refs & (STORAGE | {"core"})
            or re.search(r"\bCore\b|\bTx\b|\.\s*store\b", masked_rust_source(path.read_text()))
        ):
            errors.append("src/signin.rs: browser sign-in protocol refers directly to Core or storage")
        if path == SRC / "outpost.rs" and (
            refs & (STORAGE | {"core"})
            or re.search(r"\bCore\b|\bTx\b|\.\s*store\b", masked_rust_source(path.read_text()))
        ):
            errors.append("src/outpost.rs: proxy SSO protocol refers directly to Core or storage")
        if path == SRC / "directory.rs" and (
            refs & (STORAGE | {"core"})
            or re.search(r"\bCore\b|\bTx\b|\.\s*store\b", masked_rust_source(path.read_text()))
        ):
            errors.append("src/directory.rs: LDAP import adapter refers directly to Core or storage")
        if path == SRC / "cloud_directory.rs" and (
            re.search(
                r"\b(?:SyncRun|RETRY_LIMIT|RETRY_WINDOW|budget_exhausted|window_open|ensure_budget|record_failure|reset_budget)\b",
                masked_rust_source(path.read_text()),
            )
            or '"cloud_directory_runs"' in path.read_text()
        ):
            errors.append("src/cloud_directory.rs: retry-budget storage belongs in assembly")
        if path == SRC / "cloud_directory.rs":
            if rust_function_body(masked_rust_source(path.read_text()), "cloud_snapshot_actor") is not None:
                errors.append("src/cloud_directory.rs: snapshot authority/revision check belongs in assembly")
            reconcile = rust_function_body(masked_rust_source(path.read_text()), "cloud_reconcile")
            if (
                reconcile is None
                or not re.search(r"\bcloud_reconcile_pending\s*\(", reconcile)
                or re.search(r"\.\s*store\s*\.\s*read\s*\(", reconcile)
            ):
                errors.append("src/cloud_directory.rs: pending reconcile-plan read belongs in assembly")
            plan_get = rust_function_body(masked_rust_source(path.read_text()), "cloud_plan_get")
            if plan_get is None or re.search(r"\.\s*store\b|\bTx\b|\btx\b", plan_get):
                errors.append("src/cloud_directory.rs: reviewed-plan read belongs in assembly")
            plan_internal = rust_function_body(masked_rust_source(path.read_text()), "cloud_plan_internal")
            if (
                plan_internal is None
                or not re.search(r"\bcloud_snapshot_actor_revision\s*\(", plan_internal)
                or re.search(r"let\s*\(\s*actor\s*,\s*revision\s*\)\s*=\s*self\s*\.\s*store\s*\.\s*read", plan_internal)
            ):
                errors.append("src/cloud_directory.rs: snapshot actor/revision read belongs in assembly")
            snapshot_prepare = rust_function_body(
                masked_rust_source((SRC / "assembly/cloud_directory_snapshot.rs").read_text()),
                "cloud_snapshot_prepare",
            )
            if (
                plan_internal is None
                or not re.search(r"\bcloud_snapshot_prepare\s*\(", plan_internal)
                or re.search(r"let\s*\(\s*prior\s*,\s*mut\s+draft\s*,\s*restarted\s*,\s*authority_digest\s*\)\s*=\s*self\s*\.\s*store\s*\.\s*read", plan_internal)
                or snapshot_prepare is None
                or not re.search(r"\.\s*store\s*\.\s*read\s*\(", snapshot_prepare)
            ):
                errors.append("src/cloud_directory.rs: planning draft authorization/read belongs in assembly")
            plan_materialize = rust_function_body(
                masked_rust_source((SRC / "assembly/cloud_directory_snapshot.rs").read_text()),
                "cloud_plan_materialize",
            )
            if (
                plan_internal is None
                or not re.search(r"\bcloud_plan_materialize\s*\(", plan_internal)
                or re.search(r"\.\s*store\s*\.\s*read\s*\(", plan_internal)
                or plan_materialize is None
                or not re.search(r"\.\s*store\s*\.\s*read\s*\(", plan_materialize)
                or not re.search(r"\bcloud_snapshot_actor\s*\([\s\S]*\bmaterialize_completed_draft\s*\(", plan_materialize)
            ):
                errors.append("src/cloud_directory.rs: completed planning snapshot read belongs in assembly")
            snapshot_stage = rust_function_body(
                masked_rust_source((SRC / "assembly/cloud_directory_snapshot.rs").read_text()),
                "cloud_snapshot_stage",
            )
            if (
                plan_internal is None
                or not re.search(r"\bcloud_snapshot_stage\s*\(", plan_internal)
                or re.search(r"\breturn\s+self\s*\.\s*store\s*\.\s*write\s*\(", plan_internal)
                or snapshot_stage is None
                or not re.search(r"\.\s*store\s*\.\s*write\s*\(", snapshot_stage)
            ):
                errors.append("src/cloud_directory.rs: planning snapshot staging write belongs in assembly")
            plan_preview = rust_function_body(
                masked_rust_source((SRC / "assembly/cloud_directory_plan.rs").read_text()),
                "cloud_plan_preview",
            )
            if (
                plan_internal is None
                or not re.search(r"\bcloud_plan_preview\s*\(", plan_internal)
                or re.search(r"\.\s*store\s*\.\s*preview\s*\(", plan_internal)
                or plan_preview is None
                or not re.search(r"\.\s*store\s*\.\s*preview\s*\(", plan_preview)
                or not re.search(r"\bcloud_snapshot_actor\s*\([\s\S]*\bremoval_impact\s*\([\s\S]*\breconcile\s*\(", plan_preview)
            ):
                errors.append("src/cloud_directory.rs: reviewed-plan preview belongs in assembly")
            plan_commit = rust_function_body(
                masked_rust_source((SRC / "assembly/cloud_directory_plan.rs").read_text()),
                "cloud_plan_commit",
            )
            if (
                plan_internal is None
                or not re.search(
                    r"\bcloud_budget_reset\s*\([\s\S]*\bcloud_plan_preview\s*\([\s\S]*\bcloud_plan_commit\s*\(",
                    plan_internal,
                )
                or re.search(r"\.\s*store\s*\.\s*write\s*\(", plan_internal)
                or plan_commit is None
                or not re.search(r"\.\s*store\s*\.\s*write\s*\(", plan_commit)
                or not re.search(r"\.count\s*\(\s*\)\s*>=\s*16", plan_commit)
                or not re.search(
                    r"\bcloud_snapshot_actor\s*\([\s\S]*\btx\.get::<u64>\s*\([\s\S]*"
                    r"\btx\.get::<CloudSnapshotDraft>\s*\([\s\S]*\btx\.list::<Plan>\s*\([\s\S]*"
                    r"\bReviewBinding::new\s*\([\s\S]*\.\s*validate\s*\([\s\S]*"
                    r"\brequire_backup_safe_record\s*\([\s\S]*\btx\.put\s*\([\s\S]*"
                    r"\btx\.delete\s*\([\s\S]*\baudit_scoped\s*\(",
                    plan_commit,
                )
            ):
                errors.append("src/cloud_directory.rs: final reviewed-plan write belongs in assembly")
            apply_confirmed = rust_function_body(masked_rust_source(path.read_text()), "cloud_apply_confirmed")
            if (
                apply_confirmed is None
                or not re.search(r"\bcloud_applied_plan_sync_authorized\s*\(", apply_confirmed)
                or re.search(r"if\s+initially_applied\s*\{\s*self\s*\.\s*store\s*\.\s*read", apply_confirmed)
            ):
                errors.append("src/cloud_directory.rs: applied-plan sync read belongs in assembly")
            if (
                rust_function_body(masked_rust_source(path.read_text()), "cloud_apply_actor") is not None
                or apply_confirmed is None
                or not re.search(r"\bcloud_apply_snapshot_prepare\s*\(", apply_confirmed)
                or re.search(r"let\s*\(\s*prior\s*,\s*mut\s+apply\s*,\s*restarted\s*\)\s*=\s*self\s*\.\s*store\s*\.\s*read", apply_confirmed)
            ):
                errors.append("src/cloud_directory.rs: apply snapshot authorization/read belongs in assembly")
            apply_materialize = rust_function_body(
                masked_rust_source((SRC / "assembly/cloud_directory_plan.rs").read_text()),
                "cloud_apply_materialize",
            )
            if (
                apply_confirmed is None
                or not re.search(r"\bcloud_apply_materialize\s*\([\s\S]*\bif\s+entries\s*!=\s*plan\.entries", apply_confirmed)
                or re.search(r"\.\s*store\s*\.\s*read\s*\(", apply_confirmed)
                or apply_materialize is None
                or not re.search(r"\.\s*store\s*\.\s*read\s*\(", apply_materialize)
                or not re.search(r"\bcloud_apply_actor\s*\([\s\S]*\bmaterialize_completed_draft\s*\(", apply_materialize)
            ):
                errors.append("src/cloud_directory.rs: completed apply snapshot read belongs in assembly")
            apply_stage = rust_function_body(
                masked_rust_source((SRC / "assembly/cloud_directory_plan.rs").read_text()),
                "cloud_apply_snapshot_stage",
            )
            if (
                apply_confirmed is None
                or not re.search(r"\bcloud_apply_snapshot_stage\s*\(", apply_confirmed)
                or re.search(r"\.\s*store\s*\.\s*write\s*\(", apply_confirmed)
                or apply_stage is None
                or not re.search(r"\.\s*store\s*\.\s*write\s*\(", apply_stage)
            ):
                errors.append("src/cloud_directory.rs: apply snapshot staging write belongs in assembly")
            apply_commit = rust_function_body(
                masked_rust_source((SRC / "assembly/cloud_directory_plan.rs").read_text()),
                "cloud_apply_commit",
            )
            if (
                apply_confirmed is None
                or not re.search(
                    r"\bcloud_budget_reset\s*\([\s\S]*\bcloud_apply_materialize\s*\([\s\S]*"
                    r"\bif\s+entries\s*!=\s*plan\.entries[\s\S]*\bcloud_apply_commit\s*\(",
                    apply_confirmed,
                )
                or re.search(r"\.\s*mutation\s*\(", apply_confirmed)
                or apply_commit is None
                or not re.search(r"\.\s*mutation\s*\(", apply_commit)
                or not re.search(
                    r"\bmanagement\s*\([\s\S]*\btx\s*\.\s*get::<Plan>\s*\([\s\S]*"
                    r"\bif\s+plan\.review\s*!=\s*observed_review[\s\S]*"
                    r"\bif\s+plan\.applied\s*\{[\s\S]*\bcloud_apply_actor\s*\([\s\S]*"
                    r"\btx\s*\.\s*get::<CloudApplyDraft>\s*\([\s\S]*\bremoval_impact\s*\([\s\S]*"
                    r"\bApplyGate\s*\{[\s\S]*\.\s*validate\s*\([\s\S]*"
                    r"\breconcile\s*\([\s\S]*\btx\.put\s*\([\s\S]*"
                    r"\btx\.delete\s*\([\s\S]*\baudit_scoped\s*\(",
                    apply_commit,
                )
            ):
                errors.append("src/cloud_directory.rs: final apply mutation belongs in assembly")
            if rust_function_body(masked_rust_source(path.read_text()), "cloud_directories") is not None:
                errors.append("src/cloud_directory.rs: scoped catalog read belongs in assembly")
            cleanup = rust_function_body(masked_rust_source(path.read_text()), "cleanup")
            if cleanup is None or re.search(r"\bmaintenance_page\s*::\s*<\s*Plan\s*>", cleanup):
                errors.append("src/cloud_directory.rs: reviewed-plan retention belongs in assembly")
            if (
                cleanup is None
                or not re.search(r"\bcloud_snapshot_cleanup\s*\(", cleanup)
                or re.search(r"\bmaintenance_page\s*::\s*<\s*Cloud(?:Snapshot|Apply)Draft\s*>", cleanup)
            ):
                errors.append("src/cloud_directory.rs: snapshot retention belongs in assembly")
        if path == SRC / "cloud_operations.rs":
            operations = rust_function_body(masked_rust_source(path.read_text()), "cloud_operations")
            catalog_source = (SRC / "assembly/cloud_directory_catalog.rs").read_text()
            initial_auth = rust_function_body(
                masked_rust_source(catalog_source), "cloud_operation_authorize_read"
            )
            initial_auth_raw = rust_function_body(catalog_source, "cloud_operation_authorize_read")
            initial_call = (
                re.search(r"\bself\.cloud_operation_authorize_read\s*\(\s*token\s*,\s*&scope\s*\)\s*\?\s*;", operations)
                if operations is not None
                else None
            )
            if (
                operations is None
                or initial_call is None
                or not (0 <= operations.find("resource(kind, id)") < initial_call.start() < operations.find("match kind"))
                or re.search(r"\.\s*store\s*\.\s*read\s*\(", operations[:operations.find("match kind")])
                or initial_auth is None
                or not re.search(r"\.\s*store\s*\.\s*read\s*\(", initial_auth)
                or initial_auth_raw is None
                or not re.search(
                    r'self\.management\s*\(\s*tx\s*,\s*token\s*,\s*"directory\.read"\s*,\s*scope\s*\)\s*\?[\s\S]*Ok\s*\(\s*\(\s*\)\s*\)',
                    initial_auth_raw,
                )
            ):
                errors.append("src/cloud_operations.rs: initial read authorization belongs in assembly")
            missing_groups = rust_function_body(
                masked_rust_source(catalog_source),
                "cloud_operation_missing_groups",
            )
            if (
                operations is None
                or not re.search(r"\bcloud_operation_missing_groups\s*\(", operations)
                or re.search(r"\btx\s*\.\s*get\s*::<\s*Group\s*>", operations)
                or missing_groups is None
                or not re.search(r"\.\s*store\s*\.\s*read\s*\(", missing_groups)
                or not re.search(
                    r"\bmanagement\s*\([\s\S]*\btx\s*\.\s*get\s*::<\s*Group\s*>",
                    missing_groups,
                )
            ):
                errors.append("src/cloud_operations.rs: authorized local-group lookup belongs in assembly")
            can_sync = rust_function_body(
                masked_rust_source(catalog_source), "cloud_operation_can_sync"
            )
            can_sync_raw = rust_function_body(catalog_source, "cloud_operation_can_sync")
            can_sync_call = (
                re.search(
                    r"\blet\s+can_sync\s*=\s*self\.cloud_operation_can_sync\s*\(\s*token\s*,\s*&scope\s*\)\s*\?\s*;",
                    operations,
                )
                if operations is not None
                else None
            )
            if (
                operations is None
                or can_sync_call is None
                or not (
                    operations.find("cloud_operation_missing_groups")
                    < can_sync_call.start()
                    < operations.find("reconciliation_schedules")
                )
                or re.search(
                    r"\.\s*store\s*\.\s*read\s*\(",
                    operations[
                        operations.find("cloud_operation_missing_groups"):
                        operations.find("reconciliation_schedules")
                    ],
                )
                or re.search(r"\bprincipal\s*\(", operations)
                or can_sync is None
                or not re.search(r"\.\s*store\s*\.\s*read\s*\(", can_sync)
                or not re.search(
                    r"\bprincipal\s*\(\s*tx\s*,\s*token\s*\)\s*\?\s*\.\s*allows\s*\(",
                    can_sync,
                )
                or re.search(r"\bmanagement\s*\(", can_sync)
                or can_sync_raw is None
                or not re.search(r'\.allows\s*\(\s*"directory\.sync"\s*,\s*scope\s*\)', can_sync_raw)
            ):
                errors.append("src/cloud_operations.rs: sync authority read belongs in assembly")
            controller_check = rust_function_body(
                masked_rust_source(catalog_source), "cloud_operation_controller_check"
            )
            controller_check_raw = rust_function_body(catalog_source, "cloud_operation_controller_check")
            controller_call = (
                re.search(
                    r"\blet\s+last_check\s*=\s*self\.cloud_operation_controller_check\s*\(\s*token\s*,\s*&scope\s*\)\s*\?\s*;",
                    operations,
                )
                if operations is not None
                else None
            )
            if (
                operations is None
                or controller_call is None
                or not (
                    0 <= operations.find("let controller = if can_sync")
                    < operations.find("controller_fingerprint")
                    < controller_call.start()
                )
                or not re.search(r"\blast_check\s*\.\s*filter\s*\(", operations[controller_call.end():])
                or re.search(r"\bCloudControllerCheck\b", masked_rust_source(path.read_text()))
                or controller_check is None
                or not re.search(r"\.\s*store\s*\.\s*read\s*\(", controller_check)
                or controller_check_raw is None
                or not re.search(
                    r'self\.management\s*\(\s*tx\s*,\s*token\s*,\s*"directory\.sync"\s*,\s*scope\s*\)\s*\?[\s\S]*'
                    r'tx\.get\s*::<\s*CloudControllerCheck\s*>\s*\(\s*"cloud_controller_checks"\s*,\s*scope\s*\)',
                    controller_check_raw,
                )
            ):
                errors.append("src/cloud_operations.rs: scoped controller check read belongs in assembly")
            connection_check = rust_function_body(
                masked_rust_source(catalog_source), "cloud_operation_last_connection_check"
            )
            connection_check_raw = rust_function_body(
                catalog_source, "cloud_operation_last_connection_check"
            )
            connection_call = (
                re.search(
                    r"\blet\s+last_connection_check\s*=\s*self\.cloud_operation_last_connection_check\s*\(\s*token\s*,\s*&scope\s*\)\s*\?\s*;",
                    operations,
                )
                if operations is not None
                else None
            )
            if (
                operations is None
                or connection_call is None
                or not (
                    operations.find("reconciliation_jobs")
                    < connection_call.start()
                    < operations.find("Ok(json!")
                )
                or re.search(r"\btx\s*\.\s*get\s*::<\s*Value\s*>", operations)
                or connection_check is None
                or not re.search(r"\.\s*store\s*\.\s*read\s*\(", connection_check)
                or connection_check_raw is None
                or not re.search(
                    r'self\.management\s*\(\s*tx\s*,\s*token\s*,\s*"directory\.read"\s*,\s*scope\s*\)\s*\?[\s\S]*'
                    r'tx\.get\s*::<\s*Value\s*>\s*\(\s*"cloud_connection_checks"\s*,\s*scope\s*\)',
                    connection_check_raw,
                )
            ):
                errors.append("src/cloud_operations.rs: scoped connection check read belongs in assembly")
            read_auth_calls = (
                list(re.finditer(
                    r"\bself\.cloud_operation_authorize_read\s*\(\s*token\s*,\s*&scope\s*\)\s*\?\s*;",
                    operations,
                ))
                if operations is not None
                else []
            )
            if (
                connection_call is None
                or len(read_auth_calls) != 2
                or not (connection_call.end() <= read_auth_calls[1].start() < operations.find("Ok(json!"))
                or operations[connection_call.end():read_auth_calls[1].start()].strip()
                or operations[read_auth_calls[1].end():operations.find("Ok(json!")].strip()
            ):
                errors.append("src/cloud_operations.rs: final read authorization must follow connection check")
            probe = rust_function_body(masked_rust_source(path.read_text()), "cloud_test_connection")
            probe_auth = rust_function_body(
                masked_rust_source(catalog_source), "cloud_operation_authorize_probe"
            )
            probe_auth_raw = rust_function_body(catalog_source, "cloud_operation_authorize_probe")
            probe_call = (
                re.search(r"\bself\.cloud_operation_authorize_probe\s*\(\s*token\s*,\s*&scope\s*\)\s*\?\s*;", probe)
                if probe is not None
                else None
            )
            if (
                probe is None
                or probe_call is None
                or not (
                    0 <= probe.find("resource(kind, id)")
                    < probe_call.start()
                    < probe.find("let checked_at")
                    < probe.find("cloud_connection_probe")
                )
                or re.search(r"\.\s*store\s*\.\s*read\s*\(", probe[:probe.find("cloud_connection_probe")])
                or probe_auth is None
                or not re.search(r"\.\s*store\s*\.\s*read\s*\(", probe_auth)
                or probe_auth_raw is None
                or not re.search(
                    r'self\.management\s*\(\s*tx\s*,\s*token\s*,\s*"directory\.sync"\s*,\s*scope\s*\)\s*\?[\s\S]*Ok\s*\(\s*\(\s*\)\s*\)',
                    probe_auth_raw,
                )
            ):
                errors.append("src/cloud_operations.rs: pre-probe sync authorization belongs in assembly")
            probe_auth_calls = (
                list(re.finditer(
                    r"\bself\.cloud_operation_authorize_probe\s*\(\s*token\s*,\s*&scope\s*\)\s*\?\s*;",
                    probe,
                ))
                if probe is not None
                else []
            )
            probe_result = (
                re.search(r"\blet\s+result\s*=\s*self\.cloud_connection_probe\s*\(\s*kind\s*,\s*id\s*\)\s*;", probe)
                if probe is not None
                else None
            )
            if (
                probe is None
                or len(probe_auth_calls) != 2
                or probe_result is None
                or not (probe_result.end() <= probe_auth_calls[1].start() < probe.find("Ok(match result"))
                or probe[probe_result.end():probe_auth_calls[1].start()].strip()
                or probe[probe_auth_calls[1].end():probe.find("Ok(match result")].strip()
                or re.search(r"\.\s*store\s*\.\s*read\s*\(", probe)
            ):
                errors.append("src/cloud_operations.rs: post-probe sync recheck belongs in assembly")
            verify = rust_function_body(masked_rust_source(path.read_text()), "cloud_verify_credential")
            preflight = rust_function_body(
                masked_rust_source(catalog_source), "cloud_operation_credential_preflight"
            )
            preflight_raw = rust_function_body(catalog_source, "cloud_operation_credential_preflight")
            preflight_call = (
                re.search(
                    r"\bself\.cloud_operation_credential_preflight\s*\(\s*token\s*,\s*&scope\s*\)\s*\?",
                    verify,
                )
                if verify is not None else None
            )
            if (
                verify is None
                or preflight_call is None
                or not (0 <= verify.find("resource(kind, id)") < preflight_call.start()
                        < verify.find("match kind") < verify.find("cloud_connection_probe")
                        < verify.find("cloud_operation_record_credential_check"))
                or re.search(r"\.\s*store\s*\.\s*read\s*\(", masked_rust_source(path.read_text()))
                or re.search(r"\b(?:management|replay_receipt)\s*\(", verify)
                or preflight is None
                or not re.search(r"\.\s*store\s*\.\s*read\s*\(", preflight)
                or not (0 <= preflight.find("self.management") < preflight.find("replay_receipt")
                        < preflight.find("actor.agent") < preflight.find("tx.get::<u64>"))
                or preflight_raw is None
                or not re.search(r'self\.management\s*\(\s*tx\s*,\s*token\s*,\s*"directory\.sync"\s*,\s*scope\s*\)', preflight_raw)
                or "digest(&format!" not in preflight_raw
                or "actor.permissions" not in preflight_raw
                or "context.fingerprint" not in preflight_raw
                or not re.search(r'tx\.get\s*::<\s*u64\s*>\s*\(\s*"meta"\s*,\s*"revision"\s*\)', preflight_raw)
            ):
                errors.append("src/cloud_operations.rs: credential replay and revision preflight belongs in assembly")
            record = rust_function_body(
                masked_rust_source(catalog_source), "cloud_operation_record_credential_check"
            )
            record_raw = rust_function_body(catalog_source, "cloud_operation_record_credential_check")
            record_call = (
                re.search(
                    r"\bself\.cloud_operation_record_credential_check\s*\(\s*token\s*,\s*&scope\s*,\s*outcome\s*\)",
                    verify,
                )
                if verify is not None else None
            )
            if (
                verify is None
                or record_call is None
                or not (0 <= verify.find("let outcome = match self.cloud_connection_probe")
                        < record_call.start())
                or re.search(r"\.\s*(?:mutation|store\s*\.\s*(?:read|write))\s*\(", masked_rust_source(path.read_text()))
                or record is None
                or not (0 <= record.find("self.mutation") < record.find("self.management")
                        < record.find("tx.put") < record.find("audit") < record.find("Ok(outcome)"))
                or record_raw is None
                or not re.search(r'self\.management\s*\(\s*tx\s*,\s*token\s*,\s*"directory\.sync"\s*,\s*scope\s*\)', record_raw)
                or not re.search(r'tx\.put\s*\(\s*"cloud_connection_checks"\s*,\s*scope\s*,\s*&outcome\s*\)', record_raw)
                or not re.search(r'audit\s*\(\s*tx\s*,\s*&actor\.id\s*,\s*"cloud_directory\.credential_verify"\s*,\s*scope\s*\)', record_raw)
            ):
                errors.append("src/cloud_operations.rs: credential verification write belongs in assembly")
        if path == SRC / "source.rs":
            source_protocol = masked_rust_source(path.read_text())
            source_catalog = (SRC / "assembly/source_catalog.rs").read_text()
            if rust_function_body(source_protocol, "source_list") is not None:
                errors.append("src/source.rs: authorized source catalog read belongs in assembly")
            source_put = rust_function_body(
                masked_rust_source(source_catalog), "source_put"
            )
            source_put_raw = rust_function_body(source_catalog, "source_put")
            if (
                rust_function_body(source_protocol, "source_put") is not None
                or re.search(r"\bself\.mutation\s*\(", source_protocol)
                or source_put is None
                or not (0 <= source_put.find("Zeroizing::new")
                        < source_put.find("self.mutation")
                        < source_put.find("self.principal")
                        < source_put.find("management::write_source")
                        < source_put.find("Ok(json!(input.source))"))
                or source_put_raw is None
                or not re.search(r"\bpub\s+fn\s+source_put\s*\(", source_catalog)
                or "SourceWrite::Direct" not in source_put_raw
                or "secret.as_deref().map(String::as_str)" not in source_put_raw
                or "&input.source" not in source_put_raw
            ):
                errors.append("src/source.rs: source configuration mutation belongs in assembly")
            source_start = rust_function_body(
                masked_rust_source(source_catalog), "source_start"
            )
            if (
                rust_function_body(source_protocol, "source_start") is not None
                or source_start is None
                or not re.search(r"\.\s*store\s*\.\s*write\s*\(", source_start)
                or not re.search(
                    r"self\.source_start_in\s*\(\s*tx\s*,\s*id\s*,\s*&input\s*,\s*token\s*,\s*None\s*\)",
                    source_start,
                )
                or not re.search(r"\.map\s*\(\s*\|started\|\s*started\.body\s*\)", source_start)
                or rust_function_body(source_protocol, "source_start_in") is None
                or not re.search(r"\bpub\(crate\)\s+fn\s+source_start_in\s*\(", source_protocol)
            ):
                errors.append("src/source.rs: source start writer belongs in assembly")
            source_unlink = rust_function_body(
                masked_rust_source(source_catalog), "source_unlink"
            )
            source_unlink_raw = rust_function_body(source_catalog, "source_unlink")
            if (
                rust_function_body(source_protocol, "source_unlink") is not None
                or source_unlink is None
                or not re.search(r"\.\s*store\s*\.\s*write\s*\(", source_unlink)
                or not (0 <= source_unlink.find("self.session")
                        < source_unlink.find("management::unlink_source"))
                or source_unlink_raw is None
                or not re.search(r"\bpub\s+fn\s+source_unlink\s*\(", source_catalog)
                or not re.search(r"self\.session\s*\(\s*tx\s*,\s*token\s*\)\s*\?", source_unlink_raw)
                or not re.search(
                    r"management::unlink_source\s*\(\s*tx\s*,\s*&user\s*,\s*&session\s*,\s*link_id\s*\)",
                    source_unlink_raw,
                )
            ):
                errors.append("src/source.rs: session-bound source unlink writer belongs in assembly")
            source_links = rust_function_body(
                masked_rust_source(source_catalog), "source_links"
            )
            source_links_raw = rust_function_body(source_catalog, "source_links")
            if (
                rust_function_body(source_protocol, "source_links") is not None
                or re.search(r"\bself\s*\.\s*store\s*\.\s*read\s*\(", source_protocol)
                or source_links is None
                or not re.search(r"\.\s*store\s*\.\s*read\s*\(", source_links)
                or not (0 <= source_links.find("self.session")
                        < source_links.find("source::links_of"))
                or source_links_raw is None
                or not re.search(r"\bpub\s+fn\s+source_links\s*\(", source_catalog)
                or not re.search(r"self\.session\s*\(\s*tx\s*,\s*token\s*\)\s*\?", source_links_raw)
                or not re.search(r"source::links_of\s*\(\s*tx\s*,\s*&user\.id\s*\)\s*\?", source_links_raw)
            ):
                errors.append("src/source.rs: session-scoped source link read belongs in assembly")
            source_stage_assembly = (SRC / "assembly/source_stage.rs").read_text()
            source_stage_cancel = rust_function_body(
                masked_rust_source(source_stage_assembly), "source_stage_cancel"
            )
            source_stage_cancel_raw = rust_function_body(
                source_stage_assembly, "source_stage_cancel"
            )
            if (
                rust_function_body(source_protocol, "source_stage_cancel") is not None
                or not re.search(r"\bpub\(crate\)\s+fn\s+cancel_stage\s*\(", source_protocol)
                or source_stage_cancel is None
                or not re.search(r"\.\s*store\s*\.\s*write\s*\(", source_stage_cancel)
                or source_stage_cancel_raw is None
                or not re.search(r"\bpub\s+fn\s+source_stage_cancel\s*\(", source_stage_assembly)
                or not re.search(
                    r"self\.store\s*\.\s*write\s*\(\s*\|tx\|\s*self\.cancel_stage\s*\(\s*tx\s*,\s*stage_id\s*,\s*authorization_id\s*\)\s*\)",
                    source_stage_cancel_raw,
                )
                or not re.search(r"\bmod\s+source_stage\s*;", (SRC / "assembly.rs").read_text())
            ):
                errors.append("src/source.rs: one-use source stage cancellation writer belongs in assembly")
            source_stage_resume = rust_function_body(
                masked_rust_source(source_stage_assembly), "source_stage_resume"
            )
            source_stage_resume_raw = rust_function_body(
                source_stage_assembly, "source_stage_resume"
            )
            if (
                rust_function_body(source_protocol, "source_stage_resume") is not None
                or not re.search(r"\bpub\(crate\)\s+fn\s+resume_stage\s*\(", source_protocol)
                or source_stage_resume is None
                or not re.search(r"\.\s*store\s*\.\s*write\s*\(", source_stage_resume)
                or source_stage_resume_raw is None
                or not re.search(r"\bpub\s+fn\s+source_stage_resume\s*\(", source_stage_assembly)
                or not re.search(
                    r"self\.store\s*\.\s*write\s*\(\s*\|tx\|\s*self\.resume_stage\s*\(\s*tx\s*,\s*stage_id\s*,\s*authorization_id\s*,\s*otp\.as_deref\s*\(\s*\)\s*\)\s*\)\s*\?",
                    source_stage_resume_raw,
                )
            ):
                errors.append("src/source.rs: charged source stage resume writer belongs in assembly")
            source_finish_assembly = (SRC / "assembly/source_finish.rs").read_text()
            source_finish = rust_function_body(source_finish_assembly, "source_finish")
            source_finish_compact = re.sub(r"\s+", "", source_finish or "")
            if (
                rust_function_body(source_protocol, "source_finish") is not None
                or re.search(r"\bself\.store\s*\.\s*(?:read|write)\s*\(", source_protocol)
                or source_finish is None
                or not re.search(r"\bpub\s+fn\s+source_finish\s*\(", source_finish_assembly)
                or not (0 <= source_finish_compact.find("Zeroizing::new(input.credential)")
                        < source_finish_compact.find("self.store.write")
                        < source_finish_compact.find('tx.get::<String>("source_polls",&digest(&credential))')
                        < source_finish_compact.find('tx.get::<Login>("source_logins",&state)')
                        < source_finish_compact.find("p.expires_at>now()&&!p.failed&&p.attempts<5")
                        < source_finish_compact.find("pending.stage.is_some()||pending.workflow.is_some()")
                        < source_finish_compact.find("self.complete_source_login"))
                or not re.search(
                    r"self\.complete_source_login\(tx,&state,&mutpending,input\.approve,input\.otp\.as_deref\(\),None,?\)",
                    source_finish_compact,
                )
                or not source_finish_compact.endswith("})?")
                or not re.search(r"\bpub\(crate\)\s+fn\s+complete_source_login\s*\(", source_finish_assembly)
                or not re.search(r"\bmod\s+source_finish\s*;", (SRC / "assembly.rs").read_text())
            ):
                errors.append("src/source.rs: charged source finish writer belongs in assembly")
            source_finish_browser = rust_function_body(
                source_finish_assembly.replace("source_finish_browser<T>", "source_finish_browser"),
                "source_finish_browser",
            )
            source_finish_browser_compact = re.sub(r"\s+", "", source_finish_browser or "")
            if (
                re.search(r"\bfn\s+source_finish_browser\s*<", path.read_text())
                or source_finish_browser is None
                or not re.search(r"\bpub\(crate\)\s+fn\s+source_finish_browser\s*<T>\s*\(", source_finish_assembly)
                or not (0 <= source_finish_browser_compact.find("self.store.write")
                        < source_finish_browser_compact.find('tx.get::<String>("source_polls",&digest(credential))')
                        < source_finish_browser_compact.find('tx.get::<Login>("source_logins",&state)')
                        < source_finish_browser_compact.find("pending.stage.is_some()||pending.workflow.is_some()")
                        < source_finish_browser_compact.find("bind(tx,pending.target.as_ref())?")
                        < source_finish_browser_compact.find("letlinking=pending.target.is_some()")
                        < source_finish_browser_compact.find("self.complete_source_login")
                        < source_finish_browser_compact.find("Ok(body)=>deliver(tx,linking,&body).map(Ok)")
                        < source_finish_browser_compact.find("Err(error)=>Ok(Err(error))"))
                or not source_finish_browser_compact.endswith("})?")
            ):
                errors.append("src/source.rs: browser source finish transaction belongs in assembly")
            source_completion = rust_function_body(source_finish_assembly, "complete_source_login")
            source_completion_compact = re.sub(r"\s+", "", source_completion or "")
            clear_return = rust_function_body(source_finish_assembly, "clear_browser_return")
            if (
                rust_function_body(source_protocol, "complete_source_login") is not None
                or rust_function_body(source_protocol, "clear_browser_return") is not None
                or source_completion is None
                or not (0 <= source_completion_compact.find("pending.browser_return_confirmed")
                        < source_completion_compact.find("pending.workflow.is_some()")
                        < source_completion_compact.find("enabled(tx,&pending.source)")
                        < source_completion_compact.find("pending.fingerprint!=source.fingerprint()")
                        < source_completion_compact.find("pending.result.clone()")
                        < source_completion_compact.find("self.identity_user(tx,target)")
                        < source_completion_compact.find("if!approve")
                        < source_completion_compact.find("write_source_memberships")
                        < source_completion_compact.find("pending.attempts+=1")
                        < source_completion_compact.find('tx.put("source_logins",state,&pending)')
                        < source_completion_compact.find("returnOk(Err(Error::unauthorized()))")
                        < source_completion_compact.find('tx.put("users",&user.id,&user)')
                        < source_completion_compact.find("write_source_link")
                        < source_completion_compact.find("pin_retired:false")
                        < source_completion_compact.find('tx.put("authentication",&digest(challenge),&transaction)')
                        < source_completion_compact.find('tx.put("saml_source_sessions",&sid,upstream)')
                        < source_completion_compact.find('tx.put("sessions",&sid,&session)')
                        < source_completion_compact.find('tx.put("session_tokens",&session.token_hash,&sid)')
                        < source_completion_compact.find("clear_browser_return(tx,pending)")
                        < source_completion_compact.find('tx.delete("source_polls",&pending.poll_hash)')
                        < source_completion_compact.find('tx.delete("source_logins",state)')
                        < source_completion_compact.rfind("audit("))
                or clear_return is None
                or not re.search(r'tx\.delete\s*\(\s*"source_returns"\s*,\s*token\s*\)', clear_return)
                or not re.search(r"\bpub\(crate\)\s+use\s+source_finish::clear_browser_return\s*;", (SRC / "assembly.rs").read_text())
                or not re.search(r"\bassembly::clear_browser_return\b", path.read_text())
            ):
                errors.append("src/source.rs: source completion identity and one-use writes belong in assembly")
            source_callback = rust_function_body(source_protocol, "source_callback")
            source_callback_raw = rust_function_body(path.read_text(), "source_callback")
            callback_assembly = (SRC / "assembly/source_callback.rs").read_text()
            callback_claim = rust_function_body(
                masked_rust_source(callback_assembly), "source_callback_claim"
            )
            callback_claim_raw = rust_function_body(
                callback_assembly, "source_callback_claim"
            )
            callback_claim_compact = re.sub(r"\s+", "", callback_claim_raw or "")
            if (
                source_callback is None
                or re.search(r"\.\s*store\s*\.\s*write\s*\(", source_callback)
                or source_callback_raw is None
                or not re.search(
                    r"context::scope\s*\(\s*context\s*,\s*\|\|\s*\{\s*worker\.source_callback_claim\s*\(\s*source_id\s*,\s*request_state\s*,\s*presented\s*\)",
                    source_callback_raw,
                )
                or callback_claim is None
                or not (0 <= callback_claim_compact.find("self.store.write")
                        < callback_claim_compact.find('tx.get::<Source>("sources",id)')
                        < callback_claim_compact.find('tx.get::<Login>("source_logins",&digest(state))')
                        < callback_claim_compact.find("presented_source_retired(")
                        < callback_claim_compact.find("browser_binding_matches")
                        < callback_claim_compact.find("pending.claimed=true")
                        < callback_claim_compact.find("pending.failed=true")
                        < callback_claim_compact.find('tx.put("source_logins",&digest(state),&pending)')
                        < callback_claim_compact.find('audit(tx,"upstream","source.login_failed",id)')
                        < callback_claim_compact.find("CallbackClaim::Retired")
                        < callback_claim_compact.find('tx.get::<String>("source_secrets",id)'))
                or callback_claim_raw is None
                or not re.search(r"\bpub\(crate\)\s+fn\s+source_callback_claim\s*\(", callback_assembly)
                or not re.search(r'let\s+source\s*=\s*tx\.get::<Source>\s*\(\s*"sources"\s*,\s*id\s*\)\s*\?', callback_claim_raw)
                or not re.search(r'p\.expires_at\s*>\s*now\s*\(\s*\)\s*&&\s*!p\.claimed', callback_claim_raw)
                or not re.search(r"\bpub\(crate\)\s+fn\s+browser_binding_matches\s*\(", path.read_text())
                or not re.search(r"\bpub\(crate\)\s+fn\s+presented_source_retired\s*\(", path.read_text())
            ):
                errors.append("src/source.rs: one-use source callback claim belongs in assembly")
            callback_record = rust_function_body(
                masked_rust_source(callback_assembly), "source_callback_record"
            )
            callback_record_raw = rust_function_body(
                callback_assembly, "source_callback_record"
            )
            callback_record_compact = re.sub(r"\s+", "", callback_record or "")
            if (
                source_callback is None
                or re.search(r"\.\s*store\s*\.\s*write\s*\(", source_callback)
                or source_callback_raw is None
                or not re.search(
                    r"context::scope\s*\(\s*context\s*,\s*\|\|\s*worker\.source_callback_record\s*\(\s*state\s*,\s*id\s*,\s*result\s*\)",
                    source_callback_raw,
                )
                or rust_function_body(source_protocol, "source_callback_record") is not None
                or callback_record is None
                or not (0 <= callback_record_compact.find("self.store.write")
                        < callback_record_compact.find("tx.get::<Login>")
                        < callback_record_compact.find("tx.get::<Source>")
                        < callback_record_compact.find("matchresult")
                        < callback_record_compact.find("tx.put")
                        < callback_record_compact.find("audit")
                        < callback_record_compact.find("callback_body"))
                or callback_record_raw is None
                or not re.search(r"\bpub\(crate\)\s+fn\s+source_callback_record\s*\(", callback_assembly)
                or not re.search(r"current\.failed\s*=\s*true\s*;\s*current\.result\s*=\s*None", callback_record_raw)
                or not re.search(r"\bmod\s+source_callback\s*;", (SRC / "assembly.rs").read_text())
            ):
                errors.append("src/source.rs: post-verification callback write belongs in assembly")
        if path == SRC / "ldap_server.rs" and (
            refs & (STORAGE | {"core"})
            or re.search(r"\bCore\b|\bTx\b|\.\s*store\b", masked_rust_source(path.read_text()))
        ):
            errors.append("src/ldap_server.rs: LDAP protocol refers directly to Core or storage")
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
