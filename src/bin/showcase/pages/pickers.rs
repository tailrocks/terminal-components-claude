//! Pickers (Overlays, plan §7.5 SC20): one searchable modal list for
//! files, tabs, levels, commands and remote branches, opened on the
//! shell's layer stack with the page as its owner.
//!
//! - **Playground**: Open quickly (fuzzy over files and tasks as you type,
//!   Tab cycles the scope, Alt+Enter the alternate action); Switch tab (a
//!   live switcher: running tabs tick and the scope counts them, Delete
//!   closes a row); Choose a level (no search box); Run command (rows
//!   with keys, glyphs and groups, disabled rows saying why); Remote
//!   branch (loading, then ready, or an error that `r` retries). A card
//!   reads the options back from a built picker, another shows the last
//!   result and action.
//! - **Options**: one control per builder (`pickers.<key>`).
//! - **States**: a picker at rest, hovered, with a disabled row, empty,
//!   loading, failed and overflowing, drawn by the picker.
//!
//! The page is wiring and demo data only: search, keys, pointer and
//! dismissal are the picker's own.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::app::PageId;
use crate::options::{PageValues, page_options};
use crate::pages::template::{Section, Template};
use crate::pages::{Cx, Hint, Page, PageEvent, Report};
use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::id::WidgetId;
use junie_tui::core::intent::Phase;
use junie_tui::core::keymap::NavKeys;
use junie_tui::core::pointer::MouseTarget;
use junie_tui::theme::CursorStyle;
use junie_tui::ui::ctx::{Backdrop, RenderCtx};
use junie_tui::ui::layer::LayerTag;
use junie_tui::ui::layers::LayerResult;
use junie_tui::ui::popup::{Dismiss, Place, Placement, VAlign};
use junie_tui::ui::specimen::{SpecimenState, StateGrid};
use junie_tui::ui::text::fuzzy;
use junie_tui::widgets::button::{Button, row_layout};
use junie_tui::widgets::empty::EmptyState;
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::picker::{Picker, PickerAction, PickerItem, PickerStatus};
use junie_tui::widgets::props::{self, Prop};
use junie_tui::widgets::tree::TreeNode;

const ID: WidgetId = WidgetId::of("pickers");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Quick,
    Tabs,
    Level,
    Commands,
    Remote,
}

impl Kind {
    const ALL: [Kind; 5] = [
        Kind::Quick,
        Kind::Tabs,
        Kind::Level,
        Kind::Commands,
        Kind::Remote,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scope {
    All,
    Files,
    Tasks,
}

#[derive(Clone)]
struct Entry {
    label: String,
    detail: String,
    glyph: &'static str,
    group: &'static str,
}

fn flatten(nodes: &[TreeNode], path: &str, out: &mut Vec<Entry>) {
    for n in nodes {
        let p = if path.is_empty() {
            n.label.clone()
        } else {
            format!("{path}/{}", n.label)
        };
        if n.children.is_empty() {
            out.push(Entry {
                label: n.label.clone(),
                detail: p.clone(),
                glyph: "F",
                group: "Files",
            });
        } else {
            flatten(&n.children, &p, out);
        }
    }
}

const LEVELS: [(&str, &str); 6] = [
    (
        "Silent",
        "Writes run without asking. Destructive statements still confirm.",
    ),
    ("Alert", "Every write asks for confirmation before it runs."),
    (
        "Alert (Full)",
        "Every statement, reads included, asks for confirmation.",
    ),
    (
        "Safe Mode",
        "Writes ask for confirmation and a deliberate acknowledgement.",
    ),
    (
        "Safe Mode (Full)",
        "Every statement asks for confirmation and a deliberate acknowledgement.",
    ),
    (
        "Read-Only",
        "Writes are refused. Reads and exports still work.",
    ),
];

/// The command palette: label, glyph, key, why it is disabled (if it is).
const COMMANDS: &[(&str, &str, &str, Option<&str>)] = &[
    ("Run query", "▶", "run.query", None),
    ("Explain query", "?", "run.explain", None),
    (
        "Export results…",
        "⇩",
        "results.export",
        Some("Run a query first"),
    ),
    ("Format SQL", "≡", "edit.format", None),
    (
        "Commit transaction",
        "✓",
        "tx.commit",
        Some("No open transaction"),
    ),
    ("Reconnect", "↻", "conn.reconnect", None),
];

const BRANCHES: &[&str] = &[
    "main",
    "feat/rate-limit",
    "fix/session-expiry",
    "chore/uuid-sessions",
    "release/4.2",
];

/// Ticks a remote fetch takes.
const FETCH_TICKS: u32 = 3;

pub struct PickersPage {
    template: Template,
    opts: PageValues,
    pub buttons: Vec<Button>,
    /// The kind of the picker the page has open on the shell's layer
    /// stack (its tag's `n`), which gives the button that opened it its
    /// focus back when it closes (K9).
    open: Option<Kind>,
    scope: Scope,
    entries: Vec<Entry>,
    /// Open tabs: name and, for a running query, its elapsed ticks.
    tabs: Vec<(String, Option<u32>)>,
    level: usize,
    chosen: Option<(String, String)>,
    pub last: String,
    opened: u32,
    /// Ticks left before the remote fetch answers; `None` when idle.
    fetching: Option<u32>,
}

/// What the pickers' rows are built from: a copy the shell hands to the
/// open picker (`Cx::with_top`).
struct Rows {
    entries: Vec<Entry>,
    scope: Scope,
    tabs: Vec<(String, Option<u32>)>,
    level: usize,
    fetching: Option<u32>,
    remote_fails: bool,
}

impl Rows {
    fn fill(&self, kind: Kind, p: &mut Picker) {
        let q = p.query.clone();
        match kind {
            Kind::Quick => {
                let mut ranked: Vec<(u32, PickerItem)> = self
                    .entries
                    .iter()
                    .filter(|e| match self.scope {
                        Scope::All => true,
                        Scope::Files => e.group == "Files",
                        Scope::Tasks => e.group == "Tasks",
                    })
                    .filter_map(|e| {
                        let (pen, matched) = fuzzy(&e.label, &q)?;
                        let mut it = PickerItem::new(&e.label)
                            .detail(&e.detail)
                            .glyph(e.glyph)
                            .group(e.group);
                        it.matched = matched;
                        Some((pen + if e.group == "Files" { 0 } else { 5 }, it))
                    })
                    .collect();
                ranked.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.label.cmp(&b.1.label)));
                p.scope = Some(format!(
                    "{} · Tab scope",
                    match self.scope {
                        Scope::All => "All",
                        Scope::Files => "Files",
                        Scope::Tasks => "Tasks",
                    }
                ));
                p.set_items(ranked.into_iter().map(|r| r.1).collect());
            }
            Kind::Tabs => {
                let items = self
                    .tabs
                    .iter()
                    .enumerate()
                    .filter_map(|(i, (t, run))| {
                        let (_, matched) = fuzzy(t, &q)?;
                        let mut it = PickerItem::new(t)
                            .detail(match run {
                                Some(s) => format!("{s}s"),
                                None if i == 0 => "query".into(),
                                None => "public · data".into(),
                            })
                            .glyph(if run.is_some() { "◌" } else { "T" })
                            .group("Open tabs")
                            .key(t.as_str());
                        it.matched = matched;
                        it.tag = (i == 1).then_some("active");
                        Some(it)
                    })
                    .collect();
                let live = self.tabs.iter().filter(|t| t.1.is_some()).count();
                p.scope = Some(format!("{live} live"));
                // a tick keeps the cursor on its row
                p.refresh_items(items);
            }
            Kind::Level => {
                p.set_items(
                    LEVELS
                        .iter()
                        .enumerate()
                        .map(|(i, (l, d))| {
                            let mut it = PickerItem::new(*l).detail(*d);
                            it.tag = (i == self.level).then_some("current");
                            it
                        })
                        .collect(),
                );
                p.cursor = self.level;
            }
            Kind::Commands => p.set_items(
                COMMANDS
                    .iter()
                    .filter(|c| fuzzy(c.0, &q).is_some())
                    .map(|(l, g, k, why)| {
                        let it = PickerItem::new(*l)
                            .glyph(g)
                            .group("Commands")
                            .key(*k)
                            .detail(why.unwrap_or(k));
                        it.disabled(why.is_some())
                    })
                    .collect(),
            ),
            Kind::Remote => match self.fetching {
                Some(_) => {
                    p.set_items(vec![]);
                    p.status = PickerStatus::Loading("Fetching branches from origin…".into());
                }
                None if self.remote_fails => {
                    p.set_items(vec![]);
                    p.status = PickerStatus::Error {
                        message: "Could not reach origin".into(),
                        detail: Some("r retries".into()),
                    };
                }
                None => {
                    p.status = PickerStatus::Ready;
                    p.set_items(
                        BRANCHES
                            .iter()
                            .map(|b| PickerItem::new(*b).glyph("⎇").group("origin"))
                            .collect(),
                    );
                }
            },
        }
    }
}

impl PickersPage {
    pub fn new() -> Self {
        let mut entries = Vec::new();
        flatten(&crate::data::project_tree(), "", &mut entries);
        for t in crate::data::tasks().into_iter().take(12) {
            entries.push(Entry {
                label: t.name,
                detail: format!("#{} · {}", t.id, t.owner),
                glyph: "T",
                group: "Tasks",
            });
        }
        Self {
            template: Template::new(ID, states_grid()),
            opts: PageValues::new(page_options(PageId::Pickers)),
            buttons: vec![
                Button::primary(ID.sub("quick"), "Open quickly"),
                Button::secondary(ID.sub("tabs"), "Switch tab"),
                Button::secondary(ID.sub("level"), "Choose a level"),
                Button::secondary(ID.sub("commands"), "Run command"),
                Button::secondary(ID.sub("remote"), "Remote branch"),
            ],
            open: None,
            scope: Scope::All,
            entries,
            tabs: vec![
                ("Query 1".into(), Some(0)),
                ("orders".into(), None),
                ("order_items".into(), Some(4)),
                ("History".into(), None),
            ],
            level: 3,
            chosen: None,
            last: "none yet · QueryChanged · Chosen(i) · ChosenAlt(i) · Submit · Secondary(i) · NextScope · PrevScope · Back · Cancelled".into(),
            opened: 0,
            fetching: None,
        }
    }

    fn kind(&self) -> Option<Kind> {
        self.open
    }

    /// Refill the open picker's rows (a query, scope, row or tick change).
    fn refill(&self, kind: Kind, cx: &mut Cx) {
        let rows = self.rows();
        cx.with_top::<Picker>(move |p| rows.fill(kind, p));
    }

    /// What the pickers' rows are built from, as they are now.
    fn rows(&self) -> Rows {
        Rows {
            entries: self.entries.clone(),
            scope: self.scope,
            tabs: self.tabs.clone(),
            level: self.level,
            fetching: self.fetching,
            remote_fails: self.opts.value("pickers.remote") == "fails",
        }
    }

    /// The builders an app calls, from the options.
    fn style(&self, p: Picker, anchor: Rect) -> Picker {
        let o = &self.opts;
        let mut p = p
            .max_rows(o.value("pickers.max_rows").parse().unwrap_or(12))
            .fit(match o.value("pickers.fit") {
                "third" => Place::DEFAULT.v_align(VAlign::Third),
                "flush" => Place::FLUSH,
                _ => Place::DEFAULT,
            })
            .backdrop(match o.value("pickers.backdrop") {
                "full" => Backdrop::FULL,
                _ => Backdrop::DEFAULT,
            })
            .cursor_style(match o.value("pickers.cursor_style") {
                "bar" => CursorStyle::Bar,
                _ => CursorStyle::Fill,
            })
            .delete_is_secondary(o.on("pickers.delete_is_secondary"))
            .nav(match o.value("pickers.nav") {
                "vim" => NavKeys::VIM,
                _ => NavKeys::default(),
            })
            .dismiss(match o.value("pickers.dismiss") {
                "escape" => Dismiss::ESCAPE,
                _ => Dismiss::LIGHT,
            });
        match o.value("pickers.width") {
            "48" => p = p.width(48),
            "96" => p = p.width(96),
            _ => {}
        }
        p = if o.value("pickers.placement") == "below" {
            p.anchor(anchor, Placement::Below)
        } else {
            p.placement(Placement::Center)
        };
        p = if o.value("pickers.empty") == "state" {
            p.empty(EmptyState::new("Nothing matches").hint("Esc clears the search"))
        } else {
            p.empty_text("No matches — Esc clears the search")
        };
        if o.value("pickers.placeholder") == "custom" {
            p = p.placeholder("Type to narrow…");
        }
        if o.value("pickers.hints") == "short" {
            p = p.hints("Enter Choose · Esc Close");
        }
        p
    }

    fn open(&mut self, kind: Kind, cx: &mut Cx) {
        let i = Kind::ALL.iter().position(|k| *k == kind).unwrap_or(0);
        let anchor = self.buttons.get(i).map_or(Rect::ZERO, |b| b.area);
        let mut p = match kind {
            Kind::Quick => Picker::new(ID.sub("picker.quick"), "Open quickly")
                .placeholder("Files and tasks…")
                .width(80),
            Kind::Tabs => Picker::new(ID.sub("picker.tabs"), "Open tabs")
                .placeholder("Filter tabs…")
                .width(48),
            Kind::Level => {
                let mut p =
                    Picker::new(ID.sub("picker.level"), "Safe Mode · this connection").width(112);
                p.searchable = false;
                p
            }
            Kind::Commands => Picker::new(ID.sub("picker.commands"), "Run command")
                .placeholder("Commands…")
                .width(64),
            Kind::Remote => {
                let mut p =
                    Picker::new(ID.sub("picker.remote"), "Remote branches · origin").width(56);
                p.searchable = false;
                p
            }
        };
        p.hints = match kind {
            Kind::Quick => {
                "↑↓ Move · Enter Open · Alt+Enter New tab · Tab Scope · Esc Clear / Close"
            }
            Kind::Tabs => "↑↓ Move · Enter Switch · Delete Close tab · Esc Close",
            Kind::Level => "↑↓ Move · Enter Set level · Esc Keep",
            Kind::Commands => "↑↓ Move · Enter Run · Esc Close",
            Kind::Remote => "↑↓ Move · Enter Check out · r Retry · Esc Close",
        }
        .to_owned();
        let mut p = self.style(p, anchor);
        self.opened += 1;
        if kind == Kind::Remote {
            self.fetching = Some(FETCH_TICKS);
        }
        self.rows().fill(kind, &mut p);
        cx.open(p, LayerTag::new("picker").n(i));
        self.open = Some(kind);
    }

    /// Handle what the open picker reported: an event keeps it open, a
    /// result closed it (and the stack gave focus back, K9).
    fn on_report(&mut self, kind: Kind, r: Report, cx: &mut Cx) {
        if r.closed.is_some() {
            self.open = None;
            if kind == Kind::Remote {
                self.fetching = None;
            }
        }
        let Some(LayerResult::Picker(ev)) = r.result else {
            return;
        };
        self.last = format!("{ev:?}");
        let picker = r.closed.as_ref().and_then(|e| e.get::<Picker>()).cloned();
        match ev {
            PickerAction::QueryChanged => self.refill(kind, cx),
            PickerAction::Chosen(i) | PickerAction::ChosenAlt(i) => {
                let alt = matches!(ev, PickerAction::ChosenAlt(_));
                if let Some(it) = picker.as_ref().and_then(|p| p.items.get(i)) {
                    if kind == Kind::Level {
                        self.level = i;
                    }
                    self.chosen = Some((it.label.clone(), it.detail.clone()));
                    cx.status(format!(
                        "{} {}",
                        if alt { "Opened in a new tab:" } else { "Chose" },
                        it.label
                    ));
                }
            }
            PickerAction::Secondary(i) => {
                if kind == Kind::Tabs && i < self.tabs.len() && self.tabs.len() > 1 {
                    let (name, _) = self.tabs.remove(i);
                    cx.status(format!("Closed {name}"));
                    self.refill(kind, cx);
                }
            }
            // free text is not an item in any showcase picker: the query
            // stays, the picker stays open
            PickerAction::Submit => {}
            PickerAction::NextScope | PickerAction::PrevScope => {
                if kind == Kind::Quick {
                    let back = ev == PickerAction::PrevScope;
                    self.scope = match (self.scope, back) {
                        (Scope::All, false) | (Scope::Tasks, true) => Scope::Files,
                        (Scope::Files, false) | (Scope::All, true) => Scope::Tasks,
                        (Scope::Tasks, false) | (Scope::Files, true) => Scope::All,
                    };
                    self.refill(kind, cx);
                }
            }
            PickerAction::Cancelled | PickerAction::Back => {
                if kind == Kind::Remote {
                    self.fetching = None;
                }
            }
        }
    }

    /// A tick: running tabs count up, a remote fetch answers.
    fn tick(&mut self, cx: &mut Cx) -> bool {
        let Some(kind) = self.kind() else {
            return false;
        };
        match kind {
            Kind::Tabs => {
                for t in &mut self.tabs {
                    if let Some(s) = &mut t.1 {
                        *s += 1;
                    }
                }
                self.refill(kind, cx);
                true
            }
            Kind::Remote => match self.fetching {
                Some(n) if n > 1 => {
                    self.fetching = Some(n - 1);
                    false
                }
                Some(_) => {
                    self.fetching = None;
                    self.refill(kind, cx);
                    true
                }
                None => false,
            },
            _ => false,
        }
    }

    /// What the options built, read back from a built picker.
    fn readout(&self) -> Vec<Prop> {
        let p = self.style(Picker::new(ID.sub("picker.sample"), "Sample"), Rect::ZERO);
        let on = |b: bool| if b { "on" } else { "off" };
        // the first command and the disabled ones, with why
        let commands = COMMANDS
            .iter()
            .enumerate()
            .filter(|(i, c)| *i == 0 || c.3.is_some())
            .map(|(_, (l, g, k, why))| match why {
                Some(w) => format!("{g} {l} ({k}, disabled: {w})"),
                None => format!("{g} {l} ({k})"),
            })
            .collect::<Vec<_>>()
            .join(" · ");
        vec![
            Prop::new(
                "Size",
                format!(
                    "width {} · max rows {} · {:?}{}",
                    p.width,
                    p.max_rows,
                    p.placement,
                    if p.placement == Placement::Center {
                        ""
                    } else {
                        " the button"
                    }
                ),
            ),
            Prop::new(
                "Fit",
                format!(
                    "margin {} · {:?} · backdrop keeps {} bottom rows",
                    p.fit.margin, p.fit.v_align, p.backdrop.keep_bottom
                ),
            ),
            Prop::new(
                "Rows",
                format!(
                    "cursor {:?} · Delete secondary {} · nav {}",
                    p.cursor_style,
                    on(p.delete_is_secondary),
                    self.opts.value("pickers.nav")
                ),
            ),
            Prop::new(
                "Close",
                format!(
                    "Esc {} · outside click {}",
                    on(p.dismiss.escape),
                    on(p.dismiss.outside_click)
                ),
            ),
            Prop::new(
                "Text",
                format!(
                    "placeholder “{}” · no matches: {} · hints “{}”",
                    p.placeholder,
                    match &p.empty {
                        Some(e) => format!("state “{}”", e.title),
                        None => format!("“{}”", p.empty_text),
                    },
                    if p.hints.is_empty() {
                        "per picker"
                    } else {
                        p.hints.as_str()
                    }
                ),
            ),
            Prop::new("Commands", commands),
            Prop::new(
                "Remote",
                format!(
                    "Loading, then {}",
                    if self.opts.value("pickers.remote") == "fails" {
                        "Error (r retries)"
                    } else {
                        "Ready"
                    }
                ),
            ),
        ]
    }

    fn render_playground(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        // as many buttons per row as fit, the rest on the next rows
        let widths: Vec<u16> = self.buttons.iter().map(|b| b.width()).collect();
        let room = area.width.saturating_sub(4);
        let mut lines: Vec<Vec<usize>> = vec![vec![]];
        let mut used = 0u16;
        for (i, w) in widths.iter().enumerate() {
            let need = if used == 0 { *w } else { used + 2 + w };
            if need > room && used > 0 {
                lines.push(vec![]);
                used = *w;
            } else {
                used = need;
            }
            lines.last_mut().expect("a line").push(i);
        }
        let n = lines.len() as u16;
        let rows = crate::pages::layout::rows(area, &[n + 5, 1, 10, 1, 0]);
        let panel = Panel::card(Some("Open a picker")).meta("Enter · click");
        let bg = panel.bg(t);
        let inner = panel.render(rows[0], buf, t);
        for (r, line) in lines.iter().enumerate() {
            let y = inner.y + r as u16;
            let ws: Vec<u16> = line.iter().map(|&i| widths[i]).collect();
            let rects = row_layout(Rect::new(inner.x, y, inner.width, 1), &ws, 2);
            for (&i, rect) in line.iter().zip(rects) {
                self.buttons[i].render(rect, buf, ctx, bg);
            }
        }
        let inner = Rect::new(
            inner.x,
            inner.y + n - 1,
            inner.width,
            inner.height.saturating_sub(n - 1),
        );
        if inner.y + 2 < inner.bottom() {
            buf.set_string(
                inner.x,
                inner.y + 2,
                junie_tui::ui::text::truncate(
                    "Quick: fuzzy as you type, Tab scope, Alt+Enter alternate · Tabs: live, Delete closes · Level: no search box · Commands: disabled rows say why · Remote: loads, r retries",
                    inner.width as usize,
                ),
                t.muted().bg(bg),
            );
        }

        let panel = Panel::card(Some("Applied")).meta("read back from a built picker");
        let bg = panel.bg(t);
        let inner = panel.render(rows[2], buf, t);
        props::render(inner, buf, t, &self.readout(), bg);

        let panel = Panel::card(Some("Result")).meta("PickerAction");
        let bg = panel.bg(t);
        let inner = panel.render(
            Rect::new(rows[4].x, rows[4].y, rows[4].width, rows[4].height.min(8)),
            buf,
            t,
        );
        let (label, detail) = self
            .chosen
            .clone()
            .unwrap_or_else(|| ("nothing yet".into(), "—".into()));
        let props = vec![
            Prop::new("Chosen", label),
            Prop::new("Detail", detail),
            Prop::new("Level", LEVELS[self.level].0),
            Prop::new(
                "Open tabs",
                self.tabs
                    .iter()
                    .map(|t| t.0.as_str())
                    .collect::<Vec<_>>()
                    .join(" · "),
            ),
            Prop::new("Last action", self.last.clone()),
            Prop::new("Pickers opened", self.opened.to_string()),
        ];
        props::render(inner, buf, t, &props, bg);
    }
}

/// The States grid: a branch picker without a search box.
fn states_grid() -> StateGrid {
    use SpecimenState::*;
    let id = ID.sub("states");
    let mut p = Picker::new(id.child(0), "Remote branches").width(44);
    p.searchable = false;
    p.set_items(
        ["main", "feat/rate-limit", "release/4.2"]
            .iter()
            .map(|b| PickerItem::new(*b).glyph("⎇"))
            .collect(),
    );
    StateGrid::new(id)
        .rows(&[Rest, Hover, Disabled, Empty, Loading, Error, Overflow])
        .column_width(50)
        .column("Branches", p)
}

impl Page for PickersPage {
    fn title(&self) -> &'static str {
        "Pickers"
    }
    fn blurb(&self) -> &'static str {
        "One modal list for files, tabs, levels, commands and branches: search, scope, status"
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
        if let PageEvent::Tick = ev {
            return if self.tick(cx) {
                Outcome::Changed
            } else {
                Outcome::Ignored
            };
        }
        // the shell routes the open picker; the page underneath takes
        // nothing meanwhile
        if self.open.is_some() {
            return Outcome::Ignored;
        }
        if let Some(o) = self.template.handle(ev, cx) {
            return o;
        }
        if self.template.section() != Section::Playground {
            return Outcome::Ignored;
        }
        match ev {
            PageEvent::Key(key) => {
                let Some(f) = cx.focus.current() else {
                    return Outcome::Ignored;
                };
                let Some(i) = self.buttons.iter().position(|b| b.id == f) else {
                    return Outcome::Ignored;
                };
                let (o, act) = self.buttons[i].on_key(key);
                if act {
                    self.open(Kind::ALL[i], cx);
                }
                o
            }
            PageEvent::Pointer(p) => {
                let hit = (self.buttons.iter_mut().enumerate())
                    .find_map(|(i, b)| b.route(p).map(|r| (i, r)));
                let Some((i, r)) = hit else {
                    return Outcome::Ignored;
                };
                if p.is(Phase::Click) {
                    cx.focus.focus(p.id);
                }
                if r.action().is_some() {
                    self.open(Kind::ALL[i], cx);
                    return Outcome::Changed;
                }
                r.outcome()
            }
            _ => Outcome::Ignored,
        }
    }

    fn hints(&self, focus: Option<WidgetId>) -> Vec<Hint> {
        if self.open.is_some() {
            return vec![("Esc", "Close")];
        }
        if let Some(h) = self.template.hints(focus) {
            return h;
        }
        vec![("Enter", "Open"), ("Tab", "Next")]
    }

    fn animating(&self) -> bool {
        matches!(self.kind(), Some(Kind::Tabs)) || self.fetching.is_some()
    }

    /// A failed fetch retries with r (PickerStatus::Error asks the owner
    /// to): the page's own key over its picker.
    fn on_layer_key(&mut self, key: &Key, cx: &mut Cx) -> Outcome {
        let failed = self.fetching.is_none() && self.opts.value("pickers.remote") == "fails";
        if !(key.is_char('r') && self.kind() == Some(Kind::Remote) && failed) {
            return Outcome::Ignored;
        }
        self.fetching = Some(FETCH_TICKS);
        self.refill(Kind::Remote, cx);
        self.last = "retry".into();
        Outcome::Changed
    }

    fn on_layer(&mut self, tag: &LayerTag, report: Report, cx: &mut Cx) -> Outcome {
        self.on_report(Kind::ALL[tag.n], report, cx);
        Outcome::Changed
    }

    fn option_value(&self, key: &str) -> Option<usize> {
        self.opts.get(key)
    }

    fn set_option(&mut self, key: &str, value: usize) {
        self.opts.set(key, value);
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
