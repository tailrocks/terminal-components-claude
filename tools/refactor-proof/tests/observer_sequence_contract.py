#!/usr/bin/env python3
"""Deterministic observer-sequence contract tests.

These tests exercise the runner-side contract independently of the Rust
materializer.  The Rust seam is intentionally left explicit: it must emit the
same sequence into each hashed context before the bundled worker can accept it.
"""

from __future__ import annotations

import json
import os
import sys
import tempfile
import threading
import unittest
from pathlib import Path
from typing import Callable
from unittest.mock import patch

PROOF_ROOT = Path(__file__).resolve().parents[1]
if str(PROOF_ROOT) not in sys.path:
    sys.path.insert(0, str(PROOF_ROOT))

from runner.context import (  # noqa: E402
    Reject,
    bind_context,
    load_context,
    validate_observer_sequence,
    validate_oracle_sequence,
)
from runner.json_util import canonical, sha256_bytes  # noqa: E402
from runner.observer import ObserverClient, ObserverError  # noqa: E402


RUN_ID = "run-observer-sequence"
TASK_ID = "TASK-071"
CHECK_ID = "CHK-004"
SOURCE_COMMIT = "a" * 40
SOURCE_TREE = "b" * 40
NONCE = "observer-sequence-nonce"


def _context(sequence: list[str], family: str | None = None) -> dict[str, object]:
    qualification: dict[str, object] = {}
    if family is not None:
        qualification["family"] = family
    return {
        "schema": "tc-proof-context/v1",
        "run_id": RUN_ID,
        "task_id": TASK_ID,
        "check_id": CHECK_ID,
        "operation": "oracle",
        "observer_sequence": sequence,
        "qualification": qualification,
    }


def _event(request: dict[str, object]) -> dict[str, object]:
    return {
        "schema": "tc-proof-observation/v1",
        "nonce": request["nonce"],
        "run_id": request["run_id"],
        "task_id": request["task_id"],
        "check_id": request["check_id"],
        "request_id": request["request_id"],
        "operation": request["operation"],
        "source_commit": request["source_commit"],
        "tree": request["tree"],
        "exit": 0,
        "stdout": "",
        "stderr": "",
        "files": {},
        "payload": {"request_id": request["request_id"]},
        "records": [{"request_id": request["request_id"], "status": "observed"}],
    }


def _client_with_provider(
    sequence: list[str],
    *,
    response_count: int | None = None,
    mutate: Callable[[dict[str, object]], dict[str, object]] | None = None,
) -> tuple[ObserverClient, threading.Thread]:
    request_read, request_write = os.pipe()
    response_read, response_write = os.pipe()
    count = len(sequence) if response_count is None else response_count

    def serve() -> None:
        with os.fdopen(request_read, "rb") as incoming, os.fdopen(response_write, "wb") as outgoing:
            for _ in range(count):
                raw = incoming.readline()
                if not raw:
                    return
                request = json.loads(raw)
                event = _event(request)
                if mutate is not None:
                    event = mutate(event)
                outgoing.write(canonical(event) + b"\n")
                outgoing.flush()

    thread = threading.Thread(target=serve, daemon=True)
    thread.start()
    return (
        ObserverClient(NONCE, RUN_ID, TASK_ID, CHECK_ID, request_write, response_read, sequence),
        thread,
    )


def _close_client(client: ObserverClient) -> None:
    client.close()


class ObserverSequenceTests(unittest.TestCase):
    def test_native_oracle_declares_one_request(self) -> None:
        self.assertEqual(
            validate_oracle_sequence(_context(["oracle"], "native"), "oracle"),
            ["oracle"],
        )

    def test_synthetic_oracle_declares_two_requests(self) -> None:
        self.assertEqual(
            validate_oracle_sequence(
                _context(["oracle", "oracle"], "synthetic"), "oracle"
            ),
            ["oracle", "oracle"],
        )

    def test_missing_oracle_family_is_rejected(self) -> None:
        with self.assertRaises(Reject) as raised:
            validate_oracle_sequence(_context(["oracle", "oracle"]), "oracle")
        self.assertEqual(raised.exception.category, "PROTOCOL")

    def test_unknown_oracle_family_is_rejected(self) -> None:
        with self.assertRaises(Reject) as raised:
            validate_oracle_sequence(_context(["oracle", "oracle"], "accounting"), "oracle")
        self.assertEqual(raised.exception.category, "PROTOCOL")

    def test_missing_sequence_is_rejected(self) -> None:
        context = _context(["oracle"])
        del context["observer_sequence"]
        with self.assertRaises(Reject) as raised:
            validate_observer_sequence(context, "oracle")
        self.assertEqual(raised.exception.category, "PROTOCOL")

    def test_extra_oracle_request_is_rejected(self) -> None:
        with self.assertRaises(Reject) as raised:
            validate_oracle_sequence(_context(["oracle", "oracle", "oracle"]), "oracle")
        self.assertEqual(raised.exception.category, "PROTOCOL")

    def test_reordered_sequence_is_rejected(self) -> None:
        with self.assertRaises(Reject) as raised:
            validate_observer_sequence(_context(["capture", "oracle"]), "oracle")
        self.assertEqual(raised.exception.category, "PROTOCOL")

    def test_native_family_cannot_claim_synthetic_sequence(self) -> None:
        with self.assertRaises(Reject) as raised:
            validate_oracle_sequence(_context(["oracle", "oracle"], "native"), "oracle")
        self.assertEqual(raised.exception.category, "PROTOCOL")

    def test_synthetic_family_cannot_claim_native_sequence(self) -> None:
        with self.assertRaises(Reject) as raised:
            validate_oracle_sequence(_context(["oracle"], "synthetic"), "oracle")
        self.assertEqual(raised.exception.category, "PROTOCOL")

    def test_missing_second_observation_is_rejected(self) -> None:
        client, thread = _client_with_provider(["oracle", "oracle"], response_count=1)
        try:
            client.request("oracle", SOURCE_COMMIT, SOURCE_TREE)
            with self.assertRaises(ObserverError):
                client.request("oracle", SOURCE_COMMIT, SOURCE_TREE)
        finally:
            _close_client(client)
            thread.join(timeout=2)
            self.assertFalse(thread.is_alive())

    def test_extra_request_is_rejected_before_transport(self) -> None:
        client, thread = _client_with_provider(["oracle"])
        try:
            client.request("oracle", SOURCE_COMMIT, SOURCE_TREE)
            with self.assertRaises(ObserverError):
                client.request("oracle", SOURCE_COMMIT, SOURCE_TREE)
            client.require_complete()
        finally:
            _close_client(client)
            thread.join(timeout=2)
            self.assertFalse(thread.is_alive())

    def test_wrong_id_is_rejected(self) -> None:
        def mutate(event: dict[str, object]) -> dict[str, object]:
            event["request_id"] = 99
            return event

        client, thread = _client_with_provider(["oracle"], mutate=mutate)
        try:
            with self.assertRaises(ObserverError):
                client.request("oracle", SOURCE_COMMIT, SOURCE_TREE)
        finally:
            _close_client(client)
            thread.join(timeout=2)
            self.assertFalse(thread.is_alive())

    def test_wrong_nonce_is_rejected(self) -> None:
        def mutate(event: dict[str, object]) -> dict[str, object]:
            event["nonce"] = "replayed-nonce"
            return event

        client, thread = _client_with_provider(["oracle"], mutate=mutate)
        try:
            with self.assertRaises(ObserverError):
                client.request("oracle", SOURCE_COMMIT, SOURCE_TREE)
        finally:
            _close_client(client)
            thread.join(timeout=2)
            self.assertFalse(thread.is_alive())

    def test_cross_run_response_is_rejected(self) -> None:
        def mutate(event: dict[str, object]) -> dict[str, object]:
            event["run_id"] = "another-run"
            return event

        client, thread = _client_with_provider(["oracle"], mutate=mutate)
        try:
            with self.assertRaises(ObserverError):
                client.request("oracle", SOURCE_COMMIT, SOURCE_TREE)
        finally:
            _close_client(client)
            thread.join(timeout=2)
            self.assertFalse(thread.is_alive())

    def test_mutated_sequence_after_hash_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory(prefix="tc-proof-sequence-") as directory:
            path = Path(directory) / "CHK-004.json"
            original = _context(["oracle", "oracle"])
            path.write_bytes(canonical(original) + b"\n")
            original_hash = sha256_bytes(path.read_bytes())
            mutated = dict(original)
            mutated["observer_sequence"] = ["oracle"]
            path.write_bytes(canonical(mutated) + b"\n")
            with patch.dict(
                os.environ,
                {"TC_PROOF_CONTEXT_SHA256": original_hash, "TC_PROOF_RUN_ID": RUN_ID},
                clear=False,
            ):
                _, _, mutated_hash = load_context(path)
                with self.assertRaises(Reject) as raised:
                    bind_context(mutated, mutated_hash)
            self.assertEqual(raised.exception.category, "INTEGRITY")


if __name__ == "__main__":
    unittest.main()
