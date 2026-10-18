use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;

use crate::app::PageId;
use crate::options::{PageValues, page_options};
use crate::pages::template::{Section, Template};
use crate::pages::{Cx, Hint, Page, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::focus::forward_blur;
use junie_tui::core::id::WidgetId;
use junie_tui::core::intent::Phase;
use junie_tui::core::keymap::{KeyBinding, NavKeys};
use junie_tui::core::pointer::MouseTarget;
use junie_tui::core::scroll::TrackClick;
use junie_tui::theme::{CursorStyle, Tone};
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::ui::layout::StackChild;
use junie_tui::ui::popup::{Dismiss, Place, Placement};
use junie_tui::ui::specimen::{SpecimenState, StateGrid};
use junie_tui::widgets::chips::{Chip, ChipAction, ChipBar, ChipCmd};
use junie_tui::widgets::empty::{self, EmptyState};
use junie_tui::widgets::icon_button::IconButton;
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::props::{self, Prop};
use junie_tui::widgets::segments::{self, Segment};
use junie_tui::widgets::select::{Select, SelectAction, SelectArrows, TabInOpen, WheelMode};

const ID: WidgetId = WidgetId::of("chips");

const CANDIDATES: &[&str] = &[
    "created_at > '2026-01-01'",
    "currency = 'EUR'",
    "notes is not null",
    "seats between 5 and 50",
];

const SORT_COLUMNS: &[&str] = &[
    "created_at",
    "total",
    "status",
    "customer",
    "country",
    "currency",
    "discount",
    "id",
    "notes",
    "owner",
    "seats",
    "updated_at",
];

pub struct ChipsPage {
    template: Template,
    opts: PageValues,
    filters: ChipBar,
    streams: ChipBar,
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
        let mut bad = Chip::new("amount > 'abc'");
        bad.error = true;
        filters.chips = vec![
            Chip::new("status = 'pending'"),
            Chip::new("total > 100"),
            Chip::new("country in (DE, FR)"),
            bad,
        ];
        filters.chips[2].enabled = false;
        let mut streams = ChipBar::new(ID.sub("streams"));
        streams.chips = ["api", "worker", "db", "cron"]
            .iter()
            .map(|l| {
                let mut c = Chip::new(l);
                c.removable = false;
                c
            })
            .collect();
        streams.chips[3].enabled = false;
        let mut page = Self {
            template: Template::new(ID, states_grid()),
            opts: PageValues::new(page_options(PageId::Chips)),
            filters,
            streams,
            match_all: true,
            next_candidate: 0,
            sort: Select::new(ID.sub("sort"), "Sort by", SORT_COLUMNS, 0),
            page_size: Select::new(ID.sub("size"), "Page size", &["25", "50", "100", "500"], 1),
            engine: Select::new(ID.sub("engine"), "Engine", &["PostgreSQL"], 0),
            last: "nothing yet".into(),
            sections: crate::pages::layout::Sections::new(ID.sub("page")),
        };
        page.rebuild();
        page
    }

    fn rebuild(&mut self) {
        let o = &self.opts;
        let nav = match o.value("chipsselects.chip_nav") {
            "vim" => NavKeys::VIM,
            "no-home-end" => NavKeys::default().home_end(false),
            _ => NavKeys::default(),
        };
        let keymap = match o.value("chipsselects.chip_keymap") {
            "custom" => vec![
                (KeyBinding::key(KeyCode::Delete), ChipCmd::Remove),
                (KeyBinding::char('n'), ChipCmd::Add),
                (KeyBinding::key(KeyCode::Delete).shift(), ChipCmd::ClearAll),
                (KeyBinding::char('*'), ChipCmd::Lead),
            ],
            _ => <ChipCmd as junie_tui::core::keymap::Actions>::default_keymap(),
        };
        let bar = |b: &ChipBar, id: WidgetId| {
            let mut n = ChipBar::new(id)
                .overflow_marker(o.on("chipsselects.overflow_marker"))
                .overflow_menu(o.on("chipsselects.overflow_menu"))
                .nav(nav)
                .keymap(keymap.clone());
            if o.value("chipsselects.close_button") == "danger" {
                n = n.close_button(
                    IconButton::close(WidgetId::of("chips.remove")).danger_hover(true),
                );
            }
            if o.value("chipsselects.more_button") == "wide" {
                n = n.more_button(IconButton::new(WidgetId::of("chips.more"), "+").pad(2));
            }
            if let Some(w) = o.number("chipsselects.close_target") {
                n = n.close_target_w(w);
            }
            n.chips = b.chips.clone();
            n.cursor = b.cursor;
            n.lead = b.lead.clone();
            n.add_label = b.add_label.clone();
            n
        };
        self.filters = bar(&self.filters, ID.sub("filters"));
        self.streams =
            bar(&self.streams, ID.sub("streams")).digit_toggles(o.on("chipsselects.digit_toggles"));
        self.streams.add_label = None;
        self.streams.lead = Some("streams".into());

        let select = |s: &Select| {
            let (min, max) = match o.value("chipsselects.popup_width") {
                "8-16" => (8, 16),
                "30-60" => (30, 60),
                _ => Select::POPUP_WIDTH,
            };
            let mut n = Select::new(
                s.id,
                &s.label,
                &s.options.iter().map(String::as_str).collect::<Vec<_>>(),
                s.selected,
            )
            .arrows(match o.value("chipsselects.arrows") {
                "step" => SelectArrows::Step,
                _ => SelectArrows::Open,
            })
            .wheel(match o.value("chipsselects.wheel") {
                "cursor" => WheelMode::Cursor,
                _ => WheelMode::Viewport,
            })
            .tab_in_open(match o.value("chipsselects.tab_in_open") {
                "close" => TabInOpen::Close,
                "swallow" => TabInOpen::Swallow,
                _ => TabInOpen::Choose,
            })
            .max_rows(
                o.number("chipsselects.max_rows")
                    .unwrap_or(Select::MAX_ROWS),
            )
            .placement(match o.value("chipsselects.placement") {
                "above" => Placement::Above,
                _ => Placement::Below,
            })
            .popup_fit(match o.value("chipsselects.popup_fit") {
                "flush" => Place::FLUSH,
                "menu" => Place::MENU,
                _ => Place::DEFAULT,
            })
            .popup_width(min, max)
            .cursor_style(match o.value("chipsselects.cursor_style") {
                "bar" => CursorStyle::Bar,
                _ => CursorStyle::Fill,
            })
            .hover_moves_cursor(o.on("chipsselects.hover_moves_cursor"))
            .press_drag(o.on("chipsselects.press_drag"))
            .label_hit(o.on("chipsselects.label_hit"))
            .wrap(o.on("chipsselects.wrap"))
            .type_ahead(o.on("chipsselects.type_ahead"))
            .close_on_blur(o.on("chipsselects.close_on_blur"))
            .nav(match o.value("chipsselects.select_nav") {
                "vim" => NavKeys::VIM,
                _ => NavKeys::default(),
            });
            match o.value("chipsselects.track_click") {
                "jump" => n = n.track_click(TrackClick::Jump),
                "page" => n = n.track_click(TrackClick::Page),
                _ => {}
            }
            n
        };
        let mut sort = select(&self.sort).help("Applies to the next query");
        if o.value("chipsselects.dismiss") == "escape" {
            sort = sort.dismiss(Dismiss::ESCAPE);
        }
        if o.on("chipsselects.mixed") {
            sort = sort.mixed("Mixed");
        }
        self.sort = sort;

        self.page_size = select(&self.page_size).dismiss(Dismiss::LIGHT.pass_through(true));
        self.engine = select(&self.engine)
            .disabled(true)
            .help("Fixed by the connection");
        self.sync_lead();
    }

    #[allow(dead_code)]
    pub fn sort(&self) -> &Select {
        &self.sort
    }

    fn sync_lead(&mut self) {
        self.filters.lead = Some(if self.match_all {
            "match all ▾".into()
        } else {
            "match any ▾".into()
        });
    }

    fn on_chip(&mut self, ev: Option<ChipAction>, cx: &mut Cx) {
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

    fn on_stream(&mut self, ev: Option<ChipAction>) {
        if let Some(ChipAction::Toggle(i) | ChipAction::Activate(i)) = ev {
            let c = &mut self.streams.chips[i];
            c.enabled = !c.enabled;
            self.last = format!(
                "stream {} {}",
                c.label,
                if c.enabled { "on" } else { "off" }
            );
        }
    }

    fn selects(&mut self) -> [&mut Select; 3] {
        [&mut self.sort, &mut self.page_size, &mut self.engine]
    }

    fn bar_width(&self, room: u16) -> u16 {
        self.opts
            .number("chipsselects.bar_width")
            .map_or(room, |w| w.min(room))
    }

    fn render_playground(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let has = |id| ctx.interaction.has_focus(id);
        let focus = if has(self.filters.id) {
            Some(0)
        } else if has(self.streams.id) {
            Some(1)
        } else if has(self.sort.id) || has(self.page_size.id) || has(self.engine.id) {
            Some(2)
        } else {
            None
        };
        let rows = self.sections.layout(
            area,
            &[
                StackChild::fixed(6),
                StackChild::fixed(5),
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
            Some(r) => self.render_streams(r, buf, ctx),
            None => ctx.focus_stop(self.streams.id),
        }
        match rows.slot(2) {
            Some(r) => self.render_selects(r, buf, ctx),
            None => {
                for sel in self.selects() {
                    ctx.focus_stop(sel.id);
                }
            }
        }
        if let Some(r) = rows.slot(3) {
            render_strip(r, buf, ctx);
        }
        if let Some(r) = rows.slot(4) {
            render_below(r, buf, ctx);
        }
    }

    fn render_filters(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let focused = ctx.interaction.focused(self.filters.id);
        let active = self.filters.chips.iter().filter(|c| c.enabled).count();
        let meta = format!("{active} active");
        let panel = Panel::card(Some("Filters")).focused(focused).meta(&meta);
        let bg = panel.bg(t);
        let inner = panel.render(area, buf, t);
        let w = self.bar_width(inner.width);
        self.filters
            .render(Rect::new(inner.x, inner.y, w, 1), buf, ctx, bg);
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

    fn render_streams(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let focused = ctx.interaction.focused(self.streams.id);
        let meta = if self.streams.digit_toggles {
            "not removable · 1–9 switch a stream"
        } else {
            "not removable · Space switches a stream"
        };
        let panel = Panel::card(Some("Streams")).focused(focused).meta(meta);
        let bg = panel.bg(t);
        let inner = panel.render(area, buf, t);
        let w = self.bar_width(inner.width);
        self.streams
            .render(Rect::new(inner.x, inner.y, w, 1), buf, ctx, bg);
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

        for (sel, cell) in self.selects().into_iter().zip(cells) {
            sel.render(cell, buf, ctx, bg);
        }
    }

    fn select_changed(sel: &Select, a: Option<SelectAction>, cx: &mut Cx) {
        if let Some(SelectAction::Changed(i)) = a {
            cx.status(format!("{} → {}", sel.label, sel.options[i]));
        }
    }
}

fn states_grid() -> StateGrid {
    use SpecimenState::*;
    let id = ID.sub("states");
    let mut bar = ChipBar::new(id.child(0));
    bar.chips = vec![Chip::new("status = 'ok'"), Chip::new("total > 9")];
    bar.add_label = None;
    StateGrid::new(id)
        .column_width(40)
        .rows(&[
            Rest,
            Hover,
            Pressed,
            Focused,
            FocusedHover,
            Disabled,
            Error,
            Empty,
            Overflow,
            Narrow,
        ])
        .column("Chip bar", bar)
        .column(
            "Select",
            Select::new(id.child(1), "Sort by", &["created_at", "total"], 0),
        )
}

impl Page for ChipsPage {
    fn title(&self) -> &'static str {
        "Chips & selects"
    }
    fn blurb(&self) -> &'static str {
        "Removable chips, a popup select, and strips that drop what does not fit"
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
        if let PageEvent::Blur(ch) = ev {
            return forward_blur(
                *ch,
                &mut [&mut self.sort, &mut self.page_size, &mut self.engine],
            );
        }
        if self.template.section() == Section::Playground
            && let Some(o) = self.sections.handle(ev)
        {
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
                if f == self.streams.id {
                    let (o, ev) = self.streams.on_key(key);
                    self.on_stream(ev);
                    return o;
                }
                for sel in self.selects() {
                    if sel.id == f {
                        let (o, a) = sel.on_key(key);
                        Self::select_changed(sel, a, cx);
                        return o;
                    }
                }
                Outcome::Ignored
            }
            PageEvent::Pointer(ev) => {
                let click = ev.is(Phase::Click);
                if let Some(r) = self.filters.route(ev) {
                    if click {
                        cx.focus.focus(self.filters.id);
                    }
                    let (o, ev) = r.into_parts();
                    self.on_chip(ev, cx);
                    return o;
                }
                if let Some(r) = self.streams.route(ev) {
                    if click {
                        cx.focus.focus(self.streams.id);
                    }
                    let (o, ev) = r.into_parts();
                    self.on_stream(ev);
                    return o;
                }

                for sel in self.selects() {
                    let sid = sel.id;
                    if let Some(r) = sel.route(ev) {
                        if (click || ev.is(Phase::Press)) && ev.id == sid {
                            cx.focus.focus(sid);
                        }
                        let (o, a) = r.into_parts();
                        Self::select_changed(sel, a, cx);
                        return o;
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
        if focus == Some(self.filters.id) {
            vec![
                ("←→", "Move"),
                ("Space", "Toggle"),
                ("Enter", "Edit / add"),
                ("x", "Remove"),
                ("X", "Clear all"),
            ]
        } else if focus == Some(self.streams.id) {
            if self.streams.digit_toggles {
                vec![("←→", "Move"), ("Space", "Switch"), ("1–9", "Switch")]
            } else {
                vec![("←→", "Move"), ("Space", "Switch")]
            }
        } else if focus
            .is_some_and(|f| [self.sort.id, self.page_size.id, self.engine.id].contains(&f))
        {
            vec![("Enter", "Open"), ("↑↓", "Choose"), ("Esc", "Close")]
        } else {
            vec![("Tab", "Next")]
        }
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
