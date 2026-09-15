"""tc-proof-runner-result/v1 writer."""

from __future__ import annotations

import os
import sys
from typing import Any

from .json_util import save_path


REPORT_KEYS = {
    "schema",
    "run_id",
    "operation",
    "context_sha256",
    "status",
    "category",
    "observation_digests",
    "outputs",
}


def write_result(
    *,
    operation: str,
    context_sha256: str,
    status: str,
    category: str | None,
    observation_digests: list[str],
    outputs: dict[str, Any],
) -> None:
    result_path = os.environ.get("TC_PROOF_RESULT")
    if not result_path:
        raise RuntimeError("TC_PROOF_RESULT not set")
    report = {
        "schema": "tc-proof-runner-result/v1",
        "run_id": os.environ["TC_PROOF_RUN_ID"],
        "operation": operation,
        "context_sha256": context_sha256,
        "status": status,
        "category": category,
        "observation_digests": observation_digests,
        "outputs": outputs,
    }
    if set(report) != REPORT_KEYS:
        raise RuntimeError("internal result schema mismatch")
    save_path(result_path, report)


def finish(status: str, category: str | None, outputs: dict[str, Any], digests: list[str], operation: str, context_sha256: str) -> int:
    write_result(
        operation=operation,
        context_sha256=context_sha256,
        status=status,
        category=category,
        observation_digests=digests,
        outputs=outputs,
    )
    return 1 if status == "rejected" else 0
