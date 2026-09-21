#!/usr/bin/env python3
"""TASK-077 CHK-003 gate: bundle executable, valid, regenerable byte-identical.

Regenerates into a temp file via the qualified
tools/refactor-proof/architecture/rebundle.py generator (never by hand),
requires byte-identical output to the tracked bundle, and requires the
staged generator product to carry the fixed five-member allowlist (so a
fresh-but-unfixed bundle fails this gate). Fails on the current tree;
passes once the source fix is regenerated. Runs from the worktree root.
"""

from __future__ import annotations

import ast
import hashlib
import importlib.util
import tempfile
from pathlib import Path

ROOT = Path("tools/refactor-proof")
BUNDLE = ROOT / "bin/tc-proof"
GENERATOR = ROOT / "architecture/rebundle.py"
ALLOWLIST_NAME = "NATIVE_ORACLE_NAMESPACES"
EXPECTED = frozenset({"showcase", "holla", "jackin", "tablepro", "components"})


def staged_allowlist(data: bytes) -> frozenset | None:
    try:
        tree = ast.parse(data.decode("utf-8"), filename="<staged tc-proof>")
    except (SyntaxError, ValueError, UnicodeDecodeError):
        return None
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
                return frozenset(elt.value for elt in value.args[0].elts)
            return None
    return None


def main() -> int:
    failures: list[str] = []
    if not BUNDLE.is_file():
        print(f"FAIL: {BUNDLE} is not a file")
        return 1
    mode = BUNDLE.stat().st_mode
    if (mode & 0o111) != 0o111:
        failures.append(
            f"bundle mode {oct(mode & 0o777)} lacks user/group/other exec bits"
        )
    else:
        print(f"PASS: bundle executable (mode {oct(mode & 0o777)})")
    try:
        text = BUNDLE.read_text(encoding="utf-8")
    except OSError as error:
        print(f"FAIL: cannot read {BUNDLE}: {error}")
        return 1
    try:
        ast.parse(text, filename=str(BUNDLE))
        compile(text, str(BUNDLE), "exec")
    except (SyntaxError, ValueError) as error:
        failures.append(f"bundle is not valid python: {error}")
    else:
        print("PASS: bundle parses and compiles")
    spec = importlib.util.spec_from_file_location(
        "tc_proof_rebundle_gate", GENERATOR
    )
    if spec is None or spec.loader is None:
        print(f"FAIL: cannot load qualified generator {GENERATOR}")
        return 1
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    with tempfile.TemporaryDirectory(prefix="tc077-rebundle-") as tmp:
        staged = Path(tmp) / "tc-proof"
        module.BIN = staged
        module.bundle()
        want = staged.read_bytes()
    have = BUNDLE.read_bytes()
    if want != have:
        failures.append(
            "regenerated bundle differs from tracked bundle "
            f"(staged sha256 {hashlib.sha256(want).hexdigest()[:16]} "
            f"tracked sha256 {hashlib.sha256(have).hexdigest()[:16]})"
        )
    else:
        print("PASS: qualified generator reproduces the tracked bundle byte-identical")
    staged_set = staged_allowlist(want)
    if staged_set != EXPECTED:
        failures.append(
            "staged generator product allowlist is "
            f"{sorted(staged_set) if staged_set is not None else staged_set}, "
            f"want {sorted(EXPECTED)}"
        )
    else:
        print("PASS: staged generator product carries the fixed allowlist")
    for failure in failures:
        print(f"FAIL: {failure}")
    if failures:
        return 1
    print("regeneration probe: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
