//! Source-qualified production `Id`s.
//!
//! `id!("name")` mixes `module_path!()`, so adapter-local `id!` tokens are a
//! different identity. These roots copy the production module path from
//! `apps/showcase/src/pages/*.rs`.

use junie_tui::Id;

/// `apps/showcase/src/pages/buttons.rs` `id!("buttons")`.
pub const BUTTONS: Id = Id::root("showcase_app::pages::buttons::buttons");

/// `SPECS` index of "Start long job".
pub const LONG_JOB: usize = 8;

/// `SPECS` index of "Run task".
pub const RUN_TASK: usize = 0;
