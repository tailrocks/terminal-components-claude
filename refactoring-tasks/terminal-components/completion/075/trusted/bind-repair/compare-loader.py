#!/usr/bin/env python3
"""TASK-075 trusted compare-branch loader. This is not tc-proof.

Executes the exact candidate `tools/refactor-proof/runner/__main__.py`
source bytes in place (never the frozen dispatcher bundle) with
`compare --context <path>` and no `TC_PROOF_*` environment preset, then
attests that the branch returned to its caller. With the old `execv`
branch this process image is replaced and no attestation is written; with
the fixed subprocess branch `main()` returns and the attestation records
the outcome. Standard library only.
"""

from __future__ import annotations

import argparse
import importlib.util
import json
import os
import sys
import traceback
from pathlib import Path

SUPERVISION_SCHEMA = "tc-proof-bind-repair-supervision/v1"


def load_source_main(worktree):
    # type: (str) -> tuple
    """Import candidate `runner/__main__.py` as a synthetic package module.

    Returns `(main_callable, loaded_path)`. Relative imports resolve inside
    the candidate tree: `tcrepair075` roots at `tools/refactor-proof`, so
    `..accounting` and `.context` load the candidate's real modules.
    """
    root = Path(worktree) / "tools" / "refactor-proof"
    target = root / "runner" / "__main__.py"
    if target.is_symlink() or not target.is_file():
        raise RuntimeError("candidate source branch is not a regular file: %s"
                           % target)

    def package(name, path):
        # type: (...) -> object
        spec = importlib.util.spec_from_loader(name, loader=None)
        module = importlib.util.module_from_spec(spec)
        module.__path__ = [str(path)]
        sys.modules[name] = module
        return module

    package("tcrepair075", root)
    package("tcrepair075.runner", root / "runner")
    spec = importlib.util.spec_from_file_location(
        "tcrepair075.runner.__main__", str(target))
    if spec is None or spec.loader is None:
        raise RuntimeError("candidate source branch cannot be loaded")
    module = importlib.util.module_from_spec(spec)
    sys.modules["tcrepair075.runner.__main__"] = module
    spec.loader.exec_module(module)
    main = getattr(module, "main", None)
    if not callable(main):
        raise RuntimeError("candidate source branch has no main()")
    return main, str(target.resolve())


def parse_args(argv):
    # type: (list) -> argparse.Namespace
    parser = argparse.ArgumentParser(
        description="Run the candidate source compare branch once.")
    parser.add_argument("--worktree", required=True)
    parser.add_argument("--context", required=True)
    parser.add_argument("--sentinel", required=True)
    return parser.parse_args(argv)


def main(argv):
    # type: (list) -> int
    args = parse_args(argv)
    for key in [key for key in os.environ if key.startswith("TC_PROOF_")]:
        os.environ.pop(key)
    sentinel_path = Path(args.sentinel)
    if sentinel_path.exists() or sentinel_path.is_symlink():
        print("loader: sentinel already exists: %s" % sentinel_path,
              file=sys.stderr)
        return 2
    try:
        branch_main, loaded_path = load_source_main(args.worktree)
    except Exception as error:  # noqa: BLE001 - attest nothing on load failure
        traceback.print_exc()
        print("loader: source branch failed to load: %s" % error,
              file=sys.stderr)
        return 2
    try:
        exit_code = branch_main(["compare", "--context", args.context])
    except SystemExit as error:
        code = error.code
        exit_code = code if isinstance(code, int) else 1
    except Exception:  # noqa: BLE001 - a raising branch did not return cleanly
        traceback.print_exc()
        return 2
    if not isinstance(exit_code, int):
        print("loader: branch returned a non-integer exit code",
              file=sys.stderr)
        return 2
    sentinel = {
        "schema": SUPERVISION_SCHEMA,
        "check_id": Path(args.context).stem,
        "operation": "compare",
        "loaded_path": loaded_path,
        "returned": True,
        "exit_code": exit_code,
    }
    try:
        sentinel_path.write_bytes(
            json.dumps(sentinel, sort_keys=True, separators=(",", ":"))
            .encode("utf-8"))
    except OSError as error:
        print("loader: sentinel unwritable: %s" % error, file=sys.stderr)
        return 2
    return exit_code


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
