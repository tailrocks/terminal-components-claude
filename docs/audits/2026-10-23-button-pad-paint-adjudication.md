# Button pad-paint adjudication (2026-10-23)

## The conflict

Commit `d1ac0c0dac` ("fix(showcase): match the buttons audit chrome",
2026-10-22 00:58 +0700) rewrote `Button` pad painting in
`crates/termrock-controls/src/button.rs` as part of a showcase-chrome fix.
The pressed-focus adjudication
(`docs/audits/2026-10-23-button-pressed-focus-adjudication.md`) stopped here:
after its repair, `button_contract` stayed 18/19 with
`owner_wide_reference_styles_every_part_without_live_registration_or_cursor`
red, and the `render_components` `button::default`/`button::pressed` digests
stayed red with byte-identical got-digests on both sides of that conflict.

`d1ac0c0dac`'s button-paint changes, in detail:

1. The area fill changed from the container style to a "pad" style —
   container foreground replaced by the container background, BOLD removed.
2. `Part::GUTTER` blanks (`Slot::Inherit | Slot::Clear`) stopped being
   filled with the gutter part style, relying on the pad area fill.
3. A fill of the container style was added on the cell after the checked
   marker ("label ink, not the button fill").
4. A fill of `label.over(container)` was added on the trailing pad cell
   ("the trailing pad keeps the label ink").

## What the frozen oracle says

The frozen tag is `4169783ae` (tag object `2718a6b81`, commit
`4169783ae883788186d161af09ba9c481bd84e5c`).

### Frozen source (`4169783ae:src/widgets/button.rs`, `src/theme.rs`)

The oracle button clips to its natural width (`w = width().min(area.width)`)
and paints only three runs: the gutter cell (`buf.set_string(area.x, …,
it.gutter, gutter_style)`), the text run (marker/dot lane and label in one
pass with the body `style`), and a trailing blank
(`buf.set_string(area.x + w - 1, area.y, " ", style)`). There are no
interior pad cells: the oracle never stretches a button (ButtonRow places
every button at its natural width; a wider area is simply left unpainted).

`Theme::interactive()` resolves, for a non-glyph kind:

- `style = fg(fg).bg(bg).add_modifier(mods)` — the body style. `PRIMARY` is
  `weighted`, so `mods` includes `m.focused = Modifier::BOLD` at every tier.
- `gutter = " "` when no focus bar shows, with
  `gutter_style = fg(gutter_fg).bg(bg)` and `gutter_fg = bg` — the blank
  gutter cell is **bg-on-bg without BOLD**.

So the oracle paints every non-gutter cell of a button with the body style
(container ink, container background, BOLD kept) and only the gutter cell
with the background as its own foreground.

### Frozen frame bytes (`snapshots/showcase/audit/buttons/100x30/truecolor.ansi`)

Decoded per cell (SGR stream, escape-stripped read-only):

- Primary "Run task" row: gutter cell `fg=(72,224,84) bg=(72,224,84)
  bold=false`; label cells `fg=(25,25,28) bg=(72,224,84) bold=true`;
  trailing cell (x=32) `fg=(25,25,28) bg=(72,224,84) bold=true`.
- Toggle row ("○ Auto-approve"): the blank after the dot is
  `fg=(255,255,255) bg=(39,39,42)` — the container ink (the dot and its
  trailing space are painted in one body-style pass), not bg-on-bg.
- Disabled row ("Disabled primary"): gutter `fg=(39,39,42) bg=(39,39,42)`
  (bg-on-bg); label and trailing cells `fg=(77,77,77)` (the disabled
  container ink), BOLD off — all one disabled body style.

This confirms the source reading byte-for-byte: gutter blank = bg-on-bg,
every other cell = container/body style.

## Findings on each of `d1ac0c0dac`'s paint changes

1. **Gutter blank bg-on-bg — oracle-correct, kept.** Pre-`d1ac0c0dac` the
   blank was filled with the gutter part style, which resolves to the
   container style (measured pre-change frame: `fg=(25,25,28) bg=green
   BOLD`), contradicting the frozen `fg(bg).bg(bg)` blank. `d1ac0c0dac`'s
   change fixed this cell; the frozen frames show bg-on-bg blanks for
   primary, toggle and disabled buttons alike.
2. **Pad area fill (fg = bg, BOLD removed) — regression, reverted.** The pad
   style reaches cells the oracle body-styles: lane gap cells (e.g. the cell
   between the status icon and the checked marker). There it replaces
   container ink — including an owner-wide `Part::CONTAINER` patch — with
   the background color. This is what broke
   `owner_wide_reference_styles_every_part_without_live_registration_or_cursor`
   (`part style at x=2`: actual background white vs the patched container
   fg `0x112233`), a recorded contract
   (`docs/audits/2026-10-21-button-shared-contract.md`: owner-wide
   `Ui::reference` styles the actual Button parts).
3. **After-marker container-ink fill — redundant, removed.** With the area
   fill restored to the container style, the covered cell is painted anyway.
4. **Trailing label-ink fill — regression, removed.** The frozen trailing
   cell is a body-styled cell: container ink with BOLD kept (frame x=32).
   `label.over(container)` drops the container BOLD where the label fills
   its whole lane (the digest scenario frames: trailing modifier 0 vs the
   baseline's 1), and it would redirect the cell to the `Part::LABEL`
   channel, contradicting both the oracle (one body style) and the
   part-patch contract (the trailing pad is container-styled).

## Verdict

`d1ac0c0dac`'s button-paint rewrite is **not** a deliberate shared-layer
change to keep: it mixed one oracle-correct fix (gutter blank) with two
regressions (pad fg override, trailing label-ink override) against the
frozen frames and the recorded part-patch contract, with no decision record.
The repair restores the frozen paint: container-style area fill, bg-on-bg
gutter blank, no special-casing of the after-marker or trailing cells.

## The digests are a defective reference, not a code problem

With the repair applied, the `render_components` button rows still fail with
got-digests **byte-identical to tip** (paired runs, this lane):

| Row (first failing per scenario) | Baseline | Got (tip) | Got (after repair) |
|---|---|---|---|
| `button::default 120 40 junie truecolor` | `7cad0003dd9cd1fb` | `2c4fd377d36d6c75` | `2c4fd377d36d6c75` |
| `button::pressed 120 40 junie truecolor` | `c04d0deb30b15747` | `1e7c2be1435e6d13` | `1e7c2be1435e6d13` |

All eight `button::*` scenarios are red at tip and stay red with identical
got-digests after the repair (`disabled` `3f782a734dc10cfd`→`75c2eee429cc3159`,
`editing` `7cad0003dd9cd1fb`→`2c4fd377d36d6c75`, `empty`
`ff907b36ea83dd9d`→`4817e184286edbc7`, `focused`
`eee5d0405c09ef07`→`ea73d954fa02a17e`, `hovered`
`06853fb0452f390f`→`cde439e3c593114d`, `selected`
`c3a5557f65a5ff3f`→`5b63d54220704690`).

Mechanism: the stored baselines were blessed from the pre-`d1ac0c0dac`
paint, whose gutter blank carried the container ink (fg = OnAccent, BOLD) —
bytes that the frozen frames contradict (bg-on-bg, no BOLD). The digest
delta between baseline and tip/repair is exactly that gutter cell; the
label/trailing cells are byte-stable across all three paints for these
scenarios. Making the rows green from the product side would require
painting the blank with container ink again — the non-frozen bytes this
adjudication just removed.

Disposition: **reference defect, owner decision owed** — same closure shape
as the holla case
(`docs/audits/2026-10-22-holla-reference-regeneration-case.md`): the
affected `crates/termrock-conformance/tests/baselines/components.txt` button
rows (8 scenarios × 2 grids × 2 themes × 2 color levels = 64 rows) are
regenerated only after the owner records the approval at a pinned HEAD. No
BLESS was run by this lane; no expected store was edited.

## Additional finding: `gutter_recipe_gate` blank matrix (red at tip, missed earlier)

`crates/termrock-controls/tests/gutter_recipe_gate.rs`
(`button_adoptable_states_match_the_helper_cell`, authored 2026-10-21 15:30,
before `d1ac0c0dac`) pins the resting/disabled blank gutter as "keeps the
container ink, so the state could adopt" the shared `GutterCell::Inherit`
helper. That pin measured the pre-`d1ac0c0dac` paint — which the frozen
frames contradict. The test was already red at tip (verified by a stash-run
before this repair) and stayed red with identical bytes after it; the
pressed-focus adjudication's gate run did not cover this file.

The gate is designed for this: states move between "matches" and "diverges"
as the measured paint changes. The blank rows moved to the diverges side
with a bg-on-bg expectation (`resolver_bg`), the module matrix notes were
re-measured accordingly, and the obsolete DANGER-blank special case (recipe
stamping danger surface ink) is superseded by the frozen bg-on-bg rule that
now covers every variant.

## Additional finding: `showcase_visual_baseline` red at tip (pre-existing, unchanged)

`showcase-ui::visual showcase_visual_baseline` is red at tip with
`Overview 80 24 junie truecolor`: baseline `80780cbbac55284d`, got
`8b54edf362c1d6fc` — and the got-digest is **byte-identical** with this
repair applied (paired stash-run). Same `d1ac0c0dac` drift family (the
overview page renders buttons); recorded, not fixed here, and covered by the
same owner regeneration decision as the `render_components` rows.

## Required repairs (applied in this change)

1. `crates/termrock-controls/src/button.rs`: area fill restored to
   `container.style`; the bg-on-bg `pad` style retained for the
   `Part::GUTTER` blank fill; the after-marker and trailing special-case
   fills removed.
2. Failing-first proof (rendered cells through the real `Harness`, red at
   tip before the fix, green after):
   - `button_cells_carry_container_ink_and_bold_like_the_frozen_frame` —
     red at tip (trailing BOLD dropped), green after.
   - `pad_gap_cells_carry_container_ink_not_background` — red at tip
     (`pad/gap cell fg at x=2`: background vs patched container ink), green
     after. Together with the pre-existing
     `owner_wide_reference_styles_every_part_without_live_registration_or_cursor`
     red, this covers both regressions.
3. `crates/termrock-controls/tests/gutter_recipe_gate.rs`: blank matrix
   re-measured to the frozen bg-on-bg rule (see above).

## Gate results for this repair

- `cargo nextest run -p termrock-controls` (all targets): 187 run, 187
  passed — includes `button_contract` 21/21 (was 18/19 at tip),
  `gutter_recipe_gate` green.
- `cargo nextest run -p termrock-conformance --test control_states`:
  107 passed / 1 skipped.
- `render_components` button rows: red before and after with byte-identical
  got-digests — the defective reference recorded above; untouched.
- `showcase-ui --test visual showcase_visual_baseline`: red at tip and after
  with byte-identical got-digest (`8b54edf362c1d6fc`); pre-existing,
  recorded above.
- `cargo clippy --workspace --all-targets`: clean.
- `cargo fmt --check`: clean on touched files.
- Showcase suites (`showcase-ui`, `showcase-demos`, `showcase`,
  `showcase-data`): run after the repair; `d1ac0c0dac`'s showcase-side
  chrome changes are not touched by this repair.

## Owner decision owed

Regeneration scope approval for the `render_components` button rows (and the
button-bearing `showcase.txt` rows) at a pinned HEAD, after an independent
review records a verdict on this document. Until then the rows stay red with
the got-digests preserved in the tables above; nothing is blessed from
candidate output.
