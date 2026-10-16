//! Segment bar: a single line of labelled facts (identity strip, status
//! line). Segments carry a tone and a priority; low-priority segments drop
//! first when the line is narrow. Segments with an id are clickable and
//! lift on hover.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier};

use crate::core::id::WidgetId;
use crate::theme::Tone;
use crate::ui::ctx::RenderCtx;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment {
    pub text: String,
    pub tone: Tone,
    pub bold: bool,
    pub id: Option<WidgetId>,
    /// Higher survives longer when space is short.
    pub priority: u8,
    /// A glyph before the text in its own tone ([`Self::lead`]).
    pub lead: Option<(String, Tone)>,
}

impl Segment {
    pub fn new(text: impl Into<String>, tone: Tone) -> Self {
        Self {
            text: text.into(),
            tone,
            bold: false,
            id: None,
            priority: 5,
            lead: None,
        }
    }
    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }
    pub fn clickable(mut self, id: WidgetId) -> Self {
        self.id = Some(id);
        self
    }
    pub fn priority(mut self, p: u8) -> Self {
        self.priority = p;
        self
    }
    /// Lead the text with a glyph in its own tone, one space apart: a
    /// count in secondary behind a live dot in [`Tone::Active`]
    /// (`● 2 running`). The glyph and the text drop together.
    pub fn lead(mut self, glyph: impl Into<String>, tone: Tone) -> Self {
        self.lead = Some((glyph.into(), tone));
        self
    }
    /// The text as drawn: the lead glyph, a space, the text.
    fn shown(&self) -> String {
        match &self.lead {
            Some((g, _)) => format!("{g} {}", self.text),
            None => self.text.clone(),
        }
    }
}

/// Render `left` segments from the left and `right` segments from the
/// right, separated by two cells. Returns the areas of clickable segments.
pub fn render(
    area: Rect,
    buf: &mut Buffer,
    ctx: &mut RenderCtx,
    left: &[Segment],
    right: &[Segment],
    bg: Color,
) {
    let area = area.intersection(*buf.area());
    if area.is_empty() {
        return;
    }
    let t = ctx.theme;
    let sep = 2u16;
    // the same inset on both sides: the theme's chrome edge
    let edge = t.spacing.chrome_edge;
    let w = |s: &Segment| crate::ui::text::width(&s.shown()) as u16;
    // decide which segments survive
    let mut keep_l: Vec<bool> = vec![true; left.len()];
    let mut keep_r: Vec<bool> = vec![true; right.len()];
    let total = |kl: &[bool], kr: &[bool]| -> u16 {
        let l: u16 = left
            .iter()
            .zip(kl)
            .filter(|(_, k)| **k)
            .map(|(s, _)| w(s) + sep)
            .sum();
        let r: u16 = right
            .iter()
            .zip(kr)
            .filter(|(_, k)| **k)
            .map(|(s, _)| w(s) + sep)
            .sum();
        // separators sit between segments only; the edges are kept clear
        (l + r).saturating_sub(sep) + 2 * edge
    };
    while total(&keep_l, &keep_r) > area.width {
        // drop the lowest priority surviving segment (rightmost on ties)
        let mut best: Option<(u8, bool, usize)> = None;
        for (i, s) in left.iter().enumerate() {
            if keep_l[i] && best.is_none_or(|b| s.priority < b.0) {
                best = Some((s.priority, true, i));
            }
        }
        for (i, s) in right.iter().enumerate() {
            if keep_r[i] && best.is_none_or(|b| s.priority <= b.0) {
                best = Some((s.priority, false, i));
            }
        }
        match best {
            Some((_, true, i)) => keep_l[i] = false,
            Some((_, false, i)) => keep_r[i] = false,
            None => break,
        }
    }
    let draw = |buf: &mut Buffer, ctx: &mut RenderCtx, x: u16, s: &Segment| {
        // a clickable segment walks the ladder; its tone survives hover
        let mut st = match s.id {
            Some(id) => {
                let mut vs = ctx.state(id);
                vs.focused = false;
                t.interactive(crate::theme::Kind::Title, vs, t.plane_of(bg))
                    .style
            }
            None => ratatui::style::Style::new().bg(bg),
        };
        st = st.fg(t.legible(t.tone(s.tone), st.bg.unwrap_or(bg)));
        if s.bold {
            st = st.add_modifier(Modifier::BOLD);
        }
        let pad = u16::from(s.id.is_some());
        let text = if s.id.is_some() {
            format!(" {} ", s.shown())
        } else {
            s.shown()
        };
        buf.set_string(x, area.y, &text, st);
        if let Some((g, tone)) = &s.lead {
            let ink = t.legible(t.tone(*tone), st.bg.unwrap_or(bg));
            buf.set_style(
                Rect::new(x + pad, area.y, crate::ui::text::width(g) as u16, 1),
                ratatui::style::Style::new()
                    .fg(ink)
                    .remove_modifier(Modifier::BOLD),
            );
        }
        if let Some(id) = s.id {
            ctx.clickable(
                id,
                Rect::new(x, area.y, crate::ui::text::width(&text) as u16, 1),
            );
        }
    };
    let mut x = area.x + edge;
    for (s, k) in left.iter().zip(&keep_l) {
        if !k {
            continue;
        }
        let sw = w(s) + if s.id.is_some() { 2 } else { 0 };
        let start = if s.id.is_some() {
            x.saturating_sub(1)
        } else {
            x
        };
        draw(buf, ctx, start, s);
        x += sw.saturating_sub(if s.id.is_some() { 2 } else { 0 }) + sep;
    }
    let mut rx = area.right().saturating_sub(edge);
    for (s, k) in right.iter().zip(&keep_r).rev() {
        if !k {
            continue;
        }
        let sw = w(s);
        rx = rx.saturating_sub(sw);
        let start = if s.id.is_some() {
            rx.saturating_sub(1)
        } else {
            rx
        };
        draw(buf, ctx, start, s);
        rx = rx.saturating_sub(sep);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;

    #[test]
    fn a_lead_glyph_keeps_its_own_tone_before_the_text() {
        let t = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 1));
        let seg = Segment::new("2 running", Tone::Secondary).lead("●", Tone::Active);
        let area = buf.area;
        render(area, &mut buf, &mut ctx, &[seg], &[], t.canvas);
        let row: String = (0..40u16).map(|x| buf[(x, 0)].symbol()).collect();
        let at = row.find('●').unwrap_or_else(|| panic!("{row:?}"));
        let x = row[..at].chars().count() as u16;
        assert_eq!(&row[at..at + "● 2 running".len()], "● 2 running");
        assert_eq!(buf[(x, 0)].fg, t.accent);
        assert_eq!(buf[(x + 2, 0)].fg, t.text_secondary);
    }
}
