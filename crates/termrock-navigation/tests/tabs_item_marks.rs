use core::fmt;
use core::time::Duration;
use std::sync::atomic::{AtomicUsize, Ordering};

use ratatui_core::layout::Rect;
use ratatui_core::style::Color;
use termrock_core::{Id, ItemKey, Part, PartRef, Response};
use termrock_navigation::{TabMarks, Tabs, TabsAction, TabsState};
use termrock_runtime::rowui::RowUi;
use termrock_runtime::{App, Cx, Moment, Status, Ui};
use termrock_test_support::Harness;
use termrock_theme::{
    ColorLevel, Family, GlyphRole, Modifier, Recipes, Role, Slot, StateFlags, StylePatch, Surface,
    Theme, Variant,
};

const TABS: Id = Id::root("tabs.item-marks");
const AREA: Rect = Rect::new(0, 0, 80, 2);

#[derive(Clone, Copy)]
struct Tab {
    label: &'static str,
    marks: TabMarks,
}

impl fmt::Display for Tab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label)
    }
}

fn item_marks(tab: &Tab) -> TabMarks {
    tab.marks
}

fn counted_marks(tab: &Tab) -> TabMarks {
    MAPPER_CALLS.fetch_add(1, Ordering::SeqCst);
    tab.marks
}

fn always_error(_: &Tab) -> TabMarks {
    TabMarks::default().error(true)
}

static MAPPER_CALLS: AtomicUsize = AtomicUsize::new(0);

struct Page {
    state: TabsState,
    items: Vec<Tab>,
    mapped: bool,
    closable: bool,
    allow_new: bool,
    status: Status,
    close_reveal: termrock_navigation::TabCloseReveal,
    planned: bool,
    actions: Vec<TabsAction>,
}

impl Page {
    fn new(items: Vec<Tab>, mapped: bool, closable: bool, status: Status) -> Self {
        Self {
            state: TabsState::default(),
            items,
            mapped,
            closable,
            allow_new: false,
            status,
            close_reveal: termrock_navigation::TabCloseReveal::Always,
            planned: false,
            actions: Vec::new(),
        }
    }

    fn with_plan(mut self) -> Self {
        self.planned = true;
        self
    }

    fn with_close_reveal(mut self, close_reveal: termrock_navigation::TabCloseReveal) -> Self {
        self.close_reveal = close_reveal;
        self
    }

    fn with_new_button(mut self) -> Self {
        self.allow_new = true;
        self
    }

    fn tabs(&self) -> Tabs<'static, Tab> {
        let tabs = Tabs::new(TABS)
            .allow_new(self.allow_new)
            .closable(self.closable)
            .close_reveal(self.close_reveal)
            .status(self.status);
        if self.mapped {
            tabs.item_marks(item_marks)
        } else {
            tabs
        }
    }
}

impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = self.tabs().update(cx, &mut self.state, &self.items);
        if let Some(action) = response.take_action() {
            self.actions.push(action);
        }
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let tabs = self.tabs();
        let area = ui.full();
        if self.planned {
            let layout = tabs.layout_default(area, area, &self.state, &self.items, None);
            tabs.draw_layout(ui, &layout)
                .expect("a plan built from this frame's Tabs configuration draws");
        } else {
            tabs.draw(ui, area, &self.state, &self.items);
        }
    }
}

type CustomTabs = Tabs<'static, Tab, fn(&Tab) -> ItemKey, for<'a> fn(&Tab, &mut RowUi<'a>)>;

fn stable_key(_: &Tab) -> ItemKey {
    ItemKey::num(41)
}

fn long_custom_row(tab: &Tab, row: &mut RowUi<'_>) {
    row.leading_part(Part::PREFIX, 1, 1, |cell| {
        cell.text(">");
    });
    row.meta("S");
    row.label(tab.label);
}

fn custom_tabs(marks_first: bool) -> CustomTabs {
    let tabs = Tabs::new(TABS).closable(true);
    if marks_first {
        tabs.item_marks(item_marks)
            .key(stable_key as fn(&Tab) -> ItemKey)
            .row(long_custom_row as for<'a> fn(&Tab, &mut RowUi<'a>))
    } else {
        tabs.key(stable_key as fn(&Tab) -> ItemKey)
            .row(long_custom_row as for<'a> fn(&Tab, &mut RowUi<'a>))
            .item_marks(item_marks)
    }
}

struct CustomPage {
    state: TabsState,
    item: Tab,
    marks_first: bool,
    actions: Vec<TabsAction>,
}

impl App for CustomPage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = custom_tabs(self.marks_first).update(cx, &mut self.state, &[self.item]);
        if let Some(action) = response.take_action() {
            self.actions.push(action);
        }
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        custom_tabs(self.marks_first).draw(ui, ui.full(), &self.state, &[self.item]);
    }
}

struct PlannedPage {
    layout: termrock_navigation::tabs::TabLayout,
    items: Vec<Tab>,
    mapper: fn(&Tab) -> TabMarks,
}

impl PlannedPage {
    fn new(items: Vec<Tab>, mapper: fn(&Tab) -> TabMarks) -> Self {
        let layout = make_plan(&items, mapper);
        Self {
            layout,
            items,
            mapper,
        }
    }

    fn rebuild(&mut self) {
        self.layout = make_plan(&self.items, self.mapper);
    }
}

fn make_plan(items: &[Tab], mapper: fn(&Tab) -> TabMarks) -> termrock_navigation::tabs::TabLayout {
    Tabs::new(TABS).item_marks(mapper).layout_default(
        AREA,
        AREA,
        &TabsState::default(),
        items,
        None,
    )
}

impl App for PlannedPage {
    fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Tabs::new(TABS)
            .item_marks(self.mapper)
            .draw_layout(ui, &self.layout)
            .expect("the plan was made for the current component configuration");
    }
}

fn tab_area<A: App>(h: &Harness<A>, key: ItemKey) -> Rect {
    h.area_of_part(TABS, PartRef::item(Part::TAB, key))
        .expect("each visible tab is registered")
}

#[test]
fn canonical_marks_reserve_independent_lanes_and_keep_priority_and_global_status() {
    let items = vec![
        Tab {
            label: "A",
            marks: TabMarks::default().dirty(true),
        },
        Tab {
            label: "B",
            marks: TabMarks::default().busy(true),
        },
        Tab {
            label: "C",
            marks: TabMarks::default().error(true),
        },
        Tab {
            label: "D",
            marks: TabMarks::default().done(true),
        },
        Tab {
            label: "E",
            marks: TabMarks::default()
                .dirty(true)
                .busy(true)
                .error(true)
                .done(true),
        },
        Tab {
            label: "F",
            marks: TabMarks::default(),
        },
    ];
    let mut theme = Theme::junie();
    theme.recipes = Recipes::default();
    let frames = theme.design.motion.spinner_frames;
    assert!(frames.len() > 1);
    let h = Harness::new(
        Page::new(items.clone(), true, true, Status::Error),
        theme.clone(),
        AREA.width,
        AREA.height,
    );
    let baseline = Harness::new(
        Page::new(items.clone(), false, true, Status::Error),
        theme.clone(),
        AREA.width,
        AREA.height,
    );

    let expected = [
        (0, Some(GlyphRole::Dirty), Some(Role::Warning)),
        (1, None, Some(Role::Accent)),
        (2, Some(GlyphRole::Error), Some(Role::Danger)),
        (3, Some(GlyphRole::ProgressDone), Some(Role::Success)),
        (4, None, Some(Role::Accent)),
    ];
    for (index, glyph, role) in expected {
        let tab = tab_area(&h, ItemKey::index(index));
        let mark_cell = tab.x.saturating_add(3);
        let expected_symbol =
            glyph.map_or(frames[0], |glyph| Theme::junie().design.glyphs.get(glyph));
        assert_eq!(h.cell(mark_cell, 0).symbol(), expected_symbol);
        let role = role.expect("every checked item has a mark");
        assert_eq!(
            Some(h.cell(mark_cell, 0).fg),
            termrock_theme::resolve::bind_role(&theme, role, Surface::Canvas)
        );
        assert_eq!(h.cell(mark_cell, 0).bg, h.cell(tab.x, 0).bg);
        let plain_tab = tab_area(&baseline, ItemKey::index(index));
        assert_eq!(
            h.cell(tab.x.saturating_add(1), 0).fg,
            baseline.cell(plain_tab.x.saturating_add(1), 0).fg
        );
        assert!(
            h.area_of_part(TABS, PartRef::item(Part::CLOSE, ItemKey::index(index)))
                .is_some()
        );
        assert_eq!(
            h.area_of_part(TABS, PartRef::item(Part::ICON, ItemKey::index(index))),
            None,
            "a status mark is painted inside the existing tab hit region"
        );
    }
    let clean = tab_area(&h, ItemKey::index(5));
    assert_eq!(
        h.area_of_part(TABS, PartRef::item(Part::ICON, ItemKey::index(5))),
        None
    );
    assert_eq!(h.cell(clean.x.saturating_add(1), 0).symbol(), "F");
    assert_eq!(
        h.cell(AREA.width.saturating_sub(2), 0).symbol(),
        Theme::junie().design.glyphs.get(GlyphRole::Error),
        "the strip-wide readiness ICON lane remains independent"
    );
}

#[test]
fn absent_and_clean_mappers_preserve_cells_hits_and_global_busy_defaults() {
    let items = ["Alpha", "Beta", "Gamma", "Delta"].map(|label| Tab {
        label,
        marks: TabMarks::default(),
    });
    let plain = Harness::new(
        Page::new(items.to_vec(), false, true, Status::Busy).with_new_button(),
        Theme::junie(),
        24,
        2,
    );
    let clean = Harness::new(
        Page::new(items.to_vec(), true, true, Status::Busy).with_new_button(),
        Theme::junie(),
        24,
        2,
    );
    assert_eq!(plain.text(), clean.text());
    for index in 0..items.len() {
        let key = ItemKey::index(index);
        assert_eq!(
            plain.area_of_part(TABS, PartRef::item(Part::TAB, key)),
            clean.area_of_part(TABS, PartRef::item(Part::TAB, key))
        );
        assert_eq!(
            plain.area_of_part(TABS, PartRef::item(Part::CLOSE, key)),
            clean.area_of_part(TABS, PartRef::item(Part::CLOSE, key))
        );
    }
    for (part, key) in [
        (Part::ICON, None),
        (Part::NEW, None),
        (Part::OVERFLOW, Some(ItemKey::index(1))),
    ] {
        let part_ref = key.map_or_else(|| PartRef::of(part), |key| PartRef::item(part, key));
        assert_eq!(
            plain.area_of_part(TABS, part_ref),
            clean.area_of_part(TABS, part_ref),
            "clean mapper preserves the existing {part:?} hit"
        );
    }
    assert!(
        plain
            .area_of_part(TABS, PartRef::item(Part::OVERFLOW, ItemKey::index(1)))
            .is_some()
    );
    assert_eq!(plain.app().state.active(), clean.app().state.active());
    assert_eq!(plain.app().state.cursor(), clean.app().state.cursor());
    assert_eq!(plain.next_deadline(), None);
    assert_eq!(clean.next_deadline(), None);

    for y in 0..2 {
        for x in 0..24 {
            let left = plain.cell(x, y);
            let right = clean.cell(x, y);
            assert_eq!(left.symbol(), right.symbol());
            assert_eq!(left.style(), right.style());
        }
    }
}

#[test]
fn direct_and_planned_default_rows_match_cells_and_existing_hit_bounds() {
    let items = vec![
        Tab {
            label: "A",
            marks: TabMarks::default().dirty(true),
        },
        Tab {
            label: "B",
            marks: TabMarks::default().error(true),
        },
        Tab {
            label: "C",
            marks: TabMarks::default().done(true),
        },
    ];
    let direct = Harness::new(
        Page::new(items.clone(), true, true, Status::Error),
        Theme::junie(),
        48,
        2,
    );
    let planned = Harness::new(
        Page::new(items, true, true, Status::Error).with_plan(),
        Theme::junie(),
        48,
        2,
    );
    assert_eq!(direct.text(), planned.text());
    for y in 0..2 {
        for x in 0..48 {
            assert_eq!(direct.cell(x, y).symbol(), planned.cell(x, y).symbol());
            assert_eq!(direct.cell(x, y).style(), planned.cell(x, y).style());
        }
    }
    for index in 0..3 {
        assert_eq!(
            tab_area(&direct, ItemKey::index(index)),
            tab_area(&planned, ItemKey::index(index))
        );
        assert_eq!(
            direct.area_of_part(TABS, PartRef::item(Part::CLOSE, ItemKey::index(index))),
            planned.area_of_part(TABS, PartRef::item(Part::CLOSE, ItemKey::index(index)))
        );
    }
}

#[test]
fn item_clicks_still_activate_tabs_and_custom_rows_keep_mark_and_close_tail() {
    for marks_first in [true, false] {
        let mut h = Harness::new(
            CustomPage {
                state: TabsState::default(),
                item: Tab {
                    label: "LONG-LONG",
                    marks: TabMarks::default().dirty(true),
                },
                marks_first,
                actions: Vec::new(),
            },
            Theme::junie(),
            20,
            2,
        );
        let tab = tab_area(&h, ItemKey::num(41));
        let close = h
            .area_of_part(TABS, PartRef::item(Part::CLOSE, ItemKey::num(41)))
            .expect("closable custom row registers its close target");
        let icon_x = close.x.saturating_sub(2);
        assert_eq!(h.cell(tab.x.saturating_add(1), 0).symbol(), ">");
        assert_eq!(h.cell(tab.x.saturating_add(3), 0).symbol(), "L");
        assert_eq!(h.cell(close.x.saturating_sub(4), 0).symbol(), "S");
        assert_eq!(
            h.cell(icon_x, 0).symbol(),
            Theme::junie().design.glyphs.get(GlyphRole::Dirty)
        );
        assert!(icon_x > tab.x);
        assert!(close.x > icon_x);
        assert_eq!(close.x.saturating_sub(icon_x), 2);
        assert_eq!(
            h.area_of_part(TABS, PartRef::item(Part::ICON, ItemKey::num(41))),
            None
        );

        let _ = h.click(icon_x, 0);
        assert_eq!(h.app().actions, [TabsAction::Activated(ItemKey::num(41))]);
        h.app_mut().actions.clear();
        let _ = h.click(close.x, 0);
        assert_eq!(h.app().actions, [TabsAction::Close(ItemKey::num(41))]);
    }
}

#[test]
fn marked_hover_close_keeps_its_reserved_tail_and_real_close_eligibility() {
    let key = ItemKey::index(1);
    let mut h = Harness::new(
        Page::new(
            vec![
                Tab {
                    label: "A",
                    marks: TabMarks::default(),
                },
                Tab {
                    label: "B",
                    marks: TabMarks::default().dirty(true),
                },
            ],
            true,
            true,
            Status::Ready,
        )
        .with_close_reveal(termrock_navigation::TabCloseReveal::Hover),
        Theme::junie(),
        20,
        2,
    );
    let tab = tab_area(&h, key);
    assert_eq!(
        h.area_of_part(TABS, PartRef::item(Part::CLOSE, key)),
        None,
        "the inactive hover-close target is not registered"
    );
    let close_x = tab.right().saturating_sub(2);
    let mark_x = close_x.saturating_sub(2);
    let _ = h.mouse(
        termrock_runtime::MouseKind::Move,
        tab.x.saturating_add(1),
        tab.y,
    );
    assert_eq!(
        h.area_of_part(TABS, PartRef::item(Part::CLOSE, key)),
        Some(Rect::new(close_x, tab.y, 1, 1))
    );
    assert_eq!(
        h.cell(mark_x, tab.y).symbol(),
        Theme::junie().design.glyphs.get(GlyphRole::Dirty)
    );
    h.app_mut().actions.clear();
    let _ = h.click_part(TABS, PartRef::item(Part::CLOSE, key));
    assert_eq!(h.app().actions, [TabsAction::Close(key)]);
}

fn wide_mark_slot(ui: &mut Ui<'_>, area: Rect) {
    ui.paint_str(area, "界x", ui.surface_style());
}

struct SlotPage(Page);

impl App for SlotPage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.0.update(cx)
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Tabs::new(TABS)
            .closable(true)
            .item_marks(item_marks)
            .slot(Part::ICON, &wide_mark_slot)
            .draw(ui, ui.full(), &self.0.state, &self.0.items);
    }
}

#[test]
fn wide_default_glyphs_and_custom_icon_slots_stay_inside_the_mark_lane() {
    let wide_theme = Theme::junie()
        .builder()
        .glyph(GlyphRole::Dirty, "界")
        .build();
    let h = Harness::new(
        Page::new(
            vec![
                Tab {
                    label: "A",
                    marks: TabMarks::default().dirty(true),
                },
                Tab {
                    label: "B",
                    marks: TabMarks::default(),
                },
            ],
            true,
            true,
            Status::Ready,
        ),
        wide_theme,
        20,
        2,
    );
    assert_eq!(h.cell(2, 0).symbol(), "界");
    assert_eq!(
        h.cell(5, 0).symbol(),
        Theme::junie().design.glyphs.get(GlyphRole::Close)
    );
    assert_eq!(h.cell(9, 0).symbol(), "B");

    let h = Harness::new(
        SlotPage(Page::new(
            vec![
                Tab {
                    label: "A",
                    marks: TabMarks::default().dirty(true),
                },
                Tab {
                    label: "B",
                    marks: TabMarks::default(),
                },
            ],
            true,
            true,
            Status::Ready,
        )),
        Theme::junie(),
        20,
        2,
    );
    assert_eq!(h.cell(2, 0).symbol(), "界");
    assert_eq!(
        h.cell(5, 0).symbol(),
        Theme::junie().design.glyphs.get(GlyphRole::Close)
    );
    assert_eq!(h.cell(9, 0).symbol(), "B");
    assert_eq!(
        h.area_of_part(TABS, PartRef::item(Part::ICON, ItemKey::index(0))),
        None
    );
}

#[test]
fn reference_busy_marks_are_static_and_suppress_live_registrations_and_deadlines() {
    struct ReferencePage;
    impl App for ReferencePage {
        fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let items = [Tab {
                label: "A",
                marks: TabMarks::default().busy(true),
            }];
            let state = TabsState::default();
            ui.reference(None, |ui| {
                Tabs::new(TABS)
                    .item_marks(item_marks)
                    .draw(ui, ui.full(), &state, &items);
            });
        }
    }

    let h = Harness::new(ReferencePage, Theme::junie(), 20, 2);
    assert_eq!(
        h.cell(3, 0).symbol(),
        Theme::junie().design.motion.spinner_frames[0]
    );
    assert_eq!(h.next_deadline(), None);
    assert_eq!(
        h.area_of_part(TABS, PartRef::item(Part::TAB, ItemKey::index(0))),
        None
    );
    assert_eq!(
        h.area_of_part(TABS, PartRef::item(Part::ICON, ItemKey::index(0))),
        None
    );
}

#[test]
fn narrow_overflow_and_clipped_plans_keep_marks_inside_allocated_geometry() {
    let items = [
        Tab {
            label: "A",
            marks: TabMarks::default().dirty(true),
        },
        Tab {
            label: "B",
            marks: TabMarks::default().error(true),
        },
    ];
    let tabs = Tabs::new(TABS).closable(true).item_marks(item_marks);
    let plan = tabs.layout_default(
        Rect::new(0, 0, 12, 2),
        Rect::new(3, 0, 9, 2),
        &TabsState::default(),
        &items,
        Some(termrock_navigation::tabs::TabWindowReceipt {
            viewport_len: 1,
            content_len: items.len(),
        }),
    );
    assert_eq!(plan.rows().len(), 1);
    assert_eq!(plan.hidden_right(), 1);
    let (logical, visible) = plan
        .part_bounds(PartRef::item(Part::ICON, ItemKey::index(0)))
        .expect("a partially clipped mark retains logical and visible bounds");
    assert_eq!(logical, Rect::new(2, 0, 2, 1));
    assert_eq!(visible, Rect::new(3, 0, 1, 1));
    let close = plan
        .part_bounds(PartRef::item(Part::CLOSE, ItemKey::index(0)))
        .expect("the mark does not consume or overlap the close target")
        .0;
    assert!(logical.right() <= close.x);
    assert!(
        plan.part_bounds(PartRef::item(Part::OVERFLOW, ItemKey::index(1)))
            .is_some()
    );

    let h = Harness::new(
        Page::new(items.to_vec(), true, true, Status::Ready),
        Theme::junie(),
        6,
        2,
    );
    assert_eq!(
        h.area_of_part(TABS, PartRef::item(Part::TAB, ItemKey::index(0))),
        None
    );
    assert_eq!(
        h.area_of_part(TABS, PartRef::item(Part::CLOSE, ItemKey::index(0))),
        None
    );
}

#[test]
fn planned_rows_snapshot_marks_and_geometry_without_rerunning_mapper_during_draw() {
    MAPPER_CALLS.store(0, Ordering::SeqCst);
    let mut h = Harness::new(
        PlannedPage::new(
            vec![Tab {
                label: "A",
                marks: TabMarks::default().dirty(true),
            }],
            counted_marks,
        ),
        Theme::junie(),
        AREA.width,
        AREA.height,
    );
    assert_eq!(MAPPER_CALLS.load(Ordering::SeqCst), 1);
    let bounds = h
        .app()
        .layout
        .part_bounds(PartRef::item(Part::ICON, ItemKey::index(0)))
        .expect("the planned row exposes its logical and clipped mark allocation");
    assert_eq!(bounds.0.width, 2);
    assert_eq!(bounds.1, bounds.0);
    assert_eq!(
        h.cell(bounds.0.x.saturating_add(1), 0).symbol(),
        Theme::junie().design.glyphs.get(GlyphRole::Dirty)
    );
    assert_eq!(MAPPER_CALLS.load(Ordering::SeqCst), 1);
    assert_eq!(
        h.area_of_part(TABS, PartRef::item(Part::ICON, ItemKey::index(0))),
        None,
        "mark bounds do not create a focusable or interactive ICON target"
    );

    h.app_mut().items[0].marks = TabMarks::default().done(true);
    h.app_mut().mapper = always_error;
    h.draw();
    assert_eq!(MAPPER_CALLS.load(Ordering::SeqCst), 1);
    assert_eq!(
        h.cell(bounds.0.x.saturating_add(1), 0).symbol(),
        Theme::junie().design.glyphs.get(GlyphRole::Dirty)
    );

    h.app_mut().rebuild();
    h.draw();
    let rebuilt = h
        .app()
        .layout
        .part_bounds(PartRef::item(Part::ICON, ItemKey::index(0)))
        .expect("the rebuilt plan stores the changed mark");
    assert_eq!(rebuilt.0, bounds.0);
    assert_eq!(
        h.cell(rebuilt.0.x.saturating_add(1), 0).symbol(),
        Theme::junie().design.glyphs.get(GlyphRole::Error)
    );
}

#[test]
fn semantic_defaults_survive_capability_downgrade_and_custom_icon_channels_win() {
    let cases = [
        (
            TabMarks::default().busy(true),
            None,
            Role::Accent,
            StateFlags::BUSY,
        ),
        (
            TabMarks::default().error(true),
            Some(GlyphRole::Error),
            Role::Danger,
            StateFlags::ERROR,
        ),
        (
            TabMarks::default().dirty(true),
            Some(GlyphRole::Dirty),
            Role::Warning,
            StateFlags::DIRTY,
        ),
        (
            TabMarks::default().done(true),
            Some(GlyphRole::ProgressDone),
            Role::Success,
            StateFlags::CHECKED,
        ),
    ];
    for (theme_name, base_theme) in [("Junie", Theme::junie()), ("Paper", Theme::paper())] {
        for level in [
            ColorLevel::TrueColor,
            ColorLevel::Ansi256,
            ColorLevel::Ansi16,
            ColorLevel::Mono,
        ] {
            let mut theme = base_theme.clone().downgrade(level);
            theme.recipes = Recipes::default();
            let items = cases
                .iter()
                .enumerate()
                .map(|(index, (marks, _, _, _))| Tab {
                    label: match index {
                        0 => "A",
                        1 => "B",
                        2 => "C",
                        _ => "D",
                    },
                    marks: *marks,
                })
                .collect();
            let h = Harness::new(
                Page::new(items, true, false, Status::Ready),
                theme.clone(),
                48,
                2,
            );
            for (index, (marks, glyph, role, _)) in cases.iter().enumerate() {
                let tab = tab_area(&h, ItemKey::index(index));
                let mark_x = tab.x.saturating_add(3);
                if let Some(glyph) = glyph {
                    assert_eq!(h.cell(mark_x, 0).symbol(), theme.design.glyphs.get(*glyph));
                } else {
                    assert_eq!(
                        h.cell(mark_x, 0).symbol(),
                        theme.design.motion.spinner_frames[0]
                    );
                }
                assert_eq!(
                    Some(h.cell(mark_x, 0).fg),
                    termrock_theme::resolve::bind_role(&theme, *role, Surface::Canvas),
                    "semantic default ink for {marks:?} at {theme_name}/{level:?}"
                );
                assert_eq!(h.cell(mark_x, 0).bg, h.cell(tab.x, 0).bg);
            }
            let done_tab = tab_area(&h, ItemKey::index(3));
            assert!(
                !h.cell(done_tab.x.saturating_add(3), 0)
                    .modifier
                    .contains(Modifier::BOLD)
            );
        }
    }

    let theme = Theme::junie();
    let items = cases
        .iter()
        .enumerate()
        .map(|(index, (marks, _, _, _))| Tab {
            label: match index {
                0 => "A",
                1 => "B",
                2 => "C",
                _ => "D",
            },
            marks: *marks,
        })
        .collect();
    let h = Harness::new(
        Page::new(items, true, false, Status::Ready),
        theme.clone(),
        48,
        2,
    );
    for (index, (_, _, _, flags)) in cases.iter().enumerate() {
        let tab = tab_area(&h, ItemKey::index(index));
        assert_eq!(
            Some(h.cell(tab.x.saturating_add(3), 0).fg),
            Some(
                theme
                    .resolve(
                        Family::TABS,
                        Variant::DEFAULT,
                        Part::ICON,
                        *flags,
                        Surface::Canvas,
                    )
                    .style
                    .fg
                    .unwrap_or(Color::Reset)
            ),
            "the authored TABS/ICON recipe remains above low-priority mark defaults"
        );
    }

    let mut custom_recipe_theme = Theme::junie();
    custom_recipe_theme
        .recipes
        .get_mut(Family::TABS)
        .parts
        .entry(Part::ICON)
        .base = StylePatch::new()
        .set_fg(Role::Custom(Color::Rgb(193, 17, 61)))
        .set_glyph(GlyphRole::WarningMark);
    let h = Harness::new(
        Page::new(
            vec![Tab {
                label: "A",
                marks: TabMarks::default().error(true),
            }],
            true,
            false,
            Status::Ready,
        ),
        custom_recipe_theme,
        20,
        2,
    );
    let tab = tab_area(&h, ItemKey::index(0));
    assert_eq!(
        h.cell(tab.x.saturating_add(3), 0).symbol(),
        Theme::junie().design.glyphs.get(GlyphRole::WarningMark),
        "consumer-authored TABS/ICON recipe glyph wins over mark defaults"
    );
    assert_eq!(
        h.cell(tab.x.saturating_add(3), 0).fg,
        Color::Rgb(193, 17, 61),
        "consumer-authored TABS/ICON recipe ink wins over the semantic fallback"
    );

    const GLOBAL_ICON: StylePatch = StylePatch::new().set_bg(Role::Custom(Color::Rgb(23, 35, 47)));
    const CUSTOM_ICON: [(Part, StylePatch); 1] = [(
        Part::ICON,
        StylePatch::new()
            .set_fg(Role::Custom(Color::Rgb(201, 4, 83)))
            .set_bg(Role::Custom(Color::Rgb(7, 31, 67)))
            .add(Modifier::UNDERLINED)
            .set_glyph(GlyphRole::WarningMark),
    )];
    struct StyledPage(Page);
    impl App for StyledPage {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            self.0.update(cx)
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            Tabs::new(TABS)
                .item_marks(item_marks)
                .patch(&GLOBAL_ICON)
                .patch_part(&CUSTOM_ICON)
                .draw(ui, ui.full(), &self.0.state, &self.0.items);
        }
    }
    let h = Harness::new(
        StyledPage(Page::new(
            vec![Tab {
                label: "A",
                marks: TabMarks::default().dirty(true),
            }],
            true,
            false,
            Status::Ready,
        )),
        Theme::junie(),
        20,
        2,
    );
    let mark = tab_area(&h, ItemKey::index(0)).x.saturating_add(3);
    assert_eq!(
        h.cell(mark, 0).symbol(),
        Theme::junie().design.glyphs.get(GlyphRole::WarningMark)
    );
    assert_eq!(h.cell(mark, 0).fg, Color::Rgb(201, 4, 83));
    assert_eq!(h.cell(mark, 0).bg, Color::Rgb(7, 31, 67));
    assert!(h.cell(mark, 0).modifier.contains(Modifier::UNDERLINED));
    assert_eq!(h.cell(1, 0).bg, Color::Rgb(23, 35, 47));
    assert!(!h.cell(1, 0).modifier.contains(Modifier::UNDERLINED));

    const CLEAR_ICON: [(Part, StylePatch); 1] = [(
        Part::ICON,
        StylePatch {
            glyph: Slot::Clear,
            ..StylePatch::new()
        },
    )];
    struct ClearedPage(Page);
    impl App for ClearedPage {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            self.0.update(cx)
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            Tabs::new(TABS)
                .item_marks(item_marks)
                .patch_part(&CLEAR_ICON)
                .draw(ui, ui.full(), &self.0.state, &self.0.items);
        }
    }
    let h = Harness::new(
        ClearedPage(Page::new(
            vec![Tab {
                label: "A",
                marks: TabMarks::default().dirty(true),
            }],
            true,
            false,
            Status::Ready,
        )),
        Theme::junie(),
        20,
        2,
    );
    let tab = tab_area(&h, ItemKey::index(0));
    assert_eq!(h.cell(tab.x.saturating_add(3), 0).symbol(), " ");
    assert_eq!(
        h.cell(tab.x.saturating_add(3), 0).bg,
        h.cell(tab.x, 0).bg,
        "a cleared glyph keeps the tab-plane background"
    );

    const RESET_ICON: [(Part, StylePatch); 1] = [(
        Part::ICON,
        StylePatch::new()
            .set_fg(Role::Custom(Color::Reset))
            .set_bg(Role::Custom(Color::Reset)),
    )];
    struct ResetPage(Page);
    impl App for ResetPage {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            self.0.update(cx)
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            Tabs::new(TABS)
                .item_marks(item_marks)
                .patch_part(&RESET_ICON)
                .draw(ui, ui.full(), &self.0.state, &self.0.items);
        }
    }
    let h = Harness::new(
        ResetPage(Page::new(
            vec![Tab {
                label: "A",
                marks: TabMarks::default().dirty(true),
            }],
            true,
            false,
            Status::Ready,
        )),
        Theme::junie(),
        20,
        2,
    );
    let tab = tab_area(&h, ItemKey::index(0));
    assert_eq!(
        h.cell(tab.x.saturating_add(3), 0).symbol(),
        Theme::junie().design.glyphs.get(GlyphRole::Dirty),
        "an inherited glyph keeps the default mark while explicit Reset colors apply"
    );
    assert_eq!(h.cell(tab.x.saturating_add(3), 0).fg, Color::Reset);
    assert_eq!(h.cell(tab.x.saturating_add(3), 0).bg, Color::Reset);

    const BOLD_DONE: [(Part, StylePatch); 1] =
        [(Part::ICON, StylePatch::new().add(Modifier::BOLD))];
    struct BoldDonePage(Page);
    impl App for BoldDonePage {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            self.0.update(cx)
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            Tabs::new(TABS)
                .item_marks(item_marks)
                .patch_part(&BOLD_DONE)
                .draw(ui, ui.full(), &self.0.state, &self.0.items);
        }
    }
    let h = Harness::new(
        BoldDonePage(Page::new(
            vec![Tab {
                label: "A",
                marks: TabMarks::default().done(true),
            }],
            true,
            false,
            Status::Ready,
        )),
        Theme::junie(),
        20,
        2,
    );
    let mark = tab_area(&h, ItemKey::index(0)).x.saturating_add(3);
    assert!(h.cell(mark, 0).modifier.contains(Modifier::BOLD));
}

#[test]
fn busy_marks_use_configured_clock_and_empty_or_single_frame_lists_do_not_loop() {
    static CUSTOM_FRAMES: &[&str] = &["<", "=", "+"];
    const CUSTOM_TICK_MS: u64 = 25;

    let item = Tab {
        label: "A",
        marks: TabMarks::default().busy(true),
    };
    let mut theme = Theme::junie();
    theme.design.motion.spinner_frames = CUSTOM_FRAMES;
    theme.design.motion.tick_ms = CUSTOM_TICK_MS;
    let frames = theme.design.motion.spinner_frames;
    assert!(frames.len() > 1);
    let tick = theme.design.motion.tick_ms;
    let mut h = Harness::new(
        Page::new(vec![item], true, false, Status::Ready),
        theme,
        20,
        2,
    );
    let mark_x = tab_area(&h, ItemKey::index(0)).x.saturating_add(3);
    assert_eq!(h.cell(mark_x, 0).symbol(), "<");
    assert_eq!(h.next_deadline(), Some(Moment::from_millis(tick)));
    let _ = h.advance(Duration::from_millis(tick));
    assert_eq!(h.cell(mark_x, 0).symbol(), "=");
    assert_eq!(
        h.next_deadline(),
        Some(Moment::from_millis(tick.saturating_mul(2)))
    );
    let _ = h.advance(Duration::from_millis(tick));
    assert_eq!(h.cell(mark_x, 0).symbol(), "+");
    assert_eq!(
        h.next_deadline(),
        Some(Moment::from_millis(tick.saturating_mul(3)))
    );
    let _ = h.advance(Duration::from_millis(tick));
    assert_eq!(h.cell(mark_x, 0).symbol(), "<");
    assert_eq!(
        h.next_deadline(),
        Some(Moment::from_millis(tick.saturating_mul(4)))
    );

    static NO_FRAMES: &[&str] = &[];
    static ONE_FRAME: &[&str] = &["*"];
    for (spinner_frames, expected) in [(NO_FRAMES, " "), (ONE_FRAME, "*")] {
        let mut theme = Theme::junie();
        theme.design.motion.spinner_frames = spinner_frames;
        theme.design.motion.tick_ms = 0;
        let h = Harness::new(
            Page::new(vec![item], true, false, Status::Ready),
            theme,
            20,
            2,
        );
        let mark_x = tab_area(&h, ItemKey::index(0)).x.saturating_add(3);
        assert_eq!(h.cell(mark_x, 0).symbol(), expected);
        assert_eq!(h.next_deadline(), None);
    }
}

#[test]
fn clean_and_busy_statuses_without_opted_in_items_schedule_no_item_animation() {
    for (mapped, marks, status) in [
        (false, TabMarks::default().busy(true), Status::Ready),
        (true, TabMarks::default(), Status::Ready),
        (false, TabMarks::default(), Status::Busy),
    ] {
        let h = Harness::new(
            Page::new(vec![Tab { label: "A", marks }], mapped, false, status),
            Theme::junie(),
            20,
            2,
        );
        assert_eq!(h.next_deadline(), None);
    }
}
