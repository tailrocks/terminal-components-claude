//! The control side of `Field<C>` (`COMPONENT_ARCHITECTURE.md` §15, §21 item 7).

use ratatui_core::layout::Rect;

use crate::id::Id;
use crate::measure::{Constraints, Size};
use crate::response::StateFlags;
use crate::ui::Ui;

/// Draw-time chrome only. `Field` never registers a focus stop and never
/// runs `update`; the control keeps its own `Id` and its own `update`, and
/// the chrome's parts are registered as `Decorative` regions under this id.
pub trait FieldControl {
    /// The control's durable state type.
    type State;

    /// The control's id; the chrome registers under it.
    fn id(&self) -> Id;

    /// Draw the control.
    fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &Self::State) -> Rect;

    /// Measure the control.
    fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size;

    /// Adopt an owning container's forced state (A11 composition, §12.1).
    ///
    /// A container that can be forced into a reference state forces every
    /// component it owns: a forced rendering is a picture, not a control, at
    /// any depth. `None` — the container is not forced — is the identity,
    /// and so is the default implementation, so a control with no overrides
    /// is unaffected.
    #[must_use]
    fn inherit_forced(self, s: Option<StateFlags>) -> Self
    where
        Self: Sized,
    {
        let _ = s;
        self
    }
}
