use ratatui_core::layout::Rect;
use termrock::{AppShell, Ui};
use termrock_layout::{Composition, Constraints, Size};
use termrock_runtime::author::composition_leaf;
use termrock_runtime::composition::CompositionSlots;

#[test]
fn direct_slots_keeps_explicit_composition_type() {
    let leaf = composition_leaf(
        |_: &Ui<'_>, _: Constraints| Size {
            min: (0, 0),
            preferred: (1, 1),
        },
        |_: &mut Ui<'_>, _: Rect| {},
    );
    let group: CompositionSlots<'_> = AppShell::new().slots(&leaf, &leaf, &leaf);
    let _ = group.spec(Composition::new());
}

#[test]
fn temporary_builder_can_release_after_returning_slots() {
    let leaf = composition_leaf(
        |_: &Ui<'_>, _: Constraints| Size {
            min: (0, 0),
            preferred: (1, 1),
        },
        |_: &mut Ui<'_>, _: Rect| {},
    );
    let group: CompositionSlots<'_> = AppShell::new().with_slots().body(&leaf).slots();
    let _ = group.spec(Composition::new());
}
