




use junie_tui::runtime::Application;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Position;

use crate::app::{App, PageId};
use junie_tui::core::event::{Input, Key, Mouse, MouseKind};
use junie_tui::core::id::WidgetId;
use junie_tui::theme::Theme;
use junie_tui::widgets::scrollbar;

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
        self.app.handle(Input::Key(Key {
            code,
            mods: KeyModifiers::NONE,
        }));
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
        (0..buf.area.width)
            .map(|x| buf[(x, y)].symbol().to_owned())
            .collect()
    }

    fn text(&self) -> String {
        let buf = self.term.backend().buffer();
        (0..buf.area.height)
            .map(|y| self.row(y))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn area_of(&self, id: WidgetId) -> ratatui::layout::Rect {
        self.app
            .shell
            .hits
            .area_of(id)
            .unwrap_or_else(|| panic!("{id:?} not on screen:\n{}", self.text()))
    }

    fn flashing(&self) -> Option<WidgetId> {
        self.app.shell.pointer.flashing(self.app.clock.now_ms())
    }
}




#[test]
fn a_press_focuses_the_control_under_it_before_the_release() {
    let mut h = Harness::new(120, 40, PageId::Buttons);
    let id = WidgetId::of("buttons").child(0);
    let area = h.area_of(id);
    assert_ne!(h.app.shell.focus.current(), Some(id));
    h.mouse(MouseKind::Down, area.x + 1, area.y);
    assert_eq!(h.app.shell.focus.current(), Some(id), "the press focuses");
    h.mouse(MouseKind::Up, area.x + 1, area.y);
    assert_eq!(h.app.shell.focus.current(), Some(id));

    let mut h = Harness::new(120, 40, PageId::Buttons);
    h.app
        .apply_opt(crate::options::parse_opt("shell.focus_on=release").unwrap());
    h.draw();
    h.mouse(MouseKind::Down, area.x + 1, area.y);
    assert_ne!(h.app.shell.focus.current(), Some(id), "release: not yet");
    h.mouse(MouseKind::Up, area.x + 1, area.y);
    assert_eq!(
        h.app.shell.focus.current(),
        Some(id),
        "release: the click focuses"
    );
}



#[test]
fn every_activated_target_flashes_header_sidebar_and_dialog_buttons_included() {

    let mut h = Harness::new(120, 40, PageId::Overview);
    let inspect = h.area_of(WidgetId::of("app.header.inspect"));
    h.click(inspect.x + 1, inspect.y);
    assert!(h.app.inspector, "the link acted");
    assert_eq!(
        h.flashing(),
        Some(WidgetId::of("app.header.inspect")),
        "the header link flashes"
    );


    let mut h = Harness::new(120, 40, PageId::Overview);

    let i = (0..h.app.nav.rows().len())
        .find(|&i| PageId::from_nav_path(&h.app.nav.rows()[i].path).is_none())
        .expect("a group row in the sidebar");
    let rows = h.app.nav.rows().len();
    let area = h.area_of(h.app.nav.row_id(i));
    let at = Position::new(area.x + 2, area.y);

    let target = h
        .app
        .shell
        .hits
        .hit(at)
        .expect("a target under the pointer");
    assert_eq!(
        h.app.shell.hits.part_of(target).map(|hit| hit.owner),
        Some(WidgetId::of("app.nav")),
        "a part of the sidebar tree"
    );
    h.click(at.x, at.y);
    assert!(h.app.nav.rows().len() > rows, "the row opened its group");
    assert_eq!(h.flashing(), Some(target), "the sidebar row flashes");


    let mut h = Harness::new(120, 40, PageId::Overview);
    let brand = h.area_of(crate::app::HEADER_BRAND);
    h.click(brand.x, brand.y);
    assert!(h.app.dialog().is_some(), "the quit confirmation is open");
    let cancel = crate::app::QUIT_DIALOG.sub("cancel");
    let area = h.area_of(cancel);
    h.click(area.x + 1, area.y);
    assert!(h.app.dialog().is_none(), "Cancel closed it");
    assert_eq!(h.flashing(), Some(cancel), "the dialog button flashes");
}


#[test]
fn a_click_on_a_footer_hint_runs_its_key() {
    let mut h = Harness::new(120, 40, PageId::Overview);
    h.key(KeyCode::Char('0'));
    let footer = h.term.backend().buffer().area.height - 1;
    let row = h.row(footer);
    let x = row.find("/ Filter").expect("the sidebar's filter hint") as u16;
    assert!(h.app.nav_filter.is_none());
    h.click(x, footer);
    assert!(
        h.app.nav_filter.is_some(),
        "the click ran `/`:\n{}",
        h.row(footer)
    );
}



#[test]
fn a_drag_on_the_keyboard_references_scrollbar_scrolls_it() {
    let mut h = Harness::new(72, 20, PageId::Overview);
    h.key(KeyCode::Char('?'));
    let help = h.app.help().expect("the reference is open");
    let bar = scrollbar::id_for(help.id);
    let track = h.area_of(bar);
    assert_eq!(help.scroll.v.offset, 0);

    h.mouse(MouseKind::Down, track.x, track.y);
    h.mouse(MouseKind::Drag, track.x, track.bottom() - 1);
    let offset = h.app.help().unwrap().scroll.v.offset;
    assert!(offset > 0, "the drag scrolled the reference:\n{}", h.text());
    h.mouse(MouseKind::Up, track.x, track.bottom() - 1);
    assert!(h.app.help().is_some(), "the drag is no outside click");
}



#[test]
fn a_click_outside_an_open_picker_closes_it() {
    let mut h = Harness::new(120, 40, PageId::Pickers);
    let quick = h.area_of(WidgetId::of("pickers").sub("quick"));
    h.click(quick.x + 1, quick.y);
    assert!(!h.app.layers.is_empty(), "open");
    h.click(1, 1);
    assert!(
        h.app.layers.is_empty(),
        "the outside click closed it:\n{}",
        h.text()
    );
}






#[test]
fn a_click_that_keeps_focus_on_purpose_is_not_overridden_on_release() {
    let mut h = Harness::new(120, 40, PageId::Editor);
    h.app
        .apply_opt(crate::options::parse_opt("shell.focus_on=release").unwrap());
    h.draw();
    let code = WidgetId::of("editor").sub("code");
    let find = WidgetId::of("editor").sub("find");
    let area = h.area_of(code);
    h.click(area.x + 2, area.y + 1);
    assert_eq!(
        h.app.shell.focus.current(),
        Some(code),
        "the editor focused"
    );
    assert!(!h.text().contains("type to find"), "no find yet");
    let button = h.area_of(find);
    h.click(button.x + 1, button.y);
    assert!(
        h.text().contains("type to find"),
        "the click opened find:\n{}",
        h.text()
    );
    assert_eq!(
        h.app.shell.focus.current(),
        Some(code),
        "Find hands focus to the editor, which already had it:\n{}",
        h.text()
    );
}
