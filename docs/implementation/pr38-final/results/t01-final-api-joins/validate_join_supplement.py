#!/usr/bin/env python3
"""Validate source, identity, and pointer closure for the T01 join supplement."""

from __future__ import annotations

import hashlib
import json
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[5]
BASE = Path(__file__).resolve().parent
INPUT = "c83f7bc0a0957922a98a94bfe38ad8a8a400df9d"
DEFINITION = "542349e2f70ec9883d743594f906ac6c4e96ba5a"
PRODUCT = "1d6ee92adf6ccca38eeee64b1371cba1497e8fcd"
FRAGMENTS = {
    "foundation-api": "docs/implementation/pr38-final/results/t01-foundation-api/foundation-api.fragment.json",
    "components-api": "docs/implementation/pr38-final/results/t01-components-api/fragment.json",
}
EXPECTED_HASHES = {
    "foundation-api": "79c634ea8e20c6cb62a0c5463f6bfe05442ce15c9e4bb55d59cc6dc612485d73",
    "components-api": "237bae18fc224fc87b71148a7ff81ce88a7418679aa9f25d4fa1e5133dd6ff1d",
    "app-routes": "c63bc59a4d2312aceec26483a49eecbd854635fe559f5dc811cdfa1672b9ad0c",
    "history-requirements": "98acf988c44ae7258678a721b5709c948378a07798d1f8cce1e0b418a7c6ad27",
}


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def git_bytes(commit: str, path: str) -> bytes:
    return subprocess.check_output(["git", "show", f"{commit}:{path}"], cwd=ROOT)


def compact(value: str) -> str:
    return re.sub(r"\s+", " ", value).strip()


def resolve(root, pointer: str):
    value = root
    for token in pointer.lstrip("/").split("/"):
        token = token.replace("~1", "/").replace("~0", "~")
        value = value[int(token)] if isinstance(value, list) else value[token]
    return value


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def main() -> dict:
    path = BASE / "api-join-supplement.json"
    result = json.loads(path.read_text(encoding="utf-8"))
    fragments = {name: json.loads((ROOT / p).read_text(encoding="utf-8")) for name, p in FRAGMENTS.items()}
    foundation = fragments["foundation-api"]
    component = fragments["components-api"]
    require(result["schema"] == "termrock.pr38.t01.final_api_join_supplement.v1", "schema mismatch")
    require(result["provenance"]["fixed_candidate_input"]["commit"] == INPUT, "fixed candidate input mismatch")
    require(result["provenance"]["definition_input"]["commit"] == DEFINITION, "definition input mismatch")
    require(result["provenance"]["product_source"]["commit"] == PRODUCT, "product source mismatch")
    head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    require(
        subprocess.run(["git", "merge-base", "--is-ancestor", INPUT, head], cwd=ROOT, check=False).returncode == 0,
        "fixed candidate input is not an ancestor of checkout head",
    )
    source_diff = subprocess.run(["git", "diff", "--quiet", INPUT, head, "--", "crates"], cwd=ROOT, check=False)
    require(source_diff.returncode == 0, "product source differs from fixed candidate input")
    worktree_source_diff = subprocess.run(["git", "diff", "--quiet", "HEAD", "--", "crates"], cwd=ROOT, check=False)
    require(worktree_source_diff.returncode == 0, "product source has unstaged or staged worktree changes")
    unchanged = subprocess.run(["git", "diff", "--quiet", DEFINITION, INPUT, "--", "crates"], cwd=ROOT, check=False)
    require(unchanged.returncode == 0, "product source bytes changed between definition and candidate input")
    for producer in result["provenance"]["accepted_producers"]:
        require(sha((ROOT / producer["path"]).read_bytes()) == producer["artifact_sha256"], f"producer hash mismatch: {producer['fragment_id']}")
    for row in result["source_hash_records"]:
        require(sha(git_bytes(row["source_commit"], row["path"])) == row["sha256"], f"source hash mismatch: {row['path']}")

    all_join_ids = []
    locator_refs = set()
    w44_rows = result["w44_api_joins"]
    require(len(w44_rows) == 37, "W44 row count is not 37")
    for row in w44_rows:
        all_join_ids.append(row["join_id"])
        source_ref = row["source_locator_ref"]
        locator = resolve(component, source_ref["record_pointer"])
        require(source_ref["component_wid"] == "W44", "W44 locator has the wrong component ID")
        require(source_ref["record_pointer"] not in locator_refs, "W44 locator is joined more than once")
        locator_refs.add(source_ref["record_pointer"])
        api_ref = row["api_index_ref"]
        item = resolve(foundation, api_ref["record_pointer"])
        export_ref = row["source_export_ref"]
        export = resolve(foundation, export_ref["record_pointer"])
        require(api_ref["fragment_id"] == "foundation-api", "W44 join does not use the accepted foundation API index")
        require(api_ref["record_id"] == item["api_record_id"], "W44 API record ID does not close")
        require(item["current_qualified_symbol"] == locator["qualified_symbol"], "W44 symbol mismatch")
        require(item["definition_owner"] == api_ref["definition_owner"], "W44 definition owner mismatch")
        require(item["source_path"] == locator["source"] and item["line"] == locator["line"], "W44 source location mismatch")
        require(item["source_sha256"] == locator["sha256"], "W44 source hash mismatch")
        require(export["export_id"] == export_ref["record_id"], "W44 public export ID does not close")
        require(export["kind"] == "public_definition", "W44 export pointer is not a public definition")
        require(export["current_qualified_symbol"] == locator["qualified_symbol"], "W44 public export symbol mismatch")
        require(export["definition_owner"] == item["definition_owner"], "W44 public export owner mismatch")
        require(row["status"] == "RESOLVED_SOURCE_EXACT_API_JOIN", "W44 row has a nonresolved status")
        require(row["api"]["api_record_id"] == item["api_record_id"], "W44 copied API ID differs")
        require(locator["api_id"] is None, "W44 producer locator changed from unjoined state")
        require(row["api_index_ref"]["canonical_public_path"] == export["export_key"]["canonical_public_path"], "W44 canonical public path mismatch")
        require(row["api_index_ref"]["canonical_public_path"] in row["public_path_expansion"]["paths"], "W44 direct path missing from path expansion")
        require(row["source_evidence"]["file_sha256"] == sha(git_bytes(INPUT, locator["source"])), "W44 source bytes differ")
        require(row["source_evidence"]["source_signature"] == row["api"]["signature"] or compact(row["source_evidence"]["source_signature"]) == compact(item["signature"]), "W44 signature text differs")
    expected_locators = {
        f"/components/{next(i for i, x in enumerate(component['components']) if x['wid'] == 'W44')}/unjoined_api_declarations/{i}"
        for i in range(37)
    }
    require(locator_refs == expected_locators, "W44 locator references do not close over all 37 source locators")

    all_item_ids = {item["api_record_id"] for item in foundation["items"]}
    state_ref = result["w44_relationships"]["state_type_ref"]
    action_ref = result["w44_relationships"]["action_type_ref"]
    require(state_ref["record_id"] in all_item_ids and action_ref["record_id"] in all_item_ids, "W44 state/action API IDs are absent")
    require(result["w44_relationships"]["update_method_ref"]["uses_state"] == state_ref["record_id"], "update state link differs")
    require(result["w44_relationships"]["update_method_ref"]["returns_action"] == action_ref["record_id"], "update action link differs")
    require(result["w44_relationships"]["draw_method_ref"]["uses_state"] == state_ref["record_id"], "draw state link differs")
    require(result["w44_relationships"]["state_field_api_refs"], "TerminalViewState fields are not linked")
    require(result["w44_relationships"]["action_variant_api_refs"], "TerminalAction variants are not linked")
    require(result["w44_relationships"]["builder_api_refs"], "W44 builders are not linked")
    default = result["w44_relationships"]["derived_default"]
    all_join_ids.append(default["join_id"])
    require(default["record_kind"] == "SUPPLEMENTAL_DERIVED_TRAIT_IMPL", "derived Default record kind mismatch")
    require(default["api_id"] is None and default["trait_path"] == "core::default::Default", "derived Default invented an API ID")
    receiver = resolve(foundation, default["receiver_api_ref"]["record_pointer"])
    require(receiver["api_record_id"] == default["receiver_api_ref"]["record_id"], "Default receiver API reference does not close")
    require(default["source_evidence"]["attribute_text"] == "#[derive(Debug, Clone, PartialEq, Eq, Default)]", "derive evidence differs")
    require(default["source_evidence"]["attribute_line"] == 222, "derive source line differs")
    require(default["component_default_locator_ref"]["record_id"] is None, "derived Default producer ID was fabricated")

    extensions = result["extension_joins"]
    require(len(extensions) == 30, "extension count is not 30")
    seen_extensions = set()
    null_count = 0
    trait_member_ids = set()
    component_ext_by_id = {row["extension_id"]: (index, row) for index, row in enumerate(component["extension_points"])}
    for row in extensions:
        all_join_ids.append(row["join_id"])
        extension_id = row["extension_id"]
        require(extension_id not in seen_extensions, "extension ID appears more than once")
        seen_extensions.add(extension_id)
        producer_ref = row["producer_ref"]
        producer = resolve(component, producer_ref["record_pointer"])
        require(producer["extension_id"] == extension_id == producer_ref["record_id"], "extension producer pointer does not close")
        require(producer["api_id"] == producer_ref["producer_api_id"], "extension producer API ID changed")
        if producer["api_id"] is None:
            null_count += 1
            require(row["api_index_ref"]["fragment_id"] == "foundation-api", "null extension ID lacks foundation-item join")
            require(row["api_index_ref"]["record_id"] == resolve(foundation, row["api_index_ref"]["record_pointer"])["api_record_id"], "extension foundation API ID does not close")
        else:
            require(row["api_index_ref"]["fragment_id"] == "components-api", "existing extension ID lacks component-item join")
            require(row["api_index_ref"]["record_id"] == producer["api_id"], "extension component API ID differs")
        api_item = resolve(fragments[row["api_index_ref"]["fragment_id"]], row["api_index_ref"]["record_pointer"])
        source = row["source_evidence"]
        source_bytes = git_bytes(INPUT, source["path"])
        source_text = source_bytes.decode("utf-8")
        require(sha(source_bytes) == source["file_sha256"] == producer["definition"]["sha256"], "extension file hash mismatch")
        require(source["signature"] == producer["signature"] == producer["source_declaration_ref"]["statement"], "extension full source signature differs")
        require(source["signature"] in source_text, "extension full signature is absent from pinned source")
        require(source["qualified_symbol"] == producer["qualified_symbol"], "extension source symbol differs")
        require(source["definition_owner"] == row["api_index_ref"]["definition_owner"], "extension definition owner mismatch")
        require(source["source_package_id"] == row["api_index_ref"]["source_package_id"], "extension source package ID differs")
        require(source["definition_package_id"] == row["api_index_ref"]["definition_package_id"], "extension definition package ID differs")
        require(source["manifest_sha256"] == row["api_index_ref"]["manifest_sha256"], "extension manifest identity differs")
        manifest = git_bytes(INPUT, source["manifest_path"])
        require(sha(manifest) == source["manifest_sha256"], "extension manifest bytes differ")
        paths = row["public_visibility"]["public_paths"]
        evidence_paths = {item["path"] for item in row["public_visibility"]["public_path_evidence"]}
        require(set(paths) == evidence_paths, "extension public path lacks or adds evidence")
        require(all(item.get("externally_reachable") is True for item in row["public_visibility"]["public_path_evidence"]), "extension public path is not source-reachable")
        require(set(row["api_index_ref"]["public_paths"]) == set(paths), "extension index reference public paths differ")
        canonical = row["api_index_ref"]["canonical_public_path"]
        require(canonical is None or canonical in paths, "extension canonical public path is unsupported")
        for ref in row["public_visibility"]["public_path_evidence_refs"]:
            evidence = resolve(component, ref["record_pointer"])
            require(evidence["path"] == ref["public_path"], "extension path evidence pointer does not close")
        for ref in row["public_visibility"]["public_export_refs"]:
            export = resolve(fragments[ref["fragment_id"]], ref["record_pointer"])
            require(ref["public_path"] in export.get("public_qualified_paths", []) or ref["public_path"] in export.get("public_paths", []) or any(
                item.get("public_path") == ref["public_path"] and item.get("actual_definition_owner") == row["api_index_ref"]["definition_owner"]
                for item in export.get("expanded_symbols", [])
            ), "extension export pointer does not carry its path")
        for member_ref in row["trait_member_refs"]:
            member = resolve(foundation, member_ref["api_index_ref"]["record_pointer"])
            require(member["api_record_id"] == member_ref["api_index_ref"]["record_id"], "trait member API ID does not close")
            require(member["definition_owner"] == row["api_index_ref"]["definition_owner"], "trait member has a different owner")
            require(member["source_path"] == source["path"] and member["source_sha256"] == source["file_sha256"], "trait member source identity differs")
            require(member["line"] == member_ref["line"], "trait member source line differs")
            trait_member_ids.add(member["api_record_id"])
        require(row["status"] == "RESOLVED_SOURCE_EXACT_API_ITEM_JOIN", "extension row is not source-resolved")
    require(null_count == 9, "expected nine completed null extension joins")
    require(len(trait_member_ids) >= 20, "trait member API links are incomplete")

    inspector = result["inspector_body_disposition"]
    all_join_ids.append(inspector["join_id"])
    producer = resolve(component, inspector["producer_ref"]["record_pointer"])
    require(producer["name"] == "InspectorBody" and producer["disposition"] == "UNRESOLVED", "InspectorBody producer row differs")
    require(inspector["disposition"] == "UNRESOLVED" and inspector["decision"] is None, "InspectorBody was retired or decided")
    require(inspector["task_id"] == "T07" and inspector["case_id"] in producer["case_ids"], "InspectorBody ownership/case link differs")
    require(inspector["later_case_proposal"]["case_id"] is None, "later proposed case was given an unapproved ID")
    require(inspector["later_case_proposal"]["status"] == "PROPOSED_NOT_APPROVED", "later case approval state differs")
    require(inspector["blocks_final_goal"] is True and inspector["blocks_t01_inventory"] is False, "InspectorBody gap scope differs")
    exact = re.compile(r"\bInspectorBody\b")
    for record in inspector["product_gap"].values():
        if isinstance(record, dict) and record.get("path") and record.get("sha256"):
            source_commit = record.get("commit") or record.get("source_commit") or (PRODUCT if record is inspector["product_gap"].get("historical_source") or record is inspector["product_gap"].get("historical_public_exports") else INPUT)
            require(sha(git_bytes(source_commit, record["path"])) == record["sha256"], "InspectorBody source hash differs")
    source_scope = [
        (INPUT, "crates/termrock-feedback/src/inspector_body.rs"),
        (INPUT, "crates/termrock-feedback/src/lib.rs"),
        (PRODUCT, "crates/termrock-feedback/src/inspector_body.rs"),
        (PRODUCT, "crates/termrock-feedback/src/lib.rs"),
    ]
    require(not any(exact.search(git_bytes(commit, path).decode("utf-8")) for commit, path in source_scope), "exact InspectorBody token exists in scoped source")
    for fragment in fragments.values():
        for collection in ("items", "exports"):
            require(not any(exact.search(json.dumps(row, ensure_ascii=False)) for row in fragment.get(collection, [])), "exact InspectorBody token exists in accepted public index")
    audit_path = "docs/audits/evidence/2026-10-21-current-blockers/inspector-body-f8f3-independent-rejection.json"
    audit = json.loads(git_bytes(DEFINITION, audit_path).decode("utf-8"))
    require(audit.get("decision") is None, "historical source audit now has a decision")

    require(len(all_join_ids) == len(set(all_join_ids)), "join IDs are not unique")
    require(result["verification"]["functional_case_count"] == 0, "functional cases were claimed")
    require(result["verification"]["functional_status"] == "NOT VERIFIED", "functional status is not NOT VERIFIED")
    require(result["verification"]["independent_review"] == "PENDING", "independent review is not pending")
    summary = result["summary"]
    require(summary["w44_resolved_count"] == 37 and summary["w44_locator_count"] == 37, "W44 summary count differs")
    require(summary["extension_completed_null_api_id_count"] == 9 and summary["extension_preexisting_api_id_count"] == 21, "extension summary count differs")
    return {
        "status": "PASS_STRUCTURAL_ONLY",
        "w44_locators_resolved": len(locator_refs),
        "w44_default_supplements": 1,
        "extension_rows": len(extensions),
        "extension_null_api_ids_completed": null_count,
        "trait_member_api_rows": len(trait_member_ids),
        "inspector_body_disposition": "UNRESOLVED_T07_PRODUCT_GAP_RECORDED",
        "join_ids_unique": len(all_join_ids),
        "functional_cases": 0,
        "functional_status": "NOT VERIFIED",
    }


if __name__ == "__main__":
    print(json.dumps(main(), indent=2))
