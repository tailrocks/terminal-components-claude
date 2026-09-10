# HP08 — Cargo build, test, lint and clean

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Give build/test/lint/clean distinct review/output/result fixtures, exact argv and custom/shared target effects. Tool-native clean must remain visibly permanent in the simulation.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [ ] Implement all mapped UI variants and typed simulated outcomes.
- [ ] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [ ] Inspect keyboard/pointer, size/color and decisive state captures.
- [ ] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Actual Cargo process execution, target discovery and cleanup.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve cargo build, test, clippy --all-targets --all-features and cargo clean. Existing generic output and target-path assumption are insufficient.

**Source evidence and mandatory scope:** [matrix HP08](../holla-parity-matrix.md#hp08--cargo-build-test-lint-and-clean) — `OP15`, `OP16`, `OP17`, `OP18`, `OP19`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Expose all four legacy choices as Cargo alternatives with exact flags and cwd. Keep richer check/fmt/run/nextest choices as category B. Provide distinct build/test/lint success/failure output; clean must state tool-native permanent effects rather than Trash. Resolve actual target ownership so the modeled effect matches the command; shared/custom targets must not produce a false deletion claim.

**Architecture / reusable components:** Use Cargo action specification and semantic task effects; cleanup target resolution is shared with artifact observations, not id-prefix inference. Reuse activity output and safety review; no new component.

**Required deterministic fixture:** `parity-cargo` — cwd manifest/tool gates; clean and failed builds; passing/failing tests; all-targets/all-features clippy; target missing/custom/shared; clean refused/cancelled/succeeded.

**Acceptance / automated verification:** Assert all four argv vectors, distinct diagnostic/result states, correct target effect and no mutation before confirmation. Compare expected target ownership with post-run rescan; do not claim freed bytes from command success alone. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture Cargo alternatives, compile/test/lint failure and clean review/result with tool-native recovery wording.

**Dependencies:** HP01/HP07/HP14/HP18/HP21/HP22; F10–F15/F23.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
