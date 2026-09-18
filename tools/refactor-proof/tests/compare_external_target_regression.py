#!/usr/bin/env python3
"""Qualify compare through a verifier-owned external native binary."""

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


def run(
    command: list[str],
    *,
    cwd: Path,
    env: dict[str, str],
    timeout: float,
) -> subprocess.CompletedProcess[str]:
    try:
        return subprocess.run(
            command,
            cwd=cwd,
            env=env,
            capture_output=True,
            text=True,
            check=False,
            timeout=timeout,
        )
    except subprocess.TimeoutExpired as error:
        raise SystemExit(f"command timed out after {timeout}s: {command}") from error


def require_rejected(
    bundle: Path,
    context: Path,
    *,
    cwd: Path,
    env: dict[str, str],
    reason: str,
) -> None:
    result = run(
        [str(bundle), "compare", "--context", str(context)],
        cwd=cwd,
        env=env,
        timeout=15.0,
    )
    if result.returncode == 0 or "native comparator rejected" not in result.stderr:
        raise SystemExit(
            f"{reason} was accepted or failed for the wrong reason:\n"
            f"exit={result.returncode}\n{result.stderr}"
        )


def main() -> None:
    source = Path(__file__).resolve().parents[3]
    bundle = source / "tools/refactor-proof/bin/tc-proof"
    driver = source / "docs/refactoring-plan/evidence/proof-comparator-bootstrap.py"
    raw_native = os.environ.get("TC_PROOF_NATIVE_BINARY")
    if not raw_native:
        raise SystemExit("TC_PROOF_NATIVE_BINARY must name the external verifier binary")
    native = Path(raw_native)
    if not native.is_absolute() or native.is_symlink() or not native.is_file():
        raise SystemExit(f"invalid external verifier binary: {native}")
    if native.stat().st_nlink != 1 or not os.access(native, os.X_OK):
        raise SystemExit(f"external verifier binary is not a single-link executable: {native}")
    if any(parent.is_symlink() for parent in (native, *native.parents)):
        raise SystemExit(f"external verifier binary has a symlinked path component: {native}")
    if native == source or source in native.parents:
        raise SystemExit(f"external verifier binary is inside the candidate worktree: {native}")

    environment = os.environ.copy()
    environment.pop("TC_PROOF_NATIVE_LAUNCHER", None)
    environment["TC_PROOF_NATIVE_BINARY"] = str(native)
    qualification = run(
        [sys.executable, str(driver), "--runner", str(bundle), "--timeout", "20"],
        cwd=source,
        env=environment,
        timeout=180.0,
    )
    if qualification.returncode != 0:
        raise SystemExit(
            "external comparator qualification failed:\n"
            f"exit={qualification.returncode}\n{qualification.stdout}\n{qualification.stderr}"
        )
    try:
        report = json.loads(qualification.stdout)
    except json.JSONDecodeError as error:
        raise SystemExit(f"external comparator qualification was not JSON: {error}") from error
    if (
        report.get("case_count") != 72
        or report.get("invocation_count") != 141
        or report.get("failures") != []
    ):
        raise SystemExit(f"external comparator qualification was incomplete: {report}")

    with tempfile.TemporaryDirectory(prefix="tc-proof-compare-path-") as directory:
        root = Path(directory).resolve()
        direct_context = root / "direct-context.json"
        direct_context.write_text(
            json.dumps({"schema": "tc-proof-compare-context/v1"}) + "\n", encoding="utf-8"
        )

        invalid = dict(environment)
        invalid["TC_PROOF_NATIVE_BINARY"] = "relative/tc-proof"
        require_rejected(bundle, direct_context, cwd=source, env=invalid, reason="relative override")

        missing = dict(environment)
        missing["TC_PROOF_NATIVE_BINARY"] = str(root / "missing")
        require_rejected(bundle, direct_context, cwd=source, env=missing, reason="missing override")

        linked = root / "linked"
        linked.symlink_to(native)
        symlinked = dict(environment)
        symlinked["TC_PROOF_NATIVE_BINARY"] = str(linked)
        require_rejected(bundle, direct_context, cwd=source, env=symlinked, reason="symlink override")

        forged = root / "forged"
        forged.write_bytes(b"#!/bin/sh\nexit 0\n")
        forged.chmod(stat.S_IRUSR | stat.S_IWUSR | stat.S_IXUSR)
        runner_context = root / "runner-context.json"
        runner_context.write_text(
            json.dumps(
                {
                    "schema": "tc-proof-context/v1",
                    "qualification": {
                        "common": {
                            "worktree": str(source),
                            "comparator": {"path": str(native), "sha256": digest(native)},
                        }
                    },
                }
            )
            + "\n",
            encoding="utf-8",
        )
        mismatched = dict(environment)
        mismatched["TC_PROOF_NATIVE_BINARY"] = str(forged)
        require_rejected(
            bundle,
            runner_context,
            cwd=source,
            env=mismatched,
            reason="unbound comparator override",
        )

        ambiguous = dict(environment)
        ambiguous["TC_PROOF_NATIVE_BINARY"] = str(native)
        ambiguous["TC_PROOF_NATIVE_LAUNCHER"] = str(forged)
        require_rejected(bundle, direct_context, cwd=source, env=ambiguous, reason="ambiguous override")

        inside = dict(environment)
        inside["TC_PROOF_NATIVE_BINARY"] = str(source / "tools/refactor-proof/bin/tc-proof")
        require_rejected(bundle, direct_context, cwd=source, env=inside, reason="candidate-worktree override")

        defaulted = dict(environment)
        defaulted.pop("TC_PROOF_NATIVE_BINARY", None)
        defaulted.pop("TC_PROOF_NATIVE_LAUNCHER", None)
        if (source / "target").is_symlink():
            require_rejected(bundle, direct_context, cwd=source, env=defaulted, reason="shared-cache default")

    print(json.dumps({
        "schema": "tc-proof-compare-external-target-regression/v1",
        "qualification": report,
        "rejections": [
            "relative",
            "missing",
            "symlink",
            "unbound",
            "ambiguous",
            "candidate-worktree",
            "shared-cache-default",
        ],
    }, sort_keys=True))


if __name__ == "__main__":
    main()
