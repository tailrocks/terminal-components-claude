# Legacy test disposition — the per-test record that Slice 5's deletion destroys

Slice 5 deletes the root package's `src/`, its `[lib]` and its three `[[bin]]`s. That
deletion also destroys the only evidence from which a per-test disposition could be
written. This file is that record, made **before** the deletion.

`COMPONENT_ARCHITECTURE.md` §16.4 accounts for the application *integration* tests by
name (26 showcase, 23 tablepro, 28 jackin). §16.1/§16.2 declare the new library tests.
**Nothing mapped an old library test to a new one, or to a reasoned deletion.** That is
what this file supplies, for all five targets.

An aggregate argument — "the new suite has more tests than the old one" — is not a
record and is not accepted here. Every row below names a successor test, a destination
file, or a reason.

---

## 0. Measured per-target counts

Measured on this tree, `2026-09-04`, with the workspace under concurrent edit. The root
package (`junie-tui`) built and listed cleanly; no target had to be read from source
because it would not build.

```
cargo test -p junie-tui --lib             -- --list | grep -c ': test$'   →  76
cargo test -p junie-tui --bin showcase    -- --list | grep -c ': test$'   →  33
cargo test -p junie-tui --bin tablepro    -- --list | grep -c ': test$'   →  41
cargo test -p junie-tui --bin jackin-preview -- --list | grep -c ': test$' → 67
cargo test -p junie-tui --test perf       -- --list | grep -c ': test$'   →  30
```

| target | count |
|---|---|
| `--lib` (`junie_tui`) | 76 |
| `--bin showcase` | 33 |
| `--bin tablepro` | 41 |
| `--bin jackin-preview` | 67 |
| `--test perf` | 30 |
| **total** | **247** |

**They sum to exactly 247.** The recorded `76 + 67 + 33 + 41 + 30 = 247` split is
correct, and this is the first time it has been attributed to targets: the 67 is
`jackin-preview` and the 33 is `showcase`, not the other way round.

### 0.1 A second accounting gap, found while attributing the targets

The recorded application counts are **not** the target counts. §16.4 retains "26
showcase, 23 tablepro, 28 jackin (22 + 6 chrome) plus the in-module
`rain`/`arbiter`/`clock`/`scenario` unit tests" — 87 tests. The three bin targets hold
**141**. §16.6 names all 17 in-bin `perf_tests` and the Slice 6/7 gates name the two
`visual_tests`, which accounts for 19 more. That leaves **35 tests inside the three
binaries with no named destination anywhere**:

* `tablepro`: `model::tests` (4), `sql::tests` (6) — 10
* `jackin-preview`: `domain::*::tests` (12), `sim::*::tests` (10), `screens::inspect::tests` (3) — 25

They are at far lower risk than the library tests, because Slices 6 and 7 own
`apps/<app>/**` *in full* and the modules they live in move with the app rather than
being deleted. They are enumerated below anyway, because "moves with the tree" is a
claim that should be written down once rather than assumed 35 times.

---

## 1. THE IMPORTANT SECTION — legacy assertions with no successor

These are properties the legacy suite asserts that the new suite does **not**. Each is a
hole in `crates/tui`'s coverage that only this exercise surfaces. They are listed first
because they are the entire point of the file.

### GAP-1 — a clickable `Brand` is never exercised by any test in `crates/tui`

Legacy: `widgets::brand::tests::clickable_lockup_registers_and_lifts_on_hover`
asserts that `Lockup::render_clickable` (a) registers a hit region that resolves to the
lockup's own id and (b) paints `accent_hover` under the pointer.

`crates/tui/src/components/brand.rs` has the successor feature — `.clickable(bool)`,
`Focusability::ClickOnly`, `Response<Activated>`, a `PartRef::of(Part::LABEL)` region —
and its rustdoc describes all of it.

**Nothing sets it.** `grep -rn 'clickable' crates/tui/tests crates/tui/examples` returns
three hits, none of which is a call: two prose comments and one unrelated doc line.
`BrandCase` in `crates/tui/tests/conformance.rs` declares `Caps::empty()` and
`mono_narrowing_reason() = "…Brand is a stateless brand surface"`, so every conformance
case that could exercise it is narrowed away; `draw_brand` in
`crates/tui/tests/render_components.rs` never calls `.clickable`.

So the entire clickable branch of `Brand::update`, `Brand::draw`'s registration and its
`HOVERED`/`PRESSED` painting are **unexecuted by any test**, and the legacy test that did
execute them is about to be deleted. Impact: a regression that made `.clickable(true)`
register nothing would be invisible.

### GAP-2 — nothing asserts that `DISABLED` absorbs `HOVERED`

Legacy: `theme::tests::disabled_button_ignores_hover` asserts
`t.button(Primary, {disabled}) == t.button(Primary, {disabled, hovered})`.

In the new theme this property is not a rule anywhere; it is an **emergent consequence of
declaration order** in `crates/tui/src/theme/builtin/mod.rs`. For `Family::BUTTON` /
`Variant::PRIMARY` the recipe declares `.when(HOVERED, set_bg(AccentHover))` before
`.when(DISABLED, set_fg(DisabledFg).set_bg(DisabledBg).remove(BOLD))`. Both rules are
single-flag, so `PartRecipe::states` stores them at equal specificity
(`theme::recipe::tests::state_rules_are_stored_in_specificity_order`) and ties break by
declaration order (`theme::recipe::tests::state_rules_tie_break_by_declaration_order`).
A live state of `DISABLED | HOVERED` therefore applies both, and `DISABLED` wins only
because it was written second.

**Swapping those two `.when` calls silently restores the hover colour on a disabled
button, and no test in the tree fails.** `conformance::*::disabled_cannot_activate` tests
*behaviour*, not style; `mono_states_are_distinguishable` runs under `Mono` only and does
not compare `DISABLED` against `DISABLED | HOVERED`.

This is the highest-value finding in the file: a real accessibility/visual rule, held up
by nothing but source ordering, about to lose its only test.

### GAP-3 — "focus does not move the plane, it adds weight" is unasserted

Legacy: `theme::tests::hover_and_focus_are_distinct_styles` asserts three clauses:
`base.bg != hovered.bg`, **`base.bg == focused.bg`**, and `focused` is `BOLD` while
`hovered` is not.

New coverage: `theme::tests::field_raises_to_field_hover` covers the first clause for
`Field`, and `render::components::button::{default,hovered,focused}` record three
distinct digests. A digest proves the three renderings *differ*; it does not encode
*which* attribute differs. The clause "focus keeps the plane and only adds `BOLD`" — the
one that distinguishes the design system's focus treatment from its hover treatment — has
no named assertion.

### GAP-4 — the hint-bar overflow marker is no longer asserted

Legacy: `widgets::hintbar::tests::narrow_rows_drop_from_the_right_and_mark_it` asserts
that a truncated hint row **contains `…`**, and that a wide row does not.

New: `components::hintbar::tests::narrow_rows_drop_hints_from_the_right` asserts the
*count* of fitting hints and the used width. It never renders, so it cannot see the
marker. Nothing else asserts it. A `HintBar` that silently dropped hints with no
overflow affordance would pass.

### GAP-5 — the meter's colour ladder is unasserted

Legacy: `widgets::progress::tests::line_mode_draws_runs_with_the_level_colour` renders at
50 %, 70 % and 95 % and asserts the run's `fg` is `text_secondary`, `warning` and `error`
respectively — i.e. that the *tone* actually reaches the *paint*.

New: `components::meter::tests::tone_follows_the_design_thresholds_not_a_hard_coded_match`
asserts `ratio → MeterTone`, and `every_tone_names_a_meter_role` asserts
`MeterTone → MeterRole`. Neither renders. The last link — `MeterRole → the painted cell` —
is covered only by the `render::components::meter::*` digests, which all use one ratio,
so the ladder is exercised at exactly one rung.

### GAP-6 — `MeterVisual::Block` is drawn by nothing, and the code says so

Legacy: `widgets::progress::tests::block_mode_fills_the_used_share_as_background` is the
only test of block mode: bar filled to the used share as a *background*, value text
inside the fill, on-fill foreground inverted.

`crates/tui/src/components/meter.rs`'s own rustdoc records the gap verbatim:

> nothing draws `MeterVisual::Block`, nothing calls `.tone(…)`, and nothing draws a meter
> without a `.ratio`, so the `Stale`, `Unknown` and `Series` runs, the block mode's
> `OnAccent` overlay and the value-only path have no coverage at all.

The library documents an untested code path and the only test that covered it is in the
tree about to be deleted. This is a documented gap becoming an undocumented one.

### GAP-7 — `MeterTone::{Stale, Unknown}` keep their token but lose their test

Legacy `widgets::progress::tests::domain_states_render_their_markers` covers six tones.
Three (`Warning`, `Exhausted`, `Refreshing`) are moved to jackin by §18.2 and are Slice 7
work. Two (`Stale`, `Unknown`) are **retained in the library** — they are variants of the
new `MeterTone` and have `MeterRole::Stale` / `MeterRole::Unknown` tokens — but per
GAP-6's citation nothing draws them. The retained half of this test has no successor.

### GAP-8 — `StatusBar` placement geometry is unasserted

Legacy: `widgets::statusbar::tests::groups_keep_their_order_and_sides` asserts the actual
placed coordinates — left group starts at `x = 1`, the right group's last item ends flush
at `width − 1`, the centre group sits strictly between the two, and within a group items
run left-to-right in declaration order.

New: `components::status::tests::a_wide_row_keeps_every_item` asserts only the survivor
**bitmask** `[0b11, 0b1, 0b111]`. `an_item_reports_its_own_columns` asserts widths. No
test asserts where the three groups land. A `StatusBar` that right-aligned the left group
would keep every item and pass.

### GAP-9 — the last surviving status item is no longer required to be truncated

Legacy: the same narrow-row test asserts the one surviving item is `…`-truncated and
`width <= 14` in a 16-column row. New
`components::status::tests::narrow_rows_drop_centre_then_right_then_left_and_keep_the_name`
asserts which item survives, not that it is made to fit. A survivor that overflowed its
row would pass the successor.

### GAP-10 / GAP-11 — the tab strip's two structural rules are digest-only

Legacy `widgets::tabs::tests::active_tab_has_a_plane_and_the_only_accent_underline_and_no_gutter`
asserts three named rules: the active tab sits one plane up, **only** the active tab
carries the accent `━` rule (inactive carries `─`), and a tab strip contains no `▎`
gutter glyph. `hover_and_cursor_differ_from_active` asserts the three-plane ladder
(inactive → hover/cursor at two lifts → active at one lift) and that the keyboard cursor
is `BOLD` while hover is not.

`render::components::tabs::{default,focused,hovered,pressed,selected}` are blessed
digests and will catch a change — but a digest cannot say *what* rule broke, and cannot
distinguish "hover and active are both lifted, indistinguishably" from a legitimate
restyle. `components::tabs::tests` covers reconciliation and the mono `PRESSED` bracket
only. The three named rules are now unnamed.

### GAP-12 — `StatusBar` hover has a successor, and it is red and uncommitted

`widgets::statusbar::tests::render_fills_the_row_and_registers_hover` maps to
`crates/tui/tests/status_bar_hover.rs::only_the_hit_status_item_lifts_and_keyboard_suppression_clears_hover`,
which is a **better** test than the legacy one. But it currently fails:

```
test result: FAILED. 0 passed; 1 failed
```

and per `COORDINATION.md` Incidents 3 and 4 it is deliberately left untracked and
uncommitted. If Slice 5 deletes the legacy test while this one is still red and untracked,
the `StatusBar` hover/registration property has **zero** coverage in the tree.

### GAP-13 — a legacy perf assertion that has never executed

`perf::no_full_collection_clone_per_frame` gates both of its assertions behind
`if env_flag("PERF_TARGET")`. §16.6 records `PERF_TARGET` as folded into `PERF_STRICT`
(MI-14) and nothing in the repository sets it, so the `bytes/frame < 64 KiB` bound this
test is named for **has never been asserted**. Seven of the 30 root perf tests are gated
the same way. §16.6 keeps the test by name with the same threshold; whoever migrates it
must make the threshold live, or it migrates as decoration.

### GAP-14 — sequencing: 32 library tests have destinations that do not exist yet

Slice 5 (which deletes `src/`) is planned **after** Slice 4 wave 2, so on paper every
destination exists first. Today it does not: `crates/tui/src/components/` contains no
`grid.rs`, `menu.rs`, `picker.rs`, `tree.rs`, `steps.rs`, `viewport.rs`, `diff.rs`,
`split.rs` or `panel.rs`. The 32 library tests dispositioned "MIGRATES IN SLICE 4"
below, plus 9 of the 30 perf tests, are the ones that vanish uncovered if Slice 5 runs
early. **This file is the precondition for that not being silent; it is not a
substitute for the ordering constraint.**

---

## 2. `--lib` — the 76 legacy library unit tests

All 76 rows were produced by reading the test bodies in the legacy tree and the named
successor in `crates/tui`. Where a claim is inferred rather than executed, the row says so.

### 2.1 `core::event` (2)

| module path + test name | what it asserts | disposition |
|---|---|---|
| `core::event::tests::outcome_combines_with_changed_dominating` | `Outcome::or` is a lattice — `Changed > Consumed > Ignored` — and `Ignored.consumed()` is false | **ALREADY DUPLICATED** — the 3-value `Outcome` split into `Flow` × `Invalidate`; `response::tests::bitor_takes_consumed_over_ignored`, `response::tests::bitor_takes_max_invalidate`, `response::tests::repaint_raises_relayout_raises_further` |
| `core::event::tests::key_helpers` | `Key::ctrl_char('a')` discriminates on `CONTROL`; `Key::is_char` rejects a modified key; a `SHIFT`-only key is `plain()` and `is_char('A')` | **ALREADY DUPLICATED** — `event::tests::chord_matches_shifted_chars_and_display_is_readable` (`Key::is` tolerates `SHIFT` only, `bare_char`), `event::tests::chord_hashes_by_code_and_mods` (`Chord::key('a')` and `Chord::with('a', CONTROL)` are distinct) |

### 2.2 `core::focus` (3)

| module path + test name | what it asserts | disposition |
|---|---|---|
| `core::focus::tests::tab_cycles_forward_and_backward` | `Focus::next` walks registration order and wraps; `prev` is the reverse | **ALREADY DUPLICATED** — `focus::tests::tab_cycles_forward_and_backward` (same name), `focus::tests::shift_tab_is_the_exact_reverse` |
| `core::focus::tests::barrier_traps_focus_and_restores` | after `push_barrier`, traversal is confined to entries above the barrier and entries below are no longer `contains`ed | **ALREADY DUPLICATED** — `focus::tests::trap_confines_traversal_to_the_scope`, `focus::tests::trap_wraps_inside_the_scope`, `focus::tests::scope_restore_returns_focus_to_the_opener`, `focus::tests::trap_is_armed_when_the_layer_is_pushed_not_when_it_draws` |
| `core::focus::tests::ensure_valid_falls_back_to_first` | focus on an id absent from the ring resolves to the ring's first entry | **ALREADY DUPLICATED** — `focus::tests::reconcile_falls_back_to_scope_first_enabled`, `focus::tests::reconcile_yields_none_when_nothing_is_reachable`, `focus::tests::reconcile_prefers_nearest_surviving_entry_by_previous_index` |

### 2.3 `core::hit` (3)

| module path + test name | what it asserts | disposition |
|---|---|---|
| `core::hit::tests::topmost_wins` | a later, smaller region shadows an earlier larger one at the overlap; a point outside every region is `None` | **ALREADY DUPLICATED** — `hit::tests::last_registration_wins`, `hit::tests::empty_rects_are_rejected` |
| `core::hit::tests::barrier_shadows_lower_regions` | after `push_barrier`, a point over a pre-barrier region hits nothing | **ALREADY DUPLICATED** — `hit::tests::higher_layer_shadows_lower`, `hit::tests::a_lower_layer_region_registered_later_does_not_shadow_a_higher_one`, `layer::tests::modal_pushes_a_trap_and_a_pointer_barrier` |
| `core::hit::tests::scroll_only_regions_ignore_hover` | a scroll-only region answers `hit_scroll` and never `hit` | **ALREADY DUPLICATED** — `hit::tests::hit_scroll_skips_regions_that_do_not_handle_the_axis`, `hit::tests::hit_scroll_returns_the_innermost_handler_of_the_axis`, `hit::tests::hit_scroll_returns_a_region_at_zero_headroom` |

### 2.4 `core::id` (1)

| module path + test name | what it asserts | disposition |
|---|---|---|
| `core::id::tests::ids_are_stable_and_distinct` | equal paths are equal ids; different paths, different `child(i)` and different `sub(s)` are all distinct | **ALREADY DUPLICATED** — `id::tests::id_equality_is_exactly_hash_equality`, `id::tests::root_sub_part_index_item_are_all_distinct`, `id::tests::separator_prevents_concatenation_collision`, `id::tests::item_key_text_is_stable_across_runs` |

### 2.5 `core::scroll` (4) — all four survive verbatim, same names

| module path + test name | what it asserts | disposition |
|---|---|---|
| `core::scroll::tests::clamps_offset_to_content` | over-scroll in both directions clamps to `[0, max_offset]`; `page_down`/`jump_end` land correctly | **ALREADY DUPLICATED** — `scroll::tests::clamps_offset_to_content` |
| `core::scroll::tests::ensure_visible_moves_minimally` | `ensure_visible` scrolls the least amount that brings the index into view, and is a no-op when already visible | **ALREADY DUPLICATED** — `scroll::tests::ensure_visible_moves_minimally` |
| `core::scroll::tests::thumb_covers_track_proportionally` | thumb length/position are proportional; a non-overflowing state fills the whole track | **ALREADY DUPLICATED** — `scroll::tests::thumb_covers_track_proportionally` |
| `core::scroll::tests::track_position_round_trips` | `offset_for_track_pos` maps track ends to `0` and `max_offset` | **ALREADY DUPLICATED** — `scroll::tests::track_position_round_trips` |

### 2.6 `core::text` (6)

| module path + test name | what it asserts | disposition |
|---|---|---|
| `core::text::tests::insert_and_move_by_grapheme` | insert/left/home/end/delete/backspace all move and edit by grapheme, not byte (`é`) | **ALREADY DUPLICATED** — `text::buffer::tests::insert_and_move_by_grapheme` |
| `core::text::tests::selection_replaces_on_insert` | an insert over a selection replaces it and clears the selection | **ALREADY DUPLICATED** — `text::buffer::tests::selection_replaces_on_insert` |
| `core::text::tests::word_motion_and_deletion` | word-left/right skip runs of whitespace; `delete_word_left` keeps the trailing space run | **ALREADY DUPLICATED** — `text::buffer::tests::word_motion_and_deletion`, `text::buffer::tests::word_chars_are_consistent_between_buffer_and_viewport` |
| `core::text::tests::multiline_vertical_motion_keeps_column` | vertical motion preserves the goal column and reports `false` at the document edges | **ALREADY DUPLICATED** — `text::buffer::tests::multiline_vertical_motion_keeps_column` |
| `core::text::tests::single_line_rejects_newline` | a single-line buffer drops `\n` from both `insert_char` and `insert_str`; a multi buffer accepts it | **ALREADY DUPLICATED** — `text::buffer::tests::single_line_rejects_newline` |
| `core::text::tests::wide_characters_count_as_two_columns` | `cursor_pos().col` counts display columns; `offset_at` snaps into a wide cell | **ALREADY DUPLICATED** — `text::buffer::tests::wide_characters_count_as_two_columns`, `text::buffer::tests::pos_of_and_offset_at_round_trip` |

### 2.7 `theme` (3)

| module path + test name | what it asserts | disposition |
|---|---|---|
| `theme::tests::accent_survives_downgrade` | `Ansi256` maps the accent to an `Indexed`; `Ansi16` maps accent → `LightGreen`, error → `LightRed`, canvas → `Black` | **ALREADY DUPLICATED** — `theme::downgrade::tests::ansi16_preserves_hue_family_and_brightness` (all three `Ansi16` colours by name), `theme::downgrade::tests::downgrade_is_deterministic_per_level` (`Rgb → Indexed(77)`), `theme::downgrade::tests::downgrade_maps_every_token_exhaustively` (no `Rgb`/`Indexed` survives `Ansi16`) |
| `theme::tests::hover_and_focus_are_distinct_styles` | hover changes the row background, **focus does not**, and focus alone adds `BOLD` | **ALREADY DUPLICATED** — `theme::tests::field_raises_to_field_hover`, `render::components::button::{default,hovered,focused}` (three distinct digests), `conformance::button::mono_states_are_distinguishable`. **Partial: see GAP-3** — the `base.bg == focused.bg` clause has no named successor |
| `theme::tests::disabled_button_ignores_hover` | a disabled button resolves identically with and without `hovered` | **DIES WITH THE LEGACY CODE** — `Theme::button(ButtonKind, VisualState, Color)` and `VisualState` are deleted; §11.3's recipe/state-rule chain replaces them, and in the replacement the property is an unasserted consequence of `.when` declaration order in `crates/tui/src/theme/builtin/mod.rs`. **See GAP-2 — the property survives, its test does not** |

### 2.8 `ui::layout` (2)

| module path + test name | what it asserts | disposition |
|---|---|---|
| `ui::layout::tests::splits_respect_minimums_and_maximize` | the seam consumes exactly the gap; `toggle_max` gives one pane the whole rect; below the minima the first pane wins the whole rect | **ALREADY DUPLICATED** — `layout::tests::split_first_pane_wins_on_both_axes_when_minima_do_not_fit` (same three clauses plus `handle`, on both axes) |
| `ui::layout::tests::drag_moves_the_seam_and_respects_minima` | `handle` returns the seam rect; `drag_to` puts the seam under the pointer, clamped by `min_first`; `nudge` moves it by a delta | **ALREADY DUPLICATED** — `layout::tests::split_percent_is_clamped_to_5_95` (the same `drag_to(70)`, `drag_to(2) → 10`, `nudge(5) → 15` sequence on the same `101×20` rect) |

### 2.9 `ui::popup` (2)

| module path + test name | what it asserts | disposition |
|---|---|---|
| `ui::popup::tests::places_below_then_flips_then_clamps` | a popup places below its anchor, flips above when it does not fit, clamps to the right edge, and never exceeds the screen height | **ALREADY DUPLICATED** — `layer::tests::anchor_rect_flips_then_clamps`, `layer::tests::popover_flips_above_when_the_content_does_not_fit_below`, `layer::tests::fixed_size_is_clamped_never_grown` |
| `ui::popup::tests::centers_in_upper_third` | `Placement::Center` is horizontally centred and vertically in the upper third | **ALREADY DUPLICATED** — `layer::tests::anchor_screen_center_sits_in_the_upper_third` |

### 2.10 `ui::text` (3)

| module path + test name | what it asserts | disposition |
|---|---|---|
| `ui::text::tests::truncates_with_ellipsis` | `truncate` appends `…` only when it must; `fit`/`fit_right` pad left/right to a width | **ALREADY DUPLICATED** — `text::measure::tests::truncates_with_ellipsis_and_middle`; the `fit`/`fit_right` padding is now `RowUi`'s in-place write, asserted by `collection::rowui::tests::row_ui_label_writes_cells_without_an_intermediate_string` and `collection::rowui::tests::row_ui_columns_clip_to_the_row` (§20.9-6 deletes `fit`/`fit_right` from every render path) |
| `ui::text::tests::middle_truncation_and_thousands` | `truncate_middle` keeps head and tail around `…`; `thousands(1_203_338) == "1,203,338"` | **ALREADY DUPLICATED** — `text::measure::tests::truncates_with_ellipsis_and_middle` covers the middle-truncation half. **The `thousands` half is a deliberate deletion, not a gap**: `thousands` is recorded as internal (`crates/tui/src/lib.rs:88`, `crates/tui/src/author.rs:39`), and its successor `collection::rowui::format_i64` explicitly does **not** group (`format_i64(-1234) == "-1234"`, `collection::rowui::tests::in_place_number_formatting`). Digit grouping survives only in `format_money`, pinned by the same test. The one library caller that needed grouped integers was `DataGrid::position_label`, which §18.2 moves to TablePro in Slice 6 |
| `ui::text::tests::wraps_words_and_hard_wraps_long_tokens` | wrap breaks on words, hard-wraps an unbreakable token, honours `\n`, and returns one empty row for `""` | **ALREADY DUPLICATED** — `text::measure::tests::wraps_words_and_hard_wraps_long_tokens` (same name), `text::measure::tests::wrapped_rows_matches_wrap` |

### 2.11 `widgets::brand` (2)

| module path + test name | what it asserts | disposition |
|---|---|---|
| `widgets::brand::tests::lockup_is_padded_accent_and_bold` | the lockup is one pad cell either side, filled with `accent`, `text_on_accent`, `BOLD`; `width()` = label + 2, compact = label | **ALREADY DUPLICATED** — `render::components::brand::default` ×8 (two themes × two colour levels × two sizes) plus `conformance::brand::draw_stays_inside_its_area` and `conformance::brand::survives_tiny_rects_0x0_to_3x3`. **Caveat**: the successors are digests, not property assertions, and `crates/tui/tests/baselines/components.txt` currently has **no `brand` rows** — those eight tests are red (missing-baseline panics) until blessed |
| `widgets::brand::tests::clickable_lockup_registers_and_lifts_on_hover` | `render_clickable` registers a hit region resolving to the lockup id, and the hovered lockup paints `accent_hover` | **DIES WITH THE LEGACY CODE** — `Lockup::render_clickable(x, y, buf, ctx, id)` is deleted; the replacement is `Brand::clickable(bool)` + `Brand::update → Response<Activated>` + a `PartRef::of(Part::LABEL)` region. **The replacement is called by no test — see GAP-1** |

### 2.12 `widgets::diff` (4) — the whole family is Slice 4H

`crates/tui/src/components/diff.rs` does not exist yet. §18.2 retains `DiffView` behind a
`DiffSource` trait; Appendix A puts it in WP 4H (wave 2), alongside `code.rs`.

| module path + test name | what it asserts | disposition |
|---|---|---|
| `widgets::diff::tests::counts_and_headers` | `additions`/`deletions` count marker lines; `summary()` is `"+2 −2 · 2 hunks"`; hunk header is `"@@ -10,3 +10,4 @@"`; file header is `"M path"` | **MIGRATES IN SLICE 4** — `crates/tui/src/components/diff.rs` (WP 4H) |
| `widgets::diff::tests::unified_lists_every_line_with_markers` | unified mode emits header, hunk header, gutter line numbers, `-`/`+` markers, and tones the removed line `Tone::Error` on both spans | **MIGRATES IN SLICE 4** — `crates/tui/src/components/diff.rs` (WP 4H) |
| `widgets::diff::tests::review_pairs_columns_and_emphasises_the_change` | review mode pairs old/new in two columns split by `│`, bolds the changed sub-token on both sides, leaves the old column blank for an unpaired add, and draws a `────` separator between hunks | **MIGRATES IN SLICE 4** — `crates/tui/src/components/diff.rs` (WP 4H); §18.2 turns `review_lines(f, width)` into `measure` |
| `widgets::diff::tests::view_renders_and_scrolls` | the view renders the header on row 0, a wheel returns `Changed` and moves the offset, a re-render does not undo it, and `toggle_mode` switches to review | **MIGRATES IN SLICE 4** — `crates/tui/src/components/diff.rs` (WP 4H). The "re-render does not undo the wheel" clause becomes structural (`draw` is `&self`) and is additionally covered by `conformance::*::draw_twice_leaves_state_equal` |

### 2.13 `widgets::grid` (9) — split between WP 4I (library `Grid`) and Slice 6 (TablePro `GridModel`)

§18.2 deletes `CellValue`, `PendingChanges`, `UndoAction`, `default_validator`,
`cmp_cells`, the `Validator` fn pointer and `primary`/`nullable` **from the library**;
Slice 6 rebuilds them in `apps/tablepro/src/grid_model.rs`.

| module path + test name | what it asserts | disposition |
|---|---|---|
| `widgets::grid::tests::dirty_back_to_original_clears_change` | editing a cell back to its original value removes the pending change and returns the row to `Clean` | **MIGRATES IN SLICE 6** — `apps/tablepro/src/grid_model.rs` (`PendingChanges` is deleted from the library, §18.2) |
| `widgets::grid::tests::delete_removes_update_and_undo_restores` | marking a row deleted drops its pending cell edits; `undo` restores `Clean` | **MIGRATES IN SLICE 6** — `apps/tablepro/src/grid_model.rs` (`UndoAction` deleted from the library) |
| `widgets::grid::tests::insert_then_undo_shifts_nothing_else` | `insert_row` appends, marks `Inserted`, puts the cursor on the new row's first editable column; `undo` removes it and leaves nothing pending | **MIGRATES IN SLICE 6** — `apps/tablepro/src/grid_model.rs`; the cursor clause depends on `GridState`'s cursor surface, which `COORDINATION.md` records as Slice 6 **Q2, unadjudicated** |
| `widgets::grid::tests::edit_commit_validates_by_kind` | a `Number` cell rejects `"abc"` with `"Must be a number"` and stays editing; a `Bool` cell cycles without opening an editor | **MIGRATES IN SLICE 6** — validation-by-`CellKind` is the `GridModel`'s (`apps/tablepro/src/grid_model.rs`); the "bool cycles without an editor" clause is library `EditIntent`/`CellAction` and migrates in **Slice 4** to `crates/tui/src/components/grid.rs` (WP 4I) |
| `widgets::grid::tests::keys_navigate_select_and_sort_request` | arrows move the cursor; Space selects a row; `s` cycles `Asc → Desc → None` as `GridEvent::SortRequested`; with `local_sort` the permutation applies and a dirty cell key survives the reorder | **MIGRATES IN SLICE 4** — `crates/tui/src/components/grid.rs` (WP 4I). The sort **comparison** is `COORDINATION.md` Slice 6 **Q1, unadjudicated**; the dirty-key-survives-sort clause needs Slice 6's `GridModel` |
| `widgets::grid::tests::range_selection_and_copy` | Shift+arrows extend a rectangular range and `copy_text` emits TSV rows for it | **MIGRATES IN SLICE 4** — `crates/tui/src/components/grid.rs` (WP 4I) |
| `widgets::grid::tests::position_label_variants` | `"rows 1–5 of 10"`, `"rows 1–5 of 10 loaded · ~1,203,338 total"` for an estimated total, `"0 rows"` when empty | **MIGRATES IN SLICE 6** — `apps/tablepro/src/grid_model.rs`; **inferred**, not measured: §18.2 does not name `RowTotal`/`position_label` explicitly, but the label's `~1,203,338` needs the grouped-integer formatting the library deliberately dropped (row 2.10 above), which puts it app-side |
| `widgets::grid::tests::fetch_more_row_is_reachable` | with `more`, `G` reaches a synthetic trailing row and `Enter` there emits `GridEvent::FetchMore` | **MIGRATES IN SLICE 4** — `crates/tui/src/components/grid.rs` (WP 4I) |
| `widgets::grid::tests::commit_result_folds_and_drops` | `Ok` folds pending edits into the rows and drops deleted rows; `Err((row, msg))` marks that row `Error` | **MIGRATES IN SLICE 6** — `apps/tablepro/src/grid_model.rs` (`apply_commit_result` is domain) |

### 2.14 `widgets::hintbar` (2)

| module path + test name | what it asserts | disposition |
|---|---|---|
| `widgets::hintbar::tests::topmost_layer_wins_and_fallback_is_empty` | the first `Some` layer in the stack wins; an all-`None` stack yields the default layer | **ALREADY DUPLICATED** — `components::hintbar::tests::the_topmost_layer_wins_and_the_fallback_is_none` |
| `widgets::hintbar::tests::narrow_rows_drop_from_the_right_and_mark_it` | a narrow row keeps the leftmost hints, drops from the right, and **renders `…`**; a wide row keeps all five and renders no `…` | **ALREADY DUPLICATED** — `components::hintbar::tests::narrow_rows_drop_hints_from_the_right` covers the drop order and the width budget. **Partial: see GAP-4** — the `…` marker is asserted by nothing |

### 2.15 `widgets::list` (1)

| module path + test name | what it asserts | disposition |
|---|---|---|
| `widgets::list::scroll_tests::wheel_moves_the_viewport_and_keeps_the_cursor` | a wheel moves the viewport without moving the cursor; a subsequent render does **not** reset the offset; a key pulls the cursor back into the visible range; over-scroll back clamps to 0 | **ALREADY DUPLICATED** — `scroll::tests::wheel_at_the_boundary_is_consumed_without_repaint`, `scroll::tests::clamps_offset_to_content`, `scroll::tests::ensure_visible_on_next_layout_is_set_only_by_cursor_motion`, `conformance::list::wheel_at_boundary_is_consumed_without_repaint`, `conformance::list::draw_twice_leaves_state_equal`. The "render must not reset the scroll" clause is now structural — `List::draw` takes `&ListState` — and `draw_twice_leaves_state_equal` is its assertion |

### 2.16 `widgets::menu` (5) — the whole family is Slice 4F

`crates/tui/src/components/menu.rs` does not exist yet.

| module path + test name | what it asserts | disposition |
|---|---|---|
| `widgets::menu::tests::keyboard_skips_disabled_wraps_and_chooses` | Down skips a disabled row, wraps at the end; `End` jumps to the last; Enter emits `Chosen(i)`; Esc emits `Dismissed` | **MIGRATES IN SLICE 4** — `crates/tui/src/components/menu.rs` (WP 4F) |
| `widgets::menu::tests::placement_is_clamped_to_the_screen_and_flips_up` | an anchored menu clamps inside the screen and flips above the anchor; a point-anchored menu opens one row below the point | **MIGRATES IN SLICE 4** — `crates/tui/src/components/menu.rs` (WP 4F); §18.2 merges the menu's own `Placement` into the shared `Anchor`, so `layer::tests::anchor_rect_flips_then_clamps` will carry part of it |
| `widgets::menu::tests::click_selects_rows_and_outside_dismisses` | each row registers its own hit region; clicking it emits `Chosen`; clicking a disabled row does nothing; clicking outside emits `Dismissed`; a `danger` row paints `error_soft` and right-aligns its shortcut | **MIGRATES IN SLICE 4** — `crates/tui/src/components/menu.rs` (WP 4F); §18.2 turns positional `row_id(i)` into `ItemKey` |
| `widgets::menu::tests::hover_moves_the_cursor` | hovering a row moves the keyboard cursor to it | **MIGRATES IN SLICE 4** — `crates/tui/src/components/menu.rs` (WP 4F); §18.2 makes this an explicit `Intent::Pointer{Move}` in `update` rather than the render-time mutation at `menu.rs:243` |
| `widgets::menu::tests::menubar_opens_switches_and_chooses` | labels lay out right of the brand; Enter opens, Right switches menu, Enter emits `Chosen(menu, item)`; clicking a label toggles; the open popover anchors under its label; clicking the brand emits `Brand`; Esc closes | **MIGRATES IN SLICE 4** — `crates/tui/src/components/menu.rs` (WP 4F) |

### 2.17 `widgets::picker` (3)

| module path + test name | what it asserts | disposition |
|---|---|---|
| `widgets::picker::tests::wheel_scrolls_the_rows_and_survives_the_next_render` | a wheel moves the row window, the next render is byte-identical to the scrolled one, the selection does not move, and scrolling back restores the original frame | **MIGRATES IN SLICE 4** — `crates/tui/src/components/{picker,filter_list}.rs` (WP 4F) |
| `widgets::picker::tests::keyboard_navigation_pulls_the_cursor_back_into_view` | after a wheel scroll, a Down key moves the cursor to 1 **and** pulls the viewport back so the cursor is visible | **MIGRATES IN SLICE 4** — `crates/tui/src/components/picker.rs` (WP 4F); the mechanism is now `scroll::tests::ensure_visible_on_next_layout_is_set_only_by_cursor_motion` |
| `widgets::picker::tests::wheel_at_the_boundary_is_consumed_not_changed` | a wheel with no headroom returns `Consumed`, not `Changed` | **ALREADY DUPLICATED** — `scroll::tests::wheel_at_the_boundary_is_consumed_without_repaint`, and the per-component driver `conformance::<case>::wheel_at_boundary_is_consumed_without_repaint`, which will cover `Picker` the moment WP 4F registers it. This is the specific `picker.rs:142-145` violation §18.2 records as fixed |

### 2.18 `widgets::progress` (4)

| module path + test name | what it asserts | disposition |
|---|---|---|
| `widgets::progress::tests::levels_follow_the_shared_thresholds` | `MeterLevel::of` uses the shared `< 60 / < 85 / else` thresholds; `MeterTone::Stale.level(_)` is `None` | **ALREADY DUPLICATED** — `components::meter::tests::tone_follows_the_design_thresholds_not_a_hard_coded_match` (the same six boundary values, plus a moved-threshold case the legacy test lacked), `components::meter::tests::every_tone_names_a_meter_role` |
| `widgets::progress::tests::line_mode_draws_runs_with_the_level_colour` | line mode draws `━`/`─` runs, right-aligns the value, and colours the run `text_secondary` at 50 %, `warning` at 70 %, `error` at 95 % | **DIES WITH THE LEGACY CODE** — the free `Meter::render(area, buf, ctx, bg)` and `MeterLevel` are deleted; the replacement resolves `MeterTone → MeterRole → Resolved`. **The three-rung ladder is re-asserted nowhere — see GAP-5** |
| `widgets::progress::tests::block_mode_fills_the_used_share_as_background` | block mode fills the used share as a background, puts the value text inside the fill, and inverts the foreground over the fill | **DIES WITH THE LEGACY CODE** — same deleted render entry point. `MeterVisual::Block` is retained in `crates/tui/src/components/meter.rs`, and that file's own rustdoc records that **nothing draws it**. **See GAP-6** |
| `widgets::progress::tests::domain_states_render_their_markers` | six tones render their markers: `Warning → ▲`, `Exhausted → !` bold, `Stale → text_faint`, `Error → "read failed !"` with no run, `Unknown → —`, `Refreshing → "refreshing"` | **MIGRATES IN SLICE 7** — `apps/jackin-preview/**`; §18.2's `progress` row moves `MeterTone::{Warning, Exhausted, Stale, Refreshing}` (the jackin quota lifecycle) out of the library. **The `Stale` and `Unknown` halves stay in the library and lose their test — see GAP-7** |

### 2.19 `widgets::statusbar` (3)

| module path + test name | what it asserts | disposition |
|---|---|---|
| `widgets::statusbar::tests::groups_keep_their_order_and_sides` | six items place into three groups; left starts at `x = 1`; the right group ends flush at `width − 1`; items within a group run left-to-right; the centre sits strictly between left and right | **DIES WITH THE LEGACY CODE** — `StatusBar::layout(Rect) -> Vec<Placed>` and the `Placed { x, width, group, index, text }` type are deleted; the replacement `StatusBar::survivors(...)` returns bitmasks and never geometry. **The placement rules are re-asserted nowhere — see GAP-8** |
| `widgets::statusbar::tests::narrow_rows_drop_center_then_right_then_left_and_keep_the_name` | the centre group leaves first, then the right, then the low-priority left items; the strongest left item always survives and is truncated with `…` to fit | **ALREADY DUPLICATED** — `components::status::tests::narrow_rows_drop_centre_then_right_then_left_and_keep_the_name` (same drop order, same "identity stays" clause), `components::status::tests::ties_take_the_later_item_first`. **Partial: see GAP-9** — the survivor's `…`-truncation and width bound have no successor |
| `widgets::statusbar::tests::render_fills_the_row_and_registers_hover` | every cell outside an item keeps the strip plane; a clickable chip registers a hit region under its id; the hovered chip paints one plane up; the strong left item is `BOLD` | **ALREADY DUPLICATED** — `crates/tui/tests/status_bar_hover.rs::only_the_hit_status_item_lifts_and_keyboard_suppression_clears_hover` covers registration, the hover lift, per-item isolation and keyboard hover suppression (more than the legacy test did). **Conditional: see GAP-12** — that test is currently **failing** and is deliberately uncommitted |

### 2.20 `widgets::steps` (1)

| module path + test name | what it asserts | disposition |
|---|---|---|
| `widgets::steps::tests::frontier_and_counts` | `frontier()` is the first non-terminal step; `counts()` reports (done, skipped, failed); a failed step is reported by `failed()` and moves the frontier past it | **MIGRATES IN SLICE 4** — `crates/tui/src/components/steps.rs` (WP 4C); §18.2 keeps `Steps` a display rail with a frontier and moves the step *flow* to the separate `Wizard` (J7) |

### 2.21 `widgets::table` (6) — `DataTable` is deleted; `Grid` with `NavUnit` absorbs it

§18.2's `table` row: "**remove** — absorbed by `Grid` with `NavUnit::{Row, Cell}`".

| module path + test name | what it asserts | disposition |
|---|---|---|
| `widgets::table::tests::sort_cycles_asc_desc_none` | clicking one column three times cycles `Asc → Desc → None` and restores the identity permutation | **MIGRATES IN SLICE 4** — `crates/tui/src/components/grid.rs` (WP 4I); the same cycle is the legacy grid's `keys_navigate_select_and_sort_request` clause, so one successor covers both |
| `widgets::table::tests::numeric_sort_is_not_lexicographic` | a column declared numeric sorts `1, 2, 10`, not `1, 10, 2` | **MIGRATES IN SLICE 6** — `apps/tablepro/src/grid_model.rs`; §18.2 deletes `cmp_cells` and the string sort from the library, so the comparison is the `GridModel`'s. `COORDINATION.md` Slice 6 **Q1** ("on what comparison") is unadjudicated |
| `widgets::table::tests::sort_keeps_cursor_on_same_row` | after a sort the cursor still addresses the same *source* row | **MIGRATES IN SLICE 4** — `crates/tui/src/components/grid.rs` (WP 4I). This is exactly `COORDINATION.md` Slice 6 **Q2** ("what `GridState` exposes; … three migrated tests need the cursor"), unadjudicated |
| `widgets::table::tests::edit_commit_and_cancel` | `begin_edit` opens an editor seeded with the cell; commit writes the cell and emits `Committed { row, col }`; cancel discards | **MIGRATES IN SLICE 4** — `crates/tui/src/components/grid.rs` (WP 4I), via `GridEditor` reachable only from `Grid::update_editable` (§23 K2) |
| `widgets::table::tests::validation_blocks_commit` | a rejecting validator keeps the editor open and exposes the message via `edit_error()` | **MIGRATES IN SLICE 6** — `apps/tablepro/src/grid_model.rs`; the `Validator` fn pointer is deleted from the library (§18.2) and validation becomes a `GridModel` responsibility |
| `widgets::table::tests::tab_moves_to_next_editable_cell_and_leaves_at_end` | Tab commits and advances to the next editable cell, still editing; Tab past the last cell emits `LeaveForward` and stops editing | **MIGRATES IN SLICE 4** — `crates/tui/src/components/grid.rs` (WP 4I); `NavUnit::Cell` plus the `EditIntent` table |

### 2.22 `widgets::tabs` (3)

| module path + test name | what it asserts | disposition |
|---|---|---|
| `widgets::tabs::tests::active_tab_has_a_plane_and_the_only_accent_underline_and_no_gutter` | the strip contains no `▎` gutter glyph; the active tab is one plane up and the inactive stays on the strip plane; **only** the active tab carries `━` in the accent, inactive tabs carry `─` | **DIES WITH THE LEGACY CODE** — `Tabs::areas: Vec<Rect>` (the per-frame `Vec` §18.2 deletes) is the only way this test reads a tab's rect, and `tabs.rs`'s free `render(area, buf, ctx, bg)` is replaced by `Tabs<'a,T,K,R>::draw`. The rules survive in `crates/tui/src/components/tabs.rs` (`GlyphRole::{RuleActive, RuleQuiet}`) and are covered only as digests. **See GAP-10** |
| `widgets::tabs::tests::hover_and_cursor_differ_from_active` | hover and keyboard cursor sit **two** planes up while active sits one; the cursor is `BOLD` and hover is not; neither carries the accent rule | **DIES WITH THE LEGACY CODE** — same reason (`areas` + free `render`). The three-plane ladder survives in the recipe and is covered only as digests. **See GAP-11** |
| `widgets::tabs::tests::suffix_state_glyph_renders_after_the_label` | `TabItem::suffix("▶")` renders after the label as `"Claude ▶"` | **DIES WITH THE LEGACY CODE** — `TabItem`, and with it `TabItem::suffix(&'static str)`, is deleted: §18.2 replaces the owned item list with `Tabs<'a, T, K, R>` over borrowed `&'a [T]` and a caller-supplied `RowFn` (`Tabs::row`, `crates/tui/src/components/tabs.rs:363`). A per-tab state glyph is now painted by the caller's `RowUi`, so the assertion belongs to jackin's Capsule tab strip and lands in Slice 7, not to the library |

### 2.23 `widgets::tree` (1)

| module path + test name | what it asserts | disposition |
|---|---|---|
| `widgets::tree::scroll_tests::wheel_moves_the_viewport_and_keeps_the_cursor` | a wheel moves the tree viewport without moving the cursor; the next render preserves the offset; a Down key pulls the cursor into the visible range | **MIGRATES IN SLICE 4** — `crates/tui/src/components/tree.rs` (WP 4C). The scroll half is already covered by `scroll::tests::*` and `conformance::<case>::wheel_at_boundary_is_consumed_without_repaint`; only the `Tree`-specific wiring needs the new home |

### 2.24 `widgets::viewport` (3) — the whole family is Slice 4E

| module path + test name | what it asserts | disposition |
|---|---|---|
| `widgets::viewport::tests::follows_tail_and_wheel_leaves_it` | a fresh viewport renders at the tail; a wheel up leaves follow mode and records the scrollback depth; `End` re-enters follow | **MIGRATES IN SLICE 4** — `crates/tui/src/components/viewport.rs` (WP 4E) |
| `widgets::viewport::tests::drag_selects_and_copies_text` | click-then-drag selects across lines with correct text; `y` emits `ViewportEvent::Copy`; `select_word_at` selects one word | **MIGRATES IN SLICE 4** — `crates/tui/src/components/viewport.rs` (WP 4E). The drag half will additionally be covered by `conformance::<case>::pointer_capture_delivers_drag_and_release` once the case registers |
| `widgets::viewport::tests::wraps_long_lines_and_bounds_retention` | `max_lines(3)` retains only the last three pushed lines; with wrap on, each 30-column line occupies two visual rows in a 20-column area | **MIGRATES IN SLICE 4** — `crates/tui/src/components/viewport.rs` (WP 4E); §20.9-7 rewrites the storage to `(range, width)` with windowed layout, so this test also becomes the functional guard for that rewrite |

### 2.25 `--lib` balance

| disposition | count |
|---|---|
| ALREADY DUPLICATED | 36 |
| MIGRATES IN SLICE 4 / 6 / 7 | 32 |
| DIES WITH THE LEGACY CODE | 8 |
| **total** | **76** |

Of the 36 duplicated, **6** are partial and their unasserted clauses are GAP-3, GAP-4,
GAP-9 and the GAP-12 caveat. Of the 8 dying, **7** die taking a property with them that nothing re-asserts (GAP-1,
GAP-2, GAP-5, GAP-6, GAP-8, GAP-10, GAP-11). Only one — `suffix_state_glyph_renders_after_the_label`
— dies for a reason §18.2 records, its feature having been deliberately replaced by a
caller-supplied row renderer. Separately, the `thousands` half of
`middle_truncation_and_thousands` (counted as duplicated) is a recorded deliberate change.

---

## 3. `--test perf` — the 30 root benchmark tests

Measured: 18 of the 30 names appear verbatim in `crates/tui/tests/perf.rs` today
(`comm -12` of the two `--list` outputs). **Name equality is what was measured; whether
each successor benchmarks the same workload was not re-derived per test.** Nine of the
new perf tests have no legacy counterpart and are §16.6 additions, not replacements.

### 3.1 Already present in `crates/tui/tests/perf.rs` (18)

`event_dispatch_is_not_o_n` · `fit_10k_grapheme_line_to_80` · `focus_tab_traversal_ring_200` ·
`frame_testbackend_empty_120x40` · `list_100k_rows_render` · `list_1k_rows_render` ·
`mouse_move_over_1000_regions` · `render_twice_allocates_the_same` ·
`style_backdrop_full_screen_120x40` · `style_downgrade_theme_all_levels` ·
`style_resolve_10k_parts` · `style_resolve_10k_parts_with_two_overlays` ·
`textbuffer_offset_at_10k_line` · `textbuffer_pos_of_10k_line` ·
`truncate_10k_grapheme_line_to_80` · `truncate_middle_10k_to_40` ·
`width_10k_grapheme_line` · `wrap_10k_graphemes_to_80`

Each asserts the allocation/byte counts §16.6's threshold table gives for that name, and
reports ns. **ALREADY DUPLICATED** — the same-named test in `crates/tui/tests/perf.rs`,
with the "after" threshold from §16.6.

### 3.2 Migrating with their component (10)

| module path + test name | what it asserts | disposition |
|---|---|---|
| `grid_500x12_render` | allocations per frame for a 500-row × 12-column grid | **MIGRATES IN SLICE 4** — `crates/tui/tests/perf.rs` (WP 4I); §16.6 "after: **< 100 allocs/frame**" |
| `grid_500x12_set_rows` | allocations to install a 500×12 result set | **MIGRATES IN SLICE 4** — `crates/tui/tests/perf.rs` (WP 4I). **Inferred**: §16.6 names `grid_500x12_load` (the TablePro-side one) but not `set_rows`; the library `Grid`'s row-installation cost is the natural home |
| `grid_100k_local_sort` | allocations per local sort of 100 000 rows | **MIGRATES IN SLICE 4** — `crates/tui/tests/perf.rs` (WP 4I); §16.6 "after: report; documents why `local_sort` stays opt-in" |
| `tree_100k_nodes_flatten` | allocations per expand/collapse toggle | **MIGRATES IN SLICE 4** — `crates/tui/tests/perf.rs` (WP 4C); §16.6 "after: allocs **< 10 × viewport**"; §20.9-8 |
| `tree_100k_nodes_render` | allocations per frame for a 100 000-node tree | **MIGRATES IN SLICE 4** — `crates/tui/tests/perf.rs` (WP 4C); §16.6 "after: allocs/frame independent of node count" |
| `key_tree_toggle_10k` | allocations per toggle key on a 10 000-node tree | **MIGRATES IN SLICE 4** — `crates/tui/tests/perf.rs` (WP 4C); §16.6 and §20.9-8 |
| `viewport_100k_lines_push` | allocations per pushed line into a 100 000-line viewport | **MIGRATES IN SLICE 4** — `crates/tui/tests/perf.rs` (WP 4E); §16.6 "after: allocs **independent of `lines.len()`**"; §20.9-7 |
| `viewport_100k_lines_render` | allocations per frame for a 100 000-line viewport | **MIGRATES IN SLICE 4** — `crates/tui/tests/perf.rs` (WP 4E); §16.6 calls it "**the binding acceptance for §20.9-7**" |
| `viewport_layout_10k_grapheme_line` | allocations to lay out one 10 000-grapheme line | **MIGRATES IN SLICE 4** — `crates/tui/tests/perf.rs` (WP 4E); §16.6 "after: **0** allocations" |

`no_full_collection_clone_per_frame` is also in this group:

| module path + test name | what it asserts | disposition |
|---|---|---|
| `no_full_collection_clone_per_frame` | bytes copied per frame stay under 64 KiB for a 100 000-row list frame **and** a 100 000-line viewport frame | **MIGRATES IN SLICE 4** — `crates/tui/tests/perf.rs`; the list half needs no new component, the viewport half needs WP 4E. §16.6 keeps the name and the 64 KiB threshold. **See GAP-13** — both assertions are behind `if env_flag("PERF_TARGET")`, which nothing sets, so this test has never asserted anything |

### 3.3 Dying (2)

| module path + test name | what it asserts | disposition |
|---|---|---|
| `capsule_pane_clone_4x2000` | the cost of cloning four 2 000-line panes per frame | **DIES WITH THE LEGACY CODE** — the per-frame `pane.term.clone()` it measures is deleted by §20.9-10 ("has no replacement because it has no reason to exist"); §16.6's row reads "**the test is deleted**", and the Slice 7 gate repeats it. Its absence is itself asserted, by line-absence in the frozen root `tests/perf_baseline.txt` (§21 item 28, corrected by §37) |
| `list_100k_rows_construct` | the cost of constructing 100 000 owned `ListItem`s | **DIES WITH THE LEGACY CODE** — the type it measures is deleted: §18.2's `list` row replaces the owned `ListItem` with a borrowed `&'a [T]`, so the new `List` has no construction step to benchmark. §16.6's row is "before: report / after: report" — it was never an assertion |

### 3.4 `--test perf` balance

| disposition | count |
|---|---|
| ALREADY DUPLICATED | 18 |
| MIGRATES IN SLICE 4 | 10 |
| DIES WITH THE LEGACY CODE | 2 |
| **total** | **30** |

---

## 4. `--bin showcase` (33) — every row migrates in Slice 5

Destination for the 26 `app_tests`: `apps/showcase/tests/app_tests.rs`, via the §16.4
`Harness`. Destination for the 7 `perf_tests`: `apps/showcase/tests/perf.rs`. All seven
perf names appear in the §16.6 threshold table.

**Enumerated fully by name; the "what it asserts" clauses in this section are INFERRED
from the test name and its module, not from reading each body.** That is acceptable here
and not for §2, because §16.4 already retains all 26 by name with a named destination and
a `Harness` operation table written to make each expressible; the library tests had no
such record, which is why §2 was read line by line.

| test name (`app_tests::`) | asserts (inferred) | disposition |
|---|---|---|
| `launches_and_renders_shell` | the app starts and paints its shell chrome | **MIGRATES IN SLICE 5** — `apps/showcase/tests/app_tests.rs` |
| `quit_keys` | the declared quit chords exit | **MIGRATES IN SLICE 5** — same |
| `keyboard_navigation_between_pages` | page navigation by keyboard reaches each page | **MIGRATES IN SLICE 5** — same |
| `tab_traversal_is_deterministic_and_wraps` | the focus ring order is stable and wraps | **MIGRATES IN SLICE 5** — same; the library half is `focus::tests::tab_cycles_forward_and_backward` |
| `hover_and_focus_render_differently` | hover and focus produce different frames | **MIGRATES IN SLICE 5** — same; **relevant to GAP-3**, this app test is currently the only place the hover/focus distinction is exercised end to end |
| `mouse_click_activates_and_keyboard_enter_activates` | both input paths activate a control | **MIGRATES IN SLICE 5** — same; library half is `conformance::*::keyboard_and_mouse_activation_are_equivalent` |
| `disabled_buttons_are_skipped_and_cannot_activate` | disabled controls are not focus stops and do not activate | **MIGRATES IN SLICE 5** — same; library half is `conformance::*::disabled_cannot_activate` |
| `hit_testing_prefers_rows_over_their_container` | the innermost registered region wins | **MIGRATES IN SLICE 5** — same |
| `list_scrolling_and_selection` | list wheel/keys scroll and select | **MIGRATES IN SLICE 5** — same |
| `tree_expand_collapse_and_focus_bar_column_is_stable` | expand/collapse keeps the focus gutter column fixed | **MIGRATES IN SLICE 5** — same; the theme coupling becomes `h.resolved(id, Part::GUTTER)` per §16.4 |
| `table_sorts_both_directions_and_clears` | header sort cycles both directions and clears | **MIGRATES IN SLICE 5** — same; depends on WP 4I `Grid` |
| `header_click_sorts` | clicking a header sorts | **MIGRATES IN SLICE 5** — same |
| `editable_table_commit_cancel_and_validation` | cell edit commit/cancel and validation | **MIGRATES IN SLICE 5** — same |
| `input_editing_commit_and_revert` | text input commit and revert | **MIGRATES IN SLICE 5** — same; library half is `components::input::tests::{commit_writes_the_controlled_value, cancel_restores_the_snapshot}` |
| `textarea_scrolls_with_wheel_and_keys` | textarea wheel and key scrolling | **MIGRATES IN SLICE 5** — same |
| `form_validation_blocks_submit_and_focuses_first_error` | a failing form blocks submit and focuses the first invalid field | **MIGRATES IN SLICE 5** — same; depends on WP 4F `Form` |
| `modal_traps_focus_and_restores_it` | a modal traps focus and restores it on close | **MIGRATES IN SLICE 5** — same; library half is `layer::runtime_tests::nested_layers_each_trap` |
| `prompt_dialog_validates_and_returns_value` | a prompt dialog validates and yields its value | **MIGRATES IN SLICE 5** — same |
| `settings_screen_remove_member_flow` | the settings remove-member journey | **MIGRATES IN SLICE 5** — same |
| `task_runner_animates_and_can_be_cancelled` | tick-driven animation and cancellation | **MIGRATES IN SLICE 5** — same; needs `Harness::ticks(n)` |
| `scrollbar_click_and_drag_move_the_view` | scrollbar track click and thumb drag | **MIGRATES IN SLICE 5** — same; library half is `conformance::scroll_region::pointer_capture_delivers_drag_and_release` |
| `below_minimum_size_shows_reduced_state` | the too-small screen appears below the minimum size | **MIGRATES IN SLICE 5** — same; §16.4 fact 7 keeps the copy strings verbatim |
| `resize_recovers_from_too_small` | growing back restores the normal layout | **MIGRATES IN SLICE 5** — same |
| `every_page_renders_at_representative_sizes_without_panic` | every page renders at the representative size matrix without panicking | **MIGRATES IN SLICE 5** — same |
| `color_downgrade_still_renders` | every colour level renders | **MIGRATES IN SLICE 5** — same; library half is `theme::downgrade::tests::downgrade_maps_every_token_exhaustively` |
| `showcase_visual_baseline` | the showcase digest matches its baseline | **MIGRATES IN SLICE 5** — `apps/showcase/tests/visual.rs` + `apps/showcase/tests/baselines/showcase.txt` |
| `perf_tests::frame_showcase_lists_120x40` | allocs/hits/ring for the list frame at 120×40 | **MIGRATES IN SLICE 5** — `apps/showcase/tests/perf.rs`; §16.6 "after: **< 20 allocs/frame**" |
| `perf_tests::frame_showcase_lists_80x24` | the same frame at 80×24 | **MIGRATES IN SLICE 5** — same; §16.6 "≤ the 120×40 case" |
| `perf_tests::frame_showcase_dialog_open` | hit-registry size with a modal open | **MIGRATES IN SLICE 5** — same; §16.6 "hits **< 25 %**" (`inert_below`, §20.9-16) |
| `perf_tests::key_showcase_down_lists` | allocations per Down key | **MIGRATES IN SLICE 5** — same; §16.6 "**0 allocs/event**" |
| `perf_tests::mouse_move_showcase_frame` | allocations per pointer move | **MIGRATES IN SLICE 5** — same; §16.6 "**0 allocs**" |
| `perf_tests::wheel_showcase_lists` | allocations per wheel event | **MIGRATES IN SLICE 5** — same; §16.6 "**0 allocs**" |
| `perf_tests::render_twice_allocates_the_same` | two identical frames allocate identically | **MIGRATES IN SLICE 5** — same; §16.6 "equal counts". Note the root `--test perf` has a same-named test; they are **two distinct tests** on different workloads and both survive |

---

## 5. `--bin tablepro` (41) — every row migrates in Slice 6

Destinations: the 23 `app_tests` → `apps/tablepro/tests/app_tests.rs` (§16.4, and the
Slice 6 gate's "all 23 existing tests green"); `model::tests` and `sql::tests` stay
in-module and move with the tree; `perf_tests` → `apps/tablepro/tests/perf.rs`;
`visual_tests` → `apps/tablepro/tests/visual.rs` + `tests/baselines/tablepro.txt`.

`app_tests` clauses are **INFERRED** from names; the 10 `model`/`sql` clauses were read.

### 5.1 `app_tests` (23)

| test name | asserts (inferred) | disposition |
|---|---|---|
| `connections_screen_lists_and_connects_with_keyboard` | keyboard journey through the connection list to a connection | **MIGRATES IN SLICE 6** — `apps/tablepro/tests/app_tests.rs` |
| `failed_connection_shows_error_and_retry` | a failed connection surfaces an error and a retry affordance | **MIGRATES IN SLICE 6** — same |
| `read_only_connection_refuses_writes` | a read-only connection refuses write statements | **MIGRATES IN SLICE 6** — same; §16.4 adds `tablepro::view_grid_is_read_only_with_a_reason` |
| `explorer_opens_table_and_grid_navigates` | opening a table from the explorer and navigating the grid | **MIGRATES IN SLICE 6** — same |
| `mouse_opens_table_and_switches_tabs` | the same journey by mouse | **MIGRATES IN SLICE 6** — same |
| `narrow_terminals_turn_the_explorer_into_a_drawer` | the explorer becomes a drawer below a width threshold | **MIGRATES IN SLICE 6** — same |
| `quick_switcher_opens_table` | the quick switcher opens a table | **MIGRATES IN SLICE 6** — same; depends on WP 4F `Picker` |
| `tab_strip_overflow_and_tab_list` | tab-strip overflow and the overflow list | **MIGRATES IN SLICE 6** — same; library half is `components::tabs::tests::close_targets_the_logical_tab_after_a_reorder` |
| `sort_and_filter_on_table_tab` | sorting and filtering a table tab | **MIGRATES IN SLICE 6** — same |
| `structure_view_toggle` | toggling the structure view | **MIGRATES IN SLICE 6** — same; §18.2 turns the Structure tab into six `GridModel`s |
| `editor_completion_and_execution` | completion in the editor and statement execution | **MIGRATES IN SLICE 6** — same; depends on WP 4F `Completion` |
| `execution_error_marks_editor_and_result` | an execution error marks both editor and result panes | **MIGRATES IN SLICE 6** — same |
| `cancel_running_query` | cancelling a running query | **MIGRATES IN SLICE 6** — same |
| `explain_opens_plan_tree` | EXPLAIN opens the plan tree | **MIGRATES IN SLICE 6** — same; depends on WP 4C `Tree` |
| `history_tab_reopens_query` | the history tab reopens a past query | **MIGRATES IN SLICE 6** — same |
| `pending_edits_preview_and_save` | pending edits preview as SQL and save | **MIGRATES IN SLICE 6** — same; the model half is `model::tests::preview_sql_orders_updates_inserts_deletes` |
| `safe_mode_picker_changes_level_and_strip` | the safe-mode picker changes level and updates the identity strip | **MIGRATES IN SLICE 6** — same |
| `safety_gate_intercepts_dangerous_statement_on_production` | a dangerous statement on production is intercepted | **MIGRATES IN SLICE 6** — same; the model half is `sql::tests::classifies_like_tablepro` |
| `safety_gate_typed_token_executes` | typing the confirmation token executes the gated statement | **MIGRATES IN SLICE 6** — same |
| `silent_level_runs_scoped_writes_but_confirms_destructive` | the silent level runs scoped writes but still confirms destructive ones | **MIGRATES IN SLICE 6** — same |
| `every_screen_renders_at_representative_sizes` | every screen renders across the size matrix | **MIGRATES IN SLICE 6** — same |
| `acceptance_flow_keyboard_only` | the full keyboard acceptance journey | **MIGRATES IN SLICE 6** — same; §16.4 renames it `tablepro::keyboard_flow_full_journey` |
| `acceptance_flow_mouse` | the full mouse acceptance journey | **MIGRATES IN SLICE 6** — same; §16.4 renames it `tablepro::mouse_flow_full_journey` |

### 5.2 `model::tests` (4) — read, not inferred

| test name | what it asserts | disposition |
|---|---|---|
| `model::tests::preview_sql_orders_updates_inserts_deletes` | the preview emits UPDATE before INSERT before `DELETE FROM public.orders WHERE id = '…'`, and collapses a no-op edit to one statement | **MIGRATES IN SLICE 6** — stays in-module at `apps/tablepro/src/model.rs`; **not named by §16.4 (see §0.1)** |
| `model::tests::history_search_is_multi_term_and` | history search ANDs its terms and can filter to failed entries only | **MIGRATES IN SLICE 6** — same |
| `model::tests::completion_is_context_aware` | completion offers tables after `FROM`, columns after a qualified prefix, reports the replace length and matched grapheme indices, and `auto_trigger` fires after `FROM ` but not mid-identifier | **MIGRATES IN SLICE 6** — same; the fuzzy-match half now has a library counterpart in `text::fuzzy::tests::fuzzy_returns_grapheme_indices_into_the_original_label` |
| `model::tests::switcher_ranks_tables_first_and_prefix_first` | the switcher ranks tables above recent queries and prefix matches above others, and groups results | **MIGRATES IN SLICE 6** — same; library half is `text::fuzzy::tests::fuzzy_ranks_prefix_before_boundary_before_substring_before_subsequence` |

### 5.3 `sql::tests` (6) — read, not inferred

| test name | what it asserts | disposition |
|---|---|---|
| `sql::tests::splits_and_finds_statement_at_cursor` | a script splits into three statements and the cursor resolves to the containing one | **MIGRATES IN SLICE 6** — `apps/tablepro/src/sql.rs`; **not named by §16.4** |
| `sql::tests::parses_select_with_predicates_order_limit` | a SELECT parses into schema, table, two predicates, an ORDER BY direction and a LIMIT | **MIGRATES IN SLICE 6** — same |
| `sql::tests::classifies_like_tablepro` | statement tiers (`Safe`/`Write`/`Destructive`), `is_dangerous` (an unqualified DELETE is, a `WHERE`-qualified one is not, `EXPLAIN ANALYZE DELETE` is), and the `gate(level, stmt) → Decision` matrix across four safety levels | **MIGRATES IN SLICE 6** — same. This is the single densest domain test in the tree and the only coverage of the safety gate's decision matrix |
| `sql::tests::runs_filtered_sorted_select` | a filtered, sorted SELECT returns 20 rows all matching the predicate, sorted, and marks the result editable | **MIGRATES IN SLICE 6** — same |
| `sql::tests::errors_are_specific` | an unknown column reports `column "nope"` and a syntax error reports position 0 | **MIGRATES IN SLICE 6** — same |
| `sql::tests::explain_builds_tree` | EXPLAIN builds a `Limit → Sort → …` plan tree that renders with the root first | **MIGRATES IN SLICE 6** — same |

### 5.4 `perf_tests` (7) and `visual_tests` (1)

| test name | asserts | disposition |
|---|---|---|
| `perf_tests::frame_tablepro_grid_500x12_120x40` | allocs and hits for the 500×12 grid frame | **MIGRATES IN SLICE 6** — `apps/tablepro/tests/perf.rs`; §16.6 "**< 100 allocs/frame**, hits ≤ 320"; named by the Slice 6 gate |
| `perf_tests::grid_500x12_load` | allocations to load a 500×12 result set | **MIGRATES IN SLICE 6** — same; §16.6 "**< 8 000 allocs**"; named by the Slice 6 gate; §20.9-11 |
| `perf_tests::key_tablepro_grid_cursor` | allocations per cursor key | **MIGRATES IN SLICE 6** — same; §16.6 "**0 allocs/event**" |
| `perf_tests::key_tablepro_grid_sort_local` | allocations per comparison during a local sort | **MIGRATES IN SLICE 6** — same; §16.6 "**≤ 1 alloc/comparison**" |
| `perf_tests::mouse_click_grid_cell` | allocations and time per cell click | **MIGRATES IN SLICE 6** — same; §16.6 "**0 allocs**, ns **< 0.2×**" |
| `perf_tests::wheel_tablepro_grid` | allocations per wheel event | **MIGRATES IN SLICE 6** — same; §16.6 "**0 allocs**" |
| `perf_tests::debug_and_release_alloc_counts_match` | the debug and release profiles record the same allocation count | **MIGRATES IN SLICE 6** — same; §16.6 "equal **±1 allocation**"; §20.9-5, P-B |
| `visual_tests::tablepro_visual_baseline` | the TablePro digest matches `tests/baselines/tablepro.txt` | **MIGRATES IN SLICE 6** — `apps/tablepro/tests/visual.rs`; the Slice 6 gate requires the baseline regenerated with every difference classified against §20.10 |

---

## 6. `--bin jackin-preview` (67) — every row migrates in Slice 7

Destinations: 22 `app_tests` + 6 `app_tests_chrome` → `apps/jackin-preview/tests/app_tests.rs`
(§16.4's "28 jackin (22 + 6 chrome)"); `arbiter`/`clock`/`rain`/`scenario` (10) are the
in-module unit tests §16.4 and the Slice 7 gate name explicitly; `domain::*`, `sim::*` and
`screens::inspect` (25) stay in-module and move with the tree but are **not named anywhere**
(§0.1); `perf_tests` → `apps/jackin-preview/tests/perf.rs`; `visual_tests` →
`apps/jackin-preview/tests/visual.rs`.

`app_tests` clauses are **INFERRED** from names; the `arbiter`/`clock`/`rain`/`scenario`/
`domain`/`sim` clauses were read.

### 6.1 `app_tests` (22) and `app_tests_chrome` (6) — inferred

| test name | asserts (inferred) | disposition |
|---|---|---|
| `app_tests::first_use_plays_intro_then_manager_and_no_replay_when_returning` | the intro plays once on first use and not on return | **MIGRATES IN SLICE 7** — `apps/jackin-preview/tests/app_tests.rs` |
| `app_tests::manager_navigation_expand_and_detail_focus` | manager list navigation, expansion and detail-pane focus | **MIGRATES IN SLICE 7** — same |
| `app_tests::manager_launch_picker_hides_agents_without_an_account` | the launch picker omits agents with no configured account | **MIGRATES IN SLICE 7** — same; domain half is `domain::fixtures::tests::offered_agents_skip_the_unconfigured_and_block_the_unusable` |
| `app_tests::prelude_creates_a_pending_workspace_and_opens_the_editor` | the prelude creates a pending workspace and opens the editor | **MIGRATES IN SLICE 7** — same |
| `app_tests::prelude_refuses_a_duplicate_name_and_cancels_cleanly` | duplicate workspace names are refused; cancel leaves no residue | **MIGRATES IN SLICE 7** — same |
| `app_tests::editor_edits_count_once_preview_then_saves_and_returns` | edits count once, preview then save, return to the caller | **MIGRATES IN SLICE 7** — same; domain half is `domain::workspace::tests::change_count_tracks_fields_and_rows` |
| `app_tests::editor_env_plain_value_stays_masked` | a plain env value stays masked; the `m` command does not reveal it | **MIGRATES IN SLICE 7** — same; §16.4 adds `jackin::form_dialog_secret_never_reaches_the_screen_as_a_string` |
| `app_tests::editor_accounts_tab_switches_inherited_defaults_off_and_extra_accounts_on` | inherited defaults can be switched off and extra accounts on | **MIGRATES IN SLICE 7** — same |
| `app_tests::accounts_register_with_a_1password_reference_and_never_render_the_secret` | a 1Password-referenced account never renders its secret | **MIGRATES IN SLICE 7** — same; domain half is `sim::onepassword::tests::resolves_only_inside_the_closure` |
| `app_tests::accounts_plain_key_is_masked_everywhere_and_remove_asks_first` | a plain key is masked in every surface; removal confirms first | **MIGRATES IN SLICE 7** — same; domain half is `domain::account::tests::masking_helpers` |
| `app_tests::usage_overlay_is_read_only_and_hands_off_to_accounts` | the usage overlay is read-only and hands off to accounts | **MIGRATES IN SLICE 7** — same |
| `app_tests::cockpit_resolves_every_effective_account_for_the_container` | the cockpit resolves the effective account set | **MIGRATES IN SLICE 7** — same; domain half is `domain::fixtures::tests::workspace_policy_builds_a_deterministic_effective_set` |
| `app_tests::launch_runs_all_stages_and_hands_off_to_the_capsule` | the launch sequence runs to completion and hands off | **MIGRATES IN SLICE 7** — same; domain half is `sim::launch::tests::clean_plan_walks_all_eleven_stages_in_order` |
| `app_tests::launch_failure_returns_to_the_construct_when_another_instance_runs` | a launch failure with another instance running returns to the construct | **MIGRATES IN SLICE 7** — same |
| `app_tests::detach_reconnect_and_final_exit_plays_one_outro` | detach, reconnect, and exactly one outro on final exit | **MIGRATES IN SLICE 7** — same; domain half is `arbiter::tests::exit_token_has_one_consumer_and_fails_closed` |
| `app_tests::still_inside_feedback_when_other_instances_remain` | the "still inside" feedback appears while instances remain | **MIGRATES IN SLICE 7** — same |
| `app_tests::settings_trust_toggle_and_failed_save_keep_edits` | a failed save preserves the pending edits | **MIGRATES IN SLICE 7** — same |
| `app_tests::environments_stay_readable_with_a_hundred_roles` | the environments surface stays readable at 100 roles | **MIGRATES IN SLICE 7** — same |
| `app_tests::hard_cases_refresh_keeps_last_good_and_help_opens_everywhere` | a failed refresh keeps the last good data; help opens from every screen | **MIGRATES IN SLICE 7** — same; depends on WP 4F `HelpOverlay` |
| `app_tests::reduced_motion_and_paused_frames_are_deterministic` | reduced motion and paused frames are byte-deterministic | **MIGRATES IN SLICE 7** — same; the Slice 7 gate keeps the two-run determinism assertion |
| `app_tests::too_small_state_and_resize_recover` | too-small state and recovery on resize | **MIGRATES IN SLICE 7** — same; §16.4 fact 7 keeps the copy strings |
| `app_tests::complete_jackin_flow_keyboard_first` | the full keyboard-first journey | **MIGRATES IN SLICE 7** — same; §16.4 retains it as `jackin::complete_flow_keyboard_first` |
| `app_tests_chrome::capsule_has_a_menu_bar_and_a_status_bar_instead_of_the_identity_line` | the Capsule uses `MenuBar` + `StatusBar` chrome | **MIGRATES IN SLICE 7** — same; depends on WP 4F `MenuBar` |
| `app_tests_chrome::menu_bar_opens_switches_and_runs_an_action` | menu-bar open, switch, and action dispatch | **MIGRATES IN SLICE 7** — same; library half migrates as `widgets::menu::tests::menubar_opens_switches_and_chooses` (§2.16) |
| `app_tests_chrome::tab_context_menu_renames_and_closes_by_mouse_and_keyboard` | the tab context menu renames and closes via both input paths | **MIGRATES IN SLICE 7** — same |
| `app_tests_chrome::command_palette_scrolls_with_the_wheel_and_keeps_the_selection` | the palette wheel-scrolls without moving the selection | **MIGRATES IN SLICE 7** — same; library half migrates as `widgets::picker::tests::wheel_scrolls_the_rows_and_survives_the_next_render` (§2.17) |
| `app_tests_chrome::hint_bar_stays_on_the_last_row_across_layers` | the hint bar stays on the last row as layers change | **MIGRATES IN SLICE 7** — same; library half is `components::hintbar::tests::the_topmost_layer_wins_and_the_fallback_is_none` |
| `app_tests_chrome::inspect_changes_opens_from_the_view_menu_in_both_modes` | Inspect Changes opens from the View menu in compact and advanced modes | **MIGRATES IN SLICE 7** — same |

### 6.2 The 10 in-module unit tests §16.4 and the Slice 7 gate name — read

| test name | what it asserts | disposition |
|---|---|---|
| `arbiter::tests::empty_construct_plays_once_and_join_skips` | the first entry plays the intro, the pending flag clears after one consumption, and a later entry joins the active construct | **MIGRATES IN SLICE 7** — in-module, `apps/jackin-preview/src/arbiter.rs` |
| `arbiter::tests::foreign_claim_suppresses_duplicate_intro` | a foreign claim yields `Duplicate` and does not arm the intro | **MIGRATES IN SLICE 7** — same |
| `arbiter::tests::exit_token_has_one_consumer_and_fails_closed` | exactly one consumer wins the exit token; a second request reports `AlreadyEnded`; a non-consumer sees `exit_consumed() == false` | **MIGRATES IN SLICE 7** — same |
| `arbiter::tests::missing_entry_time_omits_elapsed` | with no entry time the elapsed clause is omitted rather than defaulted | **MIGRATES IN SLICE 7** — same |
| `clock::tests::clock_is_pure_over_ticks` | the clock advances only by ticks, never by wall time; `stamp`/`weekday`/`ago`/`reset_label` are pure functions of the tick count | **MIGRATES IN SLICE 7** — `apps/jackin-preview/src/clock.rs`. **Highest-risk row in this section**: `COORDINATION.md` records that the virtual clock advances by the *route's* `tick_ms`, and re-basing it breaks ~40 tick counts and every fixture timestamp at once |
| `clock::tests::durations_use_two_units` | `format_duration` renders at most two units (`"7 min 30 s"`, `"2 h 14 min"`, `"1 d 3 h"`, and drops the second when zero) | **MIGRATES IN SLICE 7** — same |
| `rain::tests::intro_timeline_follows_the_original_pacing` | `P1_LEN == 64`, phrase indices per tick, the knock gap, phase transitions and `WARP_START` | **MIGRATES IN SLICE 7** — `apps/jackin-preview/src/rain.rs`, rewritten onto `Role` + `Ui::dim_layer` |
| `rain::tests::outro_skips_and_captions_like_the_original` | outro phase order, skip behaviour, and `format_universe_duration`'s long-form units | **MIGRATES IN SLICE 7** — same |
| `rain::tests::starfield_is_deterministic_and_restrained` | the starfield is a pure function of its seed and tick | **MIGRATES IN SLICE 7** — same |
| `scenario::tests::names_round_trip` | every `Scenario` round-trips through its name; `Motion::resolve` honours the reduced-motion flag and an explicit override | **MIGRATES IN SLICE 7** — `apps/jackin-preview/src/scenario.rs` |

### 6.3 The 25 in-module tests named nowhere — read (see §0.1)

| test name | what it asserts | disposition |
|---|---|---|
| `domain::account::tests::masking_helpers` | `masked` keeps only the last four characters; `fingerprint` is 8 characters; `tail_of` extracts the tail | **MIGRATES IN SLICE 7** — `apps/jackin-preview/src/domain/account.rs`; **not named by §16.4 or the Slice 7 gate** |
| `domain::account::tests::one_default_per_provider` | setting a default clears the previous one; name collisions are detected per provider; `endpoint` is Grok-only; setting a default on an ineligible account errors | **MIGRATES IN SLICE 7** — same; **not named** |
| `domain::agent::tests::axes_are_linked_but_distinct` | agent → provider → usage surface are three linked but distinct axes; `registerable`, `auth_modes` counts, `supports_endpoint` | **MIGRATES IN SLICE 7** — `domain/agent.rs`; **not named** |
| `domain::fixtures::tests::precedence_order_and_why` | the six-level account precedence ladder resolves in order and each result carries a human `why` string | **MIGRATES IN SLICE 7** — `domain/fixtures.rs`; **not named** |
| `domain::fixtures::tests::workspace_policy_builds_a_deterministic_effective_set` | the effective account set is deterministic, ordered, tagged with origin and usability, and stable across recomputation | **MIGRATES IN SLICE 7** — same; **not named** |
| `domain::fixtures::tests::offered_agents_skip_the_unconfigured_and_block_the_unusable` | offered agents exclude unconfigured ones and mark unusable ones blocked | **MIGRATES IN SLICE 7** — same; **not named** |
| `domain::fixtures::tests::every_scenario_builds` | every declared scenario constructs a valid world | **MIGRATES IN SLICE 7** — same; **not named**. This is the cheapest whole-fixture smoke test in the tree |
| `domain::instance::tests::hidden_statuses_and_actions` | which instance statuses are hidden, stoppable and reconnectable, and the agent-state rank order | **MIGRATES IN SLICE 7** — `domain/instance.rs`; **not named** |
| `domain::usage::tests::quota_status_thresholds` | `QuotaStatus::from_pct` thresholds and the `"1,240 / 5,000 credits"` value label | **MIGRATES IN SLICE 7** — `domain/usage.rs`; **not named**. The grouped-integer label is app-side, consistent with §2.10 |
| `domain::usage::tests::empty_registry_is_empty_health` | an empty registry reports `HealthWord::Empty` and `"0 accounts · 0 enabled · 0 providers"` | **MIGRATES IN SLICE 7** — same; **not named** |
| `domain::workspace::tests::masking_never_reveals_the_value` | `mask` never leaks a prefix, masks short values wholly, renders `"(empty)"`, and `env_key_error` rejects reserved and malformed keys | **MIGRATES IN SLICE 7** — `domain/workspace.rs`; **not named**. Library half is `secret::tests::debug_and_display_redact` |
| `domain::workspace::tests::change_count_tracks_fields_and_rows` | the change counter counts field and row edits and is zero for an identical pair; `Isolation::next` cycles | **MIGRATES IN SLICE 7** — same; **not named** |
| `sim::changes::tests::deterministic_and_realistic` | the generated change set is deterministic for a seed and has the declared shape (5 files, hunk counts, one of each status, unpushed count, summary prefix) | **MIGRATES IN SLICE 7** — `sim/changes.rs`; **not named** |
| `sim::changes::tests::fewer_uncommitted_than_touched_keeps_every_touched_file` | reducing the uncommitted count never drops a touched file; a zero-size request is empty | **MIGRATES IN SLICE 7** — same; **not named** |
| `sim::changes::tests::no_secret_shaped_content` | no generated diff line is secret-shaped | **MIGRATES IN SLICE 7** — same; **not named**. A security-relevant assertion with no successor named anywhere |
| `sim::launch::tests::clean_plan_walks_all_eleven_stages_in_order` | a clean plan runs all eleven stages in `Stage::ALL` order, skips `AgentBinaries`, counts (10, 1), emits every build line, and ends with `Ready` | **MIGRATES IN SLICE 7** — `sim/launch.rs`; **not named** |
| `sim::launch::tests::failure_and_blocked_plans_stop_the_frontier` | a failure records its stage, emits `Failed`, and leaves downstream stages `Queued`; a blocked plan records `blocked_at` | **MIGRATES IN SLICE 7** — same; **not named** |
| `sim::launch::tests::credential_error_holds_until_retry` | a credential error holds the run (no advance) until retried, then completes | **MIGRATES IN SLICE 7** — same; **not named** |
| `sim::onepassword::tests::resolves_only_inside_the_closure` | an `op://` reference canonicalises correctly, `describe` returns only the masked form, and the secret resolves **only** inside the resolution closure | **MIGRATES IN SLICE 7** — `sim/onepassword.rs`; **not named**. The strongest secret-containment assertion in the repository |
| `sim::pty::tests::split_close_and_nearest` | pane split produces the expected leaves and seams; directional `nearest` navigation; closing panes collapses tabs and updates the tab label | **MIGRATES IN SLICE 7** — `sim/pty.rs`; **not named**. Depends on WP 4E `SplitPane` |
| `sim::pty::tests::agent_process_emits_boots_and_replies` | the simulated agent transitions `Working → Done`, emits boot output and replies, and the shell keeps a caret | **MIGRATES IN SLICE 7** — same; **not named** |
| `sim::world::tests::masks_private_paths` | a home path renders `~/…` and a foreign path `…/tail` | **MIGRATES IN SLICE 7** — `sim/world.rs`; **not named** |
| `screens::inspect::tests::compact_opens_a_file_and_returns_to_the_list` | compact mode opens a file and returns to the list | **MIGRATES IN SLICE 7** — `screens/inspect.rs`; **not named**. §18.3 #23 keeps this screen domain, composing `Tree` + `DiffView` + `SplitPane` |
| `screens::inspect::tests::advanced_tree_drives_the_diff_and_modes_toggle` | selecting in the advanced tree drives the diff pane and the mode toggle works | **MIGRATES IN SLICE 7** — same; **not named**. Depends on WP 4H `DiffView` and WP 4C `Tree` |
| `screens::inspect::tests::narrow_terminal_stacks_the_advanced_layout` | the advanced layout stacks below a width threshold | **MIGRATES IN SLICE 7** — same; **not named** |

### 6.4 `perf_tests` (3) and `visual_tests` (1)

| test name | asserts | disposition |
|---|---|---|
| `perf_tests::frame_jackin_capsule_4panes_120x40` | allocations per Capsule frame with four panes | **MIGRATES IN SLICE 7** — `apps/jackin-preview/tests/perf.rs`; §16.6 "**< 200 allocs/frame**" from a measured 1 080 602; the Slice 7 gate names it |
| `perf_tests::frame_jackin_manager_100rows_120x40` | allocations per manager frame with 100 rows | **MIGRATES IN SLICE 7** — same; §16.6 "**< 60 allocs/frame**"; §20.9-15 |
| `perf_tests::key_jackin_manager_move` | allocations per manager navigation key | **MIGRATES IN SLICE 7** — same; §16.6 "**0 allocs/key**" from a measured 132 |
| `visual_tests::jackin_visual_baseline` | the jackin digest matches `tests/baselines/jackin.txt` | **MIGRATES IN SLICE 7** — `apps/jackin-preview/tests/visual.rs`; the Slice 7 gate requires the baseline regenerated with differences classified |

---

## 7. The balance

```
247 = 54 duplicated + 183 migrating + 10 dying
```

| target | duplicated | migrating | dying | total |
|---|---:|---:|---:|---:|
| `--lib` | 36 | 32 | 8 | 76 |
| `--test perf` | 18 | 10 | 2 | 30 |
| `--bin showcase` | 0 | 33 | 0 | 33 |
| `--bin tablepro` | 0 | 41 | 0 | 41 |
| `--bin jackin-preview` | 0 | 67 | 0 | 67 |
| **total** | **54** | **183** | **10** | **247** |

### 7.1 Coverage of this file

* `--lib` (76) — **enumerated fully, from reading every test body and every named successor.**
* `--test perf` (30) — **enumerated fully.** Successor identification for the 18 duplicated
  rows is by **name equality** between the two `--list` outputs; the workloads were not
  re-derived per test.
* `--bin showcase` (33), `--bin tablepro` (41), `--bin jackin-preview` (67) — **enumerated
  fully by name and disposition.** The "what it asserts" clauses are **read** for the 10
  tablepro `model`/`sql` tests and the 35 jackin in-module tests, and **inferred from the
  test name and module** for the 74 `app_tests`/`app_tests_chrome`/`perf_tests`/`visual_tests`
  rows. Each inferred row is marked "(inferred)" in its own column.

### 7.2 Nothing was left unclassified

Every one of the 247 has exactly one disposition. Four rows carry a caveat rather than an
open question, and each is stated in the row itself:

1. `widgets::grid::tests::position_label_variants` — Slice 6 destination is **inferred**;
   §18.2 does not name `RowTotal`/`position_label` explicitly.
2. `perf::grid_500x12_set_rows` — WP 4I destination is **inferred**; §16.6 names
   `grid_500x12_load` but not `set_rows`.
3. `widgets::brand::tests::lockup_is_padded_accent_and_bold` — the named successors
   (`render::components::brand::*`) are currently **red**, because
   `crates/tui/tests/baselines/components.txt` has no `brand` rows. Only six of the twenty registered
   `render_components` matrices are blessed (`button`, `dialog`, `field`, `list`, `tabs`,
   `text_input`); the other fourteen are unblessed in the same way; they panic on
   the missing baseline rather than passing vacuously, so the state is visible, but the
   disposition "ALREADY DUPLICATED" is contingent on blessing them.
4. `widgets::statusbar::tests::render_fills_the_row_and_registers_hover` — GAP-12; the
   successor exists, is better, is **failing**, and is deliberately uncommitted.

### 7.3 The four preconditions this file establishes for Slice 5

1. **Do not delete `src/` before WP 4C, 4D, 4E, 4F, 4H and 4I have landed.** 32 library
   tests and 10 perf tests name destinations in components that do not exist today
   (GAP-14). The slice plan already orders it this way; this is the enumerated cost of
   getting the order wrong.
2. **Bless the 14 unblessed `render::components::*` matrices before deleting the legacy
   tree**, or four "ALREADY DUPLICATED" dispositions above are claims against red tests.
3. **Resolve GAP-12** — land `crates/tui/tests/status_bar_hover.rs` green, or the
   `StatusBar` hover and registration properties go to zero coverage.
4. **Write the eleven missing assertions (GAP-1 through GAP-11) into `crates/tui` before
   the deletion, not after.** After the deletion there is nothing left to write them from.
