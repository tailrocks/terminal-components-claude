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
   `render_vertical`: track `┄` (`─` before the 2026-10-16 decision
   below), thumb `━`, proportional thumb, same theme
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
  would make the two axes configurable together. (Done: `7338c7ecf`,
  `200e688ad`.)
- Holla and jackin screens get the sideways wheel through the pointer
  refactor's `on_wheel_h`; none of their screens has wide content yet.
  (Done for the capsule panes and the Files preview, with app tests in
  `12e442e71`.)

## Status: TextViewport and vertical track paging

The `TextViewport` and `DiffView` follow-ups and the vertical
`TrackClick::Page` follow-up are done.

| Surface | Commit |
|---|---|
| `TextViewport` (+ `DiffView` delegation) | `6f075829a` feat(viewport) |
| Showcase Terminal/Diff, jackin capsule panes and Inspect diff, holla Files preview | `b68c2210e` feat(apps) |
| Vertical `TrackClick::Page` (`ScrollPanel`, `ListBox`, `TreeView`, `DataTable`, `DataGrid`, `CodeEditor`, `TextViewport`) | `7338c7ecf` feat(scroll) |

How `TextViewport` handles the horizontal offset:

- Mouse: `pos_at` adds the offset, so click, drag and double-click
  select the content under the pointer. A drag past the left or right
  edge scrolls sideways one step and keeps selecting.
- Painting: selection and marks are drawn at their scrolled columns. A
  wide cell cut by the left edge leaves its visible half blank.
- Search: `reveal_line` brings the line's current mark into view
  sideways, so the existing holla search jumps work without changes.
  `reveal_mark(line, bytes)` does the same for any range. A Shift+arrow
  selection head stays in view.
- Follow (tail) mode: following is vertical only. Scrolling sideways
  never pauses it, and new lines keep the horizontal offset.
- Caret: the producer caret maps through both offsets (`screen_pos`).
  While it is scrolled out of view it is hidden, never parked at an
  edge. The jackin capsule places its re-drawn caret through
  `screen_pos`.
- Wrapping: `wrap(true)` turns the axis off (no bar, no reserved row).
  The holla activity and plan viewports and the jackin cockpit log
  wrap, so they have no horizontal axis.
- Input: sideways wheel and Shift+wheel go to `on_wheel_h`. Plain ←/→
  and h/l scroll sideways only while no producer caret is shown; the
  jackin capsule maps ←/→ that way while scrolled back. Home/End stay
  vertical, as on `ScrollPanel`. Bar press and drag go through
  `on_scrollbar` and `on_bar_drag`, and `owns()` covers both bar ids.
  `hbar()` exposes the bar row for owners that render a copy of the
  viewport (jackin panes).
- jackin `CustomModal::on_wheel_h` lets a modal's content take the
  sideways wheel. Dialogs take it too.

Knobs added:

| Name | Type | Default |
|---|---|---|
| `TextViewport::horizontal` | `Option<HScroll>` | `None` (app default) |
| `RenderCtx::track_click` | `TrackClick` (vertical bars, app-wide) | `Jump` |
| widget `track_click` (`TextViewport`, `ScrollPanel`, `ListBox`, `TreeView`, `DataTable`, `DataGrid`, `CodeEditor`) | `Option<TrackClick>` | `None` (app default) |

## Status: remaining track paging and app-level proof

| Item | Commit |
|---|---|
| `TrackClick::Page` on the vertical bars of `Dialog` (scrolling body), `StepRail`, `PropsList` and `ui::layout::Stack` | `200e688ad` feat(scroll) |
| App test: jackin capsule pane pans a wide line (sideways wheel, ←/→ while scrolled back, a press on its bar routed through `hbar()`) | `12e442e71` test(apps) |
| App test: holla Files preview pans `long.txt` (sideways wheel, a track press; the thumb moves in the rendered cells) and a newly opened file starts at its first column | `12e442e71` test(apps) |

Knobs added:

| Name | Type | Default |
|---|---|---|
| `Dialog::track_click`, `StepRail::track_click`, `PropsList::track_click` (field; builder on `StepRail` and `PropsList`) | `Option<TrackClick>` | `None` (follows `RenderCtx::track_click`) |
| `Stack::track_click` (field and builder) | `TrackClick` | `Jump`. `Stack::layout` has no `RenderCtx`, so an owner that follows the app-wide policy passes `ctx.track_click`. |

Every vertical bar in the library now takes the track-click policy.

Still open:

- Showcase and holla pages still route bars by hand
  (`scrollbar::is_bar_of`). This waits for the shared pointer router.
- jackin capsule pane bars are routed by position (`hbar()`), because
  the pane draws a copy of the viewport inside an inert region.

## Decision: the track is a dotted rail (2026-10-16)

**Request (product owner).** The scrollbar track must not look like a
panel border. The vertical track was `│` in `border_subtle`, the same
glyph and ink as every panel border and divider; the horizontal track
was `─`, a border's top edge. Option A was chosen: a faint dotted track,
the same on both axes.

**Standards.** A macOS scroller track never reads as a frame: it is a
faint rail that only exists to carry the knob. WCAG 1.4.1 (use of
colour): shape, not colour, has to tell a track from a border, because at
16 colours both are `DarkGray` and under `NO_COLOR` there is no colour
at all. A dotted rail does that at every level.

**What changed.**

| | Before | After |
|---|---|---|
| Vertical track, Unicode | `│` U+2502 | `┆` U+2506 light triple dash vertical |
| Horizontal track, Unicode | `─` U+2500 | `┄` U+2504 light triple dash horizontal |
| Thumbs, Unicode | `┃` / `━` | unchanged |
| Vertical track, ASCII | `\|` | `:` |
| Horizontal track, ASCII | `-` | `.` |
| Thumbs, ASCII | `#` / `=` | `#` / `#` (one thumb glyph on both axes) |
| Track ink | `border_subtle` | `Theme::scroll_track` (new token) |

- `┆`/`┄` are a matched pair (same dash count, same weight) from the
  box-drawing block, the same East Asian Width class (ambiguous) as the
  `│`/`┃` they replace, so they are no new width risk; `unicode-width`
  gives 1 and the harness renders them in one cell. `┊`/`┈` (quadruple
  dash) were not used: they carry less ink per cell than the triple dash,
  so in the already faint track ink they fade out on the card plane
  first; the triple dash keeps a visible rail.
- ASCII `:` and `.` are the dotted counterparts of the `|` and `-` that
  ASCII borders and dividers use, so the ASCII track reads as a rail, not
  a frame. `=` stays the progress fill (`bar_fill`); the thumb is `#` on
  both axes.
- Track ink, fainter than the thumb at rest at every level and apart
  from both border inks where the level has the steps (contrast against
  the canvas, WCAG relative luminance; the track is decoration, the
  thumb carries position and state):

| Level | Track | Thumb at rest | Border subtle / strong | Track : canvas | Thumb : canvas | Thumb : track |
|---|---|---|---|---|---|---|
| truecolor | `#474747` (`WHITE_28`) | `#808080` | `#3a3a3a` / `#4d4d4d` | 2.3:1 (1.9:1 on a card) | 5.3:1 | 2.4:1 |
| 256 | 239 `#4e4e4e` | 245 `#8a8a8a` | 238 / 242 | 2.5:1 | 6.1:1 | 2.4:1 |
| 16 | `DarkGray` | `Gray` | `DarkGray` / `Gray` | 2.8:1 (VGA palette) | 9.0:1 | 3.2:1 |
| mono | glyph only (`┆` vs `┃`) | glyph only | glyph only | n/a | n/a | n/a |

  At 16 colours the track shares `DarkGray` with `border_subtle` (there
  is no grey between `DarkGray` and the thumb's `Gray`); the dotted glyph
  carries the difference, as it does in mono.
- Thumb rest / hover / pressed are unchanged (`Ladders::handle`, one
  glyph, modifiers in mono).

**Dividers get their own glyphs.** `Glyphs::scroll_track` /
`scroll_h_track` used to double as the divider line. That is now
`Glyphs::divider` (`│`, ASCII `|`) and `Glyphs::h_divider` (`─`, ASCII
`-`), used by the splitter seam at rest, the column-resize seam, the
side-by-side diff separator, the segmented option editor and the
jackin Inspect stacked split (which drew a literal `─`). A divider stays
solid; only a scrollbar is dotted.

**Divider that is also a scrollbar** (`scrollbar::render_divider`, the
`TreeView::divider` pane edge and the showcase Sidebars nav). While the
content overflows it *is* a scrollbar: dotted track, thumb and click
region, the same as every other vertical bar. While the content fits it
is a plain divider: solid `│` in `border_subtle`, no click region.
Rationale: the line is only an affordance when there is something to
scroll (macOS shows no scroller then), and the shape should say which it
is. The horizontal axis has no divider-scrollbar today; one would follow
the same rule with `┄` / `─`.

**Every drawing site.** All tracks go through `scrollbar::render_vertical`,
`render_horizontal(_with)` or `render_divider`, which read
`Theme::glyphs` and `Theme::scrollbar_track()`: `ListBox`, `TreeView`,
`DataTable`, `DataGrid` (both axes), `ScrollPanel` (both axes),
`CodeEditor` (both axes), `TextViewport`/`DiffView` (both axes),
`TextArea`, `Dialog` body and code preview, `FormDialog`, `InfoDialog`,
`Help`, `Picker`, `Completion`, `Select` popup, `StepRail`, `PropsList`,
`ui::layout::Stack`, and the holla and jackin screens that call them. No
widget, page or app draws its own track (searched for `scroll_track`,
`scroll_h_track`, literal `│`/`─`/`┃`/`━` writes). The corner cell where
both bars meet stays blank (`scrollbar::Bars`). Menus and the anchored
select use `▴`/`▾` more-markers, not tracks.

**Overrides.** The track glyphs are `Glyphs` fields, so an app picks:
dotted (default), solid (`glyphs.scroll_track = glyphs.divider;
glyphs.scroll_h_track = glyphs.h_divider`, option C), or hidden
(`" "`, option B: the thumb alone, like macOS overlay scrollers). The
ink is the `Theme::scroll_track` token (and `LevelOverrides` per level).
No `ScrollTrack` enum was added: the glyph fields already are the
override and an enum would be a second way to say the same thing.

**Proof.** `scrollbar::tests::both_tracks_are_one_dotted_pair_in_the_track_ink_never_a_border`
(every level, Unicode and ASCII, both axes, thumb at rest unchanged,
track fainter than the thumb), `the_glyph_sets_pair_their_tracks_and_thumbs`,
`a_divider_is_solid_while_it_fits_and_a_dotted_scrollbar_once_it_overflows`,
and the showcase `the_data_grid_draws_both_tracks_dotted_in_one_frame`
(both axes in one frame, every level, both glyph sets). Intended visual
change: `tests/showcase_baseline.txt` regenerated; tuiscotti showcase
captures re-approved in snapshot-only commits after a cell-by-cell
attribution (every changed cell is a track cell: `│`→`┆`, `─`→`┄`, and
at 256/truecolor the ink `238`→`239` / `#3a3a3a`→`#474747`).
