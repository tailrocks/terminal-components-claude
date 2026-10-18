#!/usr/bin/env python3
"""Mechanically join authored traceability fragments and matrix ownership columns."""

from __future__ import annotations

import argparse
import csv
import io
import json
import tomllib
from pathlib import Path


COLUMNS = ["source_namespace", "source_id", "task_id", "requirement_id", "acceptance_id", "check_id", "role", "disposition"]
FRAGMENTS = ["traceability-history.tsv", "traceability-components.tsv", "traceability-showcase-holla.tsv", "traceability-jackin-tablepro.tsv", "traceability-closure.tsv"]


def read(path: Path) -> tuple[list[str], list[dict[str, str]]]:
    with path.open(newline="", encoding="utf-8") as stream:
        reader = csv.DictReader(stream, delimiter="\t")
        rows = list(reader)
        columns = reader.fieldnames or []
    if len(columns) != len(set(columns)) or any(None in row or None in row.values() for row in rows):
        raise ValueError(f"Malformed table: {path}")
    return columns, rows


def render(columns: list[str], rows: list[dict[str, str]]) -> str:
    output = io.StringIO(newline="")
    writer = csv.DictWriter(output, fieldnames=columns, delimiter="\t", lineterminator="\n")
    writer.writeheader()
    writer.writerows(rows)
    return output.getvalue()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[3])
    parser.add_argument("--write", action="store_true", help="Apply only the deterministic joins; no semantic assignments are inferred")
    args = parser.parse_args()
    docs = args.root.resolve() / "docs/refactoring-plan"
    combined = []
    seen = set()
    for name in FRAGMENTS:
        columns, rows = read(docs / name)
        if columns != COLUMNS:
            raise ValueError(f"Traceability schema differs: {name}")
        for row in rows:
            identity = tuple(row[column] for column in COLUMNS)
            if identity in seen:
                raise ValueError(f"Duplicate authored edge: {name}: {identity}")
            seen.add(identity)
            combined.append(row)
    combined.sort(key=lambda row: tuple(row[column] for column in COLUMNS))
    owners: dict[tuple[str, str], set[str]] = {}
    for row in combined:
        owners.setdefault((row["source_namespace"], row["source_id"]), set()).add(row["task_id"])
    outputs = {docs / "traceability.tsv": render(COLUMNS, combined)}
    for namespace, name, key, owner_key in (
        ("ARCH", "architecture-matrix.tsv", "id", "task_ids"),
        ("COMP", "component-parity.tsv", "family", "owning_task_ids"),
        ("APP", "application-parity.tsv", "scenario_id", "owning_task_ids"),
        ("DEC", "decision-ledger.tsv", "id", "task_ids"),
    ):
        columns, rows = read(docs / name)
        for row in rows:
            source = (namespace, row[key])
            if source not in owners:
                raise ValueError(f"No authored owner for {source}; join stopped before writing")
            row[owner_key] = ";".join(sorted(owners[source]))
        outputs[docs / name] = render(columns, rows)
    columns, rows = read(docs / "task-index.tsv")
    for row in rows:
        package = args.root.resolve() / "refactoring-tasks/terminal-components/completion" / row["task_id"].removeprefix("TASK-")
        verify = tomllib.loads((package / "verify.toml").read_text(encoding="utf-8"))
        metadata = tomllib.loads((package / "task.toml").read_text(encoding="utf-8"))
        row["dependencies"] = ";".join("TASK-" + value.rsplit("/", 1)[-1] for value in metadata["dependencies"])
        row["writable_paths"] = ";".join(verify["writable_paths"])
    outputs[docs / "task-index.tsv"] = render(columns, rows)
    # This is an explicit bulk mechanical rewrite. All semantic assignments
    # remain in independently authored fragments; this tool never invents them.
    changed = []
    for path, content in outputs.items():
        if not path.exists() or path.read_text(encoding="utf-8") != content:
            changed.append(str(path.relative_to(args.root.resolve())))
            if args.write:
                path.write_text(content, encoding="utf-8")
    print(json.dumps({"schema": "tc-plan-assembly/v1", "written": args.write, "edges": len(combined), "sources": len(owners), "changed": changed}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
