//! Optional component measurement against the live `Ui`.

use super::Ui;

use crate::measure::{Constraints, Size};

/// Optional measurement for components that can size themselves.
pub trait Measure {
    /// Measure against the design tokens and the offered constraints.
    fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size;
}
