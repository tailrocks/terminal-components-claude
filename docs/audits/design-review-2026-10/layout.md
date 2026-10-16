Ranked findings, most severe first. Snapshot paths are relative to `/Users/donbeave/Projects/terminal-components-claude/snapshots/`. I made no edits to the repo.

**1. HIGH | No clip region; stacked layouts overflow their rect.**
- Root cause: `src/bin/tablepro/connections.rs:1176-1211` stacks fields with `y += fh` and never checks against `body`. `src/bin/showcase/pages/taskrunner.rs:239-330` draws rows down to `inner.bottom()` and then puts the action row on `bottom-1`. `src/widgets/panel.rs:105-109` returns an empty rect at the bottom edge, and `src/bin/showcase/pages/panels.rs:107` writes to it anyway.
- Affected:
  - `tablepro/connections/form_new{,-filled}/72x20` and `/80x24`: field labels overwrite the hint bar ("EntersEditme← →pBasic…"), and the button row is drawn over "Database" ("Test connection na Cancel").
  - `tablepro/connections/form_advanced/72x20`.
  - `showcase/pages/taskrunner/72x20`: "Run pipeline qu Cancel".
  - `showcase/pages/panels/72x20` (row 19 "Target") and `showcase/pages/chips/80x24` (row 23 segment strip): content lands in the spacer row.
  - `jackin/editor/general/72x20`: "On dirty exit" sits on the same row as Cancel/Save.
- Fix:
  - Add a `ui::layout::Stack`/`Form` that measures its children and scrolls when they overflow.
  - Add a `RenderCtx::clip(rect)` buffer view so widgets can't write outside their rect.
  - Make `Panel::render` return `Option<Rect>`.
  - Options: `overflow: Scroll|Clip` (default Scroll), `min_child_rows`.

**2. HIGH | Dialog `Text` body is cut off with no scroll and no hint.**
- Root cause: `src/widgets/dialog.rs:422-430` stops drawing at `actions_y-1`, and height is capped at `screen.height-2` (`:397`).
- Affected: `tablepro/overlays/help_overlay/72x20` ends at "find (editor)". Alt+D, Ctrl+S, Ctrl+Y and q are missing; compare the 120x40 snapshot.
- Fix: give the body a `ScrollState`, scrollbar and fade edge, plus a "↓ N more" marker; add ↑/↓/PgUp/PgDn to the dialog hints. Options: `body_scroll: bool` (default true), `max_height: Option<u16>`.

**3. HIGH | Facts dialog hides facts silently while leaving rows empty.**
- Root cause:
  - `dialog.rs:445-455` gives the facts only the leftover rows.
  - `props::render` (`props.rs:107-110`) breaks at the bottom with no marker.
  - `ack_rows` (`dialog.rs:212-214`) reserves the input's empty help row (`TextInput::HEIGHT = 3`).
- Affected:
  - `tablepro/ack/delete_gate/72x20`: the Risk line is cut mid-sentence and Safe Mode is gone.
  - `holla/flows/cleanup/gate2_typed/72x20`: Estimate, Denied, Revision, Threat model and Paths are hidden. The safety prompt itself is truncated ("Type TRASH 1 … to confi…").
  - Both screens show 2–3 blank rows above the buttons.
- Fix: make the facts region scrollable with a scrollbar; make `TextInput::height()` dynamic (help row only when there is help text); wrap the ack label, never truncate it.

**4. HIGH-MED | Modals are sized against the whole screen, not the app's body area.**
- Root cause: `dialog.rs:396-398` centres in `screen` and caps at `h-2`.
- Affected: `holla/flows/cleanup/gate2_typed/72x20` (bottom border over the status bar: "~╰──╯0"); `tablepro/ack/*/72x20` (covers the tab strip).
- Fix: `Dialog::render(bounds)` takes the body rect. Option `outer_margin` (default 1 row / 2 cols).

**5. MED | Popup and dialog borders collide with the panel borders underneath.**
- Collisions look like "││", "╰╰", "╭─ ╭──" and "│╭──╮│".
- Root cause: `src/ui/popup.rs:101-136` (`place` has no margin; clamps to screen or panel edge) and `dialog.rs:396` (`screen.width-4`).
- Affected: `jackin/manager/hard_launch_picker/72x20`, `jackin/manager/menu_open/72x20`, `tablepro/overlays/tablist_open/72x20`, `tablist_tables/72x20`, `tablepro/query/completion/72x20`, `tablepro/ack/gate/72x20`, `jackin/accounts/add_form/72x20`.
- Fix: `place(.., margin)`, default 1 cell from any border; dialog horizontal margin default 3.

**6. MED | Hint bar drops hints strictly from the right, and the status can take the whole row.**
- Root cause: `src/widgets/keyhint.rs:~88-93` (`available` = whole row minus x) and the drop loop at `:131-145`.
- Affected: `jackin/settings/save_preview/72x20` shows only "…" plus the status; Esc and Help are lost on many 72-column screens.
- Fix:
  - Add a `Hint.priority` so Esc and ? are pinned.
  - Cap the status's share of the row (`status_max_share`, default 50%).
  - Before dropping hints, switch to a key-only compact mode.

**7. MED | Panel title gives way to the meta down to 4 cells.**
- Root cause: `src/widgets/panel.rs:29` (`TITLE_MIN = 4`) and `:159-190`.
- Affected: `showcase/pages/lists/72x20` and `/80x24` ("Lan… 1–13 of …" — both parts truncated), `showcase/pages/datagrid/72x20` ("cus… rows…"), `showcase/pages/panels/72x20` ("Framed…").
- Fix: options `title_min` (default 12) and `meta_short: Option<&str>` (e.g. "1–9/24"). Shorten or drop the meta before the title goes below `title_min`.

**8. MED | An empty meta draws a blank gap in a framed title ("───  ─╮").**
- Root cause: `panel.rs:195-196` always formats `" {meta} "`.
- Affected: `tablepro/query/new_tab/72x20`, `close_confirm/72x20`, `completion/72x20`, `overlays/help_overlay/120x40`.
- Fix: treat `Some("")` as `None`.

**9. MED | Grid overflow badge: click area doesn't match what's drawn, and it collides with a header.**
- Root cause:
  - `src/widgets/grid.rs:1621` and `:1686` size the click area with `lbl.len()` (bytes): "2›" is 4 bytes but 2 cells, so 2 extra cells are clickable.
  - The partially visible next column (`:1518-1525`) runs into the badge, giving "Isolatio2›" with no ellipsis.
- Affected: `jackin/settings/mounts/72x20` and `/80x24`.
- Fix: use `text::width`; reserve the badge width in `cols_area`; ellipsize the partial header. Option `peek_next_column` (default true, min 6).

**10. MED | EXPLAIN plan labels are cut with no ellipsis.**
- Root cause: `src/bin/tablepro/tabs.rs:1781-1850` paints metric columns over the tree's text. The "%" values also sit 1 cell left of their header.
- Affected: `tablepro/query/explain{,_analyze}/*` — "Parallel Seq Scan on p" (72x20), "public.or" (100x30), "public." (160x50).
- Fix: Tree/TreeTable option `trailing_columns: Vec<Column>` or `reserve_right: u16`, so labels truncate with "…" and columns align with their headers.

**11. MED | Help and key-reference overlays wrap without a hanging indent.**
- Root cause: TablePro uses a Dialog `Text` body; `text::wrap` (`src/ui/text.rs:167-175`) also drops leading indentation.
- Affected: `tablepro/overlays/help_overlay/72x20` and `/120x40` ("(editor)", "zoom" wrap to column 0). Holla's help (`holla/flows/help/overlay/72x20`) does indent correctly, so the apps disagree.
- Fix: a shared `KeyReference` component built on props-style wrapping; make `wrap` keep leading whitespace (`keep_indent`, default true).

**12. MED | Inner padding differs between containers.**
- Measured:

  | Container | Code | Padding |
  |---|---|---|
  | Dialog | `dialog.rs:409` | 2 cols inside the border |
  | Framed Panel | `panel.rs:141-147` | 2 left / 1 right |
  | Card | — | 2 / 2 |
  | Popup surface | `popup.rs:175` | 0, rows add 1 |
  | Jackin help | — | 2 |

- Visible across `tablepro/overlays/picker_open/72x20`, `jackin/manager/help_overlay/72x20` and the dialogs.
- Fix: theme tokens `insets.dialog`, `insets.panel`, `insets.popup` (suggest 2/2/1, symmetric).

**13. MED | Dialog input label sits 1 column right of the title and body.**
- Root cause: `dialog.rs:433-437` and `:478-483` place the input at `inner.x-1`, and the field's gutter is 2 wide.
- Affected: `showcase/flows/dialogs/prompt/*` ("│  Rename task" vs "│   Task name *"); `tablepro/ack/gate/*`.
- Fix: `TextInput::GUTTER` constant; render the field at `inner.x - GUTTER` so the label lines up with the title.

**14. MED | Gap between content and scrollbar varies (0 or 1).**
- Root cause: `PropsList` uses `row_w = width-1` with no gap; the code view and diff do the same. `props::label_width` is also unbounded (`props.rs:59-66, 117`), so values can be drawn past the area.
- Affected: `jackin/accounts/drawer/72x20` ("Work ›│"), `holla/flows/help/overlay/72x20` ("arguments,│"), `showcase/pages/lists/80x24` ("TypeScript┃"), `showcase/pages/codeeditor/80x24`, `showcase/flows/diff/review/80x24`.
- Fix: a shared `scrollbar::split(area) -> (content, track)` with `gap` (default 1). For props: `label_max` (default 40% of width) with a stacked fallback when narrower.

**15. MED | Button rows truncate the last (usually primary) button.**
- Root cause: `src/widgets/button.rs:167-187`. Buttons that don't fit get zero width.
- Affected: `jackin/accounts/drawer/72x20` ("Remov…").
- Gaps also differ: 1 in dialogs, 2 in forms, about 5 in jackin editors.
- Fix: a `ButtonRow` with `overflow: Wrap|More` (default More menu), primary never truncated, and a theme gap token (default 2).

**16. LOW-MED | App column widths are fixed percentages or constants and don't redistribute spare space.**
- Root cause (about 20 sites):
  - `src/bin/holla/screens/finder.rs:875-886`: when the reason column is hidden, its space goes unused.
  - `src/bin/jackin_preview/screens/usage.rs:661`: fixed 15-cell label.
  - The TablePro picker's group column.
- Affected: `holla/concept/docker-cleanup/72x20` ("Remove all unused imag…" with 13 blank cells next to it), `jackin/usage/detail/100x30` ("Session · 5-…" despite plenty of room), `tablepro/overlays/picker_open/72x20` ("audit ·…").
- Fix: a library `Columns` solver with min/ideal/max widths, flex, and drop/shrink priority based on measured content.

**17. LOW-MED | Showcase squeezes sections instead of scrolling the page.**
- Root cause: `src/bin/showcase/pages/mod.rs:145-179` (fixed `rows`; `columns` stacks only below a width threshold).
- Affected: `showcase/pages/tables/72x20` (1 table row visible); `panels/72x20` and `sidebars/72x20` (prose clipped with no marker).
- Fix: a page-level ScrollView with per-section `min_height`; a clip/more indicator for Card bodies.

**18. LOW | Determinate and indeterminate progress bars end at different columns.**
- Root cause: `src/widgets/progress.rs:54` reserves the % column; `:361` doesn't. Labels are measured per bar.
- Affected: `showcase/pages/progress/72x20` and `/120x40` (bar ends differ by 2 cells).
- Fix: options `label_width: Option<u16>` and `reserve_value: bool` (default true for both bar types).

**19. LOW | The sidebar divider is really the sidebar's scrollbar.**
- When the sidebar doesn't overflow there is no divider (`showcase/pages/lists/100x30`).
- At 72 and 80 columns it shows as "┃/│"; at 120 section headers appear. The look flips at each breakpoint.
- Fix: always draw a divider column and overlay the thumb on it.

**20. LOW | Modal positioning is inconsistent.**
- `Placement::Center` uses the upper third (`popup.rs:107`); dialogs are truly centred (`dialog.rs:398`).
- `Below` aligns the popup border, not its text, with the anchor (`popup.rs:130`); item text is about 2 cells right of the typed text in `tablepro/query/completion/72x20`.
- Fix: option `v_align: Center|Third` (default Center); anchored popups shift left by border + padding.

**21. LOW | Byte slicing for truncation.**
- Root cause: `taskrunner.rs:262` and similar use `&label[..label.len().min(10)]`.
- This cuts with no ellipsis and panics on multibyte names.
- Fix: use `text::truncate`.

**22. LOW | Holla Explore chips are dropped silently on overflow.**
- Root cause: `finder.rs:~912` (`break`).
- Fix: show "+N"; option `overflow_marker` (default true).

**23. LOW | Dialog width can exceed a very narrow screen.**
- Root cause: `dialog.rs:396` (`.min(w-4).max(20)`) is wider than the screen below 24 columns.
- Fix: clamp to the screen width last.

**24. LOW | Centred hint bar is off by 1 and alignment varies by app.**
- Root cause: `keyhint.rs:110-128` counts the trailing 2-cell gap when centring.
- Holla and Jackin centre their hint bars; TablePro and Showcase left-align. Both are visible in `jackin/*/100x30` and `tablepro/*/100x30`.
- Fix: exclude the trailing gap from the width; set alignment once in the theme (`hintbar.align`).

**25. LOW | Holla header insets are uneven.**
- Root cause: `src/bin/holla/app.rs:3121-3150` (menu bar) with `segments` EDGE = 1. The left inset is 2 and the right is 1.
- Affected: all `holla/*` header rows.
- Fix: one shared chrome edge token (default 2).

Scratch files (dumps of all snapshot text by size) are in `/private/tmp/claude-501/-Users-donbeave-Projects-terminal-components-claude/42325d29-0331-4e82-a42c-409070ca54fc/scratchpad/audit-layout/`.

## Status

Every finding is fixed in a shared component, or partly fixed with the
remainder recorded here with its reason. Commits are on
`design/interaction-polish`.

| # | Status | Commits |
|---|---|---|
| 1 | Fixed: `ui::layout::Stack` (measures, scrolls by child; `overflow` Scroll/Clip), `RenderCtx::clip`, `RenderCtx::focus_stop`, `Panel::try_render -> Option<Rect>`; TablePro connection form, jackin editor General, showcase Task runner / Panels / Chips | b5a9b2992, 8f5e07e39, 8fe94cbf1, a1091ba6e |
| 2 | Fixed: Dialog Text body scrolls (keys, wheel, scrollbar), "↓ N more", "↑↓ Scroll" hint; `body_scroll`, `max_height` | d2d6896b9 |
| 3 | Fixed: Facts rows scroll; `TextInput::height()`/`height_for()` drop the empty help row; `TextInput::wrap_label` (on for typed acknowledgements) | d2d6896b9 |
| 4 | Fixed: `Dialog::render_in(screen, bounds)` + `dialog::body_bounds`, `outer_margin`; Holla text modals and jackin's modal frame use the same bounds | d2d6896b9, e9a9c1941 |
| 5 | Fixed: `popup::place_with` + `Place::margin` (1), `ContextMenu::edge_margin` (1), dialog margin 3 columns | 6612b4443, d2d6896b9 |
| 6 | Fixed with two deviations (below) | f0cb83a02 |
| 7 | Fixed: `Panel::title_min` (12), `meta_short`, automatic short position label | 4a4e30830 |
| 8, 9, 21, 23 | Fixed earlier on this branch | — |
| 10 | Fixed: `TreeView::reserve_right` | 46c86a96f |
| 11 | Fixed: `WrapOpts::keep_indent`, `hang_at_gap`; `KeyReference` used by TablePro's help; partial (below) | f2c4e04cb |
| 12 | Tokens added (`Theme::spacing`); dialogs adopt `spacing.dialog`; partial (below) | 2a3fe1d42, c394ed8a4 |
| 13 | Fixed: `TextInput::GUTTER`; dialog fields render at `inner.x - GUTTER` | d2d6896b9 |
| 14 | Fixed: `scrollbar::split` with gap 1 in ListBox, PropsList, CodeEditor, TextViewport/Diff; `props::Sheet::label_max_share` with stacked fallback; partial (below) | e5afa0423, 72dff1756 |
| 15 | Fixed: `ButtonRow` (primary never cut; More/Wrap; theme gap 2) in jackin's account drawer and every Dialog; partial (below) | 72dff1756, 355227030 |
| 16 | Fixed: `ui::columns` solver in the quick switcher, Holla results, jackin usage | d09f244c9 |
| 17 | Fixed for the audited pages: showcase `Sections`, `Panel::mark_clipped`; partial (below) | 8fe94cbf1, 919cd85bb |
| 18 | Fixed: `BarLayout::label_width`, `reserve_value` | 5b0c2fd42 |
| 19 | Fixed: `scrollbar::render_divider`; partial (below) | 9a1672acc |
| 20 | Fixed: `Place::v_align` (Center default, Third on request), `Place::text_inset`; completion text lines up with the typed word | 6612b4443 |
| 22 | Fixed: `chips::overflow_run` "+N", `ChipBar::overflow_marker`, `Finder::explore_overflow_marker`; partial (below) | 55264ea76 |
| 24 | Fixed: centring excludes the trailing gap; `Theme::hintbar.align` set once per app | f0cb83a02 |
| 25 | Fixed: `Spacing::chrome_edge` (2) for menu bars and segment strips | 23ccf0a20 |

### Follow-ups landed later

- **#1, `Stack` scrollbar.** A press on the bare track now follows
  `Stack::track_click` (default `Jump`; `TrackClick::Page` pages by
  children), as do the vertical bars of `Dialog` bodies (#2), `StepRail`
  and `PropsList` (#14): `200e688ad`.
- **#14, `PropsList` reveal.** `RevealMode::InlineWrap` wraps the cursor
  row's value in place (the row grows, the rows below move down, a long
  value scrolls before the sheet) instead of falling back to the
  tooltip: `580d279f1`.

### Not done, with reasons

- **#1, `Panel::render` signature.** It still returns `Rect`. Changing it to
  `Option<Rect>` touches 80+ call sites in four apps that other agents were
  editing at the same time; `Panel::try_render` returns `Option<Rect>` and
  call sites move to it as they are touched.
- **#6, `status_max_share`.** Implemented as `HintBarStyle::status_min_share`
  (default 50%) instead: a hard cap whenever hints wait cut error and
  confirmation messages short (TablePro's read-only notice, Holla status
  lines). The status now gives up room only to the pinned hints (Esc, ?,
  F1), which is what was being lost.
- **#6, compact mode.** Built (`HintBarStyle::compact`) but off by default:
  on jackin's footer it produced a row of bare keys with no verbs, which
  says less than dropping hints from the right.
- **#11, Holla and jackin help.** Not moved to `KeyReference`: both already
  indent wrapped lines correctly through their own key/value lines, and
  moving them means rewriting their modal widgets for no visible change.
- **#12, panels and popups.** Framed panels pad 2 left / 1 right and popup
  surfaces 0; `spacing.panel` (2×0) and `spacing.popup` (1×0) exist but are
  not wired in, because doing so moves every framed panel and popup by a
  column. That is a deliberate visual change needing its own baseline
  re-approval, not a side effect of this pass.
- **#14, tables and grids.** Their scrollbars do not use `scrollbar::split`
  yet; they already keep their own one-column gutter.
- **#15, other button rows.** TablePro's connection form, the showcase task
  runner and the Holla/jackin modals still lay buttons out with
  `row_layout`. A `RowOverflow::More` row needs its owner to open a menu of
  the hidden buttons, which each of those screens would have to add.
- **#17, other showcase pages.** Only the pages the audit names use
  `Sections`; the rest keep fixed `rows()` and convert one by one.
- **#19, breakpoints.** The sidebar's switch between section headers and the
  compact list at 120 columns is intended responsive behaviour and stays.
  The divider no longer changes.
- **#22, Holla Explore row.** Wired to the shared "+N" helper, but the row
  never overflows at any supported size (72 columns minimum), so there is
  no app-level test; `chips::overflow_run` and `ChipBar` have unit tests.
- **#4, quick switcher.** `Picker` still centres in the whole screen (now
  with the one-cell margin and true vertical centring from #5 and #20). It
  draws its own hint line in the footer, so its placement was left with
  the screen.