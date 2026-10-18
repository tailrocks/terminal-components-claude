#!/usr/bin/env python3
"""Independent finite Rust qualification preparation, not a TC verifier.

The executable candidate is read-only. Expected verdicts and the compiler/runtime
observer remain planner-owned. Read the protocol's blocking limitations first.
"""
from __future__ import annotations

import argparse
import base64
import importlib.util
import json
import os
from pathlib import Path
import secrets
import shutil
import subprocess
import sys

HERE = Path(__file__).resolve().parent
runner_path = HERE / "runner-bootstrap-driver.py"
if not runner_path.is_file():
    runner_path = HERE.parent / "runner-bootstrap" / "runner-bootstrap-driver.py"
spec = importlib.util.spec_from_file_location("frozen_runner_bootstrap", runner_path)
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)
require, sha, canonical, git = runner.require, runner.sha, runner.canonical, runner.git
VECTORS = json.loads((HERE / "architecture-bootstrap-vectors.json").read_bytes())


def replace_once(source, before, after):
    require(source.count(before) == 1, "mutation target must occur exactly once")
    return source.replace(before, after)


def source_variant(mutation):
    files = {name: (HERE / "architecture-bootstrap-fixture" / name).read_text()
             for name in ("app.rs", "library.rs", "main.rs")}
    app, library = files["app.rs"], files["library.rs"]
    call = "Self::control_props(self.disabled)"
    draw = "Widget::draw(Self::control_props(self.disabled), self.value, busy, slot, ui);"
    copied = "ui.row(|ui| { ui.paint(0, 91); ui.paint(1, 48 + self.value % 10); ui.paint(2, if busy { 42 } else { 43 }); });"
    receivers = {
        "receiver_value": ("self", "(*self)"),
        "receiver_shared": ("&self", "self"),
        "receiver_mut": ("&mut self", "(&mut *self.clone())"),
        "receiver_typed_value": ("self: Self", "(*self)"),
        "receiver_typed_shared": ("self: &Self", "self"),
        "receiver_typed_mut": ("self: &mut Self", "(&mut *self.clone())"),
        "receiver_box": ("self: Box<Self>", "Box::new(*self)"),
        "receiver_rc": ("self: std::rc::Rc<Self>", "std::rc::Rc::new(*self)"),
        "receiver_arc": ("self: std::sync::Arc<Self>", "std::sync::Arc::new(*self)"),
        "receiver_pin": ("self: std::pin::Pin<&Self>", "std::pin::Pin::new(&*self)"),
    }
    if mutation in receivers:
        receiver, expression = receivers[mutation]
        # The mutable temporary preserves the read-only production draw receiver.
        expression = expression.replace("(&mut *self.clone())", "(&mut (*self))")
        app = app.replace("fn control_props(disabled: bool)", f"fn control_props({receiver}, disabled: bool)")
        app = app.replace(call, expression + ".control_props(self.disabled)")
        if mutation in {"receiver_mut", "receiver_typed_mut"}:
            app = app.replace("(&mut (*self)).control_props(self.disabled)", "(&mut { *self }).control_props(self.disabled)")
    elif mutation == "private_free":
        helper = "    fn control_props(disabled: bool) -> Props {\n        Props::new(7).disabled(disabled)\n    }\n"
        app = replace_once(app, helper, "").replace(call, "control_props(self.disabled)")
        app += "\n" + helper
    elif mutation == "explicit_data":
        app = app.replace("fn control_props(disabled: bool)", "fn control_props(this: &Self)")
        app = app.replace("Props::new(7).disabled(disabled)", "Props::new(7).disabled(this.disabled)")
        app = app.replace(call, "Self::control_props(self)")
    elif mutation == "custom_art":
        app = replace_once(app, "ui.paint(3, 64)", "ui.paint(3, 35)")
    elif mutation == "trait_projection":
        app = app.replace("Widget::draw(", "<Projection as Painter>::paint(")
        app += "\ntrait Painter { fn paint(p: Props, v: u32, b: bool, s: Option<(u32, SlotFn)>, u: &mut Ui); }\nstruct Projection;\nimpl Painter for Projection { fn paint(p: Props, v: u32, b: bool, s: Option<(u32, SlotFn)>, u: &mut Ui) { Widget::draw(p, v, b, s, u); } }\n"
    elif mutation == "public_props":
        app = app.replace("fn control_props(", "pub fn control_props(")
    elif mutation == "duplicate_props":
        app = replace_once(app, draw, draw.replace(call, "Props::new(7).disabled(self.disabled)"))
    elif mutation == "missing_update":
        app = replace_once(app, "Widget::update(Self::control_props(self.disabled), &mut self.value, ui);", "let _ = ui;")
    elif mutation == "disabled_bypass":
        library = replace_once(library, "if !props.disabled", "if true")
    elif mutation == "renamed_copy":
        app = replace_once(app, draw, "self.screen_picture(busy, ui);")
        app += "\nimpl App { fn screen_picture(&self, busy: bool, ui: &mut Ui) { " + copied + " } }\n"
    elif mutation == "dead_call":
        app = replace_once(app, draw, "if false { " + draw + " }\n        " + copied)
    elif mutation == "inert_reference":
        app = replace_once(app, draw, draw.replace(", ui);", ", &mut Ui::new(false));") + "\n        " + copied)
    elif mutation == "paint_over":
        app = replace_once(app, draw, draw + "\n        ui.row(|ui| ui.paint(1, 48 + self.value % 10));")
    elif mutation == "phase_drift":
        app = replace_once(app, "Widget::update(Self::control_props(self.disabled)", "Widget::update(Self::control_props(false)")
    elif mutation == "undeclared_part":
        library = replace_once(library, "ui.resolve(props.id, 2);", "ui.resolve(props.id, 2); ui.resolve(props.id, 3);")
    elif mutation == "unreachable_part":
        library = replace_once(library, "PARTS: &[u32] = &[0, 1, 2]", "PARTS: &[u32] = &[0, 1, 2, 3]")
    elif mutation == "row_attribution":
        library = replace_once(library, "self.owner = 2;", "self.owner = 1;")
    elif mutation == "lost_row_restore":
        library = replace_once(library, "self.owner = previous;", "let _ = previous;")
    elif mutation == "fake_owner_id":
        library = replace_once(library, "self.resolutions.push([id, part, self.owner]);", "self.resolutions.push([if self.owner == 2 { 999 } else { id }, part, self.owner]);")
    elif mutation == "registry_omission":
        library = replace_once(library, "REGISTRY: &[u32] = &[7]", "REGISTRY: &[u32] = &[]")
    elif mutation == "ignored_slot":
        library = replace_once(library, "if let Some((part, render)) = slot", "if let Some((part, render)) = slot.filter(|_| false)")
    elif mutation == "busy_ignored_slot":
        library = replace_once(library, "if let Some((part, render)) = slot", "if let Some((part, render)) = slot.filter(|_| !busy)")
    elif mutation == "slot_doc_extra":
        library = replace_once(library, "DOCUMENTED_SLOTS: &[u32] = &[1, 2]", "DOCUMENTED_SLOTS: &[u32] = &[0, 1, 2]")
    elif mutation == "parse_error":
        app += "\nfn broken( {\n"
    elif mutation == "unreferenced_parse_error":
        files["unreferenced.rs"] = "fn broken( {\n"
    elif mutation == "cfg_test_tail":
        app += "\n#[cfg(test)] mod tests {}\nfn broken( {\n"
    elif mutation == "missing_helper":
        app = app.replace(call, "Self::missing_props(self.disabled)")
    elif mutation == "dynamic_unconfigured":
        app += "\nfn dynamic(id: u32, disabled: bool) -> Props { Props::new(id).disabled(disabled) }\nfn plain() -> Props { Props::new(123) }\n"
    elif mutation == "test_fixture":
        app += "\n#[cfg(test)] mod tests { use super::*; pub fn fixture() -> Props { Props::new(7).disabled(true) } }\n"
    else:
        require(mutation in {"valid", "missing_root", "empty_root"}, "unknown mutation")
    files.update({"app.rs": app, "library.rs": library})
    return {name: value.encode() for name, value in files.items()}


def rustc_path():
    rustup = shutil.which("rustup")
    if rustup:
        return Path(subprocess.check_output([rustup, "which", "rustc"], text=True).strip()).resolve()
    value = shutil.which("rustc")
    require(value is not None, "a real Rust compiler is required")
    return Path(value).resolve()


def assert_runtime(payload, seed, custom_art=False):
    require(payload["parts"] == [0, 1, 2] and payload["slots"] == [1, 2] and payload["registry"] == [7], "declaration/registry equality")
    scenes = payload["scenes"]
    require(len(scenes) == 24, "exact state expansion")
    seen = set()
    for scene in scenes:
        key = (scene["disabled"], scene["busy"], scene["poison"], scene["slot"])
        require(key not in seen, "duplicate state")
        seen.add(key)
        disabled, busy, poison, slot = key
        require(type(disabled) is bool and type(busy) is bool and type(poison) is bool and slot in (0, 1, 2), "state axes")
        value = seed + (not disabled)
        cells = [91, 48 + value % 10 + (16 if poison else 0), 42 if busy else 43, 35 if custom_art else 64]
        if slot:
            cells[slot] = 126
        require(scene["value"] == value and scene["cells"] == cells, "actual model/cells differ")
        require(scene["draws"] == scene["updates"] == 1, "production phases did not execute exactly once")
        require(scene["resolutions"] == [[7, 0, 1], [7, 1, 1], [7, 2, 1], [7, 1, 1],
                [7, 99, 2], [7, 98, 2], [7, 97, 2], [7, 0, 1]], "real id and nested ownership provenance")
        owned = {part for ident, part, owner in scene["resolutions"] if owner == 1}
        require(owned == set(payload["parts"]), "PARTS exact component-owned union")
        writes = [[0, 91, 1], [1, 48 + value % 10 + (16 if poison else 0), 1], [2, 42 if busy else 43, 1]]
        if slot:
            writes.append([slot, 126, 1])
        writes.append([3, 35 if custom_art else 64, 2])
        require(scene["writes"] == writes, "paint-over or missing real slot write")


class RustFixture(runner.Fixture):
    def __init__(self, row):
        super().__init__("architecture", "valid")
        self.row, self.mutation = row, row["mutation"]
        files = source_variant(self.mutation)
        self.rust_source = self.private / "rust"
        self.rust_source.mkdir()
        visible = self.public / "source"
        visible.mkdir()
        for name, data in files.items():
            (self.rust_source / name).write_bytes(data)
            # The harness is not a candidate-provided executable. Source byte
            # identity is visible so the candidate can verify the build binding.
            (visible / name).write_bytes(data)
            (self.source / name).write_bytes(data)
        git(self.source, "add", "*.rs")
        git(self.source, "-c", "user.name=Qualification", "-c", "user.email=qualification@example.invalid",
            "-c", "commit.gpgsign=false", "commit", "-q", "-s", "-m", "Independent Rust qualification source",
            "-m", "Co-authored-by: Codex <codex@openai.com>")
        self.source_tree = git(self.source, "rev-parse", "HEAD^{tree}")
        self.seed = secrets.randbelow(9000) + 100
        self.compiler = rustc_path()
        self.executable = self.private / "rust-subject"
        argv = [str(self.compiler), "--edition=2024", "--cap-lints=allow", str(self.rust_source / "main.rs"), "-o", str(self.executable)]
        compile_result = subprocess.run(argv, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=60, check=False)
        self.compilation = {"argv": argv, "compiler_sha256": sha(self.compiler.read_bytes()),
                            "version": subprocess.check_output([str(self.compiler), "--version", "--verbose"], text=True),
                            "sources": {name: sha(data) for name, data in sorted(files.items())},
                            "exit": compile_result.returncode, "stdout": base64.b64encode(compile_result.stdout).decode(),
                            "stderr": base64.b64encode(compile_result.stderr).decode(),
                            "executable_sha256": sha(self.executable.read_bytes()) if self.executable.is_file() else None}
        require((compile_result.returncode == 0) == row["rust_compiles"], "fixture compiler premise failed: " + row["id"])
        roots = [str(visible)]
        if self.mutation == "missing_root":
            roots.append(str(self.public / "absent"))
        elif self.mutation == "empty_root":
            (self.public / "empty").mkdir()
            roots.append(str(self.public / "empty"))
        self.context.update({"tree": self.source_tree, "tool": {"path": str(self.compiler), "sha256": sha(self.compiler.read_bytes())},
            "architecture_profile": {"schema": "tc-architecture-rust-profile/v1", "source_roots": roots,
                "entry": str(visible / "app.rs"), "library": str(visible / "library.rs"),
                "sources": self.compilation["sources"], "phases": ["App::update", "App::draw"],
                "props": "Props", "component": "Widget", "id": 7,
                "registry": "REGISTRY", "parts": "PARTS", "slots": "DOCUMENTED_SLOTS",
                "ownership": {"component": 1, "row": 2}, "seed": self.seed}})
        runner.save(self.context_path, self.context)
        self.context_hash = sha(self.context_path.read_bytes())
        self.before = self.snapshot()

    def launch(self, request):
        require(set(request) == {"schema", "nonce", "operation", "source_commit", "tree"}, "observer request fields")
        require(request == {"schema": "tc-proof-runner-observe/v1", "nonce": self.nonce, "operation": "architecture",
                            "source_commit": self.source_commit, "tree": self.source_tree}, "observer identity")
        require(not self.events, "observer replay")
        if self.compilation["exit"] == 0:
            require(sha(self.executable.read_bytes()) == self.compilation["executable_sha256"], "compiled executable changed")
            profile = self.sandbox(writable=[], unreadable=[self.public, self.output])
            result = subprocess.run(["/usr/bin/sandbox-exec", "-p", profile, str(self.executable), str(self.seed)],
                                    stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=20, check=False,
                                    env={"PATH": "/usr/bin:/bin", "LC_ALL": "C"})
            code, stdout, stderr = result.returncode, result.stdout, result.stderr
            payload = runner.load_bytes(stdout) if code == 0 else None
        else:
            code, stdout, stderr, payload = self.compilation["exit"], b"", b"", None
        event = {"operation": "architecture", "run_id": self.run_id, "tree": self.source_tree,
                 "source_commit": self.source_commit, "compilation": self.compilation, "exit": code,
                 "payload": payload, "stdout_sha256": sha(stdout), "stderr": base64.b64encode(stderr).decode()}
        self.events.append(event)
        return event

    def validate(self, result, category):
        code, report = result
        require(len(self.events) == 1, "source verdict lacks independent compiler/runtime observation")
        self.validate_premise()
        if category:
            require(code != 0 and report["status"] == "rejected" and report["category"] == category and report["outputs"] == {}, "negative did not reject exactly")
        else:
            require(code == 0 and report["status"] == "passed" and report["category"] is None and
                    canonical(report["outputs"]) == canonical({"observations": self.events}), "valid source rejected or output fabricated")

    def validate_premise(self):
        event = self.events[0]
        if self.row["runtime"] == "none":
            require(event["exit"] != 0 and event["payload"] is None, "compiler rejection missing")
            return
        require(event["exit"] == 0, "Rust execution failed")
        try:
            assert_runtime(event["payload"], self.seed, self.mutation == "custom_art")
        except (ValueError, KeyError, TypeError):
            require(self.row["runtime"] == "invalid", "valid Rust behavior failed independent expectation")
        else:
            require(self.row["runtime"] == "valid", "runtime mutation ineffective")


def host_sandbox():
    path = runner_path.parent / "host-bootstrap-observer.py"
    module_spec = importlib.util.spec_from_file_location("frozen_architecture_host", path)
    module = importlib.util.module_from_spec(module_spec)
    module_spec.loader.exec_module(module)
    return module.sandbox


def self_test():
    require(sys.platform == "darwin", "protected Darwin execution required")
    counts = {"compiled": 0, "compiler_rejections": 0, "runtime_mutants": 0, "source_only_mutants": 0}
    frozen = {name: sha(data) for name, data in source_variant("valid").items()}
    for row in VECTORS["cases"]:
        fixture = RustFixture(row)
        fixture.sandbox = host_sandbox()
        try:
            fixture.launch({"schema": "tc-proof-runner-observe/v1", "nonce": fixture.nonce,
                            "operation": "architecture", "source_commit": fixture.source_commit, "tree": fixture.source_tree})
            fixture.validate_premise()
            counts["compiled" if row["rust_compiles"] else "compiler_rejections"] += 1
            counts["runtime_mutants"] += row["runtime"] == "invalid"
            counts["source_only_mutants"] += row["runtime"] == "valid" and row["expect"] is not None
        finally:
            fixture.cleanup()
        require(frozen == {name: sha(data) for name, data in source_variant("valid").items()}, "mutation did not reverse to exact frozen valid bytes")
    fixture = RustFixture(VECTORS["cases"][0])
    try:
        liar = fixture.public / "liar"
        liar.write_text("#!" + sys.executable + "\nimport json,os\n"
            "r={'schema':'tc-proof-runner-result/v1','run_id':os.environ['TC_PROOF_RUN_ID'],"
            "'operation':'architecture','context_sha256':os.environ['TC_PROOF_CONTEXT_SHA256'],"
            "'status':'passed','category':None,'observation_digests':[],'outputs':{'observations':[]}}\n"
            "with open(os.environ['TC_PROOF_RESULT'],'w') as f: json.dump(r,f)\n")
        liar.chmod(0o755)
        try:
            fixture.validate(fixture.run(liar), None)
        except ValueError:
            pass
        else:
            raise ValueError("zero-observer candidate accepted")
    finally:
        fixture.cleanup()
    return {"cases": len(VECTORS["cases"]), **counts, "zero_observer_rejected": True,
            "mutation_reversal": "exact frozen bytes", "ready_for_072_dispatch": VECTORS["ready_for_072_dispatch"],
            "hard_uncovered": VECTORS["hard_uncovered"]}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--runner", type=Path)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        print(json.dumps(self_test(), sort_keys=True))
        return
    require(args.runner is not None and args.runner.is_file(), "a built submitted architecture dispatcher is required")
    require(VECTORS["ready_for_072_dispatch"], "hard Rust/TC coverage requirements remain uncovered; TASK072 dispatch is not qualified")
    rows = list(VECTORS["cases"])
    secrets.SystemRandom().shuffle(rows)
    for row in rows:
        for selected in [row] + ([rows[0] if rows[0]["mutation"] == "valid" else VECTORS["cases"][0]] if row["expect"] else []):
            fixture = RustFixture(selected)
            try:
                fixture.validate(fixture.run(args.runner.resolve()), selected["expect"])
            finally:
                fixture.cleanup()
    actual_spec = importlib.util.spec_from_file_location("frozen_actual_architecture", HERE / "architecture-bootstrap-actual-driver.py")
    actual = importlib.util.module_from_spec(actual_spec)
    actual_spec.loader.exec_module(actual)
    actual_result = actual.qualify(args.runner.resolve())
    print(json.dumps({"standalone_cases": len(rows), "actual": actual_result, "failures": []}))


if __name__ == "__main__":
    main()
