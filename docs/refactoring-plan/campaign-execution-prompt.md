# Historical campaign execution prompt

> **Superseded.** Never arm `/goal` from this prompt. It records an earlier
> catalog/toolchain state. Current execution authority is
> [`execution-readiness-report.md`](execution-readiness-report.md), currently
> **NO-GO**.

Stored for **campaign preparation** on **2026-09-15**; re-verified **2026-09-15** on branch `visual-baseline` @ `84742bf6` (tag `visual-baseline` peeled commit remains frozen at `4a79c0a2`). This file freezes the canonical `/goal` prompt for executing the terminal-components refactoring campaign against the pinned catalog. Do not edit planning contracts or task packages during campaign execution without explicit replanning.

**Coordinator paste (2026-09-15):** a verbatim coordinator-supplied body was analyzed and merged here. The paste implied a full 7,550-combo visual matrix on every production edit; the stored body **amends** that with tiered gates in [Campaign iteration guide](campaign-iteration-guide.md) (edit-loop targeted `TUISNAP_FAST=1` filters vs fidelity full matrix at acceptance boundaries only). **Arm from this file, not the unamended paste.**

**Fast iteration:** edit-loop visual tiers and targeted filters are in [Campaign iteration guide](campaign-iteration-guide.md) (mandatory addendum — not a relaxation of acceptance).

**Readiness gate:** [`execution-readiness-assessment.md`](execution-readiness-assessment.md) — do not arm until `READY FOR REFACTORING EXECUTION` is issued on frozen catalog bytes.

**Frozen catalog references:** task packages under `refactoring-tasks/terminal-components/completion/`, planning artifacts under `docs/refactoring-plan/`, and source contracts under `docs/sources/`. See [Campaign executor adaptation](campaign-executor-protocol.md), [Planning progress](PROGRESS.md), and [Proof contract](proof-contract.md).

Copy the prompt below verbatim when arming the campaign goal.

---

/goal Execute the complete `terminal-components` refactoring campaign from the repository:

https://github.com/donbeave/terminal-components-claude

The execution specification and task catalog are on the frozen planning/oracle tree represented by `visual-baseline`.

# Mission

Complete the entire refactoring described by:

* `refactoring-tasks/**`
* `docs/refactoring-plan/**`
* `docs/sources/REFACTORING_COMPLETION_PLAN.md`
* the protected source/trust material referenced by those contracts

The final result must complete the intended reusable terminal-components architecture while preserving the existing product experience exactly.

Architecture may change.

User-visible behavior, interaction semantics, CLI-visible behavior covered by the contracts, and visual representation must not regress.

The four applications that must remain behaviorally and visually equivalent are:

* Showcase
* Holla
* Jackin
* TablePro

Execute the whole campaign through its final closure task. Do not stop after implementing only the reusable library or only one application.

Use subagents aggressively for all implementation, investigation and verification work.

The parent `/goal` agent is the campaign coordinator and trusted integration authority. It should orchestrate work rather than directly implementing large production changes.

---

# Critical source identities

Before changing anything, fetch the repository and independently verify all identities instead of trusting this prompt blindly.

Expected current identities are:

* architectural `main` starting point:
  `7b27732a8c3c131760ec3438f641cb3c11343a42`
* frozen annotated tag `visual-baseline` (peeled commit — do not move):
  `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`
* current planning branch `visual-baseline` catalog tip (verify at arm time; may be ahead of tag):
  `84742bf6` (verify at arm time with `git rev-parse HEAD`)
* immutable product-experience oracle referenced by the task contracts:
  `02f5294bfdbf38004cc49130d0aff1d01f31434c`
* task-format authority:
  `52d9f1eb7721f409bc47beb9fced7997b5c13ede`

Treat these identities as serving different purposes.

`main` is the architecture starting point.

The immutable product oracle defines the expected product experience.

The frozen `visual-baseline` tag pins the immutable oracle-era baseline; the planning branch carries the live task catalog, visual evidence, task packages and supporting baseline infrastructure. Record the exact catalog SHA when arming the campaign.

Do not substitute one authority for another.

Never move, delete, recreate, retarget or force-update the `visual-baseline` tag.

Never redefine the oracle from candidate output.

---

# PHASE 0 — close execution readiness before production work

The current `visual-baseline` planning artifacts explicitly state that execution readiness was reopened by the source-first re-audit.

Therefore DO NOT blindly begin TASK-001.

First finish the existing planning/readiness work.

Read completely:

* `GOAL.md`
* `docs/refactoring-plan/PROGRESS.md`
* `docs/refactoring-plan/reaudit-plan.md`
* `docs/refactoring-plan/reaudit-findings.tsv`
* `docs/refactoring-plan/branch-continuation-proposal.md`
* `docs/refactoring-plan/proof-contract.md`
* `docs/refactoring-plan/campaign-executor-protocol.md`
* `docs/refactoring-plan/task-index.tsv`
* `docs/refactoring-plan/task-graph.md`
* `docs/refactoring-plan/traceability.tsv`
* `docs/refactoring-plan/architecture-adjudication.md`
* `docs/refactoring-plan/execution-readiness-assessment.md`
* `docs/refactoring-plan/campaign-arming-readiness.md`
* all referenced re-audit reports
* every package under `refactoring-tasks/terminal-components/completion/`

Spawn independent planning/readiness subagents across the existing re-audit partitions.

Use different subagents to verify repairs than the subagents that authored them.

Resolve every material outstanding finding, stale source identity, stale external-tool fact, missing synchronization, incomplete branch-diff partition, pending qualification, task-contract problem, dependency problem and traceability gap.

In particular, re-check current external facts rather than preserving stale prose merely because it existed in the planning branch.

Do not modify production implementation during this phase.

Do not manufacture readiness merely by changing a status field.

Run all deterministic planning, task-format, task-graph, source-binding, payload-integrity, traceability and qualification validators required by the planning goal.

Execution may begin only after fresh independent reviewers establish:

`READY FOR REFACTORING EXECUTION`

against the exact frozen catalog bytes that will be used by the campaign.

If catalog/task contracts must be repaired during Phase 0, repair them first, independently verify them, freeze the resulting catalog, record its exact tree/hash, and execute that frozen catalog.

Do not execute an obsolete version of a task merely because it was originally numbered 001–073.

If Phase 0 discovers a genuinely unresolved decision for which no evidence-backed resolution exists, report `BLOCKED` rather than guessing.

Otherwise continue directly into execution once readiness is established.

---

# Execution topology

Production implementation must start from the architectural `main` tree, not from the visual/oracle branch.

Use a **single campaign worktree** (`.worktrees/campaign` on branch `refactor/holla-parity`, seeded from architectural `main` @ `7b27732a…` or absorbed bootstrap). Do not create separate `task-001-bootstrap` / `.worktrees/main` production trees. See [`campaign-policy.md`](campaign-policy.md) and [`path-contract.md`](path-contract.md).

Keep the frozen catalog/oracle/trusted material outside candidate authority.

Do not modify `visual-baseline` as part of production implementation.

Do not merge or push to `main`.

The result of this goal is a fully verified integration tree/branch ready for a separately authorized merge.

Derive execution order from the canonical `task.toml` dependency graph.

Task numbers are identifiers, not ordering.

Do not simply run `001, 002, 003, ...`.

The currently expected broad dependency progression is:

1. `TASK-001`
2. `TASK-070`
3. `TASK-071`
4. `TASK-072`
5. dependency-ready baseline producers `TASK-002`, `003`, `004`, `005`, and inventory `007`
6. `TASK-006`
7. `TASK-008`
8. `TASK-073`
9. reusable component/runtime tasks `009–030` according to their actual dependencies
10. `TASK-031`
11. the four application restoration chains, parallel where dependency-safe:

* Showcase `032–039`
* Holla `040–050`
* Jackin `051–057`
* TablePro `058–064`

12. closure work:

* `065`
* `067` when dependency-ready
* `066`
* `068`
* `069`

This summary is only a cross-check.

The canonical `task.toml` graph always wins.

Before every dispatch, prove that every hard dependency has an accepted receipt and its accepted source is actually integrated into the parent tree.

A mutable `status = done` flag or an agent summary is never dependency proof.

---

# Parallelism

Use subagents aggressively.

Keep all ready work parallel whenever the actual DAG and writable scopes permit it.

Every modifying task gets its own isolated worktree.

Never let two agents modify the same worktree.

Before parallel dispatch:

1. compute the dependency-ready set from canonical `task.toml`;
2. verify the accepted predecessor receipts;
3. verify writable/forbidden scopes;
4. verify there is no unsafe shared-file ownership;
5. record the exact parent SHA supplied to each agent.

After parallel siblings complete, do not merely concatenate individually passing branches.

Integrate them through the qualified compare-and-swap host mechanism into a fresh combined tree.

Then rerun the union of impacted contracts, test accounting, workspace gates and visual checks per [Campaign iteration guide](campaign-iteration-guide.md) §4 for the combined impacted surface; full matrix if any sibling touched cross-app rendering.

Individually passing siblings are not evidence that their composition passes.

---

# Mandatory per-task agent protocol

Every task must have at least two independent subagent roles:

## A. IMPLEMENTER

Spawn a dedicated implementation subagent for exactly one task package.

Give it:

* the immutable task package
* the exact integration parent
* only its allowed candidate worktree
* the task's `README.md`
* `AGENTS.md`
* `task.toml`
* `verify.toml`
* `trusted/**`
* every file listed under `Read before editing`
* accepted dependency receipts/results relevant to that task

The implementer must read the complete task before editing.

It must state:

* task ID
* one-sentence objective
* requirements
* acceptance IDs
* first checklist leaf
* exact starting SHA

It must obey the task's writable paths exactly.

No drive-by cleanup.

No unrelated refactoring.

No changes to the task contract from inside the task execution.

No changes outside `writable_paths`.

No weakening tests.

No modifying protected fixtures.

No special-casing verifier inputs.

No changing expected evidence.

No disabling warnings/lints/checks.

No hiding errors behind cfgs.

No replacing real production behavior with mocks that only satisfy tests.

No compatibility layer whose purpose is to avoid completing the intended reusable architecture.

The implementer may run ordinary local tests for feedback, but those are advisory and do not establish canonical acceptance.

The implementer may also run targeted `TUISNAP_FAST=1` visual filters (see [Campaign iteration guide](campaign-iteration-guide.md) §1) for edit-loop feedback; those runs must not be claimed as task acceptance.

## B. INDEPENDENT VERIFIER

After the implementation candidate is frozen and canonical machine verification has run, spawn a DIFFERENT fresh-context subagent.

It must not be the implementation agent.

Give the verifier read-only access to:

* the complete task contract
* exact starting parent
* exact candidate tree/diff
* accepted predecessor evidence
* canonical host/check outputs
* visual-regression results where applicable
* trusted obligation mappings
* relevant source/oracle evidence

Do not give it an implementation-agent summary as the source of truth.

Tell the verifier:

> Assume the implementation agent misunderstood the task or optimized for the verifier. Find concrete ways this implementation could pass the literal checks while violating the source requirement, architecture, functionality, interaction semantics, or visual baseline.

The verifier must inspect actual code and evidence.

It must explicitly check:

* every requirement
* every acceptance criterion
* every trusted source obligation
* forbidden-path integrity
* behavior preservation
* reusable ownership
* API boundaries
* regression tests
* visual parity where applicable
* dependency assumptions
* absence of fixture-specific cheating
* absence of duplicated app-local reusable behavior
* absence of legacy/new parallel architecture
* absence of test weakening
* absence of unverified behavior hidden behind mocks

Its result must be one of:

`VERIFIED`
`REJECTED`
`BLOCKED`

Only `VERIFIED` can proceed to integration.

---

# Failure / repair loop

If any canonical check or independent verifier fails:

1. do not integrate the candidate;
2. preserve the failure evidence;
3. identify the smallest violated requirement/acceptance/check;
4. spawn a repair subagent against the rejected candidate or a fresh candidate as appropriate;
5. make only evidence-backed corrections;
6. freeze again;
7. rerun the COMPLETE canonical verification for that task;
8. rerun the applicable functional and visual gates — targeted visual filters with `TUISNAP_FAST=1` while repairing (see [Campaign iteration guide](campaign-iteration-guide.md) §1); mandatory full matrix per §4 before re-freeze and a fresh independent verifier;
9. spawn a fresh independent verifier.

Do not reuse a previous verifier's approval after candidate bytes change.

Every candidate revision invalidates the previous exact-tree verdict.

Repeat until both:

* canonical host verification passes, and
* independent verification passes.

Do not spin indefinitely on the same unexplained failure. Follow the task's `BLOCKED`, `NEEDS_REPLAN`, and `INCOMPLETE` semantics.

If satisfying a task requires changing its contract, architecture decision, scope or dependencies, stop that candidate as `NEEDS_REPLAN`, return to the planning authority, repair/freeze the contract independently, and only then redispatch it.

---

# Canonical verification authority

Follow:

`docs/refactoring-plan/campaign-executor-protocol.md`

and

`docs/refactoring-plan/proof-contract.md`

exactly.

The implementation subagent does NOT own the verdict.

The coordinator/trusted operator owns:

* source freeze
* immutable task contexts
* standalone taskfmt invocation
* trusted proof installation
* dependency receipt resolution
* exact-tree binding
* verification
* acceptance receipts
* integration ref

Special-case `TASK-001` exactly as documented: it bootstraps the future host and therefore cannot use itself as its own trust root.

Use the independently frozen bootstrap comparator/host fixtures to qualify TASK-001.

After TASK-001 is accepted, use the resulting qualified host for later tasks as the contracts require.

Do not use the taskfmt `run`, monitor-dispatch, or `promote` lifecycle for this campaign.

It targets `main` and is explicitly prohibited here.

Use the supported standalone taskfmt verification with:

* explicit immutable base
* non-empty canonical progress
* pinned configuration
* immutable per-check contexts
* frozen candidate tree

The campaign-specific executor protocol supersedes any generic AGENTS instruction that would allow an empty-progress verification invocation.

A task succeeds only when the actual host-controlled canonical gate succeeds for the exact frozen tree.

---

# Progress

Each executor must use canonical task-format progress semantics.

Keep real progress in the task's required progress event stream.

Do not treat chat summaries as progress authority.

Record leaf completion only with evidence.

For every task preserve:

* task ID
* starting parent SHA
* candidate SHA/tree
* implementation subagent
* canonical progress
* commands run
* canonical verification result
* host verdict/receipt identity
* independent verifier identity
* verifier verdict
* integration result

The campaign coordinator must maintain a campaign ledger so execution can be resumed without guessing.

---

# Product-preservation invariant

This is a refactor, not a redesign.

Preserve all existing contracted behavior including:

* layout and geometry
* spacing and alignment
* dimensions and clipping
* borders and separators
* glyphs/icons/text
* colors and theme behavior
* focus and focus traversal
* hover
* pressed/selected/disabled/active states
* keyboard controls
* mouse controls
* pointer hit geometry
* cursor behavior
* text editing
* grapheme behavior
* text selection and copy
* scrolling
* scroll boundaries
* scrollbar geometry
* scroll-edge fades
* dialogs
* menus
* submenus
* overlays/popovers
* tables/grids
* truncation/wrapping
* status/hints
* empty/error states
* navigation
* resize behavior
* narrow/minimum terminal behavior
* deterministic tick/motion behavior covered by the oracle
* CLI-visible behavior covered by the contracts
* application flows
* lifecycle semantics

for Showcase, Holla, Jackin and TablePro.

Do not accept “similar”.

Do not accept “looks correct”.

Do not accept compilation as parity proof.

Do not accept new screenshots as proof that a changed UI is correct.

Exact contract evidence wins.

---

# Visual regression is a hard fail-closed gate

The committed grouped snapshot store is read-only acceptance evidence:

`snapshots/<app>/<family>/[<surface>/][<state>/]<cols>x<rows>/<color>.{ansi,txt,png,html}`

The current baseline covers thousands of exact scenarios and multiple terminal sizes/color modes.

Candidate execution must NEVER:

* modify `snapshots/`
* modify `shots/`
* remove expected snapshots
* change expected snapshot contents
* run `tuisnap accept`
* bless candidate output
* weaken cell comparisons
* weaken pixel comparisons
* change comparison thresholds to make drift pass
* change scenario membership to hide a regression
* change action coordinates based on candidate output
* change the oracle reference
* silently update a golden file

Root repository rule:

Use `cargo nextest`, never `cargo test`, for terminal-components Rust validation.

For iteration tiers, targeted filters, CI smoke, and when the mandatory full gate applies vs edit loops, see [Campaign iteration guide](campaign-iteration-guide.md). During edit loops use targeted filters + `TUISNAP_FAST=1` (§1); run the mandatory full gate only at acceptance/integration boundaries (§4).

For every production edit that can affect output, run targeted feedback first if useful:

**Edit loop (iteration — not acceptance):**

```sh
TUISNAP_FAST=1 cargo nextest run --run-ignored only \
  -E 'binary(visual_baseline) & test(<filter>)'
```

Pick `<filter>` from [Campaign iteration guide](campaign-iteration-guide.md) §5 (e.g. `showcase_`, `holla_flows_`, one capture root, or a single combo). Do not run all 7,550 captures during iteration.

**Acceptance / integration boundary (mandatory full gate):**

```sh
# Full matrix at fidelity timing (~45–60 min) — required for acceptance
cargo nextest run --run-ignored only -E 'binary(visual_baseline)'
```

Optional nightly speed feedback (not a substitute for acceptance when snapshots are fidelity-blessed):

```sh
TUISNAP_FAST=1 cargo nextest run --run-ignored only -E 'binary(visual_baseline)'
```

Also retain ordinary regression coverage:

```sh
cargo nextest run
cargo nextest run -E 'test(store_integrity)'
```

Default `cargo nextest run` excludes ignored PTY captures (~544 tests, ~1–3 min); see [Campaign iteration guide](campaign-iteration-guide.md) §6. It is not a substitute for visual regression on rendering tasks.

When a visual failure reports `cells-differ` or `pixels-differ`, the candidate fails.

Inspect:

`target/tuisnap/diff/<name>.png`

and the first-difference evidence.

Fix the implementation.

Do NOT update the baseline.

If evidence indicates the frozen expected output itself is genuinely defective, classify that as a separate oracle/adjudication blocker. Do not allow a task executor to bless a replacement.

After every accepted task that can affect product rendering or interaction, preserve the full visual result with its task receipt.

At application-chain closures (`039`, `050`, `057`, `064`) run the complete required application oracle/PTY and visual closure — **fidelity** full matrix (see [Campaign iteration guide](campaign-iteration-guide.md) §3–§4).

At cross-application joins and final closure, rerun the complete fidelity visual suite again.

Before TASK-069 and merge readiness, run the same fidelity full gate so every combo renders PNG/HTML.

The final tree must have zero unexpected visual differences.

---

# Functional non-regression

For every task:

* run the focused tests named by the contract;
* run the canonical verifier;
* run additional regression tests required by the task's affected surface;
* run the full ordinary `cargo nextest run` after production changes unless the canonical host already proves an equivalent stronger complete inventory;
* retain every pre-existing compatible test;
* follow TASK-007/TASK-008 disposition authority for migrated/conflicting historical tests.

Never delete or rewrite a failing historical test merely because the new architecture does not satisfy it.

A historical assertion may change only through the explicitly approved TASK-008 disposition/span mechanism.

Preserve test identity and relocation evidence as required.

---

# Architecture completion

Do not achieve visual parity by recreating the old architecture inside application code.

The final architecture must preserve the accepted contracts around:

* caller-owned durable state
* short-lived borrowed props
* separate mutable update and shared-reference draw
* typed actions/responses
* stable identity
* runtime-owned input/focus/layers/time
* semantic paint attribution
* one reusable implementation per component family
* backend-free reusable core
* curated application/author facade
* shared reusable ownership of generic visible pixels and interaction behavior

Reject solutions that reintroduce:

* the old widget facade
* an owned runtime component tree
* untyped generic action buses
* universal Widget/Theme abstractions
* application-owned generic overlay stacks
* Grid-owned SQL semantics
* generic RGB role inference
* duplicate old/new component implementations
* compatibility shims that make the migration indefinitely incomplete
* app-local copies of reusable component behavior
* paint-over layers that call a reusable component and then independently redraw another model over it

Visual parity AND architecture correctness are both required.

Neither can compensate for failure of the other.

---

# Application chains

After reusable component conformance closes in TASK-031, execute the four application chains as independently as the DAG allows.

Use dedicated application subagents and independent application verifier subagents.

Showcase must restore and close its entire contracted route/page/state set through TASK-039.

Holla must restore and close its entire contracted world/route/flow set through TASK-050.

Jackin must restore and close all contracted states, worlds, motion and capsule behavior through TASK-057.

TablePro must restore and close all contracted connection/workbench/grid/query/safety/history behavior through TASK-064.

Each application's final closure must prove actual application behavior and reusable-component ownership together.

No application may reach its closure through compatibility painting or inert visual controls.

---

# Cross-task architecture and regression reviewers

In addition to the per-task verifier, periodically spawn fresh cross-cutting read-only reviewers.

At minimum after:

* TASK-008
* TASK-073
* TASK-031
* each application closure
* TASK-065
* TASK-068
* before TASK-069 acceptance

Use independent specialists for:

1. reusable architecture/API
2. behavior and interaction parity
3. visual regression
4. verifier/security bypass analysis
5. test preservation
6. performance/allocation contracts
7. DAG/integration integrity

Tell them to find regressions, not to confirm prior work.

Every material finding must either be fixed and freshly verified, or rejected with concrete source evidence.

---

# Git/integration safety

Never work directly on `main`.

Never work directly on the frozen oracle/tag.

Never force-update shared refs.

Every implementation task begins from its exact host-selected integration parent.

Every verified task is integrated with compare-and-swap against that same expected parent.

If the integration parent moved, reject the integration and rebase/reconstruct/reverify on the actual new combined tree according to the host protocol.

Never attach an old successful receipt to a different tree.

Prefer normal merge-visible history where appropriate; do not erase meaningful campaign boundaries merely for a linear history.

Keep each task/change attributable.

Do not push or merge into `main` unless separately and explicitly authorized after this `/goal`.

---

# Dependency/tool changes

Do not perform opportunistic dependency upgrades.

Use the exact source/tool pins required by the frozen campaign.

If an external prerequisite such as tui-snap has moved or an earlier planning statement is stale:

* resolve the actual current source identity;
* compare it with the accepted campaign pin;
* preserve/requalify the exact accepted revision required by the task;
* update planning authority only through the Phase-0/frozen-contract process when necessary.

Never silently replace a qualified dependency with “latest”.

---

# TASK-069 final closure

TASK-069 is validation-only.

Do not repair product behavior by hiding changes inside TASK-069's report/artifact path.

If TASK-069 finds a failure:

1. identify the owning earlier task;
2. reopen/repair that task through the complete implementer → canonical verification → independent verifier flow;
3. reintegrate;
4. rerun all affected downstream closure;
5. rerun TASK-069 from the fresh integration tree.

TASK-069 must prove one exact integration tree with:

* genuine accepted dependency ancestry
* direct production captures
* executable PTY captures
* exact comparison
* complete required test accounting
* architecture verification
* build/toolchain/MSRV requirements
* formatting/lint/documentation/API gates
* performance gates
* trust-root immutability
* visual non-regression
* complete application closure
* merge-readiness ancestry/integration safety

No unresolved set is permitted at final closure.

---

# Never fake completion

Do not mark a task complete because:

* code compiles;
* unit tests pass;
* an implementation agent says it is complete;
* an old similar test passes;
* a screenshot looks correct;
* task metadata says `done`;
* a previous tree passed;
* a PR existed with similar changes;
* candidate-authored evidence says success.

Completion requires the exact frozen-tree canonical gate plus independent review.

---

# Final adversarial campaign review

After TASK-069 passes, but before declaring success, spawn fresh agents that have not implemented the campaign.

Give them the final tree and frozen contracts.

Ask independently:

> Find any concrete way this tree changed the product experience, broke an interaction, bypassed the intended reusable architecture, lost a historical requirement, weakened a test, modified oracle authority, or obtained a passing task receipt for the wrong tree.

Review at minimum:

* architecture
* Showcase parity
* Holla parity
* Jackin parity
* TablePro parity
* visual parity
* test preservation
* API/docs
* performance
* proof-chain integrity
* Git ancestry and receipts

Any material finding invalidates the final readiness claim until repaired and TASK-069/final review are repeated.

---

# Definition of done

The goal is DONE only when:

* Phase 0 produces a fresh `READY FOR REFACTORING EXECUTION` against a frozen catalog;
* every canonical task in that frozen catalog has been executed;
* all task dependencies are satisfied by actual accepted ancestry/receipts;
* every task has a successful canonical exact-tree verification;
* every implementation task has an independent verifier approval;
* every rejected candidate was repaired and freshly reverified;
* the complete reusable architecture is implemented;
* Showcase is fully restored;
* Holla is fully restored;
* Jackin is fully restored;
* TablePro is fully restored;
* historical tests are preserved/dispositioned correctly;
* the visual baseline has zero unintended differences (proven by full-suite command in [Campaign iteration guide](campaign-iteration-guide.md) §3/§4 — smoke or targeted filters alone are insufficient);
* `snapshots/` and `shots/` have not been modified;
* the complete functional inventory passes;
* architecture gates pass;
* API/docs gates pass;
* performance gates pass;
* TASK-069 passes against the exact final integration tree;
* final fresh-context adversarial reviewers find no unresolved material issue;
* the final integration tree is merge-ready;
* no push or merge into `main` has occurred without separate authorization.

Do not reduce scope to fit a turn.

Use subagents and resumable task-format progress instead.

Do not skip work silently.

---

# Final report

Return an evidence-based campaign report containing:

* frozen catalog/source SHA
* immutable oracle identity
* architectural starting SHA
* task-format identity
* integration branch/ref and final SHA
* Phase-0 readiness evidence
* number of tasks executed
* per-task status ledger
* per-task implementer/verifier pairing
* blocked/replanned/retried tasks
* canonical host verdict/receipt for every task
* application closure results
* full functional-test result
* full visual-baseline result — exact command, environment (`TUISNAP_FAST`, `--profile ci`, etc.), and tier (smoke / fast full / fidelity)
* proof that `snapshots/` and `shots/` were unchanged
* architecture/API verification result
* performance result
* TASK-069 final receipt
* final adversarial-review findings and dispositions
* exact final Git ancestry
* any remaining blocker

Finish with exactly one of:

`REFACTORING COMPLETE — VERIFIED AND READY FOR SEPARATELY AUTHORIZED MERGE`

or

`REFACTORING INCOMPLETE — BLOCKED`

Never report success with any unresolved correctness, functionality, interaction, visual, architecture, verification, task-contract or trust-chain issue.
