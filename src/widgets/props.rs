//! Label / value facts, e.g. a connection's details or a plan node's
//! metadata. Labels are muted and right-padded to a shared width.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::theme::{Theme, Tone};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prop {
    pub label: String,
    pub value: String,
    pub tone: Tone,
    pub wrap: bool,
}

impl Prop {
    pub fn new(label: &str, value: impl Into<String>) -> Self {
        Self {
            label: label.to_owned(),
            value: value.into(),
            tone: Tone::Normal,
            wrap: false,
        }
    }
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }
    pub fn wrap(mut self) -> Self {
        self.wrap = true;
        self
    }
}

/// Returns the number of rows used.
pub fn render(area: Rect, buf: &mut Buffer, t: &Theme, props: &[Prop], bg: Color) -> u16 {
    let area = area.intersection(*buf.area());
    if area.is_empty() {
        return 0;
    }
    let label_w = props
        .iter()
        .map(|p| crate::ui::text::width(&p.label))
        .max()
        .unwrap_or(0) as u16
        + 2;
    let mut y = area.y;
    for p in props {
        if y >= area.bottom() {
            break;
        }
        buf.set_string(area.x, y, &p.label, t.muted().bg(bg));
        let vw = area.width.saturating_sub(label_w) as usize;
        let style = ratatui::style::Style::new().fg(t.tone(p.tone)).bg(bg);
        if p.wrap {
            for line in crate::ui::text::wrap(&p.value, vw.max(4)) {
                if y >= area.bottom() {
                    break;
                }
                buf.set_string(area.x + label_w, y, &line, style);
                y += 1;
            }
        } else {
            buf.set_string(
                area.x + label_w,
                y,
                crate::ui::text::truncate(&p.value, vw),
                style,
            );
            y += 1;
        }
    }
    y - area.y
}
