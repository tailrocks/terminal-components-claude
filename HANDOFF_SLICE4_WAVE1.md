# Goal — Finish Slice 4 wave 1 of the Rust TUI component-system refactor

Continue the in-progress refactor in this repository. Do not restart, do not re-audit, do not redesign.

## Read first, in this order

1. `REFACTORING_STATE.md` — the durable ledger. Its "Session 2 addendum" blocks describe exactly where the previous session stopped.
2. `COMPONENT_ARCHITECTURE.md` — the accepted architecture. Adjudications A–P are recorded in §21–§28. Change control is stated at line 3: any change to a Decision, invariant, exact type or precedence rule requires fresh `opus-analyst` adjudication recorded in the document and the ledger.
3. `docs/reviews/adjudication-q-residuals.md` — accepted, not yet applied.
4. `docs/audit/modern-api-audit.md` §2 and §6 — the binding API rules R‑1…R‑20 and the forbidden-pattern table.

## Mandatory execution model

- Every audit, research question, architecture decision, alternative comparison, root-cause diagnosis, public-API or test-design critique, domain-boundary decision, security analysis, performance interpretation, visual judgement and independent verification goes to a **fresh, read-only `opus-analyst`** subagent.
- Every implementation, command run, capture, test and documentation edit goes to a **`fable-builder`** subagent.
- You coordinate: spawn agents, record their results in `REFACTORING_STATE.md`, commit, push, report. Do not implement directly.
- Never use generic, inheriting, built-in Explore or built-in Plan agents. Never pass a per-invocation model or effort override — the definitions in `.claude/agents/` own routing (all three are `claude-opus-5`, effort `high`; that is a recorded, user-authorised deviation from the goal's Fable mandate because Fable credits were exhausted).
- Parallel builders must have explicit, disjoint file ownership. For the contended files `crates/tui/src/components/mod.rs`, `crates/tui/src/lib.rs`, `crates/tui/src/author.rs` and `xtask/named_tests_allow.txt`, instruct builders to make minimal single-line insertions in alphabetical position, re-reading immediately before each edit and retrying on failure.
- Commit and push to `origin/main` after every recorded result.

## Scope — finish Slice 4 wave 1, and nothing else

1. **Clear the build.** Run `cargo build -p junie-tui --all-targets` and `cargo test -p junie-tui --lib` and fix what they actually report. Two prior reports disagreed (one `E0502` versus 17–18 errors), so measure rather than trusting either. The one confirmed item is `crates/tui/src/components/choice.rs:1229`, where `st` is borrowed mutably and immutably in the same call; hoist `let i = st.cursor_index();` before the call.
2. **Finish work packages 4B and 4G**, which delivered components but not their proofs. Missing: the `lib.rs` facade line for 4G's components; conformance registrations in `conformance_suite!` for `TextArea`, `Select`, `RadioGroup`, `Checkbox`, `Toggle`, `ChipBar`, `StatusBar`, `HintBar`, `KeyHint`, `ProgressBar`, `Spinner`, `Meter`, `Empty`, `Brand`, with correct `Caps` and all 20 cases passing; the four hard-coded case lists in `conformance.rs`; the `render_components.rs` digest matrix for each new component across `{junie, paper} × {truecolor, mono} × {120×40, 40×10}`; blessing those digests; and the deletion of any `xtask/named_tests_allow.txt` entry that becomes satisfied (the check fails if a satisfied entry remains).
3. **Apply Adjudication Q** (`docs/reviews/adjudication-q-residuals.md`): Q1 a shared bracket helper taking two reserved cells, and fix `Button`'s in-run bracket which can truncate a full-width label; Q2 make `Fixture::state_override` and `Fixture::status` private with `forced()`/`status()` accessors so `force` is the only writer; Q3 add `Conformance::mono_narrowing_reason()` with the case-9 check and write the roughly eight missing reasons. Record it as §29 in `COMPONENT_ARCHITECTURE.md` with the nine amendments its "Exact document amendments" section lists. **Confirm its R1 first**: disable the `Tabs` bracket block and check that conformance case 9 is still red — if it is green, `CONTAINER`'s `BOLD` already distinguished the states and the recorded reason is wrong.
4. **Fix the live `RowUi` glyph defect.** `RowUi::marker` and `RowUi::part` discard `Resolved.glyph`, so every mono `MARKER` rule is inert. `Resolved.glyph` must become `Slot<GlyphRole>` — `Option` cannot distinguish `Slot::Clear` from unset — which is a §11.2 amendment. Note that Adjudication Q's acceptance condition A4 asserts no caller exists; that is false, `examples/07_borrowed_rows.rs:31` and `examples/08_dynamic_tabs.rs:30` already call it, so A4 must be re-stated.

Three questions were returned by the builders and need a fresh `opus-analyst` before the affected work closes:
- `Caps::OVERLAY` conflates "opens a layer" with "traps focus": case 14 asserts the focus ring shrinks and Tab wraps inside, but §9.1 makes a `Popover` a pointer barrier with no focus scope and `Select` keeps focus while open, so `SelectCase` cannot declare `OVERLAY` without asserting a property the layer kind forbids.
- `FieldControl` has no item channel: §15 says implement it for `Select` and `RadioGroup`, but §24 M3 moved items to the per-phase channel and `draw(&self, ui, area, st)` cannot carry `&[T]`.
- `RadioGroup` needed a `.value(ItemKey)` draw-phase controlled prop that §17.0 A7 does not declare, and `ChipBar`'s add affordance emits `Activated(k)` because `ChipBarAction` has no `Added` variant.
- A stateless `StatusBar` cannot paint per-item hover, which the legacy widget did: the frame snapshot carries only `hover: Option<Id>` while the runtime holds `(Id, PartRef)`. This is a live visual regression until `FrameRead::hovered_part` exists.

## Out of scope — do not start

Work packages 4A, 4C, 4E, 4D, 4F, 4H, 4I; Slices 5–8; the `junie-tui` → `junie-tui` rename; any migration of `showcase`, `tablepro` or `jackin-preview`; any edit under the legacy `src/` tree. The legacy root package must keep building and its 247 tests must keep passing.

## Gate — every command must exit 0 before you claim the slice is done

```
cargo fmt --all --check
cargo clippy -p junie-tui -p junie-tui-testing --all-targets --all-features -- -D warnings
cargo test -p junie-tui -p junie-tui-testing --all-targets --all-features
cargo test -p junie-tui --doc
RUSTDOCFLAGS="-D warnings" cargo doc -p junie-tui --all-features --no-deps
cargo build -p junie-tui --examples
cargo test -p junie-tui --test render --test render_components
cargo test -p junie-tui --test perf --release -- --test-threads=1
cargo run -p xtask -- doc-check
cargo run -p xtask -- boundary
cargo test --all-targets
```

`xtask boundary` must report every check ok with `crates/tui/tests/allow/legacy_api.txt` and `allow/domain.txt` both empty. Blessing is now thread-count independent, so `--test-threads=1` is no longer required for it; any baseline you move needs a `docs/visual-changes.md` entry classified against §20.10 **before** you bless, per §16.3's change → capture → classify → bless order.

## Rules

Follow R‑1…R‑20 literally. `#[expect(lint, reason = "…")]`, never `#[allow]`. No `todo!`, `unimplemented!`, stubs or placeholders. `#![deny(missing_docs)]` satisfied. No `unsafe` in `crates/tui`. No literal palette colours outside `theme/builtin`. No raw `bg: Color` parameters. No TablePro or Jackin vocabulary in the library.

If a builder hits an unresolved architectural or public-invariant question, it must stop that item, keep everything else green, and return a precise research request naming file, line and the conflict — never silently deviate and never edit a foundation file to work around it.

## Done condition, and what to leave behind for verification

Slice 4 wave 1 is done when every gate command above exits 0, all 20 conformance cases pass for every registered component, the render matrix covers every new component, Adjudication Q is applied and recorded as §29, and the `RowUi` glyph defect is fixed.

Leave the evidence in a form the next session can check without re-running everything: record in `REFACTORING_STATE.md` the exact command output (test counts per target and exit codes), every decision taken with its adjudication reference, every deviation from the letter of an accepted decision with its reason, and every open question. State plainly anything you could not finish. Do not claim completion for anything you did not measure.
