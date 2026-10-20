use core::fmt;

use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::Modifier;

use super::{PartStyle, SlotFn};
use crate::hit::Axes;
use crate::id::{Id, Part, PartRef};
use crate::intent::{Intent, Phase};
use crate::measure::{Constraints, Size};
use crate::response::{Response, StateFlags};
use crate::scroll::ScrollState;
use crate::theme::{Family, GlyphRole, Slot, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, LayoutFacts, Ui};

pub struct ScrollRegion<'a> {
    id: Id,
    family: Family,
    scrollbar_visible: bool,
    preserve_underlay: bool,
    ov: PartStyle<'a>,
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
            .field("scrollbar_visible", &self.scrollbar_visible)
            .field("overrides", &self.ov)
            .finish()
    }
}

impl<'a> ScrollRegion<'a> {
    pub const PARTS: &'static [Part] = &[Part::TRACK, Part::THUMB, Part::CONTAINER];

    pub const fn new(id: Id) -> Self {
        ScrollRegion {
            id,
            family: Family::SCROLLBAR,
            scrollbar_visible: true,
            preserve_underlay: false,
            ov: PartStyle::new(),
        }
    }

    #[must_use]
    pub const fn scrollbar_visible(mut self, visible: bool) -> Self {
        self.scrollbar_visible = visible;
        self
    }

    /// Leave cells the caller already painted. The picker card samples backdrop ink first.
    #[must_use]
    pub const fn preserve_underlay(mut self, preserve: bool) -> Self {
        self.preserve_underlay = preserve;
        self
    }

    pub const fn inherit_family(mut self, family: Family) -> Self {
        self.family = family;
        self
    }

    pub const fn with_overrides(mut self, ov: PartStyle<'a>) -> Self {
        self.ov = ov;
        self
    }

    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.global(p);
        self
    }

    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

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

    pub fn prepare(&self, cx: &Cx<'_>, st: &mut ScrollState, content_len: usize) -> u16 {
        let viewport = cx
            .layout(self.id)
            .map_or(st.viewport_len(), |l| l.viewport_len);
        st.apply_layout(viewport, content_len);
        cx.layout(self.id)
            .map_or(viewport.min(usize::from(u16::MAX)) as u16, |l| l.rows)
    }

    pub fn handle_intent(
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
        if !self.scrollbar_visible {
            if cx.capture_owner() == Some(self.id) {
                cx.release_capture();
            }
            return Response::consumed();
        }
        let before = st.offset();
        match phase {
            Phase::Press if part == Part::THUMB => {
                let _ = cx.capture(self.id, PartRef::of(Part::THUMB));
                Response::consumed()
            }
            Phase::Press => {
                let track_pos = usize::from(local.y);
                st.scroll_to(st.offset_for_track_pos(track_pos, usize::from(track_len)));
                moved(st.offset() != before)
            }
            Phase::Drag => {
                let capture_area = cx.capture_area().unwrap_or_default();
                let origin = cx.capture_origin().unwrap_or(pos);
                let track_y = cx
                    .area_of_part(self.id, PartRef::of(Part::TRACK))
                    .or_else(|| cx.area(self.id))
                    .map_or(capture_area.y, |area| area.y);
                if cx.capture_owner() != Some(self.id) {
                    let track_pos = usize::from(pos.y.saturating_sub(track_y));
                    st.scroll_to(st.offset_for_track_pos(track_pos, usize::from(track_len)));
                    return moved(st.offset() != before);
                }
                let (_, thumb_len) = st.thumb(usize::from(track_len));
                let grab = origin.y.saturating_sub(capture_area.y);
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

    pub fn view(st: &ScrollState, area: Rect, content_len: usize) -> ScrollState {
        let mut v = *st;
        v.apply_layout(usize::from(area.height), content_len);
        v
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &ScrollState, content_len: usize) -> Rect {
        if area.is_empty() {
            return area;
        }
        let view = Self::view(st, area, content_len);

        let container = self.ov.style(
            ui,
            self.id,
            self.family,
            Variant::DEFAULT,
            Part::CONTAINER,
            StateFlags::empty(),
        );
        if !self.preserve_underlay {
            ui.fill(area, container.style);
        }
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
        if !self.scrollbar_visible || !view.overflows() {
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
        let track_rect = bar;
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
        // `set_style` unions modifiers, so a bold cell under the bar stays bold unless subtracted.
        let track_style = without_bold(track.style, ov.part_patch(Part::TRACK));
        let thumb_style = without_bold(thumb.style, ov.part_patch(Part::THUMB));
        if let Some(f) = ov.slot_for(Part::TRACK) {
            f(ui, bar);
        } else {
            match track.glyph {
                Slot::Set(glyph) if glyph != GlyphRole::ScrollTrack => {
                    for row in bar.rows() {
                        ui.glyph(row, glyph, track_style);
                    }
                }
                Slot::Clear => ui.fill(bar, track_style),
                Slot::Inherit | Slot::Set(_) => {
                    for row in track_rect.rows() {
                        ui.glyph(row, GlyphRole::ScrollTrack, track_style);
                    }
                    let set = ui.design().glyphs.scrollbar();
                    ui.paint_cell(Position::new(bar.x, bar.y), set.begin, track_style);
                    if bar.height > 1 {
                        ui.paint_cell(
                            Position::new(bar.x, bar.bottom().saturating_sub(1)),
                            set.end,
                            track_style,
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
                    ui.glyph(row, g, thumb_style);
                } else {
                    ui.fill(row, thumb_style);
                }
            }
        }
        ui.register_part(self.id, PartRef::of(Part::TRACK), bar);
        ui.register_part(self.id, PartRef::of(Part::THUMB), thumb_rect);
    }

    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        Size {
            min: (if self.scrollbar_visible { 2 } else { 1 }, 1),
            preferred: c.max,
        }
        .fit(c)
    }
}

fn without_bold(
    style: crate::theme::PaintStyle,
    patch: Option<StylePatch>,
) -> crate::theme::PaintStyle {
    let style = style.remove_modifier(Modifier::BOLD);
    if patch.is_some_and(|patch| patch.add.contains(Modifier::BOLD)) {
        style.add_modifier(Modifier::BOLD)
    } else {
        style
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
    fn scrollbar_paints_typed_caps_outside_the_thumb() {
        let mut rt = Runtime::new(Stub::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        let area = Rect::new(0, 0, 5, 6);
        let mut st = ScrollState::new(100);
        st.set_viewport(6);
        st.scroll_to(47);
        rt.draw_scene(SCREEN, &mut buf, |ui, _| {
            ScrollRegion::new(ID).draw(ui, area, &st, 100);
        })
        .commit_presented();
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
        })
        .commit_presented();
        assert!(rt.area_of(ID).is_none());
    }

    #[test]
    fn owner_state_never_leaks_into_the_scrollbar() {
        let render = |target: Option<ReferenceTarget>| {
            let mut runtime = Runtime::new(Stub::default(), Theme::junie());
            let mut buffer = Buffer::empty(SCREEN);
            let state = ScrollState::new(100);
            runtime
                .draw_scene(SCREEN, &mut buffer, |ui, _area| {
                    ui.reference(target, |ui| {
                        ScrollRegion::new(ID).draw(ui, Rect::new(0, 0, 5, 6), &state, 100);
                    });
                })
                .commit_presented();
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
            runtime
                .draw_scene(SCREEN, &mut buffer, |ui, _| {
                    let target = part
                        .map(|part| ReferenceTarget::new(ID, ReferenceState::PRESSED).part(part));
                    ui.reference(target, |ui| {
                        ScrollRegion::new(ID).draw(ui, Rect::new(0, 0, 5, 6), &state, 100);
                    });
                })
                .commit_presented();
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
            })
            .commit_presented();
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
            runtime
                .draw_scene(SCREEN, &mut buffer, |ui, _| {
                    let mut region = ScrollRegion::new(ID);
                    if let Some(part) = slot {
                        region = region.slot(part, &marker);
                    }
                    region.draw(ui, Rect::new(0, 0, 5, 6), &state, 100);
                })
                .commit_presented();
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

    #[test]
    fn scrollbar_drops_bold_inherited_from_the_cell_under_it() {
        let mut rt = Runtime::new(Stub::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        let area = Rect::new(0, 0, 5, 6);
        let under =
            ratatui_core::style::Style::new().add_modifier(Modifier::BOLD | Modifier::ITALIC);
        for y in area.y..area.bottom() {
            if let Some(cell) = buf.cell_mut(Position::new(area.right() - 1, y)) {
                cell.set_symbol("X").set_style(under);
            }
        }
        let mut st = ScrollState::new(100);
        st.set_viewport(6);
        st.scroll_to(47);
        rt.draw_scene(SCREEN, &mut buf, |ui, _| {
            ScrollRegion::new(ID).draw(ui, area, &st, 100);
        })
        .commit_presented();
        for y in area.y..area.bottom() {
            let Some(cell) = buf.cell(Position::new(area.right() - 1, y)) else {
                panic!("bar cell missing at y={y}");
            };
            assert_ne!(cell.symbol(), "X", "bar did not paint over y={y}");
            assert!(
                !cell.modifier.contains(Modifier::BOLD),
                "bar kept bold at y={y}"
            );
            assert!(
                cell.modifier.contains(Modifier::ITALIC),
                "bar wiped italic at y={y}"
            );
        }
    }
}
