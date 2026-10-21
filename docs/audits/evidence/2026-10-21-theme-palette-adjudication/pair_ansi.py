#!/usr/bin/env python3
"""Pair truecolor and 256 ANSI frames cell-by-cell and report the mapping."""
import re
import sys
from collections import Counter

SGR = re.compile(r"\x1b\[([0-9;]*)m")


def parse(path):
    """Return list of rows; each row is a list of (fg, bg) tuples."""
    data = open(path, "rb").read().decode("utf-8", "replace")
    rows = []
    for line in data.split("\n"):
        fg, bg = None, None
        cells = []
        pos = 0
        for m in SGR.finditer(line):
            if m.start() > pos:
                chunk = line[pos : m.start()]
                for ch in chunk:
                    cells.append((fg, bg))
            codes = m.group(1)
            parts = codes.split(";") if codes else ["0"]
            i = 0
            while i < len(parts):
                p = parts[i]
                if p in ("", "0"):
                    fg, bg = None, None
                elif p == "38" and i + 1 < len(parts):
                    kind = parts[i + 1]
                    if kind == "2":
                        fg = tuple(int(x) for x in parts[i + 2 : i + 5])
                        i += 4
                    elif kind == "5":
                        fg = int(parts[i + 2])
                        i += 2
                elif p == "48" and i + 1 < len(parts):
                    kind = parts[i + 1]
                    if kind == "2":
                        bg = tuple(int(x) for x in parts[i + 2 : i + 5])
                        i += 4
                    elif kind == "5":
                        bg = int(parts[i + 2])
                        i += 2
                i += 1
            pos = m.end()
        if pos < len(line):
            for ch in line[pos:]:
                cells.append((fg, bg))
        rows.append(cells)
    return rows


def main(tc_path, i256_path):
    tc = parse(tc_path)
    q = parse(i256_path)
    n = min(len(tc), len(q))
    fgm = Counter()
    bgm = Counter()
    for r in range(n):
        a, b = tc[r], q[r]
        for c in range(min(len(a), len(b))):
            (tfg, tbg), (qfg, qbg) = a[c], b[c]
            if tfg is not None:
                fgm[(tfg, qfg)] += 1
            if tbg is not None:
                bgm[(tbg, qbg)] += 1
    print("=== fg: truecolor -> 256 ===")
    for (t, q2), c in fgm.most_common(40):
        print(f"  {t} -> {q2}   x{c}")
    print("=== bg: truecolor -> 256 ===")
    for (t, q2), c in bgm.most_common(25):
        print(f"  {t} -> {q2}   x{c}")


main(sys.argv[1], sys.argv[2])
