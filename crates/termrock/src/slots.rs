use ratatui_core::layout::Rect;

use crate::author::LayerSlotAuthor;
use crate::{
    AsItem, Dialog, DialogState, FormDialog, FormDialogState, LayerSlot, Picker, PickerState,
    PropsList, PropsRow, PropsState, RowFn,
};

/// Typed consumer methods for drawing a dialog in a resolved layer slot.
pub trait DialogSlotExt {
    /// Draw a dialog and compose its body through a nested component slot.
    fn dialog(
        &mut self,
        dialog: &Dialog<'_>,
        state: &DialogState,
        body: impl FnOnce(&mut LayerSlot<'_, '_>),
    );
}

impl DialogSlotExt for LayerSlot<'_, '_> {
    fn dialog(
        &mut self,
        dialog: &Dialog<'_>,
        state: &DialogState,
        body: impl FnOnce(&mut LayerSlot<'_, '_>),
    ) {
        self.render_component(|ui, area| dialog.draw_body_slot(ui, area, state, body));
    }
}

/// Typed consumer method for drawing a reusable form dialog in its resolved slot.
pub trait FormDialogSlotExt {
    fn form_dialog<D: crate::forms::FormData + ?Sized>(
        &mut self,
        component: &FormDialog<'_>,
        state: &FormDialogState,
        data: &D,
    );
}

impl FormDialogSlotExt for LayerSlot<'_, '_> {
    fn form_dialog<D: crate::forms::FormData + ?Sized>(
        &mut self,
        component: &FormDialog<'_>,
        state: &FormDialogState,
        data: &D,
    ) {
        self.render_component(|ui, area| component.draw(ui, area, state, data));
    }
}

/// Typed consumer methods for drawing property rows in a resolved slot.
pub trait PropsSlotExt {
    /// Draw keyed property rows with the component's existing renderer.
    fn props_list(
        &mut self,
        component: &PropsList<'_>,
        state: &PropsState,
        rows: &[PropsRow<'_>],
    ) -> Rect;
}

impl PropsSlotExt for LayerSlot<'_, '_> {
    fn props_list(
        &mut self,
        component: &PropsList<'_>,
        state: &PropsState,
        rows: &[PropsRow<'_>],
    ) -> Rect {
        self.render_component(|ui, area| component.draw(ui, area, state, rows))
    }
}

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
