# Visual re-approval against `visual-baseline` (tablepro, holla, jackin)

Status: 2026-10-17. tablepro re-approved (gates). holla re-approved
(gates). jackin not started.

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

| holla, Files results and browser, Disk tree, Cleanup lists, plan steps | the details on the selected row (an accent-tinted fill, `#173f1c`; 256: 22) kept their plane inks: the parent path at 3.45:1 (256: 2.3:1), warning metas at 4.3:1; the baseline row was grey at 4.7:1 | holla draws these rows by hand and never stepped its inks up for the fill, as the library lists do with `Theme::row_ink` | `fix(holla): details on a cursor row stay readable on its fill`: `ui::ctx::legible_on_fill` runs every ink on a row fill through `Theme::legible` | holla `details_on_the_cursor_row_stay_readable_on_its_fill` |
| holla, every Here page with a Recent section (about 30 roots) | with `HOLLA_NO_HISTORY=1` (pinned by the harness) the first frame listed learned use (`Pull git here used 20 times on mbp`, `everything used here is suggested above`) where the baseline said `nothing used here yet` | `main` swapped in the disabled store after the Here page was built; the finder used to rebuild every tick and hid it, and since it rebuilds only on a view change the stale rows stay | `fix(holla): HOLLA_NO_HISTORY holds from the first frame`: `App::launch` applies the opt-out before the first page | holla `the_history_opt_out_holds_from_the_first_frame` |

Each fix moved approved showcase captures that had the defect in them;
those were re-approved with the fix, in snapshot-only commits that name
it (`showcase/flows/selectable/delete_confirm`, the Inspector overlay
digest in `tests/showcase_baseline.txt`).

## Stale scenario steps fixed (harness, app unchanged)

| Root | Was | Now |
|---|---|---|
| `holla/flows/cleanup/report`, `cleanup/linux_report`, `docker/drift`, `docker/done` | Tab, Enter after the gate-2 phrase and its Enter. A matching typed confirm submits on Enter (3ec122eb2), so the two keys acted on the next screen: the report roots captured Cleanup history (and timed out at 160x50), the Docker roots never reached their state | the stale Tab, Enter dropped; every combo matches its baseline frame apart from attributed changes |
| `holla/flows/brew/services_stop_failed` | `wait:Bootstrap failed`; at 72x20 the `! ` stderr gutter (299b941d2) wraps it across two rows, so the wait timed out | `wait:exit 1` (the status line), as `docker/remove_failed` |
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

After the re-approval, `cargo xtask visual check-touched` caught the
grid fixes that landed meanwhile (middle-cut reference values eee37267b,
cursor column kept in view 33fb0d4ce, one row of scroll context
ab64d7535): 247 tablepro captures re-approved with them by name, and the
tab-pane focus fix moved 80 dialog frames, re-approved with it.

Owner confirmation: none needed for tablepro.

## holla

3,125 captures (125 roots x 5 sizes x 5 colours): **3,125 accepted, 0
pending** (the pending rows and the narrow tier stay pending as for
every app). Classified on the branch after the two holla fixes and the
step fixes above, then captured once more on the HEAD after S6 (one
`cargo xtask visual determinism` run per group of six or seven roots:
3,125 of 3,125 byte-identical across the two runs) and accepted from
that run. No capture is left unexplained.

Re-review after the classification. S5, S6, the type-ahead and the
backdrop fixes landed between the classification capture and the final
one; the residual of every root at the five sizes was diffed against the
classified residual (footers separately) and the palette, `fg == bg` and
contrast checks were re-run on all 3,125:

- `flows/cleanup/artifacts` (72x20, 80x24, 100x30, 120x40) and
  `flows/upgrade/excluded` (72x20) moved: the list keeps one row of
  scroll context past the cursor, so after `End` the view sits at its
  end (thumb at the bottom) and after five `↓` the next plan step shows
  below the cursor. That is the library default of ab64d7535 (SVg9),
  which the classification capture predated.
- S6 (13cea0a2c holla pointer routing, e57fe76e8 light-dismiss frames,
  79b40b343 picker backdrop) changed the text of no stored holla capture:
  every stored root is keyboard-driven. The menu that now opens on press
  shows only in the pending `elements/menubar/file-pressed` row.
- Palette, `fg == bg` and contrast results are the classified ones (40
  popup-frame cells at 16 colours and `none`; four status bars cut at a
  palette label of another width).
- After the acceptance the branch took 7403f8920 (`legible_on_fill`
  leaves the focus gutter alone) and the tuiscotti 3382e57 pin
  (ccfac7676): 88 captures at 256 colours (19 roots) changed one cell,
  the `▎` on a cursor row's fill back to the focus ink (78 on 22) from
  the stepped-up white; re-approved with it by name. Nothing else moved.

Attributed differences (all sizes went through the same residual
review; counts are roots):

| Roots | Cause |
|---|---|
| about 60 | the Here tab's breadcrumb trail above the tab strip (35cffbafe) |
| 12 | stderr lines lead with a red `!` gutter and wrap one column later (299b941d2) |
| 8 | plan rows say `required` / `required · sudo` beside the faint marker and the summary drops `N required ·` (299b941d2) |
| 4 | `✓ trashed`, `2 conflicts` in the status bar, `▲` after a warning-level meter (299b941d2 and the meter levels of the colour roles) |
| 7 | the typed-confirm field's focus gutter sits inside the dialog margin; dialog bodies scroll with `↓ N more` and the frame meets the page border at small sizes (c394ed8a4, d2d6896b9, e9a9c1941) |
| 11 | popups and dialogs truly centred instead of the upper third (6612b4443): help, activities, quit, alternatives, menus, Go to path, upgrade confirm, batch picker |
| 9 | one focus indicator: the baseline drew a second `▎` on the Explore tabs or under menus (1d9055e8b) |
| every root with a modal or a long list | close plate `─ × ─╮` (9f06503a2, e1325c04f); dotted scrollbar tracks (ef2f81640); horizontal bars under wide viewports (d11ab0792, 6f075829a, b68c2210e) |
| several | copy: `Facts` to `Details`, `Actions` to `Alternatives` (9f2711d7a, 133434765), `Shift+F10 Menu` in footers (ee613b1a0) |
| 7 | suggestion order in `concept/first-use`, `concept/monorepo-root`, `concept/activities-multi` and the overlays opened over them (`flows/help/overlay`, `flows/quit/confirm`, `flows/activities/overlay`, `flows/activities/empty`). With the history opt-out applied before the first page (fix above) the ranking no longer reads the fixture's usage history; the baseline ranked with it (the leak was already there, only Recent hid it) |

Checks on the actuals: text is palette-independent (the status bar cuts
at a palette label of another width, as in the baseline); no `fg == bg`
text except the popup menu frame at 16 colours and `none`, which the
baseline has too; no text falls under 4.5:1 from 4.5:1 or more except
inactive content under a backdrop and two 16-colour tab glyphs.

Recorded, not fixed (pre-existing, or graphics just under 3:1):

- 16 colours and `none`: popup menu frames draw `DarkGray` on the
  `DarkGray` popover plane, so the frame is invisible (the plane still
  stands off the black canvas). Same in the baseline.
- 16 colours: a running tab's spinner `⠋` is LightGreen on the DarkGray
  selected tab, 2.9:1 (graphics need 3:1); a queued `○` is 3.2:1. The
  selected-tab fill comes from 5d0c4467b.

### Decision: holla captures keep `HOLLA_NO_HISTORY=1` (product owner, 2026-10-17)

The suggestion order of the seven roots above is intended: with the
opt-out pinned in `support::PINNED_ENV` the captures show what a new
user sees (no learned use; Recent says `nothing used here yet`). The
ranking feature keeps visual coverage through three rows that turn the
fixture's seeded usage back on for that row only (`Root::env`,
`HISTORY_ON` = `HOLLA_NO_HISTORY=0`): `concept/first-use-with-history`,
`concept/monorepo-root-with-history` and
`concept/activities-multi-with-history`. The history is the in-memory
fixture on the fixture clock; nothing is read from or written to disk,
and the per-scenario `HOME` stays empty. They rank as the
`visual-baseline` frames of the plain roots did, with `used 20 times on
mbp` in Recent; 75 captures, determinism 75 of 75, approved.

## jackin (not started)

1,200 stored captures, all still legacy. Before re-approving, check
jackin's hand-drawn rows (`screens/{accounts,manager,usage,editor,
settings,inspect,config}.rs` draw details on `Theme::row`) for the
cursor-row contrast defect fixed in holla; `ui::ctx::legible_on_fill`
is the fix.
