




use junie_tui::runtime::Application;
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
        app.open_page(page);
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
        let i = self.app.page().index();
        self.app.pages[i]
            .template_mut()
            .expect("a templated page")
            .set_section(section);
        self.draw();
    }


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

    assert!(h.text().contains("Size Column to Fit"));
    h.opt("columns.resizable=off");
    assert!(!columns_page(&h).sized.sizing.resizable);
    let text = h.text();
    assert!(!text.contains("Size Column to Fit"), "{text}");
    assert!(text.contains("fixed widths"), "{text}");

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
    assert_eq!(
        h.app.shell.focus.current(),
        Some(toggle),
        "the click focuses it"
    );
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

    let id = tables_page(&h).table.id;
    h.app.shell.focus.focus(id);
    h.key(KeyCode::Char(' '));
    assert_eq!(tables_page(&h).table.selected_rows(), vec![0]);
    assert!(line_with(&h.text(), "#1040").contains("[✓] #1040"));
    h.opt("basictable.selection=none");
    let id = tables_page(&h).table.id;
    h.app.shell.focus.focus(id);
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

fn grid_page(h: &Harness) -> &crate::pages::grid::GridPage {
    h.app.pages[PageId::Grid.index()]
        .as_any()
        .and_then(|a| a.downcast_ref())
        .expect("the Data grid page")
}


fn grid_cell(h: &mut Harness, col: usize) {
    let id = grid_page(h).grid.id;
    h.app.shell.focus.focus(id);
    h.draw();
    h.key(KeyCode::Home);
    for _ in 0..col {
        h.key(KeyCode::Right);
    }
}

fn status(h: &Harness) -> String {
    format!("{:?}", h.app.shell.status)
}

#[test]
fn grid_access_option_makes_the_grid_read_only_and_says_why() {
    let mut h = Harness::new(160, 50, PageId::Grid);
    assert!(grid_page(&h).grid.editable);
    h.opt("datagrid.access=view");
    assert!(!grid_page(&h).grid.editable);
    assert!(h.text().contains("read-only · rows"), "{}", h.text());
    grid_cell(&mut h, 1);
    h.key(KeyCode::Enter);
    assert!(
        status(&h).contains("customer_summary is a view"),
        "{}",
        status(&h)
    );
    assert!(!grid_page(&h).grid.is_editing());
    h.opt("datagrid.access=no-key");
    grid_cell(&mut h, 1);
    h.key(KeyCode::Enter);
    assert!(status(&h).contains("no primary key"), "{}", status(&h));
}

#[test]
fn grid_row_total_option_reaches_the_position_label() {
    let mut h = Harness::new(160, 50, PageId::Grid);
    assert!(h.text().contains("~4,812 total"), "estimated by default");
    h.opt("datagrid.total=exact");
    let text = h.text();
    assert!(
        text.contains("4,812 total") && !text.contains("~4,812"),
        "{text}"
    );
    h.opt("datagrid.total=unknown");
    assert!(!h.text().contains("4,812"), "{}", h.text());
}

#[test]
fn grid_filter_on_cell_adds_a_chip_and_clear_filters_removes_it() {
    let mut h = Harness::new(160, 50, PageId::Grid);
    grid_cell(&mut h, 2);
    h.key(KeyCode::Char('f'));
    assert!(h.text().contains("plan = enterprise"), "{}", h.text());
    assert!(grid_page(&h).grid.filtered_cols.contains(&2));
    h.key(KeyCode::Char('F'));
    assert!(!h.text().contains("plan = enterprise"));
    assert!(grid_page(&h).grid.filtered_cols.is_empty());
}

#[test]
fn grid_save_review_asks_for_the_table_name_before_it_saves() {
    let mut h = Harness::new(160, 50, PageId::Grid);

    grid_cell(&mut h, 6);
    h.key(KeyCode::Delete);
    assert_eq!(grid_page(&h).grid.pending.total(), 1);
    h.key_mod(KeyCode::Char('s'), KeyModifiers::CONTROL);
    assert!(h.app.dialog().is_some(), "Ctrl+S opens the save review");
    assert!(h.text().contains("Save changes"), "{}", h.text());

    h.key(KeyCode::Enter);
    assert_eq!(grid_page(&h).grid.pending.total(), 1);
    for c in "customers".chars() {
        h.key(KeyCode::Char(c));
    }
    h.key(KeyCode::Enter);
    assert!(h.app.dialog().is_none(), "{}", h.text());
    for _ in 0..6 {
        h.app.handle(Input::Tick);
    }
    assert!(grid_page(&h).grid.pending.is_empty());
    assert!(status(&h).contains("Saved 1 changes"), "{}", status(&h));
}

#[test]
fn grid_keymap_option_rebinds_commands_and_the_reference_follows() {
    let mut h = Harness::new(160, 50, PageId::Grid);
    grid_cell(&mut h, 6);
    h.key(KeyCode::Char('x'));
    assert!(
        grid_page(&h).grid.pending.is_empty(),
        "x is not bound by default"
    );
    h.opt("datagrid.keymap=custom");
    grid_cell(&mut h, 6);
    h.key(KeyCode::Char('x'));
    assert_eq!(grid_page(&h).grid.pending.total(), 1, "x clears the cell");

    let mut h = Harness::new(220, 80, PageId::Grid);
    h.opt("datagrid.keymap=custom");
    let text = h.text();
    assert!(
        text.lines()
            .any(|l| l.contains(" x ") && l.contains("Set NULL · CellChanged")),
        "{text}"
    );
}

#[test]
fn grid_follows_a_reference_and_draws_literals_in_their_hues() {
    let mut h = Harness::new(160, 50, PageId::Grid);
    grid_cell(&mut h, 7);
    h.key_mod(KeyCode::Char(']'), KeyModifiers::CONTROL);
    assert!(
        status(&h).contains("Follow account_id = 201 into accounts"),
        "{}",
        status(&h)
    );

    let t = Theme::junie();
    let buf = h.term.backend().buffer().clone();
    let cell = |needle: &str| {
        let (x, y) = h.find(needle).unwrap_or_else(|| panic!("{needle}"));
        buf[(x, y)].clone()
    };
    assert_eq!(cell("NULL").fg, t.syntax_literal);
    assert_eq!(cell("false").fg, t.syntax_literal);
    assert_eq!(cell("{\"owner\"").fg, t.syntax_string);
}

#[test]
fn grid_states_draw_every_row() {
    use junie_tui::ui::specimen::SpecimenState::*;
    let mut h = Harness::new(220, 80, PageId::Grid);
    h.section(Section::States);

    let mut seen = Vec::new();
    for _ in 0..12 {
        for s in h.app.pages[PageId::Grid.index()]
            .template()
            .unwrap()
            .state_rows()
        {
            if !seen.contains(s) {
                seen.push(*s);
            }
        }

        h.mouse(MouseKind::WheelDown, 120, 40);
    }
    for s in [
        Rest,
        Hover,
        Pressed,
        Focused,
        FocusedHover,
        Selected,
        Editing,
        Error,
        Empty,
        Loading,
        Overflow,
        Narrow,
    ] {
        assert!(seen.contains(&s), "{s:?} in {seen:?}");
    }
}

fn editable_page(h: &Harness) -> &crate::pages::editable::EditablePage {
    h.app.pages[PageId::Editable.index()]
        .as_any()
        .and_then(|a| a.downcast_ref())
        .expect("the Editable page")
}

fn cell_editors_page(h: &Harness) -> &crate::pages::celleditors::CellEditorsPage {
    h.app.pages[PageId::CellEditors.index()]
        .as_any()
        .and_then(|a| a.downcast_ref())
        .expect("the Cell editors page")
}


fn focus(h: &mut Harness, id: junie_tui::core::id::WidgetId) {
    h.app.shell.focus.focus(id);
    h.draw();
}

#[test]
fn editable_keymap_option_rebinds_sort_and_the_hint_follows() {
    let mut h = Harness::new(160, 50, PageId::Editable);
    let id = editable_page(&h).table.id;
    focus(&mut h, id);
    assert!(h.text().contains("s Sort"), "{}", h.text());
    h.opt("editable.keymap=custom");
    let id = editable_page(&h).table.id;
    focus(&mut h, id);
    h.key(KeyCode::Char('s'));
    assert_eq!(editable_page(&h).table.sort, None, "s no longer sorts");
    h.key(KeyCode::Char('o'));
    assert!(editable_page(&h).table.sort.is_some(), "o sorts");
    assert!(h.text().contains("o Sort"), "{}", h.text());
}

#[test]
fn editable_rows_and_type_ahead_options_rebuild_the_table() {
    let mut h = Harness::new(160, 50, PageId::Editable);
    h.opt("editable.rows=empty");
    assert!(h.text().contains("No tasks yet"), "{}", h.text());
    h.opt("editable.rows=tasks");
    h.opt("editable.type_ahead=task");
    let id = editable_page(&h).table.id;
    focus(&mut h, id);
    h.key(KeyCode::Char('H'));
    let t = &editable_page(&h).table;
    let src = t.source_row(t.cursor_row);
    assert_eq!(t.rows[src][1].text, "Harden CSP headers");
}

#[test]
fn selection_options_reach_the_members_table() {
    let mut h = Harness::new(160, 50, PageId::Selectable);
    let page = |h: &Harness| -> junie_tui::widgets::table::DataTable {
        let p: &crate::pages::selectable::SelectablePage = h.app.pages[PageId::Selectable.index()]
            .as_any()
            .and_then(|a| a.downcast_ref())
            .expect("the Selection page");
        p.table.clone()
    };
    assert!(page(&h).esc_clears && page(&h).is_disabled(0));
    h.opt("selection.esc_clears=off");
    h.opt("selection.disabled_rows=none");
    h.opt("selection.toggle_mods=alt");
    let t = page(&h);
    assert!(!t.esc_clears && !t.is_disabled(0));
    assert_eq!(t.toggle_mods, KeyModifiers::ALT);

    focus(&mut h, t.id);
    h.key(KeyCode::Char(' '));
    assert_eq!(page(&h).selected_rows(), vec![0], "the owner row selects");
    h.key(KeyCode::Esc);
    assert_eq!(page(&h).selected_rows(), vec![0], "Esc does not clear it");
    h.opt("selection.check_column=off");
    assert!(!page(&h).selection_column);
}

#[test]
fn a_standalone_choice_editor_commits_cancels_and_tabs_on() {
    let mut h = Harness::new(220, 80, PageId::CellEditors);
    let pick = cell_editors_page(&h).pick.id;
    focus(&mut h, pick);
    h.key(KeyCode::Enter);
    assert!(cell_editors_page(&h).chooser.is_some(), "Enter opens it");
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    let p = cell_editors_page(&h);
    assert!(p.chooser.is_none());
    assert_eq!(p.region, "eu-west-2", "the next region");
    assert!(h.text().contains("last: Commit"), "{}", h.text());
    h.key(KeyCode::Enter);
    h.key(KeyCode::Esc);
    assert_eq!(cell_editors_page(&h).region, "eu-west-2");
    assert!(h.text().contains("last: Cancel"));
    h.key(KeyCode::Enter);
    h.key(KeyCode::Down);
    h.key(KeyCode::Tab);
    assert_eq!(cell_editors_page(&h).region, "eu-west-3");
    assert_ne!(h.app.shell.focus.current(), Some(pick), "Tab moves on");

    focus(&mut h, pick);
    h.key(KeyCode::Enter);
    let option = cell_editors_page(&h)
        .chooser
        .as_ref()
        .expect("the open list")
        .option_id(6);
    let r = h
        .app
        .shell
        .hits
        .area_of(option)
        .unwrap_or_else(|| panic!("eu-west-1 is on screen\n{}", h.text()));
    h.click(r.x + 1, r.y);
    assert_eq!(cell_editors_page(&h).region, "eu-west-1");
}

#[test]
fn cell_editor_options_set_the_editor_config() {
    let mut h = Harness::new(160, 50, PageId::CellEditors);
    h.opt("celleditors.affordance=always");
    h.opt("celleditors.max_rows=4");
    h.opt("celleditors.placement=above");
    h.opt("celleditors.tag_separator=dot");
    let cfg = &cell_editors_page(&h).table.editor_config;
    assert_eq!(
        cfg.affordance,
        junie_tui::widgets::option_editor::Affordance::Always
    );
    assert_eq!(cfg.max_rows, 4);
    assert_eq!(cfg.placement, junie_tui::ui::popup::Placement::Above);

    let text = h.text();
    assert!(text.contains("api · web"), "{text}");
    assert!(
        line_with(&text, "billing-worker").matches('▾').count() >= 3,
        "{text}"
    );
}

#[test]
fn sc13_states_grids_draw_their_rows() {
    use junie_tui::ui::specimen::SpecimenState::*;
    for (page, rows) in [
        (
            PageId::Editable,
            vec![
                Rest, Hover, Pressed, Focused, Editing, Error, Disabled, Empty, Overflow, Narrow,
            ],
        ),
        (
            PageId::Selectable,
            vec![
                Rest,
                Hover,
                Pressed,
                Focused,
                FocusedHover,
                Selected,
                Disabled,
                Empty,
                Narrow,
            ],
        ),
        (
            PageId::CellEditors,
            vec![
                Rest, Hover, Focused, Editing, Error, Disabled, Empty, Overflow, Narrow,
            ],
        ),
    ] {
        let mut h = Harness::new(220, 80, page);
        h.section(Section::States);
        let mut seen = Vec::new();
        for _ in 0..12 {
            for s in h.app.pages[page.index()].template().unwrap().state_rows() {
                if !seen.contains(s) {
                    seen.push(*s);
                }
            }
            h.mouse(MouseKind::WheelDown, 120, 40);
        }
        for s in rows {
            assert!(seen.contains(&s), "{page:?}: {s:?} in {seen:?}");
        }
    }
}
