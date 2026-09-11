#!/usr/bin/env python3
"""Enumerate exact branch deltas; this is inventory, never a review receipt."""
from __future__ import annotations
import argparse
import csv
import io
import json
from pathlib import Path
import subprocess

MAIN = "7b27732a8c3c131760ec3438f641cb3c11343a42"
HOLLA = "2e2401393c47360741ebd321679de08982dca50a"
ORACLE = "02f5294bfdbf38004cc49130d0aff1d01f31434c"

def git(root, *args):
    return subprocess.check_output(["git", *args], cwd=root)

def partition(path):
    for name, old, new in (
        ("showcase", "src/bin/showcase/", "apps/showcase/"),
        ("holla", "src/bin/holla/", "apps/holla/"),
        ("jackin", "src/bin/jackin_preview/", "apps/jackin-preview/"),
        ("tablepro", "src/bin/tablepro/", "apps/tablepro/"),
    ):
        if path.startswith((old, new)):
            return name
    if path.startswith(("baseline/", "shots/", "parity/")):
        return "artifacts"
    if path.startswith("docs/") or path.endswith(".md") and "/" not in path:
        return "history"
    if path.startswith(("src/widgets/", "crates/tui/src/components/", "crates/tui/src/collection/")):
        return "components"
    if path.startswith(("src/", "crates/tui/src/")):
        return "foundation"
    return "tooling-tests"

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[3]
    for ref, expected in (("main", MAIN), ("holla", HOLLA), ("holla-fable-2026-09-10^{}", ORACLE)):
        if git(root, "rev-parse", ref).decode().strip() != expected:
            raise ValueError("Branch identity changed: " + ref)
    raw = git(root, "diff", "--raw", "--no-abbrev", "--no-renames", "-z", HOLLA, MAIN)
    fields = raw.split(b"\0")
    rows = []
    for index in range(0, len(fields) - 1, 2):
        metadata, path = fields[index].decode().split(), fields[index + 1].decode()
        old_mode, new_mode, old_blob, new_blob, status = metadata
        rows.append(dict(path=path, status=status, holla_mode=old_mode.removeprefix(":"),
                         main_mode=new_mode, holla_blob=old_blob, main_blob=new_blob,
                         partition=partition(path), review_status="unread"))
    if len({row["path"] for row in rows}) != len(rows):
        raise ValueError("Duplicate delta path")
    stats = git(root, "diff", "--numstat", "--no-renames", "-z", HOLLA, MAIN)
    by_path = {row["path"]: row for row in rows}
    for record in stats.split(b"\0"):
        if not record:
            continue
        added, removed, path = record.decode().split("\t", 2)
        by_path[path].update(added=added, removed=removed, kind="binary" if added == "-" else "text")
    if any("kind" not in row for row in rows):
        raise ValueError("Missing numstat row")
    summary = {"main": MAIN, "holla": HOLLA, "oracle": ORACLE, "rename_detection": False,
               "changed_paths": len(rows), "partitions": {}}
    for row in rows:
        counter = summary["partitions"].setdefault(row["partition"], dict(paths=0, text=0, binary=0, added=0, removed=0))
        counter["paths"] += 1
        counter[row["kind"]] += 1
        if row["kind"] == "text":
            counter["added"] += int(row["added"])
            counter["removed"] += int(row["removed"])
    columns = ["path", "status", "holla_mode", "main_mode", "holla_blob", "main_blob",
               "partition", "kind", "added", "removed", "review_status"]
    output = io.StringIO(newline="")
    writer = csv.DictWriter(output, fieldnames=columns, delimiter="\t", lineterminator="\n")
    writer.writeheader()
    writer.writerows(rows)
    if args.write:
        (root / "docs/refactoring-plan/branch-diff-inventory.tsv").write_text(output.getvalue())
        (root / "docs/refactoring-plan/branch-diff-inventory.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))

if __name__ == "__main__":
    main()
