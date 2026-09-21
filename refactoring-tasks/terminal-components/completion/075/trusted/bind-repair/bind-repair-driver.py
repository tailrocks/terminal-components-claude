#!/usr/bin/env python3
"""TASK-075 independent bind-repair judge. This is not tc-proof.

Asserts that `tc-proof prepare` emitted genuine runnable per-check contexts
(real compare roots, five-key accounting inventory) and that the fixed
source `runner/__main__.py` compare branch supervises the native comparator
and emits its runner result. Binding profiles read prepared context files
only; the supervision profile executes the fixed source branch exactly once
as a subprocess. The driver never executes 071/072-owned workers, the
observer provider, or the frozen dispatcher bundle, and it imports nothing
outside the standard library.

Every asserted field is either bound by prepare today (regression guard) or
derivable by prepare from the task manifest, the oracle, and the candidate
worktree. The exact derivation design is the implementer's; this driver
checks presence, shape, and cross-consistency against the named demand sites
in trusted/obligations.md.
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

TASK_ID = "TASK-075"

CONTEXT_SCHEMA = "tc-proof-context/v1"
COMPARE_CONTEXT_SCHEMA = "tc-proof-compare-context/v1"
CONTEXT_INDEX_SCHEMA = "tc-proof-context-index/v1"
PREPARATION_RESULT_SCHEMA = "tc-proof-preparation-result/v1"
ARTIFACTS_SCHEMA = "tc-proof-artifacts/v1"
REQUIRED_SCHEMA = "tc-proof-required/v1"
PROVENANCE_SCHEMA = "tc-proof-provenance/v1"
COMPARISON_SCHEMA = "tc-proof-comparison/v1"
RUNNER_RESULT_SCHEMA = "tc-proof-runner-result/v1"
SUPERVISION_SCHEMA = "tc-proof-bind-repair-supervision/v1"

RUNNER_RESULT_KEYS = {
    "schema", "run_id", "operation", "context_sha256", "status",
    "category", "observation_digests", "outputs",
}
INVENTORY_KEYS = {"package", "target", "profile", "source_commit", "name"}

SUPERVISION_TIMEOUT_SECONDS = 300
SUPERVISION_STDERR_TAIL = 2048

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


# Hardcoded fixture default in verifier.rs build_context. Worker-path checks
# must bind task-derived membership instead (see obligations.md O-005).
FIXTURE_AXES = {"lanes": ["direct", "pty"], "widths": [8, 12],
                "palettes": ["blue", "yellow"]}
FIXTURE_MEMBERS = [
    "tiny/direct/8/blue", "tiny/direct/8/yellow", "tiny/direct/12/blue",
    "tiny/direct/12/yellow", "tiny/pty/8/blue", "tiny/pty/8/yellow",
    "tiny/pty/12/blue", "tiny/pty/12/yellow",
]

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


def canonical_bytes(value):
    # type: (object) -> bytes
    """Canonical JSON bytes: sorted keys, compact separators, UTF-8.

    Mirrors `json_util::canonical_json` (sorted keys, no escaping beyond
    JSON string rules). Member identifiers, scenario paths, and digests are
    plain ASCII, so Python and serde_json serializations agree byte for byte.
    """
    return json.dumps(value, sort_keys=True, separators=(",", ":"),
                      ensure_ascii=False).encode("utf-8")


def is_safe_relative_path(path):
    # type: (object) -> bool
    """Mirror of `json_util::is_safe_relative_path`."""
    return (isinstance(path, str) and bool(path)
            and not path.startswith("/") and "\\" not in path
            and all(part != ".." for part in path.split("/")))


def scan_tree_symlink_free(root):
    # type: (Path) -> str | None
    """Return None when `root` holds no symlink; else the offending path."""
    if root.is_symlink():
        return str(root)
    stack = [root]
    while stack:
        current = stack.pop()
        try:
            with os.scandir(current) as entries:
                children = list(entries)
        except OSError:
            return str(current)
        for entry in children:
            try:
                if entry.is_symlink():
                    return str(Path(entry.path))
                if entry.is_dir(follow_symlinks=False):
                    stack.append(Path(entry.path))
            except OSError:
                return str(Path(entry.path))
    return None


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
    # adapter end-to-end gates; here we require well-formed bindings and,
    # for worker paths, task-derived (non-fixture-default) membership.
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


def check_task_derived_members(args, context):
    # type: (...) -> None
    scope = args.check_id
    if context.get("axes") == FIXTURE_AXES \
            and context.get("members") == FIXTURE_MEMBERS:
        fail(scope, "members",
             "worker-path binding must be task-derived, not the tiny "
             "fixture default")


def check_regular_single_link(scope, path, field):
    # type: (str, Path, str) -> bytes | None
    """Read `path` when it is a regular single-link file; else fail."""
    if path.is_symlink() or not path.is_file():
        fail(scope, field, "not a regular file: %s" % path)
        return None
    try:
        if path.stat().st_nlink != 1:
            fail(scope, field, "link count is not 1: %s" % path)
            return None
    except OSError as error:
        fail(scope, field, "stat failed: %s" % error)
        return None
    try:
        return path.read_bytes()
    except OSError as error:
        fail(scope, field, "unreadable: %s" % error)
        return None


def check_manifest_tree(scope, root, bound_digest, required_ids):
    # type: (...) -> dict | None
    """Verify one artifact-tree manifest against bytes on disk.

    Returns the parsed manifest value on success; every mismatch fails.
    Mirrors `compare.rs::verify_manifest` for the structural file contract.
    """
    manifest_path = root / "manifest.json"
    raw = check_regular_single_link(scope, manifest_path,
                                    "tree.manifest")
    if raw is None:
        return None
    if sha256_bytes(raw) != bound_digest:
        fail(scope, "tree.manifest_sha256",
             "bound digest is not the manifest bytes digest: %s" % root)
        return None
    try:
        manifest = json.loads(raw.decode("utf-8"), object_pairs_hook=_unique)
    except (ValueError, UnicodeDecodeError) as error:
        fail(scope, "tree.manifest", "invalid JSON: %s" % error)
        return None
    if not isinstance(manifest, dict):
        fail(scope, "tree.manifest", "top level is not an object")
        return None
    if manifest.get("schema") != ARTIFACTS_SCHEMA:
        fail(scope, "tree.manifest.schema", "must be %s" % ARTIFACTS_SCHEMA)
        return None
    scenario_ids = manifest.get("scenario_ids")
    if (not isinstance(scenario_ids, list)
            or len(set(scenario_ids)) != len(scenario_ids)
            or scenario_ids != required_ids):
        fail(scope, "tree.manifest.scenario_ids",
             "must equal required_ids in order")
        return None
    files = manifest.get("files")
    if not isinstance(files, list):
        fail(scope, "tree.manifest.files", "must be a list")
        return None
    seen = set()
    ok = True
    for entry in files:
        if not isinstance(entry, dict):
            fail(scope, "tree.manifest.files", "entries must be objects")
            ok = False
            continue
        rel = entry.get("path")
        size = entry.get("size")
        digest = entry.get("sha256")
        if not is_safe_relative_path(rel):
            fail(scope, "tree.manifest.files.path",
                 "must be a safe relative path: %r" % (rel,))
            ok = False
            continue
        if rel in seen:
            fail(scope, "tree.manifest.files.path",
                 "duplicate entry: %s" % rel)
            ok = False
            continue
        seen.add(rel)
        if not isinstance(size, int) or isinstance(size, bool) or size < 0:
            fail(scope, "tree.manifest.files.size",
                 "must be an unsigned integer: %s" % rel)
            ok = False
            continue
        if not isinstance(digest, str) or not HEX64.match(digest):
            fail(scope, "tree.manifest.files.sha256",
                 "must be 64-char hex: %s" % rel)
            ok = False
            continue
        data = check_regular_single_link(scope, root / rel,
                                         "tree.file")
        if data is None:
            ok = False
            continue
        if len(data) != size or sha256_bytes(data) != digest:
            fail(scope, "tree.file",
                 "size/digest mismatch: %s" % rel)
            ok = False
    if not ok:
        return None
    return manifest


def check_required_document(scope, oracle_root, nested, required_ids):
    # type: (...) -> dict | None
    """Verify oracle `required.json` and its canonical digest binding.

    Mirrors `compare.rs::check_required_set` for the structural contract:
    schema, scenario coverage in order, and `required_sha256` equal to the
    canonical digest of the document value (not of the id list).
    """
    required_path = oracle_root / "required.json"
    raw = check_regular_single_link(scope, required_path,
                                    "tree.required")
    if raw is None:
        return None
    try:
        required = json.loads(raw.decode("utf-8"), object_pairs_hook=_unique)
    except (ValueError, UnicodeDecodeError) as error:
        fail(scope, "tree.required", "invalid JSON: %s" % error)
        return None
    if not isinstance(required, dict):
        fail(scope, "tree.required", "top level is not an object")
        return None
    if required.get("schema") != REQUIRED_SCHEMA:
        fail(scope, "tree.required.schema", "must be %s" % REQUIRED_SCHEMA)
        return None
    if sha256_bytes(canonical_bytes(required)) != nested.get("required_sha256"):
        fail(scope, "nested.required_sha256",
             "must be the canonical digest of required.json")
        return None
    scenarios = required.get("scenarios")
    if not isinstance(scenarios, list):
        fail(scope, "tree.required.scenarios", "must be a list")
        return None
    ids = [scenario.get("id") if isinstance(scenario, dict) else None
           for scenario in scenarios]
    if ids != required_ids:
        fail(scope, "tree.required.scenarios",
             "scenario ids must equal required_ids in order")
        return None
    return required


def check_scenario_files(scope, roots, nested, scenarios):
    # type: (...) -> None
    """Verify per-scenario files exist under both roots with provenance.

    Mirrors `compare.rs::compare_scenario`/`check_provenance` for the
    structural contract. Frame/state byte equality stays the comparator's
    verdict and is not asserted here.
    """
    oracle_commit = nested.get("oracle_commit")
    candidate_tree = nested.get("candidate_source_tree")
    for scenario in scenarios:
        if not isinstance(scenario, dict):
            fail(scope, "tree.scenario", "entries must be objects")
            return
        scenario_id = scenario.get("id")
        refs = {}
        for key in ("frame", "state", "provenance"):
            ref = scenario.get(key)
            if not is_safe_relative_path(ref):
                fail(scope, "tree.scenario.%s" % key,
                     "must be a safe relative path for %r" % (scenario_id,))
                refs = {}
                break
            refs[key] = ref
        if not refs:
            continue
        for key in ("lane", "checkpoint"):
            if not isinstance(scenario.get(key), str) \
                    or not scenario[key]:
                fail(scope, "tree.scenario.%s" % key,
                     "must be a non-empty string for %r" % (scenario_id,))
                refs = {}
                break
        if not refs:
            continue
        if not isinstance(scenario.get("state_keys"), list):
            fail(scope, "tree.scenario.state_keys",
                 "must be a list for %r" % (scenario_id,))
            continue
        for side in ("oracle", "candidate"):
            root = roots["%s_root" % side]
            for key in ("frame", "state", "provenance"):
                if check_regular_single_link(
                        scope, root / refs[key],
                        "tree.scenario.%s" % key) is None:
                    refs = {}
                    break
            if refs:
                check_provenance(scope, root, refs["provenance"], nested,
                                 scenario, side, oracle_commit,
                                 candidate_tree)


def check_provenance(scope, root, ref, nested, scenario, side,
                     oracle_commit, candidate_tree):
    # type: (...) -> None
    raw = check_regular_single_link(scope, root / ref, "tree.provenance")
    if raw is None:
        return
    try:
        provenance = json.loads(raw.decode("utf-8"),
                                object_pairs_hook=_unique)
    except (ValueError, UnicodeDecodeError) as error:
        fail(scope, "tree.provenance", "invalid JSON: %s" % error)
        return
    if not isinstance(provenance, dict):
        fail(scope, "tree.provenance", "top level is not an object")
        return
    if provenance.get("schema") != PROVENANCE_SCHEMA:
        fail(scope, "tree.provenance.schema",
             "must be %s" % PROVENANCE_SCHEMA)
        return
    expected_source = oracle_commit if side == "oracle" else candidate_tree
    want = (("source_tree", expected_source),
            ("scenario_id", scenario.get("id")),
            ("lane", scenario.get("lane")),
            ("checkpoint", scenario.get("checkpoint")),
            ("binary_path", "binary.bin"),
            ("actions_sha256", nested.get("actions_sha256")),
            ("tool_sha256", nested.get("tool_sha256")),
            ("adapter_sha256", nested.get("%s_adapter_sha256" % side)))
    for field, value in want:
        if provenance.get(field) != value:
            fail(scope, "tree.provenance.%s" % field,
                 "must agree with the bound %s context" % side)
    if side == "candidate":
        if provenance.get("run_id") != nested.get("run_id"):
            fail(scope, "tree.provenance.run_id", "must match the context")
        if provenance.get("task_id") != TASK_ID:
            fail(scope, "tree.provenance.task_id",
                 "must match %s" % TASK_ID)
    binary = check_regular_single_link(scope, root / "binary.bin",
                                       "tree.binary")
    if binary is not None \
            and provenance.get("binary_sha256") != sha256_bytes(binary):
        fail(scope, "tree.provenance.binary_sha256",
             "must be the binary.bin bytes digest")


def expected_artifact_paths(required):
    # type: (dict) -> set
    """Mirror of `compare.rs::expected_artifact_paths`."""
    paths = {"binary.bin"}
    scenarios = required.get("scenarios")
    if isinstance(scenarios, list):
        for scenario in scenarios:
            if not isinstance(scenario, dict):
                continue
            for key in ("frame", "state", "provenance"):
                ref = scenario.get(key)
                if isinstance(ref, str):
                    paths.add(ref)
    return paths


def check_compare_roots(args, run_dir, context, qualification):
    # type: (...) -> None
    scope = args.check_id
    comparator = qualification.get("comparator")
    if not isinstance(comparator, dict):
        fail(scope, "qualification.comparator", "must be an object")
        return
    if comparator.get("schema") != COMPARE_CONTEXT_SCHEMA:
        fail(scope, "qualification.comparator.schema",
             "must be %s" % COMPARE_CONTEXT_SCHEMA)
    bound_runtime = (qualification.get("common") or {}).get("outputs") or {}
    bound_runtime = bound_runtime.get("runtime")
    expected_report = None
    if isinstance(bound_runtime, str) and Path(bound_runtime).is_dir():
        expected_report = str(Path(bound_runtime)
                              / ("%s.compare.json" % args.check_id))
    bound_report = comparator.get("report_path")
    if expected_report is None or not isinstance(bound_report, str) \
            or Path(bound_report).resolve() != Path(expected_report).resolve():
        fail(scope, "qualification.comparator.report_path",
             "must be the host-bound %s.compare.json report" % args.check_id)
        expected_report = (bound_report if isinstance(bound_report, str)
                           else str(run_dir))
    nested = comparator.get("context")
    if not isinstance(nested, dict):
        fail(scope, "qualification.comparator.context", "must be an object")
        return
    if nested.get("schema") != COMPARE_CONTEXT_SCHEMA:
        fail(scope, "nested.schema", "must be %s" % COMPARE_CONTEXT_SCHEMA)
    nested_run_id = nested.get("run_id")
    if not isinstance(nested_run_id, str) \
            or Path(nested_run_id).resolve() != run_dir.resolve():
        fail(scope, "nested.run_id", "must match the outer context")
    identity = (("task_id", TASK_ID), ("check_id", args.check_id),
                ("oracle_commit", context.get("oracle_commit")),
                ("candidate_source_tree", context.get("tree")))
    for field, want in identity:
        if nested.get(field) != want:
            fail(scope, "nested.%s" % field, "must match the outer context")
    nested_report = nested.get("report_path")
    if not isinstance(nested_report, str) \
            or Path(nested_report).resolve() != Path(expected_report).resolve():
        fail(scope, "nested.report_path", "must match the outer report")
    worktree = (qualification.get("common") or {}).get("worktree")
    worktree_path = Path(worktree).resolve() \
        if isinstance(worktree, str) and Path(worktree).is_dir() else None
    gitdir_path = None
    if worktree_path is not None:
        try:
            common_dir = git(str(worktree_path), "rev-parse",
                             "--git-common-dir")
        except RuntimeError as error:
            fail(scope, "common.worktree.gitdir", str(error))
        else:
            joined = Path(common_dir) \
                if Path(common_dir).is_absolute() \
                else worktree_path / common_dir
            try:
                gitdir_path = joined.resolve()
            except OSError as error:
                fail(scope, "common.worktree.gitdir",
                     "unresolvable: %s" % error)
    roots = {}
    for field in ("oracle_root", "candidate_root"):
        value = nested.get(field)
        path = Path(value) if isinstance(value, str) else None
        if path is None or not path.is_absolute():
            fail(scope, "nested.%s" % field, "must be an absolute path")
            continue
        if not path.is_dir() or path.is_symlink():
            fail(scope, "nested.%s" % field,
                 "must exist as a real directory")
            continue
        resolved = path.resolve()
        if worktree_path is not None and resolved == worktree_path:
            fail(scope, "nested.%s" % field,
                 "must not be the candidate worktree root")
            continue
        if gitdir_path is not None and resolved == gitdir_path:
            fail(scope, "nested.%s" % field,
                 "must not be the git object-store directory")
            continue
        offender = scan_tree_symlink_free(resolved)
        if offender is not None:
            fail(scope, "nested.%s" % field,
                 "tree must hold no symlink: %s" % offender)
            continue
        roots[field] = resolved
    if len(roots) == 2:
        if roots["oracle_root"] == roots["candidate_root"]:
            fail(scope, "nested.roots", "must be distinct directories")
        report = Path(expected_report)
        for field, root in sorted(roots.items()):
            if report == root or root in report.parents:
                fail(scope, "nested.%s" % field,
                     "must not contain the report path")
    for field in ("oracle_manifest_sha256", "candidate_manifest_sha256",
                  "required_sha256", "actions_sha256",
                  "oracle_adapter_sha256", "candidate_adapter_sha256"):
        check_hex(scope, nested.get(field), "nested.%s" % field, False)
    tool_sha = nested.get("tool_sha256")
    check_hex(scope, tool_sha, "nested.tool_sha256", False)
    tool = context.get("tool")
    if (isinstance(tool, dict) and isinstance(tool_sha, str)
            and tool_sha != tool.get("sha256")):
        fail(scope, "nested.tool_sha256",
             "must match the bound candidate tool")
    required_ids = nested.get("required_ids")
    if (not isinstance(required_ids, list) or not required_ids
            or any(not isinstance(item, str) or not item
                   for item in required_ids)
            or len(set(required_ids)) != len(required_ids)):
        fail(scope, "nested.required_ids",
             "must be a non-empty unique string list")
        required_ids = None
    if nested.get("required_count") != (
            len(required_ids) if isinstance(required_ids, list) else -1):
        fail(scope, "nested.required_count",
             "must equal len(required_ids)")
    if len(roots) == 2 and isinstance(required_ids, list):
        if (roots["candidate_root"] / "approved").exists() \
                or (roots["candidate_root"] / "approved").is_symlink():
            fail(scope, "tree.approved",
                 "candidate root must hold no approved entry")
        manifests = {}
        for field, digest_field in (
                ("oracle_root", "oracle_manifest_sha256"),
                ("candidate_root", "candidate_manifest_sha256")):
            manifests[field] = check_manifest_tree(
                scope, roots[field], nested.get(digest_field), required_ids)
        required = check_required_document(scope, roots["oracle_root"],
                                           nested, required_ids)
        if required is not None:
            expected_candidate = expected_artifact_paths(required)
            expected_oracle = set(expected_candidate)
            expected_oracle.update(("actions.json", "font.bin",
                                    "profile.json", "required.json"))
            for field, expected in (
                    ("oracle_root", expected_oracle),
                    ("candidate_root", expected_candidate)):
                manifest = manifests.get(field)
                if manifest is None:
                    continue
                listed = {entry["path"] for entry in manifest["files"]}
                if listed != expected:
                    fail(scope, "tree.manifest.files",
                         "%s file set must equal the required set" % field)
            scenarios = required.get("scenarios")
            if isinstance(scenarios, list):
                check_scenario_files(scope, roots, nested, scenarios)
    check_task_derived_members(args, context)


def check_account_inventory(args, context, qualification, task_dir):
    # type: (...) -> None
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
    # Derived preparation-register bindings (accounting demand).
    original = qualification.get("original")
    if not isinstance(original, dict):
        fail(scope, "qualification.original", "must be an object")
    else:
        check_hex(scope, original.get("source_sha256"),
                  "qualification.original.source_sha256", False)
        check_hex(scope, original.get("non_test_sha256"),
                  "qualification.original.non_test_sha256", False)
        if "assertions" not in original:
            fail(scope, "qualification.original.assertions",
                 "must be bound")
    required = qualification.get("required")
    if not isinstance(required, list) or not required:
        fail(scope, "qualification.required",
             "must be a non-empty object list")
        required = None
    else:
        seen = set()
        for item in required:
            if not isinstance(item, dict) or set(item) != INVENTORY_KEYS:
                fail(scope, "qualification.required",
                     "every entry must carry exactly %s"
                     % sorted(INVENTORY_KEYS))
                required = None
                break
            if any(not isinstance(item[key], str) or not item[key]
                   for key in INVENTORY_KEYS):
                fail(scope, "qualification.required",
                     "every identity field must be a non-empty string")
                required = None
                break
            if item["source_commit"] != context.get("oracle_commit"):
                fail(scope, "qualification.required.source_commit",
                     "must equal the bound oracle commit (source-derived)")
                required = None
                break
            key = sha256_bytes(canonical_bytes(item))
            if key in seen:
                fail(scope, "qualification.required",
                     "identities must be unique")
                required = None
                break
            seen.add(key)
    future = qualification.get("future", [])
    if not isinstance(future, list):
        fail(scope, "qualification.future", "must be a list")
    elif not isinstance(required, list):
        pass
    elif future != []:
        fail(scope, "qualification.future",
             "preparation mode must bind an empty future")
    elif qualification.get("preparation_register") != {"required": required,
                                                      "future": future}:
        fail(scope, "qualification.preparation_register",
             "must equal {required, future}")
    if qualification.get("accepted_inventory") or \
            qualification.get("accepted_disposition"):
        fail(scope, "qualification.receipts",
             "preparation mode must not claim accepted receipts")
    check_task_derived_members(args, context)


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


def run_supervision(args, run_dir, task_dir, context, context_raw,
                    qualification):
    # type: (...) -> None
    """Execute the source compare branch once and judge its supervision.

    Runs the exact candidate `runner/__main__.py` bytes in place via the
    trusted loader, with no `TC_PROOF_*` environment preset, then asserts
    the branch returned (attestation: no `execv`), emitted its bound runner
    result via `finish()`, and left the comparison report behind.
    """
    scope = args.check_id
    worktree = (qualification.get("common") or {}).get("worktree")
    if not isinstance(worktree, str) or not Path(worktree).is_dir():
        fail(scope, "supervision.worktree", "candidate worktree is unbound")
        return
    outputs = run_dir / "outputs"
    result_path = outputs / ("%s.result.json" % scope)
    sentinel_path = outputs / ("%s.supervision.json" % scope)
    for path, label in ((result_path, "result"), (sentinel_path, "sentinel")):
        if path.exists() or path.is_symlink():
            fail(scope, "supervision.%s" % label,
                 "pre-existing artifact cannot be attributed: %s" % path)
            return
    loader = task_dir / "trusted" / "bind-repair" / "compare-loader.py"
    if not check_host_file(scope, loader, "supervision loader"):
        return
    context_path = run_dir / "contexts" / ("%s.json" % scope)
    child_env = {key: value for key, value in os.environ.items()
                 if not key.startswith("TC_PROOF_")}
    command = [sys.executable, str(loader), "--worktree", worktree,
               "--context", str(context_path), "--sentinel",
               str(sentinel_path)]
    try:
        child = subprocess.run(
            command, cwd=worktree, env=child_env,
            stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            timeout=SUPERVISION_TIMEOUT_SECONDS, check=False)
    except subprocess.TimeoutExpired:
        fail(scope, "supervision.timeout",
             "branch did not return within %ds"
             % SUPERVISION_TIMEOUT_SECONDS)
        return
    except OSError as error:
        fail(scope, "supervision.spawn", "child failed: %s" % error)
        return
    if child.returncode != 0:
        tail = child.stderr.decode("utf-8", "replace")
        tail = tail[-SUPERVISION_STDERR_TAIL:]
        for line in tail.splitlines():
            print("CHILD %s stderr: %s" % (scope, line))
    loaded = load_strict(sentinel_path)
    if isinstance(loaded, Exception):
        fail(scope, "supervision.sentinel",
             "branch never returned (execv still in place?): %s" % loaded)
        return
    _, sentinel = loaded
    if not isinstance(sentinel, dict):
        fail(scope, "supervision.sentinel", "top level is not an object")
        return
    if sentinel.get("schema") != SUPERVISION_SCHEMA:
        fail(scope, "supervision.sentinel.schema",
             "must be %s" % SUPERVISION_SCHEMA)
    if sentinel.get("check_id") != scope:
        fail(scope, "supervision.sentinel.check_id", "must match")
    if sentinel.get("operation") != "compare":
        fail(scope, "supervision.sentinel.operation", "must be compare")
    if sentinel.get("returned") is not True:
        fail(scope, "supervision.sentinel.returned",
             "branch must return to its caller")
    expected_source = (Path(worktree) / "tools" / "refactor-proof"
                       / "runner" / "__main__.py")
    loaded_path = sentinel.get("loaded_path")
    if not isinstance(loaded_path, str):
        fail(scope, "supervision.sentinel.loaded_path",
             "must be the executed source path")
    else:
        try:
            same = Path(loaded_path).resolve() == expected_source.resolve()
        except OSError:
            same = False
        if not same:
            fail(scope, "supervision.sentinel.loaded_path",
                 "must be the candidate source branch: %s" % expected_source)
    if sentinel.get("exit_code") != child.returncode:
        fail(scope, "supervision.sentinel.exit_code",
             "must match the child exit status")
    loaded = load_strict(result_path)
    if isinstance(loaded, Exception):
        fail(scope, "supervision.result",
             "no runner result was emitted: %s" % loaded)
        return
    _, result = loaded
    if not isinstance(result, dict):
        fail(scope, "supervision.result", "top level is not an object")
        return
    if set(result) != RUNNER_RESULT_KEYS:
        fail(scope, "supervision.result.keys",
             "must be exactly %s" % sorted(RUNNER_RESULT_KEYS))
    if result.get("schema") != RUNNER_RESULT_SCHEMA:
        fail(scope, "supervision.result.schema",
             "must be %s" % RUNNER_RESULT_SCHEMA)
    if result.get("run_id") != context.get("run_id"):
        fail(scope, "supervision.result.run_id", "must match the context")
    if result.get("operation") != "compare":
        fail(scope, "supervision.result.operation", "must be compare")
    if result.get("context_sha256") != sha256_bytes(context_raw):
        fail(scope, "supervision.result.context_sha256",
             "must match the executed context bytes")
    status = result.get("status")
    if status not in ("passed", "rejected"):
        fail(scope, "supervision.result.status",
             "must be passed or rejected")
    elif (result.get("category") is None) != (status == "passed"):
        fail(scope, "supervision.result.category",
             "must be null exactly when passed")
    digests = result.get("observation_digests")
    if (not isinstance(digests, list) or not digests
            or any(not isinstance(item, str) or not HEX64.match(item)
                   for item in digests)):
        fail(scope, "supervision.result.observation_digests",
             "must be a non-empty 64-hex list")
    if not isinstance(result.get("outputs"), dict):
        fail(scope, "supervision.result.outputs", "must be an object")
    if status in ("passed", "rejected") \
            and (status == "passed") != (child.returncode == 0):
        fail(scope, "supervision.coherence",
             "status passed must match child exit 0")
    nested = (qualification.get("comparator") or {}).get("context")
    report_value = nested.get("report_path") \
        if isinstance(nested, dict) else None
    if not isinstance(report_value, str):
        fail(scope, "supervision.report", "nested report path is unbound")
        return
    loaded = load_strict(Path(report_value))
    if isinstance(loaded, Exception):
        fail(scope, "supervision.report",
             "comparison report is missing: %s" % loaded)
        return
    _, report = loaded
    if not isinstance(report, dict):
        fail(scope, "supervision.report", "top level is not an object")
        return
    if report.get("schema") != COMPARISON_SCHEMA:
        fail(scope, "supervision.report.schema",
             "must be %s" % COMPARISON_SCHEMA)
    if report.get("run_id") != context.get("run_id"):
        fail(scope, "supervision.report.run_id", "must match the context")
    if report.get("task_id") != TASK_ID:
        fail(scope, "supervision.report.task_id",
             "must match %s" % TASK_ID)
    if report.get("context_sha256") != sha256_bytes(context_raw):
        fail(scope, "supervision.report.context_sha256",
             "must match the executed context bytes")
    if report.get("required_count") != (nested.get("required_count")
                                       if isinstance(nested, dict) else None):
        fail(scope, "supervision.report.required_count",
             "must match the bound required count")
    if not isinstance(report.get("results"), list) \
            or not isinstance(report.get("failures"), list):
        fail(scope, "supervision.report",
             "results/failures must be lists")
    elif status == "passed" and report.get("failures") != []:
        fail(scope, "supervision.coherence",
             "passed status needs an empty report failure list")


GATE_PROFILES = {"CHK-002": "compare-roots",
                 "CHK-003": "account-inventory"}


PROFILE_OPERATIONS = {"compare-roots": "compare",
                      "account-inventory": "account-tests",
                      "compare-supervision": "compare"}


def manifest_supervise(check_id):
    # type: (str) -> bool | None
    """Read --supervise for one check from this package's verify.toml."""
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
        return re.search(r"--supervise(\s+|\s*$)", table) is not None
    return None


def run_profile(args, run_dir, task_dir, check_id, profile):
    # type: (...) -> bool
    """Run common + profile assertions for one prepared context."""
    before = len(FAILURES)
    operation = PROFILE_OPERATIONS[profile]
    sub = argparse.Namespace(check_id=check_id, operation=operation,
                             dispatcher=args.dispatcher)
    context_path = run_dir / "contexts" / ("%s.json" % check_id)
    if not check_host_file(check_id, context_path, "context file"):
        return False
    loaded = load_strict(context_path)
    if isinstance(loaded, Exception):
        fail(check_id, "context", "unreadable: %s" % loaded)
        return False
    raw, context = loaded
    qualification = check_common(sub, run_dir, context, raw, context_path)
    if profile == "compare-roots":
        check_compare_roots(sub, run_dir, context, qualification)
    elif profile == "account-inventory":
        check_account_inventory(sub, context, qualification, task_dir)
    elif profile == "compare-supervision":
        check_compare_roots(sub, run_dir, context, qualification)
        run_supervision(sub, run_dir, task_dir, context, raw, qualification)
    return len(FAILURES) == before


def parse_args(argv):
    # type: (list) -> argparse.Namespace
    parser = argparse.ArgumentParser(
        description="Assert one prepared tc-proof context is genuine.")
    parser.add_argument("--check-id", required=True)
    parser.add_argument("--context", required=True)
    parser.add_argument("--supervise", action="store_true",
                        help="execute the source compare branch once")
    parser.add_argument("--gate", default=None,
                        help="comma-separated sibling checks to re-verify")
    parser.add_argument("dispatcher",
                        help="candidate dispatcher (identity cross-check)")
    parser.add_argument("operation", help="expected bound operation")
    return parser.parse_args(argv)


def main(argv):
    # type: (list) -> int
    args = parse_args(argv)
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
    if args.operation == "close" and args.gate:
        wanted = [item.strip() for item in args.gate.split(",") if item.strip()]
        if sorted(wanted) != sorted(GATE_PROFILES):
            fail(args.check_id, "--gate",
                 "must list exactly the two binding profiles")
        for check_id in wanted:
            if check_id in GATE_PROFILES:
                run_profile(args, run_dir_path, task_dir, check_id,
                            GATE_PROFILES[check_id])
    profile = None
    if args.operation == "compare":
        manifest_flag = manifest_supervise(args.check_id)
        if manifest_flag is None:
            fail(args.check_id, "verify.toml",
                 "manifest entry for %s unreadable" % args.check_id)
        elif args.supervise != manifest_flag:
            fail(args.check_id, "--supervise",
                 "CLI flag must match verify.toml")
        elif args.supervise and args.check_id == "CHK-004":
            profile = "compare-supervision"
        elif not args.supervise and args.check_id == "CHK-002":
            profile = "compare-roots"
        else:
            fail(args.check_id, "operation",
                 "compare checks are CHK-002 (roots) and CHK-004 (supervised)")
    elif args.operation == "account-tests":
        if args.supervise or args.check_id != "CHK-003":
            fail(args.check_id, "operation",
                 "account-tests checks are CHK-003 without --supervise")
        else:
            profile = "account-inventory"
    elif args.operation in ("preflight", "close"):
        if args.supervise:
            fail(args.check_id, "--supervise",
                 "only the CHK-004 compare check supervises")
        else:
            before = len(FAILURES)
            if check_host_file(args.check_id, context_path, "context file"):
                loaded = load_strict(context_path)
                if isinstance(loaded, Exception):
                    fail(args.check_id, "context",
                         "unreadable: %s" % loaded)
                else:
                    raw, context = loaded
                    check_common(args, run_dir_path, context, raw,
                                 context_path)
    else:
        fail(args.check_id, "operation", "unknown operation %r"
             % (args.operation,))
    if profile is not None:
        run_profile(args, run_dir_path, task_dir, args.check_id, profile)
    if FAILURES:
        for line in FAILURES:
            print(line)
        print("REPAIR-RESULT %s status=rejected failures=%d"
              % (args.check_id, len(FAILURES)))
        return 1
    print("REPAIR-RESULT %s status=passed" % args.check_id)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
