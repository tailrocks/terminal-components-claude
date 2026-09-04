# Goal — finish the Rust TUI component-system refactor

Continue the in-progress refactor in this repository. Do not restart it, do not re-audit what is already recorded, and do not redesign what is already adjudicated. Everything you need is on disk.

## Read first, in this order

1. `REFACTORING_STATE.md` — the durable ledger: current slice, accepted decisions, file ownership, evidence, unresolved findings, next action. Its "Session 2 addendum" blocks describe the last interruption.
2. `GOAL.md` and `REFACTORING_GOAL.md` — the contract. §29 is the definition of done; §30 is the required final report; §26 is the quality-gate command set; §27 is the eight-slice plan.
3. `COMPONENT_ARCHITECTURE.md` — the accepted architecture (~6,300 lines). §1–§20 are the design; §21–§28 are Adjudications J, K, L, M, N, O and P. **Change control is stated at line 3**: any change to a Decision, an invariant, an exact type or a precedence rule requires a fresh `opus-analyst` adjudication recorded in that document *and* in the ledger.
4. `docs/reviews/` — eight review and adjudication documents. `adjudication-q-residuals.md` is accepted and **not yet fully applied**. `findings-from-documentation.md` holds eight measured defects.
5. `docs/plans/` — `slice6-tablepro.md`, `slice7-jackin.md`, and `PLAN-FINDINGS.md`. The plans are executable; the findings file lists measured discrepancies that block work.
6. `docs/audit/` — the six Slice-1 audits plus the modern-API audit (binding rules R‑1…R‑20 and the forbidden-pattern table in §2 and §6).
7. `docs/guides/` — the five §24 guides. Every code block in them was compiled; treat them as a description of the API that actually exists.

## Mandatory execution model

**Model routing.** The coordinator and every implementation worker run `claude-opus-5` at effort `high`. So does the read-only analyst. All three definitions in `.claude/agents/` already pin this; **never pass a per-invocation model or effort override** — the definitions own routing. This is a recorded, user-authorised deviation from the goal's `claude-fable-5-1` mandate, made because Fable credits were exhausted; it is documented inline in `.claude/agents/refactor-coordinator.md` and must be surfaced in the §30 final report rather than re-flagged as a blocker.

**Use subagents aggressively.** This is not optional and not a style preference — it is how the work gets done at this scale. Concretely:

- Spawn **fresh, read-only `opus-analyst`** agents for *every* exploratory audit, research question, architecture decision, alternative comparison, root-cause diagnosis, public-API critique, test-design review, domain-boundary decision, security analysis, performance interpretation, visual judgement and independent verification. Fresh means fresh: an independent verifier must not inherit the context of the work it is checking.
- Spawn **`fable-builder`** agents for *every* repository mutation: production code, migrations, tests, fixtures, captures, benchmark runs, documentation, cleanup, correction loops and the final report.
- **You coordinate and do not implement.** Your own tool use should be limited to: measuring state, spawning agents, recording their results in `REFACTORING_STATE.md` and `COMPONENT_ARCHITECTURE.md`, committing, pushing, and reporting. If you find yourself editing a source file, you have taken a builder's job.
- **Run many agents at once.** Anything with disjoint file ownership runs in parallel. Five to eight concurrent agents is normal and expected here. Do not serialise work that does not conflict.
- Never use generic, inheriting, built-in Explore or built-in Plan agents for any part of this goal.

**Disjoint ownership is the rule that makes parallelism safe.** Give every builder an explicit, exclusive file list. For genuinely contended files — `crates/tui/src/components/mod.rs`, `crates/tui/src/lib.rs`, `crates/tui/src/author.rs`, `crates/tui/tests/conformance.rs`'s suite list, `xtask/named_tests_allow.txt` — instruct builders to make **minimal single-line insertions in alphabetical position**, re-reading immediately before each edit and retrying on failure. The `Edit` tool fails on a stale match rather than clobbering, so a race surfaces as an error, not as lost work. Never let two agents rewrite a region of the same file.

**Escalation.** If a builder hits an unresolved architectural or public-invariant question, it must stop that item, keep everything else green, and return a precise research request naming file, line and the conflict. It must never silently deviate and never edit a foundation file to work around a missing primitive. You then obtain a fresh Opus adjudication, record it, and resume.

**Commit and push `origin/main` after every recorded result.** Commit only the files whose agent has finished; never capture another agent's half-written state.

## Current state — measure it, do not trust this summary

Two external agents were recently working in `crates/**`, so the tree may have moved. **Step 0 is always to measure**, then report what you actually found:

```
git log --oneline -5 && git status --short
cargo build -p tui-next --all-targets
cargo test  -p tui-next -p tui-next-testing --all-targets --all-features
cargo run   -p xtask -- boundary
cargo run   -p xtask -- doc-check
cargo test  --all-targets          # the legacy root package must stay green at 247
```

As last measured, this was the picture — verify each line before relying on it:

- **Slices 1–3 are complete.** Six audits, a 499-capture before-refactor evidence set, TablePro and Jackin cell-exact digests, a perf baseline (tag `perf/baseline`), the architecture with Adjudications A–P, the foundations crate, and the Slice-2 prototype (7 components, 193 conformance tests, 384 render digests, override/overlay/showcase tests).
- **The library crate is temporarily named `tui-next` / `tui_next`.** It is renamed to `junie-tui` / `junie_tui` in Slice 5 by a single scripted rename. This is deliberate; do not "fix" it early.
- **Slice 4 wave 1 is partially done.** 4B (fields, inputs, textarea, select, choice, chips) and 4G (status, hints, progress, meters, chrome) delivered components; their conformance registrations, render matrix and gate were incomplete. Packages 4A, 4C, 4E, 4D, 4F, 4H, 4I are not started.
- **Known-failing at last measure**: `cargo fmt` on 5 files; ~18 clippy errors in `tui-next` and 2 in `xtask`; `components::tabs::tests::mono_pressed_brackets_the_reserved_pad_cells`; `render_components` 44 passed / 116 failed (matrix mid-bless); `boundary` failing only on `conformance_covers_every_public_component`. `PERF_STRICT` shows wall-clock regressions with **zero** allocation or byte failures, which is expected on a shared runner.

## Open decisions that block work — resolve these with fresh Opus agents, in parallel, early

These are not optional cleanups. Several block packages that are otherwise ready to start.

1. **§29 does not exist.** `COMPONENT_ARCHITECTURE.md` cites "§29 / Adjudication Q" in eight places (lines 9, 45, 1734, 1815, 1869, 3943, 6211, 6222) and the document ends at §28.8. `docs/reviews/adjudication-q-residuals.md` is accepted and must be recorded as §29 with the nine amendments its "Exact document amendments" section lists. Slice 6 depends on §29's `Slot<GlyphRole>` contract.
2. **`RowUi::marker` and `RowUi::part` discard `Resolved.glyph`**, so every mono `MARKER` rule is inert. The fix requires `Resolved.glyph: Slot<GlyphRole>` (`Option` cannot distinguish `Slot::Clear` from unset) — a §11.2 amendment. Adjudication Q's acceptance condition A4 asserts no caller exists; **that is false** (`examples/07_borrowed_rows.rs:31`, `08_dynamic_tabs.rs:30`), so A4 must be re-stated.
3. **F1 — undeclared custom families receive none of the 18 mono fallback rules.** `Recipes::apply_mono_fallbacks` iterates `by_family` only and never reaches `Recipes::neutral`, so at `ColorLevel::Mono` a `Family::custom(…)` gets zero rules. This re-opens §29's "state readable without relying only on colour" on the default path a downstream author takes.
4. **F2 — the architecture's own Scenario G reference fails two conformance cases.** `examples/12_author_component.rs` lacks `.patch_part` and `.state_override`, so it fails `local_override_does_not_mutate_the_theme` and `mono_states_are_distinguishable`. A corrected version passing all 21 tests has already been built and described.
5. **Slice 6 Q1–Q3 block Slice 4 wave 2**, not merely Slice 6: does the library `Grid` own a sort permutation and on what comparison (it sees only `CellRef` and cannot reproduce NULLs-last ordering); what does `GridState` expose (eleven TablePro chords and three migrated tests need the cursor); may a control register a zero-area focus entry (the explorer drawer's focus stop depends on it, and rule R5 forbids registering from a component that cannot draw).
6. **Slice 7 Q2 blocks Jackin entirely**: the accepted `App` trait has no tick hook and no accessor for elapsed virtual time, yet Jackin's clock, both rain state machines, the launch stage machine, `world.jobs` and the status timeout are all tick-driven.
7. **Slice 7 Q1**: `Screen::strip_right` is listed as removed with no named replacement, and `HintLayer.status` cannot carry its priorities and tones.
8. **`xtask bless-guard` does not exist**, though §16.3 requires it in CI as the mechanism enforcing change → capture → classify → bless for every moved baseline. `xtask` dispatches only `doc-check`, `boundary` and `list`.
9. **Four recorded counts are wrong** and fail `every_named_test_exists` until corrected: TablePro has **23** app tests (§16.4 says 21); Jackin has **28** (22 + 6 chrome, §16.4 says 22); the Jackin `Screen` trait has **23** methods (the audit says 20); the capsule frame baseline is **1,080,602** allocations (§16.6 says ≈480,000, so the `< 200` target is a 5,400× reduction).
10. Six further measured defects in `docs/reviews/findings-from-documentation.md`: `PARTS` ordering is load-bearing and undocumented; a `Button` rustdoc claim contradicts the Junie theme; `define_family` with an empty edit silently discards the neutral styling; `run()` never calls `ColorLevel::detect()`, so a `NO_COLOR` terminal gets truecolor; facade gaps; two competing binding-lookup idioms.

## Ordered plan

**Finish Slice 4.** Close wave 1 (4B and 4G's conformance registrations, the render matrix, the gate), then wave 1's remaining packages **4A** buttons/choices/brand-chrome, **4C** lists/trees/props/steps/nav, **4E** containers/scrolling — all three in parallel, disjoint files. Then wave 2: **4D** tabs, **4F** overlays/dialogs/menus/pickers/forms, **4H** code editor and diff, **4I** the generic grid. Appendix A gives the exact file ownership per package. After **each** package lands, spawn a fresh `opus-analyst` to review API consistency against §13, and have a builder apply verified corrections — that cadence is required by §27 Slice 4 and is what keeps the families coherent.

**Slice 5 — showcase.** The scripted `tui-next` → `junie-tui` rename, removal of the root `src/` and its three `[[bin]]`s, `tools/capture.sh`'s `BIN` default, then migrate every showcase page, add custom-theme and local-override coverage, the author-level custom component, complete conformance captures, and remove privileged access to internals. Also merge the two halves of the Buttons page into `apps/showcase` and strike §18.3 #4's deviation paragraph (Adjudication P's P1 obligation).

**Slices 6 and 7 — TablePro and Jackin, in parallel** over disjoint app trees. Execute `docs/plans/slice6-tablepro.md` and `docs/plans/slice7-jackin.md`; they are per-screen, cited to `file:line`, with the regression contract for all 23 TablePro tests, 42 TablePro digests, 28 Jackin tests and 36 Jackin digests. Slice 7's highest risk is stated explicitly: Jackin's virtual clock advances by the **route's** interval (`Route::tick_ms`), and re-basing it on a uniform token or wall-clock delta breaks ~40 tick counts, every fixture timestamp and the outro caption at once.

**Slice 8 — cleanup and independent verification.** Remove every legacy path and dead module, tighten visibility, complete `README.md`, `DESIGN.md` and the migration mapping, regenerate only reviewed baselines, run the full gate set, then run a fresh read-only `opus-analyst` **architecture** review and a *separate* fresh read-only `opus-analyst` **visual** review, and have builders correct every verified issue. Finally produce the §30 report with all fifteen required items.

## Quality gates

Every command must exit 0 before a slice is called done:

```
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test  --workspace --all-targets --all-features
cargo test  --workspace --doc --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
cargo build --workspace --all-targets --all-features
cargo build -p tui-next --examples
cargo test  -p tui-next --test render --test render_components
cargo test  --workspace --test perf --release -- --test-threads=1
cargo check -p tui-next --no-default-features
cargo +1.88.0 check --workspace --all-targets --all-features
cargo run   -p xtask -- doc-check
cargo run   -p xtask -- boundary
cargo test  --all-targets
```

`boundary` must report every check ok with `crates/tui/tests/allow/legacy_api.txt` and `allow/domain.txt` both empty. **Name both render targets** — a gate naming only `--test render` silently runs half the matrix (§28 P2). Any moved baseline needs a `docs/visual-changes.md` entry classified against a numbered §20.10 item **before** blessing; that order is §16.3's contract. `.github/workflows/ci.yml` already encodes this set.

## Rules

Follow the modern-API rules R‑1…R‑20 literally. `#[expect(lint, reason = "…")]`, never `#[allow]`. No `todo!`, `unimplemented!`, stubs or placeholders. `#![deny(missing_docs)]` satisfied. No `unsafe` in `crates/tui`. No literal palette colours outside `theme/builtin`. No raw `bg: Color` parameters. No TablePro or Jackin vocabulary in the library. Preserve edition 2024 and MSRV 1.88 — the MSRV is now a *verified* fact, not a declared field, and the CI leg is pinned to exactly `1.88.0`.

## Evidence discipline — this is how the goal is judged

Report at every turn end: completed evidence, unresolved criteria, active blockers. **Do not claim completion for anything you did not measure yourself.** Re-run a builder's gate rather than repeating its summary; agents have reported "green" on trees that were not. When a measurement contradicts something recorded earlier — including something you wrote — say so plainly and correct the record. Several of the most valuable findings in this refactor came from an agent checking a premise it was handed and discovering it was false; treat that as the standard, not the exception.

Keep `REFACTORING_STATE.md` current enough that a cold session could resume from it alone: current slice, model assignments, accepted decisions with their adjudication references, explicit file ownership, completed gates with their exact output, pre-existing failures kept separate from regressions, unresolved findings, and the next action.

The goal is complete only when every item in §29 is **proven**, the §26 gates exit 0 after final corrections, a fresh Opus architecture review and a separate fresh Opus visual review find no unresolved material issue, `REFACTORING_STATE.md` reports no remaining work, and the §30 final report contains all fifteen required items.
