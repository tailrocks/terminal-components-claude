//! Completion and code-editor routing integration: the popup forwards
//! unclaimed keys to the embedded editor and accepts from its matches.

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;

use termrock_editors::code::{CodeEditor, CodeEditorState};
use termrock_overlays::completion::{
    Completion, CompletionAction, CompletionController, CompletionState, Item,
};
use termrock_runtime::action::ActionKey;
use termrock_runtime::event::{Input, KeyModifiers, Mouse, MouseKind};
use termrock_runtime::keymap::KeyMap;
use termrock_runtime::runtime::stub::key;
use termrock_runtime::runtime::{App, Runtime};
use termrock_runtime::{Chord, Cx, Id, ItemKey, KeyCode, Position, Response, TextBuffer, Ui};
use termrock_theme::Theme;

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
        let (completion, accepted, dismissed) = (
            &mut self.completion,
            &mut self.accepted,
            &mut self.dismissed,
        );
        let completion_update = Completion::<Item<'_>>::new(POPUP)
            .update_for(EDITOR, cx, completion, ITEMS)
            .on_action(|action| match action {
                CompletionAction::Accepted(key) => *accepted = Some(key),
                CompletionAction::Dismissed => *dismissed = true,
                CompletionAction::Moved => {}
            });
        let editor = CodeEditor::new(EDITOR, 4).update(cx, &mut self.editor);
        completion_update.erase() | editor.erase()
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
    items: &'static [Item<'static>],
}

impl App for ControllerApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if self.request {
            self.controller.request(
                cx,
                &mut self.completion,
                Rect::new(2, 1, 1, 1),
                3,
                self.items,
            );
            self.request = false;
        }
        if self.accept {
            let _ =
                self.controller
                    .accept(cx, &mut self.completion, &mut self.buffer, self.items[0]);
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
            items: ITEMS,
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

/// `CompletionController::splice` is internal; the observable splice contract
/// is `accept`, which splices the item's insertion text and dismisses in one
/// committed update.
fn splice_via_accept(items: &'static [Item<'static>]) -> Runtime<ControllerApp> {
    let mut runtime = controller_runtime();
    runtime.app_mut().items = items;
    runtime.app_mut().request = true;
    let _ = termrock_runtime::stub::deliver(&mut runtime, Input::Tick);
    runtime.app_mut().accept = true;
    let _ = termrock_runtime::stub::deliver(&mut runtime, Input::Tick);
    runtime
}

#[test]
fn distinct_insert_text_is_spliced() {
    const COUNT: &[Item<'static>] = &[Item::new(ItemKey::num(1), "count").insert("COUNT(")];
    let runtime = splice_via_accept(COUNT);
    assert_eq!(runtime.app().buffer.text(), "SEL COUNT(");
    assert!(!runtime.app().completion.is_open());
}

#[test]
fn none_insert_uses_label() {
    const COUNT: &[Item<'static>] = &[Item::new(ItemKey::num(1), "count")];
    let runtime = splice_via_accept(COUNT);
    assert_eq!(runtime.app().buffer.text(), "SEL count");
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
fn pointer_click_accepts_only_the_clicked_popup_row() {
    // `pressed_target` is internal; the observable contract is that a click
    // resolves to the semantic row under the pointer, and only that row.
    fn click(runtime: &mut Runtime<RoutingApp>, y: u16) {
        for kind in [MouseKind::Down, MouseKind::Up] {
            let _ = termrock_runtime::stub::deliver(
                runtime,
                Input::Mouse(Mouse {
                    kind,
                    pos: Position::new(4, y),
                    mods: KeyModifiers::NONE,
                }),
            );
        }
    }
    // The popover anchors below Rect::new(2, 1, 1, 1); after its border the
    // rows sit at y = 3 (alpha) and y = 4 (beta).
    let (mut runtime, _) = routing_runtime();
    click(&mut runtime, 4);
    assert_eq!(runtime.app().accepted, Some(SECOND));

    let (mut runtime, _) = routing_runtime();
    click(&mut runtime, 3);
    assert_eq!(runtime.app().accepted, Some(FIRST));

    let (mut runtime, _) = routing_runtime();
    click(&mut runtime, 2);
    assert_eq!(runtime.app().accepted, None);
}
