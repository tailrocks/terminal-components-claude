//! Public-facade consumer using live data and the reusable picker card policy.

use termrock::{
    App, Cx, Id, Item, ItemKey, Picker, PickerAction, PickerLayout, PickerState, Response, Runtime,
    ScreenAlign, Theme, Ui,
};

const PICKER: Id = Id::root("example.picker-card");
const PROJECTS: [Item<'static>; 3] = [
    Item::new(ItemKey::num(41), "Terminal Components"),
    Item::new(ItemKey::num(73), "Tuiscotti"),
    Item::new(ItemKey::num(88), "Termrock Consumer"),
];

struct Consumer {
    state: PickerState,
    opened: bool,
    chosen: Option<ItemKey>,
}

fn picker() -> Picker<'static, Item<'static>> {
    Picker::new(PICKER)
        .title("Choose a workspace")
        .width(72)
        .searchable(true)
        .layout(PickerLayout::card())
        .max_rows(12)
        .align(ScreenAlign::GeometricCenter)
}

impl App for Consumer {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let picker = picker();
        if !self.opened {
            self.opened = true;
            cx.open_layer(PICKER, picker.layer(cx, &PROJECTS));
        }
        let mut response = picker.update(cx, &mut self.state, &PROJECTS);
        if let Some(PickerAction::Chosen(key)) = response.take_action() {
            self.chosen = Some(key);
        }
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let picker = picker();
        ui.layer(PICKER, |ui, area| {
            picker.draw(ui, area, &self.state, &PROJECTS)
        });
    }
}

fn main() {
    let _runtime = Runtime::new(
        Consumer {
            state: PickerState::default(),
            opened: false,
            chosen: None,
        },
        Theme::junie(),
    );
}
