//! Editable connection names retain whole-run grapheme boundaries in the footer.
use junie_tui::{Id, KeyCode, Part, Theme};
use junie_tui_testing::Harness;
use tablepro_app::{CONNECTION_NAME, TableProApp};
#[test]
fn connection_form_names_preserve_footer_grapheme_boundaries() {
    for name in [
        "\u{0301}lead",
        "\u{200d}👩",
        "界🙂",
        "e\u{0301}",
        "\u{0301}",
        "👩\u{200d}💻",
    ] {
        let mut app = TableProApp::default();
        app.begin_connection_form();
        let mut h = Harness::new(app, Theme::junie(), 120, 40);
        assert!(h.tab_to(CONNECTION_NAME));
        let _ = h.key(KeyCode::End);
        for _ in 0..16 {
            let _ = h.key(KeyCode::Backspace);
        }
        let _ = h.type_str(name);
        let _ = h.key(KeyCode::Enter);
        assert!(
            h.tab_to(
                Id::root("tablepro.connections.form")
                    .part(Part::ACTIONS)
                    .index(3)
            )
        );
        let _ = h.key(KeyCode::Enter);
        assert!(!h.app().connection_form_open());
        assert!(
            h.row(39).contains(&format!("Connected to {name}")),
            "footer lost connection-name grapheme: {:?}",
            h.row(39)
        );
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
}
