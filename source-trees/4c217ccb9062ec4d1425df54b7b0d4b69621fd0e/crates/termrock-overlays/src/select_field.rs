//! `SelectField` — a [`Select`] bound to its items as a [`FieldControl`].

use core::fmt;

use ratatui_core::layout::Rect;

use super::select::{Select, SelectState};
use crate::collection::{KeyFn, RowFn};
use crate::id::Id;
use crate::measure::{Constraints, Size};
use crate::ui::{FieldControl, Ui};

/// A `Field` control over a fully-built [`Select`] and its items.
///
/// `Select::draw` needs the option slice every frame, so a bare `Select`
/// cannot implement [`FieldControl`] on its own; this adapter carries both
/// halves. The caller builds the `Select` first (`.key()`, `.row()` and any
/// patches are non-`const` generic transforms owned by `Select`), then binds
/// it to the items:
///
/// ```ignore
/// Field::new("Column", SelectField::new(Select::new(COL).key(k).row(r), &columns))
/// ```
///
/// Identity, drawing and measurement are pure passthrough to the wrapped
/// `Select`; the chrome registers under the select's own id.
pub struct SelectField<'a, T, K, R> {
    select: Select<'a, T, K, R>,
    items: &'a [T],
}

impl<T, K, R> fmt::Debug for SelectField<'_, T, K, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SelectField")
            .field("select", &self.select.id())
            .field("items", &self.items.len())
            .finish_non_exhaustive()
    }
}

impl<'a, T, K, R> SelectField<'a, T, K, R> {
    /// Bind a fully-built `Select` to the items it draws.
    pub const fn new(select: Select<'a, T, K, R>, items: &'a [T]) -> Self {
        SelectField { select, items }
    }
}

impl<T, K: KeyFn<T>, R: RowFn<T>> FieldControl for SelectField<'_, T, K, R> {
    type State = SelectState;

    fn id(&self) -> Id {
        self.select.id()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &SelectState) -> Rect {
        self.select.draw(ui, area, st, self.items)
    }

    fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size {
        self.select.measure(ui, c)
    }
}

#[cfg(test)]
mod tests {
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::Position;

    use super::*;
    use crate::field::Field;
    use crate::id::ItemKey;
    use crate::runtime::Runtime;
    use crate::runtime::stub::{SCREEN, Stub};
    use crate::theme::{GlyphRole, Theme};

    const SEL: Id = Id::root("select_field.tests");

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

    fn row_text(buf: &Buffer, y: u16, x0: u16, len: u16) -> String {
        (x0..x0.saturating_add(len))
            .map(|x| cell_symbol(buf, x, y))
            .collect()
    }

    #[test]
    fn id_matches_the_wrapped_select() {
        let items = ["alpha", "beta"];
        let field = SelectField::new(Select::new(SEL), &items);
        assert_eq!(field.id(), SEL);
    }

    #[test]
    fn measure_matches_the_wrapped_select() {
        let items = ["alpha", "beta"];
        let (mut rt, mut buf) = scene();
        rt.draw_scene(SCREEN, &mut buf, |ui, _| {
            for c in [
                Constraints::loose(28, 2),
                Constraints::loose(29, 3),
                Constraints::tight(20, 1),
            ] {
                let select: Select<'_, &str> = Select::new(SEL);
                let field = SelectField::new(Select::new(SEL), &items);
                assert_eq!(
                    field.measure(ui, c),
                    select.measure(ui, c),
                    "passthrough for {c:?}"
                );
            }
            let select: Select<'_, &str> = Select::new(SEL);
            let field = SelectField::new(select, &items);
            assert_eq!(field.measure(ui, Constraints::loose(28, 2)).preferred.1, 1);
        })
        .commit_presented();
    }

    #[test]
    fn field_chrome_wraps_the_closed_row_unchanged() {
        let items = ["alpha", "beta"];
        let mut st = SelectState::default();
        st.set_value(Some(ItemKey::index(0)));

        // the bare control row is the oracle: the adapter must add nothing
        let (mut rt, mut bare) = scene();
        rt.draw_scene(SCREEN, &mut bare, |ui, _| {
            let select: Select<'_, &str> = Select::new(SEL);
            let painted = select.draw(ui, Rect::new(2, 2, 30, 1), &st, &items);
            assert_eq!(painted, Rect::new(2, 2, 30, 1));
        })
        .commit_presented();

        let (mut rt, mut buf) = scene();
        rt.draw_scene(SCREEN, &mut buf, |ui, _| {
            let select: Select<'_, &str> = Select::new(SEL);
            let used = Field::new("Column", SelectField::new(select, &items))
                .plain(true)
                .optional_suffix(false)
                .draw(ui, Rect::new(2, 1, 30, 3), &st);
            assert_eq!(used, Rect::new(2, 1, 30, 3));
        })
        .commit_presented();

        // label row through the chrome
        assert_eq!(row_text(&buf, 1, 4, 6), "Column");
        // gutter / value / marker cells
        assert_eq!(cell_symbol(&buf, 2, 2), cell_symbol(&bare, 2, 2), "gutter");
        assert_eq!(row_text(&buf, 2, 4, 5), "alpha", "value");
        let marker = Theme::junie().design.glyphs.get(GlyphRole::SelectClosed);
        assert_eq!(cell_symbol(&buf, 30, 2), marker, "marker");
        // the whole control row is pixel-identical to the bare select
        for x in 2..32 {
            assert_eq!(
                cell_symbol(&buf, x, 2),
                cell_symbol(&bare, x, 2),
                "control cell x={x}"
            );
        }
    }
}
