#!/usr/bin/env python3
"""Build source-bound T01 API joins from accepted fragments."""

from __future__ import annotations

import hashlib
import json
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[5]
OUT_DIR = Path(__file__).resolve().parent
OUT = OUT_DIR / "api-join-supplement.json"
FIXED_INPUT = "c83f7bc0a0957922a98a94bfe38ad8a8a400df9d"
DEFINITION_INPUT = "542349e2f70ec9883d743594f906ac6c4e96ba5a"
PRODUCT_SOURCE = "1d6ee92adf6ccca38eeee64b1371cba1497e8fcd"
FOUNDATION_PATH = "docs/implementation/pr38-final/results/t01-foundation-api/foundation-api.fragment.json"
COMPONENT_PATH = "docs/implementation/pr38-final/results/t01-components-api/fragment.json"
METADATA_PATH = "docs/implementation/pr38-final/results/t37-final-integration/metadata-root-final.json"

PRODUCERS = [
    {
        "fragment_id": "foundation-api",
        "output_commit": "b2183a222424a1ae4645e6bf8d3a0518c47fa725",
        "artifact_sha256": "79c634ea8e20c6cb62a0c5463f6bfe05442ce15c9e4bb55d59cc6dc612485d73",
        "path": FOUNDATION_PATH,
    },
    {
        "fragment_id": "components-api",
        "output_commit": "aa168c5e3d89759925aa4814111903d53c223850",
        "artifact_sha256": "237bae18fc224fc87b71148a7ff81ce88a7418679aa9f25d4fa1e5133dd6ff1d",
        "path": COMPONENT_PATH,
    },
    {
        "fragment_id": "app-routes",
        "output_commit": "7f6fa8e5e2643f6c9de0b0a6ec9752f800cbcfc7",
        "artifact_sha256": "c63bc59a4d2312aceec26483a49eecbd854635fe559f5dc811cdfa1672b9ad0c",
        "path": "docs/implementation/pr38-final/results/t01-app-routes/fragment.json",
    },
    {
        "fragment_id": "history-requirements",
        "output_commit": "71b857175ca709cc0483b1b528ef093ec18d071c",
        "artifact_sha256": "98acf988c44ae7258678a721b5709c948378a07798d1f8cce1e0b418a7c6ad27",
        "path": "docs/implementation/pr38-final/results/t01-history-requirements/fragment.json",
    },
]


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def compact(value: str) -> str:
    return re.sub(r"\s+", " ", value).strip()


def git_bytes(commit: str, path: str) -> bytes:
    return subprocess.check_output(["git", "show", f"{commit}:{path}"], cwd=ROOT)


def read_json(path: str) -> dict:
    return json.loads((ROOT / path).read_text(encoding="utf-8"))


def pointer(root, pointer: str):
    value = root
    for token in pointer.lstrip("/").split("/"):
        token = token.replace("~1", "/").replace("~0", "~")
        value = value[int(token)] if isinstance(value, list) else value[token]
    return value


def source_header(source: str, line: int) -> str:
    lines = source.splitlines(keepends=True)
    tail = "".join(lines[: line - 1])
    offset = len(tail)
    text = source[offset:]
    depth = 0
    quote = False
    escaped = False
    for index, char in enumerate(text):
        if quote:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                quote = False
            continue
        if char == '"':
            quote = True
        elif char in "([":
            depth += 1
        elif char in ")]":
            depth -= 1
        elif char in "{;" and depth == 0:
            return text[: index + 1].strip()
    raise ValueError(f"cannot find declaration terminator at source line {line}")


def api_ref(
    fragment_id: str,
    pointer_path: str,
    record_id: str,
    public_path: str,
    source_package_id: str,
    definition_package_id: str,
    definition_owner: str,
    workspace_id: str,
    manifest_path: str,
    normalized_package_identity: str,
    manifest_sha256: str,
    public_paths: list[str] | None = None,
) -> dict:
    return {
        "fragment_id": fragment_id,
        "record_pointer": pointer_path,
        "record_id": record_id,
        "public_path": public_path,
        "canonical_public_path": public_path,
        "public_paths": public_paths or ([public_path] if public_path else []),
        "source_package_id": source_package_id,
        "definition_package_id": definition_package_id,
        "definition_owner": definition_owner,
        "workspace_id": workspace_id,
        "manifest_path": manifest_path,
        "normalized_package_identity": normalized_package_identity,
        "manifest_sha256": manifest_sha256,
    }


def component_package_index(component: dict) -> dict:
    return {row["manifest"]: row for row in component["crates"]}


def foundation_package_index(foundation: dict) -> dict:
    return {row["manifest_path"]: row for row in foundation["crates"]}


def complete_package_index(foundation: dict, component: dict, metadata: dict, metadata_hash: str) -> dict:
    packages = {row["manifest_path"]: dict(row) for row in foundation["crates"]}
    component_by_manifest = component_package_index(component)
    root = metadata["workspace_root"].rstrip("/")
    for package in metadata["packages"]:
        manifest_path = package["manifest_path"]
        relative_manifest = manifest_path[len(root) + 1 :] if manifest_path.startswith(root + "/") else None
        if relative_manifest is None:
            continue
        prior = packages.get(relative_manifest, {})
        component_row = component_by_manifest.get(relative_manifest, {})
        name = package["name"]
        version = package["version"]
        manifest_bytes = git_bytes(FIXED_INPUT, relative_manifest)
        manifest_sha256 = sha(manifest_bytes)
        recorded_sha256 = prior.get("manifest_sha256") or component_row.get("manifest_sha256")
        if recorded_sha256 and recorded_sha256 != manifest_sha256:
            raise ValueError(f"package manifest hash differs from accepted metadata: {relative_manifest}")
        packages[relative_manifest] = {
            **prior,
            "package_name": name,
            "package_id": package["id"],
            "metadata_package_id": package["id"],
            "version": version,
            "manifest_path": relative_manifest,
            "manifest_sha256": manifest_sha256,
            "source_scope": prior.get("source_scope", "root_workspace_package"),
            "workspace_kind": prior.get("workspace_kind", "root_workspace_member"),
            "workspace_id": prior.get("workspace_id", "workspace:."),
            "workspace_root": ".",
            "normalized_package_identity": prior.get(
                "normalized_package_identity", f"{name}@{version} at {relative_manifest}"
            ),
            "metadata_artifact": {
                "path": METADATA_PATH,
                "commit": DEFINITION_INPUT,
                "sha256": metadata_hash,
                "workspace_id": prior.get("workspace_id", "workspace:."),
            },
        }
    return packages


def metadata_for_path(path: str, foundation_packages: dict) -> dict:
    parts = path.split("/")
    if len(parts) < 3 or parts[0] != "crates":
        raise ValueError(f"source path has no workspace manifest mapping: {path}")
    manifest = f"crates/{parts[1]}/Cargo.toml"
    row = foundation_packages.get(manifest)
    if row is None:
        raise ValueError(f"package identity is absent from accepted T37 metadata: {manifest}")
    return row


def build() -> dict:
    head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    tree = subprocess.check_output(["git", "rev-parse", f"{FIXED_INPUT}^{{tree}}"], cwd=ROOT, text=True).strip()
    foundation = read_json(FOUNDATION_PATH)
    component = read_json(COMPONENT_PATH)
    if head != FIXED_INPUT:
        raise ValueError(f"fixed input mismatch: expected {FIXED_INPUT}, got {head}")
    if foundation["input_commit"] != DEFINITION_INPUT or component["input_commit"] != DEFINITION_INPUT:
        raise ValueError("accepted API fragments do not share the pinned definition input")
    if foundation["product_source_commit"] != PRODUCT_SOURCE:
        raise ValueError("foundation product-source commit differs from the pinned source")
    unchanged = subprocess.run(
        ["git", "diff", "--quiet", DEFINITION_INPUT, FIXED_INPUT, "--", "crates"],
        cwd=ROOT,
        check=False,
    )
    if unchanged.returncode != 0:
        raise ValueError("product source bytes changed between definition input and fixed candidate")
    for producer in PRODUCERS:
        data = (ROOT / producer["path"]).read_bytes()
        if sha(data) != producer["artifact_sha256"]:
            raise ValueError(f"accepted producer artifact hash mismatch: {producer['path']}")

    metadata_bytes = git_bytes(DEFINITION_INPUT, METADATA_PATH)
    metadata = json.loads(metadata_bytes.decode("utf-8"))
    metadata_record = next(
        row for row in foundation["metadata_workspaces"] if row["evidence_path"] == METADATA_PATH
    )
    if sha(metadata_bytes) != metadata_record["sha256"]:
        raise ValueError("accepted T37 metadata artifact hash differs from foundation provenance")
    foundation_packages = complete_package_index(foundation, component, metadata, sha(metadata_bytes))
    component_packages = component_package_index(component)
    source_records: dict[tuple[str, str], dict] = {}

    def record_source(commit: str, path: str, purpose: str) -> bytes:
        data = git_bytes(commit, path)
        key = (commit, path)
        row = source_records.setdefault(
            key,
            {"path": path, "source_commit": commit, "sha256": sha(data), "purposes": []},
        )
        if row["sha256"] != sha(data):
            raise ValueError(f"source hash changed while building: {path}")
        if purpose not in row["purposes"]:
            row["purposes"].append(purpose)
        return data

    # Pin every API declaration file and its manifest at the fixed candidate.
    record_source(DEFINITION_INPUT, METADATA_PATH, "accepted T37 package and workspace identity")
    paths = {row["source_path"] for row in foundation["items"] if row.get("source_path") == "crates/termrock-terminal/src/terminal_view.rs"}
    paths.update(row["definition"]["path"] for row in component["extension_points"])
    paths.update(
        {
            "crates/termrock-terminal/src/lib.rs",
            "crates/termrock/src/lib.rs",
            "crates/termrock-feedback/src/inspector_body.rs",
            "crates/termrock-feedback/src/lib.rs",
        }
    )
    for path in sorted(paths):
        record_source(FIXED_INPUT, path, "current source declaration, export, or visibility evidence")
        package = metadata_for_path(path, foundation_packages)
        manifest_data = record_source(FIXED_INPUT, package["manifest_path"], "package and manifest identity")
        if sha(manifest_data) != package["manifest_sha256"]:
            raise ValueError(f"manifest hash differs from accepted package identity: {package['manifest_path']}")

    # W44: join each component locator to one foundation item and one direct export.
    w44_index, w44 = next((i, row) for i, row in enumerate(component["components"]) if row["wid"] == "W44")
    requirement_index, w44_requirement = next(
        (i, row) for i, row in enumerate(component["requirements"]) if row.get("requirement_id") == "T01-COMP-W44"
    )
    terminal_path = "crates/termrock-terminal/src/terminal_view.rs"
    terminal_source = record_source(FIXED_INPUT, terminal_path, "W44 declarations")
    terminal_text = terminal_source.decode("utf-8")
    foundation_items = foundation["items"]
    foundation_exports = foundation["exports"]
    w44_joins = []
    w44_api_by_symbol = {}
    for locator_index, locator in enumerate(w44["unjoined_api_declarations"]):
        candidates = [
            (index, item)
            for index, item in enumerate(foundation_items)
            if item.get("source_path") == locator["source"]
            and item.get("line") == locator["line"]
            and item.get("source_sha256") == locator["sha256"]
            and item.get("current_qualified_symbol") == locator["qualified_symbol"]
            and item.get("visibility") == "pub"
        ]
        if len(candidates) != 1:
            raise ValueError(f"W44 locator does not have one exact foundation item: {locator['qualified_symbol']}")
        item_index, item = candidates[0]
        header = source_header(terminal_text, locator["line"])
        if compact(header) != compact(item["signature"]):
            raise ValueError(f"W44 source signature differs from index item: {locator['qualified_symbol']}")
        if sha(header.encode("utf-8")) != item["signature_sha256"]:
            raise ValueError(f"W44 source signature hash differs: {locator['qualified_symbol']}")
        exports = [
            (index, row)
            for index, row in enumerate(foundation_exports)
            if row.get("kind") == "public_definition"
            and row.get("source_path") == locator["source"]
            and row.get("line") == locator["line"]
            and row.get("current_qualified_symbol") == locator["qualified_symbol"]
            and row.get("definition_owner") == item["definition_owner"]
        ]
        if len(exports) != 1:
            raise ValueError(f"W44 locator does not have one exact public_definition export: {locator['qualified_symbol']}")
        export_index, export = exports[0]
        if export.get("source_sha256") != locator["sha256"]:
            raise ValueError(f"W44 export source hash differs: {locator['qualified_symbol']}")
        package = foundation_packages[item["manifest_path"]]
        pointer_path = f"/items/{item_index}"
        join_id = f"T01-JOIN-W44-{item['api_record_id']}"
        owner = item["definition_owner"]
        w44_api_by_symbol[item["current_qualified_symbol"]] = (item_index, item)
        w44_joins.append(
            {
                "join_id": join_id,
                "status": "RESOLVED_SOURCE_EXACT_API_JOIN",
                "source_locator_ref": {
                    "fragment_id": "components-api",
                    "record_pointer": f"/components/{w44_index}/unjoined_api_declarations/{locator_index}",
                    "component_wid": "W44",
                    "api_id_before_join": locator.get("api_id"),
                    "join_state_before_join": locator.get("join_state"),
                },
                "producer_component_ref": {
                    "fragment_id": "components-api",
                    "record_pointer": f"/components/{w44_index}",
                    "record_id": "W44",
                },
                "api_index_ref": api_ref(
                    "foundation-api",
                    pointer_path,
                    item["api_record_id"],
                    export["export_key"]["canonical_public_path"],
                    item["source_package_id"],
                    item["definition_package_id"],
                    owner,
                    item["workspace_id"],
                    item["manifest_path"],
                    item["normalized_package_identity"],
                    package["manifest_sha256"],
                ),
                "source_export_ref": {
                    "fragment_id": "foundation-api",
                    "record_pointer": f"/exports/{export_index}",
                    "record_id": export["export_id"],
                    "kind": export["kind"],
                    "export_key": export["export_key"],
                },
                "source_evidence": {
                    "path": locator["source"],
                    "line": locator["line"],
                    "file_sha256": locator["sha256"],
                    "source_signature": header,
                    "signature_sha256": item["signature_sha256"],
                    "declaration_kind": locator["declaration_kind"],
                    "cfg_attributes": item["cfg_attributes"],
                    "cfg_features": item["cfg_features"],
                    "visibility": item.get("visibility"),
                    "package_id": item["package_id"],
                    "source_package_id": item["source_package_id"],
                    "definition_package_id": item["definition_package_id"],
                    "workspace_id": item["workspace_id"],
                    "manifest_path": item["manifest_path"],
                    "normalized_package_identity": item["normalized_package_identity"],
                    "manifest_sha256": package["manifest_sha256"],
                    "definition_owner": owner,
                },
                "api": {
                    "qualified_symbol": item["current_qualified_symbol"],
                    "definition_owner": owner,
                    "kind": item["kind"],
                    "api_record_id": item["api_record_id"],
                    "signature": item["signature"],
                    "semantic_kinds": item["semantic_kinds"],
                    "states": item["states"],
                    "actions": item["actions"],
                    "builders": item["builders"],
                    "defaults": item["defaults"],
                    "public_qualified_paths": item["public_qualified_paths"],
                },
                "disposition": w44_requirement["disposition"],
                "decision": None,
                "task_id": w44_requirement["named_owner_task"],
                "case_id": w44_requirement["case_ids"][0],
                "evidence_state": "NOT VERIFIED",
            }
        )

    if len(w44_joins) != 37:
        raise ValueError(f"expected 37 W44 joins, found {len(w44_joins)}")

    # Resolve the seven public type paths and their associated item expansions.
    type_rows = {
        row["current_qualified_symbol"].rsplit("::", 1)[-1]: row
        for _, row in w44_api_by_symbol.values()
        if row["kind"] in ("struct", "enum")
    }
    facade_file = "crates/termrock/src/lib.rs"
    terminal_lib_file = "crates/termrock-terminal/src/lib.rs"
    facade_text = record_source(FIXED_INPUT, facade_file, "W44 facade paths") .decode("utf-8")
    terminal_lib_text = record_source(FIXED_INPUT, terminal_lib_file, "W44 package-root paths").decode("utf-8")
    facade_lines = facade_text.splitlines()
    terminal_lib_lines = terminal_lib_text.splitlines()
    explicit_line = w44["all_reexports"][0]["reexport_chain"][0]["line"]
    nested_module_line = next(
        edge["line"] for edge in w44["all_reexports"][2]["reexport_chain"] if edge.get("kind") == "facade_module_declaration"
    )
    nested_glob_line = next(
        edge["line"] for edge in w44["all_reexports"][2]["reexport_chain"] if edge.get("kind") == "facade_glob_reexport"
    )
    if not re.search(r"\bpub\s+mod\s+terminal_view\s*;", terminal_lib_text):
        raise ValueError("terminal_view public module declaration is absent")
    if "pub use terminal_view::*;" not in terminal_lib_text:
        raise ValueError("terminal package-root glob is absent")
    explicit_statement = w44["all_reexports"][0]["reexport_chain"][0]["statement"]
    if not all(re.search(rf"\b{re.escape(name)}\b", explicit_statement) for name in type_rows):
        raise ValueError("explicit facade export does not name every W44 type")
    if "pub use termrock_terminal::*;" not in facade_lines[nested_glob_line - 1]:
        raise ValueError("nested facade glob no longer matches the accepted edge")
    if "pub mod terminal {" not in facade_lines[nested_module_line - 1]:
        raise ValueError("nested facade module no longer matches the accepted edge")
    if "pub use termrock_terminal::{" not in facade_lines[explicit_line - 1]:
        raise ValueError("explicit facade export line no longer matches the accepted edge")

    base_paths = {}
    for name, item in type_rows.items():
        source_paths = list(item["public_qualified_paths"])
        expected_paths = [
            f"termrock::{name}",
            f"termrock::terminal::{name}",
            f"termrock::terminal::terminal_view::{name}",
        ]
        base_paths[name] = list(dict.fromkeys(source_paths + expected_paths))

    def public_paths_for(item: dict) -> dict:
        owner = item["definition_owner"]
        type_name = owner.rsplit("::", 1)[-1]
        bases = base_paths.get(type_name)
        if bases is None:
            raise ValueError(f"W44 declaration has no exact public type owner: {owner}")
        symbol = item["current_qualified_symbol"]
        suffix = symbol[len(owner) :]
        if suffix and not suffix.startswith("::"):
            raise ValueError(f"invalid associated item symbol: {symbol}")
        return {
            "paths": [path + suffix for path in bases],
            "base_type_paths": bases,
            "basis": "Exact source declaration plus public type re-export; associated suffix follows the public type path.",
            "evidence_refs": [
                {"fragment_id": "components-api", "record_pointer": f"/components/{w44_index}"},
                {"fragment_id": "foundation-api", "record_pointer": f"/items/{next(i for i, x in enumerate(foundation_items) if x['api_record_id'] == item['api_record_id'])}"},
                {"fragment_id": "components-api", "record_pointer": f"/components/{w44_index}/all_reexports/0"},
            ],
        }

    for row in w44_joins:
        item = foundation_items[int(row["api_index_ref"]["record_pointer"].split("/")[-1])]
        row["public_path_expansion"] = public_paths_for(item)

    child_kinds = {"field", "enum_variant"}
    type_owners = {row["current_qualified_symbol"] for row in type_rows.values()}
    associated_items = []
    for item_index, item in enumerate(foundation_items):
        if (
            item.get("source_path") == terminal_path
            and item.get("source_sha256") == w44["definition"]["sha256"]
            and item.get("definition_owner") in type_owners
            and item.get("kind") in child_kinds
        ):
            associated_items.append(
                {
                    "api_index_ref": api_ref(
                        "foundation-api",
                        f"/items/{item_index}",
                        item["api_record_id"],
                        item["current_qualified_symbol"],
                        item["source_package_id"],
                        item["definition_package_id"],
                        item["definition_owner"],
                        item["workspace_id"],
                        item["manifest_path"],
                        item["normalized_package_identity"],
                        foundation_packages[item["manifest_path"]]["manifest_sha256"],
                    ),
                    "kind": item["kind"],
                    "visibility": item.get("visibility"),
                    "signature": item["signature"],
                    "signature_sha256": item["signature_sha256"],
                    "public_qualified_paths": item["public_qualified_paths"],
                    "public_path_expansion": public_paths_for(item),
                }
            )

    semantic_by_kind = {}
    for _, item in w44_api_by_symbol.values():
        for semantic in item["semantic_kinds"]:
            semantic_by_kind.setdefault(semantic, []).append(item["api_record_id"])

    def w44_api_pointer(item_index: int, item: dict) -> dict:
        return {
            "fragment_id": "foundation-api",
            "record_pointer": f"/items/{item_index}",
            "record_id": item["api_record_id"],
            "qualified_symbol": item["current_qualified_symbol"],
            "definition_owner": item["definition_owner"],
        }

    builder_api_refs = [
        w44_api_pointer(index, item)
        for index, item in w44_api_by_symbol.values()
        if item.get("kind") == "method" and "builder" in item.get("semantic_kinds", [])
    ]
    state_method_api_refs = [
        w44_api_pointer(index, item)
        for index, item in w44_api_by_symbol.values()
        if item.get("kind") == "method" and item.get("definition_owner") == "termrock_terminal::terminal_view::TerminalViewState"
    ]
    constructor_api_refs = [
        w44_api_pointer(index, item)
        for index, item in w44_api_by_symbol.values()
        if item.get("kind") == "method" and item["current_qualified_symbol"].rsplit("::", 1)[-1] == "new"
    ]
    state_symbol = "termrock_terminal::terminal_view::TerminalViewState"
    action_symbol = "termrock_terminal::terminal_view::TerminalAction"
    state_item_index, state_item = w44_api_by_symbol[state_symbol]
    action_item_index, action_item = w44_api_by_symbol[action_symbol]
    update_item_index, update_item = w44_api_by_symbol["termrock_terminal::terminal_view::TerminalView::update"]
    draw_item_index, draw_item = w44_api_by_symbol["termrock_terminal::terminal_view::TerminalView::draw"]
    if "TerminalViewState" not in update_item["signature"] or "TerminalAction" not in update_item["signature"]:
        raise ValueError("TerminalView::update signature no longer carries state and action types")
    if "TerminalViewState" not in draw_item["signature"]:
        raise ValueError("TerminalView::draw signature no longer carries TerminalViewState")

    # Derived Default has source evidence but no explicit trait implementation row.
    state_source = terminal_text.splitlines()
    derive_line = state_item["line"] - 1
    if not state_source[derive_line - 1].startswith("#[derive("):
        raise ValueError("TerminalViewState derive attribute is not adjacent to its type declaration")
    derive_text = state_source[derive_line - 1].strip()
    if derive_text != "#[derive(Debug, Clone, PartialEq, Eq, Default)]":
        raise ValueError("TerminalViewState derive list differs from accepted source")
    has_default_impl = any(
        row.get("implemented_trait") == "core::default::Default"
        and row.get("definition_owner") == state_symbol
        for row in foundation_items
    ) or any(
        row.get("trait") == "core::default::Default"
        and row.get("receiver") == state_symbol
        for row in foundation.get("unresolved_trait_impls", [])
    )
    if has_default_impl:
        raise ValueError("Default is now present in the accepted trait implementation index")
    if any(
        "TerminalViewState as core::default::Default" in json.dumps(row)
        for row in foundation_exports
    ):
        raise ValueError("Default is now present in accepted foundation exports")
    default_record = {
        "join_id": "T01-JOIN-W44-DERIVED-DEFAULT-7911c30fd31409f94ba7d20292bc738b5261ff25174e06b5781ba7b22c5552a7",
        "record_kind": "SUPPLEMENTAL_DERIVED_TRAIT_IMPL",
        "status": "RESOLVED_SOURCE_DERIVE_EVIDENCE_INDEX_ROW_ABSENT",
        "api_id": None,
        "trait_path": "core::default::Default",
        "receiver_api_ref": api_ref(
            "foundation-api",
            f"/items/{state_item_index}",
            state_item["api_record_id"],
            state_item["current_qualified_symbol"],
            state_item["source_package_id"],
            state_item["definition_package_id"],
            state_item["definition_owner"],
            state_item["workspace_id"],
            state_item["manifest_path"],
            state_item["normalized_package_identity"],
            foundation_packages[state_item["manifest_path"]]["manifest_sha256"],
        ),
        "component_default_locator_ref": {
            "fragment_id": "components-api",
            "record_pointer": f"/components/{w44_index}/state_action_builder_default_api_ids/default_impls/0",
            "record_id": None,
        },
        "source_evidence": {
            "path": terminal_path,
            "attribute_line": derive_line,
            "receiver_definition_line": state_item["line"],
            "file_sha256": w44["definition"]["sha256"],
            "attribute_text": derive_text,
            "declaration_kind": "derive_generated_trait_impl",
            "cfg_attributes": state_item["cfg_attributes"],
            "cfg_features": state_item["cfg_features"],
            "source_package_id": state_item["source_package_id"],
            "definition_package_id": state_item["definition_package_id"],
            "workspace_id": state_item["workspace_id"],
            "manifest_path": state_item["manifest_path"],
            "normalized_package_identity": state_item["normalized_package_identity"],
            "manifest_sha256": foundation_packages[state_item["manifest_path"]]["manifest_sha256"],
        },
        "index_absence": {
            "foundation_items": "No explicit Default impl item was found for this receiver.",
            "foundation_trait_impls": "No Default receiver record was found in unresolved_trait_impls or export rows.",
            "claim_limit": "A scoped source/index join; no rustc all-feature inventory was run.",
        },
        "disposition": None,
        "decision": None,
        "task_id": w44_requirement["named_owner_task"],
        "case_id": w44_requirement["case_ids"][0],
    }

    # Extension API joins: retain all 30 IDs and path evidence.
    extension_joins = []
    for extension_index, extension in enumerate(component["extension_points"]):
        path = extension["definition"]["path"]
        line = extension["definition"]["lines"][0]
        file_sha = extension["definition"]["sha256"]
        symbol = extension["qualified_symbol"]
        signature = extension["signature"]
        source = record_source(FIXED_INPUT, path, "extension declaration")
        source_text = source.decode("utf-8")
        if sha(source) != file_sha:
            raise ValueError(f"extension source file hash differs: {symbol}")
        source_ref = extension["source_declaration_ref"]
        if source_ref["statement"] != signature or source_ref["line"] != line:
            raise ValueError(f"extension source declaration evidence differs: {symbol}")
        if signature not in source_text:
            raise ValueError(f"extension signature does not occur in pinned source: {symbol}")
        if extension.get("api_id") is not None:
            expected_api_id = f"API:{symbol}@{path}:{line}"
            if extension["api_id"] != expected_api_id:
                raise ValueError(f"existing extension API ID does not match source locator: {symbol}")
            candidates = [
                (index, item)
                for index, item in enumerate(component["items"])
                if item.get("api_id") == extension["api_id"]
                and item.get("source") == path
                and item.get("line_start") == line
                and item.get("sha256") == file_sha
                and item.get("qualified_symbol") == symbol
                and item.get("signature") == signature
                and item.get("definition_symbol") == symbol
            ]
            source_fragment = "components-api"
            rows = component["items"]
            prefix = "components-api"
            if len(candidates) != 1:
                raise ValueError(f"existing extension API ID does not have one exact component item: {symbol}")
            item_index, item = candidates[0]
            item_ref_id = item["api_id"]
            canonical_path = item["definition_symbol"]
            owner = item["definition_symbol"]
            index_signature_status = "EXACT_FULL_SOURCE_SIGNATURE"
        else:
            candidates = [
                (index, item)
                for index, item in enumerate(foundation_items)
                if item.get("source_path") == path
                and item.get("line") == line
                and item.get("source_sha256") == file_sha
                and item.get("current_qualified_symbol") == symbol
            ]
            source_fragment = "foundation-api"
            rows = foundation_items
            prefix = "foundation-api"
            if len(candidates) != 1:
                raise ValueError(f"null extension ID lacks one exact foundation item: {symbol}")
            item_index, item = candidates[0]
            header = compact(source_header(source_text, line))
            index_signature = compact(item["signature"])
            if index_signature == header:
                index_signature_status = "EXACT_SOURCE_HEADER; FULL_SIGNATURE_BOUND_BY_EXTENSION_SOURCE_RECORD"
            elif index_signature.startswith(header):
                index_signature_status = "SOURCE_HEADER_PREFIX; INDEX_SIGNATURE_HAS_TRAILING_TEXT"
            else:
                raise ValueError(f"extension header differs from exact foundation item: {symbol}")
            item_ref_id = item["api_record_id"]
            canonical_path = item["current_qualified_symbol"]
            owner = item["definition_owner"]
        if source_fragment == "components-api":
            if item["reachability_paths"] and not set(item["reachability_paths"]).issubset(
                set(extension["public_reachability_paths"])
            ):
                raise ValueError(f"component API reachability paths disagree with extension evidence: {symbol}")
            source_package = metadata_for_path(path, foundation_packages)
            source_package_id = source_package["package_id"]
            definition_package_id = source_package_id
            workspace_id = source_package["workspace_id"]
            manifest_path = source_package["manifest_path"]
            normalized_identity = source_package["normalized_package_identity"]
            manifest_sha = source_package["manifest_sha256"]
            module_visibility = extension["definition_module_visibility"]
            item_kind = item["kind"]
            declaration_visibility = extension["source_declaration_visibility"]
        else:
            source_package = foundation_packages[item["manifest_path"]]
            source_package_id = item["source_package_id"]
            definition_package_id = item["definition_package_id"]
            workspace_id = item["workspace_id"]
            manifest_path = item["manifest_path"]
            normalized_identity = item["normalized_package_identity"]
            manifest_sha = source_package["manifest_sha256"]
            module_visibility = extension["definition_module_visibility"]
            item_kind = item["kind"]
            declaration_visibility = extension["source_declaration_visibility"]
        if module_visibility.get("unresolved_edges"):
            raise ValueError(f"extension declaration module path has unresolved edges: {symbol}")
        path_evidence_paths = [row["path"] for row in extension["public_path_evidence"]]
        if len(path_evidence_paths) != len(set(path_evidence_paths)):
            raise ValueError(f"duplicate public path evidence for extension: {symbol}")
        if set(path_evidence_paths) != set(extension["public_reachability_paths"]):
            raise ValueError(f"extension public paths lack exact path evidence: {symbol}")
        if not all(row.get("externally_reachable") is True for row in extension["public_path_evidence"]):
            raise ValueError(f"extension has a public path without source reachability evidence: {symbol}")

        public_export_refs = []
        public_path_evidence_refs = []
        for public_path in extension["public_reachability_paths"]:
            evidence_index = next(
                index for index, evidence in enumerate(extension["public_path_evidence"]) if evidence["path"] == public_path
            )
            public_path_evidence_refs.append(
                {
                    "public_path": public_path,
                    "fragment_id": "components-api",
                    "record_pointer": f"/extension_points/{extension_index}/public_path_evidence/{evidence_index}",
                }
            )
            for fragment_id, export_rows in (("foundation-api", foundation_exports), ("components-api", component["exports"])):
                for export_index, export_row in enumerate(export_rows):
                    if fragment_id == "components-api":
                        export_paths = set(export_row.get("public_paths", []))
                        owner_matches = any(
                            expanded.get("actual_definition_owner") == owner
                            and expanded.get("public_path") == public_path
                            for expanded in export_row.get("expanded_symbols", [])
                        )
                    else:
                        export_paths = set(export_row.get("public_qualified_paths", []))
                        canonical = (export_row.get("export_key") or {}).get("canonical_public_path")
                        if canonical:
                            export_paths.add(canonical)
                        owner_matches = export_row.get("definition_owner") == owner
                    if public_path in export_paths and owner_matches:
                        public_export_refs.append(
                            {
                                "fragment_id": fragment_id,
                                "record_pointer": f"/exports/{export_index}",
                                "record_id": export_row.get("export_id"),
                                "public_path": public_path,
                                "definition_owner": owner,
                            }
                        )

        trait_members = []
        if item_kind == "trait":
            start_line = line
            end_line = line + len(signature.splitlines()) - 1
            member_rows = [
                (index, member)
                for index, member in enumerate(foundation_items)
                if member.get("source_path") == path
                and member.get("source_sha256") == file_sha
                and member.get("definition_owner") == owner
                and member.get("kind") in ("trait_method", "associated_type", "associated_const")
                and start_line < member.get("line", 0) <= end_line
            ]
            for member_index, member in member_rows:
                trait_members.append(
                    {
                        "api_index_ref": api_ref(
                            "foundation-api",
                            f"/items/{member_index}",
                            member["api_record_id"],
                            member["current_qualified_symbol"]
                            if member["current_qualified_symbol"] in member.get("public_qualified_paths", [])
                            else None,
                            member["source_package_id"],
                            member["definition_package_id"],
                            member["definition_owner"],
                            member["workspace_id"],
                            member["manifest_path"],
                            member["normalized_package_identity"],
                            foundation_packages[member["manifest_path"]]["manifest_sha256"],
                        member.get("public_qualified_paths", []),
                        ),
                        "kind": member["kind"],
                        "signature": member["signature"],
                        "signature_sha256": member["signature_sha256"],
                        "visibility_in_trait": "implicit_public_trait_member",
                        "source_path": member["source_path"],
                        "line": member["line"],
                        "source_sha256": member["source_sha256"],
                    }
                )
        join_id = f"T01-JOIN-EXT-{hashlib.sha256(extension['extension_id'].encode()).hexdigest()[:20]}"
        extension_joins.append(
            {
                "join_id": join_id,
                "extension_id": extension["extension_id"],
                "name": extension["name"],
                "status": "RESOLVED_SOURCE_EXACT_API_ITEM_JOIN",
                "join_basis": "Exact declaration signature, source symbol, source line, file hash, package manifest identity, module visibility, and public path evidence.",
                "producer_ref": {
                    "fragment_id": "components-api",
                    "record_pointer": f"/extension_points/{extension_index}",
                    "record_id": extension["extension_id"],
                    "producer_api_id": extension.get("api_id"),
                    "producer_join_status": extension.get("api_item_join", {}).get("status"),
                },
                "api_index_ref": api_ref(
                    source_fragment,
                    f"/items/{item_index}",
                    item_ref_id,
                    canonical_path if canonical_path in extension["public_reachability_paths"] else None,
                    source_package_id,
                    definition_package_id,
                    owner,
                    workspace_id,
                    manifest_path,
                    normalized_identity,
                    manifest_sha,
                    extension["public_reachability_paths"],
                ),
                "source_evidence": {
                    "path": path,
                    "line_start": line,
                    "line_end": line + len(signature.splitlines()) - 1,
                    "file_sha256": file_sha,
                    "signature": signature,
                    "signature_sha256": sha(signature.encode("utf-8")),
                    "api_index_signature": item["signature"],
                    "api_index_signature_sha256": item.get("signature_sha256"),
                    "api_index_signature_status": index_signature_status,
                    "qualified_symbol": symbol,
                    "declaration_kind": extension["declaration_kind"],
                    "declaration_visibility": declaration_visibility,
                    "cfg": extension["cfg"],
                    "source_package_id": source_package_id,
                    "definition_package_id": definition_package_id,
                    "workspace_id": workspace_id,
                    "manifest_path": manifest_path,
                    "manifest_sha256": manifest_sha,
                    "normalized_package_identity": normalized_identity,
                    "definition_owner": owner,
                    "index_kind": item_kind,
                },
                "public_visibility": {
                    "declaration_visibility": declaration_visibility,
                    "module_visibility": module_visibility,
                    "public_paths": extension["public_reachability_paths"],
                    "public_path_evidence": extension["public_path_evidence"],
                    "public_export_refs": public_export_refs,
                    "public_path_evidence_refs": public_path_evidence_refs,
                    "canonical_path_status": "SOURCE_PATH_PUBLIC" if canonical_path in extension["public_reachability_paths"] else "NO_SINGLE_CANONICAL_PUBLIC_PATH_SELECTED",
                    "rustc_all_feature_visibility_checked": False,
                },
                "trait_member_refs": trait_members,
                "trait_member_index_status": "JOINED_EXACT_SOURCE_OWNER" if trait_members else ("NOT_A_TRAIT" if item_kind != "trait" else "NO_SEPARATE_MEMBER_ROWS_IN_ACCEPTED_FOUNDATION_INDEX"),
                "disposition": None,
                "decision": None,
                "task_id": extension.get("named_owner_task"),
                "case_id": None,
                "evidence_state": extension.get("evidence", "NOT_VERIFIED"),
            }
        )
    if len(extension_joins) != 30:
        raise ValueError(f"expected 30 extension joins, found {len(extension_joins)}")
    if sum(row["producer_ref"]["producer_api_id"] is None for row in extension_joins) != 9:
        raise ValueError("the nine null extension API IDs changed")

    # InspectorBody remains a source-backed product gap with no retirement decision.
    inspector_index, inspector = next(
        (i, row) for i, row in enumerate(component["application_specific_apis"]) if row.get("name") == "InspectorBody"
    )
    if not inspector["missing_export"] or inspector["disposition"] != "UNRESOLVED" or not inspector["blocks_final"]:
        raise ValueError("InspectorBody producer disposition changed")
    inspector_source_path = "crates/termrock-feedback/src/inspector_body.rs"
    inspector_lib_path = "crates/termrock-feedback/src/lib.rs"
    current_inspector = record_source(FIXED_INPUT, inspector_source_path, "current InspectorBody gap")
    current_lib = record_source(FIXED_INPUT, inspector_lib_path, "current InspectorBody public exports")
    historical_inspector = record_source(PRODUCT_SOURCE, inspector_source_path, "historical InspectorBody gap")
    historical_lib = record_source(PRODUCT_SOURCE, inspector_lib_path, "historical InspectorBody public exports")
    exact_name = re.compile(r"\bInspectorBody\b")
    scoped_sources = [current_inspector.decode(), current_lib.decode(), historical_inspector.decode(), historical_lib.decode()]
    if any(exact_name.search(text) for text in scoped_sources):
        raise ValueError("an exact InspectorBody token exists in a scoped current or historical source file")
    exact_index_hits = []
    for fragment_id, fragment in (("foundation-api", foundation), ("components-api", component)):
        for collection in ("items", "exports"):
            for index, record in enumerate(fragment.get(collection, [])):
                if exact_name.search(json.dumps(record, ensure_ascii=False)):
                    exact_index_hits.append({"fragment_id": fragment_id, "record_pointer": f"/{collection}/{index}"})
    if exact_index_hits:
        raise ValueError(f"an exact InspectorBody token exists in an accepted public API index: {exact_index_hits}")
    audit_path = "docs/audits/evidence/2026-10-21-current-blockers/inspector-body-f8f3-independent-rejection.json"
    audit_record = json.loads(record_source(DEFINITION_INPUT, audit_path, "historical independent source audit").decode("utf-8"))
    if audit_record.get("decision") is not None:
        raise ValueError("historical InspectorBody audit now has a decision")
    matrix_path = "docs/architecture/termrock-migration-matrix.md"
    matrix_text = record_source(DEFINITION_INPUT, matrix_path, "historical migration evidence").decode("utf-8")
    matrix_lines = matrix_text.splitlines()
    matrix_matches = [i for i, line in enumerate(matrix_lines, 1) if "InspectorBody" in line]
    api_rules_path = "docs/implementation/pr38-final/API-RULES.md"
    matrix_plan_path = "docs/implementation/pr38-final/COMPONENT-MATRIX.md"
    api_rules_text = record_source(FIXED_INPUT, api_rules_path, "retained inventory contract").decode("utf-8")
    matrix_plan_text = record_source(FIXED_INPUT, matrix_plan_path, "retained owner contract").decode("utf-8")
    inspector_exports = [
        "InspectTone",
        "InspectSpan",
        "InspectLine",
        "InspectorBodyWidths",
        "paint_inspector_body",
        "inspector_body_row_count",
        "paint_inspector_body_lines_window",
    ]
    if not all(name in current_lib.decode() and name in inspector["matched_public_symbols"] for name in inspector_exports):
        raise ValueError("retained InspectorBody support exports differ from producer evidence")
    current_source_lines = current_inspector.decode().splitlines()
    retained_declarations = []
    for name in inspector_exports:
        pattern = re.compile(rf"\b(?:pub\s+(?:struct|enum|fn)\s+){re.escape(name)}\b")
        found = [i for i, line in enumerate(current_source_lines, 1) if pattern.search(line)]
        if len(found) != 1:
            raise ValueError(f"retained source declaration does not occur once: {name}")
        retained_declarations.append({"name": name, "path": inspector_source_path, "line": found[0]})
    inspector_record = {
        "join_id": "T01-JOIN-APPAPI-INSPECTORBODY-UNRESOLVED",
        "producer_ref": {
            "fragment_id": "components-api",
            "record_pointer": f"/application_specific_apis/{inspector_index}",
            "record_id": inspector.get("api_id"),
        },
        "name": "InspectorBody",
        "disposition": "UNRESOLVED",
        "decision": None,
        "task_id": inspector["named_owner_task"],
        "case_id": inspector["case_ids"][0],
        "producer_blocks_final": inspector["blocks_final"],
        "t01_disposition_evidence_complete": True,
        "blocks_t01_inventory": False,
        "blocks_final_goal": True,
        "product_gap": {
            "status": "MISSING_PUBLIC_DECLARATION",
            "current_source": {"path": inspector_source_path, "sha256": sha(current_inspector)},
            "current_public_exports": {"path": inspector_lib_path, "sha256": sha(current_lib), "retained_support_apis": retained_declarations},
            "historical_source": {
                "commit": PRODUCT_SOURCE,
                "path": inspector_source_path,
                "sha256": sha(historical_inspector),
                "exact_name_declaration_found": False,
            },
            "historical_public_exports": {"path": inspector_lib_path, "sha256": sha(historical_lib), "exact_name_export_found": False},
            "scope": "Pinned source files and accepted public API indexes only; macro expansion and rustc all-feature visibility are not proven.",
        },
        "retained_contract": {
            "api_rules": {
                "path": api_rules_path,
                "sha256": sha(git_bytes(FIXED_INPUT, api_rules_path)),
                "line": next(i for i, line in enumerate(api_rules_text.splitlines(), 1) if line.strip() == "## 13. Complete the public inventory"),
            },
            "component_matrix": {
                "path": matrix_plan_path,
                "sha256": sha(git_bytes(FIXED_INPUT, matrix_plan_path)),
                "line": next(i for i, line in enumerate(matrix_plan_text.splitlines(), 1) if "inspector paint APIs" in line),
                "text": next(line.strip() for line in matrix_plan_text.splitlines() if "inspector paint APIs" in line),
            },
            "historical_migration_matrix": {
                "path": matrix_path,
                "source_commit": DEFINITION_INPUT,
                "sha256": sha(git_bytes(DEFINITION_INPUT, matrix_path)),
                "matching_lines": matrix_matches,
            },
            "historical_audit": {
                "path": audit_path,
                "source_commit": DEFINITION_INPUT,
                "sha256": sha(git_bytes(DEFINITION_INPUT, audit_path)),
                "decision": audit_record.get("decision"),
                "evidence_state": audit_record.get("evidence_state"),
            },
        },
        "later_case_proposal": {
            "status": "PROPOSED_NOT_APPROVED",
            "case_id": None,
            "task_id": "T07",
            "text": "After T07 records the product declaration and path, call that API from an external consumer. Check projected rows, field actions, clipping, and natural size.",
        },
    }

    # Source-level public module and facade edges retained for all W44 rows.
    public_edges = [
        {"path": terminal_lib_file, "line": 2, "sha256": sha(git_bytes(FIXED_INPUT, terminal_lib_file)), "statement": "pub mod terminal_view;"},
        {"path": terminal_lib_file, "line": 4, "sha256": sha(git_bytes(FIXED_INPUT, terminal_lib_file)), "statement": "pub use terminal_view::*;"},
        {"path": facade_file, "line": nested_module_line, "sha256": sha(git_bytes(FIXED_INPUT, facade_file)), "statement": "pub mod terminal {"},
        {"path": facade_file, "line": nested_glob_line, "sha256": sha(git_bytes(FIXED_INPUT, facade_file)), "statement": "    pub use termrock_terminal::*;"},
        {"path": facade_file, "line": explicit_line, "sha256": sha(git_bytes(FIXED_INPUT, facade_file)), "statement": explicit_statement},
    ]
    result = {
        "schema": "termrock.pr38.t01.final_api_join_supplement.v1",
        "task_id": "T01",
        "status": "SCOPED_STRUCTURAL_SUPPLEMENT_REVIEW_PENDING",
        "provenance": {
            "fixed_candidate_input": {"commit": FIXED_INPUT, "tree": tree},
            "definition_input": {"commit": DEFINITION_INPUT, "tree": foundation["input_tree"]},
            "product_source": {"commit": PRODUCT_SOURCE, "behavior": "NOT VERIFIED"},
            "source_bytes_unchanged_from_definition_input": True,
            "accepted_producers": PRODUCERS,
        },
        "summary": {
            "w44_locator_count": len(w44["unjoined_api_declarations"]),
            "w44_resolved_count": len(w44_joins),
            "w44_direct_public_definition_export_count": len(w44_joins),
            "w44_associated_field_and_variant_count": len(associated_items),
            "w44_derived_default_supplemental_count": 1,
            "extension_count": len(extension_joins),
            "extension_preexisting_api_id_count": sum(row["producer_ref"]["producer_api_id"] is not None for row in extension_joins),
            "extension_completed_null_api_id_count": sum(row["producer_ref"]["producer_api_id"] is None for row in extension_joins),
            "inspector_body_disposition_count": 1,
        },
        "source_hash_records": sorted(source_records.values(), key=lambda row: (row["source_commit"], row["path"])),
        "w44_api_joins": w44_joins,
        "w44_relationships": {
            "component_ref": {"fragment_id": "components-api", "record_pointer": f"/components/{w44_index}", "record_id": "W44"},
            "component_requirement_ref": {"fragment_id": "components-api", "record_pointer": f"/requirements/{requirement_index}", "record_id": w44_requirement["requirement_id"]},
            "state_type_ref": {"fragment_id": "foundation-api", "record_pointer": f"/items/{state_item_index}", "record_id": state_item["api_record_id"], "symbol": state_symbol},
            "action_type_ref": {"fragment_id": "foundation-api", "record_pointer": f"/items/{action_item_index}", "record_id": action_item["api_record_id"], "symbol": action_symbol},
            "update_method_ref": {"fragment_id": "foundation-api", "record_pointer": f"/items/{update_item_index}", "record_id": update_item["api_record_id"], "uses_state": state_item["api_record_id"], "returns_action": action_item["api_record_id"]},
            "draw_method_ref": {"fragment_id": "foundation-api", "record_pointer": f"/items/{draw_item_index}", "record_id": draw_item["api_record_id"], "uses_state": state_item["api_record_id"]},
            "semantic_api_record_ids": semantic_by_kind,
            "constructor_api_refs": constructor_api_refs,
            "builder_api_refs": builder_api_refs,
            "state_method_api_refs": state_method_api_refs,
            "action_method_api_refs": [
                w44_api_pointer(index, item)
                for index, item in w44_api_by_symbol.values()
                if item.get("kind") == "method" and "TerminalAction" in item.get("signature", "")
            ],
            "associated_fields_and_variants": associated_items,
            "state_field_api_refs": [
                row for row in associated_items if row["kind"] == "field" and row["api_index_ref"]["definition_owner"] == state_symbol
            ],
            "action_variant_api_refs": [
                row for row in associated_items if row["kind"] == "enum_variant" and row["api_index_ref"]["definition_owner"] == action_symbol
            ],
            "public_reexport_sources": public_edges,
            "derived_default": default_record,
        },
        "extension_joins": extension_joins,
        "inspector_body_disposition": inspector_record,
        "verification": {
            "scope": "Structural preflight only.",
            "functional_case_count": 0,
            "functional_status": "NOT VERIFIED",
            "cargo_or_product_tests_run": False,
            "independent_review": "PENDING",
        },
        "limits": [
            "No Cargo command, build, product test, harness edit, fixture edit, doctest edit, or source edit was run.",
            "The join uses accepted source inventories and pinned source bytes; it does not prove rustc all-feature visibility.",
            "The derived Default row is supplemental evidence, not an API item ID.",
            "InspectorBody remains an unresolved product gap for T07; no retirement decision exists.",
        ],
    }
    return result


if __name__ == "__main__":
    result = build()
    OUT.write_text(json.dumps(result, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"wrote {OUT.relative_to(ROOT)}")
