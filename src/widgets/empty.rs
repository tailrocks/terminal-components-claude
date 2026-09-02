//! Empty states: a quiet title, an optional hint, never a big glyph.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EmptyKind {
    #[default]
    Empty,
    /// A failed state: bold `!` in error tone before the title.
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EmptyState {
    pub title: String,
    pub hint: Option<String>,
    pub kind: EmptyKind,
}

impl EmptyState {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_owned(),
            hint: None,
            kind: EmptyKind::Empty,
        }
    }
    pub fn error(title: &str) -> Self {
        Self {
            title: title.to_owned(),
            hint: None,
            kind: EmptyKind::Error,
        }
    }
    pub fn hint(mut self, h: &str) -> Self {
        self.hint = Some(h.to_owned());
        self
    }
}

/// Render centred in `area`.
pub fn render(area: Rect, buf: &mut Buffer, t: &Theme, e: &EmptyState, bg: Color) {
    let area = area.intersection(*buf.area());
    if area.is_empty() {
        return;
    }
    let hint_lines: Vec<String> = e
        .hint
        .as_ref()
        .map(|h| crate::ui::text::wrap(h, area.width.saturating_sub(4).max(8) as usize))
        .unwrap_or_default();
    let total = 1 + if hint_lines.is_empty() {
        0
    } else {
        hint_lines.len() + 1
    } as u16;
    let y0 = area.y + area.height.saturating_sub(total) / 2;
    let put = |buf: &mut Buffer, y: u16, s: &str, style: ratatui::style::Style| {
        if y >= area.bottom() {
            return;
        }
        let s = crate::ui::text::truncate(s, area.width as usize);
        let x = area.x + area.width.saturating_sub(crate::ui::text::width(&s) as u16) / 2;
        buf.set_string(x, y, &s, style.bg(bg));
    };
    match e.kind {
        EmptyKind::Empty => put(buf, y0, &e.title, t.muted()),
        EmptyKind::Error => {
            let title = format!("! {}", e.title);
            put(buf, y0, &title, t.error_fg());
            // the `!` is bold on its own
            let s = crate::ui::text::truncate(&title, area.width as usize);
            let x = area.x + area.width.saturating_sub(crate::ui::text::width(&s) as u16) / 2;
            if y0 < area.bottom() {
                buf.set_string(
                    x,
                    y0,
                    "!",
                    t.error_fg()
                        .bg(bg)
                        .add_modifier(ratatui::style::Modifier::BOLD),
                );
            }
        }
    }
    for (i, l) in hint_lines.iter().enumerate() {
        put(buf, y0 + 2 + i as u16, l, t.faint());
    }
}
