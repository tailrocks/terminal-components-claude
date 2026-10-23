use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Modifier, Style};
use termrock_runtime::runtime::App;
use termrock_runtime::{Cx, Focusability, Id, LayerSpec, Part, PartRef, Response, Ui};
use termrock_test_support::Harness;
use termrock_theme::resolve::bind_role;
use termrock_theme::{ColorLevel, PaintStyle, Role, StylePatch, Surface, Theme};
use termrock_viewport::viewport::{
    CellPos, ProjectedText, TextViewport, ViewportAction, ViewportLine, ViewportState,
};

const OWNER: Id = Id::root("viewport.projected-paint.integration");
const MODAL_LAYER: Id = Id::root("viewport.projected-paint.modal");
const MODAL_CONTROL: Id = Id::root("viewport.projected-paint.modal-control");
const AREA: Rect = Rect::new(2, 1, 34, 10);
const SCREEN: (u16, u16) = (40, 13);
const METER: &str = "12██·· e\u{301}界👩\u{200d}💻 tail\tend";
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
    actions: Vec<ViewportAction>,
    revision: u64,
    modal_open: bool,
}

impl ProjectedPage {
    fn new(mode: SourceMode, theme: &Theme) -> Self {
        let mut state = ViewportState::default();
        state.set_follow(false);
        Self {
            state,
            mode,
            rows: ROWS.to_vec(),
            plain: plain_projection(&ROWS),
            spans: spans_projection(&ROWS),
            rich: rich_projection(theme, &ROWS),
            actions: Vec::new(),
            revision: 1,
            modal_open: false,
        }
    }

    fn viewport() -> TextViewport<'static> {
        TextViewport::new(OWNER)
            .horizontal_pan(true)
            .layout_revision(1)
            .patch(&BASE_PATCH)
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
        let viewport = Self::viewport().layout_revision(self.revision);
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
        let viewport = Self::viewport().layout_revision(self.revision);
        match self.mode {
            SourceMode::Plain | SourceMode::Spans => {
                let lines = self.borrowed_lines();
                let _ = viewport.draw(ui, AREA, &self.state, &lines);
            }
            SourceMode::ProjectedPlain => {
                let _ = viewport.draw_projected(ui, AREA, &self.state, &self.plain);
            }
            SourceMode::ProjectedSpans => {
                let _ = viewport.draw_projected(ui, AREA, &self.state, &self.spans);
            }
            SourceMode::Rich => {
                let _ = viewport.draw_projected(ui, AREA, &self.state, &self.rich);
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
    lines.push_styled_line([
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
    ]);
    for line in rows.iter().copied().skip(1) {
        lines.push_line([(line, None, Modifier::empty())]);
    }
    lines
}

fn harness(mode: SourceMode, theme: Theme) -> Harness<ProjectedPage> {
    Harness::new(ProjectedPage::new(mode, &theme), theme, SCREEN.0, SCREEN.1)
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

    let _ = baseline.resize(46, 15);
    let _ = rich.resize(46, 15);
    assert_same_geometry_and_symbols(&baseline, &rich);

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
