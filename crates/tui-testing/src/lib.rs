//! Test support for `junie-tui` (`COMPONENT_ARCHITECTURE.md` §16): the
//! deterministic [`Harness`], the headless digest [`Scene`], the shared
//! conformance driver and the counting allocator for the perf suite.
//!
//! Dev-only: `publish = false`, depended on with `[dev-dependencies]` only,
//! so nothing here reaches a shipped binary.
#![deny(unsafe_code)]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::missing_panics_doc,
    clippy::arithmetic_side_effects,
    clippy::cast_lossless,
    clippy::many_single_char_names,
    clippy::field_reassign_with_default,
    clippy::struct_excessive_bools,
    clippy::type_complexity,
    clippy::print_stdout,
    clippy::format_push_string,
    reason = "a test-support crate: assertions and indexing are the product, not a hazard"
)]

pub mod conformance;
pub mod digest;
pub mod harness;
pub mod perf;

pub use conformance::{Caps, Conformance, Fixture, FixtureRow};
pub use digest::{Baseline, NoApp, Scene};
pub use harness::Harness;

/// Publish actual application geometry and settle focus before delivering one event.
///
/// Behavioral raw-runtime tests use this migration helper. Lifecycle tests should
/// call `Runtime::handle` directly to inspect retained `PendingInput` values.
/// No initialization or timer event is synthesized.
pub fn deliver<A: junie_tui::App>(
    runtime: &mut junie_tui::Runtime<A>,
    area: junie_tui::Rect,
    input: junie_tui::Input,
) -> junie_tui::Response<()> {
    let mut buffer = ratatui_core::buffer::Buffer::empty(area);
    deliver_buffer(runtime, &mut buffer, input)
}

/// The reusable-buffer variant of [`deliver`] for measured full lifecycle work.
/// Frame preparation, publication and settling remain inside the measured call.
pub fn deliver_buffer<A: junie_tui::App>(
    runtime: &mut junie_tui::Runtime<A>,
    buffer: &mut ratatui_core::buffer::Buffer,
    input: junie_tui::Input,
) -> junie_tui::Response<()> {
    let area = *buffer.area();
    for _ in 0..16 {
        if runtime.needs_settle() {
            let _ = runtime.settle();
        }
        if !runtime.needs_present() && !runtime.needs_settle() {
            return runtime
                .handle(input)
                .expect("published fixture must accept input");
        }
        runtime.draw_buffer(area, buffer).commit_presented();
    }
    panic!(
        "fixture publication did not settle: {:?}",
        runtime.diagnostics()
    );
}
