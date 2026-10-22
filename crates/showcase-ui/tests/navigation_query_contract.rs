use showcase_ui::{App, PageId};
use termrock::{Id, KeyCode, Part, PartRef, Theme};
use termrock_test_support::Harness;

const NAV: Id = Id::root("showcase_ui::app::navigation");
const HEADER_HELP: Id = Id::root("showcase_ui::app::header.help");
const HELP: Id = Id::root("showcase_ui::app::help");
const INSPECTOR: Id = Id::root("showcase_ui::app::inspector");
const TEXT_AREA: Id = Id::root("showcase_ui::pages::textareas::textareas.body");

fn harness(app: App) -> Harness<App> {
    Harness::new(app, Theme::junie(), 120, 40)
}

fn nav_contains(h: &Harness<App>, label: &str) -> bool {
    let Some(nav) = h.area_of(NAV) else {
        return false;
    };
    (nav.y..nav.bottom()).any(|y| {
        h.row(y)
            .chars()
            .take(usize::from(nav.width))
            .collect::<String>()
            .contains(label)
    })
}

fn nav_row(h: &Harness<App>, label: &str) -> Option<(u16, u16)> {
    let nav = h.area_of(NAV)?;
    (nav.y..nav.bottom()).find_map(|y| {
        let row = h.row(y);
        let sidebar = row.chars().take(usize::from(nav.width)).collect::<String>();
        sidebar.find(label).map(|byte| {
            let column =
                u16::try_from(sidebar[..byte].chars().count()).expect("sidebar column fits u16");
            (nav.x.saturating_add(column), y)
        })
    })
}

fn assert_query(h: &Harness<App>, expected: &str) {
    let debug = format!("{:?}", h.app());
    assert!(
        debug.contains(&format!("nav_query: Some(NavQuery {{ text: {expected:?}")),
        "query {expected:?} missing from app state: {debug}"
    );
}

fn assert_query_closed(h: &Harness<App>) {
    assert!(
        format!("{:?}", h.app()).contains("nav_query: None"),
        "navigation query remains active: {:?}",
        h.app()
    );
}

#[test]
fn slash_filters_unicode_aliases_and_tree_printables_without_routing() {
    let mut h = harness(App::new());
    let _ = h.key(KeyCode::Char('/'));
    let _ = h.type_str("cell");

    assert_query(&h, "cell");
    assert!(nav_contains(&h, "Cell editors"), "{}", h.text());
    assert_eq!(h.app().page(), PageId::Overview, "typing only filters");

    let _ = h.key(KeyCode::Esc);
    assert_query_closed(&h);
    assert_eq!(h.focus(), Some(NAV), "Escape keeps navigation focus");
    assert!(
        nav_contains(&h, "Overview"),
        "query clear reveals chosen route"
    );

    let mut unicode = harness(App::new());
    let _ = unicode.key(KeyCode::Char('/'));
    let _ = unicode.type_str("CAFÉ");
    assert_query(&unicode, "CAFÉ");
    assert!(unicode.text().contains("No matching pages"));
    let _ = unicode.key(KeyCode::Backspace);
    assert_query(&unicode, "CAF");
    assert!(unicode.text().contains("No matching pages"));
}

#[test]
fn aliases_and_global_shortcuts_are_query_text_while_nav_owns_focus() {
    let mut alias = harness(App::new());
    let _ = alias.key(KeyCode::Char('/'));
    let _ = alias.type_str("OPTIONS");
    assert_query(&alias, "OPTIONS");
    assert!(nav_contains(&alias, "Cell editors"), "{}", alias.text());
    assert_eq!(alias.app().page(), PageId::Overview);

    let mut global = harness(App::new());
    let _ = global.key(KeyCode::Char('/'));
    let _ = global.type_str("inputs");
    let _ = global.key(KeyCode::Char('?'));
    let _ = global.key(KeyCode::Char('q'));
    assert_query(&global, "inputs?q");
    assert!(!global.is_open(HELP), "question mark remains query text");
    assert!(global.area_of(INSPECTOR).is_none());
    assert!(!global.app().quit(), "q remains query text");
    assert_eq!(global.app().page(), PageId::Overview);

    let mut overview = harness(App::new());
    let _ = overview.key(KeyCode::Char('/'));
    let _ = overview.type_str("overview");
    assert!(nav_contains(&overview, "Overview"));
    assert!(!nav_contains(&overview, "Launch cockpit"));
    assert!(!nav_contains(&overview, "Read-only report"));
}

#[test]
fn query_enter_opens_match_and_no_match_enter_cannot_reuse_stale_cursor() {
    let mut matching = harness(App::new());
    let _ = matching.key(KeyCode::Char('/'));
    let _ = matching.type_str("settings");
    assert!(nav_contains(&matching, "Settings"), "{}", matching.text());
    assert_eq!(matching.app().page(), PageId::Overview);
    let _ = matching.key(KeyCode::Enter);
    assert_eq!(matching.app().page(), PageId::Settings);
    assert_query_closed(&matching);

    let mut empty = harness(App::new());
    let _ = empty.key(KeyCode::Char('/'));
    let _ = empty.type_str("no-such-page");
    assert!(
        empty.text().contains("No matching pages"),
        "{}",
        empty.text()
    );
    assert!(format!("{:?}", empty.app()).contains("cursor: None"));
    let _ = empty.key(KeyCode::Enter);
    assert_eq!(
        empty.app().page(),
        PageId::Overview,
        "no stale cursor routes"
    );

    for _ in "no-such-page".chars() {
        let _ = empty.key(KeyCode::Backspace);
    }
    assert_query(&empty, "");
    assert!(nav_contains(&empty, "Overview"), "{}", empty.text());
    assert!(!empty.text().contains("No matching pages"));
    let _ = empty.key(KeyCode::Esc);
    assert_query_closed(&empty);
    assert_eq!(empty.app().page(), PageId::Overview);
}

#[test]
fn query_reveals_collapsed_ancestors_without_changing_group_folds() {
    let mut h = harness(App::new());
    let nav = h.area_of(NAV).expect("navigation tree");
    let (_, group_y) = nav_row(&h, "App shell").expect("App shell group");
    let _ = h.click(nav.x.saturating_add(1), group_y);
    assert!(!nav_contains(&h, "Chrome"), "group collapsed");

    let _ = h.key(KeyCode::Char('/'));
    let _ = h.type_str("Chrome");
    assert!(nav_contains(&h, "App shell"));
    assert!(nav_contains(&h, "Chrome"), "{}", h.text());
    assert_eq!(h.app().page(), PageId::Overview);

    let _ = h.key(KeyCode::Esc);
    assert_query_closed(&h);
    assert!(!nav_contains(&h, "Chrome"), "original fold state restored");
    assert_eq!(h.app().page(), PageId::Overview);
}

#[test]
fn focus_out_clears_query_and_page_text_or_modal_keeps_key_ownership() {
    let mut text_page = harness(App::for_page_name("Text areas"));
    let _ = text_page.key(KeyCode::Char('/'));
    let _ = text_page.type_str("text");
    assert_query(&text_page, "text");
    let _ = text_page.key(KeyCode::Tab);
    assert_query_closed(&text_page);
    assert_ne!(text_page.focus(), Some(NAV));
    assert_eq!(text_page.focus(), Some(TEXT_AREA));
    let _ = text_page.key(KeyCode::Enter);
    let _ = text_page.type_str("/query-owner");
    assert_query_closed(&text_page);
    assert!(
        text_page.text().contains("/query-owner"),
        "{}",
        text_page.text()
    );
    assert_eq!(text_page.app().page(), PageId::TextAreas);

    let mut modal = harness(App::new());
    let _ = modal.key(KeyCode::Char('/'));
    let _ = modal.type_str("inputs");
    assert_query(&modal, "inputs");
    let help = modal
        .area_of_part(HEADER_HELP, PartRef::of(Part::LABEL))
        .expect("header help action");
    let _ = modal.click(help.x.saturating_add(1), help.y);
    assert!(modal.is_open(HELP));
    assert_query_closed(&modal);
    let _ = modal.key(KeyCode::Char('/'));
    assert_query_closed(&modal);
    assert_eq!(modal.app().page(), PageId::Overview);
}
