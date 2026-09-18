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

Documentation integrity is enforced by the repository root
[`lychee.toml`](../../lychee.toml) and the blocking `Markdown links` job in
[`.github/workflows/ci.yml`](../../.github/workflows/ci.yml). It audits every
tracked Markdown input, including task and historical documents that remain
referenced. Lychee is verifier-only; it does not orchestrate agents or start
containers. Plain-text repository paths still require semantic review and Git
history investigation when they are not Markdown link destinations.

Use only taskfmt `0.2.0` at revision
`afd3b575dbcc7044620bec4b9493a74eca3e5ef2`, from
`/Users/donbeave/Projects/taskfmt/task-format`. Older taskfmt commands,
revisions, fingerprints, and receipts are not valid execution inputs.

## Historical evidence

Retired dated reports, prompts, wave plans, readiness stubs, and TASK-001
runbooks were removed from this branch. Git history is the provenance source
for them. The remaining dated reports, reviews, and audits are retained only
when current task contracts or machine ledgers bind their source evidence.
They are never active instructions; they may describe superseded branches,
tool versions, or failed experiments. Do not replay their commands. Reconcile
any disagreement against the current readiness report and contracts above.

Archived records may mention `docs/sources/PLANNING_GOAL.md`. That path is not
a current-checkout document: where the historical source is still material,
the authoritative reference is the immutable file at the
[`visual-baseline` commit](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2/docs/sources/PLANNING_GOAL.md).
Do not restore that deleted path merely to satisfy an archived reference.

Root-level goal and coordination files follow the same rule: `GOAL.md` is the
active product goal; `COMPONENT_ARCHITECTURE.md` and `DESIGN.md` are active
architecture/design references; retired goal, handoff, state, coordination,
prompt, and report files are absent from the current tree and recoverable only
from Git history.

The frozen `visual-baseline` tag remains the policy-protected oracle reference.
Its local and remote pointers are unchanged, but provider-enforced tag/release
immutability is not established. Never move, retarget, recreate, or write to
its tag, release, or oracle store.

No refactoring task may start a container or use a container runtime. All task
implementation and task verification runs through isolated subagents. Taskfmt
is limited to per-task `lint` and `verify`.
