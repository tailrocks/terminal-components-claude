use tablepro_domain::model::SwitchTarget;
use termrock::ItemKey;
use termrock::{Color, Modifier, Position, Theme};
use termrock_test_support::Harness;

use super::{Screen, Surface, TableProApp};
use crate::tab_list;

fn production() -> TableProApp {
    let mut app = TableProApp::default();
    let index = app
        .connections
        .iter()
        .position(|connection| connection.name == "Production")
        .unwrap_or(usize::MAX);
    assert!(app.connect(index), "production connection");
    app
}

#[test]
fn ctrl_o_shows_the_approved_picker_cell() {
    let mut harness = Harness::new(production(), Theme::junie(), 120, 40);
    assert_eq!(harness.app().screen(), Screen::Workbench);
    assert_ne!(harness.app().surface(), Surface::QuickSwitcher);
    assert_ne!(
        harness.cell(18, 11).symbol(),
        "O",
        "closed workbench took the picker stamp: {}",
        harness.row(11)
    );
    let _ = harness.ctrl('o');
    assert_eq!(harness.app().screen(), Screen::Workbench);
    assert_eq!(harness.app().surface(), Surface::QuickSwitcher);
    assert!(!harness.app().workbench.maximized);
    assert!(harness.app().destructive_intent.is_none());
    let title = harness.cell(18, 11);
    assert_eq!(title.symbol(), "O", "text={}", harness.text());
    assert_eq!(title.fg, Color::Rgb(255, 255, 255));
    assert_eq!(title.bg, Color::Rgb(24, 24, 27));
    assert!(title.modifier.contains(Modifier::BOLD));
    assert_eq!(harness.cursor(), Some(Position::new(20, 12)));
    assert!(harness.runtime().cursor_bar());
}

#[test]
fn ctrl_g_shows_the_approved_tab_cell() {
    let mut harness = Harness::new(production(), Theme::junie(), 120, 40);
    assert_eq!(harness.app().screen(), Screen::Workbench);
    assert!(!harness.is_open(tab_list::ID));
    assert_ne!(
        harness.cell(30, 17).symbol(),
        "O",
        "closed workbench took the tab list: {}",
        harness.row(17)
    );
    let _ = harness.ctrl('g');
    assert_eq!(harness.app().screen(), Screen::Workbench);
    assert!(harness.is_open(tab_list::ID));
    assert_ne!(harness.app().surface(), Surface::QuickSwitcher);
    assert!(!harness.app().workbench.maximized);
    assert!(harness.app().destructive_intent.is_none());
    let title = harness.cell(30, 17);
    assert_eq!(title.symbol(), "O", "text={}", harness.text());
    assert_eq!(title.fg, Color::Rgb(255, 255, 255));
    assert_eq!(title.bg, Color::Rgb(24, 24, 27));
    assert!(title.modifier.contains(Modifier::BOLD));
    assert_eq!(harness.cursor(), Some(Position::new(32, 18)));
    assert!(harness.runtime().cursor_bar());
}

#[test]
#[allow(clippy::expect_used)]
fn ctrl_g_opens_tabs_for_another_connection_and_size() {
    let mut app = TableProApp::default();
    let index = app
        .connections
        .iter()
        .position(|connection| connection.name != "Production")
        .expect("second connection");
    assert!(app.connect(index), "other connection");
    assert!(app.workbench.new_query("SELECT 1").is_some());
    let mut harness = Harness::new(app, Theme::junie(), 100, 30);
    let _ = harness.ctrl('g');
    assert!(harness.is_open(tab_list::ID));
    assert!(
        harness.text().contains("Open tabs"),
        "text={}",
        harness.text()
    );
    assert!(
        harness.text().contains("Query 2"),
        "text={}",
        harness.text()
    );
}

#[test]
fn ctrl_l_renders_the_safe_mode_picker_through_its_layer_slot() {
    let mut harness = Harness::new(production(), Theme::junie(), 120, 40);
    let _ = harness.ctrl('l');
    assert!(harness.is_open(crate::safe_mode::ID));
    assert!(
        harness.text().contains("Safe Mode · this connection"),
        "text={}",
        harness.text()
    );
    assert!(
        harness
            .text()
            .contains("↑↓ Move · Enter Set level · Esc Keep"),
        "text={}",
        harness.text()
    );
}

#[test]
fn quick_switcher_keeps_a_bar_caret_and_restores_workbench_focus() {
    let mut harness = Harness::new(production(), Theme::junie(), 120, 40);
    let workbench_focus = harness.runtime().focus();
    let _ = harness.ctrl('o');
    assert_eq!(harness.runtime().focus(), Some(super::quick_switcher::ID));
    assert!(harness.runtime().cursor_bar());

    let _ = harness.type_str("orders");
    assert!(harness.text().contains("orders"));
    assert!(harness.runtime().cursor_bar());
    let _ = harness.key(termrock::KeyCode::Esc);
    assert!(harness.is_open(super::quick_switcher::ID));
    let _ = harness.key(termrock::KeyCode::Esc);

    assert!(!harness.is_open(super::quick_switcher::ID));
    assert_eq!(harness.runtime().focus(), workbench_focus);
}

#[test]
fn quick_switcher_query_refresh_selects_first_match_after_surviving_cursor()
-> Result<(), &'static str> {
    let mut harness = Harness::new(production(), Theme::junie(), 120, 40);
    let _ = harness.ctrl('o');

    let candidates = ["a", "e", "i", "o", "r", "s", "t", "u"];
    let Some((query, first_key, prior_key, prior_index)) = candidates.iter().find_map(|query| {
        let matches = harness
            .app()
            .workbench
            .switcher()
            .search(query)
            .into_iter()
            .filter(|item| !matches!(&item.target, SwitchTarget::Connection(_)))
            .collect::<Vec<_>>();
        let first_key = matches.first()?.key.clone();
        let prior = matches.iter().skip(1).find_map(|item| {
            harness
                .app()
                .switcher
                .items
                .iter()
                .position(|current| current.key == item.key)
                .map(|index| (item.key.clone(), index))
        })?;
        Some((*query, first_key, prior.0, prior.1))
    }) else {
        return Err("fixture must contain an overlapping query result");
    };

    harness
        .app_mut()
        .switcher
        .state
        .set_cursor(prior_index, ItemKey::text(&prior_key));
    assert_eq!(
        harness.app().switcher.state.cursor(),
        Some(ItemKey::text(&prior_key))
    );

    let _ = harness.type_str(query);
    assert_eq!(
        harness.app().switcher.state.cursor(),
        Some(ItemKey::text(&first_key)),
        "query {query:?} must start at its first enabled result"
    );
    assert!(harness.is_open(super::quick_switcher::ID));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    Ok(())
}
