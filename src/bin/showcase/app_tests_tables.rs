//! The Tables pages on the Playground | Options | States template (plan
//! §7, SC11–SC13), driven through the real app: options set through the
//! page's Options section and `--opt` rebuild the widget through its
//! builder, and the States grids draw their rows.

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Position;

use crate::app::{App, PageId};
use crate::options;
use crate::pages::template::Section;
use junie_tui::core::event::{Input, Key, Mouse, MouseKind};
use junie_tui::theme::Theme;

struct Harness {
    app: App,
    term: Terminal<TestBackend>,
}

impl Harness {
    fn new(w: u16, h: u16, page: PageId) -> Self {
        let mut app = App::new(Theme::junie());
        app.goto(page);
        let term = Terminal::new(TestBackend::new(w, h)).unwrap();
        let mut h = Self { app, term };
        h.draw();
        h
    }

    fn draw(&mut self) {
        self.term.draw(|f| self.app.render(f)).unwrap();
    }

    fn key(&mut self, code: KeyCode) {
        self.key_mod(code, KeyModifiers::NONE);
    }

    fn key_mod(&mut self, code: KeyCode, mods: KeyModifiers) {
        self.app.handle(Input::Key(Key { code, mods }));
        self.draw();
    }

    fn mouse(&mut self, kind: MouseKind, x: u16, y: u16) {
        self.app
            .handle(Input::Mouse(Mouse::new(kind, Position::new(x, y))));
        self.draw();
    }

    fn click(&mut self, x: u16, y: u16) {
        self.mouse(MouseKind::Down, x, y);
        self.mouse(MouseKind::Up, x, y);
    }

    fn row(&self, y: u16) -> String {
        let buf = self.term.backend().buffer();
        (0..buf.area.width).map(|x| buf[(x, y)].symbol()).collect()
    }

    fn text(&self) -> String {
        let buf = self.term.backend().buffer();
        (0..buf.area.height)
            .map(|y| self.row(y))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn find(&self, needle: &str) -> Option<(u16, u16)> {
        let buf = self.term.backend().buffer();
        (0..buf.area.height).find_map(|y| {
            let row = self.row(y);
            let i = row.find(needle)?;
            Some((row[..i].chars().count() as u16, y))
        })
    }

    fn section(&mut self, section: Section) {
        let i = self.app.page.index();
        self.app.pages[i]
            .template_mut()
            .expect("a templated page")
            .set_section(section);
        self.draw();
    }

    /// Set an option through `--opt`'s path.
    fn opt(&mut self, arg: &str) {
        let set = options::parse_opt(arg).unwrap_or_else(|e| panic!("{arg}: {e}"));
        self.app.apply_opt(set);
        self.draw();
    }
}

fn columns_page(h: &Harness) -> &crate::pages::columns::ColumnsPage {
    h.app.pages[PageId::Columns.index()]
        .as_any()
        .and_then(|a| a.downcast_ref())
        .expect("the Columns page")
}

fn tables_page(h: &Harness) -> &crate::pages::tables::TablesPage {
    h.app.pages[PageId::Tables.index()]
        .as_any()
        .and_then(|a| a.downcast_ref())
        .expect("the Basic table page")
}

/// The first rendered row that holds `needle`.
fn line_with(text: &str, needle: &str) -> String {
    text.lines()
        .find(|l| l.contains(needle))
        .unwrap_or_default()
        .to_owned()
}

#[test]
fn columns_width_option_drops_the_least_important_columns_first() {
    let mut h = Harness::new(160, 50, PageId::Columns);
    let text = h.text();
    assert!(!text.contains("dropped"), "a full row keeps every column");
    assert!(text.contains("seats renew"), "{text}");
    h.opt("columns.width=40");
    let text = h.text();
    // drop 0 (Notes) goes first, then drop 1 (Renews); Plan (drop 2) and
    // the never-dropped columns stay
    assert!(line_with(&text, "Notes  ").contains("dropped"), "{text}");
    assert!(line_with(&text, "Renews  ").contains("dropped"), "{text}");
    assert!(!line_with(&text, "Plan  ").contains("dropped"), "{text}");
    assert!(text.contains("Row at 40 cells"), "{text}");
    assert!(
        !text.contains("seats renew"),
        "the dropped column leaves the row"
    );
    h.opt("columns.width=28");
    let text = h.text();
    assert!(line_with(&text, "Plan  ").contains("dropped"), "{text}");
    assert!(!line_with(&text, "Account  ").contains("dropped"), "{text}");
}

#[test]
fn columns_sizing_options_rebuild_the_table_through_its_builders() {
    let mut h = Harness::new(160, 50, PageId::Columns);
    assert!(columns_page(&h).sized.sizing.resizable);
    assert_eq!(columns_page(&h).sized.sizing.seam_w, 2);
    h.opt("columns.seam_w=3");
    h.opt("columns.min_width=10");
    let s = &columns_page(&h).sized.sizing;
    assert_eq!((s.seam_w, s.min_width), (3, 10));
    // the header menu offers the sizing commands only while resizable
    assert!(h.text().contains("Size Column to Fit"));
    h.opt("columns.resizable=off");
    assert!(!columns_page(&h).sized.sizing.resizable);
    let text = h.text();
    assert!(!text.contains("Size Column to Fit"), "{text}");
    assert!(text.contains("fixed widths"), "{text}");
    // custom labels reach the menu
    h.opt("columns.resizable=on");
    h.opt("columns.header_labels=custom");
    let text = h.text();
    assert!(
        text.contains("Sort A to Z") && text.contains("Fit Width"),
        "{text}"
    );
}

#[test]
fn a_page_option_set_in_the_options_section_goes_through_the_shell() {
    let mut h = Harness::new(160, 50, PageId::Columns);
    h.section(Section::Options);
    let i = PageId::Columns.index();
    let toggle = h.app.pages[i]
        .template()
        .and_then(|t| t.option_control("columns.resizable"))
        .expect("the option's control");
    let (x, y) = h.find("Resizable columns").expect("the toggle");
    h.click(x, y);
    assert_eq!(h.app.focus.current(), Some(toggle), "the click focuses it");
    assert!(
        !columns_page(&h).sized.sizing.resizable,
        "the click sets it off"
    );
    let opt = options::find("columns.resizable").unwrap();
    assert_eq!(
        h.app.option_value(opt),
        0,
        "the shell reads the page's value"
    );
    assert!(h.text().contains("Resizable columns off"), "{}", h.text());
    h.key(KeyCode::Char(' '));
    assert!(
        columns_page(&h).sized.sizing.resizable,
        "Space sets it on again"
    );
}

#[test]
fn basic_table_selection_option_sets_the_selection_model() {
    use junie_tui::core::selection::SelectionMode;
    let mut h = Harness::new(160, 50, PageId::Tables);
    assert_eq!(tables_page(&h).table.selection_mode, SelectionMode::Single);
    h.opt("basictable.selection=multi");
    assert_eq!(tables_page(&h).table.selection_mode, SelectionMode::Multi);
    assert!(tables_page(&h).table.selection_column);
    assert!(line_with(&h.text(), "#1040").contains("[ ] #1040"));
    // Space marks the cursor row in a multi-select table
    let id = tables_page(&h).table.id;
    h.app.focus.focus(id);
    h.key(KeyCode::Char(' '));
    assert_eq!(tables_page(&h).table.selected_rows(), vec![0]);
    assert!(line_with(&h.text(), "#1040").contains("[✓] #1040"));
    h.opt("basictable.selection=none");
    let id = tables_page(&h).table.id;
    h.app.focus.focus(id);
    h.key(KeyCode::Enter);
    assert_eq!(
        tables_page(&h).table.selected,
        None,
        "no row stays selected"
    );
}

#[test]
fn reviewers_header_checkbox_reads_mixed_and_selects_all() {
    let mut h = Harness::new(160, 50, PageId::Tables);
    assert!(
        h.text().contains("2 selected · header mixed"),
        "{}",
        h.text()
    );
    let (x, y) = h.find("[–] Reviewer").expect("the mixed header checkbox");
    h.click(x + 1, y);
    assert_eq!(tables_page(&h).reviewers.selected_rows(), vec![0, 1, 2, 3]);
    assert!(h.text().contains("4 selected · header all"), "{}", h.text());
    assert!(h.find("[✓] Reviewer").is_some());
}

#[test]
fn tables_states_grids_draw_every_row() {
    use junie_tui::ui::specimen::SpecimenState::*;
    let cases = [
        (
            PageId::Tables,
            vec![
                Rest,
                Hover,
                Pressed,
                Focused,
                FocusedHover,
                Selected,
                Disabled,
                Empty,
                Overflow,
                Narrow,
            ],
        ),
        (
            PageId::Columns,
            vec![Rest, Hover, Focused, Selected, Overflow, Narrow],
        ),
    ];
    for (page, rows) in cases {
        let mut h = Harness::new(220, 80, page);
        h.section(Section::States);
        let shown = h.app.pages[page.index()]
            .template()
            .unwrap()
            .state_rows()
            .to_vec();
        assert_eq!(shown, rows, "{page:?}");
    }
}
