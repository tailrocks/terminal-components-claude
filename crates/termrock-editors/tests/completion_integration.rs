//! Completion and code-editor routing integration: the popup forwards
//! unclaimed keys to the embedded editor and accepts from its matches.

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;

use termrock_overlays::completion::{
    Completion, CompletionAction, CompletionController, CompletionState, Item,
};
use termrock_runtime::event::Input;
use termrock_runtime::keymap::KeyMap;
use termrock_runtime::runtime::stub::key;
use termrock_runtime::runtime::{App, Runtime};
use termrock_theme::Theme;
use termrock_editors::code::{CodeEditor, CodeEditorState};
use termrock_runtime::action::ActionKey;
use termrock_runtime::{Chord, Cx, Id, ItemKey, KeyCode, Response, TextBuffer, Ui};

const MOVE_DOWN: ActionKey = ActionKey::custom("completion.down");

const EDITOR: Id = Id::root("completion.tests.editor");
const POPUP: Id = Id::root("completion.tests.popup");
const AREA: Rect = Rect::new(0, 0, 80, 24);
const FIRST: ItemKey = ItemKey::num(1);
const SECOND: ItemKey = ItemKey::num(2);
const ITEMS: &[Item<'static>] = &[Item::new(FIRST, "alpha"), Item::new(SECOND, "beta")];

struct RoutingApp {
    editor: CodeEditorState,
    completion: CompletionState,
    keymap: KeyMap,
    open_next: bool,
    accepted: Option<ItemKey>,
    dismissed: bool,
}

impl App for RoutingApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if self.open_next {
            CompletionController::new(EDITOR, POPUP).request(
                cx,
                &mut self.completion,
                Rect::new(2, 1, 1, 1),
                0,
                ITEMS,
            );
            self.open_next = false;
        }
        let completion = Completion::<Item<'_>>::new(POPUP).update_for(
            EDITOR,
            cx,
            &mut self.completion,
            ITEMS,
        );
        match completion.action_ref() {
            Some(CompletionAction::Accepted(key)) => self.accepted = Some(key),
            Some(CompletionAction::Dismissed) => self.dismissed = true,
            Some(CompletionAction::Moved) | None => {}
        }
        let editor = CodeEditor::new(EDITOR, 4).update(cx, &mut self.editor);
        completion.erase() | editor.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        CodeEditor::new(EDITOR, 4).draw(ui, Rect::new(0, 0, 60, 4), &self.editor);
        if self.completion.is_open() {
            let _ = ui.layer(POPUP, |ui, area| {
                Completion::<Item<'_>>::new(POPUP).draw(ui, area, &self.completion, ITEMS);
            });
        }
    }

    fn keymap(&self) -> &KeyMap {
        &self.keymap
    }
}

fn routing_runtime() -> (Runtime<RoutingApp>, Buffer) {
    let mut runtime = Runtime::new(
        RoutingApp {
            editor: CodeEditorState::new("alpha\nbeta"),
            completion: CompletionState::default(),
            keymap: KeyMap::new(),
            open_next: false,
            accepted: None,
            dismissed: false,
        },
        Theme::junie(),
    );
    let _ = runtime.initialize();
    let mut buffer = Buffer::empty(AREA);
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
    runtime.set_focus(Some(EDITOR));
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
    let _ = termrock_runtime::stub::deliver(&mut runtime, key(KeyCode::Char('i')));
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
    runtime.app_mut().open_next = true;
    let _ = termrock_runtime::stub::deliver(&mut runtime, Input::Tick);
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
    (runtime, buffer)
}

struct ControllerApp {
    controller: CompletionController,
    completion: CompletionState,
    buffer: TextBuffer,
    request: bool,
    accept: bool,
    editor_motion: bool,
}

impl App for ControllerApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if self.request {
            self.controller
                .request(cx, &mut self.completion, Rect::new(2, 1, 1, 1), 3, ITEMS);
            self.request = false;
        }
        if self.accept {
            let _ =
                self.controller
                    .accept(cx, &mut self.completion, &mut self.buffer, ITEMS[0]);
            self.accept = false;
        }
        if self.editor_motion {
            self.controller
                .dismiss_on_editor_motion(cx, &mut self.completion);
            self.editor_motion = false;
        }
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        if self.completion.is_open() {
            let completion = Completion::<Item<'_>>::new(POPUP);
            let _ = ui.layer(POPUP, |ui, area| {
                completion.draw(ui, area, &self.completion, ITEMS);
            });
        }
    }
}

fn controller_runtime() -> Runtime<ControllerApp> {
    let mut runtime = Runtime::new(
        ControllerApp {
            controller: CompletionController::new(EDITOR, POPUP),
            completion: CompletionState::default(),
            buffer: TextBuffer::single("SEL cou"),
            request: false,
            accept: false,
            editor_motion: false,
        },
        Theme::junie(),
    );
    let _ = runtime.initialize();
    runtime
}

#[test]
fn completion_controller_accept_splices_and_dismisses_atomically() {
    let mut runtime = controller_runtime();
    let mut buffer = Buffer::empty(AREA);
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
    runtime.app_mut().request = true;
    let _ = termrock_runtime::stub::deliver(&mut runtime, Input::Tick);
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
    assert!(runtime.app().completion.is_open());
    assert!(runtime.is_open(POPUP));

    runtime.app_mut().accept = true;
    let _ = termrock_runtime::stub::deliver(&mut runtime, Input::Tick);

    assert_eq!(runtime.app().buffer.text(), "SEL alpha");
    assert!(!runtime.app().completion.is_open());
    assert!(!runtime.is_open(POPUP));
}

#[test]
fn completion_controller_dismisses_on_editor_motion() {
    let mut runtime = controller_runtime();
    let mut buffer = Buffer::empty(AREA);
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
    runtime.app_mut().request = true;
    let _ = termrock_runtime::stub::deliver(&mut runtime, Input::Tick);
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
    assert!(runtime.is_open(POPUP));

    runtime.app_mut().editor_motion = true;
    let _ = termrock_runtime::stub::deliver(&mut runtime, Input::Tick);

    assert!(!runtime.app().completion.is_open());
    assert!(!runtime.is_open(POPUP));
}

#[test]
fn distinct_insert_text_is_spliced() {
    let mut buffer = TextBuffer::single("SEL cou");
    let item = Item::new(ItemKey::num(1), "count").insert("COUNT(");
    assert!(CompletionController::splice(&mut buffer, 3, item));
    assert_eq!(buffer.text(), "SEL COUNT(");
}

#[test]
fn none_insert_uses_label() {
    let mut buffer = TextBuffer::single("SEL cou");
    let item = Item::new(ItemKey::num(1), "count");
    assert!(CompletionController::splice(&mut buffer, 3, item));
    assert_eq!(buffer.text(), "SEL count");
}

#[test]
fn focused_editor_down_moves_completion_not_editor() {
    let (mut runtime, mut buffer) = routing_runtime();
    let editor_offset = runtime.app().editor.cursor_offset();
    let _ = termrock_runtime::stub::deliver(&mut runtime, key(KeyCode::Down));
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
    assert_eq!(runtime.app().completion.cursor(), Some(SECOND));
    assert_eq!(runtime.app().editor.cursor_offset(), editor_offset);
}

#[test]
fn focused_editor_tab_and_enter_accept_completion() {
    for code in [KeyCode::Tab, KeyCode::Enter] {
        let (mut runtime, _) = routing_runtime();
        let _ = termrock_runtime::stub::deliver(&mut runtime, key(code));
        assert_eq!(runtime.app().accepted, Some(FIRST));
    }
}

#[test]
fn focused_editor_escape_dismisses_completion() {
    let (mut runtime, _) = routing_runtime();
    let _ = termrock_runtime::stub::deliver(&mut runtime, key(KeyCode::Esc));
    assert!(runtime.app().dismissed);
    assert!(!runtime.app().completion.is_open());
}

#[test]
fn ordinary_text_remains_owned_by_the_editor() {
    let (mut runtime, _) = routing_runtime();
    let _ = termrock_runtime::stub::deliver(&mut runtime, key(KeyCode::Char('x')));
    assert_eq!(runtime.app().editor.text(), "xalpha\nbeta");
    assert_eq!(runtime.app().completion.cursor(), Some(FIRST));
}

#[test]
fn editor_owned_completion_binding_honours_remap_and_remove() {
    let (mut runtime, mut buffer) = routing_runtime();
    runtime
        .app_mut()
        .keymap
        .remap_component(EDITOR, MOVE_DOWN, Chord::key(KeyCode::F(4)));
    let _ = termrock_runtime::stub::deliver(&mut runtime, key(KeyCode::Down));
    assert_eq!(runtime.app().completion.cursor(), Some(FIRST));
    let _ = termrock_runtime::stub::deliver(&mut runtime, key(KeyCode::F(4)));
    assert_eq!(runtime.app().completion.cursor(), Some(SECOND));

    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
    runtime.app_mut().keymap.remove_component(EDITOR, MOVE_DOWN);
    let _ = termrock_runtime::stub::deliver(&mut runtime, key(KeyCode::Up));
    let _ = termrock_runtime::stub::deliver(&mut runtime, key(KeyCode::F(4)));
    assert_eq!(runtime.app().completion.cursor(), Some(FIRST));
}

#[test]
fn pressed_part_targets_only_its_popup_row() {
    let pressed = Some(PartRef::item(Part::ROW, SECOND));
    assert!(!Completion::<Item<'static>>::pressed_target(pressed, FIRST,));
    assert!(Completion::<Item<'static>>::pressed_target(pressed, SECOND,));
}
