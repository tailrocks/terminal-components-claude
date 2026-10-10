#!/usr/bin/env python3
"""Validate and render accepted Termrock visibility work records.

The accepted task JSON is the authority. WORK_QUEUE.md is a deterministic view.
This module never treats a task state as evidence that product checks passed.
Instruction discovery here only builds a static task packet; it does not prove
that an agent runtime loaded the listed instructions. Review receipts bind the
recorded external file digest and expected subject in the source record; they do
not authenticate reviewer identity or provide a cryptographic signature.
"""

from __future__ import annotations

import argparse
import copy
import fcntl
import hashlib
import json
import os
import re
import secrets
import selectors
import signal
import stat
import subprocess
import sys
import tempfile
import time
from contextlib import contextmanager
from datetime import datetime, timezone
from pathlib import Path, PurePosixPath
from typing import Any, Iterator, Mapping, Sequence
from urllib.parse import urlsplit


ROOT = Path(__file__).resolve().parents[2]
TASKS_PATH = ROOT / "docs/implementation/visibility/tasks.json"
QUEUE_PATH = ROOT / "WORK_QUEUE.md"
PRIORITY = {"P0": 0, "P1": 1, "P2": 2, "P3": 3}
STATES = {"ready", "claimed", "in_progress", "review", "blocked", "verified"}
ACTIVE_STATES = {"claimed", "in_progress", "review", "blocked"}
SHA1 = re.compile(r"^[0-9a-f]{40}$")
SHA256 = re.compile(r"^[0-9a-f]{64}$")
FROZEN_VISUAL_TAG_OBJECT_SHA = "1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5"
FROZEN_VISUAL_TAG_COMMIT_SHA = "4a79c0a2d40fca46fc406b77157ce3b3f12ec16b"
WORK_ID = re.compile(r"^[A-Z][A-Z0-9]+-[0-9]{2,}$")
TOKEN = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]*$")
RFC3339_UTC = re.compile(
    r"^[0-9]{4}-(?:0[1-9]|1[0-2])-(?:0[1-9]|[12][0-9]|3[01])"
    r"T(?:[01][0-9]|2[0-3]):[0-5][0-9]:[0-5][0-9]"
    r"(?:\.[0-9]{1,6})?(?:Z|\+00:00)$"
)

RECORD_FIELDS = {
    "schema_version", "queue_revision", "accepted_by", "acceptance_mode", "accepted_at", "tasks",
}
TASK_REQUIRED_FIELDS = {
    "work_id", "requirement_ids", "priority", "priority_reason", "dependencies", "branch",
    "base_sha", "allowed_paths", "owner", "reviewer", "claim_token", "expiry", "state",
    "evidence", "handoff", "accepted_queue_revision",
}
TASK_OPTIONAL_FIELDS = {
    "shared_api_decisions", "claim_history", "review_subject_sha256", "review_record",
    "branch_scopes",
    "review_record_sha256",
}
HANDOFF_FIELDS = {
    "handoff_at", "handoff_by", "from_owner", "to_owner", "owner_released",
    "coordinator_verified_stopped", "stop_evidence", "worktree", "git_status", "diff_sha256",
    "changed_paths", "unpublished_commits", "checks", "next_action",
}
HANDOFF_REQUIRED_FIELDS = HANDOFF_FIELDS - {"stop_evidence"}
REVIEW_FIELDS = {"reviewer", "decision", "subject_sha256", "reviewed_at", "evidence"}
CLAIM_SNAPSHOT_FIELDS = {
    "owner", "reviewer", "branch", "base_sha", "allowed_paths", "claim_token", "expiry",
    "state", "accepted_queue_revision", "evidence",
}
CLAIM_SNAPSHOT_OPTIONAL_FIELDS = {"branch_scopes"}
REFERENCE_BRANCH = "visual-baseline"
REFERENCE_PACKAGE_SCOPE = "crates/termrock-e2e/**"
BRANCH_SCOPE_AMENDMENT_WORK_IDS = {"VIS-01", "VIS-02", "VIS-11"}
VIS01_REFERENCE_PATH_COUNT = 89
MAX_REFERENCE_PROBE_BYTES = 4096
REFERENCE_PROBE_TIMEOUT_SECONDS = 10.0
REFERENCE_REF_OUTPUT = re.compile(rb"([0-9a-f]{40})\trefs/heads/visual-baseline\n")
REFERENCE_SSH_ORIGIN = "git@github.com:tailrocks/terminal-components-claude.git"
REFERENCE_HTTPS_ORIGIN = "https://github.com/tailrocks/terminal-components-claude.git"
ROOT_REGISTRY_FIELDS = {"schema_version", "repositories", "worktrees", "visual_authority"}
ROOT_REGISTRY_V2_FIELDS = {
    "schema_version", "queue_authority", "repositories", "worktrees", "visual_authority",
}
QUEUE_AUTHORITY_FIELDS = {"repository_id", "worktree_id"}
REPOSITORY_FIELDS = {"origin_url"}
WORKTREE_FIELDS = {
    "repository_id", "root", "common_dir", "mode", "branch", "pinned_sha",
    "writable", "source_pin_id", "allowed_paths", "source_selection_review",
}
WORKTREE_V2_FIELDS = WORKTREE_FIELDS | {"role"}
VISUAL_AUTHORITY_FIELDS = {"tag", "tag_object_sha", "peeled_commit_sha"}
SOURCE_SELECTION_REVIEW_REF_FIELDS = {"review_record_path", "review_record_sha256"}
SOURCE_SELECTION_REVIEW_FIELDS = {
    "schema", "reviewer", "decision", "reviewed_at", "subject", "evidence",
}
SOURCE_SELECTION_SUBJECT_FIELDS = {
    "repository_id", "commit_sha", "tree_sha", "path_policy_sha256",
}
TASK_IDENTITY_FIELDS = {"repository_id", "worktree_id"}
CLAIM_SNAPSHOT_V2_FIELDS = CLAIM_SNAPSHOT_FIELDS | TASK_IDENTITY_FIELDS
IDENTITY_MAP_SCHEMA = "termrock-visibility-identity-map/v1"
MIGRATION_PREVIEW_SCHEMA = "termrock-visibility-migration-preview/v1"
MIGRATION_BRANCH_SCOPE_ERROR = (
    "tasks[0].branch_scopes is a schema-v1 claim extension"
)
REGISTRY_ID = re.compile(r"^[a-z][a-z0-9-]*$")
WORKTREE_MODES = {"branch", "source-pin", "scratch"}
WORKTREE_ROLES = {
    "queue-coordinator", "candidate-source-authority", "reference-source",
    "immutable-source-pin", "writable-isolated-scratch",
}
REGISTRY_V2_LIVE_WORKTREES = {"queue-coordinator", "candidate-source", "reference"}
MAX_SOURCE_SELECTION_REVIEW_BYTES = 64 * 1024


class QueueError(ValueError):
    """An invalid or stale accepted-work record."""


def require(condition: bool, message: str) -> None:
    if not condition:
        raise QueueError(message)


def _exact_keys(
    value: Mapping[str, Any], required: set[str], optional: set[str], label: str,
) -> None:
    require(all(isinstance(key, str) for key in value), "{} keys must be text".format(label))
    missing = required - set(value)
    unknown = set(value) - required - optional
    require(not missing, "{} is missing fields: {}".format(label, ", ".join(sorted(missing))))
    require(not unknown, "{} has unknown fields: {}".format(label, ", ".join(sorted(unknown))))


def _unique_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        require(key not in result, "JSON object repeats key {!r}".format(key))
        result[key] = value
    return result


def _reject_constant(value: str) -> None:
    raise QueueError("JSON contains unsupported non-finite number {}".format(value))


def strict_json_loads(value: str | bytes) -> Any:
    try:
        return json.loads(value, object_pairs_hook=_unique_object, parse_constant=_reject_constant)
    except QueueError:
        raise
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise QueueError("input is not valid UTF-8 JSON") from error


def parse_time(value: Any, label: str) -> datetime:
    require(isinstance(value, str) and bool(value), "{} must be a timestamp".format(label))
    require(RFC3339_UTC.fullmatch(value) is not None,
            "{} must be a canonical RFC3339 UTC timestamp".format(label))
    normalized = value[:-1] + "+00:00" if value.endswith("Z") else value
    try:
        parsed = datetime.fromisoformat(normalized)
    except ValueError as error:
        raise QueueError("{} must be a valid RFC3339 calendar timestamp".format(label)) from error
    require(parsed.tzinfo is not None, "{} must include a timezone".format(label))
    require(parsed.utcoffset() == timezone.utc.utcoffset(parsed), "{} must be UTC".format(label))
    return parsed.astimezone(timezone.utc)


def _text(value: Any, label: str) -> str:
    require(isinstance(value, str) and bool(value.strip()), "{} must be nonempty text".format(label))
    return value


def normalize_scope(value: Any) -> str:
    """Accept exact paths and recursive directory scopes ending in ``/**``."""
    require(isinstance(value, str) and bool(value), "allowed path must be nonempty text")
    require("\\" not in value and not value.startswith("/"), "allowed path must be repository-relative")
    require("\x00" not in value and not re.match(r"^[A-Za-z]:", value), "invalid allowed path")
    path = PurePosixPath(value)
    parts = path.parts
    require(bool(parts), "allowed path is empty")
    require(all(part not in {"", ".", ".."} for part in parts), "allowed path traverses its root")

    recursive = parts[-1] == "**"
    if recursive:
        require(len(parts) > 1, "root-wide ** claims are too broad")
        require(all(part not in {"*", "?"} and "[" not in part for part in parts[:-1]),
                "only exact directory prefixes may use /**")
    else:
        require(not any(any(char in part for char in "*?[") for part in parts),
                "only exact paths and directory /** scopes are supported")
    normalized = "/".join(parts)
    require(normalized == value, "allowed path is not normalized")
    return normalized


def _scope_parts(scope: str) -> tuple[tuple[str, ...], bool]:
    normalized = normalize_scope(scope)
    parts = PurePosixPath(normalized).parts
    if parts[-1] == "**":
        return tuple(parts[:-1]), True
    return tuple(parts), False


def scopes_overlap(left: str, right: str) -> bool:
    """Return whether two supported path scopes can name the same file."""
    left_parts, left_recursive = _scope_parts(left)
    right_parts, right_recursive = _scope_parts(right)
    if not left_recursive and not right_recursive:
        return left_parts == right_parts
    if left_recursive and right_recursive:
        short, long = sorted((left_parts, right_parts), key=len)
        return long[: len(short)] == short
    recursive_parts, exact_parts = (
        (left_parts, right_parts) if left_recursive else (right_parts, left_parts)
    )
    return exact_parts[: len(recursive_parts)] == recursive_parts


def _validate_handoff(value: Any, label: str) -> Mapping[str, Any]:
    require(isinstance(value, dict), "{} must be an object".format(label))
    _exact_keys(value, HANDOFF_REQUIRED_FIELDS, {"stop_evidence"}, label)
    for field in ("handoff_at", "handoff_by", "from_owner", "to_owner", "worktree",
                  "git_status", "next_action"):
        _text(value.get(field), "{}.{}".format(label, field))
    parse_time(value.get("handoff_at"), "{}.handoff_at".format(label))
    require(type(value.get("owner_released")) is bool
            and type(value.get("coordinator_verified_stopped")) is bool,
            "{} owner-release fields must be booleans".format(label))
    require(value.get("owner_released") is True or value.get("coordinator_verified_stopped") is True,
            "{} must record owner release or coordinator-verified stop".format(label))
    if value.get("coordinator_verified_stopped") is True:
        _text(value.get("stop_evidence"), "{}.stop_evidence".format(label))
    else:
        require("stop_evidence" not in value,
                "{}.stop_evidence is only valid for coordinator-verified stops".format(label))
    require(isinstance(value.get("diff_sha256"), str)
            and SHA256.fullmatch(value["diff_sha256"]) is not None,
            "{}.diff_sha256 must be a full lowercase SHA-256".format(label))
    for field in ("changed_paths", "unpublished_commits"):
        items = value.get(field)
        require(isinstance(items, list) and all(isinstance(item, str) for item in items),
                "{}.{} must be a text array".format(label, field))
    checks = value.get("checks")
    require(isinstance(checks, list), "{}.checks must be an array".format(label))
    for index, check in enumerate(checks):
        require(isinstance(check, dict), "{}.checks[{}] must be an object".format(label, index))
        _exact_keys(check, {"command", "result"}, set(), "{}.checks[{}]".format(label, index))
        _text(check.get("command"), "{}.checks[{}].command".format(label, index))
        _text(check.get("result"), "{}.checks[{}].result".format(label, index))
    return value


def _task_token(task: Mapping[str, Any]) -> str:
    return _text(task.get("claim_token"), "{}.claim_token".format(task.get("work_id", "task")))


def _validate_review(value: Any, reviewer: str, label: str) -> Mapping[str, Any]:
    require(isinstance(value, dict), "{} is required to verify a task".format(label))
    _exact_keys(value, REVIEW_FIELDS, set(), label)
    require(value.get("reviewer") == reviewer,
            "{} must identify the assigned independent reviewer".format(label))
    require(value.get("decision") == "approved", "{} must record an approved review".format(label))
    require(isinstance(value.get("subject_sha256"), str)
            and SHA256.fullmatch(value["subject_sha256"]) is not None,
            "{}.subject_sha256 must be a full lowercase SHA-256".format(label))
    parse_time(value.get("reviewed_at"), "{}.reviewed_at".format(label))
    _text(value.get("evidence"), "{}.evidence".format(label))
    return value


def _task_scopes(task: Mapping[str, Any]) -> list[str]:
    raw = task.get("allowed_paths")
    require(isinstance(raw, list) and bool(raw), "{}.allowed_paths must be a nonempty array".format(task.get("work_id")))
    scopes = [normalize_scope(path) for path in raw]
    require(len(scopes) == len(set(scopes)), "{} repeats an allowed path".format(task.get("work_id")))
    for index, scope in enumerate(scopes):
        for other in scopes[index + 1 :]:
            require(not scopes_overlap(scope, other),
                    "{} contains overlapping allowed paths {} and {}".format(task.get("work_id"), scope, other))
    return scopes


def _validated_branch_scopes(value: Any, label: str, work_id: str) -> list[dict[str, Any]]:
    require(isinstance(value, list), "{}.branch_scopes must be an array".format(label))
    scopes: list[dict[str, Any]] = []
    branches: set[str] = set()
    for index, item in enumerate(value):
        item_label = "{}.branch_scopes[{}]".format(label, index)
        require(isinstance(item, dict), "{} must be an object".format(item_label))
        _exact_keys(item, {"branch", "base_sha", "allowed_paths"}, set(), item_label)
        branch = item.get("branch")
        require(branch == REFERENCE_BRANCH,
                "{}.branch must be the authorized {} branch".format(item_label, REFERENCE_BRANCH))
        require(branch not in branches, "{}.branch_scopes repeats {}".format(label, branch))
        branches.add(branch)
        base_sha = item.get("base_sha")
        require(isinstance(base_sha, str) and SHA1.fullmatch(base_sha) is not None,
                "{}.base_sha must be a full lowercase SHA-1".format(item_label))
        allowed_paths = item.get("allowed_paths")
        require(isinstance(allowed_paths, list) and bool(allowed_paths),
                "{}.allowed_paths must be a nonempty array".format(item_label))
        normalized = [normalize_scope(path) for path in allowed_paths]
        if work_id == "VIS-02":
            require(normalized == [REFERENCE_PACKAGE_SCOPE],
                    "{}.allowed_paths must contain only {}".format(item_label, REFERENCE_PACKAGE_SCOPE))
        else:
            require(work_id in {"VIS-01", "VIS-11"},
                    "{}.branch_scopes are not enabled for {}".format(item_label, work_id))
            require(all(not scope.endswith("/**") for scope in normalized),
                    "{}.allowed_paths must be literal file paths for {}".format(item_label, work_id))
            require(len(normalized) == len(set(normalized)),
                    "{}.allowed_paths repeats a literal path".format(item_label))
            for path_index, scope in enumerate(normalized):
                for other in normalized[path_index + 1 :]:
                    require(not scopes_overlap(scope, other),
                            "{}.allowed_paths contains overlapping literal paths".format(item_label))
            if work_id == "VIS-01":
                require(len(normalized) == VIS01_REFERENCE_PATH_COUNT,
                        "VIS-01 reference grant must contain exactly 89 literal paths")
        scopes.append({
            "branch": branch,
            "base_sha": base_sha,
            "allowed_paths": normalized,
        })
    return scopes


def _task_branch_grants(task: Mapping[str, Any]) -> list[tuple[str, list[str]]]:
    grants = [(task["branch"], _task_scopes(task))]
    for scope in _validated_branch_scopes(task.get("branch_scopes", []), task["work_id"], task["work_id"]):
        grants.append((scope["branch"], scope["allowed_paths"]))
    return grants


def _claim_snapshot(task: Mapping[str, Any]) -> dict[str, Any]:
    fields = (
        "owner", "reviewer", "branch", "base_sha", "allowed_paths", "claim_token",
        "expiry", "state", "accepted_queue_revision", "evidence",
    )
    snapshot = {field: copy.deepcopy(task.get(field)) for field in fields}
    if "branch_scopes" in task:
        snapshot["branch_scopes"] = copy.deepcopy(task["branch_scopes"])
    return snapshot


def _validate_records(records: Any, schema_version: int) -> Mapping[str, Any]:
    require(isinstance(records, dict) and type(records.get("schema_version")) is int
            and records.get("schema_version") == schema_version,
            "unsupported accepted-task schema")
    _exact_keys(records, RECORD_FIELDS, set(), "accepted task records")
    revision = records.get("queue_revision")
    require(type(revision) is int and revision > 0, "queue_revision must be a positive integer")
    _text(records.get("accepted_by"), "accepted_by")
    _text(records.get("acceptance_mode"), "acceptance_mode")
    parse_time(records.get("accepted_at"), "accepted_at")
    tasks = records.get("tasks")
    require(isinstance(tasks, list) and bool(tasks), "tasks must be a nonempty array")

    by_id: dict[str, Mapping[str, Any]] = {}
    all_tokens: set[str] = set()
    api_owners: dict[str, str] = {}
    for index, task in enumerate(tasks):
        label = "tasks[{}]".format(index)
        require(isinstance(task, dict), "{} must be an object".format(label))
        required_task_fields = TASK_REQUIRED_FIELDS
        if schema_version == 2:
            required_task_fields = TASK_REQUIRED_FIELDS | TASK_IDENTITY_FIELDS
        _exact_keys(task, required_task_fields, TASK_OPTIONAL_FIELDS, label)
        if schema_version == 2:
            require("branch_scopes" not in task,
                    "{}.branch_scopes is a schema-v1 claim extension".format(label))
        work_id = task.get("work_id")
        require(isinstance(work_id, str) and WORK_ID.fullmatch(work_id) is not None,
                "{}.work_id is invalid".format(label))
        require(work_id not in by_id, "duplicate work ID {}".format(work_id))
        by_id[work_id] = task
        reqs = task.get("requirement_ids")
        require(isinstance(reqs, list) and bool(reqs)
                and all(isinstance(item, str) and item for item in reqs),
                "{}.requirement_ids must be a nonempty text array".format(work_id))
        require(len(reqs) == len(set(reqs)), "{} repeats a requirement ID".format(work_id))
        require(isinstance(task.get("priority"), str) and task["priority"] in PRIORITY,
                "{} has an unknown priority".format(work_id))
        _text(task.get("priority_reason"), "{}.priority_reason".format(work_id))
        deps = task.get("dependencies")
        require(isinstance(deps, list) and all(isinstance(dep, str) for dep in deps),
                "{}.dependencies must be a text array".format(work_id))
        require(work_id not in deps and len(deps) == len(set(deps)),
                "{} has a self or duplicate dependency".format(work_id))
        if schema_version == 1:
            branch = task.get("branch")
            require(isinstance(branch, str) and branch in {"termrock-implementation", REFERENCE_BRANCH},
                    "{}.branch is unsupported for schema v1".format(work_id))
        else:
            branch = task.get("branch")
            require(branch is None or (isinstance(branch, str) and bool(branch.strip())
                                       and not any(char in branch for char in "\x00\r\n")),
                    "{}.branch must be nonempty text or null for a detached worktree".format(work_id))
            for field in sorted(TASK_IDENTITY_FIELDS):
                identity = task.get(field)
                require(isinstance(identity, str) and REGISTRY_ID.fullmatch(identity) is not None,
                        "{}.{} must be a valid registry ID".format(work_id, field))
        require(isinstance(task.get("base_sha"), str) and SHA1.fullmatch(task["base_sha"]) is not None,
                "{}.base_sha must be a full lowercase SHA-1".format(work_id))
        primary_scopes = _task_scopes(task)
        if schema_version == 1 and task["branch"] == REFERENCE_BRANCH:
            require(primary_scopes == [REFERENCE_PACKAGE_SCOPE],
                    "{} visual-baseline primary grant is limited to {}".format(
                        work_id, REFERENCE_PACKAGE_SCOPE
                    ))
        branch_scopes = _validated_branch_scopes(task.get("branch_scopes", []), work_id, work_id)
        require(task["branch"] not in {scope["branch"] for scope in branch_scopes},
                "{} repeats its primary branch in branch_scopes".format(work_id))
        owner = _text(task.get("owner"), "{}.owner".format(work_id))
        reviewer = _text(task.get("reviewer"), "{}.reviewer".format(work_id))
        require(owner != reviewer, "{} owner and independent reviewer must differ".format(work_id))
        token = _task_token(task)
        require(TOKEN.fullmatch(token) is not None, "{} has an invalid claim token".format(work_id))
        require(token not in all_tokens, "duplicate claim token {}".format(token))
        all_tokens.add(token)
        parse_time(task.get("expiry"), "{}.expiry".format(work_id))
        state = task.get("state")
        require(isinstance(state, str) and state in STATES,
                "{} has an unknown task state".format(work_id))
        evidence = task.get("evidence")
        require(isinstance(evidence, list) and bool(evidence)
                and all(isinstance(item, str) and item.strip() for item in evidence),
                "{}.evidence must be a nonempty text array".format(work_id))
        accepted_revision = task.get("accepted_queue_revision")
        require(type(accepted_revision) is int and 1 <= accepted_revision <= revision,
                "{}.accepted_queue_revision is invalid".format(work_id))
        if task.get("handoff") is not None:
            current_handoff = _validate_handoff(task["handoff"], "{}.handoff".format(work_id))
            require(current_handoff["to_owner"] == owner,
                    "{}.handoff target does not match current owner".format(work_id))
        review_subject = task.get("review_subject_sha256")
        if review_subject is not None:
            require(isinstance(review_subject, str) and SHA256.fullmatch(review_subject) is not None,
                    "{}.review_subject_sha256 must be a full lowercase SHA-256".format(work_id))
        if schema_version == 1 and branch_scopes:
            require(state != "verified",
                    "{} cannot be verified by a candidate-only review subject while branch scopes are active".format(
                        work_id
                    ))
            require(review_subject is None,
                    "{}.review_subject_sha256 cannot bind additional branch scopes".format(work_id))
        if state == "verified":
            require(review_subject is not None,
                    "{}.review_subject_sha256 must be in accepted source before verification".format(work_id))
            _validate_review(task.get("review_record"), reviewer, "{}.review_record".format(work_id))
            require(isinstance(task.get("review_record_sha256"), str)
                    and SHA256.fullmatch(task["review_record_sha256"]) is not None,
                    "{}.review_record_sha256 must be a full lowercase SHA-256".format(work_id))
            require(task["review_record"]["subject_sha256"] == review_subject,
                    "{}.review subject does not match accepted subject binding".format(work_id))
        else:
            require("review_record" not in task and "review_record_sha256" not in task,
                    "{}.review receipt is only valid for verified state".format(work_id))
        history = task.get("claim_history", [])
        require(isinstance(history, list), "{}.claim_history must be an array".format(work_id))
        for history_index, item in enumerate(history):
            history_label = "{}.claim_history[{}]".format(work_id, history_index)
            require(isinstance(item, dict) and isinstance(item.get("claim"), dict),
                    "{} must contain a prior claim snapshot".format(history_label))
            _exact_keys(item, {"claim", "handoff"}, set(), history_label)
            prior = item["claim"]
            expected_snapshot_fields = CLAIM_SNAPSHOT_FIELDS
            if schema_version == 2:
                expected_snapshot_fields = CLAIM_SNAPSHOT_V2_FIELDS
            expected_snapshot_keys = {frozenset(expected_snapshot_fields)}
            if schema_version == 1:
                expected_snapshot_keys.add(frozenset(expected_snapshot_fields | CLAIM_SNAPSHOT_OPTIONAL_FIELDS))
            require(frozenset(prior) in expected_snapshot_keys,
                    "{} claim snapshot has missing or unknown fields".format(history_label))
            if schema_version == 1 and "branch_scopes" in prior:
                prior_branch_scopes = _validated_branch_scopes(prior["branch_scopes"], history_label, work_id)
                require(prior.get("branch") not in {scope["branch"] for scope in prior_branch_scopes},
                        "{} repeats its primary branch in branch_scopes".format(history_label))
            if schema_version == 2:
                for field in sorted(TASK_IDENTITY_FIELDS):
                    identity = prior.get(field)
                    require(isinstance(identity, str) and REGISTRY_ID.fullmatch(identity) is not None,
                            "{}.{} must be a valid registry ID".format(history_label, field))
            prior_owner = _text(prior.get("owner"), "{}.owner".format(history_label))
            prior_reviewer = _text(prior.get("reviewer"), "{}.reviewer".format(history_label))
            require(prior_owner != prior_reviewer,
                    "{} prior owner and reviewer must differ".format(history_label))
            require(prior.get("branch") == task["branch"]
                    and prior.get("base_sha") == task["base_sha"],
                    "{} changed branch or base during handoff".format(history_label))
            prior_paths = prior.get("allowed_paths")
            require(isinstance(prior_paths, list) and
                    [normalize_scope(path) for path in prior_paths] == _task_scopes(task),
                    "{} changed allowed paths during handoff".format(history_label))
            prior_token = _text(prior.get("claim_token"), "{}.claim_token".format(history_label))
            require(TOKEN.fullmatch(prior_token) is not None and prior_token not in all_tokens,
                    "{} has a duplicate or invalid prior claim token".format(history_label))
            all_tokens.add(prior_token)
            parse_time(prior.get("expiry"), "{}.expiry".format(history_label))
            require(isinstance(prior.get("state"), str) and prior["state"] in ACTIVE_STATES,
                    "{} prior claim was not active".format(history_label))
            prior_revision = prior.get("accepted_queue_revision")
            require(type(prior_revision) is int and 1 <= prior_revision <= revision,
                    "{}.accepted_queue_revision is invalid".format(history_label))
            require(isinstance(prior.get("evidence"), list) and bool(prior["evidence"])
                    and all(isinstance(item, str) and item.strip() for item in prior["evidence"]),
                    "{}.evidence must be a nonempty text array".format(history_label))
            prior_handoff = _validate_handoff(item.get("handoff"), "{}.handoff".format(history_label))
            require(prior_handoff["from_owner"] == prior_owner,
                    "{} handoff source does not match prior owner".format(history_label))
            if history_index + 1 < len(history):
                next_entry = history[history_index + 1]
                require(isinstance(next_entry, dict) and isinstance(next_entry.get("claim"), dict),
                        "{} next claim snapshot is invalid".format(history_label))
                next_owner = _text(next_entry["claim"].get("owner"), "{}.next.owner".format(history_label))
            else:
                next_owner = owner
            require(prior_handoff["to_owner"] == next_owner,
                    "{} handoff target does not match next claim owner".format(history_label))
        api_keys = task.get("shared_api_decisions", [])
        require(isinstance(api_keys, list) and all(isinstance(key, str) and key for key in api_keys),
                "{}.shared_api_decisions must be a text array".format(work_id))
        require(len(api_keys) == len(set(api_keys)), "{} repeats a shared API decision".format(work_id))
        for key in api_keys:
            prior_owner = api_owners.setdefault(key, owner)
            require(prior_owner == owner,
                    "shared API decision {} has competing owners {} and {}".format(key, prior_owner, owner))

    for task in tasks:
        for dependency in task["dependencies"]:
            require(dependency in by_id, "{} depends on unknown work ID {}".format(task["work_id"], dependency))
        unresolved = [dependency for dependency in task["dependencies"]
                      if by_id[dependency]["state"] != "verified"]
        if unresolved:
            require(task["state"] in {"blocked", "ready"},
                    "{} must remain blocked until dependencies are verified: {}".format(
                        task["work_id"], ", ".join(unresolved)
                    ))

    visiting: set[str] = set()
    visited: set[str] = set()

    def visit(work_id: str) -> None:
        if work_id in visited:
            return
        require(work_id not in visiting, "dependency cycle includes {}".format(work_id))
        visiting.add(work_id)
        for dependency in by_id[work_id]["dependencies"]:
            visit(dependency)
        visiting.remove(work_id)
        visited.add(work_id)

    for work_id in by_id:
        visit(work_id)

    active = [task for task in tasks if task["state"] in ACTIVE_STATES]
    for index, task in enumerate(active):
        for other in active[index + 1 :]:
            if schema_version == 2 and task["repository_id"] != other["repository_id"]:
                continue
            task_grants = _task_branch_grants(task) if schema_version == 1 else [
                (task.get("branch") or "", task["allowed_paths"])
            ]
            other_grants = _task_branch_grants(other) if schema_version == 1 else [
                (other.get("branch") or "", other["allowed_paths"])
            ]
            for branch, scopes in task_grants:
                for other_branch, other_scopes in other_grants:
                    if schema_version == 1 and branch != other_branch:
                        continue
                    for scope in scopes:
                        for other_scope in other_scopes:
                            require(not scopes_overlap(scope, other_scope),
                                    "active path conflict: {} {} ({}) overlaps {} {} ({})".format(
                                        task["work_id"], branch, scope,
                                        other["work_id"], other_branch, other_scope,
                                    ))
    return records


def validate_records(records: Any) -> Mapping[str, Any]:
    """Validate the live schema-v1 accepted queue without changing its rules."""
    return _validate_records(records, 1)


def validate_records_v2(records: Any) -> Mapping[str, Any]:
    """Validate read-only schema-v2 records with repository-aware conflicts."""
    return _validate_records(records, 2)


def highest_open_priority(records: Mapping[str, Any]) -> str | None:
    open_tasks = [task for task in records["tasks"] if task["state"] != "verified"]
    if not open_tasks:
        return None
    return min((task["priority"] for task in open_tasks), key=PRIORITY.__getitem__)


def _ensure_revision(records: Mapping[str, Any], expected_revision: Any) -> None:
    require(type(expected_revision) is int and expected_revision == records["queue_revision"],
            "stale queue revision: expected {}, current {}".format(expected_revision, records["queue_revision"]))


def _now(value: datetime | None = None) -> datetime:
    current = value or datetime.now(timezone.utc)
    require(current.tzinfo is not None, "current time must include a timezone")
    return current.astimezone(timezone.utc)


def accept_record(
    records_value: Any,
    proposed_value: Any,
    expected_revision: int,
    *,
    current_branch: str,
    current_head: str,
    now: datetime | None = None,
) -> dict[str, Any]:
    records = copy.deepcopy(validate_records(records_value))
    _ensure_revision(records, expected_revision)
    require(isinstance(proposed_value, dict), "claim record must be an object")
    proposed = copy.deepcopy(proposed_value)
    require("review_subject_sha256" not in proposed,
            "new claim cannot prebind a review subject; bind after entering review")
    require("branch_scopes" not in proposed,
            "new claim cannot preseed an additional branch scope; use amend-branch-scope")
    require("accepted_queue_revision" not in proposed and "claim_history" not in proposed,
            "new claim cannot supply acceptance revision or prior history")
    work_id = proposed.get("work_id")
    require(isinstance(work_id, str), "claim work_id must be text")
    require(work_id not in {task["work_id"] for task in records["tasks"]},
            "work ID is already present")
    require(current_branch == "termrock-implementation", "acceptance is only allowed on termrock-implementation")
    require(proposed.get("branch") == current_branch, "claim branch does not match current branch")
    require(current_head == proposed.get("base_sha") and SHA1.fullmatch(current_head or "") is not None,
            "claim base SHA does not match current HEAD")
    proposed_state = proposed.get("state")
    require(isinstance(proposed_state, str) and proposed_state in {"claimed", "blocked"},
            "newly accepted work must start claimed or blocked")
    require(proposed.get("handoff") is None, "new claim cannot carry a handoff")
    current_time = _now(now)
    expiry = parse_time(proposed.get("expiry"), "claim.expiry")
    require(expiry > current_time, "new claim expiry must be in the future")
    highest = highest_open_priority(records)
    if highest is not None:
        require(proposed.get("priority") == highest,
                "cannot accept lower priority {} while {} remains open".format(proposed.get("priority"), highest))
    by_id = {task["work_id"]: task for task in records["tasks"]}
    dependencies = proposed.get("dependencies")
    require(isinstance(dependencies, list) and all(isinstance(dep, str) for dep in dependencies),
            "claim dependencies must be a text array")
    for dependency in dependencies:
        require(dependency in by_id, "claim depends on unknown work ID {}".format(dependency))
    unresolved = [dependency for dependency in dependencies if by_id[dependency]["state"] != "verified"]
    if unresolved:
        require(proposed["state"] == "blocked",
                "claim must start blocked until dependencies are verified: {}".format(
                    ", ".join(unresolved)
                ))
    records["queue_revision"] += 1
    proposed["accepted_queue_revision"] = records["queue_revision"]
    records["tasks"].append(proposed)
    validate_records(records)
    return records


TRANSITIONS = {
    "claimed": {"in_progress", "blocked"},
    "in_progress": {"review", "blocked"},
    "review": {"in_progress", "blocked", "verified"},
    "blocked": {"in_progress", "review"},
    "ready": set(),
    "verified": set(),
}


def transition_record(
    records_value: Any,
    work_id: str,
    new_state: str,
    evidence: str,
    expected_revision: int,
    *,
    review_record_raw: bytes | None = None,
    review_record_sha256: str | None = None,
    current_subject_sha256: str | None = None,
) -> dict[str, Any]:
    records = copy.deepcopy(validate_records(records_value))
    _ensure_revision(records, expected_revision)
    _text(evidence, "transition evidence")
    task = next((item for item in records["tasks"] if item["work_id"] == work_id), None)
    require(task is not None, "unknown work ID {}".format(work_id))
    require(isinstance(new_state, str) and new_state in TRANSITIONS[task["state"]],
            "invalid task state transition {} -> {}".format(task["state"], new_state))
    if new_state in {"in_progress", "review", "verified"}:
        by_id = {item["work_id"]: item for item in records["tasks"]}
        unresolved = [dependency for dependency in task["dependencies"]
                      if by_id[dependency]["state"] != "verified"]
        require(not unresolved,
                "cannot start or verify work until dependencies are verified: {}".format(
                    ", ".join(unresolved)
                ))
    if new_state == "verified":
        expected_subject = task.get("review_subject_sha256")
        require(isinstance(expected_subject, str) and SHA256.fullmatch(expected_subject) is not None,
                "{}.review_subject_sha256 must be recorded in accepted source before verification".format(work_id))
        require(isinstance(review_record_sha256, str)
                and SHA256.fullmatch(review_record_sha256) is not None,
                "review_record_sha256 must be a full lowercase SHA-256 supplied by the caller")
        require(isinstance(review_record_raw, bytes),
                "external review record bytes are required to verify a task")
        actual_review_sha256 = hashlib.sha256(review_record_raw).hexdigest()
        require(review_record_sha256 == actual_review_sha256,
                "external review record hash does not match the supplied raw-file SHA-256")
        review_record = strict_json_loads(review_record_raw)
        _validate_review(review_record, task["reviewer"], "review_record")
        require(review_record["subject_sha256"] == expected_subject,
                "review subject does not match accepted subject binding")
        require(current_subject_sha256 == expected_subject,
                "current allowed-path digest does not match accepted review subject")
        task["review_record"] = copy.deepcopy(review_record)
        task["review_record_sha256"] = actual_review_sha256
    elif review_record_raw is not None or review_record_sha256 is not None or current_subject_sha256 is not None:
        raise QueueError("review_record is only accepted when transitioning to verified")
    if task["state"] == "review" and new_state in {"in_progress", "blocked"}:
        archived_subject = task.pop("review_subject_sha256", None)
        if archived_subject is not None:
            task["evidence"].append(
                "archived and cleared review subject SHA-256 {} on review -> {}; rebind required".format(
                    archived_subject, new_state
                )
            )
    elif task["state"] in {"in_progress", "blocked"} and new_state == "review":
        archived_subject = task.pop("review_subject_sha256", None)
        if archived_subject is not None:
            task["evidence"].append(
                "archived and cleared legacy review subject SHA-256 {} on {} -> review; rebind required".format(
                    archived_subject, task["state"]
                )
            )
    task["state"] = new_state
    task["evidence"].append(evidence)
    records["queue_revision"] += 1
    validate_records(records)
    return records


def append_evidence_record(
    records_value: Any,
    work_id: str,
    claim_token: str,
    evidence: str,
    expected_revision: int,
) -> dict[str, Any]:
    records = copy.deepcopy(validate_records(records_value))
    _ensure_revision(records, expected_revision)
    _text(claim_token, "claim token")
    _text(evidence, "evidence")
    task = next((item for item in records["tasks"] if item["work_id"] == work_id), None)
    require(task is not None, "unknown work ID {}".format(work_id))
    require(task["state"] in ACTIVE_STATES,
            "evidence can only be appended to an active claim")
    require(_task_token(task) == claim_token,
            "claim token does not match the current task claim")
    highest = highest_open_priority(records)
    require(highest is not None and task["priority"] == highest,
            "cannot append evidence for lower priority {} while {} remains open".format(
                task["priority"], highest or "no priority"
            ))
    require(evidence not in task["evidence"], "evidence entry already exists")
    task["evidence"].append(evidence)
    records["queue_revision"] += 1
    validate_records(records)
    return records


def reviewer_handoff_record(
    records_value: Any,
    work_id: str,
    claim_token: str,
    expected_reviewer: str,
    new_reviewer: str,
    evidence: str,
    expected_revision: int,
) -> dict[str, Any]:
    records = copy.deepcopy(validate_records(records_value))
    _ensure_revision(records, expected_revision)
    _text(claim_token, "claim token")
    _text(expected_reviewer, "expected reviewer")
    _text(new_reviewer, "new reviewer")
    require(not any(character.isspace() or character == "\x00" for character in new_reviewer),
            "new reviewer must be a single-line identity")
    _text(evidence, "reviewer handoff evidence")
    task = next((item for item in records["tasks"] if item["work_id"] == work_id), None)
    require(task is not None, "unknown work ID {}".format(work_id))
    require(task["state"] in {"claimed", "in_progress", "blocked"},
            "reviewer handoff requires a claimed, in-progress, or blocked task")
    require(_task_token(task) == claim_token,
            "claim token does not match the current task claim")
    require(task["reviewer"] == expected_reviewer,
            "assigned reviewer does not match expected reviewer")
    require(new_reviewer != expected_reviewer,
            "new reviewer must differ from assigned reviewer")
    require(new_reviewer != task["owner"],
            "new reviewer must differ from current owner")
    require(task.get("review_subject_sha256") is None
            and task.get("review_record") is None
            and task.get("review_record_sha256") is None,
            "reviewer handoff cannot change an active review subject")
    prior_reason = "; " + evidence
    require(evidence not in task["evidence"] and not any(
        item.startswith("reviewer handoff at queue revision ") and item.endswith(prior_reason)
        for item in task["evidence"]
    ), "reviewer handoff evidence entry already exists")

    next_revision = records["queue_revision"] + 1
    task["reviewer"] = new_reviewer
    audit_entry = "reviewer handoff at queue revision {}: {} -> {}; {}".format(
        next_revision, expected_reviewer, new_reviewer, evidence
    )
    return append_evidence_record(
        records, work_id, claim_token, audit_entry, expected_revision
    )


def bind_review_subject_record(
    records_value: Any,
    work_id: str,
    claim_token: str,
    expected_subject_sha256: str,
    expected_revision: int,
    *,
    current_subject_sha256: str,
) -> dict[str, Any]:
    records = copy.deepcopy(validate_records(records_value))
    _ensure_revision(records, expected_revision)
    _text(claim_token, "claim token")
    require(isinstance(expected_subject_sha256, str)
            and SHA256.fullmatch(expected_subject_sha256) is not None,
            "expected subject SHA-256 must be a full lowercase SHA-256")
    require(isinstance(current_subject_sha256, str)
            and SHA256.fullmatch(current_subject_sha256) is not None,
            "current subject SHA-256 must be a full lowercase SHA-256")
    task = next((item for item in records["tasks"] if item["work_id"] == work_id), None)
    require(task is not None, "unknown work ID {}".format(work_id))
    require(task["state"] == "review",
            "review subject can only be bound while the task is in review")
    require(_task_token(task) == claim_token,
            "claim token does not match the current task claim")
    require(not task.get("branch_scopes"),
            "candidate-only review subject cannot bind an additional branch scope")
    require(task.get("review_subject_sha256") is None,
            "review subject is already bound; reopen or hand off before binding a new review round")
    require(current_subject_sha256 == expected_subject_sha256,
            "current allowed-path digest does not match expected review subject")
    task["review_subject_sha256"] = current_subject_sha256
    records["queue_revision"] += 1
    task["evidence"].append(
        "bound review subject SHA-256 {} at queue revision {}".format(
            current_subject_sha256, records["queue_revision"]
        )
    )
    validate_records(records)
    return records


def _accepted_branch_scope_plan(
    records: Mapping[str, Any], work_id: str, plan_sha256: str, review_sha256: str,
) -> dict[str, Any] | None:
    # Accepted evidence pins local artifact bytes; it is queue policy data, not identity proof.
    plan_task = next((task for task in records["tasks"] if task["work_id"] == "VIS-10"), None)
    if plan_task is None:
        return None
    accepted: list[dict[str, Any]] = []
    evidence_pattern = re.compile(
        r"^Integrator acceptance: reviewed v1 branch-scope amendment plan (\S+) "
        r"SHA-256 ([0-9a-f]{64}); independent review (\S+) SHA-256 ([0-9a-f]{64})\."
    )
    for evidence in plan_task.get("evidence", []):
        if not isinstance(evidence, str):
            continue
        match = evidence_pattern.match(evidence)
        if match is None or match.group(2) != plan_sha256 or match.group(4) != review_sha256:
            continue
        try:
            plan_path = _registry_absolute_path(
                match.group(1), "accepted VIS-10 branch-scope plan path"
            )
            review_path = _registry_absolute_path(
                match.group(3), "accepted VIS-10 branch-scope review path"
            )
            plan_raw = _safe_read(
                plan_path, "accepted VIS-10 branch-scope plan", max_bytes=256 * 1024
            )
            review_raw = _safe_read(
                review_path, "accepted VIS-10 branch-scope plan review", max_bytes=256 * 1024
            )
        except (QueueError, OSError):
            continue
        if hashlib.sha256(plan_raw).hexdigest() != plan_sha256:
            continue
        if hashlib.sha256(review_raw).hexdigest() != review_sha256:
            continue
        try:
            plan_text = plan_raw.decode("utf-8")
        except UnicodeDecodeError:
            continue
        json_blocks = re.findall(r"(?ms)^```json[ \t]*\n(.*?)\n```[ \t]*$", plan_text)
        requests = []
        for block in json_blocks:
            try:
                request = strict_json_loads(block.encode("utf-8"))
            except QueueError:
                continue
            if isinstance(request, dict) and {
                "expected_primary", "branch_scopes", "authorization_evidence"
            }.issubset(request):
                requests.append(request)
        if len(requests) == 1:
            request_work_id = requests[0].get("work_id", "VIS-02")
            if request_work_id == work_id and ("work_id" in requests[0] or work_id == "VIS-02"):
                accepted.append(requests[0])
    return accepted[0] if len(accepted) == 1 else None


def amend_branch_scope_record(
    records_value: Any,
    work_id: str,
    claim_token: str,
    amendment_value: Any,
    expected_revision: int,
) -> tuple[dict[str, Any], str]:
    records = copy.deepcopy(validate_records(records_value))
    _ensure_revision(records, expected_revision)
    require(work_id in BRANCH_SCOPE_AMENDMENT_WORK_IDS,
            "branch-scope amendment is authorized only for VIS-01, VIS-02, or VIS-11")
    _text(claim_token, "claim token")
    require(isinstance(amendment_value, dict), "branch-scope amendment must be an object")
    _exact_keys(
        amendment_value,
        {"expected_primary", "branch_scopes", "authorization_evidence", "plan_sha256", "review_sha256"},
        {"current_branch_scopes"},
        "branch-scope amendment",
    )
    expected_primary = amendment_value["expected_primary"]
    require(isinstance(expected_primary, dict), "expected_primary must be an object")
    primary_fields = {"owner", "reviewer", "priority", "branch", "base_sha", "allowed_paths"}
    _exact_keys(expected_primary, primary_fields, set(), "expected_primary")
    plan_sha256 = amendment_value["plan_sha256"]
    review_sha256 = amendment_value["review_sha256"]
    require(isinstance(plan_sha256, str) and SHA256.fullmatch(plan_sha256) is not None,
            "plan_sha256 must be a full lowercase SHA-256")
    require(isinstance(review_sha256, str) and SHA256.fullmatch(review_sha256) is not None,
            "review_sha256 must be a full lowercase SHA-256")
    accepted_plan = _accepted_branch_scope_plan(records, work_id, plan_sha256, review_sha256)
    require(accepted_plan is not None,
            "plan and review hashes are not bound to accepted VIS-10 evidence")
    _exact_keys(
        accepted_plan,
        {"expected_primary", "branch_scopes", "authorization_evidence"},
        {"work_id", "current_branch_scopes"},
        "accepted branch-scope plan request",
    )
    if "work_id" not in accepted_plan:
        require(work_id == "VIS-02",
                "accepted VIS-10 plan request must bind its work_id")
    else:
        require(accepted_plan["work_id"] == work_id,
                "accepted VIS-10 plan request work_id differs from the target task")

    declared_current_scopes: list[dict[str, Any]] | None = None
    operational_current_scopes: list[dict[str, Any]] | None = None
    if "current_branch_scopes" in accepted_plan:
        declared_current_scopes = _validated_branch_scopes(
            accepted_plan["current_branch_scopes"], "accepted VIS-10 plan", work_id,
        )
        require("current_branch_scopes" in amendment_value,
                "branch-scope amendment must declare current_branch_scopes")
        operational_current_scopes = _validated_branch_scopes(
            amendment_value["current_branch_scopes"], "branch-scope amendment", work_id,
        )
        require(operational_current_scopes == declared_current_scopes,
                "current_branch_scopes differs from the accepted VIS-10 plan")
    else:
        require("current_branch_scopes" not in amendment_value,
                "current_branch_scopes is unsupported without an accepted plan declaration")

    replacement = _validated_branch_scopes(amendment_value["branch_scopes"], work_id, work_id)
    accepted_scopes = _validated_branch_scopes(accepted_plan["branch_scopes"], work_id, work_id)
    require(len(accepted_scopes) == 1,
            "accepted VIS-10 plan must name exactly one visual-baseline branch grant")
    accepted_base_sha = accepted_scopes[0]["base_sha"]
    require(replacement in ([], accepted_scopes),
            "branch-scope replacement differs from the accepted VIS-10 plan")

    task = next((item for item in records["tasks"] if item["work_id"] == work_id), None)
    require(task is not None, "unknown work ID {}".format(work_id))
    require(task["state"] in {"claimed", "in_progress"},
            "branch scopes can only be amended on a claimed or in-progress task")
    require(_task_token(task) == claim_token,
            "claim token does not match the current task claim")
    require(task.get("review_subject_sha256") is None,
            "branch scopes cannot change while a review subject is active")
    for field in sorted(primary_fields):
        require(task.get(field) == expected_primary[field],
                "expected_primary.{} does not match the current task".format(field))
    accepted_primary = accepted_plan["expected_primary"]
    _exact_keys(
        accepted_primary, primary_fields, set(), "accepted branch-scope primary assignment"
    )
    require(expected_primary == accepted_primary,
            "expected_primary differs from the accepted VIS-10 plan")
    for field, value in accepted_primary.items():
        require(task.get(field) == value,
                "{} {} differs from the accepted primary assignment".format(work_id, field))
    current_scopes = _validated_branch_scopes(task.get("branch_scopes", []), work_id, work_id)
    if declared_current_scopes is None:
        require(current_scopes in ([], accepted_scopes),
                "existing visual-baseline grant differs from the accepted VIS-10 plan")
    else:
        require(current_scopes == declared_current_scopes,
                "existing visual-baseline grant differs from the accepted VIS-10 plan")
    require(replacement != current_scopes, "branch-scope amendment is a no-op")

    authorization_evidence = _text(
        amendment_value["authorization_evidence"], "authorization_evidence"
    )
    require(authorization_evidence == accepted_plan["authorization_evidence"],
            "authorization_evidence differs from the accepted VIS-10 plan")

    previous_json = json.dumps(current_scopes, sort_keys=True, separators=(",", ":"))
    replacement_json = json.dumps(replacement, sort_keys=True, separators=(",", ":"))
    records["queue_revision"] += 1
    task["branch_scopes"] = replacement
    task["evidence"].append(authorization_evidence)
    task["evidence"].append(
        "amended branch scopes at queue revision {}: previous {}; replacement {}; "
        "observed refs/heads/visual-baseline {}; accepted plan SHA-256 {}; independent review "
        "SHA-256 {}".format(
            records["queue_revision"], previous_json, replacement_json, accepted_base_sha,
            plan_sha256, review_sha256,
        )
    )
    validate_records(records)
    return records, accepted_base_sha


def reassign_record(
    records_value: Any,
    work_id: str,
    new_claim_value: Any,
    handoff_value: Any,
    expected_revision: int,
    *,
    now: datetime | None = None,
) -> dict[str, Any]:
    records = copy.deepcopy(validate_records(records_value))
    _ensure_revision(records, expected_revision)
    task = next((item for item in records["tasks"] if item["work_id"] == work_id), None)
    require(task is not None, "unknown work ID {}".format(work_id))
    require(task["state"] in ACTIVE_STATES, "only active claims can be reassigned")
    new_claim = copy.deepcopy(new_claim_value)
    require(isinstance(new_claim, dict), "new claim details must be an object")
    allowed = {"owner", "reviewer", "claim_token", "expiry"}
    require(set(new_claim) == allowed, "new claim must contain only owner, reviewer, token, and expiry")
    new_owner = _text(new_claim.get("owner"), "new_claim.owner")
    reviewer = _text(new_claim.get("reviewer"), "new_claim.reviewer")
    require(new_owner != task["owner"], "handoff must assign a different owner")
    require(new_owner != reviewer, "new owner and independent reviewer must differ")
    expiry = parse_time(new_claim.get("expiry"), "new_claim.expiry")
    require(expiry > _now(now), "new claim expiry must be in the future")
    token = _text(new_claim.get("claim_token"), "new_claim.claim_token")
    require(TOKEN.fullmatch(token) is not None, "new claim token is invalid")

    handoff = copy.deepcopy(_validate_handoff(handoff_value, "handoff"))
    require(handoff["from_owner"] == task["owner"] and handoff["to_owner"] == new_owner,
            "handoff owners do not match the current and new claim")
    archived_subject = task.pop("review_subject_sha256", None)
    if archived_subject is not None:
        task["evidence"].append(
            "archived and cleared review subject SHA-256 {} in prior claim evidence before handoff; rebind required".format(
                archived_subject
            )
        )
    prior = _claim_snapshot(task)
    history = task.setdefault("claim_history", [])
    history.append({"claim": prior, "handoff": handoff})
    task["handoff"] = copy.deepcopy(handoff)
    task["owner"] = new_owner
    task["reviewer"] = reviewer
    task["claim_token"] = token
    task["expiry"] = new_claim["expiry"]
    by_id = {item["work_id"]: item for item in records["tasks"]}
    unresolved = [dependency for dependency in task["dependencies"]
                  if by_id[dependency]["state"] != "verified"]
    task["state"] = "blocked" if task["state"] == "blocked" or unresolved else "claimed"
    records["queue_revision"] += 1
    task["accepted_queue_revision"] = records["queue_revision"]
    validate_records(records)
    return records


def _markdown_cell(value: Any) -> str:
    if isinstance(value, list):
        rendered = ", ".join(str(item) for item in value) or "—"
    else:
        rendered = str(value) if value is not None else "—"
    return rendered.replace("|", "\\|").replace("\r", " ").replace("\n", " ")


def render_queue(records_value: Any) -> str:
    records = validate_records(records_value)
    highest = highest_open_priority(records)
    ordered = sorted(records["tasks"], key=lambda task: (PRIORITY[task["priority"]], task["work_id"]))
    lines = [
        "# Work queue",
        "",
        "Generated from [accepted task records](docs/implementation/visibility/tasks.json).",
        "Queue revision: {}. Acceptance owner: `{}`.".format(
            records["queue_revision"], records["accepted_by"]
        ),
        "",
    ]
    if highest is None:
        lines.append("Highest open priority: **none**. All recorded work is verified.")
    else:
        lines.append("Highest open priority: **{}**.".format(highest))
    lines.extend([
        "",
        "| Work | Requirements | Priority | State | Dependencies | Owner | Reviewer | Allowed paths |",
        "| --- | --- | --- | --- | --- | --- | --- | --- |",
    ])
    for task in ordered:
        lines.append("| {} | {} | {} | {} | {} | {} | {} | {} |".format(
            _markdown_cell(task["work_id"]),
            _markdown_cell(task["requirement_ids"]),
            _markdown_cell(task["priority"]),
            _markdown_cell(task["state"]),
            _markdown_cell(task["dependencies"]),
            _markdown_cell(task["owner"]),
            _markdown_cell(task["reviewer"]),
            _markdown_cell(task["allowed_paths"]),
        ))
    lines.extend(["", "## Claim details", ""])
    for task in ordered:
        lines.extend([
            "### {}".format(task["work_id"]),
            "",
            "Priority reason: {}".format(_markdown_cell(task["priority_reason"])),
            "",
            "Claim token: `{}`; accepted at queue revision {}; expires `{}`.".format(
                task["claim_token"], task["accepted_queue_revision"], task["expiry"]
            ),
            "",
            "Branch/base: `{}` / `{}`.".format(task["branch"], task["base_sha"]),
            "",
            "Evidence: {}".format(_markdown_cell(task["evidence"])),
        ])
        branch_scopes = _validated_branch_scopes(task.get("branch_scopes", []), task["work_id"], task["work_id"])
        if branch_scopes:
            lines.extend(["", "Additional branch scopes:"])
            for scope in branch_scopes:
                lines.append("- `{}` / `{}`: {}".format(
                    _markdown_cell(scope["branch"]),
                    _markdown_cell(scope["base_sha"]),
                    _markdown_cell(scope["allowed_paths"]),
                ))
        handoff = task.get("handoff")
        if handoff is not None:
            lines.extend(["", "Last handoff: {} → {} by `{}`. Recorded next action at handoff: {}".format(
                _markdown_cell(handoff["from_owner"]), _markdown_cell(handoff["to_owner"]),
                _markdown_cell(handoff["handoff_by"]), _markdown_cell(handoff["next_action"]),
            )])
        lines.append("")
    lines.extend([
        "Expiry does not release an active claim. Reassignment requires a recorded safe handoff.",
        "Task state is assignment state; it does not prove test results or requirement completion.",
        "",
    ])
    return "\n".join(lines)


def render_queue_v2(records_value: Any) -> str:
    """Render a deterministic schema-v2 queue without changing the v1 view."""
    records = validate_records_v2(records_value)
    highest = highest_open_priority(records)
    ordered = sorted(records["tasks"], key=lambda task: (PRIORITY[task["priority"]], task["work_id"]))
    lines = [
        "# Work queue",
        "",
        "Generated from [accepted task records](docs/implementation/visibility/tasks.json) (schema v2).",
        "Queue revision: {}. Acceptance owner: `{}`.".format(
            records["queue_revision"], records["accepted_by"]
        ),
        "",
    ]
    if highest is None:
        lines.append("Highest open priority: **none**. All recorded work is verified.")
    else:
        lines.append("Highest open priority: **{}**.".format(highest))
    lines.extend([
        "",
        "| Work | Repository | Worktree | Requirements | Priority | State | Dependencies | Owner | Reviewer | Allowed paths |",
        "| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |",
    ])
    for task in ordered:
        lines.append("| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |".format(
            _markdown_cell(task["work_id"]),
            _markdown_cell(task["repository_id"]),
            _markdown_cell(task["worktree_id"]),
            _markdown_cell(task["requirement_ids"]),
            _markdown_cell(task["priority"]),
            _markdown_cell(task["state"]),
            _markdown_cell(task["dependencies"]),
            _markdown_cell(task["owner"]),
            _markdown_cell(task["reviewer"]),
            _markdown_cell(task["allowed_paths"]),
        ))
    lines.extend(["", "## Claim details", ""])
    for task in ordered:
        lines.extend([
            "### {}".format(task["work_id"]),
            "",
            "Repository/worktree: `{}` / `{}`.".format(
                _markdown_cell(task["repository_id"]), _markdown_cell(task["worktree_id"])
            ),
            "",
            "Priority reason: {}".format(_markdown_cell(task["priority_reason"])),
            "",
            "Claim token: `{}`; accepted at queue revision {}; expires `{}`.".format(
                task["claim_token"], task["accepted_queue_revision"], task["expiry"]
            ),
            "",
            "Branch/base: `{}` / `{}`.".format(
                _markdown_cell(task["branch"]), _markdown_cell(task["base_sha"])
            ),
            "",
            "Evidence: {}".format(_markdown_cell(task["evidence"])),
        ])
        handoff = task.get("handoff")
        if handoff is not None:
            lines.extend(["", "Last handoff: {} → {} by `{}`. Recorded next action at handoff: {}.".format(
                _markdown_cell(handoff["from_owner"]), _markdown_cell(handoff["to_owner"]),
                _markdown_cell(handoff["handoff_by"]), _markdown_cell(handoff["next_action"]),
            )])
        lines.append("")
    lines.extend([
        "Expiry does not release an active claim. Reassignment requires a recorded safe handoff.",
        "Task state is assignment state; it does not prove test results or requirement completion.",
        "",
    ])
    return "\n".join(lines)


def instruction_packet(root: Path, allowed_paths: Sequence[str]) -> list[dict[str, str]]:
    """Return ancestor/scoped AGENTS paths and hashes for a static task packet."""
    root = _absolute_path(root)
    found: set[str] = set()
    root_fd = _open_directory_chain(root)
    dir_flags = os.O_RDONLY | getattr(os, "O_DIRECTORY", 0) | getattr(os, "O_NOFOLLOW", 0)
    dir_flags |= getattr(os, "O_CLOEXEC", 0)

    def inspect_directory(directory_fd: int, parts: Sequence[str]) -> None:
        relative_dir = "/".join(parts)
        agents_label = "{}/AGENTS.md".format(relative_dir) if relative_dir else "AGENTS.md"
        claude_label = "{}/CLAUDE.md".format(relative_dir) if relative_dir else "CLAUDE.md"
        try:
            agents_stat = os.stat("AGENTS.md", dir_fd=directory_fd, follow_symlinks=False)
        except FileNotFoundError:
            agents_stat = None
        if agents_stat is not None:
            require(stat.S_ISREG(agents_stat.st_mode),
                    "AGENTS.md must be a regular file: {}".format(agents_label))
            found.add(agents_label)
        try:
            claude_stat = os.stat("CLAUDE.md", dir_fd=directory_fd, follow_symlinks=False)
        except FileNotFoundError:
            claude_stat = None
        if claude_stat is not None:
            require(stat.S_ISLNK(claude_stat.st_mode),
                    "CLAUDE.md must remain a symlink to adjacent AGENTS.md: {}".format(claude_label))
            require(os.readlink("CLAUDE.md", dir_fd=directory_fd) == "AGENTS.md",
                    "CLAUDE.md must point directly to adjacent AGENTS.md: {}".format(claude_label))
            require(agents_stat is not None and stat.S_ISREG(agents_stat.st_mode),
                    "CLAUDE.md target must be a regular adjacent AGENTS.md: {}".format(claude_label))

    def add_directory_chain(parts: Sequence[str]) -> None:
        descriptor = os.dup(root_fd)
        walked: list[str] = []
        try:
            inspect_directory(descriptor, walked)
            for component in parts:
                try:
                    next_descriptor = os.open(component, dir_flags, dir_fd=descriptor)
                except FileNotFoundError:
                    return
                except OSError as error:
                    raise QueueError("instruction parent contains a symlink or non-directory: {}".format(
                        "/".join((*walked, component))
                    )) from error
                os.close(descriptor)
                descriptor = next_descriptor
                walked.append(component)
                inspect_directory(descriptor, walked)
        finally:
            os.close(descriptor)

    def scan_recursive(directory_fd: int, parts: Sequence[str]) -> None:
        inspect_directory(directory_fd, parts)
        for name in sorted(os.listdir(directory_fd)):
            metadata = os.stat(name, dir_fd=directory_fd, follow_symlinks=False)
            relative = "/".join((*parts, name))
            if stat.S_ISLNK(metadata.st_mode):
                require(name == "CLAUDE.md",
                        "instruction scope contains a symlink: {}".format(relative))
                continue
            if not stat.S_ISDIR(metadata.st_mode):
                continue
            try:
                child_fd = os.open(name, dir_flags, dir_fd=directory_fd)
            except OSError as error:
                raise QueueError("cannot safely inspect instruction scope {}: {}".format(
                    relative, error
                )) from error
            try:
                scan_recursive(child_fd, (*parts, name))
            finally:
                os.close(child_fd)

    try:
        for raw_scope in allowed_paths:
            scope = normalize_scope(raw_scope)
            parts, recursive = _scope_parts(scope)
            directory_parts = parts if recursive else parts[:-1]
            add_directory_chain(directory_parts)
            if not recursive and parts:
                parent_fd = _open_relative_directory(root_fd, parts[:-1], scope)
                if parent_fd is not None:
                    try:
                        try:
                            leaf_stat = os.stat(parts[-1], dir_fd=parent_fd, follow_symlinks=False)
                        except FileNotFoundError:
                            leaf_stat = None
                        if leaf_stat is not None and stat.S_ISLNK(leaf_stat.st_mode):
                            require(parts[-1] == "CLAUDE.md",
                                    "instruction scope leaf is a symlink: {}".format(scope))
                            inspect_directory(parent_fd, parts[:-1])
                    finally:
                        os.close(parent_fd)
            if recursive:
                directory_fd = _open_relative_directory(root_fd, parts, scope)
                if directory_fd is not None:
                    try:
                        scan_recursive(directory_fd, parts)
                    finally:
                        os.close(directory_fd)
    finally:
        os.close(root_fd)

    return [
        {"path": relative, "sha256": hashlib.sha256(_safe_read(root / relative, relative)).hexdigest()}
        for relative in sorted(found)
    ]


def _absolute_path(path: Path) -> Path:
    candidate = Path(path)
    require(".." not in candidate.parts, "path must not contain parent traversal")
    if not candidate.is_absolute():
        candidate = Path.cwd() / candidate
    return candidate


def _open_directory_chain(directory: Path) -> int:
    """Open each directory component without following symlinks."""
    require(hasattr(os, "O_NOFOLLOW") and hasattr(os, "O_DIRECTORY"),
            "platform lacks required no-follow directory-open flags")
    absolute = _absolute_path(directory)
    require(absolute.is_absolute(), "directory path must be absolute")
    flags = os.O_RDONLY | getattr(os, "O_DIRECTORY", 0) | getattr(os, "O_NOFOLLOW", 0)
    flags |= getattr(os, "O_CLOEXEC", 0)
    descriptor = os.open(absolute.anchor or "/", flags)
    try:
        for component in absolute.parts[1:]:
            next_descriptor = os.open(component, flags, dir_fd=descriptor)
            os.close(descriptor)
            descriptor = next_descriptor
        return descriptor
    except OSError:
        os.close(descriptor)
        raise


def _open_relative_directory(root_fd: int, parts: Sequence[str], label: str) -> int | None:
    descriptor = os.dup(root_fd)
    flags = os.O_RDONLY | getattr(os, "O_DIRECTORY", 0) | getattr(os, "O_NOFOLLOW", 0)
    flags |= getattr(os, "O_CLOEXEC", 0)
    try:
        for component in parts:
            try:
                next_descriptor = os.open(component, flags, dir_fd=descriptor)
            except FileNotFoundError:
                os.close(descriptor)
                return None
            except OSError as error:
                raise QueueError("{} contains a symlink or non-directory component: {}".format(label, component)) from error
            os.close(descriptor)
            descriptor = next_descriptor
        return descriptor
    except Exception:
        try:
            os.close(descriptor)
        except OSError:
            pass
        raise


def _check_scope_tree(directory_fd: int, relative: str) -> None:
    for name in os.listdir(directory_fd):
        metadata = os.stat(name, dir_fd=directory_fd, follow_symlinks=False)
        path = "{}/{}".format(relative, name)
        require(not stat.S_ISLNK(metadata.st_mode),
                "write claim scope contains a symlink: {}".format(path))
        if stat.S_ISDIR(metadata.st_mode):
            child_flags = os.O_RDONLY | getattr(os, "O_DIRECTORY", 0) | getattr(os, "O_NOFOLLOW", 0)
            child_flags |= getattr(os, "O_CLOEXEC", 0)
            try:
                child_fd = os.open(name, child_flags, dir_fd=directory_fd)
            except OSError as error:
                raise QueueError("cannot safely inspect write claim scope {}: {}".format(path, error)) from error
            try:
                _check_scope_tree(child_fd, path)
            finally:
                os.close(child_fd)
        else:
            require(stat.S_ISREG(metadata.st_mode),
                    "write claim scope contains a non-regular file: {}".format(path))


def validate_filesystem_scopes(root: Path, records: Mapping[str, Any]) -> None:
    """Reject existing symlink paths in active write scopes at mutation time.

    This is a queue preflight only; it cannot sandbox the task owner or prevent
    a link from being created after the check.
    """
    validate_records(records)
    root_fd = _open_directory_chain(root)
    try:
        for task in records["tasks"]:
            if task["state"] not in ACTIVE_STATES:
                continue
            for scope in task["allowed_paths"]:
                parts, recursive = _scope_parts(scope)
                label = "{} ({})".format(task["work_id"], scope)
                if recursive:
                    directory_fd = _open_relative_directory(root_fd, parts, label)
                    if directory_fd is None:
                        continue
                    try:
                        _check_scope_tree(directory_fd, scope[:-3])
                    finally:
                        os.close(directory_fd)
                    continue
                parent_fd = _open_relative_directory(root_fd, parts[:-1], label)
                if parent_fd is None:
                    continue
                try:
                    try:
                        metadata = os.stat(parts[-1], dir_fd=parent_fd, follow_symlinks=False)
                    except FileNotFoundError:
                        continue
                    require(not stat.S_ISLNK(metadata.st_mode),
                            "write claim leaf is a symlink: {}".format(scope))
                    require(stat.S_ISREG(metadata.st_mode),
                            "exact write claim path is not a regular file: {}".format(scope))
                finally:
                    os.close(parent_fd)
    finally:
        os.close(root_fd)


def scope_digest(root: Path, allowed_paths: Sequence[str]) -> str:
    """Hash current regular files in the accepted scopes, excluding tasks.json."""
    require(isinstance(allowed_paths, list) and bool(allowed_paths),
            "review subject allowed_paths must be a nonempty array")
    scopes = [normalize_scope(scope) for scope in allowed_paths]
    require(len(scopes) == len(set(scopes)), "review subject repeats an allowed path")
    for index, scope in enumerate(scopes):
        for other in scopes[index + 1 :]:
            require(not scopes_overlap(scope, other),
                    "review subject contains overlapping allowed paths")
    root_fd = _open_directory_chain(root)
    entries: dict[str, bytes] = {}
    scope_names: list[str] = []
    dir_flags = os.O_RDONLY | getattr(os, "O_DIRECTORY", 0) | getattr(os, "O_NOFOLLOW", 0)
    dir_flags |= getattr(os, "O_CLOEXEC", 0)
    file_flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_CLOEXEC", 0)

    def read_file(directory_fd: int, name: str, relative: str) -> bytes:
        try:
            descriptor = os.open(name, file_flags, dir_fd=directory_fd)
        except OSError as error:
            raise QueueError("cannot safely read review subject file {}: {}".format(relative, error)) from error
        try:
            metadata = os.fstat(descriptor)
            require(stat.S_ISREG(metadata.st_mode),
                    "review subject must contain only regular files: {}".format(relative))
            with os.fdopen(descriptor, "rb") as stream:
                descriptor = -1
                return stream.read()
        finally:
            if descriptor >= 0:
                os.close(descriptor)

    def collect_tree(directory_fd: int, parts: Sequence[str]) -> None:
        for name in sorted(os.listdir(directory_fd)):
            child_parts = (*parts, name)
            relative = "/".join(child_parts)
            if relative == "docs/implementation/visibility/tasks.json":
                continue
            metadata = os.stat(name, dir_fd=directory_fd, follow_symlinks=False)
            require(not stat.S_ISLNK(metadata.st_mode),
                    "review subject scope contains a symlink: {}".format(relative))
            if stat.S_ISDIR(metadata.st_mode):
                try:
                    child_fd = os.open(name, dir_flags, dir_fd=directory_fd)
                except OSError as error:
                    raise QueueError("cannot safely inspect review subject scope {}: {}".format(
                        relative, error
                    )) from error
                try:
                    collect_tree(child_fd, child_parts)
                finally:
                    os.close(child_fd)
            else:
                require(stat.S_ISREG(metadata.st_mode),
                        "review subject must contain only regular files: {}".format(relative))
                entries[relative] = read_file(directory_fd, name, relative)

    try:
        for scope in scopes:
            scope_names.append(scope)
            parts, recursive = _scope_parts(scope)
            if recursive:
                directory_fd = _open_relative_directory(root_fd, parts, scope)
                require(directory_fd is not None,
                        "review subject directory does not exist: {}".format(scope[:-3]))
                try:
                    collect_tree(directory_fd, parts)
                finally:
                    os.close(directory_fd)
                continue
            parent_fd = _open_relative_directory(root_fd, parts[:-1], scope)
            require(parent_fd is not None,
                    "review subject parent directory does not exist: {}".format(scope))
            try:
                if scope == "docs/implementation/visibility/tasks.json":
                    continue
                try:
                    metadata = os.stat(parts[-1], dir_fd=parent_fd, follow_symlinks=False)
                except FileNotFoundError as error:
                    raise QueueError("review subject file does not exist: {}".format(scope)) from error
                require(stat.S_ISREG(metadata.st_mode),
                        "review subject path must be a regular file: {}".format(scope))
                entries[scope] = read_file(parent_fd, parts[-1], scope)
            finally:
                os.close(parent_fd)
    finally:
        os.close(root_fd)

    digest = hashlib.sha256()
    digest.update(b"termrock-visibility-scope-digest-v1\0")
    for scope in sorted(scope_names):
        digest.update(b"scope\0")
        digest.update(scope.encode("utf-8"))
        digest.update(b"\0")
    for relative, contents in sorted(entries.items()):
        digest.update(b"file\0")
        digest.update(relative.encode("utf-8"))
        digest.update(b"\0")
        digest.update(len(contents).to_bytes(8, "big"))
        digest.update(contents)
    require(bool(entries),
            "review subject must include a regular file outside accepted tasks.json")
    return digest.hexdigest()


def _open_parent(path: Path) -> tuple[int, str]:
    absolute = _absolute_path(path)
    require(absolute.name not in {"", ".", ".."}, "file path must name a leaf")
    try:
        return _open_directory_chain(absolute.parent), absolute.name
    except OSError as error:
        raise QueueError("cannot safely open parent directory for {}: {}".format(path, error)) from error


def _safe_read(path: Path, label: str, *, max_bytes: int | None = None) -> bytes:
    parent_fd, leaf = _open_parent(path)
    flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_CLOEXEC", 0)
    try:
        try:
            descriptor = os.open(leaf, flags, dir_fd=parent_fd)
        except OSError as error:
            raise QueueError("cannot safely open {}: {}".format(label, error)) from error
        try:
            metadata = os.fstat(descriptor)
            require(stat.S_ISREG(metadata.st_mode), "{} must be a regular file".format(label))
            if max_bytes is not None:
                require(metadata.st_size <= max_bytes,
                        "{} exceeds the {}-byte limit".format(label, max_bytes))
            with os.fdopen(descriptor, "rb") as stream:
                descriptor = -1
                raw = stream.read() if max_bytes is None else stream.read(max_bytes + 1)
                if max_bytes is not None:
                    require(len(raw) <= max_bytes,
                            "{} exceeds the {}-byte limit".format(label, max_bytes))
                return raw
        finally:
            if descriptor >= 0:
                os.close(descriptor)
    finally:
        os.close(parent_fd)


def _registry_absolute_path(value: Any, label: str) -> Path:
    require(isinstance(value, str) and bool(value), "{} must be an absolute path".format(label))
    require("\x00" not in value and "\n" not in value and "\r" not in value,
            "{} contains an invalid character".format(label))
    candidate = Path(value)
    require(candidate.is_absolute(), "{} must be absolute".format(label))
    require(".." not in candidate.parts and os.path.normpath(value) == value,
            "{} must be normalized and must not traverse its parent".format(label))
    return candidate


def _registry_allowed_paths(value: Any, label: str, *, required: bool) -> list[str]:
    require(isinstance(value, list), "{}.allowed_paths must be an array".format(label))
    require(not required or bool(value), "{}.allowed_paths must be nonempty".format(label))
    scopes = [normalize_scope(path) for path in value]
    require(all(not scope.endswith("/**") for scope in scopes),
            "{}.allowed_paths must use exact paths without recursive globs".format(label))
    require(scopes == sorted(scopes), "{}.allowed_paths must be sorted".format(label))
    require(len(scopes) == len(set(scopes)), "{}.allowed_paths repeats a path".format(label))
    for index, scope in enumerate(scopes):
        scope_parts = PurePosixPath(scope).parts
        for other in scopes[index + 1:]:
            other_parts = PurePosixPath(other).parts
            require(not (scope_parts == other_parts[:len(scope_parts)]
                         or other_parts == scope_parts[:len(other_parts)]),
                    "{}.allowed_paths has overlapping paths {} and {}".format(label, scope, other))
    return scopes


def _validate_source_selection_reference(value: Any, label: str) -> Mapping[str, Any]:
    require(isinstance(value, dict), "{}.source_selection_review must be an object".format(label))
    _exact_keys(value, SOURCE_SELECTION_REVIEW_REF_FIELDS, set(),
                "{}.source_selection_review".format(label))
    path = _registry_absolute_path(
        value.get("review_record_path"), "{}.source_selection_review.review_record_path".format(label)
    )
    require(isinstance(value.get("review_record_sha256"), str)
            and SHA256.fullmatch(value["review_record_sha256"]) is not None,
            "{}.source_selection_review.review_record_sha256 must be a full lowercase SHA-256".format(label))
    require(path.name not in {"", ".", ".."},
            "{}.source_selection_review.review_record_path must name a file".format(label))
    return value


def validate_root_registry(registry: Any) -> Mapping[str, Any]:
    """Validate the portable shape of an externally pinned local root registry."""
    require(isinstance(registry, dict) and type(registry.get("schema_version")) is int
            and registry.get("schema_version") == 1,
            "unsupported root-registry schema")
    _exact_keys(registry, ROOT_REGISTRY_FIELDS, set(), "root registry")

    repositories = registry.get("repositories")
    require(isinstance(repositories, dict) and bool(repositories),
            "root registry repositories must be a nonempty object")
    for repository_id, repository in repositories.items():
        require(isinstance(repository_id, str) and REGISTRY_ID.fullmatch(repository_id) is not None,
                "root registry has an invalid repository ID")
        require(isinstance(repository, dict),
                "repository {} must be an object".format(repository_id))
        _exact_keys(repository, REPOSITORY_FIELDS, set(), "repository {}".format(repository_id))
        origin = _text(repository.get("origin_url"), "repository {}.origin_url".format(repository_id))
        require("\n" not in origin and "\r" not in origin and "\x00" not in origin,
                "repository {}.origin_url contains an invalid character".format(repository_id))

    worktrees = registry.get("worktrees")
    require(isinstance(worktrees, dict) and {"candidate", "reference"} <= set(worktrees),
            "root registry must bind candidate and reference worktrees")
    seen_roots: set[str] = set()
    root_paths: list[Path] = []
    review_paths: list[Path] = []
    for worktree_id, worktree in worktrees.items():
        label = "worktree {}".format(worktree_id)
        require(isinstance(worktree_id, str) and REGISTRY_ID.fullmatch(worktree_id) is not None,
                "root registry has an invalid worktree ID")
        require(isinstance(worktree, dict), "{} must be an object".format(label))
        _exact_keys(worktree, WORKTREE_FIELDS, set(), label)
        repository_id = worktree.get("repository_id")
        require(isinstance(repository_id, str) and repository_id in repositories,
                "{}.repository_id must name a registered repository".format(label))
        root = _registry_absolute_path(worktree.get("root"), label)
        _registry_absolute_path(worktree.get("common_dir"), "{}.common_dir".format(label))
        require(str(root) not in seen_roots, "root registry repeats a worktree root")
        require(all(root not in existing.parents and existing not in root.parents
                    for existing in root_paths),
                "registered worktree roots must not contain one another")
        seen_roots.add(str(root))
        root_paths.append(root)
        mode = worktree.get("mode")
        require(isinstance(mode, str) and mode in WORKTREE_MODES,
                "{}.mode must be branch, source-pin, or scratch".format(label))
        require(type(worktree.get("writable")) is bool,
                "{}.writable must be a boolean policy field".format(label))
        branch = worktree.get("branch")
        pinned_sha = worktree.get("pinned_sha")
        source_pin_id = worktree.get("source_pin_id")
        allowed_paths = _registry_allowed_paths(
            worktree.get("allowed_paths"), label, required=mode == "scratch"
        )
        selection_review = worktree.get("source_selection_review")
        if mode == "source-pin":
            _validate_source_selection_reference(selection_review, label)
            review_paths.append(Path(selection_review["review_record_path"]))
        else:
            require(selection_review is None,
                    "{}.source_selection_review is only valid on a source-pin worktree".format(label))
        if mode == "source-pin":
            require(not allowed_paths,
                    "source-pin worktrees must not declare writable allowed_paths")
        if pinned_sha is not None:
            require(isinstance(pinned_sha, str) and SHA1.fullmatch(pinned_sha) is not None,
                    "{}.pinned_sha must be a full lowercase SHA-1 or null".format(label))
        if worktree["mode"] == "branch":
            _text(branch, "{}.branch".format(label))
            require(source_pin_id is None, "{}.source_pin_id is only valid for scratch".format(label))
            require(worktree["writable"] is True,
                    "branch worktrees must be declared writable")
        elif worktree["mode"] == "source-pin":
            require(branch is None, "source-pin worktrees must be detached")
            require(isinstance(pinned_sha, str) and SHA1.fullmatch(pinned_sha) is not None,
                    "{}.pinned_sha is required for source-pin".format(label))
            require(worktree["writable"] is False,
                    "source-pin worktrees must be declared read-only")
            require(source_pin_id is None, "source-pin cannot reference another source pin")
        else:
            require(branch is None, "scratch worktrees must remain detached")
            require(isinstance(pinned_sha, str) and SHA1.fullmatch(pinned_sha) is not None,
                    "{}.pinned_sha is required for scratch".format(label))
            require(worktree["writable"] is True,
                    "scratch worktrees must be declared writable")
            require(isinstance(source_pin_id, str)
                    and REGISTRY_ID.fullmatch(source_pin_id) is not None,
                    "{}.source_pin_id must name an immutable source-pin worktree".format(label))
    candidate = worktrees["candidate"]
    reference = worktrees["reference"]
    require(candidate["mode"] == "branch"
            and candidate["branch"] == "termrock-implementation"
            and candidate["writable"] is True,
            "candidate must be the writable termrock-implementation branch")
    require(reference["mode"] == "branch"
            and reference["branch"] == "visual-baseline"
            and reference["repository_id"] == candidate["repository_id"]
            and isinstance(reference.get("pinned_sha"), str),
            "reference must pin visual-baseline in the candidate repository")

    has_velnor_source_pin = "velnor-source-pin" in worktrees
    has_velnor_scratch = "velnor-scratch" in worktrees
    require(has_velnor_source_pin == has_velnor_scratch,
            "Velnor registry entries must include both source pin and isolated scratch")
    if has_velnor_source_pin:
        velnor_source_pin = worktrees["velnor-source-pin"]
        velnor_scratch = worktrees["velnor-scratch"]
        require(velnor_source_pin["mode"] == "source-pin"
                and velnor_scratch["mode"] == "scratch"
                and velnor_scratch["source_pin_id"] == "velnor-source-pin",
                "Velnor scratch must refer to its immutable source-pin worktree")

    for source_pin_id, source_pin in worktrees.items():
        if source_pin["mode"] != "source-pin":
            continue
        associated_scratch = [
            worktree_id for worktree_id, worktree in worktrees.items()
            if worktree["mode"] == "scratch" and worktree["source_pin_id"] == source_pin_id
        ]
        require(len(associated_scratch) == 1,
                "source-pin {} must have exactly one associated scratch worktree".format(source_pin_id))

    for review_path in review_paths:
        require(all(review_path != root and not root in review_path.parents for root in root_paths),
                "source-selection review records must be outside every registered worktree root")

    for worktree_id, worktree in worktrees.items():
        if worktree["mode"] != "scratch":
            continue
        source_pin_id = worktree["source_pin_id"]
        source_pin = worktrees.get(source_pin_id)
        require(isinstance(source_pin, dict)
                and source_pin["mode"] == "source-pin"
                and source_pin["repository_id"] == worktree["repository_id"]
                and source_pin["common_dir"] == worktree["common_dir"]
                and source_pin["pinned_sha"] == worktree["pinned_sha"]
                and source_pin["root"] != worktree["root"],
                "scratch {} must use a separate root at its matching immutable source pin".format(
                    worktree_id
                ))

    candidate_origin = repositories[candidate["repository_id"]]["origin_url"]
    for worktree_id, worktree in worktrees.items():
        if worktree["mode"] != "source-pin":
            continue
        source_origin = repositories[worktree["repository_id"]]["origin_url"]
        require(worktree["repository_id"] != candidate["repository_id"]
                and source_origin != candidate_origin,
                "external source-pin {} must use a repository identity and origin distinct from candidate".format(
                    worktree_id
                ))

    authority = registry.get("visual_authority")
    require(isinstance(authority, dict), "root registry visual_authority must be an object")
    _exact_keys(authority, VISUAL_AUTHORITY_FIELDS, set(), "visual_authority")
    require(authority.get("tag") == "visual-baseline",
            "visual_authority.tag must remain visual-baseline")
    for field in ("tag_object_sha", "peeled_commit_sha"):
        require(isinstance(authority.get(field), str) and SHA1.fullmatch(authority[field]) is not None,
                "visual_authority.{} must be a full lowercase SHA-1".format(field))
    require(authority["tag_object_sha"] != authority["peeled_commit_sha"],
            "visual tag object and peeled commit must remain separate identities")
    return registry


def _validate_registry_origin_v2(value: Any, label: str) -> str:
    origin = _text(value, label)
    require(not any(char.isspace() or char in "\x00\r\n" for char in origin),
            "{} must be a valid repository origin URL".format(label))
    try:
        parsed = urlsplit(origin)
        parsed_port = parsed.port
    except ValueError as error:
        raise QueueError(
            "{} must be an HTTP(S), SSH, or scp-style repository origin".format(label)
        ) from error
    if parsed.scheme:
        credentials_are_safe = (
            parsed.username is None and parsed.password is None
            if parsed.scheme in {"http", "https"}
            else parsed.username in {None, "git"} and parsed.password is None
        )
        require(parsed.scheme in {"http", "https", "ssh"}
                and bool(parsed.hostname) and credentials_are_safe
                and (parsed_port is None or 1 <= parsed_port <= 65535)
                and bool(parsed.path.strip("/")),
                "{} must be an HTTP(S), SSH, or scp-style repository origin".format(label))
    else:
        require(re.fullmatch(r"(?:[A-Za-z0-9._-]+@)?[A-Za-z0-9.-]+:[A-Za-z0-9_./-]+", origin)
                is not None,
                "{} must be an HTTP(S), SSH, or scp-style repository origin".format(label))
    return origin


def validate_root_registry_v2(registry: Any) -> Mapping[str, Any]:
    """Validate role and identity relationships without qualifying live roots."""
    require(isinstance(registry, dict) and type(registry.get("schema_version")) is int
            and registry.get("schema_version") == 2,
            "unsupported root-registry schema")
    _exact_keys(registry, ROOT_REGISTRY_V2_FIELDS, set(), "root registry v2")

    queue_authority = registry.get("queue_authority")
    require(isinstance(queue_authority, dict), "queue_authority must be an object")
    _exact_keys(queue_authority, QUEUE_AUTHORITY_FIELDS, set(), "queue_authority")
    for field in sorted(QUEUE_AUTHORITY_FIELDS):
        value = queue_authority.get(field)
        require(isinstance(value, str) and REGISTRY_ID.fullmatch(value) is not None,
                "queue_authority.{} must be a valid registry ID".format(field))
    require(queue_authority["worktree_id"] == "queue-coordinator",
            "queue_authority must select the queue-coordinator role")

    repositories = registry.get("repositories")
    require(isinstance(repositories, dict) and bool(repositories),
            "root registry repositories must be a nonempty object")
    origins: dict[str, str] = {}
    for repository_id, repository in repositories.items():
        require(isinstance(repository_id, str) and REGISTRY_ID.fullmatch(repository_id) is not None,
                "root registry has an invalid repository ID")
        require(isinstance(repository, dict), "repository {} must be an object".format(repository_id))
        _exact_keys(repository, REPOSITORY_FIELDS, set(), "repository {}".format(repository_id))
        origins[repository_id] = _validate_registry_origin_v2(
            repository.get("origin_url"), "repository {}.origin_url".format(repository_id)
        )

    worktrees = registry.get("worktrees")
    required_ids = {"queue-coordinator", "candidate-source", "reference"}
    optional_ids = {"velnor-source-pin", "velnor-scratch"}
    require(isinstance(worktrees, dict) and required_ids <= set(worktrees)
            and set(worktrees) <= required_ids | optional_ids,
            "root registry v2 must contain coordinator, candidate source, reference, and an optional Velnor pair")
    require(("velnor-source-pin" in worktrees) == ("velnor-scratch" in worktrees),
            "Velnor registry entries must include both source pin and isolated scratch")

    roots: list[Path] = []
    common_dirs: dict[str, str] = {}
    review_paths: list[Path] = []
    expected_roles = {
        "queue-coordinator": "queue-coordinator",
        "candidate-source": "candidate-source-authority",
        "reference": "reference-source",
        "velnor-source-pin": "immutable-source-pin",
        "velnor-scratch": "writable-isolated-scratch",
    }
    for worktree_id, worktree in worktrees.items():
        label = "worktree {}".format(worktree_id)
        require(isinstance(worktree, dict), "{} must be an object".format(label))
        _exact_keys(worktree, WORKTREE_V2_FIELDS, set(), label)
        require(worktree.get("role") == expected_roles[worktree_id],
                "{}.role does not match its registered worktree identity".format(label))
        repository_id = worktree.get("repository_id")
        require(isinstance(repository_id, str) and repository_id in repositories,
                "{}.repository_id must name a registered repository".format(label))
        root = _registry_absolute_path(worktree.get("root"), "{}.root".format(label))
        common_dir = _registry_absolute_path(worktree.get("common_dir"), "{}.common_dir".format(label))
        require(all(root != existing
                    and root not in existing.parents
                    and existing not in root.parents
                    for existing in roots),
                "registered worktree roots must be disjoint")
        roots.append(root)
        common_text = str(common_dir)
        prior_common = common_dirs.setdefault(repository_id, common_text)
        require(prior_common == common_text,
                "worktrees of one repository must share a Git common directory")

        mode = worktree.get("mode")
        require(isinstance(mode, str) and mode in WORKTREE_MODES,
                "{}.mode must be branch, source-pin, or scratch".format(label))
        require(type(worktree.get("writable")) is bool,
                "{}.writable must be a boolean policy field".format(label))
        branch = worktree.get("branch")
        pinned_sha = worktree.get("pinned_sha")
        source_pin_id = worktree.get("source_pin_id")
        allowed_paths = _registry_allowed_paths(
            worktree.get("allowed_paths"), label, required=mode == "scratch"
        )
        if pinned_sha is not None:
            require(isinstance(pinned_sha, str) and SHA1.fullmatch(pinned_sha) is not None,
                    "{}.pinned_sha must be a full lowercase SHA-1 or null".format(label))
        selection_review = worktree.get("source_selection_review")
        if worktree_id == "velnor-source-pin":
            _validate_source_selection_reference(selection_review, label)
            review_paths.append(Path(selection_review["review_record_path"]))
        else:
            require(selection_review is None,
                    "{}.source_selection_review is only valid on the Velnor source pin".format(label))

        if worktree_id == "queue-coordinator":
            require(repository_id == queue_authority["repository_id"]
                    and mode == "branch" and branch == "termrock-implementation"
                    and pinned_sha is None and worktree["writable"] is True
                    and source_pin_id is None and not allowed_paths,
                    "queue-coordinator must be the writable termrock-implementation worktree")
        elif worktree_id == "candidate-source":
            require(mode == "source-pin" and branch is None
                    and isinstance(pinned_sha, str) and SHA1.fullmatch(pinned_sha) is not None
                    and worktree["writable"] is False and source_pin_id is None
                    and not allowed_paths,
                    "candidate-source-authority must be a pinned read-only detached worktree")
        elif worktree_id == "reference":
            require(mode == "branch" and branch == "visual-baseline"
                    and isinstance(pinned_sha, str) and SHA1.fullmatch(pinned_sha) is not None
                    and worktree["writable"] is False and source_pin_id is None
                    and not allowed_paths,
                    "reference-source must be a pinned read-only visual-baseline worktree")
        elif worktree_id == "velnor-source-pin":
            require(mode == "source-pin" and branch is None
                    and isinstance(pinned_sha, str) and SHA1.fullmatch(pinned_sha) is not None
                    and worktree["writable"] is False and source_pin_id is None
                    and not allowed_paths,
                    "Velnor source pin must be pinned, detached, and read-only")
        else:
            require(mode == "scratch" and branch is None
                    and isinstance(pinned_sha, str) and SHA1.fullmatch(pinned_sha) is not None
                    and worktree["writable"] is True
                    and source_pin_id == "velnor-source-pin" and bool(allowed_paths),
                    "Velnor scratch must be a detached writable worktree with an exact allowlist")

    coordinator = worktrees["queue-coordinator"]
    candidate_source = worktrees["candidate-source"]
    reference = worktrees["reference"]
    candidate_repository_id = coordinator["repository_id"]
    require(candidate_source["repository_id"] == candidate_repository_id
            and reference["repository_id"] == candidate_repository_id,
            "coordinator, candidate source, and reference must share the Termrock repository identity")
    require(queue_authority["worktree_id"] in worktrees
            and worktrees[queue_authority["worktree_id"]]["role"] == "queue-coordinator",
            "queue_authority does not identify the queue coordinator")
    if "velnor-source-pin" in worktrees:
        source_pin = worktrees["velnor-source-pin"]
        scratch = worktrees["velnor-scratch"]
        require(source_pin["repository_id"] != candidate_repository_id
                and origins[source_pin["repository_id"]] != origins[candidate_repository_id],
                "Velnor source must use a repository identity and origin distinct from candidate")
        require(scratch["repository_id"] == source_pin["repository_id"]
                and scratch["common_dir"] == source_pin["common_dir"]
                and scratch["pinned_sha"] == source_pin["pinned_sha"]
                and source_pin["root"] != scratch["root"],
                "Velnor scratch must use a separate root at its matching immutable source pin")
    referenced_repositories = {worktree["repository_id"] for worktree in worktrees.values()}
    require(referenced_repositories == set(repositories),
            "root registry contains an unreferenced repository identity")
    for review_path in review_paths:
        require(all(review_path != root and root not in review_path.parents for root in roots),
                "source-selection review records must be outside every registered worktree root")

    authority = registry.get("visual_authority")
    require(isinstance(authority, dict), "root registry visual_authority must be an object")
    _exact_keys(authority, VISUAL_AUTHORITY_FIELDS, set(), "visual_authority")
    require(authority.get("tag") == "visual-baseline"
            and authority.get("tag_object_sha") == FROZEN_VISUAL_TAG_OBJECT_SHA
            and authority.get("peeled_commit_sha") == FROZEN_VISUAL_TAG_COMMIT_SHA,
            "visual_authority must match the immutable Termrock tag identities")
    return registry


def _read_root_registry_v2(
    registry_path: Path, expected_sha256: str,
) -> tuple[dict[str, Any], str]:
    require(isinstance(expected_sha256, str) and SHA256.fullmatch(expected_sha256) is not None,
            "registry_sha256 must be a full lowercase SHA-256 supplied by the caller")
    raw = _safe_read(registry_path, "root registry v2")
    actual_sha256 = hashlib.sha256(raw).hexdigest()
    require(actual_sha256 == expected_sha256,
            "root registry raw SHA-256 does not match the caller's pin")
    registry = validate_root_registry_v2(strict_json_loads(raw))
    registry_absolute = _absolute_path(registry_path)
    roots = [Path(worktree["root"]) for worktree in registry["worktrees"].values()]
    require(all(registry_absolute != root and root not in registry_absolute.parents for root in roots),
            "root registry file must be outside every registered worktree root")
    for worktree in registry["worktrees"].values():
        review = worktree["source_selection_review"]
        if review is not None:
            require(Path(review["review_record_path"]) != registry_absolute,
                    "source-selection review record must be separate from the root registry")
    return dict(registry), actual_sha256


def check_root_registry_v2_shape(registry_path: Path, expected_sha256: str) -> dict[str, Any]:
    """Validate caller-pinned v2 role data only; do not probe or qualify live roots."""
    registry, actual_sha256 = _read_root_registry_v2(registry_path, expected_sha256)
    return {
        "schema_version": 2,
        "queue_authority": dict(registry["queue_authority"]),
        "worktree_ids": sorted(registry["worktrees"]),
        "registry_sha256": actual_sha256,
        "live_roots_qualified": False,
    }


def _git_registry_environment() -> dict[str, str]:
    environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
    environment["GIT_CONFIG_NOSYSTEM"] = "1"
    environment["GIT_CONFIG_GLOBAL"] = os.devnull
    environment["GIT_OPTIONAL_LOCKS"] = "0"
    environment["GIT_NO_LAZY_FETCH"] = "1"
    environment["GIT_NO_REPLACE_OBJECTS"] = "1"
    return environment


def _git_registry_output(root: Path, arguments: Sequence[str]) -> str:
    """Run a local read-only Git query without inherited repository overrides."""
    environment = _git_registry_environment()
    command = ["git", "--no-replace-objects", "-C", str(root), *arguments]
    try:
        result = subprocess.run(command, check=True, capture_output=True, text=True, env=environment)
    except (OSError, subprocess.CalledProcessError) as error:
        detail = getattr(error, "stderr", None)
        raise QueueError("cannot verify registered Git worktree with {}: {}".format(
            " ".join(arguments), (detail or str(error)).strip()
        )) from error
    return result.stdout.rstrip("\n")


def _require_git_ancestor(root: Path, ancestor: str, descendant: str, label: str) -> None:
    environment = _git_registry_environment()
    command = [
        "git", "--no-replace-objects", "-C", str(root), "merge-base", "--is-ancestor",
        ancestor, descendant,
    ]
    try:
        result = subprocess.run(command, capture_output=True, text=True, env=environment)
    except OSError as error:
        raise QueueError("cannot verify {} ancestry: {}".format(label, error)) from error
    require(result.returncode == 0,
            "{} is not reachable from {}".format(ancestor, descendant))


def _source_selection_path_policy_digest(
    registry: Mapping[str, Any], source_pin_id: str, scratch_id: str, tree_sha: str,
) -> str:
    source_pin = registry["worktrees"][source_pin_id]
    scratch = registry["worktrees"][scratch_id]
    policy = {
        "repository_id": source_pin["repository_id"],
        "source_pin": {
            "commit_sha": source_pin["pinned_sha"],
            "role": "immutable-source-pin",
            "tree_sha": tree_sha,
            "worktree_id": source_pin_id,
        },
        "scratch": {
            "allowed_paths": sorted(scratch["allowed_paths"]),
            "role": "writable-isolated-scratch",
            "worktree_id": scratch_id,
        },
        "schema": "termrock-source-selection-path-policy/v1",
    }
    canonical = json.dumps(
        policy, ensure_ascii=False, sort_keys=True, separators=(",", ":"), allow_nan=False
    ).encode("utf-8")
    digest = hashlib.sha256()
    digest.update(b"termrock-source-selection-path-policy/v1\0")
    digest.update(canonical)
    return digest.hexdigest()


def _verify_source_selection_review(
    registry: Mapping[str, Any], source_pin_id: str, scratch_id: str,
) -> dict[str, Any]:
    source_pin = registry["worktrees"][source_pin_id]
    reference = source_pin["source_selection_review"]
    review_path = _registry_absolute_path(
        reference["review_record_path"], "source-selection review_record_path"
    )
    raw = _safe_read(
        review_path, "source-selection review record", max_bytes=MAX_SOURCE_SELECTION_REVIEW_BYTES
    )
    actual_sha256 = hashlib.sha256(raw).hexdigest()
    require(actual_sha256 == reference["review_record_sha256"],
            "source-selection review raw SHA-256 does not match the registry pin")
    record = strict_json_loads(raw)
    require(isinstance(record, dict), "source-selection review record must be an object")
    _exact_keys(record, SOURCE_SELECTION_REVIEW_FIELDS, set(), "source-selection review record")
    require(record.get("schema") == "termrock-source-selection-review/v1",
            "unsupported source-selection review schema")
    reviewer = _text(record.get("reviewer"), "source-selection review reviewer")
    require("\n" not in reviewer and "\r" not in reviewer and "\x00" not in reviewer,
            "source-selection review reviewer contains an invalid character")
    require(record.get("decision") == "approved_for_source_selection",
            "source-selection review decision must be approved_for_source_selection")
    parse_time(record.get("reviewed_at"), "source-selection review reviewed_at")
    evidence = record.get("evidence")
    require(isinstance(evidence, list) and bool(evidence),
            "source-selection review evidence must be a nonempty array of links")
    for index, link in enumerate(evidence):
        require(isinstance(link, str) and bool(link) and not any(char.isspace() for char in link),
                "source-selection review evidence[{}] must be an HTTPS link".format(index))
        try:
            parsed_link = urlsplit(link)
        except ValueError as error:
            raise QueueError(
                "source-selection review evidence[{}] must be an HTTPS link".format(index)
            ) from error
        require(parsed_link.scheme == "https" and bool(parsed_link.hostname)
                and parsed_link.username is None and parsed_link.password is None,
                "source-selection review evidence[{}] must be an HTTPS link".format(index))

    subject = record.get("subject")
    require(isinstance(subject, dict), "source-selection review subject must be an object")
    _exact_keys(subject, SOURCE_SELECTION_SUBJECT_FIELDS, set(), "source-selection review subject")
    require(subject.get("repository_id") == source_pin["repository_id"],
            "source-selection review subject repository_id does not match the source pin")
    require(subject.get("commit_sha") == source_pin["pinned_sha"],
            "source-selection review subject commit_sha does not match the source pin")
    for field in ("commit_sha", "tree_sha"):
        require(isinstance(subject.get(field), str) and SHA1.fullmatch(subject[field]) is not None,
                "source-selection review subject.{} must be a full lowercase SHA-1".format(field))
    tree_sha = _git_registry_output(
        _registry_absolute_path(source_pin["root"], "source-pin root"),
        ["rev-parse", "--verify", "HEAD^{tree}"],
    )
    require(SHA1.fullmatch(tree_sha) is not None,
            "registered source-pin tree is not a full lowercase SHA-1")
    require(subject["tree_sha"] == tree_sha,
            "source-selection review subject tree_sha does not match the source-pin Git tree")
    expected_policy_sha256 = _source_selection_path_policy_digest(
        registry, source_pin_id, scratch_id, tree_sha
    )
    require(isinstance(subject.get("path_policy_sha256"), str)
            and SHA256.fullmatch(subject["path_policy_sha256"]) is not None,
            "source-selection review subject.path_policy_sha256 must be a full lowercase SHA-256")
    require(subject["path_policy_sha256"] == expected_policy_sha256,
            "source-selection review subject path policy does not match the registered scratch allowlist")
    return {
        "schema": record["schema"],
        "reviewer": reviewer,
        "decision": record["decision"],
        "reviewed_at": record["reviewed_at"],
        "record_sha256": actual_sha256,
        "path_policy_sha256": expected_policy_sha256,
        "commit_sha": subject["commit_sha"],
        "tree_sha": tree_sha,
    }


def _verify_registered_worktree(
    registry: Mapping[str, Any], worktree_id: str, *, visited: set[str] | None = None,
    require_clean: bool = True,
) -> dict[str, Any]:
    worktrees = registry["worktrees"]
    require(worktree_id in worktrees, "unknown registered worktree {}".format(worktree_id))
    visited = set() if visited is None else visited
    require(worktree_id not in visited, "root registry source-pin cycle includes {}".format(worktree_id))
    visited.add(worktree_id)
    worktree = worktrees[worktree_id]
    root = _registry_absolute_path(worktree["root"], "worktree {}".format(worktree_id))
    common_dir = _registry_absolute_path(
        worktree["common_dir"], "worktree {}.common_dir".format(worktree_id)
    )
    root_fd = -1
    common_fd = -1
    try:
        root_fd = _open_directory_chain(root)
        common_fd = _open_directory_chain(common_dir)
        os.close(root_fd)
        root_fd = -1
        os.close(common_fd)
        common_fd = -1
    except OSError as error:
        raise QueueError("registered worktree {} root/common-dir contains a symlink or is unavailable: {}".format(
            worktree_id, error
        )) from error
    finally:
        if root_fd >= 0:
            os.close(root_fd)
        if common_fd >= 0:
            os.close(common_fd)

    top_level = _git_registry_output(root, ["rev-parse", "--show-toplevel"])
    require(top_level == str(root),
            "registered worktree {} Git top-level does not match its root".format(worktree_id))
    actual_common = _git_registry_output(root, ["rev-parse", "--git-common-dir"])
    actual_common_path = Path(actual_common)
    if not actual_common_path.is_absolute():
        actual_common_path = root / actual_common_path
    require(os.path.normpath(str(actual_common_path)) == str(common_dir),
            "registered worktree {} Git common-dir does not match the registry".format(worktree_id))
    membership = _git_registry_output(root, ["worktree", "list", "--porcelain"])
    listed_roots = [line[len("worktree "):] for line in membership.splitlines()
                    if line.startswith("worktree ")]
    require(str(root) in listed_roots,
            "registered root {} is not a member of its Git common-dir".format(worktree_id))

    repository = registry["repositories"][worktree["repository_id"]]
    fetch_urls = _git_registry_output(root, ["remote", "get-url", "--all", "origin"]).splitlines()
    push_urls = _git_registry_output(root, ["remote", "get-url", "--push", "--all", "origin"]).splitlines()
    expected_origin = repository["origin_url"]
    require(fetch_urls == [expected_origin] and push_urls == [expected_origin],
            "registered worktree {} origin does not match its canonical repository URL".format(
                worktree_id
            ))

    branch = _git_registry_output(root, ["branch", "--show-current"])
    head = _git_registry_output(root, ["rev-parse", "--verify", "HEAD"])
    require(SHA1.fullmatch(head) is not None,
            "registered worktree {} HEAD is not a full lowercase SHA-1".format(worktree_id))
    if worktree["mode"] == "branch":
        require(branch == worktree["branch"],
                "registered worktree {} is on the wrong branch".format(worktree_id))
    else:
        require(branch == "", "registered worktree {} must be detached".format(worktree_id))
    pinned_sha = worktree.get("pinned_sha")
    if pinned_sha is not None:
        require(head == pinned_sha,
                "registered worktree {} HEAD does not match its pinned SHA".format(worktree_id))
    if worktree_id == "reference":
        _require_git_ancestor(
            root, worktree["pinned_sha"], "refs/remotes/origin/visual-baseline",
            "reference worktree pin",
        )
    status = _git_registry_output(
        root, ["status", "--porcelain=v1", "--untracked-files=all", "--ignore-submodules=none"]
    )
    clean = status == ""
    if require_clean:
        require(clean, "registered worktree {} must be clean".format(worktree_id))

    source_pin_id = worktree.get("source_pin_id")
    if source_pin_id is not None:
        _verify_registered_worktree(
            registry, source_pin_id, visited=visited, require_clean=require_clean
        )
    visited.remove(worktree_id)
    return {
        "worktree_id": worktree_id,
        "repository_id": worktree["repository_id"],
        "mode": worktree["mode"],
        "branch": branch or None,
        "head_sha": head,
        "clean": clean,
    }


def _verify_visual_authority(registry: Mapping[str, Any], candidate_root: Path) -> dict[str, str]:
    authority = registry["visual_authority"]
    tag = authority["tag"]
    tag_object_sha = _git_registry_output(
        candidate_root, ["rev-parse", "--verify", "refs/tags/{}^{{tag}}".format(tag)]
    )
    peeled_commit_sha = _git_registry_output(
        candidate_root, ["rev-parse", "--verify", "refs/tags/{}^{{commit}}".format(tag)]
    )
    require(tag_object_sha == FROZEN_VISUAL_TAG_OBJECT_SHA
            and peeled_commit_sha == FROZEN_VISUAL_TAG_COMMIT_SHA,
            "visual-baseline tag no longer matches the immutable frozen authority")
    require(tag_object_sha == authority["tag_object_sha"],
            "visual tag object does not match the caller-pinned authority")
    require(peeled_commit_sha == authority["peeled_commit_sha"],
            "visual tag peeled commit does not match the caller-pinned authority")
    return {
        "tag": tag,
        "tag_object_sha": tag_object_sha,
        "peeled_commit_sha": peeled_commit_sha,
    }


def check_root_registry_v2(
    registry_path: Path,
    expected_sha256: str,
    authority_root: Path,
    worktree_id: str,
) -> dict[str, Any]:
    require(worktree_id in REGISTRY_V2_LIVE_WORKTREES,
            "registry-check-v2 supports only queue-coordinator, candidate-source, and reference")
    registry, actual_sha256 = _read_root_registry_v2(registry_path, expected_sha256)
    coordinator = registry["worktrees"]["queue-coordinator"]
    authority = _absolute_path(authority_root)
    coordinator_root = _registry_absolute_path(
        coordinator["root"], "worktree queue-coordinator"
    )
    require(str(authority) == str(coordinator_root),
            "root registry queue-coordinator root does not exactly match --root")

    coordinator_facts = _verify_registered_worktree(
        registry, "queue-coordinator", require_clean=False
    )
    selected_facts = coordinator_facts
    if worktree_id != "queue-coordinator":
        selected_facts = _verify_registered_worktree(
            registry, worktree_id, require_clean=False
        )

    selected = registry["worktrees"][worktree_id]
    selected_worktree: dict[str, Any] = {
        "worktree_id": worktree_id,
        "role": selected["role"],
        "repository_id": selected["repository_id"],
        "root": selected["root"],
        "common_dir": selected["common_dir"],
        "origin_url": registry["repositories"][selected["repository_id"]]["origin_url"],
        "head_sha": selected_facts["head_sha"],
        "pinned_sha": selected.get("pinned_sha"),
        "branch": selected_facts["branch"],
        "clean": selected_facts["clean"],
        "identity_matches": True,
        "identity_qualified": selected_facts["clean"],
    }
    if worktree_id == "reference":
        reference_root = _registry_absolute_path(
            selected["root"], "worktree reference"
        )
        tracking_ref = "refs/remotes/origin/visual-baseline"
        tracking_sha = _git_registry_output(
            reference_root, ["rev-parse", "--verify", tracking_ref]
        )
        require(SHA1.fullmatch(tracking_sha) is not None,
                "reference tracking ref is not a full lowercase SHA-1")
        selected_worktree["tracking_ref_sha"] = tracking_sha
        selected_worktree["pinned_sha_is_ancestor"] = True

    candidate_source_root = _registry_absolute_path(
        registry["worktrees"]["candidate-source"]["root"],
        "worktree candidate-source",
    )
    try:
        candidate_source_fd = _open_directory_chain(candidate_source_root)
    except OSError as error:
        raise QueueError(
            "registered candidate-source root contains a symlink or is unavailable: {}".format(
                error
            )
        ) from error
    else:
        os.close(candidate_source_fd)
    visual_authority = _verify_visual_authority(registry, candidate_source_root)

    result: dict[str, Any] = {
        "schema": "termrock-visibility-registry-observation/v1",
        "registry_sha256": actual_sha256,
        "queue_authority": {
            "repository_id": registry["queue_authority"]["repository_id"],
            "worktree_id": registry["queue_authority"]["worktree_id"],
            "root": coordinator["root"],
        },
        "selected_worktree": selected_worktree,
        "visual_authority": visual_authority,
        "optional_roles": {
            "velnor-source-pin": "not_probed",
            "velnor-scratch": "not_probed",
        },
        "limits": {
            "read_only_observation": True,
            "unselected_roles_qualified": False,
            "writer_capability": False,
            "actor_authenticated": False,
            "source_selection_decision": False,
            "build_or_product_qualification": False,
        },
    }
    if worktree_id != "queue-coordinator":
        result["coordinator_observation"] = {
            "identity_matches": True,
            "head_sha": coordinator_facts["head_sha"],
            "branch": coordinator_facts["branch"],
            "clean": coordinator_facts["clean"],
            "identity_qualified": coordinator_facts["clean"],
        }
    return result


def check_root_registry(
    registry_path: Path,
    expected_sha256: str,
    authority_root: Path,
    worktree_id: str,
) -> dict[str, Any]:
    require(isinstance(expected_sha256, str) and SHA256.fullmatch(expected_sha256) is not None,
            "registry_sha256 must be a full lowercase SHA-256 supplied by the caller")
    raw = _safe_read(registry_path, "root registry")
    actual_sha256 = hashlib.sha256(raw).hexdigest()
    require(actual_sha256 == expected_sha256,
            "root registry raw SHA-256 does not match the caller's pin")
    registry = validate_root_registry(strict_json_loads(raw))
    registry_absolute = Path(os.path.normpath(str(_absolute_path(registry_path))))
    for registered_worktree in registry["worktrees"].values():
        worktree_root = _registry_absolute_path(
            registered_worktree["root"], "registered worktree root"
        )
        require(registry_absolute != worktree_root and worktree_root not in registry_absolute.parents,
                "root registry file must be outside every registered worktree root")
    for worktree in registry["worktrees"].values():
        if worktree["mode"] != "source-pin":
            continue
        review_path = Path(worktree["source_selection_review"]["review_record_path"])
        require(review_path != registry_absolute,
                "source-selection review record must be separate from the root registry")
    require(worktree_id in registry["worktrees"],
            "unknown registered worktree {}".format(worktree_id))
    authority = _absolute_path(authority_root)
    candidate_root = _registry_absolute_path(
        registry["worktrees"]["candidate"]["root"], "worktree candidate"
    )
    require(str(candidate_root) == str(authority),
            "root registry candidate root does not exactly match --root")
    try:
        candidate_fd = _open_directory_chain(candidate_root)
    except OSError as error:
        raise QueueError("registered candidate root contains a symlink or is unavailable: {}".format(
            error
        )) from error
    else:
        os.close(candidate_fd)
    candidate_facts = _verify_registered_worktree(registry, "candidate")
    if worktree_id == "candidate":
        result = candidate_facts
    else:
        result = _verify_registered_worktree(registry, worktree_id)
        result["candidate_worktree"] = candidate_facts
    selected = registry["worktrees"][worktree_id]
    if selected["mode"] in {"source-pin", "scratch"}:
        if selected["mode"] == "scratch":
            source_pin_id = selected["source_pin_id"]
            scratch_id = worktree_id
        else:
            source_pin_id = worktree_id
            scratch_ids = [
                registered_id for registered_id, candidate in registry["worktrees"].items()
                if candidate["mode"] == "scratch" and candidate["source_pin_id"] == source_pin_id
            ]
            require(len(scratch_ids) == 1,
                    "source pin must have exactly one associated scratch worktree")
            scratch_id = scratch_ids[0]
        result["source_selection_review"] = _verify_source_selection_review(
            registry, source_pin_id, scratch_id
        )
    visual_root = candidate_root
    if worktree_id == "reference":
        visual_root = _registry_absolute_path(
            registry["worktrees"]["reference"]["root"], "worktree reference"
        )
    visual_authority = _verify_visual_authority(registry, visual_root)
    result["visual_authority"] = visual_authority
    result["registry_sha256"] = actual_sha256
    result["authority_root"] = str(authority)
    return result


def read_records(path: Path = TASKS_PATH) -> dict[str, Any]:
    raw = _safe_read(path, "accepted task records")
    value = strict_json_loads(raw)
    return dict(validate_records(value))


def check_rendered_queue(tasks_path: Path = TASKS_PATH, queue_path: Path = QUEUE_PATH) -> str:
    records = read_records(tasks_path)
    expected = render_queue(records)
    try:
        current = _safe_read(queue_path, "generated WORK_QUEUE.md").decode("utf-8")
    except UnicodeDecodeError as error:
        raise QueueError("generated WORK_QUEUE.md is not valid UTF-8") from error
    require(current == expected,
            "WORK_QUEUE.md is stale; run `python3 tools/visibility/queue.py repair-render` explicitly")
    return expected


def read_records_v2(path: Path = TASKS_PATH) -> dict[str, Any]:
    raw = _safe_read(path, "schema-v2 accepted task records")
    value = strict_json_loads(raw)
    return dict(validate_records_v2(value))


def check_rendered_queue_v2(tasks_path: Path = TASKS_PATH, queue_path: Path = QUEUE_PATH) -> str:
    records = read_records_v2(tasks_path)
    expected = render_queue_v2(records)
    try:
        current = _safe_read(queue_path, "generated WORK_QUEUE.md").decode("utf-8")
    except UnicodeDecodeError as error:
        raise QueueError("generated WORK_QUEUE.md is not valid UTF-8") from error
    require(current == expected,
            "schema-v2 WORK_QUEUE.md is stale; this increment has no v2 repair command")
    return expected


def migration_preview(
    tasks_path: Path,
    queue_path: Path,
    repository_id: Any,
    worktree_id: Any,
) -> dict[str, Any]:
    """Preview a uniform v1 identity mapping without locking or writing files."""
    for label, value in (("repository_id", repository_id), ("worktree_id", worktree_id)):
        require(isinstance(value, str) and REGISTRY_ID.fullmatch(value) is not None,
                "{} must be a valid registry ID".format(label))

    tasks_raw = _safe_read(tasks_path, "accepted task records")
    view_raw = _safe_read(queue_path, "generated WORK_QUEUE.md")
    source_value = strict_json_loads(tasks_raw)
    tasks = source_value.get("tasks") if isinstance(source_value, dict) else []
    for task in tasks if isinstance(tasks, list) else []:
        if not isinstance(task, dict):
            continue
        if "branch_scopes" in task:
            require(False, MIGRATION_BRANCH_SCOPE_ERROR)
        history = task.get("claim_history")
        for history_entry in history if isinstance(history, list) else []:
            claim = history_entry.get("claim") if isinstance(history_entry, dict) else None
            if isinstance(claim, dict) and "branch_scopes" in claim:
                require(False, MIGRATION_BRANCH_SCOPE_ERROR)
    source_records = validate_records(source_value)
    expected_v1_view = render_queue(source_records)
    try:
        source_view = view_raw.decode("utf-8")
    except UnicodeDecodeError as error:
        raise QueueError("generated WORK_QUEUE.md is not valid UTF-8") from error
    require(source_view == expected_v1_view,
            "WORK_QUEUE.md is stale; migration preview requires the current generated v1 view")

    proposed = copy.deepcopy(source_records)
    proposed["schema_version"] = 2
    for task in proposed["tasks"]:
        task["repository_id"] = repository_id
        task["worktree_id"] = worktree_id
        for history_entry in task.get("claim_history", []):
            history_entry["claim"]["repository_id"] = repository_id
            history_entry["claim"]["worktree_id"] = worktree_id
    validate_records_v2(proposed)

    return {
        "preview_schema": MIGRATION_PREVIEW_SCHEMA,
        "identity_map": {
            "schema": IDENTITY_MAP_SCHEMA,
            "repository_id": repository_id,
            "worktree_id": worktree_id,
        },
        "source": {
            "schema_version": 1,
            "queue_revision": source_records["queue_revision"],
            "tasks_sha256": hashlib.sha256(tasks_raw).hexdigest(),
            "work_queue_sha256": hashlib.sha256(view_raw).hexdigest(),
        },
        "records": proposed,
        "rendered_queue": render_queue_v2(proposed),
        "writes_files": False,
    }


def _canonical_json(records: Mapping[str, Any]) -> bytes:
    return (json.dumps(records, ensure_ascii=False, indent=2, allow_nan=False) + "\n").encode("utf-8")


def _atomic_write(path: Path, content: bytes) -> None:
    parent_fd, leaf = _open_parent(path)
    temporary_name: str | None = None
    try:
        try:
            metadata = os.stat(leaf, dir_fd=parent_fd, follow_symlinks=False)
        except FileNotFoundError:
            mode = 0o644
        else:
            require(stat.S_ISREG(metadata.st_mode), "output must be a regular non-symlink file: {}".format(path))
            mode = stat.S_IMODE(metadata.st_mode)
        flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_NOFOLLOW", 0)
        flags |= getattr(os, "O_CLOEXEC", 0)
        for _ in range(8):
            candidate = ".{}-{}.tmp".format(leaf, secrets.token_hex(8))
            try:
                descriptor = os.open(candidate, flags, 0o600, dir_fd=parent_fd)
                temporary_name = candidate
                break
            except FileExistsError:
                continue
        else:
            raise QueueError("could not allocate an atomic temporary file for {}".format(path))
        os.fchmod(descriptor, mode)
        with os.fdopen(descriptor, "wb") as stream:
            stream.write(content)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary_name, leaf, src_dir_fd=parent_fd, dst_dir_fd=parent_fd)
        temporary_name = None
        os.fsync(parent_fd)
    finally:
        if temporary_name is not None:
            try:
                os.unlink(temporary_name, dir_fd=parent_fd)
            except FileNotFoundError:
                pass
        os.close(parent_fd)


@contextmanager
def _exclusive_lock(path: Path) -> Iterator[None]:
    canonical_path = _absolute_path(path)
    lock_key = hashlib.sha256(str(canonical_path).encode("utf-8")).hexdigest()
    temp_root = Path(os.path.realpath(tempfile.gettempdir()))
    try:
        temp_fd = _open_directory_chain(temp_root)
    except OSError as error:
        raise QueueError("cannot safely open temporary directory for queue lock: {}".format(error)) from error
    lock_dir_name = "termrock-visibility-queue-{}".format(os.geteuid())
    lock_dir_fd = -1
    lock_fd = -1
    try:
        try:
            os.mkdir(lock_dir_name, 0o700, dir_fd=temp_fd)
            os.fsync(temp_fd)
        except FileExistsError:
            pass
        dir_flags = os.O_RDONLY | getattr(os, "O_DIRECTORY", 0) | getattr(os, "O_NOFOLLOW", 0)
        dir_flags |= getattr(os, "O_CLOEXEC", 0)
        lock_dir_fd = os.open(lock_dir_name, dir_flags, dir_fd=temp_fd)
        directory_stat = os.fstat(lock_dir_fd)
        require(stat.S_ISDIR(directory_stat.st_mode)
                and directory_stat.st_uid == os.geteuid()
                and stat.S_IMODE(directory_stat.st_mode) & 0o077 == 0,
                "queue lock directory must be private and owned by the current user")
        lock_name = lock_key + ".lock"
        no_follow = getattr(os, "O_NOFOLLOW", 0)
        close_on_exec = getattr(os, "O_CLOEXEC", 0)
        create_flags = os.O_RDWR | os.O_CREAT | os.O_EXCL | no_follow | close_on_exec
        existing_flags = os.O_RDWR | no_follow | close_on_exec
        try:
            lock_fd = os.open(lock_name, create_flags, 0o600, dir_fd=lock_dir_fd)
        except FileExistsError:
            lock_fd = os.open(lock_name, existing_flags, dir_fd=lock_dir_fd)

        def verify_lock_leaf() -> None:
            lock_stat = os.fstat(lock_fd)
            require(stat.S_ISREG(lock_stat.st_mode)
                    and lock_stat.st_uid == os.geteuid()
                    and stat.S_IMODE(lock_stat.st_mode) & 0o077 == 0,
                    "queue lock file must be private and owned by the current user")
            entry_stat = os.stat(lock_name, dir_fd=lock_dir_fd, follow_symlinks=False)
            require(stat.S_ISREG(entry_stat.st_mode)
                    and entry_stat.st_uid == os.geteuid()
                    and stat.S_IMODE(entry_stat.st_mode) & 0o077 == 0
                    and entry_stat.st_dev == lock_stat.st_dev
                    and entry_stat.st_ino == lock_stat.st_ino,
                    "queue lock file path must remain bound to its private regular descriptor")

        verify_lock_leaf()
        fcntl.flock(lock_fd, fcntl.LOCK_EX)
        try:
            verify_lock_leaf()
            yield
        finally:
            fcntl.flock(lock_fd, fcntl.LOCK_UN)
    except OSError as error:
        raise QueueError("cannot safely lock queue state: {}".format(error)) from error
    finally:
        if lock_fd >= 0:
            os.close(lock_fd)
        if lock_dir_fd >= 0:
            os.close(lock_dir_fd)
        os.close(temp_fd)


def _git_subject(root: Path) -> tuple[str, str]:
    environment = _git_registry_environment()
    try:
        branch = subprocess.run(
            ["git", "-C", str(root), "branch", "--show-current"],
            check=True, capture_output=True, text=True, env=environment, stdin=subprocess.DEVNULL,
        ).stdout.strip()
        head = subprocess.run(
            ["git", "-C", str(root), "rev-parse", "--verify", "HEAD"],
            check=True, capture_output=True, text=True, env=environment, stdin=subprocess.DEVNULL,
        ).stdout.strip()
    except (OSError, subprocess.CalledProcessError) as error:
        raise QueueError("cannot verify current Git subject: {}".format(error)) from error
    require(SHA1.fullmatch(head) is not None, "current HEAD is not a full lowercase SHA-1")
    return branch, head


def _git_noninteractive_environment() -> dict[str, str]:
    environment = _git_registry_environment()
    environment["GIT_TERMINAL_PROMPT"] = "0"
    environment["GCM_INTERACTIVE"] = "Never"
    environment["SSH_ASKPASS_REQUIRE"] = "never"
    environment.pop("SSH_ASKPASS", None)
    return environment


def _kill_reap_process_group(process: subprocess.Popen[bytes]) -> None:
    cleanup_error: OSError | None = None
    try:
        os.killpg(process.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass
    except OSError as error:
        cleanup_error = error
        try:
            process.kill()
        except OSError:
            pass
    try:
        process.wait()
    except OSError as error:
        cleanup_error = cleanup_error or error
    for stream in (process.stdout, process.stderr):
        if stream is not None:
            try:
                stream.close()
            except OSError as error:
                cleanup_error = cleanup_error or error
    if cleanup_error is not None:
        raise QueueError("cannot clean up bounded Git reference probe: {}".format(cleanup_error))


def _visual_baseline_branch_tip(root: Path) -> str:
    stdout = bytearray()
    stderr = bytearray()
    total_bytes = 0
    selector = selectors.DefaultSelector()
    deadline = time.monotonic() + REFERENCE_PROBE_TIMEOUT_SECONDS
    command = [
        "git", "--no-replace-objects",
        "-c", "credential.helper=",
        "-c", "url.{}.insteadOf={}".format(REFERENCE_HTTPS_ORIGIN, REFERENCE_SSH_ORIGIN),
        "-C", str(root), "ls-remote", "--exit-code", "--refs",
        "origin", "refs/heads/visual-baseline",
    ]
    try:
        process = subprocess.Popen(
            command,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            env=_git_noninteractive_environment(),
            close_fds=True,
            start_new_session=True,
            bufsize=0,
        )
    except OSError as error:
        selector.close()
        raise QueueError("cannot start bounded visual-baseline reference probe: {}".format(error)) from error

    streams = ((process.stdout, stdout), (process.stderr, stderr))
    try:
        for stream, _buffer in streams:
            assert stream is not None
            os.set_blocking(stream.fileno(), False)
            selector.register(stream, selectors.EVENT_READ)
        while selector.get_map() or process.poll() is None:
            remaining = deadline - time.monotonic()
            require(remaining > 0, "visual-baseline reference probe exceeded 10-second deadline")
            if not selector.get_map():
                time.sleep(min(remaining, 0.02))
                continue
            events = selector.select(remaining)
            if not events:
                require(time.monotonic() < deadline,
                        "visual-baseline reference probe exceeded 10-second deadline")
                continue
            for key, _mask in events:
                stream = key.fileobj
                buffer = stdout if stream is process.stdout else stderr
                read_limit = min(8192, MAX_REFERENCE_PROBE_BYTES - total_bytes + 1)
                chunk = os.read(stream.fileno(), read_limit)
                if not chunk:
                    selector.unregister(stream)
                    stream.close()
                    continue
                buffer.extend(chunk)
                total_bytes += len(chunk)
                require(total_bytes <= MAX_REFERENCE_PROBE_BYTES,
                        "visual-baseline reference probe exceeded 4096-byte output cap")
        remaining = deadline - time.monotonic()
        require(remaining > 0, "visual-baseline reference probe exceeded 10-second deadline")
        try:
            return_code = process.wait(timeout=remaining)
        except subprocess.TimeoutExpired as error:
            raise QueueError("visual-baseline reference probe exceeded 10-second deadline") from error
    except BaseException as error:
        try:
            _kill_reap_process_group(process)
        except QueueError as cleanup_error:
            raise QueueError("{}; {}".format(error, cleanup_error)) from error
        raise
    finally:
        selector.close()
        for stream, _buffer in streams:
            if stream is not None and not stream.closed:
                stream.close()

    require(return_code == 0, "visual-baseline reference probe exited with status {}".format(return_code))
    require(not stderr, "visual-baseline reference probe wrote to stderr")
    match = REFERENCE_REF_OUTPUT.fullmatch(bytes(stdout))
    require(match is not None,
            "visual-baseline reference probe must return one exact SHA/ref line")
    return match.group(1).decode("ascii")


def _write_pair(records: Mapping[str, Any], tasks_path: Path, queue_path: Path) -> None:
    validate_records(records)
    queue_bytes = render_queue(records).encode("utf-8")
    # Source is replaced first. If the process stops before the derived view is
    # replaced, check fails closed and repair-render is required explicitly.
    _atomic_write(tasks_path, _canonical_json(records))
    _atomic_write(queue_path, queue_bytes)
    check_rendered_queue(tasks_path, queue_path)


def _read_json_input(value: str) -> bytes:
    if value == "-":
        return sys.stdin.buffer.read()
    return _safe_read(Path(value), "JSON input")


def _load_json_input(value: str) -> Any:
    return strict_json_loads(_read_json_input(value))


def _mutate(
    expected_revision: int,
    update: Any,
    *,
    tasks_path: Path,
    queue_path: Path,
    repo_root: Path,
    subject_reader: Any,
) -> dict[str, Any]:
    with _exclusive_lock(tasks_path):
        records = read_records(tasks_path)
        # A stale generated view is never silently repaired as part of a claim.
        check_rendered_queue(tasks_path, queue_path)
        branch, head = subject_reader()
        require(branch == "termrock-implementation",
                "queue mutations are only allowed on termrock-implementation")
        validate_filesystem_scopes(repo_root, records)
        changed = update(records, branch, head)
        validate_filesystem_scopes(repo_root, changed)
        previously_verified = {
            task["work_id"] for task in records["tasks"] if task["state"] == "verified"
        }
        for task in changed["tasks"]:
            if task["state"] == "verified" and task["work_id"] not in previously_verified:
                require(scope_digest(repo_root, task["allowed_paths"]) == task["review_subject_sha256"],
                        "{} allowed-path digest changed before the review receipt was recorded".format(
                            task["work_id"]
                        ))
        _write_pair(changed, tasks_path, queue_path)
        return changed


def _repair_render(
    tasks_path: Path,
    queue_path: Path,
    *,
    repo_root: Path,
    subject_reader: Any,
) -> int:
    with _exclusive_lock(tasks_path):
        branch, _ = subject_reader()
        require(branch == "termrock-implementation",
                "queue mutations are only allowed on termrock-implementation")
        records = read_records(tasks_path)
        _atomic_write(queue_path, render_queue(records).encode("utf-8"))
        check_rendered_queue(tasks_path, queue_path)
        return records["queue_revision"]


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT, help=argparse.SUPPRESS)
    subparsers = parser.add_subparsers(dest="command", required=True)
    subparsers.add_parser("check", help="validate records and confirm WORK_QUEUE.md is current")
    subparsers.add_parser("render", help="print the deterministic WORK_QUEUE.md view")
    subparsers.add_parser("repair-render", help="explicitly regenerate stale WORK_QUEUE.md")

    packet = subparsers.add_parser("instructions", help="print static instruction paths and hashes for a claim")
    packet.add_argument("work_id")

    subject = subparsers.add_parser(
        "subject-digest", help="hash current regular files in a claim's allowed paths"
    )
    subject.add_argument("work_id")

    accept = subparsers.add_parser("accept", help="accept one claim using revision compare-and-swap")
    accept.add_argument("--expected-revision", required=True, type=int)
    accept.add_argument("--record", required=True, help="JSON file, or - for stdin")

    transition = subparsers.add_parser("transition", help="record a task state transition and evidence")
    transition.add_argument("work_id")
    transition.add_argument("state", choices=sorted(STATES))
    transition.add_argument("--expected-revision", required=True, type=int)
    transition.add_argument("--evidence", required=True)
    transition.add_argument("--review-record", help="external review JSON, required for verified state")
    transition.add_argument(
        "--review-record-sha256",
        help="trusted caller's SHA-256 of the exact external review file bytes",
    )

    append_evidence = subparsers.add_parser(
        "append-evidence", help="append claim evidence through revision compare-and-swap"
    )
    append_evidence.add_argument("work_id")
    append_evidence.add_argument("--expected-revision", required=True, type=int)
    append_evidence.add_argument("--claim-token", required=True)
    append_evidence.add_argument("--evidence", required=True)

    reviewer_handoff = subparsers.add_parser(
        "reviewer-handoff", help="change only an active claim's assigned reviewer through revision CAS"
    )
    reviewer_handoff.add_argument("work_id")
    reviewer_handoff.add_argument("--expected-revision", required=True, type=int)
    reviewer_handoff.add_argument("--claim-token", required=True)
    reviewer_handoff.add_argument("--expected-reviewer", required=True)
    reviewer_handoff.add_argument("--new-reviewer", required=True)
    reviewer_handoff.add_argument("--evidence", required=True)

    bind_subject = subparsers.add_parser(
        "bind-review-subject", help="bind the current review scope through revision compare-and-swap"
    )
    bind_subject.add_argument("work_id")
    bind_subject.add_argument("--expected-revision", required=True, type=int)
    bind_subject.add_argument("--claim-token", required=True)
    bind_subject.add_argument("--expected-subject-sha256", required=True)

    amend_scope = subparsers.add_parser(
        "amend-branch-scope", help="amend an accepted schema-v1 branch grant through revision CAS"
    )
    amend_scope.add_argument("work_id")
    amend_scope.add_argument("--expected-revision", required=True, type=int)
    amend_scope.add_argument("--claim-token", required=True)
    amend_scope.add_argument("--record", required=True, help="reviewed branch-scope amendment JSON file")

    handoff = subparsers.add_parser("handoff", help="safely reassign an active or expired claim")
    handoff.add_argument("work_id")
    handoff.add_argument("--expected-revision", required=True, type=int)
    handoff.add_argument("--record", required=True, help="JSON with new_claim and handoff, or - for stdin")

    registry = subparsers.add_parser(
        "registry-check", help="check a caller-pinned local worktree registry without writing"
    )
    registry.add_argument("--registry", required=True, type=Path)
    registry.add_argument("--registry-sha256", required=True)
    registry.add_argument("--worktree", required=True)

    registry_v2 = subparsers.add_parser(
        "registry-validate-v2", help="validate caller-pinned schema-v2 role data without probing roots"
    )
    registry_v2.add_argument("--registry", required=True, type=Path)
    registry_v2.add_argument("--registry-sha256", required=True)

    registry_check_v2 = subparsers.add_parser(
        "registry-check-v2", help="observe one caller-pinned schema-v2 worktree without writing"
    )
    registry_check_v2.add_argument("--registry", required=True, type=Path)
    registry_check_v2.add_argument("--registry-sha256", required=True)
    registry_check_v2.add_argument("--worktree", required=True)

    subparsers.add_parser("check-v2", help="validate schema-v2 records and confirm their view is current")
    subparsers.add_parser("render-v2", help="print the deterministic schema-v2 queue view")

    preview = subparsers.add_parser(
        "migration-preview", help="preview a no-write schema-v1 to v2 identity mapping"
    )
    preview.add_argument("--repository-id", required=True)
    preview.add_argument("--worktree-id", required=True)
    return parser


def run(args: Sequence[str] | None = None) -> int:
    parser = build_parser()
    options = parser.parse_args(args)
    root = _absolute_path(options.root)
    tasks_path = root / "docs/implementation/visibility/tasks.json"
    queue_path = root / "WORK_QUEUE.md"
    try:
        if options.command == "registry-check":
            result = check_root_registry(
                options.registry, options.registry_sha256, root, options.worktree
            )
            print(json.dumps(result, sort_keys=True))
        elif options.command == "registry-check-v2":
            result = check_root_registry_v2(
                options.registry, options.registry_sha256, root, options.worktree
            )
            print(json.dumps(result, sort_keys=True, allow_nan=False))
        elif options.command == "registry-validate-v2":
            result = check_root_registry_v2_shape(options.registry, options.registry_sha256)
            print(json.dumps(result, sort_keys=True, allow_nan=False))
        elif options.command == "check":
            records = read_records(tasks_path)
            check_rendered_queue(tasks_path, queue_path)
            print("queue revision {}; {} accepted task(s); highest open priority {}".format(
                records["queue_revision"], len(records["tasks"]), highest_open_priority(records) or "none"
            ))
        elif options.command == "check-v2":
            check_rendered_queue_v2(tasks_path, queue_path)
            print("schema-v2 queue view is current")
        elif options.command == "render":
            records = read_records(tasks_path)
            sys.stdout.write(render_queue(records))
        elif options.command == "render-v2":
            records = read_records_v2(tasks_path)
            sys.stdout.write(render_queue_v2(records))
        elif options.command == "migration-preview":
            preview = migration_preview(
                tasks_path,
                queue_path,
                options.repository_id,
                options.worktree_id,
            )
            print(json.dumps(preview, sort_keys=True, allow_nan=False))
        elif options.command == "repair-render":
            revision = _repair_render(
                tasks_path, queue_path, repo_root=root, subject_reader=lambda: _git_subject(root)
            )
            print("rendered WORK_QUEUE.md from queue revision {}".format(revision))
        elif options.command == "instructions":
            records = read_records(tasks_path)
            task = next((item for item in records["tasks"] if item["work_id"] == options.work_id), None)
            require(task is not None, "unknown work ID {}".format(options.work_id))
            packet = instruction_packet(root, task["allowed_paths"])
            print(json.dumps({"work_id": options.work_id, "static_only": True, "instructions": packet}, indent=2))
        elif options.command == "subject-digest":
            records = read_records(tasks_path)
            task = next((item for item in records["tasks"] if item["work_id"] == options.work_id), None)
            require(task is not None, "unknown work ID {}".format(options.work_id))
            print(scope_digest(root, task["allowed_paths"]))
        elif options.command == "accept":
            proposed = _load_json_input(options.record)
            changed = _mutate(
                options.expected_revision,
                lambda current, branch, head: accept_record(
                    current, proposed, options.expected_revision,
                    current_branch=branch, current_head=head,
                ),
                tasks_path=tasks_path, queue_path=queue_path,
                repo_root=root, subject_reader=lambda: _git_subject(root),
            )
            print("accepted {} at queue revision {}".format(proposed["work_id"], changed["queue_revision"]))
        elif options.command == "transition":
            review_bytes = _read_json_input(options.review_record) if options.review_record else None

            def update_transition(current: Mapping[str, Any], _branch: str, _head: str) -> dict[str, Any]:
                current_subject = None
                if options.state == "verified":
                    task = next((item for item in current["tasks"] if item["work_id"] == options.work_id), None)
                    require(task is not None, "unknown work ID {}".format(options.work_id))
                    current_subject = scope_digest(root, task["allowed_paths"])
                return transition_record(
                    current, options.work_id, options.state, options.evidence, options.expected_revision,
                    review_record_raw=review_bytes,
                    review_record_sha256=options.review_record_sha256,
                    current_subject_sha256=current_subject,
                )

            changed = _mutate(
                options.expected_revision,
                update_transition,
                tasks_path=tasks_path, queue_path=queue_path,
                repo_root=root, subject_reader=lambda: _git_subject(root),
            )
            print("updated {} at queue revision {}".format(options.work_id, changed["queue_revision"]))
        elif options.command == "append-evidence":
            changed = _mutate(
                options.expected_revision,
                lambda current, _branch, _head: append_evidence_record(
                    current, options.work_id, options.claim_token,
                    options.evidence, options.expected_revision,
                ),
                tasks_path=tasks_path, queue_path=queue_path,
                repo_root=root, subject_reader=lambda: _git_subject(root),
            )
            print("appended evidence for {} at queue revision {}".format(
                options.work_id, changed["queue_revision"]
            ))
        elif options.command == "reviewer-handoff":
            changed = _mutate(
                options.expected_revision,
                lambda current, _branch, _head: reviewer_handoff_record(
                    current, options.work_id, options.claim_token,
                    options.expected_reviewer, options.new_reviewer, options.evidence,
                    options.expected_revision,
                ),
                tasks_path=tasks_path, queue_path=queue_path,
                repo_root=root, subject_reader=lambda: _git_subject(root),
            )
            print("updated reviewer for {} at queue revision {}".format(
                options.work_id, changed["queue_revision"]
            ))
        elif options.command == "bind-review-subject":
            def update_review_subject(current: Mapping[str, Any], _branch: str, _head: str) -> dict[str, Any]:
                task = next((item for item in current["tasks"] if item["work_id"] == options.work_id), None)
                require(task is not None, "unknown work ID {}".format(options.work_id))
                current_subject = scope_digest(root, task["allowed_paths"])
                return bind_review_subject_record(
                    current, options.work_id, options.claim_token,
                    options.expected_subject_sha256, options.expected_revision,
                    current_subject_sha256=current_subject,
                )

            changed = _mutate(
                options.expected_revision, update_review_subject,
                tasks_path=tasks_path, queue_path=queue_path,
                repo_root=root, subject_reader=lambda: _git_subject(root),
            )
            task = next(item for item in changed["tasks"] if item["work_id"] == options.work_id)
            print("bound review subject for {} at queue revision {}: {}".format(
                options.work_id, changed["queue_revision"], task["review_subject_sha256"]
            ))
        elif options.command == "amend-branch-scope":
            amendment = _load_json_input(options.record)

            def update_branch_scope(current: Mapping[str, Any], _branch: str, _head: str) -> dict[str, Any]:
                _ensure_revision(current, options.expected_revision)
                changed, accepted_base_sha = amend_branch_scope_record(
                    current, options.work_id, options.claim_token, amendment,
                    options.expected_revision,
                )
                reference_tip = _visual_baseline_branch_tip(root)
                require(reference_tip == accepted_base_sha,
                        "visual-baseline branch tip changed from the accepted VIS-10 plan base SHA")
                return changed

            changed = _mutate(
                options.expected_revision, update_branch_scope,
                tasks_path=tasks_path, queue_path=queue_path,
                repo_root=root, subject_reader=lambda: _git_subject(root),
            )
            print("amended branch scopes for {} at queue revision {}".format(
                options.work_id, changed["queue_revision"]
            ))
        elif options.command == "handoff":
            payload = _load_json_input(options.record)
            require(isinstance(payload, dict) and set(payload) == {"new_claim", "handoff"},
                    "handoff JSON must contain new_claim and handoff")
            changed = _mutate(
                options.expected_revision,
                lambda current, _branch, _head: reassign_record(
                    current, options.work_id, payload["new_claim"], payload["handoff"],
                    options.expected_revision,
                ),
                tasks_path=tasks_path, queue_path=queue_path,
                repo_root=root, subject_reader=lambda: _git_subject(root),
            )
            print("reassigned {} at queue revision {}".format(options.work_id, changed["queue_revision"]))
        return 0
    except (QueueError, OSError) as error:
        print("visibility queue error: {}".format(error), file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(run())
