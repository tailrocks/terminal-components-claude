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
3. **The showcase keeps `FocusFallback::First` as its default**, the
   library's is `Nearest`. Every other page assumes focus returns to the
   sidebar when a page's control vanishes on navigation; `Nearest` is one
   option away and is what the Focus & navigation page demonstrates.
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
   defaults, the ring, the fallback and focus requests.

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
   Follow-up, not built: an overflow menu. Items that do not fit are not
   drawn, so an app gives the toolbar its `width()`; a `»` menu of the
   hidden items (as `Tabs` has `+N`) belongs with the S5 overflow work.
   The States grid has no overflow row for that reason.

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
2. **Open: Panels and Scrolling still hint `g G Ends` by hand**
   (`pages/panels.rs`, `pages/scrolling.rs`). Those pages belong to SC22;
   it should take the hint from `NavKeys::ends_hint` the same way.
3. **Type-ahead precedence and timeout are left for SC29.**
   `TypeAheadPrecedence` and the `type_ahead_timeout` builders are
   allowlisted to SC29, which shows the Cursor/RowSource defaults together
   on the Scrolling page.
