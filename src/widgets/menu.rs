//! Anchored menus: a context menu popover that opens at a position or under
//! an anchor rectangle, and a menu bar whose labels open the same popover
//! beneath them. Rows are list rows (bar gutter, `accent_bg` on the cursor,
//! hover lifts); shortcuts sit right-aligned in the muted tone; danger rows
//! use the error tone; disabled rows are faint and cannot be chosen.

use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Modifier, Style};

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::ui::ctx::{RenderCtx, fill};
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
}

impl MenuItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            shortcut: None,
            disabled: false,
            danger: false,
            separator_after: false,
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
    pub fn danger(mut self) -> Self {
        self.danger = true;
        self
    }
    pub fn separator(mut self) -> Self {
        self.separator_after = true;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    /// Under the anchor, left edges aligned.
    Below,
    /// Above the anchor, left edges aligned.
    Above,
    /// To the right of the anchor, top edges aligned.
    Right,
}

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
}

impl ContextMenu {
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
        }
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
        id == self.id || self.locate(id).is_some()
    }

    /// Frame-inclusive size.
    pub fn size(&self) -> (u16, u16) {
        let label_w = self
            .items
            .iter()
            .map(|i| width(&i.label) as u16 + i.shortcut.map(|s| width(s) as u16 + 3).unwrap_or(0))
            .max()
            .unwrap_or(4);
        let title_w = self.title.as_ref().map(|t| width(t) as u16).unwrap_or(0);
        // gutter + marker gap + label + 2 inset + frame
        let w = label_w.max(title_w).max(8) + 6;
        let seps = self.items.iter().filter(|i| i.separator_after).count() as u16;
        let title_rows = if self.title.is_some() { 1 } else { 0 };
        let h = self.items.len() as u16 + seps + title_rows + 2;
        (w, h)
    }

    fn placed(&self, screen: Rect) -> Rect {
        let (w, h) = self.size();
        let w = w.min(screen.width);
        let h = h.min(screen.height);
        let (mut x, mut y) = match self.placement {
            Placement::Below => (self.anchor.x, self.anchor.bottom()),
            Placement::Above => (self.anchor.x, self.anchor.y.saturating_sub(h)),
            Placement::Right => (self.anchor.right(), self.anchor.y),
        };
        if y + h > screen.bottom() {
            // flip above when there is more room there
            let above = self.anchor.y.saturating_sub(h);
            y = if self.placement == Placement::Below && above >= screen.y {
                above
            } else {
                screen.bottom().saturating_sub(h)
            };
        }
        if y < screen.y {
            y = screen.y;
        }
        if x + w > screen.right() {
            x = screen.right().saturating_sub(w);
        }
        if x < screen.x {
            x = screen.x;
        }
        Rect::new(x, y, w, h)
    }

    fn step(&mut self, delta: isize) {
        let n = self.items.len();
        if n == 0 {
            return;
        }
        let mut i = self.cursor as isize;
        for _ in 0..n {
            i = (i + delta).rem_euclid(n as isize);
            if !self.items[i as usize].disabled {
                self.cursor = i as usize;
                return;
            }
        }
    }

    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<MenuEvent>) {
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
            KeyCode::Esc => (Outcome::Changed, Some(MenuEvent::Dismissed)),
            _ => (Outcome::Consumed, None),
        }
    }

    /// A completed click on `id`: a row chooses it, anywhere else dismisses.
    pub fn on_click(&mut self, id: WidgetId) -> Option<MenuEvent> {
        match self.locate(id) {
            Some(i) if !self.items[i].disabled => {
                self.cursor = i;
                Some(MenuEvent::Chosen(i))
            }
            Some(_) => None,
            None if id == self.id => None,
            None => Some(MenuEvent::Dismissed),
        }
    }

    pub fn on_click_outside(&mut self) -> Option<MenuEvent> {
        Some(MenuEvent::Dismissed)
    }

    pub fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let area = self.placed(screen).intersection(*buf.area());
        self.area = area;
        if area.is_empty() {
            return;
        }
        // the pointer moves the cursor
        if let Some(h) = ctx.interaction.hover
            && let Some(i) = self.locate(h)
            && !self.items[i].disabled
        {
            self.cursor = i;
        }
        let bg = t.popover;
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
            let mut s = ctx.state(rid);
            s.selected = i == self.cursor;
            s.focused = i == self.cursor;
            s.disabled = item.disabled;
            if item.disabled {
                s.hovered = false;
            }
            let row = Rect::new(inner.x, y, inner.width, 1);
            let mut st = t.row(s, bg);
            if item.danger && !item.disabled {
                st = st.fg(t.error);
            }
            fill(buf, row, st);
            buf.set_string(row.x, y, "▎", t.gutter(s, st.bg.unwrap_or(bg), false));
            let label_w = inner.width.saturating_sub(3) as usize;
            buf.set_string(row.x + 2, y, truncate(&item.label, label_w), st);
            if let Some(sc) = item.shortcut {
                let sw = width(sc) as u16;
                if sw + width(&item.label) as u16 + 5 <= inner.width {
                    let scs = if item.disabled {
                        st
                    } else {
                        st.fg(t.text_muted).remove_modifier(Modifier::BOLD)
                    };
                    buf.set_string(row.right().saturating_sub(sw + 1), y, sc, scs);
                }
            }
            ctx.clickable(rid, row);
            y += 1;
            if item.separator_after && y < inner.bottom() {
                for x in inner.x + 1..inner.right().saturating_sub(1) {
                    buf.set_string(x, y, "─", Style::new().fg(t.border_subtle).bg(bg));
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
        }
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
        if let Some(menu) = self.open.as_mut() {
            match key.code {
                KeyCode::Left | KeyCode::Char('h') if n > 1 => {
                    let i = (self.cursor + n - 1) % n;
                    self.open_menu(i);
                    return (Outcome::Changed, Some(MenuBarEvent::Opened(i)));
                }
                KeyCode::Right | KeyCode::Char('l') if n > 1 => {
                    let i = (self.cursor + 1) % n;
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
                Some(MenuEvent::Dismissed) => {
                    self.close();
                    (Outcome::Changed, Some(MenuBarEvent::Closed))
                }
                None => (o, None),
            };
        }
        match key.code {
            KeyCode::Left | KeyCode::Char('h') if n > 0 => {
                self.cursor = (self.cursor + n - 1) % n;
                (Outcome::Changed, None)
            }
            KeyCode::Right | KeyCode::Char('l') if n > 0 => {
                self.cursor = (self.cursor + 1) % n;
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
    /// fires `Brand`, anything else while open closes the menu.
    pub fn on_click(&mut self, id: WidgetId) -> (Outcome, Option<MenuBarEvent>) {
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
                Some(MenuEvent::Dismissed) => {
                    self.close();
                    (Outcome::Changed, Some(MenuBarEvent::Closed))
                }
                None => (Outcome::Consumed, None),
            };
        }
        (Outcome::Ignored, None)
    }

    /// Hovering another label while a menu is open switches to it.
    pub fn on_hover(&mut self, id: Option<WidgetId>) -> Outcome {
        if self.open.is_none() {
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

    /// Draw the bar row. Call `render_open` after the body so the popover
    /// sits on top.
    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return;
        }
        let t = ctx.theme;
        fill(buf, area, Style::new().bg(bg));
        ctx.control(self.id, area, false);
        let focused = ctx.interaction.focused(self.id);
        let mut x = area.x + 1;
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
            let hovered = ctx.interaction.hovered(lid);
            let is_open = open == Some(i);
            let is_cursor = focused && self.cursor == i && open.is_none();
            let mut st = Style::new().fg(t.text_secondary).bg(bg);
            if is_open {
                st = st
                    .fg(t.text_primary)
                    .bg(t.surface_elevated)
                    .add_modifier(Modifier::BOLD);
            } else if hovered {
                st = st.fg(t.text_primary).bg(t.lift(bg));
            }
            if is_cursor {
                st = st.fg(t.text_primary).add_modifier(Modifier::BOLD);
                buf.set_string(
                    x.saturating_sub(1),
                    area.y,
                    "▎",
                    Style::new().fg(t.focus).bg(bg),
                );
            }
            buf.set_string(x, area.y, &text, st);
            let r = Rect::new(x, area.y, w, 1);
            self.areas.push(r);
            ctx.clickable(lid, r);
            x += w + 1;
        }
        if let Some(m) = self.open.as_mut() {
            // keep the popover under its (possibly re-laid-out) label
            if let Some(i) = open
                && let Some(r) = self.areas.get(i)
            {
                m.anchor = *r;
            }
        }
    }

    /// Draw the open dropdown over `screen`.
    pub fn render_open(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        if let Some(m) = self.open.as_mut() {
            m.render(screen, buf, ctx);
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
        let row3 = m.row_id(3);
        let area = hits.area_of(row3).expect("row registered");
        assert_eq!(hits.hit(Position::new(area.x + 2, area.y)), Some(row3));
        assert_eq!(m.on_click(row3), Some(MenuEvent::Chosen(3)));
        assert_eq!(m.on_click(m.row_id(1)), None, "disabled rows do nothing");
        assert_eq!(
            m.on_click(WidgetId::of("elsewhere")),
            Some(MenuEvent::Dismissed)
        );
        // danger row is drawn in the error tone, shortcut right-aligned
        assert_eq!(buf[(area.x + 2, area.y)].fg, t.error);
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
        bar.render_open(Rect::new(0, 0, 60, 20), &mut buf, &mut ctx);
        let popover = bar.open.as_ref().unwrap().area;
        assert_eq!(popover.y, 1, "anchored beneath the label");
        assert_eq!(popover.x, bar.areas[0].x);
        let (_, ev) = bar.on_click(bar.label_id(0));
        assert_eq!(ev, Some(MenuBarEvent::Closed));
        let (_, ev) = bar.on_click(bar.brand_id());
        assert_eq!(ev, Some(MenuBarEvent::Brand));
        // Esc closes an open menu
        bar.open_menu(1);
        let (_, ev) = bar.on_key(&key(KeyCode::Esc));
        assert_eq!(ev, Some(MenuBarEvent::Closed));
    }
}
