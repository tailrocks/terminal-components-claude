#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from collections import Counter
from pathlib import Path, PurePosixPath
from typing import Any

SOURCE_COMMIT = "c817b99e9e43abbbf3519d56b2f5ee86924243d8"
PRODUCT_COMMIT = "1d6ee92adf6ccca38eeee64b1371cba1497e8fcd"
CLEANUP_COMMIT = "7368d4fc5a71e46cae5ee92963bcf3a6943953e9"
FOUNDATION_COMMIT = "2fee2c277d1213fc5fad5d81eb77479600182b69"
FOUNDATION_SOURCE_PATH = "docs/implementation/pr38-final/results/t37-foundations/source-api-prose.json"
FOUNDATION_SOURCE_SHA256 = "6ead70836c20e3ad1c2dc15f0de44daf35d74c6507e1bbfd086de5fa6e85b619"
FOUNDATION_RECORD_COUNT = 1122
OUTPUT_PATH = "docs/implementation/pr38-final/results/t37-shared/foundation-api-prose-crosswalk.json"

COHORTS: tuple[dict[str, Any], ...] = (
    {
        "key": "values",
        "cohort_id": "T37-API-PROSE-COHORT-A",
        "label": "foundation values",
        "path": "docs/implementation/pr38-final/results/t37-api-values/decisions.json",
        "runbook_path": "docs/implementation/pr38-final/results/t37-api-values/agent-runbook.json",
        "integration_commit": "9463cb8c36f0bc6e53af241334efb19bb8a2e5f1",
        "reviewed_source_commit": "66728b43068c1ca54deba2a9cbd4d9a9973af182",
        "artifact_sha256": "6c5c9147f0d0339b5fbba9adfe1fc8703c773c078fb7806e9ec67ee07e958cf4",
        "runbook_sha256": "acafc73498889bf6b4ecb4dae80659d1b6ec170316a2d37b880a4683989d4f1a",
        "record_count": 322,
        "disposition_key": "disposition",
        "identity_reference_key": "identity_source",
        "review": {
            "status": "ACCEPTED",
            "reviewer": "/root/t37_components_review",
            "model": "gpt-6.1-sol",
            "effort": "high",
            "decision_id": "T37-API-PROSE-01",
            "reviewed_source_commit": "66728b43068c1ca54deba2a9cbd4d9a9973af182",
            "evidence_source": "Coordinator decision record for independent acceptance of this reviewed source commit.",
        },
    },
    {
        "key": "runtime",
        "cohort_id": "T37-API-PROSE-COHORT-B",
        "label": "foundation runtime",
        "path": "docs/implementation/pr38-final/results/t37-api-runtime/foundation-runtime-disposition.json",
        "runbook_path": "docs/implementation/pr38-final/results/t37-api-runtime/agent-runbook.json",
        "integration_commit": "6e38a1d3be4b0dcfd47f793e7975299254e38d14",
        "reviewed_source_commit": "4890b3d9ae40323b07f8d112d0fcf038e22059c3",
        "artifact_sha256": "100b55db7b05031451bca35c2142dbda6b3905d69d7ca404308ed4819a2b03a4",
        "runbook_sha256": "f1b9a3095146a1c4a57ebea448b9aa900d2e44cbc0dd6a0a809307441175f166",
        "record_count": 304,
        "disposition_key": "semantic_disposition",
        "identity_reference_key": "identity_crosswalk",
        "review": {
            "status": "ACCEPTED",
            "reviewer": "/root/t00_independent_review",
            "model": "gpt-6.1-sol",
            "effort": "high",
            "decision_id": "T37-API-PROSE-01",
            "reviewed_source_commit": "4890b3d9ae40323b07f8d112d0fcf038e22059c3",
            "evidence_source": "Coordinator decision record for independent acceptance of this reviewed source commit.",
        },
    },
    {
        "key": "support",
        "cohort_id": "T37-API-PROSE-COHORT-C",
        "label": "foundation support",
        "path": "docs/implementation/pr38-final/results/t37-api-support/foundation-support-disposition.json",
        "runbook_path": "docs/implementation/pr38-final/results/t37-api-support/agent-runbook.json",
        "integration_commit": None,
        "reviewed_source_commit": None,
        "artifact_sha256": None,
        "runbook_sha256": None,
        "record_count": 496,
        "disposition_key": "semantic_disposition",
        "identity_reference_key": None,
        "review": {
            "status": "REVIEW/PENDING",
            "reviewer": "/root/t00_independent_review",
            "model": "gpt-6.1-sol",
            "effort": "high",
            "decision_id": "T37-API-PROSE-01",
            "pending_work": "Confirmed maintained-guide corrections await independent review.",
        },
    },
)


def fail(message: str) -> None:
    raise ValueError(message)


def canonical(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode("utf-8")


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def git_bytes(root: Path, spec: str) -> bytes:
    return subprocess.run(
        ["git", "-C", str(root), "show", spec],
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    ).stdout


def git_text(root: Path, *args: str) -> str:
    return subprocess.run(
        ["git", "-C", str(root), *args],
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    ).stdout.strip()


def latest_path_commit(root: Path, head: str, path: str) -> str:
    commit = git_text(root, "log", "-1", "--format=%H", head, "--", path)
    if not commit:
        fail(f"no committed input revision for {path}")
    return commit


def read_json(data: bytes, label: str) -> dict[str, Any]:
    value = json.loads(data)
    if not isinstance(value, dict):
        fail(f"{label} must contain a JSON object")
    return value


def resolve_root(script: Path, requested: Path | None) -> Path:
    root = requested.resolve() if requested else script.resolve().parents[5]
    top = Path(git_text(root, "rev-parse", "--show-toplevel")).resolve()
    if top != root:
        fail(f"repository root mismatch: {root} != {top}")
    return root


def load_foundation_source(root: Path) -> tuple[dict[str, Any], dict[str, Any]]:
    object_bytes = git_bytes(root, f"{FOUNDATION_COMMIT}:{FOUNDATION_SOURCE_PATH}")
    object_sha256 = sha256(object_bytes)
    if object_sha256 != FOUNDATION_SOURCE_SHA256:
        fail(f"immutable foundation source hash mismatch: {object_sha256}")
    local_path = root / FOUNDATION_SOURCE_PATH
    if local_path.exists():
        local_bytes = local_path.read_bytes()
        if sha256(local_bytes) != FOUNDATION_SOURCE_SHA256 or local_bytes != object_bytes:
            fail("integrated foundation source differs from its accepted Git blob")
        integration_status = "INTEGRATED_AND_HASH_VERIFIED"
        input_mode = "integrated_file_validated_against_immutable_git_blob"
    else:
        local_bytes = object_bytes
        integration_status = "PENDING_INTEGRATION"
        input_mode = "immutable_git_blob; local integration pending"
    source = read_json(local_bytes, FOUNDATION_SOURCE_PATH)
    records = source.get("records")
    if source.get("record_count") != FOUNDATION_RECORD_COUNT or not isinstance(records, list) or len(records) != FOUNDATION_RECORD_COUNT:
        fail("foundation source record count mismatch")
    paths = {record.get("path") for record in records}
    metadata = {
        "path": FOUNDATION_SOURCE_PATH,
        "accepted_source_commit": FOUNDATION_COMMIT,
        "sha256": FOUNDATION_SOURCE_SHA256,
        "record_count": len(records),
        "source_path_count": len(paths),
        "integration_status": integration_status,
        "input_mode": input_mode,
    }
    return source, metadata


def source_record_for(source: dict[str, Any], index: int) -> dict[str, Any]:
    records = source["records"]
    if not isinstance(index, int) or index < 0 or index >= len(records):
        fail(f"source_map_record out of range: {index}")
    record = records[index]
    if not isinstance(record, dict):
        fail(f"source record {index} is not an object")
    return record


def validate_destination(root: Path, destination: Any, cohort_id: str, index: int) -> None:
    if destination is None:
        return
    if not isinstance(destination, str) or not destination:
        fail(f"invalid destination in {cohort_id} record {index}")
    target_path, separator, fragment = destination.partition("#")
    candidate_path = PurePosixPath(target_path)
    if candidate_path.is_absolute() or any(part in ("", ".", "..") for part in candidate_path.parts):
        fail(f"non-canonical destination path in {cohort_id} record {index}: {destination}")
    if not (target_path.startswith("docs/") or target_path.startswith("crates/")):
        fail(f"destination outside candidate tree in {cohort_id} record {index}: {destination}")
    target = (root / Path(*candidate_path.parts)).resolve()
    if root not in target.parents or not target.is_file():
        fail(f"destination does not resolve inside candidate in {cohort_id} record {index}: {destination}")
    if separator and not fragment:
        fail(f"empty destination anchor in {cohort_id} record {index}: {destination}")


def validate_identity(
    root: Path,
    source: dict[str, Any],
    cohort: dict[str, Any],
    raw_record: dict[str, Any],
    seen_indices: set[int],
    seen_ids: set[str],
) -> dict[str, Any]:
    cohort_id = cohort["cohort_id"]
    index = raw_record.get("source_map_record")
    if not isinstance(index, int):
        fail(f"missing integer source_map_record in {cohort_id}")
    if index in seen_indices:
        fail(f"overlapping source_map_record {index}")
    seen_indices.add(index)
    source_record = source_record_for(source, index)
    stable_id = sha256(canonical(source_record))
    if raw_record.get("id") != stable_id:
        fail(f"stable id mismatch in {cohort_id} record {index}")
    if stable_id in seen_ids:
        fail(f"duplicate stable id {stable_id}")
    seen_ids.add(stable_id)
    expected_line_range = {"start": source_record["start_line"], "end": source_record["end_line"]}
    if raw_record.get("path") != source_record["path"]:
        fail(f"source path mismatch in {cohort_id} record {index}")
    if raw_record.get("line_range") != expected_line_range:
        fail(f"source line range mismatch in {cohort_id} record {index}")
    if raw_record.get("source_sha256") != source_record["source_sha256"]:
        fail(f"source file hash mismatch in {cohort_id} record {index}")
    text_sha256 = sha256(canonical(source_record["text"]))
    if raw_record.get("text_sha256") != text_sha256:
        fail(f"source text hash mismatch in {cohort_id} record {index}")
    if "source_text" in raw_record:
        source_text = raw_record["source_text"]
        expected_text = source_record["text"] if isinstance(source_text, list) else "\n".join(source_record["text"])
        if source_text != expected_text:
            fail(f"source text identity mismatch in {cohort_id} record {index}")
    if not any(key in raw_record for key in ("attached_item", "cleanup_attachment")):
        fail(f"lexical attachment missing in {cohort_id} record {index}")
    if cohort["disposition_key"] not in raw_record:
        fail(f"cohort disposition missing in {cohort_id} record {index}")
    validate_destination(root, raw_record.get("destination"), cohort_id, index)
    return {
        "cohort_id": cohort_id,
        "source_map_record": index,
        "id": stable_id,
        "path": source_record["path"],
        "line_range": expected_line_range,
        "source_sha256": source_record["source_sha256"],
        "text_sha256": text_sha256,
        "source_record": source_record,
        "cohort_record": raw_record,
    }


def load_cohort(root: Path, head: str, spec: dict[str, Any]) -> tuple[dict[str, Any], dict[str, Any], list[dict[str, Any]]]:
    cohort_path = spec["path"]
    artifact_path = root / cohort_path
    if not artifact_path.is_file():
        fail(f"cohort record missing: {cohort_path}")
    artifact_bytes = artifact_path.read_bytes()
    artifact_sha = sha256(artifact_bytes)
    if spec["artifact_sha256"] is not None and artifact_sha != spec["artifact_sha256"]:
        fail(f"pinned cohort hash mismatch for {cohort_path}: {artifact_sha}")
    if spec["integration_commit"] is None:
        integration_commit = latest_path_commit(root, head, cohort_path)
    else:
        integration_commit = spec["integration_commit"]
    committed_artifact = git_bytes(root, f"{integration_commit}:{cohort_path}")
    if committed_artifact != artifact_bytes:
        fail(f"working cohort differs from its source commit: {cohort_path}")
    if spec["reviewed_source_commit"] is not None:
        reviewed_artifact = git_bytes(root, f"{spec['reviewed_source_commit']}:{cohort_path}")
        if reviewed_artifact != artifact_bytes:
            fail(f"cohort differs from its independently reviewed source: {cohort_path}")
    input_record = read_json(artifact_bytes, cohort_path)
    raw_records = input_record.get("records")
    if not isinstance(raw_records, list) or len(raw_records) != spec["record_count"]:
        fail(f"cohort count mismatch for {cohort_path}")
    if input_record.get("task_id") != "T37":
        fail(f"unexpected task id in {cohort_path}")
    if input_record.get("work_input_commit") not in (SOURCE_COMMIT, "e164e7b5b17e9c2d78bf30cfbbfe6bdb36e7b7ab") and input_record.get("source_input_commit") != SOURCE_COMMIT:
        fail(f"unexpected work input commit in {cohort_path}")
    raw_inventory = input_record.get("raw_inventory") or input_record.get("raw_source_inventory") or input_record.get("source_inventory")
    if not isinstance(raw_inventory, dict):
        fail(f"source inventory provenance missing in {cohort_path}")
    if raw_inventory.get("commit") not in (None, FOUNDATION_COMMIT):
        fail(f"unexpected raw inventory commit in {cohort_path}")
    if raw_inventory.get("accepted_foundation_source_commit") not in (None, FOUNDATION_COMMIT):
        fail(f"unexpected accepted foundation commit in {cohort_path}")
    if raw_inventory.get("sha256") != FOUNDATION_SOURCE_SHA256:
        fail(f"raw source inventory hash mismatch in {cohort_path}")
    runbook_path = spec["runbook_path"]
    runbook_bytes = (root / runbook_path).read_bytes()
    runbook_sha = sha256(runbook_bytes)
    if spec["runbook_sha256"] is not None and runbook_sha != spec["runbook_sha256"]:
        fail(f"pinned runbook hash mismatch for {runbook_path}: {runbook_sha}")
    runbook_commit = latest_path_commit(root, head, runbook_path) if spec["runbook_sha256"] is None else spec["integration_commit"]
    if git_bytes(root, f"{runbook_commit}:{runbook_path}") != runbook_bytes:
        fail(f"working runbook differs from its source commit: {runbook_path}")
    runbook = read_json(runbook_bytes, runbook_path)
    cohort_metadata = {key: value for key, value in input_record.items() if key != "records"}
    historical_identity_reference = cohort_metadata.pop(spec["identity_reference_key"], None) if spec["identity_reference_key"] else None
    if historical_identity_reference is not None:
        historical_identity_reference = {
            "status": "HISTORICAL_PRECOMPOSITION_IDENTITY_REFERENCE",
            "replaced_by": OUTPUT_PATH,
            "source_value": historical_identity_reference,
        }
    review = dict(spec["review"])
    if spec["key"] == "support":
        source_status = input_record.get("status") or input_record.get("review_state")
        review["source_record_status"] = source_status
        explicit_review = input_record.get("independent_review") or input_record.get("review_feedback") or runbook.get("review_feedback")
        if isinstance(explicit_review, dict):
            review["source_review_record"] = explicit_review
            if explicit_review.get("reviewer"):
                review["reviewer"] = explicit_review["reviewer"]
            if explicit_review.get("model"):
                review["model"] = explicit_review["model"]
            if explicit_review.get("effort"):
                review["effort"] = explicit_review["effort"]
            if explicit_review.get("id"):
                review["decision_id"] = explicit_review["id"]
            if explicit_review.get("result") in ("ACCEPT", "ACCEPTED"):
                review["status"] = "ACCEPTED"
                review.pop("pending_work", None)
            elif explicit_review.get("result") in ("CORRECTION", "REVIEW"):
                review["status"] = "REVIEW/PENDING"
        elif source_status in ("ACCEPT", "ACCEPTED", "ACCEPTED_SCOPED"):
            review["status"] = "ACCEPTED"
            review.pop("pending_work", None)
    cohort_info = {
        "cohort_id": spec["cohort_id"],
        "label": spec["label"],
        "input_artifact": {
            "path": cohort_path,
            "integration_commit": integration_commit,
            "independently_reviewed_source_commit": spec["reviewed_source_commit"],
            "sha256": artifact_sha,
            "record_count": len(raw_records),
        },
        "input_runbook": {
            "path": runbook_path,
            "integration_commit": runbook_commit,
            "sha256": runbook_sha,
            "agent_id": runbook.get("agent_id") or runbook.get("worker"),
            "model": runbook.get("model"),
            "effort": runbook.get("effort"),
        },
        "independent_review": review,
        "historical_identity_reference": historical_identity_reference,
        "recorded_cohort_metadata": cohort_metadata,
    }
    return input_record, cohort_info, raw_records


def build(root: Path) -> dict[str, Any]:
    head = git_text(root, "rev-parse", "HEAD")
    source, source_metadata = load_foundation_source(root)
    seen_indices: set[int] = set()
    seen_ids: set[str] = set()
    rows: list[dict[str, Any]] = []
    cohorts: list[dict[str, Any]] = []
    disposition_counts: dict[str, dict[str, int]] = {}
    for spec in COHORTS:
        cohort_record, cohort_info, raw_records = load_cohort(root, head, spec)
        counts: Counter[str] = Counter()
        for raw_record in raw_records:
            row = validate_identity(root, source, spec, raw_record, seen_indices, seen_ids)
            rows.append(row)
            counts[str(raw_record[spec["disposition_key"]])] += 1
        if len(raw_records) != spec["record_count"]:
            fail(f"unexpected record count in {spec['cohort_id']}")
        cohort_info["record_count"] = len(raw_records)
        cohort_info["recorded_disposition_counts"] = dict(sorted(counts.items()))
        cohorts.append(cohort_info)
        disposition_counts[spec["cohort_id"]] = dict(sorted(counts.items()))
    expected_indices = set(range(FOUNDATION_RECORD_COUNT))
    if seen_indices != expected_indices:
        missing = sorted(expected_indices - seen_indices)
        extra = sorted(seen_indices - expected_indices)
        fail(f"cohort coverage mismatch: missing={missing[:8]} extra={extra[:8]}")
    if len(rows) != FOUNDATION_RECORD_COUNT or len(seen_ids) != FOUNDATION_RECORD_COUNT:
        fail("global one-to-one coverage count mismatch")
    rows.sort(key=lambda row: row["source_map_record"])
    return {
        "schema": "t37.foundation-api-prose-crosswalk.v2",
        "task_id": "T37",
        "status": "REVIEW/PENDING",
        "review_state": "PENDING_INDEPENDENT_SOL_HIGH_REVIEW",
        "work_input_commit": SOURCE_COMMIT,
        "composition_input_commit": git_text(root, "rev-parse", "HEAD"),
        "product_source_commit": PRODUCT_COMMIT,
        "cleanup_source_commit": CLEANUP_COMMIT,
        "accepted_foundation_source_commit": FOUNDATION_COMMIT,
        "source_inventory": source_metadata,
        "cohorts": cohorts,
        "provenance_scope": {
            "embedded_cohort_hashes": "Verbatim input-record evidence tied to each cohort source commit; not current candidate target hashes.",
            "historical_identity_references": "Pre-composition references are explicitly labeled as historical.",
        },
        "coverage": {
            "expected_source_records": FOUNDATION_RECORD_COUNT,
            "composed_records": len(rows),
            "unique_source_map_indices": len(seen_indices),
            "unique_stable_ids": len(seen_ids),
            "coverage": "one-to-one; disjoint cohort indices 0-1121",
            "cohort_record_counts": {cohort["cohort_id"]: cohort["record_count"] for cohort in cohorts},
            "disposition_counts_by_cohort": disposition_counts,
        },
        "required_decision": {
            "id": "T37-API-PROSE-01",
            "reviewer_role": "independent gpt-6.1-sol high",
            "status": "PENDING",
            "scope": "Exact composed output, provenance, and structural coverage only; cohort semantic dispositions remain source records.",
        },
        "behavior_evidence": {
            "selected_behavior_cases": 0,
            "status": "NOT VERIFIED",
            "reason": "Documentation composition only; no product behavior case selected or run.",
        },
        "limits": [
            "The producer composes the three explicit cohort records. It does not infer disposition from keywords, source survival, or matching signatures.",
            "Each cohort record is embedded unchanged. Its attachment, disposition, clauses, excerpt, reason, destination, and decision identifiers retain their cohort-specific field names.",
            "Every source identity is checked against the accepted immutable Git blob. The local foundation source inventory is not integrated yet.",
            "Hashes inside embedded cohort records remain source-commit evidence and are not represented as current candidate target hashes.",
            "The current result remains REVIEW/PENDING while cohort C and independent global review remain open.",
            "No behavior tests, harnesses, fixtures, doctests, source changes, or Cargo commands are part of this unit.",
        ],
        "records": rows,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", type=Path)
    parser.add_argument("--output", type=Path, default=Path(OUTPUT_PATH))
    args = parser.parse_args()
    root = resolve_root(Path(__file__), args.repo_root)
    output = args.output if args.output.is_absolute() else root / args.output
    resolved_output = output.resolve()
    if root not in resolved_output.parents:
        fail("output path must remain inside the candidate checkout")
    result = build(root)
    encoded = (json.dumps(result, indent=2, ensure_ascii=False) + "\n").encode("utf-8")
    resolved_output.parent.mkdir(parents=True, exist_ok=True)
    resolved_output.write_bytes(encoded)
    print(
        f"wrote {resolved_output.relative_to(root)}: "
        f"{result['coverage']['composed_records']} groups; "
        f"{result['coverage']['cohort_record_counts']}; sha256 {sha256(encoded)}"
    )


if __name__ == "__main__":
    main()
