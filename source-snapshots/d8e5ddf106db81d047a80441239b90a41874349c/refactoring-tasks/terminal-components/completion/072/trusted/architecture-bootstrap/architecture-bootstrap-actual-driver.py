#!/usr/bin/env python3
"""Independent actual-TC qualification transport, not a production checker."""
from __future__ import annotations
import argparse
import base64
import difflib
import importlib.util
import json
import re
from pathlib import Path
import secrets
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent


def module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    loaded = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(loaded)
    return loaded


actual = module("frozen_actual_tc", HERE / "architecture-bootstrap-main-driver.py")
runner_path = HERE / "runner-bootstrap-driver.py"
if not runner_path.is_file():
    runner_path = HERE.parent / "runner-bootstrap" / "runner-bootstrap-driver.py"
runner = module("frozen_actual_runner", runner_path)
require, sha, canonical = runner.require, runner.sha, runner.canonical


def semantic_output(data):
    # Measured libtest process identities are not source/runtime semantics.
    # Preserve test name, source location, assertion and every observation.
    data = re.sub(rb"(?m)^(test result: [^\n]*; )finished in [0-9.]+s$", rb"\1finished in <duration>s", data)
    return re.sub(rb"(?m)^(thread '[^'\n]+' )\([0-9]+\)( panicked at )", rb"\1(<thread-id>)\2", data)


def prepared_record(name, source, result, category, profile):
    compilation = result["compilation"]
    executable = Path(compilation["worker_argv"][0])
    data = executable.read_bytes()
    require(sha(data) == compilation["executable_sha256"], "compiler artifact changed before freeze")
    return {"name": name, "source": source, "executable": data,
            "worker_args": compilation["worker_argv"][1:], "category": category,
            "profile": profile, "compilation": compilation,
            "expected_exit": result["exit"], "stdout_sha256": result["stdout_sha256"],
            "stderr_sha256": result["stderr_sha256"],
            "_expected_stdout": result["_stdout"], "_expected_stderr": result["_stderr"],
            "semantic_stdout_sha256": sha(semantic_output(result["_stdout"])),
            "semantic_stderr_sha256": sha(semantic_output(result["_stderr"]))}


def prepare():
    """Freeze every executable/source/expectation before the first candidate."""
    original = actual.main_sources(actual.frozen_archive())
    prepared, main_results = [], {}
    with tempfile.TemporaryDirectory(prefix="tc-architecture-main-", dir="/tmp") as directory:
        root, target = Path(directory) / "source", Path(directory) / "target"
        # The whole production app is deliberately negative even where a
        # particular paint-over was removed: other pinned defects remain.
        names = ["main", "grid_model", "grid_uncovered", "grid_uncovered_model", "grid_dead", "grid_inert",
                 "brand_removed", "status_removed", "brand_model", "chrome_uncovered", "chrome_uncovered_brand"]
        for name in names:
            source = actual.mutate(original, name)
            result = actual.run_subject(root, source, target)
            main_results[name] = result
            prepared.append(prepared_record(name, source, result, "ARCHITECTURE", {
                "kind": "application", "roots": ["apps/showcase/src"],
                "entries": ["apps/showcase/src/app.rs", "apps/showcase/src/pages/grid.rs"],
                "phases": ["App::update", "App::draw", "GridPage::update", "GridPage::draw"],
                "dependencies": ["crates/tui/src"], "checks": ["source", "application-ownership"]}))
        actual.validate_causality(main_results)
        full_source = actual.conformance_source(original)
        full = actual.run_conformance(root, full_source, target)
        require(len(full["cases"]) == 44 and any(case["missing"] or case["extra"] for case in full["cases"].values()), "whole-main negative registry premise")
        prepared.append(prepared_record("whole-registry", full_source, full, "ARCHITECTURE", {
            "kind": "conformance", "roots": ["crates/tui/tests/conformance.rs"], "entries": ["crates/tui/tests/conformance.rs"],
            "registry": "conformance_suite", "checks": ["source", "registry", "parts", "slots"], "subject": "whole-pinned-registry"}))
        focused_states = None
        for mutation in ["valid", "owned_extra", "declared_unreachable", "registry_omission", "ignored_icon_slot", "measure_query", "slot_doc_drift", "state_omission", "fake_row_owner"]:
            source = actual.conformance_source(original, mutation, focused=True)
            failure = mutation in {"ignored_icon_slot", "measure_query", "slot_doc_drift", "state_omission", "fake_row_owner"}
            result = actual.run_conformance(root, source, target, expected_failure=mutation if failure else False)
            if mutation == "valid":
                require(set(result["cases"]) == {"button"} and not result["cases"]["button"]["missing"] and not result["cases"]["button"]["extra"], "focused Button must be positive")
                focused_states = result["cases"]["button"]["states"]
                require(len(focused_states) == 21, "frozen Button state/geometry set omitted")
            elif mutation == "registry_omission":
                require(result["cases"] == {}, "missing invocation must execute no registry cases")
            elif mutation in {"owned_extra", "declared_unreachable"}:
                key = "extra" if mutation == "owned_extra" else "missing"
                require(len(result["cases"]["button"][key]) == 1 and result["cases"]["button"]["states"] == focused_states, "focused PARTS mutant or state preservation ineffective")
            prepared.append(prepared_record("focused-" + mutation, source, result, None if mutation == "valid" else "ARCHITECTURE", {
                "kind": "conformance", "roots": ["crates/tui/tests/conformance.rs"], "entries": ["crates/tui/tests/conformance.rs"],
                "registry": "conformance_suite", "checks": ["source", "registry", "parts", "slots"],
                "subject": "standalone-public-Button-consumer", "required_states": focused_states}))
        for broken in (False, True):
            source = actual.nested_source(original, broken)
            actual.write_sources(root, source)
            process, compilation = actual.compile_execute(root, target, "junie-tui", "junie_tui", "architecture_bootstrap_nested_owner", kind="lib")
            require((process.returncode == 0) != broken, "real nested owner restoration premise")
            if broken:
                require(process.returncode == 101 and b"nested return leaked the inner owner" in process.stderr,
                        "nested negative failed for the wrong reason")
            result = {"compilation": compilation, "exit": process.returncode, "stdout_sha256": sha(process.stdout), "stderr_sha256": sha(process.stderr), "_stdout": process.stdout, "_stderr": process.stderr}
            prepared.append(prepared_record("nested-corrupt" if broken else "nested-valid", source, result, "ARCHITECTURE" if broken else None, {
                "kind": "ownership-fixture", "roots": ["crates/tui/src/collection/rowui.rs"],
                "entries": ["crates/tui/src/collection/rowui.rs"], "checks": ["source", "nested-row-ownership"]}))
        source = dict(original)
        source["apps/jackin-preview/tests/architecture_bootstrap.rs"] = (HERE / "architecture-bootstrap-rain-probe.rs").read_bytes()
        actual.write_sources(root, source)
        process, compilation = actual.compile_execute(root, target, "jackin-preview", "architecture_bootstrap", None)
        require(process.returncode == 0 and b"ARCHRAIN|" in process.stdout, "real app-owned art positive failed")
        result = {"compilation": compilation, "exit": process.returncode, "stdout_sha256": sha(process.stdout), "stderr_sha256": sha(process.stderr), "_stdout": process.stdout, "_stderr": process.stderr}
        prepared.append(prepared_record("rain-valid", source, result, None, {"kind": "application", "roots": ["apps/jackin-preview/tests/architecture_bootstrap.rs", "apps/jackin-preview/src/rain.rs"],
            "entries": ["apps/jackin-preview/tests/architecture_bootstrap.rs", "apps/jackin-preview/src/rain.rs"],
            "checks": ["source", "application-ownership"], "subject": "standalone-RainApp-consumer"}))
        actual.write_sources(root, original)
        process, compilation = actual.compile_execute(root, target, "junie-tui", "12_author_component", None, kind="example")
        require(process.returncode == 0 and b"test result: ok" in process.stdout, "real external-author positive failed")
        result = {"compilation": compilation, "exit": process.returncode, "stdout_sha256": sha(process.stdout), "stderr_sha256": sha(process.stderr), "_stdout": process.stdout, "_stderr": process.stderr}
        prepared.append(prepared_record("external-author-valid", original, result, None, {
            "kind": "external-author", "roots": ["crates/tui/examples/12_author_component.rs"], "entries": ["crates/tui/examples/12_author_component.rs"],
            "checks": ["source", "application-ownership"]}))
    # The preparation filesystem is gone. No expected verdict, source mutation,
    # or executable can be changed by a later candidate process.
    return prepared


class ActualFixture(runner.Fixture):
    def __init__(self, prepared):
        super().__init__("architecture", "valid")
        self.prepared = prepared
        visible = self.public / "source"
        visible.mkdir()
        for path, data in prepared["source"].items():
            for base in (visible, self.source / "actual-source"):
                destination = base / path
                destination.parent.mkdir(parents=True, exist_ok=True)
                destination.write_bytes(data)
        runner.git(self.source, "add", "actual-source")
        # write-tree binds actual generated fixture bytes; no project commit.
        self.source_tree = runner.git(self.source, "write-tree")
        self.executable = self.private / "actual-rust-test"
        self.executable.write_bytes(prepared["executable"])
        self.executable.chmod(0o755)
        profile = dict(prepared["profile"])
        profile.update({"schema": "tc-architecture-actual-rust-profile/v1", "source_directory": str(visible),
                        "source_roots": [str(visible / path) for path in profile.pop("roots")],
                        "main_commit": actual.PIN, "main_archive_sha256": actual.ARCHIVE_SHA256,
                        "sources": {path: sha(data) for path, data in prepared["source"].items()}})
        if profile["kind"] == "conformance":
            vectors = json.loads((HERE / "architecture-bootstrap-state-vectors.json").read_bytes())
            profile["required_state_vectors"] = vectors if profile["subject"] == "whole-pinned-registry" else {
                **vectors, "states": {"button": vectors["states"]["button"]}}
        compiler = prepared["compilation"]["toolchain"]["rustc"]
        self.context.update({"tree": self.source_tree, "architecture_profile": profile,
                             "tool": {"path": compiler["path"], "sha256": compiler["sha256"]}})
        runner.save(self.context_path, self.context)
        self.context_hash = sha(self.context_path.read_bytes())
        self.before = self.snapshot()

    def launch(self, request):
        require(request == {"schema": "tc-proof-runner-observe/v1", "nonce": self.nonce,
                "operation": "architecture", "source_commit": self.source_commit, "tree": self.source_tree}, "actual observer request binding")
        require(not self.events, "actual observer replay")
        require(sha(self.executable.read_bytes()) == self.prepared["compilation"]["executable_sha256"], "actual executable identity")
        result = subprocess.run(["/usr/bin/sandbox-exec", "-p", self.sandbox(writable=[], unreadable=[self.public, self.output]),
                str(self.executable), *self.prepared["worker_args"]], cwd=self.private,
                env={"PATH": "/usr/bin:/bin", "LC_ALL": "C"}, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=60, check=False)
        # Raw bytes remain authoritative; only measured libtest wall duration
        # and panic-header runtime thread IDs are excluded from replay equality.
        event = {"operation": "architecture", "run_id": self.run_id, "tree": self.source_tree, "source_commit": self.source_commit,
                 "compilation": self.prepared["compilation"], "exit": result.returncode,
                 "payload": {"schema": "tc-architecture-actual-rust-observation/v1",
                     "stdout": base64.b64encode(result.stdout).decode(), "stderr": base64.b64encode(result.stderr).decode()},
                 "stdout_sha256": sha(result.stdout), "stderr_sha256": sha(result.stderr)}
        require(result.returncode == self.prepared["expected_exit"], "replayed actual executable changed its assertion verdict")
        for stream, observed in (("stdout", result.stdout), ("stderr", result.stderr)):
            if sha(semantic_output(observed)) != self.prepared["semantic_" + stream + "_sha256"]:
                delta = list(difflib.unified_diff(semantic_output(self.prepared["_expected_" + stream]).decode().splitlines(), semantic_output(observed).decode().splitlines()))
                raise ValueError("actual replay " + self.prepared["name"] + " " + stream + " changed: " + "\n".join(delta[:24]))
        self.events.append(event)
        return event

    def validate(self, result, category):
        code, report = result
        require(len(self.events) == 1, "actual source qualification omitted protected execution")
        if category:
            require(code != 0 and report["status"] == "rejected" and report["category"] == category and report["outputs"] == {}, "actual negative did not reject exactly")
        else:
            require(code == 0 and report["status"] == "passed" and report["category"] is None and
                    report["outputs"] == {"observations": self.events}, "actual positive rejected or observations forged")


def qualify(executable, *, self_test=False):
    prepared = prepare()
    if self_test:
        for replay in ("focused-valid", "nested-corrupt"):
            fixture = ActualFixture(next(row for row in prepared if row["name"] == replay))
            fixture.sandbox = actual.sandbox_module().sandbox
            try:
                fixture.launch({"schema": "tc-proof-runner-observe/v1", "nonce": fixture.nonce,
                                "operation": "architecture", "source_commit": fixture.source_commit, "tree": fixture.source_tree})
                if replay == "focused-valid":
                    fixture.validate((0, {"status": "passed", "category": None, "outputs": {"observations": fixture.events}}), None)
                else:
                    fixture.validate((1, {"status": "rejected", "category": "ARCHITECTURE", "outputs": {}}), "ARCHITECTURE")
            finally:
                fixture.cleanup()
        accepted = 0
        rejected = 0
        transport_rejections = 0
        for name, mode in (("focused-valid", "valid"), ("focused-owned_extra", "always-pass"),
                           ("focused-valid", "forged"), ("focused-valid", "zero"), ("focused-valid", "recovery")):
            fixture = ActualFixture(next(row for row in prepared if row["name"] == name))
            try:
                adapter = fixture.public / "transport-only-candidate"
                adapter.write_text("#!" + __import__("sys").executable + "\n" +
                    "import json,os,hashlib\n"
                    "request={'schema':'tc-proof-runner-observe/v1','nonce':os.environ['TC_PROOF_OBSERVER_NONCE'],'operation':'architecture','source_commit':os.environ['TC_PROOF_ORACLE_COMMIT'],'tree':os.environ['TC_PROOF_SOURCE_TREE']}\n"
                    "with os.fdopen(int(os.environ['TC_PROOF_OBSERVER_REQUEST_FD']),'w') as f: f.write(json.dumps(request)+'\\n'); f.flush()\n"
                    "with os.fdopen(int(os.environ['TC_PROOF_OBSERVER_RESPONSE_FD'])) as f: event=json.loads(f.readline())\n"
                    "digest=hashlib.sha256(json.dumps(event,sort_keys=True,separators=(',',':')).encode()).hexdigest()\n"
                    "result={'schema':'tc-proof-runner-result/v1','run_id':os.environ['TC_PROOF_RUN_ID'],'operation':'architecture','context_sha256':os.environ['TC_PROOF_CONTEXT_SHA256'],'status':'passed','category':None,'observation_digests':[digest],'outputs':{'observations':[event]}}\n"
                    "with open(os.environ['TC_PROOF_RESULT'],'w') as f: json.dump(result,f)\n")
                adapter.chmod(0o755)
                if mode == "forged":
                    adapter.write_text(adapter.read_text().replace("digest=hashlib", "event['payload']['stdout']='forged'\ndigest=hashlib"))
                elif mode == "zero":
                    adapter.write_text("#!" + __import__("sys").executable + "\nimport json,os\n"
                        "result={'schema':'tc-proof-runner-result/v1','run_id':os.environ['TC_PROOF_RUN_ID'],'operation':'architecture','context_sha256':os.environ['TC_PROOF_CONTEXT_SHA256'],'status':'passed','category':None,'observation_digests':[],'outputs':{'observations':[]}}\n"
                        "with open(os.environ['TC_PROOF_RESULT'],'w') as f: json.dump(result,f)\n")
                try:
                    fixture.validate(fixture.run(adapter), fixture.prepared["category"])
                except ValueError as error:
                    require(name == "focused-owned_extra" or mode in {"forged", "zero"}, "valid full IPC transport failed")
                    expected_error, event_count = {
                        "always-pass": ("actual negative did not reject exactly", 1),
                        "forged": ("forged observation", 1),
                        "zero": ("actual source qualification omitted protected execution", 0),
                    }[mode]
                    require(str(error) == expected_error and len(fixture.events) == event_count,
                            "attack failed for an unrelated reason: " + str(error))
                    rejected += name == "focused-owned_extra"
                    transport_rejections += mode in {"forged", "zero"}
                else:
                    require(name == "focused-valid" and mode in {"valid", "recovery"}, "invalid actual candidate escaped fixture")
                    accepted += 1
            finally:
                fixture.cleanup()
        return {"actual_prepared_cases": len(prepared), "protected_relocated_execution": True,
                "actual_ipc_positive_recoveries": accepted, "always_pass_rejected": rejected,
                "zero_and_forged_rejected": transport_rejections}
    rows = list(prepared)
    secrets.SystemRandom().shuffle(rows)
    recovery = next(row for row in rows if row["name"] == "focused-valid")
    for row in rows:
        for selected in [row] + ([recovery] if row["category"] else []):
            fixture = ActualFixture(selected)
            try:
                fixture.validate(fixture.run(executable), selected["category"])
            finally:
                fixture.cleanup()
    return {"actual_cases": len(rows), "recoveries": sum(row["category"] is not None for row in rows), "failures": []}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--runner", type=Path)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    require(args.self_test or (args.runner and args.runner.is_file()), "actual submitted dispatcher required")
    print(json.dumps(qualify(args.runner.resolve() if args.runner else None, self_test=args.self_test), sort_keys=True))


if __name__ == "__main__":
    main()
