#!/usr/bin/env python3
"""Fail-closed, source-bound architecture checks for termrock acceptance."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
from typing import Any

HERE = Path(__file__).resolve().parent
POLICY_PATH = HERE / "acceptance_policy.json"
SOURCE_SUFFIXES = {".rs", ".inc", ".in", ".txt", ".ansi", ".ans", ".json", ".toml"}
SOURCE_ROOTS = ("src/",)
STAMP_RULES = (
    ("frame-marker", re.compile(r"(?i)(frame stamp|captured (?:terminal )?frame|approved (?:frame )?cells|snapshot trailer|captured screen)")),
    ("ansi-frame-replay", re.compile(r"(?i)(include_(?:str|bytes)!.*(?:snapshots?|baselines?)/.*\.(?:ansi|ans)|from_ansi|parse_ansi_frame)")),
    ("frame-stamp-type", re.compile(r"(?is)\b(?:struct|enum)\s+\w+\s*\{[^}]{0,800}\bx\s*:\s*u(?:16|32|size)\b[^}]{0,400}\by\s*:\s*u(?:16|32|size)\b[^}]{0,600}\btext\s*:\s*&?\s*'static\s+str")),
)
INCLUDE_RE = re.compile(r"\binclude(?:_str|_bytes)?!\s*\(\s*\"([^\"]+)\"\s*\)")
DYNAMIC_INCLUDE_RE = re.compile(r"\binclude(?:_str|_bytes)?!\s*\(")
PATH_MODULE_RE = re.compile(r"#\s*\[\s*path\s*=\s*\"([^\"]+)\"\s*\]")


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def run(args: list[str], cwd: Path) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(args, cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False, timeout=90)


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


def production_files(root: Path, package_root: str, tracked: list[str]) -> list[str]:
    prefix = package_root.rstrip("/") + "/"
    out = []
    for path in tracked:
        if path == prefix + "Cargo.toml" or path == prefix + "build.rs":
            out.append(path)
        elif path.startswith(prefix + "src/") and Path(path).suffix in SOURCE_SUFFIXES:
            out.append(path)
    source_dir = root / package_root / "src"
    if source_dir.exists():
        for path in source_dir.rglob("*"):
            if path.is_file() or path.is_symlink():
                out.append(path.relative_to(root).as_posix())
    return sorted(set(out))


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
        "packages": {"production_expected": policy["production_library_packages"], "production_observed": [], "missing_production": [], "applications_missing": []},
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
    by_id = {}
    for pkg in packages:
        by_name.setdefault(str(pkg.get("name", "")), []).append(pkg)
        by_id[str(pkg.get("id", ""))] = pkg
    expected = set(policy["production_library_packages"])
    observed = expected & set(by_name)
    report["packages"]["production_observed"] = sorted(observed)
    report["packages"]["missing_production"] = sorted(expected - observed)
    report["packages"]["duplicate_production"] = sorted(name for name in expected if len(by_name.get(name, [])) != 1 and name in by_name)

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
    forbidden_prefixes = tuple(policy["forbidden_application_dependency_prefixes"])
    production_pkgs: list[dict[str, Any]] = []
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
                direct_forbidden.setdefault(name, []).append({
                    "dependency": actual,
                    "alias": dep.get("rename"),
                    "target": dep.get("target"),
                    "kind": dep.get("kind") or "normal",
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

    for name in sorted(expected & set(by_name)):
        for pkg in by_name[name]:
            production_pkgs.append(pkg)

    source_records: list[dict[str, Any]] = []
    file_bytes: dict[str, bytes] = {}
    scan_pkgs = (
        [(pkg, "library") for pkg in production_pkgs]
        + [(pkg, "application") for pkg in application_pkgs]
        + [(pkg, "application_component") for pkg in application_component_pkgs]
    )
    for pkg, package_role in sorted(scan_pkgs, key=lambda pair: (str(pair[0].get("name", "")), pair[1])):
        manifest = Path(str(pkg.get("manifest_path", ""))).resolve()
        try:
            rel_manifest = manifest.relative_to(root).as_posix()
        except ValueError:
            report["errors"].append(f"package manifest is outside workspace: {pkg.get('name')} {manifest}")
            continue
        pkg_root = manifest.parent
        pkg_rel_root = pkg_root.relative_to(root).as_posix()
        files = production_files(root, pkg_rel_root, tracked)
        if rel_manifest not in files:
            files.append(rel_manifest)
        for rel in sorted(set(files)):
            path = root / rel
            try:
                resolved = path.resolve(strict=True)
                resolved.relative_to(root)
                data = path.read_bytes()
            except (OSError, ValueError) as exc:
                report["errors"].append(f"unreadable or out-of-root production input {rel}: {exc}")
                continue
            file_bytes[rel] = data
            source_records.append({"path": rel, "sha256": sha(data), "bytes": len(data), "package": pkg["name"], "role": package_role})
            if rel.endswith("build.rs"):
                report["unaccounted_inputs"].append(f"build-generated production inputs require independent accounting: {rel}")

            text = data.decode("utf-8", "replace")
            lines = text.splitlines()
            for match in DYNAMIC_INCLUDE_RE.finditer(text):
                if not INCLUDE_RE.match(text, match.start()):
                    line = text.count("\n", 0, match.start()) + 1
                    report["unaccounted_inputs"].append(f"dynamic include path cannot be enumerated: {rel}:{line}")
            for match in INCLUDE_RE.finditer(text):
                included = (path.parent / match.group(1)).resolve()
                try:
                    included_rel = included.relative_to(root).as_posix()
                    try:
                        inc_data = included.read_bytes()
                    except FileNotFoundError:
                        if included_rel not in tracked:
                            raise
                        # Sparse checkouts may omit a tracked include. Read its
                        # index object, preserving the exact checked-out input.
                        blob = run(["git", "show", f":{included_rel}"], root)
                        if blob.returncode:
                            raise OSError(blob.stderr.decode("utf-8", "replace").strip())
                        inc_data = blob.stdout
                    source_records.append({"path": included_rel, "sha256": sha(inc_data), "bytes": len(inc_data), "package": pkg["name"], "included_by": rel})
                    file_bytes[included_rel] = inc_data
                    if re.search(r"(?i)(?:snapshots?|baselines?)/.*\.(?:ansi|ans)$", match.group(1)):
                        line = text.count("\n", 0, match.start()) + 1
                        report["findings"].append(finding("ansi-frame-replay", rel, line, data, f"includes captured terminal asset {included_rel}"))
                except (OSError, ValueError) as exc:
                    line = text.count("\n", 0, match.start()) + 1
                    report["errors"].append(f"unreadable or out-of-root include from {rel}:{line}: {match.group(1)} ({exc})")

            for match in PATH_MODULE_RE.finditer(text):
                referenced = (path.parent / match.group(1)).resolve()
                try:
                    referenced_rel = referenced.relative_to(root).as_posix()
                    try:
                        ref_data = referenced.read_bytes()
                    except FileNotFoundError:
                        if referenced_rel not in tracked:
                            raise
                        blob = run(["git", "show", f":{referenced_rel}"], root)
                        if blob.returncode:
                            raise OSError(blob.stderr.decode("utf-8", "replace").strip())
                        ref_data = blob.stdout
                    source_records.append({"path": referenced_rel, "sha256": sha(ref_data), "bytes": len(ref_data), "package": pkg["name"], "included_by": rel})
                    file_bytes[referenced_rel] = ref_data
                except (OSError, ValueError) as exc:
                    line = text.count("\n", 0, match.start()) + 1
                    report["errors"].append(f"unreadable or out-of-root path module from {rel}:{line}: {match.group(1)} ({exc})")

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
        report["errors"], report["packages"]["missing_production"], report["packages"]["duplicate_production"],
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
