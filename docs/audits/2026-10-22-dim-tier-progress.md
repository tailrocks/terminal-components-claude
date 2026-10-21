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
