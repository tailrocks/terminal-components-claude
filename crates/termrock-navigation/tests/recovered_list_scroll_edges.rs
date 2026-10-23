use ratatui_core::buffer::Cell;
use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Modifier, Style};
use termrock_collections::SelectMode;
use termrock_core::{Axis, Id, ItemKey, KeyCode, MouseKind, Response, StateFlags};
use termrock_navigation::{List, ListState};
use termrock_runtime::{App, Cx, Family, Part, PartRef, Ui};
use termrock_test_support::Harness;
use termrock_theme::{GlyphRole, Role, StylePatch, Surface, Theme, Variant};

const LIST: Id = Id::root("recovered.list.scroll-edges");
const SCREEN_WIDTH: u16 = 30;
const SCREEN_HEIGHT: u16 = 8;
const LIST_AREA: Rect = Rect {
    x: 2,
    y: 1,
    width: 22,
    height: 6,
};
const ROW_COUNT: usize = 40;
const ROW_FG: Color = Color::Rgb(220, 175, 90);
const ROW_BG: Color = Color::Rgb(18, 26, 38);
const OUTSIDE_FG: Color = Color::Rgb(175, 190, 205);
const OUTSIDE_BG: Color = Color::Rgb(24, 31, 42);

struct Page {
    items: Vec<String>,
    state: ListState,
}

impl Page {
    fn new() -> Self {
        Self {
            items: (0..ROW_COUNT).map(|i| format!("Row {i:02}")).collect(),
            state: ListState::default(),
        }
    }
}

impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        List::new(LIST)
            .select_mode(SelectMode::Multi)
            .update(cx, &mut self.state, &self.items)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let outside = Style::new().fg(OUTSIDE_FG).bg(OUTSIDE_BG);
        ui.fill(ui.full(), outside);
        let _ = ui.paint_str(Rect::new(0, 0, SCREEN_WIDTH, 1), "LIST HEADER", outside);
        let _ = ui.paint_str(
            Rect::new(0, SCREEN_HEIGHT - 1, SCREEN_WIDTH, 1),
            "LIST FOOTER",
            outside,
        );
        List::new(LIST).select_mode(SelectMode::Multi).draw(
            ui,
            LIST_AREA,
            &self.state,
            &self.items,
        );
    }
}

fn rgb_theme() -> Theme {
    Theme::junie().override_family(Family::LIST, |recipe| {
        recipe.part(Part::CONTAINER).base(
            StylePatch::new()
                .set_fg(Role::Custom(ROW_FG))
                .set_bg(Role::Custom(ROW_BG)),
        );
        recipe
            .part(Part::LABEL)
            .base(StylePatch::new().set_fg(Role::Custom(ROW_FG)));
    })
}

fn harness() -> (Harness<Page>, Theme) {
    let theme = rgb_theme();
    let h = Harness::new(Page::new(), theme.clone(), SCREEN_WIDTH, SCREEN_HEIGHT);
    (h, theme)
}

fn row(h: &Harness<Page>, index: usize) -> Rect {
    h.area_of_part(LIST, PartRef::item(Part::ROW, ItemKey::index(index)))
        .expect("visible list item has a row area")
}

fn label_x(row: Rect) -> u16 {
    row.x.saturating_add(4)
}

fn outside_cells(h: &Harness<Page>) -> Vec<((u16, u16), Cell)> {
    let mut cells = Vec::new();
    for y in 0..SCREEN_HEIGHT {
        for x in 0..SCREEN_WIDTH {
            if x < LIST_AREA.x
                || x >= LIST_AREA.right()
                || y < LIST_AREA.y
                || y >= LIST_AREA.bottom()
            {
                cells.push(((x, y), h.cell(x, y).clone()));
            }
        }
    }
    cells
}

fn assert_scrollbar(h: &Harness<Page>, theme: &Theme) {
    let track = h
        .area_of_part(LIST, PartRef::of(Part::TRACK))
        .expect("overflowing list paints its scrollbar track");
    let thumb = h
        .area_of_part(LIST, PartRef::of(Part::THUMB))
        .expect("overflowing list paints its scrollbar thumb");
    assert_eq!(track.x, LIST_AREA.right().saturating_sub(1));
    assert_eq!(track.y, LIST_AREA.y);
    assert_eq!(track.width, 1);
    assert_eq!(track.height, LIST_AREA.height);
    assert_eq!(thumb.x, track.x);
    assert!(thumb.height > 0);

    let track_glyph = theme.design.glyphs.get(GlyphRole::ScrollTrack);
    let thumb_glyph = theme.design.glyphs.get(GlyphRole::ScrollThumb);
    let track_style = theme
        .resolve(
            Family::SCROLLBAR,
            Variant::DEFAULT,
            Part::TRACK,
            StateFlags::empty(),
            Surface::Canvas,
        )
        .style;
    let thumb_style = theme
        .resolve(
            Family::SCROLLBAR,
            Variant::DEFAULT,
            Part::THUMB,
            StateFlags::empty(),
            Surface::Canvas,
        )
        .style;
    for y in track.y..track.bottom() {
        let cell = h.cell(track.x, y);
        let (glyph, style) = if y >= thumb.y && y < thumb.bottom() {
            (thumb_glyph, thumb_style)
        } else {
            (track_glyph, track_style)
        };
        assert_eq!(
            cell.symbol(),
            glyph,
            "scrollbar glyph at ({}, {y})",
            track.x
        );
        assert_eq!(cell.fg, style.fg.unwrap_or(Color::Reset));
        assert_eq!(
            cell.bg, ROW_BG,
            "scrollbar glyph preserves the list's painted underlay at ({}, {y})",
            track.x
        );
        assert_eq!(cell.modifier, style.add_modifier);
    }
}

#[test]
fn list_draw_fades_only_hidden_edges_and_preserves_row_surfaces_and_chrome() {
    let (mut h, theme) = harness();
    let outside_before = outside_cells(&h);
    assert!(h.tab_to(LIST), "the real list receives keyboard focus");
    let _ = h.key(KeyCode::Home);

    let initial_scroll = h.app().state.scroll();
    assert_eq!(h.app().state.cursor(), Some(ItemKey::index(0)));
    assert_eq!(initial_scroll.offset(), 0);
    assert_eq!(initial_scroll.viewport_len(), usize::from(LIST_AREA.height));
    assert!(initial_scroll.content_len() > initial_scroll.viewport_len());

    let first = row(&h, 0);
    let second = row(&h, 1);
    let fourth = row(&h, 4);
    let fifth = row(&h, 5);
    assert_eq!(first.y, LIST_AREA.y);
    assert_eq!(fifth.bottom(), LIST_AREA.bottom());
    let text_x = label_x(first);
    assert_eq!(h.cell(text_x, first.y).fg, ROW_FG);
    assert_eq!(h.cell(text_x, first.y).bg, ROW_BG);
    assert_eq!(h.cell(text_x, first.y).fg, h.cell(text_x, second.y).fg);

    let mut missing_fades = Vec::new();
    if h.cell(text_x, fifth.y).fg == h.cell(text_x, fourth.y).fg {
        missing_fades.push(format!(
            "top position: hidden rows below, but bottom fg {:?} equals interior fg {:?}",
            h.cell(text_x, fifth.y).fg,
            h.cell(text_x, fourth.y).fg,
        ));
    }
    if h.cell(text_x, first.y).fg != h.cell(text_x, second.y).fg {
        missing_fades.push("top position: the first row faded despite no hidden rows above".into());
    }
    assert_scrollbar(&h, &theme);

    for _ in 0..5 {
        let _ = h.key(KeyCode::Down);
    }
    assert_eq!(h.app().state.cursor(), Some(ItemKey::index(5)));
    assert_eq!(h.app().state.scroll().offset(), 0);
    let cursor_edge_fg = h.cell(text_x, fifth.y).fg;
    assert_eq!(cursor_edge_fg, ROW_FG, "the visible cursor row stays whole");
    assert_eq!(h.cell(fifth.x, fifth.y).symbol(), "▎");
    let _ = h.key(KeyCode::Up);
    assert_eq!(h.app().state.cursor(), Some(ItemKey::index(4)));
    let ordinary_edge_fg = h.cell(text_x, fifth.y).fg;
    if ordinary_edge_fg == cursor_edge_fg {
        missing_fades
            .push("top position after cursor leaves: ordinary bottom row did not fade".into());
    }

    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::Char(' '));
    assert!(h.app().state.checked().contains(ItemKey::index(5)));
    let _ = h.key(KeyCode::Up);
    let checked_edge = h.cell(text_x, fifth.y);
    assert_eq!(
        checked_edge.bg, ROW_BG,
        "checked row keeps its source surface"
    );
    assert_eq!(
        h.cell(fifth.x.saturating_add(1), fifth.y).symbol(),
        theme.design.glyphs.get(GlyphRole::Checked),
        "fading ink must not remove the checked marker",
    );
    if checked_edge.fg == ROW_FG {
        missing_fades.push("checked row at hidden bottom edge did not fade its ink".into());
    }

    // The Junie theme scales one physical wheel step to three list rows.
    let _ = h.wheel(Axis::V, 1, text_x, LIST_AREA.y.saturating_add(2));
    assert_eq!(h.app().state.cursor(), Some(ItemKey::index(4)));
    assert_eq!(h.app().state.scroll().offset(), 3);
    let _ = h.key(KeyCode::Down);
    assert_eq!(h.app().state.cursor(), Some(ItemKey::index(5)));
    let hover_x = label_x(row(&h, 3));
    let hover_y = row(&h, 3).y;
    let _ = h.mouse(MouseKind::Move, hover_x, hover_y);
    assert_eq!(h.hover(), Some(LIST));
    let hovered_top = h.cell(hover_x, hover_y);
    let expected_hover_bg = theme
        .resolve(
            Family::LIST,
            Variant::DEFAULT,
            Part::CONTAINER,
            StateFlags::HOVERED,
            Surface::Canvas,
        )
        .style
        .bg
        .unwrap_or(Color::Reset);
    assert_eq!(hovered_top.bg, expected_hover_bg);
    assert_ne!(
        hovered_top.fg, ROW_FG,
        "eligible hover ink still receives the final edge fade"
    );
    assert_eq!(row(&h, 3).y, LIST_AREA.y);
    assert_eq!(row(&h, 8).bottom(), LIST_AREA.bottom());
    assert_eq!(h.app().state.scroll().offset(), 3);
    let middle = h.cell(label_x(row(&h, 6)), row(&h, 6).y).fg;
    let next_middle = h.cell(label_x(row(&h, 7)), row(&h, 7).y).fg;
    assert_eq!(middle, ROW_FG);
    assert_eq!(
        middle, next_middle,
        "ordinary middle rows remain unmodified"
    );
    let middle_bottom = h.cell(label_x(row(&h, 7)), row(&h, 7).y).fg;
    let hidden_bottom = h.cell(label_x(row(&h, 8)), row(&h, 8).y).fg;
    if hidden_bottom == middle_bottom {
        missing_fades.push("middle position: bottom edge did not fade toward hidden rows".into());
    }
    assert_scrollbar(&h, &theme);

    for _ in 0..4 {
        let _ = h.key(KeyCode::Down);
    }
    assert_eq!(h.app().state.cursor(), Some(ItemKey::index(9)));
    assert_eq!(h.app().state.scroll().offset(), 4);
    let _ = h.key(KeyCode::Char(' '));
    assert!(h.app().state.checked().contains(ItemKey::index(9)));
    let _ = h.key(KeyCode::Up);
    assert_eq!(h.app().state.cursor(), Some(ItemKey::index(8)));
    let checked_bottom = h.cell(label_x(row(&h, 9)), row(&h, 9).y);
    assert_eq!(checked_bottom.bg, ROW_BG);
    assert_eq!(
        h.cell(row(&h, 9).x.saturating_add(1), row(&h, 9).y)
            .symbol(),
        theme.design.glyphs.get(GlyphRole::Checked),
    );
    if checked_bottom.fg == ROW_FG {
        missing_fades.push("checked middle-position bottom row did not fade its ink".into());
    }

    let _ = h.mouse(MouseKind::Move, 0, 0);
    assert_ne!(h.hover(), Some(LIST));
    let _ = h.key(KeyCode::End);
    let _ = h.tick();
    assert_eq!(h.app().state.cursor(), Some(ItemKey::index(ROW_COUNT - 1)));
    assert_eq!(
        h.app().state.scroll().offset(),
        ROW_COUNT - usize::from(LIST_AREA.height)
    );
    assert_eq!(row(&h, ROW_COUNT - 1).bottom(), LIST_AREA.bottom());
    let end_top = h.cell(text_x, LIST_AREA.y).fg;
    let end_next = h.cell(text_x, LIST_AREA.y.saturating_add(1)).fg;
    let end_previous = h.cell(text_x, LIST_AREA.bottom().saturating_sub(2)).fg;
    let end_bottom = h.cell(text_x, LIST_AREA.bottom().saturating_sub(1)).fg;
    if end_top == end_next {
        missing_fades.push("end position: top edge did not fade toward hidden rows".into());
    }
    assert_eq!(
        end_bottom, end_previous,
        "end position: bottom edge must not fade when no rows are below",
    );
    assert_scrollbar(&h, &theme);

    assert_eq!(
        outside_cells(&h),
        outside_before,
        "list painting stays within its area"
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    assert!(
        missing_fades.is_empty(),
        "default List is missing fade behavior:\n{}",
        missing_fades.join("\n")
    );
}

const CONFIG_LIST: Id = Id::root("recovered.list.fade-options");
const CONFIG_FG: Color = Color::Rgb(220, 175, 90);
const CONFIG_BG: Color = Color::Rgb(18, 26, 38);
const OTHER_BG: Color = Color::Rgb(9, 17, 25);

type ConfigKey = fn(&String) -> ItemKey;
type ConfigRow = for<'row> fn(&String, &mut termrock_runtime::RowUi<'row>);
type ConfiguredList<'a> = List<'a, String, ConfigKey, ConfigRow>;

#[expect(
    clippy::ptr_arg,
    reason = "List's typed KeyFn receives &T; this harness uses T=String"
)]
fn config_key(item: &String) -> ItemKey {
    ItemKey::index(config_index(item))
}

fn config_index(item: &str) -> usize {
    item.strip_prefix("Row ")
        .map(|value| {
            value
                .chars()
                .take_while(char::is_ascii_digit)
                .collect::<String>()
        })
        .and_then(|digits| digits.parse::<usize>().ok())
        .unwrap_or(usize::MAX)
}

#[expect(
    clippy::ptr_arg,
    reason = "List's typed RowFn receives &T; this harness uses T=String"
)]
fn config_row(item: &String, row: &mut termrock_runtime::RowUi<'_>) {
    let (label, style) = item.split_once('|').unwrap_or((item, ""));
    let patch = match style {
        "fg" => StylePatch::new().set_fg(Role::Custom(Color::Rgb(240, 130, 60))),
        "bg" => StylePatch::new().set_bg(Role::Custom(OTHER_BG)),
        "clear-bg" => StylePatch::new().clear_bg(),
        "reset-bg" => StylePatch::new().set_bg(Role::Custom(Color::Reset)),
        "reversed" => StylePatch::new().add(Modifier::REVERSED),
        "underlined" => StylePatch::new().add(Modifier::UNDERLINED),
        _ => {
            row.label(label);
            return;
        }
    };
    row.label_patched(label, &patch);
}

struct ConfigPage {
    items: Vec<String>,
    state: ListState,
    screen_width: u16,
    screen_height: u16,
    area: Rect,
    parent_clip: Option<Rect>,
    fade: Option<bool>,
    builder_order: u8,
    row_height: u16,
    row_gap: u16,
    whole_rows: bool,
    custom_renderer: bool,
    fade_after_renderer: bool,
    global_patch: Option<StylePatch>,
    part_patches: Vec<(Part, StylePatch)>,
    renderer_calls: core::cell::Cell<usize>,
    rendered_rows: core::cell::RefCell<Vec<(Rect, StateFlags, ItemKey)>>,
}

impl ConfigPage {
    fn new(screen_width: u16, screen_height: u16, area: Rect, fade: Option<bool>) -> Self {
        Self {
            items: (0..ROW_COUNT).map(|i| format!("Row {i:02}")).collect(),
            state: ListState::default(),
            screen_width,
            screen_height,
            area,
            parent_clip: None,
            fade,
            builder_order: 0,
            row_height: 1,
            row_gap: 0,
            whole_rows: false,
            custom_renderer: false,
            fade_after_renderer: false,
            global_patch: None,
            part_patches: Vec::new(),
            renderer_calls: core::cell::Cell::new(0),
            rendered_rows: core::cell::RefCell::new(Vec::new()),
        }
    }

    fn configured_list(&self, skip_fade: bool) -> ConfiguredList<'_> {
        let list = if skip_fade || self.fade.is_none() {
            match self.builder_order {
                1 => List::new(CONFIG_LIST)
                    .key(config_key as ConfigKey)
                    .row(config_row as ConfigRow),
                2 => List::new(CONFIG_LIST)
                    .row(config_row as ConfigRow)
                    .key(config_key as ConfigKey),
                _ => List::new(CONFIG_LIST)
                    .key(config_key as ConfigKey)
                    .row(config_row as ConfigRow),
            }
        } else {
            let enabled = self.fade.unwrap_or(false);
            match self.builder_order {
                1 => List::new(CONFIG_LIST)
                    .key(config_key as ConfigKey)
                    .fade_edges(enabled)
                    .row(config_row as ConfigRow),
                2 => List::new(CONFIG_LIST)
                    .row(config_row as ConfigRow)
                    .fade_edges(enabled)
                    .key(config_key as ConfigKey),
                _ => List::new(CONFIG_LIST)
                    .fade_edges(enabled)
                    .key(config_key as ConfigKey)
                    .row(config_row as ConfigRow),
            }
        };
        let mut list = list
            .select_mode(SelectMode::Multi)
            .row_height(self.row_height)
            .row_gap(self.row_gap)
            .whole_rows(self.whole_rows);
        if let Some(patch) = &self.global_patch {
            list = list.patch(patch);
        }
        if !self.part_patches.is_empty() {
            list = list.patch_part(&self.part_patches);
        }
        list
    }

    fn update_list(&self) -> ConfiguredList<'static> {
        List::new(CONFIG_LIST)
            .key(config_key as ConfigKey)
            .row(config_row as ConfigRow)
            .select_mode(SelectMode::Multi)
    }

    fn draw_list(&self, ui: &mut Ui<'_>) {
        let list = self.configured_list(self.custom_renderer && self.fade_after_renderer);
        if self.custom_renderer {
            let renderer = |ui: &mut Ui<'_>, row: Rect, flags, key, item: &String| {
                self.renderer_calls
                    .set(self.renderer_calls.get().saturating_add(1));
                self.rendered_rows.borrow_mut().push((row, flags, key));
                let style = Style::new().fg(CONFIG_FG).bg(CONFIG_BG);
                ui.fill(row, style);
                let label = Rect {
                    x: row.x.saturating_add(4),
                    width: row.width.saturating_sub(4),
                    ..row
                };
                let _ = ui.paint_str(label, item, style);
            };
            let rendered = list.render_row(&renderer);
            let rendered = if self.fade_after_renderer {
                match self.fade {
                    Some(enabled) => rendered.fade_edges(enabled),
                    None => rendered,
                }
            } else {
                rendered
            };
            let _ = rendered.draw(ui, self.area, &self.state, &self.items);
        } else {
            let _ = list.draw(ui, self.area, &self.state, &self.items);
        }
    }
}

impl App for ConfigPage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.update_list()
            .update(cx, &mut self.state, &self.items)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let outside = Style::new().fg(OUTSIDE_FG).bg(OUTSIDE_BG);
        ui.fill(ui.full(), outside);
        let draw = |ui: &mut Ui<'_>| self.draw_list(ui);
        if let Some(clip) = self.parent_clip {
            ui.with_area(clip, draw);
        } else {
            draw(ui);
        }
    }
}

fn config_harness(page: ConfigPage, theme: Theme) -> Harness<ConfigPage> {
    let width = page.screen_width;
    let height = page.screen_height;
    Harness::new(page, theme, width, height)
}

fn config_row_area(h: &Harness<ConfigPage>, index: usize) -> Option<Rect> {
    h.area_of_part(CONFIG_LIST, PartRef::item(Part::ROW, ItemKey::index(index)))
}

fn config_cells(h: &Harness<ConfigPage>) -> Vec<Cell> {
    let (width, height) = (h.app().screen_width, h.app().screen_height);
    (0..height)
        .flat_map(|y| (0..width).map(move |x| (x, y)))
        .map(|(x, y)| h.cell(x, y).clone())
        .collect()
}

fn assert_only_fade_channels(before: &Harness<ConfigPage>, after: &Harness<ConfigPage>) {
    let area = before.app().area;
    for y in area.y..area.bottom() {
        for x in area.x..area.right() {
            let old = before.cell(x, y);
            let new = after.cell(x, y);
            assert_eq!(new.symbol(), old.symbol(), "symbol at ({x}, {y})");
            assert_eq!(new.bg, old.bg, "background at ({x}, {y})");
            assert_eq!(
                new.underline_color, old.underline_color,
                "underline color at ({x}, {y})"
            );
            assert_eq!(
                new.modifier.difference(Modifier::DIM),
                old.modifier.difference(Modifier::DIM),
                "non-DIM modifiers at ({x}, {y})"
            );
        }
    }
}

#[test]
fn fade_configuration_survives_key_row_and_renderer_builder_orders() {
    let area = Rect::new(2, 1, 22, 6);
    let theme = rgb_theme();
    let mut off_frames = Vec::new();
    let mut on_edge_fgs = Vec::new();

    for order in 0..3 {
        let mut off = ConfigPage::new(SCREEN_WIDTH, SCREEN_HEIGHT, area, Some(false));
        off.builder_order = order;
        let off = config_harness(off, theme.clone());
        off_frames.push(config_cells(&off));

        let mut on = ConfigPage::new(SCREEN_WIDTH, SCREEN_HEIGHT, area, Some(true));
        on.builder_order = order;
        let on = config_harness(on, theme.clone());
        let edge = config_row_area(&on, 5).expect("last visible row");
        on_edge_fgs.push(on.cell(label_x(edge), edge.y).fg);
        assert_only_fade_channels(&off, &on);
    }
    assert!(off_frames.windows(2).all(|pair| pair[0] == pair[1]));
    assert!(on_edge_fgs.windows(2).all(|pair| pair[0] == pair[1]));
    assert!(
        on_edge_fgs[0] != CONFIG_FG,
        "all builder orders retain explicit fade-on"
    );

    let mut custom_auto = ConfigPage::new(SCREEN_WIDTH, SCREEN_HEIGHT, area, None);
    custom_auto.custom_renderer = true;
    let mut custom_auto = config_harness(custom_auto, theme.clone());
    custom_auto.app_mut().renderer_calls.set(0);
    custom_auto.app_mut().rendered_rows.borrow_mut().clear();
    custom_auto.draw();
    let callback_frames = config_cells(&custom_auto);
    let callback_count = custom_auto.app().renderer_calls.get();
    let callback_rows = custom_auto.app().rendered_rows.borrow().clone();
    assert_eq!(
        callback_count, 6,
        "custom callbacks run once per visible row"
    );
    assert_eq!(callback_rows.len(), callback_count);

    let mut custom_off = ConfigPage::new(SCREEN_WIDTH, SCREEN_HEIGHT, area, Some(false));
    custom_off.custom_renderer = true;
    let custom_off = config_harness(custom_off, theme.clone());
    assert_eq!(callback_frames, config_cells(&custom_off));

    let mut custom_on_before = ConfigPage::new(SCREEN_WIDTH, SCREEN_HEIGHT, area, Some(true));
    custom_on_before.custom_renderer = true;
    let custom_on_before = config_harness(custom_on_before, theme.clone());
    let mut custom_on_after = ConfigPage::new(SCREEN_WIDTH, SCREEN_HEIGHT, area, Some(true));
    custom_on_after.custom_renderer = true;
    custom_on_after.fade_after_renderer = true;
    let custom_on_after = config_harness(custom_on_after, theme);
    assert_eq!(
        config_cells(&custom_on_before),
        config_cells(&custom_on_after)
    );
    let edge = config_row_area(&custom_on_before, 5).expect("last visible row");
    assert_ne!(custom_on_before.cell(label_x(edge), edge.y).fg, CONFIG_FG);
    assert_only_fade_channels(&custom_off, &custom_on_before);
}

#[test]
fn fade_requires_overflow_an_admitted_bar_and_full_vertical_content() {
    let theme = rgb_theme();
    let cases = [
        (Rect::new(2, 1, 22, 6), 4, None),
        (Rect::new(2, 1, 22, 6), 0, None),
        (Rect::new(2, 1, 1, 6), ROW_COUNT, None),
        (Rect::new(2, 1, 0, 6), ROW_COUNT, None),
        (Rect::new(2, 1, 22, 0), ROW_COUNT, None),
        (
            Rect::new(2, 1, 22, 6),
            ROW_COUNT,
            Some(Rect::new(2, 2, 22, 4)),
        ),
    ];
    for (area, count, clip) in cases {
        let mut auto = ConfigPage::new(SCREEN_WIDTH, SCREEN_HEIGHT, area, None);
        auto.items.truncate(count);
        auto.parent_clip = clip;
        let auto = config_harness(auto, theme.clone());
        let mut off = ConfigPage::new(SCREEN_WIDTH, SCREEN_HEIGHT, area, Some(false));
        off.items.truncate(count);
        off.parent_clip = clip;
        let off = config_harness(off, theme.clone());
        assert_eq!(
            config_cells(&auto),
            config_cells(&off),
            "no fade for area={area:?}, item_count={count}, clip={clip:?}"
        );
    }

    let area = Rect::new(2, 1, 22, 6);
    let horizontal_clip = Rect::new(
        area.x.saturating_add(2),
        area.y,
        area.width.saturating_sub(2),
        area.height,
    );
    let mut clipped = ConfigPage::new(SCREEN_WIDTH, SCREEN_HEIGHT, area, None);
    clipped.parent_clip = Some(horizontal_clip);
    let clipped = config_harness(clipped, theme.clone());
    let mut clipped_off = ConfigPage::new(SCREEN_WIDTH, SCREEN_HEIGHT, area, Some(false));
    clipped_off.parent_clip = Some(horizontal_clip);
    let clipped_off = config_harness(clipped_off, theme);
    assert_only_fade_channels(&clipped_off, &clipped);
    let edge = config_row_area(&clipped, 5).expect("visible bottom row");
    assert_ne!(clipped.cell(label_x(edge), edge.y).fg, CONFIG_FG);
    for y in 0..SCREEN_HEIGHT {
        for x in 0..SCREEN_WIDTH {
            if x < horizontal_clip.x || x >= horizontal_clip.right() {
                assert_eq!(clipped.cell(x, y), clipped_off.cell(x, y));
            }
        }
    }
}

#[test]
fn fade_depth_changes_at_helper_threshold_and_cursor_protection_uses_rows() {
    let theme = rgb_theme();
    for height in [11, 12] {
        let area = Rect::new(2, 1, 22, height);
        let auto = ConfigPage::new(SCREEN_WIDTH, SCREEN_HEIGHT + 6, area, None);
        let off = ConfigPage::new(SCREEN_WIDTH, SCREEN_HEIGHT + 6, area, Some(false));
        let mut auto = config_harness(auto, theme.clone());
        let mut off = config_harness(off, theme.clone());
        let _ = auto.wheel(
            Axis::V,
            1,
            area.x.saturating_add(5),
            area.y.saturating_add(2),
        );
        let _ = off.wheel(
            Axis::V,
            1,
            area.x.saturating_add(5),
            area.y.saturating_add(2),
        );
        assert!(auto.app().state.scroll().offset() > 0);
        let inner_x = label_x(config_row_area(&auto, 3).expect("top visible item"));
        let inner_y = area.y.saturating_add(1);
        let auto_inner = auto.cell(inner_x, inner_y);
        let off_inner = off.cell(inner_x, inner_y);
        if height == 11 {
            assert_eq!(
                auto_inner, off_inner,
                "shallow view fades only the outer row"
            );
        } else {
            assert_ne!(auto_inner.fg, off_inner.fg, "12 rows enable the inner fade");
        }
    }

    let area = Rect::new(2, 1, 22, 12);
    let mut auto = ConfigPage::new(SCREEN_WIDTH, SCREEN_HEIGHT + 6, area, None);
    auto.row_height = 2;
    auto.row_gap = 1;
    auto.whole_rows = true;
    auto.state.set_cursor(3, ItemKey::index(3));
    let mut off = ConfigPage::new(SCREEN_WIDTH, SCREEN_HEIGHT + 6, area, Some(false));
    off.row_height = 2;
    off.row_gap = 1;
    off.whole_rows = true;
    off.state.set_cursor(3, ItemKey::index(3));
    let auto = config_harness(auto, theme.clone());
    let off = config_harness(off, theme);
    let cursor = config_row_area(&auto, 3).expect("cursor row is painted");
    assert_eq!(cursor.y, area.y.saturating_add(9));
    assert_eq!(cursor.height, 2);
    let label = label_x(cursor);
    let protected_y = cursor.bottom().saturating_sub(1);
    assert_eq!(auto.cell(label, protected_y), off.cell(label, protected_y));
}

#[test]
fn fade_preserves_non_ink_channels_and_obeys_background_and_reverse_rules() {
    let theme = rgb_theme();
    let cases = [
        ("fg", true, "explicit foreground still fades"),
        ("bg", false, "different authored background is ineligible"),
        (
            "clear-bg",
            true,
            "cleared patch remains eligible when its resolved background is the majority",
        ),
        (
            "reset-bg",
            true,
            "reset patch remains eligible when its resolved background is the majority",
        ),
        ("reversed", false, "reversed cells are ineligible"),
        (
            "underlined",
            true,
            "underline remains while eligible ink fades",
        ),
    ];

    for (tag, should_change, name) in cases {
        let area = Rect::new(2, 1, 22, 6);
        let mut before_page = ConfigPage::new(SCREEN_WIDTH, SCREEN_HEIGHT, area, Some(false));
        before_page.items[5] = format!("Row 05|{tag}");
        let before = config_harness(before_page, theme.clone());
        let mut after_page = ConfigPage::new(SCREEN_WIDTH, SCREEN_HEIGHT, area, Some(true));
        after_page.items[5] = format!("Row 05|{tag}");
        let after = config_harness(after_page, theme.clone());
        assert_only_fade_channels(&before, &after);

        let edge = config_row_area(&before, 5).expect("bottom edge row");
        let old = before.cell(label_x(edge), edge.y);
        let new = after.cell(label_x(edge), edge.y);
        match tag {
            "bg" => assert_eq!(old.bg, OTHER_BG, "Set changes the final cell surface"),
            "clear-bg" | "reset-bg" => assert_eq!(old.bg, ROW_BG),
            "reversed" => assert!(old.modifier.contains(Modifier::REVERSED)),
            "underlined" => assert!(old.modifier.contains(Modifier::UNDERLINED)),
            _ => {}
        }
        let changed = old.fg != new.fg || old.modifier != new.modifier;
        assert_eq!(changed, should_change, "{name}: old={old:?}, new={new:?}");
        if !should_change {
            assert_eq!(new, old, "{name}");
        }
        if name.contains("underline") {
            assert!(new.modifier.contains(Modifier::UNDERLINED));
        }
    }

    for level in [
        termrock_theme::ColorLevel::TrueColor,
        termrock_theme::ColorLevel::Ansi256,
        termrock_theme::ColorLevel::Ansi16,
        termrock_theme::ColorLevel::Mono,
    ] {
        let area = Rect::new(2, 1, 22, 6);
        let before = config_harness(
            ConfigPage::new(SCREEN_WIDTH, SCREEN_HEIGHT, area, Some(false)),
            rgb_theme(),
        )
        .with_color(level);
        let after = config_harness(
            ConfigPage::new(SCREEN_WIDTH, SCREEN_HEIGHT, area, Some(true)),
            rgb_theme(),
        )
        .with_color(level);
        assert_only_fade_channels(&before, &after);
        let edge = config_row_area(&before, 5).expect("bottom edge row");
        let old = before.cell(label_x(edge), edge.y);
        let new = after.cell(label_x(edge), edge.y);
        assert!(
            old.fg != new.fg
                || (!old.modifier.contains(Modifier::DIM) && new.modifier.contains(Modifier::DIM)),
            "eligible edge ink follows the helper at {level:?}"
        );
    }
}

#[test]
fn faded_cursor_protection_tracks_stable_items_and_does_not_follow_offscreen_cursor() {
    let area = Rect::new(2, 1, 22, 6);
    let theme = rgb_theme();
    let mut page = ConfigPage::new(SCREEN_WIDTH, SCREEN_HEIGHT, area, None);
    page.state.set_cursor(5, ItemKey::index(5));
    let mut h = config_harness(page, theme.clone());
    let row5 = config_row_area(&h, 5).expect("initial cursor row");
    assert_eq!(row5.bottom(), area.bottom());
    assert_eq!(h.cell(label_x(row5), row5.y).fg, CONFIG_FG);

    h.app_mut().items.swap(3, 5);
    let _ = h.tick();
    assert_eq!(h.app().state.cursor(), Some(ItemKey::index(5)));
    let moved_cursor = config_row_area(&h, 5).expect("stable cursor after reorder");
    assert_eq!(moved_cursor.y, area.y.saturating_add(3));
    assert_eq!(h.cell(label_x(moved_cursor), moved_cursor.y).fg, CONFIG_FG);
    let bottom_after_reorder = config_row_area(&h, 3).expect("new bottom item");
    assert_eq!(bottom_after_reorder.bottom(), area.bottom());
    assert_ne!(
        h.cell(label_x(bottom_after_reorder), bottom_after_reorder.y)
            .fg,
        CONFIG_FG
    );

    h.app_mut().items.remove(0);
    let _ = h.tick();
    assert_eq!(h.app().state.cursor(), Some(ItemKey::index(5)));
    let moved_again = config_row_area(&h, 5).expect("stable cursor after removal");
    assert_eq!(moved_again.y, area.y.saturating_add(2));
    let bottom_after_remove = config_row_area(&h, 6).expect("bottom item after removal");
    assert_eq!(bottom_after_remove.bottom(), area.bottom());
    assert_ne!(
        h.cell(label_x(bottom_after_remove), bottom_after_remove.y)
            .fg,
        CONFIG_FG
    );

    let mut offscreen_page = ConfigPage::new(SCREEN_WIDTH, SCREEN_HEIGHT, area, None);
    offscreen_page.state.set_cursor(0, ItemKey::index(0));
    let mut offscreen = config_harness(offscreen_page, theme);
    let _ = offscreen.wheel(Axis::V, 1, area.x + 5, area.y + 2);
    assert_eq!(offscreen.app().state.cursor(), Some(ItemKey::index(0)));
    assert!(offscreen.app().state.scroll().offset() > 0);
    let top = config_row_area(&offscreen, 3).expect("top visible row after wheel");
    assert_eq!(top.y, area.y);
    assert_ne!(offscreen.cell(label_x(top), top.y).fg, CONFIG_FG);
}

#[test]
fn scrollbar_track_and_thumb_hits_still_scroll_and_reveal_real_edges() {
    let (mut h, _) = harness();
    let track = h
        .area_of_part(LIST, PartRef::of(Part::TRACK))
        .expect("overflow track registers a hit target");
    let thumb = h
        .area_of_part(LIST, PartRef::of(Part::THUMB))
        .expect("overflow thumb registers a hit target");
    let start = h.app().state.scroll().offset();
    let _ = h.drag(
        (thumb.x, thumb.y),
        (thumb.x, track.bottom().saturating_sub(1)),
    );
    assert!(h.app().state.scroll().offset() > start);
    assert!(h.area_of_part(LIST, PartRef::of(Part::TRACK)).is_some());
    assert!(h.area_of_part(LIST, PartRef::of(Part::THUMB)).is_some());
    let visible_first = row(&h, h.app().state.scroll().offset());
    let text = h.cell(label_x(visible_first), visible_first.y);
    assert_ne!(
        text.fg, ROW_FG,
        "the visible first row has hidden rows above"
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
