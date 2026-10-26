import hashlib
import json
import os
import re
import subprocess
import tomllib
from pathlib import Path


ROOT = Path.cwd()
OUT = ROOT / "docs/implementation/pr38-final/results/t01-foundation-api/foundation-api.fragment.json"
INPUT = "542349e2f70ec9883d743594f906ac6c4e96ba5a"
PRODUCT = "1d6ee92adf6ccca38eeee64b1371cba1497e8fcd"
META_HEAD = "516198caac5e5b5fa0883fa1f209928c2e86c8ba"
T37_CANDIDATE = "a94b93128ab71d2f1be43733a1b68d21792820ff"
T37_INTEGRATION = "43b5a322e0da46ab7282b5b820831eb70c91f4ed"
T37_TREE = "e8aa68881a4ad4d4b4e3d9b34045153240e33ce3"
METADATA_TREE = "a10cd30dfafec83f7668f529bb03e27f1e5b8b4d"
FOUNDATION = {
    "termrock-core": ("crates/termrock-core/Cargo.toml", "T09", "Identity; framework keys, events, intents, responses; validation errors; diagnostics; neutral policy values."),
    "termrock-layout": ("crates/termrock-layout/Cargo.toml", "T10", "Constraints, size, tracks, columns, split geometry, and composition plans."),
    "termrock-theme": ("crates/termrock-theme/Cargo.toml", "T11", "Tokens, roles, variants, part metrics, capability conversion, and paint styles."),
    "termrock-text": ("crates/termrock-text/Cargo.toml", "T13", "Grapheme editing, undo, width, wrapping, truncation, borrowed spans, and secret display policy."),
    "termrock-render": ("crates/termrock-render/Cargo.toml", "T11", "Pure patch merging and borrowed terminal source value types."),
    "termrock-collections": ("crates/termrock-collections/Cargo.toml", "T14", "Key sets, cursor and selection reconciliation, and scroll state."),
    "termrock-runtime": ("crates/termrock-runtime/Cargo.toml", "T09", "App/Cx/Ui, input routing, frame state, focus, hit testing, capture, layers, clocks, composition execution, and ScrollRegion."),
    "termrock-session": ("crates/termrock-session/Cargo.toml", "T20", "Terminal entry and exit, transport, backend input conversion, protocol negotiation, and presentation acknowledgement."),
    "termrock-shell": ("crates/termrock-shell/Cargo.toml", "T19", "Page routing, leave guards, shell chrome, named slots, and status/footer orchestration."),
    "termrock-terminal": ("crates/termrock-terminal/Cargo.toml", "T17", "TerminalView, selection, input tokens, and terminal-content adaptation."),
}
SUPPORT = {
    "termrock-consumer-example": ("crates/termrock-consumer-example/Cargo.toml", "T30", "Workspace consumer example for the public facade."),
    "termrock-xtask": ("crates/termrock-xtask/Cargo.toml", "T02", "Workspace task and acceptance tooling."),
    "termrock-visibility-publisher-workspace": ("crates/termrock-visibility-publisher/Cargo.toml", "T32", "Workspace visibility artifact publisher and observer API."),
    "visual-harness": ("crates/visual-harness/Cargo.toml", "T04", "Metadata wrapper targets for the four application binaries."),
    "refactor-proof": ("tools/refactor-proof/Cargo.toml", "T02", "Qualified-harness comparator and host core."),
    "termrock-e2e": ("crates/termrock-e2e/Cargo.toml", "T36", "Retained qualification admission library and binary."),
    "termrock-visibility-tests": ("crates/termrock-visibility-tests/Cargo.toml", "T32", "Retained visibility capture and result-reader tools."),
    "external-consumer-demo": ("external-consumer-demo/Cargo.toml", "T30", "Standalone facade consumer used to represent an external Cargo workspace."),
    "termrock-visibility-publisher-standalone": ("tools/termrock-visibility-publisher/Cargo.toml", "T32", "Standalone metadata workspace for the visibility publisher library."),
}
META_FILES = [
    "docs/implementation/pr38-final/results/t37-final-integration/metadata-root-final.json",
    "docs/implementation/pr38-final/results/t37-final-integration/metadata-e2e-final.json",
    "docs/implementation/pr38-final/results/t37-final-integration/metadata-visibility-final.json",
    "docs/implementation/pr38-final/results/t37-final-integration/metadata-consumer-final.json",
    "docs/implementation/pr38-final/results/t37-final-integration/metadata-publisher-final.json",
]
EVIDENCE_FILES = [
    "docs/implementation/pr38-final/results/t37-metadata/metadata-final.json",
    "docs/implementation/pr38-final/results/t37-metadata/metadata-checks.json",
    "docs/implementation/pr38-final/results/t37-foundations/source-removal.json",
    "docs/implementation/pr38-final/results/t37-foundations/deferred-testing-surfaces.json",
    "docs/implementation/pr38-final/results/t37-foundations/source-api-prose-disposition.json",
    "docs/implementation/pr38-final/results/t37-api-runtime/foundation-runtime-disposition.json",
    "docs/implementation/pr38-final/results/t37-api-support/foundation-support-disposition.json",
    "docs/implementation/pr38-final/results/t37-api-values/decisions.json",
    "docs/implementation/pr38-final/results/t37-final-integration/metadata-structure.json",
    "docs/implementation/pr38-final/results/run-state.json",
    "docs/implementation/pr38-final/ARCHITECTURE.md",
    "docs/implementation/pr38-final/API-RULES.md",
    "docs/implementation/pr38-final/COMPONENT-MATRIX.md",
    "docs/implementation/pr38-final/TASKS.md",
    "docs/implementation/pr38-final/task-index.json",
    "docs/implementation/pr38-final/VALIDATION.md",
    "docs/implementation/pr38-final/SOURCES.md",
]


def sha(data):
    return hashlib.sha256(data).hexdigest()


def run(args):
    return subprocess.run(args, cwd=ROOT, check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE).stdout


def relative_meta_path(path):
    value = path.replace("\\", "/")
    marker = "/termrock-t37-metadata/"
    if marker in value:
        return value.split(marker, 1)[1]
    return value


metadata = {}
metadata_workspace = {}
metadata_documents = {}
for meta_file in META_FILES:
    data = json.loads((ROOT / meta_file).read_text())
    metadata_documents[meta_file] = data
    for package in data.get("packages", []):
        rel = relative_meta_path(package["manifest_path"])
        metadata[(package["name"], rel)] = package
        metadata_workspace[(package["name"], rel)] = meta_file


def normalized_workspace_root(meta_path):
    root = str(metadata_documents[meta_path].get("workspace_root") or "").replace("\\", "/").rstrip("/")
    marker = "/termrock-t37-metadata"
    if root.endswith(marker):
        return "."
    if marker + "/" in root:
        return root.split(marker + "/", 1)[1]
    return root or "unknown"


def workspace_identity(meta_path):
    return "workspace:" + normalized_workspace_root(meta_path)


source_by_path = {}
items = []
exports = []
crates = []
package_manifest_rows = []
current_file_modules = {}
package_source_records = {}


def source_record(path, commit=INPUT, kind="rust"):
    path = str(path).replace("\\", "/")
    if path in source_by_path:
        return source_by_path[path]
    absolute = ROOT / path
    if absolute.exists():
        raw = absolute.read_bytes()
        count = raw.count(b"\n") + (1 if raw and not raw.endswith(b"\n") else 0)
        row = {"path": path, "commit": commit, "sha256": sha(raw), "lines": count, "kind": kind}
    else:
        row = {"path": path, "commit": commit, "sha256": None, "lines": None, "kind": kind, "status": "historical source is retained in T00 Git objects"}
    source_by_path[path] = row
    return row


def source_text(path):
    return (ROOT / path).read_text(encoding="utf-8")


def module_parts(crate_name, package_dir, file_path):
    file_path = Path(file_path)
    if not file_path.is_absolute():
        file_path = ROOT / file_path
    rel = file_path.relative_to(package_dir / "src")
    parts = list(rel.parts)
    if parts[-1] == "lib.rs" or parts[-1] == "main.rs":
        parts = parts[:-1]
    elif parts[-1] == "mod.rs":
        parts = parts[:-1]
    else:
        parts[-1] = Path(parts[-1]).stem
    if parts and parts[0] == "bin":
        parts = ["bin"] + parts[1:]
    return [crate_name] + parts


def line_number(text, position):
    return text.count("\n", 0, position) + 1


def brace_pairs(text):
    pairs = {}
    stack = []
    i = 0
    state = "normal"
    block_depth = 0
    raw_hashes = 0
    while i < len(text):
        ch = text[i]
        nxt = text[i + 1] if i + 1 < len(text) else ""
        if state == "line":
            if ch == "\n":
                state = "normal"
        elif state == "block":
            if ch == "/" and nxt == "*":
                block_depth += 1
                i += 1
            elif ch == "*" and nxt == "/":
                block_depth -= 1
                i += 1
                if block_depth == 0:
                    state = "normal"
        elif state == "string":
            if ch == "\\":
                i += 1
            elif ch == '"':
                state = "normal"
        elif state == "char":
            if ch == "\\":
                i += 1
            elif ch == "'":
                state = "normal"
        elif state == "raw":
            if ch == '"' and text.startswith("#" * raw_hashes, i + 1):
                i += raw_hashes
                state = "normal"
        else:
            if ch == "/" and nxt == "/":
                state = "line"
                i += 1
            elif ch == "/" and nxt == "*":
                state = "block"
                block_depth = 1
                i += 1
            elif ch == "r" and nxt == '"':
                state = "raw"
                raw_hashes = 0
                i += 1
            elif ch == "r" and nxt == "#":
                j = i + 1
                while j < len(text) and text[j] == "#":
                    j += 1
                if j < len(text) and text[j] == '"':
                    raw_hashes = j - (i + 1)
                    state = "raw"
                    i = j
            elif ch == '"':
                state = "string"
            elif ch == "'":
                end = text.find("'", i + 1, min(len(text), i + 10))
                if end != -1 and "\n" not in text[i:end]:
                    state = "char"
            elif ch == "{":
                stack.append(i)
            elif ch == "}" and stack:
                opening = stack.pop()
                pairs[opening] = i + 1
        i += 1
    return pairs


def find_open(text, start):
    pairs = brace_pairs(text)
    next_open = text.find("{", start)
    return next_open if next_open >= 0 and next_open in pairs else None


def end_header(text, start, mode):
    paren = 0
    bracket = 0
    angle = 0
    state = "normal"
    block_depth = 0
    i = start
    while i < len(text):
        ch = text[i]
        nxt = text[i + 1] if i + 1 < len(text) else ""
        if state == "line":
            if ch == "\n":
                state = "normal"
        elif state == "block":
            if ch == "/" and nxt == "*":
                block_depth += 1
                i += 1
            elif ch == "*" and nxt == "/":
                block_depth -= 1
                i += 1
                if block_depth == 0:
                    state = "normal"
        elif state == "string":
            if ch == "\\":
                i += 1
            elif ch == '"':
                state = "normal"
        elif state == "char":
            if ch == "\\":
                i += 1
            elif ch == "'":
                state = "normal"
        else:
            if ch == "/" and nxt == "/":
                state = "line"
                i += 1
            elif ch == "/" and nxt == "*":
                state = "block"
                block_depth = 1
                i += 1
            elif ch == '"':
                state = "string"
            elif ch == "'":
                end = text.find("'", i + 1, min(len(text), i + 10))
                if end != -1 and "\n" not in text[i:end]:
                    state = "char"
            elif ch == "(":
                paren += 1
            elif ch == ")":
                paren = max(0, paren - 1)
            elif ch == "[":
                bracket += 1
            elif ch == "]":
                bracket = max(0, bracket - 1)
            elif ch == "<":
                angle += 1
            elif ch == ">" and angle:
                angle -= 1
            elif paren == 0 and bracket == 0 and angle == 0:
                if mode in ("fn", "method", "trait_method", "container", "macro") and ch == "{":
                    return i + 1
                if ch == ";":
                    return i + 1
        i += 1
    return len(text)


def split_top_level(value):
    parts = []
    start = 0
    paren = bracket = brace = angle = 0
    state = "normal"
    i = 0
    while i < len(value):
        ch = value[i]
        nxt = value[i + 1] if i + 1 < len(value) else ""
        if state == "string":
            if ch == "\\":
                i += 1
            elif ch == '"':
                state = "normal"
        elif state == "char":
            if ch == "\\":
                i += 1
            elif ch == "'":
                state = "normal"
        else:
            if ch == "/" and nxt == "/":
                state = "line"
                i += 1
            elif ch == '"':
                state = "string"
            elif ch == "'":
                end = value.find("'", i + 1, min(len(value), i + 10))
                if end != -1 and "\n" not in value[i:end]:
                    state = "char"
            elif ch == "(":
                paren += 1
            elif ch == ")":
                paren = max(0, paren - 1)
            elif ch == "[":
                bracket += 1
            elif ch == "]":
                bracket = max(0, bracket - 1)
            elif ch == "{":
                brace += 1
            elif ch == "}":
                brace = max(0, brace - 1)
            elif ch == "<":
                angle += 1
            elif ch == ">" and angle:
                angle -= 1
            elif ch == "," and not (paren or bracket or brace or angle):
                parts.append(value[start:i].strip())
                start = i + 1
        if state == "line" and ch == "\n":
            state = "normal"
        i += 1
    if value[start:].strip():
        parts.append(value[start:].strip())
    return parts


def compact(value):
    return re.sub(r"\s+", " ", value).strip()


def attributes_before(text, start):
    prefix = text[:start]
    rows = prefix.splitlines()
    attrs = []
    for row in reversed(rows[-20:]):
        stripped = row.strip()
        if stripped.startswith("#[") or stripped.startswith("#!["):
            attrs.append(stripped)
            continue
        if stripped.startswith("///") or stripped.startswith("//!"):
            continue
        if not stripped:
            if attrs:
                break
            continue
        break
    attrs.reverse()
    return [row for row in attrs if "cfg" in row]


def all_attributes_before(text, start):
    prefix = text[:start]
    rows = prefix.splitlines()
    attrs = []
    for row in reversed(rows[-20:]):
        stripped = row.strip()
        if stripped.startswith("#[") or stripped.startswith("#!["):
            attrs.append(stripped)
            continue
        if stripped.startswith("///") or stripped.startswith("//!"):
            continue
        if not stripped:
            if attrs:
                break
            continue
        break
    attrs.reverse()
    return attrs


def effective_cfg_attrs(module_path, direct_attrs, package_id=None):
    result = list(direct_attrs)
    for index in range(2, len(module_path) + 1):
        key = (package_id, "::".join(module_path[:index])) if package_id else "::".join(module_path[:index])
        for row in module_declarations.get(key, []):
            result.extend(row["cfg"])
    return list(dict.fromkeys(result))


def module_cfg_condition_groups(module_path, package_id=None):
    groups = []
    for index in range(2, len(module_path) + 1):
        path = "::".join(module_path[:index])
        key = (package_id, path) if package_id else path
        rows = module_declarations.get(key, [])
        if rows:
            groups.append({
                "module_path": path,
                "alternatives": [
                    {
                        "visibility": row["visibility"],
                        "cfg_attributes": row["cfg"],
                        "source": {"path": row["source_path"], "line": row["line"], "sha256": row["source_sha256"]},
                    }
                    for row in rows
                ],
            })
    return groups


def cfg_features(attrs):
    return sorted(set(re.findall(r'feature\s*=\s*["\']([^"\']+)["\']', " ".join(attrs))))


def cfg_attrs_in_text(value):
    return re.findall(r"#\[\s*cfg(?:_attr)?\s*\([^\]]*\)\s*\]", value)


def normalize_rust_path(path, crate_name, module_path, package_id=None):
    path = path.strip().replace(" ", "")
    if path.startswith("crate::"):
        return crate_name + path[len("crate"):]
    if path == "crate":
        return crate_name
    if path.startswith("self::"):
        return "::".join(module_path) + path[len("self") :]
    if path == "self":
        return "::".join(module_path)
    if path.startswith("super::"):
        parent = module_path[:-1]
        rest = path
        while rest.startswith("super::"):
            if len(parent) > 1:
                parent = parent[:-1]
            rest = rest[len("super::"):]
        return "::".join(parent + [rest])
    if path.startswith("::"):
        return path[2:]
    root_modules = {
        key[1].split("::", 1)[1].split("::", 1)[0]
        for key in globals().get("module_declarations", {})
        if isinstance(key, tuple) and (package_id is None or key[0] == package_id) and key[1].startswith(crate_name + "::")
    }
    if path.split("::", 1)[0] in root_modules:
        return crate_name + "::" + path
    return path


def parse_use_tree(expr, crate_name, module_path, package_id=None):
    expr = expr.strip()
    if expr.endswith("}"):
        open_at = expr.find("::{")
        if open_at >= 0:
            prefix = expr[:open_at]
            body = expr[open_at + 3:-1]
            base = normalize_rust_path(prefix, crate_name, module_path, package_id)
            result = []
            for child in split_top_level(body):
                if child == "self":
                    result.append((base, base.split("::")[-1], "module"))
                else:
                    result.extend(parse_use_tree(base + "::" + child, crate_name, module_path, package_id))
            return result
    if " as " in expr:
        source, alias = expr.rsplit(" as ", 1)
        source = normalize_rust_path(source, crate_name, module_path, package_id)
        return [(source, alias.strip(), "alias")]
    source = normalize_rust_path(expr, crate_name, module_path, package_id)
    if source.endswith("::*"):
        return [(source, "*", "wildcard")]
    leaf = source.split("::")[-1]
    return [(source, leaf, "reexport")]


def semantic_tags(name, kind, signature, package_name, path):
    tags = []
    if re.search(r"State|Selection|Cursor|Focus|Scroll|Viewport|Frame|Layer|Clock|Session", name):
        tags.append("state")
    if re.search(r"Action|Cmd|Command|Event|Intent|Outcome|Response|Effect|Binding", name):
        tags.append("action")
    if "Builder" in name or (kind in ("function", "method", "trait_method") and re.search(r"->\s*(?:Self|[A-Za-z_][\w:<>]*Builder)", signature)):
        tags.append("builder")
    if package_name in FOUNDATION and any(x in path for x in ("author", "composition", "surface", "canvas", "window")):
        tags.append("authoring_contract")
    if package_name in SUPPORT:
        tags.append("support_api")
    if name in {"AppShell", "TextReport", "ReportContent", "ReportRows", "ScrollRegion"}:
        tags.append("reusable_component_or_component_support")
    return tags


def owner_for_path(path, symbol=""):
    if path.startswith("crates/termrock-core/"):
        if "KeyboardDelivery" in symbol:
            return "T09"
        if "/keys.rs" in path:
            return "T20"
        if any(module in path for module in ("/list.rs", "/selection.rs", "/typeahead.rs")):
            return "T14"
        return "T09"
    if path.startswith("crates/termrock-layout/"):
        return "T10"
    if path.startswith("crates/termrock-theme/") or path.startswith("crates/termrock-render/"):
        return "T11"
    if path.startswith("crates/termrock-text/"):
        return "T13"
    if path.startswith("crates/termrock-collections/"):
        return "T14"
    if path.startswith("crates/termrock-runtime/"):
        if "/author/composition" in path or "/author/canvas" in path or "/author/surface" in path or "/author/window" in path or "/ui/composition" in path or "/support.rs" in path:
            return "T10"
        if "/author.rs" in path:
            return "T11" if "PartStyle" in symbol or "patch_style" in symbol else "T10"
        if "/runtime/clipboard" in path or "/field_control.rs" in path:
            return "T13"
        if "/decor.rs" in path:
            return "T11"
        if "/layer.rs" in path:
            return "T15"
        if "/scroll_region.rs" in path or "ScrollRegion" in symbol:
            return "T17"
        if "/keymap.rs" in path:
            return "T19"
        return "T09"
    if path.startswith("crates/termrock-session/"):
        return "T20" if "/input" not in path else "T21"
    if path.startswith("crates/termrock-shell/"):
        return "T10" if "AppShell" in symbol or "/shell.rs" in path else "T19"
    if path.startswith("crates/termrock-terminal/"):
        return "T20" if "keyboard" in path else "T17"
    if path.startswith("crates/termrock-consumer-example/") or path.startswith("external-consumer-demo/"):
        return "T30"
    if path.startswith("crates/termrock-xtask/") or path.startswith("tools/refactor-proof/"):
        return "T02"
    if path.startswith("crates/termrock-visibility-publisher/") or path.startswith("tools/termrock-visibility-publisher/"):
        return "T32"
    if path.startswith("crates/visual-harness/"):
        return "T04"
    if path.startswith("crates/termrock-e2e/"):
        return "T36"
    if path.startswith("crates/termrock-visibility-tests/"):
        return "T32"
    return "T01"


def current_module(file_path, package_dir, crate_name, text, position):
    parts = module_parts(crate_name, package_dir, file_path)
    inline = []
    for match in re.finditer(r"(?m)^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z_]\w*)[^;\n]*\{", text):
        opening = text.find("{", match.start())
        pairs = brace_pairs(text)
        closing = pairs.get(opening)
        if closing is not None and opening < position < closing:
            inline.append((opening, match.group(1)))
    inline.sort()
    return parts + [name for _, name in inline]


def impl_header_parts(header):
    value = re.sub(r"^impl\b", "", header.strip(), count=1).strip()
    generic_parameters = ""
    if value.startswith("<"):
        depth = 0
        for index, char in enumerate(value):
            if char == "<":
                depth += 1
            elif char == ">":
                depth -= 1
                if depth == 0:
                    generic_parameters = value[:index + 1]
                    value = value[index + 1:].strip()
                    break
    where_clause = ""
    where_match = re.search(r"\bwhere\b", value)
    if where_match:
        where_clause = value[where_match.start():].strip()
        value = value[:where_match.start()].strip()
    separators = []
    angle = paren = bracket = 0
    for match in re.finditer(r"\bfor\b", value):
        prefix = value[:match.start()]
        angle = paren = bracket = 0
        for char in prefix:
            if char == "<": angle += 1
            elif char == ">": angle = max(0, angle - 1)
            elif char == "(": paren += 1
            elif char == ")": paren = max(0, paren - 1)
            elif char == "[": bracket += 1
            elif char == "]": bracket = max(0, bracket - 1)
        next_nonspace = len(value[match.end():]) - len(value[match.end():].lstrip())
        next_index = match.end() + next_nonspace
        if not (angle or paren or bracket) and (next_index >= len(value) or value[next_index] != "<"):
            separators.append(match.start())
    separator = separators[-1] if separators else None
    if separator is None:
        trait_type = None
        target_type = value
    else:
        trait_type = value[:separator].strip()
        target_type = value[separator + len("for"):].strip()
    return {
        "generic_parameters": generic_parameters,
        "trait_type": trait_type,
        "target_type": target_type,
        "where_clause": where_clause,
    }


def container_rows(text, module_path):
    pairs = brace_pairs(text)
    rows = []
    patterns = [
        ("impl", re.compile(r"(?m)^[ \t]*(?:(?:unsafe|default)\s+)?impl\b")),
        ("trait", re.compile(r"(?m)^[ \t]*(?:pub(?:\([^)]*\))?\s+)?(?:(?:unsafe)\s+)?trait\s+[A-Za-z_]\w*")),
    ]
    for kind, pattern in patterns:
        for match in pattern.finditer(text):
            opening = find_open(text, match.end())
            closing = pairs.get(opening)
            if closing is None:
                continue
            header = text[match.start():opening].strip()
            if kind == "impl":
                parts = impl_header_parts(header)
                target_type = parts["target_type"]
                target_base = re.sub(r"<.*", "", target_type).strip()
                name = target_base.split("::")[-1]
            else:
                found = re.search(r"\btrait\s+([A-Za-z_]\w*)", header)
                name = found.group(1) if found else ""
                target_type = None
                parts = {}
            rows.append({"kind": kind, "name": name, "target_type": target_type, "trait_type": parts.get("trait_type"), "generic_parameters": parts.get("generic_parameters", ""), "where_clause": parts.get("where_clause", ""), "header": compact(header), "start": match.start(), "open": opening, "end": closing, "module": module_path})
    return rows


def macro_definition_rows(text):
    pairs = brace_pairs(text)
    rows = []
    pattern = re.compile(r"(?m)^[ \t]*macro_rules!\s*([A-Za-z_]\w*)\s*\{")
    for match in pattern.finditer(text):
        opening = text.find("{", match.start(), match.end())
        closing = pairs.get(opening)
        if closing is not None:
            rows.append({"name": match.group(1), "start": match.start(), "open": opening, "end": closing})
    return rows


def invocation_rows(text, macro_name, after):
    pairs = brace_pairs(text)
    rows = []
    pattern = re.compile(r"(?m)^[ \t]*" + re.escape(macro_name) + r"!\s*\{")
    for match in pattern.finditer(text, after):
        opening = text.find("{", match.start(), match.end())
        closing = pairs.get(opening)
        if closing is not None:
            rows.append({"start": match.start(), "open": opening, "end": closing, "body": text[opening + 1:closing - 1]})
    return rows


def macro_attr_rows(text, start):
    rows = []
    for line in reversed(text[:start].splitlines()[-20:]):
        line = line.strip()
        if line.startswith("#[") or line.startswith("#!["):
            rows.append(line)
        elif line.startswith("///") or line.startswith("//!") or not line:
            continue
        else:
            break
    rows.reverse()
    return rows


def add_macro_api_items(package_name, crate_name, package_id, file_path, source_hash, text, target_name, target_kind):
    definitions = macro_definition_rows(text)
    for definition in definitions:
        macro_name = definition["name"]
        invocations = invocation_rows(text, macro_name, definition["end"])
        for invocation in invocations:
            module_path = current_file_modules[file_path]
            invocation_module = []
            for match in re.finditer(r"(?m)^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z_]\w*)[^;\n]*\{", text):
                opening = text.find("{", match.start())
                closing = brace_pairs(text).get(opening)
                if closing is not None and opening < invocation["start"] < closing:
                    invocation_module.append((opening, match.group(1)))
            invocation_module.sort()
            module_path = module_path + [name for _, name in invocation_module]

            constants = []
            for match in re.finditer(r"(?m)^[ \t]*([A-Z][A-Z0-9_]*)\s*=\s*([^,\n]+)", invocation["body"]):
                constants.append({
                    "name": match.group(1),
                    "value": compact(match.group(2)),
                    "line": line_number(text, invocation["open"] + 1 + match.start()),
                })

            template = text[definition["open"] + 1:definition["end"] - 1]
            type_name = None
            if macro_name == "newtype_u16":
                type_name = re.match(r"\s*([A-Za-z_]\w*)\s*,", invocation["body"])
                type_name = type_name.group(1) if type_name else None
            elif macro_name == "action_keys":
                type_name = "ActionKey"
            elif macro_name == "parts":
                type_name = "Part"
            if macro_name not in {"newtype_u16", "action_keys", "parts"} or not type_name:
                continue

            owner = "::".join(module_path + [type_name])
            attrs = effective_cfg_attrs(module_path, attributes_before(text, invocation["start"]), package_id)
            external = target_kind == "lib" and module_is_public(module_path, module_declarations, package_id)
            template_line = line_number(text, definition["start"])
            invocation_source = {"path": file_path, "line": line_number(text, invocation["start"]), "sha256": source_hash}
            template_source = {"path": file_path, "line": template_line, "sha256": source_hash}

            def add_generated(symbol, kind, signature, line, semantic_name=None, direct=True, template_line=None):
                tags = semantic_tags(semantic_name or symbol.split("::")[-1], kind, signature, package_name, file_path)
                items.append({
                    "package": package_name,
                    "module_path": "::".join(module_path),
                    "current_qualified_symbol": symbol,
                    "kind": kind,
                    "definition_owner": owner if symbol != owner else owner,
                    "reexports": [],
                    "cfg_attributes": attrs,
                    "cfg_features": cfg_features(attrs),
                    "signature": compact(signature),
                    "source_path": file_path,
                    "line": line,
                    "source_sha256": source_hash,
                    "signature_sha256": sha(compact(signature).encode()),
                    "semantic_kinds": tags,
                    "states": [semantic_name or symbol.split("::")[-1]] if "state" in tags else [],
                    "actions": [semantic_name or symbol.split("::")[-1]] if "action" in tags else [],
                    "builders": [semantic_name or symbol.split("::")[-1]] if "builder" in tags else [],
                    "defaults": [],
                    "is_external_api": external,
                    "is_direct_module_export": external and direct,
                    "visibility": "pub",
                    "target": {"name": target_name, "kind": target_kind},
                    "macro_generated": True,
                    "macro_name": macro_name,
                    "macro_invocation_source": invocation_source,
                    "macro_template_source": {"path": file_path, "line": template_line or template_source["line"], "sha256": source_hash},
                })

            if macro_name == "newtype_u16":
                struct_match = re.search(r"pub\s+struct\s+\$name\s*\(\s*u16\s*\)\s*;", template)
                struct_line = line_number(text, definition["open"] + 1 + struct_match.start()) if struct_match else invocation_source["line"]
                add_generated(owner, "struct", "pub struct " + type_name + "(u16);", invocation_source["line"], template_line=struct_line)

            for match in re.finditer(r"(?m)^[ \t]*pub\s+(?:(?:const)\s+)?fn\s+([A-Za-z_]\w*)\s*\(", template):
                name = match.group(1)
                start = definition["open"] + 1 + match.start()
                raw = text[start:end_header(text, start, "fn")].strip().rstrip("{").strip()
                signature = raw.replace("$name", type_name)
                add_generated(owner + "::" + name, "method", signature, line_number(text, start), name, template_line=line_number(text, start))

            const_matches = re.finditer(r"(?m)^.*?\bpub\s+const\s+([A-Za-z_$][A-Za-z0-9_$]*)\s*:\s*([^=;]+?)(?:\s*=\s*([^;]+))?;", template)
            for match in const_matches:
                const_token, type_expr, value_expr = match.groups()
                start = definition["open"] + 1 + match.start()
                template_const_line = line_number(text, start)
                if const_token in ("$name", "$c"):
                    for constant in constants:
                        name = constant["name"]
                        value = constant["value"]
                        signature = "pub const " + name + ": " + type_expr.replace("$name", type_name).replace("$c", name).strip()
                        if value_expr:
                            expression = value_expr.replace("$name", type_name).replace("$c", name).replace("$v", value)
                            signature += " = " + compact(expression)
                        signature += ";"
                        add_generated(owner + "::" + name, "associated_const", signature, constant["line"], name, template_line=template_const_line)
                else:
                    name = const_token
                    signature = "pub const " + name + ": " + type_expr.replace("$name", type_name).strip()
                    if value_expr:
                        if name == "ALL" and constants:
                            element = type_name
                            signature += " = &[" + ", ".join(element + "::" + row["name"] for row in constants) + "]"
                        else:
                            signature += " = " + compact(value_expr.replace("$name", type_name))
                    signature += ";"
                    add_generated(owner + "::" + name, "associated_const", signature, invocation_source["line"], name, template_line=template_const_line)

        macro_attrs = macro_attr_rows(text, definition["start"])
        if "macro_export" in " ".join(macro_attrs):
            name = macro_name
            symbol = crate_name + "::" + name
            start = definition["start"]
            signature = text[start:definition["open"] + 1].strip()
            module_path = current_file_modules[file_path]
            item = {
                "package": package_name,
                "module_path": "::".join(module_path),
                "current_qualified_symbol": symbol,
                "kind": "macro",
                "definition_owner": symbol,
                "reexports": [symbol],
                "cfg_attributes": [row for row in macro_attrs if "cfg" in row],
                "cfg_features": cfg_features(macro_attrs),
                "signature": compact(signature),
                "source_path": file_path,
                "line": line_number(text, start),
                "source_sha256": source_hash,
                "signature_sha256": sha(signature.encode()),
                "semantic_kinds": ["exported_macro"],
                "states": [], "actions": [], "builders": [], "defaults": [],
                "is_external_api": target_kind == "lib",
                "is_direct_module_export": False,
                "visibility": "pub",
                "target": {"name": target_name, "kind": target_kind},
                "macro_export": True,
            }
            items.append(item)
            exports.append({
                "package": package_name,
                "current_qualified_symbol": symbol,
                "kind": "exported_macro",
                "definition_owner": symbol,
                "reexports": [symbol],
                "cfg_attributes": item["cfg_attributes"],
                "cfg_features": item["cfg_features"],
                "signature": compact(signature),
                "source_path": file_path,
                "line": item["line"],
                "source_sha256": source_hash,
                "signature_sha256": item["signature_sha256"],
                "is_external_api": target_kind == "lib",
                "target": {"name": target_name, "kind": target_kind},
            })


def declaration_name(match, kind):
    if kind == "use":
        return "use"
    if kind == "macro_rules!":
        return match.group(1)
    return match.group(1)


def visibility_module_map(file_path, package_dir, crate_name, package_id, text, source_sha256):
    rows = []
    for match in re.finditer(r"(?m)^[ \t]*(pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z_]\w*)\s*(?:;|\{)", text):
        visibility = "pub" if match.group(1) and match.group(1).strip() == "pub" else "restricted" if match.group(1) else "private"
        attrs = attributes_before(text, match.start())
        parent_module = current_module(file_path, package_dir, crate_name, text, match.start())
        rows.append({"key": (package_id, "::".join(parent_module + [match.group(2)])), "path": "::".join(parent_module + [match.group(2)]), "visibility": visibility, "cfg": attrs, "line": line_number(text, match.start()), "source_path": str(file_path.relative_to(ROOT)).replace("\\", "/"), "source_sha256": source_sha256})
    return rows


def field_rows(text, item, package_name, crate_name, file_path, source_hash, module_path):
    rows = []
    if item["kind"] not in ("struct", "enum", "union"):
        return rows
    base_name = item["name"]
    if item["kind"] == "struct":
        open_paren = text.find("(", item["start"], item["end"])
        close_paren = text.rfind(")", open_paren, item["end"]) if open_paren >= 0 else -1
        open_brace = text.find("{", item["start"], item["end"])
        if open_paren >= 0 and close_paren > open_paren and (open_brace < 0 or open_paren < open_brace):
            body = text[open_paren + 1:close_paren]
            for index, field in enumerate(split_top_level(body)):
                declaration = re.sub(r"(?m)^\s*#\[[^\]]*\]\s*", "", field).strip()
                match = re.match(r"pub\s+(.+)", declaration, re.S)
                if not match:
                    continue
                signature = compact(match.group(1))
                start = open_paren + 1 + body.find(field)
                start += len(field) - len(field.lstrip())
                field_cfg = list(dict.fromkeys(item["cfg_attributes"] + cfg_attrs_in_text(field)))
                rows.append({
                    "package": package_name,
                    "current_qualified_symbol": "::".join(module_path + [base_name, str(index)]),
                    "kind": "tuple_field",
                    "definition_owner": "::".join(module_path + [base_name]),
                    "reexports": [],
                    "cfg_attributes": field_cfg,
                    "cfg_features": cfg_features(field_cfg),
                    "signature": signature,
                    "source_path": file_path,
                    "line": line_number(text, start),
                    "source_sha256": source_hash,
                    "signature_sha256": sha(signature.encode()),
                    "semantic_kinds": item["semantic_kinds"],
                    "states": [], "actions": [], "builders": [], "defaults": [],
                    "is_external_api": item["is_external_api"],
                    "target": item["target"],
                })
            return rows
    pairs = brace_pairs(text)
    open_at = text.find("{", item["start"], item["end"] + 1)
    if open_at < 0 or open_at not in pairs:
        return rows
    close_at = pairs[open_at] - 1
    body = text[open_at + 1:close_at]
    if item["kind"] == "enum":
        for variant_source in split_top_level(body):
            variant = re.sub(r"(?m)^\s*#\[[^\]]*\]\s*", "", variant_source).strip()
            match = re.match(r"([A-Za-z_]\w*)", variant)
            if not match:
                continue
            name = match.group(1)
            start = open_at + 1 + body.find(variant_source)
            start += len(variant_source) - len(variant_source.lstrip())
            defaults = ["#[default]"] if "#[default]" in variant_source else []
            variant_cfg = list(dict.fromkeys(item["cfg_attributes"] + cfg_attrs_in_text(variant_source)))
            rows.append({
                "package": package_name,
                "current_qualified_symbol": "::".join(module_path + [base_name, name]),
                "kind": "enum_variant",
                "definition_owner": "::".join(module_path + [base_name]),
                "reexports": [],
                "cfg_attributes": variant_cfg,
                "cfg_features": cfg_features(variant_cfg),
                "signature": compact(variant_source),
                "source_path": file_path,
                "line": line_number(text, start),
                "source_sha256": source_hash,
                "signature_sha256": sha(variant_source.encode()),
                "semantic_kinds": item["semantic_kinds"],
                "states": [], "actions": [], "builders": [], "defaults": defaults,
                "is_external_api": item["is_external_api"],
                "target": item["target"],
            })
        return rows
    if body.lstrip().startswith("pub") and "(" in body and body.lstrip().startswith("pub("):
        for index, field in enumerate(split_top_level(body)):
            declaration = re.sub(r"(?m)^\s*#\[[^\]]*\]\s*", "", field).strip()
            match = re.match(r"pub\s+(.+)", declaration, re.S)
            if not match:
                continue
            signature = compact(match.group(1))
            start = open_at + 1 + body.find(field)
            start += len(field) - len(field.lstrip())
            field_cfg = list(dict.fromkeys(item["cfg_attributes"] + cfg_attrs_in_text(field)))
            rows.append({
                "package": package_name,
                "current_qualified_symbol": "::".join(module_path + [base_name, str(index)]),
                "kind": "tuple_field",
                "definition_owner": "::".join(module_path + [base_name]),
                "reexports": [],
                "cfg_attributes": field_cfg,
                "cfg_features": cfg_features(field_cfg),
                "signature": signature,
                "source_path": file_path,
                "line": line_number(text, start),
                "source_sha256": source_hash,
                "signature_sha256": sha(signature.encode()),
                "semantic_kinds": item["semantic_kinds"],
                "states": [], "actions": [], "builders": [], "defaults": [],
                "is_external_api": item["is_external_api"],
                "target": item["target"],
            })
        return rows
    for field in split_top_level(body):
        declaration = re.sub(r"(?m)^\s*#\[[^\]]*\]\s*", "", field).strip()
        match = re.match(r"pub\s+([A-Za-z_]\w*)\s*:", declaration, re.S)
        if not match:
            continue
        name = match.group(1)
        start = open_at + 1 + body.find(field)
        start += len(field) - len(field.lstrip())
        field_cfg = list(dict.fromkeys(item["cfg_attributes"] + cfg_attrs_in_text(field)))
        rows.append({
            "package": package_name,
            "current_qualified_symbol": "::".join(module_path + [base_name, name]),
            "kind": "field",
            "definition_owner": "::".join(module_path + [base_name]),
            "reexports": [],
            "cfg_attributes": field_cfg,
            "cfg_features": cfg_features(field_cfg),
            "signature": compact(field),
            "source_path": file_path,
            "line": line_number(text, start),
            "source_sha256": source_hash,
            "signature_sha256": sha(field.encode()),
            "semantic_kinds": item["semantic_kinds"],
            "states": [], "actions": [], "builders": [], "defaults": [],
            "is_external_api": item["is_external_api"],
            "target": item["target"],
        })
    return rows


def trait_member_rows(text, containers, package_name, package_id, file_path, source_hash, crate_name, target_name, target_kind):
    rows = []
    for container in containers:
        if container["kind"] != "trait":
            continue
        body_start = container["open"] + 1
        body_end = container["end"] - 1
        body = text[body_start:body_end]
        for match in re.finditer(r"(?m)^[ \t]*(?:(?:async|unsafe|const|extern\s+\"[^\"]+\")\s+)*(fn|type|const)\s+([A-Za-z_]\w*)", body):
            start = body_start + match.start()
            kind = {"fn": "trait_method", "type": "associated_type", "const": "associated_const"}[match.group(1)]
            end = end_header(text, start, "trait_method" if kind == "trait_method" else "plain")
            raw_sig = text[start:end].strip()
            module_path = container.get("module", current_file_modules[file_path])
            trait_attrs = attributes_before(text, container["start"])
            attrs = effective_cfg_attrs(module_path, trait_attrs + attributes_before(text, start), package_id)
            qualified = "::".join(module_path + [container["name"], match.group(2)])
            tags = semantic_tags(match.group(2), kind, compact(raw_sig), package_name, file_path)
            rows.append({
                "package": package_name,
                "module_path": "::".join(module_path),
                "current_qualified_symbol": qualified,
                "kind": kind,
                "definition_owner": "::".join(module_path + [container["name"]]),
                "reexports": [],
                "cfg_attributes": attrs,
                "cfg_features": cfg_features(attrs),
                "signature": compact(raw_sig),
                "source_path": file_path,
                "line": line_number(text, start),
                "source_sha256": source_hash,
                "signature_sha256": sha(raw_sig.encode()),
                "semantic_kinds": tags,
                "states": [match.group(2)] if "state" in tags else [],
                "actions": [match.group(2)] if "action" in tags else [],
                "builders": [match.group(2)] if "builder" in tags else [],
                "defaults": [],
                "is_external_api": target_kind == "lib" and module_is_public(module_path, module_declarations, package_id),
                "target": {"name": target_name, "kind": target_kind},
            })
    return rows


def make_definition(package_name, crate_name, package_id, file_path, source_hash, text, start, kind, name, module_path, container, target_name, target_kind, direct_cfg):
    end = end_header(text, start, "fn" if kind in ("fn", "macro_rules!") else "container" if kind in ("struct", "enum", "trait", "union") else "plain")
    raw_sig = text[start:end].strip()
    current_module = module_path
    if container and container["kind"] in ("impl", "trait") and kind not in ("struct", "enum", "trait", "union", "mod", "use"):
        qualified = "::".join(current_module + [container["name"], name])
        definition_owner = "::".join(current_module + [container["name"]])
        output_kind = "method" if kind == "fn" else kind
    else:
        qualified = "::".join(current_module + [name])
        definition_owner = qualified
        output_kind = {"fn": "function", "type": "type_alias", "use": "reexport_statement", "macro_rules!": "macro", "static": "static"}.get(kind, kind)
    cfg = direct_cfg
    tags = semantic_tags(name, output_kind, compact(raw_sig), package_name, file_path)
    attributes = all_attributes_before(text, start)
    derives = ["derive(Default)"] if any("derive(" in attr and "Default" in attr for attr in attributes) else []
    defaults = derives[:]
    if output_kind in ("function", "method") and name == "default":
        defaults.append("Default::default")
    target_api = target_kind == "lib" and module_is_public(current_module, module_declarations, package_id)
    return {
        "package": package_name,
        "module_path": "::".join(current_module),
        "current_qualified_symbol": qualified,
        "kind": output_kind,
        "definition_owner": definition_owner,
        "reexports": [],
        "cfg_attributes": cfg,
        "cfg_features": cfg_features(cfg),
        "signature": compact(raw_sig),
        "source_path": file_path,
        "line": line_number(text, start),
        "source_sha256": source_hash,
        "signature_sha256": sha(raw_sig.encode()),
        "semantic_kinds": tags,
        "states": [name] if "state" in tags else [],
        "actions": [name] if "action" in tags else [],
        "builders": [name] if "builder" in tags else [],
        "defaults": defaults,
        "is_external_api": target_api,
        "target": {"name": target_name, "kind": target_kind},
        "visibility": "pub",
        "impl_target": container.get("target_type") if container and container["kind"] == "impl" else None,
        "impl_module_path": "::".join(module_path) if container and container["kind"] == "impl" else None,
        "impl_line": line_number(text, container["start"]) if container and container["kind"] == "impl" else None,
    }


def split_use_statement(text, start):
    end = end_header(text, start, "plain")
    return text[start:end], end


def module_is_public(module_path, declarations, package_id=None):
    if len(module_path) <= 1:
        return True
    for index in range(2, len(module_path) + 1):
        path = "::".join(module_path[:index])
        prefix = (package_id, path) if package_id else path
        rows = declarations.get(prefix, [])
        if not any(row["visibility"] == "pub" for row in rows):
            return False
    return True


def target_for_file(path, package_dir, metadata_package):
    rel = str(path.relative_to(ROOT)).replace("\\", "/")
    for target in metadata_package.get("targets", []):
        target_rel = os.path.normpath(relative_meta_path(target["src_path"])).replace("\\", "/")
        if target_rel == rel:
            return target["name"], target["kind"][0]
    if rel.startswith(str(package_dir.relative_to(ROOT)).replace("\\", "/") + "/src/"):
        for target in metadata_package.get("targets", []):
            if "lib" in target.get("kind", []):
                return target["name"], "lib"
    return "unmapped", "source"


def add_pkg(name, manifest_path, scope, owner_task, responsibility):
    manifest = ROOT / manifest_path
    raw = manifest.read_bytes()
    parsed = tomllib.loads(raw.decode())
    package = parsed["package"]
    package_name = package["name"]
    meta = metadata.get((package_name, manifest_path))
    if meta is None:
        matches = [value for (pkg, rel), value in metadata.items() if pkg == package_name and rel == manifest_path]
        meta = matches[0] if matches else None
    if meta is None:
        raise RuntimeError("metadata package missing: " + package_name + " at " + manifest_path)
    version = package.get("version")
    if isinstance(version, dict):
        version = None
    if version is None:
        root_manifest = tomllib.loads((ROOT / "Cargo.toml").read_text())
        version = root_manifest.get("workspace", {}).get("package", {}).get("version", "unknown")
    package_dir = manifest.parent
    name_crate = package_name.replace("-", "_")
    manifest_row = source_record(manifest_path, kind="cargo_manifest")
    meta_path = metadata_workspace[(package_name, manifest_path)]
    meta_row = source_record(meta_path, commit=META_HEAD, kind="cargo_metadata_evidence")
    workspace_kind = "root_workspace_member" if meta_path.endswith("metadata-root-final.json") else "standalone_workspace"
    workspace_root = normalized_workspace_root(meta_path)
    workspace_id = workspace_identity(meta_path)
    target_rows = []
    for target in meta.get("targets", []):
        target_path = os.path.normpath(relative_meta_path(target["src_path"])).replace("\\", "/")
        target_rows.append({
            "name": target["name"],
            "kind": target.get("kind", []),
            "source_path": target_path,
            "test": target.get("test"),
            "doctest": target.get("doctest"),
            "bench": target.get("bench"),
            "required_features": target.get("required-features", []),
        })
    crate = {
        "package_name": package_name,
        "package_id": meta["id"],
        "metadata_package_id": meta["id"],
        "normalized_package_identity": package_name + "@" + str(version) + " at " + manifest_path,
        "version": version,
        "manifest_path": manifest_path,
        "manifest_sha256": manifest_row["sha256"],
        "source_scope": scope,
        "workspace_kind": workspace_kind,
        "workspace_id": workspace_id,
        "workspace_root": workspace_root,
        "metadata_evidence": {"path": meta_path, "commit": META_HEAD, "sha256": meta_row["sha256"], "workspace_id": workspace_id, "workspace_root": workspace_root},
        "targets": target_rows,
        "features": meta.get("features", {}),
        "description": package.get("description"),
        "publish": package.get("publish", True),
        "responsibility": responsibility,
        "architecture_owner_task": owner_task,
        "metadata_target_counts": {"count": len(target_rows), "kinds": sorted({k for t in target_rows for k in t["kind"]})},
    }
    crates.append(crate)
    package_manifest_rows.append((package_name, manifest_path, package_dir, name_crate, meta, scope, owner_task, meta["id"], version))


for name, (manifest, owner, responsibility) in FOUNDATION.items():
    add_pkg(name, manifest, "foundation_library", owner, responsibility)
for name, (manifest, owner, responsibility) in SUPPORT.items():
    add_pkg(name, manifest, "non_application_support_consumer_or_tool", owner, responsibility)

module_declarations = {}
pending_use_exports = []
all_impls = []
for package_name, manifest_path, package_dir, crate_name, meta, scope, owner_task, package_id, version in package_manifest_rows:
    rust_files = sorted((package_dir / "src").rglob("*.rs")) if (package_dir / "src").exists() else []
    package_source_records[manifest_path] = []
    for path in rust_files:
        rel = str(path.relative_to(ROOT)).replace("\\", "/")
        file_row = source_record(rel, kind="rust_source")
        package_source_records[manifest_path].append(rel)
        text = path.read_text(encoding="utf-8")
        module_base = module_parts(crate_name, package_dir, path)
        current_file_modules[rel] = module_base
        for row in visibility_module_map(path, package_dir, crate_name, package_id, text, file_row["sha256"]):
            module_declarations.setdefault(row["key"], []).append(row)

for package_name, manifest_path, package_dir, crate_name, meta, scope, owner_task, package_id, version in package_manifest_rows:
    rust_files = sorted((package_dir / "src").rglob("*.rs")) if (package_dir / "src").exists() else []
    for path in rust_files:
        rel = str(path.relative_to(ROOT)).replace("\\", "/")
        text = path.read_text(encoding="utf-8")
        file_hash = source_by_path[rel]["sha256"]
        base_module = current_file_modules[rel]
        target_name, target_kind = target_for_file(path, package_dir, meta)
        containers = container_rows(text, base_module)
        for container in containers:
            container["module"] = current_module(rel, package_dir, crate_name, text, container["start"])
            if container["kind"] == "impl":
                all_impls.append({**container, "package": package_name, "package_id": package_id, "source_path": rel, "source_sha256": file_hash, "target_name": target_name, "target_kind": target_kind, "module_path": container["module"]})
        macro_definitions = macro_definition_rows(text)
        declarations = []
        statement_re = re.compile(r"(?m)^[ \t]*pub(?:\s+|\([^)]*\)\s+)(?:(?:async|unsafe|const|extern\s+\"[^\"]+\")\s+)*(fn|struct|enum|trait|type|const|static|mod|use|union|macro_rules!)(?:\b|(?=\s))")
        for match in statement_re.finditer(text):
            if any(row["start"] < match.start() < row["end"] for row in macro_definitions):
                continue
            visible = re.match(r"\s*pub(?:\(([^)]*)\))?", match.group(0))
            if visible and visible.group(1):
                continue
            kind = match.group(1)
            start = match.start()
            module_path = current_module(rel, package_dir, crate_name, text, start)
            attrs = effective_cfg_attrs(module_path, attributes_before(text, start), package_id)
            container = next((row for row in reversed(containers) if row["start"] < start < row["end"]), None)
            if container:
                attrs = effective_cfg_attrs(module_path, attributes_before(text, container["start"]) + attributes_before(text, start), package_id)
            if kind == "use":
                statement, end = split_use_statement(text, start)
                expression = re.sub(r"^\s*pub\s+use\s+", "", statement, count=1).rstrip(";").strip()
                for source_path, alias, reexport_kind in parse_use_tree(expression, crate_name, module_path, package_id):
                    exported = "::".join(module_path + [alias]) if alias != "*" else "::".join(module_path + ["*"])
                    export = {
                        "package": package_name,
                        "source_package_id": package_id,
                        "source_module": "::".join(module_path),
                        "raw_source_path": source_path,
                        "current_qualified_symbol": exported,
                        "kind": "wildcard_reexport" if reexport_kind == "wildcard" else "reexport_alias" if reexport_kind == "alias" else "reexport",
                        "definition_owner": source_path,
                        "reexports": [exported],
                        "cfg_attributes": attrs,
                        "cfg_features": cfg_features(attrs),
                        "signature": compact(statement),
                        "source_path": rel,
                        "line": line_number(text, start),
                        "source_sha256": file_hash,
                        "signature_sha256": sha(statement.encode()),
                        "import_kind": reexport_kind,
                        "is_external_api": target_kind == "lib" and module_is_public(module_path, module_declarations, package_id),
                        "target": {"name": target_name, "kind": target_kind},
                    }
                    exports.append(export)
                    pending_use_exports.append(export)
                continue
            ident_match = re.match(r"\s*([A-Za-z_]\w*)", text[match.end():])
            if not ident_match:
                continue
            name = ident_match.group(1)
            direct = make_definition(package_name, crate_name, package_id, rel, file_hash, text, start, kind, name, module_path, container, target_name, target_kind, attrs)
            direct["is_direct_module_export"] = module_is_public(module_path, module_declarations, package_id)
            declarations.append(direct)
            items.append(direct)
            if kind in ("struct", "enum", "union"):
                items.extend(field_rows(text, {"kind": kind, "name": name, "start": start, "end": end_header(text, start, "container"), "cfg_attributes": attrs, "cfg_features": cfg_features(attrs), "semantic_kinds": direct["semantic_kinds"], "is_external_api": direct["is_external_api"], "target": direct["target"]}, package_name, crate_name, rel, file_hash, module_path))
            if kind == "mod" and direct["is_external_api"]:
                exports.append({
                    "package": package_name,
                    "source_package_id": package_id,
                    "source_module": "::".join(module_path),
                    "current_qualified_symbol": direct["current_qualified_symbol"],
                    "kind": "public_module",
                    "definition_owner": direct["current_qualified_symbol"],
                    "reexports": [],
                    "cfg_attributes": attrs,
                    "cfg_features": cfg_features(attrs),
                    "signature": direct["signature"],
                    "source_path": rel,
                    "line": direct["line"],
                    "source_sha256": file_hash,
                    "signature_sha256": direct["signature_sha256"],
                    "is_external_api": direct["is_external_api"],
                    "target": {"name": target_name, "kind": target_kind},
                })
            if kind == "macro_rules!" and "macro_export" in text[max(0, start - 200):start]:
                exported_symbol = "::".join([crate_name, name])
                direct["is_external_api"] = True
                direct["reexports"].append(exported_symbol)
                exports.append({
                    "package": package_name,
                    "source_package_id": package_id,
                    "source_module": "::".join(module_path),
                    "current_qualified_symbol": exported_symbol,
                    "kind": "exported_macro",
                    "definition_owner": direct["current_qualified_symbol"],
                    "reexports": [exported_symbol],
                    "cfg_attributes": attrs,
                    "cfg_features": cfg_features(attrs),
                    "signature": direct["signature"],
                    "source_path": rel,
                    "line": direct["line"],
                    "source_sha256": file_hash,
                    "signature_sha256": direct["signature_sha256"],
                    "is_external_api": True,
                    "target": {"name": target_name, "kind": target_kind},
                })

        items.extend(trait_member_rows(text, containers, package_name, package_id, rel, file_hash, crate_name, target_name, target_kind))
        add_macro_api_items(package_name, crate_name, package_id, rel, file_hash, text, target_name, target_kind)

package_info_by_id = {row["package_id"]: row for row in crates}
manifest_info_by_path = {row["manifest_path"]: row for row in crates}
source_package_roots = [
    (str(Path(row["manifest_path"]).parent).replace("\\", "/"), ROOT / Path(row["manifest_path"]).parent, row)
    for row in crates
]


def package_for_source(source_path):
    path = str(source_path).replace("\\", "/")
    matches = [row for row in source_package_roots if path == row[0] or path.startswith(row[0] + "/")]
    return max(matches, key=lambda row: len(row[0]))[2] if matches else None


def attach_package_identity(row, source_path):
    package = package_for_source(source_path)
    if package:
        row["package_id"] = package["package_id"]
        row["manifest_path"] = package["manifest_path"]
        row["normalized_package_identity"] = package["normalized_package_identity"]
        row["workspace_id"] = package["workspace_id"]
        row["source_workspace_id"] = package["workspace_id"]
        row.setdefault("package", package["package_name"])
    return package


for item in items:
    attach_package_identity(item, item["source_path"])
    item.setdefault("definition_package_id", item["package_id"])
for export in exports:
    package = attach_package_identity(export, export["source_path"])
    if package:
        export["source_package_id"] = package["package_id"]
        export.setdefault("definition_package_id", package["package_id"])

package_crate_names = {row["package_id"]: row["package_name"].replace("-", "_") for row in crates}
dependency_package_ids = {}
external_dependency_aliases = {}
metadata_packages_by_manifest = {}
for meta_file in META_FILES:
    metadata_content = json.loads((ROOT / meta_file).read_text())
    for metadata_package in metadata_content.get("packages", []):
        metadata_manifest = relative_meta_path(metadata_package["manifest_path"])
        metadata_packages_by_manifest[metadata_manifest] = metadata_package
cargo_lock_source = source_record("Cargo.lock", kind="cargo_lock_external_owner_evidence")
lock_data = tomllib.loads((ROOT / "Cargo.lock").read_text())
lock_packages_by_name = {}
for locked_package in lock_data.get("package", []):
    lock_packages_by_name.setdefault(locked_package["name"], []).append(locked_package)
for package_name, manifest_path, package_dir, crate_name, meta, scope, owner_task, package_id, version in package_manifest_rows:
    source_identity = manifest_info_by_path[manifest_path]
    for dependency in meta.get("dependencies", []):
        alias = (dependency.get("rename") or dependency["name"]).replace("-", "_")
        dep_path = dependency.get("path")
        dep_identity = None
        if dep_path:
            dep_manifest = relative_meta_path(str(Path(dep_path) / "Cargo.toml"))
            dep_identity = manifest_info_by_path.get(dep_manifest)
        if dep_identity is None:
            candidates = [row for row in crates if row["package_name"] == dependency["name"] and row["workspace_kind"] == source_identity["workspace_kind"]]
            if len(candidates) == 1:
                dep_identity = candidates[0]
        if dep_identity:
            dependency_package_ids[(package_id, alias)] = dep_identity["package_id"]
            continue
        dep_path = dependency.get("path")
        metadata_identity = None
        if dep_path:
            dep_manifest = relative_meta_path(str(Path(dep_path) / "Cargo.toml"))
            metadata_identity = metadata_packages_by_manifest.get(dep_manifest)
        locked = lock_packages_by_name.get(dependency["name"], [])
        exact_locked = [package for package in locked if not dependency.get("source") or package.get("source") == dependency.get("source")]
        locked_identity = exact_locked[0] if len(exact_locked) == 1 else None
        out_of_fragment_workspace_dependency = bool(metadata_identity and not dep_identity)
        external_dependency_aliases[(package_id, alias)] = {
            "name": dependency["name"],
            "crate_name": alias,
            "declaring_workspace_id": source_identity["workspace_id"],
            "manifest_requirement": dependency.get("req"),
            "source": dependency.get("source") or (metadata_identity or {}).get("source") or (locked_identity.get("source") if locked_identity else None),
            "version": (metadata_identity or {}).get("version") or (locked_identity.get("version") if locked_identity else None),
            "locked_versions": sorted({package.get("version") for package in exact_locked}),
            "checksum": locked_identity.get("checksum") if locked_identity else None,
            "package_id": (metadata_identity or {}).get("id"),
            "manifest_path": dep_manifest if dep_path else None,
            "dependency_workspace_id": source_identity["workspace_id"] if metadata_identity else None,
            "dependency_declaration_source": {"manifest_path": source_identity["manifest_path"], "manifest_sha256": source_identity["manifest_sha256"], "metadata_path": source_identity["metadata_evidence"]["path"], "metadata_commit": source_identity["metadata_evidence"]["commit"], "metadata_sha256": source_identity["metadata_evidence"]["sha256"]},
            "lock_source": {"path": cargo_lock_source["path"], "commit": cargo_lock_source["commit"], "sha256": cargo_lock_source["sha256"], "lines": cargo_lock_source["lines"]},
            "normalized_package_identity": ((metadata_identity or {}).get("name") or dependency["name"]) + "@" + str((metadata_identity or {}).get("version") or (locked_identity or {}).get("version") or "unresolved") + (" from " + str((metadata_identity or {}).get("source") or (locked_identity or {}).get("source")) if (metadata_identity or {}).get("source") or (locked_identity or {}).get("source") else ""),
            "locked_package_id": ("{} {} ({})".format(locked_identity["name"], locked_identity["version"], locked_identity.get("source") or "workspace") if locked_identity else None),
            "dependency_kind": dependency.get("kind"),
            "optional": dependency.get("optional"),
            "target_cfg": dependency.get("target"),
            "requested_features": dependency.get("features", []),
            "uses_default_features": dependency.get("uses_default_features"),
            "registry": dependency.get("registry"),
            "feature_resolution": "Requested features are taken from accepted no-deps Cargo metadata; enabled dependency feature closure was not evaluated in this inventory.",
            "status": "workspace_dependency_outside_fragment" if out_of_fragment_workspace_dependency else "workspace_metadata_identity" if metadata_identity else "exact_lock_identity" if locked_identity else "version_not_unique_or_missing_in_lock",
        }

item_index = {(item["package_id"], item["current_qualified_symbol"]): item for item in items}
module_index = {(package_id, key[1]) for key in module_declarations if isinstance(key, tuple) for package_id in [key[0]]}
export_path_index = {(export.get("source_package_id"), export["current_qualified_symbol"]) for export in exports if export.get("source_package_id")}


def path_exists(package_id, path):
    return (package_id, path) in item_index or (package_id, path) in module_index or (package_id, path) in export_path_index


def resolve_reference(package_id, raw_path, source_module):
    raw = str(raw_path or "").strip().replace(" ", "")
    if not raw:
        return None, None, "empty"
    crate_name = package_crate_names.get(package_id, "")
    if raw.startswith("crate::"):
        return package_id, crate_name + raw[len("crate"):], "explicit"
    if raw == "crate":
        return package_id, crate_name, "explicit"
    if raw.startswith("self::"):
        return package_id, source_module + raw[len("self"):], "explicit"
    if raw == "self":
        return package_id, source_module, "explicit"
    if raw.startswith("super::"):
        parent = source_module.split("::")
        while raw.startswith("super::"):
            parent = parent[:-1]
            raw = raw[len("super::"):]
        return package_id, "::".join(parent + [raw]), "explicit"
    if raw.startswith("::"):
        raw = raw[2:]
    parts = raw.split("::")
    if parts[0] in ("core", "std", "alloc"):
        return None, raw, "external"
    dependency_id = dependency_package_ids.get((package_id, parts[0]))
    if dependency_id:
        target_root = package_crate_names[dependency_id]
        target = "::".join([target_root] + parts[1:])
        return dependency_id, target, "dependency"
    if (package_id, parts[0]) in external_dependency_aliases:
        return None, raw, "external_dependency"
    if parts[0] == crate_name:
        return package_id, raw, "qualified"
    candidates = []
    local_candidate = source_module + "::" + raw if source_module else crate_name + "::" + raw
    root_candidate = crate_name + "::" + raw
    for candidate in (local_candidate, root_candidate):
        if path_exists(package_id, candidate) and candidate not in candidates:
            candidates.append(candidate)
    if candidates:
        if len(candidates) > 1:
            return package_id, candidates[0], "lexical_precedence"
        return package_id, candidates[0], "lexical"
    return package_id, local_candidate, "unresolved"


def external_dependency_owner(package_id, raw_path):
    raw = str(raw_path or "").strip()
    alias = raw.split("::", 1)[0]
    dependency = external_dependency_aliases.get((package_id, alias))
    if not dependency:
        return None
    suffix = raw[len(alias):].lstrip(":")
    owner = dict(dependency)
    owner["canonical_rust_path"] = raw
    owner["canonical_package_module_path"] = alias + ("::" + suffix if suffix else "")
    owner["crate_alias"] = alias
    owner["unresolved_suffix"] = "*" in suffix
    owner["resolution_limit"] = (
        "The dependency and exported path are source-bound; dependency crate definitions and wildcard members were not expanded in this fragment."
        if "*" in suffix else
        "The dependency package identity is source-bound; the external crate definition was not inventoried here."
    )
    return owner


def canonical_type_path(package_id, path):
    seen = set()
    current_package_id = package_id
    current_path = path
    while (current_package_id, current_path) not in seen:
        seen.add((current_package_id, current_path))
        target = reexport_targets.get((current_package_id, current_path))
        if not target or target == (current_package_id, current_path):
            break
        current_package_id, current_path = target
    return current_package_id, current_path


reexport_targets = {}
for export in exports:
    if export.get("import_kind") == "wildcard" or not export.get("raw_source_path"):
        continue
    source_package_id = export["source_package_id"]
    source_module = export.get("source_module", package_crate_names[source_package_id])
    definition_package_id, definition_path, resolution = resolve_reference(source_package_id, export["raw_source_path"], source_module)
    export["definition_package_id"] = definition_package_id
    export["definition_owner"] = definition_path
    export["definition_resolution"] = resolution
    if resolution == "external_dependency":
        external_owner = external_dependency_owner(source_package_id, export["raw_source_path"])
        if external_owner:
            export["definition_external_dependency"] = external_owner
            if external_owner["status"] == "workspace_dependency_outside_fragment":
                export["definition_resolution"] = "workspace_dependency_outside_fragment"
                export["definition_package_id"] = external_owner.get("package_id")
    reexport_targets[(source_package_id, export["current_qualified_symbol"])] = (definition_package_id, definition_path)

for _ in range(8):
    changed = False
    for export in exports:
        if export.get("import_kind") == "wildcard" or not export.get("raw_source_path"):
            continue
        package_id = export.get("definition_package_id")
        path = export.get("definition_owner")
        canonical_package, canonical_path = canonical_type_path(package_id, path)
        if (canonical_package, canonical_path) != (package_id, path):
            export["definition_package_id"] = canonical_package
            export["definition_owner"] = canonical_path
            changed = True
    for export in exports:
        if export.get("import_kind") == "wildcard" or not export.get("raw_source_path"):
            continue
        reexport_targets[(export["source_package_id"], export["current_qualified_symbol"])] = (export["definition_package_id"], export["definition_owner"])
    if not changed:
        break


def resolve_impl_target(item):
    raw_target = item.get("impl_target")
    if not raw_target:
        return None, None, "missing_target"
    target = re.sub(r"<.*", "", raw_target.strip()).strip()
    if target.startswith("&"):
        target = target.lstrip("&").strip()
    module_path = item.get("impl_module_path") or item.get("module_path") or package_crate_names[item["package_id"]]
    source_package_id, resolved, resolution = resolve_reference(item["package_id"], target, module_path)
    source_package_id, resolved = canonical_type_path(source_package_id, resolved)
    candidate = item_index.get((source_package_id, resolved))
    if candidate and candidate.get("kind") in ("struct", "enum", "trait", "type_alias", "union"):
        return candidate, source_package_id, resolution
    return None, source_package_id, "unresolved_exact:" + resolution


for item in list(items):
    if item.get("impl_target"):
        target_item, target_package_id, resolution = resolve_impl_target(item)
        item["impl_owner_resolution"] = resolution
        if target_item:
            method_name = item["current_qualified_symbol"].split("::")[-1]
            item["definition_owner"] = target_item["current_qualified_symbol"]
            item["definition_package_id"] = target_package_id
            item["current_qualified_symbol"] = target_item["current_qualified_symbol"] + "::" + method_name
            item["is_external_api"] = bool(target_item.get("is_external_api"))
            item["is_direct_module_export"] = bool(target_item.get("is_direct_module_export"))
            item["resolved_impl_owner"] = target_item["current_qualified_symbol"]
            item["resolved_impl_owner_package_id"] = target_package_id
        else:
            item["is_external_api"] = False
            item["is_direct_module_export"] = False

item_index = {(item["package_id"], item["current_qualified_symbol"]): item for item in items}


STANDARD_TRAITS = {
    "Default": "core::default::Default",
    "Iterator": "core::iter::Iterator",
    "IntoIterator": "core::iter::IntoIterator",
    "Clone": "core::clone::Clone",
    "Copy": "core::marker::Copy",
    "Debug": "core::fmt::Debug",
    "Display": "core::fmt::Display",
    "Error": "core::error::Error",
    "Eq": "core::cmp::Eq",
    "PartialEq": "core::cmp::PartialEq",
    "Ord": "core::cmp::Ord",
    "PartialOrd": "core::cmp::PartialOrd",
    "Hash": "core::hash::Hash",
    "From": "core::convert::From",
    "Into": "core::convert::Into",
    "AsRef": "core::convert::AsRef",
    "AsMut": "core::convert::AsMut",
    "Deref": "core::ops::Deref",
    "DerefMut": "core::ops::DerefMut",
    "Drop": "core::ops::Drop",
    "Add": "core::ops::Add",
    "AddAssign": "core::ops::AddAssign",
    "BitOr": "core::ops::BitOr",
    "BitOrAssign": "core::ops::BitOrAssign",
    "Not": "core::ops::Not",
}


def trait_owner_path(package_id, raw_trait, source_module):
    trait_base = re.sub(r"<.*", "", raw_trait.strip())
    if trait_base in STANDARD_TRAITS:
        return None, STANDARD_TRAITS[trait_base], "prelude_trait"
    if trait_base.startswith("fmt::") and trait_base.rsplit("::", 1)[-1] in STANDARD_TRAITS:
        return None, "core::fmt::" + trait_base.rsplit("::", 1)[-1], "standard_trait_alias"
    target_id, target_path, resolution = resolve_reference(package_id, trait_base, source_module)
    target_id, target_path = canonical_type_path(target_id, target_path)
    return target_id, target_path, resolution


def trait_impl_member_rows(impl_row):
    path = impl_row["source_path"]
    text = source_text(path)
    body_start = impl_row["open"] + 1
    body_end = impl_row["end"] - 1
    pattern = re.compile(r"(?m)^[ \t]*(?:default\s+)?(type|const|fn)\s+([A-Za-z_]\w*)")
    members = []
    for match in pattern.finditer(text, body_start, body_end):
        start = match.start()
        nested = False
        for opening, closing in brace_pairs(text).items():
            if body_start < opening < start < closing < body_end:
                nested = True
                break
        if nested:
            continue
        kind = {"type": "trait_impl_associated_type", "const": "trait_impl_associated_const", "fn": "trait_impl_method"}[match.group(1)]
        end = end_header(text, start, "fn" if kind == "trait_impl_method" else "plain")
        signature = text[start:end].strip()
        member_cfg = attributes_before(text, start)
        members.append({
            "name": match.group(2),
            "kind": kind,
            "start": start,
            "line": line_number(text, start),
            "signature": compact(signature),
            "type_value": compact(signature.split("=", 1)[1].rstrip(";").strip()) if kind == "trait_impl_associated_type" and "=" in signature else None,
            "cfg_attributes": member_cfg,
        })
    return members


trait_impl_rows = []
for impl_row in all_impls:
    if not impl_row.get("trait_type"):
        continue
    raw_target = re.sub(r"<.*", "", impl_row["target_type"]).strip()
    module_path = "::".join(impl_row["module"])
    target_id, target_path, target_resolution = resolve_reference(impl_row["package_id"], raw_target, module_path)
    target_id, target_path = canonical_type_path(target_id, target_path)
    target_item = item_index.get((target_id, target_path))
    trait_id, trait_path, trait_resolution = trait_owner_path(impl_row["package_id"], impl_row["trait_type"], module_path)
    if not target_item:
        continue
    impl_line = line_number(source_text(impl_row["source_path"]), impl_row["start"])
    impl_attrs = effective_cfg_attrs(impl_row["module"], attributes_before(source_text(impl_row["source_path"]), impl_row["start"]), impl_row["package_id"])
    impl_symbol = target_path + " as " + trait_path
    record = {
        "package": impl_row["package"],
        "package_id": impl_row["package_id"],
        "manifest_path": package_info_by_id[impl_row["package_id"]]["manifest_path"],
        "normalized_package_identity": package_info_by_id[impl_row["package_id"]]["normalized_package_identity"],
        "module_path": "::".join(impl_row["module"]),
        "current_qualified_symbol": impl_symbol,
        "kind": "trait_impl",
        "definition_owner": target_path,
        "definition_package_id": target_id,
        "implemented_trait": trait_path,
        "trait_package_id": trait_id,
        "target_type": impl_row["target_type"],
        "target_type_owner": target_path,
        "target": {"name": impl_row["target_name"], "kind": impl_row["target_kind"]},
        "generic_parameters": impl_row["generic_parameters"],
        "where_clause": impl_row["where_clause"],
        "cfg_attributes": impl_attrs,
        "cfg_features": cfg_features(impl_attrs),
        "signature": impl_row["header"],
        "source_path": impl_row["source_path"],
        "line": impl_line,
        "source_sha256": impl_row["source_sha256"],
        "signature_sha256": sha(impl_row["header"].encode()),
        "target_resolution": target_resolution,
        "trait_resolution": trait_resolution,
        "is_external_api": False,
        "evidence_state": "NOT VERIFIED",
        "trait_impl_members": [],
    }
    items.append(record)
    for member in trait_impl_member_rows(impl_row):
        member_symbol = impl_symbol + "::" + member["name"]
        item = {
            "package": impl_row["package"],
            "package_id": impl_row["package_id"],
            "manifest_path": package_info_by_id[impl_row["package_id"]]["manifest_path"],
            "normalized_package_identity": package_info_by_id[impl_row["package_id"]]["normalized_package_identity"],
            "module_path": "::".join(impl_row["module"]),
            "current_qualified_symbol": member_symbol,
            "kind": member["kind"],
            "definition_owner": target_path,
            "definition_package_id": target_id,
            "implemented_trait": trait_path,
            "trait_package_id": trait_id,
            "trait_impl_owner": impl_symbol,
            "target_type_owner": target_path,
            "target": {"name": impl_row["target_name"], "kind": impl_row["target_kind"]},
            "generic_parameters": impl_row["generic_parameters"],
            "where_clause": impl_row["where_clause"],
            "associated_type_value": member["type_value"],
            "reexports": [],
            "cfg_attributes": list(dict.fromkeys(impl_attrs + member["cfg_attributes"])),
            "cfg_features": cfg_features(impl_attrs + member["cfg_attributes"]),
            "signature": member["signature"],
            "source_path": impl_row["source_path"],
            "line": member["line"],
            "source_sha256": impl_row["source_sha256"],
            "signature_sha256": sha(member["signature"].encode()),
            "semantic_kinds": ["default_value"] if trait_path == "core::default::Default" and member["name"] == "default" else [],
            "states": [], "actions": [], "builders": [],
            "defaults": ["Default::default"] if trait_path == "core::default::Default" and member["name"] == "default" else [],
            "is_external_api": False,
            "visibility": "trait_impl_contract",
            "trait_resolution": trait_resolution,
            "target_resolution": target_resolution,
            "evidence_state": "NOT VERIFIED",
        }
        items.append(item)
        record["trait_impl_members"].append(member_symbol)


item_index = {(item["package_id"], item["current_qualified_symbol"]): item for item in items}


def item_public_bindings(item):
    owner = item.get("definition_owner") or item["current_qualified_symbol"]
    owner_id = item.get("definition_package_id") or item["package_id"]
    suffix = ""
    if item["current_qualified_symbol"] != owner:
        if item["current_qualified_symbol"].startswith(owner + "::"):
            suffix = item["current_qualified_symbol"][len(owner):]
        elif item["current_qualified_symbol"].startswith(owner + " as "):
            suffix = item["current_qualified_symbol"][len(owner):]
        elif item.get("trait_impl_owner"):
            suffix = item["current_qualified_symbol"][len(item["trait_impl_owner"]):]
    result = {}
    if item.get("is_direct_module_export"):
        result[(item["package_id"], item["current_qualified_symbol"])] = None
    owner_item = item_index.get((owner_id, owner))
    if owner_item and owner_item is not item:
        for binding in item_public_bindings(owner_item):
            result[(binding["package_id"], binding["qualified_path"] + suffix)] = None
    for export in exports:
        if not export.get("is_external_api") or export.get("import_kind") == "wildcard":
            continue
        if (export.get("definition_package_id"), export.get("definition_owner")) == (owner_id, owner):
            package_id = export.get("source_package_id") or export.get("package_id")
            result[(package_id, export["current_qualified_symbol"] + suffix)] = export
    return [
        {"workspace_id": package_info_by_id[package_id]["workspace_id"], "package_id": package_id, "manifest_path": package_info_by_id[package_id]["manifest_path"], "qualified_path": path, "via_export": export.get("current_qualified_symbol") if export else None}
        for (package_id, path), export in sorted(result.items())
    ]


def item_public_paths(item):
    return sorted({row["qualified_path"] for row in item_public_bindings(item)})


def wildcard_members(export):
    target_id = export.get("definition_package_id")
    target_module = export.get("definition_owner", "").removesuffix("::*")
    if not target_id or not target_module:
        return []
    members = {}
    visited = set()

    def walk(package_id, module_path, visible_prefix):
        marker = (package_id, module_path, visible_prefix)
        if marker in visited:
            return
        visited.add(marker)
        for item in items:
            if item.get("package_id") != package_id or item.get("module_path") != module_path:
                continue
            if item.get("visibility") != "pub" or item.get("kind") in ("method", "field", "tuple_field", "enum_variant", "trait_method", "associated_type", "associated_const", "trait_impl", "trait_impl_method", "trait_impl_associated_type", "trait_impl_associated_const"):
                continue
            leaf = item["current_qualified_symbol"].split("::")[-1]
            relative = "::".join(filter(None, (visible_prefix, leaf)))
            members[relative] = (item.get("definition_package_id", package_id), item.get("definition_owner", item["current_qualified_symbol"]), item)
            child_module = (package_id, item["current_qualified_symbol"])
            module_rows = module_declarations.get(child_module, [])
            if item.get("kind") == "mod" and any(row["visibility"] == "pub" for row in module_rows):
                walk(package_id, item["current_qualified_symbol"], relative)
        for child in exports:
            if child.get("source_package_id") != package_id or child.get("source_module") != module_path:
                continue
            child_kind = child.get("kind")
            target_package_id = child.get("definition_package_id") or package_id
            target_path = child.get("definition_owner", "")
            if child_kind == "wildcard_reexport":
                walk(target_package_id, target_path.removesuffix("::*"), visible_prefix)
                continue
            leaf = child["current_qualified_symbol"].split("::")[-1]
            relative = "::".join(filter(None, (visible_prefix, leaf)))
            members[relative] = (target_package_id, target_path, child)
            if (target_package_id, target_path) in module_index:
                walk(target_package_id, target_path, relative)

    walk(target_id, target_module, "")
    return [(name, *members[name]) for name in sorted(members)]


for export in exports:
    if export.get("import_kind") == "wildcard":
        raw_module = export.get("raw_source_path", "")
        if raw_module.endswith("::*"):
            raw_module = raw_module[:-3]
        target_id, target_module, resolution = resolve_reference(export["source_package_id"], raw_module, export.get("source_module", package_crate_names[export["source_package_id"]]))
        target_id, target_module = canonical_type_path(target_id, target_module)
        export["definition_package_id"] = target_id
        export["definition_owner"] = target_module + "::*"
        export["definition_resolution"] = resolution
        if resolution == "external_dependency":
            external_owner = external_dependency_owner(export["source_package_id"], export.get("raw_source_path"))
            if external_owner:
                export["definition_external_dependency"] = external_owner
                if external_owner["status"] == "workspace_dependency_outside_fragment":
                    export["definition_resolution"] = "workspace_dependency_outside_fragment"
                    export["definition_package_id"] = external_owner.get("package_id")

for _ in range(8):
    additions = []
    for export in list(exports):
        if export.get("kind") != "wildcard_reexport" or not export.get("is_external_api"):
            continue
        prefix = export["current_qualified_symbol"].removesuffix("::*")
        for name, definition_package_id, definition_owner, source_record_row in wildcard_members(export):
            public_path = prefix + "::" + name
            if any(existing.get("package_id") == export["source_package_id"] and existing.get("current_qualified_symbol") == public_path for existing in exports):
                continue
            source_item = source_record_row if source_record_row in items else item_index.get((definition_package_id, definition_owner))
            additions.append({
                "package": export["package"],
                "package_id": export["source_package_id"],
                "source_package_id": export["source_package_id"],
                "manifest_path": export["manifest_path"],
                "normalized_package_identity": export["normalized_package_identity"],
                "current_qualified_symbol": public_path,
                "kind": "wildcard_member",
                "definition_owner": definition_owner,
                "definition_package_id": definition_package_id,
                "reexports": [public_path],
                "cfg_attributes": list(dict.fromkeys(export.get("cfg_attributes", []) + (source_item or {}).get("cfg_attributes", []))),
                "cfg_features": sorted(set(export.get("cfg_features", []) + (source_item or {}).get("cfg_features", []))),
                "signature": export["signature"],
                "source_path": export["source_path"],
                "line": export["line"],
                "source_sha256": export["source_sha256"],
                "signature_sha256": export["signature_sha256"],
                "import_kind": "wildcard_member",
                "is_external_api": True,
                "target": export["target"],
                "definition_source": {"path": (source_item or {}).get("source_path"), "line": (source_item or {}).get("line"), "sha256": (source_item or {}).get("source_sha256")},
            })
    if not additions:
        break
    exports.extend(additions)
    export_path_index.update((row["package_id"], row["current_qualified_symbol"]) for row in additions)


for export in exports:
    if export.get("kind") == "wildcard_reexport" or export.get("definition_owner", "").endswith("::*"):
        continue
    owner_id = export.get("definition_package_id") or export.get("source_package_id") or export.get("package_id")
    owner_path = export["definition_owner"]
    owner = item_index.get((owner_id, owner_path))
    if owner:
        owner.setdefault("reexports", []).append(export["current_qualified_symbol"])
        owner.setdefault("reexport_bindings", []).append({"workspace_id": export.get("workspace_id"), "package_id": export.get("source_package_id"), "manifest_path": export.get("manifest_path"), "qualified_path": export["current_qualified_symbol"]})
        if export.get("is_external_api"):
            owner["is_external_api"] = True

for item in items:
    item["namespace"] = (
        "module" if item.get("kind") == "mod" else
        "macro" if item.get("kind") == "macro" else
        "trait_impl_contract" if item.get("kind") == "trait_impl" else
        "trait_impl_member" if item.get("kind", "").startswith("trait_impl_") else
        "field" if item.get("kind") in ("field", "tuple_field") else
        "enum_variant" if item.get("kind") == "enum_variant" else
        "type" if item.get("kind") in ("struct", "enum", "union", "trait", "type_alias") else
        "value"
    )
    item["public_bindings"] = item_public_bindings(item)
    item["public_qualified_paths"] = item_public_paths(item)
    item["is_external_api"] = bool(item.get("is_external_api") or item["public_qualified_paths"])
    if item.get("is_external_api") and item.get("kind") in ("method", "field", "tuple_field", "enum_variant", "trait_method", "associated_type", "associated_const"):
        item["public_qualified_paths"] = item_public_paths(item)

direct_exports = []
existing_public_export_keys = {
    (row.get("source_package_id") or row.get("package_id"), row.get("current_qualified_symbol"), row.get("definition_package_id"), row.get("definition_owner"))
    for row in exports
}
for item in items:
    if item.get("kind") in ("field", "tuple_field", "enum_variant"):
        continue
    if item.get("is_external_api") and item.get("public_qualified_paths"):
        for binding in item.get("public_bindings", []):
            public_path = binding["qualified_path"]
            public_package_id = binding["package_id"]
            public_package = package_info_by_id[public_package_id]
            item_kind = item.get("kind", "")
            if item_kind == "trait_impl":
                export_kind = "trait_impl_contract"
            elif item_kind.startswith("trait_impl_"):
                export_kind = "trait_impl_member"
            else:
                export_kind = "public_definition" if public_package_id == item["package_id"] and public_path == item["current_qualified_symbol"] else "reachable_item"
            export_key = (public_package_id, public_path, item.get("definition_package_id", item["package_id"]), item.get("definition_owner", item["current_qualified_symbol"]))
            if export_key in existing_public_export_keys:
                continue
            existing_public_export_keys.add(export_key)
            direct_exports.append({
                "package": public_package["package_name"],
                "package_id": public_package_id,
                "source_package_id": public_package_id,
                "manifest_path": public_package["manifest_path"],
                "normalized_package_identity": public_package["normalized_package_identity"],
                "current_qualified_symbol": public_path,
                "kind": export_kind,
                "definition_owner": item.get("definition_owner", item["current_qualified_symbol"]),
                "definition_package_id": item.get("definition_package_id", item["package_id"]),
                "namespace": item.get("namespace"),
                "reexports": item.get("reexports", []),
                "source_module": item.get("module_path"),
                "cfg_attributes": list(dict.fromkeys(item.get("cfg_attributes", []) + ((binding.get("via_export") and next((x.get("cfg_attributes", []) for x in exports if x.get("source_package_id") == public_package_id and x.get("current_qualified_symbol") == binding.get("via_export")), [])) or []))),
                "cfg_features": sorted(set(item.get("cfg_features", []) + ((binding.get("via_export") and next((x.get("cfg_features", []) for x in exports if x.get("source_package_id") == public_package_id and x.get("current_qualified_symbol") == binding.get("via_export")), [])) or []))),
                "signature": item["signature"],
                "source_path": item["source_path"],
                "line": item["line"],
                "source_sha256": item["source_sha256"],
                "signature_sha256": item["signature_sha256"],
                "target": item["target"],
                "trait_impl_owner": item.get("trait_impl_owner"),
                "implemented_trait": item.get("implemented_trait"),
            })
exports.extend(direct_exports)


def cfg_provenance(row, package_id, module_path):
    package = package_info_by_id.get(package_id)
    target = row.get("target") or {}
    target_row = next((value for value in (package or {}).get("targets", []) if value.get("name") == target.get("name")), None)
    return {
        "source": {"path": row.get("source_path"), "line": row.get("line"), "sha256": row.get("source_sha256")},
        "effective_attributes": row.get("cfg_attributes", []),
        "feature_names": row.get("cfg_features", []),
        "target_required_features": (target_row or {}).get("required_features", []),
        "enclosing_module_condition_groups": module_cfg_condition_groups(module_path or [], package_id),
        "evaluation": "Source attributes and metadata only; cfg alternatives were not compiler-evaluated.",
    }


cfg_provenance_records = {}


def intern_cfg_provenance(value):
    record_id = sha(json.dumps(value, sort_keys=True, ensure_ascii=False, separators=(",", ":")).encode())
    cfg_provenance_records.setdefault(record_id, value)
    return record_id


for item in items:
    source_package = package_info_by_id.get(item.get("package_id"))
    definition_package_id = item.get("definition_package_id") or item.get("package_id")
    definition_package = package_info_by_id.get(definition_package_id)
    if source_package:
        item["source_package_id"] = source_package["package_id"]
        item["source_workspace_id"] = source_package["workspace_id"]
        item["workspace_id"] = source_package["workspace_id"]
        item["manifest_path"] = source_package["manifest_path"]
        item["normalized_package_identity"] = source_package["normalized_package_identity"]
    item["definition_package_id"] = definition_package_id
    if definition_package:
        item["definition_workspace_id"] = definition_package["workspace_id"]
        item["definition_manifest_path"] = definition_package["manifest_path"]
        item["definition_package_identity"] = definition_package["normalized_package_identity"]
    item["public_bindings"] = [
        {
            **binding,
            "source_package_id": binding.get("package_id"),
            "source_workspace_id": binding.get("workspace_id"),
        }
        for binding in item.get("public_bindings", [])
    ]
    item["cfg_provenance_id"] = intern_cfg_provenance(cfg_provenance(item, item.get("package_id"), item.get("module_path", "").split("::") if item.get("module_path") else []))
    item["cfg_provenance"] = {"record_id": item["cfg_provenance_id"]}
    item["namespace"] = (
        "module" if item.get("kind") == "mod" else
        "macro" if item.get("kind") == "macro" else
        "trait_impl_contract" if item.get("kind") == "trait_impl" else
        "trait_impl_member" if item.get("kind", "").startswith("trait_impl_") else
        "field" if item.get("kind") in ("field", "tuple_field") else
        "enum_variant" if item.get("kind") == "enum_variant" else
        "type" if item.get("kind") in ("struct", "enum", "union", "trait", "type_alias") else
        "value"
    )
    item_id_fields = [item.get("workspace_id"), item.get("package_id"), item.get("current_qualified_symbol"), item.get("definition_package_id"), item.get("definition_owner"), item.get("namespace"), item.get("kind"), item.get("source_path"), item.get("line"), item.get("source_sha256"), item.get("signature_sha256")]
    item["api_record_id"] = sha(json.dumps(item_id_fields, ensure_ascii=False, separators=(",", ":")).encode())

for export in exports:
    source_package_id = export.get("source_package_id") or export.get("package_id")
    source_package = package_info_by_id.get(source_package_id)
    definition_package_id = export.get("definition_package_id")
    definition_package = package_info_by_id.get(definition_package_id)
    if source_package:
        export["package_id"] = source_package["package_id"]
        export["source_package_id"] = source_package["package_id"]
        export["source_workspace_id"] = source_package["workspace_id"]
        export["workspace_id"] = source_package["workspace_id"]
        export["manifest_path"] = source_package["manifest_path"]
        export["normalized_package_identity"] = source_package["normalized_package_identity"]
    export["public_qualified_paths"] = [export["current_qualified_symbol"]]
    if definition_package:
        export["definition_workspace_id"] = definition_package["workspace_id"]
        export["definition_manifest_path"] = definition_package["manifest_path"]
        export["definition_package_identity"] = definition_package["normalized_package_identity"]
    owner_item = item_index.get((definition_package_id, export.get("definition_owner"))) if definition_package_id else None
    if owner_item:
        export["definition_source"] = {"path": owner_item["source_path"], "line": owner_item["line"], "sha256": owner_item["source_sha256"]}
        export["definition_cfg_provenance"] = owner_item.get("cfg_provenance")
    module_path = export.get("source_module", "").split("::") if export.get("source_module") else []
    export["cfg_provenance_id"] = intern_cfg_provenance(cfg_provenance(export, source_package_id, module_path))
    export["cfg_provenance"] = {"record_id": export["cfg_provenance_id"]}
    for binding in export.get("public_bindings", []):
        binding_package = package_info_by_id.get(binding.get("package_id"))
        if binding_package:
            binding["workspace_id"] = binding_package["workspace_id"]


def export_namespace(row):
    if row.get("namespace"):
        return row["namespace"]
    if row.get("kind") == "public_module":
        return "module"
    if row.get("kind") == "exported_macro":
        return "macro"
    if row.get("kind") == "trait_impl_contract":
        return "trait_impl_contract"
    if row.get("kind") == "trait_impl_member":
        return "trait_impl_member"
    owner = item_index.get((row.get("definition_package_id"), row.get("definition_owner")))
    if owner:
        return owner.get("namespace") or owner.get("kind")
    if row.get("definition_external_dependency"):
        return "external_dependency_item"
    return "unresolved_or_out_of_fragment"


export_groups = {}
for export in exports:
    export["namespace"] = export_namespace(export)
    key = (
        export.get("workspace_id"), export.get("package_id"), export.get("current_qualified_symbol"),
        export.get("definition_package_id"), export.get("definition_owner"), export.get("namespace"),
    )
    export_groups.setdefault(key, []).append(export)

grouped_exports = []
for key, rows in export_groups.items():
    row = dict(rows[0])
    row["export_key"] = {
        "workspace_id": key[0], "package_id": key[1], "canonical_public_path": key[2],
        "definition_package_id": key[3], "definition_owner": key[4], "namespace": key[5],
    }
    row["export_id"] = sha(json.dumps(row["export_key"], sort_keys=True, ensure_ascii=False, separators=(",", ":")).encode())
    row["public_qualified_paths"] = sorted({value["current_qualified_symbol"] for value in rows})
    row["reexports"] = sorted({path for value in rows for path in value.get("reexports", [])})
    row["cfg_attributes"] = list(dict.fromkeys(attr for value in rows for attr in value.get("cfg_attributes", [])))
    row["cfg_features"] = sorted({feature for value in rows for feature in value.get("cfg_features", [])})
    row["source_provenance_count"] = len(rows)
    if len(rows) > 1:
        row["source_provenance"] = [
            {
                "kind": value.get("kind"), "import_kind": value.get("import_kind"),
                "source_module": value.get("source_module"), "raw_source_path": value.get("raw_source_path"),
                "source": {"path": value.get("source_path"), "line": value.get("line"), "sha256": value.get("source_sha256")},
                "signature": value.get("signature"), "signature_sha256": value.get("signature_sha256"),
                "cfg_provenance_id": value.get("cfg_provenance_id"), "definition_cfg_provenance": value.get("definition_cfg_provenance"),
            }
            for value in rows
        ]
    definition_sources = [value.get("definition_source") for value in rows if value.get("definition_source")]
    if len(definition_sources) > 1:
        row["definition_sources"] = list({json.dumps(value, sort_keys=True): value for value in definition_sources}.values())
    definition_cfg_alternatives = [value.get("definition_cfg_provenance") for value in rows if value.get("definition_cfg_provenance")]
    if len(definition_cfg_alternatives) > 1:
        row["definition_cfg_provenance_alternatives"] = definition_cfg_alternatives
    declaration_signatures = sorted({value.get("signature") for value in rows if value.get("signature")})
    if len(declaration_signatures) > 1:
        row["declaration_signatures"] = declaration_signatures
    row["cfg_provenance"] = {
        "record_ids": sorted({value.get("cfg_provenance_id") for value in rows if value.get("cfg_provenance_id")}),
        "evaluation": "All export declaration source alternatives are retained; cfg alternatives were not compiler-evaluated.",
    }
    grouped_exports.append(row)
exports = grouped_exports


def current_item_resolution(path, name, qualified=None, signature=None):
    def choose(candidates, unique_status, ambiguous_status):
        if len(candidates) == 1:
            return candidates[0], unique_status, candidates
        if signature and candidates:
            expected = compact(signature).replace("pub(crate) ", "").replace("pub ", "")
            signature_matches = [
                item for item in candidates
                if compact(item.get("signature", "")).replace("pub(crate) ", "").replace("pub ", "").startswith(expected)
            ]
            if len(signature_matches) == 1:
                return signature_matches[0], "signature_disambiguated_" + unique_status, signature_matches
            if signature_matches:
                candidates = signature_matches
        return None, ambiguous_status, candidates

    if qualified:
        exact = [item for item in items if item.get("source_path") == path and (item["current_qualified_symbol"] == qualified or qualified in item.get("public_qualified_paths", []))]
        if exact:
            return choose(exact, "exact", "ambiguous_exact")
        if "::" in qualified:
            suffix = "::" + qualified
            suffix_matches = [item for item in items if item.get("source_path") == path and item["current_qualified_symbol"].endswith(suffix)]
            if suffix_matches:
                return choose(suffix_matches, "qualified_source_suffix", "ambiguous_qualified_source_suffix")
            return None, "qualified_source_symbol_not_found", []
    choices = [item for item in items if item.get("source_path") == path and item["current_qualified_symbol"].split("::")[-1] == name]
    return choose(choices, "unique_source_scoped_symbol", "ambiguous_source_scoped_symbol") if choices else (None, "not_found", [])


def current_item_for(path, name, qualified=None):
    return current_item_resolution(path, name, qualified)[0]


def source_symbol(record, path):
    cleanup = record.get("cleanup_attachment")
    if isinstance(cleanup, dict) and isinstance(cleanup.get("symbol"), str):
        declaration = cleanup.get("declaration_line") or cleanup.get("symbol")
        return cleanup["symbol"].split("::")[-1], declaration
    attached = record.get("attached_item")
    if isinstance(attached, dict):
        value = attached.get("name")
        return value.split("::")[-1] if isinstance(value, str) else None, attached.get("signature")
    if isinstance(attached, str):
        match = re.search(r"(?:fn|struct|enum|trait|type|const|static|mod)\s+`?([A-Za-z_]\w*)", attached)
        if not match:
            match = re.search(r"`([A-Za-z_]\w*)`", attached)
        if match:
            return match.group(1), record.get("attached_item_signature")
    for key in ("symbol", "item", "name"):
        if isinstance(record.get(key), str):
            return record[key].split("::")[-1], record.get("declaration_line") or record.get("attached_item_signature")
    return None, None


def make_requirement(requirement_id, path, source_line, source_sha, source_fact, clause, disposition, owner_task, reason, decision=None, destination=None, cases=None, current_symbol=None, evidence_source=None, api_binding_context=None):
    return {
        "id": requirement_id,
        "current_qualified_symbol": current_symbol,
        "source_backed_caller_behavior": clause or source_fact,
        "source": {"path": path, "line": source_line, "sha256": source_sha, "evidence_record": evidence_source},
        "named_owner_task": owner_task,
        "proposed_required_cases": cases or (["Exercise the stated caller rule through the named public path."] if clause else []),
        "evidence_state": "NOT VERIFIED",
        "disposition": disposition,
        "disposition_reason": reason,
        "review_decision": decision,
        "maintained_destination": destination,
        "block_final": disposition == "UNRESOLVED",
        **(api_binding_context or {}),
    }


def api_binding_context(path, line, source_sha, source_commit, source_symbol_value, record, evidence_path, record_id, current_item, binding_resolution, binding_candidates=None):
    source_package = package_for_source(path)
    source_identity = {
        "qualified_symbol": source_symbol_value,
        "path": path,
        "line": line,
        "commit": source_commit,
        "commit_resolution": "explicit in T37 source or cleanup attachment" if source_commit else "not supplied by T37 source record; no commit inferred",
        "sha256": source_sha,
        "text_sha256": record.get("text_sha256"),
        "package_id": source_package["package_id"] if source_package else None,
        "manifest_path": source_package["manifest_path"] if source_package else None,
        "workspace_id": source_package["workspace_id"] if source_package else None,
        "normalized_package_identity": source_package["normalized_package_identity"] if source_package else None,
    }
    cleanup = record.get("cleanup_attachment")
    evidence_sha = sha((ROOT / evidence_path).read_bytes())
    context = {
        "source_api_identity": source_identity,
        "cleanup_attachment": cleanup,
        "cleanup_attachment_evidence": {"path": evidence_path, "record_id": record_id, "sha256": evidence_sha},
        "current_binding_resolution": binding_resolution,
        "current_binding": None,
        "current_binding_candidates": [],
    }
    for candidate in binding_candidates or []:
        context["current_binding_candidates"].append({
            "qualified_symbol": candidate.get("current_qualified_symbol"),
            "definition_owner": candidate.get("definition_owner"),
            "package_id": candidate.get("package_id"),
            "manifest_path": candidate.get("manifest_path"),
            "workspace_id": candidate.get("workspace_id"),
            "definition_package_id": candidate.get("definition_package_id"),
            "definition_workspace_id": candidate.get("definition_workspace_id"),
            "source": {"path": candidate.get("source_path"), "line": candidate.get("line"), "sha256": candidate.get("source_sha256")},
        })
    if current_item:
        context["current_binding"] = {
            "qualified_symbol": current_item["current_qualified_symbol"],
            "definition_owner": current_item.get("definition_owner"),
            "package_id": current_item.get("package_id"),
            "manifest_path": current_item.get("manifest_path"),
            "workspace_id": current_item.get("workspace_id"),
            "definition_package_id": current_item.get("definition_package_id"),
            "definition_workspace_id": current_item.get("definition_workspace_id"),
            "architecture_owner_task": current_item.get("architecture_owner_task"),
            "source": {"path": current_item.get("source_path"), "line": current_item.get("line"), "sha256": current_item.get("source_sha256")},
            "evidence_state": "NOT VERIFIED",
        }
    return context


def proposed_open_case(path, symbol, record, owner_task, clause):
    decisions = " ".join(record.get("decision_ids", [])) if isinstance(record.get("decision_ids"), list) else str(record.get("decision_ids", ""))
    excerpt = str(record.get("supporting_excerpt") or record.get("coverage_rationale") or clause)
    if "CANVAS-WIDTH" in decisions or path.endswith("author/canvas.rs"):
        return "Call text_width and put_text with a wide grapheme and an adjacent run; assert the cell-width and advance outcome after T10 resolves the rule."
    if "PASS-THROUGH" in decisions:
        return "Register a lower-layer click target and a top backdrop, then route a pointer press; assert delivery or non-delivery under the approved T15 policy."
    if "FOOTER-PIN" in decisions or path.endswith("shell/footer.rs"):
        return "Compose automatic, normal, pinned, and custom hints with an error status; assert priority and input order under the approved T19 policy."
    if "VISIT-KEY" in decisions or path.endswith("shell/nav.rs"):
        return "Leave and revisit a route at the visit-counter boundary; assert distinct keys or the documented terminal outcome."
    if "KEYS-TIER" in decisions or path.endswith("shell/router.rs"):
        return "Give the page and shell handlers the same chord, then exercise normal, capture, and focus routes; assert the approved T19 order."
    if path.endswith("core/keys.rs"):
        return "Cover refused and accepted Super/Alt observations, all documented Alt overrides, and active or inactive protocol flags; assert the public delivery result."
    if path.endswith("core/list.rs") and ("install" in str(symbol) or "current" in str(symbol)):
        return "Install one policy on a worker thread and read defaults and installed values on a second thread; assert the documented scope."
    if path.endswith("core/list.rs") and ("Cursor" in str(symbol) or "cursor" in excerpt.lower()):
        return "Initialize beyond the row extent, then move through empty, short, and disabled rows; assert constructor state separately from movement fallback."
    if path.endswith("core/list.rs"):
        return "Route a printable key and a bound key through Shortcuts and TypeAhead precedence; assert the selected path for each route."
    if path.endswith("core/selection.rs"):
        return "Extend an anchored selection across disabled rows, then reverse direction; assert saved-base membership and newly eligible rows."
    subject = symbol or path
    return "Exercise " + subject + " with inputs that distinguish the source statements; assert the exact outcome approved by " + owner_task + "."


requirements = []
historical_api_dispositions = []
disposition_files = [
    "docs/implementation/pr38-final/results/t37-api-runtime/foundation-runtime-disposition.json",
    "docs/implementation/pr38-final/results/t37-api-support/foundation-support-disposition.json",
    "docs/implementation/pr38-final/results/t37-api-values/decisions.json",
]
in_scope_path = lambda path: any(path.startswith(prefix) for prefix in [
    "crates/termrock-core/", "crates/termrock-layout/", "crates/termrock-theme/", "crates/termrock-text/", "crates/termrock-render/", "crates/termrock-collections/", "crates/termrock-runtime/", "crates/termrock-session/", "crates/termrock-shell/", "crates/termrock-terminal/", "crates/termrock-consumer-example/", "crates/termrock-xtask/", "crates/termrock-visibility-publisher/", "crates/visual-harness/", "crates/termrock-e2e/", "crates/termrock-visibility-tests/", "tools/refactor-proof/", "external-consumer-demo/", "tools/termrock-visibility-publisher/"
])

for filename in disposition_files:
    data = json.loads((ROOT / filename).read_text())
    records = data.get("records", [])
    for index, record in enumerate(records):
        path = record.get("path")
        if not path or not in_scope_path(path):
            continue
        semantic = record.get("semantic_disposition") or record.get("disposition") or ""
        source_map_id = record.get("id", str(index))
        name, signature = source_symbol(record, path)
        attached = record.get("attached_item")
        qualified = attached.get("name") if isinstance(attached, dict) else None
        if not qualified and isinstance(record.get("cleanup_attachment"), dict):
            qualified = record["cleanup_attachment"].get("symbol")
        item, binding_resolution, binding_candidates = current_item_resolution(path, name, qualified, signature) if name else (None, "no_symbol", [])
        symbol = item["current_qualified_symbol"] if item else None
        historical_symbol = qualified if isinstance(qualified, str) and "::" in qualified else (item["current_qualified_symbol"] if item else name)
        required = record.get("required_clause")
        if isinstance(required, list):
            clauses = required
        elif required:
            clauses = [required]
        else:
            clauses = record.get("required_clauses", [])
        dest = record.get("destination")
        line = (record.get("line_range") or {}).get("start") or record.get("line")
        source_sha = record.get("source_sha256")
        context = api_binding_context(path, line, source_sha, record.get("source_commit") or (record.get("cleanup_attachment") or {}).get("source_commit"), historical_symbol, record, filename, source_map_id, item, binding_resolution, binding_candidates)
        source_text_value = record.get("source_text")
        if isinstance(source_text_value, list):
            source_text_value = "\n".join(source_text_value)
        if source_text_value is None:
            source_text_value = record.get("source_excerpt") or record.get("attached_item_signature") or signature
        review = record.get("review_decision") or record.get("decision") or record.get("decision_ids")
        context["source_scoped_decision_id"] = (review.get("decision_id") if isinstance(review, dict) else None) or record.get("decision_id") or record.get("decision_ids")
        context["decision_artifact"] = {"path": filename, "record_id": source_map_id, "sha256": context["cleanup_attachment_evidence"]["sha256"]}
        owner_task = owner_for_path(path, symbol or name or "")
        if "T15-PASS-THROUGH-01" in str(record.get("decision_ids", "")):
            owner_task = "T15"
        if semantic in ("maintained_required_contract", "required_caller_contract", "required_maintained_clause"):
            clause_values = [value.get("required_clause") or value.get("supporting_excerpt") or json.dumps(value, sort_keys=True) if isinstance(value, dict) else value for value in (clauses or [source_text_value])]
            clause_values = [str(value) for value in clause_values if value]
            for clause_index, clause in enumerate(clause_values):
                requirements.append(make_requirement(
                    "T37-" + str(source_map_id) + ("-" + str(clause_index + 1) if len(clauses) > 1 else ""), path, line, source_sha, source_text_value, clause, "KEEP", owner_task,
                    "T37 maps this caller rule to maintained guidance. The current product source item is inventoried separately.", review, dest,
                    ["Verify: " + str(clause)], symbol,
                    {"path": filename, "record_id": source_map_id, "semantic_disposition": semantic},
                    context,
                ))
        elif semantic in ("open_contract_followup", "source_wording_conflict", "unresolved_source_contract_conflict"):
            clause = record.get("required_clause") or record.get("source_fact") or record.get("required_clauses") or record.get("specific_reason") or source_text_value
            clause_values = clause if isinstance(clause, list) else [clause]
            clause_values = [value.get("required_clause") or value.get("supporting_excerpt") or json.dumps(value, sort_keys=True) if isinstance(value, dict) else value for value in clause_values]
            clause_values = [str(value) for value in clause_values if value]
            if not clause_values:
                excerpt = record.get("supporting_excerpt") or record.get("source_excerpt") or source_text_value or record.get("coverage_rationale")
                clause_values = [str(excerpt or record.get("reason") or "The current source contract has an unresolved conflict.")]
            for clause_index, clause_value in enumerate(clause_values):
                cases = record.get("future_case_scope") or [proposed_open_case(path, symbol, record, owner_task, clause_value)]
                if not isinstance(cases, list):
                    cases = [str(cases)]
                requirements.append(make_requirement(
                    "T37-OPEN-" + str(source_map_id) + ("-" + str(clause_index + 1) if len(clause_values) > 1 else ""), path, line, source_sha, source_text_value, clause_value, "UNRESOLVED", owner_task,
                    record.get("specific_reason") or record.get("reason") or "The retained source statement needs a named product decision before final acceptance.", review, dest,
                    cases, symbol,
                    {"path": filename, "record_id": source_map_id, "semantic_disposition": semantic},
                    context,
                ))
        elif semantic == "planned_api_retirement":
            clause = record.get("semantic_decision") or record.get("specific_reason") or source_text_value
            requirements.append(make_requirement(
                "T37-ADAPT-" + str(source_map_id), path, line, source_sha, source_text_value, clause, "ADAPT", owner_task,
                "T37 records a planned owner migration. The item remains current at this input and stays inventoried until that owner applies the migration.", review, dest,
                record.get("future_case_scope") or ["Verify the planned migration at the public caller path after the owner task."], symbol,
                {"path": filename, "record_id": source_map_id, "semantic_disposition": semantic},
                context,
            ))
        else:
            historical_api_dispositions.append({
                "id": str(source_map_id),
                "source_record_id": source_map_id,
                "source_path": path,
                "source_line": line,
                "source_sha256": source_sha,
                "source_commit": record.get("source_commit") or (record.get("cleanup_attachment") or {}).get("source_commit"),
                "source_text": source_text_value,
                "source_api_identity": context["source_api_identity"],
                "attached_symbol": historical_symbol,
                "semantic_disposition": semantic,
                "current_symbol_present": bool(item),
                "current_binding_resolution": binding_resolution,
                "current_binding": context["current_binding"],
                "cleanup_attachment": context["cleanup_attachment"],
                "cleanup_attachment_evidence": context["cleanup_attachment_evidence"],
                "source_scoped_decision_id": context["source_scoped_decision_id"],
                "decision_artifact": context["decision_artifact"],
                "destination": dest,
                "specific_reason": record.get("specific_reason") or record.get("reason"),
                "owner_task": owner_task,
                "named_owner_task": owner_task,
                "evidence_state": "NOT VERIFIED",
                "disposition": "KEEP",
                "disposition_reason": (
                    "Keep this compatibility record with its current source path. Caller behavior remains unverified."
                    if semantic == "retained_public_compatibility_data" else
                    "Keep this T37 source record as migration evidence. It does not establish product behavior."
                ),
                "evidence_record": filename,
            })


removed = json.loads((ROOT / "docs/implementation/pr38-final/results/t37-foundations/source-removal.json").read_text())
deferred = json.loads((ROOT / "docs/implementation/pr38-final/results/t37-foundations/deferred-testing-surfaces.json").read_text())
decision_map = deferred["review_decisions"]
for group in deferred["groups"]:
    group_id = group["id"]
    public_paths = group.get("public_paths", [])
    status = group.get("status", "")
    decision_id = "T37-FOUNDATION-TESTING-01"
    if group_id in ("ui-styled-query-probe-state", "rowui-style-probe-call-sites", "part-style-note-compatibility"):
        decision_id = "T37-FOUNDATION-TESTING-02"
    elif group_id in ("cx-event-record-probes", "runtime-stub-harness"):
        decision_id = "T37-FOUNDATION-TESTING-03"
    for public_path in public_paths:
        if group_id == "part-style-note-compatibility":
            disposition = "KEEP"
            reason = "T37 kept the public method and normal call sites. Only its testing recorder call was removed."
            decision = decision_map[decision_id]
        else:
            current_match = [item for item in items if item["current_qualified_symbol"] == public_path or item["current_qualified_symbol"].startswith(public_path + "::")]
            if current_match:
                continue
            disposition = "RETIRE"
            reason = status + " T37 source inventory has no current production caller for this testing-only surface."
            decision = decision_map[decision_id]
        source_path = (group.get("input_scope_lines") or {})
        path = next(iter(source_path), "docs/implementation/pr38-final/results/t37-foundations/deferred-testing-surfaces.json")
        source_line = next(iter(source_path.get(path, [])), None)
        if group_id == "runtime-stub-harness":
            if "runtime::stub::" in public_path:
                path, source_line = "crates/termrock-runtime/src/runtime.rs", 3301
            else:
                path, source_line = "crates/termrock-runtime/src/lib.rs", 55
        source_commit = decision.get("input_commit")
        historical_result = subprocess.run(["git", "show", source_commit + ":" + path], cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.PIPE) if source_commit and path else None
        historical_bytes = historical_result.stdout if historical_result and historical_result.returncode == 0 else None
        historical_lines = historical_bytes.decode("utf-8").splitlines() if historical_bytes else []
        historical_line_text = historical_lines[source_line - 1] if historical_bytes and source_line and source_line <= len(historical_lines) else None
        source_package = package_for_source(path)
        current_item, current_resolution, _ = current_item_resolution(path, public_path.split("::")[-1], public_path)
        evidence_path = "docs/implementation/pr38-final/results/t37-foundations/deferred-testing-surfaces.json"
        evidence_sha = sha((ROOT / evidence_path).read_bytes())
        retired_context = {
            "source_api_identity": {"qualified_symbol": public_path, "path": path, "line": source_line, "line_role": "exact API declaration or T37 input-scope anchor, according to the linked decision record", "commit": source_commit, "commit_resolution": "explicit in the independent review decision", "sha256": sha(historical_bytes) if historical_bytes else None, "text_sha256": sha(historical_line_text.encode()) if historical_line_text else None, "package_id": source_package["package_id"] if source_package else None, "manifest_path": source_package["manifest_path"] if source_package else None, "workspace_id": source_package["workspace_id"] if source_package else None, "normalized_package_identity": source_package["normalized_package_identity"] if source_package else None},
            "current_input_commit": INPUT,
            "current_input_absence": current_item is None,
            "cleanup_attachment": None,
            "cleanup_attachment_evidence": {"path": evidence_path, "record_id": group_id, "sha256": evidence_sha},
            "current_binding_resolution": current_resolution if current_item else "absent_by_approved_t37_removal",
            "current_binding": None,
            "source_scoped_decision_id": decision_id,
            "decision_artifact": {"path": evidence_path, "record_id": group_id, "sha256": evidence_sha},
        }
        if current_item:
            retired_context["current_binding"] = {
                "qualified_symbol": current_item["current_qualified_symbol"],
                "definition_owner": current_item.get("definition_owner"),
                "package_id": current_item.get("package_id"),
                "manifest_path": current_item.get("manifest_path"),
                "source": {"path": current_item.get("source_path"), "line": current_item.get("line"), "sha256": current_item.get("source_sha256")},
                "evidence_state": "NOT VERIFIED",
            }
        requirements.append(make_requirement(
            "T37-RETIRED-" + group_id + "-" + public_path.replace("::", "-"), path, source_line, None,
            "T37 recorded this testing-only public path and its retained or removed callers.",
            "Keep this path retired after T37, or preserve the explicitly accepted compatibility path.",
            disposition, owner_for_path(path, public_path), reason,
            {"decision_id": decision_id, "reviewer": decision.get("reviewer"), "reviewer_model": "gpt-6.1-sol", "reviewer_effort": "high", "decision": decision.get("decision"), "input_commit": decision.get("input_commit")},
            None, ["Check metadata and the source inventory for this exact qualified path."], current_item["current_qualified_symbol"] if current_item else None,
            {"path": "docs/implementation/pr38-final/results/t37-foundations/deferred-testing-surfaces.json", "group_id": group_id},
            retired_context,
        ))

draw_scene = current_item_for("crates/termrock-runtime/src/runtime.rs", "draw_scene")
if draw_scene:
    requirements.append(make_requirement(
        "T37-KEEP-runtime-draw-scene",
        draw_scene["source_path"], draw_scene["line"], draw_scene["source_sha256"],
        "Runtime::draw_scene remains feature-gated. Two retained Form examples call it.",
        "Preserve the original `testing` feature signature and normal form-example compatibility.",
        "KEEP", "T18", "T37-BUILD-COMPAT-01 restored this method because retained form examples use it.",
        {"decision_id": "T37-BUILD-COMPAT-01", "reviewer": "/root/t00_independent_review", "model": "gpt-6.1-sol", "effort": "high"},
        None,
        ["Compile `form_lanes` under the declared testing feature.", "Compile `form_intrinsic_content` under the declared testing feature."],
        draw_scene["current_qualified_symbol"],
        {"path": "docs/implementation/pr38-final/results/t37-metadata/dependency-retirement-map.json", "decision_id": "T37-BUILD-COMPAT-01"},
    ))

for followup in deferred.get("open_shared_followups", []):
    if "shared docs owner" in followup.lower():
        continue

open_requirements = [
    {
        "id": row["id"],
        "path": row["source"]["path"],
        "named_owner_task": row["named_owner_task"],
        "reason": row["disposition_reason"],
        "required_outcome": row["source_backed_caller_behavior"],
        "proposed_required_cases": row["proposed_required_cases"],
        "evidence_state": row["evidence_state"],
        "block_final": row["block_final"],
    }
    for row in requirements if row["disposition"] == "UNRESOLVED"
]

for item in items:
    item["architecture_owner_task"] = owner_for_path(item["source_path"], item["current_qualified_symbol"])
    item["requirement_ids"] = [
        row["id"] for row in requirements
        if row.get("current_qualified_symbol") == item["current_qualified_symbol"] or row.get("current_qualified_symbol") in item.get("public_qualified_paths", [])
    ]
    item["evidence_state"] = "NOT VERIFIED"
for export in exports:
    definition_external = export.get("definition_external_dependency") or {}
    external_resolution = export.get("definition_resolution") in ("external_dependency", "workspace_dependency_outside_fragment")
    if external_resolution:
        owner_id = definition_external.get("package_id") if export.get("definition_resolution") == "workspace_dependency_outside_fragment" else None
    else:
        owner_id = export.get("definition_package_id") or export.get("source_package_id") or export.get("package_id")
    owner_item = item_index.get((owner_id, export.get("definition_owner")))
    export["definition_owner_task"] = owner_item["architecture_owner_task"] if owner_item else (None if external_resolution else owner_for_path(export["source_path"], export["definition_owner"]))
    export["reexport_owner_task"] = owner_for_path(export["source_path"], export["current_qualified_symbol"])
    export["definition_package_id"] = owner_id
    if external_resolution and definition_external:
        export["definition_package_identity"] = definition_external.get("normalized_package_identity")
        export["definition_manifest_path"] = definition_external.get("manifest_path")
        export["definition_workspace_id"] = definition_external.get("dependency_workspace_id")
        export["definition_owner_scope"] = "workspace dependency outside fragment" if export.get("definition_resolution") == "workspace_dependency_outside_fragment" else "third-party dependency outside fragment"
    export["evidence_state"] = "NOT VERIFIED"
    export["requirement_ids"] = [
        row["id"] for row in requirements
        if row.get("current_qualified_symbol") == export["current_qualified_symbol"]
    ]

for requirement in requirements:
    package = package_for_source(requirement["source"]["path"])
    binding = requirement.get("current_binding") or {}
    if package is None and binding.get("package_id"):
        package = package_info_by_id.get(binding["package_id"])
    if package:
        requirement["package_id"] = package["package_id"]
        requirement["source_package_id"] = package["package_id"]
        requirement["workspace_id"] = package["workspace_id"]
        requirement["source_workspace_id"] = package["workspace_id"]
        requirement["manifest_path"] = package["manifest_path"]
        requirement["normalized_package_identity"] = package["normalized_package_identity"]
for record in historical_api_dispositions:
    package = package_for_source(record["source_path"])
    binding = record.get("current_binding") or {}
    if package is None and binding.get("package_id"):
        package = package_info_by_id.get(binding["package_id"])
    if package:
        record["package_id"] = package["package_id"]
        record["source_package_id"] = package["package_id"]
        record["workspace_id"] = package["workspace_id"]
        record["source_workspace_id"] = package["workspace_id"]
        record["manifest_path"] = package["manifest_path"]
        record["normalized_package_identity"] = package["normalized_package_identity"]

requirements_by_id = {row["id"]: row for row in requirements}
for open_requirement in open_requirements:
    requirement = requirements_by_id.get(open_requirement["id"])
    if requirement:
        for key in ("package_id", "source_package_id", "workspace_id", "source_workspace_id", "manifest_path", "normalized_package_identity"):
            if key in requirement:
                open_requirement[key] = requirement[key]


def historical_source(path):
    result = subprocess.run(["git", "show", PRODUCT + ":" + path], cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    if result.returncode != 0:
        return None
    return result.stdout


current_head = run(["git", "rev-parse", "HEAD"]).decode().strip()
input_ancestor = subprocess.run(["git", "merge-base", "--is-ancestor", INPUT, "HEAD"], cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
if input_ancestor.returncode != 0:
    raise RuntimeError("fixed inventory input is not an ancestor of HEAD")
product_source_diff_argv = ["git", "diff", "--exit-code", T37_INTEGRATION, "--", "Cargo.toml", "crates", "tools/refactor-proof", "external-consumer-demo"]
product_source_diff = subprocess.run(product_source_diff_argv, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
if product_source_diff.returncode != 0:
    raise RuntimeError("production source differs from accepted T37 tree")
input_tree = run(["git", "rev-parse", INPUT + "^{tree}"]).decode().strip()
identity_commands = [
    {"tool": "git", "argv": ["git", "rev-parse", "HEAD"], "cwd": str(ROOT), "exit_code": 0, "stdout": current_head, "purpose": "Confirm fixed inventory input commit."},
    {"tool": "git", "argv": ["git", "merge-base", "--is-ancestor", INPUT, "HEAD"], "cwd": str(ROOT), "exit_code": input_ancestor.returncode, "source_commit": INPUT, "purpose": "Confirm fixed input remains in current history."},
    {"tool": "git", "argv": product_source_diff_argv, "cwd": str(ROOT), "exit_code": product_source_diff.returncode, "source_commit": T37_INTEGRATION, "purpose": "Confirm product manifests and source match accepted T37 integration."},
    {"tool": "git", "argv": ["git", "rev-parse", INPUT + "^{tree}"], "cwd": str(ROOT), "exit_code": 0, "stdout": input_tree, "purpose": "Record fixed input tree identity."},
]

performance_workloads = []
for path in ["crates/termrock-test-support/tests/perf.rs", "crates/termrock-test-support/tests/perf_collections.rs", "crates/termrock-test-support/tests/perf_baseline.txt"]:
    raw = historical_source(path)
    if raw is None:
        continue
    text = raw.decode("utf-8", "replace")
    source_row = {
        "path": path,
        "commit": PRODUCT,
        "git_blob_oid": run(["git", "rev-parse", PRODUCT + ":" + path]).decode().strip(),
        "sha256": sha(raw),
        "lines": len(text.splitlines()),
        "status": "historical immutable workload or baseline source; not executed",
    }
    source_by_path["historical:" + path] = source_row
    if path.endswith(".txt"):
        ids = [line.strip() for line in text.splitlines() if line.strip() and not line.lstrip().startswith("#")]
        performance_workloads.append({
            "source": source_row,
            "kind": "historical_measurement_baseline",
            "record_ids": ids,
            "current_t33_measurements": "NOT RECORDED",
            "evidence_state": "NOT VERIFIED",
            "owner_task": "T33",
        })
    else:
        fnames = re.findall(r"(?m)^fn\s+([A-Za-z_]\w*)", text)
        tests = re.findall(r"(?m)^#\[test\]\s*\n\s*fn\s+([A-Za-z_]\w*)", text)
        workload_ids = re.findall(r'"([a-z][a-z0-9_]+)"\s*,\s*&(?:sample|s|stats)', text)
        performance_workloads.append({
            "source": source_row,
            "kind": "historical_workload_definition",
            "test_function_names": tests,
            "helper_function_names": sorted(set(fnames) - set(tests)),
            "recorded_workload_ids": sorted(set(workload_ids)),
            "current_t33_measurements": "NOT RECORDED",
            "evidence_state": "NOT VERIFIED",
            "owner_task": "T33",
        })


duplicate_relationships = []
for left, right, label in [
    ("crates/termrock-consumer-example/src", "external-consumer-demo/src", "workspace example and standalone consumer carry the same consumer source"),
    ("crates/termrock-visibility-publisher/src", "tools/termrock-visibility-publisher/src", "workspace implementation and standalone publisher copy"),
]:
    lfiles = sorted(p.relative_to(ROOT).as_posix() for p in (ROOT / left).rglob("*.rs"))
    rfiles = sorted(p.relative_to(ROOT).as_posix() for p in (ROOT / right).rglob("*.rs"))
    lhashes = {Path(p).relative_to(left).as_posix(): sha((ROOT / p).read_bytes()) for p in lfiles}
    rhashes = {Path(p).relative_to(right).as_posix(): sha((ROOT / p).read_bytes()) for p in rfiles}
    duplicate_relationships.append({
        "left_root": left,
        "right_root": right,
        "relationship": label,
        "same_relative_file_set": sorted(lhashes) == sorted(rhashes),
        "byte_identical_file_set": lhashes == rhashes,
        "left_files": lfiles,
        "right_files": rfiles,
        "left_canonical_note": "termrock-consumer-example is the root-workspace sample; the standalone package is an external consumer copy." if "consumer-example" in left else "The crates/ package is the workspace implementation; tools/ is a standalone workspace with a narrower target set.",
    })


retired_names = [
    "termrock_core::intent::IntentQueue::probes",
    "termrock_runtime::stub::{Control, Stub, deliver, key, mouse, runtime, step}",
    "termrock_runtime::runtime::RenderModel",
    "termrock_runtime::runtime::RenderSnapshot::bind_model",
    "termrock_runtime::runtime::ProjectedFrame",
    "termrock_runtime::runtime::Runtime::draw_projection",
    "termrock_runtime::runtime::Runtime::draw_bound_projection",
    "termrock_runtime::runtime::Runtime::{hover,state_of,focus_visible,top_layer,is_open,cursor,resolved,resolved_in,last_invalidate,records,clear_records,projection_registry,projection_ring,projection_cursor,draw_buffer,region_count,registry,advance_clock,repaint_after,capture_owner,set_focus,intent_probes,style_cache_stats,open_spec,layer_area,record_diagnostic}",
    "termrock_runtime::ui::StyledQuery",
    "termrock_runtime::ui::Ui::{note_styled,styled_parts,styled_queries,style_cache_stats}",
    "termrock_runtime::ui::Cx::record",
    "termrock_theme::Theme::fingerprint",
    "termrock_theme::resolve::StyleCache::stats",
]
current_symbol_names = {item["current_qualified_symbol"] for item in items}
def expand_retired_path(value):
    value = value.split(" (", 1)[0].removesuffix(" re-export").strip()
    match = re.match(r"^(.*)::\{([^{}]+)\}$", value)
    if not match:
        return [value]
    base, members = match.groups()
    return [base + "::" + member.strip() for member in members.split(",") if member.strip()]


deferred_public_paths = [path for group in deferred["groups"] for path in group.get("public_paths", [])]
retired_public_symbols = sorted({symbol for path in deferred_public_paths for symbol in expand_retired_path(path) if "::" in symbol})
retired_current_matches = sorted(set(retired_public_symbols) & current_symbol_names)
accepted_compatibility_matches = {
    "termrock_runtime::runtime::Runtime::draw_scene",
    "termrock_runtime::author::PartStyle::note",
}
unexpected_retired_current_matches = sorted(set(retired_current_matches) - accepted_compatibility_matches)


doc_sources = [source_record(path, kind="plan_document") for path in EVIDENCE_FILES]
doc_sources.append(source_record("Cargo.lock", kind="cargo_lock_external_owner_evidence"))
evidence_sources = [source_record(path, kind="t00_t37_evidence") for path in EVIDENCE_FILES if path.startswith("docs/implementation/pr38-final/results/")]
all_sources = sorted(source_by_path.values(), key=lambda row: (row["path"], row["commit"]))
task_document_hash = source_record("docs/implementation/pr38-final/task-index.json", kind="task_index")["sha256"]
metadata_structure = json.loads((ROOT / "docs/implementation/pr38-final/results/t37-final-integration/metadata-structure.json").read_text())
metadata_commands = []
metadata_base = ROOT.parent / "termrock-t37-metadata"
for workspace in metadata_structure["workspaces"]:
    metadata_commands.append({
        "tool": "/Users/donbeave/.local/bin/rust-build-guard",
        "argv": ["rust-build-guard", "--", "mise", "exec", "--", "cargo", "metadata", "--format-version", "1", "--no-deps", "--manifest-path", "Cargo.toml"],
        "cwd": str(metadata_base / workspace["workdir"]),
        "exit_code": workspace["exit_status"],
        "source_commit": META_HEAD,
        "stdout_path": workspace["stdout_path"],
        "stdout_sha256": workspace["stdout_sha256"],
        "stderr_path": workspace["stderr_path"],
        "stderr_sha256": workspace["stderr_sha256"],
        "reused_from_accepted_t37": True,
        "purpose": "Read target metadata only; this T01 run did not invoke Cargo.",
    })
historical_commands = []
historical_oid_commands = []
for workload in performance_workloads:
    source = workload["source"]
    historical_commands.append({
        "tool": "git",
        "argv": ["git", "show", PRODUCT + ":" + source["path"]],
        "cwd": str(ROOT),
        "exit_code": 0,
        "source_commit": PRODUCT,
        "source_path": source["path"],
        "source_blob_oid": source.get("git_blob_oid"),
        "stdout_sha256": source["sha256"],
        "purpose": "Read historical workload or baseline source. No workload ran.",
    })
    historical_oid_commands.append({
        "tool": "git",
        "argv": ["git", "rev-parse", PRODUCT + ":" + source["path"]],
        "cwd": str(ROOT),
        "exit_code": 0,
        "source_commit": PRODUCT,
        "source_path": source["path"],
        "stdout": source.get("git_blob_oid"),
        "purpose": "Record immutable historical source blob identity.",
    })

source_code_counts = {}
for crate in crates:
    package = crate["package_name"]
    manifest_path = crate["manifest_path"]
    rows = [item for item in items if item.get("manifest_path") == manifest_path]
    source_definitions = [item for item in rows if not item.get("kind", "").startswith("trait_impl")]
    source_code_counts[crate["manifest_path"]] = {
        "public_definitions": len(source_definitions),
        "trait_impl_contracts": sum(1 for item in rows if item.get("kind") == "trait_impl"),
        "trait_impl_members": sum(1 for item in rows if item.get("kind", "").startswith("trait_impl_") ),
        "direct_exports": sum(1 for export in exports if export.get("manifest_path") == manifest_path and export["kind"] == "public_definition"),
        "trait_impl_contract_exports": sum(1 for export in exports if export.get("manifest_path") == manifest_path and export["kind"] == "trait_impl_contract"),
        "trait_impl_member_exports": sum(1 for export in exports if export.get("manifest_path") == manifest_path and export["kind"] == "trait_impl_member"),
        "reexports": sum(1 for export in exports if export.get("manifest_path") == manifest_path and export["kind"] in ("reexport", "reexport_alias", "wildcard_reexport", "wildcard_member", "reachable_item")),
        "public_modules": sum(1 for export in exports if export.get("manifest_path") == manifest_path and export["kind"] == "public_module"),
    }
    crate["inventory_counts"] = source_code_counts[crate["manifest_path"]]


def public_export_key(workspace_id, package_id, public_path, definition_package_id, definition_owner, namespace):
    return workspace_id, package_id, public_path, definition_package_id, definition_owner, namespace


expected_public_binding_keys = set()
for item in items:
    if item.get("kind") in ("field", "tuple_field", "enum_variant"):
        continue
    definition_package_id = item.get("definition_package_id") or item.get("package_id")
    definition_owner = item.get("definition_owner") or item.get("current_qualified_symbol")
    for binding in item.get("public_bindings", []):
        expected_public_binding_keys.add(public_export_key(
            binding.get("workspace_id"), binding.get("package_id"), binding.get("qualified_path"),
            definition_package_id, definition_owner, item.get("namespace"),
        ))
actual_public_export_keys = {
    public_export_key(
        row["export_key"]["workspace_id"], row["export_key"]["package_id"], row["export_key"]["canonical_public_path"],
        row["export_key"]["definition_package_id"], row["export_key"]["definition_owner"], row["export_key"]["namespace"],
    )
    for row in exports
}
missing_public_binding_exports = sorted(expected_public_binding_keys - actual_public_export_keys)
if missing_public_binding_exports:
    raise RuntimeError("current public item bindings are missing export inventory rows; update the migration inventory before accepting T01")

source_records_by_path = {row["path"]: row for row in all_sources}
missing_current_source_bindings = []
for row in items + exports:
    source = source_records_by_path.get(row.get("source_path"))
    if not source or source.get("sha256") != row.get("source_sha256") or not row.get("line"):
        missing_current_source_bindings.append({"qualified_symbol": row.get("current_qualified_symbol"), "source_path": row.get("source_path"), "line": row.get("line")})
    for provenance in row.get("source_provenance", []):
        evidence = provenance.get("source", {})
        source = source_records_by_path.get(evidence.get("path"))
        if not source or source.get("sha256") != evidence.get("sha256") or not evidence.get("line"):
            missing_current_source_bindings.append({"qualified_symbol": row.get("current_qualified_symbol"), "source_path": evidence.get("path"), "line": evidence.get("line")})
if missing_current_source_bindings:
    raise RuntimeError("a current public source row is missing exact source path/line/hash evidence")

package_identity_keys = [(row["workspace_id"], row["package_id"], row["manifest_path"]) for row in crates]
if len(package_identity_keys) != len(set(package_identity_keys)) or len({row["package_id"] for row in crates}) != len(crates):
    raise RuntimeError("manifest package identity is not unique")
if len({row["export_id"] for row in exports}) != len(exports) or len({row["api_record_id"] for row in items}) != len(items):
    raise RuntimeError("API record IDs are not unique")
if any(row.get("definition_package_id") is not None for row in exports if row.get("definition_resolution") == "external_dependency"):
    raise RuntimeError("a third-party dependency was assigned an owned source package")
if any(not row.get("named_owner_task") or row.get("evidence_state") != "NOT VERIFIED" for row in requirements):
    raise RuntimeError("a requirement is missing its owner task or evidence state")
retired_requirements = [row for row in requirements if row.get("disposition") == "RETIRE"]
if any(not row.get("source_scoped_decision_id") or not row.get("decision_artifact", {}).get("sha256") or row.get("current_binding") for row in retired_requirements):
    raise RuntimeError("a retired API row is missing reviewed absence evidence")
if any(not row.get("source_api_identity", {}).get("path") or not row.get("source_api_identity", {}).get("sha256") or not row.get("cleanup_attachment_evidence", {}).get("sha256") for row in historical_api_dispositions):
    raise RuntimeError("a historical API disposition is missing its source or attachment evidence")

structural_audit = {
    "status": "PASS",
    "evidence_kind": "source, Cargo metadata, and migration record structure only",
    "functional_behavior": "NOT VERIFIED",
    "checks": {
        "manifest_package_identity_unique": len(package_identity_keys) == len(set(package_identity_keys)) and len({row["package_id"] for row in crates}) == len(crates),
        "public_item_binding_export_keys": len(expected_public_binding_keys),
        "missing_public_item_export_keys": len(missing_public_binding_exports),
        "export_ids_unique": len(exports),
        "item_record_ids_unique": len(items),
        "current_source_rows_exact_path_line_hash": len(items) + len(exports) + sum(len(row.get("source_provenance", [])) for row in exports),
        "historical_dispositions_with_source_and_evidence": len(historical_api_dispositions),
        "historical_source_commits_not_supplied_by_t37_record": sum(1 for row in historical_api_dispositions if not row.get("source_api_identity", {}).get("commit")),
        "retired_requirements_with_approved_absence_and_decision": len(retired_requirements),
        "third_party_exports_with_unowned_definition_package": sum(1 for row in exports if row.get("definition_resolution") == "external_dependency"),
    },
}

result = {
    "schema": "termrock.pr38.t01.fragment.v1",
    "task_id": "T01",
    "fragment_id": "foundation-api",
    "input_commit": INPUT,
    "input_tree": input_tree,
    "product_source_commit": PRODUCT,
    "metadata_evidence_source_commit": META_HEAD,
    "preflight_basis": {
        "t37_status": "ACCEPTED_PREFLIGHT",
        "accepted_candidate": T37_CANDIDATE,
        "normal_integration_commit": T37_INTEGRATION,
        "identical_tree": T37_TREE,
        "metadata_evidence_commit": META_HEAD,
        "metadata_evidence_tree": METADATA_TREE,
        "production_source_matches_t37_tree": True,
        "production_behavior": "NOT VERIFIED",
    },
    "scope": {
        "definition": "Foundation library packages and non-application support, tool, and consumer packages outside the component-family and facade scopes.",
        "foundation_crates": list(FOUNDATION),
        "support_manifest_paths": [record[0] for record in SUPPORT.values()],
        "excluded": ["termrock facade", "component-family crates", "showcase", "holla", "jackin-preview", "tablepro"],
        "app_package_prefixes_excluded": ["showcase", "holla", "jackin-preview", "tablepro"],
        "wid_policy": "This fragment assigns no W IDs. The component worker owns all W ID assignment.",
    },
    "metadata_workspaces": [
        {"workspace_id": workspace_identity(path), "workspace_root": normalized_workspace_root(path), "evidence_path": path, "source_commit": META_HEAD, "sha256": source_record(path, commit=META_HEAD, kind="cargo_metadata_evidence")["sha256"], "packages": len(json.loads((ROOT / path).read_text()).get("packages", []))}
        for path in META_FILES
    ],
    "crates": crates,
    "sources": all_sources,
    "exports": exports,
    "items": items,
    "cfg_provenance_records": [{"id": record_id, "provenance": cfg_provenance_records[record_id]} for record_id in sorted(cfg_provenance_records)],
    "requirements": requirements,
    "historical_api_dispositions": historical_api_dispositions,
    "performance_workloads": performance_workloads,
    "duplicate_relationships": duplicate_relationships,
    "open_requirements": open_requirements,
    "retired_testing_api_inventory": {
        "groups": removed["retired_testing_api_group_ids"],
        "review_decisions": decision_map,
        "current_export_symbol_matches": retired_current_matches,
        "accepted_compatibility_matches": sorted(set(retired_current_matches) & accepted_compatibility_matches),
        "unexpected_current_matches": unexpected_retired_current_matches,
        "normal_draw_scene_preserved": bool(draw_scene),
        "source_paths": removed["removed_source_items"],
        "limits": ["The removed helpers are historical migration requirements. They do not appear as current API items.", "T37 selected zero behavior cases. Product behavior remains NOT VERIFIED."],
    },
    "structural_inventory_gate": {
        "condition": "A current public source export missing from this inventory blocks T01 structural acceptance until its migration row is updated.",
        "evidence_kind": "metadata and source inventory only",
        "functional_test": False,
        "unreviewed_api_removal_allowed": False,
        "source_binding_reconciliation": {"expected_public_item_binding_keys": len(expected_public_binding_keys), "missing_keys": len(missing_public_binding_exports), "result": "PASS"},
        "retired_api_migration_reconciliation": {"retired_requirement_rows": len(retired_requirements), "rows_without_source_scoped_decision_or_absence": 0, "result": "PASS"},
    },
    "structural_audit": structural_audit,
    "commands": [
        {"tool": "python3", "argv": ["python3", "docs/implementation/pr38-final/results/t01-foundation-api/build_fragment.py"], "cwd": str(ROOT), "exit_code": 0, "source_path": "docs/implementation/pr38-final/results/t01-foundation-api/build_fragment.py", "source_sha256": sha((ROOT / "docs/implementation/pr38-final/results/t01-foundation-api/build_fragment.py").read_bytes()), "output_path": str(OUT.relative_to(ROOT)), "purpose": "Read current source, consume accepted metadata, and write the static API fragment."},
        *identity_commands,
        *metadata_commands,
        *historical_commands,
        *historical_oid_commands,
    ],
    "tool_evidence": {"cargo_metadata": "Reused accepted T37 metadata records. No Cargo command ran in this T01 fragment.", "functional_cases_selected": 0, "behavior_evidence": "NOT VERIFIED", "task_index_sha256": task_document_hash, "root_declared_normal_build_dependency_count": 20, "dependency_registry_or_functionality_proven": False},
    "limits": [
        "Cargo metadata came from the accepted T37 preflight source. T01 source files match the accepted integration tree for product paths.",
        "The static declaration scan records source-level public declarations. It does not resolve Rust macro expansion or compiler reachability.",
        "Feature conditions retain direct cfg attributes, enclosing module declaration alternatives, and Cargo metadata target features. Feature behavior remains unverified.",
        "Local wildcard re-exports expand source-visible members and preserve each wildcard declaration's provenance. External dependency wildcard members were not expanded.",
        "Third-party re-export owners use accepted dependency metadata and Cargo.lock package identity, version, checksum, requested features, and target cfg. Their external definitions and enabled feature closure were not inventoried.",
        "Four public API declarative macros were inspected. Three macro invocations were expanded into source-level API rows; Rust compiler expansion was not run.",
        "The root library's 20 declared normal and build dependency rows do not prove registry availability or functionality.",
        "T37 retained API clauses and open follow-ups remain NOT VERIFIED until approved cases run after T38.",
        "433 historical T37 source records do not supply a source commit. Their recorded source hash and decision artifact are preserved; no commit was inferred.",
        "Historical performance definitions are source evidence only. T33 measurements are NOT RECORDED.",
    ],
    "counts": {
        "crates": len(crates),
        "foundation_crates": len(FOUNDATION),
        "support_manifests": len(SUPPORT),
        "source_files_and_documents": len(all_sources),
        "public_items": len(items),
        "exports": len(exports),
        "requirements": len(requirements),
        "historical_api_dispositions": len(historical_api_dispositions),
        "open_requirements": len(open_requirements),
        "performance_records": len(performance_workloads),
        "retired_testing_api_current_matches": len(retired_current_matches),
        "unexpected_retired_testing_api_current_matches": len(unexpected_retired_current_matches),
        "direct_exports": sum(1 for row in exports if row["kind"] == "public_definition"),
        "trait_impl_contract_exports": sum(1 for row in exports if row["kind"] == "trait_impl_contract"),
        "trait_impl_member_exports": sum(1 for row in exports if row["kind"] == "trait_impl_member"),
        "public_module_exports": sum(1 for row in exports if row["kind"] == "public_module"),
        "reexport_index_rows": sum(1 for row in exports if row["kind"] in ("reexport", "reexport_alias", "wildcard_reexport", "wildcard_member", "reachable_item")),
        "trait_impl_contract_items": sum(1 for row in items if row["kind"] == "trait_impl"),
        "trait_impl_member_items": sum(1 for row in items if row["kind"].startswith("trait_impl_")),
    },
}
OUT.write_text(json.dumps(result, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
print(json.dumps(result["counts"], sort_keys=True))
