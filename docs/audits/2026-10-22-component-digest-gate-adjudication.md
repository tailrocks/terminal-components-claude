# Component digest gate — which frozen rendering the expectations encode

- Date: 2026-10-22
- Branch: `wip/component-digest-drift-20261021` (base `origin/parity/termrock-visual-baseline` at `ef23c3303c`)
- Scope: `crates/termrock-conformance/tests/render_components.rs` (the
  `render::components::*` matrix) and the library crates that paint it.
- Status: **the product was corrected to the frozen expectations; the
  expectations were not touched.**

## The question

`crates/termrock-conformance/tests/baselines/components.txt` holds the
expected digest of every component scene. It was last written at
`032e9faa3a` ("restore: land the termrock crate split with all source work
(#24)") and restored to that exact blob by `4d7a358d3c` under the migration
record's D-011 ("Restore the pre-37f55475 component expectations pending
adjudication"). The blob is `3343d127cfe998069e28037b9abaa95a77f466a4`;
no `BLESS=1` run and no expectation edit happened on this branch.

Two readings were possible:

1. the digests are stale and the product is right, or
2. the product drifted and the digests are right.

The branch tip answered "1" in
`docs/audits/2026-10-22-component-digest-reference-decision.md`, with the
measurement "320 tests run: 318 passed, 2 failed". **That measurement does
not reproduce.** The tip measures:

```text
cargo nextest run -p termrock-conformance --all-targets -E 'test(/render::components/)'
ef23c3303c: 320 tests run: 74 passed, 246 failed
```

## What the frozen expectations encode

The decisive measurement is the tree that carries the expectations. At the
split commit the matrix is green against the very same blob:

```text
cd <worktree at 032e9faa3a>
cargo nextest run -p termrock-conformance --all-targets -E 'test(/render::components/)'
032e9faa3a: 320 tests run: 320 passed, 1758 skipped
```

So the digests in `components.txt` encode the rendering of `032e9faa3a`,
which paints the Junie muted ink `0x808080`, the faint ink `0x4d4d4d`, the
scrollbar track as `U+2502 │`, the danger ink `0xe44545`, and a restrained
syntax palette (keyword white, string and number in the secondary grey,
operator and punctuation in the muted grey).

That rendering is the one the admitted corpus carries as well:

| Evidence | Reading |
| --- | --- |
| `baselines/tuiscotti-v1/` (the admitted baseline store the conformance gates compare against) | muted `0x808080` → ANSI-256 244, faint `0x4d4d4d` → 238, track `U+2502`, danger `0xe44545` → 167 |
| `baselines/tuiscotti-v1/corpus-index.json` | corpus commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b` — the same commit the `visual-baseline-old` tag object points at |
| `docs/audits/2026-10-21-junie-palette-ladder.md` (the repo's own palette adjudication) | "two frozen references, and they disagree"; "the corpus pins the operative expected indexes; the tag pins the historical ones"; the shipped ladder is 244/238 |
| `crates/termrock-conformance/tests/baselines/components.txt` at `032e9faa3a` | the same 244/238 ladder, verified by the 320/320 run above |

The `visual-baseline` tag source (`4169783ae`) paints the newer ladder:
`0x8a8a8a` → 245, `0x737373` → 242, track `U+2506 ┆`, danger `0xf05d5d` → 203
(its own frames in the frozen source's snapshot store carry 73 cells of
`38;2;138;138;138` and 9 cells of `38;2;115;115;115` in
`showcase/pages/codeeditor/120x40/truecolor.ansi`). The corpus carries
neither: `0x8a8a8a` occurs zero times in it.

So the two frozen references disagree, and the *gate* compares against the
corpus, not against the tag. The tag source is the reference for the look
direction; the corpus and the component digests are the reference for what
the gate expects today. The tip's palette migration
(`ec783f6c5`, `1c17d8f262`) moved the product to the tag ladder without a
corpus re-capture, which is the one action D-011 forbids ("Red comparisons
against the restored expectations remain open product/reference conflicts;
they are not grounds to regenerate, bless, skip or weaken expectations").

That migration also moved the gate the wrong way:

| Tree | component digest gate | library suite (10 crates) |
| --- | --- | --- |
| `032e9faa3a` (split) | 320 run: 320 passed | — |
| `849d26dad` (pre-tip base) | 320 run: 89 passed, 231 failed | 1757 run: 1756 passed, 1 failed |
| `ef23c3303c` (tip) | 320 run: 74 passed, 246 failed | 1779 run: 1763 passed, 16 failed |

## The change landed on this branch

1. **`crates/termrock-theme`** — restore the corpus-era Junie palette:
   the truecolor foreground ladder (`0x808080` muted, `0x4d4d4d` faint), the
   restrained syntax tokens, the danger ink `0xe44545`, the status green on
   the accent cell, the field-hover surface `0x232328`, the scrollbar track
   `U+2502`, the ANSI-256 ladder (244/238, danger 167, disabled 238) and the
   ANSI-16/mono ladders (faint `DarkGray`), plus the `downgrade.rs` pins that
   produced them. The theme's own assertions
   (`builtin::tests::junie_tokens_equal_the_legacy_palette_exactly`,
   `downgrade::tests::*`, `glyph::tests::ascii_glyph_set_has_no_box_drawing`,
   `capability_palettes::all_73_junie_slots_match_reference_capability_output`,
   `empty_style_inheritance::*`, `theme_holla_states::*`) now state the
   restored values.
2. **`crates/termrock-controls`** — repair the button's focus-visibility
   policy and its pad painting. The tip left four red
   `button_contract` tests behind: the `Always` policy never promoted
   `FOCUS_VISIBLE` while the pointer was held down, it promoted focus inside
   an inert scope, and the pad cells no longer carried the container style.
   The button now promotes the focus bar for a pointer-focused button
   (`Always`), never promotes one in an inert scope, and paints its pad
   cells with the container style. The button hover plane keeps
   `Role::RaisedSurface`, so the disabled background no longer disappears
   under hover.

The slots the corpus never renders stay at the tag's values, because the
corpus carries no evidence for them (`docs/audits/2026-10-21-junie-palette-ladder.md`):
the field-hover surface `#2f2f34` (256 index 237), `highlight_bg` 25,
`highlight_danger_bg` 88, `danger_tint` 52 and `match_current_bg` 55.

Before/after on the same tree:

```text
ef23c3303c (tip):      320 tests run: 74 passed, 246 failed
wip/component-digest-drift-20261021: 320 tests run: 211 passed, 109 failed
```

The library suite over the same ten crates:

```text
ef23c3303c (tip):      1779 tests run: 1763 passed, 16 failed, 3 skipped
wip/component-digest-drift-20261021: 1779 tests run: 1774 passed, 5 failed, 3 skipped
```

`termrock-theme` alone: 194 run, 194 passed (the tip left eight red there).

## What is still red, and why

The remaining 104 failures are no longer palette drift. They are product
evolution after the expectations were written (the split, `032e9faa3a`, to
the tip, `ef23c3303c`), concentrated in the families the tip's week of work
touched: `picker`, `picker_chain`, `help_overlay`, `dialog`, `completion`,
`code_editor` (all eight states each), `split_pane`, `scroll_region`,
`menu_bar`, `grid`, `filter_list`, `context_menu` (seven states each) and a
dozen single-state cases (`button`, `tabs`, `steps`, `tree`, `toggle`,
`radio_group`, `nav_list`, `wizard`, `form`, `chip_bar`, `checkbox`).

They are recorded, not blessed: each is a real difference between the
product and the frozen expectation and needs either a product correction or
a per-key reference decision under D-011.

## The remaining library reds

Five tests stay red on this branch, four of them red at the tip as well:

| Test | State at the tip | Why it stays red here |
| --- | --- | --- |
| `termrock-navigation::item_row_columns constructor_defaults_match_the_pre_policy_full_cell_fixture` | red | a historical fixture that no current palette reproduces (see the table below) |
| `termrock-overlays picker::tests::query_field_resolves_picker_family_theme_and_part_patch` | red | untouched by this branch |
| `termrock-overlays::context_menu_danger_profile disabled_active_danger_profile_returns_to_popover_surface` | red | the disabled ink's ANSI-256 projection: the corpus renders 238, the tag's frozen cells render 239 |
| `termrock-overlays::context_menu_danger_profile keyboard_active_danger_row_uses_frozen_background_ladder` | red | the same projection conflict on the Mono tier |
| `termrock-overlays::context_menu_danger_profile disabled_danger_row_matches_frozen_cells_at_rest_hover_and_press` | green at the tip | the same projection conflict: the tip mixed 239 and 238 for the disabled ink, the restored palette projects it uniformly to 238 |

That last row is the only library red this branch adds. It is the
tag-versus-corpus conflict in one slot: the corpus renders the disabled ink
`0x4d4d4d` as 238 and never paints 239, the tag's `apply_256` pins it to 239,
and the three danger-profile tests assert the tag's projection. The
corpus decides here, as it does for every other slot it renders, and the
three tests stay red until the coordinator rules on the slot.

## The named neighbouring failures

| Test | State at the tip | State on this branch |
| --- | --- | --- |
| `termrock-theme::capability_palettes all_73_junie_slots_match_reference_capability_output` | red (the tip's ladder contradicted the reference formula) | green, restated against the corpus ladder |
| `termrock-overlays menu::tests::menu_bar_keeps_public_default_variant_overlay_and_local_patch_precedence` | green | green, untouched |
| `termrock-editors::completing_code_editor focus_loss_dismisses_completion_but_preserves_configured_edit_session` | green | green, untouched |
| `termrock-editors code::tests::focus_out_commits_by_default_but_can_preserve_the_edit_session` | green | green, untouched |
| `termrock-navigation::item_row_columns constructor_defaults_match_the_pre_policy_full_cell_fixture` | red (`left: 4189130516631402064`, `right: 5235702202963140133`) | red (`left: 9596688686787365186`, `right: 5235702202963140133`) — a historical fixture, not a regression from this branch |

The item-row fixture needs its own reading. `docs/audits/2026-10-21-item-row-historical-palette-fixture.md`
records it as "a historical library-constructor fixture, not the frozen
application visual oracle": its expected full-cell fingerprint
(`5235702202963140133`) was captured from one pre-policy rendering, and no
palette the product has painted since reproduces it — the tip's ladder gives
`4189130516631402064`, the restored corpus-era ladder gives
`9596688686787365186`. The fingerprint therefore needs a per-key decision
before it can be restored or re-recorded; this branch leaves it untouched
and red, as it was at the tip.

## The request that remains with the coordinator

**Reference-decision request.** The two frozen references disagree about the
Junie palette, and the disagreement is now visible in the gates:

| slot | the corpus renders | the tag's frames render | this branch |
| --- | --- | --- | --- |
| muted ink | `0x808080` (244) | `0x8a8a8a` (245) | corpus |
| faint ink | `0x4d4d4d` (238) | `0x737373` (242) | corpus |
| scrollbar track | `U+2502` | `U+2506` | corpus |
| danger ink | `0xe44545` (167) | `0xf05d5d` (203) | corpus |
| disabled ink (256) | 238 | 239 | corpus |
| syntax | restrained grey ladder | blue/teal/lavender | corpus |
| field hover | never rendered | `#2f2f34` (237) | tag |

Every slot the corpus renders follows the corpus; every slot the corpus
never renders follows the tag. The coordinator has to rule which reference
the product's look follows:

* **If the corpus wins** (this branch's reading, and the one the component
  digests and the admitted baseline store carry), the branch stands as it is
  and the remaining 109 component failures are the product differences left
  to fix.
* **If the tag wins**, then `baselines/tuiscotti-v1/`,
  `crates/termrock-conformance/tests/baselines/components.txt` and the three
  danger-profile expectations have to be re-captured from a build that
  paints the tag ladder, under the D-011 process and with an independent
  review of the exact changed keys. That is a blessing action reserved to
  the coordinator; this branch does not pre-empt it, and the palette
  restoration on this branch is the piece that would then be undone.

Either way the expectations on this branch are untouched: no `BLESS=1`, no
expectation edit, and `components.txt` stays at blob
`3343d127cfe998069e28037b9abaa95a77f466a4`.

The corpus and the tag disagree about the Junie palette, and only one of
them can be the gate's expectation at a time. This branch keeps the gate's
expectation intact, because changing it is a blessing action that D-011
reserves for the reference decision and because no expectation may be edited
here. If the coordinator rules that the tag ladder (`0x8a8a8a` / `0x737373`
/ `U+2506 ┆` / `0xf05d5d`) is the product's look, then the corpus
(`baselines/tuiscotti-v1/`) and the component digests have to be re-captured
from a build that paints that ladder, under the D-011 process and with an
independent review of the exact changed keys. That is a separate, explicit
decision; this branch does not pre-empt it.

## Verification

```text
cargo nextest run -p termrock-conformance --all-targets -E 'test(/render::components/)'
cargo nextest run -p termrock-theme -p termrock-controls -p termrock-navigation \
  -p termrock-overlays -p termrock-runtime -p termrock-grid -p termrock-editors \
  -p termrock-forms -p termrock-fields -p termrock-feedback --all-targets
cargo clippy -p termrock-theme -p termrock-controls --all-targets -- -D warnings
cargo fmt --check
python3 tools/acceptance_architecture.py
```

`crates/termrock-conformance/tests/baselines/components.txt` is byte-identical
to `3343d127cfe998069e28037b9abaa95a77f466a4` on this branch.
