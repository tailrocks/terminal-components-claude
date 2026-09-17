#!/usr/bin/env python3
"""Read-only cross-artifact validation; taskfmt remains the task-schema authority."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import re
import runpy
import shlex
import sys
import tomllib
from collections import Counter
from pathlib import Path


TC_PROOF_OPERATIONS = frozenset({
    "preflight", "required", "oracle", "capture", "compare",
    "account-tests", "architecture", "close",
})
TC_PROOF_ORACLE_NAMESPACES = frozenset({"showcase", "holla", "jackin", "tablepro", "components"})
TC_PROOF_CAPTURE_LANES = frozenset({"direct", "pty"})
VERIFY_CHECK_PHASES = frozenset({"precondition", "focused", "regression", "lint", "gate"})
VERIFY_CHECK_REQUIRED = frozenset({"id", "phase", "expected", "requirements", "acceptance"})
CHECK_ID_PATTERN = re.compile(r"^CHK-\d{3}$")
TC_PROOF_BINARY = "tools/refactor-proof/bin/tc-proof"
TC_PROOF_HOST_BINARY = "tools/refactor-proof/bin/tc-proof-host"
CONTEXT_ROOT = ".campaign/evidence/contexts"
TRUSTED_ROOT = "refactoring-tasks/terminal-components/completion"
LEGACY_NAMESPACE = re.compile(r"(?<![A-Za-z0-9_.-])/(?:task|work|proof|run)(?:/|$)")
FORBIDDEN_RUNTIME = re.compile(
    r"\b(?:docker|podman|mount|firmlink)\b|"
    r"\btaskfmt\s+(?:init|status|host|runtime|run|lifecycle)\b"
)


class Audit:
    def __init__(self, root: Path):
        self.root = root
        self.docs = root / "docs/refactoring-plan"
        self.errors: list[str] = []
        self.counts: dict[str, int] = {}

    def require(self, condition: bool, message: str) -> None:
        if not condition:
            self.errors.append(message)

    def host_local_command(self, task_id: str, check_id: str, check: dict, package: Path) -> list[str]:
        """Validate the repository-relative command contract used by taskfmt verify."""
        has_argv = "argv" in check
        has_shell = "shell" in check
        self.require(has_argv != has_shell, f"Command must use exactly one of argv/shell: {task_id}:{check_id}")
        if has_argv:
            command = check.get("argv", [])
            self.require(isinstance(command, list) and bool(command), f"Empty command: {task_id}:{check_id}")
            if not isinstance(command, list):
                return []
            tokens = command
        else:
            shell = check.get("shell", "")
            self.require(isinstance(shell, str) and bool(shell.strip()), f"Empty shell command: {task_id}:{check_id}")
            if not isinstance(shell, str):
                return []
            try:
                tokens = shlex.split(shell)
            except ValueError as error:
                self.errors.append(f"Invalid shell command {task_id}:{check_id}: {error}")
                return []
            self.require(bool(tokens), f"Empty shell command: {task_id}:{check_id}")

        for token in tokens:
            if not isinstance(token, str):
                continue
            self.require(not token.startswith("/"), f"Absolute command path: {task_id}:{check_id}:{token}")
            self.require(not LEGACY_NAMESPACE.search(token), f"Legacy namespace in command: {task_id}:{check_id}:{token}")
            self.require(not FORBIDDEN_RUNTIME.search(token), f"Forbidden runtime command: {task_id}:{check_id}:{token}")

        trusted_prefix = f"{TRUSTED_ROOT}/{task_id.removeprefix('TASK-')}/trusted/"
        for token in tokens:
            if not isinstance(token, str):
                continue
            if token.startswith(TRUSTED_ROOT + "/"):
                self.require(token.startswith(trusted_prefix), f"Cross-task trusted input: {task_id}:{check_id}:{token}")
                self.require((self.root / token).is_file(), f"Missing trusted input: {task_id}:{check_id}:{token}")
            elif token in {TC_PROOF_BINARY, TC_PROOF_HOST_BINARY}:
                self.require((self.root / token).is_file(), f"Missing proof executable: {task_id}:{check_id}:{token}")

        expected_context = f"{CONTEXT_ROOT}/{check_id}.json"
        if has_argv and isinstance(check.get("argv"), list):
            argv = check["argv"]
            if argv and argv[0] == TC_PROOF_BINARY:
                self.require("--context" in argv, f"Missing operation context: {task_id}:{check_id}")
                if "--context" in argv:
                    position = argv.index("--context") + 1
                    self.require(position < len(argv) and argv[position] == expected_context,
                                 f"Per-check context mismatch: {task_id}:{check_id}")
        return tokens

    def table(self, name: str, key: str | None = None) -> list[dict[str, str]]:
        path = self.docs / name
        if not path.is_file():
            self.errors.append(f"Missing table: {name}")
            return []
        with path.open(newline="", encoding="utf-8") as stream:
            reader = csv.DictReader(stream, delimiter="\t")
            rows = list(reader)
            headers = reader.fieldnames or []
        self.require(bool(headers) and len(headers) == len(set(headers)), f"Invalid headers: {name}")
        for line, row in enumerate(rows, 2):
            self.require(None not in row and all(v is not None for v in row.values()), f"Invalid TSV shape: {name}:{line}")
            self.require(any(v and v.strip() for v in row.values() if isinstance(v, str)), f"Blank record: {name}:{line}")
        if key:
            ids = [row.get(key, "") for row in rows]
            self.require(all(ids), f"Empty {key}: {name}")
            self.require(len(ids) == len(set(ids)), f"Duplicate {key}: {name}")
        self.counts[name] = len(rows)
        return rows

    def inventory(self) -> dict[tuple[str, str], dict[str, str]]:
        sources: dict[tuple[str, str], dict[str, str]] = {}
        canonical = self.table("historical-obligations-canonical.tsv", "id")
        for row in canonical:
            sources["HIST", row["id"]] = row
        original: set[str] = set()
        canonical_by_id = {row["id"]: row for row in canonical}
        for path in sorted(self.docs.glob("*obligations.tsv")):
            if path.name == "historical-obligations-canonical.tsv":
                continue
            for line, row in enumerate(self.table(path.name, "id"), 2):
                source_id = row.get("id", "")
                self.require(source_id not in original, f"Colliding historical ID: {source_id}")
                original.add(source_id)
                joined = canonical_by_id.get(source_id)
                if joined is not None:
                    expected_source = f"docs/refactoring-plan/{path.name}:{line}"
                    self.require(joined.get("source_ledger_row") == expected_source,
                                 f"Canonical historical source-row drift: {source_id}")
                    # These two source ledgers already use the canonical field
                    # schema. Their projection must not reinterpret a clause.
                    # Heterogeneous historical ledgers retain their explicitly
                    # adjudicated mappings, rather than an invented coercion.
                    if path.name in {"historical-obligations.tsv", "history-middle-obligations.tsv"}:
                        for field, value in row.items():
                            self.require(joined.get(field) == value,
                                         f"Canonical historical field drift: {source_id}:{field}")
        canonical_ids = {row["id"] for row in canonical}
        self.require(original == canonical_ids, f"Historical canonical join differs: missing={sorted(original-canonical_ids)}, extra={sorted(canonical_ids-original)}")
        for namespace, filename, key in (
            ("ARCH", "architecture-matrix.tsv", "id"),
            ("COMP", "component-parity.tsv", "family"),
            ("DEC", "decision-ledger.tsv", "id"),
        ):
            for row in self.table(filename, key):
                sources[namespace, row[key]] = row
        scenarios: dict[str, str] = {}
        for app, key in (("showcase", "scenario_id"), ("holla", "id"), ("jackin", "id"), ("tablepro", "scenario_id")):
            for row in self.table(f"{app}-scenarios.tsv", key):
                scenario_id = row[key]
                self.require(scenario_id not in scenarios, f"Duplicate cross-app scenario: {scenario_id}")
                scenarios[scenario_id] = app
                sources["APP", scenario_id] = row
        application = self.table("application-parity.tsv", "scenario_id")
        joined = {row["scenario_id"]: row["application"] for row in application}
        self.require(joined == scenarios, "Application parity join differs from four source inventories")
        for filename in ("app-flow-contributions.tsv", "app-flow-frame-contributions.tsv"):
            for line, row in enumerate(self.table(filename, "contribution_id"), 2):
                source = ("APP-FLOW", row["contribution_id"])
                self.require(source not in sources, f"Duplicate derived contribution: {source}")
                self.require(row["parent_scenario"] in scenarios, f"Unknown contribution parent: {source}")
                self.require(row.get("required_for_closure") == "true", f"Optional derived contribution: {source}")
                sources[source] = {**row, "_source_file": filename, "_source_line": str(line)}
        self.counts["source_obligations"] = len(sources)
        return sources

    def catalog(self) -> tuple[dict[str, dict], dict[str, list[str]]]:
        index = self.table("task-index.tsv", "task_id")
        indexed = {row["task_id"]: row for row in index}
        tasks: dict[str, dict] = {}
        dependencies: dict[str, list[str]] = {}
        catalog = self.root / "refactoring-tasks/terminal-components/completion"
        package_ids = {f"TASK-{p.name}" for p in catalog.glob("[0-9][0-9][0-9]") if p.is_dir()}
        self.require(package_ids == set(indexed), f"Catalog/index differ: missing={sorted(set(indexed)-package_ids)}, extra={sorted(package_ids-set(indexed))}")
        for task_id, row in indexed.items():
            package = catalog / task_id.removeprefix("TASK-")
            for filename in ("README.md", "AGENTS.md", "CAMPAIGN_AGENTS.md", "task.toml", "verify.toml"):
                self.require((package / filename).is_file(), f"Missing {task_id}/{filename}")
            try:
                metadata = tomllib.loads((package / "task.toml").read_text())
                verify = tomllib.loads((package / "verify.toml").read_text())
                readme = (package / "README.md").read_text()
            except (OSError, tomllib.TOMLDecodeError) as error:
                self.errors.append(f"Unreadable package {task_id}: {error}")
                continue
            self.require(metadata.get("status") == "pending", f"Non-pending execution task: {task_id}")
            prefix = "terminal-components/completion/"
            raw_deps = metadata.get("dependencies", [])
            self.require(all(isinstance(dep, str) and dep.startswith(prefix) for dep in raw_deps), f"Unqualified dependency: {task_id}")
            deps = ["TASK-" + dep.removeprefix(prefix) for dep in raw_deps]
            dependencies[task_id] = deps
            self.require(set(deps) == set(filter(None, row["dependencies"].split(";"))), f"Index dependency drift: {task_id}")
            self.require(len(deps) == len(set(deps)), f"Duplicate dependency: {task_id}")
            self.require(verify.get("task_id") == task_id, f"Verification ID mismatch: {task_id}")
            self.require(set(verify.get("writable_paths", [])) == set(filter(None, row["writable_paths"].split(";"))), f"Index scope drift: {task_id}")
            requirements = set(re.findall(r"\*\*(R-\d+) \(", readme))
            acceptance = set(re.findall(r"^### (AC-\d+)\b", readme, re.M))
            checks = {check["id"]: check for check in verify.get("checks", [])}
            self.require(bool(requirements) and bool(acceptance) and bool(checks), f"Empty contract graph: {task_id}")
            for check_id, check in checks.items():
                self.host_local_command(task_id, check_id, check, package)
                argv = check.get("argv", [])
                if argv and argv[0] == TC_PROOF_BINARY:
                    self.require("--context" in argv, f"Missing operation context: {task_id}:{check_id}")
                    if "--context" in argv:
                        position = argv.index("--context") + 1
                        expected_context = f"{CONTEXT_ROOT}/{check_id}.json"
                        self.require(position < len(argv) and argv[position] == expected_context, f"Per-check context mismatch: {task_id}:{check_id}")
                if any("runner-bootstrap-driver.py" in argument for argument in argv) and "--group" in argv:
                    position = argv.index("--group") + 1
                    self.require(position < len(argv) and argv[position] in {"070", "071", "072"}, f"Invalid runner qualification group: {task_id}:{check_id}")
            protocol = (package / "AGENTS.md").read_bytes()
            normalized = re.sub(rb"TASK-\d{3}", b"TASK-000", protocol)
            self.require(hashlib.sha256(normalized).hexdigest() == "86bbf024a51aa62f6bc739a0ed46d54faab76cf87ffe918c682dfbc453b964de", f"Canonical execution protocol drift: {task_id}")
            self.require("source-obligations.tsv" in readme, f"Historical payload not bound by README: {task_id}")
            read_before = readme.split("Read before editing:", 1)
            self.require(len(read_before) == 2, f"Missing Read before editing section: {task_id}")
            if len(read_before) == 2:
                self.require("CAMPAIGN_AGENTS.md" in read_before[1].split("##", 1)[0],
                             f"{task_id} README must list CAMPAIGN_AGENTS.md under Read before editing")
            tasks[task_id] = {"requirements": requirements, "acceptance": acceptance, "checks": checks, "package": package}
        for task_id, deps in dependencies.items():
            self.require(task_id not in deps, f"Self dependency: {task_id}")
            self.require(set(deps) <= set(indexed), f"Missing dependency target: {task_id}")
        return tasks, dependencies

    def catalog_argv_smoke(self) -> None:
        """Dry-run verify.toml commands without invoking configured checkers."""
        catalog = self.root / "refactoring-tasks/terminal-components/completion"
        tc_proof_checks = 0
        for verify_path in sorted(catalog.glob("*/verify.toml")):
            relative = verify_path.relative_to(self.root)
            try:
                data = tomllib.loads(verify_path.read_text())
            except (OSError, tomllib.TOMLDecodeError) as error:
                self.errors.append(f"Unreadable verify argv smoke: {relative}: {error}")
                continue
            task_id = data.get("task_id", "")
            self.require(data.get("schema") == "verify/v2", f"Unsupported verify schema: {relative}")
            self.require(bool(task_id), f"Missing verify task_id: {relative}")
            self.require(task_id == f"TASK-{verify_path.parent.name}", f"Verify task_id mismatch: {relative}")
            seen_ids: set[str] = set()
            for check in data.get("checks", []):
                check_id = check.get("id", "")
                self.require(bool(CHECK_ID_PATTERN.fullmatch(check_id)), f"Invalid check id: {relative}:{check_id}")
                self.require(check_id not in seen_ids, f"Duplicate check id: {relative}:{check_id}")
                seen_ids.add(check_id)
                self.require(set(check) >= VERIFY_CHECK_REQUIRED, f"Missing check fields: {relative}:{check_id}")
                self.host_local_command(task_id, check_id, check, verify_path.parent)
                phase = check.get("phase", "")
                self.require(phase in VERIFY_CHECK_PHASES, f"Unknown check phase: {relative}:{check_id}:{phase}")
                argv = check.get("argv", [])
                if "argv" in check:
                    self.require(isinstance(argv, list) and bool(argv), f"Empty argv: {relative}:{check_id}")
                self.require(all(isinstance(argument, str) and argument for argument in argv),
                             f"Blank argv segment: {relative}:{check_id}")
                expected = check.get("expected", {})
                self.require(isinstance(expected, dict) and "exit" in expected,
                             f"Missing expected exit: {relative}:{check_id}")
                for field in ("requirements", "acceptance"):
                    values = check.get(field, [])
                    self.require(isinstance(values, list) and bool(values), f"Empty {field}: {relative}:{check_id}")
                if not argv or argv[0] != TC_PROOF_BINARY:
                    continue
                tc_proof_checks += 1
                self.require(len(argv) >= 2, f"Missing tc-proof operation: {relative}:{check_id}")
                operation = argv[1]
                self.require(operation in TC_PROOF_OPERATIONS,
                             f"Unknown tc-proof operation: {relative}:{check_id}:{operation}")
                self.require("--context" in argv, f"Missing tc-proof context: {relative}:{check_id}")
                position = argv.index("--context") + 1
                self.require(position < len(argv), f"Dangling tc-proof --context: {relative}:{check_id}")
                expected_context = f"{CONTEXT_ROOT}/{check_id}.json"
                self.require(argv[position] == expected_context,
                             f"Context check id mismatch: {relative}:{check_id}")
                if operation == "oracle":
                    self.require("--namespace" in argv, f"Missing oracle namespace: {relative}:{check_id}")
                    namespace_position = argv.index("--namespace") + 1
                    self.require(namespace_position < len(argv)
                                 and argv[namespace_position] in TC_PROOF_ORACLE_NAMESPACES,
                                 f"Invalid oracle namespace: {relative}:{check_id}")
                elif operation == "capture":
                    self.require("--lane" in argv, f"Missing capture lane: {relative}:{check_id}")
                    lane_position = argv.index("--lane") + 1
                    self.require(lane_position < len(argv) and argv[lane_position] in TC_PROOF_CAPTURE_LANES,
                                 f"Invalid capture lane: {relative}:{check_id}")
                else:
                    self.require("--namespace" not in argv, f"Unexpected oracle namespace: {relative}:{check_id}")
                    self.require("--lane" not in argv, f"Unexpected capture lane: {relative}:{check_id}")
        self.counts["catalog_argv_smoke_tc_proof_checks"] = tc_proof_checks

    def historical_prose(self, sources: dict, tasks: dict) -> None:
        """Check redundant prose without making it the ownership inventory."""
        count = 0
        declared = self.table("history-prose-membership.tsv")
        expected_membership = {(row.get("task_id"), row.get("source_id")) for row in declared}
        self.require(len(expected_membership) == len(declared), "Duplicate HIST prose membership")
        self.require(all(task in tasks and ("HIST", source) in sources for task, source in expected_membership),
                     "Unknown HIST prose membership")
        observed_membership = set()
        for task_id, task in tasks.items():
            path = task["package"] / "trusted/obligations.md"
            if not path.is_file():
                self.errors.append(f"Missing trusted obligations: {task_id}")
                continue
            text = path.read_text()
            seen = set()
            for heading in re.finditer(r"(?m)^### HIST:([^\s]+)\s*$", text):
                source_id = heading.group(1)
                observed_membership.add((task_id, source_id))
                self.require(source_id not in seen, f"Duplicate historical prose: {task_id}:{source_id}")
                seen.add(source_id)
                row = sources.get(("HIST", source_id))
                if row is None:
                    self.errors.append(f"Unknown historical prose: {task_id}:{source_id}")
                    continue
                following = re.search(r"(?m)^#{1,3} ", text[heading.end():])
                end = heading.end() + following.start() if following else len(text)
                block = text[heading.end():end]
                expected = {
                    "Source": row["historical_revision"] + "; " + row["source_document"],
                    "Requirement": row["decision_requirement"],
                    "Disposition": row["authority_status"] + "; current " + row["current_main_status"],
                    "Remaining proof": row["remaining_work"], "Gates": row["tests_gates"],
                    "Origin": row["source_ledger_row"] + "; " + row["relationship"],
                }
                for label, value in expected.items():
                    matches = re.findall(r"(?m)^- " + re.escape(label) + r": ([^\n]*)$", block)
                    self.require(matches == [value], f"Historical prose field drift: {task_id}:{source_id}:{label}")
                count += 1
        self.require(observed_membership == expected_membership, "HIST prose membership differs")
        self.counts["historical_prose_sections"] = count

    def frozen_assets(self) -> None:
        # Import only the trusted sibling, never a helper from --root. Sharing
        # expansion prevents the writer and validator from accepting different
        # group/source/destination memberships when asset bytes happen to match.
        helper = runpy.run_path(str(Path(__file__).resolve().with_name("freeze-bootstrap-assets.py")))
        bindings = set(helper["expected_bindings"]())
        rows = self.table("bootstrap-assets.tsv", "destination")
        observed = {(row.get("group"), row.get("source"), row.get("destination")) for row in rows}
        self.require(observed == bindings,
                     f"Bootstrap asset membership differs: missing={sorted(bindings - observed)}, extra={sorted(observed - bindings, key=str)}")
        self.require(len(observed) == len(rows), "Duplicate bootstrap asset binding")
        for row in rows:
            self.require(set(row) == {"group", "source", "destination", "sha256"}, "Bootstrap asset schema differs")
            self.require(row.get("group") in helper["GROUPS"], "Unknown bootstrap asset group")
            expected = row.get("sha256", "")
            self.require(bool(re.fullmatch(r"[0-9a-f]{64}", expected)), "Invalid bootstrap asset digest")
            for column in ("source", "destination"):
                relative = Path(row.get(column, ""))
                if relative.is_absolute() or ".." in relative.parts:
                    self.errors.append(f"Escaping bootstrap asset: {relative}")
                    continue
                path = self.root / relative
                self.require(not any((self.root / Path(*relative.parts[:n])).is_symlink() for n in range(1, len(relative.parts) + 1)), f"Symlinked bootstrap asset: {relative}")
                if not path.is_file():
                    self.errors.append(f"Missing frozen bootstrap asset: {relative}")
                    continue
                self.require(hashlib.sha256(path.read_bytes()).hexdigest() == expected, f"Frozen bootstrap asset differs: {relative}")

    def graph(self, dependencies: dict[str, list[str]]) -> dict:
        active: set[str] = set()
        depth: dict[str, int] = {}
        predecessors: dict[str, list[str]] = {}

        def visit(task: str) -> int:
            if task in active:
                raise ValueError(f"Dependency cycle at {task}")
            if task in depth:
                return depth[task]
            if task not in dependencies:
                raise ValueError(f"Dependency package missing: {task}")
            active.add(task)
            values = {dep: visit(dep) for dep in dependencies[task]}
            maximum = max(values.values(), default=0)
            depth[task] = maximum + 1
            predecessors[task] = sorted(dep for dep, value in values.items() if value == maximum)
            active.remove(task)
            return depth[task]

        try:
            for task in sorted(dependencies):
                visit(task)
        except ValueError as error:
            self.errors.append(str(error))
            return {}
        maximum = max(depth.values(), default=0)
        return {"maximum_dependency_depth": maximum, "deepest_tasks": sorted(task for task, value in depth.items() if value == maximum), "depth": depth, "longest_path_predecessors": predecessors}

    def accounting_context_bindings(self) -> None:
        bindings_path = (
            self.root
            / "refactoring-tasks/terminal-components/completion/071/trusted/check-context-templates/accounting-mode-bindings.tsv"
        )
        self.require(bindings_path.is_file(), "Missing TASK-071 accounting-mode-bindings.tsv")
        if not bindings_path.is_file():
            return
        with bindings_path.open(newline="", encoding="utf-8") as stream:
            expected = {(row["task_id"], row["check_id"]): row for row in csv.DictReader(stream, delimiter="\t")}
        completion = self.root / "refactoring-tasks/terminal-components/completion"
        found: set[tuple[str, str]] = set()
        for verify in sorted(completion.glob("*/verify.toml")):
            data = tomllib.loads(verify.read_text())
            task_id = data.get("task_id", "")
            if not task_id:
                continue
            num = task_id.split("-")[1]
            for check in data.get("checks", []):
                if "account-tests" not in check.get("argv", []):
                    continue
                check_id = check["id"]
                template = completion / num / "trusted/check-context-templates" / f"{check_id}.json"
                self.require(template.is_file(), f"Missing accounting context template: {task_id}:{check_id}")
                if not template.is_file():
                    continue
                body = json.loads(template.read_text())
                mode = body.get("qualification", {}).get("mode")
                key = (task_id, check_id)
                row = expected.get(key)
                self.require(row is not None, f"Unbound accounting check: {task_id}:{check_id}")
                if row is None:
                    continue
                self.require(mode == row["qualification_mode"], f"Accounting mode mismatch: {task_id}:{check_id}")
                if task_id in {f"TASK-{n:03d}" for n in range(2, 9)}:
                    self.require(mode == "preparation", f"Preparation task forbidden production mode: {task_id}:{check_id}")
                else:
                    self.require(mode == "production", f"Production task must use production mode: {task_id}:{check_id}")
                found.add(key)
        missing = set(expected) - found
        extra = found - set(expected)
        self.require(not missing, f"Missing accounting templates for bindings: {sorted(missing)[:5]}")
        self.require(not extra, f"Unexpected accounting templates: {sorted(extra)[:5]}")

    def runner_index_receipt_bindings(self) -> None:
        specs = {
            "TASK-071": {
                "binding": "refactoring-tasks/terminal-components/completion/071/trusted/runner-index-receipt-binding.md",
                "check_id": "CHK-008",
                "group": "070",
            },
            "TASK-072": {
                "binding": "refactoring-tasks/terminal-components/completion/072/trusted/runner-index-receipt-binding.md",
                "check_id": "CHK-009",
                "group": "070",
            },
        }
        for task_id, spec in specs.items():
            binding = self.root / spec["binding"]
            self.require(binding.is_file(), f"Missing runner index binding: {spec['binding']}")
            num = task_id.split("-")[1]
            readme = self.root / f"refactoring-tasks/terminal-components/completion/{num}/README.md"
            verify = self.root / f"refactoring-tasks/terminal-components/completion/{num}/verify.toml"
            self.require(readme.is_file() and verify.is_file(), f"Missing TASK package for index binding: {task_id}")
            if readme.is_file():
                text = readme.read_text()
                self.require("runner-index-receipt-binding.md" in text, f"{task_id} README must reference runner-index-receipt-binding.md")
                self.require("D-006" in text, f"{task_id} README must declare D-006 index receipt binding")
            if verify.is_file():
                data = tomllib.loads(verify.read_text())
                checks = data.get("checks", [])
                match = next(
                    (
                        check
                        for check in checks
                        if check.get("id") == spec["check_id"]
                        and spec["group"] in check.get("argv", [])
                        and "runner-bootstrap-driver.py" in " ".join(check.get("argv", []))
                    ),
                    None,
                )
                self.require(match is not None, f"{task_id} verify.toml must wire {spec['check_id']} to runner --group {spec['group']}")

    def task069_host_context(self) -> None:
        base = self.root / "refactoring-tasks/terminal-components/completion/069/trusted"
        required = [
            base / "coordinator-review-contract.md",
            base / "host-context/host-context-spec.md",
            base / "host-context/CHK-001.template.json",
            base / "host-context/CHK-005.template.json",
            base / "host-context/CHK-007.template.json",
        ]
        for path in required:
            self.require(path.is_file(), f"Missing TASK-069 host artifact: {path.relative_to(self.root)}")
        obligations = (base / "obligations.md").read_text()
        self.require("coordinator-review-contract.md" in obligations, "TASK-069 obligations must reference coordinator-review-contract.md")
        self.require("host-context/CHK-005.template.json" in obligations, "TASK-069 obligations must bind fidelity visual template")
        close = json.loads((base / "host-context/CHK-007.template.json").read_text())
        self.require("human_review" in close.get("forbid_fields", []), "CHK-007 template must forbid human_review fields")

    def coordinator_branch_bindings(self) -> None:
        projection = self.table("branch-diff-components-b-coordinator-projection.tsv")
        disposition_doc = self.docs / "branch-diff-components-b-test-disposition-bindings.tsv"

        row031 = next((row for row in projection if "TASK-031 host projection" in row.get("integration_item", "")), None)
        self.require(row031 is not None, "Missing TASK-031 coordinator projection row")
        if row031 is None:
            return
        claimed = {witness.strip() for witness in row031.get("projection_binding", "").split(";") if witness.strip().startswith("W-")}
        claimed.discard("existing W-031-10/11")

        proj_path = self.root / "refactoring-tasks/terminal-components/completion/031/trusted/branch-host-projection.tsv"
        template_path = (
            self.root
            / "refactoring-tasks/terminal-components/completion/031/trusted/check-context-templates/CHK-006.template.json"
        )
        spec_path = self.root / "refactoring-tasks/terminal-components/completion/031/trusted/host-context-spec.md"
        self.require(proj_path.is_file(), "Missing TASK-031 branch-host-projection.tsv")
        self.require(template_path.is_file(), "Missing TASK-031 CHK-006.template.json")
        self.require(spec_path.is_file(), "Missing TASK-031 host-context-spec.md")
        if template_path.is_file():
            body = json.loads(template_path.read_text())
            self.require(body.get("operation") == "architecture", "TASK-031 CHK-006 template must bind architecture")
            template_binding = body.get("branch_host_projection", {})
            self.require(Path(str(template_binding.get("index", ""))).name == proj_path.name,
                         "TASK-031 CHK-006 template index mismatch")
            self.require(set(template_binding.get("required_roles", [])) == {"branch_host_projection", "native_conformance"},
                         "TASK-031 CHK-006 template role mismatch")
            self.require("W-042-JUMP-SUBMIT" in template_binding.get("forbidden_witness_ids", []),
                         "TASK-031 CHK-006 template must forbid W-042-JUMP-SUBMIT")
        if proj_path.is_file():
            with proj_path.open(newline="", encoding="utf-8") as stream:
                rows = list(csv.DictReader(stream, delimiter="\t"))
                indexed = {row["witness_id"] for row in rows}
            self.require(claimed <= indexed, f"TASK-031 projection missing witnesses: {sorted(claimed - indexed)}")
            self.require("W-042-JUMP-SUBMIT" not in indexed, "W-042-JUMP-SUBMIT must not appear in TASK-031 branch projection")
            for row in rows:
                witness_id = row.get("witness_id", "")
                producer_path = row.get("producer_trusted_path", "")
                if "#" in producer_path:
                    relative, anchor = producer_path.split("#", 1)
                else:
                    relative, anchor = producer_path, None
                path = self.root / relative
                self.require(path.is_file(), f"TASK-031 projection missing producer file: {witness_id}:{relative}")
                if path.is_file() and anchor:
                    self.require(anchor in path.read_text(),
                                 f"TASK-031 projection missing producer anchor: {witness_id}:{anchor}")

        row008 = next((row for row in projection if "TASK-008 assertion dispositions" in row.get("integration_item", "")), None)
        self.require(row008 is not None, "Missing TASK-008 coordinator disposition row")
        disp_claimed = {"W-021-07", "W-025-07", "W-025-08", "W-026-05"}

        trusted_disp = self.root / "refactoring-tasks/terminal-components/completion/008/trusted/branch-test-disposition-bindings.tsv"
        obligations_path = self.root / "refactoring-tasks/terminal-components/completion/008/trusted/obligations.md"
        external_path = self.root / "refactoring-tasks/terminal-components/completion/008/trusted/external-test-source-scope.md"
        self.require(trusted_disp.is_file(), "Missing TASK-008 branch-test-disposition-bindings.tsv")
        self.require(external_path.is_file(), "Missing TASK-008 external-test-source-scope.md")
        if trusted_disp.is_file():
            with trusted_disp.open(newline="", encoding="utf-8") as stream:
                disp_ids = {row["witness_id"] for row in csv.DictReader(stream, delimiter="\t")}
            self.require(disp_claimed <= disp_ids, "TASK-008 disposition TSV missing branch witnesses")
        if obligations_path.is_file():
            obligations = obligations_path.read_text()
            for witness_id in disp_claimed:
                self.require(witness_id in obligations, f"TASK-008 obligations.md missing {witness_id}")
        if external_path.is_file() and disposition_doc.is_file():
            external = external_path.read_text()
            with disposition_doc.open(newline="", encoding="utf-8") as stream:
                for row in csv.DictReader(stream, delimiter="\t"):
                    for token in row.get("conflicting_scope", "").split(";"):
                        token = token.strip()
                        if token.startswith("crates/tui/tests/") and "*" not in token:
                            self.require(token in external, f"External test path not scoped: {token}")

    def traceability(self, sources: dict, tasks: dict) -> None:
        rows = self.table("traceability.tsv")
        required_columns = {"source_namespace", "source_id", "task_id", "requirement_id", "acceptance_id", "check_id", "role", "disposition"}
        roles = {"primary", "preservation", "baseline", "integration", "forbidden", "disposition", "closure", "contribution", "frame-contribution"}
        anchors = {"primary", "preservation", "forbidden", "disposition"}
        seen: set[tuple] = set()
        covered: dict[tuple[str, str], set[str]] = {}
        task_coverage: Counter = Counter()
        for line, row in enumerate(rows, 2):
            self.require(set(row) == required_columns, f"Traceability schema drift at line {line}")
            if not required_columns <= set(row):
                continue
            identity = tuple(row[key] for key in sorted(required_columns))
            self.require(identity not in seen, f"Duplicate traceability row: {line}")
            seen.add(identity)
            source = row["source_namespace"], row["source_id"]
            task_id = row["task_id"]
            self.require(source in sources, f"Unknown traceability source: {source}")
            if source[0] == "HIST" and source in sources:
                authority = sources[source]
                suffix = (" Source disposition: " + authority["authority_status"]
                          + ". Exact source: " + authority["source_ledger_row"] + ".")
                self.require(row["disposition"].endswith(suffix)
                             and row["disposition"].count(" Source disposition:") == 1,
                             f"Historical authority/provenance drift: {line}:{source[1]}")
            self.require(task_id in tasks, f"Unknown traceability task: {task_id}")
            self.require(row["role"] in roles and bool(row["disposition"].strip()), f"Invalid traceability role/disposition: {line}")
            covered.setdefault(source, set()).add(row["role"])
            task_coverage[task_id] += 1
            if task_id not in tasks:
                continue
            task = tasks[task_id]
            requirement, acceptance, check_id = row["requirement_id"], row["acceptance_id"], row["check_id"]
            self.require(requirement in task["requirements"], f"Unknown requirement {task_id}:{requirement}")
            self.require(acceptance in task["acceptance"], f"Unknown acceptance {task_id}:{acceptance}")
            self.require(check_id in task["checks"], f"Unknown check {task_id}:{check_id}")
            check = task["checks"].get(check_id, {})
            self.require(requirement in check.get("requirements", []) and acceptance in check.get("acceptance", []), f"Traceability check mapping mismatch: {task_id}:{check_id}:{requirement}:{acceptance}")
        for source in sources:
            self.require(bool(covered.get(source, set()) & anchors), f"No owning obligation disposition: {source}")
            if source[0] in {"APP", "APP-FLOW"}:
                self.require("baseline" in covered.get(source, set()), f"No oracle producer: {source}")
            if source[0] == "APP-FLOW":
                contribution = sources[source]
                own_edges = [row for row in rows if (row["source_namespace"], row["source_id"]) == source]
                primary_owners = {row["task_id"] for row in own_edges if row["role"] == "primary"}
                self.require(primary_owners == {contribution["task_owner"]}, f"Derived contribution owner differs: {source}")
                baseline = "TASK-004" if contribution["parent_scenario"].startswith("JA-") else "TASK-005"
                self.require({row["task_id"] for row in own_edges if row["role"] == "baseline"} == {baseline}, f"Derived contribution baseline differs: {source}")
        for task_id in tasks:
            self.require(task_coverage[task_id] > 0, f"Orphan task: {task_id}")
        contribution_ids = set()
        for filename, default_kind in (("shell-contributions.tsv", "semantic"), ("shell-frame-contributions.tsv", "frame"), ("holla-stage-contributions.tsv", None)):
            for contribution in self.table(filename, "contribution_id"):
                identity = contribution["contribution_id"]
                self.require(identity not in contribution_ids, f"Duplicate named contribution: {identity}")
                contribution_ids.add(identity)
                self.require(("APP", contribution["parent_scenario"]) in sources, f"Unknown named contribution parent: {identity}")
                kind = default_kind or contribution["proof_kind"]
                if kind == "state":
                    kind = "semantic"
                self.require(kind in {"frame", "semantic"}, f"Unknown named contribution proof kind: {identity}")
                role = "frame-contribution" if kind == "frame" else "contribution"
                check_id = "CHK-004" if kind == "frame" else "CHK-006"
                matches = [row for row in rows if row["source_namespace"] == "APP" and row["source_id"] == contribution["parent_scenario"] and row["task_id"] == contribution["task_owner"] and row["role"] == role and row["check_id"] == check_id]
                self.require(bool(matches), f"Unbound named contribution: {identity}")
                if filename == "holla-stage-contributions.tsv":
                    self.require(any(identity in row["disposition"] for row in matches), f"Missing exact named contribution edge: {identity}")
                    self.require(contribution["minimum_checkpoint_count"].isdigit() and int(contribution["minimum_checkpoint_count"]) > 0, f"Empty contribution allowed: {identity}")
                else:
                    self.require(contribution["required_for_closure"] == "true", f"Optional shell contribution: {identity}")
        self.counts["additional_named_contributions"] = len(contribution_ids)
        embedded_columns = ("source_namespace", "source_id", "task_id", "requirement_id", "acceptance_id", "check_id", "role", "disposition")
        protected_edges = {
            tuple(row[column] for column in embedded_columns)
            for row in rows if row.get("source_namespace") in {"HIST", "APP-FLOW"}
        }
        embedded_edges = set()
        for task_id, task in tasks.items():
            payload = task["package"] / "trusted/source-obligations.tsv"
            if not payload.is_file():
                self.errors.append(f"Missing protected historical payload: {task_id}")
                continue
            with payload.open(newline="", encoding="utf-8") as stream:
                payload_rows = list(csv.DictReader(stream, delimiter="\t"))
            for row in payload_rows:
                source_id = row.get("source_id", "")
                namespace = row.get("source_namespace", "")
                canonical = sources.get((namespace, source_id))
                if canonical is None:
                    self.errors.append(f"Unknown protected source: {task_id}:{namespace}:{source_id}")
                    continue
                if namespace == "HIST":
                    self.require(all(row.get(key) == value for key, value in canonical.items()), f"Protected historical clause differs: {task_id}:{source_id}")
                elif namespace == "APP-FLOW":
                    exact = canonical.get("exact_assertions", canonical.get("exact_checkpoint_selection"))
                    self.require(row.get("decision_requirement") == exact, f"Protected contribution clause differs: {task_id}:{source_id}")
                    self.require(row.get("files_components") == canonical["parent_scenario"], f"Protected contribution parent differs: {task_id}:{source_id}")
                    source_path = "docs/refactoring-plan/" + canonical["_source_file"]
                    self.require(row.get("source_document") == source_path and row.get("source_ledger_row") == source_path + ":" + canonical["_source_line"], f"Protected contribution provenance differs: {task_id}:{source_id}")
                    self.require(row.get("historical_revision") == "02f5294bfdbf38004cc49130d0aff1d01f31434c", f"Protected contribution oracle differs: {task_id}:{source_id}")
                else:
                    self.errors.append(f"Unsupported embedded namespace: {task_id}:{namespace}")
                edge = tuple(row.get(column, "") if column != "task_id" else task_id for column in embedded_columns)
                self.require(edge not in embedded_edges, f"Duplicate protected edge: {task_id}:{source_id}")
                embedded_edges.add(edge)
        self.require(embedded_edges == protected_edges, f"Protected edge join differs: missing={len(protected_edges-embedded_edges)}, extra={len(embedded_edges-protected_edges)}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[3])
    parser.add_argument("--inventory-only", action="store_true", help="Validate factual inventories; does not certify task readiness")
    parser.add_argument("--summary", action="store_true", help="Print bounded diagnostics without changing the verdict")
    arguments = parser.parse_args()
    audit = Audit(arguments.root.resolve())
    sources = audit.inventory()
    graph = {}
    if not arguments.inventory_only:
        tasks, dependencies = audit.catalog()
        audit.catalog_argv_smoke()
        audit.historical_prose(sources, tasks)
        audit.frozen_assets()
        graph = audit.graph(dependencies)
        audit.coordinator_branch_bindings()
        root_campaign = audit.root / "refactoring-tasks/terminal-components/CAMPAIGN_AGENTS.md"
        audit.require(root_campaign.is_file(), "Missing refactoring-tasks/terminal-components/CAMPAIGN_AGENTS.md")
        audit.accounting_context_bindings()
        audit.runner_index_receipt_bindings()
        audit.task069_host_context()
        audit.traceability(sources, tasks)
    report = {"schema": "tc-planning-audit/v1", "mode": "inventory" if arguments.inventory_only else "artifact-integrity", "passed": not audit.errors, "counts": audit.counts, "graph": graph, "errors": audit.errors}
    if arguments.summary:
        report["error_count"] = len(audit.errors)
        report["errors"] = audit.errors[:25]
        report["graph"] = {key: value for key, value in graph.items() if key in {"maximum_dependency_depth", "deepest_tasks"}}
    print(json.dumps(report, indent=2, sort_keys=True))
    return int(bool(audit.errors))


if __name__ == "__main__":
    sys.exit(main())
