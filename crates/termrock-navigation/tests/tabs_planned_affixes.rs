use core::cell::{Cell, RefCell};
use core::fmt;

use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::{Color, Modifier};
use termrock_core::{Id, ItemKey, Part, PartRef, Response};
use termrock_navigation::tabs::{TabAffixes, TabLayout, TabMarks, TabWindowReceipt};
use termrock_navigation::{Tabs, TabsAction, TabsState};
use termrock_runtime::keymap::KeyMap;
use termrock_runtime::{App, Cx, FrameRead, RowUi, Ui};
use termrock_test_support::Harness;
use termrock_theme::resolve::bind_role;
use termrock_theme::{ColorLevel, Family, FgStep, Role, StylePatch, Surface, Theme, Variant};

const TABS: Id = Id::root("tabs.planned-affixes");
const MENU: Id = TABS.sub("overflow-menu");
const MENU_ITEMS: Id = MENU.sub("items");
const WIDTH: u16 = 32;
const HEIGHT: u16 = 14;

type TabKey = fn(&AffixTab) -> ItemKey;
type AffixMapper = fn(&AffixTab) -> TabAffixes;
type MarkMapper = fn(&AffixTab) -> TabMarks;
type TabsWidget = Tabs<'static, AffixTab, TabKey>;
type CustomRow = for<'a> fn(&AffixTab, &mut RowUi<'a>);

const META_PATCHES: [(Part, StylePatch); 1] = [(
    Part::META,
    StylePatch::new().set_fg(Role::Info).add(Modifier::BOLD),
)];

struct AffixTab {
    id: u64,
    label: String,
    prefix: Cell<char>,
    wide_suffix: Cell<bool>,
    suppress_suffix: Cell<bool>,
    suppress_affixes: Cell<bool>,
    marks: Cell<TabMarks>,
    display_calls: Cell<usize>,
    affix_calls: Cell<usize>,
    custom_row_calls: Cell<usize>,
}

impl fmt::Display for AffixTab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display_calls
            .set(self.display_calls.get().saturating_add(1));
        f.write_str(&self.label)
    }
}

fn tab_key(tab: &AffixTab) -> ItemKey {
    ItemKey::num(tab.id)
}

fn tab_affixes(tab: &AffixTab) -> TabAffixes {
    tab.affix_calls.set(tab.affix_calls.get().saturating_add(1));
    if tab.suppress_affixes.get() {
        return TabAffixes::default();
    }
    let mut affixes = TabAffixes::default().prefix(format!("{}{}", tab.prefix.get(), tab.id));
    if !tab.suppress_suffix.get() {
        let suffix = if tab.wide_suffix.get() {
            "界".to_owned()
        } else {
            format!("s{}", tab.id)
        };
        affixes = affixes.suffix(suffix);
    }
    affixes
}

fn tab_marks(tab: &AffixTab) -> TabMarks {
    tab.marks.get()
}

fn custom_row(tab: &AffixTab, row: &mut RowUi<'_>) {
    tab.custom_row_calls
        .set(tab.custom_row_calls.get().saturating_add(1));
    row.label(&tab.label);
}

fn planned_tabs(explicit_meta: bool) -> TabsWidget {
    let tabs = Tabs::new(TABS)
        .key(tab_key as TabKey)
        .item_affixes(tab_affixes as AffixMapper)
        .item_marks(tab_marks as MarkMapper)
        .closable(true);
    if explicit_meta {
        tabs.patch_part(&META_PATCHES)
    } else {
        tabs
    }
}

struct Page {
    state: TabsState,
    items: Vec<AffixTab>,
    layouts: RefCell<Vec<TabLayout>>,
    actions: RefCell<Vec<TabsAction>>,
    keymap: KeyMap,
    tabs_area: Rect,
    explicit_meta: bool,
    custom_rows: bool,
    clip_x: u16,
}

impl Page {
    fn new(count: u64, explicit_meta: bool, custom_rows: bool) -> Self {
        let items = (1..=count)
            .map(|id| AffixTab {
                id,
                label: format!("Tab{id}"),
                prefix: Cell::new('p'),
                wide_suffix: Cell::new(false),
                suppress_suffix: Cell::new(false),
                suppress_affixes: Cell::new(false),
                marks: Cell::new(TabMarks::default()),
                display_calls: Cell::new(0),
                affix_calls: Cell::new(0),
                custom_row_calls: Cell::new(0),
            })
            .collect();
        let mut state = TabsState::default();
        if count > 0 {
            state.set_active(0, ItemKey::num(1));
        }
        Self {
            state,
            items,
            layouts: RefCell::new(Vec::new()),
            actions: RefCell::new(Vec::new()),
            keymap: KeyMap::new(),
            tabs_area: Rect::new(0, 0, WIDTH, 2),
            explicit_meta,
            custom_rows,
            clip_x: 0,
        }
    }

    fn custom_widget() -> Tabs<'static, AffixTab, TabKey, CustomRow> {
        Tabs::new(TABS)
            .key(tab_key as TabKey)
            .item_affixes(tab_affixes as AffixMapper)
            .closable(true)
            .row(custom_row as CustomRow)
    }
}

impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = if self.custom_rows {
            Self::custom_widget().update(cx, &mut self.state, &self.items)
        } else {
            planned_tabs(self.explicit_meta).update(cx, &mut self.state, &self.items)
        };
        if let Some(action) = response.take_action() {
            self.actions.borrow_mut().push(action);
        }
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let canvas = ui.full();
        let area = canvas.intersection(self.tabs_area);
        let clip = Rect {
            x: canvas.x.saturating_add(self.clip_x),
            width: canvas.width.saturating_sub(self.clip_x),
            ..canvas
        };
        ui.with_area(clip, |ui| {
            if self.custom_rows {
                Self::custom_widget().draw(ui, area, &self.state, &self.items);
                return;
            }
            let widget = planned_tabs(self.explicit_meta);
            let previous = ui.layout(TABS).map(|facts| TabWindowReceipt {
                viewport_len: facts.viewport_len,
                content_len: facts.content_len,
            });
            let layout = widget.layout_default(area, clip, &self.state, &self.items, previous);
            assert!(widget.draw_layout(ui, &layout).is_ok());
            self.layouts.borrow_mut().push(layout);
        });
    }

    fn keymap(&self) -> &KeyMap {
        &self.keymap
    }
}

fn harness(count: u64, explicit_meta: bool) -> Harness<Page> {
    Harness::new(
        Page::new(count, explicit_meta, false),
        Theme::junie(),
        WIDTH,
        HEIGHT,
    )
}

fn more_part() -> PartRef {
    PartRef::item(Part::OVERFLOW, ItemKey::index(2))
}

fn last_layout(page: &Page) -> TabLayout {
    page.layouts
        .borrow()
        .last()
        .expect("the planned frame was captured")
        .clone()
}

#[test]
fn one_fresh_plan_captures_original_display_and_affixes_once_for_hidden_items() {
    let mut h = harness(12, false);
    let before = h
        .app()
        .items
        .iter()
        .map(|item| (item.display_calls.get(), item.affix_calls.get()))
        .collect::<Vec<_>>();

    h.draw();

    let after = h
        .app()
        .items
        .iter()
        .map(|item| (item.display_calls.get(), item.affix_calls.get()))
        .collect::<Vec<_>>();
    assert!(before.iter().zip(after).all(
        |((display_before, affix_before), (display_after, affix_after))| {
            display_after == display_before + 1 && affix_after == affix_before + 1
        }
    ));

    let layout = last_layout(h.app());
    assert!(layout.hidden_right() > 0, "the plan includes hidden rows");
    assert!(
        layout.rows().len() < h.app().items.len(),
        "only the visible window becomes row geometry"
    );
    assert!(
        h.area_of_part(TABS, more_part()).is_some(),
        "the existing More owner remains live"
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn changed_same_width_affix_updates_visible_debug_geometry_and_fresh_hidden_menu() {
    let mut h = harness(12, false);
    let old = last_layout(h.app());
    let old_debug = format!("{old:?}");
    let prefix = old
        .part_bounds(PartRef::item(Part::PREFIX, ItemKey::num(1)))
        .expect("the first row has a semantic prefix")
        .0;
    assert_eq!(prefix.width, 3, "two prefix cells plus their separator");
    assert_eq!(h.cell(prefix.x, prefix.y).symbol(), "p");

    h.app().items[0].prefix.set('q');
    h.draw();
    let changed = last_layout(h.app());
    assert_ne!(
        old, changed,
        "visible affix text participates in layout equality"
    );
    assert_ne!(old_debug, format!("{changed:?}"));
    let changed_prefix = changed
        .part_bounds(PartRef::item(Part::PREFIX, ItemKey::num(1)))
        .expect("the same stable key keeps its prefix bound")
        .0;
    assert_eq!(
        changed_prefix, prefix,
        "same-width text keeps the exact geometry"
    );
    assert_eq!(h.cell(changed_prefix.x, changed_prefix.y).symbol(), "q");

    let hidden = ItemKey::num(3);
    let _ = h.click_part(TABS, more_part());
    assert!(h.is_open(MENU));
    assert_eq!(h.focus(), Some(MENU_ITEMS));
    assert!(
        h.area_of_part(MENU_ITEMS, PartRef::item(Part::ROW, hidden))
            .is_some()
    );
    assert!(h.text().contains("p3 Tab3 s3"));
    let _ = h.key(termrock_core::KeyCode::Esc);
    assert!(!h.is_open(MENU));

    h.app().items[2].prefix.set('z');
    h.draw();
    let _ = h.click_part(TABS, more_part());
    assert!(h.is_open(MENU));
    assert!(h.text().contains("z3 Tab3 s3"));
    assert_eq!(h.app().state.active(), Some(ItemKey::num(1)));
    assert!(h.app().actions.borrow().is_empty());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn affix_fallback_roles_and_authored_meta_patch_are_capability_aware() {
    for level in [ColorLevel::TrueColor, ColorLevel::Mono] {
        let theme = Theme::junie().for_level(level);
        let expected = [
            (
                ItemKey::num(1),
                bind_role(&theme, Role::Fg(FgStep::Secondary), Surface::Canvas),
            ),
            (
                ItemKey::num(2),
                bind_role(&theme, Role::Fg(FgStep::Muted), Surface::Canvas),
            ),
        ];
        let h = Harness::new(Page::new(2, false, false), theme, 80, HEIGHT);
        let layout = last_layout(h.app());
        for (key, expected_color) in expected {
            let prefix = layout
                .part_bounds(PartRef::item(Part::PREFIX, key))
                .expect("each row owns prefix geometry")
                .0;
            let suffix = layout
                .part_bounds(PartRef::item(Part::META, key))
                .expect("each row owns suffix geometry")
                .0;
            for (x, y) in [(prefix.x, prefix.y), (suffix.x + 1, suffix.y)] {
                let cell = h.cell(x, y);
                assert_eq!(
                    Some(cell.fg),
                    expected_color,
                    "the fallback role follows active state and {level:?}"
                );
                assert!(
                    !cell.modifier.contains(Modifier::BOLD),
                    "affixes remain subordinate to the label by default"
                );
            }
        }
    }

    let theme = Theme::junie();
    let expected_color = bind_role(&theme, Role::Info, Surface::Canvas);
    let explicit = Harness::new(Page::new(2, true, false), theme, 80, HEIGHT);
    let layout = last_layout(explicit.app());
    let suffix = layout
        .part_bounds(PartRef::item(Part::META, ItemKey::num(2)))
        .expect("the explicit suffix has semantic geometry")
        .0;
    let cell = explicit.cell(suffix.x + 1, suffix.y);
    assert_eq!(
        Some(cell.fg),
        expected_color,
        "an authored foreground survives the inactive Muted fallback"
    );
    assert!(
        cell.modifier.contains(Modifier::BOLD),
        "an authored BOLD addition survives the affix default removal"
    );
}

#[test]
fn capability_meta_profiles_keep_canonical_color_and_modifier_ownership() {
    let profile_role = Role::Custom(Color::Rgb(37, 151, 203));
    for (level, profile_color) in [
        (ColorLevel::TrueColor, Color::Rgb(37, 151, 203)),
        (ColorLevel::Ansi256, Color::Indexed(201)),
        (ColorLevel::Ansi16, Color::Red),
        (ColorLevel::Mono, Color::White),
    ] {
        let profile_role = Role::Custom(profile_color);
        let theme = Theme::junie()
            .define_variant_for_level(level, Family::TABS, Variant::DEFAULT, |recipe| {
                recipe
                    .part(Part::META)
                    .base(StylePatch::new().set_fg(profile_role).add(Modifier::BOLD));
            })
            .for_level(level);
        let expected = profile_color;
        let h = Harness::new(Page::new(2, false, false), theme, 80, HEIGHT);
        let layout = last_layout(h.app());

        for key in [ItemKey::num(1), ItemKey::num(2)] {
            let suffix = layout
                .part_bounds(PartRef::item(Part::META, key))
                .expect("each active and inactive row has a suffix allocation")
                .0;
            let cell = h.cell(suffix.x + 1, suffix.y);
            assert_eq!(
                cell.fg, expected,
                "eligible META profile color remains canonical at {level:?}"
            );
            assert!(
                cell.modifier.contains(Modifier::BOLD),
                "authored profile BOLD survives the subordinate affix default at {level:?}"
            );
        }
        assert!(h.text().contains("p1 Tab1 s1"));
        assert!(h.text().contains("p2 Tab2 s2"));
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }

    let fg_only_role = Role::Custom(Color::Rgb(122, 79, 211));
    let theme = Theme::junie()
        .define_variant_for_level(
            ColorLevel::TrueColor,
            Family::TABS,
            Variant::DEFAULT,
            |recipe| {
                recipe
                    .part(Part::META)
                    .base(StylePatch::new().set_fg(fg_only_role));
            },
        )
        .for_level(ColorLevel::TrueColor);
    let expected = bind_role(&theme, fg_only_role, Surface::Canvas);
    let h = Harness::new(Page::new(2, false, false), theme, 80, HEIGHT);
    let layout = last_layout(h.app());
    let suffix = layout
        .part_bounds(PartRef::item(Part::META, ItemKey::num(2)))
        .expect("inactive suffix is present")
        .0;
    let cell = h.cell(suffix.x + 1, suffix.y);
    assert_eq!(Some(cell.fg), expected);
    assert!(
        !cell.modifier.contains(Modifier::BOLD),
        "foreground ownership alone does not suppress the default BOLD removal"
    );

    let theme = Theme::junie()
        .define_variant_for_level(
            ColorLevel::TrueColor,
            Family::TABS,
            Variant::DEFAULT,
            |recipe| {
                recipe
                    .part(Part::META)
                    .base(StylePatch::new().set_fg(profile_role));
            },
        )
        .for_level(ColorLevel::TrueColor);
    let expected = bind_role(&theme, Role::Info, Surface::Canvas);
    let h = Harness::new(Page::new(2, true, false), theme, 80, HEIGHT);
    let layout = last_layout(h.app());
    let suffix = layout
        .part_bounds(PartRef::item(Part::META, ItemKey::num(2)))
        .expect("local override suffix is present")
        .0;
    let cell = h.cell(suffix.x + 1, suffix.y);
    assert_eq!(
        Some(cell.fg),
        expected,
        "local style still wins over a profile"
    );
    assert!(cell.modifier.contains(Modifier::BOLD));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn clipped_affix_bounds_stay_inside_the_label_content_and_match_painted_cells() {
    let mut full = Harness::new(Page::new(4, false, false), Theme::junie(), 80, HEIGHT);
    full.app_mut().tabs_area = Rect::new(0, 0, 80, 2);
    full.draw();
    let mut h = Harness::new(Page::new(4, false, false), Theme::junie(), 80, HEIGHT);
    h.app_mut().tabs_area = Rect::new(0, 0, 80, 2);
    h.app_mut().clip_x = 2;
    h.draw();
    let layout = last_layout(h.app());
    let row = layout
        .rows()
        .iter()
        .find(|row| row.key == ItemKey::num(1))
        .expect("the first stable row remains in the planned window");
    let prefix = layout
        .part_bounds(PartRef::item(Part::PREFIX, ItemKey::num(1)))
        .expect("the clipped prefix retains its semantic part");
    let suffix = layout
        .part_bounds(PartRef::item(Part::META, ItemKey::num(1)))
        .expect("the clipped suffix retains its semantic part");
    assert_eq!(prefix.0.x, row.logical_content.x);
    assert_eq!(prefix.1.x, row.visible_content.x);
    assert!(prefix.0.right() <= row.logical_content.right());
    assert!(suffix.0.right() <= row.logical_content.right());
    assert_eq!(h.cell(prefix.1.x, prefix.1.y).symbol(), "1");
    assert_eq!(h.cell(suffix.1.x + 1, suffix.1.y).symbol(), "s");

    let full_layout = last_layout(full.app());
    assert_eq!(full_layout.rows().len(), layout.rows().len());
    assert!(
        layout.rows().len() >= 2,
        "the comparison includes a later tab"
    );
    for (fitting_row, clipped_row) in full_layout.rows().iter().zip(layout.rows()) {
        assert_eq!(fitting_row.key, clipped_row.key);
        assert_eq!(fitting_row.logical_content, clipped_row.logical_content);
        assert_eq!(fitting_row.logical_tab, clipped_row.logical_tab);
        assert_eq!(fitting_row.logical_close, clipped_row.logical_close);
    }
    for x in row.visible_content.x..row.logical_content.right() {
        let clipped = h.cell(x, row.logical_content.y);
        let fitting = full.cell(x, row.logical_content.y);
        assert_eq!(clipped.symbol(), fitting.symbol(), "logical cell x={x}");
        assert_eq!(clipped.fg, fitting.fg, "foreground x={x}");
        assert_eq!(clipped.bg, fitting.bg, "background x={x}");
        assert_eq!(clipped.modifier, fitting.modifier, "modifiers x={x}");
    }
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn clipped_affix_skips_a_cut_wide_grapheme_without_reflowing_following_text() {
    let mut full = harness(2, false);
    full.app_mut().items[0].prefix.set('界');
    full.draw();
    let mut clipped = harness(2, false);
    clipped.app_mut().items[0].prefix.set('界');
    clipped.app_mut().clip_x = 2;
    clipped.draw();

    let full_layout = last_layout(full.app());
    let clipped_layout = last_layout(clipped.app());
    let full_row = full_layout
        .rows()
        .iter()
        .find(|row| row.key == ItemKey::num(1))
        .expect("the fitting frame contains the keyed row");
    let clipped_row = clipped_layout
        .rows()
        .iter()
        .find(|row| row.key == ItemKey::num(1))
        .expect("the clipped frame contains the keyed row");
    assert_eq!(full_row.logical_content, clipped_row.logical_content);
    let digit_x = clipped_row.logical_content.x.saturating_add(2);
    assert_eq!(
        full.cell(digit_x, clipped_row.logical_content.y).symbol(),
        "1"
    );
    assert_eq!(
        clipped
            .cell(digit_x, clipped_row.logical_content.y)
            .symbol(),
        "1"
    );
    assert_ne!(
        clipped
            .cell(digit_x.saturating_sub(1), clipped_row.logical_content.y)
            .symbol(),
        "界",
        "a clip through the wide glyph leaves it cropped instead of restarting it"
    );
    for x in digit_x..clipped_row.logical_content.right() {
        let clipped_cell = clipped.cell(x, clipped_row.logical_content.y);
        let fitting_cell = full.cell(x, clipped_row.logical_content.y);
        assert_eq!(
            clipped_cell.symbol(),
            fitting_cell.symbol(),
            "logical cell x={x}"
        );
        assert_eq!(clipped_cell.fg, fitting_cell.fg, "foreground x={x}");
        assert_eq!(clipped_cell.bg, fitting_cell.bg, "background x={x}");
        assert_eq!(
            clipped_cell.modifier, fitting_cell.modifier,
            "modifiers x={x}"
        );
    }
    assert!(
        clipped.diagnostics().is_empty(),
        "{:?}",
        clipped.diagnostics()
    );
}

#[test]
fn affixed_logical_extent_counts_trailing_wide_suffix_and_trims_blank_tail() {
    let mut wide = harness(1, false);
    wide.app_mut().items[0].wide_suffix.set(true);
    wide.draw();
    let wide_layout = last_layout(wide.app());
    let wide_row = wide_layout
        .rows()
        .iter()
        .find(|row| row.key == ItemKey::num(1))
        .expect("the planned row is present");
    assert_eq!(wide_row.label_extent, 10);
    assert_eq!(wide_row.logical_tab.width, 14);

    let mut trailing_blank = harness(1, false);
    trailing_blank.app_mut().items[0].label.push(' ');
    trailing_blank.app_mut().items[0].suppress_suffix.set(true);
    trailing_blank.draw();
    let blank_layout = last_layout(trailing_blank.app());
    let blank_row = blank_layout
        .rows()
        .iter()
        .find(|row| row.key == ItemKey::num(1))
        .expect("the prefix-only planned row is present");
    assert_eq!(blank_row.label_extent, 7);
}

#[test]
fn empty_and_unicode_affixes_keep_one_owned_width_and_skip_empty_plans() {
    let mut h = harness(2, false);
    h.app_mut().items[0].prefix.set('界');
    h.app_mut().items[0].label = "e\u{301}界".to_owned();
    h.app_mut().items[1].suppress_affixes.set(true);
    h.draw();

    let layout = last_layout(h.app());
    let unicode_prefix = layout
        .part_bounds(PartRef::item(Part::PREFIX, ItemKey::num(1)))
        .expect("the Unicode prefix has captured semantic geometry")
        .0;
    let unicode_suffix = layout
        .part_bounds(PartRef::item(Part::META, ItemKey::num(1)))
        .expect("the suffix follows the captured Unicode display width")
        .0;
    assert_eq!(unicode_prefix.width, 4, "two-cell prefix, digit, separator");
    assert_eq!(h.cell(unicode_prefix.x, unicode_prefix.y).symbol(), "界");
    assert_eq!(h.cell(unicode_prefix.x + 2, unicode_prefix.y).symbol(), "1");
    assert_eq!(h.cell(unicode_suffix.x + 1, unicode_suffix.y).symbol(), "s");
    assert!(
        layout
            .part_bounds(PartRef::item(Part::PREFIX, ItemKey::num(2)))
            .is_none(),
        "empty prefix text creates no semantic part"
    );
    assert!(
        layout
            .part_bounds(PartRef::item(Part::META, ItemKey::num(2)))
            .is_none(),
        "empty suffix text creates no semantic part"
    );
    assert!(h.text().contains("Tab2"));
    assert!(!h.text().contains("p2 Tab2 s2"));

    let before = h
        .app()
        .items
        .iter()
        .map(|item| (item.display_calls.get(), item.affix_calls.get()))
        .collect::<Vec<_>>();
    h.app_mut().tabs_area = Rect::new(0, 0, 0, 0);
    h.draw();
    let after = h
        .app()
        .items
        .iter()
        .map(|item| (item.display_calls.get(), item.affix_calls.get()))
        .collect::<Vec<_>>();
    assert_eq!(after, before, "an empty allocation invokes neither mapper");
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn affix_geometry_coexists_with_live_mark_and_close_owners() {
    let mut h = harness(2, false);
    h.app_mut().items[0]
        .marks
        .set(TabMarks::default().dirty(true));
    h.draw();

    let key = ItemKey::num(1);
    let layout = last_layout(h.app());
    let row = layout
        .rows()
        .iter()
        .find(|row| row.key == key)
        .expect("the marked row is in the owned plan");
    let prefix = layout
        .part_bounds(PartRef::item(Part::PREFIX, key))
        .expect("the prefix is part of this row's owned allocation")
        .0;
    let suffix = layout
        .part_bounds(PartRef::item(Part::META, key))
        .expect("the suffix is part of this row's owned allocation")
        .0;
    let mark = layout
        .part_bounds(PartRef::item(Part::ICON, key))
        .expect("the dirty mark retains its independent allocation")
        .0;
    let close_bounds = layout
        .part_bounds(PartRef::item(Part::CLOSE, key))
        .expect("the close target retains its independent allocation");
    assert!(prefix.intersection(mark).is_empty());
    assert!(prefix.intersection(close_bounds.0).is_empty());
    assert!(suffix.intersection(mark).is_empty());
    assert!(suffix.intersection(close_bounds.0).is_empty());
    assert_eq!(
        h.area_of_part(TABS, PartRef::item(Part::CLOSE, key)),
        Some(close_bounds.1),
        "the hit registry publishes the plan's clipped close owner"
    );
    assert!(
        row.logical_content
            .contains(Position::new(prefix.x, prefix.y))
    );

    let _ = h.click_part(TABS, PartRef::item(Part::CLOSE, key));
    assert!(
        h.app().actions.borrow().contains(&TabsAction::Close(key)),
        "the live close owner still emits its stable key"
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn custom_row_callbacks_remain_visible_only_and_ignore_planned_affix_mapper() {
    let mut h = Harness::new(Page::new(3, false, true), Theme::junie(), 80, HEIGHT);
    assert!(h.text().contains("Tab1"));
    assert!(!h.text().contains("p1 Tab1"));
    let post_bootstrap_custom_row_calls = h
        .app()
        .items
        .iter()
        .map(|item| {
            assert_eq!(item.affix_calls.get(), 0);
            let calls = item.custom_row_calls.get();
            assert!(calls > 0, "bootstrap completed a custom row paint");
            calls
        })
        .collect::<Vec<_>>();

    h.draw();
    for (item, post_bootstrap_calls) in h.app().items.iter().zip(post_bootstrap_custom_row_calls) {
        assert_eq!(item.affix_calls.get(), 0);
        assert_eq!(item.custom_row_calls.get(), post_bootstrap_calls + 1);
    }
    assert!(h.app().actions.borrow().is_empty());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
