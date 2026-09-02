//! Searchable picker: a centered modal with a query field and grouped,
//! ranked rows. Used for quick switchers, tab lists and enum pickers.
//! Ranking is the owner's job (it supplies the rows for the query).

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Modifier, Style};

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::core::scroll::ScrollState;
use crate::ui::ctx::{RenderCtx, fill};
use crate::ui::popup::{Placement, place};
use crate::ui::text::{truncate, width};
use crate::widgets::scrollbar;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickerItem {
    pub label: String,
    pub detail: String,
    pub glyph: &'static str,
    pub group: &'static str,
    /// Trailing hint on the row (e.g. "open").
    pub tag: Option<&'static str>,
    pub matched: Vec<usize>,
    pub disabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum PickerStatus {
    #[default]
    Ready,
    /// Spinner row in the list area; Enter is refused.
    Loading(String),
    /// `! message` with an optional faint detail; Enter is refused.
    Error {
        message: String,
        detail: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub struct Picker {
    pub id: WidgetId,
    pub status: PickerStatus,
    pub title: String,
    pub placeholder: String,
    pub query: String,
    pub items: Vec<PickerItem>,
    pub cursor: usize,
    pub scroll: ScrollState,
    pub width: u16,
    pub max_rows: u16,
    /// Optional right-aligned scope label in the query row.
    pub scope: Option<String>,
    pub empty_text: String,
    pub area: Rect,
    /// Show the query field (false for fixed-choice pickers).
    pub searchable: bool,
    /// The cursor moved since the last render: the next render pulls it
    /// into view. Wheel scrolling leaves it alone so the viewport and the
    /// selection never fight.
    cursor_dirty: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickerEvent {
    /// Query text changed; the owner re-supplies items.
    QueryChanged,
    Chosen(usize),
    /// Chosen with the alternate modifier (e.g. open in new tab).
    ChosenAlt(usize),
    /// Secondary action on the cursor row (e.g. close tab).
    Secondary(usize),
    NextScope,
    Cancelled,
    /// Backspace on an empty query: the owner rewinds one step.
    Back,
}

impl Picker {
    pub fn new(id: WidgetId, title: &str) -> Self {
        Self {
            id,
            status: PickerStatus::Ready,
            title: title.to_owned(),
            placeholder: "Type to search…".into(),
            query: String::new(),
            items: vec![],
            cursor: 0,
            scroll: ScrollState::default(),
            width: 64,
            max_rows: 12,
            scope: None,
            empty_text: "No matches".into(),
            area: Rect::ZERO,
            searchable: true,
            cursor_dirty: true,
        }
    }

    pub fn set_items(&mut self, items: Vec<PickerItem>) {
        self.items = items;
        self.cursor = self.items.iter().position(|i| !i.disabled).unwrap_or(0);
        self.scroll = ScrollState::new(self.items.len());
        self.cursor_dirty = true;
    }

    /// Move the cursor and ask the next render to keep it in view.
    pub fn set_cursor(&mut self, i: usize) {
        self.cursor = i.min(self.items.len().saturating_sub(1));
        self.cursor_dirty = true;
        self.scroll.ensure_visible(self.cursor);
    }

    pub fn row_id(&self, i: usize) -> WidgetId {
        self.id.child(i)
    }
    pub fn locate(&self, id: WidgetId) -> Option<usize> {
        self.scroll.visible_range().find(|&i| self.row_id(i) == id)
    }
    pub fn owns(&self, id: WidgetId) -> bool {
        id == self.id || id == scrollbar::id_for(self.id) || self.locate(id).is_some()
    }

    fn step(&mut self, delta: isize) {
        if self.items.is_empty() {
            return;
        }
        let n = self.items.len() as isize;
        let mut c = self.cursor as isize;
        for _ in 0..n {
            c = (c + delta).clamp(0, n - 1);
            if !self.items[c as usize].disabled {
                break;
            }
            if c == 0 || c == n - 1 {
                break;
            }
        }
        self.cursor = c as usize;
        self.cursor_dirty = true;
        self.scroll.ensure_visible(self.cursor);
    }

    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<PickerEvent>) {
        match key.code {
            KeyCode::Esc => {
                if self.searchable && !self.query.is_empty() {
                    self.query.clear();
                    return (Outcome::Changed, Some(PickerEvent::QueryChanged));
                }
                (Outcome::Changed, Some(PickerEvent::Cancelled))
            }
            KeyCode::Enter => {
                if self.status == PickerStatus::Ready
                    && self.items.get(self.cursor).is_some_and(|i| !i.disabled)
                {
                    let ev = if key.alt() {
                        PickerEvent::ChosenAlt(self.cursor)
                    } else {
                        PickerEvent::Chosen(self.cursor)
                    };
                    (Outcome::Changed, Some(ev))
                } else {
                    (Outcome::Consumed, None)
                }
            }
            KeyCode::Down => {
                self.step(1);
                (Outcome::Changed, None)
            }
            KeyCode::Up => {
                self.step(-1);
                (Outcome::Changed, None)
            }
            KeyCode::Char('n') | KeyCode::Char('j') if key.ctrl() => {
                self.step(1);
                (Outcome::Changed, None)
            }
            KeyCode::Char('p') | KeyCode::Char('k') if key.ctrl() => {
                self.step(-1);
                (Outcome::Changed, None)
            }
            KeyCode::PageDown => {
                self.step(self.max_rows as isize);
                (Outcome::Changed, None)
            }
            KeyCode::PageUp => {
                self.step(-(self.max_rows as isize));
                (Outcome::Changed, None)
            }
            KeyCode::Tab => (Outcome::Changed, Some(PickerEvent::NextScope)),
            KeyCode::Delete => (Outcome::Changed, Some(PickerEvent::Secondary(self.cursor))),
            KeyCode::Backspace if self.query.is_empty() => {
                (Outcome::Changed, Some(PickerEvent::Back))
            }
            KeyCode::Backspace if self.searchable => {
                self.query.pop();
                (Outcome::Changed, Some(PickerEvent::QueryChanged))
            }
            KeyCode::Char('u') if key.ctrl() && self.searchable => {
                self.query.clear();
                (Outcome::Changed, Some(PickerEvent::QueryChanged))
            }
            KeyCode::Char(c) if self.searchable && !key.ctrl() && !key.alt() => {
                self.query.push(c);
                (Outcome::Changed, Some(PickerEvent::QueryChanged))
            }
            KeyCode::Char('j') if !self.searchable => {
                self.step(1);
                (Outcome::Changed, None)
            }
            KeyCode::Char('k') if !self.searchable => {
                self.step(-1);
                (Outcome::Changed, None)
            }
            _ => (Outcome::Consumed, None),
        }
    }

    pub fn on_click(&mut self, id: WidgetId) -> Option<PickerEvent> {
        let i = self.locate(id)?;
        if self.items[i].disabled || self.status != PickerStatus::Ready {
            return None;
        }
        self.cursor = i;
        self.cursor_dirty = true;
        Some(PickerEvent::Chosen(i))
    }

    /// Wheel scrolls the viewport and keeps the selection where it is.
    pub fn on_wheel(&mut self, delta: i32) -> Outcome {
        let before = self.scroll.offset;
        self.scroll.scroll_by(delta as isize);
        if self.scroll.offset == before {
            Outcome::Consumed
        } else {
            Outcome::Changed
        }
    }

    pub fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, hints: &str) {
        let t = ctx.theme;
        // dim + modal like Dialog
        let dim = Rect::new(
            screen.x,
            screen.y,
            screen.width,
            screen.height.saturating_sub(1),
        );
        for pos in dim.positions() {
            if let Some(c) = buf.cell_mut(pos) {
                let st = t.backdrop(c.style());
                c.set_style(st);
                c.modifier = Modifier::empty();
            }
        }
        ctx.begin_modal();
        let rows = if self.status == PickerStatus::Ready {
            (self.items.len() as u16).clamp(1, self.max_rows)
        } else {
            (self.items.len() as u16).clamp(2, self.max_rows)
        };
        let query_rows = if self.searchable { 2 } else { 0 };
        let h = (2 + 1 + query_rows + rows + 2).min(screen.height.saturating_sub(2));
        let w = self.width.min(screen.width.saturating_sub(4));
        let area = place(screen, Rect::ZERO, w, h, Placement::Center);
        let _ = Constraint::Length(0);
        self.area = area;
        let bg = t.surface_elevated;
        fill(buf, area, Style::new().bg(bg));
        let block = ratatui::widgets::Block::new()
            .borders(ratatui::widgets::Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(t.border(true).bg(bg));
        ratatui::widgets::Widget::render(block, area, buf);
        ctx.hits.register(self.id, area);
        let inner = area.inner(ratatui::layout::Margin::new(2, 1));
        if inner.is_empty() {
            return;
        }
        let mut y = inner.y;
        buf.set_string(
            inner.x,
            y,
            truncate(&self.title, inner.width as usize),
            t.title().bg(bg),
        );
        if let Some(scope) = &self.scope {
            // the scope never overwrites the title: it takes the room that is left
            let room = inner.width.saturating_sub(width(&self.title) as u16 + 3) as usize;
            if room >= 6 {
                let scope = truncate(scope, room);
                let sw = width(&scope) as u16;
                buf.set_string(
                    inner.right().saturating_sub(sw),
                    y,
                    &scope,
                    t.muted().bg(bg),
                );
            }
        }
        y += 1;
        if self.searchable {
            let field = Rect::new(inner.x, y, inner.width, 1);
            let fs = t.field_style(crate::ui::ctx::VisualState {
                focused: true,
                editing: true,
                ..Default::default()
            });
            fill(buf, field, fs);
            buf.set_string(
                field.x,
                y,
                "▎",
                Style::new().fg(t.focus).bg(fs.bg.unwrap_or(bg)),
            );
            if self.query.is_empty() {
                buf.set_string(
                    field.x + 2,
                    y,
                    truncate(&self.placeholder, field.width.saturating_sub(3) as usize),
                    fs.fg(t.text_muted),
                );
            } else {
                buf.set_string(
                    field.x + 2,
                    y,
                    truncate(&self.query, field.width.saturating_sub(3) as usize),
                    fs.add_modifier(Modifier::UNDERLINED)
                        .underline_color(t.accent),
                );
            }
            ctx.set_cursor(ratatui::layout::Position::new(
                field.x + 2 + width(&self.query).min(field.width.saturating_sub(3) as usize) as u16,
                y,
            ));
            y += 2;
        }
        let list = Rect::new(
            inner.x,
            y,
            inner.width,
            inner.bottom().saturating_sub(y + 1),
        );
        self.scroll.set_content(self.items.len());
        self.scroll.set_viewport(list.height as usize);
        // only a cursor move pulls the viewport; a wheel scroll must survive
        if self.cursor_dirty {
            self.scroll.ensure_visible(self.cursor);
            self.cursor_dirty = false;
        }
        match &self.status {
            PickerStatus::Loading(label) => {
                crate::widgets::progress::render_spinner(
                    Rect::new(list.x + 1, list.y, list.width.saturating_sub(1), 1),
                    buf,
                    ctx,
                    label,
                    bg,
                );
                // hints row still applies
                if !hints.is_empty() {
                    let hy = inner.bottom().saturating_sub(1);
                    buf.set_string(
                        inner.x,
                        hy,
                        truncate(hints, inner.width as usize),
                        t.faint().bg(bg),
                    );
                }
                return;
            }
            PickerStatus::Error { message, detail } => {
                let mut e = crate::widgets::empty::EmptyState::error(message);
                if let Some(d) = detail {
                    e = e.hint(d);
                }
                crate::widgets::empty::render(list, buf, t, &e, bg);
                if !hints.is_empty() {
                    let hy = inner.bottom().saturating_sub(1);
                    buf.set_string(
                        inner.x,
                        hy,
                        truncate(hints, inner.width as usize),
                        t.faint().bg(bg),
                    );
                }
                return;
            }
            PickerStatus::Ready => {}
        }
        if self.items.is_empty() {
            crate::widgets::empty::render(
                list,
                buf,
                t,
                &crate::widgets::empty::EmptyState::new(&self.empty_text),
                bg,
            );
        }
        let has_sb = self.scroll.overflows();
        let row_w = list.width.saturating_sub(u16::from(has_sb));
        // column widths come from every item, not just the visible ones,
        // so scrolling never shifts the columns
        let label_col = (self
            .items
            .iter()
            .map(|i| width(&i.label) as u16)
            .max()
            .unwrap_or(6))
        .clamp(6, (row_w * 45 / 100).max(6));
        let tag_col = self
            .items
            .iter()
            .filter_map(|i| i.tag.map(|t| width(t) as u16))
            .max()
            .unwrap_or(0);
        let group_col = self
            .items
            .iter()
            .map(|i| width(i.group) as u16)
            .max()
            .unwrap_or(0);
        let mut last_group = "";
        for (k, i) in self.scroll.visible_range().enumerate() {
            let ry = list.y + k as u16;
            let it = &self.items[i];
            let rid = self.row_id(i);
            let mut s = ctx.state(rid);
            s.focused = i == self.cursor;
            s.disabled = it.disabled;
            let st = t.row(s, bg);
            let row = Rect::new(list.x, ry, list.width.saturating_sub(u16::from(has_sb)), 1);
            fill(buf, row, st);
            buf.set_string(row.x, ry, "▎", t.gutter(s, st.bg.unwrap_or(bg), false));
            // group label inline (first row of a group shows it right-aligned muted)
            let show_group = it.group != last_group && !it.group.is_empty();
            last_group = it.group;
            buf.set_string(
                row.x + 1,
                ry,
                it.glyph,
                st.fg(if s.focused {
                    t.text_primary
                } else {
                    t.text_muted
                })
                .remove_modifier(Modifier::BOLD),
            );
            // fixed columns: label · detail · tag · group, so rows line up
            let mut x = row.x + 3;
            let label = truncate(&it.label, label_col as usize);
            for (bi, ch) in label.char_indices() {
                let mut cs = st;
                if it.matched.contains(&bi) {
                    cs = cs.add_modifier(Modifier::BOLD);
                } else if !s.focused {
                    cs = cs.remove_modifier(Modifier::BOLD);
                }
                let g = ch.to_string();
                buf.set_string(x, ry, &g, cs);
                x += width(&g) as u16;
            }
            let mut rx = row.right();
            if group_col > 0 {
                rx = rx.saturating_sub(group_col + 1);
                if show_group {
                    buf.set_string(
                        rx,
                        ry,
                        it.group,
                        st.fg(t.text_faint).remove_modifier(Modifier::BOLD),
                    );
                }
            }
            if tag_col > 0 {
                rx = rx.saturating_sub(tag_col + 2);
                if let Some(tag) = it.tag {
                    buf.set_string(
                        rx,
                        ry,
                        tag,
                        st.fg(t.text_secondary).remove_modifier(Modifier::BOLD),
                    );
                }
            }
            if !it.detail.is_empty() {
                let dx = row.x + 3 + label_col + 2;
                let room = rx.saturating_sub(dx + 1) as usize;
                if room >= 4 {
                    buf.set_string(
                        dx,
                        ry,
                        truncate(&it.detail, room),
                        st.fg(t.text_muted).remove_modifier(Modifier::BOLD),
                    );
                }
            }
            if !it.disabled {
                ctx.clickable(rid, row);
            }
        }
        if has_sb {
            scrollbar::render_vertical(
                Rect::new(list.right() - 1, list.y, 1, list.height),
                buf,
                ctx,
                self.id,
                &self.scroll,
                true,
            );
        }
        // hints row: owners with a shell-level hint bar pass an empty string
        if !hints.is_empty() {
            let hy = inner.bottom().saturating_sub(1);
            buf.set_string(
                inner.x,
                hy,
                truncate(hints, inner.width as usize),
                t.faint().bg(bg),
            );
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

    fn picker(n: usize) -> Picker {
        let mut p = Picker::new(WidgetId::of("p"), "Palette");
        p.max_rows = 6;
        p.set_items(
            (0..n)
                .map(|i| PickerItem {
                    label: format!("Item {i:02}"),
                    detail: String::new(),
                    glyph: "·",
                    group: "",
                    tag: None,
                    matched: vec![],
                    disabled: false,
                })
                .collect(),
        );
        p
    }

    fn render(p: &mut Picker) -> String {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
        p.render(Rect::new(0, 0, 80, 24), &mut buf, &mut ctx, "");
        let mut out = String::new();
        for y in 0..24 {
            for x in 0..80 {
                out.push_str(buf[(x, y)].symbol());
            }
            out.push('\n');
        }
        out
    }

    #[test]
    fn wheel_scrolls_the_rows_and_survives_the_next_render() {
        let mut p = picker(30);
        let before = render(&mut p);
        assert!(before.contains("Item 00"));
        assert!(!before.contains("Item 10"));
        p.on_wheel(3);
        let after = render(&mut p);
        assert!(!after.contains("Item 00"), "wheel moved the viewport");
        assert!(after.contains("Item 03"));
        assert_eq!(p.scroll.offset, 3);
        // a second render keeps the offset
        let again = render(&mut p);
        assert_eq!(again, after);
        assert_eq!(p.scroll.offset, 3);
        assert_eq!(p.cursor, 0, "selection is preserved while wheel scrolling");
        p.on_wheel(-3);
        let back = render(&mut p);
        assert_eq!(back, before);
    }

    #[test]
    fn keyboard_navigation_pulls_the_cursor_back_into_view() {
        let mut p = picker(30);
        render(&mut p);
        p.on_wheel(10);
        render(&mut p);
        assert_eq!(p.scroll.offset, 10);
        let key = Key {
            code: KeyCode::Down,
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        };
        p.on_key(&key);
        let s = render(&mut p);
        assert_eq!(p.cursor, 1);
        assert!(p.scroll.visible_range().contains(&p.cursor));
        assert!(s.contains("Item 01"));
    }

    #[test]
    fn wheel_at_the_boundary_is_consumed_not_changed() {
        let mut p = picker(3);
        render(&mut p);
        assert_eq!(p.on_wheel(1), Outcome::Consumed);
        let mut p = picker(30);
        render(&mut p);
        assert_eq!(p.on_wheel(1), Outcome::Changed);
    }
}
