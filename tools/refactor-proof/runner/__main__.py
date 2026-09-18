#!/usr/bin/env python3
"""tc-proof runner operations entrypoint."""

from __future__ import annotations

import argparse
import os
import sys
from pathlib import Path

from .context import load_context
from .index import load_index
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


def bind_native_environment(context_path: Path) -> None:
    """Bind the selected context to the native launch/index ABI."""
    context, _, context_hash = load_context(context_path)
    if context.get("schema") != "tc-proof-runner-context/v2":
        return
    check_id = context.get("check_id")
    if not isinstance(check_id, str) or context_path.name != f"{check_id}.json":
        raise RuntimeError("context filename/check identity mismatch")
    os.environ["TC_PROOF_CHECK_ID"] = check_id
    os.environ["TC_PROOF_CONTEXT_SHA256"] = context_hash
    os.environ.setdefault("TC_PROOF_RUN_ID", str(context["run_id"]))
    qualification = context.get("qualification") or {}
    common = qualification.get("common") or {}
    if isinstance(common, dict):
        oracle = common.get("oracle") or {}
        observer = common.get("observer") or {}
        os.environ.setdefault("TC_PROOF_SOURCE_TREE", str(context["tree"]))
        if isinstance(oracle, dict) and oracle.get("commit"):
            os.environ.setdefault("TC_PROOF_ORACLE_COMMIT", str(oracle["commit"]))
        if isinstance(observer, dict):
            if observer.get("nonce"):
                os.environ.setdefault("TC_PROOF_OBSERVER_NONCE", str(observer["nonce"]))
            if observer.get("socket"):
                os.environ.setdefault("TC_PROOF_OBSERVER_SOCKET", str(observer["socket"]))
    index_path = os.environ.get("TC_PROOF_CONTEXT_INDEX")
    if not index_path:
        return
    index, _ = load_index()
    selected = next((member for member in index["members"] if member.get("check_id") == check_id), None)
    if selected is None or Path(selected["context_path"]).resolve() != context_path.resolve():
        raise RuntimeError("context/index selection mismatch")
    result_path = Path(index_path).resolve().parent / selected["output_id"]
    bound_result = os.environ.get("TC_PROOF_RESULT")
    if bound_result and Path(bound_result).resolve() != result_path:
        raise RuntimeError("result path was already bound differently")
    os.environ["TC_PROOF_RESULT"] = str(result_path)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv or sys.argv[1:])
    if args.operation in RUNNER_OPS:
        try:
            bind_native_environment(args.context)
        except (RuntimeError, ValueError, OSError, KeyError):
            return finish(
                "rejected",
                "CONTEXT_INDEX",
                {},
                [],
                args.operation,
                os.environ.get("TC_PROOF_CONTEXT_SHA256", "0" * 64),
            )
    if args.approve:
        return finish(
            "rejected",
            "PROTOCOL",
            {},
            [],
            args.operation,
            os.environ.get("TC_PROOF_CONTEXT_SHA256", "0" * 64),
        )
    if args.operation == "compare":
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
