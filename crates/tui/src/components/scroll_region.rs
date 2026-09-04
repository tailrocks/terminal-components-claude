//! `ScrollRegion` (`COMPONENT_ARCHITECTURE.md` §8.3, §12.2, Appendix A 4E).

use core::fmt;

use ratatui_core::layout::{Position, Rect};

use super::{Overrides, SlotFn};
use crate::hit::Axes;
use crate::id::{Id, Part, PartRef};
use crate::intent::{Intent, Phase};
use crate::measure::{Constraints, Size};
use crate::response::{Response, StateFlags};
use crate::scroll::ScrollState;
use crate::theme::{Family, GlyphRole, Slot, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, LayoutFacts, Ui};

/// A vertical scroll region: wheel routing, a scrollbar with track
/// arithmetic, thumb drag through pointer capture and `ensure_visible`.
///
/// ## Construction
/// `ScrollRegion::new(id)` — `id` is the **container's** id: the scrollbar
/// is `Part::TRACK` / `Part::THUMB` of its container, never a separate id.
///
/// ## Ownership
/// The caller owns a [`ScrollState`] (offset, content and viewport length);
/// the content length is per-phase data. The runtime owns the capture and
/// the wheel routing.
///
/// ## Configuration
/// `.patch`, `.patch_part`, `.slot`. The axis is vertical; `Axes::V` is
/// registered.
///
/// ## Variants
/// `Family::SCROLLBAR`, `DEFAULT` only.
///
/// ## States
/// The thumb wears `PRESSED` only when it is the pressed/captured part;
/// sibling track/container parts never inherit their owner's state.
///
/// ## Actions
/// `Response<()>`: `Consumed` without a repaint at a boundary, `Paint`
/// when the offset moved (the boundary-wheel rule, §8.3).
///
/// ## Focus
/// Never a focus stop. The container is registered as a `Decorative`
/// region so `area_of` answers; the track and thumb are `Part` regions.
///
/// ## Keyboard
/// None; keys belong to the container (`List`, `Tabs`, …).
///
/// ## Mouse
/// `PartRef::of(Part::TRACK)`: a press scrolls to that position.
/// `PartRef::of(Part::THUMB)`: a press claims capture; drags scroll to the
/// pointer's track position; release ends the capture. `Wheel` intents
/// over the content scroll by the delta.
///
/// ## Layout
/// `draw` returns the content rect: `area` minus one scrollbar column when
/// the content overflows the viewport, else `area` unchanged. `measure`
/// takes whatever the container offers (minimum `2 × 1`: the bar column plus
/// one content column). Degenerate rects register nothing (R5).
/// `viewport_len` / `content_len` are reported through `LayoutFacts` and
/// consumed by the next `update`.
///
/// ## Parts
/// `CONTAINER` (the whole region), `TRACK` (the bar column), `THUMB` (the
/// visible-range knob).
///
/// ## Overrides
/// `.patch`, `.patch_part`, `.slot(Part::TRACK | Part::THUMB, …)`.
///
/// ## Identity
/// The container's id; no items.
///
/// ## Testing
/// `ScrollRegionCase` with `SCROLLS | CAPTURES`; `render::components::list`
/// covers the bar inside a list.
///
/// ## Invariants
/// A wheel at a boundary is consumed without a repaint and never chains
/// outward; a wheel never moves focus or the cursor; `ensure_visible` is
/// requested only by cursor motion and applied by `update` from last
/// frame's viewport.
pub struct ScrollRegion<'a> {
    id: Id,
    family: Family,
    ov: Overrides<'a>,
}

#[derive(Clone, Copy)]
struct ScrollPointer {
    phase: Phase,
    part: Part,
    pos: Position,
    local: Position,
    track_len: u16,
}

impl fmt::Debug for ScrollRegion<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScrollRegion")
            .field("id", &self.id)
            .field("family", &self.family)
            .field("overrides", &self.ov)
            .finish()
    }
}

impl<'a> ScrollRegion<'a> {
    /// The parts this component styles.
    pub const PARTS: &'static [Part] = &[Part::TRACK, Part::THUMB, Part::CONTAINER];

    /// A scroll region for the container `id`.
    pub const fn new(id: Id) -> Self {
        ScrollRegion {
            id,
            family: Family::SCROLLBAR,
            ov: Overrides::new(),
        }
    }

    /// Resolve a composed scrollbar through its owning component's recipe.
    pub(crate) const fn inherit_family(mut self, family: Family) -> Self {
        self.family = family;
        self
    }

    /// An instance patch over every part.
    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.patch(p);
        self
    }

    /// Per-part instance patches.
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.patch_part(ps);
        self
    }

    /// Replace one part's painting.
    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    /// Apply last frame's layout facts (viewport and content) and the
    /// pending reveal, then drain wheel and scrollbar intents.
    pub fn update(
        &self,
        cx: &mut Cx<'_>,
        st: &mut ScrollState,
        content_len: usize,
    ) -> Response<()> {
        let track_len = self.prepare(cx, st, content_len);
        let mut r = Response::ignored();
        for it in cx.intents(self.id) {
            r |= self.handle_intent(cx, st, track_len, it);
        }
        r.for_id(self.id)
    }

    /// Apply last frame's layout facts without touching the intent queue.
    pub(crate) fn prepare(&self, cx: &Cx<'_>, st: &mut ScrollState, content_len: usize) -> u16 {
        let viewport = cx
            .layout(self.id)
            .map_or(st.viewport_len(), |l| l.viewport_len);
        st.apply_layout(viewport, content_len);
        cx.layout(self.id)
            .map_or(viewport.min(usize::from(u16::MAX)) as u16, |l| l.rows)
    }

    /// Handle one intent that belongs to the scroll region.
    ///
    /// Non-scroll intents are deliberately ignored so a parent control can
    /// route the same owner bucket to its own editor or interaction logic.
    pub(crate) fn handle_intent(
        &self,
        cx: &mut Cx<'_>,
        st: &mut ScrollState,
        track_len: u16,
        it: Intent<'_>,
    ) -> Response<()> {
        match it {
            Intent::Wheel { delta, .. } => st.wheel(delta),
            Intent::Pointer {
                phase,
                part,
                pos,
                local,
                ..
            } if part.part == Part::TRACK || part.part == Part::THUMB => self.pointer(
                cx,
                st,
                ScrollPointer {
                    phase,
                    part: part.part,
                    pos,
                    local,
                    track_len,
                },
            ),
            _ => Response::ignored(),
        }
    }

    fn pointer(
        &self,
        cx: &mut Cx<'_>,
        st: &mut ScrollState,
        pointer: ScrollPointer,
    ) -> Response<()> {
        let ScrollPointer {
            phase,
            part,
            pos,
            local,
            track_len,
        } = pointer;
        let before = st.offset();
        match phase {
            Phase::Press if part == Part::THUMB => {
                let _ = cx.capture(self.id, PartRef::of(Part::THUMB));
                Response::consumed()
            }
            Phase::Press => {
                let track_pos = usize::from(local.y.saturating_sub(1));
                st.scroll_to(st.offset_for_track_pos(track_pos, usize::from(track_len)));
                moved(st.offset() != before)
            }
            Phase::Drag => {
                let (_, thumb_len) = st.thumb(usize::from(track_len));
                let capture_area = cx.capture_area().unwrap_or_default();
                let origin = cx.capture_origin().unwrap_or(pos);
                let grab = origin.y.saturating_sub(capture_area.y);
                let track_y = cx
                    .area(self.id)
                    .map_or(capture_area.y, |area| area.y.saturating_add(1));
                let centered = thumb_drag_position(pos.y, track_y, grab, thumb_len);
                st.scroll_to(st.offset_for_track_pos(centered, usize::from(track_len)));
                moved(st.offset() != before)
            }
            Phase::Release | Phase::DragEnd => {
                if cx.capture_owner() == Some(self.id) {
                    cx.release_capture();
                }
                Response::consumed()
            }
            _ => Response::consumed(),
        }
    }

    /// The effective view for `area` and `content_len`: last frame's state
    /// with the viewport applied and the pending reveal honoured, so `draw`
    /// and the next `update` agree on the offset.
    pub fn view(st: &ScrollState, area: Rect, content_len: usize) -> ScrollState {
        let mut v = *st;
        v.apply_layout(usize::from(area.height), content_len);
        v
    }

    /// Register the scroll region, paint the bar when the content
    /// overflows, and return the content rect (`area` minus the bar column).
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &ScrollState, content_len: usize) -> Rect {
        if area.is_empty() {
            return area;
        }
        let view = Self::view(st, area, content_len);
        // A composed scrollbar shares its owner's id, but focus and owner-wide
        // press belong to the owner. Only an exact thumb part target is local.
        let container = self.ov.style(
            ui,
            self.id,
            self.family,
            Variant::DEFAULT,
            Part::CONTAINER,
            StateFlags::empty(),
        );
        ui.fill(area, container.style);
        let track_height = area.height.saturating_sub(2);
        ui.report_layout(
            self.id,
            LayoutFacts::new(
                usize::from(area.height),
                content_len,
                track_height,
                area.width,
            ),
        );
        ui.register_decor(self.id, PartRef::of(Part::CONTAINER), area);
        ui.register_scroll(self.id, area, Axes::V, view.headroom_v());
        if !view.overflows() {
            return area;
        }
        let bar = Rect {
            x: area.right().saturating_sub(1),
            y: area.y,
            width: 1,
            height: area.height,
        };
        let content = Rect {
            width: area.width.saturating_sub(1),
            ..area
        };
        self.paint_bar(ui, bar, &view);
        content
    }

    fn paint_bar(&self, ui: &mut Ui<'_>, bar: Rect, view: &ScrollState) {
        let track_rect = Rect {
            y: bar.y.saturating_add(1),
            height: bar.height.saturating_sub(2),
            ..bar
        };
        let track_len = usize::from(track_rect.height);
        let (start, len) = view.thumb(track_len);
        let ov = self.ov;
        let part_live = StateFlags::empty();
        let track = ov.style(
            ui,
            self.id,
            self.family,
            Variant::DEFAULT,
            Part::TRACK,
            part_live,
        );
        let thumb_live = if ui.pressed_part(self.id) == Some(PartRef::of(Part::THUMB)) {
            part_live | StateFlags::PRESSED
        } else {
            part_live
        };
        let thumb = ov.style(
            ui,
            self.id,
            self.family,
            Variant::DEFAULT,
            Part::THUMB,
            thumb_live,
        );
        let thumb_rect = Rect {
            y: track_rect
                .y
                .saturating_add(start.min(usize::from(u16::MAX)) as u16),
            height: len.min(usize::from(u16::MAX)) as u16,
            ..track_rect
        };
        if let Some(f) = ov.slot_for(Part::TRACK) {
            f(ui, bar);
        } else {
            match track.glyph {
                Slot::Set(glyph) => {
                    for row in bar.rows() {
                        ui.glyph(row, glyph, track.style);
                    }
                }
                Slot::Clear => ui.fill(bar, track.style),
                Slot::Inherit => {
                    for row in track_rect.rows() {
                        ui.glyph(row, GlyphRole::ScrollTrack, track.style);
                    }
                    let set = ui.design().glyphs.scrollbar();
                    ui.paint_cell(Position::new(bar.x, bar.y), set.begin, track.style);
                    if bar.height > 1 {
                        ui.paint_cell(
                            Position::new(bar.x, bar.bottom().saturating_sub(1)),
                            set.end,
                            track.style,
                        );
                    }
                }
            }
        }
        if let Some(f) = ov.slot_for(Part::THUMB) {
            f(ui, thumb_rect);
        } else {
            let glyph = match thumb.glyph {
                Slot::Set(g) => Some(g),
                Slot::Inherit => Some(GlyphRole::ScrollThumb),
                Slot::Clear => None,
            };
            for row in thumb_rect.rows() {
                if let Some(g) = glyph {
                    ui.glyph(row, g, thumb.style);
                } else {
                    ui.fill(row, thumb.style);
                }
            }
        }
        ui.register_part(self.id, PartRef::of(Part::TRACK), bar);
        ui.register_part(self.id, PartRef::of(Part::THUMB), thumb_rect);
    }

    /// A scroll region takes whatever its container gives it; the minimum is
    /// the bar column plus one content column.
    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        Size {
            min: (2, 1),
            preferred: c.max,
        }
        .fit(c)
    }
}

const fn moved(yes: bool) -> Response<()> {
    if yes {
        Response::changed()
    } else {
        Response::consumed()
    }
}

fn thumb_drag_position(pos_y: u16, track_y: u16, grab: u16, thumb_len: usize) -> usize {
    usize::from(pos_y.saturating_sub(track_y).saturating_sub(grab)).saturating_add(thumb_len / 2)
}

#[cfg(test)]
mod tests {
    use ratatui_core::buffer::Buffer;

    use super::*;
    use crate::runtime::Runtime;
    use crate::runtime::stub::{SCREEN, Stub};
    use crate::theme::Theme;
    use crate::{ReferenceState, ReferenceTarget};

    const ID: Id = Id::root("scroll-region.tests");

    #[test]
    fn thumb_drag_preserves_the_grab_offset() {
        assert_eq!(thumb_drag_position(18, 10, 2, 4), 8);
        assert_eq!(thumb_drag_position(9, 10, 2, 4), 2);
    }

    #[test]
    fn scrollbar_paints_typed_begin_and_end_caps() {
        let mut rt = Runtime::new(Stub::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        let area = Rect::new(0, 0, 5, 6);
        let st = ScrollState::new(100);
        rt.draw_scene(SCREEN, &mut buf, |ui, _| {
            ScrollRegion::new(ID).draw(ui, area, &st, 100);
        });
        let set = Theme::junie().design.glyphs.scrollbar();
        assert_eq!(
            buf.cell(Position::new(4, 0))
                .map(ratatui_core::buffer::Cell::symbol),
            Some(set.begin)
        );
        assert_eq!(
            buf.cell(Position::new(4, 5))
                .map(ratatui_core::buffer::Cell::symbol),
            Some(set.end)
        );
    }

    #[test]
    fn a_reference_scroll_region_registers_nothing() {
        let mut rt = Runtime::new(Stub::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        let st = ScrollState::new(100);
        rt.draw_scene(SCREEN, &mut buf, |ui, area| {
            ui.reference(None, |ui| {
                ScrollRegion::new(ID).draw(ui, area, &st, 100);
            });
        });
        assert!(rt.area_of(ID).is_none());
    }

    #[test]
    fn owner_state_never_leaks_into_the_scrollbar() {
        let render = |target: Option<ReferenceTarget>| {
            let mut runtime = Runtime::new(Stub::default(), Theme::junie());
            let mut buffer = Buffer::empty(SCREEN);
            let state = ScrollState::new(100);
            runtime.draw_scene(SCREEN, &mut buffer, |ui, _area| {
                ui.reference(target, |ui| {
                    ScrollRegion::new(ID).draw(ui, Rect::new(0, 0, 5, 6), &state, 100);
                });
            });
            buffer
        };
        let plain = render(None);
        let focused = render(Some(ReferenceTarget::new(ID, ReferenceState::FOCUSED)));
        let pressed = render(Some(ReferenceTarget::new(ID, ReferenceState::PRESSED)));
        assert_eq!(focused, plain, "owner focus styled its scrollbar");
        assert_eq!(pressed, plain, "owner press styled its scrollbar");
    }

    #[test]
    fn direct_reference_press_targets_only_the_thumb() {
        let render = |part: Option<PartRef>| {
            let mut runtime = Runtime::new(Stub::default(), Theme::junie());
            let mut buffer = Buffer::empty(SCREEN);
            let state = ScrollState::new(100);
            runtime.draw_scene(SCREEN, &mut buffer, |ui, _| {
                let target =
                    part.map(|part| ReferenceTarget::new(ID, ReferenceState::PRESSED).part(part));
                ui.reference(target, |ui| {
                    ScrollRegion::new(ID).draw(ui, Rect::new(0, 0, 5, 6), &state, 100);
                });
            });
            buffer
        };
        let plain = render(None);
        let track = render(Some(PartRef::of(Part::TRACK)));
        let thumb = render(Some(PartRef::of(Part::THUMB)));
        assert_eq!(track, plain, "a track target styled the thumb");
        assert_ne!(thumb, plain, "an exact thumb target did not paint PRESSED");
    }

    #[test]
    fn container_patch_paints_even_without_overflow() {
        let patch = StylePatch::new()
            .set_fg(crate::theme::Role::Warning)
            .set_bg(crate::theme::Role::Danger);
        let render = |patched: bool| {
            let mut rt = Runtime::new(Stub::default(), Theme::junie());
            let mut buf = Buffer::empty(SCREEN);
            let st = ScrollState::new(1);
            rt.draw_scene(SCREEN, &mut buf, |ui, _| {
                let parts = [(Part::CONTAINER, patch)];
                let mut region = ScrollRegion::new(ID);
                if patched {
                    region = region.patch_part(&parts);
                }
                region.draw(ui, Rect::new(0, 0, 5, 3), &st, 1);
            });
            buf
        };
        assert_ne!(render(false), render(true));
    }

    #[test]
    fn slot_addressable_parts_are_exactly_track_and_thumb() {
        let marker = |ui: &mut Ui<'_>, area: Rect| {
            let style = ui.surface_style();
            ui.paint_str(area, "ZZZZZZZZ", style);
        };
        let render = |slot: Option<Part>| {
            let mut runtime = Runtime::new(Stub::default(), Theme::junie());
            let mut buffer = Buffer::empty(SCREEN);
            let state = ScrollState::new(100);
            runtime.draw_scene(SCREEN, &mut buffer, |ui, _| {
                let mut region = ScrollRegion::new(ID);
                if let Some(part) = slot {
                    region = region.slot(part, &marker);
                }
                region.draw(ui, Rect::new(0, 0, 5, 6), &state, 100);
            });
            buffer
        };
        let plain = render(None);
        for part in Part::ALL {
            let changed = render(Some(*part)) != plain;
            assert_eq!(
                changed,
                matches!(*part, Part::TRACK | Part::THUMB),
                "unexpected slot behavior for {part:?}"
            );
        }
    }
}
