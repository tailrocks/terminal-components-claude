use crate::library::{Props, SlotFn, Ui, Widget};

#[derive(Clone, Copy)]
pub struct App { pub value: u32, pub disabled: bool }
impl App {
    fn control_props(disabled: bool) -> Props {
        Props::new(7).disabled(disabled)
    }
    pub fn update(&mut self, ui: &mut Ui) {
        Widget::update(Self::control_props(self.disabled), &mut self.value, ui);
    }
    pub fn draw(&self, busy: bool, slot: Option<(u32, SlotFn)>, ui: &mut Ui) {
        Widget::draw(Self::control_props(self.disabled), self.value, busy, slot, ui);
        crate::library::compose(Self::control_props(self.disabled), ui);
        // Product art deliberately occupies a disjoint cell.
        ui.row(|ui| ui.paint(3, 64));
    }
}
