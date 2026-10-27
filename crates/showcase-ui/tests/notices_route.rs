use showcase_ui::{App, NAV_ENTRIES, PageId};
use std::time::Duration;

use termrock::{Axis, Id, ItemKey, KeyCode, MouseKind, Part, PartRef, Position, Rect, Theme};
use termrock_test_support::Harness;

const NAV: Id = Id::root("showcase_ui::app::navigation");
const PAGE_TABS: Id = Id::root("showcase_ui::pages::notices.page_tabs");
const PLAY_SCROLL: Id = Id::root("showcase_ui::pages::notices.play_scroll");
const OPTIONS_SCROLL: Id = Id::root("showcase_ui::pages::notices.options_scroll");
const TEST_CONNECTION: Id = Id::root("showcase_ui::pages::notices.test_connection");
const KIND: Id = Id::root("showcase_ui::pages::notices.option.kind");
const MARK_CUT: Id = Id::root("showcase_ui::pages::notices.option.mark_cut");
const HELP: Id = Id::root("showcase_ui::app::help");
const HEADER_HELP: Id = Id::root("showcase_ui::app::header.help");
const REVEAL_OWNERS: [Id; 6] = [
    Id::root("showcase_ui::pages::notices.reveal.connection"),
    Id::root("showcase_ui::pages::notices.reveal.query"),
    Id::root("showcase_ui::pages::notices.reveal.inline.0"),
    Id::root("showcase_ui::pages::notices.reveal.inline.1"),
    Id::root("showcase_ui::pages::notices.reveal.inline.2"),
    Id::root("showcase_ui::pages::notices.reveal.inline.3"),
];
const CUT_PART: Part = Part::custom("notice-reveal-cut");

fn footer(h: &Harness<App>) -> String {
    h.row(h.buffer().area().height.saturating_sub(1))
}

fn first_cut(h: &Harness<App>, owner: Id) -> Option<(PartRef, Rect)> {
    (0_u64..3).find_map(|source| {
        (0_u64..64).find_map(|row| {
            let part = PartRef::item(CUT_PART, ItemKey::num((source << 16) | row));
            h.area_of_part(owner, part).map(|area| (part, area))
        })
    })
}

fn first_live_cut(h: &Harness<App>) -> Option<(Id, PartRef, Rect)> {
    REVEAL_OWNERS
        .iter()
        .find_map(|owner| first_cut(h, *owner).map(|(part, area)| (*owner, part, area)))
}

fn reveal_area(h: &Harness<App>, owner: Id) -> Option<Rect> {
    h.area_of_part(
        owner.sub("notice-reveal-scroll"),
        PartRef::of(Part::CONTAINER),
    )
}

fn play_scroll_offset(h: &Harness<App>) -> usize {
    format!("{:?}", h.app())
        .split("play_scroll: ScrollState { offset: ")
        .nth(1)
        .and_then(|tail| tail.split(',').next())
        .and_then(|offset| offset.parse().ok())
        .expect("Notices page has a play-scroll state")
}

#[test]
fn real_app_registers_live_notice_cuts_for_the_same_layer_reveal() {
    let h = Harness::new(App::for_page_name("Notices"), Theme::junie(), 120, 40);

    assert!(
        REVEAL_OWNERS
            .iter()
            .any(|owner| first_cut(&h, *owner).is_some()),
        "the live connection Notice must publish its current cut rows; app={:?}; screen={}",
        h.app(),
        h.text()
    );
}

fn notices_harness(paused: bool, width: u16, height: u16) -> Harness<App> {
    let app = if paused {
        App::with_page_motion(PageId::Overview, true, 0)
    } else {
        App::for_page_name("Notices")
    };
    let mut h = Harness::new(app, Theme::junie(), width, height);
    if paused {
        for _ in 0..33 {
            let _ = h.key(KeyCode::Char(']'));
        }
        assert!(format!("{:?}", h.app()).contains("nav_target: Placeholder(33)"));
    }
    h
}

fn hover_cut(h: &mut Harness<App>, owner: Id, area: Rect) {
    let _ = h.mouse(
        MouseKind::Move,
        area.x.saturating_add(area.width / 2),
        area.y,
    );
    assert!(
        first_cut(h, owner).is_some(),
        "the live cut remains registered"
    );
}

fn scroll_reveal_to_tail(h: &mut Harness<App>, owner: Id) {
    for _ in 0..48 {
        if h.text().contains("NOTICE_REVEAL_SOURCE_TAIL") {
            break;
        }
        let area = reveal_area(h, owner).expect("the overflowing reveal keeps its scroll area");
        let _ = h.wheel(Axis::V, 1, area.x, area.y);
    }
    let rendered = h.text();
    assert!(rendered.contains("北京"), "CJK tail survives: {rendered}");
    assert!(
        rendered.contains("e\u{301}"),
        "combining tail survives: {rendered}"
    );
    assert!(
        rendered.contains("👩‍👩‍👧‍👦"),
        "ZWJ grapheme tail survives: {rendered}"
    );
    assert!(rendered.contains("NOTICE_REVEAL_SOURCE_TAIL"), "{rendered}");
}

#[test]
fn showcase_root_reveal_uses_live_and_paused_300ms_timing_without_taking_focus() {
    for paused in [false, true] {
        let mut h = notices_harness(paused, 120, 40);
        let (owner, _part, source) = first_live_cut(&h).expect("a current live Notice is cut");
        let before_focus = h.focus();
        let before_cursor = h.cursor();

        hover_cut(&mut h, owner, source);
        assert!(reveal_area(&h, owner).is_none());
        let _ = h.advance(Duration::from_millis(299));
        assert!(reveal_area(&h, owner).is_none(), "paused={paused}");
        assert!(!h.text().contains("NOTICE_REVEAL_SOURCE_TAIL"));

        let _ = h.advance(Duration::from_millis(1));
        let pane = reveal_area(&h, owner).expect("the exact 300 ms deadline paints the pane");
        assert!(pane.height > 0 && pane.width > 0);
        assert_eq!(h.focus(), before_focus, "paused={paused}");
        assert_eq!(h.cursor(), before_cursor, "paused={paused}");

        scroll_reveal_to_tail(&mut h, owner);
        assert_eq!(h.focus(), before_focus, "passive scrolling keeps focus");
        assert_eq!(h.cursor(), before_cursor, "passive scrolling keeps cursor");
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
}

#[test]
fn app_reveal_wheel_falls_through_to_the_page_at_both_ends() {
    let mut h = notices_harness(false, 120, 24);
    let (owner, _, source) = first_live_cut(&h).expect("a current live Notice is cut");
    hover_cut(&mut h, owner, source);
    let _ = h.advance(Duration::from_millis(300));
    let start_page_offset = play_scroll_offset(&h);
    assert_eq!(start_page_offset, 0, "the first reveal starts at page top");

    for _ in 0..48 {
        if h.text().contains("NOTICE_REVEAL_SOURCE_TAIL") {
            break;
        }
        let pane = reveal_area(&h, owner).expect("the current source reveal remains active");
        let _ = h.wheel(Axis::V, 1, pane.x, pane.y);
    }
    assert!(h.text().contains("NOTICE_REVEAL_SOURCE_TAIL"));
    assert_eq!(play_scroll_offset(&h), start_page_offset);

    let before_forward_fallback = play_scroll_offset(&h);
    for _ in 0..12 {
        let pane = reveal_area(&h, owner).expect("the tail remains under the same live source");
        let _ = h.wheel(Axis::V, 1, pane.x, pane.y);
        if play_scroll_offset(&h) > before_forward_fallback {
            break;
        }
    }
    let after_forward_fallback = play_scroll_offset(&h);
    assert!(
        after_forward_fallback > before_forward_fallback,
        "one wheel at the reveal tail reaches the page: app={:?}",
        h.app()
    );
    assert!(
        reveal_area(&h, owner).is_none(),
        "page movement removes the old projection"
    );

    let (scrolled_owner, _, scrolled_source) =
        first_live_cut(&h).expect("page scroll leaves a current visible cut row");
    hover_cut(&mut h, scrolled_owner, scrolled_source);
    let _ = h.advance(Duration::from_millis(300));
    assert!(reveal_area(&h, scrolled_owner).is_some());
    scroll_reveal_to_tail(&mut h, scrolled_owner);

    for _ in 0..48 {
        if !h.text().contains("NOTICE_REVEAL_SOURCE_TAIL") {
            break;
        }
        let pane =
            reveal_area(&h, scrolled_owner).expect("the current source reveal remains active");
        let _ = h.wheel(Axis::V, -1, pane.x, pane.y);
    }
    assert!(!h.text().contains("NOTICE_REVEAL_SOURCE_TAIL"));
    let before_reverse_fallback = play_scroll_offset(&h);
    assert!(before_reverse_fallback > 0);
    for _ in 0..12 {
        let pane = reveal_area(&h, scrolled_owner)
            .expect("the head remains under the same live source before fallback");
        let _ = h.wheel(Axis::V, -1, pane.x, pane.y);
        if play_scroll_offset(&h) < before_reverse_fallback {
            break;
        }
    }
    assert!(
        play_scroll_offset(&h) < before_reverse_fallback,
        "one wheel at the reveal head reaches the page"
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn same_layer_reveal_paints_over_button_without_intercepting_first_click() {
    let mut h = notices_harness(false, 120, 40);
    let (owner, _, source) = first_live_cut(&h).expect("a current live Notice is cut");
    let button = h
        .area_of(TEST_CONNECTION)
        .expect("the live connection action is visible");
    hover_cut(&mut h, owner, source);
    let mut underlay = Vec::new();
    for y in button.y..button.bottom() {
        for x in button.x..button.right() {
            underlay.push((x, y, format!("{:?}", h.cell(x, y))));
        }
    }
    let _ = h.advance(Duration::from_millis(300));
    let pane = reveal_area(&h, owner).expect("the active Notice has a reveal pane");
    let overlap = pane.intersection(button);
    assert!(
        !overlap.is_empty(),
        "pane {pane:?} overlaps button {button:?}"
    );
    let painted_over_button = underlay
        .into_iter()
        .find(|(x, y, original)| {
            overlap.contains(Position::new(*x, *y)) && format!("{:?}", h.cell(*x, *y)) != *original
        })
        .map(|(x, y, _)| Position::new(x, y))
        .expect("the same-layer pane paints over actual button cells");

    let _ = h.click(painted_over_button.x, painted_over_button.y);
    assert!(
        h.text().contains("Could not reach the host"),
        "{}",
        h.text()
    );
    assert!(footer(&h).contains("Could not reach the host"));
    assert!(!h.text().contains("Connected in 182 ms"));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn navigation_query_and_help_suppress_the_live_notice_reveal() {
    let mut h = notices_harness(false, 120, 40);
    let (owner, _, source) = first_live_cut(&h).expect("a current live Notice is cut");
    hover_cut(&mut h, owner, source);
    let _ = h.advance(Duration::from_millis(300));
    assert!(reveal_area(&h, owner).is_some());

    assert!(h.tab_to(NAV));
    let _ = h.key(KeyCode::Char('/'));
    let _ = h.type_str("Notices");
    assert!(format!("{:?}", h.app()).contains("nav_query: Some(NavQuery { text: \"Notices\""));
    assert!(!h.text().contains("NOTICE_REVEAL_SOURCE_TAIL"));
    assert!(
        REVEAL_OWNERS
            .iter()
            .all(|owner| reveal_area(&h, *owner).is_none())
    );
    let _ = h.key(KeyCode::Tab);
    assert!(format!("{:?}", h.app()).contains("nav_query: None"));

    assert!(h.tab_to(NAV));
    let _ = h.key(KeyCode::Char('/'));
    let _ = h.type_str("Notice");
    assert!(h.text().contains("/ Notice"));
    let _ = h.key(KeyCode::Esc);
    assert!(format!("{:?}", h.app()).contains("nav_query: None"));
    assert_eq!(h.focus(), Some(NAV), "Escape preserves navigation focus");

    let (owner, _, source) = first_live_cut(&h)
        .unwrap_or_else(|| panic!("the live page restores current cuts; app={:?}", h.app()));
    hover_cut(&mut h, owner, source);
    let _ = h.advance(Duration::from_millis(300));
    assert!(reveal_area(&h, owner).is_some());
    let help = h
        .area_of_part(HEADER_HELP, PartRef::of(Part::LABEL))
        .expect("the shell Help control stays visible");
    let _ = h.click(help.x.saturating_add(1), help.y);
    assert!(h.is_open(HELP));
    assert!(reveal_area(&h, owner).is_none(), "Help owns the top layer");
    let _ = h.key(KeyCode::Esc);
    assert!(!h.is_open(HELP));
    assert!(
        reveal_area(&h, owner).is_none(),
        "closing Help does not retain stale text"
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn root_reveal_revalidates_after_options_resize_scroll_and_route_changes() {
    let mut h = notices_harness(false, 120, 24);
    let (owner, _, source) = first_live_cut(&h).expect("a current live Notice is cut");
    let stationary = Position::new(source.x.saturating_add(1), source.y);
    let _ = h.mouse(MouseKind::Move, stationary.x, stationary.y);
    let _ = h.advance(Duration::from_millis(300));
    let old_pane = reveal_area(&h, owner).expect("the active source has a reveal");

    let _ = h.resize(100, 24);
    assert!(
        reveal_area(&h, owner).is_none(),
        "resize clears the old projection"
    );
    let (_, _, resized_source) = first_live_cut(&h).expect("resize keeps a live cut");
    assert!(
        resized_source.contains(stationary),
        "the stationary pointer remains over the current source row: {resized_source:?}"
    );
    let _ = h.tick();
    let _ = h.advance(Duration::from_millis(299));
    assert!(
        reveal_area(&h, owner).is_none(),
        "resize starts a fresh rest interval"
    );
    let _ = h.advance(Duration::from_millis(1));
    let resized_pane = reveal_area(&h, owner)
        .unwrap_or_else(|| panic!("the current geometry is re-resolved; app={:?}", h.app()));
    assert!(resized_pane.contains(Position::new(resized_source.x, resized_source.y)));
    assert_ne!(
        old_pane.x, resized_pane.x,
        "the pane follows the resized page"
    );

    assert!(h.tab_to(PAGE_TABS));
    let _ = h.key(KeyCode::Right);
    assert!(h.tab_to(MARK_CUT));
    let _ = h.key(KeyCode::Char(' '));
    assert!(format!("{:?}", h.app()).contains("mark_cut: false"));
    assert!(
        REVEAL_OWNERS
            .iter()
            .all(|owner| reveal_area(&h, *owner).is_none())
    );
    assert!(h.tab_to(PAGE_TABS));
    let _ = h.key(KeyCode::Left);
    let (_, _, option_source) = first_live_cut(&h).expect("hidden marker keeps source metadata");
    assert!(option_source.contains(stationary));
    let _ = h.mouse(MouseKind::Move, stationary.x, stationary.y);
    let _ = h.advance(Duration::from_millis(299));
    assert!(
        reveal_area(&h, owner).is_none(),
        "option revision resets readiness"
    );
    let _ = h.advance(Duration::from_millis(1));
    assert!(
        reveal_area(&h, owner).is_some(),
        "mark_cut=false stays revealable"
    );

    let play = h.area_of(PLAY_SCROLL).expect("live page scroll region");
    let scroll_point = Position::new(play.x.saturating_add(1), play.y.saturating_add(1));
    let _ = h.mouse(MouseKind::Move, scroll_point.x, scroll_point.y);
    let old_offset = play_scroll_offset(&h);
    let _ = h.wheel(Axis::V, 1, scroll_point.x, scroll_point.y);
    assert!(
        play_scroll_offset(&h) > old_offset,
        "page receives the wheel"
    );
    assert!(
        REVEAL_OWNERS
            .iter()
            .all(|owner| reveal_area(&h, *owner).is_none())
    );

    let _ = h.key(KeyCode::Char('['));
    assert!(format!("{:?}", h.app()).contains("nav_target: Placeholder(32)"));
    assert!(
        REVEAL_OWNERS
            .iter()
            .all(|owner| reveal_area(&h, *owner).is_none())
    );
    let _ = h.key(KeyCode::Char(']'));
    assert!(format!("{:?}", h.app()).contains("nav_target: Placeholder(33)"));
    assert!(
        REVEAL_OWNERS
            .iter()
            .all(|owner| reveal_area(&h, *owner).is_none())
    );

    let (owner, _, current_source) = first_live_cut(&h).expect("route restores current cuts");
    hover_cut(&mut h, owner, current_source);
    let _ = h.advance(Duration::from_millis(300));
    assert!(
        reveal_area(&h, owner).is_some(),
        "new page ownership starts cleanly"
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
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
    assert_eq!(PageId::ALL.len(), 26);
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
