//! Small shell modals: a scrollable text modal used for the key reference,
//! About, and "Why is this here?".

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::id::WidgetId;
use junie_tui::core::scroll::ScrollState;
use junie_tui::theme::Tone;
use junie_tui::ui::ctx::{RenderCtx, fill};
use junie_tui::ui::popup::{Placement, place};
use junie_tui::ui::text::{truncate, width, wrap};
use junie_tui::widgets::scrollbar;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

/// One line of a text modal: an optional bold key column and a value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextLine {
    pub key: String,
    pub value: String,
    pub tone: Tone,
    pub heading: bool,
}

impl TextLine {
    pub fn kv(key: &str, value: &str) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            tone: Tone::Secondary,
            heading: false,
        }
    }
    pub fn text(value: &str, tone: Tone) -> Self {
        Self {
            key: String::new(),
            value: value.into(),
            tone,
            heading: false,
        }
    }
    pub fn heading(value: &str) -> Self {
        Self {
            key: String::new(),
            value: value.into(),
            tone: Tone::Faint,
            heading: true,
        }
    }
    pub fn blank() -> Self {
        Self::text("", Tone::Normal)
    }
}

pub struct TextModal {
    pub id: WidgetId,
    pub title: String,
    pub subtitle: String,
    pub lines: Vec<TextLine>,
    pub scroll: ScrollState,
    pub width: u16,
    pub closed: bool,
    pub area: Rect,
    wrapped: Vec<(String, String, Tone, bool)>,
    wrapped_for: u16,
}

impl TextModal {
    pub fn new(id: WidgetId, title: &str, lines: Vec<TextLine>) -> Self {
        Self {
            id,
            title: title.into(),
            subtitle: String::new(),
            lines,
            scroll: ScrollState::default(),
            width: 72,
            closed: false,
            area: Rect::ZERO,
            wrapped: vec![],
            wrapped_for: 0,
        }
    }
    pub fn subtitle(mut self, s: &str) -> Self {
        self.subtitle = s.into();
        self
    }
    pub fn width(mut self, w: u16) -> Self {
        self.width = w;
        self
    }

    fn key_col(&self) -> usize {
        self.lines
            .iter()
            .filter(|l| !l.heading)
            .map(|l| width(&l.key))
            .max()
            .unwrap_or(0)
    }

    fn layout(&mut self, inner_w: u16) {
        if self.wrapped_for == inner_w {
            return;
        }
        self.wrapped_for = inner_w;
        let kc = self.key_col();
        let vw = (inner_w as usize)
            .saturating_sub(if kc > 0 { kc + 2 } else { 0 })
            .max(8);
        let mut out = vec![];
        for l in &self.lines {
            if l.heading {
                out.push((String::new(), l.value.clone(), l.tone, true));
                continue;
            }
            let parts = if l.value.is_empty() {
                vec![String::new()]
            } else {
                wrap(&l.value, vw)
            };
            for (i, p) in parts.into_iter().enumerate() {
                out.push((
                    if i == 0 { l.key.clone() } else { String::new() },
                    p,
                    l.tone,
                    false,
                ));
            }
        }
        self.wrapped = out;
        self.scroll.set_content(self.wrapped.len());
    }

    pub fn on_key(&mut self, key: &Key) -> Outcome {
        match key.code {
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
                self.closed = true;
                Outcome::Changed
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.scroll.scroll_by(-1);
                Outcome::Changed
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.scroll.scroll_by(1);
                Outcome::Changed
            }
            KeyCode::PageUp => {
                self.scroll.page_up();
                Outcome::Changed
            }
            KeyCode::PageDown => {
                self.scroll.page_down();
                Outcome::Changed
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.scroll.jump_start();
                Outcome::Changed
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.scroll.jump_end();
                Outcome::Changed
            }
            _ => Outcome::Consumed,
        }
    }

    pub fn on_wheel(&mut self, delta: i32) -> Outcome {
        self.scroll.scroll_by(delta as isize);
        Outcome::Changed
    }

    pub fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
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
        let w = self.width.min(screen.width.saturating_sub(4)).max(30);
        let inner_w = w.saturating_sub(6);
        self.layout(inner_w);
        let content_h = self.wrapped.len() as u16;
        let h = (content_h + 6).min(screen.height.saturating_sub(3)).max(8);
        let area = place(screen, Rect::ZERO, w, h, Placement::Center);
        self.area = area;
        let bg = t.surface_elevated;
        fill(buf, area, Style::new().bg(bg));
        let block = ratatui::widgets::Block::new()
            .borders(ratatui::widgets::Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(t.border(true).bg(bg));
        ratatui::widgets::Widget::render(block, area, buf);
        ctx.hits.register(self.id, area);
        ctx.ring.register(self.id);
        let inner = area.inner(ratatui::layout::Margin::new(3, 2));
        if inner.is_empty() {
            return;
        }
        buf.set_string(
            inner.x,
            inner.y,
            truncate(&self.title, inner.width as usize),
            t.title().bg(bg),
        );
        if !self.subtitle.is_empty() {
            let sw = width(&self.subtitle) as u16;
            let room = inner.width.saturating_sub(width(&self.title) as u16 + 3);
            if sw <= room {
                buf.set_string(
                    inner.right().saturating_sub(sw),
                    inner.y,
                    &self.subtitle,
                    t.muted().bg(bg),
                );
            }
        }
        let body = Rect::new(
            inner.x,
            inner.y + 2,
            inner.width,
            inner.height.saturating_sub(2),
        );
        self.scroll.set_viewport(body.height as usize);
        ctx.scrollable(self.id, body);
        let kc = self.key_col() as u16;
        let has_sb = self.scroll.overflows();
        for (k, i) in self.scroll.visible_range().enumerate() {
            let y = body.y + k as u16;
            let (key, value, tone, heading) = &self.wrapped[i];
            if *heading {
                buf.set_string(
                    body.x,
                    y,
                    truncate(value, body.width as usize),
                    t.faint().bg(bg),
                );
                continue;
            }
            if kc > 0 {
                buf.set_string(body.x, y, key, t.key_hint_key().bg(bg));
                let vx = body.x + kc + 2;
                buf.set_string(
                    vx,
                    y,
                    truncate(
                        value,
                        body.width.saturating_sub(kc + 2 + u16::from(has_sb)) as usize,
                    ),
                    Style::new().fg(t.tone(*tone)).bg(bg),
                );
            } else {
                buf.set_string(
                    body.x,
                    y,
                    truncate(value, body.width.saturating_sub(u16::from(has_sb)) as usize),
                    Style::new().fg(t.tone(*tone)).bg(bg),
                );
            }
        }
        if has_sb {
            junie_tui::ui::fade::scroll_edges(
                buf,
                ctx,
                Rect::new(
                    body.x,
                    body.y,
                    (body.right() - 1).saturating_sub(body.x),
                    body.height,
                ),
                &self.scroll,
            );
            scrollbar::render_vertical(
                Rect::new(body.right() - 1, body.y, 1, body.height),
                buf,
                ctx,
                self.id,
                &self.scroll,
                true,
            );
        }
    }
}
