use junie_tui::runtime::Application;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Position;

use crate::app::{App, NAV_ENTRIES, NAV_GROUPS, PAGE_ALIASES, PageId};
use crate::pages::template::{FOCUS_LEGEND, Section};
use junie_tui::core::event::{Input, Key, Mouse, MouseKind};
use junie_tui::core::id::WidgetId;
use junie_tui::theme::{ColorLevel, Theme};
use junie_tui::ui::specimen::SpecimenState;

struct Harness {
    app: App,
    term: Terminal<TestBackend>,
}

impl Harness {
    fn new(w: u16, h: u16, page: PageId) -> Self {
        Self::with_theme(w, h, page, Theme::junie())
    }

    fn with_theme(w: u16, h: u16, page: PageId, theme: Theme) -> Self {
        let mut app = App::new(theme);
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

    fn click(&mut self, x: u16, y: u16) {
        for kind in [MouseKind::Down, MouseKind::Up] {
            self.app
                .handle(Input::Mouse(Mouse::new(kind, Position::new(x, y))));
        }
        self.draw();
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

    fn section(&self) -> Section {
        let page = &self.app.pages[self.app.page().index()];
        page.template().expect("a templated page").section()
    }

    fn focus_bars(&self) -> usize {
        let t = &self.app.theme;
        self.term
            .backend()
            .buffer()
            .content()
            .iter()
            .filter(|c| c.symbol() == t.glyphs.focus_bar)
            .count()
    }

    fn focus_strip(&mut self) {
        self.key(KeyCode::Tab);
        assert_eq!(
            self.app.shell.focus.current(),
            Some(self.strip_id()),
            "the strip is the page's first stop"
        );
    }

    fn strip_id(&self) -> WidgetId {
        let page = &self.app.pages[self.app.page().index()];
        page.template().unwrap().tabs.id
    }
}

fn wheel_down(h: &mut Harness, x: u16, y: u16) {
    h.app.handle(Input::Mouse(Mouse::new(
        MouseKind::WheelDown,
        Position::new(x, y),
    )));
    h.draw();
}

#[test]
fn the_strip_switches_sections_by_key_and_click() {
    let mut h = Harness::new(120, 40, PageId::Buttons);
    assert_eq!(h.section(), Section::Playground);
    assert!(h.text().contains("Run task"));
    h.focus_strip();
    h.key(KeyCode::Right);
    assert_eq!(h.section(), Section::Options);

    assert!(h.text().contains("Focus ring"));
    assert!(!h.text().contains("Run task"), "one section at a time");
    h.key(KeyCode::Char('3'));
    assert_eq!(h.section(), Section::States);
    assert!(h.text().contains("Compare levels"));
    let (x, y) = h.find("Playground").unwrap();
    h.click(x + 1, y);
    assert_eq!(h.section(), Section::Playground);
    assert_eq!(h.app.shell.focus.current(), Some(h.strip_id()));
    assert!(h.text().contains("Run task"));
}

#[test]
fn the_template_lays_out_wide_and_narrow() {
    let mut wide = Harness::new(120, 40, PageId::Inputs);
    wide.focus_strip();
    wide.key(KeyCode::Char('3'));
    let text = wide.text();
    assert!(text.contains("every state, drawn by the widget"), "{text}");
    assert!(text.contains(FOCUS_LEGEND), "{text}");
    let page = &wide.app.pages[PageId::Inputs.index()];
    let rows = page.template().unwrap().state_rows().to_vec();
    assert_eq!(rows.first(), Some(&SpecimenState::Rest));
    assert_eq!(rows.last(), Some(&SpecimenState::Narrow), "every row fits");

    let (_, strip_y) = wide.find("Playground").unwrap();
    let (_, toggle_y) = wide.find("Compare levels").unwrap();
    assert_eq!(toggle_y, strip_y + 3, "strip, rule, blank row, body");

    let mut narrow = Harness::new(90, 30, PageId::Inputs);
    narrow.focus_strip();
    narrow.key(KeyCode::Char('3'));
    let text = narrow.text();
    assert!(!text.contains("every state, drawn by the widget"), "{text}");
    assert!(!text.contains(FOCUS_LEGEND), "{text}");
    let rows = |h: &Harness| {
        h.app.pages[PageId::Inputs.index()]
            .template()
            .unwrap()
            .state_rows()
            .to_vec()
    };
    let first = rows(&narrow);
    assert!(first.contains(&SpecimenState::Rest));
    assert!(!first.contains(&SpecimenState::Narrow), "{text}");

    let (x, y) = narrow.find("Compare levels").unwrap();
    for _ in 0..12 {
        wheel_down(&mut narrow, x + 30, y + 4);
    }
    assert!(
        rows(&narrow).contains(&SpecimenState::Narrow),
        "{}",
        narrow.text()
    );
}

#[test]
fn the_states_grid_never_adds_a_live_focus_bar() {
    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        for page in [PageId::Buttons, PageId::Inputs] {
            let mut h = Harness::with_theme(120, 40, page, Theme::for_level(level));
            h.focus_strip();
            h.key(KeyCode::Char('3'));
            assert_eq!(h.focus_bars(), 1, "{level:?} {page:?}\n{}", h.text());
            let (sx, sy) = h.find("States").unwrap();
            assert_eq!(
                h.row(sy).chars().nth(sx as usize - 1),
                Some('▎'),
                "on the strip"
            );
            let (_, fy) = h.find("focused").unwrap();
            let row = h.row(fy) + &h.row(fy + 1);
            assert!(row.contains('▏'), "{level:?} {page:?}: depicted\n{row}");
        }
    }
}

#[test]
fn compare_levels_shows_every_colour_level() {
    let mut h = Harness::new(220, 80, PageId::Buttons);
    h.focus_strip();
    h.key(KeyCode::Char('3'));
    h.key(KeyCode::Tab);
    let compare = h.app.pages[PageId::Buttons.index()]
        .template()
        .unwrap()
        .compare
        .id;
    assert_eq!(h.app.shell.focus.current(), Some(compare));
    h.key(KeyCode::Enter);
    let text = h.text();
    for level in ["truecolor", "256 colors", "16 colors", "no color"] {
        assert!(text.contains(&format!("{level} ")), "{level}\n{text}");
    }
    let rows = h.app.pages[PageId::Buttons.index()]
        .template()
        .unwrap()
        .state_rows()
        .to_vec();
    assert!(
        rows.contains(&SpecimenState::Overflow),
        "every pane drew every row"
    );

    let mut h = Harness::new(120, 40, PageId::Buttons);
    h.focus_strip();
    h.key(KeyCode::Char('3'));
    h.key(KeyCode::Tab);
    h.key(KeyCode::Enter);
    assert!(h.text().contains("truecolor"));
    assert!(!h.text().contains("no color"), "{}", h.text());
    let (x, y) = h.find("Compare levels").unwrap();
    for _ in 0..4 {
        wheel_down(&mut h, x + 20, y + 4);
    }
    assert!(h.text().contains("no color"), "{}", h.text());
}

fn slug(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

const SLUGS: &str = include_str!("../../../tests/showcase_slugs.txt");

fn golden() -> Vec<(&'static str, &'static str)> {
    SLUGS
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| {
            let (name, label) = l.split_once(" | ").expect("`name | label`");
            (name.trim(), label.trim())
        })
        .collect()
}

#[test]
fn the_tree_keeps_its_rules_and_its_page_names_only_grow() {
    let label_of = |id: PageId| NAV_ENTRIES[id.index()].label;

    for e in NAV_ENTRIES {
        let depth = e.id.nav_path().len();
        assert!(depth <= 2, "{} is {depth} levels deep", e.label);
        if !e.group.is_empty() {
            assert!(
                NAV_GROUPS.contains(&e.group),
                "{}: group {}",
                e.label,
                e.group
            );
            assert_ne!(
                slug(e.label),
                slug(e.group),
                "{} repeats its group",
                e.label
            );
        }
    }
    let mut app = App::new(Theme::junie());
    app.nav_expand = crate::app::NavExpand::All;
    app.open_page(PageId::Overview);
    assert!(app.nav.rows().iter().all(|r| r.depth <= 1), "two levels");

    let mut slugs: Vec<(String, PageId)> = Vec::new();
    for e in NAV_ENTRIES {
        let s = slug(e.label);
        assert!(!s.is_empty(), "{}", e.label);
        if let Some((_, other)) = slugs.iter().find(|(x, _)| *x == s) {
            panic!("{} and {} share the slug {s:?}", e.label, label_of(*other));
        }
        assert_eq!(PageId::from_name(&s), Some(e.id), "{s:?} opens {}", e.label);
        slugs.push((s, e.id));
    }
    let mut aliases: Vec<&str> = Vec::new();
    for &(alias, id) in PAGE_ALIASES {
        assert_eq!(alias, slug(alias), "alias {alias:?} is already normalised");
        assert!(!aliases.contains(&alias), "alias {alias:?} twice");
        aliases.push(alias);
        if let Some((_, owner)) = slugs.iter().find(|(s, _)| s == alias) {
            assert_eq!(
                *owner,
                id,
                "alias {alias:?} of {} is the slug of {}",
                label_of(id),
                label_of(*owner)
            );
        }
        assert_eq!(PageId::from_name(alias), Some(id), "{alias:?}");
    }

    let golden = golden();
    for &(name, label) in &golden {
        let page = PageId::from_name(name)
            .unwrap_or_else(|| panic!("--page {name:?} no longer opens a page (was {label})"));
        assert_eq!(label_of(page), label, "--page {name:?} moved");
    }
    let missing: Vec<String> = slugs
        .iter()
        .map(|(s, id)| (s.as_str(), *id))
        .chain(PAGE_ALIASES.iter().copied())
        .filter(|(name, _)| !golden.iter().any(|(g, _)| g == name))
        .map(|(name, id)| format!("{name} | {}", label_of(id)))
        .collect();
    assert!(
        missing.is_empty(),
        "add these --page names to tests/showcase_slugs.txt:\n{}",
        missing.join("\n")
    );
}

#[test]
fn every_page_opens_by_its_slug() {
    for e in NAV_ENTRIES {
        let id = PageId::from_name(&slug(e.label)).expect("slug");
        let h = Harness::new(120, 40, id);
        assert_eq!(h.app.page(), e.id);
        let text = h.text();
        assert!(text.contains(e.label), "{}\n{text}", e.label);
        assert!(
            !text.contains(&format!("{} arrives with SC", e.label)),
            "{} is still the planned-page stub",
            e.label
        );
    }
}

#[test]
fn every_page_is_on_the_template() {
    for e in NAV_ENTRIES {
        let h = Harness::new(120, 40, e.id);
        let page = &h.app.pages[e.id.index()];
        let tpl = page
            .template()
            .unwrap_or_else(|| panic!("{} is not on the template", e.label));
        assert_eq!(
            tpl.section(),
            Section::Playground,
            "{} opens on its Playground",
            e.label
        );
        let text = h.text();
        for s in Section::ALL {
            assert!(
                text.contains(s.label()),
                "{}: no {} section in its strip\n{text}",
                e.label,
                s.label()
            );
        }
    }
}
