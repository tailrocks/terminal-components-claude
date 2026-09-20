"""Runner-context/v1-v2 qualification schema, index, and group70 operations."""

from __future__ import annotations

import os
import stat
from pathlib import Path
from typing import Any

from ..runner.context import (
    Reject,
    bind_context,
    load_context,
    validate_close_evidence,
    validate_dependencies,
    validate_required_members,
    validate_source_authority,
    validate_tool,
)
from ..runner.json_util import canonical, load_path, sha256_bytes, sha256_canonical
from ..runner.observer import ObserverError
from ..runner.result import finish
from ..runner.validate import (
    validate_capture_payload,
    validate_native_extension,
    validate_oracle_repeat,
)
from .observer import QualificationObserver

QUALIFICATION_V1 = "tc-proof-runner-context/v1"
QUALIFICATION_V2 = "tc-proof-runner-context/v2"
QUALIFICATION_SCHEMAS = {QUALIFICATION_V1, QUALIFICATION_V2}
ALLOWED_QUALIFICATION_V1_KEYS = {
    "schema",
    "run_id",
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
}
QUALIFICATION_V2_EXTRA_KEYS = {"task_id", "check_id"}
ALLOWED_SYSTEM_SYMLINKS = {
    Path("/etc"): Path("/private/etc"),
    Path("/home"): Path("/System/Volumes/Data/home"),
    Path("/tmp"): Path("/private/tmp"),
    Path("/var"): Path("/private/var"),
}


def is_qualification_schema(context: dict[str, Any]) -> bool:
    return context.get("schema") in QUALIFICATION_SCHEMAS


def qualification_validated_executable(value: object, label: str) -> Path:
    """Allow documented macOS firmlink ancestry; reject other symlink parents."""
    if not isinstance(value, str) or not value:
        raise RuntimeError(f"{label} path is missing")
    path = Path(value)
    if not path.is_absolute():
        raise RuntimeError(f"{label} path must be absolute")

    current = Path(path.anchor)
    for component in path.parts[1:-1]:
        current /= component
        try:
            metadata = current.lstat()
        except OSError as error:
            raise RuntimeError(f"{label} path is unavailable: {current}") from error
        if stat.S_ISLNK(metadata.st_mode):
            allowed = ALLOWED_SYSTEM_SYMLINKS.get(current)
            if allowed is None or current.resolve() != allowed:
                raise RuntimeError(f"{label} path contains a symlink: {current}")
            continue
        if not stat.S_ISDIR(metadata.st_mode):
            raise RuntimeError(f"{label} parent path is not a directory: {current}")

    try:
        metadata = path.lstat()
    except OSError as error:
        raise RuntimeError(f"{label} path is unavailable: {path}") from error
    if path.is_symlink() or not stat.S_ISREG(metadata.st_mode) or metadata.st_nlink != 1:
        raise RuntimeError(f"{label} is not a regular single-link file")
    if not os.access(path, os.X_OK):
        raise RuntimeError(f"{label} is not executable")
    return path


def validate_qualification_schema(context: dict[str, Any]) -> None:
    schema = context.get("schema")
    keys = set(context)
    optional = {"qualification"}
    if schema == QUALIFICATION_V1:
        required = ALLOWED_QUALIFICATION_V1_KEYS - optional
    elif schema == QUALIFICATION_V2:
        required = (ALLOWED_QUALIFICATION_V1_KEYS | QUALIFICATION_V2_EXTRA_KEYS) - optional
    else:
        raise Reject("INTEGRITY")
    if keys - required - optional:
        raise Reject("INTEGRITY")
    if required - keys:
        raise Reject("INTEGRITY")
    if schema == QUALIFICATION_V2:
        if context.get("task_id") != os.environ.get("TC_PROOF_TASK_ID"):
            raise Reject("CONTEXT_INDEX")
        if context.get("check_id") != os.environ.get("TC_PROOF_CHECK_ID"):
            raise Reject("CONTEXT_INDEX")


def qualification_load_index() -> tuple[dict[str, Any], str]:
    index_path = os.environ.get("TC_PROOF_CONTEXT_INDEX")
    index_hash = os.environ.get("TC_PROOF_CONTEXT_INDEX_SHA256")
    if not index_path or not index_hash:
        raise Reject("CONTEXT_INDEX")
    path = Path(index_path)
    raw = path.read_bytes()
    if sha256_bytes(raw) != index_hash:
        raise Reject("CONTEXT_INDEX")
    return load_path(path), index_hash


def qualification_validate_index(context: dict[str, Any], context_hash: str) -> None:
    index, index_hash = qualification_load_index()
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
    required_child = (ALLOWED_QUALIFICATION_V1_KEYS | QUALIFICATION_V2_EXTRA_KEYS) - optional
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
            child.get("schema") != QUALIFICATION_V2
            or member.get("schema") != QUALIFICATION_V2
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


def qualification_validate_close_records(event: dict[str, Any]) -> None:
    records = event.get("records")
    if not isinstance(records, list):
        raise Reject("CLOSURE")
    if [row["check_id"] for row in records] != ["CHK-001", "CHK-002", "CHK-003"]:
        raise Reject("CLOSURE")
    if any(row.get("status") != "passed" for row in records):
        raise Reject("CLOSURE")


def qualification_validate_index_close_outputs(
    event: dict[str, Any],
    index: dict[str, Any],
    output_dir: Path,
) -> None:
    records = event.get("records", [])
    members = {member["check_id"]: member for member in index["members"]}
    for row in records:
        member = members.get(row["check_id"])
        if member is None:
            raise Reject("CLOSURE")
        output = output_dir / member["output_id"]
        if not output.is_file():
            raise Reject("CLOSURE")
        report = load_path(output)
        if sha256_canonical(report) != row.get("report_sha256"):
            raise Reject("CLOSURE")
        if row.get("context_sha256") != member["context_sha256"]:
            raise Reject("CLOSURE")
        if row.get("output_id") != member["output_id"]:
            raise Reject("CLOSURE")


def _reported_hash(context_hash: str) -> str:
    return os.environ.get("TC_PROOF_CONTEXT_SHA256", context_hash)


def _guard_qualification(context: dict[str, Any], context_hash: str) -> None:
    if os.environ.get("TC_PROOF_CONTEXT_INDEX"):
        qualification_validate_index(context, context_hash)
        return
    bind_context(context, context_hash)


def _load_or_reject(context_path: Path, default_operation: str) -> tuple[dict[str, Any], str, str] | Reject:
    try:
        context, _, context_hash = load_context(context_path)
    except Reject as error:
        return error
    return context, context_hash, context.get("operation", default_operation)


def _collect_observations(
    client: QualificationObserver,
    *,
    operation: str,
    source_commit: str,
    tree: str,
    count: int,
) -> list[dict[str, Any]]:
    events = []
    for _ in range(count):
        events.append(client.request(operation, source_commit, tree))
    return events


def _validate_execution_events(
    events: list[dict[str, Any]],
    context: dict[str, Any],
    expected_members: list[str],
    operation: str,
) -> None:
    lane = context["lane"]
    config = context["configuration"]
    for event in events:
        validate_capture_payload(
            event,
            lane=lane,
            expected_members=expected_members,
            config=config,
            operation=operation,
        )


def run_qualification_preflight(context_path: Path) -> int:
    loaded = _load_or_reject(context_path, "preflight")
    if isinstance(loaded, Reject):
        return finish("rejected", loaded.category, {}, [], "preflight", _reported_hash("0" * 64))
    context, context_hash, operation = loaded
    reported = _reported_hash(context_hash)
    try:
        _guard_qualification(context, context_hash)
        validate_qualification_schema(context)
        validate_tool(context)
        validate_dependencies(context)
        if context.get("candidate_success"):
            raise Reject("CLOSURE")
        return finish("passed", None, {"validated": True}, [], operation, reported)
    except Reject as error:
        return finish("rejected", error.category, {}, [], operation, reported)


def run_qualification_required(context_path: Path) -> int:
    loaded = _load_or_reject(context_path, "required")
    if isinstance(loaded, Reject):
        return finish("rejected", loaded.category, {}, [], "required", _reported_hash("0" * 64))
    context, context_hash, operation = loaded
    reported = _reported_hash(context_hash)
    try:
        _guard_qualification(context, context_hash)
        validate_qualification_schema(context)
        members = validate_required_members(context)
        return finish("passed", None, {"members": members}, [], operation, reported)
    except Reject as error:
        return finish("rejected", error.category, {}, [], operation, reported)


def run_qualification_oracle(context_path: Path, namespace: str | None) -> int:
    loaded = _load_or_reject(context_path, "oracle")
    if isinstance(loaded, Reject):
        return finish("rejected", loaded.category, {}, [], "oracle", _reported_hash("0" * 64))
    context, context_hash, operation = loaded
    reported = _reported_hash(context_hash)
    events: list[dict[str, Any]] = []
    digests: list[str] = []
    try:
        _guard_qualification(context, context_hash)
        if namespace != "synthetic":
            raise Reject("PROTOCOL")
        validate_source_authority(context, operation)
        validate_qualification_schema(context)
        expected_members = validate_required_members(context)
        qualification = context.get("qualification")
        client = QualificationObserver.from_env()
        if qualification and qualification.get("family") == "native":
            events = _collect_observations(
                client,
                operation=operation,
                source_commit=os.environ["TC_PROOF_ORACLE_COMMIT"],
                tree=os.environ["TC_PROOF_SOURCE_TREE"],
                count=1,
            )
            digests = [client.digest(event) for event in events]
            validate_native_extension(events[0]["payload"], qualification, context["oracle_commit"])
            return finish("passed", None, {"observations": events}, digests, operation, reported)
        events = _collect_observations(
            client,
            operation=operation,
            source_commit=os.environ["TC_PROOF_ORACLE_COMMIT"],
            tree=os.environ["TC_PROOF_SOURCE_TREE"],
            count=2,
        )
        digests = [client.digest(event) for event in events]
        validate_oracle_repeat(events)
        _validate_execution_events(events, context, expected_members, operation)
        return finish("passed", None, {"observations": events}, digests, operation, reported)
    except Reject as error:
        return finish("rejected", error.category, {}, digests, operation, reported)
    except ObserverError:
        return finish("rejected", "EXECUTION", {}, digests, operation, reported)


def run_qualification_capture(context_path: Path, lane: str | None) -> int:
    loaded = _load_or_reject(context_path, "capture")
    if isinstance(loaded, Reject):
        return finish("rejected", loaded.category, {}, [], "capture", _reported_hash("0" * 64))
    context, context_hash, operation = loaded
    reported = _reported_hash(context_hash)
    events: list[dict[str, Any]] = []
    digests: list[str] = []
    try:
        _guard_qualification(context, context_hash)
        if lane != context["lane"]:
            raise Reject("PROTOCOL")
        validate_source_authority(context, operation)
        validate_qualification_schema(context)
        expected_members = validate_required_members(context)
        client = QualificationObserver.from_env()
        events = _collect_observations(
            client,
            operation=operation,
            source_commit=os.environ["TC_PROOF_ORACLE_COMMIT"],
            tree=os.environ["TC_PROOF_SOURCE_TREE"],
            count=1,
        )
        digests = [client.digest(event) for event in events]
        _validate_execution_events(events, context, expected_members, operation)
        return finish("passed", None, {"observations": events}, digests, operation, reported)
    except Reject as error:
        return finish("rejected", error.category, {}, digests, operation, reported)
    except ObserverError:
        return finish("rejected", "EXECUTION", {}, digests, operation, reported)


def run_qualification_close(context_path: Path) -> int:
    loaded = _load_or_reject(context_path, "close")
    if isinstance(loaded, Reject):
        return finish("rejected", loaded.category, {}, [], "close", _reported_hash("0" * 64))
    context, context_hash, operation = loaded
    reported = _reported_hash(context_hash)
    events: list[dict[str, Any]] = []
    digests: list[str] = []
    try:
        if context.get("candidate_success"):
            raise Reject("CLOSURE")
        _guard_qualification(context, context_hash)
        validate_qualification_schema(context)
        if os.environ.get("TC_PROOF_CONTEXT_INDEX"):
            index, _ = qualification_load_index()
            client = QualificationObserver.from_env()
            events = _collect_observations(
                client,
                operation=operation,
                source_commit=os.environ["TC_PROOF_ORACLE_COMMIT"],
                tree=os.environ["TC_PROOF_SOURCE_TREE"],
                count=1,
            )
            digests = [client.digest(event) for event in events]
            qualification_validate_close_records(events[0])
            output_dir = Path(os.environ["TC_PROOF_RESULT"]).parent
            qualification_validate_index_close_outputs(events[0], index, output_dir)
            return finish("passed", None, {"observations": events}, digests, operation, reported)
        validate_close_evidence(context)
        client = QualificationObserver.from_env()
        events = _collect_observations(
            client,
            operation=operation,
            source_commit=os.environ["TC_PROOF_ORACLE_COMMIT"],
            tree=os.environ["TC_PROOF_SOURCE_TREE"],
            count=1,
        )
        digests = [client.digest(event) for event in events]
        return finish("passed", None, {"observations": events}, digests, operation, reported)
    except Reject as error:
        return finish("rejected", error.category, {}, digests, operation, reported)
    except ObserverError:
        return finish("rejected", "CLOSURE", {}, digests, operation, reported)


def qualification_dispatch(args: Any) -> int:
    account_tests = globals().get("run_account_tests")
    if account_tests is None:
        from .dispatch import run_account_tests as account_tests

    dispatch = {
        "preflight": lambda: run_qualification_preflight(args.context),
        "required": lambda: run_qualification_required(args.context),
        "oracle": lambda: run_qualification_oracle(args.context, args.namespace),
        "capture": lambda: run_qualification_capture(args.context, args.lane),
        "account-tests": lambda: account_tests(args.context),
        "close": lambda: run_qualification_close(args.context),
    }
    handler = dispatch.get(args.operation)
    if handler is None:
        import sys

        print(f"tc-proof: unknown command: {args.operation}", file=sys.stderr)
        return 2

    previous_save = globals().get("save_path")
    if callable(previous_save):
        def _qualification_save_path(path: Any, value: Any) -> None:
            Path(path).write_bytes(canonical(value))

        globals()["save_path"] = _qualification_save_path
    try:
        return handler()
    finally:
        if callable(previous_save):
            globals()["save_path"] = previous_save
