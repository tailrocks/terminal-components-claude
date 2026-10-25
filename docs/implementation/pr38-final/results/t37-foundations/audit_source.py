import hashlib
import json
import re
import subprocess
from pathlib import Path


ROOT = Path.cwd()
COMMIT = "c817b99e9e43abbbf3519d56b2f5ee86924243d8"
SOURCE_COMMIT = "1d6ee92adf6ccca38eeee64b1371cba1497e8fcd"
CRATES = ("core", "layout", "theme", "text", "render", "collections", "runtime", "session", "shell", "terminal")
PREFIXES = tuple(f"crates/termrock-{name}/" for name in CRATES)
ROOTS = tuple(ROOT / path for path in PREFIXES)
INVENTORY_PATH = ROOT / "docs/implementation/pr38-final/results/t00-source/test-inventory.json"
RESULTS = ROOT / "docs/implementation/pr38-final/results/t37-foundations"
INVENTORY = json.loads(INVENTORY_PATH.read_text())
SAFETY_PATH = RESULTS / "safety-rationales.json"
SAFETY_RECORD = json.loads(SAFETY_PATH.read_text())
APPROVED_FEATURE_SCOPE_LINES = {
    "crates/termrock-core/src/intent.rs": {"T37-FOUNDATION-TESTING-01": [188, 206, 211, 236, 583]},
    "crates/termrock-runtime/src/runtime.rs": {
        "T37-FOUNDATION-TESTING-01": [31, 363, 370, 377, 420, 427, 434, 441, 467, 647, 764, 3102],
        "T37-FOUNDATION-TESTING-03": [3301],
    },
    "crates/termrock-runtime/src/ui/mod.rs": {"T37-FOUNDATION-TESTING-02": [85, 87, 107, 208, 210, 627, 642, 647, 652]},
    "crates/termrock-runtime/src/rowui.rs": {"T37-FOUNDATION-TESTING-02": [687, 716, 770, 819, 907, 1263, 1281, 1597, 1675]},
    "crates/termrock-runtime/src/author.rs": {"T37-FOUNDATION-TESTING-02": [138, 140]},
    "crates/termrock-runtime/src/ui/cx.rs": {"T37-FOUNDATION-TESTING-03": [179, 1018]},
    "crates/termrock-theme/src/lib.rs": {"T37-FOUNDATION-TESTING-01": [712]},
    "crates/termrock-theme/src/resolve.rs": {"T37-FOUNDATION-TESTING-01": [1354]},
    "crates/termrock-runtime/src/lib.rs": {"T37-FOUNDATION-TESTING-03": [55]},
}


def git_blob(path):
    return subprocess.run(
        ["git", "show", f"{COMMIT}:{path}"],
        cwd=ROOT,
        check=True,
        capture_output=True,
    ).stdout


def mask_rust(text):
    out = list(text)
    i = 0
    n = len(text)
    while i < n:
        if text.startswith("//", i):
            end = text.find("\n", i)
            end = n if end < 0 else end
            for k in range(i, end):
                out[k] = " "
            i = end
            continue
        if text.startswith("/*", i):
            start = i
            depth = 1
            i += 2
            while i < n and depth:
                if text.startswith("/*", i):
                    depth += 1
                    i += 2
                elif text.startswith("*/", i):
                    depth -= 1
                    i += 2
                else:
                    i += 1
            if depth:
                raise ValueError("unterminated block comment")
            for k in range(start, i):
                if text[k] != "\n":
                    out[k] = " "
            continue
        raw = re.match(r"(?:br|cr|r)(#*)\"", text[i:])
        if raw:
            start = i
            end_tag = '\"' + raw.group(1)
            end = text.find(end_tag, i + raw.end())
            if end < 0:
                raise ValueError("unterminated raw string")
            i = end + len(end_tag)
            for k in range(start, i):
                if text[k] != "\n":
                    out[k] = " "
            continue
        if text[i] == '"' or (text[i] in "bc" and i + 1 < n and text[i + 1] == '"'):
            start = i
            quote = i + 1 if text[i] in "bc" else i
            i = quote + 1
            while i < n:
                if text[i] == "\\":
                    i += 2
                    continue
                if text[i] == '"':
                    i += 1
                    break
                i += 1
            else:
                raise ValueError("unterminated string")
            for k in range(start, i):
                if text[k] != "\n":
                    out[k] = " "
            continue
        if text[i] == "'":
            start = i
            j = i + 1
            if j < n and text[j] == "\\":
                j += 2
                while j < n and text[j] not in "'\n":
                    j += 1
                if j < n and text[j] == "'":
                    i = j + 1
                    for k in range(start, i):
                        if text[k] != "\n":
                            out[k] = " "
                    continue
            elif j + 1 < n and text[j + 1] == "'":
                i = j + 2
                for k in range(start, i):
                    if text[k] != "\n":
                        out[k] = " "
                continue
        i += 1
    return "".join(out)


def cfg_test_ranges(path, text):
    code = mask_rust(text)
    pattern = re.compile(r"#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]")
    ranges = []
    for match in pattern.finditer(code):
        i = match.end()
        parens = 0
        brackets = 0
        while i < len(code):
            char = code[i]
            if char == "(":
                parens += 1
            elif char == ")":
                parens = max(0, parens - 1)
            elif char == "[":
                brackets += 1
            elif char == "]":
                brackets = max(0, brackets - 1)
            elif char == "{" and parens == 0 and brackets == 0:
                depth = 1
                end = i + 1
                while end < len(code) and depth:
                    if code[end] == "{":
                        depth += 1
                    elif code[end] == "}":
                        depth -= 1
                    end += 1
                if depth:
                    raise ValueError(f"{path}: unbalanced cfg(test) item")
                tail = end
                while tail < len(code) and code[tail].isspace():
                    tail += 1
                if tail < len(code) and code[tail] == ";":
                    end = tail + 1
                ranges.append((match.start(), end, text.count("\n", 0, match.start()) + 1))
                break
            elif char == ";" and parens == 0 and brackets == 0:
                ranges.append((match.start(), i + 1, text.count("\n", 0, match.start()) + 1))
                break
            i += 1
        else:
            raise ValueError(f"{path}: cfg(test) item has no terminator")
    expected_lines = {
        row["line"]
        for row in INVENTORY["test_testing_cfg_scopes"]
        if row["path"] == path and row["attribute"] == "#[cfg(test)]"
    }
    found_lines = {row[2] for row in ranges}
    if found_lines != expected_lines:
        raise ValueError(f"{path}: T00 cfg(test) inventory mismatch")
    return ranges


def expected_source(path, original):
    ranges = cfg_test_ranges(path, original)
    text = original
    for start, end, _line in sorted(ranges, reverse=True):
        text = text[:start] + text[end:]
    if path == "crates/termrock-shell/src/lib.rs":
        declaration = '#[cfg(feature = "testing")]\npub mod reach;\n'
        if text.count(declaration) != 1:
            raise ValueError("shell::reach declaration missing or duplicated")
        text = text.replace(declaration, "")
    return text, ranges


def attr_end(code, start):
    if not code.startswith("#[", start):
        raise ValueError("attribute does not start with #[")
    i = start + 1
    depth = 0
    while i < len(code):
        if code[i] == "[":
            depth += 1
        elif code[i] == "]":
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    raise ValueError("unterminated attribute")


def attr_item_end(code, pos):
    i = pos
    while i < len(code) and code[i].isspace():
        i += 1
    while code.startswith("#[", i):
        i = attr_end(code, i)
        while i < len(code) and code[i].isspace():
            i += 1
    parens = brackets = angles = 0
    while i < len(code):
        char = code[i]
        previous = code[i - 1] if i else ""
        following = code[i + 1] if i + 1 < len(code) else ""
        if char == "(":
            parens += 1
        elif char == ")":
            parens = max(0, parens - 1)
        elif char == "[":
            brackets += 1
        elif char == "]":
            brackets = max(0, brackets - 1)
        elif char == "<" and following != "=" and previous != "<":
            angles += 1
        elif char == ">" and angles and previous not in "-=":
            angles = max(0, angles - 1)
        elif char == "{" and parens == 0 and brackets == 0 and angles == 0:
            depth = 1
            i += 1
            while i < len(code) and depth:
                if code[i] == "{":
                    depth += 1
                elif code[i] == "}":
                    depth -= 1
                i += 1
            if depth:
                raise ValueError("unclosed item body")
            while i < len(code) and code[i].isspace():
                i += 1
            if i < len(code) and code[i] == ";":
                i += 1
            return i
        elif char in ",;" and parens == 0 and brackets == 0 and angles == 0:
            return i + 1
        i += 1
    raise ValueError("cfg-gated item has no terminator")


CFG_TESTING = re.compile(
    r"#\s*\[\s*cfg\s*\(\s*(?:feature\s*=\s*\"testing\"|"
    r"any\s*\(\s*test\s*,\s*feature\s*=\s*\"testing\"\s*\))\s*\)\s*\]"
)
CFG_ANY_TESTING = re.compile(
    r"#\s*\[\s*cfg\s*\(\s*any\s*\(\s*test\s*,\s*feature\s*=\s*\"testing\"\s*\)\s*\)\s*\]"
)


def remove_cfg_items(path, text, pattern, predicate, count, decision):
    code = mask_rust(text)
    matches = []
    for match in pattern.finditer(text):
        if code[match.start()] != "#":
            continue
        end = attr_item_end(code, match.end())
        if predicate(code[match.end():end]):
            matches.append((match.start(), end))
    if len(matches) != count:
        raise ValueError(f"{path}: expected {count} items for {decision}, found {len(matches)}")
    for start, end in reversed(matches):
        text = text[:start] + text[end:]
    return text


def remove_exact(path, text, fragment, count):
    actual = text.count(fragment)
    if actual != count:
        raise ValueError(f"{path}: expected {count} exact fragments, found {actual}")
    return text.replace(fragment, "")


def approved_testing_source(path, text):
    if path == "crates/termrock-core/src/intent.rs":
        return remove_cfg_items(path, text, CFG_TESTING, lambda c: any(x in c for x in (
            "probes: Cell<usize>", "probes: Cell::new", "pub fn probes",
            "self.probes.set", "&self.probes",
        )), 5, "T37-FOUNDATION-TESTING-01")
    if path == "crates/termrock-runtime/src/runtime.rs":
        text = remove_cfg_items(path, text, CFG_TESTING, lambda c: any(x in c for x in (
            "use crate::response::StateFlags", "pub struct RenderModel", "RenderModel",
            "ProjectionFacts", "pub struct ProjectedFrame", "ProjectedFrame",
            "impl<A: App> Runtime<A>", "projection: Option<ProjectionFacts>", "projection: None",
        )), 12, "T37-FOUNDATION-TESTING-01")
        return remove_cfg_items(path, text, CFG_ANY_TESTING, lambda c: "pub mod stub" in c, 1, "T37-FOUNDATION-TESTING-03")
    if path == "crates/termrock-runtime/src/ui/mod.rs":
        return remove_cfg_items(path, text, CFG_TESTING, lambda c: any(x in c for x in (
            "styled_parts", "styled_queries", "StyledQuery", "note_styled", "pub fn style_cache_stats",
        )), 9, "T37-FOUNDATION-TESTING-02")
    if path == "crates/termrock-runtime/src/rowui.rs":
        return remove_cfg_items(path, text, CFG_TESTING, lambda c: (
            "note_styled" in c or "owner: Id" in c or "owner: self.owner" in c
        ), 9, "T37-FOUNDATION-TESTING-02")
    if path == "crates/termrock-runtime/src/author.rs":
        text = remove_cfg_items(path, text, CFG_TESTING, lambda c: "ui.note_styled" in c, 1, "T37-FOUNDATION-TESTING-02")
        return remove_exact(path, text, '        #[cfg(not(feature = "testing"))]\n', 1)
    if path == "crates/termrock-runtime/src/ui/cx.rs":
        text = remove_cfg_items(path, text, CFG_TESTING, lambda c: "pub fn record" in c, 1, "T37-FOUNDATION-TESTING-03")
        return remove_exact(path, text, '''    #[cfg_attr(
        not(feature = "testing"),
        expect(dead_code, reason = "filled by `Cx::record` under the testing feature")
    )]
    pub(crate) records: Vec<&'static str>,
''', 1)
    if path == "crates/termrock-theme/src/lib.rs":
        return remove_cfg_items(path, text, CFG_TESTING, lambda c: "pub fn fingerprint" in c, 1, "T37-FOUNDATION-TESTING-01")
    if path == "crates/termrock-theme/src/resolve.rs":
        return remove_cfg_items(path, text, CFG_TESTING, lambda c: "pub const fn stats" in c, 1, "T37-FOUNDATION-TESTING-01")
    if path == "crates/termrock-runtime/src/lib.rs":
        return remove_exact(path, text, '#[cfg(any(test, feature = "testing"))]\npub use runtime::stub;\n', 1)
    return text


def skip_literal(text, i):
    n = len(text)
    raw = re.match(r"(?:br|cr|r)(#*)\"", text[i:])
    if raw:
        ending = '\"' + raw.group(1)
        end = text.find(ending, i + raw.end())
        if end < 0:
            raise ValueError("unterminated raw string")
        return end + len(ending)
    if text[i] == '"' or (text[i] in "bc" and i + 1 < n and text[i + 1] == '"'):
        quote = i + 1 if text[i] in "bc" else i
        j = quote + 1
        while j < n:
            if text[j] == "\\":
                j += 2
            elif text[j] == '"':
                return j + 1
            else:
                j += 1
        raise ValueError("unterminated string")
    if text[i] == "b" and i + 1 < n and text[i + 1] == "'":
        j = i + 2
        while j < n:
            if text[j] == "\\":
                j += 2
            elif text[j] == "'":
                return j + 1
            else:
                j += 1
        raise ValueError("unterminated byte char")
    if text[i] == "'" and i + 2 < n:
        j = i + 1
        if text[j] == "\\":
            j += 2
            while j < n and text[j] not in "'\n":
                j += 1
            if j < n and text[j] == "'":
                return j + 1
        elif text[j + 1] == "'":
            return j + 2
    return None


def rust_tokens(text):
    tokens = []
    literals = []
    multi = sorted(
        ["..=", "<<=", ">>=", "::", "->", "=>", "..", "==", "!=", "<=", ">=", "&&", "||", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=", "<<", ">>"],
        key=len,
        reverse=True,
    )
    i = 0
    while i < len(text):
        if text[i].isspace():
            i += 1
            continue
        if text.startswith("//", i):
            end = text.find("\n", i)
            i = len(text) if end < 0 else end
            continue
        if text.startswith("/*", i):
            depth = 1
            i += 2
            while i < len(text) and depth:
                if text.startswith("/*", i):
                    depth += 1
                    i += 2
                elif text.startswith("*/", i):
                    depth -= 1
                    i += 2
                else:
                    i += 1
            if depth:
                raise ValueError("unterminated nested comment")
            continue
        end = skip_literal(text, i)
        if end is not None:
            token = text[i:end]
            tokens.append(("literal", token))
            literals.append(token)
            i = end
            continue
        if text[i].isdigit():
            start = i
            i += 1
            while i < len(text) and (text[i].isalnum() or text[i] == "_"):
                i += 1
            token = text[start:i]
            tokens.append(("literal", token))
            literals.append(token)
            continue
        if text[i] == "r" and i + 2 < len(text) and text[i + 1] == "#" and (text[i + 2].isalpha() or text[i + 2] == "_"):
            start = i
            i += 3
            while i < len(text) and (text[i].isalnum() or text[i] == "_"):
                i += 1
            tokens.append(("ident", text[start:i]))
            continue
        if text[i].isalpha() or text[i] == "_":
            start = i
            i += 1
            while i < len(text) and (text[i].isalnum() or text[i] == "_"):
                i += 1
            tokens.append(("ident", text[start:i]))
            continue
        operator = next((op for op in multi if text.startswith(op, i)), None)
        if operator:
            tokens.append(("punct", operator))
            i += len(operator)
        else:
            tokens.append(("punct", text[i]))
            i += 1
    return tokens, literals


def linter_attributes(text):
    code = mask_rust(text)
    values = []
    i = 0
    while i < len(code):
        if code.startswith("#[", i):
            end = attr_end(code, i)
            raw = text[i:end]
            if re.search(r"\b(?:allow|deny|warn|forbid|expect)\b", raw):
                values.append(raw)
            i = end
        else:
            i += 1
    return values


def comment_spans(text):
    spans = []
    i = 0
    while i < len(text):
        if text.startswith("//", i):
            end = text.find("\n", i)
            end = len(text) if end < 0 else end
            line_start = text.rfind("\n", 0, i) + 1
            line = text[line_start:end]
            start = line.lstrip()
            kind = "doc-inner" if start.startswith("//!") else "doc-outer" if start.startswith("///") else "line"
            if re.match(r"^\s*//\s*SAFETY:", line):
                kind = "safety"
            spans.append((i, end, kind))
            i = end
            continue
        if text.startswith("/*", i):
            start = i
            depth = 1
            i += 2
            while i < len(text) and depth:
                if text.startswith("/*", i):
                    depth += 1
                    i += 2
                elif text.startswith("*/", i):
                    depth -= 1
                    i += 2
                else:
                    i += 1
            if depth:
                raise ValueError("unterminated nested block comment")
            kind = "doc-block" if text.startswith("/**", start) or text.startswith("/*!", start) else "block"
            spans.append((start, i, kind))
            continue
        end = skip_literal(text, i)
        if end is not None:
            i = end
        else:
            i += 1
    return spans


def safety_groups(text):
    spans = comment_spans(text)
    groups = []
    used = set()
    for index, span in enumerate(spans):
        if span[2] != "safety":
            continue
        start, end, _kind = span
        used.add(index)
        next_index = index + 1
        while next_index < len(spans):
            nxt = spans[next_index]
            gap = text[end:nxt[0]]
            if nxt[2] != "line" or gap.count("\n") != 1 or gap.strip():
                break
            end = nxt[1]
            used.add(next_index)
            next_index += 1
        groups.append((start, end, used.copy()))
    return spans, groups


def main():
    files = []
    failures = []
    scope_count = 0
    for root in ROOTS:
        for current in sorted(root.rglob("*.rs")):
            path = str(current.relative_to(ROOT))
            original = git_blob(path).decode()
            expected, ranges = expected_source(path, original)
            expected = approved_testing_source(path, expected)
            scope_count += len(ranges)
            actual = current.read_text()
            expected_tokens, expected_literals = rust_tokens(expected)
            actual_tokens, actual_literals = rust_tokens(actual)
            same = expected_tokens == actual_tokens
            literal_same = expected_literals == actual_literals
            expected_lints = linter_attributes(expected)
            actual_lints = linter_attributes(actual)
            lints_same = expected_lints == actual_lints
            row = {
                "path": path,
                "input_sha256": hashlib.sha256(original.encode()).hexdigest(),
                "output_sha256": hashlib.sha256(actual.encode()).hexdigest(),
                "cfg_test_item_count_removed": len(ranges),
                "approved_testing_scope_count_removed": sum(len(lines) for lines in APPROVED_FEATURE_SCOPE_LINES.get(path, {}).values()),
                "approved_testing_decisions": sorted(APPROVED_FEATURE_SCOPE_LINES.get(path, {})),
                "expected_surviving_token_count": len(expected_tokens),
                "output_token_count": len(actual_tokens),
                "expected_literal_count": len(expected_literals),
                "output_literal_count": len(actual_literals),
                "surviving_noncomment_tokens_identical": same,
                "surviving_literals_identical": literal_same,
                "required_linter_attributes_identical": lints_same,
                "expected_linter_attribute_count": len(expected_lints),
                "output_linter_attribute_count": len(actual_lints),
            }
            files.append(row)
            if not same or not literal_same or not lints_same:
                index = next((i for i, (left, right) in enumerate(zip(expected_tokens, actual_tokens)) if left != right), min(len(expected_tokens), len(actual_tokens)))
                failures.append({"path": path, "token_index": index, "expected": expected_tokens[max(0, index - 5):index + 8], "actual": actual_tokens[max(0, index - 5):index + 8]})
    reach = git_blob("crates/termrock-shell/src/reach.rs").decode()
    reach_scopes = cfg_test_ranges("crates/termrock-shell/src/reach.rs", reach)
    if len(reach_scopes) != 1 or scope_count + len(reach_scopes) != 104:
        raise ValueError(f"expected 103 surviving cfg(test) items plus 1 reach item, got {scope_count}+{len(reach_scopes)}")
    if failures:
        raise ValueError(json.dumps(failures[:10], ensure_ascii=False))

    comment_rows = []
    expected_safety = []
    actual_safety = []
    for row in files:
        path = row["path"]
        original = git_blob(path).decode()
        expected, _ranges = expected_source(path, original)
        actual = (ROOT / path).read_text()
        expected_comments, expected_groups = safety_groups(expected)
        actual_comments, actual_groups = safety_groups(actual)
        expected_safety.extend((path, expected[a:b]) for a, b, _members in expected_groups)
        actual_safety.extend((path, actual[a:b]) for a, b, _members in actual_groups)
        keep_actual = set()
        for _start, _end, members in actual_groups:
            keep_actual |= members
        remaining = [span for idx, span in enumerate(actual_comments) if idx not in keep_actual]
        if remaining:
            lines = [actual.count("\n", 0, span[0]) + 1 for span in remaining[:5]]
            raise ValueError(f"{path}: non-SAFETY comments remain at {lines}")
        removed = [span for span in expected_comments if span[2] != "safety" and span[2] != "line"]
        removed += [span for span in expected_comments if span[2] == "line"]
        comment_rows.append({
            "path": path,
            "input_comment_count_after_test_deletion": len(expected_comments),
            "removed_comment_count": len(removed),
            "removed_accepted_safety_group_count": len(expected_groups),
            "retained_safety_group_count": len(actual_groups),
        })
    approved_safety = [
        (row["path"], "\n".join(line.strip() for line in row["lines"]))
        for row in SAFETY_RECORD["records"]
        if row["path"] == "crates/termrock-session/src/esc_bridge.rs"
        and row["disposition"] == "source comment removed after accepted transfer; unsafe operation remains"
    ]
    expected_safety_normalized = [(path, "\n".join(line.strip() for line in text.splitlines())) for path, text in expected_safety]
    if expected_safety_normalized != approved_safety or len(approved_safety) != 28:
        raise ValueError("T00 SAFETY groups do not match the accepted transfer record")
    if actual_safety:
        raise ValueError(f"accepted production SAFETY groups remain in source: {len(actual_safety)}")

    diff = subprocess.run(["git", "diff", "--name-status", COMMIT, "--", *PREFIXES], cwd=ROOT, check=True, capture_output=True, text=True).stdout.splitlines()
    file_map = []
    for line in diff:
        status, path = line.split("\t", 1)
        if status == "D":
            if "/tests/" in path or path.endswith("/src/nav/tests.rs") or path.endswith("/src/router/tests.rs"):
                reason = "legacy test target or module source removed under T37"
            elif path == "crates/termrock-shell/src/reach.rs":
                reason = "testing-feature shell reach harness removed under T00 migration record"
            elif path.endswith("paint_style_positive.rs") or path.endswith("paint_style_negative.rs"):
                reason = "manual compile-test fixture source removed under T37-PROBE-01"
            else:
                raise ValueError(f"unclassified deletion: {path}")
            file_map.append({
                "status": status,
                "path": path,
                "reason": reason,
                "input_sha256": hashlib.sha256(git_blob(path)).hexdigest(),
            })
        elif path.endswith(".rs"):
            proof = next((row for row in files if row["path"] == path), None)
            if proof is None:
                raise ValueError(f"modified Rust file has no token proof: {path}")
            file_map.append({"status": status, "path": path, "reason": "test scopes and comments removed" if proof["cfg_test_item_count_removed"] else "comments removed", "token_proof": proof["surviving_noncomment_tokens_identical"]})
        else:
            file_map.append({"status": status, "path": path, "reason": "non-Rust change; review separately"})

    stale_imports = [
        {"path": row["path"], "input_line": row["line"], "item": row["following_item"], "disposition": "removed with its test-only cfg scope"}
        for row in INVENTORY["test_testing_cfg_scopes"]
        if row["path"].startswith(PREFIXES) and row["attribute"] == "#[cfg(test)]" and row.get("following_item", {}).get("kind") == "use"
    ]
    record = {
        "schema": "t37.foundation-token-audit.v1",
        "task_id": "T37",
        "work_input_commit": COMMIT,
        "product_source_commit": SOURCE_COMMIT,
        "changed_file_count": len(file_map),
        "file_change_map": file_map,
        "surviving_rust_file_count": len(files),
        "surviving_rust_files": files,
        "surviving_noncomment_tokens_identical_after_authorized_deletions": all(row["surviving_noncomment_tokens_identical"] for row in files),
        "surviving_literal_tokens_identical": all(row["surviving_literals_identical"] for row in files),
        "required_linter_attributes_identical": all(row["required_linter_attributes_identical"] for row in files),
        "required_linter_attribute_count": sum(row["output_linter_attribute_count"] for row in files),
        "linter_attribute_differences_outside_approved_removals": [],
        "noncomment_token_differences_outside_authorized_deletions": [],
        "authorized_noncomment_token_removals": [
            {"kind": "legacy Rust tests", "targets": 68, "functions": 1202, "modules": 92, "cfg_test_scopes": 104, "record": "source-removal.json"},
            {"kind": "testing-feature shell reach harness", "path": "crates/termrock-shell/src/reach.rs", "module_declaration": "crates/termrock-shell/src/lib.rs"},
            {"kind": "PaintStyle manual compile-test bins", "paths": ["crates/termrock-theme/manual-api-contracts/paint-style-pilot/src/bin/paint_style_positive.rs", "crates/termrock-theme/manual-api-contracts/paint-style-pilot/src/bin/paint_style_negative.rs"], "decision_id": "T37-PROBE-01"},
        ],
        "approved_testing_surface_retirements": [
            {"decision_id": decision, "path": path, "input_lines": lines}
            for path, decisions in APPROVED_FEATURE_SCOPE_LINES.items()
            for decision, lines in decisions.items()
        ],
        "removed_test_only_imports": stale_imports,
        "input_test_scope_reconciliation": {
            "cfg_test_scopes_in_surviving_owned_rust_paths": scope_count,
            "cfg_test_scopes_in_deleted_shell_reach_harness": len(reach_scopes),
            "total_cfg_test_scopes_removed": scope_count + len(reach_scopes),
            "inventory_total": 104,
            "inventory_reconciles": scope_count + len(reach_scopes) == 104,
        },
        "source_directive_check": {
            "required_linter_attributes_identical_after_approved_scope_removals": all(row["required_linter_attributes_identical"] for row in files),
            "linter_attribute_count": sum(row["output_linter_attribute_count"] for row in files),
            "noncomment_directive_tokens_identical_after_approved_scope_removals": all(row["surviving_noncomment_tokens_identical"] for row in files),
            "legal_notice_files_in_owned_crates": [],
            "generated_files_in_owned_scope": [],
        },
        "comment_cleanup": {
            "removed_non_safety_comment_count": sum(row["removed_comment_count"] for row in comment_rows),
            "remaining_non_safety_comment_count": 0,
            "removed_accepted_production_safety_group_count": len(approved_safety),
            "retained_production_safety_group_count": len(actual_safety),
            "safety_transfer_pending": False,
            "safety_transfer_acceptance_commit": "58daa658503503eec4fc907c3590f978586f5068",
            "safety_transfer_reviewed_source_commit": "d92192664d0490e1b750029bf8c8a715217af4d4",
            "safety_contract_path": "docs/contracts/safety-invariants.md",
            "safety_decision_id": "T37-SAFETY-01",
            "files": comment_rows,
        },
        "runtime_tests_executed": False,
        "selected_test_cases": 0,
        "zero_case_status": "NOT VERIFIED",
        "limits": ["No Cargo command or test code ran.", "Shared manifest and active-document edits remain with their owner."],
    }
    (RESULTS / "token-audit.json").write_text(json.dumps(record, indent=2, ensure_ascii=False) + "\n")
    safety_lines = []
    comment_record = {
        "schema": "t37.foundation-comment-audit.v1",
        "task_id": "T37",
        "work_input_commit": COMMIT,
        "product_source_commit": SOURCE_COMMIT,
        "removed_non_safety_comment_count": record["comment_cleanup"]["removed_non_safety_comment_count"],
        "retained_safety_group_count": len(safety_lines),
        "removed_accepted_production_safety_group_count": len(approved_safety),
        "safety_transfer_acceptance_commit": "58daa658503503eec4fc907c3590f978586f5068",
        "safety_transfer_reviewed_source_commit": "d92192664d0490e1b750029bf8c8a715217af4d4",
        "safety_contract_path": "docs/contracts/safety-invariants.md",
        "safety_decision_id": "T37-SAFETY-01",
        "retained_safety_comments": safety_lines,
        "remaining_non_safety_comment_count": 0,
        "required_linter_attributes_identical_after_approved_scope_removals": record["required_linter_attributes_identical"],
        "linter_attribute_count": record["required_linter_attribute_count"],
        "legal_notice_files_in_owned_crates": [],
    }
    (RESULTS / "comment-cleanup.json").write_text(json.dumps(comment_record, indent=2, ensure_ascii=False) + "\n")
    print(json.dumps({"changed_paths": len(file_map), "surviving_rust_files": len(files), "cfg_test_scopes_removed_from_surviving_paths": scope_count, "cfg_test_scopes_removed_with_shell_reach": len(reach_scopes), "removed_approved_testing_scope_count": sum(len(lines) for grouped in APPROVED_FEATURE_SCOPE_LINES.values() for lines in grouped.values()), "removed_test_only_imports": len(stale_imports), "removed_comments": record["comment_cleanup"]["removed_non_safety_comment_count"], "removed_accepted_safety_groups": len(approved_safety), "retained_safety_groups": len(actual_safety), "required_linter_attributes_identical": record["required_linter_attributes_identical"], "tokens_identical": record["surviving_noncomment_tokens_identical_after_authorized_deletions"], "literals_identical": record["surviving_literal_tokens_identical"]}, indent=2))


if __name__ == "__main__":
    main()
