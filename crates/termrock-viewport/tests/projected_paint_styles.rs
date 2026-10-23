use ratatui_core::backend::TestBackend;
use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Modifier, Style};
use ratatui_core::terminal::Terminal;
use termrock_runtime::runtime::App;
use termrock_runtime::{Cx, Focusability, Id, Input, LayerSpec, Part, PartRef, Response, Ui};
use termrock_test_support::Harness;
use termrock_theme::resolve::bind_role;
use termrock_theme::{ColorLevel, PaintStyle, Role, StylePatch, Surface, Theme};
use termrock_viewport::viewport::{
    CellPos, ProjectedText, TextViewport, ViewportAction, ViewportLine, ViewportState,
};

const OWNER: Id = Id::root("viewport.projected-paint.integration");
const MODAL_LAYER: Id = Id::root("viewport.projected-paint.modal");
const MODAL_CONTROL: Id = Id::root("viewport.projected-paint.modal-control");
const SCREEN: (u16, u16) = (40, 13);
const METER: &str = "12██·· e\u{301}界👩\u{200d}💻 tail\tend";
const REPLACEMENT: &str = "new界e\u{301}\tupdated-tail";
const SHRUNK: &str = "ne";
const LONG: &str =
    "a long second source row that proves horizontal and vertical positions remain native";
const REST: &str = "source row used to keep the vertical scrollbar live";
const ROWS: [&str; 20] = [
    METER, LONG, REST, REST, REST, REST, REST, REST, REST, REST, REST, REST, REST, REST, REST,
    REST, REST, REST, REST, REST,
];

const SPANS: [termrock_text::Span<'static>; 4] = [
    termrock_text::Span::new("12"),
    termrock_text::Span::new("██").bold(),
    termrock_text::Span::new("··").role(Role::Warning).italic(),
    termrock_text::Span::new(" e\u{301}界👩\u{200d}💻 tail"),
];

const BASE_PATCH: StylePatch = StylePatch::new()
    .set_fg(Role::Fg(termrock_theme::FgStep::Primary))
    .add(Modifier::BOLD)
    .add(Modifier::ITALIC);
const REMOVE_BOLD_PATCH: StylePatch = StylePatch::new().remove(Modifier::BOLD);

#[derive(Clone, Copy)]
enum SourceMode {
    Plain,
    Spans,
    ProjectedPlain,
    ProjectedSpans,
    Rich,
}

struct ProjectedPage {
    state: ViewportState,
    mode: SourceMode,
    rows: Vec<&'static str>,
    plain: ProjectedText,
    spans: ProjectedText,
    rich: ProjectedText,
    base_patch: &'static StylePatch,
    wrap: bool,
    actions: Vec<ViewportAction>,
    revision: u64,
    modal_open: bool,
}

impl ProjectedPage {
    fn new(mode: SourceMode, theme: &Theme) -> Self {
        Self::with_view_options(mode, theme, &BASE_PATCH, false)
    }

    fn with_view_options(
        mode: SourceMode,
        theme: &Theme,
        base_patch: &'static StylePatch,
        wrap: bool,
    ) -> Self {
        let mut state = ViewportState::default();
        state.set_follow(false);
        Self {
            state,
            mode,
            rows: ROWS.to_vec(),
            plain: plain_projection(&ROWS),
            spans: spans_projection(&ROWS),
            rich: rich_projection(theme, &ROWS),
            base_patch,
            wrap,
            actions: Vec::new(),
            revision: 1,
            modal_open: false,
        }
    }

    fn viewport(&self) -> TextViewport<'static> {
        TextViewport::new(OWNER)
            .horizontal_pan(true)
            .wrap(self.wrap)
            .layout_revision(1)
            .patch(self.base_patch)
    }

    fn borrowed_lines(&self) -> Vec<ViewportLine<'static>> {
        self.rows
            .iter()
            .enumerate()
            .map(|(index, line)| {
                if index == 0 && matches!(self.mode, SourceMode::Spans) {
                    ViewportLine::Spans(&SPANS)
                } else {
                    ViewportLine::Plain(line)
                }
            })
            .collect()
    }

    fn update_viewport(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let viewport = self.viewport().layout_revision(self.revision);
        let response = match self.mode {
            SourceMode::Plain | SourceMode::Spans => {
                let lines = self.borrowed_lines();
                viewport.update(cx, &mut self.state, &lines)
            }
            SourceMode::ProjectedPlain => {
                viewport.update_projected(cx, &mut self.state, &self.plain)
            }
            SourceMode::ProjectedSpans => {
                viewport.update_projected(cx, &mut self.state, &self.spans)
            }
            SourceMode::Rich => viewport.update_projected(cx, &mut self.state, &self.rich),
        };
        if let Some(action) = response.action_ref() {
            self.actions.push(action.clone());
        }
        if self.modal_open {
            cx.open_layer(
                MODAL_LAYER,
                LayerSpec::modal(OWNER).initial_focus(MODAL_CONTROL),
            );
        }
        response.erase()
    }

    fn draw_viewport(&self, ui: &mut Ui<'_>) {
        let viewport = self.viewport().layout_revision(self.revision);
        let area = viewport_area(ui.full());
        match self.mode {
            SourceMode::Plain | SourceMode::Spans => {
                let lines = self.borrowed_lines();
                let _ = viewport.draw(ui, area, &self.state, &lines);
            }
            SourceMode::ProjectedPlain => {
                let _ = viewport.draw_projected(ui, area, &self.state, &self.plain);
            }
            SourceMode::ProjectedSpans => {
                let _ = viewport.draw_projected(ui, area, &self.state, &self.spans);
            }
            SourceMode::Rich => {
                let _ = viewport.draw_projected(ui, area, &self.state, &self.rich);
            }
        }
        let _ = ui.layer(MODAL_LAYER, |ui, area| {
            ui.register_control(MODAL_CONTROL, area, Focusability::Focusable);
            ui.paint_str(area, "modal input owner", Style::default());
        });
    }

    fn replace_rows(&mut self, rows: Vec<&'static str>, theme: &Theme) {
        self.revision = self.revision.saturating_add(1);
        self.rows = rows;
        self.plain = plain_projection(&self.rows);
        self.spans = spans_projection(&self.rows);
        self.rich = rich_projection(theme, &self.rows);
    }

    fn append_row(&mut self, row: &'static str, theme: &Theme) {
        self.revision = self.revision.saturating_add(1);
        self.rows.push(row);
        self.plain = plain_projection(&self.rows);
        self.spans = spans_projection(&self.rows);
        self.rich = rich_projection(theme, &self.rows);
    }
}

impl App for ProjectedPage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.update_viewport(cx)
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.draw_viewport(ui);
    }

    fn min_size(&self) -> termrock_runtime::measure::Size {
        termrock_runtime::measure::Size {
            min: (0, 0),
            preferred: SCREEN,
        }
    }
}

fn plain_projection(rows: &[&'static str]) -> ProjectedText {
    let mut lines = ProjectedText::default();
    for line in rows.iter().copied() {
        lines.push_line([(line, None, Modifier::empty())]);
    }
    lines
}

fn spans_projection(rows: &[&'static str]) -> ProjectedText {
    let mut lines = ProjectedText::default();
    lines.push_line([
        ("12", None, Modifier::empty()),
        ("██", None, Modifier::BOLD),
        ("··", Some(Role::Warning), Modifier::ITALIC),
        (" e\u{301}界👩\u{200d}💻 tail", None, Modifier::empty()),
    ]);
    for line in rows.iter().copied().skip(1) {
        lines.push_line([(line, None, Modifier::empty())]);
    }
    lines
}

fn rich_projection(theme: &Theme, rows: &[&'static str]) -> ProjectedText {
    let warning = bind_role(theme, Role::Warning, Surface::Canvas).unwrap_or(Color::Reset);
    let field =
        bind_role(theme, Role::Surface(Surface::Field), Surface::Canvas).unwrap_or(Color::Reset);
    let warning_field = PaintStyle::bound(
        Style::new()
            .fg(warning)
            .bg(field)
            .add_modifier(Modifier::UNDERLINED),
        Some(Role::Warning),
        Some(Role::Surface(Surface::Field)),
        Surface::Canvas,
    );
    let bar = PaintStyle::new()
        .fg(Color::Rgb(231, 189, 37))
        .bg(Color::Rgb(23, 52, 91))
        .add_modifier(Modifier::BOLD)
        .remove_modifier(Modifier::ITALIC);
    let track = PaintStyle::new()
        .fg(Color::Reset)
        .bg(Color::Reset)
        .underline_color(Color::Reset)
        .remove_modifier(Modifier::BOLD)
        .add_modifier(Modifier::UNDERLINED);
    let mark = PaintStyle::new()
        .fg(Color::Rgb(175, 85, 205))
        .add_modifier(Modifier::DIM);
    let accent = PaintStyle::new()
        .fg(Color::Rgb(29, 177, 151))
        .add_modifier(Modifier::BOLD);
    let tail = PaintStyle::new()
        .fg(Color::Rgb(206, 212, 218))
        .remove_modifier(Modifier::ITALIC);

    let mut lines = ProjectedText::default();
    match rows.first().copied() {
        Some(METER) => lines.push_styled_line([
            ("12", warning_field),
            ("██", bar),
            ("··", track),
            (" ", PaintStyle::new()),
            ("e", accent),
            ("\u{301}", mark),
            ("界", tail),
            ("👩\u{200d}", mark),
            ("💻", tail),
            (" tail", accent),
            ("\tend", tail),
        ]),
        Some(REPLACEMENT) => lines.push_styled_line([
            ("new", warning_field),
            ("界", tail),
            ("e", accent),
            ("\u{301}", mark),
            ("\tupdated-tail", tail),
        ]),
        Some(first) => lines.push_styled_line([(first, accent)]),
        None => {}
    }
    for line in rows.iter().copied().skip(1) {
        lines.push_line([(line, None, Modifier::empty())]);
    }
    lines
}

fn harness(mode: SourceMode, theme: Theme) -> Harness<ProjectedPage> {
    Harness::new(ProjectedPage::new(mode, &theme), theme, SCREEN.0, SCREEN.1)
}

fn viewport_area(screen: Rect) -> Rect {
    Rect::new(
        screen.x.saturating_add(2),
        screen.y.saturating_add(1),
        screen.width.saturating_sub(6),
        screen.height.saturating_sub(3),
    )
}

fn viewport_text_area(screen: Rect) -> Rect {
    let area = viewport_area(screen);
    Rect::new(
        area.x.saturating_add(1),
        area.y,
        area.width.saturating_sub(2),
        area.height,
    )
}

fn configured_harness(
    mode: SourceMode,
    theme: Theme,
    size: (u16, u16),
    base_patch: &'static StylePatch,
    wrap: bool,
) -> Harness<ProjectedPage> {
    Harness::new(
        ProjectedPage::with_view_options(mode, &theme, base_patch, wrap),
        theme,
        size.0,
        size.1,
    )
}

fn resize_runtime_and_draw(
    harness: &mut Harness<ProjectedPage>,
    size: (u16, u16),
) -> Terminal<TestBackend> {
    let mut terminal = Terminal::new(TestBackend::new(size.0, size.1)).expect("test terminal");
    let area = terminal.get_frame().area();
    let runtime = harness.runtime_mut();
    let _ = runtime
        .handle(Input::Resize(size.0, size.1))
        .expect("runtime accepts terminal resize");
    let prepared = runtime
        .prepare(area)
        .expect("runtime prepares resized frame");
    terminal
        .draw(|frame| {
            runtime
                .draw_prepared(prepared, frame)
                .expect("runtime projects resized frame");
        })
        .expect("test backend draws resized frame");
    runtime
        .acknowledge_presented(prepared)
        .expect("runtime acknowledges resized frame");
    terminal
}

fn tick_runtime_and_draw(
    harness: &mut Harness<ProjectedPage>,
    size: (u16, u16),
) -> Terminal<TestBackend> {
    let mut terminal = Terminal::new(TestBackend::new(size.0, size.1)).expect("test terminal");
    let area = terminal.get_frame().area();
    let runtime = harness.runtime_mut();
    let _ = runtime
        .handle(Input::Tick)
        .expect("runtime accepts follow-up tick");
    let prepared = runtime
        .prepare(area)
        .expect("runtime prepares follow-up frame");
    terminal
        .draw(|frame| {
            runtime
                .draw_prepared(prepared, frame)
                .expect("runtime projects follow-up frame");
        })
        .expect("test backend draws follow-up frame");
    runtime
        .acknowledge_presented(prepared)
        .expect("runtime acknowledges follow-up frame");
    terminal
}

fn assert_same_geometry_and_cells(left: &Harness<ProjectedPage>, right: &Harness<ProjectedPage>) {
    assert_eq!(left.buffer(), right.buffer());
    assert_same_geometry(left, right);
}

fn assert_same_geometry_and_symbols(left: &Harness<ProjectedPage>, right: &Harness<ProjectedPage>) {
    assert_same_geometry(left, right);
    for y in 0..left.buffer().area.height {
        for x in 0..left.buffer().area.width {
            assert_eq!(
                left.cell(x, y).symbol(),
                right.cell(x, y).symbol(),
                "symbol differs at ({x}, {y})"
            );
        }
    }
}

fn assert_same_geometry(left: &Harness<ProjectedPage>, right: &Harness<ProjectedPage>) {
    assert_eq!(left.area_of(OWNER), right.area_of(OWNER));
    assert_eq!(
        left.area_of_part(OWNER, PartRef::of(Part::TEXT)),
        right.area_of_part(OWNER, PartRef::of(Part::TEXT))
    );
    assert_eq!(left.cursor(), right.cursor());
    assert_eq!(left.focus(), right.focus());
    assert_eq!(left.state_of(OWNER), right.state_of(OWNER));
}

#[test]
fn old_plain_and_span_projection_paths_keep_exact_full_buffer_and_hit_parity() {
    let theme = Theme::junie();
    let plain = harness(SourceMode::Plain, theme.clone());
    let projected_plain = harness(SourceMode::ProjectedPlain, theme.clone());
    assert_same_geometry_and_cells(&plain, &projected_plain);

    let spans = harness(SourceMode::Spans, theme.clone());
    let projected_spans = harness(SourceMode::ProjectedSpans, theme);
    assert_same_geometry_and_cells(&spans, &projected_spans);
}

#[test]
fn rich_projection_paints_complete_channels_and_keeps_native_selection_and_copy() {
    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        for theme in [Theme::junie(), Theme::paper()] {
            let theme = theme.downgrade(level);
            let mut plain = harness(SourceMode::ProjectedPlain, theme.clone());
            let mut rich = harness(SourceMode::Rich, theme.clone());
            let text_area = rich
                .area_of_part(OWNER, PartRef::of(Part::TEXT))
                .expect("viewport text part is registered");
            let before_copy_symbols = (0..text_area.width)
                .map(|dx| rich.cell(text_area.x + dx, text_area.y).symbol().to_owned())
                .collect::<Vec<_>>();

            // Native geometry and glyph positions are identical even though
            // each projected run carries a complete style.
            assert_eq!(plain.area_of(OWNER), rich.area_of(OWNER));
            assert_eq!(
                plain.area_of_part(OWNER, PartRef::of(Part::TEXT)),
                Some(text_area)
            );
            assert_eq!(
                plain.area_of_part(OWNER, PartRef::of(Part::TRACK)),
                rich.area_of_part(OWNER, PartRef::of(Part::TRACK))
            );
            for dx in 0..text_area.width {
                assert_eq!(
                    plain.cell(text_area.x + dx, text_area.y).symbol(),
                    rich.cell(text_area.x + dx, text_area.y).symbol(),
                    "glyph changed at display column {dx} for {level:?}"
                );
            }

            let warning = bind_role(&theme, Role::Warning, Surface::Canvas).unwrap_or(Color::Reset);
            let field = bind_role(&theme, Role::Surface(Surface::Field), Surface::Canvas)
                .unwrap_or(Color::Reset);
            let readout = rich.cell(text_area.x, text_area.y);
            assert_eq!(readout.fg, warning);
            assert_eq!(readout.bg, field);
            assert!(
                readout
                    .modifier
                    .contains(Modifier::BOLD | Modifier::ITALIC | Modifier::UNDERLINED)
            );

            let bar = rich.cell(text_area.x + 2, text_area.y);
            assert_eq!(bar.fg, Color::Rgb(231, 189, 37));
            assert_eq!(bar.bg, Color::Rgb(23, 52, 91));
            assert!(bar.modifier.contains(Modifier::BOLD));
            assert!(!bar.modifier.contains(Modifier::ITALIC));

            // The viewport's base carries bold+italic; the track run explicitly
            // clears bold and both inherited ink channels while adding underline.
            let track = rich.cell(text_area.x + 4, text_area.y);
            assert_eq!(track.fg, Color::Reset);
            assert_eq!(track.bg, Color::Reset);
            assert_eq!(track.underline_color, Color::Reset);
            assert!(!track.modifier.contains(Modifier::BOLD));
            assert!(track.modifier.contains(Modifier::UNDERLINED));

            // Runs split both a combining grapheme and a joined emoji. The
            // native first-scalar source position owns the paint for each.
            assert_eq!(rich.cell(text_area.x + 7, text_area.y).symbol(), "e\u{301}");
            assert_eq!(
                rich.cell(text_area.x + 7, text_area.y).fg,
                Color::Rgb(29, 177, 151)
            );
            assert_eq!(
                rich.cell(text_area.x + 10, text_area.y).symbol(),
                "👩\u{200d}💻"
            );
            assert_eq!(
                rich.cell(text_area.x + 10, text_area.y).fg,
                Color::Rgb(175, 85, 205)
            );
            for dx in 17..20 {
                let tab_cell = rich.cell(text_area.x + dx, text_area.y);
                assert_eq!(tab_cell.symbol(), " ");
                assert_eq!(tab_cell.fg, Color::Rgb(206, 212, 218));
                assert!(tab_cell.modifier.contains(Modifier::BOLD));
                assert!(!tab_cell.modifier.contains(Modifier::ITALIC));
            }

            // Selection color and background remain owned by the viewport's
            // selected TEXT style, even when the producer carries ink there.
            plain
                .app_mut()
                .state
                .select(CellPos::new(0, 2), CellPos::new(0, 4));
            plain.draw();
            rich.app_mut()
                .state
                .select(CellPos::new(0, 2), CellPos::new(0, 4));
            rich.draw();
            let selected = rich.cell(text_area.x + 2, text_area.y);
            assert_eq!(selected.fg, plain.cell(text_area.x + 2, text_area.y).fg);
            assert_eq!(selected.bg, plain.cell(text_area.x + 2, text_area.y).bg);
            assert!(selected.modifier.contains(Modifier::BOLD));
            assert!(
                !selected.modifier.contains(Modifier::ITALIC),
                "selected rich run must retain its explicit italic removal"
            );

            // Tabs are expanded into viewport-owned cells, but retain the
            // originating run's modifier patch while selected.
            plain
                .app_mut()
                .state
                .select(CellPos::new(0, 17), CellPos::new(0, 21));
            rich.app_mut()
                .state
                .select(CellPos::new(0, 17), CellPos::new(0, 21));
            plain.draw();
            rich.draw();
            for dx in 17..21 {
                let selected_tab = rich.cell(text_area.x + dx, text_area.y);
                let plain_tab = plain.cell(text_area.x + dx, text_area.y);
                assert_eq!(selected_tab.fg, plain_tab.fg);
                assert_eq!(selected_tab.bg, plain_tab.bg);
                assert!(selected_tab.modifier.contains(Modifier::BOLD));
                assert!(!selected_tab.modifier.contains(Modifier::ITALIC));
            }

            // Source selection and copy remain in logical display cells and
            // return the source graphemes, not a rendered/truncated string.
            rich.app_mut()
                .state
                .select(CellPos::new(0, 7), CellPos::new(0, 12));
            rich.draw();
            assert!(rich.tab_to(OWNER));
            let _ = rich.key(termrock_runtime::event::KeyCode::Char('y'));
            assert!(
                rich.app()
                    .actions
                    .contains(&ViewportAction::Copy("e\u{301}界👩\u{200d}💻".to_owned()))
            );
            let mut direct_copy = String::new();
            assert!(
                rich.app()
                    .state
                    .copy_from_projected(&rich.app().rich, &mut direct_copy)
            );
            assert_eq!(direct_copy, "e\u{301}界👩\u{200d}💻");
            rich.app_mut()
                .state
                .select(CellPos::new(0, 12), CellPos::new(0, 7));
            rich.draw();
            let _ = rich.key(termrock_runtime::event::KeyCode::Char('y'));
            assert_eq!(
                rich.app().actions.last(),
                Some(&ViewportAction::Copy("e\u{301}界👩\u{200d}💻".to_owned()))
            );
            assert_eq!(
                before_copy_symbols,
                (0..text_area.width)
                    .map(|dx| rich.cell(text_area.x + dx, text_area.y).symbol().to_owned())
                    .collect::<Vec<_>>()
            );
        }
    }
}

#[test]
fn selected_rich_modifier_additions_match_legacy_span_additions() {
    let theme = Theme::junie();
    let mut rich = configured_harness(
        SourceMode::Rich,
        theme.clone(),
        SCREEN,
        &REMOVE_BOLD_PATCH,
        false,
    );
    let mut spans = configured_harness(SourceMode::Spans, theme, SCREEN, &REMOVE_BOLD_PATCH, false);
    rich.app_mut()
        .state
        .select(CellPos::new(0, 2), CellPos::new(0, 4));
    spans
        .app_mut()
        .state
        .select(CellPos::new(0, 2), CellPos::new(0, 4));
    rich.draw();
    spans.draw();

    let rich_text = rich
        .area_of_part(OWNER, PartRef::of(Part::TEXT))
        .expect("rich text part");
    let spans_text = spans
        .area_of_part(OWNER, PartRef::of(Part::TEXT))
        .expect("span text part");
    for dx in 2..4 {
        let rich_bar = rich.cell(rich_text.x + dx, rich_text.y);
        let span_bar = spans.cell(spans_text.x + dx, spans_text.y);
        assert_eq!(rich_bar.fg, span_bar.fg);
        assert_eq!(rich_bar.bg, span_bar.bg);
        assert_eq!(
            rich_bar.modifier.contains(Modifier::BOLD),
            span_bar.modifier.contains(Modifier::BOLD),
            "rich producer's BOLD addition must survive a selected base that removes BOLD"
        );
        assert!(rich_bar.modifier.contains(Modifier::BOLD));
    }
}

#[test]
fn selected_expanded_tab_cells_keep_run_modifiers_and_selected_ink() {
    let theme = Theme::junie();
    let mut rich = harness(SourceMode::Rich, theme.clone());
    let mut plain = harness(SourceMode::ProjectedPlain, theme);
    for harness in [&mut rich, &mut plain] {
        harness
            .app_mut()
            .state
            .select(CellPos::new(0, 17), CellPos::new(0, 21));
        harness.draw();
    }
    let rich_text = rich
        .area_of_part(OWNER, PartRef::of(Part::TEXT))
        .expect("rich text part");
    let plain_text = plain
        .area_of_part(OWNER, PartRef::of(Part::TEXT))
        .expect("plain text part");
    for dx in 17..21 {
        let actual = rich.cell(rich_text.x + dx, rich_text.y);
        let expected = plain.cell(plain_text.x + dx, plain_text.y);
        assert_eq!(actual.symbol(), " ");
        assert_eq!(actual.fg, expected.fg);
        assert_eq!(actual.bg, expected.bg);
        assert!(actual.modifier.contains(Modifier::BOLD));
        assert!(!actual.modifier.contains(Modifier::ITALIC));
    }
}

#[test]
fn rich_wrapping_keeps_boundary_styles_source_cells_and_pointer_copy_native() {
    let theme = Theme::junie();
    let mut rich = configured_harness(SourceMode::Rich, theme.clone(), (16, 11), &BASE_PATCH, true);
    let mut plain = configured_harness(
        SourceMode::ProjectedPlain,
        theme,
        (16, 11),
        &BASE_PATCH,
        true,
    );
    let text = rich
        .area_of_part(OWNER, PartRef::of(Part::TEXT))
        .expect("wrapped viewport text part");
    assert_eq!(text.width, 8);
    assert_eq!(
        rich.app().state.scroll().content_len(),
        plain.app().state.scroll().content_len()
    );

    // At width 8 the combining grapheme fills the first visual row. The CJK
    // and joined-emoji graphemes continue on the next row, even though the
    // emoji's UTF-8 bytes cross rich-run boundaries.
    assert_eq!(rich.cell(text.x + 7, text.y).symbol(), "e\u{301}");
    assert_eq!(rich.cell(text.x, text.y + 1).symbol(), "界");
    assert_eq!(rich.cell(text.x + 2, text.y + 1).symbol(), "👩\u{200d}💻");
    assert_eq!(rich.cell(text.x, text.y + 1).fg, Color::Rgb(206, 212, 218));
    assert_eq!(
        rich.cell(text.x + 2, text.y + 1).fg,
        Color::Rgb(175, 85, 205)
    );

    // The selection begins at the combining grapheme on visual row one, then
    // continues over the CJK and joined-emoji graphemes on visual row two.
    // Selection ink remains viewport-owned while producer modifiers survive.
    rich.app_mut()
        .state
        .select(CellPos::new(0, 7), CellPos::new(0, 12));
    plain
        .app_mut()
        .state
        .select(CellPos::new(0, 7), CellPos::new(0, 12));
    rich.draw();
    plain.draw();
    for (dx, dy) in [(7, 0), (0, 1), (2, 1)] {
        let actual = rich.cell(text.x + dx, text.y + dy);
        let expected = plain.cell(text.x + dx, text.y + dy);
        assert_eq!(actual.fg, expected.fg);
        assert_eq!(actual.bg, expected.bg);
    }
    let selected_cjk = rich.cell(text.x, text.y + 1);
    assert!(selected_cjk.modifier.contains(Modifier::BOLD));
    assert!(!selected_cjk.modifier.contains(Modifier::ITALIC));
    assert!(
        rich.cell(text.x + 2, text.y + 1)
            .modifier
            .contains(Modifier::DIM)
    );
    let mut copied = String::new();
    assert!(
        rich.app()
            .state
            .copy_from_projected(&rich.app().rich, &mut copied)
    );
    assert_eq!(copied, "e\u{301}界👩\u{200d}💻");

    // The pointer endpoint is on a soft-wrapped continuation row. Native
    // hit reconciliation maps it back to the same logical source interval.
    rich.app_mut().state.clear_selection();
    plain.app_mut().state.clear_selection();
    rich.draw();
    plain.draw();
    let from = (text.x + 7, text.y);
    let to = (text.x + 4, text.y + 1);
    let _ = rich.drag(from, to);
    let _ = plain.drag(from, to);
    let expected = Some((CellPos::new(0, 7), CellPos::new(0, 12)));
    assert_eq!(rich.app().state.selection(), expected);
    assert_eq!(rich.app().state.selection(), plain.app().state.selection());
    assert!(rich.tab_to(OWNER));
    let _ = rich.key(termrock_runtime::event::KeyCode::Char('y'));
    assert!(plain.tab_to(OWNER));
    let _ = plain.key(termrock_runtime::event::KeyCode::Char('y'));
    assert_eq!(
        rich.app().actions.last(),
        Some(&ViewportAction::Copy("e\u{301}界👩\u{200d}💻".to_owned()))
    );
    assert_eq!(rich.app().actions, plain.app().actions);

    // Widening the real frame widens the viewport and changes the wrapped
    // row budget on the very first resized draw, without a hand-maintained
    // test rectangle.
    let before = rich
        .area_of_part(OWNER, PartRef::of(Part::TEXT))
        .expect("text area before resize");
    assert_eq!(
        rich.cell(before.x + 2, before.y + 1).symbol(),
        "👩\u{200d}💻"
    );
    let narrow_track = rich
        .area_of_part(OWNER, PartRef::of(Part::TRACK))
        .expect("vertical track before resize");
    let narrow_thumb = rich
        .area_of_part(OWNER, PartRef::of(Part::THUMB))
        .expect("vertical thumb before resize");
    let narrow_rows = rich.app().state.scroll().content_len();
    let rich_resized = resize_runtime_and_draw(&mut rich, (20, 11));
    let plain_resized = resize_runtime_and_draw(&mut plain, (20, 11));
    let after = rich
        .area_of_part(OWNER, PartRef::of(Part::TEXT))
        .expect("text area on first resized frame");
    assert_eq!(before.width, 8);
    assert_eq!(after.width, 12);
    assert_ne!(before.width, after.width);
    assert_eq!(
        rich.area_of_part(OWNER, PartRef::of(Part::TEXT)),
        plain.area_of_part(OWNER, PartRef::of(Part::TEXT))
    );
    let wide_track = rich
        .area_of_part(OWNER, PartRef::of(Part::TRACK))
        .expect("vertical track after resize");
    let wide_thumb = rich
        .area_of_part(OWNER, PartRef::of(Part::THUMB))
        .expect("vertical thumb after resize");
    assert_eq!(
        wide_track,
        plain.area_of_part(OWNER, PartRef::of(Part::TRACK)).unwrap()
    );
    assert_eq!(
        wide_thumb,
        plain.area_of_part(OWNER, PartRef::of(Part::THUMB)).unwrap()
    );
    assert_ne!(narrow_track, wide_track);
    assert_ne!(narrow_thumb, wide_thumb);
    assert_same_geometry(&rich, &plain);
    let rich_buffer = rich_resized.backend().buffer();
    let plain_buffer = plain_resized.backend().buffer();
    let resized_emoji = rich_buffer
        .cell((after.x + 10, after.y))
        .expect("emoji in resized buffer");
    let plain_emoji = plain_buffer
        .cell((after.x + 10, after.y))
        .expect("plain emoji in resized buffer");
    assert_eq!(resized_emoji.symbol(), "👩\u{200d}💻");
    assert!(resized_emoji.modifier.contains(Modifier::DIM));
    let resized_tail = rich_buffer
        .cell((after.x + 1, after.y + 1))
        .expect("tail in resized buffer");
    assert_eq!(resized_tail.symbol(), "t");
    assert_eq!(resized_tail.fg, Color::Rgb(29, 177, 151));
    let plain_tail = plain_resized
        .backend()
        .buffer()
        .cell((after.x + 1, after.y + 1))
        .expect("plain tail in resized buffer");
    assert_eq!(plain_tail.symbol(), resized_tail.symbol());
    assert_eq!(resized_emoji.fg, plain_emoji.fg);
    assert_eq!(resized_emoji.bg, plain_emoji.bg);

    // Resize publishes the new frame geometry before the app's next update.
    // The first-frame cells above prove immediate wrapping; this subsequent
    // normal update lets the stored scroll model consume that published width.
    let _ = tick_runtime_and_draw(&mut rich, (20, 11));
    let _ = tick_runtime_and_draw(&mut plain, (20, 11));
    assert!(rich.app().state.scroll().content_len() < narrow_rows);
    assert_eq!(
        rich.app().state.scroll().content_len(),
        plain.app().state.scroll().content_len()
    );
}

#[test]
fn rich_wrap_omits_an_unfittable_wide_cluster_without_losing_copy_source() {
    let theme = Theme::junie();
    let mut rich = configured_harness(SourceMode::Rich, theme.clone(), (9, 8), &BASE_PATCH, true);
    rich.app_mut().replace_rows(vec!["界x"], &theme);
    let _ = rich.tick();
    rich.draw();

    let text = rich
        .area_of_part(OWNER, PartRef::of(Part::TEXT))
        .expect("one-cell wrapped text part");
    assert_eq!(text.width, 1);
    assert_eq!(rich.app().state.scroll().content_len(), 2);
    assert_eq!(rich.cell(text.x, text.y).symbol(), " ");
    assert_eq!(rich.cell(text.x, text.y + 1).symbol(), "x");
    assert_eq!(rich.cell(text.x, text.y + 1).fg, Color::Rgb(29, 177, 151));

    rich.app_mut()
        .state
        .select(CellPos::new(0, 0), CellPos::new(0, 3));
    let mut copied = String::new();
    assert!(
        rich.app()
            .state
            .copy_from_projected(&rich.app().rich, &mut copied)
    );
    assert_eq!(copied, "界x");
}

#[test]
fn rich_projection_replaces_same_count_source_then_reconciles_selected_shrink() {
    let theme = Theme::junie();
    let mut rich = harness(SourceMode::Rich, theme.clone());
    let old_line_count = rich.app().rich.line_count();
    let mut replacement_rows = ROWS.to_vec();
    replacement_rows[0] = REPLACEMENT;
    rich.app_mut().replace_rows(replacement_rows, &theme);
    let _ = rich.tick();
    rich.draw();

    assert_eq!(rich.app().rows.len(), ROWS.len());
    assert_eq!(rich.app().rich.line_count(), old_line_count);
    let text = rich
        .area_of_part(OWNER, PartRef::of(Part::TEXT))
        .expect("text part after same-count source replacement");
    assert_eq!(rich.cell(text.x, text.y).symbol(), "n");
    assert_eq!(
        rich.cell(text.x, text.y).fg,
        bind_role(&theme, Role::Warning, Surface::Canvas).unwrap_or(Color::Reset)
    );
    assert_eq!(rich.cell(text.x + 3, text.y).symbol(), "界");
    assert_eq!(rich.cell(text.x + 3, text.y).fg, Color::Rgb(206, 212, 218));
    assert!(
        !rich
            .cell(text.x + 3, text.y)
            .modifier
            .contains(Modifier::ITALIC)
    );
    for dx in 6..10 {
        let tab = rich.cell(text.x + dx, text.y);
        assert_eq!(tab.symbol(), " ");
        assert_eq!(tab.fg, Color::Rgb(206, 212, 218));
        assert!(tab.modifier.contains(Modifier::BOLD));
    }

    rich.app_mut()
        .state
        .select(CellPos::new(0, 0), CellPos::new(0, 6));
    let mut same_count_copy = String::new();
    assert!(
        rich.app()
            .state
            .copy_from_projected(&rich.app().rich, &mut same_count_copy)
    );
    assert_eq!(same_count_copy, "new界e\u{301}");
    rich.app_mut().state.clear_selection();
    rich.draw();

    assert!(rich.tab_to(OWNER));
    let _ = rich.key(termrock_runtime::event::KeyCode::PageDown);
    rich.draw();
    assert!(rich.app().state.scroll().offset() > 0);
    rich.app_mut()
        .state
        .select(CellPos::new(19, 0), CellPos::new(19, 2));
    rich.draw();
    assert_eq!(
        rich.app().state.selection(),
        Some((CellPos::new(19, 0), CellPos::new(19, 2)))
    );
    let mut pre_shrink_copy = String::new();
    assert!(
        rich.app()
            .state
            .copy_from_projected(&rich.app().rich, &mut pre_shrink_copy)
    );
    assert_eq!(pre_shrink_copy, "so");

    rich.app_mut().replace_rows(vec![SHRUNK], &theme);
    let _ = rich.tick();
    rich.draw();
    assert_eq!(rich.app().rich.line_count(), 1);
    assert_eq!(rich.app().state.scroll().offset(), 0);
    assert_eq!(rich.focus(), Some(OWNER));
    assert_eq!(
        rich.app().state.selection(),
        Some((CellPos::new(19, 0), CellPos::new(19, 2))),
        "source replacement is displayed before a selection command reconciles its saved range"
    );
    let text = rich
        .area_of_part(OWNER, PartRef::of(Part::TEXT))
        .expect("text part before command-triggered reconciliation");
    assert_eq!(rich.cell(text.x, text.y).symbol(), "n");
    assert_eq!(rich.cell(text.x + 1, text.y).symbol(), "e");

    // Shift+Right is a real boundary command: the reconciled head is already
    // at the end of the new source, so it cannot move farther right.
    let _ = rich.key_mod(
        termrock_runtime::event::KeyCode::Right,
        termrock_runtime::event::KeyModifiers::SHIFT,
    );
    assert_eq!(
        rich.app().state.selection(),
        Some((CellPos::new(0, 0), CellPos::new(0, 2)))
    );
    let text = rich
        .area_of_part(OWNER, PartRef::of(Part::TEXT))
        .expect("text part after selected-source shrink");
    assert_eq!(rich.cell(text.x, text.y).symbol(), "n");
    assert_eq!(rich.cell(text.x + 1, text.y).symbol(), "e");
    let mut after_shrink_copy = String::new();
    assert!(
        rich.app()
            .state
            .copy_from_projected(&rich.app().rich, &mut after_shrink_copy)
    );
    assert_eq!(after_shrink_copy, "ne");

    // Appending after reconciliation must not revive the obsolete row-19
    // endpoint that was present before the boundary command.
    rich.app_mut().append_row(REST, &theme);
    let _ = rich.tick();
    rich.draw();
    assert_eq!(rich.app().rich.line_count(), 2);
    assert_eq!(rich.app().state.scroll().offset(), 0);
    assert_eq!(
        rich.app().state.selection(),
        Some((CellPos::new(0, 0), CellPos::new(0, 2)))
    );
    let mut after_append_copy = String::new();
    assert!(
        rich.app()
            .state
            .copy_from_projected(&rich.app().rich, &mut after_append_copy)
    );
    assert_eq!(after_append_copy, "ne");
}

#[test]
fn rich_projection_keeps_native_pan_scroll_resize_and_replacement_behavior() {
    let theme = Theme::junie();
    let mut baseline = harness(SourceMode::ProjectedPlain, theme.clone());
    let mut rich = harness(SourceMode::Rich, theme);
    assert_same_geometry_and_symbols(&baseline, &rich);

    assert!(baseline.tab_to(OWNER));
    assert!(rich.tab_to(OWNER));
    let _ = baseline.key(termrock_runtime::event::KeyCode::PageDown);
    let _ = rich.key(termrock_runtime::event::KeyCode::PageDown);
    assert!(rich.app().state.scroll().offset() > 0);
    let _ = baseline.key(termrock_runtime::event::KeyCode::PageUp);
    let _ = rich.key(termrock_runtime::event::KeyCode::PageUp);
    assert_eq!(rich.app().state.scroll().offset(), 0);
    let _ = baseline.key(termrock_runtime::event::KeyCode::Right);
    let _ = rich.key(termrock_runtime::event::KeyCode::Right);
    assert!(rich.app().state.horizontal_offset() > 0);
    let panned_text = rich
        .area_of_part(OWNER, PartRef::of(Part::TEXT))
        .expect("text part after horizontal pan");
    assert_eq!(rich.app().state.horizontal_offset(), 4);
    assert_eq!(rich.cell(panned_text.x, panned_text.y).symbol(), "·");
    assert_eq!(rich.cell(panned_text.x, panned_text.y).fg, Color::Reset);
    assert_eq!(
        baseline.app().state.scroll().offset(),
        rich.app().state.scroll().offset()
    );
    assert_eq!(
        baseline.app().state.horizontal_offset(),
        rich.app().state.horizontal_offset()
    );
    assert_eq!(
        baseline.app().state.selection(),
        rich.app().state.selection()
    );
    assert_eq!(baseline.app().state.caret(), rich.app().state.caret());
    assert_eq!(baseline.app().actions, rich.app().actions);
    assert_same_geometry_and_symbols(&baseline, &rich);

    let text = rich
        .area_of_part(OWNER, PartRef::of(Part::TEXT))
        .expect("text part for pointer selection");
    let producer_caret = CellPos::new(0, 17);
    baseline.app_mut().state.set_caret(Some(producer_caret));
    rich.app_mut().state.set_caret(Some(producer_caret));
    baseline.draw();
    rich.draw();
    let from_text = (text.x + 2, text.y);
    // Stop at the logical boundary after the joined emoji rather than inside
    // its second physical cell, so reconciliation preserves the grapheme edge.
    let to_text = (text.x + 8, text.y);
    let source_left = rich.app().state.horizontal_offset() + 2;
    let source_right = rich.app().state.horizontal_offset() + 8;
    let _ = baseline.drag(from_text, to_text);
    let _ = rich.drag(from_text, to_text);
    assert!(rich.app().state.selection().is_some());
    assert_eq!(
        baseline.app().state.selection(),
        rich.app().state.selection()
    );
    assert_eq!(
        rich.app().state.selection(),
        Some((CellPos::new(0, source_left), CellPos::new(0, source_right)))
    );
    assert_eq!(baseline.app().state.caret(), rich.app().state.caret());
    assert_eq!(rich.app().state.caret(), Some(producer_caret));
    assert_eq!(baseline.app().actions, rich.app().actions);
    assert_same_geometry_and_symbols(&baseline, &rich);

    let _ = baseline.key(termrock_runtime::event::KeyCode::Char('y'));
    let _ = rich.key(termrock_runtime::event::KeyCode::Char('y'));
    assert_eq!(baseline.app().actions, rich.app().actions);
    assert_eq!(
        rich.app().actions.last(),
        Some(&ViewportAction::Copy(" e\u{301}界👩\u{200d}💻".to_owned()))
    );

    let before = rich.app().state.scroll().offset();
    let thumb = rich
        .area_of_part(OWNER, PartRef::of(Part::THUMB))
        .expect("overflow creates a vertical thumb");
    let track = rich
        .area_of_part(OWNER, PartRef::of(Part::TRACK))
        .expect("overflow creates a vertical track");
    let from = (thumb.x + thumb.width / 2, thumb.y);
    let to = (
        track.right().saturating_sub(1),
        track.bottom().saturating_sub(1),
    );
    let _ = baseline.drag(from, to);
    let _ = rich.drag(from, to);
    assert!(rich.app().state.scroll().offset() > before);
    assert_eq!(
        baseline.app().state.scroll().offset(),
        rich.app().state.scroll().offset()
    );
    assert_eq!(baseline.app().actions, rich.app().actions);
    assert_same_geometry_and_symbols(&baseline, &rich);

    // Clear previous interactions before replacing the displayed source.
    baseline.app_mut().state.reset_scroll_offsets();
    rich.app_mut().state.reset_scroll_offsets();
    baseline.app_mut().state.clear_selection();
    rich.app_mut().state.clear_selection();
    baseline.draw();
    rich.draw();

    // Replacing the displayed source with fewer lines exercises the viewport's
    // existing source/layout reconciliation rather than a style-owned cache.
    baseline
        .app_mut()
        .replace_rows(vec![METER], &Theme::junie());
    rich.app_mut().replace_rows(vec![METER], &Theme::junie());
    baseline.draw();
    rich.draw();
    assert_eq!(
        baseline.app().state.selection(),
        rich.app().state.selection()
    );
    assert_eq!(
        baseline.app().state.scroll().offset(),
        rich.app().state.scroll().offset()
    );
    assert_eq!(baseline.area_of(OWNER), rich.area_of(OWNER));
    assert_same_geometry_and_symbols(&baseline, &rich);

    let before_resize = rich
        .area_of_part(OWNER, PartRef::of(Part::TEXT))
        .expect("text part before resize");
    assert_eq!(
        before_resize,
        viewport_text_area(Rect::new(0, 0, SCREEN.0, SCREEN.1))
    );
    let resized_baseline = resize_runtime_and_draw(&mut baseline, (46, 15));
    let resized_rich = resize_runtime_and_draw(&mut rich, (46, 15));
    let after_resize = rich
        .area_of_part(OWNER, PartRef::of(Part::TEXT))
        .expect("text part on first resized frame");
    assert_eq!(after_resize, viewport_text_area(Rect::new(0, 0, 46, 15)));
    assert_ne!(before_resize.width, after_resize.width);
    assert_ne!(before_resize.height, after_resize.height);
    let resized_buffer = resized_rich.backend().buffer();
    let resized_first = resized_buffer
        .cell((after_resize.x, after_resize.y))
        .expect("first source cell after resize");
    assert_eq!(resized_first.symbol(), "1");
    assert_eq!(
        resized_first.fg,
        bind_role(&Theme::junie(), Role::Warning, Surface::Canvas).unwrap_or(Color::Reset)
    );
    assert_eq!(
        *resized_baseline.backend().buffer().area(),
        *resized_rich.backend().buffer().area()
    );
    assert_eq!(
        rich.area_of_part(OWNER, PartRef::of(Part::TEXT)),
        Some(after_resize)
    );
}

#[test]
fn rich_projection_is_rebuilt_when_theme_dependent_paint_changes() {
    let first_theme = Theme::junie();
    let mut rich = harness(SourceMode::Rich, first_theme.clone());
    let text = rich
        .area_of_part(OWNER, PartRef::of(Part::TEXT))
        .expect("viewport text part");
    let first = rich.cell(text.x, text.y).fg;

    let mut next_theme = Theme::paper();
    next_theme.color.warning = Color::Rgb(1, 2, 3);
    rich.runtime_mut().set_theme(next_theme.clone());
    let rows = rich.app().rows.clone();
    rich.app_mut().rich = rich_projection(&next_theme, &rows);
    rich.draw();

    let text = rich
        .area_of_part(OWNER, PartRef::of(Part::TEXT))
        .expect("viewport text part after theme change");
    let next = rich.cell(text.x, text.y).fg;
    assert_eq!(
        next,
        bind_role(&next_theme, Role::Warning, Surface::Canvas).unwrap_or(Color::Reset)
    );
    assert_ne!(
        first, next,
        "the semantic warning ink differs between these themes"
    );
}

#[test]
fn rich_projection_does_not_take_keyboard_input_from_a_modal_owner() {
    let mut rich = harness(SourceMode::Rich, Theme::junie());
    assert!(rich.tab_to(OWNER));
    rich.app_mut().modal_open = true;
    let _ = rich.tick();
    assert_eq!(rich.focus(), Some(MODAL_CONTROL));

    let before = (
        rich.app().state.scroll().offset(),
        rich.app().state.horizontal_offset(),
        rich.app().state.selection(),
        rich.app().state.caret(),
        rich.app().actions.len(),
    );
    let _ = rich.key(termrock_runtime::event::KeyCode::Right);
    assert_eq!(
        before,
        (
            rich.app().state.scroll().offset(),
            rich.app().state.horizontal_offset(),
            rich.app().state.selection(),
            rich.app().state.caret(),
            rich.app().actions.len(),
        )
    );
}
