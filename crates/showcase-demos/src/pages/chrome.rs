//! Application chrome: brand lockup, status strip and inline meters.

use core::cell::Cell;

use termrock::{
    ActionKey, Anchor, Brand, Chord, ContextMenu, CrossAlign, Cx, FrameRead, Hint, HintBar,
    HintKey, HintLayer, Id, ItemKey, KeyCode, List, ListState, Menu, MenuAction, MenuBar, MenuItem,
    MenuState, Panel, PanelKind, Rect, Response, Role, RowUi, SelectMode, Side, StateFlags,
    StatusBar, StatusItem, Ui, id,
};

use super::{Page, PageUpdate, frame};

const BRAND: Id = id!("chrome.brand");
const BAR: Id = id!("chrome.menubar");
const SESSIONS_PANEL: Id = id!("chrome.sessions.panel");
const SESSIONS: Id = id!("chrome.sessions");
const CONTEXT: Id = id!("chrome.context");
const STATUS_BAR: Id = id!("chrome.status");
const HINT_BAR: Id = id!("chrome.hintbar");

const ACTION_NEW_TAB: ActionKey = ActionKey::custom("chrome.file.new_tab");
const ACTION_SPLIT_RIGHT: ActionKey = ActionKey::custom("chrome.file.split_right");
const ACTION_EXPORT: ActionKey = ActionKey::custom("chrome.file.export");
const ACTION_CLOSE_TAB: ActionKey = ActionKey::custom("chrome.file.close_tab");

const ACTION_ZOOM: ActionKey = ActionKey::custom("chrome.view.zoom");
const ACTION_REDRAW: ActionKey = ActionKey::custom("chrome.view.redraw");
const ACTION_USAGE: ActionKey = ActionKey::custom("chrome.view.usage");
const ACTION_INSPECT: ActionKey = ActionKey::custom("chrome.view.inspect");

const ACTION_KEY_REF: ActionKey = ActionKey::custom("chrome.help.ref");

const ACTION_CHANGE_TITLE: ActionKey = ActionKey::custom("chrome.context.change_title");
const ACTION_MOVE_LEFT: ActionKey = ActionKey::custom("chrome.context.move_left");
const ACTION_MOVE_RIGHT: ActionKey = ActionKey::custom("chrome.context.move_right");
const ACTION_CLOSE_SESSION: ActionKey = ActionKey::custom("chrome.context.close");
const ACTION_OPEN_CONTEXT: ActionKey = ActionKey::custom("chrome.context.open");

const FILE_MENU: [MenuItem<'static>; 4] = [
    MenuItem::new(ACTION_NEW_TAB, "New tab").chord(Chord::key(KeyCode::Char('c'))),
    MenuItem::new(ACTION_SPLIT_RIGHT, "Split right").chord(Chord::key(KeyCode::Char('%'))),
    MenuItem::new(ACTION_EXPORT, "Export…").separator(),
    MenuItem::new(ACTION_CLOSE_TAB, "Close tab")
        .chord(Chord::key(KeyCode::Char('&')))
        .danger(),
];

const VIEW_MENU: [MenuItem<'static>; 4] = [
    MenuItem::new(ACTION_ZOOM, "Zoom pane").chord(Chord::key(KeyCode::Char('z'))),
    MenuItem::new(ACTION_REDRAW, "Redraw")
        .chord(Chord::key(KeyCode::Char('r')))
        .separator(),
    MenuItem::new(ACTION_USAGE, "Usage").chord(Chord::key(KeyCode::Char('u'))),
    MenuItem::new(ACTION_INSPECT, "Inspect changes").disabled(true),
];

const HELP_MENU: [MenuItem<'static>; 1] =
    [MenuItem::new(ACTION_KEY_REF, "Key reference").chord(Chord::key(KeyCode::Char('?')))];

const MENUS: [Menu<'static>; 3] = [
    Menu::new("File", &FILE_MENU),
    Menu::new("View", &VIEW_MENU),
    Menu::new("Help", &HELP_MENU),
];

fn context_items(cursor: usize) -> [MenuItem<'static>; 4] {
    [
        MenuItem::new(ACTION_CHANGE_TITLE, "Change title…").chord(Chord::key(KeyCode::Char('r'))),
        MenuItem::new(ACTION_MOVE_LEFT, "Move left").disabled(cursor == 0),
        MenuItem::new(ACTION_MOVE_RIGHT, "Move right")
            .disabled(cursor + 1 >= SESSION_ITEMS.len())
            .separator(),
        MenuItem::new(ACTION_CLOSE_SESSION, "Close")
            .chord(Chord::key(KeyCode::Char('x')))
            .danger(),
    ]
}

const PANEL_PARTS: &[(termrock::Part, termrock::StylePatch)] = &[(
    termrock::Part::DETAIL,
    termrock::StylePatch::new().set_fg(Role::Fg(termrock::FgStep::Faint)),
)];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SessionItem {
    key: usize,
    label: &'static str,
    meta: &'static str,
}

const SESSION_ITEMS: [SessionItem; 4] = [
    SessionItem {
        key: 0,
        label: "1 Claude Code (Work)",
        meta: "working",
    },
    SessionItem {
        key: 1,
        label: "2 Codex (Primary)",
        meta: "idle",
    },
    SessionItem {
        key: 2,
        label: "3 Shell",
        meta: "",
    },
    SessionItem {
        key: 3,
        label: "4 docs",
        meta: "blocked",
    },
];

const PR_KEY: ItemKey = ItemKey::index(1);
const USAGE_KEY: ItemKey = ItemKey::index(2);

const STATUS_LEFT: [StatusItem<'static>; 2] = [
    StatusItem::new("payments-platform").strong().priority(9),
    StatusItem::new("PR #482 · settlement backoff")
        .tone(Role::Fg(termrock::FgStep::Secondary))
        .priority(7)
        .key(PR_KEY),
];

const STATUS_CENTER: [StatusItem<'static>; 1] = [StatusItem::new("Claude Code · working · 2 tabs")
    .tone(Role::Fg(termrock::FgStep::Secondary))
    .priority(4)];

const STATUS_RIGHT: [StatusItem<'static>; 3] = [
    StatusItem::new("Weekly 59%")
        .tone(Role::Warning)
        .chip()
        .priority(6)
        .key(USAGE_KEY),
    StatusItem::new("jackin-payments-7f3a")
        .tone(Role::Fg(termrock::FgStep::Muted))
        .chip()
        .priority(3),
    StatusItem::new("run 9c41")
        .tone(Role::Fg(termrock::FgStep::Faint))
        .priority(2),
];

fn brand() -> Brand<'static> {
    Brand::new(BRAND, "app❯").clickable(true)
}

fn menu_bar() -> MenuBar<'static> {
    MenuBar::new(BAR, &MENUS)
}

fn status_bar() -> StatusBar<'static> {
    StatusBar::new(STATUS_BAR)
        .left(&STATUS_LEFT)
        .center(&STATUS_CENTER)
        .right(&STATUS_RIGHT)
}

/// Chrome keeps a clickable brand and a deterministic status strip in state.
#[derive(Debug)]
pub struct ChromePage {
    brand_clicks: u32,
    last: String,
    zoomed: bool,
    bar_state: MenuState,
    sessions_state: ListState,
    context_state: MenuState,
    context_anchor: Anchor,
    context_open: bool,
    /// List area recorded by the last draw; the `m` binding anchors here.
    sessions_area: Cell<Rect>,
}

impl Default for ChromePage {
    fn default() -> Self {
        Self {
            brand_clicks: 0,
            last: "nothing yet".to_string(),
            zoomed: false,
            bar_state: MenuState::default(),
            sessions_state: ListState::default(),
            context_state: MenuState::default(),
            context_anchor: Anchor::Screen(termrock::ScreenAlign::UpperThird),
            context_open: false,
            sessions_area: Cell::new(Rect::default()),
        }
    }
}

impl ChromePage {
    pub fn new() -> Self {
        Self::default()
    }

    fn cursor_index(&self) -> usize {
        match self.sessions_state.cursor() {
            Some(ItemKey::Index(i)) => i,
            _ => 0,
        }
    }

    fn open_context(&mut self, cx: &mut Cx<'_>, anchor: Anchor) {
        let cursor = self
            .cursor_index()
            .min(SESSION_ITEMS.len().saturating_sub(1));
        self.context_anchor = anchor;
        let menu_items = context_items(cursor);
        let menu = ContextMenu::new(CONTEXT, &menu_items, self.context_anchor)
            .title(SESSION_ITEMS[cursor].label);
        cx.open_layer(CONTEXT, menu.layer(cx));
        self.context_open = true;
    }

    fn hint_layer(&self, ui: &Ui<'_>) -> HintLayer {
        if self.bar_state.is_open() {
            HintLayer {
                hints: vec![
                    Hint {
                        key: HintKey::Label("↑↓"),
                        label: "Move",
                        priority: 10,
                    },
                    Hint {
                        key: HintKey::Label("← →"),
                        label: "Switch menu",
                        priority: 9,
                    },
                    Hint {
                        key: HintKey::Label("Enter"),
                        label: "Choose",
                        priority: 8,
                    },
                    Hint {
                        key: HintKey::Label("Esc"),
                        label: "Close",
                        priority: 7,
                    },
                ],
                badge: None,
                status: None,
                centered: false,
            }
        } else if self.context_open {
            HintLayer {
                hints: vec![
                    Hint {
                        key: HintKey::Label("↑↓"),
                        label: "Move",
                        priority: 10,
                    },
                    Hint {
                        key: HintKey::Label("Enter"),
                        label: "Choose",
                        priority: 9,
                    },
                    Hint {
                        key: HintKey::Label("Esc"),
                        label: "Close",
                        priority: 8,
                    },
                ],
                badge: None,
                status: None,
                centered: false,
            }
        } else if ui.state(BAR).contains(StateFlags::FOCUSED) {
            HintLayer {
                hints: vec![
                    Hint {
                        key: HintKey::Label("← →"),
                        label: "Menu",
                        priority: 10,
                    },
                    Hint {
                        key: HintKey::Label("Enter"),
                        label: "Open",
                        priority: 9,
                    },
                    Hint {
                        key: HintKey::Label("Tab"),
                        label: "Next",
                        priority: 8,
                    },
                ],
                badge: None,
                status: None,
                centered: false,
            }
        } else {
            let last_str = if self.brand_clicks > 0 {
                format!("brand activations: {}", self.brand_clicks)
            } else {
                format!("last: {}", self.last)
            };
            HintLayer {
                hints: vec![
                    Hint {
                        key: HintKey::Label("↑↓"),
                        label: "Move",
                        priority: 10,
                    },
                    Hint {
                        key: HintKey::Label("m"),
                        label: "Context menu",
                        priority: 9,
                    },
                    Hint {
                        key: HintKey::Label("right-click"),
                        label: "Context menu",
                        priority: 8,
                    },
                    Hint {
                        key: HintKey::Label("Tab"),
                        label: "Next",
                        priority: 7,
                    },
                ],
                badge: if self.zoomed { Some("ZOOM") } else { None },
                status: Some(last_str.into()),
                centered: false,
            }
        }
    }
}

impl Page for ChromePage {
    fn title(&self) -> &'static str {
        "Chrome"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let mut response = Response::ignored();

        let brand_resp = brand().update(cx);
        if brand_resp.action_ref().is_some() {
            self.brand_clicks = self.brand_clicks.saturating_add(1);
            self.last = "brand".into();
            response |= Response::changed();
        }
        response |= brand_resp.erase();

        let bar_resp = menu_bar().update(cx, &mut self.bar_state);
        if let Some(MenuAction::Chosen(action_key)) = bar_resp.action_ref().copied() {
            if action_key == ACTION_ZOOM {
                self.zoomed = !self.zoomed;
            }
            for m in &MENUS {
                for it in m.items {
                    if it.action() == action_key {
                        self.last = format!("{} › {}", m.label, it.label());
                    }
                }
            }
            response |= Response::changed();
        }
        response |= bar_resp.erase();

        let cursor_before = self
            .cursor_index()
            .min(SESSION_ITEMS.len().saturating_sub(1));
        if self.context_open || cx.is_open(CONTEXT) {
            let menu_items = context_items(cursor_before);
            let menu = ContextMenu::new(CONTEXT, &menu_items, self.context_anchor)
                .title(SESSION_ITEMS[cursor_before].label);
            let ctx_resp = menu.update(cx, &mut self.context_state);
            if let Some(action) = ctx_resp.action_ref().copied() {
                match action {
                    MenuAction::Chosen(act) => {
                        for it in &menu_items {
                            if it.action() == act {
                                self.last = format!("tab › {}", it.label());
                            }
                        }
                        self.context_open = false;
                        response |= Response::changed();
                    }
                    MenuAction::Closed(_) => {
                        self.context_open = false;
                        response |= Response::changed();
                    }
                    _ => {}
                }
            }
            response |= ctx_resp.erase();
        }

        let list_resp = List::new(SESSIONS)
            .key(|s: &SessionItem| ItemKey::index(s.key))
            .row(|s: &SessionItem, row: &mut RowUi<'_>| {
                if !s.meta.is_empty() {
                    row.meta(s.meta);
                }
                row.label(s.label);
            })
            .select_mode(SelectMode::Single)
            .update(cx, &mut self.sessions_state, &SESSION_ITEMS);
        response |= list_resp.erase();

        for intent in cx.intents(SESSIONS) {
            match intent {
                termrock::Intent::Pointer {
                    phase: termrock::Phase::Secondary,
                    part:
                        termrock::PartRef {
                            item: Some(ItemKey::Index(row)),
                            ..
                        },
                    pos,
                    ..
                } => {
                    if row < SESSION_ITEMS.len() {
                        self.sessions_state.set_cursor(row, ItemKey::index(row));
                        cx.focus(SESSIONS);
                        self.open_context(
                            cx,
                            Anchor::Rect {
                                rect: Rect::new(pos.x, pos.y, 1, 1),
                                side: Side::Below,
                                align: CrossAlign::Start,
                            },
                        );
                        response |= Response::changed();
                    }
                }
                termrock::Intent::Binding(action) if action == ACTION_OPEN_CONTEXT => {
                    let row = self
                        .cursor_index()
                        .min(SESSION_ITEMS.len().saturating_sub(1));
                    let area = self.sessions_area.get();
                    let anchor_rect = Rect::new(
                        area.x.saturating_add(2),
                        area.y.saturating_add(row as u16),
                        1,
                        1,
                    );
                    self.open_context(
                        cx,
                        Anchor::Rect {
                            rect: anchor_rect,
                            side: Side::Below,
                            align: CrossAlign::Start,
                        },
                    );
                    response |= Response::changed();
                }
                _ => {}
            }
        }

        let status_resp = status_bar().update(cx);
        if let Some(termrock::StatusAction::Chose(key)) = status_resp.action_ref().copied() {
            if key == PR_KEY {
                self.last = "status › PR".into();
                response |= Response::changed();
            } else if key == USAGE_KEY {
                self.last = "status › usage".into();
                response |= Response::changed();
            }
        }
        response |= status_resp.erase();

        response.into()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: termrock::Rect) {
        frame(
            ui,
            area,
            self.title(),
            "Brand lockup · menu bar with anchored menus · status bar planes and priorities · context menu · hint layers",
            |ui, body| {
                let brand_rect = Rect {
                    x: body.x.saturating_add(1),
                    y: body.y,
                    width: 6,
                    height: 1,
                };
                brand().draw(ui, brand_rect);

                let bar_rect = Rect {
                    x: body.x.saturating_add(8),
                    y: body.y,
                    width: body.width.saturating_sub(8),
                    height: 1,
                };
                menu_bar().draw(ui, bar_rect, &self.bar_state);

                let panel_rect = Rect::new(
                    body.x,
                    body.y.saturating_add(2),
                    body.width,
                    body.height.saturating_sub(6),
                );
                let focused = ui.state(SESSIONS).contains(StateFlags::FOCUSED) || self.context_open;
                Panel::new(SESSIONS_PANEL)
                    .kind(PanelKind::Card)
                    .title("Sessions")
                    .meta("right-click or m for the tab menu")
                    .patch_part(PANEL_PARTS)
                    .focused(focused)
                    .draw(ui, panel_rect, |ui, inner| {
                        let list_area =
                            Rect::new(inner.x, inner.y, inner.width.min(48), inner.height.min(6));
                        self.sessions_area.set(list_area);
                        let list_flags = ui.state(SESSIONS);
                        let hovered_key = if self.context_open {
                            Some(ItemKey::index(self.cursor_index()))
                        } else {
                            None
                        };
                        List::new(SESSIONS)
                            .key(|s: &SessionItem| ItemKey::index(s.key))
                            .row(|s: &SessionItem, row: &mut RowUi<'_>| {
                                if !s.meta.is_empty() {
                                    row.meta(s.meta);
                                }
                                row.label(s.label);
                            })
                            .select_mode(SelectMode::Single)
                            .focused(focused)
                            .hovered_key(hovered_key)
                            .draw(ui, list_area, &self.sessions_state, &SESSION_ITEMS);
                        // Dynamic bindings extend the component's published
                        // table, so they must come after its draw.
                        ui.publish_dynamic_bindings(
                            SESSIONS,
                            list_flags,
                            [(ACTION_OPEN_CONTEXT, Some(Chord::key(KeyCode::Char('m'))))]
                                .into_iter(),
                        );

                        let notes = [
                            "The status bar below sits on its own plane: three groups, no",
                            "separator glyphs, and items leave by priority when the row is",
                            "narrow — resize the terminal to watch the center go first.",
                            "",
                            "Brand: one lockup, accent-filled, the only accent-filled control.",
                        ];
                        let nx = inner.x.saturating_add(list_area.width).saturating_add(4);
                        if nx < inner.right() {
                            let notes_area = Rect::new(
                                nx,
                                inner.y,
                                inner.right().saturating_sub(nx),
                                inner.height,
                            );
                            super::lines(ui, notes_area, &notes);
                        }
                    });

                if self.context_open {
                    let cursor = self
                        .cursor_index()
                        .min(SESSION_ITEMS.len().saturating_sub(1));
                    let menu_items = context_items(cursor);
                    let menu = ContextMenu::new(CONTEXT, &menu_items, self.context_anchor)
                        .title(SESSION_ITEMS[cursor].label);
                    let _ = ui.layer(CONTEXT, |ui, layer_area| {
                        menu.draw(ui, layer_area, &self.context_state);
                    });
                }

                let status_row = Rect::new(body.x, body.bottom().saturating_sub(3), body.width, 1);
                status_bar().draw(ui, status_row);

                let hint_label_row = Rect::new(
                    body.x.saturating_add(1),
                    body.bottom().saturating_sub(2),
                    body.width.saturating_sub(1),
                    1,
                );
                super::lines_faint(ui, hint_label_row, &["hint bar · topmost layer wins:"]);

                let hint_row = Rect::new(body.x, body.bottom().saturating_sub(1), body.width, 1);
                let layer = self.hint_layer(ui);
                HintBar::new(HINT_BAR, &layer).draw(ui, hint_row);
            },
        );
    }

    fn hints(&self, ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        if self.bar_state.is_open() {
            &[
                ("↑↓", "Move"),
                ("← →", "Switch menu"),
                ("Enter", "Choose"),
                ("Esc", "Close"),
            ]
        } else if self.context_open {
            &[("↑↓", "Move"), ("Enter", "Choose"), ("Esc", "Close")]
        } else if ui.state(BAR).contains(StateFlags::FOCUSED) {
            &[("← →", "Menu"), ("Enter", "Open")]
        } else {
            &[
                ("↑↓", "Move"),
                ("m", "Context menu"),
                ("right-click", "Context menu"),
            ]
        }
    }
}

#[cfg(test)]
mod context_tests {
    use super::*;
    use termrock::{App, KeyCode, Theme};
    use termrock_test_support::Harness;

    struct PageApp(ChromePage);

    impl App for PageApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            self.0.update(cx).response
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let full = ui.full();
            self.0.draw(ui, full);
        }
    }

    /// `m` must anchor the tab menu at the sessions list, not at the
    /// screen origin: draw records the list area and the keyboard path
    /// reuses it (the pointer path uses the event position instead).
    #[test]
    fn m_anchors_context_menu_at_sessions_list() {
        const W: u16 = 120;
        const H: u16 = 40;
        let mut h = Harness::new(PageApp(ChromePage::new()), Theme::junie(), W, H);
        // An unbound key: runs `update` and draws, recording the list area.
        let _ = h.key(KeyCode::Null);

        let area = h.app().0.sessions_area.get();
        assert!(
            !area.is_empty(),
            "draw must record the sessions list area, got {area:?}"
        );
        let label_row = h.row(area.y);
        assert!(
            label_row.contains("1 Claude Code (Work)"),
            "cursor row 0 must paint on the recorded row {}, got {label_row:?}",
            area.y
        );

        // Primary-click the cursor row to focus the list, then use the key.
        let _ = h.click(area.x.saturating_add(4), area.y);

        let _ = h.key(KeyCode::Char('m'));
        assert!(h.app().0.context_open, "m must open the context menu");
        let Anchor::Rect { rect, .. } = h.app().0.context_anchor else {
            panic!(
                "m must anchor at a rect, got {:?}",
                h.app().0.context_anchor
            );
        };
        assert_eq!(
            (rect.x, rect.y),
            (area.x.saturating_add(2), area.y),
            "menu anchor must sit on the cursor row of the sessions list"
        );
    }
}
