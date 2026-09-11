"""Immutable synthetic checker; no imports from candidate or host implementation."""
import pathlib
import sys

mode = sys.argv[1]
if mode not in {"sentinel", "payload", "all"}:
    raise SystemExit(64)
if mode in {"sentinel", "all"}:
    if pathlib.Path("protected.txt").read_bytes() != b"planner-owned sentinel\n":
        raise SystemExit(3)
if mode in {"payload", "all"}:
    if pathlib.Path("src/payload.txt").read_bytes() != b"qualified\n":
        raise SystemExit(4)
