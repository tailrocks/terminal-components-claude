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
