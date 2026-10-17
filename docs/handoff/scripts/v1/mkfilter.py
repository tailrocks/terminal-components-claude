# List the legacy showcase roots (snapshots/showcase/**/*.txt) and write $V1_DIR/rootnames.txt, filter_all.txt and groups/<group>.filter; run from the worktree root.
import os, sys
S = os.environ["V1_DIR"]
W = os.getcwd()
roots = set()
for dp, dn, fn in os.walk(os.path.join(W, "snapshots/showcase")):
    for f in fn:
        if f.endswith(".txt"):
            rel = os.path.relpath(dp, os.path.join(W, "snapshots"))
            roots.add(os.path.dirname(rel))  # strip size
roots = sorted(roots)
open(os.path.join(S, "rootnames.txt"), "w").write("\n".join(roots) + "\n")
def filt(rs):
    return "test(/^(" + "|".join(r.replace("/", "::") for r in rs) + ")::[0-9]+x[0-9]+::/)"
open(os.path.join(S, "filter_all.txt"), "w").write(filt(roots))
# groups
groups = {}
for r in roots:
    p = r.split("/")
    g = p[1] + "/" + p[2] if p[1] == "flows" else p[1]
    groups.setdefault(g, []).append(r)
os.makedirs(os.path.join(S, "groups"), exist_ok=True)
for g, rs in groups.items():
    open(os.path.join(S, "groups", g.replace("/", "_") + ".filter"), "w").write(filt(rs))
print(len(roots), len(groups))
