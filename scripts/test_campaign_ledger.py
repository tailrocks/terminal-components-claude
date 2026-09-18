"""Deterministic contract checks for campaign-preflight ledger acceptance."""

from __future__ import annotations

from campaign_ledger import accepted_verifier_rows, validate_preflight_ledger


BRANCH = "refactor/holla-parity"


def base_ledger() -> dict[str, object]:
    return {
        "schema": "campaign-ledger/v1",
        "integration_ref": f"refs/heads/{BRANCH}",
        "armed": False,
        "catalog": {"commit": "catalog-sha"},
        "tasks": [],
    }


def main() -> None:
    ledger = base_ledger()
    assert not accepted_verifier_rows(ledger)
    try:
        validate_preflight_ledger(ledger, BRANCH)
    except AssertionError as error:
        assert str(error) == "no current accepted verifier receipt exists"
    else:
        raise AssertionError("empty ledger unexpectedly passed preflight")

    ledger["tasks"] = [{"status": "verified", "verifier_verdict": "VERIFIED"}]
    validate_preflight_ledger(ledger, BRANCH)

    ledger["tasks"] = [{"status": "integrated", "verifier_verdict": "VERIFIED"}]
    validate_preflight_ledger(ledger, BRANCH)

    for row in (
        {"status": "accepted", "verifier_verdict": "PASS"},
        {"status": "verified", "verifier_verdict": "PASS"},
        {"status": "pending", "verifier_verdict": "VERIFIED"},
    ):
        assert not accepted_verifier_rows({"tasks": [row]})

    print("campaign ledger acceptance contract: PASS")


if __name__ == "__main__":
    main()
