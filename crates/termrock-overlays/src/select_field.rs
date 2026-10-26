use core::fmt;

use ratatui_core::layout::Rect;

use crate::id::Id;
use crate::measure::{Constraints, Size};
use crate::select::{Select, SelectState};
use crate::ui::Ui;
use termrock_collections::collection::KeyFn;
use termrock_runtime::field_control::FieldControl;
use termrock_runtime::rowui::RowFn;

pub struct SelectField<'a, T, K, R> {
    select: Select<'a, T, K, R>,
    items: &'a [T],
}

impl<T, K, R> fmt::Debug for SelectField<'_, T, K, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SelectField")
            .field("select", &self.select.id())
            .field("items", &self.items.len())
            .finish_non_exhaustive()
    }
}

impl<'a, T, K, R> SelectField<'a, T, K, R> {
    pub const fn new(select: Select<'a, T, K, R>, items: &'a [T]) -> Self {
        SelectField { select, items }
    }
}

impl<T, K: KeyFn<T>, R: RowFn<T>> FieldControl for SelectField<'_, T, K, R> {
    type State = SelectState;

    fn id(&self) -> Id {
        self.select.id()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &SelectState) -> Rect {
        self.select.draw(ui, area, st, self.items)
    }

    fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size {
        self.select.measure(ui, c)
    }
}
