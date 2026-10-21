# Dim-tier parity progress — app/theme streams (2026-10-22)

Working tree: detached worktree at `origin/parity/termrock-visual-baseline`
(bdf62e3be). Oracle: regenerated modal-backdrop frames under `snapshots/`
(see `2026-10-21-backdrop-frame-regeneration.md` and the adjudication +
independent-review records). All comparisons read-only: no bless, no
expected-output edits.

## Baseline

Measured at bdf62e3be over the 92-case dialog/picker target list
(`/tmp/target_cases.json`): **88 FAIL / 4 PASS**.

## Root causes found and fixed

1. **Modal-content dim missing at Ansi16/Mono** (dominant; most of the 88).
   The frozen renderer dims the modal layer's own written cells at 16 and
   mono levels, preserving fg/bg/bold. Fixed in the composite path:
   `LayerDraw::composite_onto` gained a `content_dim` parameter
   (crates/termrock-runtime/src/ui/layer_buf.rs) applied to the layer's own
   content cells only — cells composed through an underlay intent (the dimmed
   backdrop region) keep the underlay treatment without the dim. Dimmed
   bright black is lifted 8 → 7 on the Ansi16 ladder.

2. **Hint-line truncation off-by-one.** Candidate clipped an over-long picker
   hint line hard at the popup edge (one extra char, no ellipsis); frozen
   elides with `…`. Fixed by truncating to the content width in
   crates/termrock-overlays/src/picker.rs before `paint_str`.

3. **Footer status green tier inks.** The earlier `nearest_256` /
   `nearest_16` rewrite was reverted (it broke the frozen general-rule tests,
   e.g. (72,224,84) → 78). Correct mechanism: token overrides. Frozen
   `apply_256` sets `success = i(71)` and `apply_16` sets
   `success = Green` (tag `src/theme.rs:952` and `:1023`); junie_256 /
   junie_16 palettes now author those slots
   (crates/termrock-theme/src/palettes.rs), the reference test
   (`capability_palettes.rs`) pins them, and tablepro paints the status
   through the theme slot (`TAG_OK_FG = Role::Success`,
   crates/tablepro-ui/src/app.rs) instead of a custom `#3fb950` hex.

4. **Detail-column truncation in the Columns item layout.** Pickers
   (`ItemRowLayout::Columns`, `ItemColumns::paint`) hard-clipped an
   over-long detail at the flex-cell boundary. Frozen elides with `…` at
   `room = min(detail_col, rx - (dx + 1))` (tag `src/widgets/picker.rs:884`).
   Fixed by truncating with `termrock_text::truncate` to
   `min(solved_detail_col, flex + trailing − 2)` in
   crates/termrock-navigation/src/item.rs (`ItemColumns::paint`;
   `solve_label` now returns the solved detail column). The `ItemRow::row`
   compact path got the same end-ellipsis rule.

## Scenario results (80x24 spot set)

- `tablepro/overlays/safemode_picker/80x24/16`: 294 → 1 differing cell
  (class 1 below).
- `tablepro/overlays/picker_open/80x24/16`: 419 → 10 differing cells
  (classes 1–3 below).
- full 92-case tally: 88 FAIL / 4 PASS by count; per-case diffs collapsed
  (spot numbers above; the 4 passing cases are unchanged and no case
  regressed).

## Remaining reds and hypotheses

1. `tablepro/overlays/safemode_picker/*/16|none|nocolor`, cell (6,6): a single
   popup-border cell (`─`) frozen leaves **undimmed** (fg indexed 7, all three
   low tiers; not present at 256/truecolor). No other border cell is exempt,
   and the same column is dimmed on every other row, so it is not a seam or
   column carve-out. Hypothesis: a per-cell repaint artifact of the frozen
   render (one cell repainted after the dim pass), exact mechanism not found
   in the tag source (its only dim sites are the underlay `modal_backdrop_with`
   and `fade.rs`). 1 cell × 3 tiers.

2. Blank fg==bg cells frozen leaves undimmed, dimmed by the candidate
   blanket: `picker_open` popup-1 blanks (6,5), (12,5), (22,5) and the
   safemode lower-card cells (12,5), (22,5), thumb (6,5). The tag dims
   already-painted lower-layer content inside `modal_backdrop_with` and skips
   blank fg==bg cells; the candidate's per-written-cell dim cannot express
   that skip without re-introducing a worse mismatch on the top picker's own
   blank gutter cells, which frozen does dim.

3. Scaffold scroll-track cells (75,13)…(75,18) in `picker_open` low tiers:
   the candidate composes the grid scroll track into the modal layer buffer,
   so the Ansi16 8 → 7 lift reaches it (`2;38;5;7`); the frozen frame keeps
   the track ink at `2;38;5;8` (its `scroll_track` slot is authored DarkGray
   at 16 and the track cells are underlay, not modal content). Without the
   lift the modal borders regress by hundreds of cells, so the lift stays;
   6 cells per frame remain.

4. 256/truecolor-tier dialogs and picker content drift (dialog text/tab
   order, junie header muted/faint inks #808080/#4d4d4d expected vs
   #8a8a8a/#737373 actual) — tracked separately from dim work.

## Wave 2 (2026-10-22, continuation stream)

Base: `de6b004f6c` (restoration stream tip). Commits:
`7b38ba9e68` (runtime), `233b00824a` (theme).

**Correction to "Root causes" 1.** The blanket Ansi16 8 → 7 dim lift is
removed (`7b38ba9e68`). The lift was scoped too broadly: it also lifted the
popup scroll-track cells (`2;38;5;8` frozen), which the modal layer writes
as plain content, and the Mono tier. `composite_onto` now carries
`content_dim: bool` (SGR 2 on the layer's own written cells, fg/bg/bold
preserved) with **no ink rewrite**; the grey inks come from authored theme
slots instead (`233b00824a`): tag `apply_16` / `apply_mono` set
`border_strong = Gray` and `text_faint = Gray` (tag `src/theme.rs`
apply_16/apply_mono), while `scroll_track` stays DarkGray — the junie_16 /
junie_mono palettes and the `capability_palettes` reference test author
those slots exactly.

**Family dispositions (task families 1–5).**

1. safemode single undimmed border cell (6,6), 3 low tiers — **RECORDED**
   (mechanism not in tag source; positional artifact). Now the only
   remaining red of `safemode_picker/80x24/16` (97 → 2 cells mid-wave after
   an incorrect first split, → 2 unique prints = the single (6,6) cell).
2. blank fg==bg cells — **RECORDED** (value-based dim cannot express the
   frozen skip; x=4 gutter cells frozen dims in the same frames).
3. scaffold scroll-track cells (75,13–18) — **FIXED** by the lift removal +
   slot authoring above: the track stays DarkGray and the popup border/hint
   ink go Gray without the composite hack. `picker_open/80x24/16` remaining
   69 cells are the title-row class (see family 5) + the recorded blank /
   (6,6) classes; the track cells are no longer in any diff.
4. junie header muted/faint inks — **FIXED upstream** by the restoration
   stream (`b4b1a33468`, junie_256 `fg[2] = 245` / `fg[3] = 242`); nothing
   left in this lane.
5. 256/truecolor dialogs content drift — **DIAGNOSED, PARTIALLY RECORDED**.
   `picker_open/80x24/256` is 388 → 175 after the stream's
   `baseline_ansi256` restore. Remaining classes, all shared-surface:
   (a) picker/dialog title-row padding blanks render `backdrop_fg` (235)
   where frozen keeps faint 242 / muted 245 per source ink — the chrome
   fill maps every covered page cell through the single `backdrop_fg`
   token (`support.rs` backdrop-chrome path); frozen instead dims the
   page ink per cell. Needs per-source-ink backdrop mapping (same shape as
   the stream's `baseline_ansi256` table), not a token value change.
   (b) the quick-switcher title bar (`━` frame + `│` edge) paints
   `border_subtle` (235) where frozen paints 242. (c) scope-label chips:
   frozen 245; the candidate's PICKER recipe has no META part (row_like
   skips META for picker rows), so it falls to ghost — an attempt to author
   `Part::META → Muted` on the PICKER recipe regressed the list-row detail
   cells (safemode 16 2 → 50, picker 16 69 → 77, picker 256 175 → 183) and
   was reverted; the scope label needs a title-row-specific part.

**Mono ('none' tier) mass class, re-measured.** The wave-1 "mass none
red" shrank 555 → 389 (safemode) and 475 → 292 (picker) from the mono
`border_strong = Gray` slot alone. Remaining classes, both in the dimmed
underlay: (a) frozen preserves the page ink under the dim (Gray text →
`#898989`+dim, White text → `#b2b2b2`+dim); the candidate's popup-card
chrome fill maps every covered page cell to the mono `backdrop_fg`
(Black) **without dim** — header, rules and status chrome under the card
(~350 cells). Same per-source-ink backdrop gap as 5(a), Mono face.
(b) the selected tab strip keeps its DarkGray field under the dim in the
frozen frame; the candidate drops the field to black.

**Pre-existing reds at the base commit (not from this delta).** On clean
`de6b004f6c` sources, `termrock-theme` has two failing tests:
`empty_style_inheritance::standalone_empty_cells_match_pre_inheritance_reference`
(fingerprint 8680504039722522311 vs expected 312677554172290677,
`crates/termrock-theme/tests/empty_style_inheritance.rs:289`) and
`builtin::tests::a_hovered_disabled_part_keeps_every_slot_the_disabled_rule_writes`
(BUTTON/SUBTLE/CONTAINER disabled bg now survives hover; the
`DISABLED_BG_LOST_TO_HOVER` entry must be deleted once adjudicated). Both
reproduce with the stream's own `crates/termrock-theme` checked out, so
they belong to the restoration stream's lane.

**Tally.** Full 92-case target list (`/tmp/target_cases.json`):
88 FAIL / 4 PASS (wave-1 base) → **70 FAIL / 22 PASS**
(`nextest` Summary over the exact filter list). Spot counts
(before → after this wave, same 80x24 cases): safemode 16: 1 → 1
(the recorded (6,6) artifact); safemode none: 555 → 389;
safemode nocolor: 338 → 340 (two residual class-2 cells);
picker 16: 10 → 69 (see correction above: wave-1's 10 came with the
over-broad lift; the honest no-lift residual is 69, dominated by the
title-row class); picker none: 475 → 292; picker nocolor: 188 → 190;
picker 256: 388 → 175. The new passes are the truecolor and small/nocolor
dialog and picker cases listed by `nextest`.
