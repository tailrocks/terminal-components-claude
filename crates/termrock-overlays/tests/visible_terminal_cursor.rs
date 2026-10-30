use ratatui_core::layout::{Position, Rect};
use termrock_controls::Button;
use termrock_core::action::ActionKey;
use termrock_core::id::Id;
use termrock_core::response::Response;
use termrock_fields::{TextInput, TextInputState};
use termrock_overlays::{ContextMenu, MenuAction, MenuItem, MenuState};
use termrock_runtime::{App, Cx, Ui, UpdateCause};
use termrock_test_support::Harness;
use termrock_theme::Theme;
use termrock_viewport::{CellPos, TextViewport, ViewportLine, ViewportState};

const TERMINAL: Id = Id::root("visible-cursor.terminal");
const OPEN: Id = Id::root("visible-cursor.open");
const MENU: Id = Id::root("visible-cursor.menu");
const EDITOR: Id = Id::root("visible-cursor.editor");
const CHOOSE_FIRST: ActionKey = ActionKey::application("visible-cursor.first");
const CHOOSE_SECOND: ActionKey = ActionKey::application("visible-cursor.second");
const ITEMS: &[MenuItem<'static>] = &[
    MenuItem::new(CHOOSE_FIRST, "First action"),
    MenuItem::new(CHOOSE_SECOND, "Second action"),
];
const LINES: &[ViewportLine<'static>] = &[
    ViewportLine::Plain("log output"),
    ViewportLine::Plain("more output"),
    ViewportLine::Plain("$ A界e\u{301} tail"),
];

fn menu(retain_cursor: bool) -> ContextMenu<'static> {
    let menu = ContextMenu::at(MENU, ITEMS, Position::new(15, 1));
    if retain_cursor {
        menu.retain_cursor(TERMINAL)
    } else {
        menu
    }
}

struct Console {
    viewport: ViewportState,
    menu: MenuState,
    chosen: Option<ActionKey>,
    draw_terminal: bool,
    empty_terminal: bool,
    allow_unfocused_caret: bool,
    retain_cursor: bool,
    editor_enabled: bool,
    editor: TextInputState,
    editor_value: String,
}

impl Default for Console {
    fn default() -> Self {
        Console {
            viewport: ViewportState::default(),
            menu: MenuState::default(),
            chosen: None,
            draw_terminal: true,
            empty_terminal: false,
            allow_unfocused_caret: true,
            retain_cursor: true,
            editor_enabled: false,
            editor: TextInputState::default(),
            editor_value: String::new(),
        }
    }
}

impl Console {
    fn terminal(&self) -> TextViewport<'static> {
        TextViewport::new(TERMINAL).caret_when_unfocused(self.allow_unfocused_caret)
    }

    fn menu(&self) -> ContextMenu<'static> {
        menu(self.retain_cursor)
    }
}

impl App for Console {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = self
            .terminal()
            .update(cx, &mut self.viewport, LINES)
            .erase();
        response |= Button::new(OPEN, "Actions")
            .update(cx)
            .on_activated(|| cx.open_layer(MENU, self.menu().layer(cx)));
        response |= self
            .menu()
            .update(cx, &mut self.menu)
            .on_action(|action| {
                if let MenuAction::Chosen(key) = action {
                    self.chosen = Some(key);
                }
            })
            .erase();
        if self.editor_enabled && cx.is_open(MENU) {
            if cx.update_cause() == UpdateCause::Event {
                cx.focus(EDITOR);
            }
            response |= TextInput::new(EDITOR)
                .update(cx, &mut self.editor, &mut self.editor_value)
                .erase();
        }
        response
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let full = ui.full();
        let area = if self.empty_terminal {
            Rect::new(full.x, full.y, 0, full.height.saturating_sub(2))
        } else {
            Rect {
                height: full.height.saturating_sub(2),
                ..full
            }
        };
        if self.draw_terminal {
            self.terminal().draw(ui, area, &self.viewport, LINES);
        }
        Button::new(OPEN, "Actions").draw(
            ui,
            Rect::new(full.x, full.bottom().saturating_sub(1), 12, 1),
        );
        let menu = self.menu();
        ui.layer(MENU, |ui, area| {
            menu.draw(ui, area, &self.menu);
            if self.editor_enabled {
                TextInput::new(EDITOR).draw(ui, Rect::new(10, 2, 12, 1), &self.editor);
            }
        });
    }
}

fn console() -> Harness<Console> {
    let mut page = Console::default();
    page.viewport.set_caret(Some(CellPos::new(2, 7)));
    Harness::new(page, Theme::junie(), 40, 16)
}

#[test]
fn real_terminal_caret_survives_menu_focus_transfer() {
    let mut h = console();
    let expected = Position::new(8, 2);
    assert_eq!(h.runtime().focus(), Some(TERMINAL));
    assert_eq!(h.cursor(), Some(expected));

    let _ = h.click_id(OPEN);
    assert!(h.runtime().is_open(MENU));
    assert_eq!(h.runtime().focus(), Some(MENU));
    assert_eq!(h.cursor(), Some(expected));
    assert_eq!(h.app().viewport.scroll().offset(), 0);

    let _ = h.key(termrock_runtime::KeyCode::Down);
    assert_eq!(h.app().menu.cursor(), 1);
    assert_eq!(h.app().viewport.scroll().offset(), 0);
    let _ = h.key(termrock_runtime::KeyCode::Enter);
    assert_eq!(h.app().chosen, Some(CHOOSE_SECOND));
    assert!(!h.runtime().is_open(MENU));
    assert_eq!(h.runtime().focus(), Some(OPEN));
    assert_eq!(
        h.cursor(),
        None,
        "the closed menu cannot retain a stale caret"
    );

    let _ = h.click_id(OPEN);
    assert_eq!(h.runtime().focus(), Some(MENU));
    assert_eq!(h.cursor(), Some(expected));
    let _ = h.key(termrock_runtime::KeyCode::Esc);
    assert!(!h.runtime().is_open(MENU));
    assert_eq!(h.runtime().focus(), Some(OPEN));
    assert_eq!(h.cursor(), None);
}

#[test]
fn visible_caret_tracks_live_viewport_projection_and_suppression() {
    let mut h = console();
    let _ = h.click_id(OPEN);
    let _ = h.key(termrock_runtime::KeyCode::Down);
    assert_eq!(h.cursor(), Some(Position::new(8, 2)));

    h.app_mut().viewport.set_caret(Some(CellPos::new(2, 9)));
    h.draw();
    assert_eq!(h.cursor(), Some(Position::new(10, 2)));

    h.app_mut()
        .viewport
        .select(CellPos::new(2, 1), CellPos::new(2, 3));
    h.draw();
    assert_eq!(h.cursor(), None, "selection suppresses background caret");

    h.app_mut().viewport.clear_selection();
    h.app_mut().viewport.set_follow(false);
    h.draw();
    assert_eq!(
        h.cursor(),
        None,
        "manual scrolling suppresses background caret"
    );

    h.app_mut().viewport.set_follow(true);
    h.draw();
    assert_eq!(h.cursor(), Some(Position::new(10, 2)));

    let _ = h.resize(6, 16);
    assert_eq!(h.cursor(), None, "clipped cursor must not survive resize");
    let _ = h.resize(40, 16);
    assert_eq!(h.cursor(), Some(Position::new(10, 2)));

    h.app_mut().draw_terminal = false;
    h.draw();
    assert_eq!(h.cursor(), None, "unrendered owner cannot retain a cursor");
    assert_eq!(h.runtime().area_of(TERMINAL), None);

    h.app_mut().draw_terminal = true;
    h.app_mut().empty_terminal = true;
    h.draw();
    assert_eq!(h.cursor(), None, "zero-width owner cannot retain a cursor");
}

#[test]
fn cursor_retention_is_opt_in_and_does_not_change_the_frame() {
    let mut retained = console();
    let mut default_menu = console();
    default_menu.app_mut().retain_cursor = false;
    let mut focused_only = console();
    focused_only.app_mut().allow_unfocused_caret = false;

    for h in [&mut retained, &mut default_menu, &mut focused_only] {
        let _ = h.click_id(OPEN);
        assert_eq!(h.runtime().focus(), Some(MENU));
    }

    assert_eq!(retained.buffer(), default_menu.buffer());
    assert_eq!(retained.buffer(), focused_only.buffer());
    assert_eq!(retained.cursor(), Some(Position::new(8, 2)));
    assert_eq!(
        default_menu.cursor(),
        None,
        "ContextMenu default remains strict"
    );
    assert_eq!(
        focused_only.cursor(),
        None,
        "TextViewport default remains strict"
    );
    assert!(!retained.cursor_bar());
}

#[test]
fn editable_overlay_cursor_scope_precedes_a_retained_terminal_caret() {
    let mut h = console();
    let _ = h.click_id(OPEN);
    assert_eq!(h.cursor(), Some(Position::new(8, 2)));

    h.app_mut().editor_enabled = true;
    h.app_mut().editor.begin("");
    h.draw();
    let _ = h.key(termrock_runtime::KeyCode::Char('x'));
    assert_eq!(h.runtime().focus(), Some(EDITOR));
    assert_eq!(h.cursor(), Some(Position::new(12, 2)));
    assert!(h.cursor_bar());
}
