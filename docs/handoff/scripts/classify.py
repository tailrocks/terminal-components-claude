# Classify every changed cell of failing captures (blank / undim / other); needs cells.py beside it.
"""Classify every changed cell of the failing captures (run cells.py's parser).

usage: python3 -I classify.py <worktree> <vis-log>...
Classes: blank (space whose style lost stale fg/dim, bg kept),
undim (glyph whose style lost only a leading DIM `2;` or `2`), other.
"""
import importlib.util
import os
import re
import sys

spec = importlib.util.spec_from_file_location(
    "cells", os.path.join(os.path.dirname(os.path.abspath(__file__)), "cells.py"))
src = open(spec.origin).read().replace("\nmain()\n", "\n")
ns = {}
exec(compile(src, spec.origin, "exec"), ns)
parse = ns["parse"]

root = sys.argv[1]
names = set()
for log in sys.argv[2:]:
    for l in open(log, encoding="utf-8", errors="replace"):
        m = re.match(r"\s+FAIL \[.*visual_baseline (\S+)$", l.rstrip())
        if m:
            names.add(m.group(1).replace("::", "/"))
totals = {"blank": 0, "undim": 0, "other": 0}
others = []
for name in sorted(names):
    e = parse(os.path.join(root, "snapshots", name + ".ansi"))
    a = parse(os.path.join(root, "target/tuiscotti/actual", name + ".ansi"))
    for y in range(max(len(e), len(a))):
        er = e[y] if y < len(e) else []
        ar = a[y] if y < len(a) else []
        for x in range(max(len(er), len(ar))):
            ec = er[x] if x < len(er) else (None, ())
            ac = ar[x] if x < len(ar) else (None, ())
            if ec == ac:
                continue
            es, as_ = ";".join(ec[1]), ";".join(ac[1])
            if ec[0] != ac[0]:
                cls = "other"
            elif ec[0] == " " and es.endswith(as_):
                cls = "blank"
            elif es in ("2;" + as_, "2") and (as_ != "" or es == "2"):
                cls = "undim"
            else:
                cls = "other"
            totals[cls] += 1
            if cls == "other" and len(others) < 20:
                others.append((name, y, x, ec, ac))
print(len(names), "captures", totals)
for o in others:
    print(o)
