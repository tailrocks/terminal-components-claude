"""Immutable multi-check sequence qualification; child schemas remain explicit."""
import copy
import importlib.util
from pathlib import Path
import secrets

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("index_base", HERE / "runner-bootstrap-driver.py")
B = importlib.util.module_from_spec(spec)
spec.loader.exec_module(B)
require, sha, canonical, save = B.require, B.sha, B.canonical, B.save


class IndexFixture(B.Fixture):
    def __init__(self, mutation):
        super().__init__("preflight", "valid")
        self.index_mutation = mutation
        self.task_id = "terminal-components/completion/970"
        self.contexts = self.public / "contexts"
        self.contexts.mkdir()
        self.children, self.members = {}, []
        for check_id, operation, lane in [("CHK-001", "preflight", None), ("CHK-002", "capture", "direct"),
                                          ("CHK-003", "capture", "pty"), ("CHK-004", "close", None)]:
            child = copy.deepcopy(self.context)
            child.update(schema="tc-proof-runner-context/v2", check_id=check_id, task_id=self.task_id, operation=operation)
            child["lane"] = lane or "direct"
            path = self.contexts / (check_id + ".json")
            save(path, child)
            member = {"check_id": check_id, "context_path": str(path), "context_sha256": sha(path.read_bytes()),
                      "schema": child["schema"], "operation": operation, "lane": lane, "namespace": None,
                      "required_ids": [item for item in self.expected_members if operation == "capture" and item.split("/")[1] == lane],
                      "output_id": check_id + ".result.json"}
            self.children[check_id] = child
            self.members.append(member)
        self.expected_index = {"schema": "tc-proof-context-index/v1", "run_id": self.run_id, "task_id": self.task_id,
            "tree": self.source_tree, "trust_sha256": sha(canonical({"oracle_commit": self.source_commit})), "members": copy.deepcopy(self.members)}
        self.index = copy.deepcopy(self.expected_index)
        if mutation == "missing_child":
            Path(self.members[1]["context_path"]).unlink()
        elif mutation == "extra_child":
            save(self.contexts / "CHK-999.json", self.children["CHK-001"])
        elif mutation == "swap_direct_pty":
            left, right = (Path(self.members[index]["context_path"]) for index in (1, 2))
            a, b = left.read_bytes(), right.read_bytes()
            left.write_bytes(b)
            right.write_bytes(a)
        elif mutation in {"other_run", "other_tree", "other_task"}:
            child = self.children["CHK-002"]
            child[{"other_run": "run_id", "other_tree": "tree", "other_task": "task_id"}[mutation]] = secrets.token_hex(24)
            save(Path(self.members[1]["context_path"]), child)
            self.index["members"][1]["context_sha256"] = sha(Path(self.members[1]["context_path"]).read_bytes())
        elif mutation in {"child_wrong_schema", "child_extra_field"}:
            child = self.children["CHK-002"]
            if mutation == "child_wrong_schema":
                child["schema"] = "tc-proof-invalid/v999"
            else:
                child["unrecognized_authority"] = True
            save(Path(self.members[1]["context_path"]), child)
            self.index["members"][1]["context_sha256"] = sha(Path(self.members[1]["context_path"]).read_bytes())
        elif mutation == "output_alias":
            self.index["members"][2]["output_id"] = self.index["members"][1]["output_id"]
        elif mutation == "replay_second_id":
            self.index["members"][2].update(context_path=self.members[1]["context_path"],
                context_sha256=self.members[1]["context_sha256"], lane="direct")
        self.index_path = self.public / "context-index.json"
        save(self.index_path, self.index)
        self.index_hash = sha(self.index_path.read_bytes())
        self.completed = []
        self.before = self.snapshot()

    def validate_index(self):
        require(sha(self.index_path.read_bytes()) == self.index_hash, "index changed between operations")
        index = B.load_bytes(self.index_path.read_bytes())
        require(set(index) == set(self.expected_index) and index["schema"] == "tc-proof-context-index/v1", "index schema")
        require(all(index[key] == self.expected_index[key] for key in ("run_id", "task_id", "tree", "trust_sha256")), "index authority")
        require(len(index["members"]) == 4, "index membership")
        require({path.name for path in self.contexts.iterdir()} == {item["check_id"] + ".json" for item in self.members}, "context file membership")
        require(len({item["output_id"] for item in index["members"]}) == 4, "output alias")
        for expected, member in zip(self.members, index["members"]):
            require(set(member) == set(expected) and all(member[key] == expected[key] for key in
                    ("check_id", "context_path", "schema", "operation", "lane", "namespace", "required_ids", "output_id")), "member binding")
            path = Path(member["context_path"])
            require(path.is_file() and not path.is_symlink() and path.stat().st_nlink == 1 and
                    sha(path.read_bytes()) == member["context_sha256"], "child integrity")
            child = B.load_bytes(path.read_bytes())
            required_keys = set(self.children["CHK-001"])
            require(type(child) is dict and set(child) == required_keys and child["schema"] == member["schema"] == "tc-proof-runner-context/v2",
                    "child schema or fields do not match declared version")
            require(all(child[key] == value for key, value in {"run_id": self.run_id, "tree": self.source_tree,
                    "task_id": self.task_id, "check_id": member["check_id"], "operation": member["operation"]}.items()), "foreign child")
            if member["operation"] == "capture":
                require(child["lane"] == member["lane"], "lane substitution")

    def select(self, check_id):
        member = next(item for item in self.members if item["check_id"] == check_id)
        self.context = copy.deepcopy(self.children[check_id])
        self.operation = member["operation"]
        self.context_path = Path(member["context_path"])
        self.context_hash = member["context_sha256"]
        self.events = []
        self.violations = []
        self.nonce = secrets.token_hex(32)
        self.operator_env = {"TC_PROOF_CONTEXT_INDEX": str(self.index_path), "TC_PROOF_CONTEXT_INDEX_SHA256": self.index_hash,
            "TC_PROOF_CHECK_ID": check_id, "TC_PROOF_TASK_ID": self.task_id, "TC_PROOF_RESULT": str(self.output / member["output_id"])}

    def launch(self, request):
        if self.operation != "close":
            return super().launch(request)
        require(request == {"schema": "tc-proof-runner-observe/v1", "nonce": self.nonce, "operation": "close",
                            "source_commit": self.source_commit, "tree": self.source_tree} and not self.events, "closure observer binding")
        event = {"operation": "close", "run_id": self.run_id, "tree": self.source_tree,
                 "context_index_sha256": self.index_hash, "records": copy.deepcopy(self.completed)}
        self.events.append(event)
        return event

    def validate_close(self):
        require(len(self.events) == 1, "closure execution missing")
        records = self.events[0]["records"]
        require([row["check_id"] for row in records] == ["CHK-001", "CHK-002", "CHK-003"], "closure omitted or replayed check")
        require(all(row["status"] == "passed" for row in records), "closure accepted failed member")
        for row in records:
            member = next(item for item in self.members if item["check_id"] == row["check_id"])
            require(row["context_sha256"] == member["context_sha256"] and row["output_id"] == member["output_id"], "closure member binding")
            output = self.output / member["output_id"]
            require(output.is_file() and not output.is_symlink() and sha(canonical(B.load_bytes(output.read_bytes()))) == row["report_sha256"],
                    "completed result changed before closure")

    def qualify(self, runner):
        delayed = self.index_mutation in {"between_operations", "failed_member_close", "completed_output_changed"}
        for index, member in enumerate(self.members):
            self.select(member["check_id"])
            category = None
            if self.index_mutation != "valid" and not delayed:
                category = "CONTEXT_INDEX"
            elif self.index_mutation == "between_operations" and index == 1:
                changed = B.load_bytes(self.index_path.read_bytes())
                changed["trust_sha256"] = "0" * 64
                save(self.index_path, changed)
                # This is a protected-input corruption fixture, not a candidate
                # write permission. Keep the original independently pinned hash.
                self.before = self.snapshot()
                category = "CONTEXT_INDEX"
            elif self.index_mutation == "failed_member_close" and index == 3:
                self.completed[1]["status"] = "failed"
                category = "CLOSURE"
            elif self.index_mutation == "completed_output_changed" and index == 3:
                save(self.output / self.members[1]["output_id"], {"status": "passed", "forged": True})
                category = "CLOSURE"
            code, report = self.run(runner)
            if category:
                require(code != 0 and report["status"] == "rejected" and report["category"] == category and report["outputs"] == {}, "index rejection")
                return
            self.validate_index()
            if self.operation == "close":
                self.validate_close()
                require(code == 0 and report["status"] == "passed" and report["outputs"] == {"observations": self.events}, "indexed closure")
            else:
                super().validate((code, report), None)
                self.completed.append({"check_id": member["check_id"], "status": "passed", "context_sha256": member["context_sha256"],
                                       "output_id": member["output_id"], "report_sha256": sha(canonical(report))})


MUTATIONS = ["valid", "missing_child", "extra_child", "swap_direct_pty", "other_run", "other_tree", "other_task",
             "output_alias", "replay_second_id", "between_operations", "failed_member_close", "completed_output_changed",
             "child_wrong_schema", "child_extra_field"]


def self_test():
    for mutation in MUTATIONS:
        fixture = IndexFixture(mutation)
        try:
            if mutation in {"between_operations", "failed_member_close", "completed_output_changed"}:
                fixture.validate_index()
                if mutation == "between_operations":
                    index = B.load_bytes(fixture.index_path.read_bytes())
                    index["trust_sha256"] = "0" * 64
                    save(fixture.index_path, index)
            try:
                fixture.validate_index()
            except ValueError:
                require(mutation not in {"valid", "failed_member_close", "completed_output_changed"}, "valid index rejected")
            else:
                require(mutation in {"valid", "failed_member_close", "completed_output_changed"}, "index mutant passed")
            if mutation in {"valid", "failed_member_close", "completed_output_changed"}:
                # Real direct and PTY calls remain independently observed in the
                # same immutable multi-operation source campaign.
                spec = importlib.util.spec_from_file_location("index_observer", HERE / "host-bootstrap-observer.py")
                module = importlib.util.module_from_spec(spec)
                spec.loader.exec_module(module)
                fixture.sandbox = module.sandbox
                for member in fixture.members[:3]:
                    fixture.select(member["check_id"])
                    if fixture.operation == "capture":
                        fixture.launch({"schema": "tc-proof-runner-observe/v1", "nonce": fixture.nonce,
                            "operation": fixture.operation, "source_commit": fixture.source_commit, "tree": fixture.source_tree})
                        fixture.validate((0, {"status": "passed", "category": None, "outputs": {"observations": fixture.events}}), None)
                    observed_report = {"check_id": member["check_id"], "observations": fixture.events, "status": "passed"}
                    save(fixture.output / member["output_id"], observed_report)
                    fixture.completed.append({"check_id": member["check_id"], "status": "passed", "context_sha256": member["context_sha256"],
                        "output_id": member["output_id"], "report_sha256": sha(canonical(observed_report))})
                if mutation == "failed_member_close":
                    fixture.completed[1]["status"] = "failed"
                elif mutation == "completed_output_changed":
                    save(fixture.output / fixture.members[1]["output_id"], {"forged": True})
                fixture.select("CHK-004")
                fixture.launch({"schema": "tc-proof-runner-observe/v1", "nonce": fixture.nonce,
                    "operation": "close", "source_commit": fixture.source_commit, "tree": fixture.source_tree})
                try:
                    fixture.validate_close()
                except ValueError:
                    require(mutation in {"failed_member_close", "completed_output_changed"}, "valid sequence closure failed")
                else:
                    require(mutation == "valid", "failed member closure passed")
        finally:
            fixture.cleanup()
    return {"index_cases": len(MUTATIONS)}


def run(runner):
    for mutation in MUTATIONS:
        for selected in [mutation] + (["valid"] if mutation != "valid" else []):
            fixture = IndexFixture(selected)
            try:
                fixture.qualify(runner)
            finally:
                fixture.cleanup()
    return len(MUTATIONS)


if __name__ == "__main__":
    print(self_test())
