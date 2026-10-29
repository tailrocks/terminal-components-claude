use core::fmt;

use crate::theme::PaintStyle;
use ratatui_core::layout::Rect;
use ratatui_core::style::Modifier;

use super::{PartStyle, SlotFn, first_row, shift};
use crate::id::{Id, ItemKey, Part, PartRef};
use crate::intent::{Intent, Phase};
use crate::measure::{Constraints, Size};
use crate::meter::{Meter, MeterTone};
use crate::progress::{PCT_COLUMNS, spinner_glyph};
use crate::response::{Response, StateFlags};
use crate::text::width;
use crate::theme::{Family, GlyphRole, Role, Slot, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, Ui};
use termrock_runtime::Status;

pub const MAX_ITEMS: usize = 16;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Group {
    Left,
    Center,
    Right,
}

impl Group {
    pub const ALL: [Group; 3] = [Group::Left, Group::Center, Group::Right];

    const fn index(self) -> usize {
        match self {
            Group::Left => 0,
            Group::Center => 1,
            Group::Right => 2,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[non_exhaustive]
pub enum Emphasis {
    #[default]
    Plain,
    Strong,
    Chip,
}

#[derive(Clone, Copy, Debug)]
pub struct StatusItem<'a> {
    text: &'a str,
    tone: Option<Role>,
    priority: u8,
    key: Option<ItemKey>,
    emphasis: Emphasis,
    ratio: Option<f64>,
    meter_tone: Option<MeterTone>,
    spinner: Option<usize>,
}

impl<'a> StatusItem<'a> {
    pub const DEFAULT_PRIORITY: u8 = 5;

    pub const fn new(text: &'a str) -> Self {
        StatusItem {
            text,
            tone: None,
            priority: Self::DEFAULT_PRIORITY,
            key: None,
            emphasis: Emphasis::Plain,
            ratio: None,
            meter_tone: None,
            spinner: None,
        }
    }

    #[must_use]
    pub const fn spinner(mut self, frame: usize) -> Self {
        self.spinner = Some(frame);
        self
    }

    #[must_use]
    pub const fn tone(mut self, r: Role) -> Self {
        self.tone = Some(r);
        self
    }

    #[must_use]
    pub const fn priority(mut self, p: u8) -> Self {
        self.priority = p;
        self
    }

    #[must_use]
    pub const fn key(mut self, k: ItemKey) -> Self {
        self.key = Some(k);
        self
    }

    #[must_use]
    pub const fn strong(mut self) -> Self {
        self.emphasis = Emphasis::Strong;
        self
    }

    #[must_use]
    pub const fn chip(mut self) -> Self {
        self.emphasis = Emphasis::Chip;
        self
    }

    #[must_use]
    pub fn meter(mut self, ratio: f64) -> Self {
        self.ratio = Some(ratio.clamp(0.0, 1.0));
        self
    }

    #[must_use]
    pub const fn meter_tone(mut self, t: MeterTone) -> Self {
        self.meter_tone = Some(t);
        self
    }

    pub const fn text(&self) -> &'a str {
        self.text
    }

    pub const fn priority_of(&self) -> u8 {
        self.priority
    }

    pub const fn key_of(&self) -> Option<ItemKey> {
        self.key
    }

    pub const fn emphasis(&self) -> Emphasis {
        self.emphasis
    }

    fn spinner_glyph(&self, metrics: ItemMetrics) -> &'static str {
        self.spinner
            .and_then(|frame| spinner_glyph(metrics.spinner_frames, frame))
            .unwrap_or("")
    }

    fn spinner_columns(&self, metrics: ItemMetrics) -> u16 {
        let glyph = self.spinner_glyph(metrics);
        if glyph.is_empty() {
            0
        } else {
            width(glyph).saturating_add(u16::from(!self.text.is_empty()))
        }
    }

    fn label_columns(&self, metrics: ItemMetrics) -> u16 {
        let w = width(self.text).saturating_add(self.spinner_columns(metrics));
        match self.emphasis {
            Emphasis::Chip => w.saturating_add(2),
            Emphasis::Plain | Emphasis::Strong => w,
        }
    }

    fn columns(&self, metrics: ItemMetrics) -> u16 {
        match self.ratio {
            Some(_) => self
                .label_columns(metrics)
                .saturating_add(1)
                .saturating_add(metrics.meter_columns),
            None => self.label_columns(metrics),
        }
    }
}

#[derive(Clone, Copy)]
struct ItemMetrics {
    meter_columns: u16,
    spinner_frames: &'static [&'static str],
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum StatusAction {
    Chose(ItemKey),
}

pub struct StatusBar<'a> {
    id: Id,
    left: &'a [StatusItem<'a>],
    center: &'a [StatusItem<'a>],
    right: &'a [StatusItem<'a>],
    variant: Variant,
    status: Status,
    frame: usize,
    patch: Option<&'a StylePatch>,
    parts: &'a [(Part, StylePatch)],
    ov: PartStyle<'a>,
}

impl fmt::Debug for StatusBar<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StatusBar")
            .field("id", &self.id)
            .field("left", &self.left.len())
            .field("center", &self.center.len())
            .field("right", &self.right.len())
            .field("status", &self.status)
            .finish_non_exhaustive()
    }
}

type Keep = [u32; 3];

fn all_alive(n: usize) -> u32 {
    let n = u32::try_from(n).unwrap_or(u32::BITS);

    u32::MAX
        .checked_shr(u32::BITS.saturating_sub(n))
        .unwrap_or(0)
}

impl<'a> StatusBar<'a> {
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::LABEL,
        Part::MARKER,
        Part::ICON,
        Part::TRACK,
        Part::THUMB,
        Part::OVERFLOW,
    ];

    pub const fn new(id: Id) -> Self {
        StatusBar {
            id,
            left: &[],
            center: &[],
            right: &[],
            variant: Variant::DEFAULT,
            status: Status::Ready,
            frame: 0,
            patch: None,
            parts: &[],
            ov: PartStyle::new(),
        }
    }

    pub const fn id(&self) -> Id {
        self.id
    }

    #[must_use]
    pub const fn left(mut self, items: &'a [StatusItem<'a>]) -> Self {
        self.left = items;
        self
    }

    #[must_use]
    pub const fn center(mut self, items: &'a [StatusItem<'a>]) -> Self {
        self.center = items;
        self
    }

    #[must_use]
    pub const fn right(mut self, items: &'a [StatusItem<'a>]) -> Self {
        self.right = items;
        self
    }

    #[must_use]
    pub const fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }

    #[must_use]
    pub const fn status(mut self, s: Status) -> Self {
        self.status = s;
        self
    }

    #[must_use]
    pub const fn frame(mut self, f: usize) -> Self {
        self.frame = f;
        self
    }

    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.patch = Some(p);
        self.ov = self.ov.global(p);
        self
    }

    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.parts = ps;
        self.ov = self.ov.part(ps);
        self
    }

    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    fn group(&self, g: Group) -> &'a [StatusItem<'a>] {
        let items = match g {
            Group::Left => self.left,
            Group::Center => self.center,
            Group::Right => self.right,
        };
        items.get(..items.len().min(MAX_ITEMS)).unwrap_or(items)
    }

    const fn busy(&self) -> bool {
        matches!(self.status, Status::Busy | Status::Loading)
    }

    fn meter_columns(ui: &Ui<'_>) -> u16 {
        ui.design()
            .size
            .meter_track
            .saturating_add(PCT_COLUMNS)
            .saturating_add(2)
    }

    fn marker_glyph(&self, ui: &Ui<'_>, live: StateFlags) -> Slot<GlyphRole> {
        let base = ui
            .resolve(Family::STATUSBAR, self.variant, Part::MARKER, live)
            .glyph;
        self.ov
            .part_patch(Part::MARKER)
            .map_or(base, |p| p.glyph.over(base))
    }

    fn readiness(&self, ui: &Ui<'_>, live: StateFlags) -> Option<&'static str> {
        if self.busy() {
            return spinner_glyph(ui.design().motion.spinner_frames, self.frame);
        }
        if live.contains(StateFlags::ERROR) {
            let g = match self.marker_glyph(ui, live) {
                Slot::Set(g) => g,
                Slot::Inherit => GlyphRole::Error,
                Slot::Clear => return None,
            };
            return Some(ui.glyph_str(g));
        }
        if live.contains(StateFlags::WARNING) {
            let g = match self.marker_glyph(ui, live) {
                Slot::Set(g) => g,
                Slot::Inherit => GlyphRole::Dirty,
                Slot::Clear => return None,
            };
            return Some(ui.glyph_str(g));
        }
        None
    }

    fn group_columns(&self, g: Group, keep: Keep, metrics: ItemMetrics, gap: u16) -> u16 {
        let items = self.group(g);
        let mask = keep.get(g.index()).copied().unwrap_or(0);
        let mut w = 0u16;
        let mut n = 0u16;
        for (i, it) in items.iter().enumerate() {
            if mask & (1 << i) != 0 {
                w = w.saturating_add(it.columns(metrics));
                n = n.saturating_add(1);
            }
        }
        if n > 0 {
            w.saturating_add(n.saturating_sub(1).saturating_mul(gap))
        } else {
            0
        }
    }

    fn all_alive_keep(&self) -> Keep {
        Group::ALL.map(|g| all_alive(self.group(g).len()))
    }

    fn needed(&self, keep: Keep, metrics: ItemMetrics, gap: u16, edge: u16, lead: u16) -> u16 {
        let ws = Group::ALL.map(|g| self.group_columns(g, keep, metrics, gap));
        let present = ws.iter().filter(|w| **w > 0).count().min(3) as u16;
        ws.iter()
            .fold(0u16, |a, w| a.saturating_add(*w))
            .saturating_add(present.saturating_sub(1).saturating_mul(gap))
            .saturating_add(edge.saturating_mul(2))
            .saturating_add(lead)
    }

    fn survivors(&self, total: u16, metrics: ItemMetrics, gap: u16, edge: u16, lead: u16) -> Keep {
        let mut keep: Keep = self.all_alive_keep();
        while self.needed(keep, metrics, gap, edge, lead) > total {
            let mut victim: Option<(u8, usize, usize)> = None;
            for g in [Group::Center, Group::Right, Group::Left] {
                let gi = g.index();
                let items = self.group(g);
                let mask = keep.get(gi).copied().unwrap_or(0);
                let alive = mask.count_ones();
                if alive == 0 || (g == Group::Left && alive == 1) {
                    continue;
                }

                let mut worst: Option<(u8, usize)> = None;
                for (i, it) in items.iter().enumerate() {
                    if mask & (1 << i) == 0 {
                        continue;
                    }
                    if worst.is_none_or(|(p, _)| it.priority <= p) {
                        worst = Some((it.priority, i));
                    }
                }
                if let Some((p, i)) = worst
                    && victim.is_none_or(|(vp, _, _)| p < vp)
                {
                    victim = Some((p, gi, i));
                }
            }
            match victim {
                Some((_, gi, i)) => {
                    if let Some(m) = keep.get_mut(gi) {
                        *m &= !(1u32 << i);
                    }
                }
                None => break,
            }
        }
        keep
    }

    pub fn update(&self, cx: &mut Cx<'_>) -> Response<StatusAction> {
        let mut r: Response<StatusAction> = Response::ignored();
        for it in cx.intents(self.id) {
            match it {
                Intent::Pointer {
                    phase: Phase::Click | Phase::DoubleClick,
                    part,
                    ..
                } => {
                    if let Some(k) = part.item {
                        r = Response::action(StatusAction::Chose(k));
                    }
                }

                Intent::Pointer {
                    phase: Phase::Move, ..
                } => {}
                Intent::Pointer { .. } if r.action_ref().is_none() => r = Response::changed(),
                _ => {}
            }
        }
        r.for_id(self.id)
    }

    fn item_style(&self, ui: &mut Ui<'_>, it: &StatusItem<'_>, live: StateFlags) -> PaintStyle {
        let hovered = it.key.is_some_and(|key| {
            FrameRead::hovered_part(ui, self.id) == Some(PartRef::item(Part::LABEL, key))
        });
        let live = if hovered {
            live
        } else {
            live.difference(StateFlags::HOVERED)
        };
        let base = self.ov.style(
            ui,
            self.id,
            Family::STATUSBAR,
            self.variant,
            Part::LABEL,
            live,
        );
        let mut delta = StylePatch::new();
        match it.emphasis {
            Emphasis::Strong => delta = delta.add(Modifier::BOLD),
            Emphasis::Chip => delta = delta.set_bg(Role::RaisedSurface),
            Emphasis::Plain => {}
        }
        if let Some(r) = it.tone {
            delta = delta.set_fg(r);
        }
        if delta.is_empty() {
            return base.style;
        }
        let top = crate::theme::resolve::bind(ui.theme_ref(), delta, None, ui.surface()).style;
        base.style.patch(top)
    }

    fn paint_item(
        &self,
        ui: &mut Ui<'_>,
        it: &StatusItem<'_>,
        cell: Rect,
        live: StateFlags,
        metrics: ItemMetrics,
    ) -> u16 {
        if cell.is_empty() {
            return 0;
        }
        let label_w = it.label_columns(metrics).min(cell.width);
        let label = Rect {
            width: label_w,
            ..cell
        };
        if let Some(f) = self.ov.slot_for(Part::LABEL) {
            f(ui, label);
        } else {
            let style = self.item_style(ui, it, live);
            let text = if it.emphasis == Emphasis::Chip {
                ui.fill(label, style);
                shift(label, 1)
            } else {
                label
            };
            let glyph = it.spinner_glyph(metrics);
            ui.paint_str(text, glyph, style);
            ui.paint_str(shift(text, it.spinner_columns(metrics)), it.text, style);
        }
        if it.label_columns(metrics) > cell.width && cell.width > 0 {
            let last = Rect {
                x: cell.right().saturating_sub(1),
                width: 1,
                ..cell
            };
            if let Some(f) = self.ov.slot_for(Part::OVERFLOW) {
                f(ui, last);
            } else {
                let s = self.ov.style(
                    ui,
                    self.id,
                    Family::STATUSBAR,
                    self.variant,
                    Part::OVERFLOW,
                    live,
                );
                match s.glyph {
                    Slot::Set(g) => {
                        ui.glyph(last, g, s.style);
                    }
                    Slot::Inherit => {
                        ui.glyph(last, GlyphRole::Ellipsis, s.style);
                    }
                    Slot::Clear => {
                        ui.fill(last, s.style);
                    }
                }
            }
        }
        if let Some(k) = it.key
            && !ui.is_inert()
        {
            ui.register_part(self.id, PartRef::item(Part::LABEL, k), label);
        }
        let mut used = label_w;
        if it.ratio.is_some() {
            let m = Rect {
                x: cell.x.saturating_add(label_w).saturating_add(1),
                width: cell
                    .width
                    .saturating_sub(label_w)
                    .saturating_sub(1)
                    .min(metrics.meter_columns),
                ..cell
            };
            if !m.is_empty() {
                let mut meter = Meter::new(self.id).value("").patch_part(self.parts);
                if let Some(p) = self.patch {
                    meter = meter.patch(p);
                }
                if let Some(f) = self.ov.slot_for(Part::TRACK) {
                    meter = meter.slot(Part::TRACK, f);
                }
                if let Some(r) = it.ratio {
                    meter = meter.ratio(r);
                }
                if let Some(t) = it.meter_tone {
                    meter = meter.tone(t);
                }
                meter.draw(ui, m);
                used = used.saturating_add(1).saturating_add(m.width);
            }
        }
        used
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one pass over the readiness affordance and the three groups"
    )]
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        let area = first_row(area);
        if area.is_empty() {
            return area;
        }
        let live = PartStyle::flags(ui.state(self.id), self.status.flags());
        let ov = self.ov;
        let id = self.id;
        let d = ui.design();
        let gap = d.space.gap.max(1);
        let edge = d.space.gutter.max(1);
        let metrics = ItemMetrics {
            meter_columns: Self::meter_columns(ui),
            spinner_frames: ui.design().motion.spinner_frames,
        };
        let container = ov.style(
            ui,
            id,
            Family::STATUSBAR,
            self.variant,
            Part::CONTAINER,
            live,
        );
        ui.fill(area, container.style);

        let ready = self.readiness(ui, live);
        let lead = ready.map_or(0, |g| width(g).saturating_add(1));
        if let Some(g) = ready {
            let part = if self.busy() {
                Part::ICON
            } else {
                Part::MARKER
            };
            let cell = Rect {
                x: area.x.saturating_add(edge),
                width: area.width.saturating_sub(edge),
                ..area
            };
            if let Some(f) = ov.slot_for(part) {
                f(ui, cell);
            } else {
                let s = ov.style(ui, id, Family::STATUSBAR, self.variant, part, live);
                ui.paint_str(cell, g, s.style);
            }
        }

        let keep = self.survivors(area.width, metrics, gap, edge, lead);
        let inner_left = area.x.saturating_add(edge).saturating_add(lead);
        let inner_right = area.right().saturating_sub(edge);

        let mut x = inner_left;
        let left_mask = keep.first().copied().unwrap_or(0);
        for (i, it) in self.group(Group::Left).iter().enumerate() {
            if left_mask & (1 << i) == 0 {
                continue;
            }
            let room = inner_right.saturating_sub(x);
            if room == 0 {
                break;
            }
            let cell = Rect {
                x,
                width: it.columns(metrics).min(room),
                ..area
            };
            let used = self.paint_item(ui, it, cell, live, metrics);
            x = x.saturating_add(used).saturating_add(gap);
        }
        let left_end = x.saturating_sub(gap);

        let right_mask = keep.get(2).copied().unwrap_or(0);
        let mut rx = inner_right;
        let right_items = self.group(Group::Right);
        for (i, it) in right_items.iter().enumerate().rev() {
            if right_mask & (1 << i) == 0 {
                continue;
            }
            let w = it.columns(metrics);
            if rx.saturating_sub(w) <= left_end {
                break;
            }
            rx = rx.saturating_sub(w);
            let cell = Rect {
                x: rx,
                width: w,
                ..area
            };
            self.paint_item(ui, it, cell, live, metrics);
            rx = rx.saturating_sub(gap);
        }
        let right_start = if right_mask == 0 {
            inner_right
        } else {
            rx.saturating_add(gap)
        };

        let cw = self.group_columns(Group::Center, keep, metrics, gap);
        if cw > 0 {
            let lo = left_end.saturating_add(gap);
            let hi = right_start.saturating_sub(gap);
            let free = hi.saturating_sub(lo);
            let mut cx = lo.saturating_add(free.saturating_sub(cw) / 2);
            let center_mask = keep.get(1).copied().unwrap_or(0);
            for (i, it) in self.group(Group::Center).iter().enumerate() {
                if center_mask & (1 << i) == 0 {
                    continue;
                }
                let w = it.columns(metrics);
                let cell = Rect {
                    x: cx,
                    width: hi.saturating_sub(cx).min(w),
                    ..area
                };
                self.paint_item(ui, it, cell, live, metrics);
                cx = cx.saturating_add(w).saturating_add(gap);
            }
        }
        area
    }

    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size {
        let metrics = ItemMetrics {
            meter_columns: Self::meter_columns(ui),
            spinner_frames: ui.design().motion.spinner_frames,
        };
        let gap = ui.design().space.gap.max(1);
        let edge = ui.design().space.gutter.max(1);
        let full = self.all_alive_keep();
        let preferred = self.needed(full, metrics, gap, edge, 0);
        let strongest = self
            .group(Group::Left)
            .iter()
            .map(|it| it.columns(metrics))
            .next()
            .unwrap_or(0);
        Size {
            min: (strongest.saturating_add(edge.saturating_mul(2)), 1),
            preferred: (preferred, 1),
        }
        .fit(c)
    }
}

#[cfg(test)]
mod tests {
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::Position;

    use super::*;
    use crate::event::MouseKind;
    use crate::response::Invalidate;
    use crate::runtime::stub::{Stub, mouse};
    use crate::runtime::{App, Runtime};
    use crate::theme::Theme;

    const LEFT: [StatusItem<'static>; 2] = [
        StatusItem::new("payments-platform").strong().priority(9),
        StatusItem::new("PR #482 settlement backoff").priority(7),
    ];
    const CENTER: [StatusItem<'static>; 1] = [StatusItem::new("agent working").priority(4)];
    const RIGHT: [StatusItem<'static>; 3] = [
        StatusItem::new("Weekly 59%").chip().priority(6),
        StatusItem::new("run-7f3a").chip().priority(3),
        StatusItem::new("run 9c41").priority(2),
    ];

    const TEST_METRICS: ItemMetrics = ItemMetrics {
        meter_columns: 16,
        spinner_frames: &[],
    };

    fn bar() -> StatusBar<'static> {
        StatusBar::new(Id::root("t"))
            .left(&LEFT)
            .center(&CENTER)
            .right(&RIGHT)
    }

    #[test]
    fn a_wide_row_keeps_every_item() {
        let keep = bar().survivors(200, TEST_METRICS, 2, 1, 0);
        assert_eq!(keep, [0b11, 0b1, 0b111]);
    }

    #[test]
    fn narrow_rows_drop_centre_then_right_then_left_and_keep_the_name() {
        let keep = bar().survivors(60, TEST_METRICS, 2, 1, 0);
        assert_eq!(keep.get(1).copied(), Some(0), "the centre leaves first");
        assert_ne!(keep.first().copied(), Some(0), "identity stays");

        let keep = bar().survivors(10, TEST_METRICS, 2, 1, 0);
        assert_eq!(keep.get(1).copied(), Some(0));
        assert_eq!(keep.get(2).copied(), Some(0));
        assert_eq!(
            keep.first().copied().map(u32::count_ones),
            Some(1),
            "exactly the strongest left item survives"
        );
        assert_eq!(keep.first().copied(), Some(0b1));
    }

    #[test]
    fn ties_take_the_later_item_first() {
        const TIED: [StatusItem<'static>; 3] = [
            StatusItem::new("aaaaaaaa").priority(5),
            StatusItem::new("bbbbbbbb").priority(5),
            StatusItem::new("cccccccc").priority(5),
        ];
        let bar = StatusBar::new(Id::root("t")).right(&TIED);
        let keep = bar.survivors(22, TEST_METRICS, 2, 1, 0);
        assert_eq!(keep.get(2).copied(), Some(0b011), "the last item leaves");
    }

    #[test]
    fn items_past_the_cap_are_ignored() {
        const MANY: [StatusItem<'static>; 20] = [StatusItem::new("x"); 20];
        let bar = StatusBar::new(Id::root("t")).left(&MANY);
        assert_eq!(bar.group(Group::Left).len(), MAX_ITEMS);
    }

    #[test]
    fn an_item_reports_its_own_columns() {
        let plain = StatusItem::new("abc");
        assert_eq!(plain.columns(TEST_METRICS), 3);
        assert_eq!(StatusItem::new("abc").chip().columns(TEST_METRICS), 5);
        assert_eq!(
            StatusItem::new("abc").meter(0.5).columns(TEST_METRICS),
            3 + 1 + 16
        );
    }

    #[test]
    fn a_truncated_item_paints_the_overflow_glyph() {
        const ROW: Rect = Rect::new(0, 0, 8, 1);
        const ITEMS: [StatusItem<'static>; 1] =
            [StatusItem::new("identity-that-does-not-fit").strong()];
        let theme = Theme::junie();
        let overflow = theme.design.glyphs.get(GlyphRole::Ellipsis);
        let mut runtime = Runtime::new(Stub::default(), theme);
        let mut buffer = Buffer::empty(ROW);

        runtime
            .draw_scene(ROW, &mut buffer, |ui, area| {
                StatusBar::new(Id::root("status.overflow"))
                    .left(&ITEMS)
                    .draw(ui, area);
            })
            .commit_presented();

        assert!(
            painted_row(&buffer, ROW.width).contains(overflow),
            "a retained item clipped by the row must paint the overflow glyph"
        );
    }

    const CLICK_ID: Id = Id::root("status.click");
    const CLICK_KEY: ItemKey = ItemKey::num(7);
    const CLICK_ITEMS: [StatusItem<'static>; 1] = [StatusItem::new("Open").key(CLICK_KEY)];

    #[derive(Default)]
    struct ClickApp {
        action: Option<StatusAction>,
    }

    impl App for ClickApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let response = StatusBar::new(CLICK_ID).left(&CLICK_ITEMS).update(cx);
            self.action = response.action_ref().copied().or(self.action);
            response.erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            StatusBar::new(CLICK_ID)
                .left(&CLICK_ITEMS)
                .draw(ui, Rect::new(0, 0, 20, 1));
        }
    }

    #[test]
    fn a_keyed_item_click_reports_chose() {
        const SCREEN: Rect = Rect::new(0, 0, 20, 1);
        let mut runtime = Runtime::new(ClickApp::default(), Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let item = runtime
            .area_of_part(CLICK_ID, PartRef::item(Part::LABEL, CLICK_KEY))
            .unwrap_or(Rect::ZERO);
        assert!(
            !item.is_empty(),
            "the keyed item must register a hit region"
        );
        let x = item.x.saturating_add(item.width / 2);

        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Down, x, item.y));
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Up, x, item.y));

        assert_eq!(runtime.app().action, Some(StatusAction::Chose(CLICK_KEY)));
    }

    const HOVER_ID: Id = Id::root("status.hover");
    const FIRST: ItemKey = ItemKey::num(1);
    const SECOND: ItemKey = ItemKey::num(2);
    const HOVER_ITEMS: [StatusItem<'static>; 2] = [
        StatusItem::new("alpha").key(FIRST),
        StatusItem::new("bravo").key(SECOND),
    ];
    const HOVER_ROW: Rect = Rect::new(0, 0, 40, 1);

    struct HoverApp;

    impl App for HoverApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            StatusBar::new(HOVER_ID)
                .left(&HOVER_ITEMS)
                .update(cx)
                .erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            StatusBar::new(HOVER_ID)
                .left(&HOVER_ITEMS)
                .draw(ui, HOVER_ROW);
        }
    }

    #[test]
    fn a_pointer_move_without_a_hover_transition_does_not_repaint() {
        let mut runtime = Runtime::new(HoverApp, Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(HOVER_ROW);
        runtime
            .draw_buffer(HOVER_ROW, &mut buffer)
            .commit_presented();
        let area = |runtime: &Runtime<HoverApp>, k: ItemKey| {
            runtime
                .area_of_part(HOVER_ID, PartRef::item(Part::LABEL, k))
                .unwrap_or(Rect::ZERO)
        };
        let first = area(&runtime, FIRST);
        let second = area(&runtime, SECOND);
        assert!(first.width >= 2 && !second.is_empty());

        let entered =
            crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Move, first.x, first.y));
        runtime
            .draw_buffer(HOVER_ROW, &mut buffer)
            .commit_presented();
        let stayed = crate::runtime::stub::deliver(
            &mut runtime,
            mouse(MouseKind::Move, first.x.saturating_add(1), first.y),
        );
        runtime
            .draw_buffer(HOVER_ROW, &mut buffer)
            .commit_presented();
        let crossed =
            crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Move, second.x, second.y));
        runtime
            .draw_buffer(HOVER_ROW, &mut buffer)
            .commit_presented();

        assert_eq!(entered.invalidate(), Invalidate::Paint);
        assert_eq!(
            stayed.invalidate(),
            Invalidate::None,
            "moving inside one item repainted the whole strip"
        );
        assert_eq!(crossed.invalidate(), Invalidate::Paint);
    }

    fn painted_row(buf: &Buffer, w: u16) -> String {
        let mut row = String::new();
        for x in 0..w {
            if let Some(cell) = buf.cell(Position::new(x, 0)) {
                row.push_str(cell.symbol());
            }
        }
        row
    }

    #[test]
    fn the_left_group_starts_at_the_gutter_the_right_is_flush_and_the_centre_sits_between() {
        const ROW: Rect = Rect {
            x: 0,
            y: 0,
            width: 60,
            height: 1,
        };
        const L: [StatusItem<'static>; 2] = [
            StatusItem::new("alpha").priority(9),
            StatusItem::new("bravo").priority(8),
        ];
        const C: [StatusItem<'static>; 1] = [StatusItem::new("charlie").priority(7)];
        const R: [StatusItem<'static>; 2] = [
            StatusItem::new("delta").priority(6),
            StatusItem::new("echo").priority(5),
        ];

        let theme = Theme::junie();
        let edge = theme.design.space.gutter.max(1);
        let bar = StatusBar::new(Id::root("status.geometry"))
            .left(&L)
            .center(&C)
            .right(&R);
        let mut rt = Runtime::new(Stub::default(), theme);
        let mut buf = Buffer::empty(ROW);
        rt.draw_scene(ROW, &mut buf, |ui, area| {
            bar.draw(ui, area);
        })
        .commit_presented();
        let row = painted_row(&buf, ROW.width);

        let at = |needle: &str| -> u16 {
            let found = row.find(needle);
            assert!(found.is_some(), "{needle:?} is not painted in {row:?}");
            found
                .and_then(|i| u16::try_from(i).ok())
                .unwrap_or_default()
        };
        let end = |needle: &str| -> u16 { at(needle).saturating_add(width(needle)) };

        assert_eq!(
            at("alpha"),
            ROW.x.saturating_add(edge),
            "the left group starts at the gutter, in {row:?}"
        );
        assert!(
            end("alpha") < at("bravo"),
            "left items run left-to-right in declaration order, in {row:?}"
        );

        assert_eq!(
            end("echo"),
            ROW.right().saturating_sub(edge),
            "the right group is flush against the trailing gutter, in {row:?}"
        );
        assert!(
            end("delta") < at("echo"),
            "right items run left-to-right in declaration order, in {row:?}"
        );

        assert!(
            end("bravo") < at("charlie") && end("charlie") < at("delta"),
            "the centre group sits strictly between left and right, in {row:?}"
        );
        let before = at("charlie").saturating_sub(end("bravo"));
        let after = at("delta").saturating_sub(end("charlie"));
        assert!(
            before.abs_diff(after) <= 1,
            "the centre group is centred in the free span: {before} before, {after} after, in {row:?}"
        );
    }
}
