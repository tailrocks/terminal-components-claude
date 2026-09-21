#!/usr/bin/env python3
"""TASK-076 CHK-003 gate: bundle executable, valid, regenerable byte-identical.

Regenerates into a temp file via the qualified
tools/refactor-proof/architecture/rebundle.py generator (never by hand) and
requires byte-identical output to the tracked bundle. Runs from the
worktree root.
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
    with tempfile.TemporaryDirectory(prefix="tc076-rebundle-") as tmp:
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
    for failure in failures:
        print(f"FAIL: {failure}")
    if failures:
        return 1
    print("regeneration probe: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
