use core::fmt;

use ratatui_core::layout::Rect;

use super::{PartStyle, SlotFn, first_row, shift};
use crate::id::{Id, Part};
use crate::keyhint::KeyHint;
use crate::keymap::{HintKey, HintLayer};
use crate::measure::{Constraints, Size};
use crate::response::StateFlags;
use crate::text::width;
use crate::theme::{
    ColorLevel, Family, FgStep, GlyphRole, Role, Slot, StateRule, StylePatch, Theme, Variant,
};
use crate::ui::{FrameRead, Ui};
use termrock_runtime::Status;

#[derive(Clone, Copy, Debug, Default)]
enum MetadataOverride<'a> {
    #[default]
    Inherit,
    Set(Option<&'a str>),
}

impl<'a> MetadataOverride<'a> {
    fn resolve(self, inherited: Option<&'a str>) -> Option<&'a str> {
        match self {
            Self::Inherit => inherited,
            Self::Set(value) => value,
        }
    }
}

pub struct HintBar<'a> {
    id: Id,
    layer: &'a HintLayer,
    badge_override: MetadataOverride<'a>,
    status_text_override: MetadataOverride<'a>,
    centered_override: Option<bool>,
    peek: Option<&'a str>,
    variant: Variant,
    status: Status,

    frame: usize,
    patch: Option<&'a StylePatch>,
    parts: &'a [(Part, StylePatch)],
    ov: PartStyle<'a>,
}

fn pinned_way_out(key: HintKey) -> bool {
    matches!(key, HintKey::Label("Esc" | "?" | "F1"))
}

fn hint_cost(widths: &[u16], keep: &[usize], cut: bool) -> u16 {
    if keep.is_empty() {
        return 0;
    }
    let mut cost = 0u16;
    for (n, &index) in keep.iter().enumerate() {
        cost = cost.saturating_add(widths.get(index).copied().unwrap_or(0));
        if n.saturating_add(1) < keep.len() || cut {
            cost = cost.saturating_add(HintBar::HINT_GAP);
        }
    }
    cost
}

fn marker_states(part: Option<&crate::theme::PartRecipe>) -> &[StateRule] {
    match part {
        Some(part) => part.states.as_slice(),
        None => &[],
    }
}

impl fmt::Debug for HintBar<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HintBar")
            .field("id", &self.id)
            .field("hints", &self.layer.hints.len())
            .field("badge", &self.layer.badge)
            .field("status", &self.status)
            .finish_non_exhaustive()
    }
}

impl<'a> HintBar<'a> {
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::BADGE,
        Part::KEY,
        Part::ACTION,
        Part::LABEL,
        Part::MARKER,
        Part::ICON,
        Part::OVERFLOW,
    ];

    const HINT_GAP: u16 = 2;

    pub const fn new(id: Id, layer: &'a HintLayer) -> Self {
        HintBar {
            id,
            layer,
            badge_override: MetadataOverride::Inherit,
            status_text_override: MetadataOverride::Inherit,
            centered_override: None,
            peek: None,
            variant: Variant::DEFAULT,
            status: Status::Ready,
            frame: 0,
            patch: None,
            parts: &[],
            ov: PartStyle::new(),
        }
    }

    pub const fn derived(id: Id) -> DerivedHintBar<'a> {
        DerivedHintBar {
            id,
            top: None,
            mode: None,
            screen: None,
            global: None,
            badge: MetadataOverride::Inherit,
            status_text: MetadataOverride::Inherit,
            centered: None,
            peek: None,
        }
    }

    #[must_use]
    pub fn resolve<'l>(layers: &[Option<&'l HintLayer>]) -> Option<&'l HintLayer> {
        layers.iter().flatten().copied().next()
    }

    pub const fn id(&self) -> Id {
        self.id
    }

    pub const fn layer(&self) -> &'a HintLayer {
        self.layer
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
    pub const fn peek(mut self, text: Option<&'a str>) -> Self {
        self.peek = text;
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

    const fn busy(&self) -> bool {
        matches!(self.status, Status::Busy | Status::Loading)
    }

    fn derived_flags(&self) -> StateFlags {
        { self.status.flags() }
    }

    fn hint(&self, i: usize) -> Option<KeyHint<'a>> {
        let h = self.layer.hints.get(i)?;
        let mut k = KeyHint::from_hint(self.id, h)
            .variant(self.variant)
            .patch_part(self.parts);
        if let Some(p) = self.patch {
            k = k.patch(p);
        }
        if let Some(f) = self.ov.slot_for(Part::KEY) {
            k = k.slot(Part::KEY, f);
        } else if let Some(f) = self.ov.slot_for(Part::ACTION) {
            k = k.slot(Part::ACTION, f);
        }
        Some(k)
    }

    fn badge_width(&self) -> u16 {
        self.badge_text()
            .filter(|b| !b.is_empty())
            .map_or(0, |b| width(b).saturating_add(2))
    }

    fn status_width(&self, ui: &Ui<'_>, live: StateFlags) -> u16 {
        let Some(s) = self.message().filter(|s| !s.is_empty()) else {
            return 0;
        };
        let glyph = self.status_glyph(ui, live);
        width(s).saturating_add(glyph.map_or(0, |g| width(g).saturating_add(1)))
    }

    fn badge_text(&self) -> Option<&str> {
        self.badge_override.resolve(self.layer.badge)
    }

    fn message(&self) -> Option<&str> {
        self.status_text_override
            .resolve(self.layer.status.as_deref())
    }

    fn marker_glyph(&self, ui: &Ui<'_>, live: StateFlags) -> Slot<GlyphRole> {
        let base = ui
            .resolve(Family::HINTBAR, self.variant, Part::MARKER, live)
            .glyph;
        self.ov
            .part_patch(Part::MARKER)
            .map_or(base, |p| p.glyph.over(base))
    }

    fn status_glyph(&self, ui: &Ui<'_>, live: StateFlags) -> Option<&'static str> {
        if self.busy() {
            let frames = ui.design().motion.spinner_frames;
            return frames
                .get(self.frame.checked_rem(frames.len()).unwrap_or(0))
                .copied();
        }
        if live.contains(StateFlags::ERROR) {
            return match self.marker_glyph(ui, live) {
                Slot::Set(g) => Some(ui.glyph_str(g)),
                Slot::Inherit => Some(ui.glyph_str(GlyphRole::Error)),
                Slot::Clear => None,
            };
        }
        if live.contains(StateFlags::WARNING) {
            return self.warning_glyph(ui, live);
        }
        None
    }

    fn warning_glyph(&self, ui: &Ui<'_>, live: StateFlags) -> Option<&'static str> {
        if let Some(patch) = self.ov.part_patch(Part::MARKER) {
            match patch.glyph {
                Slot::Set(g) => return Some(ui.glyph_str(g)),
                Slot::Clear => return None,
                Slot::Inherit => {}
            }
        }
        let authored = self.theme_marker_slot(ui.theme_ref(), live);
        if authored.speaks() {
            return match authored {
                Slot::Set(g) => Some(ui.glyph_str(g)),
                Slot::Clear => None,
                Slot::Inherit => Some(ui.glyph_str(GlyphRole::WarningMark)),
            };
        }

        match self.marker_glyph(ui, live) {
            Slot::Set(GlyphRole::Dirty) if ui.theme_ref().capability.color == ColorLevel::Mono => {
                Some(ui.glyph_str(GlyphRole::WarningMark))
            }
            Slot::Set(g) => Some(ui.glyph_str(g)),
            Slot::Inherit => Some(ui.glyph_str(GlyphRole::WarningMark)),
            Slot::Clear => None,
        }
    }

    fn theme_marker_slot(&self, theme: &Theme, live: StateFlags) -> Slot<GlyphRole> {
        let recipes = &theme.recipes;
        let recipe = recipes.get_or_neutral(Family::HINTBAR);
        let variant = if self.variant == Variant::DEFAULT {
            recipe.default_variant
        } else {
            self.variant
        };
        let family = recipe.parts.get(Part::MARKER);
        let delta = recipe
            .variant(variant)
            .and_then(|parts| parts.get(Part::MARKER));
        let mut acc = StylePatch::new();
        if let Some(part) = family {
            acc = part.apply_base(acc);
        }
        if let Some(part) = delta {
            acc = part.apply_base(acc);
        }
        acc = crate::theme::recipe::merge_states(
            acc,
            marker_states(family),
            marker_states(delta),
            live,
        );
        for over in recipes.overrides() {
            if over.family != Family::HINTBAR {
                continue;
            }
            let applies = match over.variant {
                None => true,
                Some(candidate) => candidate == variant,
            };
            if applies && let Some(part) = over.parts.get(Part::MARKER) {
                acc = part.apply(acc, live);
            }
        }
        acc.glyph
    }

    fn fitting_stack(&self, budget: u16, keep_buf: &mut [usize; 32]) -> (usize, u16) {
        let n = self.layer.hints.len();
        if n == 0 {
            return (0, 0);
        }
        let mut widths = [0u16; 32];
        let mut pinned = [false; 32];
        for i in 0..n {
            widths[i] = self.hint(i).map(|h| h.width()).unwrap_or(0);
            pinned[i] = self
                .layer
                .hints
                .get(i)
                .is_some_and(|hint| pinned_way_out(hint.key));
            keep_buf[i] = i;
        }
        let mut keep_len = n;
        if hint_cost(&widths[..n], &keep_buf[..keep_len], false) > budget {
            let mut order = [0usize; 32];
            let mut order_len = 0;
            for i in (0..n).rev() {
                if !pinned[i] {
                    order[order_len] = i;
                    order_len += 1;
                }
            }
            for i in (0..n).rev() {
                if pinned[i] {
                    order[order_len] = i;
                    order_len += 1;
                }
            }
            for &gone in &order[..order_len] {
                if hint_cost(&widths[..n], &keep_buf[..keep_len], true) <= budget {
                    break;
                }
                let mut w = 0;
                for r in 0..keep_len {
                    if keep_buf[r] != gone {
                        keep_buf[w] = keep_buf[r];
                        w += 1;
                    }
                }
                keep_len = w;
            }
        }
        let cut = keep_len < n;
        let full = hint_cost(&widths[..n], &keep_buf[..keep_len], cut);
        let used = if cut {
            full.saturating_sub(Self::HINT_GAP)
        } else {
            full
        };
        (keep_len, used)
    }

    fn full_hint_width(&self) -> u16 {
        let count = self.layer.hints.len();
        (0..count).fold(0u16, |used, index| {
            used.saturating_add(self.hint(index).map_or(0, |hint| hint.width()))
                .saturating_add(if index.saturating_add(1) < count {
                    Self::HINT_GAP
                } else {
                    0
                })
        })
    }

    fn fitting_heap(&self, budget: u16) -> (Vec<usize>, u16) {
        let n = self.layer.hints.len();
        let widths: Vec<u16> = (0..n)
            .map(|i| self.hint(i).map(|h| h.width()).unwrap_or(0))
            .collect();
        let pinned: Vec<bool> = (0..n)
            .map(|i| {
                self.layer
                    .hints
                    .get(i)
                    .is_some_and(|hint| pinned_way_out(hint.key))
            })
            .collect();
        let mut keep: Vec<usize> = (0..n).collect();
        if hint_cost(&widths, &keep, false) > budget {
            let mut order: Vec<usize> = (0..n).rev().filter(|&i| !pinned[i]).collect();
            order.extend((0..n).rev().filter(|&i| pinned[i]));
            for gone in order {
                if hint_cost(&widths, &keep, true) <= budget {
                    break;
                }
                keep.retain(|&i| i != gone);
            }
        }
        let cut = keep.len() < n;
        let full = hint_cost(&widths, &keep, cut);
        let used = if cut {
            full.saturating_sub(Self::HINT_GAP)
        } else {
            full
        };
        (keep, used)
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one pass over the status, the badge, the hints and the cut marker"
    )]
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        let area = first_row(area);
        if area.is_empty() {
            return area;
        }

        let live = PartStyle::flags(StateFlags::empty(), self.derived_flags());
        let ov = self.ov;
        let id = self.id;
        let container = ov.style(ui, id, Family::HINTBAR, self.variant, Part::CONTAINER, live);
        ui.fill(area, container.style);

        let status_w = self.status_width(ui, live);
        let mut right_limit = area.right();
        if status_w > 0 && area.width > status_w.saturating_add(2) {
            let glyph = self.status_glyph(ui, live);
            let mut x = area.right().saturating_sub(status_w).saturating_sub(1);
            right_limit = x.saturating_sub(Self::HINT_GAP);
            if let Some(g) = glyph {
                let part = if self.busy() {
                    Part::ICON
                } else {
                    Part::MARKER
                };
                let cell = Rect {
                    x,
                    width: area.right().saturating_sub(x),
                    ..area
                };
                let used = if let Some(f) = ov.slot_for(part) {
                    f(ui, cell);
                    width(g).min(cell.width)
                } else {
                    let s = ov.style(ui, id, Family::HINTBAR, self.variant, part, live);
                    ui.paint_str(cell, g, s.style)
                };
                let gap_x = x.saturating_add(used);
                x = gap_x.saturating_add(1);
            }
            if let Some(text) = self.message() {
                let cell = Rect {
                    x,
                    width: area.right().saturating_sub(x),
                    ..area
                };
                if let Some(f) = ov.slot_for(Part::LABEL) {
                    f(ui, cell);
                } else {
                    let s = ov.style(ui, id, Family::HINTBAR, self.variant, Part::LABEL, live);
                    ui.paint_str(cell, text, s.style);
                }
            }
        } else if status_w > 0 {
            right_limit = self.paint_truncated_status(ui, area, live);
        }

        let mut x = area.x.saturating_add(1);
        let badge_w = self.badge_width();
        let badge_fits = badge_w == 0 || x.saturating_add(badge_w) <= right_limit;
        if badge_w > 0 && badge_fits {
            if let Some(b) = self.badge_text() {
                let cell = Rect {
                    x,
                    width: badge_w,
                    ..area
                };
                if let Some(f) = ov.slot_for(Part::BADGE) {
                    f(ui, cell);
                } else {
                    let s = ov.style(ui, id, Family::HINTBAR, self.variant, Part::BADGE, live);
                    ui.fill(cell, s.style);
                    ui.paint_str(shift(cell, 1), b, s.style);
                }
            }
            x = x.saturating_add(badge_w).saturating_add(Self::HINT_GAP);
        }

        let mut peek_text = None;
        let mut peek_area = None;
        if badge_fits
            && self.message().is_none()
            && let Some(text) = self.peek.filter(|text| !text.is_empty())
        {
            let full_hints = self.full_hint_width();
            let spare = right_limit
                .saturating_sub(x.saturating_add(full_hints))
                .saturating_sub(Self::HINT_GAP)
                .saturating_sub(1);
            if spare >= 8 {
                let clipped = crate::text::truncate(text, spare);
                let peek_width = width(&clipped).min(spare);
                if peek_width > 0 {
                    let peek_x = right_limit.saturating_sub(peek_width).saturating_sub(1);
                    let hint_limit = peek_x.saturating_sub(Self::HINT_GAP);
                    if x.saturating_add(full_hints) <= hint_limit {
                        right_limit = hint_limit;
                        peek_area = Some(Rect {
                            x: peek_x,
                            width: peek_width,
                            ..area
                        });
                        peek_text = Some(clipped);
                    }
                }
            }
        }

        let budget = right_limit.saturating_sub(x);
        let mut keep_buf = [0usize; 32];
        let heap_keep: Vec<usize>;
        let (keep, used): (&[usize], u16) = if self.layer.hints.len() <= 32 {
            let (len, used) = self.fitting_stack(budget, &mut keep_buf);
            (&keep_buf[..len], used)
        } else {
            let (vec, used) = self.fitting_heap(budget);
            heap_keep = vec;
            (&heap_keep[..], used)
        };
        if self.centered_override.unwrap_or(self.layer.centered) {
            let free = area.width.saturating_sub(used);
            let mid = area.x.saturating_add(free / 2);
            x = mid.max(x).min(right_limit.saturating_sub(used).max(x));
        }
        for &i in keep {
            let Some(h) = self.hint(i) else { break };
            let w = h.width();
            let cell = Rect {
                x,
                width: right_limit.saturating_sub(x).min(w),
                ..area
            };
            if cell.is_empty() {
                break;
            }
            h.draw(ui, cell);
            x = x.saturating_add(w).saturating_add(Self::HINT_GAP);
        }
        if keep.len() < self.layer.hints.len() && x < right_limit {
            let cell = Rect {
                x,
                width: right_limit.saturating_sub(x),
                ..area
            };
            if let Some(f) = ov.slot_for(Part::OVERFLOW) {
                f(ui, cell);
            } else {
                let s = ov.style(ui, id, Family::HINTBAR, self.variant, Part::OVERFLOW, live);
                match s.glyph {
                    Slot::Set(g) => {
                        ui.glyph(cell, g, s.style);
                    }
                    Slot::Inherit => {
                        ui.glyph(cell, GlyphRole::Ellipsis, s.style);
                    }
                    Slot::Clear => {
                        ui.fill(cell, s.style);
                    }
                }
            }
        }
        if let (Some(text), Some(cell)) = (peek_text.as_deref(), peek_area) {
            let label = ov.style(ui, id, Family::HINTBAR, self.variant, Part::LABEL, live);
            let secondary = ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Secondary)));
            ui.paint_str(cell, text, label.style.with_fg_from(secondary));
        }
        area
    }

    fn paint_truncated_status(&self, ui: &mut Ui<'_>, area: Rect, live: StateFlags) -> u16 {
        let glyph = self.status_glyph(ui, live);
        let run = match (glyph, self.message()) {
            (Some(g), Some(text)) => format!("{g} {text}"),
            (Some(g), None) => g.to_owned(),
            (None, Some(text)) => text.to_owned(),
            (None, None) => String::new(),
        };
        let text = crate::text::truncate(&run, area.width.saturating_sub(2));
        let w = width(&text);
        if w == 0 {
            return area.right();
        }
        let x = area.right().saturating_sub(w).saturating_sub(1);
        if x < area.x {
            return area.right();
        }
        let right_limit = x.saturating_sub(Self::HINT_GAP);
        let cell = Rect {
            x,
            width: area.right().saturating_sub(x),
            ..area
        };
        if cell.is_empty() {
            return area.right();
        }
        let ov = self.ov;
        let id = self.id;
        let label = ov.style(ui, id, Family::HINTBAR, self.variant, Part::LABEL, live);
        ui.paint_str(cell, &text, label.style);
        if let Some(g) = glyph.filter(|g| text.starts_with(g)) {
            let part = if self.busy() {
                Part::ICON
            } else {
                Part::MARKER
            };
            let mark = Rect {
                x,
                width: width(g).min(cell.width),
                ..area
            };
            if mark.is_empty() {
                return right_limit;
            }
            if let Some(f) = ov.slot_for(part) {
                f(ui, mark);
            } else {
                let marker = ov.style(ui, id, Family::HINTBAR, self.variant, part, live);
                ui.paint_str(mark, g, marker.style);
            }
        }
        right_limit
    }

    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size {
        let live = PartStyle::flags(StateFlags::empty(), self.derived_flags());
        let hints: u16 = (0..self.layer.hints.len())
            .filter_map(|i| self.hint(i))
            .fold(0u16, |acc, h| {
                acc.saturating_add(h.width()).saturating_add(Self::HINT_GAP)
            });
        let w = self
            .badge_width()
            .saturating_add(hints)
            .saturating_add(self.status_width(ui, live))
            .saturating_add(2);
        Size {
            min: (self.badge_width().saturating_add(2), 1),
            preferred: (w, 1),
        }
        .fit(c)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DerivedHintBar<'a> {
    id: Id,
    top: Option<&'a HintLayer>,
    mode: Option<&'a HintLayer>,
    screen: Option<&'a HintLayer>,
    global: Option<&'a HintLayer>,
    badge: MetadataOverride<'a>,
    status_text: MetadataOverride<'a>,
    centered: Option<bool>,
    peek: Option<&'a str>,
}

impl<'a> DerivedHintBar<'a> {
    #[must_use]
    pub const fn top(mut self, layer: &'a HintLayer) -> Self {
        self.top = Some(layer);
        self
    }

    #[must_use]
    pub const fn mode(mut self, layer: &'a HintLayer) -> Self {
        self.mode = Some(layer);
        self
    }

    #[must_use]
    pub const fn screen(mut self, layer: &'a HintLayer) -> Self {
        self.screen = Some(layer);
        self
    }

    #[must_use]
    pub const fn global(mut self, layer: &'a HintLayer) -> Self {
        self.global = Some(layer);
        self
    }

    #[must_use]
    pub const fn status_text(mut self, text: Option<&'a str>) -> Self {
        self.status_text = MetadataOverride::Set(text);
        self
    }

    #[must_use]
    pub const fn badge(mut self, text: Option<&'a str>) -> Self {
        self.badge = MetadataOverride::Set(text);
        self
    }

    #[must_use]
    pub const fn centered(mut self, centered: bool) -> Self {
        self.centered = Some(centered);
        self
    }

    #[must_use]
    pub const fn peek(mut self, text: Option<&'a str>) -> Self {
        self.peek = text;
        self
    }

    fn draw_layer(&self, ui: &mut Ui<'_>, area: Rect, layer: &HintLayer) -> Rect {
        let mut bar = HintBar::new(self.id, layer);
        bar.badge_override = self.badge;
        bar.status_text_override = self.status_text;
        bar.centered_override = self.centered;
        bar.peek = self.peek;
        bar.draw(ui, area)
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        if let Some(layer) = self.top.filter(|layer| !layer.is_empty()) {
            return self.draw_layer(ui, area, layer);
        }
        if let Some(layer) = self.mode.filter(|layer| !layer.is_empty()) {
            return self.draw_layer(ui, area, layer);
        }
        if let Some(rect) = ui
            .with_focused_hints(|ui, layer| {
                (!layer.is_empty()).then(|| self.draw_layer(ui, area, layer))
            })
            .flatten()
        {
            return rect;
        }
        if let Some(layer) = self.screen.filter(|layer| !layer.is_empty()) {
            return self.draw_layer(ui, area, layer);
        }
        if let Some(layer) = self.global.filter(|layer| !layer.is_empty()) {
            return self.draw_layer(ui, area, layer);
        }
        if self
            .badge
            .resolve(None)
            .is_some_and(|text| !text.is_empty())
            || self
                .status_text
                .resolve(None)
                .is_some_and(|text| !text.is_empty())
            || self.peek.is_some_and(|text| !text.is_empty())
        {
            return self.draw_layer(ui, area, &HintLayer::empty());
        }
        Rect {
            width: 0,
            height: 0,
            ..area
        }
    }
}
