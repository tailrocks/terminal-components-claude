# Agent work rules

## Purpose

Use this file for the implementation run.
Read [GOAL.md](GOAL.md) for the complete objective.
Read [TASKS.md](TASKS.md) for the task instructions.

This file assigns work and review.
It does not change the technical contracts in [API-RULES.md](API-RULES.md).

## Prohibited model

Never use any Astra model under any circumstances or in any scenario.
This includes `gpt-6-astra`, all Astra variants, fallbacks, reviews, planning, and design escalation.
This prohibition applies to the coordinator, every worker, and all nested agents.

## Model assignment

| Work | Model | Effort | Required independence |
| --- | --- | --- | --- |
| Coordinate tasks and integrate reviewed commits | GPT-6.1-Sol | high | The coordinator does not write implementation code. |
| Change Rust, tests, scripts, manifests, or documents | GPT-6 Luna | max | Use a worker with explicit file ownership. |
| Review a runtime, API, safety, feature, or performance change | GPT-6.1-Sol | high | Use an agent other than the implementer. |
| Review a bounded page or application change | GPT-6.1-Sol | medium | Escalate to high if shared behavior changes. |
| Check text, links, or file names after technical acceptance | GPT-6.1-Sol | low | This check cannot replace technical review. |
| Resolve a required architecture change | GPT-6.1-Sol | high | Record the design decision before implementation. |
| Review the final integrated result | GPT-6.1-Sol | high | Use at least two independent review scopes. |

Use `gpt-6-luna` and `gpt-6.1-sol` as the model IDs when available.
Use `fork_turns="none"` when the agent API requires it for a model override.
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
Run source review, behavior tests, and document checks at the same time when they do not share mutable files.

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
A source change without a test result is not accepted.
A reviewed commit becomes unreviewed if later edits change its affected contract.

## Start a worker

Use this complete instruction template.
Replace each bracketed field with a real value before the call.

~~~text
Model: GPT-6 Luna.
Effort: max.
Task: [task ID and complete task text].
Input commit: [full commit ID].
Worktree: [absolute path].
Owned files: [exact paths or bounded directories].
Required documents: [absolute paths].
Required findings: [review finding IDs].
Required tests: [exact cases and commands].
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
5. Run the focused tests independently.
6. Check that each test selected at least one case.
7. Inspect at least one meaningful negative case.
8. Check public consumers for API changes.
9. Check feature and dependency effects for manifest changes.
10. Record the decision for that exact output commit.

Use high effort when the change affects an API or shared contract.
A medium review must not approve an unplanned architecture change.
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
Run affected package tests and the applicable integration gates.
Record the new integration commit.
Mark the task accepted only after these checks pass.

Never use a previous green CI result for a different commit.
Fetch the current CI result before the final report.

## Escalate a design problem

A new abstraction needs a concrete consumer and a failed contract.
Give GPT-6.1-Sol high these inputs:

1. The current approved contract.
2. The exact source and test evidence.
3. The smallest change that can meet the contract.
4. The affected public API and consumers.
5. The migration and verification cost.

Ask GPT-6.1-Sol high to select and describe one design.
Record that decision in `ARCHITECTURE.md`.
Update the affected task and acceptance cases.
Then assign the implementation to Luna.

Until GPT-6.1-Sol high resolves the design question, keep the affected task blocked.
Continue other ready tasks.
Do not let a lower-effort agent silently change the design.

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
