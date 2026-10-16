//! Mouse-only navigation in the showcase (docs/audits/design-review-2026-10/mouse.md):
//! quitting and leaving every dialog work by pointer alone.

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Position;

use crate::app::{App, HEADER_BRAND, PageId};
use junie_tui::core::event::{Input, Key, Mouse, MouseKind};
use junie_tui::theme::Theme;
use junie_tui::ui::modal::CLOSE;

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
        self.app.handle(Input::Key(Key {
            code,
            mods: KeyModifiers::NONE,
        }));
        self.draw();
    }

    fn click(&mut self, x: u16, y: u16) {
        for kind in [MouseKind::Down, MouseKind::Up] {
            self.app
                .handle(Input::Mouse(Mouse::new(kind, Position::new(x, y))));
            self.draw();
        }
    }

    fn text(&self) -> String {
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

    fn area_of(&self, id: junie_tui::core::id::WidgetId) -> ratatui::layout::Rect {
        self.app
            .hits
            .area_of(id)
            .unwrap_or_else(|| panic!("{id:?} not on screen:\n{}", self.text()))
    }
}

// mouse #40: the brand is the pointer's way to quit
#[test]
fn the_brand_asks_and_quits_by_mouse() {
    let mut h = Harness::new(120, 40, PageId::Overview);
    let brand = h.area_of(HEADER_BRAND);
    h.click(brand.x, brand.y);
    assert!(h.text().contains("Quit the showcase?"), "{}", h.text());
    assert!(!h.app.quit);
    let ok = h.area_of(crate::app::QUIT_DIALOG.sub("ok"));
    let (x, y) = (ok.x + 1, ok.y);
    h.click(x, y);
    assert!(h.app.quit, "{}", h.text());
}

// mouse #13: the help dialog closes by the × in its frame
#[test]
fn the_help_dialog_closes_by_its_close_button() {
    let mut h = Harness::new(120, 40, PageId::Overview);
    h.key(KeyCode::Char('?'));
    assert!(h.app.dialog.is_some());
    let x = h.area_of(CLOSE);
    assert!(h.text().lines().nth(x.y as usize).unwrap().contains('×'));
    h.click(x.x + 1, x.y);
    assert!(
        h.app.dialog.is_none(),
        "{:?} {}",
        h.app
            .dialog
            .as_ref()
            .map(|d| (d.id, d.title.clone(), d.result)),
        h.text()
    );
}
