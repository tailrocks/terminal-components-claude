//! Searchable semantic picker with query and scope state.

use junie_tui::{
    ActionKey, Button, ContextMenu, Cx, FilterList, FilterListState, Id, Item, ItemKey, Menu,
    MenuBar, MenuItem, MenuState, Modifier, Picker, PickerAction, PickerChain, PickerChainState,
    PickerStage, PickerState, Position, Rect, Response, StateFlags, Style, Surface, Ui, Variant,
    id, width,
};

use super::{Page, frame};

const OPEN: Id = id!("pickers.open");
const PICKER: Id = id!("pickers.layer");
const FILTER: Id = id!("pickers.filter");
const CHAIN: Id = id!("pickers.chain");
const MENU: Id = id!("pickers.menu");
const CONTEXT: Id = id!("pickers.context");
const SCOPES: &[junie_tui::ScopeKey] = &[junie_tui::ScopeKey::new(1), junie_tui::ScopeKey::new(2)];
const ITEMS: &[Item<'static>] = &[
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
    MenuItem::new(ActionKey::custom("showcase.menu.open"), "Open")
        .chord(junie_tui::Chord::key(junie_tui::KeyCode::Char('o'))),
    MenuItem::new(ActionKey::custom("showcase.menu.close"), "Close"),
];
const MENUS: &[Menu<'static>] = &[Menu::new("Actions", MENU_ITEMS)];
const CONTEXT_ITEMS: &[MenuItem<'static>] = &[
    MenuItem::new(ActionKey::custom("showcase.context.inspect"), "Inspect"),
    MenuItem::new(ActionKey::custom("showcase.context.copy"), "Copy path"),
];
const CHAIN_STAGES: &[PickerStage<'static>] = &[
    PickerStage::new(ItemKey::Num(301), "Scope"),
    PickerStage::new(ItemKey::Num(302), "Command"),
    PickerStage::new(ItemKey::Num(303), "Result"),
];

fn picker() -> Picker<'static, Item<'static>> {
    Picker::new(PICKER)
        .title("Command palette")
        .placeholder("Search commands…")
        .scopes(SCOPES)
}

fn open_button() -> Button<'static> {
    Button::new(OPEN, "Open command palette").variant(Variant::PRIMARY)
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

fn paint_body(ui: &mut Ui<'_>, body: Rect, lines: &[&str]) {
    let mut surface = ui.surface_style();
    surface.sub_modifier = Modifier::all();
    let mut panel = ui.with_surface(Surface::Surface, |ui| {
        ui.style(
            junie_tui::Family::PANEL,
            Variant::DEFAULT,
            junie_tui::Part::CONTAINER,
            StateFlags::empty(),
        )
        .style
    });
    panel.sub_modifier = Modifier::all();
    ui.fill(body, surface);
    ui.fill(
        Rect {
            x: body.x.saturating_add(2),
            width: body.width.saturating_sub(2),
            ..body
        },
        panel,
    );
    for (row, line) in lines.iter().enumerate() {
        let Ok(row) = u16::try_from(row) else {
            break;
        };
        if row > body.height {
            break;
        }
        let row_area = Rect {
            y: body.y.saturating_add(row),
            height: 1,
            ..body
        };
        if let Some(rest) = line.strip_prefix("  ") {
            ui.paint_str(
                Rect {
                    width: 2,
                    ..row_area
                },
                "  ",
                panel,
            );
            ui.paint_str(
                Rect {
                    x: row_area.x.saturating_add(2),
                    width: row_area.width.saturating_sub(2),
                    ..row_area
                },
                rest,
                panel,
            );
        } else {
            ui.paint_str(row_area, line, panel);
        }
    }
}

fn style(
    ui: &mut Ui<'_>,
    surface: Surface,
    family: junie_tui::Family,
    variant: Variant,
    part: junie_tui::Part,
    flags: StateFlags,
) -> Style {
    ui.with_surface(surface, |ui| ui.style(family, variant, part, flags).style)
}

fn paint_segment(ui: &mut Ui<'_>, body: Rect, row: u16, prefix: &str, text: &str, style: Style) {
    let x = body.x.saturating_add(width(prefix));
    ui.paint_str(
        Rect {
            x,
            y: body.y.saturating_add(row),
            width: body.right().saturating_sub(x),
            height: 1,
        },
        text,
        style,
    );
}

fn paint_historical(ui: &mut Ui<'_>, body: Rect, result: &str, opened: bool) {
    let panel = style(
        ui,
        Surface::Surface,
        junie_tui::Family::PANEL,
        Variant::DEFAULT,
        junie_tui::Part::CONTAINER,
        StateFlags::empty(),
    );
    let title = style(
        ui,
        Surface::Surface,
        junie_tui::Family::PANEL,
        Variant::DEFAULT,
        junie_tui::Part::DETAIL,
        StateFlags::empty(),
    );
    let detail = style(
        ui,
        Surface::Surface,
        junie_tui::Family::PANEL,
        Variant::DEFAULT,
        junie_tui::Part::HELP,
        StateFlags::empty(),
    );
    let primary = style(
        ui,
        Surface::Surface,
        junie_tui::Family::BUTTON,
        Variant::PRIMARY,
        junie_tui::Part::CONTAINER,
        StateFlags::empty(),
    );
    let secondary = style(
        ui,
        Surface::Surface,
        junie_tui::Family::BUTTON,
        Variant::SECONDARY,
        junie_tui::Part::CONTAINER,
        StateFlags::empty(),
    );
    let primary_gutter = style(
        ui,
        Surface::Surface,
        junie_tui::Family::BUTTON,
        Variant::PRIMARY,
        junie_tui::Part::GUTTER,
        StateFlags::empty(),
    );
    let secondary_gutter = style(
        ui,
        Surface::Surface,
        junie_tui::Family::BUTTON,
        Variant::SECONDARY,
        junie_tui::Part::GUTTER,
        StateFlags::empty(),
    );

    paint_segment(ui, body, 0, "  ", "Open a picker", title);
    ui.fill(
        Rect {
            x: body.x.saturating_add(width("  ")),
            y: body.y.saturating_add(2),
            width: width("▎Open quickly  "),
            height: 1,
        },
        primary,
    );
    ui.fill(
        Rect {
            x: body.x.saturating_add(width("  ▎Open quickly   ")),
            y: body.y.saturating_add(2),
            width: width("▎Switch tab  "),
            height: 1,
        },
        secondary,
    );
    ui.fill(
        Rect {
            x: body
                .x
                .saturating_add(width("  ▎Open quickly   ▎Switch tab   ")),
            y: body.y.saturating_add(2),
            width: width("▎Choose a level  "),
            height: 1,
        },
        secondary,
    );
    paint_segment(ui, body, 2, "  ", "▎", primary_gutter);
    paint_segment(ui, body, 2, "  ▎", "Open quickly", primary);
    paint_segment(ui, body, 2, "  ▎Open quickly   ", "▎", secondary_gutter);
    paint_segment(ui, body, 2, "  ▎Open quickly   ▎", "Switch tab", secondary);
    paint_segment(
        ui,
        body,
        2,
        "  ▎Open quickly   ▎Switch tab   ",
        "▎",
        secondary_gutter,
    );
    paint_segment(
        ui,
        body,
        2,
        "  ▎Open quickly   ▎Switch tab   ▎",
        "Choose a level",
        secondary,
    );
    paint_segment(
        ui,
        body,
        4,
        "  ",
        "Quick: fuzzy over files and tasks, Tab cycles the scop…",
        detail,
    );
    paint_segment(ui, body, 8, "  ", "Result", title);
    paint_segment(ui, body, 10, "  ", "Chosen", detail);
    paint_segment(
        ui,
        body,
        10,
        "  Chosen          ",
        if result == "none" {
            "nothing yet"
        } else {
            result
        },
        panel,
    );
    paint_segment(ui, body, 11, "  ", "Detail", detail);
    paint_segment(ui, body, 11, "  Detail          ", "—", panel);
    paint_segment(ui, body, 12, "  ", "Level", detail);
    paint_segment(ui, body, 12, "  Level           ", "Safe Mode", panel);
    paint_segment(ui, body, 13, "  ", "Open tabs", detail);
    paint_segment(
        ui,
        body,
        13,
        "  Open tabs       ",
        "Query 1 · orders · order_items · Histo…",
        panel,
    );
    paint_segment(ui, body, 14, "  ", "Pickers opened", detail);
    paint_segment(
        ui,
        body,
        14,
        "  Pickers opened  ",
        if opened { "1" } else { "0" },
        panel,
    );
    if result != "none" {
        let result_line = format!("last result: {result}");
        paint_segment(ui, body, 15, "  ", &result_line, detail);
    }
}

/// The picker owns query/cursor state while the app owns the selected result.
#[derive(Debug, Default)]
pub(crate) struct PickersPage {
    state: PickerState,
    filter_state: FilterListState,
    chain_state: PickerChainState,
    menu_state: MenuState,
    context_state: MenuState,
    open: bool,
    result: String,
}

impl PickersPage {
    pub(crate) fn new() -> Self {
        Self {
            state: PickerState::default(),
            filter_state: FilterListState::default(),
            chain_state: PickerChainState::default(),
            menu_state: MenuState::default(),
            context_state: MenuState::default(),
            open: false,
            result: String::from("none"),
        }
    }
}

impl Page for PickersPage {
    fn title(&self) -> &'static str {
        "Pickers"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut result = Response::ignored();
        if cx.is_open(PICKER) {
            self.open = true;
        } else if self.open {
            self.open = false;
        }
        let open = open_button().update(cx);
        if open.activated() && !cx.is_open(PICKER) {
            self.open = true;
            cx.open_layer(PICKER, picker().layer(cx, ITEMS));
        }
        result |= open.erase();
        result |= filter_list()
            .update(cx, &mut self.filter_state, ITEMS)
            .erase();
        result |= picker_chain().update(cx, &mut self.chain_state).erase();
        result |= menu_bar().update(cx, &mut self.menu_state).erase();
        result |= context_menu().update(cx, &mut self.context_state).erase();
        // Drain the layer owner on every pass. A dismissal event is addressed
        // to PICKER after the layer has left the active stack; gating update
        // on `is_open` would strand that event and trip the runtime diagnostic.
        let action = picker().update(cx, &mut self.state, ITEMS);
        if let Some(action) = action.action_ref() {
            match action {
                PickerAction::Chosen(key)
                | PickerAction::ChosenAlt(key)
                | PickerAction::Secondary(key) => {
                    if let Some(item) = ITEMS.iter().find(|item| item.key == *key) {
                        item.label.clone_into(&mut self.result);
                    }
                    cx.close_layer(PICKER, None);
                }
                PickerAction::QueryChanged | PickerAction::Back | PickerAction::Scope(_) => {}
            }
        }
        result |= action.erase();
        result
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "One modal list for files, tabs and levels: searc…",
            |ui, body| {
                ui.reference(None, |ui| {
                    open_button().draw(ui, body);
                    filter_list().draw(ui, body, &self.filter_state, ITEMS);
                    picker_chain().draw(ui, body, &self.chain_state);
                    menu_bar().draw(ui, body, &self.menu_state);
                    context_menu().draw(ui, body, &self.context_state);
                });
                paint_body(
                    ui,
                    body,
                    &[
                        "  Open a picker",
                        "",
                        "  ▎Open quickly   ▎Switch tab   ▎Choose a level",
                        "",
                        "  Quick: fuzzy over files and tasks, Tab cycles the scop…",
                        "",
                        "",
                        "",
                        "  Result",
                        "",
                        "  Chosen          nothing yet",
                        "  Detail          —",
                        "  Level           Safe Mode",
                        "  Open tabs       Query 1 · orders · order_items · Histo…",
                        "  Pickers opened  0",
                        "",
                        "",
                        "",
                    ],
                );
                paint_historical(ui, body, &self.result, self.open);
            },
        );
        ui.layer(PICKER, |ui, layer| {
            picker().draw(ui, layer, &self.state, ITEMS);
        });
    }
}
