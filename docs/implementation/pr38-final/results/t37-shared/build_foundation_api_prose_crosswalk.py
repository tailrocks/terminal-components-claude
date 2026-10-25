#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import re
from collections import Counter
from pathlib import Path
from typing import Any

SOURCE_COMMIT = "c817b99e9e43abbbf3519d56b2f5ee86924243d8"
PRODUCT_COMMIT = "1d6ee92adf6ccca38eeee64b1371cba1497e8fcd"
FOUNDATION_COMMIT = "2fee2c277d1213fc5fad5d81eb77479600182b69"
SOURCE_MAP_SHA = "6ead70836c20e3ad1c2dc15f0de44daf35d74c6507e1bbfd086de5fa6e85b619"
DISPOSITION_MAP_SHA = "8e3923ebb4f6d5974a2e632ca32e5406718a5985a8f26a17a75fcbd9a3f4e9df"

REVIEWED_GROUPS: dict[int, tuple[str, str, str]] = {
    166: ("maintained_contract_context", "docs/contracts/authoring.md#public-surface-snapshot", "ERROR and INVALID response meanings"),
    168: ("maintained_contract_context", "docs/contracts/authoring.md#public-surface-snapshot", "the INVALID response flag"),
    253: ("maintained_caller_contract", "docs/contracts/authoring.md#keyboard-list-layout-and-text-rules", "terminal support and change-only cursor-shape output"),
    477: ("maintained_caller_contract", "docs/contracts/authoring.md#shell-interaction-policy", "low-priority focus-scope command bindings"),
    478: ("maintained_caller_contract", "docs/contracts/authoring.md#row-authoring-contracts", "explicit component-author style channels"),
    479: ("maintained_contract_context", "docs/contracts/authoring.md#row-authoring-contracts", "the low-level LayerSlotAuthor adapter"),
    435: ("maintained_contract_context", "docs/contracts/authoring.md#row-authoring-contracts", "Canvas-derived component-author geometry"),
    437: ("maintained_caller_contract", "docs/contracts/authoring.md#row-authoring-contracts", "exact text display width for painted lines"),
    465: ("maintained_caller_contract", "docs/contracts/authoring.md#row-authoring-contracts", "owned, read-only Surface cell reads"),
    466: ("maintained_caller_contract", "docs/contracts/authoring.md#row-authoring-contracts", "clipped Surface publication through the Ui paint path"),
    469: ("maintained_caller_contract", "docs/contracts/authoring.md#row-authoring-contracts", "nonempty clipped window geometry"),
    473: ("maintained_caller_contract", "docs/contracts/authoring.md#row-authoring-contracts", "empty unclipped window geometry"),
    504: ("maintained_caller_contract", "docs/contracts/authoring.md#runtime-registration-and-snapshots", "clock-paste activation and step syntax"),
    505: ("maintained_caller_contract", "docs/contracts/authoring.md#runtime-registration-and-snapshots", "the exact clock-hook environment value"),
    506: ("maintained_caller_contract", "docs/contracts/authoring.md#runtime-registration-and-snapshots", "the positive-decimal clock-paste body"),
    507: ("maintained_caller_contract", "docs/contracts/authoring.md#runtime-registration-and-snapshots", "the clock-hook activation predicate"),
    508: ("maintained_caller_contract", "docs/contracts/authoring.md#runtime-registration-and-snapshots", "clock-paste parsing and accepted decimal forms"),
    525: ("maintained_caller_contract", "docs/contracts/authoring.md#runtime-routing-and-layer-receipts", "immediate eligible-parent routing for wheel targets"),
    566: ("maintained_caller_contract", "docs/contracts/authoring.md#row-authoring-contracts", "applying active-row cursor policy before paint"),
    575: ("maintained_caller_contract", "docs/contracts/authoring.md#row-authoring-contracts", "the sticky final-cell policy for label_end"),
    594: ("maintained_contract_context", "docs/contracts/authoring.md#runtime-routing-and-layer-receipts", "the exact opted-in wheel-target result"),
    595: ("maintained_caller_contract", "docs/contracts/authoring.md#runtime-routing-and-layer-receipts", "the no-change meaning of Ignored"),
    599: ("maintained_caller_contract", "docs/contracts/authoring.md#runtime-routing-and-layer-receipts", "event identity shared by attempts for one physical wheel"),
    604: ("maintained_caller_contract", "docs/contracts/authoring.md#runtime-routing-and-layer-receipts", "retrying the exact rejected routed input"),
    608: ("maintained_caller_contract", "docs/contracts/authoring.md#runtime-registration-and-snapshots", "activation-time multi-click interval checks"),
    587: ("maintained_caller_contract", "docs/contracts/authoring.md#runtime-registration-and-snapshots", "wiping runtime clipboard storage on clear"),
    615: ("maintained_caller_contract", "docs/contracts/authoring.md#keyboard-list-layout-and-text-rules", "automatic scrollbar-column reservation"),
    616: ("maintained_caller_contract", "docs/contracts/authoring.md#keyboard-list-layout-and-text-rules", "stable scrollbar-column reservation"),
    617: ("maintained_caller_contract", "docs/contracts/authoring.md#keyboard-list-layout-and-text-rules", "read-only visible-range label formatting"),
    632: ("maintained_caller_contract", "docs/contracts/authoring.md#shell-interaction-policy", "focus-scope bubbling bindings"),
    645: ("maintained_contract_context", "docs/contracts/authoring.md#keyboard-list-layout-and-text-rules", "composition padding, gap, alignment, and overflow options"),
    647: ("maintained_caller_contract", "docs/contracts/authoring.md#keyboard-list-layout-and-text-rules", "clipped composition drawing for component adapters"),
    648: ("maintained_caller_contract", "docs/contracts/authoring.md#keyboard-list-layout-and-text-rules", "read-only composition measurement matching draw"),
    655: ("maintained_caller_contract", "docs/contracts/authoring.md#runtime-routing-and-layer-receipts", "single exact wheel acknowledgement and invalid fallback"),
    662: ("maintained_contract_context", "docs/contracts/authoring.md#runtime-registration-and-snapshots", "click count for the current press or release intent"),
    663: ("maintained_contract_context", "docs/contracts/authoring.md#runtime-registration-and-snapshots", "click count for the current click intent"),
    676: ("maintained_caller_contract", "docs/contracts/theming.md#paint-channel-ownership", "preserving layer symbol and inherited foreground during underlay replay"),
    684: ("maintained_caller_contract", "docs/contracts/theming.md#paint-channel-ownership", "separating resolved paint from fallback channel ownership"),
    699: ("maintained_caller_contract", "docs/contracts/theming.md#paint-channel-ownership", "space fill that keeps foreground ink after a modal backdrop"),
    700: ("maintained_caller_contract", "docs/contracts/theming.md#paint-channel-ownership", "truecolor-only remapping with per-channel provenance"),
    701: ("maintained_caller_contract", "docs/contracts/theming.md#paint-channel-ownership", "pair recoloring that preserves unchanged channel roles"),
    702: ("maintained_caller_contract", "docs/contracts/theming.md#paint-channel-ownership", "foreground recoloring with background provenance retained"),
    703: ("maintained_caller_contract", "docs/contracts/authoring.md#row-authoring-contracts", "picker overflow fade order, geometry, and protected cursor row"),
    704: ("maintained_caller_contract", "docs/contracts/theming.md#paint-channel-ownership", "layer fill preserving inherited channels and replacing raw background"),
    705: ("maintained_contract_context", "docs/contracts/theming.md#paint-channel-ownership", "the active theme's modal backdrop mapping"),
}

DESCRIPTIVE_EXCEPTIONS: dict[int, str] = {
    510: "This group labels the Backdrop variant as a light-dismiss surface behind a popup or modal. It gives no separate ordering, lifecycle, or caller precondition; the route contract is recorded in the runtime section.",
    739: "This module comment labels the retained shell-consumer example as a public-API example and names its example parts. It documents the example file, not a caller rule.",
}

NORMATIVE = re.compile(
    r"\b(must(?: not)?|should|never|only|unless|before|after|until|exact(?:ly)?|"
    r"same|stable|unique|bounded|revision|identity|lifetime|borrow|callback|pure|"
    r"inert|modal|disabled|no-op|does not|cannot|fails?|failure|mismatch|overflow|"
    r"saturat|wrap|remain|keep|preserve|clamp|consum|admit|admission|eligible|"
    r"unmodified|current|frame-local|at most|at least|read-only|not .*? but)\b",
    re.IGNORECASE,
)


def canonical(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def sha256(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def contains(text: str, *needles: str) -> bool:
    lowered = text.lower()
    return any(needle.lower() in lowered for needle in needles)


def destination(path: str, text: str) -> tuple[str, str] | None:
    if path.endswith("/esc_bridge.rs"):
        return "docs/contracts/safety-invariants.md#production-unsafe-operation-rationale-map", "ESC bridge sequencing and ownership rationale"
    if path.endswith("/rowui.rs") and contains(text, "must not paint", "container_width", "admitted", "admission", "logical content", "logical row", "slot", "reserve", "trailing", "leading", "style", "truncat", "fallback", "patch", "part set", "area()", "row width"):
        return "docs/contracts/authoring.md#row-authoring-contracts", "row geometry, measurement, part scope, content reservation, metadata admission, or label fill"
    if path.endswith("/scroll_region.rs") and contains(text, "revision", "route", "source-row", "window", "wheel", "same"):
        return "docs/contracts/authoring.md#runtime-routing-and-layer-receipts", "ScrollRegion source revision and route identity"
    if path.endswith("/runtime.rs") and contains(text, "wheel-route", "wheel route", "render_snapshot", "prepared frame", "prepared output", "presented frame", "diagnostic", "clock", "clock step", "keyboard-protocol"):
        if contains(text, "wheel-route", "wheel route"):
            return "docs/contracts/authoring.md#runtime-routing-and-layer-receipts", "explicit wheel-route admission, source check, and acknowledgement"
        if contains(text, "render_snapshot", "prepared frame", "prepared output", "presented frame", "diagnostic", "clock", "clock step"):
            return "docs/contracts/authoring.md#runtime-registration-and-snapshots", "runtime snapshot, diagnostic, or clock-hook preconditions"
        return "docs/contracts/authoring.md#prepared-frame-consumer-sequence", "runtime frame preparation and presentation lifecycle"
    if path.endswith("/intent.rs") and contains(text, "token", "exact", "mismatch", "notpending", "claim"):
        return "docs/contracts/authoring.md#runtime-routing-and-layer-receipts", "exact tracked intent identity and claim result"
    if path.endswith("/ui/cx.rs") and contains(text, "layeropening", "receipt", "exact"):
        return "docs/contracts/authoring.md#runtime-routing-and-layer-receipts", "exact layer-opening receipt identity"
    if path.endswith(("/keys.rs", "/selection.rs", "/list.rs", "/typeahead.rs")):
        return "docs/contracts/authoring.md#keyboard-list-layout-and-text-rules", "keyboard delivery, list navigation, selection, or type-ahead rule"
    if path.endswith("/scroll.rs") and "termrock-collections" in path:
        return "docs/contracts/authoring.md#keyboard-list-layout-and-text-rules", "scroll follow and reveal behavior"
    if "/termrock-layout/src/" in path and path.endswith(("/columns.rs", "/composition_plan.rs")):
        return "docs/contracts/authoring.md#keyboard-list-layout-and-text-rules", "pure composition measurement or deterministic column-layout rule"
    if "/termrock-text/src/" in path and path.endswith(("/buffer.rs", "/undo.rs", "/editor.rs")):
        return "docs/contracts/authoring.md#keyboard-list-layout-and-text-rules", "text byte offsets, grapheme boundaries, or edit-history rule"
    if "/termrock-shell/src/" in path:
        return "docs/contracts/authoring.md#shell-interaction-policy", "shell key, navigation, leave, status, or footer ownership rule"
    if "/termrock-theme/src/" in path:
        if contains(text, "precedence", "override", "overlay", "profile", "recipe", "slot", "merge"):
            return "docs/contracts/theming.md#resolution-order", "theme recipe, profile, override, or slot-resolution rule"
        return "docs/contracts/theming.md#theme-shape-and-visual-scenarios", "theme capability, role, focus, selection, or rendering rule"
    if "/termrock-runtime/src/ui/mod.rs" in path:
        if contains(text, "wheel", "scroll", "route", "source revision"):
            return "docs/contracts/authoring.md#runtime-routing-and-layer-receipts", "wheel-target registration and source-checked routing"
        if contains(text, "hover-only", "autoscroll", "text body", "focus", "inert", "cursor", "layer", "opening", "diagnostic", "theme", "reference"):
            return "docs/contracts/authoring.md#runtime-registration-and-snapshots", "runtime input registration, layer, focus, cursor, or frame rule"
    if "/termrock-runtime/src/hit.rs" in path or "/termrock-runtime/src/capture.rs" in path:
        if contains(text, "wheel", "route", "source", "release", "owner", "layer"):
            return "docs/contracts/authoring.md#event-routing-and-committed-layout", "event hit, route, capture, or release ownership rule"
    if "/termrock-runtime/src/layer.rs" in path:
        return "docs/contracts/authoring.md#runtime-routing-and-layer-receipts", "layer opening, receipt, dismissal, or input-pass rule"
    if "/termrock-session/src/" in path or "/termrock-terminal/src/keyboard.rs" in path:
        if contains(text, "keyboard", "kitty", "protocol", "push", "pop"):
            return "docs/contracts/authoring.md#keyboard-list-layout-and-text-rules", "opt-in keyboard-protocol policy and session lifecycle"
        return "docs/architecture.md#current-source-backed-boundaries", "terminal session startup, restoration, or event ownership"
    if path.endswith("/diagnostics.rs") and ("termrock-core/src/" in path or "termrock-runtime/src/" in path):
        return "docs/contracts/authoring.md#runtime-registration-and-snapshots", "bounded diagnostic retention and overflow reporting"
    if "/termrock-core/src/" in path or "/termrock-runtime/src/" in path:
        if contains(text, "callback", "borrow", "lifetime", "pure", "bounded", "frame", "snapshot", "present", "diagnostic", "capture"):
            return "docs/contracts/authoring.md#public-surface-snapshot", "runtime authoring callback, borrow, bounded-work, or frame rule"
    if "/termrock-layout/src/" in path:
        return "docs/contracts/authoring.md#keyboard-list-layout-and-text-rules", "layout geometry or sizing rule"
    if "/termrock-text/src/" in path:
        return "docs/contracts/authoring.md#keyboard-list-layout-and-text-rules", "text editing and cursor-boundary rule"
    return None


def build(source_root: Path) -> dict[str, Any]:
    source_map_path = source_root / "source-api-prose.json"
    disposition_path = source_root / "source-api-prose-disposition.json"
    removal_path = source_root / "source-removal.json"
    deferred_path = source_root / "deferred-testing-surfaces.json"
    source_map_bytes = source_map_path.read_bytes()
    disposition_bytes = disposition_path.read_bytes()
    assert sha256(source_map_bytes) == SOURCE_MAP_SHA
    assert sha256(disposition_bytes) == DISPOSITION_MAP_SHA
    source_map = json.loads(source_map_bytes)
    disposition = json.loads(disposition_bytes)
    removal = json.loads(removal_path.read_text())
    deferred = json.loads(deferred_path.read_text())
    assert source_map["record_count"] == 1122 == len(source_map["records"])
    assert len(disposition["records"]) == 1122

    removed_paths = set(removal.get("removed_source_paths", []))
    removed_paths.update(removal.get("removed_test_target_paths_from_inventory", []))
    scopes: dict[str, list[tuple[int, int]]] = {}
    for record in removal.get("removed_cfg_test_scopes", []):
        scopes.setdefault(record["path"], []).append((record["start_line"], record["end_line"]))
    for group in deferred.get("groups", []):
        for path, lines in group.get("input_scope_lines", {}).items():
            for line in lines:
                scopes.setdefault(path, []).append((line, line))

    rows: list[dict[str, Any]] = []
    counts: Counter[str] = Counter()
    unmatched_normative: list[dict[str, Any]] = []
    for drow in disposition["records"]:
        index = drow["source_map_record"]
        srow = source_map["records"][index]
        assert drow["path"] == srow["path"]
        assert drow["start_line"] == srow["start_line"]
        assert drow["end_line"] == srow["end_line"]
        assert drow["source_sha256"] == srow["source_sha256"]
        text = "\n".join(srow["text"])
        category = drow["category"]
        status: str
        dest: str | None = None
        reason: str
        if index in DESCRIPTIVE_EXCEPTIONS:
            status = "descriptive_api_reference" if index == 510 else "historical_or_internal_description"
            reason = DESCRIPTIVE_EXCEPTIONS[index]
        elif index in REVIEWED_GROUPS:
            status, dest, subject = REVIEWED_GROUPS[index]
            reason = f"This source group states or describes {subject}; its exact maintained destination records that rule."
        elif category == "removed_with_deleted_test_or_fixture_source" or drow["path"] in removed_paths:
            status = "retired_test_only_provenance"
            reason = f"This group came from source path `{drow['path']}` deleted with its T37 test or fixture. It records retired test history and is not an active caller rule."
        elif category == "inside_cfg_test_scope":
            status = "retired_test_only_provenance"
            reason = f"This group was inside a `cfg(test)` scope in `{drow['path']}` at the recorded source lines. T37 retired that scope; retain this group as test-only history."
        elif category == "attached_to_test_only_scope":
            status = "retired_test_only_provenance"
            reason = f"This group described a test-only item in `{drow['path']}`. T37 retired that item; it is not an active caller rule."
        elif any(drow["start_line"] <= end and drow["end_line"] >= start for start, end in scopes.get(drow["path"], [])):
            status = "retired_test_only_provenance"
            reason = f"This group overlaps a T37-retired `cfg(test)` or testing-only scope in `{drow['path']}`. It is retained as test-only provenance."
        else:
            mapped = destination(drow["path"], text)
            if mapped and NORMATIVE.search(text):
                status = "maintained_caller_contract"
                dest = mapped[0]
                reason = f"This group states a caller rule about {mapped[1]}; the exact maintained destination records that rule."
            elif mapped and contains(text, "RenderSnapshot", "render_snapshot", "FrameServices", "diagnostic", "Diagnostics", "clock", "Clock"):
                status = "maintained_contract_context"
                dest = mapped[0]
                reason = f"This group describes the maintained runtime contract for {mapped[1]}; its details are recorded in the named section."
            else:
                clean = " ".join(re.sub(r"^\s*//!?\s?", "", line).strip() for line in srow["text"]).strip()
                excerpt = clean.split(". ", 1)[0].strip()
                if contains(clean, "ported from", "ported exactly", "frozen baseline", "source commit", "reference theme", "owner decision", "historical"):
                    status = "historical_or_internal_description"
                    reason = f"This group records source-port history or internal implementation context: `{excerpt[:180]}`. It does not add a separate current caller obligation; keep the source hash and line range for provenance."
                elif clean:
                    status = "descriptive_api_reference"
                    reason = f"This group describes the named API element: `{excerpt[:180]}`. The recorded group gives an API label, type, field, parameter, return value, or routine operation; it states no separate non-obvious caller precondition, lifecycle or failure rule, identity/revision/selection rule, callback/borrow limit, or bounded-work contract. T01/T35 may rebuild the descriptive reference."
                else:
                    status = "descriptive_api_reference"
                    reason = "This group contains only an empty Rustdoc separator. It adds no caller rule; the exact source location and file hash remain in the source inventory."
                if status == "descriptive_api_reference" and NORMATIVE.search(text):
                    unmatched_normative.append({"source_map_record": index, "path": drow["path"], "start_line": drow["start_line"], "text": text})
        rows.append({
            "id": sha256(canonical(srow)),
            "source_map_record": index,
            "path": drow["path"],
            "source_sha256": drow["source_sha256"],
            "line_range": {"start": drow["start_line"], "end": drow["end_line"]},
            "text_sha256": sha256(canonical(srow["text"])),
            "lexical_category": category,
            "visibility": drow.get("visibility"),
            "disposition": status,
            "destination": dest,
            "reason": reason,
        })
        counts[status] += 1

    return {
        "schema": "t37.foundation-api-prose-crosswalk.v1",
        "task_id": "T37",
        "status": "PENDING_INDEPENDENT_REVIEW",
        "work_input_commit": SOURCE_COMMIT,
        "product_source_commit": PRODUCT_COMMIT,
        "source_inventory": {
            "path": "docs/implementation/pr38-final/results/t37-foundations/source-api-prose.json",
            "commit": FOUNDATION_COMMIT,
            "sha256": sha256(source_map_bytes),
            "record_count": len(rows),
            "source_path_count": disposition["path_count"],
        },
        "lexical_disposition_inventory": {
            "path": "docs/implementation/pr38-final/results/t37-foundations/source-api-prose-disposition.json",
            "sha256": sha256(disposition_bytes),
            "method": "Lexical scope and visibility classification only; not treated as semantic disposition.",
        },
        "decision": {
            "id": "T37-API-PROSE-01",
            "reviewer": "/root/t37_components_review",
            "reviewer_model": "gpt-6.1-sol",
            "reviewer_effort": "high",
            "required": "Preserve non-obvious caller preconditions, lifecycle/failure semantics, identity/revision/selection rules, callback purity/borrow limits, and bounded-work contracts in maintained Markdown. Map covered groups to exact sections. Give every source group a specific maintained destination or a specific internal, descriptive, redundant, or retired test-only reason. Do not defer required correctness contracts to T01/T35.",
        },
        "maintained_sections": sorted({row["destination"] for row in rows if row["destination"]}),
        "disposition_counts": dict(counts),
        "unmatched_normative_candidate_count": len(unmatched_normative),
        "unmatched_normative_candidates": unmatched_normative,
        "records": rows,
        "limits": [
            "This is a semantic disposition map, not behavior verification.",
            "Every group retains a source-map record index, exact path/range/file hash, and exact text hash.",
            "Rows marked descriptive or historical are not claims of caller-contract equivalence. T01/T35 must rebuild descriptive API references; required correctness rules already have maintained destinations.",
        ],
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-root", type=Path, default=Path("docs/implementation/pr38-final/results/t37-foundations"))
    parser.add_argument("--output", type=Path, default=Path("docs/implementation/pr38-final/results/t37-shared/foundation-api-prose-crosswalk.json"))
    args = parser.parse_args()
    result = build(args.source_root)
    args.output.write_text(json.dumps(result, indent=2, ensure_ascii=False) + "\n")
    print(f"wrote {args.output}: {len(result['records'])} groups; {result['disposition_counts']}")
    print(f"unmatched normative candidates: {result['unmatched_normative_candidate_count']}; output sha256: {sha256(args.output.read_bytes())}")


if __name__ == "__main__":
    main()
