use ratatui_core::layout::Rect;
use termrock_fields::{EditCapabilities, EditMenuRequest, TextEditCommand};

use crate::action::ActionKey;
use crate::event::{Chord, KeyCode, KeyModifiers};
use crate::id::Id;
use crate::layer::DismissReason;
use crate::menu::{ContextMenu, MenuAction, MenuItem, MenuState};
use crate::response::Response;
use crate::ui::{Cx, Ui};

const CUT: ActionKey = ActionKey::custom("edit-menu.cut");
const COPY: ActionKey = ActionKey::custom("edit-menu.copy");
const PASTE: ActionKey = ActionKey::custom("edit-menu.paste");
const SELECT_ALL: ActionKey = ActionKey::custom("edit-menu.select-all");

const ITEMS: [MenuItem<'static>; 4] = [
    MenuItem::new(CUT, "Cut").chord(Chord::with(KeyCode::Char('x'), KeyModifiers::CONTROL)),
    MenuItem::new(COPY, "Copy").chord(Chord::with(KeyCode::Char('c'), KeyModifiers::CONTROL)),
    MenuItem::new(PASTE, "Paste")
        .chord(Chord::with(KeyCode::Char('v'), KeyModifiers::CONTROL))
        .separator(),
    MenuItem::new(SELECT_ALL, "Select all")
        .chord(Chord::with(KeyCode::Char('a'), KeyModifiers::CONTROL)),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditMenuAction {
    Command(TextEditCommand),
    Closed(DismissReason),
}

#[derive(Clone, Debug)]
pub struct EditMenuState {
    request: Option<EditMenuRequest>,
    menu: MenuState,
    clipboard_has_text: bool,
    items: Vec<MenuItem<'static>>,
}

impl Default for EditMenuState {
    fn default() -> Self {
        Self {
            request: None,
            menu: MenuState::default(),
            clipboard_has_text: false,
            items: ITEMS.to_vec(),
        }
    }
}

impl EditMenuState {
    pub const fn is_open(&self) -> bool {
        self.request.is_some()
    }
}

/// Context edit menu controller for a field owner.
///
/// The field remains the only owner of its editor state; this adapter yields a
/// typed command after `ContextMenu` has dismissed its layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditMenu {
    owner: Id,
    id: Id,
}

impl EditMenu {
    pub const fn new(owner: Id) -> Self {
        Self {
            owner,
            id: owner.sub("edit-menu"),
        }
    }

    pub const fn id(&self) -> Id {
        self.id
    }

    pub fn is_open(&self, cx: &Cx<'_>) -> bool {
        cx.is_open(self.id)
    }

    pub fn open(
        &self,
        cx: &mut Cx<'_>,
        state: &mut EditMenuState,
        request: EditMenuRequest,
        capabilities: EditCapabilities,
    ) {
        if request.owner != self.owner {
            return;
        }
        state.request = Some(request);
        state.menu = MenuState::default();
        state.clipboard_has_text = cx.clipboard_text().is_some_and(|text| !text.is_empty());
        state.items = menu_items(capabilities, state.clipboard_has_text);
        let menu = self.menu(request, &state.items);
        cx.open_layer(self.id, menu.layer(cx));
    }

    pub fn update(
        &self,
        cx: &mut Cx<'_>,
        state: &mut EditMenuState,
        capabilities: EditCapabilities,
    ) -> Response<EditMenuAction> {
        let Some(request) = state.request else {
            return Response::ignored();
        };
        state.clipboard_has_text = cx.clipboard_text().is_some_and(|text| !text.is_empty());
        state.items = menu_items(capabilities, state.clipboard_has_text);
        let menu = self.menu(request, &state.items);
        let response = menu.update(cx, &mut state.menu);
        let result = match response.action_ref().copied() {
            Some(MenuAction::Chosen(action)) => {
                let Some(command) = command_for(action) else {
                    state.request = None;
                    return Response::ignored().repaint().for_id(self.owner);
                };
                state.request = None;
                Some(EditMenuAction::Command(command))
            }
            Some(MenuAction::Closed(reason)) => {
                state.request = None;
                Some(EditMenuAction::Closed(reason))
            }
            Some(MenuAction::Opened(_) | MenuAction::Submenu(_)) | None => None,
        };
        if let Some(action) = result {
            return Response::action(action).for_id(self.owner);
        }
        if response.is_changed() {
            Response::changed().for_id(self.owner)
        } else if response.is_consumed() {
            Response::consumed().for_id(self.owner)
        } else {
            Response::ignored().for_id(self.owner)
        }
    }

    pub fn draw(&self, ui: &mut Ui<'_>, state: &EditMenuState) {
        let Some(request) = state.request else {
            return;
        };
        let menu = self.menu(request, &state.items);
        let _ = ui.layer(self.id, |ui, area: Rect| menu.draw(ui, area, &state.menu));
    }

    fn menu<'a>(
        &self,
        request: EditMenuRequest,
        items: &'a [MenuItem<'static>],
    ) -> ContextMenu<'a> {
        ContextMenu::at(self.id, items, request.anchor)
    }
}

fn menu_items(capabilities: EditCapabilities, clipboard_has_text: bool) -> Vec<MenuItem<'static>> {
    let mut items = Vec::with_capacity(4);
    if capabilities.editable {
        items.push(ITEMS[0].disabled(!capabilities.can_cut));
    }
    items.push(ITEMS[1].disabled(!capabilities.can_copy));
    if capabilities.editable {
        items.push(ITEMS[2].disabled(!capabilities.can_paste || !clipboard_has_text));
    }
    items.push(ITEMS[3].disabled(!capabilities.can_select_all));
    items
}

fn command_for(action: ActionKey) -> Option<TextEditCommand> {
    if action == CUT {
        Some(TextEditCommand::Cut)
    } else if action == COPY {
        Some(TextEditCommand::Copy)
    } else if action == PASTE {
        Some(TextEditCommand::Paste)
    } else if action == SELECT_ALL {
        Some(TextEditCommand::SelectAll)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use ratatui_core::layout::{Position, Rect};

    use super::*;
    use crate::event::{Input, Key, KeyCode, KeyModifiers, Mouse, MouseKind};
    use crate::measure::Size;
    use crate::response::Response;
    use crate::runtime::stub::deliver;
    use crate::runtime::{App, Runtime};
    use crate::theme::Theme;
    use crate::ui::Cx;
    use crate::ui::Ui;
    use termrock_fields::{
        BlurPolicy, NoValidate, TextArea, TextAreaState, TextInput, TextInputState,
    };

    const INPUT: Id = Id::root("edit-menu.input");
    const AREA: Id = Id::root("edit-menu.area");
    const BACKGROUND: Id = Id::root("edit-menu.background");

    struct InputApp {
        value: String,
        state: TextInputState,
        background_value: String,
        background_state: TextInputState,
        menu: EditMenu,
        menu_state: EditMenuState,
        commands: Vec<TextEditCommand>,
        closed: Vec<DismissReason>,
    }

    impl InputApp {
        fn new() -> Self {
            Self {
                value: "alpha beta".to_owned(),
                state: TextInputState::default(),
                background_value: "background".to_owned(),
                background_state: TextInputState::default(),
                menu: EditMenu::new(INPUT),
                menu_state: EditMenuState::default(),
                commands: Vec::new(),
                closed: Vec::new(),
            }
        }
    }

    impl App for InputApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let field = TextInput::new(INPUT).blur(if self.menu_state.is_open() {
                BlurPolicy::Keep
            } else {
                BlurPolicy::Commit
            });
            let response = field.update(cx, &mut self.state, &mut self.value);
            let background = TextInput::new(BACKGROUND);
            let background_response =
                background.update(cx, &mut self.background_state, &mut self.background_value);
            if let Some(request) = self.state.take_edit_menu_request() {
                let capabilities = field.edit_capabilities(&self.state, &self.value);
                self.menu
                    .open(cx, &mut self.menu_state, request, capabilities);
            }
            let capabilities = field.edit_capabilities(&self.state, &self.value);
            let menu_response = self.menu.update(cx, &mut self.menu_state, capabilities);
            match menu_response.action_ref() {
                Some(EditMenuAction::Command(command)) => {
                    self.commands.push(*command);
                    let _ =
                        field.execute_edit_command(cx, &mut self.state, &mut self.value, *command);
                }
                Some(EditMenuAction::Closed(reason)) => self.closed.push(*reason),
                None => {}
            }
            response.erase() | background_response.erase() | menu_response.erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            TextInput::new(INPUT)
                .value(&self.value)
                .draw(ui, Rect::new(0, 0, 24, 1), &self.state);
            TextInput::new(BACKGROUND)
                .value(&self.background_value)
                .draw(ui, Rect::new(40, 0, 24, 1), &self.background_state);
            self.menu.draw(ui, &self.menu_state);
        }

        fn min_size(&self) -> Size {
            Size {
                min: (48, 14),
                preferred: (48, 14),
            }
        }
    }

    struct AreaApp {
        value: String,
        state: TextAreaState,
        menu: EditMenu,
        menu_state: EditMenuState,
        commands: Vec<TextEditCommand>,
        seed_clipboard: bool,
    }

    impl App for AreaApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            if !self.seed_clipboard {
                cx.clipboard_copy("X");
                self.seed_clipboard = true;
            }
            let field = TextArea::new(AREA, 4).blur(BlurPolicy::Keep);
            let response = field.update(cx, &mut self.state, &mut self.value);
            if let Some(request) = self.state.take_edit_menu_request() {
                let capabilities = field.edit_capabilities(&self.state, &self.value);
                self.menu
                    .open(cx, &mut self.menu_state, request, capabilities);
            }
            let capabilities = field.edit_capabilities(&self.state, &self.value);
            let menu_response = self.menu.update(cx, &mut self.menu_state, capabilities);
            if let Some(EditMenuAction::Command(command)) = menu_response.action_ref() {
                self.commands.push(*command);
                if field.execute_edit_command(cx, &mut self.state, &mut self.value, *command) {
                    let _ = self.state.commit(&mut self.value, &NoValidate);
                }
            }
            response.erase() | menu_response.erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            TextArea::new(AREA, 4)
                .value(&self.value)
                .draw(ui, Rect::new(0, 0, 28, 5), &self.state);
            self.menu.draw(ui, &self.menu_state);
        }

        fn min_size(&self) -> Size {
            Size {
                min: (48, 14),
                preferred: (48, 14),
            }
        }
    }

    fn key(code: KeyCode, mods: KeyModifiers) -> Input {
        Input::Key(Key { code, mods })
    }

    fn mouse(kind: MouseKind, x: u16, y: u16) -> Input {
        Input::Mouse(Mouse {
            kind,
            pos: Position::new(x, y),
            mods: KeyModifiers::NONE,
        })
    }

    #[test]
    fn menu_rows_retain_readonly_secret_disabled_and_clipboard_availability() {
        let readonly = menu_items(
            EditCapabilities {
                editable: false,
                can_copy: true,
                can_select_all: true,
                ..EditCapabilities::default()
            },
            false,
        );
        assert_eq!(readonly.len(), 2);
        assert_eq!(readonly[0].label(), "Copy");
        assert!(!readonly[0].is_disabled());
        assert_eq!(readonly[1].label(), "Select all");
        assert!(!readonly[1].is_disabled());
        assert_eq!(
            readonly[1].chord_ref(),
            Some(Chord::with(KeyCode::Char('a'), KeyModifiers::CONTROL,))
        );

        let secret = menu_items(
            EditCapabilities {
                editable: true,
                can_select_all: true,
                ..EditCapabilities::default()
            },
            true,
        );
        assert!(secret[0].is_disabled());
        assert!(secret[1].is_disabled());
        assert!(secret[2].is_disabled());
        assert!(!secret[3].is_disabled());

        let disabled = menu_items(EditCapabilities::default(), true);
        assert_eq!(disabled.len(), 2);
        assert!(disabled.iter().all(MenuItem::is_disabled));
    }

    #[test]
    fn keyboard_menu_activation_returns_select_all_once_and_restores_input_focus() {
        let mut runtime = Runtime::new(InputApp::new(), Theme::junie());
        let _ = runtime.initialize();
        let _ = deliver(&mut runtime, mouse(MouseKind::Down, 4, 0));
        let _ = deliver(&mut runtime, mouse(MouseKind::Up, 4, 0));
        assert_eq!(runtime.focus(), Some(INPUT));
        let _ = deliver(&mut runtime, key(KeyCode::F(10), KeyModifiers::SHIFT));
        assert!(runtime.is_open(INPUT.sub("edit-menu")));
        assert_eq!(runtime.focus(), Some(INPUT.sub("edit-menu")));

        // Disabled commands are skipped and Enter activates Select All.
        let _ = deliver(&mut runtime, key(KeyCode::Enter, KeyModifiers::NONE));
        let _ = deliver(&mut runtime, Input::Tick);
        assert_eq!(runtime.app().commands, [TextEditCommand::SelectAll]);
        assert_eq!(runtime.app().state.selection(), Some(0..10));
        assert!(!runtime.is_open(INPUT.sub("edit-menu")));
        assert_eq!(runtime.focus(), Some(INPUT));
        assert!(
            runtime.cursor().is_some(),
            "the restored field should own its cursor"
        );
        let _ = deliver(&mut runtime, key(KeyCode::Enter, KeyModifiers::NONE));
        let _ = deliver(&mut runtime, Input::Tick);
        assert_eq!(runtime.app().commands, [TextEditCommand::SelectAll]);
    }

    #[test]
    fn textarea_secondary_menu_pastes_at_prepared_caret() {
        let mut runtime = Runtime::new(
            AreaApp {
                value: "one\ntwo".to_owned(),
                state: TextAreaState::default(),
                menu: EditMenu::new(AREA),
                menu_state: EditMenuState::default(),
                commands: Vec::new(),
                seed_clipboard: false,
            },
            Theme::junie(),
        );
        let _ = runtime.initialize();
        let _ = deliver(&mut runtime, mouse(MouseKind::Secondary, 3, 1));
        assert!(runtime.is_open(AREA.sub("edit-menu")));
        let _ = deliver(&mut runtime, key(KeyCode::Enter, KeyModifiers::NONE));
        let _ = deliver(&mut runtime, Input::Tick);
        assert_eq!(runtime.app().commands, [TextEditCommand::Paste]);
        assert_eq!(runtime.app().value, "one\ntXwo");
        assert_eq!(runtime.focus(), Some(AREA));
        assert!(
            runtime.cursor().is_some(),
            "the restored textarea should own its cursor"
        );
    }

    #[test]
    fn escape_and_outside_click_close_without_background_key_activation() {
        for dismiss in [DismissReason::Esc, DismissReason::OutsideClick] {
            let mut runtime = Runtime::new(InputApp::new(), Theme::junie());
            let _ = runtime.initialize();
            let _ = deliver(&mut runtime, mouse(MouseKind::Secondary, 4, 0));
            assert!(runtime.is_open(INPUT.sub("edit-menu")));
            let _ = deliver(&mut runtime, key(KeyCode::Char('z'), KeyModifiers::NONE));
            assert_eq!(runtime.app().value, "alpha beta");
            if dismiss == DismissReason::Esc {
                let _ = deliver(&mut runtime, key(KeyCode::Esc, KeyModifiers::NONE));
            } else {
                let _ = deliver(&mut runtime, mouse(MouseKind::Down, 44, 0));
                let _ = deliver(&mut runtime, mouse(MouseKind::Up, 44, 0));
            }
            let _ = deliver(&mut runtime, Input::Tick);
            assert!(!runtime.is_open(INPUT.sub("edit-menu")));
            assert_eq!(runtime.app().closed.last(), Some(&dismiss));
            assert_eq!(runtime.app().closed.len(), 1);
            assert!(runtime.app().commands.is_empty());
            assert_eq!(runtime.app().background_value, "background");
            assert_eq!(runtime.focus(), Some(INPUT));
            assert!(
                runtime.cursor().is_some(),
                "focus dismissal should restore the field cursor"
            );
            assert_eq!(runtime.cursor(), Some(Position::new(4, 0)));
        }
    }
}
