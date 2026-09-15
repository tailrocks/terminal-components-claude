#!/usr/bin/env python3
"""tc-proof runner operations entrypoint."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

from .operations import run_capture, run_close, run_oracle, run_preflight, run_required
from .result import finish

RUNNER_OPS = {"preflight", "required", "oracle", "capture", "close"}


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(prog="tc-proof")
    parser.add_argument("operation")
    parser.add_argument("--context", type=Path, required=True)
    parser.add_argument("--namespace")
    parser.add_argument("--lane")
    parser.add_argument("--approve", action="store_true")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv or sys.argv[1:])
    if args.approve:
        import os

        return finish(
            "rejected",
            "PROTOCOL",
            {},
            [],
            args.operation,
            os.environ.get("TC_PROOF_CONTEXT_SHA256", "0" * 64),
        )
    if args.operation == "compare":
        import os

        root = Path(__file__).resolve().parent.parent
        workspace = root.parent.parent
        rust = workspace / "target" / "debug" / "tc-proof"
        os.execv(str(rust), [str(rust), "compare", *[str(a) for a in sys.argv[2:]]])
    if args.operation not in RUNNER_OPS:
        print(f"tc-proof: unknown command: {args.operation}", file=sys.stderr)
        return 2
    dispatch = {
        "preflight": lambda: run_preflight(args.context),
        "required": lambda: run_required(args.context),
        "oracle": lambda: run_oracle(args.context, args.namespace),
        "capture": lambda: run_capture(args.context, args.lane),
        "close": lambda: run_close(args.context),
    }
    return dispatch[args.operation]()


if __name__ == "__main__":
    raise SystemExit(main())
