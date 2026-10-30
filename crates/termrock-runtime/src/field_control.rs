use ratatui_core::layout::Rect;

use crate::id::Id;
use crate::measure::{Constraints, Size};
use crate::ui::Ui;

pub trait FieldControl {
    type State;

    fn id(&self) -> Id;

    fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &Self::State) -> Rect;

    fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size;
}
