



use junie_tui::runtime::Application;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Position;

use crate::app::{App, PageId};
use crate::options;
use crate::pages::tabs::TabsPage;
use crate::pages::template::Section;
use junie_tui::core::event::{Input, Key, Mouse, MouseKind};
use junie_tui::core::id::WidgetId;
use junie_tui::theme::Theme;
use junie_tui::ui::specimen::SpecimenState;


pub(super) struct Harness {
    pub app: App,
    pub term: Terminal<TestBackend>,
}

#[allow(dead_code)]
impl Harness {
    fn new(w: u16, h: u16) -> Self {
        Self::on(w, h, PageId::Tabs)
    }

    pub fn on(w: u16, h: u16, page: PageId) -> Self {
        let mut app = App::new(Theme::junie());
        app.open_page(page);
        let term = Terminal::new(TestBackend::new(w, h)).unwrap();
        let mut h = Self { app, term };
        h.draw();
        h
    }

    pub fn draw(&mut self) {
        self.term.draw(|f| self.app.render(f)).unwrap();
    }

    pub fn key(&mut self, code: KeyCode) {
        self.key_mod(code, KeyModifiers::NONE);
    }

    pub fn key_mod(&mut self, code: KeyCode, mods: KeyModifiers) {
        self.app.handle(Input::Key(Key { code, mods }));
        self.draw();
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

    pub fn opt(&mut self, kv: &str) {
        let set = options::parse_opt(kv).unwrap();
        self.app.apply_opt(set);
        self.draw();
    }

    pub fn row(&self, y: u16) -> String {
        let buf = self.term.backend().buffer();
        (0..buf.area.width).map(|x| buf[(x, y)].symbol()).collect()
    }

    pub fn text(&self) -> String {
        let buf = self.term.backend().buffer();
        (0..buf.area.height)
            .map(|y| self.row(y))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn find(&self, needle: &str) -> Option<(u16, u16)> {
        let buf = self.term.backend().buffer();
        (0..buf.area.height).find_map(|y| {
            let row = self.row(y);
            let i = row.find(needle)?;
            Some((row[..i].chars().count() as u16, y))
        })
    }

    fn page(&self) -> &TabsPage {
        self.app.pages[PageId::Tabs.index()]
            .as_any()
            .and_then(|a| a.downcast_ref::<TabsPage>())
            .expect("the Tabs page")
    }

    pub fn section(&mut self, s: Section) {
        let page = self.app.page();
        if let Some(t) = self.app.pages[page.index()].template_mut() {
            t.set_section(s);
        }
        self.draw();
    }

    pub fn focus(&mut self, id: WidgetId) {
        self.app.shell.focus.focus(id);
        self.draw();
    }

    pub fn bars(&self) -> usize {
        self.text().matches('▎').count()
    }
}



#[test]
fn tabs_dump() {
    if std::env::var("DUMP_PAGE").is_err() {
        return;
    }
    let page = std::env::var("DUMP_PAGE").unwrap();
    let page = PageId::from_name(&page).unwrap_or(PageId::Tabs);
    let mut h = Harness::on(120, 40, page);
    for s in Section::ALL {
        h.section(s);
        println!("{}\n{}", "=".repeat(120), h.text());
    }
}

fn labels(h: &Harness) -> Vec<String> {
    h.page()
        .editor
        .items
        .iter()
        .map(|t| t.label.clone())
        .collect()
}

#[test]
fn the_editor_strip_closes_opens_and_asks_before_closing_a_dirty_tab() {
    let mut h = Harness::new(120, 40);
    let strip = h.page().editor.id;
    h.focus(strip);
    let n = labels(&h).len();

    h.key(KeyCode::Char('x'));
    assert_eq!(labels(&h).len(), n, "a pinned tab does not close");

    h.key(KeyCode::Right);
    h.key(KeyCode::Char('x'));
    assert!(h.app.dialog().is_some(), "closing a dirty tab asks first");
    h.key(KeyCode::Esc);
    assert!(h.app.dialog().is_none());
    assert_eq!(labels(&h).len(), n, "Esc keeps the tab");
    assert!(h.page().last.contains("Kept orders.sql open"));

    h.key(KeyCode::Char('x'));
    let (x, y) = h.find("Discard").expect("the dialog's Discard button");
    h.click(x + 1, y);
    assert!(!labels(&h).contains(&"orders.sql".to_owned()));

    let (x, y) = h.find("Migrate").expect("the Migrate tab");
    let close = h
        .row(y)
        .chars()
        .skip(x as usize)
        .position(|c| c == '×')
        .expect("its ×") as u16
        + x;
    h.click(close, y);
    assert!(!labels(&h).contains(&"Migrate".to_owned()), "× closes");
    assert!(h.page().last.starts_with("Close("), "{}", h.page().last);

    h.focus(strip);
    h.key(KeyCode::Char('n'));
    assert_eq!(
        labels(&h).last().map(String::as_str),
        Some("untitled-1.sql")
    );
    let e = &h.page().editor;
    assert_eq!(e.active, e.items.len() - 1);
    let (x, y) = h.find(" + ").map(|(x, y)| (x + 1, y)).unwrap();
    h.click(x, y);
    assert_eq!(
        labels(&h).last().map(String::as_str),
        Some("untitled-2.sql")
    );
}

#[test]
fn a_middle_click_closes_the_tab_under_the_pointer_unless_turned_off() {
    let mut h = Harness::new(120, 40);
    let (x, y) = h.find("Import").expect("the Import tab");
    h.mouse(MouseKind::Middle, x, y);
    assert!(
        !labels(&h).contains(&"Import".to_owned()),
        "middle click closes"
    );
    h.opt("tabs.middle_click=off");
    let (x, y) = h.find("Migrate").expect("the Migrate tab");
    h.mouse(MouseKind::Middle, x, y);
    assert!(labels(&h).contains(&"Migrate".to_owned()), "turned off");
    assert!(
        !h.text().contains("middle-click closes"),
        "the readout follows"
    );
}

#[test]
fn the_overflowing_strip_scrolls_and_lists_its_hidden_tabs_in_a_menu() {
    let mut h = Harness::new(120, 40);
    let (before, after) = h.page().editor.hidden();
    assert_eq!(before, 0);
    assert!(after > 0, "eleven tabs overflow at 120 columns");
    let more = format!("+{after}");
    let (x, y) = h.find(&more).expect("the +N button");
    h.click(x, y);
    assert!(h.page().editor.menu_open(), "+N opens the hidden tabs");
    assert!(h.text().contains("events"), "the menu lists a hidden tab");
    let (x, y) = h.find("events").unwrap();
    h.click(x + 1, y);
    let e = &h.page().editor;
    assert_eq!(e.items[e.active].label, "events", "choosing activates it");
    assert!(!e.menu_open());
    assert!(e.hidden().0 > 0, "and scrolls it into view");

    let active = e.active;
    let first = e.first;
    let (x, y) = h.find("events").unwrap();
    h.mouse(MouseKind::WheelUp, x, y);
    assert_eq!(h.page().editor.active, active);
    assert!(h.page().editor.first < first, "the window moved");

    h.opt("tabs.count=few");
    assert_eq!(h.page().editor.hidden(), (0, 0));
    assert!(h.find(&more).is_none());
}

#[test]
fn options_rebuild_the_editor_strip_through_its_builders() {
    let mut h = Harness::new(120, 40);
    assert!(h.text().contains("x close · n new tab"));
    h.opt("tabs.keymap=ctrl");
    assert!(
        h.text().contains("Ctrl+W close · Ctrl+T new tab"),
        "{}",
        h.text()
    );
    let strip = h.page().editor.id;
    h.focus(strip);
    h.key(KeyCode::Right);
    let n = labels(&h).len();
    h.key(KeyCode::Char('x'));
    assert!(h.app.dialog().is_none(), "x no longer closes");
    h.key_mod(KeyCode::Char('t'), KeyModifiers::CONTROL);
    assert_eq!(labels(&h).len(), n + 1, "Ctrl+T opens a tab");
    h.opt("tabs.keymap=default");
    h.opt("tabs.close_keys=delete");
    assert!(h.text().contains("Delete close"));
    h.opt("tabs.allow_new=off");
    assert!(!h.text().contains("new tab"));
    h.opt("tabs.digit_jump=off");
    assert!(!h.text().contains("–9 jump"));

    let before = h.text().matches('×').count();
    h.opt("tabs.close_reveal=hover");
    assert!(h.text().matches('×').count() < before);

    let rule = |h: &Harness| {
        let e = &h.page().editor;
        let r = e.areas[e.active];
        h.term.backend().buffer()[(r.x + 1, r.y + 1)].fg
    };
    assert_eq!(rule(&h), Theme::junie().accent);
    h.opt("tabs.rule=neutral");
    assert_eq!(rule(&h), Theme::junie().border_strong);
}




#[test]
fn the_keys_readout_names_the_hidden_tabs_key_while_the_strip_has_a_menu() {
    let mut h = Harness::new(160, 40);
    assert!(h.text().contains("↓ hidden tabs"), "{}", h.text());
    let strip = h.page().editor.id;
    h.focus(strip);
    assert!(
        h.page().editor.hidden() != (0, 0),
        "the default strip overflows"
    );
    h.key(KeyCode::Down);
    assert!(h.page().editor.menu_open(), "↓ opens the hidden tabs");
    h.key(KeyCode::Esc);
    h.opt("tabs.overflow_menu=off");
    assert!(!h.text().contains("hidden tabs"), "{}", h.text());
}

#[test]
fn the_vim_nav_profile_adds_h_and_l_to_the_strip() {
    let mut h = Harness::new(120, 40);
    let strip = h.page().editor.id;
    h.focus(strip);
    h.key(KeyCode::Char('l'));
    assert_eq!(h.page().editor.active, 0, "desktop keys: l is not a move");
    assert!(!h.text().contains("h l move"));
    h.opt("tabs.nav=vim");
    h.focus(strip);
    h.key(KeyCode::Char('l'));
    assert_eq!(h.page().editor.active, 1, "vim: l moves right");
    assert!(h.text().contains("←→ h l move"));
}

#[test]
fn an_options_control_sets_the_page_option_through_the_app() {
    let mut h = Harness::new(120, 40);
    h.section(Section::Options);
    assert!(!h.page().editor.wrap);
    let (x, y) = h.find("Wrap off").expect("the Wrap toggle");
    h.click(x, y);
    assert!(
        h.page().editor.wrap,
        "the toggle rebuilt the strip with wrap"
    );
    assert_eq!(
        h.app.option_value(options::find("tabs.wrap").unwrap()),
        1,
        "through the app's option path"
    );
    assert!(h.text().contains("Wrap on"));
}

#[test]
fn nested_strips_jump_with_digits_and_follow_in_the_crumb() {
    let mut h = Harness::new(120, 40);
    assert!(h.text().contains("orders › Structure › Columns"));
    let sub = h.page().sub.id;
    h.focus(sub);
    h.key(KeyCode::Char('4'));
    assert!(h.text().contains("orders › Structure › Triggers"));
    assert_eq!(h.bars(), 1, "one focus indicator, on the focused strip");
    let outer = h.page().outer.id;
    h.focus(outer);
    h.key(KeyCode::Char('3'));
    assert!(h.text().contains("orders › DDL"));
    assert!(
        !h.text().contains("Triggers"),
        "the sub-tabs belong to Structure"
    );
}

#[test]
fn the_states_grid_draws_every_strip_state() {
    let mut h = Harness::new(120, 40);
    h.section(Section::States);
    let page = &h.app.pages[PageId::Tabs.index()];
    let rows = page.template().unwrap().state_rows().to_vec();
    use SpecimenState::*;
    for s in [
        Rest,
        Hover,
        Pressed,
        Focused,
        FocusedHover,
        Busy,
        Error,
        Overflow,
        Narrow,
    ] {
        assert!(rows.contains(&s), "{s:?} row drawn");
    }
    assert!(h.bars() <= 1, "samples depict focus, never a live ▎");
}
