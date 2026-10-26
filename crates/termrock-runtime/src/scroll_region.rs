use core::fmt;
use core::ops::Range;

use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::Modifier;

use super::{PartStyle, SlotFn};
use crate::author::VerticalWindow;
use crate::diagnostics::Diagnostic;
use crate::hit::{Axes, ScrollRouteKey};
use crate::id::{Id, Part, PartRef};
use crate::intent::{Intent, Phase};
use crate::measure::{Constraints, Size};
use crate::response::{Response, StateFlags};
use crate::runtime::WheelRouteResult;
use crate::scroll::ScrollState;
use crate::theme::{Family, GlyphRole, Slot, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, LayoutFacts, Ui};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollbarGutter {
    #[default]
    Auto,

    Stable,
}

pub fn position_label(scroll: &ScrollState) -> String {
    if !scroll.overflows() {
        return String::new();
    }
    let visible = scroll.visible_range();
    if visible.is_empty() {
        return String::new();
    }
    format!(
        "{}–{} of {}",
        visible.start.saturating_add(1),
        visible.end,
        scroll.content_len()
    )
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

#[derive(Clone, Copy)]
enum ScrollRegistration {
    Suppressed,
    Ordinary,
    Routed { source_revision: u64 },
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

    pub fn update_routed(
        &self,
        cx: &mut Cx<'_>,
        st: &mut ScrollState,
        content_len: usize,
        source_revision: u64,
    ) -> Response<()> {
        let Some(attempt) =
            cx.wheel_route_attempt(self.id, PartRef::of(Part::CONTAINER), crate::event::Axis::V)
        else {
            return self.update(cx, st, content_len);
        };
        self.service_routed_wheel_attempt(cx, st, content_len, source_revision, attempt)
    }

    pub fn service_routed_wheel(
        &self,
        cx: &mut Cx<'_>,
        st: &mut ScrollState,
        content_len: usize,
        source_revision: u64,
    ) -> Response<()> {
        let Some(attempt) =
            cx.wheel_route_attempt(self.id, PartRef::of(Part::CONTAINER), crate::event::Axis::V)
        else {
            return Response::ignored().for_id(self.id);
        };
        self.service_routed_wheel_attempt(cx, st, content_len, source_revision, attempt)
    }

    fn service_routed_wheel_attempt(
        &self,
        cx: &mut Cx<'_>,
        st: &mut ScrollState,
        content_len: usize,
        source_revision: u64,
        attempt: crate::runtime::WheelRouteAttempt,
    ) -> Response<()> {
        if attempt.source_revision() != source_revision {
            return Response::ignored().for_id(self.id);
        }

        let before_prepare = *st;
        self.prepare(cx, st, content_len);
        let prepared_offset = st.offset();
        let overflows = st.overflows();
        let wheel_response = st.wheel(attempt.delta());
        let changed = before_prepare.offset() != prepared_offset || prepared_offset != st.offset();
        let result = if changed {
            WheelRouteResult::Changed
        } else if overflows {
            WheelRouteResult::Consumed
        } else {
            WheelRouteResult::Ignored
        };
        let response = if before_prepare.offset() != prepared_offset {
            wheel_response | Response::changed()
        } else {
            wheel_response
        };
        if !cx.acknowledge_wheel_route(&attempt, result) {
            *st = before_prepare;
            return Response::ignored().for_id(self.id);
        }
        response.for_id(self.id)
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

    pub fn draw_without_wheel_registration(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        st: &ScrollState,
        content_len: usize,
    ) -> Rect {
        if area.is_empty() {
            return area;
        }
        self.draw_window_registered(
            ui,
            VerticalWindow::full(area),
            st,
            content_len,
            ScrollRegistration::Suppressed,
        )
        .0
    }

    pub fn draw_routed(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        st: &ScrollState,
        content_len: usize,
        source_revision: u64,
    ) -> (Rect, Option<ScrollRouteKey>) {
        if area.is_empty() {
            return (area, None);
        }
        self.draw_window_routed(
            ui,
            VerticalWindow::full(area),
            st,
            content_len,
            source_revision,
        )
    }

    pub fn draw_window(
        &self,
        ui: &mut Ui<'_>,
        window: VerticalWindow,
        st: &ScrollState,
        content_len: usize,
    ) -> Rect {
        self.draw_window_registered(ui, window, st, content_len, ScrollRegistration::Ordinary)
            .0
    }

    pub fn draw_window_routed(
        &self,
        ui: &mut Ui<'_>,
        window: VerticalWindow,
        st: &ScrollState,
        content_len: usize,
        source_revision: u64,
    ) -> (Rect, Option<ScrollRouteKey>) {
        self.draw_window_registered(
            ui,
            window,
            st,
            content_len,
            ScrollRegistration::Routed { source_revision },
        )
    }

    fn draw_window_registered(
        &self,
        ui: &mut Ui<'_>,
        window: VerticalWindow,
        st: &ScrollState,
        content_len: usize,
        registration: ScrollRegistration,
    ) -> (Rect, Option<ScrollRouteKey>) {
        let area = window.visible_body();
        if area.is_empty() {
            ui.report_layout(
                self.id,
                LayoutFacts::new(window.logical_rows(), content_len, 0, area.width)
                    .with_vertical_window(window),
            );
            return (area, None);
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
        let route_key = match registration {
            ScrollRegistration::Suppressed => None,
            ScrollRegistration::Ordinary => {
                ui.register_scroll(self.id, area, Axes::V, view.headroom_v());
                None
            }
            ScrollRegistration::Routed { source_revision } => ui.register_routed_scroll(
                self.id,
                area,
                Axes::V,
                view.headroom_v(),
                source_revision,
            ),
        };
        if !self.scrollbar_visible {
            return (area, route_key);
        }
        let overflows = view.overflows();
        if !overflows && self.scrollbar_gutter == ScrollbarGutter::Auto {
            return (area, route_key);
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
        (content, route_key)
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
