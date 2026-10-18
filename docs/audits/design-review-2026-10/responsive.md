# Responsive: every screen keeps its content reachable

## Status

Historical evidence. Its dated findings and source pins describe the recorded review only.

## Authority

Current authority: main defines the intended split-crate architecture; design/interaction-polish defines UI/UX, colors, and behavior.

## Current evidence

Use only findings tied to the revisions and methods recorded in this document; they do not establish current implementation state.

## Remaining work

Preserve concrete requirements and unresolved items in docs/remaining-work.md; superseded procedural directions are not current instructions.

## Verification

Verification is limited to the recorded commands, revisions, and results. Re-run against current sources before closure.


Owner priority (2026-10-17): every screen keeps all of its content
reachable at any terminal size. When content cannot be shown in full, a
scrollbar appears (vertical and horizontal) and the keyboard, the wheel
and a drag reach the rest; text cut to fit ends in `…` and reveals its
full value. Widgets resize; a layout that squeezes content away with no
bar, a dialog that clips its buttons, or a column that overflows the
frame breaks the rule. The "terminal too small" screen shows only below
the true minimum.

This file covers the three apps (tablepro, holla, jackin-preview): every
route, page, panel and overlay. The showcase pages are covered by the
template conversion (branch `responsive-lib`).

## Method: the reachability gate

`junie_tui::shell::reach` drives a real app on a test backend:

1. The case's state (a page, or an overlay opened over a page) is drawn
   at the reference size 300×100, where nothing needs to scroll, and its
   words are read (runs of letters and digits of 3+ cells, the footer row
   left out: its hints drop by priority and stay in the help overlay; the
   page under an overlay is the page's own case).
2. The same state is drawn at the size under test. Every scroll
   container of the frame (found from its scrollbars in the hit registry)
   is paged with the wheel to its end on each axis, both axes together
   for a container with two bars, and the bars that appear at a
   container's end are explored one level down.
3. A case may name **reveals**: the ways the app itself shows hidden
   content at small sizes (a drawer that follows focus, a preview toggled
   by an advertised key, a detail beside a list that follows the cursor).
   Each is applied at the small size and explored the same way.
4. A reference word never reached is **missing**. A word reached only as
   a prefix cut with `…` is shown with its reveal and counts as reached.
   A too-small frame reports nothing (it names the size it needs).

The size matrix is `reach::SIZES`: 40×12, 60×18, 72×20, 80×24, 100×25,
106×29, 120×40, 160×50. The apps' minimum is 72×20 (`TooSmall`), so 40×12
and 60×18 show the too-small screen today.

The gate (`app_tests_reach.rs` in each app, one test per case, run by
the default `cargo nextest run`, about 7 s wall per app in parallel)
compares each (case, size) with its line in `tests/reach_allow.txt`: the
missing words must match exactly. A new hidden word fails the gate, and
so does a fixed one whose line is still there (the failure prints the
lines to paste). `REACH_DUMP=<case>:<w>:<h> cargo nextest run --bin <app>
reach_dump --no-capture` prints a case's frame and its findings.

## Findings (2026-10-17, first run)

Every case is clean at 40×12 and 60×18 (the too-small screen, R-apps-13)
and most are clean at 160×50. The classes are by cause; the shared
component each fix belongs in is named, never a screen. File:line refer
to the code of this run. Kind: **a** real defect (content dropped or cut
with no bar and no reveal); **b** a reveal exists that the audit does not
drive (hover, `→`, an overflow menu); **c** audit limitation.

| Id | Kind | Cause | Where (examples) | Shared fix |
|---|---|---|---|---|
| R-apps-1 | a | **Fixed** (rapps-6-1): chrome strips dropped whole items by priority with no `…` and no reveal: the top strip / header segments and the status bar. Now the dropped items collapse into one `…` overflow item (`ui::reveal::overflow`, first of the right side) whose expansion tooltip shows them on hover; the audit counts them as reached through that reveal, like text cut with `…` (`ui::reveal::overflowed`). 46 allow lines gone, 58 shrank. Tests `dropped_segments_collapse_into_an_overflow_item_that_reveals_them`, `dropped_items_collapse_into_an_overflow_item_that_reveals_them`. Open: the `…` is pointer-only; strips are chrome outside the focus ring, so the keyboard path is A11 (focusable status bar peek line). Re-triaged (rapps-6b-2): the 68 allow lines still tagged R-apps-1 hold only words of other classes (the strips' dropped items are reached through the `…`); the tag is gone from them. | `widgets/segments.rs:104` (tablepro `app.rs:2137`, jackin `app.rs:1919`), `widgets/statusbar.rs:199-226` (jackin capsule, holla) | Dropped segments and status items collapse into one `…` item that reveals them (`ui::reveal`). |
| R-apps-2 | a | Hand-rolled master-detail drops the detail (or a third column) below a width instead of collapsing into the library's summary and drawer. **Partly fixed** (rapps-6b-2): holla plan no longer draws its consequence line over the narrow details drawer (test `the_plan_summary_stays_off_the_details_drawer`); its remaining 72×20 words are the PageHeader summary (R-apps-4). **Fixed 2026-10-17 (rapps-2b):** `DetailSize::MasterShare { pct, min, max }` (builder `master_share`) sizes the master by its share, clamped, and gives the detail the rest, so a hand-made sidebar keeps its geometry. Jackin editor and settings Environments: the variable card is a focus stop (`PanelFocus::Bar`) and the list sits on `MasterDetail` (`detail_share(0, 56, 56)`, collapse below 150): wide pixel-identical, Tab from the list now reaches the card (WCAG 2.4.3 / 3.2.3: the same order as the narrow drawer), narrow Tab opens the card as a drawer, Esc returns (tests `a_narrow_editor_opens_the_variable_card_as_a_drawer`, `a_wide_settings_card_is_a_tab_stop_after_the_list`); the 72x20/80x24 words left are card values cut with no reveal (R-apps-4). Tablepro connections on `MasterDetail` (`master_share(33, 26, 40)`, collapse below 80): wide geometry and Tab order unchanged, narrow Tab from the list focuses Connect and opens the card as a drawer, Esc returns; a focused card button survives narrowing (tests `a_narrow_connections_screen_opens_the_card_as_a_drawer`, `a_narrowed_card_keeps_its_focused_button_in_the_drawer`); conn-failed 72x20 keeps its one-line error message (R-apps-5). By design: connections-empty `Select moves` (the empty detail has no focus stop; the narrow master's empty state offers Ctrl+N). The connection form covering the narrow list (conn-form-* lines) is a form over the list, not a master-detail. Open: EXPLAIN node card (needs a card focus stop), jackin manager aside (a third pane), usage swap. | tablepro connections `connections.rs:1230` (<80), EXPLAIN node card `tabs.rs:1880` (<110); jackin manager aside `manager.rs:2488` (<150), editor/settings detail `config.rs:768` (<150), usage `usage.rs:939`; holla plan summary drawn over the drawer `plan.rs:993` | `ui::master_detail::MasterDetail` (drawer when narrow; an aside pane that stacks under the detail). |
| R-apps-3 | a | A metadata column is dropped (all rows or per row) without `…`, or a hand-rolled table zeroes columns. | `widgets/tree.rs:980,1073` (explorer counts "312"), `widgets/list.rs:309` (history "412 ms"); holla files `files.rs:762-800`, plan headings `plan.rs:1646`; jackin mounts `config.rs:2556` | Tree / List cut the meta with `…` and reveal it; tables move onto `widgets::table` (column priority, horizontal bar, cell reveal). |
| R-apps-4 | a | Text cut with `…` but no reveal registered. **Mostly fixed 2026-10-17** (rapps-458): one cut that keeps its reveal, `ui::reveal::cut(area, full)` (the expansion tooltip on hover, through `queue_cut`) and `ui::reveal::fit(ctx, area, full, focused)` (also the footer's peek line for the focused widget or cursor row, S10). TextInput's placeholder, help and error line, PageHeader's status and subtitle and `Heading` cut through them; the apps' 68 one-line `set_string(…, truncate(…), …)` cuts (holla, jackin, tablepro screens) moved onto `reveal::cut` by script, jackin's config rows onto `reveal::fit`, and two clipped holla subtitles (args, trust) are now cut with `…`. Tests `a_cut_help_line_reveals_on_hover_and_on_the_peek_line`, `a_cut_heading_and_subtitle_reveal_on_hover`; 43 allow lines gone, 13 shrank. Open: cuts not drawn as one `set_string` (holla activity tab strip "btm", plan summary "excluded included", config hash, cleanup toggles; jackin cockpit rail and credentials, editor-accounts "registry", manager aside rows, usage "identities"; tablepro connection-form list, history "Copy" (R-apps-7), open-quickly "Close") — each moves onto `reveal::cut`/`fit` or a library list. | `widgets/input.rs:877,944`, `widgets/picker.rs:708`, `widgets/page_header.rs:120-150`, `Heading` (`page_header.rs:197`); holla `review.rs:521`, `snapshot.rs:1497`, `disk.rs:1635`; jackin `manager.rs:1812`, `cockpit.rs:911` | One `ui::text` cut that registers its reveal, so no caller can cut without one. |
| R-apps-5 | a | **Fixed 2026-10-17** (rapps-458): line clamps dropped text without `…`. `Notice` reveals a message or detail cut at its line limit on hover from its last line (and its hint through `reveal::cut`); the shared wrap-clamp `ui::reveal::clamp(ctx, area, full, focused)` marks the cut ` …` and reveals it (hover; peek line when focused). Tablepro's error card no longer turns the mark off (`mark_cut(false)` gone), the safe-mode description clamps through `reveal::clamp` (peek while the Safe mode field is focused), jackin's blocker notices mark their cut. Tests `a_message_cut_at_its_line_limit_reveals_on_hover`, `a_clamp_marks_its_cut_and_reveals_the_text`; 11 allow lines gone, 6 shrank, the R-apps-5 tags left re-triaged (connection list under the form R-apps-2, the error card's dropped hint R-apps-6). | tablepro safe-mode description `connections.rs:1523` (`.take(2)`), `widgets/notice.rs:90` (`max_lines = 2`, the error card) | `Notice` and a shared wrap-clamp mark the cut and reveal or scroll it. |
| R-apps-6 | a | Fixed-size bodies drop rows with no scrollbar. **Partly fixed**: holla disk no longer drops its Filesystems below 22 rows (the tree under them scrolls; test `a_short_screen_keeps_the_disk_filesystems`). 2026-10-17 (rapps-6b-2): `props::render_in` draws a sheet through a vertical `ScrollView` at its natural height; holla gate, trust and snapshot facts, tablepro history detail card (query, facts, actions) and executed-statement card, and the jackin settings Agents tab now scroll instead of dropping rows. A wheel notch never moves further than its view (`ScrollPane::wheel`): a two-row view no longer skips a row per notch (this also cleared tablepro result-rows 72×20, R-apps-12). Tests `a_sheet_taller_than_its_area_scrolls_to_its_last_row`, `a_notch_never_moves_further_than_the_view`, `a_short_screen_scrolls_the_review_facts` (holla), `a_short_result_pane_scrolls_the_statement_facts` (tablepro). Open: jackin manager detail (its own row scroll over hand-drawn rows), cockpit credentials line (dropped under the rail when the centred block does not fit), holla config and cleanup-history bodies, tablepro error card | `widgets/props.rs` (`render_in`); jackin `manager.rs:1265`, `cockpit.rs:913`; tablepro error card `tabs.rs:1846` | The shared two-axis scroll container (branch `responsive-lib`) around page bodies; `PropsList` scrolls. |
| R-apps-7 | a | Button rows drop the buttons that do not fit. **Fixed 2026-10-18**: `button::row_layout` (and `row_layout_right`) never drop or cut a button the row has no room for; it wraps onto the next line, and `button::row_lines` says how many lines to reserve (one wrap rule, shared with `ButtonRow`'s `RowOverflow::Wrap`). Decision: wrap, not a `…` menu, for the free layout. A wrapping row keeps every button a direct Tab stop and click target with no extra layer, matching WAI-ARIA's toolbar pattern (every control stays in the toolbar's own focus order) and macOS dialogs, which grow rather than hide a button; the `…` fold stays `ButtonRow`'s default (`RowOverflow::More`), a macOS-toolbar-style overflow, for one-line rows that cannot grow. Callers reserve the lines: tablepro history card, holla activity and plan follow-ups, jackin manager card and editor/settings save rows, showcase Terminal step rail and Buttons toolbar. Tests `row_layout_wraps_the_buttons_that_do_not_fit_instead_of_dropping_them`, holla `a_narrowed_follow_up_row_wraps_and_keeps_the_focused_button`; tablepro history's "Copy" is reached (2 allow lines gone, 2 shrank); holla activity-failed's two lines keep only the header's project-kind cut, now filed under R-apps-4. Open: tablepro's editor toolbar is one row (`tabs.rs`, its commands have chords) and keeps only what fits; it should move onto `widgets::toolbar::Toolbar` and its `»` menu. The holla activity and plan follow-up rows ("Next" + buttons) are the same hand-rolled pattern in two screens and should become one library row. | `widgets/button.rs:262` (`row_layout`): tablepro history "Copy"; holla `activity.rs:924` | `ButtonRow` wraps or moves the rest into a `…` menu. |
| R-apps-8 | b | **Fixed 2026-10-17** (rapps-458): cut cells reveal on hover (or `→`) only; the audit drove neither, and Picker had no keyboard reveal (WCAG 2.1.1 gap). Now every reveal records its full text for the frame (`reveal::overflowed`: `truncated`, `RevealState::cell`, `peek`, overflow items) and the audit counts those words as reached through it, like text cut with `…`. Picker: the query field keeps `→` for its caret, so a cut cursor row shows in full on the footer's peek line (`label — detail`, S10); holla's finder rows queue their cut label and reason for hover and show the cursor row on the peek line (keyboard path; the preview, Tab, also shows it). Footer finding, **fixed 2026-10-17** (footer-peek): the peek line took the status slot ahead of the hints, so at narrow widths it dropped the primary key hint (jackin launch-picker 72×20 kept only `Esc Cancel`; holla's finder lost `Review…`, so its peek had been turned off). Now `shell::Footer` gives the peek only the width left after every hint the row draws without it (keys before a preview: a hint never drops because of the peek), cut with `…`, and leaves the peek out below 8 cells. A primary-hint-first variant that let the peek push out secondary hints (the finder's `Tab Preview`, `Alt+Enter Alternatives…`, `Ctrl+↑↓ Scope`) was rejected: discoverable keys win over a preview. The finder peeks again, only while it has focus (a popup over it no longer shows the row under it). Tests `the_peek_line_never_drops_the_primary_hint` (jackin, fails on the parent) and `the_finder_peek_never_drops_a_hint` (holla, 120x40). Tests `a_cut_cell_with_a_reveal_counts_as_reached`, `the_cut_cursor_row_shows_in_full_on_the_peek_line`; 12 allow lines gone (finder, launch-picker, safe-mode), 9 shrank, the R-apps-8 tags left re-triaged to R-apps-4 (text cut with no reveal at all). Captures: picker footers gain the peek line (attributed by script). | `widgets/picker.rs:859`; Tree, Props and List rows (holla finder, jackin manager, launch picker) | Picker registers `RevealState::cell` for `→`; the audit could drive the reveal key. |
| R-apps-9 | a | **Fixed.** A dialog showed a scrollbar but a wheel over its fields was ignored, and FormDialog scrolled its focused field back into view every frame, undoing the wheel. | `widgets/form_dialog.rs`, `widgets/info_dialog.rs` (jackin account, env and mount forms, cockpit failure); the help overlay's padding beside its bar | `ScrollPane::draw_with_fade` registers the body under its children (`register_scroll_under`); FormDialog follows focus only when it moves. |
| R-apps-10 | a | **Fixed.** Dialog clipped: the anchored code preview (6 lines) left the facts 0 rows. | `widgets/dialog.rs` (holla start-plan 72×20) | The preview shrinks first (`CODE_LINES`, `MIN_FACTS`): the facts keep 2 rows to scroll. |
| R-apps-11 | b | Tabs past the strip sit behind the `+N` overflow menu. | `widgets/tabs.rs:119` (holla activity) | — (reachable; the audit does not open the menu). |
| R-apps-12 | c | Audit limits: a 3-row grid at 72×20 the probe could not sweep (fixed: a wheel notch moves at most one view, R-apps-6); a path wrapped mid-word; a page under a modal that the open step changes. | tablepro result-rows, holla finder-children, jackin cockpit-failure | — |
| R-apps-13 | a | The too-small screen shows below 72×20 for every app, also where a scroll container could hold the layout. | `widgets/too_small.rs` (72×20) | Lower each app's minimum to its true one once its bodies scroll. |
| R-apps-14 | a | A Dialog's code preview shows at most 6 lines and summarises the rest as "… N more", reachable at no size; at 72×20 the shrunk preview hides more. **Fixed 2026-10-18**: the preview joins the body scroll (facts, a blank row, every code line in one sheet; its horizontal bar and the typed acknowledgement stay anchored above the actions), so no line is summarised and `CODE_LINES`/`MIN_FACTS` are gone. Test `every_line_of_a_long_code_preview_is_reached_by_scrolling_the_body`; holla start-plan allow line gone. | `widgets/dialog.rs` (holla start-plan) | The preview scrolls vertically, or joins the body scroll. |

Also seen, not missing words (the help overlay's dead padding column
beside its bar is fixed with R-apps-9): tablepro conn-failed at 120×40 draws the
error notice over the button row (`connections.rs:1401`); the DDL card's
position label is computed before render (`tabs.rs:854`).

### Screen × size

Cells name the classes found at that size (blank: clean; cases not listed
are clean at every size). 40×12 and 60×18
are the too-small screen everywhere (R-apps-13). `tests/reach_allow.txt`
holds the exact words.

| Screen | 72x20 | 80x24 | 100x25 | 106x29 | 120x40 | 160x50 |
|---|---|---|---|---|---|---|
| holla activity | 4 11 | 4 11 | 4 11 |  |  |  |
| holla activity-failed | 7 | 7 |  |  |  |  |
| holla activity-merged | 4 11 | 4 11 | 4 11 |  |  |  |
| holla args | 4 |  |  |  |  |  |
| holla cleanup | 4 | 4 |  | 4 | 4 |  |
| holla cleanup-history | 4 6 | 4 6 | 4 6 | 4 6 | 4 6 |  |
| holla config | 4 6 | 4 6 | 4 6 |  |  |  |
| holla disk-overview | 4 | 4 | 4 |  |  |  |
| holla files | 3 | 3 | 3 | 3 | 3 | 3 |
| holla finder | 8 |  |  |  | 12 |  |
| holla finder-children | 8 12 | 8 12 |  |  | 8 12 |  |
| holla finder-page | 8 | 8 |  |  |  |  |
| holla gate-plan | 4 | 4 | 4 | 4 | 4 |  |
| holla plan-review | 4 |  |  |  |  |  |
| holla plan-running | 3 | 3 | 3 | 3 |  |  |
| holla start-plan | 14 |  |  |  |  |  |
| holla top-files | 4 | 4 | 4 | 4 |  |  |
| holla trust | 4 8 | 4 8 |  |  |  |  |
| holla upgrade-plan | 4 |  |  |  |  |  |
| jackin capsule | 6 | 6 | 6 | 6 | 6 | 6 |
| jackin capsule-zoom | 6 | 6 | 6 | 6 |  |  |
| jackin cockpit | 4 6 | 4 6 | 4 6 | 4 6 | 4 6 | 4 6 |
| jackin cockpit-failure | 12 | 12 | 12 |  |  |  |
| jackin editor-accounts | 4 | 4 |  |  |  |  |
| jackin editor-env | 4 | 4 |  |  |  |  |
| jackin editor-mounts | 3 | 3 | 3 |  |  |  |
| jackin editor-roles | 4 | 4 |  |  |  |  |
| jackin launch-picker | 8 | 8 |  |  |  |  |
| jackin manager | 2 8 | 2 8 | 2 8 | 2 8 | 2 8 | 2 8 |
| jackin manager-detail | 2 8 | 2 8 | 2 8 | 2 8 | 2 8 | 2 8 |
| jackin manager-expanded | 2 8 | 2 8 | 2 8 | 2 8 | 2 8 | 2 8 |
| jackin manager-hard | 2 6 8 | 2 6 8 | 2 6 8 | 2 6 8 | 2 6 8 | 2 6 8 |
| jackin settings | 4 | 4 |  |  |  |  |
| jackin settings-agents | 4 |  |  |  |  |  |
| jackin settings-env | 4 | 4 |  |  |  |  |
| jackin settings-mounts | 3 | 3 | 3 | 3 |  |  |
| jackin usage | 4 | 4 | 4 | 4 | 4 | 4 |
| tablepro conn-failed | 5 |  |  |  |  |  |
| tablepro conn-form-advanced | 2 4 | 2 4 | 2 4 | 2 4 |  |  |
| tablepro conn-form-edit | 2 4 5 | 2 4 5 | 2 4 5 | 2 4 5 | 2 4 5 |  |
| tablepro conn-form-errors | 2 4 5 | 2 4 5 | 2 4 5 | 2 4 5 | 2 4 5 |  |
| tablepro conn-form-new | 2 4 5 | 2 4 5 | 2 4 5 | 2 4 5 | 2 4 5 |  |
| tablepro connections-empty | 2 | 2 |  |  |  |  |
| tablepro history | 4 7 | 7 |  | 3 7 | 3 7 |  |
| tablepro query-editing |  |  |  | 3 | 3 |  |
| tablepro result-affected |  |  |  | 3 | 3 |  |
| tablepro result-error | 5 6 | 5 6 |  | 3 5 6 | 3 5 6 |  |
| tablepro result-multi |  |  |  | 3 | 3 |  |
| tablepro result-plan | 2 | 2 | 2 | 2 3 | 2 3 |  |
| tablepro safe-mode | 8 | 8 |  |  |  |  |
| tablepro table-ddl |  |  |  | 3 | 3 |  |
| tablepro table-structure |  |  |  | 3 | 3 |  |
| tablepro workbench |  |  |  | 3 | 3 |  |
