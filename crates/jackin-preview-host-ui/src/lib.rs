#![doc = r" Route-owned compositions for Jackin overlays: host chrome screens"]
#![doc = r" (manager, editor, accounts, cockpit, settings, usage)."]
#![doc = r""]
#![doc = r" The application shell owns layer lifetime and focus.  These modules own"]
#![doc = r" only the durable domain state needed by a modal, so a redraw cannot mutate"]
#![doc = r" a selection or copy credential material out of a provider operation."]
#![forbid(unsafe_code)]
#![expect(
    clippy::pedantic,
    reason = "fixture and rendering code favors explicit deterministic data"
)]
#![expect(
    clippy::arithmetic_side_effects,
    reason = "all arithmetic is over bounded deterministic fixture values"
)]
#![cfg_attr(
    test,
    allow(
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic,
        clippy::unwrap_used
    )
)]

pub mod screens;
