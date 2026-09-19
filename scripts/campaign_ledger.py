"""Fail-closed campaign ledger and proof-preparation validation.

The ledger is operator state, not candidate authority. A task row is useful to
preflight only when its receipt, result, reviewer record, dependency receipts,
and Git identities describe the same current campaign state.

This module deliberately raises explicit exceptions instead of using
``assert``. ``python -O`` must not remove an authority check.
"""

from __future__ import annotations

import hashlib
import json
import os
import re
import stat
import subprocess
from collections.abc import Mapping, Sequence
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any, NoReturn


LEDGER_SCHEMA = "campaign-ledger/v1"
RECEIPT_SCHEMA = "campaign-receipt/v2"
EVIDENCE_SCHEMA = "campaign-verifier-evidence/v1"
RESULT_SCHEMA = "campaign-task-result/v1"
REVIEW_SCHEMA = "campaign-review/v1"
PROOF_PREPARATION_SCHEMA = "campaign-proof-preparation/v1"
CONTEXT_INDEX_SCHEMA = "tc-proof-context-index/v1"
CONTEXT_SCHEMA = "tc-proof-context/v1"
PREPARATION_RESULT_SCHEMA = "tc-proof-preparation-result/v1"
OBSERVER_SCHEMA = "tc-proof-observer-capability/v1"
NATIVE_BUILD_SCHEMA = "tc-proof-native-build/v1"
PREPARATION_QUALIFICATION_SCHEMA = "campaign-preparation-qualification/v1"
PREPARATION_VERIFIER_SCHEMA = "campaign-preparation-verifier/v1"
PREPARATION_REVIEW_SCHEMA = "campaign-preparation-review/v1"
PREPARATION_EVIDENCE_SCHEMA = "campaign-preparation-evidence/v1"
PREFLIGHT_REPORT_SCHEMA = "campaign-preflight-report/v1"
PREPARATION_RECORD_KEY = "preparation"

FROZEN_ORACLE_TAG = "refs/tags/visual-baseline"
FROZEN_ORACLE_COMMIT = "4a79c0a2d40fca46fc406b77157ce3b3f12ec16b"
FROZEN_ORACLE_TREE = "0b1f13431fdfd6060cf9f45a114afa5a99cc6c26"
FROZEN_ORACLE_REF_SHA256 = "dd1df64115ad97feb40cdd1b888356d2461483c7f6372ad8b183364fb317b036"

ACCEPTED_TASK_STATUSES = frozenset({"verified", "integrated"})
PREPARATION_TASK_STATUSES = frozenset({"pending", "blocked"})
VALID_PREFLIGHT_TASK_STATUSES = ACCEPTED_TASK_STATUSES | PREPARATION_TASK_STATUSES
ACCEPTED_VERIFIER_VERDICT = "VERIFIED"
ACCEPTED_REVIEWER_VERDICT = "VERIFIED"
TASK_STATUSES = frozenset(
    {"pending", "dispatched", "verified", "integrated", "blocked"}
)
VERDICTS = frozenset({"VERIFIED", "REJECTED", "BLOCKED", "NOT_RUN"})

_FULL_SHA = re.compile(r"^[0-9a-f]{40}$")
_SHA256 = re.compile(r"^[0-9a-f]{64}$")
_TASK_ID = re.compile(r"^TASK-[0-9]{3}$")
_RECEIPT_KEY = re.compile(r"^task-[0-9]{3}$")

# Native macOS exposes temporary and home paths through these stable system
# aliases. This must stay byte-for-byte equivalent in policy to
# campaign-path-guards.sh and the Rust verifier: only the exact alias and
# exact resolved target are allowed. Candidate-created symlinks remain
# rejected.
_ALLOWED_SYSTEM_SYMLINKS = {
    Path("/etc"): Path("/private/etc"),
    Path("/home"): Path("/System/Volumes/Data/home"),
    Path("/tmp"): Path("/private/tmp"),
    Path("/var"): Path("/private/var"),
}


class LedgerValidationError(AssertionError):
    """A ledger or preparation receipt failed an authority check."""


def _reject(message: str) -> NoReturn:
    raise LedgerValidationError(message)


def _mapping(value: Any, field: str) -> Mapping[str, Any]:
    if not isinstance(value, Mapping):
        _reject(f"{field} must be an object")
    return value


def _string(value: Any, field: str, *, nonempty: bool = True) -> str:
    if not isinstance(value, str) or (nonempty and not value):
        _reject(f"{field} must be a non-empty string")
    return value


def _full_sha(value: Any, field: str) -> str:
    value = _string(value, field)
    if _FULL_SHA.fullmatch(value) is None:
        _reject(f"{field} must be a full lowercase commit SHA")
    return value


def _sha256(value: Any, field: str) -> str:
    value = _string(value, field)
    if _SHA256.fullmatch(value) is None:
        _reject(f"{field} must be a lowercase SHA-256")
    return value


def _timestamp(value: Any, field: str) -> str:
    value = _string(value, field)
    try:
        parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError:
        _reject(f"{field} must be an ISO-8601 timestamp")
    if parsed.tzinfo is None:
        _reject(f"{field} must include a timezone")
    return value


def _absolute_path(value: Any, field: str) -> str:
    value = _string(value, field)
    if not Path(value).is_absolute():
        _reject(f"{field} must be an absolute path")
    return value


def _relative_path(value: Any, field: str) -> str:
    value = _string(value, field)
    path = Path(value)
    if path.is_absolute() or not path.parts or ".." in path.parts:
        _reject(f"{field} must be a repository-relative path without '..'")
    return value


def _tree_sha(value: Any, field: str) -> str:
    """Validate a Git tree object identity using the same shape as a commit."""

    return _full_sha(value, field)


def _parse_timestamp(value: Any, field: str) -> datetime:
    value = _timestamp(value, field)
    try:
        parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError:  # pragma: no cover - _timestamp already checked this
        _reject(f"{field} must be an ISO-8601 timestamp")
    if parsed.tzinfo is None:  # pragma: no cover - _timestamp already checked
        _reject(f"{field} must include a timezone")
    return parsed.astimezone(timezone.utc)


def _path_components_have_no_symlink(path: Path, field: str) -> None:
    """Reject unsafe symlinks while accepting only exact native system aliases."""

    if not path.is_absolute():
        _reject(f"{field} must be absolute: {path}")
    current = Path(path.anchor)
    for component in path.parts[1:]:
        current /= component
        try:
            mode = os.lstat(current).st_mode
        except OSError as error:
            _reject(f"{field} has an unreadable path component {current}: {error}")
        if stat.S_ISLNK(mode):
            allowed_target = _ALLOWED_SYSTEM_SYMLINKS.get(current)
            if allowed_target is None or Path(os.path.realpath(current)) != allowed_target:
                _reject(f"{field} contains a symlinked path component: {current}")


def _qualification_file(path: Path, field: str) -> Path:
    """Return a single-link regular file suitable as a trust input."""

    _path_components_have_no_symlink(path, field)
    try:
        metadata = os.lstat(path)
    except OSError as error:
        _reject(f"{field} is unreadable: {error}")
    if not stat.S_ISREG(metadata.st_mode):
        _reject(f"{field} is not a regular file: {path}")
    if metadata.st_nlink != 1:
        _reject(f"{field} is hard-linked; trust inputs require one link: {path}")
    return path


def _qualification_directory(path: Path, field: str) -> Path:
    _path_components_have_no_symlink(path, field)
    try:
        metadata = os.lstat(path)
    except OSError as error:
        _reject(f"{field} is unreadable: {error}")
    if not stat.S_ISDIR(metadata.st_mode):
        _reject(f"{field} is not a directory: {path}")
    return path


def _qualification_file_under(path: Path, parent: Path, field: str) -> Path:
    """Validate a single-link regular file without following trust-path links."""

    if not path.is_absolute() or ".." in path.parts:
        _reject(f"{field} must be a safe absolute path")
    file_path = _qualification_file(path, field)
    try:
        file_path.resolve().relative_to(parent.resolve())
    except ValueError:
        _reject(f"{field} is outside its bound directory")
    return file_path


def _path_ref_shape(value: Any, field: str) -> None:
    value = _mapping(value, field)
    _unknown(value, {"path", "sha256"}, field)
    _required(value, ("path", "sha256"), field)
    _absolute_path(value["path"], f"{field}.path")
    _sha256(value["sha256"], f"{field}.sha256")


def _validate_taskfmt_record(value: Mapping[str, Any], field: str) -> None:
    allowed = {
        "taskfmt_revision",
        "taskfmt_version",
        "taskfmt_sha256",
        "taskfmt_source",
        "taskfmt_path",
    }
    _unknown(value, allowed, field)
    _required(value, tuple(sorted(allowed)), field)
    _full_sha(value["taskfmt_revision"], f"{field}.taskfmt_revision")
    _string(value["taskfmt_version"], f"{field}.taskfmt_version")
    _sha256(value["taskfmt_sha256"], f"{field}.taskfmt_sha256")
    _absolute_path(value["taskfmt_source"], f"{field}.taskfmt_source")
    _absolute_path(value["taskfmt_path"], f"{field}.taskfmt_path")


def _validate_preparation_qualification_shape(
    preparation: Mapping[str, Any], field: str = "ledger.preparation"
) -> None:
    """Validate the tracked shape before consulting external evidence."""

    allowed = {
        "schema",
        "integration_ref",
        "candidate_commit",
        "candidate_tree",
        "oracle",
        "catalog",
        "task_graph",
        "taskfmt",
        "proof_preparation",
        "verifier",
        "reviewer",
        "freshness",
    }
    _unknown(preparation, allowed, field)
    _required(preparation, tuple(sorted(allowed)), field)
    if preparation["schema"] != PREPARATION_QUALIFICATION_SCHEMA:
        _reject(f"{field}.schema is not {PREPARATION_QUALIFICATION_SCHEMA}")
    _string(preparation["integration_ref"], f"{field}.integration_ref")
    _full_sha(preparation["candidate_commit"], f"{field}.candidate_commit")
    _tree_sha(preparation["candidate_tree"], f"{field}.candidate_tree")

    oracle = _mapping(preparation["oracle"], f"{field}.oracle")
    _unknown(oracle, {"tag", "commit", "tree"}, f"{field}.oracle")
    _required(oracle, ("tag", "commit", "tree"), f"{field}.oracle")
    _string(oracle["tag"], f"{field}.oracle.tag")
    _full_sha(oracle["commit"], f"{field}.oracle.commit")
    _tree_sha(oracle["tree"], f"{field}.oracle.tree")

    for name in ("catalog", "task_graph"):
        identity = _mapping(preparation[name], f"{field}.{name}")
        _unknown(
            identity,
            {"commit", "tree", "manifest"},
            f"{field}.{name}",
        )
        _required(identity, ("commit", "tree", "manifest"), f"{field}.{name}")
        _full_sha(identity["commit"], f"{field}.{name}.commit")
        _tree_sha(identity["tree"], f"{field}.{name}.tree")
        manifest = _mapping(identity["manifest"], f"{field}.{name}.manifest")
        _unknown(manifest, {"path", "sha256"}, f"{field}.{name}.manifest")
        _required(manifest, ("path", "sha256"), f"{field}.{name}.manifest")
        _relative_path(manifest["path"], f"{field}.{name}.manifest.path")
        _sha256(manifest["sha256"], f"{field}.{name}.manifest.sha256")

    taskfmt = _mapping(preparation["taskfmt"], f"{field}.taskfmt")
    _validate_taskfmt_record(taskfmt, f"{field}.taskfmt")

    proof = _mapping(preparation["proof_preparation"], f"{field}.proof_preparation")
    _unknown(
        proof,
        {
            "task_id",
            "run_id",
            "preparation_receipt",
            "context_index",
            "observer",
            "contexts",
            "results",
        },
        f"{field}.proof_preparation",
    )
    _required(
        proof,
        (
            "task_id",
            "run_id",
            "preparation_receipt",
            "context_index",
            "observer",
            "contexts",
            "results",
        ),
        f"{field}.proof_preparation",
    )
    _task_id(proof["task_id"], f"{field}.proof_preparation.task_id")
    _absolute_path(proof["run_id"], f"{field}.proof_preparation.run_id")
    for name in ("preparation_receipt", "context_index", "observer"):
        _path_ref_shape(proof[name], f"{field}.proof_preparation.{name}")
    for name in ("contexts", "results"):
        entries = proof[name]
        if not isinstance(entries, list) or not entries:
            _reject(f"{field}.proof_preparation.{name} must be non-empty")
        seen: set[str] = set()
        for index, entry in enumerate(entries):
            entry = _mapping(entry, f"{field}.proof_preparation.{name}[{index}]")
            _unknown(
                entry,
                {"check_id", "path", "sha256"},
                f"{field}.proof_preparation.{name}[{index}]",
            )
            _required(
                entry,
                ("check_id", "path", "sha256"),
                f"{field}.proof_preparation.{name}[{index}]",
            )
            check_id = _string(
                entry["check_id"],
                f"{field}.proof_preparation.{name}[{index}].check_id",
            )
            if re.fullmatch(r"CHK-[0-9]{3}", check_id) is None:
                _reject(f"{field}.proof_preparation.{name}[{index}].check_id is invalid")
            if check_id in seen:
                _reject(f"{field}.proof_preparation.{name} has duplicate {check_id}")
            seen.add(check_id)
            _absolute_path(
                entry["path"],
                f"{field}.proof_preparation.{name}[{index}].path",
            )
            _sha256(
                entry["sha256"],
                f"{field}.proof_preparation.{name}[{index}].sha256",
            )

    for name in ("verifier", "reviewer"):
        record = _mapping(preparation[name], f"{field}.{name}")
        allowed_evidence = {"run_id", "evidence", "verdict", "recorded_at"}
        _unknown(record, allowed_evidence, f"{field}.{name}")
        _required(record, tuple(sorted(allowed_evidence)), f"{field}.{name}")
        _absolute_path(record["run_id"], f"{field}.{name}.run_id")
        _path_ref_shape(record["evidence"], f"{field}.{name}.evidence")
        if record["verdict"] != ACCEPTED_VERIFIER_VERDICT:
            _reject(f"{field}.{name}.verdict is not VERIFIED")
        _timestamp(record["recorded_at"], f"{field}.{name}.recorded_at")

    freshness = _mapping(preparation["freshness"], f"{field}.freshness")
    _unknown(
        freshness,
        {"qualified_at", "expires_at", "max_age_seconds"},
        f"{field}.freshness",
    )
    _required(
        freshness,
        ("qualified_at", "expires_at", "max_age_seconds"),
        f"{field}.freshness",
    )
    _timestamp(freshness["qualified_at"], f"{field}.freshness.qualified_at")
    _timestamp(freshness["expires_at"], f"{field}.freshness.expires_at")
    if type(freshness["max_age_seconds"]) is not int or freshness["max_age_seconds"] <= 0:
        _reject(f"{field}.freshness.max_age_seconds must be a positive integer")


def _task_id(value: Any, field: str = "task_id") -> str:
    value = _string(value, field)
    if _TASK_ID.fullmatch(value) is None:
        _reject(f"{field} must match TASK-NNN")
    return value


def _receipt_key_for(task_id: str) -> str:
    return "task-" + task_id[5:]


def _task_dependencies(value: Any, field: str) -> list[str]:
    if not isinstance(value, list):
        _reject(f"{field} must be an array")
    result: list[str] = []
    for index, item in enumerate(value):
        task_id = _task_id(item, f"{field}[{index}]")
        if task_id in result:
            _reject(f"{field} contains duplicate {task_id}")
        result.append(task_id)
    return result


def _unknown(value: Mapping[str, Any], allowed: set[str], field: str) -> None:
    unknown = sorted(set(value) - allowed)
    if unknown:
        _reject(f"{field} has unknown fields: {', '.join(unknown)}")


def _required(value: Mapping[str, Any], fields: Sequence[str], field: str) -> None:
    missing = [name for name in fields if name not in value]
    if missing:
        _reject(f"{field} is missing required fields: {', '.join(missing)}")


def _validate_result_shape(result: Mapping[str, Any], field: str = "result") -> None:
    allowed = {
        "schema",
        "task_id",
        "base",
        "candidate_tree_sha",
        "integration_commit",
        "run_id",
        "exit",
        "last_line",
        "status",
        "path",
        "sha256",
        "recorded_at",
    }
    _unknown(result, allowed, field)
    _required(
        result,
        (
            "schema",
            "task_id",
            "base",
            "candidate_tree_sha",
            "integration_commit",
            "run_id",
            "exit",
            "last_line",
            "status",
            "path",
            "sha256",
            "recorded_at",
        ),
        field,
    )
    if result["schema"] != RESULT_SCHEMA:
        _reject(f"{field}.schema is not {RESULT_SCHEMA}")
    _task_id(result["task_id"], f"{field}.task_id")
    _full_sha(result["base"], f"{field}.base")
    _full_sha(result["candidate_tree_sha"], f"{field}.candidate_tree_sha")
    _full_sha(result["integration_commit"], f"{field}.integration_commit")
    _absolute_path(result["run_id"], f"{field}.run_id")
    if type(result["exit"]) is not int:
        _reject(f"{field}.exit must be an integer")
    if result["exit"] != 0:
        _reject(f"{field}.exit must be 0")
    if result["last_line"] != "DONE":
        _reject(f"{field}.last_line must be DONE")
    if result["status"] != "passed":
        _reject(f"{field}.status must be passed")
    _absolute_path(result["path"], f"{field}.path")
    _sha256(result["sha256"], f"{field}.sha256")
    _timestamp(result["recorded_at"], f"{field}.recorded_at")


def _validate_reviewer_shape(
    reviewer: Mapping[str, Any], field: str = "reviewer"
) -> None:
    allowed = {
        "schema",
        "task_id",
        "base",
        "candidate_tree_sha",
        "integration_commit",
        "run_id",
        "verdict",
        "evidence",
        "evidence_sha256",
        "result_sha256",
        "recorded_at",
    }
    _unknown(reviewer, allowed, field)
    _required(
        reviewer,
        (
            "schema",
            "task_id",
            "base",
            "candidate_tree_sha",
            "integration_commit",
            "run_id",
            "verdict",
            "evidence",
            "evidence_sha256",
            "result_sha256",
            "recorded_at",
        ),
        field,
    )
    if reviewer["schema"] != REVIEW_SCHEMA:
        _reject(f"{field}.schema is not {REVIEW_SCHEMA}")
    _task_id(reviewer["task_id"], f"{field}.task_id")
    _full_sha(reviewer["base"], f"{field}.base")
    _full_sha(reviewer["candidate_tree_sha"], f"{field}.candidate_tree_sha")
    _full_sha(reviewer["integration_commit"], f"{field}.integration_commit")
    _absolute_path(reviewer["run_id"], f"{field}.run_id")
    if reviewer["verdict"] not in VERDICTS:
        _reject(f"{field}.verdict is not a known reviewer verdict")
    _absolute_path(reviewer["evidence"], f"{field}.evidence")
    _sha256(reviewer["evidence_sha256"], f"{field}.evidence_sha256")
    _sha256(reviewer["result_sha256"], f"{field}.result_sha256")
    _timestamp(reviewer["recorded_at"], f"{field}.recorded_at")


def _validate_receipt_shape(receipt: Mapping[str, Any], field: str) -> None:
    allowed = {
        "schema",
        "task_id",
        "product",
        "dependencies",
        "base",
        "candidate_tree_sha",
        "integration_commit",
        "run_id",
        "receipt_sha256",
        "evidence",
        "result",
        "reviewer",
        "verify_exit",
        "verify_last_line",
        "recorded_at",
    }
    _unknown(receipt, allowed, field)
    _required(
        receipt,
        (
            "schema",
            "task_id",
            "product",
            "dependencies",
            "base",
            "candidate_tree_sha",
            "integration_commit",
            "run_id",
            "receipt_sha256",
            "evidence",
            "result",
            "reviewer",
            "recorded_at",
        ),
        field,
    )
    if receipt["schema"] != RECEIPT_SCHEMA:
        _reject(f"{field}.schema is not {RECEIPT_SCHEMA}")
    _task_id(receipt["task_id"], f"{field}.task_id")
    _string(receipt["product"], f"{field}.product")
    _task_dependencies(receipt["dependencies"], f"{field}.dependencies")
    _full_sha(receipt["base"], f"{field}.base")
    _full_sha(receipt["candidate_tree_sha"], f"{field}.candidate_tree_sha")
    _full_sha(receipt["integration_commit"], f"{field}.integration_commit")
    _absolute_path(receipt["run_id"], f"{field}.run_id")
    _sha256(receipt["receipt_sha256"], f"{field}.receipt_sha256")
    _relative_path(receipt["evidence"], f"{field}.evidence")
    _validate_result_shape(
        _mapping(receipt["result"], f"{field}.result"), f"{field}.result"
    )
    _validate_reviewer_shape(
        _mapping(receipt["reviewer"], f"{field}.reviewer"), f"{field}.reviewer"
    )
    if "verify_exit" in receipt:
        if type(receipt["verify_exit"]) is not int or receipt["verify_exit"] != 0:
            _reject(f"{field}.verify_exit must be 0")
    if "verify_last_line" in receipt and receipt["verify_last_line"] != "DONE":
        _reject(f"{field}.verify_last_line must be DONE")
    _timestamp(receipt["recorded_at"], f"{field}.recorded_at")


def _validate_task_row_shape(row: Mapping[str, Any], field: str) -> None:
    allowed = {
        "task_id",
        "status",
        "parent_sha",
        "candidate_tree_sha",
        "verifier_verdict",
        "reviewer_verdict",
        "receipt_key",
        "dependencies",
        "run_id",
        "integrated_at",
        "notes",
        "driver_evidence",
        "verify_evidence",
    }
    _unknown(row, allowed, field)
    _required(row, ("task_id", "status"), field)
    task_id = _task_id(row["task_id"], f"{field}.task_id")
    if row["status"] not in TASK_STATUSES:
        _reject(f"{field}.status is not a known task status")
    for name in ("parent_sha", "candidate_tree_sha"):
        if name in row:
            _full_sha(row[name], f"{field}.{name}")
    if "verifier_verdict" in row and row["verifier_verdict"] not in VERDICTS:
        _reject(f"{field}.verifier_verdict is not a known verifier verdict")
    if "reviewer_verdict" in row and row["reviewer_verdict"] not in VERDICTS:
        _reject(f"{field}.reviewer_verdict is not a known reviewer verdict")
    if "receipt_key" in row:
        key = _string(row["receipt_key"], f"{field}.receipt_key")
        if _RECEIPT_KEY.fullmatch(key) is None:
            _reject(f"{field}.receipt_key is not a task receipt key")
    if "dependencies" in row:
        _task_dependencies(row["dependencies"], f"{field}.dependencies")
    if "run_id" in row:
        _absolute_path(row["run_id"], f"{field}.run_id")
    if "integrated_at" in row:
        _timestamp(row["integrated_at"], f"{field}.integrated_at")
    for name in ("notes", "driver_evidence", "verify_evidence"):
        if name in row:
            _string(row[name], f"{field}.{name}")
    if row["status"] in ACCEPTED_TASK_STATUSES:
        _required(
            row,
            (
                "parent_sha",
                "candidate_tree_sha",
                "verifier_verdict",
                "reviewer_verdict",
                "receipt_key",
                "dependencies",
                "run_id",
            ),
            field,
        )
        if row["status"] == "integrated" and "integrated_at" not in row:
            _reject(f"{field}.integrated_at is required for integrated rows")
        if row["verifier_verdict"] != ACCEPTED_VERIFIER_VERDICT:
            _reject(f"{field}.verifier_verdict is not accepted")
        if row["reviewer_verdict"] != ACCEPTED_REVIEWER_VERDICT:
            _reject(f"{field}.reviewer_verdict is not accepted")
        if row["receipt_key"] != _receipt_key_for(task_id):
            _reject(f"{field}.receipt_key does not bind task_id")


def validate_ledger_schema(ledger: Mapping[str, Any]) -> None:
    """Validate the JSON shape before any ledger value can authorize work."""

    ledger = _mapping(ledger, "ledger")
    allowed = {
        "schema",
        "integration_ref",
        "integration_head",
        "armed",
        "armed_at",
        "catalog",
        "toolchain",
        "receipts",
        "tasks",
        PREPARATION_RECORD_KEY,
        "notes",
    }
    _unknown(ledger, allowed, "ledger")
    _required(
        ledger,
        (
            "schema",
            "integration_ref",
            "integration_head",
            "armed",
            "catalog",
            "toolchain",
            "receipts",
            "tasks",
        ),
        "ledger",
    )
    if ledger["schema"] != LEDGER_SCHEMA:
        _reject(f"ledger.schema is not {LEDGER_SCHEMA}")
    _string(ledger["integration_ref"], "ledger.integration_ref")
    _full_sha(ledger["integration_head"], "ledger.integration_head")
    if type(ledger["armed"]) is not bool:
        _reject("ledger.armed must be boolean")
    if "armed_at" in ledger:
        _timestamp(ledger["armed_at"], "ledger.armed_at")
    if "notes" in ledger:
        _string(ledger["notes"], "ledger.notes")
    if PREPARATION_RECORD_KEY in ledger:
        _validate_preparation_qualification_shape(
            _mapping(ledger[PREPARATION_RECORD_KEY], "ledger.preparation"),
            "ledger.preparation",
        )

    catalog = _mapping(ledger["catalog"], "ledger.catalog")
    _unknown(
        catalog,
        {"commit", "tree", "branch", "manifest", "recorded_at"},
        "ledger.catalog",
    )
    _required(
        catalog,
        ("commit", "tree", "branch", "manifest", "recorded_at"),
        "ledger.catalog",
    )
    _full_sha(catalog["commit"], "ledger.catalog.commit")
    _tree_sha(catalog["tree"], "ledger.catalog.tree")
    _string(catalog["branch"], "ledger.catalog.branch")
    manifest = _mapping(catalog["manifest"], "ledger.catalog.manifest")
    _unknown(manifest, {"path", "sha256"}, "ledger.catalog.manifest")
    _required(manifest, ("path", "sha256"), "ledger.catalog.manifest")
    _relative_path(manifest["path"], "ledger.catalog.manifest.path")
    _sha256(manifest["sha256"], "ledger.catalog.manifest.sha256")
    _timestamp(catalog["recorded_at"], "ledger.catalog.recorded_at")

    toolchain = _mapping(ledger["toolchain"], "ledger.toolchain")
    toolchain_allowed = {
        "architectural_main",
        "oracle_commit",
        "visual_baseline_tag_peeled",
        "taskfmt_revision",
        "taskfmt_version",
        "taskfmt_sha256",
        "taskfmt_source",
        "taskfmt_path",
        "tuisnap_path",
    }
    _unknown(toolchain, toolchain_allowed, "ledger.toolchain")
    _required(
        toolchain,
        (
            "taskfmt_revision",
            "taskfmt_version",
            "taskfmt_sha256",
            "taskfmt_source",
            "taskfmt_path",
        ),
        "ledger.toolchain",
    )
    for name in ("architectural_main", "oracle_commit", "visual_baseline_tag_peeled"):
        if name in toolchain and toolchain[name]:
            _full_sha(toolchain[name], f"ledger.toolchain.{name}")
    _full_sha(toolchain["taskfmt_revision"], "ledger.toolchain.taskfmt_revision")
    _string(toolchain["taskfmt_version"], "ledger.toolchain.taskfmt_version")
    _sha256(toolchain["taskfmt_sha256"], "ledger.toolchain.taskfmt_sha256")
    _absolute_path(toolchain["taskfmt_source"], "ledger.toolchain.taskfmt_source")
    _absolute_path(toolchain["taskfmt_path"], "ledger.toolchain.taskfmt_path")
    if "tuisnap_path" in toolchain and toolchain["tuisnap_path"]:
        _absolute_path(toolchain["tuisnap_path"], "ledger.toolchain.tuisnap_path")

    receipts = _mapping(ledger["receipts"], "ledger.receipts")
    for key, receipt in receipts.items():
        if not isinstance(key, str) or _RECEIPT_KEY.fullmatch(key) is None:
            _reject(f"ledger.receipts has invalid key {key!r}")
        receipt = _mapping(receipt, f"ledger.receipts.{key}")
        # Blocked historical rows may retain old receipts. They are never
        # accepted: an accepted row must carry the strict v2 receipt below.
        if "schema" in receipt:
            _validate_receipt_shape(receipt, f"ledger.receipts.{key}")

    tasks = ledger["tasks"]
    if not isinstance(tasks, list):
        _reject("ledger.tasks must be an array")
    seen: set[str] = set()
    for index, row in enumerate(tasks):
        row = _mapping(row, f"ledger.tasks[{index}]")
        _validate_task_row_shape(row, f"ledger.tasks[{index}]")
        task_id = row["task_id"]
        if task_id in seen:
            _reject(f"ledger.tasks contains duplicate {task_id}")
        seen.add(task_id)


def validate_taskfmt_binding(
    ledger: Mapping[str, Any], expected_taskfmt: Mapping[str, str]
) -> None:
    """Bind the ledger to the currently qualified standalone taskfmt."""

    ledger = _mapping(ledger, "ledger")
    toolchain = _mapping(ledger.get("toolchain"), "ledger.toolchain")
    required = {
        "taskfmt_revision",
        "taskfmt_version",
        "taskfmt_sha256",
        "taskfmt_source",
        "taskfmt_path",
    }
    missing = sorted(required - set(expected_taskfmt))
    if missing:
        _reject(f"expected taskfmt identity is missing: {', '.join(missing)}")
    for name in sorted(required):
        if toolchain.get(name) != expected_taskfmt[name]:
            _reject(
                f"ledger.toolchain.{name} does not bind current taskfmt "
                f"({toolchain.get(name)!r} != {expected_taskfmt[name]!r})"
            )


def _validate_accepted_binding(
    row: Mapping[str, Any], receipt: Mapping[str, Any], *, field: str
) -> None:
    """Check all duplicated identities before filesystem/Git checks."""

    _validate_task_row_shape(row, field)
    _validate_receipt_shape(receipt, f"{field}.receipt")
    task_id = row["task_id"]
    comparisons = (
        (receipt["task_id"], task_id, "task_id"),
        (receipt["base"], row["parent_sha"], "base/parent_sha"),
        (receipt["candidate_tree_sha"], row["candidate_tree_sha"], "candidate_tree_sha"),
        (receipt["run_id"], row["run_id"], "run_id"),
        (receipt["dependencies"], row["dependencies"], "dependencies"),
        (receipt["reviewer"]["verdict"], row["reviewer_verdict"], "reviewer_verdict"),
    )
    for actual, expected, name in comparisons:
        if actual != expected:
            _reject(f"{field} receipt {name} is not bound to the task row")

    result = receipt["result"]
    reviewer = receipt["reviewer"]
    for source, name in ((result, "result"), (reviewer, "reviewer")):
        for key in ("task_id", "base", "candidate_tree_sha", "integration_commit", "run_id"):
            if source[key] != receipt[key]:
                _reject(f"{field}.{name}.{key} is not bound to the receipt")
    if reviewer["result_sha256"] != result["sha256"]:
        _reject(f"{field}.reviewer.result_sha256 is not bound to result")
    if "verify_exit" in receipt and receipt["verify_exit"] != result["exit"]:
        _reject(f"{field}.verify_exit is not bound to result")
    if "verify_last_line" in receipt and receipt["verify_last_line"] != result["last_line"]:
        _reject(f"{field}.verify_last_line is not bound to result")


def _dependency_map(value: Mapping[str, Any]) -> dict[str, list[str]]:
    value = _mapping(value, "dependency graph")
    if "tasks" in value:
        if value.get("schema") != "tc-plan-graph/v1":
            _reject("dependency graph schema is not tc-plan-graph/v1")
        value = _mapping(value["tasks"], "dependency graph.tasks")
    result: dict[str, list[str]] = {}
    for task_id, record in value.items():
        _task_id(task_id, "dependency graph task_id")
        record = _mapping(record, f"dependency graph.{task_id}")
        if "dependencies" not in record:
            _reject(f"dependency graph.{task_id} has no dependencies")
        result[task_id] = _task_dependencies(
            record["dependencies"], f"dependency graph.{task_id}.dependencies"
        )
    for task_id, dependencies in result.items():
        for dependency in dependencies:
            if dependency not in result:
                _reject(f"dependency graph references unknown {dependency}")

    visiting: set[str] = set()
    visited: set[str] = set()

    def visit(task_id: str) -> None:
        if task_id in visiting:
            _reject(f"dependency graph contains a cycle at {task_id}")
        if task_id in visited:
            return
        visiting.add(task_id)
        for dependency in result[task_id]:
            visit(dependency)
        visiting.remove(task_id)
        visited.add(task_id)

    for task_id in result:
        visit(task_id)
    return result


def _is_ancestor(repository_root: Path, ancestor: str, descendant: str) -> bool:
    if ancestor == descendant:
        return True
    completed = subprocess.run(
        [
            "git",
            "-C",
            str(repository_root),
            "merge-base",
            "--is-ancestor",
            ancestor,
            descendant,
        ],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=False,
    )
    return completed.returncode == 0


def _regular_file(path: Path, field: str) -> Path:
    if path.is_symlink() or not path.is_file():
        _reject(f"{field} is not a non-symlink regular file: {path}")
    return path


def _read_json(path: Path, field: str) -> Mapping[str, Any]:
    _regular_file(path, field)
    try:
        with path.open(encoding="utf-8") as stream:
            value = json.load(stream)
    except (OSError, json.JSONDecodeError) as error:
        _reject(f"{field} is not valid JSON: {error}")
    return _mapping(value, field)


def _file_sha256(path: Path, field: str) -> str:
    _regular_file(path, field)
    try:
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
    except OSError as error:
        _reject(f"cannot hash {field}: {error}")
    return digest


def _canonical_json_sha256(value: Any) -> str:
    """Match Rust ``tc-proof`` canonical JSON hashing."""

    raw = json.dumps(
        value,
        ensure_ascii=False,
        sort_keys=True,
        separators=(",", ":"),
    ).encode("utf-8")
    return hashlib.sha256(raw).hexdigest()


def _validate_executable_identity(
    value: Mapping[str, Any],
    *,
    expected_path: Path,
    field: str,
) -> None:
    """Validate a proof context executable binding at the receipt boundary."""

    _unknown(value, {"path", "sha256"}, field)
    _required(value, ("path", "sha256"), field)
    path = _qualification_file(
        Path(_absolute_path(value["path"], f"{field}.path")),
        f"{field}.path",
    )
    if not os.access(path, os.X_OK):
        _reject(f"{field}.path is not executable")
    _sha256(value["sha256"], f"{field}.sha256")
    if path.resolve() != expected_path.resolve():
        _reject(f"{field}.path is not the expected executable")
    if _file_sha256(path, f"{field}.path") != value["sha256"]:
        _reject(f"{field}.sha256 does not match the executable")


def _under(path: Path, parent: Path, field: str) -> Path:
    try:
        path.resolve().relative_to(parent.resolve())
    except ValueError:
        _reject(f"{field} escapes its bound directory")
    return path


def _validate_receipt_files(
    receipt: Mapping[str, Any], repository_root: Path, *, field: str
) -> None:
    evidence_path = _under(
        repository_root / receipt["evidence"], repository_root, f"{field}.evidence"
    )
    if _file_sha256(evidence_path, f"{field}.evidence") != receipt["receipt_sha256"]:
        _reject(f"{field}.evidence hash does not match receipt_sha256")
    evidence = _read_json(evidence_path, f"{field}.evidence")
    _unknown(
        evidence,
        {
            "schema",
            "task_id",
            "base",
            "candidate_tree_sha",
            "integration_commit",
            "run_id",
            "result_sha256",
            "reviewer_evidence_sha256",
        },
        f"{field}.evidence",
    )
    _required(
        evidence,
        (
            "schema",
            "task_id",
            "base",
            "candidate_tree_sha",
            "integration_commit",
            "run_id",
            "result_sha256",
            "reviewer_evidence_sha256",
        ),
        f"{field}.evidence",
    )
    if evidence["schema"] != EVIDENCE_SCHEMA:
        _reject(f"{field}.evidence.schema is not {EVIDENCE_SCHEMA}")
    for key in ("task_id", "base", "candidate_tree_sha", "integration_commit", "run_id"):
        if evidence[key] != receipt[key]:
            _reject(f"{field}.evidence.{key} is not bound to receipt")
    if evidence["result_sha256"] != receipt["result"]["sha256"]:
        _reject(f"{field}.evidence.result_sha256 is not bound to result")
    if evidence["reviewer_evidence_sha256"] != receipt["reviewer"]["evidence_sha256"]:
        _reject(f"{field}.evidence.reviewer_evidence_sha256 is not bound to reviewer")

    run_dir = Path(receipt["run_id"])
    if run_dir.is_symlink() or not run_dir.is_dir():
        _reject(f"{field}.run_id is not a real run directory")
    result_path = _under(
        Path(receipt["result"]["path"]), run_dir, f"{field}.result.path"
    )
    if _file_sha256(result_path, f"{field}.result.path") != receipt["result"]["sha256"]:
        _reject(f"{field}.result.path hash does not match result.sha256")
    result = _read_json(result_path, f"{field}.result.path")
    _validate_result_shape(
        {
            **result,
            "path": receipt["result"]["path"],
            "sha256": receipt["result"]["sha256"],
        },
        f"{field}.result.file",
    )
    for key in (
        "task_id",
        "base",
        "candidate_tree_sha",
        "integration_commit",
        "run_id",
    ):
        if result[key] != receipt[key]:
            _reject(f"{field}.result.file.{key} is not bound to receipt")

    reviewer_path = _under(
        Path(receipt["reviewer"]["evidence"]),
        run_dir,
        f"{field}.reviewer.evidence",
    )
    if (
        _file_sha256(reviewer_path, f"{field}.reviewer.evidence")
        != receipt["reviewer"]["evidence_sha256"]
    ):
        _reject(f"{field}.reviewer.evidence hash does not match reviewer")
    reviewer = _read_json(reviewer_path, f"{field}.reviewer.evidence")
    _unknown(
        reviewer,
        {
            "schema",
            "task_id",
            "base",
            "candidate_tree_sha",
            "integration_commit",
            "run_id",
            "verdict",
            "result_sha256",
            "recorded_at",
        },
        f"{field}.reviewer.evidence",
    )
    _required(
        reviewer,
        (
            "schema",
            "task_id",
            "base",
            "candidate_tree_sha",
            "integration_commit",
            "run_id",
            "verdict",
            "result_sha256",
            "recorded_at",
        ),
        f"{field}.reviewer.evidence",
    )
    if reviewer["schema"] != REVIEW_SCHEMA:
        _reject(f"{field}.reviewer.evidence.schema is not {REVIEW_SCHEMA}")
    for key in (
        "task_id",
        "base",
        "candidate_tree_sha",
        "integration_commit",
        "run_id",
        "verdict",
    ):
        if reviewer[key] != receipt["reviewer"][key]:
            _reject(f"{field}.reviewer.evidence.{key} is not bound to reviewer")
    if reviewer["verdict"] != ACCEPTED_REVIEWER_VERDICT:
        _reject(f"{field}.reviewer.evidence.verdict is not accepted")
    if reviewer["result_sha256"] != receipt["result"]["sha256"]:
        _reject(f"{field}.reviewer.evidence.result_sha256 is not bound to result")
    _timestamp(reviewer["recorded_at"], f"{field}.reviewer.evidence.recorded_at")


def accepted_verifier_rows(ledger: Mapping[str, Any]) -> list[dict[str, Any]]:
    """Return only rows with a structurally bound verifier and reviewer receipt."""

    if not isinstance(ledger, Mapping):
        return []
    rows = ledger.get("tasks", [])
    receipts = ledger.get("receipts", {})
    if not isinstance(rows, list) or not isinstance(receipts, Mapping):
        return []
    accepted: list[dict[str, Any]] = []
    for row in rows:
        if not isinstance(row, Mapping):
            continue
        if row.get("status") not in ACCEPTED_TASK_STATUSES:
            continue
        if row.get("verifier_verdict") != ACCEPTED_VERIFIER_VERDICT:
            continue
        task_id = row.get("task_id")
        if not isinstance(task_id, str):
            continue
        receipt = receipts.get(_receipt_key_for(task_id))
        if not isinstance(receipt, Mapping):
            continue
        try:
            _validate_accepted_binding(row, receipt, field=f"task {task_id}")
        except LedgerValidationError:
            continue
        accepted.append(dict(row))
    return accepted


def _qualification_ref_file(
    reference: Mapping[str, Any],
    *,
    field: str,
    run_dir: Path | None = None,
    repository_root: Path | None = None,
) -> tuple[Path, Mapping[str, Any]]:
    """Validate and hash one immutable external qualification artifact."""

    reference = _mapping(reference, field)
    reference_shape = {
        "path": reference.get("path"),
        "sha256": reference.get("sha256"),
    }
    _path_ref_shape(reference_shape, field)
    path = Path(reference["path"])
    if ".." in path.parts:
        _reject(f"{field}.path contains '..'")
    file_path = _qualification_file(path, f"{field}.path")
    resolved = file_path.resolve()
    if run_dir is not None:
        try:
            resolved.relative_to(run_dir.resolve())
        except ValueError:
            _reject(f"{field}.path is outside its run directory")
    if repository_root is not None and resolved == repository_root.resolve():
        _reject(f"{field}.path is the repository root")
    actual_sha = _file_sha256(file_path, f"{field}.path")
    if actual_sha != reference["sha256"]:
        _reject(f"{field}.path hash does not match sha256")
    return file_path, reference


def _qualification_run(
    value: Any,
    *,
    field: str,
    candidate: Path,
) -> Path:
    path = Path(_absolute_path(value, field))
    if ".." in path.parts:
        _reject(f"{field} contains '..'")
    _qualification_directory(path, field)
    resolved = path.resolve()
    try:
        resolved.relative_to(candidate.resolve())
    except ValueError:
        return resolved
    _reject(f"{field} must be external to the candidate worktree")


def _git_identity(repository_root: Path, ref: str, field: str) -> dict[str, str]:
    values: dict[str, str] = {}
    for suffix, name in (("^{commit}", "commit"), ("^{tree}", "tree")):
        completed = subprocess.run(
            ["git", "-C", str(repository_root), "rev-parse", f"{ref}{suffix}"],
            capture_output=True,
            text=True,
            check=False,
        )
        if completed.returncode != 0:
            _reject(f"cannot resolve {field} {ref}{suffix}")
        values[name] = _full_sha(completed.stdout.strip(), f"{field}.{name}")
    return values


def _identity_expected(
    expected: Mapping[str, Any] | None,
    *,
    field: str,
    current_head: str,
    current_tree: str,
) -> dict[str, Any]:
    if expected is None:
        return {"commit": current_head, "tree": current_tree}
    expected = _mapping(expected, f"expected {field}")
    _required(expected, ("commit", "tree"), f"expected {field}")
    _full_sha(expected["commit"], f"expected {field}.commit")
    _tree_sha(expected["tree"], f"expected {field}.tree")
    if "manifest" in expected:
        manifest = _mapping(expected["manifest"], f"expected {field}.manifest")
        _unknown(manifest, {"path", "sha256"}, f"expected {field}.manifest")
        _required(manifest, ("path", "sha256"), f"expected {field}.manifest")
        _relative_path(manifest["path"], f"expected {field}.manifest.path")
        _sha256(manifest["sha256"], f"expected {field}.manifest.sha256")
    return dict(expected)


def _validate_preparation_identity(
    actual: Mapping[str, Any],
    expected: Mapping[str, Any],
    *,
    field: str,
    repository_root: Path,
) -> None:
    for name in ("commit", "tree"):
        if actual[name] != expected[name]:
            _reject(f"{field}.{name} is not bound to the current identity")
    manifest = _mapping(actual["manifest"], f"{field}.manifest")
    if "manifest" in expected and manifest != expected["manifest"]:
        _reject(f"{field}.manifest is not bound to the current identity")
    manifest_path = repository_root / _relative_path(
        manifest["path"], f"{field}.manifest.path"
    )
    _qualification_file(manifest_path, f"{field}.manifest.path")
    if _file_sha256(manifest_path, f"{field}.manifest.path") != manifest["sha256"]:
        _reject(f"{field}.manifest hash does not match the current file")


def _validate_ledger_catalog(
    catalog: Mapping[str, Any],
    *,
    expected: Mapping[str, Any],
    integration_branch: str,
    repository_root: Path,
) -> None:
    """Bind the operator catalog row to the exact current candidate catalog."""

    if catalog["branch"] != integration_branch:
        _reject("ledger.catalog.branch is not the integration branch")
    actual = {
        "commit": catalog["commit"],
        "tree": catalog["tree"],
        "manifest": catalog["manifest"],
    }
    _validate_preparation_identity(
        actual,
        expected,
        field="ledger.catalog",
        repository_root=repository_root,
    )


def _common_preparation_evidence(
    evidence: Mapping[str, Any],
    *,
    schema: str,
    field: str,
    record: Mapping[str, Any],
    proof_run_id: str,
    proof_hashes: Mapping[str, str],
    scope_base: str,
    decision_run_id: str,
) -> None:
    allowed = {
        "schema",
        "verdict",
        "run_id",
        "proof_run_id",
        "candidate_commit",
        "candidate_tree",
        "integration_ref",
        "oracle",
        "catalog",
        "task_graph",
        "taskfmt",
        "proof_preparation_sha256",
        "context_index_sha256",
        "observer_sha256",
        "scope_base",
        "recorded_at",
        "verifier_run_id",
        "verifier_evidence_sha256",
    }
    _unknown(evidence, allowed, field)
    required = {
        "schema",
        "verdict",
        "run_id",
        "proof_run_id",
        "candidate_commit",
        "candidate_tree",
        "integration_ref",
        "oracle",
        "catalog",
        "task_graph",
        "taskfmt",
        "proof_preparation_sha256",
        "context_index_sha256",
        "observer_sha256",
        "scope_base",
        "recorded_at",
    }
    _required(evidence, tuple(sorted(required)), field)
    if evidence["schema"] != schema:
        _reject(f"{field}.schema is not {schema}")
    if evidence["verdict"] != ACCEPTED_VERIFIER_VERDICT:
        _reject(f"{field}.verdict is not VERIFIED")
    _absolute_path(evidence["run_id"], f"{field}.run_id")
    _absolute_path(evidence["proof_run_id"], f"{field}.proof_run_id")
    _full_sha(evidence["candidate_commit"], f"{field}.candidate_commit")
    _tree_sha(evidence["candidate_tree"], f"{field}.candidate_tree")
    _string(evidence["integration_ref"], f"{field}.integration_ref")
    _full_sha(evidence["scope_base"], f"{field}.scope_base")
    for name in ("proof_preparation_sha256", "context_index_sha256", "observer_sha256"):
        _sha256(evidence[name], f"{field}.{name}")
    _timestamp(evidence["recorded_at"], f"{field}.recorded_at")
    if evidence["run_id"] != decision_run_id:
        _reject(f"{field}.run_id is not bound to its ledger record")
    if evidence["proof_run_id"] != proof_run_id:
        _reject(f"{field}.proof_run_id is not bound to proof preparation")
    if evidence["candidate_commit"] != record["candidate_commit"]:
        _reject(f"{field}.candidate_commit is stale")
    if evidence["candidate_tree"] != record["candidate_tree"]:
        _reject(f"{field}.candidate_tree is stale")
    if evidence["integration_ref"] != record["integration_ref"]:
        _reject(f"{field}.integration_ref is stale")
    if evidence["oracle"] != record["oracle"]:
        _reject(f"{field}.oracle is not bound to the ledger")
    if evidence["catalog"] != record["catalog"]:
        _reject(f"{field}.catalog is not bound to the ledger")
    if evidence["task_graph"] != record["task_graph"]:
        _reject(f"{field}.task_graph is not bound to the ledger")
    if evidence["taskfmt"] != record["taskfmt"]:
        _reject(f"{field}.taskfmt is not bound to the ledger")
    if evidence["scope_base"] != scope_base:
        _reject(f"{field}.scope_base is not bound to proof preparation")
    if evidence["proof_preparation_sha256"] != proof_hashes["preparation"]:
        _reject(f"{field}.proof_preparation_sha256 is stale")
    if evidence["context_index_sha256"] != proof_hashes["context_index"]:
        _reject(f"{field}.context_index_sha256 is stale")
    if evidence["observer_sha256"] != proof_hashes["observer"]:
        _reject(f"{field}.observer_sha256 is stale")


def _validate_external_proof_preparation(
    proof: Mapping[str, Any],
    *,
    candidate: Path,
    candidate_commit: str,
    candidate_tree: str,
    expected_taskfmt: Mapping[str, str],
    repository_root: Path,
) -> tuple[dict[str, str], str]:
    """Validate the proof-preparation ABI and return its hashes/scope base."""

    run_dir = _qualification_run(proof["run_id"], field="proof_preparation.run_id", candidate=candidate)
    task_id = _task_id(proof["task_id"], "proof_preparation.task_id")
    ref_paths: set[Path] = set()

    def ref(name: str) -> tuple[Path, Mapping[str, Any]]:
        path, value = _qualification_ref_file(
            _mapping(proof[name], f"proof_preparation.{name}"),
            field=f"proof_preparation.{name}",
            run_dir=run_dir,
            repository_root=repository_root,
        )
        if path in ref_paths:
            _reject(f"proof preparation reuses a path for {name}")
        ref_paths.add(path)
        return path, value

    preparation_path, preparation_ref = ref("preparation_receipt")
    preparation = _read_json(preparation_path, "proof_preparation.preparation_receipt")
    _validate_preparation_file(
        preparation,
        worktree=candidate,
        current_head=candidate_commit,
        current_tree=candidate_tree,
        run_dir=run_dir,
        expected_taskfmt=expected_taskfmt,
    )
    if preparation.get("schema") != PROOF_PREPARATION_SCHEMA:
        _reject("proof preparation receipt has the wrong schema")
    if preparation.get("task_id") != task_id:
        _reject("proof preparation receipt task mismatch")
    if preparation.get("worktree") != str(candidate.resolve()):
        _reject("proof preparation receipt worktree mismatch")
    if preparation.get("commit") != candidate_commit:
        _reject("proof preparation receipt commit mismatch")
    if preparation.get("run_id") != str(run_dir.resolve()):
        _reject("proof preparation receipt run mismatch")
    _full_sha(preparation.get("scope_base"), "proof_preparation.scope_base")
    scope_base = preparation["scope_base"]
    validate_taskfmt_binding(
        {"toolchain": _mapping(preparation.get("taskfmt"), "proof_preparation.taskfmt")},
        expected_taskfmt,
    )

    native = _mapping(preparation.get("native_build"), "proof_preparation.native_build")
    _required(native, ("receipt", "receipt_sha256", "binary", "binary_sha256"), "proof_preparation.native_build")
    receipt_path = Path(_absolute_path(native["receipt"], "proof_preparation.native_build.receipt"))
    binary_path = Path(_absolute_path(native["binary"], "proof_preparation.native_build.binary"))
    for path, name, digest in (
        (receipt_path, "native_build.receipt", native["receipt_sha256"]),
        (binary_path, "native_build.binary", native["binary_sha256"]),
    ):
        if ".." in path.parts:
            _reject(f"proof_preparation.{name} contains '..'")
        _qualification_file(path, f"proof_preparation.{name}")
        try:
            path.resolve().relative_to(run_dir.resolve())
        except ValueError:
            _reject(f"proof_preparation.{name} is outside the proof run")
        _sha256(digest, f"proof_preparation.native_build.{name}_sha256")
        if _file_sha256(path, f"proof_preparation.{name}") != digest:
            _reject(f"proof_preparation.{name} hash mismatch")
        if path in ref_paths:
            _reject(f"proof preparation reuses a path for {name}")
        ref_paths.add(path)
    build = _read_json(receipt_path, "proof_preparation.native_build.receipt")
    _unknown(
        build,
        {
            "schema",
            "worktree",
            "target_dir",
            "cargo_target_dir",
            "commit",
            "tree",
            "binary",
            "binary_sha256",
            "command",
        },
        "proof_preparation.native_build.receipt",
    )
    _required(
        build,
        ("schema", "worktree", "target_dir", "commit", "tree", "binary", "binary_sha256"),
        "proof_preparation.native_build.receipt",
    )
    if build["schema"] != NATIVE_BUILD_SCHEMA:
        _reject("proof preparation native build schema mismatch")
    build_worktree = Path(_absolute_path(build["worktree"], "native_build.worktree"))
    _qualification_directory(build_worktree, "native_build.worktree")
    if build_worktree.resolve() != candidate.resolve() or build["commit"] != candidate_commit:
        _reject("proof preparation native build source mismatch")
    if build["tree"] != candidate_tree:
        _reject("proof preparation native build tree mismatch")
    target = Path(_absolute_path(build["target_dir"], "native_build.target_dir"))
    _qualification_directory(target, "native_build.target_dir")
    target = target.resolve()
    try:
        target.relative_to(run_dir.resolve())
    except ValueError:
        _reject("native build target is outside the proof run")
    if build.get("cargo_target_dir") != build["target_dir"]:
        _reject("native build Cargo target mismatch")
    _qualification_directory(target / "debug", "native_build.target_dir/debug")
    if binary_path.resolve() != target / "debug" / "tc-proof":
        _reject("native build binary does not match target/debug/tc-proof")
    if receipt_path.resolve() != target / "debug" / "tc-proof.build.json":
        _reject("native build receipt path does not match target/debug/tc-proof.build.json")
    build_binary = _qualification_file_under(
        Path(_absolute_path(build["binary"], "native_build.binary")),
        run_dir,
        "native_build.binary",
    )
    if build_binary.resolve() != binary_path.resolve():
        _reject("native build binary path mismatch")
    if build["binary_sha256"] != native["binary_sha256"]:
        _reject("native build binary hash is not bound")
    if _file_sha256(binary_path, "native_build.binary") != native["binary_sha256"]:
        _reject("native build binary has mutated")

    index_path, index_ref = ref("context_index")
    if preparation.get("context_index") != dict(index_ref):
        _reject("proof preparation context index reference mismatch")
    index = _read_json(index_path, "proof_preparation.context_index")
    _unknown(
        index,
        {"schema", "task_id", "run_id", "worktree_commit", "scope_base", "contexts", "results", "observer"},
        "proof_preparation.context_index",
    )
    _required(
        index,
        ("schema", "task_id", "run_id", "worktree_commit", "scope_base", "contexts", "results", "observer"),
        "proof_preparation.context_index",
    )
    if index["schema"] != CONTEXT_INDEX_SCHEMA:
        _reject("proof preparation context index schema mismatch")
    if (
        index["task_id"] != task_id
        or index["run_id"] != str(run_dir.resolve())
        or index["worktree_commit"] != candidate_commit
        or index["scope_base"] != scope_base
    ):
        _reject("proof preparation context index identity mismatch")

    observer_path, observer_ref = ref("observer")
    if preparation.get("observer") != dict(observer_ref) or index.get("observer") != dict(observer_ref):
        _reject("proof preparation observer reference mismatch")
    observer = _read_json(observer_path, "proof_preparation.observer")
    _unknown(
        observer,
        {"schema", "task_id", "run_id", "worktree_commit", "scope_base", "transport", "nonce_sha256"},
        "proof_preparation.observer",
    )
    _required(
        observer,
        ("schema", "task_id", "run_id", "worktree_commit", "scope_base", "transport", "nonce_sha256"),
        "proof_preparation.observer",
    )
    if observer["schema"] != OBSERVER_SCHEMA or observer["transport"] != "inherited-pipe/v1":
        _reject("proof preparation observer schema/transport mismatch")
    if (
        observer["task_id"] != task_id
        or observer["run_id"] != str(run_dir.resolve())
        or observer["worktree_commit"] != candidate_commit
        or observer["scope_base"] != scope_base
    ):
        _reject("proof preparation observer identity mismatch")
    _sha256(observer["nonce_sha256"], "proof preparation observer nonce")

    def entry_map(value: Any, field: str) -> dict[str, Mapping[str, Any]]:
        if not isinstance(value, list) or not value:
            _reject(f"{field} must be a non-empty array")
        result: dict[str, Mapping[str, Any]] = {}
        for index_number, item in enumerate(value):
            item = _mapping(item, f"{field}[{index_number}]")
            _unknown(item, {"check_id", "path", "sha256"}, f"{field}[{index_number}]")
            _required(item, ("check_id", "path", "sha256"), f"{field}[{index_number}]")
            check_id = _string(item["check_id"], f"{field}[{index_number}].check_id")
            if re.fullmatch(r"CHK-[0-9]{3}", check_id) is None:
                _reject(f"{field}[{index_number}].check_id is invalid")
            if check_id in result:
                _reject(f"{field} contains duplicate {check_id}")
            _absolute_path(item["path"], f"{field}[{index_number}].path")
            _sha256(item["sha256"], f"{field}[{index_number}].sha256")
            result[check_id] = item
        return result

    prep_contexts = entry_map(preparation.get("contexts"), "proof_preparation.contexts")
    prep_results = entry_map(preparation.get("results"), "proof_preparation.results")
    index_contexts = entry_map(index["contexts"], "proof_preparation.context_index.contexts")
    index_results = entry_map(index["results"], "proof_preparation.context_index.results")
    if set(prep_contexts) != set(prep_results) or set(prep_contexts) != set(index_contexts) or set(prep_results) != set(index_results):
        _reject("proof preparation context/result check sets differ")
    top_contexts = entry_map(proof["contexts"], "ledger.preparation.proof_preparation.contexts")
    top_results = entry_map(proof["results"], "ledger.preparation.proof_preparation.results")
    if set(top_contexts) != set(prep_contexts) or set(top_results) != set(prep_results):
        _reject("ledger proof context/result sets differ")

    expected_paths = {"contexts": set(), "results": set()}
    for check_id in sorted(prep_contexts):
        for name, entries, index_entries, top_entries in (
            ("context", prep_contexts, index_contexts, top_contexts),
            ("result", prep_results, index_results, top_results),
        ):
            item = entries[check_id]
            if item != index_entries[check_id] or item != top_entries[check_id]:
                _reject(f"proof preparation {name} reference mismatch for {check_id}")
            path, _ = _qualification_ref_file(
                item,
                field=f"proof_preparation.{name}.{check_id}",
                run_dir=run_dir,
                repository_root=repository_root,
            )
            if path in ref_paths:
                _reject(f"proof preparation reuses a path for {check_id}")
            ref_paths.add(path)
            expected_paths["contexts" if name == "context" else "results"].add(path)
            record = _read_json(path, f"proof_preparation.{name}.{check_id}")
            if name == "context":
                _unknown(
                    record,
                    {
                        "schema",
                        "run_id",
                        "task_id",
                        "check_id",
                        "worktree_commit",
                        "scope_base",
                        "operation",
                        "tree",
                        "oracle_commit",
                        "oracle_tree",
                        "bundle",
                        "bundle_sha256",
                        "tool",
                        "dependencies",
                        "adapter",
                        "lane",
                        "axes",
                        "members",
                        "inventory",
                        "evidence",
                        "configuration",
                        "qualification",
                        "architecture_profile",
                        "branch_host_projection",
                    },
                    f"proof_preparation.context.{check_id}",
                )
                _required(
                    record,
                    ("schema", "task_id", "check_id", "run_id", "worktree_commit", "scope_base", "operation"),
                    f"proof_preparation.context.{check_id}",
                )
                if record["schema"] != CONTEXT_SCHEMA or record["operation"] == "":
                    _reject(f"proof preparation context schema/operation mismatch for {check_id}")
                _validate_proof_context_bindings(
                    record,
                    candidate=candidate,
                    candidate_commit=candidate_commit,
                    candidate_tree=candidate_tree,
                    run_path=run_dir,
                    task_id=task_id,
                    scope_base=scope_base,
                    check_id=check_id,
                    native_binary=Path(
                        _absolute_path(
                            preparation["native_build"]["binary"],
                            "proof_preparation.native_build.binary",
                        )
                    ),
                    taskfmt=preparation["taskfmt"],
                )
            else:
                _required(
                    record,
                    ("schema", "task_id", "check_id", "run_id", "worktree_commit", "scope_base", "context_sha256", "status"),
                    f"proof_preparation.result.{check_id}",
                )
                if record["schema"] != PREPARATION_RESULT_SCHEMA or record["status"] != "ready":
                    _reject(f"proof preparation result status/schema mismatch for {check_id}")
                if record["context_sha256"] != prep_contexts[check_id]["sha256"]:
                    _reject(f"proof preparation result context hash mismatch for {check_id}")
            if (
                record["task_id"] != task_id
                or record["check_id"] != check_id
                or record["run_id"] != str(run_dir.resolve())
                or record["worktree_commit"] != candidate_commit
                or record["scope_base"] != scope_base
            ):
                _reject(f"proof preparation {name} identity mismatch for {check_id}")
    for directory_name, expected in expected_paths.items():
        directory = _qualification_directory(run_dir / directory_name, f"proof_preparation.{directory_name}")
        actual = {
            _qualification_file(path, f"proof_preparation.{directory_name} member").resolve()
            for path in directory.iterdir()
        }
        if actual != {path.resolve() for path in expected}:
            _reject(f"proof preparation {directory_name} contains missing or extra files")

    return {
        "preparation": preparation_ref["sha256"],
        "context_index": index_ref["sha256"],
        "observer": observer_ref["sha256"],
    }, scope_base


def validate_preparation_qualification(
    preparation: Mapping[str, Any],
    *,
    worktree: str | Path,
    current_head: str,
    current_tree: str,
    integration_branch: str,
    expected_oracle: Mapping[str, str] | None,
    expected_taskfmt: Mapping[str, str],
    catalog_identity: Mapping[str, Any] | None = None,
    task_graph_identity: Mapping[str, Any] | None = None,
    repository_root: str | Path | None = None,
    now: datetime | str | None = None,
) -> None:
    """Validate a non-production preparation qualification record.

    A valid record is sufficient for readiness when the ledger has zero
    accepted production rows. It never changes task statuses and never arms
    the campaign.
    """

    _validate_preparation_qualification_shape(preparation)
    current_head = _full_sha(current_head, "current_head")
    current_tree = _tree_sha(current_tree, "current_tree")
    integration_branch = _string(integration_branch, "integration_branch")
    root = Path(_absolute_path(str(repository_root or worktree), "repository_root")).resolve()
    candidate = Path(_absolute_path(str(worktree), "worktree"))
    _qualification_directory(candidate, "worktree")
    if candidate.resolve() != root:
        _reject("worktree and repository_root must identify the same candidate")
    if preparation["integration_ref"] != f"refs/heads/{integration_branch}":
        _reject("preparation integration_ref is not the integration branch")
    if preparation["candidate_commit"] != current_head:
        _reject("preparation candidate commit is stale")
    if preparation["candidate_tree"] != current_tree:
        _reject("preparation candidate tree is stale")

    if expected_oracle is None:
        expected_oracle = {
            "tag": FROZEN_ORACLE_TAG,
            **_git_identity(root, FROZEN_ORACLE_TAG, "frozen oracle"),
        }
    else:
        expected_oracle = dict(expected_oracle)
    _required(expected_oracle, ("tag", "commit", "tree"), "expected frozen oracle")
    _string(expected_oracle["tag"], "expected frozen oracle.tag")
    _full_sha(expected_oracle["commit"], "expected frozen oracle.commit")
    _tree_sha(expected_oracle["tree"], "expected frozen oracle.tree")
    if expected_oracle["tag"] != FROZEN_ORACLE_TAG:
        _reject("preparation oracle tag is not the protected visual-baseline tag")
    if expected_oracle["commit"] != FROZEN_ORACLE_COMMIT:
        _reject("preparation oracle commit is not the protected peeled tag")
    if expected_oracle["tree"] != FROZEN_ORACLE_TREE:
        _reject("preparation oracle tree is not the protected baseline tree")
    if preparation["oracle"] != expected_oracle:
        _reject("preparation oracle identity is stale or substituted")

    expected_catalog = _identity_expected(
        catalog_identity,
        field="catalog",
        current_head=current_head,
        current_tree=current_tree,
    )
    expected_graph = _identity_expected(
        task_graph_identity,
        field="task_graph",
        current_head=current_head,
        current_tree=current_tree,
    )
    _validate_preparation_identity(
        preparation["catalog"], expected_catalog, field="preparation.catalog", repository_root=root
    )
    _validate_preparation_identity(
        preparation["task_graph"], expected_graph, field="preparation.task_graph", repository_root=root
    )
    _validate_taskfmt_record(preparation["taskfmt"], "preparation.taskfmt")
    validate_taskfmt_binding({"toolchain": preparation["taskfmt"]}, expected_taskfmt)

    proof = _mapping(preparation["proof_preparation"], "preparation.proof_preparation")
    proof_hashes, scope_base = _validate_external_proof_preparation(
        proof,
        candidate=candidate,
        candidate_commit=current_head,
        candidate_tree=current_tree,
        expected_taskfmt=expected_taskfmt,
        repository_root=root,
    )
    proof_run = _qualification_run(
        proof["run_id"], field="preparation.proof_preparation.run_id", candidate=candidate
    )
    verifier = _mapping(preparation["verifier"], "preparation.verifier")
    reviewer = _mapping(preparation["reviewer"], "preparation.reviewer")
    verifier_run = _qualification_run(verifier["run_id"], field="preparation.verifier.run_id", candidate=candidate)
    reviewer_run = _qualification_run(reviewer["run_id"], field="preparation.reviewer.run_id", candidate=candidate)
    if len({proof_run, verifier_run, reviewer_run}) != 3:
        _reject("proof preparation, verifier, and reviewer must use separate run directories")
    verifier_path, verifier_ref = _qualification_ref_file(
        verifier["evidence"], field="preparation.verifier.evidence", run_dir=verifier_run, repository_root=root
    )
    reviewer_path, reviewer_ref = _qualification_ref_file(
        reviewer["evidence"], field="preparation.reviewer.evidence", run_dir=reviewer_run, repository_root=root
    )
    if verifier_path == reviewer_path or verifier_path == proof_run or reviewer_path == proof_run:
        _reject("verifier evidence path is reused across runs")
    verifier_evidence = _read_json(verifier_path, "preparation.verifier.evidence")
    reviewer_evidence = _read_json(reviewer_path, "preparation.reviewer.evidence")
    common_kwargs = {
        "record": preparation,
        "proof_run_id": str(proof_run),
        "proof_hashes": proof_hashes,
        "scope_base": scope_base,
        "decision_run_id": str(verifier_run),
    }
    _common_preparation_evidence(
        verifier_evidence,
        schema=PREPARATION_VERIFIER_SCHEMA,
        field="preparation.verifier.evidence",
        **common_kwargs,
    )
    common_kwargs["decision_run_id"] = str(reviewer_run)
    _common_preparation_evidence(
        reviewer_evidence,
        schema=PREPARATION_REVIEW_SCHEMA,
        field="preparation.reviewer.evidence",
        **common_kwargs,
    )
    if reviewer_evidence.get("verifier_run_id") != str(verifier_run):
        _reject("reviewer evidence verifier-run binding mismatch")
    if reviewer_evidence.get("verifier_evidence_sha256") != verifier_ref["sha256"]:
        _reject("reviewer evidence does not bind verifier evidence")
    if verifier_evidence.get("recorded_at") != verifier["recorded_at"]:
        _reject("verifier evidence recorded_at is not bound to the ledger")
    if reviewer_evidence.get("recorded_at") != reviewer["recorded_at"]:
        _reject("reviewer evidence recorded_at is not bound to the ledger")
    if verifier["verdict"] != ACCEPTED_VERIFIER_VERDICT or reviewer["verdict"] != ACCEPTED_REVIEWER_VERDICT:
        _reject("preparation verifier and reviewer verdicts must both be VERIFIED")
    if verifier_evidence.get("verdict") != verifier["verdict"] or reviewer_evidence.get("verdict") != reviewer["verdict"]:
        _reject("preparation evidence verdict is not bound to the ledger")
    if verifier_ref["sha256"] != _file_sha256(verifier_path, "verifier evidence") or reviewer_ref["sha256"] != _file_sha256(reviewer_path, "reviewer evidence"):
        _reject("preparation evidence mutated after hashing")

    qualified_at = _parse_timestamp(preparation["freshness"]["qualified_at"], "preparation.freshness.qualified_at")
    expires_at = _parse_timestamp(preparation["freshness"]["expires_at"], "preparation.freshness.expires_at")
    max_age = preparation["freshness"]["max_age_seconds"]
    if expires_at != qualified_at + timedelta(seconds=max_age):
        _reject("preparation freshness expiry does not match max_age_seconds")
    verifier_at = _parse_timestamp(verifier["recorded_at"], "preparation.verifier.recorded_at")
    reviewer_at = _parse_timestamp(reviewer["recorded_at"], "preparation.reviewer.recorded_at")
    if verifier_at > reviewer_at:
        _reject("preparation evidence ordering is not fresh")
    if qualified_at != reviewer_at:
        _reject("preparation qualified_at must equal reviewer recorded_at")
    if isinstance(now, datetime):
        if now.tzinfo is None:
            _reject("now must include a timezone")
        observed_at = now.astimezone(timezone.utc)
    elif now is not None:
        observed_at = _parse_timestamp(now, "now")
    else:
        observed_at = datetime.now(timezone.utc)
    if observed_at < qualified_at:
        _reject("preparation qualification is from the future")
    if observed_at > expires_at:
        _reject("preparation qualification is expired")


def validate_preflight_ledger(
    ledger: Mapping[str, Any],
    integration_branch: str,
    *,
    current_head: str | None = None,
    expected_taskfmt: Mapping[str, str] | None = None,
    dependency_graph: Mapping[str, Any] | None = None,
    repository_root: str | Path | None = None,
    current_tree: str | None = None,
    expected_oracle: Mapping[str, str] | None = None,
    catalog_identity: Mapping[str, Any] | None = None,
    task_graph_identity: Mapping[str, Any] | None = None,
    now: datetime | str | None = None,
) -> None:
    """Validate every ledger binding required by the authorizing preflight."""

    validate_ledger_schema(ledger)
    expected_ref = "refs/heads/" + _string(integration_branch, "integration_branch")
    if ledger["integration_ref"] != expected_ref:
        _reject(
            "ledger.integration_ref is not the requested integration branch "
            f"({ledger['integration_ref']!r} != {expected_ref!r})"
        )
    if ledger["armed"] is not False:
        _reject("ledger shows armed=true")
    if current_head is None:
        _reject("current integration HEAD is required")
    current_head = _full_sha(current_head, "current_head")
    if ledger["integration_head"] != current_head:
        _reject(
            "ledger.integration_head is stale "
            f"({ledger['integration_head']} != current {current_head})"
        )
    if current_tree is None:
        _reject("current integration tree is required")
    current_tree = _tree_sha(current_tree, "current_tree")
    if repository_root is None:
        _reject("repository root is required for receipt verification")
    root = Path(repository_root).resolve()
    if not root.is_dir():
        _reject(f"repository root is not a directory: {root}")
    if catalog_identity is None:
        _reject("current catalog identity is required")
    expected_catalog = _identity_expected(
        catalog_identity,
        field="catalog",
        current_head=current_head,
        current_tree=current_tree,
    )
    if "manifest" not in expected_catalog:
        _reject("current catalog manifest identity is required")
    _validate_ledger_catalog(
        _mapping(ledger["catalog"], "ledger.catalog"),
        expected=expected_catalog,
        integration_branch=integration_branch,
        repository_root=root,
    )
    if expected_taskfmt is None:
        _reject("current taskfmt identity is required")
    validate_taskfmt_binding(ledger, expected_taskfmt)
    if dependency_graph is None:
        _reject("current dependency graph is required")
    dependencies = _dependency_map(dependency_graph)
    rows = {row["task_id"]: row for row in ledger["tasks"]}
    disallowed = [
        row["task_id"]
        for row in ledger["tasks"]
        if row["status"] not in VALID_PREFLIGHT_TASK_STATUSES
    ]
    if disallowed:
        _reject(
            "preparation qualification cannot bypass production task status: "
            + ", ".join(disallowed)
        )
    accepted_candidates = [
        row for row in ledger["tasks"] if row["status"] in ACCEPTED_TASK_STATUSES
    ]
    if not accepted_candidates:
        preparation = ledger.get(PREPARATION_RECORD_KEY)
        if not isinstance(preparation, Mapping):
            _reject("no accepted production receipt or preparation qualification exists")
        validate_preparation_qualification(
            preparation,
            worktree=root,
            current_head=current_head,
            current_tree=current_tree,
            integration_branch=integration_branch,
            expected_oracle=expected_oracle,
            expected_taskfmt=expected_taskfmt,
            catalog_identity=catalog_identity,
            task_graph_identity=task_graph_identity,
            repository_root=root,
            now=now,
        )
        return

    accepted: dict[str, Mapping[str, Any]] = {}
    for row in accepted_candidates:
        task_id = row["task_id"]
        if task_id not in dependencies:
            _reject(f"task {task_id} is absent from the current dependency graph")
        expected_dependencies = dependencies[task_id]
        if row["dependencies"] != expected_dependencies:
            _reject(f"task {task_id} dependency binding is stale or reordered")
        receipt_key = row["receipt_key"]
        receipt = ledger["receipts"].get(receipt_key)
        if not isinstance(receipt, Mapping):
            _reject(f"task {task_id} has no receipt at {receipt_key}")
        _validate_accepted_binding(row, receipt, field=f"task {task_id}")
        if receipt["reviewer"]["verdict"] != ACCEPTED_REVIEWER_VERDICT:
            _reject(f"task {task_id} has no accepted independent reviewer")
        _validate_receipt_files(receipt, root, field=f"task {task_id}")
        integration_commit = receipt["integration_commit"]
        if not _is_ancestor(root, integration_commit, current_head):
            _reject(f"task {task_id} receipt is not bound to current HEAD ancestry")
        if not _is_ancestor(root, row["parent_sha"], row["candidate_tree_sha"]):
            _reject(f"task {task_id} parent is not an ancestor of candidate")
        if not _is_ancestor(root, row["candidate_tree_sha"], integration_commit):
            _reject(f"task {task_id} candidate is not an ancestor of integration commit")
        accepted[task_id] = row

    if not accepted:
        _reject("no current accepted verifier receipt exists")

    for task_id, row in accepted.items():
        for dependency in dependencies[task_id]:
            dependency_row = rows.get(dependency)
            if dependency_row is None:
                _reject(f"task {task_id} is missing dependency row {dependency}")
            if dependency_row["status"] != "integrated":
                _reject(f"task {task_id} dependency {dependency} is not integrated")
            dependency_receipt = ledger["receipts"].get(dependency_row["receipt_key"])
            if not isinstance(dependency_receipt, Mapping):
                _reject(f"task {task_id} dependency {dependency} has no receipt")
            _validate_accepted_binding(
                dependency_row,
                dependency_receipt,
                field=f"dependency {dependency}",
            )
            if not _is_ancestor(
                root,
                dependency_receipt["integration_commit"],
                ledger["receipts"][row["receipt_key"]]["base"],
            ):
                _reject(
                    f"task {task_id} dependency {dependency} receipt is not an "
                    "ancestor of the task base"
            )


def _validate_proof_context_bindings(
    context: Mapping[str, Any],
    *,
    candidate: Path,
    candidate_commit: str,
    candidate_tree: str,
    run_path: Path,
    task_id: str,
    scope_base: str,
    check_id: str,
    native_binary: Path,
    taskfmt: Mapping[str, Any],
) -> None:
    """Independently validate trust identities carried by one strong context."""

    required = (
        "schema",
        "run_id",
        "task_id",
        "check_id",
        "worktree_commit",
        "scope_base",
        "operation",
        "tree",
        "oracle_commit",
        "oracle_tree",
        "bundle",
        "bundle_sha256",
        "tool",
        "qualification",
    )
    _unknown(
        context,
        {
            "schema",
            "run_id",
            "task_id",
            "check_id",
            "worktree_commit",
            "scope_base",
            "operation",
            "tree",
            "oracle_commit",
            "oracle_tree",
            "bundle",
            "bundle_sha256",
            "tool",
            "dependencies",
            "adapter",
            "lane",
            "axes",
            "members",
            "inventory",
            "evidence",
            "configuration",
            "qualification",
            "architecture_profile",
            "branch_host_projection",
        },
        f"proof preparation.context.{check_id}",
    )
    _required(context, required, f"proof preparation.context.{check_id}")
    field = f"proof preparation.context.{check_id}"
    if context["schema"] != CONTEXT_SCHEMA:
        _reject(f"{field} schema mismatch")
    if (
        context["run_id"] != str(run_path)
        or context["task_id"] != task_id
        or context["check_id"] != check_id
        or context["worktree_commit"] != candidate_commit
        or context["scope_base"] != scope_base
    ):
        _reject(f"{field} identity mismatch")
    _string(context["operation"], f"{field}.operation")
    if context["tree"] != candidate_tree:
        _reject(f"{field} source tree is not bound to the candidate")
    if context["oracle_commit"] != FROZEN_ORACLE_COMMIT:
        _reject(f"{field} oracle commit is not the protected baseline")
    if context["oracle_tree"] != FROZEN_ORACLE_TREE:
        _reject(f"{field} oracle tree is not the protected baseline")
    if context["bundle"] != FROZEN_ORACLE_TAG:
        _reject(f"{field} oracle tag is not the protected visual-baseline tag")
    _sha256(context["bundle_sha256"], f"{field}.bundle_sha256")

    qualification = _mapping(context["qualification"], f"{field}.qualification")
    _unknown(
        qualification,
        {
            "common",
            "trust_manifest",
            "trust_manifest_sha256",
            "check",
            "worker_context",
            "template_sha256",
            "comparator",
        },
        f"{field}.qualification",
    )
    _required(
        qualification,
        ("common", "trust_manifest", "trust_manifest_sha256"),
        f"{field}.qualification",
    )
    common = _mapping(qualification["common"], f"{field}.qualification.common")
    _unknown(
        common,
        {
            "run_id",
            "task_id",
            "run_dir",
            "worktree",
            "scope_base",
            "candidate_commit",
            "candidate_tree",
            "oracle",
            "files",
            "dependencies",
            "tool",
            "comparator",
            "taskfmt",
            "observer",
            "outputs",
        },
        f"{field}.qualification.common",
    )
    _required(
        common,
        (
            "run_id",
            "task_id",
            "run_dir",
            "worktree",
            "scope_base",
            "candidate_commit",
            "candidate_tree",
            "oracle",
            "tool",
            "comparator",
            "taskfmt",
            "observer",
            "outputs",
        ),
        f"{field}.qualification.common",
    )
    if (
        common["run_id"] != str(run_path)
        or common["task_id"] != task_id
        or common["run_dir"] != str(run_path)
        or common["scope_base"] != scope_base
        or common["candidate_commit"] != candidate_commit
        or common["candidate_tree"] != candidate_tree
        or Path(_absolute_path(common["worktree"], f"{field}.worktree")).resolve()
        != candidate.resolve()
    ):
        _reject(f"{field} common identity mismatch")

    oracle = _mapping(common["oracle"], f"{field}.qualification.common.oracle")
    _unknown(
        oracle,
        {"tag", "tag_ref_sha256", "commit", "tree"},
        f"{field}.qualification.common.oracle",
    )
    _required(
        oracle,
        ("tag", "tag_ref_sha256", "commit", "tree"),
        f"{field}.qualification.common.oracle",
    )
    if oracle["tag"] != FROZEN_ORACLE_TAG:
        _reject(f"{field} common oracle tag is not protected")
    if oracle["commit"] != FROZEN_ORACLE_COMMIT:
        _reject(f"{field} common oracle commit is not protected")
    if oracle["tree"] != FROZEN_ORACLE_TREE:
        _reject(f"{field} common oracle tree is not protected")
    _sha256(oracle["tag_ref_sha256"], f"{field}.oracle.tag_ref_sha256")
    if oracle["tag_ref_sha256"] != FROZEN_ORACLE_REF_SHA256:
        _reject(f"{field} oracle tag ref is not the protected baseline ref")
    if context["bundle_sha256"] != oracle["tag_ref_sha256"]:
        _reject(f"{field} top-level oracle ref digest is not bound")

    candidate_tool = candidate / "tools/refactor-proof/bin/tc-proof"
    _validate_executable_identity(
        _mapping(common["tool"], f"{field}.qualification.common.tool"),
        expected_path=candidate_tool,
        field=f"{field}.qualification.common.tool",
    )
    if context["tool"] != common["tool"]:
        _reject(f"{field} worker binding is not copied from common trust")

    comparator = _mapping(
        common["comparator"], f"{field}.qualification.common.comparator"
    )
    _validate_executable_identity(
        comparator,
        expected_path=native_binary,
        field=f"{field}.qualification.common.comparator",
    )

    taskfmt_identity = _mapping(
        common["taskfmt"], f"{field}.qualification.common.taskfmt"
    )
    expected_taskfmt_identity = {
        "path": taskfmt["taskfmt_path"],
        "sha256": taskfmt["taskfmt_sha256"],
    }
    if taskfmt_identity != expected_taskfmt_identity:
        _reject(f"{field} taskfmt binding is not bound to the strong receipt")
    _validate_executable_identity(
        taskfmt_identity,
        expected_path=Path(taskfmt["taskfmt_path"]),
        field=f"{field}.qualification.common.taskfmt",
    )
    _qualification_directory(
        Path(taskfmt["taskfmt_source"]), f"{field}.taskfmt_source"
    )

    observer = _mapping(
        common["observer"], f"{field}.qualification.common.observer"
    )
    _required(observer, ("transport", "nonce", "capability"), f"{field}.observer")
    if observer["transport"] != "inherited-pipe/v1":
        _reject(f"{field} observer transport is not inherited-pipe/v1")
    nonce = _string(observer["nonce"], f"{field}.observer.nonce")
    capability = Path(_absolute_path(observer["capability"], f"{field}.observer.capability"))
    if capability.resolve() != (run_path / "observer.json").resolve():
        _reject(f"{field} observer capability escaped the run")
    observer_record = _read_json(capability, f"{field}.observer.capability")
    if observer_record.get("nonce_sha256") != hashlib.sha256(nonce.encode()).hexdigest():
        _reject(f"{field} observer nonce is not bound to the capability")

    outputs = _mapping(common["outputs"], f"{field}.qualification.common.outputs")
    _required(outputs, ("runtime", "taskfmt_logs"), f"{field}.outputs")
    if Path(_absolute_path(outputs["runtime"], f"{field}.outputs.runtime")).resolve() != (
        run_path / "outputs"
    ).resolve():
        _reject(f"{field} runtime output root escaped the run")
    if Path(_absolute_path(outputs["taskfmt_logs"], f"{field}.outputs.taskfmt_logs")).resolve() != (
        run_path / "taskfmt-logs"
    ).resolve():
        _reject(f"{field} taskfmt log root escaped the run")

    trust_manifest = _mapping(
        qualification["trust_manifest"], f"{field}.qualification.trust_manifest"
    )
    if trust_manifest.get("schema") != "tc-proof-trust-manifest/v1":
        _reject(f"{field} trust manifest schema mismatch")
    if trust_manifest.get("common") != common:
        _reject(f"{field} trust manifest common binding mismatch")
    _sha256(
        qualification["trust_manifest_sha256"],
        f"{field}.qualification.trust_manifest_sha256",
    )
    if _canonical_json_sha256(trust_manifest) != qualification["trust_manifest_sha256"]:
        _reject(f"{field} trust manifest digest mismatch")
    checks = trust_manifest.get("checks")
    if not isinstance(checks, list) or not any(
        isinstance(check, Mapping)
        and check.get("check_id") == check_id
        and check.get("operation") == context["operation"]
        for check in checks
    ):
        _reject(f"{field} trust manifest lacks the bound check")

    comparator_qualification = qualification.get("comparator")
    if comparator_qualification is not None:
        comparator_qualification = _mapping(
            comparator_qualification, f"{field}.qualification.comparator"
        )
        nested = _mapping(
            comparator_qualification.get("context"),
            f"{field}.qualification.comparator.context",
        )
        if (
            nested.get("run_id") != str(run_path)
            or nested.get("task_id") != task_id
            or nested.get("check_id") != check_id
            or nested.get("candidate_source_tree") != candidate_tree
            or nested.get("oracle_commit") != FROZEN_ORACLE_COMMIT
        ):
            _reject(f"{field} comparator context binding mismatch")
        report_path = Path(
            _absolute_path(
                comparator_qualification.get("report_path"),
                f"{field}.qualification.comparator.report_path",
            )
        )
        if report_path.parent != (run_path / "outputs") or report_path.name != f"{check_id}.compare.json":
            _reject(f"{field} comparator report escaped runtime outputs")


def _validate_preparation_file(
    preparation: Mapping[str, Any],
    *,
    worktree: str | Path,
    current_head: str,
    current_tree: str | None,
    run_dir: str | Path,
    expected_taskfmt: Mapping[str, str],
) -> None:
    preparation = _mapping(preparation, "proof preparation")
    allowed = {
        "schema",
        "task_id",
        "worktree",
        "commit",
        "scope_base",
        "run_id",
        "native_build",
        "taskfmt",
        "context_index",
        "contexts",
        "results",
        "observer",
    }
    _unknown(preparation, allowed, "proof preparation")
    _required(preparation, tuple(allowed), "proof preparation")
    if preparation["schema"] != PROOF_PREPARATION_SCHEMA:
        _reject(f"proof preparation schema is not {PROOF_PREPARATION_SCHEMA}")
    task_id = _task_id(preparation["task_id"], "proof preparation.task_id")
    worktree_path = Path(
        _absolute_path(preparation["worktree"], "proof preparation.worktree")
    )
    _qualification_directory(worktree_path, "proof preparation.worktree")
    worktree_path = worktree_path.resolve()
    expected_worktree = Path(
        _absolute_path(str(worktree), "worktree")
    )
    _qualification_directory(expected_worktree, "worktree")
    expected_worktree = expected_worktree.resolve()
    if worktree_path != expected_worktree:
        _reject("proof preparation worktree is not the verifier worktree")
    _full_sha(preparation["commit"], "proof preparation.commit")
    if preparation["commit"] != _full_sha(current_head, "current_head"):
        _reject("proof preparation commit is stale")
    if current_tree is None:
        _reject("proof preparation current tree is required")
    current_tree = _tree_sha(current_tree, "current_tree")
    _full_sha(preparation["scope_base"], "proof preparation.scope_base")
    run_path = _qualification_run(
        preparation["run_id"], field="proof preparation.run_id", candidate=worktree_path
    )
    expected_run = _qualification_run(
        str(run_dir), field="run_dir", candidate=worktree_path
    )
    if run_path != expected_run:
        _reject("proof preparation run_id is not the requested run directory")
    if run_path == worktree_path or worktree_path in run_path.parents:
        _reject("proof preparation run directory is inside the worktree")
    receipt_file = _qualification_file_under(
        run_path / "proof-preparation.json",
        run_path,
        "proof_preparation.receipt_file",
    )
    if _read_json(receipt_file, "proof_preparation.receipt_file") != dict(preparation):
        _reject("proof preparation receipt file does not match the validated object")
    _validate_taskfmt_record(preparation["taskfmt"], "proof preparation.taskfmt")
    validate_taskfmt_binding({"toolchain": preparation["taskfmt"]}, expected_taskfmt)

    native_build = _mapping(preparation["native_build"], "proof preparation.native_build")
    _unknown(
        native_build,
        {"receipt", "receipt_sha256", "binary", "binary_sha256"},
        "proof preparation.native_build",
    )
    _required(
        native_build,
        ("receipt", "receipt_sha256", "binary", "binary_sha256"),
        "proof preparation.native_build",
    )
    build_receipt_path = _qualification_file_under(
        Path(_absolute_path(native_build["receipt"], "native_build.receipt")),
        run_path,
        "native_build.receipt",
    )
    binary_path = _qualification_file_under(
        Path(_absolute_path(native_build["binary"], "native_build.binary")),
        run_path,
        "native_build.binary",
    )
    # Native proof artifacts live in the verifier-owned external run/target
    # namespace.  They must never be accepted from the candidate's target
    # path (which may be a shared-cache symlink).
    _sha256(native_build["receipt_sha256"], "native_build.receipt_sha256")
    _sha256(native_build["binary_sha256"], "native_build.binary_sha256")
    if (
        _file_sha256(build_receipt_path, "native_build.receipt")
        != native_build["receipt_sha256"]
    ):
        _reject("native build receipt hash mismatch")
    build = _read_json(build_receipt_path, "native_build.receipt")
    _unknown(
        build,
        {
            "schema",
            "worktree",
            "target_dir",
            "cargo_target_dir",
            "commit",
            "tree",
            "binary",
            "binary_sha256",
            "command",
        },
        "native_build.receipt",
    )
    _required(
        build,
        (
            "schema",
            "worktree",
            "target_dir",
            "commit",
            "tree",
            "binary",
            "binary_sha256",
        ),
        "native_build.receipt",
    )
    if build["schema"] != NATIVE_BUILD_SCHEMA:
        _reject("native build receipt has the wrong schema")
    build_worktree = Path(_absolute_path(build["worktree"], "native_build.worktree"))
    _qualification_directory(build_worktree, "native_build.worktree")
    if build_worktree.resolve() != worktree_path:
        _reject("native build receipt worktree mismatch")
    if build["commit"] != preparation["commit"]:
        _reject("native build receipt commit mismatch")
    if build["tree"] != current_tree:
        _reject("native build receipt tree mismatch")
    target_path = Path(_absolute_path(build["target_dir"], "native_build.receipt.target_dir"))
    _qualification_directory(target_path, "native_build.receipt.target_dir")
    if build.get("cargo_target_dir") != build["target_dir"]:
        _reject("native build receipt Cargo target mismatch")
    target_path = target_path.resolve()
    if target_path == worktree_path or worktree_path in target_path.parents:
        _reject("native build target is inside the worktree")
    try:
        target_path.relative_to(run_path)
    except ValueError:
        _reject("native build target is outside the proof run")
    if target_path.parent != run_path.resolve():
        _reject("native build target must be a direct child of the proof run")
    _qualification_directory(target_path / "debug", "native_build.receipt.target_dir/debug")
    if binary_path.resolve() != target_path / "debug" / "tc-proof":
        _reject("native build receipt binary does not match target/debug/tc-proof")
    if build_receipt_path.resolve() != target_path / "debug" / "tc-proof.build.json":
        _reject("native build receipt path does not match target/debug/tc-proof.build.json")
    if (
        Path(_absolute_path(build["binary"], "native_build.binary")).resolve()
        != binary_path.resolve()
    ):
        _reject("native build receipt binary mismatch")
    if build["binary_sha256"] != native_build["binary_sha256"]:
        _reject("native build receipt binary hash is not bound")
    if _file_sha256(binary_path, "native_build.binary") != native_build["binary_sha256"]:
        _reject("native comparator binary hash mismatch")

    index_record = _mapping(preparation["context_index"], "proof preparation.context_index")
    _unknown(index_record, {"path", "sha256"}, "proof preparation.context_index")
    _required(index_record, ("path", "sha256"), "proof preparation.context_index")
    index_path = _qualification_file_under(
        Path(_absolute_path(index_record["path"], "context_index.path")),
        run_path,
        "context_index.path",
    )
    _sha256(index_record["sha256"], "context_index.sha256")
    if _file_sha256(index_path, "context_index") != index_record["sha256"]:
        _reject("context index hash mismatch")
    index = _read_json(index_path, "context_index")
    _unknown(
        index,
        {
            "schema",
            "task_id",
            "run_id",
            "worktree_commit",
            "scope_base",
            "contexts",
            "results",
            "observer",
        },
        "context_index",
    )
    _required(
        index,
        (
            "schema",
            "task_id",
            "run_id",
            "worktree_commit",
            "scope_base",
            "contexts",
            "results",
            "observer",
        ),
        "context_index",
    )
    if index["schema"] != CONTEXT_INDEX_SCHEMA:
        _reject("context index has the wrong schema")
    if index["task_id"] != task_id or index["run_id"] != str(run_path):
        _reject("context index task/run binding mismatch")
    if index["worktree_commit"] != preparation["commit"]:
        _reject("context index commit mismatch")
    if index["scope_base"] != preparation["scope_base"]:
        _reject("context index scope base mismatch")

    contexts = preparation["contexts"]
    results = preparation["results"]
    if not isinstance(contexts, list) or not contexts:
        _reject("proof preparation.contexts must be non-empty")
    if not isinstance(results, list) or not results:
        _reject("proof preparation.results must be non-empty")
    if len(contexts) != len(results):
        _reject("proof preparation context/result counts differ")

    def entries(value: list[Any], field: str) -> dict[str, Mapping[str, Any]]:
        result: dict[str, Mapping[str, Any]] = {}
        for index_number, item in enumerate(value):
            item = _mapping(item, f"{field}[{index_number}]")
            _unknown(
                item,
                {"check_id", "path", "sha256"},
                f"{field}[{index_number}]",
            )
            _required(
                item,
                ("check_id", "path", "sha256"),
                f"{field}[{index_number}]",
            )
            check_id = _string(
                item["check_id"], f"{field}[{index_number}].check_id"
            )
            if re.fullmatch(r"CHK-[0-9]{3}", check_id) is None:
                _reject(f"{field}[{index_number}].check_id is invalid")
            if check_id in result:
                _reject(f"{field} contains duplicate {check_id}")
            _absolute_path(item["path"], f"{field}[{index_number}].path")
            _sha256(item["sha256"], f"{field}[{index_number}].sha256")
            result[check_id] = item
        return result

    context_entries = entries(contexts, "proof preparation.contexts")
    result_entries = entries(results, "proof preparation.results")
    if set(context_entries) != set(result_entries):
        _reject("proof preparation context/result check sets differ")

    index_contexts = entries(index["contexts"], "context_index.contexts")
    index_results = entries(index["results"], "context_index.results")
    if set(index_contexts) != set(context_entries) or set(index_results) != set(result_entries):
        _reject("context index check set does not match preparation")
    index_observer = _mapping(index["observer"], "context_index.observer")
    _unknown(index_observer, {"path", "sha256"}, "context_index.observer")
    _required(index_observer, ("path", "sha256"), "context_index.observer")
    if index_observer != preparation["observer"]:
        _reject("context index observer binding mismatch")
    expected_members = {"contexts": set(), "results": set()}
    for check_id in context_entries:
        for left, right, name in (
            (context_entries[check_id], index_contexts[check_id], "context"),
            (result_entries[check_id], index_results[check_id], "result"),
        ):
            if left["path"] != right["path"] or left["sha256"] != right["sha256"]:
                _reject(f"context index {name} binding mismatch for {check_id}")
        context_path = _qualification_file_under(
            Path(context_entries[check_id]["path"]),
            run_path,
            f"{check_id}.context",
        )
        result_path = _qualification_file_under(
            Path(result_entries[check_id]["path"]),
            run_path,
            f"{check_id}.result",
        )
        expected_members["contexts"].add(context_path.resolve())
        expected_members["results"].add(result_path.resolve())
        if (
            _file_sha256(context_path, f"{check_id}.context")
            != context_entries[check_id]["sha256"]
        ):
            _reject(f"{check_id} context hash mismatch")
        if (
            _file_sha256(result_path, f"{check_id}.result")
            != result_entries[check_id]["sha256"]
        ):
            _reject(f"{check_id} result hash mismatch")
        context = _read_json(context_path, f"{check_id}.context")
        _validate_proof_context_bindings(
            context,
            candidate=worktree_path,
            candidate_commit=preparation["commit"],
            candidate_tree=current_tree,
            run_path=run_path,
            task_id=task_id,
            scope_base=preparation["scope_base"],
            check_id=check_id,
            native_binary=binary_path,
            taskfmt=preparation["taskfmt"],
        )
        _required(
            context,
            (
                "schema",
                "task_id",
                "check_id",
                "run_id",
                "worktree_commit",
                "scope_base",
                "operation",
            ),
            f"{check_id}.context",
        )
        if context["schema"] != CONTEXT_SCHEMA:
            _reject(f"{check_id} context schema mismatch")
        if (
            context["task_id"] != task_id
            or context["check_id"] != check_id
            or context["run_id"] != str(run_path)
            or context["worktree_commit"] != preparation["commit"]
            or context["scope_base"] != preparation["scope_base"]
        ):
            _reject(f"{check_id} context identity mismatch")
        _string(context["operation"], f"{check_id}.context.operation")
        result = _read_json(result_path, f"{check_id}.result")
        _required(
            result,
            (
                "schema",
                "task_id",
                "check_id",
                "run_id",
                "worktree_commit",
                "scope_base",
                "context_sha256",
                "status",
            ),
            f"{check_id}.result",
        )
        if result["schema"] != PREPARATION_RESULT_SCHEMA or result["status"] != "ready":
            _reject(f"{check_id} result is not a preparation-ready result")
        if (
            result["task_id"] != task_id
            or result["check_id"] != check_id
            or result["run_id"] != str(run_path)
            or result["worktree_commit"] != preparation["commit"]
            or result["scope_base"] != preparation["scope_base"]
            or result["context_sha256"] != context_entries[check_id]["sha256"]
        ):
            _reject(f"{check_id} result identity mismatch")

    for directory_name, expected in expected_members.items():
        directory = _qualification_directory(
            run_path / directory_name,
            f"proof preparation.{directory_name}",
        )
        actual = {
            _qualification_file(path, f"proof preparation.{directory_name} member").resolve()
            for path in directory.iterdir()
        }
        if actual != expected:
            _reject(
                f"proof preparation {directory_name} contains missing or extra files"
            )

    expected_run_members = {
        target_path.name,
        "contexts",
        "results",
        "outputs",
        "taskfmt-logs",
        "observer.json",
        "context-index.json",
        "proof-preparation.json",
    }
    if {path.name for path in run_path.iterdir()} != expected_run_members:
        _reject("proof preparation run directory contains missing or extra members")
    for name in ("outputs", "taskfmt-logs"):
        _qualification_directory(run_path / name, f"proof preparation.{name}")

    observer = _mapping(preparation["observer"], "proof preparation.observer")
    _unknown(observer, {"path", "sha256"}, "proof preparation.observer")
    _required(observer, ("path", "sha256"), "proof preparation.observer")
    observer_path = _qualification_file_under(
        Path(_absolute_path(observer["path"], "observer.path")),
        run_path,
        "observer.path",
    )
    if observer_path.resolve() != run_path / "observer.json":
        _reject("observer.path must be run/observer.json")
    observer_members = {
        path.resolve()
        for path in run_path.iterdir()
        if path.name.startswith("observer")
    }
    if observer_members != {observer_path.resolve()}:
        _reject("proof preparation observer members are missing or extra")
    _sha256(observer["sha256"], "observer.sha256")
    if _file_sha256(observer_path, "observer") != observer["sha256"]:
        _reject("observer capability hash mismatch")
    observer_record = _read_json(observer_path, "observer")
    _unknown(
        observer_record,
        {
            "schema",
            "task_id",
            "run_id",
            "worktree_commit",
            "scope_base",
            "transport",
            "nonce_sha256",
        },
        "observer",
    )
    _required(
        observer_record,
        (
            "schema",
            "task_id",
            "run_id",
            "worktree_commit",
            "scope_base",
            "transport",
            "nonce_sha256",
        ),
        "observer",
    )
    if observer_record["schema"] != OBSERVER_SCHEMA:
        _reject("observer capability schema mismatch")
    if (
        observer_record["task_id"] != task_id
        or observer_record["run_id"] != str(run_path)
        or observer_record["worktree_commit"] != preparation["commit"]
        or observer_record["scope_base"] != preparation["scope_base"]
    ):
        _reject("observer capability identity mismatch")
    if observer_record["transport"] != "inherited-pipe/v1":
        _reject("observer capability transport mismatch")
    _sha256(observer_record["nonce_sha256"], "observer.nonce_sha256")


def validate_proof_preparation(
    preparation: Mapping[str, Any],
    *,
    worktree: str | Path,
    current_head: str,
    current_tree: str | None = None,
    run_dir: str | Path,
    expected_taskfmt: Mapping[str, str],
) -> None:
    """Validate native proof inputs without authorizing a task or ledger.

    This path intentionally never reads accepted task rows and never mutates
    ledger state. A passing preparation receipt only proves verifier inputs are
    bound and ready; a reviewer receipt is still required before preflight can
    accept any task row.
    """

    _validate_preparation_file(
        preparation,
        worktree=worktree,
        current_head=current_head,
        current_tree=current_tree,
        run_dir=run_dir,
        expected_taskfmt=expected_taskfmt,
    )


# Descriptive alias for callers that name the artifact rather than the phase.
validate_proof_preparation_receipt = validate_proof_preparation


if __name__ == "__main__":
    raise SystemExit("import campaign_ledger; do not run this module directly")
