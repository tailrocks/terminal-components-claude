#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "test assertions prove each resize invalidation and presentation"
)]

use ratatui_core::backend::TestBackend;
use ratatui_core::layout::Rect;
use ratatui_core::terminal::Terminal;
use termrock_core::{Input, Invalidate, Response};
use termrock_runtime::{App, Cx, Runtime, Ui};
use termrock_theme::Theme;

#[derive(Default)]
struct Blank;

impl App for Blank {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, _ui: &mut Ui<'_>) {}
}

fn terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    Terminal::new(TestBackend::new(width, height)).expect("test terminal")
}

fn publish(runtime: &mut Runtime<Blank>, terminal: &mut Terminal<TestBackend>) {
    let actual_area = terminal.get_frame().area();
    let prepared = runtime
        .prepare(actual_area)
        .expect("runtime prepares the actual terminal area");
    assert_eq!(prepared.area(), actual_area);

    let mut projected = None;
    terminal
        .draw(|frame| {
            projected = Some(runtime.draw_prepared(prepared, frame));
        })
        .expect("test backend draws prepared frame");
    assert!(
        !projected
            .expect("draw callback ran")
            .expect("prepared area matches terminal"),
        "blank page does not request a cursor bar"
    );
    runtime
        .acknowledge_presented(prepared)
        .expect("successfully drawn frame becomes current geometry");
}

fn resize_and_publish(
    runtime: &mut Runtime<Blank>,
    width: u16,
    height: u16,
) -> Terminal<TestBackend> {
    let response = runtime
        .handle(Input::Resize(width, height))
        .expect("ready runtime accepts resize");
    assert_eq!(response.invalidate(), Invalidate::Layout);

    let expected_area = Rect::new(0, 0, width, height);
    assert_eq!(runtime.screen(), expected_area);
    assert!(runtime.needs_present());

    let mut resized_terminal = terminal(width, height);
    let actual_area = resized_terminal.get_frame().area();
    assert_eq!(actual_area, expected_area);
    publish(runtime, &mut resized_terminal);

    assert_eq!(runtime.screen(), actual_area);
    assert!(!runtime.needs_present());
    assert!(!runtime.needs_settle());
    resized_terminal
}

#[test]
fn one_runtime_publishes_each_shrink_grow_shrink_geometry() {
    let mut runtime = Runtime::new(Blank, Theme::junie());
    let mut current_terminal = terminal(120, 40);
    let initial_area = current_terminal.get_frame().area();
    assert_eq!(initial_area, Rect::new(0, 0, 120, 40));
    publish(&mut runtime, &mut current_terminal);
    assert_eq!(runtime.screen(), initial_area);
    assert!(!runtime.needs_present());

    for (width, height) in [(80, 24), (120, 40), (72, 20)] {
        current_terminal = resize_and_publish(&mut runtime, width, height);
        let actual_area = current_terminal.get_frame().area();
        assert_eq!(
            runtime.screen(),
            actual_area,
            "screen after {width}x{height}"
        );
        assert_eq!(actual_area, Rect::new(0, 0, width, height));
    }
}
