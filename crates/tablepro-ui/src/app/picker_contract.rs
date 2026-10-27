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
