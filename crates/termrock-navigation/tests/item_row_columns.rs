use ratatui_core::layout::Rect;
use termrock_navigation::{
    FilterList, FilterListAction, FilterListState, FilterPolicy, Item, ItemRowLayout,
};
use termrock_runtime::{
    App, Cx, Id, ItemKey, KeyCode, Part, PartRef, Response, RowUi, Ui, UpdateCause,
};
use termrock_test_support::Harness;
use termrock_theme::{ColorLevel, CursorStyle, Modifier, Theme};
use unicode_segmentation::UnicodeSegmentation;
const ID: Id = Id::root("columns.list");
const ITEMS: &[Item<'static>] = &[
    Item::new(ItemKey::num(1), "e\u{301}界x")
        .glyph("T")
        .matched(&[0, 2])
        .detail("public.alpha")
        .tag("open")
        .group("Tables"),
    Item::new(ItemKey::num(2), "second")
        .glyph("V")
        .detail("public.beta")
        .group("Tables"),
    Item::new(ItemKey::num(3), "disabled")
        .glyph("Q")
        .group("History")
        .disabled(true),
    Item::new(ItemKey::num(4), "very long label")
        .glyph("S")
        .detail("analytics")
        .tag("tag")
        .group("Schemas"),
];
struct Page {
    state: FilterListState,
    items: Vec<Item<'static>>,
    layout: ItemRowLayout,
    width: u16,
    height: u16,
    chosen: Vec<ItemKey>,
    custom: bool,
    searchable: bool,
    policy: FilterPolicy,
    cursor_style: CursorStyle,
    metadata_gap: u16,
}
fn custom_row(_: &Item<'_>, row: &mut RowUi<'_>) {
    row.label("CUSTOM");
}
impl Page {
    fn list(&self) -> FilterList<'static, Item<'static>> {
        FilterList::new(ID)
            .searchable(self.searchable)
            .filter(self.policy)
            .item_layout(self.layout)
            .cursor_style(self.cursor_style)
            .metadata_gap(self.metadata_gap)
    }
}
impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if cx.update_cause() == UpdateCause::Bootstrap {
            cx.focus(ID);
        }
        let mut response = if self.custom {
            self.list()
                .row(custom_row)
                .update(cx, &mut self.state, &self.items)
        } else {
            self.list().update(cx, &mut self.state, &self.items)
        };
        if let Some(FilterListAction::Chose(key)) = response.take_action() {
            self.chosen.push(key);
        }
        response.erase()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        let area = Rect::new(1, 1, self.width, self.height);
        if self.custom {
            self.list()
                .row(custom_row)
                .draw(ui, area, &self.state, &self.items);
        } else {
            self.list().draw(ui, area, &self.state, &self.items);
        }
    }
}
fn page(
    layout: ItemRowLayout,
    width: u16,
    height: u16,
    custom: bool,
    level: ColorLevel,
) -> Harness<Page> {
    page_with_policy(
        layout,
        width,
        height,
        custom,
        level,
        CursorStyle::Bar,
        1,
        ITEMS.to_vec(),
    )
}
#[expect(
    clippy::too_many_arguments,
    reason = "the test fixture keeps each render setting explicit"
)]
fn page_with_policy(
    layout: ItemRowLayout,
    width: u16,
    height: u16,
    custom: bool,
    level: ColorLevel,
    cursor_style: CursorStyle,
    metadata_gap: u16,
    items: Vec<Item<'static>>,
) -> Harness<Page> {
    Harness::new(
        Page {
            state: FilterListState::default(),
            items,
            layout,
            width,
            height,
            chosen: vec![],
            custom,
            searchable: false,
            policy: FilterPolicy::Caller,
            cursor_style,
            metadata_gap,
        },
        Theme::junie().for_level(level),
        84,
        8,
    )
}
fn row(h: &Harness<Page>, key: u64) -> Rect {
    let area = h.area_of_part(ID, PartRef::item(Part::ROW, ItemKey::num(key)));
    assert!(area.is_some(), "visible semantic row {key}");
    area.unwrap_or_default()
}
fn at(h: &Harness<Page>, area: Rect, offset: u16, text: &str) {
    let mut x = area.x.saturating_add(offset);
    for grapheme in text.graphemes(true) {
        assert_eq!(h.cell(x, area.y).symbol(), grapheme);
        x = x.saturating_add(termrock_text::width(grapheme));
    }
}
#[test]
fn columns_align_complete_projection_and_emphasize_original_graphemes() {
    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        let h = page(ItemRowLayout::Columns, 64, 4, false, level);
        let a = row(&h, 1);
        at(&h, a, 1, "T");
        at(&h, a, 3, "e\u{301}界x");

        at(&h, a, 20, "public.alpha");
        at(&h, a, a.width.saturating_sub(14), "open");
        at(&h, a, a.width.saturating_sub(8), "Tables");
        let b = row(&h, 2);
        at(&h, b, 20, "public.beta");
        assert_eq!(
            h.cell(b.x.saturating_add(b.width.saturating_sub(8)), b.y)
                .symbol(),
            " "
        );
        let c = row(&h, 3);
        at(&h, c, c.width.saturating_sub(8), "History");

        let mut h = h;
        let _ = h.key(KeyCode::Down);
        let a = row(&h, 1);
        assert!(
            h.cell(a.x.saturating_add(3), a.y)
                .modifier
                .contains(Modifier::BOLD)
        );
        assert!(
            !h.cell(a.x.saturating_add(4), a.y)
                .modifier
                .contains(Modifier::BOLD)
        );
        assert!(
            h.cell(a.x.saturating_add(6), a.y)
                .modifier
                .contains(Modifier::BOLD)
        );
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
}

#[test]
fn explicit_default_policies_preserve_full_frame_and_cursor_at_each_color_level() {
    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        let default = page(ItemRowLayout::Columns, 64, 4, false, level);
        let explicit = page_with_policy(
            ItemRowLayout::Columns,
            64,
            4,
            false,
            level,
            CursorStyle::Bar,
            1,
            ITEMS.to_vec(),
        );
        assert_eq!(default.buffer(), explicit.buffer(), "{level:?}");
        assert_eq!(default.cursor(), explicit.cursor(), "{level:?}");
    }
}

#[test]
fn fill_cursor_uses_selection_colors_and_keeps_the_reserved_gutter_blank() {
    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        let h = page_with_policy(
            ItemRowLayout::Columns,
            64,
            4,
            false,
            level,
            CursorStyle::Fill,
            1,
            ITEMS.to_vec(),
        );
        let row = row(&h, 1);
        let gutter = h.cell(row.x, row.y);
        let label = h.cell(row.x.saturating_add(4), row.y);
        let theme = Theme::junie().for_level(level);
        assert_eq!(gutter.symbol(), " ", "{level:?}");
        assert_eq!(label.bg, theme.color.selection_bg, "{level:?}");
        assert_eq!(label.fg, theme.color.selection_fg, "{level:?}");
        assert!(h.app().state.cursor().is_some());
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());

        let bar = page(ItemRowLayout::Columns, 64, 4, false, level);
        assert_eq!(
            bar.app().state.cursor(),
            h.app().state.cursor(),
            "{level:?}"
        );
        assert_eq!(bar.cursor(), h.cursor(), "{level:?}");
        assert_ne!(bar.buffer(), h.buffer(), "{level:?}");
        assert_ne!(bar.cell(row.x, row.y).symbol(), " ", "{level:?}");
    }
}

#[test]
fn metadata_gap_only_changes_rows_with_metadata_columns() {
    let plain = vec![Item::new(ItemKey::num(10), "界 label")];
    let plain_default = page_with_policy(
        ItemRowLayout::Columns,
        64,
        2,
        false,
        ColorLevel::TrueColor,
        CursorStyle::Bar,
        1,
        plain.clone(),
    );
    let plain_wide_gap = page_with_policy(
        ItemRowLayout::Columns,
        64,
        2,
        false,
        ColorLevel::TrueColor,
        CursorStyle::Bar,
        2,
        plain,
    );
    assert_eq!(plain_default.buffer(), plain_wide_gap.buffer());

    let rich = vec![
        Item::new(ItemKey::num(11), "e\u{301}界 label")
            .detail("public.alpha")
            .tag("open")
            .group("Tables"),
    ];
    let narrow_gap = page_with_policy(
        ItemRowLayout::Columns,
        64,
        2,
        false,
        ColorLevel::TrueColor,
        CursorStyle::Bar,
        1,
        rich.clone(),
    );
    let baseline_gap = page_with_policy(
        ItemRowLayout::Columns,
        64,
        2,
        false,
        ColorLevel::TrueColor,
        CursorStyle::Bar,
        2,
        rich,
    );
    assert_ne!(narrow_gap.buffer(), baseline_gap.buffer());
    assert!(baseline_gap.text().contains("public.alpha"));
    assert!(baseline_gap.text().contains("open"));
    assert!(baseline_gap.text().contains("Tables"));
}
#[test]
fn scroll_top_repeats_group_without_shifting_columns_or_mutating_on_draw() {
    let mut h = page(ItemRowLayout::Columns, 64, 1, false, ColorLevel::TrueColor);
    let first = row(&h, 1);
    at(&h, first, 20, "public.alpha");
    let _ = h.key(KeyCode::Down);
    let second = row(&h, 2);
    at(&h, second, 20, "public.beta");
    at(&h, second, second.width.saturating_sub(8), "Tables");
    let before = h.text();
    let cursor = h.app().state.cursor();
    for _ in 0..3 {
        h.draw();
        assert_eq!(h.text(), before);
        assert_eq!(h.app().state.cursor(), cursor);
    }
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
#[test]
fn full_row_hit_is_authoritative_including_group_and_disabled_rows() {
    let mut h = page(ItemRowLayout::Columns, 64, 4, false, ColorLevel::TrueColor);
    let target = row(&h, 2);
    let _ = h.click(target.right().saturating_sub(1), target.y);
    assert_eq!(h.app().chosen, [ItemKey::num(2)]);
    let disabled = row(&h, 3);
    let _ = h.click(disabled.x.saturating_add(3), disabled.y);
    assert_eq!(h.app().chosen, [ItemKey::num(2)]);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
#[test]
fn custom_callback_replaces_columns_and_keeps_nonsearchable_policy() {
    let mut h = page(ItemRowLayout::Columns, 64, 4, true, ColorLevel::TrueColor);
    assert!(h.text().contains("CUSTOM"));
    assert!(!h.text().contains("Tables"));
    let _ = h.key(KeyCode::Char('z'));
    assert_eq!(h.app().state.query(), "");
    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().chosen, [ItemKey::num(2)]);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
#[test]
fn narrow_columns_never_escape_registered_rows() {
    for width in 1..40 {
        let h = page(
            ItemRowLayout::Columns,
            width,
            4,
            false,
            ColorLevel::TrueColor,
        );
        for y in 1..5 {
            assert_eq!(h.cell(0, y).symbol(), " ");
            assert_eq!(h.cell(width.saturating_add(1), y).symbol(), " ");
        }
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
}

#[global_allocator]
static ALLOC: termrock_test_support::perf::Counting = termrock_test_support::perf::Counting;
#[test]
fn warmed_columns_paint_allocates_nothing() {
    let mut h = page(ItemRowLayout::Columns, 64, 4, false, ColorLevel::TrueColor);
    h.draw();
    let before = termrock_test_support::perf::allocs();
    h.draw();
    assert_eq!(
        termrock_test_support::perf::allocs().checked_sub(before),
        Some(0)
    );
}

struct Modal {
    state: termrock_overlays::PickerState,
    custom: bool,
    chosen: Option<ItemKey>,
}
impl Modal {
    fn picker() -> termrock_overlays::Picker<'static, Item<'static>> {
        termrock_overlays::Picker::new(ID)
            .width(64)
            .searchable(false)
            .filter(FilterPolicy::Caller)
            .item_layout(ItemRowLayout::Columns)
    }
}
impl App for Modal {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if cx.update_cause() == UpdateCause::Bootstrap {
            cx.open_layer(ID, Self::picker().layer(cx, ITEMS));
        }
        let mut response = if self.custom {
            Self::picker()
                .row(custom_row)
                .update(cx, &mut self.state, ITEMS)
        } else {
            Self::picker().update(cx, &mut self.state, ITEMS)
        };
        if let Some(termrock_overlays::PickerAction::Chosen(key)) = response.take_action() {
            self.chosen = Some(key);
        }
        response.erase()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        ui.layer(ID, |ui, area| {
            if self.custom {
                Self::picker()
                    .row(custom_row)
                    .draw(ui, area, &self.state, ITEMS)
            } else {
                Self::picker().draw(ui, area, &self.state, ITEMS)
            }
        });
    }
}
#[test]
fn picker_forwards_columns_and_custom_row_retains_width_search_and_caller_policy() {
    for custom in [false, true] {
        let mut state = termrock_overlays::PickerState::default();
        state.set_query("stale");
        let mut h = Harness::new(
            Modal {
                state,
                custom,
                chosen: None,
            },
            Theme::junie(),
            100,
            20,
        );
        assert_eq!(h.layer_area(ID).map(|area| area.width), Some(64));
        assert_eq!(h.app().state.query(), "");
        assert_eq!(h.text().contains("CUSTOM"), custom);
        assert_eq!(h.text().contains("Tables"), !custom);
        let _ = h.key(KeyCode::Char('z'));
        assert_eq!(h.app().state.query(), "");
        let _ = h.key(KeyCode::Down);
        let _ = h.key(KeyCode::Enter);
        assert_eq!(h.app().chosen, Some(ItemKey::num(2)));
        let _ = h.key(KeyCode::Esc);
        assert!(h.layer_area(ID).is_none());
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
}

#[test]
fn measurement_uses_filtered_projection_not_hidden_source_items() {
    let mut state = FilterListState::default();
    state.set_query("second");
    let h = Harness::new(
        Page {
            state,
            items: ITEMS.to_vec(),
            layout: ItemRowLayout::Columns,
            width: 64,
            height: 4,
            chosen: vec![],
            custom: false,
            searchable: true,
            policy: FilterPolicy::Label,
            cursor_style: CursorStyle::Bar,
            metadata_gap: 1,
        },
        Theme::junie(),
        84,
        8,
    );
    let area = row(&h, 2);
    at(&h, area, 3, "second");
    at(&h, area, 11, "public.beta");
    at(&h, area, area.width.saturating_sub(7), "Tables");
    assert!(
        h.area_of_part(ID, PartRef::item(Part::ROW, ItemKey::num(4)))
            .is_none()
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn independent_ancestor_top_clip_repeats_first_visible_group() {
    struct Clipped(Page);
    impl App for Clipped {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            self.0.update(cx)
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            ui.with_area(Rect::new(1, 2, 64, 2), |ui| self.0.draw(ui));
        }
    }
    let h = Harness::new(
        Clipped(Page {
            state: FilterListState::default(),
            items: ITEMS.to_vec(),
            layout: ItemRowLayout::Columns,
            width: 64,
            height: 4,
            chosen: vec![],
            custom: false,
            searchable: false,
            policy: FilterPolicy::Caller,
            cursor_style: CursorStyle::Bar,
            metadata_gap: 1,
        }),
        Theme::junie(),
        84,
        8,
    );
    assert!(
        h.row(2).contains("Tables"),
        "first actually visible group missing: {}",
        h.row(2)
    );
}

#[test]
fn owner_part_styles_win_over_painter_defaults_in_columns_mode() {
    use ratatui_core::style::Color;
    use termrock_theme::{Family, Overlay, OverlayRule, Role, StylePatch, Variant};
    const FG: Color = Color::Rgb(17, 83, 149);
    const RULES: &[OverlayRule] = &[
        (
            Family::PICKER,
            Variant::DEFAULT,
            Part::ICON,
            termrock_runtime::StateFlags::empty(),
            StylePatch::new().set_fg(Role::Custom(FG)),
        ),
        (
            Family::PICKER,
            Variant::DEFAULT,
            Part::META,
            termrock_runtime::StateFlags::empty(),
            StylePatch::new().set_fg(Role::Custom(FG)),
        ),
        (
            Family::PICKER,
            Variant::DEFAULT,
            Part::LABEL,
            termrock_runtime::StateFlags::empty(),
            StylePatch::new().add(Modifier::BOLD),
        ),
    ];
    struct Styled {
        page: Page,
        overlay: bool,
    }
    impl App for Styled {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            self.page.update(cx)
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            let draw = |ui: &mut Ui<'_>| {
                self.page.list().draw(
                    ui,
                    Rect::new(1, 1, 64, 4),
                    &self.page.state,
                    &self.page.items,
                );
            };
            if self.overlay {
                ui.with_overlay(&Overlay::new(RULES), draw);
            } else {
                draw(ui);
            }
        }
    }
    let mut failures = Vec::new();
    for (tier, overlay) in [("theme", false), ("overlay", true)] {
        let mut theme = Theme::junie();
        if !overlay {
            theme = theme.define_family(Family::PICKER, |f| {
                f.part(Part::ICON)
                    .base(StylePatch::new().set_fg(Role::Custom(FG)));
                f.part(Part::META)
                    .base(StylePatch::new().set_fg(Role::Custom(FG)));
                f.part(Part::LABEL)
                    .base(StylePatch::new().add(Modifier::BOLD));
            });
        }
        let mut h = Harness::new(
            Styled {
                overlay,
                page: Page {
                    state: FilterListState::default(),
                    items: ITEMS.to_vec(),
                    layout: ItemRowLayout::Columns,
                    width: 64,
                    height: 4,
                    chosen: vec![],
                    custom: false,
                    searchable: false,
                    policy: FilterPolicy::Caller,
                    cursor_style: CursorStyle::Bar,
                    metadata_gap: 1,
                },
            },
            theme,
            84,
            8,
        );
        let _ = h.key(KeyCode::Down);
        for (name, x) in [("ICON", 2), ("META", 21)] {
            if h.cell(x, 1).fg != FG {
                failures.push(format!("{tier} {name}: {:?}", h.cell(x, 1).fg));
            }
        }
        if !h.cell(5, 1).modifier.contains(Modifier::BOLD) {
            failures.push(format!("{tier} LABEL BOLD erased"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("; "));
}
