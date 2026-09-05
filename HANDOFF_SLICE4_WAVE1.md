# Goal — Finish Slice 4 wave 1 of the Rust TUI component-system refactor

## Latest stop update — 2026-09-05

The user stopped the continuation again. Resume from pushed `main` at
`d7255b0`; all tracked chrome changes below are preserved in the next stop
commit. Do not claim Slice 4 or the merge goal complete.

Pushed proof commits now on `origin/main`:

- `ac2637a` — defers the unresolved Choice geometry proof in the named-test
  allow list; this is intentional and remains an open proof obligation.
- `d7255b0` — aligns ChipBar/TextArea render fixtures.

Measured by the proof worker:

- actual package is `junie-tui`; `tui-next` is invalid;
- `cargo build -p junie-tui --all-targets`: pass;
- `cargo test -p junie-tui --lib`: 741 passed;
- conformance: 934 passed;
- named-test boundary: pass;
- render matrix: 326 passed, with six pre-existing Grid digest failures;
- formatting and diff checks: pass.

Tracked but unverified at this stop:

- `apps/jackin-preview/src/app.rs`;
- `crates/tui/src/components/menu.rs`.

These contain an in-progress chrome attempt: top-level menu switching,
expanded Capsule menu/palette entries, context-menu lifecycle state, prefix
routing, last-row generic hints, and `Ctrl+B` display. The chrome suite was
previously `0/6`; no post-edit chrome result exists because the user stopped
the worker. Review or revert selectively after running the focused suite; do
not overwrite the files wholesale.

The architecture audit resolved the four old open questions: no new
amendment is needed; `FieldControl` stays unwidened, Select/RadioGroup/ChipBar
use per-phase data, and RowUi/Q are already applied. Independent visual
blessing remains unauthorized: app capture provenance is stale, TablePro and
Jackin visual evidence remain unresolved, and six Grid digest failures remain.

Only build-cache directories are untracked. Never stage them with `git add -A`.

## Latest session handoff — 2026-09-05 stop

The user stopped the merge/implementation session. Resume from pushed `main` at
`6584b66`; the Capsule chrome implementation is preserved in `c9b765b`.
Do not treat the chrome work below as complete: it is committed for preservation
but its focused suite is red.

The tracked tree is clean after the handoff commit. The preserved implementation
includes:

- `apps/jackin-preview/src/app.rs`: Capsule shell chrome, menu/context/palette
  state, status/hint rendering, inspect/rename/help behavior.
- `crates/tui/src/components/menu.rs`: public `MenuBar::open_menu` helper.
- This handoff update.

Observed focused result before handoff:

```text
app_tests_chrome: 0 passed; 6 failed
```

Known failure roots from the run:

- `MenuBar` open-dropdown `Right` does not switch top-level menus yet.
- Generic shell `HintBar` is not painted on the physical last row; Capsule
  renders `Ctrl+b` while the acceptance text expects `Ctrl+B`.
- Capsule command palette has only four rows, so wheel scrolling cannot move
  its viewport at the tested height.
- Inspect/context-menu assertions remain blocked behind the menu/hint dispatch
  failures.

The latest chrome implementation compiled and `git diff --check` was clean.
Previous pushed functional evidence at `f01703a`: Jackin app tests passed
27/27, targeted exit/active-identity tests passed, and workspace formatting
passed. Re-run both focused suites after fixing chrome; do not infer that the
current dirty-state tests still pass.

Untracked `.codex-target-*` directories are build caches. They are intentionally
not committed. Preserve or move them to `/private/tmp`; never stage with
`git add -A`.

At the stop point, `main` and `origin/main` both point to `6584b66`. Registered
non-root worktrees remain for later cleanup:

- `/private/tmp/terminal-components-before-e4` — stale ancestor `4daa524`.
- `/private/tmp/terminal-components-launch-scope` — `codex/launch-scope-fix`
  at `f01703a`; no unique implementation was found.
- `/private/tmp/tui-conformance-repair.NuX6gv` — stale ancestor `d053258`.

Do not remove them until the next session has rechecked that no uncommitted
work is present and has recovered anything useful.

Fast restart order:

1. Fix `MenuBar` open-state Left/Right switching in
   `crates/tui/src/components/menu.rs`.
2. Fix generic/Capsule hint placement and display case.
3. Give the command palette enough semantic items to exercise wheel offset;
   map only real actions, and retain `New tab` as the first selected item.
4. Run:

   ```text
   CARGO_TARGET_DIR=/tmp/tc-final-chrome rtk proxy cargo test -p jackin-preview --test app_tests_chrome -- --nocapture
   CARGO_TARGET_DIR=/tmp/tc-final-functional rtk proxy cargo test -p jackin-preview --test app_tests -- --nocapture
   ```

5. Fix workspace launch scope separately: selected non-first workspace and
   CurrentDirectory still lose identity before `materialize_launch`.
6. Run full release gates, then prune only stale registered worktrees. Confirm
   `HEAD == origin/main`, clean tracked status, one root worktree, and no stale
   implementation branches before declaring the merge goal complete.

The handoff commit itself may contain known-red tests because the user asked to
stop and preserve all current tracked work. Report that fact plainly.

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
