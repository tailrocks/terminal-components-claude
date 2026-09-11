#!/usr/bin/env python3
"""Record or check exact final planning artifacts without approving their content."""

from __future__ import annotations

import argparse
import csv
import hashlib
import io
import json
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[3])
    parser.add_argument("--write", action="store_true", help="Mechanically regenerate the final hash inventory after reviewed edits")
    args = parser.parse_args()
    root = args.root.resolve()
    manifest = root / "docs/refactoring-plan/planning-artifacts.tsv"
    paths = [root / "REFACTORING_COMPLETION_PLAN.md"]
    for directory in (root / "docs/refactoring-plan", root / "refactoring-tasks"):
        for path in directory.rglob("*"):
            if path.is_symlink():
                raise ValueError(f"Unexpected symlink in planning artifact set: {path}")
            if "__pycache__" in path.parts or path.suffix == ".pyc":
                raise ValueError(f"Generated Python cache is not a planning artifact: {path}")
            if path.is_file() and path != manifest:
                paths.append(path)
    rows = []
    for path in sorted(paths):
        with path.open("rb") as stream:
            digest = hashlib.file_digest(stream, "sha256").hexdigest()
        rows.append({"path": str(path.relative_to(root)), "sha256": digest, "bytes": path.stat().st_size})
    output = io.StringIO(newline="")
    writer = csv.DictWriter(output, fieldnames=("path", "sha256", "bytes"), delimiter="\t", lineterminator="\n")
    writer.writeheader()
    writer.writerows(rows)
    expected = output.getvalue()
    if args.write:
        manifest.write_text(expected, encoding="utf-8")
    elif not manifest.is_file() or manifest.read_text(encoding="utf-8") != expected:
        raise ValueError("Planning artifact hashes/membership differ; review changes before regenerating")
    print(json.dumps({"schema": "tc-planning-integrity/v1", "files": len(rows),
                      "bytes": sum(row["bytes"] for row in rows), "written": args.write,
                      "manifest_sha256": hashlib.sha256(expected.encode()).hexdigest()}, indent=2))


if __name__ == "__main__":
    main()
