//! Breadcrumbs & rails (plan §7, SC5): the `Breadcrumb` trail and the
//! collapsible sidebar's `SidebarRail`.
//!
//! Playground: a folder explorer in a framed sidebar whose title carries
//! the rail's `‹` collapse button; collapsed, it leaves the rail (`›` over
//! the sidebar's name written down the column) that a click opens again.
//! Ctrl+B hides and shows it from the keyboard. The location is a live,
//! clickable `Breadcrumb`: a click on an earlier level goes back to it.
//! Beside it, the same trail as plain text and a deep trail at a chosen
//! width (the outer levels collapse into `…`, a long current page is cut
//! in the middle). Options: `clickable`, the trail width, the rail
//! (`collapsed_rail`) and the sidebar collapsed at start. States: both
//! trails and the rail in every state they have.

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;

use crate::app::PageId;
use crate::options::page_options;
use crate::pages::template::{Section, Template};
use crate::pages::{Cx, Hint, Page, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::{Part, WidgetId};
use junie_tui::core::intent::Phase;
use junie_tui::core::pointer::MouseTarget;
use junie_tui::theme::Tone;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::ui::specimen::{SpecimenState, StateGrid};
use junie_tui::ui::text;
use junie_tui::widgets::breadcrumb::Breadcrumb;
use junie_tui::widgets::list::{ListBox, ListItem, SelectMode};
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::props::{self, Prop};
use junie_tui::widgets::rail::{RailAction, SidebarRail};

const ID: WidgetId = WidgetId::of("breadcrumbsrails");
/// The explorer's folder list.
pub const LIST: WidgetId = WidgetId::of("breadcrumbsrails.list");
/// The live trail of the explorer's location.
pub const CRUMBS: WidgetId = WidgetId::of("breadcrumbsrails.crumbs");
/// The explorer's rail and collapse button.
pub const RAIL: WidgetId = WidgetId::of("breadcrumbsrails.rail");

/// The deep trail cut to the chosen width.
const DEEP: [&str; 6] = [
    "Disk",
    "Users",
    "alex",
    "Projects",
    "payments-platform",
    "settlement-backoff-retry-policy-v2.md",
];

/// One entry of a folder: its name, its metadata, whether it opens.
type Entry = (&'static str, &'static str, bool);

/// The folders the explorer walks (demo data).
fn entries(path: &[String]) -> Vec<Entry> {
    match path.join("/").as_str() {
        "Disk" => vec![
            ("Applications", "folder", true),
            ("Library", "folder", true),
            ("System", "locked", false),
            ("Users", "folder", true),
        ],
        "Disk/Users" => vec![("alex", "folder", true), ("Shared", "folder", true)],
        "Disk/Users/alex" => vec![
            ("Desktop", "folder", true),
            ("Documents", "folder", true),
            ("Downloads", "folder", true),
            ("Projects", "folder", true),
        ],
        "Disk/Users/alex/Projects" => vec![
            ("design-system", "folder", true),
            ("payments-platform", "folder", true),
            ("notes.md", "4 KB", false),
        ],
        "Disk/Users/alex/Projects/payments-platform" => vec![
            ("src", "folder", true),
            ("Cargo.toml", "1 KB", false),
            ("settlement-backoff-retry-policy-v2.md", "12 KB", false),
        ],
        "Disk/Applications" => vec![("Terminal", "app", false), ("Xcode", "app", false)],
        "Disk/Library" => vec![("Caches", "folder", true), ("Fonts", "folder", true)],
        _ => vec![],
    }
}

pub struct BreadcrumbsRailsPage {
    template: Template,
    /// The explorer's location, outermost first.
    path: Vec<String>,
    list: ListBox,
    crumbs: Breadcrumb,
    rail: SidebarRail,
    /// The sidebar is collapsed into its rail.
    collapsed: bool,
    /// Value indices of the page options, in `page_options` order.
    opts: Vec<usize>,
    /// What the trail and the rail reported last.
    last_crumb: String,
    last_rail: String,
}

impl BreadcrumbsRailsPage {
    pub fn new() -> Self {
        let path = vec!["Disk".to_owned(), "Users".to_owned(), "alex".to_owned()];
        let mut page = Self {
            template: Template::new(ID, states_grid()),
            list: ListBox::new(LIST, vec![], SelectMode::Single),
            crumbs: Breadcrumb::new(CRUMBS, path.clone()),
            path,
            rail: SidebarRail::new(RAIL, "Explorer"),
            collapsed: false,
            opts: page_options(PageId::BreadcrumbsRails)
                .iter()
                .map(|o| o.default)
                .collect(),
            last_crumb: "—".into(),
            last_rail: "—".into(),
        };
        page.rebuild();
        page.load_folder(0);
        page
    }

    /// The value of option `key` as the table spells it.
    fn opt(&self, key: &str) -> &'static str {
        let table = page_options(PageId::BreadcrumbsRails);
        let i = table
            .iter()
            .position(|o| o.key == key)
            .unwrap_or_else(|| panic!("no option {key}"));
        table[i].values[self.opts[i]]
    }

    /// The trail and the rail, built from the options through their
    /// builders.
    fn rebuild(&mut self) {
        self.crumbs = Breadcrumb::new(CRUMBS, self.path.clone())
            .clickable(self.opt("breadcrumbsrails.clickable") == "on");
        self.rail = SidebarRail::new(RAIL, "Explorer")
            .collapsed_rail(self.opt("breadcrumbsrails.collapsed_rail") == "on");
    }

    /// Show the folder at `path` with the cursor on row `cursor`.
    fn load_folder(&mut self, cursor: usize) {
        let items = entries(&self.path)
            .into_iter()
            .map(|(name, meta, opens)| {
                let item = ListItem::new(name).meta(meta);
                if meta == "locked" {
                    item.meta_tone(Tone::Warning).disabled(true)
                } else if opens {
                    item
                } else {
                    item.meta_tone(Tone::Secondary)
                }
            })
            .collect::<Vec<_>>();
        let n = items.len();
        self.list = ListBox::new(LIST, items, SelectMode::Single).empty_text("Empty folder");
        self.list.cursor = cursor.min(n.saturating_sub(1));
        self.crumbs.segments = self.path.clone();
    }

    /// The trail went back to `level` (0 is the outermost).
    fn go_to_level(&mut self, level: usize, cx: &mut Cx) {
        let left = self.path.get(level + 1).cloned();
        self.path.truncate(level + 1);
        let cursor = left
            .and_then(|l| entries(&self.path).iter().position(|e| e.0 == l))
            .unwrap_or(0);
        self.load_folder(cursor);
        cx.status(format!("Went back to {}", self.path.join(" › ")));
    }

    /// Open the chosen row: a folder descends, anything else reports.
    fn open_row(&mut self, i: usize, cx: &mut Cx) {
        let Some((name, _, opens)) = entries(&self.path).get(i).copied() else {
            return;
        };
        if opens {
            self.path.push(name.to_owned());
            self.load_folder(0);
        } else {
            cx.status(format!("Opened {name}"));
        }
    }

    /// Collapse or expand the sidebar (its rail, its button or Ctrl+B).
    fn set_collapsed(&mut self, on: bool, cx: &mut Cx) {
        self.collapsed = on;
        self.last_rail = if on { "Collapse" } else { "Expand" }.into();
        if on {
            if cx.focus.is(LIST) {
                cx.focus.focus(self.template.tabs.id);
            }
        } else {
            cx.focus.focus(LIST);
        }
    }

    fn render_playground(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        // the explorer, or its rail
        let side_w = 34.min(area.width / 2);
        let height = area.height.min(18);
        let right_x = if self.collapsed {
            let col = Rect::new(area.x, area.y, SidebarRail::WIDTH, height);
            self.rail.render_rail(col, buf, ctx, t.canvas);
            area.x + self.rail.width() + 1
        } else {
            let side = Rect::new(area.x, area.y, side_w, height);
            // the meta keeps room for the collapse button after it
            let meta = format!("{} items    ", self.list.items.len());
            let panel = Panel::framed(Some("Explorer"))
                .focused(ctx.interaction.focused(LIST))
                .meta(&meta);
            let bg = panel.bg(t);
            let inner = panel.render(side, buf, t);
            self.rail.render_collapse(side, buf, ctx, t.canvas);
            self.list.render(inner, buf, ctx, bg);
            side.right() + 2
        };
        let right = Rect::new(
            right_x,
            area.y,
            area.right().saturating_sub(right_x),
            area.height,
        );
        if right.width < 20 {
            return;
        }

        // the live trail and what it and the rail reported
        let meta = if self.crumbs.clickable {
            "clickable"
        } else {
            "clickable(false)"
        };
        let panel = Panel::card(Some("Location")).meta(meta);
        let bg = panel.bg(t);
        let card = Rect::new(right.x, right.y, right.width, 9.min(right.height));
        let inner = panel.render(card, buf, t);
        self.crumbs
            .render(Rect::new(inner.x, inner.y, inner.width, 1), buf, ctx, bg);
        let facts = vec![
            Prop::new("Depth", self.path.len().to_string()),
            Prop::new("Last crumb", self.last_crumb.clone()),
            Prop::new("Last RailAction", self.last_rail.clone()),
            Prop::new("Keys", "Enter opens · Backspace goes up · Ctrl+B explorer"),
        ];
        props::render(
            Rect::new(
                inner.x,
                inner.y + 2,
                inner.width,
                inner.height.saturating_sub(2),
            ),
            buf,
            t,
            &facts,
            bg,
        );

        // the same trail as plain text, and a deep trail at a set width
        let y = card.bottom() + 1;
        if y + 8 > right.bottom() {
            return;
        }
        let panel = Panel::card(Some("Trails"));
        let bg = panel.bg(t);
        let card = Rect::new(right.x, y, right.width, 8);
        let inner = panel.render(card, buf, t);
        let w = match self.opt("breadcrumbsrails.width") {
            "full" => inner.width,
            n => n.parse::<u16>().unwrap_or(inner.width).min(inner.width),
        };
        let rows: [(&str, Breadcrumb, u16); 2] = [
            (
                "clickable(false)",
                Breadcrumb::new(ID.sub("plain"), self.path.clone()).clickable(false),
                inner.width,
            ),
            (
                "deep, cut to fit",
                Breadcrumb::new(
                    ID.sub("deep"),
                    DEEP.iter().map(|s| (*s).to_owned()).collect(),
                ),
                w,
            ),
        ];
        let mut yy = inner.y;
        for (label, mut trail, w) in rows {
            if yy + 1 >= inner.bottom() {
                break;
            }
            let cap = format!("{label} · width {w}");
            buf.set_string(
                inner.x,
                yy,
                text::truncate(&cap, inner.width as usize),
                t.faint().bg(bg),
            );
            trail.render(Rect::new(inner.x, yy + 1, w, 1), buf, ctx, bg);
            yy += 3;
        }

        // the rail's actions
        let y = card.bottom() + 1;
        if y + 4 > right.bottom() {
            return;
        }
        let panel = Panel::card(Some("SidebarRail"));
        let bg = panel.bg(t);
        let h = 6.min(right.bottom() - y);
        let inner = panel.render(Rect::new(right.x, y, right.width, h), buf, t);
        let facts = vec![
            Prop::new("RailAction::Collapse", "‹ in the frame, or Ctrl+B"),
            Prop::new("RailAction::Expand", "a click on the › rail, or Ctrl+B"),
            Prop::new(
                "collapsed_rail",
                if self.rail.collapsed_rail {
                    "on: a collapsed sidebar leaves its rail"
                } else {
                    "off: only the keyboard brings it back"
                },
            ),
        ];
        props::render(inner, buf, t, &facts, bg);
    }
}

/// The States grid: both trails and the rail, drawn by themselves.
fn states_grid() -> StateGrid {
    use SpecimenState::*;
    let id = ID.sub("states");
    let trail = || {
        ["Disk", "Users", "alex", "Projects"]
            .iter()
            .map(|s| (*s).to_owned())
            .collect::<Vec<_>>()
    };
    StateGrid::new(id)
        .rows(&[Rest, Hover, Pressed, Overflow, Narrow])
        .column_width(34)
        .column("Clickable", Breadcrumb::new(id.child(0), trail()))
        .column(
            "Plain text",
            Breadcrumb::new(id.child(1), trail()).clickable(false),
        )
        .column("Rail", SidebarRail::new(id.child(2), "Files"))
}

impl Page for BreadcrumbsRailsPage {
    fn title(&self) -> &'static str {
        "Breadcrumbs & rails"
    }

    fn blurb(&self) -> &'static str {
        "Breadcrumb trails (static, clickable, collapsing) and the collapsible SidebarRail"
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
        // Ctrl+B is the sidebar's shortcut wherever focus is on the page
        if let PageEvent::Key(key) = ev
            && key.ctrl_char('b')
            && self.template.section() == Section::Playground
        {
            self.set_collapsed(!self.collapsed, cx);
            return Outcome::Changed;
        }
        if let Some(o) = self.template.handle(ev, cx) {
            return o;
        }
        match ev {
            PageEvent::Key(key) if cx.focus.is(LIST) => {
                if key.code == KeyCode::Backspace && key.plain() {
                    if self.path.len() > 1 {
                        let level = self.path.len() - 2;
                        self.go_to_level(level, cx);
                        self.last_crumb = format!("Backspace → level {level}");
                    }
                    return Outcome::Changed;
                }
                let o = self.list.on_key(key);
                if let Some(i) = self.list.chosen.take() {
                    self.open_row(i, cx);
                    return Outcome::Changed;
                }
                o
            }
            PageEvent::Pointer(ev) => {
                if let Some(r) = self.crumbs.route(ev) {
                    let (o, level) = r.into_parts();
                    if let Some(level) = level {
                        self.last_crumb = format!("{} (level {level})", self.path[level]);
                        self.go_to_level(level, cx);
                        return Outcome::Changed;
                    }
                    return o;
                }
                if let Some(r) = self.rail.route(ev) {
                    let (o, a) = r.into_parts();
                    if let Some(a) = a {
                        self.set_collapsed(a == RailAction::Collapse, cx);
                        return Outcome::Changed;
                    }
                    return o;
                }
                if ev.on(LIST) && ev.is(Phase::Click) && ev.index_of(Part::ROW).is_some() {
                    cx.focus.focus(LIST);
                }
                if let Some(r) = self.list.route(ev) {
                    let o = r.outcome();
                    if let Some(i) = self.list.chosen.take() {
                        self.open_row(i, cx);
                        return Outcome::Changed;
                    }
                    return o;
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
        if focus == Some(LIST) {
            return vec![
                ("↑↓", "Move"),
                ("Enter", "Open"),
                ("Backspace", "Up a level"),
                ("Ctrl+B", "Hide explorer"),
            ];
        }
        if self.collapsed {
            return vec![("Ctrl+B", "Show explorer")];
        }
        Vec::new()
    }

    fn option_value(&self, key: &str) -> Option<usize> {
        let i = page_options(PageId::BreadcrumbsRails)
            .iter()
            .position(|o| o.key == key)?;
        Some(self.opts[i])
    }

    fn set_option(&mut self, key: &str, value: usize) {
        let table = page_options(PageId::BreadcrumbsRails);
        if let Some(i) = table.iter().position(|o| o.key == key) {
            self.opts[i] = value.min(table[i].values.len() - 1);
            if key == "breadcrumbsrails.collapsed" {
                self.collapsed = self.opts[i] == 1;
            }
            self.rebuild();
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
    Box::new(BreadcrumbsRailsPage::new())
}
