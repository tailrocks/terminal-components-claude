use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::{Color, Modifier, Style};
use termrock_controls::Button;
use termrock_core::id::{Id, Part, PartRef};
use termrock_editors::EditEntry;
use termrock_editors::code::{CodeDiagnostic, CodeEditor, CodeEditorState, CodeSeverity};
use termrock_editors::completion::{
    CompletingCodeEditor, CompletingCodeEditorState, CompletionBatch, CompletionTrigger,
};
use termrock_runtime::event::{Axis, Input, Key, KeyCode, KeyModifiers, Mouse, MouseKind};
use termrock_runtime::runtime::{App, Runtime};
use termrock_runtime::{Cx, Response, Ui};
use termrock_theme::{ColorLevel, Family, Role, StateFlags, StylePatch, Theme, Variant};

const EDITOR: Id = Id::root("code.idle_scroll_position.editor");
const ELSEWHERE: Id = Id::root("code.idle_scroll_position.elsewhere");
const POPUP: Id = Id::root("code.idle_scroll_position.completion");
const SCREEN: Rect = Rect::new(0, 0, 100, 40);
const AREA: Rect = Rect::new(8, 4, 58, 8);
const ELSEWHERE_AREA: Rect = Rect::new(0, 0, 12, 1);
const DOCUMENT: &str = "row 00\nrow 01\nrow 02\nrow 03\nrow 04\nrow 05\nrow 06\nrow 07\nrow 08\nrow 09\nrow 10\nrow 11\nrow 12\nrow 13\nrow 14\nrow 15\nrow 16\nrow 17\nrow 18\nrow 19\nrow 20\nrow 21\nrow 22\nrow 23\nrow 24\nrow 25\nrow 26\nrow 27\nrow 28\nrow 29\nrow 30\nrow 31";

const EMPTY_PATCH: StylePatch = StylePatch::new();
const EMPTY_META: [(Part, StylePatch); 1] = [(Part::META, StylePatch::new())];
const SET_META: [(Part, StylePatch); 1] = [(
    Part::META,
    StylePatch::new()
        .set_fg(Role::Custom(Color::Rgb(231, 23, 147)))
        .set_bg(Role::Custom(Color::Rgb(17, 29, 41)))
        .set_underline(Role::Custom(Color::Rgb(83, 109, 137)))
        .add(Modifier::ITALIC)
        .add(Modifier::UNDERLINED)
        .remove(Modifier::BOLD),
)];
const CLEAR_META: [(Part, StylePatch); 1] = [(Part::META, StylePatch::new().clear_fg().clear_bg())];
const RESET_META: [(Part, StylePatch); 1] = [(
    Part::META,
    StylePatch::new().set_fg(Role::Custom(Color::Reset)),
)];
const MODIFIER_META: [(Part, StylePatch); 1] = [(
    Part::META,
    StylePatch::new()
        .add(Modifier::ITALIC)
        .remove(Modifier::BOLD),
)];

#[derive(Clone, Copy)]
enum PositionOption {
    Default,
    Explicit(bool),
}

#[derive(Clone, Copy, Debug)]
enum MetaOverride {
    None,
    EmptyGlobal,
    EmptyPart,
    Set,
    Clear,
    Reset,
    Modifiers,
}

struct CodeApp {
    area: Rect,
    rows: u16,
    state: CodeEditorState,
    position: PositionOption,
    meta: MetaOverride,
    read_only: bool,
    disabled: bool,
}

impl CodeApp {
    fn editor(&self) -> CodeEditor<'static> {
        let editor = CodeEditor::new(EDITOR, self.rows)
            .read_only(self.read_only)
            .disabled(self.disabled)
            .edit_entry(EditEntry::Explicit);
        let editor = match self.position {
            PositionOption::Default => editor,
            PositionOption::Explicit(enabled) => editor.idle_scroll_position(enabled),
        };
        match self.meta {
            MetaOverride::None => editor,
            MetaOverride::EmptyGlobal => editor.patch(&EMPTY_PATCH),
            MetaOverride::EmptyPart => editor.patch_part(&EMPTY_META),
            MetaOverride::Set => editor.patch_part(&SET_META),
            MetaOverride::Clear => editor.patch_part(&CLEAR_META),
            MetaOverride::Reset => editor.patch_part(&RESET_META),
            MetaOverride::Modifiers => editor.patch_part(&MODIFIER_META),
        }
    }
}

impl App for CodeApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let _ = Button::new(ELSEWHERE, "Elsewhere").update(cx);
        self.editor().update(cx, &mut self.state).erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Button::new(ELSEWHERE, "Elsewhere").draw(ui, ELSEWHERE_AREA);
        self.editor().draw(ui, self.area, &self.state);
    }
}

fn app(text: &str, position: PositionOption) -> CodeApp {
    CodeApp {
        area: AREA,
        rows: AREA.height,
        state: CodeEditorState::new(text),
        position,
        meta: MetaOverride::None,
        read_only: false,
        disabled: false,
    }
}

fn sentinel_buffer() -> Buffer {
    let mut buffer = Buffer::empty(SCREEN);
    for y in SCREEN.y..SCREEN.bottom() {
        for x in SCREEN.x..SCREEN.right() {
            let cell = &mut buffer[(x, y)];
            cell.set_symbol("~");
            cell.set_style(
                Style::default()
                    .fg(Color::Rgb(7, 19, 31))
                    .bg(Color::Rgb(2, 4, 6)),
            );
        }
    }
    buffer
}

fn present<A: App>(runtime: &mut Runtime<A>) -> Buffer {
    let mut buffer = sentinel_buffer();
    runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
    buffer
}

fn runtime(text: &str, position: PositionOption, theme: Theme) -> Runtime<CodeApp> {
    let mut runtime = Runtime::new(app(text, position), theme);
    let _ = runtime.initialize();
    let _ = present(&mut runtime);
    let _ = termrock_runtime::runtime::stub::deliver(&mut runtime, Input::Tick);
    let _ = present(&mut runtime);
    runtime
}

fn key(runtime: &mut Runtime<CodeApp>, code: KeyCode, mods: KeyModifiers) -> Buffer {
    let _ = termrock_runtime::runtime::stub::deliver(runtime, Input::Key(Key { code, mods }));
    present(runtime)
}

fn wheel(runtime: &mut Runtime<CodeApp>, x: u16, y: u16, delta: i16) -> Buffer {
    let _ = termrock_runtime::runtime::stub::deliver(
        runtime,
        Input::Mouse(Mouse {
            kind: MouseKind::Wheel(Axis::V, delta),
            pos: Position::new(x, y),
            mods: KeyModifiers::NONE,
        }),
    );
    present(runtime)
}

fn row_text(buffer: &Buffer, area: Rect, y: u16) -> String {
    (area.x..area.right())
        .map(|x| buffer[(x, y)].symbol())
        .collect()
}

fn footer_text(buffer: &Buffer, area: Rect) -> String {
    if area.height == 0 {
        return String::new();
    }
    row_text(buffer, area, area.bottom() - 1)
}

fn footer_label(buffer: &Buffer, area: Rect) -> String {
    footer_text(buffer, area).trim().to_owned()
}

fn assert_outside_sentinel(buffer: &Buffer, area: Rect) {
    let sentinel = sentinel_buffer();
    for y in SCREEN.y..SCREEN.bottom() {
        for x in SCREEN.x..SCREEN.right() {
            let point = Position::new(x, y);
            let in_editor = area.contains(point);
            let in_peer = ELSEWHERE_AREA.contains(point);
            if in_editor || in_peer {
                continue;
            }
            assert_eq!(
                format!("{:?}", buffer[(x, y)]),
                format!("{:?}", sentinel[(x, y)]),
                "outside sentinel cell changed at {point:?}"
            );
        }
    }
}

fn buffer_cells(buffer: &Buffer) -> Vec<String> {
    (SCREEN.y..SCREEN.bottom())
        .flat_map(|y| (SCREEN.x..SCREEN.right()).map(move |x| format!("{:?}", buffer[(x, y)])))
        .collect()
}

fn assert_same_except_footer_label(enabled: &Buffer, disabled: &Buffer, area: Rect, label: &str) {
    let row = footer_text(enabled, area);
    let start = row
        .find(label)
        .unwrap_or_else(|| panic!("footer {row:?} does not contain {label:?}"));
    let first = area.x.saturating_add(row[..start].chars().count() as u16);
    let end = first.saturating_add(label.chars().count() as u16);
    for y in SCREEN.y..SCREEN.bottom() {
        for x in SCREEN.x..SCREEN.right() {
            if y == area.bottom().saturating_sub(1) && x >= first && x < end {
                continue;
            }
            assert_eq!(
                format!("{:?}", enabled[(x, y)]),
                format!("{:?}", disabled[(x, y)]),
                "idle option changed a non-label cell at ({x},{y})"
            );
        }
    }
    assert_eq!(footer_label(disabled, area), "");
}

fn text_area<A: App>(runtime: &Runtime<A>) -> Option<Rect> {
    runtime.area_of_part(EDITOR, PartRef::of(Part::TEXT))
}

fn label_cells(buffer: &Buffer, area: Rect, label: &str) -> Vec<ratatui_core::buffer::Cell> {
    let row = footer_text(buffer, area);
    let start = row
        .find(label)
        .unwrap_or_else(|| panic!("footer {row:?} does not contain {label:?}"));
    let start_x = area.x.saturating_add(row[..start].chars().count() as u16);
    (0..label.chars().count())
        .map(|offset| buffer[(start_x + offset as u16, area.bottom() - 1)].clone())
        .collect()
}

#[test]
fn default_and_explicit_false_preserve_full_overflow_interaction_frames() {
    assert!(
        format!("{:?}", CodeEditor::new(EDITOR, AREA.height))
            .contains("idle_scroll_position: false")
    );
    assert!(
        format!(
            "{:?}",
            CodeEditor::new(EDITOR, AREA.height).idle_scroll_position(true)
        )
        .contains("idle_scroll_position: true")
    );
    assert!(
        format!(
            "{:?}",
            CodeEditor::new(EDITOR, AREA.height)
                .patch_part(&SET_META)
                .idle_scroll_position(true)
        )
        .contains("idle_scroll_position: true")
    );
    let mut default = runtime(DOCUMENT, PositionOption::Default, Theme::junie());
    let mut explicit_false = runtime(DOCUMENT, PositionOption::Explicit(false), Theme::junie());

    let default_buffer = present(&mut default);
    let false_buffer = present(&mut explicit_false);
    assert_eq!(buffer_cells(&default_buffer), buffer_cells(&false_buffer));
    assert_eq!(footer_label(&default_buffer, AREA), "");
    assert_eq!(default.focus(), Some(ELSEWHERE));
    assert_eq!(explicit_false.focus(), Some(ELSEWHERE));
    assert_eq!(text_area(&default), text_area(&explicit_false));
    assert_eq!(
        default.area_of_part(EDITOR, PartRef::of(Part::TRACK)),
        explicit_false.area_of_part(EDITOR, PartRef::of(Part::TRACK))
    );
    assert_eq!(default.cursor(), explicit_false.cursor());
    assert_eq!(default.app().state, explicit_false.app().state);
    assert_outside_sentinel(&default_buffer, AREA);

    let default_wheel = wheel(&mut default, AREA.x + 5, AREA.y + 2, 1);
    let false_wheel = wheel(&mut explicit_false, AREA.x + 5, AREA.y + 2, 1);
    assert_eq!(buffer_cells(&default_wheel), buffer_cells(&false_wheel));
    assert_eq!(default.app().state, explicit_false.app().state);
    assert_eq!(
        default.app().state.scroll().offset(),
        3,
        "stock Junie scales one wheel detent by its default three-row motion token"
    );
    assert_eq!(footer_label(&default_wheel, AREA), "");

    default.set_focus(Some(EDITOR));
    explicit_false.set_focus(Some(EDITOR));
    let default_focus = present(&mut default);
    let false_focus = present(&mut explicit_false);
    assert_eq!(buffer_cells(&default_focus), buffer_cells(&false_focus));
    assert!(footer_label(&default_focus, AREA).starts_with("ln "));

    let default_edit = key(&mut default, KeyCode::Enter, KeyModifiers::NONE);
    let false_edit = key(&mut explicit_false, KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(buffer_cells(&default_edit), buffer_cells(&false_edit));
    assert_eq!(default.app().state, explicit_false.app().state);

    let default_find = key(&mut default, KeyCode::Char('f'), KeyModifiers::CONTROL);
    let false_find = key(
        &mut explicit_false,
        KeyCode::Char('f'),
        KeyModifiers::CONTROL,
    );
    assert_eq!(buffer_cells(&default_find), buffer_cells(&false_find));
    assert!(footer_text(&default_find, AREA).contains("find "));

    let diagnostic = CodeDiagnostic::new(0..4, CodeSeverity::Warning, "warning details");
    default
        .app_mut()
        .state
        .set_diagnostics(vec![diagnostic.clone()]);
    explicit_false
        .app_mut()
        .state
        .set_diagnostics(vec![diagnostic]);
    let default_diagnostic = present(&mut default);
    let false_diagnostic = present(&mut explicit_false);
    assert_eq!(
        buffer_cells(&default_diagnostic),
        buffer_cells(&false_diagnostic)
    );

    for text in [DOCUMENT, "alpha\nbeta"] {
        let mut off = runtime(text, PositionOption::Default, Theme::junie());
        let mut explicit_off = runtime(text, PositionOption::Explicit(false), Theme::junie());
        assert_eq!(
            buffer_cells(&present(&mut off)),
            buffer_cells(&present(&mut explicit_off)),
            "default and false must match for fit and overflow"
        );
    }

    for (read_only, disabled) in [(true, false), (false, true)] {
        let mut default_app = app(DOCUMENT, PositionOption::Default);
        default_app.read_only = read_only;
        default_app.disabled = disabled;
        let mut explicit_app = app(DOCUMENT, PositionOption::Explicit(false));
        explicit_app.read_only = read_only;
        explicit_app.disabled = disabled;
        let mut default_mode = Runtime::new(default_app, Theme::paper());
        let mut explicit_mode = Runtime::new(explicit_app, Theme::paper());
        let _ = default_mode.initialize();
        let _ = explicit_mode.initialize();
        assert_eq!(
            buffer_cells(&present(&mut default_mode)),
            buffer_cells(&present(&mut explicit_mode)),
            "default and false must match for read_only={read_only} disabled={disabled}"
        );
    }
}

#[test]
fn opt_in_uses_the_shared_range_for_live_wheel_offsets_without_taking_focus() {
    let mut one_row_wheel = Theme::junie();
    one_row_wheel.design.motion.wheel_rows = 1;
    let mut default_mode = runtime(DOCUMENT, PositionOption::Default, one_row_wheel.clone());
    let mut runtime = runtime(DOCUMENT, PositionOption::Explicit(true), one_row_wheel);
    let state_before = runtime.app().state.clone();
    let focus_before = runtime.focus();
    let cursor_before = runtime.cursor();
    let top = present(&mut runtime);
    assert_eq!(footer_label(&top, AREA), "1–7 of 32");
    assert_eq!(focus_before, Some(ELSEWHERE));
    assert_eq!(runtime.focus(), focus_before);
    assert_eq!(runtime.cursor(), cursor_before);
    assert_eq!(
        runtime.app().state,
        state_before,
        "drawing is observational"
    );
    assert!(!runtime.state_of(EDITOR).contains(StateFlags::FOCUSED));
    assert_eq!(
        text_area(&runtime).expect("overflowing document registers TEXT"),
        Rect::new(14, 4, 51, 7)
    );
    assert_eq!(
        runtime.area_of_part(EDITOR, PartRef::of(Part::TRACK)),
        Some(Rect::new(65, 4, 1, 7)),
        "overflowing document registers its complete scrollbar track"
    );
    assert_eq!(runtime.app().state.scroll().offset(), 0);
    assert_eq!(runtime.app().state.scroll().viewport_len(), 7);
    assert_outside_sentinel(&top, AREA);
    let default_top = present(&mut default_mode);
    assert_eq!(footer_label(&default_top, AREA), "");
    assert_same_except_footer_label(&top, &default_top, AREA, "1–7 of 32");
    assert_eq!(text_area(&default_mode), text_area(&runtime));
    assert_eq!(
        default_mode.area_of_part(EDITOR, PartRef::of(Part::TRACK)),
        runtime.area_of_part(EDITOR, PartRef::of(Part::TRACK))
    );

    let middle = wheel(&mut runtime, AREA.x + 5, AREA.y + 2, 1);
    let default_middle = wheel(&mut default_mode, AREA.x + 5, AREA.y + 2, 1);
    assert_eq!(runtime.app().state.scroll().offset(), 1);
    assert_eq!(default_mode.app().state.scroll().offset(), 1);
    assert_eq!(runtime.app().state, default_mode.app().state);
    assert_same_except_footer_label(&middle, &default_middle, AREA, "2–8 of 32");
    assert_outside_sentinel(&default_middle, AREA);
    assert_eq!(runtime.focus(), Some(ELSEWHERE));
    assert_eq!(
        runtime.app().state.cursor_offset(),
        state_before.cursor_offset()
    );
    assert_eq!(runtime.cursor(), cursor_before);
    let middle_label = footer_label(&middle, AREA);
    assert_eq!(middle_label, "2–8 of 32");
    assert_eq!(
        middle_label,
        termrock_runtime::scroll_region::position_label(runtime.app().state.scroll())
    );
    assert_eq!(
        text_area(&runtime).expect("middle view registers TEXT"),
        Rect::new(14, 4, 51, 7)
    );
    assert_eq!(
        runtime.area_of_part(EDITOR, PartRef::of(Part::TRACK)),
        Some(Rect::new(65, 4, 1, 7))
    );
    assert_outside_sentinel(&middle, AREA);

    let mut end = middle;
    for _ in 0..64 {
        end = wheel(&mut runtime, AREA.x + 5, AREA.y + 2, 12);
    }
    assert!(runtime.app().state.scroll().at_end());
    assert_eq!(runtime.app().state.scroll().offset(), 25);
    assert_eq!(footer_label(&end, AREA), "26–32 of 32");
    assert_eq!(runtime.focus(), Some(ELSEWHERE));
    assert_eq!(
        text_area(&runtime).expect("bottom view registers TEXT"),
        Rect::new(14, 4, 51, 7)
    );
    assert_eq!(
        runtime.area_of_part(EDITOR, PartRef::of(Part::TRACK)),
        Some(Rect::new(65, 4, 1, 7))
    );
    assert_outside_sentinel(&end, AREA);

    let fit_area = Rect::new(8, 4, 58, 8);
    let mut fit = Runtime::new(
        CodeApp {
            area: fit_area,
            rows: fit_area.height,
            state: CodeEditorState::new("alpha\nbeta"),
            position: PositionOption::Explicit(true),
            meta: MetaOverride::None,
            read_only: false,
            disabled: false,
        },
        Theme::junie(),
    );
    let _ = fit.initialize();
    let fit_buffer = present(&mut fit);
    assert_eq!(footer_label(&fit_buffer, fit_area), "");
    assert_outside_sentinel(&fit_buffer, fit_area);

    let mut exact_fit_app = app(
        "alpha\nbeta\ngamma\ndelta\nepsilon\nzeta\neta",
        PositionOption::Explicit(true),
    );
    exact_fit_app.rows = 8;
    let mut exact_fit = Runtime::new(exact_fit_app, Theme::junie());
    let _ = exact_fit.initialize();
    assert_eq!(footer_label(&present(&mut exact_fit), AREA), "");

    let mut empty = Runtime::new(app("", PositionOption::Explicit(true)), Theme::junie());
    let _ = empty.initialize();
    assert_eq!(footer_label(&present(&mut empty), AREA), "");
}

#[test]
fn focused_editing_find_read_only_and_disabled_paths_keep_their_own_priority() {
    let mut runtime = runtime(DOCUMENT, PositionOption::Explicit(true), Theme::junie());
    runtime.set_focus(Some(EDITOR));
    let focused = present(&mut runtime);
    assert!(footer_label(&focused, AREA).starts_with("ln 1/32 · col 1"));
    assert!(!footer_label(&focused, AREA).contains(" of 32"));

    let editing = key(&mut runtime, KeyCode::Enter, KeyModifiers::NONE);
    assert!(runtime.app().state.is_editing());
    let editing_label = footer_label(&editing, AREA);
    assert!(
        editing_label.starts_with("ln 1/32 · col 1"),
        "explicit Enter starts editing without inserting a newline: {editing_label:?}"
    );

    let find = key(&mut runtime, KeyCode::Char('f'), KeyModifiers::CONTROL);
    assert_eq!(
        runtime.app().state.find_info().map(|info| info.query),
        Some("")
    );
    assert!(footer_text(&find, AREA).contains("find "));
    assert!(!footer_text(&find, AREA).contains(" of 32"));

    let closed = key(&mut runtime, KeyCode::Esc, KeyModifiers::NONE);
    assert!(runtime.app().state.find_info().is_none());
    assert_eq!(footer_label(&closed, AREA), editing_label);

    runtime.set_focus(Some(ELSEWHERE));
    let _ = termrock_runtime::runtime::stub::deliver(&mut runtime, Input::Tick);
    assert_eq!(runtime.focus(), Some(ELSEWHERE));
    assert!(
        !runtime.app().state.is_editing(),
        "the pending FocusOut must commit editing before idle position is shown"
    );
    let idle = present(&mut runtime);
    assert_eq!(footer_label(&idle, AREA), "1–7 of 32");

    let mut read_only_app = app(DOCUMENT, PositionOption::Explicit(true));
    read_only_app.read_only = true;
    let mut read_only = Runtime::new(read_only_app, Theme::paper());
    let _ = read_only.initialize();
    let _ = present(&mut read_only);
    assert_eq!(read_only.focus(), Some(ELSEWHERE));
    let passive = present(&mut read_only);
    assert_eq!(footer_label(&passive, AREA), "1–7 of 32");
    read_only.set_focus(Some(EDITOR));
    let read_only_focused = present(&mut read_only);
    assert!(footer_label(&read_only_focused, AREA).starts_with("ln 1/32 · col 1"));
    assert!(
        read_only.cursor().is_some(),
        "focused read-only keeps its caret"
    );

    let mut disabled_app = app(DOCUMENT, PositionOption::Explicit(true));
    disabled_app.disabled = true;
    let mut disabled = Runtime::new(disabled_app, Theme::junie());
    let _ = disabled.initialize();
    let before = disabled.app().state.clone();
    let disabled_buffer = present(&mut disabled);
    assert_eq!(disabled.focus(), Some(ELSEWHERE));
    assert_eq!(footer_label(&disabled_buffer, AREA), "1–7 of 32");
    let _ = wheel(&mut disabled, AREA.x + 5, AREA.y + 2, 1);
    let _ = key(&mut disabled, KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(
        disabled.app().state,
        before,
        "disabled update remains ignored"
    );
    assert_ne!(disabled.focus(), Some(EDITOR));
}

#[test]
fn find_diagnostics_fit_and_tiny_allocations_stay_inside_the_assigned_frame() {
    let mut runtime = runtime(DOCUMENT, PositionOption::Explicit(true), Theme::junie());
    let before = present(&mut runtime);
    let before_text = text_area(&runtime);
    runtime
        .app_mut()
        .state
        .set_diagnostics(vec![CodeDiagnostic::new(
            0..5,
            CodeSeverity::Error,
            "syntax error near row zero",
        )]);
    let wide = present(&mut runtime);
    assert!(footer_text(&wide, AREA).contains("syntax error"));
    assert!(footer_label(&wide, AREA).contains(" of 32"));
    assert_eq!(
        text_area(&runtime),
        before_text,
        "diagnostic changed body geometry"
    );
    assert_outside_sentinel(&wide, AREA);
    assert_ne!(buffer_cells(&before), buffer_cells(&wide));

    let narrow_area = Rect::new(8, 4, 28, 8);
    let mut narrow_app = app(DOCUMENT, PositionOption::Explicit(true));
    narrow_app.area = narrow_area;
    let mut narrow = Runtime::new(narrow_app, Theme::junie());
    let _ = narrow.initialize();
    narrow
        .app_mut()
        .state
        .set_diagnostics(vec![CodeDiagnostic::new(
            0..5,
            CodeSeverity::Error,
            "syntax error near row zero",
        )]);
    let narrow_buffer = present(&mut narrow);
    let narrow_footer = footer_text(&narrow_buffer, narrow_area);
    assert!(narrow_footer.contains("syntax"), "{narrow_footer:?}");
    assert!(narrow_footer.contains("of 32"), "{narrow_footer:?}");
    assert_eq!(text_area(&narrow).expect("narrow TEXT part").width, 21);
    assert_outside_sentinel(&narrow_buffer, narrow_area);

    for (width, height) in [(0, 8), (1, 8), (2, 8), (12, 4), (58, 0), (58, 1), (58, 2)] {
        let area = Rect::new(8, 4, width, height);
        let mut tiny_app = app(DOCUMENT, PositionOption::Explicit(true));
        tiny_app.area = area;
        tiny_app.rows = height;
        let mut tiny = Runtime::new(tiny_app, Theme::junie());
        let _ = tiny.initialize();
        let buffer = present(&mut tiny);
        assert_outside_sentinel(&buffer, area);
        if width == 0 || height <= 1 {
            assert_eq!(footer_label(&buffer, area), "");
        }
        assert!(tiny.cursor().is_none_or(|cursor| {
            cursor.x >= area.x
                && cursor.x < area.right()
                && cursor.y >= area.y
                && cursor.y < area.bottom()
        }));
    }
}

#[test]
fn direct_resize_and_shrink_draw_use_current_local_layout_before_model_convergence() {
    let mut runtime = runtime(DOCUMENT, PositionOption::Explicit(true), Theme::junie());
    let text_before = runtime.app().state.text().to_owned();
    let initial = present(&mut runtime);
    assert_eq!(footer_label(&initial, AREA), "1–7 of 32");
    assert_eq!(runtime.app().state.scroll().content_len(), 32);
    assert_outside_sentinel(&initial, AREA);

    let mut old_bottom = initial;
    for _ in 0..64 {
        old_bottom = wheel(&mut runtime, AREA.x + 5, AREA.y + 2, 12);
    }
    assert_eq!(runtime.app().state.scroll().offset(), 25);
    assert_eq!(runtime.app().state.scroll().viewport_len(), 7);
    assert_eq!(runtime.app().state.scroll().content_len(), 32);
    assert_eq!(footer_label(&old_bottom, AREA), "26–32 of 32");
    assert_outside_sentinel(&old_bottom, AREA);

    let resized = Rect::new(8, 4, 44, 10);
    runtime.app_mut().area = resized;
    runtime.app_mut().rows = resized.height;
    let before_resize_draw = runtime.app().state.clone();
    let focus_before_resize_draw = runtime.focus();
    let cursor_before_resize_draw = runtime.cursor();
    let first_resized = present(&mut runtime);
    assert_eq!(footer_label(&first_resized, resized), "24–32 of 32");
    assert!(row_text(&first_resized, resized, resized.y).contains("row 23"));
    assert_eq!(
        text_area(&runtime).expect("resized current TEXT registration"),
        Rect::new(14, 4, 37, 9)
    );
    assert_eq!(
        runtime.area_of_part(EDITOR, PartRef::of(Part::TRACK)),
        Some(Rect::new(51, 4, 1, 9))
    );
    assert_eq!(runtime.app().state, before_resize_draw);
    assert_eq!(runtime.focus(), focus_before_resize_draw);
    assert_eq!(runtime.cursor(), cursor_before_resize_draw);
    assert_eq!(runtime.app().state.scroll().offset(), 25);
    assert_eq!(runtime.app().state.scroll().viewport_len(), 7);
    assert_eq!(runtime.app().state.scroll().content_len(), 32);
    assert_outside_sentinel(&first_resized, resized);

    let _ = termrock_runtime::runtime::stub::deliver(&mut runtime, Input::Tick);
    let converged_resize = present(&mut runtime);
    assert_eq!(runtime.app().state.scroll().offset(), 23);
    assert_eq!(runtime.app().state.scroll().viewport_len(), 9);
    assert_eq!(runtime.app().state.scroll().content_len(), 32);
    assert_eq!(footer_label(&converged_resize, resized), "24–32 of 32");
    assert_outside_sentinel(&converged_resize, resized);

    let same_line_count = (0..32)
        .map(|line| format!("replacement {line:02}界"))
        .collect::<Vec<_>>()
        .join("\n");
    runtime.app_mut().state.set_text(&same_line_count);
    assert_eq!(runtime.app().state.scroll().offset(), 0);
    assert_eq!(runtime.app().state.scroll().content_len(), 32);
    let first_replacement = present(&mut runtime);
    assert!(row_text(&first_replacement, resized, resized.y).contains("replacement"));
    assert_eq!(footer_label(&first_replacement, resized), "1–9 of 32");
    assert_eq!(
        text_area(&runtime)
            .expect("current TEXT registration")
            .height,
        9
    );
    assert_eq!(runtime.app().state.scroll().content_len(), 32);
    assert_ne!(runtime.app().state.text(), text_before);
    assert_outside_sentinel(&first_replacement, resized);

    let mut replacement_bottom = first_replacement;
    for _ in 0..64 {
        replacement_bottom = wheel(&mut runtime, resized.x + 5, resized.y + 2, 12);
    }
    assert_eq!(runtime.app().state.scroll().offset(), 23);
    assert_eq!(footer_label(&replacement_bottom, resized), "24–32 of 32");
    assert_outside_sentinel(&replacement_bottom, resized);

    let shorter = (0..12)
        .map(|line| format!("short {line:02}"))
        .collect::<Vec<_>>()
        .join("\n");
    runtime.app_mut().state.set_text(&shorter);
    assert_eq!(runtime.app().state.scroll().offset(), 0);
    assert_eq!(runtime.app().state.scroll().viewport_len(), 9);
    assert_eq!(
        runtime.app().state.scroll().content_len(),
        32,
        "set_text resets the offset while old layout facts remain pending"
    );
    let first_shrink = present(&mut runtime);
    assert!(row_text(&first_shrink, resized, resized.y).contains("short 00"));
    assert_eq!(footer_label(&first_shrink, resized), "1–9 of 12");
    assert_eq!(runtime.app().state.scroll().content_len(), 32);
    assert_eq!(
        text_area(&runtime).expect("first shrink frame uses current TEXT registration"),
        Rect::new(14, 4, 37, 9)
    );
    assert_outside_sentinel(&first_shrink, resized);

    let _ = termrock_runtime::runtime::stub::deliver(&mut runtime, Input::Tick);
    let converged = present(&mut runtime);
    assert_eq!(runtime.app().state.scroll().content_len(), 12);
    assert_eq!(footer_label(&converged, resized), "1–9 of 12");
    assert_outside_sentinel(&converged, resized);
}

#[test]
fn meta_option_and_authored_channels_survive_idle_label_paint_at_all_levels() {
    for (theme_name, theme) in [("Junie", Theme::junie()), ("Paper", Theme::paper())] {
        for level in [
            ColorLevel::TrueColor,
            ColorLevel::Ansi256,
            ColorLevel::Ansi16,
            ColorLevel::Mono,
        ] {
            let base = theme.downgrade(level);
            let with_bold = base.clone().override_family(Family::CODE, |recipe| {
                recipe
                    .part(Part::META)
                    .base(StylePatch::new().add(Modifier::BOLD));
            });
            for patch_case in [
                MetaOverride::Set,
                MetaOverride::Clear,
                MetaOverride::Reset,
                MetaOverride::Modifiers,
            ] {
                let mut enabled_app = app(DOCUMENT, PositionOption::Explicit(true));
                enabled_app.meta = patch_case;
                let mut runtime = Runtime::new(enabled_app, with_bold.clone());
                let _ = runtime.initialize();
                let buffer = present(&mut runtime);
                let mut disabled_app = app(DOCUMENT, PositionOption::Explicit(false));
                disabled_app.meta = patch_case;
                let mut disabled = Runtime::new(disabled_app, with_bold.clone());
                let _ = disabled.initialize();
                let disabled_buffer = present(&mut disabled);
                let label = footer_label(&buffer, AREA);
                assert_eq!(label, "1–7 of 32", "{theme_name} {level:?} {patch_case:?}");
                assert_same_except_footer_label(&buffer, &disabled_buffer, AREA, "1–7 of 32");
                let cells = label_cells(&buffer, AREA, "1–7 of 32");
                match patch_case {
                    MetaOverride::Set => {
                        for cell in &cells {
                            let expected_theme =
                                with_bold.clone().override_family(Family::CODE, |recipe| {
                                    recipe.part(Part::META).base(SET_META[0].1);
                                });
                            let expected = expected_theme
                                .resolve(
                                    Family::CODE,
                                    Variant::DEFAULT,
                                    Part::META,
                                    runtime.state_of(EDITOR),
                                    termrock_theme::Surface::Canvas,
                                )
                                .style;
                            assert_eq!(
                                cell.fg,
                                expected.fg.unwrap_or(Color::Reset),
                                "{theme_name} {level:?}"
                            );
                            assert_eq!(
                                cell.bg,
                                expected.bg.unwrap_or(Color::Reset),
                                "{theme_name} {level:?}"
                            );
                            assert_eq!(
                                cell.underline_color,
                                expected.underline_color.unwrap_or(Color::Reset),
                                "{theme_name} {level:?}"
                            );
                            assert!(cell.modifier.contains(Modifier::ITALIC));
                            assert!(cell.modifier.contains(Modifier::UNDERLINED));
                            assert!(!cell.modifier.contains(Modifier::BOLD));
                        }
                    }
                    MetaOverride::Clear => {
                        for cell in &cells {
                            assert_eq!(
                                cell.fg,
                                Color::Rgb(7, 19, 31),
                                "clear fg inherits the underlay"
                            );
                            assert_eq!(
                                cell.bg,
                                with_bold.bg(termrock_theme::Surface::Canvas),
                                "clear bg inherits the component Canvas fill"
                            );
                            assert!(cell.modifier.contains(Modifier::BOLD));
                        }
                    }
                    MetaOverride::Reset => {
                        for cell in &cells {
                            // `paint_str` treats the resolver's no-color Reset as no authored ink,
                            // leaving the previously painted underlay in these exact cells.
                            assert_eq!(
                                cell.fg,
                                Color::Rgb(7, 19, 31),
                                "raw Reset leaves existing buffer ink in this paint path"
                            );
                            assert_eq!(
                                cell.bg,
                                with_bold.bg(termrock_theme::Surface::Canvas),
                                "raw Reset leaves the component Canvas fill"
                            );
                            assert!(cell.modifier.contains(Modifier::BOLD));
                        }
                    }
                    MetaOverride::Modifiers => {
                        for cell in &cells {
                            assert!(cell.modifier.contains(Modifier::ITALIC));
                            assert!(!cell.modifier.contains(Modifier::BOLD));
                        }
                    }
                    MetaOverride::None | MetaOverride::EmptyGlobal | MetaOverride::EmptyPart => {
                        unreachable!("the style loop contains only explicit patch cases")
                    }
                }
            }

            let mut no_patch =
                Runtime::new(app(DOCUMENT, PositionOption::Explicit(true)), base.clone());
            let mut empty_global_app = app(DOCUMENT, PositionOption::Explicit(true));
            empty_global_app.meta = MetaOverride::EmptyGlobal;
            let mut empty_global = Runtime::new(empty_global_app, base.clone());
            let mut empty_part_app = app(DOCUMENT, PositionOption::Explicit(true));
            empty_part_app.meta = MetaOverride::EmptyPart;
            let mut empty_part = Runtime::new(empty_part_app, base);
            for runtime in [&mut no_patch, &mut empty_global, &mut empty_part] {
                let _ = runtime.initialize();
            }
            let baseline = present(&mut no_patch);
            let global = present(&mut empty_global);
            let part = present(&mut empty_part);
            assert_eq!(
                buffer_cells(&baseline),
                buffer_cells(&global),
                "{theme_name} {level:?} empty global"
            );
            assert_eq!(
                buffer_cells(&baseline),
                buffer_cells(&part),
                "{theme_name} {level:?} empty part"
            );
        }
    }
}

fn no_completion(_: &str, _: usize, _: CompletionTrigger) -> Option<CompletionBatch> {
    None
}

struct CompletingApp {
    state: CompletingCodeEditorState,
}

impl App for CompletingApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let _ = Button::new(ELSEWHERE, "Elsewhere").update(cx);
        CodeEditor::new(EDITOR, AREA.height)
            .idle_scroll_position(true)
            .edit_entry(EditEntry::Explicit)
            .with_completion(POPUP, &no_completion)
            .update(cx, &mut self.state)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Button::new(ELSEWHERE, "Elsewhere").draw(ui, ELSEWHERE_AREA);
        let editor: CompletingCodeEditor<'_> = CodeEditor::new(EDITOR, AREA.height)
            .idle_scroll_position(true)
            .edit_entry(EditEntry::Explicit)
            .with_completion(POPUP, &no_completion);
        editor.draw(ui, AREA, &self.state);
    }
}

#[test]
fn completion_wrapper_keeps_the_inner_idle_position_option() {
    let mut runtime = Runtime::new(
        CompletingApp {
            state: CompletingCodeEditorState::new(DOCUMENT),
        },
        Theme::junie(),
    );
    let _ = runtime.initialize();
    let _ = termrock_runtime::runtime::stub::deliver(&mut runtime, Input::Tick);
    let buffer = present(&mut runtime);
    assert_eq!(footer_label(&buffer, AREA), "1–7 of 32");
    assert!(runtime.app().state.editor().text().starts_with("row 00"));
}
