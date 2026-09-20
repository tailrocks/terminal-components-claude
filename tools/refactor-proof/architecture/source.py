"""Actual-source profile architecture checks."""

from __future__ import annotations

import base64
import json
import re
from pathlib import Path
from typing import Any

from ..runner.context import Reject
from .broker import validate_broker_observation


def _decode(record: dict[str, Any], stream: str) -> bytes:
    return base64.b64decode(record[stream])


def _compiler_codes(record: dict[str, Any]) -> list[str]:
    codes: list[str] = []
    for line in _decode(record, "stderr").splitlines():
        try:
            value = json.loads(line)
        except json.JSONDecodeError:
            continue
        if value.get("level") == "error" and value.get("code"):
            codes.append(value["code"]["code"])
    return codes


def _read_source_texts(profile: dict[str, Any]) -> dict[str, str]:
    directory = profile.get("source_directory")
    if not directory:
        return {}
    import hashlib

    sources: dict[str, str] = {}
    for path, digest in profile.get("sources", {}).items():
        data = (Path(directory) / path).read_bytes()
        if hashlib.sha256(data).hexdigest() != digest:
            raise Reject("ARCHITECTURE")
        if path.endswith((".rs", ".toml", ".md", ".txt")):
            sources[path] = data.decode()
    return sources


def _consumers(files: dict[str, bytes]) -> list[tuple[str, bytes]]:
    result: list[tuple[str, bytes]] = []
    for path, data in sorted(files.items()):
        if path.endswith(".rs"):
            result.append((path, data))
        elif path.endswith(".md"):
            blocks = re.findall(rb"(?m)^```rust[^\n]*\n(.*?)^```\s*$", data, flags=re.S)
            if not blocks:
                raise Reject("ARCHITECTURE")
            result.extend((f"{path}#{index}", block) for index, block in enumerate(blocks))
    if not result:
        raise Reject("ARCHITECTURE")
    return result


def _graph_paths(metadata: dict[str, Any], root_name: str, target_name: str, *, ids: bool = False) -> list[list[Any]]:
    by_id = {package["id"]: package for package in metadata["packages"]}
    nodes = {node["id"]: node for node in metadata["resolve"]["nodes"]}
    roots = [key for key, value in by_id.items() if value["name"] == root_name]
    if len(roots) != 1:
        raise Reject("ARCHITECTURE")
    paths: list[list[Any]] = []
    pending = [(roots[0], [])]
    while pending:
        current, visited = pending.pop()
        if current in visited:
            continue
        path = visited + [current]
        if by_id[current]["name"] == target_name:
            paths.append(path if ids else [by_id[key]["name"] for key in path])
        for dependency in nodes[current]["deps"]:
            if any(kind["kind"] in (None, "build") for kind in dependency["dep_kinds"]):
                pending.append((dependency["pkg"], path))
    return paths


def _graph_violations(metadata: dict[str, Any], contract: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    by_name: dict[str, list[dict[str, Any]]] = {}
    for package in metadata["packages"]:
        by_name.setdefault(package["name"], []).append(package)
    if "junie-tui" not in by_name:
        raise Reject("ARCHITECTURE")
    core = by_name["junie-tui"][0]
    root_name = contract.get("package", "junie-tui")
    direct = {dependency["name"] for dependency in core["dependencies"] if dependency["kind"] is None}
    if "normal_direct" in contract and direct != set(contract["normal_direct"]):
        failures.append("direct-dependencies")
    for name in contract.get("forbidden_closure", []):
        if name in by_name and any(_graph_paths(metadata, root, name) for root in contract.get("packages", [root_name])):
            failures.append(f"forbidden-{name}")
    for name in contract.get("single_version", []):
        reachable_ids = {path[-1] for path in _graph_paths(metadata, root_name, name, ids=True)}
        reachable = [package for package in by_name.get(name, []) if package["id"] in reachable_ids]
        if len({package["version"] for package in reachable}) != 1:
            failures.append(f"duplicate-{name}")
    for name in contract.get("backend_only", []):
        if name in by_name and any("ratatui-crossterm" not in path for path in _graph_paths(metadata, root_name, name)):
            failures.append(f"unowned-backend-{name}")
    ratatui_core = by_name["ratatui-core"][0]
    node = next(node for node in metadata["resolve"]["nodes"] if node["id"] == ratatui_core["id"])
    if "ratatui_core_features" in contract and set(node["features"]) != set(contract["ratatui_core_features"]):
        failures.append("core-features")
    return failures


def _parse_allowance(text: bytes, policy: dict[str, Any]) -> list[dict[str, str]]:
    rows: list[dict[str, str]] = []
    for line in text.decode().splitlines():
        if not line.strip() or line.startswith("#"):
            continue
        parts = line.split("\t")
        if len(parts) != 5:
            raise Reject("ARCHITECTURE")
        rows.append({"kind": parts[0], "symbol": parts[1], "source": parts[2], "owner": parts[3], "expires": parts[4]})
    return rows


def _validate_allowance(rows: list[dict[str, str]], policy: dict[str, Any], records: list[dict[str, Any]]) -> None:
    if not rows:
        raise Reject("ARCHITECTURE")
    seen: set[tuple[str, str, str]] = set()
    for row in rows:
        if row["kind"] != policy.get("kind"):
            raise Reject("ARCHITECTURE")
        if row["owner"] not in policy.get("allowed_future_owners", []):
            raise Reject("ARCHITECTURE")
        try:
            expires = int(row["expires"])
        except ValueError:
            raise Reject("ARCHITECTURE")
        if expires < policy.get("qualification_date", 0):
            raise Reject("ARCHITECTURE")
        key = (row["kind"], row["symbol"], row["source"])
        if key in seen:
            raise Reject("ARCHITECTURE")
        seen.add(key)
    failures = [record for record in records if record.get("exit") != 0]
    if not failures:
        raise Reject("ARCHITECTURE")
    failure_origins = {record["origin"].split("#", 1)[0] for record in failures}
    for row in rows:
        if row["source"] not in failure_origins:
            raise Reject("ARCHITECTURE")
    if any("DefinitelyAbsent" in row["symbol"] and row["symbol"] != "junie_tui::*" for row in rows):
        for record in records:
            if record.get("exit") == 0:
                raise Reject("ARCHITECTURE")
    if any(row["symbol"] == "junie_tui::*" for row in rows):
        raise Reject("ARCHITECTURE")


def validate_external_consumers(event: dict[str, Any], profile: dict[str, Any]) -> None:
    payload = event.get("payload") or {}
    records = payload.get("records")
    if not isinstance(records, list):
        raise Reject("ARCHITECTURE")
    consumer_dir = Path(profile["consumer_directory"])
    files: dict[str, bytes] = {}
    for path, digest in profile.get("consumer_sources", {}).items():
        data = (consumer_dir / path).read_bytes()
        import hashlib

        if hashlib.sha256(data).hexdigest() != digest:
            raise Reject("ARCHITECTURE")
        files[path] = data
    expected = _consumers(files)
    if len(records) != len(expected):
        raise Reject("ARCHITECTURE")
    failures = [record for record in records if record.get("exit") != 0]
    allowance = profile.get("allowance_policy")
    if allowance:
        rows = _parse_allowance((consumer_dir / "doc_check_allow.tsv").read_bytes(), allowance)
        _validate_allowance(rows, allowance, records)
        return
    if failures:
        raise Reject("ARCHITECTURE")


def validate_cargo_dependencies(event: dict[str, Any], profile: dict[str, Any]) -> None:
    payload = event.get("payload") or {}
    records = payload.get("records")
    if not isinstance(records, list) or len(records) != 1:
        raise Reject("ARCHITECTURE")
    record = records[0]
    if record.get("exit") != 0:
        raise Reject("ARCHITECTURE")
    graph = json.loads(_decode(record, "stdout"))
    policy = profile.get("core_dependency_policy") or profile.get("graph_policy") or {}
    violations = _graph_violations(graph, policy)
    if violations:
        raise Reject("ARCHITECTURE")


def validate_source_policy(event: dict[str, Any], profile: dict[str, Any]) -> None:
    payload = event.get("payload") or {}
    records = payload.get("records")
    if not isinstance(records, list) or len(records) != 1:
        raise Reject("ARCHITECTURE")
    record = records[0]
    if record.get("exit") != 0:
        raise Reject("ARCHITECTURE")
    body = json.loads(_decode(record, "stdout"))
    if body.get("schema") != "tc-protected-source-syntax/v1":
        raise Reject("ARCHITECTURE")
    policy = profile.get("policy", "")
    if policy == "ADJ-13-private-unix-signal-broker/v1":
        validate_broker_observation(body, profile)
        return
    observed = [entry["path"] for entry in body.get("files", [])]
    required = profile.get("required_files") or profile.get("source_roots") or []
    if observed != required:
        raise Reject("ARCHITECTURE")
    sources = _read_source_texts(profile)
    if policy == "complete-production-source-inventory":
        _validate_inventory(body, profile, sources)
    elif policy == "grid-capability-syntax":
        _validate_grid_policy(body, sources)
    elif policy == "unsafe-syntax":
        _validate_unsafe_policy(body, sources)
    elif policy == "public-component-rustdoc-sections":
        _validate_doc_policy(body, profile, sources)
    elif policy == "production-todo-syntax":
        _validate_todo_policy(body, sources)
    elif policy == "legacy-dispatch-exhaustion":
        _validate_legacy_policy(body, sources)
    elif policy == "gate-dispatch-wrapper-table-join":
        _validate_gate_join(body, profile, sources)
    else:
        raise Reject("ARCHITECTURE")


def _entry_source(entry: dict[str, Any], sources: dict[str, str]) -> str:
    if "source" in entry:
        return entry["source"]
    path = entry.get("path")
    if path and path in sources:
        return sources[path]
    return ""


def _validate_inventory(body: dict[str, Any], profile: dict[str, Any], sources: dict[str, str]) -> None:
    for entry in body.get("files", []):
        path = entry.get("path")
        if path not in sources:
            if entry.get("read_error") != "NotFound":
                raise Reject("ARCHITECTURE")
            continue
        if _entry_source(entry, sources) != sources[path]:
            raise Reject("ARCHITECTURE")
        if path.endswith(".rs") and "parse_error" in entry:
            raise Reject("ARCHITECTURE")
    required_examples = set(profile.get("required_examples", []))
    required_manifests = set(profile.get("required_application_manifests", []))
    required_pages = set(profile.get("production_page_modules", []))
    present = {entry["path"] for entry in body.get("files", []) if "read_error" not in entry}
    if not required_examples.issubset(present):
        raise Reject("ARCHITECTURE")
    if not required_manifests.issubset(present):
        raise Reject("ARCHITECTURE")
    for page in required_pages:
        if page not in present:
            raise Reject("ARCHITECTURE")
        entry = next(item for item in body["files"] if item["path"] == page)
        text = _entry_source(entry, sources)
        if text.lstrip().startswith("#![cfg(test)]"):
            raise Reject("ARCHITECTURE")
        if "qualification_removed_update" in text or "qualification_removed_draw" in text:
            raise Reject("ARCHITECTURE")
        if not re.search(r"\bfn update\s*\(", text) or not re.search(r"\bfn draw\s*\(", text):
            raise Reject("ARCHITECTURE")
    app = sources.get("apps/showcase/src/app.rs", "")
    if "apps/showcase/src/app.rs" in profile.get("required_files", []):
        if not app.strip() or "PageId::ALL" not in app:
            raise Reject("ARCHITECTURE")
    if "PageId::ALL" in app:
        order = re.findall(r"Self::(\w+)", app.split("pub const ALL", 1)[1].split("];", 1)[0])
        expected = profile.get("required_page_order", [])
        if expected and order != expected:
            raise Reject("ARCHITECTURE")
        if "PageId::Grid => Box::new(GridPage::new())," not in app and expected:
            raise Reject("ARCHITECTURE")
    for manifest in required_manifests:
        text = sources.get(manifest, "")
        if manifest == "apps/showcase/Cargo.toml":
            if text.count("[[bin]]") > 1:
                raise Reject("ARCHITECTURE")
            if "publish = true" in text:
                raise Reject("ARCHITECTURE")
    if not any(path.startswith("apps/showcase/src/") for path in present):
        if "apps/showcase/src/app.rs" in profile.get("required_files", []):
            raise Reject("ARCHITECTURE")


def _validate_grid_policy(body: dict[str, Any], sources: dict[str, str]) -> None:
    path = "crates/tui/src/components/grid.rs"
    text = sources.get(path, "")
    if "editable< T >" in text:
        raise Reject("ARCHITECTURE")
    if re.search(r"\btrait\s+GridCellActions\b", text):
        raise Reject("ARCHITECTURE")
    if path not in sources:
        raise Reject("ARCHITECTURE")


def _forbid_unsafe_inner_attribute(entry: dict[str, Any]) -> bool:
    attrs = entry.get("file_attributes", "")
    return (
        "AttrStyle::Inner" in attrs
        and re.search(r"ident:\s*Ident\s*\(\s*forbid\s*\)", attrs) is not None
        and "unsafe_code" in attrs
    )


def _validate_unsafe_policy(body: dict[str, Any], sources: dict[str, str]) -> None:
    path = "crates/tui/src/lib.rs"
    entry = next((file for file in body.get("files", []) if file.get("path") == path), None)
    if entry is None:
        raise Reject("ARCHITECTURE")
    text = entry.get("source") or sources.get(path, "")
    facts = entry.get("facts") or []
    if any(fact.get("kind") in ("unsafe_block", "unsafe_impl") for fact in facts):
        raise Reject("ARCHITECTURE")
    if re.search(r"unsafe\s+impl\s+Send", text):
        raise Reject("ARCHITECTURE")
    if not _forbid_unsafe_inner_attribute(entry):
        raise Reject("ARCHITECTURE")


def _validate_doc_policy(body: dict[str, Any], profile: dict[str, Any], sources: dict[str, str]) -> None:
    path = "crates/tui/src/components/button.rs"
    text = sources.get(path, "")
    required = profile.get("required_rustdoc_headings", [])
    for heading in required:
        if f"/// ## {heading}" not in text:
            raise Reject("ARCHITECTURE")


def _validate_todo_policy(body: dict[str, Any], sources: dict[str, str]) -> None:
    path = "apps/showcase/src/app.rs"
    text = sources.get(path, "")
    if 'todo!("no_todo_or_unimplemented")' in text:
        raise Reject("ARCHITECTURE")


def _validate_legacy_policy(body: dict[str, Any], sources: dict[str, str]) -> None:
    app = sources.get("apps/showcase/src/app.rs", "")
    allow = sources.get("crates/tui/tests/allow/legacy_api.txt", "")
    legacy_calls = (
        ".owns(" in app
        or ".locate(" in app
        or ".child(" in app
        or "scrollbar::id_for" in app
    )
    if legacy_calls:
        raise Reject("ARCHITECTURE")
    if allow.strip() and "fn owns(" not in app:
        raise Reject("ARCHITECTURE")


def _validate_gate_join(body: dict[str, Any], profile: dict[str, Any], sources: dict[str, str]) -> None:
    gate = profile.get("joined_gates", ["no_todo_or_unimplemented"])[0]
    xtask = sources.get("xtask/src/main.rs", "")
    wrapper = sources.get("crates/tui/tests/architecture.rs", "")
    doc = sources.get("COMPONENT_ARCHITECTURE.md", "")
    dispatch = f'("{gate}", {gate}),'
    if dispatch not in xtask:
        raise Reject("ARCHITECTURE")
    if xtask.count(dispatch) != 1:
        raise Reject("ARCHITECTURE")
    if f'fn {gate}()' not in wrapper:
        raise Reject("ARCHITECTURE")
    if not re.search(rf'^\| `architecture::{re.escape(gate)}`', doc, re.M):
        raise Reject("ARCHITECTURE")
    required = profile.get("required_registry_names", [])
    found = re.findall(r'"([a-z_]+)"', xtask.split("const CHECKS", 1)[1].split("];", 1)[0])
    if required and found != required:
        raise Reject("ARCHITECTURE")
