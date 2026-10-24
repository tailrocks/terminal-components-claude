use core::fmt;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use ratatui::crossterm::event::{Event, MouseButton, MouseEvent, MouseEventKind};
use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Modifier, Style};
use termrock_controls::Button;
use termrock_core::event::{KeyModifiers, RoutedInput};
use termrock_core::response::Response;
use termrock_core::{Axis, Id, ItemKey, KeyCode, MouseKind, Part, PartRef, StateFlags};
use termrock_navigation::tabs::TabWindowReceipt;
use termrock_navigation::{TabMarks, TabRule, Tabs, TabsAction, TabsState};
use termrock_runtime::rowui::RowUi;
use termrock_runtime::{
    Anchor, App, Axes, Cx, Dismiss, FrameRead, LayerId, LayerSize, LayerSpec, Position,
    ReferenceState, ReferenceTarget, RegionKind, Status, Ui,
};
use termrock_test_support::Harness;
use termrock_theme::resolve::bind_role;
use termrock_theme::{ColorLevel, Family, GlyphRole, Role, StylePatch, Surface, Theme};

const TABS: Id = Id::root("tabs.rule-policy");
const OTHER: Id = Id::root("tabs.rule-policy.other");
const CONTEXT_TABS: Id = Id::root("tabs.rule-policy.context");
const CONTEXT_OTHER: Id = Id::root("tabs.rule-policy.context.other");
const CONTEXT_POPOVER: Id = Id::root("tabs.rule-policy.context.popover");
const TABS_AREA: Rect = Rect::new(0, 0, 48, 2);
const CONTEXT_AREA: Rect = Rect::new(0, 0, 48, 2);
const CONTEXT_REFERENCE_AREA: Rect = Rect::new(0, 2, 48, 2);
const LEVELS: [ColorLevel; 4] = [
    ColorLevel::TrueColor,
    ColorLevel::Ansi256,
    ColorLevel::Ansi16,
    ColorLevel::Mono,
];

#[derive(Clone, Copy)]
struct Tab {
    id: u64,
    label: &'static str,
    marks: TabMarks,
}

impl fmt::Display for Tab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label)
    }
}

fn key(tab: &Tab) -> ItemKey {
    ItemKey::num(tab.id)
}

fn marks(tab: &Tab) -> TabMarks {
    tab.marks
}

fn clean_marks(_: &Tab) -> TabMarks {
    TabMarks::default()
}

fn custom_row(tab: &Tab, row: &mut RowUi<'_>) {
    row.leading_part(Part::PREFIX, 1, 1, |cell| {
        cell.text("›");
    });
    row.label(tab.label);
}

const GLOBAL_RULE: StylePatch = StylePatch::new()
    .set_fg(Role::Info)
    .set_bg(Role::Custom(Color::DarkGray))
    .add(Modifier::BOLD);
const PART_RULE: [(Part, StylePatch); 1] = [(
    Part::RULE,
    StylePatch::new()
        .set_fg(Role::Success)
        .set_bg(Role::Custom(Color::Blue))
        .set_underline(Role::Custom(Color::Yellow))
        .remove(Modifier::BOLD),
)];
const PART_RULE_RESET: [(Part, StylePatch); 1] = [(Part::RULE, StylePatch::new().clear_fg())];

#[derive(Clone, Copy, Debug)]
enum MarkMode {
    None,
    Clean,
    Authored,
}

#[derive(Clone, Copy)]
enum RulePatch {
    None,
    Global,
    GlobalAndPart,
    Reset,
}

#[derive(Clone)]
struct Page {
    state: TabsState,
    items: Vec<Tab>,
    rule: Option<TabRule>,
    rule_focus_hidden: Option<bool>,
    planned: bool,
    planned_wheel_area: Cell<Option<Rect>>,
    marks: MarkMode,
    status: Status,
    close_reveal: termrock_navigation::TabCloseReveal,
    patch: RulePatch,
    reference: Option<ReferenceState>,
    open_other_focus: bool,
    actions: Vec<TabsAction>,
}

impl Page {
    fn new(rule: Option<TabRule>) -> Self {
        let mut state = TabsState::default();
        state.set_active(0, ItemKey::num(1));
        Self {
            state,
            items: vec![
                Tab {
                    id: 1,
                    label: "Alpha",
                    marks: TabMarks::default().dirty(true),
                },
                Tab {
                    id: 2,
                    label: "Beta",
                    marks: TabMarks::default().busy(true),
                },
            ],
            rule,
            rule_focus_hidden: None,
            planned: false,
            planned_wheel_area: Cell::new(None),
            marks: MarkMode::None,
            status: Status::Ready,
            close_reveal: termrock_navigation::TabCloseReveal::Always,
            patch: RulePatch::None,
            reference: None,
            open_other_focus: false,
            actions: Vec::new(),
        }
    }

    fn tabs(&self) -> Tabs<'static, Tab, fn(&Tab) -> ItemKey> {
        let base = Tabs::new(TABS)
            .key(key as fn(&Tab) -> ItemKey)
            .closable(true)
            .allow_new(true)
            .middle_click_closes(true)
            .close_reveal(self.close_reveal)
            .status(self.status);
        let base = match self.rule {
            Some(rule) => base.rule(rule),
            None => base,
        };
        let base = match self.rule_focus_hidden {
            Some(hidden) => base.rule_focus_hidden(hidden),
            None => base,
        };
        let base = match self.marks {
            MarkMode::None => base,
            MarkMode::Clean => base.item_marks(clean_marks),
            MarkMode::Authored => base.item_marks(marks),
        };
        match self.patch {
            RulePatch::None => base,
            RulePatch::Global => base.patch(&GLOBAL_RULE),
            RulePatch::GlobalAndPart => base.patch(&GLOBAL_RULE).patch_part(&PART_RULE),
            RulePatch::Reset => base.patch_part(&PART_RULE_RESET),
        }
    }
}

impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = self.tabs().update(cx, &mut self.state, &self.items);
        if let Some(action) = response.take_action() {
            self.actions.push(action);
        }
        let mut response = response.erase();
        if self.open_other_focus {
            response |= Button::new(OTHER, "Other").update(cx).erase();
        }
        response
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let tabs = self.tabs();
        let draw_tabs = |ui: &mut Ui<'_>| {
            if self.planned {
                let area = TABS_AREA;
                let previous = ui.layout(TABS).map(|facts| TabWindowReceipt {
                    viewport_len: facts.viewport_len,
                    content_len: facts.content_len,
                });
                let layout = tabs.layout_default(area, area, &self.state, &self.items, previous);
                let wheel_area = (!layout.visible_area.is_empty()
                    && layout.viewport_len > 0
                    && layout.content_len > layout.viewport_len)
                    .then_some(layout.visible_area);
                self.planned_wheel_area.set(wheel_area);
                tabs.draw_layout(ui, &layout)
                    .expect("a rule policy is part of the planned rendering configuration");
            } else {
                tabs.draw(ui, TABS_AREA, &self.state, &self.items);
            }
        };
        if let Some(state) = self.reference {
            ui.reference(
                Some(ReferenceTarget::new(TABS, state).part(PartRef::of(Part::RULE))),
                draw_tabs,
            );
        } else {
            draw_tabs(ui);
        }
        if self.open_other_focus {
            Button::new(OTHER, "Other").draw(ui, Rect::new(0, 4, 16, 1));
        }
    }
}

fn harness(page: Page, theme: Theme) -> Harness<Page> {
    Harness::new(page, theme, 64, 5)
}

fn tab_area(h: &Harness<Page>, id: u64) -> Rect {
    h.area_of_part(TABS, PartRef::item(Part::TAB, ItemKey::num(id)))
        .expect("visible tabs register their item area")
}

fn expected_role(theme: &Theme, role: Role) -> Color {
    bind_role(theme, role, Surface::Canvas).unwrap_or(Color::Reset)
}

fn distinct_rule_theme(level: ColorLevel) -> Theme {
    let mut theme = Theme::junie().for_level(level);
    theme.color.accent = Color::Red;
    theme.color.border_strong = Color::Blue;
    theme
}

fn assert_non_foreground_channels_equal(
    actual: &ratatui_core::buffer::Cell,
    expected: &ratatui_core::buffer::Cell,
    context: &str,
) {
    assert_eq!(actual.symbol(), expected.symbol(), "{context}: symbol");
    assert_eq!(actual.bg, expected.bg, "{context}: background");
    assert_eq!(
        actual.underline_color, expected.underline_color,
        "{context}: underline"
    );
    assert_eq!(actual.modifier, expected.modifier, "{context}: modifiers");
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RuleContextReading {
    context: &'static str,
    covered: bool,
    inert: bool,
    owner_focused: bool,
    state_focused: bool,
    focus_visible: bool,
}

struct ContextPage {
    state: TabsState,
    items: [Tab; 1],
    rule: TabRule,
    focus_hidden: Option<bool>,
    planned: bool,
    popover_open: bool,
    declare_focused: bool,
    reference: Option<ReferenceState>,
    readings: Rc<RefCell<Vec<RuleContextReading>>>,
    actions: Vec<TabsAction>,
}

impl ContextPage {
    fn new(rule: TabRule, popover_open: bool, reference: Option<ReferenceState>) -> Self {
        let mut state = TabsState::default();
        state.set_active(0, ItemKey::num(1));
        Self {
            state,
            items: [Tab {
                id: 1,
                label: "Context",
                marks: TabMarks::default(),
            }],
            rule,
            focus_hidden: None,
            planned: false,
            popover_open,
            declare_focused: false,
            reference,
            readings: Rc::new(RefCell::new(Vec::new())),
            actions: Vec::new(),
        }
    }

    fn tabs(&self) -> Tabs<'static, Tab, fn(&Tab) -> ItemKey> {
        let mut tabs = Tabs::new(CONTEXT_TABS)
            .key(key as fn(&Tab) -> ItemKey)
            .rule(self.rule);
        if let Some(hidden) = self.focus_hidden {
            tabs = tabs.rule_focus_hidden(hidden);
        }
        tabs
    }

    fn observe(&self, context: &'static str, ui: &Ui<'_>) {
        let state = ui.state(CONTEXT_TABS);
        self.readings.borrow_mut().push(RuleContextReading {
            context,
            covered: ui.is_covered_by_native_layer(),
            inert: ui.is_inert(),
            owner_focused: ui.owner_has_focus_presentation(CONTEXT_TABS),
            state_focused: state.contains(StateFlags::FOCUSED),
            focus_visible: state.contains(StateFlags::FOCUS_VISIBLE),
        });
    }

    fn draw_tabs(&self, ui: &mut Ui<'_>, area: Rect) {
        let tabs = self.tabs();
        if self.planned {
            let layout = tabs.layout_default(area, area, &self.state, &self.items, None);
            tabs.draw_layout(ui, &layout)
                .expect("context page plans and draws with the same rule configuration");
        } else {
            tabs.draw(ui, area, &self.state, &self.items);
        }
    }
}

impl App for ContextPage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if self.popover_open && !cx.is_open(CONTEXT_POPOVER) {
            cx.open_layer(
                CONTEXT_POPOVER,
                LayerSpec::popover(CONTEXT_POPOVER, Anchor::Point(Position::new(56, 5)))
                    .size(LayerSize::Fixed(12, 3))
                    .dismiss(Dismiss::NONE),
            );
        } else if !self.popover_open && cx.is_open(CONTEXT_POPOVER) {
            cx.close_layer(CONTEXT_POPOVER, None);
        }

        let mut response = self.tabs().update(cx, &mut self.state, &self.items);
        if let Some(action) = response.take_action() {
            self.actions.push(action);
        }
        let mut response = response.erase();
        response |= Button::new(CONTEXT_OTHER, "Other").update(cx).erase();
        response
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.readings.borrow_mut().clear();
        if self.declare_focused {
            ui.declare_state(CONTEXT_TABS, StateFlags::FOCUSED);
        }
        self.observe("live", ui);
        self.draw_tabs(ui, CONTEXT_AREA);
        if let Some(reference) = self.reference {
            ui.reference(
                Some(ReferenceTarget::new(CONTEXT_TABS, reference).part(PartRef::of(Part::RULE))),
                |reference_ui| {
                    self.observe("reference", reference_ui);
                    self.draw_tabs(reference_ui, CONTEXT_REFERENCE_AREA);
                },
            );
        }
        Button::new(CONTEXT_OTHER, "Other").draw(ui, Rect::new(0, 6, 20, 1));
        if self.popover_open {
            let _ = ui.layer(CONTEXT_POPOVER, |popover, area| {
                Button::new(CONTEXT_POPOVER, "Popover").draw(popover, area);
            });
        }
    }
}

fn context_harness(rule: TabRule, theme: Theme, covered: bool) -> Harness<ContextPage> {
    Harness::new(ContextPage::new(rule, covered, None), theme, 72, 8)
}

fn context_rule_fg(harness: &Harness<ContextPage>, y: u16) -> Color {
    harness.cell(0, y).fg
}

fn context_reading(page: &ContextPage, context: &str) -> RuleContextReading {
    page.readings
        .borrow()
        .iter()
        .copied()
        .find(|reading| reading.context == context)
        .unwrap_or_else(|| panic!("missing {context} context reading"))
}

fn assert_same_regions(left: &Harness<ContextPage>, right: &Harness<ContextPage>) {
    let left = left.runtime().registry().regions();
    let right = right.runtime().registry().regions();
    assert_eq!(left.len(), right.len());
    for (left, right) in left.iter().zip(right) {
        assert_eq!(left.id, right.id);
        assert_eq!(left.owner, right.owner);
        assert_eq!(left.part, right.part);
        assert_eq!(left.area, right.area);
        assert_eq!(left.layer, right.layer);
        assert_eq!(left.kind, right.kind);
        assert_eq!(left.scroll(), right.scroll());
        assert_eq!(left.pointer_enabled(), right.pointer_enabled());
        assert_eq!(left.pass_through(), right.pass_through());
    }
}

fn assert_legacy_regions_match_with_planned_scroll(
    legacy: &Harness<Page>,
    planned: &Harness<Page>,
) {
    let legacy_generation = legacy.runtime().registry().generation();
    let planned_generation = planned.runtime().registry().generation();
    let mut legacy_regions = legacy.runtime().registry().regions().to_vec();
    let mut planned_regions = planned.runtime().registry().regions().to_vec();
    assert!(
        legacy_regions
            .iter()
            .all(|region| region.generation == legacy_generation)
    );
    assert!(
        planned_regions
            .iter()
            .all(|region| region.generation == planned_generation)
    );

    let expected_area = planned.app().planned_wheel_area.get();
    let planned_scrolls = planned_regions
        .iter()
        .filter(|region| region.kind == RegionKind::Scroll)
        .count();
    assert_eq!(
        planned_scrolls,
        if expected_area.is_some() { 1 } else { 0 },
        "planned scroll registration matches the overflowing plan"
    );
    assert!(
        legacy_regions
            .iter()
            .all(|region| region.kind != RegionKind::Scroll),
        "legacy rendering has no planned scroll registration"
    );
    if let Some(area) = expected_area {
        let scroll = planned_regions
            .first()
            .copied()
            .expect("the planned scroll registration is the leading region");
        assert_eq!(scroll.kind, RegionKind::Scroll);
        assert_eq!(scroll.id, TABS);
        assert_eq!(scroll.owner, TABS);
        assert_eq!(scroll.part, PartRef::of(Part::CONTAINER));
        assert_eq!(scroll.layer, LayerId::PAGE);
        assert_eq!(scroll.area, area);
        assert_eq!(
            scroll.scroll().map(|(axes, _)| axes),
            Some(Axes::Both),
            "planned registration handles both wheel axes"
        );
        let point = Position::new(area.x, area.y);
        for axis in [Axis::H, Axis::V] {
            let hit = planned
                .runtime()
                .registry()
                .hit_scroll(point, axis)
                .expect("the planned scroll area resolves for both axes");
            assert_eq!(hit.owner, TABS);
            assert_eq!(hit.part, PartRef::of(Part::CONTAINER));
            assert_eq!(hit.layer, LayerId::PAGE);
            assert_eq!(hit.kind, RegionKind::Scroll);
        }
        planned_regions.remove(0);
    }
    for region in &mut legacy_regions {
        region.generation = 0;
    }
    for region in &mut planned_regions {
        region.generation = 0;
    }
    assert_eq!(
        planned_regions, legacy_regions,
        "all existing full Region records remain equal in order"
    );
}

#[test]
fn fixed_rule_roles_follow_theme_tokens_and_change_only_active_foreground() {
    for (theme_name, base) in [("Junie", Theme::junie()), ("Paper", Theme::paper())] {
        for level in LEVELS {
            let theme = base.clone().for_level(level);
            for rule in [
                TabRule::Accent,
                TabRule::Neutral,
                TabRule::Auto,
                TabRule::FocusOnly,
            ] {
                let mut original = harness(Page::new(None), theme.clone());
                let mut configured = harness(Page::new(Some(rule)), theme.clone());
                let beta = tab_area(&configured, 2);
                let _ = configured.mouse(MouseKind::Down, beta.x.saturating_add(1), beta.y);
                let _ = original.mouse(MouseKind::Down, beta.x.saturating_add(1), beta.y);

                assert_eq!(original.app().state.active(), Some(ItemKey::num(1)));
                assert_eq!(configured.app().state.active(), Some(ItemKey::num(1)));
                assert_eq!(original.app().state.cursor(), Some(ItemKey::num(2)));
                assert_eq!(configured.app().state.cursor(), Some(ItemKey::num(2)));
                assert_eq!(
                    original.runtime().registry().regions(),
                    configured.runtime().registry().regions(),
                    "{theme_name} {level:?} {rule:?}: rule policy must not change hit regions"
                );

                let active = tab_area(&configured, 1);
                let role = match rule {
                    TabRule::Accent => Role::Accent,
                    TabRule::Neutral => Role::BorderStrong,
                    TabRule::Auto | TabRule::FocusOnly => Role::Accent,
                    _ => unreachable!("future rule policies are outside this fixture"),
                };
                let expected = expected_role(&theme, role);
                for y in 0..5 {
                    for x in 0..64 {
                        let old = original.cell(x, y);
                        let new = configured.cell(x, y);
                        if y == 1 && (active.x..active.right()).contains(&x) {
                            assert_eq!(
                                new.fg, expected,
                                "{theme_name} {level:?} {rule:?} at ({x},{y})"
                            );
                            assert_non_foreground_channels_equal(new, old, "active rule");
                        } else {
                            assert_eq!(new, old, "{theme_name} {level:?} {rule:?} at ({x},{y})");
                        }
                    }
                }
                assert_eq!(
                    configured.cell(active.x, 1).symbol(),
                    theme.design.glyphs.get(GlyphRole::RuleActive),
                    "the semantic active glyph is retained"
                );
            }
        }
    }
}

#[test]
fn auto_and_focus_only_follow_live_focus_and_native_coverage_at_every_color_tier() {
    for level in LEVELS {
        let theme = distinct_rule_theme(level);
        let accent = expected_role(&theme, Role::Accent);
        let neutral = expected_role(&theme, Role::BorderStrong);
        assert_ne!(
            accent, neutral,
            "custom RULE roles must stay distinguishable"
        );

        let mut auto = context_harness(TabRule::Auto, theme.clone(), false);
        assert_eq!(
            context_rule_fg(&auto, 1),
            accent,
            "Auto is accented while uncovered even before focus is inspected"
        );
        assert!(!context_reading(auto.app(), "live").covered);
        assert!(auto.tab_to(CONTEXT_OTHER));
        assert_eq!(
            context_rule_fg(&auto, 1),
            accent,
            "Auto ignores owner focus"
        );

        let mut focus_only = context_harness(TabRule::FocusOnly, theme.clone(), false);
        assert_eq!(focus_only.focus(), Some(CONTEXT_TABS));
        assert_eq!(context_rule_fg(&focus_only, 1), accent);
        assert!(focus_only.tab_to(CONTEXT_OTHER));
        assert_eq!(context_rule_fg(&focus_only, 1), neutral);
        let reading = context_reading(focus_only.app(), "live");
        assert!(!reading.owner_focused);

        assert!(focus_only.tab_to(CONTEXT_TABS));
        assert_eq!(context_rule_fg(&focus_only, 1), accent);
        let reading = context_reading(focus_only.app(), "live");
        assert!(reading.owner_focused);
        assert!(reading.state_focused);

        let mut pointer_focus = context_harness(TabRule::FocusOnly, theme.clone(), false);
        assert!(pointer_focus.tab_to(CONTEXT_OTHER));
        assert_eq!(pointer_focus.focus(), Some(CONTEXT_OTHER));
        let _ = pointer_focus.mouse(MouseKind::Down, 1, 0);
        assert_eq!(pointer_focus.focus(), Some(CONTEXT_TABS));
        assert_eq!(context_rule_fg(&pointer_focus, 1), accent);
        let reading = context_reading(pointer_focus.app(), "live");
        assert!(reading.owner_focused);
        assert!(
            !reading.focus_visible,
            "pointer-established owner focus does not require the keyboard ring"
        );

        let mut declared_only = context_harness(TabRule::FocusOnly, theme.clone(), false);
        assert!(declared_only.tab_to(CONTEXT_OTHER));
        declared_only.app_mut().declare_focused = true;
        let _ = declared_only.key(KeyCode::F(12));
        let reading = context_reading(declared_only.app(), "live");
        assert!(reading.state_focused, "the fixture declares FOCUSED");
        assert!(!reading.owner_focused, "declared state is not owner focus");
        assert_eq!(context_rule_fg(&declared_only, 1), neutral);

        let mut covered = context_harness(TabRule::Auto, theme.clone(), false);
        assert_eq!(context_rule_fg(&covered, 1), accent);
        covered.app_mut().popover_open = true;
        let _ = covered.key(KeyCode::F(12));
        assert!(covered.is_open(CONTEXT_POPOVER));
        let reading = context_reading(covered.app(), "live");
        assert!(reading.covered);
        assert!(!reading.inert, "the native Popover does not inert the page");
        assert_eq!(context_rule_fg(&covered, 1), neutral);
        covered.app_mut().popover_open = false;
        let _ = covered.key(KeyCode::F(12));
        assert!(!covered.is_open(CONTEXT_POPOVER));
        assert!(!context_reading(covered.app(), "live").covered);
        assert_eq!(context_rule_fg(&covered, 1), accent);

        let mut covered_focused = context_harness(TabRule::FocusOnly, theme.clone(), false);
        assert_eq!(covered_focused.focus(), Some(CONTEXT_TABS));
        covered_focused.app_mut().popover_open = true;
        let _ = covered_focused.key(KeyCode::F(12));
        assert!(covered_focused.is_open(CONTEXT_POPOVER));
        let reading = context_reading(covered_focused.app(), "live");
        assert!(reading.covered);
        assert!(reading.owner_focused, "nonmodal Popover retains page focus");
        assert_eq!(context_rule_fg(&covered_focused, 1), neutral);

        let mut detached_auto =
            ContextPage::new(TabRule::Auto, true, Some(ReferenceState::default()));
        detached_auto.planned = true;
        let detached_auto = Harness::new(detached_auto, theme.clone(), 72, 8);
        let live = context_reading(detached_auto.app(), "live");
        let reference = context_reading(detached_auto.app(), "reference");
        assert!(live.covered && !live.inert);
        assert!(!reference.covered && reference.inert);
        assert_eq!(context_rule_fg(&detached_auto, 1), neutral);
        assert_eq!(context_rule_fg(&detached_auto, 3), accent);

        let mut detached_focus =
            ContextPage::new(TabRule::FocusOnly, true, Some(ReferenceState::FOCUSED));
        detached_focus.planned = true;
        let detached_focus = Harness::new(detached_focus, theme.clone(), 72, 8);
        let reference = context_reading(detached_focus.app(), "reference");
        assert!(!reference.covered && reference.inert);
        assert!(reference.owner_focused && reference.state_focused);
        assert_eq!(context_rule_fg(&detached_focus, 3), accent);

        let mut detached_unfocused =
            ContextPage::new(TabRule::FocusOnly, true, Some(ReferenceState::default()));
        detached_unfocused.planned = true;
        let detached_unfocused = Harness::new(detached_unfocused, theme.clone(), 72, 8);
        let reference = context_reading(detached_unfocused.app(), "reference");
        assert!(!reference.covered && reference.inert);
        assert!(!reference.owner_focused);
        assert_eq!(context_rule_fg(&detached_unfocused, 3), neutral);
    }
}

#[test]
fn explicit_host_hidden_override_changes_only_dynamic_rule_presentation() {
    let theme = distinct_rule_theme(ColorLevel::TrueColor);
    let accent = expected_role(&theme, Role::Accent);
    let neutral = expected_role(&theme, Role::BorderStrong);

    let mut host_hidden = ContextPage::new(TabRule::Auto, false, None);
    host_hidden.focus_hidden = Some(true);
    let host_hidden = Harness::new(host_hidden, theme.clone(), 72, 8);
    assert!(!context_reading(host_hidden.app(), "live").covered);
    assert_eq!(context_rule_fg(&host_hidden, 1), neutral);

    let mut host_visible = ContextPage::new(TabRule::Auto, true, None);
    host_visible.focus_hidden = Some(false);
    let mut host_visible = Harness::new(host_visible, theme.clone(), 72, 8);
    let reading = context_reading(host_visible.app(), "live");
    assert!(reading.covered && !reading.inert);
    assert_eq!(context_rule_fg(&host_visible, 1), accent);

    let mut host_hidden_focus = ContextPage::new(TabRule::FocusOnly, false, None);
    host_hidden_focus.focus_hidden = Some(true);
    let host_hidden_focus = Harness::new(host_hidden_focus, theme.clone(), 72, 8);
    assert_eq!(host_hidden_focus.focus(), Some(CONTEXT_TABS));
    assert!(context_reading(host_hidden_focus.app(), "live").owner_focused);
    assert_eq!(context_rule_fg(&host_hidden_focus, 1), neutral);

    let mut host_visible_focus = ContextPage::new(TabRule::FocusOnly, false, None);
    host_visible_focus.focus_hidden = Some(false);
    let host_visible_focus = Harness::new(host_visible_focus, theme.clone(), 72, 8);
    assert_eq!(host_visible_focus.focus(), Some(CONTEXT_TABS));
    assert_eq!(context_rule_fg(&host_visible_focus, 1), accent);

    let mut covered_focus_visible = ContextPage::new(TabRule::FocusOnly, false, None);
    covered_focus_visible.focus_hidden = Some(false);
    let mut covered_focus_visible = Harness::new(covered_focus_visible, theme.clone(), 72, 8);
    assert_eq!(covered_focus_visible.focus(), Some(CONTEXT_TABS));
    covered_focus_visible.app_mut().popover_open = true;
    let _ = covered_focus_visible.key(KeyCode::F(12));
    assert!(covered_focus_visible.is_open(CONTEXT_POPOVER));
    let reading = context_reading(covered_focus_visible.app(), "live");
    assert!(reading.covered);
    assert!(reading.owner_focused, "nonmodal Popover retains page focus");
    assert_eq!(context_rule_fg(&covered_focus_visible, 1), accent);

    let mut host_hidden_under_popover = ContextPage::new(TabRule::Auto, true, None);
    host_hidden_under_popover.focus_hidden = Some(true);
    let host_hidden_under_popover = Harness::new(host_hidden_under_popover, theme.clone(), 72, 8);
    assert!(context_reading(host_hidden_under_popover.app(), "live").covered);
    assert_same_regions(&host_visible, &host_hidden_under_popover);

    let actions_before = host_visible.app().actions.clone();
    let active_before = host_visible.app().state.active();
    let _ = host_visible.mouse(MouseKind::Down, 1, 0);
    let _ = host_visible.mouse(MouseKind::Up, 1, 0);
    assert_eq!(host_visible.app().actions, actions_before);
    assert_eq!(host_visible.app().state.active(), active_before);

    for rule in [TabRule::Accent, TabRule::Neutral] {
        for covered in [false, true] {
            let baseline_page = ContextPage::new(rule, covered, None);
            let baseline = Harness::new(baseline_page, Theme::junie(), 72, 8);
            let mut override_page = ContextPage::new(rule, covered, None);
            override_page.focus_hidden = Some(!covered);
            let overridden = Harness::new(override_page, Theme::junie(), 72, 8);
            assert_eq!(
                baseline.buffer(),
                overridden.buffer(),
                "fixed {rule:?} rules ignore dynamic presentation overrides"
            );
        }
    }
}

#[test]
fn dynamic_rule_legacy_and_planned_rows_match_for_live_host_contexts() {
    for level in LEVELS {
        let theme = distinct_rule_theme(level);
        for rule in [TabRule::Auto, TabRule::FocusOnly] {
            for hidden in [None, Some(false), Some(true)] {
                for covered in [false, true] {
                    let mut legacy_page = ContextPage::new(rule, covered, None);
                    legacy_page.focus_hidden = hidden;
                    let legacy = Harness::new(legacy_page, theme.clone(), 72, 8);

                    let mut planned_page = ContextPage::new(rule, covered, None);
                    planned_page.focus_hidden = hidden;
                    planned_page.planned = true;
                    let planned = Harness::new(planned_page, theme.clone(), 72, 8);

                    assert_eq!(
                        legacy.buffer(),
                        planned.buffer(),
                        "{rule:?}, hidden={hidden:?}, covered={covered}, {level:?}"
                    );
                    assert_same_regions(&legacy, &planned);
                }
            }
        }
    }
}

#[test]
fn rule_role_colors_remain_distinguishable_with_a_custom_mono_palette() {
    let mut theme = Theme::junie().for_level(ColorLevel::Mono);
    theme.color.accent = Color::Red;
    theme.color.border_strong = Color::Blue;
    theme.color.border_subtle = Color::DarkGray;
    assert_ne!(
        expected_role(&theme, Role::Accent),
        expected_role(&theme, Role::BorderStrong)
    );

    for (rule, role) in [
        (TabRule::Accent, Role::Accent),
        (TabRule::Neutral, Role::BorderStrong),
    ] {
        let h = harness(Page::new(Some(rule)), theme.clone());
        let active = tab_area(&h, 1);
        assert_eq!(h.cell(active.x, 1).fg, expected_role(&theme, role));
        assert_ne!(
            h.cell(active.x, 1).fg,
            expected_role(&theme, Role::BorderSubtle)
        );
    }
}

#[test]
fn absent_policy_keeps_legacy_and_planned_defaults_across_the_full_theme_matrix() {
    let items = vec![
        Tab {
            id: 1,
            label: "Overview",
            marks: TabMarks::default(),
        },
        Tab {
            id: 2,
            label: "orders.sql",
            marks: TabMarks::default().dirty(true),
        },
        Tab {
            id: 3,
            label: "Import",
            marks: TabMarks::default().busy(true),
        },
        Tab {
            id: 4,
            label: "Migrate-日本",
            marks: TabMarks::default().error(true),
        },
    ];
    for (theme_name, base) in [("Junie", Theme::junie()), ("Paper", Theme::paper())] {
        for level in LEVELS {
            let theme = base.clone().for_level(level);
            for marks in [MarkMode::None, MarkMode::Clean, MarkMode::Authored] {
                for status in [Status::Ready, Status::Busy] {
                    for close_reveal in [
                        termrock_navigation::TabCloseReveal::Always,
                        termrock_navigation::TabCloseReveal::Hover,
                    ] {
                        let mut legacy_page = Page::new(None);
                        legacy_page.items = items.clone();
                        legacy_page.marks = marks;
                        legacy_page.status = status;
                        legacy_page.close_reveal = close_reveal;
                        let planned_page = Page {
                            planned: true,
                            ..legacy_page.clone()
                        };
                        let legacy = harness(legacy_page, theme.clone());
                        let planned = harness(planned_page, theme.clone());
                        assert_eq!(
                            legacy.buffer(),
                            planned.buffer(),
                            "{theme_name} {level:?} marks={marks:?} status={status:?} reveal={close_reveal:?}"
                        );
                        assert_legacy_regions_match_with_planned_scroll(&legacy, &planned);
                    }
                }
            }
        }
    }
}

fn custom_rule_theme() -> Theme {
    Theme::junie().override_family(Family::TABS, |recipe| {
        recipe
            .part(Part::RULE)
            .base(
                StylePatch::new()
                    .set_fg(Role::Danger)
                    .set_bg(Role::Custom(Color::Blue))
                    .set_underline(Role::Custom(Color::Yellow))
                    .set_glyph(GlyphRole::Error)
                    .add(Modifier::UNDERLINED),
            )
            .when(
                StateFlags::ACTIVE,
                StylePatch::new()
                    .set_fg(Role::Warning)
                    .set_glyph(GlyphRole::Checked)
                    .remove(Modifier::UNDERLINED),
            );
    })
}

#[test]
fn explicit_rule_policy_overrides_recipe_foreground_only() {
    let theme = custom_rule_theme();
    let original = harness(Page::new(None), theme.clone());
    for (rule, role) in [
        (TabRule::Accent, Role::Accent),
        (TabRule::Neutral, Role::BorderStrong),
        (TabRule::Auto, Role::Accent),
        (TabRule::FocusOnly, Role::Accent),
    ] {
        let configured = harness(Page::new(Some(rule)), theme.clone());
        let active = tab_area(&configured, 1);
        let old = original.cell(active.x, 1);
        let new = configured.cell(active.x, 1);
        assert_eq!(new.fg, expected_role(&theme, role));
        assert_non_foreground_channels_equal(new, old, "custom active RULE recipe");
        assert_eq!(new.symbol(), theme.design.glyphs.get(GlyphRole::Checked));
        assert_eq!(new.bg, Color::Blue);
        assert_eq!(new.underline_color, Color::Yellow);
        assert!(!new.modifier.contains(Modifier::UNDERLINED));
    }
}

#[test]
fn explicit_global_and_part_rule_patches_override_policy_including_reset() {
    let theme = Theme::junie();
    for rule in [
        TabRule::Accent,
        TabRule::Neutral,
        TabRule::Auto,
        TabRule::FocusOnly,
    ] {
        let mut global = Page::new(Some(rule));
        global.patch = RulePatch::Global;
        let global = harness(global, theme.clone());
        let active = tab_area(&global, 1);
        assert_eq!(
            global.cell(active.x, 1).fg,
            expected_role(&theme, Role::Info),
            "global RULE foreground overrides {rule:?}"
        );
        assert_eq!(
            global.cell(active.x, 1).bg,
            expected_role(&theme, Role::Custom(Color::DarkGray))
        );
        assert!(global.cell(active.x, 1).modifier.contains(Modifier::BOLD));

        let mut part = Page::new(Some(rule));
        part.patch = RulePatch::GlobalAndPart;
        let part = harness(part, theme.clone());
        let active = tab_area(&part, 1);
        assert_eq!(
            part.cell(active.x, 1).fg,
            expected_role(&theme, Role::Success),
            "part RULE foreground overrides {rule:?}"
        );
        assert_eq!(part.cell(active.x, 1).bg, Color::Blue);
        assert_eq!(part.cell(active.x, 1).underline_color, Color::Yellow);
        assert!(!part.cell(active.x, 1).modifier.contains(Modifier::BOLD));

        let mut reset = Page::new(Some(rule));
        reset.patch = RulePatch::Reset;
        let reset = harness(reset, theme.clone());
        let active = tab_area(&reset, 1);
        assert_eq!(
            reset.cell(active.x, 1).fg,
            Color::Reset,
            "RULE clear remains authoritative over {rule:?}"
        );
        assert_eq!(
            reset.cell(active.x, 1).symbol(),
            Theme::junie().design.glyphs.get(GlyphRole::RuleActive)
        );
    }
}

#[test]
fn fixed_rules_ignore_focus_and_reference_contexts() {
    let theme = Theme::junie().for_level(ColorLevel::Ansi256);
    for (rule, role) in [
        (TabRule::Accent, Role::Accent),
        (TabRule::Neutral, Role::BorderStrong),
    ] {
        for reference in [
            ReferenceState::FOCUSED,
            ReferenceState::FOCUSED.union(ReferenceState::FOCUS_VISIBLE),
            ReferenceState::HOVERED,
            ReferenceState::PRESSED,
        ] {
            let mut page = Page::new(Some(rule));
            page.reference = Some(reference);
            let h = harness(page, theme.clone());
            let active = h.cell(0, 1);
            assert_eq!(active.fg, expected_role(&theme, role));
            assert!(
                h.area_of_part(TABS, PartRef::item(Part::TAB, ItemKey::num(1)))
                    .is_none(),
                "reference rendering stays inert"
            );
        }

        let mut elsewhere = Page::new(Some(rule));
        elsewhere.open_other_focus = true;
        let mut h = harness(elsewhere, theme.clone());
        for _ in 0..4 {
            if h.focus() == Some(OTHER) {
                break;
            }
            let _ = h.key(KeyCode::Tab);
        }
        assert_eq!(h.focus(), Some(OTHER));
        assert_eq!(h.cell(0, 1).fg, expected_role(&theme, role));
    }
}

struct MismatchPage {
    empty: bool,
    rejected: std::cell::Cell<bool>,
}

impl App for MismatchPage {
    fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let full = ui.full();
        ui.fill(full, Style::default().bg(Color::Blue));
        let area = if self.empty {
            Rect::new(full.x, full.y, 0, 0)
        } else {
            TABS_AREA
        };
        let state = TabsState::default();
        let items = ["Alpha"];
        let planned = Tabs::<&str>::new(TABS)
            .rule(TabRule::Accent)
            .layout_default(area, full, &state, &items, None);
        self.rejected.set(
            Tabs::<&str>::new(TABS)
                .rule(TabRule::Neutral)
                .draw_layout(ui, &planned)
                .is_err(),
        );
    }
}

#[test]
fn rule_policy_mismatch_rejects_before_paint_or_registration_even_for_empty_plans() {
    for empty in [false, true] {
        let h = Harness::new(
            MismatchPage {
                empty,
                rejected: std::cell::Cell::new(false),
            },
            Theme::junie(),
            64,
            5,
        );
        assert!(h.app().rejected.get());
        for y in 0..5 {
            for x in 0..64 {
                assert_eq!(h.cell(x, y).bg, Color::Blue, "sentinel at ({x},{y})");
            }
        }
        assert!(
            h.area_of_part(TABS, PartRef::item(Part::TAB, ItemKey::index(0)))
                .is_none(),
            "a rejected plan does not register controls"
        );
    }
}

struct HostHiddenMismatchPage {
    rejected: std::cell::Cell<bool>,
}

impl App for HostHiddenMismatchPage {
    fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let full = ui.full();
        ui.fill(full, Style::default().bg(Color::Blue));
        let state = TabsState::default();
        let items = ["Alpha"];
        let planned = Tabs::<&str>::new(TABS)
            .rule(TabRule::Auto)
            .rule_focus_hidden(true)
            .layout_default(TABS_AREA, full, &state, &items, None);
        self.rejected.set(
            Tabs::<&str>::new(TABS)
                .rule(TabRule::Auto)
                .rule_focus_hidden(false)
                .draw_layout(ui, &planned)
                .is_err(),
        );
    }
}

#[test]
fn host_hidden_plan_mismatch_rejects_before_paint_or_registration() {
    let h = Harness::new(
        HostHiddenMismatchPage {
            rejected: std::cell::Cell::new(false),
        },
        Theme::junie(),
        64,
        5,
    );
    assert!(h.app().rejected.get());
    for y in 0..5 {
        for x in 0..64 {
            assert_eq!(h.cell(x, y).bg, Color::Blue, "sentinel at ({x},{y})");
        }
    }
    assert!(
        h.area_of_part(TABS, PartRef::item(Part::TAB, ItemKey::index(0)))
            .is_none(),
        "a rejected hidden-context plan does not register controls"
    );
}

#[test]
fn rule_and_existing_builder_settings_survive_key_row_and_marks_order() {
    type CustomTabs = Tabs<'static, Tab, fn(&Tab) -> ItemKey, for<'a> fn(&Tab, &mut RowUi<'a>)>;

    struct CustomPage {
        state: TabsState,
        items: [Tab; 1],
        rule_first: bool,
        actions: Vec<TabsAction>,
    }

    impl App for CustomPage {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let mut response =
                build_custom(self.rule_first).update(cx, &mut self.state, &self.items);
            if let Some(action) = response.take_action() {
                self.actions.push(action);
            }
            response.erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            build_custom(self.rule_first).draw(ui, ui.full(), &self.state, &self.items);
        }
    }

    fn build_custom(rule_first: bool) -> CustomTabs {
        let key = key as fn(&Tab) -> ItemKey;
        let row = custom_row as for<'a> fn(&Tab, &mut RowUi<'a>);
        let base = Tabs::new(TABS)
            .closable(true)
            .middle_click_closes(false)
            .key(key);
        if rule_first {
            base.rule(TabRule::Auto)
                .rule_focus_hidden(true)
                .item_marks(marks)
                .row(row)
        } else {
            base.row(row)
                .item_marks(marks)
                .rule(TabRule::FocusOnly)
                .rule_focus_hidden(true)
        }
    }

    for (rule_first, role) in [(true, Role::BorderStrong), (false, Role::BorderStrong)] {
        let page = CustomPage {
            state: TabsState::default(),
            items: [Tab {
                id: 9,
                label: "Marked row",
                marks: TabMarks::default().dirty(true),
            }],
            rule_first,
            actions: Vec::new(),
        };
        let mut h = Harness::new(page, Theme::junie(), 48, 2);
        let active = h
            .area_of_part(TABS, PartRef::item(Part::TAB, ItemKey::num(9)))
            .unwrap();
        assert_eq!(h.cell(active.x, 1).fg, expected_role(&Theme::junie(), role));
        let middle = RoutedInput::from_crossterm(Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Middle),
            column: active.x.saturating_add(2),
            row: active.y,
            modifiers: KeyModifiers::NONE.into(),
        }))
        .expect("the routed decoder retains a middle-button event");
        let _ = h
            .runtime_mut()
            .handle_routed(middle)
            .expect("a presented runtime handles backend-routed input");
        assert!(
            h.app().actions.is_empty(),
            "the independent Middle=false setting survives builders"
        );
    }
}
