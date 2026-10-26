#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::too_many_lines,
    reason = "focused native geometry assertions"
)]

use core::cell::Cell;

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::Modifier;
use termrock_collections::ScrollState;
use termrock_runtime::Color;
use termrock_runtime::author::VerticalWindow;
use termrock_runtime::hit::ScrollRouteKey;
use termrock_runtime::runtime::{UpdateCause, WheelRouteAttempt};
use termrock_runtime::ui::LayoutFacts;
use termrock_runtime::{
    App, Axis, Cx, FrameRead, Id, Intent, KeyCode, MouseKind, Part, PartRef, Phase, Response,
    ScrollRegion, Ui,
};
use termrock_test_support::{Harness, Scene};
use termrock_theme::{ColorLevel, Theme};
use termrock_viewport::viewport::{
    CellPos, ProjectedText, TextViewport, ViewportLine, ViewportState,
};

const OWNER: Id = Id::root("viewport.window.owner");
const PARENT: Id = Id::root("viewport.window.parent");
const SCREEN: Rect = Rect::new(0, 0, 32, 16);
const BODY: Rect = Rect::new(2, 1, 18, 8);
const LINES: [&str; 20] = [
    "ab e\u{301}界👩\u{200d}💻\tend",
    "row-01 abcdefghijklmnopqrstuvwxyz",
    "row-02 abcdefghijklmnopqrstuvwxyz",
    "row-03 abcdefghijklmnopqrstuvwxyz",
    "row-04 abcdefghijklmnopqrstuvwxyz",
    "row-05 abcdefghijklmnopqrstuvwxyz",
    "row-06 abcdefghijklmnopqrstuvwxyz",
    "row-07 abcdefghijklmnopqrstuvwxyz",
    "row-08 abcdefghijklmnopqrstuvwxyz",
    "row-09 abcdefghijklmnopqrstuvwxyz",
    "row-10 abcdefghijklmnopqrstuvwxyz",
    "row-11 abcdefghijklmnopqrstuvwxyz",
    "row-12 abcdefghijklmnopqrstuvwxyz",
    "row-13 abcdefghijklmnopqrstuvwxyz",
    "row-14 abcdefghijklmnopqrstuvwxyz",
    "row-15 abcdefghijklmnopqrstuvwxyz",
    "row-16 abcdefghijklmnopqrstuvwxyz",
    "row-17 abcdefghijklmnopqrstuvwxyz",
    "row-18 abcdefghijklmnopqrstuvwxyz",
    "row-19 abcdefghijklmnopqrstuvwxyz",
];

const HORIZONTAL_TRACK: Part = Part::custom("viewport.horizontal-track");
const HORIZONTAL_THUMB: Part = Part::custom("viewport.horizontal-thumb");

#[derive(Clone, Copy)]
struct WindowSpec {
    area: Rect,
    logical_rows: usize,
    first_source_row: usize,
}

impl WindowSpec {
    fn window(self) -> VerticalWindow {
        if self.area.is_empty() {
            VerticalWindow::full(self.area)
        } else {
            VerticalWindow::new(self.area, self.logical_rows, self.first_source_row)
                .expect("test window maps within its logical rows")
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum DrawApi {
    Legacy,
    Window,
}

#[derive(Clone, Copy, Debug)]
enum TextSource {
    Borrowed,
    Projected,
}

#[derive(Clone, Copy)]
enum PendingEventMutation {
    ProjectedSource { width: usize, layout_revision: u64 },
    SourceRevision(u64),
}

struct WindowPage {
    state: ViewportState,
    parent: ScrollState,
    window: WindowSpec,
    candidate_window: WindowSpec,
    draw_candidate_once: Cell<bool>,
    draw_api: DrawApi,
    clip: Option<Rect>,
    source: TextSource,
    wrap: bool,
    pan: bool,
    reserve_gutter: bool,
    layout_revision: u64,
    routed: bool,
    return_view: bool,
    fit_window_to_viewport: bool,
    revision: u64,
    projected: ProjectedText,
    seen_facts: Cell<Option<LayoutFacts>>,
    seen_return: Cell<Option<Rect>>,
    seen_view: Cell<Option<Option<ScrollState>>>,
    route_key: Cell<Option<ScrollRouteKey>>,
    seen_capture: Cell<Option<Id>>,
    pending_event_mutation: Cell<Option<PendingEventMutation>>,
    pending_mutation_cause: Cell<Option<UpdateCause>>,
    seen_track_event: Cell<Option<(Rect, Position, usize)>>,
    seen_route_revisions: Cell<Option<(u64, u64)>>,
    parent_route_calls: Cell<usize>,
}

impl WindowPage {
    fn new(window: WindowSpec) -> Self {
        let mut state = ViewportState::default();
        state.set_follow(false);
        Self {
            state,
            parent: ScrollState::new(200),
            window,
            candidate_window: window,
            draw_candidate_once: Cell::new(false),
            draw_api: DrawApi::Window,
            clip: None,
            source: TextSource::Borrowed,
            wrap: false,
            pan: false,
            reserve_gutter: false,
            layout_revision: 3,
            routed: false,
            return_view: false,
            fit_window_to_viewport: false,
            revision: 11,
            projected: projection(),
            seen_facts: Cell::new(None),
            seen_return: Cell::new(None),
            seen_view: Cell::new(None),
            route_key: Cell::new(None),
            seen_capture: Cell::new(None),
            pending_event_mutation: Cell::new(None),
            pending_mutation_cause: Cell::new(None),
            seen_track_event: Cell::new(None),
            seen_route_revisions: Cell::new(None),
            parent_route_calls: Cell::new(0),
        }
    }

    fn viewport(&self) -> TextViewport<'static> {
        TextViewport::new(OWNER)
            .reserve_gutter(self.reserve_gutter)
            .wrap(self.wrap)
            .horizontal_pan(self.pan)
            .layout_revision(self.layout_revision)
    }

    fn update_viewport(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let viewport = self.viewport();
        match (self.routed, self.source) {
            (true, TextSource::Borrowed) => viewport
                .update_routed(cx, &mut self.state, &borrowed_lines(), self.revision)
                .erase(),
            (true, TextSource::Projected) => viewport
                .update_projected_routed(cx, &mut self.state, &self.projected, self.revision)
                .erase(),
            (false, TextSource::Borrowed) => viewport
                .update(cx, &mut self.state, &borrowed_lines())
                .erase(),
            (false, TextSource::Projected) => viewport
                .update_projected(cx, &mut self.state, &self.projected)
                .erase(),
        }
    }

    fn draw_viewport(&self, ui: &mut Ui<'_>) {
        if let Some(clip) = self.clip {
            ui.with_area(clip, |ui| self.draw_viewport_contents(ui));
        } else {
            self.draw_viewport_contents(ui);
        }
    }

    fn draw_viewport_contents(&self, ui: &mut Ui<'_>) {
        let spec = if self.draw_candidate_once.replace(false) {
            self.candidate_window
        } else {
            self.window
        };
        let viewport = self.viewport();
        let mut projected_view = None;
        let (text, route_key) = match (self.draw_api, self.routed, self.source) {
            (DrawApi::Legacy, false, TextSource::Borrowed) => (
                viewport.draw(ui, spec.area, &self.state, &borrowed_lines()),
                None,
            ),
            (DrawApi::Window, false, TextSource::Borrowed) => (
                viewport.draw_window(ui, spec.window(), &self.state, &borrowed_lines()),
                None,
            ),
            (DrawApi::Legacy, true, TextSource::Borrowed) => {
                viewport.draw_routed(ui, spec.area, &self.state, &borrowed_lines(), self.revision)
            }
            (DrawApi::Window, true, TextSource::Borrowed) if self.return_view => {
                let (text, key, view) = viewport.draw_window_routed_with_view(
                    ui,
                    spec.window(),
                    &self.state,
                    &borrowed_lines(),
                    self.revision,
                );
                projected_view = Some(view);
                (text, key)
            }
            (DrawApi::Window, true, TextSource::Borrowed) => viewport.draw_window_routed(
                ui,
                spec.window(),
                &self.state,
                &borrowed_lines(),
                self.revision,
            ),
            (DrawApi::Legacy, false, TextSource::Projected) => (
                viewport.draw_projected(ui, spec.area, &self.state, &self.projected),
                None,
            ),
            (DrawApi::Window, false, TextSource::Projected) => (
                viewport.draw_projected_window(ui, spec.window(), &self.state, &self.projected),
                None,
            ),
            (DrawApi::Legacy, true, TextSource::Projected) => viewport.draw_projected_routed(
                ui,
                spec.area,
                &self.state,
                &self.projected,
                self.revision,
            ),
            (DrawApi::Window, true, TextSource::Projected) if self.return_view => {
                let (text, key, view) = viewport.draw_projected_window_routed_with_view(
                    ui,
                    spec.window(),
                    &self.state,
                    &self.projected,
                    self.revision,
                );
                projected_view = Some(view);
                (text, key)
            }
            (DrawApi::Window, true, TextSource::Projected) => viewport
                .draw_projected_window_routed(
                    ui,
                    spec.window(),
                    &self.state,
                    &self.projected,
                    self.revision,
                ),
        };
        self.seen_return.set(Some(text));
        self.seen_view.set(projected_view);
        self.route_key.set(route_key);
    }
}

impl App for WindowPage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if self.fit_window_to_viewport {
            let height = cx.viewport().height.saturating_sub(2);
            self.window = WindowSpec {
                area: Rect::new(2, 1, 18, height),
                logical_rows: usize::from(height),
                first_source_row: 0,
            };
        }
        let committed_facts = cx.layout(OWNER);
        self.seen_facts.set(committed_facts);
        let committed_track = cx.area_of_part(OWNER, PartRef::of(Part::TRACK));
        let track_press = cx.intents(OWNER).find_map(|intent| match intent {
            Intent::Pointer {
                phase: Phase::Press,
                part,
                pos,
                ..
            } if part == PartRef::of(Part::TRACK) => Some(pos),
            _ => None,
        });
        self.seen_track_event
            .set(committed_track.zip(track_press).and_then(|(track, pos)| {
                committed_facts?
                    .vertical_window?
                    .source_row_for_physical(pos.y)
                    .map(|source_row| (track, pos, source_row))
            }));
        let route_attempt_revision = cx
            .wheel_route_attempt(OWNER, PartRef::of(Part::CONTAINER), Axis::V)
            .map(|attempt| attempt.source_revision());
        if cx.update_cause() == UpdateCause::Event
            && let Some(mutation) = self.pending_event_mutation.take()
        {
            match mutation {
                PendingEventMutation::ProjectedSource {
                    width,
                    layout_revision,
                } => {
                    self.projected = projection_with_width(width);
                    self.layout_revision = layout_revision;
                }
                PendingEventMutation::SourceRevision(revision) => {
                    self.revision = revision;
                }
            }
            self.pending_mutation_cause.set(Some(cx.update_cause()));
        }
        if let Some(attempt_revision) = route_attempt_revision {
            self.seen_route_revisions
                .set(Some((attempt_revision, self.revision)));
        }
        let mut response = if self.routed {
            ScrollRegion::new(PARENT)
                .scrollbar_visible(false)
                .update_routed(cx, &mut self.parent, 200, 19)
        } else {
            Response::ignored().for_id(PARENT)
        };
        response |= self.update_viewport(cx);
        self.seen_capture.set(cx.capture_owner());
        response
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        if self.routed {
            let parent = ScrollRegion::new(PARENT).scrollbar_visible(false);
            let (_, parent_key) = parent.draw_routed(ui, ui.full(), &self.parent, 200, 19);
            self.draw_viewport(ui);
            if let (Some(child), Some(parent)) = (self.route_key.get(), parent_key) {
                let _ = ui.link_scroll_parent(child, parent);
            }
        } else {
            self.draw_viewport(ui);
        }
    }

    fn route_wheel_attempt(&mut self, cx: &mut Cx<'_>, attempt: WheelRouteAttempt) -> Response<()> {
        if attempt.owner() == OWNER {
            match self.source {
                TextSource::Borrowed => self
                    .viewport()
                    .service_routed_wheel(cx, &mut self.state, &borrowed_lines(), self.revision)
                    .erase(),
                TextSource::Projected => self
                    .viewport()
                    .service_projected_routed_wheel(
                        cx,
                        &mut self.state,
                        &self.projected,
                        self.revision,
                    )
                    .erase(),
            }
        } else if attempt.owner() == PARENT {
            self.parent_route_calls
                .set(self.parent_route_calls.get().saturating_add(1));
            ScrollRegion::new(PARENT).service_routed_wheel(cx, &mut self.parent, 200, 19)
        } else {
            Response::ignored()
        }
    }
}

fn borrowed_lines() -> Vec<ViewportLine<'static>> {
    LINES.into_iter().map(ViewportLine::Plain).collect()
}

fn projection() -> ProjectedText {
    let mut projected = ProjectedText::default();
    for (index, line) in LINES.into_iter().enumerate() {
        if index == 0 {
            projected.push_line([
                ("ab e", None, Modifier::BOLD),
                ("\u{301}界", None, Modifier::ITALIC),
                ("👩\u{200d}💻\tend", None, Modifier::empty()),
            ]);
        } else {
            projected.push_line([(line, None, Modifier::empty())]);
        }
    }
    projected
}

fn projection_with_width(width: usize) -> ProjectedText {
    let mut projected = ProjectedText::default();
    projected.push_line([("x".repeat(width), None, Modifier::empty())]);
    for _ in 1..LINES.len() {
        projected.push_line([("r", None, Modifier::empty())]);
    }
    projected
}

fn projection_with_row_count(rows: usize) -> ProjectedText {
    let mut projected = ProjectedText::default();
    for _ in 0..rows {
        projected.push_line([("row", None, Modifier::empty())]);
    }
    projected
}

fn bar_only_page(width: usize) -> Harness<WindowPage> {
    let mut app = WindowPage::new(WindowSpec {
        area: Rect::new(2, 4, 18, 1),
        logical_rows: 8,
        first_source_row: 7,
    });
    app.pan = true;
    app.source = TextSource::Projected;
    app.projected = projection_with_width(width);
    let mut h = page_with_app(app);
    h.draw();
    let _ = h.tick();
    h
}

fn page(spec: WindowSpec) -> Harness<WindowPage> {
    Harness::new(
        WindowPage::new(spec),
        Theme::junie(),
        SCREEN.width,
        SCREEN.height,
    )
}

fn full_spec(area: Rect) -> WindowSpec {
    WindowSpec {
        area,
        logical_rows: usize::from(area.height),
        first_source_row: 0,
    }
}

fn assert_window_cells_match_full_rows(
    full: &Harness<WindowPage>,
    clipped: &Harness<WindowPage>,
    source_rows: core::ops::Range<u16>,
) {
    for y in source_rows {
        for x in 2..19 {
            assert_eq!(
                full.cell(x, y).symbol(),
                clipped.cell(x, y).symbol(),
                "visible wrapped/source row differs at ({x}, {y})"
            );
            assert_eq!(
                full.cell(x, y).fg,
                clipped.cell(x, y).fg,
                "visible source style differs at ({x}, {y})"
            );
        }
    }
}

#[test]
fn full_window_methods_match_all_legacy_wrapper_pairs() {
    for source in [TextSource::Borrowed, TextSource::Projected] {
        for routed in [false, true] {
            let make = |draw_api| {
                let mut app = WindowPage::new(full_spec(BODY));
                app.draw_api = draw_api;
                app.source = source;
                app.routed = routed;
                app.pan = true;
                page_with_app(app)
            };
            let mut legacy = make(DrawApi::Legacy);
            let mut windowed = make(DrawApi::Window);
            legacy.draw();
            windowed.draw();
            let _ = legacy.tick();
            let _ = windowed.tick();
            assert_eq!(legacy.buffer(), windowed.buffer());
            assert_eq!(
                legacy.app().seen_return.get(),
                windowed.app().seen_return.get()
            );
            assert_eq!(legacy.area_of(OWNER), windowed.area_of(OWNER));
            for part in [
                Part::CONTAINER,
                Part::TEXT,
                Part::GUTTER,
                Part::TRACK,
                Part::THUMB,
            ] {
                assert_eq!(
                    legacy.area_of_part(OWNER, PartRef::of(part)),
                    windowed.area_of_part(OWNER, PartRef::of(part)),
                    "part {part:?}, source={source:?}, routed={routed}"
                );
            }
            for part in [HORIZONTAL_TRACK, HORIZONTAL_THUMB] {
                assert_eq!(
                    legacy.area_of_part(OWNER, PartRef::of(part)),
                    windowed.area_of_part(OWNER, PartRef::of(part))
                );
            }
            assert_eq!(legacy.cursor(), windowed.cursor());
            assert_eq!(
                legacy.app().seen_facts.get(),
                windowed.app().seen_facts.get()
            );
            assert_eq!(
                legacy.app().state.scroll().viewport_len(),
                windowed.app().state.scroll().viewport_len()
            );
            assert_eq!(
                legacy.app().state.scroll().offset(),
                windowed.app().state.scroll().offset()
            );
            assert_eq!(
                legacy.app().state.horizontal_offset(),
                windowed.app().state.horizontal_offset()
            );
            if routed {
                assert!(legacy.app().route_key.get().is_some());
                assert!(windowed.app().route_key.get().is_some());
            }
        }
    }
}

#[test]
fn routed_window_view_matches_legacy_draw_for_both_sources_and_follow_modes() {
    let spec = WindowSpec {
        area: Rect::new(2, 2, 18, 3),
        logical_rows: 8,
        first_source_row: 2,
    };
    for source in [TextSource::Borrowed, TextSource::Projected] {
        for follow in [false, true] {
            let make = |return_view| {
                let mut app = WindowPage::new(spec);
                app.routed = true;
                app.return_view = return_view;
                app.source = source;
                app.state.set_follow(follow);
                page_with_app(app)
            };
            let mut legacy = make(false);
            let mut with_view = make(true);

            let state_before_draw = with_view.app().state.clone();
            legacy.draw();
            with_view.draw();
            assert_eq!(
                with_view.app().state,
                state_before_draw,
                "drawing exposes a projection without mutating caller state"
            );
            assert_eq!(legacy.buffer(), with_view.buffer());
            assert_eq!(
                legacy.app().seen_return.get(),
                with_view.app().seen_return.get()
            );
            assert_eq!(legacy.app().seen_view.get(), None);
            assert_eq!(legacy.area_of(OWNER), with_view.area_of(OWNER));
            assert_eq!(legacy.cursor(), with_view.cursor());
            assert_eq!(
                legacy.app().seen_facts.get(),
                with_view.app().seen_facts.get()
            );
            assert_eq!(
                legacy.app().route_key.get().is_some(),
                with_view.app().route_key.get().is_some()
            );
            for part in [
                Part::CONTAINER,
                Part::TEXT,
                Part::GUTTER,
                Part::TRACK,
                Part::THUMB,
            ] {
                assert_eq!(
                    legacy.area_of_part(OWNER, PartRef::of(part)),
                    with_view.area_of_part(OWNER, PartRef::of(part)),
                    "part {part:?}, source={source:?}, follow={follow}"
                );
            }
            for part in [HORIZONTAL_TRACK, HORIZONTAL_THUMB] {
                assert_eq!(
                    legacy.area_of_part(OWNER, PartRef::of(part)),
                    with_view.area_of_part(OWNER, PartRef::of(part))
                );
            }

            let mut expected = *legacy.app().state.scroll();
            expected.apply_layout(spec.logical_rows, LINES.len());
            if follow {
                expected.jump_end();
            }
            assert_eq!(with_view.app().seen_view.get(), Some(Some(expected)));
            assert_eq!(expected.viewport_len(), 8);
            assert_eq!(expected.content_len(), 20);
            assert_eq!(
                expected.offset(),
                if follow {
                    expected.max_offset()
                } else {
                    legacy
                        .app()
                        .state
                        .scroll()
                        .offset()
                        .min(expected.max_offset())
                }
            );

            let _ = legacy.wheel(Axis::V, 2, 3, 2);
            let _ = with_view.wheel(Axis::V, 2, 3, 2);
            assert_eq!(legacy.app().state, with_view.app().state);
            assert_eq!(legacy.app().parent, with_view.app().parent);
            assert_eq!(
                legacy.app().parent_route_calls.get(),
                with_view.app().parent_route_calls.get()
            );
            assert_eq!(
                legacy.app().seen_capture.get(),
                with_view.app().seen_capture.get()
            );

            let legacy_track = legacy.area_of_part(OWNER, PartRef::of(Part::TRACK));
            let projected_track = with_view.area_of_part(OWNER, PartRef::of(Part::TRACK));
            assert_eq!(legacy_track, projected_track);
            if let Some(track) = legacy_track {
                let y = track.y.saturating_add(track.height / 2);
                let _ = legacy.click(track.x, y);
                let _ = with_view.click(track.x, y);
                assert_eq!(legacy.app().state, with_view.app().state);
            }

            let _ = legacy.mouse(MouseKind::Down, 3, 2);
            let _ = with_view.mouse(MouseKind::Down, 3, 2);
            assert_eq!(legacy.app().seen_capture.get(), Some(OWNER));
            assert_eq!(with_view.app().seen_capture.get(), Some(OWNER));
            let _ = legacy.mouse(MouseKind::Drag, 9, 2);
            let _ = with_view.mouse(MouseKind::Drag, 9, 2);
            assert!(legacy.app().state.selection().is_some());
            assert_eq!(legacy.app().state, with_view.app().state);
            let held_legacy_state = legacy.app().state.clone();
            let held_view_state = with_view.app().state.clone();
            let held_legacy_parent = legacy.app().parent;
            let held_view_parent = with_view.app().parent;
            let held_legacy_source = legacy.app().projected.clone();
            let held_view_source = with_view.app().projected.clone();
            legacy.draw();
            with_view.draw();
            assert_eq!(legacy.app().state, held_legacy_state);
            assert_eq!(with_view.app().state, held_view_state);
            assert_eq!(legacy.app().parent, held_legacy_parent);
            assert_eq!(with_view.app().parent, held_view_parent);
            assert_eq!(legacy.app().projected, held_legacy_source);
            assert_eq!(with_view.app().projected, held_view_source);
            assert_eq!(legacy.app().state, with_view.app().state);
            assert_eq!(legacy.app().parent, with_view.app().parent);
            assert_eq!(
                legacy.app().state.selection(),
                with_view.app().state.selection()
            );

            let _ = legacy.mouse(MouseKind::Drag, 10, 2);
            let _ = with_view.mouse(MouseKind::Drag, 10, 2);
            assert_eq!(legacy.app().seen_capture.get(), Some(OWNER));
            assert_eq!(with_view.app().seen_capture.get(), Some(OWNER));
            assert_eq!(legacy.app().state, with_view.app().state);
            let _ = legacy.mouse(MouseKind::Up, 9, 2);
            let _ = with_view.mouse(MouseKind::Up, 9, 2);
            assert_eq!(legacy.app().seen_capture.get(), None);
            assert_eq!(with_view.app().seen_capture.get(), None);
            let mut legacy_copy = String::new();
            let mut view_copy = String::new();
            let legacy_copied = match source {
                TextSource::Borrowed => legacy
                    .app()
                    .state
                    .copy_into(&borrowed_lines(), &mut legacy_copy),
                TextSource::Projected => legacy
                    .app()
                    .state
                    .copy_from_projected(&legacy.app().projected, &mut legacy_copy),
            };
            let view_copied = match source {
                TextSource::Borrowed => with_view
                    .app()
                    .state
                    .copy_into(&borrowed_lines(), &mut view_copy),
                TextSource::Projected => with_view
                    .app()
                    .state
                    .copy_from_projected(&with_view.app().projected, &mut view_copy),
            };
            assert!(legacy_copied);
            assert!(view_copied);
            assert_eq!(legacy_copy, view_copy);
        }
    }
}

#[test]
fn routed_window_view_projects_resize_and_content_shrink_without_mutating_state() {
    let mut app = WindowPage::new(full_spec(BODY));
    app.routed = true;
    app.return_view = true;
    app.fit_window_to_viewport = true;
    app.state.set_follow(false);
    let mut h = Harness::new(app, Theme::junie(), SCREEN.width, 10).with_auto_draw(false);
    let _ = h.tick();
    h.draw();
    let initial = h
        .app()
        .seen_view
        .get()
        .flatten()
        .expect("first nonempty draw returns its projected view");
    assert_eq!(initial.viewport_len(), 8);
    assert_eq!(initial.content_len(), 20);

    let _ = h.wheel(Axis::V, 20, 3, 2);
    assert_eq!(h.app().state.scroll().offset(), 12);
    let _ = h.resize(SCREEN.width, SCREEN.height);
    let before_resized_draw = h.app().state.clone();
    h.draw();
    let resized = h
        .app()
        .seen_view
        .get()
        .flatten()
        .expect("resized nonempty draw returns its projected view");
    assert_eq!(resized.viewport_len(), 14);
    assert_eq!(resized.content_len(), 20);
    assert_eq!(
        resized.offset(),
        6,
        "the draw-time view clamps to the larger viewport"
    );
    assert_eq!(
        h.app().state,
        before_resized_draw,
        "drawing the resized layout does not reconcile caller state"
    );

    h.app_mut().source = TextSource::Projected;
    h.app_mut().projected = projection_with_row_count(5);
    let revision = h.app().revision.saturating_add(1);
    let layout_revision = h.app().layout_revision.saturating_add(1);
    h.app_mut().revision = revision;
    h.app_mut().layout_revision = layout_revision;
    let before_shrink_draw = h.app().state.clone();
    h.draw();
    let shrunk = h
        .app()
        .seen_view
        .get()
        .flatten()
        .expect("content shrink remains a nonempty draw");
    assert_eq!(shrunk.viewport_len(), 14);
    assert_eq!(shrunk.content_len(), 5);
    assert_eq!(
        shrunk.offset(),
        0,
        "the projected view clamps after content shrink"
    );
    assert_eq!(h.app().state, before_shrink_draw);
}

#[test]
fn routed_window_view_distinguishes_empty_body_from_zero_text_width() {
    let mut empty = WindowPage::new(WindowSpec {
        area: Rect::new(2, 2, 0, 4),
        logical_rows: 8,
        first_source_row: 0,
    });
    empty.routed = true;
    empty.return_view = true;
    let empty = page_with_app(empty);
    assert_eq!(empty.app().seen_view.get(), Some(None));

    let mut zero_width = WindowPage::new(full_spec(Rect::new(2, 2, 1, 4)));
    zero_width.routed = true;
    zero_width.return_view = true;
    zero_width.reserve_gutter = true;
    let zero_width = page_with_app(zero_width);
    let projected = zero_width
        .app()
        .seen_view
        .get()
        .flatten()
        .expect("a nonempty body returns a view even when its text width is zero");
    assert_eq!(projected.viewport_len(), 4);
    assert_eq!(projected.content_len(), 20);
    assert_eq!(zero_width.app().route_key.get(), None);
}

#[test]
fn legacy_rect_baseline_geometry_has_an_independent_literal_oracle() {
    let mut legacy = WindowPage::new(full_spec(BODY));
    legacy.draw_api = DrawApi::Legacy;
    let mut h = page_with_app(legacy);
    h.draw();
    assert_eq!(h.app().seen_return.get(), Some(Rect::new(2, 1, 17, 8)));
    assert_eq!(
        h.area_of_part(OWNER, PartRef::of(Part::TRACK)),
        Some(Rect::new(19, 1, 1, 8))
    );
    assert_eq!(h.cell(2, 1).symbol(), "a");
    assert_eq!(h.cell(3, 1).symbol(), "b");
    assert_eq!(
        h.app()
            .seen_facts
            .get()
            .and_then(|facts| facts.logical_area),
        Some(BODY),
        "the pre-window Rect layout fact remains the full body"
    );
}

#[test]
fn gutter_consuming_text_width_preserves_the_legacy_empty_rect_origin_and_height() {
    let area = Rect::new(3, 2, 1, 6);
    let mut app = WindowPage::new(full_spec(area));
    app.draw_api = DrawApi::Legacy;
    app.reserve_gutter = true;
    let mut h = page_with_app(app);
    h.draw();

    assert_eq!(
        h.app().seen_return.get(),
        Some(Rect::new(4, 2, 0, 6)),
        "the old Rect path retained the text origin and logical row height"
    );
    assert_eq!(h.area_of_part(OWNER, PartRef::of(Part::TEXT)), None);
    assert_eq!(h.area_of_part(OWNER, PartRef::of(Part::TRACK)), None);
}

fn page_with_app(app: WindowPage) -> Harness<WindowPage> {
    Harness::new(app, Theme::junie(), SCREEN.width, SCREEN.height)
}

#[test]
fn partial_window_keeps_wrapped_unicode_rows_selection_and_document_copy() {
    let full_area = Rect::new(2, 0, 18, 8);
    let clipped_spec = WindowSpec {
        area: Rect::new(2, 2, 18, 3),
        logical_rows: 8,
        first_source_row: 2,
    };
    let mut full_app = WindowPage::new(full_spec(full_area));
    full_app.wrap = true;
    full_app.source = TextSource::Projected;
    full_app
        .state
        .select(CellPos::new(0, 2), CellPos::new(2, 4));
    let mut clipped_app = WindowPage::new(clipped_spec);
    clipped_app.wrap = true;
    clipped_app.source = TextSource::Projected;
    clipped_app
        .state
        .select(CellPos::new(0, 2), CellPos::new(2, 4));
    let mut full = page_with_app(full_app);
    let mut clipped = page_with_app(clipped_app);
    full.draw();
    clipped.draw();
    let _ = full.tick();
    let _ = clipped.tick();

    assert_window_cells_match_full_rows(&full, &clipped, 2..5);
    assert_eq!(
        clipped
            .app()
            .seen_facts
            .get()
            .map(|facts| facts.viewport_len),
        Some(8),
        "a three-row physical fragment retains the eight-row logical text viewport"
    );
    assert_eq!(
        clipped
            .app()
            .seen_facts
            .get()
            .and_then(|facts| facts.vertical_window),
        Some(clipped_spec.window())
    );
    assert_eq!(
        clipped.app().state.selection(),
        Some((CellPos::new(0, 2), CellPos::new(2, 4)))
    );
    let mut copied = String::new();
    assert!(
        clipped
            .app()
            .state
            .copy_from_projected(&clipped.app().projected, &mut copied)
    );
    let mut full_copy = String::new();
    assert!(
        full.app()
            .state
            .copy_from_projected(&full.app().projected, &mut full_copy)
    );
    assert_eq!(copied, full_copy);
}

#[test]
fn huge_logical_window_keeps_usize_rows_without_phantom_origin_arithmetic() {
    let spec = WindowSpec {
        area: Rect::new(2, 3, 18, 2),
        logical_rows: usize::MAX,
        first_source_row: usize::MAX - 3,
    };
    let mut h = page(spec);
    h.draw();
    let _ = h.tick();
    assert_eq!(h.app().state.scroll().viewport_len(), usize::MAX);
    assert_eq!(
        h.app()
            .seen_facts
            .get()
            .and_then(|facts| facts.vertical_body_window)
            .map(VerticalWindow::logical_rows),
        Some(usize::MAX)
    );
    assert_eq!(h.area_of(OWNER), Some(spec.area));
    assert_eq!(h.cell(spec.area.x, spec.area.y).symbol(), " ");
}

#[test]
fn clipped_pointer_selection_and_caret_map_through_parent_source_rows() {
    let clipped_spec = WindowSpec {
        area: Rect::new(2, 2, 18, 3),
        logical_rows: 8,
        first_source_row: 2,
    };
    let mut clipped_app = WindowPage::new(clipped_spec);
    clipped_app.state.set_caret(Some(CellPos::new(5, 1)));
    let mut full_app = WindowPage::new(full_spec(Rect::new(2, 0, 18, 8)));
    full_app.state.set_caret(Some(CellPos::new(5, 1)));
    let mut clipped = page_with_app(clipped_app);
    let mut full = page_with_app(full_app);
    clipped.draw();
    full.draw();

    assert!(clipped.tab_to(OWNER));
    assert!(full.tab_to(OWNER));
    assert_eq!(
        clipped.cursor(),
        None,
        "the source caret is initially hidden"
    );
    assert_eq!(full.cursor(), Some(Position::new(3, 5)));
    for _ in 0..3 {
        let _ = clipped.key(KeyCode::Down);
        let _ = full.key(KeyCode::Down);
    }
    assert_eq!(clipped.app().state.scroll().offset(), 3);
    assert_eq!(full.app().state.scroll().offset(), 3);
    assert_eq!(clipped.cursor(), Some(Position::new(3, 2)));
    assert_eq!(full.cursor(), Some(Position::new(3, 2)));

    let _ = clipped.drag((2, 2), (8, 2));
    let _ = full.drag((2, 2), (8, 2));
    let selected = Some((CellPos::new(5, 0), CellPos::new(5, 6)));
    assert_eq!(clipped.app().state.selection(), selected);
    assert_eq!(full.app().state.selection(), selected);
    let mut copied = String::new();
    assert!(
        clipped
            .app()
            .state
            .copy_into(&borrowed_lines(), &mut copied)
    );
    assert_eq!(copied, "row-05");

    let _ = clipped.mouse(MouseKind::Down, 2, 2);
    assert_eq!(clipped.app().seen_capture.get(), Some(OWNER));
    let _ = clipped.mouse(MouseKind::Drag, 2, 1);
    assert_eq!(clipped.app().seen_capture.get(), Some(OWNER));
    let _ = clipped.mouse(MouseKind::Up, 2, 15);
    assert_eq!(clipped.app().seen_capture.get(), None);
}

#[test]
fn one_visible_text_row_keeps_source_selection_and_cursor_coordinates() {
    let clipped_spec = WindowSpec {
        area: Rect::new(2, 3, 18, 1),
        logical_rows: 8,
        first_source_row: 4,
    };
    let mut clipped_app = WindowPage::new(clipped_spec);
    clipped_app.state.set_caret(Some(CellPos::new(4, 1)));
    let mut full_app = WindowPage::new(full_spec(Rect::new(2, 0, 18, 8)));
    full_app.state.set_caret(Some(CellPos::new(4, 1)));
    let mut clipped = page_with_app(clipped_app);
    let mut full = page_with_app(full_app);
    clipped.draw();
    full.draw();

    assert!(clipped.tab_to(OWNER));
    assert!(full.tab_to(OWNER));
    assert_eq!(clipped.cursor(), Some(Position::new(3, 3)));
    assert_eq!(full.cursor(), Some(Position::new(3, 4)));
    for x in 2..19 {
        assert_eq!(clipped.cell(x, 3).symbol(), full.cell(x, 4).symbol());
        assert_eq!(clipped.cell(x, 3).fg, full.cell(x, 4).fg);
    }

    let _ = clipped.drag((2, 3), (5, 3));
    let _ = full.drag((2, 4), (5, 4));
    let selection = Some((CellPos::new(4, 0), CellPos::new(4, 3)));
    assert_eq!(clipped.app().state.selection(), selection);
    assert_eq!(full.app().state.selection(), selection);
}

#[test]
fn partial_true_edge_fade_preserves_the_selected_row_paint() {
    let full_window_spec = full_spec(Rect::new(2, 1, 18, 8));
    let clipped_spec = WindowSpec {
        area: Rect::new(2, 8, 18, 1),
        logical_rows: 8,
        first_source_row: 7,
    };
    let mut full_app = WindowPage::new(full_window_spec);
    full_app
        .state
        .select(CellPos::new(7, 0), CellPos::new(7, 5));
    let mut clipped_app = WindowPage::new(clipped_spec);
    clipped_app
        .state
        .select(CellPos::new(7, 0), CellPos::new(7, 5));
    let mut full = page_with_app(full_app);
    let mut clipped = page_with_app(clipped_app);
    full.draw();
    clipped.draw();

    for x in 2..20 {
        assert_eq!(
            clipped.cell(x, 8),
            full.cell(x, 8),
            "selected or unselected style differs at true bottom edge x={x}"
        );
    }
}

#[test]
fn visible_cursor_protects_the_true_top_fade_in_a_partial_window() {
    let mut app = WindowPage::new(WindowSpec {
        area: Rect::new(2, 2, 18, 3),
        logical_rows: 8,
        first_source_row: 0,
    });
    app.state.set_caret(Some(CellPos::new(3, 0)));
    let mut h = page_with_app(app);
    h.draw();
    assert!(h.tab_to(OWNER));
    for _ in 0..3 {
        let _ = h.key(KeyCode::Down);
    }
    h.draw();
    assert_eq!(h.cursor(), Some(Position::new(2, 2)));
    assert_eq!(
        h.cell(2, 2).fg,
        h.cell(2, 3).fg,
        "the cursor-protected true top row keeps the unfaded text ink"
    );
}

#[test]
fn horizontal_route_and_track_follow_the_true_logical_bottom() {
    let mut app = WindowPage::new(full_spec(Rect::new(2, 1, 18, 8)));
    app.pan = true;
    app.routed = true;
    let mut h = page_with_app(app);
    h.draw();
    assert_eq!(
        h.area_of_part(OWNER, PartRef::of(HORIZONTAL_TRACK)),
        Some(Rect::new(2, 8, 17, 1)),
        "horizontal track maps to body source row 7 after reserving one text row"
    );
    let _ = h.wheel(Axis::H, 1, 4, 2);
    assert!(h.app().state.horizontal_offset() > 0);
    assert_eq!(h.app().state.scroll().offset(), 0);
    let before_track_press = h.app().state.horizontal_offset();
    let _ = h.mouse(MouseKind::Down, 18, 8);
    assert!(h.app().state.horizontal_offset() > before_track_press);
    let _ = h.mouse(MouseKind::Up, 18, 8);

    h.app_mut().window = WindowSpec {
        area: Rect::new(2, 2, 18, 3),
        logical_rows: 8,
        first_source_row: 2,
    };
    h.draw();
    assert_eq!(
        h.area_of_part(OWNER, PartRef::of(HORIZONTAL_TRACK)),
        None,
        "a parent cut above logical row 7 must not move the bar to the fragment bottom"
    );
    let before_old_track = h.app().state.horizontal_offset();
    let _ = h.mouse(MouseKind::Down, 18, 8);
    assert_eq!(
        h.app().state.horizontal_offset(),
        before_old_track,
        "the previous horizontal row is not a live pointer target after the window moves"
    );
    let _ = h.wheel(Axis::H, 1, 4, 2);
    assert!(h.app().state.horizontal_offset() > 4);
}

#[test]
fn vertical_track_pointer_uses_logical_rows_for_a_clipped_window() {
    let spec = WindowSpec {
        area: Rect::new(2, 2, 18, 3),
        logical_rows: 8,
        first_source_row: 2,
    };
    let mut h = page(spec);
    h.draw();
    let track = h
        .area_of_part(OWNER, PartRef::of(Part::TRACK))
        .expect("partial text window keeps its visible vertical track");
    assert_eq!(track, Rect::new(19, 2, 1, 3));
    let _ = h.mouse(MouseKind::Down, track.x, track.y.saturating_add(2));
    assert!(h.app().state.scroll().offset() > 0);
    assert!(h.app().state.scroll().offset() < h.app().state.scroll().max_offset());
    let _ = h.mouse(MouseKind::Up, track.x, track.y.saturating_add(2));
}

#[test]
fn clipped_vertical_thumb_drag_matches_the_same_logical_rows_in_a_full_window() {
    let clipped_spec = WindowSpec {
        area: Rect::new(2, 3, 18, 3),
        logical_rows: 8,
        first_source_row: 2,
    };
    let mut clipped = page(clipped_spec);
    let mut full = page(full_spec(Rect::new(2, 0, 18, 8)));
    clipped.draw();
    full.draw();
    assert_eq!(
        clipped.area_of_part(OWNER, PartRef::of(Part::THUMB)),
        Some(Rect::new(19, 3, 1, 1)),
        "only the thumb/source-row intersection is registered"
    );

    let _ = clipped.mouse(MouseKind::Down, 19, 3);
    let _ = full.mouse(MouseKind::Down, 19, 2);
    assert_eq!(clipped.app().seen_capture.get(), Some(OWNER));
    assert_eq!(full.app().seen_capture.get(), Some(OWNER));
    let _ = clipped.mouse(MouseKind::Drag, 19, 5);
    let _ = full.mouse(MouseKind::Drag, 19, 4);
    assert_eq!(
        clipped.app().state.scroll().offset(),
        full.app().state.scroll().offset(),
        "captured drag maps through the original logical track rows"
    );
    let _ = clipped.mouse(MouseKind::Up, 19, 5);
    let _ = full.mouse(MouseKind::Up, 19, 4);
    assert_eq!(clipped.app().seen_capture.get(), None);
    assert_eq!(full.app().seen_capture.get(), None);
}

#[test]
fn bottom_cut_track_uses_original_source_rows_and_keeps_only_true_thumb_parts() {
    let spec = WindowSpec {
        area: Rect::new(2, 3, 18, 3),
        logical_rows: 8,
        first_source_row: 5,
    };
    let mut h = page(spec);
    h.draw();
    assert_eq!(
        h.area_of_part(OWNER, PartRef::of(Part::TRACK)),
        Some(Rect::new(19, 3, 1, 3))
    );
    assert_eq!(
        h.area_of_part(OWNER, PartRef::of(Part::THUMB)),
        None,
        "the top thumb rows are outside this bottom-cut window"
    );

    let _ = h.mouse(MouseKind::Down, 19, 5);
    assert!(h.app().state.scroll().offset() > 0);
    let _ = h.mouse(MouseKind::Up, 19, 5);
}

#[test]
fn horizontal_bar_only_window_keeps_body_facts_without_text_or_vertical_hits() {
    let mut app = WindowPage::new(WindowSpec {
        area: Rect::new(2, 4, 18, 1),
        logical_rows: 8,
        first_source_row: 7,
    });
    app.pan = true;
    let mut h = page_with_app(app);
    h.draw();
    let _ = h.tick();
    let facts = h.app().seen_facts.get().expect("committed viewport facts");
    assert_eq!(facts.viewport_len, 7);
    assert_eq!(facts.vertical_window, None);
    assert_eq!(facts.vertical_body_window, Some(h.app().window.window()));
    assert_eq!(h.area_of_part(OWNER, PartRef::of(Part::TEXT)), None);
    assert_eq!(h.area_of_part(OWNER, PartRef::of(Part::TRACK)), None);
    assert_eq!(
        h.area_of_part(OWNER, PartRef::of(HORIZONTAL_TRACK)),
        Some(Rect::new(2, 4, 17, 1))
    );
}

#[test]
fn fresh_bar_only_track_click_and_thumb_drag_prepare_horizontal_state() {
    let mut click = bar_only_page(80);
    let track = click
        .area_of_part(OWNER, PartRef::of(HORIZONTAL_TRACK))
        .expect("bar-only window registers its horizontal track");
    let vertical_before = click.app().state.scroll().offset();
    let _ = click.mouse(MouseKind::Down, track.right().saturating_sub(1), track.y);
    assert!(
        click.app().state.horizontal_offset() > 0,
        "a newly mounted bar can page from its fresh content and viewport lengths"
    );
    assert_eq!(click.app().state.scroll().offset(), vertical_before);
    assert_eq!(click.app().state.scroll().viewport_len(), 0);
    let _ = click.wheel(Axis::V, 1, track.x, track.y);
    assert_eq!(click.app().state.scroll().offset(), vertical_before);
    assert_eq!(click.app().state.scroll().viewport_len(), 0);
    let _ = click.mouse(MouseKind::Up, track.right().saturating_sub(1), track.y);

    let mut drag = bar_only_page(80);
    let thumb = drag
        .area_of_part(OWNER, PartRef::of(HORIZONTAL_THUMB))
        .expect("fresh bar paints a horizontal thumb");
    let track = drag
        .area_of_part(OWNER, PartRef::of(HORIZONTAL_TRACK))
        .expect("fresh bar keeps its horizontal track");
    let _ = drag.mouse(MouseKind::Down, thumb.x, thumb.y);
    assert_eq!(drag.app().seen_capture.get(), Some(OWNER));
    let _ = drag.mouse(MouseKind::Drag, track.right().saturating_sub(2), thumb.y);
    assert!(
        drag.app().state.horizontal_offset() > 0,
        "a fresh thumb drag uses the current logical content and text width"
    );
    let _ = drag.mouse(MouseKind::Up, track.right().saturating_sub(2), thumb.y);
    assert_eq!(drag.app().seen_capture.get(), None);
}

#[test]
fn bar_only_pointer_reconciles_horizontal_content_shrink() {
    let mut h = bar_only_page(80);
    let track = h
        .area_of_part(OWNER, PartRef::of(HORIZONTAL_TRACK))
        .expect("bar-only horizontal track");
    let _ = h.click(track.right().saturating_sub(1), track.y);
    assert!(h.app().state.horizontal_offset() > 5);

    h.app_mut().projected = projection_with_width(22);
    h.app_mut().layout_revision = 4;
    let track = h
        .area_of_part(OWNER, PartRef::of(HORIZONTAL_TRACK))
        .expect("the committed bar remains authoritative until repaint");
    let _ = h.mouse(MouseKind::Down, track.right().saturating_sub(1), track.y);
    assert_eq!(
        h.app().seen_facts.get().map(|facts| facts.viewport_len),
        Some(7)
    );
    assert_eq!(
        h.app()
            .seen_facts
            .get()
            .and_then(|facts| facts.vertical_window),
        None,
        "content shrink keeps the bar-only fragment without inventing text rows"
    );
    assert_eq!(
        h.app().state.horizontal_offset(),
        5,
        "the bar-only input transaction clamps against the shrunken content"
    );
    assert_eq!(h.app().state.scroll().viewport_len(), 0);
    let _ = h.mouse(MouseKind::Up, track.right().saturating_sub(1), track.y);
}

#[test]
fn reservation_transitions_use_one_prepared_vertical_track_length() {
    let clipped = WindowSpec {
        area: Rect::new(2, 2, 18, 4),
        logical_rows: 8,
        first_source_row: 4,
    };

    // A wide-source replacement reserves the horizontal row before repaint.
    // The old last vertical-track row maps from the committed clipped interval,
    // while the new seven-row viewport defines the current track length.
    let mut grows = WindowPage::new(clipped);
    grows.source = TextSource::Projected;
    grows.projected = projection_with_width(10);
    grows.pan = true;
    let mut grows = page_with_app(grows);
    grows.draw();
    let old_track = grows
        .area_of_part(OWNER, PartRef::of(Part::TRACK))
        .expect("narrow source has a full clipped vertical track");
    assert_eq!(old_track, Rect::new(19, 2, 1, 4));
    let grow_click = Position::new(old_track.x, old_track.bottom() - 1);
    grows
        .app()
        .pending_event_mutation
        .set(Some(PendingEventMutation::ProjectedSource {
            width: 80,
            layout_revision: 4,
        }));
    let _ = grows.mouse(MouseKind::Down, old_track.x, old_track.bottom() - 1);
    assert!(grows.app().pending_event_mutation.get().is_none());
    assert_eq!(
        grows.app().pending_mutation_cause.get(),
        Some(UpdateCause::Event),
        "the pending source change is applied in the track event transaction"
    );
    assert_eq!(
        grows.app().seen_track_event.get(),
        Some((old_track, grow_click, 7)),
        "the click uses the committed track and its old source row"
    );
    assert_eq!(grows.app().state.scroll().viewport_len(), 7);
    assert_eq!(
        grows.app().state.scroll().offset(),
        grows.app().state.scroll().max_offset(),
        "the old last row is clamped to the prepared seven-row track"
    );

    // Removing the horizontal row grows the prepared viewport. The old last
    // registered row keeps its committed source identity instead of using the
    // old seven-row track length.
    let mut shrinks = WindowPage::new(clipped);
    shrinks.source = TextSource::Projected;
    shrinks.projected = projection_with_width(80);
    shrinks.pan = true;
    let mut shrinks = page_with_app(shrinks);
    shrinks.draw();
    let old_track = shrinks
        .area_of_part(OWNER, PartRef::of(Part::TRACK))
        .expect("wide source reserves one row but keeps its clipped track");
    assert_eq!(old_track, Rect::new(19, 2, 1, 3));
    let shrink_click = Position::new(old_track.x, old_track.bottom() - 1);
    shrinks
        .app()
        .pending_event_mutation
        .set(Some(PendingEventMutation::ProjectedSource {
            width: 10,
            layout_revision: 4,
        }));
    let _ = shrinks.mouse(MouseKind::Down, old_track.x, old_track.bottom() - 1);
    assert!(shrinks.app().pending_event_mutation.get().is_none());
    assert_eq!(
        shrinks.app().pending_mutation_cause.get(),
        Some(UpdateCause::Event),
        "the pending source change is applied in the track event transaction"
    );
    assert_eq!(
        shrinks.app().seen_track_event.get(),
        Some((old_track, shrink_click, 6)),
        "the click uses the committed track and its old source row"
    );
    let mut expected = ScrollState::new(LINES.len());
    expected.set_viewport(8);
    assert_eq!(
        shrinks.app().state.scroll().viewport_len(),
        8,
        "the no-bar source prepares the full logical text height"
    );
    assert_eq!(
        shrinks.app().state.scroll().offset(),
        expected.offset_for_track_pos(6, 8),
        "the former last visible row maps from the committed source interval"
    );
}

#[test]
fn stale_route_revision_cannot_change_the_child_or_fallback_parent() {
    let spec = WindowSpec {
        area: Rect::new(2, 2, 18, 3),
        logical_rows: 8,
        first_source_row: 2,
    };
    let mut app = WindowPage::new(spec);
    app.routed = true;
    let mut h = page_with_app(app);
    h.draw();
    assert!(h.app().route_key.get().is_some());
    let _ = h.wheel(Axis::V, 1, 4, 2);
    assert!(h.app().state.scroll().offset() > 0);
    let child_before = h.app().state.scroll().offset();
    let parent_before = h.app().parent.offset();

    let registered_revision = h.app().revision;
    let stale_revision = registered_revision.saturating_add(1);
    let parent_fallback_calls = h.app().parent_route_calls.get();
    h.app()
        .pending_event_mutation
        .set(Some(PendingEventMutation::SourceRevision(stale_revision)));
    let response = h.wheel(Axis::V, 1, 4, 2);
    assert!(h.app().pending_event_mutation.get().is_none());
    assert_eq!(
        h.app().pending_mutation_cause.get(),
        Some(UpdateCause::Event),
        "the live source revision changes within the wheel event transaction"
    );
    assert_eq!(
        h.app().seen_route_revisions.get(),
        Some((registered_revision, stale_revision)),
        "the event carries the committed revision while the child sees the changed source"
    );
    assert!(!response.is_changed(), "a stale route is not acknowledged");
    assert_eq!(h.app().state.scroll().offset(), child_before);
    assert_eq!(h.app().parent.offset(), parent_before);
    assert_eq!(
        h.app().parent_route_calls.get(),
        parent_fallback_calls,
        "an unacknowledged stale attempt does not fall back to the parent"
    );
}

#[test]
fn aborted_draw_cache_mutation_cannot_replace_committed_window_geometry() {
    let committed = WindowSpec {
        area: Rect::new(2, 2, 18, 3),
        logical_rows: 8,
        first_source_row: 2,
    };
    let mut app = WindowPage::new(committed);
    app.candidate_window = WindowSpec {
        area: Rect::new(2, 7, 18, 3),
        logical_rows: 8,
        first_source_row: 5,
    };
    let mut h = page_with_app(app);
    h.draw();
    h.app_mut().draw_candidate_once.set(true);
    let mut discarded = Buffer::empty(SCREEN);
    drop(h.runtime_mut().draw_buffer(SCREEN, &mut discarded));

    let _ = h.drag((2, 2), (5, 2));
    assert_eq!(
        h.app().state.selection(),
        Some((CellPos::new(2, 0), CellPos::new(2, 3))),
        "input after the aborted candidate draw still maps through the committed source row 2"
    );
    assert_eq!(
        h.app()
            .seen_facts
            .get()
            .and_then(|facts| facts.vertical_body_window),
        Some(committed.window()),
        "the last committed body fact remains authoritative"
    );
}

const FADE_FG: Color = Color::Rgb(120, 160, 200);

fn fade_scene(window: VerticalWindow, state: &ScrollState, full: bool) -> Scene {
    fade_scene_with_clip(window, state, full, None)
}

fn paint_fade_scene(ui: &mut Ui<'_>, window: VerticalWindow, state: &ScrollState, full: bool) {
    let area = window.visible_body();
    for row in area.rows() {
        ui.paint_str(
            Rect::new(area.x, row.y, area.width, 1),
            "window-fade",
            termrock_runtime::Style::new()
                .fg(FADE_FG)
                .bg(Color::Rgb(20, 40, 60)),
        );
    }
    if full {
        ui.scroll_edges(
            Rect::new(
                0,
                1,
                16,
                window.logical_rows().min(usize::from(u16::MAX)) as u16,
            ),
            state,
        );
    } else {
        ui.scroll_edges_window(window, state);
    }
}

fn fade_scene_with_clip(
    window: VerticalWindow,
    state: &ScrollState,
    full: bool,
    clip: Option<Rect>,
) -> Scene {
    let mut scene = Scene::new(
        "viewport_window_fade",
        Theme::junie(),
        ColorLevel::TrueColor,
        16,
        16,
    );
    scene.draw(|ui, _| {
        if let Some(clip) = clip {
            ui.with_area(clip, |ui| paint_fade_scene(ui, window, state, full));
        } else {
            paint_fade_scene(ui, window, state, full);
        }
    });
    scene
}

#[test]
fn window_fades_follow_logical_edges_and_never_the_fragment_edges() {
    for rows in [3usize, 4, 11, 12] {
        let mut state = ScrollState::new(100);
        state.set_viewport(rows);
        state.scroll_to(2);

        let inside = VerticalWindow::new(Rect::new(0, 5, 16, 1), rows, rows / 2).unwrap();
        let inside_scene = fade_scene(inside, &state, false);
        assert_eq!(
            inside_scene
                .buffer()
                .cell(Position::new(0, 5))
                .map(|cell| cell.fg),
            Some(FADE_FG),
            "logical interior row faded for height {rows}"
        );

        let top = VerticalWindow::new(Rect::new(0, 1, 16, 1), rows, 0).unwrap();
        let top_scene = fade_scene(top, &state, false);
        let row_count = u16::try_from(rows).expect("test fade height fits physical rows");
        let full_scene = fade_scene(
            VerticalWindow::full(Rect::new(0, 1, 16, row_count)),
            &state,
            true,
        );
        assert_eq!(
            top_scene
                .buffer()
                .cell(Position::new(0, 1))
                .map(|cell| cell.fg),
            full_scene
                .buffer()
                .cell(Position::new(0, 1))
                .map(|cell| cell.fg),
            "true top fade differs from full-window result for height {rows}"
        );

        let bottom_y = 1 + row_count - 1;
        let bottom = VerticalWindow::new(Rect::new(0, bottom_y, 16, 1), rows, rows - 1).unwrap();
        let bottom_scene = fade_scene(bottom, &state, false);
        assert_eq!(
            bottom_scene
                .buffer()
                .cell(Position::new(0, bottom_y))
                .map(|cell| cell.fg),
            full_scene
                .buffer()
                .cell(Position::new(0, bottom_y))
                .map(|cell| cell.fg),
            "true bottom fade differs from full-window result for height {rows}"
        );
    }
}

#[test]
fn legacy_fade_helper_uses_visible_vertical_clip_edges() {
    let window = VerticalWindow::full(Rect::new(0, 1, 16, 8));
    let mut state = ScrollState::new(20);
    state.set_viewport(8);
    state.scroll_to(2);
    let clips = [
        ("top-only", Rect::new(0, 3, 16, 6)),
        ("bottom-only", Rect::new(0, 1, 16, 6)),
        ("both-sided", Rect::new(0, 3, 16, 4)),
    ];

    for (name, clip) in clips {
        let legacy = fade_scene_with_clip(window, &state, true, Some(clip));
        let logical = fade_scene_with_clip(window, &state, false, Some(clip));
        for y in clip.y..clip.bottom() {
            let pos = Position::new(0, y);
            let legacy_faded = legacy
                .buffer()
                .cell(pos)
                .is_some_and(|cell| cell.fg != FADE_FG);
            let logical_faded = logical
                .buffer()
                .cell(pos)
                .is_some_and(|cell| cell.fg != FADE_FG);
            let expected_legacy = y == clip.y || y == clip.bottom().saturating_sub(1);
            let expected_logical = y == window.visible_body().y
                || y == window.visible_body().bottom().saturating_sub(1);
            assert_eq!(
                legacy_faded, expected_legacy,
                "old scroll_edges visible-height fade differs for {name} at row {y}"
            );
            assert_eq!(
                logical_faded, expected_logical,
                "window helper true-edge fade differs for {name} at row {y}"
            );
        }
    }
}

fn clipped_viewport_for_fade(
    draw_api: DrawApi,
    source: TextSource,
    routed: bool,
    rows: u16,
    clip: Rect,
    selected: bool,
) -> Harness<WindowPage> {
    let area = Rect::new(2, 1, 18, rows);
    let mut app = WindowPage::new(full_spec(area));
    app.draw_api = draw_api;
    app.clip = Some(clip);
    app.source = source;
    app.routed = routed;
    if selected {
        app.state.select(CellPos::new(0, 0), CellPos::new(19, 0));
    }
    let mut h = page_with_app(app);
    h.draw();
    let response = h.wheel(Axis::V, 1, clip.x.saturating_add(2), clip.y);
    assert!(
        response.is_consumed(),
        "the clipped viewport receives wheel input"
    );
    assert!(
        h.app().state.scroll().offset() > 0,
        "the fade needs an overflowing leading edge"
    );
    h.draw();
    h
}

#[test]
fn full_rect_and_window_fades_match_at_legacy_depth_thresholds() {
    for rows in [3u16, 4, 11, 12] {
        let legacy = clipped_viewport_for_fade(
            DrawApi::Legacy,
            TextSource::Borrowed,
            false,
            rows,
            Rect::new(2, 1, 18, rows),
            false,
        );
        let windowed = clipped_viewport_for_fade(
            DrawApi::Window,
            TextSource::Borrowed,
            false,
            rows,
            Rect::new(2, 1, 18, rows),
            false,
        );
        assert_eq!(
            legacy.buffer(),
            windowed.buffer(),
            "full Rect and logical-window fades diverge at {rows} rows"
        );
    }
}

#[test]
fn rect_and_window_fades_keep_distinct_vertical_clip_edge_policies() {
    let body = Rect::new(2, 1, 18, 8);
    let clips = [
        ("top-only", Rect::new(2, 3, 18, 6), true, false),
        ("bottom-only", Rect::new(2, 1, 18, 6), false, true),
        ("both-sided", Rect::new(2, 3, 18, 4), true, true),
    ];

    for (source, routed) in [
        (TextSource::Borrowed, false),
        (TextSource::Borrowed, true),
        (TextSource::Projected, false),
        (TextSource::Projected, true),
    ] {
        for (name, clip, clipped_top, clipped_bottom) in clips {
            let legacy =
                clipped_viewport_for_fade(DrawApi::Legacy, source, routed, body.height, clip, true);
            let windowed =
                clipped_viewport_for_fade(DrawApi::Window, source, routed, body.height, clip, true);
            let x = body.x.saturating_add(1);
            let top = clip.y;
            let bottom = clip.bottom().saturating_sub(1);
            if clipped_top {
                assert_ne!(
                    legacy.cell(x, top).fg,
                    windowed.cell(x, top).fg,
                    "{name} clip must use the visible Rect top for legacy {source:?} routed={routed}"
                );
                assert_eq!(
                    legacy.cell(x, top).bg,
                    windowed.cell(x, top).bg,
                    "fade policy must preserve selection background at {name} top"
                );
            } else {
                assert_eq!(
                    legacy.cell(x, top).fg,
                    windowed.cell(x, top).fg,
                    "the visible true top edge must match for {name}"
                );
            }
            if clipped_bottom {
                assert_ne!(
                    legacy.cell(x, bottom).fg,
                    windowed.cell(x, bottom).fg,
                    "{name} clip must use the visible Rect bottom for legacy {source:?} routed={routed}"
                );
                assert_eq!(
                    legacy.cell(x, bottom).bg,
                    windowed.cell(x, bottom).bg,
                    "fade policy must preserve selection background at {name} bottom"
                );
            } else {
                assert_eq!(
                    legacy.cell(x, bottom).fg,
                    windowed.cell(x, bottom).fg,
                    "the visible true bottom edge must match for {name}"
                );
            }
        }
    }

    let clip = Rect::new(2, 3, 18, 6);
    let selected = clipped_viewport_for_fade(
        DrawApi::Legacy,
        TextSource::Borrowed,
        false,
        body.height,
        clip,
        true,
    );
    let unselected = clipped_viewport_for_fade(
        DrawApi::Legacy,
        TextSource::Borrowed,
        false,
        body.height,
        clip,
        false,
    );
    assert_ne!(
        selected.cell(body.x.saturating_add(1), clip.y).bg,
        unselected.cell(body.x.saturating_add(1), clip.y).bg,
        "the clipped edge sample carries a real selection background"
    );
}

#[test]
fn legacy_rect_depth_uses_clipped_height_while_window_depth_uses_logical_height() {
    let body = Rect::new(2, 1, 18, 12);
    let clip = Rect::new(2, 2, 18, 11);
    let legacy = clipped_viewport_for_fade(
        DrawApi::Legacy,
        TextSource::Borrowed,
        false,
        body.height,
        clip,
        false,
    );
    let windowed = clipped_viewport_for_fade(
        DrawApi::Window,
        TextSource::Borrowed,
        false,
        body.height,
        clip,
        false,
    );
    let x = body.x.saturating_add(1);
    let inner_bottom = body.bottom().saturating_sub(2);
    assert_eq!(
        legacy.cell(x, inner_bottom).fg,
        legacy.cell(x, inner_bottom.saturating_sub(1)).fg,
        "eleven visible rows retain the legacy one-row fade depth"
    );
    assert_ne!(
        windowed.cell(x, inner_bottom).fg,
        windowed.cell(x, inner_bottom.saturating_sub(1)).fg,
        "the twelve-row logical window keeps its inner true-bottom fade"
    );
}

#[test]
fn legacy_rect_vertical_clip_fade_protects_a_visible_cursor_row() {
    let clip = Rect::new(2, 3, 18, 4);
    let mut with_cursor = WindowPage::new(full_spec(Rect::new(2, 1, 18, 8)));
    with_cursor.draw_api = DrawApi::Legacy;
    with_cursor.clip = Some(clip);
    let mut cursor_page = page_with_app(with_cursor);
    cursor_page.draw();
    assert!(cursor_page.tab_to(OWNER));
    let _ = cursor_page.wheel(Axis::V, 1, clip.x.saturating_add(2), clip.y);
    let caret_line = cursor_page.app().state.scroll().offset().saturating_add(2);
    cursor_page
        .app_mut()
        .state
        .set_caret(Some(CellPos::new(caret_line, 0)));
    cursor_page.draw();
    assert_eq!(cursor_page.cursor(), Some(Position::new(2, clip.y)));

    let mut without_cursor = WindowPage::new(full_spec(Rect::new(2, 1, 18, 8)));
    without_cursor.draw_api = DrawApi::Legacy;
    without_cursor.clip = Some(clip);
    let mut plain_page = page_with_app(without_cursor);
    plain_page.draw();
    let _ = plain_page.wheel(Axis::V, 1, clip.x.saturating_add(2), clip.y);
    plain_page.draw();

    assert_eq!(
        cursor_page.cell(2, clip.y).fg,
        cursor_page.cell(2, clip.y.saturating_add(1)).fg,
        "the cursor row keeps its unfaded foreground at a clipped Rect edge"
    );
    assert_ne!(
        plain_page.cell(2, clip.y).fg,
        plain_page.cell(2, clip.y.saturating_add(1)).fg,
        "the same clipped Rect edge fades when it has no cursor"
    );
}

#[test]
fn full_window_fades_match_the_rect_baseline_under_an_additional_ui_clip() {
    let window = VerticalWindow::full(Rect::new(0, 1, 16, 8));
    let clip = Rect::new(4, 1, 8, 8);
    let mut state = ScrollState::new(20);
    state.set_viewport(8);
    state.scroll_to(2);
    let baseline = fade_scene_with_clip(window, &state, true, Some(clip));
    let windowed = fade_scene_with_clip(window, &state, false, Some(clip));

    for y in 0..16 {
        for x in 0..16 {
            let pos = Position::new(x, y);
            assert_eq!(
                baseline
                    .buffer()
                    .cell(pos)
                    .map(|cell| (cell.symbol(), cell.fg, cell.bg)),
                windowed
                    .buffer()
                    .cell(pos)
                    .map(|cell| (cell.symbol(), cell.fg, cell.bg)),
                "full-window fade differs from the old Rect helper at ({x}, {y})"
            );
        }
    }
}
