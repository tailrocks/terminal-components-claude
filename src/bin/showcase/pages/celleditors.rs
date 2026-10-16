//! "Cell editors": a table whose editable columns only accept values from
//! a fixed set, one column per constrained editor (docs/showcase/tables.md).

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Rect};

use crate::pages::{Hint, Page, PageCtx, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::WidgetId;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::widgets::option_editor::{ColumnEditor, OptionItem, Options};
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::scrollbar;
use junie_tui::widgets::table::{Cell, Column, DataTable, TableEvent, Tone};

const ID: WidgetId = WidgetId::of("celleditors");

/// Column indices (the app tests address cells by them).
#[allow(dead_code)]
pub const ENV: usize = 2;
#[allow(dead_code)]
pub const REGION: usize = 3;
#[allow(dead_code)]
pub const ROLLOUT: usize = 4;
#[allow(dead_code)]
pub const OWNER: usize = 5;
#[allow(dead_code)]
pub const NOTIFY: usize = 6;
#[allow(dead_code)]
pub const LABELS: usize = 7;
#[allow(dead_code)]
pub const PRIORITY: usize = 8;

fn environments() -> Options {
    Options::new([
        OptionItem::new("dev").label("Development"),
        OptionItem::new("stage").label("Staging"),
        OptionItem::new("prod")
            .label("Production")
            .tone(Tone::Warning),
        OptionItem::new("sandbox").label("Sandbox"),
        OptionItem::new("preview").label("Preview"),
        OptionItem::new("perf").label("Performance"),
        OptionItem::new("dr")
            .label("Disaster recovery")
            .disabled("Frozen until Friday"),
    ])
}

fn regions() -> Options {
    let mut items: Vec<OptionItem> = [
        "us-east-1",
        "us-east-2",
        "us-west-1",
        "us-west-2",
        "ca-central-1",
        "sa-east-1",
        "eu-west-1",
        "eu-west-2",
        "eu-west-3",
        "eu-central-1",
        "eu-north-1",
        "ap-south-1",
        "ap-southeast-1",
        "ap-southeast-2",
        "ap-northeast-1",
        "ap-northeast-2",
        "me-south-1",
    ]
    .iter()
    .map(|r| OptionItem::new(r))
    .collect();
    items.push(OptionItem::new("af-south-1").disabled("Not enabled"));
    Options::new(items)
}

fn rollouts() -> Options {
    Options::new([
        OptionItem::new("off").label("Off").tone(Tone::Muted),
        OptionItem::new("canary").label("Canary"),
        OptionItem::new("half").label("50 %"),
        OptionItem::new("full").label("Full"),
    ])
}

fn owners() -> Options {
    Options::values(&[
        "Ana Lima",
        "Bea Costa",
        "Chen Wei",
        "Dara Okoye",
        "Eli Park",
        "Farah Aziz",
        "Gus Moreno",
        "Hana Sato",
        "Ivo Petrov",
        "Jonas Berg",
        "Kai Nakamura",
        "Lena Vogel",
        "Mira Okafor",
        "Omar Haddad",
        "Priya Nair",
        "Sofia Rossi",
        "Theo Grant",
    ])
}

fn labels() -> Options {
    Options::new([
        OptionItem::new("api"),
        OptionItem::new("web"),
        OptionItem::new("worker"),
        OptionItem::new("db"),
        OptionItem::new("cache"),
        OptionItem::new("edge"),
        OptionItem::new("legacy").disabled("Retired label"),
    ])
}

fn priorities() -> Options {
    Options::new([
        OptionItem::new("P1")
            .tone(Tone::Error)
            .description("Page on-call now"),
        OptionItem::new("P2")
            .tone(Tone::Warning)
            .description("Fix today"),
        OptionItem::new("P3").description("This sprint"),
        OptionItem::new("P4").disabled("Use P3 or close"),
    ])
}

/// Status is a read-only pill: toned text, never an editor affordance.
fn status(s: &str) -> Cell {
    let tone = match s {
        "Down" => Tone::Error,
        "Degraded" => Tone::Warning,
        _ => Tone::Secondary,
    };
    Cell::new(s).tone(tone)
}

pub fn table() -> DataTable {
    let columns = vec![
        Column::new("ID", Constraint::Length(5)),
        Column::new("Service", Constraint::Min(14)),
        Column::new("Env", Constraint::Length(13)).editor(ColumnEditor::Select(environments())),
        Column::new("Region", Constraint::Length(15)).editor(ColumnEditor::for_options(regions())),
        Column::new("Rollout", Constraint::Length(8)).editor(ColumnEditor::for_options(rollouts())),
        Column::new("Owner", Constraint::Length(13)).editor(ColumnEditor::for_options(owners())),
        Column::new("Notify", Constraint::Length(6))
            .editor(ColumnEditor::for_options(Options::values(&["no", "yes"]))),
        Column::new("Labels", Constraint::Length(14)).editor(ColumnEditor::Tags(labels())),
        Column::new("Priority", Constraint::Length(8)).editor(ColumnEditor::Radio(priorities())),
        Column::new("Status", Constraint::Length(9)),
    ];
    let data: [[&str; 10]; 8] = [
        [
            "D-101",
            "checkout-api",
            "prod",
            "us-east-1",
            "canary",
            "Mira Okafor",
            "yes",
            "api, web",
            "P1",
            "Healthy",
        ],
        [
            "D-102",
            "search",
            "stage",
            "eu-west-1",
            "full",
            "Jonas Berg",
            "no",
            "api",
            "P3",
            "Healthy",
        ],
        [
            "D-103",
            "billing-worker",
            "prod",
            "eu-central-1",
            "half",
            "Sofia Rossi",
            "yes",
            "worker, db",
            "P2",
            "Degraded",
        ],
        [
            "D-104",
            "edge-cache",
            "preview",
            "ap-southeast-1",
            "off",
            "Kai Nakamura",
            "no",
            "cache, edge",
            "P3",
            "Healthy",
        ],
        [
            "D-105",
            "auth",
            "dev",
            "us-west-2",
            "full",
            "Ana Lima",
            "yes",
            "api",
            "P2",
            "Healthy",
        ],
        // imported from an old inventory: values outside today's options
        [
            "D-106",
            "reports",
            "qa-legacy",
            "mars-1",
            "canary",
            "Priya Nair",
            "no",
            "batch",
            "P3",
            "Down",
        ],
        [
            "D-107",
            "notifications",
            "sandbox",
            "sa-east-1",
            "off",
            "Omar Haddad",
            "yes",
            "worker",
            "P4",
            "Healthy",
        ],
        [
            "D-108",
            "media",
            "perf",
            "ap-northeast-1",
            "half",
            "Hana Sato",
            "no",
            "web, cache",
            "P3",
            "Healthy",
        ],
    ];
    let rows = data
        .iter()
        .map(|r| {
            let mut cells: Vec<Cell> = r[..9].iter().map(|v| Cell::new(*v)).collect();
            cells[0] = cells[0].clone().tone(Tone::Muted);
            cells.push(status(r[9]));
            cells
        })
        .collect();
    let mut t = DataTable::new(ID.sub("table"), columns, rows).cell_nav(true);
    t.cursor_col = ENV;
    t
}

pub struct CellEditorsPage {
    pub table: DataTable,
    edits: u32,
}

impl CellEditorsPage {
    pub fn new() -> Self {
        Self {
            table: table(),
            edits: 0,
        }
    }

    fn on_event(&mut self, ev: Option<TableEvent>, cx: &mut PageCtx) {
        match ev {
            Some(TableEvent::Committed { col, .. }) => {
                self.edits += 1;
                let name = self.table.columns[col].title.clone();
                cx.status(format!("{name} saved"));
            }
            Some(TableEvent::Cancelled) => cx.status("Edit cancelled"),
            Some(TableEvent::LeaveForward) => cx.focus_next(),
            Some(TableEvent::LeaveBackward) => cx.focus_prev(),
            _ => {}
        }
    }
}

/// The legend: one line per editor, what it is for and the column using it.
const LEGEND: &[(&str, &str, &str)] = &[
    ("▾", "Select", "Env: 6–15 values in a dropdown"),
    (
        "/",
        "Combo",
        "Region, Owner: more than 15 values, type to filter",
    ),
    ("│", "Segmented", "Rollout: 2–5 short values on one strip"),
    ("[✓]", "Checkbox", "Notify: two values flip in place"),
    ("[✓]", "Tags", "Labels: several values from a set"),
    ("(●)", "Radio", "Priority: values that need a description"),
    (
        "!",
        "Outside the set",
        "imported values stay flagged until fixed",
    ),
    ("", "Read only", "ID, Service and the Status pill"),
];

impl Page for CellEditorsPage {
    fn title(&self) -> &'static str {
        "Cell editors"
    }
    fn blurb(&self) -> &'static str {
        "Columns that only take values from a fixed set"
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let err = self.table.edit_error().map(str::to_owned);
        let panel =
            Panel::card(Some("Deployment targets")).focused(ctx.interaction.focused(self.table.id));
        let bg = panel.bg(t);
        let legend_h = LEGEND.len() as u16 + 2;
        let card_h = (self.table.len() as u16 + 5)
            .min(area.height.saturating_sub(legend_h.min(area.height / 3)))
            .max(6.min(area.height));
        let card = Rect::new(area.x, area.y, area.width, card_h);
        let inner = panel.render(card, buf, t);
        self.table.render(inner, buf, ctx, bg);
        let flagged = self.table.out_of_set_cells().len();
        let meta = match (&err, flagged) {
            (Some(e), _) => e.clone(),
            (None, 0) => format!("{} edits", self.edits),
            (None, n) => format!(
                "{} edits · {n} value{} outside the set",
                self.edits,
                if n == 1 { "" } else { "s" }
            ),
        };
        panel.draw_meta(card, buf, t, &meta);
        for (y, (glyph, name, what)) in (area.y + card_h + 1..).zip(LEGEND) {
            if y >= area.bottom() {
                break;
            }
            let gs = match *name {
                "Outside the set" => t.error_fg(),
                "Read only" => t.muted(),
                _ => t.secondary(),
            };
            buf.set_string(area.x + 1, y, glyph, gs);
            buf.set_string(area.x + 6, y, name, t.primary());
            buf.set_string(
                area.x + 23,
                y,
                junie_tui::ui::text::truncate(what, area.width.saturating_sub(24) as usize),
                t.muted(),
            );
        }
    }

    fn handle(&mut self, ev: &PageEvent, cx: &mut PageCtx) -> Outcome {
        if let PageEvent::Blur(ch) = ev {
            if ch.from != self.table.id {
                return Outcome::Ignored;
            }
            let tev = self.table.on_blur();
            let out = if tev.is_some() {
                Outcome::Changed
            } else {
                Outcome::Ignored
            };
            self.on_event(tev, cx);
            return out;
        }
        match ev {
            PageEvent::Key(key) => {
                if !cx.focus.is(self.table.id) {
                    return Outcome::Ignored;
                }
                let (out, tev) = self.table.on_key(key);
                self.on_event(tev, cx);
                out
            }
            PageEvent::Paste(text) => self.table.on_paste(text),
            PageEvent::Click { id, pos } => {
                if let Some((out, tev)) = self.table.on_click_editor(*id) {
                    cx.focus.focus(self.table.id);
                    self.on_event(tev, cx);
                    return out;
                }
                if let Some(c) = self.table.locate_header(*id) {
                    cx.focus.focus(self.table.id);
                    return self.table.on_click_header(c);
                }
                if let Some((row, col)) = self.table.locate(*id) {
                    cx.focus.focus(self.table.id);
                    let col = col.unwrap_or(self.table.cursor_col);
                    let (out, tev) = self.table.on_click_cell(row, col, *pos);
                    self.on_event(tev, cx);
                    return out;
                }
                if scrollbar::is_bar_of(*id, self.table.id) {
                    return self.table.on_scrollbar(*pos);
                }
                Outcome::Ignored
            }
            PageEvent::Drag { pressed, pos } if scrollbar::is_bar_of(*pressed, self.table.id) => {
                self.table.on_bar_drag(*pressed, *pos)
            }
            PageEvent::Wheel { id, delta } if self.table.owns(*id) => self.table.on_wheel(*delta),
            PageEvent::WheelH { id, delta } if self.table.owns(*id) => {
                self.table.on_wheel_h(*delta)
            }
            _ => Outcome::Ignored,
        }
    }

    fn editing(&self) -> bool {
        self.table.is_editing()
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }

    fn hints(&self, _focus: Option<WidgetId>) -> Vec<Hint> {
        if self.table.choice.is_some() {
            vec![
                ("↑↓", "Choose"),
                ("Enter", "Apply"),
                ("Tab", "Apply · next"),
                ("Esc", "Cancel"),
            ]
        } else if self.table.is_editing() {
            vec![("Enter", "Apply"), ("Esc", "Cancel"), ("Tab", "Next cell")]
        } else {
            vec![
                ("↑↓←→", "Cell"),
                ("Enter", "Edit"),
                ("Alt+↓", "Open"),
                ("click twice", "Edit"),
            ]
        }
    }
}
