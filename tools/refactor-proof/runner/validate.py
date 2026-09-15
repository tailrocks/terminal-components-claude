"""Observation and extension payload validation."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .context import Reject, expanded_members
from .json_util import load_path, sha256_canonical


def seeded_value(config: dict[str, Any]) -> int:
    return config["seed"] + len(config.get("seed_steps", []))


def validate_capture_payload(
    event: dict[str, Any],
    *,
    lane: str,
    expected_members: list[str],
    config: dict[str, Any],
    operation: str,
) -> None:
    if event.get("exit", 0) != 0 or not isinstance(event.get("payload"), dict):
        raise Reject("EXECUTION")
    body = event["payload"]
    draw_count = 2 if operation == "architecture" else 10
    calls = body.get("calls", [])
    if "App.update" not in calls:
        raise Reject("EXECUTION")
    if calls.count("Widget.draw") != draw_count:
        raise Reject("EXECUTION")
    if "Props.enabled" not in calls:
        raise Reject("EXECUTION")
    seeded = seeded_value(config)
    expected_value = seeded + (0 if operation == "architecture" else 1)
    if "value" not in body or body["value"] != expected_value:
        raise Reject("EXECUTION")
    expected_pty = operation == "capture" and lane == "pty"
    if body.get("pty") != expected_pty:
        raise Reject("EXECUTION")
    if operation in {"capture", "direct", "pty"}:
        selected_lane = lane if operation == "capture" else "direct"
        members = [name for name in expected_members if name.split("/")[1] == selected_lane]
        captures = body.get("captures")
        if not isinstance(captures, list) or [row["id"] for row in captures] != members:
            raise Reject("EXECUTION")
        for row in captures:
            if set(row) != {"id", "before", "after"}:
                raise Reject("EXECUTION")
            _, _, width, palette = row["id"].split("/")
            for checkpoint, increment in (("before", 0), ("after", 1)):
                expected = {
                    "width": int(width),
                    "custom_art": "*",
                    "control": {
                        "text": str(seeded + increment),
                        "foreground": palette,
                        "owner": "Widget",
                    },
                }
                if row[checkpoint] != expected:
                    raise Reject("EXECUTION")


def validate_oracle_repeat(events: list[dict[str, Any]]) -> None:
    if len(events) != 2:
        raise Reject("EXECUTION")
    first = events[0].get("payload")
    second = events[1].get("payload")
    if events[0].get("exit") != 0 or events[1].get("exit") != 0:
        raise Reject("EXECUTION")
    if first != second:
        raise Reject("REPEAT")


def validate_native_extension(payload: dict[str, Any], contract: dict[str, Any], source_commit: str) -> None:
    if payload.get("source") != contract["source_sha256"]:
        raise Reject("SOURCE")
    if contract.get("mapping_owner") != source_commit:
        raise Reject("SOURCE")
    if contract.get("mapping") != {"footer_row": "height - 2", "pointer": [2, "height - 2"]}:
        raise Reject("SOURCE")
    runs = payload.get("runs")
    if not runs or runs[0].get("exit") != 0:
        raise Reject("SOURCE")
    observed = runs[0]["stdout"]
    native = observed["native"]
    if [(row["width"], row["height"], row["row"]) for row in native] != [(120, 40, 38), (100, 30, 28)]:
        raise Reject("SOURCE")
    resized = observed["resized"]
    if [[row["width"], row["height"]] for row in resized] != contract["resized_sizes"]:
        raise Reject("SOURCE")
    for row in native + resized:
        if row["row"] != row["height"] - 2:
            raise Reject("SOURCE")
        if row["pointer"] != [2, row["height"] - 2]:
            raise Reject("SOURCE")
        if len(row["rows"]) != row["height"]:
            raise Reject("SOURCE")
        if any(len(line) != row["width"] for line in row["rows"]):
            raise Reject("SOURCE")
        if row.get("expected", "attached") not in row["rows"][row["row"]]:
            raise Reject("SOURCE")


def validate_close_records(event: dict[str, Any]) -> None:
    records = event.get("records")
    if not isinstance(records, list):
        raise Reject("CLOSURE")
    if [row["check_id"] for row in records] != ["CHK-001", "CHK-002", "CHK-003"]:
        raise Reject("CLOSURE")
    if any(row.get("status") != "passed" for row in records):
        raise Reject("CLOSURE")


def validate_index_close_outputs(event: dict[str, Any], index: dict[str, Any], output_dir: Path) -> None:
    records = event.get("records", [])
    members = {member["check_id"]: member for member in index["members"]}
    for row in records:
        member = members.get(row["check_id"])
        if member is None:
            raise Reject("CLOSURE")
        output = output_dir / member["output_id"]
        if not output.is_file():
            raise Reject("CLOSURE")
        report = load_path(output)
        if sha256_canonical(report) != row.get("report_sha256"):
            raise Reject("CLOSURE")
        if row.get("context_sha256") != member["context_sha256"]:
            raise Reject("CLOSURE")
        if row.get("output_id") != member["output_id"]:
            raise Reject("CLOSURE")
