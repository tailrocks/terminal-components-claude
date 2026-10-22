use ratatui_core::layout::Rect;
use termrock_core::id::{Id, ItemKey, Part, PartRef};
use termrock_core::response::Response;
use termrock_navigation::FilterPolicy;
use termrock_overlays::picker::{Item, Picker, PickerLayout, PickerState};
use termrock_runtime::layer::ScreenAlign;
use termrock_runtime::{App, Cx, Ui};
use termrock_test_support::Harness;
use termrock_theme::Theme;

const ID: Id = Id::root("picker.max-rows");
const ITEMS: [Item<'static>; 13] = [
    Item::new(ItemKey::num(0), "Alpha"),
    Item::new(ItemKey::num(1), "Beta"),
    Item::new(ItemKey::num(2), "Gamma"),
    Item::new(ItemKey::num(3), "Delta"),
    Item::new(ItemKey::num(4), "Epsilon"),
    Item::new(ItemKey::num(5), "Zeta"),
    Item::new(ItemKey::num(6), "Eta"),
    Item::new(ItemKey::num(7), "Theta"),
    Item::new(ItemKey::num(8), "Iota"),
    Item::new(ItemKey::num(9), "Kappa"),
    Item::new(ItemKey::num(10), "Lambda"),
    Item::new(ItemKey::num(11), "Mu"),
    Item::new(ItemKey::num(12), "Nu"),
];

struct Page {
    state: PickerState,
    opened: bool,
    layout: PickerLayout,
}

impl Page {
    fn picker(&self) -> Picker<'static, Item<'static>> {
        Picker::new(ID)
            .width(72)
            .filter(FilterPolicy::Caller)
            .layout(self.layout)
            .align(ScreenAlign::GeometricCenter)
    }
}

impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let picker = self.picker();
        if !self.opened {
            self.opened = true;
            cx.open_layer(ID, picker.layer(cx, &ITEMS));
        }
        picker.update(cx, &mut self.state, &ITEMS).erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let picker = self.picker();
        ui.layer(ID, |ui, area| picker.draw(ui, area, &self.state, &ITEMS));
    }
}

fn page(layout: PickerLayout, width: u16, height: u16) -> Harness<Page> {
    Harness::new(
        Page {
            state: PickerState::default(),
            opened: false,
            layout,
        },
        Theme::junie(),
        width,
        height,
    )
}

#[test]
fn max_rows_caps_the_requested_layer_height() {
    let harness = page(PickerLayout::card().max_rows(1), 60, 40);
    let area = harness.layer_area(ID).expect("picker layer is open");

    assert_eq!(area, Rect::new(2, 16, 56, 8));
    assert!(
        harness
            .runtime()
            .registry()
            .area_of_part(ID, PartRef::item(Part::ROW, ItemKey::num(0)))
            .is_some()
    );
}

#[test]
fn zero_max_rows_still_requests_one_result_row() {
    let harness = page(PickerLayout::card().max_rows(0), 60, 40);
    let area = harness.layer_area(ID).expect("picker layer is open");

    // The card adds its border, query, spare row, and footer around one result.
    assert_eq!(area.height, 8);
    assert!(
        harness
            .runtime()
            .registry()
            .area_of_part(ID, PartRef::item(Part::ROW, ItemKey::num(0)))
            .is_some()
    );
}

#[test]
fn max_rows_height_clamps_to_the_card_viewport_margins() {
    let harness = page(PickerLayout::card().max_rows(8), 60, 12);
    let area = harness.layer_area(ID).expect("picker layer is open");

    assert_eq!(area, Rect::new(2, 1, 56, 10));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}
