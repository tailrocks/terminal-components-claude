"""Actual terminal-components architecture checks."""

from __future__ import annotations

import base64
import json
import re
from typing import Any

from ..runner.context import Reject


def _decode_stream(payload: dict[str, Any], stream: str) -> bytes:
    return base64.b64decode(payload.get(stream, ""))


def _semantic_output(data: bytes) -> bytes:
    data = re.sub(rb"(?m)^(test result: [^\n]*; )finished in [0-9.]+s$", rb"\1finished in <duration>s", data)
    return re.sub(rb"(?m)^(thread '[^'\n]+' )\([0-9]+\)( panicked at )", rb"\1(<thread-id>)\2", data)


def _parse_capture(stdout: bytes) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for line in stdout.decode().splitlines():
        if "ARCHFRAME|" in line:
            line = line[line.index("ARCHFRAME|") :]
        fields = line.split("|")
        if fields[0] == "ARCHFRAME":
            result[fields[1]] = {"width": int(fields[2]), "height": int(fields[3]), "focus": fields[4], "area": fields[5], "cells": []}
        elif fields[0] == "ARCHCELL":
            result[fields[1]]["cells"].append(fields[2:])
        elif fields[0] == "ARCHTEXT":
            result[fields[1]]["text"] = bytes.fromhex(fields[2]).decode()
    if set(result) != {"initial", "activated"}:
        raise Reject("ARCHITECTURE")
    for frame in result.values():
        if (frame["width"], frame["height"]) != (120, 40) or len(frame["cells"]) != 4800:
            raise Reject("ARCHITECTURE")
    return result


def _read_actual_sources(profile: dict[str, Any]) -> dict[str, str]:
    directory = profile.get("source_directory")
    if not directory:
        raise Reject("ARCHITECTURE")
    sources: dict[str, str] = {}
    for path, digest in profile.get("sources", {}).items():
        import hashlib
        from pathlib import Path

        data = (Path(directory) / path).read_bytes()
        if hashlib.sha256(data).hexdigest() != digest:
            raise Reject("ARCHITECTURE")
        if path.endswith((".rs", ".toml", ".md", ".txt")):
            sources[path] = data.decode()
    return sources


def _validate_application_sources(sources: dict[str, str], *, subject: str | None = None) -> None:
    if subject == "standalone-RainApp-consumer":
        return
    grid = sources.get("apps/showcase/src/pages/grid.rs", "")
    app = sources.get("apps/showcase/src/app.rs", "")
    if "paint_body(" in grid:
        raise Reject("ARCHITECTURE")
    if "metrics().draw(ui, body, &self.state, &MetricModel)" in grid and "if false {" not in grid:
        if "name: \"Q7W9 metrics\"" not in grid:
            raise Reject("ARCHITECTURE")
    if "if false { " in grid and "metrics().draw" in grid:
        raise Reject("ARCHITECTURE")
    if "ui.reference(None, |ui|" in grid and "metrics().draw" in grid:
        raise Reject("ARCHITECTURE")
    if "shell_brand().draw(ui, shell.header);" not in app and "brand_removed" not in app:
        if "Brand::new(BRAND," in app and "paint_header(" in app:
            pass
    if "paint_header(" in app and "paint_footer(" in app:
        start = app.index("fn draw(&self, ui:")
        body = app[start:]
        if "paint_header(" in body and "paint_body(" not in grid:
            raise Reject("ARCHITECTURE")
    if "Brand::new(BRAND, \"Q7W9!\")" in app:
        raise Reject("ARCHITECTURE")


def _validate_conformance_stdout(stdout: bytes, profile: dict[str, Any], *, expect_failure: str | None = None) -> None:
    if expect_failure:
        diagnostic = "\n".join(line for line in stdout.decode().splitlines() if not line.startswith("ARCHSTYLE|"))
        expected = {
            "ignored_icon_slot": "documented slot ignored actual paint",
            "measure_query": "measurement must not create a paint query",
            "slot_doc_drift": "rustdoc slot set differs from actually paintable set",
            "state_omission": "capability-implied state was omitted",
            "fake_row_owner": "observation cannot invent a fake owner ID",
        }
        message = expected.get(expect_failure)
        if not message or message not in diagnostic:
            raise Reject("ARCHITECTURE")
        return
    rows = [line[line.index("ARCHCASE|") :] for line in stdout.decode().splitlines() if "ARCHCASE|" in line]
    slots = [line for line in stdout.decode().splitlines() if "ARCHSLOT|" in line]
    styles = [line for line in stdout.decode().splitlines() if "ARCHSTYLE|" in line]
    if profile.get("subject") == "whole-pinned-registry" and len(rows) != 44:
        raise Reject("ARCHITECTURE")
    if profile.get("subject") == "standalone-public-Button-consumer":
        if len(rows) != 1 or not rows[0].startswith("ARCHCASE|button|"):
            raise Reject("ARCHITECTURE")
    if len(slots) != 12:
        raise Reject("ARCHITECTURE")
    row_parts = [line.split("ARCHROWPARTS|", 1)[1].split("|") for line in stdout.decode().splitlines() if "ARCHROWPARTS|" in line]
    if len(row_parts) != 1:
        raise Reject("ARCHITECTURE")
    if not any("tests/conformance.rs|" in line and line.split("|")[6] == row_parts[0][0] for line in styles):
        raise Reject("ARCHITECTURE")
    cases: dict[str, dict[str, Any]] = {}
    active = None
    for line in stdout.decode().splitlines():
        if "ARCHCASE|" in line:
            _, active, owner, parts = line[line.index("ARCHCASE|") :].split("|", 3)
            if active in cases:
                raise Reject("ARCHITECTURE")
            cases[active] = {"owner": owner, "declared": parts.strip("[]").split(", "), "owned": set(), "row": set(), "states": set()}
        elif active and "ARCHBEGIN|" in line:
            state = line[line.index("ARCHBEGIN|") :]
            if state in cases[active]["states"]:
                raise Reject("ARCHITECTURE")
            cases[active]["states"].add(state)
        elif active and "ARCHSTYLE|" in line:
            fields = line[line.index("ARCHSTYLE|") :].split("|", 7)
            bucket = "row" if fields[1].endswith("tests/conformance.rs") else "owned"
            if fields[3] == cases[active]["owner"]:
                cases[active][bucket].add(fields[6])
    for case in cases.values():
        case["missing"] = sorted(set(case["declared"]) - case["owned"])
        case["extra"] = sorted(case["owned"] - set(case["declared"]))
    vectors = profile.get("required_state_vectors") or {}
    states_map = vectors.get("states") or {}
    geometries = vectors.get("geometries") or [[40, 12], [8, 4], [0, 0]]
    for name, case in cases.items():
        expected_states = states_map.get(name)
        if expected_states is None:
            continue
        expected = {f"ARCHBEGIN|{name}|{state}|{width}|{height}" for state in expected_states for width, height in geometries}
        if set(case["states"]) != expected:
            raise Reject("ARCHITECTURE")
        if case["missing"] or case["extra"]:
            raise Reject("ARCHITECTURE")
    button = sources_from_profile(profile)
    if "filter(|_| false)" in button.get("crates/tui/src/components/button.rs", ""):
        raise Reject("ARCHITECTURE")
    if "Part::custom(\"architecture.extra-owned\")" in button.get("crates/tui/src/components/button.rs", ""):
        raise Reject("ARCHITECTURE")
    if "Part::custom(\"architecture.unreachable\")" in button.get("crates/tui/src/components/button.rs", ""):
        raise Reject("ARCHITECTURE")
    if "Id::root(\"architecture.fake-owner\")" in button.get("crates/tui/src/collection/rowui.rs", ""):
        raise Reject("ARCHITECTURE")
    if "const STATES: [StateFlags; 6]" in button.get("crates/tui/tests/conformance.rs", ""):
        raise Reject("ARCHITECTURE")
    if profile.get("subject") == "standalone-public-Button-consumer" and "conformance_suite!(" not in button.get("crates/tui/tests/conformance.rs", ""):
        if "button => ButtonCase" not in button.get("crates/tui/tests/conformance.rs", ""):
            raise Reject("ARCHITECTURE")
    if profile.get("subject") == "whole-pinned-registry" and cases and any(case["missing"] or case["extra"] for case in cases.values()):
        raise Reject("ARCHITECTURE")


def sources_from_profile(profile: dict[str, Any]) -> dict[str, str]:
    try:
        return _read_actual_sources(profile)
    except Reject:
        return {}


def validate_actual_event(event: dict[str, Any], profile: dict[str, Any]) -> None:
    kind = profile.get("kind")
    payload = event.get("payload") or {}
    if payload.get("schema") == "tc-architecture-actual-rust-observation/v1":
        stdout = _decode_stream(payload, "stdout")
        stderr = _decode_stream(payload, "stderr")
    else:
        stdout = b""
        stderr = b""
    sources = _read_actual_sources(profile)
    if kind == "application":
        _validate_application_sources(sources, subject=profile.get("subject"))
        if event.get("exit") != 0:
            raise Reject("ARCHITECTURE")
        if profile.get("subject") == "standalone-RainApp-consumer":
            if b"ARCHRAIN|" not in stdout:
                raise Reject("ARCHITECTURE")
            return
        frames = _parse_capture(stdout)
        base = frames["initial"]
        if "customers" not in base.get("text", "") or "P95 latency" in base.get("text", ""):
            raise Reject("ARCHITECTURE")
        if "selected metric:" not in frames["activated"].get("text", ""):
            raise Reject("ARCHITECTURE")
        return
    if kind == "external-author":
        if event.get("exit") != 0 or b"test result: ok" not in stdout:
            raise Reject("ARCHITECTURE")
        return
    if kind == "ownership-fixture":
        if "nested return leaked the inner owner" in stderr.decode():
            raise Reject("ARCHITECTURE")
        if event.get("exit") != 0 or b"ARCHNESTED|" not in stdout:
            raise Reject("ARCHITECTURE")
        if "outer.owner = INNER" in sources.get("crates/tui/src/collection/rowui.rs", ""):
            raise Reject("ARCHITECTURE")
        return
    if kind == "conformance":
        subject = profile.get("subject")
        sources_text = "\n".join(sources.values())
        expect_failure = None
        if ".filter(|_| false)" in sources.get("crates/tui/src/components/button.rs", ""):
            expect_failure = "ignored_icon_slot"
        elif "note_styled(OWNER" in sources.get("crates/tui/tests/conformance.rs", ""):
            expect_failure = "measure_query"
        elif "/// `MARKER` and `LABEL`" in sources.get("crates/tui/src/components/button.rs", "") and "ICON" not in sources.get("crates/tui/src/components/button.rs", ""):
            expect_failure = "slot_doc_drift"
        elif "const STATES: [StateFlags; 6]" in sources.get("crates/tui/tests/conformance.rs", ""):
            expect_failure = "state_omission"
        elif "architecture.fake-owner" in sources.get("crates/tui/src/collection/rowui.rs", ""):
            expect_failure = "fake_row_owner"
        if expect_failure:
            if event.get("exit") == 0:
                raise Reject("ARCHITECTURE")
            _validate_conformance_stdout(stdout, profile, expect_failure=expect_failure)
            return
        if "conformance_suite!(" not in sources.get("crates/tui/tests/conformance.rs", "") and subject != "whole-pinned-registry":
            if event.get("exit") == 0 and not cases_empty(stdout):
                raise Reject("ARCHITECTURE")
            raise Reject("ARCHITECTURE")
        if event.get("exit") != 0:
            raise Reject("ARCHITECTURE")
        _validate_conformance_stdout(stdout, profile)
        return
    raise Reject("ARCHITECTURE")


def cases_empty(stdout: bytes) -> bool:
    return not any("ARCHCASE|" in line for line in stdout.decode().splitlines())
