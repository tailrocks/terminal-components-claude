#!/usr/bin/env python3
"""Decode a tuiscotti oracle .ansi frame into cells and compute the
termrock FNV digest (same algorithm as crates/termrock-test-support/src/digest.rs)."""

import json
import re
import sys

FNV_OFFSET = 0xCBF29CE484222325
FNV_PRIME = 0x0100000001B3
MASK = 0xFFFFFFFFFFFFFFFF


def fnv(h, data: bytes):
    for b in data:
        h ^= b
        h = (h * FNV_PRIME) & MASK
    return h


NAMED = {
    30: "Black", 31: "Red", 32: "Green", 33: "Yellow", 34: "Blue",
    35: "Magenta", 36: "Cyan", 37: "Gray", 39: "Reset",
    90: "DarkGray", 91: "LightRed", 92: "LightGreen", 93: "LightYellow",
    94: "LightBlue", 95: "LightMagenta", 96: "LightCyan", 97: "White",
}
NAMED_BG = {k - 10: v for k, v in NAMED.items() if k != 39}
NAMED_BG[49] = "Reset"

BOLD, DIM, ITALIC, UNDERLINED = 1, 2, 4, 8
REVERSED, HIDDEN, CROSSED_OUT = 64, 128, 256


class Cell:
    __slots__ = ("symbol", "fg", "bg", "mod")

    def __init__(self):
        self.symbol = " "
        self.fg = "Reset"
        self.bg = "Reset"
        self.mod = 0


def parse_ansi(text, width, height):
    grid = [[Cell() for _ in range(width)] for _ in range(height)]
    fg = ("named", "Reset")
    bg = ("named", "Reset")
    mod = 0
    x = 0
    y = 0
    i = 0
    n = len(text)
    while i < n:
        ch = text[i]
        if ch == "\x1b":
            m = re.match(r"\x1b\[([0-9;?]*)([A-Za-z])", text[i:])
            if not m:
                i += 1
                continue
            params, final = m.group(1), m.group(2)
            i += m.end()
            if final == "m":
                codes = [int(p) if p else 0 for p in params.split(";")] if params else [0]
                j = 0
                while j < len(codes):
                    c = codes[j]
                    if c == 0:
                        fg = ("named", "Reset"); bg = ("named", "Reset"); mod = 0
                    elif c == 1:
                        mod |= BOLD
                    elif c == 2:
                        mod |= DIM
                    elif c == 3:
                        mod |= ITALIC
                    elif c == 4:
                        mod |= UNDERLINED
                    elif c == 7:
                        mod |= REVERSED
                    elif c == 8:
                        mod |= HIDDEN
                    elif c == 9:
                        mod |= CROSSED_OUT
                    elif c == 22:
                        mod &= ~(BOLD | DIM)
                    elif c == 23:
                        mod &= ~ITALIC
                    elif c == 24:
                        mod &= ~UNDERLINED
                    elif c == 27:
                        mod &= ~REVERSED
                    elif c == 28:
                        mod &= ~HIDDEN
                    elif c == 29:
                        mod &= ~CROSSED_OUT
                    elif c == 39:
                        fg = ("named", "Reset")
                    elif c == 49:
                        bg = ("named", "Reset")
                    elif c in NAMED:
                        fg = ("named", NAMED[c])
                    elif c in NAMED_BG:
                        bg = ("named", NAMED_BG[c])
                    elif c in (38, 48):
                        target = "fg" if c == 38 else "bg"
                        if j + 1 < len(codes) and codes[j + 1] == 2:
                            val = ("rgb", tuple(codes[j + 2:j + 5]))
                            j += 4
                        elif j + 1 < len(codes) and codes[j + 1] == 5:
                            val = ("idx", codes[j + 2])
                            j += 2
                        else:
                            j += 1
                            continue
                        if target == "fg":
                            fg = val
                        else:
                            bg = val
                    j += 1
            elif final == "H":
                nums = [int(p) if p else 1 for p in params.split(";")] if params else [1, 1]
                y = (nums[0] if nums else 1) - 1
                x = (nums[1] if len(nums) > 1 else 1) - 1
            elif final == "J":
                for yy in range(y, height):
                    for xx in range(x, width):
                        grid[yy][xx] = Cell()
            elif final == "K":
                for xx in range(x, width):
                    grid[y][xx] = Cell()
            elif final == "C":
                x += int(params) if params else 1
            continue
        if ch == "\n":
            y += 1
            x = 0
            i += 1
            continue
        if ch == "\r":
            x = 0
            i += 1
            continue
        if 0 <= y < height and 0 <= x < width:
            cell = grid[y][x]
            cell.symbol = ch
            cell.fg = color_debug(fg)
            cell.bg = color_debug(bg)
            cell.mod = mod
        x += 1
        i += 1
    return grid


def color_debug(val):
    kind, payload = val
    if kind == "named":
        return payload
    if kind == "rgb":
        return "Rgb(%d, %d, %d)" % payload
    return "Indexed(%d)" % payload


def digest(grid):
    h = FNV_OFFSET
    for row in grid:
        for cell in row:
            h = fnv(h, cell.symbol.encode("utf-8"))
            h = fnv(h, ("%s|%s|%d" % (cell.fg, cell.bg, cell.mod)).encode("utf-8"))
    return "%016x" % h


def main():
    path = sys.argv[1]
    width = int(sys.argv[2])
    height = int(sys.argv[3])
    text = open(path, "rb").read().decode("utf-8", "replace")
    grid = parse_ansi(text, width, height)
    print(json.dumps({
        "path": path,
        "digest": digest(grid),
        "text": "\n".join("".join(c.symbol for c in row) for row in grid),
    }, ensure_ascii=False))


if __name__ == "__main__":
    main()
