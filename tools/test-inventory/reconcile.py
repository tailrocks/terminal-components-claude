#!/usr/bin/env python3
"""Map historical obligations onto source-discovered identities. Never approval."""
from __future__ import annotations

import json
from pathlib import Path
import re

from inventory import Invalid, digest, encoded, require, unique


CANONICAL_ROWS = 620

# Historical target strings predate the current workspace layout. Lib names
# (holla_app, jackin_app, showcase_app, tablepro_app) and bin paths map to
# current packages; junie-tui examples map via their file stems. Targets like
# "Running tests/perf.rs [perf-<hash>]" carry no package hint and return None:
# mapping then falls back to exact identity match, else ambiguous/unresolved.
TARGET_PACKAGE = (
    (re.compile(r"crates/tui-testing"), "junie-tui-testing"),
    (re.compile(r"crates/tui"), "junie-tui"),
    (re.compile(r"jackin_app|jackin_preview|jackin-preview"), "jackin-preview"),
    (re.compile(r"src/bin/holla|\[holla-|\[holla_app-"), "holla"),
    (re.compile(r"src/bin/tablepro|\[tablepro-|\[tablepro_app-"), "tablepro"),
    (re.compile(r"src/bin/showcase|\[showcase-|\[showcase_app-"), "showcase"),
    (re.compile(r"02_custom_theme|03_partial_theme|04_family_recipe|12_author_component"), "junie-tui"),
    (re.compile(r"junie_tui_testing"), "junie-tui-testing"),
    (re.compile(r"junie_tui"), "junie-tui"),
    (re.compile(r"\[xtask-|unittests src/main.rs \[xtask"), "xtask"),
)


def infer_package(target):
    for pattern, package in TARGET_PACKAGE:
        if pattern.search(target):
            return package
    return None


def load_canonical(path):
    lines = Path(path).read_text().splitlines()
    require(lines, "canonical historical union is empty")
    header = lines[0].split("\t")
    require("id" in header, "canonical union missing id column")
    rows = []
    for line in lines[1:]:
        if not line.strip():
            continue
        cells = line.split("\t")
        require(len(cells) == len(header), "canonical union column mismatch")
        rows.append(dict(zip(header, cells)))
    unique([row["id"] for row in rows], "canonical union id")
    require(len(rows) == CANONICAL_ROWS, "canonical historical union must have 620 rows")
    return rows


def index_identities(identities):
    by_key = {}
    by_identity = {}
    for row in identities:
        key = (row["package"], row["kind"], row["target"], row["identity"])
        by_key[key] = row
        by_identity.setdefault(row["identity"], []).append(row)
    return by_key, by_identity


def match_obligation(obligation, by_identity):
    identity = obligation["identity"]
    hits = by_identity.get(identity, [])
    package = infer_package(obligation["target"])
    if package:
        packaged = [row for row in hits if row["package"] == package]
        if len(packaged) == 1:
            return "exact-current", packaged[0], None
        if len(packaged) > 1:
            return "ambiguous", None, packaged
    if len(hits) == 1:
        return "exact-current", hits[0], None
    if len(hits) > 1:
        return "ambiguous", None, hits
    basename = identity.rsplit("::", 1)[-1]
    candidates = []
    for rows in by_identity.values():
        for row in rows:
            if row["identity"].rsplit("::", 1)[-1] == basename:
                if package is None or row["package"] == package:
                    candidates.append(row)
    if candidates:
        return "unapproved-relocation", None, candidates
    return "unresolved", None, []


def destination_tuple(row):
    profile = row["package"] + "-all"
    return [profile, row["package"], row["kind"], row["target"], row["identity"]]


def compact_identity(row):
    keys = ("package", "kind", "target", "identity", "path", "line", "origin",
            "classification", "ignored", "blob_sha256", "macro")
    return {key: row.get(key) for key in keys}


def reconcile(historical, discovery, canonical_rows, required):
    require(required.get("approval") == "pending", "required.json must stay pending until complete matrix")
    require(historical["schema"] == 1, "unknown historical schema")
    require(len(historical["obligations"]) == len(required["obligations"]),
            "historical obligation omitted or invented")
    require(required["catalog_sha256"] == digest(encoded(historical)),
            "historical catalog fingerprint differs")
    unique([row["source_id"] for row in historical["obligations"]], "historical obligation")
    require(len(canonical_rows) == CANONICAL_ROWS, "canonical union incomplete")
    _, by_identity = index_identities(discovery["identities"])
    proposals = []
    matched = 0
    unresolved = 0
    for obligation in historical["obligations"]:
        status, row, candidates = match_obligation(obligation, by_identity)
        destinations = [destination_tuple(row)] if row is not None else []
        if destinations:
            matched += 1
        else:
            unresolved += 1
        proposals.append({
            "source_id": obligation["source_id"],
            "origin": obligation["origin"],
            "historical_target": obligation["target"],
            "historical_identity": obligation["identity"],
            "review": None,
            "status": status,
            "destinations": destinations,
            "candidates": [
                [c["package"], c["kind"], c["target"], c["identity"]] for c in (candidates or [])[:8]
            ],
        })
    conflicts = [compact_identity(row) | {
        "assertions": row.get("assertions", []),
        "byte_start": row.get("byte_start"),
        "byte_end": row.get("byte_end"),
        "end_line": row.get("end_line"),
    } for row in discovery["identities"] if row.get("classification") == "oracle-conflict"]
    doctest_blockers = [b for b in discovery.get("blockers", []) if "doctest" in b.get("reason", "")]
    complete = False
    return {
        "schema": 1,
        "classification": "proposed-not-approved",
        "approval": "pending",
        "complete_matrix": complete,
        "catalog_sha256": required["catalog_sha256"],
        "canonical_rows": len(canonical_rows),
        "canonical_ids": [row["id"] for row in canonical_rows],
        "historical_obligations": len(historical["obligations"]),
        "discovered_identities": len(discovery["identities"]),
        "matched": matched,
        "unresolved": unresolved,
        "matrix_blockers": (
            ["cargo-nextest has no doctest runner"] if doctest_blockers else []
        ) + ["listing/execution capture not bound; required.json targets empty"],
        "obligations": proposals,
        "conflicts": conflicts,
    }


def refuse_approval(result):
    require(result.get("approval") == "pending", "reconcile cannot approve required identities")
    require(result.get("complete_matrix") is False, "complete matrix requires executed capture")


def write_reconcile(result, output):
    refuse_approval(result)
    require(not output.exists(), "refuse existing output")
    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open("x") as stream:
        json.dump(result, stream, indent=2)
        stream.write("\n")
