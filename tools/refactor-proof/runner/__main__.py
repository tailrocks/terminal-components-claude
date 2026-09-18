#!/usr/bin/env python3
"""tc-proof runner operations entrypoint."""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
from pathlib import Path

from ..accounting import run_account_tests
from ..architecture import run_architecture
from .context import load_context
from .index import load_index
from .operations import run_capture, run_close, run_oracle, run_preflight, run_required
from .result import finish

RUNNER_OPS = {"preflight", "required", "oracle", "capture", "account-tests", "architecture", "close"}


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(prog="tc-proof")
    parser.add_argument("operation")
    parser.add_argument("--context", type=Path, required=True)
    parser.add_argument("--namespace")
    parser.add_argument("--lane")
    parser.add_argument("--approve", action="store_true")
    return parser.parse_args(argv)


def _bind_environment(name: str, value: str) -> None:
    existing = os.environ.get(name)
    if existing is not None and existing != value:
        raise RuntimeError(f"native environment binding mismatch: {name}")
    os.environ[name] = value


def bind_native_environment(context_path: Path) -> None:
    """Bind the selected context to the native launch/index ABI."""
    context, _, context_hash = load_context(context_path)
    if context.get("schema") != "tc-proof-context/v1":
        raise RuntimeError("worker context is not tc-proof-context/v1")
    if not context_path.is_absolute():
        raise RuntimeError("worker context path is not absolute")
    check_id = context.get("check_id")
    if not isinstance(check_id, str) or context_path.name != f"{check_id}.json":
        raise RuntimeError("context filename/check identity mismatch")
    if context_path.is_symlink() or not context_path.is_file() or context_path.stat().st_nlink != 1:
        raise RuntimeError("context path is not a host-owned regular file")

    index_path = os.environ.get("TC_PROOF_CONTEXT_INDEX")
    if not index_path:
        raise RuntimeError("native context index is missing")
    index, _ = load_index()
    index_root = Path(index_path).resolve().parent
    if (
        index.get("schema") != "tc-proof-context-index/v1"
        or index.get("task_id") != context.get("task_id")
        or index.get("run_id") != context.get("run_id")
        or Path(str(index.get("run_id", ""))).resolve() != index_root
        or index.get("worktree_commit") != context.get("worktree_commit")
        or index.get("scope_base") != context.get("scope_base")
    ):
        raise RuntimeError("native context index identity mismatch")
    selected = next(
        (
            member
            for member in index.get("contexts", [])
            if isinstance(member, dict) and member.get("check_id") == check_id
        ),
        None,
    )
    if not isinstance(selected, dict):
        raise RuntimeError("selected context is not in native index")
    if Path(str(selected.get("path", ""))) != context_path:
        raise RuntimeError("selected context path is not host-bound")
    if selected.get("sha256") != context_hash:
        raise RuntimeError("selected context digest is not host-bound")

    qualification = context.get("qualification")
    common = qualification.get("common") if isinstance(qualification, dict) else None
    if not isinstance(common, dict):
        raise RuntimeError("trusted native common binding is missing")
    oracle = common.get("oracle")
    observer = common.get("observer")
    comparator = common.get("comparator")
    tool = common.get("tool")
    if not all(isinstance(value, dict) for value in (oracle, observer, comparator, tool)):
        raise RuntimeError("trusted native identity binding is incomplete")
    if context.get("tree") != common.get("candidate_tree"):
        raise RuntimeError("context source tree is not bound to trusted candidate tree")
    if context.get("oracle_commit") != oracle.get("commit"):
        raise RuntimeError("context oracle commit is not bound to trusted oracle")
    if context.get("oracle_tree") != oracle.get("tree"):
        raise RuntimeError("context oracle tree is not bound to trusted oracle")
    if context.get("tool") != tool:
        raise RuntimeError("context tool identity is not bound to trusted tool")
    if observer.get("transport") != "inherited-pipe/v1":
        raise RuntimeError("observer transport is not inherited-pipe/v1")
    if os.environ.get("TC_PROOF_NATIVE_CHILD") != "1":
        launcher = os.environ.get("TC_PROOF_NATIVE_LAUNCHER")
        if not launcher or Path(launcher) != Path(str(comparator.get("path", ""))):
            raise RuntimeError("native launcher is not the trusted comparator")

    _bind_environment("TC_PROOF_RUN_ID", str(context["run_id"]))
    _bind_environment("TC_PROOF_TASK_ID", str(context["task_id"]))
    _bind_environment("TC_PROOF_CHECK_ID", check_id)
    _bind_environment("TC_PROOF_CONTEXT_SHA256", context_hash)
    _bind_environment("TC_PROOF_SOURCE_TREE", str(common["candidate_tree"]))
    _bind_environment("TC_PROOF_ORACLE_COMMIT", str(oracle["commit"]))
    _bind_environment("TC_PROOF_OBSERVER_NONCE", str(observer["nonce"]))

    result_path = index_root / "outputs" / f"{check_id}.result.json"
    bound_result = os.environ.get("TC_PROOF_RESULT")
    if bound_result and Path(bound_result) != result_path:
        raise RuntimeError("result path was already bound differently")
    os.environ["TC_PROOF_RESULT"] = str(result_path)


def launch_native_worker(args: argparse.Namespace) -> int:
    """Re-enter this worker only through the trusted Rust launcher."""
    bind_native_environment(args.context)
    launcher = os.environ.get("TC_PROOF_NATIVE_LAUNCHER")
    index_path = os.environ.get("TC_PROOF_CONTEXT_INDEX")
    if not launcher or not index_path:
        raise RuntimeError("native launcher binding is missing")
    child_environment = os.environ.copy()
    child_environment["TC_PROOF_NATIVE_CHILD"] = "1"
    child_environment["TC_PROOF_NATIVE_LAUNCH"] = "0"
    timeout_ms = os.environ.get("TC_PROOF_NATIVE_TIMEOUT_MS", "600000")
    command = [
        launcher,
        "launch",
        "--run-dir",
        str(Path(index_path).resolve().parent),
        "--check-id",
        args.context.stem,
        "--timeout-ms",
        timeout_ms,
        "--",
        sys.executable,
        str(Path(__file__).resolve()),
        *sys.argv[1:],
    ]
    return subprocess.run(command, env=child_environment, check=False).returncode


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv or sys.argv[1:])
    if args.operation in RUNNER_OPS and os.environ.get("TC_PROOF_NATIVE_CHILD") != "1":
        if os.environ.get("TC_PROOF_NATIVE_LAUNCH") != "1":
            print("tc-proof: native launch binding is missing", file=sys.stderr)
            return 1
        try:
            return launch_native_worker(args)
        except (RuntimeError, ValueError, OSError, KeyError) as error:
            print(f"tc-proof: native launch rejected: {error}", file=sys.stderr)
            return 1
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
        "account-tests": lambda: run_account_tests(args.context),
        "architecture": lambda: run_architecture(args.context),
        "close": lambda: run_close(args.context),
    }
    return dispatch[args.operation]()


if __name__ == "__main__":
    raise SystemExit(main())
