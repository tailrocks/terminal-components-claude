//! Explicit picker widths retain shared placement, selection, and resizing.
use junie_tui::{
    App, Cx, Id, Item, ItemKey, KeyCode, Picker, PickerAction, PickerState, Response, Theme, Ui,
    UpdateCause,
};
use junie_tui_testing::Harness;
const ID: Id = Id::root("picker.width");
const ITEMS: &[Item<'static>] = &[
    Item::new(ItemKey::num(0), "Run").detail("cargo build"),
    Item::new(ItemKey::num(1), "Set alias…").detail("teach a short name · the query matches it"),
];
fn picker(width: Option<u16>) -> Picker<'static, Item<'static>> {
    let picker = Picker::new(ID).title("Actions");
    if let Some(width) = width {
        picker.width(width)
    } else {
        picker
    }
}
struct Page {
    state: PickerState,
    width: Option<u16>,
    chosen: Option<ItemKey>,
}
impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if cx.update_cause() == UpdateCause::Bootstrap {
            cx.open_layer(ID, picker(self.width).layer(cx, ITEMS));
        }
        let mut response = picker(self.width).update(cx, &mut self.state, ITEMS);
        if let Some(PickerAction::Chosen(key)) = response.take_action() {
            self.chosen = Some(key);
        }
        response.erase()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        ui.layer(ID, |ui, area| {
            picker(self.width).draw(ui, area, &self.state, ITEMS)
        });
    }
}
fn page(width: Option<u16>, screen: u16) -> Harness<Page> {
    Harness::new(
        Page {
            state: PickerState::default(),
            width,
            chosen: None,
        },
        Theme::junie(),
        screen,
        20,
    )
}
#[test]
fn explicit_width_preserves_labels_and_keyboard_selection() {
    let mut h = page(Some(64), 120);
    assert_eq!(h.layer_area(ID).map(|area| area.width), Some(64));
    assert!(h.text().contains("Set alias…"), "{}", h.text());
    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().chosen, Some(ItemKey::num(1)));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
#[test]
fn default_buffer_matches_explicit_default_width() {
    let default = page(None, 120);
    let width = default.layer_area(ID).map(|area| area.width);
    assert!(width.is_some());
    let explicit = page(width, 120);
    assert_eq!(default.buffer(), explicit.buffer());
    assert_eq!(default.focus(), explicit.focus());
}
#[test]
fn layer_resolver_clamps_and_resize_restores_requested_width() {
    let mut h = page(Some(64), 120);
    for screen in [1, 5, 20, 80, 120] {
        let _ = h.resize(screen, 20);
        let area = h.layer_area(ID);
        assert!(area.is_some());
        if let Some(area) = area {
            assert!(area.right() <= screen);
            assert!(area.width <= screen.min(64));
            if screen >= 80 {
                assert_eq!(area.width, 64);
            }
        }
    }
    assert!(h.text().contains("Set alias…"));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
