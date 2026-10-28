# Testing and source-comment rules

## Purpose and precedence

These rules govern test planning, test evidence, test cleanup, and source comments.
They apply to every crate, application, tool, fixture, harness, and acceptance gate.

These rules replace conflicting test and comment instructions in older plan files.
They do not remove unrelated API, safety, architecture, model, or acceptance requirements.
Keep historical findings, but update claims that refer to deleted tests or comments.

Tests are required after their plans pass review.
The initial test cleanup does not create a permanent test ban.
The continuing comment rule forbids nonfunctional source comments, not tool-required text.

## Required task sequence

1. Complete T00 source and branch audit before source changes.
2. Complete T37 cleanup after T00 and before T01.
3. Complete T01 requirement and API inventory after T37.
4. Complete T38 test-architecture review after T01.
5. Keep T02, T03, and T04 blocked until T38 passes independent review.

T37 removes every old test function, test module, fixture, harness, and doctest from the candidate.
It covers Rust crates, application suites, tools, scripts, and other test languages.
It does not remove immutable source references, approved visual artifacts, production diagnostics, or contracts.
It leaves no active test file, fixture helper, CI filter, inventory row, or citation to deleted test code.
Keep immutable source references and historical evidence outside the active test inventory.
This cleanup does not authorize unrelated API removal or production behavior changes.

Do not add replacement tests during T37.
Remove test-only dependencies, targets, fixtures, and dead inputs with their tests.
Update CI, generators, filters, manifests, test citations, and documentation that names deleted tests.
Regenerate managed output through its declared generator.
Keep unrelated production behavior and diagnostic capabilities.

The empty suite is a transition state after T37.
Record every test lane with zero selected cases as `NOT VERIFIED`.
Do not count a successful command, skipped lane, or empty suite as test evidence.
Do not close final acceptance while a required suite remains empty.

## Architecture review before tests

T38 must define the test architecture before any test code returns.
One `gpt-6.1-sol` agent with medium effort must design T38.
A different `gpt-6.1-sol` agent with medium effort must review and approve T38.
Luna records the approved design in the maintained plan documents.
The plan must identify requirements, public behavior, test seams, external dependencies, and proof limits.
It must separate production acceptance from optional algorithm tests.

Before each later task writes or refactors a test, prepare a task-level test plan.
An independent `gpt-6.1-sol` reviewer with medium effort must approve the plan before the first test write.
Do not write a test while its plan or required architecture decision is pending.

Every plan must include:

- The requirement ID and the source contract that defines it.
- The production path and public API used by the test.
- Realistic positive, negative, and boundary scenarios.
- The expected user-visible result for each scenario.
- An oracle that does not repeat the implementation logic.
- The real dependencies, test doubles, and exact limits of each double.
- The test command, selected case IDs, execution environment, and failure artifacts.
- A known-buggy source revision or a realistic fault for red-green proof.
- The evidence needed to distinguish a product failure from an infrastructure failure.

Plan before adding or refactoring a regression test for an escaped issue.
First explain why current tests missed the issue.
Then ask a separate `gpt-6.1-sol` medium architect agent to review the testability gap.
Record the testability finding before you write the replacement test.
Use high effort only if the finding requires a product architecture decision.
Do not append a test without this analysis.

## What tests must prove

Write tests to prove required functionality works and keeps working.
Do not write tests for a coverage number, test count, or other KPI.
Do not judge a suite by its test count, coverage percentage, or speed.
Use requirement and case inventories to find missing behavior, not as quality scores.

Every acceptance test must exercise a real production path through a public API.
Assert the resulting state, output, or user-visible action.
Do not assert internal call order or copied implementation logic.
Do not test a library capability only through an algorithm helper when a consumer can observe it.
Use a pure algorithm test only for a named gap that the reviewed plan approves.
Keep such a test supplemental to production-path evidence.

Use the real implementation of a dependency when it is practical and deterministic.
Do not mock, fake, or stub code owned by this repository.
Use a test double only at an unowned external I/O boundary behind an interface this repository owns.
The plan must justify each double and state what real behavior it preserves or omits.
Use seeded data, temporary directories, fake clocks, or local services to control external state.
Do not depend on uncontrolled live network services.

Use the smallest test that gives useful evidence.
Keep tests runnable after every change and run the narrow cases first.
Run the complete relevant suite before task acceptance.
Use `cargo nextest` for Rust test targets.
Record exact selected test names and case IDs.

## Red-green and verification

Each kept test must fail on broken behavior and pass on the fixed behavior.
Run the test against a known-buggy parent revision or inject a realistic fault at a real boundary.
The red result must fail at the intended product assertion.
A compiler, setup, missing-fixture, or harness error is not a red result.
Restore the clean candidate source before the green run.
Run the same case against the fixed candidate and record both results.

An independent agent must verify each new test before it enters the required suite.
The verifier must inspect the plan, test code, production path, oracle, and red-green evidence.
The verifier must record exact source SHA, environment, case IDs, command, logs, and result.
Do not accept the implementer's own review as independent verification.

A flaky case blocks acceptance until its cause is fixed or its requirement is explicitly dispositioned.
You may isolate a flake for diagnosis, but it is not a pass.
Do not silently skip, weaken, edit, or delete a test to make the suite green.
Classify infrastructure failures separately from product assertion failures.
Keep every selected test result tied to the exact tested source SHA.

A green result proves only the declared cases in the recorded environment.
It does not prove every input or every environment works.
Do not claim universal correctness from a passing suite.

## Source comments

Do not keep explanatory, TODO, or `SAFETY` source comments.
Rewrite the code when names or structure can express the rule clearly.
Put essential safety and invariant rationale in maintained contracts and design records.
Maintain caller guidance in project documentation and complete consumer examples.

Keep only comments or text that tools or users require for function:

- Keep SPDX license lines required by REUSE.
- Keep shebangs and parser directives.
- Keep linter directives that control a required check, such as ShellCheck source directives.
- Keep generated comments only when the declared generator verifies the output byte-for-byte.
- Keep command help as explicit Clap `#[command]` and `#[arg]` attributes.

Replace Clap doc comments with explicit `about`, `help`, or equivalent attributes.
Do not remove a generated header or other required bytes by hand.
Do not weaken unsafe-code lints to remove a required safety explanation.
If a mandatory lint conflicts with this policy, stop and request a scoped Sol high decision.
Do not use a blanket lint allowance for that conflict.

## Cleanup and evidence limits

Before you delete test-only code, check whether a feature also serves users, diagnostics, or production tools.
Delete a dependency only when no production or supported diagnostic path uses it.
Update inventories and generated workflows when a test target disappears.
Do not let a zero-test CI job report success as test evidence.

Keep approved visual references immutable during test implementation.
A candidate capture must use a separate output directory.
Never change expected bytes to hide a product failure.
Bind each visual result to source, cases, inputs, and environment.

Report limits with evidence.
Keep unrun tests, missing environments, zero-case commands, and unresolved flakes visible.
