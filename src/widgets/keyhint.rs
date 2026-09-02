//! Contextual key hints for a footer line: `key Action` pairs, dropping the
//! least important from the right when the line is narrow.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::theme::{BadgeKind, Theme, Tone};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hint {
    pub key: &'static str,
    pub action: &'static str,
}

pub const fn hint(key: &'static str, action: &'static str) -> Hint {
    Hint { key, action }
}

/// Render hints left-aligned; `badge` (e.g. EDIT) leads; `right` is a
/// status message that always wins the right edge. Returns the number of
/// hints that fit.
pub fn render(
    area: Rect,
    buf: &mut Buffer,
    t: &Theme,
    hints: &[Hint],
    badge: Option<(&str, BadgeKind)>,
    right: Option<&str>,
) -> usize {
    render_toned(
        area,
        buf,
        t,
        hints,
        badge,
        right.map(|r| (r, Tone::Secondary)),
    )
}

/// Like [`render`], with a toned status: an error status is drawn as
/// `! message` in the error tone. Hints that do not fit are dropped from
/// the right and a faint `…` marks the cut.
pub fn render_toned(
    area: Rect,
    buf: &mut Buffer,
    t: &Theme,
    hints: &[Hint],
    badge: Option<(&str, BadgeKind)>,
    right: Option<(&str, Tone)>,
) -> usize {
    let area = area.intersection(*buf.area());
    if area.is_empty() {
        return 0;
    }
    let mut x = area.x + 1;
    let mut right_w = 0u16;
    if let Some((r, tone)) = right {
        let text = if tone == Tone::Error {
            format!("! {r}")
        } else {
            r.to_owned()
        };
        let w = crate::ui::text::width(&text) as u16;
        if area.width > w + 2 {
            let st = ratatui::style::Style::new().fg(t.tone(tone));
            buf.set_string(area.right() - w - 1, area.y, &text, st);
            if tone == Tone::Error {
                buf.set_string(
                    area.right() - w - 1,
                    area.y,
                    "!",
                    st.add_modifier(ratatui::style::Modifier::BOLD),
                );
            }
            right_w = w + 3;
        }
    }
    if let Some((text, kind)) = badge {
        let b = format!(" {text} ");
        buf.set_string(x, area.y, &b, t.badge(kind));
        x += crate::ui::text::width(&b) as u16 + 2;
    }
    let limit = area.right().saturating_sub(right_w);
    let mut drawn = 0usize;
    for (i, h) in hints.iter().enumerate() {
        let kw = crate::ui::text::width(h.key) as u16;
        let w = kw + 1 + crate::ui::text::width(h.action) as u16 + 2;
        // keep two cells for the cut marker when more hints follow
        let reserve = if i + 1 < hints.len() { 2 } else { 0 };
        if x + w + reserve > limit {
            break;
        }
        buf.set_string(x, area.y, h.key, t.key_hint_key());
        buf.set_string(x + kw + 1, area.y, h.action, t.key_hint_action());
        x += w;
        drawn += 1;
    }
    if drawn < hints.len() && x + 1 <= limit {
        buf.set_string(x, area.y, "…", t.faint());
    }
    drawn
}
