# Refactoring plan navigation

Status: **NO-GO; preparation only; `.campaign/ledger.json` remains `armed: false`** (2026-09-19).

This reconciliation is bound to preparation source HEAD
`f9801a89cbe0d154f38d184b007fca07d058f39b` and tree
`550d22547254eb9eee2bd11a74d6e77505c5c913`, observed before this bounded
docs-only update. This documentation commit changes the source tree again;
source-bound proof receipts must bind the post-commit tree afresh. The current
bounded results and blockers are indexed in the
[current preparation evidence](evidence/current-preparation-2026-09-19.md).

## Current authority

[`execution-readiness-report.md`](execution-readiness-report.md) is the sole
current readiness report. It governs whether the `refactor/holla-parity`
campaign may be armed or executed.

[`next-implementation-goal.md`](next-implementation-goal.md) is the generated
`/goal` startup prompt. It is **NOT AUTHORIZED FOR EXECUTION** while readiness
is **NO-GO**.

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

Current preparation facts: the qualified binary SHA-256 is
`f9781ef8ad5909a8dc9f5902aafa177623310eb72cb1645a37de4567016664de`, plan
validation passes with `error_count: 0`, and standalone taskfmt lint passes
`73/73`. Preflight exits `1` at the explicit readiness-report **NO-GO** gate;
the stale ledger has no current accepted verifier receipt, remains
`armed: false`, and has no accepted task rows. Proof fixes landed in
`8c9e050c`, `f4ce758e`, `4737da3c`, `7639e7ae`, `19f6d2eb`, and
`12ac27ff`. The proof parent `12ac27ff` has tree
`8d66c95408d7e9e404eaf9019857b33659906e84`; current HEAD `f9801a89` is a
docs-only commit on top of it. The current proof is
**UNVERIFIED/PENDING** pending a fresh verifier; no independent verifier has
run or accepted it. The fresh verifier evidence for historical candidate
`4737da3c` is explicitly **REJECTED** and applies only to that candidate; no
independent reviewer has accepted the current proof chain. The external
corrected baseline config has SHA-256
`bdbe0a8958a696e4b5108ae190a0c07089f2d7aea91c64a20d52e3346e7092da`.
Its latest calibration selected 302 tests: 301 passed, 1 failed, and 2
skipped; the targeted calibration passed 3/3. The failure is
`tablepro_connections_form_advanced_120x40_truecolor` with 76/100 artifacts
mismatching. Baseline refs/store and snapshots remain unchanged. These facts
do not authorize arming, dispatch, or execution.

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

The frozen `visual-baseline` tag remains the policy-protected oracle reference
at peeled commit
`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`. Its local and remote pointers are
unchanged, but provider-enforced tag/release
immutability is not established. Never move, retarget, recreate, or write to
its tag, release, or oracle store.

No refactoring task may start a container or use a container runtime. All task
implementation and task verification runs through isolated subagents. Taskfmt
is limited to per-task `lint` and `verify`.
