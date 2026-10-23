#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "small test geometry and lifecycle assertions"
)]

use ratatui_core::backend::TestBackend;
use ratatui_core::layout::{Position, Rect};
use ratatui_core::terminal::Terminal;
use termrock_core::{Id, Input, Intent, KeyModifiers, Mouse, MouseKind, Phase, Response};
use termrock_runtime::{App, Cx, Focusability, Runtime, Ui};
use termrock_theme::Theme;

const A: Id = Id::root("resize-pointer.a");
const B: Id = Id::root("resize-pointer.b");
const C: Id = Id::root("resize-pointer.c");

#[derive(Default)]
struct Page {
    pointer_events: Vec<(Id, Phase)>,
}

impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        for owner in [A, B, C] {
            for intent in cx.intents(owner) {
                if let Intent::Pointer { phase, .. } = intent {
                    self.pointer_events.push((owner, phase));
                }
            }
        }
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let (a, b, c) = controls(ui.full());
        for (owner, area) in [(A, a), (B, b), (C, c)] {
            ui.register_control(owner, area, Focusability::Focusable);
        }
    }
}

fn controls(area: Rect) -> (Rect, Rect, Rect) {
    let a = Rect::new(
        area.x + area.width / 10,
        area.y + area.height / 4,
        area.width / 5,
        area.height / 3,
    );
    let b = Rect::new(
        area.x + area.width / 2,
        area.y + area.height / 4,
        area.width / 4,
        area.height / 3,
    );
    let c = Rect::new(
        area.x + (area.width * 3) / 4,
        area.y + area.height / 2,
        area.width / 5,
        area.height / 3,
    );
    (a, b, c)
}

fn center(area: Rect) -> Position {
    Position::new(area.x + area.width / 2, area.y + area.height / 2)
}

fn contains(area: Rect, point: Position) -> bool {
    point.x >= area.x
        && point.y >= area.y
        && point.x < area.x.saturating_add(area.width)
        && point.y < area.y.saturating_add(area.height)
}

fn publish(runtime: &mut Runtime<Page>, terminal: &mut Terminal<TestBackend>) {
    let area = terminal.get_frame().area();
    let prepared = runtime
        .prepare(area)
        .expect("runtime prepares current terminal area");
    let mut projected = None;
    terminal
        .draw(|frame| {
            projected = Some(runtime.draw_prepared(prepared, frame));
        })
        .expect("test backend draws prepared frame");
    assert!(
        !projected
            .expect("draw callback ran")
            .expect("frame matches terminal area"),
        "test controls do not request a cursor bar"
    );
    runtime
        .acknowledge_presented(prepared)
        .expect("successfully drawn frame becomes current geometry");
}

fn handle_and_publish(
    runtime: &mut Runtime<Page>,
    terminal: &mut Terminal<TestBackend>,
    input: Input,
) {
    let _response = runtime.handle(input).expect("runtime is ready for input");
    publish(runtime, terminal);
}

fn click(runtime: &mut Runtime<Page>, terminal: &mut Terminal<TestBackend>, point: Position) {
    for kind in [MouseKind::Down, MouseKind::Up] {
        handle_and_publish(
            runtime,
            terminal,
            Input::Mouse(Mouse {
                kind,
                pos: point,
                mods: KeyModifiers::NONE,
            }),
        );
    }
}

fn resized_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    Terminal::new(TestBackend::new(width, height)).expect("test terminal")
}

fn assert_pointer_events(page: &Page, expected: &[(Id, Phase)]) {
    assert_eq!(page.pointer_events, expected);
}

#[test]
fn resize_keeps_focus_and_routes_clicks_to_republished_geometry() {
    let mut runtime = Runtime::new(Page::default(), Theme::junie());
    let mut terminal = resized_terminal(40, 12);
    publish(&mut runtime, &mut terminal);

    let old_area = terminal.get_frame().area();
    let (old_a, old_b, old_c) = controls(old_area);
    assert_eq!(runtime.screen(), old_area);
    assert_eq!(runtime.area_of(A), Some(old_a));
    assert_eq!(runtime.area_of(B), Some(old_b));
    assert_eq!(runtime.area_of(C), Some(old_c));

    click(&mut runtime, &mut terminal, center(old_b));
    assert_eq!(runtime.focus(), Some(B));
    assert_pointer_events(
        runtime.app(),
        &[(B, Phase::Press), (B, Phase::Release), (B, Phase::Click)],
    );

    let _response = runtime
        .handle(Input::Resize(60, 16))
        .expect("runtime accepts grow resize");
    let mut grown_terminal = resized_terminal(60, 16);
    publish(&mut runtime, &mut grown_terminal);
    let grown_area = grown_terminal.get_frame().area();
    let (grown_a, grown_b, grown_c) = controls(grown_area);
    assert_eq!(runtime.screen(), grown_area);
    assert_eq!(runtime.focus(), Some(B));
    assert_eq!(runtime.area_of(A), Some(grown_a));
    assert_eq!(runtime.area_of(B), Some(grown_b));
    assert_eq!(runtime.area_of(C), Some(grown_c));
    let grown_a_point = center(grown_a);
    assert!(contains(grown_a, grown_a_point));
    assert!(!contains(old_a, grown_a_point));

    click(&mut runtime, &mut grown_terminal, grown_a_point);
    assert_eq!(runtime.focus(), Some(A));
    assert_pointer_events(
        runtime.app(),
        &[
            (B, Phase::Press),
            (B, Phase::Release),
            (B, Phase::Click),
            (A, Phase::Press),
            (A, Phase::Release),
            (A, Phase::Click),
        ],
    );

    let _response = runtime
        .handle(Input::Resize(32, 10))
        .expect("runtime accepts shrink resize");
    let mut shrunk_terminal = resized_terminal(32, 10);
    publish(&mut runtime, &mut shrunk_terminal);
    let shrunk_area = shrunk_terminal.get_frame().area();
    let (shrunk_a, shrunk_b, shrunk_c) = controls(shrunk_area);
    assert_eq!(runtime.screen(), shrunk_area);
    assert_eq!(runtime.focus(), Some(A));
    assert_eq!(runtime.area_of(A), Some(shrunk_a));
    assert_eq!(runtime.area_of(B), Some(shrunk_b));
    assert_eq!(runtime.area_of(C), Some(shrunk_c));
    let shrunk_c_point = center(shrunk_c);
    assert!(contains(shrunk_c, shrunk_c_point));
    assert!(!contains(grown_c, shrunk_c_point));

    click(&mut runtime, &mut shrunk_terminal, shrunk_c_point);
    assert_eq!(runtime.focus(), Some(C));
    assert_pointer_events(
        runtime.app(),
        &[
            (B, Phase::Press),
            (B, Phase::Release),
            (B, Phase::Click),
            (A, Phase::Press),
            (A, Phase::Release),
            (A, Phase::Click),
            (C, Phase::Press),
            (C, Phase::Release),
            (C, Phase::Click),
        ],
    );
    assert!(runtime.diagnostics().is_empty());
}
