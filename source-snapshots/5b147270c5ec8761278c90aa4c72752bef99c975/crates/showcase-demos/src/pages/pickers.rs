//! Searchable semantic picker with query and scope state.

use termrock::{
    ActionKey, Button, ContextMenu, Cx, FgStep, FilterList, FilterListState, Id, Item, ItemKey,
    ItemRowLayout, LayerSize, Menu, MenuBar, MenuItem, MenuState, Modifier, Panel, PanelKind, Part,
    Picker, PickerAction, PickerChain, PickerChainState, PickerStage, PickerState, Position, Props,
    Rect, Response, Role, ScreenAlign, StylePatch, Ui, Variant, id,
};

use super::{Page, PageStatus, PageUpdate, frame};

const OPEN_QUICK: Id = id!("pickers.open.quick");
const OPEN_TABS: Id = id!("pickers.open.tabs");
const OPEN_LEVEL: Id = id!("pickers.open.level");
const PICKER: Id = id!("pickers.layer");
/// Shell-owned navigation control (`showcase-ui` `NAV`). Ids are content
/// hashes, so this names the same control without a dependency cycle.
/// Historical post-modal focus target: choosing or cancelling a picker
/// returns focus to NAV (S4 PICKER-QUERY-001 / PICKER-TABS-002 live).
const SHELL_NAV: Id = id!("navigation");
const FILTER: Id = id!("pickers.filter");
const CHAIN: Id = id!("pickers.chain");
const MENU: Id = id!("pickers.menu");
const CONTEXT: Id = id!("pickers.context");
const LAUNCHER_PANEL: Id = id!("pickers.launcher");
const RESULTS_PANEL: Id = id!("pickers.results");

const PANEL_PARTS: &[(Part, StylePatch)] = &[(
    Part::TITLE,
    StylePatch::new()
        .set_fg(Role::Fg(FgStep::Secondary))
        .remove(Modifier::BOLD),
)];

const SCOPES: &[termrock::ScopeKey] = &[
    termrock::ScopeKey::new(1),
    termrock::ScopeKey::new(2),
    termrock::ScopeKey::new(3),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PickerKind {
    Quick,
    Tabs,
    Level,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum QuickScope {
    #[default]
    All,
    Files,
    Tasks,
}

const QUICK_ITEMS: &[Item<'static>] = &[
    Item::new(ItemKey::Num(100), "Cargo.toml")
        .glyph("F")
        .detail("Cargo.toml")
        .group("Files"),
    Item::new(ItemKey::Num(101), "README.md")
        .glyph("F")
        .detail("README.md")
        .group("Files"),
    Item::new(ItemKey::Num(102), "architecture.md")
        .glyph("F")
        .detail("docs/architecture.md")
        .group("Files"),
    Item::new(ItemKey::Num(103), "auth.rs")
        .glyph("F")
        .detail("src/api/auth.rs")
        .group("Files"),
    Item::new(ItemKey::Num(104), "auth_flow.rs")
        .glyph("F")
        .detail("tests/auth_flow.rs")
        .group("Files"),
    Item::new(ItemKey::Num(105), "billing.rs")
        .glyph("F")
        .detail("src/api/billing.rs")
        .group("Files"),
    Item::new(ItemKey::Num(106), "checkout.rs")
        .glyph("F")
        .detail("tests/checkout.rs")
        .group("Files"),
    Item::new(ItemKey::Num(107), "config.rs")
        .glyph("F")
        .detail("src/config.rs")
        .group("Files"),
    Item::new(ItemKey::Num(108), "dispatch.rs")
        .glyph("F")
        .detail("src/api/webhooks/dispatch.rs")
        .group("Files"),
    Item::new(ItemKey::Num(109), "lib.rs")
        .glyph("F")
        .detail("src/lib.rs")
        .group("Files"),
    Item::new(ItemKey::Num(110), "mailer.rs")
        .glyph("F")
        .detail("src/workers/mailer.rs")
        .group("Files"),
    Item::new(ItemKey::Num(111), "main.rs")
        .glyph("F")
        .detail("src/main.rs")
        .group("Files"),
    Item::new(ItemKey::Num(112), "migrations.rs")
        .glyph("F")
        .detail("src/db/migrations.rs")
        .group("Files"),
    Item::new(ItemKey::Num(113), "mod.rs")
        .glyph("F")
        .detail("src/api/mod.rs")
        .group("Files"),
    Item::new(ItemKey::Num(114), "mod.rs")
        .glyph("F")
        .detail("src/api/webhooks/mod.rs")
        .group("Files"),
    Item::new(ItemKey::Num(115), "orders.json")
        .glyph("F")
        .detail("tests/fixtures/orders.json")
        .group("Files"),
    Item::new(ItemKey::Num(116), "pool.rs")
        .glyph("F")
        .detail("src/db/pool.rs")
        .group("Files"),
    Item::new(ItemKey::Num(117), "retry.rs")
        .glyph("F")
        .detail("src/api/webhooks/retry.rs")
        .group("Files"),
    Item::new(ItemKey::Num(118), "schema.rs")
        .glyph("F")
        .detail("src/db/schema.rs")
        .group("Files"),
    Item::new(ItemKey::Num(119), "scheduler.rs")
        .glyph("F")
        .detail("src/workers/scheduler.rs")
        .group("Files"),
    Item::new(ItemKey::Num(120), "users.json")
        .glyph("F")
        .detail("tests/fixtures/users.json")
        .group("Files"),
    Item::new(ItemKey::Num(121), "webhooks.md")
        .glyph("F")
        .detail("docs/webhooks.md")
        .group("Files"),
    Item::new(ItemKey::Num(200), "Add OpenTelemetry tracing spans")
        .glyph("T")
        .detail("#1047 · kai")
        .group("Tasks"),
    Item::new(ItemKey::Num(201), "Add rate limiting to auth endpoints")
        .glyph("T")
        .detail("#1040 · mira")
        .group("Tasks"),
    Item::new(ItemKey::Num(202), "Extract billing service module")
        .glyph("T")
        .detail("#1046 · sofia")
        .group("Tasks"),
    Item::new(ItemKey::Num(203), "Fix flaky checkout integration test")
        .glyph("T")
        .detail("#1042 · ana")
        .group("Tasks"),
    Item::new(ItemKey::Num(204), "Generate API client from OpenAPI")
        .glyph("T")
        .detail("#1049 · sofia")
        .group("Tasks"),
    Item::new(ItemKey::Num(205), "Harden CSP headers")
        .glyph("T")
        .detail("#1050 · mira")
        .group("Tasks"),
    Item::new(ItemKey::Num(206), "Migrate sessions table to UUID keys")
        .glyph("T")
        .detail("#1041 · jonas")
        .group("Tasks"),
    Item::new(ItemKey::Num(207), "Remove legacy feature flags")
        .glyph("T")
        .detail("#1048 · ana")
        .group("Tasks"),
    Item::new(ItemKey::Num(208), "Replace deprecated Vue mixins")
        .glyph("T")
        .detail("#1044 · kai")
        .group("Tasks"),
    Item::new(ItemKey::Num(209), "Speed up cold start of worker")
        .glyph("T")
        .detail("#1051 · jonas")
        .group("Tasks"),
    Item::new(ItemKey::Num(210), "Upgrade Postgres driver to 0.9")
        .glyph("T")
        .detail("#1045 · jonas")
        .group("Tasks"),
    Item::new(ItemKey::Num(211), "Write release notes for 3.2")
        .glyph("T")
        .detail("#1043 · mira")
        .group("Tasks"),
];

const TABS_ITEMS: &[Item<'static>] = &[
    Item::new(ItemKey::Num(400), "Query 1")
        .glyph("≡")
        .detail("query")
        .group("Open tabs"),
    Item::new(ItemKey::Num(401), "orders")
        .glyph("T")
        .detail("public · data")
        .tag("active")
        .group("Open tabs"),
    Item::new(ItemKey::Num(402), "order_items")
        .glyph("T")
        .detail("public · data")
        .group("Open tabs"),
    Item::new(ItemKey::Num(403), "History")
        .glyph("T")
        .detail("public · data")
        .group("Open tabs"),
];

const LEVEL_ITEMS: &[Item<'static>] = &[
    Item::new(ItemKey::Num(500), "Silent")
        .detail("Writes run without asking. Destructive statements still confirm."),
    Item::new(ItemKey::Num(501), "Alert")
        .detail("Every write asks for confirmation before it runs."),
    Item::new(ItemKey::Num(502), "Alert (Full)")
        .detail("Every statement, reads included, asks for confirmation."),
    Item::new(ItemKey::Num(503), "Safe Mode")
        .detail("Writes ask for confirmation and a deliberate acknowledgement.")
        .tag("current"),
    Item::new(ItemKey::Num(504), "Safe Mode (Full)")
        .detail("Every statement asks for confirmation and a deliberate acknowledgement."),
    Item::new(ItemKey::Num(505), "Read-Only")
        .detail("Writes are refused. Reads and exports still work."),
];
const REFERENCE_ITEMS: &[Item<'static>] = &[
    Item::new(ItemKey::Num(1), "Deploy production")
        .glyph("▶")
        .detail("release pipeline")
        .tag("run")
        .group("Actions"),
    Item::new(ItemKey::Num(2), "Open pull request")
        .glyph("↗")
        .detail("review changes")
        .tag("review")
        .group("Actions"),
    Item::new(ItemKey::Num(3), "Inspect logs")
        .glyph("≡")
        .detail("workspace output")
        .tag("debug")
        .group("Navigation"),
    Item::new(ItemKey::Num(4), "Rotate credentials")
        .glyph("◆")
        .detail("security settings")
        .tag("secure")
        .group("Navigation"),
    Item::new(ItemKey::Num(5), "Delete branch")
        .glyph("×")
        .detail("destructive action")
        .tag("danger")
        .disabled(true)
        .group("Actions"),
];
const MENU_ITEMS: &[MenuItem<'static>] = &[
    MenuItem::new(ActionKey::application("showcase.menu.open"), "Open")
        .chord(termrock::Chord::key(termrock::KeyCode::Char('o'))),
    MenuItem::new(ActionKey::application("showcase.menu.close"), "Close"),
];
const MENUS: &[Menu<'static>] = &[Menu::new("Actions", MENU_ITEMS)];
const CONTEXT_ITEMS: &[MenuItem<'static>] = &[
    MenuItem::new(
        ActionKey::application("showcase.context.inspect"),
        "Inspect",
    ),
    MenuItem::new(ActionKey::application("showcase.context.copy"), "Copy path"),
];
const CHAIN_STAGES: &[PickerStage<'static>] = &[
    PickerStage::new(ItemKey::Num(301), "Scope"),
    PickerStage::new(ItemKey::Num(302), "Command"),
    PickerStage::new(ItemKey::Num(303), "Result"),
];

fn picker(
    kind: PickerKind,
    item_count: usize,
    scope: QuickScope,
) -> Picker<'static, Item<'static>> {
    let (title, placeholder, searchable, width, height, footer) = match kind {
        PickerKind::Quick => {
            let rows = item_count.clamp(1, 12) as u16;
            (
                "Open quickly",
                "Files and tasks…",
                true,
                80,
                2u16.saturating_add(1)
                    .saturating_add(2)
                    .saturating_add(rows)
                    .saturating_add(2),
                "↑↓ Move · Enter Open · Alt+Enter New tab · Tab Scope · Esc Clear / Close",
            )
        }
        PickerKind::Tabs => (
            "Open tabs",
            "Filter tabs…",
            true,
            48,
            11,
            "↑↓ Move · Enter Switch · Delete Close tab · Esc Close",
        ),
        PickerKind::Level => (
            "Safe Mode · this connection",
            "",
            false,
            112,
            11,
            "↑↓ Move · Enter Set level · Esc Keep",
        ),
    };
    let meta = if matches!(kind, PickerKind::Quick) {
        match scope {
            QuickScope::All => Some("All · Tab scope"),
            QuickScope::Files => Some("Files · Tab scope"),
            QuickScope::Tasks => Some("Tasks · Tab scope"),
        }
    } else {
        None
    };
    let mut picker = Picker::new(PICKER)
        .title(title)
        .placeholder(placeholder)
        .searchable(searchable)
        .size(LayerSize::Fixed(width, height))
        .align(ScreenAlign::UpperThird)
        .item_layout(ItemRowLayout::Columns)
        .footer(footer);
    if let Some(meta) = meta {
        picker = picker.meta(meta);
    }
    if matches!(kind, PickerKind::Quick) {
        picker.scopes(SCOPES)
    } else {
        picker
    }
}

fn quick_button() -> Button<'static> {
    Button::new(OPEN_QUICK, "Open quickly").variant(Variant::PRIMARY)
}

fn tabs_button() -> Button<'static> {
    Button::new(OPEN_TABS, "Switch tab").variant(Variant::SECONDARY)
}

fn level_button() -> Button<'static> {
    Button::new(OPEN_LEVEL, "Choose a level").variant(Variant::SECONDARY)
}

fn filter_list() -> FilterList<'static, Item<'static>> {
    FilterList::new(FILTER)
}

fn picker_chain() -> PickerChain<'static> {
    PickerChain::new(CHAIN, CHAIN_STAGES)
}

fn menu_bar() -> MenuBar<'static> {
    MenuBar::new(MENU, MENUS)
}

fn context_menu() -> ContextMenu<'static> {
    ContextMenu::at(CONTEXT, CONTEXT_ITEMS, Position::new(0, 0)).title("Context")
}

/// The picker owns query/cursor state while the app owns the selected result.
#[derive(Debug, Default)]
pub struct PickersPage {
    state: PickerState,
    filter_state: FilterListState,
    chain_state: PickerChainState,
    menu_state: MenuState,
    context_state: MenuState,
    open_kind: Option<PickerKind>,
    quick_scope: QuickScope,
    items: Vec<Item<'static>>,
    tabs: Vec<Item<'static>>,
    result: String,
    detail: String,
    level: usize,
    opened: u32,
}

impl PickersPage {
    pub fn new() -> Self {
        Self {
            state: PickerState::default(),
            filter_state: FilterListState::default(),
            chain_state: PickerChainState::default(),
            menu_state: MenuState::default(),
            context_state: MenuState::default(),
            open_kind: None,
            quick_scope: QuickScope::All,
            items: QUICK_ITEMS.to_vec(),
            tabs: TABS_ITEMS.to_vec(),
            result: String::from("none"),
            detail: String::new(),
            level: 3,
            opened: 0,
        }
    }

    fn rebuild_quick_items(&mut self) {
        self.items = QUICK_ITEMS
            .iter()
            .copied()
            .filter(|item| match self.quick_scope {
                QuickScope::All => true,
                QuickScope::Files => item.group == Some("Files"),
                QuickScope::Tasks => item.group == Some("Tasks"),
            })
            .collect();
    }

    fn items(&self, kind: PickerKind) -> &[Item<'static>] {
        match kind {
            PickerKind::Quick => &self.items,
            PickerKind::Tabs => &self.tabs,
            PickerKind::Level => LEVEL_ITEMS,
        }
    }

    fn open_picker(&mut self, cx: &mut Cx<'_>, kind: PickerKind) {
        self.state = PickerState::default();
        self.open_kind = Some(kind);
        if kind == PickerKind::Quick {
            // The scope persists across opens (S4 PICKER-QUERY-001 N1
            // live): only the query/cursor state resets per open.
            self.rebuild_quick_items();
        }
        if kind == PickerKind::Level
            && let Some(item) = LEVEL_ITEMS.get(self.level)
        {
            self.state.set_cursor(self.level, item.key);
        }
        self.opened = self.opened.saturating_add(1);
        let items = match kind {
            PickerKind::Quick => self.items.as_slice(),
            PickerKind::Tabs => self.tabs.as_slice(),
            PickerKind::Level => LEVEL_ITEMS,
        };
        let spec = picker(kind, items.len(), self.quick_scope).layer(cx, items);
        cx.open_layer(PICKER, spec);
    }
}

impl Page for PickersPage {
    fn title(&self) -> &'static str {
        "Pickers"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let mut result = Response::ignored();
        let quick = quick_button().update(cx);
        if quick.activated() && !cx.is_open(PICKER) {
            self.open_picker(cx, PickerKind::Quick);
        }
        result |= quick.erase();
        let tabs = tabs_button().update(cx);
        if tabs.activated() && !cx.is_open(PICKER) {
            self.open_picker(cx, PickerKind::Tabs);
        }
        result |= tabs.erase();
        let level = level_button().update(cx);
        if level.activated() && !cx.is_open(PICKER) {
            self.open_picker(cx, PickerKind::Level);
        }
        result |= level.erase();
        result |= filter_list()
            .update(cx, &mut self.filter_state, REFERENCE_ITEMS)
            .erase();
        result |= picker_chain().update(cx, &mut self.chain_state).erase();
        result |= menu_bar().update(cx, &mut self.menu_state).erase();
        result |= context_menu().update(cx, &mut self.context_state).erase();
        // Drain the layer owner on every pass. A dismissal event is addressed
        // to PICKER after the layer has left the active stack; gating update
        // on `is_open` would strand that event and trip the runtime diagnostic.
        let kind = self.open_kind.unwrap_or(PickerKind::Quick);
        let items = match kind {
            PickerKind::Quick => self.items.as_slice(),
            PickerKind::Tabs => self.tabs.as_slice(),
            PickerKind::Level => LEVEL_ITEMS,
        };
        let was_open = cx.is_open(PICKER);
        let action = picker(kind, items.len(), self.quick_scope).update(cx, &mut self.state, items);
        let action_value = action.action_ref().copied();
        result |= action.erase();
        let mut status = None;
        if let Some(action) = action_value {
            match action {
                PickerAction::Chosen(key) | PickerAction::ChosenAlt(key) => {
                    let selected = self
                        .items(kind)
                        .iter()
                        .find(|item| item.key == key)
                        .map(|item| (item.label, item.detail));
                    if let Some((label, detail)) = selected {
                        self.result = label.to_owned();
                        self.detail = detail.to_owned();
                        let prefix = if matches!(action, PickerAction::ChosenAlt(_)) {
                            "Opened in a new tab:"
                        } else {
                            "Chose"
                        };
                        status = Some(PageStatus(format!("{prefix} {label}")));
                    }
                    if kind == PickerKind::Level
                        && let Some(index) = LEVEL_ITEMS.iter().position(|item| item.key == key)
                    {
                        self.level = index;
                    }
                    cx.close_layer(PICKER, None);
                }
                PickerAction::Secondary(key) if kind == PickerKind::Tabs => {
                    // Tabs: Delete closes the row and the modal stays open;
                    // the last remaining tab is refused (S4 PICKER-TABS-002
                    // S1/A1 live).
                    let pos = (self.tabs.len() > 1)
                        .then(|| self.tabs.iter().position(|item| item.key == key))
                        .flatten();
                    if let Some(pos) = pos {
                        let label = self.tabs[pos].label.to_owned();
                        self.tabs.remove(pos);
                        status = Some(PageStatus(format!("Closed {label}")));
                    }
                }
                PickerAction::Scope(scope) if kind == PickerKind::Quick => {
                    self.quick_scope = match scope.get() {
                        2 => QuickScope::Files,
                        3 => QuickScope::Tasks,
                        _ => QuickScope::All,
                    };
                    self.rebuild_quick_items();
                }
                PickerAction::QueryChanged
                | PickerAction::Back
                | PickerAction::Scope(_)
                | PickerAction::Secondary(_) => {}
            }
        }
        if was_open && !cx.is_open(PICKER) {
            // Historical post-modal focus: choosing or cancelling a picker
            // returns focus to the shell navigation, so one Tab reaches
            // Quick again (S4 PICKER-QUERY-001 / PICKER-TABS-002 live).
            cx.focus(SHELL_NAV);
        }
        PageUpdate {
            response: result,
            status,
        }
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "One modal list for files, tabs and levels: search, scope, tag, alternate action",
            |ui, body| {
                let launcher_area = Rect { height: 7, ..body };
                Panel::new(LAUNCHER_PANEL)
                    .title("Open a picker")
                    .kind(PanelKind::Card)
                    .patch_part(PANEL_PARTS)
                    .draw(ui, launcher_area, |ui, inner| {
                        let btn1 = quick_button().draw(
                            ui,
                            Rect {
                                x: inner.x,
                                y: inner.y,
                                width: inner.width,
                                height: 1,
                            },
                        );
                        let btn2_x = btn1.right().saturating_add(2);
                        let btn2 = tabs_button().draw(
                            ui,
                            Rect {
                                x: btn2_x,
                                y: inner.y,
                                width: inner.right().saturating_sub(btn2_x),
                                height: 1,
                            },
                        );
                        let btn3_x = btn2.right().saturating_add(2);
                        let _btn3 = level_button().draw(
                            ui,
                            Rect {
                                x: btn3_x,
                                y: inner.y,
                                width: inner.right().saturating_sub(btn3_x),
                                height: 1,
                            },
                        );
                        super::lines(
                            ui,
                            Rect {
                                x: inner.x,
                                y: inner.y.saturating_add(2),
                                width: inner.width,
                                height: 1,
                            },
                            &["Quick: fuzzy over files and tasks, Tab cycles the scope, Alt+Enter is the alternate action · Tabs: Delete closes a row · Level: no search box"],
                        );
                    });

                let level = LEVEL_ITEMS
                    .get(self.level)
                    .map_or("Safe Mode", |item| item.label);
                let opened_str = self.opened.to_string();
                let prop_rows = [
                    (
                        "Chosen",
                        if self.result == "none" {
                            "nothing yet"
                        } else {
                            self.result.as_str()
                        },
                    ),
                    (
                        "Detail",
                        if self.detail.is_empty() {
                            "—"
                        } else {
                            self.detail.as_str()
                        },
                    ),
                    ("Level", level),
                    ("Open tabs", "Query 1 · orders · order_items · History"),
                    ("Pickers opened", opened_str.as_str()),
                ];
                let results_area = Rect {
                    y: body.y.saturating_add(8),
                    height: body.height.saturating_sub(8).min(8),
                    ..body
                };
                Panel::new(RESULTS_PANEL)
                    .title("Result")
                    .kind(PanelKind::Card)
                    .patch_part(PANEL_PARTS)
                    .draw(ui, results_area, |ui, inner| {
                        Props::new(&prop_rows).draw(ui, inner);
                    });
            },
        );
        if let Some(kind) = self.open_kind {
            ui.layer(PICKER, |ui, layer| {
                let items = self.items(kind);
                picker(kind, items.len(), self.quick_scope).draw(ui, layer, &self.state, items);
            });
        }
    }

    fn hints(&self, _ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        if self.open_kind.is_some() {
            &[("Esc", "Close")]
        } else {
            &[("Enter", "Open")]
        }
    }
}

pub fn action_keys() -> impl Iterator<Item = ActionKey> {
    MENU_ITEMS.iter().chain(CONTEXT_ITEMS).map(MenuItem::action)
}
