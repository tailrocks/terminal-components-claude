# Exact implementation tasks

## Use these instructions

Read [PLAN.md](PLAN.md) before you start.
Use [AGENT-RUNBOOK.md](AGENT-RUNBOOK.md) for every worker and review call.
Use [API-RULES.md](API-RULES.md) for every public or shared API change.
Use [TESTING-RULES.md](TESTING-RULES.md) for every test or harness design.

All tasks below are future work.
No task is accepted by this planning review.
The coordinator must update task state from actual evidence.

T00 captures the complete legacy evidence before deletion.
T37 is the first Termrock source change. It removes all legacy tests and nonfunctional code comments.
T01 inventories the cleaned source. T38 defines a reviewed test architecture before implementation work.
Task test cases describe required outcomes. They do not authorize edits to test code by themselves.
Every test or harness edit needs an approved task plan ID and a different Sol medium reviewer.
All existing test paths and commands are proposed replacement targets after review.
They are not instructions to retain or execute deleted legacy targets.

## Prohibited model

Never use any Astra model under any circumstances or in any scenario.
This includes `gpt-6-astra`, all Astra variants, fallbacks, reviews, planning, and design escalation.
This prohibition applies to the coordinator, every worker, and all nested agents.

Use GPT-6 Luna with max effort for all implementation.
Use a separate GPT-6.1-Sol agent at the stated effort for review.
This model rule also applies to every child task.
Use GPT-6.1-Sol high to resolve a required product architecture or product contract change.
Keep the affected task blocked until GPT-6.1-Sol high records the product design decision.
Do not substitute another model or effort.

Start a task only after its predecessors are accepted.
The owner list gives the maximum source scope.
Before dispatch, resolve each broad entry to exact worker files.
Give each file one active writer.
Include the complete task, matrix cases, source findings, and API rules in each worker prompt.
Include its approved task-scoped test plan ID and reviewer before any test edit.

## Tests and command results

The commands are proposed future executions.
They do not report a pass and may name deleted legacy targets.
Use them only after the approved test plan names replacement targets.
T00 records the legacy inventory. T01 records the cleaned production inventory.
Replace fixed review commit IDs with recorded current IDs when a command requires current source.

A command list is a starting check set.
The required cases and [VALIDATION.md](VALIDATION.md) also apply.
Add exact selected test names to the task result.
Run real CLI visual scenarios through the reviewed replacement runner.
Keep the current capture runner only if T00 proves production ownership.
Do not execute legacy harness targets after T37.

T02, T06, and T11 may expose failures recorded before cleanup.
Keep each failure in the migration record with its source hash and owner task.
Reproduce behavior only through approved functional scenarios.
Do not report a gate as passed without selected replacement cases.
Accept only the bounded scanner, policy, lint, or theme work that the task completed.
T32 and T36 require the complete final gates to pass.

A green result applies only to declared cases and its recorded environment.
An empty suite after T37 is unverified. It never proves product behavior.
Before T22, page workers need an approved scenario through the production `Page` or factory API and public component API.
The fixture must use real owned page and component code, then observe public output.
This proves page behavior; it does not prove route registration.
T22 requires live CLI proof of registry routing.
T31 and final acceptance require live CLI evidence for every route and application journey.
Do not add a fixture or harness until its task-scoped design passes independent review.
Keep production registry edits with T22. Do not mark a page accepted from source inspection alone.

## Child task rules

Use the child work split in each task.
The coordinator may reduce simultaneous work to fit available slots.
It must preserve model selection and independent review.
Use one integration owner for shared declarations.
Do not run performance measurements and visual capture on the same host at the same time.

Every worker must return the structured record from [AGENT-RUNBOOK.md](AGENT-RUNBOOK.md).
Every reviewer must read the exact output commit.
A correction must return to a Luna max worker.
Only accepted integration evidence can close the task.

## Task index

| ID | Work | Predecessors | Sol review effort |
| --- | --- | --- | --- |
| [T00](#t00) | Capture source, legacy evidence, and approved rules | None | high |
| [T01](#t01) | Create one complete requirement and API inventory after cleanup | T37 | high |
| [T02](#t02) | Rebuild functional gate inputs and record historical CI failures | T00, T38 | high |
| [T03](#t03) | Remove backend leakage from pure library configurations | T01, T38 | high |
| [T04](#t04) | Prepare a source-bound visual reference candidate | T00, T01, T38 | high |
| [T05](#t05) | Make the facade feature set and API paths explicit | T03 | high |
| [T06](#t06) | Enforce crate direction and the lint policy | T05 | high |
| [T07](#t07) | Move Jackin screen code out of feedback | T06, T10, T11 | high |
| [T08](#t08) | Separate application domain state from UI state | T06 | high |
| [T09](#t09) | Preserve response effects and the frame lifecycle | T03, T06 | high |
| [T10](#t10) | Complete the existing composition and author API | T05, T09 | high |
| [T11](#t11) | Put all shared appearance rules in the theme owner | T06, T09 | high |
| [T12](#t12) | Complete controls and generic feedback | T10, T11 | high |
| [T13](#t13) | Complete fields, validation, and clipboard outcomes | T09, T11, T12 | high |
| [T14](#t14) | Make collection revisions and navigation identity explicit | T09, T11, T13 | high |
| [T15](#t15) | Complete overlay behavior and shared layer ownership | T10, T11, T13, T14 | high |
| [T16](#t16) | Complete Grid and Table as model-driven components | T11, T13, T14, T15 | high |
| [T17](#t17) | Complete text viewports and editor extension paths | T09, T11, T13, T15 | high |
| [T18](#t18) | Complete Form, FormDialog, and Wizard | T10, T12, T13, T14, T15 | high |
| [T19](#t19) | Use one shell policy for keys, hints, and protected leave | T09, T13, T15, T18 | high |
| [T20](#t20) | Give terminal session state one safe owner | T03, T06, T09 | high |
| [T21](#t21) | Adapt bounded input pumping without weakening frame rules | T09, T20 | high |
| [T22](#t22) | Connect all Showcase pages through one typed registry | T19, T23, T24, T25, T26 | high |
| [T23](#t23) | Build the five missing shell demonstrations | T10, T11, T19 | medium |
| [T24](#t24) | Build the five missing control and overlay demonstrations | T12, T14, T15 | medium |
| [T25](#t25) | Build the five missing data, layout, and text demonstrations | T11, T14, T16, T17 | medium |
| [T26](#t26) | Build the nine missing application-pattern demonstrations | T07, T08, T16, T17, T18, T19 | medium |
| [T27](#t27) | Finish Holla as a thin deterministic consumer | T08, T14, T15, T18, T19, T21 | high |
| [T28](#t28) | Finish Jackin Preview after the ownership moves | T07, T08, T15, T17, T18, T19, T21 | high |
| [T29](#t29) | Finish TablePro without moving SQL policy into Grid | T08, T13, T15, T16, T17, T18, T19, T21 | high |
| [T30](#t30) | Prove easy use and advanced customization in real consumers | T05, T10, T12, T13, T14, T15, T16, T17, T18, T19, T20 | high |
| [T31](#t31) | Close every required component and capability record | T12, T13, T14, T15, T16, T17, T18, T19, T22, T30 | high |
| [T32](#t32) | Connect the complete gate set to maintained CI | T02, T04, T06, T21, T27, T28, T29, T31, T33, T34 | high |
| [T33](#t33) | Prove bounded work and measure performance | T02, T09, T10, T14, T16, T17, T21, T22, T27, T28, T29 | high |
| [T34](#t34) | Verify crate packages outside the repository | T05, T06, T07, T08, T20, T30 | high |
| [T35](#t35) | Rewrite current guides and publish the migration instructions | T22, T27, T28, T29, T30, T31, T34 | medium |
| [T36](#t36) | Verify the final integrated commit and close the goal | T04, T32, T33, T34, T35, T37, T38 | high |
| [T37](#t37) | Remove legacy tests and nonfunctional source comments | T00 | high |
| [T38](#t38) | Approve the functional test architecture | T01 | medium |

<a id="t00"></a>

## T00: Capture source, legacy evidence, and approved rules

**Implementer:** GPT-6 Luna, max effort.
**Independent reviewer:** GPT-6.1-Sol, high effort.
**Predecessors:** None.
**Source findings:** [F20](REVIEW.md#f20).

### Owned files

- docs/implementation/pr38-final/
- AGENTS.md
- T00 source and legacy-evidence records under results/
- `results/run-state.json` in the imported package

Do not edit Termrock source, tests, harnesses, fixtures, doctests, CI inputs, or snapshots.

### Child work

- Use one Luna worker for source and tool records.
- Use a second Luna worker for the plan import and root links.
- Review both records before any Rust task starts.

### Required changes

1. Read the local plan package. Follow README.md to create and verify the separate target checkout.
2. Treat docs/implementation/pr38-final/ as absent until this task imports the package.
3. Fetch both pull request heads through the GitHub connection. Record the target, reference, base, and common-ancestor commit IDs.
4. Compare the current heads with the review IDs. Recheck each finding whose source changed.
5. Read root AGENTS.md, CLAUDE.md, current product contracts, and applicable directory instructions.
6. Copy this package to docs/implementation/pr38-final/. Add approved root links without removing existing rules.
7. Record every current CI failure, test inventory, harness, fixture, doctest, and comment directive before deletion.
8. Record production build failures from current CI logs. Do not run legacy test code.
9. Hash every captured test-related file and record its source commit and path.
10. Record production diagnostics, CLI help attributes, legal notices, generated files, and required parser, lint, and shebang directives.
11. Create separate source worktrees. Keep the reference worktree read-only. Record the integration branch and starting commit.
12. Install the pinned tools through mise. Record rustc, cargo, nextest, toolchain, and dependency-lock versions.
13. Create a result record with source IDs, tool versions, model assignments, and review limits.
14. Create `results/run-state.json` in the imported package. Record the SHA-256 plan hash, task states, worktree paths, source and output SHAs, and file locks.
15. Record each agent ID, role, model, and effort. Record approved test plans, reviewer IDs, evidence paths, blockers, and ready tasks.
16. Update `results/run-state.json` after every task-state, readiness, path, SHA, lock, agent, review, evidence, or blocker change.

### Required cases

- The source record uses complete commit IDs and matches the fetched branch heads.
- The root AGENTS links resolve. The existing rules remain present.
- The target checkout is separate from the plan-package checkout. The imported package path is created during T00.
- The run-state checkpoint records the plan hash, task states, paths, SHAs, and locks.
- The checkpoint records each agent ID, role, model, and effort.
- The checkpoint records approved test plans, reviewer IDs, evidence, blockers, and ready tasks.
- A changed target finding has a fresh source check. An unchanged finding retains its original evidence.
- Current production build failures are recorded without running legacy test code.
- The legacy inventory includes every language, test target, test function, harness, fixture, doctest, and test-only dependency.
- The directive inventory identifies each comment that parsers, linters, shells, licenses, or generators require.
- No Termrock source, test, harness, fixture, CI, or snapshot file changes in T00.

### Commands

~~~sh
git status --short
git rev-parse HEAD
git merge-base 5b3dbd7ae799f325efbb4f4cc11b2f5518dec948 e08ac57d10ae54c87337eae0be08b1f5e8f4bda4
mise install
rustc --version --verbose
cargo nextest --version
git ls-files | rg '(^|/)(tests?|fixtures?)(/|$)|\.rs$|\.py$|\.sh$|\.toml$'
~~~

### Acceptance

The source and model records are complete. The rules and run-state checkpoint are installed. Required unavailable tools have explicit blocked gates.

<a id="t01"></a>

## T01: Create one complete requirement and API inventory after cleanup

**Implementer:** GPT-6 Luna, max effort.
**Independent reviewer:** GPT-6.1-Sol, high effort.
**Predecessors:** T37.
**Source findings:** [F12](REVIEW.md#f12), [F13](REVIEW.md#f13), [F20](REVIEW.md#f20).

### Owned files

- component-ownership.json
- tools/acceptance_policy.json
- docs/implementation/pr38-final/requirements.json
- docs/contracts/migration.md

### Child work

- Split the inventory by library families, Showcase routes, and application requirements.
- Give each worker a separate result fragment.
- Use one Luna owner to combine the fragments and check missing IDs.

### Required changes

1. Use the current source to list production crates, public exports, component definitions, aliases, state types, actions, and builders.
2. Use T00's immutable records for deleted test targets, failures, comment directives, source hashes, and workload definitions.
3. Do not invoke a deleted test, harness, fixture, doctest, or test-only helper.
4. Preserve W01 through W45. Add IDs for omitted reusable components. Record each implementation owner and each public re-export separately.
5. Include RadioField, Note, TextSection, SelectField, Table, CompletingCodeEditor, FormDialog, AppShell, MasterDetail, SplitBlock, Notice, DerivedHintBar, and TextReport.
6. Map every one of the 54 reference routes to its current factory, shared components, and required behaviors.
7. Read the retained architecture and application migration records. Give each required historical item a current source and task disposition.
8. Use KEEP, ADAPT, RETIRE, or UNRESOLVED for each item. RETIRE needs a concrete conflict or replacement and a review decision.
9. Reclassify the 4,511 historical coverage rows as requirements, not active tests. Separate aliases, values, behavior, removed test APIs, and product paths.
10. Correct stale route and NULL-sort claims. Preserve historical records instead of rewriting them as current results.
11. Rebuild performance workload definitions from T00. Keep missing measurements visible for T33.

### Required cases

- No W ID is removed or reused. PropsList and ScrollRegion record their actual definition owners.
- Every public component and all 54 routes have an owner, task, required cases, and evidence state.
- Every required historical item has a disposition. Age alone is not a retirement reason.
- An intentionally removed export causes the inventory check to fail until its migration row changes.

### Commands

~~~sh
cargo metadata --locked --format-version 1 --no-deps
git status --short
~~~

### Acceptance

The inventory covers current source and retained requirements. Every unresolved requirement has a named task and blocks final completion.

<a id="t02"></a>

## T02: Rebuild functional gate inputs and record historical CI failures

**Implementer:** GPT-6 Luna, max effort.
**Independent reviewer:** GPT-6.1-Sol, high effort.
**Predecessors:** T00, T38.
**Source findings:** [F01](REVIEW.md#f01), [F18](REVIEW.md#f18).

### Owned files

- tools/acceptance_policy.json
- tools/acceptance_architecture.py
- crates/termrock-xtask/src/ownership/
- integration/functional/gates/
- result records for historical CI failure migration

### Child work

- Split production ownership policy and functional gate scenarios into separate Luna tasks.
- Give each task an approved test-plan ID before test edits.
- Use an independent Sol medium reviewer for each test or harness diff.

### Required changes

1. Fetch current-head CI records. Preserve the reviewed failures as historical evidence in T00's source record.
2. Reproduce required behavior failures through task-scoped functional cases from T38 and independent Sol medium review.
3. Do not copy old test names or assertions. Do not change expected bytes to hide a behavior difference.
4. Replace obsolete package profiles with current production consumers and approved feature configurations.
5. Rebuild ownership checks around production source and approved functional modules. Keep real production findings visible.
6. Inspect all 13 historical overlay findings. Separate test-only helpers from the two production disk functions.
7. Give each production overlay site its shared-layer implementation and functional case. Do not add a broad exemption.
8. Replace the deleted workflow assumption with checks against maintained generator inputs and generated CI plans.
9. Add reviewed gate cases for a missed production site, an invalid feature profile, and a hidden command failure.
10. Assign each remaining implementation issue to T11, T19, T27, T28, T29, or another exact owner task.

### Required cases

- Reviewed functional probes detect a production overlay and ignore unowned external test data.
- The gate rejects an unknown package, an empty required case set, and a hidden command failure.
- Gate execution reaches its runtime assertions instead of failing during setup.
- Historical ownership and visual findings remain visible until reviewed functional evidence closes them.

### Commands

~~~sh
cargo metadata --locked --format-version 1 --no-deps
cargo run --locked -p termrock-xtask -- check-ownership --format json
cargo nextest run --locked --workspace --all-features
~~~

### Acceptance

Gate inputs describe the current workspace. Every original failure has an exact disposition. This task does not require unrelated visual fixes to finish.

<a id="t03"></a>

## T03: Remove backend leakage from pure library configurations

**Implementer:** GPT-6 Luna, max effort.
**Independent reviewer:** GPT-6.1-Sol, high effort.
**Predecessors:** T01, T38.
**Source findings:** [F03](REVIEW.md#f03), [F04](REVIEW.md#f04).

### Owned files

- Cargo.toml workspace dependency declarations
- crates/termrock-core/Cargo.toml
- crates/termrock-runtime/Cargo.toml
- crates/termrock-runtime/src/runtime.rs
- crates/termrock-runtime/src/runtime/time.rs
- production library manifests for backend edges

### Child work

- Split pure time support, dependency pruning, and detached probe construction by file.
- Fix the normal dependency graph before feature probes run.
- Use a separate Sol high agent to inspect the resolved graph.

### Required changes

1. Compile runtime/time.rs without the crossterm feature. Keep ClockError and Moment available in every runtime build.
2. Find each production use of ratatui and ratatui-crossterm. Remove unused normal dependencies.
3. Use ratatui-core for shared geometry, style, frame, and buffer values where it already provides the required type.
4. Keep TestBackend dependencies under dev-dependencies when production code does not use them.
5. Disable backend defaults on internal library edges. Let the session adapter request backend conversion explicitly.
6. Keep framework key and modifier types public. Preserve Press and Repeat conversion and Release rejection.
7. Create reviewed detached leaf scenarios under integration/functional/. Give each probe one direct production dependency.
8. Run separate no-default, default, each-feature, and all-supported-feature builds. Save each resolved dependency graph.

### Required cases

- A no-default termrock-runtime consumer compiles and can construct Moment.
- A controls-only consumer has no session, editor, grid, app, or test-support dependency.
- The normal library graph has no terminal backend unless the selected feature requests it.
- Backend conversion preserves key and modifier identity. Paste Debug output stays redacted.
- Platform-neutral components compile for Windows without Unix session source.

### Commands

~~~sh
cargo check --locked -p termrock-runtime --lib --no-default-features
cargo check --locked -p termrock-core --lib --no-default-features
cargo nextest run --locked --workspace --all-features
cargo check --locked --workspace --all-targets
~~~

### Acceptance

Every declared leaf configuration passes in an isolated consumer. Dependency graphs prove the selected backend boundary.

<a id="t04"></a>

## T04: Prepare a source-bound visual reference candidate

**Implementer:** GPT-6 Luna, max effort.
**Independent reviewer:** GPT-6.1-Sol, high effort.
**Predecessors:** T00, T01, T38.
**Source findings:** [F14](REVIEW.md#f14).

### Owned files

- A production capture tool only if T00 proves an active production use
- Approved functional capture code under integration/functional/
- new reference-candidate manifests under the declared evidence root
- baselines/oracle/ORACLE.toml only after the required approval

### Child work

- Split case inventory, real CLI capture, and semantic assertion preparation.
- Run captures alone on the selected capture host.
- Require task-scoped Sol medium review before every new harness or assertion edit.

### Required changes

1. Preserve the immutable annotated visual-baseline tag, its snapshot bytes, and the tuiscotti pin.
2. Keep a legacy capture runner only if T00 proves a production use. Remove its old test harness under T37.
3. Create a separate capture candidate from the fixed pull request 26 source. Record source, lockfile, binary, and environment hashes.
4. Derive the required case set from T00's archived inventory and COMPONENT-MATRIX.md. Do not execute deleted harness targets.
5. Include all 54 route identities and the four applications' required journeys. Specify semantic states through public application entry points.
6. Run the reference twice with fixed input, size, color, seed, and clock. Reject nondeterministic frames or checkpoint order.
7. Record each output cell, attribute, cursor field, semantic state, and case identity. Keep screenshots as review views.
8. Classify defects and missing reference states with a named product rule. Do not silently approve a defective historical state.
9. Prepare the complete candidate manifest and visual review before any required approval request.
10. Promote only an approved candidate through the declared oracle process. Keep old evidence immutable and preserve its identity.

### Required cases

- An absent, duplicate, skipped, reordered, or uncaptured required checkpoint fails the candidate check.
- A candidate application cannot modify the reference tree without detection.
- An inspector-open case proves visible inspector controls and an active dismissal route.
- Static state specimens have no live input regions or focus registrations.
- The report separates current reference, historical baseline, and candidate application output.

### Commands

Use the exact production CLI invocations recorded by T00.
Do not run a legacy capture tool or test harness.

~~~sh
git diff --check
~~~

### Acceptance

A complete reference candidate and its review exist. Required promotion must be authorized and complete. Otherwise, keep T04 BLOCKED.

<a id="t05"></a>

## T05: Make the facade feature set and API paths explicit

**Implementer:** GPT-6 Luna, max effort.
**Independent reviewer:** GPT-6.1-Sol, high effort.
**Predecessors:** T03.
**Source findings:** [F04](REVIEW.md#f04), [F10](REVIEW.md#f10).

### Owned files

- crates/termrock/Cargo.toml
- crates/termrock/src/lib.rs
- crates/termrock/src/composition.rs feature gates
- crates/termrock/src/composition/adapters.rs feature gates
- facade API compile tests
- first-party Cargo.toml files for facade feature declarations only

### Child work

- Keep facade manifests and exports with one Luna owner.
- Use separate Luna workers for family import probes and application feature declarations.
- Integrate only after the feature names are fixed.

### Required changes

1. Implement the exact feature map in ARCHITECTURE.md. Use default=[components]. Keep session behind its explicit feature.
2. Gate each family export and adapter with the feature that provides its type.
3. Keep foundations available without family features. Do not enable testing through normal defaults.
4. Add complete family namespaces. Include omitted public options such as ButtonFocusVisibility, CheckboxState, and PropsPresentation.
5. Keep advanced author functions under termrock::author. Preserve old root aliases during the recorded transition when ownership permits them.
6. Update first-party dependency declarations to request their actual families and session adapter.
7. Record this source and feature migration explicitly. If a published release requires a version change, apply the selected version policy.
8. Test the documented minimal, common, custom-author, and session dependency examples.

### Required cases

- No-default facade builds. Each family feature builds alone with its required transitive families.
- The default graph has all documented reusable families and no terminal session backend.
- A selected controls consumer does not load grid, editors, forms, terminal view, or session.
- Every documented option and state type has an import path in its owning namespace.
- Old compatible import paths compile during the declared transition.

### Commands

~~~sh
cargo check --locked -p termrock --lib --no-default-features
cargo check --locked -p termrock --lib --no-default-features --features controls
cargo check --locked -p termrock --lib
cargo nextest run --locked -p termrock --all-features
~~~

### Acceptance

The facade feature matrix matches the docs. All selected consumer graphs and import paths pass.

<a id="t06"></a>

## T06: Enforce crate direction and the lint policy

**Implementer:** GPT-6 Luna, max effort.
**Independent reviewer:** GPT-6.1-Sol, high effort.
**Predecessors:** T05.
**Source findings:** [F02](REVIEW.md#f02), [F05](REVIEW.md#f05), [F06](REVIEW.md#f06).

### Owned files

- tools/acceptance_policy.json
- tools/acceptance_architecture.py
- tools/test_acceptance_architecture.py
- production Cargo.toml lint sections
- scoped production lint corrections

### Child work

- Assign architecture policy and mutation tests to separate bounded files.
- Split source lint corrections by crate.
- Keep one Luna owner for shared root and feature declarations.

### Required changes

1. Encode the allowed crate edges from ARCHITECTURE.md in the existing architecture policy.
2. Reject library-to-application, library-to-test-tool, lower-crate-to-facade, and cyclic edges.
3. Record normal, build, dev, and feature-activated edges separately.
4. Add workspace lint inheritance to all 20 production library crates.
5. Assign each newly exposed source warning to a Luna child task with bounded file ownership.
6. Correct the source cause. Keep only operation-specific or test-specific exceptions with a concrete reason.
7. Review session FFI exceptions against its safety invariants. Do not add a crate-wide allow for convenience.
8. Add mutation tests for a forbidden dependency, a cycle, a removed lint inheritance line, and a hidden generated input.
9. Keep each initial ownership failure visible until its named source task fixes it.
10. Record the exact failing edge or export, source commit, and owner task. Include T07, T08, and T20 where applicable.

### Required cases

- Each forbidden mutation fails the gate with the source path and dependency kind.
- Every production library inherits the declared lints.
- The gate distinguishes an exported alias from the file that implements the type.
- Clippy runs with the intended policy on each corrected crate.
- An intentional dev dependency does not create a false normal-dependency claim.

### Commands

~~~sh
PYTHONDONTWRITEBYTECODE=1 python3 tools/test_acceptance_architecture.py
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo run --locked -p termrock-xtask -- check-ownership --format json
~~~

### Acceptance

The policy catches forbidden changes. Lint inheritance is active. Record each existing ownership failure until its named source task removes it.

<a id="t07"></a>

## T07: Move Jackin screen code out of feedback

**Implementer:** GPT-6 Luna, max effort.
**Independent reviewer:** GPT-6.1-Sol, high effort.
**Predecessors:** T06, T10, T11.
**Source findings:** [F05](REVIEW.md#f05), [F15](REVIEW.md#f15).

### Owned files

- crates/termrock-feedback/src/ product-specific modules
- crates/jackin-preview-host-ui/src/
- crates/jackin-preview-capsule-ui/src/
- crates/jackin-preview-presentation/src/
- affected facade export migration

### Child work

- Split host UI moves, Capsule UI moves, and export migration records.
- Keep shared presentation types with one owner.
- Review source moves before changes to screen behavior.

### Required changes

1. Move account, settings, usage, manager, and inspector page models to jackin-preview-host-ui.
2. Move capsule menu, session chrome, and zoom page models to jackin-preview-capsule-ui.
3. Use jackin-preview-presentation only for projections shared by both UI owners.
4. Replace repeated page-specific menu, field, meter, clipping, and scrollbar code with existing shared components.
5. Keep application labels, account fields, mount policy, launch policy, and domain actions in Jackin.
6. Retain generic Notice, StatusBar, HintBar, ProgressBar, Spinner, Meter, and TextSection in feedback.
7. For each removed public product export, add a migration row and update all first-party consumers.
8. Keep only forward-only compatibility adapters that use generic primitives. Do not reverse-depend on Jackin.
9. Compare each moved screen with its approved reference before and after the move.

### Required cases

- Feedback source contains no account, mount, workspace, provider-quota, or capsule page model.
- The normal feedback graph has no application dependency.
- Two independent consumers can use each retained generic component.
- Moved screens retain actions, focus, overlays, scroll behavior, and exact approved cells.
- No public path disappears without a recorded migration and consumer update.

### Commands

~~~sh
cargo nextest run --locked -p termrock-feedback --all-features
cargo nextest run --locked -p jackin-preview-host-ui -p jackin-preview-capsule-ui -p jackin-preview-app --all-features
cargo run --locked -p termrock-xtask -- check-ownership --format json
~~~

### Acceptance

Product screen ownership is in Jackin. Shared feedback contains reusable behavior and passes both consumer and visual checks.

<a id="t08"></a>

## T08: Separate application domain state from UI state

**Implementer:** GPT-6 Luna, max effort.
**Independent reviewer:** GPT-6.1-Sol, high effort.
**Predecessors:** T06.
**Source findings:** [F06](REVIEW.md#f06).

### Owned files

- crates/jackin-preview-domain/
- crates/jackin-preview-sim/
- crates/jackin-preview-presentation/
- crates/tablepro-domain/
- crates/tablepro-ui/src/ domain adapters
- crates/showcase-data/Cargo.toml

### Child work

- Split Jackin domain work, TablePro domain work, and Holla/Showcase boundary checks.
- Fix each domain mapping before its UI consumer migration.
- Use a separate reviewer for domain semantics.

### Required changes

1. Replace Jackin domain's full facade dependency with the smallest owner of its required neutral text-wipe function.
2. Give Jackin simulation its own launch and task state. Map that state to StepState and presentation roles in the UI.
3. Remove geometry and widget state from pure simulation data unless a documented domain requirement needs the value.
4. Move CompletingCodeEditorState, GridState, form specifications, and UI adapters from TablePro domain to TablePro UI.
5. Keep connection data, result identity, schema, validation, and SQL ordering in domain or SQL crates.
6. Check Holla and Showcase data boundaries with the same rule. Correct full-facade edges that serve one neutral value.
7. Preserve stable IDs, deterministic simulation, secret policy, and transactional validation during each move.
8. Keep moves and behavior changes in separate commits.

### Required cases

- Domain package dependency graphs contain no session, widget family, or facade dependency.
- Domain tests run without terminal setup or UI state.
- UI adapters preserve row IDs, draft values, and pending edits across data replacement.
- Secret wiping and redaction remain effective through approved public behavior scenarios.
- No live database, host cleanup, process launch, or external network effect enters a demonstration.

### Commands

~~~sh
cargo nextest run --locked -p jackin-preview-domain -p jackin-preview-sim -p tablepro-domain -p tablepro-sql
cargo nextest run --locked -p holla-domain -p holla-plan -p holla-sim
cargo check --locked --workspace --all-targets
~~~

### Acceptance

Domain state is independent of UI families. All mapped UI behavior retains its prior contract.

<a id="t09"></a>

## T09: Preserve response effects and the frame lifecycle

**Implementer:** GPT-6 Luna, max effort.
**Independent reviewer:** GPT-6.1-Sol, high effort.
**Predecessors:** T03, T06.
**Source findings:** [F07](REVIEW.md#f07), [F08](REVIEW.md#f08), [F11](REVIEW.md#f11), [F16](REVIEW.md#f16).

### Owned files

- crates/termrock-core/src/response.rs
- crates/termrock-runtime/src/runtime.rs
- crates/termrock-runtime/src/runtime/time.rs
- crates/termrock-runtime/src/ui/cx.rs
- focused runtime tests
- external-consumer-demo/ lifecycle probe only
- crates/termrock-core/src/keys.rs pure KeyboardDelivery value only
- Runtime, Cx, and Ui keyboard delivery snapshot accessors

### Child work

- Split Response tests, frame lifecycle tests, and the external-host probe.
- Keep each shared source file with one Luna owner.
- Review effects and frame acknowledgement with separate Sol high scopes.
- Assign the pure KeyboardDelivery value and runtime snapshot to one bounded Luna child task.

### Required changes

1. Keep the existing Response fields and action-mapping methods.
2. Add merge_effects for a typed parent response only as specified in API-RULES.md.
3. Merge flow and invalidation with their maximum values. Preserve the parent ID, state, and action.
4. Use erase when a parent intentionally drops an action. Do not rebuild Response fields manually.
5. Keep update as the semantic mutation phase. Permit only paint, registration, diagnostics, and derived caches during draw.
6. Test prepare, projection, transport, and acknowledgement as separate steps.
7. Reject stale runtime identity, generation, area, theme, and app-state receipts before input can use their geometry.
8. Preserve the earliest repaint deadline through unrelated input and redraw. Reject backward time.
9. Keep reference rendering inert across the complete subtree.
10. Provide an external-host example that drives the production lifecycle without the testing feature.
11. Add the exact KeyboardDelivery value and runtime accessors from API-RULES section 7.
12. Store one delivery snapshot per Runtime. A changed snapshot invalidates prepared output, affected caches, and layout.
13. Keep protocol lifetime and input observations outside this snapshot.

### Required cases

- All nine invalidation pairs and all flow pairs preserve the maximum effect.
- Action erasure preserves ID, state, and effects. Effect merge keeps the parent action.
- A failed or incomplete transport does not acknowledge the frame.
- Resize or app mutation rejects the previous prepared token and requires preparation again.
- An early input does not postpone an existing animation deadline.
- A reference specimen registers no input, focus, layer, binding, or cursor side effect.
- Opposite snapshots in two runtimes remain independent.
- An equal setter preserves a prepared token. A changed setter rejects it and refreshes hint width.

### Commands

~~~sh
cargo nextest run --locked -p termrock-core --all-features
cargo nextest run --locked -p termrock-runtime --all-features
cargo nextest run --locked --manifest-path external-consumer-demo/Cargo.toml
~~~

### Acceptance

Response composition cannot lose Layout. Only acknowledged current geometry can route input.

<a id="t10"></a>

## T10: Complete the existing composition and author API

**Implementer:** GPT-6 Luna, max effort.
**Independent reviewer:** GPT-6.1-Sol, high effort.
**Predecessors:** T05, T09.
**Source findings:** [F07](REVIEW.md#f07), [F08](REVIEW.md#f08).

### Owned files

- crates/termrock-runtime/src/author/composition.rs
- crates/termrock-runtime/src/ui/composition.rs
- crates/termrock/src/composition.rs
- crates/termrock/src/composition/adapters.rs
- crates/termrock/src/blocks/
- crates/termrock-shell/src/shell.rs
- external-consumer-demo/ composition adapter use only
- crates/termrock-consumer-example/src/app.rs composition adapter use only

### Child work

- Keep nested validation and depth rules with one Luna owner.
- Assign facade adapter use and external-consumer cleanup to another worker.
- Use a third worker for depth and inert-failure cases after signatures are fixed.

### Required changes

1. Add the validation hook specified in API-RULES.md to the existing CompositionLeaf trait.
2. Make Child, Group, Panel, MasterDetail, ShellSlots, and owned slot adapters report their nested depth.
3. Validate the complete built-in tree before measure, paint, or registration.
4. Remove silent error replacement from the supported composition path. Return the exact CompositionError.
5. Keep direct infallible draw inert and emit a diagnostic for invalid content. Checked composition entry points must validate before infallible Measure.
6. Use one closure-leaf implementation. Preserve compatible facade constructors through a re-export or small adapter.
7. Use existing menu_bar and status_bar adapters in the public consumer. Remove its duplicate MenuLeaf and StatusLeaf.
8. Keep measure free of input handling and domain effects.
9. Keep Panel and Dialog callbacks exactly once under the declared contract. Keep SplitPane callbacks in stable logical order.
10. Preserve custom author access to Rect, Buffer-related values, clipping, parts, and registration.

### Required cases

- Depth 128 succeeds and depth 129 fails through every built-in wrapper path.
- Failure produces no partial paint, hit registration, focus registration, or layer state.
- Direct Child-to-CompositionLeaf conversion cannot hide an invalid tree.
- An invalid deep tree can be dropped safely.
- Empty and collapsed slots obey the documented callback count.
- A custom external leaf works through the same measure, clipping, and registration path.

### Commands

~~~sh
cargo nextest run --locked -p termrock-runtime --all-features
cargo nextest run --locked -p termrock --all-features
cargo nextest run --locked -p termrock-shell --all-features
cargo nextest run --locked --manifest-path external-consumer-demo/Cargo.toml
~~~

### Acceptance

Composition has one implementation and one visible error contract. The ordinary consumer needs no duplicate adapters.

<a id="t11"></a>

## T11: Put all shared appearance rules in the theme owner

**Implementer:** GPT-6 Luna, max effort.
**Independent reviewer:** GPT-6.1-Sol, high effort.
**Predecessors:** T06, T09.
**Source findings:** [F01](REVIEW.md#f01), [F08](REVIEW.md#f08), [F15](REVIEW.md#f15).

### Owned files

- crates/termrock-theme/
- crates/termrock-render/src/lib.rs
- crates/termrock-runtime/src/author.rs patch support
- crates/showcase-demos/src/pages/mod.rs theme workarounds
- affected component style tests

### Child work

- Keep theme roles and patch precedence with one owner.
- Split component adoption by family after the shared rules pass.
- Give a separate reviewer the reference images and custom-theme cases.

### Required changes

1. Give pure patch merging one implementation in termrock-render. Let runtime add slot callbacks and style accounting.
2. Preserve the documented precedence for theme rules, local scope, instance patch, part patch, and slot override.
3. Keep Inherit, Set, and Clear distinct. A later scope must not change a sibling.
4. Add semantic roles for currently unnamed seam and selection treatments. Preserve the approved palette at each color level.
5. Remove exact-RGB comparisons and app-local tint repair after the shared role supplies the required value.
6. Remove indexed-color conversion to an RGB-like integer from shared demonstration helpers.
7. Keep active focus, chosen selection, covered selection, disabled, read-only, hover, and pressed treatments distinct.
8. Use full, reduced, and paused motion through deterministic time. Do not make style selection depend on wall-clock reads.
9. Keep user-defined themes and custom palettes functional. Do not hard-code Junie assumptions in generic components.
10. Correct each application mismatch whose cause is a shared theme rule. Record remaining mismatches under T22 or T27-T29.

### Required cases

- Each layer in the override precedence has a direct assertion and a sibling-isolation assertion.
- Slot Clear removes the inherited slot. Slot Inherit retains it. Slot Set uses the caller's slot.
- TrueColor, 256, 16, and monochrome keep visible focus and selection.
- Disabled controls ignore hover. Covered controls cannot display active-layer focus.
- Seam and scrollbar changes affect only their reserved cells.
- Custom-theme color equality does not trigger an application-specific workaround.

### Commands

~~~sh
cargo nextest run --locked -p termrock-theme -p termrock-render --all-features
cargo nextest run --locked -p termrock-conformance --all-features
cargo nextest run --locked -p showcase-ui -p holla-ui -p tablepro-ui -p jackin-preview-app --all-features
~~~

### Acceptance

Theme-owner and custom-theme cases pass. Record each remaining application mismatch under its application task. Do not claim those checks passed.

<a id="t12"></a>

## T12: Complete controls and generic feedback

**Implementer:** GPT-6 Luna, max effort.
**Independent reviewer:** GPT-6.1-Sol, high effort.
**Predecessors:** T10, T11.
**Source findings:** [F10](REVIEW.md#f10).

### Owned files

- crates/termrock-controls/
- crates/termrock-runtime/ borrowed tooltip request, lifecycle, and presentation support
- control examples
- crates/termrock-feedback/src/progress.rs
- crates/termrock-feedback/src/meter.rs
- crates/termrock-feedback/src/status.rs
- crates/termrock-feedback/src/hintbar.rs
- crates/termrock-feedback/src/keyhint.rs
- crates/termrock-feedback/src/notice.rs
- crates/termrock-feedback/src/section.rs
- generic feedback tests; product modules and feedback/lib.rs export moves remain with T07

### Child work

- Split Button, other controls, and runtime tooltip presentation by source owner.
- Fix the neutral tooltip contract before control and presenter work diverge.
- Assign ProgressBar, Spinner, and Meter to one bounded Luna child task.
- Assign StatusBar, HintBar, KeyHint, and DerivedHintBar to another Luna child task.
- Assign Notice and TextSection to a separate Luna child task.
- Keep product moves and feedback/lib.rs export changes with T07.
- Assign conformance cases after each public signature is fixed.

### Required changes

1. Complete W01-W05, W32-W43, RadioField, control MasterDetail, Notice, TextSection, and DerivedHintBar against COMPONENT-MATRIX.md.
2. Restore Button mnemonic display through a named builder. Keep mnemonic display separate from activation binding.
3. Add borrowed tooltip configuration through a neutral runtime-owned request type.
4. Use the existing tooltip layer kind and layer sizing path. Keep the controls-to-runtime dependency direction.
5. Resolve shortcut text from the effective binding after user remapping. Do not store an unrelated shortcut string.
6. Implement tooltip delay, cancellation, clipping, and active-layer ownership with deterministic time.
7. Preserve Checkbox mixed state, caller-controlled RadioGroup values, disabled pointer absorption, and click-only focus policy.
8. Keep Panel, SplitPane, Props, Empty, and TooSmall behavior under their explicit owners.
9. Expose each real configurable part through the existing part and slot contracts.
10. Filter effective tooltip chords through Ui::keyboard_delivery(). Do not consult core or shell globals.
11. Complete ProgressBar, Spinner, and Meter frame, state, label, threshold, and clipping contracts. Keep timers with their existing providers.
12. Complete StatusBar, HintBar, KeyHint, and DerivedHintBar layout and action contracts. Test caller-supplied hint metadata.
13. Complete Notice and TextSection measurement, tone, clipping, and supported reveal contracts.
14. Keep product-specific feedback modules outside this task. Return shared export changes through the T07 file owner.

### Required cases

- A Button with Tooltip::action shows and activates the same effective remapped chord.
- A disabled or inactive Button has no activation or tooltip side effect.
- Two Buttons with the same label have separate IDs, hover state, and tooltip ownership.
- A tooltip closes on owner removal, modal cover, or the configured dismissal event.
- Empty, 1×1, narrow, CJK, combining-mark, and emoji labels do not panic or draw outside the clip.
- Controlled checkbox/radio state changes only when the caller accepts the action.
- An unfocused hovered Button with Tooltip::action shows its effective remapped chord. It does not publish or enable a command.
- PropsList and control MasterDetail preserve navigation, selection, focus, and independent instance state.
- ProgressBar handles zero, full, out-of-range, indeterminate, done, and status values under its existing policy.
- Spinner uses the caller's frame. Empty and custom frame sets preserve Unicode clipping. Do not add component timers.
- Meter tests cover threshold boundaries, Unknown, Stale, Series, Line, Block, and value/suffix/activity layout.
- StatusBar preserves left, center, and right content, priority elision, width limits, and keyed StatusAction::Chose hit regions.
- HintBar preserves badge, status, overflow, and metadata Inherit versus Set(None).
- DerivedHintBar preserves top, mode, focus, screen, then global precedence. T19 proves final shell capability integration.
- Notice preserves cut records, vertical windows, and tones. TextSection preserves row, inset, patch, and clipping measurements.
- Motion policy and status expiry remain with their existing runtime or application providers.

### Commands

~~~sh
cargo nextest run --locked -p termrock-controls --all-features
cargo nextest run --locked -p termrock-runtime -p termrock-overlays --all-features
cargo nextest run --locked -p termrock-conformance --all-features
cargo nextest run --locked -p termrock-feedback --all-features
~~~

### Acceptance

Controls and generic feedback pass their matrix cases. Button restores reference behavior through shared runtime ownership.

<a id="t13"></a>

## T13: Complete fields, validation, and clipboard outcomes

**Implementer:** GPT-6 Luna, max effort.
**Independent reviewer:** GPT-6.1-Sol, high effort.
**Predecessors:** T09, T11, T12.
**Source findings:** [F16](REVIEW.md#f16).

### Owned files

- crates/termrock-fields/
- crates/termrock-text/ edit and secret paths
- crates/termrock-runtime/src/runtime/clipboard.rs
- field and clipboard consumer tests

### Child work

- Split text editing, field state, and clipboard transport results.
- Keep shared SecretPolicy changes with one owner.
- Use a separate reviewer for plaintext exposure and failed-commit cases.

### Required changes

1. Complete TextInput, TextArea, Field, Note, FieldErrorChrome, and shared editing behavior.
2. Preserve separate draft and committed values. Validate before committing a value or leaving a protected edit.
3. Keep disabled and read-only behavior distinct. Permit navigation and permitted copy in read-only fields.
4. Use grapheme boundaries for movement, selection, deletion, copy, and paste.
5. Add a result-bearing copy adapter with internal-only, external-success, unsupported, refused, too-large, and transport-error outcomes.
6. Keep the existing copy hook as a documented compatibility path with an unknown external result.
7. Apply SecretPolicy before any copy adapter call. Keep redacted Debug and wipe-on-replacement behavior.
8. Pass clipboard outcomes to shared status behavior. Do not display external success after an unsupported or failed transfer.
9. Keep validation and copy callbacks able to borrow or capture caller context through the existing lifetime model.

### Required cases

- Invalid commit preserves the previous committed value and keeps the draft for correction.
- Cancel restores the declared prior value. Failed save does not close the field or route.
- Test combining marks, CJK, ZWJ emoji, regional flags, empty paste, multiline paste, and large paste.
- Secret values do not enter copy transport, Debug, diagnostics, or captured output unless the explicit policy permits it.
- Two runtimes have separate clipboard state.
- Each copy outcome produces the correct action and status without a real system clipboard write.

### Commands

~~~sh
cargo nextest run --locked -p termrock-text -p termrock-fields --all-features
cargo nextest run --locked -p termrock-runtime --all-features
cargo nextest run --locked --manifest-path external-consumer-demo/Cargo.toml
~~~

### Acceptance

Field state transitions are explicit and atomic. Clipboard results describe the actual operation.

<a id="t14"></a>

## T14: Make collection revisions and navigation identity explicit

**Implementer:** GPT-6 Luna, max effort.
**Independent reviewer:** GPT-6.1-Sol, high effort.
**Predecessors:** T09, T11, T13.
**Source findings:** [F09](REVIEW.md#f09).

### Owned files

- crates/termrock-collections/
- crates/termrock-navigation/
- collection and navigation performance tests
- crates/termrock-test-support/tests/perf_collections.rs navigation cases only

### Child work

- Keep CollectionCore revision handling with one Luna owner.
- Split List/NavList/ChipBar adoption and Tree/Steps checks by file.
- Add performance assertions after the revision contract is fixed.

### Required changes

1. Reuse the existing termrock_core::Revision type. Do not add another revision type.
2. Add source_revision(Revision) -> Self to List, NavList, and ChipBar builders.
3. Pass that revision to one revision-aware CollectionCore reconciliation path. Keep current update signatures and explicit invalidate.
4. Advance the source revision for changes to ordered keys or eligibility. Compare row extent inputs separately.
5. Keep the existing Tree query revision and GridModel revision contracts. Do not replace them with the list builder API.
6. Keep cursor, chosen item, checked set, anchor, and focus as distinct state.
7. Complete List, FilterList, NavList, Tree, Steps, Tabs, ChipBar, and row adapters.
8. Keep selection by stable key through sort, filter, insertion, deletion, and reorder.
9. Make disabled rows unreachable for activation and Select All. Keep read-only or skipped rows inspectable where the contract permits it.
10. Keep Tree query projection separate from persistent expansion. Invalidate matcher-dependent caches when matcher meaning changes.
11. Keep Steps frontier updates bounded and deterministic. Preserve metadata, reveal, and stable action identity.
12. Use the existing row/part override path. Do not duplicate list or tree input logic in applications.

### Required cases

- Reorder [a,b,c,d] to [a,c,b,d] through the supported update path and preserve selected keys.
- Replace a middle item at constant length and preserve valid cursor fallback.
- Change enabled state and row extent without changing endpoint keys.
- Shift extension takes priority over selection-toggle modifiers.
- Right-click on a selected row preserves the selected set. Right-click on another row selects that row.
- The warm 1,000/100,000-item picker test retains 38 reads for 19 visible rows and zero warm allocations.
- Tree and Steps work counters remain within their existing declared bounds.
- A saturated Revision disables the shortcut. Switching between revision and legacy paths cannot reuse a stale cache stamp.
- Test explicit invalidation, revisioned-to-legacy transitions, and saturated Revision(u64::MAX).
- Assert semantic key, cached index, checked-key removal, and extent clamp after constant-endpoint changes.

### Commands

~~~sh
cargo nextest run --locked -p termrock-collections -p termrock-navigation --all-features
PERF_FULL=1 cargo nextest run --locked --release -p termrock-test-support --test perf_collections
~~~

### Acceptance

Data changes have an explicit contract. Identity and bounded work hold for every supported update path.

<a id="t15"></a>

## T15: Complete overlay behavior and shared layer ownership

**Implementer:** GPT-6 Luna, max effort.
**Independent reviewer:** GPT-6.1-Sol, high effort.
**Predecessors:** T10, T11, T13, T14.
**Source findings:** [F11](REVIEW.md#f11), [F14](REVIEW.md#f14).

### Owned files

- crates/termrock-overlays/
- crates/termrock-runtime/src/layer.rs for agreed layer fixes
- overlay conformance and consumer tests

### Child work

- Split picker families, dialog lifecycle, and menu families by file.
- Keep runtime layer changes with one owner.
- Review modal routing and request identity independently.

### Required changes

1. Complete Select, SelectField, Picker, CommandPalette, PickerChain, Completion, Dialog, Menu, ContextMenu, MenuBar, HelpOverlay, and NoticeReveal.
2. Use runtime-owned layer opening IDs, resolved areas, hit regions, and focus barriers.
3. Make outside-click dismissal produce the owner's configured cancel action.
4. Keep nested layer completion tied to the correct opening generation. Reject stale completion after close and reopen.
5. Wrap Tab and BackTab inside a modal's reachable controls. Restore focus only to a valid prior owner in its scope.
6. Keep active query focus separate from popup row selection. Avoid a second live focus marker.
7. Apply borrowed visible-row data and the shared data-revision contract to picker families.
8. Expose active-layer hint metadata through the agreed runtime contract. Return shell integration to T19 and route-whitelist removal to T22.
9. Keep callback invocation and validation failure behavior explicit for dialog bodies.

### Required cases

- Nested modal, popup, and context menu receive input before the background.
- Outside click, Esc, Enter, and owner removal produce exactly one declared result.
- A stale result from the prior opening cannot close or modify a new opening.
- Disabled pointer targets absorb the event without activating background content.
- Resize, empty results, long labels, and narrow screens preserve reachable dismissal and action controls.
- Menu hover, click, keyboard navigation, and remapped hints agree.
- SelectField preserves field identity through popup commit or cancel. NoticeReveal rejects stale text and window state.

### Commands

~~~sh
cargo nextest run --locked -p termrock-overlays --all-features
cargo nextest run --locked -p termrock-runtime --all-features
cargo nextest run --locked -p termrock-conformance --all-features
~~~

### Acceptance

All floating content uses one layer path. Focus, dismissal, and completion identity pass the matrix.

<a id="t16"></a>

## T16: Complete Grid and Table as model-driven components

**Implementer:** GPT-6 Luna, max effort.
**Independent reviewer:** GPT-6.1-Sol, high effort.
**Predecessors:** T11, T13, T14, T15.
**Source findings:** [F20](REVIEW.md#f20).

### Owned files

- crates/termrock-grid/
- grid external-consumer tests
- grid conformance and performance tests

### Child work

- Split schema/edit behavior, row identity, and column/viewport behavior.
- Keep the Grid model trait and action types with one owner.
- Use a separate worker for hostile-model and ragged-row tests.

### Required changes

1. Keep one column schema as the source of column count and identity.
2. Use borrowed cell data and stable row/column keys. Keep model ordering in the caller.
3. Emit keyed sort and edit requests. Do not move SQL or application validation into Grid.
4. Preserve cursor, selection, checked rows, anchor, and pending edit identity across sort and filter.
5. Treat absent cells in ragged rows as structural holes. Do not invoke editor, decoration, or action hooks for them.
6. Complete Table through the shared Grid and row mechanisms where their contracts match.
7. Preserve per-column alignment, sizing, sticky columns, horizontal scroll, overflow hints, and custom cell presentation.
8. Use explicit typed errors for invalid cursor, schema, or edit requests.
9. Keep width sampling bounded by the declared sample policy.
10. Complete GridViewport bar interdependence, seam geometry, and focus-reveal behavior through its existing owner.

### Required cases

- Ragged, empty, duplicate-key, removed-row, reordered-column, and read-only data obey the documented result.
- A model that panics on an invalid cell-hook call never receives that call.
- Sort, filter, and projection changes preserve surviving semantic identities.
- A 100,000-row grid reads only the visible or explicitly sampled range.
- Horizontal and vertical scrollbars keep valid geometry after resize.
- TablePro's NULLS-last behavior remains in the domain owner and has an approved functional case.

### Commands

~~~sh
cargo nextest run --locked -p termrock-grid --all-features
cargo nextest run --locked -p termrock-conformance --all-features
PERF_FULL=1 cargo nextest run --locked --release -p termrock-test-support --test perf_collections
~~~

### Acceptance

Grid and Table expose reusable model contracts without application ordering or editor-state leakage.

<a id="t17"></a>

## T17: Complete text viewports and editor extension paths

**Implementer:** GPT-6 Luna, max effort.
**Independent reviewer:** GPT-6.1-Sol, high effort.
**Predecessors:** T09, T11, T13, T15.
**Source findings:** [F08](REVIEW.md#f08).

### Owned files

- crates/termrock-viewport/
- crates/termrock-editors/
- crates/termrock-terminal/src/terminal_view.rs
- editor and viewport consumer tests
- crates/termrock-runtime/src/scroll_region.rs
- ScrollRegion runtime tests for W45

### Child work

- Split viewport/diff, editors/completion, and TerminalView by crate.
- Keep shared text contracts with T13's accepted source.
- Use a separate reviewer for invalidation and source ownership.

### Required changes

1. Complete CodeEditor, CompletingCodeEditor, DiffView, TextViewport, TerminalView, and ScrollRegion against COMPONENT-MATRIX.md.
2. Keep caller-owned editor state and borrowed highlighter, segmenter, and row data where supported.
3. Replace manual completing-editor response selection with the shared effect-merge contract.
4. Bind completion data to current text and owner. Apply request identity only where an existing request adapter needs it.
5. Invalidate wrapped-line and projection caches on text, width, style-metric, or revision changes that affect output.
6. Keep append paths incremental when the existing source contract permits it.
7. Preserve selection, cursor, reveal, copy, undo, redo, and read-only policies across scroll and resize.
8. Keep TerminalView's content and input model separate from session transport and protocol negotiation.
9. Do not add a background executor to a synchronous component.
10. Complete ScrollRegion nested-wheel, partial-delta, reveal, drag, clipping, and owner-identity contracts.

### Required cases

- Unicode cursor and selection remain valid after edit, wrap, scroll, and resize.
- Where a request adapter exists, completion result A cannot replace newer result B.
- Completion layout invalidation survives an editor paint-only response.
- Diff selection and copy preserve source identity in every supported mode.
- Viewport append and width invalidation pass the existing work-counter cases.
- TerminalView respects Interactive, ReadOnly, and SelectionOnly behavior.
- Monochrome selection remains visible without changing text geometry.
- A nested ScrollRegion consumes only the applied wheel delta. Its parent receives the remaining delta.
- Horizontal retry, thumb drag, reveal, and resize preserve valid clip and hit geometry.

### Commands

~~~sh
cargo nextest run --locked -p termrock-viewport -p termrock-editors -p termrock-terminal --all-features
PERF_FULL=1 cargo nextest run --locked --release -p termrock-test-support --test perf_collections
~~~

### Acceptance

Text, editor, diff, and terminal view APIs preserve state and permit real custom presentation without copied behavior.

<a id="t18"></a>

## T18: Complete Form, FormDialog, and Wizard

**Implementer:** GPT-6 Luna, max effort.
**Independent reviewer:** GPT-6.1-Sol, high effort.
**Predecessors:** T10, T12, T13, T14, T15.
**Source findings:** [F08](REVIEW.md#f08).

### Owned files

- crates/termrock-forms/
- form public-consumer tests
- form conformance tests

### Child work

- Split Form data/field adapters, FormDialog, and Wizard.
- Keep shared validation and result types with one owner.
- Use a separate worker for failed-commit and focus-return tests.

### Required changes

1. Keep application values in caller-owned FormData or the current typed adapter. Complete FormLayout lane and error-overflow behavior.
2. Keep FieldSpec and field-state policy separate from domain data.
3. Use shared field controls for input, select, radio, checkbox, and action rows.
4. Preserve validation callbacks that capture caller context.
5. Apply inherited disabled state without erasing read-only behavior.
6. Keep measurement and scroll reveal independent from semantic field mutation.
7. Use shared effect merge for composite field and dialog responses.
8. Make Wizard step changes validate the current step before mutation.
9. Keep Save, Discard, Cancel, Back, Next, and Finish results typed and explicit.

### Required cases

- Invalid submit keeps values, focus, and the dialog open.
- Cancel follows the declared rollback policy.
- Fields outside the viewport can receive keyboard reveal without invalid hit regions.
- Nested selector dismissal returns focus to its field.
- Secret editing does not clone plaintext through a form adapter.
- Conditional and disabled fields do not corrupt traversal.
- Wizard back/next/finish preserves valid values and reports exactly one action.

### Commands

~~~sh
cargo nextest run --locked -p termrock-forms --all-features
cargo nextest run --locked -p termrock --all-features
cargo nextest run --locked --manifest-path external-consumer-demo/Cargo.toml
~~~

### Acceptance

Forms and dialogs use shared controls and preserve caller data through every validated transition.

<a id="t19"></a>

## T19: Use one shell policy for keys, hints, and protected leave

**Implementer:** GPT-6 Luna, max effort.
**Independent reviewer:** GPT-6.1-Sol, high effort.
**Predecessors:** T09, T13, T15, T18.
**Source findings:** [F11](REVIEW.md#f11), [F16](REVIEW.md#f16).

### Owned files

- crates/termrock-shell/
- crates/termrock-runtime/src/keymap.rs for agreed routing fixes
- shared shell consumer tests
- first-party Footer::compose call sites only, resolved to exact files before dispatch

### Child work

- Split key routing, protected leave, and capability-derived hints.
- Keep the shell public policy types with one owner.
- Review copy-versus-quit and modal priority independently.
- Assign TextReport and its bounded lazy-row tests to a separate Luna child task.

### Required changes

1. Use the existing Stakes, KeyContext, QuitKeys, and LeaveGuard concepts.
2. Route top-layer and focused-component input before a fallback quit action.
3. Give an allowed selected-text copy priority over Ctrl+C quit.
4. Disable character shortcuts while an edit or dirty context owns those characters.
5. Require Save/Discard/Cancel for unsaved edits. Require Stop/Stay for active work.
6. Preserve application-specific quit keys and remapping. Do not make Esc a universal quit key.
7. Remove shell delivery globals. Use the runtime KeyboardDelivery snapshot for typed hints and literal-label compatibility.
8. Build hint text from effective bindings and active capability state.
9. Use named shell slots for header, navigation, page, inspector, status, and hints.
10. Keep route data and brand in applications. Keep reusable chrome and policy in shell.
11. Add Footer::compose_with_delivery. Keep compose as a conservative wrapper and migrate first-party callers.
12. Complete TextReport, ReportContent, and ReportRows through crates/termrock-shell/src/report.rs.
13. Preserve report row grids, insets, heading, subheading, empty/body content, height policy, style patches, and cell clipping.
14. Keep TextReport noninteractive. Do not add selection or scrolling to the report painter.

### Required cases

- Selected input plus Ctrl+C copies and does not quit.
- Dirty route leave and quit exercise each result, including failed save.
- A nested modal owns the first applicable key.
- Two detached runtimes with opposite KeyboardDelivery snapshots display different correct hints.
- Pointer and keyboard focus use the same owner policy.
- A second detached consumer uses the shell without Showcase imports.
- Small-screen inspector and help remain reachable and dismissible.
- A report invokes lazy row access only for the declared visible content.
- Report minimum-details height, empty content, custom patches, and clipped body rows obey the existing contract.

### Commands

~~~sh
cargo nextest run --locked -p termrock-shell -p termrock-runtime --all-features
cargo nextest run --locked --manifest-path external-consumer-demo/Cargo.toml
~~~

### Acceptance

Applications can use one shared shell policy without route-specific key or hint workarounds.

<a id="t20"></a>

## T20: Give terminal session state one safe owner

**Implementer:** GPT-6 Luna, max effort.
**Independent reviewer:** GPT-6.1-Sol, high effort.
**Predecessors:** T03, T06, T09.
**Source findings:** [F17](REVIEW.md#f17).

### Owned files

- crates/termrock-session/
- crates/termrock-terminal/src/keyboard.rs ownership migration
- session PTY tests
- session platform manifests
- crates/termrock-core/src/keys.rs global protocol/delivery retirement through the T09 owner

### Child work

- Split session lifetime, bridge restoration, and process-isolated probes.
- Keep process-global lease and panic-hook changes with one owner.
- Use a separate Sol high reviewer for unsafe operations and cleanup order.

### Required changes

1. Add an exclusive active-session lease before any process terminal state changes.
2. Reject nested or concurrent entry with a typed error before terminal mutation.
3. Move protocol negotiation out of terminal-view ownership and into session or neutral protocol values.
4. Gate Unix PTY, file-descriptor, and termios code by target. Keep neutral libraries portable.
5. Give panic cleanup a live-session token. Deactivate it after successful leave.
6. Preserve a host-installed panic hook. Do not restore an obsolete hook over a newer host hook.
7. Make final bridge restoration fallible. Keep retryable state when a restore operation fails.
8. Return cleanup failure through the explicit finish path. Keep Drop as best effort.
9. Preserve raw mode, alternate screen, mouse, paste, cursor, keyboard stack, and stdin restoration order.
10. Adapt supported Unix suspend/resume behavior from the reference after the lifetime contract passes.
11. Keep one session-owned KeyboardNegotiation. Derive KeyboardDelivery from AltKeys, explicit probe answers, input observations, and actual protocol state.
12. Refresh the runtime snapshot after changed probe, observation, suspend, or resume results.
13. Retire the core global protocol and delivery mirrors through migration rows. Preserve refusal and ambiguous-Alt rules.

### Required cases

- Process-isolated tests cover normal exit, entry failure at each stage, panic unwind, repeated sessions, and a late panic after leave.
- Nested and concurrent entry fail without mode changes.
- Injected stdin restoration failure returns an error and retains the declared recovery state.
- Partial writes, Interrupted, zero-byte writes, EOF, and draw/poll/read failures do not hang.
- Each acquired descriptor closes once. Stdin identity and keyboard protocol depth return to their initial state.
- Suspend restores the terminal before suspension. Resume rechecks size and produces a current acknowledged frame.
- Unsupported platform session features fail clearly without blocking platform-neutral components.
- A process-isolated probe installs a host panic hook. That hook observes restored real stdin, termios, and screen modes.
- Two detached runtimes and sequential sessions do not share delivery observations.

### Commands

~~~sh
cargo nextest run --locked -p termrock-session --all-features
cargo nextest run --locked -p termrock-terminal --all-features
cargo clippy --locked -p termrock-session --all-targets --all-features -- -D warnings
~~~

### Acceptance

The explicit exit path reports restoration failures. Completed sessions cannot change later terminal or panic behavior.

<a id="t21"></a>

## T21: Adapt bounded input pumping without weakening frame rules

**Implementer:** GPT-6 Luna, max effort.
**Independent reviewer:** GPT-6.1-Sol, high effort.
**Predecessors:** T09, T20.
**Source findings:** Reference behavior and the shared contract.

### Owned files

- crates/termrock-session/src/ input pump module
- crates/termrock-session/src/session.rs event loop
- session input-burst tests

### Child work

- Keep pump scheduling and queue ownership with one Luna owner.
- Assign event-order and readiness-boundary tests to another worker.
- Use a separate reviewer for frame acknowledgement and dropped input.

### Required changes

1. Adapt the reference InputPump policy through the existing session event loop.
2. Use a bounded queue with a default limit of 64 events and a 16 ms cooperative admission budget.
3. Keep every key repeat, pointer event, resize, paste, and command in its original order.
4. Treat batching as scheduling. Do not replace several semantic events with one synthetic event.
5. Attempt delivery only while the runtime is ready. If it returns pending input, retain that exact input.
6. Settle, prepare, transport, and acknowledge whenever the runtime requires presentation or current geometry.
7. End the batch at a deadline, incompatible event kind, owner change, layer change, capture change, or required frame boundary.
8. Keep a policy that disables batching for comparison and integration tests.
9. Preserve the idle path. Do not add periodic redraws when no deadline or input requires work.
10. Measure presentation count and latency later in T33. Do not claim the reference's frame-count result for an unproved target path.
11. Check elapsed time between deliveries. Admit no further event once the budget expires.
12. Do not claim a hard 16 ms execution limit. A synchronous handler can exceed that duration.

### Required cases

- 120 Down events produce 120 semantic movements, subject only to the component's bounds.
- Resize events preserve order. A following click uses the acknowledged current area.
- A modal opening or focus change ends any batch that could use stale geometry.
- Paste and activation actions are delivered exactly once.
- An unchanged-size probe timeout does not redraw without a runtime deadline.
- Batch-enabled and batch-disabled runs produce the same final state and required checkpoints.
- A fake clock proves count and admission boundaries under continuous input. Record any single-event time overrun separately.

### Commands

~~~sh
cargo nextest run --locked -p termrock-session --all-features
cargo nextest run --locked -p termrock-runtime --all-features
~~~

### Acceptance

Input scheduling is bounded. It preserves all events, semantic results, and presentation requirements.

<a id="t22"></a>

## T22: Connect all Showcase pages through one typed registry

**Implementer:** GPT-6 Luna, max effort.
**Independent reviewer:** GPT-6.1-Sol, high effort.
**Predecessors:** T19, T23, T24, T25, T26.
**Source findings:** [F11](REVIEW.md#f11), [F12](REVIEW.md#f12), [F13](REVIEW.md#f13).

### Owned files

- crates/showcase-ui/src/app.rs
- crates/showcase-ui/src/pages/ registry integration
- crates/showcase-demos/src/pages/mod.rs module integration
- crates/showcase-ui/tests/ route and CLI tests

### Child work

- Keep the typed registry and shared module declarations with one owner.
- Assign CLI tests and route census checks to separate test files.
- Integrate page workers' factory records without concurrent registry edits.

### Required changes

1. Create one PageDescriptor registry with stable PageId, label, group, aliases, factory, capabilities, and minimum-size policy.
2. Preserve all 54 reference identities and current accepted aliases.
3. Replace numeric Placeholder exceptions for ChecksRadios, EmptyError, Notices, MasterDetail, and Diff with typed routes.
4. Give Selection a normal stable route identity.
5. Integrate the page modules from T23 through T26. Remove every paint_placeholder production route.
6. Use shared shell slots, key policy, leave protection, hints, and inspector behavior.
7. Remove page-name whitelists for modal hints. Use the active layer's declared hints.
8. Add useful --help output and deterministic invalid-option errors. Preserve supported page, theme, color, motion, and frame options.
9. Keep the color argument as a capability ceiling. Preserve NO_COLOR behavior.
10. Update route tests and the current-state route census from the registry.

### Required cases

- Exactly 54 required routes have unique identities and factories.
- Every name and alias reaches the correct page through CLI, keyboard, pointer, and navigation search.
- No required route uses placeholder paint or a numeric index exception.
- Reordering navigation groups does not change route identity or page state.
- Ctrl+C selection and protected leave cases pass in the real Showcase shell.
- Unknown options fail with useful text and a nonzero status. Help exits without terminal-mode entry.
- All existing page heading and application visual cases pass.

### Commands

~~~sh
cargo nextest run --locked -p showcase-ui --all-features
cargo nextest run --locked -p showcase-demos --all-features
cargo run --locked -p showcase -- --help
~~~

### Acceptance

All 54 pages are real, reachable consumers. Registry data is the sole route identity source.

<a id="t23"></a>

## T23: Build the five missing shell demonstrations

**Implementer:** GPT-6 Luna, max effort.
**Independent reviewer:** GPT-6.1-Sol, medium effort.
**Predecessors:** T10, T11, T19.
**Source findings:** [F12](REVIEW.md#f12).

### Owned files

- crates/showcase-demos/src/pages/status_hints.rs
- crates/showcase-demos/src/pages/breadcrumbs_rails.rs
- crates/showcase-demos/src/pages/help_keys.rs
- crates/showcase-demos/src/pages/quit_leave.rs
- crates/showcase-demos/src/pages/focus_navigation.rs
- matching bounded page tests

### Child work

- Create one Luna child task for each of the five page files.
- Keep shared registry edits in T22.
- Review shell behavior and reference appearance as separate scopes.

### Required changes

1. Use S-A in COMPONENT-MATRIX.md and the matching five reference page files.
2. Create one module per listed page. Use the existing Page trait and PageUpdate contract.
3. Demonstrate effective status and hints, breadcrumb/rail navigation, help and remapping, protected leave, and focus traversal.
4. Use shared components for every demonstrated control. Keep only example data and page state in the page.
5. Give each example stable IDs and semantic assertions for its input sequence.
6. Demonstrate at least one useful customization on each page.
7. Split the five files into independent Luna child tasks. Give each child the full source and test contract.
8. Return factory names and capability records to T22. Do not edit the shared registry from a page worker.

### Required cases

- Status hints agree with effective keys and active capabilities.
- Breadcrumb and rail actions return typed navigation intent.
- Help remains reachable at a small size and closes to the correct focus owner.
- Quit/leave demonstrates Save, Discard, Cancel, Stop, and Stay through real policy.
- Focus navigation demonstrates forward, backward, disabled, read-only, and nested-modal traversal.
- Each page passes its reference size/color states and one custom-theme case.

### Commands

~~~sh
cargo nextest run --locked -p showcase-demos --all-features
~~~

### Acceptance

All five S-A pages use shared behavior and have executable page tests and reference comparisons.

<a id="t24"></a>

## T24: Build the five missing control and overlay demonstrations

**Implementer:** GPT-6 Luna, max effort.
**Independent reviewer:** GPT-6.1-Sol, medium effort.
**Predecessors:** T12, T14, T15.
**Source findings:** [F10](REVIEW.md#f10), [F12](REVIEW.md#f12).

### Owned files

- crates/showcase-demos/src/pages/toolbars.rs
- crates/showcase-demos/src/pages/tabs.rs
- crates/showcase-demos/src/pages/menus.rs
- crates/showcase-demos/src/pages/popups_tooltips.rs
- crates/showcase-demos/src/pages/file_browser.rs
- matching bounded page tests

### Child work

- Create one Luna child task for each of the five page files.
- Keep shared registry edits in T22.
- Review layer behavior and borrowed customization as separate scopes.

### Required changes

1. Use S-B in COMPONENT-MATRIX.md and the matching reference files.
2. Build toolbars, tabs, menus, popups/tooltips, and a deterministic file-browser example.
3. Use shared action, selection, layer, and data-revision behavior.
4. Keep file-browser data in a deterministic fixture. Do not access the user's files.
5. Demonstrate tab close, disabled actions, nested menus, tooltip delay, and an empty file selection.
6. Show custom row/part styles and remapped actions through public API.
7. Use independent Luna workers for separate page modules.
8. Return module and factory metadata to T22. Keep shared registry edits with its owner.

### Required cases

- Keyboard and pointer actions produce the same typed result.
- Tabs preserve identity after close, reorder, and overflow.
- Nested menus and tooltips respect active-layer ownership and dismissal.
- File-browser filtering preserves selected identity and handles empty results.
- All pages remain usable with long Unicode labels and narrow geometry.
- Every displayed option changes real component behavior or output.

### Commands

~~~sh
cargo nextest run --locked -p showcase-demos --all-features
cargo nextest run --locked -p termrock-controls -p termrock-overlays --all-features
~~~

### Acceptance

All five S-B pages are live examples. No page contains a copied widget event or overlay implementation.

<a id="t25"></a>

## T25: Build the five missing data, layout, and text demonstrations

**Implementer:** GPT-6 Luna, max effort.
**Independent reviewer:** GPT-6.1-Sol, medium effort.
**Predecessors:** T11, T14, T16, T17.
**Source findings:** [F12](REVIEW.md#f12), [F15](REVIEW.md#f15).

### Owned files

- crates/showcase-demos/src/pages/columns.rs
- crates/showcase-demos/src/pages/meters.rs
- crates/showcase-demos/src/pages/splits_stacks.rs
- crates/showcase-demos/src/pages/pane_tree.rs
- crates/showcase-demos/src/pages/text_utilities.rs
- matching bounded page tests

### Child work

- Create one Luna child task for each of the five page files.
- Keep shared registry edits in T22.
- Review geometry and data identity independently.

### Required changes

1. Use S-C in COMPONENT-MATRIX.md and the matching reference files.
2. Demonstrate typed columns, meter policies, split/stack layout, pane trees, and text utilities.
3. Use shared measurement, clipping, pane identity, and scrollbar mechanisms.
4. Keep pane topology in caller-owned state. Use one shared geometry resolver.
5. Demonstrate explicit meter thresholds and semantic color policies.
6. Show grapheme-safe width, wrap, truncation, and selection behavior with varied scripts.
7. Create independent Luna child tasks for the separate page modules.
8. Return factories and coverage records to T22.

### Required cases

- Column reorder and sizing preserve keys and selected cells.
- Meter zero, boundary, over-range, and custom-threshold states follow the declared policy.
- Split drag paints only the seam and preserves pane minimum sizes.
- Pane collapse, resize, and focus restore keep stable identity.
- Text utility output is valid UTF-8 and respects terminal-cell width.
- All pages pass the required color modes and small-screen reachability cases.

### Commands

~~~sh
cargo nextest run --locked -p showcase-demos --all-features
cargo nextest run --locked -p termrock-grid -p termrock-viewport -p termrock-text --all-features
~~~

### Acceptance

All five S-C pages expose real reusable layout and text behavior without local repair code.

<a id="t26"></a>

## T26: Build the nine missing application-pattern demonstrations

**Implementer:** GPT-6 Luna, max effort.
**Independent reviewer:** GPT-6.1-Sol, medium effort.
**Predecessors:** T07, T08, T16, T17, T18, T19.
**Source findings:** [F12](REVIEW.md#f12).

### Owned files

- crates/showcase-demos/src/pages/query_workbench.rs
- crates/showcase-demos/src/pages/connections.rs
- crates/showcase-demos/src/pages/finder.rs
- crates/showcase-demos/src/pages/disk_cleanup.rs
- crates/showcase-demos/src/pages/activity.rs
- crates/showcase-demos/src/pages/workspaces.rs
- crates/showcase-demos/src/pages/accounts_usage.rs
- crates/showcase-demos/src/pages/launch_cockpit.rs
- crates/showcase-demos/src/pages/readonly_report.rs
- showcase-data fixtures for these pages

### Child work

- Create the three named child groups in the task steps.
- Split each group by page file when capacity permits.
- Give each child the exact reference file and its matrix row.

### Required changes

1. Use S-D in COMPONENT-MATRIX.md and all nine matching reference files.
2. Assign query workbench, connections, and finder to one Luna child task.
3. Assign disk cleanup, activity, and workspaces to a second Luna child task.
4. Assign accounts/usage, launch cockpit, and read-only report to a third Luna child task.
5. Let each child divide further by file when worker capacity permits.
6. Build each page from shared grids, editors, forms, navigation, feedback, overlays, and shell slots.
7. Keep domain fixtures and simulation state in Showcase data or the page.
8. Do not copy Jackin, Holla, or TablePro product screen models into Termrock.
9. Demonstrate validation, empty/error states, pending edits, task progress, and cancellation where the page requires them.
10. Return module, factory, and capability records to T22. Keep shared registry edits with its owner.
11. Build the report page from facts, meters, notices, refresh/copy, and navigation actions. Preserve read-only, offline, and stale states.

### Required cases

- Query edit/run/result and connection validation use typed shared actions.
- Finder filter/selection/reveal works with empty and large data.
- Disk cleanup remains a deterministic simulation with explicit confirmation.
- Activity timing excludes prompt wait where the reference contract requires it.
- Workspace and account changes preserve stable item identity.
- Launch progress and report read-only behavior use shared state policies.
- Each page has a real input journey, a small-screen case, and a reference comparison.

### Commands

~~~sh
cargo nextest run --locked -p showcase-data -p showcase-demos --all-features
~~~

### Acceptance

All nine S-D pages provide complete example journeys through reusable APIs.

<a id="t27"></a>

## T27: Finish Holla as a thin deterministic consumer

**Implementer:** GPT-6 Luna, max effort.
**Independent reviewer:** GPT-6.1-Sol, high effort.
**Predecessors:** T08, T14, T15, T18, T19, T21.
**Source findings:** [F01](REVIEW.md#f01).

### Owned files

- crates/holla-domain/
- crates/holla-catalog/
- crates/holla-plan/
- crates/holla-sim/
- crates/holla-ui/
- crates/holla/

### Child work

- Split domain/simulation checks, UI adoption, and journey tests.
- Keep shared application state transitions with one owner.
- Use separate reviewers for simulation safety and UI behavior.

### Required changes

1. Map every retained Holla requirement to current source and an approved journey scenario.
2. Use shared controls, navigation, grids, forms, overlays, shell, and clipboard outcomes.
3. Remove app-local copies of shared selection, focus, row paint, scroll, and leave behavior.
4. Keep catalog ranking, plan validation, target facts, effect policy, and simulation in Holla owners.
5. Preserve reviewed-plan identity, target revision, and dependency success checks before simulated execution.
6. Keep required, optional, excluded, and policy-skipped states distinct.
7. Port the reference timing rule that excludes prompt-wait time from task duration.
8. Preserve normalized alias identity and deterministic catalog behavior.
9. Correct current visual failures against the accepted source reference.
10. Keep real host effects outside this goal. Preserve existing CLI and help behavior.

### Required cases

- Home, search, files, cleanup, activities, plans, prompts, history, and completion journeys have current evidence.
- Invalid plan or stale target fails before any simulated effect.
- Wrong confirmation text does not expose the expected secret or approval phrase.
- Prompt wait, pause, stop, and resume produce correct task duration.
- Shared key, dirty-leave, clipboard, modal, and narrow-screen cases pass.
- Historical replay and current visual scenarios pass without a blanket expected-byte change.

### Commands

~~~sh
cargo nextest run --locked -p holla-domain -p holla-catalog -p holla-plan -p holla-sim
cargo nextest run --locked -p holla-ui -p holla --all-features
cargo run --locked -p holla -- --help
~~~

### Acceptance

Holla owns domain behavior and consumes shared UI behavior. All retained required journeys pass.

<a id="t28"></a>

## T28: Finish Jackin Preview after the ownership moves

**Implementer:** GPT-6 Luna, max effort.
**Independent reviewer:** GPT-6.1-Sol, high effort.
**Predecessors:** T07, T08, T15, T17, T18, T19, T21.
**Source findings:** [F01](REVIEW.md#f01), [F05](REVIEW.md#f05), [F06](REVIEW.md#f06).

### Owned files

- crates/jackin-preview-app/
- crates/jackin-preview-host-ui/
- crates/jackin-preview-capsule-ui/
- crates/jackin-preview-presentation/
- crates/jackin-preview-sim/
- crates/jackin-preview/

### Child work

- Split host screens, Capsule screens, and app timing/journey checks.
- Keep shared route/world timing with one owner.
- Use separate reviewers for product state and UI behavior.

### Required changes

1. Use the new Jackin UI owners from T07. Remove duplicate page assembly from the app coordinator.
2. Keep routes, virtual world, run identity, product status, and launch policy in their application owners.
3. Use shared forms, menus, lists, meters, splits, terminal views, overlays, and shell behavior.
4. Preserve Bootstrap, Event, Tick, and Settle semantics. Advance product time only through its route/world cadence.
5. Keep the earliest repaint deadline through early input, modal changes, and animation dismissal.
6. Keep status as a pure projection with stable action identity and explicit tone.
7. Preserve RunId validation, collision avoidance, deterministic labels, secret masks, and save-key validation.
8. Complete host, manager, account, settings, launch, Capsule, rain/handoff, failure, and outro journeys.
9. Correct the accounts-mixed visual failure through shared behavior or application composition.
10. Keep full, reduced, and paused motion deterministic.

### Required cases

- Repeated draw at the same time does not change product state.
- Early input cannot delay a scheduled frame or advance the virtual world twice.
- A closed route ignores stale results and restores valid focus.
- Secret fields remain masked in view, copy, logs, and diagnostics.
- Manager and Capsule pages have exact reference frames and real action assertions.
- Small terminals preserve access to required actions and dismissal.
- Named timing and RunId behavior has approved scenarios and passes at the final task commit.

### Commands

~~~sh
cargo nextest run --locked -p jackin-preview-domain -p jackin-preview-sim -p jackin-preview-presentation
cargo nextest run --locked -p jackin-preview-host-ui -p jackin-preview-capsule-ui -p jackin-preview-app --all-features
cargo run --locked -p jackin-preview -- --help
~~~

### Acceptance

Jackin has clear domain, presentation, UI, and app owners. Shared library crates contain no Jackin screen model.

<a id="t29"></a>

## T29: Finish TablePro without moving SQL policy into Grid

**Implementer:** GPT-6 Luna, max effort.
**Independent reviewer:** GPT-6.1-Sol, high effort.
**Predecessors:** T08, T13, T15, T16, T17, T18, T19, T21.
**Source findings:** [F01](REVIEW.md#f01), [F06](REVIEW.md#f06), [F20](REVIEW.md#f20).

### Owned files

- crates/tablepro-demo/
- crates/tablepro-sql/
- crates/tablepro-domain/
- crates/tablepro-ui/
- crates/tablepro/

### Child work

- Split domain/order checks, UI adoption, and journey tests.
- Keep draft/edit transaction state with one owner.
- Use separate reviewers for data identity and visual behavior.

### Required changes

1. Complete the UI-state moves from T08 and keep result/order semantics in domain or SQL.
2. Keep the current cmp_values_nulls_last implementation. Do not restore the older reversed comparator.
3. Use shared Grid sorting requests and stable row/column keys.
4. Complete connection, query, filter, history, structure, plan, result, picker, and dialog journeys.
5. Preserve pending edits and draft validation through sort, filter, schema projection, and undo.
6. Use shared field, completion, overlay, clipboard, and protected-leave behavior.
7. Keep result grids read-only where the application contract requires it.
8. Correct the history-tab visual failure without changing an approved expectation.
9. Keep all database work simulated. Do not connect to an external database.
10. Update stale documentation only after fresh approved behavior cases pass.

### Required cases

- NULL sorts last in both directions for typed comparisons.
- Numeric values sort numerically, including documented floating-point boundary cases.
- Sort and filter preserve surviving row, cursor, checked, anchor, and edit identity.
- Filter Apply followed by edit/save affects the intended domain row.
- Invalid edits preserve the prior value and display the typed error.
- Masked connection secrets do not enter copied output or logs.
- All required reference journeys and narrow-screen cases pass.

### Commands

~~~sh
cargo nextest run --locked -p tablepro-demo -p tablepro-sql -p tablepro-domain
cargo nextest run --locked -p tablepro-ui -p tablepro --all-features
cargo run --locked -p tablepro -- --help
~~~

### Acceptance

TablePro uses shared UI components and retains its domain semantics and visual behavior.

<a id="t30"></a>

## T30: Prove easy use and advanced customization in real consumers

**Implementer:** GPT-6 Luna, max effort.
**Independent reviewer:** GPT-6.1-Sol, high effort.
**Predecessors:** T05, T10, T12, T13, T14, T15, T16, T17, T18, T19, T20.
**Source findings:** [F04](REVIEW.md#f04), [F08](REVIEW.md#f08), [F19](REVIEW.md#f19).

### Owned files

- external-consumer-demo/
- crates/termrock-consumer-example/
- crates/termrock/examples/
- docs/implementation/pr38-final/recipes/

### Child work

- Create one Luna child task per complete example.
- Keep facade exports with their established owner.
- Use a detached consumer reviewer who imports only the public API.

### Required changes

1. Replace handwritten MenuLeaf and StatusLeaf with the shared adapters.
2. Create a complete quick-start example with a real Button, TextInput, typed action, and session setup.
3. Create a master/detail example with caller-owned list data, stable keys, a form, a modal, and status.
4. Create a custom-theme example with part overrides, slot Clear, custom glyphs, and level-specific palettes.
5. Create a custom-row example with borrowed data and custom hit-safe paint.
6. Create a component-author example through termrock::author using the existing traits.
7. Create an external-host example that uses prepare, draw_prepared, and acknowledge_presented without a session helper or testing feature.
8. Keep copyable recipes as complete source files with dependency declarations and licenses from established repository material.
9. Compile each example through public APIs. Run at least one interaction sequence for each behavior example.
10. Document the smallest import and feature set beside each recipe.

### Required cases

- Common application examples need no raw key loop, duplicate focus registry, or copied scrollbar logic.
- Custom author examples can use explicit geometry without private modules.
- Two copies of one recipe have independent identity, state, styles, and actions.
- A scoped style override cannot change a sibling.
- Every documented builder changes a named visible or semantic result.
- The detached consumer builds with its own lockfile and no application dependency.

### Commands

~~~sh
cargo check --locked -p termrock --examples --all-features
cargo nextest run --locked -p termrock-consumer-example
cargo nextest run --locked --manifest-path external-consumer-demo/Cargo.toml
~~~

### Acceptance

The public API supports simple use and meaningful extension through complete tested examples.

<a id="t31"></a>

## T31: Close every required component and capability record

**Implementer:** GPT-6 Luna, max effort.
**Independent reviewer:** GPT-6.1-Sol, high effort.
**Predecessors:** T12, T13, T14, T15, T16, T17, T18, T19, T22, T30.
**Source findings:** [F12](REVIEW.md#f12), [F13](REVIEW.md#f13).

### Owned files

- crates/termrock-conformance/
- crates/showcase-ui/src/coverage/
- crates/showcase-ui/tests/coverage_gate.rs
- component-ownership.json final acceptance fields

### Child work

- Split conformance proof work by component family.
- Keep the coverage record merger with one Luna owner.
- Use an independent reviewer to sample negative controls across families.

### Required changes

1. Use the T01 inventory and every row in COMPONENT-MATRIX.md.
2. Add executable conformance cases for each required state, option, action, and extension path.
3. Bind each proof to a real owner, route or consumer, input sequence, expected result, and exact test.
4. Remove a pending coverage row when its proof passes. Keep valid exclusions explicit and narrowly scoped.
5. Test aliases through their real owner and descriptor types through their consuming widgets.
6. Use meaningful negative controls for disconnected builders, missing state specimens, and absent update/draw paths.
7. Keep live interaction separate from inert reference specimens.
8. Verify all 54 routes and additional public components beyond W45.
9. Require explicit remaining obligations for a provider-owned behavior. Do not claim a provider test proves a widget's complete behavior.

### Required cases

- No required W ID, extra component, route, or behavior remains pending.
- Removing one required action path causes the related proof to fail.
- Removing a required option effect causes its example or conformance case to fail.
- A nonvisual type has a concrete consumer proof or a reviewed scope reason.
- Every accepted proof selects and runs its named test.
- Coverage counts come from source records and test results, not a manually entered total.

### Commands

~~~sh
cargo nextest run --locked -p termrock-conformance --all-features
cargo nextest run --locked -p showcase-ui --test coverage_gate --all-features
cargo nextest run --locked -p termrock --all-features
~~~

### Acceptance

Every required public capability has executable evidence or an approved non-required disposition. Required omissions keep this task open.

<a id="t32"></a>

## T32: Connect the complete gate set to maintained CI

**Implementer:** GPT-6 Luna, max effort.
**Independent reviewer:** GPT-6.1-Sol, high effort.
**Predecessors:** T02, T04, T06, T21, T27, T28, T29, T31, T33, T34.
**Source findings:** [F01](REVIEW.md#f01), [F02](REVIEW.md#f02), [F14](REVIEW.md#f14), [F18](REVIEW.md#f18).

### Owned files

- .velnor/config.toml
- mise.toml and mise.lock when the approved generator pin changes
- tools/ acceptance and inventory integration
- generator-owned output only through regeneration

### Child work

- Split gate specification, generator input changes, and verdict mutation checks.
- Keep generated output with one Luna owner.
- Review complete job accounting independently.

### Required changes

1. Add isolated features, external consumers, package checks, performance work bounds, and complete visual tiers to the generated CI plan.
2. Use existing inventory and architecture tools. Preserve source, archive, receipt, and binary identity checks.
3. Keep approved functional scenarios separate from real-CLI visual scenarios.
4. Reject empty filters, missing reports, missing required artifacts, stale heads, and hidden shell pipeline failures.
5. Run expensive visual cases in deterministic chunks with complete case and checkpoint accounting.
6. Make the required verdict include every applicable gate.
7. Change .velnor/config.toml and other declared inputs. Regenerate managed output with the pinned Velnor tool.
8. If the generator lacks a required capability, prepare and test an upstream correction. Follow current authorization before external submission.
9. Do not hand-edit generated workflows or add a local workflow shim.
10. Preserve immutable operational evidence when current Markdown guidance changes.

### Required cases

- A missing required artifact or skipped required job makes the verdict fail.
- A failed command before a successful pipe consumer still fails the job.
- A wrong source SHA or stale report fails the result check.
- Full visual results account for every required scenario and checkpoint.
- Removing a feature or package probe causes an inventory failure.
- Generated files exactly match the pinned generator and declared inputs.

### Commands

~~~sh
mise exec -- velnor-actions generate
PYTHONDONTWRITEBYTECODE=1 python3 tools/test_perf_workflow.py
PYTHONDONTWRITEBYTECODE=1 python3 tools/test_acceptance_architecture.py
PYTHONDONTWRITEBYTECODE=1 python3 tools/test-inventory/test_inventory.py
cargo fmt --all -- --check
~~~

### Acceptance

Current-head CI runs and requires the complete declared gate set. No required failure or skip is hidden.

<a id="t33"></a>

## T33: Prove bounded work and measure performance

**Implementer:** GPT-6 Luna, max effort.
**Independent reviewer:** GPT-6.1-Sol, high effort.
**Predecessors:** T02, T09, T10, T14, T16, T17, T21, T22, T27, T28, T29.
**Source findings:** [F18](REVIEW.md#f18).

### Owned files

- crates/termrock-test-support/src/perf.rs
- crates/termrock-test-support/tests/perf*.rs
- application perf tests
- tools/test_perf_workflow.py performance contract after T02

### Child work

- Split workload preparation by owner crate.
- Run final measurements serially on the controlled host.
- Use independent reviewers for deterministic work and statistical claims.

### Required changes

1. Use the initial workload records from T01 and the exact final candidate source.
2. Add explicit gate mode that rejects missing baselines and zero selected workloads.
3. Keep exploratory measurements separate from blocking acceptance.
4. Run deterministic allocation, requested-byte, accessor, and work-count checks in release mode with PERF_FULL=1.
5. Preserve the existing picker and scroll-region zero-allocation contracts for their named warm operations.
6. Measure controls, Unicode edits, 1,000/100,000-row collections, grid sampling, viewport append/resize, focus/hit dispatch, and overlay open/close.
7. Measure one real frame/input journey from each of the four applications.
8. Measure cold setup separately from warm update and draw.
9. Use repeated paired timing on one controlled idle runner. Record raw samples and a confidence interval.
10. Use the existing 20 percent timing threshold only after runner noise is below that threshold.
11. Compare batching enabled and disabled. Report actual event, frame, and latency counts.
12. Return a measured regression to its owner task. Do not raise a budget only to pass.

### Required cases

- Injected allocation growth, a missing workload, and a missing baseline fail gate mode.
- Warm visible-row work does not scale with hidden rows when the API promises viewport-bounded access.
- A stale cache is rejected even when caching would improve a measurement.
- Reference and candidate use the same compiler, flags, host, dataset, dimensions, and color mode.
- Every reported speed or memory claim has a raw measurement and exact operation definition.
- Thread-local allocation counters are not reported as total heap or worker-thread measurements.

### Commands

~~~sh
PERF_FULL=1 cargo nextest run --locked --release -p termrock-test-support --test perf --test perf_collections
PERF_FULL=1 cargo nextest run --locked --release -p showcase-ui -p tablepro-ui -p holla-ui -p jackin-preview-app --test perf
~~~

### Acceptance

Deterministic work bounds pass. Controlled timing stays within the declared budget or has a reviewed corrective change.

<a id="t34"></a>

## T34: Verify crate packages outside the repository

**Implementer:** GPT-6 Luna, max effort.
**Independent reviewer:** GPT-6.1-Sol, high effort.
**Predecessors:** T05, T06, T07, T08, T20, T30.
**Source findings:** [F19](REVIEW.md#f19).

### Owned files

- production and internal Cargo.toml package metadata
- package README files
- package verification fixture under the existing consumer structure
- release manifest and migration record

### Child work

- Split package metadata, staged-registry fixtures, and detached consumers.
- Keep version selection and publication order with one Luna owner.
- Use a separate reviewer to inspect the package bytes and dependency graph.

### Required changes

1. Check the registry state for every proposed public crate name and version.
2. Declare the public package list. Mark applications, demos, test tools, and evidence tools publish=false.
3. Keep the existing version if this is an unpublished initial release. Use a compatible release policy if a published API must change.
4. Add version requirements to every internal published dependency while retaining local path development.
5. Add descriptions, repository metadata, feature documentation, package readme paths, and established license material.
6. Review each cargo package --list result. Exclude large visual corpora, local reports, and repository-only tooling from production packages.
7. Build the package set in dependency order.
8. Consume the staged packages through an isolated local registry with clean caches and no checkout-path patch.
9. Run one detached leaf probe per public crate and one full facade consumer.
10. Prepare a publication order and dry-run report. Do not publish during this task.

### Required cases

- A package consumer cannot read or resolve a path in the source checkout.
- Packaged examples use only public dependencies and included files.
- No production package requires an application, conformance, or visibility package.
- Minimal and full feature configurations work from staged package bytes.
- Source, API migration, and package version agree.
- Any destination-registry dependency availability limit is distinct from a source or manifest failure.

### Commands

~~~sh
cargo package --list -p termrock-core
cargo package --list -p termrock
cargo publish --dry-run -p termrock-core --locked
~~~

### Acceptance

All public packages pass the staged registry consumer gate. Publication remains a separate authorized action.

<a id="t35"></a>

## T35: Rewrite current guides and publish the migration instructions

**Implementer:** GPT-6 Luna, max effort.
**Independent reviewer:** GPT-6.1-Sol, medium effort.
**Predecessors:** T22, T27, T28, T29, T30, T31, T34.
**Source findings:** [F20](REVIEW.md#f20).

### Owned files

- README.md
- docs/README.md
- docs/current-state.md
- docs/architecture.md
- docs/product.md
- docs/plan.md
- docs/development.md
- docs/components-contracts.md
- docs/contracts/
- docs/implementation/pr38-final/ final records

### Child work

- Split maintained documents by file.
- Keep public API migration records with one Luna owner.
- Run technical review before the final language and link check.

### Required changes

1. Update current documents to describe the verified architecture, API, features, and route coverage.
2. Keep historical evidence and source-pinned audit records unchanged.
3. Write a short quick start and link complete tested source examples.
4. Write a crate-selection guide and a customization guide.
5. List every changed public path, replacement, feature requirement, semantic change, and migration example.
6. Describe supported platforms and adapters exactly as tested.
7. State synchronous host ownership. Add async guidance only for a real tested adapter.
8. Replace stale test and package names. Label unexecuted proposed commands clearly.
9. Use ASD-STE100 rules for all new prose. Apply the technical word list in SOURCES.md.
10. Check sentences, terminology, links, and source examples.
11. Keep root AGENTS.md lean and linked to the maintained rules.

### Required cases

- Each documented API example compiles from its linked source file.
- Each changed public path has an explicit migration row.
- Every current pass claim names an exact source revision and result.
- All local Markdown links and command target names resolve.
- Procedural sentences stay within 20 words. Descriptive sentences stay within 25 words.
- Sol low can check final text only after medium or high technical acceptance.

### Commands

~~~sh
cargo check --locked -p termrock --examples --all-features
cargo nextest run --locked -p termrock-xtask --all-features
git diff --check
~~~

### Acceptance

Current guides match verified source. The complete new prose uses the defined STE writing rules.

<a id="t36"></a>

## T36: Verify the final integrated commit and close the goal

**Implementer:** GPT-6 Luna, max effort.
**Independent reviewer:** GPT-6.1-Sol, high effort.
**Predecessors:** T04, T32, T33, T34, T35, T37, T38.
**Source findings:** [F01](REVIEW.md#f01), [F14](REVIEW.md#f14), [F19](REVIEW.md#f19).

### Owned files

- docs/implementation/pr38-final/results/final-acceptance.json
- docs/implementation/pr38-final/FINAL-REPORT.md
- final corrections through their original task owners

### Child work

- Assign the two required independent review scopes to separate Sol high agents.
- Return each correction to a Luna max task with its original file owner.
- Use one Luna owner for the final evidence record.

### Required changes

1. Freeze the final candidate commit and verify the worktree state.
2. Run every applicable gate in VALIDATION.md on that exact source.
3. Run the complete accepted visual case set twice with separate output roots.
4. Confirm exact case, checkpoint, binary, source, and reference identities.
5. Use one independent Sol high agent for API and dependency review.
6. Use another independent Sol high agent for UI, lifecycle, visual, and evidence review.
7. Return every correction to a Luna max worker. Rerun affected gates after integration.
8. Fetch all current pull request reviews, comments, replies, and unresolved or outdated threads.
9. Record the disposition of each finding. Use current authorization for any external reply.
10. Check all tasks and all required inventory rows. Keep the goal open if any required item is failed, pending, or blocked.
11. Prepare the final report, migration guide, package report, and exact source references.
12. Merge or publish only when the user has authorized that specific action.

### Required cases

- The final required CI verdict is successful at the final candidate SHA.
- Every required task is accepted. Every required component and route has current evidence.
- Visual runs contain zero unexplained cell or cursor differences and zero missing required checkpoints.
- Known accepted design differences have exact source-bound approval records.
- All supported leaf, facade, platform, and package checks pass.
- No review finding is unaddressed. No old result is presented as a current result.

### Commands

~~~sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo nextest run --locked --workspace --all-features
cargo nextest run --locked --manifest-path external-consumer-demo/Cargo.toml
git diff --check
~~~

### Acceptance

All independent acceptance categories pass at one final source revision. The report states any external action that remains unauthorized.

<a id="t37"></a>

## T37: Remove legacy tests and nonfunctional source comments

**Implementer:** GPT-6 Luna, max effort.
**Independent reviewer:** GPT-6.1-Sol, high effort.
**Predecessors:** T00.
**Source findings:** [F01](REVIEW.md#f01), [F18](REVIEW.md#f18), [F20](REVIEW.md#f20).

### Owned files

- Candidate test functions, test modules, test targets, harnesses, fixtures, and doctests across all languages.
- Test-only helpers, dependencies, package entries, and lockfile entries that become unused.
- CI generator inputs that cite removed targets.
- Generated output only through its declared generator.
- Nonfunctional source and code comments identified by T00.
- `docs/contracts/safety-invariants.md` and narrow active-document citations mapped by T00.
- The T37 cleanup record under `results/`.

### Child work

- Split language-specific deletion by disjoint paths.
- Keep one Luna owner for shared manifests, lockfile changes, and CI generator inputs.
- Use an independent Sol high reviewer for the full deletion and build evidence.
- Do not start T38 or any test-writing task before T37 is accepted.

### Required changes

1. Use T00's complete inventory, source hashes, failure records, and directive list.
2. Delete every legacy test function, test module, test target, harness, fixture, and doctest from the candidate.
3. Remove orphaned test helpers, dependencies, package entries, and CI citations.
4. Remove all nonfunctional line, block, documentation comments, and docstrings across languages.
5. Move required safety rationale and API prose into maintained contracts or Markdown.
6. Remove active documentation citations to deleted test targets or label immutable source links as historical.
7. Preserve legal notices, SPDX identifiers, parser and linter directives, shell shebangs, and required generated directives.
8. Keep required CLI help in command attributes. Keep production-owned diagnostics in production code.
9. Preserve production generated output byte-for-byte. Regenerate managed CI output after input changes.
10. Preserve all production behavior, public requirements, immutable references, tags, and approved oracle evidence.
11. Do not add a test, harness, fixture, doctest, snapshot, or replacement gate.
12. If an active linter requires a deleted comment, stop that path and request a Sol high design decision.
13. Do not add blanket lint exceptions to pass cleanup.
14. Record every removed path, orphan dependency, retained directive, generated hash, and unresolved behavior obligation.

### Required cases

- T00's inventory reconciles with every removed test and harness path.
- No nonfunctional source comment or docstring remains outside the declared exceptions.
- No legacy test target, test function, test module, harness, fixture, or doctest remains active.
- No active generator input or CI plan cites a removed test target.
- Cleanup adds no production build failure against the T00 baseline.
- A pre-existing build failure stays assigned to its owner task. It does not block unrelated cleanup.
- Generated production files remain byte-identical. Managed CI output matches its declared generator inputs.
- Legal, SPDX, parser, linter, shebang, and generated directives remain valid.
- Required CLI help and production diagnostics remain available.
- Production behavior and required public APIs remain present.
- An empty test selection is reported as `NOT VERIFIED`, never as a pass.
- T37 adds no replacement test or harness.

### Commands

~~~sh
git diff --check
rg -n '#\[(test|cfg\(test\))\]|doctest|test-inventory|visual-harness' --glob '!docs/implementation/pr38-final/**'
cargo metadata --locked --format-version 1 --no-deps
cargo check --locked --workspace --all-targets --all-features
mise exec -- velnor-actions generate
~~~

### Acceptance

Structural deletion, generator output, and build changes pass review against T00. Product behavior remains unverified until approved functional cases run.

<a id="t38"></a>

## T38: Approve the functional test architecture

**Implementer:** GPT-6 Luna, max effort.
**Independent reviewer:** GPT-6.1-Sol, medium effort.
**Predecessors:** T01.
**Source findings:** [F01](REVIEW.md#f01), [F14](REVIEW.md#f14), [F18](REVIEW.md#f18), [F20](REVIEW.md#f20).

### Owned files

- `results/test-architecture-design.json`
- `results/test-architecture.json`
- Test architecture records owned by this task

### Child work

- Use a Sol medium architect to make and record the test design decision from T01's inventory.
- Use one Luna max worker to record the approved decision in the case manifest.
- Use a different Sol medium reviewer to accept the written record before any test edit.
- Do not edit tests, harnesses, fixtures, or product source in T38.

### Required changes

1. Ask Sol medium to derive required behavior from public contracts, production entry points, T01's inventory, and COMPONENT-MATRIX.md.
2. Require the architect to record case shape, oracle, fault model, environment, and limits before Luna writes the manifest.
3. Prefer integration/functional scenarios. Use an existing owner seam when the source requires it.
4. Do not create a parallel test framework without a named source constraint and Sol medium decision.
5. Prefer real user scenarios through production APIs and observable output.
6. Specify positive, negative, boundary, and recovery cases for each required behavior.
7. Name the production entry point, public API, observable result, independent oracle, and controlled environment for each case.
8. Use realistic controlled dependencies and faults at unowned external boundaries.
9. Never mock Termrock-owned production code. Use an external double only with a reviewed reason and fidelity record.
10. Define deterministic setup, inputs, clock, seed, tool versions, cleanup, and repeat rules.
11. Define red as a runtime assertion failure for a known defect or reviewed realistic fault.
12. Define green as the same assertion passing after correction or fault removal. Compiler or setup failures are not red evidence.
13. Record only the declared cases, source, tools, and environment as green. Do not claim absolute correctness.
14. Allow test-only timing or performance counters only as supplemental evidence under an approved contract.
15. Treat test count as inventory, not as a success metric.
16. Require an independent Sol medium review of every task-specific test plan before test edits.
17. If a scenario exposes a product gap, stop test edits. Require Sol medium testability-gap analysis first. Use Sol high only if the evidence requires a product architecture decision.
18. Keep T38 approval scoped to architecture. It does not approve later test changes.

### Required cases

- Every required component, route, application journey, API contract, and gate has a proposed functional case or a named open disposition.
- Each case uses a production entry point and observes public behavior.
- Positive, negative, boundary, and recovery cases cover the stated behavior.
- Each oracle can detect an incorrect result without reusing the implementation's internal decision.
- No case mocks Termrock-owned production code.
- Any external double has a reviewed reason and fidelity statement.
- Environment and deterministic repeat rules make results independently reproducible.
- Red and green evidence use the same runtime assertion and declared environment.
- Missing, skipped, unselected, or setup-failed cases cannot report green.
- The architecture record names the Sol medium architect, independent reviewer, and exact reviewed manifest hash.
- Luna writes the manifest only after the Sol medium design decision exists.
- T38 adds no test, harness, fixture, doctest, or production change.

### Commands

~~~sh
git diff --check
python3 -m json.tool docs/implementation/pr38-final/results/test-architecture.json
~~~

### Acceptance

The Sol medium architect and independent reviewer approve the complete test architecture and manifest. Test or harness edits remain blocked until each later task receives its own review.
