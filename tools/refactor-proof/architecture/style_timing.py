"""Style timing measurement architecture checks."""

from __future__ import annotations

import ast
import base64
import hashlib
import re
import subprocess
from typing import Any

from ..runner.context import Reject

CENSUS = [
    "style",
    "style_inherited",
    "style_defaults",
    "paint_patch",
    "style_patched",
    "resolve",
    "surface_style",
    "bg",
    "CellUi::drop::bind",
    "StatusBar::item_style::bind",
    "Grid::apply_style_delta::bind",
]


def _sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _parse(result: subprocess.CompletedProcess[Any]) -> dict[str, Any]:
    if result.returncode != 0:
        raise Reject("ARCHITECTURE")
    frames = [line.split(b"STYLE_FRAME|", 1)[1] for line in result.stdout.splitlines() if b"STYLE_FRAME|" in line]
    if len(frames) != 1:
        raise Reject("ARCHITECTURE")
    rows: list[dict[str, Any]] = []
    times: dict[tuple[int, int], int] = {}
    effective: dict[tuple[int, int], int] = {}
    branches: list[dict[str, Any]] = []
    caches: dict[tuple[int, int], str] = {}
    for line in result.stdout.decode().splitlines():
        if "STYLE_EFFECTIVE_TIME|" in line:
            batch, mode, denom = map(int, line.split("STYLE_EFFECTIVE_TIME|", 1)[1].split("|"))
            effective[(batch, mode)] = denom
        if "STYLE_CACHE|" in line:
            batch, mode, value = line.split("STYLE_CACHE|", 1)[1].split("|", 2)
            caches[(int(batch), int(mode))] = value
        if "STYLE_BRANCH|" in line:
            mode, witness, intervals = line.split("STYLE_BRANCH|", 1)[1].split("|", 2)
            branches.append({"mode": int(mode), "witness": ast.literal_eval(witness), "intervals": ast.literal_eval(intervals)})
        if "STYLE_FRAME_TIME|" in line:
            batch, mode, value = map(int, line.split("STYLE_FRAME_TIME|", 1)[1].split("|"))
            times[(batch, mode)] = value
        if "STYLE_ROW|" not in line:
            continue
        fields = line.split("STYLE_ROW|", 1)[1].split("|", 12)
        if len(fields) != 13:
            raise Reject("ARCHITECTURE")
        nums = list(map(int, fields[:11]))
        witness = ast.literal_eval(fields[11])
        intervals = ast.literal_eval(fields[12])
        rows.append(dict(zip(["batch", "mode", "frame_ns", "allocations", "bytes", "cache_hits", "cache_misses", "reported_ns", "reported_mode", "cal_source", "policy"], nums), witness=witness, intervals=intervals))
    if [(row["batch"], row["mode"]) for row in rows] != [(b, m) for b in range(9) for m in range(3)]:
        raise Reject("ARCHITECTURE")
    if len(times) != 27 or len(effective) != 27 or len(caches) != 27:
        raise Reject("ARCHITECTURE")
    for row in rows:
        row["boundary_ns"] = times[(row["batch"], row["mode"])]
        row["effective_ns"] = effective[(row["batch"], row["mode"])]
        row["cache_contents"] = caches[(row["batch"], row["mode"])]
    return {"frame": frames[0].decode(), "rows": rows, "branches": branches, "stdout_sha256": _sha(result.stdout), "stderr_sha256": _sha(result.stderr)}


def _judge(
    observed: dict[str, Any],
    baseline: dict[str, Any],
    *,
    injected: bool,
    strict: bool,
    arithmetic: bool,
    expected_witness: list[int] | None,
    attest: bool,
) -> dict[str, Any]:
    if strict and injected and not arithmetic:
        raise Reject("ARCHITECTURE")
    if observed["frame"] != baseline["frame"]:
        raise Reject("ARCHITECTURE")
    rows = observed["rows"]
    baseline_rows = baseline["rows"]
    reference = rows[0]["witness"]
    if not reference or set(reference) - set(range(len(CENSUS))):
        raise Reject("ARCHITECTURE")
    if 8 not in reference:
        raise Reject("ARCHITECTURE")
    if expected_witness is not None and reference != expected_witness:
        raise Reject("ARCHITECTURE")
    branches = observed["branches"]
    if [branch["mode"] for branch in branches] != [0, 1, 2]:
        raise Reject("ARCHITECTURE")
    for branch in branches:
        if set(branch["witness"]) != set(range(len(CENSUS))):
            raise Reject("ARCHITECTURE")
        if branch["witness"] != branches[0]["witness"]:
            raise Reject("ARCHITECTURE")
        if branch["mode"] != 0 and ([item[0] for item in branch["intervals"]] != branch["witness"] or any(item[3] != 0 for item in branch["intervals"])):
            raise Reject("ARCHITECTURE")
    corrected: list[int] = []
    denominators: list[int] = []
    for index, row in enumerate(rows):
        if row["policy"] != 1:
            raise Reject("ARCHITECTURE")
        if row["reported_mode"] != row["mode"]:
            raise Reject("ARCHITECTURE")
        if row["cal_source"] != 1:
            raise Reject("ARCHITECTURE")
        if row["witness"] != reference:
            raise Reject("ARCHITECTURE")
        for key in ["allocations", "bytes", "cache_hits", "cache_misses", "cache_contents"]:
            if row[key] != baseline_rows[index][key]:
                raise Reject("ARCHITECTURE")
        intervals = row["intervals"]
        if not all(isinstance(item, tuple) and len(item) == 4 and 0 <= item[1] <= item[2] and item[3] == 0 for item in intervals):
            raise Reject("ARCHITECTURE")
        if row["mode"] == 0:
            if intervals or row["reported_ns"] != 0:
                raise Reject("ARCHITECTURE")
        else:
            if [item[0] for item in intervals] != reference:
                raise Reject("ARCHITECTURE")
            if sum(item[2] - item[1] for item in intervals) != row["reported_ns"]:
                raise Reject("ARCHITECTURE")
        if row["frame_ns"] <= 0:
            raise Reject("ARCHITECTURE")
        if row["frame_ns"] != row["effective_ns"] or (not injected and row["frame_ns"] != row["boundary_ns"]):
            raise Reject("ARCHITECTURE")
        if row["mode"] == 2:
            value = row["reported_ns"] - rows[index - 1]["reported_ns"]
            if not attest and value < 0:
                raise Reject("ARCHITECTURE")
            corrected.append(value)
            denominators.append(row["frame_ns"] if injected else baseline_rows[index]["frame_ns"])
    low = min(corrected) / max(denominators)
    high = max(corrected) / min(denominators)
    if injected and (low <= 0.05 < high):
        raise Reject("ARCHITECTURE")
    quality = "INVALID" if min(corrected) < 0 else "REJECT" if high > 0.05 else "PASS"
    if strict and not attest and high > 0.05:
        raise Reject("ARCHITECTURE")
    if attest and quality != "PASS":
        raise Reject("ARCHITECTURE")
    return {"quality": quality}


def _case_options(name: str) -> dict[str, bool]:
    attest = name.startswith("attest-")
    return {
        "injected": not attest,
        "strict": attest or name.startswith("strict-"),
        "arithmetic": not attest and name != "strict-injected-forgery",
        "attest": attest,
    }


def validate_style_timing_event(event: dict[str, Any], profile: dict[str, Any]) -> None:
    payload = event.get("payload") or {}
    if payload.get("schema") != "tc-style-timing-actual-observation/v1":
        raise Reject("ARCHITECTURE")
    control = payload.get("control") or {}
    subject = payload.get("subject") or {}
    if subject.get("exit") not in (0, 101):
        raise Reject("ARCHITECTURE")
    control_result = subprocess.CompletedProcess([], control.get("exit", 1), base64.b64decode(control.get("stdout", "")), base64.b64decode(control.get("stderr", "")))
    subject_result = subprocess.CompletedProcess([], subject.get("exit", 1), base64.b64decode(subject.get("stdout", "")), base64.b64decode(subject.get("stderr", "")))
    if subject.get("exit") == 101:
        raise Reject("ARCHITECTURE")
    acceptance = profile.get("acceptance", "protocol-arithmetic")
    attest = acceptance == "attestation"
    options = {
        "injected": profile.get("clock_kind") == "injected-arithmetic-only",
        "strict": acceptance in {"strict", "strict-arithmetic", "attestation"},
        "arithmetic": acceptance in {"strict-arithmetic", "protocol-arithmetic"},
        "attest": attest,
    }
    observed = _parse(subject_result)
    baseline = _parse(control_result)
    _judge(observed, baseline, expected_witness=profile.get("expected_invocations"), **options)
