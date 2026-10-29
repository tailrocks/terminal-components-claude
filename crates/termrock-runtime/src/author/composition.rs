use crate::author::{Measure, Rect, Ui};
use crate::measure::{Constraints, Size};

/// Component-author contract used by the geometry-private composition API.
///
/// Implementations measure without input handling or domain effects. Drawing
/// receives logical bounds while `Ui` clips painting and registration to the
/// resolved visible bounds.
pub trait CompositionLeaf: Measure {
    fn draw(&self, ui: &mut Ui<'_>, logical: Rect);
}

/// Closure-backed component leaf for adapters and focused consumer examples.
pub struct ClosureLeaf<M, D> {
    measure: M,
    draw: D,
}

/// Build a component-author leaf from pure measurement and rendering callbacks.
pub const fn composition_leaf<M, D>(measure: M, draw: D) -> ClosureLeaf<M, D> {
    ClosureLeaf { measure, draw }
}

impl<M, D> Measure for ClosureLeaf<M, D>
where
    M: for<'ui> Fn(&Ui<'ui>, Constraints) -> Size,
{
    fn measure(&self, ui: &Ui<'_>, constraints: Constraints) -> Size {
        (self.measure)(ui, constraints)
    }
}

impl<M, D> CompositionLeaf for ClosureLeaf<M, D>
where
    M: for<'ui> Fn(&Ui<'ui>, Constraints) -> Size,
    D: for<'ui> Fn(&mut Ui<'ui>, Rect),
{
    fn draw(&self, ui: &mut Ui<'_>, logical: Rect) {
        (self.draw)(ui, logical);
    }
}
