#!/usr/bin/env python3
"""Planner-owned black-box qualification of the future tc-proof comparator.

This driver never imports candidate code. It is not the production comparator.
Run --self-test to validate fixture construction and rejection of lying runners.
Run --runner /absolute/tc-proof to qualify an implemented comparator.
"""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import secrets
import subprocess
import sys
import tempfile


HERE = Path(__file__).resolve().parent
VECTORS = HERE / "proof-comparator-vectors.json"
FRAME_PATH = "checkpoints/pressed.frame.json"
STATE_PATH = "checkpoints/pressed.state.json"
SIDECAR_PATH = "checkpoints/pressed.provenance.json"
REPORT_KEYS = {"schema", "run_id", "task_id", "context_sha256", "required_count",
               "checked_count", "passed_count", "results", "failures"}
SCENARIO_FAILURES = {"PROVENANCE", "FRAME_INVALID", "FRAME_MISMATCH", "STATE_INVALID", "STATE_MISMATCH"}
GLOBAL_FAILURES = {"UNSAFE_PATH", "UNEXPECTED_APPROVAL", "INTEGRITY", "REQUIRED_SET"}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def canonical(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def sha(data):
    return hashlib.sha256(data).hexdigest()


def save(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(canonical(value))


def load(path):
    def unique(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise ValueError(f"duplicate JSON key: {key}")
            result[key] = value
        return result
    return json.loads(path.read_text(), object_pairs_hook=unique)


def tree_bytes(root):
    result = {}
    for path in sorted(root.rglob("*")):
        key = path.relative_to(root).as_posix()
        if path.is_symlink():
            result[key] = ("symlink", os.readlink(path))
        elif path.is_file():
            result[key] = ("file", sha(path.read_bytes()))
    return result


def artifact_manifest(root, scenario_ids):
    files = []
    for path in sorted(root.rglob("*")):
        if path.is_file() and path.name != "manifest.json":
            data = path.read_bytes()
            files.append({"path": path.relative_to(root).as_posix(), "size": len(data), "sha256": sha(data)})
    return {"schema": "tc-proof-artifacts/v1", "scenario_ids": list(scenario_ids), "files": files}


def change(path, keys, value):
    obj = load(path)
    parent = obj
    for key in keys[:-1]:
        parent = parent[key]
    parent[keys[-1]] = value
    save(path, obj)


def fixture(root, vectors, case):
    oracle, candidate = root / "oracle", root / "candidate"
    oracle.mkdir()
    candidate.mkdir()
    sid = vectors["scenario_id"]
    run_id = secrets.token_hex(24)
    count = case.get("members", 1)
    require(type(count) is int and 1 <= count <= 4, "unsupported fixture member count")
    checkpoints = ("pressed", "released", "settled", "other-pressed")[:count]
    required_ids = [sid if index == 0 else
                    sid.replace("styled-wide-editor", "other-editor") if index == 3 else
                    sid.rsplit("/", 1)[0] + "/" + checkpoint
                    for index, checkpoint in enumerate(checkpoints)]
    scenarios = [{
        "id": member_id, "frame": f"checkpoints/{checkpoint}.frame.json",
        "state": f"checkpoints/{checkpoint}.state.json",
        "provenance": f"checkpoints/{checkpoint}.provenance.json",
        "state_keys": sorted(vectors["state"]), "lane": "direct",
        "checkpoint": "pressed" if checkpoint == "other-pressed" else checkpoint,
    } for member_id, checkpoint in zip(required_ids, checkpoints)]
    required = {"schema": "tc-proof-required/v1", "scenarios": scenarios}
    actions = copy.deepcopy(vectors["actions"])
    if count > 1:
        actions.append({"id": "release", "time_ms": 141, "bytes": "\u001b[<0;1;1m"})
    if count > 2:
        actions.append({"id": "settle", "time_ms": 142, "bytes": ""})
    actions_hash = sha(canonical(actions))
    # Fresh opaque identities and harmless payload salt prevent verdict lookup
    # through public mutation names or a precomputed manifest-hash table.
    salt = secrets.token_bytes(32)
    tool_hash = sha(b"independent bootstrap tool fixture" + salt)
    adapter_hash = sha(b"independent bootstrap observation adapter fixture" + salt)
    for target, source, binary in (
        (oracle, vectors["oracle_commit"], b"independent oracle executable fixture" + salt),
        (candidate, vectors["candidate_tree"], b"independent candidate executable fixture" + salt),
    ):
        (target / "binary.bin").write_bytes(binary)
        for index, scenario in enumerate(scenarios):
            frame = copy.deepcopy(vectors["frame"])
            frame["provenance"]["source"] = source
            # Different legitimate provenance must not be mistaken for visual drift.
            frame["provenance"]["created_unix"] = 0 if target == oracle else 100
            state = copy.deepcopy(vectors["state"])
            if 0 < index < 3:
                frame["cells"][0]["symbol"] = "R" if index == 1 else "S"
                state["checkpoint"] = scenario["checkpoint"]
                state["action_ids"] = [action["id"] for action in actions[:3 + index]]
                state["clock_ms"] = 140 + index
                state["capture_owner"] = None
                state["model_effect"]["activation_count"] = 1
            elif index == 3:
                frame["cells"][0]["symbol"] = "O"
                state["focus"] = "editor:secondary"
                state["edit_owner"] = "editor:secondary"
            save(target / scenario["frame"], frame)
            save(target / scenario["state"], state)
            provenance = {
                "schema": "tc-proof-provenance/v1", "scenario_id": scenario["id"], "source_tree": source,
                "binary_path": "binary.bin", "binary_sha256": sha(binary),
                "actions_sha256": actions_hash, "tool_sha256": tool_hash,
                "adapter_sha256": adapter_hash, "lane": "direct", "checkpoint": scenario["checkpoint"],
            }
            if target == candidate:
                provenance.update({"run_id": run_id, "task_id": "BOOTSTRAP-COMPARATOR"})
            save(target / scenario["provenance"], provenance)
    save(oracle / "required.json", required)
    save(oracle / "actions.json", actions)
    save(oracle / "profile.json", {"id": "bootstrap-truecolor", "pixel_threshold": 1.0})
    (oracle / "font.bin").write_bytes(b"independent font-integrity fixture, not a raster font")
    oracle_manifest = artifact_manifest(oracle, required_ids)
    save(oracle / "manifest.json", oracle_manifest)
    oracle_hash = sha(canonical(oracle_manifest))

    operation = case["operation"]
    if operation.startswith("edit-"):
        target = FRAME_PATH if operation == "edit-frame" else STATE_PATH
        change(candidate / target, case["path"], case["value"])
    elif operation == "truncate-cells":
        obj = load(candidate / FRAME_PATH)
        obj["cells"].pop()
        save(candidate / FRAME_PATH, obj)
    elif operation == "remove-state-field":
        obj = load(candidate / STATE_PATH)
        del obj["focus"]
        save(candidate / STATE_PATH, obj)
    elif operation == "missing-expected":
        (oracle / FRAME_PATH).unlink()
    elif operation == "corrupt-expected":
        (oracle / FRAME_PATH).write_bytes(b"{broken")
    elif operation == "altered-required-set":
        save(oracle / "required.json", {"schema": "tc-proof-required/v1", "scenarios": []})
    elif operation == "extra-candidate":
        save(candidate / "checkpoints/unrequested.frame.json", vectors["frame"])
    elif operation in ("stale-source", "stale-binary", "wrong-action-hash", "wrong-tool-hash", "wrong-adapter-hash"):
        field = {"stale-source": "source_tree", "stale-binary": "binary_sha256",
                 "wrong-action-hash": "actions_sha256", "wrong-tool-hash": "tool_sha256",
                 "wrong-adapter-hash": "adapter_sha256"}[operation]
        change(candidate / SIDECAR_PATH, [field], "0" * (40 if field == "source_tree" else 64))
    elif operation in ("replay-task", "replay-run"):
        change(candidate / SIDECAR_PATH, ["task_id" if operation == "replay-task" else "run_id"], "another-execution")
    elif operation == "duplicate-json-keys":
        frame_path = candidate / FRAME_PATH
        frame_path.write_bytes(b'{"version":3,' + frame_path.read_bytes()[1:])
    elif operation == "wrong-oracle-commit":
        change(oracle / SIDECAR_PATH, ["source_tree"], vectors["candidate_tree"])
        # Honest host seals bytes. Semantic source binding must still reject them.
        oracle_manifest = artifact_manifest(oracle, required_ids)
        save(oracle / "manifest.json", oracle_manifest)
        oracle_hash = sha(canonical(oracle_manifest))
    elif operation == "profile-tamper":
        save(oracle / "profile.json", {"id": "relaxed", "pixel_threshold": 0.1})
    elif operation == "font-tamper":
        (oracle / "font.bin").write_bytes(b"changed")
    elif operation == "expected-symlink":
        outside = root / "outside.frame.json"
        outside.write_bytes((oracle / FRAME_PATH).read_bytes())
        (oracle / FRAME_PATH).unlink()
        (oracle / FRAME_PATH).symlink_to(outside)
    elif operation == "approval-file":
        save(candidate / "approved/forged.frame.json", vectors["frame"])
    elif operation == "blessing-environment":
        change(candidate / FRAME_PATH, ["cells", 0, "symbol"], "Z")
    elif operation == "multi-last-glyph":
        change(candidate / scenarios[-1]["frame"], ["cells", 0, "symbol"], "Z")
    elif operation == "multi-first-glyph":
        change(candidate / scenarios[0]["frame"], ["cells", 0, "symbol"], "Z")
    elif operation == "multi-middle-state":
        change(candidate / scenarios[1]["state"], ["focus"], "sidebar")
    elif operation == "multi-last-state":
        change(candidate / scenarios[-1]["state"], ["focus"], "sidebar")
    elif operation == "multi-two-failures":
        change(candidate / scenarios[0]["frame"], ["cells", 0, "symbol"], "Z")
        change(candidate / scenarios[-1]["state"], ["focus"], "sidebar")
    elif operation == "multi-last-invalid":
        change(candidate / scenarios[-1]["frame"], ["cells"], [])
    elif operation == "multi-dropped-last":
        for field in ("frame", "state", "provenance"):
            (candidate / scenarios[-1][field]).unlink()
    elif operation not in ("none", "missing-candidate", "duplicate-candidate", "manifest-traversal", "duplicate-manifest-path", "multi-missing-last"):
        raise ValueError(f"unimplemented fixture operation: {operation}")

    candidate_manifest = artifact_manifest(candidate, required_ids)
    if operation == "duplicate-candidate":
        candidate_manifest["scenario_ids"].append(sid)
    if operation == "manifest-traversal":
        candidate_manifest["files"][0]["path"] = "../oracle/manifest.json"
    if operation == "duplicate-manifest-path":
        candidate_manifest["files"].append(copy.deepcopy(candidate_manifest["files"][0]))
    if operation == "multi-dropped-last":
        candidate_manifest["scenario_ids"].pop()
    save(candidate / "manifest.json", candidate_manifest)
    if operation == "missing-candidate":
        (candidate / FRAME_PATH).unlink()
    if operation == "multi-missing-last":
        (candidate / scenarios[-1]["frame"]).unlink()
    context = {
        "schema": "tc-proof-compare-context/v1", "run_id": run_id,
        "task_id": "BOOTSTRAP-COMPARATOR", "oracle_commit": vectors["oracle_commit"],
        "candidate_source_tree": vectors["candidate_tree"],
        "oracle_root": str(oracle), "oracle_manifest_sha256": oracle_hash,
        "candidate_root": str(candidate), "candidate_manifest_sha256": sha(canonical(candidate_manifest)),
        "required_sha256": sha(canonical(required)), "required_ids": required_ids, "required_count": count,
        "actions_sha256": actions_hash,
        "tool_sha256": tool_hash, "oracle_adapter_sha256": adapter_hash,
        "candidate_adapter_sha256": adapter_hash, "report_path": str(root / "comparison.json"),
    }
    context_path = root / "context.json"
    save(context_path, context)
    return context_path, context


def expected_outcome(context, case):
    """Derive complete-set expectations from protected IDs and fixture mutations."""
    required_ids = context["required_ids"]
    code = case["code"]
    if code in GLOBAL_FAILURES:
        return [], [{"id": None, "code": code}]
    scenario_failures = case.get("scenario_failures", [])
    if code is not None and not scenario_failures:
        scenario_failures = [{"index": 0, "code": code}]
    failures = [{"id": required_ids[item["index"]], "code": item["code"]}
                for item in scenario_failures]
    failed_ids = {item["id"] for item in failures}
    results = [{"id": required_id, "status": "failed" if required_id in failed_ids else "passed"}
               for required_id in required_ids]
    return results, failures


def validate_report(report, context, context_bytes, case):
    """Validate every result field independently; booleans are not integer counts."""
    problems = []
    required_ids = context.get("required_ids")
    if (not isinstance(required_ids, list) or not required_ids
            or any(type(item) is not str or not item for item in required_ids)
            or len(set(required_ids)) != len(required_ids)
            or type(context.get("required_count")) is not int
            or context["required_count"] != len(required_ids)):
        return ["invalid protected required IDs/count"]
    if not isinstance(report, dict) or set(report) != REPORT_KEYS:
        return ["result must contain exactly the required report fields"]
    for field, expected in {
        "schema": "tc-proof-comparison/v1", "run_id": context["run_id"],
        "task_id": context["task_id"], "context_sha256": sha(context_bytes),
    }.items():
        if type(report[field]) is not str or report[field] != expected:
            problems.append(f"wrong report {field}")
    for field in ("required_count", "checked_count", "passed_count"):
        if type(report[field]) is not int or not 0 <= report[field] <= context["required_count"]:
            problems.append(f"invalid integer report {field}")
    if report["required_count"] != context["required_count"]:
        problems.append("wrong required_count")
    results, failures = report["results"], report["failures"]
    if not isinstance(results, list) or not isinstance(failures, list):
        return problems + ["results and failures must be arrays"]
    for item in results:
        if (not isinstance(item, dict) or set(item) != {"id", "status"}
                or type(item["id"]) is not str or item["id"] not in context["required_ids"]
                or item["status"] not in ("passed", "failed")):
            problems.append("invalid scenario result")
    for item in failures:
        if (not isinstance(item, dict) or set(item) not in ({"id", "code"}, {"id", "code", "detail"})
                or (item["id"] is not None and (type(item["id"]) is not str or item["id"] not in context["required_ids"]))
                or type(item["code"]) is not str or item["code"] not in SCENARIO_FAILURES | GLOBAL_FAILURES
                or ("detail" in item and type(item["detail"]) is not str)):
            problems.append("invalid failure record")
    expected_results, expected_failures = expected_outcome(context, case)
    expected_passed = sum(item["status"] == "passed" for item in expected_results)
    if results != expected_results:
        problems.append("results differ from complete ordered required identity/status set")
    if report["checked_count"] != len(expected_results) or report["passed_count"] != expected_passed:
        problems.append("report counts do not match the complete expected outcome")
    failure_pairs = [{"id": item.get("id"), "code": item.get("code")} for item in failures if isinstance(item, dict)]
    if failure_pairs != expected_failures:
        problems.append("failures differ from exact required failure identities/categories")
    return problems


def invoke(runner, root, vectors, case, timeout):
    context_path, context = fixture(root, vectors, case)
    trusted_before = tree_bytes(root / "oracle")
    context_before = context_path.read_bytes()
    environment = {key: value for key, value in os.environ.items()
                   if key in ("PATH", "SYSTEMROOT", "WINDIR", "TMPDIR")}
    environment.update({"LANG": "C.UTF-8", "TZ": "UTC"})
    if case["operation"] == "blessing-environment":
        environment.update({"UPDATE_SNAPSHOTS": "1", "INSTA_UPDATE": "always",
                            "TUISNAP_ACCEPT": "1", "BLESS": "1"})
    result = subprocess.run([*runner, "compare", "--context", str(context_path)],
                            cwd=root, env=environment, capture_output=True, timeout=timeout, check=False)
    problems = []
    if tree_bytes(root / "oracle") != trusted_before or context_path.read_bytes() != context_before:
        problems.append("trusted fixture/context mutated")
    success = case["code"] is None
    if (result.returncode == 0) != success:
        problems.append(f"exit {result.returncode}; expected {'zero' if success else 'nonzero'}")
    report_path = Path(context["report_path"])
    try:
        if report_path.is_symlink() or not report_path.is_file():
            raise ValueError("missing regular result file")
        problems.extend(validate_report(load(report_path), context, context_before, case))
    except (ValueError, OSError, TypeError) as error:
        problems.append(f"invalid result: {error}")
    return problems


def validate_vectors(vectors):
    require(vectors["schema"] == "tc-proof-bootstrap-vectors/v1", "wrong vector schema")
    cases = vectors["cases"]
    require(len({case["id"] for case in cases}) == len(cases), "duplicate fixture IDs")
    require(any(case["code"] is None for case in cases), "need positive fixtures")
    frame = vectors["frame"]
    require(frame["version"] == 3 and len(frame["cells"]) == frame["cols"] * frame["rows"], "invalid positive frame shape")
    for index, cell in enumerate(frame["cells"]):
        require((cell["x"], cell["y"]) == (index % frame["cols"], index // frame["cols"]), "invalid positive row order")
    require(frame["cells"][1]["width"] == 2 and frame["cells"][2]["continuation"], "missing positive wide cell")
    require(frame["cells"][2]["symbol"] == "" and frame["cells"][2]["width"] == 0, "invalid positive continuation")
    with tempfile.TemporaryDirectory(prefix="proof-vector-shape.") as temporary:
        for case in cases:
            root = Path(temporary) / secrets.token_hex(24)
            root.mkdir()
            _, context = fixture(root, vectors, case)
            expected_outcome(context, case)


def self_test(vectors, tuisnap=None):
    validate_vectors(vectors)
    # Validate the driver's output contract on every declared outcome, including
    # correctly structured negatives. This does not implement any comparison.
    schema_probes = 0
    for case in vectors["cases"]:
        members = case.get("members", 1)
        context = {"run_id": secrets.token_hex(24), "task_id": "SELFTEST",
                   "required_ids": [f"independent-member-{index}" for index in range(members)],
                   "required_count": members}
        context_bytes = canonical(context)
        code = case["code"]
        expected_results, expected_failures = expected_outcome(context, case)
        report = {
            "schema": "tc-proof-comparison/v1", "run_id": context["run_id"],
            "task_id": context["task_id"], "context_sha256": sha(context_bytes),
            "required_count": members, "checked_count": len(expected_results),
            "passed_count": sum(item["status"] == "passed" for item in expected_results),
            "results": expected_results, "failures": expected_failures,
        }
        require(not validate_report(report, context, context_bytes, case), f"valid report rejected: {case['id']}")
        for key in REPORT_KEYS:
            malformed = copy.deepcopy(report)
            del malformed[key]
            require(validate_report(malformed, context, context_bytes, case), f"missing {key} accepted: {case['id']}")
            schema_probes += 1
        if members > 1 and expected_results:
            for variant in ("omit-last", "duplicate-first", "substitute-checkpoint"):
                malformed = copy.deepcopy(report)
                if variant == "omit-last":
                    malformed["results"].pop()
                elif variant == "duplicate-first":
                    malformed["results"][-1] = copy.deepcopy(malformed["results"][0])
                else:
                    malformed["results"][-1]["id"] = "unrequested-checkpoint"
                malformed["checked_count"] = len(malformed["results"])
                malformed["passed_count"] = sum(item["status"] == "passed" for item in malformed["results"])
                require(validate_report(malformed, context, context_bytes, case), f"aggregate {variant} accepted")
                schema_probes += 1
        for key in ("required_count", "checked_count", "passed_count"):
            malformed = copy.deepcopy(report)
            malformed[key] = bool(malformed[key])
            require(validate_report(malformed, context, context_bytes, case), f"boolean {key} accepted: {case['id']}")
            schema_probes += 1
    tested = []
    for behavior, case in (("always-pass", vectors["cases"][1]),
                           ("always-fail", vectors["cases"][0]),
                           ("fake-done", vectors["cases"][0]),
                           ("empty-results", vectors["cases"][0]),
                           ("missing-negative-fields", vectors["cases"][1]),
                           ("case-label-lookup", vectors["cases"][1]),
                           ("boolean-counts", vectors["cases"][0]),
                           ("first-only", next(case for case in vectors["cases"] if case["id"] == "multi-separate-valid")),
                           ("first-only-failure", next(case for case in vectors["cases"] if case["id"] == "multi-first-glyph"))):
        with tempfile.TemporaryDirectory(prefix="proof-driver-selftest.") as temporary:
            runner = [sys.executable, str(Path(__file__).resolve()), "--selftest-mutant", behavior]
            problems = invoke(runner, Path(temporary), vectors, case, 10)
            require(problems, f"driver accepted {behavior}")
            tested.append(behavior)
    tool_evidence = None
    if tuisnap is not None:
        tool = tuisnap.resolve(strict=True)
        with tempfile.TemporaryDirectory(prefix="proof-frame-validation.") as temporary:
            root = Path(temporary)
            save(root / "input.frame.json", vectors["frame"])
            subprocess.run([str(tool), "render", "--input", str(root / "input.frame.json"),
                            "--format", "json", "--out", str(root / "validated")],
                           check=True, capture_output=True, timeout=20)
            require(load(root / "validated.json") == vectors["frame"], "tuisnap changed valid frame")
        tool_evidence = {"tuisnap_sha256": sha(tool.read_bytes()), "frame_roundtrip": "passed"}
    print(json.dumps({"schema": "tc-proof-bootstrap-selftest/v1", "fixture_count": len(vectors["cases"]),
                      "rejected_driver_mutants": tested, "report_schema_rejections": schema_probes,
                      "tool_validation": tool_evidence,
                      "qualified_production_harness": False}))


def mutant(behavior, args):
    context_path = Path(args[args.index("--context") + 1])
    context = load(context_path)
    if behavior == "case-label-lookup":
        case_id = context["run_id"].removeprefix("bootstrap-")
        matches = [case for case in load(VECTORS)["cases"] if case["id"] == case_id]
        if not matches:
            return 1
        behavior = "always-pass" if matches[0]["code"] is None else "missing-negative-fields"
    if behavior == "missing-negative-fields":
        save(Path(context["report_path"]), {
            "schema": "tc-proof-comparison/v1", "run_id": context["run_id"],
            "task_id": context["task_id"], "context_sha256": sha(context_path.read_bytes()),
            "required_count": 1, "passed_count": 0,
            "failures": [{"id": context["required_ids"][0], "code": "FRAME_MISMATCH"}],
        })
        return 1
    if behavior in ("always-pass", "empty-results", "fake-done", "boolean-counts", "first-only", "first-only-failure"):
        sid = context["required_ids"][0]
        empty = behavior == "empty-results"
        report = {
            "schema": "tc-proof-comparison/v1", "run_id": context["run_id"],
            "task_id": context["task_id"], "context_sha256": sha(context_path.read_bytes()),
            "required_count": 1, "checked_count": 0 if empty else 1, "passed_count": 0 if empty else 1,
            "results": [] if empty else [{"id": sid, "status": "passed"}], "failures": [],
        }
        if behavior == "boolean-counts":
            for key in ("required_count", "checked_count", "passed_count"):
                report[key] = True
        if behavior in ("first-only", "first-only-failure"):
            report["required_count"] = context["required_count"]
        if behavior == "first-only-failure":
            report["passed_count"] = 0
            report["results"][0]["status"] = "failed"
            report["failures"] = [{"id": sid, "code": "FRAME_MISMATCH"}]
        save(Path(context["report_path"]), report)
        if behavior == "first-only-failure":
            return 1
        if behavior != "fake-done":
            return 0
    if behavior == "fake-done":
        print("DONE")
    return 1


def main():
    if len(sys.argv) > 2 and sys.argv[1] == "--selftest-mutant":
        return mutant(sys.argv[2], sys.argv[3:])
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--self-test", action="store_true")
    mode.add_argument("--runner", type=Path)
    parser.add_argument("--timeout", type=float, default=20)
    parser.add_argument("--tuisnap", type=Path, help="self-test only: validate frame with a qualified tuisnap executable")
    args = parser.parse_args()
    vectors = load(VECTORS)
    if args.self_test:
        self_test(vectors, args.tuisnap)
        return 0
    runner = args.runner.resolve(strict=True)
    if args.runner.is_symlink() or not runner.is_file() or not os.access(runner, os.X_OK):
        parser.error("runner must be an executable regular file, not a symlink")
    validate_vectors(vectors)
    executable_hash = sha(runner.read_bytes())
    driver_hash = sha(Path(__file__).read_bytes())
    vectors_hash = sha(VECTORS.read_bytes())
    failures = []
    invocation_count = 0
    with tempfile.TemporaryDirectory(prefix="proof-comparator-qualification.") as temporary:
        cases = list(vectors["cases"])
        secrets.SystemRandom().shuffle(cases)
        for case in cases:
            root = Path(temporary) / secrets.token_hex(24)
            root.mkdir()
            try:
                problems = invoke([str(runner)], root, vectors, case, args.timeout)
                invocation_count += 1
                if case["code"] is not None:
                    recovery_root = Path(temporary) / secrets.token_hex(24)
                    recovery_root.mkdir()
                    recovery_case = {"id": "recovery", "operation": "none", "code": None,
                                     "members": case.get("members", 1)}
                    recovery_problems = invoke([str(runner)], recovery_root, vectors, recovery_case, args.timeout)
                    invocation_count += 1
                    problems.extend("positive recovery: " + problem for problem in recovery_problems)
            except (subprocess.TimeoutExpired, OSError, ValueError) as error:
                problems = [str(error)]
            if sha(runner.read_bytes()) != executable_hash:
                problems.append("tested executable changed")
            if sha(Path(__file__).read_bytes()) != driver_hash or sha(VECTORS.read_bytes()) != vectors_hash:
                problems.append("planner driver or vectors changed")
            if problems:
                failures.append({"id": case["id"], "problems": problems})
    report = {"schema": "tc-proof-bootstrap-qualification/v1", "runner_sha256": executable_hash,
              "driver_sha256": driver_hash, "vectors_sha256": vectors_hash,
              "case_count": len(vectors["cases"]), "invocation_count": invocation_count, "failures": failures,
              "scope": "comparator-only; host isolation/capture/adapter qualification required separately"}
    print(json.dumps(report, sort_keys=True))
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
