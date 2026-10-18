



use junie_tui::runtime::Application;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Cell;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::{Position, Rect};

use crate::app::{App, PageId};
use crate::options;
use crate::pages::template::Section;
use junie_tui::core::event::{Input, Key, Mouse, MouseKind};
use junie_tui::core::id::WidgetId;
use junie_tui::theme::Theme;

pub(crate) struct Harness {
    pub app: App,
    pub term: Terminal<TestBackend>,
}

#[allow(dead_code)]
impl Harness {
    pub fn new(w: u16, h: u16, page: PageId) -> Self {
        let mut app = App::new(Theme::junie());
        app.goto(page);
        let term = Terminal::new(TestBackend::new(w, h)).unwrap();
        let mut h = Self { app, term };
        h.draw();
        h
    }

    pub fn opt(&mut self, arg: &str) {
        self.app
            .apply_opt(options::parse_opt(arg).unwrap_or_else(|e| panic!("{arg}: {e}")));
        self.draw();
    }

    pub fn section(&mut self, s: Section) {
        let p = self.app.page;
        self.app.pages[p.index()]
            .template_mut()
            .expect("templated page")
            .set_section(s);
        self.draw();
    }

    pub fn draw(&mut self) {
        self.term.draw(|f| self.app.render(f)).unwrap();
    }

    pub fn key_mod(&mut self, code: KeyCode, mods: KeyModifiers) {
        self.app.handle(Input::Key(Key { code, mods }));
        self.draw();
    }

    pub fn key(&mut self, code: KeyCode) {
        self.key_mod(code, KeyModifiers::NONE);
    }

    pub fn tick(&mut self) {
        self.app.handle(Input::Tick);
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

    pub fn area(&self, id: WidgetId) -> Rect {
        self.app
            .shell
            .hits
            .area_of(id)
            .unwrap_or_else(|| panic!("missing hit area for {id:?}\n{}", self.text()))
    }

    pub fn click_id(&mut self, id: WidgetId) {
        let a = self.area(id);
        self.click(a.x + a.width.saturating_sub(1) / 2, a.y);
    }

    pub fn focus(&mut self, id: WidgetId) {
        self.app.shell.focus.focus(id);
        self.draw();
    }

    pub fn cell(&self, x: u16, y: u16) -> &Cell {
        &self.term.backend().buffer()[(x, y)]
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
        let want: Vec<&str> =
            unicode_segmentation::UnicodeSegmentation::graphemes(needle, true).collect();
        for y in 0..buf.area.height {
            let cells: Vec<&str> = (0..buf.area.width).map(|x| buf[(x, y)].symbol()).collect();
            for x in 0..cells.len().saturating_sub(want.len() - 1) {
                if cells[x..x + want.len()] == want[..] {
                    return Some((x as u16, y));
                }
            }
        }
        None
    }

    pub fn at(&self, needle: &str) -> (u16, u16) {
        self.find(needle)
            .unwrap_or_else(|| panic!("{needle:?} not on screen:\n{}", self.text()))
    }

    pub fn status(&self) -> String {
        self.app.shell.status.text().unwrap_or_default().to_owned()
    }
}




#[test]
#[ignore]
fn dump_feedback_pages() {
    let pages = std::env::var("DUMP_PAGES").unwrap_or_else(|_| "progress".into());
    let size = std::env::var("DUMP_SIZE").unwrap_or_else(|_| "120x40".into());
    let (w, h) = size.split_once('x').unwrap();
    for name in pages.split(',') {
        let page = PageId::from_name(name).unwrap();
        let mut h = Harness::new(w.parse().unwrap(), h.parse().unwrap(), page);
        if let Ok(opts) = std::env::var("DUMP_OPTS") {
            for o in opts.split(',').filter(|o| !o.is_empty()) {
                h.opt(o);
            }
        }
        for s in Section::ALL {
            h.section(s);
            println!("==== {name} · {}\n{}", s.label(), h.text());
        }
    }
}




fn track_start(h: &Harness, label: &str) -> u16 {
    let t = Theme::junie();
    let (x, y) = h.at(label);
    (x + label.chars().count() as u16..h.term.backend().buffer().area.width)
        .find(|&cx| [t.glyphs.bar_fill, t.glyphs.bar_empty].contains(&h.cell(cx, y).symbol()))
        .expect("a track")
}


fn track_end(h: &Harness, label: &str) -> u16 {
    let t = Theme::junie();
    let (x, y) = h.at(label);
    (x..h.term.backend().buffer().area.width)
        .filter(|&cx| [t.glyphs.bar_fill, t.glyphs.bar_empty].contains(&h.cell(cx, y).symbol()))
        .max()
        .expect("a track")
}

#[test]
fn progress_label_width_lines_the_bars_up_or_measures_each_label() {
    let mut h = Harness::new(160, 50, PageId::Progress);
    assert_eq!(
        track_start(&h, "Build image"),
        track_start(&h, "Push to registry"),
        "a fixed label column lines the tracks up"
    );
    h.opt("progress.label_width=measured");
    assert!(track_start(&h, "Build image") < track_start(&h, "Push to registry"));
    h.opt("progress.label_width=10");

    assert_eq!(track_start(&h, "Build ima"), track_start(&h, "Push to r"));
}

#[test]
fn progress_reserve_value_ends_an_indeterminate_bar_with_the_others() {
    let mut h = Harness::new(160, 50, PageId::Progress);
    assert_eq!(track_end(&h, "Warm caches"), track_end(&h, "Build image"));
    h.opt("progress.reserve_value=off");
    assert!(track_end(&h, "Warm caches") > track_end(&h, "Build image"));
}

#[test]
fn progress_tally_tones_only_the_failed_clause() {
    let t = Theme::junie();
    let h = Harness::new(160, 50, PageId::Progress);
    let (x, y) = h.at("2 of 7 done · 3 running · 1 paused · 1 failed");
    assert_eq!(h.cell(x, y).fg, t.text_secondary);
    let (fx, _) = h.at("1 failed");
    assert_eq!(h.cell(fx, y).fg, t.error);
    let (rx, _) = h.at("3 running");
    assert_eq!(h.cell(rx, y).fg, t.text_secondary);
}

#[test]
fn progress_meter_options_switch_the_visual_and_the_stale_style() {
    let t = Theme::junie();
    let mut h = Harness::new(160, 50, PageId::Progress);
    let (_, y) = h.at("Stale     ");
    assert!(h.row(y).contains(t.glyphs.bar_stale));
    h.opt("progress.stale_style=faint");
    let (_, y) = h.at("Stale     ");
    assert!(!h.row(y).contains(t.glyphs.bar_stale), "{}", h.row(y));
    let (x, y) = h.at("Low       ");
    let left: String = (x..x + 40).map(|cx| h.cell(cx, y).symbol()).collect();
    assert!(left.contains(t.glyphs.bar_fill), "{left}");
    h.opt("progress.meter_visual=block");
    let left: String = (x..x + 40).map(|cx| h.cell(cx, y).symbol()).collect();
    assert!(!left.contains(t.glyphs.bar_fill), "{left}");
    let filled = (x + 11..x + 30).any(|cx| h.cell(cx, y).bg != h.cell(x, y).bg);
    assert!(filled, "the block visual fills: {}", h.row(y));
}

#[test]
fn progress_states_grid_shows_bars_and_meters_in_every_content_state() {
    let mut h = Harness::new(160, 50, PageId::Progress);
    h.section(Section::States);
    let text = h.text();
    for label in [
        "Bar active",
        "Bar done",
        "Meter stale",
        "Meter block",
        "rest",
        "empty",
        "loading",
        "error",
        "overflow",
        "narrow",
        "⠋ refreshing",
        "read failed !",
    ] {
        assert!(text.contains(label), "{label}: {text}");
    }
}



#[test]
fn empty_kind_hint_and_actions_follow_the_options() {
    let mut h = Harness::new(120, 40, PageId::EmptyError);
    let text = h.text();
    assert!(text.contains("No connections yet"));
    assert!(text.contains("A connection holds a host"));
    assert!(text.contains("New connection") && text.contains("Import…"));
    h.opt("emptyerror.kind=error");
    let (x, y) = h.at("! Could not load connections");
    assert_eq!(h.cell(x, y).fg, Theme::junie().error);
    assert!(h.text().contains("Retry"));
    h.opt("emptyerror.hint=off");
    assert!(!h.text().contains("unreadable"));
    h.opt("emptyerror.actions=one");
    assert!(!h.text().contains("Import…"));
    h.opt("emptyerror.actions=none");
    assert!(!h.text().contains("Retry"));
}

#[test]
fn empty_actions_answer_a_click_and_the_keyboard() {
    let mut h = Harness::new(120, 40, PageId::EmptyError);
    let (x, y) = h.at("New connection");
    h.click(x + 1, y);
    assert_eq!(h.status(), "New connection chosen");
    h.focus(WidgetId::of("emptyerror").sub("import"));
    h.key(KeyCode::Enter);
    assert_eq!(h.status(), "Import… chosen");
}

#[test]
fn read_only_grid_says_why_on_an_attempted_edit() {
    let mut h = Harness::new(120, 40, PageId::EmptyError);
    assert!(h.text().contains("orders_view · read-only"));
    h.focus(WidgetId::of("emptyerror").sub("grid"));
    h.key(KeyCode::Enter);
    assert!(
        h.status().contains("views cannot be edited"),
        "{}",
        h.status()
    );
    h.opt("emptyerror.reason=no-key");
    h.key(KeyCode::Enter);
    assert!(h.status().contains("no primary key"), "{}", h.status());
}

#[test]
fn read_only_row_actions_stay_listed_with_their_reason() {
    let mut h = Harness::new(120, 40, PageId::EmptyError);
    h.click_id(WidgetId::of("emptyerror").sub("actions"));
    let (_, y) = h.at("Edit cell");
    assert!(h.row(y).contains("read-only view"), "{}", h.row(y));

    h.key(KeyCode::Enter);
    assert_eq!(h.status(), "Copied row 1041");
    assert!(h.find("Edit cell").is_none(), "the menu closed");
}

#[test]
fn empty_states_grid_shows_the_action_states() {
    let mut h = Harness::new(120, 40, PageId::EmptyError);
    h.section(Section::States);
    let text = h.text();
    for label in ["Title + hint", "With actions", "rest", "hover", "pressed"] {
        assert!(text.contains(label), "{label}: {text}");
    }
    assert!(text.contains("! No rows"), "{text}");
}
