#!/usr/bin/env python3
"""Bundle runner modules into the sandbox-visible bin/tc-proof executable."""

from __future__ import annotations

import ast
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RUNNER = ROOT / "runner"
BIN = ROOT / "bin" / "tc-proof"
MODULES = [
    "json_util.py",
    "observer.py",
    "result.py",
    "context.py",
    "validate.py",
    "index.py",
    "operations.py",
]


def strip_imports_and_docstring(source: str) -> str:
    mod = ast.parse(source)
    lines = source.splitlines()
    remove = set()
    if mod.body and isinstance(mod.body[0], ast.Expr) and isinstance(getattr(mod.body[0], "value", None), ast.Constant):
        node = mod.body[0]
        for ln in range(node.lineno - 1, node.end_lineno):
            remove.add(ln)
    for node in mod.body:
        if isinstance(node, (ast.Import, ast.ImportFrom)):
            for ln in range(node.lineno - 1, node.end_lineno):
                remove.add(ln)
    return "\n".join(line for i, line in enumerate(lines) if i not in remove and not line.startswith("from __future__"))


def main() -> None:
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
    for name in MODULES:
        parts.append(f"# ----- {name} -----\n{strip_imports_and_docstring((RUNNER / name).read_text())}\n\n")
    main_src = strip_imports_and_docstring((RUNNER / "__main__.py").read_text())
    main_lines = []
    for line in main_src.splitlines():
        if line.startswith('if __name__ == "__main__"'):
            break
        main_lines.append(line)
    parts.append(f"# ----- __main__ -----\n" + "\n".join(main_lines) + "\n\n")
    parts.append(
        """

def _dispatch():
    import sys
    op = sys.argv[1] if len(sys.argv) > 1 else ""
    if op == "compare":
        import os
        root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        workspace = os.path.dirname(os.path.dirname(root))
        rust = os.path.join(workspace, "target", "debug", "tc-proof")
        os.execv(rust, [rust, "compare", *sys.argv[2:]])
    return main()

if __name__ == "__main__":
    raise SystemExit(_dispatch())
"""
    )
    BIN.write_text("".join(parts))
    BIN.chmod(0o755)


if __name__ == "__main__":
    main()
