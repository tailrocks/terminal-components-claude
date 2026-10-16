//! Application chrome: the brand lockup, a menu bar with anchored menus, a
//! status bar with three groups that collapse by priority, a context menu
//! on list rows, and the hint-bar layer precedence.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::pages::row_menu::RowMenu;
use crate::pages::{Hint, Page, PageCtx, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::WidgetId;
use junie_tui::theme::{BadgeKind, Tone};
use junie_tui::ui::context::ContextRequest;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::widgets::brand::Lockup;
use junie_tui::widgets::hintbar::{HintBar, HintLayer};
use junie_tui::widgets::keyhint::hint;
use junie_tui::widgets::list::{ListBox, ListItem, SelectMode};
use junie_tui::widgets::menu::{MenuBar, MenuBarAction, MenuItem};
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::statusbar::{StatusBar, StatusItem};
use ratatui::crossterm::event::KeyCode;

const BAR: WidgetId = WidgetId::of("chrome.menubar");
const SESSIONS: WidgetId = WidgetId::of("chrome.sessions");
const CONTEXT: WidgetId = WidgetId::of("chrome.context");
const USAGE: WidgetId = WidgetId::of("chrome.status.usage");
const PR: WidgetId = WidgetId::of("chrome.status.pr");

pub struct ChromePage {
    bar: MenuBar,
    sessions: ListBox,
    context: RowMenu<usize>,
    last: String,
    zoomed: bool,
    hint_area: Rect,
}

impl ChromePage {
    pub fn new() -> Self {
        let bar = MenuBar::new(
            BAR,
            vec![
                (
                    "File",
                    vec![
                        MenuItem::new("New tab").shortcut("c"),
                        MenuItem::new("Split right").shortcut("%"),
                        MenuItem::new("Export…").separator(),
                        MenuItem::new("Close tab").shortcut("&").danger(),
                    ],
                ),
                (
                    "View",
                    vec![
                        MenuItem::new("Zoom pane").shortcut("z"),
                        MenuItem::new("Redraw").shortcut("r").separator(),
                        MenuItem::new("Usage").shortcut("u"),
                        MenuItem::new("Inspect changes").disabled(true),
                    ],
                ),
                ("Help", vec![MenuItem::new("Key reference").shortcut("?")]),
            ],
        )
        .brand(Lockup::new("app❯"));
        let items = [
            ("1 Claude Code (Work)", "working"),
            ("2 Codex (Primary)", "idle"),
            ("3 Shell", ""),
            ("4 docs", "blocked"),
        ]
        .into_iter()
        .map(|(l, m)| ListItem::new(l).meta(m))
        .collect();
        Self {
            bar,
            sessions: ListBox::new(SESSIONS, items, SelectMode::Single),
            context: RowMenu::default(),
            last: "nothing yet".into(),
            zoomed: false,
            hint_area: Rect::ZERO,
        }
    }

    fn status_bar(&self) -> StatusBar {
        let mut b = StatusBar::new();
        b.left.push(
            StatusItem::new("payments-platform", Tone::Normal)
                .strong()
                .priority(9),
        );
        b.left.push(
            StatusItem::new("PR #482 · settlement backoff", Tone::Secondary)
                .priority(7)
                .clickable(PR),
        );
        b.center
            .push(StatusItem::new("Claude Code · working · 2 tabs", Tone::Secondary).priority(4));
        b.right.push(
            StatusItem::new("Weekly 59%", Tone::Warning)
                .chip()
                .priority(6)
                .clickable(USAGE),
        );
        b.right.push(
            StatusItem::new("jackin-payments-7f3a", Tone::Muted)
                .chip()
                .priority(3),
        );
        b.right
            .push(StatusItem::new("run 9c41", Tone::Faint).priority(2));
        b
    }

    /// Open the tab menu about session `row`, as `req` places it.
    fn open_context(&mut self, req: ContextRequest<usize>) {
        let row = req.target;
        let label = self.sessions.items[row].label.clone();
        self.context
            .open(req.menu(CONTEXT, self.context_items(row)).title(label), row);
    }

    /// The tab menu's items for session `row`.
    fn context_items(&self, row: usize) -> Vec<MenuItem> {
        vec![
            MenuItem::new("Change title…").shortcut("r"),
            MenuItem::new("Move left").disabled(row == 0),
            MenuItem::new("Move right")
                .disabled(row + 1 == self.sessions.items.len())
                .separator(),
            MenuItem::new("Close").shortcut("x").danger(),
        ]
    }

    fn layer(&self, focus: Option<WidgetId>) -> HintLayer {
        let menu = self.bar.is_open().then(|| {
            HintLayer::new(vec![
                hint("↑↓", "Move"),
                hint("←→", "Switch menu"),
                hint("Enter", "Choose"),
                hint("Esc", "Close"),
            ])
        });
        let context = self.context.is_open().then(|| {
            HintLayer::new(vec![
                hint("↑↓", "Move"),
                hint("Enter", "Choose"),
                hint("Esc", "Close"),
            ])
        });
        let screen = Some(if focus == Some(BAR) {
            HintLayer::new(vec![
                hint("←→", "Menu"),
                hint("Enter", "Open"),
                hint("Tab", "Next"),
            ])
        } else {
            HintLayer::new(vec![
                hint("↑↓", "Move"),
                hint("Shift+F10", "Context menu"),
                hint("right-click", "Context menu"),
                hint("Tab", "Next"),
            ])
            .status(format!("last: {}", self.last), Tone::Secondary)
        });
        let mut l = HintBar::resolve(&[menu, context, screen]);
        if self.zoomed {
            l = l.badge("ZOOM", BadgeKind::Edit);
        }
        l
    }

    fn act(&mut self, menu: usize, item: usize, cx: &mut PageCtx) {
        let label = self.bar.menus[menu][item].label.clone();
        if menu == 1 && item == 0 {
            self.zoomed = !self.zoomed;
        }
        self.last = format!("{} › {label}", self.bar.labels[menu]);
        cx.status(label);
    }
}

impl Page for ChromePage {
    fn layer_open(&self) -> bool {
        self.bar.hides_page_focus() || self.context.hides_page_focus()
    }
    fn title(&self) -> &'static str {
        "Chrome"
    }
    fn blurb(&self) -> &'static str {
        "Brand lockup · menu bar with anchored menus · status bar planes and priorities · context menu · hint layers"
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        // row 0: menu bar on the canvas
        let bar_row = Rect::new(area.x, area.y, area.width, 1);
        self.bar.render(bar_row, buf, ctx, t.canvas);
        // body: a list of sessions in a card
        let body = Rect::new(
            area.x,
            area.y + 2,
            area.width,
            area.height.saturating_sub(6),
        );
        let panel = Panel::card(Some("Sessions"))
            .meta("right-click or Shift+F10 for the tab menu")
            .focused(ctx.interaction.focused(SESSIONS));
        let bg = panel.bg(t);
        let inner = panel.render(body, buf, t);
        let list_area = Rect::new(inner.x, inner.y, inner.width.min(48), inner.height.min(6));
        self.sessions.render(list_area, buf, ctx, bg);
        let notes = [
            "The status bar below sits on its own plane: three groups, no",
            "separator glyphs, and items leave by priority when the row is",
            "narrow — resize the terminal to watch the center go first.",
            "",
            "Brand: one lockup, accent-filled, the only accent-filled control.",
        ];
        let nx = inner.x + list_area.width + 4;
        for (i, n) in notes.iter().enumerate() {
            let y = inner.y + i as u16;
            if nx < inner.right() && y < inner.bottom() {
                buf.set_string(
                    nx,
                    y,
                    junie_tui::ui::text::truncate(n, inner.right().saturating_sub(nx) as usize),
                    t.muted().bg(bg),
                );
            }
        }
        // status bar row above the page footer
        let status_row = Rect::new(area.x, area.bottom().saturating_sub(3), area.width, 1);
        self.status_bar().render(status_row, buf, ctx);
        // the hint layer that a shell would pin to the bottom
        let hint_row = Rect::new(area.x, area.bottom().saturating_sub(1), area.width, 1);
        self.hint_area = hint_row;
        let layer = self.layer(ctx.interaction.focus);
        buf.set_string(
            area.x + 1,
            hint_row.y.saturating_sub(1),
            "hint bar · topmost layer wins:",
            t.faint().bg(t.canvas),
        );
        HintBar::render(hint_row, buf, t, &layer);
        // popovers last so they sit on top
        self.context.render(area, buf, ctx);
    }

    fn handle(&mut self, ev: &PageEvent, cx: &mut PageCtx) -> Outcome {
        if let Some((o, chosen)) = self.context.handle(ev) {
            if let Some((row, i)) = chosen {
                let label = self.context_items(row)[i].label.clone();
                self.last = format!("tab › {label}");
                cx.status(label);
            }
            return o;
        }
        match ev {
            PageEvent::Key(key) => {
                if self.bar.is_open() || cx.focus.is(BAR) {
                    let (o, ev) = self.bar.on_key(key);
                    if let Some(MenuBarAction::Chosen(m, i)) = ev {
                        self.act(m, i, cx);
                    }
                    if o.consumed() {
                        return o;
                    }
                }
                if cx.focus.is(SESSIONS) {
                    if let Some(req) = self.context.config.on_key(&self.sessions, key) {
                        self.open_context(req);
                        return Outcome::Changed;
                    }
                    if key.code == KeyCode::F(10) && key.plain() {
                        cx.focus.focus(BAR);
                        self.bar.open_menu(0);
                        return Outcome::Changed;
                    }
                    return self.sessions.on_key(key);
                }
                Outcome::Ignored
            }
            PageEvent::Secondary { id, pos } => {
                if let Some(req) = self
                    .context
                    .config
                    .on_secondary(&mut self.sessions, *id, *pos)
                {
                    cx.focus.focus(SESSIONS);
                    self.open_context(req);
                    return Outcome::Changed;
                }
                Outcome::Ignored
            }
            // a title opens its menu on the press (MenuBar::open_on)
            PageEvent::Press { id, .. } if self.bar.owns(*id) => {
                let (o, ev) = self.bar.on_press(*id);
                if let Some(MenuBarAction::Opened(_)) = ev {
                    cx.focus.focus(BAR);
                }
                o
            }
            PageEvent::Click { id, .. } => {
                if self.bar.owns(*id) || self.bar.is_open() {
                    let (o, ev) = self.bar.on_click(*id);
                    match ev {
                        Some(MenuBarAction::Chosen(m, i)) => self.act(m, i, cx),
                        Some(MenuBarAction::Brand) => {
                            self.last = "brand".into();
                            cx.status("Brand lockup activated");
                        }
                        Some(MenuBarAction::Opened(_)) => cx.focus.focus(BAR),
                        _ => {}
                    }
                    if o.consumed() {
                        return o;
                    }
                }
                if let Some(row) = self.sessions.locate(*id) {
                    cx.focus.focus(SESSIONS);
                    return self.sessions.on_click(row);
                }
                if *id == USAGE || *id == PR {
                    self.last = if *id == USAGE {
                        "status › usage".into()
                    } else {
                        "status › PR".into()
                    };
                    cx.status(if *id == USAGE {
                        "Usage chip"
                    } else {
                        "PR context"
                    });
                    return Outcome::Changed;
                }
                Outcome::Ignored
            }
            PageEvent::Wheel { id, delta } => {
                if self.sessions.owns(*id) {
                    return self.sessions.on_wheel(*delta);
                }
                Outcome::Ignored
            }
            _ => Outcome::Ignored,
        }
    }

    fn hints(&self, focus: Option<WidgetId>) -> Vec<Hint> {
        self.layer(focus)
            .hints
            .iter()
            // the chrome layers are all literal hints, so every action is borrowed
            .filter_map(|h| match h.action {
                std::borrow::Cow::Borrowed(a) => Some((h.key, a)),
                std::borrow::Cow::Owned(_) => None,
            })
            .collect()
    }
}
