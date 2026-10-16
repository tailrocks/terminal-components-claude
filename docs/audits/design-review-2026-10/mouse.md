Audited `origin/design/interaction-polish` at 9776abef7, mouse first. Most findings were checked against the code. Throwaway harness tests in showcase, tablepro, holla and jackin confirmed each probed behaviour as broken or missing at that revision; they were not committed. Severity scale: H = dead end or clearly wrong, M = gap against desktop behaviour, L = polish.

## 1. Mouse interaction model to adopt library-wide

**Routing (the root cause of most gaps).** Each of the four apps has its own ~150-line `on_mouse` router. Move this into the library:
- A `core::pointer::PointerRouter` turns `Mouse` plus `HitRegistry` into typed events: `Press{id,pos,count,mods}`, `Click{id,count,mods}`, `Drag{id,pos,grab}`, `Secondary{id,pos}`, `Middle{id}`, `Wheel{id,dx,dy}`.
- Every widget implements one `MouseTarget` trait. Apps only map an id to its owner.
- Modals get the same routing, with the wheel sent through `hit_scroll`.
- A grabbed `hit::Bar` goes to `scrollbar::apply` automatically.
- Middle and secondary clicks are dispatched generically.

**Affordance.** If it reacts to the mouse, it is drawn through `Theme::interactive` with rest, hover and pressed states. If it doesn't, it never hovers.
- Key hints, links and inline key mentions become clickable title-ladder items.
- Selects show ▾. Editable cells are underlined (already done).
- Splitter seams use a glyph that is not the scrollbar track glyph.
- Optional: pointer shape through OSC 22 (`PointerConfig::pointer_shapes`, default on where supported: text, pointer, col-resize, row-resize).
- Caret shape through DECSCUSR (`caret_shape: Bar`).

**Hover.**
- Controls react immediately.
- The truncated-text reveal waits `Reveal::delay_ms` (default 300).
- A new generic `Tooltip` shows a description plus its shortcut after `tooltip_delay_ms` (default 600), for buttons, icons, hints and segments.
- When the pointer leaves, hover clears, including in menus.

**Press and release.**
- The pressed look shows on Down and cancels when dragged off. Activation happens on Up over the same target (exists).
- Menus and menu-bar titles open on Down (`MenuBar::open_on: Press`). Releasing over an item chooses it (`press_drag: true`).
- Context menus open on secondary Down. A secondary Up over an item after 1 cell or more of movement chooses it. This needs `SecondaryUp` and `SecondaryDrag` in `event.rs`.

**Click counts** (`double_click_ms` 500, slop 1, exists):
- 1 click: focus, select, and place the caret.
- 2 clicks: open or activate, edit a cell, select a word.
- 3 clicks: select a line or everything.
- Knobs: `ClickAction::{Select (default), Activate}`, `edit_on: DoubleClick`, `CheckClick::BoxOnly`.

**Modifiers.** Shift extends, Cmd or Ctrl toggles (`TOGGLE_CTRL_CMD`/`TOGGLE_WITH_ALT`), Alt picks the alternative or acts recursively. On macOS Ctrl-click is a secondary click (exists). Every selection widget and every text field takes `mods`.

**Right-click.**
- First fix the selection the way macOS does: a click inside the selection keeps it; outside, it selects the clicked item only.
- Then open the menu. Each widget ships default `context_items()` built from its `KeyMap` labels and shortcuts.
- Text widgets get a standard Edit menu (Cut, Copy, Paste, Select All) backed by OSC 52 (`field_context_menu: true`).
- Empty space gets a background target.

**Drag.**
- Drag starts after `drag_threshold` (1 cell).
- The scrollbar thumb keeps its grab offset. A track press pages with hold-repeat (`TrackClick::Page` on macOS).
- Splitters get a hit area padded ±1 cell; double-click resets or maximises.
- Text and range selection autoscroll on a tick while held past the edge (`autoscroll_ms` 50).
- Tab and column reorder and column resize are opt-in knobs.

**Wheel.**
- It always goes to the innermost scrollable under the pointer, including inside modals, on both axes.
- It scrolls the viewport, never the cursor.
- Lists follow the cursor only after a cursor move (`follow_cursor: OnMove`).
- Use the same `wheel_lines` on both axes. Shift+wheel scrolls sideways (exists).

**Focus on click.**
- `FocusOn::Press` (exists).
- Clicking a label focuses its field.
- A press on dead space ends the edit (exists).
- Buttons, tabs, chips and rows use `FocusVisible::KeyboardOnly`; text fields always show the caret.

**Caret.**
- A single click on a field starts editing with the caret at the click cell, accounting for horizontal scroll and wide characters (exists for TextInput and TextArea; missing for CodeEditor, Picker, finder and the Grid cell editor).
- `type_to_edit` defaults to true for search fields.

**Dismissal.**
- `Dismiss::LIGHT` for menus, popovers and info dialogs; `ESCAPE` for forms.
- A refused outside click gives visible feedback (`Dismiss::refuse_feedback: Flash`).
- Every modal has a visible way to close it: a × in the frame or a Cancel/Done button.

**Rule: no keyboard-only state.** Every `KeyMap` binding shown in a footer is either clickable there or present in a menu. Enforce this with a coverage test that walks each screen's `KeyMap`.

## 2. Ranked findings

| # | Sev | App/screen or widget | Root cause (file:line) | What the mouse user experiences | Library fix (+ knob/default) |
|---|---|---|---|---|---|
| 1 | H | Footer in all 4 apps (keyhint, hintbar) | `widgets/keyhint.rs:198` and `widgets/hintbar.rs:59` render without `RenderCtx`, so nothing is registered | Every "Ctrl+S Save", "Esc Back", "x Cancel" is dead text. Under a LIGHT dialog, clicking "y Delete" lands outside and cancels it | `HintBar::render_ctx` registers one id per hint, with hover and press; a click replays the key. `HintBarStyle::clickable=true`. Register the footer as a surface under modals |
| 2 | H | CodeEditor (tablepro query, showcase editor) | `code.rs:666-688`: the drag anchor is set only on a completed click; release runs `jump_to`, which clears the selection; there is no press handler and no click count | Drag-select starts at the previous click and vanishes on release. No double- or triple-click selection | Implement `PointerField` (`on_press_count` / `click_select`). Make `offset_at_pos` signed so a drag above or left of the text scrolls |
| 3 | H | All text surfaces | `field_common.rs:225-317` has no Copy or Cut; no ContextSource for Input, TextArea, CodeEditor, TextViewport or ScrollPanel; copy is the `y` key only (`diff.rs:176`) | Mouse capture turns off the terminal's own selection, so the user can select text but never copy it. Right-click on any text does nothing | Edit menu ContextSource with an OSC 52 sink. `field_context_menu=true` |
| 4 | H | jackin Roles, Mounts/Env, usage, accounts lists | `editor.rs:687,956`, `config.rs:839`, `usage.rs:468`, `accounts.rs:1331` call `ensure_visible(cursor)` on every render | Wheel and scrollbar snap back. Long lists scroll only from the keyboard (harness-confirmed on Roles) | `ScrollState::follow_cursor: OnMove` (default) |
| 5 | H | holla page stack (Disk, Files, Cleanup) | `screens/mod.rs:350`: Esc is the only way back; the crumb is a single tab label (`app.rs:3266`) | No way back one page by mouse (harness-confirmed) | `Breadcrumb` widget with a hit per segment that pops to that depth. `crumbs_clickable=true` |
| 6 | H | Dead-end screens: jackin cockpit, capsule takeover, intro/outro; tablepro empty states | `cockpit.rs:574-654`, `capsule.rs:1934-2049`, jackin `app.rs:1735`, `empty.rs:16-46` (no action) | Can't cancel a launch, close the build log, reconnect, skip the intro (with Paused motion), or create the first connection by mouse | `EmptyState::actions(Vec<Button>)`, `PointerConfig::click_skips_animation=true`, give the log overlay a `Dismiss` |
| 7 | H | tablepro query tab | `tabs.rs:1593-1690`; key handling at `app.rs:742-754` | Run, Run all, Explain and Cancel are chords only; "Esc cancels" is plain text | Toolbar or button-row block plus the editor context menu (#3) |
| 8 | H | tablepro explorer | `workbench.rs:1290-1334` (narrow drawer); Ctrl+B at `app.rs:794` | Once hidden or unfocused, nothing on screen brings it back | Collapsible sidebar with a clickable rail. `collapsed_rail=true` |
| 9 | H | DataGrid, DataTable edit entry | `grid.rs:1544-1558`, `table.rs:1006`: a second click on the cursor cell edits with no click count | A slow second click, or the first click into the grid, starts editing by accident | `edit_on: DoubleClick` (default). The click that gives focus never edits |
| 10 | H | DataGrid, Table, List right-click | `ui/context.rs:304-307` moves the cursor but keeps the anchor; Table and List keep their selection (`:262-269`) | Right-click outside a range stretches it to the clicked cell; Copy and Delete then hit the wrong cells | Selection rule from the model, applied in `context_select` |
| 11 | H | Tabs overflow (all apps) | `tabs.rs:388-395` moves `first`, then render resets it (`:461-480`); no `scrollable` (`:435`) | ‹ and › do nothing while the active tab is at the edge; the wheel does nothing (harness-confirmed in tablepro); hidden tabs can't be reached | Keep the visible window separate from the active tab; `Tabs::on_wheel` on both axes; a `+N` overflow menu |
| 12 | H | MenuBar, ContextMenu | `menu.rs:915` acts on Up only; `event.rs:235-241` maps only right-button Down | Menus open on release; press-drag-release does nothing; right-drag-release does nothing | `MenuBar::open_on=Press`, `press_drag=true`; `MouseKind::SecondaryUp/SecondaryDrag`; `ContextMenu::release_selects=true` |
| 13 | H | HelpOverlay, Dialog and Frame chrome | `help.rs:94,108-110` leaves ~2 columns of backdrop; `dialog.rs:1048`, `ui/modal.rs:320-336` draw no × | Help can be closed only by hitting a thin edge strip (it opens with `?` only); no modal has a close control | `Frame::close_button` (default on for LIGHT and ESCAPE); `HelpOverlay::max_size` 80% |
| 14 | M | Pointer routing in the 4 apps | `tablepro/app.rs:2290`, `showcase/app.rs:826`, `jackin/app.rs:1833` send Middle to Ignored; showcase horizontal wheel is ±1 vs ±3 elsewhere (`showcase/app.rs:856`); showcase swallows right-click in dialogs (`:830`) | Middle-click doesn't close tabs even though `Tabs::middle_click_closes` defaults on (harness-confirmed); behaviour differs per app | `PointerRouter` + `MouseTarget` (model, routing) |
| 15 | M | List, Tree, Table single click | `list.rs:143` (Activate), `tree.rs:259,707`, `table.rs:1015`; holla `disk.rs:1338` toggles the check on any click; tablepro `connections.rs:839-848` has an empty double-click branch | One click opens or commits; in holla Disk a click marks the row for deletion (harness-confirmed); double-click on a connection does nothing | `ClickAction::Select` default with double-click to activate; `CheckClick::BoxOnly` |
| 16 | M | Modifier clicks | `list.rs:396`, `grid.rs:1463,1508`, `field_common.rs:160-172` take no mods | No Shift-click range or Cmd-click toggle in List or Grid; a grid gutter click toggles rows; no Shift-click extend in fields | `on_click_with(..., mods)` everywhere, as in `DataTable::on_click_cell_with` |
| 17 | M | Scrollbars in Picker, Completion, TextArea, jackin Help | `picker.rs:939`, `completion.rs:338`, `textarea.rs:483` draw and register bars with no handler; `scrollbar::apply` is never called | Thumbs light up on hover, but click and drag do nothing (prior audit #10 still open) | The router delivers bar grabs generically through a `ScrollOwner` trait |
| 18 | M | Wheel inside modals | `form_dialog.rs:626`; `tablepro/app.rs:2309`; `holla/app.rs:1757`; `jackin/app.rs:1890` | The wheel over an open Select scrolls the form behind it; Choice and Filter lists swallow the wheel; the wheel over the backdrop scrolls the dialog (prior #8 still open) | Wheel routed by `hit_scroll` in modals; bodies register `ctx.scrollable` |
| 19 | M | Select | `select.rs:423-439` (wheel moves cursor), `:571-612` (no bar; ▴/▾ not clickable; hover separate from cursor) | Wheel changes the highlighted value; two highlighted rows; long lists can't be dragged | `WheelMode::Viewport`, `hover_moves_cursor=true`, a real scrollbar, press-drag-release |
| 20 | M | Splitters | tablepro has no handle (`tabs.rs:1593,2478`; `workbench.rs:1295` fixed width); `splitter.rs:74` 1-cell hit; `ui/layout.rs:75-91` no grab offset and whole-percent steps | Panes can't be resized by mouse in tablepro (Ctrl+↑/↓ only); elsewhere the seam is tiny and jumps | `Split::render_handle(ctx,id)` by default; hit pad ±1; cell-precise drag with grab offset; double-click to reset or maximise |
| 21 | M | Picker query, holla finder and files queries | `picker.rs:687-724`, `finder.rs:1137-1330`, `files.rs:1712` (hand-built `String`) | Clicking the query doesn't place the caret; no drag or word select; in a long query the caret sits on the ellipsis; no clear × | Embed `TextInput`; `clear_button=true` for search fields |
| 22 | M | Widget actions with no mouse path | `grid.rs:432-469` (insert, delete, undo, filter, copy); `chips.rs:183,205` (toggle, clear all); `tree.rs:178-191` (expand all) | Users can't undo, insert or delete rows, disable a filter chip or expand all by mouse | Default `context_items()` per widget built from `KeyMap` labels |
| 23 | M | App actions with no mouse path | tablepro `app.rs:711,785,834,845,897`, `tabs.rs:1400,1441` ("r Raw" looks like a hint), `context.rs:96-117` skips result tabs; holla `disk.rs:1129-1215`, `files.rs:1248-1337`, `activity.rs:420-480`; jackin `editor.rs:1268-1324`, `settings.rs:734-964`, `inspect.rs:653-657` | Zoom, History, Find, pin and raw view; sort, fold, select all; parent folder, go to path, hidden files; detach and find; role load and set default are all keyboard-only | Default View menu in the shared MenuBar; `RowMenu` everywhere; inline key mentions become links (#1). Add a coverage test (model) |
| 24 | M | Refused outside click | `dialog.rs:975-981`; `choice_dialog.rs:44` and `form_dialog.rs:181` default to ESCAPE | Clicking the backdrop does nothing visible and feels broken | `Dismiss::refuse_feedback: Flash` (default) |
| 25 | M | Reveal tooltip | `reveal.rs:518` shows instantly; `:571-600` holds the wheel | Sweeping across a list flashes overlays; the wheel stalls over a long cut cell | `Reveal::delay_ms=300`; pass the wheel through at the tooltip's ends |
| 26 | M | Buttons, icons, segments | No tooltip API exists | Shortcuts can't be discovered on hover (only menus show them) | `Tooltip` primitive: `Button::tooltip` / `shortcut`, delay 600 |
| 27 | M | Drag past the edge | `grid.rs:1569-1592`; `input.rs:462-470`; `textarea.rs:307-316`; `viewport.rs:1356` | Range and text selection stop at the edge unless the mouse keeps moving; a gutter drag does nothing; Table has no row drag | Tick-driven autoscroll in `PointerState` while a press is held outside the region |
| 28 | M | Table and Grid headers | `ui/columns.rs:75` has no user widths; `table.rs:1213`, `grid.rs:1925` cut titles | No column resize or reorder, no header menu, no reveal on cut headers | Seam hits on column edges with `on_header_drag` (double-click autofits); header ContextSource; reveal on headers |
| 29 | M | Tree | `tree.rs:851-853` skips rows before `clickable` (`:928`); note rows hover at `:835-845` | Deeply indented rows can't be clicked; note rows show hover but do nothing | Register before the early `continue`; note rows are disabled; Alt-click expands recursively |
| 30 | M | Chips | `chips.rs:263-270`: `+N` is not registered; × is 1 cell (`:293`) | Chips past the overflow marker are unreachable | Clickable `+N` popup; `close_target_w=3` |
| 31 | M | ScrollPanel, TextViewport | `panel.rs:405-650` has no click or drag; `viewport.rs:1332` click always clears | Log panes can't be selected; no Shift-click extend or triple-click line | Back ScrollPanel with TextViewport selection; `on_click_with(mods,count)` |
| 32 | M | PropsList, InfoDialog | `props.rs:376-382` activates on click; `info_dialog.rs:205-212` copies | A click meant to select silently copies to the clipboard | `PropsList::click=Select`; a visible Copy chip on hover |
| 33 | M | TextArea | `textarea.rs:318` is vertical only | Long lines can't be wheeled sideways | `on_wheel(delta, horizontal)` honouring `ctx.hscroll` |
| 34 | L | Small or oversized targets | `tabs.rs:571,592` ×; `grid.rs:1536` →; `choice.rs:89,135,335` | 1-cell × buttons; a checkbox toggles from blank space far right of its label | `close_target_w=3`; `Checkbox::hit=Label` |
| 35 | L | Scrollbar track | `scrollbar.rs:286-300` hard-codes Jump; `ui/ctx.rs:171` | Clicking the track jumps; no paging, no hold-repeat | `TrackClick::Page` on macOS, honoured by `apply`, with repeat |
| 36 | L | Field labels | `input.rs:666`, `textarea.rs:367`, `select.rs:500` register the field row only | Clicking a label doesn't focus its field | `label_hit=true` |
| 37 | L | Terminal pointer and caret shape | No OSC 22 or DECSCUSR code anywhere | No I-beam or resize pointer; a block caret where macOS shows a bar | `PointerConfig::pointer_shapes`, `caret_shape=Bar` |
| 38 | L | ContextMenu | `menu.rs:432-437` keeps hover after the pointer leaves; `:476-495` ▲/▼ are glyphs only | A stale highlight stays outside the menu; long menus scroll by wheel or keys only | Clear hover when the pointer leaves; markers auto-scroll on hover and page on click |
| 39 | L | Read-only CodeEditor, ChoiceDialog | `code.rs:805,1075`; `choice_dialog.rs:155` | Clicking moves a caret the user can't see; double-clicking an option doesn't confirm | Draw a caret when focused; `double_click_confirms=true` |
| 40 | L | Quit and connection switch (showcase, tablepro) | `showcase/app.rs:634`, `tablepro/app.rs:707`; Connections is never re-entered (`tablepro/app.rs:235`) | No mouse way to quit, switch connection or disconnect | App menu with Quit; a connection menu in the status strip |

## 3. Per-widget mouse behaviour (current → target)

| Widget | Hit targets | Hover | Press | Click / 2× / 3× / right-click | Drag | Wheel | Caret on click |
|---|---|---|---|---|---|---|---|
| TextInput | field row (label → add) | Field ladder; off while editing or disabled | yes; the 2nd press selects a word | 1: edit + caret; 2: word; 3: all; right-click: none → Edit menu; Shift-click → extend | select, by word after a double press; edge autoscroll only on motion → tick | n/a | yes, horizontal scroll aware; past the end goes to the end |
| TextArea | body + scroll region; bar registered with no handler → add | yes | yes | as TextInput, 3: line | select | vertical only → add horizontal | yes |
| CodeEditor | text area incl. gutter; both bars handled | none | none → `PointerField` | single click only → 2× and 3×; right-click → Edit menu | broken (#2) | both axes | yes when editable; invisible when read-only |
| Select | field incl. ▾, option rows, popup scroll region | rows tinted, separate from cursor → hover moves cursor | field | click opens and closes; click on an item chooses; backdrop per `Dismiss` | none → press-drag-release | moves the cursor → viewport | n/a |
| Completion | rows; bar inert | rows | n/a | click accepts | none → bar | viewport | n/a |
| Picker | whole modal as one id, rows; bar inert | rows | n/a | click chooses; query not clickable → TextInput | none → bar | viewport | none → caret on click |
| FormDialog | child fields and buttons | children | children | ESCAPE; refused click silent → flash | inputs only | form body even over an open Select → by position | via TextInput |
| Button | label + 2 padding cells | yes; none when disabled | on Down, cancels when dragged off; 140 ms flash | activates on release; right-click app-defined | n/a | n/a | n/a |
| Checkbox / Toggle / Radio | full row (→ label only) / each option row | yes | yes | toggle or select | n/a | n/a | n/a |
| Dialog / InfoDialog / ChoiceDialog | buttons, body, bars, surface = dead space; no × → add | buttons | buttons | outside click per `Dismiss` (LIGHT / ESCAPE); y/n keys; InfoDialog row click copies | scrollbars | body, not routed by position | n/a |
| HelpOverlay | frame, scroll area, bar | none | none | Esc, ?, q only → × / Done | bar (app-wired) | yes | n/a |
| MenuBar | titles, brand, chevron, backdrop | `hover_switch` | title ladder | opens on release → on Down; clicking the open title closes | none → press-drag-release | n/a | n/a |
| ContextMenu | rows, surface, backdrop | moves cursor; disabled rows skipped; stale after leaving | cursor fill | click chooses; shortcuts right-aligned; right-click outside reopens at the new spot | no right-drag release | yes; ▲/▼ not clickable | n/a |
| Footer hints (keyhint, hintbar, keyref) | none → one per hint | none → title ladder | none → yes | none → replays the key | n/a | n/a | n/a |
| ListBox | full row, container, bar | row ladder | yes + key flash | 1: activates → selects, 2: activates; no mods → Shift/Cmd; right-click moves cursor → selection rule | bar only → row drag-select | viewport (cursor kept) | n/a |
| TreeView | row, 2-cell disclosure; deep rows unregistered | row (note rows too → off) | yes | parent row toggles, leaf activates → select + 2×; no Alt-click | bar only | vertical only | n/a |
| DataTable | sortable header, check-all, rows, cells, row checks | row and cell | yes | header sorts; Shift/Cmd in multi; 2nd click edits → 2×; right-click keeps selection | scrollbars; no row drag | vertical rows, horizontal columns | edit caret placed by click |
| DataGrid | header, badges, gutter (with row numbers), cells, bar buttons, 1-cell → | row and cell; editable underlined | yes | header asc/desc/off; gutter toggles → select + mods; 2nd click edits → 2×; right-click keeps anchor (bug) | range drag (no autoscroll), both bars | both axes | caret placed; no word select or drag in a cell |
| Scrollbar | 1-cell track + thumb | thumb lit | held glyph | track Jump → Page on macOS + repeat | thumb grab, both axes | via owner | n/a |
| Splitter | gap strip (→ ±1 pad) | seam ladder | held | none → 2× reset or maximise | `drag_to` (→ grab offset, cell-precise) | n/a | n/a |
| Panel / ScrollPanel | container; cut title hover | none | none | none → selection | bars | vertical + horizontal step | n/a |
| TextViewport / Diff | text + bars | n/a | n/a | press anchors; no Shift or 3× → add; copy is `y` only → Edit menu | select, autoscroll on motion | both axes | browse caret |
| Tabs | tab, 1-cell ×, ‹N / N› (4 cells), + (3 cells) | tab ladder, × separate | yes | activates; middle-click closes (wired in holla only); right-click via ContextSource | none → opt-in reorder | none → both axes | n/a |
| Chips | lead, chip, 1-cell ×, add; `+N` unregistered | chip ladder | yes | toggle or activate; × removes; Toggle and Clear all keys only → menu | none | none | n/a |
| Segments / StatusBar | opt-in clickable segments; cut text gets hover reveal | title ladder | yes | app id | none | none | n/a |
| Steps | rows when selectable | yes | yes | moves cursor | bar | vertical | n/a |
| Stack (layout) | scroll region + track | n/a | n/a | n/a | bar | one child per notch | n/a |
| Columns | none (solver only) | n/a | n/a | n/a | none → resize seams | n/a | n/a |
| Reveal | overlay, no hit | instant → 300 ms delay | n/a | n/a | n/a | scrolls a long tooltip, stalls at its ends | n/a |
| PropsList | rows + bar | row ladder | yes | click activates → selects; Copy chip | bar | yes | n/a |
| EmptyState / Progress | none | none | none | none → `EmptyState::actions` | n/a | n/a | n/a |
| Brand | the lockup | lifts | yes | click gives `Brand` | n/a | n/a | n/a |

**Fix order:** start with the router and #1, which together unlock #14, #17, #18 and much of #22–23. Then #2–3 (text copy and editing), then #9–12 (selection and menu correctness), then the dead ends in #4–8 and #13.

## Status (widget-level mouse pass)

This pass took the widget-level findings, plus the standards review's
corrections: track click pages (macOS "Jump to the next page", Windows);
the hover switch needs any-motion reporting; hover reveal follows WCAG
1.4.13; SGR horizontal wheel buttons 66/67.

| # | Status | Commit(s) | Notes |
|---|---|---|---|
| 11 | Fixed | 4201feae1, c9c4c8475 | The strip's scroll window is separate from the active tab: it follows the active tab only when that tab changes, so ‹/› work at the edges. `Tabs::on_wheel(dx, dy)` scrolls on both axes (tablepro, holla). A `+N` button opens a menu of hidden tabs (`Tabs::overflow_menu`, default true). `Tabs::set_items` keeps the scroll when apps rebuild the strip each frame. |
| 12 | Fixed | 1cee36e98 | `MouseKind::SecondaryDrag` / `SecondaryUp` (and the Ctrl-primary equivalents under `ctrl_click_secondary`). `MenuBar::open_on: OpenOn::{Press, Release}` (default `Press`). `MenuBar::press_drag` (default true): press a title, drag to a row, release chooses; a release on the title keeps the menu open. `ContextMenu::release_selects` (default true): a secondary release on a row after at least 1 cell of movement chooses it. The hover switch acts only once `PointerState::reports_motion()` saw a buttonless move (`Interaction::motion_unreported`); before that, click, press-drag and ←/→ switch menus. Wired in all four apps. A press dragged off and released outside leaves the menu open (non-destructive; macOS closes it). |
| 19 | Fixed | 8e7c4f4f1 | `Select::wheel: WheelMode::{Viewport, Cursor}` (default `Viewport`). `hover_moves_cursor` (default true; one highlight). The popup list has a draggable scrollbar. `Select::press_drag` (default true): press opens, release on an option chooses, release in place keeps it open. Routed in the showcase selectable page, `FormDialog` and jackin forms. Tablepro filter/connection selects and holla's review select still open on click (their routers do not send press/drag/release). |
| 20 | Fixed | c4847ed22 | `Splitter::hit_pad` (default 1), on cells no pane control claims (`HitRegistry::register_under`). The drag keeps the grab offset and moves by exact cells (finer `Split` ratio). Double-click restores `Split::default_percent` (`SeamDoubleClick::{Reset, Ignore}`, default `Reset`). Maximise is not offered: a maximised pane has no seam left to double-click back, a mouse trap. Tablepro gets handles on query editor/results, history list/detail and the explorer width (visible seam rule: intended visual change). jackin Inspect and capsule seams lack double-click (no click count on that path). |
| 25 | Fixed | 0e99f5329 | `Reveal::delay_ms` (default 300) from `Interaction::hover_ms`; keyboard reveal stays immediate; paused motion shows at once. `Reveal::wheel_passes_at_ends` (default true). |
| 26 | Fixed | 961abe9b4 | `ui::tooltip` with description and shortcut (`Glyphs::key_label`). `RenderCtx::tooltips: Tooltips { enabled, delay_ms: 600, max_width: 48 }`. WCAG 1.4.13: Esc hides only the tooltip (handled in the runtime), the tooltip is hoverable and stays until the pointer leaves both, Esc, a press or keyboard use. `Button::tooltip` / `Button::shortcut`, `StatusItem::tooltip`; used on the showcase Buttons page and the tablepro history toolbar. |
| 27 | Fixed | d92c2e6d8 | `RenderCtx::autoscroll` registers the drag area; `PointerState::autoscroll_due(now_ms)` replays the drag every `PointerConfig::autoscroll_ms` (default 50) while held past the edge (`autoscroll`, default true); the step grows with the distance. DataGrid range and row-number drags, TextInput, TextArea, TextViewport. Showcase and tablepro tick at `autoscroll_ms` while autoscrolling; holla and jackin replay on their own tick (simulated clock). DataTable has no row drag-select yet, so it has nothing to autoscroll. |
| 28 | Fixed | 46ed2b0d7, 22acec925, c70ad727d | `ui::columns::ColumnSizing` keeps user widths (`min_width` 3). Header seams (`resizable_columns`, default true; `seam_hit_w`, default 2) resize cell by cell with a grab offset. A double press autofits to the header and visible rows. The header menu has Sort Ascending/Descending, Clear Sort, Size Column to Fit and Reset Column Widths (`HeaderLabels`). Cut headers reveal on hover. Wired in tablepro, showcase Tables (seams and menu) and Grid (seams only). Hide column: deferred, no hidden-column support. User widths are not persisted. |
| 29 | Fixed | a1d762b97 | Deep rows register before the early `continue`; note rows are inert; Alt-click opens or closes a whole subtree (`TreeView::alt_click_recursive`, default true; lazy branches load one level). Tablepro trees still toggle one level: its `Cx` carries no modifiers. |
| 30 | Fixed | 184700eda | Chips `+N` is a button with a menu of hidden chips (`ChipBar::overflow_menu`, default true); `ChipBar::close_target_w` (default 3). Showcase "Chips & selects" baseline re-approved (intended). |
| 33 | Fixed | 66fa25acd | `TextArea::on_wheel_h(delta)` scrolls by `HScroll::step`, also when not editing (`TextArea::horizontal`, default follows `ctx.hscroll`). A separate method, matching `TextViewport`, so `on_wheel` callers keep their signature. |
| 34 | Fixed | 4201feae1, 49ccae0ac | `Tabs::close_target_w` (default 3, inside the tab). `Checkbox::hit: CheckHit::{Label, Row}` (default `Label`: box, label and one cell). |
| 35 | Fixed | 9315b44f1 | `TrackClick::Page` is the default everywhere (`Jump` stays an option, `scrollbar::apply_with`). Hold-repeat: `PointerConfig::track_repeat` (true), `track_repeat_delay_ms` (300), `track_repeat_ms` (50), on the app tick in all four apps; it stops when the thumb reaches the pointer. |
| 36 | Fixed | 66fa25acd | `label_hit` (default true) on TextInput, TextArea and Select: the label is a hit target of the field, not an extra focus stop. Wheel routing in tablepro and showcase offers a notch to each owner under the pointer so a label does not swallow it. |
| 37 | Fixed | 47d7340f4 | `core::shape`: OSC 22 pointer shapes (text, pointer, col/row-resize, default) declared through `RenderCtx::pointer_shape`, sent only on change. `PointerConfig::pointer_shapes` (default true) is gated to kitty, WezTerm, ghostty and foot, never through tmux or screen. `PointerConfig::caret_shape: CaretShape::{Bar, Block, Underline, Terminal}` (default `Bar`) through DECSCUSR on the xterm family and those terminals. Both reset on exit, suspend and panic. The caret shape is sent once at start (the shape has no effect while the cursor is hidden). xterm gets no pointer shapes (X11 names). |
| 38 | Fixed | 1cee36e98 | A context menu clears its highlight when the pointer leaves the rows (Enter does nothing until a row is highlighted again). ▲/▼ are hit targets with rest, hover and pressed looks: hover scrolls a row per tick, a click pages. |
| SGR 66/67 | Fixed | 1cee36e98 | crossterm 0.29 decodes buttons 66/67 as ScrollLeft/Right; they map to `WheelLeft`/`WheelRight` with or without Shift. Regression test guards the mapping (it already passed; the raw parser is private to crossterm). |

Open from this pass:

- When a Tabs or Chips `+N` menu is open, the page underneath still shows its
  focus indicator (one-focus rule). Apps need to call `page_under_layer` for
  these menus; none does yet.
- jackin's router does not send the wheel to its tab strips.
- The I-beam also shows over a field's label (#36 registers the label under
  the field id); TextViewport and diff text declare no I-beam yet.
- Visual: tablepro now draws seam rules between split panes (#20) and an
  overflowing tab strip shows `+N` (#11). Tuiscotti scenarios covering those
  screens re-approve with these changes; they were not re-approved here.
