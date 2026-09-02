#!/usr/bin/env python3
"""Convert `tmux capture-pane -e -p` output (SGR truecolor) into a standalone HTML page.

Usage: ansi2html.py <in.ansi> <out.html> [cols] [rows]
"""
import html
import re
import sys

SGR = re.compile(r"\x1b\[([0-9;:]*)m")
OTHER = re.compile(r"\x1b\[[0-9;?]*[A-Za-z]")
BASIC16 = [
    "#000000", "#cd3131", "#0dbc79", "#e5e510", "#2472c8", "#bc3fbc", "#11a8cd", "#e5e5e5",
    "#666666", "#f14c4c", "#23d18b", "#f5f543", "#3b8eea", "#d670d6", "#29b8db", "#ffffff",
]


def xterm256(n: int) -> str:
    if n < 16:
        return BASIC16[n]
    if n < 232:
        n -= 16
        r, g, b = n // 36, (n // 6) % 6, n % 6
        conv = lambda v: 0 if v == 0 else 55 + v * 40
        return f"#{conv(r):02x}{conv(g):02x}{conv(b):02x}"
    v = 8 + (n - 232) * 10
    return f"#{v:02x}{v:02x}{v:02x}"


class State:
    def __init__(self):
        self.reset()

    def reset(self):
        self.fg = None
        self.bg = None
        self.bold = False
        self.dim = False
        self.italic = False
        self.underline = False
        self.reverse = False
        self.strike = False

    def css(self, default_fg, default_bg):
        fg = self.fg or default_fg
        bg = self.bg or default_bg
        if self.reverse:
            fg, bg = bg, fg
        parts = [f"color:{fg}", f"background:{bg}"]
        if self.bold:
            parts.append("font-weight:700")
        if self.dim:
            parts.append("opacity:.6")
        if self.italic:
            parts.append("font-style:italic")
        deco = []
        if self.underline:
            deco.append("underline")
        if self.strike:
            deco.append("line-through")
        if deco:
            parts.append("text-decoration:" + " ".join(deco))
        return ";".join(parts)


def apply(state: State, params: str):
    if params == "":
        state.reset()
        return
    toks = [int(t) if t else 0 for t in re.split(r"[;:]", params)]
    i = 0
    while i < len(toks):
        t = toks[i]
        if t == 0:
            state.reset()
        elif t == 1:
            state.bold = True
        elif t == 2:
            state.dim = True
        elif t == 3:
            state.italic = True
        elif t == 4:
            state.underline = True
        elif t == 7:
            state.reverse = True
        elif t == 9:
            state.strike = True
        elif t == 22:
            state.bold = state.dim = False
        elif t == 23:
            state.italic = False
        elif t == 24:
            state.underline = False
        elif t == 27:
            state.reverse = False
        elif t == 29:
            state.strike = False
        elif 30 <= t <= 37:
            state.fg = BASIC16[t - 30]
        elif 90 <= t <= 97:
            state.fg = BASIC16[t - 90 + 8]
        elif 40 <= t <= 47:
            state.bg = BASIC16[t - 40]
        elif 100 <= t <= 107:
            state.bg = BASIC16[t - 100 + 8]
        elif t == 39:
            state.fg = None
        elif t == 49:
            state.bg = None
        elif t in (38, 48, 58):
            mode = toks[i + 1] if i + 1 < len(toks) else 0
            if mode == 2 and i + 4 < len(toks):
                col = f"#{toks[i+2]:02x}{toks[i+3]:02x}{toks[i+4]:02x}"
                i += 4
            elif mode == 5 and i + 2 < len(toks):
                col = xterm256(toks[i + 2])
                i += 2
            else:
                col = None
            if t == 38:
                state.fg = col
            elif t == 48:
                state.bg = col
        i += 1


def convert(text: str, cols: int, rows: int, default_fg="#d0d0d0", default_bg="#000000") -> str:
    lines = text.split("\n")
    out = []
    # tmux emits SGR deltas that carry across line boundaries
    state = State()
    for line in lines[:rows]:
        buf = []
        pos = 0
        width = 0
        line = OTHER.sub(lambda m: m.group(0) if m.group(0).endswith("m") else "", line)
        for m in SGR.finditer(line):
            seg = line[pos:m.start()]
            if seg:
                buf.append(f'<span style="{state.css(default_fg, default_bg)}">{html.escape(seg)}</span>')
                width += len(seg)
            apply(state, m.group(1))
            pos = m.end()
        seg = line[pos:]
        if seg:
            buf.append(f'<span style="{state.css(default_fg, default_bg)}">{html.escape(seg)}</span>')
            width += len(seg)
        if width < cols:
            buf.append(f'<span style="background:{default_bg}">{" " * (cols - width)}</span>')
        out.append("".join(buf))
    while len(out) < rows:
        out.append(f'<span style="background:{default_bg}">{" " * cols}</span>')
    body = "\n".join(out)
    return f"""<!doctype html><html><head><meta charset="utf-8"><title>capture</title>
<style>
html,body{{margin:0;background:#1a1a1a}}
pre{{margin:16px;display:inline-block;font-family:"JetBrainsMono Nerd Font Mono","JetBrains Mono",Menlo,monospace;font-size:14px;line-height:18px;white-space:pre;background:{default_bg}}}
span{{display:inline-block;height:18px;vertical-align:top}}
</style></head><body><pre>{body}</pre></body></html>"""


if __name__ == "__main__":
    src, dst = sys.argv[1], sys.argv[2]
    cols = int(sys.argv[3]) if len(sys.argv) > 3 else 120
    rows = int(sys.argv[4]) if len(sys.argv) > 4 else 40
    with open(src, encoding="utf-8", errors="replace") as f:
        text = f.read()
    with open(dst, "w", encoding="utf-8") as f:
        f.write(convert(text, cols, rows))
