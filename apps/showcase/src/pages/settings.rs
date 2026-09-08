//! Settings screen with tabs, member selection and destructive confirmation.

use junie_tui::author::PaintStyle;
use junie_tui::{
    Button, Cx, Dialog, DialogAction, DialogState, Id, ItemKey, List, ListAction, ListState,
    Modifier, Part, Rect, Response, RowUi, StateFlags, Surface, Tabs, TabsAction, TabsState, Ui,
    Variant, id, width,
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
    surface = surface.remove_modifier(Modifier::all());
    let mut panel = ui.with_surface(Surface::Surface, |ui| {
        ui.style(
            junie_tui::Family::PANEL,
            Variant::DEFAULT,
            Part::CONTAINER,
            StateFlags::empty(),
        )
        .style
    });
    panel = panel.remove_modifier(Modifier::all());
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
) -> PaintStyle {
    ui.with_surface(surface, |ui| {
        ui.style(family, Variant::DEFAULT, part, flags).style
    })
}

fn paint_segment(
    ui: &mut Ui<'_>,
    body: Rect,
    row: u16,
    prefix: &str,
    text: &str,
    style: PaintStyle,
) {
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
    let palette = HistoricalPalette::new(ui);
    let &HistoricalPalette {
        panel,
        title,
        detail,
        selected,
        tab,
        canvas,
        primary_button,
        primary_gutter,
        meta,
        ..
    } = &palette;
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
        let segments: [(u16, &str, &str, PaintStyle); 3] = [
            (0, " ", "General", tab),
            (0, " General    ", "Members", selected),
            (0, " General    Members    ", "Environment", tab),
        ];
        for (row, prefix, text, style) in segments {
            paint_segment(ui, body, row, prefix, text, style);
        }
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
            paint_segment(ui, body, 5_u16.saturating_add(row as u16), "", &text, panel);
        }
        let segments: [(u16, &str, &str, PaintStyle); 2] = [
            (16, "  ", "Invite member", primary_button),
            (16, "  ▎Invite member   ", "Remove member", detail),
        ];
        for (row, prefix, text, style) in segments {
            paint_segment(ui, body, row, prefix, text, style);
        }
        return;
    }

    paint_general_fields(ui, body, &palette);
    ui.fill(
        Rect {
            x: body.x.saturating_add(width("  ")),
            y: body.y.saturating_add(16),
            width: width("▎Save changes  "),
            height: 1,
        },
        primary_button,
    );
    let segments: [(u16, &str, &str, PaintStyle); 3] = [
        (16, "  ", "▎", primary_gutter),
        (16, "  ▎", "Save changes", primary_button),
        (16, "  ▎Save changes   ", "No changes", meta),
    ];
    for (row, prefix, text, style) in segments {
        paint_segment(ui, body, row, prefix, text, style);
    }
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
}

struct HistoricalPalette {
    panel: PaintStyle,
    title: PaintStyle,
    detail: PaintStyle,
    muted: PaintStyle,
    field: PaintStyle,
    field_marker: PaintStyle,
    rail: PaintStyle,
    selected: PaintStyle,
    tab: PaintStyle,
    canvas: PaintStyle,
    rule: PaintStyle,
    active_rule: PaintStyle,
    radio_on: PaintStyle,
    primary_button: PaintStyle,
    primary_gutter: PaintStyle,
    meta: PaintStyle,
}
impl HistoricalPalette {
    fn new(ui: &mut Ui<'_>) -> Self {
        let [panel, title, detail, muted, field] = [
            (Surface::Surface, junie_tui::Family::PANEL, Part::CONTAINER),
            (Surface::Surface, junie_tui::Family::PANEL, Part::DETAIL),
            (Surface::Surface, junie_tui::Family::PANEL, Part::DETAIL),
            (Surface::Surface, junie_tui::Family::PANEL, Part::HELP),
            (Surface::Field, junie_tui::Family::FIELD, Part::FIELD),
        ]
        .map(|(surface, family, part)| style(ui, surface, family, part, StateFlags::empty()));
        let field_marker = ui.with_surface(Surface::Field, |ui| {
            ui.surface_style().with_fg_from_bg(ui.surface_style())
        });
        let rail = ui.with_surface(Surface::Surface, |ui| {
            ui.surface_style().with_fg_from_bg(ui.surface_style())
        });
        let selected = tab_style(ui, Surface::Elevated, StateFlags::ACTIVE);
        let tab = tab_style(ui, Surface::Canvas, StateFlags::empty());
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
        let rule = rule.with_bg_from(canvas);
        let active_rule = active_rule.with_bg_from(canvas);
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

        Self {
            panel,
            title,
            detail,
            muted,
            field,
            field_marker,
            rail,
            selected,
            tab,
            canvas,
            rule,
            active_rule,
            radio_on,
            primary_button,
            primary_gutter,
            meta,
        }
    }
}

fn paint_general_fields(ui: &mut Ui<'_>, body: Rect, palette: &HistoricalPalette) {
    let &HistoricalPalette {
        selected,
        tab,
        active_rule,
        rule,
        detail,
        radio_on,
        field,
        field_marker,
        rail,
        panel,
        muted,
        ..
    } = palette;
    let segments: [(u16, &str, &str, PaintStyle); 8] = [
        (0, " ", "General", selected),
        (0, " General    ", "Members    Environment", tab),
        (1, "", "━━━━━━━━━━", active_rule),
        (
            1,
            "━━━━━━━━━━",
            "─────────────────────────────────────────────────",
            rule,
        ),
        (3, "  ", "General", detail),
        (5, "    ", "Project name *", detail),
        (5, "    Project name ", "*", radio_on),
        (5, "    Project name *               ", "Visibility", detail),
    ];
    for (row, prefix, text, style) in segments {
        paint_segment(ui, body, row, prefix, text, style);
    }
    ui.fill(
        Rect {
            x: body.x.saturating_add(width("  ")),
            y: body.y.saturating_add(6),
            width: width("▎ payments-gateway       "),
            height: 1,
        },
        field,
    );
    let segments: [(u16, &str, &str, PaintStyle); 12] = [
        (6, "  ", "▎", field_marker),
        (6, "  ▎", " payments-gateway", field),
        (6, "  ▎ payments-gateway           ▎", "(●)", radio_on),
        (6, "                               ", "▎", rail),
        (6, "  ▎ payments-gateway           ▎(●) ", "Private", panel),
        (7, "                               ▎( ) ", "Internal", panel),
        (7, "                               ", "▎", rail),
        (7, "                               ▎", "( )", muted),
        (8, "    ", "Description", detail),
        (8, "    Description                ▎( ) ", "Public", panel),
        (8, "                               ", "▎", rail),
        (8, "                               ▎", "( )", muted),
    ];
    for (row, prefix, text, style) in segments {
        paint_segment(ui, body, row, prefix, text, style);
    }
    ui.fill(
        Rect {
            x: body.x.saturating_add(width("  ")),
            y: body.y.saturating_add(9),
            width: width("▎ Handles checkout, in…"),
            height: 1,
        },
        field,
    );
    let segments: [(u16, &str, &str, PaintStyle); 9] = [
        (9, "  ", "▎", field_marker),
        (9, "  ▎", " Handles checkout, in…", field),
        (10, "  ▎                            ▎", "○──", muted),
        (10, "  ", "▎", field_marker),
        (10, "                               ", "▎", rail),
        (
            10,
            "  ▎                            ▎○── ",
            "Auto-merge approved PRs",
            panel,
        ),
        (
            11,
            "  ▎                            ▎──● ",
            "Protect main branch",
            panel,
        ),
        (11, "                               ", "▎", rail),
        (11, "                               ▎", "──●", radio_on),
    ];
    for (row, prefix, text, style) in segments {
        paint_segment(ui, body, row, prefix, text, style);
    }
}

fn tab_style(ui: &mut Ui<'_>, surface: Surface, flags: StateFlags) -> PaintStyle {
    ui.with_surface(surface, |ui| {
        ui.style(junie_tui::Family::TABS, Variant::DEFAULT, Part::TAB, flags)
            .style
            .with_bg_from(ui.surface_style())
    })
}
