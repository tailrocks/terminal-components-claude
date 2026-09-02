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
}

impl Segment {
    pub fn new(text: impl Into<String>, tone: Tone) -> Self {
        Self {
            text: text.into(),
            tone,
            bold: false,
            id: None,
            priority: 5,
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
    let w = |s: &Segment| crate::ui::text::width(&s.text) as u16;
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
        l + r + 2
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
        let hovered = s.id.is_some_and(|id| ctx.interaction.hovered(id));
        let mut st = ratatui::style::Style::new().fg(t.tone(s.tone)).bg(bg);
        if hovered {
            st = st.bg(t.lift(bg)).fg(t.text_primary);
        }
        if s.bold {
            st = st.add_modifier(Modifier::BOLD);
        }
        let text = if s.id.is_some() {
            format!(" {} ", s.text)
        } else {
            s.text.clone()
        };
        buf.set_string(x, area.y, &text, st);
        if let Some(id) = s.id {
            ctx.clickable(
                id,
                Rect::new(x, area.y, crate::ui::text::width(&text) as u16, 1),
            );
        }
    };
    let mut x = area.x + 1;
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
    let mut rx = area.right().saturating_sub(1);
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
