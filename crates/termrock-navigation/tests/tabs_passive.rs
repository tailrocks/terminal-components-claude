#![allow(clippy::unwrap_used, reason = "real runtime contract assertions")]

use core::fmt;
use std::cell::Cell;

use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::{Color, Style};
use termrock_controls::Button;
use termrock_core::action::ActionKey;
use termrock_core::event::{Chord, KeyModifiers, RoutedInput};
use termrock_core::response::Response;
use termrock_core::{Id, ItemKey, KeyCode, MouseKind, Part, PartRef, StateFlags};
use termrock_fields::{TextInput, TextInputState};
use termrock_navigation::tabs::TabWindowReceipt;
use termrock_navigation::{TabCloseReveal, TabMarks, TabRule, Tabs, TabsAction, TabsState};
use termrock_runtime::keymap::KeyMap;
use termrock_runtime::rowui::RowUi;
use termrock_runtime::{
    Anchor, App, BindingState, Bindings, Cx, Dismiss, FrameRead, LayerSize, LayerSpec,
    ReferenceState, ReferenceTarget, RegionKind, Ui, UpdateCause,
};
use termrock_test_support::Harness;
use termrock_theme::resolve::bind_role;
use termrock_theme::{ColorLevel, GlyphRole, Modifier, Role, StylePatch, Surface, Theme};

const TABS_ID: Id = Id::root("tabs.passive");
const HOST_ID: Id = Id::root("tabs.passive.host");
const OVERLAY_ID: Id = Id::root("tabs.passive.overlay");
const TABS_AREA: Rect = Rect::new(0, 0, 72, 2);
const HOST_AREA: Rect = Rect::new(0, 4, 32, 3);
const REFERENCE_AREA: Rect = Rect::new(0, 6, 48, 2);
const CLOSE_KEYS: [ItemKey; 2] = [ItemKey::Num(1), ItemKey::Num(3)];
const LEVELS: [ColorLevel; 4] = [
    ColorLevel::TrueColor,
    ColorLevel::Ansi256,
    ColorLevel::Ansi16,
    ColorLevel::Mono,
];

const GLOBAL_RULE: StylePatch = StylePatch::new()
    .set_fg(Role::Info)
    .set_bg(Role::Custom(Color::DarkGray))
    .add(Modifier::BOLD);
const PART_RULE_SET: [(Part, StylePatch); 1] = [(
    Part::RULE,
    StylePatch::new()
        .set_fg(Role::Success)
        .set_bg(Role::Custom(Color::Blue))
        .add(Modifier::UNDERLINED),
)];
const PART_RULE_CLEAR: [(Part, StylePatch); 1] = [(Part::RULE, StylePatch::new().clear_fg())];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

fn tab_key(tab: &Tab) -> ItemKey {
    ItemKey::num(tab.id)
}

fn tab_marks(tab: &Tab) -> TabMarks {
    tab.marks
}

fn tab_row(tab: &Tab, row: &mut RowUi<'_>) {
    row.label(tab.label);
    row.meta("S");
}

fn tab_row_with_gutter(tab: &Tab, row: &mut RowUi<'_>) {
    row.gutter();
    row.label(tab.label);
    row.meta("S");
}

type TabsWidget = Tabs<'static, Tab, fn(&Tab) -> ItemKey, for<'a> fn(&Tab, &mut RowUi<'a>)>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Coverage {
    None,
    Popover,
    PopoverTabs,
    Modal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RulePatch {
    None,
    GlobalSet,
    PartSet,
    PartClear,
}

struct Page {
    state: TabsState,
    items: Vec<Tab>,
    passive: bool,
    passive_focused: bool,
    passive_configured: bool,
    tabs_y: u16,
    builder_before_key_row: bool,
    custom_row: bool,
    allow_new: bool,
    middle_click_closes: bool,
    close_reveal: TabCloseReveal,
    close_target_width: Option<u16>,
    close_danger_hover: bool,
    rule: Option<TabRule>,
    rule_focus_hidden: Option<bool>,
    rule_patch: RulePatch,
    declare_tabs_focused: bool,
    coverage: Coverage,
    reference: Option<ReferenceState>,
    planned: bool,
    plan_passive: Option<bool>,
    plan_passive_focused: Option<bool>,
    empty_plan: bool,
    rejected_plan: Cell<bool>,
    sentinel: bool,
    tabs_width: u16,
    host_state: TextInputState,
    host_value: String,
    keymap: KeyMap,
    actions: Vec<TabsAction>,
}

impl Page {
    fn new(passive: bool) -> Self {
        let items = vec![
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
            Tab {
                id: 3,
                label: "Gamma",
                marks: TabMarks::default().error(true),
            },
            Tab {
                id: 4,
                label: "Delta",
                marks: TabMarks::default().done(true),
            },
        ];
        let mut state = TabsState::default();
        state.set_active(0, ItemKey::num(1));
        Self {
            state,
            items,
            passive,
            passive_focused: false,
            passive_configured: passive,
            tabs_y: 0,
            builder_before_key_row: false,
            custom_row: false,
            allow_new: true,
            middle_click_closes: true,
            close_reveal: TabCloseReveal::Always,
            close_target_width: None,
            close_danger_hover: false,
            rule: None,
            rule_focus_hidden: None,
            rule_patch: RulePatch::None,
            declare_tabs_focused: false,
            coverage: Coverage::None,
            reference: None,
            planned: false,
            plan_passive: None,
            plan_passive_focused: None,
            empty_plan: false,
            rejected_plan: Cell::new(false),
            sentinel: false,
            tabs_width: TABS_AREA.width,
            host_state: TextInputState::default(),
            host_value: String::new(),
            keymap: KeyMap::new(),
            actions: Vec::new(),
        }
    }

    fn configure_tabs<K, R>(&self, mut tabs: Tabs<'static, Tab, K, R>) -> Tabs<'static, Tab, K, R> {
        if let Some(width) = self.close_target_width {
            tabs = tabs.close_target_w(width);
        }
        if self.close_danger_hover {
            tabs = tabs.close_danger_hover(true);
        }
        if let Some(rule) = self.rule {
            tabs = tabs.rule(rule);
        }
        if let Some(hidden) = self.rule_focus_hidden {
            tabs = tabs.rule_focus_hidden(hidden);
        }
        match self.rule_patch {
            RulePatch::None => tabs,
            RulePatch::GlobalSet => tabs.patch(&GLOBAL_RULE),
            RulePatch::PartSet => tabs.patch_part(&PART_RULE_SET),
            RulePatch::PartClear => tabs.patch_part(&PART_RULE_CLEAR),
        }
    }

    fn tabs_with(&self, passive: bool, passive_focused: bool) -> TabsWidget {
        let base: Tabs<'static, Tab> = Tabs::new(TABS_ID)
            .allow_new(self.allow_new)
            .closable_keys(&CLOSE_KEYS)
            .middle_click_closes(self.middle_click_closes)
            .close_reveal(self.close_reveal);
        let tabs = if self.builder_before_key_row {
            let base = if self.passive_configured {
                base.passive(passive).passive_focused(passive_focused)
            } else {
                base
            };
            base.item_marks(tab_marks as fn(&Tab) -> TabMarks)
                .key(tab_key as fn(&Tab) -> ItemKey)
                .row(self.row_fn())
        } else {
            let tabs = base
                .item_marks(tab_marks as fn(&Tab) -> TabMarks)
                .key(tab_key as fn(&Tab) -> ItemKey)
                .row(self.row_fn());
            if self.passive_configured {
                tabs.passive(passive).passive_focused(passive_focused)
            } else {
                tabs
            }
        };
        self.configure_tabs(tabs)
    }

    fn tabs_default_with(
        &self,
        passive: bool,
        passive_focused: bool,
    ) -> Tabs<'static, Tab, fn(&Tab) -> ItemKey> {
        let base: Tabs<'static, Tab> = Tabs::new(TABS_ID)
            .allow_new(self.allow_new)
            .closable_keys(&CLOSE_KEYS)
            .middle_click_closes(self.middle_click_closes)
            .close_reveal(self.close_reveal);
        let tabs = if self.builder_before_key_row {
            let base = if self.passive_configured {
                base.passive(passive).passive_focused(passive_focused)
            } else {
                base
            };
            base.item_marks(tab_marks as fn(&Tab) -> TabMarks)
                .key(tab_key as fn(&Tab) -> ItemKey)
        } else {
            let tabs = base
                .item_marks(tab_marks as fn(&Tab) -> TabMarks)
                .key(tab_key as fn(&Tab) -> ItemKey);
            if self.passive_configured {
                tabs.passive(passive).passive_focused(passive_focused)
            } else {
                tabs
            }
        };
        self.configure_tabs(tabs)
    }

    fn tabs(&self) -> TabsWidget {
        self.tabs_with(self.passive, self.passive_focused)
    }

    fn row_fn(&self) -> for<'a> fn(&Tab, &mut RowUi<'a>) {
        if self.custom_row {
            tab_row_with_gutter
        } else {
            tab_row
        }
    }

    fn tabs_area(&self) -> Rect {
        Rect::new(TABS_AREA.x, self.tabs_y, self.tabs_width, TABS_AREA.height)
    }

    fn update_coverage(&self, cx: &mut Cx<'_>) {
        match self.coverage {
            Coverage::None if cx.is_open(OVERLAY_ID) => cx.close_layer(OVERLAY_ID, None),
            Coverage::Popover | Coverage::PopoverTabs if !cx.is_open(OVERLAY_ID) => {
                let anchor = if self.coverage == Coverage::PopoverTabs {
                    Anchor::Point(Position::new(0, 0))
                } else {
                    Anchor::Point(Position::new(54, 4))
                };
                cx.open_layer(
                    OVERLAY_ID,
                    LayerSpec::popover(OVERLAY_ID, anchor)
                        .size(LayerSize::Fixed(14, 3))
                        .dismiss(Dismiss::NONE),
                );
            }
            Coverage::Modal if !cx.is_open(OVERLAY_ID) => {
                cx.open_layer(
                    OVERLAY_ID,
                    LayerSpec::modal(OVERLAY_ID).initial_focus(OVERLAY_ID),
                );
            }
            _ => {}
        }
    }

    fn draw_tabs(&self, ui: &mut Ui<'_>) {
        if self.declare_tabs_focused {
            ui.declare_state(TABS_ID, StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE);
        }
        let area = self.tabs_area();
        let tabs = self.tabs_default_with(self.passive, self.passive_focused);
        if self.planned && !self.custom_row {
            let plan_tabs = self.tabs_default_with(
                self.plan_passive.unwrap_or(self.passive),
                self.plan_passive_focused.unwrap_or(self.passive_focused),
            );
            let previous = ui.layout(TABS_ID).map(|facts| TabWindowReceipt {
                viewport_len: facts.viewport_len,
                content_len: facts.content_len,
            });
            let area = if self.empty_plan {
                Rect::new(area.x, area.y, 0, 0)
            } else {
                area
            };
            let layout = plan_tabs.layout_default(area, area, &self.state, &self.items, previous);
            self.rejected_plan
                .set(tabs.draw_layout(ui, &layout).is_err());
        } else if self.custom_row {
            self.tabs().draw(ui, area, &self.state, &self.items);
        } else {
            tabs.draw(ui, area, &self.state, &self.items);
        }
    }
}

impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.update_coverage(cx);
        if cx.update_cause() == UpdateCause::Bootstrap {
            self.host_state.begin(&self.host_value);
        }
        let mut tabs_response = self.tabs().update(cx, &mut self.state, &self.items);
        if let Some(action) = tabs_response.take_action() {
            self.actions.push(action);
        }
        let mut response = tabs_response.erase();
        let input = TextInput::new(HOST_ID).update(cx, &mut self.host_state, &mut self.host_value);
        response |= input.erase();
        if self.coverage != Coverage::None {
            response |= Button::new(OVERLAY_ID, "Overlay").update(cx).erase();
        }
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        if self.sentinel {
            ui.fill(ui.full(), Style::default().bg(Color::Blue));
        }
        self.draw_tabs(ui);
        if let Some(reference) = self.reference {
            let tabs = self.tabs_default_with(self.passive, self.passive_focused);
            ui.reference(
                Some(ReferenceTarget::new(TABS_ID, reference).part(PartRef::of(Part::RULE))),
                |reference_ui| {
                    tabs.draw(reference_ui, REFERENCE_AREA, &self.state, &self.items);
                },
            );
        }
        TextInput::new(HOST_ID)
            .value(&self.host_value)
            .draw(ui, HOST_AREA, &self.host_state);
        if self.coverage != Coverage::None {
            let _ = ui.layer(OVERLAY_ID, |layer, area| {
                Button::new(OVERLAY_ID, "Overlay").draw(layer, area);
            });
        }
    }

    fn keymap(&self) -> &KeyMap {
        &self.keymap
    }
}

fn harness(page: Page, theme: Theme) -> Harness<Page> {
    Harness::new(page, theme, 72, 10)
}

fn tab_part(part: Part, key: u64) -> PartRef {
    PartRef::item(part, ItemKey::num(key))
}

fn expected_role(theme: &Theme, role: Role) -> Color {
    bind_role(theme, role, Surface::Canvas).unwrap_or(Color::Reset)
}

fn distinct_theme(level: ColorLevel) -> Theme {
    let mut theme = Theme::junie().for_level(level);
    theme.color.accent = Color::Red;
    theme.color.border_strong = Color::Blue;
    theme
}

fn click_middle(h: &mut Harness<Page>, part: PartRef) {
    let area = h
        .area_of_part(TABS_ID, part)
        .expect("passive part is registered");
    let pos = Position::new(area.x.saturating_add(area.width / 2), area.y);
    let input = RoutedInput::MiddleClick {
        pos,
        mods: KeyModifiers::NONE,
    };
    if let Err(pending) = h.runtime_mut().handle_routed(input) {
        let input = pending.into_routed_input();
        h.draw();
        let _ = h
            .runtime_mut()
            .handle_routed(input)
            .expect("runtime is ready after its scheduled presentation");
    }
    h.draw();
}

#[test]
fn passive_omits_automatic_focus_and_bindings_while_host_keeps_typing() {
    let mut passive = Page::new(true);
    passive.keymap = KeyMap::new().bind_component(
        TABS_ID,
        ActionKey::custom("tabs.next"),
        Chord::key(KeyCode::Char('q')),
    );
    let mut h = harness(passive, Theme::junie());
    let tabs: Tabs<'static, Tab> = Tabs::new(TABS_ID).allow_new(true).passive(true);

    assert!(
        h.next_deadline().is_some(),
        "a busy mark keeps its scheduled animation in passive mode"
    );
    assert!(tabs.bindings(BindingState::default()).is_empty());
    assert!(!h.ring().entries().iter().any(|entry| entry.id == TABS_ID));
    assert!(h.ring().entries().iter().any(|entry| entry.id == HOST_ID));
    assert_eq!(h.focus(), Some(HOST_ID));
    assert!(h.runtime().area_of(TABS_ID).is_none());
    assert!(
        h.runtime()
            .registry()
            .regions()
            .iter()
            .all(|region| { region.owner != TABS_ID || region.kind == RegionKind::Part })
    );
    assert!(h.area_of_part(TABS_ID, tab_part(Part::TAB, 1)).is_some());
    assert!(h.area_of_part(TABS_ID, PartRef::of(Part::NEW)).is_some());
    assert!(h.runtime().registry().hit(Position::new(60, 0)).is_none());
    assert!(h.runtime().registry().hit(Position::new(40, 3)).is_none());

    let _ = h.key(KeyCode::Tab);
    assert_eq!(
        h.focus(),
        Some(HOST_ID),
        "Tab traversal skips the passive strip"
    );
    let _ = h.click(4, 4);
    assert_eq!(h.focus(), Some(HOST_ID));
    assert!(h.app().host_state.is_editing());
    let _ = h.type_str("q");
    assert_eq!(
        h.app().host_state.draft_text(),
        Some("q"),
        "the host receives a Tabs-remapped key"
    );
    assert_eq!(h.app().state.active(), Some(ItemKey::num(1)));
    assert!(h.app().actions.is_empty());

    let mut active = Page::new(false);
    active.keymap = KeyMap::new().bind_component(
        TABS_ID,
        ActionKey::custom("tabs.next"),
        Chord::key(KeyCode::Char('q')),
    );
    let mut active = harness(active, Theme::junie());
    assert!(
        active.next_deadline().is_some(),
        "the ordinary busy mark retains its animation schedule"
    );
    let active_tabs: Tabs<'static, Tab> = Tabs::new(TABS_ID);
    assert_eq!(
        active_tabs
            .allow_new(true)
            .bindings(BindingState::default())
            .len(),
        16,
        "the ordinary table remains present when passive is absent"
    );
    assert!(active.tab_to(TABS_ID));
    let _ = active.key(KeyCode::Char('q'));
    assert_eq!(active.app().state.active(), Some(ItemKey::num(2)));
    assert_eq!(
        active.app().actions,
        [TabsAction::Activated(ItemKey::num(2))]
    );
}

#[test]
fn passive_keeps_tab_close_new_middle_and_overflow_parts_live() {
    let page = Page::new(true);
    let original_items = page.items.clone();
    let mut h = harness(page, Theme::junie());
    assert_eq!(h.focus(), Some(HOST_ID));

    let _ = h.click_part(TABS_ID, tab_part(Part::TAB, 2));
    assert_eq!(h.app().actions, [TabsAction::Activated(ItemKey::num(2))]);
    assert_eq!(h.focus(), Some(HOST_ID));
    assert_eq!(
        h.app().items,
        original_items,
        "actions leave host-owned data alone"
    );

    h.app_mut().actions.clear();
    let _ = h.click_part(TABS_ID, tab_part(Part::CLOSE, 1));
    assert_eq!(h.app().actions, [TabsAction::Close(ItemKey::num(1))]);
    assert_eq!(h.focus(), Some(HOST_ID));
    assert!(h.area_of_part(TABS_ID, tab_part(Part::CLOSE, 2)).is_none());

    h.app_mut().actions.clear();
    click_middle(&mut h, tab_part(Part::TAB, 3));
    assert_eq!(h.app().actions, [TabsAction::Close(ItemKey::num(3))]);
    assert_eq!(h.focus(), Some(HOST_ID));

    h.app_mut().actions.clear();
    let _ = h.click_part(TABS_ID, PartRef::of(Part::NEW));
    assert_eq!(h.app().actions, [TabsAction::New]);
    assert_eq!(h.focus(), Some(HOST_ID));

    let mut no_middle = Page::new(true);
    no_middle.middle_click_closes = false;
    let mut no_middle = harness(no_middle, Theme::junie());
    click_middle(&mut no_middle, tab_part(Part::TAB, 1));
    assert!(no_middle.app().actions.is_empty());
    assert_eq!(no_middle.focus(), Some(HOST_ID));

    let mut overflow = Page::new(true);
    overflow.tabs_width = 24;
    overflow.items = (1..=8)
        .map(|id| Tab {
            id,
            label: "Tab",
            marks: TabMarks::default(),
        })
        .collect();
    overflow.state = TabsState::default();
    overflow.state.set_active(1, ItemKey::num(2));
    let mut overflow = harness(overflow, Theme::junie());
    let right = PartRef::item(Part::OVERFLOW, ItemKey::index(1));
    let area = overflow
        .area_of_part(TABS_ID, right)
        .expect("right overflow is visible");
    let position = Position::new(area.x.saturating_add(area.width / 2), area.y);
    let hit = overflow
        .runtime()
        .registry()
        .hit(position)
        .expect("right overflow is a live pointer target");
    assert_eq!(hit.owner, TABS_ID);
    assert_eq!(hit.part, right);
    let response = overflow.click_part(TABS_ID, right);
    assert!(
        response.is_changed(),
        "overflow pointer updates the visible window"
    );
    assert_eq!(overflow.app().state.first(), Some(ItemKey::num(2)));
    assert!(overflow.app().actions.is_empty());
    assert_eq!(overflow.focus(), Some(HOST_ID));
    assert!(
        overflow.diagnostics().is_empty(),
        "{:?}",
        overflow.diagnostics()
    );
}

#[test]
fn passive_parts_respect_modal_and_overlapping_popover_hit_boundaries() {
    for coverage in [Coverage::PopoverTabs, Coverage::Modal] {
        let mut page = Page::new(true);
        if coverage == Coverage::PopoverTabs {
            page.tabs_y = 1;
        }
        let mut h = harness(page, Theme::junie());
        let tab = tab_part(Part::TAB, 1);
        let area = h
            .area_of_part(TABS_ID, tab)
            .expect("passive tab remains drawn");
        let position = Position::new(area.x.saturating_add(1), area.y);
        h.app_mut().coverage = coverage;
        let _ = h.tick();
        assert!(h.is_open(OVERLAY_ID), "{coverage:?}");
        let top_hit = h
            .runtime()
            .registry()
            .hit(position)
            .expect("the covering native layer owns this point");
        assert_eq!(top_hit.owner, OVERLAY_ID, "{coverage:?}");

        let _ = h.click(position.x, position.y);
        assert!(h.app().actions.is_empty(), "{coverage:?}");
        assert_eq!(h.app().state.active(), Some(ItemKey::num(1)));
        assert_eq!(h.focus(), Some(OVERLAY_ID), "top-layer part owns the click");
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
}

#[test]
fn passive_hover_close_press_keeps_the_tab_target_and_reveals_after_activation() {
    let mut page = Page::new(true);
    page.close_reveal = TabCloseReveal::Hover;
    let mut h = harness(page, Theme::junie());
    assert!(h.area_of_part(TABS_ID, tab_part(Part::CLOSE, 1)).is_some());
    assert!(h.area_of_part(TABS_ID, tab_part(Part::CLOSE, 3)).is_none());
    let tab = h
        .area_of_part(TABS_ID, tab_part(Part::TAB, 3))
        .expect("third tab is a live part");

    let _ = h.mouse(MouseKind::Down, tab.x.saturating_add(1), tab.y);
    assert_eq!(h.app().state.cursor(), Some(ItemKey::num(3)));
    assert_eq!(h.app().state.active(), Some(ItemKey::num(1)));
    assert!(h.area_of_part(TABS_ID, tab_part(Part::CLOSE, 3)).is_none());
    let _ = h.mouse(MouseKind::Up, tab.x.saturating_add(1), tab.y);
    assert_eq!(h.app().actions, [TabsAction::Activated(ItemKey::num(3))]);
    assert!(h.area_of_part(TABS_ID, tab_part(Part::CLOSE, 3)).is_some());
    assert_eq!(h.focus(), Some(HOST_ID));

    h.app_mut().actions.clear();
    let _ = h.click_part(TABS_ID, tab_part(Part::CLOSE, 3));
    assert_eq!(h.app().actions, [TabsAction::Close(ItemKey::num(3))]);
    assert_eq!(h.focus(), Some(HOST_ID));
}

#[test]
fn passive_presentation_and_dynamic_rules_ignore_physical_declared_and_reference_focus() {
    for level in LEVELS {
        let theme = distinct_theme(level);
        let accent = expected_role(&theme, Role::Accent);
        let neutral = expected_role(&theme, Role::BorderStrong);
        assert_ne!(
            accent, neutral,
            "custom role colors stay distinct at {level:?}"
        );

        for rule in [
            TabRule::Auto,
            TabRule::FocusOnly,
            TabRule::Accent,
            TabRule::Neutral,
        ] {
            for passive_focused in [false, true] {
                for coverage in [Coverage::None, Coverage::Popover] {
                    for hidden in [None, Some(false), Some(true)] {
                        let mut page = Page::new(true);
                        page.passive_focused = passive_focused;
                        page.rule = Some(rule);
                        page.rule_focus_hidden = hidden;
                        page.coverage = coverage;
                        page.declare_tabs_focused = true;
                        let h = harness(page, theme.clone());
                        let actual = h.cell(0, 1).fg;
                        let hidden_now = hidden.unwrap_or(coverage != Coverage::None);
                        let expected = match rule {
                            TabRule::Auto if hidden_now => neutral,
                            TabRule::Auto => accent,
                            TabRule::FocusOnly if passive_focused => accent,
                            TabRule::FocusOnly => neutral,
                            TabRule::Accent => accent,
                            TabRule::Neutral => neutral,
                            _ => unreachable!("all currently supported rules are listed"),
                        };
                        assert_eq!(
                            actual, expected,
                            "{rule:?} passive_focused={passive_focused} coverage={coverage:?} hidden={hidden:?} {level:?}"
                        );
                        assert!(!h.ring().entries().iter().any(|entry| entry.id == TABS_ID));
                        assert_ne!(h.focus(), Some(TABS_ID));
                    }
                }
            }
        }

        let focus_glyph = theme.design.glyphs.get(GlyphRole::FocusBar);
        for passive_focused in [false, true] {
            let mut page = Page::new(true);
            page.passive_focused = passive_focused;
            page.declare_tabs_focused = true;
            page.close_reveal = TabCloseReveal::Hover;
            page.custom_row = true;
            let h = harness(page, theme.clone());
            assert_eq!(
                h.cell(1, 0).symbol() == focus_glyph,
                passive_focused,
                "only the explicit passive focus value paints the cursor gutter ({level:?})"
            );
            assert_eq!(
                h.focus(),
                Some(HOST_ID),
                "presentation does not grant focus"
            );
        }

        for reference_focused in [false, true] {
            for passive_focused in [false, true] {
                let mut page = Page::new(true);
                page.passive_focused = passive_focused;
                page.rule = Some(TabRule::FocusOnly);
                page.reference = Some(if reference_focused {
                    ReferenceState::FOCUSED.union(ReferenceState::FOCUS_VISIBLE)
                } else {
                    ReferenceState::default()
                });
                let h = harness(page, theme.clone());
                assert_eq!(
                    h.cell(0, 7).fg,
                    if passive_focused { accent } else { neutral },
                    "passive presentation outranks synthetic reference focus ({level:?})"
                );
                assert!(h.runtime().registry().hit(Position::new(0, 6)).is_none());
                assert!(!h.runtime().registry().regions().iter().any(|region| {
                    region.owner == TABS_ID && region.area.y >= REFERENCE_AREA.y
                }));
                assert_eq!(h.focus(), Some(HOST_ID));
            }
        }

        for (patch, expected) in [
            (RulePatch::GlobalSet, expected_role(&theme, Role::Info)),
            (RulePatch::PartSet, expected_role(&theme, Role::Success)),
            (RulePatch::PartClear, Color::Reset),
        ] {
            let mut page = Page::new(true);
            page.passive_focused = true;
            page.rule = Some(TabRule::FocusOnly);
            page.rule_focus_hidden = Some(true);
            page.coverage = Coverage::None;
            page.rule_patch = patch;
            let h = harness(page, theme.clone());
            assert_eq!(
                h.cell(0, 1).fg,
                expected,
                "explicit RULE patch {patch:?} at {level:?}"
            );
        }
    }
}

#[test]
fn passive_focus_controls_hover_close_for_an_inactive_cursor_row() {
    for passive_focused in [false, true] {
        let mut page = Page::new(true);
        page.passive_focused = passive_focused;
        page.close_reveal = TabCloseReveal::Hover;
        let mut h = harness(page, Theme::junie());
        let tab = h
            .area_of_part(TABS_ID, tab_part(Part::TAB, 3))
            .expect("cursor can move to an eligible inactive tab");

        let _ = h.mouse(MouseKind::Down, tab.x.saturating_add(1), tab.y);
        let _ = h.mouse(MouseKind::Up, 70, 1);
        assert_eq!(h.app().state.active(), Some(ItemKey::num(1)));
        assert_eq!(h.app().state.cursor(), Some(ItemKey::num(3)));
        assert_eq!(
            h.area_of_part(TABS_ID, tab_part(Part::CLOSE, 3)).is_some(),
            passive_focused,
            "close reveal follows passive presentation, not the host's focus"
        );
        assert_eq!(h.focus(), Some(HOST_ID));
    }
}

#[test]
fn switching_active_tabs_to_passive_rejects_stale_binding_actions() {
    for chord in [KeyCode::Right, KeyCode::Char('x')] {
        let mut h = harness(Page::new(false), Theme::junie());
        assert_eq!(h.focus(), Some(TABS_ID));
        h.app_mut().passive = true;
        h.app_mut().passive_configured = true;
        let response = h.key(chord);
        assert!(
            response.is_consumed(),
            "stale published action is acknowledged"
        );
        assert!(h.app().actions.is_empty());
        assert_eq!(h.app().state.active(), Some(ItemKey::num(1)));
        assert_eq!(h.app().state.cursor(), Some(ItemKey::num(1)));
        assert_eq!(
            h.focus(),
            Some(HOST_ID),
            "the strip leaves the ring on redraw"
        );
        assert!(h.app().tabs().bindings(BindingState::default()).is_empty());
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }

    let mut reenabled = harness(Page::new(false), Theme::junie());
    assert_eq!(reenabled.focus(), Some(TABS_ID));
    reenabled.app_mut().passive = true;
    reenabled.app_mut().passive_configured = true;
    let _ = reenabled.key(KeyCode::Right);
    reenabled.app_mut().passive = false;
    assert!(reenabled.tab_to(TABS_ID));
    let _ = reenabled.key(KeyCode::Right);
    assert_eq!(reenabled.app().state.active(), Some(ItemKey::num(2)));
    assert_eq!(
        reenabled.app().actions,
        [TabsAction::Activated(ItemKey::num(2))]
    );
}

#[test]
fn passive_configuration_is_additive_and_false_defaults_are_cell_exact() {
    for level in LEVELS {
        let theme = Theme::junie().for_level(level);
        let plain = harness(Page::new(false), theme.clone());
        let mut explicit_false_page = Page::new(false);
        explicit_false_page.passive_configured = true;
        let mut ignored_focus_page = Page::new(false);
        ignored_focus_page.passive_configured = true;
        ignored_focus_page.passive_focused = true;
        let mut explicit_false = harness(explicit_false_page, theme.clone());
        let mut ignored_focus = harness(ignored_focus_page, theme);
        let ids = |h: &Harness<Page>| {
            h.ring()
                .entries()
                .iter()
                .map(|entry| entry.id)
                .collect::<Vec<_>>()
        };
        for candidate in [&explicit_false, &ignored_focus] {
            assert_eq!(
                plain.buffer(),
                candidate.buffer(),
                "complete buffer {level:?}"
            );
            assert_eq!(
                plain.runtime().registry().regions(),
                candidate.runtime().registry().regions(),
                "all live control/part registrations {level:?}"
            );
            assert_eq!(ids(&plain), ids(candidate), "focus ring {level:?}");
            assert_eq!(plain.focus(), candidate.focus());
            assert_eq!(plain.app().state, candidate.app().state);
            assert_eq!(
                plain.app().tabs().bindings(BindingState::default()),
                candidate.app().tabs().bindings(BindingState::default())
            );
        }

        let mut plain_input = plain;
        assert!(plain_input.tab_to(TABS_ID));
        assert!(explicit_false.tab_to(TABS_ID));
        assert!(ignored_focus.tab_to(TABS_ID));
        let _ = plain_input.key(KeyCode::Right);
        let _ = explicit_false.key(KeyCode::Right);
        let _ = ignored_focus.key(KeyCode::Right);
        for candidate in [&explicit_false, &ignored_focus] {
            assert_eq!(plain_input.app().actions, candidate.app().actions);
            assert_eq!(plain_input.app().state, candidate.app().state);
            assert_eq!(
                plain_input.runtime().repaint_after(),
                candidate.runtime().repaint_after()
            );
            assert!(candidate.diagnostics().is_empty());
        }
        assert!(plain_input.diagnostics().is_empty());
    }
}

#[test]
fn passive_plans_snapshot_both_settings_and_reject_mismatch_before_effects() {
    for (plan_passive, plan_focused, empty) in [
        (Some(false), Some(true), false),
        (Some(true), Some(false), true),
        (Some(false), Some(false), true),
    ] {
        let mut page = Page::new(true);
        page.passive_focused = true;
        page.planned = true;
        page.plan_passive = plan_passive;
        page.plan_passive_focused = plan_focused;
        page.empty_plan = empty;
        page.sentinel = true;
        let h = harness(page, Theme::junie());
        assert!(h.app().rejected_plan.get(), "plan mismatch is rejected");
        assert!(
            h.runtime()
                .registry()
                .regions()
                .iter()
                .all(|region| region.owner != TABS_ID)
        );
        assert!(h.runtime().area_of(TABS_ID).is_none());
        for y in 0..2 {
            for x in 0..TABS_AREA.width {
                assert_eq!(
                    h.cell(x, y).bg,
                    Color::Blue,
                    "rejected plan left ({x},{y}) untouched"
                );
            }
        }
    }

    let mut matching = Page::new(true);
    matching.passive_focused = true;
    matching.planned = true;
    matching.plan_passive = Some(true);
    matching.plan_passive_focused = Some(true);
    let matching = harness(matching, Theme::junie());
    assert!(
        !matching.app().rejected_plan.get(),
        "matching plans still draw"
    );
    assert!(
        matching
            .area_of_part(TABS_ID, tab_part(Part::TAB, 1))
            .is_some()
    );

    for (passive, passive_focused) in [(false, false), (true, false), (true, true)] {
        for builder_before_key_row in [false, true] {
            let mut legacy = Page::new(passive);
            legacy.passive_focused = passive_focused;
            legacy.passive_configured = true;
            legacy.builder_before_key_row = builder_before_key_row;
            let legacy = harness(legacy, Theme::junie());

            let mut planned = Page::new(passive);
            planned.passive_focused = passive_focused;
            planned.passive_configured = true;
            planned.builder_before_key_row = builder_before_key_row;
            planned.planned = true;
            let planned = harness(planned, Theme::junie());
            assert!(!planned.app().rejected_plan.get());
            assert_eq!(legacy.buffer(), planned.buffer());
            assert_eq!(
                legacy.runtime().registry().regions(),
                planned.runtime().registry().regions()
            );
        }
    }
}
