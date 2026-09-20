"""Context loading and integrity checks."""

from __future__ import annotations

import itertools
import json
import os
from pathlib import Path
from typing import Any

from .json_util import load_path, sha256_bytes

V1_SCHEMA = "tc-proof-context/v1"
ALLOWED_V1_KEYS = {
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
    "observer_sequence",
}
OPTIONAL_EXTENSION_KEYS = {"architecture_profile", "branch_host_projection"}
OBSERVER_OPERATIONS = {
    "account-tests",
    "architecture",
    "capture",
    "close",
    "compare",
    "oracle",
    "preflight",
    "required",
    "external",
}


class Reject(Exception):
    def __init__(self, category: str) -> None:
        self.category = category
        super().__init__(category)


def expanded_members(axes: dict[str, list[Any]]) -> list[str]:
    return [
        "tiny/" + "/".join(str(part) for part in row)
        for row in itertools.product(axes["lanes"], axes["widths"], axes["palettes"])
    ]


def load_context(path: Path) -> tuple[dict[str, Any], bytes, str]:
    raw = path.read_bytes()
    try:
        context = load_path(path)
    except (ValueError, json.JSONDecodeError):
        raise Reject("INTEGRITY") from None
    if not isinstance(context, dict):
        raise Reject("INTEGRITY")
    return context, raw, sha256_bytes(raw)


def bind_context(context: dict[str, Any], raw_hash: str) -> None:
    expected_hash = os.environ.get("TC_PROOF_CONTEXT_SHA256")
    if expected_hash != raw_hash:
        raise Reject("INTEGRITY")
    if context.get("run_id") != os.environ.get("TC_PROOF_RUN_ID"):
        raise Reject("INTEGRITY")


def validate_schema(context: dict[str, Any]) -> None:
    schema = context.get("schema")
    keys = set(context)
    optional = {"qualification"} | OPTIONAL_EXTENSION_KEYS
    if schema != V1_SCHEMA:
        raise Reject("INTEGRITY")
    required = ALLOWED_V1_KEYS - optional
    if keys - required - optional:
        raise Reject("INTEGRITY")
    if required - keys:
        raise Reject("INTEGRITY")
    if context.get("task_id") != os.environ.get("TC_PROOF_TASK_ID"):
        raise Reject("CONTEXT_INDEX")
    if context.get("check_id") != os.environ.get("TC_PROOF_CHECK_ID"):
        raise Reject("CONTEXT_INDEX")
    validate_observer_sequence(context, context.get("operation"))


def validate_observer_sequence(context: dict[str, Any], operation: Any) -> list[str]:
    """Validate the host-bound, ordered observer request contract.

    The sequence is part of the hashed context.  It is deliberately required
    rather than inferred from the operation or qualification family: native
    oracle checks declare ``["oracle"]`` and synthetic oracle checks declare
    ``["oracle", "oracle"]``.  No pre-sequence context is accepted.
    """
    sequence = context.get("observer_sequence")
    if (
        not isinstance(sequence, list)
        or not sequence
        or len(sequence) > 8
        or any(
            not isinstance(item, str) or item not in OBSERVER_OPERATIONS
            for item in sequence
        )
        or operation not in OBSERVER_OPERATIONS
        or sequence[0] != operation
    ):
        raise Reject("PROTOCOL")
    return list(sequence)


def validate_oracle_sequence(context: dict[str, Any], operation: Any) -> list[str]:
    """Validate the exact native/synthetic oracle cardinality contract."""
    sequence = validate_observer_sequence(context, operation)
    if operation != "oracle" or any(item != "oracle" for item in sequence):
        raise Reject("PROTOCOL")
    qualification = context.get("qualification")
    family = qualification.get("family") if isinstance(qualification, dict) else None
    if len(sequence) == 1:
        if family != "native":
            raise Reject("PROTOCOL")
    elif len(sequence) == 2:
        if family != "synthetic":
            raise Reject("PROTOCOL")
    else:
        raise Reject("PROTOCOL")
    return sequence


def validate_tool(context: dict[str, Any]) -> None:
    tool = context["tool"]
    path = Path(tool["path"])
    actual = sha256_bytes(path.read_bytes())
    if actual != tool["sha256"]:
        raise Reject("PREFLIGHT")


def validate_dependencies(context: dict[str, Any]) -> None:
    for dependency in context["dependencies"]:
        if not dependency.get("accepted"):
            raise Reject("PREFLIGHT")
        if not dependency.get("integrated"):
            raise Reject("PREFLIGHT")


def validate_required_members(context: dict[str, Any]) -> list[str]:
    axes = context["axes"]
    if set(axes) != {"lanes", "widths", "palettes"}:
        raise Reject("REQUIRED_SET")
    expected = expanded_members(axes)
    members = context["members"]
    if members != expected:
        raise Reject("REQUIRED_SET")
    if len(set(members)) != len(members):
        raise Reject("REQUIRED_SET")
    return expected


def validate_source_authority(context: dict[str, Any], operation: str) -> None:
    if "source_directory" in context:
        raise Reject("SOURCE")
    adapter = context.get("adapter", {})
    if adapter.get("changes"):
        raise Reject("SOURCE")
    if context.get("oracle_commit") != os.environ.get("TC_PROOF_ORACLE_COMMIT"):
        raise Reject("SOURCE")
    if context.get("tree") != os.environ.get("TC_PROOF_SOURCE_TREE"):
        raise Reject("SOURCE")


def validate_close_evidence(context: dict[str, Any]) -> None:
    if context.get("candidate_success"):
        raise Reject("CLOSURE")
    evidence = context.get("evidence")
    if not isinstance(evidence, list):
        raise Reject("CLOSURE")
    if len(evidence) != 4:
        raise Reject("CLOSURE")
    tree = context["tree"]
    for row in evidence:
        if row.get("status") != "passed":
            raise Reject("CLOSURE")
        if row.get("tree") != tree:
            raise Reject("CLOSURE")
        if row.get("run_id") != context["run_id"]:
            raise Reject("CLOSURE")
