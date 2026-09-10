#!/usr/bin/env python3
"""Rasterize a tmux ANSI capture to PNG on the shared cell grid.

Usage: ansi2png.py <in.ansi> <out.png> [cols] [rows] [cursor-file]

Cell occupancy, clipping and the cursor come from `cells.py` (the Rust
reference segmentation): a wide grapheme owns exactly two cells, every glyph
is drawn inside its own cell box, and nothing is painted past `cols`. Glyph
coverage is asserted per grapheme against the primary font and the declared
fallbacks; a grapheme no font covers is drawn as a marked box and reported.
A `<out>.fidelity.json` sidecar records the fonts (path, digest), whether
complex shaping was available, and every missing, unshaped, clipped or
control grapheme, so a PNG is labelled approximate when it must be.
"""
import hashlib
import json
import os
import sys

from PIL import Image, ImageDraw, ImageFont, features

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from cells import layout  # noqa: E402

FONT_DIR = os.path.expanduser("~/Library/Fonts")
PRIMARY = {
    "regular": os.path.join(FONT_DIR, "JetBrainsMonoNerdFontMono-Regular.ttf"),
    "bold": os.path.join(FONT_DIR, "JetBrainsMonoNerdFontMono-Bold.ttf"),
    "italic": os.path.join(FONT_DIR, "JetBrainsMonoNerdFontMono-Italic.ttf"),
    "bolditalic": os.path.join(FONT_DIR, "JetBrainsMonoNerdFontMono-BoldItalic.ttf"),
}
MENLO = "/System/Library/Fonts/Menlo.ttc"
# fallbacks in order; each is tried only when the primary lacks a glyph
FALLBACKS = [
    ("cjk", "/System/Library/Fonts/Hiragino Sans GB.ttc", 15, False),
    ("cjk", "/System/Library/Fonts/PingFang.ttc", 15, False),
    ("unicode", "/System/Library/Fonts/Supplemental/Arial Unicode.ttf", 15, False),
    ("emoji", "/System/Library/Fonts/Apple Color Emoji.ttc", 20, True),
]
SIZE = 15
CW, CH = 9, 20  # cell size at 15px
PAD = 12
NOTDEF_PROBE = "\U0010fffd"
JOINERS = {0x200d, 0xfe0e, 0xfe0f}


def hexrgb(h):
    h = h.lstrip("#")
    return tuple(int(h[i:i + 2], 16) for i in (0, 2, 4))


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 16), b""):
            h.update(chunk)
    return h.hexdigest()


class Face:
    """One loaded font with a glyph-coverage probe."""

    def __init__(self, role, path, size, color):
        self.role, self.path, self.size, self.color = role, path, size, color
        self.font = ImageFont.truetype(path, size)
        self.notdef = self._mask(NOTDEF_PROBE)
        self.used = 0

    def _mask(self, text):
        # render into a scratch bitmap: identical bytes to the .notdef probe
        # mean the font has no glyph for the text
        try:
            im = Image.new("L", (self.size * 3, self.size * 2), 0)
            ImageDraw.Draw(im).text((0, 0), text, font=self.font, fill=255)
            return im.tobytes()
        except Exception:  # bitmap strikes may refuse some sizes/texts
            return None

    def covers(self, cp):
        if cp in JOINERS or 0x0300 <= cp <= 0x036f or 0xfe00 <= cp <= 0xfe0f:
            return True
        m = self._mask(chr(cp))
        if m is None:
            return False
        if m == self.notdef:
            return False
        return True

    def meta(self):
        try:
            family, style = self.font.getname()
        except Exception:
            family, style = os.path.basename(self.path), ""
        return {
            "role": self.role,
            "path": self.path,
            "family": family,
            "style": style,
            "size": self.size,
            "sha256": sha256(self.path),
            "used": self.used,
        }


def load_primary(kind):
    try:
        return Face(kind, PRIMARY[kind], SIZE, False)
    except OSError:
        return Face(kind + "(menlo)", MENLO, SIZE, False)


def load_fallbacks():
    faces = []
    for role, path, size, color in FALLBACKS:
        if not os.path.exists(path):
            continue
        try:
            faces.append(Face(role, path, size, color))
        except OSError:
            continue
    return faces


def pick(text, primary, fallbacks, missing_out, row, col):
    cps = [ord(c) for c in text]
    if all(primary.covers(cp) for cp in cps):
        return primary
    for f in fallbacks:
        if all(f.covers(cp) for cp in cps):
            return f
    missing_out.append({"row": row, "col": col, "text": text, "codepoints": [f"U+{cp:04X}" for cp in cps]})
    return None


def render(text, cols, rows, out, default_fg="#d0d0d0", default_bg="#000000", cursor=None):
    grid, report = layout(text, cols, rows)
    primary = {k: load_primary(k) for k in PRIMARY}
    fallbacks = load_fallbacks()
    raqm = bool(features.check("raqm"))
    img = Image.new("RGB", (cols * CW + PAD * 2, rows * CH + PAD * 2), hexrgb(default_bg))
    draw = ImageDraw.Draw(img)
    missing = []
    unshaped = []
    for row_i, row in enumerate(grid):
        for col_i, cell in enumerate(row):
            if cell.continuation:
                continue
            fg, bg = cell.style.colors(default_fg, default_bg)
            fgc, bgc = hexrgb(fg), hexrgb(bg)
            if cell.style.dim:
                fgc = tuple(int(c * 0.6 + b * 0.4) for c, b in zip(fgc, bgc))
            x = PAD + col_i * CW
            y = PAD + row_i * CH
            w = CW * cell.width
            draw.rectangle([x, y, x + w - 1, y + CH - 1], fill=bgc)
            if cell.text.strip() == "":
                if cell.style.underline:
                    draw.line([x, y + CH - 3, x + w - 1, y + CH - 3], fill=fgc)
                continue
            kind = ("bolditalic" if cell.style.bold and cell.style.italic
                    else "bold" if cell.style.bold else "italic" if cell.style.italic else "regular")
            face = pick(cell.text, primary[kind], fallbacks, missing, row_i, col_i)
            # every glyph is drawn into its own cell box and clipped there
            box = Image.new("RGBA", (w, CH), bgc + (255,))
            bd = ImageDraw.Draw(box)
            if face is None:
                bd.rectangle([1, 2, w - 2, CH - 3], outline=fgc)
                bd.line([1, 2, w - 2, CH - 3], fill=fgc)
            else:
                face.used += 1
                if len(cell.text) > 1 and any(ord(c) in JOINERS for c in cell.text) and not raqm:
                    unshaped.append({"row": row_i, "col": col_i, "text": cell.text})
                try:
                    if face.color:
                        bd.text((0, 0), cell.text, font=face.font, embedded_color=True)
                    else:
                        bd.text((0, 1), cell.text, font=face.font, fill=fgc + (255,))
                except Exception:
                    missing.append({"row": row_i, "col": col_i, "text": cell.text, "codepoints": [f"U+{ord(c):04X}" for c in cell.text], "error": "render"})
                    bd.rectangle([1, 2, w - 2, CH - 3], outline=fgc)
            img.paste(box.convert("RGB"), (x, y))
            if cell.style.underline:
                draw.line([x, y + CH - 3, x + w - 1, y + CH - 3], fill=fgc)
    if cursor is not None:
        cx, cy = cursor
        if cx < cols and cy < rows:
            x = PAD + cx * CW
            y = PAD + cy * CH
            draw.rectangle([x, y, x + CW - 1, y + CH - 1], outline=(255, 255, 255), fill=(255, 255, 255))
    img.save(out)
    fidelity = {
        "cols": cols,
        "rows": rows,
        "cell": [CW, CH],
        "raqm": raqm,
        "fonts": [f.meta() for f in list(primary.values()) + fallbacks if f.used or f.role in PRIMARY],
        "missing": missing,
        "unshaped": unshaped,
        "clipped": report["clipped"],
        "controls": report["controls"],
        "occupied": report["occupied"],
        "cursor": list(cursor) if cursor is not None else None,
        "approximate": bool(missing or unshaped or report["clipped"] or report["controls"]),
        "pillow": Image.__version__,
    }
    with open(out + ".fidelity.json", "w", encoding="utf-8") as f:
        json.dump(fidelity, f, ensure_ascii=False, indent=1)
    return fidelity


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
        result = render(f.read(), cols, rows, dst, cursor=cursor)
    if result["approximate"]:
        print(
            f"{dst}: approximate ({len(result['missing'])} missing, {len(result['unshaped'])} unshaped, "
            f"{len(result['clipped'])} clipped, {len(result['controls'])} control)",
            file=sys.stderr,
        )
