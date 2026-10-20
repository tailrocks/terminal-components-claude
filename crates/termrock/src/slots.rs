use ratatui_core::layout::Rect;

use crate::author::LayerSlotAuthor;
use crate::{AsItem, LayerSlot, Picker, PickerState, RowFn};

/// Typed consumer methods for drawing reusable components in resolved layer
/// slots.
pub trait PickerSlotExt {
    /// Draw an open picker in this slot's resolved area using the picker's
    /// existing renderer.
    fn picker<T: AsItem, R: RowFn<T>>(
        &mut self,
        picker: Picker<'_, T, R>,
        state: &PickerState,
        items: &[T],
    ) -> Rect;
}

impl PickerSlotExt for LayerSlot<'_, '_> {
    fn picker<T: AsItem, R: RowFn<T>>(
        &mut self,
        picker: Picker<'_, T, R>,
        state: &PickerState,
        items: &[T],
    ) -> Rect {
        self.render_component(|ui, area| picker.draw(ui, area, state, items))
    }
}
