"""Immutable context-index validation."""

from __future__ import annotations

import os
import re
import stat
from pathlib import Path
from typing import Any

from .context import ALLOWED_V1_KEYS, Reject, V1_SCHEMA, V2_EXTRA_KEYS, load_context
from .json_util import load_path, sha256_bytes


def load_index() -> tuple[dict[str, Any], str]:
    index_path = os.environ.get("TC_PROOF_CONTEXT_INDEX")
    index_hash = os.environ.get("TC_PROOF_CONTEXT_INDEX_SHA256")
    if not index_path or not index_hash or not re.fullmatch(r"[0-9a-f]{64}", index_hash):
        raise Reject("CONTEXT_INDEX")
    path = Path(index_path)
    try:
        metadata = path.lstat()
        if path.is_symlink() or metadata.st_nlink != 1 or not stat.S_ISREG(metadata.st_mode):
            raise Reject("CONTEXT_INDEX")
        raw = path.read_bytes()
    except OSError:
        raise Reject("CONTEXT_INDEX") from None
    if sha256_bytes(raw) != index_hash:
        raise Reject("CONTEXT_INDEX")
    index = load_path(path)
    return index, index_hash


def validate_index(context: dict[str, Any], context_hash: str) -> None:
    index, index_hash = load_index()
    if set(index) != {
        "schema",
        "task_id",
        "run_id",
        "worktree_commit",
        "scope_base",
        "contexts",
        "results",
        "observer",
    }:
        raise Reject("CONTEXT_INDEX")
    if index["schema"] != "tc-proof-context-index/v1":
        raise Reject("CONTEXT_INDEX")
    if not isinstance(index.get("task_id"), str) or not isinstance(index.get("run_id"), str):
        raise Reject("CONTEXT_INDEX")
    index_path = os.environ.get("TC_PROOF_CONTEXT_INDEX")
    if not index_path:
        raise Reject("CONTEXT_INDEX")
    index_root = Path(index_path).resolve().parent
    if index["run_id"] != context["run_id"] or Path(index["run_id"]).resolve() != index_root:
        raise Reject("CONTEXT_INDEX")
    if index["task_id"] != os.environ.get("TC_PROOF_TASK_ID"):
        raise Reject("CONTEXT_INDEX")
    if index["worktree_commit"] != context["worktree_commit"]:
        raise Reject("CONTEXT_INDEX")
    if index["scope_base"] != context["scope_base"]:
        raise Reject("CONTEXT_INDEX")
    contexts = index["contexts"]
    results = index["results"]
    if not isinstance(contexts, list) or not contexts or not isinstance(results, list):
        raise Reject("CONTEXT_INDEX")
    if len(contexts) != len(results) or any(not isinstance(member, dict) for member in contexts + results):
        raise Reject("CONTEXT_INDEX")
    check_ids = [member.get("check_id") for member in contexts]
    result_ids = [member.get("check_id") for member in results]
    if any(not isinstance(check_id, str) or re.fullmatch(r"CHK-[0-9]{3}", check_id) is None for check_id in check_ids):
        raise Reject("CONTEXT_INDEX")
    if len(set(check_ids)) != len(contexts) or sorted(check_ids) != sorted(result_ids):
        raise Reject("CONTEXT_INDEX")
    contexts_dir = index_root / "contexts"
    results_dir = index_root / "results"
    outputs_dir = index_root / "outputs"
    logs_dir = index_root / "taskfmt-logs"
    for directory in (contexts_dir, results_dir, outputs_dir, logs_dir):
        try:
            metadata = directory.lstat()
            if directory.is_symlink() or not stat.S_ISDIR(metadata.st_mode):
                raise Reject("CONTEXT_INDEX")
        except OSError:
            raise Reject("CONTEXT_INDEX") from None
    expected_files = {member["check_id"] + ".json" for member in contexts}
    if {path.name for path in contexts_dir.iterdir()} != expected_files:
        raise Reject("CONTEXT_INDEX")
    if {path.name for path in results_dir.iterdir()} != expected_files:
        raise Reject("CONTEXT_INDEX")
    check_id = os.environ.get("TC_PROOF_CHECK_ID")
    optional = {"qualification", "architecture_profile", "branch_host_projection"}
    required_child = (ALLOWED_V1_KEYS | V2_EXTRA_KEYS) - optional
    member_fields = {
        "check_id",
        "path",
        "sha256",
    }
    result_map = {member.get("check_id"): member for member in results}
    selected = None
    for member in contexts:
        if set(member) != member_fields:
            raise Reject("CONTEXT_INDEX")
        child_path = Path(member["path"])
        if child_path.parent != contexts_dir or child_path.name != member["check_id"] + ".json":
            raise Reject("CONTEXT_INDEX")
        result = result_map.get(member["check_id"])
        if result is None or set(result) != member_fields:
            raise Reject("CONTEXT_INDEX")
        result_path = Path(result["path"])
        if result_path.parent != results_dir or result_path.name != member["check_id"] + ".json":
            raise Reject("CONTEXT_INDEX")
        if result_path.is_symlink() or not result_path.is_file() or result_path.stat().st_nlink != 1:
            raise Reject("CONTEXT_INDEX")
        try:
            preparation = load_path(result_path)
        except (OSError, ValueError):
            raise Reject("CONTEXT_INDEX") from None
        if (
            not isinstance(preparation, dict)
            or set(preparation)
            != {
                "schema",
                "task_id",
                "check_id",
                "run_id",
                "worktree_commit",
                "scope_base",
                "context_sha256",
                "status",
            }
            or preparation.get("schema") != "tc-proof-preparation-result/v1"
            or preparation.get("task_id") != index["task_id"]
            or preparation.get("check_id") != member["check_id"]
            or preparation.get("run_id") != index["run_id"]
            or preparation.get("worktree_commit") != index["worktree_commit"]
            or preparation.get("scope_base") != index["scope_base"]
            or preparation.get("context_sha256") != member["sha256"]
            or preparation.get("status") != "ready"
        ):
            raise Reject("CONTEXT_INDEX")
        if sha256_bytes(result_path.read_bytes()) != result["sha256"]:
            raise Reject("CONTEXT_INDEX")
        if not child_path.is_file() or child_path.is_symlink() or child_path.stat().st_nlink != 1:
            raise Reject("CONTEXT_INDEX")
        child_raw = child_path.read_bytes()
        if sha256_bytes(child_raw) != member["sha256"]:
            raise Reject("CONTEXT_INDEX")
        try:
            child = load_path(child_path)
        except ValueError:
            raise Reject("CONTEXT_INDEX") from None
        child_keys = set(child)
        if child_keys - required_child - optional or required_child - child_keys:
            raise Reject("CONTEXT_INDEX")
        if child.get("schema") != V1_SCHEMA:
            raise Reject("CONTEXT_INDEX")
        bindings = {
            "run_id": index["run_id"],
            "task_id": index["task_id"],
            "worktree_commit": index["worktree_commit"],
            "scope_base": index["scope_base"],
            "check_id": member["check_id"],
        }
        if any(child.get(key) != value for key, value in bindings.items()):
            raise Reject("CONTEXT_INDEX")
        if member["check_id"] == check_id:
            selected = member
    if selected is None:
        raise Reject("CONTEXT_INDEX")
    if context_hash != selected["sha256"]:
        raise Reject("CONTEXT_INDEX")
    observer = index.get("observer")
    if not isinstance(observer, dict) or set(observer) != {"path", "sha256"}:
        raise Reject("CONTEXT_INDEX")
    observer_path = Path(observer["path"])
    if observer_path.parent != index_root or observer_path.name != "observer.json":
        raise Reject("CONTEXT_INDEX")
    if observer_path.is_symlink() or not observer_path.is_file() or observer_path.stat().st_nlink != 1:
        raise Reject("CONTEXT_INDEX")
    try:
        capability = load_path(observer_path)
    except (OSError, ValueError):
        raise Reject("CONTEXT_INDEX") from None
    if (
        not isinstance(capability, dict)
        or set(capability)
        != {
            "schema",
            "task_id",
            "run_id",
            "worktree_commit",
            "scope_base",
            "transport",
            "nonce_sha256",
        }
        or capability.get("schema") != "tc-proof-observer-capability/v1"
        or capability.get("task_id") != index["task_id"]
        or capability.get("run_id") != index["run_id"]
        or capability.get("worktree_commit") != index["worktree_commit"]
        or capability.get("scope_base") != index["scope_base"]
        or capability.get("transport") != "inherited-pipe/v1"
        or not isinstance(capability.get("nonce_sha256"), str)
        or re.fullmatch(r"[0-9a-f]{64}", capability["nonce_sha256"]) is None
        or sha256_bytes(observer_path.read_bytes()) != observer["sha256"]
    ):
        raise Reject("CONTEXT_INDEX")
    expected_root = {"contexts", "results", "outputs", "taskfmt-logs", "observer.json", "context-index.json"}
    if {path.name for path in index_root.iterdir()} != expected_root:
        raise Reject("CONTEXT_INDEX")
    expected_result = outputs_dir / f"{check_id}.result.json"
    bound_result = os.environ.get("TC_PROOF_RESULT")
    if bound_result and Path(bound_result) != expected_result:
        raise Reject("CONTEXT_INDEX")
    _ = index_hash
