#!/usr/bin/env python3
"""Project partition-ledger read coverage into branch-diff-inventory.tsv review_status."""
from __future__ import annotations

import argparse
import csv
import io
import json
import re
from pathlib import Path

INVENTORY_COLUMNS = [
    "path", "status", "holla_mode", "main_mode", "holla_blob", "main_blob",
    "partition", "kind", "added", "removed", "review_status",
]


def reviewed_jackin_tablepro(row: dict[str, str]) -> bool:
    return row.get("semantic_status", "").startswith("reviewed")


def reviewed_holla(row: dict[str, str]) -> bool:
    coverage = row.get("read_coverage", "")
    return coverage.startswith("full_")


def reviewed_showcase(row: dict[str, str]) -> bool:
    return "full" in row.get("read_coverage", "")


def reviewed_components(row: dict[str, str]) -> bool:
    for column in ("read_ranges", "read_coverage"):
        value = row.get(column, "").strip()
        if value:
            return True
    return False


def reviewed_foundation(path: str) -> bool:
    return bool(path.strip())


def reviewed_history(entry: dict) -> bool:
    status = entry.get("read_status", "")
    return status.startswith("full")


def reviewed_xtask(entry: dict) -> bool:
    status = entry.get("read_status", "")
    return status.startswith("full")


def reviewed_tui_tests_examples(row: dict[str, str]) -> bool:
    coverage = row.get("read_coverage", "")
    return coverage.startswith("main:") and "unread" not in coverage


def load_reviewed_paths(docs: Path) -> dict[str, set[str]]:
    by_ledger: dict[str, set[str]] = {}

    jackin_path = docs / "branch-diff-jackin-tablepro-read-ledger.tsv"
    with jackin_path.open(newline="", encoding="utf-8") as stream:
        rows = list(csv.DictReader(stream, delimiter="\t"))
    paths = {row["path"] for row in rows if reviewed_jackin_tablepro(row)}
    if len(paths) != len(rows):
        raise ValueError("Jackin/TablePro ledger row missing reviewed semantic_status")
    by_ledger["branch-diff-jackin-tablepro-read-ledger.tsv"] = paths

    holla_path = docs / "branch-diff-holla-ledger.tsv"
    with holla_path.open(newline="", encoding="utf-8") as stream:
        rows = list(csv.DictReader(stream, delimiter="\t"))
    paths = {row["path"] for row in rows if reviewed_holla(row)}
    if len(paths) != len(rows):
        raise ValueError("Holla ledger row missing full read_coverage")
    by_ledger["branch-diff-holla-ledger.tsv"] = paths

    showcase_path = docs / "branch-diff-showcase-ledger.tsv"
    with showcase_path.open(newline="", encoding="utf-8") as stream:
        rows = list(csv.DictReader(stream, delimiter="\t"))
    paths = {row["path"] for row in rows if reviewed_showcase(row)}
    if len(paths) != len(rows):
        raise ValueError("Showcase ledger row missing full read_coverage")
    by_ledger["branch-diff-showcase-ledger.tsv"] = paths

    component_paths: set[str] = set()
    for name in ("branch-diff-components-a.tsv", "branch-diff-components-b.tsv"):
        ledger = docs / name
        with ledger.open(newline="", encoding="utf-8") as stream:
            rows = list(csv.DictReader(stream, delimiter="\t"))
        paths = {row["path"] for row in rows if reviewed_components(row)}
        if len(paths) != len(rows):
            raise ValueError(f"{name} row missing read_ranges/read_coverage")
        component_paths |= paths
        by_ledger[name] = paths
    by_ledger["branch-diff-components"] = component_paths

    foundation_text = (docs / "branch-diff-foundation-ledger.md").read_text(encoding="utf-8")
    foundation_paths = {
        match.group(1).strip()
        for match in re.finditer(r"^\| ([^|]+?) \| [ADM];", foundation_text, re.M)
    }
    if len(foundation_paths) != 67:
        raise ValueError(f"Unexpected foundation ledger path count: {len(foundation_paths)}")
    by_ledger["branch-diff-foundation-ledger.md"] = foundation_paths

    history_payload = json.loads((docs / "branch-diff-history-ledger.json").read_text(encoding="utf-8"))
    history_paths = {entry["path"] for entry in history_payload["files"] if reviewed_history(entry)}
    if len(history_paths) != len(history_payload["files"]):
        raise ValueError("History ledger row missing full read_status")
    by_ledger["branch-diff-history-ledger.json"] = history_paths

    xtask_payload = json.loads((docs / "branch-diff-xtask-ledger.json").read_text(encoding="utf-8"))
    xtask_paths = {entry["path"] for entry in xtask_payload["files"] if reviewed_xtask(entry)}
    if len(xtask_paths) != len(xtask_payload["files"]):
        raise ValueError("xtask ledger row missing full read_status")
    by_ledger["branch-diff-xtask-ledger.json"] = xtask_paths

    tte_path = docs / "branch-diff-tui-tests-examples.tsv"
    with tte_path.open(newline="", encoding="utf-8") as stream:
        rows = list(csv.DictReader(stream, delimiter="\t"))
    paths = {row["path"] for row in rows if reviewed_tui_tests_examples(row)}
    if len(paths) != len(rows):
        raise ValueError("TUI tests/examples ledger row missing main read_coverage")
    by_ledger["branch-diff-tui-tests-examples.tsv"] = paths

    return by_ledger


def synchronize(inventory_path: Path, reviewed: set[str], write: bool) -> dict[str, int | list[str]]:
    with inventory_path.open(newline="", encoding="utf-8") as stream:
        rows = list(csv.DictReader(stream, delimiter="\t"))
    headers = rows and list(rows[0].keys()) or []
    if headers != INVENTORY_COLUMNS:
        raise ValueError("branch-diff-inventory.tsv schema drift")

    inventory_paths = [row["path"] for row in rows]
    if len(inventory_paths) != len(set(inventory_paths)):
        raise ValueError("Duplicate inventory path")

    unknown = sorted(reviewed - set(inventory_paths))
    if unknown:
        raise ValueError(f"Ledger paths absent from inventory: {unknown[:5]}{'...' if len(unknown) > 5 else ''}")

    before_reviewed = sum(row["review_status"] == "reviewed" for row in rows)
    promoted: list[str] = []
    for row in rows:
        if row["path"] in reviewed and row["review_status"] != "reviewed":
            row["review_status"] = "reviewed"
            promoted.append(row["path"])

    after_reviewed = sum(row["review_status"] == "reviewed" for row in rows)
    if write:
        output = io.StringIO(newline="")
        writer = csv.DictWriter(output, fieldnames=INVENTORY_COLUMNS, delimiter="\t", lineterminator="\n")
        writer.writeheader()
        writer.writerows(rows)
        inventory_path.write_text(output.getvalue(), encoding="utf-8")

    return {
        "total_paths": len(rows),
        "reviewed_before": before_reviewed,
        "reviewed_after": after_reviewed,
        "newly_reviewed": len(promoted),
        "promoted_paths": promoted,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[3])
    parser.add_argument("--write", action="store_true")
    parser.add_argument("--summary", action="store_true")
    args = parser.parse_args()

    root = args.root.resolve()
    docs = root / "docs/refactoring-plan"
    inventory_path = docs / "branch-diff-inventory.tsv"
    ledgers = load_reviewed_paths(docs)
    reviewed = set().union(*ledgers.values())
    summary = synchronize(inventory_path, reviewed, args.write)
    summary["ledger_counts"] = {name: len(paths) for name, paths in ledgers.items()}
    if args.summary or not args.write:
        print(json.dumps(summary, indent=2, sort_keys=True))
    else:
        print(json.dumps({key: summary[key] for key in (
            "total_paths", "reviewed_before", "reviewed_after", "newly_reviewed",
        )}, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
