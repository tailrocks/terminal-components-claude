# Junie capability palette ladder vs the frozen references

- Date: 2026-10-21
- Scope: `crates/termrock-theme/src/palettes.rs` (`junie_256`, `junie_16`, `junie_mono`),
  `crates/termrock-theme/tests/capability_palettes.rs`.
- Method: every frozen frame in `baselines/tuiscotti-v1/**` (1510 scenario/tier
  dirs x truecolor/256/16/none) was decoded and cross-mapped cell by cell
  (truecolor RGB -> per-tier index). The `visual-baseline` tag
  (4169783ae) `.ansi` streams were histogrammed separately (3070 SGR files).

## Result: two frozen references, and they disagree

| Reference | Content | junie muted (fg[2] source) | faint (fg[3] source) | ghost text | danger | on_accent | border_strong / disabled | accent_pressed bg | accent_tint bg |
|---|---|---|---|---|---|---|---|---|---|
| tag `visual-baseline` (4169783ae) `.ansi` + its `src/theme.rs` `apply_256` | pre-refactor app, authored ladder | 0x8a8a8a -> 245 | 0x737373 -> 242 | 0x262626 -> 238 (`text_ghost = i(238)`) | 0xf05d5d -> 203 (`error = i(203)`, 1145 fg cells) | 232 | 242 / 239 | 65 (`accent_pressed = i(65)`) | 22 (`accent_bg = i(22)`) |
| branch corpus `baselines/tuiscotti-v1` (admitted, sha in `corpus-index.json`, the gate oracle via `DEFAULT_BASELINE_STORE`) | refactor-line replay (truecolor tokens = current builtin) | 0x808080 -> 244 (1_011_55+ cells) | 0x4d4d4d -> 238 | mixed: 0x262626 -> 235 (215 439 backdrop/dim cells) and -> 238 (1244 live ghost cells: jackin/capsule/new-tab 120x40 (1,6) '●', tablepro/table/filter-editor line numbers) | 0xe44545 -> 167 (20 000 fg / 486 bg cells; index 203 appears nowhere) | 0x19191c -> 233 (9026 cells) | 0x4d4d4d -> 238 (245 347 cells; 239 appears nowhere) | 0x2b8632 bg -> 238 (140 cells: showcase focus gutters) | 0x0f2e13 bg -> 233 (32 436 cells) |

The corpus is not a lossless conversion of the tag: scenario text differs
(`accounts/add_form_required` vs `accounts/add-form-required` txt/ansi diffs),
and the tag's truecolor red is 0xf05d5d while the corpus's is 0xe44545. The
gate (`termrock-conformance` visual baseline) compares against the corpus, so
the corpus pins the operative expected indexes; the tag pins the historical
ones. Per the reference-defect rule both are preserved here; nothing was
silently resolved and no expected frame was edited.

## Ladder shipped (junie_256)

Generic frozen cube/gray rule everywhere except:

| Slot | Index | Evidence |
|---|---|---|
| fg[2] muted | 244 | corpus 0x808080 fg -> 244, uniform (1_011_55+ cells, all apps) |
| fg[3] faint | 238 | corpus 0x4d4d4d fg -> 238, uniform (245 347 cells) |
| fg[4] ghost | 235 | corpus backdrop/dim ghost 0x262626 -> 235 (215 439 cells); see conflict below |
| on_accent | 233 | corpus 0x19191c fg -> 233, uniform (9026 cells); equals the generic rule |
| border_strong | 238 | corpus dialog borders 0x4d4d4d -> 238 (jackin cockpit cancel-confirm 120x40: 321 border cells) |
| disabled_fg | 238 | corpus renders no 239 anywhere; 0x4d4d4d -> 238 |
| field_hover | 237 | tag `apply_256 field_hover = i(237)`; corpus never renders the slot |
| accent_hover | 77 | corpus 0x3ab343 bg -> 77 (288 cells) |
| accent_pressed | 238 | corpus 0x2b8632 bg -> 238 (140 cells) |
| accent_tint | 233 | corpus 0x0f2e13 bg -> 233 (32 436 cells) |
| focus_ring | 238 | same 140 focus-gutter cells as accent_pressed |
| syntax.match_bg | 233 | same 32 436 cells as accent_tint (shared source 0x0f2e13) |
| syntax.match_current_bg | 55 | tag `highlight_on_selection = i(55)`; corpus never renders the slot |
| highlight_bg | 25 | tag `highlight = i(25)` (tag frames render 48;5;25 / 2165 bg cells); corpus never renders the slot |
| highlight_danger_bg | 88 | tag `highlight_danger = i(88)` (tag frames render 48;5;88); corpus never renders the slot |
| danger_tint | 52 | tag `error_bg = i(52)`; equals the generic rule; corpus never renders the slot |
| meter stale/unknown | 238 | corpus faint/chrome ink (shared 0x4d4d4d-era source) -> 238 |

## Ladder shipped (junie_16)

Generic projection everywhere except `fg[3] = DarkGray` (corpus renders the
faint/chrome ink as index 8, uniform, 245 436 cells; the current 0x737373
source would generically land on Gray) and the pre-existing
`field_hover = DarkGray` (matches the generic rule).

## Ladder shipped (junie_mono)

Unchanged. The corpus none-tier matches the authored table
(legacy context rules live in the render path, not the palette).

## Recorded conflicts (reference defects, both sides preserved)

1. Ghost slot: tag renders 238; the corpus renders 235 for backdrop/dim
   ghost cells (215 439) and 238 for live ghost text (1244). One palette
   slot cannot emit both; 235 keeps the dominant corpus set green. The 1244
   live-ghost cells stay a known corpus-side red.
2. Danger: tag renders 203 (and 210 for strong) from source 0xf05d5d; the
   corpus renders 167 from source 0xe44545. Truecolor must stay
   byte-identical, so the corpus index (167) is kept; switching to 203 would
   turn ~20k gated cells red.
3. Muted/faint/border/disabled/on_accent: the tag's authored table (245/242/
   242/239/232) reflects tag-era source RGBs (0x8a8a8a/0x737373/WHITE_30/
   ON_GREEN via `i(232)`); the corpus pins 244/238/238/238/233 for the same
   roles. Corpus wins while it is the gate oracle.
4. accent_pressed/accent_tint/selection_bg/success/info: tag authored
   65/22/17/71/111; the corpus pins 238/233/237/78/147. Corpus wins;
   tag values recorded here for the future corpus re-approval.

A future corpus re-approval (deliberate visual change) can adopt the tag
ladder by swapping the override table in
`crates/termrock-theme/tests/capability_palettes.rs`; every slot above cites
its evidence.

## Verification

- `cargo nextest run -p termrock-theme`: `all_73_junie_slots_match_reference_
  capability_output` green (successor of the pre-existing red); all other
  capability-path tests stay green. `empty_style_inheritance::
  standalone_empty_cells_match_pre_inheritance_reference` fails on the base
  commit too (pre-existing, untouched).
- `cargo nextest run -p termrock-conformance`: failure set compared sorted
  against the base commit; delta recorded in the PR description.
- Truecolor: no `builtin` token or truecolor-path change; the ladder only
  authors `Ansi256`/`Ansi16` projections.
