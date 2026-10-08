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
import stat
import subprocess
import sys
import tempfile
from contextlib import contextmanager
from datetime import datetime, timezone
from pathlib import Path, PurePosixPath
from typing import Any, Iterator, Mapping, Sequence


ROOT = Path(__file__).resolve().parents[2]
TASKS_PATH = ROOT / "docs/implementation/visibility/tasks.json"
QUEUE_PATH = ROOT / "WORK_QUEUE.md"
PRIORITY = {"P0": 0, "P1": 1, "P2": 2, "P3": 3}
STATES = {"ready", "claimed", "in_progress", "review", "blocked", "verified"}
ACTIVE_STATES = {"claimed", "in_progress", "review", "blocked"}
SHA1 = re.compile(r"^[0-9a-f]{40}$")
SHA256 = re.compile(r"^[0-9a-f]{64}$")
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


def _claim_snapshot(task: Mapping[str, Any]) -> dict[str, Any]:
    fields = (
        "owner", "reviewer", "branch", "base_sha", "allowed_paths", "claim_token",
        "expiry", "state", "accepted_queue_revision", "evidence",
    )
    return {field: copy.deepcopy(task.get(field)) for field in fields}


def validate_records(records: Any) -> Mapping[str, Any]:
    require(isinstance(records, dict) and type(records.get("schema_version")) is int
            and records.get("schema_version") == 1,
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
        _exact_keys(task, TASK_REQUIRED_FIELDS, TASK_OPTIONAL_FIELDS, label)
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
        require(task.get("branch") == "termrock-implementation",
                "{} must use termrock-implementation".format(work_id))
        require(isinstance(task.get("base_sha"), str) and SHA1.fullmatch(task["base_sha"]) is not None,
                "{}.base_sha must be a full lowercase SHA-1".format(work_id))
        _task_scopes(task)
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
            require(set(prior) == CLAIM_SNAPSHOT_FIELDS,
                    "{} claim snapshot has missing or unknown fields".format(history_label))
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
            for scope in task["allowed_paths"]:
                for other_scope in other["allowed_paths"]:
                    require(not scopes_overlap(scope, other_scope),
                            "active path conflict: {} ({}) overlaps {} ({})".format(
                                task["work_id"], scope, other["work_id"], other_scope
                            ))
    return records


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
    task["state"] = new_state
    task["evidence"].append(evidence)
    records["queue_revision"] += 1
    validate_records(records)
    return records


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
        handoff = task.get("handoff")
        if handoff is not None:
            lines.extend(["", "Last handoff: {} → {} by `{}`. Next: {}".format(
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


def _safe_read(path: Path, label: str) -> bytes:
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
            with os.fdopen(descriptor, "rb") as stream:
                descriptor = -1
                return stream.read()
        finally:
            if descriptor >= 0:
                os.close(descriptor)
    finally:
        os.close(parent_fd)


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
        lock_flags = os.O_RDWR | os.O_CREAT | getattr(os, "O_NOFOLLOW", 0)
        lock_flags |= getattr(os, "O_CLOEXEC", 0)
        lock_fd = os.open(lock_key + ".lock", lock_flags, 0o600, dir_fd=lock_dir_fd)
        lock_stat = os.fstat(lock_fd)
        require(stat.S_ISREG(lock_stat.st_mode)
                and lock_stat.st_uid == os.geteuid()
                and stat.S_IMODE(lock_stat.st_mode) & 0o077 == 0,
                "queue lock file must be private and owned by the current user")
        fcntl.flock(lock_fd, fcntl.LOCK_EX)
        try:
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
    try:
        branch = subprocess.run(
            ["git", "-C", str(root), "branch", "--show-current"],
            check=True, capture_output=True, text=True,
        ).stdout.strip()
        head = subprocess.run(
            ["git", "-C", str(root), "rev-parse", "--verify", "HEAD"],
            check=True, capture_output=True, text=True,
        ).stdout.strip()
    except (OSError, subprocess.CalledProcessError) as error:
        raise QueueError("cannot verify current Git subject: {}".format(error)) from error
    require(SHA1.fullmatch(head) is not None, "current HEAD is not a full lowercase SHA-1")
    return branch, head


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

    handoff = subparsers.add_parser("handoff", help="safely reassign an active or expired claim")
    handoff.add_argument("work_id")
    handoff.add_argument("--expected-revision", required=True, type=int)
    handoff.add_argument("--record", required=True, help="JSON with new_claim and handoff, or - for stdin")
    return parser


def run(args: Sequence[str] | None = None) -> int:
    parser = build_parser()
    options = parser.parse_args(args)
    root = _absolute_path(options.root)
    tasks_path = root / "docs/implementation/visibility/tasks.json"
    queue_path = root / "WORK_QUEUE.md"
    try:
        if options.command == "check":
            records = read_records(tasks_path)
            check_rendered_queue(tasks_path, queue_path)
            print("queue revision {}; {} accepted task(s); highest open priority {}".format(
                records["queue_revision"], len(records["tasks"]), highest_open_priority(records) or "none"
            ))
        elif options.command == "render":
            records = read_records(tasks_path)
            sys.stdout.write(render_queue(records))
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
