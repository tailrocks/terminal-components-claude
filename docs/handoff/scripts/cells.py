# Attribute changed cells: approved snapshots/*.ansi vs target/tuiscotti/actual for every FAIL in the given visual logs.
"""Attribute changed cells between approved snapshots and actual captures.

usage: python3 -I cells.py <worktree> <vis-log>...   (prints a summary per scenario)
"""
import os
import re
import sys

SGR = re.compile(r"\x1b\[([0-9;?]*)([a-zA-Z])")


def parse(path):
    rows = []
    with open(path, encoding="utf-8", errors="replace") as f:
        text = f.read()
    for line in text.split("\n"):
        cells = []
        style = ()
        i = 0
        while i < len(line):
            m = SGR.match(line, i)
            if m:
                if m.group(2) == "m":
                    p = m.group(1)
                    if p in ("", "0"):
                        style = ()
                    else:
                        style = style + (p,)
                i = m.end()
                continue
            cells.append((line[i], style))
            i += 1
        rows.append(cells)
    return rows


def main():
    root = sys.argv[1]
    names = set()
    for log in sys.argv[2:]:
        for l in open(log, encoding="utf-8", errors="replace"):
            m = re.match(r"\s+FAIL \[.*visual_baseline (\S+)$", l.rstrip())
            if m:
                names.add(m.group(1).replace("::", "/"))
    for name in sorted(names):
        exp = os.path.join(root, "snapshots", name + ".ansi")
        act = os.path.join(root, "target/tuiscotti/actual", name + ".ansi")
        if not (os.path.exists(exp) and os.path.exists(act)):
            print(name, "MISSING")
            continue
        e, a = parse(exp), parse(act)
        text_diff = []
        style_only = []
        for y in range(max(len(e), len(a))):
            er = e[y] if y < len(e) else []
            ar = a[y] if y < len(a) else []
            for x in range(max(len(er), len(ar))):
                ec = er[x] if x < len(er) else (None, ())
                ac = ar[x] if x < len(ar) else (None, ())
                if ec[0] != ac[0]:
                    text_diff.append((y, x, ec[0], ac[0]))
                elif ec[1] != ac[1]:
                    style_only.append((y, x, ec[0], ec[1], ac[1]))
        blank_dim = all(c[2] == " " and ";".join(c[3]).endswith(";".join(c[4])) and ";4;" not in ";" + ";".join(c[3]) + ";" for c in style_only)
        rows = sorted({c[0] for c in style_only} | {c[0] for c in text_diff})
        print(f"{name}: text {len(text_diff)} style {len(style_only)} rows {rows[:8]} blank-dim-only={blank_dim and not text_diff}")
        for c in text_diff[:6]:
            print("   T", c)
        if not blank_dim:
            for c in [c for c in style_only if not (c[2] == " " and ";".join(c[3]).endswith(";".join(c[4])))][:4]:
                print("   S", c)


main()
