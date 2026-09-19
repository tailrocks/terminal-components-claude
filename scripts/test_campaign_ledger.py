"""Adversarial contract checks for campaign ledger and proof preparation."""

from __future__ import annotations

import copy
import hashlib
import json
import tempfile
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any, Callable

from campaign_ledger import (
    CONTEXT_INDEX_SCHEMA,
    CONTEXT_SCHEMA,
    EVIDENCE_SCHEMA,
    FROZEN_ORACLE_TAG,
    FROZEN_ORACLE_COMMIT,
    FROZEN_ORACLE_REF_SHA256,
    FROZEN_ORACLE_TREE,
    NATIVE_BUILD_SCHEMA,
    OBSERVER_SCHEMA,
    PREPARATION_RESULT_SCHEMA,
    PREPARATION_QUALIFICATION_SCHEMA,
    PREPARATION_REVIEW_SCHEMA,
    PREPARATION_VERIFIER_SCHEMA,
    PROOF_PREPARATION_SCHEMA,
    RECEIPT_SCHEMA,
    RESULT_SCHEMA,
    REVIEW_SCHEMA,
    LedgerValidationError,
    accepted_verifier_rows,
    validate_ledger_schema,
    validate_preflight_ledger,
    validate_preparation_qualification,
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
TREE = "b" * 40
ORACLE_COMMIT = FROZEN_ORACLE_COMMIT
ORACLE_TREE = FROZEN_ORACLE_TREE


def write_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, sort_keys=True, indent=2) + "\n", encoding="utf-8")


def file_sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def canonical_json_sha256(value: Any) -> str:
    return hashlib.sha256(
        json.dumps(
            value,
            ensure_ascii=False,
            sort_keys=True,
            separators=(",", ":"),
        ).encode("utf-8")
    ).hexdigest()


def assert_proof_schema_contract() -> None:
    schema_path = (
        Path(__file__).resolve().parents[1]
        / "docs/refactoring-plan/campaign-ledger.schema.json"
    )
    schema = json.loads(schema_path.read_text(encoding="utf-8"))
    defs = schema["$defs"]
    assert defs["proofPreparation"]["$anchor"] == "proofPreparation"
    assert (
        defs["preparationQualification"]["properties"]["proof_preparation"]["$ref"]
        == "#/$defs/preparationProof"
    )
    assert (
        defs["proofPreparation"]["properties"]["taskfmt"]["$ref"]
        == "#/$defs/proofTaskfmt"
    )


def make_ledger(root: Path) -> dict[str, Any]:
    root.mkdir(parents=True, exist_ok=True)
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
    catalog_manifest = root / "catalog-manifest.json"
    catalog_manifest.write_text("catalog\n", encoding="utf-8")

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
        "catalog": {
            "commit": SHA,
            "tree": TREE,
            "branch": BRANCH,
            "manifest": {
                "path": "catalog-manifest.json",
                "sha256": file_sha256(catalog_manifest),
            },
            "recorded_at": NOW,
        },
        "toolchain": dict(TASKFMT),
        "receipts": {"task-001": receipt},
        "tasks": [row],
    }


def validate_fixture(ledger: dict[str, Any], root: Path) -> None:
    validate_preflight_ledger(
        ledger,
        BRANCH,
        current_head=SHA,
        current_tree=TREE,
        expected_taskfmt=TASKFMT,
        dependency_graph={"TASK-001": {"dependencies": []}},
        repository_root=root,
        catalog_identity={
            "commit": SHA,
            "tree": TREE,
            "manifest": ledger["catalog"]["manifest"],
        },
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


def make_preparation(root: Path) -> tuple[dict[str, Any], Path, Path, dict[str, str]]:
    worktree = root / "candidate-worktree"
    run = root / "external-run"
    worktree.mkdir(parents=True)
    (run / "target" / "debug").mkdir(parents=True)
    (run / "contexts").mkdir(parents=True)
    (run / "results").mkdir(parents=True)
    (run / "outputs").mkdir(parents=True)
    (run / "taskfmt-logs").mkdir(parents=True)
    worker = worktree / "tools/refactor-proof/bin/tc-proof"
    worker.parent.mkdir(parents=True)
    worker.write_bytes(b"candidate-proof-worker")
    worker.chmod(0o755)
    taskfmt_source = root / "taskfmt-source"
    taskfmt_source.mkdir()
    taskfmt_path = taskfmt_source / "bin/taskfmt"
    taskfmt_path.parent.mkdir()
    taskfmt_path.write_bytes(b"qualified-taskfmt")
    taskfmt_path.chmod(0o755)
    worktree = worktree.resolve()
    run = run.resolve()
    taskfmt_source = taskfmt_source.resolve()
    taskfmt_path = taskfmt_path.resolve()
    binary = run / "target" / "debug" / "tc-proof"
    binary.write_bytes(b"qualified-native-comparator")
    binary.chmod(0o755)
    build_receipt_path = binary.with_name("tc-proof.build.json")
    build = {
        "schema": NATIVE_BUILD_SCHEMA,
        "worktree": str(worktree),
        "target_dir": str(run / "target"),
        "cargo_target_dir": str(run / "target"),
        "commit": SHA,
        "tree": TREE,
        "binary": str(binary),
        "binary_sha256": file_sha256(binary),
    }
    write_json(build_receipt_path, build)

    context_path = run / "contexts" / "CHK-001.json"
    result_path = run / "results" / "CHK-001.json"
    observer_path = run / "observer.json"
    oracle_ref_sha256 = FROZEN_ORACLE_REF_SHA256
    tool_identity = {"path": str(worker), "sha256": file_sha256(worker)}
    comparator_identity = {"path": str(binary), "sha256": file_sha256(binary)}
    taskfmt = {
        "taskfmt_revision": TASKFMT["taskfmt_revision"],
        "taskfmt_version": TASKFMT["taskfmt_version"],
        "taskfmt_sha256": file_sha256(taskfmt_path),
        "taskfmt_source": str(taskfmt_source),
        "taskfmt_path": str(taskfmt_path),
    }
    nonce = "fixture-observer-nonce"
    common = {
        "run_id": str(run),
        "task_id": "TASK-001",
        "run_dir": str(run),
        "worktree": str(worktree),
        "scope_base": BASE,
        "candidate_commit": SHA,
        "candidate_tree": TREE,
        "oracle": {
            "tag": FROZEN_ORACLE_TAG,
            "tag_ref_sha256": oracle_ref_sha256,
            "commit": ORACLE_COMMIT,
            "tree": ORACLE_TREE,
        },
        "tool": tool_identity,
        "comparator": comparator_identity,
        "taskfmt": {
            "path": taskfmt["taskfmt_path"],
            "sha256": taskfmt["taskfmt_sha256"],
        },
        "observer": {
            "transport": "inherited-pipe/v1",
            "nonce": nonce,
            "capability": str(observer_path),
        },
        "outputs": {
            "runtime": str(run / "outputs"),
            "taskfmt_logs": str(run / "taskfmt-logs"),
        },
    }
    trust_manifest = {
        "schema": "tc-proof-trust-manifest/v1",
        "common": common,
        "checks": [
            {
                "check_id": "CHK-001",
                "operation": "preflight",
                "phase": "prepare",
                "context_reference": None,
                "command_sha256": "f" * 64,
                "requirements": [],
                "acceptance": [],
            }
        ],
    }
    context = {
        "schema": CONTEXT_SCHEMA,
        "task_id": "TASK-001",
        "check_id": "CHK-001",
        "run_id": str(run),
        "worktree_commit": SHA,
        "scope_base": BASE,
        "operation": "preflight",
        "tree": TREE,
        "oracle_commit": ORACLE_COMMIT,
        "oracle_tree": ORACLE_TREE,
        "bundle": FROZEN_ORACLE_TAG,
        "bundle_sha256": oracle_ref_sha256,
        "tool": tool_identity,
        "qualification": {
            "common": common,
            "trust_manifest": trust_manifest,
            "trust_manifest_sha256": canonical_json_sha256(trust_manifest),
        },
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
        "transport": "inherited-pipe/v1",
        "nonce_sha256": hashlib.sha256(nonce.encode()).hexdigest(),
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
        "taskfmt": taskfmt,
        "context_index": {
            "path": str(index_path),
            "sha256": file_sha256(index_path),
        },
        "contexts": index["contexts"],
        "results": index["results"],
        "observer": index["observer"],
    }
    write_json(run / "proof-preparation.json", preparation)
    return preparation, worktree, run, taskfmt


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
        "top-level catalog commit",
        lambda ledger, root: ledger["catalog"].update({"commit": "e" * 40}),
        expected="ledger.catalog.commit",
    )
    expect_reject(
        "top-level catalog tree",
        lambda ledger, root: ledger["catalog"].update({"tree": "e" * 40}),
        expected="ledger.catalog.tree",
    )
    expect_reject(
        "top-level catalog manifest hash",
        lambda ledger, root: ledger["catalog"]["manifest"].update({"sha256": "e" * 64}),
        expected="ledger.catalog.manifest",
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
                current_tree=TREE,
                expected_taskfmt=TASKFMT,
                dependency_graph={"TASK-001": {"dependencies": ["TASK-002"]}},
                repository_root=root,
                catalog_identity={
                    "commit": SHA,
                    "tree": TREE,
                    "manifest": ledger["catalog"]["manifest"],
                },
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
        preparation, worktree, run, taskfmt = make_preparation(root)
        validate_proof_preparation(
            preparation,
            worktree=worktree,
            current_head=SHA,
            current_tree=TREE,
            run_dir=run,
            expected_taskfmt=taskfmt,
        )
        forged = copy.deepcopy(preparation)
        forged["commit"] = "e" * 40
        try:
            validate_proof_preparation(
                forged,
                worktree=worktree,
                current_head=SHA,
                current_tree=TREE,
                run_dir=run,
                expected_taskfmt=taskfmt,
            )
        except LedgerValidationError:
            pass
        else:
            raise RuntimeError("stale proof-preparation receipt was accepted")

        legacy_index = {
            "schema": CONTEXT_INDEX_SCHEMA,
            "task_id": "TASK-001",
            "run_id": str(run),
            "tree": SHA,
            "trust_sha256": "e" * 64,
            "members": [],
        }
        index_path = run / "context-index.json"
        write_json(index_path, legacy_index)
        legacy = copy.deepcopy(preparation)
        legacy["context_index"]["sha256"] = file_sha256(index_path)
        write_json(run / "proof-preparation.json", legacy)
        try:
            validate_proof_preparation(
                legacy,
                worktree=worktree,
                current_head=SHA,
                current_tree=TREE,
                run_dir=run,
                expected_taskfmt=taskfmt,
            )
        except LedgerValidationError as error:
            if "context_index" not in str(error):
                raise RuntimeError(f"legacy ABI: wrong rejection: {error}") from error
        else:
            raise RuntimeError("legacy Rust runner index ABI was accepted")

    qualification_tests()
    legacy_adversarial_tests()

    print("campaign ledger authority and proof-preparation contracts: PASS")


def validate_current_head_mutation(ledger: dict[str, Any]) -> None:
    # Keep the mutation explicit so the test remains effective under -O.
    ledger["integration_head"] = "e" * 40


def expect_legacy_reject(
    label: str,
    mutate: Callable[[dict[str, Any], Path, Path], None],
    expected: str,
) -> None:
    with tempfile.TemporaryDirectory(prefix="campaign-legacy-proof-") as directory:
        root = Path(directory)
        preparation, worktree, run, taskfmt = make_preparation(root)
        mutate(preparation, worktree, run)
        write_json(run / "proof-preparation.json", preparation)
        try:
            validate_proof_preparation(
                preparation,
                worktree=worktree,
                current_head=SHA,
                current_tree=TREE,
                run_dir=run,
                expected_taskfmt=taskfmt,
            )
        except LedgerValidationError as error:
            if expected not in str(error):
                raise RuntimeError(f"{label}: wrong rejection: {error}") from error
            return
        raise RuntimeError(f"{label}: forged legacy proof was accepted")


def legacy_adversarial_tests() -> None:
    def symlink_run(preparation: dict[str, Any], worktree: Path, run: Path) -> None:
        alias = run.parent / "run-alias"
        alias.symlink_to(run, target_is_directory=True)
        preparation["run_id"] = str(alias)

    expect_legacy_reject("symlinked run root", symlink_run, "symlink")

    def extra_context(preparation: dict[str, Any], worktree: Path, run: Path) -> None:
        (run / "contexts" / "unexpected.json").write_text("extra\n", encoding="utf-8")

    expect_legacy_reject("extra context member", extra_context, "extra files")

    def extra_observer(preparation: dict[str, Any], worktree: Path, run: Path) -> None:
        (run / "observer-copy.json").write_bytes((run / "observer.json").read_bytes())

    expect_legacy_reject(
        "extra observer member", extra_observer, "missing or extra members"
    )

    def hardlinked_context(preparation: dict[str, Any], worktree: Path, run: Path) -> None:
        context = run / "contexts" / "CHK-001.json"
        source = run / "context-source.json"
        source.write_bytes(context.read_bytes())
        context.unlink()
        context.hardlink_to(source)

    expect_legacy_reject("hard-linked context", hardlinked_context, "hard-linked")

    def wrong_build_tree(preparation: dict[str, Any], worktree: Path, run: Path) -> None:
        receipt = run / "target" / "debug" / "tc-proof.build.json"
        build = json.loads(receipt.read_text(encoding="utf-8"))
        build["tree"] = "f" * 40
        write_json(receipt, build)
        preparation["native_build"]["receipt_sha256"] = file_sha256(receipt)

    expect_legacy_reject("wrong build tree", wrong_build_tree, "tree mismatch")

    def wrong_build_target(preparation: dict[str, Any], worktree: Path, run: Path) -> None:
        other_target = run / "other-target"
        (other_target / "debug").mkdir(parents=True)
        receipt = run / "target" / "debug" / "tc-proof.build.json"
        build = json.loads(receipt.read_text(encoding="utf-8"))
        build["target_dir"] = str(other_target)
        build["cargo_target_dir"] = str(other_target)
        write_json(receipt, build)
        preparation["native_build"]["receipt_sha256"] = file_sha256(receipt)

    expect_legacy_reject(
        "mismatched build target and binary",
        wrong_build_target,
        "target/debug/tc-proof",
    )


def make_qualification(root: Path) -> tuple[dict[str, Any], dict[str, Any], dict[str, str], datetime]:
    """Build a complete external-run qualification fixture."""
    preparation, candidate, proof_run, taskfmt = make_preparation(root)
    verifier_run = root / "verifier-run"
    reviewer_run = root / "reviewer-run"
    verifier_run.mkdir(parents=True)
    reviewer_run.mkdir(parents=True)
    (candidate / "catalog-manifest.json").write_text("catalog\n", encoding="utf-8")
    (candidate / "task-graph.json").write_text("graph\n", encoding="utf-8")
    verifier_run = verifier_run.resolve()
    reviewer_run = reviewer_run.resolve()

    def ref(path: Path) -> dict[str, str]:
        return {"path": str(path), "sha256": file_sha256(path)}

    prep_path = proof_run / "proof-preparation.json"
    index_path = proof_run / "context-index.json"
    observer = proof_run / "observer.json"
    binary = proof_run / "target" / "debug" / "tc-proof"

    catalog = {"commit": SHA, "tree": TREE, "manifest": {"path": "catalog-manifest.json", "sha256": file_sha256(candidate / "catalog-manifest.json")}}
    graph = {"commit": SHA, "tree": TREE, "manifest": {"path": "task-graph.json", "sha256": file_sha256(candidate / "task-graph.json")}}
    oracle = {"tag": "refs/tags/visual-baseline", "commit": ORACLE_COMMIT, "tree": ORACLE_TREE}
    qualified = datetime.now(timezone.utc).replace(microsecond=0)
    verifier_at = qualified - timedelta(seconds=2)
    common = {"proof_run_id": str(proof_run), "candidate_commit": SHA, "candidate_tree": TREE, "integration_ref": f"refs/heads/{BRANCH}", "oracle": oracle, "catalog": catalog, "task_graph": graph, "taskfmt": taskfmt, "proof_preparation_sha256": file_sha256(prep_path), "context_index_sha256": file_sha256(index_path), "observer_sha256": file_sha256(observer), "scope_base": BASE}
    verifier_path = verifier_run / "verifier.json"
    verifier = {"schema": PREPARATION_VERIFIER_SCHEMA, "verdict": "VERIFIED", "run_id": str(verifier_run), **common, "recorded_at": verifier_at.isoformat().replace("+00:00", "Z")}
    write_json(verifier_path, verifier)
    reviewer_path = reviewer_run / "reviewer.json"
    reviewer = {"schema": PREPARATION_REVIEW_SCHEMA, "verdict": "VERIFIED", "run_id": str(reviewer_run), "verifier_run_id": str(verifier_run), "verifier_evidence_sha256": file_sha256(verifier_path), **common, "recorded_at": qualified.isoformat().replace("+00:00", "Z")}
    write_json(reviewer_path, reviewer)
    qualification = {"schema": PREPARATION_QUALIFICATION_SCHEMA, "integration_ref": f"refs/heads/{BRANCH}", "candidate_commit": SHA, "candidate_tree": TREE, "oracle": oracle, "catalog": catalog, "task_graph": graph, "taskfmt": taskfmt, "proof_preparation": {"task_id": "TASK-001", "run_id": str(proof_run), "preparation_receipt": ref(prep_path), "context_index": ref(index_path), "observer": ref(observer), "contexts": preparation["contexts"], "results": preparation["results"]}, "verifier": {"run_id": str(verifier_run), "evidence": ref(verifier_path), "verdict": "VERIFIED", "recorded_at": verifier["recorded_at"]}, "reviewer": {"run_id": str(reviewer_run), "evidence": ref(reviewer_path), "verdict": "VERIFIED", "recorded_at": reviewer["recorded_at"]}, "freshness": {"qualified_at": qualified.isoformat().replace("+00:00", "Z"), "expires_at": (qualified + timedelta(seconds=3600)).isoformat().replace("+00:00", "Z"), "max_age_seconds": 3600}}
    expected_catalog = {"commit": SHA, "tree": TREE, "manifest": catalog["manifest"]}
    expected_graph = {"commit": SHA, "tree": TREE, "manifest": graph["manifest"]}
    return qualification, {"candidate": candidate, "proof": proof_run, "verifier": verifier_run, "reviewer": reviewer_run, "taskfmt": taskfmt}, {"tag": oracle["tag"], "commit": oracle["commit"], "tree": oracle["tree"]}, qualified


def qualification_tests() -> None:
    with tempfile.TemporaryDirectory(prefix="campaign-qualification-") as directory:
        root = Path(directory)
        qualification, paths, oracle, qualified = make_qualification(root)
        taskfmt = paths["taskfmt"]
        validate_preparation_qualification(qualification, worktree=paths["candidate"], current_head=SHA, current_tree=TREE, integration_branch=BRANCH, expected_oracle=oracle, expected_taskfmt=taskfmt, repository_root=paths["candidate"], now=qualified + timedelta(seconds=1))
        taskfmt_path = Path(taskfmt["taskfmt_path"])
        taskfmt_hardlink = taskfmt_path.with_name("taskfmt-hardlink")
        taskfmt_hardlink.hardlink_to(taskfmt_path)
        try:
            validate_preparation_qualification(qualification, worktree=paths["candidate"], current_head=SHA, current_tree=TREE, integration_branch=BRANCH, expected_oracle=oracle, expected_taskfmt=taskfmt, repository_root=paths["candidate"], now=qualified + timedelta(seconds=1))
        except LedgerValidationError as error:
            if "hard-linked" not in str(error):
                raise RuntimeError(f"hard-linked taskfmt rejected for wrong reason: {error}") from error
        else:
            raise RuntimeError("hard-linked taskfmt was accepted")
        finally:
            taskfmt_hardlink.unlink()
        weak_ledger = make_ledger(root / "weak-ledger")
        weak_ledger["toolchain"] = dict(taskfmt)
        weak_ledger["preparation"] = qualification
        validate_ledger_schema(weak_ledger)
        ledger = make_ledger(root / "ledger-fixture")
        ledger["toolchain"] = dict(taskfmt)
        ledger["catalog"]["manifest"] = qualification["catalog"]["manifest"]
        ledger["tasks"][0]["status"] = "blocked"
        ledger["preparation"] = qualification
        validate_preflight_ledger(ledger, BRANCH, current_head=SHA, current_tree=TREE, expected_oracle=oracle, expected_taskfmt=taskfmt, dependency_graph={"TASK-001": {"dependencies": []}}, repository_root=paths["candidate"], catalog_identity=qualification["catalog"], task_graph_identity=qualification["task_graph"], now=qualified + timedelta(seconds=1))
        dispatched = copy.deepcopy(ledger)
        dispatched["tasks"][0]["status"] = "dispatched"
        try:
            validate_preflight_ledger(dispatched, BRANCH, current_head=SHA, current_tree=TREE, expected_oracle=oracle, expected_taskfmt=taskfmt, dependency_graph={"TASK-001": {"dependencies": []}}, repository_root=paths["candidate"], catalog_identity=qualification["catalog"], task_graph_identity=qualification["task_graph"], now=qualified + timedelta(seconds=1))
        except LedgerValidationError as error:
            if "cannot bypass production task status" not in str(error):
                raise RuntimeError(f"dispatched row rejected for wrong reason: {error}") from error
        else:
            raise RuntimeError("dispatched production row bypassed preparation qualification")
        historical_with_dispatched = make_ledger(root / "historical-ledger")
        dispatched_row = copy.deepcopy(historical_with_dispatched["tasks"][0])
        dispatched_row["task_id"] = "TASK-002"
        dispatched_row["receipt_key"] = "task-002"
        dispatched_row["status"] = "dispatched"
        historical_with_dispatched["tasks"].append(dispatched_row)
        try:
            validate_fixture(historical_with_dispatched, root / "historical-ledger")
        except LedgerValidationError as error:
            if "cannot bypass production task status" not in str(error):
                raise RuntimeError(
                    f"dispatched row alongside historical receipt rejected for wrong reason: {error}"
                ) from error
        else:
            raise RuntimeError(
                "dispatched production row bypassed validation beside a historical receipt"
            )
        missing_preparation = copy.deepcopy(ledger)
        del missing_preparation["preparation"]
        try:
            validate_preflight_ledger(missing_preparation, BRANCH, current_head=SHA, current_tree=TREE, expected_oracle=oracle, expected_taskfmt=taskfmt, dependency_graph={"TASK-001": {"dependencies": []}}, repository_root=paths["candidate"], catalog_identity=qualification["catalog"], task_graph_identity=qualification["task_graph"], now=qualified + timedelta(seconds=1))
        except LedgerValidationError:
            pass
        else:
            raise RuntimeError("zero-production-row preflight accepted without preparation")
        forged = copy.deepcopy(qualification)
        forged["candidate_tree"] = "c" * 40
        try:
            validate_preparation_qualification(forged, worktree=paths["candidate"], current_head=SHA, current_tree=TREE, integration_branch=BRANCH, expected_oracle=oracle, expected_taskfmt=taskfmt, repository_root=paths["candidate"], now=qualified + timedelta(seconds=1))
        except LedgerValidationError:
            pass
        else:
            raise RuntimeError("wrong candidate tree was accepted")
        wrong_oracle = dict(oracle)
        wrong_oracle["tree"] = "1" * 40
        try:
            validate_preparation_qualification(qualification, worktree=paths["candidate"], current_head=SHA, current_tree=TREE, integration_branch=BRANCH, expected_oracle=wrong_oracle, expected_taskfmt=taskfmt, repository_root=paths["candidate"], now=qualified + timedelta(seconds=1))
        except LedgerValidationError:
            pass
        else:
            raise RuntimeError("caller-supplied oracle tree was accepted")
        proof = Path(paths["proof"])
        context = proof / "contexts" / "CHK-001.json"
        symlink_target = proof / "context-original.json"
        symlink_target.write_bytes(context.read_bytes())
        context.unlink()
        context.symlink_to(symlink_target)
        try:
            validate_preparation_qualification(qualification, worktree=paths["candidate"], current_head=SHA, current_tree=TREE, integration_branch=BRANCH, expected_oracle=oracle, expected_taskfmt=taskfmt, repository_root=paths["candidate"], now=qualified + timedelta(seconds=1))
        except LedgerValidationError as error:
            if "symlink" not in str(error):
                raise RuntimeError(f"symlink rejected for wrong reason: {error}") from error
        else:
            raise RuntimeError("symlinked proof input was accepted")
        context.unlink()
        context.write_bytes(symlink_target.read_bytes())
        symlink_target.unlink()
        context.write_text(context.read_text(encoding="utf-8") + "forged\n", encoding="utf-8")
        try:
            validate_preparation_qualification(qualification, worktree=paths["candidate"], current_head=SHA, current_tree=TREE, integration_branch=BRANCH, expected_oracle=oracle, expected_taskfmt=taskfmt, repository_root=paths["candidate"], now=qualified + timedelta(seconds=1))
        except LedgerValidationError as error:
            if "hash" not in str(error):
                raise RuntimeError(f"mutated context rejected for wrong reason: {error}") from error
        else:
            raise RuntimeError("mutated context was accepted")
        # A hard-linked evidence file is rejected before content is trusted.
        context.write_text(json.dumps({"schema": CONTEXT_SCHEMA, "task_id": "TASK-001", "check_id": "CHK-001", "run_id": str(proof), "worktree_commit": SHA, "scope_base": BASE, "operation": "preflight"}) + "\n", encoding="utf-8")
        hardlink = proof / "context-source.json"
        hardlink.write_bytes(context.read_bytes())
        context.unlink()
        context.hardlink_to(hardlink)
        try:
            validate_preparation_qualification(qualification, worktree=paths["candidate"], current_head=SHA, current_tree=TREE, integration_branch=BRANCH, expected_oracle=oracle, expected_taskfmt=taskfmt, repository_root=paths["candidate"], now=qualified + timedelta(seconds=1))
        except LedgerValidationError as error:
            if "hard-linked" not in str(error):
                raise RuntimeError(f"hardlink rejected for wrong reason: {error}") from error
        else:
            raise RuntimeError("hard-linked proof input was accepted")

    expect_qualification_reject(
        "shared proof/verifier run",
        lambda qualification, paths, qualified: qualification["verifier"].update(
            {"run_id": qualification["proof_preparation"]["run_id"]}
        ),
        "separate run directories",
    )
    expect_qualification_reject(
        "qualified_at drift",
        mutate_qualified_at,
        "qualified_at must equal reviewer recorded_at",
    )
    expect_qualification_reject(
        "reviewer evidence timestamp drift",
        mutate_reviewer_evidence_timestamp,
        "recorded_at is not bound",
    )
    expect_qualification_reject(
        "future qualification",
        lambda qualification, paths, qualified: None,
        "from the future",
        now_delta=-1,
    )


def expect_qualification_reject(
    label: str,
    mutate: Callable[[dict[str, Any], dict[str, Path], datetime], None],
    expected: str,
    *,
    now_delta: int = 1,
) -> None:
    with tempfile.TemporaryDirectory(prefix="campaign-qualification-reject-") as directory:
        qualification, paths, oracle, qualified = make_qualification(Path(directory))
        taskfmt = paths["taskfmt"]
        mutate(qualification, paths, qualified)
        try:
            validate_preparation_qualification(
                qualification,
                worktree=paths["candidate"],
                current_head=SHA,
                current_tree=TREE,
                integration_branch=BRANCH,
                expected_oracle=oracle,
                expected_taskfmt=taskfmt,
                repository_root=paths["candidate"],
                now=qualified + timedelta(seconds=now_delta),
            )
        except LedgerValidationError as error:
            if expected not in str(error):
                raise RuntimeError(f"{label}: wrong rejection: {error}") from error
            return
        raise RuntimeError(f"{label}: forged preparation qualification was accepted")


def mutate_reviewer_evidence_timestamp(
    qualification: dict[str, Any], paths: dict[str, Path], qualified: datetime
) -> None:
    evidence_path = Path(qualification["reviewer"]["evidence"]["path"])
    evidence = json.loads(evidence_path.read_text(encoding="utf-8"))
    evidence["recorded_at"] = (qualified - timedelta(seconds=1)).isoformat().replace(
        "+00:00", "Z"
    )
    write_json(evidence_path, evidence)
    qualification["reviewer"]["evidence"]["sha256"] = file_sha256(evidence_path)


def mutate_qualified_at(
    qualification: dict[str, Any], paths: dict[str, Path], qualified: datetime
) -> None:
    shifted = qualified + timedelta(seconds=1)
    qualification["freshness"]["qualified_at"] = shifted.isoformat().replace(
        "+00:00", "Z"
    )
    qualification["freshness"]["expires_at"] = (
        shifted + timedelta(seconds=3600)
    ).isoformat().replace("+00:00", "Z")



if __name__ == "__main__":
    assert_proof_schema_contract()
    main()
