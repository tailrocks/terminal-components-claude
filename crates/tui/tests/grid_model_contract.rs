//! External compile witness for the three-method `GridModel` contract.

#[path = "fixtures/grid_model.rs"]
mod grid_model;

use tui_next::{Buffer, Rect, Runtime, Theme};

#[test]
fn three_method_grid_model_supports_read_only_entry_points() {
    let area = Rect::new(0, 0, 20, 3);
    let mut runtime = Runtime::new(grid_model::ModelOnlyApp::default(), Theme::junie());
    let mut buffer = Buffer::empty(area);
    runtime.draw_buffer(area, &mut buffer);

    let text = buffer
        .content()
        .iter()
        .map(tui_next::Cell::symbol)
        .collect::<String>();
    assert!(text.contains("model only"));
}

#[test]
fn read_only_update_takes_a_shared_model() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/grid_editable_update_rejects_shared_model.rs");
}
