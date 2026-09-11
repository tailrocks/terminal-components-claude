#!/usr/bin/env python3
"""Mechanically synchronize derived-flow trace edges into protected task payloads."""

from __future__ import annotations

import argparse
import csv
import io
import json
from pathlib import Path


def read(path):
    with path.open(newline="", encoding="utf-8") as stream:
        reader = csv.DictReader(stream, delimiter="\t")
        return reader.fieldnames, list(reader)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[3])
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    root = args.root.resolve()
    docs = root / "docs/refactoring-plan"
    catalog = root / "refactoring-tasks/terminal-components/completion"
    _, trace = read(docs / "traceability-jackin-tablepro.tsv")
    edges = [row for row in trace if row["source_namespace"] == "APP-FLOW"]
    exemplars = {}
    for edge in edges:
        if edge["role"] != "primary":
            continue
        package = catalog / edge["task_id"].removeprefix("TASK-")
        _, payload = read(package / "trusted/source-obligations.tsv")
        matching = [row for row in payload if row["source_namespace"] == "APP-FLOW" and row["source_id"] == edge["source_id"] and row["role"] == "primary"]
        if len(matching) != 1 or edge["source_id"] in exemplars:
            raise ValueError(f"Missing or ambiguous authored primary payload: {edge['source_id']}")
        exemplars[edge["source_id"]] = matching[0]
    outputs = {}
    for package in sorted(catalog.glob("[0-9][0-9][0-9]")):
        path = package / "trusted/source-obligations.tsv"
        columns, _ = read(path)
        original = path.read_text(encoding="utf-8")
        # Existing historical rows retain their exact authored bytes and order.
        retained = "".join(line for line in original.splitlines(keepends=True) if not line.startswith("APP-FLOW\t"))
        if not retained.endswith("\n"):
            raise ValueError(f"Missing payload final newline: {path}")
        output = io.StringIO(newline="")
        writer = csv.DictWriter(output, fieldnames=columns, delimiter="\t", lineterminator="\n")
        task_edges = [edge for edge in edges if edge["task_id"] == "TASK-" + package.name]
        for edge in sorted(task_edges, key=lambda row: tuple(row.values())):
            payload = dict(exemplars[edge["source_id"]])
            for key, value in edge.items():
                if key in columns:
                    payload[key] = value
            writer.writerow(payload)
        rendered = retained + output.getvalue()
        if rendered != original:
            outputs[path] = rendered
    if args.write:
        # This operation copies authored clauses and joins authored edge fields;
        # it never chooses owners, requirements, expected states or dispositions.
        for path, content in outputs.items():
            path.write_text(content, encoding="utf-8")
    print(json.dumps({"sources": len(exemplars), "edges": len(edges), "written": args.write,
                      "changed": [str(path.relative_to(root)) for path in outputs]}, indent=2))


if __name__ == "__main__":
    main()
