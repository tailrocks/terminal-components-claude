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
/// `! message` in the error tone and a warning as `▲ message` in the
/// warning tone, so the weight survives monochrome. Hints that do not fit are dropped from
/// the right and a faint `…` marks the cut.
pub fn render_toned(
    area: Rect,
    buf: &mut Buffer,
    t: &Theme,
    hints: &[Hint],
    badge: Option<(&str, BadgeKind)>,
    right: Option<(&str, Tone)>,
) -> usize {
    render_aligned(area, buf, t, hints, badge, right, false)
}

/// Like [`render_toned`]; `centered` places the hints that fit in the
/// middle of the row (the status keeps the right edge and wins the space).
pub fn render_aligned(
    area: Rect,
    buf: &mut Buffer,
    t: &Theme,
    hints: &[Hint],
    badge: Option<(&str, BadgeKind)>,
    right: Option<(&str, Tone)>,
    centered: bool,
) -> usize {
    let area = area.intersection(*buf.area());
    if area.is_empty() {
        return 0;
    }
    let mut x = area.x + 1;
    if let Some((text, kind)) = badge {
        let b =
            crate::ui::text::truncate(&format!(" {text} "), area.width.saturating_sub(2) as usize);
        if !b.is_empty() {
            buf.set_string(x, area.y, &b, t.badge(kind));
            x = (x + crate::ui::text::width(&b) as u16 + 2).min(area.right());
        }
    }
    let mut right_w = 0u16;
    if let Some((r, tone)) = right {
        // the glyph carries the meaning in monochrome, the tone in colour
        let mark = match tone {
            Tone::Error => "!",
            Tone::Warning => "▲",
            _ => "",
        };
        let text = if mark.is_empty() {
            r.to_owned()
        } else {
            format!("{mark} {r}")
        };
        let available = area.right().saturating_sub(x + 1) as usize;
        let text = crate::ui::text::truncate(&text, available);
        let w = crate::ui::text::width(&text) as u16;
        if w > 0 {
            let st = ratatui::style::Style::new().fg(t.tone(tone));
            buf.set_string(area.right() - w - 1, area.y, &text, st);
            if !mark.is_empty() {
                buf.set_string(
                    area.right() - w - 1,
                    area.y,
                    mark,
                    st.add_modifier(ratatui::style::Modifier::BOLD),
                );
            }
            right_w = w + 3;
        }
    }
    let limit = area.right().saturating_sub(right_w);
    if centered {
        // measure what fits, then start so the block sits mid-row
        let hint_w = |h: &Hint| {
            crate::ui::text::width(h.key) as u16 + 1 + crate::ui::text::width(h.action) as u16 + 2
        };
        let mut used = 0u16;
        let mut n = 0usize;
        for (i, h) in hints.iter().enumerate() {
            let reserve = if i + 1 < hints.len() { 2 } else { 0 };
            if x + used + hint_w(h) + reserve > limit {
                break;
            }
            used += hint_w(h);
            n += 1;
        }
        if n < hints.len() {
            used += 2;
        }
        let free = area.width.saturating_sub(used);
        let mid = area.x + free / 2;
        // never past the badge, never under the status
        let start = mid.max(x).min(limit.saturating_sub(used).max(x));
        x = start;
    }
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
    if drawn < hints.len() && x < limit {
        buf.set_string(x, area.y, "…", t.faint());
    }
    drawn
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row_text(buf: &Buffer, width: u16) -> String {
        (0..width)
            .map(|x| buf[(x, 0)].symbol().to_owned())
            .collect()
    }

    #[test]
    fn warning_and_error_statuses_carry_glyphs_so_mono_keeps_the_weight() {
        let t = Theme::junie();
        let hints = [hint("Esc", "Back")];
        for (tone, mark) in [(Tone::Warning, "▲"), (Tone::Error, "!")] {
            let mut buf = Buffer::empty(Rect::new(0, 0, 60, 1));
            render_toned(
                Rect::new(0, 0, 60, 1),
                &mut buf,
                &t,
                &hints,
                None,
                Some(("disk low", tone)),
            );
            let row = row_text(&buf, 60);
            assert!(row.contains(&format!("{mark} disk low")), "{row:?}");
        }
        let mut plain = Buffer::empty(Rect::new(0, 0, 60, 1));
        render_toned(
            Rect::new(0, 0, 60, 1),
            &mut plain,
            &t,
            &hints,
            None,
            Some(("Saved", Tone::Secondary)),
        );
        let row = row_text(&plain, 60);
        assert!(row.contains("Saved"), "{row:?}");
        assert!(!row.contains('▲') && !row.contains('!'), "{row:?}");
    }

    #[test]
    fn long_status_keeps_its_severity_and_yields_hints_without_overlapping_edit_badge() {
        for level in [
            crate::theme::ColorLevel::TrueColor,
            crate::theme::ColorLevel::Mono,
        ] {
            let t = Theme::for_level(level);
            let mut buf = Buffer::empty(Rect::new(0, 0, 32, 1));
            let count = render_toned(
                *buf.area(),
                &mut buf,
                &t,
                &[hint("Esc", "Cancel")],
                Some(("EDIT", BadgeKind::Edit)),
                Some(("无法保存配置文件：目标目录只读 e\u{301}", Tone::Error)),
            );
            let row = row_text(&buf, 32);
            assert_eq!(count, 0);
            assert!(row.starts_with("  EDIT "), "{row:?}");
            assert!(row.contains("! 无"), "{row:?}");
            assert!(row.contains('…'), "{row:?}");
            assert!(!row.contains("Cancel"));
        }
    }

    #[test]
    fn narrow_status_and_badge_stay_inside_assigned_area() {
        let t = Theme::junie();
        for width in 1..16 {
            let area = Rect::new(2, 1, width, 1);
            let mut buf = Buffer::empty(Rect::new(0, 0, 20, 3));
            for cell in &mut buf.content {
                cell.set_symbol(".");
            }
            render_toned(
                area,
                &mut buf,
                &t,
                &[hint("Esc", "Cancel")],
                Some(("EDIT", BadgeKind::Edit)),
                Some(("long failure message", Tone::Error)),
            );
            for y in 0..3 {
                for x in 0..20 {
                    if !area.contains((x, y).into()) {
                        assert_eq!(buf[(x, y)].symbol(), ".");
                    }
                }
            }
        }
    }
}
