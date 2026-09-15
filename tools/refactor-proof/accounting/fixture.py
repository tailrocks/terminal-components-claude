"""Synthetic fixture accounting validation."""

from __future__ import annotations

from typing import Any

from ..runner.context import Reject
from .identity import original_name, resolve_required_names


def validate_inventory_integrity(inventory: dict[str, Any]) -> None:
    closed = set(inventory.get("closed", []))
    future = inventory.get("future", {})
    for name in closed:
        if name in future:
            raise Reject("INTEGRITY")


def validate_event(event: dict[str, Any], inventory: dict[str, Any], config: dict[str, Any]) -> None:
    if event.get("exit", 0) != 0:
        raise Reject("TEST_ACCOUNTING")
    payload = event.get("payload")
    if not isinstance(payload, dict):
        raise Reject("TEST_ACCOUNTING")

    required_ids = resolve_required_names(inventory)
    discovered = payload.get("discovered")
    if not isinstance(discovered, list) or sorted(discovered) != required_ids:
        raise Reject("TEST_ACCOUNTING")

    results = payload.get("results")
    if not isinstance(results, list):
        raise Reject("TEST_ACCOUNTING")
    if sorted(row["id"] for row in results) != required_ids:
        raise Reject("TEST_ACCOUNTING")

    for key in ("profile", "package", "target"):
        if payload.get(key) != inventory.get(key):
            raise Reject("TEST_ACCOUNTING")

    relocations = inventory.get("relocations", {})
    closed = set(inventory.get("closed", []))
    future = inventory.get("future", {})

    for row in results:
        name = original_name(row["id"], relocations)
        if name in closed and row["status"] != "passed":
            raise Reject("TEST_ACCOUNTING")
        if row["status"] != "passed" and name not in future:
            raise Reject("TEST_ACCOUNTING")

    failures = [row for row in results if row["status"] != "passed"]
    if failures != [{"id": "test_future", "status": "failed"}]:
        raise Reject("TEST_ACCOUNTING")

    scenarios = payload.get("scenarios")
    if not isinstance(scenarios, list) or not scenarios:
        raise Reject("TEST_ACCOUNTING")
    scenario = scenarios[0]

    for contrib in inventory.get("contributions", []):
        state_key = contrib.get("state_key")
        if not state_key:
            continue
        actual = scenario.get("state", {}).get(state_key)
        expected = contrib.get("expected")
        if contrib.get("previously_closed") and actual != expected:
            raise Reject("TEST_ACCOUNTING")

    if inventory.get("claimed_closed_scenarios"):
        raise Reject("TEST_ACCOUNTING")

    expectations = inventory.get("scenario_expectations", {})
    frame = scenario.get("frame", {})
    for expected_frame in expectations.values():
        for key, value in expected_frame.items():
            if frame.get(key) == value:
                raise Reject("TEST_ACCOUNTING")

    _ = config


def build_outputs(events: list[dict[str, Any]], inventory: dict[str, Any]) -> dict[str, Any]:
    future = inventory.get("future", {})
    contributions = inventory.get("contributions", [])
    future_scenarios = inventory.get("future_scenarios", {})

    closed_contributions = [
        contrib["id"]
        for contrib in contributions
        if contrib.get("previously_closed") and contrib.get("required_for_closure")
    ]

    return {
        "observations": events,
        "unresolved_tests": sorted(future.keys()),
        "closed_contributions": closed_contributions,
        "unresolved_scenarios": sorted(future_scenarios.keys()),
    }
