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

## Launch from this directory

Start a new Codex session with the complete goal as its first prompt:

~~~sh
cd /Users/donbeave/Downloads/termrock-pr38-plan
codex --strict-config -m gpt-6.1-sol -c 'model_reasoning_effort="low"' "$(cat GOAL.md)"
~~~

This command starts the implementation run. The setup work does not execute that run.
The quoted substitution passes the full goal text as one argument.
The launch directory is `PLAN_ROOT`.
Its Git origin is `donbeave/termrock-pr38-plan`; it contains no Rust product workspace.

[.codex/config.toml](.codex/config.toml) selects the Sol low coordinator and Luna max worker default.
Project settings apply when Codex loads this trusted project.
Keep normal host permissions and trust controls.
On first use, accept normal Codex trust for this reviewed folder.
Do not broaden trust beyond the reviewed repository or disable the sandbox.
The launch command pins the coordinator model and effort.
It does not prove account access or worker model selection.

Local setup verification used Codex CLI `0.164.0`.
Strict configuration startup passed after this exact launch folder was trusted.
Ancestor trust alone did not load the project settings.
`codex debug prompt-input` exposed all four custom profiles.
These checks verify configuration loading, not Termrock product behavior.

The project has four [custom agent profiles](.codex/agents/):

| Profile | Model and effort | Scope |
| --- | --- | --- |
| `termrock_implementer` | `gpt-6-luna`, max | All implementation in assigned files. |
| `termrock_reviewer` | `gpt-6.1-sol`, medium | Bounded product reviews and test diff reviews. |
| `termrock_architect` | `gpt-6.1-sol`, high | Product architecture and shared product contract reviews. |
| `termrock_test_architect` | `gpt-6.1-sol`, medium | T38, task test plans, and escaped testability gaps. |

Profiles follow the [official custom agent format](https://learn.chatgpt.com/docs/agent-configuration/subagents).
Select the required model and effort in each agent tool call.
Use a custom profile only when the runtime exposes it with the required settings.
Otherwise use an available general role with explicit model and effort.
Do not use a fixed role that silently changes the required effort.
Record each returned agent ID and any selection limit.
Block unavailable assignments; continue independent ready work.
A separate Codex session in the product checkout must receive its own explicit configuration.
This launch configuration does not install itself in that checkout.

## Prepare the product checkout

These steps apply when launching from the original plan directory.
In an imported copy, use the recorded `PLAN_ROOT`, candidate checkout, and run checkpoint.

1. Read every package document before dispatch. Include the complete goal, contracts, task graph, test policy, and evidence limits.
2. Locate a suitable checkout of `tailrocks/terminal-components-claude`. Check its origin, branch, status, and applicable instructions.
3. If none exists, clone into an unused separate directory. The default sibling is `../terminal-components-claude`.
4. Never replace an existing directory or overwrite user work. Select another unused path when the default is occupied.
5. Record the candidate path. Verify PR 38 uses `parity/termrock-visual-baseline` in the intended repository.
6. Fetch current PR 38 and PR 26 heads. Verify PR 26 uses `design/interaction-polish`.
7. Record exact SHAs and differences from the historical review. Keep the PR 26 reference worktree read-only.
8. Import the complete approved package into `docs/implementation/pr38-final/` in the candidate checkout. Exclude only `.git/`.
9. Preserve conflicting target files for review. Keep package `AGENTS.md` and `.codex/` nested inside that imported directory.
10. Read current target instructions before applying `AGENTS.patch`. Check the patch against the current target first.
11. If patch context changed, reconcile its instruction additions through a reviewed Luna change. Preserve unrelated target rules.
12. Verify the imported package with `shasum -a 256 -c SHA256SUMS.txt` from its directory. Never overwrite target-root `AGENTS.md` or `.codex/`.
13. Record bootstrap evidence under `PLAN_ROOT` until the imported results directory is available.
14. Use `docs/implementation/pr38-final/results/run-state.json` in the candidate checkout as the canonical run checkpoint.
15. Resume from recorded state and evidence. Do not reset progress from the package's initial `NOT_STARTED` state.
16. Complete T00, T37, T01, and T38 in order. T37 is the first Termrock source change.
17. Require separate Sol medium instances for T38 design and review. Review each later test plan before test edits.
18. Then follow the dependency graph in [PLAN.md](PLAN.md) and [task-index.json](task-index.json).

Do not run Cargo or perform source cleanup in this plan repository.
Keep all T37 deletions in the verified candidate checkout.
Preserve immutable history, approved oracle evidence, safety contracts, production diagnostics, and unrelated APIs.
Before target Rust work, follow the Rust toolchain skill and `/Users/donbeave/.agents/DISK_POLICY.md`.

Historical source, tags, approved oracle evidence, and immutable references stay outside the active suite.
The empty suite after T37 is an unverified migration state.
Preflight document, structural, and build checks can pass without proving product behavior.
A zero-case behavior lane is `NOT VERIFIED`.

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
