#!/usr/bin/env python3
"""Rasterize a tmux ANSI capture to PNG using JetBrains Mono (falls back to Menlo)."""
import os
import re
import sys

from PIL import Image, ImageDraw, ImageFont

sys.path.insert(0, os.path.dirname(__file__))
from ansi2html import SGR, OTHER, State, apply  # noqa: E402

FONT_DIR = os.path.expanduser("~/Library/Fonts")
FONTS = {
    "regular": os.path.join(FONT_DIR, "JetBrainsMonoNerdFontMono-Regular.ttf"),
    "bold": os.path.join(FONT_DIR, "JetBrainsMonoNerdFontMono-Bold.ttf"),
    "italic": os.path.join(FONT_DIR, "JetBrainsMonoNerdFontMono-Italic.ttf"),
    "bolditalic": os.path.join(FONT_DIR, "JetBrainsMonoNerdFontMono-BoldItalic.ttf"),
}
SIZE = 15
CW, CH = 9, 20  # cell size at 15px
PAD = 12


def hexrgb(h):
    h = h.lstrip("#")
    return tuple(int(h[i:i + 2], 16) for i in (0, 2, 4))


def load(kind):
    try:
        return ImageFont.truetype(FONTS[kind], SIZE)
    except OSError:
        return ImageFont.truetype("/System/Library/Fonts/Menlo.ttc", SIZE)


def wcwidth(ch):
    o = ord(ch)
    if o == 0:
        return 0
    if (0x1100 <= o <= 0x115F or 0x2E80 <= o <= 0xA4CF or 0xAC00 <= o <= 0xD7A3
            or 0xF900 <= o <= 0xFAFF or 0xFE30 <= o <= 0xFE4F or 0xFF00 <= o <= 0xFF60
            or 0xFFE0 <= o <= 0xFFE6 or 0x1F300 <= o <= 0x1F64F or 0x1F900 <= o <= 0x1F9FF):
        return 2
    return 1


def render(text, cols, rows, out, default_fg="#d0d0d0", default_bg="#000000", cursor=None):
    fonts = {k: load(k) for k in FONTS}
    img = Image.new("RGB", (cols * CW + PAD * 2, rows * CH + PAD * 2), hexrgb(default_bg))
    draw = ImageDraw.Draw(img)
    lines = text.split("\n")[:rows]
    state = State()  # SGR state carries across lines in tmux captures
    for row, line in enumerate(lines):
        col = 0
        pos = 0
        line = OTHER.sub(lambda m: m.group(0) if m.group(0).endswith("m") else "", line)
        segments = []
        for m in SGR.finditer(line):
            seg = line[pos:m.start()]
            if seg:
                segments.append((seg, state.css(default_fg, default_bg), state.bold, state.italic, state.underline, state.dim, state.fg, state.bg, state.reverse))
            apply(state, m.group(1))
            pos = m.end()
        seg = line[pos:]
        if seg:
            segments.append((seg, None, state.bold, state.italic, state.underline, state.dim, state.fg, state.bg, state.reverse))
        for seg, _, bold, italic, underline, dim, fg, bg, reverse in segments:
            fg = fg or default_fg
            bg = bg or default_bg
            if reverse:
                fg, bg = bg, fg
            fgc = hexrgb(fg)
            bgc = hexrgb(bg)
            if dim:
                fgc = tuple(int(c * 0.6 + b * 0.4) for c, b in zip(fgc, bgc))
            kind = "bolditalic" if bold and italic else "bold" if bold else "italic" if italic else "regular"
            font = fonts[kind]
            for ch in seg:
                w = wcwidth(ch)
                x = PAD + col * CW
                y = PAD + row * CH
                draw.rectangle([x, y, x + CW * max(w, 1) - 1, y + CH - 1], fill=bgc)
                if ch != " ":
                    draw.text((x, y + 1), ch, font=font, fill=fgc)
                if underline:
                    draw.line([x, y + CH - 3, x + CW * max(w, 1) - 1, y + CH - 3], fill=fgc)
                col += w
    if cursor is not None:
        cx, cy = cursor
        x = PAD + cx * CW
        y = PAD + cy * CH
        draw.rectangle([x, y, x + CW - 1, y + CH - 1], outline=(255, 255, 255), fill=(255, 255, 255))
    img.save(out)


if __name__ == "__main__":
    src, dst = sys.argv[1], sys.argv[2]
    cols = int(sys.argv[3]) if len(sys.argv) > 3 else 120
    rows = int(sys.argv[4]) if len(sys.argv) > 4 else 40
    cursor = None
    if len(sys.argv) > 5 and os.path.exists(sys.argv[5]):
        parts = open(sys.argv[5]).read().split()
        if len(parts) == 3 and parts[2] == "1":
            cursor = (int(parts[0]), int(parts[1]))
    with open(src, encoding="utf-8", errors="replace") as f:
        render(f.read(), cols, rows, dst, cursor=cursor)
