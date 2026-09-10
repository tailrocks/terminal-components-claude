#!/usr/bin/env python3
"""One grapheme/cell representation shared by the HTML and PNG capture tools.

A tmux `capture-pane -e` line is parsed into styled runs, the plain text of
every line is segmented into graphemes with their cell widths by the Rust
reference (`cargo build --example cells`, the library's own
unicode-segmentation and unicode-width), and the result is laid out into a
fixed `cols` x `rows` grid. Wide graphemes occupy their extra cells as
continuations; anything past `cols` is clipped and reported, never painted
into the margin. The tools consume this grid and nothing else, so their
occupied cells, cursor and selected content agree by construction.
"""
import json
import os
import re
import subprocess
import sys

SGR = re.compile(r"\x1b\[([0-9;:]*)m")
OTHER = re.compile(r"\x1b\[[0-9;?]*[A-Za-z]")
BASIC16 = [
    "#000000", "#cd3131", "#0dbc79", "#e5e510", "#2472c8", "#bc3fbc", "#11a8cd", "#e5e5e5",
    "#666666", "#f14c4c", "#23d18b", "#f5f543", "#3b8eea", "#d670d6", "#29b8db", "#ffffff",
]
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
REFERENCE = os.environ.get("CELLS_BIN", os.path.join(ROOT, "target", "debug", "examples", "cells"))


def xterm256(n: int) -> str:
    if n < 16:
        return BASIC16[n]
    if n < 232:
        n -= 16
        r, g, b = n // 36, (n // 6) % 6, n % 6
        conv = lambda v: 0 if v == 0 else 55 + v * 40  # noqa: E731
        return f"#{conv(r):02x}{conv(g):02x}{conv(b):02x}"
    v = 8 + (n - 232) * 10
    return f"#{v:02x}{v:02x}{v:02x}"


class State:
    """SGR state; tmux emits deltas that carry across line boundaries."""

    __slots__ = ("fg", "bg", "bold", "dim", "italic", "underline", "reverse", "strike")

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

    def snapshot(self):
        return Style(self.fg, self.bg, self.bold, self.dim, self.italic, self.underline, self.reverse, self.strike)

    def css(self, default_fg, default_bg):
        return self.snapshot().css(default_fg, default_bg)


class Style:
    __slots__ = ("fg", "bg", "bold", "dim", "italic", "underline", "reverse", "strike")

    def __init__(self, fg=None, bg=None, bold=False, dim=False, italic=False, underline=False, reverse=False, strike=False):
        self.fg, self.bg = fg, bg
        self.bold, self.dim, self.italic = bold, dim, italic
        self.underline, self.reverse, self.strike = underline, reverse, strike

    def colors(self, default_fg, default_bg):
        fg = self.fg or default_fg
        bg = self.bg or default_bg
        if self.reverse:
            fg, bg = bg, fg
        return fg, bg

    def css(self, default_fg, default_bg):
        fg, bg = self.colors(default_fg, default_bg)
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

    def key(self):
        return (self.fg, self.bg, self.bold, self.dim, self.italic, self.underline, self.reverse, self.strike)


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


class Cell:
    """One terminal cell: the grapheme that starts here (empty for the
    continuation of a wide grapheme, a space for padding), its width in
    cells, and its style."""

    __slots__ = ("text", "width", "style", "continuation")

    def __init__(self, text, width, style, continuation=False):
        self.text = text
        self.width = width
        self.style = style
        self.continuation = continuation


def _runs(line: str, state: State):
    """Styled runs of one line; mutates the carried SGR state."""
    line = OTHER.sub(lambda m: m.group(0) if m.group(0).endswith("m") else "", line)
    runs = []
    pos = 0
    for m in SGR.finditer(line):
        seg = line[pos:m.start()]
        if seg:
            runs.append((seg, state.snapshot()))
        apply(state, m.group(1))
        pos = m.end()
    seg = line[pos:]
    if seg:
        runs.append((seg, state.snapshot()))
    return runs


def segment(plain_lines):
    """Graphemes and widths for each plain line from the Rust reference."""
    if not os.path.exists(REFERENCE):
        raise SystemExit(
            f"missing cell reference {REFERENCE}: run `cargo build --example cells` "
            "(the tools never guess widths)"
        )
    proc = subprocess.run(
        [REFERENCE],
        input="\n".join(plain_lines) + "\n",
        capture_output=True,
        text=True,
        check=True,
    )
    out = [json.loads(l) for l in proc.stdout.splitlines()]
    while len(out) < len(plain_lines):
        out.append([])
    return out


def layout(text: str, cols: int, rows: int):
    """Grid of `rows` lists of `cols` Cells plus a report of clipped and
    control graphemes. Rows beyond the capture are padding."""
    lines = text.split("\n")[:rows]
    state = State()
    styled = []
    for line in lines:
        runs = _runs(line, state)
        styled.append(runs)
    plain = ["".join(seg for seg, _ in runs) for runs in styled]
    segmented = segment(plain)
    grid = []
    report = {"clipped": [], "controls": [], "occupied": []}
    blank = Style()
    for row in range(rows):
        cells = [Cell(" ", 1, blank) for _ in range(cols)]
        if row < len(styled):
            runs = styled[row]
            # style per character offset of the plain line
            offsets = []
            for seg, style in runs:
                offsets.extend([style] * len(seg))
            col = 0
            char_pos = 0
            for g, w in segmented[row]:
                style = offsets[char_pos] if char_pos < len(offsets) else blank
                char_pos += len(g)
                if w == 0:
                    if any(ord(c) < 0x20 or 0x7f <= ord(c) < 0xa0 for c in g):
                        report["controls"].append({"row": row, "col": col, "text": g})
                        continue
                    # a zero-width grapheme (combining without base) attaches
                    # to the previous cell; at the line start it is dropped
                    if col > 0 and not cells[col - 1].continuation:
                        cells[col - 1].text += g
                    continue
                if col + w > cols:
                    # past the edge: reported, never painted; the column does
                    # not advance so the occupied count stays truthful
                    report["clipped"].append({"row": row, "col": col, "text": g, "width": w})
                    continue
                cells[col] = Cell(g, w, style)
                for k in range(1, w):
                    cells[col + k] = Cell("", 0, style, continuation=True)
                col += w
            report["occupied"].append(min(col, cols))
        else:
            report["occupied"].append(0)
        grid.append(cells)
    return grid, report


def occupied_text(grid):
    """The visible text per row, one string, continuations skipped."""
    return ["".join(c.text for c in row if not c.continuation) for row in grid]
