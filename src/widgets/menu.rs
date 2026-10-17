//! Anchored menus: a context menu popover that opens at a position or under
//! an anchor rectangle, and a menu bar whose labels open the same popover
//! beneath them. Rows are commands (a solid `highlight` fill on the cursor,
//! hover lifts); shortcuts sit right-aligned in the muted tone; danger rows (soft rose, deep red under the cursor)
//! use the error tone; disabled rows are faint and cannot be chosen.
//!
//! A menu's body is an [`OptionList`] (plan S12): the cursor, the scroll
//! offset, the wheel, hover, key navigation and type-ahead, and the popup
//! policy live there, once for every popup list. The menu keeps what is
//! its own: the frame and title, the ▲/▼ overflow markers, separators,
//! shortcuts and accelerators, danger rows and the press-drag-release
//! gestures.

use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Modifier, Style};

use crate::core::event::{Key, Outcome};

use crate::core::hints::{HasHints, HintCx, nav_hints};
use crate::core::hit::Axis;
use crate::core::hit::ReleaseFrom;
use crate::core::id::{ItemKey, Part, PartRef, WidgetId};
use crate::core::keymap::unassigned_chord;
use crate::core::keymap::{Nav, NavKeys};
use crate::core::list::{Cursor, Moved, Navigable, RowFn, RowSource};
use crate::core::scroll::Scrollable;
use crate::theme::{CursorStyle, Kind, Plane};
use crate::ui::ctx::{RenderCtx, fill};
use crate::ui::popup::{Dismiss, Place, backdrop_id, place_with};
use crate::ui::text::{truncate, width};
use crate::widgets::brand::Lockup;
use crate::widgets::keyhint::Hint;
use crate::widgets::option_list::{AsItem, Item, OptionList, WheelMode};
use ratatui::crossterm::event::KeyCode;

/// One command row of a menu. Its label, disabled flag and reason are an
/// [`Item`] ([`Self::item`]), so a menu's rows are an [`OptionList`]'s;
/// the shortcut, the danger tone and the separator are the menu's own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuItem {
    /// The row as the popup list sees it: `label`, `disabled`, `reason`.
    pub item: Item,
    /// Built at runtime when needed (a user-rebound key), hence `Cow`.
    pub shortcut: Option<std::borrow::Cow<'static, str>>,
    pub danger: bool,
    pub separator_after: bool,
}

impl AsItem for MenuItem {
    fn as_item(&self) -> &Item {
        &self.item
    }
}

impl MenuItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            item: Item::new(label),
            shortcut: None,
            danger: false,
            separator_after: false,
        }
    }
    pub fn shortcut(mut self, s: impl Into<std::borrow::Cow<'static, str>>) -> Self {
        self.shortcut = Some(s.into());
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.item.disabled = d;
        self
    }
    /// Disable the item and say why. An unavailable command is shown as
    /// unavailable up front instead of failing after it is chosen: the
    /// row keeps its dimmed shortcut, and the reason shows in the menu's
    /// status row (its bottom border) while the cursor or the pointer is
    /// on the row, so it never widens the menu (SI21/SV15).
    pub fn reason(mut self, reason: impl Into<String>) -> Self {
        self.item = self.item.reason(reason);
        self
    }
    pub fn danger(mut self) -> Self {
        self.danger = true;
        self
    }
    /// [`Self::danger`] decided at runtime.
    pub fn danger_if(mut self, on: bool) -> Self {
        self.danger = on;
        self
    }
    pub fn separator(mut self) -> Self {
        self.separator_after = true;
        self
    }
    /// [`Self::separator`] decided at runtime.
    pub fn separator_if(mut self, on: bool) -> Self {
        self.separator_after = on;
        self
    }
}

/// What an open menu does with a key it has no use for (no row, no
/// accelerator, no navigation) (K14).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UnknownKey {
    /// Close the menu without choosing and let the key reach the page
    /// (macOS menus: a shortcut typed with a menu open still works).
    #[default]
    CloseAndPass,
    /// Keep the menu open and swallow the key (the old behaviour).
    Swallow,
}

/// Menus are placed like every other popup: this is
/// [`crate::ui::popup::Placement`], re-exported so `menu::Placement`
/// paths keep compiling.
pub use crate::ui::popup::Placement;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    Chosen(usize),
    Dismissed,
}

#[derive(Debug, Clone)]
pub struct ContextMenu {
    pub id: WidgetId,
    pub items: Vec<MenuItem>,
    /// The menu's body ([`OptionList`]): the cursor (`list.cursor`, which
    /// wraps by default as macOS menus do), the scroll offset
    /// (`list.scroll`; items and separators count one line each), the
    /// wheel (default [`WheelMode::Viewport`]: it scrolls, the cursor
    /// stays), hover (the pointer moves the one highlight, which clears
    /// when the pointer leaves the rows), the keys and type-ahead, and the
    /// popup policy: `list.dismiss` (Esc, a click outside),
    /// `list.placement`, `list.fit` (default [`Place::MENU`]: one free
    /// column from the left and right edges, given up when the menu would
    /// not fit otherwise, none above or below, and full height kept when
    /// neither side of the anchor has room) and `list.max_rows` (default
    /// `u16::MAX`: as tall as its items, up to the screen; a menu taller
    /// than its room always scrolls, so every item stays reachable).
    pub list: OptionList,
    pub anchor: Rect,
    /// Popover rectangle from the last render (frame included).
    pub area: Rect,
    /// Optional title row above the items (e.g. what the menu is about).
    pub title: Option<String>,
    /// The last action, kept for the stack while the menu is a
    /// [`crate::ui::layer::Layer`].
    pub(crate) pending: Option<MenuAction>,
    /// While open, the menu owns the keyboard, so the page under it shows
    /// no focus (see [`ContextMenu::hides_page_focus`]). Default on.
    pub hide_owner_focus: bool,
    /// How a label cut by a narrow screen reveals its full text on hover
    /// (see [`crate::ui::reveal`]); `None` follows [`RenderCtx::reveal`].
    pub reveal: Option<crate::ui::reveal::Reveal>,
    /// Narrowest label column in cells (the frame and gutters add 6).
    /// Default 8.
    pub min_width: u16,
    /// A control whose rectangle this frame becomes the anchor when it is
    /// registered (a keyboard-opened context menu follows its cursor row);
    /// `anchor` is the fallback. See [`ContextMenu::anchor_control`].
    pub anchor_to: Option<WidgetId>,
    /// What a key the menu has no use for does. Default
    /// [`UnknownKey::CloseAndPass`]. A row's one-character shortcut (the
    /// `a` shown beside "Select all") always chooses that row: the menu
    /// does what it advertises.
    pub unknown_key: UnknownKey,
    /// Press, drag, release: a secondary release over an enabled row
    /// chooses it when the pointer moved at least one cell from the
    /// secondary press that opened the menu (macOS and Windows context
    /// menus). The release of a plain right-click never chooses. Default
    /// on. Apps deliver it through
    /// [`PointerState::on_secondary_up`](crate::core::pointer::PointerState::on_secondary_up).
    pub release_selects: bool,
    /// Primary presses that may complete on a row (a menu-bar title:
    /// press the title, drag onto a row, release). Set by [`MenuBar`].
    release_from: Vec<WidgetId>,
    /// The tick the last hover auto-scroll ran on.
    scrolled_tick: Option<u64>,
}

/// The footer while the menu is open: how the cursor moves (its own
/// [`OptionList::nav`], `Type to jump` under type-ahead), `Enter Choose`,
/// and `Esc Close` when its dismiss policy lets Esc close it.
impl HasHints for ContextMenu {
    fn hints(&self, _: &HintCx) -> Vec<Hint> {
        let mut v = nav_hints(&self.list.nav, self.list.type_ahead, false);
        v.push(Hint::new("Enter", "Choose"));
        if self.list.dismiss.escape {
            v.push(Hint::new("Esc", "Close"));
        }
        v
    }
}

impl ContextMenu {
    /// See [`Self::release_selects`].
    pub fn release_selects(mut self, on: bool) -> Self {
        self.release_selects = on;
        self
    }

    /// The ▲ marker above a scrolled menu: hovering it scrolls up one row
    /// per tick, a click pages up.
    pub fn more_above_id(&self) -> WidgetId {
        self.id.part(PartRef::OVERFLOW_BEFORE)
    }

    /// The ▼ marker below a menu with more rows: hovering it scrolls down
    /// one row per tick, a click pages down.
    pub fn more_below_id(&self) -> WidgetId {
        self.id.part(PartRef::OVERFLOW_AFTER)
    }

    /// The row highlighted as the cursor, if any: none after the pointer
    /// left the rows it was moving over.
    pub fn highlighted(&self) -> Option<usize> {
        self.list.highlighted().filter(|_| !self.items.is_empty())
    }

    /// A click on a ▲/▼ marker pages the menu; `true` when `id` is one.
    fn page_marker(&mut self, id: WidgetId) -> bool {
        let page = self.viewport().max(1) as isize;
        if id == self.more_above_id() {
            self.list.scroll.scroll_by(-page);
            true
        } else if id == self.more_below_id() {
            self.list.scroll.scroll_by(page);
            true
        } else {
            false
        }
    }

    /// See [`OptionList::max_rows`]: most rows (items and separators)
    /// shown before the menu scrolls.
    pub fn max_rows(mut self, rows: u16) -> Self {
        self.list.max_rows = rows;
        self
    }

    /// Row of item `i` among the menu's rows (separators take a row).
    fn line_of(&self, i: usize) -> usize {
        i + self.items[..i.min(self.items.len())]
            .iter()
            .filter(|it| it.separator_after)
            .count()
    }

    fn lines(&self) -> usize {
        self.line_of(self.items.len())
    }

    /// Rows of items the last render had room for.
    fn viewport(&self) -> usize {
        let title = usize::from(self.title.is_some());
        (self.area.height as usize).saturating_sub(2 + title)
    }

    /// Narrowest label column in cells. Default 8.
    pub fn min_width(mut self, cols: u16) -> Self {
        self.min_width = cols;
        self
    }

    pub fn new(id: WidgetId, items: Vec<MenuItem>) -> Self {
        let cursor = items.iter().position(|i| !i.item.disabled).unwrap_or(0);
        let mut list = OptionList::new(id)
            .look(CursorStyle::Fill)
            .max_rows(u16::MAX)
            .fit(Place::MENU);
        // disabled rows are reachable by the keyboard (APG, SI21): the
        // cursor on one shows why it is unavailable; it never activates
        list.cursor = Cursor::new(cursor).wrap(true).skip_disabled(false);
        // a menu brings a row in exactly: no context row beyond it
        list.scroll.context = 0;
        list.set_gaps(items.iter().map(|it| it.separator_after));
        Self {
            id,
            items,
            list,
            anchor: Rect::ZERO,
            area: Rect::ZERO,
            title: None,
            pending: None,
            hide_owner_focus: true,
            reveal: None,
            min_width: 8,
            anchor_to: None,
            unknown_key: UnknownKey::CloseAndPass,
            release_selects: true,
            release_from: Vec::new(),
            scrolled_tick: None,
        }
    }

    /// See [`OptionList::nav`]. PageUp/PageDown move by the rows shown.
    pub fn nav(mut self, nav: NavKeys) -> Self {
        self.list.nav = nav;
        self
    }

    /// Cells kept free between the menu and the screen's left and right
    /// edges (`list.fit.margin`). Default 1.
    pub fn edge_margin(mut self, cells: u16) -> Self {
        self.list.fit.margin = cells;
        self
    }

    /// ↑ on the first item lands on the last and ↓ on the last on the
    /// first. Default on (macOS menus wrap).
    pub fn wrap(mut self, on: bool) -> Self {
        self.list.cursor.wrap = on;
        self
    }

    /// The pause after which the next letter starts a new type-ahead
    /// prefix. Default [`crate::core::typeahead::TypeAhead::DEFAULT_TIMEOUT`]
    /// (1 s; APG suggests 500–1000 ms).
    pub fn type_ahead_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.list = self.list.type_ahead_timeout(timeout);
        self
    }

    /// Typing a prefix jumps to the first enabled row whose label starts
    /// with it (macOS menus and lists). Default from the
    /// [`crate::core::list::NavProfile`]: on under `Desktop`, off under
    /// `Vim`.
    pub fn type_ahead(mut self, on: bool) -> Self {
        self.list.type_ahead = on;
        self
    }

    /// Keyboard steps pass over disabled rows. Default off: a disabled
    /// row is reachable, so the cursor on it can say why it is unavailable
    /// (APG menus; SI21/SV15). Enter, a click and its shortcut never
    /// activate it either way. Type-ahead always skips it.
    pub fn skip_disabled(mut self, on: bool) -> Self {
        self.list.cursor.skip_disabled = on;
        self
    }

    /// What the wheel does over the menu ([`OptionList::wheel`]). Default
    /// [`WheelMode::Viewport`]: it scrolls and the cursor stays.
    pub fn wheel_mode(mut self, mode: WheelMode) -> Self {
        self.list.wheel = mode;
        self
    }

    /// See [`Self::list`] (`fit`).
    pub fn fit(mut self, fit: Place) -> Self {
        self.list.fit = fit;
        self
    }

    /// How the menu closes without a choice (Esc, a click outside).
    pub fn dismiss(mut self, dismiss: Dismiss) -> Self {
        self.list.dismiss = dismiss;
        self
    }

    pub fn hide_owner_focus(mut self, on: bool) -> Self {
        self.hide_owner_focus = on;
        self
    }

    /// Whether the page under this open menu should draw without focus:
    /// pass it to [`RenderCtx::page_under_layer`] at frame start.
    pub fn hides_page_focus(&self) -> bool {
        self.hide_owner_focus
    }

    pub fn anchor(mut self, anchor: Rect, placement: Placement) -> Self {
        self.anchor = anchor;
        self.list.placement = placement;
        self
    }

    /// Anchor at a pointer position (a context click).
    pub fn at(self, pos: Position) -> Self {
        self.anchor(Rect::new(pos.x, pos.y, 1, 1), Placement::Below)
    }

    /// Anchor below the control `id` as drawn this frame (a cursor row,
    /// a cell, a tab), falling back to `fallback` when it is not drawn
    /// before the menu renders.
    pub fn anchor_control(mut self, id: WidgetId, fallback: Rect) -> Self {
        self.anchor_to = Some(id);
        self.anchor(fallback, Placement::Below)
    }

    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }

    pub fn row_id(&self, i: usize) -> WidgetId {
        self.list.row_id(i)
    }

    /// The row `id` is: the inverse the id-taking methods
    /// ([`Self::on_click`]) need until S15 hands them the routed part.
    fn row_of(&self, id: WidgetId) -> Option<usize> {
        self.list.row_of(id, self.items.len())
    }

    /// Whether `id` is the menu, its backdrop, a page marker or a row: how
    /// a container's id-taking methods (S15) tell its open menu's clicks
    /// from its own. Routed events use [`PointerEvent::on`] instead.
    ///
    /// [`PointerEvent::on`]: crate::core::pointer::PointerEvent::on
    pub(crate) fn contains_id(&self, id: WidgetId) -> bool {
        id == self.id
            || id == backdrop_id(self.id)
            || id == self.more_above_id()
            || id == self.more_below_id()
            || self.row_of(id).is_some()
    }

    /// Frame-inclusive size.
    pub fn size(&self) -> (u16, u16) {
        let label_w = self
            .items
            .iter()
            .map(|i| {
                width(&i.item.label) as u16
                    + i.shortcut
                        .as_deref()
                        .map(|s| width(s) as u16 + 3)
                        .unwrap_or(0)
            })
            .max()
            .unwrap_or(4);
        let title_w = self.title.as_ref().map(|t| width(t) as u16).unwrap_or(0);
        // gutter + marker gap + label + 2 inset + frame
        let w = label_w.max(title_w).max(self.min_width) + 6;
        let rows = (self.lines() as u16).min(self.list.max_rows.max(1));
        let title_rows = if self.title.is_some() { 1 } else { 0 };
        let h = rows + title_rows + 2;
        (w, h)
    }

    fn placed(&self, screen: Rect) -> Rect {
        let (w, h) = self.size();
        place_with(
            screen,
            self.anchor,
            w,
            h,
            self.list.placement,
            self.list.fit,
        )
    }

    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<MenuAction>) {
        if unassigned_chord::<()>(key, &[]) {
            // an unassigned modified chord never performs a plain action
            return (Outcome::Ignored, None);
        }
        // with no row highlighted (the pointer left the rows), Enter has
        // nothing to choose; any other key takes the cursor back
        if self.highlighted().is_none() && matches!(key.code, KeyCode::Enter | KeyCode::Char(' ')) {
            return (Outcome::Consumed, None);
        }
        // a row's one-character shortcut chooses it (K14): the menu does
        // what it advertises
        if let KeyCode::Char(c) = key.code
            && let Some(i) = self.accelerator(c)
        {
            self.list.select(i, &self.items[..]);
            return (Outcome::Changed, Some(MenuAction::Chosen(i)));
        }
        match self.list.key(key, &self.items[..]) {
            Moved::To(_) => return (Outcome::Changed, None),
            Moved::Stayed(_) => return (Outcome::Consumed, None),
            // ←/→ belong to a menu bar, not to the menu
            Moved::Ignored if self.list.nav.action(key).is_some() => {
                return (Outcome::Consumed, None);
            }
            Moved::Ignored => {}
        }
        match key.code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                let i = self.list.cursor.index;
                if self.items.get(i).is_some_and(|it| !it.item.disabled) {
                    (Outcome::Changed, Some(MenuAction::Chosen(i)))
                } else {
                    (Outcome::Consumed, None)
                }
            }
            KeyCode::Esc if self.list.dismiss.escape => {
                (Outcome::Changed, Some(MenuAction::Dismissed))
            }
            KeyCode::Esc | KeyCode::Tab | KeyCode::BackTab => (Outcome::Consumed, None),
            // a key the menu has no use for closes it and reaches the page
            // (`Ignored`), unless the owner keeps the old swallow
            _ => match self.unknown_key {
                UnknownKey::CloseAndPass => (Outcome::Ignored, Some(MenuAction::Dismissed)),
                UnknownKey::Swallow => (Outcome::Consumed, None),
            },
        }
    }

    /// The enabled row whose shortcut is exactly the character `c`.
    fn accelerator(&self, c: char) -> Option<usize> {
        self.items.iter().position(|it| {
            !it.item.disabled
                && it
                    .shortcut
                    .as_deref()
                    .is_some_and(|s| s.chars().eq(std::iter::once(c)))
        })
    }

    /// See [`Self::unknown_key`].
    pub fn unknown_key(mut self, unknown: UnknownKey) -> Self {
        self.unknown_key = unknown;
        self
    }

    /// A completed click on `id`: a row chooses it, anywhere else dismisses
    /// (when the [`Dismiss`] policy allows an outside click).
    pub fn on_click(&mut self, id: WidgetId) -> Option<MenuAction> {
        if self.page_marker(id) {
            return None;
        }
        match self.row_of(id) {
            Some(i) => self.choose(i),
            None if id == self.id => None,
            None => self.on_click_outside(),
        }
    }

    /// The pointer chose row `i`: an enabled row is chosen, a disabled one
    /// never is.
    fn choose(&mut self, i: usize) -> Option<MenuAction> {
        if self.items.get(i).is_none_or(|it| it.item.disabled) {
            return None;
        }
        self.list.select(i, &self.items[..]);
        Some(MenuAction::Chosen(i))
    }

    pub fn on_click_outside(&mut self) -> Option<MenuAction> {
        self.list
            .dismiss
            .outside_click
            .then_some(MenuAction::Dismissed)
    }

    /// Place the menu inside `screen` and queue it as an overlay
    /// ([`RenderCtx::overlay`]): it lands above everything drawn after it,
    /// and a click outside reaches it through its backdrop.
    pub fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        self.prepare(screen.intersection(*buf.area()), ctx);
        let menu = self.clone();
        ctx.overlay(move |buf, ctx| {
            ctx.hits.push_barrier();
            crate::ui::popup::backdrop(ctx, menu.id, menu.list.dismiss, *buf.area());
            menu.paint(buf, ctx);
        });
    }

    /// The rows' area inside the frame (below the title) of `area`.
    fn rows_inner(&self, area: Rect) -> Rect {
        let title = u16::from(self.title.is_some());
        Rect::new(
            area.x + 1,
            area.y + 1 + title,
            area.width.saturating_sub(2),
            area.height.saturating_sub(2 + title),
        )
    }

    /// Resolve the popover rectangle and let the pointer move the cursor.
    /// State changes happen here, so [`ContextMenu::paint`] can run later
    /// (as an overlay) on a copy.
    fn prepare(&mut self, screen: Rect, ctx: &RenderCtx) {
        if let Some(r) = self.anchor_to.and_then(|id| ctx.hits.area_of(id)) {
            self.anchor = r;
        }
        self.area = self.placed(screen).intersection(screen);
        // a menu taller than its room scrolls: the list keeps a moved
        // cursor in view and the window inside the rows; the pointer moves
        // the one highlight
        self.list
            .set_gaps(self.items.iter().map(|it| it.separator_after));
        let inner = self.rows_inner(self.area);
        self.list.layout(inner, &self.items[..], ctx);
        // resting on a ▲/▼ marker scrolls one row per tick (macOS)
        let hover = ctx.interaction.live_hover();
        let tick = ctx.interaction.tick;
        let step = if hover == Some(self.more_above_id()) {
            -1
        } else if hover == Some(self.more_below_id()) {
            1
        } else {
            0
        };
        if step == 0 {
            self.scrolled_tick = None;
        } else if self.scrolled_tick != Some(tick) {
            self.scrolled_tick = Some(tick);
            self.list.scroll.scroll_by(step);
        }
    }

    fn paint(&self, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let area = self.area.intersection(*buf.area());
        if area.is_empty() {
            return;
        }
        // the popover plane: the strongest neutral, so a menu reads as a
        // floating command list over the page
        let bg = t.popover;
        let plane = Plane::Popover;
        fill(buf, area, Style::new().bg(bg));
        let block = ratatui::widgets::Block::new()
            .borders(ratatui::widgets::Borders::ALL)
            .border_set(t.glyphs.border)
            .border_style(Style::new().fg(t.frame_on(bg)).bg(bg));
        ratatui::widgets::Widget::render(block, area, buf);
        // the popover itself is a hit target so clicks inside its frame
        // never fall through to what is underneath
        ctx.clickable(self.id, area);
        let inner = Rect::new(
            area.x + 1,
            area.y + 1,
            area.width.saturating_sub(2),
            area.height.saturating_sub(2),
        );
        if let Some(title) = &self.title {
            buf.set_string(
                inner.x + 2,
                inner.y,
                truncate(title, inner.width.saturating_sub(3) as usize),
                t.muted().bg(bg),
            );
        }
        let rows = self.rows_inner(area);
        let marker_row = |y: u16| Rect::new(area.x + 1, y, area.width.saturating_sub(2), 1);
        let marker = |ctx: &RenderCtx, id: WidgetId| {
            // rest, hover and press read apart (a pressed marker pages)
            let s = ctx.state(id);
            if s.pressed {
                t.primary().bg(bg).add_modifier(Modifier::BOLD)
            } else if s.hovered {
                t.primary().bg(bg)
            } else {
                t.muted().bg(bg)
            }
        };
        if self.list.scroll.offset > 0 {
            let id = self.more_above_id();
            buf.set_string(
                area.right().saturating_sub(3),
                area.y,
                t.glyphs.more_above,
                marker(ctx, id),
            );
            ctx.register_part(self.id, PartRef::OVERFLOW_BEFORE, marker_row(area.y));
        }
        let cursor = self.highlighted();
        // an open menu is always the active control: its cursor row carries
        // the selection tint whoever holds keyboard focus
        for (k, line) in self.list.visible_lines() {
            if k >= rows.height {
                break;
            }
            let y = rows.y + k;
            let Some(i) = line else {
                // a separator
                for x in rows.x + 1..rows.right().saturating_sub(1) {
                    buf.set_string(
                        x,
                        y,
                        t.glyphs.bar_empty,
                        Style::new().fg(t.rule_on(bg)).bg(bg),
                    );
                }
                continue;
            };
            let Some(item) = self.items.get(i) else {
                break;
            };
            let rid = self.row_id(i);
            let mut s = ctx.child_state(self.id, rid, cursor == Some(i));
            s.focused = cursor == Some(i);
            s.disabled = item.item.disabled;
            s.error = item.danger;
            let row = Rect::new(rows.x, y, rows.width, 1);
            // one highlight only: the cursor row is the highlight fill (a
            // menu row is a command, not a focus stop, so it draws no `▎`
            // bar and never the accent); hover and press walk the plane's
            // ladder. A destructive row is soft rose at rest and takes the
            // deep red cursor fill: the tone of the plane, not the alarm red
            let it = t.interactive(Kind::CursorRow(CursorStyle::Fill), s, plane);
            let mut st = it.style;
            if item.danger && !it.on_fill && !item.item.disabled {
                st = st.fg(t.legible(t.error_soft, st.bg.unwrap_or(bg)));
            }
            fill(buf, row, st);
            let label_w = rows.width.saturating_sub(3) as usize;
            buf.set_string(row.x + 2, y, truncate(&item.item.label, label_w), st);
            if !item.item.disabled {
                crate::ui::reveal::truncated(
                    ctx,
                    self.reveal,
                    rid,
                    Rect::new(row.x + 2, y, label_w as u16, 1),
                    &item.item.label,
                );
            }
            if let Some(sc) = item.shortcut.as_deref() {
                let sw = width(sc) as u16;
                if sw + width(&item.item.label) as u16 + 5 <= rows.width {
                    // a shortcut reads on the cursor fill and at rest alike;
                    // a disabled row keeps it, dimmed in the row's disabled
                    // ink
                    let scs = if item.item.disabled {
                        st
                    } else {
                        let ink = t.row_ink(&it, t.text_secondary);
                        st.fg(t.legible(ink, st.bg.unwrap_or(bg)))
                            .remove_modifier(Modifier::BOLD)
                    };
                    buf.set_string(row.right().saturating_sub(sw + 1), y, sc, scs);
                }
            }
            ctx.register_part(self.id, OptionList::row_part(i), row);
            if !item.item.disabled {
                // press-drag-release: a release here may complete a press
                // that started elsewhere
                if self.release_selects {
                    ctx.hits.accept_release(rid, ReleaseFrom::Secondary);
                }
                for &from in &self.release_from {
                    ctx.hits.accept_release(rid, ReleaseFrom::Press(from));
                }
            }
        }
        let more_below = self.list.more_below(self.items.len());
        if more_below {
            let id = self.more_below_id();
            let yb = area.bottom().saturating_sub(1);
            buf.set_string(
                area.right().saturating_sub(3),
                yb,
                t.glyphs.more_below,
                marker(ctx, id),
            );
            ctx.register_part(self.id, PartRef::OVERFLOW_AFTER, marker_row(yb));
        }
        // the status row: a disabled row under the cursor or the pointer
        // says why, dimmed in the bottom border, cut to fit, never widening
        // the menu and clear of the ▼ marker (SI21/SV15)
        if let Some(reason) = self.list.status(&self.items) {
            let x = area.x + 2;
            let end = area.right().saturating_sub(if more_below { 4 } else { 2 });
            let avail = end.saturating_sub(x) as usize;
            if avail >= 3 {
                let text = format!(" {} ", truncate(reason, avail - 2));
                buf.set_string(
                    x,
                    area.bottom().saturating_sub(1),
                    text,
                    Style::new().fg(t.legible(t.text_muted, bg)).bg(bg),
                );
            }
        }
    }
}
// ------------------------------------------------------------------ bar

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuBarAction {
    /// A menu opened (index). With [`Overflow::Chevron`], `labels.len()`
    /// is the chevron's menu of hidden titles.
    Opened(usize),
    /// An item was chosen: (menu, item).
    Chosen(usize, usize),
    Closed,
    /// The brand lockup was activated.
    Brand,
}

/// When a press on a [`MenuBar`] title opens its menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OpenOn {
    /// On button down (macOS, Windows and GNOME menu bars): the menu is
    /// open while the button is still held, so the press can drag onto a
    /// row and release to choose it ([`MenuBar::press_drag`]). Pressing the
    /// open title closes it.
    #[default]
    Press,
    /// On a completed click (button released over the title).
    Release,
}

/// What a [`MenuBar`] does with titles that do not fit its row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Overflow {
    /// They are not drawn and cannot be reached with the pointer.
    #[default]
    Hide,
    /// They collapse into one chevron title
    /// ([`Glyphs::more_titles`](crate::theme::Glyphs::more_titles)) whose
    /// menu lists them; choosing one opens that title's menu under the
    /// chevron. The chevron takes part in the arrow keys like any title.
    Chevron,
}

#[derive(Debug, Clone)]
pub struct MenuBar {
    pub id: WidgetId,
    pub labels: Vec<String>,
    pub menus: Vec<Vec<MenuItem>>,
    pub brand: Option<Lockup>,
    /// Cursor among labels (keyboard); the open menu follows it.
    pub cursor: usize,
    pub open: Option<ContextMenu>,
    pub areas: Vec<Rect>,
    pub brand_area: Rect,
    /// How an open menu closes without a choice. A click outside lands on
    /// the bar's backdrop ([`backdrop_id`]); the bar titles stay live above
    /// it, so pointing at another title switches menus as on macOS.
    pub dismiss: Dismiss,
    /// With a menu open, pointing at another title opens that menu (the
    /// macOS and Windows menu-bar convention). Off: only a click or the
    /// arrow keys switch menus. It needs any-motion mouse tracking: while
    /// the terminal has reported no motion without a button held
    /// ([`Interaction::motion_unreported`](crate::ui::ctx::Interaction::motion_unreported)),
    /// only a press dragged across the titles switches, and a click or
    /// ←/→ (APG menubar) switch otherwise.
    pub hover_switch: bool,
    /// ←/→ past the last title lands on the first and back. Default on.
    pub wrap: bool,
    /// With a menu open the keyboard is in the menu, so the page under it
    /// shows no focus (see [`MenuBar::hides_page_focus`]). Default on.
    pub hide_owner_focus: bool,
    /// Which keys move between titles and through an open menu (default
    /// [`NavKeys::default`]; the open menu inherits it).
    pub nav: NavKeys,
    /// Titles that do not fit. Default [`Overflow::Hide`].
    pub overflow: Overflow,
    /// The open menu takes type-ahead ([`ContextMenu::type_ahead`]).
    /// Default from the [`crate::core::list::NavProfile`].
    pub type_ahead: bool,
    /// The open menu's [`ContextMenu::type_ahead_timeout`]. Default
    /// [`crate::core::typeahead::TypeAhead::DEFAULT_TIMEOUT`].
    pub type_ahead_timeout: std::time::Duration,
    /// The chevron title as last drawn ([`Overflow::Chevron`] with hidden
    /// titles), else `Rect::ZERO`.
    pub more_area: Rect,
    /// When a press on a title opens its menu. Default [`OpenOn::Press`].
    /// Owners route presses through [`MenuBar::on_press`]; without that
    /// the bar opens on the completed click.
    pub open_on: OpenOn,
    /// Press a title, drag onto a row, release: the release chooses the
    /// row (the desktop menu-bar gesture). A release back on the title
    /// keeps the menu open. Default on.
    pub press_drag: bool,
    /// The title whose press [`MenuBar::on_press`] already acted on: its
    /// click (the release) does nothing more.
    press_handled: Option<WidgetId>,
}

/// The footer while a menu of the bar is open: `←→ Menu` (titles), the
/// open menu's moves, `Enter Choose`, `Esc Close`. Closed, none: a focused
/// bar's keys are the page's to name.
impl HasHints for MenuBar {
    fn hints(&self, _: &HintCx) -> Vec<Hint> {
        if !self.is_open() {
            return vec![];
        }
        let mut v = vec![Hint::new("←→", "Menu")];
        v.extend(nav_hints(&self.nav, self.type_ahead, false));
        v.extend([Hint::new("Enter", "Choose"), Hint::new("Esc", "Close")]);
        v
    }
}

impl MenuBar {
    /// See [`Self::nav`].
    pub fn nav(mut self, nav: NavKeys) -> Self {
        self.nav = nav;
        self
    }

    pub fn new(id: WidgetId, entries: Vec<(&str, Vec<MenuItem>)>) -> Self {
        Self {
            id,
            labels: entries.iter().map(|(l, _)| (*l).to_owned()).collect(),
            menus: entries.into_iter().map(|(_, m)| m).collect(),
            brand: None,
            cursor: 0,
            open: None,
            areas: vec![],
            brand_area: Rect::ZERO,
            dismiss: Dismiss::LIGHT,
            hover_switch: true,
            wrap: true,
            hide_owner_focus: true,
            nav: NavKeys::default(),
            overflow: Overflow::Hide,
            more_area: Rect::ZERO,
            type_ahead: crate::core::list::NavProfile::current().type_ahead(),
            type_ahead_timeout: crate::core::typeahead::TypeAhead::DEFAULT_TIMEOUT,
            open_on: OpenOn::Press,
            press_drag: true,
            press_handled: None,
        }
    }

    /// See [`Self::open_on`].
    pub fn open_on(mut self, open_on: OpenOn) -> Self {
        self.open_on = open_on;
        self
    }

    /// See [`Self::press_drag`].
    pub fn press_drag(mut self, on: bool) -> Self {
        self.press_drag = on;
        self
    }

    /// A button down on `id`. Under [`OpenOn::Press`] a title opens its
    /// menu right away (or closes it when it is the open one); the click
    /// that follows does nothing more. Anything else is left to
    /// [`Self::on_click`].
    pub fn on_press(&mut self, id: WidgetId) -> (Outcome, Option<MenuBarAction>) {
        self.press_handled = None;
        if self.open_on != OpenOn::Press {
            return (Outcome::Ignored, None);
        }
        let Some(i) = self.title_at(id) else {
            return (Outcome::Ignored, None);
        };
        self.press_handled = Some(id);
        if self.open_index() == Some(i) {
            self.close();
            return (Outcome::Changed, Some(MenuBarAction::Closed));
        }
        self.open_menu(i);
        (Outcome::Changed, Some(MenuBarAction::Opened(i)))
    }

    /// See [`Self::type_ahead`].
    /// See [`Self::type_ahead_timeout`].
    pub fn type_ahead_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.type_ahead_timeout = timeout;
        self
    }

    pub fn type_ahead(mut self, on: bool) -> Self {
        self.type_ahead = on;
        self
    }

    /// See [`Self::overflow`].
    pub fn overflow(mut self, overflow: Overflow) -> Self {
        self.overflow = overflow;
        self
    }

    /// The chevron title's id ([`Overflow::Chevron`]).
    pub fn more_id(&self) -> WidgetId {
        self.id.part(PartRef::of(Part::OVERFLOW))
    }

    /// The chevron is on screen: [`Overflow::Chevron`] hid some title.
    fn chevron(&self) -> bool {
        !self.more_area.is_empty()
    }

    /// Titles hidden behind the chevron as of the last render, in order.
    /// Empty unless [`Overflow::Chevron`] is on and some title did not fit.
    pub fn hidden(&self) -> Vec<usize> {
        if !self.chevron() {
            return vec![];
        }
        (0..self.labels.len())
            .filter(|&i| self.areas.get(i).is_none_or(|r| r.is_empty()))
            .collect()
    }

    /// The titles the arrow keys visit, in order: the shown titles, then
    /// the chevron (`labels.len()`).
    fn ring(&self) -> Vec<usize> {
        let n = self.labels.len();
        let mut v: Vec<usize> = (0..n)
            .filter(|&i| self.areas.get(i).is_some_and(|r| !r.is_empty()))
            .collect();
        v.push(n);
        v
    }

    /// The title `delta` steps from the cursor along [`Self::ring`]. A
    /// hidden title's menu, opened from the chevron, steps from the chevron.
    fn step(&self, delta: isize) -> usize {
        let ring = self.ring();
        let n = self.labels.len();
        let at = ring
            .iter()
            .position(|&i| i == self.cursor)
            .or_else(|| ring.iter().position(|&i| i == n))
            .unwrap_or(0);
        let titles = RowFn {
            len: ring.len(),
            enabled: &|_| true,
            label: &|_| "",
        };
        let to = self.title_cursor(at).step(delta, &titles).index();
        ring[to.unwrap_or(at)]
    }

    /// A horizontal cursor over the titles at `at`.
    fn title_cursor(&self, at: usize) -> Cursor {
        Cursor::new(at).wrap(self.wrap).axis(Axis::Horizontal)
    }

    /// The title `delta` steps from the cursor over every title.
    fn title_step(&self, delta: isize) -> usize {
        self.title_cursor(self.cursor)
            .step(delta, &self.labels[..])
            .index()
            .unwrap_or(self.cursor)
    }

    /// The title `id` names: an index, or `labels.len()` for the chevron.
    fn title_at(&self, id: WidgetId) -> Option<usize> {
        if self.chevron() && id == self.more_id() {
            return Some(self.labels.len());
        }
        self.id.item_of(id, Part::TAB, 0..self.labels.len())
    }

    /// The open menu belongs to the chevron: its own list, or a hidden
    /// title opened from it.
    fn chevron_open(&self) -> bool {
        self.open.is_some()
            && self.chevron()
            && (self.cursor == self.labels.len()
                || self.areas.get(self.cursor).is_none_or(|r| r.is_empty()))
    }

    pub fn hide_owner_focus(mut self, on: bool) -> Self {
        self.hide_owner_focus = on;
        self
    }

    /// Whether the page should draw without focus this frame (a menu is
    /// open and [`MenuBar::hide_owner_focus`] is on): pass it to
    /// [`RenderCtx::page_under_layer`] at frame start.
    pub fn hides_page_focus(&self) -> bool {
        self.hide_owner_focus && self.open.is_some()
    }

    pub fn dismiss(mut self, dismiss: Dismiss) -> Self {
        self.dismiss = dismiss;
        self
    }

    /// See [`Self::wrap`].
    pub fn wrap(mut self, on: bool) -> Self {
        self.wrap = on;
        self
    }

    pub fn hover_switch(mut self, on: bool) -> Self {
        self.hover_switch = on;
        self
    }

    pub fn brand(mut self, lockup: Lockup) -> Self {
        self.brand = Some(lockup);
        self
    }

    pub fn label_id(&self, i: usize) -> WidgetId {
        self.id.part(Self::title_part(i))
    }

    fn title_part(i: usize) -> PartRef {
        PartRef::item(Part::TAB, ItemKey::index(i))
    }

    pub fn brand_id(&self) -> WidgetId {
        self.id.sub("brand")
    }

    pub fn is_open(&self) -> bool {
        self.open.is_some()
    }

    pub fn open_index(&self) -> Option<usize> {
        let m = self.open.as_ref()?;
        if m.id == self.more_id().sub("menu") {
            return Some(self.labels.len());
        }
        (0..self.labels.len()).find(|i| m.id == self.label_id(*i).sub("menu"))
    }

    pub fn open_menu(&mut self, i: usize) {
        if i == self.labels.len() && self.chevron() {
            // the chevron: a menu of the hidden titles
            self.cursor = i;
            let items = self
                .hidden()
                .into_iter()
                .map(|h| MenuItem::new(self.labels[h].clone()))
                .collect();
            let menu = ContextMenu::new(self.more_id().sub("menu"), items)
                .anchor(self.more_area, Placement::Below)
                .nav(self.nav)
                .type_ahead(self.type_ahead)
                .type_ahead_timeout(self.type_ahead_timeout);
            self.open = Some(menu);
            return;
        }
        if i >= self.labels.len() {
            return;
        }
        self.cursor = i;
        let anchor = match self.areas.get(i).copied() {
            Some(r) if !r.is_empty() => r,
            // a hidden title opens under the chevron
            _ if self.chevron() => self.more_area,
            r => r.unwrap_or(Rect::new(0, 0, 1, 1)),
        };
        let menu = ContextMenu::new(self.label_id(i).sub("menu"), self.menus[i].clone())
            .anchor(anchor, Placement::Below)
            .nav(self.nav)
            .type_ahead(self.type_ahead)
            .type_ahead_timeout(self.type_ahead_timeout);
        self.open = Some(menu);
    }

    pub fn close(&mut self) {
        self.open = None;
    }

    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<MenuBarAction>) {
        let n = self.labels.len();
        if unassigned_chord::<()>(key, &[]) {
            // an unassigned modified chord never performs a plain action
            return (Outcome::Ignored, None);
        }
        // with type-ahead, letters typed into an open menu are a prefix,
        // never `h`/`l` to the next title
        let typing = self.type_ahead && self.open.is_some() && matches!(key.code, KeyCode::Char(_));
        let nav = if typing { None } else { self.nav.action(key) };
        let chevron = self.chevron();
        let step = match nav {
            Some(Nav::Left) => Some(-1),
            Some(Nav::Right) => Some(1),
            _ => None,
        };
        if chevron
            && self.open.is_some()
            && let Some(d) = step
        {
            let i = self.step(d);
            self.open_menu(i);
            return (Outcome::Changed, Some(MenuBarAction::Opened(i)));
        }
        let from_chevron = self.cursor == n && chevron;
        if let Some(menu) = self.open.as_mut() {
            match nav {
                Some(Nav::Left) if n > 1 => {
                    let i = self.title_step(-1);
                    self.open_menu(i);
                    return (Outcome::Changed, Some(MenuBarAction::Opened(i)));
                }
                Some(Nav::Right) if n > 1 => {
                    let i = self.title_step(1);
                    self.open_menu(i);
                    return (Outcome::Changed, Some(MenuBarAction::Opened(i)));
                }
                _ => {}
            }
            let (o, ev) = menu.on_key(key);
            return match ev {
                Some(MenuAction::Chosen(i)) if from_chevron => self.choose_hidden(i),
                Some(MenuAction::Chosen(i)) => {
                    let m = self.cursor;
                    self.close();
                    (Outcome::Changed, Some(MenuBarAction::Chosen(m, i)))
                }
                // a key the menu has no use for (K14): the bar closes and
                // the key goes on to the page
                Some(MenuAction::Dismissed) if o == Outcome::Ignored => {
                    self.close();
                    (Outcome::Ignored, Some(MenuBarAction::Closed))
                }
                Some(MenuAction::Dismissed) if !self.dismiss.escape => (Outcome::Consumed, None),
                Some(MenuAction::Dismissed) => {
                    self.close();
                    (Outcome::Changed, Some(MenuBarAction::Closed))
                }
                None => (o, None),
            };
        }
        match nav {
            Some(nav @ (Nav::Home | Nav::End)) if n > 0 => {
                // APG menubar: Home/End go to the first and the last title
                // (the chevron when titles are hidden behind it)
                self.cursor = if chevron {
                    let ring = self.ring();
                    if nav == Nav::Home {
                        ring[0]
                    } else {
                        ring[ring.len() - 1]
                    }
                } else {
                    self.title_cursor(self.cursor)
                        .apply(nav, &self.labels[..], 1)
                        .index()
                        .unwrap_or(self.cursor)
                };
                return (Outcome::Changed, None);
            }
            Some(Nav::Left) if chevron => {
                self.cursor = self.step(-1);
                return (Outcome::Changed, None);
            }
            Some(Nav::Right) if chevron => {
                self.cursor = self.step(1);
                return (Outcome::Changed, None);
            }
            Some(Nav::Left) if n > 0 => {
                self.cursor = self.title_step(-1);
                return (Outcome::Changed, None);
            }
            Some(Nav::Right) if n > 0 => {
                self.cursor = self.title_step(1);
                return (Outcome::Changed, None);
            }
            _ => {}
        }
        match key.code {
            _ if n > 0
                && (matches!(key.code, KeyCode::Enter | KeyCode::Char(' '))
                    || nav == Some(Nav::Down)) =>
            {
                let i = self.cursor;
                self.open_menu(i);
                (Outcome::Changed, Some(MenuBarAction::Opened(i)))
            }
            _ => (Outcome::Ignored, None),
        }
    }

    /// A completed click. Labels toggle their menu, rows choose, the brand
    /// fires `Brand`, anything else while open closes the menu (when the
    /// [`Dismiss`] policy allows; the click is consumed either way).
    pub fn on_click(&mut self, id: WidgetId) -> (Outcome, Option<MenuBarAction>) {
        // the press on this title already opened (or closed) its menu: the
        // release over it keeps that state
        let handled = self.press_handled.take();
        if handled.is_some() && handled == Some(id) && self.title_at(id).is_some() {
            return (Outcome::Consumed, None);
        }
        if self.open.is_some() && id == backdrop_id(self.id) {
            if !self.dismiss.outside_click {
                return (Outcome::Consumed, None);
            }
            self.close();
            return (Outcome::Changed, Some(MenuBarAction::Closed));
        }
        if id == self.brand_id() {
            self.close();
            return (Outcome::Changed, Some(MenuBarAction::Brand));
        }
        if let Some(i) = self.title_at(id) {
            if self.open_index() == Some(i) {
                self.close();
                return (Outcome::Changed, Some(MenuBarAction::Closed));
            }
            self.open_menu(i);
            return (Outcome::Changed, Some(MenuBarAction::Opened(i)));
        }
        let from_chevron = self.chevron() && self.cursor == self.labels.len();
        if let Some(menu) = self.open.as_mut() {
            if menu.page_marker(id) {
                return (Outcome::Changed, None);
            }
            return match menu.on_click(id) {
                Some(MenuAction::Chosen(item)) if from_chevron => self.choose_hidden(item),
                Some(MenuAction::Chosen(item)) => {
                    let m = self.cursor;
                    self.close();
                    (Outcome::Changed, Some(MenuBarAction::Chosen(m, item)))
                }
                Some(MenuAction::Dismissed) if !self.dismiss.outside_click => {
                    (Outcome::Consumed, None)
                }
                Some(MenuAction::Dismissed) => {
                    self.close();
                    (Outcome::Changed, Some(MenuBarAction::Closed))
                }
                None => (Outcome::Consumed, None),
            };
        }
        (Outcome::Ignored, None)
    }

    /// Item `i` of the chevron's menu: open that hidden title's menu.
    fn choose_hidden(&mut self, i: usize) -> (Outcome, Option<MenuBarAction>) {
        let Some(&title) = self.hidden().get(i) else {
            return (Outcome::Consumed, None);
        };
        self.open_menu(title);
        (Outcome::Changed, Some(MenuBarAction::Opened(title)))
    }

    /// Hovering another label while a menu is open switches to it. The bar
    /// applies this itself on every render, so owners need not call it.
    pub fn on_hover(&mut self, id: Option<WidgetId>) -> Outcome {
        if self.open.is_none() || !self.hover_switch {
            return Outcome::Ignored;
        }
        if let Some(id) = id
            && let Some(i) = self.title_at(id)
            && self.open_index() != Some(i)
            // a hidden title's menu belongs to the chevron: resting on the
            // chevron keeps it open
            && !(i == self.labels.len() && self.chevron_open())
        {
            self.open_menu(i);
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    /// Draw the bar row. An open menu is queued as an overlay
    /// ([`RenderCtx::overlay`]), so it sits above whatever is drawn after
    /// the bar with no extra call from the owner.
    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return;
        }
        // with a menu open, pointing at another title opens that one
        // (unless `hover_switch` is off). A terminal that reports no motion
        // without a button held never moves hover with the pointer, so
        // there only a press dragged across the titles switches; a click
        // and the arrow keys do the rest
        let dragging = ctx
            .interaction
            .pressed
            .is_some_and(|p| self.title_at(p).is_some());
        if !ctx.interaction.motion_unreported || dragging {
            self.on_hover(ctx.interaction.live_hover());
        }
        let t = ctx.theme;

        fill(buf, area, Style::new().bg(bg));
        ctx.control(self.id, area, false);
        let focused = ctx.interaction.focused(self.id);
        let plane = t.plane_of(bg);
        // titles and the brand carry one cell of padding of their own: the
        // text lands on the theme's chrome edge
        let mut x = area.x + t.spacing.chrome_edge.saturating_sub(1);
        if let Some(b) = &self.brand {
            let w = b.render_clickable(x, area.y, buf, ctx, self.brand_id());
            self.brand_area = Rect::new(x, area.y, w, 1);
            x += w + 2;
        }
        self.areas.clear();
        let open = self.open_index();
        let more_text = format!(" {} ", t.glyphs.more_titles);
        let more_w = width(&more_text) as u16;
        // with a chevron, titles stop where the chevron still fits after
        // them; without one, each title only has to fit itself
        let limit = if self.overflow == Overflow::Chevron && {
            let all: u16 = self
                .labels
                .iter()
                .map(|l| width(l) as u16 + 3)
                .sum::<u16>()
                .saturating_sub(1);
            x + all > area.right()
        } {
            area.right().saturating_sub(more_w + 1)
        } else {
            area.right()
        };
        let chevron_open = self.chevron_open();
        let mut hiding = false;
        for (i, label) in self.labels.iter().enumerate() {
            let lid = self.label_id(i);
            let text = format!(" {label} ");
            let w = width(&text) as u16;
            if hiding || x + w > limit {
                self.areas.push(Rect::ZERO);
                // a chevron keeps the order: once one title is hidden, every
                // title after it is too
                hiding = limit < area.right();
                continue;
            }
            let is_open = open == Some(i);
            let mut s = ctx.state(lid);
            s.focused = focused && self.cursor == i && open.is_none();
            s.selected = is_open;
            // the open title sits on the popover plane so label and dropdown
            // read as one shape; hover and press walk that plane's ladder
            let on = if is_open { Plane::Popover } else { plane };
            let it = t.interactive(Kind::Title, s, on);
            if s.focused {
                buf.set_string(
                    x.saturating_sub(1),
                    area.y,
                    it.gutter,
                    it.gutter_style.bg(bg),
                );
            }
            buf.set_string(x, area.y, &text, it.style);
            let r = Rect::new(x, area.y, w, 1);
            self.areas.push(r);
            ctx.register_part(self.id, Self::title_part(i), r);
            x += w + 1;
        }
        self.more_area = Rect::ZERO;
        if limit < area.right() && x + more_w <= area.right() {
            let n = self.labels.len();
            let mid = self.more_id();
            let mut s = ctx.state(mid);
            s.focused = focused && self.cursor == n && open.is_none();
            s.selected = chevron_open;
            let on = if chevron_open { Plane::Popover } else { plane };
            let it = t.interactive(Kind::Title, s, on);
            if s.focused {
                buf.set_string(
                    x.saturating_sub(1),
                    area.y,
                    it.gutter,
                    it.gutter_style.bg(bg),
                );
            }
            buf.set_string(x, area.y, &more_text, it.style);
            self.more_area = Rect::new(x, area.y, more_w, 1);
            ctx.register_part(self.id, PartRef::of(Part::OVERFLOW), self.more_area);
        }
        let Some(m) = self.open.as_mut() else {
            return;
        };
        // keep the popover under its (possibly re-laid-out) label, or under
        // the chevron for its own menu and the hidden titles'
        if let Some(i) = open {
            match self.areas.get(i) {
                Some(r) if !r.is_empty() => m.anchor = *r,
                _ if !self.more_area.is_empty() => m.anchor = self.more_area,
                Some(r) => m.anchor = *r,
                None => {}
            }
        }
        // press a title, drag onto a row, release: any title's press may
        // complete on a row (the drag can switch menus on the way)
        m.release_from = if self.press_drag {
            let mut from: Vec<WidgetId> = (0..self.labels.len())
                .filter(|&i| self.areas.get(i).is_some_and(|r| !r.is_empty()))
                .map(|i| self.id.part(Self::title_part(i)))
                .collect();
            if !self.more_area.is_empty() {
                from.push(self.id.part(PartRef::of(Part::OVERFLOW)));
            }
            from
        } else {
            Vec::new()
        };
        m.prepare(*buf.area(), ctx);
        let menu = m.clone();
        let (id, dismiss) = (self.id, self.dismiss);
        // the titles (and the brand) stay reachable above the backdrop: a
        // click or a hover on one of them is a menu-bar gesture, not an
        // outside click
        let mut titles: Vec<(PartRef, Rect)> = (0..self.labels.len())
            .map(|i| (Self::title_part(i), self.areas[i]))
            .collect();
        if !self.more_area.is_empty() {
            titles.push((PartRef::of(Part::OVERFLOW), self.more_area));
        }
        let brand = self
            .brand
            .is_some()
            .then(|| (self.brand_id(), self.brand_area));
        ctx.overlay(move |buf, ctx| {
            ctx.hits.push_barrier();
            crate::ui::popup::backdrop(ctx, id, dismiss, *buf.area());
            for (part, r) in titles {
                ctx.register_part(id, part, r);
            }
            if let Some((bid, r)) = brand {
                ctx.clickable(bid, r);
            }
            menu.paint(buf, ctx);
        });
    }
}

/// The menu's rows as the shared nav table sees them
/// (`tests/navigable.rs`), forwarded to its items.
impl RowSource for ContextMenu {
    fn len(&self) -> usize {
        self.items.len()
    }
    fn enabled(&self, i: usize) -> bool {
        self.items[..].enabled(i)
    }
    fn label(&self, i: usize) -> &str {
        &self.items[i].item.label
    }
}

/// The list's cursor.
impl Navigable for ContextMenu {
    fn cursor(&self) -> Cursor {
        self.list.cursor
    }
}

/// The titles (the chevron is not a row).
impl RowSource for MenuBar {
    fn len(&self) -> usize {
        self.labels.len()
    }
    fn label(&self, i: usize) -> &str {
        &self.labels[i]
    }
}

impl Navigable for MenuBar {
    fn cursor(&self) -> Cursor {
        self.title_cursor(self.cursor)
    }
}

/// The menu's pointer gestures, routed by the event's part: a completed
/// click on a row chooses it (a disabled row never), on a ▲/▼ marker
/// pages, on the backdrop dismisses per its dismiss policy; a press, a
/// drag or a secondary click on it is consumed (the release chooses); the
/// wheel goes through its [`OptionList`]'s shared path
/// ([`Scrollable`](crate::core::scroll::Scrollable), per
/// [`OptionList::wheel`]; a sideways notch is consumed).
impl crate::core::pointer::MouseTarget for ContextMenu {
    type Action = MenuAction;

    fn mouse_id(&self) -> WidgetId {
        self.id
    }

    fn pointer(
        &mut self,
        ev: &crate::core::pointer::PointerEvent,
    ) -> crate::core::response::Response<MenuAction> {
        use crate::core::intent::Phase;
        use crate::core::response::Response;
        if ev.wheel().is_some() {
            // the wheel outside the menu (its backdrop) moves nothing
            if self.list.on_backdrop(ev) {
                return Response::ignored();
            }
        }
        if let Some(r) = crate::core::pointer::scroll_input(&mut self.list, ev) {
            return r.lift();
        }
        match ev.phase() {
            Some(Phase::Click) => {
                let action = if let Some(i) = self.list.row_of_event(ev) {
                    self.choose(i)
                } else if self.list.on_backdrop(ev) {
                    self.on_click_outside()
                } else {
                    self.page_marker(ev.id);
                    None
                };
                match action {
                    Some(a) => Response::emit(a),
                    None => Response::changed(),
                }
            }
            Some(Phase::Press | Phase::Drag | Phase::Secondary) => Response::consumed(),
            _ => Response::ignored(),
        }
    }
}

/// The bar's pointer gestures: a press on a title opens its menu under
/// [`OpenOn::Press`] ([`MenuBar::on_press`]); a completed click on a
/// title, the brand, a row of the open menu or the backdrop is
/// [`MenuBar::on_click`]; a vertical wheel over the open menu scrolls it.
/// The brand and the open menu are the bar's too ([`Self::route`]).
impl crate::core::pointer::MouseTarget for MenuBar {
    type Action = MenuBarAction;

    fn mouse_id(&self) -> WidgetId {
        self.id
    }

    fn route(
        &mut self,
        ev: &crate::core::pointer::PointerEvent,
    ) -> Option<crate::core::response::Response<MenuBarAction>> {
        let owner = ev.owner();
        let mine = owner == self.id
            || owner == self.brand_id()
            || self.open.as_ref().is_some_and(|m| m.id == owner);
        mine.then(|| self.pointer(ev))
    }

    fn pointer(
        &mut self,
        ev: &crate::core::pointer::PointerEvent,
    ) -> crate::core::response::Response<MenuBarAction> {
        use crate::core::intent::Phase;
        use crate::core::response::Response;
        if let Some((axis, delta)) = ev.wheel() {
            // the open menu's wheel: its list's shared path
            return match self.open.as_mut() {
                Some(m) => m.list.wheel(axis, delta).lift(),
                None => Response::ignored(),
            };
        }
        match ev.phase() {
            Some(Phase::Press) => self.on_press(ev.id).into(),
            Some(Phase::Click) => self.on_click(ev.id).into(),
            _ => Response::ignored(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;
    use ratatui::crossterm::event::KeyModifiers;

    #[test]
    fn a_menu_keeps_off_the_screen_side_edges() {
        let screen = Rect::new(0, 0, 60, 20);
        let items = vec![MenuItem::new("Open"), MenuItem::new("Close")];
        let m = ContextMenu::new(WidgetId::of("m"), items.clone())
            .anchor(Rect::new(0, 2, 1, 1), Placement::Below);
        assert_eq!(m.placed(screen).x, 1);
        let r = ContextMenu::new(WidgetId::of("m"), items.clone())
            .anchor(Rect::new(59, 2, 1, 1), Placement::Below)
            .placed(screen);
        assert_eq!(r.right(), 59);
        let flush = ContextMenu::new(WidgetId::of("m"), items)
            .edge_margin(0)
            .anchor(Rect::new(0, 2, 1, 1), Placement::Below);
        assert_eq!(flush.placed(screen).x, 0);
    }

    #[test]
    fn shortcuts_and_glyphs_can_be_built_at_runtime() {
        let modifier = String::from("Ctrl");
        let item = MenuItem::new("Save").shortcut(format!("{modifier}+S"));
        assert_eq!(item.shortcut.as_deref(), Some("Ctrl+S"));
        let risky = true;
        let item = MenuItem::new("Delete")
            .danger_if(risky)
            .separator_if(!risky);
        assert!(item.danger && !item.separator_after);
        let node = crate::widgets::tree::TreeNode::leaf("t").glyph(String::from("T"));
        assert_eq!(node.glyph.as_deref(), Some("T"));
    }

    #[test]
    fn a_submenu_opened_right_flips_left_instead_of_covering_its_anchor() {
        let screen = Rect::new(0, 0, 60, 20);
        let row = Rect::new(40, 3, 16, 1);
        let r = ContextMenu::new(WidgetId::of("m"), items())
            .anchor(row, Placement::Right)
            .placed(screen);
        assert!(r.right() <= row.x, "opens left of the row: {r:?}");
        assert_eq!(r.y, row.y);
        let r = ContextMenu::new(WidgetId::of("m"), items())
            .anchor(Rect::new(4, 3, 16, 1), Placement::Right)
            .placed(screen);
        assert_eq!(r.x, 20, "opens right of the row");
    }

    #[test]
    fn a_menu_above_its_anchor_flips_below_near_the_top() {
        let screen = Rect::new(0, 0, 60, 20);
        let r = ContextMenu::new(WidgetId::of("m"), items())
            .anchor(Rect::new(4, 1, 8, 1), Placement::Above)
            .placed(screen);
        assert_eq!(r.y, 2, "below the anchor, never over it: {r:?}");
    }

    fn key(code: KeyCode) -> Key {
        Key {
            code,
            mods: KeyModifiers::NONE,
        }
    }

    fn items() -> Vec<MenuItem> {
        vec![
            MenuItem::new("Change title…").shortcut("r"),
            MenuItem::new("Duplicate").disabled(true),
            MenuItem::new("Move left").separator(),
            MenuItem::new("Close").shortcut("x").danger(),
        ]
    }

    /// SI21/SV15: the keyboard reaches a disabled row (APG menus) but
    /// Enter never chooses it; `skip_disabled(true)` keeps the old skip.
    #[test]
    fn keyboard_reaches_disabled_rows_wraps_and_chooses() {
        let mut m = ContextMenu::new(WidgetId::of("m"), items());
        assert_eq!(m.list.cursor.index, 0);
        m.on_key(&key(KeyCode::Down));
        assert_eq!(m.list.cursor.index, 1, "the disabled row is reachable");
        assert_eq!(
            m.on_key(&key(KeyCode::Enter)),
            (Outcome::Consumed, None),
            "but never chosen"
        );
        m.on_key(&key(KeyCode::Down));
        assert_eq!(m.list.cursor.index, 2);
        m.on_key(&key(KeyCode::Down));
        assert_eq!(m.list.cursor.index, 3);
        m.on_key(&key(KeyCode::Down));
        assert_eq!(m.list.cursor.index, 0, "wraps");
        m.on_key(&key(KeyCode::End));
        assert_eq!(m.list.cursor.index, 3);
        let (_, ev) = m.on_key(&key(KeyCode::Enter));
        assert_eq!(ev, Some(MenuAction::Chosen(3)));
        let (_, ev) = m.on_key(&key(KeyCode::Esc));
        assert_eq!(ev, Some(MenuAction::Dismissed));
        let mut skip = ContextMenu::new(WidgetId::of("m"), items()).skip_disabled(true);
        skip.on_key(&key(KeyCode::Down));
        assert_eq!(skip.list.cursor.index, 2, "skipped on request");
    }

    #[test]
    fn placement_is_clamped_to_the_screen_and_flips_up() {
        let screen = Rect::new(0, 0, 40, 12);
        let m = ContextMenu::new(WidgetId::of("m"), items())
            .anchor(Rect::new(34, 9, 4, 1), Placement::Below);
        let r = m.placed(screen);
        assert!(r.right() <= 40 && r.bottom() <= 12, "{r:?}");
        assert!(r.y < 9, "flipped above the anchor: {r:?}");
        let m = ContextMenu::new(WidgetId::of("m"), items()).at(Position::new(2, 2));
        let r = m.placed(screen);
        assert_eq!((r.x, r.y), (2, 3));
    }

    #[test]
    fn click_selects_rows_and_outside_dismisses() {
        let t = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, 60, 20));
        let mut m = ContextMenu::new(WidgetId::of("m"), items()).at(Position::new(5, 2));
        m.render(Rect::new(0, 0, 60, 20), &mut buf, &mut ctx);
        ctx.flush_overlays(&mut buf);
        let row3 = m.row_id(3);
        let area = hits.area_of(row3).expect("row registered");
        assert_eq!(hits.hit(Position::new(area.x + 2, area.y)), Some(row3));
        assert_eq!(m.on_click(row3), Some(MenuAction::Chosen(3)));
        assert_eq!(m.on_click(m.row_id(1)), None, "disabled rows do nothing");
        assert_eq!(
            m.on_click(WidgetId::of("elsewhere")),
            Some(MenuAction::Dismissed)
        );
        // a click outside lands on the menu's backdrop; the policy decides
        assert_eq!(hits.hit(Position::new(50, 18)), Some(backdrop_id(m.id)));
        let mut sticky = m.clone().dismiss(Dismiss::ESCAPE);
        assert_eq!(sticky.on_click(backdrop_id(m.id)), None);
        assert_eq!(
            sticky.on_key(&key(KeyCode::Esc)).1,
            Some(MenuAction::Dismissed)
        );
        // danger row is drawn in the error tone, shortcut right-aligned
        assert_eq!(buf[(area.x + 2, area.y)].fg, t.error_soft);
        let sc_x = area.right() - 2;
        assert_eq!(buf[(sc_x, area.y)].symbol(), "x");
    }

    #[test]
    fn hover_moves_the_cursor() {
        let t = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut m = ContextMenu::new(WidgetId::of("m"), items()).at(Position::new(1, 1));
        let mut ctx = RenderCtx::new(
            &t,
            Interaction {
                hover: Some(m.row_id(3)),
                ..Default::default()
            },
            &mut hits,
            &mut ring,
        );
        let mut buf = Buffer::empty(Rect::new(0, 0, 60, 20));
        m.render(Rect::new(0, 0, 60, 20), &mut buf, &mut ctx);
        assert_eq!(m.list.cursor.index, 3);
    }

    /// K14 in a menu-bar menu: the shortcut letter shown beside a row
    /// chooses it; a key the menu has no use for closes the bar and is
    /// passed on (`Ignored`); `UnknownKey::Swallow` keeps the old way.
    #[test]
    fn menubar_menus_honour_shortcut_letters_and_pass_other_keys() {
        let bar = || {
            MenuBar::new(
                WidgetId::of("bar"),
                vec![(
                    "File",
                    vec![MenuItem::new("New").shortcut("n"), MenuItem::new("Quit")],
                )],
            )
        };
        let mut b = bar();
        b.on_key(&key(KeyCode::Enter));
        assert_eq!(
            b.on_key(&key(KeyCode::Char('n'))),
            (Outcome::Changed, Some(MenuBarAction::Chosen(0, 0)))
        );
        b.on_key(&key(KeyCode::Enter));
        // with type-ahead on (the Desktop default) a letter is a prefix: a
        // miss is consumed and the menu stays open, as in macOS menus
        assert_eq!(
            b.on_key(&key(KeyCode::Char('z'))),
            (Outcome::Consumed, None)
        );
        assert!(b.is_open());
        // a key that is neither a shortcut nor a prefix closes and passes
        assert_eq!(
            b.on_key(&key(KeyCode::Char('?'))),
            (Outcome::Ignored, Some(MenuBarAction::Closed))
        );
        assert!(!b.is_open());
        let mut b = bar().type_ahead(false);
        b.on_key(&key(KeyCode::Enter));
        assert_eq!(
            b.on_key(&key(KeyCode::Char('z'))),
            (Outcome::Ignored, Some(MenuBarAction::Closed))
        );
        assert!(!b.is_open());
        let mut m = ContextMenu::new(WidgetId::of("m"), vec![MenuItem::new("New")])
            .unknown_key(UnknownKey::Swallow);
        assert_eq!(
            m.on_key(&key(KeyCode::Char('z'))),
            (Outcome::Consumed, None)
        );
    }

    #[test]
    fn menubar_opens_switches_and_chooses() {
        let mut bar = MenuBar::new(
            WidgetId::of("bar"),
            vec![
                ("File", vec![MenuItem::new("New"), MenuItem::new("Quit")]),
                ("View", vec![MenuItem::new("Zoom")]),
            ],
        )
        .brand(Lockup::new("app❯"));
        let t = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, 60, 20));
        bar.render(Rect::new(0, 0, 60, 1), &mut buf, &mut ctx, t.canvas);
        assert_eq!(bar.areas.len(), 2);
        assert!(bar.areas[0].x > bar.brand_area.right());
        let (_, ev) = bar.on_key(&key(KeyCode::Enter));
        assert_eq!(ev, Some(MenuBarAction::Opened(0)));
        assert!(bar.is_open());
        let (_, ev) = bar.on_key(&key(KeyCode::Right));
        assert_eq!(ev, Some(MenuBarAction::Opened(1)));
        assert_eq!(bar.open_index(), Some(1));
        let (_, ev) = bar.on_key(&key(KeyCode::Enter));
        assert_eq!(ev, Some(MenuBarAction::Chosen(1, 0)));
        assert!(!bar.is_open());
        // click toggles, click on the brand fires Brand
        let (_, ev) = bar.on_click(bar.label_id(0));
        assert_eq!(ev, Some(MenuBarAction::Opened(0)));
        bar.render(Rect::new(0, 0, 60, 1), &mut buf, &mut ctx, t.canvas);
        ctx.flush_overlays(&mut buf);
        let popover = bar.open.as_ref().unwrap().area;
        assert_eq!(popover.y, 1, "anchored beneath the label");
        assert_eq!(popover.x, bar.areas[0].x);
        let (_, ev) = bar.on_click(bar.label_id(0));
        assert_eq!(ev, Some(MenuBarAction::Closed));
        let (_, ev) = bar.on_click(bar.brand_id());
        assert_eq!(ev, Some(MenuBarAction::Brand));
        // hover switching is on by default and can be turned off
        bar.open_menu(0);
        assert_eq!(bar.on_hover(Some(bar.label_id(1))), Outcome::Changed);
        assert_eq!(bar.open_index(), Some(1));
        bar.hover_switch = false;
        assert_eq!(bar.on_hover(Some(bar.label_id(0))), Outcome::Ignored);
        assert_eq!(bar.open_index(), Some(1));
        // Esc closes an open menu
        bar.open_menu(1);
        let (_, ev) = bar.on_key(&key(KeyCode::Esc));
        assert_eq!(ev, Some(MenuBarAction::Closed));
    }

    fn two_menus() -> MenuBar {
        MenuBar::new(
            WidgetId::of("bar"),
            vec![
                ("File", vec![MenuItem::new("New"), MenuItem::new("Quit")]),
                ("View", vec![MenuItem::new("Zoom")]),
            ],
        )
    }

    /// Render `bar` (and its open menu) under `interaction`; the frame's
    /// hit regions.
    fn bar_hits(bar: &mut MenuBar, interaction: Interaction) -> HitRegistry {
        let t = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&t, interaction, &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, 60, 20));
        bar.render(Rect::new(0, 0, 60, 1), &mut buf, &mut ctx, t.canvas);
        ctx.flush_overlays(&mut buf);
        hits
    }

    #[test]
    fn a_title_opens_on_the_press_and_its_release_keeps_it_open() {
        let mut bar = two_menus();
        bar_hits(&mut bar, Interaction::default());
        let file = bar.label_id(0);
        assert_eq!(bar.on_press(file).1, Some(MenuBarAction::Opened(0)));
        assert_eq!(bar.on_click(file), (Outcome::Consumed, None), "release");
        assert_eq!(bar.open_index(), Some(0));
        // pressing the open title closes it, its release keeps it shut
        assert_eq!(bar.on_press(file).1, Some(MenuBarAction::Closed));
        assert_eq!(bar.on_click(file).1, None);
        assert!(!bar.is_open());
        // OpenOn::Release: the press does nothing, the click toggles
        let mut bar = two_menus().open_on(OpenOn::Release);
        bar_hits(&mut bar, Interaction::default());
        assert_eq!(bar.on_press(file), (Outcome::Ignored, None));
        assert_eq!(bar.on_click(file).1, Some(MenuBarAction::Opened(0)));
    }

    #[test]
    fn a_press_on_a_title_may_release_on_a_row_of_its_menu() {
        let mut bar = two_menus();
        bar_hits(&mut bar, Interaction::default());
        bar.on_press(bar.label_id(0));
        let hits = bar_hits(&mut bar, Interaction::default());
        let quit = bar.label_id(0).sub("menu").child(1);
        assert!(hits.accepts_release(quit, ReleaseFrom::Press(bar.label_id(0))));
        // a drag across the titles switches menus: any title's press counts
        assert!(hits.accepts_release(quit, ReleaseFrom::Press(bar.label_id(1))));
        let (_, ev) = bar.on_click(quit);
        assert_eq!(ev, Some(MenuBarAction::Chosen(0, 1)));
        // press_drag off: a release elsewhere cancels the click
        let mut bar = two_menus().press_drag(false);
        bar_hits(&mut bar, Interaction::default());
        bar.on_press(bar.label_id(0));
        let hits = bar_hits(&mut bar, Interaction::default());
        assert!(!hits.accepts_release(quit, ReleaseFrom::Press(bar.label_id(0))));
    }

    #[test]
    fn hover_switching_waits_for_reported_motion() {
        let mut bar = two_menus();
        bar_hits(&mut bar, Interaction::default());
        bar.open_menu(0);
        let view = bar.label_id(1);
        let no_motion = Interaction {
            hover: Some(view),
            motion_unreported: true,
            ..Default::default()
        };
        bar_hits(&mut bar, no_motion);
        assert_eq!(bar.open_index(), Some(0), "a stale hover does not switch");
        // a press dragged across the titles still switches
        let dragging = Interaction {
            pressed: Some(bar.label_id(0)),
            ..no_motion
        };
        bar_hits(&mut bar, dragging);
        assert_eq!(bar.open_index(), Some(1));
        bar.open_menu(0);
        bar_hits(
            &mut bar,
            Interaction {
                hover: Some(view),
                ..Default::default()
            },
        );
        assert_eq!(bar.open_index(), Some(1), "with motion, hover switches");
    }

    #[test]
    fn a_context_menu_row_accepts_a_secondary_release_unless_turned_off() {
        let t = Theme::junie();
        let mut m = ContextMenu::new(WidgetId::of("m"), items()).at(Position::new(5, 2));
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, 60, 20));
        m.render(Rect::new(0, 0, 60, 20), &mut buf, &mut ctx);
        ctx.flush_overlays(&mut buf);
        assert!(hits.accepts_release(m.row_id(0), ReleaseFrom::Secondary));
        assert!(
            !hits.accepts_release(m.row_id(1), ReleaseFrom::Secondary),
            "a disabled row is never chosen"
        );
        let mut off = m.clone().release_selects(false);
        let mut hits = HitRegistry::default();
        let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
        off.render(Rect::new(0, 0, 60, 20), &mut buf, &mut ctx);
        ctx.flush_overlays(&mut buf);
        assert!(!hits.accepts_release(m.row_id(0), ReleaseFrom::Secondary));
    }

    #[test]
    fn the_highlight_clears_when_the_pointer_leaves_the_rows() {
        let t = Theme::junie();
        let mut m = ContextMenu::new(WidgetId::of("m"), items()).at(Position::new(1, 1));
        let hover = |id| Interaction {
            hover: id,
            ..Default::default()
        };
        let row3 = m.row_id(3);
        paint_menu(&mut m, &t, hover(Some(row3)));
        assert_eq!(m.highlighted(), Some(3));
        let buf = paint_menu(&mut m, &t, hover(Some(WidgetId::of("page"))));
        assert_eq!(m.highlighted(), None, "no stale highlight outside");
        let (x, y) = row_y(&buf, "Close");
        assert_eq!(buf[(x, y)].bg, t.popover, "the row is back at rest");
        assert_eq!(m.on_key(&key(KeyCode::Enter)), (Outcome::Consumed, None));
        // a key brings the cursor back
        m.on_key(&key(KeyCode::Down));
        assert!(m.highlighted().is_some());
        // a keyboard cursor never hides on its own
        paint_menu(&mut m, &t, hover(None));
        assert!(m.highlighted().is_some());
    }

    #[test]
    fn the_overflow_markers_page_on_click_and_scroll_on_hover() {
        let t = Theme::junie();
        let items: Vec<MenuItem> = (0..30)
            .map(|i| MenuItem::new(format!("item {i:02}")))
            .collect();
        let mut m = ContextMenu::new(WidgetId::of("m"), items).max_rows(5);
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, 60, 20));
        m.render(Rect::new(0, 0, 60, 20), &mut buf, &mut ctx);
        ctx.flush_overlays(&mut buf);
        let below = m.more_below_id();
        let r = hits.area_of(below).expect("the down marker is a target");
        assert_eq!(hits.hit(Position::new(r.x + 1, r.y)), Some(below));
        assert!(hits.area_of(m.more_above_id()).is_none(), "nothing above");
        assert!(m.contains_id(below));
        assert_eq!(m.on_click(below), None);
        assert_eq!(m.list.scroll.offset, 5, "a click pages down");
        // resting on the up marker scrolls one row per tick
        let above = m.more_above_id();
        let at = |tick| Interaction {
            hover: Some(above),
            tick,
            ..Default::default()
        };
        paint_menu(&mut m, &t, at(1));
        assert_eq!(m.list.scroll.offset, 4);
        paint_menu(&mut m, &t, at(1));
        assert_eq!(m.list.scroll.offset, 4, "once per tick");
        paint_menu(&mut m, &t, at(2));
        assert_eq!(m.list.scroll.offset, 3);
    }

    #[test]
    fn a_label_cut_by_a_narrow_screen_reveals_itself_on_hover_above_the_menu() {
        let long = "Open-the-workspace-in-a-new-window";
        let mut m = ContextMenu::new(WidgetId::of("m"), vec![MenuItem::new(long)])
            .anchor(Rect::new(0, 0, 1, 1), Placement::Below);
        let t = Theme::junie();
        let mut draw = |hover| {
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let interaction = Interaction {
                hover,
                ..Default::default()
            };
            let mut ctx = RenderCtx::new(&t, interaction, &mut hits, &mut ring);
            let mut buf = Buffer::empty(Rect::new(0, 0, 40, 6));
            m.render(Rect::new(0, 0, 30, 6), &mut buf, &mut ctx);
            ctx.flush_overlays(&mut buf);
            buf
        };
        let text = |buf: &Buffer| {
            (0..6)
                .map(|y| (0..40).map(|x| buf[(x, y)].symbol()).collect::<String>())
                .collect::<Vec<_>>()
                .join("\n")
        };
        assert!(!text(&draw(None)).contains(long));
        let row = WidgetId::of("m").child(0);
        let buf = draw(Some(row));
        assert!(text(&buf).contains(long), "{}", text(&buf));
        // the tooltip lifts off the menu's popover plane
        let (x, y) = (0..6)
            .flat_map(|y| (0..40).map(move |x| (x, y)))
            .find(|&(x, y)| buf[(x, y)].symbol() == "O")
            .unwrap();
        assert_ne!(buf[(x, y)].bg, t.popover);
    }

    fn paint_menu(m: &mut ContextMenu, t: &Theme, interaction: Interaction) -> Buffer {
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(t, interaction, &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, 60, 20));
        m.render(Rect::new(0, 0, 60, 20), &mut buf, &mut ctx);
        ctx.flush_overlays(&mut buf);
        buf
    }

    fn shows(buf: &Buffer, text: &str) -> bool {
        (0..buf.area.height).any(|y| {
            (0..buf.area.width)
                .map(|x| buf[(x, y)].symbol())
                .collect::<String>()
                .contains(text)
        })
    }

    #[test]
    fn a_menu_taller_than_its_room_scrolls_so_every_item_is_reachable() {
        let t = Theme::junie();
        let items: Vec<MenuItem> = (0..30)
            .map(|i| MenuItem::new(format!("item {i:02}")))
            .collect();
        let mut m = ContextMenu::new(WidgetId::of("m"), items.clone()).at(Position::new(2, 2));
        let buf = paint_menu(&mut m, &t, Interaction::default());
        assert!(shows(&buf, "item 00") && !shows(&buf, "item 29"));
        m.on_key(&key(KeyCode::End));
        let buf = paint_menu(&mut m, &t, Interaction::default());
        assert!(shows(&buf, "item 29"), "the last item scrolled into view");
        assert!(!shows(&buf, "item 00"));
        // the wheel scrolls without moving the cursor
        assert_eq!(
            crate::core::scroll::Scrollable::wheel(&mut m.list, Axis::Vertical, -3).outcome(),
            Outcome::Changed
        );
        assert_eq!(m.list.cursor.index, 29);
        // max_rows caps a long menu on a tall screen too
        let mut capped = ContextMenu::new(WidgetId::of("m"), items).max_rows(5);
        assert_eq!(capped.size().1, 5 + 2);
        let buf = paint_menu(&mut capped, &t, Interaction::default());
        assert!(shows(&buf, "item 04") && !shows(&buf, "item 05"));
    }

    fn row_y(buf: &Buffer, text: &str) -> (u16, u16) {
        for y in 0..buf.area.height {
            let line: String = (0..buf.area.width).map(|x| buf[(x, y)].symbol()).collect();
            if let Some(i) = line.find(text) {
                return (line[..i].chars().count() as u16, y);
            }
        }
        panic!("{text} not drawn")
    }

    #[test]
    fn a_menu_row_shows_rest_cursor_hover_and_press_apart_at_every_level() {
        use crate::theme::ColorLevel;
        for level in ColorLevel::ALL {
            let t = Theme::for_level(level);
            let mut m = ContextMenu::new(WidgetId::of("m"), items()).at(Position::new(1, 1));
            let rid = m.row_id(0);
            let look = |b: &Buffer, x: u16, y: u16| {
                let c = &b[(x, y)];
                if level == ColorLevel::Mono {
                    (None, c.modifier)
                } else {
                    (Some(c.bg), c.modifier)
                }
            };
            // the cursor sits on row 0; another row is at rest
            let b = paint_menu(&mut m, &t, Interaction::default());
            let (x, y) = row_y(&b, "Change");
            let (mx, my) = row_y(&b, "Move left");
            let cursor = look(&b, x, y);
            let rest = look(&b, mx, my);
            let b = paint_menu(
                &mut m,
                &t,
                Interaction {
                    hover: Some(rid),
                    ..Default::default()
                },
            );
            let hover = look(&b, x, y);
            let b = paint_menu(
                &mut m,
                &t,
                Interaction {
                    hover: Some(rid),
                    pressed: Some(rid),
                    ..Default::default()
                },
            );
            let pressed = look(&b, x, y);
            assert_ne!(rest, cursor, "{level:?}: rest = cursor");
            assert_ne!(cursor, hover, "{level:?}: cursor = hover");
            assert_ne!(hover, pressed, "{level:?}: hover = pressed");
            assert_ne!(cursor, pressed, "{level:?}: cursor = pressed");
        }
    }

    /// A1: the popover frame stands off the popover plane. At 16 colours
    /// and `none` both were `DarkGray` (an invisible frame, 1.0:1); there
    /// the frame now reads at 3:1 or more. Truecolor and 256 keep the soft
    /// frame (recorded in `color.md`).
    #[test]
    fn the_menu_frame_stands_off_the_popover_plane_at_every_level() {
        use crate::theme::{ColorLevel, contrast_ratio};
        for level in ColorLevel::ALL {
            let t = Theme::for_level(level);
            let mut m = ContextMenu::new(WidgetId::of("m"), items()).at(Position::new(1, 1));
            let b = paint_menu(&mut m, &t, Interaction::default());
            let corner = (0..b.area.height)
                .flat_map(|y| (0..b.area.width).map(move |x| (x, y)))
                .find(|&p| b[p].symbol() == t.glyphs.border.top_left)
                .expect("menu frame drawn");
            let c = &b[corner];
            assert_eq!(c.bg, t.popover, "{level:?}");
            assert_ne!(c.fg, c.bg, "{level:?}: frame on its own plane");
            if matches!(level, ColorLevel::Ansi16 | ColorLevel::Mono) {
                let r = contrast_ratio(c.fg, c.bg).unwrap();
                assert!(r >= 3.0, "{level:?}: frame {r:.2}");
            }
        }
    }

    #[test]
    fn a_destructive_cursor_row_is_not_grey_on_grey_at_256() {
        let t = Theme::for_level(crate::theme::ColorLevel::Ansi256);
        let mut m = ContextMenu::new(WidgetId::of("m"), items()).at(Position::new(1, 1));
        m.list.cursor.index = 3;
        let b = paint_menu(&mut m, &t, Interaction::default());
        let (x, y) = row_y(&b, "Close");
        assert_eq!(b[(x, y)].bg, t.highlight_danger);
        assert!(matches!(b[(x, y)].bg, Color::Indexed(i) if i < 232));
    }

    /// SI21/SV15: a disabled row keeps its dimmed shortcut; its reason
    /// never widens the menu and shows in the status row (the bottom
    /// border), cut to fit, while the cursor or the pointer is on the row;
    /// Enter, a click and its shortcut never choose it.
    #[test]
    fn an_unavailable_item_keeps_its_dimmed_shortcut_and_says_why_in_the_status_row() {
        let t = Theme::junie();
        let items = |reason: bool| {
            let stage = MenuItem::new("Stage image").shortcut("i");
            vec![
                MenuItem::new("Open"),
                if reason {
                    stage.reason("Not in this preview, use the full app")
                } else {
                    stage
                },
            ]
        };
        let mut m = ContextMenu::new(WidgetId::of("m"), items(true)).at(Position::new(1, 1));
        assert!(m.items[1].item.disabled);
        assert_eq!(
            m.size(),
            ContextMenu::new(WidgetId::of("m"), items(false)).size(),
            "the reason never widens the menu"
        );
        let line = |b: &Buffer, y: u16| -> String {
            (0..b.area.width).map(|x| b[(x, y)].symbol()).collect()
        };
        let b = paint_menu(&mut m, &t, Interaction::default());
        let (lx, y) = row_y(&b, "Stage image");
        let row = line(&b, y);
        assert!(row.ends_with(" i │") || row.contains(" i "), "{row}");
        // the shortcut keeps the row's disabled ink
        let sx = (lx + 11..b.area.width)
            .find(|&x| b[(x, y)].symbol() == "i")
            .unwrap();
        assert_eq!(b[(sx, y)].fg, b[(lx, y)].fg);
        let bottom = m.area.bottom() - 1;
        assert!(
            !shows(&b, "Not in this"),
            "no reason while the cursor is elsewhere"
        );
        // the keyboard reaches it: the reason shows in the bottom border
        m.on_key(&key(KeyCode::Down));
        assert_eq!(m.list.cursor.index, 1);
        let b = paint_menu(&mut m, &t, Interaction::default());
        let border = line(&b, bottom);
        assert!(border.contains(" Not in this"), "{border}");
        assert!(border.contains('…'), "cut to fit: {border}");
        assert_eq!(m.area.width, m.size().0);
        // Enter, its shortcut and a click never choose it
        assert_eq!(m.on_key(&key(KeyCode::Enter)).1, None);
        assert_ne!(
            m.on_key(&key(KeyCode::Char('i'))).1,
            Some(MenuAction::Chosen(1))
        );
        let mut m = ContextMenu::new(WidgetId::of("m"), items(true)).at(Position::new(1, 1));
        let id = m.row_id(1);
        assert_eq!(m.on_click(id), None);
        // the pointer on it says why too, without moving the cursor there
        let b = paint_menu(
            &mut m,
            &t,
            Interaction {
                hover: Some(id),
                ..Default::default()
            },
        );
        assert!(
            line(&b, bottom).contains(" Not in this"),
            "{}",
            line(&b, bottom)
        );
        assert_eq!(m.list.cursor.index, 0);
    }

    fn wide_bar(overflow: Overflow) -> MenuBar {
        MenuBar::new(
            WidgetId::of("bar"),
            vec![
                ("File", vec![MenuItem::new("New")]),
                ("Edit", vec![MenuItem::new("Undo")]),
                ("Selection", vec![MenuItem::new("All")]),
                ("Window", vec![MenuItem::new("Zoom"), MenuItem::new("Tile")]),
            ],
        )
        .overflow(overflow)
    }

    fn bar_row(bar: &mut MenuBar, t: &Theme, w: u16) -> String {
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(t, Interaction::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, w, 10));
        bar.render(Rect::new(0, 0, w, 1), &mut buf, &mut ctx, t.canvas);
        ctx.flush_overlays(&mut buf);
        (0..w).map(|x| buf[(x, 0)].symbol()).collect()
    }

    #[test]
    fn hidden_titles_are_dropped_by_default() {
        let t = Theme::junie();
        let mut bar = wide_bar(Overflow::Hide);
        let row = bar_row(&mut bar, &t, 24);
        assert!(row.contains("File") && row.contains("Edit"), "{row}");
        assert!(!row.contains('»'), "no chevron by default: {row}");
        assert_eq!(bar.more_area, Rect::ZERO);
        assert!(bar.hidden().is_empty());
    }

    #[test]
    fn a_chevron_collects_the_titles_that_do_not_fit() {
        let t = Theme::junie();
        let mut bar = wide_bar(Overflow::Chevron);
        // everything fits: no chevron
        let row = bar_row(&mut bar, &t, 60);
        assert!(row.contains("Window") && !row.contains('»'), "{row}");
        let row = bar_row(&mut bar, &t, 24);
        assert!(row.contains(" » "), "{row}");
        assert!(!row.contains("Window"), "{row}");
        assert_eq!(bar.hidden(), vec![2, 3], "order kept once one is hidden");
        assert!(bar.more_area.right() <= 24);
        // clicking the chevron lists the hidden titles
        let (_, ev) = bar.on_click(bar.more_id());
        assert_eq!(ev, Some(MenuBarAction::Opened(4)));
        let items: Vec<&str> = bar
            .open
            .as_ref()
            .unwrap()
            .items
            .iter()
            .map(|i| i.item.label.as_str())
            .collect();
        assert_eq!(items, ["Selection", "Window"]);
        // choosing one opens that title's menu under the chevron
        bar.on_key(&key(KeyCode::Down));
        let (_, ev) = bar.on_key(&key(KeyCode::Enter));
        assert_eq!(ev, Some(MenuBarAction::Opened(3)));
        assert_eq!(bar.open_index(), Some(3));
        bar_row(&mut bar, &t, 24);
        let open = bar.open.as_ref().unwrap();
        assert_eq!(open.items.len(), 2, "the Window menu");
        assert_eq!(open.area.y, 1);
        assert!(open.area.x <= bar.more_area.x && open.area.right() > bar.more_area.x);
        // its items choose as usual
        let (_, ev) = bar.on_key(&key(KeyCode::Enter));
        assert_eq!(ev, Some(MenuBarAction::Chosen(3, 0)));
    }

    #[test]
    fn the_arrow_keys_visit_the_chevron_like_a_title() {
        let t = Theme::junie();
        let mut bar = wide_bar(Overflow::Chevron);
        bar_row(&mut bar, &t, 24);
        bar.cursor = 1;
        bar.on_key(&key(KeyCode::Right));
        assert_eq!(bar.cursor, 4, "Edit -> chevron, past the hidden titles");
        bar.on_key(&key(KeyCode::Right));
        assert_eq!(bar.cursor, 0, "wraps to File");
        bar.on_key(&key(KeyCode::Left));
        assert_eq!(bar.cursor, 4);
        let (_, ev) = bar.on_key(&key(KeyCode::Enter));
        assert_eq!(ev, Some(MenuBarAction::Opened(4)));
        // with the chevron menu open, Left moves to Edit's menu
        let (_, ev) = bar.on_key(&key(KeyCode::Left));
        assert_eq!(ev, Some(MenuBarAction::Opened(1)));
        // hovering the chevron while a hidden title's menu shows keeps it
        bar.open_menu(3);
        bar_row(&mut bar, &t, 24);
        assert_eq!(bar.on_hover(Some(bar.more_id())), Outcome::Ignored);
        assert_eq!(bar.open_index(), Some(3));
    }

    #[test]
    fn the_chevron_draws_with_the_theme_glyph() {
        let t = Theme::junie().with_glyphs(crate::theme::Glyphs::ASCII);
        let mut bar = wide_bar(Overflow::Chevron);
        let row = bar_row(&mut bar, &t, 24);
        assert!(row.contains(" > ") && row.is_ascii(), "{row}");
    }

    /// Thirty rows with a separator after every fifth: taller than the
    /// 12-line screen the wheel tests draw on.
    fn long_items() -> Vec<MenuItem> {
        (0..30)
            .map(|i| MenuItem::new(format!("item {i:02}")).separator_if(i % 5 == 4))
            .collect()
    }

    /// A wheel notch on the first row in view of an open menu, routed the
    /// way an app routes it: through the menu's
    /// [`MouseTarget`](crate::core::pointer::MouseTarget) and its list's
    /// shared wheel.
    fn wheel_menu(m: &mut ContextMenu, axis: Axis, delta: i32) -> Outcome {
        use crate::core::pointer::{MouseTarget, PointerEvent};
        let t = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 12));
        m.render(Rect::new(0, 0, 40, 12), &mut buf, &mut ctx);
        ctx.flush_overlays(&mut buf);
        drop(ctx);
        let first = m.list.visible_lines().find_map(|(_, r)| r).unwrap();
        let row = m.row_id(first);
        let pos = hits.area_of(row).expect("a visible row").as_position();
        let ev = PointerEvent::wheel_on(&hits, row, axis, delta, pos, KeyModifiers::NONE);
        m.pointer(&ev).outcome()
    }

    #[test]
    fn the_wheel_over_a_menu_follows_its_wheel_mode_through_the_shared_path() {
        // Viewport (the default): the view scrolls by lines, the cursor
        // stays, and the next frame keeps the scroll
        let mut m = ContextMenu::new(WidgetId::of("m"), long_items()).at(Position::new(2, 0));
        assert_eq!(m.list.wheel, WheelMode::Viewport);
        assert_eq!(wheel_menu(&mut m, Axis::Vertical, 3), Outcome::Changed);
        assert_eq!((m.list.scroll.offset, m.list.cursor.index), (3, 0));
        assert_eq!(wheel_menu(&mut m, Axis::Vertical, -1), Outcome::Changed);
        assert_eq!(m.list.scroll.offset, 2, "the next frame kept the scroll");
        // a sideways notch is consumed: nothing beneath the menu moves
        assert_eq!(wheel_menu(&mut m, Axis::Horizontal, 1), Outcome::Consumed);
        assert_eq!(m.list.scroll.offset, 2);
        // Cursor: the wheel moves the cursor a row per line and the view
        // follows it, separators included
        let mut c = ContextMenu::new(WidgetId::of("m"), long_items())
            .at(Position::new(2, 0))
            .wheel_mode(WheelMode::Cursor);
        assert_eq!(wheel_menu(&mut c, Axis::Vertical, 12), Outcome::Changed);
        assert_eq!(c.list.cursor.index, 12);
        let line = c.list.line_of(12);
        assert_eq!(line, 14, "two separators above row 12");
        assert!(c.list.scroll.visible_range().contains(&line));
        assert_eq!(wheel_menu(&mut c, Axis::Vertical, -2), Outcome::Changed);
        assert_eq!(c.list.cursor.index, 10);
    }

    #[test]
    fn the_wheel_over_a_menu_bar_menu_scrolls_it_through_the_shared_path() {
        use crate::core::pointer::{MouseTarget, PointerEvent};
        let mut bar = MenuBar::new(
            WidgetId::of("bar"),
            vec![
                ("File", long_items()),
                ("View", vec![MenuItem::new("Zoom")]),
            ],
        );
        bar.open_menu(0);
        let t = Theme::junie();
        let draw = |bar: &mut MenuBar| {
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
            let mut buf = Buffer::empty(Rect::new(0, 0, 40, 12));
            bar.render(Rect::new(0, 0, 40, 1), &mut buf, &mut ctx, t.canvas);
            ctx.flush_overlays(&mut buf);
            drop(ctx);
            hits
        };
        let hits = draw(&mut bar);
        let m = bar.open.as_ref().unwrap();
        let row = m.row_id(0);
        let pos = hits.area_of(row).expect("a visible row").as_position();
        let ev = PointerEvent::wheel_on(&hits, row, Axis::Vertical, 2, pos, KeyModifiers::NONE);
        let r = bar.route(&ev).expect("the open menu's event is the bar's");
        assert_eq!(r.outcome(), Outcome::Changed);
        let m = bar.open.as_ref().unwrap();
        assert_eq!((m.list.scroll.offset, m.list.cursor.index), (2, 0));
        // under WheelMode::Cursor the same notch moves the cursor
        bar.open.as_mut().unwrap().list.wheel = WheelMode::Cursor;
        let hits = draw(&mut bar);
        let row = bar.open.as_ref().unwrap().row_id(2);
        let pos = hits.area_of(row).expect("a visible row").as_position();
        let ev = PointerEvent::wheel_on(&hits, row, Axis::Vertical, 2, pos, KeyModifiers::NONE);
        assert_eq!(bar.route(&ev).unwrap().outcome(), Outcome::Changed);
        assert_eq!(bar.open.as_ref().unwrap().list.cursor.index, 2);
    }
}
