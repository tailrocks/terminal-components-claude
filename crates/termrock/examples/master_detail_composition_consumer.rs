//! Default-feature, facade-only downstream composition example.

use termrock::blocks::{AppShell, MasterDetail, MasterDetailPolicy, MasterDetailState};
use termrock::composition::{self, Group};
use termrock::controls::Button;
use termrock::navigation::{NavList, NavListState};
use termrock::{Id, ItemKey, Ui};

const WORKSPACE: Id = Id::root("example.library.workspace");
const TREE: Id = Id::root("example.library.projects");
const DETAIL: Id = Id::root("example.library.detail");

struct Project {
    key: u64,
    label: &'static str,
}

fn project_key(project: &Project) -> ItemKey {
    ItemKey::num(project.key)
}

fn project_row(project: &Project, row: &mut termrock::RowUi<'_>) {
    row.label(project.label);
}

#[allow(dead_code)]
fn draw_workspace(ui: &mut Ui<'_>, projects: &[Project], tree_state: &NavListState) {
    let projects_list = NavList::new(TREE)
        .key(project_key as fn(&Project) -> ItemKey)
        .row(project_row as fn(&Project, &mut termrock::RowUi<'_>));
    let master = Group::column([composition::nav_list(&projects_list, tree_state, projects)]);

    let detail_button = Button::new(DETAIL, "Review selected project");
    let detail = Group::column([composition::button(&detail_button)]);
    let block = MasterDetail::new(WORKSPACE, TREE, DETAIL)
        .policy(MasterDetailPolicy::default().collapse_below(96))
        .master(&master)
        .detail(&detail);

    let block_state = MasterDetailState::default();
    let block = composition::master_detail(&block, &block_state)
        .expect("master and detail content are configured");
    let body = Group::column([block]);
    AppShell::new()
        .body_slot(&body)
        .show(ui)
        .expect("workspace composition resolves");
}

fn main() {}
