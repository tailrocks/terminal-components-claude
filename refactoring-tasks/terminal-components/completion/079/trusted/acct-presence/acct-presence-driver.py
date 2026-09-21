#!/usr/bin/env python3
"""TASK-079 CHK-005 accounting-presence judge. This is not tc-proof.

Asserts that the prepared production account-tests context for CHK-005
carries a genuine prepared-context envelope (identity, trust, tool,
dependency, index, and result bindings plus exact template lift) and
that both accepted claims (`accepted_inventory`, `accepted_disposition`)
are present as well-formed provenance objects. This driver reads the
prepared context file only; it never executes workers, the observer
provider, or the dispatcher bundle, and it imports nothing outside the
standard library.

Projection correctness (receipt set equals the bound stub set) is
TASK-078's ownership, consumed via the base tree; this driver asserts
envelope plus accepted-claim presence and receipt shape only. Field
details live in trusted/obligations.md O-003.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
from pathlib import Path

TASK_ID = "TASK-079"
CHECK_ID = "CHK-005"
OPERATION = "account-tests"

CONTEXT_SCHEMA = "tc-proof-context/v1"
CONTEXT_INDEX_SCHEMA = "tc-proof-context-index/v1"
PREPARATION_RESULT_SCHEMA = "tc-proof-preparation-result/v1"

RECEIPT_KEYS = {"task_id", "sha256", "integration_commit"}

# Mirror of runner/context.py ALLOWED_V1_KEYS + OPTIONAL_EXTENSION_KEYS.
ALLOWED_KEYS = {
    "schema", "run_id", "task_id", "check_id", "worktree_commit", "scope_base",
    "operation", "tree", "oracle_commit", "oracle_tree", "bundle",
    "bundle_sha256", "tool", "dependencies", "adapter", "lane", "axes",
    "members", "inventory", "evidence", "configuration", "qualification",
    "observer_sequence", "architecture_profile", "branch_host_projection",
}
OBSERVER_OPERATIONS = {
    "account-tests", "architecture", "capture", "close", "compare",
    "oracle", "preflight", "required",
}

HEX40 = re.compile(r"^[0-9a-f]{40}$")
HEX64 = re.compile(r"^[0-9a-f]{64}$")

FAILURES = []  # type: list


def fail(scope, field, detail):
    FAILURES.append("FAIL %s %s: %s" % (scope, field, detail))


def load_strict(path):
    # type: (Path) -> object
    try:
        raw = path.read_bytes()
    except OSError as error:
        return error
    try:
        return (raw, json.loads(raw.decode("utf-8"),
                               object_pairs_hook=_unique))
    except (ValueError, UnicodeDecodeError) as error:
        return error


def _unique(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("duplicate JSON key: %r" % (key,))
        result[key] = value
    return result


def sha256_bytes(data):
    # type: (bytes) -> str
    return hashlib.sha256(data).hexdigest()


def git(worktree, *args):
    # type: (...) -> str
    result = subprocess.run(
        ["/usr/bin/git", "-C", str(worktree)] + list(args),
        stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
    if result.returncode != 0:
        raise RuntimeError("git %s failed: %s"
                           % (" ".join(args),
                              result.stderr.decode("utf-8",
                                                   "replace").strip()))
    return result.stdout.decode("utf-8").strip()


def check_host_file(scope, path, label):
    # type: (str, Path, str) -> bool
    if path.is_symlink() or not path.is_file():
        fail(scope, label, "not a host-owned regular file: %s" % path)
        return False
    try:
        if path.stat().st_nlink != 1:
            fail(scope, label, "link count is not 1: %s" % path)
            return False
    except OSError as error:
        fail(scope, label, "stat failed: %s" % error)
        return False
    return True


def check_hex(scope, value, field, forty):
    # type: (str, object, str, bool) -> bool
    pattern = HEX40 if forty else HEX64
    if not isinstance(value, str) or not pattern.match(value):
        fail(scope, field, "must be %s lowercase hex"
             % ("40-char" if forty else "64-char"))
        return False
    return True


def check_common(args, run_dir, context, context_raw, context_path):
    # type: (...) -> dict
    scope = args.check_id
    if not isinstance(context, dict):
        fail(scope, "context", "top level is not an object")
        return {}
    if context.get("schema") != CONTEXT_SCHEMA:
        fail(scope, "schema", "must be %s" % CONTEXT_SCHEMA)
    if context.get("task_id") != TASK_ID:
        fail(scope, "task_id", "must be %s" % TASK_ID)
    if context.get("check_id") != args.check_id:
        fail(scope, "check_id", "must match %s" % args.check_id)
    run_id = context.get("run_id")
    if not isinstance(run_id, str) or not Path(run_id).is_absolute() \
            or Path(run_id).resolve() != run_dir.resolve():
        fail(scope, "run_id", "must be the canonical run directory")
    if context.get("operation") != args.operation:
        fail(scope, "operation", "must be %s" % args.operation)
    for field in ("worktree_commit", "scope_base", "tree", "oracle_commit",
                  "oracle_tree"):
        check_hex(scope, context.get(field), field, True)
    if not isinstance(context.get("bundle"), str) or not context["bundle"]:
        fail(scope, "bundle", "must be a non-empty tag name")
    check_hex(scope, context.get("bundle_sha256"), "bundle_sha256", False)

    unknown = set(context) - ALLOWED_KEYS
    if unknown:
        fail(scope, "keys", "unknown top-level keys: %s" % sorted(unknown))

    sequence = context.get("observer_sequence")
    if (not isinstance(sequence, list) or not sequence
            or any(item not in OBSERVER_OPERATIONS for item in sequence)
            or sequence[0] != args.operation):
        fail(scope, "observer_sequence",
             "must be a non-empty known-operator list led by %s"
             % args.operation)
    elif args.operation != "oracle" and sequence != [args.operation]:
        fail(scope, "observer_sequence",
             "must be [%s] for non-oracle operations" % args.operation)

    # Adapter / source-authority bindings (runner/context.py demand).
    if "source_directory" in context:
        fail(scope, "source_directory", "must be absent")
    adapter = context.get("adapter")
    if not isinstance(adapter, dict) or adapter.get("changes"):
        fail(scope, "adapter.changes", "must be a present empty list")

    # Axes / members shape. Exact provider agreement is proven by the
    # adapter end-to-end gates; here we require well-formed bindings.
    axes = context.get("axes")
    members = context.get("members")
    if (not isinstance(axes, dict) or set(axes) != {"lanes", "widths",
                                                   "palettes"}):
        fail(scope, "axes", "must bind lanes/widths/palettes")
    else:
        for axis in ("lanes", "widths", "palettes"):
            values = axes[axis]
            if (not isinstance(values, list) or not values
                    or any(not isinstance(item, (str, int))
                           for item in values)):
                fail(scope, "axes.%s" % axis,
                     "must be a non-empty value list")
    if (not isinstance(members, list) or not members
            or any(not isinstance(item, str) or not item
                   for item in members)):
        fail(scope, "members", "must be a non-empty string list")
    elif len(set(members)) != len(members):
        fail(scope, "members", "must be unique")

    # Qualification common trust bindings (proof-contract demand).
    qualification = context.get("qualification")
    if not isinstance(qualification, dict):
        fail(scope, "qualification", "must be an object")
        return {}
    common = qualification.get("common")
    if not isinstance(common, dict):
        fail(scope, "qualification.common", "must be an object")
        return {}
    common_run_id = common.get("run_id")
    if not isinstance(common_run_id, str) \
            or Path(common_run_id).resolve() != run_dir.resolve():
        fail(scope, "common.run_id", "must match the run directory")
    if common.get("task_id") != TASK_ID:
        fail(scope, "common.task_id", "must match %s" % TASK_ID)
    if common.get("candidate_commit") != context.get("worktree_commit"):
        fail(scope, "common.candidate_commit", "must match worktree_commit")
    if common.get("candidate_tree") != context.get("tree"):
        fail(scope, "common.candidate_tree", "must match tree")
    oracle = common.get("oracle")
    if not isinstance(oracle, dict):
        fail(scope, "common.oracle", "must be an object")
    else:
        if oracle.get("commit") != context.get("oracle_commit"):
            fail(scope, "common.oracle.commit", "must match oracle_commit")
        if oracle.get("tree") != context.get("oracle_tree"):
            fail(scope, "common.oracle.tree", "must match oracle_tree")
    outputs = common.get("outputs")
    runtime = None
    if not isinstance(outputs, dict):
        fail(scope, "common.outputs", "must be an object")
    else:
        runtime = outputs.get("runtime")
        if not isinstance(runtime, str) \
                or Path(runtime).resolve() != (run_dir / "outputs").resolve():
            fail(scope, "common.outputs.runtime",
                 "must be the run outputs directory")
        elif not Path(runtime).is_dir():
            fail(scope, "common.outputs.runtime", "must exist as a directory")
    observer = common.get("observer")
    if (not isinstance(observer, dict)
            or observer.get("transport") != "inherited-pipe/v1"
            or not isinstance(observer.get("nonce"), str)
            or not observer["nonce"]):
        fail(scope, "common.observer",
             "must bind inherited-pipe/v1 with a non-empty nonce")
    comparator = common.get("comparator")
    if not isinstance(comparator, dict):
        fail(scope, "common.comparator", "must be an object")
    else:
        if not isinstance(comparator.get("path"), str) \
                or not comparator["path"]:
            fail(scope, "common.comparator.path",
                 "must be a non-empty path")
        check_hex(scope, comparator.get("sha256"),
                  "common.comparator.sha256", False)
    tool = context.get("tool")
    if not isinstance(tool, dict) or common.get("tool") != tool:
        fail(scope, "tool", "outer and common tool bindings must agree")
    else:
        dispatcher = Path(args.dispatcher)
        if not dispatcher.is_absolute():
            dispatcher = Path.cwd() / dispatcher
        bound_tool = tool.get("path")
        if not isinstance(bound_tool, str) \
                or Path(bound_tool).resolve() != dispatcher.resolve():
            fail(scope, "tool.path", "must be the candidate dispatcher")
        else:
            try:
                actual = sha256_bytes(dispatcher.read_bytes())
            except OSError as error:
                fail(scope, "tool.sha256",
                     "dispatcher unreadable: %s" % error)
                actual = None
            if actual is not None and tool.get("sha256") != actual:
                fail(scope, "tool.sha256",
                     "must match the candidate dispatcher bytes")
            else:
                check_hex(scope, tool.get("sha256"), "tool.sha256", False)
    if not isinstance(qualification.get("trust_manifest"), dict):
        fail(scope, "trust_manifest", "must be an object")
    check_hex(scope, qualification.get("trust_manifest_sha256"),
              "trust_manifest_sha256", False)

    # Task-manifest binding: phase/requirements/acceptance from verify.toml.
    check_binding = qualification.get("check")
    manifest = read_manifest_check(args.check_id)
    if not isinstance(check_binding, dict):
        fail(scope, "qualification.check", "must be an object")
    elif manifest is None:
        fail(scope, "verify.toml", "manifest entry for %s unreadable"
             % args.check_id)
    else:
        for field in ("phase", "requirements", "acceptance"):
            if check_binding.get(field) != manifest[field]:
                fail(scope, "qualification.check.%s" % field,
                     "must match verify.toml (%r)" % (manifest[field],))
        check_hex(scope, check_binding.get("command_sha256"),
                  "qualification.check.command_sha256", False)
        if check_binding.get("lane") != "direct":
            fail(scope, "qualification.check.lane", "must be direct")

    # Dependencies: every bound receipt must be accepted and integrated.
    dependencies = context.get("dependencies")
    if not isinstance(dependencies, list):
        fail(scope, "dependencies", "must be a list")
    else:
        for entry in dependencies:
            if (not isinstance(entry, dict) or not entry.get("accepted")
                    or not entry.get("integrated")):
                fail(scope, "dependencies",
                     "every bound receipt must be accepted+integrated")
                break

    # Source binding against the real worktree.
    worktree = common.get("worktree")
    if not isinstance(worktree, str) or not Path(worktree).is_dir():
        fail(scope, "common.worktree", "must be an existing directory")
    else:
        try:
            head = git(worktree, "rev-parse", "HEAD")
            tree = git(worktree, "rev-parse", "HEAD^{tree}")
        except RuntimeError as error:
            fail(scope, "common.worktree", str(error))
        else:
            if head != context.get("worktree_commit"):
                fail(scope, "worktree_commit",
                     "must equal the worktree HEAD commit")
            if tree != context.get("tree"):
                fail(scope, "tree", "must equal the worktree HEAD tree")
            try:
                subprocess.run(
                    ["/usr/bin/git", "-C", worktree, "merge-base",
                     "--is-ancestor", str(context.get("scope_base")), head],
                    stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                    check=True)
            except subprocess.CalledProcessError:
                fail(scope, "scope_base",
                     "must resolve to an ancestor of HEAD")

    # Context-index binding (proof-contract demand).
    index_path = run_dir / "context-index.json"
    loaded = load_strict(index_path)
    if isinstance(loaded, Exception):
        fail(scope, "context-index.json", "unreadable: %s" % loaded)
    else:
        _, index = loaded
        if not isinstance(index, dict):
            fail(scope, "context-index.json", "top level is not an object")
        else:
            if index.get("schema") != CONTEXT_INDEX_SCHEMA:
                fail(scope, "context-index.json/schema",
                     "must be %s" % CONTEXT_INDEX_SCHEMA)
            for field in ("task_id", "worktree_commit", "scope_base"):
                if index.get(field) != context.get(field):
                    fail(scope, "context-index.json/%s" % field,
                         "must match the context")
            index_run_id = index.get("run_id")
            if not isinstance(index_run_id, str) \
                    or Path(index_run_id).resolve() != run_dir.resolve():
                fail(scope, "context-index.json/run_id",
                     "must match the run directory")
            members_idx = index.get("contexts")
            match = None
            if isinstance(members_idx, list):
                for member in members_idx:
                    if (isinstance(member, dict)
                            and member.get("check_id") == args.check_id):
                        match = member
            if match is None:
                fail(scope, "context-index.json",
                     "no member entry for %s" % args.check_id)
            else:
                member_path = match.get("path")
                if not isinstance(member_path, str) \
                        or Path(member_path).resolve() \
                        != context_path.resolve():
                    fail(scope, "context-index.json/path",
                         "must be the host-bound context path")
                if match.get("sha256") != sha256_bytes(context_raw):
                    fail(scope, "context-index.json/sha256",
                         "must match the context bytes")

    # Preparation-result binding (proof-contract demand).
    result_path = run_dir / "results" / ("%s.json" % args.check_id)
    loaded = load_strict(result_path)
    if isinstance(loaded, Exception):
        fail(scope, "results/%s.json" % args.check_id,
             "unreadable: %s" % loaded)
    else:
        _, result = loaded
        if not isinstance(result, dict):
            fail(scope, "results", "top level is not an object")
        else:
            if result.get("schema") != PREPARATION_RESULT_SCHEMA:
                fail(scope, "results/schema",
                     "must be %s" % PREPARATION_RESULT_SCHEMA)
            if result.get("status") != "ready":
                fail(scope, "results/status", "must be ready")
            if result.get("check_id") != args.check_id:
                fail(scope, "results/check_id", "must match")
            if result.get("context_sha256") != sha256_bytes(context_raw):
                fail(scope, "results/context_sha256",
                     "must match the context bytes")
    return qualification if isinstance(qualification, dict) else {}


def check_template_lift(args, qualification, task_dir):
    # type: (...) -> None
    """Assert the declared template values are lifted into qualification."""
    scope = args.check_id
    template_path = (task_dir / "trusted" / "check-context-templates"
                     / ("%s.json" % args.check_id))
    try:
        template_raw = template_path.read_bytes()
        template = json.loads(template_raw.decode("utf-8"),
                              object_pairs_hook=_unique)
    except (OSError, ValueError, UnicodeDecodeError) as error:
        fail(scope, "template", "unreadable: %s" % error)
        return
    if not isinstance(template, dict):
        fail(scope, "template", "top level is not an object")
        return
    declared = template.get("qualification")
    if not isinstance(declared, dict):
        fail(scope, "template.qualification", "must be an object")
        return
    # Declared template values must be lifted into the qualification.
    for field, want in sorted(declared.items()):
        if qualification.get(field) != want:
            fail(scope, "qualification.%s" % field,
                 "must lift the template declaration (%r)" % (want,))
    if qualification.get("worker_context") != template:
        fail(scope, "qualification.worker_context",
             "must equal the template bytes")
    if qualification.get("template_sha256") != sha256_bytes(template_raw):
        fail(scope, "qualification.template_sha256",
             "must match the template bytes")


def check_accepted_presence(args, qualification):
    # type: (...) -> None
    """Assert both accepted claims are present and well-formed.

    Presence and receipt shape only: receipt-set equality against the
    bound stubs is TASK-078's ownership and is not re-judged here.
    """
    scope = args.check_id
    if qualification.get("mode") != "production":
        fail(scope, "qualification.mode", "must be production")
    if qualification.get("requires_inventory_receipt") is not True \
            or qualification.get("requires_disposition_receipt") is not True:
        fail(scope, "qualification.requires_*",
             "both requires flags must be true")
        return
    for field in ("accepted_inventory", "accepted_disposition"):
        claim = qualification.get(field)
        if not isinstance(claim, dict) or not claim:
            fail(scope, "qualification.%s" % field,
                 "production mode with the requires flag must bind a "
                 "non-empty provenance object")
            continue
        receipts = claim.get("receipts")
        if not isinstance(receipts, list) or not receipts:
            fail(scope, "qualification.%s.receipts" % field,
                 "must carry a non-empty receipts list")
            continue
        for receipt in receipts:
            if not isinstance(receipt, dict) or set(receipt) != RECEIPT_KEYS:
                fail(scope, "qualification.%s.receipts" % field,
                     "every receipt must carry exactly %s"
                     % sorted(RECEIPT_KEYS))
                break
            if not isinstance(receipt["task_id"], str) \
                    or not receipt["task_id"]:
                fail(scope, "qualification.%s.receipts.task_id" % field,
                     "must be a non-empty string")
                break
            if not check_hex(scope, receipt["sha256"],
                             "qualification.%s.receipts.sha256" % field,
                             False):
                break
            if not check_hex(scope, receipt["integration_commit"],
                             "qualification.%s.receipts.integration_commit"
                             % field, True):
                break


def read_manifest_check(check_id):
    # type: (str) -> dict | None
    """Extract one check entry from this package's verify.toml.

    Targeted text parse (no TOML dependency): split on [[checks]] tables,
    then read the id/phase/requirements/acceptance lines of the match.
    """
    here = Path(__file__).resolve()
    task_dir = here.parent.parent.parent
    try:
        text = (task_dir / "verify.toml").read_text(encoding="utf-8")
    except OSError:
        return None
    tables = re.split(r"(?m)^\[\[checks\]\]\s*$", text)[1:]
    for table in tables:
        match_id = re.search(r'(?m)^id\s*=\s*"([^"]+)"\s*$', table)
        if not match_id or match_id.group(1) != check_id:
            continue
        match_phase = re.search(r'(?m)^phase\s*=\s*"([^"]+)"\s*$', table)
        match_req = re.search(r"(?m)^requirements\s*=\s*\[(.*)\]\s*$",
                              table)
        match_acc = re.search(r"(?m)^acceptance\s*=\s*\[(.*)\]\s*$", table)
        if not match_phase or not match_req or not match_acc:
            return None
        return {"phase": match_phase.group(1),
                "requirements": re.findall(r'"([^"]+)"', match_req.group(1)),
                "acceptance": re.findall(r'"([^"]+)"', match_acc.group(1))}
    return None


def parse_args(argv):
    # type: (list) -> argparse.Namespace
    parser = argparse.ArgumentParser(
        description="Assert the CHK-005 prepared context is genuine and "
                    "carries both accepted claims.")
    parser.add_argument("--check-id", required=True)
    parser.add_argument("--context", required=True)
    parser.add_argument("dispatcher",
                        help="candidate dispatcher (identity cross-check)")
    parser.add_argument("operation", help="expected bound operation")
    return parser.parse_args(argv)


def main(argv):
    # type: (list) -> int
    args = parse_args(argv)
    if args.check_id != CHECK_ID or args.operation != OPERATION:
        print("FAIL %s binding: this driver judges %s %s only"
              % (args.check_id, CHECK_ID, OPERATION))
        return 1
    run_dir = os.environ.get("RUN_DIR")
    if not run_dir:
        print("FAIL %s env: RUN_DIR is not set" % args.check_id)
        return 1
    run_dir_path = Path(run_dir)
    if not run_dir_path.is_absolute() or not run_dir_path.is_dir():
        print("FAIL %s env: RUN_DIR is not an existing absolute directory"
              % args.check_id)
        return 1
    task_dir = Path(__file__).resolve().parent.parent.parent
    context_path = Path(args.context)
    expected = run_dir_path / "contexts" / ("%s.json" % args.check_id)
    if context_path != expected:
        fail(args.check_id, "context path",
             "must be $RUN_DIR/contexts/%s.json" % args.check_id)
    if check_host_file(args.check_id, context_path, "context file"):
        loaded = load_strict(context_path)
        if isinstance(loaded, Exception):
            fail(args.check_id, "context", "unreadable: %s" % loaded)
        else:
            raw, context = loaded
            qualification = check_common(args, run_dir_path, context, raw,
                                         context_path)
            check_template_lift(args, qualification, task_dir)
            check_accepted_presence(args, qualification)
    if FAILURES:
        for line in FAILURES:
            print(line)
        print("ACCT-RESULT %s status=rejected failures=%d"
              % (args.check_id, len(FAILURES)))
        return 1
    print("ACCT-RESULT %s status=passed" % args.check_id)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
