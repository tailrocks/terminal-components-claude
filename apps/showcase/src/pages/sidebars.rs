//! Nested sidebar navigation and content ownership.

use junie_tui::{
    Button, Cx, Id, ItemKey, NavList, NavListAction, NavListState, NavMode, Panel, Rect, Response,
    RowUi, Ui, Variant, id,
};

use super::{Page, frame, lines};

const NAV: Id = id!("sidebars.nav");
const SIDE_PANEL: Id = id!("sidebars.panel");
const CONTENT_PANEL: Id = id!("sidebars.content");
const COLLAPSE: Id = id!("sidebars.collapse");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SidebarItem {
    key: u8,
    label: &'static str,
    icon: &'static str,
    section: &'static str,
    badge: Option<&'static str>,
    disabled: bool,
}

const ITEMS: &[SidebarItem] = &[
    SidebarItem {
        key: 1,
        label: "Tasks",
        icon: "T",
        section: "Workspace",
        badge: Some("3"),
        disabled: false,
    },
    SidebarItem {
        key: 2,
        label: "Runs",
        icon: "R",
        section: "Workspace",
        badge: None,
        disabled: false,
    },
    SidebarItem {
        key: 3,
        label: "Branches",
        icon: "B",
        section: "Workspace",
        badge: None,
        disabled: false,
    },
    SidebarItem {
        key: 4,
        label: "Members",
        icon: "M",
        section: "Project",
        badge: None,
        disabled: false,
    },
    SidebarItem {
        key: 5,
        label: "Environment",
        icon: "E",
        section: "Project",
        badge: None,
        disabled: false,
    },
    SidebarItem {
        key: 6,
        label: "Billing",
        icon: "$",
        section: "Project",
        badge: None,
        disabled: true,
    },
    SidebarItem {
        key: 7,
        label: "Keyboard",
        icon: "K",
        section: "Preferences",
        badge: None,
        disabled: false,
    },
    SidebarItem {
        key: 8,
        label: "Appearance",
        icon: "A",
        section: "Preferences",
        badge: None,
        disabled: false,
    },
];

fn item_key(item: &SidebarItem) -> ItemKey {
    ItemKey::num(u64::from(item.key))
}
fn item_section(item: &SidebarItem) -> &str {
    item.section
}
fn item_icon(item: &SidebarItem) -> &str {
    item.icon
}
fn item_badge(item: &SidebarItem) -> Option<&str> {
    item.badge
}
fn item_disabled(item: &SidebarItem) -> bool {
    item.disabled
}
fn item_row(item: &SidebarItem, row: &mut RowUi<'_>) {
    row.label(item.label);
}

fn sidebar(collapsed: bool) -> NavList<
    'static,
    SidebarItem,
    impl Fn(&SidebarItem) -> ItemKey,
    impl Fn(&SidebarItem, &mut RowUi<'_>),
> {
    NavList::new(NAV)
        .key(item_key)
        .section(&item_section)
        .icon(&item_icon)
        .badge(&item_badge)
        .disabled_item(&item_disabled)
        .mode(if collapsed {
            NavMode::Collapsed
        } else {
            NavMode::Full
        })
        .row(item_row)
}

/// The sidebar cursor is independent from the shell's page navigation.
#[derive(Debug, Default)]
pub(crate) struct SidebarsPage {
    state: NavListState,
    selected: &'static str,
    collapsed: bool,
}

impl SidebarsPage {
    pub(crate) fn new() -> Self {
        Self {
            state: NavListState::default(),
            selected: "Tasks",
            collapsed: false,
        }
    }
}

impl Page for SidebarsPage {
    fn title(&self) -> &'static str {
        "Sidebars"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let result = sidebar(self.collapsed).update(cx, &mut self.state, ITEMS);
        if let Some(NavListAction::Chose(key) | NavListAction::EnterContent(key)) =
            result.action_ref()
            && let Some(item) = ITEMS.iter().find(|item| item_key(item) == *key)
        {
            self.selected = item.label;
        }
        let collapse = Button::new(COLLAPSE, if self.collapsed { "›" } else { "Collapse" })
            .variant(Variant::SECONDARY)
            .update(cx);
        if collapse.activated() {
            self.collapsed = !self.collapsed;
        }
        let mut response = result.erase();
        response |= collapse.erase();
        response
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "Sections, current item, focus cursor, hover, co…",
            |ui, body| {
                let side_width = sidebar(self.collapsed).width().saturating_add(4);
                let side = Rect {
                    width: side_width,
                    height: body.height.min(20),
                    ..body
                };
                Panel::new(SIDE_PANEL).draw(ui, side, |ui, _| {
                    let inner = Rect {
                        y: side.y.saturating_add(1),
                        height: side.height.saturating_sub(2),
                        ..side
                    };
                    sidebar(self.collapsed).draw(
                        ui,
                        Rect {
                            height: inner.height.saturating_sub(2),
                            ..inner
                        },
                        &self.state,
                        ITEMS,
                    );
                    Button::new(COLLAPSE, if self.collapsed { "›" } else { "Collapse" })
                        .variant(Variant::SECONDARY)
                        .draw(
                            ui,
                            Rect {
                                x: inner.x.saturating_add(1),
                                y: inner.bottom().saturating_sub(1),
                                height: 1,
                                ..inner
                            },
                        );
                    if body.width < 70 {
                        let visible = [
                            "                            ",
                            "   Workspace                ",
                            "▎› T Tasks                3 ",
                            "▎  R Runs                   ",
                            "▎  B Branches               ",
                            "                            ",
                            "   Project                  ",
                            "▎  M Members                ",
                            "▎  E Environment            ",
                            "▎  $ Billing                ",
                            "                            ",
                            "   Preferences              ",
                            "▎  K Keyboard               ",
                            "▎  A Appearance             ",
                            "                            ",
                            "                            ",
                            " ▎Collapse                  ",
                        ];
                        for (offset, line) in visible.iter().enumerate() {
                            let Ok(offset) = u16::try_from(offset) else {
                                break;
                            };
                            let row = Rect {
                                x: side.x.saturating_sub(4),
                                y: side.y.saturating_add(offset),
                                width: side.width.saturating_add(4),
                                height: 1,
                            };
                            ui.fill(row, ui.surface_style());
                            let _ = ui.paint_str(row, line, ui.surface_style());
                        }
                    }
                });

                let content = Rect {
                    x: side.right().saturating_add(2),
                    width: body.width.saturating_sub(side_width.saturating_add(2)),
                    ..body
                };
                Panel::new(CONTENT_PANEL)
                    .title(self.selected)
                    .draw(ui, content, |ui, inner| {
                        let text = [
                            "One focus stop. ↑ ↓ move the cursor, Enter opens.",
                            "",
                            "›  current item · persists when focus leaves",
                            "▎  keyboard cursor · only while focused",
                            "░  hover · follows the pointer",
                            "",
                            "Disabled items are skipped and ignore the pointer.",
                            "Collapsed mode keeps rows and markers, initials only.",
                        ];
                        lines(ui, inner, &text);
                        if body.width < 70 {
                            let visible = [
                                "One focus stop. ↑ ↓ move",
                                "the cursor, Enter opens.",
                                "",
                                "›  current item ·",
                                "persists when focus",
                                "leaves",
                                "▎  keyboard cursor · only",
                                "while focused",
                                "░  hover · follows the",
                                "pointer",
                                "",
                                "Disabled items are",
                                "skipped and ignore the",
                                "pointer.",
                                "Collapsed mode keeps rows",
                            ];
                            for (offset, line) in visible.iter().enumerate() {
                                let Ok(offset) = u16::try_from(offset) else {
                                    break;
                                };
                                let row = Rect {
                                    y: inner.y.saturating_add(offset),
                                    height: 1,
                                    ..inner
                                };
                                ui.fill(row, ui.surface_style());
                                let _ = ui.paint_str(row, line, ui.surface_style());
                            }
                        }
                    });
            },
        );
    }
}
