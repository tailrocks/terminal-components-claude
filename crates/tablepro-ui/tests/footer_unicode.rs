#[path = "common/tab_navigation.rs"]
mod tab_navigation;

use tablepro_ui::{CONNECTION_NAME, TableProApp};
use termrock::{Id, KeyCode, Part, Theme};
use termrock_test_support::Harness;
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
        assert!(tab_navigation::tab_to(&mut h, CONNECTION_NAME));
        let _ = h.key(KeyCode::End);
        for _ in 0..16 {
            let _ = h.key(KeyCode::Backspace);
        }
        let _ = h.type_str(name);
        let _ = h.key(KeyCode::Enter);
        assert!(tab_navigation::tab_to(
            &mut h,
            Id::root("tablepro.connections.form")
                .part(Part::ACTIONS)
                .index(3)
        ));
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
