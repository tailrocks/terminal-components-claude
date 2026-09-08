//! Frozen projection facts are separate from live input-routing publication.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::arithmetic_side_effects,
    reason = "fixture assertions measure explicit projection and lifecycle contracts"
)]

use std::cell::Cell;
use std::rc::Rc;

use junie_tui::{
    App, ColorLevel, Cx, Focusability, FrameRead, Id, Input, Intent, LayerSpec, Position, Rect,
    RenderSnapshot, RenderSnapshotError, Response, Runtime, StateFlags, Theme, Ui, UpdateCause,
};
use junie_tui_testing::Scene;
use junie_tui_testing::perf::{Counting, bench, lock};
use ratatui_core::buffer::Buffer;

#[global_allocator]
static ALLOCATOR: Counting = Counting;

const PAGE: Id = Id::root("snapshot.page");
const MODAL: Id = Id::root("snapshot.modal");
const EDITOR: Id = Id::root("snapshot.editor");
const AREA: Rect = Rect::new(0, 0, 40, 10);

#[derive(Default)]
struct Model {
    modal: bool,
    alternate: bool,
    updates: usize,
    value: String,
    effects: Rc<Cell<usize>>,
}
impl App for Model {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.updates += 1;
        if cx.update_cause() == UpdateCause::Bootstrap {
            self.effects.set(self.effects.get() + 1);
            if self.modal {
                cx.open_layer(MODAL, LayerSpec::modal(MODAL));
            }
        }
        for id in [PAGE, MODAL, EDITOR] {
            for intent in cx.intents(id) {
                if let Intent::Paste(text) = intent {
                    self.value.push_str(text);
                }
            }
        }
        Response::ignored()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        let style = ui.surface_style();
        ui.paint_str(AREA, "provided model", style);
        if self.modal {
            ui.layer(MODAL, Self::editor);
        } else {
            Self::editor(
                ui,
                if self.alternate {
                    Rect::new(14, 3, 12, 1)
                } else {
                    Rect::new(1, 1, 12, 1)
                },
            );
        }
    }
}
impl Model {
    fn editor(ui: &mut Ui<'_>, area: Rect) {
        ui.register_editor(EDITOR, area, Focusability::Focusable, StateFlags::EDITING);
        let text = if ui.state(EDITOR).contains(StateFlags::FOCUSED) {
            "focused"
        } else {
            "unfocused"
        };
        let style = ui.surface_style();
        ui.paint_str(area, text, style);
        ui.set_cursor(EDITOR, Position::new(area.x, area.y));
    }
}

fn present(rt: &mut Runtime<Model>, buffer: &mut Buffer) {
    for _ in 0..16 {
        if rt.needs_settle() {
            let _ = rt.settle();
        }
        buffer.reset();
        rt.draw_buffer(AREA, buffer).commit_presented();
        if !rt.needs_present() && !rt.needs_settle() {
            return;
        }
    }
    panic!("fixture did not settle");
}
fn live(model: Model) -> (Runtime<Model>, Buffer) {
    let mut rt = Runtime::new(model, Theme::junie());
    let _ = rt.initialize();
    let mut buffer = Buffer::empty(AREA);
    present(&mut rt, &mut buffer);
    (rt, buffer)
}
fn scene() -> Scene {
    Scene::new("snapshot", Theme::junie(), ColorLevel::TrueColor, 40, 10)
}

#[test]
fn live_snapshot_rejects_uninitialized_unsettled_and_unpresented_state() {
    let mut rt = Runtime::new(Model::default(), Theme::junie());
    assert!(matches!(
        rt.render_snapshot(),
        Err(RenderSnapshotError::Uninitialized)
    ));
    let _ = rt.initialize();
    assert!(matches!(
        rt.render_snapshot(),
        Err(RenderSnapshotError::NeedsPresentation)
    ));
    let mut buffer = Buffer::empty(AREA);
    rt.draw_buffer(AREA, &mut buffer).commit_presented();
    assert!(matches!(
        rt.render_snapshot(),
        Err(RenderSnapshotError::NeedsSettle)
    ));
    let _ = rt.settle();
    assert!(matches!(
        rt.render_snapshot(),
        Err(RenderSnapshotError::NeedsPresentation)
    ));
    present(&mut rt, &mut buffer);
    assert!(rt.render_snapshot().is_ok());
    rt.app_mut().alternate = true;
    assert!(matches!(
        rt.render_snapshot(),
        Err(RenderSnapshotError::NeedsPresentation)
    ));
    present(&mut rt, &mut buffer);
    rt.set_theme(Theme::paper());
    assert!(matches!(
        rt.render_snapshot(),
        Err(RenderSnapshotError::NeedsPresentation)
    ));
}

#[test]
fn default_snapshot_has_identical_first_and_repeated_cells_without_initialization() {
    let app = Model::default();
    let mut capture = scene();
    capture.draw_app(&app);
    let first = capture.buffer().clone();
    let cursor = capture.cursor_position();
    for _ in 0..100 {
        capture.draw_app(&app);
        assert_eq!(capture.buffer(), &first);
        assert_eq!(capture.cursor_position(), cursor);
    }
    assert_eq!(app.updates, 0);
    assert_eq!(app.effects.get(), 0);
    assert_eq!(
        capture.registry().unwrap().area_of(EDITOR),
        Some(Rect::new(1, 1, 12, 1))
    );
    assert_eq!(capture.ring().unwrap().reachable().count(), 1);
}

#[test]
fn production_modal_snapshot_repeats_exact_cells_cursor_and_effect_state() {
    let (rt, buffer) = live(Model {
        modal: true,
        ..Model::default()
    });
    let before_updates = rt.app().updates;
    let before_effects = rt.app().effects.get();
    let snapshot = rt.render_snapshot().unwrap();
    let mut capture = scene();
    capture.set_snapshot(snapshot);
    for _ in 0..100 {
        capture.draw_app(rt.app());
        assert_eq!(capture.buffer(), &buffer);
        assert_eq!(capture.cursor_position(), rt.cursor_position());
        assert_eq!(capture.ring().unwrap().entries(), rt.ring().entries());
    }
    assert_eq!(rt.app().updates, before_updates);
    assert_eq!(rt.app().effects.get(), before_effects);
    assert!(rt.is_open(MODAL));
    assert_eq!(rt.clock_ms(), 0);
}

#[test]
fn replacing_snapshot_with_empty_facts_removes_modal_projection() {
    let (rt, _) = live(Model {
        modal: true,
        ..Model::default()
    });
    let mut capture = scene();
    capture.set_snapshot(rt.render_snapshot().unwrap());
    capture.draw_app(rt.app());
    assert!(capture.registry().unwrap().has_owner(EDITOR));
    capture.set_snapshot(RenderSnapshot::default());
    capture.draw_app(rt.app());
    assert!(!capture.registry().unwrap().has_owner(EDITOR));
    assert_eq!(capture.cursor_position(), None);
}

#[test]
fn foreign_projection_cannot_contaminate_later_live_routing_or_paint() {
    let (mut rt, expected) = live(Model::default());
    let (foreign, _) = live(Model {
        alternate: true,
        ..Model::default()
    });
    let snapshot = foreign.render_snapshot().unwrap();
    let live_area = rt.area_of(EDITOR);
    let live_focus = rt.focus();
    let live_cursor = rt.cursor_position();
    let updates = rt.app().updates;
    let mut buffer = Buffer::empty(AREA);
    rt.draw_projection(AREA, &mut buffer, &snapshot, |ui, _| foreign.app().draw(ui))
        .commit_inspected();
    assert_eq!(
        rt.projection_registry().unwrap().area_of(EDITOR),
        foreign.area_of(EDITOR)
    );
    assert_eq!(rt.area_of(EDITOR), live_area);
    assert_eq!(rt.focus(), live_focus);
    assert_eq!(rt.cursor_position(), live_cursor);
    assert_eq!(rt.app().updates, updates);
    let pending = rt.handle(Input::Paste("retained".into())).unwrap_err();
    present(&mut rt, &mut buffer);
    assert_eq!(buffer, expected);
    let _ = rt.handle(pending.into_input()).unwrap();
    assert_eq!(rt.app().value, "retained");
}

#[test]
fn warmed_modal_snapshot_projection_performs_zero_allocations() {
    let _lock = lock();
    let (rt, _) = live(Model {
        modal: true,
        ..Model::default()
    });
    let mut capture = scene();
    capture.set_snapshot(rt.render_snapshot().unwrap());
    let mut bound = capture.bind_app(rt.app());
    let stats = bench(4, 100, &mut || bound.draw());
    assert_eq!(
        stats.allocs, 0,
        "snapshot copies or projection metadata allocated per capture"
    );
    assert_eq!(stats.bytes, 0);
}

struct CachedModel(&'static str);
impl App for CachedModel {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        let cached = ui.cache::<Option<&'static str>>(EDITOR);
        let text = *cached.get_or_insert(self.0);
        let style = ui.surface_style();
        ui.paint_str(AREA, text, style);
    }
}
fn cached_runtime(text: &'static str) -> Runtime<CachedModel> {
    let mut rt = Runtime::new(CachedModel(text), Theme::junie());
    let _ = rt.initialize();
    let mut buffer = Buffer::empty(AREA);
    rt.draw_buffer(AREA, &mut buffer).commit_presented();
    rt
}

#[test]
fn immutable_snapshot_identity_isolates_derived_caches_from_other_snapshots_and_live_paint() {
    let mut first = cached_runtime("FIRST");
    let second = cached_runtime("OTHER");
    let first_snapshot = first.render_snapshot().unwrap();
    let second_snapshot = second.render_snapshot().unwrap();
    let mut capture = scene();
    capture.set_snapshot(first_snapshot);
    capture.draw_app(first.app());
    assert!(capture.text().contains("FIRST"));
    capture.set_snapshot(second_snapshot.clone());
    capture.draw_app(second.app());
    assert!(capture.text().contains("OTHER"));
    let mut buffer = Buffer::empty(AREA);
    first
        .draw_projection(AREA, &mut buffer, &second_snapshot, |ui, _| {
            second.app().draw(ui);
        })
        .commit_inspected();
    assert_eq!(buffer.cell(Position::new(0, 0)).unwrap().symbol(), "O");
    buffer.reset();
    first.draw_buffer(AREA, &mut buffer).commit_presented();
    assert_eq!(buffer.cell(Position::new(0, 0)).unwrap().symbol(), "F");
}

#[test]
fn one_shot_models_and_same_address_replacements_never_share_derived_cache() {
    let mut capture = scene();
    let first = CachedModel("FIRST");
    let second = CachedModel("OTHER");
    capture.draw_app(&first);
    capture.draw_app(&second);
    assert!(capture.text().contains("OTHER"));
    let mut replaced = CachedModel("FIRST");
    capture.bind_app(&replaced).draw();
    replaced.0 = "OTHER";
    capture.bind_app(&replaced).draw();
    assert!(capture.text().contains("OTHER"));
    capture.draw(|ui, _| first.draw(ui));
    capture.draw(|ui, _| second.draw(ui));
    assert!(capture.text().contains("OTHER"));
}

struct ViewportModel {
    state: junie_tui::ViewportState,
    text: &'static str,
}
impl App for ViewportModel {
    fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
        panic!("pure viewport capture updated model")
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        junie_tui::TextViewport::new(EDITOR).wrap(true).draw(
            ui,
            Rect::new(0, 0, 8, 2),
            &self.state,
            &[
                junie_tui::ViewportLine::Plain(self.text),
                junie_tui::ViewportLine::Plain("TAIL"),
            ],
        );
    }
}

#[test]
fn real_viewport_models_with_equal_generation_have_independent_wrap_geometry() {
    let _lock = lock();
    let first = ViewportModel {
        state: junie_tui::ViewportState::default(),
        text: "A",
    };
    let second = ViewportModel {
        state: junie_tui::ViewportState::default(),
        text: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
    };
    let mut capture = scene();
    let mut fresh = scene();
    capture.draw_app(&first);
    capture.draw_app(&second);
    fresh.draw_app(&second);
    assert_eq!(capture.buffer(), fresh.buffer());
    capture.bind_app(&first).draw();
    let mut bound = capture.bind_app(&second);
    let expected = fresh.buffer().clone();
    for _ in 0..100 {
        bound.draw();
        assert_eq!(bound.scene().buffer(), &expected);
    }
    let stats = bench(4, 100, &mut || bound.draw());
    assert_eq!((stats.allocs, stats.bytes), (0, 0));
}

#[test]
fn raw_runtime_projection_is_one_shot_and_bound_models_reuse_only_own_caches() {
    let first = CachedModel("FIRST");
    let second = CachedModel("OTHER");
    let mut rt = Runtime::new(junie_tui_testing::NoApp, Theme::junie());
    let snapshot = RenderSnapshot::default();
    let mut buffer = Buffer::empty(AREA);
    rt.draw_projection(AREA, &mut buffer, &snapshot, |ui, _| first.draw(ui))
        .commit_inspected();
    buffer.reset();
    rt.draw_projection(AREA, &mut buffer, &snapshot, |ui, _| second.draw(ui))
        .commit_inspected();
    assert_eq!(buffer.cell(Position::new(0, 0)).unwrap().symbol(), "O");
    let first = snapshot.bind_model(&first, |app, ui, _| app.draw(ui));
    let second = snapshot.bind_model(&second, |app, ui, _| app.draw(ui));
    for _ in 0..4 {
        buffer.reset();
        rt.draw_bound_projection(AREA, &mut buffer, &first)
            .commit_inspected();
        assert_eq!(buffer.cell(Position::new(0, 0)).unwrap().symbol(), "F");
        buffer.reset();
        rt.draw_bound_projection(AREA, &mut buffer, &second)
            .commit_inspected();
        assert_eq!(buffer.cell(Position::new(0, 0)).unwrap().symbol(), "O");
    }
}
