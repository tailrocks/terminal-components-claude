//! Settings screen with tabs, member selection and destructive confirmation.

use tui_next::{
    Button, Cx, Dialog, DialogAction, DialogState, Id, ItemKey, List, ListAction, ListState, Rect,
    Response, RowUi, Tabs, TabsAction, TabsState, Ui, Variant, id, layout,
};

use super::{Page, frame, lines};

const TAB: Id = id!("settings.tabs");
const MEMBERS: Id = id!("settings.members");
const INVITE: Id = id!("settings.invite");
const REMOVE: Id = id!("settings.remove");
const REMOVE_DIALOG: Id = id!("settings.remove.dialog");
const TABS: &[&str] = &["General", "Members", "Security"];

#[derive(Clone, Debug, PartialEq, Eq)]
struct Member {
    id: u8,
    name: &'static str,
    email: &'static str,
}

const INITIAL_MEMBERS: &[Member] = &[
    Member {
        id: 1,
        name: "Mira Okafor",
        email: "mira@acme.dev",
    },
    Member {
        id: 2,
        name: "Jonas Weber",
        email: "jonas@acme.dev",
    },
    Member {
        id: 3,
        name: "Ana Costa",
        email: "ana@acme.dev",
    },
    Member {
        id: 4,
        name: "Kai Tanaka",
        email: "kai@acme.dev",
    },
    Member {
        id: 5,
        name: "Sofia Rossi",
        email: "sofia@acme.dev",
    },
    Member {
        id: 6,
        name: "deploy-bot",
        email: "bot@acme.dev",
    },
];

fn member_key(member: &Member) -> ItemKey {
    ItemKey::num(u64::from(member.id))
}
fn member_row(member: &Member, row: &mut RowUi<'_>) {
    row.label(member.name);
    row.meta(member.email);
}
fn member_list()
-> List<'static, Member, impl Fn(&Member) -> ItemKey, impl Fn(&Member, &mut RowUi<'_>)> {
    List::new(MEMBERS).key(member_key).row(member_row)
}

fn remove_dialog() -> Dialog<'static> {
    Dialog::destructive(
        REMOVE_DIALOG,
        "Remove member?",
        "This member will lose access to the workspace.",
    )
}

/// Member records are app state; tabs, list cursor and modal draft are
/// component state owned by this screen.
#[derive(Debug)]
pub(crate) struct SettingsPage {
    tabs: TabsState,
    members: Vec<Member>,
    member_state: ListState,
    remove_state: DialogState,
    remove_open: bool,
    selected: usize,
    message: &'static str,
}

impl SettingsPage {
    pub(crate) fn new() -> Self {
        Self {
            tabs: TabsState::default(),
            members: INITIAL_MEMBERS.to_vec(),
            member_state: ListState::default(),
            remove_state: DialogState::default(),
            remove_open: false,
            selected: 0,
            message: "workspace settings",
        }
    }

    fn selected_member(&self) -> Option<&Member> {
        self.members.get(self.selected)
    }
}

impl Default for SettingsPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for SettingsPage {
    fn title(&self) -> &'static str {
        "Settings"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut result = Response::ignored();
        let tabs = Tabs::new(TAB).update(cx, &mut self.tabs, TABS);
        if let Some(TabsAction::Activated(key)) = tabs.action_ref() {
            self.message = match key {
                ItemKey::Index(0) => "general settings",
                ItemKey::Index(1) => "members",
                ItemKey::Index(2) => "security",
                _ => "tab changed",
            };
        }
        result |= tabs.erase();
        let list = member_list().update(cx, &mut self.member_state, &self.members);
        if let Some(ListAction::Moved | ListAction::Chose(_) | ListAction::Activated(_)) =
            list.action_ref()
            && let Some(key) = list.action_ref().and_then(|action| match action {
                ListAction::Moved => self.member_state.cursor(),
                ListAction::Chose(key) | ListAction::Activated(key) => Some(*key),
                _ => None,
            })
            && let Some(index) = self
                .members
                .iter()
                .position(|member| member_key(member) == key)
        {
            self.selected = index;
            self.message = "member selected";
        }
        result |= list.erase();
        let invite = Button::new(INVITE, "Invite member")
            .variant(Variant::SECONDARY)
            .update(cx);
        if invite.activated() {
            self.message = "invite flow ready";
        }
        result |= invite.erase();
        let remove = Button::new(REMOVE, "Remove member")
            .variant(Variant::DANGER)
            .disabled(self.members.is_empty())
            .update(cx);
        if remove.activated() && self.selected_member().is_some() && !cx.is_open(REMOVE_DIALOG) {
            self.remove_open = true;
            cx.open_layer(REMOVE_DIALOG, remove_dialog().layer(cx));
        }
        result |= remove.erase();
        let dialog = remove_dialog().update(cx, &mut self.remove_state);
        if let Some(action) = dialog.action_ref() {
            match action {
                DialogAction::Action(key) if *key == tui_next::ActionKey::CONFIRM => {
                    if !self.members.is_empty() {
                        self.members
                            .remove(self.selected.min(self.members.len().saturating_sub(1)));
                    }
                    self.selected = self.selected.min(self.members.len().saturating_sub(1));
                    self.message = "member removed";
                }
                DialogAction::Action(_) | DialogAction::Dismissed(_) => {
                    self.message = "remove cancelled";
                }
            }
            cx.close_layer(REMOVE_DIALOG, None);
            self.remove_open = false;
        }
        result |= dialog.erase();
        result
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "tabs · keyed members · destructive flow",
            |ui, body| {
                let (tabs_area, rest) = layout::split_v(body, 2);
                Tabs::new(TAB).draw(ui, tabs_area, &self.tabs, TABS);
                let (members_area, actions) = layout::split_v(rest, rest.height.saturating_sub(5));
                member_list().draw(ui, members_area, &self.member_state, &self.members);
                let action_rows = super::rows(actions, 3);
                Button::new(INVITE, "Invite member")
                    .variant(Variant::SECONDARY)
                    .draw(ui, action_rows.first().copied().unwrap_or(actions));
                Button::new(REMOVE, "Remove member")
                    .variant(Variant::DANGER)
                    .disabled(self.members.is_empty())
                    .draw(ui, action_rows.get(1).copied().unwrap_or(actions));
                let count = format!("{} members · {}", self.members.len(), self.message);
                let _ = ui.paint_str(
                    action_rows.get(2).copied().unwrap_or(actions),
                    &count,
                    ui.surface_style(),
                );
                lines(
                    ui,
                    Rect {
                        y: actions.bottom().saturating_add(1),
                        height: 1,
                        ..body
                    },
                    &[
                        "Member removal is confirmed in a trapped modal and updates the domain list.",
                    ],
                );
            },
        );
        ui.layer(REMOVE_DIALOG, |ui, layer| {
            remove_dialog().draw(ui, layer, &self.remove_state, |ui, body| {
                let _ = ui.paint_str(
                    body,
                    "Enter confirms · Esc keeps the member",
                    ui.surface_style(),
                );
            });
        });
    }
}
