#!/usr/bin/env bash
#
# phase2-scope-check.sh -- phase-2 testing-only scope gate.
#
# Adapted for the termrock workspace (crates/*) layout from /tmp/phase2-scopecheck.sh:
# allowlist verified against this tree (baselines/tuiscotti-v1, .config/nextest.toml,
# .velnor/config.toml, docs/testing, crates/termrock-test-support all exist; root
# tests/* and snapshots/tuiscotti/* have no match here and are retained harmlessly).
#
# Fails on any change that is not a test change. Compares the worktree
# (or the index with --staged) against BASE and checks changed PATHS
# plus in-file SECTIONS. Exit 0 pass, 1 violation, 2 error.
#
# Allowed paths (whole file):
#   crates/*/tests/*, tests/*, snapshots/tuiscotti/*,
#   baselines/tuiscotti-v1/*, docs/testing/*,
#   .config/nextest.toml, crates/termrock-test-support/*,
#   scripts/phase2-scope-check.sh
#
# Section-checked paths:
#   */src/*.rs ............ only hunks inside #[cfg(test)] modules
#   */Cargo.toml ........... only [dev-dependencies] / *test* /
#                            *tuiscotti* sections; prod dep graph
#                            must not change
#   */Cargo.lock ........... only dev-only package blocks; prod dep
#                            graph must not change
#   .github/workflows/*.yml  only test-selection lines inside jobs
#                            whose id contains "test"
#   .velnor/config.toml .... only [test_sharding] and stacks.rust
#                            test keys (test_runner, run_ignored)
#
# All other paths fail, including *.md outside docs/testing
# (docs test-evidence only) and non-src *.rs (examples, benches).
#
# Usage:
#   phase2-scope-check.sh [--base REF] [BASE]
#   phase2-scope-check.sh --staged
#   phase2-scope-check.sh --self-test
#   phase2-scope-check.sh --help

set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  phase2-scope-check.sh [--base REF] [BASE]   check worktree against BASE
  phase2-scope-check.sh --staged              check staged changes vs HEAD
  phase2-scope-check.sh --self-test           run built-in self tests
  phase2-scope-check.sh --help                show this text

Default BASE is tag "visual-baseline" when present, else HEAD.
Exit 0: all changes are test-only. Exit 1: scope violation. Exit 2: error.
EOF
}

die() {
  echo "error: $1" >&2
  exit 2
}

BASE_ARG=""
SELF_TEST=0
STAGED=0
while [ $# -gt 0 ]; do
  case "$1" in
    --base)
      [ $# -ge 2 ] || die "--base needs a value"
      BASE_ARG="$2"
      shift 2
      ;;
    --base=*)
      [ -n "${1#--base=}" ] || die "--base needs a value"
      BASE_ARG="${1#--base=}"
      shift
      ;;
    --staged)
      STAGED=1
      shift
      ;;
    --self-test)
      SELF_TEST=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    --)
      shift
      [ $# -le 1 ] || die "only one BASE allowed"
      if [ $# -eq 1 ]; then
        [ -z "$BASE_ARG" ] || die "only one BASE allowed"
        BASE_ARG="$1"
        shift
      fi
      break
      ;;
    -*)
      die "unknown option: $1"
      ;;
    *)
      [ -z "$BASE_ARG" ] || die "only one BASE allowed"
      BASE_ARG="$1"
      shift
      ;;
  esac
done

if [ "$STAGED" -eq 1 ] && [ -n "$BASE_ARG" ]; then
  die "--staged cannot be combined with --base"
fi

SELF_DIR="$(dirname -- "$0")"
SELF="$(cd -- "$SELF_DIR" >/dev/null 2>&1 && pwd -P)/$(basename -- "$0")" || die "cannot resolve self path"

TMPD="$(mktemp -d "${TMPDIR:-/tmp}/phase2-scope-check.XXXXXX")" || die "cannot create temp dir"
trap 'rm -rf "$TMPD"' EXIT INT TERM
HELPER="$TMPD/helper.py"

cat >"$HELPER" <<'PYEOF'
import json
import os
import re
import subprocess
import sys


def eprint(msg):
    sys.stderr.write(msg + "\n")


def read_text(path):
    with open(path, "r", encoding="utf-8", errors="replace") as f:
        return f.read()


def is_blank(s):
    return s.strip() == ""


HUNK_RE = re.compile(r"^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@")

# --- diff hunks ---

def parse_hunks(diff_text):
    added = set()
    removed = set()
    for line in diff_text.splitlines():
        m = HUNK_RE.match(line)
        if not m:
            continue
        old_start = int(m.group(1))
        old_count = int(m.group(2)) if m.group(2) is not None else 1
        new_start = int(m.group(3))
        new_count = int(m.group(4)) if m.group(4) is not None else 1
        for ln in range(old_start, old_start + old_count):
            if ln >= 1:
                removed.add(ln)
        for ln in range(new_start, new_start + new_count):
            if ln >= 1:
                added.add(ln)
    return added, removed


def report(label, viols, ok_msg):
    if viols:
        for v in viols[:10]:
            print("FAIL  %s: %s" % (label, v))
        if len(viols) > 10:
            print("FAIL  %s: ... and %d more" % (label, len(viols) - 10))
        return 1
    print("PASS  %s (%s)" % (label, ok_msg))
    return 0


# --- rust cfg(test) ranges ---

CFG_RE = re.compile(r"#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]")
INNER_CFG_RE = re.compile(r"#\s*!\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]")
MOD_RE = re.compile(r"\bmod\s+[A-Za-z_][A-Za-z0-9_]*")
CHAR_RE = re.compile(r"'(?:\\.|[^'\\\n])'")


def strip_code(text):
    # Blank comments, strings, char literals; keep layout.
    out = []
    i = 0
    n = len(text)
    while i < n:
        ch = text[i]
        nxt = text[i + 1] if i + 1 < n else ""
        if ch == "/" and nxt == "/":
            while i < n and text[i] != "\n":
                out.append(" ")
                i += 1
        elif ch == "/" and nxt == "*":
            out.append("  ")
            i += 2
            depth = 1
            while i < n and depth > 0:
                two = text[i:i + 2]
                if two == "/*":
                    depth += 1
                    out.append("  ")
                    i += 2
                elif two == "*/":
                    depth -= 1
                    out.append("  ")
                    i += 2
                else:
                    out.append("\n" if text[i] == "\n" else " ")
                    i += 1
        elif ch == '"':
            out.append(" ")
            i += 1
            while i < n:
                if text[i] == "\\" and i + 1 < n:
                    out.append("  ")
                    i += 2
                    continue
                if text[i] == '"':
                    out.append(" ")
                    i += 1
                    break
                out.append("\n" if text[i] == "\n" else " ")
                i += 1
        elif ch == "'":
            m = CHAR_RE.match(text, i)
            if m:
                out.append(" " * len(m.group(0)))
                i += len(m.group(0))
            else:
                out.append(ch)
                i += 1
        else:
            out.append(ch)
            i += 1
    return "".join(out)


def mod_end(clean_lines, start):
    depth = 0
    seen_open = False
    ln = start
    for ch in "\n".join(clean_lines[start - 1:]):
        if ch == "\n":
            ln += 1
            continue
        if not seen_open:
            if ch == "{":
                seen_open = True
                depth = 1
            elif ch == ";":
                return None
        elif ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                return ln
    return None


def rust_test_ranges(lines):
    for ln in lines:
        if INNER_CFG_RE.search(ln):
            return [(1, len(lines))]
    clean = strip_code("\n".join(lines)).split("\n")
    ranges = []
    total = len(lines)
    for i in range(1, total + 1):
        if not CFG_RE.search(clean[i - 1]):
            continue
        j = i
        while j <= total:
            cand = clean[j - 1]
            if MOD_RE.search(cand):
                end = mod_end(clean, j)
                if end is not None:
                    ranges.append((i, end))
                break
            s = cand.strip()
            if s == "" or s.startswith("#"):
                j += 1
                continue
            break
    return ranges


def in_ranges(ln, ranges):
    return any(a <= ln <= b for a, b in ranges)


def cmd_rust(old_p, new_p, diff_p, label):
    old = read_text(old_p).splitlines()
    new = read_text(new_p).splitlines()
    added, removed = parse_hunks(read_text(diff_p))
    old_r = rust_test_ranges(old)
    new_r = rust_test_ranges(new)
    viols = []
    for ln in sorted(removed):
        if ln > len(old) or is_blank(old[ln - 1]):
            continue
        if not in_ranges(ln, old_r):
            viols.append("removed line %d is outside #[cfg(test)]" % ln)
    for ln in sorted(added):
        if ln > len(new) or is_blank(new[ln - 1]):
            continue
        if not in_ranges(ln, new_r):
            viols.append("added line %d is outside #[cfg(test)]" % ln)
    return report(label, viols, "only #[cfg(test)] hunks")


# --- toml section checks ---

HDR_RE = re.compile(r"^\s*\[{1,2}\s*([^\[\]]+?)\s*\]{1,2}\s*(?:#.*)?$")


def toml_headers(lines):
    cur = None
    out = []
    for ln in lines:
        m = HDR_RE.match(ln)
        if m:
            cur = m.group(1).strip()
        out.append(cur)
    return out


def cmd_toml(old_p, new_p, diff_p, label, allowed_csv):
    allowed = [a.strip().lower() for a in allowed_csv.split(",") if a.strip()]

    def ok(header):
        return header is not None and any(k in header.lower() for k in allowed)

    old = read_text(old_p).splitlines()
    new = read_text(new_p).splitlines()
    added, removed = parse_hunks(read_text(diff_p))
    old_h = toml_headers(old)
    new_h = toml_headers(new)
    viols = []
    for ln in sorted(removed):
        if ln > len(old) or is_blank(old[ln - 1]):
            continue
        if not ok(old_h[ln - 1]):
            viols.append("removed line %d under [%s]" % (ln, old_h[ln - 1] or "no section"))
    for ln in sorted(added):
        if ln > len(new) or is_blank(new[ln - 1]):
            continue
        if not ok(new_h[ln - 1]):
            viols.append("added line %d under [%s]" % (ln, new_h[ln - 1] or "no section"))
    return report(label, viols, "only test sections")


VELNOR_KEY_RE = re.compile(r"^\s*(test_runner|run_ignored)\s*=")


def cmd_velnor(old_p, new_p, diff_p, label):
    def ok(header, line):
        if header is None:
            return False
        h = header.lower()
        if "test_sharding" in h:
            return True
        if h == "stacks.rust" and VELNOR_KEY_RE.match(line):
            return True
        return False

    old = read_text(old_p).splitlines()
    new = read_text(new_p).splitlines()
    added, removed = parse_hunks(read_text(diff_p))
    old_h = toml_headers(old)
    new_h = toml_headers(new)
    viols = []
    for ln in sorted(removed):
        if ln > len(old) or is_blank(old[ln - 1]):
            continue
        if not ok(old_h[ln - 1], old[ln - 1]):
            viols.append("removed line %d outside test-selection keys" % ln)
    for ln in sorted(added):
        if ln > len(new) or is_blank(new[ln - 1]):
            continue
        if not ok(new_h[ln - 1], new[ln - 1]):
            viols.append("added line %d outside test-selection keys" % ln)
    return report(label, viols, "only test-selection keys")


# --- cargo lock dev-only check ---

PKG_RE = re.compile(r"^\s*\[\[package\]\]\s*(?:#.*)?$")
NAME_RE = re.compile(r'^\s*name\s*=\s*"([^"]+)"')
DEPS_RE = re.compile(r"^\s*dependencies\s*=")


def lock_blocks(lines):
    blocks = []
    cur = None
    for i, ln in enumerate(lines, 1):
        if PKG_RE.match(ln):
            if cur is not None:
                blocks.append((cur[0], cur[1], i - 1, cur[2]))
            cur = [None, i, None]
        elif cur is not None:
            m = NAME_RE.match(ln)
            if m and cur[0] is None:
                cur[0] = m.group(1)
            if DEPS_RE.match(ln):
                depth = 0
                seen = False
                j = i
                span_end = i
                while j <= len(lines):
                    for ch in lines[j - 1].split("#", 1)[0]:
                        if ch == "[":
                            depth += 1
                            seen = True
                        elif ch == "]":
                            depth -= 1
                    span_end = j
                    if seen and depth <= 0:
                        break
                    j += 1
                cur[2] = (i, span_end)
    if cur is not None:
        blocks.append((cur[0], cur[1], len(lines), cur[2]))
    return blocks


def lock_find(blocks, ln):
    for b in blocks:
        if b[1] <= ln <= b[2]:
            return b
    return None


def cmd_lock(old_p, new_p, diff_p, closure_p, members_p, label):
    closure = set(s.strip() for s in read_text(closure_p).splitlines() if s.strip())
    members = set(s.strip() for s in read_text(members_p).splitlines() if s.strip())
    old = read_text(old_p).splitlines()
    new = read_text(new_p).splitlines()
    added, removed = parse_hunks(read_text(diff_p))
    old_b = lock_blocks(old)
    new_b = lock_blocks(new)
    viols = []

    def check(lns, blocks, lines, kind):
        for ln in sorted(lns):
            if ln > len(lines) or is_blank(lines[ln - 1]):
                continue
            b = lock_find(blocks, ln)
            if b is None:
                viols.append("%s line %d is outside [[package]]" % (kind, ln))
                continue
            name, _, _, span = b
            if name in members:
                if span is None or not (span[0] <= ln <= span[1]):
                    viols.append("%s line %d touches member '%s' outside its dependencies list" % (kind, ln, name))
                continue
            if name is None or name in closure:
                viols.append("%s line %d touches prod package '%s'" % (kind, ln, name or "?"))

    check(removed, old_b, old, "removed")
    check(added, new_b, new, "added")
    return report(label, viols, "only dev-only package blocks")


# --- workflow test-selection check ---

SELECT_RE = re.compile(
    r"nextest|cargo\s+test|run.ignored|\s-E(\s|=|$)|filter|shard"
    r"|partition|test.group|\bprofile\b|\bignored\b|\btests?\b", re.IGNORECASE)
JOB_ID_RE = re.compile(r"^([A-Za-z0-9_.\-]+)\s*:")


def indent_of(s):
    return len(s) - len(s.lstrip(" "))


def wf_blocks(lines):
    jobs_idx = None
    for i, ln in enumerate(lines, 1):
        s = ln.strip()
        if s == "" or s.startswith("#"):
            continue
        if indent_of(ln) == 0 and re.match(r"^jobs\s*:\s*(?:#.*)?$", ln):
            jobs_idx = i
            break
    if jobs_idx is None:
        return []
    child_ind = None
    for j in range(jobs_idx + 1, len(lines) + 1):
        s = lines[j - 1].strip()
        if s == "" or s.startswith("#"):
            continue
        if indent_of(lines[j - 1]) == 0:
            break
        child_ind = indent_of(lines[j - 1])
        break
    if child_ind is None:
        return []
    blocks = []
    cur_id = None
    cur_start = None
    for j in range(jobs_idx + 1, len(lines) + 1):
        raw = lines[j - 1]
        s = raw.strip()
        if s == "" or s.startswith("#"):
            continue
        ind = indent_of(raw)
        if ind == 0:
            break
        if ind == child_ind:
            m = JOB_ID_RE.match(s)
            if m:
                if cur_id is not None:
                    blocks.append((cur_id, cur_start, j - 1))
                cur_id = m.group(1)
                cur_start = j
    if cur_id is not None:
        end = len(lines)
        for j in range(cur_start + 1, len(lines) + 1):
            s = lines[j - 1].strip()
            if s == "" or s.startswith("#"):
                continue
            if indent_of(lines[j - 1]) == 0:
                end = j - 1
                break
        blocks.append((cur_id, cur_start, end))
    return blocks


def cmd_workflow(old_p, new_p, diff_p, label):
    old = read_text(old_p).splitlines()
    new = read_text(new_p).splitlines()
    added, removed = parse_hunks(read_text(diff_p))
    old_b = wf_blocks(old)
    new_b = wf_blocks(new)
    viols = []

    def check(lns, blocks, lines, kind):
        for ln in sorted(lns):
            if ln > len(lines) or is_blank(lines[ln - 1]):
                continue
            hit = [b for b in blocks if b[1] <= ln <= b[2]]
            if not hit:
                viols.append("%s line %d is outside jobs" % (kind, ln))
            elif "test" not in hit[0][0].lower():
                viols.append("%s line %d is in non-test job '%s'" % (kind, ln, hit[0][0]))
            elif not SELECT_RE.search(lines[ln - 1]):
                viols.append("%s line %d is not a test-selection line" % (kind, ln))

    check(removed, old_b, old, "removed")
    check(added, new_b, new, "added")
    return report(label, viols, "only test-selection hunks")


# --- resolved production dep graph ---

def run_metadata(manifest):
    cmd = ["cargo", "metadata", "--locked", "--offline", "--format-version", "1",
           "--manifest-path", manifest]
    env = dict(os.environ, CARGO_NET_OFFLINE="true")
    try:
        p = subprocess.run(cmd, capture_output=True, text=True, env=env)
    except OSError as ex:
        return None, str(ex)
    if p.returncode != 0:
        return None, (p.stderr.strip() or p.stdout.strip() or "exit %d" % p.returncode)
    try:
        return json.loads(p.stdout), None
    except ValueError as ex:
        return None, "bad JSON: %s" % ex


def normal_graph(meta):
    pkgs = {}
    for p in meta.get("packages", []):
        pkgs[p["id"]] = (p["name"], p.get("version", ""), str(p.get("source") or ""))
    resolve = meta.get("resolve") or {}
    nodes = {}
    for n in resolve.get("nodes", []):
        nodes[n["id"]] = n.get("deps", [])
    edges = {}
    adj = {}
    for pid, (name, ver, src) in pkgs.items():
        eds = set()
        succ = []
        for d in nodes.get(pid, []):
            kinds = d.get("dep_kinds") or [{}]
            if not any((k or {}).get("kind") in (None, "") for k in kinds):
                continue
            tgt = pkgs.get(d.get("pkg"), (d.get("name", "?"), "?", ""))
            eds.add((d.get("name", ""), tgt[0], tgt[1], tgt[2]))
            succ.append(d.get("pkg"))
        edges[pid] = sorted(eds)
        adj[pid] = succ
    members = [i for i in (meta.get("workspace_members") or []) if i in pkgs]
    seen = set()
    stack = list(members)
    while stack:
        cur = stack.pop()
        if cur in seen:
            continue
        seen.add(cur)
        stack.extend(x for x in adj.get(cur, []) if x not in seen)
    graph = {}
    for pid in seen:
        name, ver, src = pkgs[pid]
        graph["%s %s %s" % (name, ver, src)] = edges.get(pid, [])
    closure_pkgs = sorted(pkgs[i] for i in seen if i in pkgs)
    closure_names = sorted(set(pkgs[i][0] for i in seen if i in pkgs))
    member_names = sorted(set(pkgs[i][0] for i in members))
    return graph, closure_pkgs, closure_names, member_names


def cmd_metagraph(cur_man, base_man, closure_out, members_out):
    cur, err = run_metadata(cur_man)
    if cur is None:
        print("cargo metadata failed for worktree: %s" % err)
        return 2
    base, err = run_metadata(base_man)
    if base is None:
        print("cargo metadata failed for base tree: %s" % err)
        return 2
    cg, cpkgs, cnames, cmems = normal_graph(cur)
    bg, bpkgs, bnames, bmems = normal_graph(base)
    with open(closure_out, "w", encoding="utf-8") as f:
        for nm in sorted(set(cnames) | set(bnames)):
            f.write(nm + "\n")
    with open(members_out, "w", encoding="utf-8") as f:
        for nm in sorted(set(cmems) | set(bmems)):
            f.write(nm + "\n")
    if cg != bg or cpkgs != bpkgs:
        details = []
        if cpkgs != bpkgs:
            only_cur = sorted(set(cpkgs) - set(bpkgs))[:5]
            only_base = sorted(set(bpkgs) - set(cpkgs))[:5]
            if only_cur:
                details.append("only in worktree: %s" % ", ".join("%s %s" % (n, v) for n, v, s in only_cur))
            if only_base:
                details.append("only in base: %s" % ", ".join("%s %s" % (n, v) for n, v, s in only_base))
        else:
            shown = 0
            for k in sorted(set(cg) | set(bg)):
                if cg.get(k) != bg.get(k) and shown < 3:
                    details.append("edges differ for %s" % k)
                    shown += 1
        print("prod graph differs: %s" % ("; ".join(details) if details else "unknown"))
        return 1
    print("prod dep graph unchanged (%d prod packages)" % len(cpkgs))
    return 0


def main(argv):
    if len(argv) < 2:
        eprint("helper: missing subcommand")
        return 2
    cmd = argv[1]
    try:
        if cmd == "rust":
            return cmd_rust(argv[2], argv[3], argv[4], argv[5])
        if cmd == "toml":
            return cmd_toml(argv[2], argv[3], argv[4], argv[5], argv[6])
        if cmd == "velnor":
            return cmd_velnor(argv[2], argv[3], argv[4], argv[5])
        if cmd == "lock":
            return cmd_lock(argv[2], argv[3], argv[4], argv[5], argv[6], argv[7])
        if cmd == "workflow":
            return cmd_workflow(argv[2], argv[3], argv[4], argv[5])
        if cmd == "metagraph":
            return cmd_metagraph(argv[2], argv[3], argv[4], argv[5])
    except (IOError, OSError, IndexError) as ex:
        eprint("helper %s failed: %s" % (cmd, ex))
        return 2
    eprint("helper: unknown subcommand: %s" % cmd)
    return 2


sys.exit(main(sys.argv))
PYEOF

BASE=""
TOPLEVEL=""
CHECKED=0
FAILED=0
NEED_META=0
DIFF_OLD=""
DIFF_NEW=""
DIFF_TXT=""
F_OLD_EXISTS=0
F_SKIP=0

prep_diff() {
  DIFF_OLD="$TMPD/f.old"
  DIFF_NEW="$TMPD/f.new"
  DIFF_TXT="$TMPD/f.diff"
  F_OLD_EXISTS=0
  F_SKIP=0
  if [ "$STAGED" -eq 1 ]; then
    git cat-file -e "HEAD:$1" 2>/dev/null && { git show "HEAD:$1" >"$DIFF_OLD"; F_OLD_EXISTS=1; } || : >"$DIFF_OLD"
    if git cat-file -e ":$1" 2>/dev/null; then
      git show ":$1" >"$DIFF_NEW" || { F_SKIP=1; : >"$DIFF_NEW"; : >"$DIFF_TXT"; return 0; }
    else
      : >"$DIFF_NEW"
    fi
    git diff --cached -U0 --no-renames HEAD -- "$1" >"$DIFF_TXT" || true
  else
    if git cat-file -e "$BASE:$1" 2>/dev/null; then
      git show "$BASE:$1" >"$DIFF_OLD"
      F_OLD_EXISTS=1
    else
      : >"$DIFF_OLD"
    fi
    if [ -f "$1" ]; then
      if ! cp -- "$1" "$DIFF_NEW" 2>/dev/null; then
        F_SKIP=1
        : >"$DIFF_NEW"
        : >"$DIFF_TXT"
        return 0
      fi
    else
      : >"$DIFF_NEW"
    fi
    if [ "$F_OLD_EXISTS" -eq 0 ] && [ ! -f "$1" ]; then
      F_SKIP=1
      : >"$DIFF_TXT"
      return 0
    fi
    if [ "$F_OLD_EXISTS" -eq 1 ]; then
      git diff -U0 --no-renames "$BASE" -- "$1" >"$DIFF_TXT" || true
    else
      git diff --no-index -U0 -- /dev/null "$1" >"$DIFF_TXT" 2>/dev/null || true
    fi
  fi
  if grep -qE '^(old|new) mode' "$DIFF_TXT"; then
    echo "FAIL  $1 (mode change outside test scope)"
    return 1
  fi
  if grep -q '^Binary files ' "$DIFF_TXT"; then
    echo "FAIL  $1 (binary change outside test scope)"
    return 1
  fi
  return 0
}

run_helper() {
  rc=0
  python3 "$HELPER" "$@" || rc=$?
  if [ "$rc" -ne 0 ]; then
    FAILED=$((FAILED + 1))
  fi
  return 0
}

check_one() {
  CHECKED=$((CHECKED + 1))
  if ! prep_diff "$1"; then
    FAILED=$((FAILED + 1))
    return 0
  fi
  if [ "$F_SKIP" -eq 1 ]; then
    echo "SKIP  $1 (vanished during check)"
    return 0
  fi
  return 1
}

check_rust() {
  check_one "$1" || run_helper rust "$DIFF_OLD" "$DIFF_NEW" "$DIFF_TXT" "$1"
}

check_toml() {
  check_one "$1" || run_helper toml "$DIFF_OLD" "$DIFF_NEW" "$DIFF_TXT" "$1" "dev-dependencies,test,tuiscotti"
}

check_velnor() {
  check_one "$1" || run_helper velnor "$DIFF_OLD" "$DIFF_NEW" "$DIFF_TXT" "$1"
}

check_lock_file() {
  check_one "$1" || run_helper lock "$DIFF_OLD" "$DIFF_NEW" "$DIFF_TXT" "$TMPD/closure" "$TMPD/members" "$1"
}

check_workflow() {
  check_one "$1" || run_helper workflow "$DIFF_OLD" "$DIFF_NEW" "$DIFF_TXT" "$1"
}

fail_all_manifests() {
  while IFS= read -r -d '' p; do
    [ -n "$p" ] || continue
    echo "FAIL  $p ($1)"
    FAILED=$((FAILED + 1))
  done <"$TMPD/manifests"
  return 0
}

run_meta_phase() {
  if ! command -v cargo >/dev/null 2>&1; then
    echo "FAIL  prod dep graph unverified: cargo not found (fail closed)"
    fail_all_manifests "cargo not found"
    return 0
  fi
  BASE_TREE="$TMPD/base"
  mkdir -p "$BASE_TREE" || die "cannot create base tree dir"
  if ! git archive "$BASE" | tar -x -C "$BASE_TREE"; then
    echo "error: cannot extract base tree for $BASE" >&2
    exit 2
  fi
  rc=0
  python3 "$HELPER" metagraph "$TOPLEVEL/Cargo.toml" "$BASE_TREE/Cargo.toml" "$TMPD/closure" "$TMPD/members" >"$TMPD/meta.out" 2>&1 || rc=$?
  if [ "$rc" -eq 0 ]; then
    echo "INFO  $(<"$TMPD/meta.out")"
    while IFS= read -r -d '' p; do
      [ -n "$p" ] || continue
      check_lock_file "$p"
    done <"$TMPD/locks"
    return 0
  fi
  if [ "$rc" -eq 2 ]; then
    echo "FAIL  prod dep graph unverified (fail closed): $(<"$TMPD/meta.out")"
    fail_all_manifests "prod graph unverified"
    return 0
  fi
  echo "FAIL  prod dependency graph changed vs $BASE: $(<"$TMPD/meta.out")"
  fail_all_manifests "prod graph changed"
  return 0
}

FXD=""
FXBASE=""
ST_N=0
ST_PASS=0
ST_FAIL=0

mkfixture() {
  rm -rf "$FXD/fx"
  mkdir -p "$FXD/fx" || die "cannot create fixture dir"
  (
    cd "$FXD/fx" || exit 1
    git init -q . || exit 1
    git config user.email "scope-check-test@example.com" || exit 1
    git config user.name "scope-check-test" || exit 1
    git config commit.gpgsign false || exit 1
    git config core.hooksPath /dev/null || exit 1
    mkdir -p src dep-a/src dep-b/src .github/workflows tests docs/testing .config .velnor crates/widget/src crates/widget/tests || exit 1
    cat >src/lib.rs <<'EOF' || exit 1
pub fn prod() -> u32 {
    41
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
EOF
    cp src/lib.rs crates/widget/src/lib.rs || exit 1
    printf '#[test]\nfn smoke() {}\n' >crates/widget/tests/smoke.rs || exit 1
    cat >Cargo.toml <<'EOF' || exit 1
[package]
name = "fixture"
version = "0.1.0"
edition = "2021"

[dependencies]
dep-a = { path = "dep-a" }

[dev-dependencies]
EOF
    cat >dep-a/Cargo.toml <<'EOF' || exit 1
[package]
name = "dep-a"
version = "0.1.0"
edition = "2021"
EOF
    printf 'pub fn a() {}\n' >dep-a/src/lib.rs || exit 1
    cat >dep-b/Cargo.toml <<'EOF' || exit 1
[package]
name = "dep-b"
version = "0.1.0"
edition = "2021"
EOF
    printf 'pub fn b() {}\n' >dep-b/src/lib.rs || exit 1
    cat >.github/workflows/ci.yml <<'EOF' || exit 1
name: ci
on: [push]
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo build
  test-unit:
    runs-on: ubuntu-latest
    steps:
      - run: cargo nextest run --profile ci
EOF
    printf '[profile.ci]\n' >.config/nextest.toml || exit 1
    printf 'schema = 1\n\n[stacks.rust]\ncompile_driver = "cargo"\ntest_runner = "cargo_nextest"\nrun_ignored = "all"\n' >.velnor/config.toml || exit 1
    printf '# fixture\n' >README.md || exit 1
    printf '# evidence\n' >docs/testing/note.md || exit 1
    printf '' >tests/.gitkeep || exit 1
    cargo metadata --format-version 1 --offline >/dev/null || exit 1
    git add -A || exit 1
    git commit -qm base || exit 1
  ) || die "fixture setup failed"
  FXBASE="$(cd "$FXD/fx" && git rev-parse HEAD)" || die "cannot read fixture base"
}

fxreset() {
  (cd "$FXD/fx" && git reset -q --hard "$FXBASE" && git clean -qdff) || die "fixture reset failed"
}

run_case() {
  ST_N=$((ST_N + 1))
  rc=0
  out="$(cd "$FXD/fx" && bash "$SELF" --base "$FXBASE" 2>&1)" || rc=$?
  ok=0
  if [ "$1" = "pass" ] && [ "$rc" -eq 0 ]; then
    ok=1
  fi
  if [ "$1" = "fail" ] && [ "$rc" -ne 0 ]; then
    ok=1
  fi
  if [ "$ok" -eq 1 ]; then
    ST_PASS=$((ST_PASS + 1))
    echo "self-test $ST_N [$1] $2: ok"
  else
    ST_FAIL=$((ST_FAIL + 1))
    echo "self-test $ST_N [$1] $2: NOT OK (exit=$rc)"
    sed 's/^/    /' <<<"$out"
  fi
  return 0
}

run_self_tests() {
  command -v git >/dev/null 2>&1 || die "self-test needs git"
  command -v python3 >/dev/null 2>&1 || die "self-test needs python3"
  command -v cargo >/dev/null 2>&1 || die "self-test needs cargo"
  ST_N=0
  ST_PASS=0
  ST_FAIL=0
  FXD="$(mktemp -d "${TMPDIR:-/tmp}/p2sc-selftest.XXXXXX")" || die "cannot create self-test dir"
  trap 'rm -rf "$FXD" "$TMPD"' EXIT INT TERM
  mkfixture

  run_case pass "clean tree passes"

  fxreset
  printf '#[test]\nfn added() {}\n' >"$FXD/fx/tests/added.rs"
  run_case pass "new file under tests/ passes"

  fxreset
  cat >>"$FXD/fx/src/lib.rs" <<'EOF'

#[cfg(test)]
mod more_tests {
    #[test]
    fn added() {
        assert!(true);
    }
}
EOF
  run_case pass "new cfg(test) module passes"

  fxreset
  printf 'dep-b = { path = "dep-b" }\n' >>"$FXD/fx/Cargo.toml"
  (cd "$FXD/fx" && cargo generate-lockfile --offline >/dev/null) || die "lockfile refresh failed"
  run_case pass "dev-dependency plus lock update passes"

  fxreset
  printf 'fail-fast = false\n' >>"$FXD/fx/.config/nextest.toml"
  run_case pass "nextest config edit passes"

  fxreset
  printf '\n[test_sharding]\ndefault_shards = 2\n' >>"$FXD/fx/.velnor/config.toml"
  run_case pass "velnor test_sharding edit passes"

  fxreset
  python3 - "$FXD/fx/.github/workflows/ci.yml" <<'PY' || die "edit failed"
import sys
p = sys.argv[1]
s = open(p).read()
old = "      - run: cargo nextest run --profile ci\n"
assert old in s
open(p, "w").write(s.replace(old, old + "      - run: cargo nextest run --profile ci -E 'test(audit)'\n", 1))
PY
  run_case pass "test-selection line in test job passes"

  fxreset
  python3 - "$FXD/fx/src/lib.rs" <<'PY' || die "edit failed"
import sys
p = sys.argv[1]
s = open(p).read()
assert "41" in s
open(p, "w").write(s.replace("41", "42", 1))
PY
  run_case fail "prod line change fails"

  fxreset
  python3 - "$FXD/fx/Cargo.toml" <<'PY' || die "edit failed"
import sys
p = sys.argv[1]
s = open(p).read()
old = 'dep-a = { path = "dep-a" }\n'
assert old in s
open(p, "w").write(s.replace(old, old + 'dep-b = { path = "dep-b" }\n', 1))
PY
  run_case fail "dependencies section change fails"

  fxreset
  python3 - "$FXD/fx/.github/workflows/ci.yml" <<'PY' || die "edit failed"
import sys
p = sys.argv[1]
s = open(p).read()
assert "echo build" in s
open(p, "w").write(s.replace("echo build", "echo build2", 1))
PY
  run_case fail "non-test job edit fails"

  fxreset
  python3 - "$FXD/fx/.github/workflows/ci.yml" <<'PY' || die "edit failed"
import sys
p = sys.argv[1]
s = open(p).read()
old = "  test-unit:\n    runs-on: ubuntu-latest\n"
assert old in s
open(p, "w").write(s.replace(old, "  test-unit:\n    runs-on: macos-latest\n", 1))
PY
  run_case fail "non-selection line in test job fails"

  fxreset
  printf 'more docs\n' >>"$FXD/fx/README.md"
  run_case fail "README edit fails"

  fxreset
  printf 'pub fn evil() -> bool {\n    true\n}\n' >"$FXD/fx/src/sneaky.rs"
  run_case fail "new prod src file fails"

  fxreset
  printf 'x = 1\n' >"$FXD/fx/crates/widget/build.rs"
  run_case fail "non-src rs file fails"

  fxreset
  printf '\ncompile_driver = "bazel"\n' >>"$FXD/fx/.velnor/config.toml"
  run_case fail "velnor non-test key fails"

  fxreset
  if [ "$ST_FAIL" -eq 0 ]; then
    echo "self-test: $ST_PASS passed, 0 failed"
    exit 0
  fi
  echo "self-test: $ST_PASS passed, $ST_FAIL FAILED"
  exit 1
}

if [ "$SELF_TEST" -eq 1 ]; then
  run_self_tests
fi

command -v git >/dev/null 2>&1 || die "git not found"
command -v python3 >/dev/null 2>&1 || die "python3 not found"

if [ "$STAGED" -eq 1 ]; then
  BASE="HEAD"
else
  if [ -n "$BASE_ARG" ]; then
    BASE="$BASE_ARG"
  elif git rev-parse --verify --quiet 'refs/tags/visual-baseline^{commit}' >/dev/null; then
    BASE="visual-baseline"
  else
    BASE="HEAD"
  fi
fi
git rev-parse --verify --quiet "$BASE^{commit}" >/dev/null || die "unknown base ref: $BASE"

TOPLEVEL="$(git rev-parse --show-toplevel 2>/dev/null)" || die "not in a git repo"
cd "$TOPLEVEL" || die "cannot enter repo root"

: >"$TMPD/manifests"
: >"$TMPD/locks"
if [ "$STAGED" -eq 1 ]; then
  git diff --cached --name-only -z --no-renames HEAD -- >"$TMPD/paths"
else
  git diff --name-only -z --no-renames "$BASE" -- >"$TMPD/raw1"
  git ls-files --others --exclude-standard -z -- >"$TMPD/raw2"
  LC_ALL=C sort -zu "$TMPD/raw1" "$TMPD/raw2" >"$TMPD/paths"
fi

if [ ! -s "$TMPD/paths" ]; then
  echo "phase2-scope-check: no changes vs $BASE (pass)"
  exit 0
fi

while IFS= read -r -d '' path; do
  if [ -z "$path" ]; then
    continue
  fi
  case "$path" in
    tests/*|crates/*/tests/*|snapshots/tuiscotti/*|baselines/tuiscotti-v1/*|docs/testing/*|.config/nextest.toml|scripts/phase2-scope-check.sh|crates/termrock-test-support/*)
      CHECKED=$((CHECKED + 1))
      echo "PASS  $path (allowlisted path)"
      ;;
    *.rs)
      case "$path" in
        src/*|*/src/*)
          check_rust "$path"
          ;;
        *)
          CHECKED=$((CHECKED + 1))
          echo "FAIL  $path (rust outside src/ is outside test scope)"
          FAILED=$((FAILED + 1))
          ;;
      esac
      ;;
    Cargo.toml|*/Cargo.toml)
      NEED_META=1
      printf '%s\0' "$path" >>"$TMPD/manifests"
      check_toml "$path"
      ;;
    Cargo.lock|*/Cargo.lock)
      NEED_META=1
      printf '%s\0' "$path" >>"$TMPD/manifests"
      printf '%s\0' "$path" >>"$TMPD/locks"
      ;;
    .github/workflows/*.yml|.github/workflows/*.yaml)
      check_workflow "$path"
      ;;
    .velnor/config.toml)
      check_velnor "$path"
      ;;
    *)
      CHECKED=$((CHECKED + 1))
      echo "FAIL  $path (path outside test scope)"
      FAILED=$((FAILED + 1))
      ;;
  esac
done <"$TMPD/paths"

if [ "$NEED_META" -eq 1 ]; then
  run_meta_phase
fi

echo "phase2-scope-check: $CHECKED file(s) vs $BASE, $FAILED violation(s)"
if [ "$FAILED" -eq 0 ]; then
  exit 0
fi
exit 1
