use ratatui_core::layout::{Position, Rect};
use termrock_core::event::MouseKind;
use termrock_core::id::{Id, ItemKey, Part, PartRef};
use termrock_core::response::Response;
use termrock_navigation::filter_list::FilterPolicy;
use termrock_overlays::picker::{Item, Picker, PickerAction, PickerLayout, PickerState};
use termrock_runtime::layer::ScreenAlign;
use termrock_runtime::{App, Cx, Ui};
use termrock_test_support::Harness;
use termrock_theme::{ColorLevel, Theme};

const ID: Id = Id::root("picker.card-layout");
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
    items: Vec<Item<'static>>,
    searchable: bool,
    hints: &'static str,
    layout: PickerLayout,
    chosen: Option<ItemKey>,
}

impl Page {
    fn picker(&self) -> Picker<'static, Item<'static>> {
        Picker::new(ID)
            .title("Choose a workspace")
            .width(72)
            .hints(self.hints)
            .searchable(self.searchable)
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
            cx.open_layer(ID, picker.layer(cx, &self.items));
        }
        let mut response = picker.update(cx, &mut self.state, &self.items);
        if let Some(PickerAction::Chosen(key)) = response.take_action() {
            self.chosen = Some(key);
        }
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let picker = self.picker();
        ui.layer(ID, |ui, area| {
            picker.draw(ui, area, &self.state, &self.items)
        });
    }
}

fn page(items: Vec<Item<'static>>, searchable: bool, width: u16, height: u16) -> Harness<Page> {
    page_with_layout(
        items,
        searchable,
        "Enter choose",
        PickerLayout::card(),
        width,
        height,
    )
}

fn page_with_layout(
    items: Vec<Item<'static>>,
    searchable: bool,
    hints: &'static str,
    layout: PickerLayout,
    width: u16,
    height: u16,
) -> Harness<Page> {
    Harness::new(
        Page {
            state: PickerState::default(),
            opened: false,
            items,
            searchable,
            hints,
            layout,
            chosen: None,
        },
        Theme::junie(),
        width,
        height,
    )
}

fn count_items(count: usize) -> Vec<Item<'static>> {
    (0..count)
        .map(|index| {
            ITEMS
                .get(index)
                .copied()
                .unwrap_or_else(|| Item::new(ItemKey::num(index as u64), "Live row"))
        })
        .collect()
}

#[test]
fn card_size_uses_live_rows_and_viewport_margins() {
    for (count, rows) in [(0, 1), (1, 1), (5, 5), (12, 12), (13, 12), (100, 12)] {
        let h = page(count_items(count), true, 120, 40);
        let area = h.layer_area(ID).expect("picker layer is open");
        assert_eq!((area.width, area.height), (72, rows + 7), "count={count}");
        assert_eq!(area.x, (120 - area.width) / 2, "count={count}");
        assert_eq!(area.y, (40 - area.height) / 2, "count={count}");
    }

    let h = page(count_items(5), false, 120, 40);
    let area = h.layer_area(ID).expect("picker layer is open");
    assert_eq!((area.width, area.height), (72, 10));
}

#[test]
fn card_draw_uses_policy_for_query_result_hit_and_cursor_bounds() {
    let mut h = page(count_items(5), true, 120, 40);
    let area = h.layer_area(ID).expect("picker layer is open");
    assert_eq!(area, Rect::new(24, 14, 72, 12));

    let registry = h.runtime().registry();
    let query = registry.area_of(ID).expect("query owns a live hit area");
    assert_eq!(query, Rect::new(26, 16, 68, 1));
    assert_eq!(
        registry.area_of_part(ID, PartRef::item(Part::ROW, ItemKey::num(0))),
        Some(Rect::new(26, 18, 68, 1))
    );

    let _ = h.type_str("QQQ日本");
    assert_eq!(h.app().state.query(), "QQQ日本");
    assert_eq!(h.app().state.list().matched_len(), 5);
    let cursor = h.cursor().expect("query owns visible terminal cursor");
    assert_eq!(cursor.y, 16);

    let _ = h.mouse(MouseKind::Move, 26, 18);
    let registry = h.runtime().registry();
    let hit = registry
        .hit(Position::new(26, 18))
        .expect("result viewport is interactive");
    assert_ne!(hit.part.part, Part::QUERY);
}

#[test]
fn zero_footer_disables_hints_without_overlapping_live_results() {
    let items = count_items(100);
    let without_footer = page_with_layout(
        items.clone(),
        true,
        "VISIBLE_FOOTER",
        PickerLayout::card().footer_rows(0),
        120,
        40,
    );
    let without_area = without_footer
        .layer_area(ID)
        .expect("zero-footer picker is open");
    assert_eq!(without_area.height, 18);
    assert_eq!(without_footer.count("VISIBLE_FOOTER"), 0);
    assert_eq!(
        without_footer.app().state.list().scroll().viewport_len(),
        13
    );
    assert!(
        without_footer
            .runtime()
            .registry()
            .area_of_part(ID, PartRef::item(Part::ROW, ItemKey::num(0)))
            .is_some()
    );

    let with_footer = page_with_layout(
        items,
        true,
        "VISIBLE_FOOTER",
        PickerLayout::card().footer_rows(1),
        120,
        40,
    );
    let with_area = with_footer
        .layer_area(ID)
        .expect("one-row-footer picker is open");
    assert_eq!(with_area.height, 19);
    assert_eq!(with_footer.count("VISIBLE_FOOTER"), 1);
    let footer_y = with_area.bottom() - 2;
    let row_area = with_footer
        .runtime()
        .registry()
        .area_of_part(ID, PartRef::item(Part::ROW, ItemKey::num(0)))
        .expect("live results retain a registered row");
    assert!(row_area.bottom() <= footer_y);
}

#[test]
fn zero_footer_hides_hints_with_one_result_at_every_color_level() {
    assert_footer_visibility(count_items(1), "one item");
}

#[test]
fn zero_footer_hides_hints_with_empty_results_at_every_color_level() {
    assert_footer_visibility(Vec::new(), "empty");
}

fn assert_footer_visibility(items: Vec<Item<'static>>, label: &str) {
    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        let without_footer = page_with_layout(
            items.clone(),
            true,
            "VISIBLE_FOOTER",
            PickerLayout::card().footer_rows(0),
            120,
            40,
        )
        .with_color(level);
        assert_eq!(
            without_footer.count("VISIBLE_FOOTER"),
            0,
            "zero-row footer must hide hints for {label} at {level:?}"
        );

        let with_footer = page_with_layout(
            items.clone(),
            true,
            "VISIBLE_FOOTER",
            PickerLayout::card().footer_rows(1),
            120,
            40,
        )
        .with_color(level);
        assert_eq!(
            with_footer.count("VISIBLE_FOOTER"),
            1,
            "one-row footer must show hints for {label} at {level:?}"
        );
    }
}

#[test]
fn page_down_uses_the_card_result_viewport_including_its_spare_row() {
    let mut h = page(count_items(100), true, 120, 40);
    assert_eq!(h.app().state.list().scroll().viewport_len(), 13);
    let _ = h.key(termrock_core::event::KeyCode::PageDown);
    assert_eq!(h.app().state.cursor(), Some(ItemKey::num(13)));
}

#[test]
fn card_geometry_stays_inside_adjacent_and_degenerate_viewports() {
    for (width, expected_width) in [(75, 71), (76, 72), (77, 72), (79, 72), (80, 72), (81, 72)] {
        let h = page(count_items(5), true, width, 40);
        let area = h.layer_area(ID).expect("picker layer is open");
        assert_eq!(area.width, expected_width, "width={width}");
        assert!(area.right() <= width.saturating_sub(2));
    }
    for height in [19, 20, 21, 23, 24, 25] {
        let h = page(count_items(5), true, 120, height);
        let area = h.layer_area(ID).expect("picker layer is open");
        assert!(area.height <= height.saturating_sub(2).max(1));
        assert!(area.bottom() <= height.saturating_sub(1));
    }
    for width in 0..=4 {
        for height in 0..=2 {
            let h = page(count_items(1), true, width, height);
            assert!(
                h.diagnostics().is_empty(),
                "{width}x{height}: {:?}",
                h.diagnostics()
            );
            if width == 0 || height == 0 {
                assert!(h.layer_area(ID).is_none_or(Rect::is_empty));
            }
        }
    }
}
