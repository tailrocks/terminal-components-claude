//! Bootstrap is explicit in live drivers and absent from production-view painting.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use junie_tui::{
    App, ColorLevel, Cx, Focusability, Id, Input, Key, KeyCode, KeyModifiers, LayerSpec, Rect,
    Response, Runtime, Theme, Ui, UpdateCause,
};
use junie_tui_testing::{Harness, Scene};
use ratatui_core::buffer::Buffer;

const CONTROL: Id = Id::root("initialization.control");
const LAYER: Id = Id::root("initialization.layer");
const AREA: Rect = Rect::new(0, 0, 40, 12);

#[derive(Default)]
struct ProductionView {
    bootstraps: usize,
    updates: usize,
    commits: usize,
    value: String,
    effects: Rc<Cell<usize>>,
}

impl App for ProductionView {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.updates = self.updates.saturating_add(1);
        if cx.update_cause() == UpdateCause::Bootstrap {
            self.bootstraps = self.bootstraps.saturating_add(1);
            self.commits = self.commits.saturating_add(1);
            self.value.push_str(" initialized");
            self.effects.set(self.effects.get().saturating_add(1));
            cx.open_layer(LAYER, LayerSpec::modal(LAYER));
            cx.request_repaint_after(Duration::from_millis(2_200));
            return Response::changed();
        }
        for intent in cx.intents(CONTROL) {
            let _ = intent;
        }
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        ui.register_control(CONTROL, Rect::new(0, 0, 8, 1), Focusability::Focusable);
        junie_tui::Button::new(CONTROL.sub("view"), &self.value).draw(ui, Rect::new(0, 2, 30, 1));
    }
}

fn input() -> Input {
    Input::Key(Key {
        code: KeyCode::Char('x'),
        mods: KeyModifiers::NONE,
    })
}

#[test]
fn first_and_repeated_draws_cannot_initialize_commit_or_emit_effects() {
    let app = ProductionView {
        value: "fixture".into(),
        ..ProductionView::default()
    };
    let effects = Rc::clone(&app.effects);
    let mut runtime = Runtime::new(app, Theme::junie());
    let mut buffer = Buffer::empty(AREA);
    runtime.draw_buffer(AREA, &mut buffer);
    let before = buffer.clone();
    for _ in 0..1_000 {
        runtime.draw_buffer(AREA, &mut buffer);
    }
    assert_eq!(buffer, before);
    assert_eq!(runtime.app().value, "fixture");
    assert_eq!(runtime.app().updates, 0);
    assert_eq!(runtime.app().commits, 0);
    assert_eq!(effects.get(), 0);
    assert_eq!(runtime.clock_ms(), 0);
    assert_eq!(runtime.next_deadline(), None);
    assert!(!runtime.is_open(LAYER));
}

#[test]
fn explicit_initialize_runs_once_and_retains_deadline_without_advancing_time() {
    let mut runtime = Runtime::new(ProductionView::default(), Theme::junie());
    let response = runtime.initialize();
    assert!(response.is_changed());
    assert!(runtime.is_open(LAYER));
    assert_eq!(runtime.next_deadline(), Some(Duration::from_millis(2_200)));
    assert_eq!(runtime.clock_ms(), 0);
    assert_eq!(runtime.app().bootstraps, 1);
    assert!(!runtime.initialize().is_changed());
    assert_eq!(runtime.app().updates, 1);
    assert_eq!(runtime.app().effects.get(), 1);
    let mut buffer = Buffer::empty(AREA);
    runtime.draw_buffer(AREA, &mut buffer);
    assert_eq!(runtime.app().updates, 1);
}

#[test]
fn input_path_initializes_without_requiring_a_paint() {
    let mut runtime = Runtime::new(ProductionView::default(), Theme::junie());
    let _ = runtime.handle(input());
    assert_eq!(runtime.app().bootstraps, 1);
    assert_eq!(runtime.app().updates, 2);
    let _ = runtime.handle(input());
    assert_eq!(runtime.app().bootstraps, 1);
    assert_eq!(runtime.app().updates, 3);
}

#[test]
fn scene_projection_does_not_update_the_supplied_app() {
    let app = ProductionView {
        value: "scene fixture".into(),
        ..ProductionView::default()
    };
    let mut scene = Scene::new(
        "pure-initialization",
        Theme::junie(),
        ColorLevel::TrueColor,
        40,
        12,
    );
    scene.draw(|ui, _| app.draw(ui));
    scene.draw(|ui, _| app.draw(ui));
    assert_eq!(app.value, "scene fixture");
    assert_eq!(app.updates, 0);
    assert_eq!(app.effects.get(), 0);
}

#[test]
fn behavioral_harness_explicitly_initializes_before_painting() {
    let mut harness = Harness::new(ProductionView::default(), Theme::junie(), 40, 12);
    assert_eq!(harness.app().bootstraps, 1);
    assert_eq!(harness.app().effects.get(), 1);
    let updates = harness.app().updates;
    harness.draw();
    harness.draw();
    assert_eq!(harness.app().updates, updates);
}
