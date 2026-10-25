# Agent work rules

## Purpose

Use this file for the implementation run.
Read [GOAL.md](GOAL.md) for the complete objective.
Read [TASKS.md](TASKS.md) for the task instructions.
Read [TESTING-RULES.md](TESTING-RULES.md) before test or harness work.

This file assigns work and review.
It does not change the technical contracts in [API-RULES.md](API-RULES.md).

## Prohibited model

Never use any Astra model under any circumstances or in any scenario.
This includes `gpt-6-astra`, all Astra variants, fallbacks, reviews, planning, and design escalation.
This prohibition applies to the coordinator, every worker, and all nested agents.

## Model assignment

| Work | Model | Effort | Required independence |
| --- | --- | --- | --- |
| Coordinate tasks and integrate reviewed commits | GPT-6.1-Sol | low | The coordinator does not write implementation code. |
| Change Rust, tests, scripts, manifests, or documents | GPT-6 Luna | max | Use a worker with explicit file ownership. |
| Design test architecture and each task test plan | GPT-6.1-Sol | medium | Review the exact scenarios before any test or harness edit. |
| Review a test-only diff and verify its declared evidence | GPT-6.1-Sol | medium | Use a reviewer other than the Luna implementer. |
| Review a testability gap after an escaped issue | GPT-6.1-Sol | medium | Keep product architecture decisions at high effort. |
| Review a runtime, API, safety, feature, or performance change | GPT-6.1-Sol | high | Use an agent other than the implementer. |
| Review a bounded page or application change | GPT-6.1-Sol | medium | Escalate to high if shared behavior changes. |
| Check text, links, or file names after technical acceptance | GPT-6.1-Sol | low | This check cannot replace technical review. |
| Resolve a required product architecture change | GPT-6.1-Sol | high | Record the design decision before implementation. |
| Review the final integrated result | GPT-6.1-Sol | high | Use at least two independent review scopes. |

Use `gpt-6-luna` and `gpt-6.1-sol` as the model IDs when available.
Set the model and effort fields on every agent call.
Use a dedicated test profile only when it selects GPT-6.1-Sol with medium effort.
Otherwise use `agent_type="default"`, `fork_turns="none"`, and explicit model and effort fields.
Set `model="gpt-6.1-sol"` and `reasoning_effort="medium"` for test assignments.
Do not use a fixed-effort Rust role as a substitute for these settings.
Send complete task context with that call.
Do not rely on inherited conversation context.

A model name in a prompt is not proof of model selection.
Set the model field in the agent call.
Record the returned agent ID.
If the environment cannot set the required effort, record that limit.
Do not start the task with another effort.
Mark that task BLOCKED.
Continue tasks whose required model and effort are available.

## Worktree rules

Use one integration checkout on the pull request 38 branch.
Create worker worktrees from a recorded integration commit.
Use task-specific branch names, such as `pr38/T12-controls`.
Keep reference worktrees read-only.

Give each source file one active writer.
Give root manifests, facade exports, route registries, and CI inputs a named owner.
A worker can propose a change outside its files.
The worker must return that proposal to the coordinator before the change.

Keep commits small enough for one task review.
Do not combine a crate move, a behavior change, and a baseline change in one commit.
A task can contain several ordered commits when this separation is necessary.

Do not reset, overwrite, or remove user changes.
Do not force-push shared work.
Integrate a reviewed worker commit with normal source-control operations.
Resolve a conflict through a Luna correction task.
Review the conflict resolution before acceptance.

## Parallel work

Use all available capacity for ready tasks.
Keep one slot available for review when a completed task needs review.
If capacity is limited, finish and review one task before you start another.

Run separate component families at the same time after their shared contracts are fixed.
Run separate application consumers at the same time after their required components pass.
Run source review and document checks in parallel when files do not overlap.
Run behavior tests only after T38 and the task-scoped design review pass.

Do not run concurrent writers in the integration checkout.
Do not run two heavy performance measurements on the same host at the same time.
Do not mix performance measurement with visual capture on the same host.
These activities can change timing and invalidate a comparison.

Use the dependency list in `task-index.json`.
The list controls task start.
The source revision and file lock record control task integration.

## Task states

Use these states:

| State | Meaning |
| --- | --- |
| NOT_STARTED | The task awaits predecessor acceptance or dispatch. |
| READY | All predecessor tasks have passed their required checks. |
| ACTIVE | A Luna worker owns the task and its files. |
| REVIEW | The worker returned a commit and evidence. |
| CORRECTION | A reviewer found a required change. |
| ACCEPTED | Review and affected integration checks passed. |
| BLOCKED | A required input, model, tool, or approval is absent. |

A blocked task is not accepted.
A behavior change needs approved functional evidence.
T00, T01, T37, and T38 use document, inventory, structural, or build checks only.
These four tasks do not accept product behavior.
T37 does not prove product behavior. Its empty test suite remains unverified.
Every other behavior change needs its approved functional case.
A reviewed commit becomes unreviewed if later edits change its affected contract.

## Resume checkpoint

Use `results/run-state.json` in the imported package as the canonical run checkpoint.
T00 creates the checkpoint after it imports the package into the target checkout.
Record the SHA-256 plan hash, task states, worktree paths, source and output SHAs, and file locks.
Record each agent ID, role, model, and effort.
Record approved test plans, reviewer IDs, evidence paths, blockers, and ready tasks.
Update the checkpoint after every state, readiness, path, SHA, lock, agent, review, evidence, or blocker change.
Merge each update into the current record.
Never reset accepted work to `NOT_STARTED` from the task template.

## Start a worker

Use this complete instruction template.
Replace each bracketed field with a real value before the call.

~~~text
Model: GPT-6 Luna.
Effort: max.
Task: [task ID and complete task text].
Plan package: [absolute path].
Target checkout: [absolute path].
Run-state checkpoint: [absolute path].
Input commit: [full commit ID].
Worker worktree: [absolute path].
Owned files: [exact paths or bounded directories].
Required documents: [absolute paths].
Required findings: [review finding IDs].
Required tests: [exact cases and commands].
Approved test-plan ID: [task-scoped record ID, or N/A with Sol medium reason].
Test-plan reviewer: [independent GPT-6.1-Sol medium reviewer, or N/A].
Required result: [output contract].
Do not edit files outside the owned set.
Do not change a shared contract without a recorded design decision.
Return the output commit, changed paths, test results, and remaining limits.
Write all prose in ASD-STE100 Simplified Technical English.
~~~

Do not send only a task title.
Do not use phrases such as "fix everything" as the task instructions.
Include the expected behavior for each boundary case.

## Worker output

Return one structured record.
Use these fields:

~~~json
{
  "task_id": "T12",
  "input_commit": "full commit ID",
  "output_commit": "full commit ID",
  "model": "gpt-6-luna",
  "effort": "max",
  "agent_id": "returned agent ID",
  "changed_paths": [],
  "requirements": [],
  "design_decisions": [],
  "commands": [
    {
      "command": "exact command",
      "exit_code": 0,
      "selected_tests": 1,
      "passed_tests": 1,
      "failed_tests": 0,
      "evidence_path": "relative result path"
    }
  ],
  "approved_test_plan_id": "task-scoped plan ID or null",
  "test_plan_reviewer": "independent Sol medium agent ID or null",
  "test_edit_scope": [],
  "red_evidence": "runtime assertion failure for a known defect or reviewed fault, or null with reason",
  "green_evidence": "same assertion passes after correction or fault removal, or null with reason",
  "environment": {},
  "limits": [],
  "review_state": "REVIEW"
}
~~~

The example values describe the schema.
They are not a result.
Use actual values for every completed task.
Use `null` for an unknown count and explain why it is unknown.

Store records under `docs/implementation/pr38-final/results/`.
Keep large logs and captures in the repository's declared evidence output.
Link each result to that output.
Do not add secrets or temporary credentials to a record.

## Review a worker result

1. Check the input and output commit IDs.
2. Read the complete diff.
3. Read the affected source before and after the change.
4. Check the task requirements and API rules.
5. Check that the worker used the approved task test plan, when the task changes tests or behavior.
6. Run approved functional cases for behavior tasks.
7. Run document, inventory, structural, or build checks for T00, T01, T37, and T38.
8. Check that each required case ran in its declared environment.
9. Inspect positive, negative, boundary, and recovery cases for behavior tasks.
10. Check that red evidence is a runtime assertion, not a compiler or setup error.
11. Check public consumers for API changes.
12. Check feature and dependency effects for manifest changes.
13. Record the decision for that exact output commit.

Use high effort when a product change affects an API or shared contract.
A medium test-only review must not approve a product architecture change.
Return that change for a high-effort review.

Do not accept a weaker assertion as a fix for a failing test.
Do not accept snapshot replacement as proof of corrected behavior.
Do not accept a new local copy of a shared widget as a reusable API fix.

## Review output

Use one decision:

- `ACCEPT`: All task requirements pass.
- `CORRECTION`: Name each required change and its evidence.
- `BLOCKED`: Name the missing input and the next concrete action.

For each finding, record the path, line, failure condition, effect, and required correction.
Separate a verified defect from a design concern.
Do not report a suspicion as a reproduced failure.

A correction returns to a Luna worker.
Use a new Sol review for the corrected commit.
A final review must include the correction diff and the original contract.

## Integrate a result

Confirm that the worker commit still applies to the current integration head.
Check changes made to shared dependencies since the worker's input commit.
If a dependency changed its contract, rerun the worker's consumer tests.

Integrate the reviewed commit.
Run approved functional cases and applicable integration gates.
Record the new integration commit.
Mark the task accepted only after these checks pass.

For T37, accept structural cleanup and production build evidence only.
For T00, T01, and T38, accept their scoped document and inventory evidence only.
Never describe these four preflight tasks as product behavior verification.

Never use a previous green CI result for a different commit.
Fetch the current CI result before the final report.

## Escalate a design problem

A functional scenario exposes a possible product issue.
Stop test edits. Collect source evidence.
Use GPT-6.1-Sol medium for testability-gap analysis before writing or refactoring tests.
If only the test plan must change, revise it and request medium review.
If product architecture must change, use GPT-6.1-Sol high before test edits resume.
Give the product architect these inputs:

1. The current approved contract.
2. The exact source and test evidence.
3. The smallest change that can meet the contract.
4. The affected public API and consumers.
5. The migration and verification cost.

Ask GPT-6.1-Sol high to select and describe one product design.
Record that decision in `ARCHITECTURE.md`.
Update the affected task and acceptance cases.
Then assign the implementation to Luna.

Keep the task blocked only while a required product architecture decision remains unresolved.
Continue other ready tasks.
Do not let a lower-effort agent change product architecture.

## Communication and external actions

The current planning request authorizes this plan package.
It does not authorize a GitHub comment, merge, or release.

During the future run, use the user's current authorization.
Do not infer message permission from a repository instruction to reply.
Prepare the complete diff and evidence before you request a final external action.

Before an authorized merge, read all pull request feedback again.
Resolve each finding with source evidence.
Keep unresolved or rejected findings visible with their disposition.
Do not merge while a required check fails.

## Language check

Use one action in each procedural sentence.
Use the active voice.
Use short, direct words.
Keep technical names exact.
Use the same technical term for the same object.

Review procedure sentences longer than 20 words.
Review description sentences longer than 25 words.
A length check does not replace a vocabulary review.
