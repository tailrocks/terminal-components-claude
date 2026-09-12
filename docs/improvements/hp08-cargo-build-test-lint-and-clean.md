# HP08 — Cargo build, test, lint and clean

[Tracker](PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Give build/test/lint/clean distinct review/output/result fixtures, exact argv and custom/shared target effects. Tool-native clean must remain visibly permanent in the simulation.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [x] Implement all mapped UI variants and typed simulated outcomes.
- [x] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [x] Inspect keyboard/pointer, size/color and decisive state captures.
- [x] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Actual Cargo process execution, target discovery and cleanup.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve cargo build, test, clippy --all-targets --all-features and cargo clean. Existing generic output and target-path assumption are insufficient.

**Source evidence and mandatory scope:** [matrix HP08](../parity/holla-parity-matrix.md#hp08--cargo-build-test-lint-and-clean) — `OP15`, `OP16`, `OP17`, `OP18`, `OP19`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Expose all four legacy choices as Cargo alternatives with exact flags and cwd. Keep richer check/fmt/run/nextest choices as category B. Provide distinct build/test/lint success/failure output; clean must state tool-native permanent effects rather than Trash. Resolve actual target ownership so the modeled effect matches the command; shared/custom targets must not produce a false deletion claim.

**Architecture / reusable components:** Use Cargo action specification and semantic task effects; cleanup target resolution is shared with artifact observations, not id-prefix inference. Reuse activity output and safety review; no new component.

**Required deterministic fixture:** `parity-cargo` — cwd manifest/tool gates; clean and failed builds; passing/failing tests; all-targets/all-features clippy; target missing/custom/shared; clean refused/cancelled/succeeded.

**Acceptance / automated verification:** Assert all four argv vectors, distinct diagnostic/result states, correct target effect and no mutation before confirmation. Compare expected target ownership with post-run rescan; do not claim freed bytes from command success alone. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture Cargo alternatives, compile/test/lint failure and clean review/result with tool-native recovery wording.

**Dependencies:** HP01/HP07/HP14/HP18/HP21/HP22; F10–F15/F23.

## Evidence

**Slice status:** current simulated slice complete · Later operational clauses open.

**Scenario and journey:** `parity-cargo` (fixture fn `cargo` in src/bin/holla/domain/parity.rs) · `hp08_cargo_results_are_exit_codes_and_clean_lands_its_effect` in src/bin/holla/app_tests_parity.rs · supporting unit tests: `src/bin/holla/domain/outcomes.rs: docker_and_cargo_outcomes_depend_on_state`.

**What the journey asserts:**
- `cargo clippy` starts with argv `cargo clippy --all-targets --all-features @/Users/alex/work/engine`, settles `ActivityState::Succeeded` ("warnings do not fail the run") and its output contains `warning`.
- `cargo test` settles `ActivityState::Failed` with `exit == Some(101)` ("cargo test's own exit code").
- The `cargo.clean` item label contains `900.0 MiB`, its effect is `Effect::CargoClean("/Users/alex/work/engine/target")`, and its summary or effects name the shared target `engine-wt`; `/Users/alex/work/engine/target/debug/engine` exists before any run.
- `cargo clean dry run` has argv `cargo clean --dry-run --verbose @/Users/alex/work/engine`, settles `Succeeded`, and the target file still exists ("a dry run removes nothing").
- `cargo clean` (driven as `open` then `confirm`) runs with argv `cargo clean @/Users/alex/work/engine`, settles `Succeeded`, `/Users/alex/work/engine/target` no longer exists and `world.cargo.target_bytes == 0`. The presence of the dialog is shown by the capture, not asserted.
- Unit (outcomes.rs): `cargo clippy --all-targets --all-features` exits 0 with warnings, `cargo clippy --all-targets -- -D warnings` exits 101; `cargo test` exits 101 with a failure; `cargo clean --dry-run` prints `nothing removed`; `cargo build` outside a manifest directory exits 101 ("no manifest here"); a build then test sequence exits 0 when tests pass and stops at the first `FAILED`.

**Captures:** `shots/h_hp08_clean` (query `cargo clean`: the 900.0 MiB row, dry-run alternative, tool-native permanent wording and the shared `engine-wt` target) · `shots/h_hp08_clean_confirm` (the one-step confirmation dialog for `cargo clean`) · base matrix `shots/h_parity_cargo_{80x24,100x30,120x40,160x50,mono}`. Provenance in each frame's `.manifest.json` (source git revision and dirty flag, binary sha256, arguments, geometry, colour environment, tmux/python/Pillow versions, fonts) and raster fidelity in `.png.fidelity.json`. Visual inspection: recorded by the integrator in the manifest `review` field.

**Row-level representation evidence:**

| Row | Current representation evidence | Deferred clause |
| --- | --- | --- |
| OP15 | Unit: `cargo build` at `/Users/alex` (no manifest) exits 101. The fixed build/test/clippy/clean order and the cargo-executable gate are not asserted; the base matrix frames show the Cargo rows. | Real `Cargo.toml` and cargo PATH probes. |
| OP16 | Unit only: `cargo build` followed by `cargo test` exits 0 when tests pass. The journey does not run `cargo.build`, and no failed build is modelled in the fixture. | Real build execution. |
| OP17 | Journey: `cargo test` settles `Failed` with `exit == Some(101)`; unit: exit 101 with `test_failures = 1`, sequence output contains `FAILED`. | Real test execution. |
| OP18 | Journey: argv `cargo clippy --all-targets --all-features @/Users/alex/work/engine`, `Succeeded`, output contains `warning`; unit: `-- -D warnings` variant exits 101. | Real clippy execution. |
| OP19 | Journey: label `900.0 MiB`, `Effect::CargoClean(".../engine/target")`, shared target `engine-wt` named, dry run argv `cargo clean --dry-run --verbose` leaves the target, `cargo clean` removes `/Users/alex/work/engine/target` and `target_bytes == 0`; frames `h_hp08_clean`, `h_hp08_clean_confirm`. | Real cargo clean and a post-run target rescan. |

**Deferred remainder:**
- Actual Cargo process execution, target discovery and cleanup (Later).
- Not proven by tests: a failed build result state, a missing or custom (`CARGO_TARGET_DIR`) target, clean refused or cancelled, and a compare of expected target ownership with a post-run rescan (the journey checks the world's `target_bytes` and the removed path only).
- No capture of a compile, test or lint failure.

**Limits:**
- Exit codes and output come from the simulated outcome model, not from cargo.
- Frames are being regenerated; their text is indicative until the integrator records the review.
