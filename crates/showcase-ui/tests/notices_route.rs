use showcase_ui::{App, NAV_ENTRIES, PageId};
use termrock::{Id, KeyCode, Theme};
use termrock_test_support::Harness;

const NAV: Id = Id::root("showcase_ui::app::navigation");
const PAGE_TABS: Id = Id::root("showcase_ui::pages::notices.page_tabs");
const PLAY_SCROLL: Id = Id::root("showcase_ui::pages::notices.play_scroll");
const OPTIONS_SCROLL: Id = Id::root("showcase_ui::pages::notices.options_scroll");
const TEST_CONNECTION: Id = Id::root("showcase_ui::pages::notices.test_connection");
const KIND: Id = Id::root("showcase_ui::pages::notices.option.kind");

fn footer(h: &Harness<App>) -> String {
    h.row(h.buffer().area().height.saturating_sub(1))
}

#[test]
fn notices_uses_its_existing_placeholder_without_changing_public_catalogs() {
    let mut h = Harness::new(App::for_page_name("Notices"), Theme::junie(), 120, 40);
    let index = NAV_ENTRIES
        .iter()
        .position(|entry| entry.label == "Notices")
        .expect("Notices remains in the navigation catalog");

    assert_eq!(index, 33);
    assert_eq!(NAV_ENTRIES.len(), 54);
    assert_eq!(NAV_ENTRIES[index].id, PageId::Overview);
    assert_eq!(PageId::ALL.len(), 24);
    assert_eq!(h.app().page(), PageId::Overview);
    assert!(format!("{:?}", h.app()).contains("nav_target: Placeholder(33)"));
    assert!(h.text().contains("Notices"));
    assert!(h.area_of(TEST_CONNECTION).is_some());
    assert!(h.area_of(PAGE_TABS).is_some());
    assert!(
        h.area_of(NAV).is_some(),
        "the navigation tree remains present"
    );
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.focus(), Some(PAGE_TABS));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn click_and_enter_test_connection_cycle_update_the_visible_status() {
    let mut h = Harness::new(App::for_page_name("Notices"), Theme::junie(), 120, 40);
    assert!(h.area_of(TEST_CONNECTION).is_some());

    let _ = h.click_id(TEST_CONNECTION);
    assert!(
        h.text().contains("Could not reach the host"),
        "{}",
        h.text()
    );
    assert!(
        footer(&h).contains("Could not reach the host"),
        "{}",
        footer(&h)
    );

    assert!(h.tab_to(TEST_CONNECTION));
    let hints = footer(&h);
    assert!(hints.contains("Test connection"), "{hints}");
    let _ = h.key(KeyCode::Enter);
    assert!(h.text().contains("Connected in 182 ms"), "{}", h.text());
    assert!(footer(&h).contains("Connected in 182 ms"), "{}", footer(&h));

    let _ = h.key(KeyCode::Enter);
    assert!(h.text().contains("Authentication failed"), "{}", h.text());
    assert!(
        footer(&h).contains("Authentication failed"),
        "{}",
        footer(&h)
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn notices_remains_live_after_route_away_back_and_small_resize() {
    let mut h = Harness::new(App::for_page_name("Notices"), Theme::junie(), 120, 40);
    let _ = h.click_id(TEST_CONNECTION);
    let _ = h.resize(80, 24);
    assert!(h.area_of(PLAY_SCROLL).is_some());
    assert!(h.area_of(TEST_CONNECTION).is_some());
    assert!(h.text().contains("Could not reach the host"));

    let _ = h.key(KeyCode::Char('['));
    assert!(format!("{:?}", h.app()).contains("nav_target: Placeholder(32)"));
    let _ = h.key(KeyCode::Char(']'));
    assert!(format!("{:?}", h.app()).contains("nav_target: Placeholder(33)"));
    assert!(h.text().contains("Could not reach the host"));
    assert!(h.area_of(TEST_CONNECTION).is_some());

    assert!(h.tab_to(PAGE_TABS));
    assert!(footer(&h).contains("Section"), "{}", footer(&h));
    let _ = h.key(KeyCode::Right);
    assert!(
        h.area_of(KIND).is_some(),
        "the option controls are reachable at 80×24"
    );
    for id in [
        Id::root("showcase_ui::pages::notices.option.detail_lines"),
        Id::root("showcase_ui::pages::notices.option.message_lines"),
        Id::root("showcase_ui::pages::notices.option.detail_tone"),
        Id::root("showcase_ui::pages::notices.option.width"),
        Id::root("showcase_ui::pages::notices.option.detail"),
        Id::root("showcase_ui::pages::notices.option.hint"),
        Id::root("showcase_ui::pages::notices.option.mark_cut"),
    ] {
        assert!(
            h.area_of(id).is_some(),
            "option {id:?} is reachable at 80×24"
        );
    }
    assert!(h.tab_to(KIND));
    assert!(footer(&h).contains("Choose"), "{}", footer(&h));
    let _ = h.key(KeyCode::Enter);
    let _ = h.key(KeyCode::Esc);
    let detail = Id::root("showcase_ui::pages::notices.option.detail");
    assert!(h.tab_to(detail));
    assert!(footer(&h).contains("Toggle"), "{}", footer(&h));
    assert!(h.tab_to(PAGE_TABS));
    let options = h
        .area_of(OPTIONS_SCROLL)
        .expect("the compact option panel stays scrollable");
    for id in [
        KIND,
        Id::root("showcase_ui::pages::notices.option.detail_lines"),
        Id::root("showcase_ui::pages::notices.option.message_lines"),
        Id::root("showcase_ui::pages::notices.option.detail_tone"),
        Id::root("showcase_ui::pages::notices.option.width"),
        detail,
        Id::root("showcase_ui::pages::notices.option.hint"),
        Id::root("showcase_ui::pages::notices.option.mark_cut"),
    ] {
        let control = h.area_of(id).expect("each compact option is visible");
        assert!(control.x >= options.x && control.right() <= options.right());
        assert!(control.y >= options.y && control.bottom() <= options.bottom());
    }
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn notice_reference_gallery_is_not_a_second_interactive_page() {
    let mut h = Harness::new(App::for_page_name("Notices"), Theme::junie(), 120, 40);
    assert!(h.tab_to(PAGE_TABS));
    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Right);
    assert!(h.text().contains("With detail"));
    assert!(h.text().contains("With hint"));
    assert!(h.area_of(TEST_CONNECTION).is_none());
    assert!(h.area_of(KIND).is_none());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
