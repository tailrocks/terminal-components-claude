#!/usr/bin/env python3
"""Fail-closed, source-bound architecture checks for termrock acceptance."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
from typing import Any

HERE = Path(__file__).resolve().parent
POLICY_PATH = HERE / "acceptance_policy.json"
SOURCE_SUFFIXES = {".rs", ".inc", ".in", ".txt", ".ansi", ".ans", ".json", ".toml"}
STAMP_RULES = (
    ("frame-marker", re.compile(r"(?i)(frame stamp|captured (?:terminal )?frame|approved (?:frame )?cells|snapshot trailer|captured screen)")),
    ("ansi-frame-replay", re.compile(r"(?i)(include_(?:str|bytes)!.*(?:snapshots?|baselines?)/.*\.(?:ansi|ans)|from_ansi|parse_ansi_frame)")),
    ("frame-stamp-type", re.compile(r"(?is)\b(?:struct|enum)\s+\w+\s*\{[^}]{0,800}\bx\s*:\s*u(?:16|32|size)\b[^}]{0,400}\by\s*:\s*u(?:16|32|size)\b[^}]{0,600}\btext\s*:\s*&?\s*'static\s+str")),
)
DECLARED_MODULE_RE = re.compile(r"\bmod\s+(?:r#)?([A-Za-z_][A-Za-z0-9_]*)\s*;")


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def run(args: list[str], cwd: Path) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(args, cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False, timeout=90)


def skip_rust_trivia(text: str, index: int) -> int | None:
    while index < len(text):
        if text[index].isspace():
            index += 1
        elif text.startswith("//", index):
            end = text.find("\n", index + 2)
            index = len(text) if end < 0 else end + 1
        elif text.startswith("/*", index):
            depth = 1
            index += 2
            while index < len(text) and depth:
                if text.startswith("/*", index):
                    depth += 1
                    index += 2
                elif text.startswith("*/", index):
                    depth -= 1
                    index += 2
                else:
                    index += 1
            if depth:
                return None
        else:
            break
    return index


def rust_char_literal_end(text: str, index: int) -> int | None:
    """Return the end of a char/byte-char literal, leaving lifetimes untouched."""
    quote = index + 1 if text.startswith("b'", index) else index
    if quote >= len(text) or text[quote] != "'" or quote + 1 >= len(text):
        return None
    cursor = quote + 1
    if text[cursor] in "\r\n":
        return None
    if text[cursor] == "\\":
        cursor += 1
        if cursor >= len(text):
            return None
        if text[cursor] == "u" and cursor + 1 < len(text) and text[cursor + 1] == "{":
            close = text.find("}", cursor + 2)
            if close < 0:
                return None
            cursor = close + 1
        elif text[cursor] == "x":
            cursor += 3
        else:
            cursor += 1
    else:
        cursor += 1
    return cursor + 1 if cursor < len(text) and text[cursor] == "'" else None


def rust_literal_end(text: str, index: int) -> tuple[int | None, bool]:
    """Skip Rust string literals; flag malformed forms instead of hiding suffixes."""
    char_end = rust_char_literal_end(text, index)
    if char_end is not None:
        return char_end, False
    raw_start = index
    if text.startswith("br", index):
        raw_start += 2
    elif text.startswith("r", index):
        raw_start += 1
    else:
        raw_start = -1
    if raw_start >= 0:
        marker = raw_start
        while marker < len(text) and text[marker] == "#":
            marker += 1
        if marker < len(text) and text[marker] == '"':
            hashes = marker - raw_start
            terminator = '"' + "#" * hashes
            end = text.find(terminator, marker + 1)
            return (None, True) if end < 0 else (end + len(terminator), False)
    quote_start = index + 1 if text.startswith("b\"", index) else index
    if quote_start < len(text) and text[quote_start] == '"':
        cursor = quote_start + 1
        while cursor < len(text):
            if text[cursor] == "\\":
                cursor += 2
            elif text[cursor] == '"':
                return cursor + 1, False
            else:
                cursor += 1
        return None, True
    return None, False


def include_invocations(text: str) -> list[tuple[str, int, str | None]]:
    """Extract simple literal include calls; report unsupported calls to caller."""
    found: list[tuple[str, int, str | None]] = []
    index = 0
    pairs = {"(": ")", "{": "}", "[": "]"}
    while index < len(text):
        if text.startswith("//", index) or text.startswith("/*", index):
            next_index = skip_rust_trivia(text, index)
            if next_index is None:
                found.append(("lexer", index, None))
                break
            index = next_index
            continue
        literal_end, lexical_uncertainty = rust_literal_end(text, index)
        if lexical_uncertainty:
            found.append(("lexer", index, None))
            break
        if literal_end is not None:
            index = literal_end
            continue
        char = text[index]
        if char.isalpha() or char == "_":
            end = index + 1
            while end < len(text) and (text[end].isalnum() or text[end] == "_"):
                end += 1
            name = text[index:end]
            if name not in {"include", "include_str", "include_bytes"}:
                index = end
                continue
            bang = skip_rust_trivia(text, end)
            if bang is None:
                found.append((name, index, None))
                index = end
                continue
            if bang >= len(text) or text[bang] != "!":
                index = end
                continue
            opening = skip_rust_trivia(text, bang + 1)
            if opening is None or opening >= len(text) or text[opening] not in pairs:
                found.append((name, index, None))
                index = bang + 1
                continue
            cursor = skip_rust_trivia(text, opening + 1)
            literal: str | None = None
            escaped = False
            if cursor is not None and cursor < len(text) and text[cursor] == '"':
                cursor += 1
                chars: list[str] = []
                while cursor < len(text) and text[cursor] != '"':
                    if text[cursor] in "\r\n":
                        break
                    if text[cursor] == "\\":
                        escaped = True
                        if cursor + 1 >= len(text):
                            break
                        chars.append(text[cursor + 1])
                        cursor += 2
                    else:
                        chars.append(text[cursor])
                        cursor += 1
                if cursor < len(text) and text[cursor] == '"':
                    after = skip_rust_trivia(text, cursor + 1)
                    if after is not None and after < len(text) and text[after] == pairs[text[opening]] and not escaped:
                        literal = "".join(chars)
            found.append((name, index, literal))
            index = end
            continue
        index += 1
    return found


def path_attributes(text: str) -> list[tuple[str, int, str | None]]:
    """Find `path` meta items, including nested cfg_attr forms."""
    found: list[tuple[str, int, str | None]] = []
    index = 0
    while index < len(text):
        if text.startswith("//", index) or text.startswith("/*", index):
            next_index = skip_rust_trivia(text, index)
            if next_index is None:
                found.append(("lexer", index, None))
                break
            index = next_index
            continue
        literal_end, uncertain = rust_literal_end(text, index)
        if uncertain:
            found.append(("lexer", index, None))
            break
        if literal_end is not None:
            index = literal_end
            continue
        if text[index] != "#":
            index += 1
            continue
        opening = skip_rust_trivia(text, index + 1)
        if opening is None:
            found.append(("lexer", index, None))
            break
        if opening < len(text) and text[opening] == "!":
            opening = skip_rust_trivia(text, opening + 1)
            if opening is None:
                found.append(("lexer", index, None))
                break
        if opening >= len(text) or text[opening] != "[":
            index += 1
            continue
        close = rust_group_end(text, opening)
        if close is None:
            found.append(("path-module", index, None))
            break
        found.extend(path_meta_items(text, opening + 1, close))
        index = close + 1
    return found


def rust_group_end(text: str, opening: int) -> int | None:
    pairs = {"(": ")", "[": "]", "{": "}"}
    if opening >= len(text) or text[opening] not in pairs:
        return None
    stack = [pairs[text[opening]]]
    index = opening + 1
    while index < len(text):
        if text.startswith("//", index) or text.startswith("/*", index):
            next_index = skip_rust_trivia(text, index)
            if next_index is None:
                return None
            index = next_index
            continue
        literal_end, uncertain = rust_literal_end(text, index)
        if uncertain:
            return None
        if literal_end is not None:
            index = literal_end
            continue
        char = text[index]
        if char in pairs:
            stack.append(pairs[char])
        elif char in ")]}" :
            if not stack or char != stack[-1]:
                return None
            stack.pop()
            if not stack:
                return index
        index += 1
    return None


def simple_path_literal(text: str, index: int) -> tuple[str | None, int]:
    """Read a plain or raw Rust string path; unsupported escape forms stay unresolved."""
    if index < len(text) and text[index] == '"':
        end = index + 1
        escaped = False
        while end < len(text):
            if text[end] == "\\":
                escaped = True
                end += 2
            elif text[end] == '"':
                return (None if escaped else text[index + 1 : end]), end + 1
            elif text[end] in "\r\n":
                return None, end
            else:
                end += 1
        return None, len(text)
    if index < len(text) and text[index] == "r":
        marker = index + 1
        while marker < len(text) and text[marker] == "#":
            marker += 1
        if marker < len(text) and text[marker] == '"':
            hashes = marker - index - 1
            terminator = '"' + "#" * hashes
            end = text.find(terminator, marker + 1)
            if end < 0:
                return None, len(text)
            return text[marker + 1 : end], end + len(terminator)
    return None, index


def path_meta_items(text: str, start: int, end: int) -> list[tuple[str, int, str | None]]:
    found: list[tuple[str, int, str | None]] = []
    index = start
    while index < end:
        if text.startswith("//", index) or text.startswith("/*", index):
            next_index = skip_rust_trivia(text, index)
            if next_index is None:
                found.append(("lexer", index, None))
                break
            index = next_index
            continue
        literal_end, uncertain = rust_literal_end(text, index)
        if uncertain:
            found.append(("lexer", index, None))
            break
        if literal_end is not None:
            index = literal_end
            continue
        if text[index].isalpha() or text[index] == "_":
            token_end = index + 1
            while token_end < end and (text[token_end].isalnum() or text[token_end] == "_"):
                token_end += 1
            if text[index:token_end] == "path":
                equals = skip_rust_trivia(text, token_end)
                if equals is not None and equals < end and text[equals] == "=":
                    value = skip_rust_trivia(text, equals + 1)
                    if value is None:
                        found.append(("path-module", index, None))
                        break
                    literal, after = simple_path_literal(text, value)
                    found.append(("path-module", index, literal))
                    index = max(token_end, after)
                    continue
                found.append(("path-module", index, None))
            index = token_end
            continue
        index += 1
    return found


def git_files(root: Path) -> tuple[list[str] | None, str | None]:
    proc = run(["git", "ls-files", "-z"], root)
    if proc.returncode:
        return None, proc.stderr.decode("utf-8", "replace")
    return [p.decode("utf-8", "surrogateescape") for p in proc.stdout.split(b"\0") if p], None


def load_metadata(root: Path) -> tuple[dict[str, Any] | None, str | None]:
    proc = run(["cargo", "metadata", "--locked", "--format-version", "1", "--no-deps"], root)
    if proc.returncode:
        return None, proc.stderr.decode("utf-8", "replace")
    try:
        value = json.loads(proc.stdout)
        if not isinstance(value, dict) or not isinstance(value.get("packages"), list):
            return None, "cargo metadata returned malformed package inventory"
        return value, None
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        return None, f"cannot parse cargo metadata: {exc}"


def package_name(dep: dict[str, Any]) -> str:
    # `name` is the resolved package name; `rename` is only the local alias.
    return str(dep.get("package") or dep.get("name") or "")


def dependency_location(root: Path, pkg: dict[str, Any], dep: dict[str, Any]) -> tuple[str, int, str] | None:
    manifest = Path(str(pkg.get("manifest_path", ""))).resolve()
    try:
        rel = manifest.relative_to(root).as_posix()
        data = manifest.read_bytes()
        text = data.decode("utf-8")
    except (OSError, ValueError, UnicodeDecodeError):
        return None
    key = str(dep.get("rename") or dep.get("name") or "")
    pattern = re.compile(rf"^\s*{re.escape(key)}(?:\s*=|\.)")
    for number, line in enumerate(text.splitlines(), 1):
        if pattern.search(line):
            return rel, number, sha(data)
    return None


def production_files(
    root: Path,
    package_root: str,
    tracked: list[str],
    target_paths: list[str],
    target_directory: Path,
) -> tuple[list[str], list[str]]:
    prefix = package_root.rstrip("/") + "/"
    out = list(target_paths)
    errors: list[str] = []
    for path in tracked:
        if path == prefix + "Cargo.toml" or path == prefix + "build.rs":
            out.append(path)
        elif path.startswith(prefix) and Path(path).suffix == ".rs":
            out.append(path)
        elif path.startswith(prefix + "src/") and Path(path).suffix in SOURCE_SUFFIXES:
            out.append(path)
    source_dir = root / package_root
    if source_dir.exists():
        excluded_dirs = {target_directory.resolve(strict=False), (root / ".git").resolve(strict=False)}
        pending = [(source_dir, source_dir.resolve(strict=True), (source_dir.resolve(strict=True),))]
        while pending:
            logical_dir, actual_dir, ancestors = pending.pop()
            if actual_dir in excluded_dirs:
                continue
            try:
                entries = sorted(actual_dir.iterdir(), key=lambda item: item.name)
            except OSError as exc:
                errors.append(f"unreadable production source directory {logical_dir}: {exc}")
                continue
            for entry in entries:
                logical_path = logical_dir / entry.name
                try:
                    resolved = logical_path.resolve(strict=True)
                    is_dir = logical_path.is_dir()
                    if is_dir and resolved in excluded_dirs:
                        continue
                    resolved.relative_to(root)
                except (OSError, ValueError) as exc:
                    errors.append(f"unreadable or out-of-root production input {logical_path.relative_to(root)}: {exc}")
                    continue
                if is_dir:
                    if resolved in ancestors:
                        errors.append(f"recursive production source directory symlink cycle: {logical_path.relative_to(root)}")
                        continue
                    pending.append((logical_path, resolved, (*ancestors, resolved)))
                else:
                    rel_parts = logical_path.relative_to(source_dir).parts
                    rel = logical_path.relative_to(root).as_posix()
                    if logical_path.suffix == ".rs" or ("src" in rel_parts and logical_path.suffix in SOURCE_SUFFIXES):
                        out.append(rel)
    return sorted(set(out)), errors


def read_source(root: Path, rel: str) -> bytes:
    path = root / rel
    resolved = path.resolve(strict=False)
    resolved.relative_to(root)
    # The live filesystem is the Cargo input. Git's index may contain stale
    # bytes after a generated or included source file has been removed.
    return path.read_bytes()


def target_directory_module_aliases(source_roots: list[Path], target_directory: Path) -> set[str]:
    """Find source-tree symlink names that point into Cargo build output.

    Do not follow directory symlinks while walking: target directories can be
    large. Matching module names anywhere beneath package or actual Cargo
    target roots is deliberately conservative because lexical scanning does
    not model nested inline-module scopes.
    """
    aliases: set[str] = set()
    target_directory = target_directory.resolve(strict=False)
    seen_roots: set[Path] = set()
    for source_root in source_roots:
        source_root = source_root.resolve(strict=False)
        if source_root in seen_roots or not source_root.is_dir():
            continue
        try:
            source_root.relative_to(target_directory)
        except ValueError:
            pass
        else:
            continue
        seen_roots.add(source_root)
        for current, directories, _files in os.walk(source_root, followlinks=False):
            current_path = Path(current)
            try:
                current_path.resolve(strict=True).relative_to(target_directory)
            except (OSError, ValueError):
                pass
            else:
                directories[:] = []
                continue
            for name in directories:
                candidate = current_path / name
                if not candidate.is_symlink():
                    continue
                try:
                    resolved = candidate.resolve(strict=True)
                    resolved.relative_to(target_directory)
                except (OSError, ValueError):
                    continue
                aliases.add(name)
    return aliases


def finding(rule: str, rel: str, line: int, content: bytes, detail: str) -> dict[str, Any]:
    return {"rule": rule, "file": rel, "line": line, "detail": detail, "sha256": sha(content)}


def check(root: Path) -> dict[str, Any]:
    root = root.resolve()
    policy = json.loads(POLICY_PATH.read_text(encoding="utf-8"))
    policy_hash = sha(POLICY_PATH.read_bytes())
    report: dict[str, Any] = {
        "schema": "termrock-architecture-check/v1",
        "revision": None,
        "policy_sha256": policy_hash,
        "checker_sha256": sha(Path(__file__).read_bytes()),
        "source_manifest_sha256": None,
        "architecture_checks_passed": False,
        "acceptance_complete": False,
        "packages": {"production_expected": policy["production_library_packages"], "production_observed": [], "missing_production": [], "production_nonlibrary": [], "applications_missing": [], "duplicate_workspace_names": []},
        "application_shared_closure_missing": [],
        "dependency_violations": [],
        "source_inputs": [],
        "findings": [],
        "unaccounted_inputs": [],
        "unresolved": list(policy["unresolved_acceptance_requirements"]),
        "errors": [],
    }

    git = run(["git", "rev-parse", "HEAD"], root)
    if git.returncode == 0:
        report["revision"] = git.stdout.decode("ascii", "replace").strip()
    else:
        report["errors"].append("repository revision unavailable: " + git.stderr.decode("utf-8", "replace").strip())

    metadata, metadata_error = load_metadata(root)
    tracked, tracked_error = git_files(root)
    if metadata_error:
        report["errors"].append("locked cargo metadata failed: " + metadata_error.strip())
    if tracked_error:
        report["errors"].append("git tracked-source inventory failed: " + tracked_error.strip())
    if metadata is None or tracked is None:
        return report

    workspace_root = Path(str(metadata.get("workspace_root", ""))).resolve()
    if workspace_root != root:
        report["errors"].append(f"cargo workspace root escapes or differs from requested root: {workspace_root}")
    packages = metadata["packages"]
    by_name: dict[str, list[dict[str, Any]]] = {}
    for pkg in packages:
        by_name.setdefault(str(pkg.get("name", "")), []).append(pkg)
    report["packages"]["duplicate_workspace_names"] = sorted(name for name, members in by_name.items() if len(members) > 1)
    if report["packages"]["duplicate_workspace_names"]:
        report["errors"].append("workspace package names are ambiguous for locked graph resolution: " + ", ".join(report["packages"]["duplicate_workspace_names"]))
    expected = set(policy["production_library_packages"])
    observed = expected & set(by_name)
    report["packages"]["production_observed"] = sorted(observed)
    report["packages"]["missing_production"] = sorted(expected - observed)
    report["packages"]["duplicate_production"] = sorted(name for name in expected if len(by_name.get(name, [])) != 1 and name in by_name)
    report["packages"]["production_nonlibrary"] = sorted(
        name for name in expected & set(by_name)
        if not any(set(target.get("kind", [])) & {"lib", "proc-macro"} for pkg in by_name[name] for target in pkg.get("targets", []))
    )

    found_bins: set[str] = set()
    for pkg in packages:
        for target in pkg.get("targets", []):
            kinds = target.get("kind", [])
            if "bin" in kinds and target.get("name") in policy["application_binary_targets"]:
                found_bins.add(target["name"])
    report["packages"]["applications_missing"] = sorted(set(policy["application_binary_targets"]) - found_bins)
    report["packages"]["duplicate_application_targets"] = sorted(
        name for name in policy["application_binary_targets"]
        if sum(1 for pkg in packages for target in pkg.get("targets", []) if "bin" in target.get("kind", []) and target.get("name") == name) > 1
    )

    forbidden_exact = set(policy["forbidden_production_dependency_names"])
    forbidden_exact.update(policy["application_binary_targets"])
    forbidden_prefixes = tuple(policy["forbidden_application_dependency_prefixes"])
    application_pkgs: list[dict[str, Any]] = []
    target_to_pkg: dict[str, dict[str, Any]] = {}
    for pkg in packages:
        for target in pkg.get("targets", []):
            if "bin" in target.get("kind", []) and target.get("name") in policy["application_binary_targets"]:
                target_to_pkg[str(target["name"])] = pkg
    application_pkgs = [target_to_pkg[name] for name in sorted(target_to_pkg)]
    app_prefixes = tuple(policy["application_package_prefixes"])
    application_component_pkgs = [
        pkg for pkg in packages
        if str(pkg.get("name", "")).startswith(app_prefixes)
        and pkg not in application_pkgs
    ]

    # Traverse every non-dev workspace edge from each public library role.
    # Cargo's package dependency inventory includes target-specific edges.
    workspace_by_name = {str(pkg.get("name", "")): pkg for pkg in packages}
    dependency_graph: dict[str, list[str]] = {}
    direct_forbidden: dict[str, list[dict[str, Any]]] = {}
    for pkg in packages:
        name = str(pkg.get("name", ""))
        dependency_graph[name] = []
        for dep in pkg.get("dependencies", []):
            if dep.get("kind") == "dev":
                continue
            actual = package_name(dep)
            if actual in workspace_by_name:
                dependency_graph[name].append(actual)
            if actual in forbidden_exact or actual.startswith(forbidden_prefixes):
                location = dependency_location(root, pkg, dep)
                if location is None:
                    report["errors"].append(f"cannot bind Cargo dependency metadata to manifest source: {name} -> {actual}")
                    location_fields = {"file": None, "line": None, "sha256": None}
                else:
                    rel, number, digest = location
                    location_fields = {"file": rel, "line": number, "sha256": digest}
                direct_forbidden.setdefault(name, []).append({
                    "dependency": actual,
                    "alias": dep.get("rename"),
                    "target": dep.get("target"),
                    "kind": dep.get("kind") or "normal",
                    **location_fields,
                })

    for name in sorted(expected & set(by_name)):
        seen: set[str] = set()
        pending = [name]
        while pending:
            node = pending.pop()
            if node in seen:
                continue
            seen.add(node)
            for edge in direct_forbidden.get(node, []):
                report["dependency_violations"].append({"from": name, **edge, "via": node})
            pending.extend(dependency_graph.get(node, []))

    for target_name, app_pkg in sorted(target_to_pkg.items()):
        start = str(app_pkg.get("name", ""))
        seen: set[str] = set()
        pending = [start]
        while pending:
            node = pending.pop()
            if node in seen:
                continue
            seen.add(node)
            pending.extend(dependency_graph.get(node, []))
        if not seen.intersection(expected):
            report["application_shared_closure_missing"].append(target_name)

    production_closure = set(expected & set(by_name))
    production_closure.update(str(pkg.get("name", "")) for pkg in application_pkgs)
    pending_closure = list(production_closure)
    while pending_closure:
        node = pending_closure.pop()
        for dependency in dependency_graph.get(node, []):
            if dependency not in production_closure:
                production_closure.add(dependency)
                pending_closure.append(dependency)

    source_records: list[dict[str, Any]] = []
    file_bytes: dict[str, bytes] = {}
    pending_sources: list[tuple[dict[str, Any], str, str, bytes, tuple[str, ...]]] = []
    target_module_aliases_by_package: dict[str, set[str]] = {}
    scan_names = production_closure | {str(pkg.get("name", "")) for pkg in application_component_pkgs}
    scan_pkgs = []
    for name in sorted(scan_names):
        role = "library" if name in expected else "application_component" if name.startswith(app_prefixes) else "production_dependency"
        scan_pkgs.extend((pkg, role) for pkg in by_name.get(name, []))
    for pkg, package_role in sorted(scan_pkgs, key=lambda pair: (str(pair[0].get("name", "")), pair[1])):
        manifest = Path(str(pkg.get("manifest_path", ""))).resolve()
        try:
            rel_manifest = manifest.relative_to(root).as_posix()
        except ValueError:
            report["errors"].append(f"package manifest is outside workspace: {pkg.get('name')} {manifest}")
            continue
        pkg_root = manifest.parent
        pkg_rel_root = pkg_root.relative_to(root).as_posix()
        target_paths: list[str] = []
        target_source_roots = [pkg_root]
        for target in pkg.get("targets", []):
            if not set(target.get("kind", [])) & {"lib", "bin", "proc-macro", "custom-build"}:
                continue
            target_source = Path(str(target.get("src_path", ""))).resolve(strict=False)
            try:
                target_rel = target_source.relative_to(root).as_posix()
            except ValueError:
                report["errors"].append(f"production target source is outside workspace: {pkg.get('name')}:{target.get('name')} {target_source}")
                continue
            target_paths.append(target_rel)
            try:
                target_source.relative_to(pkg_root)
            except ValueError:
                report["unaccounted_inputs"].append(
                    f"production Cargo target source is outside its package; module tree is not fully accounted: {target_rel}"
                )
            target_source_roots.append(target_source.parent)
            if "custom-build" in target.get("kind", []):
                report["unaccounted_inputs"].append(f"build-generated production inputs require independent accounting: {target_rel}")
        target_module_aliases_by_package[str(pkg.get("id", pkg.get("name", "")))] = target_directory_module_aliases(
            target_source_roots,
            Path(str(metadata.get("target_directory", root / "target"))),
        )
        files, scan_errors = production_files(
            root,
            pkg_rel_root,
            tracked,
            target_paths,
            Path(str(metadata.get("target_directory", root / "target"))),
        )
        report["errors"].extend(scan_errors)
        if rel_manifest not in files:
            files.append(rel_manifest)
        for rel in sorted(set(files)):
            path = root / rel
            try:
                data = read_source(root, rel)
            except (OSError, ValueError) as exc:
                report["errors"].append(f"unreadable or out-of-root production input {rel}: {exc}")
                continue
            file_bytes[rel] = data
            source_records.append({"path": rel, "sha256": sha(data), "bytes": len(data), "package": pkg["name"], "role": package_role})
            pending_sources.append((pkg, package_role, rel, data, ()))

    visited_sources: set[tuple[str, str]] = set()
    while pending_sources:
        pkg, package_role, rel, data, ancestors = pending_sources.pop()
        key = (str(pkg.get("id", pkg.get("name", ""))), rel)
        if rel in ancestors:
            report["errors"].append(f"recursive production source include cycle: {' -> '.join((*ancestors, rel))}")
            continue
        if key in visited_sources:
            continue
        visited_sources.add(key)
        text = data.decode("utf-8", "replace")
        references = include_invocations(text)
        references.extend(path_attributes(text))
        module_aliases = target_module_aliases_by_package.get(str(pkg.get("id", pkg.get("name", ""))), set())
        # Ordinary `mod name;` declarations also select production inputs.
        # Do not traverse Cargo's potentially huge build directory through a
        # source symlink; report this selected input as unresolved instead.
        module_base = (root / rel).parent
        if Path(rel).name not in {"lib.rs", "main.rs", "mod.rs"}:
            module_base /= Path(rel).stem
        target_root = Path(str(metadata.get("target_directory", root / "target"))).resolve(strict=False)
        for match in DECLARED_MODULE_RE.finditer(text):
            name = match.group(1)
            if name in module_aliases:
                line = text.count("\n", 0, match.start()) + 1
                report["unaccounted_inputs"].append(
                    f"production module may resolve through a Cargo target-directory alias: {rel}:{line} -> {name}"
                )
            candidates = (module_base / f"{name}.rs", module_base / name / "mod.rs")
            for candidate in candidates:
                try:
                    resolved = candidate.resolve(strict=True)
                    resolved.relative_to(target_root)
                except FileNotFoundError:
                    continue
                except ValueError:
                    continue
                line = text.count("\n", 0, match.start()) + 1
                report["unaccounted_inputs"].append(
                    f"production module resolves inside Cargo target directory: {rel}:{line} -> {candidate.relative_to(root)}"
                )
                break
        for kind, offset, literal in references:
            line = text.count("\n", 0, offset) + 1
            if literal is None:
                if kind == "lexer":
                    report["unaccounted_inputs"].append(f"Rust lexical uncertainty prevents complete source scan: {rel}:{line}")
                elif kind == "path-module":
                    report["unaccounted_inputs"].append(
                        f"path attribute cannot be statically enumerated or syntax is unsupported: {rel}:{line}"
                    )
                else:
                    report["unaccounted_inputs"].append(
                        f"dynamic include path cannot be enumerated or invocation syntax is unsupported: {rel}:{line} ({kind})"
                    )
                continue
            referenced = (root / rel).parent / literal
            resolved = referenced.resolve(strict=False)
            if kind in {"include", "path-module"}:
                package_root = Path(str(pkg.get("manifest_path", ""))).resolve().parent
                try:
                    resolved.relative_to(package_root)
                except ValueError:
                    report["unaccounted_inputs"].append(
                        f"Rust source input is outside its package; module tree is not fully accounted: {rel}:{line} -> {literal}"
                    )
            target_root = Path(str(metadata.get("target_directory", root / "target"))).resolve(strict=False)
            if kind == "path-module":
                try:
                    resolved.relative_to(target_root)
                except ValueError:
                    pass
                else:
                    report["unaccounted_inputs"].append(
                        f"path module resolves inside Cargo target directory: {rel}:{line} -> {literal}"
                    )
            try:
                referenced_rel = resolved.relative_to(root).as_posix()
                ref_data = read_source(root, referenced_rel)
            except (OSError, ValueError) as exc:
                report["errors"].append(f"unreadable or out-of-root {kind} from {rel}:{line}: {literal} ({exc})")
                continue
            source_records.append({"path": referenced_rel, "sha256": sha(ref_data), "bytes": len(ref_data), "package": pkg["name"], "role": package_role, "included_by": rel})
            file_bytes[referenced_rel] = ref_data
            if kind in {"include", "include_str", "include_bytes"} and re.search(r"(?i)(?:snapshots?|baselines?)/.*\.(?:ansi|ans)$", literal):
                report["findings"].append(finding("ansi-frame-replay", rel, line, data, f"includes captured terminal asset {referenced_rel}"))
            if kind in {"include", "path-module"}:
                pending_sources.append((pkg, package_role, referenced_rel, ref_data, (*ancestors, rel)))

    for rel, data in sorted(file_bytes.items()):
        text = data.decode("utf-8", "replace")
        lines = text.splitlines()
        for index, line in enumerate(lines, 1):
            for rule, pattern in STAMP_RULES[:2]:
                if pattern.search(line):
                    report["findings"].append(finding(rule, rel, index, data, line.strip()[:240]))
        for rule, pattern in STAMP_RULES[2:]:
            match = pattern.search(text)
            if match:
                line = text.count("\n", 0, match.start()) + 1
                report["findings"].append(finding(rule, rel, line, data, "coordinate-based static text/style run type"))

    for rel in ("Cargo.toml", "Cargo.lock"):
        path = root / rel
        if not path.is_file():
            report["errors"].append(f"required workspace input missing: {rel}")
            continue
        try:
            data = path.read_bytes()
            source_records.append({"path": rel, "sha256": sha(data), "bytes": len(data), "package": "<workspace>", "role": "workspace"})
        except OSError as exc:
            report["errors"].append(f"unreadable workspace input {rel}: {exc}")

    # Stable unique inventory: duplicate includes still retain their owning edge.
    source_records.sort(key=lambda row: (row["path"], row.get("included_by", ""), row["package"]))
    report["source_inputs"] = source_records
    manifest_payload = json.dumps(source_records, sort_keys=True, separators=(",", ":")).encode()
    report["source_manifest_sha256"] = sha(manifest_payload)
    report["findings"].sort(key=lambda item: (item["file"], item["line"], item["rule"]))
    report["dependency_violations"].sort(key=lambda item: (item["from"], item["dependency"], item["target"] or ""))
    report["unresolved"] = sorted(set(report["unresolved"]))
    report["unaccounted_inputs"] = sorted(set(report["unaccounted_inputs"]))
    report["errors"] = sorted(set(report["errors"]))
    report["architecture_checks_passed"] = not any((
        report["errors"], report["packages"]["missing_production"], report["packages"]["duplicate_production"], report["packages"]["production_nonlibrary"],
        report["packages"]["applications_missing"], report["packages"]["duplicate_application_targets"],
        report["application_shared_closure_missing"], report["dependency_violations"], report["findings"],
        report["unaccounted_inputs"],
    ))
    # This bounded checker cannot attest semantic architecture, workflow coverage,
    # exact paired application behavior, or independent review.
    report["acceptance_complete"] = False
    return report


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path.cwd())
    parser.add_argument("--output", type=Path, help="write JSON report here; stdout otherwise")
    args = parser.parse_args()
    result = check(args.root)
    encoded = json.dumps(result, indent=2, sort_keys=True) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded, encoding="utf-8")
    else:
        sys.stdout.write(encoded)
    return 0 if result["architecture_checks_passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
