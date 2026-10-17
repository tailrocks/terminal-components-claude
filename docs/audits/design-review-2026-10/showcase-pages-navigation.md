# Showcase page steps: decisions recorded instead of fixed

Scope: the page steps SC14 (Tabs), SC16 (Menus), SC28 (Focus & navigation,
Toolbars) and SC27 (Settings, Task runner) of
[`shared-components-plan.md` §7](../../architecture/shared-components-plan.md).
Each entry is a finding met while building a page that the step did not
fix, with the reason.

## SC14 Tabs

1. **The `+N` overflow button is demonstrated as it is.** S5 owns the
   overflow code (badge as `IconButton`, the strip under its open menu
   showing no focus); SC14 does not edit it. The Playground shows it at
   120 columns and `tabs.more_button` swaps the button through
   `Tabs::more_button`.
2. **The overflow menu has no key of its own.** `+N` opens by pointer
   only; the keyboard reaches every tab through ←/→, Home/End and the
   digits, so no tab is keyboard-unreachable (WCAG 2.1.1 holds). A key for
   the menu belongs with S5's overflow work, not with this page.
   **Fixed in S5:** ↓ on a focused strip (`TabCmd::ShowHidden`; chip bars
   `ChipCmd::ShowHidden`) opens the menu with its first row highlighted,
   the WAI-ARIA menu-button key, which a horizontal strip has no other use
   for; only while something is hidden, otherwise ↓ does what it did. The
   `+N` tooltip names the key from the key map (remappable, shown only when
   bound). Tests: tablepro `down_on_the_strip_opens_the_overflow_menu`,
   `chips::overflow_tests::down_opens_the_menu_of_the_hidden_chips`.
3. **No "selected" or "disabled" row in the Tabs States grid.** Every strip
   has an active tab, so selection shows in every row (the accent rule);
   `Tabs` has no disabled state (`TabItem` has no `disabled`). Adding one is
   a library feature with no app asking for it yet.

## SC16 Menus

1. **`RowMenu` stays showcase routing.** `pages/row_menu.rs` holds the open
   menu and maps page events onto `ContextMenu::on_key`/`on_click`; every
   decision (placement, keys, dismissal, press-drag-release) is the
   menu's. Moving it into `ui::context` would touch the Lists, Tables,
   Trees, Selection and Chrome pages owned by other running steps (SC4,
   SC11, SC13, SC15). Follow-up: extract it as a library "open context
   menu about T" once those steps land, and point the apps' copies at it.
   Placed in plan step **S22** (`CommandMenu` + `ContextAnchor::row`):
   its contract deletes `src/bin/showcase/pages/row_menu.rs` and moves
   `pages/chrome.rs`, `columns.rs`, `lists.rs`, `menus.rs`,
   `selectable.rs`, `tables.rs` and `trees.rs` onto `CommandMenu`, beside
   jackin's `screens/rowmenu.rs`.
2. **Primary press-drag-release on a button menu is not offered.**
   `ContextMenu::release_selects` covers the secondary gesture (right-press,
   drag, release); the primary gesture is wired only for `MenuBar` titles
   (`release_from` is private). The Actions button opens on click, as
   macOS pop-up buttons do; the page says where the gesture lives.

## SC28 Focus & navigation, Toolbars

1. **Fixed: the runtime hardcoded the input normalisation and the
   keyboard protocol.** `runtime::run` read `InputConfig::default()` and
   `KeyboardProtocol::current()` itself, so an app could not turn off
   Shift+wheel sideways, Ctrl+click as a right click or the legacy Ctrl
   punctuation mapping, nor choose the protocol other than through the
   environment. `Application::{input_config, keyboard_protocol}` now
   carry them (defaults unchanged), `TerminalSession::enter_with` takes
   the protocol, and `impl_application!(App, input)` forwards an app's
   own. Proof: `runtime::tests::the_event_loop_reads_input_through_the_apps_input_config`,
   `app_tests_focusnav::the_shell_input_and_protocol_options_reach_the_runtime_hooks`.
2. **Fixed: a frame with no focus after the focused control vanished.**
   The showcase shell fell back to the ring's first stop after drawing,
   so the frame that removed the control showed no `▎` until the next
   input. It now applies `Focus::ensure_valid_with` with the
   `shell.focus_fallback` policy and draws once more when focus moved.
   Proof: `app_tests_focusnav::a_removed_focused_tag_falls_back_per_the_shell_option`
   (one `▎` in the same frame).
3. **Fixed: the showcase defaulted to `FocusFallback::First`**, the
   library's default is `Nearest`; the showcase must show library
   defaults. `shell.focus_fallback` now defaults to `nearest` and `first`
   stays one option away. Switching showed a library defect: a click that
   focused a control which cannot take focus (the disabled API token
   field) sent `Nearest` to the stop before the previous focus, so another
   field started editing. `Focus::ensure_valid_with` now keeps focus on the
   control that last held it while that control is still in the ring
   (`First` keeps its rule). Proof:
   `core::focus::tests::focus_moved_onto_an_unfocusable_control_stays_where_it_was`,
   `app_tests_focusnav::the_showcase_focus_fallback_is_the_library_default`,
   `app_tests_focusnav::a_removed_focused_tag_falls_back_per_the_shell_option`
   (default nearest, then `first`) and
   `app_tests_coverage::inputs_paste_validation_and_disabled_field`, whose
   last assertion changed from "nothing edits" (true under `First` only
   because focus left for the sidebar) to "focus stays on Branch".
4. **`Ctrl+click is right-click` has no option of its own.** It follows
   the client platform (`InputConfig::for_platform`), which
   `shell.platform` already sets; a separate auto/off/on option would
   duplicate `auto` with one of the others on every host.
5. **The Input and Protocol lines show the library defaults, not the
   shell's live values.** A page has no read access to the app's
   configuration; the Inspector's Options tab shows the live values.
6. **Not built: Strategy B nav policies (`FocusPolicy`, `AdjacentMove`,
   `AdjacentKeys`, `Scope`, `Bounds`) and their fields on Tabs, Tree,
   List, Select, Picker, Menu, Dialog and Toolbar.** They exist only as
   inventory rows (no type, no catalog id). They change how every
   navigable widget moves focus across its edges, which is an S-step's
   expand/migrate/contract work (plan §4), not a page step's; SC28 shows
   what the library has today: `NavKeys` per widget, `NavProfile`
   defaults, the ring, the fallback and focus requests. Planned: library
   step **S27** (plan §7.3: scope, acceptance, deps S4, S7, S8a), shown
   by **SC57** on the Focus & navigation and Toolbars pages. They have no
   catalog id yet, so no allowlist line; the inventory rows (§7.7) point
   at S27/SC57.

## SC27 Settings, Task runner

1. **The legacy capture `showcase/flows/settings/toggled_off` changes on
   purpose.** Two toggles leave General dirty, and the tab now carries the
   dirty mark (`TabItem::dirty`, `Theme::glyphs.dirty`) after its label;
   every other cell is as the `visual-baseline` tag draws it (the tab
   strip's look is the tag's; only the two-cell mark is new). The capture
   is `legacy` (not gated); it is re-recorded with the next showcase
   re-approval of the Settings flows.
7. **Built: `widgets::toolbar::Toolbar`** (the APG toolbar: one focus
   stop, ←/→ through `Cursor` skipping disabled items unless
   `skip_disabled(false)`, Home/End, Enter/Space run or flip a toggle,
   shortcuts and tips, separators, the one `▎` on the cursor item).
   Fixed since (nav follow-ups): items that did not fit were silently not
   drawn, so keyboard and mouse users lost commands (APG toolbar, WCAG
   2.1.1). They now go behind an overflow `»` `IconButton`, the last
   ←/→ stop, whose `ContextMenu` (overlay layer) lists them with
   shortcuts, toggle state and disabled reasons; `Toolbar::overflow`,
   `more_button` and `more_glyph` override it. The Toolbars page shows it
   in a Narrow toolbar card, `toolbars.overflow` and an Overflow States
   row. The toolbar owns its menu over the shared `ContextMenu`, the way
   S5 left `Tabs` `+N`, `ChipBar` and `MenuBar` `»` (S5 added no shared
   overflow helper); a later one moves all four together. Proof:
   `toolbar::tests::items_that_do_not_fit_go_behind_an_overflow_button_at_the_end`,
   `app_tests_toolbars::the_narrow_toolbar_keeps_hidden_commands_in_its_overflow_menu`.

## SC5 Breadcrumbs & rails

1. **No rail section on the Sidebars page.** The plan asks for the
   `SidebarRail` beside the Sidebars page's hand-made `NavList`. That page
   is rebuilt on the library `NavList` by S17/SC36, and its approved
   captures would change for a second copy of the demo the Breadcrumbs &
   rails page already shows live (rail, collapse button, `RailAction`,
   `collapsed_rail`) and in its States grid. SC36 shows the rail on
   Sidebars when it moves the list into the library.
2. **No narrow drawer.** Below 100 columns tablepro turns its explorer into
   a drawer (`workbench.rs`, the explorer takes the body while focused). That
   is app layout code; a copy on the page would be a second implementation
   of the same rule. The library drawer is S19 (master-detail, SC38), where
   the page can show it.
3. **`0` does not expand the rail.** `0` is the showcase shell's "jump to
   navigation" key. The rail opens by a click anywhere on it and by Ctrl+B,
   which the page hints advertise.
4. **Trails collapse their outer levels, not their middle.** The plan's
   "middle collapse" reads as the macOS path bar shrinking middle items.
   `Breadcrumb` drops the outermost levels into a leading `…` and always
   keeps the current page (Windows Explorer's address bar does the same);
   its tests pin that rule. The current page itself is now cut in the
   middle when it is too wide (library fix, regression test).

## SC6 Help & keys, Quit & leave

1. **The help sections are hand-listed.** The shell's `?` reference lists
   the shell's keys by hand until S8b generates them from the key maps
   (`HelpOverlay::from_keymaps`, SC33).
2. **The quit-key options drive the page's table, not the shell.**
   `quitleave.letter`, `esc_quits`, `ctrl_c` and `ctrl_q` rebuild the
   `QuitKeys` the classify table reads. The showcase's own quit keys stay
   `QuitKeys::DEFAULT` on every page; making them a page option would
   change how the showcase quits from one page only.

## SC15 Lists & trees

1. **Fixed: hints advertised keys that did nothing.** Lists hinted
   `g G Ends`; under the default desktop keys (S4) `g` and `G` type-select
   instead. `NavKeys::{move_hint, page_hint, ends_hint}` now name only the
   keys a widget's `nav` and `type_ahead` deliver, and Lists and Trees
   build their hints from them.
2. **Fixed (71273406c): Panels and Scrolling no longer hint `g G Ends` by
   hand.** `pages/panels.rs` and `pages/scrolling.rs` (SC22) take the hint
   from `NavKeys::ends_hint`, as Lists and Trees do.
3. **Type-ahead precedence and timeout are left for SC29.**
   `TypeAheadPrecedence` and the `type_ahead_timeout` builders are
   allowlisted to SC29, which shows the Cursor/RowSource defaults together
   on the Scrolling page.

## S14 Page stack in the showcase shell

1. **The header trail of a pushed page shows the stack, not the sidebar
   group.** A root page (reached from the sidebar, `--page` or
   Ctrl+PgDn) keeps "group › page" as plain text, as before: the sidebar
   is how pages are reached, and a group is not a page. A page opened from
   another page's link ("Disk cleanup →" on Read-only report) shows the
   stack ("Read-only report › Disk cleanup"); every earlier level is a
   link that pops back to it (M5, `Router::breadcrumb` and
   `Router::crumb_click`). Keeping the group in front of a pushed trail
   would show a level that looks like a link but goes nowhere (a group
   has no page), so it is dropped there. This is a deliberate visual
   change of the header on pushed pages only; root pages are unchanged.
   Tests: `report_link_pushes_the_page_and_esc_returns_to_the_button`,
   `report_header_trail_pops_back_to_the_report`.
2. **Esc at the top of a pushed page goes back** (focus returns to the
   control the page was left from); at the root of the stack it still
   returns focus to the sidebar.
3. **Fixed: the coverage gate did not scan `src/shell`.** `shell` was not
   in `ROOTS` (`src/bin/showcase/catalog_scan.rs`), and its API lives in
   private modules re-exported from `shell`, which the scan did not
   follow. Both are fixed; the 187 shell capabilities are in
   `src/catalog.rs`, pending on SC34 (the showcase shell page) except the
   run-loop entry and the OSC 52 output plumbing (`permanent`).

## Status (T19 sync, 2026-10-17)

Each entry above was checked against `git log`, the code at HEAD,
`docs/remaining-work.md` and the plan's SC outcomes; entries that only
said "open" or "planned" get their owner id, fixed ones their commit.

| # | Status | Commit(s) / owner | Notes |
|---|---|---|---|
| SC14 1 | Decision (the `+N` button is S5's code; the page demonstrates it) | — | `tabs.more_button` swaps the button. |
| SC14 2 | Fixed | 7cd9a8835 | ↓ on a focused strip opens the overflow menu (`TabCmd::ShowHidden`, `ChipCmd::ShowHidden`). |
| SC14 3 | Decision (`Tabs` has no disabled state and no app asks for one) | — | Selection shows in every row by design. |
| SC16 1 | Open → S22 | — | `RowMenu` moves onto `CommandMenu`; `pages/row_menu.rs` is deleted in S22's contract. |
| SC16 2 | Open → A13 | — | Primary press-drag-release on a button menu (`release_from` is private). |
| SC28 1 | Fixed | 8016cb3ab | `Application::{input_config, keyboard_protocol}`; test `runtime::tests::the_event_loop_reads_input_through_the_apps_input_config`. |
| SC28 2 | Fixed | 8016cb3ab | `Focus::ensure_valid_with` and a second draw; test `a_removed_focused_tag_falls_back_per_the_shell_option`. |
| SC28 3 | Fixed | aabb7771e | `shell.focus_fallback` defaults to `nearest`; test `focus_moved_onto_an_unfocusable_control_stays_where_it_was`. |
| SC28 4 | Decision (Ctrl+click follows `shell.platform`; a separate option would duplicate it) | — | — |
| SC28 5 | Decision (a page cannot read the app's configuration; the Inspector Options tab shows live values) | — | — |
| SC28 6 | Open → S27 | SC57 | Strategy B nav policies. |
| SC27 1 | Open → A13 | with V1 | The legacy `showcase/flows/settings/toggled_off` capture is re-recorded with the Settings flows. |
| SC27 7 | Fixed; one shared overflow menu open → A13 | 921049106 | `Toolbar` overflow `»` menu; Toolbar, Tabs `+N`, ChipBar and MenuBar `»` still each own their menu. |
| SC5 1 | Open → SC36 | S17 | `SidebarRail` on the Sidebars page when `NavList` moves into the library. |
| SC5 2 | Open → SC38 | S19 | The library drawer is `MasterDetail`. |
| SC5 3 | Decision (`0` is the showcase's jump-to-navigation key) | — | The rail opens by click and Ctrl+B. |
| SC5 4 | Decision (outer levels collapse, as Windows Explorer's address bar) | d62744f97 | The current page is cut in the middle when too wide (library fix in the same step). |
| SC6 1 | Open → SC33 | S8b | `HelpOverlay::from_keymaps` replaces the hand-listed `?` sections. |
| SC6 2 | Decision (a page option must not change how the showcase quits from one page only) | — | — |
| SC15 1 | Fixed | 2398b5f91 | `NavKeys::{move_hint, page_hint, ends_hint}`; Lists and Trees build their hints from them. |
| SC15 2 | Fixed | 71273406c | Panels and Scrolling take `NavKeys::ends_hint` (SC22). |
| SC15 3 | Fixed in part; rest open → SC34, SC41 | 71273406c | SC29 shows `TypeAheadPrecedence` and the Cursor/RowSource timeouts; `MenuBar::type_ahead_timeout` moved to SC34 and `ContextMenu::type_ahead_timeout` to SC41 (plan §7.8). |
