# Slice 3 foundations review — fresh read-only `opus-analyst`

**Scope.** `crates/tui` (package `tui-next`, lib `tui_next`), `crates/tui-testing`, `xtask`, `crates/tui/tests/**`, `crates/tui/examples/12_author_component.rs`, `crates/tui/README.md`, at commit `18afddd`. Read against `COMPONENT_ARCHITECTURE.md` §3–§13, §16, §17.0, §21–§24, Appendix B, and `docs/audit/modern-api-audit.md` §1–§2.

**Verdict.** **Components may build on this surface: NO as it stands; YES after the seven blockers and the eight adjudications below are applied.** The foundation is substantially correct, well-documented and unusually honestly tested for a first cut — the intent queue, focus ring, capture, scroll, layout, reconcile core and the conformance driver are ready. Seven defects are load-bearing for Slice 4 and cannot be fixed by a 4x owner (they all live in files Slice 3 owns), and four architecture-document amendments are required so the gates stop asserting things that are false.

**Method note.** Facts are `path:line` citations. Numeric colour claims marked *(estimate)* are arithmetic I did by hand and the builder must re-derive before blessing.

---

## 1. Findings, ranked

### BLOCKER

**BL-1 — Style precedence is wrong: the variant delta is applied *after* the family's state rules, and the test that should catch it is vacuous.**
`crates/tui/src/theme/resolve.rs:52-61`:

```rust
if let Some(part) = r.parts.get(p)                              { acc = part.apply(acc, live); }   // base + family STATE RULES
if let Some(part) = r.variant(variant).and_then(|m| m.get(p))   { acc = part.apply(acc, live); }   // variant base + variant state rules
```

`PartRecipe::apply` (`theme/recipe.rs:284-294`) merges the base *and then* every matching state rule. So the ordering is `family.base → family.states → variant.base → variant.states`, but §11.3 fixes it as `1 family base → 2 variant delta → 3 state rules (all of them, by specificity)`. Consequence: any variant that sets a base colour silently defeats the family's `HOVERED`/`FOCUSED`/`PRESSED`/`ERROR` state rules. Every downstream `Theme::define_variant(…, |r| r.part(P).base(…))` is affected; the built-ins dodge it only because `button_variant` re-declares the full state set per variant (`theme/builtin/mod.rs:91-188`).

The guard test `theme/resolve.rs:327-385` asserts "3 over 2" at line 347 with `Some(t.color.focus)` — and `Theme::junie()` sets `focus: GREEN` and `accent: GREEN` to the *same* colour (`theme/builtin/junie.rs:63`, `:67`). The assertion is satisfied under either ordering. **The named §16.1 test `precedence_family_then_variant_then_state_then_global_then_scope_then_instance` currently passes without proving the requirement.**

*Exact fix.* Split `PartRecipe` application into `apply_base(acc)` (base + `glyph`/`size` slots) and `apply_states(acc, live)`, and in `accumulate`:

```rust
let fam  = recipes.get(f);
let var  = fam.and_then(|r| r.variant(variant)).and_then(|m| m.get(p));
if let Some(part) = fam.and_then(|r| r.parts.get(p)) { acc = part.apply_base(acc); }   // 1
if let Some(part) = var                              { acc = part.apply_base(acc); }   // 2
// 3: family and variant state rules merged, specificity ascending, family first on a tie
acc = merge_states_in_specificity_order(acc, family_states, variant_states, live);
```

Both rule lists are already stored pre-sorted (`recipe.rs:271-281`), so step 3 is a stable two-way merge — still allocation-free, still O(n+m).

*Test.* Rewrite the "3 over 2" arm of `precedence_…` so the state rule uses a role whose bound colour differs from the variant's under **both** built-in themes (e.g. family state rule `Role::Warning`, variant base `Role::Accent`), and add `theme::state_rules_beat_a_variant_base` plus `theme::family_and_variant_state_rules_interleave_by_specificity`.

---

**BL-2 — Two "unreachable" arms are infinite spin loops, not panics.**
`crates/tui/src/ui/mod.rs:609-613` (`unreachable_cache`) and `crates/tui/src/theme/recipe.rs:247-253` (`unreachable_entry`):

```rust
fn unreachable_entry<T>() -> T { loop { core::hint::spin_loop(); } }
```

These exist only to satisfy `clippy::panic`/`expect_used` at deny. A library that hangs the process with raw mode on and the alternate screen entered is strictly worse than a panic (`TerminalSession`'s panic hook restores the terminal; a livelock leaves the user with an unusable terminal and no stack). Both arms are genuinely dead, which makes this cheap to fix.

*Exact fix.* Remove both functions.
- `PartMap::entry` (`recipe.rs:200-222`): after `Vec::insert(i, …)` the index is valid by construction — return through a single documented suppression:
  ```rust
  #[expect(clippy::expect_used, reason = "Vec::insert(i, _) makes get_mut(i) infallible")]
  self.entries.get_mut(i).map(|(_, v)| v).expect("slot just inserted")
  ```
  (or restructure so the insert branch returns `self.entries.last_mut()` after a `push` at the sorted position). Same for `Recipes::get_mut` (`recipe.rs:376-388`) and `Recipe::variant_mut` (`recipe.rs:335-346`).
- `Ui::cache` (`ui/mod.rs:490-520`): collapse the find/insert/downcast into one pass that returns the reference from the insert branch directly.

*Test.* `architecture::no_unreachable_spin_loops` — an `xtask` rule forbidding `loop {` with `spin_loop` in `crates/tui/src/**`, added to the §22.7 table as rule 27.

---

**BL-3 — `Ui::raw()` marks the whole clip rect written and clobbers the recorded roles; `CellUi::drop` calls it on every right-aligned cell.**
`crates/tui/src/ui/paint.rs:214-218` marks `self.clip` (the *component's* clip, not the cell). `CellUi::drop` (`collection/rowui.rs:467`) uses it purely to shift painted cells for alignment, and `RowUi::raw` (`rowui.rs:293-298`) does the same. Two consequences:

1. Inside a layer, `LayerDraw::written` becomes all-true for the component's clip, so `composite_onto` (`ui/layer_buf.rs:96-106`) copies **unpainted** cells over the page — the written-cell bitset that §3.3 step 12 and R3 rest on is defeated by any grid/list with a right-aligned cell inside a dialog.
2. On the page, `mark_area` writes `self.roles` into every cell of the clip (`ui/mod.rs:551-565`), so `dim_layer`'s role walk (`ui/paint.rs:224-261`, §11.6) dims those cells with the wrong role.

*Exact fix.* Give `CellUi::drop` and `RowUi::raw` a non-marking accessor scoped to their own rect:

```rust
// ui/mod.rs
pub(crate) fn buffer_in(&mut self, area: Rect) -> (&mut Buffer, Rect) {   // marks `area`, not the clip
    let a = area.intersection(self.clip);
    self.mark_area(a);
    (self.buffer(), a)
}
```
and change `Ui::raw()` to `self.mark_area(self.clip)` **only** when it is the documented public escape hatch (that behaviour is correct for `raw()` itself; the internal callers must stop using it).

*Test.* `layer::composite_copies_only_painted_cells` (draw a `RowUi` with a right-aligned `part()` inside a layer over a sentinel-filled page; assert unpainted cells keep the sentinel) and `ui::dim_layer_uses_the_role_of_the_painted_cell`.

---

**BL-4 — `Ui::paint_spans` allocates a `Vec` per call, on the row path.**
`crates/tui/src/ui/paint.rs:86-97` collects `Vec<RawSpan<'_>>` before `Buffer::set_line`. `RowUi::label_spans` (`collection/rowui.rs:189-194`) routes through it, so every span-rendered row costs one allocation per row per frame. That directly contradicts §20.9-6 (R5, "no intermediate allocation") and makes §16.6's `frame_showcase_lists_120x40 < 20 allocs/frame`, `grid_500x12_render < 100` and `viewport_100k_lines_render` unreachable for `TextViewport` and `DiffView` — and a Slice-4 owner cannot fix it, because `ui/` is Slice 3's.

*Exact fix.* Paint span-by-span through `Buffer::set_span` (still a ratatui writer, so R‑3's "cannot drift from `set_stringn`'s width accounting" holds), accumulating the x cursor and the per-span role marks:

```rust
let mut x = area.x;
for sp in spans {
    if x >= area.right() { break; }
    let cell = Rect { x, width: area.right() - x, ..area };
    let st = base.patch(span_style(theme, surface, sp));
    self.set_roles(CellRoles { fg: sp.role.or(base_fg), bg: base_bg });
    let (end, _) = self.buffer().set_span(cell.x, cell.y, &RawSpan::styled(sp.text, st), cell.width);
    self.mark_area(Rect { x, y: area.y, width: end - x, height: 1 });
    x = end;
}
```
Amend §22 R‑3 to name `Buffer::set_span` as the sanctioned per-span writer alongside `set_line`, and amend §17.0 A2's `paint_spans` signature to the implemented three-argument form (see D-13 below).

*Test.* Extend `ui::paint_spans_matches_row_ui_label_spans` (§16.1, currently **missing**) with an allocation assertion: painting 500 rows × 3 spans records 0 allocations.

---

**BL-5 — Ansi16 downgrade contradicts DESIGN.md and the existing rendered output.**
`theme/downgrade.rs:167-178` uses CIE76 ΔE over the sixteen xterm defaults; `downgrade.rs:397-404` pins `#48e054 → Color::Green` and `#e44545 → Color::Red`. `DESIGN.md:320` states *"At 16 colours the accent is LightGreen and error is LightRed"*, and the legacy metric (`src/theme.rs:604-641`) plus its test (`src/theme.rs:647-655`) produce exactly that. The document's authority order (`COMPONENT_ARCHITECTURE.md:5`) puts `DESIGN.md` and existing rendered output/tests **above** the implementation spec, and §20.10 does not list a 16-colour change, so it is a regression as defined. Full adjudication in §2.3.

---

**BL-6 — `Ui::set_cursor` keeps the *first* writer on a layer, not the *focused* one.**
`crates/tui/src/ui/mod.rs:427-448`: `keep = req.layer > cur.layer`. With two same-layer writers (two `TextInput`s in a `Form`, both flagged `EDITING`, both calling `set_cursor`), the first drawn wins; the focused one is rejected and records `Diagnostic::CursorRejected`; then §3.3 step 15 (`cursor::resolve`, `cursor.rs:33-45`, correct as written) drops the retained request because its owner is not focused. Net: **no cursor at all, plus a spurious diagnostic**, which `*::no_diagnostics_are_emitted_during_the_journey` (§16.4) will fail. §8.4 makes filtering the runtime's job, so components are entitled to write unconditionally.

*Exact fix.* Make `Ui::set_cursor` keep the best candidate by `(layer, is_focused_owner)`, not by arrival order. `Ui` already has `self.last.state(owner)`; prefer a request whose owner carries `FOCUSED`, then the higher layer, then the later write:

```rust
let focused = self.state(owner).contains(StateFlags::FOCUSED);
let better = match self.frame.cursor {
    None => true,
    Some(cur) => (req.layer, focused) > (cur.layer, cur.focused),
};
```
(store `focused` on `CursorRequest`). Record `CursorRejected` for the loser only when it is non-inert.

*Test.* `cursor::the_focused_owners_write_wins_on_the_same_layer` (§16.1 addition).

---

**BL-7 — `Harness::resolved` / `Runtime::resolved` hardcode `Family::BUTTON`.**
`crates/tui/src/runtime.rs:939-946`. §16.4's theme-coupling migration contract replaces `assert_eq!(fg, Theme::junie().focus)` with `h.resolved(id, Part::GUTTER).style.fg` — for a `List`, a `Tabs`, a `Field`, that now resolves the **button** recipe and returns a colour the component never painted. Every migrated assertion in Slices 5–7 would be silently wrong.

*Exact fix.* Record the resolution key. `Ui::style`/`style_patched` already carry `(family, variant, part)`; under `#[cfg(feature = "testing")]` extend `FrameState::styled_parts` to `Vec<(Id, Family, Variant, Part, Resolved)>` written by `RowUi::style_of` and by an explicit `Ui::note_styled` at each component's own query, and make `Runtime::resolved(id, part)` return the recorded `Resolved` (falling back to `resolved_in` only when nothing was recorded). Keep `resolved_in(f, v, id, p)` as the explicit escape hatch.

*Test.* `harness::resolved_reports_the_family_the_component_actually_queried`.

---

### MAJOR

**MA-1 — `Registry::hit` orders by registration index, not by layer.** `hit.rs:248-254` returns the last-registered covering region "regardless of layer". A page control drawn *after* `ui.layer(POPOVER, …)` shadows the popover; the runtime then sees `hit.layer < top_layer` (`runtime.rs:350`) and treats a click **on the popover** as an outside click, dismissing it. Masked for modals only because `inert_below` suppresses page registration. §9.1's "z-order is the layer order, NOT the call order" must hold for hit-testing too. `hit::higher_layer_shadows_lower` (`hit.rs:339-346`) passes by registration order and proves nothing. *Fix:* `hit()` selects `max_by_key(|r| (r.layer, index))`; add `hit::a_lower_layer_region_registered_later_does_not_shadow_a_higher_one`.

**MA-2 — `xtask`'s source scan stops at the first `#[cfg(test)]` in a file.** `xtask/src/main.rs:83-92` (`non_test_lines`) `break`s on the first `#[cfg(test)]` line. `theme/resolve.rs:239` puts `#[cfg(test)] pub(crate) const fn stats` mid-file, so **everything after line 239 in `resolve.rs` is unscanned** by all 26 forbidden-pattern rules; `runtime.rs:1028` does the same. *Fix:* skip only the `#[cfg(test)]`-attributed item (brace-match through `syn`, which xtask already depends on) or, minimally, skip a line and its following item rather than the file tail.

**MA-3 — `text::row_ui_matches_fit_for_every_fixture` skips exactly the cases it exists to protect.** `crates/tui/tests/render.rs:252-289`: the reference `fit` is re-implemented in the test over **chars** (the legacy one walks graphemes, `src/ui/text.rs`); `cellable` drops control/ZWSP/BOM inputs; line 283 `continue`s on every non-ASCII multi-byte input that needs truncation — i.e. CJK, emoji and combining marks at the cut; and both sides are `trim_end_matches(' ')`, discarding padding. §20.10's "any change to padding or ellipsis placement … is a regression" rests entirely on this test. *Fix:* use the legacy grapheme-walking `fit` verbatim as the reference, remove the `continue`, and compare cell symbols including trailing padding.

**MA-4 — The `crossterm` feature gates nothing, so `core_is_backend_free` is theatre.** `crates/tui/Cargo.toml:25` declares `crossterm = []` (an empty feature) and line 31 makes `ratatui-crossterm` a **normal, non-optional** dependency; `src/event.rs:12` re-exports `crossterm::event::{KeyCode, KeyModifiers}` unconditionally. `cargo check -p tui-next --no-default-features` (`xtask/src/main.rs:983-994`) therefore still compiles the backend crate and proves nothing about backend independence. The dependency itself is correct (see §2.1); the *claim* and the *gate* are not. Also deviates from Appendix B.2's `crossterm = ["dep:ratatui-crossterm"]`.

**MA-5 — `Capture.origin` is documented as the pointer position but set to the area's top-left.** `capture.rs:22-23` says "Where the pointer was when the claim was made"; `Cx::capture` (`ui/cx.rs:205-220`) sets `origin: Position::new(area.x, area.y)`. `Cx::capture_origin()` exists so a splitter/scrollbar can compute `pos - origin`; with this value the delta is wrong by the press offset within the thumb. The unit test (`capture.rs:109-115`) hand-builds a `Capture` with `origin: (5,5)` and never exercises `Cx::capture`. *Fix:* the runtime records the live press position in `Interaction::press`; expose it to `Cx` and use it as `origin`. Add `capture::origin_is_the_press_position`.

**MA-6 — `Family::custom(..)` resolves to nothing, so Scenario G renders invisible.** `theme/resolve.rs:52` — `recipes.get(f)` returns `None` for a downstream family, `accumulate` returns an empty patch, and `bind` yields `Style::new()`. `examples/12_author_component.rs:196-204` therefore paints an unstyled control, and `mono_states_are_distinguishable` could never pass for it. §11.2 does not say what a custom family inherits. *Fix (recommended):* add a neutral fallback recipe used when `Recipes::get(f)` misses — `row_like`'s `CONTAINER/GUTTER/MARKER/LABEL/META` set — and document it in §11.2 as "a custom family starts from the neutral recipe; `define_family` replaces it". Add `theme::a_custom_family_resolves_through_the_neutral_recipe`.

**MA-7 — Focus intents enqueued for a `pending_focus` are dropped on resize.** `runtime.rs:661-668` drains `pending_focus` into the queue, then `runtime.rs:672-675` returns early for `Input::Resize` without running `app.update`, so the `FocusOut`/`FocusIn` pair is discarded by the next `intents.clear()`. *Fix:* handle the resize, then fall through to `run_update(None)` (still with no input intents), or re-stage `pending_focus`.

**MA-8 — Case 9's default state list is a subset of §16.2's.** `crates/tui-testing/src/conformance/mod.rs:107-113` — `DEFAULT_MONO_STATES` is `{default, focused, selected, pressed, disabled}`; §16.2 case 9 requires *default / focused / selected / pressed / disabled / error / warning / editing / busy / active*. As written a component silently gets a five-state check. *Fix:* make the default the full ten and let `mono_states()` only **narrow** it, with the driver asserting that any state the component's `Caps` imply is present.

**MA-9 — Case 12 does not test what §16.2 specifies.** `conformance/driver.rs:502-555` checks click identity across a reorder but never sets cursor/checked on `k₁,k₂` nor asserts they survive `reconcile`. `CollectionCore` supports it (`collection/reconcile.rs`); the driver should exercise it.

**MA-10 — `every_named_test_exists` does not exist.** The single most valuable gate for this review is absent from `xtask`'s `CHECKS` (`xtask/src/main.rs:110-158`) and from `crates/tui/tests/architecture.rs`. Without it, the missing/renamed names in §1.4 below are invisible to CI. Also missing from the check set: `conformance_covers_every_public_component`, `state_override_is_used_only_in_apps_and_fixtures`, `all_examples_compile` / `examples_are_external_consumers`.

**MA-11 — `every_foreign_type_in_the_public_surface_is_re_exported` is a substring grep, not the §24.1 rustdoc-json check.** `crates/tui/tests/architecture.rs:153-178` asserts only that eight names appear somewhere in `lib.rs`. It cannot detect the case it was written for — a `pub` signature naming an unexported foreign type. Record as a deviation with a Slice-8 plan, or implement it against `cargo rustdoc --output-format json`.

**MA-12 — `trybuild` is absent, so three named compile-fail tests do not exist.** `crates/tui/Cargo.toml:36-37` has only `tui-next-testing` as a dev-dependency. Missing: `response::must_use_is_enforced`, `response::bitor_is_defined_only_for_unit`, `secret::is_not_clone_not_eq` (§16.1). `crates/tui/tests/ui/` does not exist. These are precisely the tests that pin the type-level guarantees §6.1 and §15 claim.

**MA-13 — `zeroize` can be elided, and its named test does not test it.** `secret.rs:74-79` fills a moved-out `Vec` that is immediately dropped; LLVM is permitted to remove the dead stores. The §16.1 name is `zeroize_overwrites_before_drop`; the code has `zeroize_clears` (`secret.rs:141-149`), which asserts only emptiness. *Fix:* add `core::hint::black_box(&bytes)` (and a `compiler_fence(Ordering::SeqCst)`) after the fill, keeping `#![forbid(unsafe_code)]`; rename the test and assert the observable property available in safe Rust — that the `Secret`'s capacity is released and a fresh `expose()` is empty — plus a comment naming the compiler-elision risk as a known limit of safe-Rust zeroization.

**MA-14 — `doc-check` misses §24 and is a heuristic, not rustdoc-json.** `xtask/src/main.rs:1398-1400` keeps `Some(3..=17 | 21..=23)`; §24 (which declares `SelectAction`, `RadioGroupAction`, `ChipBarAction`, `border::ASCII`, `FormData::options` and rewrites example 13) is not checked. Also `foreign_members()` (`:1223-1380`) allow-lists legacy names (`("Theme", ["row","gutter"])`, `("Interaction", ["pressed","focus_hidden"])`) as if they were foreign API — a fudge that should be an explicit `doc_check_allow.txt` entry instead. §21 item 34 specifies rustdoc-json; record the heuristic as a deviation with a Slice-8 upgrade.

---

### MINOR

- **MI-1** `FocusRing::innermost_scope` (`focus.rs:211-217`) uses `.rev().max_by(layer)`, which returns the **earliest** scope on the highest layer while the doc says "latest". Harmless today (one scope per layer); fix the code or the doc.
- **MI-2** `FocusRing::reconcile` (`focus.rs:350-351`) appends `.or_else(|| self.reachable().next())` beyond §3.3 step 14's `(d) None`. Better behaviour; must be recorded as an amendment to step 14 rather than left undeclared.
- **MI-3** `focus::click_only_entries_are_never_reachable` (`focus.rs:494-499`), `focus::read_only_entries_stay_in_the_ring` (`:484-491`) and `focus::restore_target_receives_keys_before_the_next_draw` (`:639-651`) exercise none of the mechanism their names claim (`Focusability::ClickOnly`/`FocusableReadOnly` live in `Ui::register_entry`, `ui/mod.rs:323-351`; key resolution lives in `Runtime`). Move them to runtime-level tests. Same for `hit::inert_below_registers_nothing` (`hit.rs:359-367`).
- **MI-4** `Runtime::handle` calls `self.app.keymap().conflicts()` on **every** input (`runtime.rs:653-655`), an O(n²) scan per event. Allocation-free when clean (`Vec::new()`), but wasteful; compute once per keymap change or under `debug_assertions`.
- **MI-5** Wheel routing is not layer-filtered (`runtime.rs:416-422`): a wheel over the page below a **popover** scrolls the page. Decide and document (I recommend: deliver only when `hit_scroll(...).layer == top_layer`, matching `deliverable()`).
- **MI-6** `dismiss_top` enqueues `Intent::Cancel` (`runtime.rs:537`) for *every* dismissal reason; §6.1 defines `Cancel` as "Esc reached this owner after layer dismissal". Gate it on `DismissReason::Esc`.
- **MI-7** `run_update`'s give-up path (`runtime.rs:636-644`) calls `apply_staged_focus()` — which enqueues `FocusOut`/`FocusIn` — then immediately `intents.clear()`, so §21 item 11's "applies the pending FocusOut **and** the matching FocusIn" does not happen. `a_fifth_focus_pass_is_diagnosed_and_applied` (`runtime.rs:1356-1367`) asserts only `focus().is_some()`. Deliver the pair on the next `handle` via `pending_focus`.
- **MI-8** `RowUi::columns` silently truncates past `MAX_COLUMNS = 16` (`collection/rowui.rs:21`, `:278`). Record a diagnostic or document the cap in §12.2.
- **MI-9** `LayerSpec::modal`'s `min_size: (0,0)` (`layer.rs:176`) makes `resolve_anchor` return the **whole screen** (`layer.rs:301-303`), so every dialog must re-implement centering — the opposite of §9.1's "one resolver". Decide: either the layer content is expected to size itself (document it, and say so on `Ui::layer`) or `Dialog` must pass `.min_size(design.size.dialog_width, h)`. 4F needs this settled.
- **MI-10** `fuzzy` allocates three `Vec`s per call (`text/fuzzy.rs:25-26`, `:52`). Fine at Slice 3; a 100k-item `Picker` filter would be 300k allocations. Flag for 4F.
- **MI-11** `pub mod text` (`lib.rs:38`) exports `TextBuffer`, `grapheme_width`, `is_word_char`, `thousands` — none of which appear in Appendix B.3/B.4's curated lists. `pub mod layout` similarly exports `distribute`/`distribute_into`. Either curate them into the facade deliberately or make the modules `pub(crate)` and re-export the named items.
- **MI-12** `Theme::fingerprint` (`theme/mod.rs:201-205`) formats the whole theme into a `String`. Public, allocation-heavy; make it hash the tokens structurally or move it behind `testing`.
- **MI-13** `Recipes::apply_mono_fallbacks` (`theme/downgrade.rs:335-343`) appends rules to `recipe.parts` only, never to `variant_mut(v)` maps. Under BL-1's fix the family rules would then be beaten by nothing, but a variant that re-declares `PRESSED` still overrides the mono bracket rule. Document the interaction or apply the fallbacks to variant maps too.
- **MI-14** `PERF_TARGET` (`crates/tui-testing/src/perf.rs:18`, used at `tests/perf.rs:233`, `:479`) is a third knob §16.6 never declares (which names only `PERF_STRICT`/`PERF_BLESS`). Declare it or fold it into `PERF_STRICT`.
- **MI-15** `xtask` rule 22 (`main.rs:378-390`) narrows §22.7's regex to `Color::Rgb\(\s*\d|Color::from_u32\(\s*0x`, which lets `Color::Rgb(r, g, b)` through anywhere. Restore the broad regex and add `crates/tui/src/theme/downgrade.rs` and `crates/tui/src/theme/builder.rs` to the rule's `allowed` **paths** (which do not feed the "allow-list must be empty" condition). See D-10.
- **MI-16** `Ui::style` takes `&mut self` (`ui/mod.rs:217`), so `Measure::measure(&self, ui: &Ui, …)` (`measure.rs:73`) cannot resolve a style. Every component whose natural size depends on a themed glyph width will need `measure` widened or a `&self` `Theme::resolve` path. Settle before 4A.

---

## 2. Part 1 — the eight adjudications

### 2.1 `ratatui-crossterm` as a normal dependency — **CONFIRMED, with a document amendment and a gate that bites**

*Facts.* `crates/tui/Cargo.toml:25` `crossterm = []`; `:31` `ratatui-crossterm.workspace = true` (non-optional). `src/event.rs:12` re-exports `KeyCode`/`KeyModifiers` unconditionally; `Input::from_crossterm` (`event.rs:201`) is unconditional; only `runtime::session` is `#[cfg(feature = "crossterm")]` (`runtime.rs:9-10`).

*Decision.* The builder's reasoning is correct and the alternatives are rejected:
- **Own `KeyCode`/`KeyModifiers`:** rejected — §22.1 already rejected it, and the reason (≈40 hand-written variants plus `From` impls, losing crossterm's `PartialEq`/`Hash` ASCII-case normalisation that `Chord: Eq + Hash` relies on, §22.2 item 6) is unchanged.
- **Gating `Intent::Key`:** rejected — it would make the shape of `Intent`, `Chord`, `Binding`, `KeyMap` and `BindingState` depend on a feature. A core whose intent enum changes shape under `--no-default-features` is not a core; the cure is worse than the disease.

*Change to code.* None to the dependency. But the `crossterm` feature must be honest and enforced:
1. `crates/tui/Cargo.toml`: keep `crossterm = []`, and put a comment naming what it gates (`TerminalSession`, `run`, `DefaultTerminal`) — already present at `:22-24`; keep.
2. Add `xtask` forbidden-pattern **rule 27**: `CrosstermBackend|ratatui_crossterm::(?!crossterm::event)` allowed only in `crates/tui/src/runtime/session.rs` — the mechanical proof that the backend is confined to one file.
3. Add `xtask` boundary check `ratatui_crossterm_is_named_in_exactly_two_files` — `src/event.rs` (the `crossterm::event` vocabulary) and `src/runtime/session.rs` (the backend).

*Change to the document.* Amend §22.1's "Consequences recorded" third bullet and §16.5's `core_is_backend_free` row and Appendix B.2's `crates/tui/Cargo.toml` block:

> `ratatui-crossterm` is a **normal, non-optional** dependency of `junie-tui`, taken for its version-unified `crossterm` re-export — the key vocabulary `Key`/`Chord`/`KeyMap` name (§6.1, R‑14) — never for `CrosstermBackend`. The `crossterm` feature gates the *terminal session* (`TerminalSession`, `run`, `DefaultTerminal`) and nothing else; `crossterm = []` is therefore the correct manifest form, not `crossterm = ["dep:ratatui-crossterm"]`. `cargo check -p junie-tui --no-default-features` remains a gate: it proves that nothing outside `runtime/session.rs` needs a backend. The stronger claim — that the widget layer is backend-independent — is proved by forbidden-pattern rule 27 (`CrosstermBackend` only in `runtime/session.rs`) and by `architecture::ratatui_crossterm_is_named_in_exactly_two_files`.

Also amend §18.1's `runtime.rs` row: `Input::from_crossterm` is **not** feature-gated (it needs no backend); `TerminalSession`/`DefaultTerminal`/`run` are.

*Test.* `architecture::ratatui_crossterm_is_named_in_exactly_two_files`; `no_deprecated_or_legacy_api_usage` rule 27.

---

### 2.2 `Id` structural equality — **CONFIRMED, but the invariant is unproven and the named test is vacuous**

*Facts.* `id.rs:56-61` derives `PartialEq, Eq, Hash, PartialOrd, Ord` over `{hash, #[cfg(debug_assertions)] label}`. §7.1 specified hash-only manual impls. Structural derives are **required**: `const Id` in a `match` pattern needs `StructuralPartialEq`, and §15.1's `FormData::value(id) → match id { NAME => … }` and example 13 (`COMPONENT_ARCHITECTURE.md:3332-3341`) depend on it. Every field is structural-match (`u64`, `&'static str`, `Part(u16)`, `ItemKey`, `usize`).

*Decision.* **Confirm the derive.** The debug label *is* a pure function of the segments that produced the hash: `root` is carried unchanged from the root segment and `tail` records the last segment (`id.rs:96-148`), both of which are determined by the segment chain the hash was computed over. Therefore equal hashes ⇒ equal segment chains ⇒ equal labels, except under a genuine FNV collision, where the label is the more honest answer. Debug and release compare identically.

*Change to the document.* §7.1: replace the manual-impl block with

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Id { hash: u64, #[cfg(debug_assertions)] label: DebugLabel }
```
and the sentence:

> Equality, hashing and ordering are **structural** (derived), because a `const Id` used as a `match` pattern requires structural equality (§15.1's `FormData`, example 13). The debug label is a pure function of the segments the hash was computed over — `root` from the root segment, `tail` from the last — so two ids with equal hashes carry equal labels and debug and release compare identically; only a genuine FNV collision could differ, and there the label is the more honest answer.

*Change to code.* None. **Test:** `id_equality_ignores_debug_label` as written (`id.rs:428-444`) compares two ids with *identical* labels and proves nothing. Replace its body with the equivalence it actually asserts:

```rust
// over a corpus of ~200 ids built by every derivation, equality is exactly hash equality
for a in &corpus { for b in &corpus {
    assert_eq!(a == b, a.hash() == b.hash());
    assert_eq!(a.cmp(b), a.hash().cmp(&b.hash()));
}}
```
and rename it `id_equality_is_exactly_hash_equality` in §16.1 (keeping the old name as a second, `#[cfg(debug_assertions)]`-only assertion that a label never changes an answer).

---

### 2.3 Ansi16 downgrade metric — **REJECT CIE76; restore the legacy categorical metric** (BL-5)

*Facts.* `theme/downgrade.rs:167-178` (CIE76); the pinned outcomes at `downgrade.rs:397-404` are `#48e054 → Green`, `#e44545 → Red`. `DESIGN.md:320`: *"At 16 colours the accent is LightGreen and error is LightRed."* Legacy `nearest_16` (`src/theme.rs:604-641`) is a **categorical** metric — grey when `max-min < 40`, else hue family by dominant channel with a `Yellow` special case, then `bright = max > 180` selecting the light half. Legacy test `src/theme.rs:647-655` asserts LightGreen/LightRed.

*Decision.* **Revert `nearest_16` to the legacy categorical metric, verbatim.** Four reasons, in order of weight:

1. **Authority.** `COMPONENT_ARCHITECTURE.md:5` orders `REFACTORING_GOAL.md › DESIGN.md › existing rendered output/tests › current source`. DESIGN.md names the outcome and the existing test pins it; §21 item 29's "CIE76" is implementation spec, which is subordinate. §20.10 lists no 16-colour change, so this is a regression by the document's own definition.
2. **The metric answers the wrong question.** A 16-colour downgrade must preserve *hue identity and brightness class*, not minimise perceptual distance. CIE76 minimisation demonstrably loses both: it maps Junie's accent and its error colour into the **dark** half, so the "accent is the brightest signal on screen" property the whole accent system rests on is gone, and `danger_soft` `#d98a8a` — which the legacy metric keeps as `LightRed` — lands on `DarkGray` under CIE76 *(estimate: ΔE≈30 to DarkGray vs ≈61 to Red; the builder must re-derive)*, so a destructive label at rest stops being red at all.
3. Both `#48e054` and `#e44545` genuinely minimise ΔE against the dark primaries *(estimate: L\*≈78 vs 72/88 for green)*, so there is no tie-break or bias that recovers DESIGN.md's answer while keeping ΔE. The metric must change, not be tuned.
4. The legacy metric is exact integer arithmetic, `const`-friendly, and already has a blessed baseline.

*Change to code.* Replace `theme/downgrade.rs:167-178` with the legacy function, operating on the `(r,g,b)` from `rgb_of`:

```rust
/// Nearest of the 16 xterm defaults by hue family and brightness class
/// (`DESIGN.md:320`): a near-grey collapses to the grey ladder; otherwise the
/// dominant channel picks the hue and `max > 180` picks the light half.
fn nearest_16(rgb: (u8, u8, u8)) -> Color {
    let (r, g, b) = rgb;
    let lum = (u32::from(r) * 299 + u32::from(g) * 587 + u32::from(b) * 114) / 1000;
    let max = u32::from(r.max(g).max(b));
    let min = u32::from(r.min(g).min(b));
    if max.saturating_sub(min) < 40 {
        return match lum { 0..=30 => Color::Black, 31..=110 => Color::DarkGray,
                           111..=200 => Color::Gray, _ => Color::White };
    }
    let bright = max > 180;
    match (r >= g && r >= b, g >= r && g >= b) {
        (true, _) if g > 120 && b < 80 => Color::Yellow,
        (true, _) => if bright { Color::LightRed }   else { Color::Red },
        (_, true) => if bright { Color::LightGreen } else { Color::Green },
        _         => if bright { Color::LightBlue }  else { Color::Blue },
    }
}
```
`lab_of` stays (it is used by `ThemeBuilder`'s L\* derivation); keep `nearest_256` and `mono` unchanged.

*Change to the document.* §21 item 29 and §11.4's `downgrade_color` comment:

> `nearest_16`: **not** a ΔE minimisation. A colour whose channel spread is under 40 collapses to the grey ladder by ITU-R BT.601 luma (`≤30 Black`, `≤110 DarkGray`, `≤200 Gray`, else `White`); otherwise the dominant channel selects the hue family (with `r ≥ g,b ∧ g > 120 ∧ b < 80` reading as Yellow) and `max(r,g,b) > 180` selects the light half. Recorded rejection: nearest-by-CIE76 ΔE. It is the more "correct" perceptual answer and the wrong design answer — it maps Junie's accent `#48e054` and error `#e44545` into the dark half, discarding the brightness contrast the accent system rests on, and collapses `danger_soft` onto a grey. `DESIGN.md:320` fixes the outcome (accent LightGreen, error LightRed) and the authority order puts it above this document.

*Test.* Amend `theme::downgrade_is_deterministic_per_level` (`downgrade.rs:385-441`) to assert `LightGreen`/`LightRed`/`Yellow`, and add `theme::ansi16_preserves_hue_family_and_brightness` pinning `DESIGN.md:320` plus `danger_soft → LightRed`, `border_subtle → Black`, `fg[1] → Gray`. **No baseline is re-blessed for this change** — it restores the recorded output.

---

### 2.4 `fit_10k_grapheme_line_to_80` — **REJECT "≤ 8"; split the benchmark and keep exactly 0**

*Facts.* `crates/tui/tests/perf.rs:304-332` asserts `s.allocs <= 8`; the corpus (`crates/tui-testing/src/perf.rs:365-381`) includes a ZWJ family emoji whose symbol exceeds ratatui `Cell`'s inline `CompactString` capacity, so each such cell heap-allocates. The benchmark also measures `Ui::paint_str`, not `RowUi::label`, so it never exercises the ellipsis path it is named for.

*Decision.* The obligation (§20.9-6, R5) is *the painter allocates nothing*. The observed allocations belong to ratatui's `Cell` symbol storage — a property of the buffer, not of the painter — so relaxing the painter's assertion to a magic constant hides the real invariant. Split it:

- **`fit_10k_grapheme_line_to_80`** — corpus restricted to graphemes that fit `Cell`'s inline symbol storage (ASCII + CJK + combining marks; no ZWJ sequences), paints through **`RowUi::label`**, asserts **exactly 0** allocations. This is §16.6's row, unchanged.
- **`fit_10k_grapheme_line_to_80_wide`** — the ZWJ corpus, **reported**, with the binding assertion *allocations are bounded by the columns painted and independent of the line length*: run it at 10 000 and 100 000 graphemes into the same 80 columns and assert the two counts are equal and `≤ 80`.

*Change to the document.* §16.6's row becomes:

| `fit_10k_grapheme_line_to_80` | exactly 3 | the `RowUi` equivalent records **0** over a corpus whose graphemes fit ratatui `Cell`'s inline symbol storage; `fit_10k_grapheme_line_to_80_wide` (added) reports the ZWJ-emoji case, where allocations are ratatui `Cell` heap symbols, **bounded by the columns painted and independent of line length** (asserted by equality between a 10 k and a 100 k line) |

*Change to code.* Add `unicode_line_inline(n)` beside `unicode_line(n)` in `crates/tui-testing/src/perf.rs`; rewrite both benchmarks to drive `RowUi::label`.

---

### 2.5 `smallvec` in the closure — **CONFIRMED unavoidable; correct §22.7 assertion (2)**

*Facts.* `smallvec` arrives via `ratatui-crossterm → crossterm → parking_lot → smallvec`; `ratatui-crossterm` is mandatory (§2.1). The implementation already prunes: `xtask/src/main.rs:538-543` runs `cargo tree --prune crossterm` and checks `FORBIDDEN` against the pruned closure (`:601-618`). That works but silently deletes a whole subtree without asserting anything about it, and §22.7 assertion (2) as written ("absent from the normal closure") is simply false.

*Decision.* Keep the prune, add a positive assertion, and rewrite the document's clause into four parts:

> **(2a)** `ratatui`, `ratatui-widgets` and `ratatui-macros` are absent from `junie-tui`'s **entire** normal closure.
> **(2b)** `critical-section` and `palette` are absent from the **entire** normal closure (they can only arrive through `ratatui-core` features we disable).
> **(2c)** `smallvec`, `parking_lot`, `parking_lot_core`, `lock_api`, `scopeguard`, `libc`, `mio` and `signal-hook*` may appear **only beneath `ratatui-crossterm`**: every path from `junie-tui` to each of them passes through `ratatui-crossterm`. They are crossterm's internals, not a choice of ours; §22.4's decision is about *our* containers and is enforced by forbidden-pattern rule 26 over our source.
> **(2d)** `junie-tui`'s **direct** normal dependencies contain no `smallvec` and no direct `crossterm`.

*Change to code.* In `dependency_graph_is_exactly_the_declared_set` (`xtask/src/main.rs:578-664`): keep the pruned check for (2a)/(2d), move `critical-section`/`palette` to the **unpruned** closure for (2b), and add (2c) as `cargo tree -p tui-next -e normal --invert <crate>` for each of the eight names, asserting `ratatui-crossterm` appears on every printed path. Print the pruned subtree once on success so the exception is visible in CI output.

---

### 2.6 `intents_drain_is_o_1_when_the_queue_is_empty` — **replace the ±10 % wall-clock threshold with a deterministic probe count**

*Facts.* `crates/tui/tests/perf.rs:445-481` measures a 500-component and a 20-component frame and calls `check_ratio(..., 1.1, env_flag("PERF_TARGET"))` — asserted only under an undeclared env knob. The measurement itself is ~600 ns of `Runtime::handle`, of which the 500 probes at ~1.2 ns each are ≈0.1 %: a ±10 % band on the total cannot detect a regression in the thing it names, and is inside the noise of a shared runner.

*Decision.* Keep the benchmark, but move the binding assertion onto something deterministic. `Cx::intents` already has the mechanism (`intent.rs:327-344`: `if self.used == 0 { return … }`).

*Change to code.*
```rust
// intent.rs, under #[cfg(feature = "testing")]
impl IntentQueue { pub(crate) fn probes(&self) -> usize; }   // Cell<usize>, bumped in bucket_index
// runtime.rs, under #[cfg(feature = "testing")]
impl<A: App> Runtime<A> { pub fn intent_probes(&self) -> usize; }
```
Assert: a 500-component frame with an **empty** queue performs **exactly 0** probes; with 2 intents it performs **exactly 500** probes (one per `cx.intents` call) and **0** allocations; and `probes(500 components) == probes(20 components)` when the queue is empty.

*Change to the document.* §16.6's added-tests table:

| `intents_drain_is_o_1_when_the_queue_is_empty` | `crates/tui/tests/perf.rs::invariants` | a 500-component frame with 0 intents performs **0 bucket probes** and **0 allocations**, and costs the same as a 20-component frame with 0 intents; with 2 intents, probes are exactly one per drain call, allocations are 0, and total probe cost is ≤ 500 × 5 ns. The wall-clock ratio is **reported always and asserted only under `PERF_STRICT=1`, with a 1.25× band** — a ±10 % band on a ~600 ns measurement is inside the noise of a shared runner. |

Also declare `PERF_TARGET` in §16.6 or fold it into `PERF_STRICT` (MI-14).

---

### 2.7 `Track::Auto` semantics — **ACCEPT, declare them, and fix §17 example 9**

*Facts.* `crates/tui/src/layout.rs:16-21` documents and `:70-132` implements: unmeasured `Auto` takes **1 cell** when explicit `Flex` tracks exist, else an **equal share** of the remainder; `rows_measured`/`columns_measured` (`:231-238`, `:251-258`) take `natural: &[u16]`. Pinned by `layout::rows_distributes_flex_after_fixed` (`:518-551`).

*Decision.* **Accept.** It is deterministic, allocation-light, degrades sensibly, and keeps `Auto` expressible without a measurement pass — which is exactly why §10 kept `Auto` in the first place. The `_measured` variants are the correct home for `Measure`-derived sizes.

*Change to the document.* §10: change the `layout` block to

```rust
pub fn rows(area: Rect, heights: &[Track]) -> Vec<Rect>;
pub fn rows_measured(area: Rect, heights: &[Track], natural: &[u16]) -> Vec<Rect>;
pub fn columns(area: Rect, widths: &[Track], spacing: u16) -> Vec<Rect>;
pub fn columns_measured(area: Rect, widths: &[Track], spacing: u16, natural: &[u16]) -> Vec<Rect>;
pub fn distribute_into(total: u16, tracks: &[Track], spacing: u16, out: &mut [u16]);  // 0-alloc, RowUi::columns
```
and add to `Track`'s declaration:

> `Track::Auto` is content-sized. Without a measurement the primitive gives it **one cell** when explicit `Flex` tracks exist (so `Auto` never starves a `Flex`) and an **equal share of the remainder** when there are none. Supply the natural size through `rows_measured` / `columns_measured` to get the content size; a component that has a `Measure` impl should always do so.

**§17 example 9 must change.** `COMPONENT_ARCHITECTURE.md:2980` uses `layout::rows(body, &[Track::Auto, Track::Fixed(1), Track::Flex(1)])` for a two-row `Props`; under the accepted rule that gives `Props` one row and clips it. Rewrite as

```rust
let props = Props::new(&[("Table", self.target.as_str()), ("Rows", "12,481")]);
let natural = [props.measure(ui, Constraints::loose(body.width, body.height)).preferred.1];
let rows = layout::rows_measured(body, &[Track::Auto, Track::Fixed(1), Track::Flex(1)], &natural);
```
and re-check every other `Track::Auto` in §17 the same way (`xtask doc-check` will not catch this class — add it to the Slice-4 wave-1 review checklist).

*Test.* Add to §16.1 `layout.rs`: `auto_takes_one_cell_beside_flex_and_an_equal_share_without_it`, `rows_measured_uses_the_natural_size`.

---

### 2.8 Style-resolution cost — **the bound is a per-frame budget, not a per-query ratio**

*Facts.* `style_resolve_10k_parts` (`crates/tui/tests/perf.rs:164-182`) asserts **0 allocations** and reports ns; there is no `≤ 2×` assertion in the code at all. §20.9-1 wrote "ns ≤ 2× the pre-refactor `Theme::row`+`gutter` baseline". The pre-refactor operation is a field read on a 30-field `Copy` struct (`src/theme.rs:229-291`); the post-refactor operation is a six-level precedence resolution with a memo lookup. A 2× bound between them was written without a measurement and cannot be met by any correct implementation of §11.3.

*Decision.* **The bound moves from per-query to per-frame budget, and a deterministic cache-health assertion replaces it as the thing that actually bites.** Goal §25.6 is about frames and events, not about a micro-operation; 13 ns × ~2 000 style queries per realistic frame ≈ **26 µs**, under 0.2 % of a 16 ms budget and a small fraction of a single `Terminal::draw` diff. The 12× per-query figure is the honest price of making §11.3's precedence chain real, and it is not frame-visible.

*Change to the document.* §20.9-1's acceptance column and §16.6's `style_resolve_10k_parts` row:

| `style_resolve_10k_parts` | `Theme::row`+`gutter`, 0 allocs, ≈1.1 ns/query | **exactly 0 allocations** (R2, hard, deterministic); **cache hit rate ≥ 90 %** over the 10 k-part frame (`StyleCache::stats`, promoted to `#[cfg(feature = "testing")]`) — the memo of §11.1 A3 is the mechanism, and a broken key shows up here and nowhere else; ns **recorded** in `perf_baseline.txt` and asserted only under `PERF_STRICT=1` against that baseline × 1.2 |
| `style_resolve_per_frame` *(added)* | — | the style-resolution share of `frame_showcase_lists_120x40` is **≤ 5 %** of that frame's total ns, asserted under `PERF_STRICT=1`. This replaces §20.9-1's "ns ≤ 2× the pre-refactor `Theme::row`+`gutter` baseline", which compared a 30-field `Copy` read against a six-level precedence resolution and was unmeetable by construction. The measured ≈12× per-query cost is recorded and accepted here. |

*Change to code.* Promote `StyleCache::stats` from `#[cfg(test)]` (`theme/resolve.rs:239-242`) to `#[cfg(feature = "testing")]`, expose it as `Runtime::style_cache_stats()`, and add the hit-rate and per-frame assertions to `crates/tui/tests/perf.rs`.

---

## 3. The twelve listed deviations and the §24.4 names

| # | Deviation | Verdict | Document change required |
|---|---|---|---|
| D-1 | `crossterm` feature gates only the session; `ratatui-crossterm` non-optional | **Accept** | §22.1 consequences bullet, §16.5 `core_is_backend_free` row, §18.1 `runtime.rs` row, Appendix B.2 manifest — text in §2.1 |
| D-2 | `Id` derives structural `PartialEq/Eq/Hash/Ord` | **Accept** | §7.1 replace the manual impls; rewrite the test (§2.2) |
| D-3 | Ansi16 CIE76 | **Reject** | §21 item 29 + §11.4 — restore the categorical metric (§2.3) |
| D-4 | `Recipes::apply_mono_fallbacks(&mut self)`; mono `DISABLED` adds `DIM` | **Accept both** | §11.4: the call is `out.recipes.apply_mono_fallbacks()` (the sketch's `(&mut out)` is a borrow error). Amend the `DISABLED` row to "no gutter glyph, no marker, `fg = Role::Fg(Faint)`, all modifiers removed **and `DIM` added**" and note the reason: colour is excluded from case 9's comparison, so a colour-only disabled rule is indistinguishable from default. Also record MI-13 (fallbacks are not applied to variant maps). |
| D-5 | `Resolved.align` | **Accept** | §11.3: add `pub align: Option<Align>` to `Resolved` (`StylePatch.align` was already declared; `Resolved` was the omission) |
| D-6 | `Ui::register_editor` / `Ui::declare_state` | **Accept** | §17.0 A2: declare both. Add the invariant: *declared flags are read back through `FrameRead::state` on the **next** frame (they live in last frame's `declared` list), the same one-frame contract as `cx.area` (S3).* This matters: `focused_is_editing` (`runtime.rs:259-263`) reads last frame's flags, so a paste in the same `handle` that began an edit is not routed. |
| D-7 | `App::on_esc` + `Cx::command()` | **Accept both** | §17.0 A1: add `fn on_esc(&mut self, cx: &mut Cx<'_>) -> Response<()> { Response::ignored() }` as §3.3 step 8(c)'s application hook (the spec put the ladder on `Screen` and left `App` without one). §17.0 A2: add `fn command(&self) -> Option<ActionKey>` as the channel by which a matched `KeyMap` chord reaches `App::update` (§3.3 step 2 said "produces an app action" and declared no channel). |
| D-8 | `conformance_suite!(name => Case)` | **Accept** | §16.2: the macro cannot derive a module ident from the `NAME` const, so the ident is written explicitly. **Add a guard**: the macro must emit `const _: () = assert!(matches!(<$case as Conformance>::NAME.as_bytes(), _));` — concretely, a generated `#[test] fn name_matches_the_module() { assert_eq!(<$case>::NAME, stringify!($name)); }` so the two cannot drift. Rewrite §16.2's `conformance_suite!` invocation as `button => ButtonCase, chip => ChipCase, …`. |
| D-9 | `validate` / `secret` / `field_control` at crate root | **Accept** | Appendix B.2's tree lists them under `components/`; that is a Slice-4 directory the Slice-3 owner may not write. Amend B.2: `secret.rs`, `validate.rs`, `field_control.rs` live at the crate root (they are foundation vocabulary consumed by `components/input.rs`, not components). |
| D-10 | Rule-22 regex narrowed | **Amend, do not accept as-is** | Restore §22.7's broad regex `Color::Rgb\(|Color::from_u32\(|#[0-9a-fA-F]{6}` and add `crates/tui/src/theme/downgrade.rs` and `crates/tui/src/theme/builder.rs` to the rule's **path** allow-list (which does not feed the "`legacy_api.txt` must be empty" condition). A narrowed regex hides the exception; a named path shows it. As written, `Color::Rgb(r, g, b)` from computed values escapes anywhere in the crate. |
| D-11 | `SecretPolicy::default().mask = GlyphRole::Dirty` | **Amend** | `GlyphRole::Dirty` is the *uncommitted-changes* marker (Junie `•`, `theme/builtin/junie.rs:129`), and §11.4's mono rule already binds `MARKER + WARNING/DIRTY → GlyphRole::Dirty`. Overloading it means a theme that changes the dirty marker changes password masking. Add `GlyphRole::SecretMask` to §11.2's list (Junie `•`, i.e. the same glyph, a distinct role) and make it the default `SecretPolicy::mask`. |
| D-12 | `ThemeBuilder` param names | **Accept** | §17.0 A5 names them positionally (`selection(bg, fg)` etc.); the implementation's spellings are compatible. No change beyond confirming `focus`, `selection`, `highlight`, `field`, `disabled` are present (§21 item 21) — they are, and `derive_unset` fills the rest. Verify `theme::builder_derives_every_unset_token_deterministically` and `theme::derived_tokens_meet_design_contrast_ratios` exist in `theme/builder.rs` (not confirmed in this pass; `every_named_test_exists` must settle it). |
| D-13 | *(not on the list, found here)* `Ui::paint_spans(area, spans, base)` takes a third `base: Style` argument; `Ui::style`/`style_patched` take `&mut self` | **Accept both** | §17.0 A2: `pub fn paint_spans(&mut self, area: Rect, spans: &[Span<'_>], base: Style) -> u16` — the base is the part style the spans inherit, and without it `RowUi::label_spans` could not honour the `LABEL` recipe. §17.0 A2: `pub fn style(&mut self, …) -> Resolved` and `pub fn style_patched(&mut self, …) -> Resolved` — `&mut` is required by the §11.1 A3 memo and by the per-cell role recording `dim_layer` depends on. Record the consequence in §10: `Measure::measure(&self, ui: &Ui<'_>, …)` cannot resolve a style; a component needing one must use `ui.theme().resolve(…)` (MI-16). |

**§24.4 self-declared names — all accepted, one amendment.**
`SelectAction { Chose(ItemKey), Opened, Closed }`, `RadioGroupAction { Chose(ItemKey) }`, `ChipBarAction { Toggled(ItemKey), Closed(ItemKey), Activated(ItemKey) }` and the three state structs follow §6.1's `XAction` convention and §4's state-holds-no-props rule; none is implemented yet (they are 4A/4B/4F types) so nothing is verifiable here beyond the naming, which is correct. **Amend Appendix B.2**: `author::raw` is an inline `pub mod raw` inside `crates/tui/src/author.rs:64-66`, not a separate `author/raw.rs`. §24.1's exact Rust (`COMPONENT_ARCHITECTURE.md:4751-4753`) already shows it inline; Appendix B.2 line 3860 is the outlier and should read `author.rs   # includes the qualified-only `author::raw` escape module (§24 M1)`.

---

## 4. Part 2 — independent API review

**(a) `lib.rs` / `author.rs` against Appendix B.3/B.4 as amended by §24.**
Broadly compliant and genuinely curated. No glob re-exports; every root line is explicit (`lib.rs:48-99`). `Frame` is root-only (`lib.rs:99`) and absent from `author` — correct per §24.1. Our `Span` is at both layers; ratatui's `Line`/`Span`/`Text` appear only in `author::raw` (`author.rs:64-66`) — correct. Three deviations from B.3 item 2's "every module is `pub(crate) mod`": `pub mod layout`, `pub mod text`, `pub mod theme` (`lib.rs:32`, `:38-39`). `theme` is sanctioned by B.4; `layout` is required by §17 example 1's `use junie_tui::{…, layout, …}`; `text` is not required by anything and leaks `TextBuffer`, `grapheme_width`, `is_word_char`, `thousands` (MI-11). `author.rs:12` re-exports the whole `crate::id` module rather than `{id!, Id, ItemKey, Part, PartRef}` — a small, unintended widening. **Fix:** `pub(crate) mod text` + curated re-exports; `pub use crate::id::{Id, ItemKey, Part, PartRef}` in `author` (the `id!` macro is `#[macro_export]` and already reachable at the root).

**(b) `Ui`/`Cx` split: are the phase-limited capabilities honest?**
Yes, structurally. `Ui` has no `&mut` path to app state, no `Cx`, no `Response`, and no layer mutation — `Ui::layer` (`ui/mod.rs:455`) can only *draw into* a layer `Cx::open_layer` already assigned, and returns `None` otherwise. `Cx` has no `Buffer`, no painting method and no `Ui`. `App::draw(&self, …)` (`runtime.rs:42`) makes the compile-error claim real at the top of the stack. Two honest caveats to record: `Ui::cache` (R8) is real mutable state reachable from `draw` and is guarded only by `architecture::cache_types_are_derived_only` (`xtask/src/main.rs:906-942`, a regex heuristic, not `syn`); and `Ui::declare_state` writes flags that the *next* frame reads — non-semantic, but it is a `draw`-phase write that the runtime consumes, and §5 R2 should name it explicitly alongside `report_layout`.

**(c) `Response<A>` composition and `Intent<'f>` borrow ergonomics.**
`Response` is complete and correct (`response.rs`); `BitOr` is `Response<()>`-only as specified, and `on_action`/`on_activated`/`erase`/`map_action` compose cleanly. Two notes: `BitOr` takes `self.id.or(rhs.id)` (`response.rs:291`) where §21 item 4 says "id: lhs" — a benign improvement, worth one line in §6.1. The `Intent<'f>` split works exactly as §21 item 6 intended: `examples/12_author_component.rs:128-169` calls `cx.request_repaint()` **inside** the drain loop and compiles, which is the whole point. A realistic `update` body reads well; the only ceremony is the `BINDINGS.iter().find(…)` lookup, which `Binding::lookup` (`keymap.rs:49-54`) already provides and example 12 should use.

**(d) `runtime.rs` against §3.3 steps 1–15 — every divergence.**

| Step | Divergence | Severity |
|---|---|---|
| 1 | Resize returns early (`runtime.rs:672-675`), so `app.update` never runs and the `pending_focus` intents enqueued at `:661-668` are dropped | MA-7 |
| 2 | Correct (`:691-697`), including the bare-`Char` swallow guard | — |
| 3 | `Registry::hit` orders by registration, not by layer | MA-1 |
| 3 | Wheel is not filtered by top layer (`:416-422`) | MI-5 |
| 4 | `MouseKind::Move` under a live capture is delivered as `Phase::Drag` (`:492-494`) — sensible, undeclared | record |
| 4 | `Cx::capture` origin ≠ press position | MA-5 |
| 5 | Correct; Tab/Shift+Tab/BackTab, press-focuses-owner, and Esc correctly moved to step 8 | — |
| 6 | Correct; queue frozen, focus intents enqueued in `FocusOut`→`FocusIn` order (`:233-250`) | — |
| 7 | Re-run loop correct and bounded at 4 passes; the give-up path enqueues then discards the pending `FocusOut`/`FocusIn` | MI-7 |
| 8 | Ladder order (a)(b)(c) correct (`:710-743`), but a matching Bubble binding short-circuits (b) and (c) even when the app returns `Ignored` | MINOR |
| 9 | Correct; per-pass `UndeliveredIntent` gated on `delivers_to` | — |
| 10 | Correct; generation bump, `FrameState::reset`, per-frame `style_cache.clear()` | — |
| 11 | Correct; layer scopes armed **before** any draw (`:788-806`) — this is what makes `trap_is_armed_when_the_layer_is_pushed_not_when_it_draws` real | — |
| 12 | Correct; composite bottom-to-top after `app.draw` (`:818-821`) — defeated in practice by BL-3 | BL-3 |
| 13 | Correct; registry swap, `release_if_stale` | — |
| 14 | Correct in shape; `services.repaint = true` set here is cleared at the next `handle` (`:656`) before anyone can observe it unless the loop reads `wants_tick()` between draw and handle — `run` does (`session.rs:141`), the `Harness` does not | MINOR |
| 15 | `cursor::resolve` is exactly §8.4; the *selection* of which request reaches it is not | BL-6 |

**(e) Focus reconcile, trap arming, capture release, wheel routing, cursor rejection — verified by reading tests.**
- Reconcile (a)(b)(c)(d): implemented at `focus.rs:309-352`, all four branches exercised by `focus.rs:556-610` **and** end-to-end by `conformance/driver.rs:558-591`. Divergence MI-2 (an extra `or_else` past (d)).
- Trap armed at open: **verified** — `runtime.rs:788-806` arms scopes before `app.draw`; `focus.rs:624-636` and `conformance/driver.rs:641-645` (resize to 1×1 with the layer still open) both prove it.
- Capture release: **verified** on resize (`runtime.rs:273`, test `capture.rs:217-224`), on owner disappearance and generation mismatch (`capture.rs:71-77`, tests `:117-141`), and on layer close (`runtime.rs:548-555`, F8).
- Wheel routing: innermost-of-axis and zero-headroom both verified (`hit.rs:384-442`); boundary rule verified at the model (`scroll.rs:280-294`) and at the conformance level (`driver.rs:687-704`). Layer filtering is the gap (MI-5).
- Cursor rejection: `cursor.rs` is correct and its four §16.1 tests are honest; case 17 (`driver.rs:707-730`) exercises it end-to-end under a real popover. The selection bug is upstream of it (BL-6).

**(f) Theme precedence, merge laws, memo cache, role binding, `inherited.patch`.**
Precedence 1→6 is **wrong at step 2/3** (BL-1); 4, 5, 6 are correct and ordered as specified (`resolve.rs:62-81`, `:180-202`). Merge laws are complete and honestly tested (`patch.rs:216-279`: identity, absorption, associativity, clear, modifier symmetry, subset matching). The memo cache is a `Box<[(u64,u32,StylePatch); 256]>` (`resolve.rs:212`) — one allocation at construction, none per frame, generation-stamped rather than zeroed, exactly §20.9-2. **It deliberately omits `Surface` from the key**, which is correct and better than the document: the memo caches steps 1–5, which are role-level and surface-independent; roles bind afterwards in `bind` (`resolve.rs:180-202`). **Amend §11.1 A3 and §20.9-2** to drop `Surface` from the key and record why. `Style::patch` is used as the final layering in `theme::patch_merge_matches_ratatui_style_patch_for_modifiers` (`resolve.rs:421-446`) and the law agrees with `StylePatch::merge`; but **no production call site performs `inherited.patch(resolved.style)`** yet — that lands with the components, and §22.2 item 10 should be added to the Slice-4 per-package checklist.

**(g) `RowUi` painting with no intermediate `String`.**
Yes for `label`/`label_patched`/`meta`/`trailing`/`label_fmt` (`collection/rowui.rs:154-250`) — all route to `Ui::paint_str` = `Buffer::set_stringn`, and `CellWriter` (`:302-325`) formats straight into cells. `num`/`money` use stack buffers (`:399-410`, `:563-621`) and are tested (`:627-638`). `columns` uses `[u16; 16]` with no allocation (`:277-279`). **Not** for `label_spans` (BL-4) and **not** during `CellUi::drop`'s alignment shift (BL-3). Also note `label_in` (`:168-186`) does not pad to the full width in the label style — the row was filled with the *container* style first — which is a behavioural difference from the legacy `fit` that MA-3's weakened test cannot detect.

**(h) Is the `author` surface sufficient for example 12 and the seven prototype components?**
Example 12 compiles against `author` alone (`examples/12_author_component.rs:14-17`) — Scenario G's mechanical proof holds. For the seven prototypes:

| Prototype | Sufficient? | Missing primitive |
|---|---|---|
| Button | Yes | — |
| Field + TextInput | Almost | `Ui::style` is `&mut self` so `Measure::measure(&self, ui: &Ui, …)` cannot resolve a themed size (MI-16); `Field` needs `design.size.field_height` only, so this bites the *first* component that measures a glyph |
| List | Almost | `RowUi` is complete; `Ui::paint_spans` must stop allocating (BL-4) |
| Tabs | Yes | — |
| Dialog-as-layer | **No** | Layer sizing is unresolved (MI-9): `LayerSpec::modal`'s `min_size (0,0)` yields the whole screen, so `Dialog` must either centre itself (contradicting §9.1's "one resolver") or the spec must require `.min_size(…)`. 4F is blocked on this decision. |
| ScrollRegion | Almost | `Cx::capture` gives the wrong `origin` for thumb drag (MA-5); thumb geometry and the track round-trip are in `ScrollState` and correct |
| Any custom-family component | **No** | `Family::custom` resolves to an empty style (MA-6) |

Two further gaps the prototypes will hit: there is no `Ui` accessor for the *inherited* surface style (needed for `inherited.patch(resolved.style)`, §22.2 item 10) — add `Ui::surface_style() -> Style`; and there is no declared `Ui::scroll_region(id, part, …)` convenience (§12.2 names it, `crates/tui/src/components/scroll_region.rs` is 4E's file, but the `Ui` half is Slice 3's).

**(i) Test quality.**

*Present and honest:* `id.rs` (11/11), `intent.rs`+`event.rs` (7/7), `focus.rs` (16/16), `hit.rs` (11/11), `capture.rs` (8/8), `scroll.rs` (7/7), `layer.rs` (14/14), `cursor.rs` (4/4), `layout.rs`+`measure.rs` (8/8), `runtime::panic_hook_restores_before_delegating`, `collection/` (10/10 — `generation_stamp_skips_a_no_op_reconcile` and `cached_index_probe_hits_before_a_scan` are call-counting and genuinely prove R1), `text::width_matches_ratatui_cell_width` (differential against `set_stringn` — exemplary), `theme::ascii_border_set_is_pure_ascii`, `theme::builtin_border_sets_are_ratatui_sets`, `key_set_contains_is_binary_search` (counts comparisons, as §22.4 demanded).

*Missing:* `response::must_use_is_enforced`, `response::bitor_is_defined_only_for_unit`, `secret::is_not_clone_not_eq` (all three need `trybuild`, absent — MA-12); `text::zeroize_overwrites_before_drop` (only `secret::zeroize_clears` exists — MA-13); `ui::paint_spans_matches_row_ui_label_spans` (§16.1, §24 M1); `theme::ascii_theme_renders_without_box_drawing_glyphs` (§24 M2); the four §16.2 suite-level tests `conformance::registry::declared_parts_are_the_parts_actually_styled`, `conformance::conflicting_visible_bindings_are_reported`, `conformance::focus_transition_settles`, `conformance::draw_registers_nothing_when_it_cannot_draw`; and `architecture::every_named_test_exists` itself (MA-10), whose absence is why this list had to be assembled by hand. `theme::builder_derives_every_unset_token_deterministically` and `theme::derived_tokens_meet_design_contrast_ratios` were not located in this pass and must be settled mechanically.

*Present but passing without proving the requirement:* `theme::precedence_…` (BL-1, the strongest case), `id_equality_ignores_debug_label` (§2.2), `text::row_ui_matches_fit_for_every_fixture` (MA-3), `hit::higher_layer_shadows_lower` (MA-1), `hit::inert_below_registers_nothing`, `focus::click_only_entries_are_never_reachable`, `focus::read_only_entries_stay_in_the_ring`, `focus::restore_target_receives_keys_before_the_next_draw` (MI-3), `runtime::a_fifth_focus_pass_is_diagnosed_and_applied` (MI-7), `architecture::every_foreign_type_in_the_public_surface_is_re_exported` (MA-11), `architecture::public_items_are_documented` (a substring pin, not the `RUSTDOCFLAGS` gate — acceptable as a pin, but the name overpromises), conformance cases 9 and 12 (MA-8, MA-9).

*Standouts worth preserving:* the `Stub` app (`runtime.rs:1029-1222`) makes runtime behaviour testable without components and is the reason the layer/capture/focus integration tests exist at all; `conformance/driver.rs`'s case 20 (`bindings_match_handled_keys`, `:854-902`) sweeps a real chord universe in both directions and is exactly what §13.1 needed.

**(j) Modern-API rule violations the boundary check missed.**
R‑1…R‑20 are otherwise clean in the source I read. What the *check* misses:
1. Everything after the first `#[cfg(test)]` in a file (MA-2) — including all of `theme/resolve.rs` past line 239 and `runtime.rs` past line 1028.
2. `Color::Rgb(r, g, b)` / `Color::from_u32(var)` from non-literal arguments (MI-15 / D-10).
3. Rule 9 does not catch field assignment (`st.fg = …`), which R‑9 forbids as a layering form; the two occurrences (`ui/paint.rs:157`, `:255-256`) are construction, not layering, and are allow-listed anyway — but the rule as written cannot tell the difference.
4. `CrosstermBackend` is not a forbidden pattern anywhere (§2.1's rule 27).
5. `no_domain_vocabulary_in_the_library` (`xtask:718-726`) scans `crates/tui/src` but the regex includes `\bworkspace\b` and `\binstance\b`, which appear in ordinary architectural prose (`per-instance patch`); the check passes today only because `non_test_lines` + `code_line` strip comments, which means a `///` doc line saying "per-instance" is invisible — fragile, and it will fire the first time a doc line is reflowed. Narrow the regex or scan code lines only, deliberately.

**(k) Readability and ceremony for a downstream author.**
Good. The module docs consistently cite the section they implement, which makes the codebase navigable against the architecture — keep that discipline in Slice 4. Example 12 is 210 lines for a real, themed, focusable, hit-testable, keyboard-and-mouse component with a binding table: that is a genuine Scenario G result. Three ceremony costs a downstream author will feel, in order:
1. **A custom family gets no styling at all** (MA-6) — the first thing an author writes is `Family::custom("x")` and nothing appears. This is the single worst first-run experience in the surface.
2. **`ui.style(...)` returns a `Resolved` whose `.style` must be threaded manually to every paint call**, and `Ui::style` is `&mut self`, so the natural `ui.fill(cell, ui.style(...).style)` does not read as one expression and `Measure` cannot use it. Consider `Ui::with_part(family, variant, part, flags, |ui, r| …)` as a convenience.
3. **Two-phase props construction** is honest but verbose; §13's "props are built once" helper convention (`fn orders_list() -> List<…>`) is the right answer and must be in the authoring guide from day one, not discovered.

---

## 5. Ordered fix list for a Fable correction pass

Serial; each step is independently testable. F1–F7 are the blockers.

1. **F1** `theme/resolve.rs` + `theme/recipe.rs`: split `PartRecipe::apply` into `apply_base`/`apply_states`; reorder `accumulate` to `family.base → variant.base → merged state rules`. Rewrite the "3 over 2" arm of `precedence_…` with a role that differs from the variant's; add `state_rules_beat_a_variant_base`.
2. **F2** Delete `unreachable_cache` (`ui/mod.rs:609`) and `unreachable_entry` (`theme/recipe.rs:247`); restructure `PartMap::entry`, `Recipes::get_mut`, `Recipe::variant_mut`, `Ui::cache`. Add `xtask` rule 27a forbidding `loop {` + `spin_loop` in `crates/tui/src`.
3. **F3** Add `Ui::buffer_in(area)`; use it from `CellUi::drop` and `RowUi::raw`. Add `layer::composite_copies_only_painted_cells` and `ui::dim_layer_uses_the_role_of_the_painted_cell`.
4. **F4** Rewrite `Ui::paint_spans` over `Buffer::set_span`; amend §22 R‑3 and §17.0 A2. Add `ui::paint_spans_matches_row_ui_label_spans` with a 0-allocation assertion.
5. **F5** Restore the legacy `nearest_16`; update `downgrade_is_deterministic_per_level`; add `ansi16_preserves_hue_family_and_brightness`. Amend §21 item 29 and §11.4.
6. **F6** `Ui::set_cursor`: keep by `(layer, owner-is-focused)`. Add `cursor::the_focused_owners_write_wins_on_the_same_layer`.
7. **F7** Record `(Id, Family, Variant, Part, Resolved)` under `testing`; make `Runtime::resolved` return the recorded value. Add `harness::resolved_reports_the_family_the_component_actually_queried`.
8. **F8** `Registry::hit` orders by `(layer, index)`; strengthen `hit::higher_layer_shadows_lower` and add the reversed-registration case (MA-1).
9. **F9** `xtask non_test_lines`: skip the `#[cfg(test)]` item, not the file tail (MA-2). Re-run the whole rule set and fix anything it newly reports.
10. **F10** Rewrite `text::row_ui_matches_fit_for_every_fixture` against the legacy grapheme-walking `fit`, without the non-ASCII skip and without trimming padding (MA-3).
11. **F11** Add `trybuild`, `crates/tui/tests/ui/`, and the three compile-fail cases (MA-12).
12. **F12** Add `architecture::every_named_test_exists` (one-directional and scoped per §21 item 28) plus `conformance_covers_every_public_component`, `state_override_is_used_only_in_apps_and_fixtures`, `all_examples_compile`. Fix whatever it reports (§4(i)'s missing list).
13. **F13** Add the four §16.2 suite-level tests.
14. **F14** Neutral fallback recipe for a custom family (MA-6); update example 12's expectations and §11.2.
15. **F15** `Cx::capture` origin = press position (MA-5). Add `capture::origin_is_the_press_position`.
16. **F16** Resize path runs `app.update` (MA-7); give-up path delivers the focus pair via `pending_focus` (MI-7).
17. **F17** Conformance cases 9 and 12 strengthened (MA-8, MA-9).
18. **F18** Adjudication changes 4, 6, 8: split `fit_…` into inline/wide; probe-count assertions for `intents_drain_…`; cache-hit-rate + per-frame budget for `style_resolve_…`. Re-bless `perf_baseline.txt` in the same commit with a note.
19. **F19** Adjudication 5: `dependency_graph_is_exactly_the_declared_set` gains the inverted-tree (2c) assertion; `critical-section`/`palette` move to the unpruned closure.
20. **F20** Adjudication 1: `xtask` rules 27/`ratatui_crossterm_is_named_in_exactly_two_files`.
21. **F21** Facade tidy: `pub(crate) mod text` + curated re-exports; `author` re-exports `id::{Id, ItemKey, Part, PartRef}` instead of the module (MI-11).
22. **F22** `GlyphRole::SecretMask` (D-11); `zeroize` `black_box` + renamed test (MA-13); rule-22 regex restored with named path exceptions (D-10).
23. **F23** `doc-check` covers §24; move the legacy names out of `foreign_members()` into `doc_check_allow.txt` (MA-14).
24. **F24** Decide MI-9 (layer sizing) before 4F starts; record the decision in §9.1.
25. **F25** Remaining MINORs: MI-1, MI-2, MI-3, MI-4, MI-5, MI-6, MI-8, MI-10, MI-12, MI-13, MI-14, MI-16.
26. **F26** Apply every document amendment in §2 and §3 to `COMPONENT_ARCHITECTURE.md`, and mirror the adjudications in `REFACTORING_STATE.md` (change-control rule, `COMPONENT_ARCHITECTURE.md:3`).

---

## 6. Slice 3 gate — exact acceptance conditions

The Appendix A Slice 3 gate stands, with these additions. All commands from the workspace root.

```bash
# — the recorded Appendix A gate, unchanged —
cargo fmt --all --check
cargo clippy -p tui-next -p tui-next-testing --all-targets --all-features -- -D warnings
cargo test  -p tui-next -p tui-next-testing --all-targets --all-features
cargo test  -p tui-next --doc
RUSTDOCFLAGS="-D warnings" cargo doc -p tui-next --all-features --no-deps
cargo build -p tui-next --examples
cargo test  -p tui-next --test architecture
cargo test  -p tui-next --test perf --release -- --test-threads=1
cargo run   -p xtask -- doc-check
cargo check -p tui-next --no-default-features
cargo +1.88.0 check --workspace --all-targets --all-features
cargo test  --all-targets                      # the legacy root package: all 198 tests stay green

# — added by this review; each must pass before Slice 4 begins —

# F1: precedence is real, not a colour coincidence
cargo test -p tui-next --lib theme::resolve::tests::precedence_family_then_variant_then_state_then_global_then_scope_then_instance
cargo test -p tui-next --lib theme::resolve::tests::state_rules_beat_a_variant_base

# F2: no hang-instead-of-panic path survives
! rg -n 'spin_loop' crates/tui/src
cargo test --workspace --test architecture no_unreachable_spin_loops

# F3/F4: the written-cell bitset and the row path
cargo test -p tui-next --test render layer::composite_copies_only_painted_cells
cargo test -p tui-next --test perf --release -- --test-threads=1 paint_spans

# F5: DESIGN.md:320 is the contract
cargo test -p tui-next --lib theme::downgrade::tests::ansi16_preserves_hue_family_and_brightness
cargo test --all-targets theme::tests::accent_survives_downgrade   # the legacy pin, still green

# F6/F7: cursor selection and the theme-coupling migration contract
cargo test -p tui-next --lib cursor::tests::the_focused_owners_write_wins_on_the_same_layer
cargo test -p tui-next --all-features harness::resolved_reports_the_family_the_component_actually_queried

# F9: the boundary check scans whole files
cargo run -p xtask -- boundary                 # must report `ok` for all checks, allow-lists printed and empty
test -s crates/tui/tests/allow/legacy_api.txt && exit 1   # legacy_api.txt is empty

# F11/F12/F13: the named-test inventory is machine-checked
cargo test --workspace --test architecture every_named_test_exists
cargo test --workspace --test architecture conformance_covers_every_public_component
cargo test -p tui-next --test conformance conformance::registry::
cargo test -p tui-next --test conformance conformance::focus_transition_settles
cargo test -p tui-next --test conformance conformance::conflicting_visible_bindings_are_reported
cargo test -p tui-next --test conformance conformance::draw_registers_nothing_when_it_cannot_draw

# F19/F20: the dependency story is asserted, not assumed
cargo test --workspace --test architecture dependency_graph_is_exactly_the_declared_set
cargo test --workspace --test architecture ratatui_crossterm_is_named_in_exactly_two_files
cargo tree -p tui-next -e normal --invert smallvec   # every path passes through ratatui-crossterm

# F18: the perf contract, re-blessed in the same commit as the code change
PERF_STRICT=1 cargo test -p tui-next --test perf --release -- --test-threads=1
git diff --exit-code crates/tui/tests/perf_baseline.txt   # blessed deliberately, reviewed in the diff

# F26: the document and the state ledger carry the eight adjudications
rg -n 'amended by the Slice 3 foundations review' COMPONENT_ARCHITECTURE.md
rg -n 'Slice 3 foundations review' REFACTORING_STATE.md
```

**Gate pass condition.** Every command above exits 0; `crates/tui/tests/allow/legacy_api.txt` and `crates/tui/tests/allow/domain.txt` are empty; `xtask boundary` prints `ok` for every check including the three new ones; `every_named_test_exists` reports no missing name from §16.1, §16.2's suite-level list and §16.4; and `COMPONENT_ARCHITECTURE.md` carries the amendments to §7.1, §9.1, §10, §11.1 A3, §11.2, §11.4, §16.2, §16.6, §17.0 A1/A2/A5, §17 example 9, §18.1, §20.9-1/-2, §21 items 4/11/29, §22.1, §22.7, §24.1 and Appendix B.2 listed above, each mirrored in `REFACTORING_STATE.md`.

**Then, and only then, Slice 4 wave 1 (4A, 4B, 4C, 4E, 4G) may start.** MI-9 (layer sizing) must additionally be decided before 4F is scheduled, and MI-16 (`Measure` cannot resolve styles) before 4A.
