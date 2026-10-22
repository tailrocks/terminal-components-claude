use showcase_demos::pages::{Page, checks_radios::ChecksRadiosPage};
use termrock::{App, Cx, Id, ItemKey, KeyCode, Part, PartRef, Rect, Response, Theme, Ui};
use termrock_test_support::Harness;

const PAGE_TABS: Id = Id::root("showcase_ui::pages::checks_radios.page_tabs");
const GROUP_TABS: Id = Id::root("showcase_ui::pages::checks_radios.group_tabs");
const CHECK_ALL: Id = Id::root("showcase_ui::pages::checks_radios.check_all");
const CHECK_NOTIFICATIONS: Id = Id::root("showcase_ui::pages::checks_radios.check_notifications");
const CHECK_ARCHIVED: Id = Id::root("showcase_ui::pages::checks_radios.check_archived");
const CHECK_MANAGED: Id = Id::root("showcase_ui::pages::checks_radios.check_managed");
const RADIO_ISOLATION: Id = Id::root("showcase_ui::pages::checks_radios.radio_isolation");
const RADIO_REGION: Id = Id::root("showcase_ui::pages::checks_radios.radio_region");
const TOGGLE_AUTOSAVE: Id = Id::root("showcase_ui::pages::checks_radios.toggle_autosave");
const TOGGLE_USAGE: Id = Id::root("showcase_ui::pages::checks_radios.toggle_usage");
const TOGGLE_BETA: Id = Id::root("showcase_ui::pages::checks_radios.toggle_beta");
const OPTION_HIT: Id = Id::root("showcase_ui::pages::checks_radios.option_hit");
const OPTION_MIXED: Id = Id::root("showcase_ui::pages::checks_radios.option_mixed");
const OPTION_RADIO_NAV: Id = Id::root("showcase_ui::pages::checks_radios.option_radio_nav");
const OPTION_DISABLED: Id = Id::root("showcase_ui::pages::checks_radios.option_disabled");
const GALLERY_CHECK: Id = Id::root("showcase_ui::pages::checks_radios.gallery.check");
const GALLERY_RADIO: Id = Id::root("showcase_ui::pages::checks_radios.gallery.radio");
const GALLERY_TOGGLE: Id = Id::root("showcase_ui::pages::checks_radios.gallery.toggle");

struct PageHost {
    page: ChecksRadiosPage,
    area: Option<Rect>,
}

impl App for PageHost {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.page.update(cx).response
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        if let Some(area) = self.area {
            let full = ui.full();
            let sentinel = "S".repeat(usize::from(full.width));
            for y in full.y..full.bottom() {
                let _ = ui.paint_str(
                    Rect::new(full.x, y, full.width, 1),
                    &sentinel,
                    ui.surface_style(),
                );
            }
            self.page.draw(ui, area);
        } else {
            self.page.draw(ui, ui.full());
        }
    }
}

fn host(width: u16, height: u16) -> Harness<PageHost> {
    Harness::new(
        PageHost {
            page: ChecksRadiosPage::new(),
            area: None,
        },
        Theme::junie(),
        width,
        height,
    )
}

fn click_tab(h: &mut Harness<PageHost>, id: Id, index: usize) {
    let _ = h.click_part(id, PartRef::item(Part::TAB, ItemKey::index(index)));
}

fn debug(h: &Harness<PageHost>) -> String {
    format!("{:?}", h.app().page)
}

fn values(h: &Harness<PageHost>) -> String {
    debug(h)
        .split_once("hit: ")
        .expect("page debug includes option values")
        .1
        .to_owned()
}

fn cell_run(h: &Harness<PageHost>, x: u16, y: u16, width: u16) -> String {
    (x..x.saturating_add(width))
        .map(|column| h.cell(column, y).symbol())
        .collect()
}

#[test]
fn canonical_defaults_and_mixed_checkbox_cycle_use_real_controls() {
    let mut h = host(120, 40);
    let state = debug(&h);
    assert!(state.contains("hit: 0"), "{state}");
    assert!(state.contains("mixed: true"), "{state}");
    assert!(state.contains("radio_nav: 0"), "{state}");
    assert!(state.contains("globally_disabled: false"), "{state}");
    assert!(
        state.contains("checks: [Mixed, Checked, Unchecked, Checked]"),
        "{state}"
    );
    assert!(state.contains("isolation: 0"), "{state}");
    assert!(state.contains("region: 0"), "{state}");
    assert!(state.contains("toggles: [true, false, false]"), "{state}");
    assert!(h.text().contains("Last: nothing yet"), "{}", h.text());
    assert!(h.area_of(CHECK_ALL).is_some());
    assert!(h.area_of(CHECK_NOTIFICATIONS).is_some());
    assert!(h.area_of(CHECK_ARCHIVED).is_some());
    assert!(h.area_of(CHECK_MANAGED).is_some());

    let _ = h.click_id(CHECK_ALL);
    assert!(debug(&h).contains("checks: [Checked, Checked, Unchecked, Checked]"));
    assert!(h.text().contains("Last: Select all rows"), "{}", h.text());
    let _ = h.click_id(CHECK_ALL);
    assert!(debug(&h).contains("checks: [Unchecked, Checked, Unchecked, Checked]"));
    let _ = h.click_id(CHECK_ALL);
    assert!(debug(&h).contains("checks: [Checked, Checked, Unchecked, Checked]"));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    assert!(h.tab_to(CHECK_NOTIFICATIONS));
    let _ = h.key(KeyCode::Char(' '));
    assert!(debug(&h).contains("checks: [Checked, Unchecked, Unchecked, Checked]"));
    assert!(h.text().contains("Last: Send notifications"));
}

#[test]
fn last_activation_records_its_result_and_survives_page_rebuilds() {
    let mut h = host(120, 40);
    assert!(h.text().contains("Last: nothing yet"));

    let _ = h.click_id(CHECK_ALL);
    assert!(
        h.text().contains("Last: Select all rows checked"),
        "{}",
        h.text()
    );
    let _ = h.click_id(CHECK_ALL);
    assert!(
        h.text().contains("Last: Select all rows unchecked"),
        "{}",
        h.text()
    );

    let _ = h.click_id(CHECK_NOTIFICATIONS);
    assert!(
        h.text().contains("Last: Send notifications unchecked"),
        "pointer activation captures its result\n{}",
        h.text()
    );

    assert!(h.tab_to(CHECK_ARCHIVED));
    let _ = h.key(KeyCode::Enter);
    assert!(
        h.text().contains("Last: Include archived projects checked"),
        "Enter activation captures its result\n{}",
        h.text()
    );

    click_tab(&mut h, PAGE_TABS, 1);
    let _ = h.click_id(OPTION_MIXED);
    assert!(
        debug(&h).contains("checks: [Unchecked, Unchecked, Checked, Checked]"),
        "{:?}",
        h.app().page
    );
    click_tab(&mut h, PAGE_TABS, 0);
    assert!(
        h.text().contains("Last: Include archived projects checked"),
        "option rebuild keeps the last activation snapshot\n{}",
        h.text()
    );

    click_tab(&mut h, GROUP_TABS, 1);
    click_tab(&mut h, GROUP_TABS, 0);
    assert!(
        h.text().contains("Last: Include archived projects checked"),
        "group changes keep the last activation snapshot\n{}",
        h.text()
    );
}

#[test]
fn label_hit_width_rejects_blank_rest_and_row_option_accepts_it() {
    let mut h = host(120, 40);
    let area = h.area_of(CHECK_ALL).expect("live checkbox row");
    assert_eq!(area.width, 5 + termrock::width("Select all rows"));
    let blank_x = area.right().saturating_add(4);
    assert!(blank_x < 120);
    let _ = h.click(blank_x, area.y);
    assert!(debug(&h).contains("checks: [Mixed, Checked, Unchecked, Checked]"));

    click_tab(&mut h, PAGE_TABS, 1);
    assert!(h.area_of(OPTION_HIT).is_some());
    let _ = h.click_part(OPTION_HIT, PartRef::item(Part::ROW, ItemKey::index(1)));
    assert!(debug(&h).contains("hit: 1"));
    click_tab(&mut h, PAGE_TABS, 0);
    let row = h.area_of(CHECK_ALL).expect("row hit area");
    assert_eq!(row.width, 120, "full row uses the available content width");
    let _ = h.click(row.right().saturating_sub(2), row.y);
    assert!(debug(&h).contains("checks: [Checked, Checked, Unchecked, Checked]"));
}

#[test]
fn option_rebuild_resets_only_the_canonical_first_checkbox() {
    let mut h = host(120, 40);
    let _ = h.click_id(CHECK_ALL);
    let _ = h.click_id(CHECK_NOTIFICATIONS);
    click_tab(&mut h, GROUP_TABS, 1);
    let _ = h.click_part(RADIO_ISOLATION, PartRef::item(Part::ROW, ItemKey::index(2)));
    click_tab(&mut h, GROUP_TABS, 2);
    let _ = h.click_id(TOGGLE_AUTOSAVE);
    let _ = h.click_id(TOGGLE_USAGE);
    click_tab(&mut h, PAGE_TABS, 1);
    let _ = h.click_part(OPTION_HIT, PartRef::item(Part::ROW, ItemKey::index(1)));
    let state = debug(&h);
    assert!(state.contains("hit: 1"), "{state}");
    assert!(
        state.contains("checks: [Mixed, Unchecked, Unchecked, Checked]"),
        "{state}"
    );
    assert!(state.contains("isolation: 2"), "{state}");
    assert!(state.contains("region: 0"), "{state}");
    assert!(
        state.contains("cursor: Some(Index(2))"),
        "radio cursor survives: {state}"
    );
    assert!(
        state.contains(
            "region_state: RadioGroupState { core: CollectionCore { cursor: Some(Index(0))"
        ),
        "disabled region selection retains its cursor: {state}"
    );
    assert!(state.contains("toggles: [false, true, false]"), "{state}");
    let _ = h.click_id(OPTION_MIXED);
    let state = debug(&h);
    assert!(state.contains("mixed: false"), "{state}");
    assert!(
        state.contains("checks: [Unchecked, Unchecked, Unchecked, Checked]"),
        "{state}"
    );
}

#[test]
fn radio_policies_are_orthogonal_to_immediate_selection_and_pointer_focus() {
    let mut h = host(120, 40);
    click_tab(&mut h, GROUP_TABS, 1);
    let _ = h.click_part(RADIO_ISOLATION, PartRef::item(Part::ROW, ItemKey::index(0)));
    assert_eq!(h.focus(), Some(RADIO_ISOLATION));
    let _ = h.key(KeyCode::Char('j'));
    assert!(
        debug(&h).contains("isolation: 0"),
        "default is Desktop: {}",
        debug(&h)
    );
    let _ = h.key(KeyCode::End);
    assert!(
        debug(&h).contains("isolation: 2"),
        "Desktop End selects last option"
    );

    click_tab(&mut h, PAGE_TABS, 1);
    let _ = h.click_part(
        OPTION_RADIO_NAV,
        PartRef::item(Part::ROW, ItemKey::index(1)),
    );
    click_tab(&mut h, PAGE_TABS, 0);
    click_tab(&mut h, GROUP_TABS, 1);
    let _ = h.click_part(RADIO_ISOLATION, PartRef::item(Part::ROW, ItemKey::index(0)));
    let _ = h.key(KeyCode::Home);
    let _ = h.key(KeyCode::Char('j'));
    assert!(
        debug(&h).contains("isolation: 1"),
        "Vim j selects next option"
    );
    let _ = h.key(KeyCode::Char('J'));
    assert!(
        debug(&h).contains("isolation: 2"),
        "Vim J selects next option"
    );
    let _ = h.key(KeyCode::Char('k'));
    assert!(
        debug(&h).contains("isolation: 1"),
        "Vim k selects previous option"
    );
    let _ = h.key(KeyCode::Char('K'));
    assert!(
        debug(&h).contains("isolation: 0"),
        "Vim K selects previous option"
    );
    let _ = h.key(KeyCode::Char('G'));
    assert!(
        debug(&h).contains("isolation: 2"),
        "Vim G selects the last option"
    );
    let _ = h.key(KeyCode::Up);
    assert!(
        debug(&h).contains("isolation: 1"),
        "Vim arrows remain available"
    );
    let _ = h.key(KeyCode::Down);
    assert!(
        debug(&h).contains("isolation: 2"),
        "Vim arrows choose immediately"
    );

    click_tab(&mut h, PAGE_TABS, 1);
    let _ = h.click_part(
        OPTION_RADIO_NAV,
        PartRef::item(Part::ROW, ItemKey::index(2)),
    );
    click_tab(&mut h, PAGE_TABS, 0);
    click_tab(&mut h, GROUP_TABS, 1);
    let _ = h.click_part(RADIO_ISOLATION, PartRef::item(Part::ROW, ItemKey::index(1)));
    let before = debug(&h);
    for key in [
        KeyCode::Home,
        KeyCode::End,
        KeyCode::Char('j'),
        KeyCode::Char('k'),
        KeyCode::Char('J'),
        KeyCode::Char('K'),
        KeyCode::Char('g'),
        KeyCode::Char('G'),
    ] {
        let _ = h.key(key);
        assert_eq!(debug(&h), before, "NoHomeEnd rejects {key:?}");
    }
    let _ = h.key(KeyCode::Down);
    assert!(
        debug(&h).contains("isolation: 2"),
        "arrows remain available"
    );
}

#[test]
fn intrinsic_disabled_controls_survive_global_disable_changes() {
    let mut h = host(120, 40);
    assert!(
        h.state_of(CHECK_MANAGED)
            .contains(termrock::StateFlags::DISABLED)
    );
    assert!(!h.ring().contains(CHECK_MANAGED));
    assert!(!h.ring().contains(RADIO_REGION));
    assert!(!h.ring().contains(TOGGLE_BETA));

    click_tab(&mut h, PAGE_TABS, 1);
    assert!(h.area_of(OPTION_DISABLED).is_some());
    let _ = h.click_id(OPTION_DISABLED);
    assert!(debug(&h).contains("globally_disabled: true"));
    click_tab(&mut h, PAGE_TABS, 0);
    let before = debug(&h);
    let _ = h.click_id(CHECK_NOTIFICATIONS);
    assert_eq!(
        debug(&h),
        before,
        "global disabled blocks playground values"
    );
    click_tab(&mut h, GROUP_TABS, 1);
    let before = debug(&h);
    let _ = h.click_part(RADIO_ISOLATION, PartRef::item(Part::ROW, ItemKey::index(2)));
    assert_eq!(debug(&h), before, "global disabled blocks radio changes");
    click_tab(&mut h, GROUP_TABS, 2);
    let before = debug(&h);
    let _ = h.click_id(TOGGLE_AUTOSAVE);
    assert_eq!(debug(&h), before, "global disabled blocks toggle changes");

    click_tab(&mut h, PAGE_TABS, 1);
    assert!(
        !h.state_of(OPTION_MIXED)
            .contains(termrock::StateFlags::DISABLED)
    );
    let _ = h.click_id(OPTION_DISABLED);
    assert!(debug(&h).contains("globally_disabled: false"));
    click_tab(&mut h, PAGE_TABS, 0);
    click_tab(&mut h, GROUP_TABS, 0);
    assert!(
        h.state_of(CHECK_MANAGED)
            .contains(termrock::StateFlags::DISABLED)
    );
    click_tab(&mut h, GROUP_TABS, 1);
    assert!(!h.ring().contains(RADIO_REGION));
    click_tab(&mut h, GROUP_TABS, 2);
    assert!(!h.ring().contains(TOGGLE_BETA));
}

#[test]
fn toggles_and_options_remain_keyboard_operable() {
    let mut h = host(100, 30);
    click_tab(&mut h, GROUP_TABS, 2);
    assert!(h.tab_to(TOGGLE_AUTOSAVE));
    let _ = h.key(KeyCode::Char(' '));
    assert!(debug(&h).contains("toggles: [false, false, false]"));

    click_tab(&mut h, PAGE_TABS, 1);
    assert!(h.tab_to(OPTION_MIXED));
    let _ = h.key(KeyCode::Char(' '));
    let state = debug(&h);
    assert!(state.contains("mixed: false"), "{state}");
    assert!(
        state.contains("checks: [Unchecked, Checked, Unchecked, Checked]"),
        "{state}"
    );

    assert!(h.tab_to(OPTION_DISABLED));
    let _ = h.key(KeyCode::Char(' '));
    assert!(debug(&h).contains("globally_disabled: true"));
    assert!(
        h.area_of(OPTION_MIXED).is_some(),
        "options stay reachable while disabled"
    );
}

#[test]
fn state_gallery_uses_inert_real_widgets_for_all_seven_rows() {
    let mut h = host(80, 24);
    click_tab(&mut h, PAGE_TABS, 2);
    for label in [
        "Rest",
        "Hover",
        "Pressed",
        "Focused",
        "Focused + hover",
        "Disabled",
        "Overflow",
    ] {
        assert!(
            h.text().contains(label),
            "missing gallery state {label}:\n{}",
            h.text()
        );
    }
    assert!(h.text().contains("off"));
    assert!(h.text().contains("radio"));
    assert!(
        h.text().contains("[-]"),
        "mixed checkbox uses its literal marker\n{}",
        h.text()
    );
    assert!(h.area_of(GALLERY_CHECK.index(0)).is_none());
    assert!(h.area_of(GALLERY_CHECK.index(2)).is_none());
    assert!(h.area_of(OPTION_HIT).is_none());
    let label_width = termrock::width("Focused + hover");
    let grid_x = label_width;
    let grid_width = 80 - label_width;
    let column_width = grid_width / 6;
    let remainder = grid_width % 6;
    let column_x = |column: u16| {
        grid_x
            .saturating_add(column.saturating_mul(column_width))
            .saturating_add(column.min(remainder))
    };
    let first_state_y = 6;
    assert_eq!(
        cell_run(&h, column_x(0).saturating_add(1), first_state_y, 3),
        "[ ]"
    );
    assert_eq!(
        cell_run(&h, column_x(1).saturating_add(1), first_state_y, 3),
        "[✓]"
    );
    assert_eq!(
        cell_run(&h, column_x(2).saturating_add(1), first_state_y, 3),
        "[-]"
    );
    assert_eq!(
        cell_run(&h, column_x(3).saturating_add(1), first_state_y, 3),
        "(●)"
    );
    assert_eq!(
        cell_run(&h, column_x(4).saturating_add(1), first_state_y, 3),
        "●──"
    );
    assert_eq!(
        cell_run(&h, column_x(5).saturating_add(1), first_state_y, 3),
        "──●"
    );
    let rest_gutter = h.cell(column_x(0), first_state_y);
    let focused_gutter = h.cell(column_x(0), first_state_y + 3);
    let rest_marker = h.cell(column_x(0).saturating_add(1), first_state_y);
    let disabled_marker = h.cell(column_x(0).saturating_add(1), first_state_y + 5);
    assert_eq!(rest_gutter.symbol(), " ");
    assert_eq!(
        focused_gutter.symbol(),
        "▎",
        "focused rows use the real focus bar"
    );
    assert_ne!(
        rest_gutter.fg, focused_gutter.fg,
        "focus bar has its focus ink"
    );
    assert_ne!(
        rest_marker.fg, disabled_marker.fg,
        "disabled marker has disabled ink"
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn options_and_all_three_playground_groups_fit_at_supported_sizes() {
    for (width, height) in [(72, 20), (80, 24), (100, 30), (120, 40)] {
        let mut h = host(width, height);
        for (index, id) in [
            CHECK_ALL,
            CHECK_NOTIFICATIONS,
            CHECK_ARCHIVED,
            CHECK_MANAGED,
        ]
        .into_iter()
        .enumerate()
        {
            if index != 0 {
                assert!(
                    h.area_of(id).is_some(),
                    "checkbox {index} at {width}x{height}"
                );
            }
        }
        assert!(
            h.tab_to(CHECK_ARCHIVED),
            "checkbox focus at {width}x{height}"
        );
        click_tab(&mut h, GROUP_TABS, 1);
        assert!(
            h.area_of(RADIO_ISOLATION).is_some(),
            "isolation at {width}x{height}"
        );
        assert!(
            h.area_of(RADIO_REGION).is_some(),
            "disabled region group at {width}x{height}"
        );
        assert!(h.tab_to(RADIO_ISOLATION), "radio focus at {width}x{height}");
        click_tab(&mut h, GROUP_TABS, 2);
        assert!(
            h.area_of(TOGGLE_AUTOSAVE).is_some(),
            "toggles at {width}x{height}"
        );
        assert!(
            h.tab_to(TOGGLE_AUTOSAVE),
            "toggle focus at {width}x{height}"
        );
        click_tab(&mut h, PAGE_TABS, 1);
        for id in [OPTION_HIT, OPTION_MIXED, OPTION_RADIO_NAV, OPTION_DISABLED] {
            assert!(
                h.area_of(id).is_some(),
                "{id:?} at {width}x{height}\n{}",
                h.text()
            );
            assert!(h.tab_to(id), "{id:?} focus at {width}x{height}");
        }
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
}

#[test]
fn focused_control_stays_registered_and_visible_across_resizes() {
    let mut h = host(72, 20);
    assert!(h.tab_to(CHECK_NOTIFICATIONS));
    for (width, height) in [(80, 24), (100, 30), (120, 40), (72, 20)] {
        let _ = h.resize(width, height);
        assert_eq!(h.focus(), Some(CHECK_NOTIFICATIONS));
        let area = h
            .area_of(CHECK_NOTIFICATIONS)
            .expect("focused control is mounted");
        assert!(area.right() <= width && area.bottom() <= height);
        assert!(
            h.state_of(CHECK_NOTIFICATIONS)
                .contains(termrock::StateFlags::FOCUSED)
        );
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
}

#[test]
fn offset_clipped_page_keeps_all_controls_inside_its_page_rect() {
    let area = Rect::new(7, 3, 56, 18);
    let mut h = Harness::new(
        PageHost {
            page: ChecksRadiosPage::new(),
            area: Some(area),
        },
        Theme::junie(),
        80,
        24,
    );
    let checkbox = h.area_of(CHECK_ALL).expect("visible checkbox");
    assert!(checkbox.x >= area.x && checkbox.right() <= area.right());
    assert!(checkbox.y >= area.y && checkbox.bottom() <= area.bottom());
    click_tab(&mut h, PAGE_TABS, 1);
    for id in [OPTION_HIT, OPTION_MIXED, OPTION_RADIO_NAV, OPTION_DISABLED] {
        let rect = h.area_of(id).expect("offset option control");
        assert!(
            rect.x >= area.x && rect.right() <= area.right(),
            "{id:?}: {rect:?}"
        );
        assert!(
            rect.y >= area.y && rect.bottom() <= area.bottom(),
            "{id:?}: {rect:?}"
        );
    }
    click_tab(&mut h, PAGE_TABS, 0);
    click_tab(&mut h, GROUP_TABS, 1);
    for id in [RADIO_ISOLATION, RADIO_REGION] {
        let rect = h.area_of(id).expect("offset radio group");
        assert!(
            rect.x >= area.x && rect.right() <= area.right(),
            "{id:?}: {rect:?}"
        );
        assert!(
            rect.y >= area.y && rect.bottom() <= area.bottom(),
            "{id:?}: {rect:?}"
        );
    }
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn short_offset_areas_clip_every_page_view_and_group() {
    let ids = [
        PAGE_TABS,
        GROUP_TABS,
        CHECK_ALL,
        CHECK_NOTIFICATIONS,
        CHECK_ARCHIVED,
        CHECK_MANAGED,
        RADIO_ISOLATION,
        RADIO_REGION,
        TOGGLE_AUTOSAVE,
        TOGGLE_USAGE,
        TOGGLE_BETA,
        OPTION_HIT,
        OPTION_MIXED,
        OPTION_RADIO_NAV,
        OPTION_DISABLED,
        GALLERY_CHECK.index(0),
        GALLERY_CHECK.index(1),
        GALLERY_CHECK.index(2),
        GALLERY_RADIO,
        GALLERY_TOGGLE.index(0),
        GALLERY_TOGGLE.index(1),
    ];
    let views = [(0, 0), (0, 1), (0, 2), (1, 0), (2, 0)];

    for (view, group) in views {
        let mut h = host(80, 24);
        if view != 0 {
            click_tab(&mut h, PAGE_TABS, view);
        } else if group != 0 {
            click_tab(&mut h, GROUP_TABS, group);
        }

        for height in 0..=17 {
            let area = Rect::new(7, 3, 56, height);
            h.app_mut().area = Some(area);
            h.draw();

            let sentinel = h.cell(0, 0).clone();
            for y in 0..24 {
                for x in 0..80 {
                    let inside =
                        x >= area.x && x < area.right() && y >= area.y && y < area.bottom();
                    if !inside {
                        assert_eq!(
                            h.cell(x, y),
                            &sentinel,
                            "outside cell changed for view={view}, group={group}, height={height}, cell=({x},{y})"
                        );
                    }
                }
            }

            for id in ids {
                if let Some(rect) = h.area_of(id) {
                    assert!(
                        rect.width > 0
                            && rect.height > 0
                            && rect.x >= area.x
                            && rect.right() <= area.right()
                            && rect.y >= area.y
                            && rect.bottom() <= area.bottom(),
                        "{id:?} escaped {area:?}: {rect:?}, view={view}, group={group}, height={height}"
                    );
                }
            }
            if let Some(focus) = h.focus() {
                let rect = h.area_of(focus).expect("focused control stays mounted");
                assert!(
                    rect.x >= area.x
                        && rect.right() <= area.right()
                        && rect.y >= area.y
                        && rect.bottom() <= area.bottom(),
                    "focus {focus:?} escaped {area:?}: {rect:?}"
                );
            }

            if view == 0 && group == 0 && height >= 13 {
                assert!(h.area_of(CHECK_MANAGED).is_some());
            }
            if view == 0 && group == 1 && height >= 17 {
                assert!(h.area_of(RADIO_REGION).is_some());
            }
            if view == 0 && group == 2 && height >= 12 {
                assert!(h.area_of(TOGGLE_BETA).is_some());
            }
            if view == 1 && height >= 11 {
                for id in [OPTION_HIT, OPTION_MIXED, OPTION_RADIO_NAV, OPTION_DISABLED] {
                    assert!(h.area_of(id).is_some(), "{id:?} at height={height}");
                }
            }
            if view == 2 && height >= 13 {
                assert!(
                    h.text().contains("Overflow"),
                    "height={height}\n{}",
                    h.text()
                );
            }
            assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());

            let before = values(&h);
            let _ = h.click(area.x.saturating_add(1), area.bottom());
            assert_eq!(values(&h), before, "outside click changed page values");
        }
    }
}

#[test]
fn checked_default_is_the_real_component_state() {
    let h = host(72, 20);
    let check = h.area_of(CHECK_NOTIFICATIONS).expect("checkbox drawn");
    assert!(h.row(check.y).contains("[✓]"), "{}", h.row(check.y));
    let managed = h
        .area_of(CHECK_MANAGED)
        .expect("intrinsic disabled checkbox drawn");
    assert!(
        h.state_of(CHECK_MANAGED)
            .contains(termrock::StateFlags::DISABLED)
    );
    assert!(h.row(managed.y).contains("[✓]"), "{}", h.row(managed.y));
    let mixed = h.area_of(CHECK_ALL).expect("mixed checkbox drawn");
    assert!(h.row(mixed.y).contains("[-]"), "{}", h.row(mixed.y));
}
