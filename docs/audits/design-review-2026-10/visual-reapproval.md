# Visual re-approval against `visual-baseline` (tablepro, holla, jackin)

Status: 2026-10-17. tablepro, holla and jackin re-approved (gate).

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
| jackin, every hand-drawn list (manager tree, detail panes and roster, accounts tree, usage list, editor roles and accounts, settings trust and agents, inspect files, the config tables) | same as holla: `saved as payments-platform` at 3.45:1 (256: 2.3:1), the mount row's `rw worktree host` at 256 3.8:1, `4 roles` at 2.5:1 (256: 1.5:1), a health `!` at 3.6:1 | jackin draws these rows by hand too | `fix(jackin): details on a cursor row stay readable on its fill` (7403f8920): every row loop calls `legible_on_fill`; the helper now leaves the focus gutter alone (a graphic in the focus ink; stepping it up turned the one focus bar white on holla's and jackin's cursor rows at 256 colours) | jackin `details_on_a_cursor_row_stay_readable_on_its_fill`, `ui::ctx::a_row_fill_steps_its_text_up_and_keeps_the_focus_gutter` |
| jackin `accounts/add_form_required` | Save on the New account form with an empty Display name never marked the field; only the owner's form-level `Display name is required` said so (the baseline showed `Required` under the field) | since `ValidateOn::Change` (f7e8b7538) an edit that passes a field does not judge it, and `FormDialog` Save never validated | `fix(form): Save marks the first invalid field instead of saving past it` (2a71f8dcf): Save validates every visible enabled input, marks and focuses the first invalid one and does not save | `form_dialog::save_marks_the_first_invalid_field_and_does_not_save`, jackin `saving_a_new_account_without_a_name_marks_the_field_and_stays` |
| jackin `accounts/add_form` at 72x20 | Tab went from Provider straight to the buttons: Credential source and the 1Password reference could not be reached by keyboard | `FormDialog` drew, and so registered for focus, only the fields inside its scrolled body | `fix(form): fields scrolled out of a short form stay in the Tab order` (669dcd9cf): `RenderCtx::focus_stop` for each undrawn field | `form_dialog::fields_scrolled_out_of_the_body_stay_in_the_tab_order`, jackin `tab_reaches_the_fields_below_a_short_add_account_form` |
| jackin `manager/detail_drawer`, `accounts/drawer` (100x30 and wider) | Tab onto the workspace card or the account inspector left no `▎` anywhere on screen | both cards are focus stops themselves; since cards mark focus-within by the title alone (e95590552) only the title's weight changed | `fix(jackin): a detail card that holds focus shows the one focus bar` (5e4728c8b): `PanelFocus::Bar` while the card itself holds focus | jackin `a_detail_card_that_holds_focus_shows_the_one_focus_bar` |
| jackin Accounts overview (`accounts/filter`, `audit/accounts`, every size up to 120x40) | the Health summary was cut to one line (`… 2 stale · 1 refres…`); the baseline wrapped it and listed every issue | its toned parts (6348ef7ed) went through `render_spans`, which draws one line | `fix(jackin): the accounts Health line wraps instead of cutting its issues` (a570ca0f2): `widgets::viewport::wrap_spans` | `viewport::toned_spans_wrap_by_word_and_keep_their_tones`, jackin `the_accounts_health_line_wraps_instead_of_cutting_its_issues` |
| jackin `usage/overview`, `audit/accounts` | xAI's on-demand window (100% of the month's spend, reported available) drew the danger `!` and error run beside `✓ current` | `quota_tone` left healthy windows on `MeterTone::Normal`, which re-derives the level from 60/85 (a942f1081) while the domain already classifies the window | `fix(jackin): a quota meter follows its window status, not the percentage` (57562ede2): available windows are `MeterTone::Fixed(Low)` | jackin `a_meter_follows_its_window_status_not_the_percentage` |
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
| `jackin/accounts/add_form_required` | `enter`, `enter` on the empty Display name: the first commits the edit focus started, which no longer judges an untouched field (f7e8b7538), the second starts the edit again, so `Required` never appeared | `backtab` (wraps from the name to Save), `enter`: Save marks the field (fix above) |
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

## jackin

1,200 captures (48 roots x 5 sizes x 5 colours): **1,200 accepted, 0
pending.** Captured on the branch after S6 (pointer routing, e9d2406b7)
and the six fixes above; the narrow tier (40x12, 60x18) has no
baseline frame and stays pending. 1,190 captures differ from their
baseline frame (1,125 in text, 65 in style only); 10 are identical
(`intro/f300` and `scenarios/outro-last` in `none` and `nocolor`).
None is left unexplained.

Captures touched per attributed cause (a capture usually has several;
counted by text feature over all 1,200):

| Captures | Cause |
|---|---|
| 660 | footer vocabulary: `←→`/`↑↓` (e97f56b7a); pinned `Esc Back` / `? Help` and the status leaving them room, so a normal hint such as `Ctrl+S Save` or `Ctrl+Q Quit` drops first on a crowded row (f0cb83a02); one meaning per letter, `t Disable`, `d Remove…`, `! Open shell`, `x Cancel launch…` (c3515f652); `Enter <button>` (66e5268fc); `Ctrl+B Pane commands…`, `Enter Detail` (04647eb25); `Enter Edit fields` (66e5268fc); `EDIT Enter Apply` on a focused field (1da20cc01, 88fd384a5) |
| 460 | running state and status as colour roles: `● 2 running` in the identity strip, `✓ current` / `✗ provider unavailable` / `✗ Kimi · discovered` status glyphs, `▲ daemon stale` (6348ef7ed) |
| 370 | dotted scrollbar track `┆` / `┄` (ef2f81640) |
| 250 | modal close plate `─ × ─╮` (9f06503a2, e1325c04f) |
| 175 | meter level glyph `▲` / `!` beside medium and high meters, stale meters dashed `╍` (a942f1081, 0bd69d581, 83758d825) |
| 100 | horizontal bars under capsule panes, one content row fewer (b68c2210e, 5edf2bba8) |
| 100 | cockpit `Cancel launch…` button and the `Keep launching` / `Cancel launch` pair (31eaf284a, 934985f8d) |
| 50 | focused tab strip carries the `▎` on its cursor tab (`▎General`) (5d0c4467b, cafc01a74) |
| 65 | style only, brand screens (`intro/f300`, `scenarios/first-use`, `scenarios/outro-last`): muted `#808080` to `#8a8a8a` and faint to `#737373` (256: 244 to 245, 238 to 242; 16: DarkGray to Gray), the on-accent ink 233 to 232 (0610904df) |
| read by hand | popups and pickers truly centred instead of the upper third (6612b4443); app modals and dialogs sit in the body with the dialog margin, one row lower, never on the menu bar (e9a9c1941, d2d6896b9); help overlay: at most 80% of the screen, two columns, `1–29 of 30`, a `Done` button (755237d5d, 9f06503a2), `Ctrl+Q quit · asks only about…` (8a8c20253); no focus under a menu or modal (`▎ Current` under the File menu goes) (1d9055e8b); popup lists mark the cursor with a fill (948e926d8), the fixed-choice launch pickers included (no `▎` on screen while one is open; owner question below); wrapped values hang under the column gap (`readable` under `quota access`) (f2c4e04cb); usage limit labels take the width they measure (d09f244c9); the account drawer's actions wrap onto a second row instead of `Remov…` (72dff1756); the General tab scrolls instead of drawing on the Save row at 72x20 (a1091ba6e); disabled palette entries say why (`Not in this preview`) (00643499a); status copy `list may be incomplete` (7d3bb70e0); `Quit jackin❯?` (d6797b63c); the health and meter fixes above |

Layout consequences counted with their cause: a popup centred over the
page shows different page text beside it (the word check flags these:
`Tab details`, `saved as payments-platform`, pane text at the capsule
sizes), a horizontal bar takes a pane's last row so its tail scrolls by
one, header spacing follows the `●` dot.

Palette checks on the accepted frames: text is palette-independent; the
only `fg == bg` cells are the popup menu frames at 16 colours and `none`
(`capsule/menu`, `manager/menu_open`, 20 captures), which the baseline
has too (recorded under holla). Text that fell below 4.5:1 from 4.5:1
or more on an unchanged row is page text under a modal backdrop (the
backdrop's dim, as in holla), the tab rule under a backdrop (`━`, ghost
by design), and graphics at 3:1 or more (`◉`, `✓`, scrollbar thumbs, the
focus gutter on the row fill at 256 colours, 4.4:1).

Recorded, not fixed:

- 16 colours: on the selected tab's DarkGray plane (5d0c4467b) the
  capsule tab number `1` draws LightGray at 3.2:1 (`capsule/zoom`, every
  capsule root) and the editor's dirty `•` Yellow at 2.4:1
  (`editor/mounts_dirty`). The same family as holla's 16-colour tab
  glyphs above: the Tabs widget does not step its extra inks up on the
  selected plane. A library fix (Tabs runs its inks through
  `Theme::legible` on the selected fill) moves every app's 16-colour tab
  captures, so it is left for one pass over all four apps.

Owner confirmation needed (jackin):

- `usage/overview` (every size): the worst window became a meter
  (6348ef7ed) and its label keeps a fixed 10 cells, so `Weekly · Opus`
  reads `Weekly · …`, `On-demand usage` `On-demand…`, `Amp Free · daily`
  `Amp Free …` while the status column has room to spare at 120x40. Keep,
  or let the label measure its width through `ui::columns` (as the
  usage limits do since d09f244c9)?
- Footers that run out of room drop normal hints from the right while
  `Esc` and `?` stay (f0cb83a02): `editor/mounts_dirty` at 160x50 drops
  `Ctrl+S Save` beside `• 1 change`, the capsule drops `Ctrl+Q Quit`.
  Should `Ctrl+S` be a pinned key while a screen has unsaved changes?
- `accounts/add_form_required` now shows the Save outcome (the field
  marked, focus on it) rather than a validated Enter on the field; the
  frame matches the baseline's apart from `EDIT` in the footer (focus
  starts the edit, 1da20cc01). Confirm Save-time validation as the
  standard for every `FormDialog` (2a71f8dcf).
- `manager/launch_picker`, `hard_launch_picker` (and tablepro's Safe
  Mode picker, holla's Activities): a fixed-choice picker marks its
  cursor with a fill and no `▎`, so no focus bar shows anywhere while it
  is open; the baseline drew the bar on the cursor row. This is the
  recorded design of the focus audit (focus.md #1: "a non-searchable
  picker also uses the fill only, matching the menu"; menus show no bar
  either), and tablepro `the_switcher_is_the_only_layer_showing_focus`
  and showcase `the_chooser_stacks_a_picker_over_the_form_and_returns_to_it`
  lock it. It sits against the rule "exactly one keyboard-focus indicator
  is visible": confirm the menu exception covers fixed-choice pickers,
  or give them `CursorStyle::Bar` (a fix was tried in this pass and
  dropped for that decision).
- `quota_tone`: available windows are now always healthy (57562ede2),
  so the domain's `QuotaStatus` (from_pct thresholds, overridable per
  window) alone decides a quota meter's level. Confirm the domain is
  the authority over the library's 60/85.

How it was checked (scratch scripts, not in the tree): capture of all
48 roots at the canonical sizes (`VISUAL_FULL=1`, two partitions);
residual review with the transforms dotted track, close plate, running
dot, meter glyph, horizontal bar rows, rule lengths; every 120x40 frame
read by hand, the other sizes by residual and by the words gained or
lost beyond the 120x40 frame; `cargo xtask visual determinism` per half
(accounts, audit, capsule, cockpit, editor: 550 identical; intro,
manager, scenarios, settings, usage: 650 identical); the accepted files'
sha list equals the reviewed one; `cargo xtask visual accept` per half
(c12eccdfa). Re-captured after rebasing onto S7 part 1 (the library
Shell, 4400f40ab): no jackin frame moved.

## Overlay coverage (V4)

`overlay_sites` (plan T15) now maps 121 of 125 overlay sites to scenario
rows (was 41 of 108 listed; the scanner also found sites added since).
The new rows are `.pending()` with both dismissals pending; each was run
at 120x40 truecolor and its open frame read cell by cell before it was
listed. Journeys and surfaces whose frame is an open overlay moved to
`OVERLAYS` with their names and approvals unchanged
(`holla/flows/{alternatives,remote/gate2,files/jump}`,
`jackin/manager/new_workspace_prelude`, `jackin/capsule/new_tab`).

For layers whose `Dismiss` policy is `ESCAPE` (typed-token gates such as
`Dialog::facts` with a token, `FormDialog`, `ChoiceDialog`, the file
browser), the `-dismissed-outside` capture shows the layer still open:
that is the policy, not a missing dismissal (AGENTS.md, "Dismissing a
dialog resolves to its cancel action. Which dismissals are allowed is the
dialog's `Dismiss` policy").

Findings (2026-10-17, found while writing the rows):

| # | Finding | Owner |
|---|---|---|
| F1 | jackin capsule: Ctrl+B i (Container info) panics on `capsule-multi`: `screens/capsule.rs` `open_container_info` slices `run_id.replace('-', "")[..8]`, and that run id has 7 characters left ("end byte index 8 is out of bounds for string of length 7"). The row `jackin/overlays/capsule_info` reproduces it. | Fixed → A25 (dd13d9c2a) |
| F2 | jackin editor: the role-load error dialog (`finish_role_load`, layer `roleerr`) is unreachable. `start_role_load` fails only for an untrusted `data-eng`, but it runs only for a trusted role or right after the trust gate marks the role trusted. | Decision: reachable, kept → A26 (6098669b2) |
| F3 | jackin: three layers open only from a virtual-time job, which needs `--motion reduced`, and the reduced frame never settles (no quiet window within 15 s): the editor's and global settings' save failure (`finish_save`, hard-cases `save_fails_once`) and the cockpit's credential error (`step`, hard-cases with 1Password locked, launch Codex). Their sites stay listed gaps in `overlay_sites` until a determinism seam runs scheduled jobs under `--motion paused`. | Fixed → V12 (seam: `ClockStep`; rows below) |
| F4 | jackin: a job that fails leaves its progress status in the footer: after the save failure the footer still reads `Saving payments-platform…` / `Saving settings…`, after the role-load error `Loading role writer…`, until the 5 s info expiry. `finish_save` / `finish_role_load` open the error dialog but do not replace the status. Seen in the V12 rows (open and both dismissals). | Open → A27 |
| F5 | jackin cockpit: the credential-error dialog (`cockpit.rs::step`) sets `cancel_index` to the danger `Cancel launch` and keeps `Dialog::destructive`'s `Dismiss::LIGHT`, so a stray click outside it cancels the launch (`jackin/overlays/cockpit_credentials-dismissed-outside` shows the failed instance). A light dismissal should never run a destructive action. | Open → A28 |

F1 and F2 resolution (2026-10-17):

- **F1 (A25), fixed in dd13d9c2a.** Every fixed byte slice of a display
  id now goes through the new `ui::text::clip(s, max)` (leading cells, no
  ellipsis, whole graphemes, never panics on short text) or a prefix
  trim: capsule Container ID, cockpit run stamp, manager image tag, the
  jackin fixture run ids and holla's config digest. The remaining range
  slices in `src/` take their bounds from `find`, `char_indices` or a
  checked boundary. Test:
  `app_tests_chrome::container_info_opens_on_a_short_run_id_without_panicking`
  (fails on the parent with the panic). Container info now reads
  `3f9crun7f3ae21a`. `jackin/overlays/capsule_info` captures; it stays
  pending (not re-approved in this pass).
- **F2 (A26), decision: reachable, kept (6098669b2).** In the real
  product a trusted Role's load still fails (fetch, I/O, manifest
  parse), and the dialog's "trust stays granted · retry from the Roles
  tab" is written for that case, so the path stays. The fixture gained
  `RoleEntry::fetch_error` (what the simulated registry answers; the
  name check in `start_role_load` is gone); `hard-cases` gives the
  trusted local `writer` a manifest parse error. Enter on a trusted row
  with a load error now retries the load instead of asking for trust
  again, as the dialog says. Test:
  `app_tests_coverage::a_trusted_role_that_fails_to_load_reports_the_error_and_retries`
  (fails on the parent: the load succeeds). The load completes on a
  1,100 ms virtual-time job, so the site joins the F3 class: its
  `overlay_sites` gap now names F3 until V12's paused-motion seam
  (closed by V12: `jackin/overlays/editor_role_load_error`).

F3 resolution (V12, 2026-10-17): a determinism seam in shared code. Under
`--motion paused` the app's `SimClock` is stopped; an explicit clock step
runs it for N virtual milliseconds through the app's own tick and stops
it again, so a job on virtual time completes and the frame then holds
still (no switch to `--motion reduced`, whose frame never settles).

- `junie_tui::clock::ClockStep` is the in-band request (a bracketed
  paste whose whole text is `junie-tui:advance-clock:<ms>`), so the step
  lands after the keys sent before it; `clock::run_step` runs a stopped
  clock through an app's tick until the step has passed and restores
  `running`.
- `shell::App::advance_clock` (default `None`): `shell::handle` asks it
  only when the process opted in (`JUNIE_TUI_TEST_CLOCK=1`, read once,
  `Shell::clock_steps`) and under `Motion::Paused`; otherwise, or when
  the app has no virtual clock, the request is the ordinary paste it is,
  so real users can never step a clock. The harness sets the variable
  only for a scenario that sends an `advance:` step. jackin implements it
  with `run_step` over `App::on_tick`; the cockpit and the handoff now
  read the world clock's `running` instead of `motion == Paused` (the
  same value outside a step), so the launch pipeline runs during a step.
- Harness: the DSL step `advance:<ms>` (`support::Session::advance_clock`);
  `scenarios::validate` checks its argument.
- Rows (pending, both dismissals, each read at 120x40 truecolor):
  `jackin/overlays/editor_save_failed` (`editor.rs::finish_save`),
  `jackin/overlays/settings_save_failed` (`settings.rs::finish_save`),
  `jackin/overlays/editor_role_load_error` (`editor.rs::finish_role_load`,
  F2's trusted `writer`), `jackin/overlays/cockpit_credentials`
  (`cockpit.rs::step`, Codex · Primary with 1Password locked).
  `overlay_sites`: 125 of 125 sites have rows, 0 gaps.
- Tests: `clock::tests` (request parsing, a step on a stopped clock, a
  tick that does not advance ends the step),
  `shell::app::tests::a_clock_step_*` (a paused app's job fires and the
  clock stops again; not paused or no clock: an ordinary paste),
  `without_the_test_opt_in_a_clock_step_request_is_delivered_unchanged`
  (shell and jackin), and
  `jackin_preview::app_tests_clock` (the four layers open after a step
  through `Input::Paste`, and ticks then change nothing). The shell and
  jackin tests fail with the shell's interception disabled.
- No approved capture changes: outside an explicit step the code paths
  are the same (`running == (motion != Paused)`), and no approved row
  sends a step.

## Status (T19 sync, 2026-10-17)

The "Regressions found and fixed" and "Stale scenario steps fixed"
tables already carry a fix and a test for every row; the rows below add
the hashes those tables name only by subject, and an owner for each
item recorded as not fixed or waiting for the owner.

| # | Status | Commit(s) / owner | Notes |
|---|---|---|---|
| Regressions: backdrop, input, tab pane, holla rows, history opt-out | Fixed | 45521da32, f7e8b7538, e7c743cdd, 7bbf22b44, c75f2ce25 | Hashes for the fixes the regressions table names by subject (backdrop dims without erasing; untouched field not flagged; tab pane shows no focus under a layer; holla `legible_on_fill`; `HOLLA_NO_HISTORY` from the first frame). |
| holla: popup frames invisible at 16 colours and `none` | Open → A1 | — | `DarkGray` frame on the `DarkGray` popover plane; also in the baseline. |
| holla: running spinner and queued `○` on the selected tab at 16 colours | Open → A1 | — | 2.9:1 and 3.2:1; Tabs should run its inks through `Theme::legible` on the selected fill. |
| jackin: capsule tab number and editor dirty `•` on the selected tab at 16 colours | Open → A1 | — | 3.2:1 and 2.4:1; same library fix as the holla row. |
| holla: `HOLLA_NO_HISTORY=1` pinned in captures | Decision (product owner, 2026-10-17) | "Decision: holla captures keep `HOLLA_NO_HISTORY=1`" | Ranking keeps coverage through three `-with-history` rows. |
| jackin owner question 1: usage meter label width | Open → D1 | — | Owner decision pending. |
| jackin owner question 2: pin `Ctrl+S` while dirty | Open → D2 | — | Owner decision pending; a library `Footer` rule in S8b if answered (b). |
| jackin owner question 3: Save-time validation | Open → D3 | — | Owner decision pending; `ValidateWhen` stays an override (S11). |
| jackin owner question 4: fixed-choice pickers show no `▎` | Open → D4 | — | Owner decision pending. |
| jackin owner question 5: domain status decides quota meters | Open → D5 | — | Owner decision pending; `QuotaTone` moves to the library in S23-Q. |
