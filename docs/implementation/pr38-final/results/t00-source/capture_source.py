#!/usr/bin/env python3
"""Create read-only source inventory artifacts for the T00 input commit."""
import gzip
import hashlib
import json
import os
import re
import subprocess
import sys
import tomllib
from pathlib import PurePosixPath

REPO = "/Users/donbeave/Downloads/terminal-components-claude"
REV = "3a2c977c881cd1745de16425c982791ee3ef2cc3"
OUT = REPO + "/docs/implementation/pr38-final/results/t00-source"


def git(*args, data=None):
    return subprocess.run(["git", "-C", REPO, *args], input=data, stdout=subprocess.PIPE,
                          stderr=subprocess.PIPE, check=True).stdout


def lang(path):
    p = path.lower()
    name = PurePosixPath(p).name
    ext = PurePosixPath(name).suffix
    table = {".rs":"rust", ".py":"python", ".pyi":"python", ".sh":"shell", ".bash":"shell",
             ".zsh":"shell", ".toml":"toml", ".json":"json", ".jsonl":"jsonl",
             ".yml":"yaml", ".yaml":"yaml", ".md":"markdown", ".mdx":"markdown",
             ".ts":"typescript", ".tsx":"typescript", ".js":"javascript", ".jsx":"javascript",
             ".cjs":"javascript", ".mjs":"javascript", ".html":"html", ".css":"css",
             ".sql":"sql", ".xml":"xml", ".lock":"lock", ".txt":"text", ".tsv":"tsv",
             ".csv":"csv", ".ansi":"ansi", ".ascii":"text", ".raw":"binary", ".png":"binary",
             ".gz":"binary", ".bundle":"binary", ".rle":"text", ".guard":"text"}
    if name in {"makefile", "dockerfile"}:
        return name
    if name in {"cargo.toml", "mise.toml"}:
        return "toml"
    if name in {"license", "notice", "copying"}:
        return "legal-text"
    return table.get(ext, "extensionless" if not ext else "other")


TEST_SEGMENTS = {"test", "tests", "testing", "fixture", "fixtures", "coverage", "harness",
                 "snapshot", "snapshots", "baseline", "baselines", "golden", "bench", "benches",
                 "conformance", "shots"}
TEST_NAME_RE = re.compile(r"(^|[._-])(test|tests|spec|specs|fixture|fixtures|harness|coverage|golden|snapshot|baseline)([._-]|$)", re.I)
TEST_CODE_RE = re.compile(rb"(?m)#\s*\[\s*(?:tokio::)?test\b|#\s*\[\s*cfg(?:_attr)?\s*\([^\n]*\btest\b|\b(?:describe|it|test|pytest\.mark\.\w+)\s*\(|\bdef\s+test_\w+|\bfunc\s+Test[A-Z]\w*\s*\(|\btest\s+\w+\s*\(")
DOCTEST_RE = re.compile(r"(?m)^\s*(?:///|//!|# )?\s*```(?:rust|rs)(?:,([^`]*)|([^`]*))?\s*$", re.I)
RUST_ATTR_RE = re.compile(r"(?m)^\s*#\s*\[([^\]]+)\]\s*$")
RUST_FN_RE = re.compile(r"(?m)^\s*(?:(?:pub(?:\([^)]*\))?)\s+)?(?:(?:async|unsafe|const|extern\s+\"[^\"]+\")\s+)*fn\s+([A-Za-z_][A-Za-z0-9_]*)")
MOD_RE = re.compile(r"(?m)^\s*#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]\s*(?:\n\s*#\[[^\n]+\]\s*)*\n?\s*(?:pub\s+)?mod\s+([A-Za-z_][A-Za-z0-9_]*)")
FEATURE_TEST_RE = re.compile(r"(?m)^\s*#\s*\[\s*cfg(?:_attr)?\s*\([^\n]*(?:feature\s*=\s*\"testing\"|\btest\b)[^\n]*\]\s*(?:\n\s*#\[[^\n]+\]\s*)*\n?\s*(?:(?:pub(?:\([^)]*\))?|unsafe|async)\s+)*(?:fn|struct|enum|trait|mod|type|const|static)\s+([A-Za-z_][A-Za-z0-9_]*)")
MOD_DECL_RE = re.compile(r"(?m)^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;")
PATH_ATTR_RE = re.compile(r"(?m)#\s*\[\s*path\s*=\s*\"([^\"]+)\"\s*\]")
INCLUDE_RE = re.compile(r"\b(include|include_str|include_bytes)!\s*\(\s*\"([^\"]+)\"\s*\)")
DIAG_RE = re.compile(r"\b(println|eprintln|dbg|trace|debug|info|warn|error|log::trace|log::debug|log::info|log::warn|log::error|tracing::trace|tracing::debug|tracing::info|tracing::warn|tracing::error)!\s*\(")
CLI_ATTR_RE = re.compile(r"#\s*\[(command|arg|value|clap)::?[^\]]*\]", re.I)
DIRECTIVE_RE = re.compile(r"(?i)(SPDX-License-Identifier|copyright|generated (?:file|by)|do not edit|clippy::|rustfmt::|allow\(|deny\(|forbid\(|warn\(|cfg(?:_attr)?\s*\(|shebang|^#!|cargo:|rustdoc|doctest|shellcheck|actionlint|jactionlint|include(_str|_bytes)?!|\[path\s*=|license|copyright)")


def segments(path):
    return {x.lower() for x in path.replace("\\", "/").split("/")}


def path_test_reason(path):
    p = path.lower()
    seg = segments(p)
    reasons = []
    hit = sorted(seg & TEST_SEGMENTS)
    if hit:
        reasons.append("path-segment:" + ",".join(hit))
    name = PurePosixPath(p).name
    if TEST_NAME_RE.search(name) or name.startswith("test_"):
        reasons.append("test-like-filename")
    if p.startswith(".github/") or p.startswith(".github-gen/") or p.startswith(".velnor/") or p.startswith(".config/"):
        reasons.append("ci-or-generator-configuration")
    if p == "cargo.lock" or p.endswith("/cargo.lock"):
        reasons.append("workspace-lock-includes-test-dependency-resolution")
    return reasons


# Full tree metadata is read at the immutable source revision. Git paths are decoded with
# surrogateescape so the inventory remains reversible for unusual path bytes.
raw_tree = git("ls-tree", "-r", "-l", "-z", REV)
entries = []
for rec in raw_tree.split(b"\0"):
    if not rec:
        continue
    meta, path_b = rec.split(b"\t", 1)
    mode, typ, oid, size = meta.decode("ascii").split()
    path = path_b.decode("utf-8", "surrogateescape")
    entries.append({"path": path, "mode": mode, "object_type": typ, "blob_oid": oid,
                    "size_bytes": int(size), "language": lang(path), "test_reasons": path_test_reason(path)})

# Stream each unique blob once. The ledger is per path; duplicate paths retain their own
# records and share the calculated content hash.
sha_by_oid = {}
source_by_oid = {}
cat = subprocess.Popen(["git", "-C", REPO, "cat-file", "--batch"], stdin=subprocess.PIPE,
                       stdout=subprocess.PIPE, stderr=subprocess.PIPE, bufsize=0)
for e in entries:
    oid = e["blob_oid"]
    if oid in sha_by_oid:
        continue
    cat.stdin.write(oid.encode("ascii") + b"\n")
    cat.stdin.flush()
    header = cat.stdout.readline()
    if not header:
        raise RuntimeError("git cat-file ended before blob " + oid)
    got_oid, got_type, got_size = header.rstrip(b"\n").split(b" ")
    if got_oid.decode("ascii") != oid or got_type != b"blob":
        raise RuntimeError("unexpected git cat-file response: " + repr(header[:200]))
    size = int(got_size)
    h = hashlib.sha256()
    keep = lang(e["path"]) in {"rust", "python", "shell", "toml", "json", "jsonl", "yaml", "markdown",
                               "typescript", "javascript", "html", "sql", "xml", "lock", "text", "legal-text",
                               "Makefile", "Dockerfile", "extensionless", "other"} and size <= 20_000_000
    parts = [] if keep else None
    left = size
    while left:
        b = cat.stdout.read(min(1024 * 1024, left))
        if not b:
            raise RuntimeError("short blob read for " + oid)
        h.update(b)
        if parts is not None:
            parts.append(b)
        left -= len(b)
    if cat.stdout.read(1) != b"\n":
        raise RuntimeError("missing blob record terminator for " + oid)
    sha_by_oid[oid] = h.hexdigest()
    if parts is not None:
        source_by_oid[oid] = b"".join(parts)
cat.stdin.close()
if cat.wait() != 0:
    raise RuntimeError(cat.stderr.read().decode("utf-8", "replace"))

# Determine content-based test evidence, dependency/config ownership, and declaration inventories.
for e in entries:
    oid, path = e["blob_oid"], e["path"]
    e["sha256"] = sha_by_oid[oid]
    data = source_by_oid.get(oid)
    if data is not None:
        reasons = e["test_reasons"]
        if TEST_CODE_RE.search(data):
            reasons.append("test-declaration-or-test-feature-token")
        low = path.lower()
        if low.endswith(".md") or low.endswith(".mdx") or low.endswith(".rs"):
            try:
                txt = data.decode("utf-8", "replace")
            except Exception:
                txt = ""
            if DOCTEST_RE.search(txt):
                reasons.append("rust-doctest-fence")
        if low.endswith("cargo.toml"):
            try:
                obj = tomllib.loads(data.decode("utf-8"))
            except Exception:
                obj = {}
            if obj.get("dev-dependencies"):
                reasons.append("manifest-dev-dependencies")
            for tname, tval in obj.get("target", {}).items():
                if tval.get("dev-dependencies"):
                    reasons.append("target-specific-dev-dependencies")
            if obj.get("test") or obj.get("bench") or obj.get("example"):
                reasons.append("manifest-test-bench-or-example-target")
            if obj.get("package", {}).get("autotests") is False:
                reasons.append("manifest-autotests-false")
        e["test_reasons"] = sorted(set(reasons))

# Full ledger, plus filtered test-related ledger, both source-SHA bound.
with gzip.open(OUT + "/tracked-tree.jsonl.gz", "wt", encoding="utf-8", newline="\n", compresslevel=6) as f:
    for e in entries:
        f.write(json.dumps(e, ensure_ascii=True, sort_keys=True) + "\n")
with gzip.open(OUT + "/test-files.jsonl.gz", "wt", encoding="utf-8", newline="\n", compresslevel=6) as f:
    for e in entries:
        if e["test_reasons"]:
            f.write(json.dumps(e, ensure_ascii=True, sort_keys=True) + "\n")

# Collect source-level test declarations, doctests, imports/includes, directives, diagnostics,
# Clap help declarations, and manifest target/dependency declarations.
tests = []
mods = []
feature_items = []
doctests = []
imports = []
directives = []
diagnostics = []
cli = []
manifests = []
nonrust_tests = []
production_capture = []
all_paths = {e["path"] for e in entries}

for e in entries:
    path, oid = e["path"], e["blob_oid"]
    data = source_by_oid.get(oid)
    if data is None:
        continue
    try:
        txt = data.decode("utf-8", "replace")
    except Exception:
        continue
    lines = txt.splitlines()
    path_l = path.lower()
    if path_l.endswith("cargo.toml"):
        try:
            obj = tomllib.loads(txt)
        except Exception as ex:
            obj = {"parse_error": str(ex)}
        package = obj.get("package", {}) if isinstance(obj, dict) else {}
        deps = []
        for kind in ("dependencies", "dev-dependencies", "build-dependencies"):
            for dep, spec in obj.get(kind, {}).items() if isinstance(obj, dict) and isinstance(obj.get(kind), dict) else []:
                deps.append({"kind": kind, "name": dep, "spec": spec})
        target_deps = []
        for target, td in obj.get("target", {}).items() if isinstance(obj, dict) and isinstance(obj.get("target"), dict) else []:
            if not isinstance(td, dict):
                continue
            for kind in ("dependencies", "dev-dependencies", "build-dependencies"):
                for dep, spec in td.get(kind, {}).items():
                    target_deps.append({"target": target, "kind": kind, "name": dep, "spec": spec})
        targets = []
        for kind in ("lib", "bin", "test", "bench", "example"):
            val = obj.get(kind, [])
            if isinstance(val, dict):
                val = [val]
            if isinstance(val, list):
                for i, item in enumerate(val):
                    if isinstance(item, dict):
                        targets.append({"kind": kind, "name": item.get("name"), "path": item.get("path"),
                                        "test": item.get("test"), "doctest": item.get("doctest"),
                                        "harness": item.get("harness"), "bench": item.get("bench")})
        manifests.append({"path": path, "package_name": package.get("name"), "autotests": package.get("autotests"),
                          "autoexamples": package.get("autoexamples"), "autobenches": package.get("autobenches"),
                          "dependencies": deps, "target_dependencies": target_deps, "targets": targets,
                          "features": obj.get("features", {})})
    if lang(path) == "rust":
        for m in MOD_RE.finditer(txt):
            mods.append({"path": path, "line": txt.count("\n", 0, m.start(1)) + 1, "module": m.group(1), "kind": "unit-test-module"})
        # Associate each test attr with the next function declaration within the next 16 lines.
        for m in re.finditer(r"(?m)^\s*#\s*\[\s*(?:tokio::)?test\s*\]\s*$", txt):
            tail = txt[m.end():m.end() + 1500]
            fm = RUST_FN_RE.search(tail)
            tests.append({"path": path, "line": txt.count("\n", 0, m.start()) + 1,
                          "function": fm.group(1) if fm else None, "attribute": m.group(0).strip()})
        for m in FEATURE_TEST_RE.finditer(txt):
            feature_items.append({"path": path, "line": txt.count("\n", 0, m.start()) + 1, "item": m.group(1), "cfg": m.group(0).splitlines()[0].strip()})
        for m in MOD_DECL_RE.finditer(txt):
            line = txt.count("\n", 0, m.start()) + 1
            mod = m.group(1)
            parent = str(PurePosixPath(path).parent)
            if PurePosixPath(path).name in {"lib.rs", "main.rs", "mod.rs"}:
                base = parent
            else:
                base = parent + "/" + PurePosixPath(path).stem
            candidates = [base + "/" + mod + ".rs", base + "/" + mod + "/mod.rs"]
            imports.append({"kind":"out-of-line-module", "path":path, "line":line, "declaration":mod,
                            "candidate_paths":candidates, "existing_candidates":[x for x in candidates if x in all_paths],
                            "resolution":"unique-textual-candidate" if sum(x in all_paths for x in candidates)==1 else "ambiguous-or-unresolved; inline module context not parsed"})
        for m in PATH_ATTR_RE.finditer(txt):
            rel = m.group(1)
            cand = str(PurePosixPath(path).parent / rel)
            imports.append({"kind":"path-attribute", "path":path, "line":txt.count("\n",0,m.start())+1,
                            "declaration":rel, "candidate_paths":[cand], "existing_candidates":[cand] if cand in all_paths else [],
                            "resolution":"present" if cand in all_paths else "missing-textual-candidate"})
        for m in INCLUDE_RE.finditer(txt):
            rel = m.group(2)
            cand = str(PurePosixPath(path).parent / rel)
            imports.append({"kind":m.group(1), "path":path, "line":txt.count("\n",0,m.start())+1,
                            "declaration":rel, "candidate_paths":[cand], "existing_candidates":[cand] if cand in all_paths else [],
                            "resolution":"present" if cand in all_paths else "missing-textual-candidate"})
        for m in DOCTEST_RE.finditer(txt):
            doctests.append({"path":path, "line":txt.count("\n",0,m.start())+1, "fence_attributes":(m.group(1) or m.group(2) or "").strip()})
    elif lang(path) in {"python", "typescript", "javascript", "shell"}:
        # Keep declarations narrow; path-based test files are still represented even when framework syntax is unusual.
        pats = [(r"(?m)^\s*def\s+(test_[A-Za-z0-9_]+)\s*\(", "function"),
                (r"(?m)^\s*async\s+def\s+(test_[A-Za-z0-9_]+)\s*\(", "async-function"),
                (r"\b(?:it|test)\s*\(\s*(['\"`])([^'\"`]+)\1", "test-case"),
                (r"\b(?:describe|suite)\s*\(\s*(['\"`])([^'\"`]+)\1", "test-suite"),
                (r"(?m)^\s*func\s+(Test[A-Z][A-Za-z0-9_]*)\s*\(", "go-test-function")]
        for pat, kind in pats:
            for m in re.finditer(pat, txt):
                name = next((x for x in m.groups() if x and not (len(x)==1 and x in "'\"`")), None)
                nonrust_tests.append({"path":path, "line":txt.count("\n",0,m.start())+1, "kind":kind, "name":name})
    for i, line in enumerate(lines, 1):
        if lang(path) == "rust":
            for m in DIAG_RE.finditer(line):
                # Exclude known test-source trees and cfg(test)-only source after path classification; retain mixed files.
                if not (set(path_l.split("/")) & {"tests", "test", "testing", "coverage"}):
                    diagnostics.append({"path":path, "line":i, "macro":m.group(1), "source":line.strip()[:240]})
            if "#[command" in line or "#[arg" in line or "#[clap" in line or "#[value" in line or "clap::" in line:
                cli.append({"path":path, "line":i, "kind":"attribute-or-clap-use", "source":line.strip()[:240]})
        md = DIRECTIVE_RE.search(line)
        if md:
            directives.append({"path":path, "line":i, "matched":md.group(0)[:100], "source":line.strip()[:300]})
        if line.startswith("#!"):
            directives.append({"path":path, "line":i, "matched":"shebang-or-crate-inner-attribute", "source":line.strip()[:300]})

# Explicit implicit-target paths and source counts by source language.
target_files = []
for m in manifests:
    mp = m["path"]
    crate_dir = str(PurePosixPath(mp).parent)
    for target in m["targets"]:
        tpath = target["path"]
        if tpath is None:
            default = {"lib":"src/lib.rs", "bin":"src/main.rs", "test":"tests/" + str(target["name"]) + ".rs",
                       "bench":"benches/" + str(target["name"]) + ".rs", "example":"examples/" + str(target["name"]) + ".rs"}.get(target["kind"])
            tpath = default
        candidate = (crate_dir + "/" + tpath) if tpath else None
        target_files.append({"manifest":mp, **target, "resolved_declared_path":candidate,
                             "path_exists":candidate in all_paths if candidate else None})

for e in entries:
    p = e["path"].lower()
    if (p.startswith("baselines/") or p.startswith("snapshots/") or "/baseline/" in p or "/snapshots/" in p) and e["size_bytes"] == 0:
        production_capture.append({"path":e["path"], "classification":"empty-asset"})

inventory = {
    "source_commit":REV,
    "tracked_paths":len(entries),
    "tracked_blob_bytes":sum(e["size_bytes"] for e in entries),
    "unique_blobs":len(sha_by_oid),
    "hashed_unique_blob_bytes":sum(int(e["size_bytes"]) for e in {e["blob_oid"]:e for e in entries}.values()),
    "test_related_paths":sum(bool(e["test_reasons"]) for e in entries),
    "hash_method":"sha256 over git cat-file blob bytes at source commit; duplicate blob OIDs hashed once and mapped to every tracked path",
    "test_related_ledger":"test-files.jsonl.gz",
    "whole_tree_ledger":"tracked-tree.jsonl.gz",
    "unit_test_modules":mods,
    "rust_test_functions":tests,
    "rust_testing_cfg_items":feature_items,
    "rustdoc_fences":doctests,
    "non_rust_test_declarations":nonrust_tests,
    "module_imports_and_includes":imports,
    "cargo_manifests":manifests,
    "declared_cargo_targets":target_files,
    "production_diagnostic_macro_sites":diagnostics,
    "clap_attribute_sites":cli,
    "comment_directive_sites":directives,
    "limits":["Static source scan only; no Cargo command, compiler, test, or legacy test code ran.",
              "Regex-based Rust module resolution lists textual candidates; inline module scope and macro-generated modules are not compiler-resolved.",
              "The whole-tree ledger preserves all tracked paths and content hashes; test-related classification combines path, manifest, and test-syntax evidence and remains reviewable per row."]
}
with open(OUT + "/test-inventory.json", "w", encoding="utf-8") as f:
    json.dump(inventory, f, ensure_ascii=False, indent=2, sort_keys=True)
    f.write("\n")
summary = {k:inventory[k] for k in ["source_commit","tracked_paths","tracked_blob_bytes","unique_blobs","hashed_unique_blob_bytes","test_related_paths","hash_method","test_related_ledger","whole_tree_ledger"]}
summary.update({"unit_test_modules":len(mods),"rust_test_functions":len(tests),"rust_testing_cfg_items":len(feature_items),"rustdoc_fences":len(doctests),"non_rust_test_declarations":len(nonrust_tests),"module_imports_and_includes":len(imports),"cargo_manifests":len(manifests),"declared_cargo_targets":len(target_files),"production_diagnostic_macro_sites":len(diagnostics),"clap_attribute_sites":len(cli),"comment_directive_sites":len(directives)})
with open(OUT + "/inventory-summary.json", "w", encoding="utf-8") as f:
    json.dump(summary, f, ensure_ascii=False, indent=2, sort_keys=True)
    f.write("\n")
print(json.dumps(summary, sort_keys=True))
