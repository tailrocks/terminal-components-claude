use core::fmt;
use core::ops::Range;

use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::Modifier;

use super::{PartStyle, SlotFn};
use crate::author::VerticalWindow;
use crate::diagnostics::Diagnostic;
use crate::hit::Axes;
use crate::id::{Id, Part, PartRef};
use crate::intent::{Intent, Phase};
use crate::measure::{Constraints, Size};
use crate::response::{Response, StateFlags};
use crate::scroll::ScrollState;
use crate::theme::{Family, GlyphRole, Slot, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, LayoutFacts, Ui};

/// Controls whether a visible scrollbar reserves its column only while it is
/// needed or for the entire lifetime of the scroll region.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollbarGutter {
    /// Reserve the scrollbar column only while content overflows.
    #[default]
    Auto,
    /// Keep one column reserved even when all content fits.
    Stable,
}

pub struct ScrollRegion<'a> {
    id: Id,
    family: Family,
    scrollbar_visible: bool,
    scrollbar_gutter: ScrollbarGutter,
    preserve_underlay: bool,
    ov: PartStyle<'a>,
    window_slot: Option<(Part, WindowSlotFn<'a>)>,
}

/// Source interval supplied to a window-aware ScrollRegion part painter.
///
/// `part_rows` is the original logical row interval of the part. `visible_rows`
/// is its intersection with `window`; the callback's Rect is the physical
/// rectangle corresponding exactly to that intersection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScrollRegionSlotWindow {
    window: VerticalWindow,
    part_rows: Range<usize>,
    visible_rows: Range<usize>,
}

impl ScrollRegionSlotWindow {
    pub const fn window(&self) -> VerticalWindow {
        self.window
    }

    pub fn part_rows(&self) -> &Range<usize> {
        &self.part_rows
    }

    pub fn visible_rows(&self) -> &Range<usize> {
        &self.visible_rows
    }
}

/// Painter for a clipped ScrollRegion part, given its visible physical area
/// and original logical source interval.
pub type WindowSlotFn<'a> = &'a dyn Fn(&mut Ui<'_>, Rect, &ScrollRegionSlotWindow);

#[derive(Clone, Copy)]
struct ScrollPointer {
    phase: Phase,
    part: Part,
    pos: Position,
    local: Position,
    track_len: usize,
    window: Option<VerticalWindow>,
}

impl fmt::Debug for ScrollRegion<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScrollRegion")
            .field("id", &self.id)
            .field("family", &self.family)
            .field("scrollbar_visible", &self.scrollbar_visible)
            .field("scrollbar_gutter", &self.scrollbar_gutter)
            .field("window_slot", &self.window_slot.map(|(part, _)| part))
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
            scrollbar_gutter: ScrollbarGutter::Auto,
            preserve_underlay: false,
            ov: PartStyle::new(),
            window_slot: None,
        }
    }

    #[must_use]
    pub const fn scrollbar_visible(mut self, visible: bool) -> Self {
        self.scrollbar_visible = visible;
        self
    }

    #[must_use]
    pub const fn scrollbar_gutter(mut self, gutter: ScrollbarGutter) -> Self {
        self.scrollbar_gutter = gutter;
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

    /// Paint one scrollbar part with source-row information when its track is
    /// clipped by an ancestor window. This slot takes precedence over the
    /// legacy slot for the same part; legacy slots remain unchanged when the
    /// entire part is visible.
    #[must_use]
    pub const fn window_slot(mut self, part: Part, f: WindowSlotFn<'a>) -> Self {
        self.window_slot = Some((part, f));
        self
    }

    pub fn update(
        &self,
        cx: &mut Cx<'_>,
        st: &mut ScrollState,
        content_len: usize,
    ) -> Response<()> {
        let track_len = self.prepare(cx, st, content_len);
        let window = cx.layout(self.id).and_then(|layout| layout.vertical_window);
        let mut r = Response::ignored();
        for it in cx.intents(self.id) {
            r |= self.handle_intent_in_window(cx, st, track_len, window, it);
        }
        r.for_id(self.id)
    }

    pub fn prepare(&self, cx: &Cx<'_>, st: &mut ScrollState, content_len: usize) -> u16 {
        let layout = cx.layout(self.id);
        let viewport = layout.map_or(st.viewport_len(), |layout| {
            layout
                .vertical_window
                .map_or(layout.viewport_len, VerticalWindow::logical_rows)
        });
        st.apply_layout(viewport, content_len);
        layout.map_or(viewport.min(usize::from(u16::MAX)) as u16, |l| l.rows)
    }

    pub fn handle_intent(
        &self,
        cx: &mut Cx<'_>,
        st: &mut ScrollState,
        track_len: u16,
        it: Intent<'_>,
    ) -> Response<()> {
        let window = cx.layout(self.id).and_then(|layout| layout.vertical_window);
        self.handle_intent_in_window(cx, st, track_len, window, it)
    }

    fn handle_intent_in_window(
        &self,
        cx: &mut Cx<'_>,
        st: &mut ScrollState,
        track_len: u16,
        window: Option<VerticalWindow>,
        it: Intent<'_>,
    ) -> Response<()> {
        match it {
            Intent::Wheel { delta, .. } => st.wheel(delta),
            Intent::Cancel => {
                if cx.capture_owner() == Some(self.id) {
                    cx.release_capture();
                }
                st.release_track();
                Response::consumed()
            }
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
                    track_len: logical_track_len(track_len, window),
                    window,
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
            window,
        } = pointer;
        if !self.scrollbar_visible {
            if cx.capture_owner() == Some(self.id) {
                cx.release_capture();
            }
            st.release_track();
            return Response::consumed();
        }
        let before = st.offset();
        match phase {
            Phase::Press if part == Part::THUMB => {
                if let Some(window) = window
                    && let Some(track_pos) = logical_track_row(window, pos.y)
                {
                    st.press_track(track_pos, track_len);
                }
                if !cx.capture(self.id, PartRef::of(Part::THUMB)) {
                    st.release_track();
                }
                Response::consumed()
            }
            Phase::Press => {
                let track_pos = window
                    .and_then(|window| window.source_row_for_physical(pos.y))
                    .unwrap_or_else(|| usize::from(local.y));
                st.scroll_to(st.offset_for_track_pos(track_pos, track_len));
                moved(st.offset() != before)
            }
            Phase::Drag => {
                if let Some(window) = window {
                    let Some(track_y) = logical_track_row_for_drag(window, pos.y) else {
                        return Response::consumed();
                    };
                    if cx.capture_owner() != Some(self.id) {
                        st.release_track();
                        st.scroll_to(st.offset_for_track_pos(track_y, track_len));
                        return moved(st.offset() != before);
                    }
                    st.drag_track(track_y, track_len);
                    return moved(st.offset() != before);
                }
                let capture_area = cx.capture_area().unwrap_or_default();
                let origin = cx.capture_origin().unwrap_or(pos);
                let track_y = cx
                    .area_of_part(self.id, PartRef::of(Part::TRACK))
                    .or_else(|| cx.area(self.id))
                    .map_or(capture_area.y, |area| area.y);
                if cx.capture_owner() != Some(self.id) {
                    let track_pos = usize::from(pos.y.saturating_sub(track_y));
                    st.scroll_to(st.offset_for_track_pos(track_pos, track_len));
                    return moved(st.offset() != before);
                }
                let (_, thumb_len) = st.thumb(track_len);
                let grab = origin.y.saturating_sub(capture_area.y);
                let centered = thumb_drag_position(pos.y, track_y, grab, thumb_len);
                st.scroll_to(st.offset_for_track_pos(centered, track_len));
                moved(st.offset() != before)
            }
            Phase::Release | Phase::DragEnd => {
                st.release_track();
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
        self.draw_window(ui, VerticalWindow::full(area), st, content_len)
    }

    /// Draw a ScrollRegion whose physical body is a visible window into the
    /// original logical viewport. The state and thumb retain logical sizing.
    pub fn draw_window(
        &self,
        ui: &mut Ui<'_>,
        window: VerticalWindow,
        st: &ScrollState,
        content_len: usize,
    ) -> Rect {
        let area = window.visible_body();
        if area.is_empty() {
            ui.report_layout(
                self.id,
                LayoutFacts::new(window.logical_rows(), content_len, 0, area.width)
                    .with_vertical_window(window),
            );
            return area;
        }
        let mut view = *st;
        view.apply_layout(window.logical_rows(), content_len);

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
                window.logical_rows(),
                content_len,
                window.logical_rows().min(usize::from(u16::MAX)) as u16,
                area.width,
            )
            .with_vertical_window(window),
        );
        ui.register_decor(self.id, PartRef::of(Part::CONTAINER), area);
        ui.register_scroll(self.id, area, Axes::V, view.headroom_v());
        if !self.scrollbar_visible {
            return area;
        }
        let overflows = view.overflows();
        if !overflows && self.scrollbar_gutter == ScrollbarGutter::Auto {
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
        if overflows {
            self.paint_bar(ui, bar, &view, window);
        }
        content
    }

    fn paint_bar(&self, ui: &mut Ui<'_>, bar: Rect, view: &ScrollState, window: VerticalWindow) {
        let track_len = window.logical_rows();
        let (start, len) = view.thumb(track_len);
        let ov = self.ov;
        let part_live = StateFlags::empty();
        let track_patch = ov.part_patch(Part::TRACK);
        let track_resolved = ui.style_defaults_with_explicit(
            self.family,
            Variant::DEFAULT,
            Part::TRACK,
            part_live,
            crate::theme::StyleDefaults::default(),
            track_patch.as_ref(),
        );
        ov.note(
            ui,
            self.id,
            self.family,
            Variant::DEFAULT,
            Part::TRACK,
            track_resolved.resolved,
        );
        let track = track_resolved.resolved;
        let thumb_live = if ui.pressed_part(self.id) == Some(PartRef::of(Part::THUMB)) {
            part_live | StateFlags::PRESSED
        } else {
            part_live
        };
        let thumb_patch = ov.part_patch(Part::THUMB);
        let thumb_resolved = ui.style_defaults_with_explicit(
            self.family,
            Variant::DEFAULT,
            Part::THUMB,
            thumb_live,
            crate::theme::StyleDefaults::default(),
            thumb_patch.as_ref(),
        );
        ov.note(
            ui,
            self.id,
            self.family,
            Variant::DEFAULT,
            Part::THUMB,
            thumb_resolved.resolved,
        );
        let thumb = thumb_resolved.resolved;
        let track_rows = 0..track_len;
        let thumb_rows = start..start.saturating_add(len);
        let Some((visible_track, track_context)) = visible_part(window, bar.x, track_rows.clone())
        else {
            return;
        };
        let visible_thumb = visible_part(window, bar.x, thumb_rows.clone());
        let track_painted = self.paint_part(
            ui,
            Part::TRACK,
            visible_track,
            &track_context,
            track.style,
            track_resolved.explicit,
            track.glyph,
            true,
        );
        let thumb_painted = if let Some((thumb_rect, ref thumb_context)) = visible_thumb {
            self.paint_part(
                ui,
                Part::THUMB,
                thumb_rect,
                thumb_context,
                thumb.style,
                thumb_resolved.explicit,
                thumb.glyph,
                false,
            )
        } else {
            false
        };
        if track_painted {
            ui.register_part(self.id, PartRef::of(Part::TRACK), visible_track);
        }
        if thumb_painted && let Some((thumb_rect, _)) = visible_thumb {
            ui.register_part(self.id, PartRef::of(Part::THUMB), thumb_rect);
        }
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "part painting keeps the resolved style and source geometry explicit"
    )]
    fn paint_part(
        &self,
        ui: &mut Ui<'_>,
        part: Part,
        visible_rect: Rect,
        context: &ScrollRegionSlotWindow,
        style: crate::theme::PaintStyle,
        explicit: StylePatch,
        glyph: Slot<GlyphRole>,
        is_track: bool,
    ) -> bool {
        if let Some((window_part, painter)) = self.window_slot
            && window_part == part
        {
            painter(ui, visible_rect, context);
            return true;
        }
        if let Some(painter) = self.ov.slot_for(part) {
            if context.part_rows != context.visible_rows {
                ui.diagnostic(Diagnostic::UnsupportedClippedComponentSlot {
                    owner: self.id,
                    part,
                });
                return false;
            }
            painter(ui, visible_rect);
            return true;
        }
        let style = without_inherited_bold(style, explicit);
        if is_track {
            match glyph {
                Slot::Set(glyph) if glyph != GlyphRole::ScrollTrack => {
                    for row in visible_rect.rows() {
                        ui.glyph(row, glyph, style);
                    }
                }
                Slot::Clear => ui.fill(visible_rect, style),
                Slot::Inherit | Slot::Set(_) => {
                    for row in visible_rect.rows() {
                        ui.glyph(row, GlyphRole::ScrollTrack, style);
                    }
                    let caps = ui.design().glyphs.scrollbar();
                    if context.visible_rows.start == 0 {
                        ui.paint_cell(
                            Position::new(visible_rect.x, visible_rect.y),
                            caps.begin,
                            style,
                        );
                    }
                    if context.visible_rows.end == context.window.logical_rows()
                        && visible_rect.height > 0
                    {
                        ui.paint_cell(
                            Position::new(visible_rect.x, visible_rect.bottom().saturating_sub(1)),
                            caps.end,
                            style,
                        );
                    }
                }
            }
        } else {
            let thumb_glyph = match glyph {
                Slot::Set(glyph) => Some(glyph),
                Slot::Inherit => Some(GlyphRole::ScrollThumb),
                Slot::Clear => None,
            };
            for row in visible_rect.rows() {
                if let Some(glyph) = thumb_glyph {
                    ui.glyph(row, glyph, style);
                } else {
                    ui.fill(row, style);
                }
            }
        }
        true
    }

    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        Size {
            min: (if self.scrollbar_visible { 2 } else { 1 }, 1),
            preferred: c.max,
        }
        .fit(c)
    }
}

fn visible_part(
    window: VerticalWindow,
    x: u16,
    part_rows: Range<usize>,
) -> Option<(Rect, ScrollRegionSlotWindow)> {
    let visible_window = window.visible_source_range();
    let start = part_rows.start.max(visible_window.start);
    let end = part_rows.end.min(visible_window.end);
    if start >= end {
        return None;
    }
    let y_offset = start.checked_sub(window.first_source_row())?;
    let y = window
        .visible_body()
        .y
        .checked_add(u16::try_from(y_offset).ok()?)?;
    let height = u16::try_from(end.saturating_sub(start)).ok()?;
    Some((
        Rect::new(x, y, 1, height),
        ScrollRegionSlotWindow {
            window,
            part_rows,
            visible_rows: start..end,
        },
    ))
}

fn logical_track_row(window: VerticalWindow, physical_y: u16) -> Option<usize> {
    window.source_row_for_physical(physical_y)
}

fn logical_track_len(fallback: u16, window: Option<VerticalWindow>) -> usize {
    window.map_or(usize::from(fallback), VerticalWindow::logical_rows)
}

fn logical_track_row_for_drag(window: VerticalWindow, physical_y: u16) -> Option<usize> {
    let logical_rows = window.logical_rows();
    if logical_rows == 0 || window.visible_body().height == 0 {
        return None;
    }
    let body_y = window.visible_body().y;
    let source = if physical_y >= body_y {
        window
            .first_source_row()
            .saturating_add(usize::from(physical_y - body_y))
    } else {
        window
            .first_source_row()
            .saturating_sub(usize::from(body_y - physical_y))
    };
    Some(source.min(logical_rows - 1))
}

fn without_inherited_bold(
    style: crate::theme::PaintStyle,
    explicit: StylePatch,
) -> crate::theme::PaintStyle {
    // Ratatui's set_style unions modifiers with the underlay. Clear underlay
    // bold unless a theme, overlay, or instance style explicitly supplies it.
    if explicit.add.contains(Modifier::BOLD)
        || style.as_style().add_modifier.contains(Modifier::BOLD)
    {
        style
    } else {
        style.remove_modifier(Modifier::BOLD)
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
    use core::cell::RefCell;

    use ratatui_core::buffer::Buffer;

    use super::*;
    use crate::event::MouseKind;
    use crate::runtime::stub::{SCREEN, Stub};
    use crate::runtime::stub::{deliver, mouse};
    use crate::runtime::{App, Runtime};
    use crate::theme::Theme;
    use crate::{ReferenceState, ReferenceTarget};

    const ID: Id = Id::root("scroll-region.tests");

    #[test]
    fn stable_gutter_reserves_one_column_without_overflow_and_once_with_it() {
        let area = Rect::new(0, 0, 10, 3);
        for (gutter, content_len, expected_width, expected_bar) in [
            (ScrollbarGutter::Auto, 2, 10, false),
            (ScrollbarGutter::Stable, 2, 9, false),
            (ScrollbarGutter::Auto, 4, 9, true),
            (ScrollbarGutter::Stable, 4, 9, true),
        ] {
            let mut runtime = Runtime::new(Stub::default(), Theme::junie());
            let mut buffer = Buffer::empty(SCREEN);
            let state = ScrollState::new(content_len);
            let mut content = None;
            runtime
                .draw_scene(SCREEN, &mut buffer, |ui, _| {
                    content = Some(ScrollRegion::new(ID).scrollbar_gutter(gutter).draw(
                        ui,
                        area,
                        &state,
                        content_len,
                    ));
                })
                .commit_presented();
            assert_eq!(
                content.unwrap().width,
                expected_width,
                "{gutter:?}, {content_len}"
            );
            assert_eq!(
                runtime.area_of_part(ID, PartRef::of(Part::TRACK)),
                expected_bar.then_some(Rect::new(9, 0, 1, 3)),
                "{gutter:?}, {content_len}"
            );
        }
    }

    fn clipped_window() -> VerticalWindow {
        VerticalWindow::new(Rect::new(0, 4, 10, 5), 12, 3).unwrap()
    }

    #[test]
    fn draw_window_clips_track_and_thumb_without_creating_caps() {
        let mut rt = Runtime::new(Stub::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        let mut st = ScrollState::new(24);
        st.set_viewport(12);
        let window = clipped_window();
        let mut content = None;
        rt.draw_scene(SCREEN, &mut buf, |ui, _| {
            content = Some(ScrollRegion::new(ID).draw_window(ui, window, &st, 24));
        })
        .commit_presented();
        assert_eq!(content, Some(Rect::new(0, 4, 9, 5)));
        assert_eq!(
            rt.area_of_part(ID, PartRef::of(Part::TRACK)),
            Some(Rect::new(9, 4, 1, 5))
        );
        assert_eq!(
            rt.area_of_part(ID, PartRef::of(Part::THUMB)),
            Some(Rect::new(9, 4, 1, 3))
        );
        // The visible track rows begin at source row 3 and stop before the
        // logical end, so neither endpoint cap belongs in this window.
        // The thumb overlays the first three physical rows; inspect the
        // exposed track row to verify it remains the ordinary track glyph.
        assert_eq!(
            buf.cell(Position::new(9, 8)).map(|cell| cell.symbol()),
            Some(Theme::junie().design.glyphs.get(GlyphRole::ScrollTrack))
        );
        assert!(rt.diagnostics().is_empty());
    }

    #[test]
    fn window_slot_receives_original_and_visible_source_intervals() {
        let seen = RefCell::new(None);
        let paint = |ui: &mut Ui<'_>, area: Rect, context: &ScrollRegionSlotWindow| {
            *seen.borrow_mut() = Some(context.clone());
            for row in area.rows() {
                let source = context.visible_rows().start + usize::from(row.y - area.y);
                let marker = char::from(b'a' + u8::try_from(source).unwrap_or(0));
                ui.paint_str(row, &marker.to_string(), ui.surface_style());
            }
        };
        let mut rt = Runtime::new(Stub::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        let mut st = ScrollState::new(24);
        st.set_viewport(12);
        let window = clipped_window();
        rt.draw_scene(SCREEN, &mut buf, |ui, _| {
            ScrollRegion::new(ID)
                .window_slot(Part::TRACK, &paint)
                .draw_window(ui, window, &st, 24);
        })
        .commit_presented();

        let seen = seen.into_inner().expect("window-aware slot must run");
        assert_eq!(seen.window(), window);
        assert_eq!(seen.part_rows(), &(0..12));
        assert_eq!(seen.visible_rows(), &(3..8));
        assert_eq!(
            rt.area_of_part(ID, PartRef::of(Part::TRACK)),
            Some(Rect::new(9, 4, 1, 5))
        );
        // The thumb covers source rows 3..6; the remaining track rows keep
        // the source pattern supplied by the caller.
        for (row, marker) in (7..9).zip(['g', 'h']) {
            assert_eq!(
                buf.cell(Position::new(9, row))
                    .and_then(|cell| cell.symbol().chars().next()),
                Some(marker)
            );
        }
    }

    #[test]
    fn clipped_legacy_slot_reports_and_registers_no_phantom_part() {
        let paint = |ui: &mut Ui<'_>, area: Rect| {
            ui.paint_str(area, "legacy", ui.surface_style());
        };
        let mut rt = Runtime::new(Stub::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        let mut st = ScrollState::new(24);
        st.set_viewport(12);
        rt.draw_scene(SCREEN, &mut buf, |ui, _| {
            ScrollRegion::new(ID).slot(Part::TRACK, &paint).draw_window(
                ui,
                clipped_window(),
                &st,
                24,
            );
        })
        .commit_presented();

        assert!(
            rt.area_of(ID).is_some(),
            "the visible scroll container remains real"
        );
        assert!(rt.area_of_part(ID, PartRef::of(Part::TRACK)).is_none());
        assert!(rt.diagnostics().iter().any(|diagnostic| matches!(
            diagnostic,
            Diagnostic::UnsupportedClippedComponentSlot { owner, part: Part::TRACK }
                if *owner == ID
        )));
        for row in 4..9 {
            assert_ne!(
                buf.cell(Position::new(9, row))
                    .and_then(|cell| cell.symbol().chars().next()),
                Some('l')
            );
        }
    }

    #[test]
    fn full_window_slot_render_matches_existing_draw() {
        let paint = |ui: &mut Ui<'_>, area: Rect| {
            ui.paint_str(area, "x", ui.surface_style());
        };
        let area = Rect::new(2, 1, 8, 7);
        let mut st = ScrollState::new(60);
        st.set_viewport(7);
        st.scroll_to(20);
        let render = |windowed: bool| {
            let mut rt = Runtime::new(Stub::default(), Theme::junie());
            let mut buf = Buffer::empty(SCREEN);
            rt.draw_scene(SCREEN, &mut buf, |ui, _| {
                let region = ScrollRegion::new(ID).slot(Part::TRACK, &paint);
                if windowed {
                    region.draw_window(ui, VerticalWindow::full(area), &st, 60);
                } else {
                    region.draw(ui, area, &st, 60);
                }
            })
            .commit_presented();
            (
                buf,
                rt.area_of_part(ID, PartRef::of(Part::TRACK)),
                rt.diagnostics().to_vec(),
            )
        };
        assert_eq!(render(false), render(true));
    }

    struct ScrollHarness {
        state: ScrollState,
        source_top: usize,
    }

    struct LargeWindowHarness {
        state: ScrollState,
    }

    impl App for LargeWindowHarness {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            ScrollRegion::new(ID).update(cx, &mut self.state, 1_000_000)
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            if let Ok(window) = VerticalWindow::new(Rect::new(0, 4, 10, 10), 100_000, 45_000) {
                ScrollRegion::new(ID).draw_window(ui, window, &self.state, 1_000_000);
            }
        }
    }

    #[test]
    fn window_track_length_uses_usize_logical_rows_above_u16() {
        for rows in [
            usize::from(u16::MAX),
            usize::from(u16::MAX) + 1,
            100_000,
            usize::MAX,
        ] {
            let first = rows.saturating_sub(1);
            let window = VerticalWindow::new(Rect::new(0, 4, 1, 1), rows, first).unwrap();
            assert_eq!(logical_track_len(13, Some(window)), rows);
            assert_eq!(logical_track_row_for_drag(window, 4), Some(first));
            assert_eq!(
                logical_track_row_for_drag(window, 3),
                Some(first.saturating_sub(1).min(rows.saturating_sub(1)))
            );
        }
        assert_eq!(logical_track_len(13, None), 13);
    }

    #[test]
    fn large_logical_window_keeps_thumb_grab_offset_when_pressed() {
        let mut state = ScrollState::new(1_000_000);
        state.set_viewport(100_000);
        state.scroll_to(450_000);
        let mut rt = Runtime::new(LargeWindowHarness { state }, Theme::junie());
        let _ = rt.initialize();
        let _ = deliver(&mut rt, mouse(MouseKind::Move, 0, 0));
        let _ = deliver(&mut rt, mouse(MouseKind::Down, 9, 4));
        assert_eq!(
            rt.app().state.offset(),
            450_000,
            "the clipped visible row is inside the thumb computed over 100,000 logical rows"
        );
    }

    impl App for ScrollHarness {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            ScrollRegion::new(ID).update(cx, &mut self.state, 24)
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let screen = ui.full();
            let height = screen.height.saturating_sub(4).min(5);
            if let Ok(window) =
                VerticalWindow::new(Rect::new(0, 4, 10, height), 12, self.source_top)
            {
                ScrollRegion::new(ID).draw_window(ui, window, &self.state, 24);
            }
        }
    }

    #[test]
    fn track_click_and_captured_thumb_drag_use_logical_source_rows() {
        let mut rt = Runtime::new(
            ScrollHarness {
                state: ScrollState::new(24),
                source_top: 3,
            },
            Theme::junie(),
        );
        let _ = rt.initialize();
        let _ = deliver(&mut rt, mouse(MouseKind::Move, 0, 0));
        let _ = deliver(&mut rt, mouse(MouseKind::Down, 9, 8));
        assert_eq!(
            rt.app().state.offset(),
            8,
            "physical source row 7 maps to logical track row 7"
        );
        let _ = deliver(&mut rt, mouse(MouseKind::Up, 9, 8));
        rt.app_mut().state.scroll_to(0);
        let _ = deliver(&mut rt, mouse(MouseKind::Down, 9, 5));
        let _ = deliver(&mut rt, mouse(MouseKind::Drag, 9, 6));
        assert_eq!(
            rt.app().state.offset(),
            2,
            "drag preserves source-row grab offset"
        );
        let _ = deliver(&mut rt, mouse(MouseKind::Up, 9, 6));
        let mut buf = Buffer::empty(SCREEN);
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
    }

    #[test]
    fn resized_source_window_reuses_the_prepared_mapping_for_input() {
        let mut rt = Runtime::new(
            ScrollHarness {
                state: ScrollState::new(24),
                source_top: 3,
            },
            Theme::junie(),
        );
        let _ = rt.initialize();
        let _ = deliver(&mut rt, mouse(MouseKind::Move, 0, 0));
        rt.app_mut().source_top = 4;
        let _ = deliver(&mut rt, crate::event::Input::Resize(40, 12));
        let _ = deliver(&mut rt, mouse(MouseKind::Down, 9, 7));
        assert_eq!(
            rt.app().state.offset(),
            8,
            "after resize, visible y=7 maps through source window row 7"
        );
    }

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

    #[test]
    fn scrollbar_preserves_resolved_theme_bold() {
        let theme = Theme::junie().override_family(Family::SCROLLBAR, |recipe| {
            recipe
                .part(Part::TRACK)
                .base(StylePatch::new().add(Modifier::BOLD));
        });
        let mut rt = Runtime::new(Stub::default(), theme);
        let mut buf = Buffer::empty(SCREEN);
        let area = Rect::new(0, 0, 5, 6);
        let mut st = ScrollState::new(100);
        st.set_viewport(6);
        st.scroll_to(47);
        rt.draw_scene(SCREEN, &mut buf, |ui, _| {
            ScrollRegion::new(ID).draw(ui, area, &st, 100);
        })
        .commit_presented();
        assert!(
            buf.cell(Position::new(area.right() - 1, area.y))
                .is_some_and(|cell| cell.modifier.contains(Modifier::BOLD)),
            "theme-level track bold is part of the resolved scrollbar style"
        );
    }
}
