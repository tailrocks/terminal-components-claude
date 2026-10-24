#![allow(clippy::unwrap_used, reason = "test lifecycle assertions")]
use ratatui_core::buffer::Buffer;
use termrock_fields::{BlurPolicy, FocusSelect, TextInput, TextInputState};
use termrock_runtime::*;
use termrock_test_support::{
    Scene,
    perf::{Counting, bench, lock},
};
use termrock_theme::{Family, Role, StylePatch};
#[global_allocator]
static ALLOCATOR: Counting = Counting;
const EDITOR: Id = Id::root("keyboard.editor");
const OTHER: Id = Id::root("keyboard.other");
const MODAL: Id = Id::root("keyboard.modal");
const AREA: Rect = Rect::new(0, 0, 40, 8);
const FIELD: Rect = Rect::new(2, 2, 12, 1);
const FOCUS_SOURCE: Id = Id::root("keyboard.focus-source");
const FOCUS_TARGET: Id = Id::root("keyboard.focus-target");
const FOCUS_AFTER: Id = Id::root("keyboard.focus-after");
const FOCUS_MODAL: Id = Id::root("keyboard.focus-modal");
const FOCUS_SOURCE_AREA: Rect = Rect::new(0, 0, 12, 1);
const FOCUS_TARGET_AREA: Rect = Rect::new(16, 0, 18, 1);
const FOCUS_AFTER_AREA: Rect = Rect::new(35, 0, 5, 1);

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

struct FocusSelectModel {
    source: TextInputState,
    target: TextInputState,
    after: TextInputState,
    source_value: String,
    target_value: String,
    after_value: String,
    initial_target_focus: bool,
    focus_policy: FocusSelect,
    target_availability: Focusability,
    open_focus_modal: bool,
}

impl FocusSelectModel {
    fn target_input(policy: FocusSelect, availability: Focusability) -> TextInput<'static> {
        TextInput::new(FOCUS_TARGET)
            .focus_select(policy)
            .disabled(availability == Focusability::Disabled)
            .read_only(availability == Focusability::FocusableReadOnly)
            .blur(BlurPolicy::Keep)
    }
}

impl App for FocusSelectModel {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if cx.update_cause() == UpdateCause::Bootstrap {
            cx.focus(if self.initial_target_focus {
                FOCUS_TARGET
            } else {
                FOCUS_SOURCE
            });
        }
        if self.open_focus_modal {
            self.open_focus_modal = false;
            cx.open_layer(
                FOCUS_MODAL,
                LayerSpec::modal(FOCUS_MODAL).initial_focus(FOCUS_MODAL),
            );
        }
        for _ in cx.intents(FOCUS_MODAL) {}
        let source = TextInput::new(FOCUS_SOURCE).blur(BlurPolicy::Keep).update(
            cx,
            &mut self.source,
            &mut self.source_value,
        );
        let target = Self::target_input(self.focus_policy, self.target_availability).update(
            cx,
            &mut self.target,
            &mut self.target_value,
        );
        let after = TextInput::new(FOCUS_AFTER).blur(BlurPolicy::Keep).update(
            cx,
            &mut self.after,
            &mut self.after_value,
        );
        source.erase() | target.erase() | after.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        TextInput::new(FOCUS_SOURCE).value(&self.source_value).draw(
            ui,
            FOCUS_SOURCE_AREA,
            &self.source,
        );
        Self::target_input(self.focus_policy, self.target_availability)
            .value(&self.target_value)
            .draw(ui, FOCUS_TARGET_AREA, &self.target);
        TextInput::new(FOCUS_AFTER).value(&self.after_value).draw(
            ui,
            FOCUS_AFTER_AREA,
            &self.after,
        );
        ui.layer(FOCUS_MODAL, |ui, area| {
            ui.register_control(FOCUS_MODAL, area, Focusability::Focusable);
        });
    }
}

fn focus_select_runtime_for(
    initial_target_focus: bool,
    target_value: &str,
    focus_policy: FocusSelect,
) -> Runtime<FocusSelectModel> {
    focus_select_runtime_for_availability(
        initial_target_focus,
        target_value,
        focus_policy,
        Focusability::Focusable,
    )
}

fn focus_select_runtime_for_availability(
    initial_target_focus: bool,
    target_value: &str,
    focus_policy: FocusSelect,
    target_availability: Focusability,
) -> Runtime<FocusSelectModel> {
    let mut runtime = Runtime::new(
        FocusSelectModel {
            source: TextInputState::default(),
            target: TextInputState::default(),
            after: TextInputState::default(),
            source_value: "source".into(),
            target_value: target_value.into(),
            after_value: "after".into(),
            initial_target_focus,
            focus_policy,
            target_availability,
            open_focus_modal: false,
        },
        Theme::junie().override_family(Family::INPUT, |recipe| {
            recipe
                .part(Part::ROW)
                .when(StateFlags::SELECTED, StylePatch::new().set_fg(Role::Danger));
        }),
    );
    let _ = runtime.initialize();
    present(&mut runtime);
    runtime
}

fn focus_select_runtime(initial_target_focus: bool) -> Runtime<FocusSelectModel> {
    focus_select_runtime_for(initial_target_focus, "orders-prod", FocusSelect::SelectAll)
}

#[test]
fn focus_select_all_is_opt_in_and_replaces_first_key() {
    let mut runtime = focus_select_runtime(false);
    assert_eq!(runtime.focus(), Some(FOCUS_SOURCE));
    send(&mut runtime, key(KeyCode::Tab));
    assert_eq!(runtime.focus(), Some(FOCUS_TARGET));
    assert_eq!(
        runtime.app().target.selection(),
        Some(0..runtime.app().target_value.len())
    );
    let mut frame = Buffer::empty(AREA);
    runtime.draw_buffer(AREA, &mut frame).commit_presented();
    assert_ne!(frame[(18, 0)].fg, frame[(31, 0)].fg);
    send(&mut runtime, key(KeyCode::Char('x')));
    assert_eq!(runtime.app().target.draft_text(), Some("x"));

    let mut default = Runtime::new(
        Model {
            state: TextInputState::default(),
            value: "unchanged".into(),
            pointer: false,
            fallback: false,
            availability: Focusability::Focusable,
            open: false,
        },
        Theme::junie(),
    );
    let _ = default.initialize();
    present(&mut default);
    assert_eq!(default.app().state.draft_text(), Some("unchanged"));
    assert_eq!(default.app().state.selection(), None);
    assert_eq!(default.app().state.cursor_offset(), Some(9));
}

#[test]
fn focus_select_all_preserves_programmatic_and_pointer_ownership() {
    let programmatic = focus_select_runtime(true);
    assert_eq!(programmatic.focus(), Some(FOCUS_TARGET));
    assert_eq!(
        programmatic.app().target.selection(),
        Some(0..programmatic.app().target_value.len())
    );

    let mut pointer = focus_select_runtime(false);
    let _ = pointer
        .handle(Input::Mouse(Mouse {
            kind: MouseKind::Down,
            pos: Position::new(22, 0),
            mods: KeyModifiers::NONE,
        }))
        .unwrap();
    present(&mut pointer);
    assert_eq!(pointer.focus(), Some(FOCUS_TARGET));
    assert_eq!(pointer.app().target.selection(), None);
    assert_eq!(pointer.app().target.cursor_offset(), Some(4));
}

#[test]
fn focus_select_all_keeps_selection_when_keep_policy_restores_focus() {
    let mut runtime = focus_select_runtime(false);
    send(&mut runtime, key(KeyCode::Tab));
    let selection = Some(0..runtime.app().target_value.len());
    assert_eq!(runtime.app().target.selection(), selection);
    send(&mut runtime, key(KeyCode::Tab));
    assert_eq!(runtime.focus(), Some(FOCUS_AFTER));
    assert_eq!(runtime.app().target.selection(), selection);
    send(&mut runtime, key(KeyCode::BackTab));
    assert_eq!(runtime.focus(), Some(FOCUS_TARGET));
    assert_eq!(runtime.app().target.selection(), selection);
}

#[test]
fn focus_select_policy_applies_when_layer_restores_focus_to_idle_input() {
    let mut runtime = focus_select_runtime(false);
    send(&mut runtime, key(KeyCode::Tab));
    runtime.app_mut().target.cancel();
    runtime.app_mut().open_focus_modal = true;
    present(&mut runtime);
    send(&mut runtime, Input::Tick);
    assert_eq!(runtime.focus(), Some(FOCUS_MODAL));

    send(&mut runtime, key(KeyCode::Esc));
    assert_eq!(runtime.focus(), Some(FOCUS_TARGET));
    assert_eq!(
        runtime.app().target.selection(),
        Some(0..runtime.app().target_value.len())
    );
}

#[test]
fn focus_select_all_uses_grapheme_safe_byte_range_for_unicode_values() {
    let value = "e\u{301}🙂界";
    let mut runtime = focus_select_runtime_for(false, value, FocusSelect::SelectAll);
    send(&mut runtime, key(KeyCode::Tab));
    assert_eq!(runtime.app().target.selection(), Some(0..value.len()));
    send(&mut runtime, key(KeyCode::Char('x')));
    assert_eq!(runtime.app().target.draft_text(), Some("x"));
}

#[test]
fn caret_start_focus_policy_is_opt_in() {
    let mut runtime = focus_select_runtime_for(false, "orders-prod", FocusSelect::CaretStart);
    send(&mut runtime, key(KeyCode::Tab));
    assert_eq!(runtime.app().target.cursor_offset(), Some(0));
    assert_eq!(runtime.app().target.selection(), None);
    let runtime = focus_select_runtime_for(true, "orders-prod", FocusSelect::CaretStart);
    assert_eq!(runtime.app().target.cursor_offset(), Some(0));
    assert_eq!(runtime.app().target.selection(), None);
}

#[test]
fn focus_selection_never_starts_read_only_or_disabled_editing() {
    for availability in [Focusability::FocusableReadOnly, Focusability::Disabled] {
        let runtime = focus_select_runtime_for_availability(
            true,
            "orders-prod",
            FocusSelect::SelectAll,
            availability,
        );
        assert!(!runtime.app().target.is_editing(), "{availability:?}");
        assert_eq!(runtime.app().target.selection(), None, "{availability:?}");
    }
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
