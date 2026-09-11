"""Additional frozen independent contracts for review findings VF03–05/PARITY04."""
from __future__ import annotations
import ast
import copy
import importlib.util
import json
import os
from pathlib import Path
import secrets
import shutil
import statistics
import subprocess
import sys

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("runner_base", HERE / "runner-bootstrap-driver.py")
B = importlib.util.module_from_spec(spec)
spec.loader.exec_module(B)
require, sha, canonical, save = B.require, B.sha, B.canonical, B.save


def sandbox_function():
    spec = importlib.util.spec_from_file_location("frozen_observer", HERE / "host-bootstrap-observer.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.sandbox


def source_contract(source):
    text = source.decode()
    lines = text.splitlines(keepends=True)
    methods = [node for node in ast.walk(ast.parse(text)) if isinstance(node, ast.FunctionDef) and node.name.startswith("test_")]
    spans = {node.name: [node.lineno, node.end_lineno] for node in methods}
    bodies = {name: sha("".join(lines[start - 1:end]).encode()) for name, (start, end) in spans.items()}
    covered = {index for start, end in spans.values() for index in range(start - 1, end)}
    projection = sha("".join(line for index, line in enumerate(lines) if index not in covered).encode())
    return {"source_sha256": sha(source), "spans": spans, "assertions": bodies, "non_test_sha256": projection}


class ExtendedFixture(B.Fixture):
    def __init__(self, family, mutation):
        operation = {"accounting": "account-tests", "performance": "architecture", "native": "oracle"}[family]
        super().__init__(operation, "valid")
        self.family, self.ext_mutation = family, mutation
        self.sandbox = sandbox_function()
        self.contract = {"schema": "tc-proof-runner-extension/v1", "family": family}
        self.script = self.private / "instrument.py"
        self.worker_config = {}
        if family == "accounting":
            original = (HERE / "runner-bootstrap-accounting.py").read_bytes()
            migration = mutation.startswith("preparation_migration")
            if migration:
                original = original.replace(b'self.assertEqual(production.future_value(config["package"]), 7)', b'self.assertEqual(production.future_value(config["package"]), 0)')
            source = original.decode()
            if migration:
                source = source.replace('self.assertEqual(production.future_value(config["package"]), 0)', 'self.assertEqual(production.future_value(config["package"]), 7)')
            if mutation == "weak_assertion":
                source = source.replace("self.assertEqual(production_value(), 7)", "pass")
            elif mutation == "approved_replacement":
                source = source.replace("self.assertEqual(production_value(), 7)", "self.assertEqual(production_value(), 7)\n        self.assertIsInstance(production_value(), int)")
            elif mutation == "outside_span":
                source = source.replace("return production.value()", "return 8")
            elif mutation == "ignored_test":
                source = source.replace("    def test_compatible(self):", "    @unittest.skip('suppressed')\n    def test_compatible(self):")
            self.script.write_text(source)
            if migration:
                self.approved_script = self.private / "approved.py"
                self.approved_script.write_text(source)
            self.archive = self.private / "original.py"
            self.archive.write_bytes(original if mutation != "archive_changed" else original + b"# altered archive\n")
            original_contract = source_contract(original)
            self.contract.update(mode="preparation" if mutation.startswith("preparation") else "production",
                original=original_contract, approved_replacements={}, accepted_inventory=True, accepted_disposition=True,
                required=[{"package": package, "target": target, "profile": profile, "name": name,
                           "source_commit": self.source_commit} for package, target, profile in
                          [("tiny-a", "unit", "primary"), ("tiny-b", "integration", "msrv")]
                          for name in ["test_compatible", "test_future"]],
                future=[], closed=[], proposals={})
            if mutation == "approved_replacement":
                self.contract["approved_replacements"] = source_contract(self.script.read_bytes())["assertions"]
            if migration and mutation != "preparation_migration_unapproved":
                self.contract["approved_replacements"] = source_contract(self.script.read_bytes())["assertions"]
            if mutation == "preparation_migration_patch_swap":
                self.script.write_text(self.script.read_text().replace("self.assertEqual(production_value(), 7)", "pass"))
            if mutation.startswith("preparation"):
                self.contract.update(accepted_inventory=False, accepted_disposition=False,
                    preparation_register={"required": copy.deepcopy(self.contract["required"]), "future": []})
            if mutation == "preparation_self_approved":
                self.contract["proposals"] = {"accept": True, "required": [], "future": ["test_compatible", "test_future"]}
            elif mutation.startswith("preparation_proposal"):
                self.contract["proposals"] = {"observed_inventory": copy.deepcopy(self.contract["required"])}
            if mutation == "preparation_omitted_inventory":
                self.contract["preparation_register"]["required"].pop()
            if mutation == "preparation_omitted_disposition":
                del self.contract["preparation_register"]["future"]
            if mutation == "production_missing_receipt":
                self.contract["accepted_disposition"] = False
            if mutation == "broadened_span":
                self.contract["proposals"] = {"spans": {"test_compatible": [1, 9999]}}
            self.worker_config = {"corrected": mutation in {"all_green", "real_correction"},
                                  "classification_error": mutation == "classification"}
            self.production = self.private / "production.py"
            values = {"tiny-a": 7 if self.worker_config["corrected"] else 0, "tiny-b": 7 if self.worker_config["corrected"] else 0}
            if mutation in {"mixed_closed", "borrowed_allowance", "borrowed_closure"}:
                values["tiny-a"] = 7
            elif mutation == "other_target_closed":
                values["tiny-b"] = 7
            self.production.write_text("def value():\n    return 7\n\ndef future_value(package):\n    return " + repr(values) + "[package]\n")
        elif family == "native":
            original = (HERE / "runner-bootstrap-native.py").read_bytes()
            self.script.write_bytes(original.replace(b'assert expected in h.rows()[row], "original fixed-row assertion"',
                b'pass # removed native assertion') if mutation == "native_assertion_removed" else original)
            if mutation == "changed_native_predicate":
                self.script.write_bytes(original.replace(b'assert expected in h.rows()[row], "original fixed-row assertion"', b'assert bool(h.rows()), "weakened native predicate"'))
            self.contract.update(source_sha256=sha(original), native_sizes=[[120, 40], [100, 30]],
                resized_sizes=[[80, 24], [100, 30], [120, 40], [160, 50]], mapping_owner=self.source_commit,
                mapping={"footer_row": "height - 2", "pointer": [2, "height - 2"]})
            if mutation == "candidate_mapping_owner":
                self.contract["mapping_owner"] = self.source_tree
            if mutation == "missing_mapping":
                self.contract["mapping"] = {}
            self.worker_config = {"wrong_mapping": mutation == "wrong_mapping",
                "candidate_geometry": mutation in {"candidate_geometry", "candidate_retargets"},
                "candidate_retargets": mutation == "candidate_retargets", "reuse_fixed_row": mutation == "reuse_fixed_row"}
        else:
            source = self.private / "performance.rs"
            source.write_bytes((HERE / "runner-bootstrap-performance.rs").read_bytes())
            resolved = subprocess.run([shutil.which("rustup") or "rustup", "which", "rustc"], check=True,
                                      stdout=subprocess.PIPE, text=True).stdout.strip()
            self.compiler = Path(resolved).resolve()
            self.executable = self.private / "performance"
            argv = [str(self.compiler), "--edition=2024", "-Dwarnings", "-Copt-level=3", str(source), "-o", str(self.executable)]
            subprocess.run(argv, check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            self.contract.update(source_sha256=sha(source.read_bytes()), executable_sha256=sha(self.executable.read_bytes()),
                compiler_sha256=sha(self.compiler.read_bytes()), profile="release", serial=True, strict=True,
                denominator="frame_ns", counter_owner="global_allocator", draws=8, allocations=8,
                style_calls=8, work=16_001_600, style_ratio_max=0.05)
            if mutation in {"debug_profile", "parallel_profile", "strict_disabled", "wrong_denominator", "wrong_counter_owner"}:
                key, value = {"debug_profile": ("profile", "debug"), "parallel_profile": ("serial", False),
                    "strict_disabled": ("strict", False), "wrong_denominator": ("denominator", "work"),
                    "wrong_counter_owner": ("counter_owner", "candidate_report")}[mutation]
                self.contract[key] = value
        if family in {"accounting", "native"}:
            # Historical identities point at a commit containing the actual
            # original assertion bytes, not merely an unrelated source commit.
            committed = self.source / "qualification.py"
            committed.write_bytes(original)
            production_original = b"def value():\n    return 7\n\ndef future_value(package):\n    return {'tiny-a': 0, 'tiny-b': 0}[package]\n"
            if family == "accounting":
                (self.source / "production.py").write_bytes(production_original)
            B.git(self.source, "add", ".")
            B.git(self.source, "-c", "user.name=Qualification", "-c", "user.email=qualification@example.invalid",
                  "-c", "commit.gpgsign=false", "commit", "-q", "-s", "-m", "Original qualification assertions",
                  "-m", "Co-authored-by: Codex <codex@openai.com>")
            self.source_commit = B.git(self.source, "rev-parse", "HEAD")
            self.context["oracle_commit"] = self.source_commit
            self.context["oracle_tree"] = B.git(self.source, "rev-parse", "HEAD^{tree}")
            B.git(self.source, "bundle", "create", str(self.public / "oracle.bundle"), "HEAD")
            self.context["bundle_sha256"] = sha((self.public / "oracle.bundle").read_bytes())
            committed.write_bytes(self.script.read_bytes())
            if family == "accounting":
                (self.source / "production.py").write_bytes(self.production.read_bytes())
            B.git(self.source, "add", ".")
            B.git(self.source, "-c", "user.name=Qualification", "-c", "user.email=qualification@example.invalid",
                  "-c", "commit.gpgsign=false", "commit", "--allow-empty", "-q", "-s", "-m", "Submitted qualification source",
                  "-m", "Co-authored-by: Codex <codex@openai.com>")
            self.source_tree = B.git(self.source, "rev-parse", "HEAD^{tree}")
            self.context["tree"] = self.source_tree
            if family == "accounting":
                for item in self.contract["required"]:
                    item["source_commit"] = self.source_commit
                if "preparation_register" in self.contract:
                    for item in self.contract["preparation_register"]["required"]:
                        item["source_commit"] = self.source_commit
                for item in self.contract["proposals"].get("observed_inventory", []):
                    item["source_commit"] = self.source_commit
            elif mutation != "candidate_mapping_owner":
                self.contract["mapping_owner"] = self.source_commit
        if family == "accounting":
            self.future_expected = [{"identity": copy.deepcopy(identity), "owner": "terminal-components/completion/" +
                ("991" if identity["package"] == "tiny-a" else "992"), "classification": "failed"}
                for identity in self.contract["required"] if identity["name"] == "test_future"]
            self.contract["future"] = copy.deepcopy(self.future_expected)
            if mutation in {"all_green", "closed_regression"}:
                self.contract["future"] = []
                self.contract["closed"] = [copy.deepcopy(row["identity"]) for row in self.future_expected]
            elif mutation in {"mixed_closed", "borrowed_allowance", "borrowed_closure", "other_target_closed"}:
                closed_package = "tiny-b" if mutation == "other_target_closed" else "tiny-a"
                self.contract["future"] = [copy.deepcopy(row) for row in self.future_expected if row["identity"]["package"] != closed_package]
                self.contract["closed"] = [copy.deepcopy(row["identity"]) for row in self.future_expected if row["identity"]["package"] == closed_package]
                if mutation == "borrowed_allowance":
                    self.contract["future"] = [copy.deepcopy(row) for row in self.future_expected if row["identity"]["package"] == closed_package]
                elif mutation == "borrowed_closure":
                    self.contract["closed"] = [copy.deepcopy(row["identity"]) for row in self.future_expected if row["identity"]["package"] != closed_package]
            elif mutation == "borrowed_owner":
                self.contract["future"][1]["owner"] = self.contract["future"][0]["owner"]
            if "preparation_register" in self.contract and mutation != "preparation_omitted_disposition":
                self.contract["preparation_register"]["future"] = copy.deepcopy(self.future_expected)
            if mutation == "preparation_proposal_omitted":
                self.contract["proposals"]["observed_inventory"].pop()
            elif mutation == "preparation_proposal_duplicate":
                self.contract["proposals"]["observed_inventory"].append(copy.deepcopy(self.contract["proposals"]["observed_inventory"][0]))
            elif mutation == "preparation_proposal_wrong_identity":
                self.contract["proposals"]["observed_inventory"][0]["profile"] = "foreign-profile"
        if family == "accounting" and mutation.startswith("preparation_migration"):
            self.prepatch = {"parent": self.source_commit, "source_sha256": sha(self.archive.read_bytes()),
                "runs": self.accounting_runs(self.archive, "prepatch")}
            candidate_source = (self.source / "qualification.py").read_bytes()
            (self.source / "qualification.py").write_bytes(self.approved_script.read_bytes())
            B.git(self.source, "add", "qualification.py")
            approved_tree = B.git(self.source, "write-tree")
            (self.source / "qualification.py").write_bytes(candidate_source)
            B.git(self.source, "add", "qualification.py")
            require(B.git(self.source, "write-tree") == self.source_tree, "rehearsal changed submitted tree")
            self.patch_manifest = {"parent": self.source_commit, "tree": approved_tree,
                "original_source_sha256": sha(self.archive.read_bytes()), "patched_source_sha256": sha(self.approved_script.read_bytes()),
                "production_sha256": sha(self.production.read_bytes()), "approved_assertions": source_contract(self.approved_script.read_bytes())["assertions"]}
            # Both complete POST-patch worker processes finish here, before any
            # submitted runner can be invoked or request an observer operation.
            self.postpatch = {"schema": "tc-proof-approved-rehearsal/v1", "parent": self.source_commit, "tree": approved_tree,
                "source_sha256": sha(self.approved_script.read_bytes()), "production_sha256": sha(self.production.read_bytes()),
                "patch_manifest_sha256": sha(canonical(self.patch_manifest)), "runs": self.accounting_runs(self.approved_script, "postpatch")}
            self.postpatch_expectations = []
            for run in self.postpatch["runs"]:
                for result in run["result"]["results"]:
                    identity = {key: run[key] for key in ("package", "target", "profile", "source_commit")}
                    identity["name"] = result["name"]
                    owner = next((row["owner"] for row in self.future_expected if row["identity"] == identity), None)
                    self.postpatch_expectations.append({"identity": identity, "status": result["status"],
                        "owner": owner if result["status"] != "passed" else None})
            self.contract["approved_patch_parent"] = self.source_commit
            self.contract["approved_patch_manifest"] = copy.deepcopy(self.patch_manifest)
            self.contract["prepatch"] = copy.deepcopy(self.prepatch)
            self.contract["postpatch"] = copy.deepcopy(self.postpatch)
            self.contract["postpatch_expectations"] = copy.deepcopy(self.postpatch_expectations)
            if mutation == "preparation_migration_wrong_parent":
                self.contract["approved_patch_parent"] = "0" * 40
            elif mutation == "preparation_migration_hidden_prepatch":
                del self.contract["prepatch"]
            elif mutation == "preparation_migration_partial_prepatch":
                self.contract["prepatch"]["runs"].pop()
            elif mutation == "preparation_migration_missing_postpatch":
                del self.contract["postpatch"]
            elif mutation == "preparation_migration_forged_postpatch":
                self.contract["postpatch"]["runs"][0]["result"]["results"][1]["status"] = "passed"
            elif mutation == "preparation_migration_forged_register":
                self.contract["postpatch_expectations"][1]["status"] = "passed"
            elif mutation == "preparation_migration_wrong_postpatch_tree":
                self.contract["postpatch"]["tree"] = "0" * 40
        self.context["qualification"] = self.contract
        save(self.context_path, self.context)
        self.context_hash = sha(self.context_path.read_bytes())
        self.before = self.snapshot()

    def accounting_runs(self, script, label):
        runs = []
        for package, target, profile in [("tiny-a", "unit", "primary"), ("tiny-b", "integration", "msrv")]:
            config = self.private / (label + "-" + package + ".json")
            save(config, dict(self.worker_config, package=package))
            result = subprocess.run(["/usr/bin/sandbox-exec", "-p", self.sandbox(writable=[], unreadable=[self.public],
                readable=[script, self.production, config]), sys.executable, "-I", str(script), str(config)],
                stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=20, check=True)
            runs.append({"package": package, "target": target, "profile": profile, "source_commit": self.source_commit,
                "rehearsal_run_id": secrets.token_hex(24), "exit": result.returncode, "result": B.load_bytes(result.stdout),
                "stdout_sha256": sha(result.stdout), "stderr_sha256": sha(result.stderr)})
        return runs

    def launch(self, request):
        require(set(request) == {"schema", "nonce", "operation", "source_commit", "tree"}, "observer fields")
        require(request == {"schema": "tc-proof-runner-observe/v1", "nonce": self.nonce, "operation": self.operation,
                            "source_commit": self.source_commit, "tree": self.source_tree}, "observer binding")
        require(not self.events, "observer replay")
        config = self.private / "worker.json"
        save(config, self.worker_config)
        if self.family == "performance":
            mode = {"allocation_perturbation": "allocation", "style_attribution": "heavy-style",
                    "smoke_digest": "smoke", "noop_workload": "noop"}.get(self.ext_mutation, "valid")
            measurements = []
            for _ in range(3):
                result = subprocess.run(["/usr/bin/sandbox-exec", "-p", self.sandbox(writable=[], unreadable=[self.public],
                    readable=[self.executable]), str(self.executable), mode], stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                    env={"PATH": "/usr/bin:/bin", "LC_ALL": "C", "TC_PERF_STRICT": "1"}, timeout=20, check=True)
                measurements.append(B.load_bytes(result.stdout))
            payload = {"measurements": measurements, "source_sha256": self.contract["source_sha256"],
                       "executable_sha256": sha(self.executable.read_bytes()), "profile": "release", "serial": True, "strict": True}
        else:
            runs = []
            identities = [("tiny-a", "unit", "primary"), ("tiny-b", "integration", "msrv")] if self.family == "accounting" else [None]
            for identity in identities:
                if identity:
                    save(config, dict(self.worker_config, package=identity[0]))
                result = subprocess.run(["/usr/bin/sandbox-exec", "-p", self.sandbox(writable=[], unreadable=[self.public],
                    readable=[self.script, config, *([self.production] if self.family == "accounting" else [])]),
                    sys.executable, "-I", str(self.script), str(config)],
                    stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=20, check=False)
                row = {"exit": result.returncode, "stdout": B.load_bytes(result.stdout) if result.returncode == 0 else None,
                       "stderr_sha256": sha(result.stderr)}
                if identity:
                    row.update(package=identity[0], target=identity[1], profile=identity[2], source_commit=self.source_commit, run_id=self.run_id)
                runs.append(row)
            payload = {"runs": runs, "source": source_contract(self.script.read_bytes()) if self.family == "accounting" else sha(self.script.read_bytes())}
            if self.family == "accounting":
                payload["archive_sha256"] = sha(self.archive.read_bytes())
                if self.ext_mutation == "duplicate_identity":
                    runs[0]["stdout"]["results"].append(runs[0]["stdout"]["results"][0])
                elif self.ext_mutation == "wrong_source":
                    runs[0]["source_commit"] = "0" * 40
                elif self.ext_mutation == "wrong_run":
                    runs[0]["run_id"] = secrets.token_hex(24)
                elif self.ext_mutation == "colliding_qualifier":
                    runs[1].update(package=runs[0]["package"], target=runs[0]["target"], profile=runs[0]["profile"])
        event = {"operation": self.operation, "run_id": self.run_id, "tree": self.source_tree,
                 "source_commit": self.source_commit, "payload": payload}
        self.events.append(event)
        return event

    def verify_observed(self):
        require(len(self.events) == 1, "missing independent execution")
        payload = self.events[0]["payload"]
        contract = self.contract
        if self.family == "accounting":
            original, actual = contract["original"], payload["source"]
            require(payload["archive_sha256"] == original["source_sha256"], "archive changed")
            require(actual["non_test_sha256"] == original["non_test_sha256"], "non-test projection changed")
            require(actual["assertions"] == (contract["approved_replacements"] or original["assertions"]), "assertion body changed")
            require(not contract["proposals"] or set(contract["proposals"]) == {"observed_inventory"}, "candidate proposal has no acceptance authority")
            if "observed_inventory" in contract["proposals"]:
                require(sorted(map(canonical, contract["proposals"]["observed_inventory"])) == sorted(map(canonical, contract["required"])),
                        "proposed inventory must exactly match independently required qualified identities")
            if contract["mode"] == "production":
                require(contract["accepted_inventory"] and contract["accepted_disposition"], "production receipts absent")
            else:
                require(contract.get("preparation_register") == {"required": contract["required"], "future": self.future_expected}, "preparation authority incomplete")
            required_ids = {canonical(identity) for identity in contract["required"]}
            future = {canonical(row["identity"]): row for row in contract["future"]}
            closed = {canonical(identity) for identity in contract["closed"]}
            require(len(future) == len(contract["future"]) and len(closed) == len(contract["closed"]) and
                    set(future).issubset(required_ids) and closed.issubset(required_ids) and not set(future).intersection(closed),
                    "qualified policy membership or overlap")
            for key, row in future.items():
                require(row in self.future_expected, "failure owner/classification borrowed across qualified identities")
            if self.ext_mutation.startswith("preparation_migration"):
                require(contract["approved_patch_parent"] == self.source_commit and contract.get("prepatch") == self.prepatch,
                        "approved migration parent or prepatch evidence missing")
                prepatch_ids = []
                for run in self.prepatch["runs"]:
                    for row in run["result"]["results"]:
                        require(row["status"] == "passed", "prepatch diagnostic not preserved")
                        prepatch_ids.append({"package": run["package"], "target": run["target"], "profile": run["profile"],
                            "source_commit": self.source_commit, "name": row["name"]})
                require(sorted(map(canonical, prepatch_ids)) == sorted(map(canonical, contract["required"])), "prepatch inventory incomplete")
                require(contract.get("approved_patch_manifest") == self.patch_manifest and contract.get("postpatch") == self.postpatch,
                        "missing or forged actual postpatch rehearsal")
                require(contract["postpatch_expectations"] == self.postpatch_expectations and
                        sorted(canonical(row["identity"]) for row in self.postpatch_expectations) == sorted(required_ids),
                        "postpatch register must exhaust actual qualified outcomes")
            for receipt in contract.get("accepted_stage_receipts", []):
                require(receipt["test_source_sha256"] == original["source_sha256"], "stage assertion lineage")
                require(all(canonical(identity) in closed and canonical(identity) not in future for identity in receipt["closed"]),
                        "accepted closure cannot be reopened by former allowance")
            seen = []
            failures = []
            for run in payload["runs"]:
                require(run["exit"] == 0 and run["run_id"] == self.run_id and run["source_commit"] == self.source_commit, "worker identity")
                for result in run["stdout"]["results"]:
                    identity = {key: run[key] for key in ("package", "target", "profile", "source_commit")}
                    identity["name"] = result["name"]
                    seen.append(identity)
                    if result["status"] != "passed":
                        key = canonical(identity)
                        require(key in future and result["status"] == future[key]["classification"] and key not in closed,
                                "failure classification or qualified closed regression")
                        failures.append(identity)
            require(sorted(map(canonical, seen)) == sorted(map(canonical, contract["required"])), "complete unique qualified inventory")
            return {"observations": self.events, "unresolved": failures}
        if self.family == "performance":
            require(contract["profile"] == "release" and contract["serial"] is True and contract["strict"] is True and
                    contract["denominator"] == "frame_ns" and contract["counter_owner"] == "global_allocator", "measurement profile/ownership")
            for row in payload["measurements"]:
                require(all(type(row[key]) is int and row[key] >= 0 for key in row), "measurement types")
                require(all(row[key] == contract[key] for key in ("draws", "allocations", "style_calls", "work")), "actual counters/workload")
                require(0 < row["style_ns"] <= row["frame_ns"], "clock attribution")
            require(statistics.median(row["style_ns"] / row["frame_ns"] for row in payload["measurements"]) <= 0.05, "measured style share")
        else:
            require(payload["source"] == contract["source_sha256"] and contract["mapping_owner"] == self.source_commit, "native source/assertion/mapping authority")
            require(contract["mapping"] == {"footer_row": "height - 2", "pointer": [2, "height - 2"]}, "required geometry mapping")
            require(payload["runs"][0]["exit"] == 0, "native assertion failed")
            observed = payload["runs"][0]["stdout"]
            require([(row["width"], row["height"], row["row"]) for row in observed["native"]] == [(120, 40, 38), (100, 30, 28)], "native dimensions/assertions")
            require([[row["width"], row["height"]] for row in observed["resized"]] == contract["resized_sizes"], "resized membership")
            for row in observed["native"] + observed["resized"]:
                require(row["row"] == row["height"] - 2 and row["pointer"] == [2, row["height"] - 2], "oracle-only geometry")
                require(len(row["rows"]) == row["height"] and all(len(line) == row["width"] for line in row["rows"]), "full frame")
                require(row.get("expected", "attached") in row["rows"][row["row"]], "mapped assertion and pointer effect")
        return {"observations": self.events}

    def validate(self, result, category):
        code, report = result
        if category:
            require(code != 0 and report["status"] == "rejected" and report["category"] == category and report["outputs"] == {}, "exact rejection")
            require(len(self.events) == 1, "negative omitted actual execution")
        else:
            require(code == 0 and report["status"] == "passed" and report["category"] is None, "positive result")
            require(canonical(report["outputs"]) == canonical(self.verify_observed()), "protected result mismatch")


def cases():
    rows = [(family, "valid", None) for family in ("accounting", "performance", "native")]
    rows += [("accounting", name, None) for name in ("approved_replacement", "all_green", "real_correction", "preparation_valid", "preparation_proposal", "preparation_migration",
        "mixed_closed", "other_target_closed")]
    rows += [("accounting", name, "TEST_ACCOUNTING") for name in (
        "weak_assertion", "outside_span", "ignored_test", "archive_changed", "broadened_span", "classification", "duplicate_identity",
        "wrong_source", "wrong_run", "colliding_qualifier", "closed_regression", "preparation_self_approved",
        "preparation_omitted_inventory", "preparation_omitted_disposition", "production_missing_receipt",
        "preparation_migration_unapproved", "preparation_migration_patch_swap", "preparation_migration_wrong_parent",
        "preparation_migration_hidden_prepatch", "preparation_migration_partial_prepatch",
        "preparation_migration_missing_postpatch", "preparation_migration_forged_postpatch",
        "preparation_migration_forged_register", "preparation_migration_wrong_postpatch_tree",
        "borrowed_allowance", "borrowed_closure", "borrowed_owner",
        "preparation_proposal_omitted", "preparation_proposal_duplicate", "preparation_proposal_wrong_identity")]
    rows += [("performance", name, "PERFORMANCE") for name in ("allocation_perturbation", "style_attribution", "smoke_digest",
        "noop_workload", "debug_profile", "parallel_profile", "strict_disabled", "wrong_denominator", "wrong_counter_owner")]
    rows += [("native", name, "SOURCE") for name in ("native_assertion_removed", "candidate_mapping_owner", "wrong_mapping", "candidate_geometry",
        "missing_mapping", "changed_native_predicate", "reuse_fixed_row", "candidate_retargets")]
    return rows


def self_test():
    count = 0
    for family, mutation, category in cases():
        fixture = ExtendedFixture(family, mutation)
        try:
            fixture.launch({"schema": "tc-proof-runner-observe/v1", "nonce": fixture.nonce,
                            "operation": fixture.operation, "source_commit": fixture.source_commit, "tree": fixture.source_tree})
            try:
                fixture.verify_observed()
            except ValueError:
                require(category is not None, "valid extension rejected")
            else:
                require(category is None, "extension mutant accepted")
            count += 1
        finally:
            fixture.cleanup()
    return {"extended_observer_cases": count, "accounting_stage_sequence": stage_sequence()}


def stage_sequence(runner=None):
    ledger = []
    baseline = None
    for mutation, category in [("valid", None), ("mixed_closed", None), ("all_green", None),
                               ("closed_regression", "TEST_ACCOUNTING"), ("reopen_allowance", "TEST_ACCOUNTING")]:
        fixture = ExtendedFixture("accounting", mutation)
        try:
            if baseline is None:
                baseline = (fixture.source_commit, (fixture.public / "oracle.bundle").read_bytes())
            else:
                # The shared historical source identity remains an actual Git
                # object across disposable candidate revisions, not a bare-name
                # alias or a fabricated hash chosen by a candidate report.
                reference = fixture.private / "stage-reference.bundle"
                reference.write_bytes(baseline[1])
                B.git(fixture.source, "fetch", str(reference), "HEAD")
                fixture.source_commit = baseline[0]
                fixture.context["oracle_commit"] = baseline[0]
                fixture.context["oracle_tree"] = B.git(fixture.source, "rev-parse", baseline[0] + "^{tree}")
                (fixture.public / "oracle.bundle").write_bytes(baseline[1])
                fixture.context["bundle_sha256"] = sha(baseline[1])
                for identity in fixture.contract["required"] + fixture.contract["closed"]:
                    identity["source_commit"] = baseline[0]
                for row in fixture.contract["future"] + fixture.future_expected:
                    row["identity"]["source_commit"] = baseline[0]
            fixture.contract["accepted_stage_receipts"] = copy.deepcopy(ledger)
            save(fixture.context_path, fixture.context)
            fixture.context_hash = sha(fixture.context_path.read_bytes())
            fixture.before = fixture.snapshot()
            if runner is not None:
                fixture.validate(fixture.run(runner), category)
            else:
                fixture.launch({"schema": "tc-proof-runner-observe/v1", "nonce": fixture.nonce,
                    "operation": fixture.operation, "source_commit": fixture.source_commit, "tree": fixture.source_tree})
                try:
                    fixture.verify_observed()
                except ValueError:
                    require(category is not None, "valid accounting stage rejected")
                else:
                    require(category is None, "closed stage regression accepted")
            if category is None:
                product = fixture.verify_observed()
                # Only the independent driver publishes this synthetic receipt,
                # after checking actual execution. Submitted proposals cannot.
                ledger.append({"schema": "tc-proof-accounting-stage/v1", "run_id": fixture.run_id,
                    "tree": fixture.source_tree, "observation_sha256": sha(canonical(fixture.events)),
                    "test_source_sha256": fixture.contract["original"]["source_sha256"],
                    "unresolved": product["unresolved"], "closed": [copy.deepcopy(row["identity"]) for row in fixture.future_expected
                        if row["identity"] not in product["unresolved"]]})
        finally:
            fixture.cleanup()
    require(len(ledger) == 3 and len(ledger[0]["unresolved"]) == 2 and len(ledger[1]["unresolved"]) == 1 and ledger[2]["unresolved"] == [],
            "actual mixed-target stage closure sequence")
    require(ledger[1]["closed"][0]["package"] == "tiny-a" and ledger[1]["unresolved"][0]["package"] == "tiny-b", "same-name identities merged")
    return 5


def run(runner, group=None):
    count = 0
    for family, mutation, category in cases():
        owner = {"native": "070", "accounting": "071", "performance": "072"}[family]
        if group and group != owner:
            continue
        for name, failure in [(mutation, category)] + ([("valid", None)] if category else []):
            fixture = ExtendedFixture(family, name)
            try:
                fixture.validate(fixture.run(runner), failure)
                count += 1
            finally:
                fixture.cleanup()
    if group in (None, "071"):
        count += stage_sequence(runner)
    return count


if __name__ == "__main__":
    print(json.dumps(self_test(), sort_keys=True))
