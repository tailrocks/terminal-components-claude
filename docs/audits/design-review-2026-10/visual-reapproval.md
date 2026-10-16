# Visual re-approval against `visual-baseline` (tablepro, holla, jackin)

Status: 2026-10-16. tablepro re-approved; holla and jackin below.

Until this pass every tablepro, holla and jackin capture was a legacy
tuisnap approval: skipped unless `VISUAL_FULL=1`, so nothing gated those
apps. The legacy `.ansi` files were last written on 2026-09-14 and are
byte-identical to the frozen `visual-baseline` tag (the tag itself was
only read: `git archive visual-baseline` into a scratch directory). This
file records, per app, how each capture differs from that frame and why,
so an approval never folds in a difference nobody can name.

## Method

Per app, on the branch HEAD:

1. Capture every stored legacy root at the five canonical sizes and five
   colours with `VISUAL_FULL=1` (pending rows and the narrow tier have no
   baseline frame and stay pending).
2. Compare each capture with its `visual-baseline` frame, row by row, after
   a list of named transforms that each stand for one attributed change
   (for example: a `│││` pane gap is the splitter seam of c4847ed22; a
   leading `✓` before `safe` is 67ddac8a8). The explorer and the tab pane
   are diffed as separate columns so a row inserted in one pane does not
   smear the other. What is left (the residual) was read by hand: every
   root at 120x40, then every root at 72x20, 80x24, 100x30 and 160x50.
3. Automated checks over all palettes:
   - text never depends on the palette (only the `truecolor · WxH` label);
   - no non-blank cell has its foreground equal to its background;
   - no text cell whose row text is unchanged drops below 3:1 where the
     baseline had at least 3:1;
   - per root and size, words gained or lost beyond those of the reviewed
     120x40 frame (catches content that appears or vanishes at one size).
4. `cargo xtask visual determinism` over the same selection (two captures,
   byte-identical) and a check that the accepted files are the reviewed
   ones.
5. `cargo xtask visual accept` of the explained set only, in a
   snapshot-only commit.

## Regressions found and fixed

| Found in | Symptom | Cause | Fix (library) | Test |
|---|---|---|---|---|
| tablepro, 16 colours, 50 captures (every picker and dialog over the tab strip, filter chips, connection detail buttons) | the selected tab and subtle buttons under a modal drew Black on Black | 16 colours draw them Black on the DarkGray raised plane; that plane is the `field_hover` token, so `Theme::backdrop` took canvas ink on it for a blank | `fix(theme): the modal backdrop dims page text without erasing it`: canvas ink is a blank only where it does not read on its plane | `theme::the_modal_backdrop_dims_text_without_erasing_it`, tablepro `page_text_under_a_modal_dims_but_never_disappears_at_16_colours` |
| tablepro truecolor, delete dialog over the connection detail | a button label under the backdrop drew `#262626` on `#27272a` (1.0:1) | ink that is not a listed text token dimmed to the ghost tier on any plane | same commit: ink that read on its plane (3:1) still reads on the backdrop plane (faint, muted, secondary); decoration stays dim | tablepro `page_text_under_a_modal_dims_but_never_disappears` |
| tablepro `connections/form_advanced` | Shift+Tab out of the untouched New connection Name put `Basic !` on the tab and marked Name required | focus starts the edit (1da20cc01) and Tab or blur commits it (e184db2b0); `TextInput::commit` always validated | `fix(input): an untouched field is not flagged when an edit passes it`: `TextInput::validate_on` (`ValidateOn::Change` default, `EveryCommit` keeps the old rule); submit still validates | `input::passing_through_an_untouched_required_field_marks_no_error`, tablepro `tabbing_past_the_untouched_name_flags_nothing_until_save` |
| tablepro, every dialog and (after 5d7422edb) every picker | the tab pane under the layer drew its focused title and border: a second focus cue under a modal | the pane read the raw focus ("anything but the explorer or the strip"); a layer that takes focus made it "focused" | `fix(tablepro): the tab pane under a modal or picker shows no focus`: `Interaction::shown_focus` (None under a layer) | tablepro `the_pane_under_a_picker_shows_no_focus` |

Each fix moved approved showcase captures that had the defect in them;
those were re-approved with the fix, in snapshot-only commits that name
it (`showcase/flows/selectable/delete_confirm`, the Inspector overlay
digest in `tests/showcase_baseline.txt`).

## Stale scenario steps fixed (harness, app unchanged)

| Root | Was | Now |
|---|---|---|
| `tablepro/table/filtered`, `tablepro/table/sorted-filtered` | Shift+Tab, Shift+Tab, Enter, Ctrl+L, `pending`, Enter. Since 1da20cc01 focus already selects the value, so Enter applied the prefilled `delivered` and Ctrl+L, outside the field, opened the Safety level picker; the capture showed `status = 'delivered'` and `▲ silent` | Shift+Tab, Shift+Tab, `pending`, Enter (the app test `sort_and_filter_on_table_tab` since 1da20cc01) |

## tablepro

1,100 captures (44 roots x 5 sizes x 5 colours): **1,100 accepted, 0
pending.** Every capture differed from its baseline frame; none was
unexplained after the fixes above.

Captures touched per attributed cause (a capture usually has several):

| Captures | Cause |
|---|---|
| 1,025 | colour as meaning (67ddac8a8): `✓ safe`, `✓ Connected to …`, `✓ 25 rows`, `✗ failed`, `▲ Execute write query?`, local connections without the `·` glyph, SSL and SSH as separate detail rows, environment and status tones |
| 785 | hint and copy vocabulary in the footer: `↑↓`/`←→` (e97f56b7a), `Enter Apply` (88fd384a5), `EDIT` on focus (1da20cc01), `Shift+F10 Menu` (08a846237, 80a5ebfc0), `e Edit…`/`d Delete…` (8049ca930), `Alt+D` (3d54fb7c6), no `y / n` and `Type "orders" to confirm` (3ec122eb2, f37b1da76), the focused button named (66e5268fc), pinned way out and help with `…` (f0cb83a02), `Tab Indent` (1da20cc01), `Data / Structure` (9f2711d7a) |
| 480 | dotted scrollbar track `┆` (ef2f81640) |
| 475 | collapsed explorer rail `› Explorer` below the explorer breakpoint (fc859d764) |
| 385 | explorer title collapse button `‹` (fc859d764) |
| 340 | splitter seams in the explorer, editor/results and history gaps (c4847ed22, called out there) |
| 330 | horizontal scrollbars under wide grids, one row fewer (f8203570d, 76ccdab11) |
| 200 | modal close plate `╭─ × ─╮` (9f06503a2, e1325c04f) |
| 80 | Safe Mode facts drop jargon: `type to confirm` (676936c8a) |
| 40 | EXPLAIN metrics keep a gap before the border (46c86a96f) |
| read by hand | query toolbar Run / Run all / Explain (1b4e14198); popups truly centred and completion text aligned (6612b4443); dialog bodies scroll inside the body with `↓ N more` (d2d6896b9); dialog padding (c394ed8a4); no focus bar under a modal (1d9055e8b) and filled popup cursors (948e926d8); the connection form scrolls by field instead of overlapping (8f5e07e39, the baseline's `Test connection na Cancel` overlap is gone); `+3` tab overflow (c9c4c8475); an empty panel meta leaves no stray rule (36ce96588); the hint bar no longer overprints the status (f0cb83a02, baseline `EntersEditme`); typing into `New connection` goes into Name (1da20cc01: the baseline frame of `form_new-filled` showed `Edit connection` for Local PostgreSQL because the typed letters ran as commands) |

Layout consequences counted with their cause, not separately: rule
lengths, `rows 1–26` instead of `1–27`, a grid scrolled one column
differently where the rail takes two columns (`table/cell_editing` at
72x20 shows `organization_id status` instead of `status total_amount
currency`), `cols 1–1 of 14` at 72x20 (the second column is drawn
clipped and is not in the viewport by definition).

Palette: text is identical across the five colours; styles follow the
colour-role commits (0610904df, a942f1081, 86f6aea01, 2fc9055d0,
67ddac8a8, 299b941d2, 6348ef7ed) and the backdrop fix. After the fixes
no cell has fg equal to bg, and the only cells that fall below 3:1 from
the baseline's 3:1 or more are the tab underline `━` under a backdrop
at truecolor (`#808080` to the ghost `#262626`): a rule, not text, and
decoration dims to ghost by design.

Baseline defect, not reproduced: in the baseline `connections/form_advanced`
frames at 100x30 and wider the Connections list pane is blank while the
form sits at the list's right; the baseline `form_new` frames at the
same sizes show the list. HEAD draws the list in both, so the form
pages now look like the baseline's own `form_new`.

Owner confirmation: none needed for tablepro.
