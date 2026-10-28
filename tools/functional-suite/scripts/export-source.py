#!/usr/bin/env python3
"""Export only the functional-suite source tree to a new external directory."""

from __future__ import annotations

import argparse
import hashlib
import os
import shutil
import stat
import sys
import tempfile
from pathlib import Path
from typing import NoReturn


SUITE_ROOT = Path(__file__).resolve().parents[1]
EXCLUDED_ROOT_DIRS = {"forks", "target"}
EXCLUDED_DIRS = {".git", "__pycache__"}
EXCLUDED_FILES = {Path("expected/.tuiscotti-approval.lock")}


def fail(message: str) -> NoReturn:
    print(f"export-source: {message}", file=sys.stderr)
    raise SystemExit(2)


def within(path: Path, root: Path) -> bool:
    try:
        path.relative_to(root)
        return True
    except ValueError:
        return False


def source_files() -> list[Path]:
    files: list[Path] = []
    for current, dirs, names in os.walk(SUITE_ROOT, topdown=True, followlinks=False):
        directory = Path(current)
        kept_dirs = []
        for name in sorted(dirs):
            child = directory / name
            if name in EXCLUDED_DIRS or (directory == SUITE_ROOT and name in EXCLUDED_ROOT_DIRS):
                continue
            if child.is_symlink():
                fail(f"refusing symlink in source tree: {child}")
            kept_dirs.append(name)
        dirs[:] = kept_dirs
        for name in sorted(names):
            source = directory / name
            if directory == SUITE_ROOT and name in EXCLUDED_ROOT_DIRS:
                continue
            if source.is_symlink():
                fail(f"refusing symlink in source tree: {source}")
            if source.relative_to(SUITE_ROOT) in EXCLUDED_FILES:
                continue
            if source.is_file():
                files.append(source)
    return sorted(files, key=lambda path: path.relative_to(SUITE_ROOT).as_posix())


def file_hash(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("destination", type=Path, help="new absolute directory for the exported package")
    parser.add_argument(
        "--forbid-root",
        action="append",
        default=[],
        type=Path,
        help="reject a destination inside this application worktree; repeat for each worktree",
    )
    args = parser.parse_args()

    if not args.destination.is_absolute():
        fail("destination must be an absolute path")
    if args.destination.is_symlink():
        fail(f"refusing symlink destination: {args.destination}")
    destination = args.destination.resolve()
    source_root = SUITE_ROOT.resolve()
    if destination == source_root or within(destination, source_root):
        fail("destination must be outside the source tree")
    for forbidden in args.forbid_root:
        forbidden_root = forbidden.resolve()
        if within(destination, forbidden_root):
            fail(f"destination is inside a forbidden worktree: {forbidden_root}")
    if destination.exists() or destination.is_symlink():
        fail(f"destination already exists: {destination}")

    files = source_files()
    if not files:
        fail("source tree is empty")
    destination.parent.mkdir(parents=True, exist_ok=True)
    staging_parent = Path(tempfile.mkdtemp(prefix=".functional-suite-export-", dir=destination.parent))
    staged = staging_parent / "package"
    try:
        staged.mkdir()
        records: list[tuple[str, str, int]] = []
        for source in files:
            relative = source.relative_to(source_root)
            target = staged / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, target)
            records.append((relative.as_posix(), file_hash(source), stat.S_IMODE(source.stat().st_mode)))

        for relative, expected_hash, expected_mode in records:
            target = staged / relative
            if not target.is_file() or file_hash(target) != expected_hash:
                fail(f"exported file differs from source: {relative}")
            if stat.S_IMODE(target.stat().st_mode) != expected_mode:
                fail(f"exported file mode differs from source: {relative}")

        os.replace(staged, destination)
    finally:
        shutil.rmtree(staging_parent, ignore_errors=True)

    manifest = "".join(f"{digest}  {relative}\n" for relative, digest, _ in records)
    manifest_hash = hashlib.sha256(manifest.encode()).hexdigest()
    print(f"exported {len(records)} source files to {destination}")
    print(f"source manifest sha256={manifest_hash}")


if __name__ == "__main__":
    main()
