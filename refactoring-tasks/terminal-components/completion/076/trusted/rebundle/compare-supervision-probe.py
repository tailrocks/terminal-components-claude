#!/usr/bin/env python3
"""TASK-076 CHK-002: the tracked bundle compare path supervises and emits.

Fails on the stale bundle (os.execv present, no run_compare); passes once
the bundle is regenerated from the fixed sources via the qualified
architecture/rebundle.py generator. Runs from the worktree root.
"""

from __future__ import annotations

import ast
from pathlib import Path

BUNDLE = Path("tools/refactor-proof/bin/tc-proof")
RUNNER_MARKER = "# ----- runner/__main__.py -----\n"


def main() -> int:
    failures: list[str] = []
    try:
        text = BUNDLE.read_text(encoding="utf-8")
    except OSError as error:
        print(f"FAIL: cannot read {BUNDLE}: {error}")
        return 1
    if "os.execv" in text:
        failures.append(
            "tracked bundle still calls os.execv "
            "(compare replaces the process and never emits a runner result)"
        )
    else:
        print("PASS: no os.execv in tracked bundle")
    start = text.find(RUNNER_MARKER)
    if start < 0:
        print("FAIL: bundle has no runner/__main__.py section")
        return 1
    section = text[start + len(RUNNER_MARKER):]
    end = section.find("\n# ----- ")
    runner = section if end < 0 else section[:end]
    if "run_compare" not in runner:
        failures.append("runner section lacks run_compare supervision")
    else:
        print("PASS: runner section carries run_compare")
    if "finish(" not in runner:
        failures.append("runner section never emits via finish(")
    else:
        print("PASS: runner section emits via finish(")
    try:
        tree = ast.parse(text, filename=str(BUNDLE))
    except SyntaxError as error:
        failures.append(f"bundle is not valid python: {error}")
        tree = None
    if tree is not None:
        names = {
            node.name for node in ast.walk(tree) if isinstance(node, ast.FunctionDef)
        }
        if "run_compare" not in names:
            failures.append("bundle AST has no run_compare definition")
        else:
            print("PASS: bundle AST defines run_compare")
    for failure in failures:
        print(f"FAIL: {failure}")
    if failures:
        return 1
    print("compare-supervision probe: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
