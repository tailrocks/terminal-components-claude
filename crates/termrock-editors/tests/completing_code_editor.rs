use std::cell::RefCell;

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};
use termrock_core::action::ActionKey;
use termrock_core::id::{Id, ItemKey, Part, PartRef};
use termrock_editors::code::{CodeAction, CodeEditor};
use termrock_editors::completion::{
    CompletionBatch, CompletionProvider, CompletionTrigger, CompletionValue, FOCUS_NEXT_ACTION,
    MANUAL_COMPLETION_ACTION,
};
use termrock_overlays::completion::AsItem;
use termrock_runtime::event::{Input, Key, KeyCode, KeyModifiers, Mouse, MouseKind};
use termrock_runtime::focus::Focusability;
use termrock_runtime::keymap::KeyMap;
use termrock_runtime::runtime::{App, Runtime};
use termrock_runtime::{Cx, Response, Ui};
use termrock_theme::Theme;

const EDITOR: Id = Id::root("completion.composite.editor");
const POPUP: Id = Id::root("completion.composite.popup");
const NEXT: Id = Id::root("completion.composite.next");
const AREA: Rect = Rect::new(0, 0, 100, 32);
const UNICODE_DETAIL: &str = "界 e\u{301} 👨‍👩‍👧‍👦 wide label 界 e\u{301} 👨‍👩‍👧‍👦 ending detail";

struct Provider {
    triggers: RefCell<Vec<CompletionTrigger>>,
    mode: ProviderMode,
}

#[derive(Clone, Copy)]
enum ProviderMode {
    Normal,
    Empty,
    Disabled,
    Noop,
    Details,
    UnicodeDetails,
}

impl CompletionProvider for Provider {
    fn complete(
        &self,
        text: &str,
        _cursor: usize,
        trigger: CompletionTrigger,
    ) -> Option<CompletionBatch> {
        self.triggers.borrow_mut().push(trigger);
        let token = text.split_whitespace().last().unwrap_or_default();
        if matches!(self.mode, ProviderMode::UnicodeDetails) {
            if token != "fe" {
                return None;
            }
            return Some(CompletionBatch {
                items: vec![
                    CompletionValue::new(ItemKey::num(20), "fetch", "fetch()")
                        .detail(UNICODE_DETAIL)
                        .menu_summary("fn"),
                ],
                replace_len: token.len(),
            });
        }
        if matches!(self.mode, ProviderMode::Details) {
            if token != "fe" {
                return None;
            }
            return Some(CompletionBatch {
                items: vec![
                    CompletionValue::new(ItemKey::num(10), "format", "format!()")
                        .detail("fn (value: &str) -> String")
                        .menu_summary("f"),
                    CompletionValue::new(ItemKey::num(11), "fetch(", "fetch()")
                        .detail(
                            "async fn (url: &str) -> Result<Body, Error>\nrequest headers\nresponse body stream\ncompletion footer",
                        )
                        .menu_summary("async fn (url: &str)"),
                ],
                replace_len: token.len(),
            });
        }
        if matches!(self.mode, ProviderMode::Empty) {
            return Some(CompletionBatch {
                items: Vec::new(),
                replace_len: token.len(),
            });
        }
        if matches!(self.mode, ProviderMode::Noop) && token == "co" {
            return Some(CompletionBatch {
                items: vec![CompletionValue::new(ItemKey::num(1), "co", "co")],
                replace_len: token.len(),
            });
        }
        if !matches!(self.mode, ProviderMode::Disabled | ProviderMode::Normal)
            || !matches!(token, "co" | "cou")
        {
            return None;
        }
        let first = CompletionValue::new(ItemKey::num(1), "count", "count")
            .disabled(matches!(self.mode, ProviderMode::Disabled));
        Some(CompletionBatch {
            items: vec![
                first,
                CompletionValue::new(ItemKey::num(2), "country", "country"),
            ],
            replace_len: token.len(),
        })
    }
}

struct EditorApp {
    state: termrock_editors::CompletingCodeEditorState,
    provider: Provider,
    keymap: KeyMap,
    actions: Vec<CodeAction>,
    read_only: bool,
    disabled: bool,
    commit_on_blur: bool,
    draw_width: u16,
    draw_x: u16,
    draw_y: u16,
    details_enabled: bool,
}

impl App for EditorApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let component = CodeEditor::new(EDITOR, 6)
            .read_only(self.read_only)
            .disabled(self.disabled)
            .with_completion(POPUP, &self.provider)
            .commit_on_blur(self.commit_on_blur);
        let component = if self.details_enabled {
            component.selected_details()
        } else {
            component
        };
        let response = component.update(cx, &mut self.state);
        if let Some(action) = response.action_ref() {
            self.actions.push(action.clone());
        }
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let component = CodeEditor::new(EDITOR, 6)
            .read_only(self.read_only)
            .disabled(self.disabled)
            .with_completion(POPUP, &self.provider)
            .commit_on_blur(self.commit_on_blur);
        let component = if self.details_enabled {
            component.selected_details()
        } else {
            component
        };
        component.draw(
            ui,
            Rect::new(self.draw_x, self.draw_y, self.draw_width, 8),
            &self.state,
        );
        ui.register_focus_only(NEXT, Focusability::Focusable);
    }

    fn keymap(&self) -> &KeyMap {
        &self.keymap
    }
}

fn runtime(text: &str) -> Runtime<EditorApp> {
    runtime_with(text, ProviderMode::Normal)
}

fn runtime_with(text: &str, mode: ProviderMode) -> Runtime<EditorApp> {
    let mut state = termrock_editors::CompletingCodeEditorState::new(text);
    state.editor_mut().jump_to(text.len());
    state.editor_mut().begin_edit();
    let mut runtime = Runtime::new(
        EditorApp {
            state,
            provider: Provider {
                triggers: RefCell::new(Vec::new()),
                mode,
            },
            keymap: KeyMap::new(),
            actions: Vec::new(),
            read_only: false,
            disabled: false,
            commit_on_blur: true,
            draw_width: 80,
            draw_x: 0,
            draw_y: 0,
            details_enabled: matches!(mode, ProviderMode::Details | ProviderMode::UnicodeDetails),
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

fn runtime_preserving_blur(text: &str) -> Runtime<EditorApp> {
    let mut runtime = runtime(text);
    runtime.app_mut().commit_on_blur = false;
    let mut buffer = Buffer::empty(AREA);
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
    runtime
}

fn key(code: KeyCode, mods: KeyModifiers) -> Input {
    Input::Key(Key { code, mods })
}

fn send(runtime: &mut Runtime<EditorApp>, code: KeyCode, mods: KeyModifiers) {
    let _ = termrock_runtime::stub::deliver(runtime, key(code, mods));
    let mut buffer = Buffer::empty(AREA);
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
}

fn visible_text(runtime: &mut Runtime<EditorApp>) -> String {
    let mut buffer = Buffer::empty(AREA);
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
    buffer_text(&buffer, AREA)
}

fn buffer_text(buffer: &Buffer, area: Rect) -> String {
    (area.y..area.bottom())
        .map(|y| {
            (area.x..area.right())
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn buffer_row_text(buffer: &Buffer, area: Rect, row: u16) -> String {
    use ratatui_core::buffer::CellWidth;

    let mut text = String::new();
    let mut x = area.x;
    while x < area.right() {
        let symbol = buffer[(x, row)].symbol();
        text.push_str(symbol);
        x = x.saturating_add(symbol.cell_width().max(1));
    }
    text.trim_end_matches(' ').to_owned()
}

fn send_into(
    runtime: &mut Runtime<EditorApp>,
    code: KeyCode,
    mods: KeyModifiers,
    area: Rect,
    buffer: &mut Buffer,
) {
    let _ = termrock_runtime::stub::deliver(runtime, key(code, mods));
    runtime.draw_buffer(area, buffer).commit_presented();
}

struct BareEditorApp {
    state: termrock_editors::code::CodeEditorState,
    actions: Vec<CodeAction>,
}

impl App for BareEditorApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let response = CodeEditor::new(EDITOR, 6).update(cx, &mut self.state);
        if let Some(action) = response.action_ref() {
            self.actions.push(action.clone());
        }
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        CodeEditor::new(EDITOR, 6).draw(ui, Rect::new(0, 0, 80, 8), &self.state);
    }
}

fn send_bare(runtime: &mut Runtime<BareEditorApp>, code: KeyCode, mods: KeyModifiers) {
    let _ = termrock_runtime::stub::deliver(runtime, key(code, mods));
    let mut buffer = Buffer::empty(AREA);
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
}

fn click(runtime: &mut Runtime<EditorApp>, x: u16, y: u16) {
    for kind in [MouseKind::Down, MouseKind::Up] {
        let _ = termrock_runtime::stub::deliver(
            runtime,
            Input::Mouse(Mouse {
                kind,
                pos: Position::new(x, y),
                mods: KeyModifiers::NONE,
            }),
        );
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_buffer(AREA, &mut buffer).commit_presented();
    }
}

#[test]
fn completion_opens_when_the_cursor_sits_one_column_past_a_narrow_field() {
    let prefix = "a".repeat(20);
    let mut runtime = runtime(&format!("{prefix} c"));
    runtime.app_mut().draw_width = 20;
    let mut buffer = Buffer::empty(AREA);
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();

    send(&mut runtime, KeyCode::Char('o'), KeyModifiers::NONE);
    assert!(
        runtime.app().state.is_completion_open(),
        "end-of-line caret one column past the field must still anchor completion"
    );
    assert_eq!(runtime.app().state.editor().text(), format!("{prefix} co"));

    send(&mut runtime, KeyCode::Tab, KeyModifiers::NONE);
    assert_eq!(
        runtime.app().state.editor().text(),
        format!("{prefix} count")
    );
    assert!(!runtime.app().state.is_completion_open());
}

#[test]
fn selected_details_follow_the_key_scroll_inside_the_same_popup_and_keep_acceptance() {
    let mut runtime = runtime_with("SELECT fe", ProviderMode::Details);
    send(&mut runtime, KeyCode::Char(' '), KeyModifiers::CONTROL);
    assert!(runtime.app().state.is_completion_open());
    assert!(runtime.app().state.selected_completion().is_none());

    let menu_area = runtime.layer_area(POPUP).expect("completion layer area");
    send(&mut runtime, KeyCode::Down, KeyModifiers::NONE);
    let selected = runtime
        .app()
        .state
        .selected_completion()
        .expect("selected completion value");
    assert_eq!(selected.as_item().key, ItemKey::num(11));
    assert_eq!(
        selected.full_detail().lines().next(),
        Some("async fn (url: &str) -> Result<Body, Error>")
    );
    assert_eq!(selected.as_item().detail, "async fn (url: &str)");
    assert_eq!(selected.as_item().insertion(), "fetch()");
    assert_eq!(menu_area.width, 34);
    let selected_row = runtime
        .area_of_part(POPUP, PartRef::item(Part::ROW, ItemKey::num(11)))
        .expect("selected row hit area");

    let cursor = runtime.app().state.editor().cursor_offset();
    send(
        &mut runtime,
        KeyCode::Char('d'),
        KeyModifiers::CONTROL | KeyModifiers::SHIFT,
    );
    assert!(runtime.app().state.is_completion_details_open());
    assert_eq!(runtime.focus(), Some(EDITOR));
    assert_eq!(runtime.layer_area(POPUP), Some(menu_area));
    assert_eq!(runtime.layer_area(POPUP).map(|area| area.width), Some(34));
    assert_eq!(runtime.app().state.editor().cursor_offset(), cursor);
    assert!(
        runtime.diagnostics().is_empty(),
        "{:?}",
        runtime.diagnostics()
    );

    let initial = visible_text(&mut runtime);
    assert!(initial.contains("async fn (url: &str)"), "{initial}");
    assert!(initial.contains("->"), "{initial}");
    assert!(initial.contains("Result<Body, Error>"), "{initial}");

    send(&mut runtime, KeyCode::Char('n'), KeyModifiers::CONTROL);
    send(&mut runtime, KeyCode::Char('p'), KeyModifiers::CONTROL);
    send(&mut runtime, KeyCode::PageDown, KeyModifiers::NONE);
    assert!(visible_text(&mut runtime).contains("response body stream"));
    send(&mut runtime, KeyCode::PageUp, KeyModifiers::NONE);
    let restored_signature = visible_text(&mut runtime);
    assert!(restored_signature.contains("async fn (url: &str)"));
    assert!(restored_signature.contains("Result<Body, Error>"));

    let mut reached_footer = false;
    for _ in 0..12 {
        let text = visible_text(&mut runtime);
        if text.contains("completion footer") {
            reached_footer = true;
            break;
        }
        send(&mut runtime, KeyCode::Down, KeyModifiers::NONE);
    }
    assert!(
        reached_footer,
        "full selected detail did not scroll into view"
    );
    assert_eq!(runtime.app().state.editor().cursor_offset(), cursor);
    assert_eq!(
        runtime
            .app()
            .state
            .selected_completion()
            .map(|item| item.as_item().key),
        Some(ItemKey::num(11))
    );
    assert!(
        runtime.diagnostics().is_empty(),
        "{:?}",
        runtime.diagnostics()
    );

    send(&mut runtime, KeyCode::Esc, KeyModifiers::NONE);
    assert!(!runtime.app().state.is_completion_details_open());
    assert!(runtime.app().state.is_completion_open());
    assert_eq!(runtime.layer_area(POPUP), Some(menu_area));
    assert_eq!(
        runtime.area_of_part(POPUP, PartRef::item(Part::ROW, ItemKey::num(11))),
        Some(selected_row)
    );
    send(&mut runtime, KeyCode::Tab, KeyModifiers::NONE);
    assert_eq!(runtime.app().state.editor().text(), "SELECT fetch()");
    assert!(!runtime.app().state.is_completion_open());
    assert!(runtime.app().state.selected_completion().is_none());
    assert!(
        runtime.diagnostics().is_empty(),
        "{:?}",
        runtime.diagnostics()
    );
}

#[test]
fn details_disabled_preserves_the_compact_popup_row_and_geometry() {
    let mut runtime = runtime_with("SELECT fe", ProviderMode::Details);
    runtime.app_mut().details_enabled = false;
    send(&mut runtime, KeyCode::Char(' '), KeyModifiers::CONTROL);
    send(&mut runtime, KeyCode::Down, KeyModifiers::NONE);

    let popup = runtime.layer_area(POPUP).expect("completion layer area");
    assert_eq!(popup.width, 34);
    assert_eq!(popup.height, 4);
    let row = runtime
        .area_of_part(POPUP, PartRef::item(Part::ROW, ItemKey::num(11)))
        .expect("fetch row hit area");
    assert_eq!(row.width, popup.width.saturating_sub(2));
    assert_eq!(row.height, 1);
    let mut buffer = Buffer::empty(AREA);
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
    let row_text = buffer_row_text(
        &buffer,
        Rect::new(row.x, row.y, row.width, row.height),
        row.y,
    );
    assert_eq!(row_text, "   fetch(  async fn (url: &str)");
    assert!(!row_text.contains("Result<Body, Error>"));

    let text_before = runtime.app().state.editor().text().to_owned();
    let cursor_before = runtime.app().state.editor().cursor_offset();
    send(
        &mut runtime,
        KeyCode::Char('d'),
        KeyModifiers::CONTROL | KeyModifiers::SHIFT,
    );
    assert!(!runtime.app().state.is_completion_details_open());
    assert!(runtime.app().state.is_completion_open());
    assert_eq!(runtime.layer_area(POPUP), Some(popup));
    assert_eq!(
        runtime.area_of_part(POPUP, PartRef::item(Part::ROW, ItemKey::num(11))),
        Some(row)
    );
    assert_eq!(runtime.app().state.editor().text(), text_before);
    assert_eq!(runtime.app().state.editor().cursor_offset(), cursor_before);
    assert!(
        runtime.diagnostics().is_empty(),
        "{:?}",
        runtime.diagnostics()
    );
}

#[test]
fn enter_accepts_from_details_and_escape_returns_to_list_before_dismissal() {
    let mut enter = runtime_with("SELECT fe", ProviderMode::Details);
    send(&mut enter, KeyCode::Char(' '), KeyModifiers::CONTROL);
    send(&mut enter, KeyCode::Down, KeyModifiers::NONE);
    send(
        &mut enter,
        KeyCode::Char('d'),
        KeyModifiers::CONTROL | KeyModifiers::SHIFT,
    );
    assert!(enter.app().state.is_completion_details_open());
    send(&mut enter, KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(enter.app().state.editor().text(), "SELECT fetch()");
    assert!(!enter.app().state.is_completion_open());
    assert!(enter.app().state.selected_completion().is_none());
    assert!(enter.diagnostics().is_empty(), "{:?}", enter.diagnostics());

    let mut escape = runtime_with("SELECT fe", ProviderMode::Details);
    send(&mut escape, KeyCode::Char(' '), KeyModifiers::CONTROL);
    send(&mut escape, KeyCode::Down, KeyModifiers::NONE);
    send(
        &mut escape,
        KeyCode::Char('d'),
        KeyModifiers::CONTROL | KeyModifiers::SHIFT,
    );
    assert!(escape.app().state.is_completion_details_open());
    send(&mut escape, KeyCode::Esc, KeyModifiers::NONE);
    assert!(!escape.app().state.is_completion_details_open());
    assert!(escape.app().state.is_completion_open());
    assert!(escape.app().state.editor().is_editing());
    send(&mut escape, KeyCode::Esc, KeyModifiers::NONE);
    assert!(!escape.app().state.is_completion_open());
    assert!(escape.app().state.editor().is_editing());
    send(&mut escape, KeyCode::Esc, KeyModifiers::NONE);
    assert!(!escape.app().state.editor().is_editing());
    assert!(
        escape.diagnostics().is_empty(),
        "{:?}",
        escape.diagnostics()
    );
}

#[test]
fn outside_click_dismisses_details_and_preserves_the_editing_session() {
    let mut runtime = runtime_with("SELECT fe", ProviderMode::Details);
    send(&mut runtime, KeyCode::Char(' '), KeyModifiers::CONTROL);
    send(&mut runtime, KeyCode::Down, KeyModifiers::NONE);
    send(
        &mut runtime,
        KeyCode::Char('d'),
        KeyModifiers::CONTROL | KeyModifiers::SHIFT,
    );
    assert!(runtime.app().state.is_completion_details_open());

    click(&mut runtime, 98, 30);

    assert!(!runtime.app().state.is_completion_details_open());
    assert!(!runtime.app().state.is_completion_open());
    assert!(runtime.app().state.editor().is_editing());
    assert!(runtime.app().state.selected_completion().is_none());
    assert!(
        runtime.diagnostics().is_empty(),
        "{:?}",
        runtime.diagnostics()
    );
}

#[test]
fn selected_details_clip_unicode_at_offset_right_and_bottom_edges() {
    use ratatui_core::buffer::CellWidth;

    let mut runtime = runtime_with("SELECT fe", ProviderMode::UnicodeDetails);
    runtime.app_mut().draw_x = 7;
    runtime.app_mut().draw_y = 4;
    let area = Rect::new(0, 0, 20, 8);
    let mut buffer = Buffer::empty(area);
    runtime.draw_buffer(area, &mut buffer).commit_presented();
    buffer[(19, 0)].set_symbol("§");

    send_into(
        &mut runtime,
        KeyCode::Char(' '),
        KeyModifiers::CONTROL,
        area,
        &mut buffer,
    );
    send_into(
        &mut runtime,
        KeyCode::Up,
        KeyModifiers::NONE,
        area,
        &mut buffer,
    );
    assert_eq!(
        runtime
            .app()
            .state
            .selected_completion()
            .map(|item| item.as_item().key),
        Some(ItemKey::num(20))
    );
    send_into(
        &mut runtime,
        KeyCode::Char('d'),
        KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        area,
        &mut buffer,
    );

    let popup = runtime.layer_area(POPUP).expect("completion popup");
    assert!(popup.right() <= area.right(), "{popup:?} within {area:?}");
    assert!(popup.bottom() <= area.bottom(), "{popup:?} within {area:?}");
    assert_eq!(popup.right(), area.right());
    assert!(popup.bottom() >= area.bottom().saturating_sub(1));
    assert_eq!(runtime.focus(), Some(EDITOR));
    assert!(runtime.app().state.is_completion_details_open());
    assert_eq!(buffer[(19, 0)].symbol(), "§");

    let content = Rect::new(
        popup.x.saturating_add(1),
        popup.y.saturating_add(1),
        popup.width.saturating_sub(2),
        popup.height.saturating_sub(2),
    );
    assert_eq!(
        content.height, 1,
        "this control keeps one detail row visible"
    );
    let expected_lines = [
        "界 e\u{301} 👨‍👩‍👧‍👦 wide label",
        "界 e\u{301} 👨‍👩‍👧‍👦 ending",
        "detail",
    ];
    assert_eq!(content.width, 18);
    assert_eq!(
        termrock_text::wrap(UNICODE_DETAIL, content.width),
        expected_lines
    );
    assert_eq!(expected_lines.join(" "), UNICODE_DETAIL);

    let y = popup.y.saturating_add(1);
    for x in popup.x.saturating_add(1)..popup.right().saturating_sub(1) {
        let symbol = buffer[(x, y)].symbol();
        let width = symbol.cell_width();
        if width > 1 {
            let end = x.saturating_add(width);
            assert!(end <= popup.right().saturating_sub(1));
            for continuation in x.saturating_add(1)..end {
                assert_eq!(buffer[(continuation, y)].symbol(), " ");
            }
        }
    }

    for (index, expected) in expected_lines.iter().enumerate() {
        assert_eq!(
            buffer_row_text(&buffer, content, content.y),
            *expected,
            "wrapped detail row {index} was not fully visible"
        );
        if index + 1 < expected_lines.len() {
            send_into(
                &mut runtime,
                KeyCode::Down,
                KeyModifiers::NONE,
                area,
                &mut buffer,
            );
        }
    }
    assert_eq!(buffer[(19, 0)].symbol(), "§");
    assert_eq!(runtime.layer_area(POPUP), Some(popup));
    assert!(
        runtime.diagnostics().is_empty(),
        "{:?}",
        runtime.diagnostics()
    );
}

#[test]
fn typing_opens_completion_tab_accepts_then_next_tab_indents_two_spaces() {
    let mut runtime = runtime("SELECT co");
    send(&mut runtime, KeyCode::Char('u'), KeyModifiers::NONE);
    assert!(runtime.app().state.is_completion_open());
    assert!(runtime.is_open(POPUP));
    assert_eq!(
        runtime.app().provider.triggers.borrow().as_slice(),
        &[CompletionTrigger::TextChanged]
    );
    assert_eq!(runtime.app().actions.last(), Some(&CodeAction::Changed));

    send(&mut runtime, KeyCode::Tab, KeyModifiers::NONE);
    assert_eq!(runtime.app().state.editor().text(), "SELECT count");
    assert!(!runtime.app().state.is_completion_open());
    assert_eq!(runtime.app().actions.last(), Some(&CodeAction::Changed));
    assert_eq!(
        runtime.app().provider.triggers.borrow().as_slice(),
        &[CompletionTrigger::TextChanged]
    );

    send(&mut runtime, KeyCode::Tab, KeyModifiers::NONE);
    assert_eq!(runtime.app().state.editor().text(), "SELECT count  ");
    assert!(!runtime.app().state.is_completion_open());
}

#[test]
fn escape_dismisses_first_then_commits_editor() {
    let mut runtime = runtime("SELECT co");
    send(&mut runtime, KeyCode::Char(' '), KeyModifiers::CONTROL);
    assert!(runtime.app().state.is_completion_open());

    send(&mut runtime, KeyCode::Esc, KeyModifiers::NONE);
    assert!(!runtime.app().state.is_completion_open());
    assert!(!runtime.is_open(POPUP));
    assert!(runtime.app().state.editor().is_editing());

    send(&mut runtime, KeyCode::Esc, KeyModifiers::NONE);
    assert!(!runtime.app().state.editor().is_editing());
    assert_eq!(runtime.app().actions.last(), Some(&CodeAction::Committed));
}

#[test]
fn remapped_manual_completion_uses_keymap_binding() {
    let mut runtime = runtime("SELECT co");
    runtime.app_mut().keymap.remap_component(
        EDITOR,
        MANUAL_COMPLETION_ACTION,
        termrock_runtime::Chord::key(KeyCode::F(4)),
    );
    send(&mut runtime, KeyCode::Char(' '), KeyModifiers::CONTROL);
    assert!(!runtime.app().state.is_completion_open());
    send(&mut runtime, KeyCode::F(4), KeyModifiers::NONE);
    assert!(runtime.app().state.is_completion_open());
    send(&mut runtime, KeyCode::F(4), KeyModifiers::NONE);
    assert!(runtime.app().state.is_completion_open());
    assert_eq!(
        runtime.app().provider.triggers.borrow().as_slice(),
        &[CompletionTrigger::Manual, CompletionTrigger::Manual]
    );
}

#[test]
fn default_manual_completion_remains_available_while_popup_is_open() {
    let mut runtime = runtime("SELECT co");
    send(&mut runtime, KeyCode::Char(' '), KeyModifiers::CONTROL);
    send(&mut runtime, KeyCode::Char(' '), KeyModifiers::CONTROL);
    assert!(runtime.app().state.is_completion_open());
    assert_eq!(
        runtime.app().provider.triggers.borrow().as_slice(),
        &[CompletionTrigger::Manual, CompletionTrigger::Manual]
    );
}

#[test]
fn manual_completion_is_ignored_while_find_is_typing() {
    let mut runtime = runtime("SELECT co");
    send(&mut runtime, KeyCode::Char('f'), KeyModifiers::CONTROL);
    assert!(runtime.app().state.editor().is_find_typing());
    send(&mut runtime, KeyCode::Char(' '), KeyModifiers::CONTROL);
    assert!(runtime.app().state.editor().is_find_typing());
    assert!(!runtime.app().state.is_completion_open());
    assert!(runtime.app().provider.triggers.borrow().is_empty());
}

#[test]
fn bare_code_editor_ctrl_space_does_not_trigger_completions() {
    let mut state = termrock_editors::code::CodeEditorState::new("SELECT co");
    state.begin_edit();
    let mut runtime = Runtime::new(
        BareEditorApp {
            state,
            actions: Vec::new(),
        },
        Theme::junie(),
    );
    let _ = runtime.initialize();
    let mut buffer = Buffer::empty(AREA);
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
    runtime.set_focus(Some(EDITOR));
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();

    send_bare(&mut runtime, KeyCode::Char(' '), KeyModifiers::CONTROL);

    assert_eq!(runtime.app().state.text(), "SELECT co");
    assert!(runtime.app().state.is_editing());
    assert!(!runtime.app().state.is_find_typing());
    assert!(runtime.app().actions.is_empty());
}

#[test]
fn draw_focus_and_ticks_do_not_request_completion() {
    let mut runtime = runtime("SELECT co");
    let mut buffer = Buffer::empty(AREA);
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
    let _ = termrock_runtime::stub::deliver(&mut runtime, Input::Tick);
    assert!(runtime.app().provider.triggers.borrow().is_empty());
    assert!(!runtime.app().state.is_completion_open());
}

#[test]
fn cursor_motion_refreshes_only_an_open_completion() {
    let mut runtime = runtime("SELECT co");
    send(&mut runtime, KeyCode::Left, KeyModifiers::NONE);
    assert!(runtime.app().provider.triggers.borrow().is_empty());
    send(&mut runtime, KeyCode::Right, KeyModifiers::NONE);
    assert!(runtime.app().provider.triggers.borrow().is_empty());
    send(&mut runtime, KeyCode::Char(' '), KeyModifiers::CONTROL);
    assert!(runtime.app().state.is_completion_open());
    assert!(runtime.is_open(POPUP));
    let before = runtime.app().state.editor().cursor_offset();
    send(&mut runtime, KeyCode::Left, KeyModifiers::NONE);
    assert_eq!(runtime.app().state.editor().cursor_offset(), before - 1);
    assert_eq!(
        runtime.app().provider.triggers.borrow().as_slice(),
        &[CompletionTrigger::Manual, CompletionTrigger::CursorMoved]
    );
    assert!(runtime.app().state.is_completion_open());
}

#[test]
fn remapped_focus_binding_works_while_completion_is_open() {
    let mut runtime = runtime("SELECT co");
    runtime.app_mut().keymap.remap_component(
        EDITOR,
        FOCUS_NEXT_ACTION,
        termrock_runtime::Chord::key(KeyCode::F(5)),
    );
    send(&mut runtime, KeyCode::Char(' '), KeyModifiers::CONTROL);
    assert!(runtime.app().state.is_completion_open());
    send(&mut runtime, KeyCode::F(6), KeyModifiers::NONE);
    assert!(runtime.app().state.is_completion_open());
    send(&mut runtime, KeyCode::F(5), KeyModifiers::NONE);
    assert!(!runtime.app().state.is_completion_open());
    assert_eq!(runtime.focus(), Some(NEXT));
    assert_eq!(
        runtime.app().actions.last(),
        Some(&CodeAction::Leave { backward: false })
    );
}

#[test]
fn find_shortcut_closes_completion_before_find_keys_are_used() {
    let mut tab = runtime("SELECT co");
    send(&mut tab, KeyCode::Char(' '), KeyModifiers::CONTROL);
    assert!(tab.app().state.is_completion_open());

    send(&mut tab, KeyCode::Char('f'), KeyModifiers::CONTROL);
    assert!(!tab.app().state.is_completion_open());
    assert!(!tab.is_open(POPUP));
    assert!(tab.app().state.editor().is_find_typing());

    send(&mut tab, KeyCode::Tab, KeyModifiers::NONE);
    assert_eq!(tab.app().state.editor().text(), "SELECT co");
    assert!(!tab.app().state.is_completion_open());
    assert!(tab.app().state.editor().is_find_typing());

    let mut enter = runtime("SELECT co");
    send(&mut enter, KeyCode::Char(' '), KeyModifiers::CONTROL);
    send(&mut enter, KeyCode::Char('f'), KeyModifiers::CONTROL);
    send(&mut enter, KeyCode::Enter, KeyModifiers::NONE);
    assert!(!enter.app().state.editor().is_find_typing());
    assert!(!enter.app().state.is_completion_open());

    let mut escape = runtime("SELECT co");
    send(&mut escape, KeyCode::Char(' '), KeyModifiers::CONTROL);
    send(&mut escape, KeyCode::Char('f'), KeyModifiers::CONTROL);
    send(&mut escape, KeyCode::Esc, KeyModifiers::NONE);
    assert!(!escape.app().state.editor().is_find_typing());
    assert!(!escape.app().state.is_completion_open());
}

#[test]
fn printable_remapped_command_matches_with_completion_open_or_closed() {
    let mut closed = runtime("SELECT co");
    closed.app_mut().keymap.remap_component(
        EDITOR,
        ActionKey::custom("Left"),
        termrock_runtime::Chord::key(KeyCode::Char('h')),
    );
    let before = closed.app().state.editor().cursor_offset();
    send(&mut closed, KeyCode::Char('h'), KeyModifiers::NONE);
    let closed_result = (
        closed.app().state.editor().text().to_owned(),
        closed.app().state.editor().cursor_offset(),
    );
    assert_eq!(closed_result.0, "SELECT co");
    assert_eq!(closed_result.1, before - 1);

    let mut open = runtime("SELECT co");
    open.app_mut().keymap.remap_component(
        EDITOR,
        ActionKey::custom("Left"),
        termrock_runtime::Chord::key(KeyCode::Char('h')),
    );
    send(&mut open, KeyCode::Char(' '), KeyModifiers::CONTROL);
    assert!(open.app().state.is_completion_open());
    let before = open.app().state.editor().cursor_offset();
    send(&mut open, KeyCode::Char('h'), KeyModifiers::NONE);
    assert_eq!(
        (
            open.app().state.editor().text().to_owned(),
            open.app().state.editor().cursor_offset()
        ),
        closed_result
    );
    assert_eq!(open.app().state.editor().cursor_offset(), before - 1);
}

#[test]
fn f6_commits_and_moves_focus_in_both_directions() {
    let mut runtime = runtime("SELECT co");
    send(&mut runtime, KeyCode::Char(' '), KeyModifiers::CONTROL);
    send(&mut runtime, KeyCode::F(6), KeyModifiers::NONE);
    assert!(!runtime.app().state.is_completion_open());
    assert!(!runtime.app().state.editor().is_editing());
    assert_eq!(runtime.focus(), Some(NEXT));
    assert_eq!(
        runtime.app().actions.last(),
        Some(&CodeAction::Leave { backward: false })
    );

    runtime.set_focus(Some(EDITOR));
    runtime.app_mut().state.editor_mut().begin_edit();
    let mut buffer = Buffer::empty(AREA);
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
    send(&mut runtime, KeyCode::Char(' '), KeyModifiers::CONTROL);
    send(&mut runtime, KeyCode::F(6), KeyModifiers::SHIFT);
    assert!(!runtime.app().state.is_completion_open());
    assert!(!runtime.app().state.editor().is_editing());
    assert_eq!(runtime.focus(), Some(NEXT));
    assert_eq!(
        runtime.app().actions.last(),
        Some(&CodeAction::Leave { backward: true })
    );
}

#[test]
fn readonly_or_disabled_editor_dismisses_without_provider_call() {
    for (read_only, disabled) in [(true, false), (false, true)] {
        let mut runtime = runtime("SELECT co");
        runtime.app_mut().read_only = read_only;
        runtime.app_mut().disabled = disabled;
        send(&mut runtime, KeyCode::Char(' '), KeyModifiers::CONTROL);
        assert!(!runtime.app().state.is_completion_open());
        assert!(runtime.app().provider.triggers.borrow().is_empty());
    }
}

#[test]
fn empty_and_noop_acceptance_close_without_reopening() {
    let mut empty = runtime_with("SELECT x", ProviderMode::Empty);
    send(&mut empty, KeyCode::Char(' '), KeyModifiers::CONTROL);
    assert!(!empty.app().state.is_completion_open());

    let mut noop = runtime_with("SELECT co", ProviderMode::Noop);
    send(&mut noop, KeyCode::Char(' '), KeyModifiers::CONTROL);
    assert!(noop.app().state.is_completion_open());
    send(&mut noop, KeyCode::Tab, KeyModifiers::NONE);
    assert_eq!(noop.app().state.editor().text(), "SELECT co");
    assert!(!noop.app().state.is_completion_open());
    assert_eq!(
        noop.app().provider.triggers.borrow().as_slice(),
        &[CompletionTrigger::Manual]
    );

    let debug = format!("{:?}", noop.app().state);
    assert!(!debug.contains("SELECT co"));
}

#[test]
fn completion_controller_closes_and_reopens_the_same_popup_owner() {
    let mut runtime = runtime("SELECT co");
    runtime.app_mut().state.editor_mut().commit();
    let mut buffer = Buffer::empty(AREA);
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
    assert!(!runtime.app().state.editor().is_editing());

    send(&mut runtime, KeyCode::Char(' '), KeyModifiers::CONTROL);
    assert!(runtime.app().state.editor().is_editing());
    assert!(runtime.app().state.is_completion_open());
    assert!(runtime.is_open(POPUP));

    send(&mut runtime, KeyCode::Esc, KeyModifiers::NONE);
    assert!(!runtime.app().state.is_completion_open());
    assert!(!runtime.is_open(POPUP));

    send(&mut runtime, KeyCode::Char(' '), KeyModifiers::CONTROL);
    assert!(runtime.app().state.is_completion_open());
    assert!(runtime.is_open(POPUP));
    assert_eq!(runtime.focus(), Some(EDITOR));
    assert!(
        runtime.diagnostics().is_empty(),
        "{:?}",
        runtime.diagnostics()
    );
}

#[test]
fn disabled_suggestion_cannot_be_accepted() {
    let mut runtime = runtime_with("SELECT co", ProviderMode::Disabled);
    send(&mut runtime, KeyCode::Char(' '), KeyModifiers::CONTROL);
    assert!(runtime.app().state.is_completion_open());
    send(&mut runtime, KeyCode::Tab, KeyModifiers::NONE);
    assert_eq!(runtime.app().state.editor().text(), "SELECT co");
    assert!(runtime.app().state.is_completion_open());
}

#[test]
fn outside_pointer_and_focus_loss_dismiss_and_commit() {
    let mut runtime = runtime("SELECT co");
    send(&mut runtime, KeyCode::Char(' '), KeyModifiers::CONTROL);
    assert!(runtime.app().state.is_completion_open());
    click(&mut runtime, 98, 30);
    assert!(!runtime.app().state.is_completion_open());
    assert!(!runtime.is_open(POPUP));
    assert!(runtime.app().state.editor().is_editing());

    send(&mut runtime, KeyCode::Char(' '), KeyModifiers::CONTROL);
    assert!(runtime.app().state.is_completion_open());
    runtime.set_focus(Some(NEXT));
    assert_eq!(runtime.focus(), Some(NEXT), "focus request before settling");
    let mut buffer = Buffer::empty(AREA);
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
    assert_eq!(
        runtime.focus(),
        Some(NEXT),
        "focus after settling the popup close"
    );
    let _ = termrock_runtime::stub::deliver(&mut runtime, Input::Tick);
    assert!(!runtime.app().state.is_completion_open());
    assert_eq!(runtime.app().actions.last(), Some(&CodeAction::Committed));
    assert_eq!(runtime.focus(), Some(NEXT));
    assert!(!runtime.app().state.editor().is_editing());
}

#[test]
fn focus_loss_dismisses_completion_but_preserves_configured_edit_session() {
    let mut runtime = runtime_preserving_blur("SELECT co");
    send(&mut runtime, KeyCode::Char(' '), KeyModifiers::CONTROL);
    assert!(runtime.app().state.is_completion_open());
    assert!(runtime.app().state.editor().is_editing());
    let text = runtime.app().state.editor().text().to_owned();
    let cursor = runtime.app().state.editor().cursor_offset();

    runtime.set_focus(Some(NEXT));
    let mut buffer = Buffer::empty(AREA);
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
    let _ = termrock_runtime::stub::deliver(&mut runtime, Input::Tick);
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();

    assert!(!runtime.app().state.is_completion_open());
    assert!(!runtime.is_open(POPUP));
    assert!(runtime.app().state.editor().is_editing());
    assert_eq!(runtime.app().state.editor().text(), text);
    assert_eq!(runtime.app().state.editor().cursor_offset(), cursor);
    assert_ne!(runtime.app().actions.last(), Some(&CodeAction::Committed));
    assert_eq!(runtime.focus(), Some(NEXT));
    assert!(
        runtime.diagnostics().is_empty(),
        "{:?}",
        runtime.diagnostics()
    );
}
