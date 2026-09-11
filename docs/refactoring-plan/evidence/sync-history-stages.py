#!/usr/bin/env python3
"""Apply the authored finite Holla history remap and synchronize historical joins."""

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


def render(columns, rows, header=True):
    output = io.StringIO(newline="")
    writer = csv.DictWriter(output, fieldnames=columns, delimiter="\t", lineterminator="\n")
    if header:
        writer.writeheader()
    writer.writerows(rows)
    return output.getvalue()


def bind_authority(edge, canonical):
    """Keep authored task semantics, but derive the source authority suffix once."""
    source = canonical[edge["source_id"]]
    semantic = edge["disposition"].split(" Source disposition:", 1)[0].rstrip()
    return {**edge, "disposition": semantic + " Source disposition: "
            + source["authority_status"] + ". Exact source: "
            + source["source_ledger_row"] + "."}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[3])
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    root = args.root.resolve()
    docs = root / "docs/refactoring-plan"
    columns, edges = read(docs / "traceability-history.tsv")
    canonical_columns, canonical_rows = read(docs / "historical-obligations-canonical.tsv")
    canonical = {row["id"]: row for row in canonical_rows}
    if len(canonical) != len(canonical_rows):
        raise ValueError("Duplicate canonical historical source")
    edges = [bind_authority(edge, canonical) for edge in edges]
    _, instructions = read(docs / "holla-history-stage-remap.tsv")
    mapping = {"task": "task_id", "requirement": "requirement_id", "acceptance": "acceptance_id", "check": "check_id", "role": "role"}
    for instruction in instructions:
        source = {"source_namespace": instruction["source_namespace"], "source_id": instruction["source_id"]}
        old = {**source, **{field: instruction["from_" + key] for key, field in mapping.items()}}
        new = bind_authority({**source, **{field: instruction["to_" + key] for key, field in mapping.items()}, "disposition": instruction["disposition"]}, canonical)
        matches = [row for row in edges if all(row[key] == value for key, value in old.items())]
        if new in edges:
            continue
        if instruction["operation"] == "replace":
            if len(matches) != 1:
                raise ValueError(f"Ambiguous authored replacement: {old}")
            edges.remove(matches[0])
        elif instruction["operation"] != "add":
            raise ValueError("Unknown history-remap operation")
        edges.append(new)
    edges.sort(key=lambda row: tuple(row[column] for column in columns))
    if len({tuple(row[column] for column in columns) for row in edges}) != len(edges):
        raise ValueError("Duplicate historical edge")
    owners = {}
    for edge in edges:
        owners.setdefault(edge["source_id"], set()).add(edge["task_id"])
    if set(canonical) != set(owners):
        raise ValueError("Historical owner union differs from canonical sources")
    for source_id, row in canonical.items():
        row["task_ids"] = ";".join(sorted(owners[source_id]))
    outputs = {
        docs / "traceability-history.tsv": render(columns, edges),
        docs / "historical-obligations-canonical.tsv": render(canonical_columns, canonical_rows),
        docs / "history-task-map.tsv": render(["source_namespace", "source_id", "task_ids"], [
            {"source_namespace": "HIST", "source_id": source_id, "task_ids": canonical[source_id]["task_ids"]}
            for source_id in sorted(canonical)
        ]),
    }
    for path in sorted(docs.glob("*obligations.tsv")):
        if path.name == "historical-obligations-canonical.tsv":
            continue
        source_columns, rows = read(path)
        if "task_ids" in source_columns:
            for row in rows:
                row["task_ids"] = canonical[row["id"]]["task_ids"]
            outputs[path] = render(source_columns, rows)
    catalog = root / "refactoring-tasks/terminal-components/completion"
    for package in sorted(catalog.glob("[0-9][0-9][0-9]")):
        path = package / "trusted/source-obligations.tsv"
        payload_columns, _ = read(path)
        payload_rows = []
        for edge in edges:
            if edge["task_id"] != "TASK-" + package.name:
                continue
            payload_rows.append({**canonical[edge["source_id"]], **{key: value for key, value in edge.items() if key in payload_columns}})
        preserved = "".join(line for line in path.read_text().splitlines(keepends=True) if line.startswith("APP-FLOW\t"))
        outputs[path] = render(payload_columns, payload_rows) + preserved
    changed = [path for path, content in outputs.items() if path.read_text() != content]
    if args.write:
        # Authored ownership/check fields and canonical authority suffixes are
        # projected mechanically. Historical requirements remain intact.
        for path in changed:
            path.write_text(outputs[path], encoding="utf-8")
    print(json.dumps({"sources": len(canonical), "edges": len(edges), "instructions": len(instructions),
                      "written": args.write, "changed_files": len(changed)}, indent=2))


if __name__ == "__main__":
    main()
