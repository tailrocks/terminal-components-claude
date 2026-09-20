#!/usr/bin/env python3
"""Derive and validate the reconciled, machine-checkable task DAG.

The task index and task packages remain authoritative. This program only
projects their dependency, verification, scope, and traceability facts into
the generated graph. It never reads task status or emits execution
authorization.
"""

from __future__ import annotations

import argparse
import csv
import json
import re
import tomllib
from collections import defaultdict
from pathlib import Path
from typing import Any


GRAPH_SCHEMA = "tc-plan-graph/v1"
COMPLETION_REL = Path("refactoring-tasks/terminal-components/completion")
INDEX_REL = Path("docs/refactoring-plan/task-index.tsv")
TRACEABILITY_REL = Path("docs/refactoring-plan/traceability.tsv")
DIRECT_TASK_COUNT = 73
RECURSIVE_VERIFY_COUNT = 77
DIRECT_CHECK_COUNT = 506
RECURSIVE_CHECK_COUNT = 526
DEPENDENCY_EDGE_COUNT = 264
EXPECTED_NESTED_OWNERS = {"TASK-001", "TASK-070", "TASK-071", "TASK-072"}
CHECKPOINT_SUFFIXES = {
    "API",
    "CLOSE",
    "COMPONENTS",
    "CONFORMANCE",
    "CONFORMANCE-FOUNDATION",
    "FINAL",
    "OWNERSHIP",
    "PERF",
    "TESTS",
}


def task_number(task_id: str) -> int:
    match = re.fullmatch(r"TASK-(\d+)", task_id)
    if not match:
        raise ValueError(f"invalid task id: {task_id}")
    return int(match.group(1))


def task_sort(task_id: str) -> tuple[int, str]:
    return task_number(task_id), task_id


def check_sort(check: dict[str, Any]) -> tuple[int, str]:
    match = re.fullmatch(r"CHK-(\d+)", check["id"])
    if not match:
        raise ValueError(f"invalid check id: {check['id']}")
    return int(match.group(1)), check["id"]


def relative(root: Path, path: Path) -> str:
    return path.resolve().relative_to(root).as_posix()


def path_overlaps(left: str, right: str) -> bool:
    left = left.rstrip("/")
    right = right.rstrip("/")
    return left == right or left.startswith(right + "/") or right.startswith(left + "/")


def read_index(root: Path) -> tuple[dict[str, dict[str, str]], list[str]]:
    with (root / INDEX_REL).open(newline="", encoding="utf-8") as stream:
        rows = list(csv.DictReader(stream, delimiter="\t"))
    index: dict[str, dict[str, str]] = {}
    wave_order: list[str] = []
    required = {"task_id", "key", "title", "dependencies", "wave", "writable_paths", "purpose"}
    for row in rows:
        if set(row) != required:
            raise ValueError(f"task index columns changed: {sorted(row)}")
        task_id = row["task_id"]
        if task_id in index:
            raise ValueError(f"duplicate task index row: {task_id}")
        index[task_id] = row
        if row["wave"] not in wave_order:
            wave_order.append(row["wave"])
    if len(index) != DIRECT_TASK_COUNT:
        raise ValueError(f"expected {DIRECT_TASK_COUNT} task packages, found {len(index)}")
    if sorted(index, key=task_sort) != sorted(index):
        raise ValueError("task index is not ordered by task identity")
    return index, wave_order


def dependency_ids(metadata: dict[str, Any], task_id: str) -> list[str]:
    if metadata.get("schema") != "task-meta/v1":
        raise ValueError(f"{task_id} has an unexpected task metadata schema")
    dependencies = metadata.get("dependencies")
    if not isinstance(dependencies, list) or any(not isinstance(value, str) for value in dependencies):
        raise ValueError(f"{task_id} has malformed dependencies")
    result = ["TASK-" + value.rsplit("/", 1)[-1] for value in dependencies]
    if len(result) != len(set(result)):
        raise ValueError(f"{task_id} repeats a dependency")
    return result


def check_descriptor(check: dict[str, Any]) -> dict[str, Any]:
    check_id = check.get("id")
    phase = check.get("phase")
    requirements = check.get("requirements")
    acceptance = check.get("acceptance")
    if not isinstance(check_id, str) or not isinstance(phase, str):
        raise ValueError("verification check lacks id or phase")
    if not isinstance(requirements, list) or not all(isinstance(item, str) for item in requirements):
        raise ValueError(f"{check_id} has malformed requirements")
    if not isinstance(acceptance, list) or not all(isinstance(item, str) for item in acceptance):
        raise ValueError(f"{check_id} has malformed acceptance")
    if ("argv" in check) == ("shell" in check):
        raise ValueError(f"{check_id} must declare exactly one executable command")
    if "expected" not in check:
        raise ValueError(f"{check_id} has no expected result")
    return {
        "id": check_id,
        "phase": phase,
        "requirements": sorted(requirements),
        "acceptance": sorted(acceptance),
    }


def describe_contract(root: Path, owner_task_id: str, path: Path, kind: str) -> dict[str, Any]:
    if path.is_symlink() or not path.is_file():
        raise ValueError(f"verification contract is not a regular file: {path}")
    value = tomllib.loads(path.read_text(encoding="utf-8"))
    if value.get("schema") != "verify/v2":
        raise ValueError(f"{relative(root, path)} is not verify/v2")
    declared_task_id = value.get("task_id")
    if not isinstance(declared_task_id, str):
        raise ValueError(f"{relative(root, path)} has no task_id")
    checks = value.get("checks")
    if not isinstance(checks, list) or not checks:
        raise ValueError(f"{relative(root, path)} has no checks")
    descriptors = sorted((check_descriptor(check) for check in checks), key=check_sort)
    ids = [check["id"] for check in descriptors]
    if len(ids) != len(set(ids)):
        raise ValueError(f"{relative(root, path)} repeats a check id")
    writable_paths = value.get("writable_paths")
    forbidden_paths = value.get("forbidden_paths")
    if not isinstance(writable_paths, list) or not all(isinstance(item, str) for item in writable_paths):
        raise ValueError(f"{relative(root, path)} has malformed writable_paths")
    if not isinstance(forbidden_paths, list) or not all(isinstance(item, str) for item in forbidden_paths):
        raise ValueError(f"{relative(root, path)} has malformed forbidden_paths")
    result = {
        "path": relative(root, path),
        "kind": kind,
        "owner_task_id": owner_task_id,
        "declared_task_id": declared_task_id,
        "schema": value["schema"],
        "check_count": len(descriptors),
        "check_ids": [check["id"] for check in descriptors],
        "gate_check_ids": [check["id"] for check in descriptors if check["phase"] == "gate"],
        "writable_paths": writable_paths,
        "forbidden_paths": forbidden_paths,
    }
    if kind == "nested-bootstrap":
        metadata_path = path.parent / "task.toml"
        if metadata_path.is_symlink() or not metadata_path.is_file():
            raise ValueError(f"nested contract lacks task metadata: {metadata_path}")
        metadata = tomllib.loads(metadata_path.read_text(encoding="utf-8"))
        if metadata.get("schema") != "task-meta/v1":
            raise ValueError(f"nested task metadata is not task-meta/v1: {metadata_path}")
        if not isinstance(metadata.get("dependencies"), list):
            raise ValueError(f"nested task metadata has malformed dependencies: {metadata_path}")
        result["task_metadata"] = {
            "path": relative(root, metadata_path),
            "schema": metadata.get("schema"),
            "dependencies": metadata.get("dependencies"),
        }
    return result


def compact_contract(contract: dict[str, Any]) -> dict[str, Any]:
    result = {
        key: contract[key]
        for key in (
            "path",
            "kind",
            "owner_task_id",
            "declared_task_id",
            "schema",
            "check_count",
            "check_ids",
            "gate_check_ids",
        )
    }
    if "task_metadata" in contract:
        result["task_metadata"] = contract["task_metadata"]
    return result


def load_contracts(
    root: Path, index: dict[str, dict[str, str]]
) -> tuple[dict[str, list[dict[str, Any]]], list[dict[str, Any]]]:
    contracts_by_task: dict[str, list[dict[str, Any]]] = {}
    nested: list[dict[str, Any]] = []
    package_root = root / COMPLETION_REL
    actual_packages = {
        path.name
        for path in package_root.iterdir()
        if path.is_dir() and re.fullmatch(r"\d{3}", path.name)
    }
    expected_packages = {task_id[5:] for task_id in index}
    if actual_packages != expected_packages:
        raise ValueError(
            f"direct task package set drift: expected {sorted(expected_packages)}, "
            f"found {sorted(actual_packages)}"
        )
    expected_nested_paths = {
        f"{COMPLETION_REL.as_posix()}/{task_id[5:]}/trusted/proof-bootstrap/host-bootstrap-fixture/verify.toml"
        for task_id in EXPECTED_NESTED_OWNERS
    }
    actual_nested_paths: set[str] = set()
    for task_id in sorted(index, key=task_sort):
        package = root / COMPLETION_REL / task_id[5:]
        direct_path = package / "verify.toml"
        nested_paths = sorted(path for path in package.rglob("verify.toml") if path != direct_path)
        all_paths = [direct_path, *nested_paths]
        if not direct_path.is_file():
            raise ValueError(f"{task_id} has no direct verify.toml")
        contracts = [
            describe_contract(root, task_id, path, "direct" if path == direct_path else "nested-bootstrap")
            for path in all_paths
        ]
        direct = contracts[0]
        if direct["declared_task_id"] != task_id:
            raise ValueError(f"{task_id} direct verify.toml declares {direct['declared_task_id']}")
        for contract in contracts[1:]:
            actual_nested_paths.add(contract["path"])
            nested.append(contract)
        contracts_by_task[task_id] = contracts
    if actual_nested_paths != expected_nested_paths:
        raise ValueError(
            "nested bootstrap contract set drift: "
            f"expected {sorted(expected_nested_paths)}, found {sorted(actual_nested_paths)}"
        )
    if len(nested) != len(EXPECTED_NESTED_OWNERS):
        raise ValueError(f"expected four nested bootstrap contracts, found {len(nested)}")
    nested.sort(key=lambda contract: task_sort(contract["owner_task_id"]))
    return contracts_by_task, nested


def read_traceability(
    root: Path, task_ids: set[str]
) -> tuple[dict[str, list[dict[str, str]]], list[dict[str, Any]], dict[str, Any]]:
    by_task: dict[str, list[dict[str, str]]] = defaultdict(list)
    source_tasks: dict[tuple[str, str], set[str]] = defaultdict(set)
    with (root / TRACEABILITY_REL).open(newline="", encoding="utf-8") as stream:
        rows = list(csv.DictReader(stream, delimiter="\t"))
    required = {
        "source_namespace",
        "source_id",
        "task_id",
        "requirement_id",
        "acceptance_id",
        "check_id",
        "role",
        "disposition",
    }
    for row in rows:
        if set(row) != required:
            raise ValueError(f"traceability columns changed: {sorted(row)}")
        if row["task_id"] not in task_ids:
            raise ValueError(f"traceability references unknown task {row['task_id']}")
        by_task[row["task_id"]].append(row)
        source_tasks[(row["source_namespace"], row["source_id"])].add(row["task_id"])
    pair_counts: dict[tuple[str, str], dict[str, Any]] = {}
    for (namespace, source_id), members in sorted(source_tasks.items()):
        if len(members) < 2:
            continue
        ordered = sorted(members, key=task_sort)
        for offset, left in enumerate(ordered):
            for right in ordered[offset + 1 :]:
                pair = pair_counts.setdefault(
                    (left, right), {"shared_source_count": 0, "source_namespaces": set()}
                )
                pair["shared_source_count"] += 1
                pair["source_namespaces"].add(namespace)
    interfaces = [
        {
            "id": f"PAIR:{left}:{right}",
            "kind": "shared-traceability-surface",
            "tasks": [left, right],
            "shared_source_count": value["shared_source_count"],
            "source_namespaces": sorted(value["source_namespaces"]),
        }
        for (left, right), value in sorted(
            pair_counts.items(), key=lambda item: (task_sort(item[0][0]), task_sort(item[0][1]))
        )
    ]
    catalog = {
        "traceability_rows": len(rows),
        "source_groups": len(source_tasks),
        "shared_source_groups": sum(len(members) > 1 for members in source_tasks.values()),
        "source_namespaces": sorted({namespace for namespace, _ in source_tasks}),
    }
    return by_task, interfaces, catalog


def calculate_graph_facts(
    tasks: dict[str, dict[str, Any]],
) -> tuple[dict[str, set[str]], dict[str, int], dict[str, list[str]], dict[str, int]]:
    ancestors: dict[str, set[str]] = {}
    depth: dict[str, int] = {}
    longest: dict[str, list[str]] = {}
    path_count: dict[str, int] = {}
    active: set[str] = set()

    def visit(task_id: str) -> None:
        if task_id in active:
            raise ValueError(f"dependency cycle at {task_id}")
        if task_id in ancestors:
            return
        active.add(task_id)
        dependencies = tasks[task_id]["dependencies"]
        for dependency in dependencies:
            if dependency not in tasks:
                raise ValueError(f"{task_id} references unknown dependency {dependency}")
            if dependency == task_id:
                raise ValueError(f"{task_id} depends on itself")
            visit(dependency)
        ancestors[task_id] = set(dependencies).union(
            *(ancestors[dependency] for dependency in dependencies)
        )
        best = max((depth[dependency] for dependency in dependencies), default=0)
        depth[task_id] = best + 1
        longest[task_id] = sorted(
            (dependency for dependency in dependencies if depth[dependency] == best),
            key=task_sort,
        )
        path_count[task_id] = (
            sum(path_count[dependency] for dependency in longest[task_id])
            if dependencies
            else 1
        )
        active.remove(task_id)

    for task_id in sorted(tasks, key=task_sort):
        visit(task_id)
    return ancestors, depth, longest, path_count


def build_graph(root: Path) -> dict[str, Any]:
    index, wave_order = read_index(root)
    contracts_by_task, nested_contracts = load_contracts(root, index)
    traceability_by_task, shared_interfaces, traceability_catalog = read_traceability(root, set(index))

    tasks: dict[str, dict[str, Any]] = {}
    for task_id in sorted(index, key=task_sort):
        row = index[task_id]
        package = root / COMPLETION_REL / task_id[5:]
        metadata = tomllib.loads((package / "task.toml").read_text(encoding="utf-8"))
        dependencies = dependency_ids(metadata, task_id)
        index_dependencies = [value for value in row["dependencies"].split(";") if value]
        if set(dependencies) != set(index_dependencies):
            raise ValueError(f"Index dependency drift: {task_id}")
        direct = contracts_by_task[task_id][0]
        indexed_paths = [value for value in row["writable_paths"].split(";") if value]
        if set(direct["writable_paths"]) != set(indexed_paths):
            raise ValueError(f"Index writable_paths drift: {task_id}")
        if any(any(char in path for char in "*?[") for path in direct["writable_paths"]):
            raise ValueError(f"wildcard writable scope requires explicit overlap support: {task_id}")
        gate_checks = direct["gate_check_ids"]
        if not gate_checks:
            raise ValueError(f"{task_id} has no gate check")
        traceability = traceability_by_task.get(task_id, [])
        tasks[task_id] = {
            "key": row["key"],
            "title": row["title"],
            "purpose": row["purpose"],
            "wave": row["wave"],
            "dependencies": dependencies,
            "verification_dependencies": [
                {
                    "task_id": dependency,
                    "source": "task.toml.dependencies",
                    "receipt": "accepted-ancestry-bound-dependency-receipt",
                }
                for dependency in dependencies
            ],
            "direct_contract": {
                "path": direct["path"],
                "declared_task_id": direct["declared_task_id"],
                "check_count": direct["check_count"],
                "check_ids": direct["check_ids"],
                "gate_check_ids": gate_checks,
            },
            "contract_paths": [contract["path"] for contract in contracts_by_task[task_id]],
            "direct_check_count": direct["check_count"],
            "recursive_check_count": sum(
                contract["check_count"] for contract in contracts_by_task[task_id]
            ),
            "file_ownership": {
                "writable_paths": direct["writable_paths"],
                "forbidden_paths": direct["forbidden_paths"],
                "scope_source": direct["path"],
            },
            "shared_interface_count": sum(
                task_id in interface["tasks"] for interface in shared_interfaces
            ),
            "risk_parity_surfaces": {
                "traceability_rows": len(traceability),
                "source_namespaces": sorted({row["source_namespace"] for row in traceability}),
                "source_ids": sorted({row["source_id"] for row in traceability}),
                "roles": sorted({row["role"] for row in traceability}),
            },
            "acceptance_join": {
                "requires_all_dependencies": True,
                "required_dependencies": dependencies,
                "gate_check_ids": gate_checks,
                "requires_own_receipt_to_start": False,
                "requires_final_product_success_to_start": False,
            },
        }

    ancestors, depth, longest, path_count = calculate_graph_facts(tasks)
    ids = sorted(tasks, key=task_sort)
    conflicts: list[dict[str, Any]] = []
    locks: list[dict[str, Any]] = []
    for offset, left in enumerate(ids):
        for right in ids[offset + 1 :]:
            left_paths = tasks[left]["file_ownership"]["writable_paths"]
            right_paths = tasks[right]["file_ownership"]["writable_paths"]
            overlapping = [
                {"left": left_path, "right": right_path}
                for left_path in left_paths
                for right_path in right_paths
                if path_overlaps(left_path, right_path)
            ]
            if not overlapping:
                continue
            if right in ancestors[left]:
                order = [right, left]
                relation = "transitive-dependency"
            elif left in ancestors[right]:
                order = [left, right]
                relation = "transitive-dependency"
            else:
                order = []
                relation = "parallel-lock"
            conflicts.append(
                {
                    "tasks": [left, right],
                    "overlap_count": len(overlapping),
                    "ordering": order,
                    "relation": relation,
                    "parallel_allowed": not order,
                }
            )
            if not order:
                locks.append(
                    {
                        "tasks": [left, right],
                        "overlap_count": len(overlapping),
                        "rule": "Do not dispatch concurrently; integrate the lower task ID first and reverify the later candidate.",
                    }
                )

    migration_edges: dict[tuple[str, str], list[dict[str, str]]] = defaultdict(list)
    for task_id in ids:
        for dependency in tasks[task_id]["dependencies"]:
            from_wave = tasks[dependency]["wave"]
            to_wave = tasks[task_id]["wave"]
            if from_wave != to_wave:
                migration_edges[(from_wave, to_wave)].append(
                    {"from": dependency, "to": task_id}
                )
    wave_position = {wave: position for position, wave in enumerate(wave_order)}
    migration_boundaries = [
        {
            "from_wave": from_wave,
            "to_wave": to_wave,
            "source": "task-index.tsv:wave plus task.toml.dependencies",
            "edges": sorted(
                edges,
                key=lambda edge: (task_sort(edge["from"]), task_sort(edge["to"])),
            ),
        }
        for (from_wave, to_wave), edges in sorted(
            migration_edges.items(),
            key=lambda item: (wave_position[item[0][0]], wave_position[item[0][1]]),
        )
    ]

    maximum_depth = max(depth.values())
    deepest_tasks = sorted(
        [task_id for task_id in ids if depth[task_id] == maximum_depth],
        key=task_sort,
    )
    representative_path = [deepest_tasks[0]]
    while longest[representative_path[-1]]:
        representative_path.append(longest[representative_path[-1]][0])
    representative_path.reverse()

    parallel_waves = []
    for level in range(1, maximum_depth + 1):
        wave_tasks = [task_id for task_id in ids if depth[task_id] == level]
        same_depth_conflicts = [
            conflict["tasks"]
            for conflict in conflicts
            if conflict["tasks"][0] in wave_tasks and conflict["tasks"][1] in wave_tasks
        ]
        parallel_waves.append(
            {
                "dependency_depth": level,
                "tasks": wave_tasks,
                "workstreams": [
                    wave
                    for wave in wave_order
                    if any(tasks[task_id]["wave"] == wave for task_id in wave_tasks)
                ],
                "candidate_parallel": True,
                "file_conflicts": same_depth_conflicts,
                "requires": [
                    "all verification_dependencies have accepted ancestry-bound receipts",
                    "each candidate uses an isolated worktree and run directory",
                    "parallel candidates are independently verified before the join",
                ],
            }
        )

    checkpoints = []
    for task_id in ids:
        suffix = tasks[task_id]["key"].split("-", 1)[1]
        if suffix in CHECKPOINT_SUFFIXES:
            checkpoints.append(
                {
                    "task_id": task_id,
                    "key": tasks[task_id]["key"],
                    "kind": "named-catalog-checkpoint",
                    "selector": f"task-index.tsv key suffix {suffix}",
                    "dependency_depth": depth[task_id],
                    "gate_check_ids": tasks[task_id]["acceptance_join"]["gate_check_ids"],
                    "requires_all_dependencies": True,
                }
            )

    rollback_boundaries = [
        {
            "task_id": task_id,
            "base_dependencies": tasks[task_id]["dependencies"],
            "scope_source": tasks[task_id]["file_ownership"]["scope_source"],
            "recovery": "Discard an unintegrated candidate and retry from the accepted parent; never recover by widening writable scope or mutating forbidden paths.",
        }
        for task_id in ids
    ]

    graph = {
        "schema": GRAPH_SCHEMA,
        "contract_rules": {
            "source_files": [
                INDEX_REL.as_posix(),
                "refactoring-tasks/terminal-components/completion/**/task.toml",
                "refactoring-tasks/terminal-components/completion/**/verify.toml",
                TRACEABILITY_REL.as_posix(),
            ],
            "task_identity_preserved": True,
            "catalog_counts_are_derived_and_fail_closed": True,
            "start_gate": {
                "requires_all_verification_dependencies": True,
                "requires_own_receipt": False,
                "requires_final_product_success": False,
            },
            "acceptance_join": "A task joins only after every declared dependency has an accepted ancestry-bound receipt; its own receipt is produced after its checks and is never a prerequisite for itself.",
            "parallelization": "Tasks at equal dependency depth are only candidates; writable-scope conflicts serialize, and every parallel join is reverified on the combined parent.",
            "integration": "Compare-and-swap the expected parent and reverify after integration; graph metadata never authorizes dispatch.",
        },
        "catalog": {
            "direct_task_packages": len(tasks),
            "recursive_verify_toml": sum(len(contracts) for contracts in contracts_by_task.values()),
            "direct_checks": sum(task["direct_check_count"] for task in tasks.values()),
            "recursive_checks": sum(task["recursive_check_count"] for task in tasks.values()),
            "dependency_edges": sum(len(task["dependencies"]) for task in tasks.values()),
            "nested_bootstrap_contracts": nested_contracts,
            "traceability": traceability_catalog,
        },
        "contract_inventory": [
            compact_contract(contract)
            for task_id in sorted(contracts_by_task, key=task_sort)
            for contract in contracts_by_task[task_id]
        ],
        "tasks": tasks,
        "dependency_depth": depth,
        "longest_path_predecessors": longest,
        "longest_path_count": sum(path_count[task_id] for task_id in deepest_tasks),
        "maximum_dependency_depth": maximum_depth,
        "deepest_tasks": deepest_tasks,
        "representative_critical_path": representative_path,
        "shared_interfaces": shared_interfaces,
        "file_conflicts": conflicts,
        "serialization_locks": locks,
        "migration_boundaries": migration_boundaries,
        "parallel_waves": parallel_waves,
        "integration_checkpoints": checkpoints,
        "rollback_recovery_boundaries": rollback_boundaries,
    }
    validate_graph(
        graph,
        root,
        index,
        contracts_by_task,
        traceability_by_task,
        shared_interfaces,
        traceability_catalog,
        wave_order,
    )
    return graph


def validate_graph(
    graph: dict[str, Any],
    root: Path,
    index: dict[str, dict[str, str]],
    contracts_by_task: dict[str, list[dict[str, Any]]],
    traceability_by_task: dict[str, list[dict[str, str]]],
    shared_interfaces: list[dict[str, Any]],
    traceability_catalog: dict[str, Any],
    wave_order: list[str],
) -> None:
    if graph.get("schema") != GRAPH_SCHEMA:
        raise ValueError("graph schema drift")
    tasks = graph.get("tasks")
    if not isinstance(tasks, dict) or set(tasks) != set(index):
        raise ValueError("graph task identity set drift")
    if graph["catalog"]["direct_task_packages"] != DIRECT_TASK_COUNT:
        raise ValueError("direct task count drift")
    if graph["catalog"]["recursive_verify_toml"] != RECURSIVE_VERIFY_COUNT:
        raise ValueError("recursive verify.toml count drift")
    if graph["catalog"]["direct_checks"] != DIRECT_CHECK_COUNT:
        raise ValueError("direct check count drift")
    if graph["catalog"]["recursive_checks"] != RECURSIVE_CHECK_COUNT:
        raise ValueError("recursive check count drift")
    if graph["catalog"]["dependency_edges"] != DEPENDENCY_EDGE_COUNT:
        raise ValueError("dependency edge count drift")
    if len(graph["catalog"]["nested_bootstrap_contracts"]) != 4:
        raise ValueError("nested bootstrap count drift")
    if graph["contract_rules"]["start_gate"] != {
        "requires_all_verification_dependencies": True,
        "requires_own_receipt": False,
        "requires_final_product_success": False,
    }:
        raise ValueError("start gate cycle rule drift")
    if graph["catalog"]["traceability"] != traceability_catalog:
        raise ValueError("traceability catalog drift")
    if graph["contract_inventory"] != [
        compact_contract(contract)
        for task_id in sorted(contracts_by_task, key=task_sort)
        for contract in contracts_by_task[task_id]
    ]:
        raise ValueError("verification contract inventory drift")
    for task_id, task in tasks.items():
        metadata = tomllib.loads(
            (root / COMPLETION_REL / task_id[5:] / "task.toml").read_text(encoding="utf-8")
        )
        if task["dependencies"] != dependency_ids(metadata, task_id):
            raise ValueError(f"dependency order drift: {task_id}")
        dependency_set = set(task["dependencies"])
        if set(item["task_id"] for item in task["verification_dependencies"]) != dependency_set:
            raise ValueError(f"verification dependency drift: {task_id}")
        if set(task["acceptance_join"]["required_dependencies"]) != dependency_set:
            raise ValueError(f"acceptance join drift: {task_id}")
        if task["acceptance_join"]["requires_own_receipt_to_start"]:
            raise ValueError(f"self-receipt start cycle: {task_id}")
        if task["acceptance_join"]["requires_final_product_success_to_start"]:
            raise ValueError(f"final-product start cycle: {task_id}")
        direct = contracts_by_task[task_id][0]
        if task["direct_contract"]["path"] != direct["path"]:
            raise ValueError(f"direct contract path drift: {task_id}")
        if task["direct_check_count"] != direct["check_count"]:
            raise ValueError(f"direct check count drift: {task_id}")
        if task["direct_contract"]["check_ids"] != direct["check_ids"]:
            raise ValueError(f"direct check identity drift: {task_id}")
        if task["direct_contract"]["gate_check_ids"] != direct["gate_check_ids"]:
            raise ValueError(f"direct gate identity drift: {task_id}")
        if task["contract_paths"] != [contract["path"] for contract in contracts_by_task[task_id]]:
            raise ValueError(f"contract path drift: {task_id}")
        if task["recursive_check_count"] != sum(
            contract["check_count"] for contract in contracts_by_task[task_id]
        ):
            raise ValueError(f"recursive check count drift: {task_id}")
        if task["file_ownership"]["writable_paths"] != direct["writable_paths"]:
            raise ValueError(f"file ownership drift: {task_id}")
        if task["risk_parity_surfaces"]["traceability_rows"] != len(
            traceability_by_task.get(task_id, [])
        ):
            raise ValueError(f"traceability count drift: {task_id}")
        gate_ids = set(direct["gate_check_ids"])
        if set(task["acceptance_join"]["gate_check_ids"]) != gate_ids:
            raise ValueError(f"gate check drift: {task_id}")

    ancestors, depth, longest, path_count = calculate_graph_facts(tasks)
    if graph["dependency_depth"] != depth:
        raise ValueError("dependency depth drift")
    if graph["longest_path_predecessors"] != longest:
        raise ValueError("longest predecessor drift")
    maximum_depth = max(depth.values())
    deepest = sorted(
        [task_id for task_id, value in depth.items() if value == maximum_depth],
        key=task_sort,
    )
    if graph["deepest_tasks"] != deepest:
        raise ValueError("deepest task drift")
    if graph["longest_path_count"] != sum(path_count[task_id] for task_id in deepest):
        raise ValueError("longest path count drift")
    if graph["catalog"]["dependency_edges"] != sum(
        len(task["dependencies"]) for task in tasks.values()
    ):
        raise ValueError("dependency edge total drift")

    if graph["shared_interfaces"] != shared_interfaces:
        raise ValueError("shared interface derivation drift")
    for task_id, task in tasks.items():
        expected_count = sum(task_id in interface["tasks"] for interface in shared_interfaces)
        if task["shared_interface_count"] != expected_count:
            raise ValueError(f"shared interface count drift: {task_id}")

    for conflict in graph["file_conflicts"]:
        left, right = conflict["tasks"]
        if left not in tasks or right not in tasks or left >= right:
            raise ValueError(f"invalid file conflict task pair: {left}/{right}")
        expected_overlap_count = sum(
            path_overlaps(left_path, right_path)
            for left_path in tasks[left]["file_ownership"]["writable_paths"]
            for right_path in tasks[right]["file_ownership"]["writable_paths"]
        )
        if conflict["overlap_count"] != expected_overlap_count:
            raise ValueError(f"file conflict overlap drift: {left}/{right}")
        if right in ancestors[left]:
            expected_order = [right, left]
        elif left in ancestors[right]:
            expected_order = [left, right]
        else:
            expected_order = []
        if conflict["ordering"] != expected_order:
            raise ValueError(f"file conflict ordering drift: {left}/{right}")
        if conflict["parallel_allowed"] != (not expected_order):
            raise ValueError(f"file conflict parallel rule drift: {left}/{right}")
        if conflict["overlap_count"] < 1:
            raise ValueError(f"empty file conflict: {left}/{right}")
        if conflict["parallel_allowed"] and conflict["relation"] != "parallel-lock":
            raise ValueError(f"parallel conflict relation drift: {left}/{right}")
        if not conflict["parallel_allowed"] and not conflict["ordering"]:
            raise ValueError(f"ordered conflict lacks ordering: {left}/{right}")
    expected_locks = [
        {
            "tasks": conflict["tasks"],
            "overlap_count": conflict["overlap_count"],
            "rule": "Do not dispatch concurrently; integrate the lower task ID first and reverify the later candidate.",
        }
        for conflict in graph["file_conflicts"]
        if conflict["parallel_allowed"]
    ]
    if graph["serialization_locks"] != expected_locks:
        raise ValueError("serialization lock derivation drift")
    if [wave["dependency_depth"] for wave in graph["parallel_waves"]] != list(
        range(1, maximum_depth + 1)
    ):
        raise ValueError("parallel wave depth set drift")
    for wave in graph["parallel_waves"]:
        for task_id in wave["tasks"]:
            if depth[task_id] != wave["dependency_depth"]:
                raise ValueError(f"parallel wave depth drift: {task_id}")
        if wave["file_conflicts"]:
            raise ValueError(f"same-depth writable conflict: {wave['dependency_depth']}")
    if sorted(
        task_id for wave in graph["parallel_waves"] for task_id in wave["tasks"]
    ) != sorted(tasks):
        raise ValueError("parallel waves do not partition task identities")
    for boundary in graph["migration_boundaries"]:
        if boundary["from_wave"] not in wave_order or boundary["to_wave"] not in wave_order:
            raise ValueError("migration boundary references unknown wave")
        for edge in boundary["edges"]:
            if tasks[edge["from"]]["wave"] != boundary["from_wave"]:
                raise ValueError("migration source wave drift")
            if tasks[edge["to"]]["wave"] != boundary["to_wave"]:
                raise ValueError("migration target wave drift")
            if edge["from"] not in tasks[edge["to"]]["dependencies"]:
                raise ValueError("migration edge is not a dependency")

    expected_checkpoint_ids = {
        task_id
        for task_id, task in tasks.items()
        if task["key"].split("-", 1)[1] in CHECKPOINT_SUFFIXES
    }
    if {checkpoint["task_id"] for checkpoint in graph["integration_checkpoints"]} != expected_checkpoint_ids:
        raise ValueError("integration checkpoint selector drift")
    if len(graph["rollback_recovery_boundaries"]) != len(tasks):
        raise ValueError("rollback boundary count drift")


def render_markdown(graph: dict[str, Any]) -> str:
    tasks = graph["tasks"]
    lines = [
        "# Reconciled execution graph",
        "",
        "This graph is generated from the canonical task index, every direct and nested task contract, and traceability. It is structural planning data only: it contains no task status, acceptance result, ledger mutation, or dispatch authorization.",
        "",
        f"The graph contains {len(tasks)} direct tasks, {graph['catalog']['recursive_verify_toml']} recursive verify.toml contracts, {graph['catalog']['direct_checks']} direct checks, {graph['catalog']['recursive_checks']} recursive checks, and {graph['catalog']['dependency_edges']} dependency edges.",
        "",
        f"The maximum dependency depth is {graph['maximum_dependency_depth']} with {graph['longest_path_count']} equally deepest dependency paths. Complete machine-checkable metadata is in task-graph.json.",
        "",
        "## Catalog audit",
        "",
        "| Measure | Derived count | Authority |",
        "| --- | ---: | --- |",
        f"| Direct task packages | {graph['catalog']['direct_task_packages']} | task-index.tsv and direct package directories |",
        f"| Recursive verify.toml contracts | {graph['catalog']['recursive_verify_toml']} | all package descendants |",
        f"| Direct checks | {graph['catalog']['direct_checks']} | direct contracts |",
        f"| Recursive checks | {graph['catalog']['recursive_checks']} | direct plus nested contracts |",
        f"| Dependency edges | {graph['catalog']['dependency_edges']} | task.toml dependencies |",
        f"| Shared traceability interfaces | {len(graph['shared_interfaces'])} | traceability.tsv source/task membership |",
        f"| Writable-scope conflict pairs | {len(graph['file_conflicts'])} | direct verify.toml writable paths |",
        f"| Incomparable writable-scope locks | {len(graph['serialization_locks'])} | conflicts without a dependency ordering |",
        "",
        "The four nested contracts are retained as catalog members, not extra dispatchable tasks:",
        "",
        "| Owner | Declared nested task | Checks | Contract |",
        "| --- | --- | ---: | --- |",
    ]
    for contract in graph["catalog"]["nested_bootstrap_contracts"]:
        lines.append(
            f"| {contract['owner_task_id']} | {contract['declared_task_id']} | {contract['check_count']} | {contract['path']} |"
        )
    lines += [
        "",
        "## Execution contract",
        "",
        "- Every canonical dependency is also a verification dependency requiring an accepted, ancestry-bound dependency receipt.",
        "- Acceptance joins require all declared dependencies and the direct contract gate check.",
        "- A task's own receipt is never required to start that task; final-product success is never a prerequisite for the first implementation task.",
        "- Equal-depth tasks are only parallel candidates. File conflicts, isolated worktrees/run directories, independent verification, and fresh join verification still apply.",
        "- Migration boundaries are derived from dependency edges whose canonical task-index workstream changes. File ownership and rollback scopes are taken directly from each direct verify.toml.",
        "",
        "## One exact longest dependency path",
        "",
        " -> ".join(graph["representative_critical_path"]),
        "",
        "## Dependency layers and parallel candidates",
        "",
        "Tasks in one layer have the same dependency depth. They are not automatically safe to dispatch together.",
        "",
        "| Depth | Tasks | Workstreams | Same-depth conflicts |",
        "| ---: | --- | --- | ---: |",
    ]
    for wave in graph["parallel_waves"]:
        lines.append(
            f"| {wave['dependency_depth']} | "
            + ", ".join(wave["tasks"])
            + f" | {', '.join(wave['workstreams'])} | {len(wave['file_conflicts'])} |"
        )
    lines += [
        "",
        "## Migration boundaries",
        "",
        "| From workstream | To workstream | Dependency edges |",
        "| --- | --- | ---: |",
    ]
    for boundary in graph["migration_boundaries"]:
        lines.append(
            f"| {boundary['from_wave']} | {boundary['to_wave']} | {len(boundary['edges'])} |"
        )
    lines += [
        "",
        "## Integration checkpoints",
        "",
        "These checkpoints are selected only from named task-index key suffixes: COMPONENTS, CONFORMANCE, CLOSE, OWNERSHIP, TESTS, PERF, API, and FINAL. Their checks remain task-owned.",
        "",
        "| Task | Key | Depth | Gate checks |",
        "| --- | --- | ---: | --- |",
    ]
    for checkpoint in graph["integration_checkpoints"]:
        lines.append(
            f"| {checkpoint['task_id']} | {checkpoint['key']} | {checkpoint['dependency_depth']} | "
            + ", ".join(checkpoint["gate_check_ids"])
            + " |"
        )
    lines += [
        "",
        "## File ownership and recovery",
        "",
        "Each task's writable and forbidden paths, direct contract, shared traceability interfaces, risk/parity source IDs, acceptance join, recursive contracts, and rollback boundary are machine-readable per task in task-graph.json. A failed candidate is recovered by discarding the unintegrated candidate and retrying from its accepted parent; widening scope or mutating forbidden paths is not a recovery path.",
        "",
        "## Shared interfaces and conflicts",
        "",
        f"traceability.tsv yields {len(graph['shared_interfaces'])} shared source/task interfaces. It also yields {len(graph['file_conflicts'])} writable-scope conflict pairs; all currently have a transitive dependency ordering, so no incomparable pair is dispatchable concurrently. The full path pairs and ordering proof are in file_conflicts and serialization_locks.",
        "",
        "At every parallel join, materialize a fresh combined tree and rerun the union of impacted contracts, complete test accounting, and workspace gates. Individually accepted siblings are not proof of the combined result. Compare-and-swap integration rejects a changed parent; it never silently attaches a tested tree to a different parent.",
    ]
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[3])
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    root = args.root.resolve()
    docs = root / "docs/refactoring-plan"
    graph = build_graph(root)
    rendered = render_markdown(graph)
    graph_path = docs / "task-graph.json"
    markdown_path = docs / "task-graph.md"
    if args.write:
        graph_path.write_text(json.dumps(graph, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        markdown_path.write_text(rendered, encoding="utf-8")
    else:
        if json.loads(graph_path.read_text(encoding="utf-8")) != graph:
            raise ValueError("Stored machine graph is stale; regenerate with --write")
        if markdown_path.read_text(encoding="utf-8") != rendered:
            raise ValueError("Stored readable graph is stale; regenerate with --write")
    print(
        json.dumps(
            {
                "tasks": graph["catalog"]["direct_task_packages"],
                "recursive_verify_toml": graph["catalog"]["recursive_verify_toml"],
                "direct_checks": graph["catalog"]["direct_checks"],
                "recursive_checks": graph["catalog"]["recursive_checks"],
                "dependency_edges": graph["catalog"]["dependency_edges"],
                "maximum_dependency_depth": graph["maximum_dependency_depth"],
                "longest_path_count": graph["longest_path_count"],
                "shared_interfaces": len(graph["shared_interfaces"]),
                "file_conflict_pairs": len(graph["file_conflicts"]),
                "serialization_pairs": len(graph["serialization_locks"]),
                "written": args.write,
            },
            indent=2,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
