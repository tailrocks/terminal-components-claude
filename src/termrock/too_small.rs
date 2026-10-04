//! Generic TooSmall terminal dimensions notice overlay.
//!
//! Renders a centered friendly warning message when the viewport is smaller
//! than required minimum dimensions.

use ratatui::style::{Modifier, Style};
use unicode_width::UnicodeWidthStr;

use crate::termrock::identity::Id;
use crate::termrock::layout::{Constraints, Rect, Size};
use crate::termrock::runtime::{MeasureCx, Ui};
use crate::termrock::theme::StylePatch;

/// Terminal too small warning notice overlay.
#[derive(Clone)]
pub struct TooSmall<'a> {
    pub id: Id,
    pub minimum: Size,
    pub current: Option<Size>,
    pub title: Option<&'a str>,
    pub message: Option<&'a str>,
    pub quit_hint: Option<&'a str>,
    pub patch: StylePatch,
}

impl<'a> TooSmall<'a> {
    pub fn new(id: Id, minimum: Size) -> Self {
        Self {
            id,
            minimum,
            current: None,
            title: None,
            message: None,
            quit_hint: Some("q Quit"),
            patch: StylePatch::empty(),
        }
    }

    pub fn with_message(id: Id, minimum: Size, message: &'a str) -> Self {
        Self {
            id,
            minimum,
            current: None,
            title: None,
            message: Some(message),
            quit_hint: Some("q Quit"),
            patch: StylePatch::empty(),
        }
    }

    pub fn current(mut self, current: Size) -> Self {
        self.current = Some(current);
        self
    }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    pub fn message(mut self, message: &'a str) -> Self {
        self.message = Some(message);
        self
    }

    pub fn quit_hint(mut self, hint: &'a str) -> Self {
        self.quit_hint = Some(hint);
        self
    }

    pub fn patch(mut self, patch: StylePatch) -> Self {
        self.patch = patch;
        self
    }

    pub fn measure(&self, _cx: &MeasureCx<'_>, constraints: Constraints) -> Size {
        constraints.clamp(self.minimum)
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        if area.is_empty() {
            return area;
        }

        let theme = ui.theme;
        let bg_style = Style::default().bg(theme.tokens.canvas);
        ui.fill_rect(area, bg_style);

        let cur_w = self.current.map(|s| s.width).unwrap_or(area.width);
        let cur_h = self.current.map(|s| s.height).unwrap_or(area.height);

        let title_text = self.title.unwrap_or("Terminal too small");
        let default_msg = format!(
            "Need {}×{}, have {}×{}",
            self.minimum.width, self.minimum.height, cur_w, cur_h
        );
        let msg_text = self.message.unwrap_or(&default_msg);

        let mut lines = vec![
            (
                title_text,
                Style::default()
                    .fg(theme.tokens.text_primary)
                    .add_modifier(Modifier::BOLD),
            ),
            (msg_text, Style::default().fg(theme.tokens.text_muted)),
        ];

        if let Some(quit) = self.quit_hint {
            lines.push((quit, Style::default().fg(theme.tokens.text_faint)));
        }

        let total_lines = lines.len() as u16;
        let y0 = area.y + area.height.saturating_sub(total_lines) / 2;

        for (i, (text, style)) in lines.iter().enumerate() {
            let w = UnicodeWidthStr::width(*text) as u16;
            let x = area.x + area.width.saturating_sub(w) / 2;
            let y = y0 + i as u16;
            if y < area.bottom() {
                ui.set_string(x, y, text, *style);
            }
        }

        area
    }
}
