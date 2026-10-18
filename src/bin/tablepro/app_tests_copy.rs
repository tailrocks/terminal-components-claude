

use ratatui::crossterm::event::KeyCode;

use crate::app::Route;
use crate::app_tests::H;



#[test]
fn ctrl_d_is_data_structure_and_alt_d_duplicates() {


    junie_tui::core::keyboard::set_alt_keys(junie_tui::core::keyboard::AltKeys::Delivered);
    let mut h = H::new(200, 40);
    assert_eq!(h.app.route(), Route::Connections);
    h.key(KeyCode::Down);
    let footer = h.text().lines().last().unwrap_or_default().to_owned();
    assert!(footer.contains("Alt+D Duplicate"), "{footer}");
    assert!(!footer.contains("Ctrl+D"), "{footer}");
    let before = h.app.connections.connections.len();
    h.ctrl('d');
    assert_eq!(h.app.connections.connections.len(), before);
    h.alt('d');
    assert_eq!(h.app.connections.connections.len(), before + 1);
    assert!(
        h.app
            .connections
            .connections
            .iter()
            .any(|c| c.name.ends_with("(Copy)"))
    );
}





#[test]
fn alt_hints_hide_on_macos_terminal_and_show_with_the_kitty_protocol() {
    use junie_tui::core::event::{ClientPlatform, Input, Key};
    use junie_tui::core::keyboard::{self, AltKeys, KeyboardProtocol};
    use junie_tui::runtime::Application;
    use ratatui::crossterm::event::KeyModifiers;
    let footer = |h: &H| h.text().lines().last().unwrap_or_default().to_owned();
    let workbench = || {
        let mut w = H::connected(200, 40);
        w.key(KeyCode::Tab);
        w.type_str("SELECT * FROM orders LIMIT 5");
        w.key(KeyCode::Esc);
        w
    };
    keyboard::set_alt_keys(AltKeys::from_env(ClientPlatform::MacOs, |k| {
        (k == "TERM_PROGRAM").then(|| "Apple_Terminal".to_owned())
    }));
    let mut h = H::new(200, 40);
    h.key(KeyCode::Down);
    let f = footer(&h);
    assert!(f.contains("Shift+F10 Menu"), "{f}");
    assert!(!f.contains("Alt+"), "{f}");
    let mut w = workbench();
    let f = footer(&w);
    assert!(f.contains("Shift+F5 Run all"), "{f}");
    assert!(!f.contains("Alt+"), "{f}");
    w.app.handle(Input::Key(Key {
        code: KeyCode::F(5),
        mods: KeyModifiers::SHIFT,
    }));
    assert!(w.wb_query().is_running(), "Shift+F5 runs all");

    keyboard::set_alt_keys(AltKeys::WithProtocol);
    keyboard::session::enter(KeyboardProtocol::On, || true);
    h.draw();
    assert!(footer(&h).contains("Alt+D Duplicate"), "{}", footer(&h));
    let f = footer(&workbench());
    assert!(f.contains("Alt+R Run all"), "{f}");
    keyboard::session::leave();
}





#[test]
fn y_copies_a_cell_and_a_history_query_for_real() {
    use junie_tui::core::edit::clipboard;
    let mut h = H::connected(160, 40);
    let w = h.app.workbench.as_mut().unwrap();
    w.open_table("public", "customers", false);
    let grid = match w.active_tab() {
        Some(crate::workbench::WorkTab::Table(t)) => t.grid.id,
        _ => panic!("a table tab"),
    };
    h.app.shell.focus.focus(grid);
    h.draw();
    clipboard::copy("");
    h.key(KeyCode::Char('y'));
    let cell = clipboard::text().unwrap_or_default();
    assert!(
        !cell.is_empty(),
        "the cell is on the clipboard: {}",
        h.text()
    );
    assert!(
        h.text()
            .contains(&format!("Copied {}", cell.chars().count())),
        "{}",
        h.text()
    );

    h.ctrl('y');
    let sql = match h.wb().active_tab() {
        Some(crate::workbench::WorkTab::History(t)) => t
            .current_entry(&h.app.history)
            .map(|e| e.sql.clone())
            .expect("a history entry"),
        _ => panic!("the history tab: {}", h.text()),
    };
    h.key(KeyCode::Char('y'));
    assert_eq!(clipboard::text().as_deref(), Some(sql.as_str()));
    assert!(h.text().contains("Copied query"), "{}", h.text());
}
