use showcase_ui::app::NavExpand;
use showcase_ui::{App, NAV_ENTRIES, PageId};
use termrock::{Id, ItemKey, KeyCode, Part, PartRef, Theme, width};
use termrock_test_support::Harness;

const NAV: Id = Id::root("showcase_ui::app::navigation");
const HEADER_HELP: Id = Id::root("showcase_ui::app::header.help");
const NAV_GROUP_KEYSPACE: u64 = 0x4e41_5647_524f_5550;

const POLICIES: [NavExpand; 3] = [
    NavExpand::All,
    NavExpand::Accordion,
    NavExpand::CurrentGroup,
];

const GROUP_CHILDREN: [(&str, &str); 9] = [
    ("App shell", "Chrome"),
    ("Inputs & forms", "Buttons"),
    ("Tables", "Editable tables"),
    ("Navigation", "Lists"),
    ("Overlays", "Dialogs"),
    ("Feedback", "Progress"),
    ("Layout", "Panels"),
    ("Text & code", "Code editor"),
    ("Screens", "Settings"),
];

fn harness(app: App) -> Harness<App> {
    Harness::new(app, Theme::junie(), 120, 80)
}

fn row(h: &Harness<App>, label: &str) -> Option<(u16, u16)> {
    let area = h.area_of(NAV)?;
    (area.y..area.bottom()).find_map(|y| {
        let text = (area.x..area.right())
            .map(|x| h.cell(x, y).symbol())
            .collect::<String>();
        text.find(label)
            .map(|byte| (area.x.saturating_add(width(&text[..byte])), y))
    })
}

fn visible(h: &Harness<App>, label: &str) -> bool {
    row(h, label).is_some()
}

fn set_policy(h: &mut Harness<App>, policy: NavExpand) {
    h.app_mut().set_nav_expand(policy);
    h.draw();
}

fn click_group(h: &mut Harness<App>, label: &str) {
    let (x, y) = row(h, label)
        .unwrap_or_else(|| panic!("navigation group {label:?} is not visible\n{}", h.text()));
    let _ = h.click(x.saturating_add(1), y);
}

fn click_entry(h: &mut Harness<App>, label: &str) {
    let (x, y) = row(h, label)
        .unwrap_or_else(|| panic!("navigation entry {label:?} is not visible\n{}", h.text()));
    let _ = h.click(x, y);
}

fn advance_route(h: &mut Harness<App>, from: &str, to: &str) {
    let index = |label: &str| {
        NAV_ENTRIES
            .iter()
            .position(|entry| entry.label == label)
            .unwrap_or_else(|| panic!("missing catalog entry {label:?}"))
    };
    let from = index(from);
    let to = index(to);
    let (key, steps) = if to >= from {
        (']', to - from)
    } else {
        ('[', from - to)
    };
    for _ in 0..steps {
        let _ = h.key(KeyCode::Char(key));
    }
}

fn group_is_expanded(h: &Harness<App>, group: &str) -> bool {
    let group_index = GROUP_CHILDREN
        .iter()
        .position(|(candidate, _)| *candidate == group)
        .unwrap_or_else(|| panic!("unknown navigation group {group:?}"));
    let key = format!(
        "{:?}",
        ItemKey::pair(NAV_GROUP_KEYSPACE, group_index as u64)
    );
    let debug = format!("{:?}", h.app());
    let expanded = debug
        .split("expanded: ")
        .nth(1)
        .and_then(|tail| tail.split(", chosen").next())
        .unwrap_or_else(|| panic!("expanded set missing from app debug: {debug}"));
    if expanded.starts_with("AllExcept") {
        !expanded.contains(&key)
    } else {
        expanded.contains(&key)
    }
}

fn assert_expanded_groups(h: &Harness<App>, expanded: &[&str]) {
    for (group, _) in GROUP_CHILDREN {
        assert_eq!(
            group_is_expanded(h, group),
            expanded.contains(&group),
            "saved fold membership for {group:?}: {:?}",
            h.app()
        );
    }
}

fn assert_groups(h: &Harness<App>, expanded: &[&str]) {
    for (group, child) in GROUP_CHILDREN {
        assert_eq!(
            visible(h, child),
            expanded.contains(&group),
            "group {group:?} child {child:?} visibility\n{}",
            h.text()
        );
    }
}

fn query_active(h: &Harness<App>) -> bool {
    format!("{:?}", h.app()).contains("nav_query: Some(")
}

fn query_text(h: &Harness<App>) -> String {
    let debug = format!("{:?}", h.app());
    debug
        .split("nav_query: Some(NavQuery { text: ")
        .nth(1)
        .and_then(|tail| tail.split_once(',').map(|(text, _)| text))
        .unwrap_or_else(|| panic!("query missing from app state: {debug}"))
        .trim_matches('"')
        .to_owned()
}

fn route_debug(h: &Harness<App>) -> String {
    format!("{:?}", h.app())
        .split("nav_target: ")
        .nth(1)
        .and_then(|tail| tail.split(',').next())
        .expect("navigation target is in app debug state")
        .to_owned()
}

fn expanded_debug(h: &Harness<App>) -> String {
    format!("{:?}", h.app())
        .split("expanded: ")
        .nth(1)
        .and_then(|tail| tail.split(", chosen").next())
        .expect("saved expansion set")
        .to_owned()
}

#[test]
fn constructor_and_all_policy_preserve_the_existing_expanded_catalog() {
    let default = harness(App::new());
    assert_groups(&default, &GROUP_CHILDREN.map(|(group, _)| group));

    let mut h = harness(App::with_page(PageId::Chrome));
    set_policy(&mut h, NavExpand::All);
    click_group(&mut h, "Tables");
    assert!(!visible(&h, "Editable tables"));
    set_policy(&mut h, NavExpand::All);
    assert_groups(&h, &GROUP_CHILDREN.map(|(group, _)| group));

    click_group(&mut h, "Tables");
    advance_route(&mut h, "Chrome", "Buttons");
    assert_eq!(h.app().page(), PageId::Buttons);
    assert!(visible(&h, "Chrome"), "All preserves unrelated folds only");
    assert!(
        !visible(&h, "Editable tables"),
        "manual fold survives a route commit"
    );
}

#[test]
fn accordion_resets_on_commits_and_same_policy_setter_calls() {
    let mut h = harness(App::with_page(PageId::Chrome));
    set_policy(&mut h, NavExpand::Accordion);
    assert_groups(&h, &["App shell"]);

    advance_route(&mut h, "Chrome", "Buttons");
    assert_eq!(h.app().page(), PageId::Buttons);
    assert_groups(&h, &["Inputs & forms"]);

    click_group(&mut h, "Tables");
    assert!(visible(&h, "Tables"));
    set_policy(&mut h, NavExpand::Accordion);
    assert_groups(&h, &["Inputs & forms"]);
}

#[test]
fn current_group_closes_only_its_auto_group_and_keeps_manual_groups_manual() {
    let mut h = harness(App::with_page(PageId::Chrome));
    set_policy(&mut h, NavExpand::CurrentGroup);
    assert_groups(&h, &["App shell"]);

    click_group(&mut h, "Tables");
    click_group(&mut h, "Inputs & forms");
    advance_route(&mut h, "Chrome", "Buttons");
    assert_eq!(h.app().page(), PageId::Buttons);
    assert_groups(&h, &["Inputs & forms", "Tables"]);

    advance_route(&mut h, "Buttons", "Inputs");
    assert_eq!(h.app().page(), PageId::Inputs);
    assert_groups(&h, &["Inputs & forms", "Tables"]);

    click_entry(&mut h, "Cell editors");
    assert_eq!(h.app().page(), PageId::CellEditors);
    assert_groups(&h, &["Inputs & forms", "Tables"]);

    advance_route(&mut h, "Cell editors", "Chrome");
    assert_eq!(h.app().page(), PageId::Chrome);
    assert_groups(&h, &["App shell", "Inputs & forms", "Tables"]);
}

#[test]
fn current_group_opens_collapsed_destinations_and_closes_only_the_previous_auto_group() {
    let mut h = harness(App::with_page(PageId::Chrome));
    set_policy(&mut h, NavExpand::CurrentGroup);
    assert_groups(&h, &["App shell"]);

    advance_route(&mut h, "Chrome", "Buttons");
    assert_eq!(h.app().page(), PageId::Buttons);
    assert_groups(&h, &["Inputs & forms"]);

    advance_route(&mut h, "Buttons", "Cell editors");
    assert_eq!(h.app().page(), PageId::CellEditors);
    assert_groups(&h, &["Tables"]);

    advance_route(&mut h, "Cell editors", "Chrome");
    assert_eq!(h.app().page(), PageId::Chrome);
    assert_groups(&h, &["App shell"]);
}

#[test]
fn manual_fold_of_the_auto_group_relinquishes_its_ownership() {
    let mut h = harness(App::with_page(PageId::Chrome));
    set_policy(&mut h, NavExpand::CurrentGroup);
    assert!(visible(&h, "Chrome"));

    click_group(&mut h, "App shell");
    assert!(!visible(&h, "Chrome"));
    click_group(&mut h, "App shell");
    assert!(visible(&h, "Chrome"));

    advance_route(&mut h, "Chrome", "Buttons");
    assert!(
        visible(&h, "Chrome"),
        "manual reopen is no longer auto-owned"
    );
    assert!(visible(&h, "Buttons"));
}

#[test]
fn query_previews_are_neutral_and_enter_commits_even_after_same_target_preview() {
    for policy in POLICIES {
        let mut h = harness(App::with_page(PageId::Chrome));
        set_policy(&mut h, policy);
        click_group(&mut h, "Tables");
        let before_preview = expanded_debug(&h);

        let _ = h.key(KeyCode::Char('/'));
        let _ = h.type_str("Buttons");
        assert_eq!(
            h.app().page(),
            PageId::Chrome,
            "filter typing is neutral: {policy:?}"
        );
        let _ = h.key(KeyCode::Down);
        assert_eq!(
            h.app().page(),
            PageId::Buttons,
            "query cursor previews route: {policy:?}"
        );
        assert!(query_active(&h));
        let after_preview = expanded_debug(&h);
        assert_eq!(
            after_preview, before_preview,
            "preview changed saved folds: {policy:?}"
        );

        let _ = h.key(KeyCode::Enter);
        assert!(!query_active(&h), "activation ends query: {policy:?}");
        assert_eq!(h.app().page(), PageId::Buttons);
        match policy {
            NavExpand::All => assert_groups(
                &h,
                &[
                    "App shell",
                    "Inputs & forms",
                    "Navigation",
                    "Overlays",
                    "Feedback",
                    "Layout",
                    "Text & code",
                    "Screens",
                ],
            ),
            NavExpand::Accordion => assert_groups(&h, &["Inputs & forms"]),
            NavExpand::CurrentGroup => assert_groups(&h, &["Inputs & forms", "Tables"]),
        }
    }
}

#[test]
fn query_activation_owns_a_newly_revealed_group_until_the_next_commit() {
    let mut h = harness(App::with_page(PageId::Chrome));
    set_policy(&mut h, NavExpand::CurrentGroup);
    click_group(&mut h, "Tables");
    assert_groups(&h, &["App shell", "Tables"]);

    let _ = h.key(KeyCode::Char('/'));
    let _ = h.type_str("Buttons");
    let _ = h.key(KeyCode::Down);
    assert_eq!(h.app().page(), PageId::Buttons);
    assert_expanded_groups(&h, &["App shell", "Tables"]);

    let _ = h.key(KeyCode::Enter);
    assert!(!query_active(&h));
    assert_eq!(h.app().page(), PageId::Buttons);
    assert_groups(&h, &["Inputs & forms", "Tables"]);

    advance_route(&mut h, "Buttons", "Lists");
    assert_eq!(h.app().page(), PageId::Lists);
    assert_groups(&h, &["Navigation", "Tables"]);
}

#[test]
fn query_activation_keeps_a_manually_open_destination_group_manual() {
    let mut h = harness(App::with_page(PageId::Chrome));
    set_policy(&mut h, NavExpand::CurrentGroup);
    click_group(&mut h, "Inputs & forms");
    assert_groups(&h, &["App shell", "Inputs & forms"]);

    let _ = h.key(KeyCode::Char('/'));
    let _ = h.type_str("Buttons");
    let _ = h.key(KeyCode::Down);
    assert_eq!(h.app().page(), PageId::Buttons);
    let _ = h.key(KeyCode::Enter);
    assert!(!query_active(&h));

    advance_route(&mut h, "Buttons", "Lists");
    assert_eq!(h.app().page(), PageId::Lists);
    assert_groups(&h, &["Inputs & forms", "Navigation"]);
}

#[test]
fn query_pointer_folds_survive_escape_tab_and_focus_out_in_every_policy() {
    #[derive(Clone, Copy, Debug)]
    enum Exit {
        Escape,
        Tab,
        FocusOut,
    }

    for policy in POLICIES {
        for exit in [Exit::Escape, Exit::Tab, Exit::FocusOut] {
            let mut h = harness(App::with_page(PageId::Buttons));
            set_policy(&mut h, policy);
            let start_expanded = visible(&h, "Chrome");
            let _ = h.key(KeyCode::Char('/'));
            click_group(&mut h, "App shell");
            assert_eq!(
                visible(&h, "Chrome"),
                !start_expanded,
                "{policy:?} query pointer edit"
            );

            match exit {
                Exit::Escape => {
                    let _ = h.key(KeyCode::Esc);
                }
                Exit::Tab => {
                    let _ = h.key(KeyCode::Tab);
                }
                Exit::FocusOut => {
                    let help = h
                        .area_of_part(HEADER_HELP, PartRef::of(Part::LABEL))
                        .expect("header Help button causes navigation focus out");
                    let _ = h.click(help.x, help.y);
                }
            }
            assert!(!query_active(&h), "query did not end: {policy:?} {exit:?}");
            assert_eq!(
                group_is_expanded(&h, "App shell"),
                !start_expanded,
                "clear should preserve explicit pointer fold: {policy:?} {exit:?}"
            );
            assert_eq!(h.app().page(), PageId::Buttons);
        }
    }
}

#[test]
fn query_pointer_edits_survive_until_clear_except_for_chosen_route_reveal() {
    for policy in POLICIES {
        let mut h = harness(App::with_page(PageId::Chrome));
        set_policy(&mut h, policy);
        let _ = h.key(KeyCode::Char('/'));
        click_group(&mut h, "App shell");
        assert!(!group_is_expanded(&h, "App shell"));
        let _ = h.key(KeyCode::Esc);
        assert!(
            group_is_expanded(&h, "App shell"),
            "clear reveals the already chosen route: {policy:?}"
        );
        assert!(visible(&h, "Chrome"));
    }
}

#[test]
fn query_pointer_fold_of_auto_group_relinquishes_ownership_before_next_commit() {
    let mut h = harness(App::with_page(PageId::Chrome));
    set_policy(&mut h, NavExpand::CurrentGroup);
    let _ = h.key(KeyCode::Char('/'));
    click_group(&mut h, "App shell");
    assert!(!group_is_expanded(&h, "App shell"));
    click_group(&mut h, "App shell");
    assert!(group_is_expanded(&h, "App shell"));
    let _ = h.key(KeyCode::Esc);

    advance_route(&mut h, "Chrome", "Buttons");
    assert_groups(&h, &["App shell", "Inputs & forms"]);
}

#[test]
fn setting_policy_during_matching_and_no_match_queries_resets_folds_without_routing() {
    let mut empty = harness(App::with_page(PageId::Chrome));
    let _ = empty.key(KeyCode::Char('/'));
    click_group(&mut empty, "Tables");
    set_policy(&mut empty, NavExpand::Accordion);
    assert!(query_active(&empty));
    assert_eq!(query_text(&empty), "");
    assert_eq!(empty.app().page(), PageId::Chrome);
    assert_eq!(empty.focus(), Some(NAV));
    assert_expanded_groups(&empty, &["App shell"]);

    let mut matching = harness(App::with_page(PageId::Chrome));
    let _ = matching.key(KeyCode::Char('/'));
    let _ = matching.type_str("Buttons");
    let cursor_before = format!("{:?}", matching.app())
        .split("cursor: ")
        .nth(1)
        .and_then(|tail| tail.split(',').next())
        .expect("cursor field")
        .to_owned();
    set_policy(&mut matching, NavExpand::Accordion);
    assert!(query_active(&matching));
    assert_eq!(query_text(&matching), "Buttons");
    assert_eq!(matching.app().page(), PageId::Chrome);
    assert_eq!(matching.focus(), Some(NAV));
    assert!(visible(&matching, "Buttons"));
    assert_expanded_groups(&matching, &["App shell"]);
    let cursor_after = format!("{:?}", matching.app())
        .split("cursor: ")
        .nth(1)
        .and_then(|tail| tail.split(',').next())
        .expect("cursor field")
        .to_owned();
    assert_eq!(
        cursor_after, cursor_before,
        "visible stable query cursor is retained"
    );

    let mut no_match = harness(App::with_page(PageId::Chrome));
    let _ = no_match.key(KeyCode::Char('/'));
    let _ = no_match.type_str("no-such-page");
    set_policy(&mut no_match, NavExpand::CurrentGroup);
    assert!(query_active(&no_match));
    assert_eq!(query_text(&no_match), "no-such-page");
    assert_eq!(no_match.app().page(), PageId::Chrome);
    assert_eq!(no_match.focus(), Some(NAV));
    let _ = no_match.key(KeyCode::Enter);
    assert_eq!(
        no_match.app().page(),
        PageId::Chrome,
        "empty results cannot activate stale cursor"
    );
}

#[test]
fn placeholder_preview_is_neutral_and_brackets_are_query_text() {
    let mut h = harness(App::with_page(PageId::Chrome));
    set_policy(&mut h, NavExpand::CurrentGroup);
    click_group(&mut h, "Tables");
    let before = expanded_debug(&h);

    let _ = h.key(KeyCode::Char('/'));
    let _ = h.type_str("Empty & error");
    assert_eq!(
        h.app().page(),
        PageId::Chrome,
        "typing a placeholder match is neutral"
    );
    let _ = h.key(KeyCode::Down);
    assert!(query_active(&h));
    let placeholder = NAV_ENTRIES
        .iter()
        .position(|entry| entry.label == "Empty & error")
        .expect("placeholder entry");
    assert_eq!(route_debug(&h), format!("Placeholder({placeholder})"));
    assert_eq!(
        h.app().page(),
        PageId::Chrome,
        "placeholder previews have no page body"
    );
    assert_eq!(
        expanded_debug(&h),
        before,
        "placeholder preview changed saved folds"
    );

    let _ = h.key(KeyCode::Enter);
    assert!(!query_active(&h));
    assert_eq!(route_debug(&h), format!("Placeholder({placeholder})"));
    assert_eq!(
        h.app().page(),
        PageId::Chrome,
        "placeholder commit keeps the last live page"
    );
    assert_groups(&h, &["Feedback", "Tables"]);

    let mut shortcuts = harness(App::with_page(PageId::Chrome));
    let _ = shortcuts.key(KeyCode::Char('/'));
    let route_before = route_debug(&shortcuts);
    let _ = shortcuts.key(KeyCode::Char(']'));
    assert_eq!(query_text(&shortcuts), "]");
    assert_eq!(
        route_debug(&shortcuts),
        route_before,
        "query owns printable next shortcut"
    );
    let _ = shortcuts.key(KeyCode::Char('['));
    assert_eq!(query_text(&shortcuts), "][");
    assert_eq!(
        route_debug(&shortcuts),
        route_before,
        "query owns printable previous shortcut"
    );
    let _ = shortcuts.key(KeyCode::Esc);
    let _ = shortcuts.key(KeyCode::Char(']'));
    assert!(route_debug(&shortcuts).contains("Placeholder(2)"));
    let _ = shortcuts.key(KeyCode::Char('['));
    assert!(route_debug(&shortcuts).contains("Live(Chrome)"));
    assert_eq!(shortcuts.app().page(), PageId::Chrome);
}
