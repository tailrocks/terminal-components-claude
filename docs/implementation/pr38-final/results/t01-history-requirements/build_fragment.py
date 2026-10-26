import collections
import hashlib
import json
import pathlib
import re
import subprocess

ROOT = pathlib.Path("/Users/donbeave/Downloads/termrock-t01-history")
OUT = ROOT / "docs/implementation/pr38-final/results/t01-history-requirements"
INPUT = "542349e2f70ec9883d743594f906ac6c4e96ba5a"
OLD = "e08ac57d10ae54c87337eae0be08b1f5e8f4bda4"
PRESERVED = "1d6ee92adf6ccca38eeee64b1371cba1497e8fcd"
COMPONENT_PATH = pathlib.Path("/Users/donbeave/Downloads/termrock-t01-components/docs/implementation/pr38-final/results/t01-components-api/fragment.json")
FOUNDATION_PATH = pathlib.Path("/Users/donbeave/Downloads/termrock-t01-foundation/docs/implementation/pr38-final/results/t01-foundation-api/foundation-api.fragment.json")
ROUTE_PATH = pathlib.Path("/Users/donbeave/Downloads/termrock-t01-apps/docs/implementation/pr38-final/results/t01-app-routes/fragment.json")
ALLOW_OLD = "crates/showcase-ui/src/coverage/allowlist.txt"
ALLOW_NEW = "crates/showcase-ui/tests/coverage/allowlist.txt"
CATALOG_OLD = "crates/showcase-ui/src/coverage/catalog_data.rs"
T37_INPUT = "c817b99e9e43abbbf3519d56b2f5ee86924243d8"
T37_PRODUCT = "1d6ee92adf6ccca38eeee64b1371cba1497e8fcd"
T37_DEFERRED = "docs/implementation/pr38-final/results/t37-foundations/deferred-testing-surfaces.json"
T37_SOURCE_REMOVAL = "docs/implementation/pr38-final/results/t37-foundations/source-removal.json"
T37_FOUNDATION_RUNBOOK = "docs/implementation/pr38-final/results/t37-foundations/agent-runbook.json"
T37_VIEWPORT_ROWS = "docs/implementation/pr38-final/results/t37-components/approved-testing-api-retirement.jsonl"
T37_COMPONENT_SUMMARY = "docs/implementation/pr38-final/results/t37-components/summary.json"
T37_SELECTION_DECISION = "docs/implementation/pr38-final/results/t37-components/approved-selection-binding-retirement.json"
T01_REACH_DECISION_COMMIT = "b953c337ed2edadea8df2b20cc99115958ecf9f9"
T01_REACH_DECISION_PATH = "docs/implementation/pr38-final/results/t01-integration/shell-reach-retirement-decision.json"
T01_REACH_DECISION_SHA256 = "4ea12ecd0b6dcea45a7acf0bc79313356929e5322518236aaebc68674e9d720b"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def git_bytes(commit, path):
    return subprocess.run(["git", "show", commit + ":" + path], cwd=ROOT, check=True, stdout=subprocess.PIPE).stdout


def lines_at(commit, path):
    return git_bytes(commit, path).decode("utf-8").splitlines()


def path_sha(path):
    return sha(path.read_bytes())


def read_json(path):
    return json.loads(path.read_text())


def api_data():
    c = read_json(COMPONENT_PATH)
    f = read_json(FOUNDATION_PATH)
    r = read_json(ROUTE_PATH)
    if any(x.get("input_commit") != INPUT for x in (c, f, r)):
        raise ValueError("sibling source revision mismatch")
    return c, f, r


def api_indexes(component, foundation):
    by_symbol = collections.defaultdict(list)
    for item in component.get("items", []):
        if item.get("qualified_symbol"):
            entry = ("component", item)
            for symbol in set([item["qualified_symbol"]] + item.get("reachability_paths", []) + item.get("actual_definition_paths", [])):
                by_symbol[symbol].append(entry)
    for item in foundation.get("items", []):
        if item.get("current_qualified_symbol"):
            entry = ("foundation", item)
            for symbol in set([item["current_qualified_symbol"]] + item.get("public_qualified_paths", []) + item.get("reexports", [])):
                by_symbol[symbol].append(entry)
    components = {x["actual_definition_owner"]: x for x in component.get("components", []) if x.get("actual_definition_owner")}
    return by_symbol, components


def api_fields(entry):
    kind, item = entry
    if kind == "component":
        symbol = item.get("qualified_symbol")
        return {
            "symbol": symbol,
            "kind": item.get("kind"),
            "owner": item.get("definition_symbol") or symbol,
            "paths": sorted(set(item.get("reachability_paths", []) + item.get("actual_definition_paths", []) + [symbol])),
            "path": item.get("source"),
            "line": item.get("line_start"),
            "sha256": item.get("sha256"),
            "task": None,
        }
    symbol = item.get("current_qualified_symbol")
    return {
        "symbol": symbol,
        "kind": item.get("kind"),
        "owner": item.get("definition_owner") or symbol,
        "paths": sorted(set([symbol] + list(item.get("reexports", [])) + list(item.get("public_qualified_paths", [])))),
        "path": item.get("source_path"),
        "line": item.get("line"),
        "sha256": item.get("source_sha256"),
        "task": item.get("architecture_owner_task"),
    }


def task_for_symbol(symbol):
    s = symbol.replace("-", "_")
    for prefix, task in [
        ("termrock_core::", "T09"), ("termrock_layout::", "T10"),
        ("termrock_theme::", "T11"), ("termrock_render::", "T11"),
        ("termrock_runtime::", "T09"),
        ("termrock_controls::", "T12"), ("termrock_feedback::", "T12"),
        ("termrock_fields::", "T13"), ("termrock_text::", "T13"),
        ("termrock_collections::", "T14"), ("termrock_navigation::", "T14"),
        ("termrock_overlays::", "T15"), ("termrock_grid::", "T16"),
        ("termrock_editors::", "T17"), ("termrock_viewport::", "T17"),
        ("termrock_forms::", "T18"), ("termrock_shell::", "T19"),
        ("termrock_session::", "T20"), ("termrock_terminal::", "T20"),
        ("termrock::composition::", "T10"), ("termrock::blocks::", "T10"), ("showcase_", "T22"),
        ("holla_", "T27"), ("jackin_", "T28"), ("tablepro_", "T29"),
    ]:
        if s.startswith(prefix):
            return task
    return "T01"


def route_task(index):
    if 2 <= index <= 6:
        return "T23"
    if index in (13, 23, 24, 28, 29):
        return "T24"
    if index in (19, 32, 36, 38, 42):
        return "T25"
    if 45 <= index <= 53:
        return "T26"
    return "T22"


def block_rewrite(symbol):
    prefixes = [
        ("termrock::blocks::MasterDetail", "termrock::blocks::master_detail::MasterDetail"),
        ("termrock::blocks::SplitBlock", "termrock::blocks::split::SplitBlock"),
        ("termrock::blocks::MasterDetailAction", "termrock::blocks::master_detail::MasterDetailAction"),
        ("termrock::blocks::MasterDetailError", "termrock::blocks::master_detail::MasterDetailError"),
        ("termrock::blocks::MasterDetailPolicy", "termrock::blocks::master_detail::MasterDetailPolicy"),
        ("termrock::blocks::SplitBlockError", "termrock::blocks::split::SplitBlockError"),
        ("termrock::blocks::StatusTone", "termrock::blocks::status::StatusTone"),
        ("termrock::blocks::StatusSeg", "termrock::blocks::status::StatusSeg"),
        ("termrock::blocks::Tab", "termrock::blocks::tabs::Tab"),
        ("termrock::blocks::paint_status", "termrock::blocks::status::paint_status"),
        ("termrock::blocks::paint_tabs", "termrock::blocks::tabs::paint_tabs"),
        ("termrock::blocks::paint_row", "termrock::blocks::line::paint_row"),
        ("termrock::blocks::paint_section_header", "termrock::blocks::section::paint_section_header"),
    ]
    for old, new in prefixes:
        if symbol == old or symbol.startswith(old + "::"):
            return new + symbol[len(old):]
    return None


def retired_record(symbol):
    s = symbol.replace("-", "_")
    if s.startswith("termrock_shell::reach::"):
        return None
    if any(token in s for token in ("ViewportWorkProbe", "ViewportWorkSnapshot", "::work_probe")):
        return ("T37-PROBE-02", "docs/implementation/pr38-final/results/t37-components/approved-testing-api-retirement.jsonl", "viewport work counters and measurement hooks", "The feature-gated counters were test instrumentation and had no independent production callers.", "Keep viewport work bounds in T17 and workloads in T33; do not recreate counters before T38.", "T17/T33")
    if s.endswith("TextViewport::selection_binding_chords"):
        return ("T37-SELECTION-BINDING-01", "docs/implementation/pr38-final/results/t37-components/approved-selection-binding-retirement.json", "TextViewport selection binding accessor", "The removed accessor only exposed testing-feature binding data; production actions and SELECTION_BINDINGS remain.", "Prove routed selection through TextViewport input using retained production bindings after T38.", "T17")
    if s in {
        "termrock_runtime::runtime::RenderModel",
        "termrock_runtime::runtime::RenderSnapshot::bind_model",
        "termrock_runtime::runtime::ProjectedFrame",
        "termrock_runtime::runtime::ProjectedFrame::commit_inspected",
        "termrock_runtime::runtime::Runtime::draw_projection",
        "termrock_runtime::runtime::Runtime::draw_bound_projection",
    }:
        return ("T37-FOUNDATION-TESTING-01", "docs/implementation/pr38-final/results/t37-foundations/deferred-testing-surfaces.json", "runtime render model and projected frame test adapter", "The removed projection adapter was a test-only frame inspection surface.", "Keep drawing through production Runtime::draw_scene and verify real consumer output after T38.", "T09")
    testing_runtime_methods = {
        "hover", "state_of", "focus_visible", "top_layer", "is_open", "cursor",
        "resolved", "resolved_in", "last_invalidate", "records", "clear_records",
        "projection_registry", "projection_ring", "projection_cursor", "draw_buffer",
        "region_count", "registry", "advance_clock", "repaint_after", "capture_owner",
        "set_focus", "intent_probes", "style_cache_stats", "open_spec", "layer_area",
        "record_diagnostic",
    }
    if s.startswith("termrock_runtime::runtime::Runtime::") and s.rsplit("::", 1)[-1] in testing_runtime_methods:
        return ("T37-FOUNDATION-TESTING-01", "docs/implementation/pr38-final/results/t37-foundations/deferred-testing-surfaces.json", "runtime testing inspection methods", "T37 approved retirement of this feature-gated inspection or recording method; draw_scene was explicitly restored and is not in this set.", "Retain the related production runtime behavior and verify it through a public consumer after T38.", "T09")
    exact = {
        "termrock_core::intent::IntentQueue::probes": ("T37-FOUNDATION-TESTING-01", "core intent queue probes", "T09"),
        "termrock_runtime::runtime::Runtime::intent_probes": ("T37-FOUNDATION-TESTING-01", "runtime testing inspection", "T09"),
        "termrock_theme::Theme::fingerprint": ("T37-FOUNDATION-TESTING-01", "theme fingerprint", "T11"),
        "termrock_theme::resolve::StyleCache::stats": ("T37-FOUNDATION-TESTING-01", "style cache counters", "T11"),
        "termrock_runtime::runtime::Runtime::style_cache_stats": ("T37-FOUNDATION-TESTING-01", "style cache counters", "T09/T11"),
        "termrock_runtime::ui::Ui::style_cache_stats": ("T37-FOUNDATION-TESTING-01", "style cache counters", "T09/T11"),
        "termrock_runtime::ui::Ui::styled_parts": ("T37-FOUNDATION-TESTING-02", "styled query probe", "T09/T11"),
        "termrock_runtime::ui::Ui::styled_queries": ("T37-FOUNDATION-TESTING-02", "styled query probe", "T09/T11"),
        "termrock_runtime::ui::Ui::note_styled": ("T37-FOUNDATION-TESTING-02", "styled query probe", "T09/T11"),
        "termrock_runtime::ui::Cx::record": ("T37-FOUNDATION-TESTING-03", "Cx event recording", "T09"),
        "termrock_runtime::ui::cx::Cx::record": ("T37-FOUNDATION-TESTING-03", "Cx event recording", "T09"),
    }
    if s in exact:
        decision, group, task = exact[s]
        return (decision, "docs/implementation/pr38-final/results/t37-foundations/deferred-testing-surfaces.json", group, "A feature-gated inspection, counter, query, or recording helper was retired as test-only surface.", "Preserve the related product contract and design consumer cases from production paths after T38.", task)
    if s.startswith(("termrock_runtime::stub::", "termrock_runtime::runtime::stub::")):
        return ("T37-FOUNDATION-TESTING-03", "docs/implementation/pr38-final/results/t37-foundations/deferred-testing-surfaces.json", "runtime stub harness", "The stub constructors and dispatch helpers existed for test harnesses.", "Use production consumers and observable input, update, and draw results after T38.", "T09/T19/T20")
    if s.endswith("StyledQuery"):
        return ("T37-FOUNDATION-TESTING-02", "docs/implementation/pr38-final/results/t37-foundations/deferred-testing-surfaces.json", "styled query probe", "StyledQuery recorded test-only style projections.", "Check style behavior through production Theme and Ui rendering paths after T38.", "T09/T11")
    return None


T37_JSON = {}
T37_LINES = {}


def t37_json(path):
    if path not in T37_JSON:
        T37_JSON[path] = json.loads(git_bytes(INPUT, path))
    return T37_JSON[path]


def source_file_list(commit, crate_root):
    return subprocess.run(["git", "ls-tree", "-r", "--name-only", commit, "--", crate_root], cwd=ROOT, check=True, text=True, stdout=subprocess.PIPE).stdout.splitlines()


def source_lines_at(commit, path):
    key = (commit, path)
    if key not in T37_LINES:
        T37_LINES[key] = git_bytes(commit, path).decode("utf-8").splitlines(keepends=True)
    return T37_LINES[key]


def source_module_path(path, lines, target_line):
    relative = path.split("/src/", 1)[-1]
    file_parts = relative[:-3].split("/") if relative.endswith(".rs") and relative not in ("lib.rs", "main.rs") else []
    if file_parts and file_parts[-1] == "mod":
        file_parts.pop()
    modules = [part for part in file_parts if part]
    stack = []
    for line_number, line in enumerate(lines[:target_line], 1):
        opened = re.match(r"\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z_]\w*)\s*\{", line)
        if opened:
            stack.append((opened.group(1), line_number, brace_end([x.rstrip("\r\n") for x in lines], line_number - 1)))
    return modules + [name for name, start, end in stack if start < target_line <= end]


def source_api_location(commit, symbol, kind, allowed_paths=None):
    normalized = symbol.replace("-", "_")
    parts = normalized.split("::")
    crate = parts[0].replace("_", "-")
    crate_root = "crates/" + crate + "/src"
    files = allowed_paths if allowed_paths is not None else source_file_list(commit, crate_root)
    expected_modules = parts[1:-1] if kind in ("function", "builder") and (len(parts) < 2 or not parts[-2][0].isupper()) else parts[1:-2] if kind in ("function", "builder") else parts[1:-1]
    owner = parts[-2] if kind in ("function", "builder") and len(parts) > 2 and parts[-2][0].isupper() else None
    token = parts[-1]
    matches = []
    for path in files:
        try:
            lines = source_lines_at(commit, path)
        except Exception:
            continue
        normalized_lines = [line.rstrip("\r\n") for line in lines]
        if kind in ("function", "builder"):
            declaration = re.compile(r"^\s*pub\s+(?:(?:const|async|unsafe)\s+)*fn\s+" + re.escape(token) + r"\b")
            impls = []
            for index, line in enumerate(normalized_lines):
                if not re.match(r"^\s*impl\b", line):
                    continue
                header = line
                header_end = index
                while "{" not in header and header_end + 1 < len(normalized_lines):
                    header_end += 1
                    header += " " + normalized_lines[header_end]
                impls.append((index, brace_end(normalized_lines, index), header))
            for index, line in enumerate(normalized_lines):
                if not declaration.search(line) or source_module_path(path, lines, index + 1) != expected_modules:
                    continue
                if owner:
                    if any(start < index < end and re.search(r"\b" + re.escape(owner) + r"\b", header) for start, end, header in impls):
                        matches.append((path, index + 1, index + 1))
                else:
                    matches.append((path, index + 1, index + 1))
        elif kind in ("ty", "token"):
            declaration = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:struct|enum|trait|type)\s+" + re.escape(token) + r"\b")
            for index, line in enumerate(normalized_lines):
                if declaration.search(line) and source_module_path(path, lines, index + 1) == expected_modules:
                    matches.append((path, index + 1, index + 1))
    return matches


def retirement_group(symbol, decision_id):
    s = symbol.replace("-", "_")
    if decision_id == "T37-FOUNDATION-TESTING-03":
        return "runtime-stub-harness" if "::stub::" in s else "cx-event-record-probes"
    if decision_id == "T37-FOUNDATION-TESTING-02":
        return "ui-styled-query-probe-state"
    if decision_id == "T37-FOUNDATION-TESTING-01":
        if "IntentQueue::probes" in s:
            return "core-intent-queue-probes"
        if "RenderModel" in s or "ProjectedFrame" in s or "bind_model" in s:
            return "runtime-render-model-and-projected-frame"
        if "Theme::fingerprint" in s:
            return "theme-fingerprint"
        if "StyleCache::stats" in s or "style_cache_stats" in s:
            return "style-cache-hit-miss-counters"
        return "runtime-testing-inspection-impl"
    return None


def retirement_evidence(symbol, kind, retired):
    decision_id, record_path, group_name, conflict, replacement, owner_task = retired
    normalized = symbol.replace("-", "_")
    if decision_id == "T37-SELECTION-BINDING-01":
        record = t37_json(T37_SELECTION_DECISION)
        input_commit = record["input_source_commit"]
        source_path = record["input_path"]
        source_lines = [record["input_line_start"], record["input_line_end"]]
        reviewer = record["independent_decision_agent"]
        artifact_paths = [T37_SELECTION_DECISION]
        recorded_public_paths = record["old_public_paths"]
        source_record_id = record["decision_id"]
        scope_group_id = None
        decision_status = "APPROVED"
        decision = {"decision": "APPROVED", "scope": record["retired_item"], "preserved": record["preserved"], "replacement": record["replacement"]}
        change_records = [{"record_id": record["decision_id"], "path": T37_SELECTION_DECISION, "input_path": record["input_path"], "input_line_start": record["input_line_start"], "input_line_end": record["input_line_end"], "input_span": record["input_span"], "output_source_sha256": record["output_source_sha256"]}]
    elif decision_id == "T37-PROBE-02":
        records = [json.loads(line) for line in git_bytes(INPUT, T37_VIEWPORT_ROWS).splitlines()]
        summary = t37_json(T37_COMPONENT_SUMMARY)["retired_testing_api"]
        input_commit = T37_INPUT
        source_path = "crates/termrock-viewport/src/viewport.rs"
        source_lines = None
        reviewer = summary["independent_decision_agent"]
        artifact_paths = [T37_VIEWPORT_ROWS, T37_COMPONENT_SUMMARY]
        recorded_public_paths = summary.get("public_paths", [normalized])
        source_record_id = None
        scope_group_id = None
        decision_status = "APPROVED" if summary.get("decision_id") == decision_id else "UNRESOLVED"
        decision = {"decision": summary.get("disposition"), "scope": summary.get("scope"), "constraints": summary.get("constraints")}
        located = source_api_location(input_commit, normalized, kind, [source_path])
        if len(located) == 1:
            source_path, start, end = located[0]
            source_lines = [start, end]
        change_records = []
        if source_lines:
            for index, item in enumerate(records, 1):
                if item.get("path") == source_path and item.get("input_line_start", 0) <= source_lines[0] <= item.get("input_line_end", 0):
                    source_record_id = T37_VIEWPORT_ROWS + "#jsonl-line-" + str(index)
                    change_records.append({"record_id": source_record_id, **item})
        if not change_records:
            source_record_id = None
    else:
        artifact = t37_json(T37_DEFERRED)
        groups = {group["id"]: group for group in artifact["groups"]}
        scope_group_id = retirement_group(symbol, decision_id)
        group = groups.get(scope_group_id)
        decision = artifact["review_decisions"].get(decision_id, {})
        source_removal = t37_json(T37_SOURCE_REMOVAL)
        removal_groups = source_removal.get("retired_testing_api_group_ids", [])
        input_commit = T37_INPUT
        source_lines = None
        source_path = None
        reviewer = decision.get("reviewer")
        artifact_paths = [T37_DEFERRED, T37_SOURCE_REMOVAL, T37_FOUNDATION_RUNBOOK]
        recorded_public_paths = group.get("public_paths", []) if group else []
        source_record_id = scope_group_id
        decision_status = decision.get("decision")
        change_records = [{"record_id": scope_group_id, "group": group}] if group else []
        if group:
            source_paths = list(group.get("input_scope_lines", {}))
            declaration_matches = source_api_location(input_commit, normalized, kind, source_paths)
            if len(declaration_matches) != 1:
                declaration_matches = source_api_location(input_commit, normalized, kind)
            if len(declaration_matches) == 1:
                source_path, start, end = declaration_matches[0]
                source_lines = [start, end]
        decision_status = "APPROVED" if decision.get("decision") == "APPROVED" and scope_group_id in removal_groups else "UNRESOLVED"
        decision = {"decision": decision.get("decision"), "scope": decision.get("scope"), "constraints": decision.get("constraints"), "group": group}
    if source_lines and source_path:
        source_bytes = git_bytes(input_commit, source_path)
        source_span = b"".join(source_bytes.splitlines(keepends=True)[source_lines[0] - 1:source_lines[1]])
        input_source = {"commit": input_commit, "path": source_path, "line_start": source_lines[0], "line_end": source_lines[1], "file_sha256": sha(source_bytes), "span_sha256": sha(source_span), "raw_text": source_span.decode("utf-8").rstrip("\r\n"), "qualified_symbol": normalized}
        product_bytes = git_bytes(T37_PRODUCT, source_path)
        current_bytes = git_bytes(INPUT, source_path)
        product_matches = source_api_location(T37_PRODUCT, normalized, kind, [source_path])
        current_matches = source_api_location(INPUT, normalized, kind, [source_path])
        current_absence = {"commit": INPUT, "path": source_path, "file_sha256": sha(current_bytes), "qualified_symbol_absent_from_source": not current_matches, "source_scope": "qualified module path and exact owner/member declaration", "source_scan_matches": current_matches, "evidence": "NOT VERIFIED"}
        historical_presence = {"commit": T37_PRODUCT, "path": source_path, "file_sha256": sha(product_bytes), "qualified_symbol_present_in_source": bool(product_matches), "source_scope": "qualified module path and exact owner/member declaration", "source_scan_matches": product_matches}
    else:
        input_source, current_absence, historical_presence = None, None, None
    artifact_bindings = [{"path": path, "commit": INPUT, "sha256": sha(git_bytes(INPUT, path))} for path in artifact_paths]
    foundation_record = t37_json(T37_FOUNDATION_RUNBOOK)
    decision_rows = {item["id"]: item for item in foundation_record.get("design_decisions", [])}
    reviewer_decision = decision_rows.get(decision_id, {})
    return {"decision_id": decision_id, "decision_status": decision_status, "decision_scope": decision, "reviewer": reviewer or reviewer_decision.get("reviewer"), "reviewer_model": "gpt-6.1-sol", "reviewer_effort": "high", "input_commit": input_commit, "source_record_id": source_record_id, "change_records": change_records, "group_id": scope_group_id, "exact_public_path": normalized, "group_public_paths": recorded_public_paths, "input_source": input_source, "product_source_commit": T37_PRODUCT, "historical_source_presence": historical_presence, "candidate_current_absence": current_absence, "artifact_bindings": artifact_bindings, "conflict": conflict, "replacement": replacement, "owner_task": owner_task, "source_binding_verified": bool(decision_status == "APPROVED" and input_source and current_absence and current_absence["qualified_symbol_absent_from_source"])}


def shell_reach_retirement(symbol, kind):
    normalized = symbol.replace("-", "_")
    decision = json.loads(git_bytes(T01_REACH_DECISION_COMMIT, T01_REACH_DECISION_PATH))
    decision_bytes = git_bytes(T01_REACH_DECISION_COMMIT, T01_REACH_DECISION_PATH)
    if sha(decision_bytes) != T01_REACH_DECISION_SHA256:
        raise ValueError("shell reach decision artifact hash mismatch")
    if decision.get("decision_id") != "T01-HISTORY-SHELL-REACH-RETIRE-01" or decision.get("decision") != "ACCEPT_RETIREMENT_SCOPED":
        raise ValueError("shell reach retirement decision identity mismatch")
    scope_rows = decision["public_scope"]["crate_paths"]
    scope_matches = [(index, item) for index, item in enumerate(scope_rows) if item.get("path") == normalized]
    if len(scope_matches) != 1:
        raise ValueError("shell reach API absent or ambiguous in independent decision scope: " + normalized)
    scope_index, scope_item = scope_matches[0]
    source_path = "crates/termrock-shell/src/reach.rs"
    input_match = source_api_location(T37_INPUT, normalized, kind, [source_path])
    input_source = None
    if len(input_match) == 1:
        _, line_start, line_end = input_match[0]
        raw = git_bytes(T37_INPUT, source_path)
        span = b"".join(raw.splitlines(keepends=True)[line_start - 1:line_end])
        input_source = {"commit": T37_INPUT, "path": source_path, "line_start": line_start, "line_end": line_end, "file_sha256": sha(raw), "span_sha256": sha(span), "raw_text": span.decode("utf-8").rstrip("\r\n"), "qualified_symbol": normalized}
        if scope_item.get("source_path") != source_path or not (scope_item["lines"][0] <= line_start <= scope_item["lines"][1]):
            raise ValueError("shell reach source location falls outside independent decision scope: " + normalized)
    product_raw = git_bytes(T37_PRODUCT, source_path)
    current_paths = source_file_list(INPUT, "crates/termrock-shell/src")
    current_matches = source_api_location(INPUT, normalized, kind, current_paths)
    current_root = git_bytes(INPUT, "crates/termrock-shell/src/lib.rs")
    product_root = git_bytes(T37_PRODUCT, "crates/termrock-shell/src/lib.rs")
    source_removal = t37_json(T37_SOURCE_REMOVAL)
    removal = next((item for item in source_removal.get("removed_source_items", []) if item.get("path") == "crates/termrock-shell/src/lib.rs" and item.get("item") == "pub mod reach"), None)
    deferred = t37_json(T37_DEFERRED)
    shell_scope = next((item for item in deferred.get("removed_scopes", []) if item.get("path") == "crates/termrock-shell/src/lib.rs"), None)
    removal_hash = sha(git_bytes(INPUT, T37_SOURCE_REMOVAL))
    historical_file = next(item for item in decision["source_boundary"]["historical_files"] if item["path"] == source_path)
    historical_module_file = next(item for item in decision["source_boundary"]["historical_files"] if item["path"] == "crates/termrock-shell/src/lib.rs")
    obligation_task_ids = {task for obligation in decision["future_obligations"] for task in obligation["owner_task_ids"]}
    return {"status": "ACCEPTED_SCOPED_RETIREMENT", "decision_id": decision["decision_id"], "decision_status": decision["decision"], "decision_scope": decision["decision_scope"], "decision_artifact": {"path": T01_REACH_DECISION_PATH, "commit": T01_REACH_DECISION_COMMIT, "sha256": sha(decision_bytes)}, "receipt": {"status": "ACCEPT_SCOPED_RECEIPT", "reviewer_id": "/root/t01_inventory_review", "commit": T01_REACH_DECISION_COMMIT, "artifact_sha256": T01_REACH_DECISION_SHA256, "scope": "Check the persisted decision against the independent source decision and all exact bindings."}, "source_api_identity": normalized, "exact_public_path": normalized, "scope_record_id": "public_scope.crate_paths[" + str(scope_index) + "]", "decision_public_scope_record": scope_item, "input_source": input_source, "t37_historical_product_source": {"commit": T37_PRODUCT, "path": source_path, "sha256": sha(product_raw), "expected_sha256": historical_file["sha256"], "git_blob": historical_file["git_blob"], "source_sha256_matches": sha(product_raw) == historical_file["sha256"], "line_start": historical_file["line_range"][0], "line_end": historical_file["line_range"][1]}, "t37_module_export_source": {"commit": T37_PRODUCT, "path": "crates/termrock-shell/src/lib.rs", "sha256": sha(product_root), "expected_sha256": historical_module_file["sha256"], "lines": historical_module_file["line_range"]}, "t37_parent_removal_record": {"path": T37_SOURCE_REMOVAL, "commit": INPUT, "sha256": removal_hash, "record_sha256": removal_hash, "record": removal}, "t37_removed_scope_record": shell_scope, "t00_immutable_evidence": decision["source_boundary"]["immutable_t00_evidence"], "candidate_current_absence": {"commit": INPUT, "reach_module_source_absent": source_path not in current_paths, "qualified_symbol_absent_from_source": not current_matches, "shell_root_path": "crates/termrock-shell/src/lib.rs", "shell_root_sha256": sha(current_root), "expected_shell_root_sha256": decision["source_boundary"]["current_absence"]["candidate_shell_lib_sha256"], "shell_root_reach_declaration_absent": not bool(re.search(rb"(?m)^\\s*pub\\s+mod\\s+reach\\b", current_root)), "current_matches": current_matches, "evidence": "NOT VERIFIED"}, "conflict": decision["conflict_and_replacement"]["specific_conflict"], "replacement": decision["conflict_and_replacement"]["replacement"], "future_obligations": decision["future_obligations"], "future_obligation_owner_task_ids": sorted(obligation_task_ids), "reviewer": decision["decision_origin"]["reviewer_id"], "reviewer_model": decision["decision_origin"]["requested_model"], "reviewer_effort": decision["decision_origin"]["requested_effort"], "review_backend_attestation": decision["decision_origin"]["backend_attestation"], "owner_task": "T19/T22/T27/T28/T29", "source_binding_verified": bool(input_source and input_source["file_sha256"] == historical_file["sha256"] and sha(product_root) == historical_module_file["sha256"] and source_path not in current_paths and not current_matches and sha(current_root) == decision["source_boundary"]["current_absence"]["candidate_shell_lib_sha256"] and scope_item.get("source_path") == source_path and scope_item.get("lines", [0, 0])[0] <= input_source["line_start"] <= scope_item.get("lines", [0, 0])[1] and T01_REACH_DECISION_SHA256 == sha(decision_bytes) and len(decision["future_obligations"]) == 4)}


SOURCE_LINES = {}
SOURCE_HASHES = {}


def source_sha(path):
    if path not in SOURCE_HASHES:
        SOURCE_HASHES[path] = sha(git_bytes(INPUT, path))
    return SOURCE_HASHES[path]


def source_member_location(path, after_line, member, kind, parent_symbol):
    token = member.split("::")[-1]
    owner = parent_symbol.split("::")[-1]
    if kind in ("function", "builder"):
        pattern = re.compile(r"^\s*pub\s+(?:(?:const|async|unsafe)\s+)*fn\s+" + re.escape(token) + r"\b")
    elif kind == "variant":
        pattern = re.compile(r"^\s*" + re.escape(token) + r"\b(?:\s*\(|\s*\{|\s*,|\s*=|\s*$)")
        candidates = [path]
    else:
        return None
    if kind != "variant":
        roots = {path.rsplit("/", 1)[0]}
        if path.endswith("/mod.rs"):
            roots.add(path.rsplit("/", 1)[0])
        else:
            roots.add(path[:-3])
        candidates = {path}
        for root in roots:
            if root == path:
                continue
            obj = subprocess.run(["git", "rev-parse", INPUT + ":" + root], cwd=ROOT, text=True, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
            if obj.returncode != 0:
                continue
            typ = subprocess.run(["git", "cat-file", "-t", obj.stdout.strip()], cwd=ROOT, text=True, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
            if typ.returncode != 0 or typ.stdout.strip() != "tree":
                continue
            candidates.update(x for x in subprocess.run(["git", "ls-tree", "-r", "--name-only", INPUT, "--", root], cwd=ROOT, check=True, text=True, stdout=subprocess.PIPE).stdout.splitlines() if x.endswith(".rs"))
    matches = []
    for source_path in sorted(candidates):
        if source_path not in SOURCE_LINES:
            try:
                SOURCE_LINES[source_path] = lines_at(INPUT, source_path)
            except Exception:
                continue
        lines = SOURCE_LINES[source_path]
        if kind == "variant":
            start = max((after_line or 1) - 1, 0)
            declaration = re.compile(r"^\s*(?:(?:pub(?:\([^)]*\))?)\s+)?enum\s+" + re.escape(owner) + r"\b")
            if start >= len(lines) or not declaration.search(lines[start]):
                continue
            end = brace_end(lines, start)
            matches.extend({"path": source_path, "line": k + 1, "sha256": source_sha(source_path)} for k in range(start + 1, end) if pattern.search(lines[k]))
            continue
        for i, line in enumerate(lines):
            if not re.match(r"^\s*impl\b", line):
                continue
            header = line
            j = i
            while "{" not in header and j + 1 < len(lines):
                j += 1
                header += " " + lines[j]
            if not re.search(r"\b" + re.escape(owner) + r"\b", header):
                continue
            end = brace_end(lines, i)
            matches.extend({"path": source_path, "line": k + 1, "sha256": source_sha(source_path)} for k in range(i + 1, end) if pattern.search(lines[k]))
    return matches[0] if len(matches) == 1 else None


def current_map(symbol, kind, index, components):
    normalized = symbol.replace("-", "_")
    if normalized.startswith("termrock_shell::reach::"):
        retirement = shell_reach_retirement(symbol, kind)
        return {"method": "reviewed_test_api_retirement", "qualified_symbol": None, "definition_owner": "termrock_shell::reach", "public_paths": [normalized], "source": {"path": T01_REACH_DECISION_PATH, "commit": T01_REACH_DECISION_COMMIT, "sha256": retirement["decision_artifact"]["sha256"], "decision_id": retirement["decision_id"]}, "product_source": {"path": retirement["input_source"]["path"], "line": retirement["input_source"]["line_start"], "sha256": retirement["input_source"]["file_sha256"], "symbol": normalized}, "owner_task": "T19/T22/T27/T28/T29", "owner_task_ids": ["T19", "T22", "T27", "T28", "T29"], "component_wid": None, "retirement": {"record_path": T01_REACH_DECISION_PATH, "record_commit": T01_REACH_DECISION_COMMIT, "record_sha256": retirement["decision_artifact"]["sha256"], "source_record_id": retirement["scope_record_id"], "group": None, **retirement}, "source_binding_verified": retirement["source_binding_verified"]}
    retired = retired_record(symbol)
    if retired:
        parent = normalized
        parent_entry = None
        while "::" in parent:
            parent = parent.rsplit("::", 1)[0]
            if parent in index and api_fields(index[parent][0]).get("kind") in ("struct", "enum", "union", "trait", "type"):
                parent_entry = index[parent][0]
                break
        parent_fields = api_fields(parent_entry) if parent_entry else None
        retirement = retirement_evidence(symbol, kind, retired)
        return {
            "method": "reviewed_test_api_retirement",
            "qualified_symbol": None,
            "definition_owner": parent_fields["owner"] if parent_fields else None,
            "public_paths": [],
            "source": {"path": retired[1], "commit": INPUT, "sha256": sha(git_bytes(INPUT, retired[1])), "decision_id": retired[0]},
            "product_source": None if not parent_fields else {"path": parent_fields["path"], "line": parent_fields["line"], "sha256": parent_fields["sha256"], "symbol": parent_fields["symbol"]},
            "owner_task": retired[5],
            "component_wid": None,
            "retirement": {"decision_id": retired[0], "record_path": retired[1], "record_commit": INPUT, "record_sha256": sha(git_bytes(INPUT, retired[1])), "group": retired[2], **retirement},
            "source_binding_verified": retirement["source_binding_verified"],
        }
    candidate = normalized
    method = "exact_current_symbol"
    if candidate not in index:
        moved = block_rewrite(symbol)
        if moved and moved in index:
            candidate, method = moved, "current_module_reorganization"
    if candidate not in index:
        parent = candidate
        while "::" in parent:
            parent = parent.rsplit("::", 1)[0]
            if parent in index and api_fields(index[parent][0]).get("kind") in ("struct", "enum", "union", "trait", "type"):
                p = api_fields(index[parent][0])
                member = candidate[len(parent) + 2:]
                location = source_member_location(p["path"], p["line"], member, kind, parent)
                if location:
                    wid = components.get(p["owner"], {}).get("wid")
                    if wid not in {"W%02d" % i for i in range(1, 46)}:
                        wid = None
                    return {"method": "current_parent_plus_source_member", "qualified_symbol": candidate, "definition_owner": p["owner"], "public_paths": [candidate], "source": {**location, "commit": INPUT}, "product_source": None, "owner_task": p["task"] or task_for_symbol(candidate), "component_wid": wid, "retirement": None, "source_binding_verified": True}
                wid = components.get(p["owner"], {}).get("wid")
                if wid not in {"W%02d" % i for i in range(1, 46)}:
                    wid = None
                return {"method": "parent_type_member_not_located", "qualified_symbol": candidate, "definition_owner": p["owner"], "public_paths": p["paths"], "source": {"path": p["path"], "line": p["line"], "sha256": p["sha256"], "commit": INPUT}, "product_source": None, "owner_task": p["task"] or task_for_symbol(candidate), "component_wid": wid, "retirement": None, "source_binding_verified": False}
        return {"method": "no_current_symbol_or_reviewed_retirement", "qualified_symbol": None, "definition_owner": None, "public_paths": [], "source": None, "product_source": None, "owner_task": task_for_symbol(candidate), "component_wid": None, "retirement": None, "source_binding_verified": False}
    p = api_fields(index[candidate][0])
    comp = components.get(p["owner"])
    wid = comp.get("wid") if comp else None
    if wid not in {"W%02d" % i for i in range(1, 46)}:
        wid = None
    if candidate != p["symbol"]:
        method = "current_reexport_path"
    return {"method": method, "qualified_symbol": candidate, "canonical_definition_symbol": p["symbol"], "definition_owner": p["owner"], "public_paths": sorted(set(p["paths"])), "source": {"path": p["path"], "line": p["line"], "sha256": p["sha256"], "commit": INPUT}, "product_source": None, "owner_task": p["task"] or (comp.get("named_owner_task") if comp else None) or task_for_symbol(candidate), "component_wid": wid, "retirement": None, "source_binding_verified": True}


def build_coverage(component, foundation, routes):
    index, components = api_indexes(component, foundation)
    old_bytes, new_bytes = git_bytes(OLD, ALLOW_OLD), git_bytes(PRESERVED, ALLOW_NEW)
    old_capture = (ROOT / "docs/implementation/pr38-final/results/t00-source/source-deltas/e08-to-1d6/old" / ALLOW_OLD).read_bytes()
    new_capture = (ROOT / "docs/implementation/pr38-final/results/t00-source/source-deltas/e08-to-1d6/new" / ALLOW_NEW).read_bytes()
    if not (old_bytes == new_bytes == old_capture == new_capture):
        raise ValueError("allowlist identity witness mismatch")
    catalog_bytes = git_bytes(OLD, CATALOG_OLD)
    catalog = {}
    catalog_text = catalog_bytes.decode()
    capability_pattern = re.compile(r'Capability::(ty|variant|function|builder|token)\s*\(\s*"([^"]+)"\s*,?\s*\)', re.DOTALL)
    for match in capability_pattern.finditer(catalog_text):
        start = catalog_text.count("\n", 0, match.start()) + 1
        end = catalog_text.count("\n", 0, match.end()) + 1
        span = catalog_text[match.start():match.end()]
        start_line = catalog_text.splitlines()[start - 1]
        catalog.setdefault(match.group(2), []).append({"kind": match.group(1), "line": start, "line_end": end, "line_sha256": sha((start_line + "\n").encode()), "span_sha256": sha(span.encode())})
    old_lines, new_lines = old_bytes.splitlines(keepends=True), new_bytes.splitlines(keepends=True)
    lane_counts, symbol_counts = collections.Counter(), collections.Counter()
    lane_ids, rows = collections.defaultdict(list), []
    for line_no, raw in enumerate(old_lines, 1):
        text = raw.decode().rstrip("\r\n")
        if not text or text.startswith("#"):
            continue
        parts = text.split("|", 2)
        if len(parts) != 3:
            raise ValueError("bad source row at " + str(line_no))
        symbol, claim, lane = [x.strip() for x in parts]
        if lane not in ("SC1", "SC2", "SC3", "review"):
            raise ValueError("bad lane " + lane)
        lane_counts[lane] += 1
        symbol_counts[symbol] += 1
        row_id = OLD + ":" + ALLOW_OLD + ":" + str(line_no)
        inv = catalog.get(symbol, [])
        if not inv:
            raise ValueError("historical ID absent from catalogue: " + symbol)
        kind, inv = inv[0]["kind"], inv[0]
        mapping = current_map(symbol, kind, index, components)
        retired = mapping.get("retirement")
        mapped_entry = index.get(mapping.get("qualified_symbol"), [None])[0]
        mapped_item = mapped_entry[1] if mapped_entry else {}
        is_alias = mapped_item.get("declaration_kind") == "pub type" and " = " in mapped_item.get("signature", "")
        if retired and retired.get("source_binding_verified"):
            category, disposition = "removed_test_api", "RETIRE"
            reason = "A source-bound independent decision retires this helper only. Linked production behavior remains assigned to its named owners and NOT VERIFIED."
        elif retired:
            category, disposition = "removed_test_api", "UNRESOLVED"
            reason = retired["conflict"] + " " + retired["replacement"] + " Bind the exact independent decision artifact before RETIRE."
        elif is_alias:
            category = "alias"
            disposition = "KEEP" if mapping["method"] == "exact_current_symbol" else "ADAPT"
            reason = "The current source declares this public type alias; preserve its target and export routes without treating it as a separate implementation."
        elif lane == "SC3" or "historical page/title association" in claim.lower():
            category, disposition = "product_path", "ADAPT"
            reason = "Historical page/title association is a lead only; resolve it through the current route inventory."
        elif kind in ("ty", "variant", "token"):
            category = "value"
            disposition = "KEEP" if mapping["method"] == "exact_current_symbol" else "ADAPT"
            reason = "Current source identifies the typed value; inventory presence does not prove behavior."
        else:
            category = "behavior"
            disposition = "KEEP" if mapping["method"] == "exact_current_symbol" else "ADAPT"
            reason = "Current source identifies the operation; inventory presence does not prove behavior."
        if mapping["method"] in ("current_module_reorganization", "current_parent_plus_source_member") and not retired:
            disposition = "ADAPT"
            reason += " Current source uses a relocated module or parent-owned member."
        if not mapping["source_binding_verified"] and not retired:
            disposition = "UNRESOLVED"
            reason = "Current source member was not located; the named task must resolve the mapping."
        case_text = ("Keep linked resize, scrolling, reveal, and input obligations on production routes; do not restore the retired reach harness.") if retired and retired.get("decision_id") == "T01-HISTORY-SHELL-REACH-RETIRE-01" else (("Exercise the retained production behavior for " + symbol + " through its owner; do not restore the retired helper.") if retired and retired.get("source_binding_verified") else (("Retain the source API identity and product behavior requirement until the independent retirement artifact is bound.") if retired else ("Resolve this historical page/title lead to its current route, input, and draw behavior." if category == "product_path" else ("Use this value or state through its current definition owner and inspect the consumer-visible result." if kind in ("ty", "variant", "token") else "Drive the operation with named input or non-default state and inspect its current production result."))))
        row_tasks = re.findall(r"T\d\d", mapping.get("owner_task") or task_for_symbol(symbol))
        case = {"case_id": "HIST:" + row_id, "requirement": case_text, "owner_task": mapping.get("owner_task") or task_for_symbol(symbol), "owner_task_ids": row_tasks, "status": "PROPOSED_NOT_APPROVED_T38", "evidence": "NOT VERIFIED", "rationale": reason}
        wid = mapping.get("component_wid")
        routes_for_row = []
        if wid and category == "product_path":
            for route in routes.get("routes", []):
                for shared in route.get("shared_components") or []:
                    if shared.get("wid") == wid:
                        routes_for_row.append({"route_identity": route.get("reference_identity"), "reference_index": route.get("reference_index"), "requirement_id": route.get("requirement_id"), "route_task": route_task(route.get("reference_index", -1)), "registry_owner_task": "T22", "required_case_ids": route.get("required_case_ids", []), "component_case_ids": shared.get("proposed_required_cases", []), "current_route_status": (route.get("current_entry") or {}).get("target_tag"), "evidence": "NOT VERIFIED"})
        new_raw = new_lines[line_no - 1]
        row = {
            "schema": "termrock.pr38.t01.historical-coverage-row.v1",
            "row_id": row_id,
            "lane": lane,
            "lane_ordinal": lane_counts[lane],
            "duplicate_occurrence": symbol_counts[symbol],
            "historical_source": {"commit": OLD, "path": ALLOW_OLD, "line": line_no, "raw_line_sha256": sha(raw), "text_sha256": sha(text.encode()), "file_sha256": sha(old_bytes), "preserved_in": {"commit": PRESERVED, "path": ALLOW_NEW, "line": line_no, "raw_line_sha256": sha(new_raw), "text_sha256": sha(new_raw.decode().rstrip("\r\n").encode()), "file_sha256": sha(new_bytes)}},
            "historical_claim": {"symbol": symbol, "reason": claim, "raw_text": text, "source_inventory": {"commit": OLD, "path": CATALOG_OLD, "line_start": inv["line"], "line_end": inv["line_end"], "file_sha256": sha(catalog_bytes), "raw_line_sha256": inv["line_sha256"], "source_span_sha256": inv["span_sha256"], "catalog_kind": kind}},
            "category": category,
            "semantic_category": kind,
            "current_mapping": mapping,
            "component_wid": wid,
            "owner_task": mapping.get("owner_task") or task_for_symbol(symbol),
            "owner_task_ids": row_tasks,
            "route_associations": routes_for_row,
            "disposition": disposition,
            "disposition_reason": reason,
            "proposed_case": case,
            "case_status": "PROPOSED_NOT_APPROVED_T38",
            "evidence": "NOT VERIFIED",
            "selected_behavior_cases": 0,
            "block_final": True,
        }
        rows.append(row)
        lane_ids[lane].append(row_id)
    if len(rows) != 4511:
        raise ValueError("expected 4511 historical rows, got " + str(len(rows)))
    path = OUT / "historical-coverage.jsonl"
    path.write_bytes(b"".join((json.dumps(x, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n").encode() for x in rows))
    return rows, lane_ids, lane_counts, old_bytes, catalog_bytes


def task_sources(component, foundation):
    all_items, by_task = [], collections.defaultdict(list)
    comps = {x.get("actual_definition_owner"): x for x in component.get("components", [])}
    for x in component.get("items", []):
        sym = x.get("qualified_symbol")
        if not sym: continue
        owner = x.get("definition_symbol") or sym
        comp = comps.get(owner)
        task = comp.get("named_owner_task") if comp else task_for_symbol(sym)
        row = {"path": x.get("source"), "line": x.get("line_start"), "sha256": x.get("sha256"), "symbol": sym, "owner_task": task}
        all_items.append(row)
        by_task[task].append(row)
    for x in foundation.get("items", []):
        sym = x.get("current_qualified_symbol")
        if not sym: continue
        task = x.get("architecture_owner_task") or task_for_symbol(sym)
        row = {"path": x.get("source_path"), "line": x.get("line"), "sha256": x.get("source_sha256"), "symbol": sym, "owner_task": task}
        all_items.append(row)
        by_task[task].append(row)
    return all_items, by_task


def choose_sources(text, tasks, all_items, by_task, cap=6):
    task_set = set(re.findall(r"T\d\d", tasks))
    words = set(re.findall(r"[A-Za-z][A-Za-z0-9_]*", text))
    ranked = []
    for item in all_items:
        symbol = item.get("symbol") or ""
        overlap = words.intersection(re.findall(r"[A-Za-z][A-Za-z0-9_]*", symbol))
        score = len(overlap) * 5 + (5 if item.get("owner_task") in task_set else 0) + (15 if symbol in text else 0)
        if score:
            ranked.append((score, item))
    ranked.sort(key=lambda item: (-item[0], item[1].get("path") or "", item[1].get("line") or 0))
    found, seen = [], set()
    for _, item in ranked:
        key = (item.get("path"), item.get("line"), item.get("symbol"))
        if key not in seen:
            found.append(item)
            seen.add(key)
        if len(found) >= cap:
            break
    if found:
        return found
    for task in task_set:
        if by_task.get(task):
            return by_task[task][:3]
    return []


def doc_sha(path):
    return sha(git_bytes(INPUT, path))


def mig(item_id, path, start, end, text, tasks, current, disposition, reason, case, extra=None, source_commit=INPUT, source_sha=None):
    for source in current:
        if str(source.get("path", "")).startswith("/"):
            source.setdefault("status", "SUPPLEMENTAL_NOT_ACCEPTED")
            source.setdefault("hash_role", "Observed draft artifact hash; it is not a current source binding.")
            source.setdefault("evidence", "NOT VERIFIED")
    task_ids = re.findall(r"T\d\d", tasks)
    source_bytes = git_bytes(source_commit, path) if source_commit else b""
    source_lines = source_bytes.splitlines(keepends=True)
    if start and end and end <= len(source_lines):
        source_span = b"".join(source_lines[start - 1:end])
        source_text = source_span.decode("utf-8").rstrip("\r\n")
        span_hash = sha(source_span)
    else:
        source_span = text.encode()
        source_text = text
        span_hash = sha(source_span)
    x = {"item_id": item_id, "historical_source": {"commit": source_commit, "path": path, "line_start": start, "line_end": end, "sha256": source_sha or (sha(source_bytes) if source_commit else None), "span_sha256": span_hash, "raw_text": source_text, "raw_text_sha256": sha(source_text.encode())}, "historical_links": re.findall(r"\]\((https?://[^)]+)\)", source_text), "original_claim": source_text, "inventory_context": text if text != source_text else None, "current_sources": current, "owner_task": tasks, "owner_task_ids": task_ids, "disposition": disposition, "disposition_reason": reason, "required_case": {"case_id": "MIG:" + item_id, "description": case, "owner_task_ids": task_ids, "status": "PROPOSED_NOT_APPROVED_T38", "evidence": "NOT VERIFIED"}, "evidence": "NOT VERIFIED", "block_final": True}
    if extra: x.update(extra)
    return x


REFERENCE_ROUTE_CACHE = {}


def reference_route_spans(commit, path, page_id):
    key = (commit, path, page_id)
    if key not in REFERENCE_ROUTE_CACHE:
        raw = git_bytes(commit, path)
        lines = raw.decode().splitlines(keepends=True)
        file_sha = sha(raw)
        nav = {}
        factory = {}
        for i, line in enumerate(lines):
            if re.search(r"\bid:\s*PageId::" + re.escape(page_id) + r"\s*,", line):
                start = next((j for j in range(i, -1, -1) if "NavEntry {" in lines[j]), None)
                end = next((j for j in range(i, len(lines)) if re.match(r"\s*},\s*$", lines[j])), None)
                if start is not None and end is not None:
                    raw_span = "".join(lines[start:end + 1])
                    nav = {"line_start": start + 1, "line_end": end + 1, "file_sha256": file_sha, "span_sha256": sha(raw_span.encode()), "raw_text": raw_span}
                break
        factory_matches = [i for i, line in enumerate(lines) if re.search(r"\bPageId::" + re.escape(page_id) + r"\s*=>", line)]
        if len(factory_matches) == 1:
            i = factory_matches[0]
            raw_span = lines[i]
            factory = {"line_start": i + 1, "line_end": i + 1, "file_sha256": file_sha, "span_sha256": sha(raw_span.encode()), "raw_text": raw_span}
        REFERENCE_ROUTE_CACHE[key] = {"file_sha256": file_sha, "nav_entry": nav, "factory": factory}
    return REFERENCE_ROUTE_CACHE[key]


def migration_items(component, foundation, routes):
    all_items, by_task = task_sources(component, foundation)
    out = []
    contract = "docs/contracts/migration.md"
    c_lines = lines_at(INPUT, contract)
    owners = ["T09", "T09", "T09/T14", "T11/T09", "T09", "T10/T15/T18", "T13/T18", "T09", "T09", "T19/T09", "T14/T16/T18", "T31", "T09/T20", "T12/T18", "T07/T08/T22", "T31/T35"]
    architecture_owners = [
        "T31/T16/T18", "T09/T11/T17/T33", "T09/T10/T15/T33",
        "T11/T12/T14/T17", "T02/T04/T06/T22/T32/T33",
        "T31/T32/T35", "T05/T22/T34", "T11/T32", "T05/T32",
        "T02/T32", "T04/T36", "T12/T14", "T12/T19", "T16",
        "T14/T33", "T07/T09/T19/T21/T27/T28", "T10/T15/T18",
        "T14/T19", "T14/T33", "T14", "T16", "T11/T17",
        "T12", "T02/T04/T32/T33", "T13/T14/T18",
        "T11/T12/T14/T15/T19", "T09/T10", "T09/T12",
        "T02/T04/T22/T27/T28/T29/T32/T36", "T04/T36",
        "T11/T12", "T09/T17", "T09/T17",
    ]
    cases = [
        "Send an event that changes flow without repaint and inspect typed actions and invalidation separately.",
        "Route pointer phases through registered geometry and inspect owner and frame freshness.",
        "Reorder, filter, sort, replace, and remove items; inspect stable identity and documented fallback.",
        "Resolve semantic surfaces at supported color capabilities and inspect custom/reset precedence.",
        "Render an exact reference target and inspect suppressed registration without inferring live input permission.",
        "Compose a caller body and drive action, focus, cancel, dismissal, and restoration through the owner.",
        "Exercise a caller-owned validator or callback with typed error and captured value at its real scope.",
        "Publish frame reads across resize and inspect absent, old, and current facts.",
        "Repeat update and draw; inspect semantic mutation in update and runtime registration during draw.",
        "Compare effective displayed chords with handled input; preserve remapping and consumption.",
        "Compare read-only and editable model paths with their documented access and ownership.",
        "Map each removed API to a current product definition, owner, retained behavior, or approved helper retirement.",
        "Resolve dependencies and session state through the owning production boundary.",
        "Exercise component states, actions, builders, defaults, and caller contracts through their owners.",
        "Resolve application routes, factories, dispatch, and draw ownership in all four apps.",
        "Reconcile the historical locator after T38 approves a separate functional architecture.",
    ]
    for ln in list(range(11, 27)) + list(range(39, 72)):
        text = c_lines[ln - 1]
        ix = ln - 11 if ln < 27 else ln - 39 + 16
        task = owners[ix] if ln < 27 else architecture_owners[ln - 39]
        case = cases[ix] if ln < 27 else [x.strip() for x in text.strip("|").split("|")][1].replace("**KEEP / REVIEW:**", "").strip()
        case_source = {"kind": "historical_requirement_excerpt", "commit": INPUT, "path": contract, "line": ln, "file_sha256": doc_sha(contract), "evidence": "NOT VERIFIED"} if ln >= 39 else None
        out.append(mig("MIG:docs/contracts/migration.md:" + str(ln), contract, ln, ln, text, task, choose_sources(text, task, all_items, by_task), "ADAPT", "Retain the historical requirement and map it to current source; behavior remains open.", case, {"record_kind": "retained_migration_contract_row", "semantic_status": "KEEP_REVIEW", "required_case_source": case_source}))
    record = "docs/architecture/termrock-migration-record.md"
    rec = lines_at(INPUT, record)
    gap_tasks = ["T10/T12/T13/T14/T15/T16/T17/T18", "T10/T19", "T09/T17", "T12", "T14/T16", "T19/T22", "T16", "T11/T22/T27/T28/T29", "T09/T11", "T32", "T20", "T09/T11"]
    for ln in range(190, 202):
        text = rec[ln - 1]
        cells = [z.strip() for z in text.strip("|").split("|")]
        gap = cells[0] if cells else "G?"
        task = gap_tasks[ln - 190]
        out.append(mig("ARCH:GAP:" + gap, record, ln, ln, text, task, choose_sources(text, task, all_items, by_task), "ADAPT", "The dated gap is a task lead; current inventory does not prove it closed.", "Trace the named app bypass to its shared owner and preserve the required app behavior.", {"record_kind": "architecture_library_gap"}))
    score_tasks = ["T15", "T09/T10", "T09/T10/T22", "T11/T12/T16", "T09/T12/T14/T17", "T13/T15/T20/T32", "T09/T12/T16"]
    for ln in range(207, 214):
        text, task = rec[ln - 1], score_tasks[ln - 207]
        out.append(mig("ARCH:SCORECARD:" + str(ln), record, ln, ln, text, task, choose_sources(text, task, all_items, by_task), "ADAPT", "The old scorecard is historical context; current source mapping does not adopt its test claim.", "Exercise the stated contract through its production owner after T38 approval.", {"record_kind": "architecture_api_contract_scorecard"}))
    for start, end, ident, task, case in [
        (217, 224, "ARCH:DEPENDENCY-DIRECTION", "T02/T04/T32", "Inspect production dependency edges and keep domain ownership separate from UI types."),
        (226, 227, "ARCH:POLICY-LIBRARY-COVERAGE", "T02/T32", "Reconcile policy coverage with current workspace crates and manifests."),
    ]:
        text = "\n".join(rec[start - 1:end])
        out.append(mig(ident, record, start, end, text, task, choose_sources(text, task, all_items, by_task), "ADAPT", "Keep the architecture claim open until current manifests and policy facts are reconciled.", case, {"record_kind": "architecture_dependency_or_policy_claim"}))
    over_tasks = ["T10/T12", "T19/T14", "T19/T12", "T22/T12/T13/T14/T15/T16/T17/T18", "T15/T19"]
    over_cases = ["Check MasterDetail width, bounds, gap, and consumer behavior against the frozen requirement.", "Check for a responsive clickable breadcrumb widget and owner.", "Check footer status and reveal ownership through a current app flow.", "Reconcile page claims against all current route factories, owners, dispatch, and draw paths.", "Inspect Menu command mapping and dialog inset ownership."]
    oi = 0
    for ln in range(231, 251):
        if not rec[ln - 1].startswith("- "): continue
        text, task = rec[ln - 1], over_tasks[oi]
        out.append(mig("ARCH:MATRIX-OVERSTATEMENT:" + str(ln), record, ln, min(ln + (1 if ln in (231, 237, 240, 242, 247) else 0), 251), text, task, choose_sources(text, task, all_items, by_task), "ADAPT", "Preserve the dated claim and inspect current implementation before adopting it.", over_cases[oi], {"record_kind": "historical_matrix_overstatement"}))
        oi += 1
    decision_heads = []
    for i, text in enumerate(rec, 1):
        m = re.match(r"### (D-\d+)\s+[—-]", text)
        if m: decision_heads.append((i, m.group(1)))
    for ix, (start, decision) in enumerate(decision_heads):
        end = decision_heads[ix + 1][0] - 1 if ix + 1 < len(decision_heads) else start
        while end > start and not rec[end - 1].strip(): end -= 1
        text = "\n".join(rec[start - 1:end])
        task = "T32/T36"
        if decision in ("D-002", "D-003", "D-004", "D-005", "D-006"): task = "T04/T22/T27/T28/T29/T32/T36"
        if decision in ("D-011", "D-012"): task = "T17/T22/T27/T28/T29/T32/T36"
        disp = "KEEP" if decision in ("D-001", "D-002", "D-003", "D-004", "D-006", "D-009") else "ADAPT"
        out.append(mig("ARCH:DECISION:" + decision, record, start, end, text, task, choose_sources(text, task, all_items, by_task), disp, "Preserve the independent historical decision; re-check its boundary against current source and keep old results historical.", "Check the decision with its current owner and linked production path; keep conflicts open.", {"record_kind": "architecture_decision", "decision_id": decision}))
    matrix = "docs/architecture/termrock-migration-matrix.md"
    ml = lines_at(INPUT, matrix)
    for ln in range(731, 736):
        text = ml[ln - 1]
        app = text.split("|")[1].strip()
        task = "T28" if app.startswith("Jackin") else {"Showcase": "T22/T23/T24/T25/T26", "TablePro": "T29", "Holla": "T27"}.get(app, "T27")
        base = {"Showcase": "crates/showcase-ui", "Jackin": "crates/jackin-preview-app", "TablePro": "crates/tablepro-ui", "Holla": "crates/holla-ui"}["Jackin" if app.startswith("Jackin") else app]
        current_path = base + "/src/app.rs"
        current_raw = git_bytes(INPUT, current_path)
        out.append(mig("APP-MATRIX:" + app + ":" + str(ln), matrix, ln, ln, text, task, [{"path": current_path, "commit": INPUT, "sha256": sha(current_raw), "line_count": len(current_raw.decode().splitlines()), "owner_task": task}], "ADAPT", "This dated app row is historical. Current app and route maps supersede it as inventory, not behavior proof.", "Walk the named app flow through its production owner and verify its states and routes after T38.", {"record_kind": "historical_application_migration_row", "application": app, "row_status": "HISTORICAL_NOT_CURRENT_RESULT"}))
    for name in ["RadioField", "Note", "TextSection", "SelectField", "Table", "CompletingCodeEditor", "FormDialog", "AppShell", "MasterDetail", "SplitBlock", "Notice", "DerivedHintBar", "TextReport"]:
        found = [x for x in component.get("components", []) if x.get("name") == name]
        if name == "MasterDetail": found = [x for x in found if x.get("actual_definition_owner", "").startswith("termrock_controls::")]
        comp = found[0] if found else None
        task, wid, current, behavior = "T01", None, [], "Inventory the current definition and preserve its task-owned contract."
        if comp:
            task, wid = comp.get("named_owner_task", "T01"), comp.get("wid")
            d = comp.get("definition", {})
            current = [{"path": d.get("path"), "line": d.get("line"), "sha256": d.get("sha256"), "commit": INPUT, "symbol": comp.get("actual_definition_owner")}]
            behavior = "; ".join(comp.get("required_cases", [])) or behavior
        text = lines_at(INPUT, "docs/implementation/pr38-final/TASKS.md")[231]
        out.append(mig("TASKS:ADDITIONAL-COMPONENT:" + name, "docs/implementation/pr38-final/TASKS.md", 232, 232, text, task, current, "ADAPT", "The named reusable surface is source-mapped; no new identifier or behavior result is claimed.", behavior, {"record_kind": "additional_named_component", "component_name": name, "component_wid": wid, "source_artifact": {"path": str(COMPONENT_PATH), "sha256": path_sha(COMPONENT_PATH)}}))
    component_policy = "docs/implementation/pr38-final/COMPONENT-MATRIX.md"
    matrix_source = "docs/implementation/pr38-final/COMPONENT-MATRIX.md"
    for wid, name, owner_task, case in [
        ("W35", "PropsList", "T12/T14", "Keep PropsList defined by controls; verify navigation only reexports it and both public paths resolve to that definition."),
        ("W45", "ScrollRegion", "T17/T09", "Keep ScrollRegion defined by runtime; verify viewport and family paths remain reexports of that definition."),
    ]:
        comp = next((x for x in component.get("components", []) if x.get("wid") == wid), None)
        d = (comp or {}).get("definition", {})
        line = d.get("line")
        src = [{"path": d.get("path"), "line": line, "sha256": d.get("sha256"), "commit": INPUT, "symbol": (comp or {}).get("actual_definition_owner")}]
        matrix_rows = lines_at(INPUT, component_policy)
        matrix_line = next((i for i, text in enumerate(matrix_rows, 1) if text.startswith("| " + wid + " |")), 89)
        source_text = matrix_rows[matrix_line - 1] if matrix_line <= len(matrix_rows) else wid + " definition owner and reexports."
        out.append(mig("COMPONENT-OWNER:" + wid, component_policy, matrix_line, matrix_line, source_text, owner_task, src, "KEEP", "Preserve the source definition owner; family and facade export paths do not transfer ownership.", case, {"record_kind": "definition_owner_reexport_contract", "wid": wid, "definition_owner": (comp or {}).get("actual_definition_owner"), "observed_reexports": (comp or {}).get("all_reexports", []), "source_artifact": {"path": str(COMPONENT_PATH), "sha256": path_sha(COMPONENT_PATH)}}))
    old_ownership = git_bytes(OLD, "component-ownership.json")
    current_wids = {x.get("wid"): x.get("actual_definition_owner") for x in component.get("components", []) if x.get("wid")}
    out.append(mig("COMPONENT-WID-IDENTITY", matrix_source, 89, 89, lines_at(INPUT, matrix_source)[88], "T01/T31", [{"path": "component-ownership.json", "commit": OLD, "sha256": sha(old_ownership), "row_count": len(json.loads(old_ownership)["components"])}, {"path": matrix_source, "commit": INPUT, "line_start": 33, "line_end": 39, "sha256": doc_sha(matrix_source)}], "KEEP", "Keep W01–W45 stable. The component inventory owner assigns new identifiers; removed identifiers stay reserved.", "Compare the fixed historical W01–W45 registry with the current component registry; keep each existing identity stable and leave removed IDs reserved.", {"record_kind": "component_identifier_nonreuse_contract", "current_component_ids": current_wids, "source_artifact": {"path": str(COMPONENT_PATH), "sha256": path_sha(COMPONENT_PATH)}, "allocated_by": "/root/t01_components_api"}))
    task_text = lines_at(INPUT, "docs/implementation/pr38-final/TASKS.md")[234]
    out.append(mig("HISTORICAL-EXPORT-RECONCILIATION", "docs/implementation/pr38-final/TASKS.md", 235, 235, task_text, "T01/T05/T31/T35", [{"path": str(COMPONENT_PATH), "sha256": path_sha(COMPONENT_PATH), "export_rows": len(component.get("exports", [])), "current_item_rows": len(component.get("items", [])), "historical_disposition_rows": len(component.get("historical_migration_dispositions", []))}, {"path": str(FOUNDATION_PATH), "sha256": path_sha(FOUNDATION_PATH), "export_rows": len(foundation.get("exports", [])), "current_item_rows": len(foundation.get("items", [])), "historical_disposition_rows": len(foundation.get("historical_api_dispositions", []))}, {"path": "docs/implementation/pr38-final/results/t37-foundations/deferred-testing-surfaces.json", "commit": INPUT, "sha256": doc_sha("docs/implementation/pr38-final/results/t37-foundations/deferred-testing-surfaces.json")}], "ADAPT", "The row requires a review decision for each RETIRE. Current declarations and dispositions must join by exact source identity; missing joins stay open.", "For each removed public path, retain a current definition/reexport map or an exact approved retirement record, decision scope, source, and replacement behavior owner.", {"record_kind": "removed_export_migration_guard", "component_historical_dispositions": component.get("historical_migration_dispositions", []), "foundation_historical_disposition_count": len(foundation.get("historical_api_dispositions", [])), "retirement_decision_paths": ["docs/implementation/pr38-final/results/t37-foundations/deferred-testing-surfaces.json", "docs/implementation/pr38-final/results/t37-components/approved-testing-api-retirement.jsonl", "docs/implementation/pr38-final/results/t37-components/approved-selection-binding-retirement.json"], "evidence": "NOT VERIFIED"}))
    route_source_index = {(x.get("commit"), x.get("path")): x for x in routes.get("sources", [])}
    for route in routes.get("routes", []):
        owner = route.get("route_owner") or {}
        refs = route.get("source_refs", [])
        ref = next((x for x in refs if x.startswith("5b3dbd7ae799f325efbb4f4cc11b2f5518dec948:")), refs[0] if refs else None)
        ref_commit, ref_path = ref.split(":", 1) if ref else (None, None)
        ref_source = route_source_index.get((ref_commit, ref_path), {})
        page_id = route.get("reference_page_id") or route["reference_identity"].split("::")[-1]
        reference_spans = reference_route_spans(ref_commit, ref_path, page_id) if ref_commit and ref_path else {}
        nav_span = reference_spans.get("nav_entry", {})
        text = nav_span.get("raw_text") or ("PR26 route source span unresolved for " + route["reference_identity"])
        target_tag = (route.get("current_entry") or {}).get("target_tag")
        task = route_task(route["reference_index"])
        current_sources = []
        for source_ref in refs:
            if not source_ref.startswith(INPUT + ":"):
                continue
            source_commit, source_path = source_ref.split(":", 1)
            source_row = route_source_index.get((source_commit, source_path), {})
            current_sources.append({"path": source_path, "commit": source_commit, "sha256": source_row.get("sha256"), "lines": source_row.get("lines"), "role": source_row.get("role")})
        current_sources.append({"path": str(ROUTE_PATH), "sha256": path_sha(ROUTE_PATH), "route_identity": route["reference_identity"], "requirement_id": route["requirement_id"], "current_factory": route.get("current_factory"), "actual_draw_path": route.get("actual_draw_path"), "required_case_ids": route.get("required_case_ids", []), "evidence": "NOT VERIFIED"})
        case_description = "Follow this route's listed cases through its current owner after T38 approval."
        route_snapshot = {k: route.get(k) for k in ("current_entry", "reference_aliases", "current_aliases", "current_name_resolution", "current_factory", "input_state_dispatch", "actual_draw_path", "current_page_owner", "route_owner", "shared_components", "required_case_ids", "required_case_status", "required_behaviors", "requirement_id", "current_route_status", "current_content_path_exists", "current_navigation_keys", "current_page_owner_state", "planned_content_owner", "update_dispatch", "command_dispatch", "draw_dispatch", "source_locations")}
        route_item = mig("APP-ROUTE:" + route["requirement_id"], ref_path or "src/bin/showcase/app.rs", nav_span.get("line_start"), nav_span.get("line_end"), text, "T22/" + task if task != "T22" else "T22", current_sources, "KEEP" if target_tag == "Live" else "ADAPT", "The route inventory maps the fixed factory, current status, owner, dispatch and proposed cases; no case has run.", case_description, {"record_kind": "current_showcase_route_crosswalk", "historical_reference": {"commit": ref_commit, "path": ref_path, "source_sha256": ref_source.get("sha256"), "reference_identity": route["reference_identity"], "reference_index": route["reference_index"], "reference_factory": route.get("reference_factory"), "nav_entry_span": nav_span, "factory_span": reference_spans.get("factory")}, "source_artifact": {"path": str(ROUTE_PATH), "sha256": path_sha(ROUTE_PATH)}, "route_snapshot": route_snapshot, "required_case_ids": route.get("required_case_ids", []), "current_route_status": target_tag, "route_registry_owner": "crates/showcase-ui/src/app.rs", "route_registry_task": "T22", "route_content_owner": route.get("current_page_owner"), "evidence": "NOT VERIFIED"}, source_commit=ref_commit, source_sha=ref_source.get("sha256"))
        route_item["historical_source"]["span_sha256"] = nav_span.get("span_sha256")
        route_item["historical_source"]["source_artifact"] = {"path": str(ROUTE_PATH), "sha256": path_sha(ROUTE_PATH)}
        out.append(route_item)
    route_history = git_bytes(INPUT, "docs/current-state.md")
    f12_lines = git_bytes(INPUT, "docs/implementation/pr38-final/REVIEW.md").splitlines(keepends=True)
    f12_span = b"".join(f12_lines[300:314])
    out.append(mig("CURRENT:SHOWCASE-ROUTE-CENSUS", "docs/current-state.md", 18, 18, lines_at(INPUT, "docs/current-state.md")[17], "T22/T23/T24/T25/T26", [{"path": "crates/showcase-ui/src/app.rs", "line_start": 275, "line_end": 600, "sha256": sha(git_bytes(INPUT, "crates/showcase-ui/src/app.rs")), "commit": INPUT}, {"path": str(ROUTE_PATH), "sha256": path_sha(ROUTE_PATH)}], "ADAPT", "The 22-versus-54 statement compares named historical branches; bind the PR38 target census to its current route source instead. Historical records remain unchanged.", "Map all 54 reference identities to the current factory, owner, dispatch, shared components, and proposed route cases.", {"record_kind": "current_correction", "correction": "stale_showcase_route_claim", "route_counts": routes.get("route_counts"), "current_route_source": {"path": "crates/showcase-ui/src/app.rs", "line_start": 275, "line_end": 600, "file_sha256": sha(git_bytes(INPUT, "crates/showcase-ui/src/app.rs")), "commit": INPUT}, "related_historical_records": [{"commit": INPUT, "path": "docs/implementation/pr38-final/REVIEW.md", "line_start": 301, "line_end": 314, "file_sha256": sha(git_bytes(INPUT, "docs/implementation/pr38-final/REVIEW.md")), "span_sha256": sha(f12_span), "raw_text": f12_span.decode("utf-8").rstrip("\r\n")}], "source_artifact": {"path": str(ROUTE_PATH), "sha256": path_sha(ROUTE_PATH)}, "evidence": "NOT VERIFIED"}))
    null_plan_lines = lines_at(INPUT, "docs/plan.md")
    f20_lines = git_bytes(INPUT, "docs/implementation/pr38-final/REVIEW.md").splitlines(keepends=True)
    f20_span = b"".join(f20_lines[463:477])
    out.append(mig("CURRENT:TABLEPRO-NULL-LAST", "docs/plan.md", 15, 15, null_plan_lines[14], "T29/T33", [{"path": "crates/tablepro-sql/src/sql.rs", "line_start": 1001, "line_end": 1017, "sha256": "3eb8903c45d00eeedd4102a54082fd98cc1531c7b6ae4125c1403de94c1a30c7", "commit": INPUT}, {"path": "crates/tablepro-domain/src/domain.rs", "line": 930, "sha256": "3177a180c0ae6b08f99ec5d8388f96eaf887ff495fc89ac953c37da84c020687", "commit": INPUT}], "ADAPT", "The referenced old comparator claim is stale. Current source keeps NULL precedence outside direction reversal; this source inventory is not behavior proof.", "Sort typed values ascending and descending through the domain path and verify NULL stays last in both.", {"record_kind": "current_correction", "correction": "stale_null_sort_claim", "current_comparator": "cmp_values_nulls_last", "related_historical_records": [{"commit": INPUT, "path": "docs/implementation/pr38-final/REVIEW.md", "line_start": 464, "line_end": 477, "file_sha256": sha(git_bytes(INPUT, "docs/implementation/pr38-final/REVIEW.md")), "span_sha256": sha(f20_span), "raw_text": f20_span.decode("utf-8").rstrip("\r\n")}], "evidence": "NOT VERIFIED"}))
    return out


def application_inventory(routes):
    rows = []
    specs = [
        ("Holla", "crates/holla-ui", "T27", [734, 735], ["Preserve simulation-owned domain actions while resolving shell, menu, picker, list, dialog, status, layout, and scroll behavior through production routes."]),
        ("Jackin", "crates/jackin-preview-app", "T28", [732], ["Preserve simulation and domain actions while resolving menu, picker, editor, completion, drawer, layout, shell, focus, cancel, and resize flows."]),
        ("Showcase", "crates/showcase-ui", "T22/T23/T24/T25/T26", [731], ["Resolve all 54 reference routes through current registry, page owner, input dispatch, factory, and draw paths."]),
        ("TablePro", "crates/tablepro-ui", "T29", [733], ["Preserve SQL and connection effects while resolving picker, tabs, grid, editor, dialogs, shell, layout, scroll, and resize flows."]),
    ]
    matrix = "docs/architecture/termrock-migration-matrix.md"
    matrix_lines = lines_at(INPUT, matrix)
    for app, base, tasks, line_numbers, cases in specs:
        source_path = base + "/src/app.rs"
        source = git_bytes(INPUT, source_path)
        current_source = {"path": source_path, "commit": INPUT, "sha256": sha(source), "line_count": len(source.decode().splitlines()), "owner_task": tasks}
        source_ids = ["APP-MATRIX:" + app + ":" + str(ln) for ln in line_numbers]
        historical_sources = [{"commit": INPUT, "path": matrix, "line": ln, "sha256": doc_sha(matrix), "text_sha256": sha(matrix_lines[ln - 1].encode())} for ln in line_numbers]
        rows.append({"application": app, "historical_migration_item_ids": source_ids, "historical_sources": historical_sources, "current_sources": [current_source], "owner_tasks": tasks.split("/"), "required_cases": [{"case_id": "APP:" + app.upper() + ":MIGRATION", "description": cases[0], "status": "PROPOSED_NOT_APPROVED_T38", "evidence": "NOT VERIFIED"}], "disposition": "ADAPT", "evidence": "NOT VERIFIED", "block_final": True, "route_crosswalk": {"path": str(ROUTE_PATH), "sha256": path_sha(ROUTE_PATH), "route_count": len(routes.get("routes", [])), "join": "reference_identity + reference_index"} if app == "Showcase" else None})
    return rows


def perf_specs():
    return [
        ("crates/termrock-test-support/tests/perf.rs", ["T33", "T09", "T10", "T11", "T12", "T13", "T14", "T15", "T16", "T17", "T18"]),
        ("crates/termrock-test-support/tests/perf_collections.rs", ["T33", "T14", "T16", "T17"]),
        ("crates/holla-ui/tests/perf.rs", ["T33", "T27"]),
        ("crates/jackin-preview-app/tests/perf.rs", ["T33", "T28"]),
        ("crates/showcase-ui/tests/perf.rs", ["T33", "T22"]),
        ("crates/tablepro-ui/tests/perf.rs", ["T33", "T29", "T16", "T17", "T18"]),
        ("crates/termrock-controls/tests/props_rich_perf.rs", ["T33", "T12"]),
    ]


def perf_current_paths(name, historical_path):
    n = name.lower()
    if "crates/holla-ui/tests/perf.rs" in historical_path: return ["crates/holla-ui/src/app.rs", "crates/holla-sim/src/scenario.rs"]
    if "crates/jackin-preview-app/tests/perf.rs" in historical_path: return ["crates/jackin-preview-app/src/app.rs", "crates/jackin-preview-presentation/src"]
    if "crates/showcase-ui/tests/perf.rs" in historical_path: return ["crates/showcase-ui/src/app.rs"]
    if "crates/tablepro-ui/tests/perf.rs" in historical_path: return ["crates/tablepro-ui/src", "crates/tablepro-domain/src/domain.rs", "crates/tablepro-sql/src/sql.rs"]
    if "holla" in n or "scenario" in n: return ["crates/holla-ui/src/app.rs", "crates/holla-sim/src/scenario.rs"]
    if "jackin" in n or "manager" in n or "capsule" in n: return ["crates/jackin-preview-app/src/app.rs", "crates/jackin-preview-presentation/src"]
    if "showcase" in n: return ["crates/showcase-ui/src/app.rs"]
    if "tablepro" in n or "grid_" in n or "local_sort" in n or "query_editor" in n or "connection_form" in n: return ["crates/tablepro-ui/src", "crates/tablepro-domain/src/domain.rs", "crates/tablepro-sql/src/sql.rs"]
    if "viewport" in n or "textbuffer" in n: return ["crates/termrock-viewport/src/viewport.rs", "crates/termrock-text/src"]
    if any(x in n for x in ("list", "picker", "tree", "steps", "collection")): return ["crates/termrock-navigation/src", "crates/termrock-collections/src"]
    if any(x in n for x in ("style", "theme", "backdrop")): return ["crates/termrock-theme/src/resolve.rs", "crates/termrock-runtime/src/ui"]
    if "diff" in n or "editor" in n: return ["crates/termrock-editors/src"]
    if "form" in n: return ["crates/termrock-forms/src"]
    if "props" in n: return ["crates/termrock-controls/src/props.rs"]
    if any(x in n for x in ("width", "truncate", "fit_", "wrap", "fuzzy", "measure")): return ["crates/termrock-text/src"]
    return ["crates/termrock-runtime/src", "crates/termrock-render/src"]


CURRENT_SOURCE_BINDINGS = {}


def current_source_binding(path):
    if path in CURRENT_SOURCE_BINDINGS:
        return CURRENT_SOURCE_BINDINGS[path]
    object_id = subprocess.run(["git", "rev-parse", INPUT + ":" + path], cwd=ROOT, check=True, text=True, stdout=subprocess.PIPE).stdout.strip()
    object_type = subprocess.run(["git", "cat-file", "-t", object_id], cwd=ROOT, check=True, text=True, stdout=subprocess.PIPE).stdout.strip()
    if object_type == "blob":
        raw = git_bytes(INPUT, path)
        result = {"path": path, "commit": INPUT, "kind": "source_file", "sha256": sha(raw), "line_count": len(raw.decode().splitlines())}
    else:
        entries = subprocess.run(["git", "ls-tree", "-r", "--full-tree", "--format=%(objectname) %(path)", INPUT, "--", path], cwd=ROOT, check=True, text=True, stdout=subprocess.PIPE).stdout.splitlines()
        result = {"path": path, "commit": INPUT, "kind": "source_tree", "git_tree_object": object_id, "tree_manifest_sha256": sha(("\n".join(entries) + "\n").encode()), "file_count": len(entries)}
    CURRENT_SOURCE_BINDINGS[path] = result
    return result


def brace_end(lines, start):
    depth, opened = 0, False
    for i in range(start, len(lines)):
        depth += lines[i].count("{") - lines[i].count("}")
        opened = opened or "{" in lines[i]
        if opened and depth <= 0: return i
    return min(start, len(lines) - 1)


def performance_measurement_model():
    path = "crates/termrock-test-support/src/perf.rs"
    raw = git_bytes(OLD, path)
    return {
        "source": {"commit": OLD, "path": path, "sha256": sha(raw)},
        "claims": [
            {"lines": [7, 24], "claim": "Thread-local allocation and byte counters increment for allocation, zeroed allocation, and growing reallocation."},
            {"lines": [92, 113], "claim": "Debug sizes and iteration defaults fall to one tenth unless PERF_FULL is set; PERF_ITERS caps iterations at a minimum of one."},
            {"lines": [115, 149], "claim": "bench warms first, runs three debug or nine release repetitions, normalizes per operation, and reports each metric median."},
            {"lines": [151, 198], "claim": "bench_sampled uses the same repetition and median protocol; Sample.measure times only its explicit closure."},
            {"lines": [200, 213], "claim": "measure_once records one elapsed time and allocation delta without repeated sampling."},
            {"lines": [267, 293], "claim": "PERF_BLESS writes the observed baseline; that historical path is not approval to change a current baseline."},
            {"lines": [295, 394], "claim": "Missing baselines print PERF-NOBASE and return; debug metric drift prints, release allocation or region drift fails, and timing fails only with PERF_STRICT."},
            {"lines": [396, 402], "claim": "check_ratio prints its ratio and fails only when its strict argument is true."},
        ],
        "clock": "The helpers use Instant::now; deterministic scenario clocks must be inventoried per entry point.",
        "baseline_claims": ["Allocation and byte values must not exceed the baseline in release.", "Hit counts tolerate ten percent drift; ring lengths require equality.", "Time above 120 percent warns unless PERF_STRICT and release are both active."],
        "current_owner_task": "T33",
        "measurement_status": "NOT VERIFIED",
    }


def workloads():
    result, sources = [], []
    interesting = re.compile(r"bench|measure_once|samples|assert|check_ratio|PERF_|report_to|baseline|KeyCode::|MouseKind::|Input::|Intent::|Motion::|Scenario::|Rect::|Terminal::|TestBackend::|\.key\(|\.mouse\(|\.draw\(|\.input\(")
    for path, tasks in perf_specs():
        raw = git_bytes(OLD, path)
        lines = raw.decode().splitlines()
        source_lines_raw = raw.splitlines(keepends=True)
        function_defs = {}
        for function_start, function_line in enumerate(lines):
            definition = re.match(r"\s*(?:pub\s+)?(?:const\s+)?fn\s+([A-Za-z_]\w*)\s*\(", function_line)
            if not definition:
                continue
            function_end = brace_end(lines, function_start)
            function_defs[definition.group(1)] = (function_start, function_end, "\n".join(lines[function_start:function_end + 1]))
        sources.append({"path": path, "commit": OLD, "sha256": sha(raw), "lines": len(lines), "status": "historical workload source; not executed"})
        for attr, line in enumerate(lines):
            if line.strip() != "#[test]": continue
            start = next((j for j in range(attr + 1, min(attr + 8, len(lines))) if re.match(r"\s*fn\s+[A-Za-z_]\w*\s*\(", lines[j])), None)
            if start is None: continue
            name = re.search(r"\bfn\s+([A-Za-z_]\w*)", lines[start]).group(1)
            end = brace_end(lines, start)
            body = lines[start:end + 1]
            code = "\n".join(body)
            span_raw = b"".join(source_lines_raw[start:end + 1])
            pending = [called_name for called_name in re.findall(r"\b([A-Za-z_]\w*)\s*\(", code) if called_name in function_defs and called_name != name]
            helper_names = set()
            search_code = code
            while pending:
                helper_name = pending.pop()
                if helper_name in helper_names or helper_name not in function_defs:
                    continue
                helper_names.add(helper_name)
                helper_code = function_defs[helper_name][2]
                search_code += "\n" + helper_code
                pending.extend(name for name in re.findall(r"\b([A-Za-z_]\w*)\s*\(", helper_code) if name in function_defs and name not in helper_names)
            helper_bindings = []
            for helper_name in sorted(helper_names, key=lambda x: function_defs[x][0]):
                helper_start, helper_end, _ = function_defs[helper_name]
                helper_span = b"".join(source_lines_raw[helper_start:helper_end + 1])
                helper_bindings.append({"function": helper_name, "path": path, "commit": OLD, "source_sha256": sha(raw), "line_start": helper_start + 1, "line_end": helper_end + 1, "span_sha256": sha(helper_span)})
            facts = [{"line": start + i + 1, "text": x.strip()[:300]} for i, x in enumerate(body) if interesting.search(x)]
            helper_claims = [{"function": helper_name, "line": i + 1, "text": x.strip()[:300]} for helper_name in helper_names for i, x in enumerate(lines[function_defs[helper_name][0]:function_defs[helper_name][1] + 1], function_defs[helper_name][0]) if interesting.search(x)]
            used_constants = set(re.findall(r"\b(?:WIDTH|HEIGHT|WARMUPS|SAMPLES|ITERATIONS|QUERY|STEPS|PERF_[A-Z_]+)\b", search_code))
            configuration_constants = [{"line": i + 1, "text": x.strip()} for i, x in enumerate(lines) if re.match(r"\s*(?:pub\s+)?const\s+[A-Z_][A-Z0-9_]*", x) and any(re.search(r"\b" + re.escape(constant) + r"\b", x) for constant in used_constants)]
            nums = []
            for m in re.finditer(r"\b\d[\d_]*(?:x\d[\d_]*)?\b", name + "\n" + code):
                if m.group(0) not in nums: nums.append(m.group(0))
            dims = sorted(set(re.findall(r"\b\d{1,4}\s*[x×]\s*\d{1,4}\b", name + "\n" + code)))
            events = [x.strip()[:240] for x in body if re.search(r"KeyCode::|MouseKind::|Input::|Intent::|Motion::|Scenario::|\.key\(|\.mouse\(", x)]
            helper_events = [{"function": helper_name, "line": i + 1, "text": x.strip()[:240]} for helper_name in helper_names for i, x in enumerate(lines[function_defs[helper_name][0]:function_defs[helper_name][1] + 1], function_defs[helper_name][0]) if re.search(r"KeyCode::|MouseKind::|Input::|Intent::|Motion::|Scenario::|\.key\(|\.mouse\(", x)]
            clock_seed_lines = [{"line": start + i + 1, "text": x.strip()[:300]} for i, x in enumerate(body) if re.search(r"clock|seed|random|Instant::|SystemTime|Motion::|\.tick\(|time\s*=|set_time", x, re.IGNORECASE)]
            helper_clock_seed_lines = [{"function": helper_name, "line": i + 1, "text": x.strip()[:300]} for helper_name in helper_names for i, x in enumerate(lines[function_defs[helper_name][0]:function_defs[helper_name][1] + 1], function_defs[helper_name][0]) if re.search(r"clock|seed|random|Instant::|SystemTime|Motion::|\.tick\(|time\s*=|set_time", x, re.IGNORECASE)]
            collection_size_lines = [{"line": start + i + 1, "text": x.strip()[:300]} for i, x in enumerate(body) if re.search(r"for\s+\w+\s+in|0\.\.|vec!|with_capacity|repeat\(|\b(?:items|rows|cols|width|height|count|size)\b", x)]
            helper_collection_size_lines = [{"function": helper_name, "line": i + 1, "text": x.strip()[:300]} for helper_name in helper_names for i, x in enumerate(lines[function_defs[helper_name][0]:function_defs[helper_name][1] + 1], function_defs[helper_name][0]) if re.search(r"for\s+\w+\s+in|0\.\.|vec!|with_capacity|repeat\(|\b(?:items|rows|cols|width|height|count|size)\b", x)]
            option_lines = [{"line": start + i + 1, "text": x.strip()[:300]} for i, x in enumerate(body) if re.search(r"PERF_|iters\(|big\(|ColorLevel::|Theme::|Motion::|with_[a-z_]+|Options|mode\s*=", x)]
            helper_option_lines = [{"function": helper_name, "line": i + 1, "text": x.strip()[:300]} for helper_name in helper_names for i, x in enumerate(lines[function_defs[helper_name][0]:function_defs[helper_name][1] + 1], function_defs[helper_name][0]) if re.search(r"PERF_|iters\(|big\(|ColorLevel::|Theme::|Motion::|with_[a-z_]+|Options|mode\s*=", x)]
            assertion_lines = [{"line": start + i + 1, "text": x.strip()[:300]} for i, x in enumerate(body) if re.search(r"assert|check_ratio|PERF_|report_to|baseline", x)]
            helper_calls = [name for name in ("bench", "bench_sampled", "measure_once", "check_ratio", "report_to", "samples") if re.search(r"\b" + name + r"\s*\(", search_code)]
            helper_assertion_lines = [{"function": helper_name, "line": i + 1, "text": x.strip()[:300]} for helper_name in helper_names for i, x in enumerate(lines[function_defs[helper_name][0]:function_defs[helper_name][1] + 1], function_defs[helper_name][0]) if re.search(r"assert|check_ratio|PERF_|report_to|baseline", x)]
            scenario_source = None
            if "Scenario::ALL" in search_code:
                scenario_path = "crates/holla-sim/src/scenario.rs"
                scenario_raw = git_bytes(OLD, scenario_path)
                scenario_lines = scenario_raw.decode().splitlines(keepends=True)
                scenario_start = next(i for i, x in enumerate(scenario_lines) if re.search(r"pub const ALL:\s*\[Scenario;", x))
                scenario_end = next(i for i in range(scenario_start, len(scenario_lines)) if "];" in scenario_lines[i])
                scenario_span = "".join(scenario_lines[scenario_start:scenario_end + 1]).encode()
                scenario_source = {"path": scenario_path, "commit": OLD, "sha256": sha(scenario_raw), "line_start": scenario_start + 1, "line_end": scenario_end + 1, "span_sha256": sha(scenario_span), "scenario_count": 34}
            if "samples" in helper_calls:
                const_values = {}
                for constant in configuration_constants:
                    match = re.search(r"const\s+([A-Z_]+):[^=]*=\s*(\d+)", constant["text"])
                    if match: const_values[match.group(1)] = match.group(2)
                measurement_scope = "Local samples helper warms " + const_values.get("WARMUPS", "unresolved") + " times, records " + const_values.get("SAMPLES", "unresolved") + " samples, and repeats " + const_values.get("ITERATIONS", "unresolved") + " operations per sample using measure_once; reports retain raw totals."
            elif "bench_sampled" in helper_calls or "bench" in helper_calls:
                measurement_scope = "Entry point calls the named repeat helper; warmup, iteration normalization, repetition count, and median follow the bound historical helper source."
            elif "measure_once" in helper_calls:
                measurement_scope = "Entry point calls one measurement helper; it does not establish a repeated median."
            elif "check_ratio" in helper_calls:
                measurement_scope = "Entry point compares the recorded values with its explicit ratio and strictness arguments."
            else:
                measurement_scope = "No named repeat helper was found in this entry point; inspect the bound body lines before adopting a replacement workload."
            current_paths = perf_current_paths(name, path)
            baseline = None
            if "holla-ui/tests/perf.rs" in path: baseline = "crates/holla-ui/tests/baselines/app.txt"
            elif "jackin-preview-app/tests/perf.rs" in path: baseline = "crates/jackin-preview-app/tests/perf_baseline.txt"
            elif "showcase-ui/tests/perf.rs" in path: baseline = "crates/showcase-ui/tests/perf_baseline.txt"
            elif "tablepro-ui/tests/perf.rs" in path: baseline = "crates/tablepro-ui/tests/perf_baseline.txt"
            elif path.endswith("termrock-test-support/tests/perf.rs"): baseline = "crates/termrock-test-support/tests/perf_baseline.txt"
            uses_baseline = "report_to" in helper_calls or "baseline" in name.lower() or "perf_baseline" in search_code
            bref, baseline_candidate = None, None
            if baseline and uses_baseline:
                try: bref = {"path": baseline, "commit": OLD, "sha256": sha(git_bytes(OLD, baseline)), "status": "historical source consumed or inspected; not measured here"}
                except Exception: bref = {"path": baseline, "commit": OLD, "sha256": None, "status": "not found at historical path"}
            elif baseline:
                try: baseline_candidate = {"path": baseline, "commit": OLD, "sha256": sha(git_bytes(OLD, baseline)), "status": "path candidate only; this entry point does not inspect or report to it"}
                except Exception: baseline_candidate = {"path": baseline, "commit": OLD, "sha256": None, "status": "not found at historical path"}
            typ = "harness_integrity_only" if name == "jackin_perf_baseline_has_required_rows_and_no_deleted_clone" else "baseline_measurement_entrypoint" if "baseline" in name or name == "debug_and_release_alloc_counts_match" else "measured_product_workload"
            result.append({
                "workload_id": OLD + ":" + path + ":" + str(start + 1) + ":" + name,
                "historical_test_surface": {"path": path, "commit": OLD, "sha256": sha(raw), "test_attribute_line": attr + 1, "line_start": start + 1, "line_end": end + 1, "span_sha256": sha(span_raw), "entrypoint": name},
                "historical_workload_kind": typ,
                "dimension_tokens": dims,
                "numeric_and_size_literals": nums,
                "event_or_scenario_lines": events,
                "helper_event_or_scenario_lines": helper_events,
                "clock_and_seed_lines": clock_seed_lines,
                "helper_clock_and_seed_lines": helper_clock_seed_lines,
                "collection_size_lines": collection_size_lines,
                "helper_collection_size_lines": helper_collection_size_lines,
                "options_and_configuration_lines": option_lines,
                "helper_options_and_configuration_lines": helper_option_lines,
                "configuration_constant_lines": configuration_constants,
                "assertion_threshold_and_claim_lines": assertion_lines,
                "helper_assertion_threshold_and_claim_lines": helper_assertion_lines,
                "measurement_and_claim_lines": facts,
                "helper_measurement_and_claim_lines": helper_claims,
                "local_helper_source_bindings": helper_bindings,
                "external_scenario_source_binding": scenario_source,
                "measurement_helper_calls": helper_calls,
                "measurement_scope": measurement_scope,
                "seed_and_clock_assessment": "Source lines are listed above; absence means this entry point declares no explicit deterministic clock or seed.",
                "baseline": bref,
                "baseline_candidate": baseline_candidate,
                "historical_entrypoint_summary": name.replace("_", " "),
                "dimensions": {"dimension_tokens": dims, "numeric_and_size_literals": nums, "entrypoint_source_lines": facts, "helper_source_lines": helper_claims, "status": "source inventory; no workload ran"},
                "collection_sizes": {"entrypoint_source_lines": collection_size_lines, "helper_source_lines": helper_collection_size_lines, "configuration_constants": configuration_constants, "status": "source inventory; values remain source text"},
                "options": {"entrypoint_source_lines": option_lines, "helper_source_lines": helper_option_lines, "configuration_constants": configuration_constants},
                "event_sequence": {"entrypoint_lines": events, "helper_lines": helper_events, "external_scenario_source": scenario_source},
                "clock_and_seed": {"entrypoint_lines": clock_seed_lines, "helper_lines": helper_clock_seed_lines, "assessment": "The source excerpt shows explicit values; an empty list means no explicit clock or seed declaration was found in the traced bodies."},
                "repeat_and_measurement_scope": {"helper_calls": helper_calls, "measurement_scope": measurement_scope, "helper_bindings": helper_bindings, "status": "NOT VERIFIED"},
                "old_thresholds_and_claims": {"entrypoint_lines": assertion_lines, "helper_lines": helper_assertion_lines, "baseline": bref, "baseline_candidate": baseline_candidate, "shared_measurement_model": "performance_measurement_model", "status": "NOT VERIFIED"},
                "current_production_equivalent": {"paths": current_paths, "source_bindings": [current_source_binding(x) for x in current_paths], "owner_tasks": tasks[1:] or tasks, "owner_task_ids": tasks[1:] or tasks, "performance_owner_task": "T33", "performance_owner_task_ids": ["T33"], "status": "source inventory only; no current measurement"},
                "source_disposition": "ADAPT",
                "source_disposition_reason": "T37 retired the old test surface. Preserve its workload as a T33 requirement; old measurements are not current evidence.",
                "old_measurement_status": "NOT VERIFIED",
                "current_measurement_status": "NOT VERIFIED",
                "proposed_case_status": "PROPOSED_NOT_APPROVED_T38",
                "selected_cases": 0,
            })
    if len(result) != 71: raise ValueError("expected 71 perf entrypoints, found " + str(len(result)))
    holla = next((x for x in result if x["historical_test_surface"]["entrypoint"] == "production_workload_measurements"), None)
    if holla:
        holla["subworkloads"] = {"scenario_frames": ["FirstUse", "RustDirty", "MonorepoRoot", "MonorepoChild", "DockerCleanup", "DiskCleanup", "UpgradePlan", "ActivitiesMulti", "RemoteHost", "LaunchFailure", "HardCases", "ParityDiscovery", "ParityHistory", "ParityFiles", "ParityBrowser", "ParityGitCurrent", "ParityGitBatch", "ParityTaskSources", "ParityCargo", "ParityDocker", "ParityBrewServices", "ParityGradle", "ParityIdea", "ParityUpgradeManagers", "ParityExecutor", "ParityTaskInput", "ParityCustomActions", "ParityDiskScan", "ParityDiskNavigation", "ParityInsights", "ParityDeleteSafety", "ParityCleanupResults", "ParityPlatforms", "ParityPlatformsLinux"], "frame": "120x40; fixture sets Motion::Paused and time 4000", "query_edit_pairs": ["HardCases: x then Backspace"], "plan_navigation": ["DockerCleanup: Enter, Tab, Down, Up"], "source_helpers": ["fixture", "samples", "frame", "query_edits", "plan_navigation"]}
    return result, sources


def main():
    component, foundation, routes = api_data()
    rows, lane_ids, lane_counts, allow_bytes, catalog_bytes = build_coverage(component, foundation, routes)
    migrations = migration_items(component, foundation, routes)
    applications = application_inventory(routes)
    perf, perf_sources = workloads()
    unresolved = [x["row_id"] for x in rows if x["disposition"] == "UNRESOLVED"]
    sources = []
    for role, path in [
        ("launch README", "docs/implementation/pr38-final/README.md"),
        ("complete launch goal", "docs/implementation/pr38-final/GOAL.md"),
        ("preserved API rules", "docs/implementation/pr38-final/API-RULES.md"),
        ("preserved architecture contract", "docs/implementation/pr38-final/ARCHITECTURE.md"),
        ("preserved component matrix", "docs/implementation/pr38-final/COMPONENT-MATRIX.md"),
        ("preserved validation policy", "docs/implementation/pr38-final/VALIDATION.md"),
        ("retained migration contract", "docs/contracts/migration.md"),
        ("retained architecture record", "docs/architecture/termrock-migration-record.md"),
        ("retained application migration record", "docs/architecture/termrock-migration-matrix.md"),
        ("task ownership", "docs/implementation/pr38-final/TASKS.md"),
        ("review SQL correction", "docs/implementation/pr38-final/REVIEW.md"),
        ("T37 test API retirement", "docs/implementation/pr38-final/results/t37-foundations/deferred-testing-surfaces.json"),
        ("T37 source removal status", "docs/implementation/pr38-final/results/t37-foundations/source-removal.json"),
        ("T37 viewport probe retirement", "docs/implementation/pr38-final/results/t37-components/approved-testing-api-retirement.jsonl"),
        ("T37 selection helper retirement", "docs/implementation/pr38-final/results/t37-components/approved-selection-binding-retirement.json"),
        ("T37 test cleanup record", "docs/implementation/pr38-final/results/t37-tools/AGENT-RUNBOOK.json"),
        ("T00 source state", "docs/implementation/pr38-final/results/t00-source/source-state.json"),
        ("T00 immutable artifact index", "docs/implementation/pr38-final/results/t00-source/artifact-index.json"),
        ("T00 finding dispositions", "docs/implementation/pr38-final/results/t00-source/finding-dispositions.json"),
        ("T00 source finding deltas", "docs/implementation/pr38-final/results/t00-source/finding-deltas-e08-to-1d6.json"),
        ("T37 immutable index reconciliation", "docs/implementation/pr38-final/results/t37-final-integration/t00-immutable-index-reconciliation.json"),
        ("current Showcase route source", "crates/showcase-ui/src/app.rs"),
        ("current SQL comparator", "crates/tablepro-sql/src/sql.rs"),
        ("current domain sort caller", "crates/tablepro-domain/src/domain.rs"),
        ("historical Showcase route comparison", "docs/current-state.md"),
        ("historical TablePro NULL-sort claim", "docs/plan.md"),
    ]:
        sources.append({"path": path, "commit": INPUT, "sha256": sha(git_bytes(INPUT, path)), "role": role})
    sources.extend({"path": x["current_sources"][0]["path"], "commit": INPUT, "sha256": x["current_sources"][0]["sha256"], "role": "current application source: " + x["application"]} for x in applications)
    sources += [
        {"path": ALLOW_OLD, "commit": OLD, "sha256": sha(allow_bytes), "role": "original allowlist"},
        {"path": ALLOW_NEW, "commit": PRESERVED, "sha256": sha(allow_bytes), "role": "byte-identical preserved allowlist"},
        {"path": CATALOG_OLD, "commit": OLD, "sha256": sha(catalog_bytes), "role": "historical bounded source catalogue"},
        {"path": "crates/termrock-test-support/src/perf.rs", "commit": OLD, "sha256": sha(git_bytes(OLD, "crates/termrock-test-support/src/perf.rs")), "role": "historical measurement helper; not executed"},
    ] + perf_sources
    siblings = []
    for role, path, data in [("component api", COMPONENT_PATH, component), ("foundation api", FOUNDATION_PATH, foundation), ("application routes", ROUTE_PATH, routes)]:
        siblings.append({"path": str(path), "sha256": path_sha(path), "role": role, "status": "SUPPLEMENTAL_NOT_ACCEPTED", "schema": data.get("schema"), "input_commit": data.get("input_commit"), "list_counts": {k: len(v) for k, v in data.items() if isinstance(v, list)}})
    sources += siblings
    sources.append({"path": T01_REACH_DECISION_PATH, "commit": T01_REACH_DECISION_COMMIT, "sha256": T01_REACH_DECISION_SHA256, "role": "independently reviewed T01 shell reach retirement decision and receipt"})
    lane_order = ["SC1", "SC2", "SC3", "review"]
    lane_counts = {x: lane_counts[x] for x in lane_order}
    jsonl_path = OUT / "historical-coverage.jsonl"
    disposition_counts = {name: collections.Counter(x["disposition"] for x in rows).get(name, 0) for name in ("KEEP", "ADAPT", "RETIRE", "UNRESOLVED")}
    category_counts = {name: collections.Counter(x["category"] for x in rows).get(name, 0) for name in ("alias", "value", "behavior", "removed_test_api", "product_path")}
    manifest = {"path": "docs/implementation/pr38-final/results/t01-history-requirements/historical-coverage.jsonl", "sha256": path_sha(jsonl_path), "schema": "termrock.pr38.t01.historical-coverage-row.v1", "row_count": len(rows), "hash_method": "SHA-256 over exact UTF-8 JSONL bytes, including LF separators.", "lane_counts": lane_counts, "lane_order": lane_order, "lane_row_ids": {x: lane_ids[x] for x in lane_order}, "disposition_counts": disposition_counts, "category_counts": category_counts, "source_row_order": "Original allowlist physical line order; comments and blanks have no records.", "join": "Join by immutable row_id (e08 commit + original path + physical line); retain duplicates and never join by symbol alone.", "source_identity": {"commit": OLD, "path": ALLOW_OLD, "sha256": sha(allow_bytes)}, "preserved_identity": {"commit": PRESERVED, "path": ALLOW_NEW, "sha256": sha(allow_bytes), "byte_identical": True}, "role": "Requirement inventory only; not a test suite, result, or behavior proof."}
    shell_decision = json.loads(git_bytes(T01_REACH_DECISION_COMMIT, T01_REACH_DECISION_PATH))
    open_requirements = [
        {"id": "HIST-COVERAGE-ALL", "block_final": True, "owner_task": "T01 and row owners", "reason": "All 4,511 entries are mapped inventory; behavior requires approved T38 cases and later execution.", "row_count": len(rows), "evidence": "NOT VERIFIED"},
        {"id": "T33-PERFORMANCE-MEASUREMENTS", "block_final": True, "owner_task": "T33", "reason": "No historical or current workload result is verified here.", "workload_count": len(perf), "evidence": "NOT VERIFIED"},
        {"id": "MIGRATION-CASES", "block_final": True, "owner_task": "per item", "reason": "Migration rows and current source maps have proposed cases only.", "item_count": len(migrations), "evidence": "NOT VERIFIED"},
        {"id": "FOUR-APPLICATION-MIGRATION", "block_final": True, "owner_task": "T22/T27/T28/T29", "reason": "Each application source is mapped; route and behavior cases remain proposed.", "application_count": len(applications), "evidence": "NOT VERIFIED"},
    ]
    if unresolved:
        open_requirements.append({"id": "HIST-COVERAGE-UNRESOLVED", "block_final": True, "owner_task": "each row owner_task", "reason": "Current source members not located need an explicit owner decision.", "row_count": len(unresolved), "row_ids": unresolved, "evidence": "NOT VERIFIED"})
    for obligation in shell_decision["future_obligations"]:
        open_requirements.append({"id": obligation["requirement_id"], "block_final": True, "owner_task_ids": obligation["owner_task_ids"], "source": {"path": T01_REACH_DECISION_PATH, "commit": T01_REACH_DECISION_COMMIT, "sha256": T01_REACH_DECISION_SHA256}, "reason": obligation["obligation"], "status": "PROPOSED_NOT_APPROVED_T38", "case_status": "PROPOSED_NOT_APPROVED_T38", "evidence": "NOT VERIFIED"})
    fragment = {
        "schema": "termrock.pr38.t01.fragment.v1",
        "task_id": "T01",
        "fragment_id": "history-requirements",
        "input_commit": INPUT,
        "input_tree": subprocess.run(["git", "rev-parse", INPUT + "^{tree}"], cwd=ROOT, check=True, text=True, stdout=subprocess.PIPE).stdout.strip(),
        "scope": {"owned_path": "docs/implementation/pr38-final/results/t01-history-requirements/", "historical_coverage_row_count": len(rows), "historical_migration_item_count": len(migrations), "historical_workload_count": len(perf), "current_application_count": len(applications), "no_product_source_edits": True, "no_legacy_execution": True, "no_benchmark_execution": True, "selected_cases": 0, "product_behavior": "NOT VERIFIED"},
        "sources": sources,
        "historical_coverage_rows": manifest,
        "historical_migration_items": migrations,
        "current_applications": applications,
        "workloads": perf,
        "performance_measurement_model": {**performance_measurement_model(), "case_status": "PROPOSED_NOT_APPROVED_T38"},
        "requirements": {"all_rows_have_disposition_source_task_and_case": all(x.get("disposition") and x.get("current_mapping") and x.get("owner_task") and x.get("owner_task_ids") and x.get("proposed_case") and x["proposed_case"].get("owner_task_ids") for x in rows), "all_rows_have_source_binding_or_approved_retirement": all(x["current_mapping"].get("source_binding_verified") and x["current_mapping"].get("source") and (x["current_mapping"].get("source").get("sha256") or x["current_mapping"].get("source").get("decision_id")) for x in rows), "all_retirements_have_decision_conflict_replacement_source": all(not x["current_mapping"].get("retirement") or all(x["current_mapping"]["retirement"].get(k) for k in ("decision_id", "conflict", "replacement", "record_sha256")) for x in rows), "all_cases_proposed": all(x["case_status"] == "PROPOSED_NOT_APPROVED_T38" for x in rows), "all_evidence_not_verified": all(x["evidence"] == "NOT VERIFIED" for x in rows), "all_workloads_owner_T33": all(x["current_production_equivalent"]["performance_owner_task"] == "T33" and x["current_production_equivalent"]["performance_owner_task_ids"] == ["T33"] for x in perf), "all_migrations_have_owner_and_case_task_ids": all(x.get("owner_task_ids") and x.get("required_case", {}).get("owner_task_ids") for x in migrations), "no_new_wid_allocations": True, "current_route_artifact": {"path": str(ROUTE_PATH), "sha256": path_sha(ROUTE_PATH), "route_count": len(routes.get("routes", [])), "join": "reference_identity + reference_index"}, "current_route_correction": "Current input has 54 routes: 24 Live, one Selection, 29 Placeholder; five Placeholder entries retain page objects and 24 paint placeholder labels.", "current_null_sort_correction": "cmp_values_nulls_last keeps NULL greater than non-NULL in both directions; only non-NULL ordering reverses. Source inventory is not behavior proof.", "alias_rows": sum(x["category"] == "alias" for x in rows), "alias_classification": "A row is alias only when its current public declaration is a source-bound public type alias; aliases remain distinct by row_id."},
        "open_requirements": open_requirements,
        "commands": [
            {"argv": ["python3", "--version"], "cwd": str(ROOT), "tool": "Python 3.15.1", "exit_code": 0, "result": "Interpreter version recorded."},
            {"argv": ["python3", "docs/implementation/pr38-final/results/t01-history-requirements/build_fragment.py"], "cwd": str(ROOT), "tool": "Python 3.15.1", "exit_code": 0, "selected_tests": 0, "result": "Generated 4,511 immutable row mappings, migration crosswalks, and 71 workload records; JSONL SHA-256 " + manifest["sha256"] + "."},
            {"argv": ["python3", "-c", "import json,pathlib; p=pathlib.Path('docs/implementation/pr38-final/results/t01-history-requirements/historical-coverage.jsonl'); rs=[json.loads(s) for s in p.read_text().splitlines()]; assert len(rs)==4511 and len({x['row_id'] for x in rs})==4511; assert all(x['schema']=='termrock.pr38.t01.historical-coverage-row.v1' and x['evidence']=='NOT VERIFIED' for x in rs); print('rows=4511 unique=4511 schema=ok evidence=NOT_VERIFIED')"], "cwd": str(ROOT), "tool": "Python 3.15.1", "exit_code": 0, "selected_tests": 0, "result": "Structural JSONL parse and inventory assertions only."},
            {"argv": ["python3", "docs/implementation/pr38-final/results/t01-history-requirements/validate_fragment.py"], "cwd": str(ROOT), "tool": "Python 3.15.1", "exit_code": 0, "selected_tests": 0, "result": "Source-span, row-identity, owner, route, retirement, workload, and NOT VERIFIED assertions pass; no product behavior is proven."},
        ],
        "limits": ["Static source mappings do not prove compiler visibility, runtime behavior, performance, route output, or semantic parity.", "Historical lanes and test names are not current test results.", "T33 owns replacement workload design; no benchmark or new counter ran.", "Route and NULL-sort corrections appear only in this fragment; historical records remain unchanged.", "T37 approvals retire test helpers only; related product behavior remains open."],
        "counts": {"historical_coverage_rows": len(rows), "lane_counts": lane_counts, "dispositions": disposition_counts, "categories": category_counts, "mapping_methods": dict(collections.Counter(x["current_mapping"]["method"] for x in rows)), "historical_migration_items": len(migrations), "workloads": len(perf), "unresolved_rows": len(unresolved), "selected_cases": 0},
    }
    fragment["historical_coverage_rows"]["sha256"] = path_sha(jsonl_path)
    (OUT / "fragment.json").write_text(json.dumps(fragment, ensure_ascii=False, sort_keys=True, indent=2) + "\n")
    (OUT / "README.md").write_text("\n".join([
        "# T01 historical requirements", "",
        "Input commit: 542349e2f70ec9883d743594f906ac6c4e96ba5a.", "",
        "The JSONL retains 4,511 rows, including duplicates and original lanes.",
        "Stable IDs combine the source commit, original path, and physical line.",
        "The preserved 1d6 allowlist is byte-identical to the e08 source file.",
        "Source allowlist SHA-256: " + sha(allow_bytes) + ".",
        "The JSONL is hashed as exact UTF-8 bytes with LF separators.", "",
        "Historical JSONL SHA-256: " + manifest["sha256"] + ".",
        "Lane counts: SC1 100; SC2 1,008; SC3 247; review 3,156.",
        "Disposition counts: " + ", ".join(k + " " + str(v) for k, v in sorted(collections.Counter(x["disposition"] for x in rows).items())) + ".",
        "Unresolved row count: " + str(len(unresolved)) + ".",
        "Rows and migration items carry current source pointers and named tasks.",
        "Source bindings do not prove behavior, compiler visibility, or test results.", "",
        "The fragment maps " + str(len(migrations)) + " migration items, 54 routes, and four application owners.",
        "The performance inventory retains 71 historical entrypoints for T33.",
        "Historical and current measurements remain NOT VERIFIED.",
        "The current route census records 24 Live, one Selection, and 29 Placeholder entries.",
        "Five Placeholder entries retain pages; 24 paint labels. Route behavior remains NOT VERIFIED.",
        "Current SQL source keeps NULL last in both sort directions; behavior remains NOT VERIFIED.",
        "The shell reach decision retires 11 helper rows; four route behavior obligations remain proposed.",
        "Historical route and NULL-sort text remains unchanged.",
        "The three sibling artifacts are supplemental and are not accepted here.", "",
        "Run the structural inventory producer:",
        "`python3 docs/implementation/pr38-final/results/t01-history-requirements/build_fragment.py`", "",
        "No legacy command, benchmark, functional test, or product code ran.",
        "Selected cases: zero. All proposed cases await T38 design and review.",
        "This fragment awaits independent Sol high review.", "",
    ]))


if __name__ == "__main__":
    main()
