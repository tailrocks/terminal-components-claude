#!/usr/bin/env python3

from __future__ import annotations

import argparse
import collections
import difflib
import hashlib
import json
import re
import subprocess
from dataclasses import dataclass
from pathlib import Path

BASE = "c817b99e9e43abbbf3519d56b2f5ee86924243d8"
ROOT = Path(__file__).resolve().parents[5]
RESULT = ROOT / "docs/implementation/pr38-final/results/t37-apps"
INVENTORY = ROOT / "docs/implementation/pr38-final/results/t00-source/test-inventory.json"
PACKAGES = (
    "showcase-data", "showcase-demos", "showcase-ui", "showcase",
    "jackin-preview-domain", "jackin-preview-sim", "jackin-preview-presentation",
    "jackin-preview-host-ui", "jackin-preview-capsule-ui", "jackin-preview-app",
    "jackin-preview", "holla-domain", "holla-catalog", "holla-plan", "holla-sim",
    "holla-ui", "holla", "tablepro-domain", "tablepro-sql", "tablepro-demo",
    "tablepro-ui", "tablepro", "termrock-consumer-example",
)
PREFIXES = tuple("crates/" + p + "/" for p in PACKAGES)
EXTRA_TEST_FILES = {
    "crates/holla-ui/src/app/disk_scan_tests.rs",
    "crates/holla-ui/src/app/disk_tests.rs",
    "crates/holla-ui/src/app/files_tests.rs",
    "crates/holla-ui/src/app/historical_tests.rs",
    "crates/holla-ui/src/app/port_args_tests.rs",
    "crates/holla-ui/src/app/task_args_tests.rs",
    "crates/holla-ui/src/app/task_run_tests.rs",
    "crates/tablepro-ui/src/app/connection_form_focus.rs",
    "crates/tablepro-ui/src/app/explorer_collapsed.rs",
    "crates/tablepro-ui/src/app/picker_contract.rs",
}
MANUAL_TEST_FILES = {
    "crates/tablepro-ui/manual-api-contracts/workbench-tabs-privacy/README.md",
    "crates/tablepro-ui/manual-api-contracts/workbench-tabs-privacy/src/bin/workbench_tabs_positive.rs",
    "crates/tablepro-ui/manual-api-contracts/workbench-tabs-privacy/src/bin/workbench_tabs_negative.rs",
}
UNGUARDED_TEST_HELPERS = {
    ("crates/holla-sim/src/fixtures.rs", "PortConflictCase", "struct"),
    ("crates/holla-sim/src/fixtures.rs", "port_conflict_case", "fn"),
    ("crates/tablepro-ui/src/app.rs", "form_open_for_test", "fn"),
}
EMPTY_TEST_MODULES = {
    ("crates/tablepro-ui/src/app.rs", "connection_form_focus"),
    ("crates/tablepro-ui/src/app.rs", "explorer_collapsed"),
}


@dataclass(frozen=True)
class Token:
    text: str
    start: int
    end: int
    kind: str


def git_bytes(*args: str) -> bytes:
    return subprocess.check_output(["git", *args], cwd=ROOT)


def git_paths() -> list[str]:
    raw = git_bytes("ls-tree", "-rz", "--name-only", BASE, "--", "crates")
    return [p.decode() for p in raw.split(b"\0") if p]


def source_bytes(path: str) -> bytes:
    return git_bytes("show", f"{BASE}:{path}")


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def is_scoped(path: str) -> bool:
    return path.startswith(PREFIXES)


def is_test_target(path: str) -> bool:
    return "/tests/" in path or path in EXTRA_TEST_FILES or path in MANUAL_TEST_FILES


def _string_end(s: str, i: int) -> tuple[int, str] | None:
    n = len(s)
    starts = i
    j = i
    if s.startswith(("b", "c"), j) and j + 1 < n and s[j + 1] in ('"', "r"):
        j += 1
    if s.startswith("r", j):
        k = j + 1
        while k < n and s[k] == "#":
            k += 1
        if k < n and s[k] == '"':
            hashes = s[j + 1:k]
            close = '"' + hashes
            end = s.find(close, k + 1)
            return (n if end < 0 else end + len(close), "literal")
    if j < n and s[j] == '"':
        j += 1
        while j < n:
            if s[j] == "\\":
                j += 2
            elif s[j] == '"':
                return min(j + 1, n), "literal"
            else:
                j += 1
        return n, "literal"
    if s[i] == "'":
        j = i + 1
        if j < n and s[j] == "\\":
            j += 2
        elif j < n:
            j += 1
        if j < n and s[j] == "'":
            return j + 1, "literal"
        return None
    return None


def lex(data: bytes) -> tuple[list[Token], list[Token]]:
    s = data.decode("utf-8")
    tokens: list[Token] = []
    comments: list[Token] = []
    n = len(s)
    i = 0
    while i < n:
        if s[i].isspace():
            i += 1
            continue
        if s.startswith("//", i):
            j = s.find("\n", i)
            j = n if j < 0 else j
            comments.append(Token(s[i:j], i, j, "comment"))
            i = j
            continue
        if s.startswith("/*", i):
            j = i + 2
            depth = 1
            while j < n and depth:
                if s.startswith("/*", j):
                    depth += 1
                    j += 2
                elif s.startswith("*/", j):
                    depth -= 1
                    j += 2
                else:
                    j += 1
            comments.append(Token(s[i:j], i, j, "comment"))
            i = j
            continue
        string = _string_end(s, i)
        if string:
            j, kind = string
            tokens.append(Token(s[i:j], i, j, kind))
            i = j
            continue
        if s[i] == "'":
            tokens.append(Token("'", i, i + 1, "punct"))
            i += 1
            continue
        c = s[i]
        if c == "_" or c.isalpha() or ord(c) >= 128:
            j = i + 1
            while j < n and (s[j] == "_" or s[j].isalnum() or ord(s[j]) >= 128):
                j += 1
            tokens.append(Token(s[i:j], i, j, "ident"))
            i = j
            continue
        if c.isdigit():
            j = i + 1
            while j < n and (s[j].isalnum() or s[j] in "._"):
                j += 1
            tokens.append(Token(s[i:j], i, j, "number"))
            i = j
            continue
        tokens.append(Token(c, i, i + 1, "punct"))
        i += 1
    return tokens, comments


def _square_end(tokens: list[Token], bracket_i: int) -> int | None:
    if bracket_i >= len(tokens) or tokens[bracket_i].text != "[":
        return None
    depth = 0
    for j in range(bracket_i, len(tokens)):
        if tokens[j].text == "[":
            depth += 1
        elif tokens[j].text == "]":
            depth -= 1
            if depth == 0:
                return j + 1
    return None


def _attr_end(tokens: list[Token], i: int) -> int | None:
    if i + 1 >= len(tokens) or tokens[i].text != "#" or tokens[i + 1].text != "[":
        return None
    return _square_end(tokens, i + 1)


def _item_start(tokens: list[Token], i: int) -> tuple[int, str, str]:
    j = i
    while j < len(tokens):
        end = _attr_end(tokens, j)
        if end is None:
            break
        j = end
    keywords = {"fn", "mod", "enum", "struct", "const", "static", "type", "trait", "impl", "use"}
    k = j
    while k < len(tokens) and tokens[k].text not in keywords and tokens[k].text not in (";", "{"):
        k += 1
    if k >= len(tokens) or tokens[k].text not in keywords:
        raise ValueError(f"cannot locate item keyword after token {i}")
    name = tokens[k + 1].text if k + 1 < len(tokens) else "<anonymous>"
    return k, tokens[k].text, name


def _item_end(tokens: list[Token], keyword_i: int) -> int:
    paren = bracket = brace = angle = 0
    body_start = None
    for j in range(keyword_i + 1, len(tokens)):
        t = tokens[j].text
        if t == "(":
            paren += 1
        elif t == ")":
            paren -= 1
        elif t == "[":
            bracket += 1
        elif t == "]":
            bracket -= 1
        elif t == "<":
            angle += 1
        elif t == ">" and angle:
            angle -= 1
        elif t == "{" and paren == bracket == brace == angle == 0:
            body_start = j
            break
        elif t == ";" and paren == bracket == brace == angle == 0:
            return j + 1
    if body_start is None:
        raise ValueError(f"cannot locate item body after token {keyword_i}")
    depth = 0
    for j in range(body_start, len(tokens)):
        if tokens[j].text == "{":
            depth += 1
        elif tokens[j].text == "}":
            depth -= 1
            if depth == 0:
                return j + 1
    raise ValueError(f"unclosed item body after token {keyword_i}")


def _cfg_spans(path: str, data: bytes) -> list[dict]:
    tokens, _ = lex(data)
    text = data.decode()
    spans = []
    for i in range(len(tokens) - 6):
        if [t.text for t in tokens[i:i + 7]] != ["#", "[", "cfg", "(", "test", ")", "]"]:
            continue
        start_i = i
        while start_i:
            prior = next(
                (k for k in range(start_i - 1, -1, -1) if _attr_end(tokens, k) == start_i),
                None,
            )
            if prior is None:
                break
            start_i = prior
        item_i, kind, name = _item_start(tokens, i + 7)
        end_i = _item_end(tokens, item_i)
        start = tokens[start_i].start
        end = tokens[end_i - 1].end
        spans.append({
            "kind": kind,
            "name": name,
            "start": start,
            "end": end,
            "line": text[:start].count("\n") + 1,
            "attribute_line": text[:tokens[i].start].count("\n") + 1,
            "end_line": text[:end].count("\n") + 1,
            "sha256": sha(text[start:end].encode()),
            "reason": "cfg(test) item",
        })
    return spans


def _named_item_span(path: str, data: bytes, name: str, expected_kind: str) -> dict:
    tokens, _ = lex(data)
    for i, token in enumerate(tokens):
        if token.text != expected_kind:
            continue
        if i + 1 >= len(tokens) or tokens[i + 1].text != name:
            continue
        end_i = _item_end(tokens, i)
        start_i = i
        if start_i and tokens[start_i - 1].text == ")":
            depth = 0
            open_i = None
            for j in range(start_i - 1, -1, -1):
                if tokens[j].text == ")":
                    depth += 1
                elif tokens[j].text == "(":
                    depth -= 1
                    if depth == 0:
                        open_i = j
                        break
            if open_i is not None and open_i and tokens[open_i - 1].text == "pub":
                start_i = open_i - 1
        elif start_i and tokens[start_i - 1].text == "pub":
            start_i -= 1
        start, end = tokens[start_i].start, tokens[end_i - 1].end
        text = data.decode()
        return {
            "kind": expected_kind, "name": name, "start": start, "end": end,
            "line": text[:start].count("\n") + 1,
            "end_line": text[:end].count("\n") + 1,
            "sha256": sha(text[start:end].encode()),
            "reason": "unguarded test-only helper; all references are in removed test scopes",
        }
    raise ValueError(f"missing helper {path}:{expected_kind} {name}")


def _module_decl_span(data: bytes, name: str) -> dict:
    tokens, _ = lex(data)
    text = data.decode()
    for i in range(len(tokens) - 3):
        if [t.text for t in tokens[i:i + 4]] == ["mod", name, ";", "}"]:
            continue
        if i + 2 < len(tokens) and [t.text for t in tokens[i:i + 3]] == ["mod", name, ";"]:
            start, end = tokens[i].start, tokens[i + 2].end
            return {
                "kind": "mod", "name": name, "start": start, "end": end,
                "line": text[:start].count("\n") + 1,
                "end_line": text[:end].count("\n") + 1,
                "sha256": sha(text[start:end].encode()),
                "reason": "empty module contained only cfg(test) test code",
            }
    raise ValueError(f"missing module declaration {name}")


def _stale_import_span(path: str, data: bytes) -> dict:
    text = data.decode()
    needle = "use crate::listeners::{ListenerModel, ListenerModelError, ListenerRecord, PortConflict};"
    start = text.find(needle)
    if start < 0:
        raise ValueError("expected stale Holla fixture import missing")
    end = start + len(needle)
    return {
        "kind": "use", "name": "ListenerModel, ListenerModelError, ListenerRecord, PortConflict",
        "start": start, "end": end, "line": text[:start].count("\n") + 1,
        "end_line": text[:end].count("\n") + 1, "sha256": sha(text[start:end].encode()),
        "reason": "all names became unused when PortConflictCase and port_conflict_case were removed",
    }


DIRECTIVE = re.compile(
    r"^(?://+|/\\*+!?)[ \t]*(?:SPDX-|Copyright\b|@generated\b|Generated\b|Code generated\b|"
    r"rustfmt::|clippy::|allow\(|deny\(|warn\(|forbid\(|NOLINT\b|noinspection\b|"
    r"spellchecker\b|cargo:|BEGIN GENERATED\b|END GENERATED\b)",
    re.IGNORECASE,
)
SYNTAX_DIRECTIVE = re.compile(
    r"\b(?:cfg|cfg_attr|allow|expect|deny|forbid|warn|path|doc|must_use|deprecated|"
    r"non_exhaustive|repr|macro_use|no_mangle|export_name|link|derive)\b"
)
TEST_FEATURE = re.compile(r"\bcfg(?:_attr)?\b[\s\S]*?\bfeature\s*=\s*['\"](?:test|testing|test-support)['\"]")


def _attribute_evidence(paths: list[str], removed_files: list[str],
                        spans_by_path: dict[str, list[dict]]) -> dict:
    retained = []
    removed = []
    test_feature_gates = []
    for path in paths:
        if not path.endswith(".rs"):
            continue
        data = source_bytes(path)
        text = data.decode()
        tokens, _ = lex(data)
        spans = _merge_spans(spans_by_path.get(path, []))
        for i in range(len(tokens) - 1):
            if tokens[i].text != "#":
                continue
            if tokens[i + 1].text == "[":
                end_i = _attr_end(tokens, i)
            elif i + 2 < len(tokens) and tokens[i + 1].text == "!" and tokens[i + 2].text == "[":
                end_i = _square_end(tokens, i + 2)
            else:
                continue
            if end_i is None:
                raise AssertionError(f"unclosed attribute in {path}:{text[:tokens[i].start].count(chr(10)) + 1}")
            raw = text[tokens[i].start:tokens[end_i - 1].end]
            if not SYNTAX_DIRECTIVE.search(raw):
                continue
            line = text[:tokens[i].start].count("\n") + 1
            deleted_by = None
            if path in removed_files:
                deleted_by = "legacy test target or test-only source module file"
            else:
                for span in spans:
                    if span["start"] <= tokens[i].start < span["end"]:
                        deleted_by = span.get("reason", "removed source item")
                        break
            row = {
                "path": path, "line": line, "source_sha256": sha(data),
                "attribute_sha256": sha(raw.encode()), "attribute": raw,
            }
            if deleted_by:
                row["deleted_by"] = deleted_by
                removed.append(row)
            else:
                retained.append(row)
            if TEST_FEATURE.search(raw):
                test_feature_gates.append({
                    "path": path, "line": line, "attribute": raw,
                    "removed": bool(deleted_by),
                })
    return {
        "input_sha": BASE,
        "comment_directive_comments": "preserved-directives.json",
        "retained_attributes": retained,
        "removed_attributes": removed,
        "test_feature_gates": test_feature_gates,
        "summary": {
            "retained_syntax_directives": len(retained),
            "removed_syntax_directives": len(removed),
            "test_or_testing_feature_gates_found": len(test_feature_gates),
        },
    }


def _cfg_inventory_reconciliation() -> dict:
    inventory = json.loads(INVENTORY.read_text())
    expected = collections.Counter(
        (item["path"], item["line"])
        for item in inventory["test_testing_cfg_scopes"]
        if is_scoped(item.get("path", ""))
    )
    actual = collections.Counter()
    inner_cfg = []
    for path in git_paths():
        if not is_scoped(path) or not path.endswith(".rs"):
            continue
        data = source_bytes(path)
        for item in _cfg_spans(path, data):
            actual[(path, item["attribute_line"])] += 1
        tokens, _ = lex(data)
        text = data.decode()
        for i in range(len(tokens) - 4):
            if [t.text for t in tokens[i:i + 5]] == ["#", "!", "[", "cfg", "("]:
                end_i = _square_end(tokens, i + 2)
                if end_i and [t.text for t in tokens[i + 3:end_i]] == ["cfg", "(", "test", ")", "]"]:
                    inner_cfg.append({
                        "path": path,
                        "line": text[:tokens[i].start].count("\n") + 1,
                        "attribute": text[tokens[i].start:tokens[end_i - 1].end],
                        "file_sha256": sha(data),
                    })
    return {
        "input_sha": BASE,
        "T00_inventory_source_sha": inventory.get("source_commit"),
        "T00_cfg_scope_candidate_count": sum(expected.values()),
        "lexed_outer_cfg_test_attribute_count": sum(actual.values()),
        "inner_cfg_test_attributes_not_in_scope_ledger": inner_cfg,
        "T00_candidates_not_lexed_as_outer_cfg_items": [
            {"path": path, "line": line, "count": count}
            for (path, line), count in sorted((expected - actual).items())
        ],
        "lexed_outer_cfg_items_missing_from_T00": [
            {"path": path, "line": line, "count": count}
            for (path, line), count in sorted((actual - expected).items())
        ],
    }


def _strip_comments(data: bytes, directives: list[dict], path: str) -> tuple[bytes, list[dict]]:
    tokens, comments = lex(data)
    text = data.decode()
    chunks = []
    last = 0
    removed = []
    for c in comments:
        raw = c.text
        if DIRECTIVE.search(raw.lstrip()):
            directives.append({
                "path": path, "line": text[:c.start].count("\n") + 1,
                "sha256": sha(raw.encode()), "text": raw,
            })
            continue
        chunks.append(text[last:c.start])
        chunks.append("".join("\n" if ch == "\n" else " " for ch in raw))
        last = c.end
        removed.append({
            "line": text[:c.start].count("\n") + 1,
            "end_line": text[:c.end].count("\n") + 1,
            "sha256": sha(raw.encode()),
            "doc": raw.startswith(("///", "//!", "/**", "/*!")),
        })
    chunks.append(text[last:])
    cleaned = "".join(chunks)
    cleaned = re.sub(r"[ \t]+(?=\n|$)", "", cleaned)
    ended_with_newline = text.endswith("\n")
    cleaned = cleaned.rstrip(" \t\n")
    if ended_with_newline:
        cleaned += "\n"
    return cleaned.encode(), removed


def _merge_spans(spans: list[dict]) -> list[dict]:
    ordered = sorted(spans, key=lambda s: (s["start"], -s["end"]))
    merged = []
    for span in ordered:
        if merged and span["start"] >= merged[-1]["start"] and span["end"] <= merged[-1]["end"]:
            continue
        if merged and span["start"] < merged[-1]["end"]:
            merged[-1]["end"] = max(merged[-1]["end"], span["end"])
            merged[-1]["sha256"] = None
            merged[-1]["covered_spans"].append(span)
        else:
            item = dict(span)
            item["covered_spans"] = []
            merged.append(item)
    return merged


def _erase_spans(data: bytes, spans: list[dict]) -> bytes:
    text = data.decode()
    for span in sorted(_merge_spans(spans), key=lambda s: s["start"], reverse=True):
        text = text[:span["start"]] + text[span["end"]:]
    return text.encode()


def _tokens(data: bytes) -> list[str]:
    return [t.text for t in lex(data)[0]]


def _literals(data: bytes) -> list[str]:
    return [t.text for t in lex(data)[0] if t.kind == "literal"]


def _const_literals(data: bytes, name: str) -> list[str]:
    tokens, _ = lex(data)
    for i in range(len(tokens) - 2):
        if tokens[i].text != "const" or tokens[i + 1].text != name:
            continue
        values = []
        for token in tokens[i + 2:]:
            if token.text == ";":
                return [t.text for t in tokens[i + 2:] if t.kind == "literal" and t.end <= token.end]
        raise ValueError(f"unterminated const {name}")
    raise ValueError(f"const {name} not found")


def _help_and_diagnostic_evidence() -> dict:
    inventory = json.loads(INVENTORY.read_text())
    help_rows = []
    for item in inventory["functional_cli_help"]:
        if ":" not in item["source"]:
            continue
        source, line = item["source"].rsplit(":", 1)
        if not is_scoped(source):
            continue
        original = source_bytes(source)
        current_path = ROOT / source
        current = current_path.read_bytes()
        before = _const_literals(original, "HELP")
        after = _const_literals(current, "HELP")
        if before != after:
            raise AssertionError(f"CLI HELP literal changed: {source}")
        help_rows.append({
            "source": source, "input_line": int(line), "kind": item["kind"],
            "input_file_sha256": sha(original), "output_file_sha256": sha(current),
            "literal_count": len(before), "literal_stream_sha256": _hash_json(before),
            "literal_stream_equal": True,
        })
    diagnostic_rows = []
    for item in inventory["production_or_mixed_diagnostic_sites"]:
        source = item.get("path", "")
        if not is_scoped(source):
            continue
        original = source_bytes(source)
        current = (ROOT / source).read_bytes()
        line = item.get("line")
        original_text = original.decode().splitlines()
        line_text = original_text[line - 1] if line and line <= len(original_text) else item.get("source")
        diagnostic_rows.append({
            "source": source, "input_line": line, "macro": item.get("macro"),
            "classification": item.get("classification"),
            "input_file_sha256": sha(original), "input_line_text": line_text,
            "output_file_sha256": sha(current),
            "production_token_stream_equal": _hash_json(_tokens(_erase_spans(
                _strip_comments(original, [], source)[0],
                _base_items(source, original),
            ))) == _hash_json(_tokens(current)),
        })
    return {"cli_help": help_rows, "diagnostic_sites": diagnostic_rows}


def _hash_json(value) -> str:
    return sha(json.dumps(value, ensure_ascii=False, separators=(",", ":")).encode())


def _base_items(path: str, data: bytes) -> list[dict]:
    spans = _cfg_spans(path, data)
    spans.extend(_named_item_span(path, data, name, kind)
                 for p, name, kind in UNGUARDED_TEST_HELPERS if p == path)
    if path == "crates/tablepro-ui/src/app.rs":
        spans.extend(_module_decl_span(data, name) for p, name in EMPTY_TEST_MODULES if p == path)
    if path == "crates/holla-sim/src/fixtures.rs":
        spans.append(_stale_import_span(path, data))
    return spans


def plan() -> tuple[list[str], dict[str, list[dict]]]:
    paths = [p for p in git_paths() if is_scoped(p)]
    removed_files = [p for p in paths if is_test_target(p)]
    source_spans = {}
    for path in paths:
        if path.endswith(".rs") and path not in removed_files:
            spans = _base_items(path, source_bytes(path))
            if spans:
                source_spans[path] = spans
    extra = set(removed_files)
    for path, spans in source_spans.items():
        data = source_bytes(path)
        text = data.decode()
        tokens, _ = lex(data)
        for span in spans:
            if span["reason"] != "cfg(test) item" or span["kind"] != "mod":
                continue
            if data[span["start"]:span["end"]].rstrip().endswith(b";"):
                stem = Path(path)
                parent = stem.parent / stem.stem
                candidates = [str(parent / f"{span['name']}.rs"), str(parent / span["name"] / "mod.rs")]
                candidates += [str(stem.parent / f"{span['name']}.rs"), str(stem.parent / span["name"] / "mod.rs")]
                for candidate in candidates:
                    if candidate in paths:
                        extra.add(candidate)
                        break
    return sorted(extra), source_spans


def _capture_prose(paths: list[str], spans_by_path: dict[str, list[dict]]) -> list[dict]:
    snippets = []
    for path in paths:
        if not path.endswith(".rs") or is_test_target(path):
            continue
        data = source_bytes(path)
        text = data.decode()
        _, comments = lex(data)
        removed_spans = _merge_spans(spans_by_path.get(path, []))
        for c in comments:
            if any(c.start >= span["start"] and c.end <= span["end"] for span in removed_spans):
                continue
            line = text[:c.start].count("\n") + 1
            end_line = text[:c.end].count("\n") + 1
            snippets.append({
                "path": path, "line": line, "end_line": end_line,
                "file_sha256": sha(data), "comment_sha256": sha(c.text.encode()),
                "kind": "source comment",
                "text": c.text,
            })
    manual_readme = "crates/tablepro-ui/manual-api-contracts/workbench-tabs-privacy/README.md"
    if manual_readme in paths:
        data = source_bytes(manual_readme)
        text = data.decode()
        snippets.append({
            "path": manual_readme, "line": 1, "end_line": text.count("\n") + 1,
            "file_sha256": sha(data), "comment_sha256": None,
            "kind": "manual API compile-boundary contract prose; fixture removed with its old target",
            "text": text,
        })
    return snippets


def _write_prose(entries: list[dict]) -> None:
    md = [
        "# Source prose archive",
        "",
        f"Input SHA: {BASE}",
        "",
        "This file preserves original Rust source comments and the removed manual API fixture README. It is an evidence archive for the shared docs owner. It does not claim that every excerpt is a maintained contract.",
        "",
    ]
    tick = chr(96)
    by_path = collections.defaultdict(list)
    for entry in entries:
        by_path[entry["path"]].append(entry)
    for path in sorted(by_path):
        file_entries = by_path[path]
        md.extend([
            "## " + tick + path + tick,
            "",
            "- Original source SHA-256: " + tick + file_entries[0]["file_sha256"] + tick,
            "- Excerpts are verbatim and retain their input line numbers.",
            "",
        ])
        for entry in file_entries:
            kind = "" if entry["kind"] == "source comment" else ": " + entry["kind"]
            md.extend([f"### Lines {entry['line']}–{entry['end_line']}{kind}", ""])
            body = entry["text"]
            longest = max((len(m.group(0)) for m in re.finditer(re.escape(tick) + r"+", body)), default=0)
            fence = tick * max(4, longest + 1)
            md.extend([fence, body.rstrip("\n"), fence, ""])
    (RESULT / "source-prose.md").write_text("\n".join(md).rstrip() + "\n")


def _test_function_inventory() -> dict[str, list[dict]]:
    result: dict[str, list[dict]] = {}
    for path in git_paths():
        if not is_scoped(path) or not path.endswith(".rs"):
            continue
        data = source_bytes(path)
        text = data.decode()
        tokens, _ = lex(data)
        for i in range(len(tokens) - 3):
            if [t.text for t in tokens[i:i + 4]] != ["#", "[", "test", "]"]:
                continue
            j = i + 4
            while j < len(tokens):
                end = _attr_end(tokens, j)
                if end is None:
                    break
                j = end
            while j < len(tokens) and tokens[j].text in ("async", "unsafe", "const", "extern"):
                j += 1
                if j < len(tokens) and tokens[j].kind == "literal":
                    j += 1
            if j + 1 < len(tokens) and tokens[j].text == "fn":
                result.setdefault(path, []).append({
                    "function": tokens[j + 1].text,
                    "line": text[:tokens[i].start].count("\n") + 1,
                    "attribute": "#[test]",
                    "status": "lexed test attribute",
                })
    return result


def _test_inventory_reconciliation(actual: dict[str, list[dict]]) -> dict:
    base_inventory = json.loads(INVENTORY.read_text())
    scope_rows = [
        item for item in base_inventory["rust_test_functions"]
        if is_scoped(item.get("path", ""))
    ]
    expected = collections.Counter((item["path"], item["function"]) for item in scope_rows)
    overlay = json.loads((RESULT.parent / "t00-source/declaration-overlay-1d6.json").read_text())
    delta = json.loads((RESULT.parent / "t00-source/test-inventory-delta-1d6.json").read_text())
    added = []
    removed = []
    for item in overlay["changed_declarations"]:
        path = item.get("path", "")
        if not is_scoped(path):
            continue
        added.extend((path, row["function"]) for row in item.get("added_declarations", []))
        removed.extend((path, row["function"]) for row in item.get("removed_declarations", []))
    for item in delta["changed_file_evidence"]:
        path = item.get("path", "")
        if not is_scoped(path):
            continue
        added.extend((path, function) for function in item.get("added_test_functions", []))
        removed.extend((path, function) for function in item.get("removed_test_functions", []))
    for key in set(added):
        expected[key] += 1
    for key in set(removed):
        expected[key] -= 1
        if expected[key] <= 0:
            del expected[key]
    observed = collections.Counter(
        (path, item["function"])
        for path, items in actual.items()
        for item in items
    )
    current_sha_matches = []
    for item in delta["changed_file_evidence"]:
        path = item["path"]
        if is_scoped(path):
            current_sha_matches.append({
                "path": path,
                "input_sha256": sha(source_bytes(path)),
                "overlay_new_sha256": item["new_sha256"],
                "equal": sha(source_bytes(path)) == item["new_sha256"],
            })
    if not all(row["equal"] for row in current_sha_matches):
        raise AssertionError("T00 1d6 overlay does not match the fixed c817 input")
    return {
        "base_inventory_source_sha": base_inventory.get("source_commit"),
        "overlay_source_sha": overlay.get("source_commit"),
        "delta_source_sha": delta.get("input_commit"),
        "base_inventory_candidate_count": len(scope_rows),
        "overlay_added_test_count": len(set(added)),
        "overlay_removed_test_count": len(set(removed)),
        "effective_inventory_candidate_count": sum(expected.values()),
        "lexed_test_attribute_count": sum(observed.values()),
        "T00_candidates_not_lexed_as_test_items": [
            {"path": path, "function": function, "count": count}
            for (path, function), count in sorted((expected - observed).items())
        ],
        "lexed_test_items_missing_from_effective_inventory": [
            {"path": path, "function": function, "count": count}
            for (path, function), count in sorted((observed - expected).items())
        ],
        "overlay_file_sha256_checks": current_sha_matches,
    }


def apply() -> None:
    removed_files, spans_by_path = plan()
    all_paths = [p for p in git_paths() if is_scoped(p)]
    directive_comments = []
    comments_by_path = {}
    for path in all_paths:
        if not path.endswith(".rs") or path in removed_files:
            continue
        data = source_bytes(path)
        transformed = _erase_spans(data, spans_by_path.get(path, []))
        transformed, comments = _strip_comments(transformed, directive_comments, path)
        target = ROOT / path
        target.write_bytes(transformed)
        comments_by_path[path] = comments
    for path in removed_files:
        target = ROOT / path
        if target.exists():
            target.unlink()
    for package in PACKAGES:
        base = ROOT / "crates" / package
        for directory in sorted((p for p in base.rglob("*") if p.is_dir()), reverse=True):
            try:
                directory.rmdir()
            except OSError:
                pass
    snippets = _capture_prose(all_paths, spans_by_path)
    _write_prose(snippets)
    (RESULT / "preserved-directives.json").write_text(json.dumps({
        "input_sha": BASE, "comments": directive_comments,
    }, ensure_ascii=False, indent=2) + "\n")


def verify() -> None:
    removed_files, spans_by_path = plan()
    all_paths = [p for p in git_paths() if is_scoped(p)]
    test_functions = _test_function_inventory()
    missing = []
    unexpected = []
    rows = []
    for path in all_paths:
        target = ROOT / path
        original = source_bytes(path)
        if path in removed_files:
            if target.exists():
                unexpected.append(path)
            original_comments = lex(original)[1]
            original_tokens = _tokens(original)
            original_literals = _literals(original)
            rows.append({
                "path": path, "input_sha256": sha(original), "output": "deleted",
                "reason": "legacy test target, test-only module/helper source, or manual compile fixture",
                "input_non_comment_token_sha256": _hash_json(original_tokens),
                "input_non_comment_token_count": len(original_tokens),
                "input_literal_sha256": _hash_json(original_literals),
                "input_literal_count": len(original_literals),
                "comment_token_count": len(original_comments),
                "doc_comment_token_count": sum(
                    c.text.startswith(("///", "//!", "/**", "/*!")) for c in original_comments
                ),
                "test_function_count": len(test_functions.get(path, [])),
            })
            continue
        if not path.endswith(".rs"):
            continue
        if not target.exists():
            missing.append(path)
            continue
        projected = _erase_spans(original, spans_by_path.get(path, []))
        directives = []
        projected, comments_removed = _strip_comments(projected, directives, path)
        expected_tokens = _tokens(projected)
        actual = target.read_bytes()
        actual_tokens = _tokens(actual)
        expected_hash = _hash_json(expected_tokens)
        actual_hash = _hash_json(actual_tokens)
        expected_literals = _literals(projected)
        actual_literals = _literals(actual)
        if expected_tokens != actual_tokens:
            diff = list(difflib.unified_diff(
                expected_tokens, actual_tokens, fromfile=path + " expected", tofile=path + " actual",
                lineterm="", n=4,
            ))
            raise AssertionError("token mismatch in " + path + "\n" + "\n".join(diff[:100]))
        removed_items = []
        original_text = original.decode("utf-8")
        for item in spans_by_path.get(path, []):
            item_source = original_text[item["start"]:item["end"]].encode("utf-8")
            item_tokens = _tokens(item_source)
            item_literals = _literals(item_source)
            removed_items.append({
                **{k: v for k, v in item.items() if k not in ("start", "end")},
                "non_comment_token_sha256": _hash_json(item_tokens),
                "non_comment_token_count": len(item_tokens),
                "literal_sha256": _hash_json(item_literals),
                "literal_count": len(item_literals),
            })
        rows.append({
            "path": path, "input_sha256": sha(original), "output_sha256": sha(actual),
            "projected_token_sha256": expected_hash, "output_token_sha256": actual_hash,
            "projected_literal_sha256": _hash_json(expected_literals),
            "output_literal_sha256": _hash_json(actual_literals),
            "token_equal": True, "literal_equal": expected_literals == actual_literals,
            "removed_items": removed_items,
            "comment_token_count": len(comments_removed),
            "doc_comment_token_count": sum(item["doc"] for item in comments_removed),
            "retained_comment_directive_count": len(directives),
            "test_function_count": len(test_functions.get(path, [])),
        })
        if expected_literals != actual_literals:
            raise AssertionError("literal mismatch in " + path)
        if directives:
            raise AssertionError("unaccounted retained directive comments in " + path)
    if missing or unexpected:
        raise AssertionError(f"file set mismatch; missing={missing}, unexpected={unexpected}")
    test_functions_removed = sum(row.get("test_function_count", 0) for row in rows)
    if test_functions_removed != sum(map(len, test_functions.values())):
        raise AssertionError("T00 test-function inventory is not fully represented in deletion map")
    parsed_cfg = []
    for path, spans in spans_by_path.items():
        for item in spans:
            if item.get("reason") == "cfg(test) item":
                parsed_cfg.append((path, item.get("attribute_line")))
    inventory = json.loads(INVENTORY.read_text())
    expected_cfg = [
        (item["path"], item["line"]) for item in inventory["test_testing_cfg_scopes"]
        if is_scoped(item.get("path", "")) and item.get("path") not in removed_files
    ]
    if sorted(parsed_cfg) != sorted(expected_cfg):
        raise AssertionError("cfg(test) attribute map differs from T00 inventory")
    for path, items in test_functions.items():
        if path in removed_files:
            continue
        source_spans = [
            s for s in spans_by_path.get(path, [])
            if s.get("reason") == "cfg(test) item"
        ]
        for test in items:
            if not any(s["line"] <= test["line"] <= s["end_line"] for s in source_spans):
                raise AssertionError(f"test function is not covered by a deleted scope: {path}:{test['line']} {test['function']}")
    remaining_cfg_test = []
    remaining_test_attrs = []
    for path in all_paths:
        if path in removed_files or not path.endswith(".rs"):
            continue
        tokens, _ = lex((ROOT / path).read_bytes())
        texts = [t.text for t in tokens]
        for i in range(len(texts) - 6):
            if texts[i:i + 7] == ["#", "[", "cfg", "(", "test", ")", "]"]:
                remaining_cfg_test.append({"path": path, "token_index": i})
        for i in range(len(texts) - 3):
            if texts[i:i + 4] == ["#", "[", "test", "]"]:
                remaining_test_attrs.append({"path": path, "token_index": i})
    if remaining_cfg_test:
        raise AssertionError("cfg(test) remains in production source: " + repr(remaining_cfg_test[:20]))
    if remaining_test_attrs:
        raise AssertionError("test attributes remain in production source: " + repr(remaining_test_attrs[:20]))
    output_evidence = _help_and_diagnostic_evidence()
    if not all(x["literal_stream_equal"] for x in output_evidence["cli_help"]):
        raise AssertionError("CLI help string changed")
    if not all(x["production_token_stream_equal"] for x in output_evidence["diagnostic_sites"]):
        raise AssertionError("production diagnostic source tokens changed")
    test_reconciliation = _test_inventory_reconciliation(test_functions)
    directive_evidence = _attribute_evidence(all_paths, removed_files, spans_by_path)
    if directive_evidence["summary"]["test_or_testing_feature_gates_found"]:
        raise AssertionError("a test-only feature gate remains or needs separate review")
    (RESULT / "directive-inventory.json").write_text(json.dumps(directive_evidence, ensure_ascii=False, indent=2) + "\n")
    cfg_reconciliation = _cfg_inventory_reconciliation()
    if cfg_reconciliation["lexed_outer_cfg_items_missing_from_T00"]:
        raise AssertionError("lexed cfg(test) item is missing from the base T00 ledger")
    exact_cfg_count = sum(
        1 for item in directive_evidence["removed_attributes"] + directive_evidence["retained_attributes"]
        if item["attribute"].replace(" ", "") in ("#[cfg(test)]", "#![cfg(test)]")
    )
    cfg_reconciliation["lexed_total_cfg_test_declarations"] = exact_cfg_count
    cfg_reconciliation["false_positive_cfg_candidates"] = len(
        cfg_reconciliation["T00_candidates_not_lexed_as_outer_cfg_items"]
    )
    cfg_reconciliation["cfg_test_declarations_remaining"] = len(remaining_cfg_test)
    cfg_reconciliation["test_feature_gates_found"] = directive_evidence["summary"]["test_or_testing_feature_gates_found"]
    (RESULT / "cfg-test-reconciliation.json").write_text(json.dumps(cfg_reconciliation, ensure_ascii=False, indent=2) + "\n")
    out = {
        "input_sha": BASE,
        "scope": list(PACKAGES),
        "token_method": "Rust-aware lexical scan; line and nested block comments and raw/quoted literals are distinct; literals are compared in order and byte spelling.",
        "files": rows,
        "non_comment_token_changes": {
            "deleted_file_token_sequences": [
                {
                    "path": row["path"],
                    "input_sha256": row["input_sha256"],
                    "token_sha256": row["input_non_comment_token_sha256"],
                    "token_count": row["input_non_comment_token_count"],
                    "literal_sha256": row["input_literal_sha256"],
                    "literal_count": row["input_literal_count"],
                }
                for row in rows if row.get("output") == "deleted"
            ],
            "removed_item_token_sequences": [
                {
                    "path": row["path"],
                    "kind": item["kind"],
                    "name": item["name"],
                    "line": item["line"],
                    "end_line": item["end_line"],
                    "reason": item["reason"],
                    "token_sha256": item["non_comment_token_sha256"],
                    "token_count": item["non_comment_token_count"],
                    "literal_sha256": item["literal_sha256"],
                    "literal_count": item["literal_count"],
                }
                for row in rows for item in row.get("removed_items", [])
            ],
            "unlisted_non_comment_token_changes": [],
            "surviving_production_token_streams_equal_to_input_after_listed_removals": all(
                row.get("token_equal", True) for row in rows
            ),
            "surviving_production_literal_streams_equal_to_input_after_listed_removals": all(
                row.get("literal_equal", True) for row in rows
            ),
        },
        "preserved_output_evidence": output_evidence,
        "test_inventory_reconciliation": test_reconciliation,
        "directive_inventory_summary": directive_evidence["summary"],
        "remaining_cfg_test_attributes": remaining_cfg_test,
        "remaining_test_attributes": remaining_test_attrs,
        "summary": {
            "removed_files": len(removed_files),
            "test_functions_removed": test_functions_removed,
            "cfg_test_candidates_in_base_ledger": cfg_reconciliation["T00_cfg_scope_candidate_count"],
            "cfg_test_items_removed": exact_cfg_count,
            "cfg_test_candidate_false_positives": cfg_reconciliation["false_positive_cfg_candidates"],
            "inner_cfg_test_attributes_removed": len(cfg_reconciliation["inner_cfg_test_attributes_not_in_scope_ledger"]),
            "cfg_test_items_removed_from_surviving_files": len(expected_cfg),
            "surviving_rust_files": sum(row.get("token_equal", False) for row in rows),
            "surviving_token_streams_equal": all(row.get("token_equal", True) for row in rows),
            "surviving_literal_streams_equal": all(row.get("literal_equal", True) for row in rows),
        },
    }
    (RESULT / "deletion-map.json").write_text(json.dumps(out, ensure_ascii=False, indent=2) + "\n")
    (RESULT / "test-inventory-reconciliation.json").write_text(json.dumps({
        "input_sha": BASE, **test_reconciliation,
        "classification": {
            "T00_candidates_not_lexed_as_test_items": [
                "crates/showcase-demos/src/pages/editor.rs::backoff_doubles is an #[test] example inside the production SAMPLE string; literal preserved by token/literal equality."
            ],
            "overlay_additions": [
                "crates/showcase-ui/tests/navigation_contract.rs::narrow_buttons_route_keeps_tree_markers_thumb_and_body is recorded as a real added test by declaration-overlay-1d6.json and test-inventory-delta-1d6.json."
            ],
        },
    }, ensure_ascii=False, indent=2) + "\n")
    _write_prose(_capture_prose(all_paths, spans_by_path))


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("mode", choices=("plan", "apply", "verify"))
    args = parser.parse_args()
    if args.mode == "plan":
        removed, spans = plan()
        print(json.dumps({
            "input_sha": BASE, "removed_files": removed,
            "source_spans": {p: [{k: v for k, v in item.items() if k not in ("start", "end")}
                                 for item in items] for p, items in spans.items()},
        }, indent=2))
    elif args.mode == "apply":
        apply()
    else:
        verify()


if __name__ == "__main__":
    main()
