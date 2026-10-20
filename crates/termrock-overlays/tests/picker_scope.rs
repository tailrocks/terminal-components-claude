use ratatui_core::layout::Rect;
use termrock_core::id::{Id, ItemKey};
use termrock_core::response::Response;
use termrock_overlays::picker::{Item, Picker, PickerAction, PickerState};
use termrock_runtime::{App, Cx, Ui, UpdateCause};
use termrock_test_support::Harness;
use termrock_theme::Theme;

const ID: Id = Id::root("picker.scope");
const ITEMS: &[Item<'static>] = &[Item::new(ItemKey::num(0), "First")];

struct Page {
    state: PickerState,
    scope: Option<&'static str>,
    width: u16,
    chosen: Option<ItemKey>,
}

impl Page {
    fn picker(&self) -> Picker<'static, Item<'static>> {
        let picker = Picker::new(ID)
            .title("Launch agent")
            .width(self.width)
            .searchable(false);
        if let Some(scope) = self.scope {
            picker.scope_label(scope)
        } else {
            picker
        }
    }
}

impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let picker = self.picker();
        if cx.update_cause() == UpdateCause::Bootstrap {
            cx.open_layer(ID, picker.layer(cx, ITEMS));
        }
        let mut response = picker.update(cx, &mut self.state, ITEMS);
        if let Some(PickerAction::Chosen(key)) = response.take_action() {
            self.chosen = Some(key);
        }
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let picker = self.picker();
        ui.layer(ID, |ui, area| picker.draw(ui, area, &self.state, ITEMS));
    }
}

fn page(scope: Option<&'static str>, width: u16, screen: u16) -> Harness<Page> {
    Harness::new(
        Page {
            state: PickerState::default(),
            scope,
            width,
            chosen: None,
        },
        Theme::junie(),
        screen,
        20,
    )
}

fn row_text(h: &Harness<Page>, y: u16) -> String {
    let mut text = String::new();
    for x in 0..h.buffer().area.width {
        if let Some(cell) = h.buffer().cell((x, y)) {
            text.push_str(cell.symbol());
        }
    }
    text
}

#[test]
fn scope_label_shares_title_row_right_aligned_and_preserves_hidden_cursor() {
    let h = page(Some("Workspace: production"), 84, 120);
    let area = h.layer_area(ID).expect("Picker layer");
    assert_eq!(area, Rect::new(18, 2, 84, 6));
    let row = row_text(&h, area.y + 1);
    assert!(row.contains("Launch agent"), "{row:?}");
    assert!(row.contains("Workspace: production"), "{row:?}");
    let border = row.rfind('│').expect("right border");
    assert!(row[..border].ends_with("Workspace: production"), "{row:?}");
    assert_eq!(h.cursor(), None);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn absent_scope_keeps_title_row_and_all_cells_unchanged() {
    let without = page(None, 84, 120);
    let explicit_empty = page(Some(""), 84, 120);
    assert_eq!(without.buffer(), explicit_empty.buffer());
    assert_eq!(without.focus(), explicit_empty.focus());
    let area = without.layer_area(ID).expect("Picker layer");
    assert!(row_text(&without, area.y + 1).contains("Launch agent"));
    assert!(
        without.diagnostics().is_empty(),
        "{:?}",
        without.diagnostics()
    );
}

#[test]
fn scope_label_truncates_and_hides_when_title_leaves_less_than_six_cells() {
    let clipped = page(Some("workspace-production-long"), 32, 120);
    let area = clipped.layer_area(ID).expect("Picker layer");
    assert!(
        row_text(&clipped, area.y + 1).contains("…"),
        "{}",
        clipped.text()
    );

    let hidden = page(Some("Workspace"), 18, 120);
    let area = hidden.layer_area(ID).expect("Picker layer");
    let row = row_text(&hidden, area.y + 1);
    assert!(!row.contains("Workspace"), "{row:?}");
    assert!(row.contains("Launch agent"), "{row:?}");
    assert!(
        clipped.diagnostics().is_empty(),
        "{:?}",
        clipped.diagnostics()
    );
    assert!(
        hidden.diagnostics().is_empty(),
        "{:?}",
        hidden.diagnostics()
    );
}

#[test]
fn scope_caption_does_not_change_picker_keyboard_selection() {
    let mut h = page(Some("Workspace: production"), 84, 120);
    let _ = h.key(termrock_core::event::KeyCode::Enter);
    assert_eq!(h.app().chosen, Some(ItemKey::num(0)));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
