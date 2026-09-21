#!/usr/bin/env python3
"""Source-qualified test identity discovery. Parsing never substitutes for execution."""
from __future__ import annotations

import json
from pathlib import Path
import re
import tomllib

from inventory import Invalid, digest, encoded, require, unique


SEED_PATH_RE = re.compile(r"^- `([^`]+)`$")
IDENT_RE = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
SKIP_DOC_LANGS = {"text", "sh", "bash", "shell", "console", "toml", "json", "csv", "ignore-test"}
RUSTDOC_HINTS = {
    "", "rust", "compile_fail", "should_panic", "no_run", "ignore",
    "edition2015", "edition2018", "edition2021", "edition2024",
}
ASSERT_MACROS = (
    "assert", "assert_eq", "assert_ne",
    "debug_assert", "debug_assert_eq", "debug_assert_ne",
)
ITEM_QUALS = {"pub", "async", "const", "unsafe", "extern", "default"}
FILE_ROOT_STEMS = {"lib", "main", "mod"}

# Oracle-conflict classification for TASK-008 inputs. Discovery never grants edit authority.
# The three holla tests are the known mandatory inline conflicts pinned in
# docs/refactoring-plan/inline-test-source-scope.md. The two integration paths
# carry product expectations contradicted by oracle behavior per branch-diff
# evidence (BF03 grapheme word motion vs scalar scans; viewport cloning deleted
# with no-clone replacement proof). Whole-file marking is coarse but fail-closed:
# TASK-008 reviews each span before any test-only migration.
ORACLE_CONFLICT_TESTS = {
    ("holla", "app::tests::home_hint_casing_preserves_lowercase_physical_shortcuts"),
    ("holla", "app::tests::home_escape_clears_scope_before_canonical_query"),
    ("holla", "scenario::tests::names_round_trip"),
}
ORACLE_CONFLICT_PATHS = {
    "crates/tui/tests/viewport.rs",
    "crates/tui/tests/keyboard_editor.rs",
}


def posix(path):
    return Path(path).as_posix()


class Cursor:
    def __init__(self, text, limit=None):
        self.text = text
        self.n = len(text) if limit is None else min(limit, len(text))
        self.i = 0

    def done(self):
        return self.i >= self.n

    def peek(self, n=1):
        return self.text[self.i:self.i + n]

    def startswith(self, value):
        return self.text.startswith(value, self.i)

    def skip(self, n=1):
        self.i += n

    def line_at(self, index=None):
        return self.text.count("\n", 0, self.i if index is None else index) + 1


def skip_ws(cur):
    while not cur.done() and cur.peek() in " \t\r\n":
        cur.skip()


def skip_line(cur):
    while not cur.done() and cur.peek() != "\n":
        cur.skip()
    if not cur.done():
        cur.skip()


def skip_block_comment(cur):
    require(cur.startswith("/*"), "not a block comment")
    cur.skip(2)
    depth = 1
    while not cur.done() and depth:
        if cur.startswith("/*"):
            depth += 1
            cur.skip(2)
        elif cur.startswith("*/"):
            depth -= 1
            cur.skip(2)
        else:
            cur.skip()
    require(depth == 0, "unterminated block comment")


def skip_string(cur):
    quote = cur.peek()
    require(quote in "\"'", "not a string")
    cur.skip()
    while not cur.done():
        ch = cur.peek()
        if ch == "\\":
            cur.skip(min(2, cur.n - cur.i))
            continue
        cur.skip()
        if ch == quote:
            return
    raise Invalid("unterminated string")


def skip_raw_string(cur):
    if cur.startswith("br") or cur.startswith("cr"):
        cur.skip(2)
    elif cur.peek() in "bc" and cur.peek(2)[1:] == "r":
        cur.skip()
    require(cur.peek() == "r", "not a raw string")
    cur.skip()
    hashes = 0
    while cur.peek() == "#":
        hashes += 1
        cur.skip()
    require(cur.peek() == "\"", "raw string missing quote")
    cur.skip()
    end = "\"" + "#" * hashes
    while not cur.done():
        if cur.startswith(end):
            cur.skip(len(end))
            return
        cur.skip()
    raise Invalid("unterminated raw string")


def skip_lifetime_or_char(cur):
    require(cur.peek() == "'", "not a lifetime or char")
    nxt = cur.peek(2)
    if len(nxt) == 2 and (nxt[1].isalpha() or nxt[1] == "_"):
        cur.skip()
        while not cur.done() and (cur.peek().isalnum() or cur.peek() == "_"):
            cur.skip()
        if cur.peek() == "'":
            cur.skip()
        return
    skip_string(cur)


def try_skip_raw(cur):
    if not (cur.startswith("r\"") or cur.startswith("r#") or cur.startswith("br") or cur.startswith("cr")):
        return False
    pos = cur.i
    try:
        skip_raw_string(cur)
        return True
    except Invalid:
        cur.i = pos
        return False


def skip_ws_comments(cur, docs=None):
    while not cur.done():
        skip_ws(cur)
        if cur.done():
            return
        if cur.startswith("///") or cur.startswith("//!"):
            begin = cur.i
            skip_line(cur)
            if docs is not None:
                docs.append(cur.text[begin:cur.i])
            continue
        if cur.startswith("//"):
            skip_line(cur)
            continue
        if cur.startswith("/*"):
            begin = cur.i
            is_doc = cur.startswith("/**") or cur.startswith("/*!")
            skip_block_comment(cur)
            if is_doc and docs is not None:
                docs.append(cur.text[begin:cur.i])
            continue
        return


def skip_delimited(cur):
    open_ch = cur.peek()
    close = {"(": ")", "[": "]", "{": "}"}[open_ch]
    cur.skip()
    depth = 1
    while not cur.done() and depth:
        skip_ws(cur)
        if cur.done():
            break
        if cur.startswith("//"):
            skip_line(cur)
            continue
        if cur.startswith("/*"):
            skip_block_comment(cur)
            continue
        if try_skip_raw(cur):
            continue
        ch = cur.peek()
        if ch == "\"":
            skip_string(cur)
            continue
        if ch == "'":
            skip_lifetime_or_char(cur)
            continue
        if ch == open_ch:
            depth += 1
            cur.skip()
        elif ch == close:
            depth -= 1
            cur.skip()
        else:
            cur.skip()
    require(depth == 0, "unterminated delimiter")


def parse_ident(cur):
    skip_ws_comments(cur)
    match = IDENT_RE.match(cur.text, cur.i)
    require(match is not None, "expected identifier")
    cur.i = match.end()
    return match.group()


def parse_attribute(cur):
    begin = cur.i
    require(cur.peek() == "#", "expected attribute")
    cur.skip()
    if cur.peek() == "!":
        cur.skip()
    skip_ws(cur)
    require(cur.peek() == "[", "expected attribute body")
    skip_delimited(cur)
    return cur.text[begin:cur.i]


def attr_inner(raw):
    text = raw.strip()
    text = text[2:] if text.startswith("#!") else text[1:]
    text = text.strip()
    require(text.startswith("[") and text.endswith("]"), "malformed attribute")
    return text[1:-1].strip()


def attr_is_test(inner):
    head = inner.split("(", 1)[0].split("=", 1)[0].strip()
    if head == "test" or head.endswith("::test"):
        return True
    return inner.startswith("cfg_attr") and bool(re.search(r"\btest\b", inner))


def attr_ignore_reason(inner):
    if inner == "ignore":
        return "ignored"
    match = re.fullmatch(r'ignore\s*=\s*"(.*)"', inner, re.DOTALL)
    if match:
        return match.group(1)
    if inner.startswith("ignore"):
        return inner
    return None


def attr_cfg(inner):
    if inner.startswith("cfg(") and inner.endswith(")"):
        return inner[4:-1].strip()
    return None


def attr_path(inner):
    match = re.fullmatch(r'path\s*=\s*"(.*)"', inner)
    return match.group(1) if match else None


def attr_doc_include(inner):
    match = re.fullmatch(r'doc\s*=\s*include_str!\s*\(\s*"(.*)"\s*\)', inner)
    return match.group(1) if match else None


def classify_assertion(package, identity, path):
    if (package, identity) in ORACLE_CONFLICT_TESTS or path in ORACLE_CONFLICT_PATHS:
        return "oracle-conflict"
    return "preserve"


def collect_assertions(text, body_start, body_end, package, identity, path):
    cur = Cursor(text, body_end)
    cur.i = body_start
    found = []
    while not cur.done():
        skip_ws_comments(cur)
        if cur.done():
            break
        if try_skip_raw(cur):
            continue
        ch = cur.peek()
        if ch == "\"":
            skip_string(cur)
            continue
        if ch == "'":
            skip_lifetime_or_char(cur)
            continue
        match = IDENT_RE.match(cur.text, cur.i)
        if match and match.group() in ASSERT_MACROS and cur.text[match.end():match.end() + 1] == "!":
            start = cur.i
            line = cur.line_at(start)
            cur.i = match.end() + 1
            skip_ws_comments(cur)
            if cur.peek() in "([{":
                skip_delimited(cur)
            found.append({
                "kind": match.group(),
                "line": line,
                "byte_start": start,
                "byte_end": min(cur.i, body_end),
                "classification": classify_assertion(package, identity, path),
            })
            continue
        cur.skip()
    return found


def parse_rustdoc_fences(text, rel_path, package, target, bare=False):
    identities = []
    lines = text.splitlines(keepends=True)
    i = 0
    offset = 0
    while i < len(lines):
        stripped = lines[i].lstrip()
        prefix = None
        body = None
        if stripped.startswith("///"):
            prefix, body = "///", stripped[3:]
        elif stripped.startswith("//!"):
            prefix, body = "//!", stripped[3:]
        elif bare and stripped.startswith("```"):
            prefix, body = "", stripped
        else:
            offset += len(lines[i])
            i += 1
            continue
        fence = body.strip()
        if not fence.startswith("```"):
            offset += len(lines[i])
            i += 1
            continue
        langs = {part.strip() for part in fence[3:].strip().split(",") if part.strip()}
        if langs & SKIP_DOC_LANGS:
            offset += len(lines[i])
            i += 1
            continue
        unknown = {part for part in langs if part not in RUSTDOC_HINTS and not part.startswith("E")}
        if unknown:
            offset += len(lines[i])
            i += 1
            continue
        start_line = i + 1
        start_off = offset
        offset += len(lines[i])
        i += 1
        while i < len(lines):
            s = lines[i].lstrip()
            chunk = s[len(prefix):] if prefix and s.startswith(prefix) else s
            offset += len(lines[i])
            i += 1
            if chunk.strip().startswith("```"):
                break
        mode = "compile_fail" if "compile_fail" in langs else (
            "should_panic" if "should_panic" in langs else (
                "ignore" if "ignore" in langs else "rust"))
        identities.append({
            "package": package,
            "kind": "doc",
            "target": target,
            "identity": f"{rel_path}:{start_line}",
            "path": rel_path,
            "blob_sha256": None,
            "line": start_line,
            "end_line": i,
            "byte_start": start_off,
            "byte_end": offset,
            "origin": "rustdoc",
            "macro": None,
            "cfg": [],
            "ignored": "rustdoc ignore" if mode == "ignore" else None,
            "assertions": [],
            "classification": "preserve",
            "doc_mode": mode,
        })
    return identities


def load_seed_paths(markdown, expected=146):
    paths = []
    for line in markdown.splitlines():
        match = SEED_PATH_RE.fullmatch(line.strip())
        if match:
            paths.append(match.group(1))
    unique(paths, "seed path")
    if expected is not None:
        require(len(paths) == expected, "inline-test seed must enumerate " + str(expected) + " paths")
    return paths


def parse_workspace_members(root):
    cargo = tomllib.loads((root / "Cargo.toml").read_text())
    members = cargo.get("workspace", {}).get("members", [])
    require(members, "workspace members missing")
    return members


def manifest_targets(root, rel_manifest):
    directory = (root / rel_manifest).parent
    cargo = tomllib.loads((directory / "Cargo.toml").read_text())
    package = cargo["package"]
    name = package["name"]
    roots = []

    def add(kind, target, rel_path, doctest=False):
        path = directory / rel_path
        if path.is_file():
            roots.append({
                "package": name,
                "kind": kind,
                "target": target,
                "path": posix(path.relative_to(root)),
                "doctest": doctest,
                "manifest_dir": posix(directory.relative_to(root)),
            })

    lib = cargo.get("lib")
    if lib:
        add("lib", lib.get("name", name.replace("-", "_")), lib.get("path", "src/lib.rs"), True)
    elif (directory / "src/lib.rs").is_file():
        add("lib", name.replace("-", "_"), "src/lib.rs", True)

    bins = cargo.get("bin", [])
    if isinstance(bins, dict):
        bins = [bins]
    declared_bins = {entry.get("path", "") for entry in bins}
    for entry in bins:
        add("bin", entry["name"], entry.get("path", "src/main.rs"))
    if package.get("autobins", True):
        if "src/main.rs" not in declared_bins:
            add("bin", name, "src/main.rs")
        bin_dir = directory / "src/bin"
        if bin_dir.is_dir():
            for path in sorted(bin_dir.rglob("*.rs")):
                rel = posix(path.relative_to(directory))
                if rel in declared_bins:
                    continue
                if path.name == "main.rs":
                    add("bin", path.parent.name, rel)
                elif path.parent == bin_dir:
                    add("bin", path.stem, rel)

    def auto_kind(kind, enabled, folder, entries):
        declared = set()
        for entry in entries:
            path = entry.get("path") or f"{folder}/{entry['name']}.rs"
            declared.add(path)
            add(kind, entry["name"], path)
        if not enabled:
            return
        base = directory / folder
        if not base.is_dir():
            return
        for path in sorted(base.glob("*.rs")):
            rel = posix(path.relative_to(directory))
            if rel not in declared:
                add(kind, path.stem, rel)
        for path in sorted(base.glob("*/main.rs")):
            rel = posix(path.relative_to(directory))
            if rel not in declared:
                add(kind, path.parent.name, rel)

    tests = cargo.get("test", [])
    if isinstance(tests, dict):
        tests = [tests]
    auto_kind("test", package.get("autotests", True), "tests", tests)
    examples = cargo.get("example", [])
    if isinstance(examples, dict):
        examples = [examples]
    auto_kind("example", package.get("autoexamples", True), "examples", examples)
    benches = cargo.get("bench", [])
    if isinstance(benches, dict):
        benches = [benches]
    auto_kind("bench", package.get("autobenches", True), "benches", benches)
    return roots


def file_child_dir(root, rel, crate_root=False):
    path = root / rel
    if crate_root or path.stem in FILE_ROOT_STEMS:
        return path.parent
    return path.parent / path.stem


def resolve_mod_path(root, rel, file_module, current_module, name, path_attr, crate_root_rel):
    current = root / rel
    if path_attr:
        return posix((current.parent / path_attr).resolve().relative_to(root.resolve()))
    extra = current_module[len(file_module):]
    base = file_child_dir(root, rel, rel == crate_root_rel)
    for part in extra:
        base = base / part
    file_rs = base / f"{name}.rs"
    file_mod = base / name / "mod.rs"
    if file_rs.is_file():
        return posix(file_rs.relative_to(root))
    if file_mod.is_file():
        return posix(file_mod.relative_to(root))
    raise Invalid(f"missing module {name} from {rel}")


def tests_from_macro_body(body):
    names = []
    cur = Cursor(body)
    while not cur.done():
        skip_ws_comments(cur)
        if cur.done():
            break
        if cur.peek() != "#":
            cur.skip()
            continue
        try:
            raw = parse_attribute(cur)
        except (Invalid, KeyError):
            cur.skip()
            continue
        if not attr_is_test(attr_inner(raw)):
            continue
        skip_ws_comments(cur)
        while not cur.done() and cur.peek() == "#":
            parse_attribute(cur)
            skip_ws_comments(cur)
        while not cur.done():
            skip_ws_comments(cur)
            ident = IDENT_RE.match(cur.text, cur.i)
            if ident is None:
                break
            word = ident.group()
            if word in ITEM_QUALS:
                cur.i = ident.end()
                if word == "pub" and cur.peek() == "(":
                    skip_delimited(cur)
                continue
            if word == "fn":
                cur.i = ident.end()
                names.append(parse_ident(cur))
            break
    return names


def parse_suite_entries(body):
    names = []
    cur = Cursor(body)
    while not cur.done():
        skip_ws_comments(cur)
        if cur.done():
            break
        if cur.peek() == ",":
            cur.skip()
            continue
        if IDENT_RE.match(cur.text, cur.i) is None:
            cur.skip()
            continue
        ident = parse_ident(cur)
        skip_ws_comments(cur)
        if cur.startswith("=>"):
            names.append(ident)
            cur.skip(2)
            while not cur.done() and cur.peek() != ",":
                if cur.peek() in "([{":
                    skip_delimited(cur)
                else:
                    cur.skip()
        else:
            break
    return names


def first_invocation_ident(body):
    cur = Cursor(body)
    skip_ws_comments(cur)
    match = IDENT_RE.match(cur.text, cur.i)
    return match.group() if match else None


def skip_qualifiers(cur):
    while not cur.done():
        skip_ws_comments(cur)
        ident = IDENT_RE.match(cur.text, cur.i)
        if ident is None:
            return
        word = ident.group()
        if word not in ITEM_QUALS:
            return
        cur.i = ident.end()
        if word == "pub" and cur.peek() == "(":
            skip_delimited(cur)
        elif word == "extern":
            skip_ws_comments(cur)
            if cur.peek() == "\"":
                skip_string(cur)


def extract_generators(text):
    generators = {}
    cur = Cursor(text)
    while not cur.done():
        skip_ws_comments(cur)
        if cur.done():
            break
        if cur.peek() == "#":
            try:
                parse_attribute(cur)
            except (Invalid, KeyError):
                cur.skip()
            continue
        ident = IDENT_RE.match(cur.text, cur.i)
        if ident is None:
            if cur.peek() in "([{":
                try:
                    skip_delimited(cur)
                except (Invalid, KeyError):
                    cur.skip()
            else:
                cur.skip()
            continue
        word = ident.group()
        if word in ITEM_QUALS:
            skip_qualifiers(cur)
            continue
        if word != "macro_rules":
            cur.i = ident.end()
            continue
        cur.i = ident.end()
        skip_ws_comments(cur)
        if cur.peek() != "!":
            continue
        cur.skip()
        name = parse_ident(cur)
        skip_ws_comments(cur)
        begin = cur.i
        if cur.peek() in "[{(":
            skip_delimited(cur)
        body = text[begin:cur.i]
        tests = tests_from_macro_body(body)
        compact = re.sub(r"\s+", "", body)
        if tests:
            kind = "suite-entries" if "$($name:ident=>" in compact else "module-per-invocation"
            generators[name] = {"tests": tests, "kind": kind}
        elif name in {"baseline_case", "baseline_case_with_variants", "audit_matrix_tests"}:
            generators[name] = {"tests": [], "kind": "wrapper", "forwards": "baseline_combo_tests"}
    return generators


class Discovery:
    def __init__(self, root, crate, generators):
        self.root = root
        self.crate = crate
        self.generators = generators
        self.identities = []
        self.blockers = []
        self.scanned = []
        self._blobs = {}

    def blob(self, path):
        if path not in self._blobs:
            self._blobs[path] = digest((self.root / path).read_bytes())
        return self._blobs[path]

    def emit(self, identity, path, line, end_line, start, end, origin, macro, cfg, ignored, assertions):
        self.identities.append({
            "package": self.crate["package"],
            "kind": self.crate["kind"],
            "target": self.crate["target"],
            "identity": identity,
            "path": path,
            "blob_sha256": self.blob(path),
            "line": line,
            "end_line": end_line,
            "byte_start": start,
            "byte_end": end,
            "origin": origin,
            "macro": macro,
            "cfg": cfg,
            "ignored": ignored,
            "assertions": assertions,
            "classification": classify_assertion(self.crate["package"], identity, path),
        })

    def parse_file(self, rel, file_module, text):
        self.scanned.append(rel)
        for row in parse_rustdoc_fences(text, rel, self.crate["package"], self.crate["target"]):
            row["blob_sha256"] = self.blob(row["path"])
            self.identities.append(row)
        cur = Cursor(text)
        self.parse_items(cur, rel, file_module, file_module, text, len(text))

    def parse_items(self, cur, rel, file_module, module_path, text, limit):
        while cur.i < limit and not cur.done():
            docs = []
            skip_ws_comments(cur, docs)
            if cur.i >= limit:
                break
            attrs = []
            while cur.i < limit and cur.peek() == "#":
                attrs.append(parse_attribute(cur))
                skip_ws_comments(cur, docs)
            cfg = [c for c in (attr_cfg(attr_inner(a)) for a in attrs) if c]
            path_attr = next((p for p in (attr_path(attr_inner(a)) for a in attrs) if p), None)
            for raw in attrs:
                include = attr_doc_include(attr_inner(raw))
                if include:
                    included = ((self.root / rel).parent / include).resolve()
                    inc_rel = posix(included.relative_to(self.root.resolve()))
                    self.scanned.append(inc_rel)
                    for row in parse_rustdoc_fences(
                            included.read_text(), inc_rel, self.crate["package"], self.crate["target"],
                            bare=included.suffix.lower() in {".md", ".txt"}):
                        row["blob_sha256"] = self.blob(row["path"])
                        self.identities.append(row)
            skip_ws_comments(cur)
            if cur.i >= limit:
                break
            skip_qualifiers(cur)
            skip_ws_comments(cur)
            ident = IDENT_RE.match(cur.text, cur.i)
            if ident is None:
                if cur.peek() in "([{":
                    skip_delimited(cur)
                else:
                    cur.skip()
                continue
            word = ident.group()
            if word == "mod":
                self.parse_mod(cur, ident, rel, file_module, module_path, path_attr, limit)
                continue
            if word == "fn":
                self.parse_fn(cur, ident, rel, module_path, text, attrs, cfg, limit)
                continue
            if word == "macro_rules":
                cur.i = ident.end()
                skip_ws_comments(cur)
                if cur.peek() == "!":
                    cur.skip()
                parse_ident(cur)
                skip_ws_comments(cur)
                if cur.peek() in "[{(":
                    skip_delimited(cur)
                continue
            if word == "impl":
                cur.i = ident.end()
                self.skip_header_then_body(cur, limit)
                continue
            self.parse_maybe_macro(cur, ident, rel, module_path, attrs, cfg, limit)

    def parse_mod(self, cur, ident, rel, file_module, module_path, path_attr, limit):
        cur.i = ident.end()
        name = parse_ident(cur)
        skip_ws_comments(cur)
        child = module_path + [name]
        if cur.peek() == ";":
            cur.skip()
            return
        if cur.peek() == "{":
            body_start = cur.i
            skip_delimited(cur)
            inner = Cursor(cur.text, cur.i - 1)
            inner.i = body_start + 1
            self.parse_items(inner, rel, file_module, child, cur.text, cur.i - 1)
            return

    def parse_fn(self, cur, ident, rel, module_path, text, attrs, cfg, limit):
        start = cur.i
        line = cur.line_at()
        cur.i = ident.end()
        name = parse_ident(cur)
        self.skip_header_then_body(cur, limit)
        if not any(attr_is_test(attr_inner(a)) for a in attrs):
            return
        ignored = next((attr_ignore_reason(attr_inner(a)) for a in attrs if attr_ignore_reason(attr_inner(a))), None)
        identity = "::".join(module_path + [name])
        assertions = collect_assertions(text, start, cur.i, self.crate["package"], identity, rel)
        self.emit(identity, rel, line, cur.line_at(), start, cur.i, "fn-test", None, cfg, ignored, assertions)

    def parse_maybe_macro(self, cur, ident, rel, module_path, attrs, cfg, limit):
        start = cur.i
        line = cur.line_at()
        path_name = ident.group()
        cur.i = ident.end()
        while cur.startswith("::"):
            cur.skip(2)
            path_name += "::" + parse_ident(cur)
        skip_ws_comments(cur)
        if cur.peek() != "!":
            self.skip_header_then_body(cur, limit)
            return
        cur.skip()
        skip_ws_comments(cur)
        inv_ident = None
        if IDENT_RE.match(cur.text, cur.i):
            inv_ident = parse_ident(cur)
            skip_ws_comments(cur)
        if cur.peek() not in "([{":
            return
        body_start = cur.i + 1
        skip_delimited(cur)
        body = cur.text[body_start:cur.i - 1]
        simple = path_name.split("::")[-1]
        generator = self.generators.get(simple)
        if generator:
            self.expand(simple, generator, body, inv_ident, module_path, rel, line, start, cur.i, cfg)
        elif tests_from_macro_body(body):
            self.blockers.append({
                "reason": "unexpanded test-generating macro",
                "macro": path_name,
                "path": rel,
                "line": line,
            })

    def skip_header_then_body(self, cur, limit):
        depth = 0
        while cur.i < limit and not cur.done():
            skip_ws_comments(cur)
            if cur.i >= limit:
                return
            if try_skip_raw(cur):
                continue
            ch = cur.peek()
            if ch == "\"":
                skip_string(cur)
                continue
            if ch == "'":
                skip_lifetime_or_char(cur)
                continue
            if ch in "([{":
                if ch == "{" and depth == 0:
                    skip_delimited(cur)
                    return
                depth += 1
                cur.skip()
                continue
            if ch in ")]}":
                depth = max(0, depth - 1)
                cur.skip()
                continue
            if ch == ";" and depth == 0:
                cur.skip()
                return
            cur.skip()

    def expand(self, name, generator, body, inv_ident, module_path, rel, line, start, end, cfg):
        tests = generator["tests"]
        ignored = None
        if name in {"baseline_combo_tests", "baseline_case", "baseline_case_with_variants", "audit_matrix_tests"}:
            combo = self.generators.get("baseline_combo_tests", generator)
            tests = combo["tests"]
            ignored = "visual baseline capture; run with --ignored"
            mod_name = inv_ident or first_invocation_ident(body)
            require(mod_name, name + " missing module ident")
            for test in tests:
                identity = "::".join(module_path + [mod_name, test])
                self.emit(identity, rel, line, line, start, end, "macro", name, cfg, ignored, [])
            return
        if generator["kind"] == "suite-entries":
            modules = parse_suite_entries(body)
            require(modules, name + " invocation produced no suite entries")
            for mod_name in modules:
                for test in tests:
                    identity = "::".join(module_path + [mod_name, test])
                    self.emit(identity, rel, line, line, start, end, "macro", name, cfg, None, [])
            return
        mod_name = inv_ident or first_invocation_ident(body)
        require(mod_name, name + " missing module ident")
        for test in tests:
            identity = "::".join(module_path + [mod_name, test])
            self.emit(identity, rel, line, line, start, end, "macro", name, cfg, None, [])


def collect_crate_files(root, crate):
    files = []
    seen = set()

    def visit(rel, file_module):
        if rel in seen:
            return
        seen.add(rel)
        text = (root / rel).read_text()
        files.append((rel, file_module, text))
        cur = Cursor(text)
        walk_mods(cur, rel, file_module, file_module, text, len(text))

    def walk_mods(cur, rel, file_module, module_path, text, limit):
        while cur.i < limit and not cur.done():
            skip_ws_comments(cur)
            if cur.i >= limit:
                break
            attrs = []
            while cur.i < limit and cur.peek() == "#":
                try:
                    attrs.append(parse_attribute(cur))
                except (Invalid, KeyError):
                    cur.skip()
                    break
                skip_ws_comments(cur)
            path_attr = next((p for p in (attr_path(attr_inner(a)) for a in attrs) if p), None)
            skip_qualifiers(cur)
            skip_ws_comments(cur)
            ident = IDENT_RE.match(cur.text, cur.i)
            if ident is None:
                if cur.peek() in "([{":
                    skip_delimited(cur)
                else:
                    cur.skip()
                continue
            word = ident.group()
            if word != "mod":
                cur.i = ident.end()
                if word == "macro_rules":
                    skip_ws_comments(cur)
                    if cur.peek() == "!":
                        cur.skip()
                    if IDENT_RE.match(cur.text, cur.i):
                        parse_ident(cur)
                    skip_ws_comments(cur)
                    if cur.peek() in "[{(":
                        skip_delimited(cur)
                    continue
                depth = 0
                while cur.i < limit and not cur.done():
                    skip_ws_comments(cur)
                    if cur.i >= limit:
                        break
                    ch = cur.peek()
                    if ch in "([{":
                        if ch == "{" and depth == 0:
                            skip_delimited(cur)
                            break
                        depth += 1
                        cur.skip()
                    elif ch in ")]}":
                        depth = max(0, depth - 1)
                        cur.skip()
                    elif ch == ";" and depth == 0:
                        cur.skip()
                        break
                    elif ch == "\"":
                        skip_string(cur)
                    elif ch == "'":
                        skip_lifetime_or_char(cur)
                    else:
                        cur.skip()
                continue
            cur.i = ident.end()
            name = parse_ident(cur)
            skip_ws_comments(cur)
            child = module_path + [name]
            if cur.peek() == ";":
                cur.skip()
                try:
                    child_rel = resolve_mod_path(
                        root, rel, file_module, module_path, name, path_attr, crate["path"])
                except Invalid as error:
                    files.append(("__missing__:" + str(error), child, ""))
                    continue
                visit(child_rel, child)
                continue
            if cur.peek() == "{":
                body_start = cur.i
                skip_delimited(cur)
                inner = Cursor(text, cur.i - 1)
                inner.i = body_start + 1
                walk_mods(inner, rel, file_module, child, text, cur.i - 1)
                continue

    visit(crate["path"], [])
    return files


def discover_trybuild(root, crate, text):
    identities = []
    manifest_dir = root / crate["manifest_dir"]
    for match in re.finditer(r'compile_fail\s*\(\s*"([^"]+)"\s*\)', text):
        pattern = match.group(1)
        matches = sorted(manifest_dir.glob(pattern))
        require(matches, "trybuild glob empty: " + pattern)
        for path in matches:
            if path.suffix != ".rs":
                continue
            rel = posix(path.relative_to(root))
            identities.append({
                "package": crate["package"],
                "kind": crate["kind"],
                "target": crate["target"],
                "identity": path.stem,
                "path": rel,
                "blob_sha256": digest(path.read_bytes()),
                "line": 1,
                "end_line": path.read_text().count("\n") + 1,
                "byte_start": 0,
                "byte_end": path.stat().st_size,
                "origin": "trybuild",
                "macro": None,
                "cfg": [],
                "ignored": None,
                "assertions": [],
                "classification": "preserve",
            })
    return identities


def discover(root, seed_text=None, seed_expected=146):
    root = Path(root).resolve()
    members = parse_workspace_members(root)
    identities = []
    blockers = []
    scanned = []
    roots = []
    for member in members:
        roots.extend(manifest_targets(root, str(Path(member) / "Cargo.toml")))
    require(roots, "no crate roots")
    collected = []
    generators = {}
    for crate in roots:
        if crate["kind"] == "bench":
            blockers.append({
                "package": crate["package"],
                "kind": crate["kind"],
                "target": crate["target"],
                "reason": "benchmark target requires separate benchmark coverage",
            })
            continue
        files = collect_crate_files(root, crate)
        missing = [item for item in files if item[0].startswith("__missing__:")]
        for item in missing:
            blockers.append({"reason": item[0].removeprefix("__missing__:"), "package": crate["package"]})
        files = [item for item in files if not item[0].startswith("__missing__:")]
        for rel, _module, text in files:
            generators.update(extract_generators(text))
        collected.append((crate, files))
    for crate, files in collected:
        discovery = Discovery(root, crate, generators)
        for rel, file_module, text in files:
            discovery.parse_file(rel, file_module, text)
            identities.extend(discover_trybuild(root, crate, text))
        identities.extend(discovery.identities)
        blockers.extend(discovery.blockers)
        scanned.extend(discovery.scanned)
    seed_paths = load_seed_paths(seed_text, seed_expected) if seed_text is not None else []
    scanned_set = set(scanned)
    missing_seed = [path for path in seed_paths if not (root / path).is_file()]
    require(not missing_seed, "seed path missing: " + ", ".join(missing_seed[:5]))
    for path in seed_paths:
        if path not in scanned_set:
            blockers.append({"reason": "seed path not reached through crate mod graph", "path": path})
    keys = [(row["package"], row["kind"], row["target"], row["identity"]) for row in identities]
    unique(keys, "source-discovered identity")
    identities.sort(key=lambda row: (row["package"], row["kind"], row["target"], row["identity"], row["path"]))
    for row in identities:
        if row["origin"] == "rustdoc":
            blockers.append({
                "package": row["package"],
                "kind": "doc",
                "target": row["target"],
                "identity": row["identity"],
                "reason": "cargo-nextest 0.9.143 has no doctest runner; explicit adapter required",
                "path": row["path"],
                "line": row["line"],
            })
    return {
        "schema": 1,
        "classification": "source-discovered-not-executed",
        "identities": identities,
        "blockers": blockers,
        "scanned": sorted(set(scanned)),
        "seed_paths": seed_paths,
        "crate_roots": roots,
    }


def write_discovery(result, output, root):
    require(not output.exists(), "refuse existing output")
    payload = dict(result)
    payload["source_sha256"] = digest(encoded([
        (path, digest((root / path).read_bytes()))
        for path in result["scanned"] if (root / path).is_file()
    ]))
    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open("x") as stream:
        json.dump(payload, stream, indent=2)
        stream.write("\n")
    return payload
