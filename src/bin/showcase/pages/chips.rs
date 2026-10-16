//! Chips, selects, and the small read-only pieces: a segment strip that
//! drops low-priority items when narrow, a property block, an empty state.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::pages::{Hint, Page, PageCtx, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::focus::forward_blur;
use junie_tui::core::id::WidgetId;
use junie_tui::theme::Tone;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::ui::layout::StackChild;
use junie_tui::ui::popup::Dismiss;
use junie_tui::widgets::chips::{Chip, ChipAction, ChipBar};
use junie_tui::widgets::empty::{self, EmptyState};
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::props::{self, Prop};
use junie_tui::widgets::segments::{self, Segment};
use junie_tui::widgets::select::{Select, SelectAction};

const ID: WidgetId = WidgetId::of("chips");

const CANDIDATES: &[&str] = &[
    "created_at > '2026-01-01'",
    "currency = 'EUR'",
    "notes is not null",
    "seats between 5 and 50",
];

pub struct ChipsPage {
    filters: ChipBar,
    match_all: bool,
    next_candidate: usize,
    sort: Select,
    page_size: Select,
    engine: Select,
    last: String,
    sections: crate::pages::layout::Sections,
}

impl ChipsPage {
    pub fn new() -> Self {
        let mut filters = ChipBar::new(ID.sub("filters"));
        filters.chips = vec![
            Chip::new("status = 'pending'"),
            Chip::new("total > 100"),
            Chip::new("country in (DE, FR)"),
        ];
        filters.chips[2].enabled = false;
        filters.lead = Some("match all ▾".into());
        filters.add_label = Some("+ Add filter".into());
        Self {
            filters,
            match_all: true,
            next_candidate: 0,
            sort: Select::new(
                ID.sub("sort"),
                "Sort by",
                &["created_at", "total", "status", "customer"],
                0,
            )
            .help("Applies to the next query"),
            // demonstrates the pass-through policy: a click outside the open
            // list closes it and also reaches the control beneath
            page_size: Select::new(ID.sub("size"), "Page size", &["25", "50", "100", "500"], 1)
                .dismiss(Dismiss::LIGHT.pass_through(true)),
            engine: Select::new(ID.sub("engine"), "Engine", &["PostgreSQL"], 0)
                .disabled(true)
                .help("Fixed by the connection"),
            last: "nothing yet".into(),
            sections: crate::pages::layout::Sections::new(ID.sub("page")),
        }
    }

    fn sync_lead(&mut self) {
        self.filters.lead = Some(if self.match_all {
            "match all ▾".into()
        } else {
            "match any ▾".into()
        });
    }

    fn on_chip(&mut self, ev: Option<ChipAction>, cx: &mut PageCtx) {
        match ev {
            Some(ChipAction::Activate(i)) => {
                self.last = format!("edit {}", self.filters.chips[i].label);
                cx.status(format!(
                    "Would open the editor for {}",
                    self.filters.chips[i].label
                ));
            }
            Some(ChipAction::Toggle(i)) => {
                let c = &mut self.filters.chips[i];
                c.enabled = !c.enabled;
                self.last = format!(
                    "{} {}",
                    if c.enabled { "enabled" } else { "disabled" },
                    c.label
                );
            }
            Some(ChipAction::Remove(i)) => {
                let c = self.filters.chips.remove(i);
                self.last = format!("removed {}", c.label);
            }
            Some(ChipAction::Add) => {
                let label = CANDIDATES[self.next_candidate % CANDIDATES.len()];
                self.next_candidate += 1;
                self.filters.chips.push(Chip::new(label));
                self.filters.cursor = self.filters.chips.len() - 1;
                self.last = format!("added {label}");
            }
            Some(ChipAction::ClearAll) => {
                self.filters.chips.clear();
                self.last = "cleared all filters".into();
            }
            Some(ChipAction::Lead) => {
                self.match_all = !self.match_all;
                self.sync_lead();
                self.last = format!("match {}", if self.match_all { "all" } else { "any" });
            }
            None => {}
        }
    }

    fn selects(&mut self) -> [&mut Select; 3] {
        [&mut self.sort, &mut self.page_size, &mut self.engine]
    }
}

impl Page for ChipsPage {
    fn title(&self) -> &'static str {
        "Chips & selects"
    }
    fn blurb(&self) -> &'static str {
        "Removable chips, a popup select, and strips that drop what does not fit"
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        // whole sections scroll on a short terminal; nothing is drawn
        // past the page into the row above the footer
        let has = |id| ctx.interaction.has_focus(id);
        let focus = if has(self.filters.id) {
            Some(0)
        } else if has(self.sort.id) || has(self.page_size.id) || has(self.engine.id) {
            Some(1)
        } else {
            None
        };
        let rows = self.sections.layout(
            area,
            &[
                StackChild::fixed(6),
                StackChild::fixed(8),
                StackChild::fixed(7),
                StackChild::flex(3, 11),
            ],
            focus,
            buf,
            ctx,
        );
        match rows.slot(0) {
            Some(r) => self.render_filters(r, buf, ctx),
            None => ctx.focus_stop(self.filters.id),
        }
        match rows.slot(1) {
            Some(r) => self.render_selects(r, buf, ctx),
            None => {
                for sel in self.selects() {
                    ctx.focus_stop(sel.id);
                }
            }
        }
        if let Some(r) = rows.slot(2) {
            render_strip(r, buf, ctx);
        }
        if let Some(r) = rows.slot(3) {
            render_below(r, buf, ctx);
        }
    }

    fn handle(&mut self, ev: &PageEvent, cx: &mut PageCtx) -> Outcome {
        if let PageEvent::Blur(ch) = ev {
            return forward_blur(
                *ch,
                &mut [&mut self.sort, &mut self.page_size, &mut self.engine],
            );
        }
        if let Some(o) = self.sections.handle(ev) {
            return o;
        }
        match ev {
            PageEvent::Key(key) => {
                let Some(f) = cx.focus.current() else {
                    return Outcome::Ignored;
                };
                if f == self.filters.id {
                    let (o, ev) = self.filters.on_key(key);
                    self.on_chip(ev, cx);
                    return o;
                }
                for sel in self.selects() {
                    if sel.id == f {
                        let (o, ev) = sel.on_key(key);
                        if let Some(SelectAction::Changed(i)) = ev {
                            let msg = format!("{} → {}", sel.label, sel.options[i]);
                            cx.status(msg);
                        }
                        return o;
                    }
                }
                Outcome::Ignored
            }
            PageEvent::Click { id, .. } => {
                if self.filters.owns(*id) {
                    cx.focus.focus(self.filters.id);
                    let (o, ev) = self.filters.on_click(*id);
                    self.on_chip(ev, cx);
                    return o;
                }
                // a click outside an open list lands on its backdrop, which
                // the select owns: it closes itself per its dismiss policy
                for sel in self.selects() {
                    if sel.owns(*id) {
                        if *id == sel.id {
                            cx.focus.focus(sel.id);
                        }
                        let (o, ev) = sel.on_click(*id);
                        if let Some(SelectAction::Changed(i)) = ev {
                            let msg = format!("{} → {}", sel.label, sel.options[i]);
                            cx.status(msg);
                        }
                        return o;
                    }
                }
                Outcome::Ignored
            }
            PageEvent::Wheel { id, delta } => {
                for sel in self.selects() {
                    if sel.open && sel.owns(*id) {
                        return sel.on_wheel(*delta);
                    }
                }
                Outcome::Ignored
            }
            _ => Outcome::Ignored,
        }
    }

    fn hints(&self, focus: Option<WidgetId>) -> Vec<Hint> {
        if focus == Some(self.filters.id) {
            vec![
                ("←→", "Move"),
                ("Space", "Toggle"),
                ("Enter", "Edit / add"),
                ("x", "Remove"),
                ("X", "Clear all"),
            ]
        } else if focus
            .is_some_and(|f| [self.sort.id, self.page_size.id, self.engine.id].contains(&f))
        {
            vec![("Enter", "Open"), ("↑↓", "Choose"), ("Esc", "Close")]
        } else {
            vec![("Tab", "Next")]
        }
    }
}

impl ChipsPage {
    fn render_filters(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;

        // filters
        let focused = ctx.interaction.focused(self.filters.id);
        let active = self.filters.chips.iter().filter(|c| c.enabled).count();
        let meta = format!("{active} active");
        let panel = Panel::card(Some("Filters")).focused(focused).meta(&meta);
        let bg = panel.bg(t);
        let inner = panel.render(area, buf, t);
        self.filters
            .render(Rect::new(inner.x, inner.y, inner.width, 1), buf, ctx, bg);
        if inner.y + 2 < inner.bottom() {
            buf.set_string(
                inner.x,
                inner.y + 2,
                junie_tui::ui::text::truncate(
                    &format!("last action: {}", self.last),
                    inner.width as usize,
                ),
                t.muted().bg(bg),
            );
        }
    }

    fn render_selects(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let panel = Panel::card(Some("Selects"));
        let bg = panel.bg(t);
        let inner = panel.render(area, buf, t);
        let third = inner.width / 3;
        let cells = [
            Rect::new(inner.x, inner.y, third.saturating_sub(2), Select::HEIGHT),
            Rect::new(
                inner.x + third,
                inner.y,
                third.saturating_sub(2),
                Select::HEIGHT,
            ),
            Rect::new(
                inner.x + third * 2,
                inner.y,
                inner.width.saturating_sub(third * 2),
                Select::HEIGHT,
            ),
        ];
        // an open list is an overlay: it lands above everything drawn below
        for (sel, cell) in self.selects().into_iter().zip(cells) {
            sel.render(cell, buf, ctx, bg);
        }
    }
}

/// The segment strip at full width and at 44 columns.
fn render_strip(area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
    let t = ctx.theme;
    let panel = Panel::card(Some("Segment strip"));
    let bg = panel.bg(t);
    let inner = panel.render(area, buf, t);
    let left = [
        Segment::new("▪", Tone::Success).priority(9),
        Segment::new("Acme", Tone::Normal).bold().priority(9),
        Segment::new("◆ production", Tone::Normal)
            .bold()
            .priority(8),
        Segment::new("acme_prod › public", Tone::Secondary).priority(6),
        Segment::new("safe", Tone::Normal).bold().priority(7),
    ];
    let right = [
        Segment::new("3 pending", Tone::Warning).priority(5),
        Segment::new("truecolor · 120×40", Tone::Muted).priority(2),
        Segment::new("? help", Tone::Muted).priority(3),
    ];
    segments::render(
        Rect::new(inner.x, inner.y, inner.width, 1),
        buf,
        ctx,
        &left,
        &right,
        bg,
    );
    let narrow = Rect::new(inner.x, inner.y + 2, inner.width.min(44), 1).intersection(inner);
    segments::render(narrow, buf, ctx, &left, &right, bg);
    if inner.y + 3 < inner.bottom() {
        buf.set_string(
            inner.x,
            inner.y + 3,
            junie_tui::ui::text::truncate(
                "the same strip at 44 columns: low-priority segments leave first, from the right",
                inner.width as usize,
            ),
            t.faint().bg(bg),
        );
    }
}

/// Properties beside an empty state.
fn render_below(below: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
    let t = ctx.theme;
    let (l, r) = crate::pages::layout::columns(below, (below.width * 55 / 100).max(36), 2);
    let panel = Panel::card(Some("Properties"));
    let bg = panel.bg(t);
    let inner = panel.render(Rect::new(l.x, l.y, l.width, l.height.min(11)), buf, t);
    let props = vec![
        Prop::new("Engine", "PostgreSQL 16.3"),
        Prop::new("Host", "prod-db-1.acme.io:5432"),
        Prop::new("Environment", "production"),
        Prop::new(
            "Safe Mode",
            "Writes ask for confirmation and a deliberate acknowledgement.",
        )
        .wrap(),
        Prop::new("Last used", "1 hour ago").tone(Tone::Muted),
    ];
    props::render(inner, buf, t, &props, bg);

    let panel = Panel::card(Some("Empty state"));
    let bg = panel.bg(t);
    let inner = panel.render(Rect::new(r.x, r.y, r.width, r.height.min(11)), buf, t);
    empty::render(
        inner,
        buf,
        t,
        &EmptyState::new("No results yet")
            .hint("A title and one hint, centred in whatever is left"),
        bg,
    );
}
