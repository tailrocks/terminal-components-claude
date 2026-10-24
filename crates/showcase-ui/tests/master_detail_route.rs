use showcase_ui::{App, NAV_ENTRIES, PageId};
use termrock::{Id, ItemKey, KeyCode, Part, PartRef, Theme};
use termrock_test_support::Harness;

const NAV: Id = Id::root("showcase_ui::app::navigation");
const NAV_ENTRY_KEYSPACE: u64 = 0x4e41_5645_4e54_5259;
const PAGE_TABS: Id = Id::root("showcase_ui::pages::master_detail.page_tabs");
const MASTER: Id = Id::root("showcase_ui::pages::master_detail.widget")
    .index(0)
    .index(1);
const PROPS: Id = Id::root("showcase_ui::pages::master_detail.widget")
    .index(0)
    .index(3);
const WIDTH: Id = Id::root("showcase_ui::pages::master_detail.option.width");
const COLLAPSE: Id = Id::root("showcase_ui::pages::master_detail.option.collapse");
const ESC: Id = Id::root("showcase_ui::pages::master_detail.option.esc");

fn footer(h: &Harness<App>) -> String {
    h.row(h.buffer().area().height.saturating_sub(1))
}

fn master_detail_target(h: &Harness<App>) -> bool {
    format!("{:?}", h.app()).contains("nav_target: Placeholder(37)")
}

fn click_tab(h: &mut Harness<App>, index: usize) {
    let _ = h.click_part(PAGE_TABS, PartRef::item(Part::TAB, ItemKey::index(index)));
}

fn click_nav_entry(h: &mut Harness<App>, index: usize) {
    let row = PartRef::item(Part::ROW, ItemKey::pair(NAV_ENTRY_KEYSPACE, index as u64));
    assert!(
        h.area_of_part(NAV, row).is_some(),
        "catalog row {index} is live"
    );
    let _ = h.click_part(NAV, row);
}

fn select_index(h: &mut Harness<App>, id: Id, current: usize, target: usize) {
    assert!(
        h.tab_to(id),
        "option {id:?} is available on the Options tab"
    );
    let _ = h.key(KeyCode::Enter);
    let direction = if target >= current {
        KeyCode::Down
    } else {
        KeyCode::Up
    };
    for _ in 0..current.abs_diff(target) {
        let _ = h.key(direction);
    }
    let _ = h.key(KeyCode::Enter);
}

#[test]
fn master_detail_uses_its_private_catalog_slot_and_keeps_neighbor_routes() {
    let index = NAV_ENTRIES
        .iter()
        .position(|entry| entry.label == "Master-detail")
        .expect("Master-detail remains in the navigation catalog");
    assert_eq!(index, 37);
    assert_eq!(NAV_ENTRIES.len(), 54);
    assert_eq!(PageId::ALL.len(), 24);
    assert_eq!(NAV_ENTRIES[36].label, "Splits & stacks");
    assert_eq!(NAV_ENTRIES[37].label, "Master-detail");
    assert_eq!(NAV_ENTRIES[38].label, "Pane tree");
    assert_eq!(NAV_ENTRIES[index].id, PageId::Overview);
    assert_eq!(PageId::from_name("Master-detail"), None);
    assert_eq!(PageId::from_name("masterdetail"), None);

    for name in ["Master-detail", "masterdetail"] {
        let mut h = Harness::new(App::for_page_name(name), Theme::junie(), 120, 40);
        assert_eq!(h.app().page(), PageId::Overview);
        assert!(master_detail_target(&h), "{name} uses Placeholder(37)");
        assert_eq!(h.focus(), Some(NAV));
        assert!(h.area_of(PAGE_TABS).is_some(), "the real page tabs mount");
        assert!(h.area_of(MASTER).is_some(), "the real table List mounts");
        assert!(h.area_of(PROPS).is_some(), "the real fact list mounts");
        assert!(h.text().contains("orders"), "{}", h.text());
        assert!(h.text().contains("shop.public.orders"), "{}", h.text());

        let _ = h.key(KeyCode::Down);
        assert!(
            format!("{:?}", h.app()).contains("nav_target: Placeholder(38)"),
            "the following catalog target remains the Pane tree placeholder"
        );
        assert!(h.area_of(PAGE_TABS).is_none());
        let _ = h.key(KeyCode::Up);
        assert!(
            master_detail_target(&h),
            "Up returns to the same catalog row"
        );
        assert!(h.area_of(PAGE_TABS).is_some());
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
}

#[test]
fn master_detail_route_forwards_page_focus_and_modal_option_hints() {
    let mut h = Harness::new(App::for_page_name("Master-detail"), Theme::junie(), 120, 40);
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.focus(), Some(PAGE_TABS), "Enter focuses page content");
    assert!(footer(&h).contains("Section"), "{}", footer(&h));

    click_tab(&mut h, 1);
    assert!(h.area_of(WIDTH).is_some());
    assert!(h.area_of(COLLAPSE).is_some());
    assert!(h.tab_to(WIDTH));
    assert!(footer(&h).contains("Choose"), "{}", footer(&h));
    let _ = h.key(KeyCode::Enter);
    assert!(footer(&h).contains("Choose"), "{}", footer(&h));
    let _ = h.key(KeyCode::Esc);

    select_index(&mut h, WIDTH, 0, 2);
    select_index(&mut h, COLLAPSE, 0, 1);
    assert!(format!("{:?}", h.app()).contains("width: 2"));
    assert!(format!("{:?}", h.app()).contains("collapse: 1"));

    click_tab(&mut h, 0);
    let master = h.area_of(MASTER).expect("live collapsed table List");
    assert!(master.width <= 72, "width option is active: {master:?}");
    assert!(
        h.area_of_part(
            Id::root("showcase_ui::pages::master_detail.widget").index(0),
            PartRef::of(Part::SEAM),
        )
        .is_none(),
        "collapsed routing uses the live master-only layout"
    );
    assert!(h.tab_to(MASTER));
    let _ = h.key(KeyCode::Tab);
    assert!(
        format!("{:?}", h.app()).contains("drawer_open: true"),
        "the route forwards focus into the real detail drawer"
    );
    assert!(h.text().contains("Esc returns to tables"), "{}", h.text());
    assert_eq!(h.focus(), Some(PROPS));
    let _ = h.key(KeyCode::Esc);
    assert_eq!(h.focus(), Some(MASTER));
    assert!(
        !format!("{:?}", h.app()).contains("drawer_open: true"),
        "the page's enabled Esc action closes the live drawer"
    );
    let _ = h.key(KeyCode::Tab);
    assert!(format!("{:?}", h.app()).contains("drawer_open: true"));
    let _ = h.resize(80, 24);
    assert!(h.area_of(PROPS).is_some(), "drawer survives app resize");
    assert!(h.text().contains("Schema"), "{}", h.text());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn master_detail_options_and_page_state_survive_a_navigation_roundtrip() {
    let mut h = Harness::new(App::for_page_name("Master-detail"), Theme::junie(), 120, 40);
    click_tab(&mut h, 1);
    select_index(&mut h, WIDTH, 0, 2);
    select_index(&mut h, COLLAPSE, 0, 1);
    select_index(&mut h, ESC, 1, 0);
    assert!(format!("{:?}", h.app()).contains("width: 2"));
    assert!(format!("{:?}", h.app()).contains("esc_returns: 0"));

    click_nav_entry(&mut h, 36);
    assert!(format!("{:?}", h.app()).contains("nav_target: Placeholder(36)"));
    assert!(h.area_of(PAGE_TABS).is_none());
    click_nav_entry(&mut h, 37);
    assert!(master_detail_target(&h));
    assert_eq!(h.app().page(), PageId::Overview);
    assert!(format!("{:?}", h.app()).contains("width: 2"));
    assert!(format!("{:?}", h.app()).contains("collapse: 1"));
    assert!(format!("{:?}", h.app()).contains("esc_returns: 0"));
    assert!(h.area_of(WIDTH).is_some(), "Options tab state is retained");

    click_tab(&mut h, 0);
    assert!(h.area_of(MASTER).is_some());
    assert!(
        !h.text().contains("Esc returns to tables"),
        "drawer copy follows the restored off setting\n{}",
        h.text()
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
