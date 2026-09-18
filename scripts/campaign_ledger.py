"""Shared campaign-ledger vocabulary used by native preflight and its self-test."""

from __future__ import annotations

from typing import Any


ACCEPTED_TASK_STATUSES = frozenset({"verified", "integrated"})
ACCEPTED_VERIFIER_VERDICT = "VERIFIED"


def accepted_verifier_rows(ledger: dict[str, Any]) -> list[dict[str, Any]]:
    """Return schema-valid task rows that carry accepted verifier evidence."""

    return [
        row
        for row in ledger.get("tasks", [])
        if row.get("status") in ACCEPTED_TASK_STATUSES
        and row.get("verifier_verdict") == ACCEPTED_VERIFIER_VERDICT
    ]


def validate_preflight_ledger(ledger: dict[str, Any], integration_branch: str) -> None:
    """Validate the ledger fields needed before native campaign dispatch."""

    assert ledger.get("schema") == "campaign-ledger/v1"
    expected_ref = "refs/heads/" + integration_branch
    assert ledger.get("integration_ref") == expected_ref, (
        ledger.get("integration_ref"),
        expected_ref,
    )
    assert ledger.get("armed") is False, "ledger shows armed=true"
    assert ledger["catalog"]["commit"] != "REPLACE_AT_INIT"
    assert accepted_verifier_rows(ledger), "no current accepted verifier receipt exists"


if __name__ == "__main__":
    raise SystemExit("import campaign_ledger; do not run this module directly")
