"""Adversarial contract checks for campaign ledger and proof preparation."""

from __future__ import annotations

import copy
import hashlib
import json
import tempfile
from pathlib import Path
from typing import Any, Callable

from campaign_ledger import (
    CONTEXT_INDEX_SCHEMA,
    CONTEXT_SCHEMA,
    EVIDENCE_SCHEMA,
    NATIVE_BUILD_SCHEMA,
    OBSERVER_SCHEMA,
    PREPARATION_RESULT_SCHEMA,
    PROOF_PREPARATION_SCHEMA,
    RECEIPT_SCHEMA,
    RESULT_SCHEMA,
    REVIEW_SCHEMA,
    LedgerValidationError,
    accepted_verifier_rows,
    validate_preflight_ledger,
    validate_proof_preparation,
)


BRANCH = "refactor/holla-parity"
SHA = "a" * 40
BASE = SHA
TASKFMT = {
    "taskfmt_revision": "c" * 40,
    "taskfmt_version": "0.2.0",
    "taskfmt_sha256": "d" * 64,
    "taskfmt_source": "/opt/taskfmt/source",
    "taskfmt_path": "/opt/taskfmt/bin/taskfmt",
}
NOW = "2026-09-18T00:00:00Z"


def write_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, sort_keys=True, indent=2) + "\n", encoding="utf-8")


def file_sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def make_ledger(root: Path) -> dict[str, Any]:
    run = root / "run-001"
    run.mkdir(parents=True)
    result_path = run / "result.json"
    reviewer_path = run / "reviewer.json"
    result = {
        "schema": RESULT_SCHEMA,
        "task_id": "TASK-001",
        "base": BASE,
        "candidate_tree_sha": SHA,
        "integration_commit": SHA,
        "run_id": str(run),
        "exit": 0,
        "last_line": "DONE",
        "status": "passed",
        "recorded_at": NOW,
    }
    write_json(result_path, result)
    result_hash = file_sha256(result_path)
    reviewer = {
        "schema": REVIEW_SCHEMA,
        "task_id": "TASK-001",
        "base": BASE,
        "candidate_tree_sha": SHA,
        "integration_commit": SHA,
        "run_id": str(run),
        "verdict": "VERIFIED",
        "result_sha256": result_hash,
        "recorded_at": NOW,
    }
    write_json(reviewer_path, reviewer)
    reviewer_hash = file_sha256(reviewer_path)
    evidence_path = root / ".campaign" / "evidence" / "task-001.json"
    evidence = {
        "schema": EVIDENCE_SCHEMA,
        "task_id": "TASK-001",
        "base": BASE,
        "candidate_tree_sha": SHA,
        "integration_commit": SHA,
        "run_id": str(run),
        "result_sha256": result_hash,
        "reviewer_evidence_sha256": reviewer_hash,
    }
    write_json(evidence_path, evidence)

    receipt = {
        "schema": RECEIPT_SCHEMA,
        "task_id": "TASK-001",
        "product": "qualified-preparation-fixture",
        "dependencies": [],
        "base": BASE,
        "candidate_tree_sha": SHA,
        "integration_commit": SHA,
        "run_id": str(run),
        "receipt_sha256": file_sha256(evidence_path),
        "evidence": ".campaign/evidence/task-001.json",
        "result": {
            **result,
            "path": str(result_path),
            "sha256": result_hash,
        },
        "reviewer": {
            **reviewer,
            "evidence": str(reviewer_path),
            "evidence_sha256": reviewer_hash,
        },
        "verify_exit": 0,
        "verify_last_line": "DONE",
        "recorded_at": NOW,
    }
    row = {
        "task_id": "TASK-001",
        "status": "integrated",
        "parent_sha": BASE,
        "candidate_tree_sha": SHA,
        "verifier_verdict": "VERIFIED",
        "reviewer_verdict": "VERIFIED",
        "receipt_key": "task-001",
        "dependencies": [],
        "run_id": str(run),
        "integrated_at": NOW,
    }
    return {
        "schema": "campaign-ledger/v1",
        "integration_ref": f"refs/heads/{BRANCH}",
        "integration_head": SHA,
        "armed": False,
        "catalog": {"commit": SHA, "branch": BRANCH, "recorded_at": NOW},
        "toolchain": dict(TASKFMT),
        "receipts": {"task-001": receipt},
        "tasks": [row],
    }


def validate_fixture(ledger: dict[str, Any], root: Path) -> None:
    validate_preflight_ledger(
        ledger,
        BRANCH,
        current_head=SHA,
        expected_taskfmt=TASKFMT,
        dependency_graph={"TASK-001": {"dependencies": []}},
        repository_root=root,
    )


def expect_reject(
    label: str,
    mutate: Callable[[dict[str, Any], Path], None],
    *,
    expected: str | None = None,
) -> None:
    with tempfile.TemporaryDirectory(prefix="campaign-ledger-") as directory:
        root = Path(directory)
        ledger = make_ledger(root)
        mutate(ledger, root)
        try:
            validate_fixture(ledger, root)
        except LedgerValidationError as error:
            if expected is not None and expected not in str(error):
                raise RuntimeError(f"{label}: wrong rejection: {error}") from error
            return
        raise RuntimeError(f"{label}: forged fixture was accepted")


def make_preparation(root: Path) -> tuple[dict[str, Any], Path, Path]:
    worktree = root / "candidate-worktree"
    run = root / "external-run"
    (worktree / "target" / "debug").mkdir(parents=True)
    (run / "contexts").mkdir(parents=True)
    (run / "results").mkdir(parents=True)
    worktree = worktree.resolve()
    run = run.resolve()
    binary = worktree / "target" / "debug" / "tc-proof"
    binary.write_bytes(b"qualified-native-comparator")
    build_receipt_path = binary.with_name("tc-proof.build.json")
    build = {
        "schema": NATIVE_BUILD_SCHEMA,
        "worktree": str(worktree),
        "commit": SHA,
        "binary": str(binary),
        "binary_sha256": file_sha256(binary),
    }
    write_json(build_receipt_path, build)

    context_path = run / "contexts" / "CHK-001.json"
    result_path = run / "results" / "CHK-001.json"
    observer_path = run / "observer.json"
    context = {
        "schema": CONTEXT_SCHEMA,
        "task_id": "TASK-001",
        "check_id": "CHK-001",
        "run_id": str(run),
        "worktree_commit": SHA,
        "scope_base": BASE,
        "operation": "preflight",
    }
    write_json(context_path, context)
    result = {
        "schema": PREPARATION_RESULT_SCHEMA,
        "task_id": "TASK-001",
        "check_id": "CHK-001",
        "run_id": str(run),
        "worktree_commit": SHA,
        "scope_base": BASE,
        "context_sha256": file_sha256(context_path),
        "status": "ready",
    }
    write_json(result_path, result)
    observer = {
        "schema": OBSERVER_SCHEMA,
        "task_id": "TASK-001",
        "run_id": str(run),
        "worktree_commit": SHA,
        "scope_base": BASE,
    }
    write_json(observer_path, observer)
    index = {
        "schema": CONTEXT_INDEX_SCHEMA,
        "task_id": "TASK-001",
        "run_id": str(run),
        "worktree_commit": SHA,
        "scope_base": BASE,
        "contexts": [
            {
                "check_id": "CHK-001",
                "path": str(context_path),
                "sha256": file_sha256(context_path),
            }
        ],
        "results": [
            {
                "check_id": "CHK-001",
                "path": str(result_path),
                "sha256": file_sha256(result_path),
            }
        ],
        "observer": {
            "path": str(observer_path),
            "sha256": file_sha256(observer_path),
        },
    }
    index_path = run / "context-index.json"
    write_json(index_path, index)
    preparation = {
        "schema": PROOF_PREPARATION_SCHEMA,
        "task_id": "TASK-001",
        "worktree": str(worktree),
        "commit": SHA,
        "scope_base": BASE,
        "run_id": str(run),
        "native_build": {
            "receipt": str(build_receipt_path),
            "receipt_sha256": file_sha256(build_receipt_path),
            "binary": str(binary),
            "binary_sha256": file_sha256(binary),
        },
        "taskfmt": dict(TASKFMT),
        "context_index": {
            "path": str(index_path),
            "sha256": file_sha256(index_path),
        },
        "contexts": index["contexts"],
        "results": index["results"],
        "observer": index["observer"],
    }
    return preparation, worktree, run


def main() -> None:
    with tempfile.TemporaryDirectory(prefix="campaign-ledger-") as directory:
        root = Path(directory)
        ledger = make_ledger(root)
        validate_fixture(ledger, root)
        if len(accepted_verifier_rows(ledger)) != 1:
            raise RuntimeError("valid bound receipt was not returned")

    minimal = {
        "tasks": [{"task_id": "TASK-001", "status": "verified", "verifier_verdict": "VERIFIED"}]
    }
    if accepted_verifier_rows(minimal):
        raise RuntimeError("minimal forged row was returned as accepted")

    expect_reject("armed", lambda ledger, root: ledger.update({"armed": True}), expected="armed")
    expect_reject("schema", lambda ledger, root: ledger.update({"schema": "wrong"}), expected="schema")
    expect_reject(
        "stale current head",
        lambda ledger, root: validate_current_head_mutation(ledger),
        expected="stale",
    )
    expect_reject(
        "taskfmt revision",
        lambda ledger, root: ledger["toolchain"].update({"taskfmt_revision": "e" * 40}),
        expected="taskfmt_revision",
    )
    expect_reject(
        "receipt binding",
        lambda ledger, root: ledger["tasks"][0].update({"receipt_key": "task-002"}),
        expected="receipt_key",
    )
    # Replace the dependency graph only for the next check; the fixture's
    # empty graph is otherwise valid.
    with tempfile.TemporaryDirectory(prefix="campaign-ledger-") as directory:
        root = Path(directory)
        ledger = make_ledger(root)
        try:
            validate_preflight_ledger(
                ledger,
                BRANCH,
                current_head=SHA,
                expected_taskfmt=TASKFMT,
                dependency_graph={"TASK-001": {"dependencies": ["TASK-002"]}},
                repository_root=root,
            )
        except LedgerValidationError as error:
            if "dependency" not in str(error):
                raise RuntimeError(f"dependency binding: wrong rejection: {error}") from error
        else:
            raise RuntimeError("dependency binding: forged dependency was accepted")

    expect_reject(
        "result hash",
        lambda ledger, root: (root / "run-001" / "result.json").write_text("forged\n"),
        expected="hash",
    )
    expect_reject(
        "reviewer verdict",
        lambda ledger, root: ledger["receipts"]["task-001"]["reviewer"].update(
            {"verdict": "REJECTED"}
        ),
        expected="reviewer_verdict",
    )
    expect_reject(
        "result exit",
        lambda ledger, root: ledger["receipts"]["task-001"]["result"].update({"exit": 1}),
        expected="exit",
    )

    with tempfile.TemporaryDirectory(prefix="campaign-preparation-") as directory:
        root = Path(directory)
        preparation, worktree, run = make_preparation(root)
        validate_proof_preparation(
            preparation,
            worktree=worktree,
            current_head=SHA,
            run_dir=run,
            expected_taskfmt=TASKFMT,
        )
        forged = copy.deepcopy(preparation)
        forged["commit"] = "e" * 40
        try:
            validate_proof_preparation(
                forged,
                worktree=worktree,
                current_head=SHA,
                run_dir=run,
                expected_taskfmt=TASKFMT,
            )
        except LedgerValidationError:
            pass
        else:
            raise RuntimeError("stale proof-preparation receipt was accepted")

    print("campaign ledger authority and proof-preparation contracts: PASS")


def validate_current_head_mutation(ledger: dict[str, Any]) -> None:
    # Keep the mutation explicit so the test remains effective under -O.
    ledger["integration_head"] = "e" * 40


if __name__ == "__main__":
    main()
