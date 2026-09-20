#!/usr/bin/env python3
"""Check that the tracked bundle is generated from the current Python sources."""

from __future__ import annotations

import ast
import importlib.util
from pathlib import Path


def main() -> None:
    root = Path(__file__).resolve().parents[1]
    architecture = root / "architecture/rebundle.py"
    bundle = root / "bin/tc-proof"
    entrypoint = root / "runner/__main__.py"
    spec = importlib.util.spec_from_file_location("tc_proof_rebundle", architecture)
    if spec is None or spec.loader is None:
        raise SystemExit("cannot load rebundle generator")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)

    source = module.strip_imports_and_docstring(entrypoint.read_text(encoding="utf-8"))
    bundled = bundle.read_text(encoding="utf-8")
    if "# ----- runner/__main__.py -----" not in bundled:
        raise SystemExit("bundle has no generated runner entrypoint")
    if source not in bundled:
        raise SystemExit("bundle runner entrypoint differs from source")
    if "import stat\n" not in bundled:
        raise SystemExit("bundle header is missing stat")
    sections = [
        (module.RUNNER, module.RUNNER_MODULES, lambda name: name),
        (module.ACCOUNTING, module.ACCOUNTING_MODULES, lambda name: f"accounting/{name}"),
        (module.ARCHITECTURE, module.ARCHITECTURE_MODULES, lambda name: f"architecture/{name}"),
        (module.RUNNER, ["__main__.py"], lambda name: "runner/__main__.py"),
    ]
    for directory, names, label in sections:
        for name in names:
            source_block = module.strip_imports_and_docstring(
                (directory / name).read_text(encoding="utf-8")
            ).strip()
            marker = f"# ----- {label(name)} -----\n"
            start = bundled.find(marker)
            if start < 0:
                raise SystemExit(f"bundle is missing generated section: {label(name)}")
            start += len(marker)
            end = bundled.find("\n# ----- ", start)
            actual_block = bundled[start:] if end < 0 else bundled[start:end]
            if actual_block.strip() != source_block:
                raise SystemExit(f"bundle section differs from source: {label(name)}")
    ast.parse(entrypoint.read_text(encoding="utf-8"), filename=str(entrypoint))
    compile(bundled, str(bundle), "exec")
    tree = ast.parse(bundled, filename=str(bundle))
    last_collect = None
    for node in tree.body:
        if isinstance(node, ast.FunctionDef) and node.name == "_collect_observations":
            last_collect = node
    if last_collect is None:
        raise SystemExit("bundle is missing runner _collect_observations")
    kwonly = {arg.arg for arg in last_collect.args.kwonlyargs}
    if "observer_sequence" not in kwonly:
        raise SystemExit(
            "bundle overlays runner _collect_observations with incompatible helper"
        )
    print("rebundle source/AST/compile check: PASS")


if __name__ == "__main__":
    main()
