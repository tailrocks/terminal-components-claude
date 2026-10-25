# Termrock: finish pull request 38

Date: 26 October 2026.
Review target: `e08ac57d10ae54c87337eae0be08b1f5e8f4bda4`.
Design reference: `5b3dbd7ae799f325efbb4f4cc11b2f5518dec948`.

## Result

Use pull request 38 as the implementation branch.
Keep its useful crate structure and runtime contracts.
Use pull request 26 as the source for appearance and interaction.
Port required behavior through approved functional probes.

This package contains a plan. It contains no completed Termrock source changes.
All 39 tasks remain future work.

## Prohibited model

Never use any Astra model under any circumstance or scenario.
This rule applies to planning, implementation, review, fallback, and every agent depth.

## Start the work

1. Read [REVIEW.md](REVIEW.md), [ARCHITECTURE.md](ARCHITECTURE.md), and [API-RULES.md](API-RULES.md).
2. Read [TESTING-RULES.md](TESTING-RULES.md) and [AGENT-RUNBOOK.md](AGENT-RUNBOOK.md).
3. Run T00 as a read-only source capture, except for importing approved instruction rules.
4. Run T37 as the first Termrock source change. Remove the legacy test suite and nonfunctional comments.
5. Run T01 after cleanup. Run T38 after T01 and before implementation work.
6. Require task-scoped independent review before each test or harness change.
7. Then follow the dependency graph in [PLAN.md](PLAN.md) and [task-index.json](task-index.json).

Historical source, tags, approved oracle evidence, and immutable references stay outside the active suite.
The empty suite after T37 is an unverified migration state. It is never a pass.

## Files

| File | Purpose |
| --- | --- |
| [GOAL.md](GOAL.md) | Complete prompt for the future work run. |
| [REVIEW.md](REVIEW.md) | Source findings, branch decisions, and review limits. |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Crate owners, dependency rules, API layers, and migration decisions. |
| [API-RULES.md](API-RULES.md) | Rules for each implementation agent. |
| [TESTING-RULES.md](TESTING-RULES.md) | Approved functional test architecture and proof rules. |
| [PLAN.md](PLAN.md) | Work order, dependencies, completion gates, and risk controls. |
| [TASKS.md](TASKS.md) | Exact instructions for Luna implementers and Sol reviewers. |
| [COMPONENT-MATRIX.md](COMPONENT-MATRIX.md) | Component checks, Showcase routes, and application journeys. |
| [VALIDATION.md](VALIDATION.md) | Candidate commands, functional proof, performance checks, and evidence rules. |
| [AGENT-RUNBOOK.md](AGENT-RUNBOOK.md) | Model selection, delegation, review, and task records. |
| [SOURCES.md](SOURCES.md) | Fixed source links, references, and the technical word list. |
| [task-index.json](task-index.json) | Task IDs, dependencies, owners, and model assignments. |
| [AGENTS.patch](AGENTS.patch) | Root instruction links for the reviewed target. |
| [EXPORT-CHECKS.json](EXPORT-CHECKS.json) | Checks of this plan package and its evidence limits. |
| [SHA256SUMS.txt](SHA256SUMS.txt) | Hashes for the exported files. |

## Plan limits

Task commands name candidate targets. They are not instructions to preserve or run deleted legacy targets.
Use them only after the approved test plan names the new target and the task review accepts it.
Green evidence applies only to declared cases, source, tools, and environment.
No task or package check claims that Termrock source work has passed.

## Completion rule

Complete every required task.
Pass every applicable gate at one final source revision.
Leave no required component, route, or application journey without evidence.
A blocked gate or a required case that is absent, skipped, or unselected keeps the goal open.

This goal permits local implementation and verification.
Follow the user's current authorization for commits, pushes, comments, merges, and releases.
The planning request does not authorize an external message, merge, or package release.

## Language

Use ASD-STE100 rules for new prose.
Use the technical terms in [SOURCES.md](SOURCES.md).
Keep code, paths, command options, and model IDs exact.

A sentence-length check supports language review.
It does not certify full ASD-STE100 conformity.
