# Termrock: finish pull request 38

Date: 26 October 2026.
Review target: `e08ac57d10ae54c87337eae0be08b1f5e8f4bda4`.
Design reference: `5b3dbd7ae799f325efbb4f4cc11b2f5518dec948`.

## Result

Use pull request 38 as the implementation branch.
Keep its useful crate structure and runtime contracts.
Use pull request 26 as the source for appearance and interaction.
Port useful behavior through the target API.

This package contains a plan. It does not contain completed Rust changes.

## Prohibited model

Never use any Astra model under any circumstances or in any scenario.
This includes `gpt-6-astra`, all Astra variants, fallbacks, reviews, planning, and design escalation.
This prohibition applies to the coordinator, every worker, and all nested agents.

## Start the work

1. Read [REVIEW.md](REVIEW.md).
2. Copy this folder to `docs/implementation/pr38-final/` in the target checkout.
3. Apply [AGENTS.patch](AGENTS.patch) after you check its target.
4. Read [ARCHITECTURE.md](ARCHITECTURE.md) and [API-RULES.md](API-RULES.md).
5. Give [GOAL.md](GOAL.md) and this complete folder to GPT-6.1-Sol.
6. Ask GPT-6.1-Sol to execute the goal.
7. Keep the files available to each agent.

The patch adds links to the existing root `AGENTS.md`.
It preserves the existing rules.
The patch does not change Rust source.

If the branch head changes, execute task T00 before you use this review.
Do not treat these commit IDs as the latest IDs for a later run.

## Files

| File | Purpose |
| --- | --- |
| [GOAL.md](GOAL.md) | The complete prompt for the future /goal run. |
| [REVIEW.md](REVIEW.md) | Source findings, branch decisions, and review limits. |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Crate owners, allowed dependencies, API layers, and migration decisions. |
| [API-RULES.md](API-RULES.md) | Rules that each implementation agent must follow. |
| [PLAN.md](PLAN.md) | Work order, dependencies, completion gates, and risk controls. |
| [TASKS.md](TASKS.md) | Exact task instructions for Luna implementers and Sol reviewers. |
| [COMPONENT-MATRIX.md](COMPONENT-MATRIX.md) | Component checks, missing Showcase routes, and application journeys. |
| [VALIDATION.md](VALIDATION.md) | Commands, test requirements, performance checks, and evidence rules. |
| [AGENT-RUNBOOK.md](AGENT-RUNBOOK.md) | Model selection, delegation, review, and task records. |
| [SOURCES.md](SOURCES.md) | Fixed source links, external references, and the technical word list. |
| [task-index.json](task-index.json) | Task IDs, dependencies, ownership, and model assignments. |
| [AGENTS.patch](AGENTS.patch) | Root instruction links for the reviewed target. |
| [EXPORT-CHECKS.json](EXPORT-CHECKS.json) | Checks of this plan package and its evidence limits. |
| [SHA256SUMS.txt](SHA256SUMS.txt) | Hashes for the exported files. |

## Plan package checks

The 37-task graph has no cycle.
The final task depends on all required tasks through that graph.
Every implementation task specifies Luna max.
Every technical review specifies Sol medium or high.
All 45 component IDs and 24 missing-page assignments are present.
All local document links resolve.
The root patch passes its apply check at the reviewed target.

These checks apply to the plan package.
They do not report completed implementation or passing Rust tests.

## Completion rule

Complete every required task.
Pass every applicable gate at the same final source revision.
Leave no required component, route, or application journey without evidence.
A blocked gate keeps the goal open.

This goal permits local implementation and verification.
Follow the user's current authorization for commits, pushes, comments, merges, and releases.
The planning request alone does not authorize an external message, a merge, or a package release.

## Language

Use ASD-STE100 rules for new prose.
Use short sentences and the active voice.
Use the technical terms in [SOURCES.md](SOURCES.md).
Keep code, paths, command options, and model IDs exact.

A sentence-length check supports the language review.
It does not certify dictionary use or complete ASD-STE100 conformity.

