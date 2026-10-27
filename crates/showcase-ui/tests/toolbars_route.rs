use showcase_ui::{App, NAV_ENTRIES, PageId};
use termrock::{Id, ItemKey, KeyCode, KeyModifiers, Part, PartRef, Theme};
use termrock_test_support::Harness;

const PAGE_TABS: Id = Id::root("showcase_ui::pages::toolbars.page_tabs");
const QUERY: Id = Id::root("showcase_ui::pages::toolbars.query");
const NARROW: Id = Id::root("showcase_ui::pages::toolbars.narrow");
const FORMAT: Id = Id::root("showcase_ui::pages::toolbars.format");
const NAV_OPTION: Id = Id::root("showcase_ui::pages::toolbars.option.nav");
const WRAP_OPTION: Id = Id::root("showcase_ui::pages::toolbars.option.wrap");
const SKIP_OPTION: Id = Id::root("showcase_ui::pages::toolbars.option.skip_disabled");
const GAP_OPTION: Id = Id::root("showcase_ui::pages::toolbars.option.gap");
const OVERFLOW_OPTION: Id = Id::root("showcase_ui::pages::toolbars.option.overflow");
const STATE_REFERENCE: Id = Id::root("showcase_ui::pages::toolbars.states");

fn query_run() -> ItemKey {
    ItemKey::text("query.run")
}

fn query_wrap() -> ItemKey {
    ItemKey::text("query.wrap")
}

fn toolbar_item(key: ItemKey) -> PartRef {
    PartRef::item(Part::ROW, key)
}

fn menu_id(id: Id) -> Id {
    id.sub("more-menu")
}

fn menu_row(h: &Harness<App>, owner: Id, label: &str) -> Option<(PartRef, termrock::Rect, String)> {
    let menu = menu_id(owner);
    (0..16).find_map(|index| {
        let part = PartRef::item(Part::ROW, ItemKey::index(index));
        let area = h.area_of_part(menu, part)?;
        let text = rect_text(h, area.x, area.y, area.width);
        text.contains(label).then_some((part, area, text))
    })
}

fn rect_text(h: &Harness<App>, x: u16, y: u16, width: u16) -> String {
    let mut text = String::new();
    for column in x..x.saturating_add(width) {
        text.push_str(h.cell(column, y).symbol());
    }
    text
}

fn click_menu_item(h: &mut Harness<App>, owner: Id, label: &str) {
    let menu = menu_id(owner);
    assert!(h.is_open(menu), "native More menu for {owner:?} is open");
    let (part, _, _) = menu_row(h, owner, label)
        .unwrap_or_else(|| panic!("native menu row {label:?} was not published\n{}", h.text()));
    let _ = h.click_part(menu, part);
}

fn click_toolbar_item(h: &mut Harness<App>, owner: Id, key: ItemKey, label: &str) {
    if h.area_of_part(owner, toolbar_item(key)).is_some() {
        let _ = h.click_part(owner, toolbar_item(key));
        return;
    }
    assert!(
        h.area_of_part(owner, PartRef::of(Part::OVERFLOW)).is_some(),
        "a hidden action has a native More part"
    );
    let _ = h.click_part(owner, PartRef::of(Part::OVERFLOW));
    click_menu_item(h, owner, label);
}

fn select_option(h: &mut Harness<App>, id: Id, index: usize) {
    assert!(h.tab_to(id), "option {id:?} is in the public focus ring");
    let _ = h.key(KeyCode::Enter);
    let _ = h.key(KeyCode::Home);
    for _ in 0..index {
        let _ = h.key(KeyCode::Down);
    }
    let _ = h.key(KeyCode::Enter);
}

#[test]
fn route_migrates_only_slot_thirteen_and_keeps_the_native_toolbar_owners() {
    let h = Harness::new(App::for_page_name("Toolbars"), Theme::junie(), 120, 40);

    assert_eq!(NAV_ENTRIES.len(), 54);
    assert_eq!(NAV_ENTRIES[13].label, "Toolbars");
    assert_eq!(NAV_ENTRIES[13].section, "Inputs & forms");
    assert_eq!(NAV_ENTRIES[13].id, PageId::Toolbars);
    assert_eq!(PageId::ALL.len(), 26);
    assert_eq!(PageId::CellEditors.index(), 23);
    assert_eq!(PageId::Toolbars.index(), 24);
    assert_eq!(PageId::from_name("Toolbars"), Some(PageId::Toolbars));
    assert_eq!(PageId::from_name("toolbars"), Some(PageId::Toolbars));
    assert_eq!(h.app().page(), PageId::Toolbars);
    assert!(
        h.text()
            .contains("A row of commands that is one focus stop")
    );
    for label in ["Playground", "Options", "States"] {
        assert!(
            h.text().contains(label),
            "missing section {label}\n{}",
            h.text()
        );
    }
    for label in [
        "Query toolbar",
        "Narrow toolbar",
        "Formatting toolbar",
        "Last",
    ] {
        assert!(
            h.text().contains(label),
            "missing card {label}\n{}",
            h.text()
        );
    }
    for owner in [QUERY, NARROW, FORMAT] {
        assert!(h.area_of(owner).is_some(), "native Toolbar owner {owner:?}");
        assert_eq!(
            h.ring()
                .entries()
                .iter()
                .filter(|entry| entry.id == owner)
                .count(),
            1,
            "each row is one native focus stop"
        );
    }
    assert_eq!(h.area_of(NARROW).map(|area| area.width), Some(32));
    assert!(h.area_of(PAGE_TABS).is_some());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn tab_and_shift_tab_treat_each_toolbar_as_one_native_focus_stop() {
    let mut h = Harness::new(App::for_page_name("Toolbars"), Theme::junie(), 120, 40);
    assert!(h.tab_to(QUERY));
    assert_eq!(h.focus(), Some(QUERY));
    assert!(h.row(39).contains("Run"));

    let _ = h.key(KeyCode::Tab);
    assert_eq!(h.focus(), Some(NARROW));
    assert!(h.row(39).contains("Run"));
    let _ = h.key(KeyCode::Tab);
    assert_eq!(h.focus(), Some(FORMAT));
    assert!(h.row(39).contains("Toggle"));
    let _ = h.key_mod(KeyCode::Tab, KeyModifiers::SHIFT);
    assert_eq!(h.focus(), Some(NARROW));
    let _ = h.key_mod(KeyCode::Tab, KeyModifiers::SHIFT);
    assert_eq!(h.focus(), Some(QUERY));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn real_toolbar_input_commits_run_state_and_refuses_disabled_activation() {
    let mut theme = Theme::junie();
    theme.design.size.popup_min_width = 25;
    let mut h = Harness::new(App::for_page_name("Toolbars"), theme, 120, 40);
    let _ = h.click_part(NARROW, PartRef::of(Part::OVERFLOW));
    click_menu_item(&mut h, NARROW, "Stop");
    assert!(
        h.is_open(menu_id(NARROW)),
        "disabled Stop leaves the menu open"
    );
    assert!(h.text().contains("No query is running"));
    assert!(
        h.text()
            .contains("none yet · Activated(i) or Toggled(i, on)")
    );
    let _ = h.key(KeyCode::Esc);

    assert!(h.tab_to(QUERY));
    let _ = h.key(KeyCode::Right);
    assert_eq!(
        h.focus(),
        Some(QUERY),
        "arrows stay inside the Toolbar stop"
    );
    click_toolbar_item(&mut h, QUERY, query_run(), "Run");
    assert!(h.text().contains("running · Stop is enabled, Run waits"));
    assert!(h.text().contains("Activated(0) · Run"));

    let before_disabled_run = h.find("Activated(0) · Run");
    click_toolbar_item(&mut h, QUERY, query_run(), "Run");
    assert_eq!(
        h.find("Activated(0) · Run"),
        before_disabled_run,
        "a disabled Run row cannot produce another action"
    );
    assert!(h.text().contains("running · Stop is enabled, Run waits"));
    assert!(h.area_of_part(QUERY, toolbar_item(query_run())).is_some());

    let _ = h.click_part(NARROW, PartRef::of(Part::OVERFLOW));
    click_menu_item(&mut h, NARROW, "Stop");
    assert!(
        h.text()
            .contains("idle · Stop is disabled until a run starts")
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn non_run_commands_leave_running_state_unchanged_and_run_all_uses_the_shared_policy() {
    let mut h = Harness::new(App::for_page_name("Toolbars"), Theme::junie(), 120, 40);
    for (key, label, index) in [
        (ItemKey::text("query.format"), "Format", 3),
        (ItemKey::text("query.explain"), "Explain", 4),
    ] {
        click_toolbar_item(&mut h, QUERY, key, label);
        assert!(
            h.text()
                .contains("idle · Stop is disabled until a run starts")
        );
        assert!(h.text().contains(&format!("Activated({index}) · {label}")));
    }

    click_toolbar_item(&mut h, NARROW, ItemKey::text("query.run_all"), "Run all");
    assert!(h.text().contains("running · Stop is enabled, Run waits"));
    assert!(h.text().contains("Activated(1) · Run all"));
    click_toolbar_item(&mut h, QUERY, ItemKey::text("query.stop"), "Stop");
    assert!(
        h.text()
            .contains("idle · Stop is disabled until a run starts")
    );
    assert!(h.text().contains("Activated(2) · Stop"));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn query_and_narrow_keep_independent_toggles_across_native_actions() {
    let mut h = Harness::new(App::for_page_name("Toolbars"), Theme::junie(), 120, 40);
    let narrow_more = PartRef::of(Part::OVERFLOW);
    assert!(h.area_of_part(NARROW, narrow_more).is_some());

    let _ = h.click_part(NARROW, narrow_more);
    assert!(h.is_open(menu_id(NARROW)));
    let initial_narrow_wrap = menu_row(&h, NARROW, "Wrap").map(|(_, _, text)| text);
    let initial_narrow_line_numbers = menu_row(&h, NARROW, "Line numbers").map(|(_, _, text)| text);
    assert!(
        initial_narrow_wrap.is_some(),
        "Wrap is natively published in More"
    );
    assert!(
        initial_narrow_line_numbers.is_some(),
        "Line numbers is natively published in More"
    );
    let _ = h.key(KeyCode::Esc);

    click_toolbar_item(&mut h, QUERY, query_wrap(), "Wrap");
    assert!(h.text().contains("Toggled(5, true) · Wrap"));
    click_toolbar_item(
        &mut h,
        QUERY,
        ItemKey::text("query.line_numbers"),
        "Line numbers",
    );
    assert!(h.text().contains("Toggled(6, false) · Line numbers"));

    let _ = h.click_part(NARROW, narrow_more);
    assert_eq!(
        menu_row(&h, NARROW, "Wrap").map(|(_, _, text)| text),
        initial_narrow_wrap
    );
    assert_eq!(
        menu_row(&h, NARROW, "Line numbers").map(|(_, _, text)| text),
        initial_narrow_line_numbers,
        "Query's Line numbers toggle does not change Narrow"
    );
    click_menu_item(&mut h, NARROW, "Wrap");
    assert!(h.text().contains("Toggled(5, true) · Wrap"));

    let _ = h.click_part(NARROW, narrow_more);
    let narrow_wrap_on = menu_row(&h, NARROW, "Wrap").map(|(_, _, text)| text);
    assert!(narrow_wrap_on.is_some());
    click_menu_item(&mut h, NARROW, "Line numbers");
    assert!(h.text().contains("Toggled(6, false) · Line numbers"));
    let _ = h.click_part(NARROW, narrow_more);
    let narrow_line_numbers_off = menu_row(&h, NARROW, "Line numbers").map(|(_, _, text)| text);
    assert_ne!(narrow_line_numbers_off, initial_narrow_line_numbers);
    let _ = h.key(KeyCode::Esc);

    click_toolbar_item(&mut h, QUERY, query_run(), "Run");
    let _ = h.click_part(NARROW, narrow_more);
    click_menu_item(&mut h, NARROW, "Stop");
    assert!(
        h.text()
            .contains("idle · Stop is disabled until a run starts")
    );
    assert!(h.text().contains("behind More:"));

    let _ = h.click_part(NARROW, narrow_more);
    let retained_narrow_wrap = menu_row(&h, NARROW, "Wrap").map(|(_, _, text)| text);
    let retained_narrow_line_numbers =
        menu_row(&h, NARROW, "Line numbers").map(|(_, _, text)| text);
    assert_eq!(retained_narrow_wrap, narrow_wrap_on);
    assert_eq!(retained_narrow_line_numbers, narrow_line_numbers_off);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn formatting_toggles_are_independent_and_gallery_references_do_not_register() {
    let mut h = Harness::new(App::for_page_name("Toolbars"), Theme::junie(), 120, 40);
    for (index, label, on) in [
        (0, "Bold", true),
        (1, "Italic", true),
        (2, "Code", true),
        (3, "Left", false),
        (4, "Center", true),
        (5, "Right", true),
    ] {
        click_toolbar_item(
            &mut h,
            FORMAT,
            ItemKey::text(&format!("format.{}", label.to_lowercase())),
            label,
        );
        assert!(
            h.text()
                .contains(&format!("Toggled({index}, {on}) · {label}")),
            "format value proposal for {label} was not committed: {}",
            h.text()
        );
    }

    let _ = h.click_part(PAGE_TABS, PartRef::item(Part::TAB, ItemKey::index(2)));
    for state in [
        "Rest",
        "Hover",
        "Pressed",
        "Focused",
        "FocusedHover",
        "Selected",
        "Disabled",
        "Overflow",
        "Narrow",
    ] {
        assert!(
            h.text().contains(state),
            "gallery row {state}\n{}",
            h.text()
        );
    }
    assert!(h.text().contains("Commands"));
    assert!(h.text().contains("Toggles"));
    assert!(h.area_of(STATE_REFERENCE.index(0)).is_none());
    assert!(h.area_of(STATE_REFERENCE.index(1)).is_none());
    assert!(
        !h.ring().entries().iter().any(|entry| {
            entry.id == STATE_REFERENCE.index(0) || entry.id == STATE_REFERENCE.index(1)
        }),
        "reference rows do not create native focus stops"
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn native_selects_rebuild_toolbar_options_without_losing_route_or_controls() {
    let mut h = Harness::new(App::for_page_name("Toolbars"), Theme::junie(), 120, 40);
    let run = h
        .area_of_part(QUERY, toolbar_item(query_run()))
        .expect("the Query Run item is published");
    let run_all = h
        .area_of_part(QUERY, toolbar_item(ItemKey::text("query.run_all")))
        .expect("the Query Run all item is published");
    assert_eq!(run_all.x.saturating_sub(run.x.saturating_add(run.width)), 1);
    let default_more = h
        .area_of_part(NARROW, PartRef::of(Part::OVERFLOW))
        .expect("the default overflow glyph is published by the native Toolbar");
    assert!(rect_text(&h, default_more.x, default_more.y, default_more.width).contains("»"));
    let _ = h.click_part(PAGE_TABS, PartRef::item(Part::TAB, ItemKey::index(1)));
    assert!(h.area_of(NAV_OPTION).is_some());

    select_option(&mut h, NAV_OPTION, 1);
    assert!(h.text().contains("Vim"));
    let nav = h.area_of(NAV_OPTION).expect("native Navigation select");
    assert!(rect_text(&h, nav.x, nav.y, nav.width).contains("Vim"));
    select_option(&mut h, WRAP_OPTION, 1);
    assert!(h.text().contains("Wrap at row ends"));
    let wrap = h.area_of(WRAP_OPTION).expect("native Wrap select");
    assert!(rect_text(&h, wrap.x, wrap.y, wrap.width).contains("on"));
    select_option(&mut h, SKIP_OPTION, 0);
    assert!(h.text().contains("Skip disabled items"));
    let skip = h.area_of(SKIP_OPTION).expect("native Skip disabled select");
    assert!(rect_text(&h, skip.x, skip.y, skip.width).contains("off"));
    select_option(&mut h, GAP_OPTION, 2);
    assert!(h.text().contains("Gap between items"));
    let gap = h.area_of(GAP_OPTION).expect("native Gap select");
    assert!(rect_text(&h, gap.x, gap.y, gap.width).contains("2"));
    select_option(&mut h, OVERFLOW_OPTION, 1);
    assert!(h.text().contains("Overflow button"));
    let overflow = h.area_of(OVERFLOW_OPTION).expect("native Overflow select");
    assert!(rect_text(&h, overflow.x, overflow.y, overflow.width).contains("…"));
    let _ = h.click_part(PAGE_TABS, PartRef::item(Part::TAB, ItemKey::index(0)));
    assert!(h.text().contains("… holds what does not fit"));
    let run = h
        .area_of_part(QUERY, toolbar_item(query_run()))
        .expect("the Query Run item remains published");
    let run_all = h
        .area_of_part(QUERY, toolbar_item(ItemKey::text("query.run_all")))
        .expect("the Query Run all item remains published");
    assert_eq!(run_all.x.saturating_sub(run.x.saturating_add(run.width)), 2);
    let more = h
        .area_of_part(NARROW, PartRef::of(Part::OVERFLOW))
        .expect("ellipsis selection keeps the native More part");
    let toolbar = h.area_of(NARROW).expect("the Narrow Toolbar is published");
    assert!(more.x >= toolbar.x);
    assert!(more.y >= toolbar.y);
    assert!(more.x.saturating_add(more.width) <= toolbar.x.saturating_add(toolbar.width));
    assert!(more.y.saturating_add(more.height) <= toolbar.y.saturating_add(toolbar.height));
    assert!(rect_text(&h, more.x, more.y, more.width).contains("…"));

    let _ = h.click_part(PAGE_TABS, PartRef::item(Part::TAB, ItemKey::index(1)));
    select_option(&mut h, OVERFLOW_OPTION, 2);
    assert!(h.text().contains("Overflow button"));
    let _ = h.click_part(PAGE_TABS, PartRef::item(Part::TAB, ItemKey::index(0)));
    assert!(h.text().contains("overflow off: no More button"));
    assert!(h.area_of(QUERY).is_some());
    assert!(h.area_of(NARROW).is_some());
    assert!(h.area_of(FORMAT).is_some());
    assert_eq!(h.area_of(NARROW).map(|area| area.width), Some(32));
    assert!(
        h.area_of_part(NARROW, PartRef::of(Part::OVERFLOW))
            .is_none()
    );
    click_toolbar_item(&mut h, QUERY, query_run(), "Run");
    assert!(h.text().contains("running · Stop is enabled, Run waits"));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn vim_keys_and_wrap_options_change_real_toolbar_navigation() {
    let mut h = Harness::new(App::for_page_name("Toolbars"), Theme::junie(), 120, 40);
    let _ = h.click_part(PAGE_TABS, PartRef::item(Part::TAB, ItemKey::index(1)));
    select_option(&mut h, NAV_OPTION, 1);
    let _ = h.click_part(PAGE_TABS, PartRef::item(Part::TAB, ItemKey::index(0)));

    assert!(h.tab_to(QUERY));
    let _ = h.key(KeyCode::Home);
    let _ = h.key(KeyCode::Char('l'));
    let _ = h.key(KeyCode::Enter);
    assert!(h.text().contains("Activated(1) · Run all"));
    assert!(h.text().contains("running · Stop is enabled, Run waits"));
    click_toolbar_item(&mut h, QUERY, ItemKey::text("query.stop"), "Stop");

    let _ = h.click_part(PAGE_TABS, PartRef::item(Part::TAB, ItemKey::index(1)));
    select_option(&mut h, WRAP_OPTION, 1);
    let _ = h.click_part(PAGE_TABS, PartRef::item(Part::TAB, ItemKey::index(0)));
    assert!(h.tab_to(QUERY));
    let _ = h.key(KeyCode::End);
    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Enter);
    assert!(h.text().contains("Activated(0) · Run"));
    assert!(h.text().contains("running · Stop is enabled, Run waits"));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn desktop_navigation_skips_the_disabled_stop_item_by_default() {
    let mut h = Harness::new(App::for_page_name("Toolbars"), Theme::junie(), 120, 40);
    assert!(h.tab_to(QUERY));
    let _ = h.key(KeyCode::Home);
    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Enter);
    assert!(h.text().contains("Activated(3) · Format"));
    assert!(
        h.text()
            .contains("idle · Stop is disabled until a run starts")
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn disabled_items_remain_unactivatable_when_skip_disabled_is_off() {
    let mut h = Harness::new(App::for_page_name("Toolbars"), Theme::junie(), 120, 40);
    let _ = h.click_part(PAGE_TABS, PartRef::item(Part::TAB, ItemKey::index(1)));
    select_option(&mut h, SKIP_OPTION, 0);
    let _ = h.click_part(PAGE_TABS, PartRef::item(Part::TAB, ItemKey::index(0)));

    click_toolbar_item(&mut h, QUERY, query_run(), "Run");
    assert!(h.text().contains("running · Stop is enabled, Run waits"));
    assert!(h.tab_to(QUERY));
    let _ = h.key(KeyCode::Home);
    let _ = h.key(KeyCode::Right);
    let before_disabled_activation = h.find("Activated(0) · Run");
    let _ = h.key(KeyCode::Enter);
    assert_eq!(
        h.find("Activated(0) · Run"),
        before_disabled_activation,
        "keyboard navigation can reach disabled Run all without activating it"
    );
    assert!(h.text().contains("running · Stop is enabled, Run waits"));

    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Enter);
    assert!(h.text().contains("Activated(2) · Stop"));
    assert!(
        h.text()
            .contains("idle · Stop is disabled until a run starts")
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn end_opens_native_more_and_escape_returns_focus_to_the_toolbar() {
    let mut h = Harness::new(App::for_page_name("Toolbars"), Theme::junie(), 120, 40);
    assert!(h.tab_to(NARROW));
    let _ = h.key(KeyCode::End);
    assert!(h.row(39).contains("Show hidden"));
    let _ = h.key(KeyCode::Enter);
    assert!(h.is_open(menu_id(NARROW)));
    assert!(h.row(39).contains("Close"));
    let _ = h.key(KeyCode::Esc);
    assert!(!h.is_open(menu_id(NARROW)));
    assert_eq!(h.focus(), Some(NARROW));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn keyboard_menu_navigation_selects_a_hidden_toggle_by_its_native_action() {
    let mut h = Harness::new(App::for_page_name("Toolbars"), Theme::junie(), 120, 40);
    assert!(h.tab_to(NARROW));
    let _ = h.key(KeyCode::End);
    let _ = h.key(KeyCode::Enter);
    assert!(h.is_open(menu_id(NARROW)));
    assert!(menu_row(&h, NARROW, "Line numbers").is_some());

    let _ = h.key(KeyCode::End);
    let _ = h.key(KeyCode::Enter);
    assert!(h.text().contains("Toggled(6, false) · Line numbers"));
    assert!(!h.is_open(menu_id(NARROW)));
    assert_eq!(h.focus(), Some(NARROW));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn outside_pointer_input_dismisses_the_native_more_menu() {
    let mut h = Harness::new(App::for_page_name("Toolbars"), Theme::junie(), 120, 40);
    let _ = h.click_part(NARROW, PartRef::of(Part::OVERFLOW));
    assert!(h.is_open(menu_id(NARROW)));
    let _ = h.click_part(PAGE_TABS, PartRef::item(Part::TAB, ItemKey::index(1)));
    assert!(!h.is_open(menu_id(NARROW)));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn compact_and_shell_minimum_routes_render_the_native_controls_without_diagnostics() {
    for (width, height) in [(80, 24), (72, 20)] {
        let h = Harness::new(
            App::for_page_name("Toolbars"),
            Theme::junie(),
            width,
            height,
        );
        assert_eq!(h.app().page(), PageId::Toolbars);
        assert!(h.text().contains("Toolbars"));
        assert!(h.area_of(QUERY).is_some());
        assert!(
            h.diagnostics().is_empty(),
            "{width}x{height}: {:?}",
            h.diagnostics()
        );
    }
}
