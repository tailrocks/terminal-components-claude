import collections
import functools
import hashlib
import json
import pathlib
import re
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[5]
OUT = pathlib.Path(__file__).resolve().parent
INPUT = "542349e2f70ec9883d743594f906ac6c4e96ba5a"
OLD = "e08ac57d10ae54c87337eae0be08b1f5e8f4bda4"
PRESERVED = "1d6ee92adf6ccca38eeee64b1371cba1497e8fcd"
ALLOW_OLD = "crates/showcase-ui/src/coverage/allowlist.txt"
ALLOW_NEW = "crates/showcase-ui/tests/coverage/allowlist.txt"


def digest(data):
    return hashlib.sha256(data).hexdigest()


@functools.lru_cache(maxsize=None)
def git_bytes(commit, path):
    return subprocess.run(["git", "show", commit + ":" + path], cwd=ROOT, check=True, stdout=subprocess.PIPE).stdout


@functools.lru_cache(maxsize=None)
def git_object(commit, path):
    return subprocess.run(["git", "rev-parse", commit + ":" + path], cwd=ROOT, check=True, text=True, stdout=subprocess.PIPE).stdout.strip()


def check(condition, message):
    if not condition:
        raise ValueError(message)


old_bytes = git_bytes(OLD, ALLOW_OLD)
new_bytes = git_bytes(PRESERVED, ALLOW_NEW)
check(old_bytes == new_bytes, "allowlist preservation mismatch")
source_lines = old_bytes.splitlines(keepends=True)
catalog_path = "crates/showcase-ui/src/coverage/catalog_data.rs"
catalog_bytes = git_bytes(OLD, catalog_path)
catalog_text = catalog_bytes.decode()
catalog_lines = catalog_text.splitlines(keepends=True)
catalog_pattern = re.compile(r'Capability::(ty|variant|function|builder|token)\s*\(\s*"([^"]+)"\s*,?\s*\)', re.DOTALL)
catalog_index = collections.defaultdict(list)
for match in catalog_pattern.finditer(catalog_text):
    line_start = catalog_text.count("\n", 0, match.start()) + 1
    line_end = catalog_text.count("\n", 0, match.end()) + 1
    catalog_index[match.group(2)].append({"kind": match.group(1), "line_start": line_start, "line_end": line_end, "span_sha256": digest(match.group(0).encode())})
jsonl_path = OUT / "historical-coverage.jsonl"
jsonl_bytes = jsonl_path.read_bytes()
rows = [json.loads(line) for line in jsonl_bytes.decode().splitlines()]
check(len(rows) == 4511, "historical row count mismatch")
check(len({row["row_id"] for row in rows}) == len(rows), "duplicate row identity")
lane_counts = collections.Counter(row["lane"] for row in rows)
check(lane_counts == {"SC1": 100, "SC2": 1008, "SC3": 247, "review": 3156}, "source lane counts mismatch")
allowed_categories = {"alias", "value", "behavior", "removed_test_api", "product_path"}
allowed_dispositions = {"KEEP", "ADAPT", "RETIRE", "UNRESOLVED"}
for row in rows:
    source = row["historical_source"]
    line_number = source["line"]
    raw = source_lines[line_number - 1]
    text = raw.decode().rstrip("\r\n")
    check(row["row_id"] == OLD + ":" + ALLOW_OLD + ":" + str(line_number), "row ID mismatch")
    check(row["historical_claim"]["raw_text"] == text, "historical text mismatch")
    check(source["raw_line_sha256"] == digest(raw), "historical line hash mismatch")
    check(source["text_sha256"] == digest(text.encode()), "historical text hash mismatch")
    check(source["file_sha256"] == digest(old_bytes), "historical file hash mismatch")
    check(source["preserved_in"]["commit"] == PRESERVED and source["preserved_in"]["raw_line_sha256"] == digest(new_bytes.splitlines(keepends=True)[line_number - 1]), "preserved row mismatch")
    check(row["category"] in allowed_categories and row["disposition"] in allowed_dispositions, "invalid category or disposition")
    check(row["case_status"] == "PROPOSED_NOT_APPROVED_T38" and row["evidence"] == "NOT VERIFIED" and row["selected_behavior_cases"] == 0, "case or evidence status mismatch")
    check(row["owner_task_ids"] and row["proposed_case"]["owner_task_ids"], "row task identity missing")
    inventory = row["historical_claim"]["source_inventory"]
    check(inventory["commit"] == OLD and inventory["path"] == catalog_path and inventory["file_sha256"] == digest(catalog_bytes), "historical catalogue identity mismatch")
    check(any(item["line_start"] == inventory["line_start"] and item["line_end"] == inventory["line_end"] and item["span_sha256"] == inventory["source_span_sha256"] and item["kind"] == inventory["catalog_kind"] for item in catalog_index[row["historical_claim"]["symbol"]]), "historical catalogue row mismatch")
    mapping = row["current_mapping"]
    check(row["component_wid"] is None or row["component_wid"] in {"W%02d" % i for i in range(1, 46)}, "historical crosswalk uses new WID")
    if mapping.get("retirement"):
        retirement = mapping["retirement"]
        check(all(retirement.get(key) for key in ("decision_id", "conflict", "replacement", "record_sha256", "record_path")), "retirement decision evidence missing")
        record_commit = retirement.get("record_commit", INPUT)
        check(digest(git_bytes(record_commit, retirement["record_path"])) == retirement["record_sha256"], "retirement record hash mismatch")
        check(retirement.get("input_source") and retirement.get("exact_public_path") == retirement["input_source"].get("qualified_symbol"), "retirement source API identity missing")
        current_absence = retirement.get("candidate_current_absence", {})
        check(current_absence.get("commit") == INPUT and current_absence.get("qualified_symbol_absent_from_source"), "retirement current-source absence mismatch")
        if retirement.get("decision_id") == "T01-HISTORY-SHELL-REACH-RETIRE-01":
            check(retirement.get("receipt", {}).get("status") == "ACCEPT_SCOPED_RECEIPT", "shell reach decision receipt missing")
            check(retirement.get("decision_artifact", {}).get("sha256") == "4ea12ecd0b6dcea45a7acf0bc79313356929e5322518236aaebc68674e9d720b", "shell reach decision artifact hash mismatch")
            check(retirement.get("scope_record_id") == retirement.get("source_record_id"), "shell reach scope record join mismatch")
            check(retirement.get("decision_public_scope_record", {}).get("path") == retirement["exact_public_path"], "shell reach public path scope mismatch")
            check(len(retirement.get("future_obligations", [])) == 4 and retirement.get("future_obligation_owner_task_ids"), "shell reach future obligations missing")
            check(current_absence.get("reach_module_source_absent") and current_absence.get("shell_root_reach_declaration_absent"), "shell reach current removal evidence mismatch")
        else:
            bindings = retirement.get("artifact_bindings", [])
            check(bindings, "retirement artifact bindings missing")
            for binding in bindings:
                check(digest(git_bytes(binding["commit"], binding["path"])) == binding["sha256"], "retirement approval artifact hash mismatch")
            historical = retirement.get("historical_source_presence")
            check(historical and historical.get("qualified_symbol_present_in_source"), "retired symbol lacks historical source presence")
            check(retirement.get("change_records") and retirement.get("source_record_id"), "retirement source change record missing")
            check(retirement.get("decision_status") == "APPROVED" and retirement.get("source_binding_verified"), "T37 retirement decision is not source-bound")
    else:
        current = mapping["source"]
        check(current.get("commit") == INPUT and digest(git_bytes(INPUT, current["path"])) == current["sha256"], "current source hash mismatch")
    check(mapping["source_binding_verified"] and mapping["source"], "current mapping lacks source binding")

fragment_path = OUT / "fragment.json"
fragment_bytes = fragment_path.read_bytes()
fragment = json.loads(fragment_bytes)
check(fragment["schema"] == "termrock.pr38.t01.fragment.v1" and fragment["input_commit"] == INPUT, "fragment identity mismatch")
check(fragment["historical_coverage_rows"]["sha256"] == digest(jsonl_bytes), "JSONL manifest hash mismatch")
check(fragment["historical_coverage_rows"]["row_count"] == len(rows), "manifest row count mismatch")
check(fragment["historical_coverage_rows"]["lane_counts"] == dict(lane_counts), "manifest lane count mismatch")
check(fragment["counts"]["historical_migration_items"] == len(fragment["historical_migration_items"]), "migration item count mismatch")
check(all(item.get("current_sources") and item.get("owner_task_ids") and item.get("required_case", {}).get("owner_task_ids") for item in fragment["historical_migration_items"]), "migration source or task mapping missing")
for item in fragment["historical_migration_items"]:
    historical = item["historical_source"]
    raw = git_bytes(historical["commit"], historical["path"])
    check(digest(raw) == historical["sha256"], "migration historical source hash mismatch")
    if historical.get("line_start") and historical.get("line_end"):
        check(historical["line_end"] <= len(raw.splitlines()), "migration historical line span out of bounds")
        span = b"".join(raw.splitlines(keepends=True)[historical["line_start"] - 1:historical["line_end"]])
        check(digest(span) == historical["span_sha256"], "migration historical source span hash mismatch")
        check(historical["raw_text"] == span.decode("utf-8").rstrip("\r\n"), "migration source text is not the exact historical span")
        check(historical["raw_text_sha256"] == digest(historical["raw_text"].encode()), "migration historical source text hash mismatch")
    for current in item["current_sources"]:
        path = current.get("path")
        source_hash = current.get("sha256")
        if not path or not source_hash:
            continue
        if pathlib.Path(path).is_absolute():
            check(current.get("status") == "SUPPLEMENTAL_NOT_ACCEPTED" and current.get("evidence") == "NOT VERIFIED", "absolute artifact treated as current source binding")
            check(len(source_hash) == 64 and current.get("hash_role"), "supplemental artifact hash lacks scope")
        else:
            commit = current.get("commit", INPUT)
            check(digest(git_bytes(commit, path)) == source_hash, "migration current source hash mismatch")
for source in fragment["sources"]:
    if pathlib.Path(source["path"]).is_absolute():
        check(source.get("status") == "SUPPLEMENTAL_NOT_ACCEPTED", "absolute source is not marked supplemental")
        check(len(source["sha256"]) == 64, "supplemental artifact hash is invalid")
route_items = [item for item in fragment["historical_migration_items"] if item.get("record_kind") == "current_showcase_route_crosswalk"]
check(len(route_items) == 54 and len({item["historical_reference"]["reference_identity"] for item in route_items}) == 54, "showcase route count or identity mismatch")
check(len({item["historical_reference"]["reference_index"] for item in route_items}) == 54, "showcase route index mismatch")
for item in route_items:
    historical = item["historical_source"]
    raw = git_bytes(historical["commit"], historical["path"])
    lines = raw.splitlines(keepends=True)
    span = b"".join(lines[historical["line_start"] - 1:historical["line_end"]])
    reference = item["historical_reference"]
    check(digest(raw) == historical["sha256"] == reference["source_sha256"], "PR26 source file hash mismatch")
    check(digest(span) == historical["span_sha256"], "PR26 route span mismatch")
    check(historical["raw_text"] == span.decode("utf-8").rstrip("\r\n") and historical["raw_text_sha256"] == digest(historical["raw_text"].encode()), "PR26 route text mismatch")
    page_id = reference["reference_identity"].split("::")[-1]
    check("PageId::" + page_id in historical["raw_text"], "PR26 route identity/source span mismatch")
    factory = reference["factory_span"]
    factory_raw = b"".join(lines[factory["line_start"] - 1:factory["line_end"]])
    check(digest(factory_raw) == factory["span_sha256"], "PR26 factory span mismatch")

check({row["application"] for row in fragment["current_applications"]} == {"Holla", "Jackin", "Showcase", "TablePro"}, "application owner census mismatch")
for app in fragment["current_applications"]:
    current = app["current_sources"][0]
    check(digest(git_bytes(INPUT, current["path"])) == current["sha256"], "application source hash mismatch")
check(len(fragment["workloads"]) == 71, "historical workload count mismatch")
for workload in fragment["workloads"]:
    check(all(workload.get(key) is not None for key in ("dimensions", "collection_sizes", "options", "event_sequence", "clock_and_seed", "repeat_and_measurement_scope", "old_thresholds_and_claims", "current_production_equivalent")), "normalized workload definition is incomplete")
    history = workload["historical_test_surface"]
    raw = git_bytes(history["commit"], history["path"])
    lines = raw.splitlines(keepends=True)
    span = b"".join(lines[history["line_start"] - 1:history["line_end"]])
    check(digest(raw) == history["sha256"] and digest(span) == history["span_sha256"], "workload source span mismatch")
    check(workload["current_production_equivalent"]["performance_owner_task"] == "T33", "workload owner mismatch")
    check(workload["old_measurement_status"] == "NOT VERIFIED" and workload["current_measurement_status"] == "NOT VERIFIED" and workload["selected_cases"] == 0, "workload evidence status mismatch")
    check(workload["repeat_and_measurement_scope"]["status"] == "NOT VERIFIED" and workload["old_thresholds_and_claims"]["status"] == "NOT VERIFIED", "workload measurement status mismatch")
    for binding in workload["current_production_equivalent"]["source_bindings"]:
        if binding["kind"] == "source_file":
            check(digest(git_bytes(INPUT, binding["path"])) == binding["sha256"], "current workload source hash mismatch")
        else:
            check(git_object(INPUT, binding["path"]) == binding["git_tree_object"], "current workload tree mismatch")
check(fragment["requirements"]["all_rows_have_source_binding_or_approved_retirement"], "current row mapping incomplete")
check(fragment["requirements"]["all_retirements_have_decision_conflict_replacement_source"], "retirement evidence incomplete")
check(fragment["counts"]["unresolved_rows"] == 0, "unresolved historical source binding remains")
check(fragment["counts"]["categories"]["alias"] == 0 and set(fragment["counts"]["categories"]) == {"alias", "value", "behavior", "removed_test_api", "product_path"}, "historical category census mismatch")
check(fragment["counts"]["dispositions"]["UNRESOLVED"] == 0 and sum(fragment["counts"]["dispositions"].values()) == 4511, "historical disposition census mismatch")
shell_obligations = [x for x in fragment["open_requirements"] if x["id"].startswith("SHELL-REACH-")]
check({x["id"] for x in shell_obligations} == {"SHELL-REACH-RESIZE-RECOVERY", "SHELL-REACH-SCROLL-CONTENT", "SHELL-REACH-TRUNCATION-REVEAL", "SHELL-REACH-PRESENTATION-INPUT"}, "shell reach future obligations mismatch")
check(all(x["block_final"] and x["status"] == "PROPOSED_NOT_APPROVED_T38" and x["evidence"] == "NOT VERIFIED" and x["owner_task_ids"] for x in shell_obligations), "shell reach future obligation status mismatch")
check(fragment["scope"]["selected_cases"] == 0 and fragment["scope"]["product_behavior"] == "NOT VERIFIED", "verification scope mismatch")
print("rows=4511 unique=4511 lanes=4511 row_source_joins=ok preserved_identity=byte_identical")
print("migration_items=" + str(len(fragment["historical_migration_items"])) + " routes=54 applications=4 workloads=71 unresolved=" + str(fragment["counts"]["unresolved_rows"]))
print("selected_cases=0 behavior=NOT_VERIFIED structural_source_checks=ok")
print("jsonl_sha256=" + digest(jsonl_bytes))
print("fragment_sha256=" + digest(fragment_bytes))
