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
import re
import subprocess
from collections.abc import Mapping, Sequence
from datetime import datetime
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

ACCEPTED_TASK_STATUSES = frozenset({"verified", "integrated"})
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

    catalog = _mapping(ledger["catalog"], "ledger.catalog")
    _unknown(catalog, {"commit", "branch", "recorded_at"}, "ledger.catalog")
    _required(catalog, ("commit", "branch", "recorded_at"), "ledger.catalog")
    _full_sha(catalog["commit"], "ledger.catalog.commit")
    _string(catalog["branch"], "ledger.catalog.branch")
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


def validate_preflight_ledger(
    ledger: Mapping[str, Any],
    integration_branch: str,
    *,
    current_head: str | None = None,
    expected_taskfmt: Mapping[str, str] | None = None,
    dependency_graph: Mapping[str, Any] | None = None,
    repository_root: str | Path | None = None,
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
    if ledger["catalog"]["branch"] != integration_branch:
        _reject("ledger.catalog.branch is not the integration branch")
    if ledger["catalog"]["commit"] == "REPLACE_AT_INIT":
        _reject("catalog commit not recorded")
    if expected_taskfmt is None:
        _reject("current taskfmt identity is required")
    validate_taskfmt_binding(ledger, expected_taskfmt)
    if dependency_graph is None:
        _reject("current dependency graph is required")
    dependencies = _dependency_map(dependency_graph)
    if repository_root is None:
        _reject("repository root is required for receipt verification")
    root = Path(repository_root).resolve()
    if not root.is_dir():
        _reject(f"repository root is not a directory: {root}")

    rows = {row["task_id"]: row for row in ledger["tasks"]}
    accepted_candidates = [
        row for row in ledger["tasks"] if row["status"] in ACCEPTED_TASK_STATUSES
    ]
    if not accepted_candidates:
        _reject("no current accepted verifier receipt exists")

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


def _validate_preparation_file(
    preparation: Mapping[str, Any],
    *,
    worktree: str | Path,
    current_head: str,
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
    ).resolve()
    expected_worktree = Path(worktree).resolve()
    if worktree_path != expected_worktree:
        _reject("proof preparation worktree is not the verifier worktree")
    _full_sha(preparation["commit"], "proof preparation.commit")
    if preparation["commit"] != _full_sha(current_head, "current_head"):
        _reject("proof preparation commit is stale")
    _full_sha(preparation["scope_base"], "proof preparation.scope_base")
    run_path = Path(
        _absolute_path(preparation["run_id"], "proof preparation.run_id")
    ).resolve()
    expected_run = Path(run_dir).resolve()
    if run_path != expected_run:
        _reject("proof preparation run_id is not the requested run directory")
    if run_path == worktree_path or worktree_path in run_path.parents:
        _reject("proof preparation run directory is inside the worktree")
    if run_path.is_symlink() or not run_path.is_dir():
        _reject("proof preparation run directory is not a real directory")

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
    build_receipt_path = Path(
        _absolute_path(native_build["receipt"], "native_build.receipt")
    )
    binary_path = Path(_absolute_path(native_build["binary"], "native_build.binary"))
    _under(build_receipt_path, worktree_path, "native_build.receipt")
    _under(binary_path, worktree_path, "native_build.binary")
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
        {"schema", "worktree", "commit", "binary", "binary_sha256", "command"},
        "native_build.receipt",
    )
    _required(
        build,
        ("schema", "worktree", "commit", "binary", "binary_sha256"),
        "native_build.receipt",
    )
    if build["schema"] != NATIVE_BUILD_SCHEMA:
        _reject("native build receipt has the wrong schema")
    if (
        Path(_absolute_path(build["worktree"], "native_build.worktree")).resolve()
        != worktree_path
    ):
        _reject("native build receipt worktree mismatch")
    if build["commit"] != preparation["commit"]:
        _reject("native build receipt commit mismatch")
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
    index_path = _under(
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
    for check_id in context_entries:
        for left, right, name in (
            (context_entries[check_id], index_contexts[check_id], "context"),
            (result_entries[check_id], index_results[check_id], "result"),
        ):
            if left["path"] != right["path"] or left["sha256"] != right["sha256"]:
                _reject(f"context index {name} binding mismatch for {check_id}")
        context_path = _under(
            Path(context_entries[check_id]["path"]), run_path, f"{check_id}.context"
        )
        result_path = _under(
            Path(result_entries[check_id]["path"]), run_path, f"{check_id}.result"
        )
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

    observer = _mapping(preparation["observer"], "proof preparation.observer")
    _unknown(observer, {"path", "sha256"}, "proof preparation.observer")
    _required(observer, ("path", "sha256"), "proof preparation.observer")
    observer_path = _under(
        Path(_absolute_path(observer["path"], "observer.path")),
        run_path,
        "observer.path",
    )
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
        run_dir=run_dir,
        expected_taskfmt=expected_taskfmt,
    )


# Descriptive alias for callers that name the artifact rather than the phase.
validate_proof_preparation_receipt = validate_proof_preparation


if __name__ == "__main__":
    raise SystemExit("import campaign_ledger; do not run this module directly")
