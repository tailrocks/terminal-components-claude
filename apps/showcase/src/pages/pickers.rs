//! Searchable semantic picker with query and scope state.

use junie_tui::{
    ActionKey, Button, ContextMenu, Cx, FilterList, FilterListState, Id, Item, ItemKey, Menu,
    MenuBar, MenuItem, MenuState, Picker, PickerAction, PickerChain, PickerChainState, PickerStage,
    PickerState, Position, Rect, Response, Ui, Variant, id, layout,
};

use super::{Page, frame, lines};

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
        if cx.is_open(PICKER) {
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
        }
        result
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "semantic items · query · scope-aware modal",
            |ui, body| {
                let (top, inventory) = layout::split_v(body, body.height.saturating_sub(12));
                let (button, note) = layout::split_v(top, 4);
                open_button().draw(ui, button);
                let query = format!(
                    "last result: {} · query: {}",
                    self.result,
                    self.state.query()
                );
                let _ = ui.paint_str(note, &query, ui.surface_style());
                lines(
                    ui,
                    Rect {
                        y: note.y.saturating_add(1),
                        height: note.height.saturating_sub(1),
                        ..note
                    },
                    &["The picker filters semantic labels and returns stable ItemKey values."],
                );
                let (menu_area, collections) = layout::split_v(inventory, 1);
                menu_bar().draw(ui, menu_area, &self.menu_state);
                let (left, right) = layout::split_h(collections, collections.width / 2);
                let (filter_area, chain_area) =
                    layout::split_v(left, left.height.saturating_sub(2));
                filter_list().draw(ui, filter_area, &self.filter_state, ITEMS);
                picker_chain().draw(ui, chain_area, &self.chain_state);
                context_menu().draw(ui, right, &self.context_state);
            },
        );
        ui.layer(PICKER, |ui, layer| {
            picker().draw(ui, layer, &self.state, ITEMS);
        });
    }
}
