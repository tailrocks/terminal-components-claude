//! The navigation sidebar as a tree driven through the real `App`: every
//! page sits under a group, groups open and close by keyboard and mouse,
//! the current page's group opens on navigation, type-ahead, the wheel,
//! `[`/`]` in tree order and stable `--page` names.

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Position;

use crate::app::{App, NAV_ENTRIES, NAV_GROUPS, NavExpand, PageId};
use junie_tui::core::event::{Input, Key, Mouse, MouseKind};
use junie_tui::core::id::WidgetId;
use junie_tui::theme::Theme;

const NAV: WidgetId = WidgetId::of("app.nav");

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

    fn mouse(&mut self, kind: MouseKind, x: u16, y: u16) {
        self.app
            .handle(Input::Mouse(Mouse::new(kind, Position::new(x, y))));
        self.draw();
    }

    fn click(&mut self, x: u16, y: u16) {
        self.mouse(MouseKind::Down, x, y);
        self.mouse(MouseKind::Up, x, y);
    }

    /// The sidebar's rows as drawn.
    fn sidebar(&self) -> Vec<String> {
        let side = self.app.sidebar_area();
        let buf = self.term.backend().buffer();
        (side.top()..side.bottom())
            .map(|y| {
                (side.left()..side.right().saturating_sub(1))
                    .map(|x| buf[(x, y)].symbol())
                    .collect::<String>()
                    .trim_end()
                    .to_owned()
            })
            .collect()
    }

    /// Screen position of the first sidebar row whose label is `label`.
    fn nav_row(&self, label: &str) -> Option<(u16, u16)> {
        let side = self.app.sidebar_area();
        self.sidebar()
            .iter()
            .position(|r| r.trim_start_matches(['▎', ' ', '›', '▸', '▾']).trim() == label)
            .map(|i| (side.x + 6, side.y + i as u16))
    }

    fn open_groups(&self) -> Vec<&'static str> {
        NAV_GROUPS
            .iter()
            .enumerate()
            .filter(|(g, _)| self.app.nav.expanded.contains(&vec![*g]))
            .map(|(_, g)| *g)
            .collect()
    }
}

#[test]
fn every_page_sits_under_a_group_and_the_tree_lists_them_in_page_order() {
    let h = Harness::new(120, 40, PageId::Overview);
    for e in NAV_ENTRIES {
        assert!(NAV_GROUPS.contains(&e.group), "{} has a group", e.label);
        let path = e.id.nav_path();
        assert_eq!(path.len(), 2, "{}: a leaf under a group", e.label);
        assert_eq!(PageId::from_nav_path(&path), Some(e.id));
    }
    // the groups appear in order and their pages follow each other, so
    // `[`/`]` (NAV_ENTRIES order) walk the tree top to bottom
    let order: Vec<&str> = NAV_ENTRIES.iter().map(|e| e.group).collect();
    let mut dedup = order.clone();
    dedup.dedup();
    assert_eq!(dedup, NAV_GROUPS, "pages are listed group by group");
    // the tables live together
    let tables: Vec<&str> = NAV_ENTRIES
        .iter()
        .filter(|e| e.group == "Tables")
        .map(|e| e.label)
        .collect();
    for want in ["Tables", "Editable tables", "Data grid", "Selectable table"] {
        assert!(tables.contains(&want), "{want} under Tables: {tables:?}");
    }
    // there are no top-level pages: every root row is a group
    let roots: Vec<_> = h.app.nav.rows().iter().filter(|r| r.depth == 0).collect();
    assert_eq!(roots.len(), NAV_GROUPS.len());
    assert!(roots.iter().all(|r| r.has_children));
}

#[test]
fn the_current_pages_group_is_open_by_default_and_follows_navigation() {
    let mut h = Harness::new(120, 40, PageId::Overview);
    assert_eq!(h.open_groups(), ["Foundations"], "{:#?}", h.sidebar());
    // navigation opens the new group and closes the one it opened itself
    h.app.goto(PageId::Grid);
    h.draw();
    assert_eq!(h.open_groups(), ["Tables"], "{:#?}", h.sidebar());
    assert!(h.nav_row("Data grid").is_some());
    // a group the user opens stays open
    h.key(KeyCode::Char('0'));
    let (x, y) = h.nav_row("Overlays").unwrap();
    h.click(x, y);
    assert!(h.open_groups().contains(&"Overlays"));
    h.app.goto(PageId::Buttons);
    h.draw();
    assert_eq!(h.open_groups(), ["Inputs & forms", "Overlays"]);
}

#[test]
fn the_expansion_policy_is_configurable() {
    let mut h = Harness::new(120, 40, PageId::Overview);
    h.app.set_nav_expand(NavExpand::All);
    h.draw();
    assert_eq!(h.open_groups().len(), NAV_GROUPS.len());
    h.app.set_nav_expand(NavExpand::Accordion);
    h.key(KeyCode::Char('0'));
    let (x, y) = h.nav_row("Overlays").unwrap();
    h.click(x, y);
    h.app.goto(PageId::Selectable);
    h.draw();
    assert_eq!(h.open_groups(), ["Tables"], "accordion: one group open");
}

#[test]
fn arrows_fold_groups_and_enter_opens_pages() {
    let mut h = Harness::new(120, 40, PageId::Overview);
    h.key(KeyCode::Char('0'));
    assert_eq!(h.app.focus.current(), Some(NAV));
    assert_eq!(
        h.app.nav.cursor_path(),
        Some(&[0, 0][..]),
        "on the current page"
    );
    // ← from a page goes to its group, ← again closes it
    h.key(KeyCode::Left);
    assert_eq!(h.app.nav.cursor_path(), Some(&[0][..]));
    h.key(KeyCode::Left);
    assert!(h.open_groups().is_empty(), "{:#?}", h.sidebar());
    assert!(h.nav_row("Overview").is_none());
    // → opens it again, Enter on a group toggles it
    h.key(KeyCode::Right);
    assert_eq!(h.open_groups(), ["Foundations"]);
    h.key(KeyCode::Enter);
    assert!(h.open_groups().is_empty());
    // down to "Tables", open it, step in, open "Data grid"
    let tables = NAV_GROUPS.iter().position(|g| *g == "Tables").unwrap();
    for _ in 0..tables {
        h.key(KeyCode::Down);
    }
    assert_eq!(h.app.nav.cursor_path(), Some(&[tables][..]));
    h.key(KeyCode::Right);
    h.key(KeyCode::Right);
    h.key(KeyCode::Down);
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    assert_eq!(h.app.page, PageId::Grid);
    assert_eq!(
        h.app.focus.current(),
        Some(NAV),
        "Enter keeps the sidebar focus"
    );
    // the footer names the keys of the row under the cursor
    let footer = |h: &Harness| {
        let buf = h.term.backend().buffer();
        let y = buf.area.height - 1;
        (0..buf.area.width)
            .map(|x| buf[(x, y)].symbol())
            .collect::<String>()
    };
    assert!(footer(&h).contains("Open"), "{}", footer(&h));
    h.key(KeyCode::Left);
    assert!(footer(&h).contains("Fold"), "{}", footer(&h));
}

#[test]
fn clicks_toggle_groups_and_open_pages() {
    let mut h = Harness::new(120, 40, PageId::Overview);
    // the disclosure of a closed group opens it
    let (_, y) = h.nav_row("Tables").unwrap();
    let side = h.app.sidebar_area();
    let disclosure = (side.x..side.right())
        .find(|&x| h.term.backend().buffer()[(x, y)].symbol() == "▸")
        .expect("closed disclosure");
    h.click(disclosure, y);
    assert!(h.open_groups().contains(&"Tables"));
    assert_eq!(h.app.page, PageId::Overview, "a group is not a page");
    // a page row opens the page
    let (x, y) = h.nav_row("Selectable table").unwrap();
    h.click(x, y);
    assert_eq!(h.app.page, PageId::Selectable);
    assert_eq!(h.app.focus.current(), Some(NAV));
    // the group row itself toggles too
    let (x, y) = h.nav_row("Foundations").unwrap();
    h.click(x, y);
    assert!(h.nav_row("Overview").is_some());
    h.click(x, y);
    assert!(h.nav_row("Overview").is_none());
}

#[test]
fn typing_a_name_jumps_in_the_sidebar_unless_the_page_claims_the_letter() {
    let mut h = Harness::new(120, 40, PageId::Overview);
    h.key(KeyCode::Char('0'));
    h.key(KeyCode::Char('t'));
    let at = h.app.nav.cursor_path().map(<[usize]>::to_vec);
    let tables = NAV_GROUPS.iter().position(|g| *g == "Tables").unwrap();
    assert_eq!(at, Some(vec![tables]), "t: the Tables group");
    h.key(KeyCode::Char('e'));
    let at = h.app.nav.cursor_path().map(<[usize]>::to_vec);
    let text = NAV_GROUPS.iter().position(|g| *g == "Text & code").unwrap();
    assert_eq!(at, Some(vec![text]), "te: Text & code");
    // the shell's own letters are never type-ahead
    h.key(KeyCode::Char('i'));
    assert!(h.app.inspector, "i toggles the inspector");
    assert_eq!(h.app.nav.cursor_path(), Some(&[text][..]));
}

#[test]
fn brackets_walk_pages_in_tree_order_across_groups() {
    let mut h = Harness::new(120, 40, PageId::Overview);
    let mut seen = vec![h.app.page];
    for _ in 0..NAV_ENTRIES.len() {
        h.key(KeyCode::Char(']'));
        seen.push(h.app.page);
        // the current page is always visible in the tree
        let label = NAV_ENTRIES[h.app.page.index()].label;
        assert!(h.nav_row(label).is_some(), "{label}: {:#?}", h.sidebar());
    }
    let order: Vec<PageId> = NAV_ENTRIES.iter().map(|e| e.id).collect();
    assert_eq!(&seen[..order.len()], &order[..]);
    assert_eq!(seen.last(), Some(&PageId::Overview), "wraps");
    h.key(KeyCode::Char('['));
    assert_eq!(h.app.page, NAV_ENTRIES.last().unwrap().id);
}

#[test]
fn the_wheel_scrolls_an_overflowing_tree_without_moving_the_cursor() {
    let mut h = Harness::new(72, 20, PageId::Overview);
    h.app.set_nav_expand(NavExpand::All);
    h.draw();
    assert!(h.app.nav.scroll.overflows());
    let side = h.app.sidebar_area();
    let cursor = h.app.nav.cursor;
    h.mouse(MouseKind::WheelDown, side.x + 2, side.y + 2);
    assert!(h.app.nav.scroll.offset > 0);
    assert_eq!(h.app.nav.cursor, cursor);
}

#[test]
fn page_names_and_aliases_still_resolve() {
    for e in NAV_ENTRIES {
        assert_eq!(PageId::from_name(e.label), Some(e.id), "{}", e.label);
    }
    for (name, id) in [
        ("tables", PageId::Tables),
        ("editable-tables", PageId::Editable),
        ("datagrid", PageId::Grid),
        ("selectable", PageId::Selectable),
        ("bulkedit", PageId::Selectable),
        ("selection", PageId::Selectable),
        ("chips-selects", PageId::Chips),
        ("taskrunner", PageId::TaskRunner),
    ] {
        assert_eq!(PageId::from_name(name), Some(id), "{name}");
    }
    assert_eq!(
        PageId::from_name("Foundations"),
        None,
        "groups are not pages"
    );
}

#[test]
fn the_current_page_marker_is_neutral_unless_the_sidebar_has_focus() {
    let mut h = Harness::new(120, 40, PageId::Lists);
    let t = h.app.theme;
    let marker = |h: &Harness| {
        let side = h.app.sidebar_area();
        let buf = h.term.backend().buffer();
        let cells: Vec<_> = (side.top()..side.bottom())
            .flat_map(|y| (side.left()..side.right()).map(move |x| (x, y)))
            .filter(|&(x, y)| buf[(x, y)].symbol() == "›")
            .map(|(x, y)| buf[(x, y)].fg)
            .collect();
        assert_eq!(cells.len(), 1, "one current-page marker");
        cells[0]
    };
    h.app.focus.set(Some(NAV));
    h.draw();
    assert_eq!(marker(&h), t.accent);
    h.key(KeyCode::Tab);
    assert_ne!(h.app.focus.current(), Some(NAV));
    assert_eq!(marker(&h), t.text_primary);
    // no focus bar in the sidebar while the page holds focus
    let side = h.app.sidebar_area();
    let buf = h.term.backend().buffer();
    assert!(
        (side.top()..side.bottom())
            .all(|y| !(buf[(side.x, y)].symbol() == "▎" && buf[(side.x, y)].fg == t.focus)),
        "{:#?}",
        h.sidebar()
    );
}
