#![doc = r" The control side of `Field<C>` (`COMPONENT_ARCHITECTURE.md` §15, §21 item 7)."]

use ratatui_core::layout::Rect;

use crate::id::Id;
use crate::measure::{Constraints, Size};
use crate::ui::Ui;

#[doc = r" Draw-time chrome only. `Field` never registers a focus stop and never"]
#[doc = r" runs `update`; the control keeps its own `Id` and its own `update`, and"]
#[doc = r" the chrome's parts are registered as `Decorative` regions under this id."]
pub trait FieldControl {
    #[doc = r" The control's durable state type."]
    type State;

    #[doc = r" The control's id; the chrome registers under it."]
    fn id(&self) -> Id;

    #[doc = r" Draw the control."]
    fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &Self::State) -> Rect;

    #[doc = r" Measure the control."]
    fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size;
}
