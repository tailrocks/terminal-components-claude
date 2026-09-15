#!/usr/bin/env python3
"""Bundle tc-proof into bin/ without modifying runner/ (TASK-071+ writable scope)."""

from __future__ import annotations

import ast
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RUNNER = ROOT / "runner"
ACCOUNTING = ROOT / "accounting"
ARCHITECTURE = ROOT / "architecture"
BIN = ROOT / "bin" / "tc-proof"
RUNNER_MODULES = [
    "json_util.py",
    "observer.py",
    "result.py",
    "context.py",
    "validate.py",
    "index.py",
    "operations.py",
]
ACCOUNTING_MODULES = [
    "identity.py",
    "fixture.py",
    "extension.py",
    "dispatch.py",
]
ARCHITECTURE_MODULES = [
    "fixture.py",
    "extension.py",
    "rust_model.py",
    "actual.py",
    "source.py",
    "broker.py",
    "style_timing.py",
    "dispatch.py",
]


def strip_imports_and_docstring(source: str) -> str:
    mod = ast.parse(source)
    lines = source.splitlines()
    remove = set()
    if mod.body and isinstance(mod.body[0], ast.Expr) and isinstance(
        getattr(mod.body[0], "value", None), ast.Constant
    ):
        node = mod.body[0]
        for ln in range(node.lineno - 1, node.end_lineno):
            remove.add(ln)
    for node in mod.body:
        if isinstance(node, (ast.Import, ast.ImportFrom)):
            for ln in range(node.lineno - 1, node.end_lineno):
                remove.add(ln)
    return "\n".join(
        line
        for i, line in enumerate(lines)
        if i not in remove and not line.startswith("from __future__")
    )


def bundle(*, include_accounting: bool, include_architecture: bool) -> None:
    header = (
        "#!/usr/bin/env python3\n"
        '"""Self-contained tc-proof dispatcher for sandbox qualification."""\n'
        "from __future__ import annotations\n\n"
        "import argparse\n"
        "import hashlib\n"
        "import itertools\n"
        "import json\n"
        "import os\n"
        "import sys\n"
        "from pathlib import Path\n"
        "from typing import Any\n\n"
    )
    parts = [header]
    for name in RUNNER_MODULES:
        parts.append(
            f"# ----- {name} -----\n{strip_imports_and_docstring((RUNNER / name).read_text())}\n\n"
        )
    if include_accounting:
        for name in ACCOUNTING_MODULES:
            parts.append(
                f"# ----- accounting/{name} -----\n"
                f"{strip_imports_and_docstring((ACCOUNTING / name).read_text())}\n\n"
            )
    if include_architecture:
        for name in ARCHITECTURE_MODULES:
            parts.append(
                f"# ----- architecture/{name} -----\n"
                f"{strip_imports_and_docstring((ARCHITECTURE / name).read_text())}\n\n"
            )

    ops = '"preflight", "required", "oracle", "capture", "close"'
    dispatch = [
        '"preflight": lambda: run_preflight(args.context),',
        '"required": lambda: run_required(args.context),',
        '"oracle": lambda: run_oracle(args.context, args.namespace),',
        '"capture": lambda: run_capture(args.context, args.lane),',
        '"close": lambda: run_close(args.context),',
    ]
    if include_accounting:
        ops = '"preflight", "required", "oracle", "capture", "account-tests", "close"'
        dispatch.insert(-1, '"account-tests": lambda: run_account_tests(args.context),')
    if include_architecture:
        ops = '"preflight", "required", "oracle", "capture", "account-tests", "architecture", "close"'
        if '"account-tests"' not in ops:
            dispatch.insert(-1, '"account-tests": lambda: run_account_tests(args.context),')
        dispatch.insert(-1, '"architecture": lambda: run_architecture(args.context),')

    parts.append(
        f"""
def parse_args(argv):
    parser = argparse.ArgumentParser(prog="tc-proof")
    parser.add_argument("operation")
    parser.add_argument("--context", type=Path, required=True)
    parser.add_argument("--namespace")
    parser.add_argument("--lane")
    parser.add_argument("--approve", action="store_true")
    return parser.parse_args(argv)

def main(argv=None):
    args = parse_args(argv or sys.argv[1:])
    if args.approve:
        return finish("rejected", "PROTOCOL", {{}}, [], args.operation, os.environ.get("TC_PROOF_CONTEXT_SHA256", "0" * 64))
    if args.operation == "compare":
        root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        workspace = os.path.dirname(os.path.dirname(root))
        rust = os.path.join(workspace, "target", "debug", "tc-proof")
        os.execv(rust, [rust, "compare", *sys.argv[2:]])
    runner_ops = {{{ops}}}
    if args.operation not in runner_ops:
        print(f"tc-proof: unknown command: {{args.operation}}", file=sys.stderr)
        return 2
    dispatch = {{
        {chr(10).join("        " + line for line in dispatch)}
    }}
    return dispatch[args.operation]()

def _dispatch():
    return main()

if __name__ == "__main__":
    raise SystemExit(_dispatch())
"""
    )
    BIN.write_text("".join(parts))
    BIN.chmod(0o755)


def main() -> None:
    import argparse

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--accounting-only",
        action="store_true",
        help="Bundle runner + accounting only (no architecture modules)",
    )
    args = parser.parse_args()
    if args.accounting_only:
        bundle(include_accounting=True, include_architecture=False)
    else:
        include_arch = ARCHITECTURE.exists() and any(ARCHITECTURE.glob("*.py"))
        bundle(include_accounting=True, include_architecture=include_arch)


if __name__ == "__main__":
    main()
