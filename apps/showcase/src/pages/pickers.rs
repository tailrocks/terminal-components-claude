//! Searchable semantic picker with query and scope state.

use tui_next::{Button, Cx, Id, Item, ItemKey, Picker, PickerAction, PickerState, Rect, Response, Ui, Variant, id, layout};

use super::{Page, frame, lines};

const OPEN: Id = id!("pickers.open");
const PICKER: Id = id!("pickers.layer");
const SCOPES: &[tui_next::ScopeKey] = &[tui_next::ScopeKey::new(1), tui_next::ScopeKey::new(2)];
const ITEMS: &[Item<'static>] = &[
    Item::new(ItemKey::Num(1), "Deploy production").glyph("▶").detail("release pipeline").tag("run").group("Actions"),
    Item::new(ItemKey::Num(2), "Open pull request").glyph("↗").detail("review changes").tag("review").group("Actions"),
    Item::new(ItemKey::Num(3), "Inspect logs").glyph("≡").detail("workspace output").tag("debug").group("Navigation"),
    Item::new(ItemKey::Num(4), "Rotate credentials").glyph("◆").detail("security settings").tag("secure").group("Navigation"),
    Item::new(ItemKey::Num(5), "Delete branch").glyph("×").detail("destructive action").tag("danger").disabled(true).group("Actions"),
];

fn picker() -> Picker<'static, Item<'static>> {
    Picker::new(PICKER).title("Command palette").placeholder("Search commands…").scopes(SCOPES)
}

/// The picker owns query/cursor state while the app owns the selected result.
#[derive(Debug, Default)]
pub(crate) struct PickersPage {
    state: PickerState,
    open: bool,
    result: String,
}

impl PickersPage {
    pub(crate) fn new() -> Self { Self { state: PickerState::default(), open: false, result: String::from("none") } }
}

impl Page for PickersPage {
    fn title(&self) -> &'static str { "Pickers" }

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut result = Response::ignored();
        if cx.is_open(PICKER) {
            self.open = true;
        } else if self.open {
            self.open = false;
        }
        let open = Button::new(OPEN, "Open command palette").variant(Variant::PRIMARY).update(cx);
        if open.activated() && !cx.is_open(PICKER) {
            self.open = true;
            cx.open_layer(PICKER, picker().layer(cx, ITEMS));
        }
        result |= open.erase();
        if cx.is_open(PICKER) {
            let action = picker().update(cx, &mut self.state, ITEMS);
            if let Some(action) = action.action_ref() {
                match action {
                    PickerAction::Chosen(key) | PickerAction::ChosenAlt(key) | PickerAction::Secondary(key) => {
                        if let Some(item) = ITEMS.iter().find(|item| item.key == *key) {
                            self.result = item.label.to_owned();
                        }
                        cx.close_layer(PICKER, None);
                    }
                    PickerAction::QueryChanged | PickerAction::Back | PickerAction::Scope(_) => {}
                }
            }
            result |= action.erase();
        }
        result
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(ui, area, self.title(), "semantic items · query · scope-aware modal", |ui, body| {
            let (button, note) = layout::split_v(body, 4);
            Button::new(OPEN, "Open command palette").variant(Variant::PRIMARY).draw(ui, button);
            let query = format!("last result: {} · query: {}", self.result, self.state.query());
            let _ = ui.paint_str(note, &query, ui.surface_style());
            lines(ui, Rect { y: note.y.saturating_add(1), height: note.height.saturating_sub(1), ..note }, &["The picker filters semantic labels and returns stable ItemKey values."]);
        });
        ui.layer(PICKER, |ui, layer| {
            picker().draw(ui, layer, &self.state, ITEMS);
        });
    }
}
