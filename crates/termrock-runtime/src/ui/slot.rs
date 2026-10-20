use ratatui_core::layout::Rect;

use crate::id::Id;

use super::Ui;

pub struct LayerSlot<'ui, 'frame> {
    ui: &'ui mut Ui<'frame>,
    area: Rect,
}

impl<'ui, 'frame> LayerSlot<'ui, 'frame> {
    pub(crate) const fn new(ui: &'ui mut Ui<'frame>, area: Rect) -> Self {
        Self { ui, area }
    }
}

impl crate::author::LayerSlotAuthor for LayerSlot<'_, '_> {
    fn render_component<R>(&mut self, render: impl FnOnce(&mut Ui<'_>, Rect) -> R) -> R {
        render(self.ui, self.area)
    }
}

impl crate::author::NestedLayerSlotAuthor for Ui<'_> {
    fn with_nested_layer_slot<R>(
        &mut self,
        area: Rect,
        render: impl FnOnce(&mut LayerSlot<'_, '_>) -> R,
    ) -> R {
        self.with_area(area, |ui| {
            let mut slot = LayerSlot::new(ui, area);
            render(&mut slot)
        })
    }
}

impl<'f> Ui<'f> {
    pub fn layer_slot<R>(
        &mut self,
        id: Id,
        render: impl FnOnce(&mut LayerSlot<'_, '_>) -> R,
    ) -> Option<R> {
        self.layer(id, |ui, area| {
            let mut slot = LayerSlot::new(ui, area);
            render(&mut slot)
        })
    }
}
