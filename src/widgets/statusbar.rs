use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

use crate::core::id::WidgetId;
use crate::theme::{Kind, Plane, Tone};
use crate::ui::ctx::{RenderCtx, VisualState, fill};
use crate::ui::text::{truncate, width};
use crate::widgets::progress::{Meter, MeterTone, MeterVisual};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Emphasis {
    #[default]
    Plain,

    Strong,

    Chip,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusItem {
    pub text: String,
    pub tone: Tone,

    pub priority: u8,
    pub id: Option<WidgetId>,
    pub emphasis: Emphasis,

    pub meter: Option<(Option<u8>, MeterTone)>,

    pub meter_stale: bool,

    pub busy: bool,

    pub tip: Option<crate::ui::tooltip::Tip>,
}

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
            meter_stale: false,
            busy: false,
            tip: None,
        }
    }

    pub fn busy(mut self) -> Self {
        self.busy = true;
        self
    }

    pub fn meter(mut self, used_pct: Option<u8>, tone: MeterTone) -> Self {
        self.meter = Some((used_pct, tone));
        self
    }

    pub fn meter_stale(mut self, on: bool) -> Self {
        self.meter_stale = on;
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

    pub fn tooltip(mut self, tip: crate::ui::tooltip::Tip) -> Self {
        self.tip = Some(tip);
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

    pub fn width(&self) -> u16 {
        let w = width(&self.text) as u16 + if self.busy { 2 } else { 0 };
        let base = match self.emphasis {
            Emphasis::Chip => w + 2,
            _ => w,
        };
        if self.meter.is_some() {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placed {
    pub group: Group,
    pub index: usize,
    pub x: u16,
    pub width: u16,

    pub text: String,
}

#[derive(Debug, Clone, Default)]
pub struct StatusBar {
    pub left: Vec<StatusItem>,
    pub center: Vec<StatusItem>,
    pub right: Vec<StatusItem>,

    pub reveal: Option<crate::ui::reveal::Reveal>,
}

impl crate::ui::measure::Measure for StatusBar {
    fn measure(
        &self,
        _avail: ratatui::layout::Size,
        _ctx: &crate::ui::ctx::RenderCtx,
    ) -> ratatui::layout::Size {
        let group = |items: &[StatusItem]| -> u16 {
            let n = items.len() as u16;
            items.iter().map(StatusItem::width).sum::<u16>() + n.saturating_sub(1) * GAP
        };
        let ws = [group(&self.left), group(&self.center), group(&self.right)];
        let present = ws.iter().filter(|w| **w > 0).count() as u16;
        let w = ws.iter().sum::<u16>() + present.saturating_sub(1) * GAP + 2 * EDGE;
        ratatui::layout::Size::new(w, 1)
    }

    fn min_width(&self, _avail: ratatui::layout::Size, _ctx: &crate::ui::ctx::RenderCtx) -> u16 {
        FLEX_MIN
    }
}

const FLEX_MIN: u16 = 24;

pub fn reveal_id(group: Group, index: usize) -> WidgetId {
    WidgetId::of("statusbar.reveal").child(group as usize * 1000 + index)
}

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

    pub fn layout(&self, area: Rect) -> Vec<Placed> {
        self.plan(area).0
    }

    fn plan(&self, area: Rect) -> (Vec<Placed>, Vec<(Group, usize)>, Option<u16>) {
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

            let gone = keep.iter().flatten().any(|k| !k);
            let more = if gone { 1 + GAP } else { 0 };
            ws.iter().sum::<u16>() + gaps + more + 2 * EDGE
        };

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
                let pad =
                    if it.emphasis == Emphasis::Chip { 2 } else { 0 } + if it.busy { 2 } else { 0 };
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
        let mut right_start = if right_items.is_empty() {
            area.right().saturating_sub(EDGE)
        } else {
            rx + GAP
        };

        let hidden: Vec<(Group, usize)> = groups
            .iter()
            .enumerate()
            .flat_map(|(g, grp)| {
                keep[g]
                    .iter()
                    .enumerate()
                    .filter(|(_, k)| !**k)
                    .map(move |(i, _)| (*grp, i))
            })
            .collect();
        let more_x = (!hidden.is_empty() && total_w > 2 * EDGE).then(|| {
            let at = if right_items.is_empty() {
                right_start.saturating_sub(1)
            } else {
                right_start.saturating_sub(GAP + 1)
            };
            right_start = at;
            at.max(left_end + u16::from(!placed.is_empty()))
                .min(area.right().saturating_sub(EDGE + 1))
        });

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
        (placed, hidden, more_x)
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return;
        }
        let t = ctx.theme;
        let bg = t.surface_elevated;
        fill(buf, area, Style::new().bg(bg));
        let (placed, hidden, more_x) = self.plan(area);
        if let Some(x) = more_x {
            let text: Vec<&str> = hidden
                .iter()
                .map(|(g, i)| self.group(*g)[*i].text.as_str())
                .collect();
            crate::ui::reveal::overflow(
                ctx,
                buf,
                Rect::new(x, area.y, 1, 1),
                &text.join("   "),
                bg,
            );
        }
        for p in placed {
            let it = &self.group(p.group)[p.index];

            let kind = match it.emphasis {
                Emphasis::Chip => Some(Kind::Chip),
                _ => it.id.map(|_| Kind::Title),
            };
            let mut st = match (kind, it.id) {
                (Some(kind), Some(id)) => {
                    let mut s = ctx.state(id);
                    s.focused = false;
                    t.interactive(kind, s, Plane::Card).style
                }
                (Some(kind), None) => {
                    t.interactive(kind, VisualState::default(), Plane::Card)
                        .style
                }
                _ => Style::new().bg(bg),
            };
            st = st.fg(t.legible(t.tone(it.tone), st.bg.unwrap_or(bg)));
            if it.emphasis == Emphasis::Strong {
                st = st.add_modifier(Modifier::BOLD);
            }
            let text = if it.emphasis == Emphasis::Chip {
                format!(" {} ", p.text)
            } else {
                p.text.clone()
            };
            if it.busy {
                let (lead, rest) = match it.emphasis {
                    Emphasis::Chip => (" ", p.text.as_str()),
                    _ => ("", p.text.as_str()),
                };
                let spin = t.glyphs.spinner_frame(ctx.interaction.tick);
                let mut x = p.x;
                if !lead.is_empty() {
                    buf.set_string(x, area.y, lead, st);
                    x += 1;
                }

                let ink = t.legible_mark(t.accent, st.bg.unwrap_or(bg));
                buf.set_string(x, area.y, spin, st.fg(ink));
                buf.set_string(x + 2, area.y, rest, st);
                if it.emphasis == Emphasis::Chip {
                    buf.set_string(x + 2 + width(rest) as u16, area.y, " ", st);
                }
            } else {
                buf.set_string(p.x, area.y, &text, st);
            }
            if let Some((pct, tone)) = it.meter {
                let label_w = width(&text) as u16;
                let mx = p.x + label_w + 1;
                let mw = p.width.saturating_sub(label_w + 1);
                let value = pct
                    .map(|v| format!("{v}%"))
                    .unwrap_or_else(|| t.glyphs.none.into());
                Meter::new(pct)
                    .value(value)
                    .tone(tone)
                    .stale(it.meter_stale)
                    .visual(MeterVisual::Line)
                    .render(Rect::new(mx, area.y, mw, 1), buf, ctx, st.bg.unwrap_or(bg));
            }
            if let Some(id) = it.id {
                ctx.clickable(id, Rect::new(p.x, area.y, p.width, 1));
            }
            if let Some(tip) = &it.tip {
                let seg = Rect::new(p.x, area.y, p.width, 1);
                let id = it.id.unwrap_or_else(|| reveal_id(p.group, p.index));
                if it.id.is_none() && !ctx.inert {
                    ctx.hits.register_hover(id, seg);
                }
                crate::ui::tooltip::show(ctx, id, seg, tip, None);
            }
            if p.text != it.text {
                let lead = u16::from(it.emphasis == Emphasis::Chip) + if it.busy { 2 } else { 0 };
                let cut = Rect::new(p.x + lead, area.y, width(&p.text) as u16, 1);
                let id = it.id.unwrap_or_else(|| reveal_id(p.group, p.index));
                let cfg = self.reveal.unwrap_or(ctx.reveal);
                if it.id.is_none() && cfg.hover {
                    ctx.clickable(id, cut);
                }
                crate::ui::reveal::truncated(ctx, Some(cfg), id, cut, &it.text);
            }
        }
    }
}

impl crate::ui::specimen::Specimen for StatusItem {
    fn specimen_id(&self) -> WidgetId {
        self.id.unwrap_or(WidgetId::of("statusbar.specimen"))
    }

    fn supports(&self, state: crate::ui::specimen::SpecimenState) -> bool {
        use crate::ui::specimen::SpecimenState::*;
        match state {
            Rest | Busy | Narrow => true,
            Hover | Pressed => self.id.is_some(),
            _ => false,
        }
    }

    fn specimen(
        &self,
        state: crate::ui::specimen::SpecimenState,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        _bg: ratatui::style::Color,
    ) {
        let mut item = self.clone();
        item.tip = None;
        item.busy |= state == crate::ui::specimen::SpecimenState::Busy;
        let mut bar = StatusBar::new();
        bar.left.push(item);
        bar.render(area, buf, ctx);
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
    fn measure_keeps_every_item() {
        use crate::ui::measure::Measure;
        let t = Theme::junie();
        let (mut hits, mut ring) = (HitRegistry::default(), FocusRing::default());
        let ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
        let b = bar();
        let w = b.measure(ratatui::layout::Size::new(10, 1), &ctx).width;
        let p = b.layout(Rect::new(0, 0, w, 1));
        assert_eq!(p.len(), 6);
        assert!(p.iter().all(|x| !x.text.ends_with('…')), "{p:?}");
        assert_eq!(b.layout(Rect::new(0, 0, w - 1, 1)).len(), 5);

        let min = b.min_width(ratatui::layout::Size::new(10, 1), &ctx);
        assert!(min < w);
        let p = b.layout(Rect::new(0, 0, min, 1));
        assert!(
            p.iter().any(|x| x.group == Group::Left && x.index == 0),
            "{p:?}"
        );
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

        let chip_x = (0..120u16)
            .find(|x| hits.hit(Position::new(*x, 0)) == Some(id))
            .unwrap();
        assert_eq!(buf[(chip_x + 1, 0)].bg, t.lift(t.surface_overlay));
        assert!(buf[(1, 0)].modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn a_busy_chips_spinner_reads_on_the_chip_at_every_level() {
        use crate::theme::{ColorLevel, contrast_ratio};
        for level in ColorLevel::ALL {
            let t = Theme::for_level(level);
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
            let mut b = StatusBar::new();
            b.right
                .push(StatusItem::new("4 running", Tone::Info).chip().busy());
            let mut buf = Buffer::empty(Rect::new(0, 0, 60, 1));
            b.render(Rect::new(0, 0, 60, 1), &mut buf, &mut ctx);
            let spin = t.glyphs.spinner_frame(ctx.interaction.tick);
            let c = (0..60u16)
                .map(|x| &buf[(x, 0)])
                .find(|c| c.symbol() == spin)
                .unwrap_or_else(|| panic!("{level:?}: no spinner"));
            let r = contrast_ratio(c.fg, c.bg).unwrap();
            assert!(r >= 3.0, "{level:?}: spinner {r:.2}");
        }
    }

    #[test]
    fn hovering_a_cut_segment_reveals_its_full_text() {
        let long = "~/Projects/terminal-components/a/very/deep/working/directory";
        let mut b = StatusBar::new();
        b.left.push(StatusItem::new(long, Tone::Normal));
        let t = Theme::junie();
        let draw = |hover| {
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let interaction = Interaction {
                hover,
                ..Default::default()
            };
            let mut ctx = RenderCtx::new(&t, interaction, &mut hits, &mut ring);
            let mut buf = Buffer::empty(Rect::new(0, 0, 80, 3));
            b.render(Rect::new(0, 2, 30, 1), &mut buf, &mut ctx);
            ctx.flush_overlays(&mut buf);
            let text = (0..3)
                .map(|y| (0..80).map(|x| buf[(x, y)].symbol()).collect::<String>())
                .collect::<Vec<_>>()
                .join("\n");
            (text, hits.hit(Position::new(3, 2)))
        };
        let (rest, hit) = draw(None);
        assert!(!rest.contains(long));
        let id = reveal_id(Group::Left, 0);
        assert_eq!(hit, Some(id), "a cut segment is reachable by the pointer");
        let (hovered, _) = draw(Some(id));
        assert!(hovered.contains(long), "{hovered}");
    }

    #[test]
    fn a_clickable_chip_keeps_its_tone_and_shows_a_press() {
        let t = Theme::junie();
        let id = WidgetId::of("usage");
        let draw = |interaction: Interaction| {
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let mut ctx = RenderCtx::new(&t, interaction, &mut hits, &mut ring);
            let mut buf = Buffer::empty(Rect::new(0, 0, 120, 1));
            bar().render(Rect::new(0, 0, 120, 1), &mut buf, &mut ctx);
            let x = (0..120u16)
                .find(|x| hits.hit(Position::new(*x, 0)) == Some(id))
                .unwrap();
            buf[(x + 1, 0)].clone()
        };
        let rest = draw(Interaction::default());
        let hover = draw(Interaction {
            hover: Some(id),
            ..Default::default()
        });
        let pressed = draw(Interaction {
            hover: Some(id),
            pressed: Some(id),
            ..Default::default()
        });
        assert_eq!(
            hover.fg, t.warning,
            "a warning chip stays amber under the pointer"
        );
        assert_ne!(rest.bg, hover.bg);
        assert_ne!(hover.bg, pressed.bg, "the chip shows a press");
        assert_eq!(pressed.bg, t.ladder(crate::theme::Plane::Raised).pressed);
    }

    #[test]
    fn a_segment_shows_its_tooltip_when_the_pointer_rests() {
        use crate::ui::tooltip::{Tip, tip_id};
        let t = Theme::junie();
        let mut b = StatusBar::new();
        b.left
            .push(StatusItem::new("main", Tone::Normal).tooltip(Tip::new("Current branch")));
        let usage = WidgetId::of("usage");
        b.right.push(
            StatusItem::new("Weekly 59%", Tone::Warning)
                .clickable(usage)
                .tooltip(Tip::new("Usage this week").shortcut("u")),
        );
        let draw = |hover: Option<WidgetId>| {
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let interaction = Interaction {
                hover,
                hover_ms: Some(1_000),
                ..Default::default()
            };
            let mut buf = Buffer::empty(Rect::new(0, 0, 60, 4));
            let mut ctx = RenderCtx::new(&t, interaction, &mut hits, &mut ring);
            b.render(Rect::new(0, 3, 60, 1), &mut buf, &mut ctx);
            ctx.flush_overlays(&mut buf);
            let text: String = (0..4)
                .flat_map(|y| (0..60).map(move |x| (x, y)))
                .map(|(x, y)| buf[(x, y)].symbol().to_owned())
                .collect();
            (text, hits.hover_at(Position::new(2, 3)))
        };
        let (text, plain) = draw(None);
        assert!(!text.contains("Current branch"));
        let plain = plain.expect("the plain segment is a hover target");
        assert_ne!(plain, tip_id(plain));
        assert!(draw(Some(plain)).0.contains("Current branch"));
        assert!(draw(Some(usage)).0.contains("Usage this week  u"));
    }

    #[test]
    fn dropped_items_collapse_into_an_overflow_item_that_reveals_them() {
        let t = Theme::junie();
        let bar = StatusBar {
            left: vec![StatusItem::new("~/work/big", Tone::Secondary).priority(9)],
            center: vec![StatusItem::new("0 selected", Tone::Secondary).priority(2)],
            right: vec![StatusItem::new("truecolor", Tone::Secondary).priority(5)],
            reveal: None,
        };
        let draw = |hover: Option<WidgetId>| {
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let i = Interaction {
                hover,
                ..Interaction::default()
            };
            let mut ctx = RenderCtx::new(&t, i, &mut hits, &mut ring);
            let mut buf = Buffer::empty(Rect::new(0, 0, 30, 3));
            bar.render(Rect::new(0, 2, 30, 1), &mut buf, &mut ctx);
            ctx.flush_overlays(&mut buf);
            drop(ctx);
            let rows: Vec<String> = (0..3u16)
                .map(|y| (0..30u16).map(|x| buf[(x, y)].symbol()).collect())
                .collect();
            (rows, hits)
        };
        let (rows, hits) = draw(None);
        let strip = &rows[2];
        assert!(!strip.contains("selected"), "{strip:?}");
        let at = strip
            .find('…')
            .unwrap_or_else(|| panic!("no overflow item: {strip:?}"));
        let x = strip[..at].chars().count() as u16;
        let id = crate::ui::reveal::overflow_id(Rect::new(x, 2, 1, 1));
        assert_eq!(hits.hover_at(Position::new(x, 2)), Some(id));
        let (rows, _) = draw(Some(id));
        assert!(
            rows.iter().any(|r| r.contains("0 selected")),
            "the hover reveals the dropped item: {rows:#?}"
        );
    }
}
