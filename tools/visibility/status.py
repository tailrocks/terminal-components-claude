#!/usr/bin/env python3
"""Render the initial, receipt-free Termrock visibility status page."""

from __future__ import annotations

import argparse
import json
import re
import sys
from datetime import datetime
from pathlib import Path
from typing import Any, Mapping, Optional, Sequence

ROOT = Path(__file__).resolve().parents[2]
FACTS_PATH = ROOT / "tools/visibility/source-facts.json"
TASKS_PATH = ROOT / "docs/implementation/visibility/tasks.json"
STATUS_PATH = ROOT / "STATUS.md"
REPOSITORY = "tailrocks/terminal-components-claude"
TAG_OBJECT = "1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5"
TAG_COMMIT = "4a79c0a2d40fca46fc406b77157ce3b3f12ec16b"
SHA = re.compile(r"^[0-9a-f]{40}$")
PRIORITY = {"P0": 0, "P1": 1, "P2": 2, "P3": 3}
TASK_STATES = {"ready", "claimed", "in_progress", "review", "blocked", "verified"}

COMMANDS = (
    ("RUN-01", "cargo run --release --bin showcase"),
    ("RUN-02", "cargo run --release --bin jackin-preview"),
    ("RUN-03", "cargo run --release --bin holla"),
    ("RUN-04", "cargo run --release --bin tablepro"),
    ("RUN-05", "cargo run --release --bin tablepro -- --connect Production"),
    ("RUN-06", "cargo run --release --bin jackin-preview -- --scenario accounts-mixed"),
    ("RUN-07", "cargo run --release --bin holla -- --scenario remote-host"),
)
APPS = ("Showcase", "Jackin Preview", "Holla", "TablePro")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def sha(value: Any, label: str) -> str:
    require(isinstance(value, str) and SHA.fullmatch(value) is not None,
            "{} must be a full lowercase SHA-1".format(label))
    return value


def timestamp(value: Any, label: str) -> str:
    require(isinstance(value, str), "{} must be text".format(label))
    try:
        datetime.strptime(value, "%Y-%m-%dT%H:%M:%SZ")
    except ValueError as error:
        raise ValueError("{} must be UTC RFC3339 seconds".format(label)) from error
    return value


def validate_facts(facts: Any) -> Mapping[str, Any]:
    require(isinstance(facts, dict) and facts.get("schema_version") == 1,
            "unsupported source-facts schema")
    require(facts.get("repository") == REPOSITORY, "repository identity changed")
    timestamp(facts.get("observed_at"), "observed_at")

    candidate, reference = facts.get("candidate"), facts.get("reference")
    require(isinstance(candidate, dict) and isinstance(reference, dict),
            "candidate and reference identities are required")
    require(candidate.get("branch") == "termrock-implementation", "wrong candidate branch")
    require(reference.get("branch") == "visual-baseline", "wrong reference branch")
    current = sha(candidate.get("head_sha"), "candidate.head_sha")
    require(sha(candidate.get("local_head_sha"), "candidate.local_head_sha") == current,
            "local candidate identity differs")
    require(sha(candidate.get("remote_head_sha"), "candidate.remote_head_sha") == current,
            "remote candidate identity differs")
    sha(reference.get("head_sha"), "reference.head_sha")

    authority = facts.get("visual_authority")
    require(isinstance(authority, dict), "visual authority is missing")
    require(authority.get("tag") == "visual-baseline"
            and authority.get("tag_object_sha") == TAG_OBJECT
            and authority.get("commit_sha") == TAG_COMMIT,
            "frozen visual tag identity changed")

    ci = facts.get("ci_observation")
    require(isinstance(ci, dict), "CI observation is missing")
    require(isinstance(ci.get("run_id"), str) and ci["run_id"].isdigit(),
            "CI run ID must be numeric text")
    require(isinstance(ci.get("workflow"), str) and ci["workflow"],
            "CI workflow must be text")
    require(ci.get("status") in {"queued", "in_progress", "completed"},
            "unknown CI run status")
    require(ci.get("conclusion") in {
        "action_required", "cancelled", "failure", "neutral", "success",
        "skipped", "stale", "timed_out", None,
    }, "unknown CI conclusion")
    sha(ci.get("head_sha"), "ci_observation.head_sha")
    timestamp(ci.get("created_at"), "ci_observation.created_at")
    require(type(ci.get("job_count")) is int and ci["job_count"] >= 0,
            "CI job count must be a nonnegative integer")
    require(type(ci.get("artifact_count")) is int and ci["artifact_count"] >= 0,
            "CI artifact count must be a nonnegative integer")
    require(
        ci.get("url")
        == "https://github.com/{}/actions/runs/{}".format(REPOSITORY, ci["run_id"]),
        "CI run URL does not match its ID",
    )
    failure_reason = ci.get("failure_reason")
    if failure_reason is not None:
        require(isinstance(failure_reason, dict), "CI failure reason must be an object")
        require(
            isinstance(failure_reason.get("reason"), str)
            and failure_reason["reason"].strip(),
            "CI failure reason text is required",
        )
        require(
            isinstance(failure_reason.get("source_url"), str)
            and failure_reason["source_url"].startswith("https://"),
            "CI failure reason source URL must be HTTPS",
        )

    dco = facts.get("dco_observation")
    require(isinstance(dco, dict), "DCO observation is missing")
    require(isinstance(dco.get("check_run_id"), str) and dco["check_run_id"].isdigit(),
            "DCO check ID must be numeric text")
    require(isinstance(dco.get("name"), str) and dco["name"], "DCO check name is required")
    require(dco.get("status") in {"queued", "in_progress", "completed"},
            "unknown DCO status")
    require(dco.get("conclusion") in {
        "action_required", "cancelled", "failure", "neutral", "success",
        "skipped", "stale", "timed_out", None,
    }, "unknown DCO conclusion")
    sha(dco.get("head_sha"), "dco_observation.head_sha")
    require(
        type(dco.get("affected_commit_count")) is int and dco["affected_commit_count"] >= 0,
        "DCO finding count must be a nonnegative integer",
    )
    require(
        isinstance(dco.get("details_url"), str)
        and dco["details_url"].startswith("https://"),
        "DCO details URL must be HTTPS",
    )

    ambient = facts.get("ambient_tools")
    require(isinstance(ambient, dict), "ambient tools are required")
    require(all(isinstance(ambient.get(field), str) and ambient[field]
                for field in ("python", "mise_rust", "cargo_nextest",
                              "nextest_command", "host")),
            "ambient tool observations must be nonempty text")
    return facts


def validate_tasks(records: Any) -> Mapping[str, Any]:
    require(isinstance(records, dict) and records.get("schema_version") == 1,
            "unsupported accepted-task schema")
    revision = records.get("queue_revision")
    require(type(revision) is int and revision > 0, "invalid queue revision")
    tasks = records.get("tasks")
    require(isinstance(tasks, list), "accepted tasks must be an array")
    seen = set()
    for task in tasks:
        require(isinstance(task, dict), "task record must be an object")
        work_id = task.get("work_id")
        require(isinstance(work_id, str) and work_id not in seen, "missing or duplicate work ID")
        seen.add(work_id)
        require(task.get("priority") in PRIORITY, "unknown task priority")
        require(task.get("state") in TASK_STATES, "unknown task state")
        require(all(isinstance(task.get(field), str)
                    for field in ("owner", "reviewer", "priority_reason")),
                "task owner, reviewer, and priority reason are required")
    return records


def cell(value: str) -> str:
    return value.replace("|", "\\|").replace("\n", " ")


def render_status(
    facts_value: Any, tasks_value: Any, role: str = "candidate"
) -> str:
    require(role in {"candidate", "reference"}, "role must be candidate or reference")
    facts, tasks = validate_facts(facts_value), validate_tasks(tasks_value)
    candidate, reference = facts["candidate"], facts["reference"]
    authority, ci, dco = facts["visual_authority"], facts["ci_observation"], facts["dco_observation"]
    ambient = facts["ambient_tools"]
    local_branch = candidate["branch"] if role == "candidate" else reference["branch"]
    local_role = "Candidate" if role == "candidate" else "Reference"
    queue_link = (
        "WORK_QUEUE.md"
        if role == "candidate"
        else "https://github.com/{}/blob/{}/WORK_QUEUE.md".format(
            REPOSITORY, candidate["branch"]
        )
    )
    ci_scope = ("matches" if ci["head_sha"] == candidate["head_sha"]
                else "does not match")
    ci_conclusion = ci["conclusion"] or "unknown"
    failure_reason = ci.get("failure_reason")
    if ci_conclusion == "failure":
        ci_note = (
            "Failure reason: {}; [source]({}).".format(
                cell(failure_reason["reason"]), failure_reason["source_url"]
            )
            if failure_reason is not None
            else "Failure reason is not recorded."
        )
    else:
        ci_note = "Run status: {}.".format(ci["status"])
    dco_conclusion = dco["conclusion"] or "unknown"
    affected_commits = dco["affected_commit_count"]
    if affected_commits == 0:
        dco_findings = "No commits are reported with sign-off problems."
    elif affected_commits == 1:
        dco_findings = "1 commit is reported with sign-off problems."
    else:
        dco_findings = "{} commits are reported with sign-off problems.".format(
            affected_commits
        )
    command_rows = "\n".join(
        "| {} | {} | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_APPLICABLE; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN |".format(
            run_id, command
        )
        for run_id, command in COMMANDS
    )
    app_rows = "\n".join(
        "| {} | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN | NOT_APPLICABLE | NOT_RUN | NOT_APPLICABLE | NOT_RUN | No current run receipt. |".format(app)
        for app in APPS
    )

    open_tasks = [task for task in tasks["tasks"] if task["state"] != "verified"]
    if open_tasks:
        highest = min(open_tasks, key=lambda task: PRIORITY[task["priority"]])["priority"]
        task_rows = "\n".join(
            "| {} | {} | {} | {} | {} |".format(
                cell(task["work_id"]), cell(task["state"]), cell(task["owner"]),
                cell(task["reviewer"]), cell(task["priority_reason"]),
            )
            for task in sorted(
                (task for task in open_tasks if task["priority"] == highest),
                key=lambda task: task["work_id"],
            )
        )
        task_section = "\n".join(
            [
                "Highest open priority: **{}**, from accepted queue revision {}.".format(
                    highest, tasks["queue_revision"]
                ),
                "",
                "| Work | State | Owner | Reviewer | Priority reason |",
                "| --- | --- | --- | --- | --- |",
                task_rows,
                "",
                "See [WORK_QUEUE.md]({}) for dependencies and claim details.".format(
                    queue_link
                ),
            ]
        )
    else:
        task_section = "No open accepted task is recorded. See [WORK_QUEUE.md]({}).".format(
            queue_link
        )

    return """# Termrock status

| Conclusion | State | Evidence |
| --- | --- | --- |
| Visibility / Complete | NOT_RUN | The shared suite, paired results, and publication checks have not been accepted as a complete current run. |
| Refactor / Ready | NOT_RUN | No complete current evidence covers required visual, interaction, API, ownership, and review checks. |
| Reference / Qualified | NOT_RUN | The tag identity is recorded, but no paired reference execution is qualified. |
| Command / Ready | NOT_RUN | No exact root command has current execution evidence. |
| Evidence freshness | NOT_RUN | No validated paired run receipt is available for the measured source pair. |

## Required commands

Each result cell shows reference and candidate separately. Ready has its own column, separate from Build, Launch, First frame, input and interaction, Exit, Restoration, Visual, and Ownership.
Reference ownership is NOT_APPLICABLE because the reference is not required to use the candidate Termrock architecture.

| ID | Exact root command | Build | Launch | First frame | Interaction | Exit | Restoration | Visual | Ownership | Ready |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
{command_rows}

This bootstrap was generated from measured facts in [source-facts.json](tools/visibility/source-facts.json). A run receipt is a record of execution results tied to an exact source pair and required test set. No validated paired run receipt is available, so this report cannot show a product pass. Product requirements remain in [CHECKLIST.md](CHECKLIST.md) and [checklist.json](checklist.json).

## Measured source identity

| Identity | Value |
| --- | --- |
| Local role | {local_role} ({local_branch}) |
| Candidate observed commit | [{candidate_sha}](https://github.com/{repository}/commit/{candidate_sha}) |
| Reference observed commit | [{reference_sha}](https://github.com/{repository}/commit/{reference_sha}) |
| Immutable visual tag object | {tag_object} |
| Immutable visual tag commit | [{tag_commit}](https://github.com/{repository}/commit/{tag_commit}) |
| Facts observed at | {observed_at} |
| Report commit | Set after this report is committed; the generated page cannot include its own commit ID. |
| Shared suite / case-set / expected-generation digests | NOT_RECORDED |

## Current CI and repository checks

| Check | Observation | Scope |
| --- | --- | --- |
| [CI run {ci_id}]({ci_url}) | {ci_conclusion} at {ci_sha}; {ci_jobs} jobs; {ci_artifacts} artifacts. {ci_note} | The run head {ci_scope} the measured candidate SHA. This workflow result is not a product test result. |
| [DCO check {dco_id}](https://github.com/{repository}/commit/{dco_sha}/checks) | {dco_conclusion}; {dco_findings} | PR gate status; separate from product results. Existing history is unchanged. |

Available in this environment: Python {python}, Rust {rust}, and cargo-nextest {nextest} through mise on {host}. This records installed tools only; it does not show that a product check ran.

## Applications

Visual, interaction, API, and ownership remain separate. API and ownership are candidate-only obligations.

| Application | Reference visual | Candidate visual | Reference interaction | Candidate interaction | Reference API | Candidate API | Reference ownership | Candidate ownership | Evidence |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
{app_rows}

## Component and coverage inventory

The required shared-case and per-component checkpoint sets are Unknown until the case inventory is audited and bound to the common suite. component-ownership.json is a proposal map, not an executed checkpoint set. No per-component result is inferred from imports, checklist text, or historical captures.

No compatible previous paired run is recorded, so trend changes are not measured.

## Next work

{task_section}

## Evidence navigation

- [Work queue]({queue_link})
- [Implementation PR #17](https://github.com/{repository}/pull/17)
- [Current CI run]({ci_url})
- [Product checklist](CHECKLIST.md)
- [Source facts and observation commands](tools/visibility/README.md)

If no current run receipt is present, the product result remains NOT_RUN. NOT_RUN is a result status, not a run-receipt status. No historical pass is promoted to current evidence.
""".format(
        command_rows=command_rows,
        candidate_sha=candidate["head_sha"],
        reference_sha=reference["head_sha"],
        local_role=local_role,
        local_branch=local_branch,
        queue_link=queue_link,
        repository=REPOSITORY,
        tag_object=authority["tag_object_sha"],
        tag_commit=authority["commit_sha"],
        observed_at=facts["observed_at"],
        ci_id=ci["run_id"],
        ci_url=ci["url"],
        ci_conclusion=ci_conclusion,
        ci_note=ci_note,
        ci_sha=ci["head_sha"],
        ci_jobs=ci["job_count"],
        ci_artifacts=ci["artifact_count"],
        ci_scope=ci_scope,
        dco_id=dco["check_run_id"],
        dco_sha=dco["head_sha"],
        dco_conclusion=dco_conclusion,
        dco_findings=dco_findings,
        python=ambient["python"],
        rust=ambient["mise_rust"],
        nextest=ambient["cargo_nextest"],
        host=ambient["host"],
        app_rows=app_rows,
        task_section=task_section,
    )


def load(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise ValueError("cannot read {}: {}".format(path, error)) from error


def main(argv: Optional[Sequence[str]] = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    modes = parser.add_mutually_exclusive_group()
    modes.add_argument("--write", action="store_true", help="write generated STATUS.md")
    modes.add_argument("--check", action="store_true", help="compare STATUS.md with current inputs")
    parser.add_argument(
        "--role",
        choices=("candidate", "reference"),
        default="candidate",
        help="identify this checkout's local presentation role (default: candidate)",
    )
    args = parser.parse_args(argv)
    try:
        output = render_status(load(FACTS_PATH), load(TASKS_PATH), role=args.role)
    except ValueError as error:
        print("status generation failed: {}".format(error), file=sys.stderr)
        return 2
    if args.write:
        STATUS_PATH.write_text(output, encoding="utf-8")
        print("wrote {}".format(STATUS_PATH.relative_to(ROOT)))
    elif args.check:
        try:
            current = STATUS_PATH.read_text(encoding="utf-8")
        except OSError as error:
            print("status check failed: {}".format(error), file=sys.stderr)
            return 1
        if current != output:
            print("STATUS.md is stale; run python3 tools/visibility/status.py --write",
                  file=sys.stderr)
            return 1
        print("STATUS.md matches measured inputs")
    else:
        sys.stdout.write(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
