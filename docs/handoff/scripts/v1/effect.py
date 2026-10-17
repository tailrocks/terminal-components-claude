# Per V1 root: the interaction effect then (tag page -> tag flow) vs now (approved page -> actual flow), text only, at one size and palette.
"""Per root: the interaction effect then (baseline page -> baseline flow) and now
(approved page -> actual flow), text only, at one size and palette."""
import os, sys, re, difflib
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import ansi

S = os.environ["V1_DIR"]
W = os.getcwd()
src = open(os.path.join(W, "tests/visual_baseline/scenarios/showcase.rs")).read()
page_of = {}
for m in re.finditer(r'Root::new\("([^"]+)",\s*Bin::Showcase,\s*&\[([^\]]*)\]', src):
    args = re.findall(r'"([^"]*)"', m.group(2))
    if "--page" in args:
        page_of[m.group(1)] = args[args.index("--page") + 1]

sel = sys.argv[1]
size = sys.argv[2] if len(sys.argv) > 2 else "120x40"
pal = sys.argv[3] if len(sys.argv) > 3 else "nocolor"
ACT = os.path.join(S, "actual")
roots = [r for r in open(os.path.join(S, "rootnames.txt")).read().split() if sel in r]


def t(p):
    return ansi.text(ansi.parse(p)) if os.path.exists(p) else None


def show(a, b, la, lb):
    if a is None or b is None:
        print(f"  (missing {la if a is None else lb})")
        return
    d = list(difflib.unified_diff(a, b, la, lb, n=0, lineterm=""))
    print("\n".join(d[2:]) if d else "  (no difference)")


for root in roots:
    page = page_of.get(root)
    print("#" * 100)
    print(f"# {root}  page={page}  {size} {pal}")
    pb = t(os.path.join(S, "base/snapshots/showcase/pages", page or "-", size, pal + ".ansi"))
    pn = t(os.path.join(W, "snapshots/showcase/pages", page or "-", size, pal + ".ansi"))
    fb = t(os.path.join(S, "base/snapshots", root, size, pal + ".ansi"))
    fa = t(os.path.join(ACT, root, size, pal + ".ansi"))
    print("## THEN: baseline page -> baseline flow")
    show(pb, fb, "page@base", "flow@base")
    print("## NOW: approved page -> actual flow")
    show(pn, fa, "page@now", "flow@now")
