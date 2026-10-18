# Task-format source assessment

This is planning evidence, not an implementation task. Task decomposition must follow the completed historical, component, and application matrices.

## Pinned authority

On 2026-09-11, `git ls-remote git@github.com:donbeave/task-format.git refs/heads/main` returned `52d9f1eb7721f409bc47beb9fced7997b5c13ede`. The local source checkout has that HEAD. The installed CLI reports `taskfmt 0.2.0 (git 52d9f1eb7721f409bc47beb9fced7997b5c13ede)`.

Authoritative files read at that revision:

- `README.md` and `docs/monitoring.md`.
- `reference/task-template/README.md`, `AGENTS.md`, and `verify.toml`.
- `harness/src/verifycfg.rs` and the package validation entry points in `harness/src/lint.rs`.

The source checkout contains unrelated user changes to its project catalog. They are not planning inputs and must not be changed. Read-only inspection of the canonical template and implementation is sufficient; no task-format repository mutation is authorized by this goal.

`git diff --exit-code HEAD` for the listed canonical source paths returned zero. Canonical template SHA-256 values are `53f47a2ffb15744ab005245d463e5a984498885f33353e7a9a3c8a671f42f464` (README), `08d3799b3165cd7cd12dedaed8d073920d57ff75a716ae625ad1e18066350df0` (AGENTS), and `2c448ebbd036ae51823f5e505a42d97fa5e76550ef08a61740c940193fe0c085` (verify.toml).

## Canonical package and catalog

Use `refactoring-tasks/` as the catalog root, `terminal-components/` as project, and `completion/` as group. Task directories use three-digit numbers. Each package contains `README.md`, `AGENTS.md`, `verify.toml`, `task.toml`, and any planner-owned `trusted/` assets. Project and group READMEs require nonempty H1 names.

README frontmatter is exactly the canonical `task/v5` contract with `id: TASK-NNN`, title, and supported kind. Its sections are Goal, Context, Preconditions, Scope, Requirements, Acceptance criteria, Fixed decisions, and Checklist. Additional explanatory text must not replace canonical sections.

Each non-gate acceptance block contains exactly one constrained Gherkin fence, then a typed Verification block with Type, Covers, and Check. Gate acceptance contains Type and Check only. The requirement/check/acceptance/checklist graph is validated by the canonical CLI. Commands and expected outputs belong in `verify.toml`, not in an acceptance block.

`task.toml` uses only the three supported fields:

```toml
schema = "task-meta/v1"
status = "pending"
dependencies = ["terminal-components/completion/001"]
```

Dependencies are fully qualified catalog IDs. No invented `soft_dependencies`, wave, owner, or concurrency keys are permitted. Soft ordering, shared-file serialization, and integration scheduling belong in the plan and task prose. Dependency eligibility does not manufacture the predecessor repository tree. Dispatch must prepare the recorded integrated parent containing every hard predecessor.

## Machine verification

`verify/v2` supports `task_id`, optional `base_tree`, optional single `predecessor.task_id`, nonempty `writable_paths`, optional `forbidden_paths`, optional regex `forbidden_patterns`, and checks. The catalog sidecar carries the complete DAG; the single predecessor field cannot express a multi-parent dependency join.

Each check has an ID, phase (`precondition`, `focused`, `regression`, `lint`, or `gate`), exactly one of argv or shell, requirement and acceptance ID arrays, and expected results. Exactly one check has phase `gate`. A check must not recursively invoke `taskfmt verify`. Supported matchers include exit code, stdout/stderr inclusion/exclusion/regex/counts, and required/forbidden artifacts. Paths must be safe repository-relative paths.

No future predecessor SHA may be fabricated. Taskfmt lifecycle gates use their recorded run base. The execution plan must distinguish the pinned source oracle, architecture starting commit, and each actual task scope base.

`gate.rs::glob_regex` treats a bare writable path as that file or directory subtree; `*` and `**` both cross directory separators. Prefer explicit paths over broad wildcard scope. Forbidden paths are checked for changes against the recorded base, not forbidden existence. The gate rejects hidden Git index entries that would conceal modifications. It runs checks in their declared order and evaluates scope independently of command success.

## Trusted evidence boundary

The task package is mounted read-only at `/task`; edits occur under `/work`. Trusted assets overlay the execution tree outside writable paths. The host freezes a candidate tree, runs the authoritative checks against that tree, and records a verdict. Candidate-authored progress and screenshots cannot substitute for the host verdict.

Oracle inputs, reference commit, scenario membership, normalization rules, comparison code, and expected output hashes must be planner-owned protected inputs. Merely excluding a snapshot directory while leaving the invoked comparison script writable is insufficient. The final packages must protect the whole proof chain and require zero missing, skipped, errored, or mismatched scenarios.

The current taskfmt dispatcher and promotion lifecycle hardcode `main`; they are prohibited for this campaign. Use the supported standalone `taskfmt verify` gate in isolated integration worktrees. The separately qualified host prepares trusted overlays, freezes source, verifies exact trees and advances only the named local integration ref with an expected-parent lease. It does not push or merge into `main`. This planning goal authorizes no execution or promotion of terminal-components tasks. See [proof-contract.md](proof-contract.md) for the complete boundary.

## Planning validation commands

Run the installed pinned CLI against the completed catalog:

```sh
taskfmt project lint terminal-components --projects-root /absolute/path/to/refactoring-tasks --config /absolute/pinned/task-format/experiment.toml
taskfmt project show terminal-components --projects-root /absolute/path/to/refactoring-tasks --config /absolute/pinned/task-format/experiment.toml --json
taskfmt lint /absolute/path/to/refactoring-tasks/terminal-components/completion/001 --config /absolute/pinned/task-format/experiment.toml
```

Even catalog-only commands currently require an experiment configuration. During this planning session the explicit path was `/Users/donbeave/Projects/donbeave/task-format/experiment.toml`; future operators must provide the accepted pinned checkout's configuration, not rely on an unrelated current directory. Schema validation proves package structure and references, not test sufficiency. Independent reviewers must additionally inspect command existence, coverage, trust boundaries, task scope, and actual dependency integration.
