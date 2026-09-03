//! `COMPONENT_ARCHITECTURE.md` §17 example 5, verbatim (crate name is temporary: `tui_next` → `junie_tui` at Slice 5).
#![expect(
    missing_docs,
    clippy::indexing_slicing,
    reason = "verbatim from §17 example 5"
)]

use tui_next::{
    Button, FrameRead, Id, Part, Rect, Role, RowAlign, StylePatch, Ui, Variant, id, layout,
};

const OK: Id = id!("ok");
const RESET: Id = id!("reset");

// One patch, declared `const`, so it costs nothing per frame.
const RESET_LABEL: [(Part, StylePatch); 2] = [
    (Part::LABEL, StylePatch::new().set_fg(Role::Warning)),
    (Part::GUTTER, StylePatch::new().set_fg(Role::Warning)),
];

pub fn draw_actions(ui: &mut Ui<'_>, area: Rect) {
    let cols = layout::action_row(area, &[10, 12], ui.design().space.gap, RowAlign::End); // RowAlign::{Start, End} (§22)
    Button::new(OK, "OK")
        .variant(Variant::PRIMARY)
        .draw(ui, cols[0]);
    Button::new(RESET, "Reset")
        .patch_part(&RESET_LABEL)
        .draw(ui, cols[1]);
}

fn main() {}
// Both buttons use the same global theme and the same renderer; only one is patched,
// and `conformance::button::local_override_does_not_mutate_the_theme` proves the
// theme is byte-identical afterwards.
