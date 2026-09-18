#!/usr/bin/env python3
"""Full tc-proof bundle including architecture modules (TASK-072 writable scope)."""

from __future__ import annotations

import ast
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RUNNER = ROOT / "runner"
ACCOUNTING = ROOT / "accounting"
ARCHITECTURE = ROOT / "architecture"
BIN = ROOT / "bin" / "tc-proof"
ENTRYPOINT = RUNNER / "__main__.py"
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
ARCHITECTURE_HEADER_IMPORTS = (
    "import ast\n"
    "import base64\n"
    "import re\n"
    "import stat\n"
    "import subprocess\n"
)


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


def bundle() -> None:
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
        f"{ARCHITECTURE_HEADER_IMPORTS}\n"
    )
    parts = [header]
    for name in RUNNER_MODULES:
        content = strip_imports_and_docstring((RUNNER / name).read_text())
        parts.append(f"# ----- {name} -----\n{content}\n\n")
    for name in ACCOUNTING_MODULES:
        parts.append(
            f"# ----- accounting/{name} -----\n"
            f"{strip_imports_and_docstring((ACCOUNTING / name).read_text())}\n\n"
        )
    for name in ARCHITECTURE_MODULES:
        parts.append(
            f"# ----- architecture/{name} -----\n"
            f"{strip_imports_and_docstring((ARCHITECTURE / name).read_text())}\n\n"
        )
    parts.append(
        f"# ----- runner/__main__.py -----\n"
        f"{strip_imports_and_docstring(ENTRYPOINT.read_text())}\n\n"
    )
    BIN.write_text("".join(parts).rstrip() + "\n")
    BIN.chmod(0o755)


def main() -> None:
    bundle()


if __name__ == "__main__":
    main()
