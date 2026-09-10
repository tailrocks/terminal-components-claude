//! A narrow physical activation-key origin exists only in its admitted event passes.
#![allow(clippy::unwrap_used, reason = "test lifecycle assertions")]
use junie_tui::{
    ActionKey, ActivationKey, App, Chord, Cx, Focusability, Id, Input, Key, KeyCode, KeyMap,
    KeyModifiers, KeyPhase, Mouse, MouseKind, Position, Rect, Response, Runtime, Theme, Ui,
    UpdateCause,
};
use ratatui_core::buffer::Buffer;
const A: Id = Id::root("origin.a");
const B: Id = Id::root("origin.b");
const COMMAND: ActionKey = ActionKey::custom("origin.command");
const AREA: Rect = Rect::new(0, 0, 20, 4);
#[derive(Default)]
struct Model {
    map: KeyMap,
    seen: Vec<(UpdateCause, Option<ActivationKey>, Option<ActionKey>)>,
}
impl App for Model {
    fn keymap(&self) -> &KeyMap {
        &self.map
    }
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.seen
            .push((cx.update_cause(), cx.activation_key(), cx.command()));
        for id in [A, B] {
            for _ in cx.intents(id) {}
        }
        if cx.command() == Some(COMMAND) {
            cx.focus(B);
            return Response::consumed();
        }
        Response::ignored()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        ui.register_control(A, Rect::new(0, 0, 4, 1), Focusability::Focusable);
        ui.register_control(B, Rect::new(5, 0, 4, 1), Focusability::Focusable);
    }
}
fn present(rt: &mut Runtime<Model>) {
    for _ in 0..8 {
        rt.draw_buffer(AREA, &mut Buffer::empty(AREA))
            .commit_presented();
        if !rt.needs_settle() {
            return;
        }
        let _ = rt.settle();
    }
    assert!(!rt.needs_settle());
}
fn key(code: KeyCode, mods: KeyModifiers) -> Input {
    Input::Key(Key { code, mods })
}
#[test]
fn queued_key_has_origin_only_on_successful_retry_and_never_bootstrap_or_settle() {
    let mut rt = Runtime::new(Model::default(), Theme::junie());
    let pending = rt
        .handle(key(KeyCode::Enter, KeyModifiers::NONE))
        .unwrap_err();
    assert!(rt.app().seen.is_empty());
    let _ = rt.initialize();
    present(&mut rt);
    assert!(rt.app().seen.iter().all(|(_, origin, _)| origin.is_none()));
    let _ = rt.handle(pending.into_input()).unwrap();
    let pending = rt.handle(Input::Paste("private".into())).unwrap_err();
    let events = rt
        .app()
        .seen
        .iter()
        .filter(|(_, origin, _)| origin.is_some())
        .count();
    assert_eq!(events, 1);
    present(&mut rt);
    let _ = rt.handle(pending.into_input()).unwrap();
    assert_eq!(
        rt.app().seen.last().map(|(_, origin, _)| *origin),
        Some(None)
    );
    let _ = rt.initialize();
    let _ = rt.settle();
    assert_eq!(
        rt.app()
            .seen
            .iter()
            .filter(|(_, origin, _)| origin.is_some())
            .count(),
        1
    );
    assert!(
        rt.activation_feedback().is_none(),
        "origin alone cannot activate feedback"
    );
}
#[test]
fn capture_and_bubble_event_passes_share_origin_but_focus_reruns_have_none() {
    for phase in [KeyPhase::Capture, KeyPhase::Bubble] {
        let model = Model {
            map: KeyMap::new().bind(phase, Chord::key(KeyCode::Enter), COMMAND),
            ..Model::default()
        };
        let mut rt = Runtime::new(model, Theme::junie());
        let _ = rt.initialize();
        present(&mut rt);
        let before = rt.app().seen.len();
        let _ = rt.handle(key(KeyCode::Enter, KeyModifiers::NONE)).unwrap();
        let seen = rt.app().seen.get(before..).unwrap();
        assert!(
            seen.iter()
                .any(|(cause, origin, cmd)| *cause == UpdateCause::Event
                    && *origin == Some(ActivationKey::Enter)
                    && *cmd == Some(COMMAND))
        );
        assert!(
            seen.iter()
                .any(|(cause, origin, _)| *cause == UpdateCause::Settle && origin.is_none())
        );
        assert!(
            seen.iter()
                .all(|(cause, origin, _)| if *cause == UpdateCause::Event {
                    *origin == Some(ActivationKey::Enter)
                } else {
                    origin.is_none()
                })
        );
        assert_eq!(
            seen.iter()
                .filter(|(cause, _, _)| *cause == UpdateCause::Event)
                .count(),
            if phase == KeyPhase::Bubble { 2 } else { 1 }
        );
    }
}
#[test]
fn modified_other_text_mouse_paste_resize_and_tick_have_no_origin_after_plain_space() {
    let mut rt = Runtime::new(Model::default(), Theme::junie());
    let _ = rt.initialize();
    present(&mut rt);
    let _ = rt
        .handle(key(KeyCode::Char(' '), KeyModifiers::NONE))
        .unwrap();
    assert_eq!(
        rt.app().seen.last().map(|(_, origin, _)| *origin),
        Some(Some(ActivationKey::Space))
    );
    for input in [
        key(KeyCode::Enter, KeyModifiers::SHIFT),
        key(KeyCode::Char(' '), KeyModifiers::CONTROL),
        key(KeyCode::Char('x'), KeyModifiers::NONE),
        Input::Paste("secret".into()),
        Input::Mouse(Mouse {
            kind: MouseKind::Move,
            pos: Position::new(18, 3),
            mods: KeyModifiers::NONE,
        }),
        Input::Resize(20, 4),
        Input::Tick,
    ] {
        present(&mut rt);
        let before = rt.app().seen.len();
        let _ = rt.handle(input).unwrap();
        assert!(
            rt.app()
                .seen
                .get(before..)
                .unwrap()
                .iter()
                .all(|(_, origin, _)| origin.is_none())
        );
    }
}
#[test]
#[cfg(feature = "crossterm")]
fn normalization_admits_press_and_repeat_and_drops_release_without_origin() {
    use crossterm::event::{Event, KeyEvent, KeyEventKind};
    use ratatui_crossterm::crossterm;
    let mut rt = Runtime::new(Model::default(), Theme::junie());
    let _ = rt.initialize();
    present(&mut rt);
    for (code, expected) in [
        (crossterm::event::KeyCode::Enter, ActivationKey::Enter),
        (crossterm::event::KeyCode::Char(' '), ActivationKey::Space),
    ] {
        for kind in [KeyEventKind::Press, KeyEventKind::Repeat] {
            let event = Event::Key(KeyEvent::new_with_kind(
                code,
                crossterm::event::KeyModifiers::NONE,
                kind,
            ));
            let input = Input::from_crossterm(event).unwrap();
            present(&mut rt);
            let _ = rt.handle(input).unwrap();
            assert_eq!(
                rt.app().seen.last().map(|(_, origin, _)| *origin),
                Some(Some(expected))
            );
        }
        let event = Event::Key(KeyEvent::new_with_kind(
            code,
            crossterm::event::KeyModifiers::NONE,
            KeyEventKind::Release,
        ));
        assert!(Input::from_crossterm(event).is_none());
    }
}
