use crate::author::{Measure, Rect, Ui};
use crate::measure::{Constraints, Size};

pub trait CompositionLeaf: Measure {
    fn draw(&self, ui: &mut Ui<'_>, logical: Rect);
}

pub struct ClosureLeaf<M, D> {
    measure: M,
    draw: D,
}

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
