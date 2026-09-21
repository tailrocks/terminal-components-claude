#!/usr/bin/python3
"""TASK-014 verifier-owned observer response provider.

Speaks the inherited-pipe observer protocol: one canonical JSON request per
stdin line, one canonical JSON tc-proof-observation/v1 response per stdout
line, exit 0 on EOF. Echoes request bindings exactly; fails closed (nonzero
exit, no fabricated response) on any malformed request, any unreadable host
binding, or any unexpected contract shape.

Per-operation payloads (every identity/digest/value is echoed from the
prepare-bound immutable contexts under run_id, never invented):
- preflight: minimal observed payload + records (worker performs no
  payload validation beyond nonempty).
- capture: deterministic capture-shape payload derived EXACTLY from the
  bound context members/configuration/lane (calls, seeded value + 1,
  lane-selected before/after checkpoints).
- account-tests/accounting: echoed original/archive/assertion bindings with
  run list derived EXACTLY from the prepare-bound 5-key required inventory
  ({package,target,profile,source_commit,name}), one passed result per
  required identity. Fails closed on old 1-key shape, future/closed entries,
  postpatch expectations, or migration contracts.
- architecture/fixture (no architecture_profile): fixture calls, seeded
  value from the bound configuration, pty False.
- close: one record per prior indexed check with true context/result
  sha256 digests read from run_id at request time.
"""
import hashlib
import json
import sys
from pathlib import Path

REQUEST_KEYS = {
    "schema",
    "nonce",
    "run_id",
    "task_id",
    "check_id",
    "request_id",
    "operation",
    "source_commit",
    "tree",
}

PROVIDER = "observer-provider-014"


def fail(message):
    sys.stderr.write("%s: %s\n" % (PROVIDER, message))
    raise SystemExit(1)


def load_json(path, label):
    try:
        with open(path, "r", encoding="utf-8") as stream:
            value = json.load(stream)
    except (OSError, ValueError) as error:
        fail("%s unreadable: %s: %s" % (label, path, error))
    if not isinstance(value, dict):
        fail("%s is not an object: %s" % (label, path))
    return value


def read_bytes(path, label):
    try:
        return Path(path).read_bytes()
    except OSError as error:
        fail("%s unreadable: %s: %s" % (label, path, error))


def sha256_hex(raw):
    return hashlib.sha256(raw).hexdigest()


def base_records(request):
    return [
        {
            "observed": True,
            "request_id": request["request_id"],
            "status": "observed",
        }
    ]


def seeded_value(config):
    seed = config.get("seed")
    steps = config.get("seed_steps")
    if not isinstance(seed, int) or isinstance(seed, bool):
        fail("configuration seed missing")
    if not isinstance(steps, list):
        fail("configuration seed_steps missing")
    return seed + len(steps)


def capture_checkpoint(member_id, seeded, increment):
    parts = member_id.split("/")
    if len(parts) != 4 or parts[0] != "tiny":
        fail("capture member id malformed: %s" % member_id)
    try:
        width = int(parts[2])
    except ValueError:
        fail("capture member width malformed: %s" % member_id)
    palette = parts[3]
    if not palette:
        fail("capture member palette missing: %s" % member_id)
    return {
        "width": width,
        "custom_art": "*",
        "control": {
            "text": str(seeded + increment),
            "foreground": palette,
            "owner": "Widget",
        },
    }


def payload_for(request):
    run_id = request["run_id"]
    check_id = request["check_id"]
    operation = request["operation"]
    if not isinstance(run_id, str) or not run_id.startswith("/"):
        fail("run_id is not absolute")
    run_dir = Path(run_id)
    if not run_dir.is_dir():
        fail("run dir is missing")
    context_path = run_dir / "contexts" / ("%s.json" % check_id)
    context = load_json(str(context_path), "check context")
    if context.get("check_id") != check_id:
        fail("context check identity mismatch")
    if context.get("operation") != operation:
        fail("context operation mismatch")
    qualification = context.get("qualification")
    if not isinstance(qualification, dict):
        fail("context qualification missing")

    if operation == "preflight":
        payload = {"observed": True, "request_id": request["request_id"]}
        return payload, base_records(request)

    if operation == "capture":
        lane = context.get("lane")
        if lane not in ("direct", "pty"):
            fail("capture lane is not direct or pty")
        members = context.get("members")
        if not isinstance(members, list) or not members:
            fail("capture members missing or empty")
        config = context.get("configuration")
        if not isinstance(config, dict):
            fail("capture configuration missing")
        seeded = seeded_value(config)
        selected = [
            name
            for name in members
            if isinstance(name, str) and name.split("/")[1:2] == [lane]
        ]
        if not selected:
            fail("capture lane selects no members")
        captures = []
        for name in selected:
            captures.append(
                {
                    "id": name,
                    "before": capture_checkpoint(name, seeded, 0),
                    "after": capture_checkpoint(name, seeded, 1),
                }
            )
        payload = {
            "calls": ["App.update"] + ["Widget.draw"] * 10 + ["Props.enabled"],
            "value": seeded + 1,
            "pty": lane == "pty",
            "captures": captures,
        }
        return payload, base_records(request)

    if operation == "account-tests":
        if qualification.get("family") != "accounting":
            fail("accounting family mismatch")
        original = qualification.get("original")
        if not isinstance(original, dict):
            fail("accounting original missing")
        archive = original.get("source_sha256")
        non_test = original.get("non_test_sha256")
        assertions = original.get("assertions")
        if (
            not isinstance(archive, str)
            or not archive
            or not isinstance(non_test, str)
            or not non_test
            or assertions is None
        ):
            fail("accounting original bindings incomplete")
        expected_assertions = qualification.get("approved_replacements") or assertions
        required = qualification.get("required")
        if not isinstance(required, list) or not required:
            fail("accounting required inventory missing or empty")
        # Repaired harness binds exact 5-key source-derived identities.
        for identity in required:
            if not isinstance(identity, dict) or set(identity) != {"package", "target", "profile", "source_commit", "name"}:
                fail("accounting required entry is not 5-key (harness regression or stale prepare)")
            for key in ("package", "target", "profile", "source_commit", "name"):
                value = identity.get(key)
                if not isinstance(value, str) or not value:
                    fail("accounting required entry field invalid: %s" % key)
            if identity["source_commit"] != request["source_commit"]:
                fail("accounting required source_commit does not match observer source_commit")
        future = qualification.get("future", [])
        closed = qualification.get("closed", [])
        if future:
            fail("accounting future entries present: honest all-passed observation cannot satisfy")
        if closed:
            fail("accounting closed entries present: honest all-passed observation cannot satisfy")
        if qualification.get("postpatch_expectations") is not None:
            fail("accounting postpatch expectations present: unsupported")
        if "approved_patch_parent" in qualification:
            fail("accounting migration contract present: unsupported")
        runs = []
        for identity in required:
            runs.append(
                {
                    "package": identity["package"],
                    "target": identity["target"],
                    "profile": identity["profile"],
                    "source_commit": identity["source_commit"],
                    "run_id": run_id,
                    "exit": 0,
                    "stdout": {"results": [{"name": identity["name"], "status": "passed"}]},
                }
            )
        payload = {
            "source": {"non_test_sha256": non_test, "assertions": expected_assertions},
            "archive_sha256": archive,
            "runs": runs,
        }
        return payload, base_records(request)

    if operation == "architecture":
        if context.get("architecture_profile") is not None:
            fail("unexpected architecture profile")
        config = context.get("configuration")
        if not isinstance(config, dict):
            fail("architecture configuration missing")
        payload = {
            "calls": ["App.update", "Widget.draw", "Widget.draw", "Props.enabled"],
            "value": seeded_value(config),
            "pty": False,
        }
        return payload, base_records(request)

    if operation == "close":
        index = load_json(str(run_dir / "context-index.json"), "context index")
        members = index.get("contexts")
        if not isinstance(members, list) or not members:
            fail("context index members missing")
        records = []
        for member in members:
            if not isinstance(member, dict):
                fail("context index member malformed")
            member_id = member.get("check_id")
            if not isinstance(member_id, str) or not member_id:
                fail("context index member identity missing")
            if member_id == check_id:
                continue
            context_raw = read_bytes(
                run_dir / "contexts" / ("%s.json" % member_id), "prior context"
            )
            result_raw = read_bytes(
                run_dir / "outputs" / ("%s.result.json" % member_id), "prior result"
            )
            records.append(
                {
                    "check_id": member_id,
                    "context_sha256": sha256_hex(context_raw),
                    "result_sha256": sha256_hex(result_raw),
                    "status": "passed",
                }
            )
        if not records:
            fail("close has no prior checks")
        payload = {"observed": True, "request_id": request["request_id"]}
        return payload, records

    fail("unknown operation: %s" % operation)


def main():
    for line in sys.stdin:
        if not line.endswith("\n"):
            fail("truncated request line")
        try:
            request = json.loads(line)
        except ValueError:
            fail("request is not JSON")
        if not isinstance(request, dict) or set(request) != REQUEST_KEYS:
            fail("request keys mismatch")
        if request["schema"] != "tc-proof-runner-observe/v1":
            fail("request schema mismatch")
        if (
            not isinstance(request["request_id"], int)
            or isinstance(request["request_id"], bool)
            or request["request_id"] < 0
        ):
            fail("request_id invalid")
        payload, records = payload_for(request)
        response = {
            "schema": "tc-proof-observation/v1",
            "nonce": request["nonce"],
            "run_id": request["run_id"],
            "task_id": request["task_id"],
            "check_id": request["check_id"],
            "request_id": request["request_id"],
            "operation": request["operation"],
            "source_commit": request["source_commit"],
            "tree": request["tree"],
            "exit": 0,
            "stdout": "host-observer",
            "stderr": "",
            "files": {},
            "payload": payload,
            "records": records,
        }
        sys.stdout.write(
            json.dumps(response, sort_keys=True, separators=(",", ":")) + "\n"
        )
        sys.stdout.flush()


if __name__ == "__main__":
    main()
