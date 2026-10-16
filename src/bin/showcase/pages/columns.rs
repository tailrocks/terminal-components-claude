//! Columns (Tables): the library's column rules and column sizing.
//!
//! - **Width solver**: `ui::columns::Col` rules (content, fixed, flex,
//!   max, droppable) solved by `columns::solve` at a chosen row width; the
//!   row under the rules is a `DataTable` laid out at the solved widths, so
//!   a narrow row drops its least important columns first.
//! - **Resizable columns**: `ColumnSizing` on a `DataTable` (seams you drag,
//!   double-click to fit, the header menu's Size Column to Fit and Reset
//!   Column Widths), with its hit width and narrowest drag as options.
//! - **Header menu**: what `columns::header_menu` offers for the table's
//!   cursor column, with its `HeaderLabels`.

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Rect};

use crate::pages::row_menu::{MENU_HINTS, RowMenu};
use crate::pages::tables::route_table;
use crate::pages::template::{Section, Template};
use crate::pages::{Cx, Hint, Page, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::{Part, WidgetId};
use junie_tui::core::intent::Phase;
use junie_tui::core::pointer::PointerEvent;
use junie_tui::ui::columns::{self, Col, HeaderLabels};
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::ui::layout::StackChild;
use junie_tui::ui::specimen::{SpecimenState, StateGrid};
use junie_tui::widgets::menu::ContextMenu;
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::props::{self, Prop};
use junie_tui::widgets::table::{Cell, Column, DataTable, Tone};

const ID: WidgetId = WidgetId::of("columns");
const HEADER_MENU: WidgetId = WidgetId::of("columns.header-menu");

/// The `columns.width` values: the card's width, then fixed row widths.
const WIDTHS: [Option<u16>; 5] = [None, Some(72), Some(56), Some(40), Some(28)];
const SEAM_WIDTHS: [u16; 3] = [1, 2, 3];
const MIN_WIDTHS: [u16; 3] = [3, 6, 10];
/// Cells between two solved columns.
const GAP: u16 = 2;

/// Labels an app might use instead of the defaults (`columns.header_labels`).
const CUSTOM_LABELS: HeaderLabels = HeaderLabels {
    sort_asc: "Sort A to Z",
    sort_desc: "Sort Z to A",
    clear_sort: "Unsort",
    autofit: "Fit Width",
    reset_widths: "Default Widths",
};

/// One column of the solver demo: its name, rule, the rule as text and a
/// sample value.
struct Rule {
    name: &'static str,
    col: Col,
    text: &'static str,
    value: &'static str,
}

fn rules() -> [Rule; 5] {
    [
        Rule {
            name: "Account",
            col: Col::content(8, 18).flex(1, 30),
            text: "content 8–18 · flex 1, max 30",
            value: "payments-platform",
        },
        Rule {
            name: "Usage",
            col: Col::fixed(12),
            text: "fixed 12",
            value: "██████░░ 74%",
        },
        Rule {
            name: "Plan",
            col: Col::content(6, 10).droppable(2),
            text: "content 6–10 · droppable 2",
            value: "enterprise",
        },
        Rule {
            name: "Renews",
            col: Col::content(10, 10).droppable(1),
            text: "content 10 · droppable 1",
            value: "2026-11-02",
        },
        Rule {
            name: "Notes",
            col: Col::content(6, 24).max(40).droppable(0),
            text: "content 6–24 · max 40 · drop 0",
            value: "seats renew with the plan",
        },
    ]
}

/// The resizable task table.
fn sized_table(resizable: bool, seam_w: u16, min_width: u16, labels: HeaderLabels) -> DataTable {
    let mut t = DataTable::new(
        ID.sub("sized"),
        crate::pages::tables::task_columns(),
        crate::pages::tables::task_rows(),
    )
    .numeric(&[0, 5, 6])
    .resizable_columns(resizable)
    .seam_hit_w(seam_w);
    t.sizing.min_width = min_width;
    t.header_labels = labels;
    t
}

/// The States grid: the resizable table and a fixed one.
fn states_grid() -> StateGrid {
    use SpecimenState::*;
    let id = ID.sub("states");
    let sample = |k: usize, resizable: bool| {
        let cols = vec![
            Column::new("Task", Constraint::Min(14)),
            Column::new("Owner", Constraint::Length(7)),
            Column::new("Changes", Constraint::Length(8)).right(),
        ];
        let rows = crate::pages::tables::task_rows()
            .into_iter()
            .take(3)
            .map(|r| vec![r[1].clone(), r[2].clone(), r[5].clone()])
            .collect();
        DataTable::new(id.child(k), cols, rows)
            .numeric(&[2])
            .resizable_columns(resizable)
    };
    StateGrid::new(id)
        .rows(&[Rest, Hover, Focused, Selected, Overflow, Narrow])
        .column_width(40)
        .narrow_width(26)
        .column("Resizable", sample(0, true))
        .column("Fixed widths", sample(1, false))
}

pub struct ColumnsPage {
    template: Template,
    width: usize,
    resizable: bool,
    seam_w: usize,
    min_width: usize,
    labels: usize,
    /// The rules as a table, with the solved width of each column.
    rules: DataTable,
    /// The row the rules lay out.
    row: DataTable,
    pub sized: DataTable,
    /// The row width the rules were last solved at.
    solved_at: u16,
    header_menu: RowMenu<usize>,
    sections: crate::pages::layout::Sections,
}

impl ColumnsPage {
    pub fn new() -> Self {
        let mut p = Self {
            template: Template::new(ID, states_grid()),
            width: 0,
            resizable: true,
            seam_w: 1,
            min_width: 0,
            labels: 0,
            rules: DataTable::new(ID.sub("rules"), Vec::new(), Vec::new()),
            row: DataTable::new(ID.sub("row"), Vec::new(), Vec::new()),
            sized: DataTable::new(ID.sub("sized"), Vec::new(), Vec::new()),
            solved_at: 0,
            header_menu: RowMenu::default(),
            sections: crate::pages::layout::Sections::new(ID.sub("page")),
        };
        p.rebuild_sized();
        p.solve(72);
        p
    }

    fn header_labels(&self) -> HeaderLabels {
        [HeaderLabels::DEFAULT, CUSTOM_LABELS][self.labels]
    }

    fn rebuild_sized(&mut self) {
        self.sized = sized_table(
            self.resizable,
            SEAM_WIDTHS[self.seam_w],
            MIN_WIDTHS[self.min_width],
            self.header_labels(),
        );
    }

    /// Solve the rules for a row `avail` cells wide: the rules table shows
    /// each solved width, the row table the surviving columns at them.
    fn solve(&mut self, avail: u16) {
        self.solved_at = avail;
        let rules = rules();
        let cols: Vec<Col> = rules.iter().map(|r| r.col).collect();
        let widths = columns::solve(&cols, avail, GAP);
        let n = |v: u16| {
            if v == u16::MAX {
                "∞".to_owned()
            } else {
                v.to_string()
            }
        };
        let rows = rules
            .iter()
            .zip(&widths)
            .map(|(r, &w)| {
                let solved = if w == 0 {
                    Cell::new("dropped").tone(Tone::Warning)
                } else {
                    Cell::new(w.to_string())
                };
                vec![
                    Cell::new(r.name),
                    Cell::new(r.text).tone(Tone::Muted),
                    Cell::new(r.col.min.to_string()),
                    Cell::new(r.col.ideal.to_string()),
                    Cell::new(n(r.col.max)),
                    Cell::new(r.col.flex.to_string()),
                    Cell::new(r.col.drop.map_or("never".into(), |d| d.to_string())),
                    solved,
                ]
            })
            .collect();
        self.rules = DataTable::new(
            ID.sub("rules"),
            vec![
                Column::new("Column", Constraint::Length(8)),
                Column::new("Rule", Constraint::Min(30)),
                Column::new("min", Constraint::Length(4)).right(),
                Column::new("ideal", Constraint::Length(5)).right(),
                Column::new("max", Constraint::Length(4)).right(),
                Column::new("flex", Constraint::Length(4)).right(),
                Column::new("drop", Constraint::Length(5)).right(),
                Column::new(&format!("at {avail}"), Constraint::Length(7)).right(),
            ],
            rows,
        )
        .numeric(&[2, 3, 4, 5, 6, 7])
        .selection_mode(junie_tui::core::selection::SelectionMode::None);
        let (cols, cells): (Vec<Column>, Vec<Cell>) = rules
            .iter()
            .zip(&widths)
            .filter(|(_, w)| **w > 0)
            .map(|(r, &w)| {
                (
                    Column::new(r.name, Constraint::Length(w)),
                    Cell::new(r.value),
                )
            })
            .unzip();
        self.row = DataTable::new(ID.sub("row"), cols, vec![cells])
            .selection_mode(junie_tui::core::selection::SelectionMode::None);
    }

    fn render_playground(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let focus = if ctx.interaction.has_focus(self.sized.id) {
            Some(1)
        } else if ctx.interaction.has_focus(self.rules.id) || ctx.interaction.has_focus(self.row.id)
        {
            Some(0)
        } else {
            None
        };
        let l = self.sections.layout(
            area,
            &[
                StackChild::fixed(12),
                StackChild::flex(9, area.height.saturating_sub(22).max(9)),
                StackChild::fixed(9),
            ],
            focus,
            buf,
            ctx,
        );
        // the width solver
        if let Some(r) = l.slot(0) {
            let panel = Panel::card(Some("Width solver"));
            let bg = panel.bg(t);
            let inner = panel.render(r, buf, t);
            // the table's gutter and edge take 5 cells around the row
            let room = inner.width.saturating_sub(5);
            let avail = WIDTHS[self.width].unwrap_or(room).min(room);
            if avail != self.solved_at {
                self.solve(avail);
            }
            let rules = Rect::new(inner.x, inner.y, inner.width, 6.min(inner.height));
            self.rules.render(rules, buf, ctx, bg);
            if inner.height > 8 {
                let y = rules.bottom() + 1;
                let caption = format!("Row at {avail} cells: the lowest drop priority goes first");
                crate::pages::layout::caption(inner.x, y, buf, t, &caption, bg);
                let row = Rect::new(inner.x, y + 1, (avail + 5).min(inner.width), 2);
                self.row.render(row, buf, ctx, bg);
            }
            panel.draw_meta(r, buf, t, "Col rules · columns::solve");
        }
        // the resizable table
        let panel =
            Panel::card(Some("Resizable columns")).focused(ctx.interaction.focused(self.sized.id));
        let bg = panel.bg(t);
        match l.slot(1) {
            Some(r) => {
                let inner = panel.render(r, buf, t);
                self.sized.render(inner, buf, ctx, bg);
                let meta = if !self.sized.sizing.resizable {
                    "fixed widths".to_owned()
                } else if self.sized.sizing.is_default() {
                    "drag a seam · double-click fits".to_owned()
                } else {
                    "user widths · right-click a header to reset".to_owned()
                };
                panel.draw_meta(r, buf, t, &meta);
            }
            None => ctx.focus_stop(self.sized.id),
        }
        // the header menu of the cursor column
        if let Some(r) = l.slot(2) {
            let c = self.sized.cursor_col;
            let title = format!("Header menu · {}", self.sized.columns[c].title);
            let panel = Panel::card(Some(&title));
            let bg = panel.bg(t);
            let inner = panel.render(r, buf, t);
            let items: Vec<Prop> = self
                .sized
                .header_menu(c)
                .into_iter()
                .map(|(_, m)| match &m.reason {
                    Some(why) => Prop::new(&m.label, why.clone()).tone(Tone::Muted),
                    None => Prop::new(&m.label, "available"),
                })
                .collect();
            props::render(inner, buf, t, &items, bg);
            panel.draw_meta(r, buf, t, "right-click a header");
        }
        self.header_menu.render(area, buf, ctx);
    }
}

impl ColumnsPage {
    /// A secondary click on a header opens its menu; the sized table takes
    /// its headers, seams (a press focuses it), cells, bar and wheel; the
    /// two specimen tables their cells.
    fn pointer(&mut self, ev: &PointerEvent, cx: &mut Cx) -> Outcome {
        if ev.is(Phase::Secondary) {
            let Some(c) = self.sized.header_menu_at(ev) else {
                return Outcome::Ignored;
            };
            cx.focus.focus(self.sized.id);
            self.sized.cursor_col = c;
            let items = self
                .sized
                .header_menu(c)
                .into_iter()
                .map(|(_, m)| m)
                .collect();
            let title = self.sized.columns[c].title.clone();
            let menu = ContextMenu::new(HEADER_MENU, items)
                .at(ev.pos())
                .title(title);
            self.header_menu.open(menu, c);
            return Outcome::Changed;
        }
        if ev.is(Phase::Press) && ev.on(self.sized.id) && ev.index_of(Part::SEAM).is_some() {
            cx.focus.focus(self.sized.id);
        }
        if ev.is(Phase::Click)
            && ev.on(self.sized.id)
            && let Some(c) = ev.index_of(Part::HEADER)
        {
            self.sized.cursor_col = c;
        }
        for t in [&mut self.sized, &mut self.rules, &mut self.row] {
            if let Some((out, _)) = route_table(t, ev, cx) {
                return out;
            }
        }
        Outcome::Ignored
    }
}

impl Page for ColumnsPage {
    fn title(&self) -> &'static str {
        "Columns"
    }
    fn blurb(&self) -> &'static str {
        "Column rules, droppable columns, resizable seams and the header menu"
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let body = self.template.render(area, buf, ctx);
        match self.template.section() {
            Section::Playground => self.render_playground(body, buf, ctx),
            Section::Options => self.template.render_options(body, buf, ctx),
            Section::States => self.template.render_states(body, buf, ctx),
        }
    }

    fn layer_open(&self) -> bool {
        self.header_menu.hides_page_focus()
    }

    fn option_value(&self, key: &str) -> Option<usize> {
        Some(match key {
            "columns.width" => self.width,
            "columns.resizable" => usize::from(self.resizable),
            "columns.seam_w" => self.seam_w,
            "columns.min_width" => self.min_width,
            "columns.header_labels" => self.labels,
            _ => return None,
        })
    }

    fn set_option(&mut self, key: &str, value: usize) {
        match key {
            "columns.width" => self.width = value.min(WIDTHS.len() - 1),
            "columns.resizable" => self.resizable = value == 1,
            "columns.seam_w" => self.seam_w = value.min(SEAM_WIDTHS.len() - 1),
            "columns.min_width" => self.min_width = value.min(MIN_WIDTHS.len() - 1),
            "columns.header_labels" => self.labels = value.min(1),
            _ => return,
        }
        if key != "columns.width" {
            self.rebuild_sized();
        }
    }

    fn handle(&mut self, ev: &PageEvent, cx: &mut Cx) -> Outcome {
        if let Some(o) = self.template.handle(ev, cx) {
            return o;
        }
        if let Some((out, chosen)) = self.header_menu.handle(ev) {
            return match chosen.and_then(|(c, i)| Some((c, self.sized.header_menu(c).get(i)?.0))) {
                Some((c, action)) => self.sized.on_header_action(c, action).or(out),
                None => out,
            };
        }
        if let Some(o) = self.sections.handle(ev) {
            return o;
        }
        match ev {
            PageEvent::Key(key) if cx.focus.is(self.sized.id) => self.sized.on_key(key).0,
            PageEvent::Key(key) if cx.focus.is(self.rules.id) => self.rules.on_key(key).0,
            PageEvent::Key(key) if cx.focus.is(self.row.id) => self.row.on_key(key).0,
            PageEvent::Pointer(p) => self.pointer(p, cx),
            _ => Outcome::Ignored,
        }
    }

    fn hints(&self, focus: Option<WidgetId>) -> Vec<Hint> {
        if let Some(h) = self.template.hints(focus) {
            return h;
        }
        if self.header_menu.is_open() {
            return MENU_HINTS.to_vec();
        }
        vec![("↑↓", "Move"), ("←→", "Column"), ("s", "Sort")]
    }

    fn template(&self) -> Option<&Template> {
        Some(&self.template)
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }

    fn template_mut(&mut self) -> Option<&mut Template> {
        Some(&mut self.template)
    }
}

pub fn page() -> Box<dyn Page> {
    Box::new(ColumnsPage::new())
}
