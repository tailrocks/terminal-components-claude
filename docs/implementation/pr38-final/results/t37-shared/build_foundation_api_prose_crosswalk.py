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
FOUNDATION_DISPOSITION_PATH = "docs/implementation/pr38-final/results/t37-foundations/source-api-prose-disposition.json"
FOUNDATION_DISPOSITION_SHA256 = "8e3923ebb4f6d5974a2e632ca32e5406718a5985a8f26a17a75fcbd9a3f4e9df"
FOUNDATION_RECORD_COUNT = 1122
OUTPUT_PATH = "docs/implementation/pr38-final/results/t37-shared/foundation-api-prose-crosswalk.json"
COORDINATOR_REVIEW_PATH = "docs/implementation/pr38-final/results/t37-api-crosswalk-integration/coordinator-review-decisions.json"

COHORTS: tuple[dict[str, Any], ...] = (
    {
        "key": "values",
        "cohort_id": "T37-API-PROSE-COHORT-A",
        "label": "foundation values",
        "path": "docs/implementation/pr38-final/results/t37-api-values/decisions.json",
        "runbook_path": "docs/implementation/pr38-final/results/t37-api-values/agent-runbook.json",
        "integration_commit": "9463cb8c36f0bc6e53af241334efb19bb8a2e5f1",
        "imported_commit": "9463cb8c36f0bc6e53af241334efb19bb8a2e5f1",
        "reviewed_source_commit": "66728b43068c1ca54deba2a9cbd4d9a9973af182",
        "artifact_sha256": "6c5c9147f0d0339b5fbba9adfe1fc8703c773c078fb7806e9ec67ee07e958cf4",
        "runbook_sha256": "acafc73498889bf6b4ecb4dae80659d1b6ec170316a2d37b880a4683989d4f1a",
        "record_count": 322,
        "disposition_key": "disposition",
        "identity_reference_key": "identity_source",
        "reviewer_id": "/root/t37_components_review",
        "reviewer_model": "gpt-6.1-sol",
        "reviewer_effort": "high",
        "decision_id": "T37-API-PROSE-01",
        "required_review_status": "ACCEPTED",
    },
    {
        "key": "runtime",
        "cohort_id": "T37-API-PROSE-COHORT-B",
        "label": "foundation runtime",
        "path": "docs/implementation/pr38-final/results/t37-api-runtime/foundation-runtime-disposition.json",
        "runbook_path": "docs/implementation/pr38-final/results/t37-api-runtime/agent-runbook.json",
        "integration_commit": "6e38a1d3be4b0dcfd47f793e7975299254e38d14",
        "imported_commit": "6e38a1d3be4b0dcfd47f793e7975299254e38d14",
        "reviewed_source_commit": "4890b3d9ae40323b07f8d112d0fcf038e22059c3",
        "artifact_sha256": "100b55db7b05031451bca35c2142dbda6b3905d69d7ca404308ed4819a2b03a4",
        "runbook_sha256": "f1b9a3095146a1c4a57ebea448b9aa900d2e44cbc0dd6a0a809307441175f166",
        "record_count": 304,
        "disposition_key": "semantic_disposition",
        "identity_reference_key": "identity_crosswalk",
        "reviewer_id": "/root/t00_independent_review",
        "reviewer_model": "gpt-6.1-sol",
        "reviewer_effort": "high",
        "decision_id": "T37-API-PROSE-01",
        "required_review_status": "ACCEPTED",
    },
    {
        "key": "support",
        "cohort_id": "T37-API-PROSE-COHORT-C",
        "label": "foundation support",
        "path": "docs/implementation/pr38-final/results/t37-api-support/foundation-support-disposition.json",
        "runbook_path": "docs/implementation/pr38-final/results/t37-api-support/agent-runbook.json",
        "integration_commit": "f2657fe1cf0dcfb7baac271b5d93c523369798f4",
        "imported_commit": "f2657fe1cf0dcfb7baac271b5d93c523369798f4",
        "reviewed_source_commit": "ee8a18a77f1a6e7340d0dfe81bbba16f4e87ddf9",
        "artifact_sha256": "15399df0af526eaf415ec31c6fe2449d659b0c2a1c52f46a0d3e97fde3b063ae",
        "runbook_sha256": None,
        "record_count": 496,
        "disposition_key": "semantic_disposition",
        "identity_reference_key": None,
        "reviewer_id": "/root/t00_independent_review",
        "reviewer_model": "gpt-6.1-sol",
        "reviewer_effort": "high",
        "decision_id": "T37-API-SUPPORT-EE8-ACCEPT",
        "required_review_status": "ACCEPTED",
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


def load_coordinator_decisions(root: Path) -> tuple[dict[str, dict[str, Any]], dict[str, Any]]:
    path = root / COORDINATOR_REVIEW_PATH
    if not path.is_file():
        fail(f"coordinator review decisions missing: {COORDINATOR_REVIEW_PATH}")
    data = path.read_bytes()
    record = read_json(data, COORDINATOR_REVIEW_PATH)
    if record.get("schema") != "t37.api-prose-coordinator-review-bindings.v1":
        fail("unexpected coordinator review decision schema")
    if record.get("recorded_by") != "/root":
        fail("coordinator review decisions lack the expected coordinator attribution")
    decisions = record.get("decisions")
    if not isinstance(decisions, list):
        fail("coordinator review decisions must be a list")
    known_ids = {cohort["cohort_id"] for cohort in COHORTS}
    by_cohort: dict[str, dict[str, Any]] = {}
    for decision in decisions:
        if not isinstance(decision, dict):
            fail("coordinator decision must be an object")
        cohort_id = decision.get("cohort_id")
        if cohort_id not in known_ids or cohort_id in by_cohort:
            fail(f"unknown or duplicate coordinator decision: {cohort_id}")
        by_cohort[cohort_id] = decision
    required = {cohort["cohort_id"] for cohort in COHORTS if cohort["required_review_status"] == "ACCEPTED"}
    if not required.issubset(by_cohort):
        fail("accepted cohort coordinator decisions are missing")
    return by_cohort, {
        "path": COORDINATOR_REVIEW_PATH,
        "sha256": sha256(data),
        "schema": record["schema"],
        "recorded_by": record.get("recorded_by"),
    }


def coordinator_review(
    root: Path,
    spec: dict[str, Any],
    artifact_bytes: bytes,
    decision: dict[str, Any] | None,
    decision_record: dict[str, Any],
    source_review_metadata: dict[str, Any],
) -> dict[str, Any]:
    if decision is None:
        if spec["required_review_status"] == "ACCEPTED":
            fail(f"accepted coordinator decision missing for {spec['cohort_id']}")
        return {
            "status": "REVIEW/PENDING",
            "reviewer": spec["reviewer_id"],
            "model": spec["reviewer_model"],
            "effort": spec["reviewer_effort"],
            "decision_id": spec["decision_id"],
            "binding_status": "NO_COORDINATOR_DECISION",
            "decision_record": decision_record,
            "source_recorded_review_metadata": source_review_metadata,
        }
    reviewed = decision.get("reviewed_artifact")
    if not isinstance(reviewed, dict):
        reviewed = {}
    reviewed_commit = reviewed.get("commit")
    reviewed_path = reviewed.get("path")
    reviewed_sha256 = reviewed.get("sha256")
    artifact_sha256 = sha256(artifact_bytes)
    commit_shape_valid = isinstance(reviewed_commit, str) and len(reviewed_commit) == 40 and all(char in "0123456789abcdef" for char in reviewed_commit)
    path_matches = reviewed_path == spec["path"]
    hash_matches = reviewed_sha256 == artifact_sha256
    source_matches = False
    if commit_shape_valid and path_matches:
        try:
            reviewed_bytes = git_bytes(root, f"{reviewed_commit}:{reviewed_path}")
            source_matches = reviewed_bytes == artifact_bytes and sha256(reviewed_bytes) == reviewed_sha256
        except subprocess.CalledProcessError:
            source_matches = False
    source_commit_matches = spec["reviewed_source_commit"] is None or reviewed_commit == spec["reviewed_source_commit"]
    imported_commit_matches = spec.get("imported_commit") is None or decision.get("imported_commit") == spec["imported_commit"]
    reviewer_matches = (
        decision.get("reviewer_id") == spec["reviewer_id"]
        and decision.get("reviewer_model") == spec["reviewer_model"]
        and decision.get("reviewer_effort") == spec["reviewer_effort"]
        and decision.get("decision_id") == spec["decision_id"]
    )
    binding_matches = path_matches and hash_matches and source_matches and source_commit_matches and imported_commit_matches and reviewer_matches
    requested_status = decision.get("status")
    effective_status = "ACCEPTED" if requested_status == "ACCEPTED" and binding_matches else "REVIEW/PENDING"
    if spec["required_review_status"] == "ACCEPTED" and (effective_status != "ACCEPTED" or not binding_matches):
        fail(f"accepted cohort review binding mismatch for {spec['cohort_id']}")
    binding_status = "EXACT_GIT_BLOB_MATCH" if binding_matches else "UNBOUND_OR_STALE_PENDING"
    return {
        "status": effective_status,
        "requested_status": requested_status,
        "reviewer": decision.get("reviewer_id"),
        "model": decision.get("reviewer_model"),
        "effort": decision.get("reviewer_effort"),
        "decision_id": decision.get("decision_id"),
        "reviewed_artifact": reviewed,
        "binding_status": binding_status,
        "binding_validation": {
            "path_matches_input": path_matches,
            "sha256_matches_input": hash_matches,
            "reviewed_git_blob_matches_input": source_matches,
            "reviewed_commit_matches_required_source": source_commit_matches,
            "imported_commit_matches_record": imported_commit_matches,
            "reviewer_and_decision_match_assignment": reviewer_matches,
        },
        "pending_work": decision.get("pending_work") if effective_status != "ACCEPTED" else None,
        "coordinator_decision_record": decision,
        "decision_record": decision_record,
        "source_recorded_review_metadata": source_review_metadata,
    }


def resolve_root(script: Path, requested: Path | None) -> Path:
    root = requested.resolve() if requested else script.resolve().parents[5]
    top = Path(git_text(root, "rev-parse", "--show-toplevel")).resolve()
    if top != root:
        fail(f"repository root mismatch: {root} != {top}")
    return root


def load_verified_inventory(
    root: Path,
    head: str,
    path: str,
    expected_sha256: str,
) -> tuple[dict[str, Any], dict[str, Any]]:
    object_bytes = git_bytes(root, f"{FOUNDATION_COMMIT}:{path}")
    if sha256(object_bytes) != expected_sha256:
        fail(f"immutable inventory hash mismatch for {path}")
    local_path = root / path
    if local_path.exists():
        local_bytes = local_path.read_bytes()
        if local_bytes != object_bytes or sha256(local_bytes) != expected_sha256:
            fail(f"integrated inventory differs from its accepted Git blob: {path}")
        integration_commit = latest_path_commit(root, head, path)
        integrated_bytes = git_bytes(root, f"{integration_commit}:{path}")
        if integrated_bytes != local_bytes:
            fail(f"local inventory differs from its committed integration: {path}")
        integration_status = "INTEGRATED_AND_HASH_VERIFIED"
        input_mode = "integrated_file_validated_against_immutable_git_blob"
    else:
        local_bytes = object_bytes
        integration_commit = None
        integration_status = "PENDING_INTEGRATION"
        input_mode = "immutable_git_blob; local integration pending"
    document = read_json(local_bytes, path)
    records = document.get("records")
    if document.get("record_count") != FOUNDATION_RECORD_COUNT or not isinstance(records, list) or len(records) != FOUNDATION_RECORD_COUNT:
        fail(f"inventory record count mismatch for {path}")
    metadata = {
        "path": path,
        "accepted_source_commit": FOUNDATION_COMMIT,
        "local_integration_commit": integration_commit,
        "sha256": expected_sha256,
        "record_count": len(records),
        "integration_status": integration_status,
        "input_mode": input_mode,
    }
    return document, metadata


def load_foundation_source(root: Path, head: str) -> tuple[dict[str, Any], dict[str, Any], dict[str, Any]]:
    source, source_metadata = load_verified_inventory(
        root,
        head,
        FOUNDATION_SOURCE_PATH,
        FOUNDATION_SOURCE_SHA256,
    )
    source_disposition, disposition_metadata = load_verified_inventory(
        root,
        head,
        FOUNDATION_DISPOSITION_PATH,
        FOUNDATION_DISPOSITION_SHA256,
    )
    if source_disposition.get("source_map_sha256") != FOUNDATION_SOURCE_SHA256:
        fail("source disposition inventory does not bind the accepted source map")
    source_metadata["source_path_count"] = len({record.get("path") for record in source["records"]})
    return source, source_metadata, disposition_metadata


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


def load_cohort(
    root: Path,
    head: str,
    spec: dict[str, Any],
    coordinator_decision: dict[str, Any] | None,
    coordinator_record: dict[str, Any],
) -> tuple[dict[str, Any], dict[str, Any], list[dict[str, Any]]]:
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
    source_review_metadata = {
        key: input_record[key]
        for key in ("status", "review_state", "correction_review", "semantic_review", "review_feedback")
        if key in input_record
    }
    review = coordinator_review(
        root,
        spec,
        artifact_bytes,
        coordinator_decision,
        coordinator_record,
        source_review_metadata,
    )
    cohort_metadata = {key: value for key, value in input_record.items() if key != "records"}
    historical_identity_reference = cohort_metadata.pop(spec["identity_reference_key"], None) if spec["identity_reference_key"] else None
    if historical_identity_reference is not None:
        historical_identity_reference = {
            "status": "HISTORICAL_PRECOMPOSITION_IDENTITY_REFERENCE",
            "replaced_by": OUTPUT_PATH,
            "source_value": historical_identity_reference,
        }
    cohort_info = {
        "cohort_id": spec["cohort_id"],
        "label": spec["label"],
        "input_artifact": {
            "path": cohort_path,
            "integration_commit": integration_commit,
            "reviewed_source_commit": (review.get("reviewed_artifact") or {}).get("commit"),
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
    source, source_metadata, disposition_metadata = load_foundation_source(root, head)
    coordinator_decisions, coordinator_record = load_coordinator_decisions(root)
    seen_indices: set[int] = set()
    seen_ids: set[str] = set()
    rows: list[dict[str, Any]] = []
    cohorts: list[dict[str, Any]] = []
    disposition_counts: dict[str, dict[str, int]] = {}
    for spec in COHORTS:
        cohort_record, cohort_info, raw_records = load_cohort(
            root,
            head,
            spec,
            coordinator_decisions.get(spec["cohort_id"]),
            coordinator_record,
        )
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
        "schema": "t37.foundation-api-prose-crosswalk.v4",
        "task_id": "T37",
        "status": "REVIEW/PENDING",
        "review_state": "PENDING_INDEPENDENT_SOL_HIGH_REVIEW",
        "work_input_commit": SOURCE_COMMIT,
        "composition_input_commit": git_text(root, "rev-parse", "HEAD"),
        "product_source_commit": PRODUCT_COMMIT,
        "cleanup_source_commit": CLEANUP_COMMIT,
        "accepted_foundation_source_commit": FOUNDATION_COMMIT,
        "source_inventory": source_metadata,
        "source_disposition_inventory": disposition_metadata,
        "coordinator_review_decisions": coordinator_record,
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
            "Every source identity is checked against the accepted immutable Git blob.",
            "Local inventory integration states are recorded in source_inventory and source_disposition_inventory.",
            f"Local foundation inventory state: {source_metadata['integration_status']}.",
            f"Local lexical inventory state: {disposition_metadata['integration_status']}.",
            "Hashes inside embedded cohort records remain source-commit evidence and are not represented as current candidate target hashes.",
            "All cohort review decisions are recorded separately from the pending independent global review.",
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
