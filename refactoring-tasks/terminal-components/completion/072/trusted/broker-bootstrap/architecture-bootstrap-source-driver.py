#!/usr/bin/env python3
"""Protected actual-TC external-consumer and Cargo-graph qualification.

This is an independent test instrument, not a production architecture checker.
Expected categories stay private. Compiler/metadata observations are obtained
after the submitted command asks the external observer to execute.
"""
from __future__ import annotations

import argparse
import base64
from contextlib import contextmanager
import importlib.util
import json
import os
from pathlib import Path
import re
import secrets
import subprocess
import sys
import tempfile
from types import SimpleNamespace

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("source_actual", HERE / "architecture-bootstrap-actual-driver.py")
transport = importlib.util.module_from_spec(spec)
spec.loader.exec_module(transport)
actual, runner = transport.actual, transport.runner
require, sha, canonical = runner.require, runner.sha, runner.canonical
policy_spec = importlib.util.spec_from_file_location("source_policy", HERE / "architecture-bootstrap-source-policy.py")
policy = importlib.util.module_from_spec(policy_spec)
policy_spec.loader.exec_module(policy)
VALID = 'use junie_tui::{Button, Id};\nfn main() { let _ = Button::new(Id::root("qualified"), "Go"); }\n'
MUTANTS = {
    "wrong-arity": (VALID.replace(', "Go"', ''), "E0061"),
    "private-method": (VALID.replace('; }', '.can_activate(); }'), "E0624"),
    "absent-public-api": (VALID.replace("Button, Id", "DefinitelyAbsent, Id").replace("Button::new", "DefinitelyAbsent::new"), "E0432"),
}


def command(argv, cwd, env=None):
    identity = actual.compiler_identity()
    is_compiler = str(argv[0]) in {tool["path"] for tool in identity.values()}
    if is_compiler:
        actual.validate_compiler_identity(identity)
        env = actual.compiler_environment(identity, env)
    result = subprocess.run(argv, cwd=cwd, env=env, stdout=subprocess.PIPE,
                            stderr=subprocess.PIPE, timeout=300, check=False)
    if is_compiler:
        actual.validate_compiler_identity(identity)
    return {"argv": list(map(str, argv)), "exit": result.returncode,
            "stdout": base64.b64encode(result.stdout).decode(),
            "stderr": base64.b64encode(result.stderr).decode(),
            "stdout_sha256": sha(result.stdout), "stderr_sha256": sha(result.stderr)}


def decode(record, stream):
    return base64.b64decode(record[stream])


def compiler_codes(record):
    codes = []
    for line in decode(record, "stderr").splitlines():
        value = json.loads(line)
        if value.get("level") == "error" and value.get("code"):
            codes.append(value["code"]["code"])
    return codes


def public_cases(original):
    """Actual public TC API consumers; no copied implementation or mock API."""
    cases = [{"name": "public-valid", "files": {"consumer.rs": VALID.encode()}, "category": None}]
    for name, (source, error) in MUTANTS.items():
        cases.append({"name": name, "files": {"consumer.rs": source.encode()},
                      "category": "ARCHITECTURE", "compiler_error": error})
    grid = ('use junie_tui::{Grid, GridState, GridModel, GridEditor, Cx};\n'
            'fn read<M: GridModel + ?Sized>(g: &Grid, cx: &mut Cx, st: &mut GridState, m: &M) { let _ = g.update(cx, st, m); }\n'
            'fn edit<M: GridEditor + ?Sized>(g: &Grid, cx: &mut Cx, st: &mut GridState, m: &mut M) { let _ = g.update_editable(cx, st, m); }\n'
            'fn main() {}\n')
    for name, source, error in [
        ("grid-api-valid", grid, None),
        ("grid-api-shared-editor", grid.replace('g.update(cx, st, m)', 'g.update_editable(cx, st, m)'), "E0308"),
        ("grid-api-boolean-capability", grid + 'fn forbidden(g: Grid) { let _ = g.editable(true); }\n', "E0599"),
    ]:
        cases.append({"name": name, "files": {"consumer.rs": source.encode()}, "consumer_family": "grid-public-capability",
                      "category": "ARCHITECTURE" if error else None, **({"compiler_error": error} if error else {})})
    testing = ('use junie_tui::{Theme, ColorLevel, Rect};\n'
               'use junie_tui_testing::{Harness, NoApp, Scene, Caps, Fixture};\n'
               'fn fixture_contract(_: Option<Fixture>) {}\n'
               'fn main() { let mut h = Harness::new(NoApp, Theme::junie(), 12, 4); '
               'let _ = Scene::new("qualified", Theme::junie(), ColorLevel::TrueColor, 12, 4); '
               'let _ = Caps::empty(); let _ = &mut h; }\n')
    for name, source, error in [
        ("testing-valid", testing, None),
        ("testing-arity", testing.replace('Theme::junie(), 12, 4', 'Theme::junie(), 12'), "E0061"),
        ("testing-private", testing.replace('let _ = &mut h;', 'h.click_rect(Rect::new(0,0,1,1));'), "E0624"),
        ("testing-absent", testing.replace('Caps, Fixture', 'AbsentTestingApi, Fixture').replace('Caps::empty()', 'AbsentTestingApi::empty()'), "E0432"),
    ]:
        cases.append({"name": name, "files": {"consumer.rs": source.encode()}, "testing": True,
                      "category": "ARCHITECTURE" if error else None, **({"compiler_error": error} if error else {})})
    # Both historical heading forms and a section beyond the actual latest one.
    architecture = original["COMPONENT_ARCHITECTURE.md"].decode()
    sections = [int(value) for value in re.findall(r"(?m)^## (\d+)[. ]", architecture)]
    require(sections and max(sections) >= 29, "actual numbered architecture sections missing")
    for section in [18, 19, 20, 29, max(sections) + 1]:
        for heading in (f"{section}. Public API", f"§{section} Public API"):
            for broken in (False, True):
                # This scoped consumer document uses real Rust fences, not a
                # pre-extracted test name or a verdict hidden in JSON.
                snippet = MUTANTS["wrong-arity"][0] if broken else VALID
                document = f"# CHECKS\n\n## {heading}\n\n```rust\n{snippet}```\n"
                cases.append({"name": f"section-{heading}-{broken}",
                              "files": {"CHECKS.md": document.encode()},
                              "category": "ARCHITECTURE" if broken else None,
                              **({"compiler_error": "E0061"} if broken else {})})
    # Rust ignore/no_run cannot silently remove a required API reference.
    for marker in ("rust,ignore", "rust,no_run"):
        cases.append({"name": "ignored-reference-" + marker,
                      "files": {"CHECKS.md": ("## 29. Required API\n\n```" + marker + "\n" + MUTANTS["absent-public-api"][0] + "```\n").encode()},
                      "category": "ARCHITECTURE", "compiler_error": "E0432"})
    live = 'missing-public-api\tjunie_tui::DefinitelyAbsent\tconsumer.rs\tTASK-068\t20260911\n'
    for name, allowance, source, error, category in [
        ("live", live, MUTANTS["absent-public-api"][0], "E0432", None),
        ("duplicate", live + live, MUTANTS["absent-public-api"][0], "E0432", "ARCHITECTURE"),
        ("expired", live.replace('20260911', '20260909'), MUTANTS["absent-public-api"][0], "E0432", "ARCHITECTURE"),
        ("missing-owner", live.replace('TASK-068', ''), MUTANTS["absent-public-api"][0], "E0432", "ARCHITECTURE"),
        ("missing-kind", live.replace('missing-public-api', ''), MUTANTS["absent-public-api"][0], "E0432", "ARCHITECTURE"),
        ("missing-expiry", live.replace('20260911', ''), MUTANTS["absent-public-api"][0], "E0432", "ARCHITECTURE"),
        ("malformed", 'DefinitelyAbsent ignored forever\n', MUTANTS["absent-public-api"][0], "E0432", "ARCHITECTURE"),
        ("zero-hit", live, VALID, None, "ARCHITECTURE"),
        ("satisfied", live.replace('DefinitelyAbsent', 'Button'), VALID, None, "ARCHITECTURE"),
        ("blanket-type", live.replace('junie_tui::DefinitelyAbsent', 'junie_tui::*'), MUTANTS["absent-public-api"][0], "E0432", "ARCHITECTURE"),
        ("wrong-source", live.replace('consumer.rs', 'foreign.rs'), MUTANTS["absent-public-api"][0], "E0432", "ARCHITECTURE"),
    ]:
        cases.append({"name": "allow-" + name, "files": {"consumer.rs": source.encode(), "doc_check_allow.tsv": allowance.encode()},
                      "category": category, "allow_policy": True, **({"compiler_error": error} if error else {})})
    return cases


def consumers(files):
    result = []
    for path, data in sorted(files.items()):
        if path.endswith(".rs"):
            result.append((path, data))
        elif path.endswith(".md"):
            blocks = re.findall(rb"(?m)^```rust[^\n]*\n(.*?)^```\s*$", data, flags=re.S)
            require(blocks, "qualification document lacks an actual Rust fence")
            result.extend((f"{path}#{index}", block) for index, block in enumerate(blocks))
    require(result, "empty consumer set")
    return result


def dependency_source(original, mode, headless=False):
    source = dict(original)
    # A real source-copy Cargo workspace. The bridge tests transitive traversal,
    # while the untouched testing crate supplies an additional legal root.
    workspace = source["Cargo.toml"].decode().replace('members = [\n', 'members = [\n    "qualification-bridge",\n', 1)
    source["Cargo.toml"] = workspace.encode()
    source["qualification-bridge/Cargo.toml"] = b'[package]\nname="qualification-bridge"\nversion="0.0.0"\nedition="2024"\n[dependencies]\n'
    source["qualification-bridge/src/lib.rs"] = b'pub fn bridge() {}\n'
    if mode in {"direct", "transitive", "permitted-only", "build-only", "testing-only"}:
        source["qualification-bridge/Cargo.toml"] += b'showcase={path="../apps/showcase"}\n'
        # Avoid a cycle so a graph traversal, not generic Cargo failure, must
        # detect the forbidden application reachability from junie-tui.
        app = source["apps/showcase/Cargo.toml"].decode()
        app = app.replace('junie-tui = { workspace = true, features = ["crossterm"] }\n', '').replace('junie-tui-testing.workspace = true\n', '')
        source["apps/showcase/Cargo.toml"] = app.encode()
        dependency = 'showcase = { path = "../../apps/showcase" }' if mode == "direct" else 'qualification-bridge = { path = "../../qualification-bridge" }'
        if mode == "testing-only":
            source["crates/tui-testing/Cargo.toml"] = source["crates/tui-testing/Cargo.toml"].replace(b'[dependencies]\n', b'[dependencies]\nshowcase = { path = "../../apps/showcase" }\n', 1)
        elif mode == "build-only":
            source["crates/tui/Cargo.toml"] += ('\n[build-dependencies]\n' + dependency + '\n').encode()
        elif mode not in {"permitted-only", "testing-only"}:
            source["crates/tui/Cargo.toml"] = source["crates/tui/Cargo.toml"].replace(b'[dependencies]\n', ('[dependencies]\n' + dependency + '\n').encode(), 1)
    elif mode in {"direct-smallvec", "direct-crossterm"}:
        dependency = 'smallvec = "=1.16.0"' if mode == "direct-smallvec" else 'crossterm = "=0.29.0"'
        source["crates/tui/Cargo.toml"] = source["crates/tui/Cargo.toml"].replace(b'[dependencies]\n', ('[dependencies]\n' + dependency + '\n').encode(), 1)
    elif mode.startswith(("forbidden-", "duplicate-", "unreachable-duplicate-")):
        package = mode.removeprefix("unreachable-").split('-', 1)[1]
        version = "0.0.1" if mode.startswith("forbidden-") else "999.0.0"
        source["Cargo.toml"] = source["Cargo.toml"].replace(b'members = [\n', b'members = [\n    "qualification-dependency",\n', 1)
        source["qualification-dependency/Cargo.toml"] = f'[package]\nname="{package}"\nversion="{version}"\nedition="2024"\n'.encode()
        source["qualification-dependency/src/lib.rs"] = b'pub fn source_fixture() {}\n'
        if not mode.startswith("unreachable-"):
            source["qualification-bridge/Cargo.toml"] += f'qualification-dependency={{package="{package}",path="../qualification-dependency"}}\n'.encode()
        source["crates/tui/Cargo.toml"] = source["crates/tui/Cargo.toml"].replace(b'[dependencies]\n', b'[dependencies]\nqualification-bridge = { path = "../../qualification-bridge" }\n', 1)
    elif mode == "missing-core-feature":
        source["Cargo.toml"] = actual.replace_once(source["Cargo.toml"].decode(), 'features = ["std", "underline-color"]', 'features = ["std"]').encode()
    elif mode == "bridge-valid":
        source["crates/tui/Cargo.toml"] = source["crates/tui/Cargo.toml"].replace(b'[dependencies]\n', b'[dependencies]\nqualification-bridge = { path = "../../qualification-bridge" }\n', 1)
    if headless:
        for path in source:
            if path.startswith("apps/") and path.endswith("Cargo.toml"):
                source[path] = source[path].replace(b'features = ["crossterm"]', b'features = []')
    return source


def graph_paths(metadata, root_name, target_name, *, ids=False):
    by_id = {package["id"]: package for package in metadata["packages"]}
    nodes = {node["id"]: node for node in metadata["resolve"]["nodes"]}
    roots = [key for key, value in by_id.items() if value["name"] == root_name]
    require(len(roots) == 1, "unique dependency root missing")
    paths, pending = [], [(roots[0], [])]
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


def graph_violations(metadata, contract):
    """Private premise checks over actual Cargo output; never emitted verdicts."""
    failures = []
    by_name = {}
    for package in metadata["packages"]:
        by_name.setdefault(package["name"], []).append(package)
    core = by_name["junie-tui"][0]
    root_name = contract.get("package", "junie-tui")
    direct = {dependency["name"] for dependency in core["dependencies"] if dependency["kind"] is None}
    if "normal_direct" in contract and direct != set(contract["normal_direct"]):
        failures.append("direct-dependencies")
    for name in contract.get("forbidden_closure", []):
        if name in by_name and any(graph_paths(metadata, root, name) for root in contract.get("packages", [root_name])):
            failures.append("forbidden-" + name)
    for name in contract.get("single_version", []):
        reachable_ids = {path[-1] for path in graph_paths(metadata, root_name, name, ids=True)}
        reachable = [package for package in by_name.get(name, []) if package["id"] in reachable_ids]
        if len({package["version"] for package in reachable}) != 1:
            failures.append("duplicate-" + name)
    for name in contract.get("backend_only", []):
        if name in by_name and any("ratatui-crossterm" not in path for path in graph_paths(metadata, root_name, name)):
            failures.append("unowned-backend-" + name)
    ratatui_core = by_name["ratatui-core"][0]
    node = next(node for node in metadata["resolve"]["nodes"] if node["id"] == ratatui_core["id"])
    if "ratatui_core_features" in contract and set(node["features"]) != set(contract["ratatui_core_features"]):
        failures.append("core-features")
    return failures


def graph_cases():
    rows = []
    def add(name, mutation, contract, negative=False, headless=False):
        rows.append((name, mutation, contract, negative, headless))
    app = {"forbidden_closure": ["showcase"], "packages": ["junie-tui-testing", "junie-tui"]}
    for mode in ("valid", "permitted-only", "direct", "transitive", "build-only", "testing-only"):
        add(mode, mode, app, mode in {"direct", "transitive", "build-only", "testing-only"})
    direct = {"normal_direct": ["ratatui-core", "ratatui-crossterm", "unicode-width", "unicode-segmentation", "bitflags"]}
    for mode in ("valid", "direct-smallvec", "direct-crossterm"):
        add("direct-set-" + mode, mode, direct, mode != "valid")
    for name in ("ratatui", "ratatui-widgets", "ratatui-macros", "critical-section", "palette"):
        contract = {"forbidden_closure": [name]}
        if name == "palette":
            contract["package"] = "qualification-bridge"
        add("forbidden-" + name + "-valid", "bridge-valid", contract)
        add("forbidden-" + name, "forbidden-" + name, contract, True)
    for name in ("unicode-width", "unicode-segmentation", "bitflags"):
        contract = {"single_version": [name]}
        add("version-" + name + "-valid", "bridge-valid", contract)
        add("version-" + name + "-unreachable", "unreachable-duplicate-" + name, contract)
        add("duplicate-" + name, "duplicate-" + name, contract, True)
    backend = {"backend_only": ["smallvec", "parking_lot", "parking_lot_core", "lock_api", "scopeguard", "libc", "mio", "signal-hook", "signal-hook-mio"]}
    add("backend-owned-valid", "valid", backend)
    add("backend-unowned", "direct-smallvec", backend, True)
    features = {"ratatui_core_features": ["std", "underline-color"]}
    add("features-valid", "valid", features, headless=True)
    add("features-missing", "missing-core-feature", features, True, headless=True)
    add("whole-main-features-negative", "valid", features, True)
    return rows


@contextmanager
def preparation():
    original = actual.main_sources(actual.frozen_archive())
    with tempfile.TemporaryDirectory(prefix="tc-architecture-main-", dir="/tmp") as directory:
        root, target = Path(directory) / "source", Path(directory) / "target"
        actual.write_sources(root, original)
        identity = actual.compiler_identity()
        env = dict(os.environ, CARGO_TARGET_DIR=str(target), CARGO_NET_OFFLINE="true", RUSTFLAGS="--cap-lints=allow")
        build = command([identity["cargo"]["path"], "build", "--lib", "-p", "junie-tui-testing", "--locked", "--offline", "--message-format=json"], root, env)
        require(build["exit"] == 0, "pinned actual public library build failed: " + decode(build, "stderr").decode()[-2000:])
        artifacts = [value for line in decode(build, "stdout").splitlines()
                     if (value := json.loads(line)).get("reason") == "compiler-artifact" and value["target"]["name"] == "junie_tui"]
        require(len(artifacts) == 1, "unique actual public library compiler artifact missing")
        libraries = [Path(path) for path in artifacts[0]["filenames"] if path.endswith(".rlib")]
        require(len(libraries) == 1, "actual public library rlib missing")
        library = libraries[0]
        library_digest = sha(library.read_bytes())
        testing_artifacts = [value for line in decode(build, "stdout").splitlines()
                             if (value := json.loads(line)).get("reason") == "compiler-artifact" and value["target"]["name"] == "junie_tui_testing"]
        require(len(testing_artifacts) == 1, "actual testing library artifact missing")
        testing_libraries = [Path(path) for path in testing_artifacts[0]["filenames"] if path.endswith(".rlib")]
        require(len(testing_libraries) == 1, "actual testing library rlib missing")
        rows = public_cases(original)
        for row in rows:
            row.update({"kind": "external-consumers", "source": original, "library": library,
                        "library_sha256": library_digest, "dependencies": target / "debug" / "deps", "build": build, "toolchain": identity})
            if row.get("testing"):
                row.update({"testing_library": testing_libraries[0], "testing_library_sha256": sha(testing_libraries[0].read_bytes())})
        for name, mode, contract, negative, headless in graph_cases():
            source = dependency_source(original, mode, headless)
            actual.write_sources(root, source)
            # Resolve the independently constructed source-copy lock before
            # freeze; every later observation is locked and read-only.
            flags = ["--no-default-features"] if headless else []
            metadata = command([identity["cargo"]["path"], "metadata", "--format-version=1", "--offline", *flags], root, env)
            require(metadata["exit"] == 0, "real metadata fixture unresolved: " + decode(metadata, "stderr").decode()[-2000:])
            source["Cargo.lock"] = (root / "Cargo.lock").read_bytes()
            graph = json.loads(decode(metadata, "stdout"))
            paths = graph_paths(graph, "junie-tui", "showcase")
            violations = graph_violations(graph, contract)
            require(bool(violations) == negative, "actual dependency mutation ineffective: " + name + " " + repr(violations))
            if mode.startswith(("forbidden-", "duplicate-")):
                require(mode in violations, "target dependency violation did not execute")
            if mode == "transitive":
                require(["junie-tui", "qualification-bridge", "showcase"] in paths, "actual transitive dependency path missing")
            rows.append({"name": "graph-" + name, "kind": "cargo-dependencies", "source": source,
                         "files": {}, "category": "ARCHITECTURE" if negative else None, "toolchain": identity,
                         "graph_policy": contract, "metadata_flags": flags})
        api = SimpleNamespace(actual=actual, require=require, sha=sha, command=command, decode=decode)
        rows.extend(policy.prepare(api, original, root, target, identity, env))
        example = "crates/tui/examples/04_family_recipe.rs"
        for broken in (False, True):
            source = dict(original)
            if broken:
                source[example] = actual.replace_once(source[example].decode(),
                    'assert_ne!(both.style.bg, Some(t.color.accent_tint));',
                    'assert_eq!(both.style.bg, Some(t.color.accent_tint));').encode()
            actual.write_sources(root, source)
            result, compilation = actual.compile_execute(root, target, "junie-tui", "04_family_recipe",
                "tests::the_more_specific_rule_wins", kind="example")
            require(result.returncode == (101 if broken else 0), "actual specificity assertion premise changed")
            if broken:
                require(b"assertion `left == right` failed" in result.stderr, "specificity mutant failed for an unrelated reason")
            record = transport.prepared_record("specificity-reversed" if broken else "specificity-valid", source,
                {"compilation": compilation, "exit": result.returncode, "stdout_sha256": sha(result.stdout),
                 "stderr_sha256": sha(result.stderr), "_stdout": result.stdout, "_stderr": result.stderr},
                "ARCHITECTURE" if broken else None, {"kind": "external-author", "roots": [example], "entries": [example],
                    "checks": ["source", "executable-public-example"], "subject": "named-family-recipe-consumer"})
            record["kind"] = "executable-example"
            rows.append(record)
        # All expectations and sources exist before any submitted process.
        yield rows


class SourceFixture(runner.Fixture):
    def __init__(self, row):
        super().__init__("architecture", "valid")
        self.row = row
        self.subject = self.private / "source-subject"
        self.subject.mkdir()
        visible = self.public / "actual-source"
        for path, data in {**row["source"], **{"qualification-consumers/" + path: data for path, data in row["files"].items()}}.items():
            for base in (self.subject, visible):
                destination = base / path
                destination.parent.mkdir(parents=True, exist_ok=True)
                destination.write_bytes(data)
        for path, data in row["files"].items():
            destination = self.source / "qualified-consumers" / path
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(data)
        # Bind the complete source inventory into the independently committed
        # tree without copying all source files into the tiny Git fixture.
        runner.save(self.source / "qualified-source-inventory.json", {path: sha(data) for path, data in row["source"].items()})
        runner.git(self.source, "add", ".")
        self.source_tree = runner.git(self.source, "write-tree")
        profile = {"schema": "tc-architecture-source-profile/v1", "kind": row["kind"],
                   "source_directory": str(visible), "main_commit": actual.PIN,
                   "main_archive_sha256": actual.ARCHIVE_SHA256,
                   "sources": {path: sha(data) for path, data in row["source"].items()}}
        if row["kind"] == "external-consumers":
            profile.update({"consumer_directory": str(visible / "qualification-consumers"),
                            "consumer_sources": {path: sha(data) for path, data in row["files"].items()},
                            "policy": "compile every Rust consumer and every Rust fence in every numbered section, including ignore/no_run; public actual-TC API only"})
            if row.get("consumer_family"):
                profile["consumer_family"] = row["consumer_family"]
            if row.get("allow_policy"):
                profile["allowance_policy"] = {"path": "doc_check_allow.tsv", "columns": ["kind", "symbol", "source", "owner", "expires_yyyymmdd"],
                    "kind": "missing-public-api", "qualification_date": 20260910, "allowed_future_owners": ["TASK-068"],
                    "rules": "exact unresolved import and source only; no wildcards; kind owner expiry mandatory; unique nonexpired rows; every row must cover a current compiler failure; satisfied/no-hit rows reject"}
        elif row["kind"] == "cargo-dependencies":
            profile.update({"roots": [{"package": "qualification-bridge", "forbidden_package_directories": []},
                                      {"package": "junie-tui-testing", "forbidden_package_directories": ["apps"]},
                                      {"package": "junie-tui", "forbidden_package_directories": ["apps"]}],
                            "policy": "traverse normal/build dependencies from every root; never prune shared visited subgraphs across roots",
                            "core_dependency_policy": {"package": "junie-tui", "backend_owner": "ratatui-crossterm", **row["graph_policy"]},
                            "metadata_flags": row["metadata_flags"]})
        else:
            profile.update(row["policy_profile"])
            self.syntax_observer = self.private / "syntax-observer"
            self.syntax_observer.write_bytes(row["observer"])
            self.syntax_observer.chmod(0o755)
            self.syntax_inputs = self.private / "syntax-inputs.json"
            runner.save(self.syntax_inputs, row["observed_paths"])
        self.context.update({"tree": self.source_tree, "architecture_profile": profile,
                             "tool": {key: row["toolchain"]["rustc"][key] for key in ("path", "sha256")}})
        runner.save(self.context_path, self.context)
        self.context_hash = sha(self.context_path.read_bytes())
        self.before = self.snapshot()

    def launch(self, request):
        require(request == {"schema": "tc-proof-runner-observe/v1", "nonce": self.nonce,
                "operation": "architecture", "source_commit": self.source_commit, "tree": self.source_tree}, "source observer binding")
        require(not self.events, "source observer replay")
        row, records = self.row, []
        actual.validate_compiler_identity(row["toolchain"])
        if row["kind"] == "external-consumers":
            require(sha(row["library"].read_bytes()) == row["library_sha256"], "actual library changed")
            if row.get("testing"):
                require(sha(row["testing_library"].read_bytes()) == row["testing_library_sha256"], "actual testing library changed")
            for index, (origin, data) in enumerate(consumers(row["files"])):
                source = self.private / f"consumer-{index}.rs"
                source.write_bytes(data)
                record = command([row["toolchain"]["rustc"]["path"], "--edition=2024", "--error-format=json", "--emit=metadata",
                                  "--crate-name", "external_consumer", "--extern", "junie_tui=" + str(row["library"]),
                                  *(["--extern", "junie_tui_testing=" + str(row["testing_library"])] if row.get("testing") else []),
                                  "-L", "dependency=" + str(row["dependencies"]), "-o", str(self.private / f"consumer-{index}.rmeta"), str(source)], self.private)
                record.update({"origin": origin, "source_sha256": sha(data)})
                records.append(record)
            require(any(record["exit"] != 0 for record in records) == bool(row.get("compiler_error")),
                    "actual compiler premise changed for " + row["name"] + ": " + "\n".join(decode(record, "stderr").decode()[-2400:] for record in records))
            if row.get("compiler_error"):
                require(any(row["compiler_error"] in compiler_codes(record) for record in records), "actual compiler failed for unrelated reason")
            payload = {"kind": row["kind"], "records": records, "library_sha256": row["library_sha256"],
                       "library_build": row["build"], "toolchain": row["toolchain"]}
            if row.get("testing"):
                payload["testing_library_sha256"] = row["testing_library_sha256"]
        elif row["kind"] == "cargo-dependencies":
            record = command([row["toolchain"]["cargo"]["path"], "metadata", "--format-version=1", "--locked", "--offline", *row["metadata_flags"]], self.subject,
                             dict(os.environ, CARGO_NET_OFFLINE="true", CARGO_TARGET_DIR=str(self.private / "target")))
            require(record["exit"] == 0, "locked actual metadata execution failed")
            graph = json.loads(decode(record, "stdout"))
            require(bool(graph_violations(graph, row["graph_policy"])) == bool(row["category"]), "actual dependency premise changed")
            payload = {"kind": row["kind"], "records": [record], "toolchain": row["toolchain"]}
        else:
            require(sha(self.syntax_observer.read_bytes()) == row["observer_sha256"], "actual parser executable changed")
            record = command(["/usr/bin/sandbox-exec", "-p", actual.sandbox_module().sandbox(writable=[], unreadable=[self.public, self.output],
                              readable=[self.subject, self.syntax_inputs, self.syntax_observer]),
                              str(self.syntax_observer), str(self.subject), str(self.syntax_inputs)], self.private,
                             {"PATH": "/usr/bin:/bin", "LC_ALL": "C"})
            policy.observation_premise(SimpleNamespace(require=require, decode=decode), self, record)
            payload = {"kind": row["kind"], "records": [record], "parser_build": row["observer_build"],
                       "parser_sha256": row["observer_sha256"], "toolchain": row["toolchain"]}
        event = {"operation": "architecture", "run_id": self.run_id, "source_commit": self.source_commit,
                 "tree": self.source_tree, "exit": 0, "payload": payload}
        self.events.append(event)
        return event

    def validate(self, result, category):
        code, report = result
        require(len(self.events) == 1, "source qualification omitted protected execution")
        if category:
            require(code != 0 and report["status"] == "rejected" and report["category"] == category and report["outputs"] == {}, "source negative did not reject exactly")
        else:
            require(code == 0 and report["status"] == "passed" and report["category"] is None and canonical(report["outputs"]) == canonical({"observations": self.events}), "source positive rejected or forged")


def measured_premise(fixture):
    if isinstance(fixture, transport.ActualFixture):
        fixture.sandbox = actual.sandbox_module().sandbox
        fixture.launch({"schema": "tc-proof-runner-observe/v1", "nonce": fixture.nonce,
                        "operation": "architecture", "source_commit": fixture.source_commit, "tree": fixture.source_tree})
        return
    row = fixture.row
    event = fixture.launch({"schema": "tc-proof-runner-observe/v1", "nonce": fixture.nonce,
                           "operation": "architecture", "source_commit": fixture.source_commit, "tree": fixture.source_tree})
    records = event["payload"]["records"]
    if row["kind"] == "external-consumers":
        require(len(records) == len(consumers(row["files"])), "compiler consumer omitted")
        require(any(record["exit"] != 0 for record in records) == bool(row.get("compiler_error")), "actual compiler verdict contradicts fixture premise")
        if row.get("compiler_error"):
            require(any(row["compiler_error"] in compiler_codes(record) for record in records), "compiler rejection failed for an unrelated reason")
    elif row["kind"] == "cargo-dependencies":
        graph = json.loads(decode(records[0], "stdout"))
        require(bool(graph_violations(graph, row["graph_policy"])) == bool(row["category"]), "actual graph contradicts fixture premise")
    else:
        policy.observation_premise(SimpleNamespace(require=require, decode=decode), fixture, records[0])


def qualify(executable=None, self_test=False):
    with preparation() as rows:
        if self_test:
            for row in rows:
                fixture = transport.ActualFixture(row) if row["kind"] == "executable-example" else SourceFixture(row)
                try:
                    measured_premise(fixture)
                finally:
                    fixture.cleanup()
            # The transport tests below are intentionally not a reference
            # checker: they only publish a constant positive observation.
            outcomes = {"positive": 0, "always_pass_rejected": 0, "zero_rejected": 0, "forged_rejected": 0,
                        "bool_exit_rejected": 0, "float_record_exit_rejected": 0}
            for name, mode in [("public-valid", "positive"), ("wrong-arity", "always_pass"),
                               ("public-valid", "zero"), ("public-valid", "forged"),
                               ("public-valid", "bool_exit"), ("public-valid", "positive"),
                               ("public-valid", "float_record_exit"), ("public-valid", "positive")]:
                fixture = SourceFixture(next(row for row in rows if row["name"] == name))
                try:
                    candidate = fixture.public / "transport-test"
                    candidate.write_text("#!" + sys.executable + "\nimport json,os,hashlib\n" +
                        ("event=None\n" if mode == "zero" else
                         "request={'schema':'tc-proof-runner-observe/v1','nonce':os.environ['TC_PROOF_OBSERVER_NONCE'],'operation':'architecture','source_commit':os.environ['TC_PROOF_ORACLE_COMMIT'],'tree':os.environ['TC_PROOF_SOURCE_TREE']}\n"
                         "with os.fdopen(int(os.environ['TC_PROOF_OBSERVER_REQUEST_FD']),'w') as f: f.write(json.dumps(request)+'\\n'); f.flush()\n"
                         "with os.fdopen(int(os.environ['TC_PROOF_OBSERVER_RESPONSE_FD'])) as f: event=json.loads(f.readline())\n") +
                        ("event['payload']['records']=[]\n" if mode == "forged" else "") +
                        "events=[] if event is None else [event]\n"
                        "report={'schema':'tc-proof-runner-result/v1','run_id':os.environ['TC_PROOF_RUN_ID'],'operation':'architecture','context_sha256':os.environ['TC_PROOF_CONTEXT_SHA256'],'status':'passed','category':None,'observation_digests':[hashlib.sha256(json.dumps(x,sort_keys=True,separators=(',',':')).encode()).hexdigest() for x in events],'outputs':{'observations':events}}\n"
                        + ("event['exit']=False\n" if mode == "bool_exit" else "")
                        + ("event['payload']['records'][0]['exit']=0.0\n" if mode == "float_record_exit" else "") +
                        "with open(os.environ['TC_PROOF_RESULT'],'w') as f: json.dump(report,f)\n")
                    candidate.chmod(0o755)
                    try:
                        fixture.validate(fixture.run(candidate), fixture.row["category"])
                    except ValueError as error:
                        errors = {"always_pass": "source negative did not reject exactly", "zero": "source qualification omitted protected execution", "forged": "forged observation",
                                  "bool_exit": "forged observation bytes", "float_record_exit": "forged observation bytes"}
                        require(mode in errors and str(error) == errors[mode], "source transport failed for an unrelated reason: " + str(error))
                        outcomes[mode + "_rejected"] += 1
                    else:
                        require(mode == "positive", "source transport attack escaped")
                        outcomes["positive"] += 1
                finally:
                    fixture.cleanup()
            return {"source_cases": len(rows), "negative_controls": sum(bool(row["category"]) for row in rows),
                    "case_kinds": {kind: sum(row["kind"] == kind for row in rows) for kind in sorted({row["kind"] for row in rows})}, **outcomes}
        secrets.SystemRandom().shuffle(rows)
        for row in rows:
            recovery = next(item for item in rows if item["kind"] == row["kind"] and item["category"] is None and
                            bool(item.get("allow_policy")) == bool(row.get("allow_policy")) and
                            bool(item.get("testing")) == bool(row.get("testing")) and
                            item.get("consumer_family") == row.get("consumer_family") and
                            item.get("graph_policy") == row.get("graph_policy") and
                            (row["kind"] != "source-policy" or item["policy_profile"]["policy"] == row["policy_profile"]["policy"]))
            for selected in [row] + ([recovery] if row["category"] else []):
                fixture = transport.ActualFixture(selected) if selected["kind"] == "executable-example" else SourceFixture(selected)
                try:
                    fixture.validate(fixture.run(executable), selected["category"])
                finally:
                    fixture.cleanup()
        return {"source_cases": len(rows), "recoveries": sum(bool(row["category"]) for row in rows), "failures": []}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--runner", type=Path)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    require(args.self_test or (args.runner and args.runner.is_file()), "submitted dispatcher required")
    print(json.dumps(qualify(args.runner.resolve() if args.runner else None, args.self_test), sort_keys=True))


if __name__ == "__main__":
    main()
