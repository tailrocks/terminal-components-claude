# Showcase coverage: app patterns

Status: inventory, 2026-10-16, a read-only pass over `design/interaction-polish` at `8f12ad218`. It lists the UI patterns tablepro, holla and jackin-preview use, whether the library owns them, and whether the showcase demonstrates them. Every ⚠️ and ❌ row is mapped to a step of the showcase coverage track (SC) in [`shared-components-plan.md` §7](../../architecture/shared-components-plan.md#7-showcase-coverage-track-sc). Line numbers drift as fixes land. Paths under `/private/tmp` refer to the auditor's scratch files, which are not kept.

Scope: a read-only pass over `origin/design/interaction-polish` (exported copy at `/private/tmp/claude-501/-Users-donbeave-Projects-terminal-components-claude/42325d29-0331-4e82-a42c-409070ca54fc/scratchpad/inv-apps/src-copy`). Nothing was edited, committed or pushed, and the `visual-baseline` tag was not touched. The inventory comes from reading code only: I rendered no screens and ran no tests.

Path prefixes: **T** = `src/bin/tablepro/`, **H** = `src/bin/holla/`, **J** = `src/bin/jackin_preview/`, **S** = `src/bin/showcase/pages/`. Coverage: ✅ the showcase shows the pattern with the variants the apps use; ⚠️ the widget is shown but app-used variants or options are missing; ❌ not shown at all.

**Library features no showcase page uses at all:**
- **Whole modules and components:** `rail::SidebarRail`, `breadcrumb::Breadcrumb`, `help::HelpOverlay`, `info_dialog::InfoDialog`, `choice_dialog::ChoiceDialog`, `form_dialog::FormDialog`, `keyref::KeyReference`, `ui::layout::Maximized`/`Split::toggle_max`, `ui::reveal::Reveal`.
- **Tabs options:** `TabItem` (prefix, suffix, closable, dirty, busy, error), `Tabs::allow_new`, `overflow_menu`, `TabRule`.
- **Status bar options:** `StatusItem::busy`, `StatusItem::meter`.
- **Picker:** `PickerStatus`.
- **Button rows:** `ButtonRow`/`RowOverflow::Wrap`, `row_layout_right`.
- **Small helpers:** `EmptyAction`/`empty::render_ctx`, `chips::overflow_run`, `ui::columns::solve`, `ui::fade::scroll_edges`, `scrollbar::render_vertical`, `viewport::Mark`/`CellPos`, `Dialog::anchor`/`placement`.

## Ranked inventory

| # | pattern | apps using it (file:line) | library component? | showcase coverage | action |
|---|---|---|---|---|---|
| 1 | Collapsible sidebar → 2-col vertical rail (`›` + vertical label), `‹` collapse button in the panel title, Ctrl+B, `0` re-show, hidden while maximised | T workbench.rs:81,1402-1408,1442; app.rs:908,1064-1077 | `rail::SidebarRail` | ❌ (S sidebars.rs:60 has a hand-made collapsed mode instead) | add demo |
| 2 | Tab strip with icons (`T` table / `≡` query / `H` history), `×` close, `+` new tab, dirty/busy/error flags, duplicate labels qualified with the schema | T workbench.rs:388-444,139; H app.rs:3380-3426; J editor.rs:2195, capsule.rs:1501 | `tabs::{Tabs, TabItem}` | ⚠️ S settings.rs:116 plain `Tabs::new` only | extend demo |
| 3 | Tab overflow `‹ ›` arrows, `+N`/`3› +3` overflow menu, wheel one tab per notch, middle-click closes, Alt/digit jump | T workbench.rs:1297-1305, tabs.rs:857-866; H app.rs:1679-1690,1881 | `Tabs` (`overflow_menu`, `on_wheel`, `middle_click_closes`, `digit_jump`) | ❌ | add demo |
| 4 | Nested tab strips (main tabs › Data/Structure segmented strip › Columns/Indexes/FKs/Constraints/Triggers/DDL sub-tabs, 1-6 jump), Ctrl+D mode toggle | T tabs.rs:420-433,781-791,857-866 | `Tabs` (app composes the nesting) | ❌ | add Screens demo |
| 5 | Preview tab (a single click opens a tab that the next single click replaces; promoted on Enter or a second click; meta "preview") | T workbench.rs:446-484,1085-1098 | ❌ app-local | ❌ | extract (`Tabs` preview flag) then demo |
| 6 | Tab list picker (Ctrl+G): glyph, detail, active/unsaved tags, Delete closes a tab and reopens the list | T app.rs:1579-1639; H activities switcher app.rs:1131-1202 (live refresh, `N live` scope) | `picker::Picker` | ⚠️ S pickers.rs:114 (no Delete/secondary action, no live refresh) | extend demo |
| 7 | Panel title with right-aligned meta ("preview · 9 cols", "N entries"), `meta_short`, badge, focus ring | T workbench.rs:1475-1502; H cards everywhere; J usage.rs:918 | `panel::Panel::meta/meta_short/badge/focus_mark` | ⚠️ meta only (S taskrunner.rs:452, terminal.rs:219); no `meta_short`, `PanelFocus` or badge | extend Panels demo |
| 8 | Pane zoom/maximise (`z`, Esc restores, chrome hides) | T app.rs:952-972, tabs.rs:1663; H plan.rs:1295-1350 (Enter/z on step output); J capsule prefix `z` | `ui::layout::Maximized`/`Split::toggle_max` (T); H and J app-local | ❌ (S chrome.rs:168 only draws a "ZOOM" hint badge) | extract (H/J onto `Split::toggle_max`) then demo |
| 9 | Splitter: drag, double-click reset, Ctrl+↑/↓ nudge, default width clamped until dragged, 3-column at ≥150 cols | T workbench.rs:1409-1431, tabs.rs:1008; J manager.rs:2616-2672, inspect.rs; H history | `ui::layout::Split` + `splitter::Splitter` | ⚠️ S terminal.rs:104 (no reset, nudge, 3-pane or vertical split) | extend demo |
| 10 | Master-detail with responsive drawer (<100 cols the detail becomes a full-body drawer; hidden pane kept as a zero-size focus stop; narrow summary card or one-line summary) | H finder.rs:1503-1572, files.rs:1719, disk.rs:1716, cleanup.rs:1290, plan.rs:883; J manager.rs:2621-2650,2791, accounts.rs:2695, usage.rs:462; T workbench.rs:1395-1468, connections.rs:1227 | ❌ app-local in about 10 places | ❌ | extract `MasterDetail` then demo |
| 11 | Empty state with action buttons | T connections.rs:1259-1270 | `empty::{EmptyState, EmptyAction, render_ctx}` | ⚠️ S chips.rs, editor.rs, selectable.rs (no actions) | extend demo |
| 12 | Typed-confirmation facts dialog (Action/Target/Scope/Risk/Reversible facts, SQL or command code, typed token, danger button, initial focus on Cancel) | T app.rs:1116-1387 (Safe Mode gate, save review); H app.rs:2521-2602 (gate 2 of 2); J manager.rs:638, capsule.rs:2457 | `Dialog::facts(.., Some(token), ..)` | ⚠️ S grid.rs:233 facts without a token; S dialogs.rs has none | extend Dialogs demo |
| 13 | Read-only facts or code dialog (SQL preview, JSON cell viewer, Close only) | T app.rs:1431-1459, 679-698 | `Dialog::facts` (app trims the actions) | ⚠️ S grid.rs:233 | extend demo (+ a `Dialog::viewer` preset) |
| 14 | Info dialog: copyable rows (`y`), error variant, "read-only" meta | J manager.rs:563, capsule.rs:407,1266, editor.rs:465 | `info_dialog::InfoDialog` | ❌ | add demo |
| 15 | Choice dialog: radio options + buttons, stepper line, option tones | J prelude.rs:164, capsule.rs:1155, config.rs:1850 | `choice_dialog::ChoiceDialog` | ❌ | add demo |
| 16 | Form dialog: inputs, select, radio, chooser, note, check, masked + `reveal_tail`, progressive disclosure, Validate action | J accounts.rs:505-680, config.rs:1912-2147 | `form_dialog::FormDialog` | ❌ | add demo |
| 17 | Help / key reference overlay (sections, multi-column, scroll, per-route content) | J app.rs:1067-1395; T app.rs:1470 (`KeyReference` in a Dialog); H modals.rs:17-300 (`TextModal`); S app.rs:986 (raw string) | `HelpOverlay`, `KeyReference`; H app-local | ❌ (the showcase's own help is a raw string) | unify on `HelpOverlay::from_keymaps`, migrate H, then demo |
| 18 | Multi-action destructive dialog (Retry / Enter plain text / Cancel launch; Stop and quit / Leave when it settles / Stay) | J cockpit.rs:262; H app.rs:1057-1129 | `Dialog::destructive().actions().cancel_label()` | ⚠️ S dialogs.rs:65-80 (confirm with 3 buttons only) | extend demo |
| 19 | Dialog anchored to a control / placement | H files.rs, app.rs; J capsule.rs, cockpit.rs, modals.rs | `Dialog::anchor/placement` | ❌ | extend Dialogs demo |
| 20 | Breadcrumb trail (clickable levels, collapses the middle, dedup) | H app.rs:3342-3362, 3664-3681; J crumb strings in segments (manager.rs:2741) | `breadcrumb::Breadcrumb` (H only; J app-local strings) | ❌ | add demo; migrate J |
| 21 | Page stack inside one tab (Esc pops; tabs are things that run, pages are things being decided) | H screens/mod.rs:120-185, app.rs:2072-2156 | ❌ app-local | ❌ | extract (`PageStack` + Breadcrumb) then demo |
| 22 | Menu bar with brand lockup, File/Go/Help menus, F10, menus built per route, choosing replays a key | H app.rs:814-842; J app.rs:883-1024, capsule.rs:242 | `menu::MenuBar` + `brand::Lockup` | ✅ S chrome.rs:41,65 | extend (per-route menus, brand opens About) |
| 23 | Top identity strip of segments (env tone `◆ production`, clickable `db › schema`, Safe Mode token, `• N pending`, spinner, `? help`, level·size; priority drop) | T app.rs:2777-2873; H app.rs:3471-3500; J app.rs:1036-1065, 2774 | `segments::{Segment, render}` | ⚠️ S chips.rs:343 (no clickable, spinner or env-tone examples) | extend App shell demo |
| 24 | Status bar: three groups, `.chip()`, `.busy()`, `.meter()`, `.strong()`, `.clickable()`, priorities, reveal | H app.rs:3503-3578; J capsule.rs:2590-2617 | `statusbar::{StatusBar, StatusItem}` | ⚠️ S chrome.rs:101 (no busy or meter items) | extend demo |
| 25 | Hint bar: layers (modal › menu › screen › fallback), EDIT badge, toggle hints, status slot with tone and 5 s TTL, centred alignment, reveal-key advert | T app.rs:2875-2916; H app.rs:3580-3659; J app.rs:2859-2998 | `hintbar::HintBar`, `keyhint::toggle_hint` (T uses `keyhint::render`); TTL app-local | ⚠️ S chrome.rs:166 (no TTL status, toggle_hint, EDIT badge or reveal advert) | extend demo; extract status TTL into the shell |
| 26 | Toast/status message with tone and expiry (frozen under Paused) | all three apps (T app.rs:551; H app.rs:455; J app.rs:2948) | ❌ app-local timer | ⚠️ showcase `Request::Status` (no tone or expiry) | extract (`StatusLine` with TTL) then demo |
| 27 | "Terminal too small" screen (brand, need W×H / have, q Quit) | T app.rs:2702-2726; H app.rs:3285; J app.rs:2615; S app.rs:1441 | ❌ copied 4 times | ⚠️ implicit in the showcase shell | extract `TooSmall` then demo |
| 28 | App shell layout (strip / crumb / tabs / body / status / hints), page dimmed under a layer | all apps app.rs | ❌ app-local (dup audit P0) | ⚠️ the showcase shell itself | extract `AppShell` (long term) |
| 29 | Context menus: row anchor vs pointer, Shift+F10, reasons on unavailable items, danger, separators, press-drag-release, items replay keys, right-click on a backdrop reopens the menu beneath | T context.rs:104-640; H screens/mod.rs:366-430; J rowmenu.rs (`RowMenu<T,C>`) | `menu::ContextMenu`, `ui::context::*`; `RowMenu`/key replay app-local | ⚠️ S row_menu.rs, lists.rs, trees.rs, tables.rs:313 (no reasons, danger or key replay) | extract `CommandMenu` then extend demo |
| 30 | App/brand menu anchored at a segment ("Open tabs…", Disconnect, Quit) | T context.rs:66-88; J capsule brand menu | `ContextMenu.at().title()` | ❌ | extend Chrome demo |
| 31 | Column header menu (sort/fit) | T context.rs:132-150 | `DataGrid::header_menu`, `HeaderAction` | ⚠️ S tables.rs:313 (DataTable only, not the grid) | extend Data grid demo |
| 32 | Open Quickly picker: fuzzy, groups, glyphs, Tab scope cycle, "open" tag, Alt+Enter alternate | T app.rs:1507-1577 | `Picker` | ✅ S pickers.rs:108 | – |
| 33 | Non-searchable level picker (Safe Mode, `›` current + tag, descriptions) | T app.rs:1641-1674 | `Picker` (`searchable=false`) | ✅ S pickers.rs:120 | – |
| 34 | Picker Loading/Error/Ready status, live suggestions as you type, `r` retry, Backspace back through a chain (1Password account › vault › item › field) | H files.rs:557-626 (go to path); J modals.rs:551-948 (OpFlow) | `PickerStatus`; chain app-local | ❌ | add demo; extract `PickerChain` |
| 35 | Command palette (disabled entries with a reason) | J capsule.rs:901-984 | `Picker` | ⚠️ (pickers page has no disabled rows) | extend demo |
| 36 | File browser modal (path input, folder list, read-only checkbox, Git URL mode with "Resolving…" spinner, stepper) | J modals.rs:61-530 | ❌ app-local composite | ❌ | extract `FileBrowser` then demo |
| 37 | Hand-built modal (filter editor: column/op selects ordered by type, BETWEEN = two inputs, "No value needed", live `WHERE …` preview, Esc only when clean) | T app.rs:1676-1742, 2918-3038 | ❌ app-local frame and layout | ❌ | move onto `FormDialog` (+ live preview slot), then demo |
| 38 | Text modal (title + subtitle, section headings, key/value, scroll + fades, light dismiss: About / Why / Help) | H modals.rs:17-300, app.rs:902-1053 | ❌ app-local (`TextModal`) | ❌ | merge into `InfoDialog`/`HelpOverlay` |
| 39 | Data grid: typed columns, nullable, enum, generated read-only, FK refs, sort/filter markers, loading, error empty, row cap/FetchMore, Alt+D duplicate, pending bar with focus stops | T tabs.rs:306-528, workbench.rs:805-887 | `grid::DataGrid` | ⚠️ S grid.rs (no `filtered_cols`, `read_only_reason`, `header_menu`, `duplicate_keys` or references) | extend demo |
| 40 | Read-only explanation (grid `read_only_reason` in footer + status on edit attempt; menu items carry reasons; "Usage · read-only" titles; discovered accounts read-only) | T tabs.rs:406-413, context.rs:517; J usage.rs:933, accounts.rs:2154 | `DataGrid.read_only_reason`, `MenuItem.reason`; rest app-local | ❌ | extend Data grid demo + add a read-only pattern demo |
| 41 | Table footer status note (`sort col ▴`, `filtered (N)`, rows/cols label, read-only reason; lowest priority dropped first) | T tabs.rs:809-855, 878-907 | ❌ app-local | ❌ | extract (`FooterNote` with priority parts, or reuse `segments`) then demo |
| 42 | Pending-change queue → typed SQL literal preview → Save review → simulated commit | T model.rs:12-138, app.rs:1263-1429 | `DataGrid.pending`; SQL app-local (domain) | ⚠️ S grid.rs (preview only) | extend (Save review with typed token) |
| 43 | Filter chip bar (match all/any lead, enable/disable, add/remove/clear all) + filter by cell value + follow FK (Ctrl+]) | T tabs.rs:529-545, workbench.rs:751-877, app.rs:615-678 | `chips::ChipBar` (`lead`, `ChipCmd`), `GridAction::FilterOnCell/FollowReference` | ⚠️ S chips.rs (no lead or enable toggle); grid.rs only logs the actions | extend demo |
| 44 | Stream toggle chips (show/hide per service, 1-9 toggle) | H activity.rs:107-130, 612-640 | `ChipBar` (`removable=false`, `enabled`) | ⚠️ | extend Chips demo |
| 45 | Chip/cell row with `+N` overflow | H finder.rs:915-962 | `chips::overflow_run`; cells app-local | ❌ | demo `overflow_run` in Chips |
| 46 | Structure tables (toned cells, `—` for none, PK/FK column, per-section empty text) | T tabs.rs:546-666 | `table::DataTable` | ✅ S tables.rs/editable.rs | – |
| 47 | DDL / raw-text card (scroll position meta, line styler) | T tabs.rs:667-710,867-884 | `ScrollPanel` (styler) + `scrollbar::position_label` | ⚠️ S panels.rs (no styler or position meta) | extend demo |
| 48 | SQL editor: app highlighter/segmenter, running statement, diagnostics, find, completion (auto + Ctrl+Space, `()` insertion) | T tabs.rs:991-1403 | `code::CodeEditor`, `completion::Completion` | ✅ S editor.rs:199 | ⚠️ add running-statement highlight and read-only mode |
| 49 | Toolbar buttons (not focusable, tooltip + shortcut, disabled while running, Cancel appears when running) | T tabs.rs:1605-1643 | `Button.tooltip().shortcut().focusable(false)` | ⚠️ S buttons.rs:26 (tooltips only) | extend Buttons demo |
| 50 | Result sub-tabs (one per result, error flag, `▪` pinned prefix not closable, selecting jumps to the statement) | T tabs.rs:1202-1256,1418-1437 | `Tabs` + app pin logic | ❌ | demo in the Query workbench screen |
| 51 | Running status line (spinner "running 1.2 s · Esc cancels", rows · duration, affected, cancelled) | T tabs.rs:1688-1764 | ❌ app-local over `spinner_frame` | ❌ | extract `ActivityLine` then demo |
| 52 | Error card (bold `!`, wrapped message/detail with `…` cut, hint) and inline failure block | T tabs.rs:1804-1868, connections.rs:1366-1418 | ❌ app-local (`Notice`) | ❌ | extract `Notice::error` then demo |
| 53 | Statement-executed / plan-node / detail fact cards | T tabs.rs:1779-1803,1976-2034; J manager.rs:1226 | `Panel::card` + `props::render` | ✅ S chips.rs (props) | – |
| 54 | Key/value detail drawn by hand (wrapped toned values, fit-to-content card, scroll + fades) | H finder.rs:652-736, disk.rs:743, cleanup.rs:800, plan.rs:488; J accounts.rs:1452 inspector line enum | ❌ app-local (`props::render` lacks scroll) | ❌ | extend `props` (scroll, `DetailCard`) then demo |
| 55 | EXPLAIN plan tree-table (reserved right metric columns, headers, >50% Warning `▲`, raw toggle `r`, node detail at ≥110 cols) | T tabs.rs:1873-2142 | `TreeView` (`reserve_right`, `trailing`); headers and metric columns app-local | ❌ | extract `TreeView::columns` (tree-table) then demo |
| 56 | Tree with glyphs, counts/meta, lazy load + busy, note rows ("No indexes"), selection glyphs ✓ · ◐ | T workbench.rs:103-321; H disk.rs:161-296; J inspect.rs:590 | `tree::{TreeView, TreeNode::lazy/note/glyph/meta/busy}` | ⚠️ S trees.rs (only `dir`/`leaf` from `data::project_tree`, no glyph, meta, lazy, note or busy) | extend demo |
| 57 | Hand-drawn trees (workspace/accounts tree with "+ New workspace" row, per-row spinner, provider folds) | J manager.rs:150,1010; accounts.rs:1301 | ❌ app-local | ❌ | migrate to `TreeView` (+ trailing action row), demo |
| 58 | Filter field over a tree/list (`/` focuses, ↓ enters the list, Esc clears, count meta) | T connections.rs:1225-1271, workbench.rs:1433-1456; J title-filter prompt | ❌ glue app-local (`TextInput` + `TreeView`) | ❌ (S sidebars has none) | extract `FilterTree`/`FilterList` then demo |
| 59 | Tri-state checkbox tree-list (category `[✓]/[~]/[ ]/[–]`, expander, size, age, eligibility; select all) | H cleanup.rs:585-798; disk top files `[✓]` | ❌ app-local | ❌ | extend `TreeView` (tri-state check column) then demo |
| 60 | Grouped result table (heading rows, column solver with droppable columns, matched chars bold, risk column never dropped, `›` cursor fill) | H finder.rs:823-1057; J usage.rs:668 | `ui::columns::solve` + app rows | ❌ | extend `DataTable` (group headers, match ranges) + demo `columns::solve` |
| 61 | Change/diff tables (`+ • −` per row, scope column, role group fold, `!` error, blocker rows, undo, inspector card) | J config.rs:750-1069 | ❌ app-local (no `DataTable`) | ❌ | extract (`DataTable` row mark + group rows) then demo |
| 62 | Plan outline table (review: required/optional checks, `01` numbers, needs, lane; run: spinner/✓/!/– state column, elapsed, waits) | H plan.rs:235-455 | ❌ app-local | ⚠️ S taskrunner (tree + bars, no outline) | extract (`StepRail` table variant) then demo |
| 63 | Step rail / launch stages (11 stages, "N/11" counter, credentials line) | J cockpit.rs:123,902-927 | `steps::StepRail` | ✅ S terminal.rs:97 | – |
| 64 | Wizard / stepper string ("✓ Source · … · Edit skipped", Back rewinds and restores) | J prelude.rs:97-482 | ❌ app-local | ❌ | extract `StepLine` + `Wizard` then demo |
| 65 | Progress: determinate bar (Active/Done/Error), indeterminate, spinner, aggregate header "k of n done · failed" | H plan.rs:1529-1646, disk.rs:1575-1645; T connections.rs:1366 | `progress::*` | ✅ S progress.rs | ⚠️ add aggregate-header example |
| 66 | Meters (line and block; tones Normal/Warning/Exhausted/Refreshing/Stale; meter list rows; hidden when compact) | H disk.rs:1660-1682, snapshot.rs:1417; J accounts.rs:1802-1949, usage.rs:691, capsule.rs:2702 | `progress::Meter` | ⚠️ S progress.rs:165 (no labelled meter list or Refreshing/Stale states) | extract `MeterList` + extend demo |
| 67 | Live log viewport (follow tail, retention-drop meta, service prefix column, find with marks + "i of n", attach, horizontal wheel, copy) | H activity.rs:82-236, files.rs:312-425, plan.rs:1655; J cockpit.rs:472 (ANSI build log) | `viewport::TextViewport` (`Mark`, `Span`) + find glue app-local | ⚠️ S terminal.rs (no find, marks, retention meta or prefixes) | extract `TextViewport::find` then extend demo |
| 68 | Attach/detach to a program (Ctrl+] detaches, hints replaced, `attached` chip) + stdin input mode (`i`, Ctrl+D EOF, EDIT badge) | H activity.rs:237-570 | ❌ app-local | ❌ | demo in a Terminal/Activity screen (behaviour stays app-level) |
| 69 | PTY panes: nested split tree, prefix commands (Ctrl+B …), pane zoom, focus/hover borders, caret, scrollback, "redrawn" flash, takeover screen | J capsule.rs:1543-1693,2059-2588; sim/pty.rs:25-160 | `TextViewport` (`CellPos`), `Split`, `Splitter::render_held`; pane tree app-local | ❌ | extract `PaneTree` (n-ary split + zoom) then demo |
| 70 | Diff viewer in a modal (file list/tree + diff, unified/review toggle `d`, stacks when narrow) | J inspect.rs:365-590 | `diff::DiffView` + `TreeView` + `Split` | ⚠️ S diff.rs (single file, no file tree) | extend demo |
| 71 | Forms: two-column measured stacks scrolling by field, Basic/Advanced tabs with an error marker, side-by-side Host/Port 70/30, inputs disabled by a toggle (SSH), masked password + `reveal_tail`, engine select resets the port, radio + description, validation jump to the first bad field, Ctrl+S, test-connection busy + inline ✓/! result | T connections.rs:160-953,1432-1655; H review.rs:1016-1359 (args form + live command preview); J editor.rs:531-633 | `TextInput`, `Select`, `Checkbox`, `RadioGroup`, `Toggle`, `TextArea`, `Stack`, `Tabs.items[].error`; engine app-local | ⚠️ S forms.rs (single column; no tabs with error, masked, select, dependent-disable, inline test result or `Stack` scroll), S settings.rs | extract `FormPanel` then extend demo |
| 72 | Leave guard (dirty form/screen → Save / Discard / Cancel; letters never leave; quit lists what will be lost) | T connections.rs:417-497, app.rs:460-511; H app.rs:1057; J app.rs:843, editor.rs:2369 | `ui::leave::*` | ✅ showcase shell (`leave_stakes`, grid/selectable) | ⚠️ add an explicit leave demo with the 3-way dialog |
| 73 | Editor shell (tabs with dirty/error, `[`/`]`, Cancel/Save… disabled when clean, "Saving…" busy, "• N changes" strip segment, save preview facts with diff lines) | J editor.rs:2195-2338, settings.rs:106-1283, config.rs:345 | ❌ app-local (copied twice) | ⚠️ S settings.rs (tabs + form, no dirty tabs, save preview or busy save) | extract `EditorShell` then demo |
| 74 | Button rows: right-aligned Cancel + primary/danger, wrap to a second line, follow-up "Next" subtle row with disabled items, inline message after the last button | H plan.rs:1031,1731-1762, review.rs:574; J accounts.rs:1708, editor.rs:2242; T connections.rs:1627 | `row_layout_right`, `ButtonRow`/`RowOverflow::Wrap`; message app-local | ⚠️ S buttons.rs (`row_layout` only) | extract `ActionRow` + extend demo |
| 75 | Connect/launch lifecycle (spinner + phase text, failure then "Reconnect", busy button) | T connections.rs:543-596; J cockpit | `Button.busy`, `render_spinner` | ⚠️ S buttons.rs (busy) | demo in a Connection screen |
| 76 | Page header (title + toned status clauses + right meta + muted subtitle; section `heading()`) | H 11 sites (activity.rs:836-918, disk.rs:1568, review.rs:495; screens/mod.rs:476); J ~17 headings | ❌ app-local | ❌ | extract `PageHeader`/`Heading` then demo |
| 77 | Finder query row (always typing-hot, scope readout `scope ‹ here ›`, undo/redo, Ctrl+U, `@scope` prefixes) | H finder.rs:422-475,738-811,1137-1330 | ❌ app-local (hand-drawn field) | ❌ | migrate to `TextInput` (right readout slot) then demo |
| 78 | Search field with right readout ("scope: conn · status: any", "n of N indexed") | T tabs.rs:2428-2533; H files.rs:817-924 | ❌ app-local overdraw | ❌ | extend `TextInput` (trailing readout) then demo |
| 79 | History list + detail (search ANDs terms, scope/failed toggles `c`/`s`, red `!` on failures, read-only `CodeEditor`, Open/Run/Copy buttons) | T tabs.rs:2428-2713, workbench.rs:889-984 | `ListBox` + `CodeEditor.read_only` + app `!` overdraw | ❌ | extend `ListItem` (status tone) then demo |
| 80 | Files browser list (kind glyphs, hidden muted, right-aligned size, mtime, dirs first, Ctrl+H) + directory preview card | H files.rs:658-1018 | ❌ app-local rows | ❌ | migrate to `DataTable` and demo |
| 81 | Snapshot read-only pages (title + `live · ago` meta, meters, facts, framed toned body, Next buttons, `r` refresh) | H snapshot.rs:39-1586 | mix (`props`, `Meter`, `Panel`); body app-local | ❌ | demo as a "Read-only report" screen |
| 82 | Usage screen / usage dialog (tabs per account, quota meters with freshness tones, refreshing spinner meta, list ↔ detail on narrow screens) | J usage.rs:462-938, capsule.rs:2702-3098 | `Tabs` + `Meter` + `columns::solve`; rows app-local | ❌ | demo in Feedback/Meters + Screens |
| 83 | Intro / outro ritual (typewriter, glitch caption, warp starfield, Enter skips, reduced-motion static) + atmosphere rain + cross-fade handoff (`dim_buffer`) | J rain.rs:102-855, app.rs:2652-2670 | ❌ app-local | ❌ | extract the generic parts (`ui::fade::dim_buffer` cross-fade, motion policy) then demo. The intro and outro animations and the rain stay in jackin (product owner decision 2026-10-16): not a library capability, never demonstrated, a `permanent` exclusion in the coverage allowlist |
| 84 | Motion policy (`--motion full/reduced/paused`, `--frame N`, 80/200/400 ms ticks, flash frozen when paused) | T app.rs:100-288; H app.rs:258-310; J app.rs:69-82 | ❌ app-local (×3) | ⚠️ the showcase has `Interaction` but no visible switch | extract `Motion` then add an Overview/Feedback toggle |
| 85 | Enter/Space press flash, focus-on-press, focus ring only after keyboard focus | all apps | `PointerState::flash`, `FocusOn` | ✅ (shell) | – |
| 86 | Scrollbars + edge fades on hand-drawn lists (track repeat, drag autoscroll) | H screens/mod.rs:522-544 (14 copies); J manager.rs:1010, usage.rs | `scrollbar::render_vertical`, `fade::scroll_edges` | ⚠️ S scrolling.rs, sidebars.rs (`scroll_edges_except`) | add `render_with_fade` helper + demo |
| 87 | Reveal truncated cell/row (reveal key advert) | T, H, J app.rs | `ui::reveal::{Reveal, advertise}` | ❌ | add demo |
| 88 | Tooltip on a status/strip item (`StatusItem::tooltip`, `tooltip::on_input`) | J | `ui::tooltip` | ⚠️ S buttons.rs:26 (buttons only) | extend Chrome demo |
| 89 | Clipboard copy with truthful result (OSC 52 size limit or error status) | H app.rs:2947-2971; J inline ×3 | ❌ app-local (library `core::edit::clipboard` hook unused) | ⚠️ showcase copy (status text only) | extract (use the library hook) then demo |
| 90 | Safe Mode / environment semantics (`◆ production` tone, Warning when silent on prod, gate Run/Confirm/Deny by worst statement) | T app.rs:1116-1260,2777-2873, connections.rs:1286-1365 | ❌ domain (app-local, fine) | ❌ | demo the visual parts (env-toned segment, gate dialog) only |
| 91 | Copyable facts / `y` copy rows in the Gate 1 sequence | H review.rs:41-630 | `props`, `Panel::framed`; rows app-local | ❌ | covered by `InfoDialog` copyable demo |
| 92 | Global keyboard grammar (Esc ladder: unmaximise › content › strip › sidebar › clear filter; `[`/`]` tabs; Ctrl+T/W/O/P/G/B; letters suppressed while editing) | T app.rs:713-1056; H app.rs:626-810; J app.rs:610-842 | ❌ app-local (keymap types are library) | ⚠️ the showcase has `[`/`]` and `?` | demo the Esc ladder in the workbench screen |

## Showcase pages and demos to add or extend

**App shell**
- **Chrome (extend S chrome.rs)**
  - Identity strip of `Segment`s: clickable app name opening an anchored app menu, env tones (`◆ production` bold / `◇ staging`), clickable `db › schema`, `• N pending` warning, spinner "running", `? help`, `level · W×H`, priority drop on resize.
  - Status bar items with `.busy()`, `.meter(pct, MeterTone)`, `.tooltip()`, clickable chip opening a picker.
  - Per-route menus; brand item opens About (`InfoDialog`).
- **New page "Status & hints"**
  - `HintBar` layer precedence (modal › menu › screen › fallback).
  - EDIT badge while a field edits; `keyhint::toggle_hint` labels that flip with state.
  - Status slot with tone, 5 s TTL, frozen under Paused; reveal-key advert when the cursor row is truncated.
  - Centred vs left alignment.
- **New demo "Too small"**: a resizable mock or `--size` showing `TooSmall` (brand, need/have, q Quit) once extracted.
- **New demo "Leave guard"**: dirty page → q / Ctrl+Q / Esc gives Save / Discard / Cancel; quit dialog listing what will be lost; letters never leave a field.
- **Motion toggle** (Overview or shell): full / reduced / paused showing flash, spinners and fades.

**Inputs & forms**
- **Forms (extend S forms.rs; mirror tablepro connections)**
  - Basic/Advanced `Tabs` with an error marker on a tab.
  - Two-column `Stack`s that scroll by field on short terminals.
  - Host + Port side by side 70/30; `Select` whose change resets another field.
  - Masked password with `reveal_tail`; "Prompt for password" checkbox; `RadioGroup` with a description line below.
  - SSH `Toggle` that disables and hides dependent inputs.
  - `TextArea` (startup commands); muted note line.
  - Submit: validation focuses the first bad field + status; Ctrl+S.
  - Test button busy → inline `✓ ok` / `! err` after the button row.
- **Buttons (extend S buttons.rs)**
  - Toolbar row: `focusable(false)`, `.shortcut()` labels, disabled while running, Cancel appearing while busy.
  - `row_layout_right` (Cancel + primary/danger).
  - `ButtonRow` with `RowOverflow::Wrap`.
  - "Next" follow-up row of subtle buttons with disabled ones.
  - `ActionRow` with a trailing message.
- **Inputs (extend S inputs.rs)**
  - `TextInput` with a right readout ("scope: conn · status: any", "n of N indexed").
  - Always-typing query field (undo/redo, Ctrl+U, scope chips `‹ here ›`).
  - `reveal_tail(n)` on secrets.
- **Chips & selects (extend S chips.rs)**
  - `ChipBar` `lead` ("match all ▾" / "match any ▾"), enabled/disabled chips, `ChipCmd` add/remove/clear all.
  - Non-removable toggle chips (stream show/hide, 1-9 toggle).
  - `chips::overflow_run` row with `+N`.
  - Segments with clickable items.

**Tables**
- **Data grid (extend S grid.rs)**
  - Typed columns incl. enum, nullable, generated read-only, FK references; `filtered_cols` + sort markers.
  - `read_only_reason` variants (view / no PK / query result) with `GridAction::ReadOnly` status.
  - `header_menu`; `duplicate_keys` (Alt+D); loading; error empty; FetchMore row cap.
  - Filter-on-cell and follow-reference actually opening something.
  - Footer note with priority-dropped parts (sort, filtered N, rows, cols, read-only reason).
  - Save review `Dialog::facts` with a typed table-name token.
- **New "Grouped results" demo**: `DataTable` with heading rows, `columns::solve` droppable columns, matched-char bold, never-dropped risk column, faint hidden rows.
- **New "Change table" demo**: rows marked `+ • −`, group fold rows, error `!`, blocker rows, undo, inspector card.
- **New "Checklist table" demo**: tri-state category rows `[✓]/[~]/[ ]/[–]` with expanders, size/age columns, select all, ineligible rows quiet.
- **History list**: `ListItem` with a failure tone/`!`, meta "when · duration", search with AND terms + scope/failed toggles, detail with read-only `CodeEditor` + Open/Run/Copy.

**Navigation**
- **Tabs (new page)**
  - `TabItem` prefix icons (`T`/`≡`/`H`), suffix (`…`/`✓`/`○`), `closable` `×`, dirty/busy/error flags.
  - `allow_new` `+`; `‹ ›` scroll; `overflow_menu` `+N`; wheel; middle-click close; digit jump.
  - `TabRule` accent vs neutral under a layer.
  - Nested strips: main tabs › segmented Data/Structure (Ctrl+D) › sub-tabs with 1-6 jump and overflow at narrow width.
  - Preview tab (single click replaces, Enter promotes) once extracted.
  - Pinned tab (`▪`, not closable).
  - Close-dirty-tab confirm.
- **Sidebars (extend S sidebars.rs)**
  - Real `rail::SidebarRail`: framed panel with `‹` collapse in the title row, 2-col rail with `›` + vertical label, click / Ctrl+B / `0` to expand.
  - Narrow drawer behaviour (covers the body while focused).
  - Replace the hand-made collapsed mode (or extract it as `NavList`).
- **Breadcrumbs (new)**: `Breadcrumb` with clickable earlier levels, middle collapse to `…`, last item middle-truncated; page stack with Esc pop.
- **Trees (extend S trees.rs)**
  - `TreeNode::glyph`, `meta` counts/sizes, `lazy` with delayed `set_children` + busy, `note` rows ("No indexes"), selection glyphs ✓ · ◐.
  - Filter field above the tree (`/`, ↓ into the tree, Esc clears, count meta) as `FilterTree`.
  - Tree-table: right metric columns + header row, hot-row warning `▲`.
  - "+ New …" action row.
- **Lists (extend S lists.rs)**: status-toned items, header rows, cursor skipping headers.

**Overlays**
- **Dialogs (extend S dialogs.rs)**
  - `Dialog::facts` variants: read-only viewer (Close only, code/JSON); confirm with facts + SQL/command code; typed-token danger (initial focus Cancel); multi-action destructive (Retry / Alternative / Cancel with `cancel_label`).
  - `Dialog::anchor` / `placement` under a control.
  - Gate "1 of 2 / 2 of 2" titles.
- **Info dialog (new)**: `InfoDialog` with copyable rows (`y`), `.error()` variant, "read-only" meta, detail block, actions.
- **Choice dialog (new)**: `ChoiceDialog` with radio options, `option_tones` (error-toned last option), `.line`, `.stepper`, Back/Next.
- **Form dialog (new)**: `FormDialog` with input / select / radio / chooser / note / check, masked + `reveal_tail(4)`, progressive disclosure via `form_changed`, Validate action, live preview slot (WHERE / command preview). Also the target for tablepro's filter editor.
- **Help (new)**: `HelpOverlay` (sections, multi-column, scroll, refuses over another modal), and `KeyReference` with hanging wrap. Ideally generated from the real keymaps; replace the showcase's raw-string help.
- **Pickers (extend S pickers.rs)**
  - `PickerStatus` Loading / Error (`r` retry) / Ready; live suggestions as you type (go to path).
  - Picker chain with crumb scope and Backspace back.
  - Disabled rows with a reason (command palette).
  - Delete as the secondary action on the tab list; live-refreshing switcher with an "N live" scope.
- **Context menus (extend S row_menu.rs / tables.rs)**: `MenuItem.reason` for unavailable items, `.danger`, separators, press-drag-release, items replaying keys (`CommandMenu`), app menu anchored at a strip segment, grid header menu, right-click on a backdrop reopening the menu beneath.
- **File browser (new, after extraction)**: path input, folder list, read-only checkbox, Git URL mode with spinner, stepper.

**Feedback**
- **Progress (extend S progress.rs)**: aggregate header ("k of n done · running · failed" with only the failed clause toned); scan line (spinner + "n of N" + inline bar → done summary / `▲` partial / cancelled); "running 1.2 s · Esc cancels" `ActivityLine`.
- **Meters (new or extend)**: `MeterList` of labelled line/block meters with value text; tones Normal / Warning / Exhausted / Refreshing / Stale; hidden when compact; status-bar meter.
- **Notices (new)**: `Notice::error` (bold `!`, wrapped message + detail with `…` cut, hint); inline connect failure; toast/status with tone + TTL.
- **Steps (extend S terminal.rs)**: `StepRail` with a counter segment "N/11"; `StepLine` wizard stepper ("✓ Source · … · skipped") with Back/rewind once extracted.
- **Empty states (extend)**: `EmptyState` with `EmptyAction` primary/secondary buttons via `render_ctx`; error variant; context-dependent hints.
- **Read-only explanations (new)**: read-only grid reason, menu-item reasons, "· read-only" panel meta, status on an attempted edit.

**Layout**
- **Panels (extend S panels.rs)**: `meta` + `meta_short` fallback, `badge`, `PanelFocus` marks, `ScrollPanel` with a line styler and `position_label` meta, `title_min`.
- **Splits (new page or extend terminal)**: horizontal and vertical `Split` with drag, double-click reset, Ctrl+↑/↓ nudge, clamped default until dragged, three-pane, pointer shapes; `Split::toggle_max`/`Maximized` zoom with `z`, Esc restore and chrome hiding.
- **Master-detail (new, after extraction)**: list + detail at wide sizes; narrow summary card / one-line summary; drawer on focus or `p`; hidden pane kept as a focus stop.
- **Pane tree (new, after extraction)**: nested n-ary splits, focus/hover borders, zoom, resize chords.
- **Scrolling (extend S scrolling.rs)**: `scrollbar::render_vertical` + `fade::scroll_edges` on a hand list, track repeat, drag autoscroll, `Reveal` of a truncated row.

**Text & code**
- **Code editor (extend S editor.rs)**: running-statement highlight (`set_running`), read-only mode re-wrapped to the pane, statement segmenter, diagnostics at the parse offset, `()` insertion on function completion.
- **Terminal / log (extend S terminal.rs)**: `TextViewport` find (`Mark`s, "find “q” · i of n", Enter/↑ next/prev), retention-drop meta, service prefix column + toned/error lines, ANSI spans, follow / `▲ depth · f follows`, horizontal wheel, `CellPos` caret, attach/detach + stdin input mode demo, "redrawn" flash.
- **Diff (extend S diff.rs)**: multi-file changes tree + diff preview following the cursor, vertical stack when narrow, compact list modal with an unpushed-commits row.
- **Animation (new)**: cross-fade dim (`dim_buffer`) and reduced-motion static fallback. The jackin intro and outro (typewriter, glitch, starfield, rain) are not demonstrated: product owner decision 2026-10-16, they stay brand-specific in jackin.

**Screens (composed)**
- **Query workbench**: Explorer sidebar with rail collapse + filter + lazy tree; main tabs with icons, `×`, `+`, overflow and a preview tab; query tab with toolbar, editor/results split, zoom `z`, result sub-tabs (pinned, error), running status line, EXPLAIN tree-table with raw toggle and node card, error card; table tab with Data/Structure strip, structure sub-tabs, DDL card, filter chips, grid footer note, pending bar, Save review with typed token; tab list Ctrl+G; Open Quickly Ctrl+O; Safe Mode picker; Esc ladder.
- **Connection manager**: grouped connection tree with env glyphs + filter; empty state with "New connection"; detail facts card with a Warning row; Connect / Edit / Duplicate / Delete…; connect spinner phases → failure block → Reconnect; the form from Inputs & forms.
- **Finder / launcher**: typing-hot query with scope readout, grouped ranked table, preview card or narrow one-line summary, alternatives menu (Alt+Enter), "Why" explanation.
- **Disk / cleanup**: usage tree with sizes / pct / scanning nodes, filesystem meters, scan progress, tri-state cleanup checklist with mode/danger summary (Trash vs PERMANENT · dry run), Gate 1 review page → Gate 2 typed phrase.
- **Plan runner**: outline table review → run with live state column, aggregate progress, step output pane with zoom, outcome line + Next row (extends the Task runner).
- **Activity**: header with state clauses, stream chips, live log, attach/input modes, stop/restart/close, status-bar contributions.
- **Workspace manager**: tree + seam + detail + roster at ≥150 cols, narrow drawer, sessions list, per-row spinners, purge typed confirm.
- **Accounts / usage**: account tree with provider folds, inspector with meters, `FormDialog` add/edit, picker chain for secret references, usage dialog with per-account tabs.
- **Editor shell**: tabs with dirty/error, change tables, Save preview with diff lines, "• N changes" strip, busy Save (extends Project settings).
- **Launch cockpit**: `StepRail` stages, activity chip opening the build log overlay, Cancel launch, cross-fade handoff.
- **Read-only report**: snapshot-style page with `live · ago` meta, meters, facts, toned body, `r` refresh, Next buttons.

## Extract to the library first (app-local today)
In priority order:
1. `MasterDetail` / drawer (#10)
2. `FormPanel` (#71)
3. `HelpOverlay::from_keymaps` (#17, #38)
4. `CommandMenu` / key-replay menus (#29)
5. `TreeView` extensions: tree-table columns, tri-state check, trailing action row (#55, #57, #59)
6. `DataTable` group rows and row marks (#60, #61)
7. `PageHeader` / `Heading` (#76)
8. `Notice` / `ActivityLine` (#51, #52)
9. `TextViewport::find` (#67)
10. `EditorShell` (#73)
11. `FileBrowser` / `PickerChain` / `StepLine` + `Wizard` (#34, #36, #64)
12. `TooSmall`, status TTL and `Motion` (#26, #27, #84)
13. `ActionRow` (#74)
14. Tab preview flag (#5)
15. `PaneTree` (#69)
16. `FooterNote` (#41)
17. `TextInput` trailing readout (#78)
18. `MeterList` (#66)
19. `scrollbar::render_with_fade` (#86)
20. A clipboard result built on the library hook (#89)

Several of these are already proposed in `docs/audits/design-review-2026-10/duplication-screens.md` (sections "Ranked findings" and "New library components to add").