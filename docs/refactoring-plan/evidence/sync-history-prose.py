#!/usr/bin/env python3
"""Synchronize existing canonical HIST prose copies; preserve authored prose."""
from __future__ import annotations
import argparse
import csv
import json
from pathlib import Path
import re

HEADER = re.compile(r"(?m)^### HIST:([^\s]+)\s*$")
LABELS = ("Source", "Requirement", "Disposition", "Remaining proof", "Gates", "Origin")

def fields(row):
    return dict(zip(LABELS, (
        row["historical_revision"] + "; " + row["source_document"],
        row["decision_requirement"],
        row["authority_status"] + "; current " + row["current_main_status"],
        row["remaining_work"], row["tests_gates"],
        row["source_ledger_row"] + "; " + row["relationship"],
    )))

def synchronize(original, canonical):
    identifiers, changes, sections = set(), [], 0
    for match in HEADER.finditer(original):
        source_id = match.group(1)
        if source_id in identifiers or source_id not in canonical:
            raise ValueError("Duplicate or unknown HIST prose ID: " + source_id)
        identifiers.add(source_id)
        end_heading = re.search(r"(?m)^#{1,3} ", original[match.end():])
        end = match.end() + end_heading.start() if end_heading else len(original)
        block = original[match.end():end]
        for label, expected in fields(canonical[source_id]).items():
            matches = list(re.finditer(r"(?m)^- " + re.escape(label) + r": ([^\n]*)$", block))
            if len(matches) != 1:
                raise ValueError(f"HIST {source_id} must have one {label} field")
            field = matches[0]
            if field.group(1) != expected:
                changes.append((match.end() + field.start(1), match.end() + field.end(1), expected))
        sections += 1
    result = original
    for start, end, expected in reversed(changes):
        result = result[:start] + expected + result[end:]
    return result, sections, len(changes)

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[3])
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    root = args.root.resolve()
    with (root / "docs/refactoring-plan/historical-obligations-canonical.tsv").open() as stream:
        rows = list(csv.DictReader(stream, delimiter="\t"))
    canonical = {row["id"]: row for row in rows}
    if len(canonical) != len(rows):
        raise ValueError("Duplicate canonical historical ID")
    with (root / "docs/refactoring-plan/task-index.tsv").open() as stream:
        task_ids = [row["task_id"] for row in csv.DictReader(stream, delimiter="\t")]
    if len(task_ids) != len(set(task_ids)) or any(not re.fullmatch(r"TASK-\d{3}", task) for task in task_ids):
        raise ValueError("Invalid canonical task inventory")
    with (root / "docs/refactoring-plan/history-prose-membership.tsv").open() as stream:
        membership = list(csv.DictReader(stream, delimiter="\t"))
    expected = {(row["task_id"], row["source_id"]) for row in membership}
    if len(expected) != len(membership) or any(task not in task_ids or source not in canonical for task, source in expected):
        raise ValueError("Invalid declared HIST prose membership")
    outputs, count, changed_fields, observed = [], 0, 0, set()
    for task_id in sorted(task_ids):
        path = root / "refactoring-tasks/terminal-components/completion" / task_id.removeprefix("TASK-") / "trusted/obligations.md"
        original = path.read_text()
        observed.update((task_id, match.group(1)) for match in HEADER.finditer(original))
        rendered, sections, changes = synchronize(original, canonical)
        count += sections
        changed_fields += changes
        if rendered != original:
            outputs.append((path, original, rendered))
    if observed != expected:
        raise ValueError("HIST prose membership differs: missing=" + str(sorted(expected-observed))
                         + "; extra=" + str(sorted(observed-expected)))
    if args.write:
        # Canonical repeated-field projection only; all surrounding authored
        # contracts, section membership and ordering remain byte-preserved.
        if any(path.read_text() != original for path, original, _ in outputs):
            raise ValueError("Source changed during synchronization")
        for path, _, rendered in outputs:
            path.write_text(rendered)
    print(json.dumps({"sections": count, "changed_fields": changed_fields,
                      "changed_files": [str(path.relative_to(root)) for path, _, _ in outputs],
                      "written": args.write}, indent=2))
    return 0 if args.write or not outputs else 1

if __name__ == "__main__":
    raise SystemExit(main())
