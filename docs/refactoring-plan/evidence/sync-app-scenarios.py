#!/usr/bin/env python3
"""Check/project existing app scenario copies; never invent routes or task ownership.

Default is read-only and exits 1 for drift. --write performs only the selected
mechanical projection. Use --scope shared while app package authors are active;
the coordinator may later select packages/all after their source handoff.
"""

from __future__ import annotations

import argparse
import csv
import io
import json
from pathlib import Path
import re


SPECS = {
    "showcase": ("scenario_id", "source_refs", range(32, 40)),
    "holla": ("id", "oracle_source", range(40, 51)),
    "jackin": ("id", "source", range(51, 58)),
    "tablepro": ("scenario_id", "oracle_source", range(58, 65)),
}
SHELL_IDS = {"SC-SHELL-C02", "SC-SHELL-C06", "SC-SHELL-C07"}
CORRECTION_BINDING = "- [holla-trace-corrections.md](/task/trusted/holla-trace-corrections.md): all sixteen named source-correct branches are normative; four preview, four Args and eight editing/idle overlay branches."
HEADER = re.compile(r"^### (?:APP:)?((?:SC-|HO-|JA-|TP-)[^\n]+)\n", re.MULTILINE)
NEXT_SECTION = re.compile(r"^#{1,3} ", re.MULTILINE)


def table(content: str, label: str, key: str) -> tuple[list[str], list[dict[str, str]]]:
    reader = csv.DictReader(io.StringIO(content, newline=""), delimiter="\t")
    columns = reader.fieldnames or []
    rows = list(reader)
    if not columns or len(columns) != len(set(columns)) or key not in columns:
        raise ValueError(f"Invalid table columns: {label}")
    if any(None in row or any(value is None for value in row.values()) for row in rows):
        raise ValueError(f"Invalid table shape: {label}")
    ids = [row[key] for row in rows]
    if not ids or any(not item for item in ids) or len(ids) != len(set(ids)):
        raise ValueError(f"Missing/duplicate table identity: {label}")
    return columns, rows


def render(columns: list[str], rows: list[dict[str, str]]) -> str:
    output = io.StringIO(newline="")
    writer = csv.DictWriter(output, fieldnames=columns, delimiter="\t", lineterminator="\n")
    writer.writeheader()
    writer.writerows(rows)
    return output.getvalue()


def project_sections(content: str, rows: dict[str, dict[str, str]], key: str,
                     label: str) -> tuple[str, int, int]:
    """Project present blocks, not expected task/scenario membership.

    Deleting one whole block can pass while other blocks remain. Independent
    source-owner review and the frozen artifact manifest own that separate gate.
    """
    changes = []
    seen = set()
    fields = 0
    for match in HEADER.finditer(content):
        scenario = match.group(1)
        if scenario not in rows or scenario in seen:
            raise ValueError(f"Unknown/duplicate scenario section: {label}:{scenario}")
        seen.add(scenario)
        end_match = NEXT_SECTION.search(content, match.end())
        end = end_match.start() if end_match else len(content)
        section = content[match.end():end]
        copied = re.findall(r"^- (?:\*\*([a-z][a-z0-9_]*):\*\*|([a-z][a-z0-9_]*):) ", section, re.MULTILINE)
        observed_fields = [bold or plain for bold, plain in copied]
        if set(observed_fields) != set(rows[scenario]) - {key}:
            raise ValueError(f"Copied field set differs: {label}:{scenario}")
        for field, value in rows[scenario].items():
            if field == key:
                continue
            if "\n" in value or "\r" in value:
                raise ValueError(f"Multiline source field cannot use line-copy format: {label}:{scenario}:{field}")
            # Both formats already exist in this catalog. Preserve each prefix.
            pattern = re.compile(r"^(- (?:\*\*" + re.escape(field)
                                 + r":\*\*|" + re.escape(field) + r":) )([^\n]*)$", re.MULTILINE)
            matches = list(pattern.finditer(section))
            if len(matches) != 1:
                raise ValueError(f"Missing/duplicate copied field: {label}:{scenario}:{field}")
            field_match = matches[0]
            changes.append((match.end() + field_match.start(2),
                            match.end() + field_match.end(2), value))
            fields += 1
    if not seen:
        raise ValueError(f"No existing scenario sections: {label}")
    for start, end, value in sorted(changes, reverse=True):
        content = content[:start] + value + content[end:]
    return content, len(seen), fields


def plan(root: Path, apps: list[str], scope: str) -> tuple[dict[Path, str], dict[Path, str | None], dict]:
    docs = root / "docs/refactoring-plan"
    catalog = root / "refactoring-tasks/terminal-components/completion"
    inputs: dict[Path, str | None] = {}
    outputs = {}
    counts = {"scenario_rows": 0, "copied_sections": 0, "copied_fields": 0}

    def read(path: Path, optional: bool = False) -> str | None:
        if path not in inputs:
            inputs[path] = path.read_text(encoding="utf-8") if path.is_file() else None
        if inputs[path] is None and not optional:
            raise ValueError(f"Missing input: {path}")
        return inputs[path]

    sources = {}
    for app in apps:
        key, source_field, numbers = SPECS[app]
        filename = app + "-scenarios.tsv"
        _, values = table(read(docs / filename), filename, key)
        sources[app] = {row[key]: row for row in values}
        counts["scenario_rows"] += len(values)
        # Baseline002–005 keep source links/host bindings; no new appendices.
        baseline = catalog / {"showcase": "002", "holla": "003", "jackin": "004", "tablepro": "005"}[app]
        if filename not in read(baseline / "trusted/obligations.md"):
            raise ValueError(f"Missing baseline source binding: {app}")
        if scope in {"all", "packages"}:
            for number in numbers:
                path = catalog / f"{number:03}/trusted/obligations.md"
                value, sections, fields = project_sections(read(path), sources[app], key, str(path))
                outputs[path] = value
                counts["copied_sections"] += sections
                counts["copied_fields"] += fields

    if scope in {"all", "shared"}:
        path = docs / "application-parity.tsv"
        columns, rows = table(read(path), str(path), "scenario_id")
        for app in apps:
            matching = [row for row in rows if row["application"] == app]
            if {row["scenario_id"] for row in matching} != set(sources[app]):
                raise ValueError(f"Application/source identity join differs: {app}")
            for row in matching:
                scenario = row["scenario_id"]
                source = sources[app][scenario][SPECS[app][1]]
                row["reference_state"] = f"{app}-scenarios.tsv#{scenario}; " + (source if source.startswith("O:") else "O:" + source)
        # No action, disposition, owner, component mapping or proof-status inference.
        outputs[path] = render(columns, rows)
        if "showcase" in apps:
            path = docs / "shell-contributions.tsv"
            columns, rows = table(read(path), str(path), "contribution_id")
            donor_path = catalog / "032/trusted/contributions.tsv"
            donor_columns, donor = table(read(donor_path), str(donor_path), "contribution_id")
            if donor_columns != columns:
                raise ValueError("Shell contribution schemas differ")
            selected = {row["contribution_id"]: row for row in donor if row["contribution_id"] in SHELL_IDS}
            if set(selected) != SHELL_IDS or not SHELL_IDS <= {row["contribution_id"] for row in rows}:
                raise ValueError("Missing exact Showcase shell contribution")
            outputs[path] = render(columns, [selected.get(row["contribution_id"], row) for row in rows])
        if "holla" in apps:
            baseline = catalog / "003/trusted/obligations.md"
            if CORRECTION_BINDING not in read(baseline):
                raise ValueError("TASK-003 lacks reviewed sixteen-branch correction binding")
            path = catalog / "003/trusted/holla-trace-corrections.md"
            read(path, optional=True)
            outputs[path] = read(docs / "holla-trace-corrections.md")
    return outputs, inputs, counts


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[3])
    parser.add_argument("--app", action="append", choices=tuple(SPECS), help="Repeat to select apps; default all four")
    parser.add_argument("--scope", choices=("shared", "packages", "all"), default="all")
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    root = args.root.resolve()
    try:
        outputs, inputs, counts = plan(root, list(dict.fromkeys(args.app or SPECS)), args.scope)
        changed = {path: value for path, value in outputs.items() if inputs[path] != value}
        if args.write:
            # Refuse a source/target changed since projection; no partial writes
            # occur before this complete check. Coordinator still serializes authors.
            for path, before in inputs.items():
                current = path.read_text(encoding="utf-8") if path.is_file() else None
                if current != before:
                    raise ValueError(f"Concurrent input change: {path}")
            for path, value in changed.items():
                path.write_text(value, encoding="utf-8")
        print(json.dumps({**counts, "scope": args.scope, "written": args.write,
                          "changed_files": [str(path.relative_to(root)) for path in changed]}, indent=2))
        return 0 if args.write or not changed else 1
    except (OSError, ValueError, KeyError) as error:
        print(json.dumps({"error": str(error)}))
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
