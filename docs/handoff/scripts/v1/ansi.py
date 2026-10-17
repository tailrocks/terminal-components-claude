# Minimal .ansi frame parser (SGR runs to cells) used by report.py and effect.py.
"""Minimal parser for tuiscotti/tuisnap .ansi frames (SGR runs, one line per row)."""
import re

SGR = re.compile(r"\x1b\[([0-9;?]*)([a-zA-Z])")


def parse(path):
    data = open(path, "rb").read().decode("utf-8", "replace")
    rows = []
    for line in data.split("\n"):
        line = line.rstrip("\r")
        cells = []
        fg = bg = None
        attrs = frozenset()
        pos = 0
        for m in SGR.finditer(line):
            for ch in line[pos:m.start()]:
                cells.append((ch, fg, bg, attrs))
            pos = m.end()
            if m.group(2) != "m":
                continue
            ps = [p for p in m.group(1).split(";")] if m.group(1) else ["0"]
            i = 0
            a = set(attrs)
            while i < len(ps):
                p = ps[i]
                p = int(p) if p.isdigit() else 0
                if p == 0:
                    fg = bg = None
                    a = set()
                elif p in (38, 48):
                    kind = int(ps[i + 1])
                    if kind == 2:
                        v = ("rgb", int(ps[i + 2]), int(ps[i + 3]), int(ps[i + 4]))
                        i += 4
                    else:
                        v = ("idx", int(ps[i + 2]))
                        i += 2
                    if p == 38:
                        fg = v
                    else:
                        bg = v
                elif p == 39:
                    fg = None
                elif p == 49:
                    bg = None
                elif 30 <= p <= 37 or 90 <= p <= 97:
                    fg = ("idx", p - 30 if p < 90 else p - 90 + 8)
                elif 40 <= p <= 47 or 100 <= p <= 107:
                    bg = ("idx", p - 40 if p < 100 else p - 100 + 8)
                elif p in (22, 23, 24, 25, 27, 28, 29):
                    off = {22: {1, 2}, 23: {3}, 24: {4}, 25: {5}, 27: {7}, 28: {8}, 29: {9}}[p]
                    a -= off
                else:
                    a.add(p)
                i += 1
            attrs = frozenset(a)
        for ch in line[pos:]:
            cells.append((ch, fg, bg, attrs))
        rows.append(cells)
    while rows and not rows[-1]:
        rows.pop()
    return rows


def text(rows):
    return ["".join(c[0] for c in r).rstrip() for r in rows]
