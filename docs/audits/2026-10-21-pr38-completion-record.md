# PR #38 completion record — 2026-10-21

This is the current running record for the Termrock completion work delivered
through PR #38 (`parity/termrock-visual-baseline`). It records fixed reference
identities, what was reviewed, what was changed, what was integrated or
excluded, the evidence, and what is still incomplete. Acceptance is not
claimed; the status table at the end is the authority.

## Fixed references

Recorded before any change and not modified during this work:

| Purpose | Revision |
| --- | --- |
| PR #26 product reference (`design/interaction-polish`) | `f4867e9b5f8a08a3bb61ce80d644198a29f51984` |
| Frozen baseline source | `4169783ae883788186d161af09ba9c481bd84e5c` |
| Frozen baseline tag object | `2718a6b8127a522a3d351d0b4bdb7542fc8da542` (peels to `4169783ae`) |
| Tuiscotti pin | `a1ea27f757d75d07b05499276cd932e9f8e067ad` (`Cargo.toml`, `termrock-e2e`, `visual-harness`, `showcase`) |

PR #38 tip recorded at the start of this session:
`7649ee7ed8e8645e49f859a171ea0193fccc1e82`. The branch advanced during the
work (`e9375eb4f`, `40d1d89b7`, …) because other workstreams are landing; the
review evidence below is pinned to `7649ee7ed`, and each landed change was
rebased and re-verified on the then-current tip before publication.

No reference revision was changed. No reference decision was required for the
work in this record.

## Root-cause fix: unstable `Theme::fingerprint`

`Theme::fingerprint` hashed the `Debug` representation. `Recipes` carries a
per-construction `revision` counter for cache invalidation, so two independent
`Theme::junie()` values printed differently and fingerprinted differently even
though `PartialEq` deliberately ignores `revision`.

Effect on the gate: every conformance check that compares a theme fingerprint
before and after a local style override failed order-dependently
(`local_override_does_not_mutate_the_theme` for every component that has one)
plus the runtime/overlay checks that compare against it. The `termrock-conformance`
package had 602 failing cases on the pinned tip.

Fix: `Recipes` has a manual `Debug` that omits `revision`; the revision stays
private bookkeeping and semantic equality is unchanged. Regression test
`fingerprint_ignores_per_construction_revision` asserts two `Theme::junie()`
values are equal and fingerprint equally.

Evidence: `cargo nextest run -p termrock-conformance` fell from 602 failures to
19 failures on the same source, with no expectation updated and no test
weakened. A later conformance-driver pass took the focused suite from 23 to 10
failing cases (see "Open verification"). The 225 `render_components` digest
mismatches are a separate, branch-wide red confirmed on the origin tip.

## Picker contract gaps closed

The branch audit found PR #38 documents contracts that the code did not
implement. Two were implemented additively; the frozen defaults of existing
consumers do not change except where the reference geometry required it.

- `FilterPolicy::Substring`: case-insensitive contiguous matching, alongside
  the existing `Label` subsequence and `Caller` policies. Test:
  `substring_policy_requires_a_contiguous_match`.
- `Picker::max_rows(rows)`: explicit row cap with the existing theme
  `popup_max_rows` as the default. Zero is treated as one. Test:
  `max_rows_caps_the_requested_layer_height`.

Picker geometry from `parity/picker-geometry-20261020` (`28cc24880`,
`6b72d77a6`) was evaluated: seven chrome rows for a searchable card, height
clamping, and `GeometricCenter`. Adopting it changes every picker frame
(TablePro `quick-switcher 120 40` `b206bc55…` → `b90250e6…`). The source
contract is plausible but the change has not been proven against the frozen
tuiscotti corpus, and the current application digests were re-approved from a
candidate. It was therefore **not** adopted in this increment; the pending
decision needs a paired reference capture. No digest was blessed.

Source: `crates/termrock-navigation/src/filter_list.rs`,
`crates/termrock-overlays/src/picker.rs`,
`crates/termrock-overlays/tests/picker_projection.rs`.

## Branch consolidation ledger

Full tables: [parity branches](evidence/2026-10-21-pr38-completion/parity-branches-review.md),
[WIP branches](evidence/2026-10-21-pr38-completion/wip-branches-review.md),
[other branches](evidence/2026-10-21-pr38-completion/other-branches-review.md).

Reviewed 51 `parity/*`, 54 `wip/*` and 18 other remote branches against the
pinned tip using patch-id equivalence plus source verification.

- Already integrated (rebased, reworked, or literal ancestors): 47 `parity/*`,
  21 ancestor + 4 patch-equivalent + 9 reworked `wip/*`, and 12 other branches.
- Integrated later / still queued: picker geometry and the picker contract
  hunks above; `edit-menu-overlay-20261020` (overlay adapter still absent) and
  `fix/picker-focus-and-rows-20261020` (only the missing hunks; the branch is
  far behind and merging it wholesale would revert current APIs).
- Needs independent review, not merged: `buttons-reference-decision` pins,
  `codex/termrock-q48-source-only`, `codex/termrock-local-proposals`,
  `owned-fresh-plan`, `menu-cursor`, `menu-underlay`, `jackin-live-capsule`,
  `jackin-rate-limit`, `public-consumer-contracts`.
- Excluded: `design/interaction-polish` (read-only product reference),
  `velnor-design-upgrade` (pre-crate-split monolith), the seven pre-rewrite
  `src/` WIP branches, `velnor-actions-0135` (superseded by 0136), and
  documentation-only/obsolete task branches.
- Deletion candidates are listed in the other-branches review with the
  protection rule applied; nothing was deleted in this increment.

## False parity inventory

Full inventory with file:line and mechanism:
[false-parity inventory](evidence/2026-10-21-pr38-completion/false-parity-inventory.md).

Confirmed still present on the pinned tip:

- **Jackin**: 26 `historical_*` functions and the 22-call dispatch tail in
  `crates/jackin-preview-app/src/app.rs`; fixed cell tables `const ENTRIES` in
  `accounts_add_form_view.rs`, `usage_detail_view.rs`,
  `usage_overview_view.rs`; frame-identity and exact-size predicates; 5 forced
  cursor registrations.
- **Holla**: 19 `*_plate` predicates and 17 `now_ms() == 80` tag frames in
  `holla-ui/src/app.rs` and `holla-sim/src/catalogue.rs`; the earlier
  handoff claim that production has no `*_plate` dispatch is false — the
  predicates were renamed from plates to width-band tag frames.
- **Showcase**: fixed cell tables in `showcase-demos` `taskrunner.rs` and
  `terminal.rs` (about 1 900 runs) plus a tick-settle gate.
- **TablePro**: `paint_legacy_tabs`, explorer micro-repaints, draw-order
  `thread_local` channels, and the 120×40 query-focus hardcode.

Order of removal (from the inventory): TablePro pickers → Showcase button
repairs → Jackin cell tables → Holla presentation selectors → residual
repaint/forced-cursor paths. None of these bypasses may be renamed or moved
into Termrock; each is replaced by the owning component and only then deleted.

## PR #26 behavior gaps

Full table: [PR26 component gap](evidence/2026-10-21-pr38-completion/pr26-component-gap.md).

Highest-priority reusable gaps still absent from Termrock: refused dismissal
with feedback (`Dismiss::allows` has no caller), popover fit/slide policy,
table column seams and drag autofit, `Dismiss::pass_through` wiring, and drag
autoscroll. Items that must **not** be ported (frozen-capture conflicts or
superseded owner decisions): modal backdrop DIM, neutral inactive selection,
`ScrollState` context default 1, compact check glyph.

## Open verification

### Conformance driver repairs (this increment)

`termrock-conformance --test conformance` moved from 23 failing cases on the
pinned tip to 10. No expectation was loosened; each change makes the shared
harness or the component's advertised contract match what the widget really
does.

- The driver opened an overlay only when `activation_id != control_id`. A
  widget whose control lives inside the layer and that declares an explicit
  `opener_id` (Dialog) was never opened, so activation, dynamic-binding and
  remap checks ran against a closed layer. `needs_layer_opened` now also
  consults `opener_id`, and `dynamic_bindings_follow_keymap` opens the layer
  before pressing the chord. `DialogCase` gained the existing
  `open_overlay` hook. Fixes `dialog::bindings_match_handled_keys` and
  `dialog::keyboard_and_mouse_activation_are_equivalent`.
- `bindings_match_handled_keys` now seeds each case from `C::fixture()`
  (default `Fixture::default()`), so a case whose widget cannot paint or
  register inside 30x6 can widen its base rect without special-casing a
  check. `HelpOverlayCase` uses a 30x10 rect.
- The chord-universe half of that check runs after focusing the control, but
  compared against the empty-state table only. It now also accepts the
  focused table, because an editor with `EditEntry::Immediate` really handles
  its edit table as soon as it is focused.
- `CodeEditor::bindings` advertises the edit table for `FOCUSED` when the
  entry mode is immediate and the editor is editable (mirrors its update).
- Composite cases declare the inner text editor's chords through
  `legacy_key_chords` (`INNER_EDITOR_CHORDS`): `field`, `form`,
  `text_input`, `text_area`, `filter_list`, `picker`. The outer typed table
  cannot name a command owned by the inner editor.
- `Picker::PARTS` now lists `Part::PLACEHOLDER`, which the picker styles
  through its query field.

### Remaining conformance failures (10)

Recorded with root cause; not fixed in this increment because each needs a
reference decision or a visual change beyond the harness:

- `filter_list::hover_does_not_steal_focus`, `picker::hover_does_not_steal_focus`:
  the key that starts query editing changes the hovered part, so the
  subsequent pointer move no longer marks the outer control `HOVERED`.
  Needs a rule for hover while an owned editor is editing.
- `picker::wheel_at_boundary_is_consumed_without_repaint`,
  `help_overlay::wheel_at_boundary_is_consumed_without_repaint`: the open
  layer's scroll target does not receive the wheel at the boundary, so the
  event is ignored instead of consumed. Needs the overlay scroll owner wired
  to the layer hit region.
- `grid::mono_states_are_distinguishable`,
  `grid_error_is_real_cell_decor_and_mono_distinct`: cell ERROR decor is
  colour-only; `NO_COLOR`/mono needs a glyph or modifier distinction.
- `scroll_region::mono_states_are_distinguishable`: the thumb paints
  `PRESSED` only from `ui.pressed_part(THUMB)`, so a reference-injected
  `PRESSED` state paints identically to rest in mono.
- `help_overlay::mono_states_are_distinguishable`: the card paints no focus
  affordance for `FOCUSED` at mono, so focused and plain frames are equal.
- `help_overlay::local_override_does_not_mutate_the_theme`: the card paints
  hardcoded ink (`ink(...)` with custom roles), so an instance part patch on
  `Part::CONTAINER` cannot reach resolution.
- `help_overlay::bindings_match_handled_keys`: the card declares `↑` on its
  own table, but after the layer opens focus belongs to the scroll/done
  child, so the root owner never receives the chord.

### Other gate red (pre-existing on the pinned tip)

- `termrock-conformance` digest suites: 225 `render_components` digest
  mismatches (`tree::default` is `94e7ce70bb65b804` in the blessed table and
  renders `68aca379bb96d54f`). Confirmed identical on
  `origin/parity/termrock-visual-baseline` tip `a55276b64`, so it is
  branch-wide and not caused by this increment. It is not accepted as green;
  it needs one classified visual decision for the whole component set, not a
  bless.
- `termrock-navigation::item_row_columns`: two pre-existing failures
  (`constructor_defaults_match_the_pre_policy_full_cell_fixture`,
  `owner_part_styles_win_over_painter_defaults_in_columns_mode`).
- Application paired comparison (tuiscotti) was not run in this increment;
  no application parity claim is made.

Full workspace nextest, clippy, and formatting were run on this source before
publication; exact results and the pushed commit are recorded in the handoff
entry for the increment.

## Status

| Requirement | Status |
| --- | --- |
| Exact product match (Showcase, TablePro, Jackin, Holla) | Incomplete — false-parity inventory above; no paired comparison in this increment |
| Reusable Termrock library; thin applications | Incomplete — picker contracts and theme fix landed; Jackin/Holla/Showcase/TablePro bypasses remain |
| Branch consolidation into PR #38 | In progress — ledger complete; two bounded integrations identified, six branches held for review |
| Reference integrity | Frozen tag, PR #26 and Tuiscotti pin unchanged |

## Integration onto tip `819ef8ef9` (2026-10-21, increment 2)

The increment-1 branch `wip/pr38-completion-verify-20261021`
(`8e976098d`) was merged into a sparse worktree based on
`origin/parity/termrock-visual-baseline` tip `819ef8ef9`
(merge `0fcfe6675`). One conflict, in `crates/termrock-overlays/src/picker.rs`:
the tip had already grown the `PickerLayout` row policy
(`PickerLayout::max_rows`, `Picker::max_rows`, searchable/footer/spare rows),
so the branch's simpler `Picker::max_rows` was dropped in favour of the tip
implementation. `Part::PLACEHOLDER` survives on the tip and the branch's
`Substring` filter policy merged cleanly.

After the merge:

- `cargo nextest run -p termrock-theme -p termrock-navigation -p termrock-overlays`:
  591 passed, 4 failed — the two known `item_row_columns` cases plus two
  tip-side red tests (below).
- `cargo nextest run -p termrock-conformance --test conformance`:
  923 passed, **12 failed**. The increment-1 driver repairs hold on the new
  tip (935 cases collected, up from 935; the earlier 23-failure count is gone).

The 12 remaining failures and their owners:

| Case | Cause | Owner |
| --- | --- | --- |
| `grid::mono_states_are_distinguishable`, `grid_error_is_real_cell_decor_and_mono_distinct` | ERROR decor is colour-only | grid workstream |
| `help_overlay::mono_states_are_distinguishable`, `help_overlay::local_override_does_not_mutate_the_theme`, `help_overlay::bindings_match_handled_keys`, `help_overlay::wheel_at_boundary_is_consumed_without_repaint` | hardcoded ink, colour-only FOCUSED, `↑` owner, wheel ownership | help workstream |
| `filter_list::hover_does_not_steal_focus`, `picker::hover_does_not_steal_focus` | hover restoration while the owned query editor edits | hover workstream |
| `picker::wheel_at_boundary_is_consumed_without_repaint` | overlay scroll owner not wired to the layer hit region | hover workstream |
| `scroll_region::mono_states_are_distinguishable` | `PRESSED` thumb carries state by colour only | scroll workstream |
| `completion::mono_states_are_distinguishable` | same thumb path through `ScrollRegion` | fixed by the scroll workstream |
| `registry::declared_parts_are_the_parts_actually_styled` | `Picker::PARTS` and `FilterList::PARTS` omitted `Part::PLACEHOLDER`, which both style | fixed here |

`registry` fix: `Part::PLACEHOLDER` added to `Picker::PARTS` and
`FilterList::PARTS`; the existing conformance case is the regression test and
now passes.

### Tip-side red tests observed (not caused by this merge)

- `termrock-overlays menu::tests::menu_bar_keeps_public_default_variant_overlay_and_local_patch_precedence`:
  expects `Rgb(7, 8, 9)` for the Title overlay background, tip commit
  `84d6477d7` resolves it through `MENU_BAR_TITLE_VARIANT` to `Rgb(28, 28, 31)`.
  The test expectation was not updated by that commit.
- `termrock-theme capability_palettes::all_73_junie_slots_match_reference_capability_output`:
  red on the merged tree; not touched by this merge, needs a separate look.

Both are recorded, not accepted as green, and were not modified to pass.
