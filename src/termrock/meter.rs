//! Capacity and consumption Meter component.
//!
//! Presents caller-supplied capacity information in a compact line or filled-block visual.

use ratatui::style::{Color, Modifier, Style};
use unicode_width::UnicodeWidthStr;

use crate::termrock::identity::Id;
use crate::termrock::layout::{Constraints, Rect, Size};
use crate::termrock::runtime::{AnimationSample, MeasureCx, Ui};
use crate::termrock::spinner::spinner_frame;
use crate::termrock::theme::StylePatch;

pub const METER_LOW_MAX: u8 = 59;
pub const METER_MEDIUM_MAX: u8 = 84;

/// Discrete consumption level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeterLevel {
    Low,
    Medium,
    High,
}

impl MeterLevel {
    pub fn of(pct: u8) -> Self {
        if pct <= METER_LOW_MAX {
            MeterLevel::Low
        } else if pct <= METER_MEDIUM_MAX {
            MeterLevel::Medium
        } else {
            MeterLevel::High
        }
    }
}

/// Visual presentation mode for a meter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MeterVisual {
    #[default]
    Line,
    Block,
}

/// Semantic tone of a meter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MeterTone {
    #[default]
    Normal,
    Level(MeterLevel),
    Warning,
    Exhausted,
    Critical,
    Stale,
    Refreshing,
    Error,
    Unknown,
}

impl MeterTone {
    pub fn level(self, pct: Option<u8>) -> Option<MeterLevel> {
        match self {
            MeterTone::Normal => pct.map(MeterLevel::of),
            MeterTone::Level(l) => Some(l),
            MeterTone::Warning => Some(MeterLevel::Medium),
            MeterTone::Exhausted | MeterTone::Critical => Some(MeterLevel::High),
            _ => None,
        }
    }
}

/// Capacity and consumption meter component.
#[derive(Clone)]
pub struct Meter<'a> {
    pub id: Id,
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub low: Option<f32>,
    pub high: Option<f32>,
    pub optimum: Option<f32>,
    pub label: Option<&'a str>,
    pub readout: Option<&'a str>,
    pub tone: MeterTone,
    pub visual: MeterVisual,
    pub animation: Option<AnimationSample>,
    pub patch: StylePatch,
}

impl<'a> Meter<'a> {
    pub fn new(id: Id, value: f32, min: f32, max: f32) -> Self {
        Self {
            id,
            value,
            min,
            max,
            low: None,
            high: None,
            optimum: None,
            label: None,
            readout: None,
            tone: MeterTone::Normal,
            visual: MeterVisual::Line,
            animation: None,
            patch: StylePatch::empty(),
        }
    }

    pub fn from_percent(id: Id, percent: Option<u8>) -> Self {
        let (val, tone) = match percent {
            Some(p) => (p as f32, MeterTone::Normal),
            None => (0.0, MeterTone::Unknown),
        };
        Self {
            id,
            value: val,
            min: 0.0,
            max: 100.0,
            low: Some(59.0),
            high: Some(84.0),
            optimum: None,
            label: None,
            readout: None,
            tone,
            visual: MeterVisual::Line,
            animation: None,
            patch: StylePatch::empty(),
        }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn readout(mut self, readout: &'a str) -> Self {
        self.readout = Some(readout);
        self
    }

    pub fn low(mut self, low: f32) -> Self {
        self.low = Some(low);
        self
    }

    pub fn high(mut self, high: f32) -> Self {
        self.high = Some(high);
        self
    }

    pub fn optimum(mut self, optimum: f32) -> Self {
        self.optimum = Some(optimum);
        self
    }

    pub fn tone(mut self, tone: MeterTone) -> Self {
        self.tone = tone;
        self
    }

    pub fn visual(mut self, visual: MeterVisual) -> Self {
        self.visual = visual;
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

    pub fn ratio(&self) -> f32 {
        if self.max <= self.min {
            0.0
        } else {
            ((self.value - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
        }
    }

    pub fn percent(&self) -> u8 {
        (self.ratio() * 100.0).round() as u8
    }

    fn palette(&self, theme: &crate::termrock::theme::Theme) -> (Color, Color, &'static str, bool) {
        let pct = self.percent();
        let level_color = |l: MeterLevel| match l {
            MeterLevel::Low => theme.tokens.text_secondary,
            MeterLevel::Medium => theme.tokens.warning,
            MeterLevel::High => theme.tokens.danger,
        };

        match self.tone {
            MeterTone::Normal | MeterTone::Level(_) => {
                let l = if let Some(high) = self.high {
                    if self.value > high {
                        MeterLevel::High
                    } else if let Some(low) = self.low {
                        if self.value > low {
                            MeterLevel::Medium
                        } else {
                            MeterLevel::Low
                        }
                    } else {
                        MeterLevel::Low
                    }
                } else {
                    self.tone.level(Some(pct)).unwrap_or(MeterLevel::Low)
                };

                let c = level_color(l);
                let text = if l == MeterLevel::Low {
                    theme.tokens.text_primary
                } else {
                    c
                };
                (c, text, "  ", true)
            }
            MeterTone::Warning => (theme.tokens.warning, theme.tokens.warning, " ▲", true),
            MeterTone::Exhausted | MeterTone::Critical => {
                (theme.tokens.danger, theme.tokens.danger, " !", true)
            }
            MeterTone::Stale => (theme.tokens.text_faint, theme.tokens.text_muted, "  ", true),
            MeterTone::Refreshing => (theme.tokens.text_muted, theme.tokens.text_muted, "  ", true),
            MeterTone::Error => (theme.tokens.danger, theme.tokens.danger, " !", false),
            MeterTone::Unknown => (
                theme.tokens.text_faint,
                theme.tokens.text_faint,
                "  ",
                false,
            ),
        }
    }

    pub fn measure(&self, _cx: &MeasureCx<'_>, constraints: Constraints) -> Size {
        let label_w = self
            .label
            .map(|l| UnicodeWidthStr::width(l) as u16)
            .unwrap_or(0);
        let min_w = if label_w > 0 {
            label_w + 2 + 10 + 6
        } else {
            10 + 6
        };
        constraints.clamp(Size::new(min_w, 1))
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        if area.is_empty() {
            return area;
        }

        let theme = ui.theme;
        let (fill_color, text_color, suffix, has_run) = self.palette(theme);
        let pct = self.percent();

        let value_str = if let Some(r) = self.readout {
            r.to_string()
        } else {
            match self.tone {
                MeterTone::Refreshing => {
                    let tick = self.animation.map(|a| a.index).unwrap_or(0);
                    format!("{} refreshing", spinner_frame(tick))
                }
                MeterTone::Unknown => "—".to_string(),
                _ => format!("{pct}%"),
            }
        };

        let label_w = self
            .label
            .map(|l| UnicodeWidthStr::width(l) as u16)
            .unwrap_or(0);
        let has_label = label_w > 0 && area.width > label_w + 6;
        let mut x = area.x;

        if has_label && let Some(lbl) = self.label {
            ui.set_string(
                x,
                area.y,
                lbl,
                Style::default().fg(theme.tokens.text_primary),
            );
            x += label_w + 2;
        }

        let val_w = UnicodeWidthStr::width(value_str.as_str()) as u16;
        let text_style = Style::default().fg(text_color);
        let mut suffix_style = Style::default().fg(fill_color);
        if matches!(self.tone, MeterTone::Exhausted | MeterTone::Critical) {
            suffix_style = suffix_style.add_modifier(Modifier::BOLD);
        }

        if !has_run {
            // No track: value and suffix only
            ui.set_string(x, area.y, &value_str, text_style);
            let sx = x + val_w;
            if sx + 2 <= area.right() {
                ui.set_string(sx, area.y, suffix, suffix_style);
            }
            return area;
        }

        let ratio = self.ratio();
        match self.visual {
            MeterVisual::Line => {
                let track_w = area.right().saturating_sub(x).saturating_sub(val_w + 3);
                if track_w < 4 {
                    ui.set_string(x, area.y, &value_str, text_style);
                    return area;
                }

                let filled = ((track_w as f32) * ratio).round() as u16;
                for i in 0..track_w {
                    let (sym, st) = if i < filled {
                        ("━", Style::default().fg(fill_color))
                    } else {
                        ("─", Style::default().fg(theme.tokens.border_subtle))
                    };
                    ui.set_string(x + i, area.y, sym, st);
                }

                let val_x = x + track_w + 1;
                ui.set_string(val_x, area.y, &value_str, text_style);
                ui.set_string(val_x + val_w, area.y, suffix, suffix_style);
            }
            MeterVisual::Block => {
                let bar_w = area.right().saturating_sub(x).saturating_sub(2);
                if bar_w < 4 {
                    ui.set_string(x, area.y, &value_str, text_style);
                    return area;
                }

                let filled = ((bar_w as f32) * ratio).round() as u16;
                let rest_bg = theme.tokens.surface_elevated;
                let on_fill = if self.tone == MeterTone::Stale {
                    theme.tokens.text_secondary
                } else {
                    theme.tokens.text_on_accent
                };
                let on_rest = if matches!(self.tone, MeterTone::Stale | MeterTone::Refreshing) {
                    theme.tokens.text_muted
                } else {
                    theme.tokens.text_primary
                };

                let chars: Vec<char> = value_str.chars().collect();
                for i in 0..bar_w {
                    let in_fill = i < filled;
                    let cell_bg = if in_fill { fill_color } else { rest_bg };
                    let fg = if in_fill { on_fill } else { on_rest };
                    let mut st = Style::default().fg(fg).bg(cell_bg);
                    if in_fill {
                        st = st.add_modifier(Modifier::BOLD);
                    }
                    let sym = if i >= 1 && (i as usize - 1) < chars.len() {
                        chars[i as usize - 1].to_string()
                    } else {
                        " ".to_string()
                    };
                    ui.set_string(x + i, area.y, &sym, st);
                }
                ui.set_string(x + bar_w, area.y, suffix, suffix_style);
            }
        }

        area
    }
}
