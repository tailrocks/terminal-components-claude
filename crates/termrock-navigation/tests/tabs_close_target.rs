use core::cell::Cell;
use core::fmt;

use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Modifier, Style};
use termrock_core::{Id, ItemKey, KeyCode, MouseKind, Part, PartRef, Response};
use termrock_navigation::{TabMarks, Tabs, TabsAction, TabsState};
use termrock_runtime::rowui::RowUi;
use termrock_runtime::{App, Cx, Status, Ui};
use termrock_test_support::Harness;
use termrock_theme::{ColorLevel, GlyphRole, Role, Slot, StylePatch, Theme};

const TABS: Id = Id::root("tabs.close-target");

const SET_CLOSE: [(Part, StylePatch); 1] = [(
    Part::CLOSE,
    StylePatch::new()
        .set_fg(Role::Warning)
        .set_bg(Role::AccentTint)
        .add(Modifier::UNDERLINED),
)];
const GLOBAL_SET: StylePatch = StylePatch::new()
    .set_fg(Role::Warning)
    .set_bg(Role::AccentTint)
    .add(Modifier::UNDERLINED);
const GLOBAL_CLEAR: StylePatch = StylePatch {
    fg: Slot::Clear,
    bg: Slot::Clear,
    underline: Slot::Clear,
    ..StylePatch::new()
};
const CLEAR_CLOSE: [(Part, StylePatch); 1] = [(
    Part::CLOSE,
    StylePatch {
        fg: Slot::Clear,
        bg: Slot::Clear,
        underline: Slot::Clear,
        ..StylePatch::new()
    },
)];
const RESET_CLOSE: [(Part, StylePatch); 1] = [(
    Part::CLOSE,
    StylePatch::new()
        .set_fg(Role::Custom(Color::Reset))
        .set_bg(Role::Custom(Color::Reset))
        .set_underline(Role::Custom(Color::Reset)),
)];
const CLEAR_GLYPH_CLOSE: [(Part, StylePatch); 1] = [(
    Part::CLOSE,
    StylePatch {
        glyph: Slot::Clear,
        ..StylePatch::new()
    },
)];

#[derive(Clone, Copy)]
struct Tab {
    id: u64,
    label: &'static str,
    marked: bool,
}

impl fmt::Display for Tab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label)
    }
}

fn tab_key(tab: &Tab) -> ItemKey {
    ItemKey::num(tab.id)
}

fn tab_row(tab: &Tab, row: &mut RowUi<'_>) {
    row.label(tab.label);
}

fn tab_mark(tab: &Tab) -> TabMarks {
    TabMarks::default().dirty(tab.marked)
}

#[derive(Clone, Copy, Debug, Default)]
enum ClosePatch {
    #[default]
    None,
    Set,
    GlobalSet,
    Clear,
    GlobalClear,
    Reset,
    CustomGlyph,
    ClearGlyph,
}

type DefaultTabs = Tabs<'static, Tab, fn(&Tab) -> ItemKey>;

struct Page {
    state: TabsState,
    items: Vec<Tab>,
    target_width: Option<u16>,
    danger_hover: bool,
    explicit_danger_false: bool,
    marked: bool,
    planned: bool,
    custom_row: bool,
    builder_order: bool,
    reveal: termrock_navigation::TabCloseReveal,
    patch: ClosePatch,
    plan_width: Option<Option<u16>>,
    plan_danger: Option<bool>,
    empty_plan: bool,
    sentinel: bool,
    rejected: Cell<bool>,
    actions: Vec<TabsAction>,
}

impl Page {
    fn new(target_width: Option<u16>, danger_hover: bool, label: &'static str) -> Self {
        Self {
            state: TabsState::default(),
            items: vec![Tab {
                id: 41,
                label,
                marked: false,
            }],
            target_width,
            danger_hover,
            explicit_danger_false: false,
            marked: false,
            planned: false,
            custom_row: false,
            builder_order: false,
            reveal: termrock_navigation::TabCloseReveal::Always,
            patch: ClosePatch::None,
            plan_width: None,
            plan_danger: None,
            empty_plan: false,
            sentinel: false,
            rejected: Cell::new(false),
            actions: Vec::new(),
        }
    }

    fn tabs(&self, width: Option<u16>, danger: bool) -> DefaultTabs {
        let base = Tabs::new(TABS)
            .closable(true)
            .middle_click_closes(false)
            .close_reveal(self.reveal);
        let configured = if self.builder_order {
            let base = match width {
                Some(width) => base.close_target_w(width),
                None => base,
            };
            let base = if danger || self.explicit_danger_false {
                base.close_danger_hover(danger)
            } else {
                base
            };
            base.key(tab_key as fn(&Tab) -> ItemKey)
        } else {
            let base = base.key(tab_key as fn(&Tab) -> ItemKey);
            let base = match width {
                Some(width) => base.close_target_w(width),
                None => base,
            };
            if danger || self.explicit_danger_false {
                base.close_danger_hover(danger)
            } else {
                base
            }
        };
        let configured = if self.marked {
            configured.item_marks(tab_mark)
        } else {
            configured
        };
        match self.patch {
            ClosePatch::None => configured,
            ClosePatch::Set => configured.patch_part(&SET_CLOSE),
            ClosePatch::GlobalSet => configured.patch(&GLOBAL_SET),
            ClosePatch::Clear => configured.patch_part(&CLEAR_CLOSE),
            ClosePatch::GlobalClear => configured.patch(&GLOBAL_CLEAR),
            ClosePatch::Reset => configured.patch_part(&RESET_CLOSE),
            ClosePatch::CustomGlyph => {
                static GLYPH: [(Part, StylePatch); 1] = [(
                    Part::CLOSE,
                    StylePatch::new()
                        .set_glyph(GlyphRole::Error)
                        .set_fg(Role::Danger),
                )];
                configured.patch_part(&GLYPH)
            }
            ClosePatch::ClearGlyph => configured.patch_part(&CLEAR_GLYPH_CLOSE),
        }
    }
}

impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let tabs = self.tabs(self.target_width, self.danger_hover);
        let mut response = if self.custom_row {
            tabs.row(tab_row as for<'a> fn(&Tab, &mut RowUi<'a>))
                .update(cx, &mut self.state, &self.items)
        } else {
            tabs.update(cx, &mut self.state, &self.items)
        };
        if let Some(action) = response.take_action() {
            self.actions.push(action);
        }
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        if self.sentinel {
            ui.fill(ui.full(), Style::default().bg(Color::Blue));
        }
        let tabs = self.tabs(self.target_width, self.danger_hover);
        if self.planned {
            let plan_tabs = self.tabs(
                self.plan_width.unwrap_or(self.target_width),
                self.plan_danger.unwrap_or(self.danger_hover),
            );
            let clip = ui.full();
            let area = if self.empty_plan {
                Rect::new(clip.x, clip.y, 0, 0)
            } else {
                clip
            };
            let layout = plan_tabs.layout_default(area, clip, &self.state, &self.items, None);
            self.rejected.set(tabs.draw_layout(ui, &layout).is_err());
        } else if self.custom_row {
            tabs.row(tab_row as for<'a> fn(&Tab, &mut RowUi<'a>)).draw(
                ui,
                ui.full(),
                &self.state,
                &self.items,
            );
        } else {
            tabs.draw(ui, ui.full(), &self.state, &self.items);
        }
    }
}

fn harness(page: Page, width: u16, theme: Theme) -> Harness<Page> {
    Harness::new(page, theme, width, 3)
}

fn tab_area(h: &Harness<Page>) -> Rect {
    h.area_of_part(TABS, PartRef::item(Part::TAB, ItemKey::num(41)))
        .expect("tab hit is present")
}

fn close_area(h: &Harness<Page>) -> Rect {
    h.area_of_part(TABS, PartRef::item(Part::CLOSE, ItemKey::num(41)))
        .expect("eligible close target is present")
}

fn close_area_for(h: &Harness<Page>, key: ItemKey) -> Option<Rect> {
    h.area_of_part(TABS, PartRef::item(Part::CLOSE, key))
}

fn tab_area_for(h: &Harness<Page>, key: ItemKey) -> Rect {
    h.area_of_part(TABS, PartRef::item(Part::TAB, key))
        .expect("tab hit is present")
}

fn marked_hover_page(target_width: Option<u16>) -> Page {
    let mut page = Page::new(target_width, false, "Active");
    page.state.set_active(0, ItemKey::num(41));
    page.reveal = termrock_navigation::TabCloseReveal::Hover;
    page.marked = true;
    page.items[0].marked = false;
    page.items.push(Tab {
        id: 42,
        label: "B",
        marked: true,
    });
    page
}

#[test]
fn omitted_target_and_explicit_one_cell_preserve_complete_default_render() {
    let omitted = harness(Page::new(None, false, "LongLabel"), 48, Theme::junie());
    let explicit = harness(Page::new(Some(1), false, "LongLabel"), 48, Theme::junie());
    let zero = harness(Page::new(Some(0), false, "LongLabel"), 48, Theme::junie());

    assert_eq!(omitted.buffer(), explicit.buffer());
    assert_eq!(omitted.buffer(), zero.buffer());
    assert_eq!(
        omitted.runtime().registry().regions(),
        explicit.runtime().registry().regions()
    );
    assert_eq!(
        omitted.runtime().registry().regions(),
        zero.runtime().registry().regions()
    );
    assert_eq!(close_area(&omitted), close_area(&explicit));
    assert_eq!(close_area(&omitted), close_area(&zero));

    let implicit_false = harness(Page::new(None, false, "LongLabel"), 48, Theme::junie());
    let mut explicit_page = Page::new(None, false, "LongLabel");
    explicit_page.explicit_danger_false = true;
    let explicit_false = harness(explicit_page, 48, Theme::junie());
    assert_eq!(implicit_false.buffer(), explicit_false.buffer());
    assert_eq!(
        implicit_false.runtime().registry().regions(),
        explicit_false.runtime().registry().regions()
    );

    let danger_only = harness(Page::new(None, true, "LongLabel"), 48, Theme::junie());
    assert_eq!(implicit_false.buffer(), danger_only.buffer());
    assert_eq!(
        implicit_false.runtime().registry().regions(),
        danger_only.runtime().registry().regions()
    );

    let mut default_hover = harness(Page::new(None, false, "VeryLongLabel"), 48, Theme::junie());
    let mut one_cell_hover = harness(
        Page::new(Some(1), false, "VeryLongLabel"),
        48,
        Theme::junie(),
    );
    let target = close_area(&default_hover);
    let _ = default_hover.mouse(MouseKind::Move, target.x, target.y);
    let _ = one_cell_hover.mouse(MouseKind::Move, target.x, target.y);
    assert_eq!(default_hover.buffer(), one_cell_hover.buffer());
    assert_eq!(
        default_hover.runtime().registry().regions(),
        one_cell_hover.runtime().registry().regions()
    );
}

#[test]
fn explicit_width_rounding_and_clamping_change_only_the_close_hit_plate() {
    let widths = [(0, 1), (1, 1), (2, 1), (3, 3), (4, 3), (5, 5)];
    let baseline = harness(Page::new(None, false, "VeryLongLabel"), 64, Theme::junie());
    let baseline_tab = tab_area(&baseline);
    let baseline_cells = baseline.buffer().clone();

    for (requested, expected_width) in widths {
        let h = harness(
            Page::new(Some(requested), false, "VeryLongLabel"),
            64,
            Theme::junie(),
        );
        let tab = tab_area(&h);
        let target = close_area(&h);
        assert_eq!(tab, baseline_tab, "requested width {requested}");
        assert_eq!(h.buffer(), &baseline_cells, "requested width {requested}");
        assert_eq!(target.width, expected_width, "requested width {requested}");
        assert_eq!(target.y, tab.y);
        assert!(target.x >= tab.x && target.right() <= tab.right());
        let expected_x = match requested {
            0..=2 => tab.right().saturating_sub(2),
            3..=4 => tab.right().saturating_sub(3),
            _ => tab.right().saturating_sub(5),
        };
        assert_eq!(target.x, expected_x, "requested width {requested}");
    }

    let maximum = harness(
        Page::new(Some(u16::MAX), false, "VeryLongLabel"),
        64,
        Theme::junie(),
    );
    let tab = tab_area(&maximum);
    let target = close_area(&maximum);
    assert_eq!(target, tab, "the maximum request is safely clamped in-tab");

    let narrow = harness(
        Page::new(Some(u16::MAX), false, "A very long clipped label"),
        8,
        Theme::junie(),
    );
    let tab = tab_area(&narrow);
    let target = close_area(&narrow);
    assert!(target.width <= tab.width);
    assert!(target.x >= tab.x && target.right() <= tab.right());

    let unicode = harness(Page::new(Some(5), false, "界e\u{301}Z"), 32, Theme::junie());
    assert_eq!(unicode.cell(1, 0).symbol(), "界");
    assert_eq!(unicode.cell(3, 0).symbol(), "e\u{301}");
    assert_eq!(close_area(&unicode).width, 5);
}

#[test]
fn every_plate_edge_is_a_real_close_target_without_changing_activation() {
    for edge in [0, 4] {
        let mut h = harness(
            Page::new(Some(5), false, "VeryLongLabel"),
            64,
            Theme::junie(),
        );
        let target = close_area(&h);
        let x = target.x + edge;
        let active = h.app().state.active();
        let cursor = h.app().state.cursor();
        let _ = h.click(x, target.y);
        assert_eq!(h.app().actions, [TabsAction::Close(ItemKey::num(41))]);
        assert_eq!(h.app().state.active(), active);
        assert_eq!(h.app().state.cursor(), cursor);
    }

    let mut zero = harness(
        Page::new(Some(0), false, "VeryLongLabel"),
        64,
        Theme::junie(),
    );
    let target = close_area(&zero);
    let _ = zero.click(target.x, target.y);
    assert_eq!(zero.app().actions, [TabsAction::Close(ItemKey::num(41))]);
}

#[test]
fn wide_hover_styles_the_plate_without_erasing_overlapping_label_or_mark_glyphs() {
    let mut page = Page::new(Some(5), false, "A");
    page.marked = true;
    page.items[0].marked = true;
    let mut h = harness(page, 32, Theme::junie());
    let target = close_area(&h);
    let before: Vec<_> = (target.x..target.right())
        .map(|x| {
            let cell = h.cell(x, target.y);
            (
                cell.symbol().to_owned(),
                cell.fg,
                cell.bg,
                cell.underline_color,
                cell.modifier,
            )
        })
        .collect();
    let tab = tab_area(&h);
    let mark = Rect::new(tab.x + 2, tab.y, 2, 1);
    assert!((mark.x..mark.right()).any(|x| h.cell(x, mark.y).symbol() != " "));
    assert!(target.x < mark.right() && mark.x < target.right());

    let _ = h.mouse(MouseKind::Move, target.x, target.y);
    for x in target.x..target.right() {
        assert_eq!(
            h.cell(x, target.y).symbol(),
            before[usize::from(x - target.x)].0
        );
    }
    let close_style = h.cell(target.right() - 1, target.y);
    for x in target.x..target.right() {
        let cell = h.cell(x, target.y);
        assert_eq!(cell.fg, close_style.fg, "plate foreground at x={x}");
        assert_eq!(cell.bg, close_style.bg, "plate background at x={x}");
        assert_eq!(cell.underline_color, close_style.underline_color);
        assert_eq!(cell.modifier, close_style.modifier);
    }

    let _ = h.mouse(MouseKind::Move, 31, 2);
    for x in target.x..target.right() {
        let cell = h.cell(x, target.y);
        let original = &before[usize::from(x - target.x)];
        assert_eq!(cell.symbol(), original.0);
        assert_eq!(cell.fg, original.1);
        assert_eq!(cell.bg, original.2);
        assert_eq!(cell.underline_color, original.3);
        assert_eq!(cell.modifier, original.4);
    }
}

#[test]
fn explicit_clear_and_reset_reach_only_the_resting_glyph_in_opted_paths() {
    for patch in [
        ClosePatch::Clear,
        ClosePatch::GlobalClear,
        ClosePatch::Reset,
    ] {
        for danger_hover in [false, true] {
            let mut page = Page::new(Some(5), danger_hover, "A");
            page.patch = patch;
            page.marked = true;
            page.items[0].marked = true;
            let mut h = harness(page, 32, Theme::junie());

            let (baseline_width, baseline_danger, baseline_patch) = match patch {
                ClosePatch::GlobalClear => (None, false, ClosePatch::GlobalClear),
                _ => (Some(5), danger_hover, ClosePatch::None),
            };
            let mut baseline_page = Page::new(baseline_width, baseline_danger, "A");
            baseline_page.marked = true;
            baseline_page.items[0].marked = true;
            baseline_page.patch = baseline_patch;
            let baseline = harness(baseline_page, 32, Theme::junie());

            let target = close_area(&h);
            let tab = tab_area(&h);
            let glyph_x = tab.right() - 2;
            let mark = Rect::new(tab.x + 2, tab.y, 2, 1);
            assert_eq!(target.width, 5);
            assert!((mark.x..mark.right()).any(|x| h.cell(x, mark.y).symbol() != " "));
            assert!(target.x < mark.right() && mark.x < target.right());
            assert!(target.x <= glyph_x && glyph_x < target.right());

            let rest_cells: Vec<_> = (target.x..target.right())
                .map(|x| {
                    let cell = h.cell(x, target.y);
                    (
                        cell.symbol().to_owned(),
                        cell.fg,
                        cell.bg,
                        cell.underline_color,
                        cell.modifier,
                    )
                })
                .collect();
            let glyph = h.cell(glyph_x, target.y);
            assert_eq!(glyph.fg, Color::Reset, "{patch:?}, danger={danger_hover}");
            assert_eq!(glyph.bg, Color::Reset, "{patch:?}, danger={danger_hover}");
            assert_eq!(
                glyph.underline_color,
                Color::Reset,
                "{patch:?}, danger={danger_hover}"
            );
            assert_eq!(glyph.symbol(), baseline.cell(glyph_x, target.y).symbol());
            for x in target.x..target.right() {
                if x != glyph_x {
                    assert_eq!(
                        h.cell(x, target.y).style(),
                        baseline.cell(x, target.y).style(),
                        "resting padding x={x}, {patch:?}, danger={danger_hover}"
                    );
                    assert_eq!(
                        h.cell(x, target.y).symbol(),
                        baseline.cell(x, target.y).symbol(),
                        "resting content x={x}, {patch:?}, danger={danger_hover}"
                    );
                }
            }

            let _ = h.mouse(MouseKind::Move, target.x, target.y);
            for x in target.x..target.right() {
                let cell = h.cell(x, target.y);
                assert_eq!(cell.fg, Color::Reset);
                assert_eq!(cell.bg, Color::Reset);
                assert_eq!(cell.underline_color, Color::Reset);
                assert_eq!(
                    cell.symbol(),
                    rest_cells[usize::from(x - target.x)].0,
                    "hover glyph preservation x={x}"
                );
            }
            let _ = h.mouse(MouseKind::Down, target.x + 2, target.y);
            for x in target.x..target.right() {
                let cell = h.cell(x, target.y);
                assert_eq!(cell.fg, Color::Reset);
                assert_eq!(cell.bg, Color::Reset);
                assert_eq!(cell.underline_color, Color::Reset);
                assert_eq!(
                    cell.symbol(),
                    rest_cells[usize::from(x - target.x)].0,
                    "pressed glyph preservation x={x}"
                );
            }
            let _ = h.mouse(MouseKind::Up, target.x + 2, target.y);
            assert_eq!(
                h.app().actions,
                [TabsAction::Close(ItemKey::num(41))],
                "release over the live plate closes once"
            );
            for x in target.x..target.right() {
                let cell = h.cell(x, target.y);
                assert_eq!(cell.fg, Color::Reset);
                assert_eq!(cell.bg, Color::Reset);
                assert_eq!(cell.underline_color, Color::Reset);
                assert_eq!(
                    cell.symbol(),
                    rest_cells[usize::from(x - target.x)].0,
                    "release glyph preservation x={x}"
                );
            }
            let _ = h.mouse(MouseKind::Move, 31, 2);
            let _ = h.advance(core::time::Duration::from_millis(1_000));
            h.draw();
            for x in target.x..target.right() {
                let cell = h.cell(x, target.y);
                let actual = (
                    cell.symbol().to_owned(),
                    cell.fg,
                    cell.bg,
                    cell.underline_color,
                    cell.modifier,
                );
                assert_eq!(
                    actual,
                    rest_cells[usize::from(x - target.x)],
                    "cleanup x={x}, target={target:?}, tab={tab:?}, glyph={glyph_x}, hover={:?}, state={:?}, {patch:?}, danger={danger_hover}",
                    h.hover(),
                    h.state_of(TABS)
                );
            }
        }
    }

    // Danger mode alone opts into explicit close resolution; omitted width
    // plus danger=false keeps the historic rendering path.
    for patch in [
        ClosePatch::Clear,
        ClosePatch::GlobalClear,
        ClosePatch::Reset,
    ] {
        let mut page = Page::new(None, true, "LongLabel");
        page.patch = patch;
        let mut h = harness(page, 32, Theme::junie());
        let mut baseline_page = Page::new(None, false, "LongLabel");
        baseline_page.patch = patch;
        let baseline = harness(baseline_page, 32, Theme::junie());
        let target = close_area(&h);
        let glyph = h.cell(target.x, target.y);
        assert_eq!(glyph.fg, Color::Reset, "danger-only {patch:?}");
        assert_eq!(glyph.bg, Color::Reset, "danger-only {patch:?}");
        assert_eq!(glyph.underline_color, Color::Reset, "danger-only {patch:?}");
        for y in 0..3 {
            for x in 0..32 {
                if (x, y) != (target.x, target.y) {
                    assert_eq!(h.cell(x, y), baseline.cell(x, y));
                }
            }
        }

        let _ = h.mouse(MouseKind::Move, target.x, target.y);
        let _ = h.mouse(MouseKind::Down, target.x, target.y);
        let _ = h.mouse(MouseKind::Up, target.x, target.y);
        let _ = h.mouse(MouseKind::Move, 31, 2);
        h.draw();
        let glyph = h.cell(target.x, target.y);
        assert_eq!(glyph.fg, Color::Reset, "danger-only cleanup {patch:?}");
        assert_eq!(glyph.bg, Color::Reset, "danger-only cleanup {patch:?}");
    }
}

#[test]
fn widened_hover_uses_tab_then_close_hits_and_cleans_pressed_only_reveal() {
    let idle = ItemKey::num(42);
    let mut h = harness(marked_hover_page(Some(5)), 48, Theme::junie());
    let idle_tab = tab_area_for(&h, idle);
    let padding_x = idle_tab.right() - 5;
    let glyph_x = idle_tab.right() - 2;
    let mark = Rect::new(idle_tab.x + 2, idle_tab.y, 2, 1);
    assert!(padding_x < glyph_x);
    assert_eq!(close_area_for(&h, idle), None);
    assert_eq!(h.cell(glyph_x, idle_tab.y).symbol(), " ");
    assert!((mark.x..mark.right()).any(|x| h.cell(x, mark.y).symbol() != " "));

    let _ = h.mouse(MouseKind::Down, padding_x, idle_tab.y);
    assert_eq!(h.app().state.active(), Some(ItemKey::num(41)));
    assert_eq!(h.app().state.cursor(), Some(idle));
    assert!(h.app().actions.is_empty());
    assert_eq!(close_area_for(&h, idle), None);
    assert_eq!(
        h.cell(glyph_x, idle_tab.y).symbol(),
        Theme::junie().design.glyphs.get(GlyphRole::Close),
        "the close paints while TAB retains the live hit"
    );
    let _ = h.mouse(MouseKind::Up, padding_x, idle_tab.y);
    assert_eq!(h.app().actions, [TabsAction::Activated(idle)]);
    assert_eq!(h.app().state.active(), Some(idle));
    assert_eq!(
        h.app().items.len(),
        2,
        "activation does not remove the item"
    );
    let target = close_area_for(&h, idle).expect("activation reveals the close");
    assert_eq!(target.width, 5);
    assert_eq!(target.x, padding_x);

    h.app_mut().actions.clear();
    let _ = h.click(padding_x, idle_tab.y);
    assert_eq!(h.app().actions, [TabsAction::Close(idle)]);
    assert_eq!(h.app().items.len(), 2, "close action remains caller-owned");

    let mut h = harness(marked_hover_page(Some(5)), 48, Theme::junie());
    let original = h.buffer().clone();
    let idle_tab = tab_area_for(&h, idle);
    let close_x = idle_tab.right() - 5;
    let glyph_x = idle_tab.right() - 2;
    let _ = h.mouse(MouseKind::Move, close_x, idle_tab.y);
    let target = close_area_for(&h, idle).expect("tab hover reveals widened target");
    assert_eq!(target.width, 5);
    assert_ne!(h.cell(glyph_x, idle_tab.y).symbol(), " ");
    let _ = h.mouse(MouseKind::Down, close_x, idle_tab.y);
    let _ = h.mouse(MouseKind::Drag, 47, 2);
    assert_eq!(close_area_for(&h, idle), Some(target));
    assert_ne!(h.cell(glyph_x, idle_tab.y).symbol(), " ");
    assert!(
        h.state_of(TABS)
            .contains(termrock_runtime::StateFlags::PRESSED)
    );
    assert!(h.app().actions.is_empty());
    let _ = h.mouse(MouseKind::Up, 47, 2);
    assert!(h.app().actions.is_empty(), "outside release does not close");
    assert_eq!(close_area_for(&h, idle), None);
    h.draw();
    assert_eq!(
        h.buffer(),
        &original,
        "label, mark, plate and glyph restore"
    );
}

#[test]
fn danger_presentation_uses_semantic_roles_at_every_color_tier() {
    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        let theme = Theme::junie().for_level(level);
        let danger = theme.color.danger;
        let on_danger = theme.color.on_danger;
        let mut danger_h = harness(Page::new(Some(5), true, "VeryLongLabel"), 64, theme.clone());
        let mut ordinary_h = harness(Page::new(Some(5), false, "VeryLongLabel"), 64, theme);
        let target = close_area(&danger_h);
        assert_eq!(
            danger_h.buffer(),
            ordinary_h.buffer(),
            "rest style at {level:?}"
        );

        let _ = danger_h.mouse(MouseKind::Move, target.x, target.y);
        let _ = ordinary_h.mouse(MouseKind::Move, target.x, target.y);
        for x in target.x..target.right() {
            let danger_cell = danger_h.cell(x, target.y);
            let ordinary_cell = ordinary_h.cell(x, target.y);
            assert_eq!(danger_cell.fg, danger, "hover role at {level:?}, x={x}");
            assert_eq!(danger_cell.bg, ordinary_cell.bg);
            assert_eq!(danger_cell.underline_color, ordinary_cell.underline_color);
            assert_eq!(danger_cell.modifier, ordinary_cell.modifier);
            assert_eq!(danger_cell.symbol(), ordinary_cell.symbol());
        }

        let _ = danger_h.mouse(MouseKind::Down, target.x + target.width / 2, target.y);
        let _ = ordinary_h.mouse(MouseKind::Down, target.x + target.width / 2, target.y);
        let pressed = danger_h.cell(target.x, target.y);
        let ordinary_pressed = ordinary_h.cell(target.x, target.y);
        assert_eq!(pressed.fg, on_danger, "pressed fg role at {level:?}");
        assert_eq!(pressed.bg, danger, "pressed bg role at {level:?}");
        assert_eq!(pressed.underline_color, ordinary_pressed.underline_color);
        assert_eq!(pressed.modifier, ordinary_pressed.modifier);
        assert_eq!(pressed.symbol(), ordinary_pressed.symbol());
    }

    let mut custom_theme = Theme::junie();
    custom_theme.capability_palettes = None;
    custom_theme.color.danger = Color::from_u32(0x00d1_2301);
    custom_theme.color.on_danger = Color::from_u32(0x0001_23d1);
    let expected_danger = custom_theme.color.danger;
    let expected_on_danger = custom_theme.color.on_danger;
    let mut custom = harness(Page::new(Some(5), true, "VeryLongLabel"), 64, custom_theme);
    let target = close_area(&custom);
    let _ = custom.mouse(MouseKind::Move, target.x, target.y);
    assert_eq!(custom.cell(target.x, target.y).fg, expected_danger);
    let _ = custom.mouse(MouseKind::Down, target.x, target.y);
    assert_eq!(custom.cell(target.x, target.y).fg, expected_on_danger);
}

#[test]
fn tab_hover_does_not_activate_close_danger_and_instance_channels_remain_authoritative() {
    let mut danger_only = harness(
        Page::new(Some(3), true, "VeryLongLabel"),
        64,
        Theme::junie(),
    );
    let mut ordinary = harness(
        Page::new(Some(3), false, "VeryLongLabel"),
        64,
        Theme::junie(),
    );
    let tab = tab_area(&danger_only);
    assert!(tab.x < close_area(&danger_only).x);
    let _ = danger_only.mouse(MouseKind::Move, tab.x, tab.y);
    let _ = ordinary.mouse(MouseKind::Move, tab.x, tab.y);
    assert_eq!(danger_only.buffer(), ordinary.buffer());

    let mut page = Page::new(Some(3), true, "VeryLongLabel");
    page.patch = ClosePatch::Set;
    let theme = Theme::junie();
    let expected_fg = theme.color.warning;
    let expected_bg = theme.color.accent_tint;
    let mut h = harness(page, 64, theme);
    let target = close_area(&h);
    let tab = tab_area(&h);
    let outside_plate = tab.x;
    assert!(outside_plate < target.x);
    let _ = h.mouse(MouseKind::Move, outside_plate, tab.y);
    let glyph = tab.right() - 2;
    assert_eq!(h.cell(glyph, tab.y).fg, expected_fg);
    assert_eq!(h.cell(glyph, tab.y).bg, expected_bg);

    let mut global = Page::new(Some(5), true, "VeryLongLabel");
    global.patch = ClosePatch::GlobalSet;
    let mut h = harness(global, 64, Theme::junie());
    let target = close_area(&h);
    let _ = h.mouse(MouseKind::Move, target.x, target.y);
    assert_eq!(h.cell(target.x, target.y).fg, expected_fg);
    assert_eq!(h.cell(target.x, target.y).bg, expected_bg);

    let mut clear = Page::new(Some(5), true, "VeryLongLabel");
    clear.patch = ClosePatch::Clear;
    let mut h = harness(clear, 64, Theme::junie());
    let target = close_area(&h);
    let _ = h.mouse(MouseKind::Move, target.x, target.y);
    for x in target.x..target.right() {
        let cell = h.cell(x, target.y);
        assert_eq!(cell.fg, Color::Reset);
        assert_eq!(cell.bg, Color::Reset);
        assert_eq!(cell.underline_color, Color::Reset);
    }

    let mut global_clear = Page::new(Some(3), true, "VeryLongLabel");
    global_clear.patch = ClosePatch::GlobalClear;
    let mut h = harness(global_clear, 64, Theme::junie());
    let target = close_area(&h);
    let _ = h.mouse(MouseKind::Move, target.x, target.y);
    let glyph = tab_area(&h).right() - 2;
    assert_eq!(h.cell(glyph, target.y).fg, Color::Reset);
    assert_eq!(h.cell(glyph, target.y).bg, Color::Reset);

    let mut reset = Page::new(Some(3), true, "VeryLongLabel");
    reset.patch = ClosePatch::Reset;
    let mut h = harness(reset, 64, Theme::junie());
    let target = close_area(&h);
    let _ = h.mouse(MouseKind::Move, target.x, target.y);
    for x in target.x..target.right() {
        let cell = h.cell(x, target.y);
        assert_eq!(cell.fg, Color::Reset);
        assert_eq!(cell.bg, Color::Reset);
        assert_eq!(cell.underline_color, Color::Reset);
    }

    let mut custom_theme = Theme::junie();
    custom_theme.color.danger = Color::from_u32(0x00d1_2301);
    custom_theme.color.warning = Color::from_u32(0x0001_23d1);
    custom_theme.color.accent_tint = Color::from_u32(0x0001_d123);
    let mut custom = Page::new(Some(5), true, "VeryLongLabel");
    custom.patch = ClosePatch::Set;
    let mut h = harness(custom, 64, custom_theme.clone());
    let target = close_area(&h);
    let _ = h.mouse(MouseKind::Move, target.x, target.y);
    assert_eq!(h.cell(target.x, target.y).fg, custom_theme.color.warning);
    assert_eq!(
        h.cell(target.x, target.y).bg,
        custom_theme.color.accent_tint
    );
    assert_ne!(custom_theme.color.warning, custom_theme.color.danger);
}

#[test]
fn custom_glyph_clear_and_builder_chains_keep_the_fixed_glyph_lane() {
    for order in [false, true] {
        let mut page = Page::new(Some(5), true, "A");
        page.marked = true;
        page.items[0].marked = true;
        page.custom_row = true;
        page.builder_order = order;
        page.patch = ClosePatch::CustomGlyph;
        let mut h = harness(page, 32, Theme::junie());
        let tab = tab_area(&h);
        let target = close_area(&h);
        assert_eq!(h.cell(tab.right() - 2, tab.y).symbol(), "!");
        assert!(target.width >= 1);
        let _ = h.click(target.x, target.y);
        assert_eq!(h.app().actions, [TabsAction::Close(ItemKey::num(41))]);

        let mut clear = Page::new(Some(5), false, "A");
        clear.patch = ClosePatch::ClearGlyph;
        let h = harness(clear, 32, Theme::junie());
        let tab = tab_area(&h);
        assert_eq!(h.cell(tab.right() - 2, tab.y).symbol(), " ");
    }
}

#[test]
fn keyboard_close_eligibility_is_independent_of_target_width() {
    let mut page = Page::new(Some(5), true, "VeryLongLabel");
    page.items.push(Tab {
        id: 42,
        label: "Second",
        marked: false,
    });
    let mut h = harness(page, 64, Theme::junie());
    let _ = h.key(KeyCode::Right);
    h.app_mut().actions.clear();
    let _ = h.key(KeyCode::Char('x'));
    assert_eq!(h.app().actions, [TabsAction::Close(ItemKey::num(42))]);
}

#[test]
fn direct_and_planned_targets_match_and_configuration_mismatch_has_no_effects() {
    let mut direct_page = Page::new(Some(5), true, "VeryLongLabel");
    direct_page.marked = true;
    let mut planned_page = Page::new(Some(5), true, "VeryLongLabel");
    planned_page.marked = true;
    planned_page.planned = true;
    let direct = harness(direct_page, 64, Theme::junie());
    let planned = harness(planned_page, 64, Theme::junie());
    assert_eq!(direct.buffer(), planned.buffer());
    assert_eq!(close_area(&direct), close_area(&planned));
    assert_eq!(tab_area(&direct), tab_area(&planned));

    for (plan_width, draw_width, plan_danger, draw_danger, empty_plan) in [
        (Some(3), Some(5), false, false, false),
        (Some(3), Some(3), false, true, false),
        (None, Some(1), false, false, false),
        (Some(2), Some(1), false, false, false),
        (Some(3), Some(5), false, false, true),
    ] {
        let mut page = Page::new(draw_width, draw_danger, "A");
        page.planned = true;
        page.plan_width = Some(plan_width);
        page.plan_danger = Some(plan_danger);
        page.empty_plan = empty_plan;
        page.sentinel = true;
        let h = harness(page, 32, Theme::junie());
        assert!(h.app().rejected.get());
        for y in 0..3 {
            for x in 0..32 {
                assert_eq!(h.cell(x, y).bg, Color::Blue, "sentinel at ({x},{y})");
            }
        }
        assert!(
            h.area_of_part(TABS, PartRef::item(Part::CLOSE, ItemKey::num(41)))
                .is_none()
        );
    }
}

#[test]
fn target_width_preserves_overflow_new_status_and_clipped_layout_geometry() {
    let labels = [
        "Alpha", "Bravo", "Charlie", "Delta", "Echo", "Foxtrot", "Golf", "Hotel",
    ];
    let items: Vec<_> = labels
        .into_iter()
        .enumerate()
        .map(|(index, label)| Tab {
            id: u64::try_from(index).expect("small fixture index") + 1,
            label,
            marked: false,
        })
        .collect();
    let area = Rect::new(0, 0, 30, 2);
    let previous = Some(termrock_navigation::tabs::TabWindowReceipt {
        viewport_len: 1,
        content_len: items.len(),
    });
    let base = Tabs::new(TABS)
        .closable(true)
        .allow_new(true)
        .status(Status::Busy)
        .key(tab_key as fn(&Tab) -> ItemKey);
    let ordinary = base.layout_default(area, area, &TabsState::default(), &items, previous);
    let widened =
        base.close_target_w(5)
            .layout_default(area, area, &TabsState::default(), &items, previous);

    assert_eq!(ordinary.viewport_len, widened.viewport_len);
    assert_eq!(ordinary.hidden_right(), widened.hidden_right());
    assert_eq!(ordinary.first_index(), widened.first_index());
    assert_eq!(ordinary.overflow_reserved, widened.overflow_reserved);
    assert_eq!(ordinary.rows().len(), widened.rows().len());
    for (before, after) in ordinary.rows().iter().zip(widened.rows()) {
        assert_eq!(before.key, after.key);
        assert_eq!(before.logical_content, after.logical_content);
        assert_eq!(before.logical_tab, after.logical_tab);
        assert_eq!(before.label_extent, after.label_extent);
        assert_eq!(before.logical_close.unwrap().width, 1);
        assert_eq!(after.logical_close.unwrap().width, 5);
    }
    for part in [
        PartRef::item(Part::OVERFLOW, ItemKey::index(0)),
        PartRef::item(Part::OVERFLOW, ItemKey::index(1)),
        PartRef::of(Part::NEW),
        PartRef::of(Part::ICON),
    ] {
        assert_eq!(ordinary.part_bounds(part), widened.part_bounds(part));
    }

    let clipped_area = Rect::new(4, 0, 20, 2);
    let clip = Rect::new(8, 0, 9, 2);
    let clipped = Tabs::new(TABS)
        .closable(true)
        .close_target_w(u16::MAX)
        .layout_default(clipped_area, clip, &TabsState::default(), &items[..1], None);
    let close = clipped.rows()[0].logical_close.expect("eligible target");
    assert_eq!(
        clipped.rows()[0].visible_close,
        Some(close.intersection(clip))
    );
    assert_eq!(close.x, clipped.rows()[0].logical_tab.x);
}
