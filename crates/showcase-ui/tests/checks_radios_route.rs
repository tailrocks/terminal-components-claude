use showcase_ui::{App, NAV_ENTRIES, PageId};
use termrock::{Id, KeyCode, Theme};
use termrock_test_support::Harness;

const CHECK_ALL: Id = Id::root("showcase_ui::pages::checks_radios.check_all");
const CHECK_NOTIFICATIONS: Id = Id::root("showcase_ui::pages::checks_radios.check_notifications");
const NAV: Id = Id::root("showcase_ui::app::navigation");
const PAGE_TABS: Id = Id::root("showcase_ui::pages::checks_radios.page_tabs");
const GROUP_TABS: Id = Id::root("showcase_ui::pages::checks_radios.group_tabs");
const RADIO_ISOLATION: Id = Id::root("showcase_ui::pages::checks_radios.radio_isolation");
const HELP: Id = Id::root("showcase_ui::app::help");
const HEADER_HELP: Id = Id::root("showcase_ui::app::header.help");
const OPTION_HIT: Id = Id::root("showcase_ui::pages::checks_radios.option_hit");

#[test]
fn checks_radios_keeps_placeholder_identity_while_showing_live_controls() {
    let h = Harness::new(
        App::for_page_name("Checks & radios"),
        Theme::junie(),
        120,
        40,
    );

    let entry = NAV_ENTRIES
        .iter()
        .position(|entry| entry.label == "Checks & radios")
        .expect("Checks & radios remains in the navigation catalog");
    assert_eq!(entry, 12);
    assert_eq!(NAV_ENTRIES.len(), 54);
    assert_eq!(h.app().page(), PageId::Overview);
    assert!(
        format!("{:?}", h.app()).contains("nav_target: Placeholder(12)"),
        "the route keeps its existing target identity"
    );
    assert!(
        h.area_of(CHECK_ALL).is_some(),
        "the placeholder body must be replaced by a live checkbox"
    );
    assert!(h.text().contains("Checkboxes"), "{}", h.text());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn checks_radios_aliases_and_every_other_catalog_target_keep_their_identity() {
    for alias in ["Checks & radios", "checks", "radios"] {
        let h = Harness::new(App::for_page_name(alias), Theme::junie(), 80, 24);
        assert!(h.area_of(CHECK_ALL).is_some(), "alias {alias}");
        assert!(format!("{:?}", h.app()).contains("nav_target: Placeholder(12)"));
        assert!(
            h.text().contains("Checks & radios"),
            "alias {alias}\n{}",
            h.text()
        );
    }

    let mut placeholder_count = 0;
    for (index, entry) in NAV_ENTRIES.iter().enumerate() {
        let target = if entry.label == "Selection" {
            "Selection".to_owned()
        } else if entry.label == "Overview" || entry.id != PageId::Overview {
            format!("Live({:?})", entry.id)
        } else {
            placeholder_count += 1;
            format!("Placeholder({index})")
        };
        let h = Harness::new(App::for_page_name(entry.label), Theme::junie(), 80, 24);
        let debug = format!("{:?}", h.app());
        assert!(
            debug.contains(&format!("nav_target: {target}")),
            "{}: {debug}",
            entry.label
        );
    }
    assert_eq!(placeholder_count, 29, "only Checks & radios gains a body");
    assert_eq!(PageId::ALL.len(), 24);
    assert_eq!(NAV_ENTRIES.len(), 54);
}

#[test]
fn keyboard_activation_and_tab_enter_the_live_placeholder_body() {
    let mut h = Harness::new(
        App::for_page_name("Checks & radios"),
        Theme::junie(),
        80,
        24,
    );
    assert_eq!(h.focus(), Some(NAV));
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.focus(), Some(PAGE_TABS));
    assert!(h.area_of(CHECK_ALL).is_some());

    let mut h = Harness::new(
        App::for_page_name("Checks & radios"),
        Theme::junie(),
        80,
        24,
    );
    let _ = h.key(KeyCode::Tab);
    assert_ne!(h.focus(), Some(NAV));
    assert!(h.area_of(PAGE_TABS).is_some());
}

#[test]
fn min_shell_size_registers_real_body_controls_without_diagnostics() {
    for (width, height) in [(72, 20), (80, 24), (100, 30), (120, 40)] {
        let mut h = Harness::new(
            App::for_page_name("Checks & radios"),
            Theme::junie(),
            width,
            height,
        );
        assert!(
            h.area_of(CHECK_ALL).is_some(),
            "{width}x{height}: {}",
            h.text()
        );
        assert!(h.area_of(PAGE_TABS).is_some());
        let _ = h.click_part(
            PAGE_TABS,
            termrock::PartRef::item(termrock::Part::TAB, termrock::ItemKey::index(1)),
        );
        for id in [
            Id::root("showcase_ui::pages::checks_radios.option_hit"),
            Id::root("showcase_ui::pages::checks_radios.option_mixed"),
            Id::root("showcase_ui::pages::checks_radios.option_radio_nav"),
            Id::root("showcase_ui::pages::checks_radios.option_disabled"),
        ] {
            assert!(
                h.area_of(id).is_some(),
                "{id:?} at {width}x{height}\n{}",
                h.text()
            );
        }
        let _ = h.click_part(
            PAGE_TABS,
            termrock::PartRef::item(termrock::Part::TAB, termrock::ItemKey::index(0)),
        );
        let _ = h.click_part(
            GROUP_TABS,
            termrock::PartRef::item(termrock::Part::TAB, termrock::ItemKey::index(1)),
        );
        assert!(
            h.area_of(RADIO_ISOLATION).is_some(),
            "radio group at {width}x{height}"
        );
        let _ = h.click_part(
            GROUP_TABS,
            termrock::PartRef::item(termrock::Part::TAB, termrock::ItemKey::index(2)),
        );
        assert!(
            h.area_of(Id::root(
                "showcase_ui::pages::checks_radios.toggle_autosave"
            ))
            .is_some()
        );
        let _ = h.click_part(
            PAGE_TABS,
            termrock::PartRef::item(termrock::Part::TAB, termrock::ItemKey::index(2)),
        );
        assert!(
            h.text().contains("Overflow"),
            "gallery at {width}x{height}\n{}",
            h.text()
        );
        assert!(
            h.area_of(CHECK_ALL).is_none(),
            "hidden controls stay unmounted"
        );
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
}

#[test]
fn query_keyboard_and_pointer_both_open_the_existing_navigation_entry() {
    let mut keyboard = Harness::new(App::for_page_name("Forms"), Theme::junie(), 120, 40);
    let _ = keyboard.key(KeyCode::Char('/'));
    let _ = keyboard.type_str("checks");
    let _ = keyboard.key(KeyCode::Enter);
    assert!(format!("{:?}", keyboard.app()).contains("nav_target: Placeholder(12)"));
    assert_eq!(
        keyboard.app().page(),
        PageId::Forms,
        "the public legacy page id stays unchanged"
    );
    assert_eq!(keyboard.focus(), Some(PAGE_TABS));
    assert!(keyboard.area_of(CHECK_ALL).is_some());

    let mut pointer = Harness::new(App::new(), Theme::junie(), 120, 40);
    let _ = pointer.key(KeyCode::Char('/'));
    let _ = pointer.type_str("checks");
    let nav = pointer.area_of(NAV).expect("filtered navigation tree");
    let row = (nav.y..nav.bottom())
        .find(|&y| {
            pointer
                .row(y)
                .chars()
                .take(25)
                .collect::<String>()
                .contains("Checks & radios")
        })
        .expect("query leaves the named route visible");
    let _ = pointer.click(nav.x.saturating_add(3), row);
    assert!(format!("{:?}", pointer.app()).contains("nav_target: Placeholder(12)"));
    assert!(pointer.area_of(CHECK_ALL).is_some());
    assert!(
        pointer.diagnostics().is_empty(),
        "{:?}",
        pointer.diagnostics()
    );
}

#[test]
fn live_state_survives_route_away_and_back_and_help_keeps_modal_ownership() {
    let mut h = Harness::new(App::for_page_name("checks"), Theme::junie(), 120, 40);
    let _ = h.click_id(CHECK_NOTIFICATIONS);
    assert!(format!("{:?}", h.app()).contains("checks: [Mixed, Unchecked, Unchecked, Checked]"));
    assert!(
        h.text().contains("Toggle"),
        "checkbox focus gets toggle hints"
    );
    let _ = h.click_part(
        PAGE_TABS,
        termrock::PartRef::item(termrock::Part::TAB, termrock::ItemKey::index(1)),
    );
    let _ = h.click_part(
        OPTION_HIT,
        termrock::PartRef::item(termrock::Part::ROW, termrock::ItemKey::index(1)),
    );
    let _ = h.click_part(
        PAGE_TABS,
        termrock::PartRef::item(termrock::Part::TAB, termrock::ItemKey::index(0)),
    );
    let _ = h.key(KeyCode::Char(']'));
    assert!(format!("{:?}", h.app()).contains("nav_target: Placeholder(13)"));
    let _ = h.key(KeyCode::Char('['));
    let debug = format!("{:?}", h.app());
    assert!(debug.contains("nav_target: Placeholder(12)"));
    assert!(
        debug.contains("hit: 1"),
        "page option survives the route: {debug}"
    );
    assert!(debug.contains("checks: [Mixed, Unchecked, Unchecked, Checked]"));

    let _ = h.click_part(
        GROUP_TABS,
        termrock::PartRef::item(termrock::Part::TAB, termrock::ItemKey::index(1)),
    );
    let _ = h.click_part(
        RADIO_ISOLATION,
        termrock::PartRef::item(termrock::Part::ROW, termrock::ItemKey::index(0)),
    );
    assert!(
        h.text().contains("Choose"),
        "radio focus gets radio hints\n{}",
        h.text()
    );
    let help = h
        .area_of_part(HEADER_HELP, termrock::PartRef::of(termrock::Part::LABEL))
        .expect("visible shell Help control");
    let _ = h.click(help.x.saturating_add(1), help.y);
    assert!(h.is_open(HELP));
    let _ = h.key(KeyCode::Char('x'));
    assert!(h.is_open(HELP));
    assert!(format!("{:?}", h.app()).contains("checks: [Mixed, Unchecked, Unchecked, Checked]"));
    let _ = h.key(KeyCode::Esc);
    assert!(!h.is_open(HELP));
    assert_eq!(h.focus(), Some(RADIO_ISOLATION));
    assert!(h.area_of(RADIO_ISOLATION).is_some());
    assert!(
        h.area_of(CHECK_ALL).is_none(),
        "hidden checkbox controls are not mounted"
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
