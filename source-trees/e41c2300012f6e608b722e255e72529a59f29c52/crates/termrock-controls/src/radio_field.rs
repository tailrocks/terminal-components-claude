//! `RadioField` — a [`RadioGroup`] bound to its items as a [`FieldControl`].

use core::fmt;

use ratatui_core::layout::Rect;

use super::choice::{RadioGroup, RadioGroupState};
use crate::collection::{KeyFn, RowFn};
use crate::field_control::FieldControl;
use crate::id::Id;
use crate::measure::{Constraints, Size};
use crate::ui::Ui;

/// A `Field` control over a fully-built [`RadioGroup`] and its items.
///
/// `RadioGroup::draw` needs the option slice every frame, so a bare group
/// cannot implement [`FieldControl`] on its own; this adapter carries both
/// halves. The caller builds the group first (`.value()` and any patches
/// are owned by `RadioGroup`), then binds it to the items:
///
/// ```ignore
/// Field::new("Environment", RadioField::new(RadioGroup::new(ENV).value(k), &envs))
/// ```
///
/// Identity, drawing and measurement are pure passthrough to the wrapped
/// group; the chrome registers under the group's own id.
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
    /// Bind a fully-built `RadioGroup` to the items it draws.
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

#[cfg(test)]
mod tests {
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::Position;

    use super::*;
    use crate::id::ItemKey;
    use crate::runtime::Runtime;
    use crate::runtime::stub::{SCREEN, Stub};
    use crate::theme::Theme;

    const RG: Id = Id::root("radio_field.tests");

    fn scene() -> (Runtime<Stub>, Buffer) {
        (
            Runtime::new(Stub::default(), Theme::junie()),
            Buffer::empty(SCREEN),
        )
    }

    fn cell_symbol(buf: &Buffer, x: u16, y: u16) -> &str {
        buf.cell(Position::new(x, y))
            .map(|cell| cell.symbol())
            .unwrap_or("<none>")
    }

    #[test]
    fn id_matches_the_wrapped_group() {
        let items = ["local", "staging"];
        let field = RadioField::new(RadioGroup::new(RG), &items);
        assert_eq!(field.id(), RG);
    }

    #[test]
    fn measure_matches_the_wrapped_group() {
        let items = ["local", "staging"];
        let (mut rt, mut buf) = scene();
        rt.draw_scene(SCREEN, &mut buf, |ui, _| {
            for c in [
                Constraints::loose(28, 4),
                Constraints::loose(29, 5),
                Constraints::tight(20, 2),
            ] {
                let group: RadioGroup<'_, &str> = RadioGroup::new(RG);
                let field = RadioField::new(RadioGroup::new(RG), &items);
                assert_eq!(
                    field.measure(ui, c),
                    group.measure(ui, c),
                    "passthrough for {c:?}"
                );
            }
        })
        .commit_presented();
    }

    #[test]
    fn draw_matches_the_bare_group() {
        let items = ["local", "staging"];
        let mut st = RadioGroupState::default();
        st.set_cursor(1, ItemKey::index(1));

        // the bare group rows are the oracle: the adapter must add nothing
        let (mut rt, mut bare) = scene();
        rt.draw_scene(SCREEN, &mut bare, |ui, _| {
            let group: RadioGroup<'_, &str> = RadioGroup::new(RG).value(ItemKey::index(0));
            let painted = group.draw(ui, Rect::new(2, 2, 30, 2), &st, &items);
            assert_eq!(painted, Rect::new(2, 2, 30, 2));
        })
        .commit_presented();

        let (mut rt, mut buf) = scene();
        rt.draw_scene(SCREEN, &mut buf, |ui, _| {
            let group: RadioGroup<'_, &str> = RadioGroup::new(RG).value(ItemKey::index(0));
            let used = RadioField::new(group, &items).draw(ui, Rect::new(2, 2, 30, 2), &st);
            assert_eq!(used, Rect::new(2, 2, 30, 2));
        })
        .commit_presented();

        assert_eq!(cell_symbol(&buf, 3, 2), "(", "on-marker head");
        for y in 2..4 {
            for x in 2..32 {
                assert_eq!(
                    cell_symbol(&buf, x, y),
                    cell_symbol(&bare, x, y),
                    "control cell ({x},{y})"
                );
            }
        }
    }
}
