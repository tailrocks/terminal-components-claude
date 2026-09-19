"""Observation and extension payload validation."""

from __future__ import annotations

import os
import stat
from pathlib import Path
from typing import Any

from .context import ALLOWED_V1_KEYS, OPTIONAL_EXTENSION_KEYS, Reject, expanded_members
from .json_util import load_path, sha256_bytes


def seeded_value(config: dict[str, Any]) -> int:
    return config["seed"] + len(config.get("seed_steps", []))


def validate_capture_payload(
    event: dict[str, Any],
    *,
    lane: str,
    expected_members: list[str],
    config: dict[str, Any],
    operation: str,
) -> None:
    if event.get("exit", 0) != 0 or not isinstance(event.get("payload"), dict):
        raise Reject("EXECUTION")
    body = event["payload"]
    draw_count = 2 if operation == "architecture" else 10
    calls = body.get("calls", [])
    if "App.update" not in calls:
        raise Reject("EXECUTION")
    if calls.count("Widget.draw") != draw_count:
        raise Reject("EXECUTION")
    if "Props.enabled" not in calls:
        raise Reject("EXECUTION")
    seeded = seeded_value(config)
    expected_value = seeded + (0 if operation == "architecture" else 1)
    if "value" not in body or body["value"] != expected_value:
        raise Reject("EXECUTION")
    expected_pty = operation == "capture" and lane == "pty"
    if body.get("pty") != expected_pty:
        raise Reject("EXECUTION")
    if operation in {"capture", "direct", "pty"}:
        selected_lane = lane if operation == "capture" else "direct"
        members = [name for name in expected_members if name.split("/")[1] == selected_lane]
        captures = body.get("captures")
        if not isinstance(captures, list) or [row["id"] for row in captures] != members:
            raise Reject("EXECUTION")
        for row in captures:
            if set(row) != {"id", "before", "after"}:
                raise Reject("EXECUTION")
            _, _, width, palette = row["id"].split("/")
            for checkpoint, increment in (("before", 0), ("after", 1)):
                expected = {
                    "width": int(width),
                    "custom_art": "*",
                    "control": {
                        "text": str(seeded + increment),
                        "foreground": palette,
                        "owner": "Widget",
                    },
                }
                if row[checkpoint] != expected:
                    raise Reject("EXECUTION")


def validate_oracle_repeat(events: list[dict[str, Any]]) -> None:
    if len(events) != 2:
        raise Reject("EXECUTION")
    first = events[0].get("payload")
    second = events[1].get("payload")
    if events[0].get("exit") != 0 or events[1].get("exit") != 0:
        raise Reject("EXECUTION")
    if first != second:
        raise Reject("REPEAT")


def validate_native_extension(payload: dict[str, Any], contract: dict[str, Any], source_commit: str) -> None:
    if payload.get("source") != contract["source_sha256"]:
        raise Reject("SOURCE")
    if contract.get("mapping_owner") != source_commit:
        raise Reject("SOURCE")
    if contract.get("mapping") != {"footer_row": "height - 2", "pointer": [2, "height - 2"]}:
        raise Reject("SOURCE")
    runs = payload.get("runs")
    if not runs or runs[0].get("exit") != 0:
        raise Reject("SOURCE")
    observed = runs[0]["stdout"]
    native = observed["native"]
    if [(row["width"], row["height"], row["row"]) for row in native] != [(120, 40, 38), (100, 30, 28)]:
        raise Reject("SOURCE")
    resized = observed["resized"]
    if [[row["width"], row["height"]] for row in resized] != contract["resized_sizes"]:
        raise Reject("SOURCE")
    for row in native + resized:
        if row["row"] != row["height"] - 2:
            raise Reject("SOURCE")
        if row["pointer"] != [2, row["height"] - 2]:
            raise Reject("SOURCE")
        if len(row["rows"]) != row["height"]:
            raise Reject("SOURCE")
        if any(len(line) != row["width"] for line in row["rows"]):
            raise Reject("SOURCE")
        if row.get("expected", "attached") not in row["rows"][row["row"]]:
            raise Reject("SOURCE")


_INDEX_KEYS = {
    "schema",
    "task_id",
    "run_id",
    "worktree_commit",
    "scope_base",
    "contexts",
    "results",
    "observer",
}
_MEMBER_KEYS = {"check_id", "path", "sha256"}
_PREPARATION_KEYS = {
    "schema",
    "task_id",
    "check_id",
    "run_id",
    "worktree_commit",
    "scope_base",
    "context_sha256",
    "status",
}
_RUNTIME_RESULT_KEYS = {
    "schema",
    "run_id",
    "operation",
    "context_sha256",
    "status",
    "category",
    "observation_digests",
    "outputs",
}


def _hex(value: Any, length: int) -> bool:
    return isinstance(value, str) and len(value) == length and all(
        char in "0123456789abcdef" for char in value
    )


def _check_id(value: Any) -> bool:
    return isinstance(value, str) and len(value) == 7 and value.startswith("CHK-") and value[4:].isdigit()


def _regular_file(path: Path) -> None:
    try:
        metadata = path.lstat()
    except OSError:
        raise Reject("CLOSURE") from None
    if path.is_symlink() or not stat.S_ISREG(metadata.st_mode) or metadata.st_nlink != 1:
        raise Reject("CLOSURE")


def _real_directory(path: Path) -> None:
    try:
        metadata = path.lstat()
    except OSError:
        raise Reject("CLOSURE") from None
    if path.is_symlink() or not stat.S_ISDIR(metadata.st_mode):
        raise Reject("CLOSURE")


def _object(path: Path) -> dict[str, Any]:
    try:
        value = load_path(path)
    except (OSError, TypeError, ValueError):
        raise Reject("CLOSURE") from None
    if not isinstance(value, dict):
        raise Reject("CLOSURE")
    return value


def _bound_member(member: Any, expected_id: str, directory: Path) -> tuple[Path, str]:
    if not isinstance(member, dict) or set(member) != _MEMBER_KEYS:
        raise Reject("CLOSURE")
    if member.get("check_id") != expected_id or not _hex(member.get("sha256"), 64):
        raise Reject("CLOSURE")
    raw_path = member.get("path")
    if not isinstance(raw_path, str):
        raise Reject("CLOSURE")
    path = Path(raw_path)
    if not path.is_absolute() or path.parent != directory or path.name != f"{expected_id}.json":
        raise Reject("CLOSURE")
    _regular_file(path)
    try:
        raw = path.read_bytes()
    except OSError:
        raise Reject("CLOSURE") from None
    if sha256_bytes(raw) != member["sha256"]:
        raise Reject("CLOSURE")
    return path, member["sha256"]


def validate_close_records(event: dict[str, Any], expected_check_ids: list[str]) -> None:
    if not isinstance(event, dict):
        raise Reject("CLOSURE")
    records = event.get("records")
    if not isinstance(records, list) or [row.get("check_id") if isinstance(row, dict) else None for row in records] != expected_check_ids:
        raise Reject("CLOSURE")
    for row in records:
        if not isinstance(row, dict) or set(row) != {
            "check_id",
            "context_sha256",
            "result_sha256",
            "status",
        }:
            raise Reject("CLOSURE")
        if row["status"] != "passed" or not _check_id(row["check_id"]):
            raise Reject("CLOSURE")
        if not _hex(row["context_sha256"], 64) or not _hex(row["result_sha256"], 64):
            raise Reject("CLOSURE")


def validate_index_close_outputs(
    event: dict[str, Any],
    index: dict[str, Any],
    output_dir: Path,
    current_check_id: str | None = None,
) -> None:
    """Validate close against the immutable context-index/v1 ABI.

    Close observes every indexed check except the selected close check. The
    observer records bind each prior result to its immutable context and raw
    runtime-result digest; no worker-provided output identifier is accepted.
    """
    if not isinstance(index, dict) or set(index) != _INDEX_KEYS:
        raise Reject("CLOSURE")
    if index.get("schema") != "tc-proof-context-index/v1":
        raise Reject("CLOSURE")
    if (
        not isinstance(index.get("task_id"), str)
        or not isinstance(index.get("run_id"), str)
        or not _hex(index.get("worktree_commit"), 40)
        or not _hex(index.get("scope_base"), 40)
    ):
        raise Reject("CLOSURE")
    run_dir = Path(index["run_id"])
    if not run_dir.is_absolute() or output_dir != run_dir / "outputs":
        raise Reject("CLOSURE")
    _real_directory(run_dir)
    _real_directory(output_dir)
    contexts_dir = run_dir / "contexts"
    results_dir = run_dir / "results"
    _real_directory(contexts_dir)
    _real_directory(results_dir)

    contexts = index.get("contexts")
    results = index.get("results")
    if not isinstance(contexts, list) or not contexts or not isinstance(results, list) or len(contexts) != len(results):
        raise Reject("CLOSURE")
    context_ids = [row.get("check_id") if isinstance(row, dict) else None for row in contexts]
    result_ids = [row.get("check_id") if isinstance(row, dict) else None for row in results]
    if (
        any(not _check_id(check_id) for check_id in context_ids)
        or context_ids != result_ids
        or len(set(context_ids)) != len(context_ids)
    ):
        raise Reject("CLOSURE")
    if current_check_id is None:
        current_check_id = os.environ.get("TC_PROOF_CHECK_ID")
    if not _check_id(current_check_id) or current_check_id not in context_ids:
        raise Reject("CLOSURE")
    expected_ids = [check_id for check_id in context_ids if check_id != current_check_id]
    if not expected_ids:
        raise Reject("CLOSURE")
    validate_close_records(event, expected_ids)

    context_entries: dict[str, tuple[Path, str, dict[str, Any]]] = {}
    required_context = ALLOWED_V1_KEYS
    allowed_context = required_context | {"qualification"} | OPTIONAL_EXTENSION_KEYS
    for check_id, member in zip(context_ids, contexts):
        context_path, context_hash = _bound_member(member, check_id, contexts_dir)
        context = _object(context_path)
        if (
            set(context) - allowed_context
            or required_context - set(context)
            or context.get("schema") != "tc-proof-context/v1"
            or any(
                context.get(key) != index[key]
                for key in ("task_id", "run_id", "worktree_commit", "scope_base")
            )
            or context.get("check_id") != check_id
            or not isinstance(context.get("operation"), str)
            or not context["operation"]
        ):
            raise Reject("CLOSURE")
        context_entries[check_id] = (context_path, context_hash, context)

    for check_id, member in zip(context_ids, results):
        _, context_hash, context = context_entries[check_id]
        result_path, _ = _bound_member(member, check_id, results_dir)
        preparation = _object(result_path)
        if (
            set(preparation) != _PREPARATION_KEYS
            or preparation.get("schema") != "tc-proof-preparation-result/v1"
            or preparation.get("task_id") != index["task_id"]
            or preparation.get("check_id") != check_id
            or preparation.get("run_id") != index["run_id"]
            or preparation.get("worktree_commit") != index["worktree_commit"]
            or preparation.get("scope_base") != index["scope_base"]
            or preparation.get("context_sha256") != context_hash
            or preparation.get("status") != "ready"
        ):
            raise Reject("CLOSURE")
    observer = index.get("observer")
    if not isinstance(observer, dict) or set(observer) != {"path", "sha256"} or not _hex(observer.get("sha256"), 64):
        raise Reject("CLOSURE")
    observer_path = observer.get("path")
    if not isinstance(observer_path, str) or Path(observer_path) != run_dir / "observer.json":
        raise Reject("CLOSURE")
    observer_file = Path(observer_path)
    _regular_file(observer_file)
    try:
        observer_raw = observer_file.read_bytes()
    except OSError:
        raise Reject("CLOSURE") from None
    if sha256_bytes(observer_raw) != observer["sha256"]:
        raise Reject("CLOSURE")
    capability = _object(observer_file)
    if set(capability) != {
        "schema",
        "task_id",
        "run_id",
        "worktree_commit",
        "scope_base",
        "transport",
        "nonce_sha256",
    } or (
        capability.get("schema") != "tc-proof-observer-capability/v1"
        or capability.get("task_id") != index["task_id"]
        or capability.get("run_id") != index["run_id"]
        or capability.get("worktree_commit") != index["worktree_commit"]
        or capability.get("scope_base") != index["scope_base"]
        or capability.get("transport") != "inherited-pipe/v1"
        or not _hex(capability.get("nonce_sha256"), 64)
    ):
        raise Reject("CLOSURE")

    expected_names = {f"{check_id}.result.json" for check_id in expected_ids}
    report_names: set[str] = set()
    for _, _, context in context_entries.values():
        qualification = context.get("qualification")
        if not isinstance(qualification, dict):
            raise Reject("CLOSURE")
        comparator = qualification.get("comparator")
        if comparator is None:
            continue
        if not isinstance(comparator, dict):
            raise Reject("CLOSURE")
        report_path = comparator.get("report_path")
        if report_path is None:
            continue
        if not isinstance(report_path, str) or Path(report_path).parent != output_dir:
            raise Reject("CLOSURE")
        report_name = Path(report_path).name
        if not report_name or report_name.endswith(".result.json"):
            raise Reject("CLOSURE")
        report_names.add(report_name)
    expected_names |= report_names
    try:
        actual_names = {path.name for path in output_dir.iterdir()}
    except OSError:
        raise Reject("CLOSURE") from None
    if actual_names != expected_names:
        raise Reject("CLOSURE")

    context_by_id = {check_id: values for check_id, values in context_entries.items()}
    for row in event["records"]:
        check_id = row["check_id"]
        output = output_dir / f"{check_id}.result.json"
        _regular_file(output)
        try:
            raw = output.read_bytes()
        except OSError:
            raise Reject("CLOSURE") from None
        report = _object(output)
        _, context_hash, context = context_by_id[check_id]
        if (
            set(report) != _RUNTIME_RESULT_KEYS
            or report.get("schema") != "tc-proof-runner-result/v1"
            or report.get("run_id") != index["run_id"]
            or report.get("operation") != context["operation"]
            or report.get("context_sha256") != context_hash
            or report.get("status") != "passed"
            or report.get("category") is not None
            or not isinstance(report.get("observation_digests"), list)
            or not report["observation_digests"]
            or not all(_hex(digest, 64) for digest in report["observation_digests"])
            or not isinstance(report.get("outputs"), dict)
            or row["context_sha256"] != context_hash
            or row["result_sha256"] != sha256_bytes(raw)
        ):
            raise Reject("CLOSURE")

    for report_name in report_names:
        _regular_file(output_dir / report_name)
        if not isinstance(_object(output_dir / report_name), dict):
            raise Reject("CLOSURE")
