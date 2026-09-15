"""Qualification-contract accounting validation."""

from __future__ import annotations

from typing import Any

from ..runner.context import Reject


def _canonical(value: Any) -> bytes:
    import json

    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


def _expected_owner(identity: dict[str, Any]) -> str:
    package = identity["package"]
    if package == "tiny-a":
        return "terminal-components/completion/991"
    if package == "tiny-b":
        return "terminal-components/completion/992"
    raise Reject("TEST_ACCOUNTING")


def _outcomes_from_runs(runs: list[dict[str, Any]]) -> list[dict[str, Any]]:
    outcomes: list[dict[str, Any]] = []
    for run in runs:
        stdout = run.get("stdout") or run.get("result", {})
        if not isinstance(stdout, dict):
            raise Reject("TEST_ACCOUNTING")
        for result in stdout.get("results", []):
            identity = {key: run[key] for key in ("package", "target", "profile", "source_commit")}
            identity["name"] = result["name"]
            owner = _expected_owner(identity) if result["status"] != "passed" else None
            outcomes.append({"identity": identity, "status": result["status"], "owner": owner})
    return outcomes


def validate_extension_event(
    event: dict[str, Any],
    contract: dict[str, Any],
    *,
    run_id: str,
    source_commit: str,
) -> dict[str, Any]:
    payload = event.get("payload")
    if not isinstance(payload, dict):
        raise Reject("TEST_ACCOUNTING")

    original = contract["original"]
    actual = payload.get("source")
    if not isinstance(actual, dict):
        raise Reject("TEST_ACCOUNTING")

    if payload.get("archive_sha256") != original["source_sha256"]:
        raise Reject("TEST_ACCOUNTING")
    if actual.get("non_test_sha256") != original["non_test_sha256"]:
        raise Reject("TEST_ACCOUNTING")
    expected_assertions = contract.get("approved_replacements") or original["assertions"]
    if actual.get("assertions") != expected_assertions:
        raise Reject("TEST_ACCOUNTING")

    proposals = contract.get("proposals") or {}
    if proposals and set(proposals) != {"observed_inventory"}:
        raise Reject("TEST_ACCOUNTING")
    if "observed_inventory" in proposals:
        required = contract["required"]
        if sorted(map(_canonical, proposals["observed_inventory"])) != sorted(map(_canonical, required)):
            raise Reject("TEST_ACCOUNTING")

    if contract["mode"] == "production":
        if not contract.get("accepted_inventory") or not contract.get("accepted_disposition"):
            raise Reject("TEST_ACCOUNTING")
    else:
        register = contract.get("preparation_register")
        expected_future = contract.get("future", [])
        expected_register = {"required": contract["required"], "future": expected_future}
        if register != expected_register:
            raise Reject("TEST_ACCOUNTING")

    required_ids = {_canonical(identity) for identity in contract["required"]}
    future = {_canonical(row["identity"]): row for row in contract.get("future", [])}
    closed = {_canonical(identity) for identity in contract.get("closed", [])}

    if len(future) != len(contract.get("future", [])):
        raise Reject("TEST_ACCOUNTING")
    if len(closed) != len(contract.get("closed", [])):
        raise Reject("TEST_ACCOUNTING")
    if not set(future).issubset(required_ids):
        raise Reject("TEST_ACCOUNTING")
    if not closed.issubset(required_ids):
        raise Reject("TEST_ACCOUNTING")
    if set(future).intersection(closed):
        raise Reject("TEST_ACCOUNTING")

    for row in contract.get("future", []):
        identity = row["identity"]
        if row.get("owner") != _expected_owner(identity):
            raise Reject("TEST_ACCOUNTING")
        if row.get("classification") != "failed":
            raise Reject("TEST_ACCOUNTING")

    _validate_migration(contract, required_ids, source_commit)
    _validate_stage_receipts(contract, original, future, closed)

    runs = payload.get("runs")
    if not isinstance(runs, list):
        raise Reject("TEST_ACCOUNTING")

    seen: list[dict[str, Any]] = []
    failures: list[dict[str, Any]] = []
    for run in runs:
        if run.get("exit") != 0:
            raise Reject("TEST_ACCOUNTING")
        if run.get("run_id") != run_id:
            raise Reject("TEST_ACCOUNTING")
        if run.get("source_commit") != source_commit:
            raise Reject("TEST_ACCOUNTING")
        stdout = run.get("stdout")
        if not isinstance(stdout, dict):
            raise Reject("TEST_ACCOUNTING")
        for result in stdout.get("results", []):
            identity = {key: run[key] for key in ("package", "target", "profile", "source_commit")}
            identity["name"] = result["name"]
            seen.append(identity)
            if result["status"] != "passed":
                key = _canonical(identity)
                if key not in future:
                    raise Reject("TEST_ACCOUNTING")
                if result["status"] != future[key]["classification"]:
                    raise Reject("TEST_ACCOUNTING")
                if key in closed:
                    raise Reject("TEST_ACCOUNTING")
                failures.append(identity)

    if sorted(map(_canonical, seen)) != sorted(map(_canonical, contract["required"])):
        raise Reject("TEST_ACCOUNTING")

    actual_outcomes = _outcomes_from_runs(runs)
    expectations = contract.get("postpatch_expectations")
    if expectations is not None:
        if sorted(map(_canonical, actual_outcomes)) != sorted(map(_canonical, expectations)):
            raise Reject("TEST_ACCOUNTING")

    return {"observations": [event], "unresolved": failures}


def _validate_migration(contract: dict[str, Any], required_ids: set[bytes], source_commit: str) -> None:
    if "approved_patch_parent" not in contract:
        return
    if contract["approved_patch_parent"] != source_commit:
        raise Reject("TEST_ACCOUNTING")

    prepatch = contract.get("prepatch")
    if not isinstance(prepatch, dict):
        raise Reject("TEST_ACCOUNTING")
    prepatch_ids: list[dict[str, Any]] = []
    for run in prepatch.get("runs", []):
        stdout = run.get("result", {})
        for row in stdout.get("results", []):
            if row.get("status") != "passed":
                raise Reject("TEST_ACCOUNTING")
            prepatch_ids.append(
                {
                    "package": run["package"],
                    "target": run["target"],
                    "profile": run["profile"],
                    "source_commit": source_commit,
                    "name": row["name"],
                }
            )
    if sorted(map(_canonical, prepatch_ids)) != sorted(map(_canonical, contract["required"])):
        raise Reject("TEST_ACCOUNTING")

    postpatch = contract.get("postpatch")
    if postpatch is None:
        raise Reject("TEST_ACCOUNTING")
    manifest = contract.get("approved_patch_manifest")
    if not isinstance(manifest, dict):
        raise Reject("TEST_ACCOUNTING")
    if postpatch.get("tree") != manifest.get("tree"):
        raise Reject("TEST_ACCOUNTING")
    if postpatch.get("parent") != source_commit:
        raise Reject("TEST_ACCOUNTING")

    expectations = contract.get("postpatch_expectations")
    if expectations is None:
        raise Reject("TEST_ACCOUNTING")
    if sorted(_canonical(row["identity"]) for row in expectations) != sorted(required_ids):
        raise Reject("TEST_ACCOUNTING")

    postpatch_outcomes = _outcomes_from_runs(postpatch.get("runs", []))
    if sorted(map(_canonical, postpatch_outcomes)) != sorted(map(_canonical, expectations)):
        raise Reject("TEST_ACCOUNTING")


def _validate_stage_receipts(
    contract: dict[str, Any],
    original: dict[str, Any],
    future: dict[bytes, dict[str, Any]],
    closed: set[bytes],
) -> None:
    for receipt in contract.get("accepted_stage_receipts", []):
        if receipt.get("test_source_sha256") != original["source_sha256"]:
            raise Reject("TEST_ACCOUNTING")
        for identity in receipt.get("closed", []):
            key = _canonical(identity)
            if key not in closed or key in future:
                raise Reject("TEST_ACCOUNTING")
