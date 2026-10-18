use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Rect;

use crate::pages::template::{Section, Template};
use crate::pages::{Cx, Hint, Page, PageEvent};
use junie_tui::core::event::{ClientPlatform, InputConfig, Key, Outcome};
use junie_tui::core::focus::cancel_focus_request;
use junie_tui::core::id::{Part, WidgetId};
use junie_tui::core::intent::Phase;
use junie_tui::core::keyboard::{self, KeyboardProtocol, ProtocolState};
use junie_tui::core::keymap::{KeyBinding, Nav, NavKeys, select_all_bindings};
use junie_tui::core::pointer::MouseTarget;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::ui::specimen::{SpecimenState, StateGrid};
use junie_tui::widgets::button::{Button, row_layout};
use junie_tui::widgets::input::{InputAction, TextInput};
use junie_tui::widgets::list::{ListBox, ListItem, SelectMode};
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::tabs::Tabs;

const ID: WidgetId = WidgetId::of("focusnav");

const NAVS: [Nav; 8] = [
    Nav::Up,
    Nav::Down,
    Nav::Left,
    Nav::Right,
    Nav::PageUp,
    Nav::PageDown,
    Nav::Home,
    Nav::End,
];

fn candidate_keys() -> Vec<(&'static str, Key)> {
    let k = |code| Key {
        code,
        mods: KeyModifiers::NONE,
    };
    vec![
        ("↑", k(KeyCode::Up)),
        ("↓", k(KeyCode::Down)),
        ("←", k(KeyCode::Left)),
        ("→", k(KeyCode::Right)),
        ("k", k(KeyCode::Char('k'))),
        ("j", k(KeyCode::Char('j'))),
        ("h", k(KeyCode::Char('h'))),
        ("l", k(KeyCode::Char('l'))),
        ("PgUp", k(KeyCode::PageUp)),
        ("PgDn", k(KeyCode::PageDown)),
        ("Home", k(KeyCode::Home)),
        ("End", k(KeyCode::End)),
        ("g", k(KeyCode::Char('g'))),
        ("G", k(KeyCode::Char('G'))),
    ]
}

fn nav_map(nav: NavKeys) -> Vec<(Nav, String)> {
    let keys = candidate_keys();
    NAVS.iter()
        .map(|&n| {
            let ks: Vec<&str> = keys
                .iter()
                .filter(|(_, k)| nav.action(k) == Some(n))
                .map(|(l, _)| *l)
                .collect();
            (
                n,
                if ks.is_empty() {
                    "—".into()
                } else {
                    ks.join(" ")
                },
            )
        })
        .collect()
}

fn nav_keys(v: usize) -> NavKeys {
    match v {
        1 => NavKeys::DESKTOP.vim(true),
        2 => NavKeys::DESKTOP.page(false),
        3 => NavKeys::DESKTOP.home_end(false),
        4 => NavKeys::DESKTOP.vim(true).arrows(false),
        _ => NavKeys::DESKTOP,
    }
}

pub struct FocusNavPage {
    template: Template,
    pub name: TextInput,
    pub email: TextInput,
    pub save: Button,
    pub cancel: Button,

    pub tags: Vec<Button>,
    pub list: ListBox,

    nav: usize,
    tab_stays: bool,

    pub last_change: String,

    ring_len: usize,

    pub last_nav: String,
}

impl FocusNavPage {
    pub fn new() -> Self {
        let tags = ["Alpha", "Beta", "Gamma", "Delta"]
            .iter()
            .enumerate()
            .map(|(i, t)| Button::secondary(ID.sub("tag").child(i), t))
            .collect();
        let items = [
            "Inbox", "Drafts", "Sent", "Archive", "Spam", "Trash", "Notes", "Later", "Travel",
            "Receipts", "Family", "Work",
        ]
        .iter()
        .map(|s| ListItem::new(s))
        .collect();
        Self {
            template: Template::new(ID, states_grid()),
            name: TextInput::new(ID.child(0), "Name").value("Ada Lovelace"),
            email: TextInput::new(ID.child(1), "Email").value("ada@example.com"),
            save: Button::primary(ID.child(2), "Save"),
            cancel: Button::subtle(ID.child(3), "Cancel"),
            tags,
            list: ListBox::new(ID.child(4), items, SelectMode::Single).nav(NavKeys::DESKTOP),
            nav: 0,
            tab_stays: false,
            last_change: "none yet".into(),
            ring_len: 0,
            last_nav: "none yet".into(),
        }
    }

    fn name_of(&self, id: Option<WidgetId>) -> String {
        let Some(id) = id else {
            return "nothing".into();
        };
        if id == self.name.id {
            return "Name field".into();
        }
        if id == self.email.id {
            return "Email field".into();
        }
        if id == self.save.id {
            return "Save".into();
        }
        if id == self.cancel.id {
            return "Cancel".into();
        }
        if id == self.list.id {
            return "Mailbox list".into();
        }
        if id == self.template.tabs.id {
            return "section strip".into();
        }
        if let Some(t) = self.tags.iter().find(|t| t.id == id) {
            return format!("{} tag", t.label);
        }
        "outside the page".into()
    }

    fn render_playground(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let rows = crate::pages::layout::rows(area, &[10, 1, 11, 1, 7, 0]);
        let half = rows[0].width / 2;
        let (left, right) = crate::pages::layout::columns(rows[0], half, 2);

        let panel = Panel::card(Some("Focus ring")).meta("Tab · Shift+Tab · click");
        let bg = panel.bg(t);
        let inner = panel.render(left, buf, t);
        let w = inner.width.min(30);
        self.name
            .render(Rect::new(inner.x, inner.y, w, 2), buf, ctx, bg);
        self.email
            .render(Rect::new(inner.x, inner.y + 2, w, 2), buf, ctx, bg);
        let y = inner.y + 5;
        let widths = [self.save.width(), self.cancel.width()];
        let r = row_layout(Rect::new(inner.x, y, inner.width, 1), &widths, 2);
        self.save.render(r[0], buf, ctx, bg);
        self.cancel.render(r[1], buf, ctx, bg);
        let y = inner.y + 7;
        let widths: Vec<u16> = self.tags.iter().map(|b| b.width()).collect();
        let r = row_layout(Rect::new(inner.x, y, inner.width, 1), &widths, 1);
        for (b, r) in self.tags.iter_mut().zip(r) {
            b.render(r, buf, ctx, bg);
        }

        let panel = Panel::card(Some("Focus"));
        let bg = panel.bg(t);
        let inner = panel.render(right, buf, t);
        let focus = self.name_of(ctx.interaction.focus);
        let ring = if self.ring_len > 0 {
            format!("{} stops · reading order", self.ring_len)
        } else {
            "reading order".into()
        };
        let lines = [
            ("Focus", focus),
            ("Change", self.last_change.clone()),
            ("Ring", ring),
            ("Fallback", "Delete on a tag removes it".into()),
            (
                "Field Tab",
                if self.tab_stays {
                    "stays: cancel_focus_request".into()
                } else {
                    "moves: request_focus_next".into()
                },
            ),
        ];
        draw_lines(inner, buf, t, bg, &lines);

        let (left, right) = crate::pages::layout::columns(rows[2], half, 2);
        let panel = Panel::card(Some("Mailboxes"))
            .meta("nav()")
            .focused(ctx.interaction.focused(self.list.id));
        let bg = panel.bg(t);
        let inner = panel.render(left, buf, t);
        self.list.render(inner, buf, ctx, bg);
        let panel = Panel::card(Some("Nav map")).meta(&self.last_nav);
        let bg = panel.bg(t);
        let inner = panel.render(right, buf, t);
        let map = nav_map(self.list.nav);
        let lines: Vec<(&str, String)> = map
            .iter()
            .map(|(n, k)| (nav_label(*n), k.clone()))
            .collect();
        draw_lines(inner, buf, t, bg, &lines);

        let panel = Panel::card(Some("Keys and input"));
        let bg = panel.bg(t);
        let inner = panel.render(rows[4], buf, t);
        let select_all: Vec<String> = select_all_bindings(())
            .filter(|(b, _)| b.deliverable())
            .map(|(b, _)| b.label())
            .collect();
        let protocol = KeyboardProtocol::current();

        let stand_in = KeyBinding::new(
            KeyCode::Left,
            ratatui::crossterm::event::KeyModifiers::CONTROL
                | ratatui::crossterm::event::KeyModifiers::SHIFT,
        );

        let mut state = ProtocolState::default();
        let push = state
            .enter(KeyboardProtocol::On, || false)
            .unwrap_or_default();
        let pop = state.leave().unwrap_or("");

        let mac = InputConfig::for_platform(ClientPlatform::MacOs);
        let other = InputConfig::for_platform(ClientPlatform::Other);
        let lines = [
            (
                "Bindings",
                format!(
                    "back {} · palette {} · select all {}",
                    KeyBinding::key(KeyCode::Tab).shift().label(),
                    KeyBinding::ctrl('k').label(),
                    select_all.join(" / ")
                ),
            ),
            (
                "Input",
                format!(
                    "Ctrl+click is right-click: macOS {}, elsewhere {} · Shift+wheel sideways {} · legacy Ctrl punctuation {}",
                    on_off(mac.ctrl_click_secondary),
                    on_off(other.ctrl_click_secondary),
                    on_off(mac.shift_wheel_horizontal && other.shift_wheel_horizontal),
                    on_off(mac.legacy_ctrl_punctuation && other.legacy_ctrl_punctuation)
                ),
            ),
            (
                "Protocol",
                format!(
                    "{protocol:?} (env) · active {} · Cmd delivered {} · On pushes {}, pops {}",
                    on_off(keyboard::active()),
                    on_off(keyboard::super_delivered()),
                    printable(&push),
                    printable(pop)
                ),
            ),
            (
                "Alt keys",
                format!(
                    "{:?} · delivered {} · resize {} · stand-in {}",
                    keyboard::AltKeys::current(),
                    on_off(keyboard::alt_delivered()),
                    KeyBinding::new(
                        KeyCode::Left,
                        ratatui::crossterm::event::KeyModifiers::ALT
                            | ratatui::crossterm::event::KeyModifiers::SHIFT
                    )
                    .label(),
                    stand_in.alt_stand_in().label()
                ),
            ),
        ];
        draw_lines(inner, buf, t, bg, &lines);
    }
}

fn on_off(b: bool) -> &'static str {
    if b { "on" } else { "off" }
}

fn printable(s: &str) -> String {
    if s.is_empty() {
        return "nothing".into();
    }
    s.replace('\x1b', "ESC")
}

fn nav_label(n: Nav) -> &'static str {
    match n {
        Nav::Up => "Up",
        Nav::Down => "Down",
        Nav::Left => "Left",
        Nav::Right => "Right",
        Nav::PageUp => "PageUp",
        Nav::PageDown => "PageDown",
        Nav::Home => "Home",
        Nav::End => "End",
    }
}

fn draw_lines(
    inner: Rect,
    buf: &mut Buffer,
    t: &junie_tui::theme::Theme,
    bg: ratatui::style::Color,
    lines: &[(&str, String)],
) {
    for (i, (k, v)) in lines.iter().enumerate() {
        let y = inner.y + i as u16;
        if y >= inner.bottom() {
            break;
        }
        buf.set_string(inner.x, y, k, t.muted().bg(bg));
        let w = inner.width.saturating_sub(10) as usize;
        buf.set_string(
            inner.x + 10,
            y,
            junie_tui::ui::text::truncate(v, w),
            t.primary().bg(bg),
        );
    }
}

fn states_grid() -> StateGrid {
    use SpecimenState::*;
    let id = ID.sub("states");
    StateGrid::new(id)
        .rows(&[Rest, Hover, Focused, FocusedHover])
        .column_width(24)
        .column("Button", Button::secondary(id.child(0), "Save"))
        .column("Field", TextInput::new(id.child(1), "Name").value("Ada"))
        .column("Tab strip", Tabs::new(id.child(2), &["Data", "Logs"]))
}

impl Page for FocusNavPage {
    fn title(&self) -> &'static str {
        "Focus & navigation"
    }

    fn blurb(&self) -> &'static str {
        "The focus ring, fallback, nav keys, key bindings, input and keyboard protocol"
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
        if let PageEvent::Blur(ch) = ev {
            self.last_change = format!("{} → {}", self.name_of(Some(ch.from)), self.name_of(ch.to));
            self.ring_len = cx.ring.reachable().len();
            for f in [&mut self.name, &mut self.email] {
                if f.id == ch.from {
                    f.on_blur();
                }
            }
        }
        if let Some(o) = self.template.handle(ev, cx) {
            return o;
        }
        if self.template.section() != Section::Playground {
            return Outcome::Ignored;
        }
        match ev {
            PageEvent::Blur(_) => Outcome::Changed,
            PageEvent::Key(key) => {
                let Some(f) = cx.focus.current() else {
                    return Outcome::Ignored;
                };
                if let Some(field) = [&mut self.name, &mut self.email]
                    .into_iter()
                    .find(|x| x.id == f)
                {
                    let (out, act) = field.on_key(key);
                    if matches!(act, Some(InputAction::CommittedTab { .. })) && self.tab_stays {
                        cancel_focus_request();
                    }
                    return out;
                }
                if f == self.list.id {
                    if let Some(n) = self.list.nav.action(key) {
                        self.last_nav = format!("last: {}", nav_label(n));
                    }
                    return self.list.on_key(key);
                }
                if let Some(i) = self.tags.iter().position(|b| b.id == f)
                    && key.code == KeyCode::Delete
                {
                    let gone = self.tags.remove(i);
                    cx.status(format!("Removed {}", gone.label));
                    return Outcome::Changed;
                }
                for b in [&mut self.save, &mut self.cancel]
                    .into_iter()
                    .chain(self.tags.iter_mut())
                {
                    if b.id == f {
                        let (out, activated) = b.on_key(key);
                        if activated {
                            cx.status(format!("{} ✓", b.label));
                        }
                        return out;
                    }
                }
                Outcome::Ignored
            }
            PageEvent::Pointer(ev) => {
                for field in [&mut self.name, &mut self.email] {
                    let fid = field.id;
                    if let Some(r) = field.route(ev) {
                        if ev.is(Phase::Click) {
                            cx.focus.focus(fid);
                        }
                        return r.outcome();
                    }
                }
                if ev.on(self.list.id) && ev.is(Phase::Click) && ev.index_of(Part::ROW).is_some() {
                    cx.focus.focus(self.list.id);
                }
                if let Some(r) = self.list.route(ev) {
                    return r.outcome();
                }
                for b in [&mut self.save, &mut self.cancel]
                    .into_iter()
                    .chain(self.tags.iter_mut())
                {
                    if let Some(r) = b.route(ev) {
                        if !ev.is(Phase::Click) {
                            return r.outcome();
                        }
                        if r.into_action().is_some() {
                            cx.status(format!("{} ✓", b.label));
                        }
                        return Outcome::Changed;
                    }
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
        match focus {
            Some(f) if self.tags.iter().any(|t| t.id == f) => {
                vec![("Enter / Space", "Activate"), ("Delete", "Remove tag")]
            }
            Some(f) if f == self.list.id => vec![("↑↓", "Move")],
            _ => Vec::new(),
        }
    }

    fn editing(&self) -> bool {
        self.name.editing || self.email.editing
    }

    fn option_value(&self, key: &str) -> Option<usize> {
        match key {
            "focusnavigation.nav" => Some(self.nav),
            "focusnavigation.tab_in_field" => Some(usize::from(self.tab_stays)),
            _ => None,
        }
    }

    fn set_option(&mut self, key: &str, value: usize) {
        match key {
            "focusnavigation.nav" => {
                self.nav = value;
                let items = self.list.items.clone();
                let cursor = self.list.cursor;
                let nav = nav_keys(value);

                self.list = ListBox::new(self.list.id, items, SelectMode::Single)
                    .nav(nav)
                    .type_ahead(!nav.vim);
                self.list.cursor = cursor;
            }
            "focusnavigation.tab_in_field" => self.tab_stays = value == 1,
            _ => {}
        }
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }

    fn template(&self) -> Option<&Template> {
        Some(&self.template)
    }

    fn template_mut(&mut self) -> Option<&mut Template> {
        Some(&mut self.template)
    }
}

pub fn page() -> Box<dyn Page> {
    Box::new(FocusNavPage::new())
}
