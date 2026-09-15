"""Immutable context-index validation."""

from __future__ import annotations

import os
from pathlib import Path
from typing import Any

from .context import ALLOWED_V1_KEYS, Reject, V2_EXTRA_KEYS, V2_SCHEMA, load_context
from .json_util import load_path, sha256_bytes


def load_index() -> tuple[dict[str, Any], str]:
    index_path = os.environ.get("TC_PROOF_CONTEXT_INDEX")
    index_hash = os.environ.get("TC_PROOF_CONTEXT_INDEX_SHA256")
    if not index_path or not index_hash:
        raise Reject("CONTEXT_INDEX")
    path = Path(index_path)
    raw = path.read_bytes()
    if sha256_bytes(raw) != index_hash:
        raise Reject("CONTEXT_INDEX")
    index = load_path(path)
    return index, index_hash


def validate_index(context: dict[str, Any], context_hash: str) -> None:
    index, index_hash = load_index()
    if set(index) != {"schema", "run_id", "task_id", "tree", "trust_sha256", "members"}:
        raise Reject("CONTEXT_INDEX")
    if index["schema"] != "tc-proof-context-index/v1":
        raise Reject("CONTEXT_INDEX")
    if index["run_id"] != context["run_id"]:
        raise Reject("CONTEXT_INDEX")
    if index["task_id"] != os.environ.get("TC_PROOF_TASK_ID"):
        raise Reject("CONTEXT_INDEX")
    if index["tree"] != context["tree"]:
        raise Reject("CONTEXT_INDEX")
    members = index["members"]
    if len(members) != 4:
        raise Reject("CONTEXT_INDEX")
    if len({member["output_id"] for member in members}) != 4:
        raise Reject("CONTEXT_INDEX")
    contexts_dir = Path(members[0]["context_path"]).parent
    expected_files = {member["check_id"] + ".json" for member in members}
    if not contexts_dir.is_dir() or {path.name for path in contexts_dir.iterdir()} != expected_files:
        raise Reject("CONTEXT_INDEX")
    check_id = os.environ.get("TC_PROOF_CHECK_ID")
    optional = {"qualification"}
    required_child = (ALLOWED_V1_KEYS | V2_EXTRA_KEYS) - optional
    member_fields = {
        "check_id",
        "context_path",
        "context_sha256",
        "schema",
        "operation",
        "lane",
        "namespace",
        "required_ids",
        "output_id",
    }
    selected = None
    for member in members:
        if set(member) != member_fields:
            raise Reject("CONTEXT_INDEX")
        child_path = Path(member["context_path"])
        if not child_path.is_file() or child_path.is_symlink() or child_path.stat().st_nlink != 1:
            raise Reject("CONTEXT_INDEX")
        child_raw = child_path.read_bytes()
        if sha256_bytes(child_raw) != member["context_sha256"]:
            raise Reject("CONTEXT_INDEX")
        try:
            child = load_path(child_path)
        except ValueError:
            raise Reject("CONTEXT_INDEX") from None
        child_keys = set(child)
        if child_keys - required_child - optional or required_child - child_keys:
            raise Reject("CONTEXT_INDEX")
        if (
            child.get("schema") != V2_SCHEMA
            or member.get("schema") != V2_SCHEMA
            or child.get("schema") != member.get("schema")
        ):
            raise Reject("CONTEXT_INDEX")
        bindings = {
            "run_id": index["run_id"],
            "tree": index["tree"],
            "task_id": index["task_id"],
            "check_id": member["check_id"],
            "operation": member["operation"],
        }
        if any(child.get(key) != value for key, value in bindings.items()):
            raise Reject("CONTEXT_INDEX")
        if member["operation"] == "capture" and child.get("lane") != member["lane"]:
            raise Reject("CONTEXT_INDEX")
        if member["check_id"] == check_id:
            selected = member
    if selected is None:
        raise Reject("CONTEXT_INDEX")
    if context_hash != selected["context_sha256"]:
        raise Reject("CONTEXT_INDEX")
    _ = index_hash
