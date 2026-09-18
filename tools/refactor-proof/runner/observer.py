"""Observer IPC client for tc-proof-runner-observe/v1."""

from __future__ import annotations

import os
import socket
from typing import Any

from .json_util import canonical, load_bytes, sha256_canonical

MAX_REQUEST_BYTES = 4096
MAX_RESPONSE_BYTES = 10_000_000


class ObserverError(Exception):
    pass


class ObserverClient:
    def __init__(self, nonce: str, request_fd: int | None = None, response_fd: int | None = None, socket_path: str | None = None) -> None:
        self.nonce = nonce
        self._socket: socket.socket | None = None
        if socket_path is not None:
            connection = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            connection.connect(socket_path)
            self._socket = connection
            self._request = connection.makefile("wb")
            self._response = connection.makefile("rb")
        elif request_fd is not None and response_fd is not None:
            self._request = os.fdopen(request_fd, "wb")
            self._response = os.fdopen(response_fd, "rb")
        else:
            raise ObserverError("observer transport unavailable")

    @classmethod
    def from_env(cls) -> ObserverClient:
        nonce = os.environ.get("TC_PROOF_OBSERVER_NONCE")
        request_fd = os.environ.get("TC_PROOF_OBSERVER_REQUEST_FD")
        response_fd = os.environ.get("TC_PROOF_OBSERVER_RESPONSE_FD")
        socket_path = os.environ.get("TC_PROOF_OBSERVER_SOCKET")
        if not nonce or ((request_fd is None or response_fd is None) and socket_path is None):
            raise ObserverError("observer transport unavailable")
        try:
            if socket_path is not None and (request_fd is None or response_fd is None):
                return cls(nonce, socket_path=socket_path)
            return cls(nonce, int(request_fd), int(response_fd))
        except (OSError, ValueError) as error:
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
