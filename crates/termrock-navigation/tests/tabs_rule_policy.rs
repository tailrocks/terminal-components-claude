use core::fmt;

use ratatui::crossterm::event::{Event, MouseButton, MouseEvent, MouseEventKind};
use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Modifier, Style};
use termrock_controls::Button;
use termrock_core::event::{KeyModifiers, RoutedInput};
use termrock_core::response::Response;
use termrock_core::{Id, ItemKey, KeyCode, MouseKind, Part, PartRef, StateFlags};
use termrock_navigation::tabs::TabWindowReceipt;
use termrock_navigation::{TabMarks, TabRule, Tabs, TabsAction, TabsState};
use termrock_runtime::rowui::RowUi;
use termrock_runtime::{App, Cx, FrameRead, ReferenceState, ReferenceTarget, Status, Ui};
use termrock_test_support::Harness;
use termrock_theme::resolve::bind_role;
use termrock_theme::{ColorLevel, Family, GlyphRole, Role, StylePatch, Surface, Theme};

const TABS: Id = Id::root("tabs.rule-policy");
const OTHER: Id = Id::root("tabs.rule-policy.other");
const TABS_AREA: Rect = Rect::new(0, 0, 48, 2);
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
    planned: bool,
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
            planned: false,
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

#[test]
fn fixed_rule_roles_follow_theme_tokens_and_change_only_active_foreground() {
    for (theme_name, base) in [("Junie", Theme::junie()), ("Paper", Theme::paper())] {
        for level in LEVELS {
            let theme = base.clone().for_level(level);
            for rule in [TabRule::Accent, TabRule::Neutral] {
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
                    _ => unreachable!("tests cover only the implemented fixed policies"),
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
                        assert_eq!(
                            legacy.runtime().registry().regions(),
                            planned.runtime().registry().regions(),
                            "legacy/planned hit geometry must remain equal"
                        );
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
    let mut global = Page::new(Some(TabRule::Accent));
    global.patch = RulePatch::Global;
    let global = harness(global, theme.clone());
    let active = tab_area(&global, 1);
    assert_eq!(
        global.cell(active.x, 1).fg,
        expected_role(&theme, Role::Info)
    );
    assert_eq!(
        global.cell(active.x, 1).bg,
        expected_role(&theme, Role::Custom(Color::DarkGray))
    );
    assert!(global.cell(active.x, 1).modifier.contains(Modifier::BOLD));

    let mut part = Page::new(Some(TabRule::Accent));
    part.patch = RulePatch::GlobalAndPart;
    let part = harness(part, theme.clone());
    let active = tab_area(&part, 1);
    assert_eq!(
        part.cell(active.x, 1).fg,
        expected_role(&theme, Role::Success)
    );
    assert_eq!(part.cell(active.x, 1).bg, Color::Blue);
    assert_eq!(part.cell(active.x, 1).underline_color, Color::Yellow);
    assert!(!part.cell(active.x, 1).modifier.contains(Modifier::BOLD));

    let mut reset = Page::new(Some(TabRule::Neutral));
    reset.patch = RulePatch::Reset;
    let reset = harness(reset, theme);
    let active = tab_area(&reset, 1);
    assert_eq!(reset.cell(active.x, 1).fg, Color::Reset);
    assert_eq!(
        reset.cell(active.x, 1).symbol(),
        Theme::junie().design.glyphs.get(GlyphRole::RuleActive)
    );
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
            base.rule(TabRule::Accent).item_marks(marks).row(row)
        } else {
            base.row(row).item_marks(marks).rule(TabRule::Neutral)
        }
    }

    for (rule_first, role) in [(true, Role::Accent), (false, Role::BorderStrong)] {
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
