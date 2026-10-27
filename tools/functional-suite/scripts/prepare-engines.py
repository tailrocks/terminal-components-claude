#!/usr/bin/env python3
"""Fetch and verify the suite's pinned engine checkouts. Runs no Cargo command."""

from __future__ import annotations

import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path
from typing import NoReturn


SUITE_ROOT = Path(__file__).resolve().parents[1]
FORKS_ROOT = SUITE_ROOT / "forks"


def fail(message: str) -> NoReturn:
    print(f"prepare-engines: {message}", file=sys.stderr)
    raise SystemExit(2)


def run(args: list[str], *, cwd: Path | None = None, capture: bool = False) -> str:
    result = subprocess.run(
        args,
        cwd=cwd,
        check=False,
        text=True,
        stdout=subprocess.PIPE if capture else None,
        stderr=subprocess.PIPE if capture else None,
    )
    if result.returncode != 0:
        detail = (result.stderr or result.stdout or "").strip() if capture else ""
        fail(f"command failed ({result.returncode}): {' '.join(args)}" + (f"\n{detail}" if detail else ""))
    return result.stdout.strip() if capture else ""


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def verify_complete_checkout(repository: Path) -> None:
    tracked = run(["git", "-C", str(repository), "ls-files", "-v"], capture=True)
    tracked_paths = []
    for line in tracked.splitlines():
        tag = line[:1]
        if tag == "S" or tag.islower():
            fail(f"checkout is sparse or has assume-unchanged files: {repository}")
        tracked_paths.append(line[2:])
    if not tracked_paths:
        fail(f"checkout has no tracked source files: {repository}")

    missing = run(["git", "-C", str(repository), "ls-files", "--deleted"], capture=True)
    if missing:
        fail(f"checkout is missing tracked files: {repository}")
    for relative in tracked_paths:
        tracked_path = repository / relative
        if not tracked_path.exists() and not tracked_path.is_symlink():
            fail(f"checkout is missing tracked path {relative}: {repository}")


def require_inside(root: Path, path: Path, label: str) -> Path:
    resolved_root = root.resolve()
    resolved_path = path.resolve()
    if os.path.commonpath((str(resolved_root), str(resolved_path))) != str(resolved_root):
        fail(f"{label} escapes suite root: {resolved_path}")
    return resolved_path


def ensure_checkout(repository: str, commit: str, destination: Path) -> None:
    if destination.is_symlink():
        fail(f"refusing symlink checkout path: {destination}")
    created = False
    if destination.exists():
        if not destination.is_dir() or not (destination / ".git").exists():
            fail(f"checkout path exists but is not a Git checkout: {destination}")
    else:
        destination.parent.mkdir(parents=True, exist_ok=True)
        run(["git", "clone", repository, str(destination)])
        created = True

    head = run(["git", "-C", str(destination), "rev-parse", "HEAD"], capture=True)
    if head != commit:
        if not created:
            fail(f"existing checkout has HEAD {head}, expected {commit}: {destination}")
        run(["git", "-C", str(destination), "checkout", "--detach", commit])
        head = run(["git", "-C", str(destination), "rev-parse", "HEAD"], capture=True)
    if head != commit:
        fail(f"checkout resolved to {head}, expected {commit}: {destination}")
    verify_complete_checkout(destination)


def status_lines(repository: Path) -> list[str]:
    result = subprocess.run(
        ["git", "-C", str(repository), "status", "--porcelain", "--untracked-files=all"],
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if result.returncode != 0:
        fail(f"git status failed for engine checkout {repository}: {result.stderr.strip()}")
    return result.stdout.splitlines() if result.stdout.strip() else []


def verify_tui_test(repository: Path, pin: dict, patch_path: Path) -> None:
    base = pin["base_commit"]
    expected_sources: dict[str, str] = pin["source_sha256"]
    expected_preimages: dict[str, str] = pin["preimage_git_blob_sha1"]
    if set(expected_preimages) != set(expected_sources):
        fail("tui-test preimage and postimage manifests cover different paths")
    for relative, expected_blob in expected_preimages.items():
        actual_blob = run(
            ["git", "-C", str(repository), "rev-parse", f"{base}:{relative}"],
            capture=True,
        )
        if actual_blob != expected_blob:
            fail(f"tui-test base preimage mismatch for {relative}: {actual_blob}")
    current_status = status_lines(repository)

    if not current_status:
        run(["git", "-C", str(repository), "apply", "--check", str(patch_path)])
        run(["git", "-C", str(repository), "apply", str(patch_path)])
        current_status = status_lines(repository)

    if not current_status or any(line[:2] != " M" or len(line) < 4 for line in current_status):
        fail(f"tui-test checkout has unexpected staged, deleted, or untracked state: {repository}")

    changed = run(
        ["git", "-C", str(repository), "diff", "--name-only", base],
        capture=True,
    )
    changed_paths = set(changed.splitlines()) if changed else set()
    expected_paths = set(expected_sources)
    if changed_paths != expected_paths:
        fail(f"tui-test changed paths differ from approved patch: {sorted(changed_paths)}")

    staged = subprocess.run(
        ["git", "-C", str(repository), "diff", "--cached", "--quiet", base],
        check=False,
    )
    if staged.returncode != 0:
        fail("tui-test checkout has staged changes")
    run(["git", "-C", str(repository), "diff", "--check", base])

    for relative, expected_hash in expected_sources.items():
        source_path = require_inside(repository, repository / relative, "tui-test source path")
        if not source_path.is_file():
            fail(f"pinned tui-test source file is missing: {relative}")
        actual_hash = sha256(source_path)
        if actual_hash != expected_hash:
            fail(f"tui-test source hash mismatch for {relative}: {actual_hash}")


def verify_tuiscotti(repository: Path, pin: dict, patch_path: Path) -> None:
    base = pin["commit"]
    expected_preimages: dict[str, str] = pin["preimage_sha256"]
    expected_sources: dict[str, str] = pin["source_sha256"]
    if set(expected_preimages) != set(expected_sources):
        fail("tuiscotti preimage and postimage manifests cover different paths")

    for relative, expected_hash in expected_preimages.items():
        result = subprocess.run(
            ["git", "-C", str(repository), "show", f"{base}:{relative}"],
            check=False,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        if result.returncode != 0:
            fail(f"cannot read pinned tuiscotti preimage {relative} at {base}")
        actual_hash = hashlib.sha256(result.stdout).hexdigest()
        if actual_hash != expected_hash:
            fail(f"tuiscotti base preimage mismatch for {relative}: {actual_hash}")

    current_status = status_lines(repository)
    if not current_status:
        run(["git", "-C", str(repository), "apply", "--check", str(patch_path)])
        run(["git", "-C", str(repository), "apply", str(patch_path)])
        current_status = status_lines(repository)

    if any(line[:2] != " M" or len(line) < 4 for line in current_status):
        fail(f"tuiscotti checkout has unexpected staged, deleted, or untracked state: {repository}")
    changed = run(
        ["git", "-C", str(repository), "diff", "--name-only", base],
        capture=True,
    )
    changed_paths = set(changed.splitlines()) if changed else set()
    if changed_paths != set(expected_sources):
        fail(f"tuiscotti changed paths differ from approved patch: {sorted(changed_paths)}")
    reported_paths = {line[3:] for line in current_status}
    if reported_paths != set(expected_sources):
        fail(f"tuiscotti worktree status differs from approved patch: {sorted(reported_paths)}")

    staged = subprocess.run(
        ["git", "-C", str(repository), "diff", "--cached", "--quiet", base],
        check=False,
    )
    if staged.returncode != 0:
        fail("tuiscotti checkout has staged changes")
    run(["git", "-C", str(repository), "diff", "--check", base])

    for relative, expected_hash in expected_sources.items():
        source_path = require_inside(repository, repository / relative, "tuiscotti source path")
        if not source_path.is_file():
            fail(f"pinned tuiscotti source file is missing: {relative}")
        actual_hash = sha256(source_path)
        if actual_hash != expected_hash:
            fail(f"tuiscotti source hash mismatch for {relative}: {actual_hash}")

def main() -> None:
    if len(sys.argv) != 1:
        fail("takes no arguments")

    if FORKS_ROOT.is_symlink():
        fail(f"refusing symlink fork directory: {FORKS_ROOT}")
    FORKS_ROOT.mkdir(parents=True, exist_ok=True)
    require_inside(SUITE_ROOT, FORKS_ROOT, "fork directory")

    pins_path = SUITE_ROOT / "engine-pins.json"
    pins = json.loads(pins_path.read_text(encoding="utf-8"))
    tui_test = pins["tui_test_rs"]
    patch_path = require_inside(SUITE_ROOT, SUITE_ROOT / tui_test["patch_file"], "patch path")
    if sha256(patch_path) != tui_test["patch_sha256"]:
        fail(f"tui-test patch hash mismatch: {patch_path}")

    tuiscotti = pins["tuiscotti"]
    tuiscotti_patch_path = require_inside(
        SUITE_ROOT,
        SUITE_ROOT / tuiscotti["pixel_gap_patch_file"],
        "tuiscotti patch path",
    )
    if sha256(tuiscotti_patch_path) != tuiscotti["pixel_gap_patch_sha256"]:
        fail(f"tuiscotti patch hash mismatch: {tuiscotti_patch_path}")
    tuiscotti_path = FORKS_ROOT / "tuiscotti-upstream"
    ensure_checkout(tuiscotti["repository"], tuiscotti["commit"], tuiscotti_path)
    verify_tuiscotti(tuiscotti_path, tuiscotti, tuiscotti_patch_path)

    tui_test_path = FORKS_ROOT / "tui-test-upstream"
    ensure_checkout(tui_test["repository"], tui_test["base_commit"], tui_test_path)
    verify_tui_test(tui_test_path, tui_test, patch_path)

    print("prepared pinned tui-test-rs and patched tuiscotti engine sources")


if __name__ == "__main__":
    main()
