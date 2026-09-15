#!/usr/bin/env python3
"""Generate trusted/check-context-templates for account-tests checks (WITNESS-01)."""
from __future__ import annotations

import json
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
COMPLETION = ROOT / "refactoring-tasks/terminal-components/completion"
CANONICAL = COMPLETION / "071/trusted/check-context-templates"

PREPARATION_TASKS = {f"TASK-{n:03d}" for n in range(2, 9)}
PREPARATION_SOURCE = {f"TASK-{n:03d}" for n in range(2, 8)}
POST_MIGRATION = {"TASK-008"}


def task_num(task_id: str) -> int:
    return int(task_id.split("-")[1])


def mode_for(task_id: str) -> str:
    return "preparation" if task_id in PREPARATION_TASKS else "production"


def register_kind_for(task_id: str) -> str:
    if task_id == "TASK-008":
        return "post-migration"
    if task_id in PREPARATION_SOURCE:
        return "source-derived"
    return "production"


def template_body(task_id: str, check_id: str) -> dict:
    mode = mode_for(task_id)
    return {
        "schema": "tc-proof-check-context-template/v1",
        "check_id": check_id,
        "operation": "account-tests",
        "qualification": {
            "mode": mode,
            "family": "accounting",
            "register_kind": register_kind_for(task_id),
            "requires_inventory_receipt": mode == "production",
            "requires_disposition_receipt": mode == "production",
        },
    }


def discover_account_checks() -> list[tuple[str, str]]:
    rows: list[tuple[str, str]] = []
    for verify_path in sorted(COMPLETION.glob("*/verify.toml")):
        data = tomllib.loads(verify_path.read_text())
        task_id = data.get("task_id", "")
        for check in data.get("checks", []):
            argv = check.get("argv", [])
            if "account-tests" in argv:
                rows.append((task_id, check["id"]))
    return sorted(set(rows))


def write_canonical_bases() -> None:
    CANONICAL.mkdir(parents=True, exist_ok=True)
    readme = CANONICAL / "README.md"
    readme.write_text(
        "# Accounting check context templates (TASK-071 authority)\n\n"
        "Frozen partial contexts merged by `tc-proof-host prepare` into "
        "`/run/tc-proof/contexts/CHK-NNN.json`. "
        "See `accounting-mode-bindings.tsv` and `proof-contract.md` §136–144.\n",
        encoding="utf-8",
    )
    bases = {
        "accounting-preparation-source.json": {
            "schema": "tc-proof-check-context-template/v1",
            "extends": None,
            "qualification": {
                "mode": "preparation",
                "family": "accounting",
                "register_kind": "source-derived",
                "requires_inventory_receipt": False,
                "requires_disposition_receipt": False,
            },
        },
        "accounting-preparation-post-migration.json": {
            "schema": "tc-proof-check-context-template/v1",
            "extends": None,
            "qualification": {
                "mode": "preparation",
                "family": "accounting",
                "register_kind": "post-migration",
                "requires_inventory_receipt": False,
                "requires_disposition_receipt": False,
            },
        },
        "accounting-production.json": {
            "schema": "tc-proof-check-context-template/v1",
            "extends": None,
            "qualification": {
                "mode": "production",
                "family": "accounting",
                "register_kind": "production",
                "requires_inventory_receipt": True,
                "requires_disposition_receipt": True,
            },
        },
    }
    for name, body in bases.items():
        (CANONICAL / name).write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")


def write_bindings(rows: list[tuple[str, str]]) -> None:
    lines = [
        "task_id\tcheck_id\tqualification_mode\tregister_kind\trequires_inventory_receipt\trequires_disposition_receipt"
    ]
    for task_id, check_id in rows:
        mode = mode_for(task_id)
        kind = register_kind_for(task_id)
        lines.append(
            f"{task_id}\t{check_id}\t{mode}\t{kind}\t{mode == 'production'}\t{mode == 'production'}"
        )
    (CANONICAL / "accounting-mode-bindings.tsv").write_text("\n".join(lines) + "\n", encoding="utf-8")


def cleanup_stale_templates(valid: set[tuple[str, str]]) -> None:
    for verify_path in sorted(COMPLETION.glob("*/verify.toml")):
        num = verify_path.parent.name
        dest_dir = verify_path.parent / "trusted/check-context-templates"
        if not dest_dir.is_dir():
            continue
        for path in dest_dir.glob("CHK-*.json"):
            task_id = tomllib.loads(verify_path.read_text()).get("task_id", "")
            check_id = path.stem
            if (task_id, check_id) not in valid:
                path.unlink()


def write_per_task_templates(rows: list[tuple[str, str]]) -> int:
    count = 0
    by_task: dict[str, list[str]] = {}
    for task_id, check_id in rows:
        by_task.setdefault(task_id, []).append(check_id)
    for task_id, check_ids in by_task.items():
        num = task_num(task_id)
        dest_dir = COMPLETION / f"{num:03d}/trusted/check-context-templates"
        dest_dir.mkdir(parents=True, exist_ok=True)
        for check_id in sorted(check_ids):
            path = dest_dir / f"{check_id}.json"
            path.write_text(json.dumps(template_body(task_id, check_id), indent=2) + "\n", encoding="utf-8")
            count += 1
    return count


def main() -> None:
    rows = discover_account_checks()
    cleanup_stale_templates(set(rows))
    write_canonical_bases()
    write_bindings(rows)
    n = write_per_task_templates(rows)
    print(f"Generated {n} accounting context templates across {len(set(t for t, _ in rows))} tasks")


if __name__ == "__main__":
    main()
