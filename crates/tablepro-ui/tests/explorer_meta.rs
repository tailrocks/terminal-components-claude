use tablepro_ui::{Surface, TableProApp};
use termrock::Theme;
use termrock_test_support::Harness;

fn app(width: u16, height: u16) -> Harness<TableProApp> {
    let mut app = TableProApp::default();
    app.set_surface(Surface::WorkbenchDefault);
    Harness::new(app, Theme::junie(), width, height)
}

#[test]
fn explorer_metadata_hides_on_every_row_when_one_row_cannot_fit_it() {
    // The narrow explorer cannot fit "T organizations" and its metadata side
    // by side, so no row shows metadata.
    let narrow = app(120, 40);
    let group = narrow.row(9);
    let object = narrow.row(13);
    assert!(group.contains("Tables"), "{group:?}");
    assert!(!group.contains('7'), "group metadata stayed: {group:?}");
    assert!(object.contains("T orders"), "{object:?}");
    assert!(
        !object.contains("1.2 M"),
        "object metadata stayed: {object:?}"
    );
}

#[test]
fn explorer_metadata_shows_when_every_row_fits_it() {
    // The wide explorer fits the longest label and its metadata, so the rows
    // carry their metadata.
    let wide = app(100, 30);
    let group = wide.row(9);
    let object = wide.row(10);
    assert!(group.contains("Tables"), "{group:?}");
    assert!(group.contains('7'), "group metadata missing: {group:?}");
    assert!(object.contains("T organizations"), "{object:?}");
    assert!(
        object.contains("1.2 k"),
        "object metadata missing: {object:?}"
    );
}
