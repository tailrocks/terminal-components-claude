//! External compile witness for the three-method `GridModel` contract.

#[path = "fixtures/grid_model.rs"]
mod grid_model;

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use termrock_runtime::Runtime;
use termrock_theme::Theme;

#[test]
fn three_method_grid_model_supports_read_only_entry_points() {
    let area = Rect::new(0, 0, 20, 3);
    let mut runtime = Runtime::new(grid_model::ModelOnlyApp::default(), Theme::junie());
    let _ = runtime.initialize();
    let mut buffer = Buffer::empty(area);
    drop(runtime.draw_buffer(area, &mut buffer));

    let text = buffer
        .content()
        .iter()
        .map(ratatui_core::buffer::Cell::symbol)
        .collect::<String>();
    assert!(text.contains("model only"));
}

// DROPPED: read_only_update_takes_a_shared_model — trybuild compile-fail test
// (ui/grid_editable_update_rejects_shared_model) requires trybuild, unavailable
// in this offline workspace.
