/goal Work only on the `visual-baseline` branch of:

https://github.com/donbeave/terminal-components-claude

## Mission

Perform a comprehensive, adversarial audit and repair of the entire refactoring specification and execution catalog.

The objective is **not to execute the refactoring yet**.

The objective is to make the refactoring plan, task graph, verification contracts, visual-regression protection, and `task-format` packages complete, current, internally consistent, independently verifiable, and safe enough that the future refactoring can be executed without accidentally changing or losing existing functionality, interactions, UI/UX, or visual output.

The primary material to analyze is:

* `docs/refactoring/**`
* `docs/refactoring-plan/**`
* `refactoring-tasks/**`

Also inspect every repository artifact those documents depend on or cite, including at minimum:

* `docs/sources/PLANNING_GOAL.md`
* `docs/sources/REFACTORING_COMPLETION_PLAN.md`
* `docs/design/DESIGN.md`
* `docs/baseline/**`
* `docs/parity/**`
* `docs/architecture/**`
* `docs/plan-verification/**`
* `docs/traceability.md`
* `tests/visual_baseline/**`
* `snapshots/**`
* current source/tests where necessary to validate assumptions
* relevant Git history
* relevant `main` history/read-only state where the refactoring documents cite or depend on it
* immutable visual/behavioral oracle references such as the `visual-baseline` tag

Do not assume an existing document is correct merely because it is marked canonical, reviewed, complete, or audited.

Re-derive important conclusions from evidence.

---

# Branch rules

Operate on **`visual-baseline` only**.

Documents that still say branch `holla` mean this tree. `holla` is no longer a live branch name.

Before doing anything:

1. Verify the current checked-out branch.
2. Fetch refs.
3. Record the current `visual-baseline` HEAD SHA.
4. Inspect the current repository state.
5. Do not silently switch the working branch.
6. Do not merge `main`.
7. Do not execute the actual refactoring.
8. `main`, historical commits, tags, and other refs may be inspected read-only as evidence.

The planning documents describe a future integration strategy involving `main`. That does not authorize implementing that strategy during this goal.

---

# Use subagents for everything

Use subagents aggressively and in parallel.

Do not perform a large serial audit in the parent agent when independent investigation can be delegated.

Create independent workstreams at minimum for:

1. **Goal/history reconstruction**

   * `docs/refactoring/**`
   * historical goals
   * contradictions
   * superseded decisions
   * current source-of-truth hierarchy

2. **Refactoring-plan audit**

   * every file under `docs/refactoring-plan/**`
   * architecture
   * sequencing
   * proof contracts
   * DAG
   * readiness criteria
   * traceability

3. **task-format conformance**

   * current `donbeave/task-format`
   * reference template
   * current schemas
   * linter/verifier behavior
   * project/group/task metadata
   * task dependencies
   * immutable contract rules

4. **Task-package audit**

   * divide `refactoring-tasks/terminal-components/completion/001..073` among multiple agents
   * inspect every task, not samples
   * inspect `README.md`, `AGENTS.md`, `task.toml`, `verify.toml`, `trusted/**`, and supporting proof material

5. **Visual-regression / snapshot audit**

   * `snapshots/**`
   * `tests/visual_baseline/**`
   * snapshot taxonomy
   * app/state/terminal-size/color coverage
   * visual-gate commands
   * forbidden write paths
   * oracle provenance
   * missing states and interactions

6. **Architecture/API reviewer**

   * verify that tasks collectively complete the intended reusable architecture
   * detect compatibility shims, duplicate architectures, app-local implementations, or gaps that the plan accidentally allows

7. **Application parity reviewers**

   * one or more independent reviewers for:

     * Showcase
     * Holla
     * Jackin
     * TablePro
   * map existing behavior and snapshots to tasks and verification

8. **Verification/security reviewer**

   * attack the proof model
   * look for ways an executor could make a task pass while breaking the product
   * inspect writable paths, forbidden paths, weak checks, self-fulfilling tests, mutable oracle inputs, candidate-controlled verification, missing negative tests, etc.

9. **DAG/atomicity reviewer**

   * verify dependencies
   * task boundaries
   * task sizing
   * missing prerequisites
   * circular or hidden dependencies
   * whether each task can actually be independently executed and verified

10. **Final adversarial reviewers**

    * after repairs are complete, use fresh-context subagents that did not author the changes
    * instruct them specifically to find reasons the repaired plan is still unsafe or incomplete

Parallelize these reviews as much as possible.

The parent agent is responsible for synthesizing findings, resolving disagreements using evidence, editing the canonical artifacts, and running final deterministic validation.

---

# task-format is an external authority

Audit the task catalog against:

https://github.com/donbeave/task-format

Do not rely only on the copy of task-format rules summarized inside this repository.

At the start of the audit:

1. Inspect/fetch the current `main` of `donbeave/task-format`.
2. Record the exact commit SHA used as the authority.
3. Read its current:

   * `README.md`
   * `reference/task-template/**`
   * relevant `docs/**`
   * schema definitions
   * linter/verifier implementation where necessary
   * project/group/task discovery rules
4. Determine whether this repository's currently pinned task-format revision remains current and intentional.

At present the refactoring documents reference:

* `task/v5`
* `verify/v2`
* `task-meta/v1`

But verify this instead of blindly assuming it remains correct.

If task-format has evolved:

* determine whether the catalog should stay intentionally pinned or migrate;
* document that decision explicitly;
* migrate the packages if migration is required;
* never invent a repository-specific task schema.

Each task must follow the real current task-format contract.

---

# What every task must prove

Audit **every one** of the 73 task packages.

Each task must represent one bounded, observable outcome.

For each package verify at least:

### Contract quality

`README.md` must have a precise:

* goal
* context
* preconditions
* in-scope work
* out-of-scope work
* MUST requirements
* MUST-NOT requirements
* non-regression requirements
* acceptance criteria
* fixed decisions
* checklist

Reject vague goals such as:

* "fix components"
* "improve parity"
* "finish refactoring"
* "clean up architecture"

unless they are decomposed into independently verifiable outcomes.

### Typed traceability

Every significant requirement must map to deterministic acceptance evidence.

Verify:

`Requirement -> Acceptance Criterion -> Check`

and the reverse direction:

`Check -> Acceptance Criterion -> Requirement`

There must be no important requirement that exists only as prose.

Acceptance criteria must follow current task-format conventions.

Use Gherkin-style observable behavior where appropriate.

### Verification authority

`verify.toml` must be deterministic and meaningful.

Audit:

* schemas
* task IDs
* phases
* check IDs
* requirement mappings
* acceptance mappings
* expected results
* writable paths
* forbidden paths
* environment assumptions
* preconditions
* focused checks
* regression checks
* lint/build checks
* final gate

A check must prove the requirement instead of merely executing a command that happens to return 0.

### Immutable verification

Executor-controlled implementation must not be able to rewrite the evidence used to prove itself correct.

Trusted evidence must remain outside writable executor scope.

Candidates must not be able to:

* alter expected snapshots
* regenerate expected visual output
* weaken tests
* change oracle inputs
* edit trusted fixtures
* change verifier logic
* broaden writable paths to bypass isolation
* remove failing scenarios
* suppress warnings or failing checks
* redefine acceptance criteria during execution

### Dependencies

Audit every `task.toml`.

Dependencies must encode the actual DAG.

Markdown order must never substitute for dependency metadata.

Look for:

* missing prerequisite edges
* unnecessary serialization
* hidden dependencies
* circular dependencies
* a task consuming artifacts that no predecessor guarantees
* tasks that depend on implementation details from another task without declaring it

Maximize safe parallelism.

---

# Visual and behavioral preservation is the critical invariant

The refactoring must preserve existing product behavior.

Architecture may change.

The product experience must not change unless an explicit, separately approved product change says otherwise.

The immutable behavioral reference and committed visual evidence must therefore be treated as hostile-to-change test oracles.

Audit preservation of:

* layout
* spacing
* dimensions
* alignment
* borders
* separators
* glyphs
* icons
* text
* colors
* themes
* focus
* hover
* pressed state
* selected state
* disabled state
* active state
* keyboard behavior
* mouse behavior
* focus traversal
* cursor behavior
* text editing
* text selection
* copy behavior
* scrolling
* scroll boundaries
* scrollbars
* scroll-edge fades
* menus
* dialogs
* overlays
* popovers
* tables
* grids
* truncation
* wrapping
* status bars
* empty states
* error states
* navigation
* terminal resize behavior
* narrow-terminal behavior
* application flows
* deterministic animations/ticks where relevant
* CLI-visible behavior where already part of the product contract

for all four applications:

* Showcase
* Holla
* Jackin
* TablePro

Do not treat compilation or unit tests as sufficient proof of parity.

---

# `snapshots/` is read-only acceptance evidence

The current committed grouped snapshot store is a critical regression oracle:

`snapshots/<app>/<family>/[<surface>/][<state>/]<cols>x<rows>/<color>.{ansi,txt,png,html}`

Inspect the whole snapshot corpus and the code generating/verifying it.

Determine whether snapshot coverage is sufficient to protect every refactoring task that can affect visible behavior.

Do not modify expected snapshots merely because current implementation output differs.

During this goal:

* do not run `tuisnap accept`
* do not bless new candidate output
* do not rewrite expected snapshots
* do not remove snapshots to make verification pass
* do not weaken pixel/cell comparisons
* do not change the immutable oracle reference
* do not turn visual mismatches into accepted output

`snapshots/` and historical oracle material are evidence, not implementation output.

If you discover that an expected snapshot is genuinely wrong, record it as an explicit finding requiring separate adjudication rather than silently changing it.

The planning/task audit itself must leave product visuals untouched.

---

# Snapshot coverage audit

Do not merely verify that a `snapshots/` directory exists.

Build a coverage matrix.

For every relevant application behavior/state, determine:

* whether a snapshot exists
* which terminal sizes are covered
* which color modes are covered
* which focus/hover/selected states are covered
* whether overlays/dialogs/menus are covered
* whether scrolling boundaries are covered
* whether resized/narrow states are covered
* whether dynamic/tick-based states are deterministic
* whether interaction sequences leading to the state are separately tested

Map:

`Application state -> Snapshot/test -> Task(s) whose changes can affect it`

Identify important unprotected states.

If a future task could break a visual behavior without causing a deterministic failure, the plan is incomplete.

Repair the task or verification design accordingly.

---

# Audit assumptions against actual source

Do not trust plan statements about what exists on `main`, `visual-baseline`, or historical commits without checking when those statements matter to task correctness.

Documents that still name branch `holla` refer to this same tree (`visual-baseline`).

For significant assumptions such as:

* "main already implemented this"
* "this only needs porting"
* "this component exists"
* "this API is complete"
* "this test already proves X"
* "this application has N routes/worlds"
* "this behavior is covered by snapshots"
* "this task is blocked by another task"
* "this path is obsolete"

verify against the actual repository/tree/history.

Update stale claims.

Classify historical implementation facts accurately, for example:

* not implemented
* partially implemented
* implemented but broken
* implemented and reusable
* implemented but contradicts oracle
* superseded
* obsolete
* requires selective port
* requires decision

Do not preserve outdated planning statements for historical politeness.

---

# Architecture audit

Verify that the complete task graph actually converges on the intended architecture.

Among other things, inspect whether the plan guarantees:

* caller-owned durable state
* borrowed short-lived props
* clean mutable update vs shared-reference draw boundaries
* typed responses/actions
* stable identity
* runtime ownership of input/focus/layers/time
* reusable semantic paint/state provenance
* one reusable implementation per component family
* backend-free reusable core consumers
* curated public application/author APIs
* correct workspace/package boundaries
* all four applications using reusable APIs

Explicitly prevent resurrection of rejected architecture such as:

* old widget facade
* owned runtime component tree
* untyped action bus
* universal Widget/Theme abstraction
* application-owned generic overlay stack
* Grid SQL semantics
* generic RGB role inference
* superficial builder wrappers over legacy implementations
* parallel old/new APIs
* compatibility shims that allow the migration to avoid finishing

If the existing task graph accidentally permits one of these outcomes, repair it.

---

# Refactoring-task atomicity

Criticize task size aggressively.

A task is too broad if:

* multiple independent failures can occur inside it;
* its acceptance criteria cannot precisely identify which obligation failed;
* different components can be executed independently but are bundled together;
* it spans unrelated architecture and product behavior;
* it requires large amounts of discretionary implementation judgment;
* its verifier can pass while substantial work remains.

A task is too small if it creates unnecessary sequencing or verification overhead without establishing a useful independently consumed state.

Split, merge, reorder, or redefine tasks where justified.

If task numbering must change, preserve traceability and update every dependency/reference consistently.

Do not preserve `001..073` merely because those numbers already exist.

But also do not churn task IDs without a concrete correctness or execution-quality reason.

---

# Analyze contradictions

Build or refresh an explicit contradiction/adjudication ledger.

Look for conflicts between:

* historical goals
* current goals
* visual-baseline / current product behavior
* main architecture
* immutable oracle
* `docs/design/DESIGN.md`
* snapshots
* tests
* task contracts
* refactoring-plan documents
* task-format rules
* implementation state

For each contradiction:

1. state both claims;
2. provide evidence;
3. identify authority order;
4. decide which claim governs;
5. update all affected tasks/docs;
6. preserve historical context where useful;
7. eliminate ambiguity for future executors.

A future executor should never have to guess which conflicting document to follow.

---

# Bidirectional traceability audit

Rebuild/check traceability end to end.

At minimum prove coverage for:

* requirement -> task
* task -> requirement
* historical accepted decision -> task
* architecture obligation -> task
* component -> task
* application route/state -> task
* application route/state -> visual/behavioral proof
* acceptance criterion -> check
* check -> requirement
* snapshot/test -> protected behavior
* task output -> dependent task

Find:

* orphan requirements
* orphan tasks
* duplicate/conflicting ownership
* behavior with no task
* task with no authoritative requirement
* verification with no protected behavior
* snapshot with unclear authority
* architecture decision with no completion gate

Repair all material gaps.

---

# Adversarial verifier review

For every task, ask:

> Could a competent but goal-seeking executor satisfy the literal verifier while violating the intended requirement?

Try to find bypasses.

Examples:

* implementing a fixture-specific special case
* editing tests within writable paths
* changing generated evidence
* satisfying grep-style architecture checks without delivering semantics
* leaving alternate legacy paths available
* passing one snapshot while breaking another state
* satisfying compile checks while controls are inert
* duplicating reusable behavior inside an app
* changing test data to match incorrect implementation
* using a mock that no longer proves production behavior
* hiding regressions behind conditional compilation
* skipping checks
* weakening lint configuration
* returning success without exercising the behavior
* producing a receipt against the wrong source tree

Where such bypasses exist, strengthen the task contract or trusted verifier.

Prefer deterministic proof over more prose.

---

# Validate the plan itself

After repairing individual tasks, validate the entire campaign.

Verify:

1. every task is executable from the state guaranteed by its declared predecessors;
2. every produced artifact has an explicit consumer or closure purpose;
3. parallel tasks do not have conflicting writable ownership;
4. integration points are represented explicitly;
5. oracle comparison happens before any candidate can redefine expectations;
6. application work cannot bypass reusable architecture;
7. architecture work cannot claim success while applications visually regress;
8. final closure tasks prove both architecture and user-visible parity;
9. the final verified tree is actually merge-ready;
10. no task relies on undocumented human intervention.

---

# Required validation

Run every available deterministic planning/task validator.

At minimum investigate and use the appropriate current equivalents of:

* `taskfmt` project/group/task linting
* task-format schema validation
* DAG validation
* dependency validation
* task package validation
* traceability validation
* repository-specific planning validation
* snapshot store integrity tests
* ordinary Rust formatting/check/lint/tests relevant to files you modify

Do not claim the task catalog is valid only because TOML parses.

If an appropriate deterministic validation tool is missing and can reasonably be added without executing product refactoring, add the reusable validator/test rather than relying forever on manual review.

---

# Allowed changes

You are expected to edit and improve the planning system.

Change whatever is necessary inside the planning/task specification, including:

* `docs/refactoring/**`
* `docs/refactoring-plan/**`
* `refactoring-tasks/**`
* traceability ledgers
* task indexes
* dependency metadata
* verification definitions
* trusted planner-owned verification material
* planning validation scripts/tests where appropriate

Do not be afraid to:

* rewrite incorrect tasks
* split tasks
* merge tasks
* add missing tasks
* remove truly obsolete tasks
* repair dependencies
* strengthen verifiers
* update stale findings
* resolve contradictions
* rewrite acceptance criteria
* add missing non-regression requirements
* expand snapshot/behavior coverage requirements
* correct stale task-format usage

But do **not** implement the actual production refactoring under this goal.

---

# Forbidden changes

Unless strictly required for a planning validator and clearly justified, do not modify production implementation.

Absolutely do not:

* perform the architecture migration
* merge branches
* push/merge to `main`
* change product UX
* redesign components
* bless changed UI
* update expected snapshots to fit candidate behavior
* weaken tests
* remove regression evidence
* repin the oracle
* auto-merge external `tui-snap` work
* mark tasks completed merely because related code already exists somewhere
* conflate "main has similar code" with "requirement is proven"

---

# Independent final review

After you believe the plan is repaired, stop authoring and spawn fresh independent reviewers.

They must review the resulting state, not your summaries.

Assign at least:

* task-format reviewer
* architecture reviewer
* visual-regression reviewer
* application-parity reviewer
* verifier/adversarial reviewer
* DAG/atomicity reviewer

Tell them explicitly:

> Assume the planning team made mistakes. Find concrete reasons this campaign could still execute successfully according to its tasks while breaking functionality, violating architecture, or changing UI/UX.

Collect every finding.

For each finding:

* ACCEPT and repair it, or
* REJECT with concrete evidence.

Then rerun all validation.

Repeat review/repair until no material unresolved finding remains.

---

# Definition of done

This goal is complete only when all of the following are true:

1. Every file in `docs/refactoring/**` has been reviewed against current evidence.
2. Every file in `docs/refactoring-plan/**` has been reviewed against current evidence.
3. Every task package in `refactoring-tasks/**` has been individually reviewed.
4. The catalog conforms to the actual current/pinned `task-format` authority.
5. Every task is bounded, executable and deterministically verifiable.
6. The DAG is correct and exposes maximal safe parallelism.
7. All material requirements have task ownership.
8. All material task requirements have deterministic proof.
9. All important application states affected by the refactor have regression protection.
10. `snapshots/` and oracle evidence cannot be changed by task executors.
11. No visual-output change can silently pass.
12. No architecture regression can silently pass.
13. No app-local compatibility implementation can satisfy a reusable-component task.
14. All contradictions have explicit adjudication.
15. Traceability is complete in both directions.
16. Stale task assumptions have been corrected.
17. Current `main` implementation facts used by the plan have been reverified where material.
18. Final closure tasks prove architecture, functionality, interaction and visual parity together.
19. Fresh independent reviewers have attacked the repaired plan and all material findings have been resolved.
20. All deterministic planning/task validation passes.

The final result should be a refactoring campaign that can be handed to independent coding agents with minimal interpretation and where **a successful deterministic gate actually means the refactoring preserved the product while completing the intended architecture**.

---

# Final report

At the end, provide a concise but evidence-based report containing:

* current `visual-baseline` SHA audited
* `task-format` SHA used as authority
* files changed
* number of tasks before/after
* tasks added/removed/split/merged
* dependency changes
* material contradictions resolved
* stale assumptions corrected
* verification weaknesses fixed
* visual/snapshot coverage gaps fixed or explicitly blocked
* traceability gaps fixed
* independent-review findings and dispositions
* validation commands executed and results
* unresolved blockers, if any
* exact readiness verdict:

`READY FOR REFACTORING EXECUTION`

or

`NOT READY FOR REFACTORING EXECUTION`

Do not return `READY` while any material correctness, verification, visual-parity, architecture, dependency, or traceability gap remains.
