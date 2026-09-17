# Refactoring plan navigation

Status: **current preparation state is NO-GO** (2026-09-18).

## Current authority

[`execution-readiness-report.md`](execution-readiness-report.md) is the sole
current readiness report. It governs whether the `refactor/holla-parity`
campaign may be armed or executed.

Current operational contracts:

- [`campaign-policy.md`](campaign-policy.md) — branch, tag, and campaign scope
- [`campaign-executor-protocol.md`](campaign-executor-protocol.md) — subagent execution contract
- [`path-contract.md`](path-contract.md) — taskfmt and host-local paths
- [`proof-contract.md`](proof-contract.md) — proof and receipt invariants
- [`task-format.md`](task-format.md) — latest taskfmt identity and command surface
- [`subagent-only-policy.md`](subagent-only-policy.md) — host-local, no-container execution policy
- [`campaign-ledger.schema.json`](campaign-ledger.schema.json) — ledger schema

Use only taskfmt `0.2.0` at revision
`afd3b575dbcc7044620bec4b9493a74eca3e5ef2`, from
`/Users/donbeave/Projects/taskfmt/task-format`. Older taskfmt commands,
revisions, fingerprints, and receipts are not valid execution inputs.

## Historical evidence

The dated reports, reviews, prompts, wave plans, and TASK-001 runbooks in this
directory are retained for provenance. They are not active instructions and
may describe superseded branches, tool versions, or failed experiments. Do
not replay their commands. Reconcile any disagreement against the current
readiness report and the current contracts above.

Root-level goal and coordination files follow the same rule: `GOAL.md` is the
active product goal; `COMPONENT_ARCHITECTURE.md` and `DESIGN.md` are active
architecture/design references; the remaining old goal, handoff, state, and
coordination files are historical context.

The frozen `visual-baseline` tag remains an immutable oracle reference. Never
move, retarget, recreate, or write to its release/store.

No refactoring task may start a container or use a container runtime. All task
implementation and task verification runs through isolated subagents. Taskfmt
is limited to per-task `lint` and `verify`.
