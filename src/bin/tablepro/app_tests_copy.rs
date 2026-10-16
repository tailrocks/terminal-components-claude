//! Copy-audit regressions: one key, one meaning across TablePro's screens.

use ratatui::crossterm::event::KeyCode;

use crate::app::Screen;
use crate::app_tests::H;

/// Ctrl+D switches Data / Structure in the workbench; duplicating is Alt+D
/// everywhere, on a connection and on a grid row.
#[test]
fn ctrl_d_is_data_structure_and_alt_d_duplicates() {
    let mut h = H::new(200, 40);
    assert_eq!(h.app.screen, Screen::Connections);
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
