use crate::focus::{ScopeId, ScopeMode};
use crate::id::Id;
use crate::keymap::Binding;
use crate::response::StateFlags;

use super::Ui;

impl Ui<'_> {
    pub fn command_scope<C: Copy + 'static, R>(
        &mut self,
        owner: Id,
        table: &'static [Binding<C>],
        render: impl FnOnce(&mut Ui<'_>) -> R,
    ) -> R {
        self.focus_scope(owner, ScopeMode::Normal, |ui| {
            let output = render(ui);
            ui.publish_scope_bindings(ScopeId::new(owner), owner, StateFlags::empty(), table);
            output
        })
    }
}
