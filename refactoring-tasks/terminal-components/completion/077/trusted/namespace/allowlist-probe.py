#!/usr/bin/env python3
"""TASK-077 CHK-001: 'components' in the native oracle allowlist.

Fails on the current tree (the allowlist holds only the four app
namespaces in both the runner source and the tracked bundle); passes
once the source gains exactly the 'components' member and the bundle is
regenerated via the qualified architecture/rebundle.py generator. Also
requires the authoritative freshness check to exit 0. Runs from the
worktree root.
"""

from __future__ import annotations

import ast
import subprocess
import sys
from pathlib import Path

SOURCE = Path("tools/refactor-proof/runner/context.py")
BUNDLE = Path("tools/refactor-proof/bin/tc-proof")
FRESHNESS = Path("tools/refactor-proof/tests/rebundle_check.py")
ALLOWLIST_NAME = "NATIVE_ORACLE_NAMESPACES"
EXPECTED = frozenset({"showcase", "holla", "jackin", "tablepro", "components"})


def extract_allowlist(path: Path) -> tuple[frozenset | None, str]:
    """AST-extract the allowlist set; (None, reason) when not exact."""
    try:
        tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
    except OSError as error:
        return None, f"cannot read {path}: {error}"
    except SyntaxError as error:
        return None, f"{path} is not valid python: {error}"
    found: list[frozenset] = []
    for node in tree.body:
        if not isinstance(node, ast.Assign):
            continue
        if any(
            isinstance(target, ast.Name) and target.id == ALLOWLIST_NAME
            for target in node.targets
        ):
            value = node.value
            if (
                isinstance(value, ast.Call)
                and isinstance(value.func, ast.Name)
                and value.func.id == "frozenset"
                and len(value.args) == 1
                and isinstance(value.args[0], (ast.Set, ast.List, ast.Tuple))
                and all(
                    isinstance(elt, ast.Constant) and isinstance(elt.value, str)
                    for elt in value.args[0].elts
                )
            ):
                found.append(
                    frozenset(elt.value for elt in value.args[0].elts)
                )
            else:
                return None, f"{path} allowlist is not a frozenset string literal"
    if len(found) != 1:
        return None, f"{path} defines {ALLOWLIST_NAME} {len(found)} times, want 1"
    return found[0], ""


def main() -> int:
    failures: list[str] = []
    for path, label in ((SOURCE, "runner source"), (BUNDLE, "tracked bundle")):
        allowlist, reason = extract_allowlist(path)
        if allowlist is None:
            failures.append(reason)
            continue
        if allowlist != EXPECTED:
            missing = sorted(EXPECTED - allowlist)
            extra = sorted(allowlist - EXPECTED)
            detail = ""
            if missing:
                detail += f" missing={missing}"
            if extra:
                detail += f" extra={extra}"
            failures.append(f"{label} allowlist is not the exact five-member set:{detail}")
        else:
            print(f"PASS: {label} allowlist is exactly {sorted(EXPECTED)}")
    completed = subprocess.run(
        [sys.executable, str(FRESHNESS)],
        capture_output=True,
        text=True,
    )
    if completed.returncode != 0:
        output = (completed.stdout + completed.stderr).strip().splitlines()
        tail = output[-1] if output else "no output"
        failures.append(
            f"authoritative freshness check exited {completed.returncode}: {tail}"
        )
    else:
        print("PASS: authoritative freshness check exits 0")
    for failure in failures:
        print(f"FAIL: {failure}")
    if failures:
        return 1
    print("allowlist probe: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
