//! Buttons (plan §7 SC7): `Button` kinds and states, a toolbar of buttons
//! outside the Tab order, `ButtonRow` alignment, gap and overflow, a "Next"
//! row, and `IconButton` plates. Every knob is an option of the page
//! (`crate::options::BUTTONS`).

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::style::Style;

use crate::app::PageId;
use crate::options::{PageValues, page_options};
use crate::pages::template::{Section, Template};
use crate::pages::{Hint, Page, PageCtx, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::WidgetId;
use junie_tui::ui::ctx::{FocusVisible, RenderCtx};
use junie_tui::ui::layout::StackChild;
use junie_tui::ui::specimen::{SpecimenState, StateGrid};
use junie_tui::ui::tooltip::Tip;
use junie_tui::widgets::button::{
    Button, ButtonRow, RowAlign, RowLayout, RowOverflow, row_layout, row_layout_right,
};
use junie_tui::widgets::icon_button::IconButton;
use junie_tui::widgets::panel::Panel;

const ID: WidgetId = WidgetId::of("buttons");
const LONG_JOB_TICKS: u8 = 28;
const RUN_TICKS: u8 = 24;

/// The toolbar: Run, Format, Explain, Cancel.
const RUN: usize = 0;
const CANCEL: usize = 3;

/// The row's owner: its `…` button is `ButtonRow::more_id(ROW)`.
const ROW: WidgetId = WidgetId::of("buttons.row");

pub struct ButtonsPage {
    template: Template,
    opts: PageValues,
    buttons: Vec<Button>,
    tools: Vec<Button>,
    row: Vec<Button>,
    next: Vec<Button>,
    icons: Vec<IconButton>,
    /// Where the row put its buttons in the last frame.
    layout: RowLayout,
    sections: crate::pages::layout::Sections,
    clicks: u32,
    last: Option<String>,
    busy_ticks: Option<u8>,
    run_ticks: Option<u8>,
}

impl ButtonsPage {
    pub fn new() -> Self {
        let opts = PageValues::new(page_options(PageId::Buttons));
        let mut page = Self {
            template: Template::new(ID, states_grid(&opts)).with_options(
                ID,
                "Buttons",
                page_options(PageId::Buttons),
            ),
            opts,
            buttons: Vec::new(),
            tools: Vec::new(),
            row: Vec::new(),
            next: Vec::new(),
            icons: Vec::new(),
            layout: RowLayout::default(),
            sections: crate::pages::layout::Sections::new(ID.sub("page")),
            clicks: 0,
            last: None,
            busy_ticks: None,
            run_ticks: None,
        };
        page.rebuild();
        page
    }

    /// Build every widget from the option values, through the builders an
    /// application calls.
    fn rebuild(&mut self) {
        let fv = match self.opts.value("buttons.focus_visible") {
            "keyboard-only" => FocusVisible::KeyboardOnly,
            _ => FocusVisible::Always,
        };
        let busy = self.buttons.get(8).is_some_and(|b| b.busy);
        self.buttons = vec![
            // rest the pointer on a button for its tooltip (Esc hides it)
            Button::primary(ID.child(0), "Run task").tooltip("Start the task in the background"),
            Button::secondary(ID.child(1), "Preview").tooltip("Show what the task would change"),
            Button::subtle(ID.child(2), "Cancel").tooltip("Leave without running anything"),
            Button::danger(ID.child(3), "Delete branch")
                .tooltip("Delete the branch, locally and on the remote"),
            Button::toggle(ID.child(4), "Auto-approve", false),
            Button::toggle(ID.child(5), "Verbose", true),
            Button::primary(ID.child(6), "Disabled primary").disabled(true),
            Button::secondary(ID.child(7), "Disabled").disabled(true),
            Button::secondary(ID.child(8), "Start long job"),
        ]
        .into_iter()
        .map(|b| b.focus_visible(fv))
        .collect();
        self.buttons[8].busy = busy;

        // toolbar buttons: clicks only, unless the option puts them in the
        // Tab order; the tooltip names the shortcut that does the same
        let tab = self.opts.on("buttons.toolbar_focus");
        let running = self.run_ticks.is_some();
        let tid = ID.sub("tools");
        let mut tools = vec![
            Button::secondary(tid.child(0), "▶ Run")
                .tooltip("Run the query")
                .shortcut("F5"),
            Button::subtle(tid.child(1), "Format").tooltip("Format the SQL"),
            Button::subtle(tid.child(2), "Explain").tooltip("Show the query plan"),
            Button::subtle(tid.child(3), "Cancel")
                .tooltip("Stop the running query")
                .shortcut("Shift+F5"),
        ];
        for b in &mut tools {
            *b = b.clone().focusable(tab).focus_visible(fv);
        }
        tools[RUN].busy = running;
        tools[2].disabled = running;
        tools[CANCEL].disabled = !running;
        self.tools = tools;

        let rid = ID.sub("row");
        self.row = [
            Button::secondary(rid.child(0), "Refresh"),
            Button::secondary(rid.child(1), "Validate"),
            Button::secondary(rid.child(2), "Duplicate"),
            Button::secondary(rid.child(3), "Edit…"),
            Button::danger(rid.child(4), "Remove…"),
        ]
        .into_iter()
        .map(|b| b.focus_visible(fv))
        .collect();

        let nid = ID.sub("next");
        self.next = [
            Button::subtle(nid.child(0), "Open logs"),
            Button::subtle(nid.child(1), "Retry failed").disabled(true),
            Button::subtle(nid.child(2), "Copy run ID"),
            Button::subtle(nid.child(3), "Share").disabled(true),
        ]
        .into_iter()
        .map(|b| b.focus_visible(fv))
        .collect();

        let pad = self.opts.number("buttons.icon_pad").unwrap_or(1);
        let danger = self.opts.on("buttons.icon_danger");
        let iid = ID.sub("icons");
        self.icons = vec![
            IconButton::close(iid.child(0))
                .pad(pad)
                .danger_hover(danger)
                .tip(Tip::new("Close the panel").shortcut("Esc")),
            IconButton::new(iid.child(1), "+")
                .plate(5)
                .tip(Tip::new("New tab")),
            IconButton::close(iid.child(2))
                .pad(pad)
                .danger_hover(true)
                .tip(Tip::new("Remove the filter")),
        ];
        self.template.grid = states_grid(&self.opts);
    }

    fn activated(&mut self, i: usize, cx: &mut PageCtx) {
        self.clicks += 1;
        let b = &self.buttons[i];
        let msg = match b.on {
            Some(true) => format!("{} on", b.label),
            Some(false) => format!("{} off", b.label),
            None => format!("{} ✓", b.label),
        };
        if i == 8 {
            self.buttons[8].busy = true;
            self.busy_ticks = Some(LONG_JOB_TICKS);
            cx.status("Working…".to_owned());
        } else {
            cx.status(msg.clone());
        }
        self.last = Some(msg);
    }

    /// A toolbar command, from its button or its shortcut.
    fn tool(&mut self, i: usize, cx: &mut PageCtx) {
        if !self.tools[i].can_activate() {
            return;
        }
        match i {
            RUN => {
                self.run_ticks = Some(RUN_TICKS);
                cx.status("Running the query…");
            }
            CANCEL => {
                self.run_ticks = None;
                cx.status("Query cancelled");
            }
            _ => cx.status(format!("{} ✓", self.tools[i].label)),
        }
        self.sync_run();
    }

    /// Run is busy and Cancel enabled while the query runs.
    fn sync_run(&mut self) {
        let running = self.run_ticks.is_some();
        self.tools[RUN].busy = running;
        self.tools[2].disabled = running;
        self.tools[CANCEL].disabled = !running;
    }

    fn row_width(&self, room: u16) -> u16 {
        self.opts
            .number("buttons.row_width")
            .map_or(room, |w| w.min(room))
    }

    fn button_row(&self) -> ButtonRow {
        let mut row = ButtonRow::new()
            .align(match self.opts.value("buttons.row_align") {
                "end" => RowAlign::End,
                _ => RowAlign::Start,
            })
            .overflow(match self.opts.value("buttons.row_overflow") {
                "wrap" => RowOverflow::Wrap,
                _ => RowOverflow::More,
            });
        if let Some(gap) = self.opts.number("buttons.row_gap") {
            row = row.gap(gap);
        }
        row
    }

    fn render_playground(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let room = area.width.saturating_sub(4);
        let row_lines = self.button_row().rows(self.row_width(room), &self.row, t);
        let has = |bs: &[Button]| bs.iter().any(|b| ctx.interaction.has_focus(b.id));
        let focus = if has(&self.buttons) {
            Some(0)
        } else if has(&self.tools) {
            Some(1)
        } else if has(&self.row)
            || has(&self.next)
            || ctx.interaction.has_focus(ButtonRow::more_id(ROW))
        {
            Some(2)
        } else {
            None
        };
        let l = self.sections.layout(
            area,
            &[
                StackChild::fixed(15),
                StackChild::fixed(6),
                StackChild::fixed(row_lines + 7),
                StackChild::fixed(6),
            ],
            focus,
            buf,
            ctx,
        );
        match l.slot(0) {
            Some(r) => self.render_kinds(r, buf, ctx),
            None => self.buttons.iter().for_each(|b| ctx.focus_stop(b.id)),
        }
        match l.slot(1) {
            Some(r) => self.render_toolbar(r, buf, ctx),
            None => self
                .tools
                .iter()
                .filter(|b| b.focusable)
                .for_each(|b| ctx.focus_stop(b.id)),
        }
        match l.slot(2) {
            Some(r) => self.render_rows(r, buf, ctx),
            None => self
                .row
                .iter()
                .chain(&self.next)
                .for_each(|b| ctx.focus_stop(b.id)),
        }
        if let Some(r) = l.slot(3) {
            self.render_icons(r, buf, ctx);
        }
    }

    /// Kinds and states, live.
    fn render_kinds(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let panel = Panel::card(Some("Playground")).meta("hover · click · Tab · Enter / Space");
        let bg = panel.bg(t);
        let inner = panel.render(area, buf, t);
        let mut y = inner.y;
        let groups: [(&str, &[usize]); 4] = [
            ("Actions", &[0, 1, 2, 3]),
            ("Toggles", &[4, 5]),
            ("Disabled", &[6, 7]),
            ("Busy", &[8]),
        ];
        for (label, idx) in groups {
            if y + 1 >= inner.bottom() {
                break;
            }
            crate::pages::layout::caption(inner.x, y, buf, t, label, bg);
            let widths: Vec<u16> = idx.iter().map(|&i| self.buttons[i].width()).collect();
            let rects = row_layout(Rect::new(inner.x, y + 1, inner.width, 1), &widths, 2);
            for (&i, r) in idx.iter().zip(rects) {
                self.buttons[i].render(r, buf, ctx, bg);
            }
            y += 3;
        }
        if let Some(last) = &self.last
            && y < inner.bottom()
        {
            let text = format!("last: {last} · {} activations", self.clicks);
            buf.set_string(inner.x, inner.bottom() - 1, &text, t.faint().bg(bg));
        }
    }

    /// A toolbar: buttons outside the Tab order (unless the option puts
    /// them in), each with a shortcut; Run is busy and Cancel enabled while
    /// the query runs.
    fn render_toolbar(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let meta = if self.tools[0].focusable {
            "in the Tab order"
        } else {
            "not in the Tab order"
        };
        let panel = Panel::card(Some("Toolbar")).meta(meta);
        let bg = panel.bg(t);
        let inner = panel.render(area, buf, t);
        let widths: Vec<u16> = self.tools.iter().map(Button::width).collect();
        let rects = row_layout(Rect::new(inner.x, inner.y, inner.width, 1), &widths, 1);
        for (b, r) in self.tools.iter_mut().zip(rects) {
            b.render(r, buf, ctx, bg);
        }
        if inner.y + 2 < inner.bottom() {
            let note = "F5 runs · Shift+F5 cancels · hover a tool for its tooltip";
            buf.set_string(
                inner.x,
                inner.y + 2,
                junie_tui::ui::text::truncate(note, inner.width as usize),
                t.faint().bg(bg),
            );
        }
    }

    /// The `ButtonRow` at the option's width, align, gap and overflow,
    /// what folded behind `…`, and a right-aligned "Next" row.
    fn render_rows(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let panel = Panel::card(Some("Button row")).meta("align · gap · overflow in Options");
        let bg = panel.bg(t);
        let inner = panel.render(area, buf, t);
        let w = self.row_width(inner.width);
        let row = self.button_row();
        let lines = row.rows(w, &self.row, t);
        let r = Rect::new(inner.x, inner.y, w, lines).intersection(inner);
        // the row's extent, so alignment and folding read at any width
        let edge = Rect::new(inner.x, r.bottom(), w, 1).intersection(inner);
        buf.set_string(
            edge.x,
            edge.y,
            "┈".repeat(edge.width as usize),
            t.faint().bg(bg),
        );
        self.layout = row.render(ROW, r, &mut self.row, buf, ctx, bg);
        let readout = if self.layout.hidden.is_empty() {
            format!("{w} columns · every button fits")
        } else {
            let names: Vec<&str> = self
                .layout
                .hidden
                .iter()
                .map(|&i| self.row[i].label.as_str())
                .collect();
            format!("{w} columns · behind …: {}", names.join(", "))
        };
        let y = edge.y + 1;
        if y < inner.bottom() {
            buf.set_string(
                inner.x,
                y,
                junie_tui::ui::text::truncate(&readout, inner.width as usize),
                t.muted().bg(bg),
            );
        }
        // a "Next" row: subtle follow-up actions, some not available yet
        let y = y + 2;
        if y < inner.bottom() {
            crate::pages::layout::caption(inner.x, y, buf, t, "Next", bg);
            let widths: Vec<u16> = self.next.iter().map(Button::width).collect();
            let rects = row_layout_right(
                Rect::new(inner.x + 5, y, inner.width.saturating_sub(5), 1),
                &widths,
                1,
            );
            for (b, r) in self.next.iter_mut().zip(rects) {
                b.render(r, buf, ctx, bg);
            }
        }
    }

    /// Icon buttons: the close plate, a title-ladder `+` on a five-cell
    /// plate, and a close with the danger hover.
    fn render_icons(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let panel = Panel::card(Some("Icon buttons")).meta("hover · press · click");
        let bg = panel.bg(t);
        let inner = panel.render(area, buf, t);
        let host = Style::default().fg(t.text_secondary).bg(bg);
        let captions = ["Close", "Title · 5-cell plate", "Danger hover"];
        let mut x = inner.x;
        for (b, cap) in self.icons.iter().zip(captions) {
            if x >= inner.right() {
                break;
            }
            buf.set_string(x, inner.y, cap, t.muted().bg(bg));
            let cw = junie_tui::ui::text::width(cap) as u16;
            b.render(x + cw + 1, inner.y, buf, ctx, host, None);
            x += cw + 1 + b.width(t) + 4;
        }
        if inner.y + 2 < inner.bottom() {
            let note = "hover a plate for its tooltip; focus stays with the owner";
            buf.set_string(
                inner.x,
                inner.y + 2,
                junie_tui::ui::text::truncate(note, inner.width as usize),
                t.faint().bg(bg),
            );
        }
    }

    fn on_row_click(&mut self, id: WidgetId, cx: &mut PageCtx) -> Option<Outcome> {
        if id == ButtonRow::more_id(ROW) {
            let names: Vec<&str> = self
                .layout
                .hidden
                .iter()
                .map(|&i| self.row[i].label.as_str())
                .collect();
            cx.status(format!("{} more: {}", names.len(), names.join(", ")));
            return Some(Outcome::Changed);
        }
        let b = self
            .row
            .iter_mut()
            .chain(self.next.iter_mut())
            .find(|b| b.id == id)?;
        if b.on_click() {
            cx.status(format!("{} ✓", b.label));
        }
        Some(Outcome::Changed)
    }
}

/// The States grid: every button kind in every state it has, drawn by
/// `Button` itself ([`junie_tui::ui::specimen`]), and the close
/// `IconButton`.
fn states_grid(opts: &PageValues) -> StateGrid {
    use SpecimenState::*;
    let id = ID.sub("states");
    let mut grid = StateGrid::new(id)
        .rows(&[
            Rest,
            Hover,
            Focused,
            FocusedHover,
            Pressed,
            Disabled,
            Selected,
            Busy,
        ])
        .row(Overflow)
        .column("Primary", Button::primary(id.child(0), "Label"))
        .column("Secondary", Button::secondary(id.child(1), "Label"))
        .column("Subtle", Button::subtle(id.child(2), "Label"))
        .column("Danger", Button::danger(id.child(3), "Label"))
        .column("Toggle", Button::toggle(id.child(4), "Label", false))
        .column("Icon ×", IconButton::close(id.child(5)))
        // six columns fit a 120-column terminal beside the sidebar
        .column_width(13);
    if let Some(w) = opts.number("buttons.label_width") {
        grid = grid.label_width(w);
    }
    if opts.value("buttons.focus_mark") == "colon" {
        grid = grid.focus_mark(":");
    }
    grid
}

impl Page for ButtonsPage {
    fn title(&self) -> &'static str {
        "Buttons"
    }
    fn blurb(&self) -> &'static str {
        "Primary, secondary, subtle, danger, toggle, disabled, busy"
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let body = self.template.render(area, buf, ctx);
        match self.template.section() {
            Section::Playground => self.render_playground(body, buf, ctx),
            Section::Options => {
                self.template.sync_options(|k| self.opts.get(k));
                self.template.render_options(body, buf, ctx);
            }
            Section::States => self.template.render_states(body, buf, ctx),
        }
    }

    fn handle(&mut self, ev: &PageEvent, cx: &mut PageCtx) -> Outcome {
        if let Some(o) = self.template.handle(ev, cx) {
            if let Some(set) = self.template.take_option() {
                self.set_option(set.key, set.value);
            }
            return o;
        }
        if self.template.section() == Section::Playground
            && let Some(o) = self.sections.handle(ev)
        {
            return o;
        }
        match ev {
            PageEvent::Tick => {
                let mut out = Outcome::Ignored;
                if let Some(ticks) = self.busy_ticks {
                    if ticks > 1 {
                        self.busy_ticks = Some(ticks - 1);
                    } else {
                        self.busy_ticks = None;
                        self.buttons[8].busy = false;
                        cx.status("Long job finished ✓");
                    }
                    out = Outcome::Changed;
                }
                if let Some(ticks) = self.run_ticks {
                    if ticks > 1 {
                        self.run_ticks = Some(ticks - 1);
                    } else {
                        self.run_ticks = None;
                        self.sync_run();
                        cx.status("Query finished · 42 rows");
                    }
                    out = Outcome::Changed;
                }
                out
            }
            PageEvent::Key(key) => {
                // the toolbar's shortcuts work wherever focus is on the page
                if key.code == KeyCode::F(5) {
                    self.tool(if key.shift() { CANCEL } else { RUN }, cx);
                    return Outcome::Changed;
                }
                let Some(f) = cx.focus.current() else {
                    return Outcome::Ignored;
                };
                if let Some(i) = self.buttons.iter().position(|b| b.id == f) {
                    let (out, activated) = self.buttons[i].on_key(key);
                    if activated {
                        self.activated(i, cx);
                    }
                    return out;
                }
                if let Some(i) = self.tools.iter().position(|b| b.id == f) {
                    let (out, activated) = self.tools[i].on_key(key);
                    if activated {
                        self.tool(i, cx);
                    }
                    return out;
                }
                let is_activate = key.is(KeyCode::Enter) || key.is_char(' ');
                if is_activate && f == ButtonRow::more_id(ROW) {
                    return self.on_row_click(f, cx).unwrap_or(Outcome::Ignored);
                }
                if let Some(b) = self
                    .row
                    .iter_mut()
                    .chain(self.next.iter_mut())
                    .find(|b| b.id == f)
                {
                    let (out, activated) = b.on_key(key);
                    if activated {
                        cx.status(format!("{} ✓", b.label));
                    }
                    return out;
                }
                Outcome::Ignored
            }
            PageEvent::Click { id, .. } => {
                if let Some(i) = self.buttons.iter().position(|b| b.id == *id) {
                    if self.buttons[i].on_click() {
                        self.activated(i, cx);
                    }
                    return Outcome::Changed;
                }
                if let Some(i) = self.tools.iter().position(|b| b.id == *id) {
                    self.tool(i, cx);
                    return Outcome::Changed;
                }
                if let Some(b) = self.icons.iter().find(|b| b.id == *id) {
                    let what = b.tip.as_ref().map_or("Icon", |t| t.text.as_str());
                    cx.status(format!("{what} ✓"));
                    return Outcome::Changed;
                }
                self.on_row_click(*id, cx).unwrap_or(Outcome::Ignored)
            }
            _ => Outcome::Ignored,
        }
    }

    fn hints(&self, focus: Option<WidgetId>) -> Vec<Hint> {
        self.template
            .hints(focus)
            .unwrap_or_else(|| vec![("Enter / Space", "Activate"), ("F5", "Run")])
    }

    fn animating(&self) -> bool {
        self.busy_ticks.is_some() || self.run_ticks.is_some()
    }

    fn option_value(&self, key: &str) -> Option<usize> {
        self.opts.get(key)
    }

    fn set_option(&mut self, key: &str, value: usize) {
        if self.opts.set(key, value) {
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
