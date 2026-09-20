#!/usr/bin/env python3
"""Bundle tc-proof with qualification overlay (TASK-071 writable scope)."""

from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ACCOUNTING = ROOT / "accounting"
BIN = ROOT / "bin" / "tc-proof"
OVERLAY_MODULES = [
    "observer.py",
    "qualification.py",
]
OVERLAY_MAIN = '''
# ----- accounting/qualification-overlay -----

_native_runner_main = main
_validated_executable = qualification_validated_executable


def main(argv=None):
    args = parse_args(argv or sys.argv[1:])
    if args.approve:
        return finish(
            "rejected",
            "PROTOCOL",
            {},
            [],
            args.operation,
            os.environ.get("TC_PROOF_CONTEXT_SHA256", "0" * 64),
        )
    if args.operation in RUNNER_OPS and os.environ.get("TC_PROOF_NATIVE_HANDOFF_FD") is None:
        if os.environ.get("TC_PROOF_NATIVE_LAUNCH") == "1":
            return launch_native_worker(args)
        try:
            context, _, _ = load_context(args.context)
        except Reject as error:
            return finish(
                "rejected",
                error.category,
                {},
                [],
                args.operation,
                os.environ.get("TC_PROOF_CONTEXT_SHA256", "0" * 64),
            )
        if is_qualification_schema(context):
            return qualification_dispatch(args)
    return _native_runner_main(argv)


if __name__ == "__main__":
    raise SystemExit(main())
'''


def _architecture_rebundle():
    path = ROOT / "architecture" / "rebundle.py"
    spec = importlib.util.spec_from_file_location("tc_proof_architecture_rebundle", path)
    if spec is None or spec.loader is None:
        raise SystemExit("cannot load architecture rebundle generator")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def bundle() -> None:
    generator = _architecture_rebundle()
    generator.bundle()
    text = BIN.read_text(encoding="utf-8")
    marker = 'if __name__ == "__main__":\n    raise SystemExit(main())'
    index = text.rfind(marker)
    if index < 0:
        raise SystemExit("bundled dispatcher is missing the runner entrypoint")
    body = text[:index].rstrip() + "\n\n"
    parts = [body]
    for name in OVERLAY_MODULES:
        source = generator.strip_imports_and_docstring((ACCOUNTING / name).read_text(encoding="utf-8"))
        parts.append(f"# ----- accounting/{name} -----\n{source}\n\n")
    parts.append(OVERLAY_MAIN.lstrip("\n"))
    BIN.write_text("".join(parts).rstrip() + "\n", encoding="utf-8")
    BIN.chmod(0o755)


def main() -> None:
    bundle()


if __name__ == "__main__":
    main()
