#!/usr/bin/env python3
"""Copy reviewed planning inputs byte-for-byte and record exact task asset hashes.

This mechanical assembler does not approve a review or qualify an implementation.
The operator invokes --write only for a group whose source bytes were reviewed.
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import io
import json
from pathlib import Path


PROOF = [
    "proof-comparator-bootstrap.py", "proof-comparator-vectors.json", "proof-comparator-protocol.md",
    "host-bootstrap-driver.py", "host-bootstrap-observer.py", "host-bootstrap-vectors.json", "host-bootstrap-protocol.md",
] + ["host-bootstrap-fixture/" + name for name in (
    "README.md", "AGENTS.md", "task.toml", "verify.toml", "check.py", "forged-host.py", "substituting-host.py",
)]
RUNNER = ["runner-bootstrap-" + name for name in (
    "driver.py", "app.py", "worker.py", "index.py", "extensions.py", "accounting.py", "native.py",
    "performance.rs", "protocol.md", "extensions-protocol.md",
)] + ["host-bootstrap-observer.py"]
FLOW = ["app-flow-" + name for name in (
    "contributions.tsv", "frame-contributions.tsv", "stage-audit.tsv", "contribution-contract.md",
)] + ["holla-stage-audit.tsv", "holla-stage-contributions.tsv", "holla-stage-contract.md", "holla-route-expansion.md", "holla-route-sites.tsv",
      "holla-trace-corrections.md", "shell-contribution-contract.md", "shell-contributions.tsv", "shell-frame-contributions.tsv"]
ARCHITECTURE = ["architecture-bootstrap-" + name for name in (
    "driver.py", "vectors.json", "protocol.md", "main-driver.py", "actual-driver.py", "state-vectors.json", "main-probe.rs",
    "conformance-probe.rs", "rain-probe.rs", "nested-row-probe.rs",
    "fixture/app.rs", "fixture/library.rs", "fixture/main.rs",
    "source-driver.py", "source-policy.py", "source-observer.rs", "source-protocol.md",
)] + ["main-source.tar.gz"]
STYLE_TIMING = ["style-timing-bootstrap-" + name for name in (
    "driver.py", "probe.rs", "frame.rs", "protocol.md",
)] + ["architecture-bootstrap-actual-driver.py", "architecture-bootstrap-main-driver.py",
      "runner-bootstrap-driver.py", "host-bootstrap-observer.py", "runner-bootstrap-app.py",
      "runner-bootstrap-worker.py", "main-source.tar.gz"]
BROKER = ["broker-bootstrap-" + name for name in (
    "driver.py", "observer.rs", "protocol.md",
)] + ARCHITECTURE + RUNNER
COLUMNS = ["group", "source", "destination", "sha256"]
GROUPS = {
    "proof": (PROOF, ("001", "070", "071", "072")),
    "runner": (RUNNER, ("070", "071", "072")),
    "flow": (FLOW, ("069", "070", "071", "072")),
    "architecture": (ARCHITECTURE, ("072",)),
    "style-timing": (STYLE_TIMING, ("072",)),
    "broker": (BROKER, ("072",)),
}


def expected_bindings(group=None):
    """Expand the trusted, complete asset membership without reading candidate data."""
    selected = GROUPS if group is None else {group: GROUPS[group]}
    rows = []
    for label, (names, tasks) in selected.items():
        prefix = "docs/refactoring-plan/" + ("" if label == "flow" else "evidence/")
        suffix = "" if label == "flow" else label + "-bootstrap/"
        for task in tasks:
            for name in names:
                rows.append((label, prefix + name,
                             f"refactoring-tasks/terminal-components/completion/{task}/trusted/{suffix}{name}"))
    if len(rows) != len(set(rows)) or len(rows) != len({row[2] for row in rows}):
        raise ValueError("Duplicate trusted bootstrap binding")
    return tuple(rows)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def bounded(root, relative):
    path = root / relative
    if Path(relative).is_absolute() or ".." in Path(relative).parts:
        raise ValueError(f"Unsafe relative path: {relative}")
    current = root
    for part in Path(relative).parts:
        current = current / part
        if current.is_symlink():
            raise ValueError(f"Symlinked asset path: {relative}")
    return path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[3])
    parser.add_argument("--group", choices=tuple(GROUPS), required=True)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--write", action="store_true")
    mode.add_argument("--check", action="store_true")
    args = parser.parse_args()
    root = args.root.resolve()
    group = args.group
    manifest = root / "docs/refactoring-plan/bootstrap-assets.tsv"
    previous = {}
    if manifest.exists():
        with manifest.open(newline="", encoding="utf-8") as stream:
            reader = csv.DictReader(stream, delimiter="\t")
            if reader.fieldnames != COLUMNS:
                raise ValueError("Bootstrap asset manifest schema differs")
            for row in reader:
                if row["destination"] in previous:
                    raise ValueError("Duplicate bootstrap destination")
                previous[row["destination"]] = row
    rows, copies = [], []
    for group, source, destination in expected_bindings(group):
        source_path, target_path = bounded(root, source), bounded(root, destination)
        if not source_path.is_file():
            raise ValueError(f"Missing reviewed source asset: {source}")
        content = source_path.read_bytes()
        row = {"group": group, "source": source, "destination": destination, "sha256": sha(content)}
        rows.append(row)
        old = previous.get(destination)
        if target_path.exists() and not target_path.is_file():
            raise ValueError(f"Nonregular asset destination: {destination}")
        current = target_path.read_bytes() if target_path.exists() else None
        if current is not None and current != content and (old is None or sha(current) != old["sha256"]):
            raise ValueError(f"Unowned destination changes: {destination}")
        if args.check and (old != row or current != content):
            raise ValueError(f"Unfrozen or stale task asset: {destination}")
        if current != content:
            copies.append((target_path, content))
    selected = {row["destination"] for row in rows}
    obsolete = [row for row in previous.values() if row["group"] == group and row["destination"] not in selected]
    if obsolete:
        raise ValueError("Manifest contains obsolete destinations; explicit reviewed disposition required")
    if args.write:
        # Explicit bulk mechanical copy; preserve every reviewed byte, including
        # the canonical fixture AGENTS.md. No instruction is authored here.
        for path, content in copies:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(content)
        combined = {**previous, **{row["destination"]: row for row in rows}}
        output = io.StringIO(newline="")
        writer = csv.DictWriter(output, fieldnames=COLUMNS, delimiter="\t", lineterminator="\n")
        writer.writeheader()
        writer.writerows(combined[key] for key in sorted(combined))
        manifest.write_text(output.getvalue(), encoding="utf-8")
    print(json.dumps({"group": group, "assets": len(rows), "changed": len(copies),
                      "written": args.write, "checked": args.check}, indent=2))


if __name__ == "__main__":
    main()
