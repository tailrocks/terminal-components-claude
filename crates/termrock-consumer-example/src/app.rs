//! The external consumer application: a recipe shelf built only from the
//! public Termrock APIs.
//!
//! The screen is composed as
//!
//! ```text
//! AppShell
//!  ├── header : MenuBar        (termrock-overlays)
//!  ├── body   : MasterDetail   (termrock blocks)
//!  │    ├── master  : NavList  (termrock-navigation)
//!  │    ├── detail  : PropsList (termrock-controls)
//!  │    └── summary : Button   (termrock-controls)
//!  └── footer : StatusBar      (termrock-feedback)
//! ```
//!
//! Every layout decision belongs to a shared component: the consumer never
//! computes a rectangle, never paints a cell and never matches on a raw key
//! or mouse event. Input is routed by the shared widgets, which answer with
//! typed actions that the consumer maps onto its own domain actions.

use termrock::author::{CompositionLeaf, Constraints, Measure, Size};
use termrock::blocks::{MasterDetail, MasterDetailAction, MasterDetailPolicy, MasterDetailState};
use termrock::composition::{self, Group};
use termrock::controls::{Button, PropsList, PropsRow, PropsState};
use termrock::feedback::{StatusAction, StatusBar, StatusItem};
use termrock::navigation::{NavList, NavListAction, NavListState};
use termrock::overlays::{Menu, MenuAction, MenuBar, MenuItem, MenuState};
use termrock::{ActionKey, App, Cx, FrameRead, Id, ItemKey, Rect, Response, RowUi, Ui, UpdateCause};

use crate::recipes::{RECIPES, Recipe};

/// The master-detail workspace that owns the screen layout.
pub const WORKSPACE: Id = Id::root("recipe-box.workspace");
/// The menu bar in the shell header.
pub const MENU_BAR: Id = Id::root("recipe-box.menu");
/// The recipe list in the master pane.
pub const RECIPE_LIST: Id = Id::root("recipe-box.recipes");
/// The property sheet in the detail pane.
pub const RECIPE_FACTS: Id = Id::root("recipe-box.facts");
/// The summary row below the master-detail panes.
pub const SCALE_BUTTON: Id = Id::root("recipe-box.scale");
/// The summary row's second action.
pub const COOK_BUTTON: Id = Id::root("recipe-box.cook");
/// The status bar in the shell footer.
pub const STATUS_BAR: Id = Id::root("recipe-box.status");
/// The keyed status item that carries the note.
pub const NOTE_ITEM: ItemKey = ItemKey::num(900);

/// The menu action that scales the selected recipe.
pub const MENU_SCALE: ActionKey = ActionKey::custom("recipe-box.menu.scale");
/// The menu action that shows the about note.
pub const MENU_ABOUT: ActionKey = ActionKey::custom("recipe-box.menu.about");

/// The status note shown in the shell footer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Note {
    /// Nothing has happened yet.
    Ready,
    /// The selected recipe was scaled.
    Scaled,
    /// The selected recipe was marked as cooked.
    Cooked,
    /// The about note was requested.
    About,
}

impl Note {
    /// The static text this note renders as.
    pub const fn text(self) -> &'static str {
        match self {
            Note::Ready => "Ready",
            Note::Scaled => "Servings scaled",
            Note::Cooked => "Marked as cooked",
            Note::About => "Recipe Box · external consumer",
        }
    }
}

/// The consumer's own typed domain actions.
///
/// Shared widgets answer with their own typed actions; the consumer maps them
/// onto these so the domain never depends on a widget's vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// The list cursor moved onto a recipe.
    Focused(ItemKey),
    /// The user chose a recipe.
    Chose(ItemKey),
    /// The detail pane opened in the narrow layout.
    DetailOpened,
    /// The detail pane closed in the narrow layout.
    DetailClosed,
    /// A menu item was chosen.
    MenuChosen(ActionKey),
    /// The status bar reported a chosen item.
    StatusChosen(ItemKey),
    /// The servings of a recipe were scaled.
    Scaled(ItemKey),
    /// A recipe was marked as cooked.
    Cooked(ItemKey),
}

const MENUS: &[Menu<'static>] = &[
    Menu::new(
        "Recipe",
        &[
            MenuItem::new(MENU_SCALE, "Scale servings"),
            MenuItem::new(MENU_ABOUT, "About Recipe Box"),
        ],
    ),
    Menu::new("Help", &[MenuItem::new(MENU_ABOUT, "Key reference")]),
];

type RecipeNav = NavList<'static, Recipe, fn(&Recipe) -> ItemKey, fn(&Recipe, &mut RowUi<'_>)>;

fn recipe_key(recipe: &Recipe) -> ItemKey {
    recipe.key()
}

fn recipe_row(recipe: &Recipe, ui: &mut RowUi<'_>) {
    ui.label(recipe.name);
}

fn recipe_course(recipe: &Recipe) -> &str {
    recipe.course
}

/// The recipe shelf application.
#[derive(Debug)]
pub struct RecipeBox {
    recipes: Vec<Recipe>,
    list_state: NavListState,
    facts_state: PropsState,
    layout: MasterDetailState,
    menu_state: MenuState,
    selected: Option<ItemKey>,
    servings: u16,
    cooked: bool,
    note: Note,
    actions: Vec<Action>,
}

impl Default for RecipeBox {
    fn default() -> Self {
        Self {
            recipes: RECIPES.to_vec(),
            list_state: NavListState::new(),
            facts_state: PropsState::default(),
            layout: MasterDetailState::default(),
            menu_state: MenuState::default(),
            selected: RECIPES.first().map(Recipe::key),
            servings: 2,
            cooked: false,
            note: Note::Ready,
            actions: Vec::new(),
        }
    }
}

impl RecipeBox {
    /// The typed actions this consumer has produced, oldest first.
    pub fn actions(&self) -> &[Action] {
        &self.actions
    }

    /// The most recent typed action, if any.
    pub fn last_action(&self) -> Option<Action> {
        self.actions.last().copied()
    }

    /// The recipe the list cursor currently addresses.
    pub fn selected(&self) -> Option<ItemKey> {
        self.selected
    }

    /// The servings of the selected recipe.
    pub const fn servings(&self) -> u16 {
        self.servings
    }

    /// Whether the selected recipe is marked as cooked.
    pub const fn cooked(&self) -> bool {
        self.cooked
    }

    /// The note the status bar renders.
    pub const fn note(&self) -> Note {
        self.note
    }

    /// The recipes on the shelf.
    pub fn recipes(&self) -> &[Recipe] {
        &self.recipes
    }

    fn selected_recipe(&self) -> Option<&Recipe> {
        let key = self.selected?;
        self.recipes.iter().find(|recipe| recipe.key() == key)
    }

    fn dispatch(&mut self, action: Action) {
        match action {
            Action::Focused(key) => {
                self.selected = Some(key);
                self.servings = 2;
                self.cooked = false;
            }
            Action::Chose(key) => {
                self.selected = Some(key);
                self.servings = 2;
                self.cooked = false;
            }
            Action::DetailOpened | Action::DetailClosed => {}
            Action::MenuChosen(key) => {
                if key == MENU_SCALE {
                    self.scale();
                } else {
                    self.note = Note::About;
                }
            }
            Action::StatusChosen(_) => {}
            Action::Scaled(key) => {
                self.selected = Some(key);
                self.scale();
            }
            Action::Cooked(key) => {
                self.selected = Some(key);
                self.cooked = true;
                self.note = Note::Cooked;
            }
        }
        self.actions.push(action);
    }

    fn scale(&mut self) {
        self.servings = self.servings.saturating_add(1);
        self.note = Note::Scaled;
    }

    fn policy(&self) -> MasterDetailPolicy {
        MasterDetailPolicy::default()
            .detail_share(40, 12, 30)
            .gap(2)
            .collapse_below(60)
            .summary_rows(1)
    }

    fn nav(&self) -> RecipeNav {
        NavList::new(RECIPE_LIST)
            .key(recipe_key as fn(&Recipe) -> ItemKey)
            .row(recipe_row as fn(&Recipe, &mut RowUi<'_>))
            .section(&recipe_course)
            .scrollable(true)
    }

    fn facts(&self) -> PropsList<'static> {
        PropsList::new(RECIPE_FACTS).sheet()
    }

    fn menu(&self) -> MenuBar<'static> {
        MenuBar::new(MENU_BAR, MENUS)
    }

    fn fact_rows(&self) -> Vec<PropsRow<'static>> {
        let Some(recipe) = self.selected_recipe() else {
            return Vec::new();
        };
        recipe
            .facts
            .iter()
            .map(|fact| {
                PropsRow::new(
                    ItemKey::num(recipe.id.saturating_mul(16).saturating_add(fact.key)),
                    fact.label,
                    fact.value,
                )
            })
            .collect()
    }

    fn status_texts(&self) -> (&'static str, &'static str, &'static str) {
        let selected = self
            .selected_recipe()
            .map_or("No recipe selected", |recipe| recipe.name);
        ("Recipe Box", selected, self.note.text())
    }
}

impl App for RecipeBox {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = Response::ignored();

        // The recipe list owns the keyboard when the shelf opens.
        if cx.update_cause() == UpdateCause::Bootstrap {
            cx.focus(RECIPE_LIST);
        }

        // The layout block gets first refusal on Escape.
        let block = self.block();
        let block_response = block.update(cx, &mut self.layout);
        if let Some(action) = block_response.action_ref().copied() {
            self.dispatch(match action {
                MasterDetailAction::DrawerOpened => Action::DetailOpened,
                MasterDetailAction::DrawerClosed => Action::DetailClosed,
            });
        }
        if block_response.is_consumed() {
            return block_response.erase();
        }
        response |= block_response.erase();

        let nav_response = self.nav().update(cx, &mut self.list_state, &self.recipes);
        if let Some(action) = nav_response.action_ref().copied() {
            let mapped = match action {
                NavListAction::Moved(key) | NavListAction::EnterContent(key) => {
                    Some(Action::Focused(key))
                }
                NavListAction::Chose(key) => Some(Action::Chose(key)),
                NavListAction::LeaveBackward | NavListAction::LeaveForward => None,
            };
            if let Some(action) = mapped {
                self.dispatch(action);
            }
            if matches!(action, NavListAction::Chose(_) | NavListAction::EnterContent(_))
                && cx
                    .layout(WORKSPACE)
                    .and_then(|facts| facts.logical_area)
                    .is_some_and(|area| self.policy().is_collapsed(area.width))
            {
                let block = self.block();
                let opened = block.open_detail(cx, &mut self.layout);
                if opened.action_ref().is_some() {
                    self.dispatch(Action::DetailOpened);
                }
                response |= opened.erase();
            }
        }
        response |= nav_response.erase();

        // The detail pane always shows the recipe the list cursor addresses.
        if let Some(key) = self.list_state.cursor() {
            self.selected = Some(key);
        }

        let rows = self.fact_rows();
        let facts_response = self.facts().update(cx, &mut self.facts_state, &rows);
        response |= facts_response.erase();

        let menu_response = self.menu().update(cx, &mut self.menu_state);
        if let Some(MenuAction::Chosen(key)) = menu_response.action_ref().copied() {
            self.dispatch(Action::MenuChosen(key));
        }
        response |= menu_response.erase();

        let (left_text, center_text, right_text) = self.status_texts();
        let left = [StatusItem::new(left_text)];
        let center = [StatusItem::new(center_text)];
        let right = [StatusItem::new(right_text).key(NOTE_ITEM)];
        let status = StatusBar::new(STATUS_BAR)
            .left(&left)
            .center(&center)
            .right(&right);
        let status_response = status.update(cx);
        if let Some(StatusAction::Chose(key)) = status_response.action_ref().copied() {
            self.dispatch(Action::StatusChosen(key));
        }
        response |= status_response.erase();

        let scale = Button::new(SCALE_BUTTON, "Scale servings");
        let scale_response = scale.update(cx);
        if scale_response.activated()
            && let Some(key) = self.selected
        {
            self.dispatch(Action::Scaled(key));
        }
        response |= scale_response.erase();
        let cook = Button::new(COOK_BUTTON, "Mark cooked");
        let cook_response = cook.update(cx);
        if cook_response.activated()
            && let Some(key) = self.selected
        {
            self.dispatch(Action::Cooked(key));
        }
        response |= cook_response.erase();

        response
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let nav = self.nav();
        let master =
            Group::column([composition::nav_list(&nav, &self.list_state, &self.recipes).flex(1)]);

        let rows = self.fact_rows();
        let facts = self.facts();
        let scale = Button::new(SCALE_BUTTON, "Scale servings");
        let cook = Button::new(COOK_BUTTON, "Mark cooked");
        let detail = Group::column([
            composition::props_list(&facts, &self.facts_state, &rows).flex(1),
            Group::row([composition::button(&scale), composition::button(&cook)]).into_child(),
        ]);

        let block = self.block().master(&master).detail(&detail);

        let (left_text, center_text, right_text) = self.status_texts();
        let left = [StatusItem::new(left_text)];
        let center = [StatusItem::new(center_text)];
        let right = [StatusItem::new(right_text).key(NOTE_ITEM)];
        let status = StatusBar::new(STATUS_BAR)
            .left(&left)
            .center(&center)
            .right(&right);
        let menu = self.menu();

        let header = MenuLeaf {
            bar: &menu,
            state: &self.menu_state,
        };
        let body = Group::column([
            composition::master_detail(&block, &self.layout)
                .expect("recipe master and detail groups are configured"),
        ]);
        let footer = StatusLeaf { bar: &status };

        termrock::AppShell::new()
            .chrome(termrock::Chrome::new().gap(1))
            .show(ui, &header, &body, &footer)
            .expect("recipe shell has a valid fixed composition");
    }
}

impl RecipeBox {
    fn block(&self) -> MasterDetail<'static> {
        MasterDetail::new(WORKSPACE, RECIPE_LIST, RECIPE_FACTS).policy(self.policy())
    }
}

/// The shell header leaf: the shared menu bar.
struct MenuLeaf<'a> {
    bar: &'a MenuBar<'a>,
    state: &'a MenuState,
}

impl Measure for MenuLeaf<'_> {
    fn measure(&self, ui: &Ui<'_>, constraints: Constraints) -> Size {
        self.bar.measure(ui, constraints)
    }
}

impl CompositionLeaf for MenuLeaf<'_> {
    fn draw(&self, ui: &mut Ui<'_>, logical: Rect) {
        self.bar.draw(ui, logical, self.state);
    }
}

/// The shell footer leaf: the shared status bar.
struct StatusLeaf<'a> {
    bar: &'a StatusBar<'a>,
}

impl Measure for StatusLeaf<'_> {
    fn measure(&self, ui: &Ui<'_>, constraints: Constraints) -> Size {
        self.bar.measure(ui, constraints)
    }
}

impl CompositionLeaf for StatusLeaf<'_> {
    fn draw(&self, ui: &mut Ui<'_>, logical: Rect) {
        self.bar.draw(ui, logical);
    }
}
