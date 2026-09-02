//! Contextual key hints for a footer line: `key Action` pairs, dropping the
//! least important from the right when the line is narrow.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::theme::{BadgeKind, Theme};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hint {
    pub key: &'static str,
    pub action: &'static str,
}

pub const fn hint(key: &'static str, action: &'static str) -> Hint {
    Hint { key, action }
}

/// Render hints left-aligned; `badge` (e.g. EDIT) leads; `right` is a
/// status message that always wins the right edge.
pub fn render(
    area: Rect,
    buf: &mut Buffer,
    t: &Theme,
    hints: &[Hint],
    badge: Option<(&str, BadgeKind)>,
    right: Option<&str>,
) {
    let area = area.intersection(*buf.area());
    if area.is_empty() {
        return;
    }
    let mut x = area.x + 1;
    let mut right_w = 0u16;
    if let Some(r) = right {
        let w = crate::ui::text::width(r) as u16;
        if area.width > w + 2 {
            buf.set_string(area.right() - w - 1, area.y, r, t.secondary());
            right_w = w + 3;
        }
    }
    if let Some((text, kind)) = badge {
        let b = format!(" {text} ");
        buf.set_string(x, area.y, &b, t.badge(kind));
        x += crate::ui::text::width(&b) as u16 + 2;
    }
    for h in hints {
        let kw = crate::ui::text::width(h.key) as u16;
        let w = kw + 1 + crate::ui::text::width(h.action) as u16 + 2;
        if x + w + right_w > area.right() {
            break;
        }
        buf.set_string(x, area.y, h.key, t.key_hint_key());
        buf.set_string(x + kw + 1, area.y, h.action, t.key_hint_action());
        x += w;
    }
}
