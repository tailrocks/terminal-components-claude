//! The widget-page template (Playground | Options | States) and the
//! States grid, driven through the real app (plan §7, SC2).

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
        let page = &self.app.pages[self.app.page.index()];
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

    /// Focus the page's section strip (Tab from the sidebar lands there).
    fn focus_strip(&mut self) {
        self.key(KeyCode::Tab);
        assert_eq!(
            self.app.focus.current(),
            Some(self.strip_id()),
            "the strip is the page's first stop"
        );
    }

    fn strip_id(&self) -> WidgetId {
        let page = &self.app.pages[self.app.page.index()];
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

/// Tab lands on the strip; ←/→, the digits and a click switch sections;
/// Options names the step that brings its controls.
#[test]
fn the_strip_switches_sections_by_key_and_click() {
    let mut h = Harness::new(120, 40, PageId::Buttons);
    assert_eq!(h.section(), Section::Playground);
    assert!(h.text().contains("Run task"));
    h.focus_strip();
    h.key(KeyCode::Right);
    assert_eq!(h.section(), Section::Options);
    assert!(h.text().contains("No live options yet"));
    assert!(h.text().contains("arrive with SC1"));
    assert!(!h.text().contains("Run task"), "one section at a time");
    h.key(KeyCode::Char('3'));
    assert_eq!(h.section(), Section::States);
    assert!(h.text().contains("Compare levels"));
    let (x, y) = h.find("Playground").unwrap();
    h.click(x + 1, y);
    assert_eq!(h.section(), Section::Playground);
    assert_eq!(h.app.focus.current(), Some(h.strip_id()));
    assert!(h.text().contains("Run task"));
}

/// Wide (≥ 100 columns): the strip carries the section's caption and the
/// States section its focus legend, and the grid has room for every row.
/// Narrow (< 100): both captions give way, the grid keeps its rows and
/// scrolls row by row.
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
    // the strip sits on the page's first two rows, the body below
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
    // the wheel over the body scrolls whole rows until the last one shows
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

/// A focused sample is depicted, never shown as a second live focus bar:
/// with the strip focused the screen has exactly one `▎`, at every colour
/// level.
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

/// "Compare levels" draws the grid at all four levels, each under its
/// name; on a page too short for all of them it scrolls pane by pane.
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
    assert_eq!(h.app.focus.current(), Some(compare));
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
    // a short page: one pane per stack child, the wheel brings the rest
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

// ------------------------------------------------------------- tree rules --

/// A label or name as `--page` compares it: letters and digits only, lower
/// case (`PageId::from_name`).
fn slug(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

const SLUGS: &str = include_str!("../../../tests/showcase_slugs.txt");

/// The golden `--page` names: `name | page label`, comments and blanks
/// skipped.
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

/// Plan §7, SCP7: the sidebar is at most two levels deep (Overview, then
/// groups of leaves); no group holds a leaf with its own name; every page
/// has one slug of its own that no other page's alias repeats; and the
/// `--page` names in `tests/showcase_slugs.txt` only ever grow: each still
/// opens the page it names, and every current slug and alias is listed.
#[test]
fn the_tree_keeps_its_rules_and_its_page_names_only_grow() {
    let label_of = |id: PageId| NAV_ENTRIES[id.index()].label;
    // two levels at most, every leaf under a known group
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
    app.goto(PageId::Overview);
    assert!(app.nav.rows().iter().all(|r| r.depth <= 1), "two levels");
    // one slug per page, never another page's alias
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
    // the golden list only grows
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

/// Every page, planned ones included, opens by its slug; a planned page
/// names the step that fills it.
#[test]
fn every_page_opens_by_its_slug() {
    let mut stubs = 0;
    for e in NAV_ENTRIES {
        let id = PageId::from_name(&slug(e.label)).expect("slug");
        let h = Harness::new(120, 40, id);
        assert_eq!(h.app.page, e.id);
        let text = h.text();
        assert!(text.contains(e.label), "{}\n{text}", e.label);
        if text.contains(&format!("{} arrives with SC", e.label)) {
            stubs += 1;
            // a stub is not a focus trap: Tab goes back to the sidebar
            let mut h = h;
            h.key(KeyCode::Tab);
            assert_eq!(h.focus_bars(), 1, "{}", e.label);
        }
    }
    assert_eq!(stubs, 29, "the planned pages of plan §7.2");
}
