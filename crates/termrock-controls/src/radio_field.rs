use core::fmt;

use ratatui_core::layout::Rect;

use crate::choice::{RadioGroup, RadioGroupState};
use crate::field_control::FieldControl;
use crate::id::Id;
use crate::measure::{Constraints, Size};
use crate::ui::Ui;
use termrock_collections::collection::KeyFn;
use termrock_runtime::rowui::RowFn;

pub struct RadioField<'a, T, K, R> {
    group: RadioGroup<'a, T, K, R>,
    items: &'a [T],
}

impl<T, K, R> fmt::Debug for RadioField<'_, T, K, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RadioField")
            .field("group", &self.group.id())
            .field("items", &self.items.len())
            .finish_non_exhaustive()
    }
}

impl<'a, T, K, R> RadioField<'a, T, K, R> {
    pub const fn new(group: RadioGroup<'a, T, K, R>, items: &'a [T]) -> Self {
        RadioField { group, items }
    }
}

impl<T, K: KeyFn<T>, R: RowFn<T>> FieldControl for RadioField<'_, T, K, R> {
    type State = RadioGroupState;

    fn id(&self) -> Id {
        self.group.id()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &RadioGroupState) -> Rect {
        self.group.draw(ui, area, st, self.items)
    }

    fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size {
        self.group.measure(ui, c)
    }
}
