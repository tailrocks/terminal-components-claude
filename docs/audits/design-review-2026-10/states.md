**1. Proposed global state model**

**Rules (one resolver, every widget)**

`Theme::interactive(kind: Kind, s: VisualState, plane: Plane) -> Interactive { style, gutter, gutter_style, on_fill }`, where:
- `Plane` = Canvas | Surface | Card | Field | Raised (secondary button, chip, status chip) | Popover | Highlight (popup or menu cursor) | Accent | Danger.
- `Kind` = Row | CursorRow(CursorStyle) | Button(ButtonKind) | Tab | Title (menu-bar title, header link, overflow badge, `+`) | Chip | Close (× glyph) | Field | Handle (scrollbar thumb, splitter).

Apply the states in this order:
1. **disabled.** Plane bg, `disabled` fg, plus DIM in Mono. Never `lift()`. Force hover and pressed off inside the resolver, not in each widget.
2. **pressed.** The plane's `pressed` token plus BOLD. The same rule holds everywhere: there is no white flood, no red flood, no inverted cell.
3. **hover.** The plane's `hover` token. It composes with selected: a selected row uses `accent_bg_hover` and keeps its tint.
4. **selected.** A marker glyph (`›` `✓` `●`) in accent. An optional `accent_bg` tint, only on the focused list.
5. **focused.** Orthogonal to the others: the `▎` gutter plus BOLD, and nothing else. Popup and menu lists use `CursorRow(Fill)` = the `highlight` plane with no `▎`.

Replace `lift()` with an explicit ladder: `struct Ladder { rest, hover, pressed }` per plane, with `Theme::ladder(Plane)`. An unknown bg should be a `debug_assert!`, not a silent fallback to `popover`.

**Ladder tokens (TrueColor / Ansi256 hand-picked index; do not use `nearest_256` for these)**

Each cell is rest → hover → pressed.

| plane | TrueColor | 256 |
|---|---|---|
| canvas | `#000000` → `#1c1c1f` → `#303036` | 16 → 234 → 236 |
| surface | `#111111` → `#26262a` → `#38383e` | 232 → 235 → 237 |
| card | `#18181b` → `#2a2a2f` → `#3c3c43` | 233 → 235 → 237 |
| field | `#1e1e22` → `#2f2f34` → `#3a3a40` | 234 → 236 → 238 |
| raised | `#27272a` → `#37373d` → `#4a4a52` | 235 → 237 → 239 |
| popover | `#3f3f46` → `#4f4f58` → `#63636d` | 237 → 239 → 241 |
| highlight | `#2f5aa8` → `#3a6bc4` → `#244a8c` | 25 → 68 → 24 |
| accent | `#48e054` → `#3ab343` → `#2e9a38` | 78 → 71 → 65 |
| danger | `#e44545` → `#ec6363` → `#b83232` | 167 → 174 → 131 |
| highlight_danger | `#7a2a2a` → — → `#5a1f1f` | 131 → — → 95 |

- Adjacent steps measure 1.20–1.36:1 at TrueColor and 1.13–1.37:1 at 256. The rest→hover steps on field (1.13 at 256) and raised (1.15 at 256) still sit at or below the 1.15 bar; nudge those two hover values up one grey before shipping.
- Text on the accent pressed fill: on-accent ink on `#2e9a38` is 4.84:1. At 256, on-accent ink 233 on index 65 is 4.57:1.
- Selected tint `accent_bg`: keep `#0f2e13` at TrueColor, use index 22 at 256 so the hue survives, and use the plane itself (no tint) at 16 and Mono.

**Ansi16 strategy** (4 greys: canvas, surface, card and field all map to Black; raised and popover map to DarkGray):
- hover on a Black plane: DarkGray bg.
- hover on a DarkGray plane: keep the bg and add UNDERLINED on the label span.
- pressed: always REVERSED + BOLD on top of the rest style, including the accent and danger fills.
- cursor fill: Blue bg, White fg, BOLD.
- selected: marker glyph in LightGreen, no tint.
- disabled: DarkGray fg on the plane, never on a DarkGray plane.

**Mono (`NO_COLOR` strips every colour; only modifiers and glyphs survive)**

| state | rendering |
|---|---|
| rest | nothing |
| hover | UNDERLINED |
| focused | `▎` + BOLD |
| selected | marker glyph |
| cursor (Fill) | REVERSED |
| pressed | REVERSED + BOLD; on a row already REVERSED (cursor), add UNDERLINED |
| disabled | DIM, and no hover or press |

Filled controls (primary, brand) in Mono: rest = REVERSED + BOLD, hover = + UNDERLINED, pressed = drop REVERSED and keep BOLD + UNDERLINED (a 140 ms flash).

**Plumbing (needed before any theme fix shows)**
- Add `Interaction::pressed_any(&[ids])` and `flash_within(container)`. Rows must count a press or flash on their cell, toggle or × ids, and on the container's focus id for the cursor row, just as they already count hover.

**2. Findings**

Severity | file:line | widget | states that collide (resolved colours) | fix

1. P0 | theme.rs:510-516, 190-191 | Primary button, brand | At 256, rest/hover = 78/77, 1.02:1, identical. Pressed `#2b8632` falls to grey 238, so the green is lost and on-accent ink sits at 1.92:1, unreadable. At 16, hover and pressed are both Green. In Mono all three are BOLD only. | Use the accent ladder 78/71/65 at 256, REVERSED for pressed at 16 and Mono, and pressed `#2e9a38` at TrueColor.
2. P0 | theme.rs:704-719 | `nearest_256` | Dark saturated tokens snap to greys: `accent_pressed`→238, `highlight_danger`→238, `accent_bg`→233 (= card), `error_bg`→233. | Prefer the colour cube when max-min > 30, and give interactive tokens explicit 256 indices in `for_level`.
3. P0 | menu.rs:333-339 | Destructive menu cursor row | `#7a2a2a` on `#3f3f46` is 1.09:1. At 256 it is 238 vs 237 (grey on grey), so the cursor is invisible. In Mono both are DarkGray and only BOLD remains. | Use `highlight_danger` index 131 at 256; Red bg at 16; REVERSED in Mono.
4. P0 | theme.rs:501-507 | Disabled button (Secondary/Primary/Danger) | bg = `lift(card)` = overlay `#27272a`, which is DarkGray at 16, and the label W30 is also DarkGray: the label is invisible at 16. On canvas the disabled bg is card while the rest bg is raised, so the plane is inconsistent. | Disabled = the kind's rest plane with `disabled` fg. At 16 keep the plane Black or the fg Gray.
5. P0 | chips.rs:182-189 | Filter chip (off) | `Toggle` and `Secondary` resolve identically. The off chip is W30 fg on overlay: DarkGray on DarkGray at 16, so the label is invisible. `s.selected` is ignored. | Chip kind with the raised ladder. Off = muted fg; on = accent marker plus tint. Do not use `text_faint` on DarkGray.
6. P0 | runtime.rs:371-381 + theme.rs:530-558 | All buttons under `NO_COLOR` | Secondary, Subtle, Danger: pressed is `fg canvas`/`bg white` with no modifier, which equals rest after the colour reset. Hover equals rest. | Every pressed style carries REVERSED + BOLD in Mono; hover adds UNDERLINED.
7. P1 | theme.rs:530-532, 543-545, 556-558 | Button pressed | Pressed is a white flood (Secondary, Subtle, Toggle, chips), a red flood (Danger), a darker fill (Primary), or a popover plane plus BOLD (rows). Four different pressed models. | Pressed = the plane ladder's pressed token plus BOLD everywhere: secondary `#4a4a52`, subtle `ladder(bg).pressed`, danger `#b83232`.
8. P1 | theme.rs:465-475 | `lift()` on popover | `lift(popover) = popover`, so on popover hover = rest and pressed = hover = rest (only BOLD differs). `lift(raised) = popover = row pressed`, so hover = pressed on raised. | Replace with `ladder(plane)`: popover hover `#4f4f58`, pressed `#63636d`.
9. P1 | theme.rs:386-388 | Row: hover on a selected+focused row | Hover overwrites `accent_bg` with `lift(bg)`, so the selection tint disappears under the pointer. | Add an `accent_bg_hover` token (TrueColor `#163d1b`, 256 index 28).
10. P1 | theme.rs:375-411 (canvas plane) | List, tree, table, grid rows | Rest/hover: `#000`/`#18181b` = 1.19 at TrueColor, 16/233 = 1.12 at 256, Black/Black at 16 and Mono. Hover and selected+focused both map to 233 at 256; at 16 selected+focused and pressed are both DarkGray + BOLD. | Canvas ladder `#1c1c1f`/`#303036` (234/236). Pressed at 16 = REVERSED. No `accent_bg` at 16.
11. P1 | theme.rs:375-411 (card plane) | Select options, picker, completion, dialog lists | Rest/selected+focused: 233/233 (1.00) at 256. Hover overlay vs pressed popover is DarkGray/DarkGray at 16, so only BOLD separates them. | Card ladder `#2a2a2f`/`#3c3c43` (235/237); `accent_bg` index 22 at 256.
12. P1 | theme.rs:418-454 | `cursor_row` Fill, Mono | The cursor is REVERSED + BOLD and pressed is REVERSED + BOLD (`on_fill` is suppressed when pressed): identical. | Mono cursor = REVERSED; pressed adds BOLD + UNDERLINED.
13. P1 | select.rs:376-386, picker.rs:566-576, completion.rs:213-223 | Popup lists | Use `row()` with `focused = cursor`, so a `▎` bar appears on the cursor row while the owner field also shows `▎`: two focus indicators. | Switch to `cursor_row(.., CursorStyle::Fill)` (the highlight plane).
14. P1 | menu.rs:328-361 | Menu rows | No pressed state (hover moves the cursor, so hover = cursor = pressed). Mono cursor is BOLD only. | Pressed = highlight pressed `#244a8c` (256 index 24); Mono cursor REVERSED, pressed + BOLD + UNDERLINED. Reuse `cursor_row`.
15. P1 | table.rs:714-723, 757; grid.rs:1756-1766 | Table and grid rows | Clicks land on cell ids, which are registered on top, and the flash carries the cell id. `s.pressed` is read only from `row_id`/`rownum_id`, so rows never show pressed. | Count a press or flash on any cell id as a row press, as hover already does.
16. P1 | tablepro/app.rs:546-552 (same in showcase/app.rs:577, holla/app.rs:633, jackin app.rs:647) | Keyboard activation flash | The flash uses the container focus id. List, tree, table, grid, radio, chip, tab and menu rows use child ids, so Enter or Space gives no pressed feedback. | `Interaction::flash_within(container)`, and widgets apply it to the cursor child.
17. P1 | tabs.rs:363-381 | Tabs | No pressed state. Hover = cursor (only BOLD differs). The active tab's plane does not change on hover. With the cursor on the active tab, a focused bar equals an unfocused bar (active is already BOLD), and no `▎` is drawn. | Use the ladder: active tab bg = `ladder(bg).hover` in a selected role, hover +1 step, pressed +2 steps; focus = `▎` gutter.
18. P1 | tabs.rs:425-433 | Tab close × | On a hovered or focused tab (popover plane) the × hover bg is `lift(popover)` = popover, i.e. no bg change. No pressed state. | `Kind::Close`: hover = next step plus primary fg, pressed = REVERSED at 16 and Mono.
19. P1 | chips.rs:179-181, 209-215 | Chip × | Hovering × lifts the whole chip, so × hover vs chip hover differ only by a BOLD glyph. × has no pressed state and is rendered muted on the white flood. | Hover on × must not set the chip's hover; give × the Close kind.
20. P1 | theme.rs:565-575 | Input, textarea, select field | Hover vs rest: `#1e1e22`/`#232328` = 1.06 at TrueColor, 234/234 at 256, Black/Black in Mono. No pressed state. | Field ladder `#2f2f34`/`#3a3a40` (236/238); Mono hover UNDERLINED on the placeholder.
21. P1 | theme.rs:599-607, scrollbar.rs:32 | Scrollbar thumb | Pressed = hover (`W70`). At 16 rest/hover are Gray/Gray; Mono Gray/Gray. Focus overrides hover, so focus+hover = focus. | rest W50 `░▐`, hover W70, pressed white plus a heavy glyph `█`; at 16 rest DarkGray, hover Gray, pressed White.
22. P2 | splitter.rs:40-47; capsule.rs:1622-1635 | Splitter, capsule seams | Rest/hover W15/W30 are DarkGray/DarkGray at 16; in Mono W15 is Black, so the rest splitter is invisible on canvas. Hover equals pressed apart from the heavy glyph. The capsule draws its own seams, so they are inconsistent. | Rest W30, hover W50 plus the heavy glyph, pressed white plus the heavy glyph. Capsule should use `Splitter`.
23. P2 | statusbar.rs:284-293; segments.rs:103-107 | Status items and segments | Hover only, no pressed. A chip on overlay hovers to popover, which equals the row pressed colour. | Title or Chip kind via the resolver.
24. P2 | menu.rs:632-655 | Menu-bar titles | No pressed state. Hover = `lift(bg)` and open = popover (which equals hover when bg is overlay). | Title kind: hover +1, pressed and open +2 (popover pressed when bg is popover).
25. P2 | grid.rs:1612-1616, 1633-1645, 1684-1688; tabs.rs:328-343, 463-492 | Overflow badges ‹n n›, sortable headers, tab `+` | Hover only (lift, or an underline for headers). No pressed state; Mono hover invisible except the header underline. | Title kind.
26. P2 | grid.rs:1879-1881; theme.rs:586-593 | Grid range and text selection | Range bg popover equals pressed-row bg and `selection()`, so a pressed row and a range are indistinguishable. | Selection tint = `accent_bg_subtle`-blue `#1f3460` (256 index 17), not popover.
27. P2 | table.rs:789-797; grid.rs:1897-1907; viewport.rs:1453 | Cell cursor, reversed terminal cells | White flood `fg canvas`/`bg white` reuses the old pressed language. | Cell cursor = highlight plane plus BOLD (Mono REVERSED).
28. P2 | brand.rs:76-81 | Brand lockup | Same collisions as finding 1. The brand renders no `▎` focus. | Route it through `Button(Primary)`.
29. P2 | steps.rs:235-239 | Non-selectable steps | Hover is zeroed but pressed still fires (popover + BOLD) on an inert row. | Zero pressed as well, or move the disabled-like gate into the resolver.
30. P2 | showcase/app.rs:1005-1010; holla finder.rs:758-766, 911-920; jackin cockpit.rs:937, 941-946, 953-962 | App-rolled clickables | Showcase Help/Inspector hover = `#111111` on `#000` (1.11 at TrueColor; Black/Black at 16), and no pressed. Jackin `CHIP_RUN` and `CHIP_ACTIVITY` have no hover or press. Finder scope and explore cells have hover only. | All of these go through `Theme::interactive(Kind::Title or Chip, ..)`.

**Notes**
- `git diff` was empty during the audit; `cursor_row` already exists in `src/theme.rs` but has no callers.
- The contrast and downgrade scripts are in `/private/tmp/claude-501/-Users-donbeave-Projects-terminal-components-claude/42325d29-0331-4e82-a42c-409070ca54fc/scratchpad/audit-states/` (`dg.py`, `m.py`, `p.py`, `q.py`, `r.py`).
## Status (T19 sync, 2026-10-17)

Each finding was checked against the code and tests at `origin/design/interaction-polish` and matched to the commit that fixed it (most commit messages cite the finding numbers).

| # | Status | Commit(s) / owner | Notes |
|---|---|---|---|
| 1 | Fixed | e40c2c08f, 189734f49 | Accent ladder 78/71/65 at 256, pressed `#2e9a38`, reversed press at 16 and mono. Test `reduced_levels_pin_state_tokens_instead_of_snapping_to_greys`. |
| 2 | Fixed | 189734f49, e40c2c08f | `nearest_256` keeps saturated colours in the cube; `for_level` applies hand-picked 256/16/mono tables (`LevelOverrides`). Same test as #1. |
| 3 | Fixed | e40c2c08f, 3aad1f2ef | Destructive cursor is `Plane::HighlightDanger`, 256 indices 88/124/52 (not the proposed 131). Test `a_destructive_cursor_row_is_not_grey_on_grey_at_256`. |
| 4 | Fixed | 189734f49, e40c2c08f | Disabled resolves to the kind's rest plane with `disabled` ink; at 16 a grey-on-grey ink falls back to muted plus DIM. Test `disabled_button_ignores_hover`. |
| 5 | Fixed | e40c2c08f, e2afaae4a, 0a965d6b4 | `Kind::Chip`; on = selected, off = secondary ink. 0a965d6b4 re-applies e2afaae4a after 090f2a2f3 reverted it. Test `an_off_chip_label_is_readable_and_distinct_from_an_on_chip_at_every_level`. |
| 6 | Fixed | e40c2c08f | `StateModifiers::MONO`: hover UNDERLINED, pressed REVERSED + BOLD. Test `no_color_buttons_and_menu_cursors_carry_state_in_modifiers`. Mono hover glyph may change under T (SV2). |
| 7 | Fixed | 189734f49, ba849cd89 | One pressed rule: the plane ladder's pressed step plus weight; no white or red flood. Pressed direction (lighter vs darker) is still open: Open → T (SVg2). |
| 8 | Fixed | 189734f49 | `Theme::ladder(Plane)` with explicit rest/hover/pressed; `lift()` is a thin wrapper over it. Test `every_kind_plane_and_level_keeps_rest_hover_and_pressed_apart`. |
| 9 | Fixed | 189734f49, e40c2c08f | `accent_bg_hover` token (256 index 28); a hovered selected row keeps its tint. |
| 10 | Fixed | 189734f49, e40c2c08f | Canvas ladder `#1c1c1f`/`#303036` (234/236); 16 colours press reversed. Covered by `every_kind_plane_and_level_keeps_rest_hover_and_pressed_apart`. |
| 11 | Fixed | 189734f49, e40c2c08f | Card ladder `#2a2a2f`/`#3c3c43`; `accent_bg` index 22 at 256. Same test as #10. |
| 12 | Fixed | 189734f49, e40c2c08f | Mono fill = REVERSED, pressed on a fill = BOLD + UNDERLINED (`StateModifiers::MONO.pressed_fill`). Test `no_color_buttons_and_menu_cursors_carry_state_in_modifiers`. |
| 13 | Fixed | 948e926d8 | Select, Picker and Completion rows use `Theme::cursor_row` with `CursorStyle::Fill` (`focus.md` #1, #6, #7, #15). Tests `only_the_query_field_carries_a_focus_bar_and_the_cursor_row_is_a_fill`, `the_cursor_row_is_a_fill_so_the_editor_keeps_the_only_focus_bar`. |
| 14 | Fixed | 3aad1f2ef | Menu rows resolve as `Kind::CursorRow(Fill)` with their own press. Test `a_menu_row_shows_rest_cursor_hover_and_press_apart_at_every_level`. Pressed direction: see #7. |
| 15 | Fixed | d5e1595a8 | Table and grid count `pressed_any` over the row's cell ids. Test `a_press_on_a_cell_or_the_keyboard_flash_presses_the_row`. |
| 16 | Fixed | d5e1595a8, 3aad1f2ef, d37f14b61, 0a965d6b4, d3a10933b | `Interaction::flash_within` and `RenderCtx::child_state` in list, tree, table, grid, radio, props, steps, menu, tabs, chips and the jackin manager tree. Tests `enter_on_a_focused_list_flashes_its_cursor_row`, `enter_on_the_strip_flashes_the_cursor_tab`. |
| 17 | Fixed | d37f14b61 | `Kind::Tab`: active tab one plane up, hover and press on its ladder, focus is the `▎` gutter. Tests `every_tab_shows_rest_hover_and_press_and_the_close_glyph_its_own`, `hover_and_cursor_differ_from_active`. |
| 18 | Fixed | d37f14b61 | Tab × resolves through `Theme::close_glyph` with its own hover and press. Same test as #17. |
| 19 | Fixed | e2afaae4a, 0a965d6b4 | Chip × is `Kind::Close` (`chips::close_style`) and no longer hovers the chip. Test `hovering_the_close_glyph_does_not_hover_the_chip_and_it_presses`. |
| 20 | Fixed | 189734f49, 3aad1f2ef | Field ladder `#2f2f34`/`#3a3a40` (236/238) through `Kind::Field`. Covered by `every_kind_plane_and_level_keeps_rest_hover_and_pressed_apart`. Mono hover may change under T (SV2). |
| 21 | Fixed | e2afaae4a, 0a965d6b4, e9b8f3631 | `Kind::Handle`. e9b8f3631 replaced the proposed white plus `█` press: one glyph, rest grey, hover lighter, pressed accent (AGENTS.md "Visual states"). Test `the_thumb_keeps_one_glyph_and_colour_carries_rest_hover_and_pressed`. |
| 22 | Fixed | e2afaae4a, 0a965d6b4 | Seams use `Kind::Seam`; the capsule draws through `Splitter::render_held`. Test `seam_rest_hover_and_drag_differ_at_every_level`. |
| 23 | Fixed | 3aad1f2ef | Status items and segments resolve as `Kind::Title`/`Kind::Chip` with a press. Test `a_clickable_chip_keeps_its_tone_and_shows_a_press`. |
| 24 | Fixed | 3aad1f2ef | Menu-bar titles resolve as `Kind::Title` with hover and press. |
| 25 | Fixed | 3aad1f2ef, d37f14b61 | Grid and table headers and overflow badges (3aad1f2ef), tab overflow badges and `+` (d37f14b61) through `RenderCtx::title_style`. |
| 26 | Fixed | d5e1595a8 | Grid range and text selection use `selection_bg` (`#1f3460`, 256 index 17), apart from the popover and pressed rows. |
| 27 | Open → A15 | d5e1595a8 (part) | Table and grid cell cursor fixed (`Theme::cell_cursor`, test `the_cell_cursor_is_the_cursor_fill_not_a_white_flood`). Viewport reversed terminal cells still draw `fg canvas`/`bg text_primary` (`viewport.rs:1816`); fix or record a decision. |
| 28 | Fixed | e2afaae4a, 0a965d6b4 | The brand walks the accent ladder like a primary button. Test `clickable_lockup_rest_hover_and_pressed_differ_at_every_level`. It is pointer-only (`render_clickable` clears focus), so it draws no `▎`. |
| 29 | Fixed | d5e1595a8 | A read-only step rail zeroes hover and press. Test `a_read_only_step_rail_does_not_react_to_hover_or_press`. |
| 30 | Fixed | d3a10933b | Showcase Help/Inspector links, holla finder scope and explore cells, and jackin cockpit chips use `RenderCtx::title_style`. Test `header_links_show_rest_hover_and_press`. |

## Inactive selection (owner decision, 2026-10-17)

**Request.** The current item of a list or tree (the showcase sidebar's
`› Scrolling`) showed the accent fill only while the list had focus; once
focus moved on, only the `›` was left and the current item was hard to
find. The same held for checked boxes, the on radio and the on toggle,
whose accent ink turned primary when unfocused.

**Standard.** macOS keeps an *inactive selection*: a neutral grey fill on
the selected row of a list whose window or view is not key, and the accent
only while focused. Checkboxes and radios keep their accent fill in an
active window whatever has focus.

**Decision.** The owner chose the accent as the default, overriding the
macOS neutral fill for rows: `Theme::inactive_selection` =
`InactiveSelection::Accent`. `Neutral` (macOS: `Theme::selection_inactive`,
`#33363e`, 256 index 238) and `None` (the previous look) stay options,
app-wide (`Theme::with_inactive_selection`, showcase
`shell.inactive_selection`) and per widget (`ListBox`, `TreeView`,
`NavList::inactive_selection`, or `VisualState::inactive_selection` for any
row). One rule serves rows (`Theme::interactive` for `Kind::Row` and the
bar cursor row, so `RowUi`, `Theme::row`, lists, trees, tables, grids and
the sidebar) and chosen marks (`Theme::chosen_in`, `RenderCtx::chosen`:
checkbox, radio, toggle, table check column, row marks).

**Layers.** Under a modal or popup (`VisualState::backdrop`, set by
`RenderCtx::state` from `Interaction::focus_hidden`) the accent falls back
to neutral: content under a layer shows no accent (AGENTS.md "Visual
states"). Keyboard focus is still only the `▎` gutter. The selected row
therefore shows a grey band under a modal or popup (the neutral fill,
dimmed by the modal backdrop to the overlay grey) where it had no fill
before; the owner confirmed keeping it (2026-10-18), so the current item
stays findable behind a dialog.

**Colour levels.** At 16 colours and in monochrome there is no row tint
(`Theme::tint_selection` is off, the fill would collide with a hover or
pressed plane); the selected row keeps `StateModifiers::selected` and its
mark, as before. Tests: `theme::tests::an_unfocused_selected_row_keeps_its_inactive_selection`
(every level, plane and look: fill, ≥ 4.5:1 text, ≥ 3:1 mark, apart from
hover and focused, neutral under a layer, widget override),
`app_tests_nav::the_current_sidebar_row_keeps_its_fill_when_focus_leaves`,
`app_tests_inputs_forms::checks_a_checked_box_keeps_its_accent_when_focus_leaves`.

**Not covered.** Chips and segmented controls already follow the tab rule
(accent while their page is the active layer). The per-widget builders are
shown through the shell option, not a page of their own.
