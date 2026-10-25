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
