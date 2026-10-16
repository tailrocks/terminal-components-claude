Found 22 issues, ranked below. The "Debug info" white-on-white bug comes from one shared cause in the theme, not from the screens. Contrast figures are WCAG ratios computed from the `palette::*` RGB values. I backed them with a scan of every snapshot `.ansi` file for cells whose text contrast falls below 4.5:1 or where text and background colours are equal. 256-colour values use `nearest_256` output. 16-colour values use the snapshot renderer's palette (DarkGray `#7f7f7f`, LightRed `#ff0000`, Gray `#e5e5e5`). `none` = `ColorLevel::Mono`. I did not run any tests or change anything in the repo.

1. **high | src/theme.rs:363 (`row()` pressed branch), with consumers that override the text colour after calling it: src/widgets/props.rs:289-307, steps.rs:273-293, grid.rs:1758-1800, select.rs:389, list.rs:283, tree.rs:567, choice.rs:84/217/313, completion.rs:187, picker.rs:569, plus every app screen that calls `t.row()` (manager, accounts, settings, editor, inspect, usage, holla finder/files/cleanup/disk/plan, showcase sidebars/app) | jackin cockpit and capsule "Debug info" (the reported bug), every InfoDialog/PropsList, every list/tree/select/picker/steps row on click or Enter-activation (`Interaction::pressed` includes the activation flash) |**
   - Pressed fills the row with `text_primary` `#ffffff` and sets the text to `canvas`. Widgets then re-apply their own text colours:
     - value text `t.tone(Normal)` = `#ffffff` on `#ffffff`: 1.0:1, invisible.
     - label `text_muted` `#808080` on white: 3.95:1.
     - "y copy" `text_faint` `#4d4d4d` on white.
     - accent marker `›`/`✓` `#48e054` on white: 1.74:1.
     - warning `•` amber on white: about 2.1:1.
   - Only table.rs:757 resets text colour on press; steps.rs only drops bold.
   - Fix: make pressed a plane, not an inverted flood: `bg = popover` (`#3f3f46`, white text 10.4:1) plus BOLD, keep `fg = text_primary`, and add `REVERSED` only in Mono.
   - Also add `Theme::row_fg(s, tone) -> Color` that returns a readable colour for the state, and have widgets call it instead of `st.fg(...)`.
   - Add a unit test in theme.rs: for every `VisualState` × `Tone`, contrast against the row background must be ≥ 4.5:1.

2. **high | src/theme.rs:308 (`backdrop()`: `fg == canvas|surface ⇒ bg`) | jackin editor/save_preview ("[ ] Keep awake" row behind the modal), any reversed, pressed or cell-cursor cell under a modal |** The rule meant to keep empty cells empty also erases reversed text (black text on a white fill). Measured: `#27272a` text on `#27272a` (1.0:1), and the checkbox at `#262626` on `#27272a` (1.02:1). Fix: when the original background is lighter than its text, treat the cell as reversed and map it to `text_muted` on `surface_overlay`. Only blank cells (no symbol) should take the background colour.

3. **high | src/theme.rs:318 (`_ => text_ghost`) together with :603 (Mono) and :629 (`nearest_16`) | 293 screens in Mono, 62 screens in 16-colour: every brand lockup under a modal, "4 roles", "3 trusted · 1 untrusted", "Save…" |**
   - Backdrop text tiers collapse into the backdrop's background at reduced colour levels.
   - Mono: `text_ghost` = Black on canvas/overlay Black.
   - 16-colour: `text_faint` and `text_ghost` = DarkGray on `surface_overlay` = DarkGray, so the brand reads `7f7f7f` on `7f7f7f`.
   - Fix: `backdrop()` must check that the text and background differ after the downgrade. In Mono and 16-colour, use `Modifier::DIM` on the original text colour instead of the colour ladder.

4. **high | src/widgets/button.rs:129 and :158 | busy primary buttons: tablepro Connect / Save & connect, jackin editor and settings "Save…" |** A busy Primary button sets its text to `text_secondary` `#b3b3b3` on accent `#48e054` (about 1.2:1), and the spinner to `accent` on `accent` (1.0:1, invisible). Fix: when the button is Primary, use `text_on_accent` for both label and spinner; better, have `Theme::button` take `s.busy` and return the colours itself.

5. **high | src/theme.rs:603-609 (Mono thresholds) | all `none` captures (27,272 black-on-black cells across 293 screens) |**
   - Values with an average below 40 all become Black: `border_subtle` `#262626`, `surface_overlay`, `field_hover`, `accent_bg`, `error_bg`. As a result:
     - every subtle border vanishes (pane frames in capsule, settings dividers);
     - hover on canvas/surface/card/field is a no-op;
     - the focused-selected tint disappears;
     - the error-row tint disappears.
   - Fix: give Mono its own token table instead of thresholding: `border_subtle` = DarkGray; selection, hover and pressed carried by `REVERSED`/`BOLD`/`UNDERLINED` modifiers. This should be a configurable per-level override hook.

6. **high | src/theme.rs:75/92 with the Mono/256 downgrade, and src/widgets/menu.rs:313-320 | every menu (jackin capsule/menu, manager/menu_open, holla menus, showcase chrome) in Mono and 256 |**
   - Mono: `highlight`, `highlight_danger` and `popover` are all DarkGray, so the cursor row is the same plane as the menu. In capsule/menu `none.png` the cursor row actually looks dimmer than the other rows.
   - 256: `highlight_danger` `#7a2a2a` becomes `#444444` against popover `#3a3a3a` (about 1.1:1), so a destructive cursor row is invisible.
   - Fix: menu cursor uses `REVERSED` in Mono. Pin `highlight_danger` in 256 to index 88 (`#870000`). More generally, `for_level` should take explicit per-level overrides for selection tokens rather than nearest-match.

7. **high | src/theme.rs:629 (`nearest_16`) and :183 (`for_level`) | all 16-colour screens: 105 screens with Gray-on-DarkGray, 35 with LightRed-on-DarkGray |**
   - `input_hover`, `surface_overlay`, `popover`, `accent_bg`, `border_subtle`, `border_strong`, `text_faint` and `disabled` all collapse to DarkGray. `text_secondary` and `text_muted` both become Gray.
   - Results:
     - menu borders and separators are invisible on the popover;
     - faint metadata on a selected row is invisible;
     - danger labels LightRed on DarkGray measure 1.0:1 (capsule "Close tab", "Delete workspace…", "Cancel launch", "Remove…", tablepro "Delete…");
     - the secondary/muted hierarchy is lost.
   - Fix: a hand-mapped 16-colour token table:
     - surfaces: Black;
     - `popover`/`surface_overlay`/`accent_bg`: DarkGray;
     - `border_subtle`: DarkGray only on Black planes, otherwise Gray;
     - `text_faint`: Gray + DIM;
     - `error_soft` on popover: White + LightRed underline or prefix glyph.

8. **med | src/theme.rs:351 (hover applied after the selection tint) | every list/tree/table/props row |** Hovering the focused-selected row replaces `accent_bg` with `lift(bg)`, so the selection disappears under the pointer, and selected and hovered rows become identical. Fix: when selected+focused, hover should lift the tint (e.g. `GREEN_20` → a `GREEN_30` token), never replace it.

9. **med | src/theme.rs:373 (`lift()` falls back to `popover`) and menu.rs:326 | all menus (popover background), anything already on `popover` or `field_hover` |** `lift(popover)` returns `popover`, so menu hover is a no-op. Other lifts are near-invisible: field to `field_hover` is 1.06:1; canvas to card and card to overlay are 1.19:1. In 256 and Mono, `field` and `field_hover` are identical. Fix: lift by a minimum luminance step (≥ 1.3:1). Add a `popover_hover` token (about `#52525b`), and assert `lift(x) != x` for all planes.

10. **med | src/theme.rs:170 (`accent_bg` `#0f2e13`) | focused-selected rows on every surface; showcase datagrid/selected (tint barely visible on chrome) |**
    - The tint is 1.42:1 against canvas and 1.2:1 against card.
    - In 256 it maps to `#121212`, the same as `surface_elevated` (and as `error_bg`), so selection inside dialogs and dropdowns is invisible and loses its green.
    - `text_muted` on it is 3.74:1; `text_faint` is 1.75:1.
    - Fix: raise to about `#173f1c` and pin the 256 mapping to index 22 (`#005f00`). Keep `text_muted` off selected rows (use `text_secondary`).

11. **med | src/widgets/menu.rs:366 (shortcut colour) | menu cursor rows: "2×click", "Enter", "Alt+0" in jackin, holla, showcase |** `text_muted` on `highlight` measures 1.69:1 (256: 1.05:1). At rest on popover it is 2.64:1. Fix: cursor-row shortcut uses `text_secondary` with white at 70% blended on the highlight (about `#c9d6ee`, ≥ 4.5:1); rest shortcuts use `text_secondary` (4.98:1 on popover).

12. **med | src/theme.rs:397 (`gutter()` with `on_accent`) | focused primary buttons (jackin quit_confirm, tablepro ack/armed, showcase buttons/dialogs) |** The `▎` bar measures:
    - white on accent: 1.74:1;
    - on a pressed Secondary (white fill), green on white: 1.74:1;
    - on a pressed Danger (red fill), green on red: 2.3:1;
    - Mono, white on Gray: 1.26:1.

    Fix: when the bar sits on a filled background, use `text_on_accent`/`canvas` (black, ≥ 10:1); generally pick black or white by the background's luminance.

13. **med | src/theme.rs:456-466 (`ButtonKind::Danger`) and `error_soft` | danger buttons and menu items across all apps |** Error `#e44545` on `surface_overlay` is 3.71:1; hovered on popover 2.6:1; pressed white on red 4.01:1. `error_soft` on popover is 3.96:1. Fix: danger rest label `#ff6b6b` (≥ 4.5:1 on `#27272a`/`#3f3f46`). Pressed: fill `#b91c1c` with white text (6.5:1).

14. **med | src/theme.rs:164 (`text_faint` = `disabled` = `#4d4d4d`) | 300 screens (about 24k cells of content text): settings help text, "Directory basename by default", tablepro status "ln 1/1 · col 1", form value "bastion.example.com", row numbers, dialog meta "read-only" |** The colour is 2.48:1 on canvas, 2.1:1 on card, 1.97:1 on field, 1.76:1 on overlay, so readable text is below 3:1. Because `disabled` uses the same value, disabled and faint/skipped items are also indistinguishable. Fix: `text_faint` about `#6e6e6e` (≥ 3:1 on card, for metadata only). Keep `disabled` a separate, configurable token (`#4d4d4d` plus DIM) so disabled ≠ faint.

15. **med | src/theme.rs:159 (`border_subtle` `#262626`) | panel and menu frames and dividers on every screen; the code editor/textarea line-number separator at 1.1:1 on field |** It is 1.39:1 on canvas, 1.25:1 on surface, 1.17:1 on card, 1.45:1 on popover, 1.03:1 on `field_hover`, below the 3:1 mark for UI glyphs. In 256 it equals `surface_overlay`. Fix: `border_subtle` about `#3a3a3a` on dark planes, and use `border_strong` on popover and field planes.

16. **med | src/widgets/table.rs:792-798, grid.rs:1872-1885 | table and data-grid cell cursor (showcase datagrid/editable, tablepro table) |** The cursor is a white-fill block that is identical to the pressed-row style, so cursor and press can't be told apart. Deleted cell: muted on white, 3.95:1. Error cell: white on red, 4.01:1. This also contradicts the theme's own rule that selection uses a ring, never a colour flood. Fix: add a `Theme::cell_cursor(err)` resolver that returns an underline/bold on the `highlight` plane, used by both table and grid.

17. **med | Cross-widget: theme.rs:340 `row()`, menu.rs:313, table.rs:792, select.rs:389, code.rs:830, viewport.rs:1473 | list vs tree vs table vs menu vs select vs editor |** Six different selection treatments:
    - list/tree: `accent_bg` tint + `▎` + bold;
    - menu: blue fill, no bar;
    - select dropdown: bar only, unless the cursor is on the chosen item;
    - table/grid: white cell flood;
    - text selection: `popover`;
    - code editor: current find match = `popover`, the same as the selection.

    Fix: one `Theme::cursor(kind)` / `Theme::selection(kind)` pair used by every widget, with the menu cursor, the list cursor and text selection as the only variants.

18. **med | src/widgets/viewport.rs:1460-1463 | terminal/scrollback search (showcase terminal, jackin capsule panes) |** The current match sets black text on a green fill and then adds `REVERSED`, which swaps them back to green text on black. The current match ends up looking weaker than other matches (underline on a lifted plane). Fix: drop `REVERSED` outside Mono, or drop the colours and keep only `REVERSED`.

19. **med | src/widgets/code.rs:830-836 | showcase codeeditor find |** The current find match and the text selection both use `bg = popover`, so they are indistinguishable. Fix: current match uses a `match_current` token (amber 20% or highlight); other matches keep the underline.

20. **low | src/theme.rs:418-420 (Primary pressed `accent_pressed` `#2b8632`) | primary buttons and the brand lockup when pressed |** Text is 3.81:1. In 256, `GREEN_60` maps to `#444444`, so a pressed primary turns grey with text about 1.9:1. Fix: `accent_pressed` about `#34a83d`, pinned to 256 index 71.

21. **low | src/theme.rs:177 (`success: GREEN` = accent = focus) | steps done, tables "Done", the holla `[✓]` marks |** Success, focus bar and accent share one colour, and every tone collapses to Gray in Mono. Fix: separate `success` token (e.g. `#3fb950`); for tone meaning in Mono/nocolor, require a glyph (✓ ! •). Widgets mostly do this already.

22. **low | src/widgets/segments.rs:106 and tabs.rs:429 | statusbar segments and tab close `×` |** Hover forces `text_primary`, so a warning or error segment loses its tone under the pointer. The tab close `×` at rest is `text_faint` (2.48:1, below the 3:1 glyph mark). Fix: keep `tone` on hover and only lift the background; `×` at rest uses `text_muted`.

**Configurable-token notes:**
- **Overrides:** add a per-level override table to `Theme::for_level`, at least for `accent_bg`, `highlight_danger`, `accent_pressed`, `border_subtle`, `text_faint` and `popover_hover`. Nearest-match downgrading is the source of findings 3, 5, 6, 7, 10 and 20.
- **New tokens:** `popover_hover`, `match_current`, a separate `disabled` and a separate `success`.
- **Contrast test:** add one test that runs every `row`, `button` and `gutter` state through all four `ColorLevel`s and requires text contrast ≥ 4.5:1, glyph contrast ≥ 3:1, and text ≠ background.

The scan scripts and output are in `/private/tmp/claude-501/-Users-donbeave-Projects-terminal-components-claude/42325d29-0331-4e82-a42c-409070ca54fc/scratchpad/audit-color/`: `scan.py`, `pal.py`, `white.py`, `eq.txt` and `lt45.txt`.