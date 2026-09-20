"""architecture operation entry point."""

from __future__ import annotations

import os
from pathlib import Path
from typing import Any

from ..accounting.observer import QualificationObserver
from ..accounting.qualification import is_qualification_schema, validate_qualification_schema
from ..runner.context import Reject, bind_context, load_context, validate_schema, validate_source_authority
from ..runner.observer import ObserverClient, ObserverError
from ..runner.result import finish
from .actual import validate_actual_event
from .broker import ADJ13_POLICY, validate_broker_event
from .extension import validate_performance_event
from .fixture import validate_fixture_event
from .rust_model import analyze_standalone, validate_runtime, validate_standalone_event
from .source import validate_cargo_dependencies, validate_external_consumers, validate_source_policy
from .style_timing import validate_style_timing_event


def _reported_hash(context_hash: str) -> str:
    return os.environ.get("TC_PROOF_CONTEXT_SHA256", context_hash)


def _observer_for(context: dict[str, Any], operation: str) -> QualificationObserver | ObserverClient:
    if is_qualification_schema(context):
        return QualificationObserver.from_env()
    sequence = context.get("observer_sequence")
    if not isinstance(sequence, list) or not sequence:
        sequence = [operation]
    return ObserverClient.from_env(sequence)


def _validate_architecture_context(context: dict[str, Any]) -> None:
    profile = context.pop("architecture_profile", None)
    branch_host_projection = context.pop("branch_host_projection", None)
    try:
        if is_qualification_schema(context):
            validate_qualification_schema(context)
        else:
            validate_schema(context)
    finally:
        if profile is not None:
            context["architecture_profile"] = profile
        if branch_host_projection is not None:
            context["branch_host_projection"] = branch_host_projection


def _validate_event(event: dict[str, Any], context: dict[str, Any], profile: dict[str, Any] | None) -> None:
    qualification = context.get("qualification") or {}
    if qualification.get("family") == "performance":
        validate_performance_event(event, qualification)
        return
    if profile is None:
        validate_fixture_event(event, context.get("configuration", {}))
        return
    if profile.get("policy") == ADJ13_POLICY:
        validate_broker_event(event, profile)
        return
    schema = profile.get("schema")
    if schema == "tc-architecture-rust-profile/v1":
        validate_standalone_event(event, profile)
        return
    if schema == "tc-architecture-actual-rust-profile/v1":
        validate_actual_event(event, profile)
        return
    if schema == "tc-style-timing-actual-profile/v1":
        validate_style_timing_event(event, profile)
        return
    if schema == "tc-architecture-source-profile/v1":
        kind = profile.get("kind")
        if kind == "external-consumers":
            validate_external_consumers(event, profile)
        elif kind == "cargo-dependencies":
            validate_cargo_dependencies(event, profile)
        elif kind == "source-policy":
            validate_source_policy(event, profile)
        elif kind == "executable-example":
            if event.get("exit") != 0:
                raise Reject("ARCHITECTURE")
        else:
            raise Reject("ARCHITECTURE")
        return
    raise Reject("ARCHITECTURE")


def _has_custom_art(profile: dict[str, Any]) -> bool:
    for root in profile.get("source_roots", []):
        path = Path(root)
        if path.is_file() and path.name == "app.rs":
            return "ui.paint(3, 35)" in path.read_text()
        candidate = path / "app.rs"
        if candidate.is_file() and "ui.paint(3, 35)" in candidate.read_text():
            return True
    return False


def run_architecture(context_path: Path) -> int:
    try:
        context, _, context_hash = load_context(context_path)
    except Reject as error:
        return finish("rejected", error.category, {}, [], "architecture", _reported_hash("0" * 64))

    operation = context.get("operation", "architecture")
    reported = _reported_hash(context_hash)
    events: list[dict[str, Any]] = []
    digests: list[str] = []

    try:
        bind_context(context, context_hash)
        _validate_architecture_context(context)
        validate_source_authority(context, operation)
        profile = context.get("architecture_profile")
        client = _observer_for(context, operation)
        event = client.request(
            operation,
            os.environ["TC_PROOF_ORACLE_COMMIT"],
            os.environ["TC_PROOF_SOURCE_TREE"],
        )
        events = [event]
        digests = [client.digest(event)]
        _validate_event(event, context, profile if isinstance(profile, dict) else None)
        return finish("passed", None, {"observations": events}, digests, operation, reported)
    except Reject as error:
        return finish("rejected", error.category, {}, digests, operation, reported)
    except ObserverError:
        return finish("rejected", "ARCHITECTURE", {}, digests, operation, reported)
