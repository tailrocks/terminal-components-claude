//! Collection fixtures: list, filter, nav, tree, steps, tabs, props.

use junie_tui::{
    App, ByIndex, Cx, DefaultRow, FilterList, FilterListState, Id, Item, ItemKey, List, ListAction,
    ListState, NavList, NavListAction, NavListState, PropsList, PropsRow, PropsState, Response,
    RowUi, SelectMode, StepState, Steps, StepsState, Tabs, TabsState, Tree, TreeNode, TreeState,
    Ui,
};

use crate::expansion::{Facet, FadePosition};
use crate::fixtures::{FixtureApp, FixtureCfg};
use crate::states::ComponentState;

const LIST: Id = Id::root("oracle.components.list");
const FILTER: Id = Id::root("oracle.components.filter");
const NAV: Id = Id::root("oracle.components.nav");
const TREE: Id = Id::root("oracle.components.tree");
const STEPS: Id = Id::root("oracle.components.steps");
const TABS: Id = Id::root("oracle.components.tabs");
const PROPS: Id = Id::root("oracle.components.props");

const ROWS: [(&str, &str); 6] = [
    ("Ada Lovelace", "analyst"),
    ("Grace Hopper", "rear admiral"),
    ("Hedy Lamarr", "inventor"),
    ("Alan Turing", "logician"),
    ("Edsger Dijkstra", "programmer"),
    ("Barbara Liskov", "professor"),
];

const TAB_LABELS: [&str; 4] = ["General", "Account", "Vault", "Review"];

const FILTER_ITEMS: [Item<'static>; 3] = [
    Item::new(ItemKey::num(1), "Ada Lovelace").detail("analyst"),
    Item::new(ItemKey::num(2), "Grace Hopper").detail("rear admiral"),
    Item::new(ItemKey::num(3), "Alan Turing").detail("logician"),
];

fn row_key(row: &(String, String)) -> ItemKey {
    ItemKey::text(row.0.as_str())
}

fn row_paint(row: &(String, String), ui: &mut RowUi<'_>) {
    ui.label(row.0.as_str());
    ui.meta(row.1.as_str());
}

fn static_row_key(row: &(&str, &str)) -> ItemKey {
    ItemKey::text(row.0)
}

fn static_row_paint(row: &(&str, &str), ui: &mut RowUi<'_>) {
    ui.label(row.0);
    ui.meta(row.1);
}

/// Owned rows with unique text keys: base names verbatim, suffixed past six.
fn owned_rows(count: usize) -> Vec<(String, String)> {
    ROWS.iter()
        .cycle()
        .take(count)
        .enumerate()
        .map(|(index, base)| {
            if count <= ROWS.len() {
                (base.0.to_owned(), base.1.to_owned())
            } else {
                (format!("{} {index:02}", base.0), base.1.to_owned())
            }
        })
        .collect()
}

fn tab_key(label: &&'static str) -> ItemKey {
    ItemKey::text(label)
}

fn tab_paint(label: &&'static str, ui: &mut RowUi<'_>) {
    ui.label(label);
}

fn step_state(row: &(String, String)) -> StepState {
    match row.0.as_str() {
        "Ada Lovelace" => StepState::Done,
        "Grace Hopper" => StepState::Running,
        "Alan Turing" => StepState::Failed,
        "Edsger Dijkstra" => StepState::Blocked,
        "Barbara Liskov" => StepState::Skipped,
        _ => StepState::Queued,
    }
}

#[derive(Clone, Copy, Debug)]
struct TreeRow {
    key: ItemKey,
    label: &'static str,
    meta: &'static str,
    node: TreeNode,
}

const TREE_ROWS: [TreeRow; 6] = [
    TreeRow {
        key: ItemKey::num(1),
        label: "Workspace",
        meta: "root",
        node: TreeNode::parent(0),
    },
    TreeRow {
        key: ItemKey::num(2),
        label: "crates",
        meta: "dir",
        node: TreeNode::parent(1),
    },
    TreeRow {
        key: ItemKey::num(3),
        label: "tui.rs",
        meta: "file",
        node: TreeNode::leaf(2),
    },
    TreeRow {
        key: ItemKey::num(4),
        label: "apps",
        meta: "dir",
        node: TreeNode::parent(1),
    },
    TreeRow {
        key: ItemKey::num(5),
        label: "showcase.rs",
        meta: "file",
        node: TreeNode::leaf(2),
    },
    TreeRow {
        key: ItemKey::num(6),
        label: "README.md",
        meta: "file",
        node: TreeNode::leaf(1),
    },
];

/// Owned tree rows: base tree verbatim, flat leaves past six.
fn owned_tree(count: usize) -> Vec<TreeRow> {
    if count <= TREE_ROWS.len() {
        return TREE_ROWS.iter().take(count).copied().collect();
    }
    let mut rows = TREE_ROWS.to_vec();
    for index in TREE_ROWS.len()..count {
        rows.push(TreeRow {
            key: ItemKey::num(index.saturating_add(1) as u64),
            label: leak_label(format!("node {index:02}")),
            meta: "file",
            node: TreeNode::leaf(1),
        });
    }
    rows
}

fn leak_label(label: String) -> &'static str {
    Box::leak(label.into_boxed_str())
}

fn tree_key(row: &TreeRow) -> ItemKey {
    row.key
}

fn tree_node(row: &TreeRow) -> TreeNode {
    row.node
}

fn tree_row(row: &TreeRow, ui: &mut RowUi<'_>) {
    ui.label(row.label);
    ui.meta(row.meta);
}

const PROPS_ROWS: [PropsRow<'static>; 4] = [
    PropsRow::new(ItemKey::num(1), "Name", "Ada Lovelace"),
    PropsRow::new(ItemKey::num(2), "Role", "analyst"),
    PropsRow::new(ItemKey::num(3), "Team", "foundations"),
    PropsRow::new(ItemKey::num(4), "Location", "London"),
];

/// Fade-facet content length: short for `Fits`, tall otherwise.
const fn fade_len(facet: Facet) -> usize {
    match facet {
        Facet::FadeViewport { position, .. } => match position {
            FadePosition::Fits => 2,
            FadePosition::Top | FadePosition::Middle | FadePosition::Bottom => 40,
        },
        _ => 6,
    }
}

/// Keyed list fixture.
#[derive(Debug)]
pub struct ListFixture {
    cfg: FixtureCfg,
    state: ListState,
    items: Vec<(String, String)>,
}

impl ListFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        let items = if cfg.state == ComponentState::Empty {
            Vec::new()
        } else {
            owned_rows(fade_len(cfg.facet))
        };
        Self {
            cfg,
            state: ListState::default(),
            items,
        }
    }

    fn widget(&self) -> List<'static, (String, String), ByIndex, DefaultRow> {
        let list = List::new(LIST).select_mode(SelectMode::Single);
        if self.cfg.state == ComponentState::Disabled {
            list.disabled_item(&|_: &(String, String)| true)
        } else {
            list
        }
    }
}

impl App for ListFixture {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let response = self
            .widget()
            .key(row_key as fn(&(String, String)) -> ItemKey)
            .row(row_paint as fn(&(String, String), &mut RowUi<'_>))
            .update(cx, &mut self.state, &self.items);
        let _ = response.action_ref().copied() as Option<ListAction>;
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.widget()
            .key(row_key as fn(&(String, String)) -> ItemKey)
            .row(row_paint as fn(&(String, String), &mut RowUi<'_>))
            .draw(ui, self.cfg.widget_rect(), &self.state, &self.items);
    }
}

impl FixtureApp for ListFixture {
    fn fixture_id(&self) -> Id {
        LIST
    }

    fn check(&self, state: ComponentState) -> bool {
        match state {
            ComponentState::Current => self.state.cursor().is_some(),
            ComponentState::Selected => self.state.chosen().is_some(),
            _ => true,
        }
    }
}

/// Filtered list fixture.
#[derive(Debug)]
pub struct FilterFixture {
    cfg: FixtureCfg,
    state: FilterListState,
}

impl FilterFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        Self {
            cfg,
            state: FilterListState::default(),
        }
    }
}

impl App for FilterFixture {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        FilterList::new(FILTER)
            .update(cx, &mut self.state, &FILTER_ITEMS)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let items: &[Item<'static>] = if self.cfg.state == ComponentState::Empty {
            &[]
        } else {
            &FILTER_ITEMS
        };
        FilterList::new(FILTER).draw(ui, self.cfg.widget_rect(), &self.state, items);
    }
}

impl FixtureApp for FilterFixture {
    fn fixture_id(&self) -> Id {
        FILTER
    }

    fn check(&self, state: ComponentState) -> bool {
        match state {
            ComponentState::Editing => !self.state.query().is_empty(),
            ComponentState::Current => self.state.cursor().is_some(),
            _ => true,
        }
    }
}

/// Navigation list fixture.
#[derive(Debug)]
pub struct NavFixture {
    cfg: FixtureCfg,
    state: NavListState,
    chosen: Option<ItemKey>,
}

impl NavFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        Self {
            cfg,
            state: NavListState::default(),
            chosen: None,
        }
    }

    fn widget(&self) -> NavList<'static, (&'static str, &'static str), ByIndex, DefaultRow> {
        NavList::new(NAV).disabled(self.cfg.state == ComponentState::Disabled)
    }
}

impl App for NavFixture {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let response = self
            .widget()
            .key(static_row_key as fn(&(&str, &str)) -> ItemKey)
            .row(static_row_paint as fn(&(&str, &str), &mut RowUi<'_>))
            .update(cx, &mut self.state, &ROWS);
        if let Some(NavListAction::Chose(key)) = response.action_ref() {
            self.chosen = Some(*key);
        }
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.widget()
            .key(static_row_key as fn(&(&str, &str)) -> ItemKey)
            .row(static_row_paint as fn(&(&str, &str), &mut RowUi<'_>))
            .draw(ui, self.cfg.widget_rect(), &self.state, &ROWS);
    }
}

impl FixtureApp for NavFixture {
    fn fixture_id(&self) -> Id {
        NAV
    }

    fn check(&self, state: ComponentState) -> bool {
        match state {
            ComponentState::Current => self.state.cursor() == Some(ItemKey::text("Grace Hopper")),
            ComponentState::Selected => self.chosen.is_some(),
            _ => true,
        }
    }
}

/// Keyed tree fixture.
#[derive(Debug)]
pub struct TreeFixture {
    cfg: FixtureCfg,
    state: TreeState,
    items: Vec<TreeRow>,
}

impl TreeFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        let mut state = TreeState::default();
        state.expand(ItemKey::num(1));
        state.expand(ItemKey::num(2));
        state.expand(ItemKey::num(4));
        Self {
            cfg,
            state,
            items: owned_tree(fade_len(cfg.facet)),
        }
    }

    fn widget(&self) -> Tree<'static, TreeRow, ByIndex, DefaultRow> {
        Tree::new(TREE).disabled(self.cfg.state == ComponentState::Disabled)
    }
}

impl App for TreeFixture {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.widget()
            .key(tree_key as fn(&TreeRow) -> ItemKey)
            .node(&(tree_node as fn(&TreeRow) -> TreeNode))
            .row(tree_row as fn(&TreeRow, &mut RowUi<'_>))
            .update(cx, &mut self.state, &self.items)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.widget()
            .key(tree_key as fn(&TreeRow) -> ItemKey)
            .node(&(tree_node as fn(&TreeRow) -> TreeNode))
            .row(tree_row as fn(&TreeRow, &mut RowUi<'_>))
            .draw(ui, self.cfg.widget_rect(), &self.state, &self.items);
    }
}

impl FixtureApp for TreeFixture {
    fn fixture_id(&self) -> Id {
        TREE
    }

    fn check(&self, state: ComponentState) -> bool {
        match state {
            ComponentState::Current => self.state.cursor().is_some(),
            ComponentState::Selected => self.state.chosen().is_some(),
            _ => true,
        }
    }
}

/// Step tracker fixture.
#[derive(Debug)]
pub struct StepsFixture {
    cfg: FixtureCfg,
    state: StepsState,
    items: Vec<(String, String)>,
}

impl StepsFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        Self {
            cfg,
            state: StepsState::default(),
            items: owned_rows(fade_len(cfg.facet)),
        }
    }

    fn widget(&self) -> Steps<'static, (String, String), ByIndex, DefaultRow> {
        Steps::navigable(STEPS).disabled(self.cfg.state == ComponentState::Disabled)
    }
}

impl App for StepsFixture {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.widget()
            .key(row_key as fn(&(String, String)) -> ItemKey)
            .row(row_paint as fn(&(String, String), &mut RowUi<'_>))
            .step(&(step_state as fn(&(String, String)) -> StepState))
            .update(cx, &mut self.state, &self.items)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.widget()
            .key(row_key as fn(&(String, String)) -> ItemKey)
            .row(row_paint as fn(&(String, String), &mut RowUi<'_>))
            .step(&(step_state as fn(&(String, String)) -> StepState))
            .draw(ui, self.cfg.widget_rect(), &self.state, &self.items);
    }
}

impl FixtureApp for StepsFixture {
    fn fixture_id(&self) -> Id {
        STEPS
    }

    fn check(&self, state: ComponentState) -> bool {
        if state == ComponentState::Current {
            return self.state.cursor().is_some();
        }
        true
    }
}

/// Tab strip fixture.
#[derive(Debug)]
pub struct TabsFixture {
    cfg: FixtureCfg,
    state: TabsState,
}

impl TabsFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        Self {
            cfg,
            state: TabsState::default(),
        }
    }

    fn widget() -> Tabs<'static, &'static str, ByIndex, DefaultRow> {
        Tabs::new(TABS).closable(true).allow_new(true)
    }
}

impl App for TabsFixture {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        Self::widget()
            .key(tab_key as fn(&&'static str) -> ItemKey)
            .row(tab_paint as fn(&&'static str, &mut RowUi<'_>))
            .update(cx, &mut self.state, &TAB_LABELS)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Self::widget()
            .key(tab_key as fn(&&'static str) -> ItemKey)
            .row(tab_paint as fn(&&'static str, &mut RowUi<'_>))
            .draw(ui, self.cfg.widget_rect(), &self.state, &TAB_LABELS);
    }
}

impl FixtureApp for TabsFixture {
    fn fixture_id(&self) -> Id {
        TABS
    }

    fn check(&self, state: ComponentState) -> bool {
        if state == ComponentState::Selected {
            return self.state.active() == Some(ItemKey::text("Account"));
        }
        true
    }
}

/// Property list fixture.
#[derive(Debug)]
pub struct PropsFixture {
    cfg: FixtureCfg,
    state: PropsState,
    rows: Vec<PropsRow<'static>>,
}

impl PropsFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        const LABELS: [(&str, &str); 4] = [
            ("Name", "Ada Lovelace"),
            ("Role", "analyst"),
            ("Team", "foundations"),
            ("Location", "London"),
        ];
        let count = match cfg.facet {
            Facet::FadeViewport { .. } => fade_len(cfg.facet),
            _ => PROPS_ROWS.len(),
        };
        let rows: Vec<PropsRow<'static>> = LABELS
            .iter()
            .cycle()
            .take(count)
            .enumerate()
            .map(|(index, base)| {
                PropsRow::new(ItemKey::num(index.saturating_add(1) as u64), base.0, base.1)
            })
            .collect();
        Self {
            cfg,
            state: PropsState::default(),
            rows,
        }
    }
}

impl App for PropsFixture {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        PropsList::new(PROPS)
            .update(cx, &mut self.state, &self.rows)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        PropsList::new(PROPS).draw(ui, self.cfg.widget_rect(), &self.state, &self.rows);
    }
}

impl FixtureApp for PropsFixture {
    fn fixture_id(&self) -> Id {
        PROPS
    }

    fn check(&self, state: ComponentState) -> bool {
        if state == ComponentState::Current {
            return self.state.cursor_index() == 1;
        }
        true
    }
}
