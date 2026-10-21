use crate::focus::{ScopeId, ScopeMode};
use crate::id::Id;
use crate::keymap::Binding;
use crate::response::StateFlags;

use super::Ui;

impl Ui<'_> {
    /// Compose a real focus scope whose unhandled commands may be resolved by
    /// the supplied bubbling bindings after its focused descendant declines.
    ///
    /// The closure must draw the scope's actual controls. This helper does not
    /// create a focus target or alter global key handling. Pass an empty table
    /// when this configuration has no scope-level fallback commands.
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
