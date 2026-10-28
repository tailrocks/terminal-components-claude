# Plan to finish Termrock in pull request 38

## 1. Decision

Finish the work on `parity/termrock-visual-baseline`.
Keep the useful crate boundaries and runtime contracts in PR #38.
Use PR #26 as the source for appearance and interaction.
Port each selected behavior through the target API.

Do not merge PR #26 as a whole.
Do not start another component framework.
Do not remove advanced behavior only to shorten the implementation.
Keep one implementation owner for each shared behavior.

The review has 20 findings in [REVIEW.md](REVIEW.md).
The work has 39 tasks in [TASKS.md](TASKS.md).
The exact dependency graph is in [task-index.json](task-index.json).
All tasks start without acceptance evidence.

Read [TESTING-RULES.md](TESTING-RULES.md) before test or harness design.
T00 captures legacy evidence without changing Termrock source.
T37 removes legacy tests, harnesses, fixtures, doctests, and nonfunctional code comments.
T01 inventories the cleaned source. T38 approves the functional test architecture.
Only then may a task add a reviewed functional test or harness.
The empty suite after T37 is unverified. It is never a passing result.

## 2. Required outcomes

| ID | Required outcome | Main tasks | Completion evidence |
| --- | --- | --- | --- |
| R01 | Every public crate has a clear owner and allowed dependency set. | T01, T03, T05–T08 | Resolved graphs, isolated consumers, and ownership checks. |
| R02 | Common components need little setup and have useful defaults. | T10, T12–T19, T30 | Complete public examples and real input scenarios. |
| R03 | Advanced customization uses the same state and runtime contracts. | T09–T19, T30, T31 | Custom rows, parts, slots, themes, components, and external-host scenarios. |
| R04 | All required reference behavior and appearance remain available. | T04, T11–T19, T22–T29, T31 | Complete route, component, application, and visual records. |
| R05 | Terminal ownership and input scheduling have explicit failure behavior. | T09, T20, T21 | Process-isolated lifecycle and event-order scenarios. |
| R06 | Warm draw and update keep their declared work bounds. | T14, T16, T17, T21, T33 | Allocation, accessor, work-count, and controlled timing results. |
| R07 | Public packages work outside the repository. | T05, T30, T34 | Staged package bytes and detached registry consumers. |
| R08 | Agents and users can follow one verified set of instructions. | T00–T02, T31, T32, T35, T36, T37, T38 | Root links, approved examples, current CI, and the final report. |

The shadcn/ui comparison defines a usability goal.
Use clear source ownership, small composition steps, useful defaults, and direct customization.
Provide complete recipes that callers can copy and modify.
Keep the Rust crate model and the existing terminal runtime.
This goal does not add a component registry service or a new installer CLI.

## 3. Source and scope

The source review used these fixed commits:

| Role | Pull request or source | Commit |
| --- | --- | --- |
| Implementation base | PR #38 | `e08ac57d10ae54c87337eae0be08b1f5e8f4bda4` |
| Design reference | PR #26 | `5b3dbd7ae799f325efbb4f4cc11b2f5518dec948` |
| Common ancestor | Shared history | `cc14dd6beae526884aabdf897e309be837b4f504` |

T00 refreshes these records before implementation.
A changed head requires a new check of affected findings.
A changed source does not invalidate unrelated evidence automatically.
The coordinator must record the exact affected scope.

Include the 20 production library crates.
Include the four demonstration applications and their required journeys.
Include the complete public API inventory, not only W01–W45.
Include the current CI, package, documentation, and visual evidence paths.

Keep all 54 Showcase route identities.
The reviewed target has 30 content paths and 24 placeholders.
Five content paths still use numeric Placeholder exceptions.
T22 removes these exceptions and uses one typed registry.
These counts describe source structure.
They do not prove behavior parity.

Do not add real host cleanup, real database access, or external process effects to a demonstration.
Preserve the current simulation and secret-handling contracts.
Do not add an asynchronous runtime without a named consumer requirement.

## 4. Work streams

The table groups related work.
It does not add barriers beyond each task's predecessor list.
Start a ready task as soon as its files and review capacity are available.

| Stream | Tasks | Order and exit condition |
| --- | --- | --- |
| Cleanup and test architecture | T00, T37, T01, T38 | Capture old evidence. Remove old test code. Inventory cleaned source. Approve test architecture. |
| Gate and reference preparation | T02, T03, T04 | Start after T38. Require task-scoped Sol medium review before every test or harness edit. |
| Dependency structure | T03, T05, T06 | Fix pure builds, then facade features, then enforce edges and lint inheritance. |
| Application ownership | T07, T08 | T08 can follow T06. T07 also needs composition and theme contracts. |
| Shared runtime and API | T09, T10, T11 | Fix response and frame rules. Then work on composition and theme in parallel. |
| Terminal session | T20, T21 | Fix exclusive lifetime and cleanup. Then adapt bounded input scheduling. |
| Component families | T12–T19 | Follow each dependency. Keep shared contracts ahead of family adoption. |
| Missing Showcase pages | T23–T26 | Use four page groups. Split each group into separate file tasks. |
| Showcase integration | T22 | Integrate the accepted page groups through one registry owner. |
| Application completion | T27–T29 | Run independent application workers after their shared dependencies pass. |
| Public consumer proof | T30 | Prove simple use, custom use, and external-host operation. |
| Coverage closure | T31 | Close every required component, route, and capability record. |
| Performance | T33 | Measure the complete candidate on a controlled host. |
| Package use | T34 | Verify every public crate through staged package bytes. |
| CI and current guides | T32, T35 | Integrate final gates and rewrite maintained guidance after their inputs pass. |
| Final acceptance | T36 | Review and verify one final integrated source revision. |

### Dependency handling

Use `deps` in `task-index.json` as the task-start rule.
Do not start a dependent task from an unreviewed predecessor commit.
A worker may inspect a future task before it is ready.
It must not implement against an unstable shared contract.

The graph has 39 task IDs and no cycle.
T22 depends on T23, T24, T25, and T26.
Those page tasks do not depend on T22.
They use separate compilation fixtures until registry integration.
This rule avoids a dependency cycle and uncompiled page evidence.

T32 depends on completed performance and package gates.
T33 and T34 must therefore work through local acceptance first.
They do not depend on T32's final CI integration.
T32 then makes those checks mandatory in CI.

T35 can run beside T32 after its own inputs pass.
T36 closes all 39 tasks and starts after T04, T32, T33, T34, T35, T37, and T38 pass.

### Initial gate failures

The reviewed head has existing failed gates.
T00 captures their logs, source hashes, test inventory, and comment directives before cleanup.
T37 removes their legacy test implementations from the active candidate.
T02 rebuilds required gate evidence through approved functional scenarios.
T06 adds policy enforcement and active lint inheritance.
Neither task may hide a real source defect.

Use one migration record for the remaining initial failures.
For each failure, record its old command, case, source hash, owner task, and expected correction.
Do not use an old test as a current result after T37.
T02 or T06 may finish bounded work with that record.
Mark every gate without approved replacement evidence as unverified.
T32 and T36 require every applicable final gate to pass.

T11 follows the same rule for known application mismatches.
It must pass theme-owner and custom-theme cases.
It records remaining application-owned failures under T22 or T27–T29.
Those failures remain open until the application task verifies the correction.

## 5. Fixed design decisions

Use these decisions during implementation.
A worker must not select a conflicting design.

| Area | Required design | Rejected change |
| --- | --- | --- |
| Crates | Keep 20 production libraries. Give each one a defined responsibility. | A crate for every widget or helper. |
| Facade | Keep foundation APIs. Use one optional feature per family. Default to `components`. | Mandatory session or editor dependencies for a controls-only consumer. |
| Session | Require the explicit `session` feature. Keep terminal-content widgets separate. | A hidden backend through default family dependencies. |
| State | Keep caller data and explicit component state. Borrow visible data. | An implicit global store or clones of entire collections during draw. |
| Response | Add the specified effect merge. Keep typed parent actions and separate Flow/Invalidate. | Manual response replacement that can lose Layout. |
| Composition | Keep existing traits and adapters. Validate nested depth before output. | A second tree language or silent invalid-content fallback. |
| Collections | Reuse `Revision`. Add the specified source-revision builders. Keep explicit invalidation. | Treating equal length and endpoint keys as proof of unchanged data. |
| Customization | Keep theme, scope, part, instance, and slot paths. Share pure patch merging. | Application-specific color repair in reusable components. |
| Button | Restore mnemonic display and borrowed tooltip requests through shared runtime services. | Raw key handling or a controls-to-overlays dependency. |
| Product screens | Move Jackin page models to Jackin UI crates. | Renaming product fields while keeping them in a generic library. |
| Data policy | Keep SQL, catalog, launch, and simulation rules in application owners. | Moving domain behavior into Grid, fields, or feedback. |
| Shell | Use shared focus, copy, quit, leave, hint, and capability rules. | Per-page key and hint whitelists. |
| Session safety | Use one active owner, a live cleanup token, and a fallible finish path. | An ended session that can later restore stdin or emit cleanup output. |
| Input bursts | Keep every event and each required frame barrier. Use a bounded admission policy. | Merging semantic events or fabricating presentation acknowledgement. |
| Examples | Compile complete public source files and prove real actions through approved scenarios. | An unregistered Rust code block used as API proof. |
| Packages | Verify registry consumers from staged package bytes. | Treating a checkout path consumer as publication proof. |

The exact API shapes are in [API-RULES.md](API-RULES.md).
The dependency ceilings and feature table are in [ARCHITECTURE.md](ARCHITECTURE.md).
The functional proof rules are in [TESTING-RULES.md](TESTING-RULES.md).
If a source constraint prevents a selected product design, use product design escalation.
Test architecture and test-only decisions follow [TESTING-RULES.md](TESTING-RULES.md).
Never use any Astra model for any task or scenario.
Give GPT-6.1-Sol high the concrete conflict and smallest proposed correction.
Keep the affected task blocked until GPT-6.1-Sol high records the product design decision.
Do not let a worker invent another architecture.

## 6. Parallel work and file ownership

Use GPT-6.1-Sol low as the coordinator.
Use GPT-6 Luna max for every implementation and correction.
Use independent GPT-6.1-Sol reviewers at each task's stated effort.
Use an independent Sol medium reviewer before each test or harness edit.
Use a separate Sol medium reviewer for the completed test or harness diff.
Use Sol low only for text and link checks after technical acceptance.

The coordinator must dispatch complete child tasks.
Each child gets its source commit, exact files, API rules, required cases, and result schema.
A task title alone is insufficient.

Use separate worktrees for concurrent writers.
Use one integration owner for these shared surfaces:

| Shared surface | Owner task |
| --- | --- |
| Root instruction links | T00 |
| Public requirement inventory | T01, then T31 for accepted evidence |
| Backend dependency declarations | T03 |
| Facade features and export migration | T05; later changes return through its owner |
| Architecture policy and lint inheritance | T06 |
| Core response and runtime frame contracts | T09 |
| Runtime keyboard delivery snapshot | T09; T20 supplies session observations |
| Composition validation and leaf adapters | T10 |
| Theme roles and patch resolution | T11 |
| Runtime tooltip request contract | T12 |
| Generic feedback components | T12; product exports remain with T07 |
| Clipboard outcome contract | T13 |
| Collection revision contract | T14 |
| Overlay layer contract | T15 |
| Shell key and leave policy | T19 |
| Session lifetime and input pump | T20, then T21 |
| Showcase registry and page module declarations | T22 |
| Coverage record closure | T31 |
| CI generator inputs and generated output | T32 |
| Package versions and release order | T34 |

Later tasks can request changes to a shared surface.
The coordinator assigns the request to one Luna writer.
It then requires the corresponding technical review again.
A file path outside the worker's assignment is not implicit permission to edit it.

Reserve capacity for review.
Do not keep all slots occupied by writers while accepted work waits.
When only two worker slots are available, use one implementer and one reviewer.
Do not run heavy timing and visual capture together on the same host.

## 7. Reference and migration control

Keep three evidence groups distinct:

| Group | Meaning | Allowed use |
| --- | --- | --- |
| Historical baseline | Existing immutable tag, committed captures, and old receipts. | Explain prior product intent and detect unintended historical changes. |
| PR #26 reference candidate | New deterministic output from the fixed reference source. | Establish the current source-bound comparison after review. |
| PR #38 candidate | Output from the implementation source under test. | Compare behavior and appearance against the accepted reference. |

The `visual-baseline` tag is annotated and unsigned.
Do not move the tag or replace its bytes.
Keep the pinned tuiscotti revision.

T04 must prepare all review material before a required approval request.
If current authorization already covers the action, use that authorization.
Otherwise, request only the concrete final promotion that remains blocked.
Continue all independent work.
Final visual acceptance remains open until the required reference is accepted.

For each public API change, record:

1. The old public path or behavior.
2. The new public path or behavior.
3. The reason for the change.
4. The affected consumers.
5. The exact migration.
6. The compatibility period, if any.
7. The source and package version.
8. The tests that prove the replacement.

Keep neutral aliases when they do not break ownership.
Do not create a library-to-application dependency to preserve an accidental export.
Do not remove a documented advanced capability without a replacement or a specific accepted retirement.

## 8. Acceptance gates

These gates are independent.
A pass in one gate cannot replace another gate.

| Gate | Required result | Main evidence owners |
| --- | --- | --- |
| G01: Source and inventory | Exact source IDs; complete legacy capture; public, route, and requirement inventories. | T00, T01, T02, T37 |
| G02: Test architecture | Reviewed real-scenario design, independent oracle, failure model, and deterministic environment. | T38 |
| G03: Crates and features | Allowed graph; active lints; isolated leaf/facade builds; clean domain boundaries. | T03, T05–T08 |
| G04: Public API | Stable state contracts; real normal and custom consumers; complete migration records. | T09–T19, T30 |
| G05: Behavior | Required component scenarios, all 54 routes, and all application journeys pass. | T22–T31 |
| G06: Visual proof | Complete accepted case set; exact cells and cursor; semantic checkpoint proof. | T04, T11, T22–T29, T36 |
| G07: Runtime and platform | Correct frame acknowledgement, terminal cleanup, input order, and declared platform support. | T09, T20, T21 |
| G08: Performance | Required work counts and allocation limits pass. Controlled timing meets the declared budget. | T33 |
| G09: Packages | Every public package works from staged registry content without the checkout. | T34 |
| G10: Final integration | Required CI, current guides, review dispositions, and exact-source final report pass. | T32, T35, T36 |

Use [VALIDATION.md](VALIDATION.md) for commands and evidence formats.
Use [COMPONENT-MATRIX.md](COMPONENT-MATRIX.md) for required UI cases.
Record all failed, skipped, absent, and blocked required results.
A zero-case run cannot pass. The empty suite after T37 remains unverified.

Keep approved functional cases and real-CLI visual cases distinct.
Run the complete required visual matrix before final acceptance.
Run the accepted reference comparison twice with separate outputs.
Require a stable case and checkpoint inventory.

Build and test the declared minimum Rust version.
Build with the pinned development version.
Compile platform-neutral components for Windows.
Qualify session behavior only on platforms that the release claims.

## 9. Completion and handoff

T36 fixes one final candidate commit.
Independent Sol high agents review API/dependencies and UI/runtime/evidence.
Luna max workers make each correction.
The coordinator reruns the affected gates after integration.

The final report must include:

- The final source and package hashes.
- The accepted reference and capture identities.
- Each gate result and evidence location.
- The complete task state record.
- The public API migration guide.
- The supported platform and feature matrix.
- The performance measurements and their limits.
- All pull request feedback dispositions.
- Any external action that still requires current user authorization.

Keep the goal open if a required item is failed, pending, or blocked.
Do not describe a prepared plan as completed implementation.
Do not claim a build proves interaction or visual parity.
Do not claim package publication from a dry run.

Use the exact handoff prompt in [GOAL.md](GOAL.md).
Keep all files beside it.
Link the API rules from root `AGENTS.md` through the supplied patch.
