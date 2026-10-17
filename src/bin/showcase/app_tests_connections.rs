//! The Connections screen (plan §7.5, SC47), driven through the real
//! showcase: the detail card follows the list, Enter connects (to a
//! connection, a failure block with Reconnect), Duplicate and Delete…
//! change the list, the form saves a new connection, and every option
//! changes the frame.

use ratatui::crossterm::event::KeyCode;

use crate::app::PageId;
use crate::app_tests_tabs::Harness;
use crate::options::{self, Effect};
use crate::pages::connections::ConnectionsPage;
use crate::pages::template::Section;
use junie_tui::core::event::Input;
use junie_tui::runtime::Application;
use junie_tui::widgets::form_dialog::FormDialog;

fn page(h: &Harness) -> &ConnectionsPage {
    h.app.pages[PageId::Connections.index()]
        .as_any()
        .and_then(|a| a.downcast_ref::<ConnectionsPage>())
        .expect("the Connections page")
}

fn ticks(h: &mut Harness, n: usize) {
    for _ in 0..n {
        h.app.handle(Input::Tick);
    }
    h.draw();
}

/// Click the list row labelled `name` (focuses the list).
fn pick(h: &mut Harness, name: &str) {
    let (x, y) = h
        .find(name)
        .unwrap_or_else(|| panic!("{name} row\n{}", h.text()));
    h.click(x, y);
}

#[test]
fn the_list_groups_connections_and_the_detail_card_shows_the_selection() {
    let mut h = Harness::on(120, 40, PageId::Connections);
    let text = h.text();
    assert!(!text.contains("arrives with SC47"), "{text}");
    for want in [
        "Personal",
        "Acme",
        "◆ Production",
        "◇ Staging",
        "prod-db-1.acme.io:5432",
        "Safe Mode",
        "Connect",
        "Duplicate",
        "Delete…",
        "New connection…",
    ] {
        assert!(text.contains(want), "{want:?} missing:\n{text}");
    }
    pick(&mut h, "Scratch");
    assert!(h.app.shell.focus.is(page(&h).tree.id));
    assert!(h.text().contains("~/scratch.db"), "{}", h.text());
    assert!(
        !h.text().contains("Safe Mode"),
        "only production asks first"
    );
}

#[test]
fn enter_connects_and_a_failure_offers_reconnect() {
    let mut h = Harness::on(120, 40, PageId::Connections);
    pick(&mut h, "Development");
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Connecting"), "{}", h.text());
    ticks(&mut h, 10);
    assert!(
        h.text().contains("✓ Connected · PostgreSQL 16.3"),
        "{}",
        h.text()
    );
    pick(&mut h, "Staging");
    h.key(KeyCode::Enter);
    ticks(&mut h, 10);
    assert!(h.text().contains("Authentication failed"), "{}", h.text());
    let (x, y) = h.find("Reconnect").expect("the failure block's action");
    h.click(x + 1, y);
    assert!(h.text().contains("Connecting"), "Reconnect tries again");
}

#[test]
fn duplicate_and_delete_change_the_list() {
    let mut h = Harness::on(120, 40, PageId::Connections);
    let (x, y) = h.find("Duplicate").unwrap();
    h.click(x + 1, y);
    assert!(h.text().contains("Production copy"), "{}", h.text());
    let (x, y) = h.find("Delete…").unwrap();
    h.click(x + 1, y);
    assert!(h.app.dialog().is_some(), "Delete asks first");
    assert!(h.text().contains("Delete Production copy?"), "{}", h.text());
    // the safe choice has focus; Tab reaches Delete
    h.key(KeyCode::Tab);
    h.key(KeyCode::Enter);
    assert!(h.app.dialog().is_none());
    assert!(!h.text().contains("◆ Production copy"), "{}", h.text());
    assert!(h.text().contains("Deleted Production copy"));
    assert!(
        h.text().contains("prod-db-1.acme.io"),
        "the neighbour is selected"
    );
}

#[test]
fn the_form_saves_a_new_connection() {
    let mut h = Harness::on(120, 40, PageId::Connections);
    let (x, y) = h.find("New connection…").unwrap();
    h.click(x + 1, y);
    assert!(
        h.app.layers.top_as::<FormDialog>().is_some(),
        "{}",
        h.text()
    );
    assert!(h.text().contains("New connection"));
    for c in "Reporting".chars() {
        h.key(KeyCode::Char(c));
    }
    h.key(KeyCode::Tab);
    let (x, y) = h.find("Test connection").unwrap();
    h.click(x + 1, y);
    assert!(h.text().contains("Connected in 12 ms"), "{}", h.text());
    let (x, y) = h.find("Save").unwrap();
    h.click(x + 1, y);
    assert!(h.app.layers.top_as::<FormDialog>().is_none());
    let text = h.text();
    assert!(
        text.contains("Reporting"),
        "the saved connection is listed:\n{text}"
    );
    assert!(text.contains("localhost:5432"), "and selected:\n{text}");
}

#[test]
fn the_empty_list_offers_a_new_connection() {
    let mut h = Harness::on(120, 40, PageId::Connections);
    h.opt("connections.list=empty");
    assert!(h.text().contains("No connections"), "{}", h.text());
    assert!(h.text().contains("Select a connection"));
    let (x, y) = h.find("New connection…").unwrap();
    h.click(x + 1, y);
    assert!(h.app.layers.top_as::<FormDialog>().is_some());
}

#[test]
fn every_frame_option_value_changes_the_playground() {
    for opt in options::page_options(PageId::Connections) {
        if opt.effect == Effect::Behaviour {
            continue;
        }
        let frame = |v: &str| {
            let mut h = Harness::on(120, 40, PageId::Connections);
            h.opt(&format!("{}={v}", opt.key));
            h.text()
        };
        let base = frame(opt.values[opt.default]);
        for v in opt.values.iter().filter(|v| **v != opt.values[opt.default]) {
            assert_ne!(
                base,
                frame(v),
                "{}={v} shows the same as the default",
                opt.key
            );
        }
    }
}

#[test]
fn at_80_by_24_the_details_are_a_drawer_over_the_list() {
    let mut h = Harness::on(80, 24, PageId::Connections);
    let text = h.text();
    assert!(text.contains("Connections"), "{text}");
    assert!(text.contains("Production"), "{text}");
    // narrow: the list and a one-line summary; the card waits in a drawer
    assert!(text.contains("→ Production · production"), "{text}");
    assert!(!text.contains("Engine"), "{text}");
    h.focus(page(&h).new.id);
    h.key(KeyCode::Tab);
    assert!(
        page(&h).md.drawer_open(),
        "Tab past the list opens the details"
    );
    assert!(h.text().contains("Engine"), "{}", h.text());
    h.key(KeyCode::Esc);
    assert!(!page(&h).md.drawer_open(), "Esc returns to the list");
    assert!(
        h.app.shell.focus.is(page(&h).tree.id),
        "focus back on the list"
    );
}

#[test]
fn the_detail_card_ends_with_its_actions_and_a_failure_is_a_notice() {
    let mut h = Harness::on(120, 40, PageId::Connections);
    let card_end = page(&h).connect.area.y;
    let (_, last) = h.find("Last used").expect("the last fact");
    assert_eq!(card_end, last + 2, "the actions follow the facts");
    pick(&mut h, "Staging");
    h.key(KeyCode::Enter);
    ticks(&mut h, 10);
    let (_, y) = h.find("Authentication failed").expect("the notice");
    let (gx, gy) = h.find("!").expect("the notice glyph");
    assert_eq!(gy, y, "the error glyph leads the message");
    assert!(gx < h.find("Authentication failed").unwrap().0);
    assert_eq!(page(&h).connect.label, "Reconnect");
}

#[test]
fn the_states_section_shows_the_list_and_the_actions() {
    let mut h = Harness::on(120, 40, PageId::Connections);
    h.section(Section::States);
    let text = h.text();
    for want in ["Connections", "Connect", "Delete…", "disabled", "pressed"] {
        assert!(text.contains(want), "{want:?}:\n{text}");
    }
}
