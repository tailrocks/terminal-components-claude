use ratatui_core::layout::Rect;
use termrock_core::{Id, ItemKey, Response, StateFlags, Variant};
use termrock_runtime::{App, Cx, Family, RowUi, Ui};
use termrock_test_support::Harness;
use termrock_theme::Theme;

const ID: Id = Id::root("rowui.metadata.gap");
struct Page {
    gap: u16,
}

impl App for Page {
    fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let area = Rect::new(4, 1, 16, 1);
        let mut row = RowUi::new(
            ui,
            ID,
            Family::TREE,
            Variant::DEFAULT,
            StateFlags::empty(),
            ItemKey::num(1),
            area,
        );
        if self.gap == 1 {
            row.meta("ID");
        } else {
            row.meta_with_gap("ID", self.gap);
        }
        row.label("abcdefghijklmnopqrstuvwxyz");
    }
}

#[test]
fn metadata_gap_is_configurable_and_the_default_stays_one_cell() {
    for gap in [1, 2] {
        let h = Harness::new(Page { gap }, Theme::junie(), 24, 4);
        assert_eq!(h.cell(18, 1).symbol(), "I");
        assert_eq!(h.cell(19, 1).symbol(), "D");
        if gap == 1 {
            assert_eq!(h.cell(16, 1).symbol(), "…");
            assert_eq!(h.cell(17, 1).symbol(), " ");
        } else {
            assert_eq!(h.cell(17, 1).symbol(), " ");
            assert_eq!(h.cell(16, 1).symbol(), " ");
            assert_eq!(h.cell(15, 1).symbol(), "…");
        }
    }
}
