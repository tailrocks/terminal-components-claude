//! Nested sidebar navigation and content ownership.

use junie_tui::{
    Cx, Id, ItemKey, NavList, NavListAction, NavListState, Rect, Response, RowUi, Ui, id, layout,
};

use super::{Page, frame, lines};

const NAV: Id = id!("sidebars.nav");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SidebarItem {
    key: u8,
    label: &'static str,
    section: &'static str,
}

const ITEMS: &[SidebarItem] = &[
    SidebarItem {
        key: 1,
        label: "Workspace",
        section: "Context",
    },
    SidebarItem {
        key: 2,
        label: "Activity",
        section: "Context",
    },
    SidebarItem {
        key: 3,
        label: "Members",
        section: "Team",
    },
    SidebarItem {
        key: 4,
        label: "Audit log",
        section: "Team",
    },
    SidebarItem {
        key: 5,
        label: "Billing",
        section: "Account",
    },
];

fn item_key(item: &SidebarItem) -> ItemKey {
    ItemKey::num(u64::from(item.key))
}
fn item_section(item: &SidebarItem) -> &str {
    item.section
}
fn item_row(item: &SidebarItem, row: &mut RowUi<'_>) {
    row.label(item.label);
}

fn sidebar() -> NavList<
    'static,
    SidebarItem,
    impl Fn(&SidebarItem) -> ItemKey,
    impl Fn(&SidebarItem, &mut RowUi<'_>),
> {
    NavList::new(NAV)
        .key(item_key)
        .section(&item_section)
        .row(item_row)
}

/// The sidebar cursor is independent from the shell's page navigation.
#[derive(Debug, Default)]
pub(crate) struct SidebarsPage {
    state: NavListState,
    selected: &'static str,
}

impl SidebarsPage {
    pub(crate) fn new() -> Self {
        Self {
            state: NavListState::default(),
            selected: "Workspace",
        }
    }
}

impl Page for SidebarsPage {
    fn title(&self) -> &'static str {
        "Sidebars"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let result = sidebar().update(cx, &mut self.state, ITEMS);
        if let Some(NavListAction::Chose(key) | NavListAction::EnterContent(key)) =
            result.action_ref()
            && let Some(item) = ITEMS.iter().find(|item| item_key(item) == *key)
        {
            self.selected = item.label;
        }
        result.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "independent cursor · sections · content",
            |ui, body| {
                let (nav_area, content) = layout::split_h(body, body.width / 3);
                sidebar().draw(ui, nav_area, &self.state, ITEMS);
                lines(
                    ui,
                    content,
                    &[
                        "Sidebar content",
                        "",
                        "The selected section owns the detail view.",
                    ],
                );
                let summary = format!("active section: {}", self.selected);
                let _ = ui.paint_str(
                    Rect {
                        y: content.bottom().saturating_sub(2),
                        height: 1,
                        ..content
                    },
                    &summary,
                    ui.surface_style(),
                );
            },
        );
    }
}
