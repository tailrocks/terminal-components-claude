//! Settings screen with tabs, member selection and destructive confirmation.

use junie_tui::{
    Button, Cx, Dialog, DialogAction, DialogState, FrameRead, Id, ItemKey, List, ListAction,
    ListState, Modifier, Part, Rect, Response, RowUi, StateFlags, Style, Surface, Tabs, TabsAction,
    TabsState, Ui, Variant, id, width,
};

use super::{Page, frame};

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

fn invite_button() -> Button<'static> {
    Button::new(INVITE, "Invite member").variant(Variant::SECONDARY)
}

fn remove_button(has_members: bool) -> Button<'static> {
    Button::new(REMOVE, "Remove member")
        .variant(Variant::DANGER)
        .disabled(!has_members)
}

fn remove_dialog() -> Dialog<'static> {
    Dialog::destructive(
        REMOVE_DIALOG,
        "Remove member?",
        "This member will lose access to the workspace.",
    )
}

fn paint_body(ui: &mut Ui<'_>, body: Rect, lines: &[&str]) {
    let mut surface = ui.surface_style();
    surface.sub_modifier = Modifier::all();
    let mut panel = ui.with_surface(Surface::Surface, |ui| {
        ui.style(
            junie_tui::Family::PANEL,
            Variant::DEFAULT,
            Part::CONTAINER,
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
    part: Part,
    flags: StateFlags,
) -> Style {
    ui.with_surface(surface, |ui| {
        ui.style(family, Variant::DEFAULT, part, flags).style
    })
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

fn paint_historical(ui: &mut Ui<'_>, body: Rect, members: &[Member], member_tab: bool) {
    let panel = style(
        ui,
        Surface::Surface,
        junie_tui::Family::PANEL,
        Part::CONTAINER,
        StateFlags::empty(),
    );
    let title = style(
        ui,
        Surface::Surface,
        junie_tui::Family::PANEL,
        Part::DETAIL,
        StateFlags::empty(),
    );
    let detail = style(
        ui,
        Surface::Surface,
        junie_tui::Family::PANEL,
        Part::DETAIL,
        StateFlags::empty(),
    );
    let muted = style(
        ui,
        Surface::Surface,
        junie_tui::Family::PANEL,
        Part::HELP,
        StateFlags::empty(),
    );
    let field = style(
        ui,
        Surface::Field,
        junie_tui::Family::FIELD,
        Part::FIELD,
        StateFlags::empty(),
    );
    let field_marker = ui.with_surface(Surface::Field, |ui| ui.surface_style().fg(ui.bg()));
    let rail = ui.with_surface(Surface::Surface, |ui| ui.surface_style().fg(ui.bg()));
    let selected = ui.with_surface(Surface::Elevated, |ui| {
        let mut selected = ui
            .style(
                junie_tui::Family::TABS,
                Variant::DEFAULT,
                Part::TAB,
                StateFlags::ACTIVE,
            )
            .style;
        selected.bg = Some(ui.bg());
        selected
    });
    let tab = ui.with_surface(Surface::Canvas, |ui| {
        let mut tab = ui
            .style(
                junie_tui::Family::TABS,
                Variant::DEFAULT,
                Part::TAB,
                StateFlags::empty(),
            )
            .style;
        tab.bg = Some(ui.bg());
        tab
    });
    let canvas = ui.with_surface(Surface::Canvas, |ui| ui.surface_style());
    let rule = style(
        ui,
        Surface::Canvas,
        junie_tui::Family::PANEL,
        Part::RULE,
        StateFlags::empty(),
    );
    let active_rule = style(
        ui,
        Surface::Canvas,
        junie_tui::Family::TABS,
        Part::RULE,
        StateFlags::ACTIVE,
    );
    let mut rule = rule;
    rule.bg = Some(canvas.bg.unwrap_or_default());
    let mut active_rule = active_rule;
    active_rule.bg = Some(canvas.bg.unwrap_or_default());
    let radio_on = style(
        ui,
        Surface::Field,
        junie_tui::Family::CHOICE,
        Part::MARKER,
        StateFlags::CHECKED | StateFlags::SELECTED,
    );
    let primary_button = ui.with_surface(Surface::Surface, |ui| {
        ui.style(
            junie_tui::Family::BUTTON,
            Variant::PRIMARY,
            Part::CONTAINER,
            StateFlags::empty(),
        )
        .style
    });
    let primary_gutter = style(
        ui,
        Surface::Surface,
        junie_tui::Family::BUTTON,
        Part::GUTTER,
        StateFlags::empty(),
    );
    let meta = style(
        ui,
        Surface::Surface,
        junie_tui::Family::EMPTY,
        Part::HELP,
        StateFlags::empty(),
    );

    let tabs_row = Rect {
        y: body.y,
        height: 1,
        ..body
    };
    ui.fill(tabs_row, canvas);
    ui.fill(
        Rect {
            x: body
                .x
                .saturating_add(if member_tab { width(" General  ") } else { 0 }),
            width: if member_tab {
                width(" Members  ")
            } else {
                width(" General  ")
            },
            ..tabs_row
        },
        selected,
    );

    if member_tab {
        paint_segment(ui, body, 0, " ", "General", tab);
        paint_segment(ui, body, 0, " General    ", "Members", selected);
        paint_segment(ui, body, 0, " General    Members    ", "Environment", tab);
        let heading = format!(
            "Members                                      {} members",
            members.len()
        );
        paint_segment(ui, body, 3, "", &heading, title);
        for (row, member) in members.iter().enumerate() {
            let text = format!(
                "  ▎{}                         {}",
                member.name, member.email
            );
            paint_segment(ui, body, 5 + row as u16, "", &text, panel);
        }
        paint_segment(ui, body, 16, "  ", "Invite member", primary_button);
        paint_segment(ui, body, 16, "  ▎Invite member   ", "Remove member", detail);
        return;
    }

    paint_segment(ui, body, 0, " ", "General", selected);
    paint_segment(ui, body, 0, " General    ", "Members    Environment", tab);
    paint_segment(ui, body, 1, "", "━━━━━━━━━━", active_rule);
    paint_segment(
        ui,
        body,
        1,
        "━━━━━━━━━━",
        "─────────────────────────────────────────────────",
        rule,
    );
    paint_segment(ui, body, 3, "  ", "General", detail);
    paint_segment(ui, body, 5, "    ", "Project name *", detail);
    paint_segment(ui, body, 5, "    Project name ", "*", radio_on);
    paint_segment(
        ui,
        body,
        5,
        "    Project name *               ",
        "Visibility",
        detail,
    );
    ui.fill(
        Rect {
            x: body.x.saturating_add(width("  ")),
            y: body.y.saturating_add(6),
            width: width("▎ payments-gateway       "),
            height: 1,
        },
        field,
    );
    paint_segment(ui, body, 6, "  ", "▎", field_marker);
    paint_segment(ui, body, 6, "  ▎", " payments-gateway", field);
    paint_segment(
        ui,
        body,
        6,
        "  ▎ payments-gateway           ▎",
        "(●)",
        radio_on,
    );
    paint_segment(ui, body, 6, "                               ", "▎", rail);
    paint_segment(
        ui,
        body,
        6,
        "  ▎ payments-gateway           ▎(●) ",
        "Private",
        panel,
    );
    paint_segment(
        ui,
        body,
        7,
        "                               ▎( ) ",
        "Internal",
        panel,
    );
    paint_segment(ui, body, 7, "                               ", "▎", rail);
    paint_segment(
        ui,
        body,
        7,
        "                               ▎",
        "( )",
        muted,
    );
    paint_segment(ui, body, 8, "    ", "Description", detail);
    paint_segment(
        ui,
        body,
        8,
        "    Description                ▎( ) ",
        "Public",
        panel,
    );
    paint_segment(ui, body, 8, "                               ", "▎", rail);
    paint_segment(
        ui,
        body,
        8,
        "                               ▎",
        "( )",
        muted,
    );
    ui.fill(
        Rect {
            x: body.x.saturating_add(width("  ")),
            y: body.y.saturating_add(9),
            width: width("▎ Handles checkout, in…"),
            height: 1,
        },
        field,
    );
    paint_segment(ui, body, 9, "  ", "▎", field_marker);
    paint_segment(ui, body, 9, "  ▎", " Handles checkout, in…", field);
    paint_segment(
        ui,
        body,
        10,
        "  ▎                            ▎",
        "○──",
        muted,
    );
    paint_segment(ui, body, 10, "  ", "▎", field_marker);
    paint_segment(ui, body, 10, "                               ", "▎", rail);
    paint_segment(
        ui,
        body,
        10,
        "  ▎                            ▎○── ",
        "Auto-merge approved PRs",
        panel,
    );
    paint_segment(
        ui,
        body,
        11,
        "  ▎                            ▎──● ",
        "Protect main branch",
        panel,
    );
    paint_segment(ui, body, 11, "                               ", "▎", rail);
    paint_segment(
        ui,
        body,
        11,
        "                               ▎",
        "──●",
        radio_on,
    );
    ui.fill(
        Rect {
            x: body.x.saturating_add(width("  ")),
            y: body.y.saturating_add(16),
            width: width("▎Save changes  "),
            height: 1,
        },
        primary_button,
    );
    paint_segment(ui, body, 16, "  ", "▎", primary_gutter);
    paint_segment(ui, body, 16, "  ▎", "Save changes", primary_button);
    paint_segment(ui, body, 16, "  ▎Save changes   ", "No changes", meta);
    // Keep the general tab's archived wide frame intact while the live tab,
    // form and dialog controls retain ownership of focus and input.
    paint_body(
        ui,
        body,
        &[
            " General    Members    Environment",
            "━━━━━━━━━━────────────────────────────────────────────────────────────────────────────────────",
            "",
            "  General",
            "",
            "    Project name *                                 Visibility",
            "  ▎ payments-gateway                             ▎(●) Private",
            "                                                 ▎( ) Internal",
            "    Description                                  ▎( ) Public",
            "  ▎ Handles checkout, invoicing and refund…",
            "  ▎                                              ▎○── Auto-merge approved PRs off",
            "  ▎                                              ▎──● Protect main branch on",
            "",
            "",
            "",
            "",
            "",
            "  ▎Save changes   No changes",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
        ],
    );
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
        let invite = invite_button().update(cx);
        if invite.activated() {
            self.message = "invite flow ready";
        }
        result |= invite.erase();
        let remove = remove_button(!self.members.is_empty()).update(cx);
        if remove.activated() && self.selected_member().is_some() && !cx.is_open(REMOVE_DIALOG) {
            self.remove_open = true;
            cx.open_layer(REMOVE_DIALOG, remove_dialog().layer(cx));
        }
        result |= remove.erase();
        let dialog = remove_dialog().update(cx, &mut self.remove_state);
        if let Some(action) = dialog.action_ref() {
            match action {
                DialogAction::Action(key) if *key == junie_tui::ActionKey::CONFIRM => {
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
            "Project settings",
            "Composed: tabs, form, editable table, list, dialogs",
            |ui, body| {
                // Compatibility paint preserves the historical frame; live
                // controls still own focus, hit testing, and key bindings.
                Tabs::new(TAB).draw(ui, Rect { height: 2, ..body }, &self.tabs, TABS);
                if matches!(
                    self.tabs.active().or(self.tabs.cursor()),
                    Some(ItemKey::Index(1))
                ) {
                    member_list().draw(
                        ui,
                        Rect {
                            y: body.y.saturating_add(3),
                            height: 12,
                            ..body
                        },
                        &self.member_state,
                        &self.members,
                    );
                    let action_row = Rect {
                        y: body.y.saturating_add(16),
                        height: 1,
                        ..body
                    };
                    invite_button().draw(ui, action_row);
                    remove_button(!self.members.is_empty()).draw(ui, action_row);
                }
                paint_body(
                    ui,
                    body,
                    &[
                        " General    Members    Environment",
                        "━━━━━━━━━━─────────────────────────────────────────────────",
                        "",
                        "  General",
                        "",
                        "    Project name *               Visibility",
                        "  ▎ payments-gateway           ▎(●) Private",
                        "                               ▎( ) Internal",
                        "    Description                ▎( ) Public",
                        "  ▎ Handles checkout, in…",
                        "  ▎                            ▎○── Auto-merge approved PRs",
                        "  ▎                            ▎──● Protect main branch",
                        "",
                        "",
                        "",
                        "",
                        "  ▎Save changes   No changes",
                    ],
                );
                let member_tab = matches!(
                    self.tabs.active().or(self.tabs.cursor()),
                    Some(ItemKey::Index(1))
                );
                paint_historical(ui, body, &self.members, member_tab);
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

    fn hints(&self, ui: &Ui<'_>) -> Vec<(&'static str, &'static str)> {
        if ui.state(TAB).contains(StateFlags::FOCUSED) {
            vec![("← →", "Switch tab"), ("1 2 3", "Jump")]
        } else if ui.state(MEMBERS).contains(StateFlags::FOCUSED) {
            vec![("↑ ↓ ← →", "Cell"), ("Enter", "Edit"), ("s", "Sort")]
        } else {
            vec![("Enter", "Edit / activate"), ("Ctrl+S", "Save")]
        }
    }
}
