use showcase_ui::{App, PageId};
use termrock::{
    Axis, Id, ItemKey, KeyCode, KeyModifiers, Part, PartRef, Position, Rect, Theme, width,
};
use termrock_test_support::Harness;

const NAV: Id = Id::root("showcase_ui::app::navigation");
const HEADER_HELP: Id = Id::root("showcase_ui::app::header.help");
const HELP: Id = Id::root("showcase_ui::app::help");
const INSPECTOR: Id = Id::root("showcase_ui::app::inspector");
const TEXT_AREA: Id = Id::root("showcase_ui::pages::textareas::textareas.body");
const NAV_GROUP_KEYSPACE: u64 = 0x4e41_5647_524f_5550;

fn app_shell_disclosure() -> PartRef {
    group_disclosure(0)
}

fn group_disclosure(index: u64) -> PartRef {
    PartRef::item(Part::ICON, ItemKey::pair(NAV_GROUP_KEYSPACE, index))
}

fn harness(app: App) -> Harness<App> {
    Harness::new(app, Theme::junie(), 120, 40)
}

fn nav_contains(h: &Harness<App>, label: &str) -> bool {
    let Some(nav) = h.area_of(NAV) else {
        return false;
    };
    (nav.y..nav.bottom()).any(|y| region_row(h, nav, y).contains(label))
}

fn nav_row(h: &Harness<App>, label: &str) -> Option<(u16, u16)> {
    let nav = h.area_of(NAV)?;
    (nav.y..nav.bottom()).find_map(|y| {
        let sidebar = region_row(h, nav, y);
        sidebar.find(label).map(|byte| {
            let column = width(&sidebar[..byte]);
            (nav.x.saturating_add(column), y)
        })
    })
}

fn region_row(h: &Harness<App>, area: Rect, y: u16) -> String {
    (area.x..area.right())
        .map(|x| h.cell(x, y).symbol().to_owned())
        .collect()
}

fn footer_text(h: &Harness<App>) -> String {
    let area = h.buffer().area();
    region_row(
        h,
        Rect::new(area.x, area.bottom().saturating_sub(1), area.width, 1),
        area.bottom().saturating_sub(1),
    )
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

fn nav_cursor(h: &Harness<App>) -> String {
    let debug = format!("{:?}", h.app());
    debug
        .split("nav_state: TreeState { core: CollectionCore { cursor: ")
        .nth(1)
        .and_then(|tail| tail.split(", cursor_index").next())
        .expect("navigation cursor is present in the app state")
        .to_owned()
}

fn nav_scroll_offset(h: &Harness<App>) -> String {
    let debug = format!("{:?}", h.app());
    debug
        .split("scroll: ScrollState { offset: ")
        .nth(1)
        .and_then(|tail| tail.split(',').next())
        .expect("navigation scroll offset is present in the app state")
        .to_owned()
}

fn nav_expanded_state(h: &Harness<App>) -> String {
    let debug = format!("{:?}", h.app());
    debug
        .split("expanded: ")
        .nth(1)
        .and_then(|tail| tail.split(", chosen").next())
        .expect("navigation expansion state is present in the app state")
        .to_owned()
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
fn query_row_uses_the_first_sidebar_row_and_tree_viewport_loses_exactly_one_row() {
    let mut h = harness(App::new());
    let inactive = h.area_of(NAV).expect("inactive navigation tree");

    let _ = h.key(KeyCode::Char('/'));
    let active = h.area_of(NAV).expect("active navigation tree");
    assert_eq!(active.x, inactive.x);
    assert_eq!(active.width, inactive.width);
    assert_eq!(active.y, inactive.y.saturating_add(1));
    assert_eq!(active.height, inactive.height.saturating_sub(1));
    assert!(
        region_row(
            &h,
            Rect::new(active.x, active.y - 1, active.width, 1),
            active.y - 1
        )
        .starts_with("/ ")
    );
    assert!(
        nav_contains(&h, "Overview"),
        "empty query retains the catalog"
    );
    assert_eq!(
        h.cursor(),
        Some(Position::new(active.x.saturating_add(2), active.y - 1))
    );

    let _ = h.type_str("界");
    let active = h.area_of(NAV).expect("tree remains below the query row");
    let query_row = Rect::new(active.x, active.y - 1, active.width, 1);
    assert!(region_row(&h, query_row, query_row.y).contains("/ 界"));
    assert_eq!(
        h.cursor(),
        Some(Position::new(
            query_row.x.saturating_add(width("/ 界")),
            query_row.y
        )),
        "caret follows the visible grapheme by display width"
    );
    assert!(
        h.cursor()
            .is_some_and(|cursor| cursor.x < query_row.right())
    );

    let _ = h.resize(80, 24);
    let resized = h.area_of(NAV).expect("resized query tree");
    assert_eq!(resized.x, 0);
    assert!(
        region_row(
            &h,
            Rect::new(resized.x, resized.y - 1, resized.width, 1),
            resized.y - 1
        )
        .starts_with("/ 界")
    );
    assert!(h.cursor().is_some_and(|cursor| cursor.x < resized.right()));
    let _ = h.click(resized.x.saturating_add(3), resized.y.saturating_sub(1));
    assert_eq!(h.app().page(), PageId::Overview);
    assert_query(&h, "界");
}

#[test]
fn query_footer_only_advertises_commands_owned_by_the_current_query_state() {
    let mut h = harness(App::new());
    let _ = h.key(KeyCode::Char('/'));
    let empty_query_footer = footer_text(&h);
    assert!(
        empty_query_footer.contains("type Filter"),
        "{empty_query_footer:?}"
    );
    assert!(
        empty_query_footer.contains("Backspace Edit"),
        "{empty_query_footer:?}"
    );
    assert!(
        empty_query_footer.contains("Esc Clear"),
        "{empty_query_footer:?}"
    );
    assert!(!empty_query_footer.contains("q Quit"), "q is query input");
    assert!(
        !empty_query_footer.contains("/ Filter"),
        "slash already opened the query"
    );

    let _ = h.type_str("q");
    let match_footer = footer_text(&h);
    assert!(match_footer.contains("Enter Open"), "{match_footer:?}");
    assert!(
        !match_footer.contains("q Quit"),
        "q remains text while filtering"
    );

    let _ = h.type_str("no-such-page");
    let no_match_footer = footer_text(&h);
    assert!(
        !no_match_footer.contains("Enter Open"),
        "{no_match_footer:?}"
    );
    assert!(!no_match_footer.contains("q Quit"), "{no_match_footer:?}");

    let _ = h.key(KeyCode::Esc);
    let idle_footer = footer_text(&h);
    assert!(idle_footer.contains("/ Filter"), "{idle_footer:?}");
    assert!(idle_footer.contains("q Quit"), "{idle_footer:?}");

    let mut group = harness(App::new());
    let nav = group.area_of(NAV).expect("navigation tree");
    let (_, group_y) = nav_row(&group, "App shell").expect("App shell group");
    let _ = group.click(nav.x.saturating_add(1), group_y);
    let group_footer = footer_text(&group);
    assert!(group_footer.contains("← → Fold"), "{group_footer:?}");
    assert!(group_footer.contains("Enter Toggle"), "{group_footer:?}");
    assert!(!group_footer.contains("Enter Open"), "{group_footer:?}");

    let mut page = harness(App::for_page_name("Text areas"));
    let _ = page.key(KeyCode::Tab);
    let _ = page.key(KeyCode::Enter);
    let editing_footer = footer_text(&page);
    assert!(
        editing_footer.contains("Enter Newline") && editing_footer.contains("Esc Done"),
        "page-specific editing hints remain owned by the focused editor: {editing_footer:?}"
    );
    assert!(!editing_footer.contains("type Filter"));
}

#[test]
fn first_pointer_after_query_open_hits_the_new_query_row_not_the_old_tree_row() {
    let mut h = harness(App::new());
    let original = h.area_of(NAV).expect("inactive tree");
    let _ = h.key(KeyCode::Char('/'));
    let nav = h.area_of(NAV).expect("query tree");
    let query_y = nav.y.saturating_sub(1);
    assert_eq!(query_y, original.y);
    let _ = h.click(nav.x.saturating_add(3), query_y);
    assert_eq!(
        h.app().page(),
        PageId::Overview,
        "query row is outside Tree hits"
    );
    assert_query(&h, "");

    let _ = h.key(KeyCode::Esc);
    let restored = h.area_of(NAV).expect("restored full-height tree");
    assert_eq!(restored, original);
    let (_, chrome_y) = nav_row(&h, "Chrome").expect("Chrome row after clear");
    let _ = h.click(restored.x.saturating_add(3), chrome_y);
    assert_eq!(
        h.app().page(),
        PageId::Chrome,
        "first restored Tree hit works"
    );
}

#[test]
fn query_time_disclosure_keeps_post_click_cursor_and_commits_the_fold() {
    let mut h = harness(App::new());
    let nav = h.area_of(NAV).expect("navigation tree");
    let (_, group_y) = nav_row(&h, "App shell").expect("App shell group");
    let _ = h.click(nav.x.saturating_add(1), group_y);
    assert!(
        !nav_contains(&h, "Chrome"),
        "idle disclosure collapses the group"
    );
    let app_shell_cursor = nav_cursor(&h);
    let _ = h.key(KeyCode::Down);
    let other_group_cursor = nav_cursor(&h);
    assert_ne!(app_shell_cursor, other_group_cursor);

    let _ = h.key(KeyCode::Char('/'));
    let scroll_before_disclosure = nav_scroll_offset(&h);
    assert!(nav_row(&h, "App shell").is_some(), "query group row");
    assert!(h.area_of_part(NAV, app_shell_disclosure()).is_some());
    let _ = h.click_part(NAV, app_shell_disclosure());
    assert_query(&h, "");
    assert!(
        nav_contains(&h, "Chrome"),
        "opening the idle fold also opens it in the empty-query projection: {}\n{:?}",
        h.text(),
        h.app()
    );
    assert_eq!(
        nav_cursor(&h),
        app_shell_cursor,
        "disclosure selects its group"
    );
    assert_eq!(
        nav_scroll_offset(&h),
        scroll_before_disclosure,
        "the explicit disclosure keeps the Tree viewport"
    );

    let _ = h.key(KeyCode::Esc);
    assert_query_closed(&h);
    assert!(
        nav_contains(&h, "Chrome"),
        "clearing preserves the explicit query disclosure"
    );
}

#[test]
fn query_pointer_disclosure_persists_except_when_clear_reveals_the_chosen_route() {
    let mut unrelated_route = harness(App::new());
    let _ = unrelated_route.key(KeyCode::Char('/'));
    let _ = unrelated_route.type_str("Chrome");
    assert!(nav_contains(&unrelated_route, "Chrome"));
    assert!(nav_row(&unrelated_route, "App shell").is_some());
    let cursor_before_click = nav_cursor(&unrelated_route);
    let scroll_before_click = nav_scroll_offset(&unrelated_route);
    assert!(
        unrelated_route
            .area_of_part(NAV, app_shell_disclosure())
            .is_some()
    );
    let _ = unrelated_route.click_part(NAV, app_shell_disclosure());
    assert_query(&unrelated_route, "Chrome");
    assert!(
        nav_contains(&unrelated_route, "Chrome"),
        "a nonmatching ancestor stays visible for the active filter"
    );
    assert_ne!(nav_cursor(&unrelated_route), cursor_before_click);
    assert_eq!(
        nav_scroll_offset(&unrelated_route),
        scroll_before_click,
        "the disclosure keeps the current filtered viewport"
    );
    let _ = unrelated_route.key(KeyCode::Esc);
    assert_query_closed(&unrelated_route);
    assert!(
        !nav_contains(&unrelated_route, "Chrome"),
        "the explicit fold remains after clearing an unrelated route filter"
    );

    let mut chosen_route = harness(App::with_page(PageId::Chrome));
    let _ = chosen_route.key(KeyCode::Char('/'));
    let _ = chosen_route.type_str("Chrome");
    assert!(nav_row(&chosen_route, "App shell").is_some());
    assert!(
        chosen_route
            .area_of_part(NAV, app_shell_disclosure())
            .is_some()
    );
    let _ = chosen_route.click_part(NAV, app_shell_disclosure());
    assert!(nav_contains(&chosen_route, "Chrome"));
    let _ = chosen_route.key(KeyCode::Esc);
    assert_query_closed(&chosen_route);
    assert_eq!(chosen_route.app().page(), PageId::Chrome);
    assert!(
        nav_contains(&chosen_route, "Chrome"),
        "clearing reveals and expands the chosen route's ancestor"
    );
}

#[test]
fn query_matching_group_disclosure_uses_the_saved_fold_during_and_after_filtering() {
    let mut h = harness(App::new());
    let _ = h.key(KeyCode::Char('/'));
    let _ = h.type_str("App shell");
    assert!(nav_contains(&h, "App shell"));
    assert!(nav_contains(&h, "Chrome"));
    let expanded = nav_expanded_state(&h);

    assert!(h.area_of_part(NAV, app_shell_disclosure()).is_some());
    let _ = h.click_part(NAV, app_shell_disclosure());
    assert!(nav_contains(&h, "App shell"));
    assert!(
        !nav_contains(&h, "Chrome"),
        "a matching group respects its explicit collapsed state"
    );
    assert_ne!(nav_expanded_state(&h), expanded);

    let _ = h.key(KeyCode::Esc);
    assert_query_closed(&h);
    assert!(
        !nav_contains(&h, "Chrome"),
        "the matching group's explicit fold survives query clear"
    );
}

#[test]
fn query_pointer_fold_keeps_a_scrolled_viewport_and_the_clicked_group_cursor() {
    let mut h = harness(App::new());
    let idle_nav = h.area_of(NAV).expect("idle tree");
    let (_, app_shell_y) = nav_row(&h, "App shell").expect("App shell group");
    let _ = h.click(idle_nav.x.saturating_add(1), app_shell_y);
    assert!(!nav_contains(&h, "Chrome"));

    let _ = h.key(KeyCode::Char('/'));
    let track = h
        .area_of_part(NAV, PartRef::of(Part::TRACK))
        .expect("expanded catalog has a query viewport track");
    let _ = h.wheel(Axis::V, 1, track.x, track.y.saturating_add(1));
    let scroll_before = nav_scroll_offset(&h)
        .parse::<usize>()
        .expect("numeric scroll offset");
    assert!(
        scroll_before > 0,
        "wheel establishes a nonzero viewport offset"
    );

    assert!(
        nav_row(&h, "Tables").is_some(),
        "the group is in view at offset {scroll_before}: {}",
        h.text()
    );
    let table_fold = group_disclosure(2);
    assert!(h.area_of_part(NAV, table_fold).is_some());
    let cursor_before = nav_cursor(&h);
    let _ = h.click_part(NAV, table_fold);
    assert_ne!(
        nav_cursor(&h),
        cursor_before,
        "Tree keeps the clicked group cursor"
    );
    assert!(!nav_contains(&h, "Editable tables"));
    let scroll_after = nav_scroll_offset(&h)
        .parse::<usize>()
        .expect("numeric scroll offset");
    assert!(
        scroll_after > 0 && scroll_after <= scroll_before,
        "the viewport remains scrolled while content shrinks: {scroll_before} -> {scroll_after}"
    );

    let _ = h.key(KeyCode::Esc);
    assert_query_closed(&h);
    assert!(!nav_contains(&h, "Editable tables"));
}

#[test]
fn query_keyboard_navigation_consumes_fold_arrows_and_enter_only_opens_leaves() {
    let mut h = harness(App::new());
    let _ = h.key(KeyCode::Char('/'));
    let _ = h.type_str("App shell");
    assert!(nav_contains(&h, "App shell"));
    assert!(nav_contains(&h, "Chrome"));
    let _ = h.key(KeyCode::Home);
    let group_cursor = nav_cursor(&h);
    let expanded = nav_expanded_state(&h);

    for key in [
        KeyCode::Left,
        KeyCode::Right,
        KeyCode::PageUp,
        KeyCode::PageDown,
    ] {
        let _ = h.key(key);
        assert_eq!(nav_cursor(&h), group_cursor, "query consumes {key:?}");
        assert_eq!(
            nav_expanded_state(&h),
            expanded,
            "query does not fold on {key:?}"
        );
    }
    let _ = h.key(KeyCode::Enter);
    assert_eq!(
        h.app().page(),
        PageId::Overview,
        "Enter does not toggle a group"
    );
    assert_eq!(nav_cursor(&h), group_cursor);
    assert_eq!(nav_expanded_state(&h), expanded);

    let _ = h.key(KeyCode::Down);
    let leaf_cursor = nav_cursor(&h);
    assert_ne!(leaf_cursor, group_cursor);
    let page_before_left = h.app().page();
    let _ = h.key(KeyCode::Left);
    assert_eq!(
        nav_cursor(&h),
        leaf_cursor,
        "query Left cannot move to parent"
    );
    assert_eq!(h.app().page(), page_before_left);
    assert_eq!(nav_expanded_state(&h), expanded);
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

    let mut tree_keys = harness(App::new());
    let _ = tree_keys.key(KeyCode::Char('/'));
    let _ = tree_keys.type_str("jkh l");
    assert_query(&tree_keys, "jkh l");
    assert_eq!(tree_keys.app().page(), PageId::Overview);

    let mut modified = harness(App::new());
    let _ = modified.key(KeyCode::Char('/'));
    let _ = modified.type_str("q");
    let _ = modified.key_mod(KeyCode::Char('c'), KeyModifiers::CONTROL);
    assert!(
        modified.app().quit(),
        "Ctrl+C remains a modified quit command"
    );
    assert_query(&modified, "q");

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
fn query_projection_reveals_collapsed_ancestors_without_changing_group_folds() {
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
    let help_footer = footer_text(&modal);
    assert!(help_footer.contains("Esc Close"), "{help_footer:?}");
    assert!(
        !help_footer.contains("q Quit"),
        "underlay hints stay hidden"
    );
    let _ = modal.key(KeyCode::Char('/'));
    assert_query_closed(&modal);
    assert_eq!(modal.app().page(), PageId::Overview);
}
