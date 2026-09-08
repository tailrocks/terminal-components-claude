//! Hover is a successfully presented geometry fact, never synthetic input.
#![allow(clippy::unwrap_used, reason = "test lifecycle assertions")]
use junie_tui::{
    App, ColorLevel, Cx, Focusability, FrameRead, Id, Input, Intent, Key, KeyCode, KeyModifiers,
    LayerSpec, Mouse, MouseKind, Phase, Position, Rect, Response, Runtime, StateFlags, Theme, Ui,
};
use junie_tui_testing::{
    Scene,
    perf::{Counting, bench, lock},
};
use ratatui_core::buffer::Buffer;
#[global_allocator]
static ALLOCATOR: Counting = Counting;
const A: Id = Id::root("pointer.a");
const B: Id = Id::root("pointer.b");
const POP: Id = Id::root("pointer.pop");
const AREA: Rect = Rect::new(0, 0, 30, 8);
#[derive(Clone, Default)]
struct Model {
    moved: bool,
    disabled: bool,
    layer: Option<bool>,
    updates: usize,
    moves: usize,
}
impl App for Model {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.updates = self.updates.saturating_add(1);
        for id in [A, B, POP] {
            for intent in cx.intents(id) {
                if matches!(
                    intent,
                    Intent::Pointer {
                        phase: Phase::Move,
                        ..
                    }
                ) {
                    self.moves = self.moves.saturating_add(1);
                }
            }
        }
        if let Some(open) = self.layer.take() {
            if open {
                cx.open_layer(
                    POP,
                    LayerSpec::popover(
                        POP,
                        junie_tui::Anchor::Screen(junie_tui::ScreenAlign::Center),
                    ),
                );
            } else {
                cx.close_layer(POP, None);
            }
        }
        Response::ignored()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        let a = Rect::new(if self.moved { 10 } else { 0 }, 0, 8, 2);
        let b = Rect::new(if self.moved { 0 } else { 10 }, 0, 8, 2);
        ui.register_control(
            A,
            a,
            if self.disabled {
                Focusability::Disabled
            } else {
                Focusability::Focusable
            },
        );
        ui.register_control(B, b, Focusability::Focusable);
        for (owner, area) in [(A, a), (B, b)] {
            let text = if ui.state(owner).contains(StateFlags::HOVERED) {
                "hover"
            } else {
                "plain"
            };
            ui.paint_str(area, text, junie_tui::Style::default());
        }
        ui.layer(POP, |ui, area| {
            ui.register_control(POP, area, Focusability::Focusable);
        });
    }
}
fn present(rt: &mut Runtime<Model>) {
    for _ in 0..8 {
        rt.draw_buffer(AREA, &mut Buffer::empty(AREA))
            .commit_presented();
        if !rt.needs_present() && !rt.needs_settle() {
            return;
        }
        if rt.needs_settle() {
            let _ = rt.settle();
        }
    }
    assert!(
        !rt.needs_present() && !rt.needs_settle(),
        "presentation failed to converge"
    );
}
fn runtime() -> Runtime<Model> {
    let mut rt = Runtime::new(Model::default(), Theme::junie());
    let _ = rt.initialize();
    present(&mut rt);
    rt
}
fn mouse(x: u16) -> Input {
    Input::Mouse(Mouse {
        kind: MouseKind::Move,
        pos: Position::new(x, 1),
        mods: KeyModifiers::NONE,
    })
}
fn send(rt: &mut Runtime<Model>, input: Input) {
    present(rt);
    let _ = rt.handle(input).unwrap();
    present(rt);
}
#[test]
fn moved_geometry_rehits_only_on_commit_and_retained_mouse_has_no_early_position_effect() {
    let mut rt = runtime();
    send(&mut rt, mouse(1));
    assert!(rt.state_of(A).contains(StateFlags::HOVERED));
    rt.app_mut().moved = true;
    let updates = rt.app().updates;
    let moves = rt.app().moves;
    drop(rt.draw_buffer(AREA, &mut Buffer::empty(AREA)));
    assert!(rt.state_of(A).contains(StateFlags::HOVERED));
    let pending = rt.handle(mouse(25)).unwrap_err();
    rt.draw_buffer(AREA, &mut Buffer::empty(AREA))
        .commit_presented();
    assert!(!rt.state_of(A).contains(StateFlags::HOVERED));
    assert!(rt.state_of(B).contains(StateFlags::HOVERED));
    assert!(rt.needs_present());
    assert_eq!((rt.app().updates, rt.app().moves), (updates, moves));
    present(&mut rt);
    assert_eq!((rt.app().updates, rt.app().moves), (updates, moves));
    let _ = rt.handle(pending.into_input()).unwrap();
    present(&mut rt);
    assert!(!rt.state_of(A).contains(StateFlags::HOVERED));
    assert!(!rt.state_of(B).contains(StateFlags::HOVERED));
}
#[test]
fn keyboard_suppression_survives_relayout_until_real_pointer_move() {
    let mut rt = runtime();
    send(&mut rt, mouse(1));
    send(
        &mut rt,
        Input::Key(Key {
            code: KeyCode::Char('x'),
            mods: KeyModifiers::NONE,
        }),
    );
    rt.app_mut().moved = true;
    present(&mut rt);
    assert!(!rt.state_of(A).contains(StateFlags::HOVERED));
    assert!(!rt.state_of(B).contains(StateFlags::HOVERED));
    send(&mut rt, mouse(1));
    assert!(rt.state_of(B).contains(StateFlags::HOVERED));
}
#[test]
fn disabled_hover_remains_admissible_and_layer_changes_rehit_stationary_position() {
    let mut rt = runtime();
    send(&mut rt, mouse(1));
    rt.app_mut().disabled = true;
    present(&mut rt);
    assert!(rt.state_of(A).contains(StateFlags::HOVERED));
    rt.app_mut().layer = Some(true);
    send(&mut rt, Input::Tick);
    assert!(!rt.state_of(A).contains(StateFlags::HOVERED));
    rt.app_mut().layer = Some(false);
    send(&mut rt, Input::Tick);
    assert!(rt.state_of(A).contains(StateFlags::HOVERED));
}
#[test]
fn frozen_scene_hover_is_unchanged_by_live_rehit_and_warm_publication_allocates_nothing() {
    let _lock = lock();
    let mut rt = runtime();
    send(&mut rt, mouse(1));
    let snapshot = rt.render_snapshot().unwrap();
    let model = rt.app().clone();
    let mut scene = Scene::new(
        "frozen_hover",
        Theme::junie(),
        ColorLevel::TrueColor,
        AREA.width,
        AREA.height,
    );
    scene.set_snapshot(snapshot);
    scene.draw_app(&model);
    let before = scene.buffer().clone();
    rt.app_mut().moved = true;
    present(&mut rt);
    scene.draw_app(&model);
    assert_eq!(scene.buffer(), &before);
    let updates = rt.app().updates;
    let mut buffer = Buffer::empty(AREA);
    let stats = bench(4, 100, &mut || {
        rt.draw_buffer(AREA, &mut buffer).commit_presented();
    });
    assert_eq!((stats.allocs, stats.bytes), (0, 0));
    assert_eq!(rt.app().updates, updates);
    assert!(!rt.needs_present());
}
