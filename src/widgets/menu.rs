//! Anchored menus: a context menu popover that opens at a position or under
//! an anchor rectangle, and a menu bar whose labels open the same popover
//! beneath them. Rows are commands (a solid `highlight` fill on the cursor,
//! hover lifts); shortcuts sit right-aligned in the muted tone; danger rows (soft rose, deep red under the cursor)
//! use the error tone; disabled rows are faint and cannot be chosen.

use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Modifier, Style};

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::theme::{CursorStyle, Kind, Plane};
use crate::ui::ctx::{RenderCtx, fill};
use crate::ui::popup::{Dismiss, Place, backdrop_id, place_with};
use crate::ui::text::{truncate, width};
use crate::widgets::brand::Lockup;
use ratatui::crossterm::event::KeyCode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuItem {
    pub label: String,
    pub shortcut: Option<&'static str>,
    pub disabled: bool,
    pub danger: bool,
    pub separator_after: bool,
    /// Why a disabled item is unavailable, shown dimmed in place of the
    /// shortcut (e.g. "Not in this preview"). `None`: no reason shown.
    pub reason: Option<String>,
}

impl MenuItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            shortcut: None,
            disabled: false,
            danger: false,
            separator_after: false,
            reason: None,
        }
    }
    pub fn shortcut(mut self, s: &'static str) -> Self {
        self.shortcut = Some(s);
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    /// Disable the item and say why. An unavailable command is shown as
    /// unavailable up front instead of failing after it is chosen.
    pub fn reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self.disabled = true;
        self
    }
    /// The text drawn in the right column: the reason while disabled,
    /// otherwise the shortcut.
    fn trailing(&self) -> Option<&str> {
        match &self.reason {
            Some(r) if self.disabled => Some(r.as_str()),
            _ => self.shortcut,
        }
    }
    pub fn danger(mut self) -> Self {
        self.danger = true;
        self
    }
    pub fn separator(mut self) -> Self {
        self.separator_after = true;
        self
    }
}

/// Menus are placed like every other popup: this is
/// [`crate::ui::popup::Placement`], re-exported so `menu::Placement`
/// paths keep compiling.
pub use crate::ui::popup::Placement;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuEvent {
    Chosen(usize),
    Dismissed,
}

#[derive(Debug, Clone)]
pub struct ContextMenu {
    pub id: WidgetId,
    pub items: Vec<MenuItem>,
    pub cursor: usize,
    pub anchor: Rect,
    pub placement: Placement,
    /// Popover rectangle from the last render (frame included).
    pub area: Rect,
    /// Optional title row above the items (e.g. what the menu is about).
    pub title: Option<String>,
    /// How the menu closes without a choice (Esc, a click outside).
    pub dismiss: Dismiss,
    /// While open, the menu owns the keyboard, so the page under it shows
    /// no focus (see [`ContextMenu::hides_page_focus`]). Default on.
    pub hide_owner_focus: bool,
    /// How a label cut by a narrow screen reveals its full text on hover
    /// (see [`crate::ui::reveal`]); `None` follows [`RenderCtx::reveal`].
    pub reveal: Option<crate::ui::reveal::Reveal>,
    /// How the menu is fitted to the screen ([`popup::place_with`]).
    /// Default [`Place::MENU`]: one free column from the left and right
    /// edges (given up when the menu would not fit otherwise), none above
    /// or below, and full height kept when neither side of the anchor has
    /// room, because a menu does not scroll.
    ///
    /// [`popup::place_with`]: crate::ui::popup::place_with
    pub fit: Place,
    /// Narrowest label column in cells (the frame and gutters add 6).
    /// Default 8.
    pub min_width: u16,
    /// ↑ on the first item lands on the last and ↓ on the last on the
    /// first. Default on (macOS menus wrap).
    pub wrap: bool,
    /// A control whose rectangle this frame becomes the anchor when it is
    /// registered (a keyboard-opened context menu follows its cursor row);
    /// `anchor` is the fallback. See [`ContextMenu::anchor_control`].
    pub anchor_to: Option<WidgetId>,
}

impl ContextMenu {
    /// Narrowest label column in cells. Default 8.
    pub fn min_width(mut self, cols: u16) -> Self {
        self.min_width = cols;
        self
    }

    pub fn new(id: WidgetId, items: Vec<MenuItem>) -> Self {
        let cursor = items.iter().position(|i| !i.disabled).unwrap_or(0);
        Self {
            id,
            items,
            cursor,
            anchor: Rect::ZERO,
            placement: Placement::Below,
            area: Rect::ZERO,
            title: None,
            dismiss: Dismiss::LIGHT,
            hide_owner_focus: true,
            reveal: None,
            fit: Place::MENU,
            min_width: 8,
            wrap: true,
            anchor_to: None,
        }
    }

    /// Cells kept free between the menu and the screen's left and right
    /// edges (`fit.margin`). Default 1.
    pub fn edge_margin(mut self, cells: u16) -> Self {
        self.fit.margin = cells;
        self
    }

    /// See [`Self::wrap`].
    pub fn wrap(mut self, on: bool) -> Self {
        self.wrap = on;
        self
    }

    /// See [`Self::fit`].
    pub fn fit(mut self, fit: Place) -> Self {
        self.fit = fit;
        self
    }

    pub fn dismiss(mut self, dismiss: Dismiss) -> Self {
        self.dismiss = dismiss;
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
        self.placement = placement;
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
        self.id.child(i)
    }

    pub fn locate(&self, id: WidgetId) -> Option<usize> {
        (0..self.items.len()).find(|i| self.row_id(*i) == id)
    }

    pub fn owns(&self, id: WidgetId) -> bool {
        id == self.id || id == backdrop_id(self.id) || self.locate(id).is_some()
    }

    /// Frame-inclusive size.
    pub fn size(&self) -> (u16, u16) {
        let label_w = self
            .items
            .iter()
            .map(|i| {
                width(&i.label) as u16 + i.trailing().map(|s| width(s) as u16 + 3).unwrap_or(0)
            })
            .max()
            .unwrap_or(4);
        let title_w = self.title.as_ref().map(|t| width(t) as u16).unwrap_or(0);
        // gutter + marker gap + label + 2 inset + frame
        let w = label_w.max(title_w).max(self.min_width) + 6;
        let seps = self.items.iter().filter(|i| i.separator_after).count() as u16;
        let title_rows = if self.title.is_some() { 1 } else { 0 };
        let h = self.items.len() as u16 + seps + title_rows + 2;
        (w, h)
    }

    fn placed(&self, screen: Rect) -> Rect {
        let (w, h) = self.size();
        place_with(screen, self.anchor, w, h, self.placement, self.fit)
    }

    fn step(&mut self, delta: isize) {
        let n = self.items.len();
        if n == 0 {
            return;
        }
        let mut i = self.cursor;
        for _ in 0..n {
            let next = crate::core::nav::step_index(i, n, delta, self.wrap);
            if next == i {
                // an end of a list that does not wrap
                return;
            }
            i = next;
            if !self.items[i].disabled {
                self.cursor = i;
                return;
            }
        }
    }

    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<MenuEvent>) {
        if matches!(key.code, KeyCode::Char(_)) && (key.ctrl() || key.alt()) {
            // an unassigned modified chord never performs a plain action
            return (Outcome::Ignored, None);
        }
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.step(1);
                (Outcome::Changed, None)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.step(-1);
                (Outcome::Changed, None)
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.cursor = self.items.iter().position(|i| !i.disabled).unwrap_or(0);
                (Outcome::Changed, None)
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.cursor = self.items.iter().rposition(|i| !i.disabled).unwrap_or(0);
                (Outcome::Changed, None)
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                if self.items.get(self.cursor).is_some_and(|i| !i.disabled) {
                    (Outcome::Changed, Some(MenuEvent::Chosen(self.cursor)))
                } else {
                    (Outcome::Consumed, None)
                }
            }
            KeyCode::Esc if self.dismiss.escape => (Outcome::Changed, Some(MenuEvent::Dismissed)),
            _ => (Outcome::Consumed, None),
        }
    }

    /// A completed click on `id`: a row chooses it, anywhere else dismisses
    /// (when the [`Dismiss`] policy allows an outside click).
    pub fn on_click(&mut self, id: WidgetId) -> Option<MenuEvent> {
        match self.locate(id) {
            Some(i) if !self.items[i].disabled => {
                self.cursor = i;
                Some(MenuEvent::Chosen(i))
            }
            Some(_) => None,
            None if id == self.id => None,
            None => self.on_click_outside(),
        }
    }

    pub fn on_click_outside(&mut self) -> Option<MenuEvent> {
        self.dismiss.outside_click.then_some(MenuEvent::Dismissed)
    }

    /// Place the menu inside `screen` and queue it as an overlay
    /// ([`RenderCtx::overlay`]): it lands above everything drawn after it,
    /// and a click outside reaches it through its backdrop.
    pub fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        self.prepare(screen.intersection(*buf.area()), ctx);
        let menu = self.clone();
        ctx.overlay(move |buf, ctx| {
            ctx.hits.push_barrier();
            crate::ui::popup::backdrop(ctx, menu.id, menu.dismiss, *buf.area());
            menu.paint(buf, ctx);
        });
    }

    /// Resolve the popover rectangle and let the pointer move the cursor.
    /// State changes happen here, so [`ContextMenu::paint`] can run later
    /// (as an overlay) on a copy.
    fn prepare(&mut self, screen: Rect, ctx: &RenderCtx) {
        if let Some(r) = self.anchor_to.and_then(|id| ctx.hits.area_of(id)) {
            self.anchor = r;
        }
        self.area = self.placed(screen).intersection(screen);
        if let Some(h) = ctx.interaction.live_hover()
            && let Some(i) = self.locate(h)
            && !self.items[i].disabled
        {
            self.cursor = i;
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
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::new().fg(t.border_subtle).bg(bg));
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
        let mut y = inner.y;
        if let Some(title) = &self.title {
            buf.set_string(
                inner.x + 2,
                y,
                truncate(title, inner.width.saturating_sub(3) as usize),
                t.muted().bg(bg),
            );
            y += 1;
        }
        // an open menu is always the active control: its cursor row carries
        // the selection tint whoever holds keyboard focus
        for (i, item) in self.items.iter().enumerate() {
            if y >= inner.bottom() {
                break;
            }
            let rid = self.row_id(i);
            let mut s = ctx.child_state(self.id, rid, i == self.cursor);
            s.focused = i == self.cursor;
            s.disabled = item.disabled;
            s.error = item.danger;
            let row = Rect::new(inner.x, y, inner.width, 1);
            // one highlight only: the cursor row is the highlight fill (a
            // menu row is a command, not a focus stop, so it draws no `▎`
            // bar and never the accent); hover and press walk the plane's
            // ladder. A destructive row is soft rose at rest and takes the
            // deep red cursor fill: the tone of the plane, not the alarm red
            let it = t.interactive(Kind::CursorRow(CursorStyle::Fill), s, plane);
            let mut st = it.style;
            if item.danger && !it.on_fill && !item.disabled {
                st = st.fg(t.legible(t.error_soft, st.bg.unwrap_or(bg)));
            }
            fill(buf, row, st);
            let label_w = inner.width.saturating_sub(3) as usize;
            buf.set_string(row.x + 2, y, truncate(&item.label, label_w), st);
            if !item.disabled {
                crate::ui::reveal::truncated(
                    ctx,
                    self.reveal,
                    rid,
                    Rect::new(row.x + 2, y, label_w as u16, 1),
                    &item.label,
                );
            }
            if let Some(sc) = item.trailing() {
                let sw = width(sc) as u16;
                if sw + width(&item.label) as u16 + 5 <= inner.width {
                    // a shortcut reads on the cursor fill and at rest alike;
                    // a disabled row's reason takes the row's disabled ink
                    let scs = if item.disabled {
                        st
                    } else {
                        let ink = t.row_ink(&it, t.text_secondary);
                        st.fg(t.legible(ink, st.bg.unwrap_or(bg)))
                            .remove_modifier(Modifier::BOLD)
                    };
                    buf.set_string(row.right().saturating_sub(sw + 1), y, sc, scs);
                }
            }
            ctx.clickable(rid, row);
            y += 1;
            if item.separator_after && y < inner.bottom() {
                for x in inner.x + 1..inner.right().saturating_sub(1) {
                    buf.set_string(x, y, "─", Style::new().fg(t.rule_on(bg)).bg(bg));
                }
                y += 1;
            }
        }
    }
}

// ------------------------------------------------------------------ bar

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuBarEvent {
    /// A menu opened (index).
    Opened(usize),
    /// An item was chosen: (menu, item).
    Chosen(usize, usize),
    Closed,
    /// The brand lockup was activated.
    Brand,
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
    /// arrow keys switch menus.
    pub hover_switch: bool,
    /// ←/→ past the last title lands on the first and back. Default on.
    pub wrap: bool,
    /// With a menu open the keyboard is in the menu, so the page under it
    /// shows no focus (see [`MenuBar::hides_page_focus`]). Default on.
    pub hide_owner_focus: bool,
}

impl MenuBar {
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
        }
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
        self.id.child(i)
    }

    pub fn brand_id(&self) -> WidgetId {
        self.id.sub("brand")
    }

    pub fn is_open(&self) -> bool {
        self.open.is_some()
    }

    pub fn open_index(&self) -> Option<usize> {
        self.open
            .as_ref()
            .and_then(|m| (0..self.labels.len()).find(|i| m.id == self.label_id(*i).sub("menu")))
    }

    pub fn owns(&self, id: WidgetId) -> bool {
        id == self.id
            || id == self.brand_id()
            || (self.open.is_some() && id == backdrop_id(self.id))
            || (0..self.labels.len()).any(|i| self.label_id(i) == id)
            || self.open.as_ref().is_some_and(|m| m.owns(id))
    }

    pub fn open_menu(&mut self, i: usize) {
        if i >= self.labels.len() {
            return;
        }
        self.cursor = i;
        let anchor = self.areas.get(i).copied().unwrap_or(Rect::new(0, 0, 1, 1));
        let menu = ContextMenu::new(self.label_id(i).sub("menu"), self.menus[i].clone())
            .anchor(anchor, Placement::Below);
        self.open = Some(menu);
    }

    pub fn close(&mut self) {
        self.open = None;
    }

    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<MenuBarEvent>) {
        let n = self.labels.len();
        if matches!(key.code, KeyCode::Char(_)) && (key.ctrl() || key.alt()) {
            // an unassigned modified chord never performs a plain action
            return (Outcome::Ignored, None);
        }
        if let Some(menu) = self.open.as_mut() {
            match key.code {
                KeyCode::Left | KeyCode::Char('h') if n > 1 => {
                    let i = crate::core::nav::step_index(self.cursor, n, -1, self.wrap);
                    self.open_menu(i);
                    return (Outcome::Changed, Some(MenuBarEvent::Opened(i)));
                }
                KeyCode::Right | KeyCode::Char('l') if n > 1 => {
                    let i = crate::core::nav::step_index(self.cursor, n, 1, self.wrap);
                    self.open_menu(i);
                    return (Outcome::Changed, Some(MenuBarEvent::Opened(i)));
                }
                _ => {}
            }
            let (o, ev) = menu.on_key(key);
            return match ev {
                Some(MenuEvent::Chosen(i)) => {
                    let m = self.cursor;
                    self.close();
                    (Outcome::Changed, Some(MenuBarEvent::Chosen(m, i)))
                }
                Some(MenuEvent::Dismissed) if !self.dismiss.escape => (Outcome::Consumed, None),
                Some(MenuEvent::Dismissed) => {
                    self.close();
                    (Outcome::Changed, Some(MenuBarEvent::Closed))
                }
                None => (o, None),
            };
        }
        match key.code {
            KeyCode::Left | KeyCode::Char('h') if n > 0 => {
                self.cursor = crate::core::nav::step_index(self.cursor, n, -1, self.wrap);
                (Outcome::Changed, None)
            }
            KeyCode::Right | KeyCode::Char('l') if n > 0 => {
                self.cursor = crate::core::nav::step_index(self.cursor, n, 1, self.wrap);
                (Outcome::Changed, None)
            }
            KeyCode::Enter | KeyCode::Down | KeyCode::Char(' ') if n > 0 => {
                let i = self.cursor;
                self.open_menu(i);
                (Outcome::Changed, Some(MenuBarEvent::Opened(i)))
            }
            _ => (Outcome::Ignored, None),
        }
    }

    /// A completed click. Labels toggle their menu, rows choose, the brand
    /// fires `Brand`, anything else while open closes the menu (when the
    /// [`Dismiss`] policy allows; the click is consumed either way).
    pub fn on_click(&mut self, id: WidgetId) -> (Outcome, Option<MenuBarEvent>) {
        if self.open.is_some() && id == backdrop_id(self.id) {
            if !self.dismiss.outside_click {
                return (Outcome::Consumed, None);
            }
            self.close();
            return (Outcome::Changed, Some(MenuBarEvent::Closed));
        }
        if id == self.brand_id() {
            self.close();
            return (Outcome::Changed, Some(MenuBarEvent::Brand));
        }
        if let Some(i) = (0..self.labels.len()).find(|i| self.label_id(*i) == id) {
            if self.open_index() == Some(i) {
                self.close();
                return (Outcome::Changed, Some(MenuBarEvent::Closed));
            }
            self.open_menu(i);
            return (Outcome::Changed, Some(MenuBarEvent::Opened(i)));
        }
        if let Some(menu) = self.open.as_mut() {
            return match menu.on_click(id) {
                Some(MenuEvent::Chosen(item)) => {
                    let m = self.cursor;
                    self.close();
                    (Outcome::Changed, Some(MenuBarEvent::Chosen(m, item)))
                }
                Some(MenuEvent::Dismissed) if !self.dismiss.outside_click => {
                    (Outcome::Consumed, None)
                }
                Some(MenuEvent::Dismissed) => {
                    self.close();
                    (Outcome::Changed, Some(MenuBarEvent::Closed))
                }
                None => (Outcome::Consumed, None),
            };
        }
        (Outcome::Ignored, None)
    }

    /// Hovering another label while a menu is open switches to it. The bar
    /// applies this itself on every render, so owners need not call it.
    pub fn on_hover(&mut self, id: Option<WidgetId>) -> Outcome {
        if self.open.is_none() || !self.hover_switch {
            return Outcome::Ignored;
        }
        if let Some(id) = id
            && let Some(i) = (0..self.labels.len()).find(|i| self.label_id(*i) == id)
            && self.open_index() != Some(i)
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
        // (unless `hover_switch` is off)
        self.on_hover(ctx.interaction.live_hover());
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
        for (i, label) in self.labels.iter().enumerate() {
            let lid = self.label_id(i);
            let text = format!(" {label} ");
            let w = width(&text) as u16;
            if x + w > area.right() {
                self.areas.push(Rect::ZERO);
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
            ctx.clickable(lid, r);
            x += w + 1;
        }
        let Some(m) = self.open.as_mut() else {
            return;
        };
        // keep the popover under its (possibly re-laid-out) label
        if let Some(i) = open
            && let Some(r) = self.areas.get(i)
        {
            m.anchor = *r;
        }
        m.prepare(*buf.area(), ctx);
        let menu = m.clone();
        let (id, dismiss) = (self.id, self.dismiss);
        // the titles (and the brand) stay reachable above the backdrop: a
        // click or a hover on one of them is a menu-bar gesture, not an
        // outside click
        let mut titles: Vec<(WidgetId, Rect)> = (0..self.labels.len())
            .map(|i| (self.label_id(i), self.areas[i]))
            .collect();
        if self.brand.is_some() {
            titles.push((self.brand_id(), self.brand_area));
        }
        ctx.overlay(move |buf, ctx| {
            ctx.hits.push_barrier();
            crate::ui::popup::backdrop(ctx, id, dismiss, *buf.area());
            for (tid, r) in titles {
                ctx.clickable(tid, r);
            }
            menu.paint(buf, ctx);
        });
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

    #[test]
    fn keyboard_skips_disabled_wraps_and_chooses() {
        let mut m = ContextMenu::new(WidgetId::of("m"), items());
        assert_eq!(m.cursor, 0);
        m.on_key(&key(KeyCode::Down));
        assert_eq!(m.cursor, 2, "disabled row skipped");
        m.on_key(&key(KeyCode::Down));
        assert_eq!(m.cursor, 3);
        m.on_key(&key(KeyCode::Down));
        assert_eq!(m.cursor, 0, "wraps");
        m.on_key(&key(KeyCode::End));
        assert_eq!(m.cursor, 3);
        let (_, ev) = m.on_key(&key(KeyCode::Enter));
        assert_eq!(ev, Some(MenuEvent::Chosen(3)));
        let (_, ev) = m.on_key(&key(KeyCode::Esc));
        assert_eq!(ev, Some(MenuEvent::Dismissed));
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
        assert_eq!(m.on_click(row3), Some(MenuEvent::Chosen(3)));
        assert_eq!(m.on_click(m.row_id(1)), None, "disabled rows do nothing");
        assert_eq!(
            m.on_click(WidgetId::of("elsewhere")),
            Some(MenuEvent::Dismissed)
        );
        // a click outside lands on the menu's backdrop; the policy decides
        assert_eq!(hits.hit(Position::new(50, 18)), Some(backdrop_id(m.id)));
        let mut sticky = m.clone().dismiss(Dismiss::ESCAPE);
        assert_eq!(sticky.on_click(backdrop_id(m.id)), None);
        assert_eq!(
            sticky.on_key(&key(KeyCode::Esc)).1,
            Some(MenuEvent::Dismissed)
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
        assert_eq!(m.cursor, 3);
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
        assert_eq!(ev, Some(MenuBarEvent::Opened(0)));
        assert!(bar.is_open());
        let (_, ev) = bar.on_key(&key(KeyCode::Right));
        assert_eq!(ev, Some(MenuBarEvent::Opened(1)));
        assert_eq!(bar.open_index(), Some(1));
        let (_, ev) = bar.on_key(&key(KeyCode::Enter));
        assert_eq!(ev, Some(MenuBarEvent::Chosen(1, 0)));
        assert!(!bar.is_open());
        // click toggles, click on the brand fires Brand
        let (_, ev) = bar.on_click(bar.label_id(0));
        assert_eq!(ev, Some(MenuBarEvent::Opened(0)));
        bar.render(Rect::new(0, 0, 60, 1), &mut buf, &mut ctx, t.canvas);
        ctx.flush_overlays(&mut buf);
        let popover = bar.open.as_ref().unwrap().area;
        assert_eq!(popover.y, 1, "anchored beneath the label");
        assert_eq!(popover.x, bar.areas[0].x);
        let (_, ev) = bar.on_click(bar.label_id(0));
        assert_eq!(ev, Some(MenuBarEvent::Closed));
        let (_, ev) = bar.on_click(bar.brand_id());
        assert_eq!(ev, Some(MenuBarEvent::Brand));
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
        assert_eq!(ev, Some(MenuBarEvent::Closed));
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

    #[test]
    fn a_destructive_cursor_row_is_not_grey_on_grey_at_256() {
        let t = Theme::for_level(crate::theme::ColorLevel::Ansi256);
        let mut m = ContextMenu::new(WidgetId::of("m"), items()).at(Position::new(1, 1));
        m.cursor = 3;
        let b = paint_menu(&mut m, &t, Interaction::default());
        let (x, y) = row_y(&b, "Close");
        assert_eq!(b[(x, y)].bg, t.highlight_danger);
        assert!(matches!(b[(x, y)].bg, Color::Indexed(i) if i < 232));
    }

    #[test]
    fn an_unavailable_item_is_disabled_and_says_why() {
        let t = Theme::junie();
        let mut m = ContextMenu::new(
            WidgetId::of("m"),
            vec![
                MenuItem::new("Open"),
                MenuItem::new("Stage image")
                    .shortcut("Ctrl+I")
                    .reason("Not in this preview"),
            ],
        )
        .at(Position::new(1, 1));
        assert!(m.items[1].disabled);
        let b = paint_menu(&mut m, &t, Interaction::default());
        let (_, y) = row_y(&b, "Stage image");
        let row: String = (0..b.area.width).map(|x| b[(x, y)].symbol()).collect();
        assert!(row.contains("Not in this preview"), "{row}");
        assert!(!row.contains("Ctrl+I"), "{row}");
        // the reason takes the row's disabled ink, not the shortcut ink
        let (rx, ry) = row_y(&b, "Not in this preview");
        let (lx, ly) = row_y(&b, "Stage image");
        assert_eq!(b[(rx, ry)].fg, b[(lx, ly)].fg);
        // the keyboard never lands on it
        m.on_key(&key(KeyCode::Down));
        assert_eq!(m.cursor, 0);
    }
}
