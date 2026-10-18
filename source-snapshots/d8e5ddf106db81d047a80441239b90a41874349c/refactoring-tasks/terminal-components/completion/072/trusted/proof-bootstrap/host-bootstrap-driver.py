#!/usr/bin/env python3
"""Independent acceptance driver. Never imports a submitted host implementation."""
from __future__ import annotations

import argparse
import base64
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time
import unittest

HERE = Path(__file__).resolve().parent
FIXTURE = HERE / "host-bootstrap-fixture"
TASKFMT_REVISION = "52d9f1eb7721f409bc47beb9fced7997b5c13ede"
TASKFMT_FINGERPRINT = "52c960db74b3b288ce93211c82e5703a338ba5ddfd40c92d6054ef93cfcd94e4"
REF = "refs/heads/refactor/holla-parity"
OBSERVER_SPEC = importlib.util.spec_from_file_location("host_bootstrap_observer", HERE / "host-bootstrap-observer.py")
OBSERVER_MODULE = importlib.util.module_from_spec(OBSERVER_SPEC)
OBSERVER_SPEC.loader.exec_module(OBSERVER_MODULE)


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def snapshot(root: Path) -> dict:
    """Include names, kinds and bytes; additions/deletions also change identity."""
    if not root.exists():
        return {}
    result = {}
    for path in sorted(root.rglob("*")):
        relative = path.relative_to(root).as_posix()
        if path.is_symlink():
            result[relative] = ["link", os.readlink(path)]
        elif path.is_file():
            result[relative] = ["file", digest(path)]
        else:
            result[relative] = ["directory"]
    return result


def run(argv: list, *, cwd: Path | None = None, env: dict | None = None) -> subprocess.CompletedProcess:
    return subprocess.run([str(x) for x in argv], cwd=cwd, env=env, text=True,
                          stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=90, check=False)


def git(root: Path, *args: str) -> str:
    environment = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}
    environment.update(GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull,
                       GIT_AUTHOR_DATE="2026-01-01T00:00:00+0000",
                       GIT_COMMITTER_DATE="2026-01-01T00:00:00+0000")
    result = run(["git", "-c", "core.hooksPath=" + os.devnull,
                  "-c", "user.name=Bootstrap Fixture", "-c", "user.email=fixture@example.invalid",
                  *args], cwd=root, env=environment)
    require(result.returncode == 0, f"git {args}: {result.stderr}")
    return result.stdout.strip()


def commit(root: Path, subject: str) -> str:
    git(root, "add", "--all")
    git(root, "commit", "-q", "-s", "-m", subject,
        "-m", "Co-authored-by: Codex <codex@openai.com>")
    return git(root, "rev-parse", "HEAD")


def repository(root: Path) -> str:
    root.mkdir()
    git(root, "init", "-q", "--initial-branch=fixture")
    (root / "src").mkdir()
    (root / ".proof").mkdir()
    (root / "src/payload.txt").write_bytes(b"pending\n")
    (root / "src/obsolete.txt").write_bytes(b"remove this tracked obsolete source\n")
    (root / "protected.txt").write_bytes(b"planner-owned sentinel\n")
    shutil.copyfile(FIXTURE / "check.py", root / ".proof/check.py")
    parent = commit(root, "Create independent qualification fixture")
    git(root, "update-ref", REF, parent)
    return parent


def complete_progress(fresh: str) -> str:
    leaves = ["1.1", "2.1", "2.2", "2.3", "3.1"]
    events = []
    for leaf in leaves:
        for status in ["STARTED", "DONE"]:
            events.append(f"- {len(events) + 1} | {status} | {leaf}")
    require("- 1 | STARTED | 1.1" in fresh, "unsupported progress-init grammar")
    return (fresh.replace("state: IN_PROGRESS", "state: DONE", 1)
            .replace("current: 1.1", "current: NONE", 1)
            .replace("latest_event: 1\n", "latest_event: 10\n", 1)
            .replace("- 1 | STARTED | 1.1", "\n".join(events), 1))


def taskfmt_pass(result: subprocess.CompletedProcess) -> bool:
    return result.returncode == 0 and bool(result.stdout.splitlines()) and result.stdout.splitlines()[-1] == "DONE"


def qualify_taskfmt(binary: Path, source: Path) -> dict:
    """Real standalone gate on a canonical package; does not simulate host isolation."""
    require(TASKFMT_REVISION in run([binary, "--version"]).stdout, "wrong taskfmt revision")
    require(run([binary, "fingerprint"]).stdout.strip() == TASKFMT_FINGERPRINT, "wrong taskfmt fingerprint")
    require(git(source, "rev-parse", "HEAD") == TASKFMT_REVISION, "taskfmt source pin changed")
    trusted = snapshot(FIXTURE)
    config = source / "experiment.toml"
    lint = run([binary, "lint", FIXTURE, "--config", config])
    require(lint.returncode == 0, "canonical fixture lint failed: " + lint.stdout + lint.stderr)
    results = {}
    for case in ["positive", "out_of_scope", "failed_check", "incomplete_progress", "done_nonzero", "missing_overlay"]:
        with tempfile.TemporaryDirectory(prefix="tc-taskfmt-bootstrap-") as directory:
            temporary = Path(directory)
            root = temporary / "work"
            parent = repository(root)
            progress = temporary / "progress.md"
            initialized = run([binary, "progress-init", FIXTURE, "--config", config, "--out", progress])
            require(initialized.returncode == 0, "progress-init failed")
            if case != "incomplete_progress":
                progress.write_text(complete_progress(progress.read_text()))
            (root / "src/payload.txt").write_bytes(b"qualified\n")
            if case == "out_of_scope":
                (root / "outside.txt").write_bytes(b"out-of-scope addition\n")
            elif case == "failed_check":
                (root / "src/payload.txt").write_bytes(b"candidate-generated expectation\n")
            elif case == "done_nonzero":
                # Deliberately hostile checker in a separate immutable scope base. The
                # package remains canonical; taskfmt must preserve this process failure.
                (root / ".proof/check.py").write_text("print('DONE')\nraise SystemExit(7)\n")
                parent = commit(root, "Install intentionally failing verifier fixture")
            elif case == "missing_overlay":
                (root / ".proof/check.py").unlink()
                parent = commit(root, "Require externally supplied trusted overlay")
            checked = run([binary, "verify", "--config", config, "--root", root,
                           "--task-dir", FIXTURE, "--base", parent, "--progress", progress,
                           "--log-dir", temporary / "logs"])
            passed = taskfmt_pass(checked)
            require(passed == (case == "positive"), f"{case}: unexpected verdict\n{checked.stdout}\n{checked.stderr}")
            if case != "positive":
                require(checked.returncode != 0, f"{case}: nonzero exit required")
            require(git(root, "rev-parse", REF) == git(root, "rev-list", "--max-parents=0", "HEAD"),
                    f"{case}: taskfmt mutated integration ref")
            results[case] = {"exit": checked.returncode, "passed": passed}
    require(snapshot(FIXTURE) == trusted, "standalone verification mutated planner fixture")
    return {"schema": "tc-host-bootstrap-taskfmt/v1", "binary_sha256": digest(binary), "cases": results}


def strict_json(text: str) -> dict:
    def pairs(items):
        result = {}
        for key, value in items:
            require(key not in result, "duplicate JSON key")
            result[key] = value
        return result
    result = json.loads(text, object_pairs_hook=pairs)
    require(isinstance(result, dict), "result must be an object")
    return result


def write_json(path: Path, value: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n")


def json_digest(value: dict) -> str:
    return hashlib.sha256((json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n").encode()).hexdigest()


def regular_beneath(root: Path, relative: str) -> Path:
    path = Path(relative)
    require(not path.is_absolute() and relative != "" and ".." not in path.parts, "unsafe output path")
    current = root
    for part in path.parts:
        current = current / part
        require(not current.is_symlink(), "symlink output path")
    require(current.is_file() and current.stat().st_nlink == 1, "missing/nonregular/hardlinked output")
    require(current.resolve().is_relative_to(root.resolve()), "output escaped run")
    return current


def tree_digest(path: Path) -> str:
    return hashlib.sha256(json.dumps(snapshot(path), sort_keys=True, separators=(",", ":")).encode()).hexdigest()


# This program is adversarial *input* executed twice in subordinate candidate
# workers. A successful temporary write is recorded even when bytes are restored.
# Expected bytes are never embedded in this candidate program.
WORKER = r'''import json, os, pathlib, sys
stage = sys.argv[1]
targets = json.loads(pathlib.Path("src/probe-targets.json").read_text())
observations = {"stage": stage, "writes": {}, "reads": {},
                "authority_env_present": "TC_PROOF_AUTHORITY_FILE" in os.environ,
                "observer_env_present": any(key.startswith("TC_PROOF_OBSERVER_") for key in os.environ)}
for name, filename in targets["writes"].items():
    path = pathlib.Path(filename)
    try:
        # Open for writing before reading: mutate-and-restore must be denied even
        # when a mount permits reads. The independent observer checks the attempt.
        with path.open("r+b") as stream:
            original = stream.read()
            stream.seek(0)
            stream.write(b"ATTACK")
            stream.flush()
            os.fsync(stream.fileno())
            stream.seek(0)
            stream.write(original)
            stream.truncate()
        observations["writes"][name] = "succeeded"
    except (OSError, PermissionError):
        observations["writes"][name] = "denied"
for name, filename in targets["reads"].items():
    try:
        pathlib.Path(filename).read_bytes()
        observations["reads"][name] = "succeeded"
    except (OSError, PermissionError):
        observations["reads"][name] = "denied"
output = pathlib.Path(os.environ["TC_PROOF_WORKER_OUTPUT"])
output.mkdir(parents=True, exist_ok=True)
(output / (stage + ".json")).write_text(json.dumps(observations, sort_keys=True))
if stage == "test":
    (output / "payload.txt").write_bytes(pathlib.Path("src/payload.txt").read_bytes())
'''


class HostFixture:
    """Operator-authored synthetic campaign; never modifies the real repository."""
    def __init__(self, temporary: Path, executable: Path, taskfmt: Path, taskfmt_source: Path, case: str):
        self.root = temporary
        self.repo = temporary / "repository"
        self.parent = repository(self.repo)
        (self.repo / ".proof/overlay.txt").write_bytes(b"independently approved verification overlay\n")
        self.scope_base = commit(self.repo, "Apply independently approved verification overlay")
        self.campaign = temporary / "campaign"
        self.run_dir = temporary / "run"
        self.candidate = temporary / "candidate"
        self.install = temporary / "install"
        self.authority = temporary / "authority.json"
        self.executable = executable
        self.observer = OBSERVER_MODULE.Observer(temporary)
        self.case = case
        self.task = "qualification/synthetic/901"
        for folder in ["receipts", "ledger", "expected", "other-run", "catalog"]:
            (self.campaign / folder).mkdir(parents=True, exist_ok=True)
        self.package = self.campaign / "catalog/task"
        shutil.copytree(FIXTURE, self.package)
        self.expected = self.campaign / "expected/payload.txt"
        self.expected.write_bytes(b"qualified\n")
        (self.campaign / "ledger/sentinel").write_bytes(b"host ledger\n")
        (self.campaign / "other-run/sentinel").write_bytes(b"other run\n")
        (self.campaign / "secret").write_bytes(os.urandom(32))
        base_receipt = {"schema": "tc-proof-host-receipt/v1", "producer": "qualification/prerequisite/900",
                        "product": "prerequisite", "source_commit": self.parent,
                        "source_tree": git(self.repo, "rev-parse", self.parent + "^{tree}"), "dependencies": []}
        if case == "wrong_predecessor":
            base_receipt["producer"] = "qualification/unrelated/999"
        elif case == "unintegrated_predecessor":
            (self.repo / "src/side.txt").write_bytes(b"not integrated\n")
            base_receipt["source_commit"] = commit(self.repo, "Unintegrated predecessor")
            base_receipt["source_tree"] = git(self.repo, "rev-parse", "HEAD^{tree}")
        self.predecessor = self.campaign / "receipts/predecessor.json"
        write_json(self.predecessor, base_receipt)
        self.harness_receipt = self.campaign / "receipts/harness.json"
        write_json(self.harness_receipt, {
            "schema": "tc-proof-host-receipt/v1", "producer": "qualification/bootstrap/000",
            "product": "harness", "source_commit": self.parent,
            "source_tree": git(self.repo, "rev-parse", self.parent + "^{tree}"),
            "dependencies": [], "executable": str(executable), "executable_sha256": digest(executable)})
        self.config = self.campaign / "campaign.json"
        write_json(self.config, {
            "schema": "tc-proof-host-campaign/v1", "repository": str(self.repo), "integration_ref": REF,
            "run_id": self.observer.nonce,
            "trusted_overlay": {"scope_base": self.scope_base, "parent": self.parent,
                                "paths": {".proof/overlay.txt": digest(self.repo / ".proof/overlay.txt")}},
            "catalog_root": str(self.campaign / "catalog"), "catalog_sha256": tree_digest(self.campaign / "catalog"),
            "receipt_root": str(self.campaign / "receipts"), "ledger_root": str(self.campaign / "ledger"),
            "harness_receipt_sha256": digest(self.harness_receipt),
            "taskfmt": {"executable": str(taskfmt), "sha256": digest(taskfmt), "revision": TASKFMT_REVISION,
                        "fingerprint": TASKFMT_FINGERPRINT, "config": str(taskfmt_source / "experiment.toml"),
                        "config_sha256": digest(taskfmt_source / "experiment.toml")},
            "tasks": {self.task: {
                "package": "task", "package_sha256": tree_digest(self.package),
                "dependencies": [{"producer": "qualification/prerequisite/900", "product": "prerequisite",
                                  "receipt_sha256": digest(self.predecessor)}],
                "seal_products": [], "required_checks": ["CHK-001", "CHK-002", "CHK-003", "CHK-004", "CHK-005"],
                "check_context_templates": [{"check_id": f"CHK-{number:03d}", "schema": "tc-host-fixture-check-context/v1",
                                             "operation": operation, "lane": None, "namespace": None,
                                             "required_ids": ["sentinel", "payload"] if operation == "all" else [operation],
                                             "output_id": f"CHK-{number:03d}.log"}
                                            for number, operation in enumerate(["sentinel", "payload", "sentinel", "all", "all"], 1)],
                "workers": [{"id": stage, "argv": ["python3", "-I", "src/worker.py", stage],
                             "required_outputs": [stage + ".json"] + (["payload.txt"] if stage == "test" else [])}
                            for stage in ["build", "test"]],
                "artifact_comparisons": [{"worker": "test", "output": "payload.txt",
                                          "expected": str(self.expected), "sha256": digest(self.expected)}]}}})
        write_json(self.authority, {"schema": "tc-proof-host-authority/v1", "campaign": str(self.config),
                                  "campaign_sha256": digest(self.config),
                                  "accepted_receipts": {digest(path): str(path) for path in [self.predecessor, self.harness_receipt]}})
        self.env = {k: v for k, v in os.environ.items() if not k.startswith(("TC_PROOF_", "TASKFMT_", "GIT_", "PYTHON"))}
        self.env["TC_PROOF_AUTHORITY_FILE"] = str(self.authority)
        git(temporary, "clone", "-q", "--no-hardlinks", str(self.repo), str(self.candidate))
        git(self.candidate, "checkout", "-q", "--detach", self.scope_base)
        (self.candidate / "src/payload.txt").write_bytes(b"qualified\n")
        (self.candidate / "src/obsolete.txt").unlink()
        (self.candidate / "src/worker.py").write_text(WORKER)
        (self.candidate / "src/allowed-untracked.txt").write_bytes(b"include this unstaged source file\n")
        targets = {"writes": {"expected": str(self.expected), "catalog": str(self.package / "verify.toml"),
                              "receipt": str(self.predecessor), "ledger": str(self.campaign / "ledger/sentinel"),
                              "other_run": str(self.campaign / "other-run/sentinel"),
                              "authority": str(self.authority), "secret": str(self.campaign / "secret"),
                              "ref": str(self.repo / ".git" / REF),
                              "harness": str(self.install / "bin/tc-proof-host")},
                   "reads": {"expected": str(self.expected), "secret": str(self.campaign / "secret"),
                             "receipt": str(self.predecessor), "authority": str(self.authority)}}
        self.targets = targets
        targets["writes"]["observer"] = str(self.observer.root / "sentinel")
        targets["reads"]["observer"] = str(self.observer.root / "sentinel")
        write_json(self.candidate / "src/probe-targets.json", targets)
        self.observer.immutable = [self.candidate, self.package, self.config, self.authority,
                                   self.campaign / "expected", self.campaign / "receipts", self.campaign / "other-run",
                                   self.campaign / "secret", self.campaign / "ledger/sentinel"]
        self.observer.host_readable = [taskfmt, taskfmt_source / "experiment.toml", executable]

    def independent_tree(self) -> str:
        private = self.observer.root / "expected-tree"
        shutil.copytree(self.candidate, private, ignore=shutil.ignore_patterns(".git"))
        git(private, "init", "-q", "--initial-branch=fixture")
        git(private, "add", "--all")
        return git(private, "write-tree")

    def protect_verified_run(self, verdict_bytes: bytes) -> None:
        # These exact bytes were read once and validated against independent
        # execution evidence. Never re-read an external copy to select authority.
        self.accepted_verdict_sha256 = hashlib.sha256(verdict_bytes).hexdigest()
        self.accepted_proof = self.observer.root / "accepted-proof"
        self.accepted_proof.mkdir()
        (self.accepted_proof / "verdict.json").write_bytes(verdict_bytes)
        self.accepted_copies = {"verdict.json": verdict_bytes}
        checks = strict_json(verdict_bytes.decode())["checks"]
        for event in self.observer.events:
            for name, encoded in event["files"].items():
                path = self.accepted_proof / event["step"] / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(base64.b64decode(encoded))
                if event["step"] != "taskfmt":
                    self.accepted_copies[f"workers/{event['step']}/{name}"] = base64.b64decode(encoded)
            if event["step"] == "taskfmt":
                for check in checks:
                    self.accepted_copies[check["log"]] = base64.b64decode(event["files"][check["id"] + ".log"])
        require(digest(regular_beneath(self.run_dir, "verdict.json")) == self.accepted_verdict_sha256,
                "external verdict changed after validation")
        self.accepted_run_snapshot = snapshot(self.run_dir)
        self.accepted_private_snapshot = snapshot(self.accepted_proof)
        # Integration needs only immutable run inputs; its authorized writes
        # are the repository object/ref update and the single ledger append.
        self.observer.immutable.append(self.run_dir)
        self.validate_accepted_copy()

    def validate_accepted_copy(self) -> None:
        require(digest(self.accepted_proof / "verdict.json") == self.accepted_verdict_sha256,
                "observer-private verdict authority changed")
        require(snapshot(self.accepted_proof) == self.accepted_private_snapshot, "observer-private execution evidence changed")
        for name, expected in self.accepted_copies.items():
            require(regular_beneath(self.run_dir, name).read_bytes() == expected, "external verification evidence differs from independent bytes")
        require(digest(regular_beneath(self.run_dir, "verdict.json")) == self.accepted_verdict_sha256 and
                snapshot(self.run_dir) == self.accepted_run_snapshot, "external verification copy changed after acceptance")

    def check_contexts(self, tree: str) -> tuple[dict, dict]:
        config = strict_json(self.config.read_text())
        trust = json_digest({"catalog_sha256": config["catalog_sha256"],
                             "harness_receipt_sha256": config["harness_receipt_sha256"],
                             "dependencies": [digest(self.predecessor)]})
        members, contexts = [], {}
        for template in config["tasks"][self.task]["check_context_templates"]:
            check, operation = template["check_id"], template["operation"]
            filename = f"contexts/{check}.json"
            required = template["required_ids"]
            child = dict(template, run_id=self.observer.nonce, task_id=self.task, tree=tree)
            contexts[filename] = child
            members.append({"check_id": check, "context_path": filename, "context_sha256": json_digest(child),
                            "schema": child["schema"], "operation": operation, "lane": None, "namespace": None,
                            "required_ids": required, "output_id": check + ".log"})
        return {"schema": "tc-proof-context-index/v1", "run_id": self.observer.nonce, "task_id": self.task,
                "tree": tree, "trust_sha256": trust, "members": members}, contexts

    def validate_contexts(self, index: dict, contexts: dict) -> None:
        path = regular_beneath(self.run_dir, "context-index.json")
        require(strict_json(path.read_text()) == index and digest(path) == json_digest(index), "context index binding changed")
        require({"contexts/" + path.name for path in (self.run_dir / "contexts").iterdir()} == set(contexts), "context membership changed")
        for name, value in contexts.items():
            path = regular_beneath(self.run_dir, name)
            require(strict_json(path.read_text()) == value and digest(path) == json_digest(value), "context member binding changed")

    def call(self, operation: str, *arguments, category: str | None = None) -> dict:
        writable = [self.install] if operation == "install" else [self.run_dir]
        if operation == "integrate":
            writable = [self.run_dir, self.repo / ".git", self.campaign / "ledger"]
        original = snapshot(self.candidate)
        result = self.observer.run_host([str(x) for x in [self.executable, operation, *arguments]], self.env, self.root, writable=writable)
        require(snapshot(self.candidate) == original, "submitted host changed independently authored candidate inputs")
        return host_result(result, operation, category)

    def protected(self) -> dict:
        result = {"receipts": snapshot(self.campaign / "receipts"), "expected": snapshot(self.campaign / "expected"),
                "catalog": snapshot(self.campaign / "catalog"), "ledger": snapshot(self.campaign / "ledger"),
                "refs": git(self.repo, "for-each-ref", "--format=%(refname) %(objectname)"),
                "authority": digest(self.authority), "install": snapshot(self.install),
                "secret": digest(self.campaign / "secret"), "other_run": snapshot(self.campaign / "other-run"),
                "repository_configuration": {"config": digest(self.repo / ".git/config"), "HEAD": digest(self.repo / ".git/HEAD"),
                                             "hooks": snapshot(self.repo / ".git/hooks"), "info": snapshot(self.repo / ".git/info")}}
        if hasattr(self, "accepted_run_snapshot"):
            result["accepted_run"] = snapshot(self.run_dir)
        return result

    def reject(self, operation: str, *arguments, category: str) -> None:
        before = self.protected()
        self.call(operation, *arguments, category=category)
        require(self.protected() == before, f"{self.case}: rejected command mutated protected state")

    def execute(self) -> None:
        case = self.case
        if case == "forged_install_receipt":
            receipt = strict_json(self.harness_receipt.read_text())
            receipt["executable_sha256"] = "0" * 64
            write_json(self.harness_receipt, receipt)
            self.reject("install", "--receipt", self.harness_receipt, "--destination", self.install, category="receipt")
            return
        self.call("install", "--receipt", self.harness_receipt, "--destination", self.install)
        installed = self.install / "bin/tc-proof-host"
        require(not installed.is_symlink() and installed.is_file() and installed.stat().st_nlink == 1
                and digest(installed) == digest(self.executable), "install did not reproduce accepted regular executable")
        self.executable = installed
        prepare = ["--campaign", self.campaign, "--task", self.task, "--parent", self.parent, "--run", self.run_dir]
        if case == "forged_predecessor_receipt":
            self.predecessor.write_text(self.predecessor.read_text() + " ")
        if case in {"forged_predecessor_receipt", "wrong_predecessor", "unintegrated_predecessor"}:
            category = {"forged_predecessor_receipt": "receipt", "wrong_predecessor": "dependency", "unintegrated_predecessor": "ancestry"}[case]
            self.reject("prepare", *prepare, category=category)
            return
        self.call("prepare", *prepare)
        context = strict_json((self.run_dir / "preparation.json").read_text())
        require(context.get("task") == self.task and context.get("parent") == self.parent, "context binding incorrect")
        progress = self.run_dir / "progress.md"
        require(progress.is_file(), "prepare omitted progress-init result")
        if case != "incomplete_progress":
            progress.write_text(complete_progress(progress.read_text()))
        if case == "premature_seal":
            self.reject("seal", "--run", self.run_dir, "--product", "oracle-showcase", category="authority")
            return
        if case == "out_of_scope":
            (self.candidate / "outside.txt").write_bytes(b"out of scope\n")
        elif case == "forbidden_checker":
            (self.candidate / ".proof/check.py").write_text("raise SystemExit(0)\n")
        elif case == "overlay_tamper":
            (self.candidate / ".proof/overlay.txt").write_bytes(b"candidate-modified overlay\n")
        elif case == "symlink_escape":
            (self.candidate / "src/escape").symlink_to(self.expected)
        elif case == "hardlink_escape":
            os.link(self.expected, self.candidate / "src/escape")
        elif case == "ignored_source":
            with (self.candidate / ".git/info/exclude").open("a") as exclude:
                exclude.write("\nsrc/hidden.py\n")
            (self.candidate / "src/hidden.py").write_text("raise SystemExit(0)\n")
        elif case == "hidden_index_flag":
            git(self.candidate, "update-index", "--assume-unchanged", "protected.txt")
        elif case == "changed_git_config":
            git(self.candidate, "config", "--local", "user.name", "Candidate-supplied authority")
        elif case == "submodule_substitution":
            git(self.candidate, "update-index", "--add", "--cacheinfo", "160000," + self.parent + ",src/submodule")
        if case in {"ignored_source", "hidden_index_flag", "changed_git_config", "submodule_substitution"}:
            self.reject("freeze", "--run", self.run_dir, "--candidate", self.candidate, category="unsafe-git")
            return
        if case in {"out_of_scope", "forbidden_checker", "overlay_tamper", "symlink_escape", "hardlink_escape"}:
            self.reject("freeze", "--run", self.run_dir, "--candidate", self.candidate,
                        category="scope" if case in {"out_of_scope", "forbidden_checker", "overlay_tamper"} else "unsafe-path")
            return
        if case in {"failed_worker", "done_nonzero_worker", "missing_worker_result"}:
            (self.candidate / "src/worker.py").write_text({"failed_worker": "raise SystemExit(9)\n",
                "done_nonzero_worker": "print('DONE')\nraise SystemExit(7)\n",
                "missing_worker_result": "raise SystemExit(0)\n"}[case])
        if case == "candidate_expected":
            # Source-level task checks stay green. Only worker output diverges.
            with (self.candidate / "src/worker.py").open("a") as worker:
                worker.write("\nif stage == 'test':\n    (output / 'payload.txt').write_bytes(b'candidate blessing\\n')\n")
        require(git(self.candidate, "write-tree") == git(self.candidate, "rev-parse", "HEAD^{tree}"), "fixture index unexpectedly staged candidate edits")
        expected_tree = self.independent_tree()
        require(expected_tree != git(self.candidate, "write-tree"), "fixture failed to exercise a stale index")
        require(git(self.candidate, "ls-files", "--", "src/obsolete.txt") == "src/obsolete.txt", "stale index lost tracked deletion case")
        require(git(self.observer.root / "expected-tree", "ls-tree", expected_tree, "--", "src/obsolete.txt") == "", "independent tree restored tracked deletion")
        self.observer.arm(self.candidate, progress, self.package,
                          Path(strict_json(self.config.read_text())["taskfmt"]["executable"]),
                          Path(strict_json(self.config.read_text())["taskfmt"]["config"]),
                          self.scope_base, expected_tree, [self.campaign, self.authority, self.install, self.repo, self.observer.root / "sentinel"])
        self.call("freeze", "--run", self.run_dir, "--candidate", self.candidate)
        frozen = strict_json((self.run_dir / "freeze.json").read_text())
        require(frozen.get("tree") == expected_tree and frozen.get("parent") == self.parent and frozen.get("scope_base") == self.scope_base,
                "freeze tree/parent/scope base differs from independent fixture")
        index, contexts = self.check_contexts(expected_tree)
        self.validate_contexts(index, contexts)
        self.observer.immutable.extend([self.run_dir / "context-index.json", self.run_dir / "contexts", self.run_dir / "freeze.json", progress])
        self.env["TC_PROOF_CONTEXT_INDEX"] = str(self.run_dir / "context-index.json")
        self.env["TC_PROOF_CONTEXT_INDEX_SHA256"] = json_digest(index)
        # Simulates further writes by the old executor: they must not reach the
        # frozen checkout, verification tree, or integrated commit.
        try:
            (self.candidate / "src/payload.txt").write_bytes(b"post-freeze executor mutation\n")
        except PermissionError:
            # Revoking old executor access is also a valid freeze boundary.
            pass
        if case == "missing_expected":
            self.expected.unlink()
        elif case == "missing_gate":
            original = (self.package / "verify.toml").read_text()
            (self.package / "verify.toml").write_text(original[:original.rfind("[[checks]]")])
        elif case == "context_index_tamper":
            altered = dict(index, tree="0" * 40)
            write_json(self.run_dir / "context-index.json", altered)
        elif case == "context_member_tamper":
            name = "contexts/CHK-002.json"
            write_json(self.run_dir / name, dict(contexts[name], output_id="CHK-001.log"))
        elif case == "context_member_missing":
            (self.run_dir / "contexts/CHK-002.json").unlink()
        elif case == "context_member_extra":
            write_json(self.run_dir / "contexts/CHK-999.json", contexts["contexts/CHK-001.json"])
        elif case == "context_member_swap":
            first, second = self.run_dir / "contexts/CHK-001.json", self.run_dir / "contexts/CHK-002.json"
            original = first.read_bytes()
            first.write_bytes(second.read_bytes())
            second.write_bytes(original)
        elif case == "context_cross_run":
            name = "contexts/CHK-002.json"
            write_json(self.run_dir / name, dict(contexts[name], run_id="unrelated-run"))
        failures = {"failed_worker": "worker", "done_nonzero_worker": "worker", "missing_worker_result": "incomplete",
                    "candidate_expected": "parity", "missing_expected": "integrity", "missing_gate": "integrity",
                    "incomplete_progress": "progress"}
        failures.update({name: "integrity" for name in ["context_index_tamper", "context_member_tamper", "context_member_missing",
                                                        "context_member_extra", "context_member_swap", "context_cross_run"]})
        if case in failures:
            self.reject("verify", "--run", self.run_dir, category=failures[case])
            return
        before_verify = self.protected()
        self.call("verify", "--run", self.run_dir)
        self.observer.require_success(self.run_dir)
        self.validate_contexts(index, contexts)
        require(self.protected() == before_verify, "verify changed accepted state before integration")
        verdict_bytes = regular_beneath(self.run_dir, "verdict.json").read_bytes()
        verdict = strict_json(verdict_bytes.decode())
        require(verdict.get("tree") == expected_tree and verdict.get("parent") == self.parent, "verdict tree binding incorrect")
        require(verdict.get("scope_base") == self.scope_base and verdict.get("task_id") == self.task and
                verdict.get("run_id") == self.observer.nonce and verdict.get("context_index_sha256") == json_digest(index),
                "verdict context binding incorrect")
        require(verdict.get("status") == "passed", "missing passed verdict")
        checks = verdict.get("checks", [])
        require(len(checks) == 5 and {check["id"] for check in checks} == {f"CHK-{n:03d}" for n in range(1, 6)}, "missing/duplicate gate result")
        for check in checks:
            require(type(check.get("exit")) is int and check["exit"] == 0, "failed check accepted")
            log = regular_beneath(self.run_dir, check["log"])
            require(log.is_file() and digest(log) == check.get("log_sha256"), "missing/unbound check log")
            independently_captured = self.observer.events[2]["files"][check["id"] + ".log"]
            require(log.read_bytes() == base64.b64decode(independently_captured), "host fabricated a taskfmt log")
        for stage in ["build", "test"]:
            observation = strict_json(regular_beneath(self.run_dir, f"workers/{stage}/{stage}.json").read_text())
            require(observation.get("stage") == stage, "wrong worker result")
            require(observation.get("authority_env_present") is False, "authority environment reached candidate worker")
            require(observation.get("observer_env_present") is False, "observer authority reached candidate worker")
            for operation in ["writes", "reads"]:
                require(observation.get(operation) == {key: "denied" for key in self.targets[operation]},
                        f"{stage}: worker accessed trust input; temporary write restoration is still a failure")
        require(regular_beneath(self.run_dir, "workers/test/payload.txt").read_bytes() == b"qualified\n", "candidate artifact mismatch")
        self.protect_verified_run(verdict_bytes)
        if case == "unauthorized_seal":
            self.reject("seal", "--run", self.run_dir, "--product", "oracle-showcase", category="authority")
            return
        integrate_ref, expected_parent = REF, self.parent
        if case == "wrong_expected_parent":
            expected_parent = "0" * 40
        elif case == "stale_parent_cas":
            (self.repo / "src/advance.txt").write_bytes(b"concurrent integration\n")
            advanced = commit(self.repo, "Advance integration parent")
            git(self.repo, "update-ref", REF, advanced, self.parent)
        elif case == "wrong_integration_ref":
            integrate_ref = "refs/heads/main"
        arguments = ["--run", self.run_dir, "--ref", integrate_ref, "--expected-parent", expected_parent]
        if case in {"wrong_expected_parent", "stale_parent_cas", "wrong_integration_ref"}:
            self.reject("integrate", *arguments, category="authority" if case == "wrong_integration_ref" else "parent")
            return
        before_integrate = self.protected()
        self.validate_accepted_copy()
        self.call("integrate", *arguments)
        self.validate_accepted_copy()
        after_integrate = self.protected()
        for key in before_integrate.keys() - {"refs", "ledger"}:
            require(after_integrate[key] == before_integrate[key], "integration mutated protected " + key)
        prior_refs = dict(line.split(" ", 1) for line in before_integrate["refs"].splitlines())
        later_refs = dict(line.split(" ", 1) for line in after_integrate["refs"].splitlines())
        require({key: value for key, value in prior_refs.items() if key != REF} ==
                {key: value for key, value in later_refs.items() if key != REF}, "integration changed another ref")
        for group in ["ledger", "receipts"]:
            for name, value in before_integrate[group].items():
                require(after_integrate[group].get(name) == value, "integration overwrote accepted " + group)
        integrated = git(self.repo, "rev-parse", REF)
        append_name = integrated + ".json"
        require(set(after_integrate["ledger"]) - set(before_integrate["ledger"]) == {append_name}, "unauthorized ledger append")
        acceptance = strict_json(regular_beneath(self.campaign / "ledger", append_name).read_text())
        require(acceptance == {"schema": "tc-proof-host-acceptance/v1", "task": self.task,
                              "commit": integrated, "tree": expected_tree, "parent": self.parent,
                              "scope_base": self.scope_base, "verdict_sha256": self.accepted_verdict_sha256,
                              "dependencies": [digest(self.predecessor)]}, "ledger acceptance is not bound to authorized task and verdict")
        require(git(self.repo, "rev-parse", integrated + "^{tree}") == expected_tree, "integrated tree was not tested")
        require(git(self.repo, "ls-tree", integrated, "--", "src/obsolete.txt") == "", "integration restored deleted source")
        require(git(self.repo, "show", "-s", "--format=%P", integrated) == self.parent, "integrated parent was not tested")
        message = git(self.repo, "show", "-s", "--format=%B", integrated)
        require("Signed-off-by:" in message and "Co-authored-by: Codex <codex@openai.com>" in message, "required commit trailers missing")


def qualify_host(executable: Path, taskfmt: Path, source: Path) -> dict:
    require(executable.is_file() and os.access(executable, os.X_OK), "host executable required")
    vectors = strict_json((HERE / "host-bootstrap-vectors.json").read_text())
    cases = ["exact_tested_tree"] + [entry["id"] for entry in vectors["negative"]]
    frozen_inputs = {"fixture": snapshot(FIXTURE), "driver": digest(Path(__file__)),
                     "observer": digest(HERE / "host-bootstrap-observer.py"),
                     "vectors": digest(HERE / "host-bootstrap-vectors.json"), "host": digest(executable)}
    for case in cases:
        with tempfile.TemporaryDirectory(prefix="tc-host-bootstrap-") as directory:
            fixture = HostFixture(Path(directory), executable, taskfmt, source, case)
            try:
                fixture.execute()
            finally:
                fixture.observer.close()
        # A fresh positive after each single negative detects lingering global
        # state and rejects an always-fail implementation.
        if case != "exact_tested_tree":
            with tempfile.TemporaryDirectory(prefix="tc-host-positive-") as directory:
                fixture = HostFixture(Path(directory), executable, taskfmt, source, "exact_tested_tree")
                try:
                    fixture.execute()
                finally:
                    fixture.observer.close()
        require({"fixture": snapshot(FIXTURE), "driver": digest(Path(__file__)),
                 "observer": digest(HERE / "host-bootstrap-observer.py"),
                 "vectors": digest(HERE / "host-bootstrap-vectors.json"), "host": digest(executable)} == frozen_inputs,
                "qualification mutated bootstrap or submitted binary")
    return {"schema": "tc-host-bootstrap-qualification/v1", "host_sha256": digest(executable), "cases": cases}


def qualify_observer(taskfmt: Path, source: Path) -> dict:
    """Prove the protected observer works and the exact reviewer forgery fails."""
    results = []
    with tempfile.TemporaryDirectory(prefix="tc-observer-qualification-") as directory:
        root = Path(directory)
        substitute = root / "substituting-host"
        shutil.copyfile(FIXTURE / "substituting-host.py", substitute)
        substitute.chmod(0o755)
        (root / "substitution").mkdir()
        substitution = HostFixture(root / "substitution", substitute, taskfmt, source, "exact_tested_tree")
        try:
            original = snapshot(substitution.candidate)
            substituted = substitution.observer.run_host([str(substitute), "prepare", "--campaign", str(substitution.campaign),
                        "--task", substitution.task, "--parent", substitution.parent, "--run", str(substitution.run_dir)],
                        substitution.env, substitution.root, writable=[substitution.run_dir])
            require(substituted.returncode != 0 and "PermissionError" in substituted.stderr and "worker.py" in substituted.stderr,
                    "probe substitution was not denied at write boundary: " + substituted.stderr)
            require(snapshot(substitution.candidate) == original, "submitting host replaced original probe")
            results.append("prepare_probe_substitution_denied")
        finally:
            substitution.observer.close()
        liar = root / "forged-host"
        shutil.copyfile(FIXTURE / "forged-host.py", liar)
        liar.chmod(0o755)
        (root / "case").mkdir()
        fixture = HostFixture(root / "case", liar, taskfmt, source, "exact_tested_tree")
        try:
            # Independently provision valid frozen inputs, then exercise only
            # the liar's verify operation. Its unrelated fake freeze logic is
            # not permitted to define the expected tree or probe bytes.
            fixture.run_dir.mkdir()
            progress = fixture.run_dir / "progress.md"
            initialized = run([taskfmt, "progress-init", fixture.package, "--config", source / "experiment.toml", "--out", progress])
            require(initialized.returncode == 0, "observer fixture progress-init failed")
            progress.write_text(complete_progress(progress.read_text()))
            tree = fixture.independent_tree()
            write_json(fixture.run_dir / "context.json", {"task": fixture.task, "parent": fixture.parent})
            write_json(fixture.run_dir / "freeze.json", {"tree": tree, "parent": fixture.parent, "scope_base": fixture.scope_base})
            write_json(fixture.run_dir / "fake.json", {"candidate": str(fixture.candidate)})
            fixture.observer.arm(fixture.candidate, progress, fixture.package, taskfmt, source / "experiment.toml",
                                 fixture.scope_base, tree, [fixture.campaign, fixture.authority, fixture.install, fixture.repo, fixture.observer.root / "sentinel"])
            fixture.call("verify", "--run", fixture.run_dir)
            try:
                fixture.observer.require_success(fixture.run_dir)
            except AssertionError as error:
                require("independent process execution" in str(error), "forgery rejected for unrelated reason: " + str(error))
            else:
                raise AssertionError("reviewer's zero-execution fabricated host passed")
            results.append("fabricated_host_rejected_for_missing_execution")

            # The positive facility check uses an independent client that only
            # requests operations; it contains no host freeze/integration logic.
            client = root / "case/client.py"
            client.write_text("""import json, os
with os.fdopen(int(os.environ['TC_PROOF_OBSERVER_REQUEST_FD']), 'w') as requests, os.fdopen(int(os.environ['TC_PROOF_OBSERVER_RESPONSE_FD'])) as responses:
    for step in ['build','test','taskfmt']:
        requests.write(json.dumps({'schema':'tc-proof-observer-request/v1','nonce':os.environ['TC_PROOF_OBSERVER_NONCE'],'step':step})+'\\n')
        requests.flush()
        result=json.loads(responses.readline())
        assert result['exit']==0, result
""")
            launched = fixture.observer.run_host([sys.executable, str(client)], fixture.env, fixture.root)
            require(launched.returncode == 0, "protected observer IPC failed: " + launched.stderr)
            for event in fixture.observer.events:
                require(event["tree"] == strict_json((fixture.run_dir / "freeze.json").read_text())["tree"], "observer tree changed")
                if event["step"] in {"build", "test"}:
                    observation = strict_json(base64.b64decode(event["files"][event["step"] + ".json"]).decode())
                    for operation in ["writes", "reads"]:
                        require(observation[operation] == {key: "denied" for key in fixture.targets[operation]}, "sandbox did not deny candidate access")
                    for name, encoded in event["files"].items():
                        output = fixture.run_dir / "workers" / event["step"] / name
                        output.parent.mkdir(parents=True, exist_ok=True)
                        output.write_bytes(base64.b64decode(encoded))
            fixture.observer.require_success(fixture.run_dir)
            results.append("real_build_test_taskfmt_observed")

            # Build a fully bound proof from the actual retained observations,
            # then exercise its publication boundary with a surviving child.
            check_records = []
            for number in range(1, 6):
                check = f"CHK-{number:03d}"
                name = f"actual-logs/{check}.log"
                path = fixture.run_dir / name
                path.parent.mkdir(exist_ok=True)
                path.write_bytes(base64.b64decode(fixture.observer.events[2]["files"][check + ".log"]))
                check_records.append({"id": check, "exit": 0, "log": name, "log_sha256": digest(path)})
            index, _ = fixture.check_contexts(tree)
            verdict = {"status": "passed", "tree": tree, "parent": fixture.parent, "scope_base": fixture.scope_base,
                       "task_id": fixture.task, "run_id": fixture.observer.nonce, "context_index_sha256": json_digest(index),
                       "checks": check_records}
            write_json(fixture.run_dir / "verdict.json", verdict)
            original_verdict = (fixture.run_dir / "verdict.json").read_bytes()
            trigger, completed = root / "case/late-trigger", fixture.run_dir / "late-completed"
            late_host = root / "case/late-host.py"
            child_code = """import pathlib,sys,time
target,trigger,completed=map(pathlib.Path,sys.argv[1:])
deadline=time.monotonic()+15
while not trigger.exists() and time.monotonic()<deadline: time.sleep(0.01)
if trigger.exists():
    target.write_bytes(target.read_bytes()+b' ')
    completed.write_text('changed')
"""
            late_host.write_text("import subprocess,sys\nsubprocess.Popen([sys.executable,'-c'," + repr(child_code) +
                                 ",*sys.argv[1:]],close_fds=True,start_new_session=True,stdin=subprocess.DEVNULL,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)\n")
            spawned = fixture.observer.run_host([sys.executable, str(late_host), str(fixture.run_dir / "verdict.json"), str(trigger), str(completed)],
                                                fixture.env, fixture.root, writable=[fixture.run_dir])
            require(spawned.returncode == 0, "surviving-child fixture could not start: " + spawned.stderr)
            fixture.protect_verified_run(original_verdict)
            trigger.write_bytes(b"go\n")
            deadline = time.monotonic() + 5
            while not completed.exists() and time.monotonic() < deadline:
                time.sleep(0.01)
            require(completed.exists(), "old-rights child did not exercise publication mutation")
            try:
                fixture.validate_accepted_copy()
            except AssertionError:
                pass
            else:
                raise AssertionError("surviving child changed accepted proof without detection")
            require((fixture.accepted_proof / "verdict.json").read_bytes() == original_verdict, "surviving child changed private authority")
            (fixture.run_dir / "verdict.json").write_bytes(original_verdict)
            completed.unlink()
            fixture.validate_accepted_copy()
            results.append("surviving_child_cannot_rebind_private_proof")

            proof_attack = root / "case/proof-attack.py"
            proof_attack.write_text("""import json,pathlib,sys
result={}
for name in sys.argv[1:]:
    path=pathlib.Path(name)
    try:
        with path.open('r+b') as stream:
            original=stream.read(); stream.seek(0); stream.write(b'attack'); stream.seek(0); stream.write(original); stream.truncate()
        result[name]='succeeded'
    except PermissionError: result[name]='denied'
print(json.dumps(result))
""")
            protected_files = [fixture.run_dir / "verdict.json", fixture.run_dir / "actual-logs/CHK-001.log", fixture.accepted_proof / "verdict.json"]
            attempted = fixture.observer.run_host([sys.executable, str(proof_attack), *map(str, protected_files)], fixture.env, fixture.root)
            require(attempted.returncode == 0 and strict_json(attempted.stdout) == {str(path): "denied" for path in protected_files},
                    "new host invocation could alter accepted proof: " + attempted.stderr)
            fixture.validate_accepted_copy()
            results.append("accepted_verdict_and_logs_write_denied")

            attack = root / "case/observer-attack.py"
            attack.write_text("""import ctypes, errno, json, os, pathlib, socket, sys
path=pathlib.Path(sys.argv[1]); target=int(sys.argv[2]); result={}
for mode in ['rb','r+b']:
    try:
        with path.open(mode) as stream:
            if mode=='rb': stream.read()
            else:
                original=stream.read(); stream.seek(0); stream.write(b'attack'); stream.seek(0); stream.write(original); stream.truncate()
        result[mode]='succeeded'
    except PermissionError: result[mode]='denied'
try: os.kill(target,0); result['signal']='succeeded'
except PermissionError: result['signal']='denied'
lib=ctypes.CDLL(None); port=ctypes.c_uint()
result['task_port']='denied' if lib.task_for_pid(lib.mach_task_self(),target,ctypes.byref(port))!=0 else 'succeeded'
try: pathlib.Path(sys.argv[3]).read_bytes(); result['ambient_read']='succeeded'
except PermissionError: result['ambient_read']='denied'
try:
    sock=socket.socket(); code=sock.connect_ex(('127.0.0.1',9)); sock.close()
    result['network']='denied' if code in [errno.EPERM,errno.EACCES] else 'succeeded'
except PermissionError: result['network']='denied'
print(json.dumps(result))
""")
            before = (fixture.observer.root / "sentinel").read_bytes()
            ambient = root / "outside-secret"
            ambient.write_bytes(os.urandom(32))
            attacked = fixture.observer.run_host([sys.executable, str(attack), str(fixture.observer.root / "sentinel"), str(os.getpid()), str(ambient)], fixture.env, fixture.root)
            require(attacked.returncode == 0, "observer boundary probe failed: " + attacked.stderr)
            require(strict_json(attacked.stdout) == {name: "denied" for name in ["rb", "r+b", "signal", "task_port", "network", "ambient_read"]},
                    "submitted host can access observer authority: " + attacked.stdout)
            require((fixture.observer.root / "sentinel").read_bytes() == before, "observer sentinel changed")
            results.append("observer_read_write_signal_task_port_network_denied")
            results.append("ambient_secret_read_denied")
            # A second invocation cannot replay the valid three-step transcript.
            replay = fixture.observer.run_host([sys.executable, str(client)], fixture.env, fixture.root)
            require(replay.returncode != 0 and fixture.observer.violations, "observer replay accepted")
            results.append("observer_replay_rejected")
        finally:
            fixture.observer.close()
    return {"schema": "tc-proof-observer-qualification/v1", "cases": results,
            "observer_sha256": digest(HERE / "host-bootstrap-observer.py"), "sandbox_sha256": digest(Path("/usr/bin/sandbox-exec")),
            "python_sha256": digest(Path(sys.executable)), "taskfmt_sha256": digest(taskfmt),
            "git_sha256": digest(Path("/Library/Developer/CommandLineTools/usr/bin/git"))}


def host_result(result: subprocess.CompletedProcess, operation: str, category: str | None = None) -> dict:
    require(bool(result.stdout.strip()), f"host {operation} returned no result: exit={result.returncode} {result.stderr}")
    document = strict_json(result.stdout)
    require(document.get("schema") == "tc-proof-host-result/v1", "unsupported host result schema")
    require(document.get("operation") == operation, "wrong operation binding")
    if category is None:
        require(result.returncode == 0 and document.get("status") == "passed", "operation did not pass")
    else:
        require(result.returncode != 0 and document.get("status") == "rejected", "negative case accepted")
        require(document.get("category") == category, "wrong failure category")
    return document


class DriverSelfTests(unittest.TestCase):
    def test_context_index_and_members_fail_closed(self):
        with tempfile.TemporaryDirectory(prefix="tc-context-test-") as directory:
            root = Path(directory)
            (root / "experiment.toml").write_text("# synthetic parser test\n")
            (root / "case").mkdir()
            fixture = HostFixture(root / "case", Path(sys.executable), Path(sys.executable), root, "exact_tested_tree")
            try:
                original_index = git(fixture.candidate, "write-tree")
                tree = fixture.independent_tree()
                self.assertNotEqual(tree, original_index)
                self.assertNotEqual(fixture.scope_base, fixture.parent)
                self.assertEqual(git(fixture.candidate, "write-tree"), original_index)
                self.assertEqual(git(fixture.candidate, "ls-files", "--", "src/obsolete.txt"), "src/obsolete.txt")
                self.assertEqual(git(fixture.observer.root / "expected-tree", "ls-tree", tree, "--", "src/obsolete.txt"), "")
                index, contexts = fixture.check_contexts(tree)
                fixture.run_dir.mkdir()
                for mode in ["positive", "index", "member", "missing", "extra", "swap", "cross-run", "output-alias"]:
                    for existing in (fixture.run_dir / "contexts").glob("*"):
                        existing.unlink()
                    write_json(fixture.run_dir / "context-index.json", index)
                    for name, value in contexts.items():
                        write_json(fixture.run_dir / name, value)
                    if mode == "index":
                        write_json(fixture.run_dir / "context-index.json", dict(index, tree="0" * 40))
                    elif mode in {"member", "cross-run", "output-alias"}:
                        key, value = {"member": ("operation", "other"), "cross-run": ("run_id", "other"), "output-alias": ("output_id", "CHK-001.log")}[mode]
                        name = "contexts/CHK-002.json"
                        write_json(fixture.run_dir / name, dict(contexts[name], **{key: value}))
                    elif mode == "missing":
                        (fixture.run_dir / "contexts/CHK-002.json").unlink()
                    elif mode == "extra":
                        write_json(fixture.run_dir / "contexts/extra.json", {})
                    elif mode == "swap":
                        write_json(fixture.run_dir / "contexts/CHK-002.json", contexts["contexts/CHK-001.json"])
                    if mode == "positive":
                        fixture.validate_contexts(index, contexts)
                    else:
                        with self.subTest(mode=mode), self.assertRaises(AssertionError):
                            fixture.validate_contexts(index, contexts)
            finally:
                fixture.observer.close()

    def test_success_strings_never_suffice(self):
        for stdout, exit_code in [("DONE\n", 0), ("{}", 0), ("not json", 7)]:
            with self.subTest(stdout=stdout), self.assertRaises((AssertionError, ValueError)):
                host_result(subprocess.CompletedProcess([], exit_code, stdout, ""), "freeze")

    def test_always_pass_host_fails_negative(self):
        document = '{"schema":"tc-proof-host-result/v1","operation":"freeze","status":"passed"}'
        with self.assertRaises(AssertionError):
            host_result(subprocess.CompletedProcess([], 0, document, ""), "freeze", "scope")

    def test_always_fail_host_fails_positive(self):
        document = '{"schema":"tc-proof-host-result/v1","operation":"freeze","status":"rejected","category":"scope"}'
        with self.assertRaises(AssertionError):
            host_result(subprocess.CompletedProcess([], 1, document, ""), "freeze")

    def test_duplicate_fields_rejected(self):
        with self.assertRaises(AssertionError):
            strict_json('{"status":"rejected","status":"passed"}')

    def test_done_with_nonzero_is_failure(self):
        self.assertFalse(taskfmt_pass(subprocess.CompletedProcess([], 7, "DONE\n", "")))

    def test_repository_is_independent(self):
        with tempfile.TemporaryDirectory(prefix="tc-host-selftest-") as temporary:
            root = Path(temporary) / "work"
            parent = repository(root)
            self.assertEqual(git(root, "rev-parse", REF), parent)
            self.assertNotEqual(root.resolve(), HERE.parents[2])

    def test_external_lying_hosts_cannot_qualify(self):
        # These are intentionally incorrect executables, not a reference host.
        # Execute them across the actual CLI boundary and observe failure.
        for kind in ["always-pass", "always-fail", "malformed"]:
            with self.subTest(kind=kind), tempfile.TemporaryDirectory(prefix="tc-fake-host-") as directory:
                temporary = Path(directory)
                host = temporary / "host"
                if kind == "malformed":
                    body = "print('not json')\n"
                else:
                    passed = kind == "always-pass"
                    body = ("import json, sys\nprint(json.dumps({'schema':'tc-proof-host-result/v1',"
                            "'operation':sys.argv[1],'status':" + repr("passed" if passed else "rejected") +
                            ",'category':'scope'}))\nraise SystemExit(" + str(0 if passed else 1) + ")\n")
                host.write_text("#!" + sys.executable + "\n" + body)
                host.chmod(0o755)
                (temporary / "experiment.toml").write_text("# never executed by fake host\n")
                # The fake host fails before taskfmt is invoked. A real current
                # Python executable supplies an existing hashable fixture path.
                with self.assertRaises((AssertionError, ValueError)):
                    qualify_host(host, Path(sys.executable), temporary)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--taskfmt", type=Path)
    parser.add_argument("--taskfmt-source", type=Path)
    parser.add_argument("--host", type=Path)
    parser.add_argument("--observer-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        suite = unittest.defaultTestLoader.loadTestsFromTestCase(DriverSelfTests)
        require(unittest.TextTestRunner(verbosity=2).run(suite).wasSuccessful(), "driver self-tests failed")
    elif args.observer_test and args.taskfmt and args.taskfmt_source:
        qualify_taskfmt(args.taskfmt.resolve(), args.taskfmt_source.resolve())
        print(json.dumps(qualify_observer(args.taskfmt.resolve(), args.taskfmt_source.resolve()), sort_keys=True))
    elif args.host and args.taskfmt and args.taskfmt_source:
        qualify_taskfmt(args.taskfmt.resolve(), args.taskfmt_source.resolve())
        qualify_observer(args.taskfmt.resolve(), args.taskfmt_source.resolve())
        print(json.dumps(qualify_host(args.host.resolve(), args.taskfmt.resolve(), args.taskfmt_source.resolve()), sort_keys=True))
    elif args.taskfmt and args.taskfmt_source:
        print(json.dumps(qualify_taskfmt(args.taskfmt.resolve(), args.taskfmt_source.resolve()), sort_keys=True))
    else:
        parser.error("select --self-test or --taskfmt and --taskfmt-source")


if __name__ == "__main__":
    main()
