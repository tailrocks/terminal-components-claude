# Horizontal scrolling and position

Product request: content wider than its area scrolls sideways, and the
horizontal position is visible. Example: tablepro `public › customers`
(11 columns) shows `cols 1–5 of 11` in the status line and a `6›` badge,
UUID cells are cut, and only the vertical `┃` bar exists.

Line numbers refer to the code when this audit was written.

## Inventory

"vbar" is `scrollbar::render_vertical` on the area's right column. There
was no horizontal bar anywhere, and no widget handled shift+wheel.

| Surface | Overflow today | H position shown | H inputs | Decision |
|---|---|---|---|---|
| `DataGrid` (`grid.rs`; tablepro table/query results, showcase grid) | column paging on `hscroll: ScrollState` (`layout_columns`), cells cut with `…`/`truncate_middle`, reveal on hover/`→` | `‹n`/`n›` header badges, `cols a–b of n` label | ←/→ move cell (scrolls), Ctrl+←/→ page, wheel left/right one column, badge clicks | **Horizontal bar** on the bottom row of the body, column units (snap). Badges and label kept, both configurable. |
| `DataTable` (`table.rs`; tablepro structure, showcase tables/editable) | column paging on `hscroll`, `…` header edge marks | `…` edge marks | ←/→ page columns (cell nav when `cell_nav`), `on_wheel_h` (showcase only) | **Horizontal bar**, column units; `on_wheel_h` one column per notch. `…` marks kept (`overflow_marks`). |
| `ScrollPanel` (`panel.rs`; tablepro DDL, EXPLAIN raw, history detail, showcase panels/scrolling) | `wrap=false` lines hard-clipped with `…` | none | none | **Two-axis**: `hscroll: ScrollState` by cell when not wrapping, bar on the bottom row, corner cell blank, ←/→ scroll (no caret). |
| `TextViewport` (`viewport.rs`; terminal, logs, unified diff, holla preview/plan/activity) | `wrap=false` lines clipped silently | none | ←/→ move caret only | **Recorded, not this pass**: selection/caret mapping (`pos_at`, `CellPos`) is cell-exact and wide; a horizontal offset touches selection, marks and the follow logic. Follow-up below. |
| `CodeEditor` (`code.rs`; tablepro editor, showcase editor) | own `hscroll: usize`, `…` edge marks, caret keeps 4-col margin | none | ←/→ ±8, wheel ±4, click maps through | **Horizontal bar** on the bottom row (it already scrolls). Clamp to the widest line. |
| `TextArea` (`textarea.rs`) | `hscroll` follows the cursor, `…` edges | none | cursor keys | No bar: single-purpose input; the caret is the position. |
| `Dialog` bodies (`dialog.rs`) | text wraps; facts truncate | none | ←/→ move between buttons | Text wraps and facts reveal. The **code preview** (SQL, paths) never wraps: it scrolls sideways under its own bar (Shift+←/→, sideways wheel, bar), plain ←/→ keep choosing the action. |
| `PropsList` / `props::render` | value `…` or `.wrap()`, `RevealState` (`→`) | none | `→` reveal | No hscroll: one value per row, reveal shows the full value. |
| `List`, `Picker`, `Completion`, `Steps` | labels `…` | none | none | No hscroll: one item per row; reveal is the tool. |
| `Tree` (`tree.rs`; EXPLAIN plan, explorer, holla disk) | labels `fit`, meta dropped | none | ←/→ collapse/expand | No hscroll (←/→ are taken by the tree); deep plans use the raw `ScrollPanel` tab, which gains hscroll. Reveal for tree labels is a follow-up. |
| `Tabs` strip | pages by `first`, `‹N`/`N›` badges | badges | ←/→ change tab | Already pages; a bar would compete with the strip. |
| `KeyHint`, `StatusBar`, `Menu`, `Select` | trailing `…`, reveal | `…` | n/a | No hscroll: chrome rows, not content. |
| tablepro EXPLAIN tab | `Tree` + raw `ScrollPanel` | none | wheel-left/right reached vertical `on_wheel` | Raw view gets the two-axis panel; horizontal wheel no longer scrolls vertically. |

## Input findings

- tablepro `workbench.rs`/`tabs.rs`, holla `app.rs` and jackin `app.rs` fed
  wheel-left/right into vertical `on_wheel` (a sideways swipe scrolled the
  list up/down). Horizontal wheel now reaches only horizontal scrolling.
- Shift+wheel: crossterm reports `ScrollUp/Down` with `SHIFT` in most
  terminals. `Input::from_crossterm` normalises it to `WheelLeft/Right`
  (one place, every app), configurable through `InputConfig`.

## Design decisions

1. **One model.** `ScrollState` already is axis-free (offset, content,
   viewport, grab). The horizontal axis is a second `ScrollState`; no new
   state type. Column widgets count columns, free text counts cells.
2. **One bar widget.** `scrollbar::render_horizontal` mirrors
   `render_vertical`: track `─`, thumb `━`, proportional thumb, same theme
   styles (`scrollbar_track`, `scrollbar_thumb(focused, hovered)`), so
   rest, hover and pressed stay as distinct as on the vertical bar. Its
   hit id is `scrollbar::h_id_for(container)`, separate from the vertical
   id, so hovering one bar never lights the other.
3. **Placement.** The bottom row of the scrolled region, like the
   vertical bar uses the right column. When both show, the horizontal
   track stops one cell short and the corner cell stays blank
   (`scrollbar::Bars`).
4. **Snap vs cells.** Grids and tables scroll by column (a half column is
   unreadable data) and the bar reflects the column position: offset =
   first visible column, viewport = columns that fit. Free text scrolls by
   cell.
5. **Track click.** Matches the vertical bars (thumb jumps under the
   pointer) so both axes agree; `TrackClick::Page` gives the macOS "next
   page" behaviour.
6. **Badges and `cols a–b of n`.** Kept on by default. The bar shows
   *where*, the badges show *how many* columns are hidden and are click
   targets, and the label is the exact position for keyboard and screen
   reader users. Both are switchable (`overflow_badges`).
7. **Cut cells (UUIDs).** No per-cell widening: the reveal system
   (`ui::reveal`, hover or `→`) already shows the full value of a cut cell
   without shifting the layout. Horizontal scrolling moves columns into
   view; reveal shows a cell's full text.
8. **Visibility.** `ScrollbarVisibility::{Auto, Always, Never}`; `Auto`
   (macOS default) draws only while content overflows.

## Status

| Surface | Commit |
|---|---|
| Bar, `HScroll`, Shift+wheel | `d11ab0792` feat(scroll) |
| `DataGrid` | `76ccdab11` feat(grid), `72a319a98` test(tablepro) |
| `DataTable` + tablepro wheel routing | `f8203570d` feat(table) |
| `ScrollPanel` + app routing, holla/jackin wheel | `5edf2bba8` feat(panel) |
| `CodeEditor` | `7e9bc5277` feat(code) |
| `Dialog` code preview | `0957f37f9` feat(dialog), `e44df5ead` fix(holla,jackin) |
| Drags without a routed press | `74d704b06` fix(scroll) |

Every widget with a horizontal bar exposes `owns(id)` (both bar ids)
and `on_bar_drag(pressed, pos)`; pages route `scrollbar::is_bar_of`.

## Knobs

| Name | Type | Default |
|---|---|---|
| `HScroll::enabled` | `bool` | `true` |
| `HScroll::bar` | `ScrollbarVisibility` | `Auto` |
| `HScroll::step` | `usize` (cells per notch/arrow, free text) | `4` |
| `HScroll::track_click` | `TrackClick` | `Jump` |
| `RenderCtx::hscroll` | `HScroll` (app-wide default) | `HScroll::default()` |
| widget `horizontal` | `Option<HScroll>` (override) | `None` |
| `DataGrid::overflow_badges` | `bool` | `true` |
| `DataTable::overflow_marks` | `bool` | `true` |
| `DataGrid`/`DataTable`/`ScrollPanel`/`CodeEditor`/`Dialog::horizontal` | `Option<HScroll>` | `None` (app default) |
| `InputConfig::shift_wheel_horizontal` | `bool` | `true` |

## Plan

1. Audit (this file).
2. `ScrollbarVisibility`, `TrackClick`, `HScroll` in `core/scroll.rs`;
   `render_horizontal`, `h_id_for`, `press_h`/`drag_h`, `Bars` in
   `widgets/scrollbar.rs`; `RenderCtx::hscroll`.
3. Shift+wheel normalisation in `core/event.rs`.
4. `DataGrid` and `DataTable`: bottom bar, owned ids, click/drag.
5. `ScrollPanel` two-axis; `CodeView` bar and clamp.
6. Apps: route horizontal wheel only to horizontal scrolling.

## Follow-ups

- `TextViewport` (`wrap=false`) horizontal offset: needs `pos_at`,
  selection painting and marks to account for the offset.
- `DiffView` unified mode inherits it from `TextViewport`.
- Reveal for `Tree`, `List`, `Picker`, `DataTable` cut labels.
- Shared pointer router (interaction refactor, `core/pointer.rs`): it
  landed during this work with `PointerConfig::shift_wheel_horizontal`,
  which nothing reads, because modifiers are dropped before the pointer
  sees the event. The normalisation lives in
  `Input::from_crossterm_with(InputConfig)`; the two knobs should become
  one (the runtime would pass the pointer's setting through).
- Per-app bar routing (`scrollbar::is_bar_of`, `on_bar_drag`) should move
  into the shared router so a page never routes a scrollbar by hand.
- Vertical bars keep `TrackClick::Jump`; offering `Page` on them too
  would make the two axes configurable together.
- Holla and jackin screens get the sideways wheel through the pointer
  refactor's `on_wheel_h`; none of their screens has wide content yet.
