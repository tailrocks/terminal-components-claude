use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::{Color, Modifier};
use termrock_controls::Button;
use termrock_core::id::{Id, Part, PartRef};
use termrock_editors::{CodeEditor, CodeEditorState, EditEntry};
use termrock_runtime::event::{Input, Key, KeyCode, KeyModifiers};
use termrock_runtime::runtime::{App, Runtime};
use termrock_runtime::{Cx, Response, Ui};
use termrock_theme::{ColorLevel, Family, Role, StateFlags, StylePatch, Surface, Theme, Variant};

const EDITOR: Id = Id::root("diagnostic.code.footer");
const ELSEWHERE: Id = Id::root("diagnostic.code.footer.elsewhere");
const SCREEN: Rect = Rect::new(0, 0, 96, 32);

struct CodeApp {
    area: Rect,
    rows: u16,
    state: CodeEditorState,
    read_only: bool,
    disabled: bool,
    edit_entry: EditEntry,
}

impl CodeApp {
    fn editor(&self) -> CodeEditor<'static> {
        CodeEditor::new(EDITOR, self.rows)
            .read_only(self.read_only)
            .disabled(self.disabled)
            .edit_entry(self.edit_entry)
    }
}

impl App for CodeApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let _ = Button::new(ELSEWHERE, "Elsewhere").update(cx);
        self.editor().update(cx, &mut self.state).erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Button::new(ELSEWHERE, "Elsewhere").draw(ui, Rect::new(0, 0, 12, 1));
        self.editor().draw(ui, self.area, &self.state);
    }
}

#[derive(Debug)]
struct CellSnapshot {
    x: u16,
    y: u16,
    symbol: String,
    foreground: Color,
    background: Color,
    underline: Color,
    modifiers: Modifier,
    diff_option: String,
}

#[derive(Debug)]
struct FrameSnapshot {
    component: Option<Rect>,
    focus_region: Option<Rect>,
    text: Option<Rect>,
    focus: Option<Id>,
    state_flags: u32,
    cursor: Option<Position>,
    text_value: String,
    editing: bool,
    cursor_offset: usize,
    cursor_line: usize,
    cursor_column: usize,
    scroll_offset: usize,
    scroll_viewport: usize,
    find: Option<(String, Option<usize>, usize, bool)>,
    cells: Vec<CellSnapshot>,
}

fn app(text: &str, area: Rect, rows: u16) -> CodeApp {
    CodeApp {
        area,
        rows,
        state: CodeEditorState::new(text),
        read_only: false,
        disabled: false,
        edit_entry: EditEntry::Explicit,
    }
}

fn present<A: App>(runtime: &mut Runtime<A>) -> Buffer {
    let mut buffer = Buffer::empty(SCREEN);
    runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
    buffer
}

fn runtime(text: &str, area: Rect, rows: u16, theme: Theme) -> Runtime<CodeApp> {
    let mut runtime = Runtime::new(app(text, area, rows), theme);
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

fn capture(runtime: &Runtime<CodeApp>, buffer: &Buffer) -> FrameSnapshot {
    let component = Some(runtime.app().area);
    let focus_region = runtime.area_of(EDITOR);
    let text = runtime.area_of_part(EDITOR, PartRef::of(Part::TEXT));
    let model = &runtime.app().state;
    let find = model.find_info().map(|find| {
        (
            find.query.to_owned(),
            find.current,
            find.matches,
            find.typing,
        )
    });
    let cells = component
        .into_iter()
        .flat_map(|area| {
            (area.y..area.bottom()).flat_map(move |y| {
                (area.x..area.right()).filter_map(move |x| {
                    buffer.cell(Position::new(x, y)).map(|cell| CellSnapshot {
                        x,
                        y,
                        symbol: cell.symbol().to_owned(),
                        foreground: cell.fg,
                        background: cell.bg,
                        underline: cell.underline_color,
                        modifiers: cell.modifier,
                        diff_option: format!("{:?}", cell.diff_option),
                    })
                })
            })
        })
        .collect();
    let cursor_pos = model.cursor_pos();
    FrameSnapshot {
        component,
        focus_region,
        text,
        focus: runtime.focus(),
        state_flags: runtime.state_of(EDITOR).bits(),
        cursor: runtime.cursor(),
        text_value: model.text().to_owned(),
        editing: model.is_editing(),
        cursor_offset: model.cursor_offset(),
        cursor_line: cursor_pos.line,
        cursor_column: cursor_pos.col,
        scroll_offset: model.scroll().offset(),
        scroll_viewport: model.scroll().viewport_len(),
        find,
        cells,
    }
}

fn row_text(buffer: &Buffer, area: Rect, y: u16) -> String {
    (area.x..area.right())
        .map(|x| {
            buffer
                .cell(Position::new(x, y))
                .map_or(" ", |cell| cell.symbol())
        })
        .collect()
}

fn footer_text(buffer: &Buffer, area: Rect) -> String {
    row_text(buffer, area, area.bottom().saturating_sub(1))
}

fn assert_footer_label(buffer: &Buffer, area: Rect, label: &str) -> u16 {
    let row = footer_text(buffer, area);
    let start = row
        .find(label)
        .unwrap_or_else(|| panic!("expected footer {label:?} in {row:?}"));
    area.x.saturating_add(row[..start].chars().count() as u16)
}

#[expect(
    clippy::print_stdout,
    reason = "retain full-cell, cursor, hit-region, and model traces for this diagnostic suite"
)]
fn trace(name: &str, frame: &FrameSnapshot) {
    let cells = frame
        .cells
        .iter()
        .map(|cell| {
            format!(
                "{}:{}={:?}/{:?}/{:?}/{:?}/{:?}/{}",
                cell.x,
                cell.y,
                cell.symbol,
                cell.foreground,
                cell.background,
                cell.underline,
                cell.modifiers,
                cell.diff_option
            )
        })
        .collect::<Vec<_>>()
        .join(" ");
    println!(
        "CODE FOOTER TRACE {name}: area={:?} focus_region={:?} text={:?} focus={:?} flags={:#x} cursor={:?} model={{text:{:?}, editing:{}, offset:{}, position:({},{}), scroll:{}/{}, find:{:?}}} cells=[{}]",
        frame.component,
        frame.focus_region,
        frame.text,
        frame.focus,
        frame.state_flags,
        frame.cursor,
        frame.text_value,
        frame.editing,
        frame.cursor_offset,
        frame.cursor_line,
        frame.cursor_column,
        frame.scroll_offset,
        frame.scroll_viewport,
        frame.find,
        cells
    );
}

#[test]
fn current_footer_tracks_real_idle_focus_edit_and_find_states() {
    let area = Rect::new(7, 4, 42, 7);
    let document = "alpha\nbeta\ngamma\ndelta";
    let mut runtime = runtime(document, area, area.height, Theme::junie());

    let idle_buffer = present(&mut runtime);
    let idle = capture(&runtime, &idle_buffer);
    trace("idle-fit-unfocused", &idle);
    assert_eq!(idle.focus, Some(ELSEWHERE));
    assert_eq!(idle.text_value, document);
    assert_eq!(idle.component, Some(area));
    assert_eq!(idle.state_flags & StateFlags::FOCUSED.bits(), 0);
    assert!(idle.text.is_some(), "the real TEXT part is registered");
    assert!(footer_text(&idle_buffer, area).trim().is_empty());

    runtime.set_focus(Some(EDITOR));
    let focused_buffer = present(&mut runtime);
    let focused = capture(&runtime, &focused_buffer);
    trace("focused-browse", &focused);
    assert_eq!(focused.focus, Some(EDITOR));
    assert_eq!(
        focused.state_flags & StateFlags::FOCUSED.bits(),
        StateFlags::FOCUSED.bits()
    );
    assert!(
        !focused.editing,
        "Explicit entry keeps focused browse non-editing"
    );
    assert_eq!(
        focused.cursor, None,
        "ordinary focused browse hides the caret"
    );
    assert_eq!(
        runtime.app().state.cursor_pos().line,
        0,
        "the label starts from the actual CodeEditorState cursor"
    );
    assert_footer_label(&focused_buffer, area, "ln 1/4 · col 1");

    let editing_buffer = key(&mut runtime, KeyCode::Enter, KeyModifiers::NONE);
    let editing = capture(&runtime, &editing_buffer);
    trace("editing", &editing);
    assert!(editing.editing, "Enter uses the actual navigation binding");
    assert!(
        editing.cursor.is_some(),
        "editing publishes a terminal cursor"
    );
    assert_eq!(editing.text_value, document);
    assert_footer_label(&editing_buffer, area, "ln 1/4 · col 1");

    let find_buffer = key(&mut runtime, KeyCode::Char('f'), KeyModifiers::CONTROL);
    let opened = capture(&runtime, &find_buffer);
    trace("find-open", &opened);
    assert_eq!(opened.find.as_ref().map(|find| find.3), Some(true));
    assert!(footer_text(&find_buffer, area).contains("find "));

    let mut query_buffer = find_buffer;
    for character in "beta".chars() {
        query_buffer = key(&mut runtime, KeyCode::Char(character), KeyModifiers::NONE);
    }
    let query = capture(&runtime, &query_buffer);
    trace("find-query", &query);
    assert_eq!(
        query.text_value, document,
        "find changes only the query state"
    );
    assert_eq!(query.find, Some(("beta".to_owned(), Some(0), 1, true)));
    assert!(footer_text(&query_buffer, area).contains("find beta"));
    assert_eq!(query.cursor.map(|cursor| cursor.y), Some(area.bottom() - 1));
}

#[test]
fn idle_overflow_and_navigation_offsets_are_observed_in_the_real_frame() {
    let document = (0..32)
        .map(|line| format!("row {line:02} — 界 e\u{301}"))
        .collect::<Vec<_>>()
        .join("\n");
    let area = Rect::new(5, 3, 34, 8);
    let mut idle = runtime(&document, area, area.height, Theme::junie());
    let idle_buffer = present(&mut idle);
    let idle_frame = capture(&idle, &idle_buffer);
    trace("overflow-idle-unfocused", &idle_frame);
    assert_eq!(idle_frame.focus, Some(ELSEWHERE));
    assert_eq!(idle_frame.state_flags & StateFlags::FOCUSED.bits(), 0);
    assert!(idle_frame.scroll_viewport > 0);
    assert!(idle_frame.scroll_viewport < 32);
    // The canonical PR26 Code widget used the generic visible-range label for an idle
    // overflowing document. Current Code intentionally leaves this unfocused footer blank;
    // this assertion records the current contract without claiming canonical paint parity.
    assert!(footer_text(&idle_buffer, area).trim().is_empty());

    idle.set_focus(Some(EDITOR));
    let focused_buffer = present(&mut idle);
    let focused = capture(&idle, &focused_buffer);
    trace("overflow-focused-top", &focused);
    assert_footer_label(&focused_buffer, area, "ln 1/32 · col 1");
    let page_buffer = key(&mut idle, KeyCode::PageDown, KeyModifiers::NONE);
    let page = capture(&idle, &page_buffer);
    trace("overflow-page-down", &page);
    assert!(page.cursor_line > 0);
    assert!(page.scroll_offset > 0);
    assert!(page.scroll_viewport > 0);
    assert!(page.text.is_some());
    let page_label = format!("ln {}/32", page.cursor_line + 1);
    assert!(footer_text(&page_buffer, area).contains(&page_label));
    assert_eq!(page.cursor, None, "browse movement does not show the caret");

    let long_line = format!("HEAD-start::{}TAIL", "0123456789".repeat(12));
    let horizontal_area = Rect::new(11, 5, 30, 6);
    let mut horizontal = runtime(
        &long_line,
        horizontal_area,
        horizontal_area.height,
        Theme::junie(),
    );
    horizontal.set_focus(Some(EDITOR));
    let _ = present(&mut horizontal);
    let _ = key(&mut horizontal, KeyCode::Enter, KeyModifiers::NONE);
    let start_buffer = present(&mut horizontal);
    let start_text = row_text(
        &start_buffer,
        horizontal_area,
        horizontal
            .area_of_part(EDITOR, PartRef::of(Part::TEXT))
            .expect("TEXT region is registered")
            .y,
    );
    assert!(start_text.contains("HEAD"));
    assert!(!start_text.contains("TAIL"));
    let end_buffer = key(&mut horizontal, KeyCode::End, KeyModifiers::NONE);
    let end = capture(&horizontal, &end_buffer);
    trace("editing-horizontal-end", &end);
    assert!(end.editing);
    assert_eq!(end.cursor_column, long_line.len());
    // The actual End command reaches the final source cell and scrolls that suffix into view,
    // while this current frame publishes no terminal caret. Preserve that observation for
    // review without treating the hidden-caret result as an approved interaction policy.
    assert_eq!(end.cursor, None);
    let first_text_row = end.text.expect("TEXT region is registered").y;
    let text_line = row_text(&end_buffer, horizontal_area, first_text_row);
    assert!(
        text_line.contains("TAIL"),
        "right edge is visible: {text_line:?}"
    );
    assert!(
        !text_line.contains("HEAD"),
        "the distinctive initial prefix is outside the scrolled window: {text_line:?}"
    );

    for (width, height, expected_full_label) in [(42, 7, true), (12, 4, false), (2, 1, false)] {
        let small_area = Rect::new(3, 2, width, height);
        let mut small = runtime("alpha\nbeta", small_area, height, Theme::junie());
        small.set_focus(Some(EDITOR));
        let buffer = present(&mut small);
        let frame = capture(&small, &buffer);
        trace(&format!("fit-area-{width}x{height}"), &frame);
        let label = footer_text(&buffer, small_area);
        if expected_full_label {
            assert!(label.contains("ln 1/2 · col 1"));
        } else {
            assert!(label.chars().count() <= usize::from(width));
            assert!(!label.contains("ln 1/2 · col 1"));
        }
        assert!(frame.cursor.is_none_or(|cursor| {
            cursor.x >= small_area.x
                && cursor.x < small_area.right()
                && cursor.y >= small_area.y
                && cursor.y < small_area.bottom()
        }));
    }
}

#[test]
fn custom_meta_cells_resolve_and_render_at_all_four_color_levels() {
    let authored_fg = Color::Rgb(231, 23, 147);
    let authored_bg = Color::Rgb(17, 29, 41);
    let area = Rect::new(4, 5, 44, 7);
    let label = "ln 1/2 · col 1";

    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        let theme = Theme::junie()
            .downgrade(level)
            .override_family(Family::CODE, |recipe| {
                recipe.part(Part::META).base(
                    StylePatch::new()
                        .set_fg(Role::Custom(authored_fg))
                        .set_bg(Role::Custom(authored_bg))
                        .add(Modifier::ITALIC | Modifier::UNDERLINED)
                        .remove(Modifier::BOLD),
                );
            });
        let mut runtime = runtime("alpha\nbeta", area, area.height, theme.clone());
        runtime.set_focus(Some(EDITOR));
        let buffer = present(&mut runtime);
        let frame = capture(&runtime, &buffer);
        trace(&format!("custom-meta-{level:?}"), &frame);

        assert_footer_label(&buffer, area, label);
        let flags = runtime.state_of(EDITOR);
        let expected = theme
            .resolve(
                Family::CODE,
                Variant::DEFAULT,
                Part::META,
                flags,
                Surface::Canvas,
            )
            .style;
        let row = footer_text(&buffer, area);
        let byte_start = row.find(label).expect("position label is rendered");
        let cell_start = area
            .x
            .saturating_add(row[..byte_start].chars().count() as u16);
        for column in 0..label.chars().count() {
            let cell = buffer
                .cell(Position::new(cell_start + column as u16, area.bottom() - 1))
                .expect("each position-label grapheme has a real buffer cell");
            assert_eq!(cell.fg, expected.fg.unwrap_or(Color::Reset), "{level:?}");
            assert_eq!(cell.bg, expected.bg.unwrap_or(Color::Reset), "{level:?}");
            assert_eq!(
                cell.underline_color,
                expected.underline_color.unwrap_or(Color::Reset),
                "{level:?}"
            );
            assert!(cell.modifier.contains(Modifier::ITALIC), "{level:?}");
            assert!(cell.modifier.contains(Modifier::UNDERLINED), "{level:?}");
            assert!(
                !cell.modifier.contains(Modifier::BOLD),
                "explicit META removal survives at {level:?}"
            );
        }
    }
}
