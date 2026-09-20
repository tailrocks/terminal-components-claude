"""account-tests operation entry point."""

from __future__ import annotations

import os
from pathlib import Path
from typing import Any

from ..runner.context import Reject, bind_context, load_context, validate_schema, validate_source_authority
from ..runner.observer import ObserverClient, ObserverError
from ..runner.result import finish
from .extension import validate_extension_event
from .fixture import build_outputs, validate_event, validate_inventory_integrity
from .observer import QualificationObserver
from .qualification import is_qualification_schema, validate_qualification_schema


def _reported_hash(context_hash: str) -> str:
    return os.environ.get("TC_PROOF_CONTEXT_SHA256", context_hash)


def _observer_for(context: dict[str, Any], operation: str) -> QualificationObserver | ObserverClient:
    if is_qualification_schema(context):
        return QualificationObserver.from_env()
    sequence = context.get("observer_sequence")
    if not isinstance(sequence, list) or not sequence:
        sequence = [operation]
    try:
        return ObserverClient.from_env(sequence)
    except TypeError:
        return ObserverClient.from_env()


def run_account_tests(context_path: Path) -> int:
    try:
        context, _, context_hash = load_context(context_path)
    except Reject as error:
        return finish("rejected", error.category, {}, [], "account-tests", _reported_hash("0" * 64))

    operation = context.get("operation", "account-tests")
    reported = _reported_hash(context_hash)
    events: list[dict[str, Any]] = []
    digests: list[str] = []

    try:
        bind_context(context, context_hash)
        if is_qualification_schema(context):
            validate_qualification_schema(context)
        else:
            validate_schema(context)
        validate_source_authority(context, operation)

        qualification = context.get("qualification")
        if qualification and qualification.get("family") == "accounting":
            client = _observer_for(context, operation)
            event = client.request(
                operation,
                os.environ["TC_PROOF_ORACLE_COMMIT"],
                os.environ["TC_PROOF_SOURCE_TREE"],
            )
            events = [event]
            digests = [client.digest(event)]
            outputs = validate_extension_event(
                event,
                qualification,
                run_id=os.environ["TC_PROOF_RUN_ID"],
                source_commit=os.environ["TC_PROOF_ORACLE_COMMIT"],
            )
            return finish("passed", None, outputs, digests, operation, reported)

        inventory = context.get("inventory")
        if not isinstance(inventory, dict):
            raise Reject("TEST_ACCOUNTING")
        validate_inventory_integrity(inventory)

        client = _observer_for(context, operation)
        event = client.request(
            operation,
            os.environ["TC_PROOF_ORACLE_COMMIT"],
            os.environ["TC_PROOF_SOURCE_TREE"],
        )
        events = [event]
        digests = [client.digest(event)]
        validate_event(event, inventory, context.get("configuration", {}))
        outputs = build_outputs(events, inventory)
        return finish("passed", None, outputs, digests, operation, reported)
    except Reject as error:
        return finish("rejected", error.category, {}, digests, operation, reported)
    except ObserverError:
        return finish("rejected", "TEST_ACCOUNTING", {}, digests, operation, reported)
