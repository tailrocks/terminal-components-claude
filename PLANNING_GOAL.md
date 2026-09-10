# /goal — Produce the complete execution plan for finishing `terminal-components-claude` refactoring without changing UI/UX

Repository:

https://github.com/donbeave/terminal-components-claude

## Mission

The `main` branch was in the middle of a substantial architectural/API refactoring that was interrupted before completion.

As a result, `main` is currently in an inconsistent, partially broken state:

* parts of the refactoring are implemented;
* parts are unfinished;
* some components or applications no longer work correctly;
* some visual behavior has regressed;
* some previews are substantially different from the intended design;
* historical refactoring documents contain important decisions and partially completed plans that must be reconstructed rather than ignored.

At the same time, the older `holla` branch represents the known-good application experience before the architectural refactoring:

https://github.com/donbeave/terminal-components-claude/tree/holla

Its architecture/API is older and must **not** be treated as the desired architecture, but its application previews, flows, visual behavior, and overall user experience are substantially correct.

The goal is therefore:

> Finish the architecture/API refactoring represented by `main` and its history while restoring and preserving the exact user-visible behavior of the known-good Holla-era implementation.

This `/goal` is **not the implementation of that refactoring**.

This `/goal` must perform the complete investigation, reconstruction, decomposition, verification design, and planning work required so that the resulting task graph can later be executed mechanically and safely.

The final result must be an execution-ready set of detailed tasks with explicit dependencies and deterministic verification.

---

# 1. Immutable UI/UX source of truth

For **all user-visible behavior**, the authoritative source of truth is this tag:

https://github.com/donbeave/terminal-components-claude/tree/holla-fable-2026-09-10

Resolve the tag before doing any analysis and record its exact commit SHA.

At the time this goal was written, the annotated tag resolves to:

`02f5294bfdbf38004cc49130d0aff1d01f31434c`

Verify that before relying on it.

This immutable tag is the acceptance oracle for:

* layout;
* spacing;
* dimensions;
* alignment;
* borders;
* separators;
* glyphs and icons;
* text;
* colors;
* themes;
* focus states;
* hover states;
* pressed states;
* selected states;
* disabled states;
* active/inactive states;
* mouse interaction;
* keyboard interaction;
* focus traversal;
* scrolling;
* scroll boundaries;
* scroll-edge fades/transparency;
* cursor behavior;
* text editing behavior;
* selection;
* menus;
* dialogs;
* overlays;
* popovers;
* tables and grids;
* truncation;
* wrapping;
* status bars;
* empty states;
* error states;
* navigation;
* resizing;
* terminal-size-dependent behavior;
* application flows;
* every other observable UI/UX characteristic.

The following applications/previews must all be covered:

* `showcase`
* `holla`
* `jackin`
* `tablepro`

## Strict parity rule

The final refactored implementation must be **1:1 equivalent** to this reference from the user's perspective.

"Similar", "close enough", "improved", or "modernized" are not acceptable.

Do **not** redesign anything during this refactor.

Do **not** alter colors, spacing, interaction patterns, layouts, flows, hover behavior, focus behavior, scrolling, component appearance, or application composition merely because the refactored implementation could be cleaner.

We are changing the **underlying implementation, architecture, APIs, component boundaries, and internal structure — not the product experience**.

If current `holla` and the immutable `holla-fable-2026-09-10` tag disagree on user-visible behavior, the immutable tag wins.

The moving `holla` branch can still be inspected for implementation context, but it is not allowed to silently redefine the visual contract.

---

# 2. Architectural/refactoring source of truth

The visual source of truth and architectural source of truth are intentionally different.

For architectural/API intent, reconstruct the entire refactoring effort represented by `main` and its history.

Read the current contents **and every materially relevant historical version** of at least these documents:

### Holla-era planning

https://github.com/donbeave/terminal-components-claude/blob/holla/IMPROVEMENTS_PLAN.md

### Refactoring architecture/state

https://github.com/donbeave/terminal-components-claude/blob/main/COMPONENT_ARCHITECTURE.md

https://github.com/donbeave/terminal-components-claude/blob/main/HANDOFF_SLICE4_WAVE1.md

https://github.com/donbeave/terminal-components-claude/blob/main/REFACTORING_GOAL.md

https://github.com/donbeave/terminal-components-claude/blob/main/REFACTORING_STATE.md

https://github.com/donbeave/terminal-components-claude/blob/main/GOAL.md

https://github.com/donbeave/terminal-components-claude/blob/main/GOAL2.md

Do not merely read the versions that exist at current HEAD.

Perform git archaeology.

For each document, inspect its history across relevant branches and commits, including renamed/deleted versions where applicable.

Use repository history such as:

* `git log --all --follow -- <path>`
* `git log --all -p -- <path>`
* relevant commit diffs;
* merge bases;
* branch divergence;
* commits associated with each refactoring slice/wave;
* files added or removed by those commits;
* historical tests and baselines;
* historical review/adjudication documents;
* relevant handoff documents;
* relevant audit/review documents;
* relevant GitHub PRs/issues/discussions where they materially clarify a decision.

Do not assume the earliest or most prominent statement in a document is still authoritative.

Specifically identify statements that were:

* proposed;
* accepted;
* rejected;
* superseded;
* amended;
* partially implemented;
* completely implemented;
* subsequently broken;
* deferred;
* abandoned.

Build a chronological understanding of what the refactoring was trying to accomplish and how far it actually got.

---

# 3. Authority hierarchy

Use this hierarchy whenever sources disagree.

### User-visible behavior

1. `holla-fable-2026-09-10` immutable tag / resolved commit.
2. Deterministic evidence captured from that tag.
3. `holla` branch as supporting implementation context.

### Architecture/API/refactoring intent

1. Accepted and non-superseded architectural decisions reconstructed from repository history.
2. Latest authoritative refactoring state/checkpoints and their supporting commits.
3. Actual implemented refactoring code on `main`.
4. Earlier plans and historical documents as evidence.

### Task specification

Use the current version of:

https://github.com/donbeave/task-format

and specifically:

https://github.com/donbeave/task-format/tree/main/reference/task-template

Do not invent a custom task schema.

### Verification

Use:

https://github.com/donbeave/tui-snap

together with normal deterministic Rust tests, architecture checks, API checks, compile checks, and other appropriate gates.

---

# 4. Do not blindly merge branches

Do not begin from the assumption that a normal Git merge of `holla` into `main`, or `main` into `holla`, is the correct solution.

First reconstruct the branch topology.

Analyze:

* merge base;
* commit ranges;
* refactoring commits on `main`;
* post-divergence Holla work;
* overlapping files;
* deleted/renamed/moved components;
* architectural changes;
* application-level changes;
* tests and baseline changes.

Determine the safest eventual integration strategy.

The intended end state is:

> a branch that preserves the known-good Holla user experience while incorporating the completed refactored architecture/API, and can then safely replace/merge into `main`.

The plan must explicitly explain whether subsequent execution should:

* continue from `holla`;
* create an integration branch from `holla`;
* continue from `main`;
* create an integration branch from `main`;
* reconstruct selected main refactoring work on top of Holla;
* selectively transplant/reimplement commits;
* or use another evidence-backed strategy.

Do not choose based on convenience.

Choose the strategy that minimizes:

* lost architectural work;
* visual regressions;
* behavioral regressions;
* hidden merge damage;
* duplicated implementation;
* temporary compatibility layers;
* untestable intermediate states.

Record the reasoning.

---

# 5. Required investigation

Use subagents aggressively and in parallel for **all substantial investigation and review work**.

The coordinator should synthesize evidence, not perform the entire audit serially.

At minimum assign independent subagents to the following areas.

## A. Git/refactoring historian

Reconstruct:

* branch divergence;
* refactoring chronology;
* slice/wave history;
* relevant commits;
* interruptions;
* superseded plans;
* unfinished work;
* regressions;
* latest accepted architectural decisions.

Produce an evidence-backed timeline.

## B. Architecture/API auditor

Compare:

* old Holla-era architecture/API;
* current `main`;
* intended architecture from historical refactoring documents;
* actual implementation state.

Create a component/API migration matrix with states such as:

* not started;
* partially refactored;
* refactored correctly;
* refactored but behavior regressed;
* duplicated;
* incompatible;
* obsolete;
* superseded;
* broken;
* needs migration;
* needs deletion.

## C. UI/UX parity auditor

Treat `holla-fable-2026-09-10` as the oracle.

Inventory every important:

* component;
* screen;
* application;
* state;
* interaction;
* flow.

Compare that reference against current `main`.

Do not restrict this to screenshots.

Cover interaction behavior as well.

## D. Application-flow auditors

Audit independently:

* Showcase
* Holla
* Jackin
* TablePro

For each application identify:

* screens;
* modes;
* navigation;
* keyboard commands;
* mouse interactions;
* focus states;
* overlays;
* dialogs;
* scrollable areas;
* editable areas;
* data presentation;
* resizing behavior;
* component dependencies.

Map every relevant flow to the underlying reusable components it exercises.

## E. Component-system auditor

Build a complete inventory of:

* primitives;
* components;
* composite components;
* layout utilities;
* styling/theme APIs;
* interaction infrastructure;
* focus infrastructure;
* event/input handling;
* scrolling;
* overlays/layers;
* forms/fields;
* collections;
* tables/grids;
* code/diff views;
* testing helpers;
* public API;
* compatibility APIs.

Identify which refactoring obligations apply to each.

## F. Verification specialist

Design how every meaningful requirement can be proven automatically.

Review `tui-snap` in depth before assuming it lacks a feature.

Use its appropriate testing modes for:

* deterministic component/view rendering;
* full application rendering;
* PTY-driven interactions;
* keyboard input;
* mouse input;
* resize;
* scrolling;
* focus transitions;
* dialogs/overlays;
* stateful flows.

## G. Task-graph planner

Translate the reconciled architecture and parity requirements into bounded, dependency-linked task packages.

## H. Independent reviewers

After the first plan exists, use separate fresh-context subagents to review it from at least these perspectives:

1. architecture/API completeness;
2. UI/UX and interaction parity;
3. task decomposition/dependencies;
4. verification sufficiency;
5. merge/integration safety.

Reviewers must try to find omissions rather than simply agree with the plan.

---

# 6. Build explicit evidence matrices before task decomposition

Do not begin creating tasks until the investigation has produced a sufficiently complete factual model.

Create at least the following matrices.

## Refactoring obligation matrix

For every requirement/decision found in the historical plans:

* source document;
* historical revision/commit;
* decision/requirement;
* current authority status;
* implementation commit if any;
* files/components affected;
* current `main` status;
* remaining work;
* relevant tests/gates;
* eventual task IDs.

Every meaningful unfinished refactoring requirement must map to at least one task.

## Component parity matrix

For every reusable component/component family:

* reference implementation/state;
* current main implementation;
* architectural target;
* visual parity status;
* interaction parity status;
* API/refactor status;
* tests available;
* tests missing;
* owning task IDs.

## Application parity matrix

For Showcase, Holla, Jackin, and TablePro:

* screen/flow;
* reference state;
* current main state;
* components involved;
* visual proof;
* interaction proof;
* current defect/regression;
* task IDs responsible for correcting it.

## Historical decision ledger

Record:

* accepted decisions;
* rejected approaches;
* superseded decisions;
* unresolved historical ideas;
* actual current authority.

This prevents an executor from accidentally resurrecting a rejected design.

---

# 7. Establish the parity baseline before planning implementation

The later refactor cannot be proven safe without a deterministic before/after contract.

Design a comprehensive baseline derived from:

`holla-fable-2026-09-10`

It must cover representative terminal sizes and all meaningful user-visible states.

Do not equate "one snapshot per screen" with complete coverage.

The baseline should include, where relevant:

* default;
* focus;
* hover;
* pressed;
* active;
* selected;
* disabled;
* empty;
* populated;
* scrolling at top;
* scrolling in middle;
* scrolling at bottom;
* scroll-edge fade behavior;
* menu open/closed;
* dialog open/closed;
* overlay stacking;
* edit mode;
* cursor movement;
* text selection;
* validation/error states;
* narrow viewport;
* standard viewport;
* resize transitions;
* mouse interaction;
* keyboard navigation.

For interaction-heavy behavior, pair visual frames with deterministic action sequences.

Example conceptually:

`initial -> key/mouse event -> expected state -> expected frame`

A screenshot alone is insufficient evidence for behavior such as:

* focus movement;
* scroll routing;
* click hitboxes;
* keyboard commands;
* editing;
* overlay capture;
* dialog dismissal;
* state transitions.

---

# 8. Use `tui-snap` as the verification foundation

Verification project:

https://github.com/donbeave/tui-snap

Inspect its current `main` and documentation before designing new tooling.

Prefer existing capabilities whenever possible.

Use both appropriate modes:

* deterministic production-view/frame testing;
* PTY/full-executable testing for real interaction behavior.

The plan must define how `tui-snap` will be used to prove equivalence against the reference tag.

The final verification strategy must be **fail-closed**:

* the golden/reference evidence comes from the immutable reference;
* executors cannot silently bless a changed refactored output;
* a mismatch fails;
* accepting a new visual result must never become an automatic way of hiding a regression.

Do not rely exclusively on image-level comparison if the canonical frame/cell representation can provide stronger deterministic evidence.

Use PNG/HTML diff artifacts for human review where valuable, while keeping deterministic machine equality as the gate where appropriate.

## If `tui-snap` is insufficient

If — and only if — a required verification capability genuinely cannot be represented using its existing APIs:

1. prove the missing capability;
2. design the smallest generally reusable improvement;
3. implement it in `donbeave/tui-snap`;
4. add its own tests;
5. preserve existing behavior;
6. create a dedicated branch;
7. create a PR to `donbeave/tui-snap`;
8. record the PR and dependency in this project's final plan.

Do not add project-specific hacks to `tui-snap`.

Any addition should be a generally useful TUI verification primitive.

Do not merge the `tui-snap` PR automatically unless repository policy explicitly requires that.

---

# 9. Task format is mandatory

Every implementation task in the resulting plan must use:

https://github.com/donbeave/task-format

and the **current** canonical template:

https://github.com/donbeave/task-format/tree/main/reference/task-template

Read the current task-format source and documentation before generating tasks.

Do not rely on a remembered schema.

Use the actual current schema and dependency model.

Each task must be a genuine execution-ready task package, not merely a Markdown bullet.

At minimum each task contract must contain the canonical equivalents of:

* Goal;
* Context;
* Preconditions;
* Scope;
* explicit in-scope work;
* explicit out-of-scope work;
* Requirements;
* MUST requirements;
* MUST NOT requirements;
* non-regression requirements;
* Acceptance criteria;
* typed verification;
* requirement-to-acceptance mapping;
* Fixed decisions;
* Checklist;
* deterministic verification configuration.

Use `verify.toml` according to the current task-format schema.

Use trusted fixtures/evidence where appropriate.

Do not invent fields unsupported by task-format.

If task-format has project/group/dependency metadata for expressing the DAG, use that mechanism exactly.

---

# 10. Task-writing quality bar

A future executor should be able to pick up any ready task and execute it without needing to rediscover the project design.

Every task must answer:

* What observable outcome must become true?
* Why does it exist?
* What historical architectural requirement does it satisfy?
* What user-visible behavior must remain identical?
* Which files/component families are likely involved?
* What is explicitly outside its scope?
* What does it depend on?
* Which tasks depend on it?
* Which architectural decisions are fixed?
* Which tempting alternatives are forbidden?
* Which reference states from the golden tag apply?
* Which exact acceptance scenarios prove success?
* Which deterministic checks verify those scenarios?
* Which app(s) exercise the changed component?
* What non-regressions are required?

Avoid vague tasks such as:

* "fix components";
* "finish refactoring";
* "improve tests";
* "make UI match";
* "clean up architecture".

Break them into bounded observable outcomes.

Prefer coherent, independently verifiable vertical slices.

Do not split so aggressively that the repository cannot remain buildable/testable between tasks.

---

# 11. Dependency graph

Produce an explicit DAG.

Do not merely number tasks in an arbitrary order.

For each task identify:

* hard dependencies;
* soft dependencies, if any;
* tasks that can execute in parallel;
* tasks that must be serialized;
* gates required before downstream work can begin.

Find the real critical path.

Use historical "Slice" and "Wave" terminology as evidence, but do not blindly preserve the old decomposition if repository reality now suggests a safer task graph.

Every task should leave the repository in a coherent state suitable for the next dependency.

---

# 12. Preserve reusable architecture

The resulting architecture must not achieve parity by copying application-specific code everywhere.

The plan must ensure the intended refactor produces reusable components/APIs and that:

* Showcase uses them;
* Holla uses them;
* Jackin uses them;
* TablePro uses them.

Look explicitly for:

* duplicated styling;
* duplicated interaction logic;
* application-local versions of reusable components;
* compatibility shims that should disappear;
* public APIs bypassed by examples;
* test-only behavior differing from production behavior;
* old and new architecture existing simultaneously;
* legacy implementations accidentally retained.

Visual parity must be achieved through the refactored reusable system, not through one-off app patches.

---

# 13. API/refactoring verification

Visual parity is necessary but not sufficient.

The task graph must also verify architecture.

Depending on what the repository currently supports, tasks should include deterministic checks for things such as:

* forbidden legacy imports;
* dependency boundaries;
* public API surface;
* component registry completeness;
* module ownership;
* old/new implementation duplication;
* deprecated path removal;
* architectural documentation references;
* compile gates;
* MSRV if still authoritative;
* clippy;
* formatting;
* tests;
* no-default-features where applicable;
* documentation builds;
* performance/non-regression gates where historically required.

Only include checks supported by reconstructed accepted requirements.

Do not blindly preserve obsolete gates.

---

# 14. No visual-regression task may "fix" the baseline

Golden evidence derived from the source-of-truth tag must be planner/trusted evidence.

An implementation task must not be able to pass by:

* overwriting the expected snapshot;
* blessing a changed output;
* weakening comparison;
* deleting a scenario;
* removing a failing fixture;
* suppressing the gate;
* changing the reference commit.

Where task-format supports trusted verification inputs, use them.

Design the task packages so an executor cannot redefine success from inside its writable scope.

---

# 15. Planning artifacts

Create a durable top-level plan in the repository.

Unless the repository already has a stronger established location/convention, use:

`REFACTORING_COMPLETION_PLAN.md`

The document should contain:

1. Executive goal.
2. Source-of-truth hierarchy.
3. Resolved reference SHAs.
4. Repository/branch topology.
5. Refactoring history timeline.
6. Historical decision ledger.
7. Current-state assessment.
8. Refactoring obligation matrix.
9. Component parity matrix.
10. Application parity matrix.
11. Verification architecture.
12. `tui-snap` assessment.
13. Chosen integration strategy.
14. Dependency DAG.
15. Execution waves.
16. Task index.
17. Per-task purpose/dependencies.
18. Parallelization opportunities.
19. Critical path.
20. Merge-readiness gate.
21. Known risks.
22. Explicit non-goals.
23. Evidence and commit references.
24. Independent-review findings and resulting corrections.

Also create the actual task-format task packages in an appropriate repository directory.

If no existing convention exists, use a clearly named directory such as:

`refactoring-tasks/`

But first inspect repository conventions and task-format conventions rather than blindly creating that path.

---

# 16. End-to-end completion gate

The plan must culminate in a final integration task/gate proving that the complete refactoring is finished.

That final gate must establish at least:

### Build correctness

All authoritative build/test/lint/architecture gates pass.

### Architectural completion

No unfinished refactoring path remains.

No obsolete old/new duplicate architecture remains unless explicitly accepted.

### API completion

All reusable components are exposed and consumed according to the intended architecture.

### Application completeness

All four application surfaces work:

* Showcase
* Holla
* Jackin
* TablePro

### Visual parity

For every covered golden state:

`final refactored output == holla-fable-2026-09-10 reference output`

according to the selected deterministic parity representation.

### Interaction parity

Golden interaction scenarios behave identically for:

* keyboard;
* mouse;
* focus;
* editing;
* scrolling;
* overlays;
* resize;
* navigation;
* important application flows.

### No hidden baseline drift

The final result must prove that expected reference artifacts were not simply regenerated from the refactored implementation.

### Merge readiness

The resulting integration branch is safe to merge into/replace `main` under the integration strategy selected by the plan.

---

# 17. Independent review is mandatory

Do not treat the first generated plan as final.

After producing the complete task graph, give it to fresh subagents that did not author those portions.

Tell them explicitly to attack the plan.

Ask them to identify:

* missing historical requirements;
* missing components;
* missing app states;
* missing interactions;
* incorrect dependencies;
* oversized tasks;
* underspecified tasks;
* unverifiable acceptance criteria;
* architecture loopholes;
* ways an executor could pass while breaking UX;
* ways an executor could preserve UX while failing the architectural goal;
* unsafe branch assumptions;
* missing cleanup;
* missing final integration gates.

For every finding:

* accept and repair it; or
* reject it with evidence.

Run another review after material corrections.

Do not stop at "reviewed".

Stop when the task graph is internally consistent and reviewers cannot identify a material uncovered obligation without contradicting repository evidence.

---

# 18. Traceability requirement

Everything important must be traceable in both directions.

It must be possible to answer:

### Requirement → task

Which task implements this historical architectural requirement?

### Task → requirement

Why does this task exist?

### Component → task

Which task completes/refactors this component?

### Application state → verification

Which scenario proves this UI state did not regress?

### Verification → requirement

What specific requirement does this check prove?

### Historical decision → implementation

Where is this accepted decision represented in the future task graph?

No orphan requirements.

No orphan tasks.

No important golden states without verification.

---

# 19. Scope of this `/goal`

For `donbeave/terminal-components-claude`, this goal is **planning and preparation only**.

Do not execute the actual refactoring tasks yet.

Do not merge into `main`.

Do not start opportunistically fixing terminal-components code simply because you discover a defect.

The deliverable is the authoritative execution plan and task graph.

The only permitted implementation exception is:

> improving `donbeave/tui-snap` when a genuinely missing reusable verification capability is required to make the planned terminal-components tasks deterministically verifiable.

That work must happen in `tui-snap` and through a separate PR.

---

# 20. Autonomous execution

Work autonomously.

Do not ask me questions.

When something is unclear:

1. inspect the code;
2. inspect git history;
3. inspect historical plans;
4. inspect tests;
5. inspect snapshots/baselines;
6. inspect related reviews/adjudications;
7. delegate investigation to subagents;
8. make the most evidence-backed conservative decision.

If genuine ambiguity remains, record:

* the ambiguity;
* competing interpretations;
* evidence for each;
* the chosen interpretation;
* why it is the least risky/reversible choice.

Do not stop and ask for clarification.

---

# 21. Important prohibitions

Do not:

* redesign the UI;
* "improve" UX as part of refactoring;
* normalize away intentional Holla behavior;
* accept approximate visual parity;
* rely only on screenshots;
* trust current `main` visually;
* trust an old "PASS" without rerunning/reconstructing evidence;
* blindly copy `main`;
* blindly copy `holla`;
* blindly merge branches;
* blindly follow stale refactoring prose;
* resurrect rejected architectural ideas;
* treat partially implemented code as an accepted decision merely because it exists;
* weaken tests to accommodate the refactor;
* regenerate golden baselines from the new implementation;
* generate vague tasks;
* create tasks without deterministic acceptance criteria;
* create tasks without dependencies;
* leave historical decisions unmapped;
* stop after producing an initial plan without independent review.

---

# 22. Definition of success for this planning goal

This `/goal` is complete only when all of the following are true:

1. The immutable UI/UX reference is pinned.
2. Branch history and divergence are understood.
3. Historical refactoring documents and their meaningful versions have been reconstructed.
4. Accepted/rejected/superseded decisions are distinguished.
5. Current `main` refactoring state is known component by component.
6. All four applications are inventoried against the golden reference.
7. Important user-visible states and flows are inventoried.
8. The verification strategy can deterministically detect visual and behavioral regressions.
9. `tui-snap` capability gaps, if any, are resolved or represented by an actual PR dependency.
10. Every remaining architectural obligation is mapped to an implementation task.
11. Every important parity obligation is mapped to automated verification.
12. Every implementation task follows current `task-format`.
13. Every task has explicit scope, requirements, acceptance criteria, verification, fixed decisions, and dependencies.
14. The complete DAG is valid and execution-ready.
15. Parallelizable and serialized portions are identified.
16. A final integration/merge-readiness gate exists.
17. Independent subagents have reviewed the plan.
18. Material review findings have been incorporated or evidence-backed rejected.
19. No known requirement is orphaned.
20. No known important UI/UX behavior is left without an owner and proof.

The final deliverable should be sufficiently precise that the next `/goal` can execute the task graph without having to redo this architectural research or infer what "correct UI/UX" means.

