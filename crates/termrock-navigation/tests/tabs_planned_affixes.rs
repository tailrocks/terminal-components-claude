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
    trailing_padding: Cell<u16>,
    plate_separators: Cell<bool>,
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
    let mut affixes = if tab.suppress_affixes.get() {
        TabAffixes::default()
    } else {
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
    };
    if tab.trailing_padding.get() > 0 {
        affixes = affixes.trailing_padding(tab.trailing_padding.get());
    }
    if tab.plate_separators.get() {
        affixes = affixes.plate_separators(true);
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
    reuse_last_layout: Cell<bool>,
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
                trailing_padding: Cell::new(0),
                plate_separators: Cell::new(false),
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
            reuse_last_layout: Cell::new(false),
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
            if self.reuse_last_layout.get() {
                let layouts = self.layouts.borrow();
                let Some(layout) = layouts.last() else {
                    return;
                };
                assert!(widget.draw_layout(ui, layout).is_ok());
                return;
            }
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

fn harness_rect_text(h: &Harness<Page>, area: Rect) -> String {
    let mut text = String::new();
    for x in area.x..area.right() {
        text.push_str(
            h.buffer()
                .cell((x, area.y))
                .expect("part lies in the frame")
                .symbol(),
        );
    }
    text
}

#[test]
fn trailing_padding_keeps_default_debug_and_custom_row_behavior() {
    assert_eq!(
        format!("{:?}", TabAffixes::default()),
        "TabAffixes { prefix: None, suffix: None }"
    );
    let padding_only = TabAffixes::default().trailing_padding(1);
    assert_eq!(
        format!("{padding_only:?}"),
        "TabAffixes { prefix: None, suffix: None, trailing_padding: 1 }"
    );
    assert_ne!(padding_only, TabAffixes::default());
    assert_eq!(
        TabAffixes::default().trailing_padding(0),
        TabAffixes::default()
    );

    let mut baseline = Harness::new(Page::new(1, false, true), Theme::junie(), WIDTH, HEIGHT);
    let mut padded = Harness::new(Page::new(1, false, true), Theme::junie(), WIDTH, HEIGHT);
    padded.app().items[0].trailing_padding.set(3);
    let baseline_post_bootstrap_calls = baseline.app().items[0].custom_row_calls.get();
    let padded_post_bootstrap_calls = padded.app().items[0].custom_row_calls.get();
    assert!(baseline_post_bootstrap_calls > 0);
    assert!(padded_post_bootstrap_calls > 0);
    baseline.draw();
    padded.draw();
    assert_eq!(baseline.text(), padded.text());
    assert_eq!(
        baseline.app().items[0].custom_row_calls.get(),
        baseline_post_bootstrap_calls + 1
    );
    assert_eq!(
        padded.app().items[0].custom_row_calls.get(),
        padded_post_bootstrap_calls + 1
    );
    assert_eq!(baseline.app().items[0].affix_calls.get(), 0);
    assert_eq!(padded.app().items[0].affix_calls.get(), 0);
    assert!(baseline.diagnostics().is_empty());
    assert!(padded.diagnostics().is_empty());
}

#[test]
fn plate_separators_are_opt_in_and_change_only_separator_paint() {
    let default = TabAffixes::default();
    let opt_in = default.clone().plate_separators(true);
    assert_eq!(
        format!("{default:?}"),
        "TabAffixes { prefix: None, suffix: None }"
    );
    assert_eq!(
        format!("{opt_in:?}"),
        "TabAffixes { prefix: None, suffix: None, plate_separators: true }"
    );
    assert_eq!(default.clone().plate_separators(false), default);
    assert_ne!(opt_in, default);

    let mut default_harness = harness(12, false);
    let mut plate_harness = harness(12, false);
    for item in &default_harness.app().items {
        item.trailing_padding.set(1);
    }
    for item in &plate_harness.app().items {
        item.trailing_padding.set(1);
        item.plate_separators.set(true);
    }
    default_harness.draw();
    plate_harness.draw();

    assert_eq!(default_harness.text(), plate_harness.text());
    assert!(
        plate_harness.text().contains("Tab1"),
        "the opt-in preserves the original Display text"
    );
    let default_layout = last_layout(default_harness.app());
    let plate_layout = last_layout(plate_harness.app());
    assert_eq!(default_layout.rows().len(), plate_layout.rows().len());
    let mut checked_prefix_separator = false;
    let mut checked_suffix_separator = false;
    for (default_row, plate_row) in default_layout.rows().iter().zip(plate_layout.rows()) {
        assert_eq!(default_row.key, plate_row.key);
        assert_eq!(default_row.logical_tab, plate_row.logical_tab);
        assert_eq!(default_row.visible_tab, plate_row.visible_tab);
        assert_eq!(default_row.logical_content, plate_row.logical_content);
        assert_eq!(default_row.label_extent, plate_row.label_extent);
        for part in [Part::PREFIX, Part::META] {
            let reference = PartRef::item(part, default_row.key);
            assert_eq!(
                default_layout.part_bounds(reference),
                plate_layout.part_bounds(reference),
                "the opt-in does not change {part:?} geometry"
            );
        }

        let (prefix, prefix_visible) = plate_layout
            .part_bounds(PartRef::item(Part::PREFIX, plate_row.key))
            .expect("the row has a semantic prefix");
        let (suffix, suffix_visible) = plate_layout
            .part_bounds(PartRef::item(Part::META, plate_row.key))
            .expect("the row has a semantic suffix");
        let mut separator_cells = Vec::with_capacity(2);
        if prefix_visible.is_empty() {
            assert!(prefix.intersection(plate_row.visible_tab).is_empty());
        } else {
            checked_prefix_separator = true;
            separator_cells.push((
                prefix_visible
                    .x
                    .saturating_add(prefix_visible.width.saturating_sub(1)),
                prefix_visible.y,
            ));
        }
        if suffix_visible.is_empty() {
            assert!(suffix.intersection(plate_row.visible_tab).is_empty());
        } else {
            checked_suffix_separator = true;
            separator_cells.push((suffix_visible.x, suffix_visible.y));
        }
        let mut text_cells = Vec::with_capacity(2);
        if prefix_visible.width > 1 {
            text_cells.push((prefix_visible.x, prefix_visible.y));
        }
        if suffix_visible.width > 1 {
            text_cells.push((suffix_visible.x.saturating_add(1), suffix_visible.y));
        }
        let plate_x = plate_row.logical_tab.right().saturating_sub(3);
        let plate_cell = plate_harness.cell(plate_x, plate_row.logical_tab.y);
        for (x, y) in separator_cells {
            let default_cell = default_harness.cell(x, y);
            let plate_separator = plate_harness.cell(x, y);
            assert_eq!(default_cell.symbol(), " ");
            assert_eq!(plate_separator.symbol(), " ");
            assert_eq!(plate_separator.fg, plate_cell.fg);
            assert_eq!(plate_separator.bg, plate_cell.bg);
            assert_eq!(
                plate_separator.modifier, plate_cell.modifier,
                "row={plate_row:?} prefix={prefix:?} suffix={suffix:?} tab={:?} content={:?} separator=({x},{y}) cell={plate_separator:?}; sampled plate=({plate_x},{}) cell={plate_cell:?}; default={default_cell:?}",
                plate_row.logical_tab, plate_row.logical_content, plate_row.logical_tab.y,
            );
            assert_ne!(
                default_cell.fg, plate_separator.fg,
                "the default semantic style still owns this separator"
            );
        }
        for (x, y) in text_cells {
            let default_cell = default_harness.cell(x, y);
            let plate_text = plate_harness.cell(x, y);
            assert_eq!(default_cell.symbol(), plate_text.symbol());
            assert_eq!(default_cell.fg, plate_text.fg);
            assert_eq!(default_cell.bg, plate_text.bg);
            assert_eq!(default_cell.modifier, plate_text.modifier);
        }
    }
    assert!(
        checked_prefix_separator,
        "a visible prefix separator is checked"
    );
    assert!(
        checked_suffix_separator,
        "a visible suffix separator is checked"
    );

    let _ = default_harness.click_part(TABS, more_part());
    let _ = plate_harness.click_part(TABS, more_part());
    assert!(default_harness.is_open(MENU));
    assert!(plate_harness.is_open(MENU));
    let hidden = PartRef::item(Part::ROW, ItemKey::num(3));
    let default_hidden = default_harness
        .area_of_part(MENU_ITEMS, hidden)
        .expect("the hidden default row is in More");
    let plate_hidden = plate_harness
        .area_of_part(MENU_ITEMS, hidden)
        .expect("the hidden opt-in row is in More");
    assert_eq!(
        harness_rect_text(&default_harness, default_hidden),
        harness_rect_text(&plate_harness, plate_hidden)
    );
    assert!(default_harness.diagnostics().is_empty());
    assert!(plate_harness.diagnostics().is_empty());
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
fn plate_separator_opt_in_is_a_noop_without_affixes_or_padding() {
    let mut baseline = harness(2, false);
    let mut opted_in = harness(2, false);
    for item in &baseline.app().items {
        item.suppress_affixes.set(true);
    }
    for item in &opted_in.app().items {
        item.suppress_affixes.set(true);
        item.plate_separators.set(true);
    }

    baseline.draw();
    opted_in.draw();

    assert_eq!(baseline.buffer(), opted_in.buffer());
    assert_eq!(baseline.text(), opted_in.text());
    let baseline_layout = last_layout(baseline.app());
    let opted_in_layout = last_layout(opted_in.app());
    assert_eq!(baseline_layout.rows().len(), opted_in_layout.rows().len());
    for (baseline_row, opted_in_row) in baseline_layout.rows().iter().zip(opted_in_layout.rows()) {
        assert_eq!(baseline_row.key, opted_in_row.key);
        assert_eq!(baseline_row.logical_tab, opted_in_row.logical_tab);
        assert_eq!(baseline_row.visible_tab, opted_in_row.visible_tab);
        assert_eq!(baseline_row.logical_content, opted_in_row.logical_content);
        assert_eq!(baseline_row.label_extent, opted_in_row.label_extent);
    }
    for (baseline_item, opted_in_item) in baseline.app().items.iter().zip(&opted_in.app().items) {
        assert_eq!(
            baseline_item.display_calls.get(),
            opted_in_item.display_calls.get()
        );
        assert_eq!(
            baseline_item.affix_calls.get(),
            opted_in_item.affix_calls.get()
        );
        assert_eq!(
            baseline_item.custom_row_calls.get(),
            opted_in_item.custom_row_calls.get()
        );
    }
    assert!(baseline.diagnostics().is_empty());
    assert!(opted_in.diagnostics().is_empty());
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
fn trailing_plate_padding_preserves_semantic_positions_and_cell_style() {
    let mut base = harness(2, false);
    let mut padded_active = harness(2, false);
    padded_active.app().items[0].trailing_padding.set(1);
    base.draw();
    padded_active.draw();

    let base_layout = last_layout(base.app());
    let padded_layout = last_layout(padded_active.app());
    let base_active = base_layout
        .rows()
        .iter()
        .find(|row| row.key == ItemKey::num(1))
        .expect("the active row is planned");
    let padded_active_row = padded_layout
        .rows()
        .iter()
        .find(|row| row.key == ItemKey::num(1))
        .expect("the padded active row is planned");
    assert_eq!(
        base_active.logical_content.x,
        padded_active_row.logical_content.x
    );
    assert_eq!(base_active.label_extent, padded_active_row.label_extent);
    assert_eq!(
        base_layout
            .part_bounds(PartRef::item(Part::PREFIX, ItemKey::num(1)))
            .map(|bounds| bounds.0),
        padded_layout
            .part_bounds(PartRef::item(Part::PREFIX, ItemKey::num(1)))
            .map(|bounds| bounds.0)
    );
    assert_eq!(
        base_layout
            .part_bounds(PartRef::item(Part::META, ItemKey::num(1)))
            .map(|bounds| bounds.0),
        padded_layout
            .part_bounds(PartRef::item(Part::META, ItemKey::num(1)))
            .map(|bounds| bounds.0)
    );
    assert_eq!(
        padded_active_row.logical_tab.width,
        base_active.logical_tab.width + 1
    );
    assert_eq!(
        padded_layout
            .rows()
            .iter()
            .find(|row| row.key == ItemKey::num(2))
            .expect("the later tab remains visible")
            .logical_tab
            .x,
        base_layout
            .rows()
            .iter()
            .find(|row| row.key == ItemKey::num(2))
            .expect("the later tab remains visible")
            .logical_tab
            .x
            + 1,
        "only later tab geometry advances by the extra plate cell"
    );

    let active_pad = padded_active.cell(
        padded_active_row.logical_tab.right().saturating_sub(3),
        padded_active_row.logical_tab.y,
    );
    let active_pad_x = padded_active_row.logical_tab.right().saturating_sub(3);
    assert_eq!(active_pad.symbol(), " ");
    let active_plate = padded_active.cell(
        padded_active_row.logical_tab.x,
        padded_active_row.logical_tab.y,
    );
    let active_prefix = padded_layout
        .part_bounds(PartRef::item(Part::PREFIX, ItemKey::num(1)))
        .expect("the active row has a semantic prefix")
        .0;
    let active_suffix = padded_layout
        .part_bounds(PartRef::item(Part::META, ItemKey::num(1)))
        .expect("the active row has a semantic suffix")
        .0;
    let close_area = padded_active.area_of_part(TABS, PartRef::item(Part::CLOSE, ItemKey::num(1)));
    let nearby_cells = (active_pad_x.saturating_sub(1)..padded_active_row.logical_tab.right())
        .map(|x| {
            format!(
                "{x}: {:?}",
                padded_active.cell(x, padded_active_row.logical_tab.y)
            )
        })
        .collect::<Vec<_>>();
    let previous_close_cell = base.cell(active_pad_x, base_active.logical_tab.y);
    assert_eq!(
        padded_active.area_of_part(TABS, PartRef::item(Part::TAB, ItemKey::num(1)),),
        Some(padded_active_row.visible_tab),
        "the native hit owner includes the plate padding"
    );
    assert_eq!(active_pad.fg, active_plate.fg);
    assert_eq!(active_pad.bg, active_plate.bg);
    assert_eq!(
        active_pad.modifier, active_plate.modifier,
        "row={padded_active_row:?} prefix={active_prefix:?} suffix={active_suffix:?} close_owner={close_area:?} pad=({active_pad_x},{}) cell={active_pad:?}; leading plate cell={active_plate:?}; previous same-coordinate cell={previous_close_cell:?}; nearby={nearby_cells:?}",
        padded_active_row.logical_tab.y,
    );
    let active_rule = padded_active.cell(
        padded_active_row.logical_tab.x,
        padded_active_row.logical_tab.y.saturating_add(1),
    );
    let active_pad_rule = padded_active.cell(
        active_pad_x,
        padded_active_row.logical_tab.y.saturating_add(1),
    );
    assert_eq!(
        active_pad_rule.fg, active_rule.fg,
        "the active rule spans the plate padding"
    );
    assert_eq!(active_pad_rule.bg, active_rule.bg);
    assert_eq!(active_pad_rule.modifier, active_rule.modifier);

    let mut padded_inactive = harness(2, false);
    padded_inactive.app().items[1].trailing_padding.set(1);
    padded_inactive.draw();
    let layout = last_layout(padded_inactive.app());
    let inactive = layout
        .rows()
        .iter()
        .find(|row| row.key == ItemKey::num(2))
        .expect("the inactive padded row is planned");
    let inactive_pad = padded_inactive.cell(
        inactive.logical_tab.right().saturating_sub(3),
        inactive.logical_tab.y,
    );
    assert_eq!(inactive_pad.symbol(), " ");
    let inactive_plate = padded_inactive.cell(inactive.logical_tab.x, inactive.logical_tab.y);
    assert_eq!(inactive_pad.fg, inactive_plate.fg);
    assert_eq!(inactive_pad.bg, inactive_plate.bg);
    assert_eq!(inactive_pad.modifier, inactive_plate.modifier);
    let inactive_rule = padded_inactive.cell(inactive.logical_tab.x, inactive.logical_tab.y + 1);
    let inactive_pad_rule = padded_inactive.cell(
        inactive.logical_tab.right().saturating_sub(3),
        inactive.logical_tab.y + 1,
    );
    assert_eq!(inactive_pad_rule.fg, inactive_rule.fg);
    assert_eq!(inactive_pad_rule.bg, inactive_rule.bg);
    assert_eq!(inactive_pad_rule.modifier, inactive_rule.modifier);
    assert!(padded_active.diagnostics().is_empty());
    assert!(padded_inactive.diagnostics().is_empty());
}

#[test]
fn trailing_padding_is_captured_once_and_never_enters_more_text() {
    let mut h = harness(12, false);
    h.app().items[0].trailing_padding.set(1);
    h.app().items[2].trailing_padding.set(1);
    h.draw();
    let planned = last_layout(h.app());
    let planned_row = planned
        .rows()
        .iter()
        .find(|row| row.key == ItemKey::num(1))
        .expect("the active padded row is visible");
    assert_eq!(planned_row.logical_tab.width, 15);
    let calls = h.app().items[0].affix_calls.get();
    let frozen = planned.clone();

    h.app().items[0].trailing_padding.set(3);
    h.app().reuse_last_layout.set(true);
    h.draw();
    assert_eq!(planned, last_layout(h.app()));
    assert_eq!(h.app().items[0].affix_calls.get(), calls);
    assert_eq!(planned, frozen);

    h.app().reuse_last_layout.set(false);
    h.draw();
    assert_ne!(planned, last_layout(h.app()));
    assert_eq!(h.app().items[0].affix_calls.get(), calls + 1);

    let mut plain_menu = harness(12, false);
    let mut padded_menu = harness(12, false);
    padded_menu.app().items[2].trailing_padding.set(1);
    plain_menu.draw();
    padded_menu.draw();
    let _ = plain_menu.click_part(TABS, more_part());
    let _ = padded_menu.click_part(TABS, more_part());
    assert!(plain_menu.is_open(MENU));
    assert!(padded_menu.is_open(MENU));
    let hidden = PartRef::item(Part::ROW, ItemKey::num(3));
    let plain_row = plain_menu
        .area_of_part(MENU_ITEMS, hidden)
        .expect("the plain hidden row has a menu owner");
    let padded_row = padded_menu
        .area_of_part(MENU_ITEMS, hidden)
        .expect("the padded hidden row has a menu owner");
    assert_eq!(
        harness_rect_text(&plain_menu, plain_row),
        harness_rect_text(&padded_menu, padded_row)
    );
    assert!(plain_menu.diagnostics().is_empty());
    assert!(padded_menu.diagnostics().is_empty());
}

#[test]
fn padding_only_rows_and_extreme_clipping_keep_geometry_bounded() {
    let mut plain = harness(1, false);
    plain.app().items[0].suppress_affixes.set(true);
    plain.draw();
    let plain_row = last_layout(plain.app()).rows()[0].clone();

    let mut padding_only = harness(1, false);
    padding_only.app().items[0].suppress_affixes.set(true);
    padding_only.app().items[0].trailing_padding.set(2);
    padding_only.draw();
    let padding_layout = last_layout(padding_only.app());
    let padding_row = padding_layout.rows()[0].clone();
    assert_eq!(
        padding_row.logical_tab.width,
        plain_row.logical_tab.width + 2
    );
    assert_eq!(padding_row.label_extent, plain_row.label_extent);
    assert!(
        padding_layout
            .part_bounds(PartRef::item(Part::PREFIX, ItemKey::num(1)))
            .is_none()
    );
    assert!(
        padding_layout
            .part_bounds(PartRef::item(Part::META, ItemKey::num(1)))
            .is_none()
    );

    let mut clipped = Harness::new(Page::new(2, false, false), Theme::junie(), WIDTH, HEIGHT);
    clipped.app().items[0].trailing_padding.set(u16::MAX);
    clipped.app_mut().tabs_area = Rect::new(0, 0, 12, 2);
    clipped.draw();
    let layout = last_layout(clipped.app());
    for row in layout.rows() {
        assert!(
            Rect::new(0, 0, 12, 2).contains(Position::new(row.logical_tab.x, row.logical_tab.y))
        );
        assert!(row.logical_tab.right() <= 12);
        assert!(row.visible_tab.right() <= 12);
    }
    assert!(
        clipped.diagnostics().is_empty(),
        "{:?}",
        clipped.diagnostics()
    );
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
    h.app().items[0].trailing_padding.set(1);
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
    let plate_padding_x = row.logical_tab.right().saturating_sub(3);
    let plate_padding = Rect::new(plate_padding_x, row.logical_tab.y, 1, 1);
    assert!(prefix.intersection(mark).is_empty());
    assert!(prefix.intersection(close_bounds.0).is_empty());
    assert!(suffix.intersection(mark).is_empty());
    assert!(suffix.intersection(close_bounds.0).is_empty());
    assert!(mark.right() <= plate_padding_x);
    assert!(plate_padding.intersection(close_bounds.0).is_empty());
    assert!(
        row.logical_tab
            .contains(Position::new(plate_padding_x, row.logical_tab.y))
    );
    assert_eq!(
        h.area_of_part(TABS, PartRef::item(Part::CLOSE, key)),
        Some(close_bounds.1),
        "the hit registry publishes the plan's clipped close owner"
    );
    assert_eq!(h.cell(plate_padding_x, row.logical_tab.y).symbol(), " ");
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
