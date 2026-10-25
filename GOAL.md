/goal Finish the Termrock crate refactor in pull request 38

## Objective

Finish the reusable Termrock library in `tailrocks/terminal-components-claude`.
Use pull request 38, branch `parity/termrock-visual-baseline`, for the final result.
Use pull request 26, branch `design/interaction-polish`, for appearance and interaction.

Build a library of reusable components, widgets, blocks, and support functions.
Give each crate one clear responsibility.
Give users useful defaults and direct ways to change behavior and appearance.
Keep advanced capabilities that have a valid consumer and approved functional evidence.

Use the existing runtime and component contracts as the starting point.
Correct poor boundaries and repeated behavior.
Replace a design only when a named defect requires replacement.
Do not create a second component framework.

This workspace contains the plan package. Read its `README.md` and local plan files before target work.
Follow `README.md` to create and verify a separate target checkout before any source task.
The target path `docs/implementation/pr38-final/` may not exist before T00 imports this package.
Do not run target source tasks in this plan-package checkout.
Start with `README.md`, `REVIEW.md`, `ARCHITECTURE.md`, `API-RULES.md`, and `TESTING-RULES.md`.
Then execute `PLAN.md` and every task in `TASKS.md`.
Use `COMPONENT-MATRIX.md` and `VALIDATION.md` as completion requirements.

## Source control

The planning review used these commits:

- Pull request 38: `e08ac57d10ae54c87337eae0be08b1f5e8f4bda4`.
- Pull request 26: `5b3dbd7ae799f325efbb4f4cc11b2f5518dec948`.
- Common ancestor: `cc14dd6beae526884aabdf897e309be837b4f504`.

Execute T00 as a read-only source capture, except for importing the approved instruction rules.
Execute T37 as the first Termrock source change.
Complete T01 after T37. Complete T38 after T01 and before implementation work.
Fetch both current pull request heads.
Record the new IDs and their differences from these IDs.
Recheck every affected finding.
Keep the old review as historical evidence.
Do not overwrite current user work.

Use one integration branch for pull request 38.
Use separate worker worktrees when tasks can run at the same time.
Do not merge pull request 26 as a whole.
Port selected behavior with approved functional probes and a source record.
Remove the legacy test suites, harnesses, fixtures, doctests, and nonfunctional code comments before implementation.
Do not write replacement tests or harnesses before T38 and task-scoped independent design review.
Preserve historical source, tags, immutable references, and approved oracle evidence outside the active suite.

## Prohibited model

Never use any Astra model under any circumstances or in any scenario.
This includes `gpt-6-astra`, all Astra variants, fallbacks, reviews, planning, and design escalation.
This prohibition applies to the coordinator, every worker, and all nested agents.

## Mandatory models

Use GPT-6.1-Sol with low effort as the coordinator.
Use GPT-6 Luna with max effort for every implementation task.
This includes Rust, tests, scripts, configuration, and documentation changes.

Use GPT-6.1-Sol for independent implementation reviews.
Use high effort for API, runtime, dependency, safety, performance, and product acceptance reviews.
Use medium effort for bounded component, page, and application reviews.
Use medium effort for test architecture, task test plans, test-only reviews and evidence verification, and testability-gap analysis.
Use low effort for coordination and final text or link checks with no behavior decision.

The coordinator must not replace a Luna implementation with its own code.
Return each required correction to a Luna worker.
Use a different agent for the review.
The worker must not approve its own work.

Use these model IDs when the environment exposes them:

- Implementer: `gpt-6-luna`, `reasoning_effort="max"`.
- Coordinator: `gpt-6.1-sol`, `reasoning_effort="low"`.
- Reviewer: `gpt-6.1-sol` at the effort assigned to the task.
- Product design escalation: `gpt-6.1-sol`, `reasoning_effort="high"`.

Use GPT-6.1-Sol medium for T38 and every task-scoped test architecture review.
Use GPT-6 Luna max for every test implementation and test documentation change.

If the required model is unavailable, record the affected task as blocked.
Continue independent tasks whose required models are available.
Do not silently select another model.

Do not let implementation workers invent a replacement architecture.
If the approved plan cannot meet a required behavior, collect source evidence.
Use GPT-6.1-Sol high to resolve a required product architecture or product contract change.
Keep the affected task blocked until GPT-6.1-Sol high records the product design decision.
Keep unrelated work active.

## Delegation

Use subagents throughout the work.
Start all ready tasks that have separate file ownership.
Reserve capacity for independent review.
Use the available worker slots without assigning the same file to two writers.

Give each worker these items:

1. The complete task text.
2. The fixed input commit.
3. The files that the worker owns.
4. The required API rules and source findings.
5. The exact test cases and completion gates.
6. The required model and effort.
7. The output record format.

Record the approved test-plan ID and independent reviewer in each test-writing task prompt and result.
T38 approves the test architecture only. It does not approve every later test change.

Require a small commit for each verified change.
Review that exact commit.
After integration, rerun tests for the affected dependency set.
A review of an old commit does not approve later changes.

## Preserve the useful design

Keep typed identity, caller-owned state, typed actions, and separate event flow.
Keep separate paint and layout invalidation.
Keep stable collection keys through sort, filter, and data changes.
Keep grapheme-safe text editing and secret redaction.
Keep the prepared-frame and presentation acknowledgement protocol.
Keep modal focus barriers and current frame input routing.
Keep deterministic time and the earliest repaint deadline.
Keep scoped style overrides and `Slot::Inherit`, `Slot::Set`, and `Slot::Clear`.
Keep full, reduced, and paused motion.
Keep TrueColor, 256-color, 16-color, and monochrome behavior.

Use the target crate map in `ARCHITECTURE.md`.
Remove application screen assembly from shared library crates.
Keep application data, SQL behavior, route policy, and simulations in application crates.
Keep the four applications as independent consumers.

Give common components a small constructor and useful defaults.
Use named options for choices that affect behavior.
Keep advanced author functions available through `termrock::author`.
Use real consumer examples to prove each extension point.
Do not require an application to copy a widget's event or focus code.

## Visual and behavior reference

Treat the exact pull request 26 source as the design reference.
Treat the immutable `visual-baseline` tag as historical evidence.
Do not call copied images proof of current output.
Do not change expected bytes to make a failed comparison pass.

T04 must create a source-bound reference plan.
Capture new reference output into a separate candidate directory.
Review the source, cases, frames, cursor, colors, and behavior before acceptance.
Keep existing approved files unchanged until the required approval exists.

Compare the reference and candidate under the same inputs.
Use real CLI processes for application visual acceptance.
Compare every cell, attribute, cursor field, and named checkpoint.
Also prove behavior through approved semantic assertions.
A matching image does not prove keyboard, pointer, copy, or validation behavior.

Preserve all 54 Showcase routes.
Replace each remaining placeholder with a live shared-component example.
Preserve the four applications' required journeys.
Keep domain safety and simulation limits.

## Work order

Follow the dependency graph in `PLAN.md` and `task-index.json`.
First run T00 and capture all source and legacy test evidence without source changes.
Run T37 as the first Termrock source change. Remove all legacy test code and nonfunctional code comments.
Then run T01 and T38. Start implementation only after the reviewed test architecture passes.
Require an independent Sol medium review of each task-specific test design before test edits.
Then fix feature isolation and crate boundaries.
Then complete shared contracts and component behavior.
Then complete pages and application consumers.
Then run performance, package, visual, and full acceptance gates.

Do not postpone a known defect because it appears small.
Do not rewrite a working subsystem only to reduce line count.
Use one owner for each behavior.
Remove a duplicate only after the replacement preserves its required behavior.

## Evidence

For each task, record the input and output commit IDs.
Record changed paths, commands, selected test count, result, and reviewer decision.
Keep a failed test visible until its cause has a verified correction.
A successful command with zero selected tests is not a pass.
A test name or source comment is not a test result.
Treat tests as evidence. Do not use test count as a success measure.
Use integration-first scenarios through production entry points and public APIs.
Do not mock Termrock-owned code. Use an external double only with reviewed reason and fidelity.
Red means a deterministic runtime assertion fails on the baseline.
Green means the same assertion passes after the correction.
A compiler or setup failure is not red evidence.
Report green only for declared cases, source, tools, and environment.
Do not claim absolute correctness from a finite test set.

Check the final resolved dependency graph.
Build production libraries without any application source.
Test minimal, default, selected-feature, and full-feature configurations separately.
Test a detached consumer through the facade.
Test selected leaf crates directly.
Test unpacked package content through an isolated registry fixture.

Use the checked-out toolchain and pinned tools.
Use `cargo nextest` for approved functional scenarios.
Prefer `integration/functional/`; use an existing owner seam when the source needs it.
Use GPT-6 Luna max for every implementation. Use GPT-6.1-Sol medium for test architecture.
If a functional case exposes a product issue, stop test edits. Ask Sol medium to review the testability gap.
Use Sol high only when the evidence requires a product architecture decision.
Respect generated CI ownership.
Change generator inputs and regenerate managed output.
Do not patch generated workflows by hand.

## Required result

Finish all 39 required tasks in `TASKS.md`.
Link the API rules from root `AGENTS.md`.
Update current project documents to match the verified result.
Provide migration instructions and complete executable examples.
Provide one final source-bound acceptance report.

The report must cover these independent results:

- Crate and dependency correctness.
- Public API usability and extension behavior.
- Component and application behavior.
- Visual equality or an explicitly accepted design difference.
- Terminal lifecycle and supported platform behavior.
- Performance and bounded work.
- Package use outside the repository.
- Required CI and review status.

Keep the goal open while a required result is blocked or failed.
An empty test suite is an unverified migration state, never a passing result.
Green results apply only to declared cases and environments.
Do not claim completion from a build alone.
Do not publish a package, merge a pull request, or send a message without current user authorization.

## Language

Write all new prose in ASD-STE100 Simplified Technical English.
Use the technical word list in `SOURCES.md`.
Keep code identifiers and commands exact.
Use one instruction per sentence.
Use no more than 20 words in a procedural sentence.
Use no more than 25 words in a descriptive sentence.
