use super::Ui;

use crate::measure::{Constraints, Size};

pub trait Measure {
    fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size;
}
