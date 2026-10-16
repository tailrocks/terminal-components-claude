//! App shell pages (plan §7, SC5 and SC6), driven through the real app:
//! Breadcrumbs & rails, Help & keys, Quit & leave.

use junie_tui::runtime::Application;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use crate::app::{App, PageId};
use crate::options;
use crate::pages::template::Section;
use junie_tui::core::event::{Input, Key, Mouse, MouseKind};
use junie_tui::theme::Theme;
use junie_tui::ui::specimen::SpecimenState;
use ratatui::layout::Position;

pub(crate) struct Harness {
    pub app: App,
    term: Terminal<TestBackend>,
}

impl Harness {
    pub fn new(w: u16, h: u16, page: PageId) -> Self {
        let mut app = App::new(Theme::junie());
        app.goto(page);
        let term = Terminal::new(TestBackend::new(w, h)).unwrap();
        let mut h = Self { app, term };
        h.draw();
        h
    }

    pub fn draw(&mut self) {
        self.term.draw(|f| self.app.render(f)).unwrap();
    }

    pub fn opt(&mut self, arg: &str) {
        self.app.apply_opt(options::parse_opt(arg).unwrap());
        self.draw();
    }

    pub fn section(&mut self, s: Section) {
        let page = self.app.page;
        self.app.pages[page.index()]
            .template_mut()
            .unwrap()
            .set_section(s);
        self.draw();
    }

    pub fn key_mods(&mut self, code: KeyCode, mods: KeyModifiers) {
        self.app.handle(Input::Key(Key { code, mods }));
        self.draw();
    }

    pub fn key(&mut self, code: KeyCode) {
        self.key_mods(code, KeyModifiers::NONE);
    }

    pub fn text(&self) -> String {
        let buf = self.term.backend().buffer();
        let mut s = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                s.push_str(buf[(x, y)].symbol());
            }
            s.push('\n');
        }
        s
    }

    /// Where `needle` starts on screen.
    pub fn find(&self, needle: &str) -> Option<(u16, u16)> {
        let buf = self.term.backend().buffer();
        for y in 0..buf.area.height {
            let row: String = (0..buf.area.width).map(|x| buf[(x, y)].symbol()).collect();
            if let Some(i) = row.find(needle) {
                let x = row[..i].chars().count() as u16;
                return Some((x, y));
            }
        }
        None
    }

    /// The row holding `needle`, as text.
    pub fn row_of(&self, needle: &str) -> String {
        let (_, y) = self.find(needle).unwrap_or_else(|| panic!("{needle:?}"));
        let buf = self.term.backend().buffer();
        (0..buf.area.width).map(|x| buf[(x, y)].symbol()).collect()
    }

    pub fn mouse(&mut self, kind: MouseKind, x: u16, y: u16) {
        self.app
            .handle(Input::Mouse(Mouse::new(kind, Position::new(x, y))));
        self.draw();
    }

    pub fn click(&mut self, x: u16, y: u16) {
        self.mouse(MouseKind::Down, x, y);
        self.mouse(MouseKind::Up, x, y);
    }

    pub fn states(&self) -> Vec<SpecimenState> {
        self.app.pages[self.app.page.index()]
            .template()
            .unwrap()
            .state_rows()
            .to_vec()
    }

    pub fn live_bars(&self) -> usize {
        self.text().matches('▎').count()
    }
}

/// Print a page at 120×40 (`--no-capture`), for sketches.
#[test]
fn dump_shell_pages() {
    if std::env::var("SHOWCASE_DUMP").is_err() {
        return;
    }
    let page = std::env::var("SHOWCASE_DUMP").unwrap();
    let id = crate::app::PageId::from_name(&page).expect("page");
    let mut h = Harness::new(120, 40, id);
    if let Ok(opts) = std::env::var("SHOWCASE_OPTS") {
        for o in opts.split(',').filter(|o| !o.is_empty()) {
            h.opt(o);
        }
    }
    if let Ok(s) = std::env::var("SHOWCASE_SECTION") {
        h.section(match s.as_str() {
            "options" => Section::Options,
            "states" => Section::States,
            _ => Section::Playground,
        });
    }
    println!("{}", h.text());
}

// ------------------------------------------------------------ SC5 ----

mod breadcrumbs_rails {
    use super::*;
    use crate::pages::breadcrumbs_rails::LIST;

    fn page() -> Harness {
        Harness::new(120, 40, PageId::BreadcrumbsRails)
    }

    #[test]
    fn enter_descends_and_a_crumb_goes_back() {
        let mut h = page();
        let (x, y) = h.find("Projects").expect("the folder list");
        h.click(x, y);
        assert!(h.app.shell.focus.is(LIST));
        assert!(
            h.text().contains("Disk › Users › alex › Projects"),
            "{}",
            h.text()
        );
        // Enter opens the folder under the cursor
        let (x, y) = h
            .find("payments-plat")
            .unwrap_or_else(|| panic!("{}", h.text()));
        h.click(x, y);
        assert!(h.text().contains("› Projects › payments-platform"));
        // a click on an earlier level goes back to it
        let (x, y) = h.find("Users ›").expect("the trail");
        h.click(x, y);
        assert!(h.row_of("Disk ›").contains("Disk › Users"));
        assert!(!h.text().contains("› alex ›"));
        assert!(h.text().contains("Users (level 1)"));
        // Backspace walks up a level
        h.app.shell.focus.focus(LIST);
        h.key(KeyCode::Backspace);
        assert!(h.text().contains("Backspace → level 0"));
        assert!(h.find("Applications").is_some());
    }

    #[test]
    fn a_plain_trail_takes_no_click() {
        let mut h = page();
        h.opt("breadcrumbsrails.clickable=off");
        assert!(h.text().contains("clickable(false)"));
        let (x, y) = h.find("Disk ›").unwrap();
        h.click(x, y);
        assert!(h.text().contains("Last crumb    —") || h.row_of("Last crumb").contains('—'));
    }

    #[test]
    fn the_rail_collapses_and_expands_by_pointer_and_ctrl_b() {
        let mut h = page();
        // ‹ in the explorer's frame collapses it
        let (x, y) = h.find("‹").expect("the collapse button");
        h.click(x, y);
        assert!(h.row_of("Last RailAction").contains("Collapse"));
        assert!(h.find("─ Explorer").is_none(), "the frame is gone");
        // the rail: `›` over the name written down the column
        let (x, y) = h.find("›    Location").expect("the rail");
        let buf = h.term.backend().buffer().clone();
        assert_eq!(buf[(x, y + 2)].symbol(), "E");
        // a click anywhere on it opens the sidebar again and focuses the list
        h.click(x, y + 4);
        assert!(h.row_of("Last RailAction").contains("Expand"));
        assert!(h.app.shell.focus.is(LIST));
        h.key_mods(KeyCode::Char('b'), KeyModifiers::CONTROL);
        assert!(h.row_of("Last RailAction").contains("Collapse"));
        assert!(!h.app.shell.focus.is(LIST), "focus leaves the hidden list");
        h.key_mods(KeyCode::Char('b'), KeyModifiers::CONTROL);
        assert!(h.app.shell.focus.is(LIST));
    }

    #[test]
    fn without_a_rail_a_collapsed_sidebar_leaves_nothing() {
        let mut h = page();
        h.opt("breadcrumbsrails.collapsed=on");
        let with = h.text();
        h.opt("breadcrumbsrails.collapsed_rail=off");
        assert_ne!(with, h.text());
        assert!(
            h.row_of("collapsed_rail")
                .contains("off: only the keyboard")
        );
    }

    #[test]
    fn the_deep_trail_collapses_and_cuts_its_current_page() {
        let mut h = page();
        h.opt("breadcrumbsrails.width=32");
        let row = h.row_of("settlement");
        assert!(row.contains("… › "), "{row}");
        h.opt("breadcrumbsrails.width=20");
        let row = h.row_of("settlement");
        assert!(row.contains("v2.md"), "middle cut keeps the end: {row}");
    }

    #[test]
    fn states_rows_are_drawn() {
        let mut h = page();
        h.section(Section::States);
        let rows = h.states();
        for s in [
            SpecimenState::Rest,
            SpecimenState::Hover,
            SpecimenState::Pressed,
        ] {
            assert!(rows.contains(&s), "{s:?} in {rows:?}");
        }
        assert!(h.live_bars() <= 1);
    }
}

/// The header's trail is the library `Breadcrumb`: plain text, the page
/// bold, collapsing before the header links go.
#[test]
fn the_header_crumb_is_the_library_trail() {
    let h = Harness::new(120, 40, PageId::BreadcrumbsRails);
    let row = h.row_of("Design system");
    assert!(row.contains("App shell › Breadcrumbs & rails"), "{row}");
    let (x, y) = h.find("Breadcrumbs & rails").unwrap();
    let buf = h.term.backend().buffer();
    assert!(
        buf[(x, y)]
            .modifier
            .contains(ratatui::style::Modifier::BOLD),
        "the current page reads as the title"
    );
    // nothing to click: a plain trail registers no link
    let (gx, gy) = h.find("App shell ›").unwrap();
    let mut h = h;
    h.click(gx, gy);
    assert_eq!(h.app.page, PageId::BreadcrumbsRails);
    // narrow: the links stay, the trail gives way
    let h = Harness::new(72, 20, PageId::BreadcrumbsRails);
    let row = h.row_of("Design system");
    assert!(row.contains("? Help"), "{row}");
}

// ------------------------------------------------------------ SC6 ----

mod help_keys {
    use super::*;
    use crate::pages::help_keys::{HELP, OPEN};

    fn page() -> Harness {
        Harness::new(120, 40, PageId::HelpKeys)
    }

    fn open(h: &mut Harness) {
        let (x, y) = h.find("Open help overlay").expect("the button");
        h.click(x + 1, y);
    }

    /// The shell's `?` is the library HelpOverlay: sections in columns,
    /// Esc closes it and focus comes back.
    #[test]
    fn question_mark_opens_the_shell_reference() {
        let mut h = Harness::new(120, 40, PageId::Overview);
        let before = h.app.shell.focus.current();
        h.key(KeyCode::Char('?'));
        let help = h.app.help.as_ref().expect("the reference opened");
        assert_eq!(help.title, "Keyboard & mouse");
        let text = h.text();
        for s in ["Focus", "Pages", "Sidebar", "Mouse"] {
            assert!(text.contains(s), "{s}\n{text}");
        }
        // sections side by side: more than one column
        assert_eq!(
            h.find("Focus  ").map(|p| p.1),
            h.find("Pages  ").map(|p| p.1),
            "{text}"
        );
        // one focus indicator, in the topmost layer: on the Done button
        assert_eq!(h.live_bars(), 1);
        assert!(h.row_of("▎").contains("Done"), "{text}");
        h.key(KeyCode::Esc);
        assert!(h.app.help.is_none());
        assert_eq!(h.app.shell.focus.current(), before);
        // `?` closes it too (HelpCmd::Close)
        h.key(KeyCode::Char('?'));
        assert!(h.app.help.is_some());
        h.key(KeyCode::Char('?'));
        assert!(h.app.help.is_none());
    }

    /// One modal at a time: the reference refuses to open over a dialog.
    #[test]
    fn it_refuses_to_open_over_a_dialog() {
        let mut h = Harness::new(120, 40, PageId::Overview);
        h.app.show_help(junie_tui::widgets::help::HelpOverlay::new(
            HELP,
            "x",
            vec![],
        ));
        assert!(h.app.help.is_some());
        h.key(KeyCode::Esc);
        h.app.dialog = Some(junie_tui::widgets::dialog::Dialog::confirm(
            junie_tui::core::id::WidgetId::of("t.dialog"),
            "Title",
            "Body",
            "OK",
        ));
        h.app.show_help(junie_tui::widgets::help::HelpOverlay::new(
            HELP,
            "x",
            vec![],
        ));
        assert!(h.app.help.is_none());
    }

    #[test]
    fn a_click_outside_closes_it_under_light_dismiss_only() {
        let mut h = page();
        open(&mut h);
        assert_eq!(h.app.help.as_ref().map(|h| h.id), Some(HELP));
        h.click(1, 20);
        assert!(h.app.help.is_none(), "light dismiss");
        h.opt("helpkeys.dismiss=escape");
        open(&mut h);
        h.click(1, 20);
        assert!(h.app.help.is_some(), "Esc only: an outside click keeps it");
        h.key(KeyCode::Esc);
        assert!(h.app.help.is_none());
        assert!(h.app.shell.focus.is(OPEN), "focus back on the button");
    }

    #[test]
    fn options_rebuild_the_overlay_through_its_builders() {
        let mut h = page();
        h.opt("helpkeys.title=custom");
        h.opt("helpkeys.keymap=f1");
        h.opt("helpkeys.max_percent=60");
        assert!(h.row_of("HelpCmd::Close").contains("F1"));
        open(&mut h);
        assert!(h.text().contains("Editor keys"));
        h.key(KeyCode::Char('?'));
        assert!(h.app.help.is_some(), "? is not bound any more");
        h.key(KeyCode::F(1));
        assert!(h.app.help.is_none(), "F1 closes");
        // vim keys scroll the reference only with the vim map
        open(&mut h);
        let top = h.text();
        h.key(KeyCode::Char('j'));
        assert_eq!(top, h.text(), "j is not a scroll key by default");
        h.key(KeyCode::Esc);
        h.opt("helpkeys.nav=vim");
        open(&mut h);
        let top = h.text();
        h.key(KeyCode::Char('j'));
        assert_ne!(top, h.text(), "j scrolls under vim keys");
    }

    #[test]
    fn the_reference_follows_its_width_and_share() {
        let mut h = page();
        h.opt("helpkeys.width=28");
        let wide = h.text();
        assert!(wide.contains("KeyReference at width 28"));
        h.opt("helpkeys.key_max_share=25");
        assert_ne!(wide, h.text());
        h.opt("helpkeys.gap=6");
        assert!(h.text().contains("gap 6"));
    }

    #[test]
    fn states_rows_are_drawn() {
        let mut h = page();
        h.section(Section::States);
        let rows = h.states();
        for s in [
            SpecimenState::Rest,
            SpecimenState::Overflow,
            SpecimenState::Narrow,
        ] {
            assert!(rows.contains(&s), "{s:?} in {rows:?}");
        }
    }
}

mod quit_leave {
    use super::*;
    use crate::pages::quit_leave::{LEAVE, QUIT, TITLE};

    fn page() -> Harness {
        Harness::new(120, 40, PageId::QuitLeave)
    }

    fn press(h: &mut Harness, id: junie_tui::core::id::WidgetId) {
        h.app.shell.focus.focus(id);
        h.key(KeyCode::Enter);
    }

    #[test]
    fn a_clean_page_leaves_at_once() {
        let mut h = page();
        press(&mut h, QUIT);
        assert!(h.app.dialog.is_none());
        assert!(h.row_of("Last request").contains("Ok(Leave::Quit)"));
        assert!(!h.app.shell.quit, "the demo never quits the showcase");
    }

    #[test]
    fn unsaved_edits_ask_and_resolve() {
        let mut h = page();
        let (x, y) = h.find("Q3 settlement report").unwrap();
        h.click(x + 2, y);
        h.key(KeyCode::End);
        h.key(KeyCode::Char('!'));
        assert!(h.app.shell.focus.is(TITLE));
        assert!(h.row_of("Edits").contains("Unsaved"), "{}", h.text());
        press(&mut h, LEAVE);
        let d = h.app.dialog.as_ref().expect("the unsaved-changes dialog");
        assert_eq!(d.actions.len(), 3, "Cancel / Discard / Save");
        // Enter presses the focused Save
        h.key(KeyCode::Enter);
        assert!(
            h.row_of("Last answer")
                .contains("Resolved::Save(Leave::To(Ove"),
            "{}",
            h.text()
        );
        assert!(h.row_of("Edits").contains("Clean"));
        // without save: Cancel / Discard
        h.opt("quitleave.save=off");
        h.opt("quitleave.stakes=unsaved");
        press(&mut h, LEAVE);
        assert_eq!(h.app.dialog.as_ref().unwrap().actions.len(), 2);
        h.key(KeyCode::Esc);
        assert!(h.row_of("Last answer").contains("Resolved::Stay"));
    }

    #[test]
    fn running_work_asks_stay_or_stop() {
        let mut h = page();
        h.opt("quitleave.stakes=running");
        assert!(h.row_of("Stakes::max").contains("Running"));
        press(&mut h, QUIT);
        assert!(h.text().contains("Stay"), "{}", h.text());
        h.key(KeyCode::Esc);
        assert!(h.row_of("Last answer").contains("Resolved::Stay"));
    }

    /// The shell's own guard sees the page's stakes: `q` never leaves a
    /// dirty page, Ctrl+Q asks first.
    #[test]
    fn the_shell_quit_keys_see_the_page_stakes() {
        let mut h = page();
        h.opt("quitleave.stakes=unsaved");
        h.app.shell.focus.focus(LEAVE);
        h.key(KeyCode::Char('q'));
        assert!(!h.app.shell.quit);
        assert!(h.app.dialog.is_none());
        h.key_mods(KeyCode::Char('q'), KeyModifiers::CONTROL);
        assert!(h.app.dialog.is_some(), "Ctrl+Q asks first");
        assert!(!h.app.shell.quit);
    }

    #[test]
    fn the_classify_table_follows_the_quit_keys() {
        let mut h = page();
        let row = h.row_of("selection  ");
        assert!(row.contains("Copy"), "Ctrl+C copies a selection: {row}");
        let esc = |h: &Harness| h.row_of("clean  ").trim_end().ends_with('—');
        assert!(esc(&h), "Esc never quits by default");
        h.opt("quitleave.esc_quits=on");
        assert!(!esc(&h));
        h.opt("quitleave.letter=x");
        assert!(h.row_of("Ctrl+C  ").contains(" x "));
        h.opt("quitleave.ctrl_c=off");
        assert!(!h.row_of("selection  ").contains("Copy"));
    }

    #[test]
    fn states_rows_are_drawn() {
        let mut h = page();
        h.section(Section::States);
        let rows = h.states();
        for s in [
            SpecimenState::Rest,
            SpecimenState::Hover,
            SpecimenState::Pressed,
            SpecimenState::Disabled,
        ] {
            assert!(rows.contains(&s), "{s:?} in {rows:?}");
        }
    }
}
