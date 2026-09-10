#!/usr/bin/env python3
"""Convert `tmux capture-pane -e -p` output (SGR truecolor) into a standalone HTML page.

Usage: ansi2html.py <in.ansi> <out.html> [cols] [rows]

Every cell comes from the shared grid in `cells.py` (the Rust reference
segmentation), so a wide grapheme spans two cells, padding never counts
scalars, and content past `cols` is clipped and reported on stderr.
"""
import html
import sys

from cells import SGR, OTHER, State, Style, apply, layout  # noqa: F401  (re-exported for callers)


def convert(text: str, cols: int, rows: int, default_fg="#d0d0d0", default_bg="#000000") -> str:
    grid, report = layout(text, cols, rows)
    out = []
    for row in grid:
        buf = []
        for cell in row:
            if cell.continuation:
                continue
            css = cell.style.css(default_fg, default_bg)
            # the span is `width` cells wide whatever the glyph's advance is
            buf.append(f'<span class="w{cell.width}" style="{css}">{html.escape(cell.text)}</span>')
        out.append("".join(buf))
    body = "\n".join(out)
    for c in report["clipped"]:
        print(f"clipped at row {c['row']} col {c['col']}: {c['text']!r}", file=sys.stderr)
    return f"""<!doctype html><html><head><meta charset="utf-8"><title>capture</title>
<style>
html,body{{margin:0;background:#1a1a1a}}
pre{{margin:16px;display:inline-block;font-family:"JetBrainsMono Nerd Font Mono","JetBrains Mono",Menlo,monospace;font-size:14px;line-height:18px;white-space:pre;background:{default_bg}}}
span{{display:inline-block;height:18px;vertical-align:top;overflow:hidden;text-align:left}}
span.w1{{width:1ch}}
span.w2{{width:2ch}}
</style></head><body><pre>{body}</pre></body></html>"""


if __name__ == "__main__":
    src, dst = sys.argv[1], sys.argv[2]
    cols = int(sys.argv[3]) if len(sys.argv) > 3 else 120
    rows = int(sys.argv[4]) if len(sys.argv) > 4 else 40
    with open(src, encoding="utf-8", errors="replace") as f:
        text = f.read()
    with open(dst, "w", encoding="utf-8") as f:
        f.write(convert(text, cols, rows))
