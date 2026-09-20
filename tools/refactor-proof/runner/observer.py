"""Observer IPC client for the host-supervised inherited-pipe ABI."""

from __future__ import annotations

import os
from typing import Any

from .json_util import canonical, load_bytes, sha256_canonical

MAX_REQUEST_BYTES = 4096
MAX_RESPONSE_BYTES = 10_000_000
OBSERVATION_KEYS = {
    "schema",
    "nonce",
    "run_id",
    "task_id",
    "check_id",
    "request_id",
    "operation",
    "source_commit",
    "tree",
    "exit",
    "stdout",
    "stderr",
    "files",
    "payload",
    "records",
}


class ObserverError(Exception):
    pass


class ObserverClient:
    def __init__(
        self,
        nonce: str,
        run_id: str,
        task_id: str,
        check_id: str,
        request_fd: int,
        response_fd: int,
        observer_sequence: list[str],
    ) -> None:
        if not observer_sequence or any(not isinstance(item, str) for item in observer_sequence):
            raise ObserverError("observer sequence is invalid")
        self.nonce = nonce
        self.run_id = run_id
        self.task_id = task_id
        self.check_id = check_id
        self.observer_sequence = tuple(observer_sequence)
        self.request_id = 0
        self._request = os.fdopen(request_fd, "wb")
        self._response = os.fdopen(response_fd, "rb")

    @classmethod
    def from_env(cls, observer_sequence: list[str]) -> ObserverClient:
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
                observer_sequence,
            )
        except (OSError, ValueError) as error:
            raise ObserverError(str(error)) from error

    def request(self, operation: str, source_commit: str, tree: str) -> dict[str, Any]:
        if self.request_id >= len(self.observer_sequence):
            raise ObserverError("observer request sequence exhausted")
        expected_operation = self.observer_sequence[self.request_id]
        if operation != expected_operation:
            raise ObserverError("observer request sequence mismatch")
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
        try:
            self._request.write(line)
            self._request.flush()
            raw = self._response.readline(MAX_RESPONSE_BYTES + 1)
        except OSError as error:
            raise ObserverError("observer transport failed") from error
        if not raw or not raw.endswith(b"\n"):
            raise ObserverError("observer response incomplete")
        if len(raw) > MAX_RESPONSE_BYTES:
            raise ObserverError("observer response exceeds maximum bound")
        event = load_bytes(raw[:-1])
        if (
            not isinstance(event, dict)
            or set(event) != OBSERVATION_KEYS
            or event.get("schema") != "tc-proof-observation/v1"
        ):
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
        if event.get("exit") != 0:
            raise ObserverError("observer response reports a failed observation")
        if not isinstance(event.get("stdout"), str) or not isinstance(event.get("stderr"), str):
            raise ObserverError("observer response streams are invalid")
        files = event.get("files")
        if not isinstance(files, dict) or any(
            not isinstance(name, str) or not isinstance(value, str)
            for name, value in files.items()
        ):
            raise ObserverError("observer response files are invalid")
        if (
            not isinstance(event.get("payload"), dict)
            or not event["payload"]
            or not isinstance(event.get("records"), list)
            or not event["records"]
        ):
            raise ObserverError("observer response payload is invalid")
        self.request_id += 1
        return event

    def require_complete(self) -> None:
        if self.request_id != len(self.observer_sequence):
            raise ObserverError("observer request sequence incomplete")

    def close(self) -> None:
        for stream in (self._request, self._response):
            try:
                stream.close()
            except OSError:
                pass

    def digest(self, event: dict[str, Any]) -> str:
        return sha256_canonical(event)
