#!/usr/bin/env python3
"""Derive graph depth, exact dependencies and conservative shared-file locks."""

from __future__ import annotations

import argparse
import csv
import json
import re
import tomllib
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[3])
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    root = args.root.resolve()
    docs = root / "docs/refactoring-plan"
    with (docs / "task-index.tsv").open(newline="") as stream:
        index = {row["task_id"]: row for row in csv.DictReader(stream, delimiter="\t")}
    tasks = {}
    for task_id, row in index.items():
        package = root / "refactoring-tasks/terminal-components/completion" / task_id[5:]
        metadata = tomllib.loads((package / "task.toml").read_text())
        verify = tomllib.loads((package / "verify.toml").read_text())
        dependencies = ["TASK-" + value.rsplit("/", 1)[-1] for value in metadata["dependencies"]]
        if set(dependencies) != set(filter(None, row["dependencies"].split(";"))):
            raise ValueError(f"Index dependency drift: {task_id}")
        tasks[task_id] = {"title": row["title"], "key": row["key"], "dependencies": dependencies, "writable_paths": verify["writable_paths"]}
    ancestors = {}
    depth = {}
    longest = {}
    path_count = {}
    active = set()

    def visit(task_id):
        if task_id in active:
            raise ValueError(f"Cycle: {task_id}")
        if task_id in ancestors:
            return
        active.add(task_id)
        deps = tasks[task_id]["dependencies"]
        for dependency in deps:
            visit(dependency)
        ancestors[task_id] = set(deps).union(*(ancestors[d] for d in deps))
        best = max((depth[d] for d in deps), default=0)
        depth[task_id] = best + 1
        longest[task_id] = sorted(d for d in deps if depth[d] == best)
        path_count[task_id] = sum(path_count[d] for d in longest[task_id]) if deps else 1
        active.remove(task_id)

    for task_id in tasks:
        visit(task_id)
    # Writable paths are explicit in this catalog. A wildcard would require
    # parser-equivalent overlap handling rather than a convenient approximation.
    if any(any(c in path for c in "*?[") for task in tasks.values() for path in task["writable_paths"]):
        raise ValueError("Wildcard scope requires explicit overlap support")

    def overlaps(left, right):
        a, b = left.rstrip("/"), right.rstrip("/")
        return a == b or a.startswith(b + "/") or b.startswith(a + "/")

    locks = []
    ids = sorted(tasks)
    for offset, left in enumerate(ids):
        for right in ids[offset + 1:]:
            if left in ancestors[right] or right in ancestors[left]:
                continue
            shared = [(a, b) for a in tasks[left]["writable_paths"] for b in tasks[right]["writable_paths"] if overlaps(a, b)]
            if shared:
                locks.append({"tasks": [left, right], "overlapping_paths": shared, "rule": "Do not dispatch concurrently; use lower task ID first when both are ready, integrate and reverify before starting the other"})
    maximum = max(depth.values())
    terminal = sorted(task_id for task_id in ids if depth[task_id] == maximum)
    selected = []
    current = terminal[0]
    while True:
        selected.append(current)
        if not longest[current]:
            break
        current = longest[current][0]
    selected.reverse()
    graph = {"schema": "tc-plan-graph/v1", "tasks": tasks, "dependency_depth": depth, "longest_path_predecessors": longest, "longest_path_count": sum(path_count[t] for t in terminal), "maximum_dependency_depth": maximum, "deepest_tasks": terminal, "representative_critical_path": selected, "serialization_locks": locks}
    lines = ["# Derived execution graph", "", "This graph is generated from canonical task.toml dependencies and verify.toml scopes. Task numbers do not define execution order. Dependency depth is an unweighted critical-path measure; no invented duration estimate is used.", "", f"The graph contains {len(tasks)} tasks, maximum depth {maximum}, and {graph['longest_path_count']} equally deepest dependency paths. The complete predecessor representation is in [task-graph.json](task-graph.json).", "", "## One exact longest dependency path", "", "`" + " → ".join(selected) + "`", "", "## Earliest dependency layers", "", "Tasks in one layer are only candidates for parallel execution. Apply the shared-file locks below and require actual prerequisite code/trust receipts, not metadata status alone.", "", "| Depth | Tasks |", "| --- | --- |"]
    for level in range(1, maximum + 1):
        lines.append(f"| {level} | " + ", ".join(f"`{t}`" for t in ids if depth[t] == level) + " |")
    lines += ["", "## Shared-file serialization", "", "Disjoint ready work may run in isolated worktrees. The following incomparable tasks have overlapping writable scope and must not execute concurrently. When both are ready, dispatch the lower task ID first, integrate its verified tree, and start the other from that accepted parent. This is a declared soft scheduling constraint, not an invented task.toml field. If only the higher task is ready, it may run first; the later task must still start after its verified integration. No two such tasks share an executor or target/output directory.", "", "| Tasks | Overlapping scopes |", "| --- | --- |"]
    for lock in locks:
        paths = "; ".join(f"`{a}` / `{b}`" if a != b else f"`{a}`" for a, b in lock["overlapping_paths"])
        lines.append(f"| {'; '.join(lock['tasks'])} | {paths} |")
    if not locks:
        lines.append("| None | Hard dependencies already serialize every scope overlap |")
    lines += ["", "At every parallel join, materialize a fresh combined tree and rerun the union of impacted contracts, complete test accounting and workspace gates. Individually accepted siblings are not proof of the combined result. Compare-and-swap integration rejects a changed parent; it never silently attaches a tested tree to a different parent.", ""]
    if args.write:
        # These files are deterministic mechanical projections, not hand-edited
        # dependency authority. Canonical task metadata remains authoritative.
        (docs / "task-graph.json").write_text(json.dumps(graph, indent=2, sort_keys=True) + "\n")
        (docs / "task-graph.md").write_text("\n".join(lines))
    else:
        if json.loads((docs / "task-graph.json").read_text()) != graph:
            raise ValueError("Stored machine graph is stale; regenerate from canonical metadata")
        if (docs / "task-graph.md").read_text() != "\n".join(lines):
            raise ValueError("Stored readable graph is stale; regenerate from canonical metadata")
    plan = (root / "REFACTORING_COMPLETION_PLAN.md").read_text()
    stated = re.findall(r"maximum dependency depth (\d+) and (\d+) equally deepest paths", plan)
    if stated != [(str(maximum), str(graph["longest_path_count"]))]:
        raise ValueError("Authoritative plan depth/path count differs from canonical DAG")
    if "TASK-065" not in tasks["TASK-066"]["dependencies"] or "TASK-065/TASK-066 serialization is a hard edge" not in plan:
        raise ValueError("Required TASK-065/TASK-066 hard serialization drift")
    if re.search(r"TASK-065.{0,30}TASK-066.{0,50}soft", plan, re.I):
        raise ValueError("Authoritative plan revives the obsolete soft-only serialization")
    with (docs / "historical-obligations-canonical.tsv").open(newline="") as stream:
        historical_count = sum(1 for _ in csv.DictReader(stream, delimiter="\t"))
    inventory_readme = (root / "refactoring-tasks/terminal-components/completion/007/README.md").read_text()
    if re.findall(r"complete (\d+)-row historical union", inventory_readme) != [str(historical_count)]:
        raise ValueError("TASK-007 historical union count differs from the canonical ledger")
    print(json.dumps({"tasks": len(tasks), "maximum_dependency_depth": maximum, "longest_path_count": graph["longest_path_count"], "serialization_pairs": len(locks), "written": args.write}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
