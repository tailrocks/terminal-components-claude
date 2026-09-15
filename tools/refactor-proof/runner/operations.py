"""tc-proof runner operations."""

from __future__ import annotations

import os
from pathlib import Path
from typing import Any

from .context import (
    Reject,
    bind_context,
    load_context,
    validate_close_evidence,
    validate_dependencies,
    validate_required_members,
    validate_schema,
    validate_source_authority,
    validate_tool,
)
from .index import validate_index
from .observer import ObserverClient, ObserverError
from .result import finish
from .index import load_index
from .validate import (
    validate_capture_payload,
    validate_close_records,
    validate_index_close_outputs,
    validate_native_extension,
    validate_oracle_repeat,
)


def _reported_hash(context_hash: str) -> str:
    return os.environ.get("TC_PROOF_CONTEXT_SHA256", context_hash)


def _guard_context(context: dict[str, Any], context_hash: str) -> None:
    if os.environ.get("TC_PROOF_CONTEXT_INDEX"):
        validate_index(context, context_hash)
        return
    bind_context(context, context_hash)


def _load_or_reject(context_path: Path, default_operation: str) -> tuple[dict[str, Any], str, str] | Reject:
    try:
        context, _, context_hash = load_context(context_path)
    except Reject as error:
        return error
    return context, context_hash, context.get("operation", default_operation)


def run_preflight(context_path: Path) -> int:
    loaded = _load_or_reject(context_path, "preflight")
    if isinstance(loaded, Reject):
        return finish("rejected", loaded.category, {}, [], "preflight", _reported_hash("0" * 64))
    context, context_hash, operation = loaded
    reported = _reported_hash(context_hash)
    try:
        _guard_context(context, context_hash)
        validate_schema(context)
        validate_tool(context)
        validate_dependencies(context)
        if context.get("candidate_success"):
            raise Reject("CLOSURE")
        return finish("passed", None, {"validated": True}, [], operation, reported)
    except Reject as error:
        return finish("rejected", error.category, {}, [], operation, reported)


def run_required(context_path: Path) -> int:
    loaded = _load_or_reject(context_path, "required")
    if isinstance(loaded, Reject):
        return finish("rejected", loaded.category, {}, [], "required", _reported_hash("0" * 64))
    context, context_hash, operation = loaded
    reported = _reported_hash(context_hash)
    try:
        _guard_context(context, context_hash)
        validate_schema(context)
        members = validate_required_members(context)
        return finish("passed", None, {"members": members}, [], operation, reported)
    except Reject as error:
        return finish("rejected", error.category, {}, [], operation, reported)


def _collect_observations(
    client: ObserverClient,
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


def run_oracle(context_path: Path, namespace: str | None) -> int:
    loaded = _load_or_reject(context_path, "oracle")
    if isinstance(loaded, Reject):
        return finish("rejected", loaded.category, {}, [], "oracle", _reported_hash("0" * 64))
    context, context_hash, operation = loaded
    reported = _reported_hash(context_hash)
    events: list[dict[str, Any]] = []
    digests: list[str] = []
    try:
        _guard_context(context, context_hash)
        if namespace != "synthetic":
            raise Reject("PROTOCOL")
        validate_source_authority(context, operation)
        validate_schema(context)
        expected_members = validate_required_members(context)
        qualification = context.get("qualification")
        client = ObserverClient.from_env()
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


def run_capture(context_path: Path, lane: str | None) -> int:
    loaded = _load_or_reject(context_path, "capture")
    if isinstance(loaded, Reject):
        return finish("rejected", loaded.category, {}, [], "capture", _reported_hash("0" * 64))
    context, context_hash, operation = loaded
    reported = _reported_hash(context_hash)
    events: list[dict[str, Any]] = []
    digests: list[str] = []
    try:
        _guard_context(context, context_hash)
        if lane != context["lane"]:
            raise Reject("PROTOCOL")
        validate_source_authority(context, operation)
        validate_schema(context)
        expected_members = validate_required_members(context)
        client = ObserverClient.from_env()
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


def run_close(context_path: Path) -> int:
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
        _guard_context(context, context_hash)
        validate_schema(context)
        if os.environ.get("TC_PROOF_CONTEXT_INDEX"):
            index, _ = load_index()
            client = ObserverClient.from_env()
            events = _collect_observations(
                client,
                operation=operation,
                source_commit=os.environ["TC_PROOF_ORACLE_COMMIT"],
                tree=os.environ["TC_PROOF_SOURCE_TREE"],
                count=1,
            )
            digests = [client.digest(event) for event in events]
            validate_close_records(events[0])
            output_dir = Path(os.environ["TC_PROOF_RESULT"]).parent
            validate_index_close_outputs(events[0], index, output_dir)
            return finish("passed", None, {"observations": events}, digests, operation, reported)
        validate_close_evidence(context)
        client = ObserverClient.from_env()
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
