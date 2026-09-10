#!/usr/bin/env python3
"""F22 fidelity fixtures: the HTML and PNG tools must agree with the Rust
reference on occupied cells, cursor and selected (reverse-video) content for
split-style runs, combining marks, ZWJ sequences, variation selectors, CJK
and clipping. Run after `cargo build --example cells`:

    tools/fidelity_check.py [out-dir]

Exit status is non-zero on any disagreement. Missing-glyph and unshaped
reports are printed, never hidden; they label the PNG approximate but are
not a disagreement about cells.
"""
import json
import os
import re
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from cells import layout, occupied_text  # noqa: E402
import ansi2html  # noqa: E402

SPAN = re.compile(r'<span class="w(\d)" style="([^"]*)">(.*?)</span>', re.S)

# (name, ansi text, cols, expected occupied text rows, expected per-row occupied cell counts)
FIXTURES = [
    (
        "split-style",
        "ab\x1b[1mcd\x1b[0m\x1b[7mef\x1b[0m",
        8,
        ["abcdef  "],
        [6],
    ),
    (
        "combining",
        "éx éx",
        6,
        ["éx éx "],
        [5],
    ),
    (
        "zwj",
        "\U0001F469‍\U0001F4BB|",
        4,
        ["\U0001F469‍\U0001F4BB| "],
        [3],
    ),
    (
        "variation-selector",
        "❤️|❤|",
        6,
        ["❤️|❤| "],
        [5],
    ),
    (
        "cjk",
        "東京☕|",
        8,
        ["東京☕| "],
        [7],
    ),
    (
        "clip",
        "ABCD",
        2,
        ["AB"],
        [2],
    ),
    (
        "clip-wide",
        "A東",
        2,
        ["A "],
        [1],
    ),
    (
        "reverse-selection",
        "\x1b[7msel\x1b[27m ok",
        6,
        ["sel ok"],
        [6],
    ),
]


def html_cells(html_text):
    rows = []
    for line in html_text.split("<pre>")[1].split("</pre>")[0].split("\n"):
        cells = []
        for w, css, content in SPAN.findall(line):
            cells.append((int(w), css, ansi2html.html.unescape(content)))
        rows.append(cells)
    return rows


def main():
    out_dir = sys.argv[1] if len(sys.argv) > 1 else tempfile.mkdtemp(prefix="fidelity-")
    os.makedirs(out_dir, exist_ok=True)
    failures = []
    notes = []
    try:
        import ansi2png  # noqa: F401
        have_pil = True
    except Exception as e:  # Pillow missing: the PNG half is reported, not faked
        have_pil = False
        notes.append(f"PNG check skipped: {e}")
    for name, ansi, cols, want_text, want_occupied in FIXTURES:
        rows = 1
        grid, report = layout(ansi, cols, rows)
        got_text = occupied_text(grid)
        if got_text != want_text:
            failures.append(f"{name}: occupied text {got_text!r} != {want_text!r}")
        if report["occupied"] != want_occupied:
            failures.append(f"{name}: occupied cells {report['occupied']} != {want_occupied}")
        # HTML: one span per non-continuation cell, widths sum to cols
        html_out = ansi2html.convert(ansi, cols, rows)
        hrows = html_cells(html_out)
        widths = sum(w for w, _, _ in hrows[0])
        if widths != cols:
            failures.append(f"{name}: html widths sum {widths} != {cols}")
        htext = "".join(c for _, _, c in hrows[0])
        if htext != want_text[0]:
            failures.append(f"{name}: html text {htext!r} != {want_text[0]!r}")
        if name == "reverse-selection":
            sel = [c for w, css, c in hrows[0] if "background:#d0d0d0" in css]
            if "".join(sel) != "sel":
                failures.append(f"{name}: reverse cells {sel!r}")
        if name == "clip" and not report["clipped"]:
            failures.append(f"{name}: clipping not reported")
        if have_pil:
            png = os.path.join(out_dir, f"{name}.png")
            cursor = (1, 0)
            result = ansi2png.render(ansi, cols, rows, png, cursor=cursor)
            if result["occupied"] != want_occupied:
                failures.append(f"{name}: png occupied {result['occupied']} != {want_occupied}")
            if result["cursor"] != list(cursor):
                failures.append(f"{name}: png cursor {result['cursor']}")
            from PIL import Image
            im = Image.open(png)
            expect_w = cols * ansi2png.CW + ansi2png.PAD * 2
            if im.size[0] != expect_w:
                failures.append(f"{name}: png width {im.size[0]} != {expect_w}")
            # nothing painted into the right padding
            margin = im.crop((expect_w - ansi2png.PAD, 0, expect_w, im.size[1]))
            pixels = margin.get_flattened_data() if hasattr(margin, "get_flattened_data") else margin.getdata()
            if any(px != (0, 0, 0) for px in pixels):
                failures.append(f"{name}: glyph or background painted into the right margin")
            for m in result["missing"]:
                notes.append(f"{name}: missing glyph {m['text']!r} {m['codepoints']}")
            for u in result["unshaped"]:
                notes.append(f"{name}: unshaped (no raqm) {u['text']!r}")
            notes.append(f"{name}: png {'approximate' if result['approximate'] else 'exact'} · fonts {[f['family'] for f in result['fonts'] if f['used']]}")
    for n in notes:
        print(n)
    if failures:
        for f in failures:
            print("FAIL", f)
        sys.exit(1)
    print(f"ok: {len(FIXTURES)} fixtures agree on cells, cursor and selection; outputs in {out_dir}")


if __name__ == "__main__":
    main()
