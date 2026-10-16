//! The application shell: behaviour every application gets from the
//! library instead of writing it (docs/architecture/shared-components-plan.md
//! §1.8). This module grows step by step; today it holds the leaf pieces:
//!
//! - [`StatusLine`]: the transient status message and its expiry rule.
//! - [`run`]: start an [`Application`] on a terminal, after discarding
//!   input queued before it started.
//! - [`impl_application!`](crate::impl_application): the
//!   [`Application`] adapter for an app struct with the conventional
//!   `handle`/`render`/`tick_interval` methods and `quit`/`pointer`/`hits`
//!   fields, until `Shell` implements [`Application`] itself (plan S7).

mod status;

pub use status::StatusLine;

use crate::runtime::Application;

/// Run `app` until it asks to quit. Input already queued when the process
/// starts (key presses typed into the launching shell) is discarded first,
/// so it cannot act on the first screen or skip an opening sequence.
pub fn run(app: &mut impl Application) -> std::io::Result<()> {
    let _ = crate::runtime::drain_pending_input();
    crate::runtime::run(app)
}

/// Implement [`Application`](crate::runtime::Application) for an app struct
/// that has inherent `handle(&mut self, Input) -> Outcome`,
/// `render(&mut self, &mut Frame)` and `tick_interval(&self) -> Duration`
/// methods and `quit: bool`, `pointer: PointerState` and
/// `hits: HitRegistry` fields. Pointer and caret shapes come from
/// [`shapes_for`](crate::core::shape::shapes_for).
///
/// An adapter until the shell implements `Application` (plan S7).
#[macro_export]
macro_rules! impl_application {
    ($app:ty) => {
        impl $crate::runtime::Application for $app {
            fn handle(
                &mut self,
                input: $crate::core::event::Input,
            ) -> $crate::core::event::Outcome {
                <$app>::handle(self, input)
            }
            fn render(&mut self, frame: &mut ::ratatui::Frame) {
                <$app>::render(self, frame)
            }
            fn should_quit(&self) -> bool {
                self.quit
            }
            fn tick_interval(&self) -> ::std::time::Duration {
                <$app>::tick_interval(self)
            }
            fn shapes(&self) -> $crate::core::shape::Shapes {
                $crate::core::shape::shapes_for(&self.pointer, &self.hits)
            }
        }
    };
}
