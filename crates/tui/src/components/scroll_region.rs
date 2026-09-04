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
/// `.patch`, `.patch_part`, `.slot`, `.state_override`. The axis is
/// vertical; `Axes::V` is registered.
///
/// ## Variants
/// `Family::SCROLLBAR`, `DEFAULT` only.
///
/// ## States
/// The thumb wears `HOVERED`, `FOCUSED` and `PRESSED` from the container's
/// runtime state (a live capture keeps `PRESSED`).
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
    ov: Overrides<'a>,
}

impl fmt::Debug for ScrollRegion<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScrollRegion")
            .field("id", &self.id)
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
            ov: Overrides::new(),
        }
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

    /// Showcase / fixture use only (A11).
    #[must_use]
    pub const fn state_override(mut self, s: StateFlags) -> Self {
        self.ov = self.ov.state_override(s);
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
                phase, part, local, ..
            } if part.part == Part::TRACK || part.part == Part::THUMB => {
                self.pointer(cx, st, phase, part.part, local, track_len)
            }
            _ => Response::ignored(),
        }
    }

    fn pointer(
        &self,
        cx: &mut Cx<'_>,
        st: &mut ScrollState,
        phase: Phase,
        part: Part,
        local: Position,
        track_len: u16,
    ) -> Response<()> {
        let before = st.offset();
        match phase {
            Phase::Press if part == Part::THUMB => {
                // the drag is measured against the track, so `local.y` is a
                // track position from the first drag onwards
                let _ = cx.capture(self.id, PartRef::of(Part::TRACK));
                Response::consumed()
            }
            Phase::Press | Phase::Drag => {
                st.scroll_to(st.offset_for_track_pos(usize::from(local.y), usize::from(track_len)));
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
        ui.report_layout(
            self.id,
            LayoutFacts::new(
                usize::from(area.height),
                content_len,
                area.height,
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
        let live = self.ov.flags(ui.state(self.id));
        let track_len = usize::from(bar.height);
        let (start, len) = view.thumb(track_len);
        let ov = self.ov;
        let track = ov.style(
            ui,
            self.id,
            Family::SCROLLBAR,
            Variant::DEFAULT,
            Part::TRACK,
            live,
        );
        let thumb = ov.style(
            ui,
            self.id,
            Family::SCROLLBAR,
            Variant::DEFAULT,
            Part::THUMB,
            live,
        );
        let thumb_rect = Rect {
            y: bar
                .y
                .saturating_add(start.min(usize::from(u16::MAX)) as u16),
            height: len.min(usize::from(u16::MAX)) as u16,
            ..bar
        };
        if let Some(f) = ov.slot_for(Part::TRACK) {
            f(ui, bar);
        } else {
            let glyph = match track.glyph {
                Slot::Set(g) => Some(g),
                Slot::Inherit => Some(GlyphRole::ScrollTrack),
                Slot::Clear => None,
            };
            for row in bar.rows() {
                if let Some(g) = glyph {
                    ui.glyph(row, g, track.style);
                } else {
                    ui.fill(row, track.style);
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
        content
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
