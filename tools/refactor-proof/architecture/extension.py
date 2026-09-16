"""Qualification-contract architecture extension validation."""

from __future__ import annotations

from typing import Any

from ..runner.context import Reject


def _median(values: list[float]) -> float:
    ordered = sorted(values)
    mid = len(ordered) // 2
    if len(ordered) % 2:
        return ordered[mid]
    return (ordered[mid - 1] + ordered[mid]) / 2


def validate_performance_event(event: dict[str, Any], contract: dict[str, Any]) -> None:
    payload = event.get("payload")
    if not isinstance(payload, dict):
        raise Reject("PERFORMANCE")
    if contract.get("profile") != "release" or contract.get("serial") is not True or contract.get("strict") is not True:
        raise Reject("PERFORMANCE")
    if contract.get("denominator") != "frame_ns" or contract.get("counter_owner") != "global_allocator":
        raise Reject("PERFORMANCE")
    measurements = payload.get("measurements")
    if not isinstance(measurements, list) or not measurements:
        raise Reject("PERFORMANCE")
    for row in measurements:
        if not isinstance(row, dict):
            raise Reject("PERFORMANCE")
        for key in row:
            if not isinstance(row[key], int) or row[key] < 0:
                raise Reject("PERFORMANCE")
        for key in ("draws", "allocations", "style_calls", "work"):
            if row.get(key) != contract[key]:
                raise Reject("PERFORMANCE")
        style_ns = row.get("style_ns")
        frame_ns = row.get("frame_ns")
        if not isinstance(style_ns, int) or not isinstance(frame_ns, int) or not (0 < style_ns <= frame_ns):
            raise Reject("PERFORMANCE")
    ratio = _median([row["style_ns"] / row["frame_ns"] for row in measurements])
    if ratio > 0.05:
        raise Reject("PERFORMANCE")
