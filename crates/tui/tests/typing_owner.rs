//! Explicit typing ownership remains independent of navigation focus.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "fixture assertions"
)]
use junie_tui::{
    ActionKey, App, BlurPolicy, Chord, Cx, Focusability, Id, Input, Intent, Key, KeyCode, KeyMap,
    KeyModifiers, KeyPhase, LayerSpec, Rect, Response, Runtime, TextInput, TextInputState, Theme,
    TypingPolicy, Ui, UpdateCause,
};
use junie_tui_testing::{
    Scene,
    perf::{Counting, bench, lock},
};
use ratatui_core::buffer::Buffer;
#[global_allocator]
static ALLOCATOR: Counting = Counting;
const QUERY: Id = Id::root("typing.query");
const ROW: Id = Id::root("typing.row");
const OTHER: Id = Id::root("typing.other");
const MODAL: Id = Id::root("typing.modal");
const NESTED: Id = Id::root("typing.nested");
const NESTED_ROW: Id = Id::root("typing.nested.row");
const MODAL_ROW: Id = Id::root("typing.modal.row");
const QUIT: ActionKey = ActionKey::application("typing.quit");
const HELP: ActionKey = ActionKey::application("typing.help");
const NEXT: ActionKey = ActionKey::application("typing.next");
const PREV: ActionKey = ActionKey::application("typing.prev");
const ZERO: ActionKey = ActionKey::application("typing.zero");
const AREA: Rect = Rect::new(0, 0, 40, 8);
fn chord(code: KeyCode) -> Chord {
    Chord {
        code,
        mods: KeyModifiers::NONE,
    }
}
fn key(code: KeyCode) -> Input {
    Input::Key(Key {
        code,
        mods: KeyModifiers::NONE,
    })
}
#[derive(Default)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "independent routing fixture switches"
)]
struct Model {
    editor: TextInputState,
    value: String,
    other: TextInputState,
    other_value: String,
    map: KeyMap,
    commands: Vec<ActionKey>,
    row_keys: Vec<KeyCode>,
    updates: usize,
    readonly: bool,
    disabled: bool,
    removed: bool,
    second: bool,
    second_fallback: bool,
    open_modal: bool,
    popover: bool,
    nested: bool,
    second_readonly: bool,
    request_focus: Option<Id>,
}
impl Model {
    fn input(&self) -> TextInput<'_> {
        TextInput::new(QUERY)
            .value(&self.value)
            .blur(BlurPolicy::Keep)
            .typing_policy(TypingPolicy::Fallback { cursor: true })
            .read_only(self.readonly)
            .disabled(self.disabled)
    }
    fn query(&self) -> &str {
        self.editor.draft_text().unwrap_or(&self.value)
    }
    fn rebuild_map(&mut self) {
        let mut map = KeyMap::default()
            .bind(KeyPhase::Capture, chord(KeyCode::Down), NEXT)
            .bind(KeyPhase::Capture, chord(KeyCode::Up), PREV)
            .bind(KeyPhase::Capture, chord(KeyCode::Char('0')), ZERO);
        if self.query().is_empty() {
            map = map
                .bind_before_typing(QUERY, chord(KeyCode::Char('q')), QUIT)
                .bind_before_typing(QUERY, chord(KeyCode::Char('?')), HELP);
        }
        self.map = map;
    }
}
impl App for Model {
    fn keymap(&self) -> &KeyMap {
        &self.map
    }
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.updates = self.updates.saturating_add(1);
        if cx.update_cause() == UpdateCause::Bootstrap {
            self.editor.begin("");
            self.other.begin("");
            cx.focus(ROW);
        }
        if let Some(owner) = self.request_focus.take() {
            cx.focus(owner);
        }
        if self.open_modal {
            self.open_modal = false;
            cx.open_layer(
                MODAL,
                if self.popover {
                    LayerSpec::popover(
                        MODAL,
                        junie_tui::Anchor::Screen(junie_tui::ScreenAlign::Center),
                    )
                } else {
                    LayerSpec::modal(MODAL)
                },
            );
        }
        if self.nested {
            self.nested = false;
            cx.open_layer(NESTED, LayerSpec::modal(NESTED));
        }
        if let Some(command) = cx.command() {
            if command == NEXT {
                cx.focus_next();
            } else if command == PREV {
                cx.focus_prev();
            } else {
                self.commands.push(command);
            }
        }
        for intent in cx.intents(ROW) {
            if let Intent::Key(k) = intent {
                self.row_keys.push(k.code);
            }
        }
        let _ = TextInput::new(QUERY)
            .blur(BlurPolicy::Keep)
            .read_only(self.readonly)
            .disabled(self.disabled)
            .update(cx, &mut self.editor, &mut self.value);
        let _ = TextInput::new(OTHER).blur(BlurPolicy::Keep).update(
            cx,
            &mut self.other,
            &mut self.other_value,
        );
        self.rebuild_map();
        Response::ignored()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        if !self.removed {
            self.input().draw(ui, Rect::new(0, 0, 30, 1), &self.editor);
        }
        ui.register_control(ROW, Rect::new(0, 2, 30, 1), Focusability::Focusable);
        if self.second {
            TextInput::new(OTHER)
                .read_only(self.second_readonly)
                .value(&self.other_value)
                .blur(BlurPolicy::Keep)
                .typing_policy(if self.second_fallback {
                    TypingPolicy::Fallback { cursor: true }
                } else {
                    TypingPolicy::Focused
                })
                .draw(ui, Rect::new(0, 4, 30, 1), &self.other);
        }
        ui.layer(NESTED, |ui, _| {
            ui.register_control(NESTED_ROW, Rect::new(1, 6, 10, 1), Focusability::Focusable);
        });
        ui.layer(MODAL, |ui, _| {
            ui.register_control(MODAL_ROW, Rect::new(0, 6, 10, 1), Focusability::Focusable);
        });
    }
}
fn present(rt: &mut Runtime<Model>, buf: &mut Buffer) {
    for _ in 0..16 {
        rt.draw_buffer(AREA, buf).commit_presented();
        if !rt.needs_settle() && !rt.needs_present() {
            return;
        }
        let _ = rt.settle();
    }
    panic!("typing fixture did not settle");
}
fn runtime() -> (Runtime<Model>, Buffer) {
    let mut rt = Runtime::new(Model::default(), Theme::junie());
    let mut buf = Buffer::empty(AREA);
    let _ = rt.initialize();
    present(&mut rt, &mut buf);
    (rt, buf)
}
fn send(rt: &mut Runtime<Model>, buf: &mut Buffer, input: Input) {
    present(rt, buf);
    let _ = rt.handle(input).unwrap();
    present(rt, buf);
}
#[test]
fn fallback_edits_shared_unicode_draft_and_keeps_query_caret_without_moving_row_focus() {
    let (mut rt, mut buf) = runtime();
    assert_eq!(rt.focus(), Some(ROW));
    assert_eq!(rt.typing_owner(), Some(QUERY));
    assert_eq!(rt.cursor_position().unwrap().y, 0);
    for ch in ['é', '界', '0', ' '] {
        send(&mut rt, &mut buf, key(KeyCode::Char(ch)));
    }
    send(&mut rt, &mut buf, Input::Paste("🍀".into()));
    assert_eq!(rt.app().query(), "é界0 🍀");
    send(&mut rt, &mut buf, key(KeyCode::Backspace));
    assert_eq!(rt.app().query(), "é界0 ");
    assert_eq!(rt.focus(), Some(ROW));
    assert!(rt.app().commands.is_empty());
    assert!(rt.app().row_keys.is_empty());
    send(&mut rt, &mut buf, key(KeyCode::Enter));
    assert_eq!(rt.app().row_keys, [KeyCode::Enter]);
    assert!(rt.app().editor.is_editing());
}
#[test]
fn scoped_empty_commands_stop_matching_after_retained_paste_is_published() {
    let (mut rt, mut buf) = runtime();
    send(&mut rt, &mut buf, key(KeyCode::Char('?')));
    send(&mut rt, &mut buf, key(KeyCode::Char('q')));
    assert_eq!(rt.app().commands, [HELP, QUIT]);
    let _ = rt.handle(Input::Paste("secret".into())).unwrap();
    let pending = rt.handle(key(KeyCode::Char('q'))).unwrap_err();
    assert_eq!(rt.app().query(), "secret");
    assert_eq!(rt.typing_owner(), None);
    present(&mut rt, &mut buf);
    let _ = rt.handle(pending.into_input()).unwrap();
    assert_eq!(rt.app().query(), "secretq");
    assert_eq!(rt.app().commands, [HELP, QUIT]);
}
#[test]
fn primary_editor_wins_and_runtime_previous_next_preserve_kept_draft() {
    let (mut rt, mut buf) = runtime();
    send(&mut rt, &mut buf, key(KeyCode::Char('x')));
    send(&mut rt, &mut buf, key(KeyCode::Up));
    assert_eq!(rt.focus(), Some(QUERY));
    send(&mut rt, &mut buf, key(KeyCode::Down));
    assert_eq!(rt.focus(), Some(ROW));
    assert_eq!(rt.app().query(), "x");
    rt.app_mut().second = true;
    rt.app_mut().request_focus = Some(OTHER);
    send(&mut rt, &mut buf, Input::Tick);
    assert_eq!(rt.typing_owner(), Some(OTHER));
    send(&mut rt, &mut buf, Input::Paste("other".into()));
    assert_eq!(rt.app().other.draft_text(), Some("other"));
    assert_eq!(rt.app().query(), "x");
    assert_eq!(rt.cursor_position().unwrap().y, 4);
}
#[test]
fn readonly_disabled_removed_and_ambiguous_targets_never_receive_fallback_paste() {
    for mode in 0..4 {
        let (mut rt, mut buf) = runtime();
        match mode {
            0 => rt.app_mut().readonly = true,
            1 => rt.app_mut().disabled = true,
            2 => rt.app_mut().removed = true,
            _ => {
                rt.app_mut().second = true;
                rt.app_mut().second_fallback = true;
            }
        }
        present(&mut rt, &mut buf);
        assert_eq!(rt.typing_owner(), None);
        send(&mut rt, &mut buf, Input::Paste("blocked".into()));
        assert_eq!(rt.app().query(), "");
        assert_eq!(rt.app().other.draft_text(), Some(""));
        if mode == 3 {
            assert!(
                rt.diagnostics()
                    .iter()
                    .any(|d| matches!(d, junie_tui::Diagnostic::TypingTargetConflict { .. }))
            );
        }
    }
}
#[test]
fn modal_and_aborted_publication_cannot_reuse_background_typing_or_scoped_commands() {
    let (mut rt, mut buf) = runtime();
    rt.app_mut().removed = true;
    drop(rt.draw_buffer(AREA, &mut buf));
    assert_eq!(rt.typing_owner(), None);
    let pending = rt.handle(Input::Paste("pending".into())).unwrap_err();
    rt.app_mut().removed = false;
    rt.app_mut().open_modal = true;
    send(&mut rt, &mut buf, Input::Tick);
    assert_eq!(rt.focus(), Some(MODAL_ROW));
    assert_eq!(rt.typing_owner(), None);
    send(&mut rt, &mut buf, pending.into_input());
    send(&mut rt, &mut buf, key(KeyCode::Char('q')));
    assert_eq!(rt.app().query(), "");
    assert!(rt.app().commands.is_empty());
    assert_eq!(rt.cursor_position(), None);
}

#[test]
fn fallback_rich_editing_uses_the_shared_action_table() {
    let (mut rt, mut buf) = runtime();
    send(&mut rt, &mut buf, Input::Paste("alpha beta".into()));
    send(
        &mut rt,
        &mut buf,
        Input::Key(Key {
            code: KeyCode::Backspace,
            mods: KeyModifiers::CONTROL,
        }),
    );
    assert_eq!(rt.app().query(), "alpha ");
    send(
        &mut rt,
        &mut buf,
        Input::Key(Key {
            code: KeyCode::Char('l'),
            mods: KeyModifiers::CONTROL,
        }),
    );
    send(&mut rt, &mut buf, Input::Paste("replacement".into()));
    assert_eq!(rt.app().query(), "replacement");
    assert_eq!(rt.focus(), Some(ROW));
}
#[test]
fn bound_scene_preserves_fallback_cursor_cells_and_model_with_zero_warm_allocations() {
    let _lock = lock();
    let (mut rt, mut buf) = runtime();
    send(&mut rt, &mut buf, Input::Paste("stable".into()));
    let updates = rt.app().updates;
    let mut scene = Scene::new(
        "typing",
        Theme::junie(),
        junie_tui::ColorLevel::TrueColor,
        40,
        8,
    );
    scene.set_snapshot(rt.render_snapshot().unwrap());
    let mut bound = scene.bind_app(rt.app());
    bound.draw();
    let first = bound.scene().buffer().clone();
    let cursor = bound.scene().cursor_position();
    bound.draw();
    assert_eq!(bound.scene().buffer(), &first);
    assert_eq!(bound.scene().cursor_position(), cursor);
    assert_eq!(cursor, rt.cursor_position());
    let stats = bench(4, 100, &mut || bound.draw());
    assert_eq!((stats.allocs, stats.bytes), (0, 0));
    assert_eq!(rt.app().updates, updates);
    assert_eq!(rt.app().query(), "stable");
}

#[test]
fn component_override_is_resolved_for_fallback_without_redirecting_commit() {
    use junie_tui::{BindingState, Bindings, TextCmd};
    let (mut rt, mut buf) = runtime();
    send(&mut rt, &mut buf, Input::Paste("abc".into()));
    let table = TextInput::new(QUERY).bindings(BindingState::default());
    let backspace = table
        .iter()
        .find(|binding| binding.cmd == TextCmd::Backspace)
        .unwrap()
        .action;
    let commit = table
        .iter()
        .find(|binding| binding.cmd == TextCmd::Commit)
        .unwrap()
        .action;
    rt.app_mut()
        .map
        .remap_component(QUERY, backspace, chord(KeyCode::F(6)));
    rt.app_mut()
        .map
        .remap_component(QUERY, commit, chord(KeyCode::F(7)));
    present(&mut rt, &mut buf);
    let _ = rt.handle(key(KeyCode::F(6))).unwrap();
    assert_eq!(rt.app().query(), "ab");
    rt.app_mut()
        .map
        .remap_component(QUERY, commit, chord(KeyCode::F(7)));
    present(&mut rt, &mut buf);
    let _ = rt.handle(key(KeyCode::F(7))).unwrap();
    assert_eq!(rt.app().query(), "ab");
    assert!(rt.app().value.is_empty());
    assert_eq!(rt.app().row_keys, [KeyCode::F(7)]);
}

#[test]
fn filtered_author_binding_accepts_captured_predicate_and_publishes_only_selected_commands() {
    use junie_tui::{Binding, StateFlags};
    const ACTION: ActionKey = ActionKey::custom("typing.filtered");
    const TABLE: &[Binding<bool>] = &[Binding {
        action: ACTION,
        chord: Some(Chord::key(KeyCode::F(9))),
        cmd: true,
        label: "filtered",
        priority: 0,
        visible: false,
    }];
    struct Custom {
        enabled: bool,
        delivered: bool,
    }
    impl App for Custom {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            if cx.update_cause() == UpdateCause::Bootstrap {
                cx.focus(ROW);
            }
            for intent in cx.intents(QUERY) {
                if let Intent::Binding(action) = intent {
                    self.delivered |= action == ACTION;
                }
            }
            for _ in cx.intents(ROW) {}
            Response::ignored()
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            ui.register_editor(
                QUERY,
                Rect::new(0, 0, 10, 1),
                Focusability::Focusable,
                StateFlags::EDITING,
            );
            ui.publish_typing_target(QUERY, TABLE, |command| command == self.enabled, false);
            ui.register_control(ROW, Rect::new(0, 2, 10, 1), Focusability::Focusable);
        }
    }
    for enabled in [false, true] {
        let mut rt = Runtime::new(
            Custom {
                enabled,
                delivered: false,
            },
            Theme::junie(),
        );
        let _ = rt.initialize();
        let _ = junie_tui_testing::deliver(&mut rt, AREA, key(KeyCode::F(9)));
        assert_eq!(rt.app().delivered, enabled);
    }
}

#[test]
fn readonly_primary_blocks_fallback_and_focused_primary_initializes_its_own_draft() {
    for readonly in [false, true] {
        let (mut rt, mut buf) = runtime();
        rt.app_mut().second = true;
        rt.app_mut().second_readonly = readonly;
        if !readonly {
            rt.app_mut().other = TextInputState::default();
        }
        rt.app_mut().request_focus = Some(OTHER);
        send(&mut rt, &mut buf, Input::Tick);
        // A focused ordinary TextInput starts editing through its real FocusIn.
        // Readonly remains non-editable and blocks the background query.
        assert_eq!(rt.focus(), Some(OTHER));
        if readonly {
            assert_eq!(rt.typing_owner(), None);
        } else {
            assert_eq!(rt.typing_owner(), Some(OTHER));
        }
        send(&mut rt, &mut buf, Input::Paste("primary".into()));
        assert_eq!(rt.app().query(), "");
        if readonly {
            assert_eq!(rt.app().other.draft_text(), Some(""));
        }
    }
}
#[test]
fn nested_modal_top_owner_blocks_background_fallback_and_focus_previous_stays_trapped() {
    let (mut rt, mut buf) = runtime();
    rt.app_mut().open_modal = true;
    send(&mut rt, &mut buf, Input::Tick);
    rt.app_mut().nested = true;
    send(&mut rt, &mut buf, Input::Tick);
    assert_eq!(rt.focus(), Some(NESTED_ROW));
    send(&mut rt, &mut buf, key(KeyCode::Up));
    assert_eq!(rt.focus(), Some(NESTED_ROW));
    send(&mut rt, &mut buf, Input::Paste("nested".into()));
    assert_eq!(rt.app().query(), "");
    assert_eq!(rt.typing_owner(), None);
    assert_eq!(rt.cursor_position(), None);
}

#[test]
fn first_paste_waits_for_explicit_initialization_and_published_fallback() {
    let mut rt = Runtime::new(Model::default(), Theme::junie());
    let mut buf = Buffer::empty(AREA);
    let pending = rt.handle(Input::Paste("first-private".into())).unwrap_err();
    assert!(!format!("{pending:?}").contains("first-private"));
    rt.draw_buffer(AREA, &mut buf).commit_presented();
    assert_eq!(rt.app().updates, 0);
    let pending = rt.handle(pending.into_input()).unwrap_err();
    let _ = rt.initialize();
    present(&mut rt, &mut buf);
    let _ = rt.handle(pending.into_input()).unwrap();
    assert_eq!(rt.app().query(), "first-private");
    assert_eq!(rt.focus(), Some(ROW));
}

#[test]
fn warmed_live_fallback_presentation_allocates_nothing_and_performs_no_updates() {
    let _lock = lock();
    let (mut rt, mut buf) = runtime();
    let updates = rt.app().updates;
    let stats = bench(4, 100, &mut || {
        rt.draw_buffer(AREA, &mut buf).commit_presented();
    });
    assert_eq!((stats.allocs, stats.bytes), (0, 0));
    assert_eq!(rt.app().updates, updates);
    assert_eq!(rt.typing_owner(), Some(QUERY));
}

#[test]
fn noninert_popover_blocks_background_typing_even_when_navigation_focus_stays_on_page() {
    let (mut rt, mut buf) = runtime();
    rt.app_mut().popover = true;
    rt.app_mut().open_modal = true;
    send(&mut rt, &mut buf, Input::Tick);
    assert_eq!(rt.focus(), Some(ROW));
    assert_eq!(rt.typing_owner(), None);
    send(&mut rt, &mut buf, Input::Paste("popover".into()));
    send(&mut rt, &mut buf, key(KeyCode::Char('q')));
    assert_eq!(rt.app().query(), "");
    assert!(rt.app().commands.is_empty());
    assert_eq!(rt.cursor_position(), None);
}

#[test]
fn idle_primary_editor_prevents_fallback_without_manufacturing_an_edit_lifecycle() {
    struct Idle {
        pasted: bool,
    }
    impl App for Idle {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            if cx.update_cause() == UpdateCause::Bootstrap {
                cx.focus(OTHER);
            }
            for owner in [QUERY, OTHER] {
                for intent in cx.intents(owner) {
                    if matches!(intent, Intent::Paste(_)) {
                        self.pasted = true;
                    }
                }
            }
            Response::ignored()
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            ui.register_editor(
                QUERY,
                Rect::new(0, 0, 10, 1),
                Focusability::Focusable,
                junie_tui::StateFlags::EDITING,
            );
            ui.publish_typing_target(QUERY, &[] as &[junie_tui::Binding<()>], |()| true, true);
            ui.register_editor(
                OTHER,
                Rect::new(0, 2, 10, 1),
                Focusability::Focusable,
                junie_tui::StateFlags::empty(),
            );
        }
    }
    let mut rt = Runtime::new(Idle { pasted: false }, Theme::junie());
    let _ = rt.initialize();
    let _ = junie_tui_testing::deliver(&mut rt, AREA, Input::Paste("blocked".into()));
    assert!(!rt.app().pasted);
    assert_eq!(rt.focus(), Some(OTHER));
}
