use ratatui_core::layout::Rect;
use ratatui_core::style::Color;
use termrock_core::id::{Id, ItemKey, Part};
use termrock_core::response::Response;
use termrock_overlays::picker::{Item, Picker, PickerAction, PickerState};
use termrock_runtime::layer::{LayerSize, ScreenAlign};
use termrock_runtime::{App, Cx, Ui, UpdateCause};
use termrock_test_support::Harness;
use termrock_theme::{Family, Role, StateFlags, StylePatch, Surface, Theme, Variant};

const ID: Id = Id::root("picker.scope");
const ITEMS: &[Item<'static>] = &[Item::new(ItemKey::num(0), "First")];
const META_PATCH: StylePatch = StylePatch::new().set_fg(Role::Custom(Color::Rgb(1, 2, 3)));
const META_PARTS: &[(Part, StylePatch)] = &[(Part::META, META_PATCH)];

struct Page {
    state: PickerState,
    scope: Option<&'static str>,
    width: u16,
    height: Option<u16>,
    title: &'static str,
    meta_patch: bool,
    draw_area: Option<Rect>,
    chosen: Option<ItemKey>,
}

impl Page {
    fn picker(&self) -> Picker<'static, Item<'static>> {
        let mut picker = Picker::new(ID)
            .title(self.title)
            .width(self.width)
            .align(ScreenAlign::Center)
            .searchable(false);
        if let Some(height) = self.height {
            picker = picker.size(LayerSize::Fixed(self.width, height));
        }
        if self.meta_patch {
            picker = picker.patch_part(META_PARTS);
        }
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
        if let Some(area) = self.draw_area {
            picker.draw(ui, area, &self.state, ITEMS);
        } else {
            ui.layer(ID, |ui, area| picker.draw(ui, area, &self.state, ITEMS));
        }
    }
}

fn page(scope: Option<&'static str>, width: u16, screen: u16) -> Harness<Page> {
    Harness::new(
        Page {
            state: PickerState::default(),
            scope,
            width,
            height: None,
            title: "Launch agent",
            meta_patch: false,
            draw_area: None,
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

fn cells_text(h: &Harness<Page>, y: u16, x: u16, width: u16) -> String {
    let mut text = String::new();
    for column in x..x.saturating_add(width) {
        if let Some(cell) = h.buffer().cell((column, y)) {
            text.push_str(cell.symbol());
        }
    }
    text
}

fn default_meta_style() -> ratatui_core::style::Style {
    *Theme::junie()
        .resolve(
            Family::PICKER,
            Variant::DEFAULT,
            Part::META,
            StateFlags::empty(),
            Surface::Overlay,
        )
        .style
}

fn explicit_page(
    scope: Option<&'static str>,
    width: u16,
    height: u16,
    meta_patch: bool,
) -> Harness<Page> {
    Harness::new(
        Page {
            state: PickerState::default(),
            scope,
            width,
            height: Some(height),
            title: "Launch · choose Agent",
            meta_patch,
            draw_area: Some(Rect::new((120 - width) / 2, 15, width, height)),
            chosen: None,
        },
        Theme::junie(),
        120,
        40,
    )
}

#[test]
fn scope_label_shares_title_row_right_aligned_and_preserves_hidden_cursor() {
    let h = explicit_page(
        Some("payments-platform › chainargos/the-architect"),
        84,
        9,
        false,
    );
    let area = Rect::new(18, 15, 84, 9);
    let row = row_text(&h, area.y + 1);
    assert!(row.contains("Launch · choose Agent"), "{row:?}");
    assert!(
        row.contains("payments-platform › chainargos/the-architect"),
        "{row:?}"
    );
    assert_eq!(cells_text(&h, 16, 20, 21), "Launch · choose Agent");
    assert_eq!(
        cells_text(&h, 16, 56, 44),
        "payments-platform › chainargos/the-architect"
    );
    let without_caption = explicit_page(None, 84, 9, false);
    let mut expected = without_caption.buffer().clone();
    expected.set_stringn(
        56,
        16,
        "payments-platform › chainargos/the-architect",
        44,
        default_meta_style(),
    );
    assert_eq!(h.buffer(), &expected);
    assert_eq!(h.cursor(), without_caption.cursor());
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

#[test]
fn explicit_size_caption_uses_unicode_cell_width_truncation_and_meta_style() {
    let h = explicit_page(Some("abc日本語"), 35, 9, true);
    assert_eq!(cells_text(&h, 16, 69, 6), "abc日 …");
    let without_caption = explicit_page(None, 35, 9, true);
    let mut expected = without_caption.buffer().clone();
    expected.set_stringn(69, 16, "abc日…", 6, {
        let mut style = default_meta_style();
        style.fg = Some(Color::Rgb(1, 2, 3));
        style
    });
    assert_eq!(h.buffer(), &expected);
    assert_eq!(h.cursor(), without_caption.cursor());
    assert_eq!(h.cursor(), None);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn explicit_size_caption_shows_at_six_cells_and_hides_at_five() {
    let visible = explicit_page(Some("abcdefghi"), 34, 9, false);
    assert_eq!(cells_text(&visible, 16, 69, 6), "abcde…");
    let without_caption = explicit_page(None, 34, 9, false);
    let mut expected = without_caption.buffer().clone();
    expected.set_stringn(69, 16, "abcde…", 6, default_meta_style());
    assert_eq!(visible.buffer(), &expected);
    assert_eq!(visible.cursor(), without_caption.cursor());

    let hidden = explicit_page(Some("abcdefghi"), 33, 9, false);
    let without_caption = explicit_page(None, 33, 9, false);
    assert_eq!(hidden.buffer(), without_caption.buffer());
    assert_eq!(hidden.cursor(), without_caption.cursor());
    assert!(
        visible.diagnostics().is_empty(),
        "{:?}",
        visible.diagnostics()
    );
    assert!(
        hidden.diagnostics().is_empty(),
        "{:?}",
        hidden.diagnostics()
    );
}
