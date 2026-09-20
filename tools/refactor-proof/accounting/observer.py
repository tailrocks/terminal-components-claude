"""Five-field qualification observer for tc-proof-runner-observe/v1."""

from __future__ import annotations

import os
from typing import Any

from ..runner.json_util import canonical, load_bytes, sha256_canonical
from ..runner.observer import ObserverError

MAX_REQUEST_BYTES = 4096
MAX_RESPONSE_BYTES = 10_000_000


class QualificationObserver:
    """Sandbox-qualification observer: exactly the five-field request ABI."""

    def __init__(self, nonce: str, request_fd: int, response_fd: int) -> None:
        self.nonce = nonce
        self._request = os.fdopen(request_fd, "wb")
        self._response = os.fdopen(response_fd, "rb")

    @classmethod
    def from_env(cls) -> QualificationObserver:
        nonce = os.environ.get("TC_PROOF_OBSERVER_NONCE")
        request_fd = os.environ.get("TC_PROOF_OBSERVER_REQUEST_FD")
        response_fd = os.environ.get("TC_PROOF_OBSERVER_RESPONSE_FD")
        if not nonce or request_fd is None or response_fd is None:
            raise ObserverError("observer transport unavailable")
        try:
            return cls(nonce, int(request_fd), int(response_fd))
        except OSError as error:
            raise ObserverError(str(error)) from error

    def request(self, operation: str, source_commit: str, tree: str) -> dict[str, Any]:
        payload = {
            "schema": "tc-proof-runner-observe/v1",
            "nonce": self.nonce,
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
        if "error" in event:
            raise ObserverError(event["error"])
        return event

    def digest(self, event: dict[str, Any]) -> str:
        return sha256_canonical(event)
