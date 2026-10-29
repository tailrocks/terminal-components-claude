use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use termrock_controls::Button;
use termrock_core::id::{Id, ItemKey};
use termrock_editors::{
    CodeAction, CodeEditor, CompletingCodeEditorState, CompletionBatch, CompletionProvider,
    CompletionTrigger, CompletionValue,
};
use termrock_runtime::event::{Input, Key, KeyCode, KeyModifiers, Mouse, MouseKind};
use termrock_runtime::keymap::KeyMap;
use termrock_runtime::runtime::{App, Runtime};
use termrock_runtime::{Cx, Response, Ui};
use termrock_theme::Theme;

const EDITOR: Id = Id::root("editor.live_find.editor");
const FIND: Id = Id::root("editor.live_find.button");
const POPUP: Id = Id::root("editor.live_find.popup");
const AREA: Rect = Rect::new(0, 0, 80, 8);
const DOC: &str = "alpha 界 e\u{301} beta alpha";

struct Provider;

impl CompletionProvider for Provider {
    fn complete(
        &self,
        text: &str,
        _cursor: usize,
        _trigger: CompletionTrigger,
    ) -> Option<CompletionBatch> {
        Some(CompletionBatch {
            items: vec![CompletionValue::new(
                ItemKey::num(1),
                "completion",
                "completion",
            )],
            replace_len: text.split_whitespace().last().unwrap_or_default().len(),
        })
    }
}

struct FindApp {
    state: CompletingCodeEditorState,
    provider: Provider,
    keymap: KeyMap,
    disabled: bool,
    read_only: bool,
    open_changed: Option<bool>,
    action: Option<CodeAction>,
    request_find: bool,
}

impl FindApp {
    fn editor(&self) -> termrock_editors::CompletingCodeEditor<'_> {
        CodeEditor::new(EDITOR, 6)
            .disabled(self.disabled)
            .read_only(self.read_only)
            .with_completion(POPUP, &self.provider)
    }

    fn record(&mut self, response: &Response<CodeAction>) {
        if let Some(action) = response.action_ref() {
            self.action = Some(action.clone());
        }
    }
}

impl App for FindApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if self.request_find {
            self.request_find = false;
            let editor = CodeEditor::new(EDITOR, 6)
                .disabled(self.disabled)
                .read_only(self.read_only)
                .with_completion(POPUP, &self.provider);
            let response = editor.open_find(cx, &mut self.state);
            self.open_changed = Some(response.is_changed());
            return response.erase();
        }
        if Button::new(FIND, "Find").update(cx).activated() {
            let editor = CodeEditor::new(EDITOR, 6)
                .disabled(self.disabled)
                .read_only(self.read_only)
                .with_completion(POPUP, &self.provider);
            let response = editor.open_find(cx, &mut self.state);
            self.open_changed = Some(response.is_changed());
            self.record(&response);
            return response.erase();
        }
        let editor = CodeEditor::new(EDITOR, 6)
            .disabled(self.disabled)
            .read_only(self.read_only)
            .with_completion(POPUP, &self.provider);
        let response = editor.update(cx, &mut self.state);
        self.open_changed = Some(response.is_changed());
        self.record(&response);
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Button::new(FIND, "Find").draw(ui, Rect::new(0, 0, 16, 1));
        self.editor().draw(ui, Rect::new(0, 2, 80, 6), &self.state);
    }

    fn keymap(&self) -> &KeyMap {
        &self.keymap
    }
}

fn runtime(disabled: bool, read_only: bool) -> Runtime<FindApp> {
    let mut runtime = Runtime::new(
        FindApp {
            state: CompletingCodeEditorState::new(DOC),
            provider: Provider,
            keymap: KeyMap::new(),
            disabled,
            read_only,
            open_changed: None,
            action: None,
            request_find: false,
        },
        Theme::junie(),
    );
    let _ = runtime.initialize();
    let mut buffer = Buffer::empty(AREA);
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
    runtime.set_focus(Some(EDITOR));
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
    runtime
}

fn draw(runtime: &mut Runtime<FindApp>, area: Rect) -> Buffer {
    let mut buffer = Buffer::empty(area);
    runtime.draw_buffer(area, &mut buffer).commit_presented();
    buffer
}

fn key(runtime: &mut Runtime<FindApp>, code: KeyCode, mods: KeyModifiers) {
    let _ = termrock_runtime::stub::deliver(runtime, Input::Key(Key { code, mods }));
    let _ = draw(runtime, AREA);
}

fn click_find(runtime: &mut Runtime<FindApp>) {
    for kind in [MouseKind::Down, MouseKind::Up] {
        let _ = termrock_runtime::stub::deliver(
            runtime,
            Input::Mouse(Mouse {
                kind,
                pos: ratatui_core::layout::Position::new(2, 0),
                mods: KeyModifiers::NONE,
            }),
        );
        let _ = draw(runtime, AREA);
    }
}

#[test]
fn keyboard_and_real_button_open_same_find_state_and_focus() {
    let mut via_key = runtime(false, false);
    key(&mut via_key, KeyCode::Char('f'), KeyModifiers::CONTROL);
    let keyboard_info = via_key.app().state.editor().find_info();
    assert_eq!(keyboard_info.map(|info| info.current), Some(None));
    assert_eq!(keyboard_info.map(|info| info.matches), Some(0));
    assert_eq!(via_key.focus(), Some(EDITOR));
    assert_eq!(via_key.app().open_changed, Some(true));

    let mut via_button = runtime(false, false);
    click_find(&mut via_button);
    let button_info = via_button.app().state.editor().find_info();
    assert_eq!(via_button.app().open_changed, Some(true));
    assert_eq!(via_button.focus(), Some(EDITOR));
    assert_eq!(keyboard_info, button_info);
}

#[test]
fn find_exposes_live_unicode_cursor_and_match_state_and_reopens_without_editing_document() {
    let mut runtime = runtime(false, true);
    let state = runtime.app_mut().state.editor_mut();
    state.jump_to("alpha 界 e\u{301}".len());
    assert_eq!(
        state.cursor_pos().col,
        10,
        "cell width counts wide and combining text"
    );
    state.jump_to(0);

    click_find(&mut runtime);
    for ch in "alpha".chars() {
        key(&mut runtime, KeyCode::Char(ch), KeyModifiers::NONE);
    }
    let frame = draw(&mut runtime, AREA);
    let footer = (0..AREA.width)
        .map(|x| {
            frame
                .cell((x, 7))
                .expect("footer cell belongs to the captured area")
                .symbol()
        })
        .collect::<Vec<_>>()
        .concat();
    assert!(footer.contains("find alpha"), "rendered footer: {footer:?}");
    let info = runtime
        .app()
        .state
        .editor()
        .find_info()
        .expect("find is open");
    assert_eq!(info.query, "alpha");
    assert_eq!(info.current, Some(0));
    assert_eq!(info.matches, 2);
    assert!(info.typing);

    key(&mut runtime, KeyCode::Enter, KeyModifiers::NONE);
    let accepted = runtime
        .app()
        .state
        .editor()
        .find_info()
        .expect("find stays visible");
    assert_eq!(accepted.current, Some(1));
    assert!(!accepted.typing);
    assert_eq!(
        runtime.app().state.editor().cursor_offset(),
        DOC.rfind("alpha").unwrap()
    );
    assert_eq!(runtime.app().state.editor().text(), DOC);

    runtime.app_mut().request_find = true;
    let _ = termrock_runtime::stub::deliver(&mut runtime, Input::Tick);
    let _ = draw(&mut runtime, AREA);
    let reopened = runtime
        .app()
        .state
        .editor()
        .find_info()
        .expect("find reopened");
    assert_eq!(reopened.query, "alpha");
    assert_eq!(reopened.current, Some(1));
    assert_eq!(reopened.matches, 2);
    assert!(reopened.typing);

    key(&mut runtime, KeyCode::Esc, KeyModifiers::NONE);
    assert_eq!(runtime.app().state.editor().find_info(), None);
    assert_eq!(runtime.app().state.editor().text(), DOC);

    let _ = draw(&mut runtime, Rect::new(0, 0, 0, 0));
    let _ = draw(&mut runtime, Rect::new(0, 0, 24, 4));
}
#[test]
fn opening_find_dismisses_completion_and_disabled_editor_ignores_request() {
    let mut active_runtime = runtime(false, false);
    key(
        &mut active_runtime,
        KeyCode::Char(' '),
        KeyModifiers::CONTROL,
    );
    assert!(active_runtime.app().state.is_completion_open());
    assert!(active_runtime.is_open(POPUP));
    active_runtime.app_mut().request_find = true;
    let _ = termrock_runtime::stub::deliver(&mut active_runtime, Input::Tick);
    let _ = draw(&mut active_runtime, AREA);
    assert!(!active_runtime.app().state.is_completion_open());
    assert!(!active_runtime.is_open(POPUP));
    assert!(active_runtime.app().state.editor().find_info().is_some());

    let mut disabled_runtime = runtime(true, false);
    disabled_runtime.app_mut().request_find = true;
    let focus_before = disabled_runtime.focus();
    let _ = termrock_runtime::stub::deliver(&mut disabled_runtime, Input::Tick);
    let _ = draw(&mut disabled_runtime, AREA);
    assert_eq!(disabled_runtime.app().open_changed, Some(false));
    assert_eq!(disabled_runtime.app().state.editor().find_info(), None);
    assert_eq!(disabled_runtime.focus(), focus_before);
}
