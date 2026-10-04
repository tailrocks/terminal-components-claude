//! Compact activity Spinner component.
//!
//! Paints one deterministic spinner phase supplied by the caller or animation sample.

use ratatui::style::Style;
use unicode_width::UnicodeWidthStr;

use crate::termrock::identity::Id;
use crate::termrock::layout::{Constraints, Rect, Size};
use crate::termrock::runtime::{AnimationSample, MeasureCx, Ui};
use crate::termrock::theme::StylePatch;

/// Ten-phase Braille spinner frames matching visual oracle.
pub const SPINNER_FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// Return the deterministic spinner glyph for a given tick.
pub fn spinner_frame(tick: u64) -> &'static str {
    SPINNER_FRAMES[(tick % SPINNER_FRAMES.len() as u64) as usize]
}

/// Operational status of a spinner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpinnerStatus {
    #[default]
    Active,
    Paused,
    Stopped,
}

/// Compact activity spinner component.
#[derive(Clone)]
pub struct Spinner<'a> {
    pub id: Id,
    pub tick: u64,
    pub sample: Option<AnimationSample>,
    pub label: Option<&'a str>,
    pub status: SpinnerStatus,
    pub patch: StylePatch,
}

impl<'a> Spinner<'a> {
    pub fn new(id: Id, tick: u64) -> Self {
        Self {
            id,
            tick,
            sample: None,
            label: None,
            status: SpinnerStatus::Active,
            patch: StylePatch::empty(),
        }
    }

    pub fn with_sample(id: Id, sample: AnimationSample) -> Self {
        Self {
            id,
            tick: sample.index,
            sample: Some(sample),
            label: None,
            status: SpinnerStatus::Active,
            patch: StylePatch::empty(),
        }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn status(mut self, status: SpinnerStatus) -> Self {
        self.status = status;
        self
    }

    pub fn patch(mut self, patch: StylePatch) -> Self {
        self.patch = patch;
        self
    }

    pub fn measure(&self, _cx: &MeasureCx<'_>, constraints: Constraints) -> Size {
        let label_w = self
            .label
            .map(|l| UnicodeWidthStr::width(l) as u16)
            .unwrap_or(0);
        let min_w = if label_w > 0 { 2 + label_w } else { 1 };
        constraints.clamp(Size::new(min_w, 1))
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        if area.is_empty() {
            return area;
        }

        let theme = ui.theme;
        let effective_tick = self.sample.map(|s| s.index).unwrap_or(self.tick);
        let frame = match self.status {
            SpinnerStatus::Stopped => " ",
            SpinnerStatus::Paused => spinner_frame(0),
            SpinnerStatus::Active => spinner_frame(effective_tick),
        };

        let spinner_style = Style::default().fg(theme.tokens.accent);
        ui.set_string(area.x, area.y, frame, spinner_style);

        if let Some(lbl) = self.label
            && area.width > 2
        {
            let lbl_style = Style::default().fg(theme.tokens.text_secondary);
            ui.set_string(area.x + 2, area.y, lbl, lbl_style);
        }

        area
    }
}
