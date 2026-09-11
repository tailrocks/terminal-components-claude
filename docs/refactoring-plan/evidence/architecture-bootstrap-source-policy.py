"""Actual source-copy policy cases and a compiled syn observation instrument."""
from __future__ import annotations
import json
from pathlib import Path
import re

HERE = Path(__file__).resolve().parent
APP = "apps/showcase/src/app.rs"
GRID = "crates/tui/src/components/grid.rs"
XTASK = "xtask/src/main.rs"
WRAPPER = "crates/tui/tests/architecture.rs"
DOC = "COMPONENT_ARCHITECTURE.md"
ALLOW = "xtask/doc_check_allow.txt"


def prepare(api, original, root, target, identity, env):
    actual, require, sha, command, decode = api.actual, api.require, api.sha, api.command, api.decode
    observer_source = dict(original)
    observer_path = "xtask/src/bin/architecture_source_observer.rs"
    observer_source[observer_path] = (HERE / "architecture-bootstrap-source-observer.rs").read_bytes()
    actual.write_sources(root, observer_source)
    build = command([identity["cargo"]["path"], "build", "-p", "xtask", "--bin", "architecture_source_observer", "--locked", "--offline", "--message-format=json"], root, env)
    require(build["exit"] == 0, "actual syn observer did not compile: " + decode(build, "stderr").decode()[-2000:])
    artifacts = [value for line in decode(build, "stdout").splitlines()
                 if (value := json.loads(line)).get("reason") == "compiler-artifact" and value.get("executable") and value["target"]["name"] == "architecture_source_observer"]
    require(len(artifacts) == 1, "exact parser executable absent")
    executable = Path(artifacts[0]["executable"]).read_bytes()
    examples = sorted(path for path in original if re.fullmatch(r"crates/tui/examples/(?:0[1-9]|1[0-3])_[^/]+\.rs", path))
    require(len(examples) == 13, "actual named 1–13 example inventory incomplete")
    manifests = sorted(path for path in original if re.fullmatch(r"apps/[^/]+/Cargo.toml", path))
    pages = sorted(path for path in original if path.startswith("apps/showcase/src/pages/") and path.endswith(".rs"))
    require(len(manifests) == 4 and len(pages) >= 22, "actual app/page inventory premise changed")
    roots = examples + manifests + pages + [APP, GRID, XTASK, WRAPPER, DOC, "crates/tui/src/lib.rs"]
    cases = []

    def add(name, changes, policy, *, positive=False, paths=None, **profile):
        source = dict(original)
        for path, data in changes.items():
            if data is None:
                require(path in source, "missing mutation target " + path)
                del source[path]
            else:
                source[path] = data.encode() if isinstance(data, str) else data
        selected = sorted(set(paths or roots))
        cases.append({"name": name, "kind": "source-policy", "source": source, "files": {},
                      "category": None if positive else "ARCHITECTURE", "toolchain": identity,
                      "observer": executable, "observer_sha256": sha(executable), "observer_build": build,
                      "observed_paths": selected,
                      "policy_profile": {"policy": policy, "required_files": selected, **profile}})

    def replace(path, before, after):
        return actual.replace_once(original[path].decode(), before, after)

    inventory = {"required_examples": examples, "required_application_manifests": manifests,
                 "required_showcase_page_modules": pages, "source_authority": actual.PIN,
                 "production_page_modules": [path for path in pages if path not in {"apps/showcase/src/pages/mod.rs", "apps/showcase/src/pages/author.rs"}]}
    all_source = original[APP].decode().split('pub const ALL: [Self; 22] = [', 1)[1].split('\n    ];', 1)[0]
    page_order = re.findall(r'Self::(\w+)', all_source)
    require(len(page_order) == 22 and len(set(page_order)) == 22, "actual PageId ALL order ambiguous")
    inventory["required_page_order"] = page_order
    add("inventory-valid", {}, "complete-production-source-inventory", positive=True, **inventory)
    add("inventory-comment-positive", {APP: original[APP] + b'\n// A mention of todo! or unsafe impl is not executable syntax.\n'},
        "complete-production-source-inventory", positive=True, **inventory)
    add("actual-source-parse-error", {APP: original[APP] + b'\nfn qualification_broken( {\n'},
        "complete-production-source-inventory", **inventory)
    add("empty-existing-app-source", {APP: b''}, "complete-production-source-inventory", **inventory)
    for path in examples:
        add("missing-example-" + Path(path).stem, {path: None}, "complete-production-source-inventory", **inventory)
    for path in manifests:
        add("missing-manifest-" + path.split('/')[1], {path: None}, "complete-production-source-inventory", **inventory)
    add("duplicate-bin", {"apps/showcase/Cargo.toml": original["apps/showcase/Cargo.toml"] + b'\n[[bin]]\nname="showcase"\npath="src/main.rs"\n'}, "complete-production-source-inventory", **inventory)
    add("published-app", {"apps/showcase/Cargo.toml": replace("apps/showcase/Cargo.toml", "publish = false", "publish = true")}, "complete-production-source-inventory", **inventory)
    add("missing-app-root", {path: None for path in original if path.startswith("apps/showcase/src/")}, "complete-production-source-inventory", **inventory)
    add("missing-page", {"apps/showcase/src/pages/grid.rs": None}, "complete-production-source-inventory", **inventory)
    add("shared-fake-page", {"apps/showcase/src/pages/grid.rs": 'pub use super::buttons::*;\n'}, "complete-production-source-inventory", **inventory)
    add("test-only-page", {"apps/showcase/src/pages/grid.rs": '#![cfg(test)]\n' + original["apps/showcase/src/pages/grid.rs"].decode()}, "complete-production-source-inventory", **inventory)
    add("page-dispatch-omission", {APP: replace(APP, '        PageId::Grid => Box::new(GridPage::new()),\n', '')}, "complete-production-source-inventory", **inventory)
    for name, changed in [
        ("page-order-swapped", all_source.replace('Self::Overview', 'Self::TEMP').replace('Self::Buttons', 'Self::Overview').replace('Self::TEMP', 'Self::Buttons')),
        ("page-order-duplicate", all_source.replace('Self::Buttons', 'Self::Overview')),
        ("page-order-omitted", all_source.replace('        Self::Buttons,\n', '')),
    ]:
        add(name, {APP: replace(APP, all_source, changed)}, "complete-production-source-inventory", **inventory)
    # author.rs is the shared external-authoring helper, not a PageId entry;
    # retain it in required source inventory but not the 22 page phase owners.
    production_pages = [path for path in pages if path not in {"apps/showcase/src/pages/mod.rs", "apps/showcase/src/pages/author.rs"}]
    require(len(production_pages) == 22, "exact source-owned page module count changed")
    for path in production_pages:
        for phase in ("update", "draw"):
            pattern = r'\bfn ' + phase + r'\s*\('
            text = original[path].decode()
            require(re.search(pattern, text), "actual page has no " + phase + ": " + path)
            changed = re.sub(pattern, 'fn qualification_removed_' + phase + '(', text, count=1)
            add("page-" + Path(path).stem + "-missing-" + phase, {path: changed}, "complete-production-source-inventory", **inventory)
    add("missing-due-grid", {GRID: None}, "grid-capability-syntax", paths=[GRID])
    for label, suffix in [
        ("generic-multiline-editable", "\nimpl<'a> Grid<'a> { pub fn editable< T >\n(self, enabled: T) -> Self { self } }\n"),
        ("inline-grid-actions", "\nmod hidden { pub trait GridCellActions {} }\n"),
    ]:
        add(label, {GRID: original[GRID] + suffix.encode()}, "grid-capability-syntax", paths=[GRID])
    add("grid-syntax-positive", {}, "grid-capability-syntax", positive=True, paths=[GRID])
    # Root attributes and token structure come from the actual parsed library;
    # comment text must not replace the crate's real unsafe prohibition.
    library = "crates/tui/src/lib.rs"
    add("unsafe-comment-only", {library: replace(library, '#![forbid(unsafe_code)]', '// #![forbid(unsafe_code)]')}, "unsafe-syntax", paths=[library])
    add("unsafe-split-tokens", {library: original[library] + b'\nunsafe\nimpl Send for QualificationUnsafe {}\nstruct QualificationUnsafe;\n'}, "unsafe-syntax", paths=[library])
    add("unsafe-positive", {}, "unsafe-syntax", positive=True, paths=[library])
    button = "crates/tui/src/components/button.rs"
    headings = re.findall(r'^/// ## ([^\n]+)$', original[button].decode(), flags=re.M)
    require(len(headings) >= 15 and len(headings) == len(set(headings)), "actual public Button doc headings incomplete")
    documentation = {"public_component": "Button", "required_rustdoc_headings": headings}
    add("public-doc-positive", {}, "public-component-rustdoc-sections", positive=True, paths=[button], **documentation)
    for heading in headings:
        add("missing-doc-" + heading.lower(), {button: replace(button, '/// ## ' + heading + '\n', '')},
            "public-component-rustdoc-sections", paths=[button], **documentation)
    add("todo-self-name", {APP: original[APP] + b'\nfn no_todo_or_unimplemented_probe() { todo!("no_todo_or_unimplemented"); }\n'}, "production-todo-syntax", paths=[APP])
    add("todo-comment-positive", {APP: original[APP] + b'\n// no_todo_or_unimplemented: todo! is a diagnostic example, not an invocation.\n'}, "production-todo-syntax", positive=True, paths=[APP])
    legacy = "crates/tui/tests/allow/legacy_api.txt"
    add("dispatch-legacy-allow", {legacy: 'apps/showcase/src/app.rs:1  # retained legacy owns route\n'}, "legacy-dispatch-exhaustion", paths=[APP, legacy])
    for operation in ("owns", "locate", "child"):
        suffix = f'\nfn qualification_legacy(cx: &junie_tui::Cx, id: junie_tui::Id) {{ let _ = cx.{operation}(id); }}\n'
        add("dispatch-source-" + operation, {APP: original[APP] + suffix.encode()}, "legacy-dispatch-exhaustion", paths=[APP, legacy])
    add("dispatch-scrollbar-id", {APP: original[APP] + b'\nfn qualification_legacy() { let _ = scrollbar::id_for(0); }\n'}, "legacy-dispatch-exhaustion", paths=[APP, legacy])
    add("dispatch-free-helper-positive", {APP: original[APP] + b'\nfn owns(_id: junie_tui::Id) -> bool { false }\n', legacy: b''}, "legacy-dispatch-exhaustion", positive=True, paths=[APP, legacy])
    add("dispatch-exhausted-positive", {legacy: b''}, "legacy-dispatch-exhaustion", positive=True, paths=[APP, legacy])
    # Compare one real joined gate across actual CHECKS, test wrapper and its
    # authoritative table. The profile scopes this join to one known-present
    # gate; it does not falsely certify all unfinished main registry rows.
    gate = "no_todo_or_unimplemented"
    dispatch = f'    ("{gate}", {gate}),\n'
    wrapper = f'    #[test]\n    fn {gate}() {{\n        check("{gate}");\n    }}\n'
    docline = next(line for line in original[DOC].decode().splitlines(keepends=True) if line.startswith('| `architecture::' + gate + '`'))
    registry_source = original[XTASK].decode().split('const CHECKS: &[Check] = &[', 1)[1].split('\n];', 1)[0]
    registry_names = re.findall(r'"([a-z_]+)"', registry_source)
    require(registry_names and len(registry_names) == len(set(registry_names)), "actual CHECKS registry names ambiguous")
    joining = {"joined_gates": [gate], "required_registry_names": registry_names,
               "registry": "CHECKS", "wrapper_module": "architecture", "documentation_section": "16.5"}
    add("gate-join-positive", {}, "gate-dispatch-wrapper-table-join", positive=True, paths=[XTASK, WRAPPER, DOC], **joining)
    for name, path, before in [("missing-gate-dispatch", XTASK, dispatch), ("missing-gate-wrapper", WRAPPER, wrapper), ("missing-gate-table", DOC, docline)]:
        add(name, {path: replace(path, before, '')}, "gate-dispatch-wrapper-table-join", paths=[XTASK, WRAPPER, DOC], **joining)
    add("duplicate-gate-dispatch", {XTASK: replace(XTASK, dispatch, dispatch + dispatch)}, "gate-dispatch-wrapper-table-join", paths=[XTASK, WRAPPER, DOC], **joining)
    add("mismatched-gate-dispatch", {XTASK: replace(XTASK, dispatch, f'    ("{gate}", no_unsafe),\n')}, "gate-dispatch-wrapper-table-join", paths=[XTASK, WRAPPER, DOC], **joining)
    add("extra-gate-dispatch", {XTASK: replace(XTASK, dispatch, dispatch + '    ("qualification_extra_gate", no_unsafe),\n')}, "gate-dispatch-wrapper-table-join", paths=[XTASK, WRAPPER, DOC], **joining)
    return cases


def observation_premise(api, fixture, record):
    """Require real complete parser transport, never a source-label verdict."""
    api.require(record["exit"] == 0, "source parser process failed: " + api.decode(record, "stderr").decode()[-3000:])
    payload = json.loads(api.decode(record, "stdout"))
    api.require(payload["schema"] == "tc-protected-source-syntax/v1", "source parser ABI")
    api.require([entry["path"] for entry in payload["files"]] == fixture.row["observed_paths"], "source parser omitted or reordered inputs")
    for entry in payload["files"]:
        source = fixture.row["source"].get(entry["path"])
        if source is None:
            api.require(entry.get("read_error") == "NotFound", "absent file observation fabricated or unrelated IO failure")
        else:
            api.require(entry.get("source", "").encode() == source, "source parser observed different bytes")
            if entry["path"].endswith(".rs"):
                api.require("facts" in entry or "parse_error" in entry, "actual Rust AST observation absent")
