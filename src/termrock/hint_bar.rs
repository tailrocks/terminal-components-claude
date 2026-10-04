//! Bottom Hint Bar chrome component.
//!
//! Paints the shell-owned bottom key hint strip with badge, status, and responsive
//! hint item truncation with ellipsis markers.

use ratatui::style::{Modifier, Style};
use unicode_width::UnicodeWidthStr;

use crate::termrock::identity::Id;
use crate::termrock::layout::{Constraints, Rect, Size};
use crate::termrock::runtime::{MeasureCx, Ui};
use crate::termrock::theme::{StylePatch, Tone};

/// Single hint item pairing a key chord with an action description.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HintItem<'a> {
    pub chord: &'a str,
    pub description: &'a str,
}

impl<'a> HintItem<'a> {
    pub const fn new(chord: &'a str, description: &'a str) -> Self {
        Self { chord, description }
    }
}

/// Bottom hint strip component.
#[derive(Clone)]
pub struct HintBar<'a> {
    pub id: Id,
    pub hints: &'a [HintItem<'a>],
    pub badge: Option<&'a str>,
    pub status: Option<(&'a str, Tone)>,
    pub centered: bool,
    pub patch: StylePatch,
}

impl<'a> HintBar<'a> {
    pub fn new(id: Id, hints: &'a [HintItem<'a>]) -> Self {
        Self {
            id,
            hints,
            badge: None,
            status: None,
            centered: false,
            patch: StylePatch::empty(),
        }
    }

    pub fn badge(mut self, badge: &'a str) -> Self {
        self.badge = Some(badge);
        self
    }

    pub fn status(mut self, status: &'a str, tone: Tone) -> Self {
        self.status = Some((status, tone));
        self
    }

    pub fn centered(mut self, centered: bool) -> Self {
        self.centered = centered;
        self
    }

    pub fn patch(mut self, patch: StylePatch) -> Self {
        self.patch = patch;
        self
    }

    pub fn measure(&self, _cx: &MeasureCx<'_>, constraints: Constraints) -> Size {
        constraints.clamp(Size::new(constraints.max.width, 1))
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        if area.is_empty() {
            return area;
        }

        let theme = ui.theme;
        let bg_style = Style::default().bg(theme.tokens.surface);
        ui.fill_rect(area, bg_style);

        let mut x = area.x + 1;

        // Render leading badge if specified (e.g. " EDIT ")
        if let Some(badge) = self.badge {
            let b_str = format!(" {badge} ");
            let bw = UnicodeWidthStr::width(b_str.as_str()) as u16;
            if area.width > bw + 2 {
                let badge_style = Style::default()
                    .bg(theme.tokens.accent)
                    .fg(theme.tokens.text_on_accent)
                    .add_modifier(Modifier::BOLD);
                ui.set_string(x, area.y, &b_str, badge_style);
                x += bw + 2;
            }
        }

        // Render trailing status if specified
        let mut right_w = 0u16;
        if let Some((status_text, tone)) = self.status {
            let mark = match tone {
                Tone::Danger => "!",
                Tone::Warning => "▲",
                _ => "",
            };
            let st_full = if mark.is_empty() {
                status_text.to_string()
            } else {
                format!("{mark} {status_text}")
            };
            let sw = UnicodeWidthStr::width(st_full.as_str()) as u16;
            if area.right() > x + sw + 1 {
                let status_color = match tone {
                    Tone::Danger => theme.tokens.danger,
                    Tone::Warning => theme.tokens.warning,
                    Tone::Success => theme.tokens.success,
                    Tone::Accent => theme.tokens.accent,
                    Tone::Muted => theme.tokens.text_muted,
                    Tone::Primary | Tone::Normal => theme.tokens.text_primary,
                };
                let sx = area.right().saturating_sub(sw + 1);
                ui.set_string(sx, area.y, &st_full, Style::default().fg(status_color));
                right_w = sw + 3;
            }
        }

        let limit = area.right().saturating_sub(right_w);
        let hint_w = |h: &HintItem<'a>| {
            UnicodeWidthStr::width(h.chord) as u16
                + 1
                + UnicodeWidthStr::width(h.description) as u16
                + 2
        };

        if self.centered {
            let mut used = 0u16;
            let mut count = 0usize;
            for (i, h) in self.hints.iter().enumerate() {
                let reserve = if i + 1 < self.hints.len() { 2 } else { 0 };
                if x + used + hint_w(h) + reserve > limit {
                    break;
                }
                used += hint_w(h);
                count += 1;
            }
            if count < self.hints.len() {
                used += 2;
            }
            let free = area.width.saturating_sub(used);
            let mid = area.x + free / 2;
            x = mid.max(x).min(limit.saturating_sub(used).max(x));
        }

        let mut drawn = 0usize;
        let key_style = Style::default()
            .fg(theme.tokens.text_primary)
            .add_modifier(Modifier::BOLD);
        let desc_style = Style::default().fg(theme.tokens.text_muted);

        for (i, h) in self.hints.iter().enumerate() {
            let kw = UnicodeWidthStr::width(h.chord) as u16;
            let w = hint_w(h);
            let reserve = if i + 1 < self.hints.len() { 2 } else { 0 };
            if x + w + reserve > limit {
                break;
            }
            ui.set_string(x, area.y, h.chord, key_style);
            ui.set_string(x + kw + 1, area.y, h.description, desc_style);
            x += w;
            drawn += 1;
        }

        if drawn < self.hints.len() && x < limit {
            ui.set_string(x, area.y, "…", Style::default().fg(theme.tokens.text_faint));
        }

        area
    }
}
