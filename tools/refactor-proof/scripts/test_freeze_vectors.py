#!/usr/bin/env python3
"""Exercise freeze-related HostFixture vectors against a built tc-proof-host."""
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
TASKFMT = Path(os.environ.get("TASKFMT_BIN", "/Users/donbeave/.cargo/bin/taskfmt"))
TASKFMT_SOURCE = Path(
    os.environ.get("TASKFMT_SOURCE", "/Users/donbeave/Projects/donbeave/task-format")
)
FREEZE_NEGATIVE = {
    "out_of_scope": "scope",
    "forbidden_checker": "scope",
    "overlay_tamper": "scope",
    "symlink_escape": "unsafe-path",
    "hardlink_escape": "unsafe-path",
    "ignored_source": "unsafe-git",
    "hidden_index_flag": "unsafe-git",
    "changed_git_config": "unsafe-git",
    "submodule_substitution": "unsafe-git",
}


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


def prepare_run(driver, fixture) -> None:
    fixture.run_dir.mkdir(parents=True, exist_ok=True)
    driver.write_json(
        fixture.run_dir / "preparation.json",
        {"task": fixture.task, "parent": fixture.parent},
    )
    progress = fixture.run_dir / "progress.md"
    progress.write_text("# synthetic progress for freeze-only qualification\nstate: DONE\n")


def apply_case(driver, fixture, case: str) -> None:
    candidate = fixture.candidate
    if case == "out_of_scope":
        (candidate / "outside.txt").write_bytes(b"out of scope\n")
    elif case == "forbidden_checker":
        (candidate / ".proof/check.py").write_text("raise SystemExit(0)\n")
    elif case == "overlay_tamper":
        (candidate / ".proof/overlay.txt").write_bytes(b"candidate-modified overlay\n")
    elif case == "symlink_escape":
        (candidate / "src/escape").symlink_to(fixture.expected)
    elif case == "hardlink_escape":
        os.link(fixture.expected, candidate / "src/escape")
    elif case == "ignored_source":
        with (candidate / ".git/info/exclude").open("a") as exclude:
            exclude.write("\nsrc/hidden.py\n")
        (candidate / "src/hidden.py").write_text("raise SystemExit(0)\n")
    elif case == "hidden_index_flag":
        git(candidate, "update-index", "--assume-unchanged", "protected.txt")
    elif case == "changed_git_config":
        git(candidate, "config", "--local", "user.name", "Candidate-supplied authority")
    elif case == "submodule_substitution":
        git(
            candidate,
            "update-index",
            "--add",
            "--cacheinfo",
            "160000," + fixture.parent + ",src/submodule",
        )


def call_freeze(driver, fixture) -> dict:
    result = fixture.observer.run_host(
        [str(HOST), "freeze", "--run", str(fixture.run_dir), "--candidate", str(fixture.candidate)],
        fixture.env,
        fixture.root,
        writable=[fixture.run_dir],
    )
    return driver.host_result(result, "freeze")


def call_freeze_reject(driver, fixture, category: str) -> None:
    before = fixture.protected()
    result = fixture.observer.run_host(
        [str(HOST), "freeze", "--run", str(fixture.run_dir), "--candidate", str(fixture.candidate)],
        fixture.env,
        fixture.root,
        writable=[fixture.run_dir],
    )
    driver.host_result(result, "freeze", category)
    after = fixture.protected()
    if before != after:
        raise AssertionError("freeze rejection mutated protected state")


def run_positive(driver) -> None:
    with tempfile.TemporaryDirectory(prefix="tc-freeze-positive-") as directory:
        fixture = driver.HostFixture(
            Path(directory), HOST, TASKFMT, TASKFMT_SOURCE, "exact_tested_tree"
        )
        try:
            prepare_run(driver, fixture)
            expected_tree = fixture.independent_tree()
            assert expected_tree != git(fixture.candidate, "write-tree")
            call_freeze(driver, fixture)
            frozen = json.loads((fixture.run_dir / "freeze.json").read_text())
            if frozen.get("tree") != expected_tree:
                raise AssertionError("freeze tree mismatch")
            if frozen.get("parent") != fixture.parent:
                raise AssertionError("freeze parent mismatch")
            if frozen.get("scope_base") != fixture.scope_base:
                raise AssertionError("freeze scope_base mismatch")
            index, contexts = fixture.check_contexts(expected_tree)
            fixture.validate_contexts(index, contexts)
        finally:
            fixture.observer.close()


def run_negative(driver, case: str, category: str) -> None:
    with tempfile.TemporaryDirectory(prefix=f"tc-freeze-{case}-") as directory:
        fixture = driver.HostFixture(Path(directory), HOST, TASKFMT, TASKFMT_SOURCE, case)
        try:
            prepare_run(driver, fixture)
            apply_case(driver, fixture, case)
            call_freeze_reject(driver, fixture, category)
        finally:
            fixture.observer.close()


def main() -> int:
    if not HOST.is_file():
        print(f"host executable missing: {HOST}", file=sys.stderr)
        return 2
    driver = load_driver()
    passed = 0
    failed: list[str] = []
    try:
        run_positive(driver)
        passed += 1
        print("PASS exact_tested_tree (freeze)")
    except Exception as error:  # noqa: BLE001
        failed.append(f"exact_tested_tree: {error}")
        print(f"FAIL exact_tested_tree (freeze): {error}")
    for case, category in FREEZE_NEGATIVE.items():
        try:
            run_negative(driver, case, category)
            passed += 1
            print(f"PASS {case} -> {category}")
        except Exception as error:  # noqa: BLE001
            failed.append(f"{case}: {error}")
            print(f"FAIL {case}: {error}")
    total = 1 + len(FREEZE_NEGATIVE)
    print(f"SUMMARY passed={passed}/{total} failed={len(failed)}")
    if failed:
        for item in failed:
            print(f"  - {item}")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
