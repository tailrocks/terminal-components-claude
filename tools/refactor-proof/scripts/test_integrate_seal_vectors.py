#!/usr/bin/env python3
"""Exercise integrate/seal HostFixture vectors against a built tc-proof-host."""
from __future__ import annotations

import importlib.util
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

WORKTREE = Path(__file__).resolve().parents[3]
REPO = WORKTREE.parents[1] if WORKTREE.name == "main" else WORKTREE
DRIVER = REPO / "refactoring-tasks/terminal-components/completion/001/trusted/proof-bootstrap/host-bootstrap-driver.py"
HOST = WORKTREE / "target/debug/tc-proof-host"
TASKFMT = Path(
    os.environ["TASKFMT_BIN"]
    if "TASKFMT_BIN" in os.environ
    else WORKTREE / ".proof/bootstrap/bin/taskfmt"
)
TASKFMT_SOURCE = Path(
    os.environ["TASKFMT_SOURCE"]
    if "TASKFMT_SOURCE" in os.environ
    else WORKTREE / ".proof/bootstrap/task-format"
)


def require_taskfmt_paths() -> None:
    missing: list[str] = []
    if not TASKFMT.is_file():
        hint = (
            "set TASKFMT_BIN"
            if "TASKFMT_BIN" in os.environ
            else f"default {WORKTREE / '.proof/bootstrap/bin/taskfmt'}"
        )
        missing.append(f"taskfmt binary missing: {TASKFMT} ({hint})")
    if not TASKFMT_SOURCE.is_dir():
        hint = (
            "set TASKFMT_SOURCE"
            if "TASKFMT_SOURCE" in os.environ
            else f"default {WORKTREE / '.proof/bootstrap/task-format'}"
        )
        missing.append(f"taskfmt source missing: {TASKFMT_SOURCE} ({hint})")
    if missing:
        for line in missing:
            print(line, file=sys.stderr)
        print(
            "Set TASKFMT_BIN and TASKFMT_SOURCE or install pinned bootstrap under .proof/bootstrap/",
            file=sys.stderr,
        )
        sys.exit(2)

SEAL_NEGATIVE = {
    "premature_seal": "authority",
    "unauthorized_seal": "authority",
}

INTEGRATE_NEGATIVE = {
    "wrong_expected_parent": "parent",
    "stale_parent_cas": "parent",
    "wrong_integration_ref": "authority",
}

REF = "refs/heads/refactor/holla-parity"


def load_driver():
    spec = importlib.util.spec_from_file_location("host_bootstrap_driver", DRIVER)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def git(root: Path, *args: str) -> str:
    env = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}
    env.update(
        GIT_CONFIG_NOSYSTEM="1",
        GIT_CONFIG_GLOBAL=os.devnull,
        GIT_AUTHOR_DATE="2026-01-01T00:00:00+0000",
        GIT_COMMITTER_DATE="2026-01-01T00:00:00+0000",
    )
    result = subprocess.run(
        ["git", "-c", "core.hooksPath=" + os.devnull, "-C", str(root), *args],
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        raise RuntimeError(f"git {' '.join(args)}: {result.stderr}")
    return result.stdout.strip()


def install_host(driver, fixture) -> None:
    fixture.call("install", "--receipt", fixture.harness_receipt, "--destination", fixture.install)
    installed = fixture.install / "bin/tc-proof-host"
    driver.require(
        not installed.is_symlink()
        and installed.is_file()
        and driver.digest(installed) == driver.digest(fixture.executable),
        "install did not reproduce accepted regular executable",
    )
    fixture.executable = installed


def write_preparation(driver, fixture) -> None:
    fixture.run_dir.mkdir(parents=True, exist_ok=True)
    driver.write_json(
        fixture.run_dir / "preparation.json",
        {"task": fixture.task, "parent": fixture.parent},
    )


def write_verified_run(driver, fixture, expected_tree: str) -> None:
    write_preparation(driver, fixture)
    driver.write_json(
        fixture.run_dir / "freeze.json",
        {
            "tree": expected_tree,
            "parent": fixture.parent,
            "scope_base": fixture.scope_base,
        },
    )
    driver.write_json(
        fixture.run_dir / "verdict.json",
        {
            "status": "passed",
            "tree": expected_tree,
            "parent": fixture.parent,
            "scope_base": fixture.scope_base,
            "task_id": fixture.task,
            "run_id": fixture.observer.nonce,
            "context_index_sha256": "0" * 64,
            "checks": [],
        },
    )


def call_seal_reject(driver, fixture, category: str) -> None:
    before = fixture.protected()
    fixture.call(
        "seal",
        "--run",
        fixture.run_dir,
        "--product",
        "oracle-showcase",
        category=category,
    )
    after = fixture.protected()
    if before != after:
        raise AssertionError("seal rejection mutated protected state")


def call_integrate_reject(
    driver,
    fixture,
    integration_ref: str,
    expected_parent: str,
    category: str,
) -> None:
    before = fixture.protected()
    fixture.call(
        "integrate",
        "--run",
        fixture.run_dir,
        "--ref",
        integration_ref,
        "--expected-parent",
        expected_parent,
        category=category,
    )
    after = fixture.protected()
    if before != after:
        raise AssertionError("integrate rejection mutated protected state")


def run_premature_seal(driver) -> None:
    with tempfile.TemporaryDirectory(prefix="tc-seal-premature-") as directory:
        fixture = driver.HostFixture(Path(directory), HOST, TASKFMT, TASKFMT_SOURCE, "premature_seal")
        try:
            install_host(driver, fixture)
            write_preparation(driver, fixture)
            call_seal_reject(driver, fixture, SEAL_NEGATIVE["premature_seal"])
        finally:
            fixture.observer.close()


def run_unauthorized_seal(driver) -> None:
    with tempfile.TemporaryDirectory(prefix="tc-seal-unauthorized-") as directory:
        fixture = driver.HostFixture(Path(directory), HOST, TASKFMT, TASKFMT_SOURCE, "unauthorized_seal")
        try:
            install_host(driver, fixture)
            expected_tree = fixture.independent_tree()
            write_verified_run(driver, fixture, expected_tree)
            call_seal_reject(driver, fixture, SEAL_NEGATIVE["unauthorized_seal"])
        finally:
            fixture.observer.close()


def run_integrate_negative(driver, case: str, category: str) -> None:
    with tempfile.TemporaryDirectory(prefix=f"tc-integrate-{case}-") as directory:
        fixture = driver.HostFixture(Path(directory), HOST, TASKFMT, TASKFMT_SOURCE, case)
        try:
            install_host(driver, fixture)
            expected_tree = fixture.independent_tree()
            write_verified_run(driver, fixture, expected_tree)
            integration_ref, expected_parent = REF, fixture.parent
            if case == "wrong_expected_parent":
                expected_parent = "0" * 40
            elif case == "stale_parent_cas":
                (fixture.repo / "src/advance.txt").write_bytes(b"concurrent integration\n")
                advanced = driver.commit(fixture.repo, "Advance integration parent")
                git(fixture.repo, "update-ref", REF, advanced, fixture.parent)
            elif case == "wrong_integration_ref":
                integration_ref = "refs/heads/main"
            call_integrate_reject(
                driver,
                fixture,
                integration_ref,
                expected_parent,
                category,
            )
        finally:
            fixture.observer.close()


def main() -> int:
    if not HOST.is_file():
        print(f"host executable missing: {HOST}", file=sys.stderr)
        return 2
    require_taskfmt_paths()
    driver = load_driver()
    passed = 0
    failed: list[str] = []
    cases = {**SEAL_NEGATIVE, **INTEGRATE_NEGATIVE}
    for case, category in cases.items():
        try:
            if case == "premature_seal":
                run_premature_seal(driver)
            elif case == "unauthorized_seal":
                run_unauthorized_seal(driver)
            else:
                run_integrate_negative(driver, case, category)
            passed += 1
            print(f"PASS {case} -> {category}")
        except Exception as error:  # noqa: BLE001
            failed.append(f"{case}: {error}")
            print(f"FAIL {case}: {error}")
    total = len(cases)
    print(f"SUMMARY passed={passed}/{total} failed={len(failed)}")
    if failed:
        for item in failed:
            print(f"  - {item}")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
