# Work handoffs

The durable work inventory is [remaining-work.md](../remaining-work.md); task IDs, scenarios, sizes, decisions, and acceptance criteria belong there or in the owning feature/task specification. This directory is only for a handoff that cannot be represented by those maintained contracts.

## Before resuming

1. Read the repository [working goal](../GOAL.md) and the root `AGENTS.md`.
2. Read the relevant open requirement in `remaining-work.md` and its linked specification.
3. Inspect the exact current branch tip and worktree state. Do not rely on a prior handoff SHA, local scratch path, gate result, or status statement.
4. Re-run the required checks for the current source revision. Historical captures, test counts, and review notes are context only.

## Active WIP branches

These tips are the branch refs observed when this index was updated. Always inspect the live remote/local refs before resuming; a listed SHA is a locator, not a gate result.

| Branch | Recorded tip | Open scope | Next action |
|---|---|---|---|
| `wip/plan-s16` | `347158e4e` | S16 library and app migrations | Continue the S16 acceptance row in `remaining-work.md`; qualify and integrate the complete slice. |
| `wip/rapps-3-7-14` | `82f03d86f` | R-apps-3 responsive metadata and table columns | Complete R-apps-3 in `remaining-work.md`; verify table consumers and captures before closure. |
| `wip/reach-pages-a` | `9c17b6bf5` | Showcase reach rows R41–R44 | Complete the listed page scenarios and run the required visual gate. |
| `wip/reach-pages-a-2` | `4e718ca48` | Showcase reach rows R34 and R40 | Complete the listed scroll and terminal scenarios and run the required visual gate. |
| `wip/s14-followups` | `c5ac9b05c` | Clipboard, K6, and PageHeader follow-ups | Resolve each open S14-related criterion in `remaining-work.md`; re-run app and visual acceptance. |
| `wip/v1-showcase` | `0a4834178` | Legacy Showcase reference qualification and D10 | Resolve the D10 decision and V1 criteria in `remaining-work.md`; do not treat prior captures as approval. |

For each branch, inspect its current tip, worktree, tests, and remaining-work rows before doing anything. Rebase and verify against the current design branch before integration. Remove or update a row when its WIP branch lands or is retired.

## Recording a new handoff

Keep it short and tied to a live task. Include the exact source revision, changed paths, required commands and their results, unresolved acceptance criteria, and links to the maintained requirement. Do not copy the requirement or backlog into a second tracker. Remove a handoff once the maintained task record is sufficient to resume independently.

The per-branch reports formerly stored under `resume/` were removed after their open acceptance criteria were retained in [remaining-work.md](../remaining-work.md). They contained dated test results, staffing, and scratch paths that do not establish current evidence.
