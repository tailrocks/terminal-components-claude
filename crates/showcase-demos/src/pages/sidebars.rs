//! Nested sidebar navigation and content ownership.

use termrock::{
    Button, Cx, Id, Insets, ItemKey, NavList, NavListAction, NavListState, NavMode, Panel, Rect,
    RowUi, Ui, Variant, id,
};

use super::{Page, PageUpdate, frame, lines};

const NAV: Id = id!("sidebars.nav");
const SIDE_PANEL: Id = id!("sidebars.panel");
const CONTENT_PANEL: Id = id!("sidebars.content");

/// The one side-panel constructor (§13), shared by update and draw.
fn side_panel() -> Panel<'static> {
    // Q67-S14 (N2-enabling): the tag draws the nav full-bleed in the
    // untitled card (`tag:sidebars.rs:378-390` — manual inner, full
    // side width); the default Card 2-cell side insets would clip the
    // gutter/marker/badge columns the N2 unmask exposes.
    Panel::new(SIDE_PANEL).inner_inset(Insets {
        l: 0,
        t: 1,
        r: 0,
        b: 1,
    })
}

/// The one content-card constructor (§13), keyed by the selected section.
fn content_panel(title: &'static str) -> Panel<'static> {
    Panel::new(CONTENT_PANEL).title(title)
}
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

fn collapse_button(collapsed: bool) -> Button<'static> {
    Button::new(COLLAPSE, if collapsed { "›" } else { "Collapse" }).variant(Variant::SECONDARY)
}

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

fn sidebar(
    collapsed: bool,
) -> NavList<
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
        // Q67-S14 (N6): sections paint at area.x+3 (`tag:sidebars.rs:200`).
        .header_indent(3)
        // Q67-S14 (N2-enabling): the tag list scrolls with a scrollbar
        // on overflow (`tag:sidebars.rs:262-276`; frozen 72x20 ┃@x48) —
        // without it the narrow badge sits one cell right of frozen.
        .scrollable(true)
        .mode(if collapsed {
            NavMode::Collapsed
        } else {
            NavMode::Full
        })
        .row(item_row)
}

/// The sidebar cursor is independent from the shell's page navigation.
#[derive(Debug, Default)]
pub struct SidebarsPage {
    state: NavListState,
    selected: &'static str,
    collapsed: bool,
}

impl SidebarsPage {
    pub fn new() -> Self {
        // Q67-S14 (N2-enabling): the tag opens with Tasks current
        // (`tag:sidebars.rs:352`); without it the unmasked NavList
        // shows no › marker and a secondary Tasks label.
        let mut state = NavListState::default();
        state.set_current(Some(item_key(&ITEMS[0])));
        Self {
            state,
            selected: "Tasks",
            collapsed: false,
        }
    }
}

impl Page for SidebarsPage {
    fn title(&self) -> &'static str {
        "Sidebars"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let result = sidebar(self.collapsed).update(cx, &mut self.state, ITEMS);
        if let Some(NavListAction::Chose(key) | NavListAction::EnterContent(key)) =
            result.action_ref()
            && let Some(item) = ITEMS.iter().find(|item| item_key(item) == *key)
        {
            self.selected = item.label;
        }
        let collapse = collapse_button(self.collapsed).update(cx);
        if collapse.activated() {
            self.collapsed = !self.collapsed;
        }
        let mut response = result.erase();
        response |= collapse.erase();
        // Both phases build the same two cards (§13).
        let _ = side_panel();
        let _ = content_panel(self.selected);
        response.into()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "Sections, current item, focus cursor, hover, collapsed mode; text first, no icons",
            |ui, body| {
                let side_width = sidebar(self.collapsed).width().saturating_add(4);
                let side = Rect {
                    width: side_width,
                    height: body.height.min(20),
                    ..body
                };
                side_panel().draw(ui, side, |ui, inner| {
                    sidebar(self.collapsed).draw(
                        ui,
                        Rect {
                            height: inner.height.saturating_sub(2),
                            ..inner
                        },
                        &self.state,
                        ITEMS,
                    );
                    collapse_button(self.collapsed).draw(
                        ui,
                        Rect {
                            x: inner.x.saturating_add(1),
                            y: inner.bottom().saturating_sub(1),
                            height: 1,
                            ..inner
                        },
                    );
                    // Q67-S14 (N2): the canned nav rows (both widths) are
                    // deleted — the real NavList above is the render
                    // (`tag:sidebars.rs:190-256`).
                });

                let content = Rect {
                    x: side.right().saturating_add(2),
                    width: body.width.saturating_sub(side_width.saturating_add(2)),
                    ..body
                };
                content_panel(self.selected).draw(ui, content, |ui, inner| {
                    Self::draw_content(ui, inner, body.width);
                });
            },
        );
    }

    fn hints(&self, _ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        &[("↑ ↓", "Move"), ("Enter", "Open")]
    }
}

impl SidebarsPage {
    fn draw_content(ui: &mut Ui<'_>, inner: Rect, body_width: u16) {
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
        // Q67-S14 OWED (N2-content): this narrow canned wrap tracks
        // the tag's wrapped prose — deleting it regresses narrow combos
        // GREEN→RED (S14 probe). Removal needs a wrap-capable `lines`
        // (component capability, own slice).
        if body_width < 70 {
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
    }
}
