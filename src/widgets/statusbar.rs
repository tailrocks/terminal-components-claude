//! Status bar: one full-width row on its own plane that reports the state
//! of a surface in three groups — left (identity/context), center
//! (activity), right (quotas, runtime facts). Groups are separated by
//! spacing and tone, not by separator glyphs; when the row is too narrow
//! items leave by ascending priority (center first, then right, then left)
//! and the strongest left item always survives, truncated.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

use crate::core::id::WidgetId;
use crate::theme::Tone;
use crate::ui::ctx::{RenderCtx, fill};
use crate::ui::text::{truncate, width};
use crate::widgets::progress::{Meter, MeterTone, MeterVisual};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Emphasis {
    #[default]
    Plain,
    /// Bold: the item that names the surface.
    Strong,
    /// ` text ` on the overlay plane: a fact with its own edge (a quota, a
    /// runtime id).
    Chip,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusItem {
    pub text: String,
    pub tone: Tone,
    /// Higher survives longer when the row is narrow (0–9).
    pub priority: u8,
    pub id: Option<WidgetId>,
    pub emphasis: Emphasis,
    /// A compact line meter after the text: `(used %, tone)`.
    pub meter: Option<(Option<u8>, MeterTone)>,
}

/// Track cells of an inline status meter.
pub const STATUS_METER_TRACK: u16 = 10;

impl StatusItem {
    pub fn new(text: impl Into<String>, tone: Tone) -> Self {
        Self {
            text: text.into(),
            tone,
            priority: 5,
            id: None,
            emphasis: Emphasis::Plain,
            meter: None,
        }
    }
    /// Append a compact line meter (label, then `━━━━──── 76%`).
    pub fn meter(mut self, used_pct: Option<u8>, tone: MeterTone) -> Self {
        self.meter = Some((used_pct, tone));
        self
    }
    pub fn priority(mut self, p: u8) -> Self {
        self.priority = p;
        self
    }
    pub fn clickable(mut self, id: WidgetId) -> Self {
        self.id = Some(id);
        self
    }
    pub fn strong(mut self) -> Self {
        self.emphasis = Emphasis::Strong;
        self
    }
    pub fn chip(mut self) -> Self {
        self.emphasis = Emphasis::Chip;
        self
    }

    /// Cells the item occupies (chips carry their own padding).
    pub fn width(&self) -> u16 {
        let w = width(&self.text) as u16;
        let base = match self.emphasis {
            Emphasis::Chip => w + 2,
            _ => w,
        };
        if self.meter.is_some() {
            // label, a space, the track, the value and its marker column
            base + 1 + STATUS_METER_TRACK + 7
        } else {
            base
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    Left,
    Center,
    Right,
}

/// One placed item: which group, which index in that group, where.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placed {
    pub group: Group,
    pub index: usize,
    pub x: u16,
    pub width: u16,
    /// Text after truncation (chips keep their padding out of this).
    pub text: String,
}

#[derive(Debug, Clone, Default)]
pub struct StatusBar {
    pub left: Vec<StatusItem>,
    pub center: Vec<StatusItem>,
    pub right: Vec<StatusItem>,
}

/// Cells between items within a group and between groups.
const GAP: u16 = 3;
const EDGE: u16 = 1;

impl StatusBar {
    pub fn new() -> Self {
        Self::default()
    }

    fn group(&self, g: Group) -> &[StatusItem] {
        match g {
            Group::Left => &self.left,
            Group::Center => &self.center,
            Group::Right => &self.right,
        }
    }

    /// Decide which items survive at `width` and where they go.
    pub fn layout(&self, area: Rect) -> Vec<Placed> {
        let total_w = area.width;
        let mut keep: [Vec<bool>; 3] = [
            vec![true; self.left.len()],
            vec![true; self.center.len()],
            vec![true; self.right.len()],
        ];
        let groups = [Group::Left, Group::Center, Group::Right];
        let group_w = |g: usize, keep: &[Vec<bool>; 3]| -> u16 {
            let items = self.group(groups[g]);
            let mut w = 0u16;
            let mut n = 0u16;
            for (it, k) in items.iter().zip(&keep[g]) {
                if *k {
                    w += it.width();
                    n += 1;
                }
            }
            if n > 0 { w + (n - 1) * GAP } else { 0 }
        };
        let needed = |keep: &[Vec<bool>; 3]| -> u16 {
            let ws = [group_w(0, keep), group_w(1, keep), group_w(2, keep)];
            let present = ws.iter().filter(|w| **w > 0).count() as u16;
            let gaps = present.saturating_sub(1) * GAP;
            ws.iter().sum::<u16>() + gaps + 2 * EDGE
        };
        // drop the lowest priority anywhere first; ties leave the center,
        // then the right, then the left; the strongest left item never
        // leaves (it truncates instead)
        while needed(&keep) > total_w {
            let mut victim: Option<(u8, usize, usize)> = None;
            for g in [1usize, 2, 0] {
                let items = self.group(groups[g]);
                let alive: Vec<usize> = (0..items.len()).filter(|i| keep[g][*i]).collect();
                if alive.is_empty() || (g == 0 && alive.len() == 1) {
                    continue;
                }
                let i = alive
                    .iter()
                    .copied()
                    .min_by_key(|i| (items[*i].priority, usize::MAX - *i))
                    .unwrap();
                let p = items[i].priority;
                if victim.is_none_or(|(vp, _, _)| p < vp) {
                    victim = Some((p, g, i));
                }
            }
            match victim {
                Some((_, g, i)) => keep[g][i] = false,
                None => break,
            }
        }
        let mut placed = vec![];
        // left
        let mut x = area.x + EDGE;
        let left_budget = total_w.saturating_sub(2 * EDGE);
        for (i, it) in self.left.iter().enumerate() {
            if !keep[0][i] {
                continue;
            }
            let mut w = it.width();
            let mut text = it.text.clone();
            let room = (area.x + EDGE + left_budget).saturating_sub(x);
            if w > room {
                let pad = if it.emphasis == Emphasis::Chip { 2 } else { 0 };
                text = truncate(&it.text, room.saturating_sub(pad) as usize);
                w = width(&text) as u16 + pad;
            }
            placed.push(Placed {
                group: Group::Left,
                index: i,
                x,
                width: w,
                text,
            });
            x += w + GAP;
        }
        let left_end = x.saturating_sub(GAP);
        // right
        let mut rx = area.right().saturating_sub(EDGE);
        let mut right_items: Vec<Placed> = vec![];
        for (i, it) in self.right.iter().enumerate().rev() {
            if !keep[2][i] {
                continue;
            }
            let w = it.width();
            rx = rx.saturating_sub(w);
            right_items.push(Placed {
                group: Group::Right,
                index: i,
                x: rx,
                width: w,
                text: it.text.clone(),
            });
            rx = rx.saturating_sub(GAP);
        }
        let right_start = if right_items.is_empty() {
            area.right().saturating_sub(EDGE)
        } else {
            rx + GAP
        };
        // center: centred in the free span between the groups
        let cw = group_w(1, &keep);
        if cw > 0 {
            let lo = left_end + GAP;
            let hi = right_start.saturating_sub(GAP);
            let free = hi.saturating_sub(lo);
            let mut cx = lo + free.saturating_sub(cw) / 2;
            for (i, it) in self.center.iter().enumerate() {
                if !keep[1][i] {
                    continue;
                }
                let w = it.width();
                placed.push(Placed {
                    group: Group::Center,
                    index: i,
                    x: cx,
                    width: w,
                    text: it.text.clone(),
                });
                cx += w + GAP;
            }
        }
        right_items.reverse();
        placed.extend(right_items);
        placed
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return;
        }
        let t = ctx.theme;
        let bg = t.surface_elevated;
        fill(buf, area, Style::new().bg(bg));
        for p in self.layout(area) {
            let it = &self.group(p.group)[p.index];
            let hovered = it.id.is_some_and(|id| ctx.interaction.hovered(id));
            let mut st = Style::new().fg(t.tone(it.tone)).bg(bg);
            match it.emphasis {
                Emphasis::Strong => st = st.add_modifier(Modifier::BOLD),
                Emphasis::Chip => st = st.bg(t.surface_overlay),
                Emphasis::Plain => {}
            }
            if hovered {
                st = st.bg(t.lift(st.bg.unwrap_or(bg))).fg(t.text_primary);
            }
            let text = if it.emphasis == Emphasis::Chip {
                format!(" {} ", p.text)
            } else {
                p.text.clone()
            };
            buf.set_string(p.x, area.y, &text, st);
            if let Some((pct, tone)) = it.meter {
                let label_w = width(&text) as u16;
                let mx = p.x + label_w + 1;
                let mw = p.width.saturating_sub(label_w + 1);
                let value = pct.map(|v| format!("{v}%")).unwrap_or_else(|| "—".into());
                Meter::new(pct)
                    .value(value)
                    .tone(tone)
                    .visual(MeterVisual::Line)
                    .render(Rect::new(mx, area.y, mw, 1), buf, ctx, st.bg.unwrap_or(bg));
            }
            if let Some(id) = it.id {
                ctx.clickable(id, Rect::new(p.x, area.y, p.width, 1));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;
    use ratatui::layout::Position;

    fn bar() -> StatusBar {
        let mut b = StatusBar::new();
        b.left.push(
            StatusItem::new("payments-platform", Tone::Normal)
                .strong()
                .priority(9),
        );
        b.left
            .push(StatusItem::new("PR #482 · settlement backoff", Tone::Secondary).priority(7));
        b.center
            .push(StatusItem::new("Claude Code · working", Tone::Secondary).priority(4));
        b.right.push(
            StatusItem::new("Weekly 59%", Tone::Warning)
                .chip()
                .priority(6)
                .clickable(WidgetId::of("usage")),
        );
        b.right.push(
            StatusItem::new("jackin-payments-7f3a", Tone::Muted)
                .chip()
                .priority(3),
        );
        b.right
            .push(StatusItem::new("run 9c41", Tone::Faint).priority(2));
        b
    }

    #[test]
    fn groups_keep_their_order_and_sides() {
        let b = bar();
        let p = b.layout(Rect::new(0, 0, 160, 1));
        assert_eq!(p.len(), 6);
        let left: Vec<&Placed> = p.iter().filter(|x| x.group == Group::Left).collect();
        assert_eq!(left[0].x, 1);
        assert!(left[1].x > left[0].x + left[0].width);
        let right: Vec<&Placed> = p.iter().filter(|x| x.group == Group::Right).collect();
        assert_eq!(right.last().unwrap().x + right.last().unwrap().width, 159);
        assert!(right[0].x < right[1].x && right[1].x < right[2].x);
        let c = p.iter().find(|x| x.group == Group::Center).unwrap();
        assert!(c.x > left[1].x + left[1].width && c.x + c.width < right[0].x);
    }

    #[test]
    fn narrow_rows_drop_center_then_right_then_left_and_keep_the_name() {
        let b = bar();
        let p = b.layout(Rect::new(0, 0, 80, 1));
        assert!(
            p.iter().all(|x| x.group != Group::Center),
            "center leaves first"
        );
        assert!(p.iter().any(|x| x.group == Group::Left && x.index == 0));
        let p = b.layout(Rect::new(0, 0, 16, 1));
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].group, Group::Left);
        assert!(p[0].text.ends_with('…'), "{:?}", p[0].text);
        assert!(p[0].width <= 14);
    }

    #[test]
    fn render_fills_the_row_and_registers_hover() {
        let t = Theme::junie();
        let id = WidgetId::of("usage");
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(
            &t,
            Interaction {
                hover: Some(id),
                ..Default::default()
            },
            &mut hits,
            &mut ring,
        );
        let mut buf = Buffer::empty(Rect::new(0, 0, 120, 1));
        bar().render(Rect::new(0, 0, 120, 1), &mut buf, &mut ctx);
        // every cell outside an item keeps the plane; chips sit one plane up
        let placed = bar().layout(Rect::new(0, 0, 120, 1));
        for x in 0..120u16 {
            let inside = placed.iter().any(|p| x >= p.x && x < p.x + p.width);
            if !inside {
                assert_eq!(
                    buf[(x, 0)].bg,
                    t.surface_elevated,
                    "cell {x} keeps the plane"
                );
            }
        }
        // the hovered chip is lifted and hit-tested
        let chip_x = (0..120u16)
            .find(|x| hits.hit(Position::new(*x, 0)) == Some(id))
            .unwrap();
        assert_eq!(buf[(chip_x + 1, 0)].bg, t.lift(t.surface_overlay));
        assert!(buf[(1, 0)].modifier.contains(Modifier::BOLD));
    }
}
