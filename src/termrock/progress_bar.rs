//! Determinate and indeterminate Progress Bar component.
//!
//! Paints a deterministic progress sample with label, track fill, percentage,
//! and status suffix.

use ratatui::style::Style;
use unicode_width::UnicodeWidthStr;

use crate::termrock::identity::Id;
use crate::termrock::layout::{Constraints, Rect, Size};
use crate::termrock::runtime::{AnimationSample, MeasureCx, Ui};
use crate::termrock::theme::StylePatch;

/// Semantic progress operation status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProgressStatus {
    #[default]
    Active,
    Done,
    Error,
    Paused,
}

/// Progress value: finite fraction or explicit indeterminate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProgressValue {
    Determinate(f32),
    Indeterminate,
}

impl From<f32> for ProgressValue {
    fn from(val: f32) -> Self {
        if val.is_nan() || val.is_infinite() {
            Self::Determinate(0.0)
        } else {
            Self::Determinate(val.clamp(0.0, 1.0))
        }
    }
}

/// Determinate and indeterminate progress bar component.
#[derive(Clone)]
pub struct ProgressBar<'a> {
    pub id: Id,
    pub fraction: f32,
    pub value: ProgressValue,
    pub status: ProgressStatus,
    pub label: Option<&'a str>,
    pub show_percentage: bool,
    pub indeterminate: bool,
    pub animation: Option<AnimationSample>,
    pub patch: StylePatch,
}

impl<'a> ProgressBar<'a> {
    pub fn new(id: Id, fraction: f32) -> Self {
        let safe_fraction = if fraction.is_nan() || fraction.is_infinite() {
            0.0
        } else {
            fraction.clamp(0.0, 1.0)
        };
        Self {
            id,
            fraction: safe_fraction,
            value: ProgressValue::Determinate(safe_fraction),
            status: ProgressStatus::Active,
            label: None,
            show_percentage: true,
            indeterminate: false,
            animation: None,
            patch: StylePatch::empty(),
        }
    }

    pub fn with_value(id: Id, value: ProgressValue) -> Self {
        let (fraction, indet) = match value {
            ProgressValue::Determinate(f) => {
                let safe = if f.is_nan() || f.is_infinite() {
                    0.0
                } else {
                    f.clamp(0.0, 1.0)
                };
                (safe, false)
            }
            ProgressValue::Indeterminate => (0.0, true),
        };
        Self {
            id,
            fraction,
            value,
            status: ProgressStatus::Active,
            label: None,
            show_percentage: true,
            indeterminate: indet,
            animation: None,
            patch: StylePatch::empty(),
        }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn status(mut self, status: ProgressStatus) -> Self {
        self.status = status;
        self
    }

    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }

    pub fn indeterminate(mut self, indeterminate: bool) -> Self {
        self.indeterminate = indeterminate;
        if indeterminate {
            self.value = ProgressValue::Indeterminate;
        }
        self
    }

    pub fn animation(mut self, sample: AnimationSample) -> Self {
        self.animation = Some(sample);
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
        let pct_w = if self.show_percentage && !self.indeterminate {
            5
        } else {
            0
        };
        let min_w = if label_w > 0 {
            label_w + 2 + 6 + pct_w + 2
        } else {
            6 + pct_w + 2
        };
        constraints.clamp(Size::new(min_w, 1))
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        if area.is_empty() {
            return area;
        }

        let theme = ui.theme;
        let label_w = self
            .label
            .map(|l| UnicodeWidthStr::width(l) as u16)
            .unwrap_or(0);
        let has_label = label_w > 0 && area.width > label_w + 8;
        let mut x = area.x;

        if has_label && let Some(lbl) = self.label {
            let lbl_style = Style::default().fg(theme.tokens.text_primary);
            ui.set_string(x, area.y, lbl, lbl_style);
            x += label_w + 2;
        }

        let is_indet = self.indeterminate || matches!(self.value, ProgressValue::Indeterminate);
        if is_indet {
            let track_w = area.right().saturating_sub(x).max(1) as i64;
            let seg = (track_w / 5).clamp(2, 8);
            let period = track_w + seg;
            let tick = self.animation.map(|a| a.index).unwrap_or(0) as i64;
            let pos = (tick % period) - seg;

            for i in 0..track_w {
                let in_seg = i >= pos && i < pos + seg;
                let (sym, st) = if in_seg {
                    ("━", Style::default().fg(theme.tokens.accent))
                } else {
                    ("─", Style::default().fg(theme.tokens.border_subtle))
                };
                ui.set_string(x + i as u16, area.y, sym, st);
            }
            return area;
        }

        // Determinate bar
        let ratio = match self.value {
            ProgressValue::Determinate(f) => f.clamp(0.0, 1.0),
            _ => self.fraction.clamp(0.0, 1.0),
        };
        let pct = format!("{:>4}", format!("{}%", (ratio * 100.0).round() as u32));
        let pct_w = if self.show_percentage { 5u16 } else { 0u16 };
        let suffix = match self.status {
            ProgressStatus::Done => " ✓",
            ProgressStatus::Error => " !",
            ProgressStatus::Paused => " ‖",
            ProgressStatus::Active => "  ",
        };

        let track_w = area.right().saturating_sub(x).saturating_sub(pct_w + 2);
        if track_w < 6 {
            // Too narrow for bar: percentage only
            if self.show_percentage {
                ui.set_string(
                    x,
                    area.y,
                    pct.trim_start(),
                    Style::default().fg(theme.tokens.text_secondary),
                );
            }
            return area;
        }

        let filled = ((track_w as f32) * ratio).round() as u16;
        let fill_color = match self.status {
            ProgressStatus::Active => theme.tokens.text_secondary,
            ProgressStatus::Done => theme.tokens.success,
            ProgressStatus::Error => theme.tokens.danger,
            ProgressStatus::Paused => theme.tokens.text_muted,
        };

        for i in 0..track_w {
            let (sym, st) = if i < filled {
                ("━", Style::default().fg(fill_color))
            } else {
                ("─", Style::default().fg(theme.tokens.border_subtle))
            };
            ui.set_string(x + i, area.y, sym, st);
        }

        x += track_w;
        if self.show_percentage {
            ui.set_string(
                x,
                area.y,
                &format!(" {pct}"),
                Style::default().fg(theme.tokens.text_secondary),
            );
        }
        ui.set_string(x + pct_w, area.y, suffix, Style::default().fg(fill_color));

        area
    }
}
