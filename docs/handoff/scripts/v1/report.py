# Per V1 root: identical / style-only / text changes of actual captures vs the visual-baseline tag, palette-dependent text and fg==bg cells; writes $V1_DIR/rep and summary JSON.
import os, sys, difflib, re, json
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import ansi

S = os.environ["V1_DIR"]
SIZES = ["72x20", "80x24", "100x30", "120x40", "160x50"]
PALS = ["truecolor", "256", "16", "none", "nocolor"]
LABEL = re.compile(r"(truecolor|256 colors|16 colors|no color|nocolor|none)\s*·\s*\d+×\d+")
ACT = sys.argv[2] if len(sys.argv) > 2 else os.path.join(S, "actual")

roots = [r for r in open(os.path.join(S, "rootnames.txt")).read().split() if r]
sel = sys.argv[1] if len(sys.argv) > 1 else ""
roots = [r for r in roots if sel in r]
out = os.path.join(S, "rep")
summary = {}
for root in roots:
    st = {"missing": 0, "identical": 0, "style": 0, "text": 0, "paldep": [], "fgbg": []}
    for size in SIZES:
        texts = {}
        for pal in PALS:
            b = os.path.join(S, "base/snapshots", root, size, pal + ".ansi")
            a = os.path.join(ACT, root, size, pal + ".ansi")
            if not os.path.exists(a):
                st["missing"] += 1
                continue
            ra, rb = ansi.parse(a), ansi.parse(b)
            ta, tb = ansi.text(ra), ansi.text(rb)
            texts[pal] = [LABEL.sub("LABEL", l) for l in ta]
            if open(a, "rb").read().replace(b"\x1b[?25l", b"") == open(b, "rb").read().replace(b"\x1b[?25l", b""):
                st["identical"] += 1
            elif ta == tb:
                st["style"] += 1
            else:
                st["text"] += 1
            if pal in ("truecolor", "256", "16"):
                for y, row in enumerate(ra):
                    for x, c in enumerate(row):
                        if c[0] != " " and c[1] is not None and c[1] == c[2]:
                            st["fgbg"].append(f"{size}/{pal} ({x},{y}) {c[0]!r}")
            if pal == "truecolor":
                d = os.path.join(out, root)
                os.makedirs(d, exist_ok=True)
                diff = difflib.unified_diff(tb, ta, "baseline", "actual", n=1, lineterm="")
                open(os.path.join(d, size + ".diff"), "w").write("\n".join(diff) + "\n")
        if texts:
            ref = texts.get("truecolor") or next(iter(texts.values()))
            for pal, t in texts.items():
                if t != ref:
                    st["paldep"].append(f"{size}/{pal}")
    summary[root] = st
    print(f"{root:55s} id={st['identical']:2d} style={st['style']:2d} text={st['text']:2d} miss={st['missing']:2d} paldep={len(st['paldep'])} fgbg={len(st['fgbg'])}")
json.dump(summary, open(os.path.join(S, "summary-" + (sel.replace('/', '_') or "all") + ".json"), "w"), indent=1)
