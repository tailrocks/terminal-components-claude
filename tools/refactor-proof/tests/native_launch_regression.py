#!/usr/bin/env python3
"""Bounded proof-worker launch regression.

This fixture models the dispatcher environment without touching a task
contract, ledger, Git ref, or product source. It proves that the tracked
Python worker invokes the configured native launcher and that an invocation
without the native binding fails closed before direct dispatch.
"""

from __future__ import annotations

import hashlib
import json
import os
import stat
import subprocess
import sys
import tempfile
from pathlib import Path


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_json(path: Path, value: object) -> None:
    path.write_bytes(json.dumps(value, sort_keys=True, separators=(",", ":")).encode())


def main() -> None:
    worktree = Path(__file__).resolve().parents[3]
    bundle = worktree / "tools/refactor-proof/bin/tc-proof"
    if not bundle.is_file() or not os.access(bundle, os.X_OK):
        raise SystemExit(f"tracked proof bundle is not executable: {bundle}")

    with tempfile.TemporaryDirectory(prefix="tc-proof-native-launch-") as directory:
        root = Path(directory).resolve()
        run_dir = root / "run"
        contexts = run_dir / "contexts"
        outputs = run_dir / "outputs"
        contexts.mkdir(parents=True)
        outputs.mkdir()
        context_path = contexts / "CHK-001.json"
        index_path = run_dir / "context-index.json"
        shim = root / "native-launcher.py"
        capture = root / "launcher-argv.json"
        shim.write_text(
            "#!/usr/bin/env python3\n"
            "import json, os, sys\n"
            "from pathlib import Path\n"
            "Path(os.environ['TC_PROOF_LAUNCH_CAPTURE']).write_text(json.dumps(sys.argv[1:]))\n"
            "raise SystemExit(97)\n",
            encoding="utf-8",
        )
        shim.chmod(shim.stat().st_mode | stat.S_IXUSR)

        common = {
            "candidate_tree": "b" * 40,
            "oracle": {"commit": "a" * 40, "tree": "c" * 40},
            "tool": {"path": str(bundle), "sha256": digest(bundle)},
            "comparator": {"path": str(shim), "sha256": digest(shim)},
            "observer": {"transport": "inherited-pipe/v1", "nonce": "fixture-nonce"},
        }
        context = {
            "schema": "tc-proof-context/v1",
            "run_id": str(run_dir),
            "task_id": "TASK-001",
            "check_id": "CHK-001",
            "worktree_commit": "d" * 40,
            "scope_base": "e" * 40,
            "tree": common["candidate_tree"],
            "oracle_commit": common["oracle"]["commit"],
            "oracle_tree": common["oracle"]["tree"],
            "tool": common["tool"],
            "qualification": {"common": common},
        }
        write_json(context_path, context)
        index = {
            "schema": "tc-proof-context-index/v1",
            "task_id": context["task_id"],
            "run_id": context["run_id"],
            "worktree_commit": context["worktree_commit"],
            "scope_base": context["scope_base"],
            "contexts": [
                {
                    "check_id": context["check_id"],
                    "path": str(context_path),
                    "sha256": digest(context_path),
                }
            ],
        }
        write_json(index_path, index)
        environment = os.environ.copy()
        environment.update(
            {
                "TC_PROOF_CONTEXT_INDEX": str(index_path),
                "TC_PROOF_CONTEXT_INDEX_SHA256": digest(index_path),
                "TC_PROOF_NATIVE_LAUNCH": "1",
                "TC_PROOF_NATIVE_CHILD": "0",
                "TC_PROOF_NATIVE_LAUNCHER": str(shim),
                "TC_PROOF_NATIVE_TIMEOUT_MS": "5000",
                "TC_PROOF_LAUNCH_CAPTURE": str(capture),
            }
        )
        launched = subprocess.run(
            [
                str(bundle),
                "preflight",
                "--context",
                str(context_path),
            ],
            cwd=worktree,
            env=environment,
            capture_output=True,
            text=True,
            check=False,
        )
        if launched.returncode != 97:
            raise SystemExit(
                f"native launcher was not reached: exit={launched.returncode}\n{launched.stderr}"
            )
        argv = json.loads(capture.read_text(encoding="utf-8"))
        if argv[:3] != ["launch", "--run-dir", str(run_dir)] or "--" not in argv:
            raise SystemExit(f"unexpected native launch argv: {argv}")
        if str(bundle) not in argv:
            raise SystemExit(f"native child was not the tracked bundle: {argv}")

        bypass_environment = dict(environment)
        bypass_environment.pop("TC_PROOF_NATIVE_LAUNCH", None)
        bypass_environment.pop("TC_PROOF_NATIVE_LAUNCHER", None)
        capture.unlink()
        bypass = subprocess.run(
            [
                str(bundle),
                "preflight",
                "--context",
                str(context_path),
            ],
            cwd=worktree,
            env=bypass_environment,
            capture_output=True,
            text=True,
            check=False,
        )
        if bypass.returncode == 0 or "native launch binding is missing" not in bypass.stderr:
            raise SystemExit(
                f"direct Python dispatch was not rejected: exit={bypass.returncode}\n{bypass.stderr}"
            )
        if capture.exists():
            raise SystemExit("direct Python dispatch reached the launcher unexpectedly")

    print("native launch regression: PASS")


if __name__ == "__main__":
    main()
