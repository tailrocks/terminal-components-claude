#!/usr/bin/env python3
"""Compute NUL-delimited lexicographic aggregate SHA-256 of snapshots/ + shots/."""
from __future__ import annotations

import hashlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def main() -> None:
    paths: list[Path] = []
    for base in ("snapshots", "shots"):
        root = ROOT / base
        if root.is_dir():
            paths.extend(sorted(p for p in root.rglob("*") if p.is_file()))
    digest = hashlib.sha256()
    for path in paths:
        digest.update(path.as_posix().encode())
        digest.update(b"\0")
        digest.update(path.read_bytes())
        digest.update(b"\0")
    print(digest.hexdigest())


if __name__ == "__main__":
    main()
