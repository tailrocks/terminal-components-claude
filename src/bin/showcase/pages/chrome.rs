use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::app::PageId;
use crate::options::page_options;
use crate::pages::row_menu::RowMenu;
use crate::pages::template::{Section, Template};
use crate::pages::{Cx, Hint, Page, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::{Part, WidgetId};
use junie_tui::core::intent::Phase;
use junie_tui::core::keymap::NavKeys;
use junie_tui::core::pointer::MouseTarget;
use junie_tui::theme::{BadgeKind, StatusKind, Tone};
use junie_tui::ui::context::ContextRequest;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::ui::popup::Dismiss;
use junie_tui::ui::specimen::{SpecimenState, StateGrid};
use junie_tui::ui::tooltip::Tip;
use junie_tui::widgets::brand::Lockup;
use junie_tui::widgets::hintbar::{HintBar, HintLayer};
use junie_tui::widgets::keyhint::hint;
use junie_tui::widgets::list::{ListBox, ListItem, SelectMode};
use junie_tui::widgets::menu::{ContextMenu, MenuBar, MenuBarAction, MenuItem, OpenOn, Overflow};
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::progress::MeterTone;
use junie_tui::widgets::segments::{self, Segment};
use junie_tui::widgets::statusbar::{Group, StatusBar, StatusItem};
use junie_tui::widgets::viewport::{Span, render_spans};
use ratatui::crossterm::event::KeyCode;

const ID: WidgetId = WidgetId::of("chrome");
const BAR: WidgetId = WidgetId::of("chrome.menubar");
const SESSIONS: WidgetId = WidgetId::of("chrome.sessions");
const CONTEXT: WidgetId = WidgetId::of("chrome.context");
const USAGE: WidgetId = WidgetId::of("chrome.status.usage");
const PR: WidgetId = WidgetId::of("chrome.status.pr");
const ENV: WidgetId = WidgetId::of("chrome.strip.env");
const PENDING: WidgetId = WidgetId::of("chrome.strip.pending");

const NARROW_BAR: u16 = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Target {
    Session(usize),

    Env,
}

pub struct ChromePage {
    template: Template,
    bar: MenuBar,
    sessions: ListBox,
    context: RowMenu<Target>,
    last: String,

    action: Option<MenuBarAction>,
    zoomed: bool,
    hint_area: Rect,

    opts: Vec<usize>,
}

impl ChromePage {
    pub fn new() -> Self {
        let opts: Vec<usize> = page_options(PageId::Chrome)
            .iter()
            .map(|o| o.default)
            .collect();
        let items = [
            ("1 Claude Code (Work)", "working"),
            ("2 Codex (Primary)", "idle"),
            ("3 Shell", ""),
            ("4 docs", "blocked"),
        ]
        .into_iter()
        .map(|(l, m)| ListItem::new(l).meta(m))
        .collect();
        let mut page = Self {
            template: Template::new(ID, states_grid()),
            bar: MenuBar::new(BAR, Vec::new()),
            sessions: ListBox::new(SESSIONS, items, SelectMode::Single),
            context: RowMenu::default(),
            last: "nothing yet".into(),
            action: None,
            zoomed: false,
            hint_area: Rect::ZERO,
            opts,
        };
        page.bar = page.build_bar();
        page
    }

    fn opt(&self, key: &str) -> &'static str {
        let table = page_options(PageId::Chrome);
        let i = table
            .iter()
            .position(|o| o.key == key)
            .unwrap_or_else(|| panic!("no option {key}"));
        table[i].values[self.opts[i]]
    }

    fn on(&self, key: &str) -> bool {
        self.opt(key) == "on"
    }

    fn build_bar(&self) -> MenuBar {
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
                    "Edit",
                    vec![
                        MenuItem::new("Copy").shortcut("y"),
                        MenuItem::new("Paste").shortcut("p"),
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
                (
                    "Session",
                    vec![
                        MenuItem::new("Rename…").shortcut("n"),
                        MenuItem::new("Detach").shortcut("d"),
                    ],
                ),
                ("Help", vec![MenuItem::new("Key reference").shortcut("?")]),
            ],
        )
        .open_on(match self.opt("chrome.open_on") {
            "release" => OpenOn::Release,
            _ => OpenOn::Press,
        })
        .overflow(match self.opt("chrome.overflow") {
            "chevron" => Overflow::Chevron,
            _ => Overflow::Hide,
        })
        .hover_switch(self.on("chrome.hover_switch"))
        .press_drag(self.on("chrome.press_drag"))
        .hide_owner_focus(self.on("chrome.hide_owner_focus"))
        .wrap(self.on("chrome.wrap"))
        .type_ahead(self.on("chrome.type_ahead"))
        .nav(match self.opt("chrome.nav") {
            "vim" => NavKeys::VIM,
            _ => NavKeys::default(),
        })
        .dismiss(match self.opt("chrome.dismiss") {
            "escape" => Dismiss::ESCAPE,
            _ => Dismiss::LIGHT,
        });
        match self.opt("chrome.brand") {
            "compact" => bar.brand(Lockup::compact("app❯")),
            "none" => bar,
            _ => bar.brand(Lockup::new("app❯")),
        }
    }

    fn readout(&self) -> [String; 2] {
        let b = &self.bar;
        let on = |v: bool| if v { "on" } else { "off" };
        let nav = if b.nav == NavKeys::VIM {
            "vim"
        } else {
            "default"
        };
        let dismiss = if b.dismiss.outside_click {
            "light"
        } else {
            "escape"
        };
        let brand = match &b.brand {
            None => "none",
            Some(l) if l.compact => "compact",
            Some(_) => "lockup",
        };
        [
            format!(
                "MenuBar · open_on {:?} · overflow {:?} · hover_switch {} · press_drag {} · hide_owner_focus {}",
                b.open_on,
                b.overflow,
                on(b.hover_switch),
                on(b.press_drag),
                on(b.hide_owner_focus),
            ),
            format!(
                "wrap {} · type_ahead {} · nav {nav} · dismiss {dismiss} · brand {brand}",
                on(b.wrap),
                on(b.type_ahead),
            ),
        ]
    }

    fn strip(&self, tick: u64, t: &junie_tui::theme::Theme) -> (Vec<Segment>, Vec<Segment>) {
        let g = &t.glyphs;
        let left = vec![
            Segment::new("payments-platform", Tone::Normal)
                .bold()
                .priority(9),
            Segment::new("production", Tone::Danger)
                .lead(g.env_production, Tone::Danger)
                .clickable(ENV)
                .priority(8),
            Segment::new("staging", Tone::Warning)
                .lead(g.env_staging, Tone::Warning)
                .priority(3),
            Segment::new("deploying", Tone::Secondary)
                .lead(g.spinner_frame(tick), Tone::Active)
                .priority(6),
        ];
        let right = vec![
            Segment::new("3 pending", Tone::Secondary)
                .lead(g.bullet, Tone::Warning)
                .clickable(PENDING)
                .priority(5),
            Segment::new("main ↑2", Tone::Muted).priority(2),
        ];
        (left, right)
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
                .clickable(PR)
                .tooltip(Tip::new("Open the pull request").shortcut("p")),
        );
        b.center.push(
            StatusItem::new("Claude Code · working · 2 tabs", Tone::Secondary)
                .busy()
                .priority(4),
        );
        b.right.push(
            StatusItem::new("Weekly 59%", Tone::Warning)
                .chip()
                .priority(6)
                .clickable(USAGE),
        );
        b.right.push(
            StatusItem::new("disk", Tone::Muted)
                .meter(Some(72), MeterTone::Level { warn: 80, crit: 95 })
                .priority(5),
        );
        b.right.push(
            StatusItem::new("quota", Tone::Muted)
                .meter(Some(88), MeterTone::Normal)
                .meter_stale(true)
                .priority(5),
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

    fn open_context(&mut self, req: ContextRequest<usize>) {
        let row = req.target;
        let label = self.sessions.items[row].label.clone();
        self.context.open(
            req.menu(CONTEXT, self.context_items(Target::Session(row)))
                .title(label),
            Target::Session(row),
        );
    }

    fn context_items(&self, target: Target) -> Vec<MenuItem> {
        match target {
            Target::Session(row) => vec![
                MenuItem::new("Change title…").shortcut("r"),
                MenuItem::new("Move left").disabled(row == 0),
                MenuItem::new("Move right")
                    .disabled(row + 1 == self.sessions.items.len())
                    .separator(),
                MenuItem::new("Close").shortcut("x").danger(),
            ],
            Target::Env => vec![
                MenuItem::new("Switch to staging"),
                MenuItem::new("Copy cluster URL").shortcut("y"),
            ],
        }
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
            .status_parts(
                vec![
                    ("last: ".to_owned(), Tone::Muted),
                    (self.last.clone(), Tone::Secondary),
                ],
                Tone::Secondary,
            )
        });
        let mut l = HintBar::resolve(&[menu, context, screen]);
        if self.zoomed {
            l = l.badge("ZOOM", BadgeKind::Edit);
        }
        l
    }

    fn act(&mut self, menu: usize, item: usize, cx: &mut Cx) {
        let label = self.bar.menus[menu][item].item.label.clone();
        if menu == 2 && item == 0 {
            self.zoomed = !self.zoomed;
        }
        self.last = format!("{} › {label}", self.bar.labels[menu]);
        cx.status(label);
    }

    fn bar_action(&mut self, ev: Option<MenuBarAction>, cx: &mut Cx) {
        let Some(ev) = ev else { return };
        self.action = Some(ev);
        match ev {
            MenuBarAction::Chosen(m, i) => self.act(m, i, cx),
            MenuBarAction::Brand => {
                self.last = "brand".into();
                cx.status("Brand lockup activated");
            }
            MenuBarAction::Opened(_) => cx.focus.focus(BAR),
            MenuBarAction::Closed => {}
        }
    }

    fn render_playground(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let mut y = area.y;

        let bar_w = if self.opt("chrome.bar") == "narrow" {
            area.width.min(NARROW_BAR)
        } else {
            area.width
        };
        self.bar
            .render(Rect::new(area.x, y, bar_w, 1), buf, ctx, t.canvas);
        y += 2;

        let foot = area.bottom().saturating_sub(5);

        let body = Rect::new(area.x, y, area.width, foot.saturating_sub(y).min(11));
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
            let ny = inner.y + i as u16;
            if nx < inner.right() && ny < inner.bottom() {
                buf.set_string(
                    nx,
                    ny,
                    junie_tui::ui::text::truncate(n, inner.right().saturating_sub(nx) as usize),
                    t.muted().bg(bg),
                );
            }
        }

        let sy = inner.y + notes.len() as u16 + 1;
        if nx + 8 < inner.right() && sy + 1 < inner.bottom() {
            let max = inner.right().saturating_sub(nx);
            let line = [
                Span::muted("checks "),
                Span::new("2 ok", Tone::Success),
                Span::muted(" · "),
                Span::new("1 failed", Tone::Error).bold(),
                Span::muted(" · "),
                Span::new("1 cancelled", Tone::Muted),
            ];
            render_spans(buf, nx, sy, max, t, &line, bg);
            junie_tui::widgets::keyhint::render_status(
                Rect::new(nx, sy + 1, max.min(44), 1),
                buf,
                t,
                &[hint("Ctrl+S", "Save")],
                None,
                Some(("Saved", Some(StatusKind::Ok))),
            );
        }

        let mut y = body.bottom() + 1;
        for line in self.readout() {
            if y < foot {
                let text = junie_tui::ui::text::truncate(&line, area.width as usize);
                buf.set_string(area.x, y, text, t.faint().bg(t.canvas));
            }
            y += 1;
        }

        if y < foot {
            let last = match self.action {
                Some(a) => format!("{a:?}"),
                None => "—".into(),
            };
            let text = format!(
                "last MenuBarAction: {last}   (Opened(menu) · Chosen(menu, item) · Closed · Brand)"
            );
            buf.set_string(
                area.x,
                y,
                junie_tui::ui::text::truncate(&text, area.width as usize),
                t.faint().bg(t.canvas),
            );
        }
        y += 2;
        if y + 1 < foot {
            buf.set_string(
                area.x,
                y,
                "identity strip · segments drop by priority · ◆ opens the app menu",
                t.faint().bg(t.canvas),
            );
            let (left, right) = self.strip(ctx.interaction.tick, t);
            let row = Rect::new(area.x, y + 1, area.width, 1);
            junie_tui::ui::ctx::fill(buf, row, ratatui::style::Style::new().bg(t.surface));
            segments::render(row, buf, ctx, &left, &right, t.surface);
        }
        let ay = y + 2;

        let status_row = Rect::new(area.x, area.bottom().saturating_sub(4), area.width, 1);
        let status = self.status_bar();
        let placed = status.layout(status_row);
        status.render(status_row, buf, ctx);
        if status_row.y > ay + 1 {
            let groups: Vec<String> = [Group::Left, Group::Center, Group::Right]
                .into_iter()
                .map(|g| {
                    let n = placed.iter().filter(|p| p.group == g).count();
                    format!("{g:?} {n}")
                })
                .collect();
            let text = format!(
                "status bar · StatusBar::layout places {} of 8 · {}",
                placed.len(),
                groups.join(", ")
            );
            buf.set_string(
                area.x + 1,
                status_row.y - 1,
                junie_tui::ui::text::truncate(&text, area.width.saturating_sub(1) as usize),
                t.faint().bg(t.canvas),
            );
        }

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

        self.context.render(area, buf, ctx);
    }
}

fn states_grid() -> StateGrid {
    use SpecimenState::*;
    let id = ID.sub("states");
    StateGrid::new(id)
        .rows(&[Rest, Hover, Pressed, Busy, Narrow])
        .column_width(26)
        .column(
            "Segment",
            Segment::new("payments-platform", Tone::Normal).bold(),
        )
        .column(
            "Clickable segment",
            Segment::new("production", Tone::Danger)
                .lead("◆", Tone::Danger)
                .clickable(id.child(1)),
        )
        .column(
            "Status item",
            StatusItem::new("PR #482 · settlement backoff", Tone::Secondary).clickable(id.child(2)),
        )
        .column(
            "Status chip",
            StatusItem::new("Weekly 59%", Tone::Warning)
                .chip()
                .clickable(id.child(3)),
        )
        .column(
            "Status meter",
            StatusItem::new("disk", Tone::Muted).meter(Some(72), MeterTone::Normal),
        )
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

    fn animating(&self) -> bool {
        true
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let body = self.template.render(area, buf, ctx);
        match self.template.section() {
            Section::Playground => self.render_playground(body, buf, ctx),
            Section::Options => self.template.render_options(body, buf, ctx),
            Section::States => self.template.render_states(body, buf, ctx),
        }
    }

    fn handle(&mut self, ev: &PageEvent, cx: &mut Cx) -> Outcome {
        if let Some(o) = self.template.handle(ev, cx) {
            return o;
        }
        if let Some((o, chosen)) = self.context.handle(ev) {
            if let Some((target, i)) = chosen {
                let label = self.context_items(target)[i].item.label.clone();
                self.last = match target {
                    Target::Session(_) => format!("tab › {label}"),
                    Target::Env => format!("production › {label}"),
                };
                cx.status(label);
            }
            return o;
        }
        match ev {
            PageEvent::Key(key) => {
                if self.bar.is_open() || cx.focus.is(BAR) {
                    let (o, ev) = self.bar.on_key(key);
                    self.bar_action(ev, cx);
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
            PageEvent::Pointer(ev) if ev.is(Phase::Secondary) => {
                if let Some(req) = self.context.config.on_secondary(&mut self.sessions, ev) {
                    cx.focus.focus(SESSIONS);
                    self.open_context(req);
                    return Outcome::Changed;
                }
                Outcome::Ignored
            }

            PageEvent::Pointer(ev) => {
                if let Some(r) = self.bar.route(ev) {
                    let (o, a) = r.into_parts();
                    self.bar_action(a, cx);
                    if o.consumed() || !ev.is(Phase::Click) {
                        return o;
                    }
                }
                if ev.on(SESSIONS) && ev.is(Phase::Click) && ev.index_of(Part::ROW).is_some() {
                    cx.focus.focus(SESSIONS);
                }
                if let Some(r) = self.sessions.route(ev) {
                    return r.outcome();
                }
                if !ev.is(Phase::Click) {
                    return Outcome::Ignored;
                }
                let id = ev.id;
                if id == ENV {
                    let menu = ContextMenu::new(CONTEXT, self.context_items(Target::Env))
                        .at(ev.pos())
                        .title("production");
                    self.context.open(menu, Target::Env);
                    return Outcome::Changed;
                }
                if id == PENDING {
                    self.last = "strip › 3 pending".into();
                    cx.status("3 pending changes");
                    return Outcome::Changed;
                }
                if id == USAGE || id == PR {
                    self.last = if id == USAGE {
                        "status › usage".into()
                    } else {
                        "status › PR".into()
                    };
                    cx.status(if id == USAGE {
                        "Usage chip"
                    } else {
                        "PR context"
                    });
                    return Outcome::Changed;
                }
                Outcome::Ignored
            }
            _ => Outcome::Ignored,
        }
    }

    fn hints(&self, focus: Option<WidgetId>) -> Vec<Hint> {
        if let Some(h) = self.template.hints(focus) {
            return h;
        }
        self.layer(focus)
            .hints
            .iter()
            .filter_map(|h| match h.action {
                std::borrow::Cow::Borrowed(a) => Some((h.key, a)),
                std::borrow::Cow::Owned(_) => None,
            })
            .collect()
    }

    fn option_value(&self, key: &str) -> Option<usize> {
        let i = page_options(PageId::Chrome)
            .iter()
            .position(|o| o.key == key)?;
        Some(self.opts[i])
    }

    fn set_option(&mut self, key: &str, value: usize) {
        let table = page_options(PageId::Chrome);
        let Some(i) = table.iter().position(|o| o.key == key) else {
            return;
        };
        self.opts[i] = value.min(table[i].values.len() - 1);
        self.bar.close();
        self.bar = self.build_bar();
    }

    fn template(&self) -> Option<&Template> {
        Some(&self.template)
    }

    fn template_mut(&mut self) -> Option<&mut Template> {
        Some(&mut self.template)
    }
}
