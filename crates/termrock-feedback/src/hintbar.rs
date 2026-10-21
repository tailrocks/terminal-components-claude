use core::fmt;

use ratatui_core::layout::Rect;

use super::{PartStyle, SlotFn, first_row, shift};
use crate::id::{Id, Part};
use crate::keyhint::KeyHint;
use crate::keymap::{HintKey, HintLayer};
use crate::measure::{Constraints, Size};
use crate::response::StateFlags;
use crate::text::width;
use crate::theme::{ColorLevel, Family, GlyphRole, Slot, StateRule, StylePatch, Theme, Variant};
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
    variant: Variant,
    status: Status,

    #[cfg(test)]
    flags: StateFlags,
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
            variant: Variant::DEFAULT,
            status: Status::Ready,
            #[cfg(test)]
            flags: StateFlags::empty(),
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
        #[cfg(test)]
        {
            self.status.flags().union(self.flags)
        }
        #[cfg(not(test))]
        {
            self.status.flags()
        }
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

    /// Baseline pins Esc, `?`, and F1. Other hints drop from the right
    /// first. A full row keeps every hint. A cut row pays one gap for `…`.
    fn fitting(&self, budget: u16) -> (Vec<usize>, u16) {
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
        if badge_w > 0 && x.saturating_add(badge_w) <= right_limit {
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

        let budget = right_limit.saturating_sub(x);
        let (keep, used) = self.fitting(budget);
        if self.centered_override.unwrap_or(self.layer.centered) {
            let free = area.width.saturating_sub(used);
            let mid = area.x.saturating_add(free / 2);
            x = mid.max(x).min(right_limit.saturating_sub(used).max(x));
        }
        for &i in &keep {
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

    fn draw_layer(&self, ui: &mut Ui<'_>, area: Rect, layer: &HintLayer) -> Rect {
        let mut bar = HintBar::new(self.id, layer);
        bar.badge_override = self.badge;
        bar.status_text_override = self.status_text;
        bar.centered_override = self.centered;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{Chord, KeyCode};
    use crate::keymap::Hint;
    use crate::theme::Theme;
    use crate::ui::cx::LastFrame;
    use crate::ui::{FrameState, UiCore};
    use ratatui_core::buffer::Buffer;

    const AREA: Rect = Rect::new(0, 0, 40, 1);

    fn glyph(theme: &Theme, status: Status) -> Option<String> {
        let mut frame = FrameState::default();
        frame.reset(1, AREA);
        let mut page = Buffer::empty(AREA);
        let mut core = UiCore::default();
        let last = LastFrame::default();
        let layer = HintLayer {
            status: Some("Attention".into()),
            ..HintLayer::empty()
        };
        let ui = Ui::new(&mut frame, &mut page, &mut core, theme, &last);
        HintBar::new(Id::root("hintbar.warning"), &layer)
            .status(status)
            .status_glyph(&ui, status.flags())
            .map(str::to_owned)
    }

    fn layer(labels: &[(&'static str, KeyCode)]) -> HintLayer {
        HintLayer {
            hints: labels
                .iter()
                .map(|(l, c)| Hint {
                    key: crate::keymap::HintKey::Chord(Chord::key(*c)),
                    label: l,
                    priority: 50,
                })
                .collect(),
            ..HintLayer::empty()
        }
    }

    #[test]
    fn the_topmost_layer_wins_and_the_fallback_is_none() {
        let screen = layer(&[("Launch", KeyCode::Enter)]);
        let modal = layer(&[("Close", KeyCode::Esc)]);
        assert_eq!(
            HintBar::resolve(&[Some(&modal), None, Some(&screen)]).map(|l| l.hints.len()),
            Some(1)
        );

        assert!(core::ptr::eq(
            HintBar::resolve(&[Some(&modal), None, Some(&screen)]).unwrap_or(&screen),
            &raw const modal
        ));
        assert!(core::ptr::eq(
            HintBar::resolve(&[None, None, Some(&screen)]).unwrap_or(&modal),
            &raw const screen
        ));
        assert!(HintBar::resolve(&[None, None]).is_none());
    }

    #[test]
    fn narrow_rows_drop_hints_from_the_right() {
        let l = layer(&[
            ("Open", KeyCode::Enter),
            ("Choose", KeyCode::Char(' ')),
            ("Next", KeyCode::Tab),
            ("Cancel", KeyCode::Esc),
        ]);
        let bar = HintBar::new(Id::root("t"), &l);
        let (all, _) = bar.fitting(200);
        assert_eq!(all.len(), 4);
        let (few, used) = bar.fitting(24);
        assert!((1..4).contains(&few.len()), "{few:?}");
        assert!(used <= 24, "{used}");
        assert!(bar.fitting(0).0.is_empty());
    }

    #[test]
    fn pinned_escape_label_outlives_earlier_hints() {
        let l = HintLayer {
            hints: vec![
                Hint {
                    key: crate::keymap::HintKey::Label("Type"),
                    label: "Filter",
                    priority: 90,
                },
                Hint {
                    key: crate::keymap::HintKey::Label("Enter"),
                    label: "Choose",
                    priority: 70,
                },
                Hint {
                    key: crate::keymap::HintKey::Label("Esc"),
                    label: "Cancel",
                    priority: 60,
                },
            ],
            ..HintLayer::empty()
        };
        let bar = HintBar::new(Id::root("t"), &l);
        assert_eq!(bar.fitting(14).0, vec![2]);
    }

    #[test]
    fn inherited_warning_status_uses_warning_mark() {
        let theme = Theme::junie();
        assert_eq!(glyph(&theme, Status::Error).as_deref(), Some("!"));
        assert_eq!(
            glyph(&theme, Status::Ready).as_deref(),
            None,
            "ready status does not imply a glyph"
        );

        let mut frame = FrameState::default();
        frame.reset(1, AREA);
        let mut page = Buffer::empty(AREA);
        let mut core = UiCore::default();
        let last = LastFrame::default();
        let layer = HintLayer::empty();
        let ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);
        assert_eq!(
            HintBar::new(Id::root("hintbar.warning"), &layer)
                .status_glyph(&ui, StateFlags::WARNING)
                .map(str::to_owned)
                .as_deref(),
            Some("▲")
        );
    }

    fn with_warning(mut bar: HintBar<'_>) -> HintBar<'_> {
        bar.flags |= StateFlags::WARNING;
        bar
    }

    fn draw_page(theme: &Theme, screen: Rect, area: Rect, bar: &HintBar<'_>) -> Buffer {
        let mut frame = FrameState::default();
        frame.reset(1, screen);
        let mut page = Buffer::empty(screen);
        let untouched = page.clone();
        let mut core = UiCore::default();
        let last = LastFrame::default();
        {
            let mut ui = Ui::new(&mut frame, &mut page, &mut core, theme, &last);
            bar.draw(&mut ui, area);
        }
        assert_painted_inside(&untouched, &page, screen, area);
        page
    }

    fn assert_painted_inside(before: &Buffer, after: &Buffer, screen: Rect, area: Rect) {
        for y in 0..screen.height {
            for x in 0..screen.width {
                let inside = area.height > 0
                    && area.width > 0
                    && y == area.y
                    && x >= area.x
                    && x < area.right();
                if inside {
                    continue;
                }
                let left = &before[(x, y)];
                let right = &after[(x, y)];
                assert_eq!(
                    (left.symbol(), left.fg, left.bg, left.modifier),
                    (right.symbol(), right.fg, right.bg, right.modifier),
                    "painted outside the area at ({x}, {y})"
                );
            }
        }
    }

    fn row_text(page: &Buffer, area: Rect) -> String {
        let mut out = String::new();
        if area.height == 0 || area.width == 0 {
            return out;
        }
        for x in area.x..area.right() {
            out.push_str(page[(x, area.y)].symbol());
        }
        out
    }

    fn theme_with_marker(glyph: Slot<GlyphRole>) -> Theme {
        Theme::junie().define_family(Family::HINTBAR, |edit| {
            edit.part(Part::MARKER).base(StylePatch {
                glyph,
                ..StylePatch::new()
            });
        })
    }

    fn choose_layer(status: &'static str) -> HintLayer {
        HintLayer {
            hints: vec![Hint {
                key: crate::keymap::HintKey::Chord(Chord::key(KeyCode::Enter)),
                label: "Choose",
                priority: 50,
            }],
            status: Some(status.into()),
            centered: true,
            ..HintLayer::empty()
        }
    }

    #[test]
    fn mono_warning_status_keeps_the_warning_mark() {
        let theme = Theme::junie().downgrade(ColorLevel::Mono);
        let area = Rect::new(0, 0, 24, 1);
        let screen = Rect::new(0, 0, 24, 3);
        let layer = HintLayer {
            status: Some("Attention".into()),
            ..HintLayer::empty()
        };
        let mut frame = FrameState::default();
        frame.reset(1, screen);
        let mut page = Buffer::empty(screen);
        let mut core = UiCore::default();
        let last = LastFrame::default();
        {
            let ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);
            assert_eq!(
                ui.resolve(
                    Family::HINTBAR,
                    Variant::DEFAULT,
                    Part::MARKER,
                    StateFlags::WARNING
                )
                .glyph,
                Slot::Set(GlyphRole::Dirty),
                "mono fallback still rewrites the resolved marker"
            );
        }
        let painted = draw_page(
            &theme,
            screen,
            area,
            &with_warning(HintBar::new(Id::root("hintbar.warning"), &layer)),
        );
        let row = row_text(&painted, area);
        assert!(row.contains('▲'), "{row}");
        assert!(!row.contains('•'), "{row}");
    }

    #[test]
    fn a_status_wider_than_the_row_truncates_and_drops_hints() {
        let theme = Theme::junie();
        let area = Rect::new(0, 0, 72, 1);
        let screen = Rect::new(0, 0, 72, 2);
        let layer = choose_layer(
            "Could not confirm running instances: instance index unreadable · entered without the ritual",
        );
        let page = draw_page(
            &theme,
            screen,
            area,
            &with_warning(HintBar::new(Id::root("hintbar.warning"), &layer)),
        );
        assert_eq!(page[(1, 0)].symbol(), "▲");
        assert_eq!(page[(2, 0)].symbol(), " ");
        assert_eq!(page[(2, 0)].fg, theme.color.warning);
        assert_eq!(page[(70, 0)].symbol(), "…");
        assert_eq!(page[(27, 0)].fg, theme.color.warning);
        let row = row_text(&page, area);
        assert!(!row.contains("Choose"), "{row}");
    }

    #[test]
    fn narrow_widths_do_not_paint_outside_the_area() {
        let theme = Theme::junie();
        let layer = choose_layer(
            "Could not confirm running instances: instance index unreadable · entered without the ritual",
        );
        let screen = Rect::new(0, 0, 20, 6);
        for width in 0..=5 {
            let area = Rect::new(3, 2, width, 2);
            let page = draw_page(
                &theme,
                screen,
                area,
                &with_warning(HintBar::new(Id::root("hintbar.narrow"), &layer)),
            );
            let row = row_text(&page, area);
            assert!(!row.contains("Choose"), "width {width}: {row}");
        }
    }

    #[test]
    fn a_unicode_wide_status_truncates_inside_a_shifted_row() {
        let theme = Theme::junie();
        let status = "警告：无法确认运行实例";
        let area = Rect::new(4, 2, 11, 3);
        let screen = Rect::new(0, 0, 24, 7);
        let layer = choose_layer(status);
        let page = draw_page(
            &theme,
            screen,
            area,
            &with_warning(HintBar::new(Id::root("hintbar.wide"), &layer)),
        );
        let expected = crate::text::truncate(&format!("▲ {status}"), area.width.saturating_sub(2));
        let w = width(&expected);
        assert!(w > 0, "{expected}");
        assert!(expected.contains('…'), "{expected}");
        let x0 = area.right().saturating_sub(w).saturating_sub(1);
        assert!(x0 >= area.x && x0 < area.right(), "{x0}");
        let mut col = x0;
        for (_, grapheme) in crate::text::measure::graphemes(&expected) {
            assert!(col < area.right(), "wide status spilled");
            assert_eq!(page[(col, area.y)].symbol(), grapheme, "at {col}");
            let columns = crate::text::measure::grapheme_width(grapheme);
            let mut step = 1u16;
            while step < columns {
                let next = col.saturating_add(step);
                assert!(next < area.right(), "wide glyph spilled at {next}");
                step = step.saturating_add(1);
            }
            col = col.saturating_add(columns);
        }
        assert!(col <= area.right());
        let row = row_text(&page, area);
        assert!(!row.contains("Choose"), "{row}");
        assert!(!row.contains('•'), "{row}");
    }

    #[test]
    fn truncated_status_respects_a_shifted_origin() {
        let theme = Theme::junie();
        let area = Rect::new(5, 3, 72, 2);
        let screen = Rect::new(0, 0, 90, 8);
        let layer = choose_layer(
            "Could not confirm running instances: instance index unreadable · entered without the ritual",
        );
        let page = draw_page(
            &theme,
            screen,
            area,
            &with_warning(HintBar::new(Id::root("hintbar.shift"), &layer)),
        );
        assert_eq!(page[(6, 3)].symbol(), "▲");
        assert_eq!(page[(7, 3)].fg, theme.color.warning);
        assert_eq!(page[(75, 3)].symbol(), "…");
        let row = row_text(&page, area);
        assert!(!row.contains("Choose"), "{row}");
    }

    #[test]
    fn custom_theme_marker_set_clear_and_inherit() {
        let screen = Rect::new(0, 0, 32, 4);
        let area = Rect::new(2, 1, 24, 1);
        let layer = HintLayer {
            status: Some("Attention".into()),
            ..HintLayer::empty()
        };
        let paint = |theme: &Theme, bar: HintBar<'_>| -> String {
            row_text(&draw_page(theme, screen, area, &bar), area)
        };
        let set = theme_with_marker(Slot::Set(GlyphRole::Error)).downgrade(ColorLevel::Mono);
        let clear = theme_with_marker(Slot::Clear).downgrade(ColorLevel::Mono);
        let inherit = theme_with_marker(Slot::Inherit).downgrade(ColorLevel::Mono);
        let mono = Theme::junie().downgrade(ColorLevel::Mono);
        let chosen = [(Part::MARKER, StylePatch::new().set_glyph(GlyphRole::Chosen))];
        let cleared = [(
            Part::MARKER,
            StylePatch {
                glyph: Slot::Clear,
                ..StylePatch::new()
            },
        )];
        let inherited = [(Part::MARKER, StylePatch::new())];

        let row = paint(
            &set,
            with_warning(HintBar::new(Id::root("hintbar.warning"), &layer)),
        );
        assert!(row.contains('!'), "theme Set: {row}");
        assert!(!row.contains('▲'), "theme Set: {row}");
        assert!(!row.contains('•'), "theme Set: {row}");

        let row = paint(
            &clear,
            with_warning(HintBar::new(Id::root("hintbar.warning"), &layer)),
        );
        assert!(row.contains("Attention"), "theme Clear: {row}");
        assert!(!row.contains('▲'), "theme Clear: {row}");
        assert!(!row.contains('•'), "theme Clear: {row}");
        assert!(!row.contains('!'), "theme Clear: {row}");

        let row = paint(
            &inherit,
            with_warning(HintBar::new(Id::root("hintbar.warning"), &layer)),
        );
        assert!(row.contains('▲'), "theme Inherit: {row}");
        assert!(!row.contains('•'), "theme Inherit: {row}");

        let row = paint(
            &mono,
            with_warning(HintBar::new(Id::root("hintbar.warning"), &layer).patch_part(&chosen)),
        );
        assert!(row.contains('›'), "instance Set: {row}");
        assert!(!row.contains('▲'), "instance Set: {row}");
        assert!(!row.contains('•'), "instance Set: {row}");

        let row = paint(
            &mono,
            with_warning(HintBar::new(Id::root("hintbar.warning"), &layer).patch_part(&cleared)),
        );
        assert!(row.contains("Attention"), "instance Clear: {row}");
        assert!(!row.contains('▲'), "instance Clear: {row}");
        assert!(!row.contains('•'), "instance Clear: {row}");
        assert!(!row.contains('›'), "instance Clear: {row}");

        let row = paint(
            &mono,
            with_warning(HintBar::new(Id::root("hintbar.warning"), &layer).patch_part(&inherited)),
        );
        assert!(row.contains('▲'), "instance Inherit: {row}");
        assert!(!row.contains('•'), "instance Inherit: {row}");

        let row = paint(
            &set,
            with_warning(HintBar::new(Id::root("hintbar.warning"), &layer).patch_part(&chosen)),
        );
        assert!(row.contains('›'), "instance Set beats theme Set: {row}");
        assert!(!row.contains('!'), "instance Set beats theme Set: {row}");
    }

    #[test]
    fn busy_status_keeps_the_spinner_ahead_of_a_warning() {
        let theme = Theme::junie().downgrade(ColorLevel::Mono);
        let spinner = theme
            .design
            .motion
            .spinner_frames
            .first()
            .copied()
            .unwrap_or("");
        let area = Rect::new(1, 1, 24, 1);
        let screen = Rect::new(0, 0, 28, 4);
        let layer = HintLayer {
            status: Some("Attention".into()),
            ..HintLayer::empty()
        };
        let mut bar = HintBar::new(Id::root("hintbar.warning"), &layer).status(Status::Busy);
        bar.flags |= StateFlags::WARNING;
        let page = draw_page(&theme, screen, area, &bar);
        let row = row_text(&page, area);
        assert!(row.contains(spinner), "{row}");
        assert!(!row.contains('▲'), "{row}");
        assert!(!row.contains('•'), "{row}");
    }
}
