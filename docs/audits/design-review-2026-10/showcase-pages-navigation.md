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
