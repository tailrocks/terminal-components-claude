//! Compile-time consumer coverage for adapters to the shared composition API.
use ratatui_core::layout::Rect;
use termrock_layout::{ChildPolicy, Composition, Constraints, Size};
use termrock_runtime::Ui;
use termrock_runtime::author::{Measure, composition_leaf};
use termrock_runtime::composition::{CompositionSlots, ContentSlot};
use termrock_shell::{AppShell, AppShellSlots};

fn consumer(ui: &Ui<'_>) {
    let leaf = composition_leaf(
        |_: &Ui<'_>, constraints: Constraints| Size {
            min: (1, 1),
            preferred: constraints.max,
        },
        |_: &mut Ui<'_>, _: Rect| {},
    );

    // `AppShellSlots::slots` keeps its former CompositionSlots return type;
    // callers can continue to configure and nest it directly.
    let builder = AppShell::new()
        .with_slots()
        .header(&leaf)
        .body(&leaf)
        .footer(&leaf);
    let explicit: CompositionSlots<'_> = builder.slots();
    let configured = explicit.spec(Composition::new().gap(2));
    let nested = ContentSlot::nested(configured, ChildPolicy::default().flex(1));
    let _outer = CompositionSlots::column([nested]);

    // Direct shell slots now own one canonical `ShellSlots` leaf. Its
    // explicit adapter gives legacy CompositionSlots consumers the same
    // resolved chrome geometry, without a second renderer.
    let shell_slots = AppShell::new().slots(&leaf, &leaf, &leaf);
    let shell_group: CompositionSlots<'_> = shell_slots.composition();
    let _nested_shell = ContentSlot::nested(shell_group, ChildPolicy::default().flex(1));

    let measured: Size = Measure::measure(&shell_slots, ui, Constraints::loose(40, 20));
    assert!(measured.min.1 >= 5);
}

#[test]
fn public_composition_adapters_compile_and_measure() {
    // Exercise the consumer function's type contracts in this integration
    // crate, which depends on `termrock-shell` exactly as an external user.
    fn assert_consumer(_: fn(&Ui<'_>)) {}
    assert_consumer(consumer);
    let _: Option<AppShellSlots<'static>> = None;
}
