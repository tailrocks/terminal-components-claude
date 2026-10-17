//! Meters (Feedback, plan §7 SC36): `MeterList`, labelled capacity meters
//! in aligned columns, as the apps use them: a plan's quota windows in
//! block meters with domain tones (jackin usage and accounts), filesystems
//! on owner thresholds with the room a narrow list keeps (holla disk), and
//! a host's load on the shared thresholds (holla snapshot). Refresh puts
//! every meter into its refreshing tone; Offline keeps the last-good
//! values, drawn stale. Options: `crate::options::METERS`.

use ratatui::buffer::Buffer;
use ratatui::layout::{Rect, Size};

use crate::app::PageId;
use crate::options::{PageValues, page_options};
use crate::pages::template::{Section, Template};
use crate::pages::{Cx, Hint, Page, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::WidgetId;
use junie_tui::core::pointer::MouseTarget;
use junie_tui::theme::Tone;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::ui::layout::StackChild;
use junie_tui::ui::measure::Measure;
use junie_tui::ui::specimen::{SpecimenState, StateGrid};
use junie_tui::widgets::button::Button;
use junie_tui::widgets::meter_list::{MeterList, MeterRow};
use junie_tui::widgets::page_header::{Heading, PageHeader};
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::progress::{Meter, MeterLevel, MeterTone, MeterVisual, StaleStyle};

const ID: WidgetId = WidgetId::of("meters");

/// Ticks a refresh runs (the meters show their refreshing tone).
const REFRESH_TICKS: u16 = 24;

/// A plan's quota windows: label, used %, the domain's tone, detail.
const QUOTAS: [(&str, Option<u8>, MeterTone, &str); 5] = [
    (
        "Session (5 h)",
        Some(38),
        MeterTone::Fixed(MeterLevel::Low),
        "resets in 2 h 14 m",
    ),
    ("Weekly", Some(72), MeterTone::Normal, "resets Mon 09:00"),
    (
        "Opus weekly",
        Some(86),
        MeterTone::Warning,
        "provider warns at 85 %",
    ),
    (
        "Extra credits",
        Some(100),
        MeterTone::Exhausted,
        "$40 of $40 spent",
    ),
    (
        "Org seats",
        None,
        MeterTone::Unknown,
        "no quota on this plan",
    ),
];

/// Filesystems on the disk thresholds (attention at 80, danger at 95).
const DISKS: [(&str, u8, &str); 4] = [
    ("/", 62, "310 of 500 GB"),
    ("/System/Volumes/Data", 84, "840 of 1000 GB"),
    ("/Volumes/Backup", 97, "1.94 of 2 TB"),
    ("/Volumes/Scratch", 12, "61 of 512 GB"),
];

/// A host's load on the shared thresholds.
const HOST: [(&str, u8, &str); 4] = [
    ("CPU", 23, "8 cores · load 1.84"),
    ("Memory", 68, "21.8 of 32 GB"),
    ("Swap", 4, "0.2 of 4 GB"),
    ("GPU", 91, "render queue full"),
];

pub fn page() -> Box<dyn Page> {
    Box::new(MetersPage::new())
}

pub struct MetersPage {
    template: Template,
    opts: PageValues,
    refresh: Button,
    offline: Button,
    /// Ticks left of a running refresh.
    refreshing: u16,
    /// Completed refreshes: each moves the quota windows on.
    epoch: u8,
    sections: crate::pages::layout::Sections,
}

impl MetersPage {
    pub fn new() -> Self {
        let opts = PageValues::new(page_options(PageId::Meters));
        Self {
            template: Template::new(ID, states_grid()),
            offline: Button::toggle(ID.sub("offline"), "Offline", opts.on("meters.offline")),
            opts,
            refresh: Button::secondary(ID.sub("refresh"), "Refresh"),
            refreshing: 0,
            epoch: 0,
            sections: crate::pages::layout::Sections::new(ID.sub("page")),
        }
    }

    /// The column options every list on the page shares.
    fn columns(&self, list: MeterList) -> MeterList {
        let mut list = list;
        if let Ok(w) = self.opts.value("meters.label_width").parse::<u16>() {
            list = list.label_width(w);
        }
        if let Ok(w) = self.opts.value("meters.meter_width").parse::<u16>() {
            list = list.meter_width(w);
        }
        list = list.label_gap(self.opts.value("meters.label_gap").parse().unwrap_or(1));
        list.detail_tone(match self.opts.value("meters.detail_tone") {
            "secondary" => Tone::Secondary,
            _ => Tone::Muted,
        })
    }

    /// A row with the page-wide tone overrides: refreshing while a refresh
    /// runs, last-good while offline.
    fn row(&self, label: &str, meter: Meter, detail: &str) -> MeterRow {
        let mut meter = meter.stale_style(match self.opts.value("meters.stale_style") {
            "faint" => StaleStyle::Faint,
            _ => StaleStyle::Dashed,
        });
        if self.refreshing > 0 {
            meter = meter.tone(MeterTone::Refreshing);
        } else if self.opts.on("meters.offline") {
            meter = meter.stale(true);
        }
        let row = MeterRow::new(label, meter);
        if self.opts.on("meters.detail") {
            row.detail(detail)
        } else {
            row
        }
    }

    fn quotas(&self) -> MeterList {
        let visual = match self.opts.value("meters.visual") {
            "line" => MeterVisual::Line,
            _ => MeterVisual::Block,
        };
        let rows = QUOTAS
            .iter()
            .map(|(label, pct, tone, detail)| {
                // each refresh moves the healthy windows on a little
                let pct = pct.map(|p| {
                    if p < 80 {
                        (p + self.epoch * 3).min(79)
                    } else {
                        p
                    }
                });
                let meter = Meter::new(pct)
                    .value(pct.map(|p| format!("{p:>3}%")).unwrap_or_default())
                    .tone(*tone)
                    .visual(visual);
                self.row(label, meter, detail)
            })
            .collect();
        self.columns(MeterList::new(rows))
    }

    fn disks(&self) -> MeterList {
        let disk = MeterTone::Level { warn: 80, crit: 95 };
        let rows = DISKS
            .iter()
            .map(|(mount, pct, detail)| {
                let meter = Meter::new(Some(*pct))
                    .value(format!("{pct}%"))
                    .tone(disk)
                    .visual(MeterVisual::Line);
                self.row(mount, meter, detail)
            })
            .collect();
        // a narrow list shrinks the meter first and drops it below 8 cells
        self.columns(MeterList::new(rows))
            .label_tone(Tone::Secondary)
            .pad_label(true)
            .reserve(4)
            .min_meter(8)
    }

    fn host(&self) -> MeterList {
        let rows = HOST
            .iter()
            .map(|(label, pct, detail)| {
                let meter = Meter::new(Some(*pct))
                    .value(format!("{pct}%"))
                    .visual(MeterVisual::Block);
                self.row(label, meter, detail)
            })
            .collect();
        self.columns(MeterList::new(rows))
            .label_tone(Tone::Muted)
            .detail_gap(3)
    }

    /// The page's header: its status follows the offline option, as a
    /// usage screen's does.
    fn header(&self) -> PageHeader {
        let offline = self.opts.on("meters.offline");
        let (status, tone) = if offline {
            (" · offline · last good 4 min ago", Tone::Warning)
        } else {
            (" · live", Tone::Success)
        };
        PageHeader::new("Usage and capacity")
            .status(status, tone)
            .right("refreshes every 2 s")
            .right_gap(4)
            .inset(1)
            .subtitle(
                "Plan quotas, filesystems and host load in aligned columns",
                Tone::Muted,
            )
    }

    fn render_playground(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        // the header, a group heading, then the cards
        let header = self.header();
        let rows = header.render(area, buf, ctx.theme);
        debug_assert_eq!(rows, header.height());
        let body = header.body(area);
        sections_heading().render(
            Rect::new(body.x + 1, body.y, body.width.saturating_sub(2), 1),
            buf,
            ctx.theme,
            ctx.theme.canvas,
        );
        let area = Rect::new(
            body.x,
            body.y + 2,
            body.width,
            body.height.saturating_sub(2),
        );
        let focus = [self.refresh.id, self.offline.id]
            .iter()
            .any(|&id| ctx.interaction.has_focus(id))
            .then_some(0);
        let (quotas, disks, host) = (self.quotas(), self.disks(), self.host());
        let children = Self::children(&quotas, &disks, &host);
        let l = self.sections.layout(area, &children, focus, buf, ctx);
        match l.slot(0) {
            Some(r) => self.render_quotas(r, &quotas, buf, ctx),
            None => {
                ctx.focus_stop(self.refresh.id);
                ctx.focus_stop(self.offline.id);
            }
        }
        if let Some(r) = l.slot(1) {
            card(r, buf, ctx, DISKS_CARD.0, DISKS_CARD.1, &disks);
        }
        if let Some(r) = l.slot(2) {
            card(r, buf, ctx, HOST_CARD.0, HOST_CARD.1, &host);
        }
    }

    /// The cards at their natural heights: a card takes its title row,
    /// a blank row and a bottom row around its body; the quotas card its
    /// button row and a blank row too.
    fn children(quotas: &MeterList, disks: &MeterList, host: &MeterList) -> [StackChild; 3] {
        [
            StackChild::fixed(quotas.height() + 5),
            StackChild::fixed(disks.height() + 3),
            StackChild::fixed(host.height() + 3),
        ]
    }

    /// The quotas card's meta for the page's state.
    fn quotas_meta(&self) -> &'static str {
        if self.refreshing > 0 {
            "refreshing…"
        } else if self.opts.on("meters.offline") {
            "offline · last-good values"
        } else {
            "domain tones · measured labels"
        }
    }

    fn render_quotas(
        &mut self,
        area: Rect,
        list: &MeterList,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
    ) {
        let t = ctx.theme;
        let panel = Panel::card(Some("Plan usage")).meta(self.quotas_meta());
        let bg = panel.bg(t);
        let inner = panel.render(area, buf, t);
        list.render(inner, buf, ctx, bg);
        let rects = junie_tui::widgets::button::row_layout(
            Rect::new(inner.x, inner.y + list.height() + 1, inner.width, 1),
            &[self.refresh.width(), self.offline.width()],
            2,
        );
        self.refresh.busy = self.refreshing > 0;
        self.offline.on = Some(self.opts.on("meters.offline"));
        self.refresh.render(rects[0], buf, ctx, bg);
        self.offline.render(rects[1], buf, ctx, bg);
    }

    fn start_refresh(&mut self, cx: &mut Cx) {
        if self.opts.on("meters.offline") {
            cx.status("Offline: the meters keep their last-good values");
            return;
        }
        self.refreshing = REFRESH_TICKS;
    }

    fn toggle_offline(&mut self) {
        let on = !self.opts.on("meters.offline");
        self.opts.set("meters.offline", usize::from(on));
        self.refreshing = 0;
    }
}

/// The Filesystems and Host cards' titles and metas.
const DISKS_CARD: (&str, &str) = ("Filesystems", "disk 80 / 95 · reserve 4 · min 8");
const HOST_CARD: (&str, &str) = ("Host", "shared thresholds · detail gap 3");

/// The heading over the cards.
fn sections_heading() -> Heading {
    Heading::new("Sections").meta("3 cards").tone(Tone::Muted)
}

/// A card holding one meter list.
fn card(
    area: Rect,
    buf: &mut Buffer,
    ctx: &mut RenderCtx,
    title: &str,
    meta: &str,
    list: &MeterList,
) {
    let t = ctx.theme;
    let panel = Panel::card(Some(title)).meta(meta);
    let bg = panel.bg(t);
    let inner = panel.render(area, buf, t);
    list.render(inner, buf, ctx, bg);
}

/// The States grid: a quota list (block) and a filesystem list (line) in
/// their content states.
fn states_grid() -> StateGrid {
    use SpecimenState::*;
    StateGrid::new(ID.sub("states"))
        .label_width(11)
        .column_width(34)
        .narrow_width(20)
        .rows(&[Rest, Loading, Error, Overflow, Narrow])
        .column(
            "Quota list",
            MeterList::new(vec![
                MeterRow::new(
                    "Session",
                    Meter::new(Some(38))
                        .value(" 38%")
                        .visual(MeterVisual::Block),
                )
                .detail("2 h"),
                MeterRow::new(
                    "Weekly",
                    Meter::new(Some(86))
                        .value(" 86%")
                        .tone(MeterTone::Warning)
                        .visual(MeterVisual::Block),
                )
                .detail("Mon"),
            ])
            .meter_width(14),
        )
        .column(
            "Disk list",
            MeterList::new(vec![
                MeterRow::new(
                    "/",
                    Meter::new(Some(62))
                        .value("62%")
                        .tone(MeterTone::Level { warn: 80, crit: 95 }),
                )
                .detail("310 GB"),
                MeterRow::new(
                    "/Backup",
                    Meter::new(Some(97))
                        .value("97%")
                        .tone(MeterTone::Level { warn: 80, crit: 95 }),
                )
                .detail("1.9 TB"),
            ])
            .label_tone(Tone::Secondary)
            .meter_width(14)
            .reserve(2)
            .min_meter(8),
        )
}

impl Page for MetersPage {
    fn title(&self) -> &'static str {
        "Meters"
    }
    fn blurb(&self) -> &'static str {
        "MeterList: labelled capacity meters in aligned columns, with tones and levels"
    }

    /// Everything whole: the header, the heading, each card at its meter
    /// list's natural width (the quotas card at least its buttons); the
    /// header's rows, a blank row, the heading, a blank row, the cards.
    fn measure(&self, _avail: Size, ctx: &RenderCtx) -> Size {
        let at = |m: &dyn Measure| m.measure(Size::ZERO, ctx);
        let (quotas, disks, host) = (self.quotas(), self.disks(), self.host());
        let header = at(&self.header());
        let buttons = self.refresh.width() + 2 + self.offline.width();
        let q = at(&quotas);
        let width = [
            header.width,
            at(&sections_heading()).width + 2,
            Panel::card(Some("Plan usage"))
                .meta(self.quotas_meta())
                .outer(Size::new(q.width.max(buttons), 0))
                .width,
            Panel::card(Some(DISKS_CARD.0))
                .meta(DISKS_CARD.1)
                .outer(at(&disks))
                .width,
            Panel::card(Some(HOST_CARD.0))
                .meta(HOST_CARD.1)
                .outer(at(&host))
                .width,
        ]
        .into_iter()
        .max()
        .unwrap_or(0);
        let cards = self
            .sections
            .stack
            .measure(&Self::children(&quotas, &disks, &host));
        Size::new(width, header.height + 1 + 2 + cards)
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
            if self.refreshing > 0 {
                self.refreshing -= 1;
                if self.refreshing == 0 {
                    self.epoch = self.epoch.wrapping_add(1) % 4;
                    cx.status("Usage refreshed ✓");
                }
                return Outcome::Changed;
            }
            return Outcome::Ignored;
        }
        if let Some(o) = self.template.handle(ev, cx) {
            return o;
        }
        if self.template.section() == Section::Playground
            && let Some(o) = self.sections.handle(ev)
        {
            return o;
        }
        match ev {
            PageEvent::Key(key) => {
                if cx.focus.is(self.refresh.id) {
                    let (o, act) = self.refresh.on_key(key);
                    if act {
                        self.start_refresh(cx);
                    }
                    return o;
                }
                if cx.focus.is(self.offline.id) {
                    let (o, act) = self.offline.on_key(key);
                    if act {
                        self.toggle_offline();
                    }
                    return o;
                }
                Outcome::Ignored
            }
            PageEvent::Pointer(ev) => {
                if let Some(r) = self.refresh.route(ev)
                    && r.outcome() != Outcome::Ignored
                {
                    if r.into_action().is_some() {
                        self.start_refresh(cx);
                    }
                    return Outcome::Changed;
                }
                if let Some(r) = self.offline.route(ev)
                    && r.outcome() != Outcome::Ignored
                {
                    if r.into_action().is_some() {
                        self.toggle_offline();
                    }
                    return Outcome::Changed;
                }
                Outcome::Ignored
            }
            _ => Outcome::Ignored,
        }
    }

    fn animating(&self) -> bool {
        self.refreshing > 0
    }

    fn hints(&self, focus: Option<WidgetId>) -> Vec<Hint> {
        if let Some(h) = self.template.hints(focus) {
            return h;
        }
        vec![("Enter", "Activate")]
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
