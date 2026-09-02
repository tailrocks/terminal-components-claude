//! Label / value facts, e.g. a connection's details or a plan node's
//! metadata. Labels are muted and right-padded to a shared width.

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Modifier};

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::core::scroll::ScrollState;
use crate::theme::{Theme, Tone};
use crate::ui::ctx::{RenderCtx, fill};
use crate::widgets::scrollbar;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prop {
    pub label: String,
    pub value: String,
    pub tone: Tone,
    pub wrap: bool,
    /// This row may be copied with `y`; everything else is read-only.
    pub copyable: bool,
}

impl Prop {
    pub fn new(label: &str, value: impl Into<String>) -> Self {
        Self {
            label: label.to_owned(),
            value: value.into(),
            tone: Tone::Normal,
            wrap: false,
            copyable: false,
        }
    }
    pub fn copyable(mut self) -> Self {
        self.copyable = true;
        self
    }
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }
    pub fn wrap(mut self) -> Self {
        self.wrap = true;
        self
    }
}

/// Returns the number of rows used.
pub fn render(area: Rect, buf: &mut Buffer, t: &Theme, props: &[Prop], bg: Color) -> u16 {
    let area = area.intersection(*buf.area());
    if area.is_empty() {
        return 0;
    }
    let label_w = props
        .iter()
        .map(|p| crate::ui::text::width(&p.label))
        .max()
        .unwrap_or(0) as u16
        + 2;
    let mut y = area.y;
    for p in props {
        if y >= area.bottom() {
            break;
        }
        buf.set_string(area.x, y, &p.label, t.muted().bg(bg));
        let vw = area.width.saturating_sub(label_w) as usize;
        let style = ratatui::style::Style::new().fg(t.tone(p.tone)).bg(bg);
        if p.wrap {
            for line in crate::ui::text::wrap(&p.value, vw.max(4)) {
                if y >= area.bottom() {
                    break;
                }
                buf.set_string(area.x + label_w, y, &line, style);
                y += 1;
            }
        } else {
            buf.set_string(
                area.x + label_w,
                y,
                crate::ui::text::truncate(&p.value, vw),
                style,
            );
            y += 1;
        }
    }
    y - area.y
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropsEvent {
    Copy(usize),
    Activate(usize),
}

/// Interactive facts: one focus stop with a row cursor, `y` copies the
/// copyable row under the cursor, Enter activates it.
#[derive(Debug, Clone)]
pub struct PropsList {
    pub id: WidgetId,
    pub props: Vec<Prop>,
    pub cursor: usize,
    pub scroll: ScrollState,
    pub area: Rect,
}

impl PropsList {
    pub fn new(id: WidgetId, props: Vec<Prop>) -> Self {
        let n = props.len();
        Self {
            id,
            props,
            cursor: 0,
            scroll: ScrollState::new(n),
            area: Rect::ZERO,
        }
    }

    pub fn set_props(&mut self, props: Vec<Prop>) {
        self.props = props;
        self.cursor = self.cursor.min(self.props.len().saturating_sub(1));
        self.scroll.set_content(self.props.len());
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

    fn set_cursor(&mut self, i: usize) {
        self.cursor = i.min(self.props.len().saturating_sub(1));
        self.scroll.ensure_visible(self.cursor);
    }

    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<PropsEvent>) {
        if self.props.is_empty() {
            return (Outcome::Ignored, None);
        }
        match key.code {
            KeyCode::Up | KeyCode::Char('k') if key.plain() => {
                self.set_cursor(self.cursor.saturating_sub(1))
            }
            KeyCode::Down | KeyCode::Char('j') if key.plain() => self.set_cursor(self.cursor + 1),
            KeyCode::PageUp => {
                self.set_cursor(self.cursor.saturating_sub(self.scroll.viewport_len.max(1)))
            }
            KeyCode::PageDown => self.set_cursor(self.cursor + self.scroll.viewport_len.max(1)),
            KeyCode::Home | KeyCode::Char('g') if key.plain() => self.set_cursor(0),
            KeyCode::End | KeyCode::Char('G') => self.set_cursor(usize::MAX),
            KeyCode::Enter => return (Outcome::Changed, Some(PropsEvent::Activate(self.cursor))),
            KeyCode::Char('y') if key.plain() => {
                if self.props[self.cursor].copyable {
                    return (Outcome::Changed, Some(PropsEvent::Copy(self.cursor)));
                }
                return (Outcome::Consumed, None);
            }
            _ => return (Outcome::Ignored, None),
        }
        (Outcome::Changed, None)
    }

    pub fn on_click(&mut self, row: usize) -> (Outcome, Option<PropsEvent>) {
        if row >= self.props.len() {
            return (Outcome::Consumed, None);
        }
        self.set_cursor(row);
        (Outcome::Changed, Some(PropsEvent::Activate(row)))
    }

    pub fn on_wheel(&mut self, delta: i32) -> Outcome {
        self.scroll.scroll_by(delta as isize);
        Outcome::Changed
    }

    pub fn on_scrollbar(&mut self, pos: Position) -> Outcome {
        let track = Rect::new(
            self.area.right().saturating_sub(1),
            self.area.y,
            1,
            self.area.height,
        );
        self.scroll
            .scroll_to(scrollbar::offset_for_click(track, pos, &self.scroll));
        Outcome::Changed
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return;
        }
        self.area = area;
        let t = ctx.theme;
        let focused = ctx.interaction.focused(self.id);
        self.scroll.set_content(self.props.len());
        self.scroll.set_viewport(area.height as usize);
        ctx.control(self.id, area, false);
        ctx.scrollable(self.id, area);
        let has_sb = self.scroll.overflows();
        let row_w = area.width.saturating_sub(u16::from(has_sb));
        let label_w = self
            .props
            .iter()
            .map(|p| crate::ui::text::width(&p.label))
            .max()
            .unwrap_or(0) as u16
            + 2;
        for (k, i) in self.scroll.visible_range().enumerate() {
            let y = area.y + k as u16;
            let p = &self.props[i];
            let rid = self.row_id(i);
            let mut s = ctx.state(rid);
            s.focused = focused && i == self.cursor;
            let row = Rect::new(area.x, y, row_w, 1);
            let st = t.row(s, bg);
            fill(buf, row, st);
            buf.set_string(row.x, y, "▎", t.gutter(s, st.bg.unwrap_or(bg), false));
            buf.set_string(
                row.x + 2,
                y,
                &p.label,
                st.fg(t.text_muted).remove_modifier(Modifier::BOLD),
            );
            let vx = row.x + 2 + label_w;
            let hint_w: u16 = if p.copyable && s.focused { 8 } else { 0 };
            let vw = row.right().saturating_sub(vx + 1 + hint_w) as usize;
            buf.set_string(
                vx,
                y,
                crate::ui::text::truncate(&p.value, vw),
                st.fg(t.tone(p.tone)),
            );
            if hint_w > 0 {
                buf.set_string(
                    row.right().saturating_sub(7),
                    y,
                    "y copy",
                    st.fg(t.text_faint).remove_modifier(Modifier::BOLD),
                );
            }
            ctx.clickable(rid, row);
        }
        if has_sb {
            let sb = Rect::new(area.right() - 1, area.y, 1, area.height);
            scrollbar::render_vertical(sb, buf, ctx, self.id, &self.scroll, focused);
        }
    }
}
