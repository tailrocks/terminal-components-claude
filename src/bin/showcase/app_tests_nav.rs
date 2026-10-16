//! The navigation sidebar as a tree driven through the real `App`: every
//! page sits under a group, groups open and close by keyboard and mouse,
//! the current page's group opens on navigation, type-ahead, the wheel,
//! `[`/`]` in tree order and stable `--page` names.

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Position;

use crate::app::{App, NAV_ENTRIES, NAV_GROUPS, NavExpand, NavRoot, PageId, nav_roots};
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
        nav_roots()
            .iter()
            .enumerate()
            .filter_map(|(r, root)| match root {
                NavRoot::Group(g) if self.app.nav.expanded.contains(&vec![r]) => Some(*g),
                _ => None,
            })
            .collect()
    }
}

/// Root index of a group in the sidebar tree.
fn root_of(group: &'static str) -> usize {
    nav_roots()
        .iter()
        .position(|r| *r == NavRoot::Group(group))
        .unwrap()
}

#[test]
fn every_page_but_overview_sits_under_a_group_two_levels_at_most() {
    let h = Harness::new(120, 40, PageId::Overview);
    for e in NAV_ENTRIES {
        let path = e.id.nav_path();
        if e.id == PageId::Overview {
            assert!(
                e.group.is_empty() && path.len() == 1,
                "Overview is top level"
            );
        } else {
            assert!(NAV_GROUPS.contains(&e.group), "{} has a group", e.label);
            assert_eq!(path.len(), 2, "{}: a leaf under a group", e.label);
        }
        assert_eq!(PageId::from_nav_path(&path), Some(e.id));
        // a group never holds a page with its own name
        assert_ne!(e.label, e.group, "{} repeats its group", e.label);
    }
    // groups appear in order and their pages follow each other, so
    // `[`/`]` (NAV_ENTRIES order) walk the tree top to bottom
    let mut order: Vec<&str> = NAV_ENTRIES
        .iter()
        .map(|e| e.group)
        .filter(|g| !g.is_empty())
        .collect();
    order.dedup();
    assert_eq!(order, NAV_GROUPS, "pages are listed group by group");
    let tables: Vec<&str> = NAV_ENTRIES
        .iter()
        .filter(|e| e.group == "Tables")
        .map(|e| e.label)
        .collect();
    for want in ["Basic table", "Editable", "Data grid", "Selection"] {
        assert!(tables.contains(&want), "{want} under Tables: {tables:?}");
    }
    assert!(NAV_GROUPS.contains(&"App shell"));
    let roots: Vec<_> = h.app.nav.rows().iter().filter(|r| r.depth == 0).collect();
    assert_eq!(roots.len(), NAV_GROUPS.len() + 1, "the groups and Overview");
    assert!(h.app.nav.rows().iter().all(|r| r.depth <= 1));
}

#[test]
fn the_current_pages_group_is_open_by_default_and_follows_navigation() {
    let mut h = Harness::new(120, 40, PageId::Overview);
    assert!(
        h.open_groups().is_empty(),
        "Overview is top level: {:#?}",
        h.sidebar()
    );
    // navigation opens the new group and closes the one it opened itself
    h.app.goto(PageId::Grid);
    h.draw();
    assert_eq!(h.open_groups(), ["Tables"], "{:#?}", h.sidebar());
    assert!(h.nav_row("Data grid").is_some());
    h.app.goto(PageId::Buttons);
    h.draw();
    assert_eq!(h.open_groups(), ["Inputs & forms"]);
    // a group the user opens stays open
    h.key(KeyCode::Char('0'));
    let (x, y) = h.nav_row("Overlays").unwrap();
    h.click(x, y);
    assert!(h.open_groups().contains(&"Overlays"));
    h.app.goto(PageId::Grid);
    h.draw();
    assert_eq!(h.open_groups(), ["Tables", "Overlays"]);
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
    let mut h = Harness::new(120, 40, PageId::Grid);
    let tables = root_of("Tables");
    h.key(KeyCode::Char('0'));
    assert_eq!(h.app.focus.current(), Some(NAV));
    assert_eq!(
        h.app.nav.cursor_path(),
        Some(&[tables, 2][..]),
        "on the current page"
    );
    // ← from a page goes to its group, ← again closes it
    h.key(KeyCode::Left);
    assert_eq!(h.app.nav.cursor_path(), Some(&[tables][..]));
    h.key(KeyCode::Left);
    assert!(h.open_groups().is_empty(), "{:#?}", h.sidebar());
    assert!(h.nav_row("Data grid").is_none());
    // → opens it again, Enter on a group toggles it
    h.key(KeyCode::Right);
    assert_eq!(h.open_groups(), ["Tables"]);
    h.key(KeyCode::Enter);
    assert!(h.open_groups().is_empty());
    // open it, step in, open "Editable"
    h.key(KeyCode::Right);
    h.key(KeyCode::Right);
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    assert_eq!(h.app.page, PageId::Editable);
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
    // a single click on a page row opens the page (navigation, not data)
    let (x, y) = h.nav_row("Selection").unwrap();
    h.click(x, y);
    assert_eq!(h.app.page, PageId::Selectable);
    assert_eq!(h.app.focus.current(), Some(NAV));
    // the group row itself toggles too
    let (x, y) = h.nav_row("App shell").unwrap();
    h.click(x, y);
    assert!(h.nav_row("Chrome").is_some());
    h.click(x, y);
    assert!(h.nav_row("Chrome").is_none());
}

#[test]
fn typing_a_name_jumps_in_the_sidebar_unless_the_page_claims_the_letter() {
    let mut h = Harness::new(120, 40, PageId::Overview);
    h.key(KeyCode::Char('0'));
    h.key(KeyCode::Char('t'));
    let at = h.app.nav.cursor_path().map(<[usize]>::to_vec);
    assert_eq!(at, Some(vec![root_of("Tables")]), "t: the Tables group");
    h.key(KeyCode::Char('e'));
    let at = h.app.nav.cursor_path().map(<[usize]>::to_vec);
    let text = root_of("Text & code");
    assert_eq!(at, Some(vec![text]), "te: Text & code");
    // the shell's own letters are never type-ahead
    h.key(KeyCode::Char('i'));
    assert!(h.app.inspector, "i toggles the inspector");
    assert_eq!(h.app.nav.cursor_path(), Some(&[text][..]));
}

#[test]
fn slash_filters_the_sidebar_and_enter_opens_the_match() {
    let mut h = Harness::new(120, 40, PageId::Overview);
    h.key(KeyCode::Char('0'));
    h.key(KeyCode::Char('/'));
    assert!(h.app.nav_filter.is_some());
    for c in "grid".chars() {
        h.key(KeyCode::Char(c));
    }
    let rows = h.sidebar();
    assert!(rows.iter().any(|r| r.contains("Data grid")), "{rows:#?}");
    assert!(
        rows.iter().any(|r| r.contains("Tables")),
        "its group as context"
    );
    assert!(!rows.iter().any(|r| r.contains("Buttons")), "{rows:#?}");
    // the card's title, its padding row, then the query field
    assert!(rows[0].contains("Components"), "the card title: {rows:#?}");
    assert!(rows[2].contains("/ grid"), "the query row: {rows:#?}");
    h.key(KeyCode::Enter);
    assert_eq!(h.app.page, PageId::Grid);
    assert!(h.app.nav_filter.is_none(), "opening ends the filter");
    // Esc clears a filter and shows the whole tree again
    h.key(KeyCode::Char('/'));
    h.key(KeyCode::Char('z'));
    h.key(KeyCode::Esc);
    assert!(h.app.nav_filter.is_none());
    assert!(h.nav_row("Data grid").is_some());
    assert!(h.nav_row("Overview").is_some());
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
        ("basic-table", PageId::Tables),
        ("editable-tables", PageId::Editable),
        ("editable", PageId::Editable),
        ("selectable-table", PageId::Selectable),
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
        (side.top()..side.bottom()).all(|y| (side.left()..side.right())
            .all(|x| !(buf[(x, y)].symbol() == "▎" && buf[(x, y)].fg == t.focus))),
        "{:#?}",
        h.sidebar()
    );
}

/// The sidebar's card belongs to the tree: its title shows focus-within
/// (primary and bold, never a second `▎`), and a click on the title or the
/// padding focuses the tree instead of landing on nothing.
#[test]
fn the_sidebar_card_title_shows_focus_and_its_padding_focuses_the_tree() {
    use ratatui::style::Modifier;
    let mut h = Harness::new(72, 20, PageId::Lists);
    let t = h.app.theme;
    h.key(KeyCode::Tab);
    assert_ne!(h.app.focus.current(), Some(NAV));
    let side = h.app.sidebar_area();
    let title = |h: &Harness| h.term.backend().buffer()[(side.x + 1, side.y)].clone();
    assert_eq!(title(&h).symbol(), "C");
    assert_eq!(
        title(&h).fg,
        t.text_secondary,
        "at rest the title is secondary"
    );
    for (x, y) in [
        (side.x + 1, side.y),
        (side.x, side.y + 5),
        (side.right() - 1, side.bottom() - 1),
    ] {
        h.key(KeyCode::Tab);
        if h.app.focus.current() == Some(NAV) {
            h.key(KeyCode::Tab);
        }
        assert_ne!(h.app.focus.current(), Some(NAV));
        h.click(x, y);
        assert_eq!(h.app.focus.current(), Some(NAV), "a click at ({x},{y})");
        assert_eq!(h.app.page, PageId::Lists, "the padding opens no page");
    }
    let cell = title(&h);
    assert_eq!(cell.fg, t.text_primary);
    assert!(cell.modifier.contains(Modifier::BOLD));
    assert_eq!(h.sidebar()[0].matches('▎').count(), 0);
}
