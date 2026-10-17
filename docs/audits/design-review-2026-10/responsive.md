# Responsive: every screen keeps its content reachable

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
| R-apps-1 | a | Chrome strips drop whole items by priority with no `…` and no reveal: the top strip / header segments and the status bar. | `widgets/segments.rs:104` (tablepro `app.rs:2137`, jackin `app.rs:1919`), `widgets/statusbar.rs:199-226` (jackin capsule, holla) | Dropped segments and status items collapse into one `…` item that reveals them (`ui::reveal`). |
| R-apps-2 | a | Hand-rolled master-detail drops the detail (or a third column) below a width instead of collapsing into the library's summary and drawer. | tablepro connections `connections.rs:1230` (<80), EXPLAIN node card `tabs.rs:1880` (<110); jackin manager aside `manager.rs:2488` (<150), editor/settings detail `config.rs:768` (<150), usage `usage.rs:939`; holla plan summary drawn over the drawer `plan.rs:993` | `ui::master_detail::MasterDetail` (drawer when narrow; an aside pane that stacks under the detail). |
| R-apps-3 | a | A metadata column is dropped (all rows or per row) without `…`, or a hand-rolled table zeroes columns. | `widgets/tree.rs:980,1073` (explorer counts "312"), `widgets/list.rs:309` (history "412 ms"); holla files `files.rs:762-800`, plan headings `plan.rs:1646`; jackin mounts `config.rs:2556` | Tree / List cut the meta with `…` and reveal it; tables move onto `widgets::table` (column priority, horizontal bar, cell reveal). |
| R-apps-4 | a | Text cut with `…` but no reveal registered. | `widgets/input.rs:877,944`, `widgets/picker.rs:708`, `widgets/page_header.rs:120-150`, `Heading` (`page_header.rs:197`); holla `review.rs:521`, `snapshot.rs:1497`, `disk.rs:1635`; jackin `manager.rs:1812`, `cockpit.rs:911` | One `ui::text` cut that registers its reveal, so no caller can cut without one. |
| R-apps-5 | a | Line clamps drop text without `…`. | tablepro safe-mode description `connections.rs:1523` (`.take(2)`), `widgets/notice.rs:90` (`max_lines = 2`, the error card) | `Notice` and a shared wrap-clamp mark the cut and reveal or scroll it. |
| R-apps-6 | a | Fixed-size bodies drop rows with no scrollbar. | `widgets/props.rs:182,199` (`props::render`), tablepro `tabs.rs:2706,1820,1846`; holla fact sheets `review.rs:488`, disk compact mode `disk.rs:1546`; jackin detail `manager.rs:1265`, the settings Agents footnote, cockpit credentials | The shared two-axis scroll container (branch `responsive-lib`) around page bodies; `PropsList` scrolls. |
| R-apps-7 | a | Button rows drop the buttons that do not fit. | `widgets/button.rs:262` (`row_layout`): tablepro history "Copy"; holla `activity.rs:924` | `ButtonRow` wraps or moves the rest into a `…` menu. |
| R-apps-8 | b | Cut cells reveal on hover (or `→`) only; the audit drives neither. Picker has no keyboard reveal (WCAG gap). | `widgets/picker.rs:859`; Tree, Props and List rows (holla finder, jackin manager, launch picker) | Picker registers `RevealState::cell` for `→`; the audit could drive the reveal key. |
| R-apps-9 | a | A dialog shows a scrollbar but a wheel over its fields is ignored (`ev.wheel() && !ev.on(self.id)`). | `widgets/form_dialog.rs:1056`, `widgets/info_dialog.rs:497` (jackin account, env and mount forms, cockpit failure) | The wheel reaches the dialog from any child region (`scroll_stack`). |
| R-apps-10 | a | Dialog clipped: the anchored code preview leaves the facts 0 rows. | `widgets/dialog.rs:1325-1329` (holla start-plan 72×20) | Shrink the code preview first; keep a minimum facts viewport. |
| R-apps-11 | b | Tabs past the strip sit behind the `+N` overflow menu. | `widgets/tabs.rs:119` (holla activity) | — (reachable; the audit does not open the menu). |
| R-apps-12 | c | Audit limits: a 3-row grid at 72×20 the probe cannot sweep in budget; a path wrapped mid-word; a page under a modal that the open step changes. | tablepro result-rows, holla finder-children, jackin cockpit-failure | — |
| R-apps-13 | a | The too-small screen shows below 72×20 for every app, also where a scroll container could hold the layout. | `widgets/too_small.rs` (72×20) | Lower each app's minimum to its true one once its bodies scroll. |

Also seen, not missing words: the wheel is dead in the padding column
beside an overlay's scrollbar (help overlay: `ctx.scrollable` covers only
`inner`, `widgets/help.rs:341`); tablepro conn-failed at 120×40 draws the
error notice over the button row (`connections.rs:1401`); the DDL card's
position label is computed before render (`tabs.rs:854`).

### Screen × size

Cells name the classes found at that size (blank: clean). 40×12 and 60×18
are the too-small screen everywhere (R-apps-13). `tests/reach_allow.txt`
holds the exact words.

| Screen | 72x20 | 80x24 | 100x25 | 106x29 | 120x40 | 160x50 |
|---|---|---|---|---|---|---|
| holla activity | 4 11 | 4 11 | 4 11 |  |  |  |
| holla activity-failed | 1 7 | 1 7 |  |  |  |  |
| holla activity-merged | 4 11 | 4 11 | 4 11 |  |  |  |
| holla args | 4 |  |  |  |  |  |
| holla cleanup | 4 | 4 |  | 4 | 4 |  |
| holla cleanup-history | 4 6 | 4 6 | 4 6 | 4 6 | 4 6 |  |
| holla config | 4 6 | 4 6 | 4 6 |  |  |  |
| holla disk | 6 | 6 | 6 |  |  |  |
| holla disk-overview | 4 | 4 | 4 |  |  |  |
| holla files | 3 | 3 | 3 | 3 | 3 | 3 |
| holla finder | 8 |  |  |  | 12 |  |
| holla finder-children | 1 8 12 | 8 12 |  |  | 8 12 |  |
| holla finder-page | 8 | 8 |  |  |  |  |
| holla gate-plan | 4 6 | 4 6 | 4 6 | 4 6 | 4 6 |  |
| holla plan-review | 1 2 4 |  | 2 4 | 2 4 |  |  |
| holla plan-running | 3 | 3 | 3 | 3 |  |  |
| holla snapshot | 1 |  |  |  |  |  |
| holla start-plan | 1 10 |  |  |  |  |  |
| holla top-files | 1 4 | 1 4 | 4 | 4 |  |  |
| holla trust | 1 6 | 1 6 |  |  |  |  |
| holla upgrade-plan | 2 4 |  |  | 2 4 |  |  |
| jackin account-form | 9 | 9 | 9 | 9 |  |  |
| jackin accounts | 1 8 | 1 | 1 |  |  |  |
| jackin capsule | 1 6 | 1 6 | 1 6 | 1 6 | 1 6 | 1 6 |
| jackin capsule-zoom | 1 6 | 1 6 | 1 6 | 1 6 | 1 | 1 |
| jackin cockpit | 1 4 6 | 1 4 6 | 1 4 6 | 1 4 6 | 4 6 | 4 6 |
| jackin cockpit-failure | 9 12 | 9 12 | 9 12 |  |  |  |
| jackin editor | 1 | 1 | 1 | 1 |  |  |
| jackin editor-accounts | 1 4 | 1 4 | 1 | 1 |  |  |
| jackin editor-env | 1 2 | 1 2 | 1 2 | 1 2 | 2 |  |
| jackin editor-mounts | 1 3 | 1 3 | 1 3 | 1 |  |  |
| jackin editor-roles | 1 4 | 1 4 | 1 | 1 |  |  |
| jackin env-form | 9 |  |  |  |  |  |
| jackin launch-picker | 8 | 8 |  |  |  |  |
| jackin manager | 1 2 8 | 1 2 8 | 1 2 8 | 1 2 8 | 1 2 8 | 1 2 8 |
| jackin manager-detail | 1 2 8 | 1 2 8 | 1 2 8 | 1 2 8 | 1 2 8 | 1 2 8 |
| jackin manager-expanded | 1 2 8 | 1 2 8 | 1 2 8 | 1 2 8 | 1 2 8 | 1 2 8 |
| jackin manager-hard | 1 2 6 8 | 1 2 6 8 | 1 2 6 8 | 1 2 6 8 | 1 2 6 8 | 2 6 8 |
| jackin mount-form | 9 |  |  |  |  |  |
| jackin prelude | 1 | 1 |  |  |  |  |
| jackin prelude-source | 1 | 1 |  |  |  |  |
| jackin settings | 1 4 | 1 4 |  |  |  |  |
| jackin settings-agents | 1 6 | 1 6 |  |  |  |  |
| jackin settings-env | 1 2 | 1 2 | 1 2 | 2 | 2 |  |
| jackin settings-mounts | 1 3 | 1 3 | 3 | 3 |  |  |
| jackin settings-trust | 1 | 1 |  |  |  |  |
| jackin usage | 1 4 | 1 4 | 4 | 4 | 4 | 4 |
| jackin usage-detail | 1 | 1 |  |  |  |  |
| tablepro conn-failed | 2 | 2 |  |  |  |  |
| tablepro conn-form-advanced | 2 4 | 2 4 | 2 4 | 2 4 |  |  |
| tablepro conn-form-edit | 2 4 5 | 2 4 5 | 2 4 5 | 2 4 5 | 2 4 5 |  |
| tablepro conn-form-errors | 2 4 5 | 2 4 5 | 2 4 5 | 2 4 5 | 2 4 5 |  |
| tablepro conn-form-new | 2 4 5 | 2 4 5 | 2 4 5 | 2 4 5 | 2 4 5 |  |
| tablepro connections | 2 | 2 |  |  |  |  |
| tablepro connections-empty | 2 | 2 |  |  |  |  |
| tablepro explorer-hidden | 1 | 1 |  |  |  |  |
| tablepro history | 1 3 4 6 7 | 1 4 6 7 |  | 3 4 6 7 | 3 4 6 7 |  |
| tablepro maximized | 1 | 1 |  |  |  |  |
| tablepro query-editing | 1 | 1 |  | 3 | 3 |  |
| tablepro result-affected | 1 6 | 1 |  | 3 | 3 |  |
| tablepro result-error | 1 5 6 | 1 5 6 |  | 3 5 6 | 3 5 6 |  |
| tablepro result-multi | 1 | 1 |  | 3 | 3 |  |
| tablepro result-plan | 1 2 | 1 2 | 2 | 2 3 | 2 3 |  |
| tablepro result-rows | 1 12 | 1 |  |  |  |  |
| tablepro safe-mode | 8 | 8 |  |  |  |  |
| tablepro table-data | 1 | 1 |  |  |  |  |
| tablepro table-ddl | 1 | 1 |  | 3 | 3 |  |
| tablepro table-filtered | 1 | 1 |  |  |  |  |
| tablepro table-pending | 1 | 1 |  | 1 |  |  |
| tablepro table-structure | 1 | 1 |  | 3 | 3 |  |
| tablepro workbench | 1 | 1 |  | 3 | 3 |  |
