#!/usr/bin/env python3
"""Independent black-box runner qualification. This is not tc-proof.

The protected observer owns actual launches, source selection, PTYs and results.
Submitted JSON cannot substitute for its private in-memory observations.
"""
from __future__ import annotations

import argparse
import base64
import hashlib
import importlib.util
import itertools
import json
import os
from pathlib import Path
import pty
import secrets
import select
import subprocess
import sys
import tempfile
import threading
import time
import tty

HERE = Path(__file__).resolve().parent
OPERATIONS = {"preflight", "required", "oracle", "capture", "account-tests", "architecture", "close"}
GROUPS = {"070": OPERATIONS - {"account-tests", "architecture"},
          "071": {"account-tests"}, "072": {"architecture"}}
REPORT_KEYS = {"schema", "run_id", "operation", "context_sha256", "status", "category",
               "observation_digests", "outputs"}
TESTS = ["test_closed", "test_draw", "test_future", "test_update"]


def require(value, message):
    if not value:
        raise ValueError(message)


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


def sha(data):
    return hashlib.sha256(data).hexdigest()


def save(path, value):
    path.write_bytes(canonical(value))


def unique(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, "duplicate JSON key")
        result[key] = value
    return result


def load_bytes(data):
    return json.loads(data, object_pairs_hook=unique)


def git(root, *args):
    result = subprocess.run(["/usr/bin/git", "-C", str(root), *args], check=True,
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    return result.stdout.decode().strip()


def expanded(axes):
    return ["tiny/" + "/".join(map(str, row)) for row in itertools.product(
        axes["lanes"], axes["widths"], axes["palettes"])]


def cases():
    rows = [(op, "valid", None) for op in sorted(OPERATIONS)]
    rows += [("capture", "valid_pty", None), ("architecture", "valid_custom_art", None),
             ("account-tests", "approved_relocation", None), ("capture", "explicit_seed", None)]
    rows += [(op, mutation, category) for op, mutation, category in [
        ("preflight", "tool_pin", "PREFLIGHT"),
        ("preflight", "unaccepted_dependency", "PREFLIGHT"),
        ("preflight", "unintegrated_dependency", "PREFLIGHT"),
        ("preflight", "context_changed", "INTEGRITY"),
        ("preflight", "duplicate_context_key", "INTEGRITY"),
        ("preflight", "unknown_flag", "PROTOCOL"),
        ("required", "omitted_member", "REQUIRED_SET"),
        ("required", "extra_member", "REQUIRED_SET"),
        ("required", "duplicate_member", "REQUIRED_SET"),
        ("required", "axis_omission", "INTEGRITY"),
        ("required", "unknown_axis", "REQUIRED_SET"),
        ("oracle", "source_pin", "SOURCE"),
        ("oracle", "source_override", "SOURCE"),
        ("oracle", "adapter_render_change", "SOURCE"),
        ("oracle", "nondeterministic", "REPEAT"),
        ("capture", "stale_source", "SOURCE"),
        ("capture", "missing_update", "EXECUTION"),
        ("capture", "missing_draw", "EXECUTION"),
        ("capture", "pty_not_allocated", "EXECUTION"),
        ("capture", "omitted_capture", "EXECUTION"),
        ("capture", "duplicate_capture", "EXECUTION"),
        ("capture", "omitted_checkpoint", "EXECUTION"),
        ("capture", "constant_extractor", "EXECUTION"),
        ("capture", "wrong_extractor", "EXECUTION"),
        ("capture", "omitted_extractor", "EXECUTION"),
        ("capture", "test_only_path", "EXECUTION"),
        ("capture", "wrong_seed", "EXECUTION"),
        ("capture", "skip_seed_transition", "EXECUTION"),
        ("account-tests", "partial_tests", "TEST_ACCOUNTING"),
        ("account-tests", "dropped_test", "TEST_ACCOUNTING"),
        ("account-tests", "renamed_test", "TEST_ACCOUNTING"),
        ("account-tests", "unknown_failure", "TEST_ACCOUNTING"),
        ("account-tests", "closed_failure", "TEST_ACCOUNTING"),
        ("account-tests", "edited_allowance", "INTEGRITY"),
        ("account-tests", "profile_substitution", "TEST_ACCOUNTING"),
        ("account-tests", "missing_worker_result", "TEST_ACCOUNTING"),
        ("account-tests", "contribution_missing", "TEST_ACCOUNTING"),
        ("account-tests", "closed_contribution_regression", "TEST_ACCOUNTING"),
        ("account-tests", "contribution_parent_close", "TEST_ACCOUNTING"),
        ("architecture", "inline_control_painter", "ARCHITECTURE"),
        ("architecture", "bypass_props", "ARCHITECTURE"),
        ("architecture", "disabled_activation", "ARCHITECTURE"),
        ("close", "missing_result", "CLOSURE"),
        ("close", "failed_result", "CLOSURE"),
        ("close", "replayed_tree", "CLOSURE"),
        ("close", "self_report", "CLOSURE"),
        ("close", "edited_result", "INTEGRITY"),
    ]]
    return rows


def source_variant(mutation):
    source = (HERE / "runner-bootstrap-app.py").read_text()
    if mutation == "missing_update":
        source = source.replace('if self.props.enabled() and key == "+":', 'if False:')
    elif mutation in {"missing_draw", "inline_control_painter"}:
        source = source.replace('self.widget.draw(self.value, self.palette)',
                                '{"text": str(self.value), "foreground": self.palette, "owner": "Widget"}')
    elif mutation in {"bypass_props", "disabled_activation"}:
        source = source.replace('self.props.enabled() and key == "+"', 'key == "+"')
    elif mutation == "valid_custom_art":
        source = source.replace('"custom_art": "*"', '"custom_art": "<>"')
    elif mutation == "test_only_path":
        source = source.replace("def update(self, key):", "def test_update(self, key):")
    return source.encode()


class Fixture:
    def __init__(self, operation, mutation):
        self.temporary = tempfile.TemporaryDirectory(prefix="tc-runner-", dir="/tmp")
        self.root = Path(self.temporary.name).resolve()
        self.public = self.root / "inputs"
        self.output = self.root / "output"
        self.private = self.root / "authority"
        for path in (self.public, self.output, self.private):
            path.mkdir()
        self.operation, self.mutation = operation, mutation
        self.events, self.violations = [], []
        self.nonce = secrets.token_hex(32)
        self.run_id = secrets.token_hex(24)
        self.source = self.private / "repository"
        self.source.mkdir()
        git(self.source, "init", "-q")
        (self.source / "app.py").write_bytes(source_variant("valid"))
        (self.source / "tests.py").write_bytes((HERE / "runner-bootstrap-worker.py").read_bytes())
        git(self.source, "add", "app.py", "tests.py")
        git(self.source, "-c", "user.name=Qualification", "-c", "user.email=qualification@example.invalid",
            "-c", "commit.gpgsign=false", "commit", "-q", "-s", "-m", "Independent synthetic source",
            "-m", "Co-authored-by: Codex <codex@openai.com>")
        oracle_commit = git(self.source, "rev-parse", "HEAD")
        oracle_tree = git(self.source, "rev-parse", "HEAD^{tree}")
        git(self.source, "bundle", "create", str(self.public / "oracle.bundle"), "HEAD")
        (self.source / "app.py").write_bytes(source_variant(mutation))
        git(self.source, "add", "app.py")
        git(self.source, "-c", "user.name=Qualification", "-c", "user.email=qualification@example.invalid",
            "-c", "commit.gpgsign=false", "commit", "--allow-empty", "-q", "-s", "-m", "Synthetic submitted source",
            "-m", "Co-authored-by: Codex <codex@openai.com>")
        candidate_tree = git(self.source, "rev-parse", "HEAD^{tree}")
        self.source_commit = oracle_commit
        self.source_tree = candidate_tree
        self.config = {"seed": secrets.randbelow(100000), "palette": secrets.choice(["blue", "yellow"]),
                       "disabled": operation == "architecture"}
        for name, key in {"partial_tests": "partial", "dropped_test": "drop_test", "renamed_test": "rename_test",
                          "unknown_failure": "unknown_test", "closed_failure": "closed_failure"}.items():
            if mutation == name:
                self.config[key] = True
        if mutation.endswith("_extractor"):
            self.config["extractor"] = mutation.removesuffix("_extractor")
        if mutation in {"explicit_seed", "wrong_seed", "skip_seed_transition"}:
            self.config["seed_steps"] = ["+"]
            self.config["wrong_seed"] = mutation == "wrong_seed"
            self.config["skip_seed_transition"] = mutation == "skip_seed_transition"
        if mutation == "profile_substitution":
            self.config["profile"] = "msrv"
        if mutation == "approved_relocation":
            self.config["rename_test"] = True
        if mutation == "contribution_missing":
            self.config["missing_contribution"] = True
        if mutation == "closed_contribution_regression":
            self.config["route_regression"] = True
        for name, key in {"omitted_capture": "omit_capture", "duplicate_capture": "duplicate_capture",
                          "omitted_checkpoint": "omit_checkpoint"}.items():
            if mutation == name:
                self.config[key] = True
        self.axes = {"lanes": ["direct", "pty"], "widths": [8, 12], "palettes": ["blue", "yellow"]}
        members = expanded(self.axes)
        self.expected_members = list(members)
        self.inventory = {"source_commit": oracle_commit, "package": "tiny", "target": "unit",
                          "profile": "primary", "required": TESTS, "closed": ["test_closed"],
                          "future": {"test_future": "terminal-components/completion/999"}, "relocations": {}}
        if mutation == "approved_relocation":
            self.inventory["relocations"] = {"test_draw": "test_renamed"}
        self.inventory["contributions"] = [{"id": "tiny-shell.route", "parent_scenario": "tiny-shell",
            "required_for_closure": True, "previously_closed": True, "state_key": "route_value",
            "expected": self.config["seed"] + 1}]
        self.inventory["scenario_expectations"] = {"tiny-shell": {"custom_art": "<>"}}
        self.inventory["future_scenarios"] = {"tiny-shell": "terminal-components/completion/999"}
        self.inventory["claimed_closed_scenarios"] = ["tiny-shell"] if mutation == "contribution_parent_close" else []
        self.evidence = [{"operation": name, "status": "passed", "run_id": self.run_id,
                          "tree": candidate_tree} for name in ("capture", "compare", "account-tests", "architecture")]
        self.context = {"schema": "tc-proof-runner-context/v1", "run_id": self.run_id,
                        "operation": operation, "tree": candidate_tree, "oracle_commit": oracle_commit,
                        "oracle_tree": oracle_tree, "bundle": str(self.public / "oracle.bundle"),
                        "bundle_sha256": sha((self.public / "oracle.bundle").read_bytes()),
                        "tool": {"path": sys.executable, "sha256": sha(Path(sys.executable).read_bytes())},
                        "dependencies": [{"accepted": True, "integrated": True}],
                        "adapter": {"changes": []}, "lane": "pty" if mutation in {"valid_pty", "pty_not_allocated"} else "direct",
                        "axes": self.axes, "members": members, "inventory": self.inventory,
                        "evidence": self.evidence, "configuration": self.config}
        if mutation == "tool_pin":
            self.context["tool"]["sha256"] = "0" * 64
        elif mutation == "unaccepted_dependency":
            self.context["dependencies"][0]["accepted"] = False
        elif mutation == "unintegrated_dependency":
            self.context["dependencies"][0]["integrated"] = False
        elif mutation == "omitted_member":
            members.pop()
        elif mutation == "extra_member":
            members.append("tiny/direct/99/blue")
        elif mutation == "duplicate_member":
            members.append(members[0])
        elif mutation == "unknown_axis":
            self.axes["undocumented"] = [1]
        elif mutation in {"source_pin", "stale_source"}:
            self.context["oracle_commit" if operation == "oracle" else "tree"] = "0" * 40
        elif mutation == "source_override":
            self.context["source_directory"] = str(self.public)
        elif mutation == "adapter_render_change":
            self.context["adapter"]["changes"] = [{"path": "app.py", "purpose": "replace draw output"}]
        elif mutation == "missing_result":
            self.evidence.pop()
        elif mutation == "failed_result":
            self.evidence[0]["status"] = "failed"
        elif mutation == "replayed_tree":
            self.evidence[0]["tree"] = "0" * 40
        elif mutation == "self_report":
            self.evidence.clear()
            self.context["candidate_success"] = True
        self.context_path = self.public / "context.json"
        save(self.context_path, self.context)
        self.context_hash = sha(self.context_path.read_bytes())
        # These mutations deliberately do not update protected authority.
        if mutation == "context_changed":
            self.context["run_id"] = secrets.token_hex(24)
        elif mutation == "axis_omission":
            self.context["axes"]["widths"].pop()
        elif mutation == "edited_allowance":
            self.context["inventory"]["future"]["test_closed"] = "terminal-components/completion/999"
        elif mutation == "edited_result":
            self.context["evidence"][0]["status"] = "failed"
        save(self.context_path, self.context)
        if mutation == "duplicate_context_key":
            self.context_path.write_bytes(self.context_path.read_bytes()[:-1] + b',"operation":"preflight"}')
            self.context_hash = sha(self.context_path.read_bytes())
        self.before = self.snapshot()

    def snapshot(self):
        return {str(path.relative_to(self.root)): sha(path.read_bytes())
                for base in (self.public, self.private) for path in base.rglob("*") if path.is_file()}

    def cleanup(self):
        self.temporary.cleanup()

    def launch(self, request):
        require(set(request) == {"schema", "nonce", "operation", "source_commit", "tree"}, "observer request fields")
        require(request["schema"] == "tc-proof-runner-observe/v1" and request["nonce"] == self.nonce, "observer binding")
        require(request["operation"] == self.operation, "observer operation")
        require(request["source_commit"] == self.source_commit and request["tree"] == self.source_tree, "observer source binding")
        count = 2 if self.operation == "oracle" else 1
        require(len(self.events) < count, "observer replay")
        if self.operation == "close":
            event = {"operation": "close", "evidence": self.evidence, "run_id": self.run_id, "tree": self.source_tree}
        else:
            selected = {"oracle": "direct", "capture": self.context["lane"], "account-tests": "tests",
                        "architecture": "architecture"}[self.operation]
            config = dict(self.config)
            if self.mutation == "nondeterministic" and self.events:
                config["seed"] += 1
            config_path = self.private / "config.json"
            save(config_path, config)
            # Only fixture-owned source and worker paths are executable here.
            source = self.source / "app.py"
            if self.operation == "oracle":
                source = self.private / "oracle-app.py"
                source.write_text(git(self.source, "show", self.source_commit + ":app.py") + "\n")
            argv = [sys.executable, "-I", str(self.source / "tests.py"), str(source), selected, str(config_path)]
            env = {"PATH": "/usr/bin:/bin", "PYTHONDONTWRITEBYTECODE": "1", "LC_ALL": "C"}
            profile = self.sandbox(writable=[], unreadable=[self.public, self.output], readable=[self.source, source, config_path])
            argv = ["/usr/bin/sandbox-exec", "-p", profile, *argv]
            if selected == "pty" and self.mutation != "pty_not_allocated":
                master, slave = pty.openpty()
                tty.setraw(slave)
                process = subprocess.Popen(argv, stdin=slave, stdout=slave, stderr=subprocess.PIPE, env=env)
                os.close(slave)
                try:
                    os.write(master, b"+")
                    output = b""
                    deadline = time.monotonic() + 20
                    while b"\n" not in output and time.monotonic() < deadline:
                        if select.select([master], [], [], 0.1)[0]:
                            try:
                                part = os.read(master, 65536)
                            except OSError:
                                break
                            if not part:
                                break
                            output += part
                    _, stderr = process.communicate(timeout=10)
                    code = process.returncode
                finally:
                    if process.poll() is None:
                        process.kill()
                        process.wait()
                    os.close(master)
            else:
                result = subprocess.run(argv, input=b"+", stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                        env=env, timeout=20, check=False)
                output, stderr, code = result.stdout, result.stderr, result.returncode
            payload = load_bytes(output) if code == 0 else None
            if self.mutation == "missing_worker_result":
                payload = None
            event = {"operation": self.operation, "run_id": self.run_id, "tree": self.source_tree,
                     "source_commit": self.source_commit, "exit": code, "payload": payload,
                     "stdout_sha256": sha(output), "stderr": base64.b64encode(stderr).decode()}
        self.events.append(event)
        return event

    def run(self, runner):
        require(sys.platform == "darwin" and Path("/usr/bin/sandbox-exec").is_file(), "protected Darwin observer unavailable")
        spec = importlib.util.spec_from_file_location("frozen_host_observer", HERE / "host-bootstrap-observer.py")
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        self.sandbox = module.sandbox
        request_read, request_write = os.pipe()
        response_read, response_write = os.pipe()

        def serve():
            with os.fdopen(request_read, "rb") as incoming, os.fdopen(response_write, "wb") as outgoing:
                while True:
                    line = incoming.readline(4097)
                    if not line:
                        return
                    try:
                        require(len(line) <= 4096 and line.endswith(b"\n"), "oversized observer request")
                        reply = self.launch(load_bytes(line))
                    except Exception as error:
                        self.violations.append(str(error))
                        reply = {"error": str(error)}
                    try:
                        outgoing.write(canonical(reply) + b"\n")
                        outgoing.flush()
                    except BrokenPipeError:
                        return

        thread = threading.Thread(target=serve, daemon=True)
        thread.start()
        env = {"PATH": os.path.dirname(sys.executable) + ":/usr/bin:/bin", "PYTHONDONTWRITEBYTECODE": "1",
               "TC_PROOF_CONTEXT_SHA256": self.context_hash, "TC_PROOF_OBSERVER_NONCE": self.nonce,
               "TC_PROOF_RUN_ID": self.run_id, "TC_PROOF_SOURCE_TREE": self.source_tree,
               "TC_PROOF_ORACLE_COMMIT": self.source_commit,
               "TC_PROOF_OBSERVER_REQUEST_FD": str(request_write), "TC_PROOF_OBSERVER_RESPONSE_FD": str(response_read),
               "TC_PROOF_RESULT": str(self.output / "result.json")}
        env.update(getattr(self, "operator_env", {}))
        argv = [str(runner), self.operation, "--context", str(self.context_path)]
        if self.operation == "oracle":
            argv += ["--namespace", "synthetic"]
        elif self.operation == "capture":
            argv += ["--lane", self.context["lane"]]
        if self.mutation == "unknown_flag":
            argv += ["--approve"]
        try:
            result = subprocess.run(["/usr/bin/sandbox-exec", "-p", self.sandbox(writable=[self.output], unreadable=[self.private],
                                     readable=[runner, self.public]), *argv],
                                    env=env, pass_fds=(request_write, response_read), cwd=self.public,
                                    stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=60, check=False)
        finally:
            os.close(request_write)
            os.close(response_read)
            thread.join(timeout=30)
        require(not thread.is_alive() and not self.violations, "observer rejected launch protocol")
        # config/oracle extraction are legitimate observer-owned working files.
        after = self.snapshot()
        for name, digest in self.before.items():
            require(after.get(name) == digest, "protected input changed")
        path = Path(env["TC_PROOF_RESULT"])
        require(path.is_file() and not path.is_symlink() and path.stat().st_nlink == 1, "missing/unsafe result")
        report = load_bytes(path.read_bytes())
        require(type(report) is dict and set(report) == REPORT_KEYS, "exact result schema")
        require(report["schema"] == "tc-proof-runner-result/v1", "result version")
        require(report["run_id"] == self.run_id and report["operation"] == self.operation and
                report["context_sha256"] == self.context_hash, "result identity")
        require(type(report["outputs"]) is dict and type(report["observation_digests"]) is list and
                all(type(item) is str for item in report["observation_digests"]), "result types")
        require(report["observation_digests"] == [sha(canonical(event)) for event in self.events], "forged observation")
        # Python value equality aliases JSON booleans, integers and floats.
        # Bind the submitted observation bytes, not merely equal Python values.
        if "observations" in report["outputs"]:
            require(canonical(report["outputs"]["observations"]) == canonical(self.events),
                    "forged observation bytes")
        return result.returncode, report

    def validate(self, result, category):
        code, report = result
        if category:
            require(code != 0 and report["status"] == "rejected" and report["category"] == category,
                    "negative result did not reject with exact category")
            require(report["outputs"] == {}, "rejected operation published outputs")
            # Runtime mutants must execute, not be guessed from source text.
            if category in {"EXECUTION", "REPEAT", "TEST_ACCOUNTING", "ARCHITECTURE"}:
                require(len(self.events) == (2 if self.operation == "oracle" else 1), "negative lacks actual execution")
            return
        require(code == 0 and report["status"] == "passed" and report["category"] is None, "positive failed")
        if self.operation in {"oracle", "capture", "account-tests", "architecture", "close"}:
            require(len(self.events) == (2 if self.operation == "oracle" else 1), "positive lacks actual execution")
            expected_outputs = {"observations": self.events}
            if self.operation == "account-tests":
                expected_outputs.update(unresolved_tests=["test_future"], closed_contributions=["tiny-shell.route"],
                                        unresolved_scenarios=["tiny-shell"])
            require(canonical(report["outputs"]) == canonical(expected_outputs), "outputs differ from protected execution or stage accounting")
            for event in self.events:
                if self.operation == "close":
                    require(len(event["evidence"]) == 4 and all(row["status"] == "passed" for row in event["evidence"]), "closure evidence")
                    continue
                require(event["exit"] == 0 and type(event["payload"]) is dict, "worker did not complete")
                payload = event["payload"]
                if self.operation == "account-tests":
                    required = sorted(self.inventory["relocations"].get(name, name) for name in TESTS)
                    require(payload["discovered"] == required and sorted(row["id"] for row in payload["results"]) == required,
                            "full test execution")
                    require(all(payload[key] == self.inventory[key] for key in ("profile", "package", "target")), "test qualifiers")
                    require([row for row in payload["results"] if row["status"] != "passed"] ==
                            [{"id": "test_future", "status": "failed"}], "honest future failure accounting")
                    require(payload["scenarios"][0]["state"].get("route_value") == self.config["seed"] + 1,
                            "missing or regressed closed contribution")
                    require(payload["scenarios"][0]["frame"]["custom_art"] != "<>" and
                            not self.inventory["claimed_closed_scenarios"], "partial contribution closed a failing full scenario")
                else:
                    draw_count = 2 if self.operation == "architecture" else 10
                    require("App.update" in payload["calls"] and payload["calls"].count("Widget.draw") == draw_count and
                            "Props.enabled" in payload["calls"], "production paths not observed")
                    seeded = self.config["seed"] + len(self.config.get("seed_steps", []))
                    require("value" in payload and payload["value"] == seeded + (0 if self.operation == "architecture" else 1), "production update/extraction")
                    require(payload["pty"] == (self.operation == "capture" and self.context["lane"] == "pty"), "actual PTY")
                    if self.operation != "architecture":
                        lane = self.context["lane"] if self.operation == "capture" else "direct"
                        members = [name for name in self.expected_members if name.split("/")[1] == lane]
                        require([row["id"] for row in payload["captures"]] == members, "capture membership")
                        for row in payload["captures"]:
                            require(set(row) == {"id", "before", "after"}, "missing or extra checkpoint")
                            _, _, width, palette = row["id"].split("/")
                            for checkpoint, increment in (("before", 0), ("after", 1)):
                                require(row[checkpoint] == {"width": int(width), "custom_art": "*", "control": {
                                    "text": str(seeded + increment), "foreground": palette, "owner": "Widget"}},
                                    "full checkpoint capture differs from real production behavior")
            if self.operation == "oracle":
                require(self.events[0]["payload"] == self.events[1]["payload"], "oracle repeat")
        elif self.operation == "required":
            require(not self.events and canonical(report["outputs"]) == canonical({"members": self.expected_members}), "finite expansion")
        else:
            require(not self.events and canonical(report["outputs"]) == canonical({"validated": True}), "preflight result")


def run_case(runner, row):
    operation, mutation, category = row
    fixture = Fixture(operation, mutation)
    try:
        fixture.validate(fixture.run(runner), category)
    finally:
        fixture.cleanup()


def self_test():
    checked = 0
    for operation, mutation, category in cases():
        fixture = Fixture(operation, mutation)
        try:
            require(len(fixture.expected_members) == 8, "finite expansion fixture")
            require(len(set(fixture.expected_members)) == 8, "unique expansion fixture")
            for bad in [({}, None), (0, {}), (0, {"status": "passed"}),
                        (0, {"status": "rejected", "category": category, "outputs": {}})]:
                try:
                    fixture.validate(bad, category)
                except (ValueError, KeyError, TypeError):
                    pass
                else:
                    raise ValueError("malformed/fake result accepted")
            checked += 1
        finally:
            fixture.cleanup()
    # A real external process returns without requesting any observer launch.
    with tempfile.TemporaryDirectory(prefix="tc-runner-liar-", dir="/tmp") as directory:
        path = Path(directory) / "liar"
        path.write_text("#!" + sys.executable + "\n" +
            "import json,os\n"
            "r={'schema':'tc-proof-runner-result/v1','run_id':os.environ['TC_PROOF_RUN_ID'],"
            "'operation':'capture','context_sha256':os.environ['TC_PROOF_CONTEXT_SHA256'],"
            "'status':'passed','category':None,'observation_digests':[],'outputs':{'observations':[]}}\n"
            "with open(os.environ['TC_PROOF_RESULT'],'w') as f: json.dump(r,f)\n")
        path.chmod(0o755)
        try:
            run_case(path, ("capture", "valid", None))
        except ValueError:
            pass
        else:
            raise ValueError("zero-worker liar accepted")
    spec = importlib.util.spec_from_file_location("frozen_host_observer", HERE / "host-bootstrap-observer.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    observed_cases = 0
    for operation, mutation, category in cases():
        if operation not in {"oracle", "capture", "account-tests", "architecture"}:
            continue
        if category and category not in {"EXECUTION", "REPEAT", "TEST_ACCOUNTING", "ARCHITECTURE"}:
            continue
        fixture = Fixture(operation, mutation)
        fixture.sandbox = module.sandbox
        try:
            for _ in range(2 if operation == "oracle" else 1):
                fixture.launch({"schema": "tc-proof-runner-observe/v1", "nonce": fixture.nonce,
                                "operation": operation, "source_commit": fixture.source_commit, "tree": fixture.source_tree})
            # This is driver-internal assertion testing, not a submitted runner.
            success = {"status": "passed", "category": None, "outputs": {"observations": fixture.events}}
            if operation == "account-tests":
                success["outputs"].update(unresolved_tests=["test_future"], closed_contributions=["tiny-shell.route"],
                                          unresolved_scenarios=["tiny-shell"])
            try:
                fixture.validate((0, success), None)
            except ValueError:
                require(category is not None, "valid protected execution failed")
            else:
                require(category is None, "genuine runtime mutant passed independent validation")
            observed_cases += 1
        finally:
            fixture.cleanup()
    return {"fixture_cases": checked, "malformed_result_rejections": checked * 4,
            "actual_observer_cases": observed_cases, "zero_worker_liar_rejected": True}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--runner", type=Path)
    parser.add_argument("--group", choices=sorted(GROUPS))
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        report = self_test()
        for filename in ("runner-bootstrap-index.py", "runner-bootstrap-extensions.py"):
            spec = importlib.util.spec_from_file_location(filename.removesuffix(".py"), HERE / filename)
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            report.update(module.self_test())
        print(json.dumps(report, sort_keys=True))
        return
    require(args.runner is not None and args.runner.is_file(), "a built submitted runner is required")
    rows = [row for row in cases() if not args.group or row[0] in GROUPS[args.group]]
    secrets.SystemRandom().shuffle(rows)
    for row in rows:
        run_case(args.runner.resolve(), row)
        if row[2]:
            run_case(args.runner.resolve(), (row[0], "valid", None))
    extra = 0
    for filename in ("runner-bootstrap-index.py", "runner-bootstrap-extensions.py"):
        if filename.endswith("index.py") and args.group not in (None, "070"):
            continue
        spec = importlib.util.spec_from_file_location(filename.removesuffix(".py"), HERE / filename)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        extra += module.run(args.runner.resolve()) if filename.endswith("index.py") else module.run(args.runner.resolve(), args.group)
    print(json.dumps({"cases": len(rows), "recoveries": sum(row[2] is not None for row in rows), "extension_invocations": extra, "failures": []}))


if __name__ == "__main__":
    main()
