//! Contextual Key Hint pill component.
//!
//! Renders a shortcut chord and its descriptive label with distinct key and action styling.

use ratatui::style::{Modifier, Style};
use unicode_width::UnicodeWidthStr;

use crate::termrock::layout::{Constraints, Rect, Size};
use crate::termrock::runtime::{MeasureCx, Ui};
use crate::termrock::theme::StylePatch;

/// Single key hint pill displaying a chord and description.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyHint<'a> {
    pub chord: &'a str,
    pub description: &'a str,
    pub patch: StylePatch,
}

impl<'a> KeyHint<'a> {
    pub fn new(chord: &'a str, description: &'a str) -> Self {
        Self {
            chord,
            description,
            patch: StylePatch::empty(),
        }
    }

    pub fn patch(mut self, patch: StylePatch) -> Self {
        self.patch = patch;
        self
    }

    pub fn measure(&self, _cx: &MeasureCx<'_>, constraints: Constraints) -> Size {
        let cw = UnicodeWidthStr::width(self.chord) as u16;
        let dw = UnicodeWidthStr::width(self.description) as u16;
        let w = cw + 1 + dw;
        constraints.clamp(Size::new(w, 1))
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        if area.is_empty() {
            return area;
        }

        let theme = ui.theme;
        let cw = UnicodeWidthStr::width(self.chord) as u16;
        let key_style = Style::default()
            .fg(theme.tokens.text_primary)
            .add_modifier(Modifier::BOLD);
        let desc_style = Style::default().fg(theme.tokens.text_muted);

        ui.set_string(area.x, area.y, self.chord, key_style);
        if area.width > cw + 1 {
            ui.set_string(area.x + cw + 1, area.y, self.description, desc_style);
        }

        area
    }
}
