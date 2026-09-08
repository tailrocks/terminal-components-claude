//! Keyboard editors retain published geometry without pointer authority.
#![allow(clippy::unwrap_used, reason = "test lifecycle assertions")]
use junie_tui::*;
use junie_tui_testing::{
    Scene,
    perf::{Counting, bench, lock},
};
use ratatui_core::buffer::Buffer;
#[global_allocator]
static ALLOCATOR: Counting = Counting;
const EDITOR: Id = Id::root("keyboard.editor");
const OTHER: Id = Id::root("keyboard.other");
const MODAL: Id = Id::root("keyboard.modal");
const AREA: Rect = Rect::new(0, 0, 40, 8);
const FIELD: Rect = Rect::new(2, 2, 12, 1);

struct Model {
    state: TextInputState,
    value: String,
    pointer: bool,
    fallback: bool,
    availability: Focusability,
    open: bool,
}
impl Model {
    fn input(&self) -> TextInput<'_> {
        TextInput::new(EDITOR)
            .pointer_enabled(self.pointer)
            .typing_policy(if self.fallback {
                TypingPolicy::Fallback { cursor: true }
            } else {
                TypingPolicy::Focused
            })
            .disabled(self.availability == Focusability::Disabled)
            .read_only(self.availability == Focusability::FocusableReadOnly)
            .blur(BlurPolicy::Keep)
    }
}
impl App for Model {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if cx.update_cause() == UpdateCause::Bootstrap {
            self.state.begin(&self.value);
            cx.focus(EDITOR);
        }
        if self.open {
            self.open = false;
            cx.open_layer(MODAL, LayerSpec::modal(MODAL).initial_focus(MODAL));
        }
        for owner in [OTHER, MODAL] {
            for _ in cx.intents(owner) {}
        }
        TextInput::new(EDITOR)
            .pointer_enabled(self.pointer)
            .disabled(self.availability == Focusability::Disabled)
            .read_only(self.availability == Focusability::FocusableReadOnly)
            .blur(BlurPolicy::Keep)
            .update(cx, &mut self.state, &mut self.value)
            .erase()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        self.input().value(&self.value).draw(ui, FIELD, &self.state);
        ui.register_control(OTHER, Rect::new(20, 2, 4, 1), Focusability::Focusable);
        ui.layer(MODAL, |ui, area| {
            ui.register_control(MODAL, area, Focusability::Focusable);
        });
    }
}
fn present<A: App>(rt: &mut Runtime<A>) {
    for _ in 0..12 {
        if rt.needs_settle() {
            let _ = rt.settle();
        }
        rt.draw_buffer(AREA, &mut Buffer::empty(AREA))
            .commit_presented();
        if !rt.needs_present() && !rt.needs_settle() {
            return;
        }
    }
    assert!(
        !rt.needs_present() && !rt.needs_settle(),
        "fixture did not settle"
    );
}
fn runtime(pointer: bool) -> Runtime<Model> {
    let mut rt = Runtime::new(
        Model {
            state: TextInputState::default(),
            value: "abcdefghijklmnopqrst".into(),
            pointer,
            fallback: false,
            availability: Focusability::Focusable,
            open: false,
        },
        Theme::junie(),
    );
    let _ = rt.initialize();
    present(&mut rt);
    rt
}
fn key(code: KeyCode) -> Input {
    Input::Key(Key {
        code,
        mods: KeyModifiers::NONE,
    })
}
fn mouse(kind: MouseKind) -> Input {
    Input::Mouse(Mouse {
        kind,
        pos: Position::new(4, 2),
        mods: KeyModifiers::NONE,
    })
}
fn send<A: App>(rt: &mut Runtime<A>, input: Input) {
    let _ = rt.handle(input).unwrap();
    present(rt);
}

#[test]
fn pointer_disabled_editor_keeps_geometry_caret_typing_and_paste() {
    let mut rt = runtime(false);
    assert_eq!(rt.area_of(EDITOR), Some(FIELD));
    assert!(
        rt.cursor_position()
            .is_some_and(|position| FIELD.contains(position))
    );
    assert!(
        rt.registry()
            .regions()
            .iter()
            .filter(|region| region.owner == EDITOR)
            .all(|region| !region.pointer_enabled())
    );
    assert!(rt.registry().hit(Position::new(4, 2)).is_none());
    send(&mut rt, mouse(MouseKind::Move));
    send(&mut rt, mouse(MouseKind::Down));
    send(&mut rt, mouse(MouseKind::Up));
    assert!(!rt.state_of(EDITOR).contains(StateFlags::HOVERED));
    send(&mut rt, key(KeyCode::Char('Z')));
    send(&mut rt, Input::Paste("!".into()));
    assert_eq!(rt.app().state.draft_text(), Some("abcdefghijklmnopqrstZ!"));
    send(&mut rt, key(KeyCode::Enter));
    assert_eq!(rt.app().value, "abcdefghijklmnopqrstZ!");
    assert!(rt.diagnostics().is_empty(), "{:?}", rt.diagnostics());
}

#[test]
fn default_pointer_path_still_repositions_caret_and_can_be_reenabled() {
    let mut rt = runtime(true);
    send(&mut rt, key(KeyCode::Home));
    send(&mut rt, key(KeyCode::End));
    send(&mut rt, mouse(MouseKind::Down));
    send(&mut rt, mouse(MouseKind::Up));
    send(&mut rt, key(KeyCode::Char('Z')));
    assert_ne!(rt.app().state.draft_text(), Some("abcdefghijklmnopqrstZ"));
    rt.app_mut().pointer = false;
    present(&mut rt);
    assert!(rt.registry().hit(Position::new(4, 2)).is_none());
    rt.app_mut().pointer = true;
    present(&mut rt);
    assert_eq!(
        rt.registry().hit(Position::new(4, 2)).map(|hit| hit.owner),
        Some(EDITOR)
    );
}

#[test]
fn retained_paste_uses_new_keyboard_publication_and_admissibility_barriers() {
    let mut rt = runtime(true);
    rt.app_mut().pointer = false;
    let pending = rt.handle(Input::Paste("queued".into())).unwrap_err();
    present(&mut rt);
    send(&mut rt, pending.into_input());
    assert!(rt.app().state.draft_text().unwrap().ends_with("queued"));
    for availability in [Focusability::Disabled, Focusability::FocusableReadOnly] {
        let mut rt = runtime(false);
        rt.app_mut().availability = availability;
        present(&mut rt);
        let before = rt.app().state.draft_text().map(str::to_owned);
        send(&mut rt, Input::Paste("blocked".into()));
        assert_eq!(rt.app().state.draft_text(), before.as_deref());
    }
    let mut rt = runtime(false);
    rt.app_mut().open = true;
    present(&mut rt);
    send(&mut rt, Input::Tick);
    let before = rt.app().state.draft_text().map(str::to_owned);
    send(&mut rt, Input::Paste("blocked".into()));
    assert_eq!(rt.app().state.draft_text(), before.as_deref());
    assert!(rt.diagnostics().is_empty(), "{:?}", rt.diagnostics());
}

#[test]
fn keyboard_only_fallback_keeps_caret_without_taking_navigation_focus() {
    let mut rt = runtime(false);
    rt.app_mut().fallback = true;
    present(&mut rt);
    send(&mut rt, key(KeyCode::Tab));
    assert_eq!(rt.focus(), Some(OTHER));
    assert!(
        rt.cursor_position()
            .is_some_and(|position| FIELD.contains(position))
    );
    send(&mut rt, Input::Paste("fallback".into()));
    assert!(rt.app().state.draft_text().unwrap().ends_with("fallback"));
    assert_eq!(rt.focus(), Some(OTHER));
    assert!(rt.diagnostics().is_empty(), "{:?}", rt.diagnostics());
}

#[derive(Default)]
struct Author {
    keyboard: bool,
    clicks: usize,
}
impl App for Author {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut press = false;
        for intent in cx.intents(EDITOR) {
            if let Intent::Pointer { phase, .. } = intent {
                press |= phase == Phase::Press;
                if phase == Phase::Click {
                    self.clicks = self.clicks.saturating_add(1);
                }
            }
        }
        if press {
            assert!(cx.capture(EDITOR, PartRef::of(Part::THUMB)));
        }
        for _ in cx.intents(OTHER) {}
        Response::ignored()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        ui.register_control(OTHER, FIELD, Focusability::ClickOnly);
        ui.register_part(EDITOR, PartRef::of(Part::THUMB), FIELD);
        if self.keyboard {
            ui.register_keyboard_editor(
                EDITOR,
                FIELD,
                Focusability::Focusable,
                StateFlags::EDITING,
            );
        } else {
            ui.register_editor(EDITOR, FIELD, Focusability::Focusable, StateFlags::EDITING);
        }
        ui.register_decor(EDITOR, PartRef::of(Part::LABEL), FIELD);
        ui.register_part(EDITOR, PartRef::of(Part::TEXT), FIELD);
        ui.register_scroll(
            EDITOR,
            FIELD,
            Axes::V,
            Headroom {
                up: 1,
                down: 1,
                ..Headroom::default()
            },
        );
    }
}

#[test]
fn owner_policy_covers_parts_before_and_after_declaration_and_cancels_capture() {
    let mut rt = Runtime::new(Author::default(), Theme::junie());
    let _ = rt.initialize();
    present(&mut rt);
    send(&mut rt, mouse(MouseKind::Down));
    assert_eq!(rt.capture_owner(), Some(EDITOR));
    rt.app_mut().keyboard = true;
    present(&mut rt);
    assert_eq!(rt.capture_owner(), None);
    assert_eq!(
        rt.registry().hit(Position::new(4, 2)).map(|hit| hit.owner),
        Some(OTHER)
    );
    assert!(
        rt.registry()
            .hit_scroll(Position::new(4, 2), Axis::V)
            .is_none()
    );
    assert!(
        rt.registry()
            .regions()
            .iter()
            .filter(|region| region.owner == EDITOR)
            .all(|region| !region.pointer_enabled())
    );
    assert_eq!(rt.area_of(EDITOR), Some(FIELD));
    assert_eq!(
        rt.area_of_part(EDITOR, PartRef::of(Part::THUMB)),
        Some(FIELD)
    );
    rt.app_mut().keyboard = false;
    present(&mut rt);
    send(&mut rt, mouse(MouseKind::Up));
    assert_eq!(rt.app().clicks, 0);
}

#[test]
fn warm_scene_and_live_keyboard_publication_allocate_nothing() {
    let _guard = lock();
    let mut rt = runtime(false);
    let mut buffer = Buffer::empty(AREA);
    let stats = bench(4, 100, &mut || {
        rt.draw_buffer(AREA, &mut buffer).commit_presented();
    });
    assert_eq!((stats.allocs, stats.bytes), (0, 0));
    let mut scene = Scene::new(
        "keyboard",
        Theme::junie(),
        ColorLevel::TrueColor,
        AREA.width,
        AREA.height,
    );
    scene.set_snapshot(rt.render_snapshot().unwrap());
    let mut projection = scene.bind_app(rt.app());
    let stats = bench(4, 100, &mut || projection.draw());
    assert_eq!((stats.allocs, stats.bytes), (0, 0));
}
