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
