#!/usr/bin/env python3
"""Protected actual-TC source-copy probe preparation; no production edits."""
from __future__ import annotations
import argparse
import hashlib
import gzip
from functools import lru_cache
import importlib.util
import io
import json
import os
import re
from pathlib import Path
import subprocess
import shutil
import tarfile
import tempfile

HERE = Path(__file__).resolve().parent
PIN = "7b27732a8c3c131760ec3438f641cb3c11343a42"
ARCHIVE_SHA256 = "732e40d9cf82e98e3d6375aaeecee349e8cb774abc5094dbbff145555201633a"
COMPRESSED_SHA256 = "ab9568812e86e3c8e18c1d945886d4b2387b1e175ad61ba4d60750b5d6f98119"
GRID = "apps/showcase/src/pages/grid.rs"
APP = "apps/showcase/src/app.rs"
PINS = {GRID: "7e381fc679c1092617d8e05bb3a63bb111ad770c", APP: "51ce4dfda51fea8b5ba30b8d42f864d692e97694",
        "crates/tui/src/ui/mod.rs": "8cdb284353263620d62442ac201459f289179309",
        "crates/tui/src/collection/rowui.rs": "960a03bc2c2722eac2b32448642dc1e31575b6a9",
        "crates/tui/tests/conformance.rs": "11e6e2f14a3d88ee154c0d9bf5b36dd77c7e12cf",
        "xtask/src/main.rs": "657e5ead20af60bc2489dde63fbba778793ec293",
        "Cargo.lock": "14c5dee15f8d138c305d98e24a932dc71fb513bb"}


def require(value, message):
    if not value:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def git_blob(data):
    return hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest()


def replace_once(source, before, after):
    require(source.count(before) == 1, "exact source mutation target missing or duplicated")
    return source.replace(before, after)


def main_sources(archive):
    require(sha(archive) == ARCHIVE_SHA256, "whole pinned source archive digest mismatch")
    result = {}
    with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
        for member in tar.getmembers():
            path = Path(member.name)
            require(not path.is_absolute() and ".." not in path.parts, "unsafe pinned archive path")
            if member.isfile():
                result[member.name] = tar.extractfile(member).read()
            else:
                require(member.isdir() or member.issym(), "unexpected archive node")
                # Instruction symlinks are not compilation inputs; no extraction
                # of symlinks and no recursive traversal through their targets.
    for path, blob in PINS.items():
        require(git_blob(result[path]) == blob, "main source pin mismatch: " + path)
    return result


def frozen_archive():
    compressed = (HERE / "main-source.tar.gz").read_bytes()
    require(sha(compressed) == COMPRESSED_SHA256, "protected compressed source archive digest mismatch")
    archive = gzip.decompress(compressed)
    require(sha(archive) == ARCHIVE_SHA256, "decompressed pinned source archive digest mismatch")
    return archive


def mutate(source, name):
    result = dict(source)
    grid, app = result[GRID].decode(), result[APP].decode()
    draw = "metrics().draw(ui, body, &self.state, &MetricModel);"
    if name in {"grid_uncovered", "grid_uncovered_model"}:
        start = grid.index("                paint_body(\n", grid.index(draw))
        end = grid.index("                if let Some(key) = self.selected", start)
        grid = grid[:start] + grid[end:]
    if name in {"grid_model", "grid_uncovered_model"}:
        grid = replace_once(grid, 'name: "P95 latency",', 'name: "Q7W9 metrics",')
    if name == "grid_dead":
        grid = replace_once(grid, draw, "if false { " + draw + " }")
    if name == "grid_inert":
        grid = replace_once(grid, draw, "ui.reference(None, |ui| { " + draw + " });")
    if name == "brand_removed":
        app = replace_once(app, "        shell_brand().draw(ui, shell.header);\n", "")
    if name == "status_removed":
        app = replace_once(app, "        shell_status().draw(ui, shell.footer);\n", "")
    if name in {"chrome_uncovered", "chrome_uncovered_brand"}:
        for call in ("paint_header", "paint_footer"):
            start = app.index("        " + call + "(\n", app.index("    fn draw(&self, ui:"))
            end = app.index("        );", start) + len("        );")
            app = app[:start] + app[end:]
    if name in {"brand_model", "chrome_uncovered_brand"}:
        app = replace_once(app, 'Brand::new(BRAND, "Junie")', 'Brand::new(BRAND, "Q7W9!")')
    require(name in {"main", "grid_uncovered", "grid_model", "grid_uncovered_model", "grid_dead", "grid_inert",
                     "brand_removed", "status_removed", "chrome_uncovered", "brand_model", "chrome_uncovered_brand"}, "unknown main mutant")
    result[GRID], result[APP] = grid.encode(), app.encode()
    result["apps/showcase/tests/architecture_bootstrap.rs"] = (HERE / "architecture-bootstrap-main-probe.rs").read_bytes()
    return result


def parse_capture(stdout):
    result = {}
    for line in stdout.decode().splitlines():
        # libtest may prefix the first line with the test name.
        if "ARCHFRAME|" in line:
            line = line[line.index("ARCHFRAME|"):]
        fields = line.split("|")
        if fields[0] == "ARCHFRAME":
            require(fields[1] not in result, "duplicate main frame")
            result[fields[1]] = {"width": int(fields[2]), "height": int(fields[3]), "focus": fields[4], "area": fields[5], "cells": []}
        elif fields[0] == "ARCHCELL":
            result[fields[1]]["cells"].append(fields[2:])
        elif fields[0] == "ARCHTEXT":
            result[fields[1]]["text"] = bytes.fromhex(fields[2]).decode()
    require(set(result) == {"initial", "activated"}, "production checkpoints missing")
    for frame in result.values():
        require((frame["width"], frame["height"]) == (120, 40) and len(frame["cells"]) == 4800, "full actual canonical cells missing")
        require(len({(cell[0], cell[1]) for cell in frame["cells"]}) == 4800, "cell omission/duplication")
    return result


def write_sources(root, source):
    require(root.parent.name.startswith("tc-architecture-main-"), "source replacement must stay in observer-owned temporary tree")
    if root.exists():
        for previous in root.rglob("*"):
            require(not previous.is_symlink(), "unexpected mutable source symlink")
            if previous.is_file() and previous.relative_to(root).as_posix() not in source:
                previous.unlink()
    for path, data in source.items():
        destination = root / path
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(data)
    actual = {path.relative_to(root).as_posix(): sha(path.read_bytes()) for path in root.rglob("*") if path.is_file()}
    require(actual == {path: sha(data) for path, data in source.items()}, "complete source inventory differs")


def run_subject(root, source, target):
    write_sources(root, source)
    process, compilation = compile_execute(root, target, "showcase", "architecture_bootstrap", None)
    require(process.returncode == 0, "actual source execution failed: " + process.stderr.decode()[-4000:])
    return {"compilation": compilation, "source_sha256": {path: sha(data) for path, data in source.items()},
            "stdout_sha256": sha(process.stdout), "stderr_sha256": sha(process.stderr), "exit": process.returncode,
            "frames": parse_capture(process.stdout), "_stdout": process.stdout, "_stderr": process.stderr}


def sandbox_module():
    path = HERE / "host-bootstrap-observer.py"
    if not path.is_file():
        path = HERE.parent / "runner-bootstrap" / "host-bootstrap-observer.py"
    spec = importlib.util.spec_from_file_location("actual_tc_protected_observer", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def selected_compiler_paths():
    # PATH cargo may be an MBX/rustup launcher. Its bytes do not identify the
    # selected compiler. Resolve both tools through one explicit host toolchain,
    # before entering any source-copy directory with its own toolchain override.
    rustup = shutil.which("rustup")
    require(rustup is not None, "protected compiler qualification requires rustup toolchain resolution")
    selected = subprocess.check_output([rustup, "show", "active-toolchain"], text=True, timeout=30).strip()
    require(bool(selected), "missing selected compiler toolchain")
    toolchain = selected.split()[0]
    paths = {}
    for name in ("cargo", "rustc"):
        path = Path(subprocess.check_output(
            [rustup, "which", "--toolchain", toolchain, name], text=True, timeout=30).strip()).resolve(strict=True)
        require(path.is_file() and os.access(path, os.X_OK), "selected compiler is not executable: " + name)
        paths[name] = path
    require(paths["cargo"].parent == paths["rustc"].parent, "Cargo and rustc belong to different toolchains")
    return toolchain, paths


def tool_version(path):
    return subprocess.check_output([str(path), "--version", "--verbose"], text=True, timeout=30).strip()


@lru_cache(maxsize=1)
def frozen_compiler_identity():
    toolchain, paths = selected_compiler_paths()
    return {name: {"path": str(path), "sha256": sha(path.read_bytes()),
                   "version": tool_version(path), "toolchain": toolchain}
            for name, path in paths.items()}


def validate_compiler_identity(identity):
    frozen = frozen_compiler_identity()
    require(identity == frozen, "protected compiler identity record changed")
    toolchain, paths = selected_compiler_paths()
    for name, tool in frozen.items():
        require(str(paths[name]) == tool["path"] and toolchain == tool["toolchain"],
                "protected compiler path/toolchain changed: " + name)
        require(sha(paths[name].read_bytes()) == tool["sha256"], "protected compiler bytes changed: " + name)
        require(tool_version(paths[name]) == tool["version"], "protected compiler version changed: " + name)


def compiler_identity():
    identity = {name: dict(tool) for name, tool in frozen_compiler_identity().items()}
    validate_compiler_identity(identity)
    return identity


def compiler_environment(identity, env=None):
    # Cargo must use the same recorded rustc, not an inherited compiler wrapper
    # or a source-directory rustup selection. Callers retain their frozen profile.
    return dict(os.environ if env is None else env, RUSTC=identity["rustc"]["path"],
                RUSTC_WRAPPER="", RUSTC_WORKSPACE_WRAPPER="")


def compile_execute(root, target, package, test, selected, kind="test"):
    # Trusted source compilation happens before a candidate can run. The actual
    # test executable is then launched by the independent protected observer.
    identity = compiler_identity()
    env = compiler_environment(identity, dict(os.environ, CARGO_TARGET_DIR=str(target),
        RUSTFLAGS="--cap-lints=allow", CARGO_NET_OFFLINE="true"))
    cargo = identity["cargo"]["path"]
    argv = [cargo, "test", "--no-run", "--message-format=json", "--locked", "--offline", "-p", package,
            *(["--lib"] if kind == "lib" else ["--" + kind, test])]
    build = subprocess.run(argv, cwd=root, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=300, check=False)
    validate_compiler_identity(identity)
    require(build.returncode == 0, "actual source compilation failed: " + build.stderr.decode()[-4000:])
    artifacts = []
    for line in build.stdout.splitlines():
        try:
            value = json.loads(line)
        except ValueError:
            continue
        if value.get("reason") == "compiler-artifact" and value.get("executable") and value.get("target", {}).get("name") == test:
            artifacts.append(Path(value["executable"]).resolve())
    require(len(artifacts) == 1, "exact compiler-produced test executable missing")
    executable = artifacts[0]
    executable_digest = sha(executable.read_bytes())
    worker_argv = [str(executable), *([selected] if selected else []), "--nocapture", "--test-threads=1"]
    profile = sandbox_module().sandbox(writable=[], unreadable=[])
    process = subprocess.run(["/usr/bin/sandbox-exec", "-p", profile, *worker_argv], cwd=root,
        env={"PATH": "/usr/bin:/bin", "LC_ALL": "C"}, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=60, check=False)
    require(sha(executable.read_bytes()) == executable_digest, "protected executable changed")
    compilation = {"argv": argv, "exit": build.returncode, "compiler_messages_sha256": sha(build.stdout),
                   "compiler_stderr_sha256": sha(build.stderr), "worker_argv": worker_argv,
                   "executable_sha256": executable_digest, "cargo_lock_sha256": sha((root / "Cargo.lock").read_bytes()),
                   "toolchain": identity, "build_policy": {"locked": True, "offline": True, "rustflags": "--cap-lints=allow",
                       "rustc": identity["rustc"]["path"], "rustc_wrapper": "", "rustc_workspace_wrapper": ""}}
    return process, compilation


def conformance_source(original, mutation="valid", focused=False):
    result = dict(original)
    button_path = "crates/tui/src/components/button.rs"
    if mutation == "slot_doc_drift":
        result[button_path] = replace_once(result[button_path].decode(), "/// `ICON`, `MARKER` and `LABEL`", "/// `MARKER` and `LABEL`").encode()
    docs = result[button_path].decode().split("/// ## Overrides\n", 1)[1].split("/// ## Identity\n", 1)[0]
    declaration = re.search(r"\.slot` on exactly (.*?) —", docs, flags=re.S)
    require(declaration is not None, "actual public slot declaration did not match frozen grammar")
    documented_parts = re.findall(r"`([A-Z_]+)`", declaration.group(1))
    require(documented_parts and len(documented_parts) == len(set(documented_parts)), "nonempty unique actual rustdoc slot set")
    path = "crates/tui/tests/conformance.rs"
    text = result[path].decode()
    if focused:
        # Exact immutable source slices, not a rewrite of ButtonCase behavior.
        header = text[:text.index("const PROBE:")]
        button = text[text.index("fn patch_of("):text.index("const INPUT:")]
        text = header + button + "\nconformance_suite!(\n    button => ButtonCase,\n);\n"
    text = replace_once(text, "use junie_tui_testing::{Harness, Scene, conformance_suite};", "use junie_tui_testing::{Harness, Scene};")
    probe = (HERE / "architecture-bootstrap-conformance-probe.rs").read_text()
    probe = replace_once(probe, "/* ARCHITECTURE_DOCUMENTED_BUTTON_SLOTS */", ", ".join("Part::" + name for name in documented_parts))
    text = replace_once(text, "conformance_suite!(\n", probe + "\nconformance_suite!(\n")
    if mutation == "registry_omission":
        if focused:
            text = replace_once(text, "conformance_suite!(\n    button => ButtonCase,\n);", "")
        else:
            text = replace_once(text, "    probe => ProbeCase,\n", "")
    if mutation == "state_omission":
        require(focused, "state omission is a fixed focused Button mutation")
        text = replace_once(text, "const STATES: [StateFlags; 7]", "const STATES: [StateFlags; 6]")
        text = replace_once(text, "            StateFlags::BUSY,\n", "")
    result[path] = text.encode()
    # Make the existing Rust caller-location mechanism pass transparently
    # through row/column methods. A default library row and an application
    # callback then have different real source locations, without changing IDs
    # or inventing an attribution flag on the production public API.
    path = "crates/tui/src/collection/rowui.rs"
    text = result[path].decode()
    text = re.sub(r"(?m)^(    )(pub(?:\(crate\))? )?fn ", r"\1#[track_caller]\n\1\2fn ", text)
    if mutation == "fake_row_owner":
        text = replace_once(text, "        RowUi {\n            ui,\n            owner,", '        RowUi {\n            ui,\n            owner: Id::root("architecture.fake-owner"),')
    result[path] = text.encode()
    path = "crates/tui/src/ui/mod.rs"
    text = result[path].decode()
    text = replace_once(text, "    pub fn note_styled(\n", "    #[track_caller]\n    pub fn note_styled(\n")
    text = replace_once(text, "        self.frame.styled_parts.push((owner, part));", "        let callsite = std::panic::Location::caller();\n        println!(\"ARCHSTYLE|{}|{}|{:?}|{:?}|{:?}|{:?}|{:?}\", callsite.file(), callsite.line(), owner, family, variant, part, resolved);\n        self.frame.styled_parts.push((owner, part));")
    result[path] = text.encode()
    if mutation in {"owned_extra", "declared_unreachable", "ignored_icon_slot", "measure_query"}:
        path = "crates/tui/src/components/button.rs"
        text = result[path].decode()
        if mutation == "owned_extra":
            text = replace_once(text, "        let container = style(ui, Part::CONTAINER);", "        let container = style(ui, Part::CONTAINER);\n        let _ = style(ui, Part::custom(\"architecture.extra-owned\"));")
        elif mutation == "declared_unreachable":
            text = replace_once(text, "    pub const PARTS: &'static [Part] = &[", "    pub const PARTS: &'static [Part] = &[\n        Part::custom(\"architecture.unreachable\"),")
        elif mutation == "ignored_icon_slot":
            text = replace_once(text, "if let Some(f) = ov.slot_for(Part::ICON)", "if let Some(f) = ov.slot_for(Part::ICON).filter(|_| false)")
        else:
            # A private observer fixture adds a styled query at measurement
            # through the already-public testing note API. It is not a fix or
            # an API proposal for the component.
            path = "crates/tui/tests/conformance.rs"
            text = result[path].decode()
            text = replace_once(text, '        let _ = Button::new(OWNER, "Measure only").measure(ui, junie_tui::Constraints::loose(40, 12));',
                '        let _ = Button::new(OWNER, "Measure only").measure(ui, junie_tui::Constraints::loose(40, 12));\n        let r = ui.style(Family::BUTTON, Variant::DEFAULT, Part::LABEL, StateFlags::empty());\n        ui.note_styled(OWNER, Family::BUTTON, Variant::DEFAULT, Part::LABEL, r);')
        result[path] = text.encode()
    require(mutation in {"valid", "registry_omission", "owned_extra", "declared_unreachable", "ignored_icon_slot", "measure_query", "slot_doc_drift", "state_omission", "fake_row_owner"}, "unknown conformance mutation")
    return result


def run_conformance(root, source, target, expected_failure=False):
    write_sources(root, source)
    process, compilation = compile_execute(root, target, "junie-tui", "conformance", "architecture_bootstrap")
    diagnostic = "\n".join(line for line in process.stderr.decode().splitlines() if not line.startswith("ARCHSTYLE|"))
    if expected_failure:
        expected = {
            "ignored_icon_slot": "documented slot ignored actual paint",
            "measure_query": "measurement must not create a paint query",
            "slot_doc_drift": "rustdoc slot set differs from actually paintable set",
            "state_omission": "capability-implied state was omitted",
            "fake_row_owner": "observation cannot invent a fake owner ID",
        }
        require(expected_failure in expected, "negative premise requires a fixed assertion identity")
        require(process.returncode == 101 and "panicked at" in diagnostic and expected[expected_failure] in diagnostic,
                "real Rust negative failed for the wrong reason: " + diagnostic[-2000:])
        return {"exit": process.returncode, "stderr_sha256": sha(process.stderr), "stdout_sha256": sha(process.stdout), "compilation": compilation,
                "_stdout": process.stdout, "_stderr": process.stderr}
    require(process.returncode == 0, "actual conformance probe failed: " + diagnostic[-4000:] + process.stdout.decode()[-2000:])
    rows = [line[line.index("ARCHCASE|"):] for line in process.stdout.decode().splitlines() if "ARCHCASE|" in line]
    slots = [line[line.index("ARCHSLOT|"):] for line in process.stdout.decode().splitlines() if "ARCHSLOT|" in line]
    styles = [line[line.index("ARCHSTYLE|"):] for line in process.stdout.decode().splitlines() if "ARCHSTYLE|" in line]
    require(len(slots) == 12, "actual slot matrix missing")
    row_parts = [line.split("ARCHROWPARTS|", 1)[1].split("|") for line in process.stdout.decode().splitlines() if "ARCHROWPARTS|" in line]
    require(len(row_parts) == 1, "independent row part identities missing")
    require(any("tests/conformance.rs|" in line and line.split("|")[6] == row_parts[0][0] and line.split("|")[3] == "architecture.bootstrap.owner" for line in styles), "actual RowUi source provenance or real owner missing")
    require(any("tests/conformance.rs|" in line and line.split("|")[6] == row_parts[0][1] and line.split("|")[3] == "architecture.bootstrap.owner" for line in styles), "actual ColumnsUi source provenance or real owner missing")
    require(any("collection/key.rs|" in line and line.split("|")[3] == "architecture.bootstrap.owner" for line in styles), "default library row falsely treated as caller-owned")
    hits = [line[line.index("ARCHSLOTHIT|"):] for line in process.stdout.decode().splitlines() if "ARCHSLOTHIT|" in line]
    require(len(hits) == 12, "actual slot hit/focus/activation matrix missing")
    cases = {}
    active = None
    for line in process.stdout.decode().splitlines():
        if "ARCHCASE|" in line:
            _, active, owner, parts = line[line.index("ARCHCASE|"):].split("|", 3)
            require(active not in cases, "duplicate compiled registry case")
            cases[active] = {"owner": owner, "declared": parts.strip("[]").split(", "), "owned": set(), "row": set(), "states": set()}
        elif active and "ARCHBEGIN|" in line:
            state = line[line.index("ARCHBEGIN|"):]
            require(state not in cases[active]["states"], "duplicate actual state checkpoint")
            cases[active]["states"].add(state)
        elif active and "ARCHSTYLE|" in line:
            fields = line[line.index("ARCHSTYLE|"):].split("|", 7)
            if fields[3] == cases[active]["owner"]:
                # The frozen conformance callbacks call row/column painters;
                # their track_caller location is external to the library. The
                # library's default row implementations remain library-owned.
                cases[active]["row" if fields[1].endswith("tests/conformance.rs") else "owned"].add(fields[6])
    for case in cases.values():
        case["missing"] = sorted(set(case["declared"]) - case["owned"])
        case["extra"] = sorted(case["owned"] - set(case["declared"]))
        for key in ("owned", "row", "states"):
            case[key] = sorted(case[key])
    state_vectors = json.loads((HERE / "architecture-bootstrap-state-vectors.json").read_bytes())
    require(state_vectors["main_commit"] == PIN and state_vectors["source_archive_sha256"] == ARCHIVE_SHA256, "frozen state vector source binding")
    for name, case in cases.items():
        expected = {f"ARCHBEGIN|{name}|{state}|{width}|{height}" for state in state_vectors["states"][name]
                    for width, height in state_vectors["geometries"]}
        require(set(case["states"]) == expected, "actual applicable-state/geometry corpus was omitted or expanded: " + name)
    return {"registrations": rows, "slots": slots, "slot_hits": hits, "styles": styles, "compilation": compilation,
            "cases": cases, "stdout_sha256": sha(process.stdout), "stderr_sha256": sha(process.stderr), "exit": process.returncode,
            "_stdout": process.stdout, "_stderr": process.stderr}


def nested_source(original, broken=False):
    source = conformance_source(original, focused=True)
    path = "crates/tui/src/collection/rowui.rs"
    text = source[path].decode()
    probe = (HERE / "architecture-bootstrap-nested-row-probe.rs").read_text()
    if broken:
        probe = replace_once(probe, "// ARCHITECTURE_NESTED_RETURN: independent lost-restoration fault.", "outer.owner = INNER; // actual retained RowUi carrier corrupted after nested return")
    require(text.rstrip().endswith("}"), "exact rowui private test module ending")
    final_brace = text.rfind("}")
    source[path] = (text[:final_brace] + probe + text[final_brace:]).encode()
    return source


def validate_causality(results):
    frames = {name: result["frames"] for name, result in results.items()}
    base = frames["main"]["initial"]
    require("customers" in base["text"] and "P95 latency" not in base["text"], "pinned main negative witness changed")
    require("selected metric:" in frames["main"]["activated"]["text"], "production Grid update did not run")
    require(frames["grid_model"]["initial"]["cells"] == base["cells"], "overpaint no longer hides actual Grid model")
    require("P95 latency" in frames["grid_uncovered"]["initial"]["text"], "actual generic Grid did not paint exposed metric")
    require("Q7W9 metrics" in frames["grid_uncovered_model"]["initial"]["text"], "actual generic Grid model perturbation absent")
    require(frames["grid_uncovered_model"]["initial"]["cells"] != frames["grid_uncovered"]["initial"]["cells"], "model perturbation has no canonical-cell consequence")
    for name in ("grid_dead", "grid_inert"):
        require(frames[name]["initial"]["cells"] == base["cells"], "fixed compatibility picture unexpectedly depends on live Grid")
        require(frames[name]["activated"]["text"] != frames["main"]["activated"]["text"], "dead/inert production interaction mutant ineffective")
    for name in ("brand_removed", "status_removed", "brand_model"):
        require(frames[name]["initial"]["cells"] == base["cells"], "chrome negative witness unexpectedly changed")
    require(frames["chrome_uncovered"]["initial"]["cells"] != base["cells"], "chrome overpaint removal has no actual cell consequence")
    require("Q7W9!" in frames["chrome_uncovered_brand"]["initial"]["text"], "uncovered Brand did not own perturbed text")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repository", type=Path)
    parser.add_argument("--conformance-only", action="store_true")
    parser.add_argument("--emit-states", action="store_true")
    args = parser.parse_args()
    archive = subprocess.check_output(["git", "-C", str(args.repository), "archive", PIN]) if args.repository else frozen_archive()
    original = main_sources(archive)
    names = ["main", "grid_model", "grid_uncovered", "grid_uncovered_model", "grid_dead", "grid_inert",
             "brand_removed", "status_removed", "brand_model", "chrome_uncovered", "chrome_uncovered_brand"]
    results = {}
    with tempfile.TemporaryDirectory(prefix="tc-architecture-main-", dir="/tmp") as directory:
        root = Path(directory) / "source"
        target = Path(directory) / "target"
        if not args.conformance_only and not args.emit_states:
            for name in names:
                results[name] = run_subject(root, mutate(original, name), target)
                print(json.dumps({"completed": name, "exit": results[name]["exit"]}), flush=True)
            validate_causality(results)
            print(json.dumps({"actual_tc_cases": len(results), "canonical_cells_per_checkpoint": 4800,
                              "main_pin": PIN, "source_archive_sha256": sha(archive), "causal_negative_witnesses": "passed"}))
        conformance = run_conformance(root, conformance_source(original), target)
        require(len(conformance["registrations"]) == 44, "whole actual registry did not execute")
        require(conformance["cases"]["button"]["missing"] == [] and conformance["cases"]["button"]["extra"] == [], "actual Button positive PARTS union premise failed")
        if args.emit_states:
            print(json.dumps({"schema": "tc-architecture-actual-state-vectors/v1", "main_commit": PIN,
                "source_archive_sha256": ARCHIVE_SHA256,
                "states": {name: sorted({int(state.split("|")[2]) for state in case["states"]}) for name, case in conformance["cases"].items()},
                "geometries": [[40, 12], [8, 4], [0, 0]]}, sort_keys=True))
            return
        print(json.dumps({"actual_registrations": len(conformance["registrations"]), "actual_slot_cases": len(conformance["slots"]),
                          "actual_style_observations": len(conformance["styles"]),
                          "parts_differences": {name: {key: case[key] for key in ("missing", "extra")} for name, case in conformance["cases"].items() if case["missing"] or case["extra"]}}))
        missing = run_conformance(root, conformance_source(original, "registry_omission"), target)
        require(set(conformance["cases"]) - set(missing["cases"]) == {"probe"} and len(missing["cases"]) == 43, "registry omission mutant did not remove exactly probe")
        for mutation, key in [("owned_extra", "extra"), ("declared_unreachable", "missing")]:
            altered = run_conformance(root, conformance_source(original, mutation), target)
            require(len(altered["cases"]["button"][key]) == 1 and conformance["cases"]["button"][key] == [], "PARTS ownership mutant ineffective")
            print(json.dumps({"actual_mutation": mutation, "observed_difference": altered["cases"]["button"][key]}))
        for mutation in ("ignored_icon_slot", "measure_query", "slot_doc_drift"):
            altered = run_conformance(root, conformance_source(original, mutation), target, expected_failure=mutation)
            print(json.dumps({"actual_mutation": mutation, "rust_assertion_exit": altered["exit"]}))
        focused = run_conformance(root, conformance_source(original, focused=True), target)
        require(set(focused["cases"]) == {"button"} and not focused["cases"]["button"]["missing"] and not focused["cases"]["button"]["extra"], "focused real Button positive failed")
        print(json.dumps({"actual_positive": "focused-button-registry", "states": len(focused["cases"]["button"]["states"])}))
        for mutation in ("state_omission", "fake_row_owner"):
            altered = run_conformance(root, conformance_source(original, mutation, focused=True), target, expected_failure=mutation)
            print(json.dumps({"actual_mutation": mutation, "rust_assertion_exit": altered["exit"]}))
        for broken in (False, True):
            write_sources(root, nested_source(original, broken))
            process, _ = compile_execute(root, target, "junie-tui", "junie_tui", "architecture_bootstrap_nested_owner", kind="lib")
            require((process.returncode == 0) != broken, "nested actual RowUi restoration premise failed: " + process.stderr.decode()[-1000:])
            require(b"ARCHNESTED|" in process.stdout if not broken else b"nested return leaked the inner owner" in process.stderr, "nested actual execution marker/diagnostic missing")
            print(json.dumps({"actual_nested_restoration_mutant": broken, "exit": process.returncode}))
        # Fresh original source recovery precedes legitimate extension controls.
        write_sources(root, original)
        rain = root / "apps/jackin-preview/tests/architecture_bootstrap.rs"
        rain.write_bytes((HERE / "architecture-bootstrap-rain-probe.rs").read_bytes())
        process, _ = compile_execute(root, target, "jackin-preview", "architecture_bootstrap", None)
        require(process.returncode == 0 and b"ARCHRAIN|" in process.stdout, "actual app-owned rain control failed")
        print(json.dumps({"actual_positive": "app-owned-rain", "exit": process.returncode}))
        process, _ = compile_execute(root, target, "junie-tui", "12_author_component", None, kind="example")
        require(process.returncode == 0 and b"test result: ok" in process.stdout and b"0 passed;" not in process.stdout, "actual external-author tests failed or empty")
        print(json.dumps({"actual_positive": "external-author-component", "exit": process.returncode}))


if __name__ == "__main__":
    main()
