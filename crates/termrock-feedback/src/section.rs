use ratatui_core::layout::Rect;

use crate::measure::{Constraints, Size};
use crate::text::width;
use crate::theme::{FgStep, PaintStyle, Role, StylePatch};
use crate::ui::Ui;

#[derive(Clone, Copy, Debug)]
pub struct TextSection<'a> {
    title: &'a str,
    subtitle: Option<&'a str>,
    body: Option<&'a str>,
    body_row: u16,
    inset: u16,
    title_patch: StylePatch,
    text_patch: StylePatch,
    title_style: Option<PaintStyle>,
}

impl<'a> TextSection<'a> {
    #[must_use]
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            subtitle: None,
            body: None,
            body_row: 3,
            inset: 1,
            title_patch: StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .add(crate::theme::Modifier::BOLD),
            text_patch: StylePatch::new().set_fg(Role::Fg(FgStep::Muted)),
            title_style: None,
        }
    }

    #[must_use]
    pub const fn subtitle(mut self, subtitle: &'a str) -> Self {
        self.subtitle = Some(subtitle);
        self
    }

    #[must_use]
    pub const fn body(mut self, body: &'a str) -> Self {
        self.body = Some(body);
        self
    }

    #[must_use]
    pub const fn body_row(mut self, row: u16) -> Self {
        self.body_row = row;
        self
    }

    #[must_use]
    pub const fn inset(mut self, cells: u16) -> Self {
        self.inset = cells;
        self
    }

    #[must_use]
    pub const fn title_patch(mut self, patch: StylePatch) -> Self {
        self.title_patch = patch;
        self
    }

    #[must_use]
    pub const fn text_patch(mut self, patch: StylePatch) -> Self {
        self.text_patch = patch;
        self
    }

    #[must_use]
    pub const fn title_style(mut self, style: PaintStyle) -> Self {
        self.title_style = Some(style);
        self
    }

    pub fn measure(&self, _ui: &Ui<'_>, constraints: Constraints) -> Size {
        let text_width = width(self.title)
            .max(self.subtitle.map_or(0, width))
            .max(self.body.map_or(0, width));
        let width = text_width.saturating_add(self.inset.saturating_mul(2));
        let height = self
            .subtitle
            .map_or(1, |_| 2)
            .max(self.body.map_or(0, |_| self.body_row.saturating_add(1)));
        Size::exact(width, height).fit(constraints)
    }

    pub fn render(self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let line = |row| {
            Rect::new(
                area.x.saturating_add(self.inset),
                area.y.saturating_add(row),
                area.width.saturating_sub(self.inset.saturating_mul(2)),
                1,
            )
        };
        let title = self
            .title_style
            .unwrap_or_else(|| ui.paint_patch(&self.title_patch));
        ui.paint_str(line(0), self.title, title);
        let text = ui.paint_patch(&self.text_patch);
        if area.height > 1
            && let Some(subtitle) = self.subtitle
        {
            ui.paint_str(line(1), subtitle, text);
        }
        if area.height > self.body_row
            && let Some(body) = self.body
        {
            ui.paint_str(line(self.body_row), body, text);
        }
    }
}
