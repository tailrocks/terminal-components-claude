"""Observer IPC client for the host-supervised inherited-pipe ABI."""

from __future__ import annotations

import os
from typing import Any

from .json_util import canonical, load_bytes, sha256_canonical

MAX_REQUEST_BYTES = 4096
MAX_RESPONSE_BYTES = 10_000_000


class ObserverError(Exception):
    pass


class ObserverClient:
    def __init__(self, nonce: str, run_id: str, task_id: str, check_id: str, request_fd: int, response_fd: int) -> None:
        self.nonce = nonce
        self.run_id = run_id
        self.task_id = task_id
        self.check_id = check_id
        self.request_id = 0
        self._request = os.fdopen(request_fd, "wb")
        self._response = os.fdopen(response_fd, "rb")

    @classmethod
    def from_env(cls) -> ObserverClient:
        nonce = os.environ.get("TC_PROOF_OBSERVER_NONCE")
        request_fd = os.environ.get("TC_PROOF_OBSERVER_REQUEST_FD")
        response_fd = os.environ.get("TC_PROOF_OBSERVER_RESPONSE_FD")
        if not nonce or request_fd is None or response_fd is None:
            raise ObserverError("observer transport unavailable")
        if os.environ.get("TC_PROOF_OBSERVER_SOCKET"):
            raise ObserverError("observer socket transport is retired")
        try:
            return cls(
                nonce,
                os.environ["TC_PROOF_RUN_ID"],
                os.environ["TC_PROOF_TASK_ID"],
                os.environ["TC_PROOF_CHECK_ID"],
                int(request_fd),
                int(response_fd),
            )
        except (OSError, ValueError) as error:
            raise ObserverError(str(error)) from error

    def request(self, operation: str, source_commit: str, tree: str) -> dict[str, Any]:
        payload = {
            "schema": "tc-proof-runner-observe/v1",
            "nonce": self.nonce,
            "run_id": self.run_id,
            "task_id": self.task_id,
            "check_id": self.check_id,
            "request_id": self.request_id,
            "operation": operation,
            "source_commit": source_commit,
            "tree": tree,
        }
        line = canonical(payload) + b"\n"
        if len(line) > MAX_REQUEST_BYTES:
            raise ObserverError("observer request exceeds 4096-byte bound")
        self._request.write(line)
        self._request.flush()
        raw = self._response.readline(MAX_RESPONSE_BYTES + 1)
        if not raw or not raw.endswith(b"\n"):
            raise ObserverError("observer response incomplete")
        if len(raw) > MAX_RESPONSE_BYTES:
            raise ObserverError("observer response exceeds maximum bound")
        event = load_bytes(raw[:-1])
        if not isinstance(event, dict) or event.get("schema") != "tc-proof-observation/v1":
            raise ObserverError("observer response schema mismatch")
        for key, expected in {
            "nonce": self.nonce,
            "run_id": self.run_id,
            "task_id": self.task_id,
            "check_id": self.check_id,
            "request_id": self.request_id,
            "operation": operation,
            "source_commit": source_commit,
            "tree": tree,
        }.items():
            if event.get(key) != expected:
                raise ObserverError(f"observer response binding mismatch: {key}")
        if not isinstance(event.get("payload"), dict) or not isinstance(event.get("records"), list):
            raise ObserverError("observer response payload is invalid")
        self.request_id += 1
        return event

    def digest(self, event: dict[str, Any]) -> str:
        return sha256_canonical(event)
