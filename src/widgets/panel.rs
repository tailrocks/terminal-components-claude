//! Panels: the surface container. Two flavours:
//! - **card**: filled `surface` rectangle, no border, title row. The default.
//! - **framed**: rounded subtle border, used when a region must read as a
//!   distinct pane (split views, dialogs).
//!
//! Focus at container level is shown by the border/title only; the accent
//! gutter bar belongs to the focused control inside.

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::Color;
use ratatui::widgets::{Block, BorderType, Borders, Widget};

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::core::scroll::ScrollState;
use crate::theme::Theme;
use crate::ui::ctx::{RenderCtx, fill};
use crate::widgets::scrollbar;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelKind {
    Card,
    Framed,
}

pub struct Panel<'a> {
    pub title: Option<&'a str>,
    pub kind: PanelKind,
    pub focused: bool,
    /// Right-aligned text in the title row (position label, badge).
    pub meta: Option<&'a str>,
    pub badge: Option<(&'a str, crate::theme::BadgeKind)>,
    pub bg_override: Option<Color>,
}

impl<'a> Panel<'a> {
    pub fn card(title: Option<&'a str>) -> Self {
        Self {
            title,
            kind: PanelKind::Card,
            focused: false,
            meta: None,
            badge: None,
            bg_override: None,
        }
    }
    pub fn framed(title: Option<&'a str>) -> Self {
        Self {
            title,
            kind: PanelKind::Framed,
            focused: false,
            meta: None,
            badge: None,
            bg_override: None,
        }
    }
    pub fn focused(mut self, f: bool) -> Self {
        self.focused = f;
        self
    }
    pub fn meta(mut self, m: &'a str) -> Self {
        self.meta = Some(m);
        self
    }

    /// Background colour the content will sit on.
    pub fn bg(&self, t: &Theme) -> Color {
        if let Some(bg) = self.bg_override {
            return bg;
        }
        match self.kind {
            PanelKind::Card => t.surface,
            PanelKind::Framed => t.canvas,
        }
    }

    /// Draw the panel chrome and return the inner content area.
    pub fn render(&self, area: Rect, buf: &mut Buffer, t: &Theme) -> Rect {
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return area;
        }
        let bg = self.bg(t);
        match self.kind {
            PanelKind::Card => {
                fill(buf, area, Style::new().bg(bg));
                let inner = area.inner(ratatui::layout::Margin::new(2, 1));
                if self.focused && self.title.is_some() {
                    // container focus: the same bar as a control, in the padding column
                    buf.set_string(area.x + 1, area.y, "▎", Style::new().fg(t.focus).bg(bg));
                }
                self.title_row(area.x + 2, area.y, area.width.saturating_sub(4), buf, t, bg);
                if self.title.is_some() {
                    Rect::new(
                        inner.x,
                        inner.y + 1,
                        inner.width,
                        inner.height.saturating_sub(1),
                    )
                } else {
                    inner
                }
            }
            PanelKind::Framed => {
                fill(buf, area, Style::new().bg(bg));
                let block = Block::new()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(t.border(self.focused).bg(bg));
                block.render(area, buf);
                if area.width > 4 {
                    self.title_row(area.x + 2, area.y, area.width.saturating_sub(4), buf, t, bg);
                }
                let inner = area.inner(ratatui::layout::Margin::new(1, 1));
                Rect::new(
                    inner.x + 2,
                    inner.y,
                    inner.width.saturating_sub(3),
                    inner.height,
                )
            }
        }
    }

    fn title_row(&self, x: u16, y: u16, w: u16, buf: &mut Buffer, t: &Theme, bg: Color) {
        if w == 0 {
            return;
        }
        let mut cx = x;
        if let Some(title) = self.title {
            let style = if self.focused {
                t.title().bg(bg)
            } else {
                t.secondary().bg(bg)
            };
            let title = crate::ui::text::truncate(title, w as usize);
            let title = if self.kind == PanelKind::Framed {
                format!(" {title} ")
            } else {
                title
            };
            buf.set_string(cx, y, &title, style);
            cx += crate::ui::text::width(&title) as u16;
        }
        let mut right = x + w;
        if let Some(meta) = self.meta {
            let mw = crate::ui::text::width(meta) as u16;
            if right > cx + mw + 1 {
                let text = if self.kind == PanelKind::Framed {
                    format!(" {meta} ")
                } else {
                    meta.to_owned()
                };
                let tw = crate::ui::text::width(&text) as u16;
                right = right.saturating_sub(tw);
                buf.set_string(right, y, &text, t.faint().bg(bg));
            }
        }
        if let Some((badge, kind)) = self.badge {
            let text = format!(" {badge} ");
            let bw = crate::ui::text::width(&text) as u16;
            if right > cx + bw + 1 {
                right = right.saturating_sub(bw + 1);
                buf.set_string(right, y, &text, t.badge(kind));
            }
        }
    }
}

use ratatui::style::Style;

/// A scrollable read-only text panel (log output, prose). It is a focus stop
/// itself because it has no focusable children.
#[derive(Debug, Clone)]
pub struct ScrollPanel {
    pub id: WidgetId,
    pub lines: Vec<String>,
    pub scroll: ScrollState,
    pub follow: bool,
    pub wrap: bool,
    pub area: Rect,
    wrapped_cache: (u16, Vec<String>),
}

impl ScrollPanel {
    pub fn new(id: WidgetId, lines: Vec<String>) -> Self {
        Self {
            id,
            lines,
            scroll: ScrollState::default(),
            follow: false,
            wrap: false,
            area: Rect::ZERO,
            wrapped_cache: (0, vec![]),
        }
    }
    pub fn wrap(mut self, w: bool) -> Self {
        self.wrap = w;
        self
    }

    pub fn push(&mut self, line: String) {
        self.lines.push(line);
        self.wrapped_cache.0 = 0;
    }

    pub fn on_key(&mut self, key: &Key) -> Outcome {
        let before = self.scroll.offset;
        match key.code {
            KeyCode::Up | KeyCode::Char('k') if key.plain() => self.scroll.scroll_by(-1),
            KeyCode::Down | KeyCode::Char('j') if key.plain() => self.scroll.scroll_by(1),
            KeyCode::PageUp => self.scroll.page_up(),
            KeyCode::PageDown => self.scroll.page_down(),
            KeyCode::Home | KeyCode::Char('g') if key.plain() => self.scroll.jump_start(),
            KeyCode::End | KeyCode::Char('G') => {
                self.scroll.jump_end();
                self.follow = true;
                return Outcome::Changed;
            }
            KeyCode::Char('f') if key.plain() => {
                self.follow = !self.follow;
                if self.follow {
                    self.scroll.jump_end();
                }
                return Outcome::Changed;
            }
            _ => return Outcome::Ignored,
        }
        if self.scroll.offset != before {
            self.follow = false;
        }
        Outcome::Changed
    }

    pub fn on_wheel(&mut self, delta: i32) -> Outcome {
        self.scroll.scroll_by(delta as isize);
        self.follow = false;
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
        self.follow = false;
        Outcome::Changed
    }

    /// Render into `area` (already the inner area of a panel).
    pub fn render(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: Color,
        style_line: fn(&Theme, &str) -> Style,
    ) {
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return;
        }
        self.area = area;
        let t = ctx.theme;
        let focused = ctx.interaction.focused(self.id);
        let text_w = area.width.saturating_sub(2);
        let lines: &Vec<String> = if self.wrap {
            if self.wrapped_cache.0 != text_w {
                let mut out = Vec::new();
                for l in &self.lines {
                    out.extend(crate::ui::text::wrap(l, text_w as usize));
                }
                self.wrapped_cache = (text_w, out);
            }
            &self.wrapped_cache.1
        } else {
            &self.lines
        };
        self.scroll.set_content(lines.len());
        self.scroll.set_viewport(area.height as usize);
        ctx.control(self.id, area, false);
        ctx.scrollable(self.id, area);
        if self.follow {
            self.scroll.jump_end();
        }
        for (i, li) in self.scroll.visible_range().enumerate() {
            let y = area.y + i as u16;
            let line = &lines[li];
            let st = style_line(t, line).bg(bg);
            let text = crate::ui::text::fit(line, text_w as usize);
            buf.set_string(area.x, y, &text, st);
        }
        if self.scroll.overflows() {
            let sb = Rect::new(area.right() - 1, area.y, 1, area.height);
            scrollbar::render_vertical(sb, buf, ctx, self.id, &self.scroll, focused);
        }
    }
}
