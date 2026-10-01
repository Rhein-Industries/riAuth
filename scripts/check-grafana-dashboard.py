#!/usr/bin/env python3
"""Check the Grafana dashboard JSON against the metric emitters and alert rules.

Reads deploy/riauth-grafana.json, deploy/riauth-alerts.yml, src/api/observability.rs,
src/telemetry.rs, and src/store/maintenance.rs. Every panel expression must be an
alert expression with its threshold and any `{queue!="<queue>"}` exclusion removed.
Every PromQL metric name must be rendered by those emitters. This does not start
Grafana or Prometheus.
"""

import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
DASHBOARD = ROOT / "deploy" / "riauth-grafana.json"
ALERTS = ROOT / "deploy" / "riauth-alerts.yml"
OBSERVABILITY = ROOT / "src" / "api" / "observability.rs"
TELEMETRY = ROOT / "src" / "telemetry.rs"
MAINTENANCE = ROOT / "src" / "store" / "maintenance.rs"

FUNCTIONS = {"clamp_min", "histogram_quantile", "increase", "rate"}
IDENT = re.compile(r"[A-Za-z_:][A-Za-z0-9_:]*")
COMPARISON = re.compile(r"\s*(?:>=|<=|==|!=|>|<)\s*-?(?:\d+(?:\.\d+)?|\.\d+)\s*$")
# The only label matcher an alert may carry: it drops one known queue from a rule.
# The dashboard still draws that queue, so the panel query has no matcher.
QUEUE_EXCLUSION = re.compile(r'\{queue!="([a-z0-9_]+)"\}')
RANGE = re.compile(r"\[[0-9]+[smhdwy](?::[0-9]+[smhdwy])?\]")
LOOP = re.compile(
    r"for\s*\(\s*name\s*,\s*[A-Za-z0-9_]+\s*\)\s*in\s*\[(.*?)\]\s*\{(?=(.{0,500}))",
    re.S,
)
REQUIRED_PHRASES = (
    "docs/roadmap/o06-grafana-loopback.md",
    "src/api/observability.rs",
    "src/telemetry.rs",
    "created_at",
    "retry",
    "connector lag",
    "readiness",
)
OBSERVABILITY_METRICS = {
    "riauth_requests_total",
    "riauth_server_errors_total",
    "riauth_request_duration_seconds_bucket",
    "riauth_rate_limited_total",
    "riauth_worker_rejections_total",
    "riauth_queue_pending",
    "riauth_queue_failed",
    "riauth_queue_oldest_pending_seconds",
}
TELEMETRY_METRICS = {
    "riauth_signing_errors_total",
    "riauth_cleanup_errors_total",
    "riauth_storage_write_wait_seconds_bucket",
}


def die(message):
    raise SystemExit(message)


def normalize(expr):
    return re.sub(r"\s+", "", expr)


def tokens(expr):
    return IDENT.findall(RANGE.sub(" ", expr))


def metrics(expr):
    return [token for token in tokens(expr) if token not in FUNCTIONS]


def add_histogram(names, base):
    names.add(base)
    names.add(base + "_bucket")
    names.add(base + "_count")
    names.add(base + "_sum")


def observability_metrics(text):
    names = set(re.findall(r"riauth_[a-z0-9_]+", text))
    if not OBSERVABILITY_METRICS <= names:
        missing = sorted(OBSERVABILITY_METRICS - names)
        die("observability emitter is missing " + ", ".join(missing))
    return names


def telemetry_metrics(text):
    names = set(re.findall(r"riauth_[a-z0-9_]+", text))
    loops = list(LOOP.finditer(text))
    if len(loops) < 4:
        die("telemetry render loops were not found")
    for match in loops:
        labels = re.findall(r'"([a-z0-9_]+)"', match.group(1))
        body = match.group(2)
        if not labels:
            die("telemetry render loop has no metric names")
        if "riauth_{name}_seconds" in body:
            for label in labels:
                add_histogram(names, f"riauth_{label}_seconds")
        elif "riauth_{name}_total" in body:
            names.update(f"riauth_{label}_total" for label in labels)
        elif "riauth_{name}_peak" in body:
            for label in labels:
                names.add(f"riauth_{label}")
                names.add(f"riauth_{label}_peak")
        else:
            die("unrecognized telemetry render loop")
    if not TELEMETRY_METRICS <= names:
        missing = sorted(TELEMETRY_METRICS - names)
        die("telemetry emitter is missing " + ", ".join(missing))
    return names


def queues(text):
    match = re.search(r"pub const QUEUES:\s*\[&str;\s*\d+\]\s*=\s*\[(.*?)\];", text, re.S)
    if not match:
        die("maintenance QUEUES constant was not found")
    found = re.findall(r'"([a-z0-9_]+)"', match.group(1))
    if not found:
        die("maintenance QUEUES constant is empty")
    return found


def alert_expressions(text):
    found = []
    for line in text.splitlines():
        stripped = line.strip()
        if not stripped.startswith("expr:"):
            continue
        value = stripped[len("expr:") :].strip()
        if len(value) >= 2 and value[0] == value[-1] and value[0] in "'\"":
            value = value[1:-1]
        if not value:
            die("alert file has a blank expr")
        found.append(value)
    if not found:
        die("alert file has no expr")
    return found


def strip_queue_exclusions(part, queue_names):
    for name in QUEUE_EXCLUSION.findall(part):
        if name not in queue_names:
            die(f"alert excludes an unknown queue: {name}")
    return QUEUE_EXCLUSION.sub("", part)


def product_queries(expressions, queue_names):
    queries = []
    for expr in expressions:
        for part in re.split(r"\s+and\s+", expr.strip()):
            part = strip_queue_exclusions(part, queue_names)
            core = COMPARISON.sub("", part).strip()
            if core == part.strip() and re.search(r"[<>]=?|==|!=", part):
                die(f"alert comparison was not removed: {part}")
            names = metrics(core)
            if names and all(name.startswith("riauth_") for name in names):
                queries.append(core)
    if len(queries) != len(set(map(normalize, queries))):
        die("alert rules repeat a product query")
    return queries


def expression_problems(expr, allowed, emitted):
    problems = []
    if normalize(expr) not in allowed:
        problems.append(f"expression is outside the alert rules: {expr}")
    for token in tokens(expr):
        if token in FUNCTIONS:
            continue
        if token == "up" or token.startswith("ready"):
            problems.append(f"readiness series in {expr}")
            continue
        if "connector" in token or "key_problem" in token:
            problems.append(f"invented series {token}")
            continue
        if not token.startswith("riauth_"):
            problems.append(f"unknown PromQL identifier {token}")
            continue
        if token not in emitted:
            problems.append(f"metric is not emitted: {token}")
    return problems


def strings(value):
    if isinstance(value, str):
        yield value
    elif isinstance(value, dict):
        for item in value.values():
            yield from strings(item)
    elif isinstance(value, list):
        for item in value:
            yield from strings(item)


def embedded_alerts(value, path=""):
    if isinstance(value, dict):
        if "alert" in value:
            die(f"dashboard embeds a Grafana alert at {path or '/'}")
        for key, item in value.items():
            embedded_alerts(item, f"{path}.{key}")
    elif isinstance(value, list):
        for index, item in enumerate(value):
            embedded_alerts(item, f"{path}[{index}]")


def panel_expressions(panel):
    found = []

    def walk(value):
        if isinstance(value, dict):
            if "expr" in value:
                if not isinstance(value["expr"], str) or not value["expr"].strip():
                    die(f"panel {panel.get('id')} has an empty expr")
                found.append(value["expr"].strip())
            for item in value.values():
                walk(item)
        elif isinstance(value, list):
            for item in value:
                walk(item)

    walk(panel)
    return found


def error_rate_scale_problems(document):
    panel = next(
        (
            item
            for item in document.get("panels") or []
            if isinstance(item, dict) and item.get("title") == "Server error rate"
        ),
        None,
    )
    if panel is None:
        return ["server error rate panel is missing"]
    defaults = (panel.get("fieldConfig") or {}).get("defaults") or {}
    problems = []
    if defaults.get("min") != 0 or defaults.get("max") != 1:
        problems.append("server error rate scale must be min 0 and max 1")
    if defaults.get("unit") != "percentunit":
        problems.append("server error rate unit must be percentunit")
    steps = (defaults.get("thresholds") or {}).get("steps") or []
    red = [step.get("value") for step in steps if isinstance(step, dict) and step.get("color") == "red"]
    if red != [0.05]:
        problems.append("server error rate threshold must stay 0.05")
    return problems


def check_dashboard(document, allowed, emitted, queue_names):
    problems = []
    if document.get("schemaVersion") != 39:
        problems.append("schemaVersion must be 39")
    if document.get("title") != "riAuth alert metrics":
        problems.append("unexpected dashboard title")
    if document.get("uid") != "riauth-alert-metrics":
        problems.append("unexpected dashboard uid")
    variables = document.get("templating", {}).get("list", [])
    if (
        len(variables) != 1
        or variables[0].get("type") != "datasource"
        or variables[0].get("query") != "prometheus"
        or variables[0].get("name") != "datasource"
    ):
        problems.append("dashboard must have one Prometheus datasource variable")
    panels = document.get("panels")
    if not isinstance(panels, list) or not 1 <= len(panels) <= 12:
        problems.append("dashboard panel count must stay between 1 and 12")
        return problems
    embedded_alerts(document)
    seen = []
    ids = set()
    for panel in panels:
        if not isinstance(panel, dict):
            problems.append("panel is not an object")
            continue
        panel_id = panel.get("id")
        if panel_id in ids:
            problems.append(f"duplicate panel id {panel_id}")
        ids.add(panel_id)
        kind = panel.get("type")
        expressions = panel_expressions(panel)
        if kind == "text":
            if expressions:
                problems.append("text panel contains PromQL")
            continue
        if kind != "timeseries":
            problems.append(f"unexpected panel type {kind}")
            continue
        if len(expressions) != 1:
            problems.append(f"panel {panel_id} must have one expression")
            continue
        datasource = panel.get("datasource", {})
        target = panel.get("targets", [{}])[0]
        target_datasource = target.get("datasource", {})
        if datasource.get("uid") != "${datasource}" or target_datasource.get("uid") != "${datasource}":
            problems.append(f"panel {panel_id} datasource is not the dashboard variable")
        if datasource.get("type") != "prometheus" or target_datasource.get("type") != "prometheus":
            problems.append(f"panel {panel_id} datasource is not Prometheus")
        problems.extend(expression_problems(expressions[0], allowed, emitted))
        seen.append(normalize(expressions[0]))
    if len(seen) != len(set(seen)):
        problems.append("dashboard repeats an expression")
    if set(seen) != allowed:
        missing = sorted(allowed - set(seen))
        extra = sorted(set(seen) - allowed)
        if missing:
            problems.append("dashboard omits alert query " + "; ".join(missing))
        if extra:
            problems.append("dashboard adds query " + "; ".join(extra))
    prose = "\n".join(strings(document)).casefold()
    for phrase in REQUIRED_PHRASES:
        if phrase.casefold() not in prose:
            problems.append(f"dashboard text is missing {phrase}")
    if "has not been imported" in prose:
        problems.append("dashboard text still says it has not been imported")
    problems.extend(error_rate_scale_problems(document))
    for name in queue_names:
        if name not in prose:
            problems.append(f"dashboard text omits queue {name}")
    for expr in seen:
        if "connector" in expr or "ready" in expr or expr.startswith("up"):
            problems.append(f"series outside the emitter in {expr}")
    return problems


def self_test(allowed, emitted):
    samples = [
        "up",
        "riauth_connector_lag_seconds",
        "sum(riauth_queue_pending)",
        "histogram_quantile(0.95, rate(riauth_request_duration_seconds_bucket[1m]))",
        "riauth_http_duration_seconds_bucket",
    ]
    for sample in samples:
        if not expression_problems(sample, allowed, emitted):
            die(f"self-test accepted {sample}")
    known = next(iter(allowed))
    # The known value is normalized; expression_problems normalizes again.
    if expression_problems(known, allowed, emitted):
        die("self-test rejected an alert query")


def main():
    emitted = observability_metrics(OBSERVABILITY.read_text())
    emitted.update(telemetry_metrics(TELEMETRY.read_text()))
    queue_names = queues(MAINTENANCE.read_text())
    queries = product_queries(alert_expressions(ALERTS.read_text()), queue_names)
    allowed = {normalize(query) for query in queries}
    for query in queries:
        problems = expression_problems(query, allowed, emitted)
        if problems:
            die("alert rule uses a metric the emitter does not render: " + "; ".join(problems))
    self_test(allowed, emitted)
    document = json.loads(DASHBOARD.read_text())
    if not isinstance(document, dict):
        die("dashboard JSON must be an object")
    if DASHBOARD.stat().st_size > 64 * 1024:
        die("dashboard JSON is larger than 64 KiB")
    problems = check_dashboard(document, allowed, emitted, queue_names)
    if problems:
        die("\n".join(problems))
    print(
        f"Grafana dashboard JSON matches {len(allowed)} emitted alert queries "
        f"across {len(queue_names)} queues"
    )


if __name__ == "__main__":
    try:
        main()
    except json.JSONDecodeError as error:
        sys.exit(f"dashboard JSON is invalid: {error}")
    except OSError as error:
        sys.exit(str(error))
