# Dim wave 3 — per-source-ink chrome dim at level tokens (2026-10-22)

Work tree: `wip/dim3-per-source-ink-20261022` (detached from
`origin/parity/termrock-visual-baseline` at 8a01e304be). Oracle: the
regenerated frozen frames under `snapshots/` (byte-identical captures of
the frozen tag binaries). All comparisons read-only: no bless, no
expected-output edits.

## Tally

Full 92-case target list (`/tmp/target_cases.json`): **70 FAIL / 22 PASS**
(wave-2 base, 8a01e304be) → **65 FAIL / 27 PASS** (this wave, nextest
Summary over the exact filter list, chunked).

Spot diff sizes for `tablepro::overlays::picker_open::80x24` (before →
after, expected-vs-actual cell diffs parsed from the ANSI frames):

- 256: 175 → 32
- 16: 69 → 4 (only the four recorded guard-rail artifacts remain)
- none (mono): 267 → 202
- nocolor: 190 → 120
- truecolor: 15 → 0 (the ANSI frame now matches; the case still fails on
  a non-frame capture gate, same as other truecolor cases)

## Changes

1. **junie_256 ghost/border/backdrop slots authored at cube cell 238**
   (crates/termrock-theme/src/palettes.rs). The frozen 256 dim ghost and
   quiet borders render as i(238) (apply_256 `text_ghost` /
   `border_subtle`), not the source #262626 cell 235. Fixed the
   143-cell covered-blank mass and the popup-2 frame at 256.

2. **Shared concrete-color dim contract `Theme::backdrop_cell` /
   `Theme::backdrop_dim_level`** (crates/termrock-theme/src/lib.rs).
   `backdrop_colors` delegates to it. The picker/dialog chrome fill
   (crates/termrock-runtime/src/support.rs) now writes the mapped ink and
   the SGR-2 DIM modifier: at Ansi16/Mono every covered cell carries the
   muted dim ink (the frozen DIM-modded underlays), blanks included; at
   truecolor/256 the blank skip and the fg==bg collision lift are
   unchanged.

Regression tests: `junie_256_authors_the_frozen_ghost_and_border_subtle_cells`,
`backdrop_cell_rewrites_the_dim_ink_only_at_reduced_levels`
(termrock-theme), `dialog_chrome_carries_the_dim_modifier_only_at_reduced_levels`
(termrock-runtime). All fail on the pre-fix code.

## Remaining classes (not fixed here, with reasons)

- **Scope-label META ink** (256: exp i245 act i238, x61-75 y3; tc: exp
  #8a8a8a act ghost): the earlier PICKER `Part::META → Muted` recipe
  attempt regressed list rows and was reverted; this needs the scoped
  title-row-only recipe from the wave-2 audit (sub-item 5c).
- **Page-source divergence under the popup title row** (256: (1,3) and
  (3,3) exp i242 act i238; the x=3 column y5-19 exp i238 act i242): the
  covered cells dim correctly for their source inks; the candidate page
  holds different source inks there than the frozen binary (tab-strip
  rule ink resolution), so the fix belongs to the page paint, not the
  dim mapping.
- **Popup-2 and outside-popup dim coverage at mono/nocolor** (116-cell
  class exp (i7,d) act undimmed; 22-cell hint class; 13 scrollbar-track
  cells): popup-1's backdrop dims only the page cells its chrome covers;
  the frozen binary dims the full screen including the lower popup's
  frame and the page outside the popup area. Needs the full-screen
  underlay dim, not the chrome fill.
- **Selection/focus classes at mono** (46-cell selected-row class exp
  (i0,i8,7bd); focus bar (4,4) exp i15d): distinct wave items, unchanged
  by this wave.
- **Guard-rail artifacts kept**: (6,5), (12,5), (22,5) blank fg==bg
  skips and the (6,6) undimmed safemode border cell; the dim-level
  rewrite now lands (i7,d) on them (16) / keeps them (256), still red,
  count unchanged.

Pre-existing reds left untouched: the two adjudication-pending
termrock-theme tests (`standalone_empty_cells...` fingerprint,
`a_hovered_disabled_part...`) and the termrock-conformance digest reds,
which reproduce at the base commit.

Landing amendment (2026-10-22, parity tip): the frozen-slot override
lines for `fg[4]`, `border_subtle` and `backdrop_fg` at Ansi256 were
registered in the
`all_73_junie_slots_match_reference_capability_output` reference test
(crates/termrock-theme/tests/capability_palettes.rs) with the
frozen-corpus citation above, as part of landing the wave on
`origin/parity/termrock-visual-baseline`; the test fails on the
pre-amendment wave tip d39b09e607 and passes with the lines.
