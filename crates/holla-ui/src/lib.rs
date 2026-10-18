#![doc = r" Holla UI and terminal application: routes, screens and the deterministic"]
#![doc = r" `App` model, composed on the shared `termrock` facade."]

pub mod app;
pub mod cli;
pub mod dispatch;
pub mod screens;

pub mod clock {
    pub use holla_sim::clock::*;
}
pub mod scenario {
    pub use holla_sim::scenario::*;
}
pub mod domain {
    pub use holla_catalog::ranking;
    pub use holla_domain::*;
    pub use holla_plan::plan;
    pub use holla_sim::fixtures;
}
pub mod sim {
    pub use holla_sim::*;
}

pub use app::App;
pub use holla_sim::scenario::{Motion, Scenario};
