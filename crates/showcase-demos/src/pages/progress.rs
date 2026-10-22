use std::time::Duration;

use termrock::{
    Button, Color, Cx, Family, FgStep, FrameRead, GlyphRole, Id, Meter, MeterTone, MeterVisual,
    Modifier, PaintStyle, Panel, Part, ProgressBar, Rect, Response, Role, ScrollState, Slot,
    Spinner, StateFlags, Status, StylePatch, Surface, Ui, Variant, width,
};

use super::{Page, PageStatus, PageUpdate, frame};

const RESTART: Id = termrock::Id::root("showcase_ui::pages::progress::progress.restart");
const PAUSE: Id = termrock::Id::root("showcase_ui::pages::progress::progress.pause");
const PANEL_LIVE: Id = termrock::Id::root("showcase_ui::pages::progress::progress.live");
const PANEL_JOBS: Id = termrock::Id::root("showcase_ui::pages::progress::progress.jobs");
const PANEL_METERS: Id = termrock::Id::root("showcase_ui::pages::progress::progress.meters");
const BAR_BUILD: Id = termrock::Id::root("showcase_ui::pages::progress::progress.live.build");
const BAR_SWEEP: Id = termrock::Id::root("showcase_ui::pages::progress::progress.live.sweep");
const SPINNER_LIVE: Id = termrock::Id::root("showcase_ui::pages::progress::progress.live.spinner");
const BAR_JOBS: Id = termrock::Id::root("showcase_ui::pages::progress::progress.jobs.bar");
const METER_LOW_LINE: Id =
    termrock::Id::root("showcase_ui::pages::progress::progress.meters.low.line");
const METER_LOW_BLOCK: Id =
    termrock::Id::root("showcase_ui::pages::progress::progress.meters.low.block");
const METER_WIDE: Id = termrock::Id::root("showcase_ui::pages::progress::progress.meters.wide");

const SECTIONS: [&str; 3] = ["Playground", "Options", "States"];
const SECTION_CAPTION: &str = "the live widget";
const SECTION_WIDE: u16 = 76;
const BLURB: &str = "Determinate, indeterminate, an aggregate tally, capacity meters";
// Breakpoints, not exact sizes: narrow shells show the compact strip, and the
// wide meter rows need the 160×50 shell width.
const FULL_MIN_W: u16 = 60;
const WIDE_MIN_W: u16 = 134;
const NARROW_CONTENT_W: u16 = 94;

const CANVAS: u32 = 0x000000;
const SURFACE: u32 = 0x111111;
const WHITE: u32 = 0xffffff;
const SECONDARY: u32 = 0xb3b3b3;
const MUTED: u32 = 0x8a8a8a;
const FAINT: u32 = 0x737373;
const RULE: u32 = 0x3a3a3a;
const ACCENT: u32 = 0x48e054;
const SUCCESS: u32 = 0x3fb950;
const ERROR: u32 = 0xf05d5d;
const WARNING: u32 = 0xf59e09;
const TAB_ACTIVE: u32 = 0x27272a;

const LABEL_W: u16 = 16;
const BAR_W: u16 = 70;
const METER_W: u16 = 34;
const LIVE_H: u16 = 11;
const JOBS_H: u16 = 12;
const METERS_H: u16 = 15;
const CONTENT_H: u16 = LIVE_H + 1 + JOBS_H + 1 + METERS_H;

const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

const NOTES: [&str; 2] = [
    "Capacity meters are never green: low ≤ 59 % grey, medium ≤ 84 % warning ▲, high error !.",
    "Stale keeps its level, dashed.",
];

#[derive(Clone, Copy)]
enum BarKind {
    Active,
    Done,
    Error,
    Paused,
    Sweep,
}

struct Job {
    label: &'static str,
    ratio: Option<f64>,
    kind: BarKind,
}

const JOBS: &[Job] = &[
    Job {
        label: "Build image",
        ratio: Some(1.0),
        kind: BarKind::Done,
    },
    Job {
        label: "Unit tests",
        ratio: Some(1.0),
        kind: BarKind::Done,
    },
    Job {
        label: "Integration",
        ratio: Some(0.64),
        kind: BarKind::Error,
    },
    Job {
        label: "Push to registry",
        ratio: Some(0.5),
        kind: BarKind::Active,
    },
    Job {
        label: "Migrate schema",
        ratio: Some(0.3),
        kind: BarKind::Paused,
    },
    Job {
        label: "Warm caches",
        ratio: None,
        kind: BarKind::Sweep,
    },
    Job {
        label: "Notify",
        ratio: Some(0.0),
        kind: BarKind::Active,
    },
];

fn rgb(hex: u32) -> Color {
    Color::Rgb(
        ((hex >> 16) & 0xff) as u8,
        ((hex >> 8) & 0xff) as u8,
        (hex & 0xff) as u8,
    )
}

fn ink(fg: u32, bg: u32) -> PaintStyle {
    PaintStyle::new().fg(rgb(fg)).bg(rgb(bg))
}

fn settled(fg: u32, bg: u32, bold: bool) -> PaintStyle {
    let style = ink(fg, bg);
    if bold {
        style
            .add_modifier(Modifier::BOLD)
            .remove_modifier(Modifier::all().difference(Modifier::BOLD))
    } else {
        style.remove_modifier(Modifier::all())
    }
}

fn face(fg: u32, bg: u32, modifier: Modifier) -> PaintStyle {
    ink(fg, bg)
        .add_modifier(modifier)
        .remove_modifier(Modifier::all().difference(modifier))
}

fn tab_width(label: &str) -> u16 {
    width(label).saturating_add(3)
}

fn compact_page(area: Rect) -> bool {
    area.width < FULL_MIN_W
}

fn wide_page(area: Rect) -> bool {
    area.width >= WIDE_MIN_W
}

fn paint_blurb(ui: &mut Ui<'_>, body: Rect) {
    let title_w = width("Progress");
    if body.width <= title_w.saturating_add(4) {
        return;
    }
    let _ = ui.paint_str(
        Rect::new(
            body.x.saturating_add(title_w).saturating_add(2),
            body.y.saturating_sub(2),
            body.width.saturating_sub(title_w).saturating_sub(3),
            1,
        ),
        BLURB,
        ink(MUTED, CANVAS),
    );
}

fn paint_section_strip(ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    let mut x = area.x;
    let mut tab_end = area.x;
    for (index, label) in SECTIONS.iter().enumerate() {
        let w = tab_width(label);
        if x.saturating_add(w) > area.right() {
            break;
        }
        let style = if index == 0 {
            ink(WHITE, TAB_ACTIVE).add_modifier(Modifier::BOLD)
        } else {
            ink(SECONDARY, CANVAS)
        };
        ui.fill(Rect::new(x, area.y, w, 1), style);
        let _ = ui.paint_str(
            Rect::new(x.saturating_add(1), area.y, w.saturating_sub(1), 1),
            label,
            style,
        );
        tab_end = x.saturating_add(w);
        x = tab_end.saturating_add(1);
    }
    if area.width >= SECTION_WIDE {
        let caption_width = width(SECTION_CAPTION);
        let caption_x = area.right().saturating_sub(caption_width.saturating_add(1));
        if caption_x > tab_end.saturating_add(2) {
            let _ = ui.paint_str(
                Rect::new(caption_x, area.y, caption_width, 1),
                SECTION_CAPTION,
                ink(FAINT, CANVAS),
            );
        }
    }
    if area.height < 2 {
        return;
    }
    let rule_y = area.y.saturating_add(1);
    let _ = ui.paint_str(
        Rect {
            y: rule_y,
            height: 1,
            ..area
        },
        &"─".repeat(usize::from(area.width)),
        ink(RULE, CANVAS),
    );
    let active = tab_width(SECTIONS[0]).min(area.width);
    let _ = ui.paint_str(
        Rect::new(area.x, rule_y, active, 1),
        &"━".repeat(usize::from(active)),
        ink(ACCENT, CANVAS),
    );
}

/// The shared scrollbar geometry: the thumb covers the share of the track
/// the viewport shows. A viewport that fits its content keeps a full thumb.
fn shared_thumb(viewport: u16, content: u16, track: u16) -> (u16, u16) {
    let mut state = ScrollState::new(usize::from(content));
    state.set_viewport(usize::from(viewport));
    if !state.overflows() {
        return (0, track);
    }
    let (start, len) = state.thumb(usize::from(track));
    (
        u16::try_from(start).unwrap_or(u16::MAX),
        u16::try_from(len).unwrap_or(u16::MAX),
    )
}

/// A shared scrollbar cell: the glyph and foreground come from the
/// scrollbar theme, the background stays the page canvas.
fn scrollbar_cell(ui: &Ui<'_>, part: Part) -> (PaintStyle, &'static str) {
    let resolved = ui.resolve(
        Family::SCROLLBAR,
        Variant::DEFAULT,
        part,
        StateFlags::empty(),
    );
    let glyph = match resolved.glyph {
        Slot::Set(glyph) => ui.glyph_str(glyph),
        Slot::Inherit | Slot::Clear => {
            if part == Part::THUMB {
                "┃"
            } else {
                "┆"
            }
        }
    };
    let canvas = ui.theme_ref().bg(Surface::Canvas);
    (resolved.style.bg(canvas), glyph)
}

fn paint_scrollbar(ui: &mut Ui<'_>, bar: Rect, content_h: u16) {
    let (start, len) = shared_thumb(bar.height, content_h, bar.height);
    let (thumb_style, thumb_glyph) = scrollbar_cell(ui, Part::THUMB);
    let (track_style, track_glyph) = scrollbar_cell(ui, Part::TRACK);
    for row in 0..bar.height {
        let on_thumb = row >= start && row < start.saturating_add(len);
        let _ = ui.paint_str(
            Rect::new(bar.x, bar.y.saturating_add(row), 1, 1),
            if on_thumb { thumb_glyph } else { track_glyph },
            if on_thumb { thumb_style } else { track_style },
        );
    }
}

fn shown(area: Rect, y: u16) -> bool {
    y >= area.y && y < area.bottom()
}

fn paint_clip(ui: &mut Ui<'_>, clip: Rect, x: u16, y: u16, text: &str, style: PaintStyle) {
    if !shown(clip, y) || x >= clip.right() || text.is_empty() {
        return;
    }
    let _ = ui.paint_str(
        Rect::new(x, y, clip.right().saturating_sub(x), 1),
        text,
        style,
    );
}

fn paint_repeat(
    ui: &mut Ui<'_>,
    clip: Rect,
    x: u16,
    y: u16,
    count: u16,
    glyph: &str,
    style: PaintStyle,
) {
    if count == 0 {
        return;
    }
    paint_clip(ui, clip, x, y, &glyph.repeat(usize::from(count)), style);
}

/// The shared card panel paints the fill with its title and meta; the rule
/// below the head keeps the panel border token.
fn paint_card(ui: &mut Ui<'_>, clip: Rect, rect: Rect, id: Id, title: &str, meta: &str) {
    let area = rect.intersection(clip);
    if area.is_empty() {
        return;
    }
    Panel::new(id)
        .title(title)
        .meta(meta)
        .draw(ui, area, |_, _| {});
    let rule_y = rect.y.saturating_add(1);
    if shown(clip, rule_y) && rect.width.saturating_sub(4) > 0 {
        let plane = ui.theme_ref().raise(ui.surface());
        let rule = ui
            .resolve(
                Family::PANEL,
                Variant::DEFAULT,
                Part::BORDER,
                StateFlags::empty(),
            )
            .style
            .bg(ui.theme_ref().bg(plane));
        paint_repeat(
            ui,
            clip,
            rect.x.saturating_add(2),
            rule_y,
            rect.width.saturating_sub(4),
            "─",
            rule,
        );
    }
}

fn fill_color(kind: BarKind) -> u32 {
    match kind {
        BarKind::Active | BarKind::Sweep => ACCENT,
        BarKind::Done => SUCCESS,
        BarKind::Error => ERROR,
        BarKind::Paused => MUTED,
    }
}

fn suffix(kind: BarKind) -> &'static str {
    match kind {
        BarKind::Done => " ✓",
        BarKind::Error => " !",
        BarKind::Paused => " ‖",
        BarKind::Active | BarKind::Sweep => "  ",
    }
}

fn pct_field(ratio: f64) -> String {
    let pct = (ratio.clamp(0.0, 1.0) * 100.0).round() as u16;
    let body = format!("{pct}%");
    format!("{body:>4}")
}

/// The shared progress bar paints the label, the track, the percentage and
/// the state icon. Its thumb falls back to the track ink, so the demo
/// states pin their fills through theme roles: the bar has no
/// per-state fill token.
const BAR_ACTIVE_FILL: &[(Part, StylePatch)] =
    &[(Part::THUMB, StylePatch::new().set_fg(Role::Accent))];
const BAR_DONE_FILL: &[(Part, StylePatch)] =
    &[(Part::THUMB, StylePatch::new().set_fg(Role::Success))];
const BAR_PAUSED_FILL: &[(Part, StylePatch)] = &[(
    Part::THUMB,
    StylePatch::new().set_fg(Role::Fg(FgStep::Muted)),
)];
const BAR_ERROR_FILL: &[(Part, StylePatch)] =
    &[(Part::THUMB, StylePatch::new().set_fg(Role::Danger))];

fn paint_bar(ui: &mut Ui<'_>, clip: Rect, area: Rect, bar: BarSpec<'_>) {
    if area.height == 0 || area.width == 0 || !shown(clip, area.y) {
        return;
    }
    let mut widget = ProgressBar::new(bar.id).label(bar.label).frame(bar.frame);
    if let Some(ratio) = bar.ratio {
        widget = widget.ratio(ratio);
    }
    widget = match bar.kind {
        BarKind::Active | BarKind::Sweep => widget.patch_part(BAR_ACTIVE_FILL),
        BarKind::Done => widget.done(true).patch_part(BAR_DONE_FILL),
        BarKind::Error => widget.status(Status::Error).patch_part(BAR_ERROR_FILL),
        BarKind::Paused => widget
            .icon(GlyphRole::ProgressPaused)
            .patch_part(BAR_PAUSED_FILL),
    };
    widget.draw(ui, area);
}

/// One shared progress bar: which bar, its label, level, state and frame.
struct BarSpec<'a> {
    id: Id,
    label: &'a str,
    ratio: Option<f64>,
    kind: BarKind,
    frame: usize,
}

/// The shared button paints its own container, gutter and label; the old
/// hand-filled shim duplicated the secondary recipe cell for cell.
fn paint_button(ui: &mut Ui<'_>, button: Button<'_>, area: Rect, _label: &str) -> Rect {
    button.draw(ui, area)
}

fn row_at(card: Rect, dy: u16, x: u16, w: u16) -> Rect {
    Rect::new(x, card.y.saturating_add(dy), w, 1)
}

fn paint_live(ui: &mut Ui<'_>, clip: Rect, card: Rect, build: f64, paused: bool, frame: usize) {
    paint_card(ui, clip, card, PANEL_LIVE, "Live", "ticks at 80 ms");
    let inner_x = card.x.saturating_add(2);
    let inner_w = card.width.saturating_sub(4);
    if inner_w == 0 {
        return;
    }
    let bar_w = inner_w.min(BAR_W);
    let kind = if build >= 1.0 {
        BarKind::Done
    } else if paused {
        BarKind::Paused
    } else {
        BarKind::Active
    };
    paint_bar(
        ui,
        clip,
        row_at(card, 2, inner_x, bar_w),
        BarSpec {
            id: BAR_BUILD,
            label: "Building",
            ratio: Some(build),
            kind,
            frame,
        },
    );
    paint_bar(
        ui,
        clip,
        row_at(card, 4, inner_x, bar_w),
        BarSpec {
            id: BAR_SWEEP,
            label: "Resolving",
            ratio: None,
            kind: BarKind::Sweep,
            frame,
        },
    );
    let spin_y = card.y.saturating_add(6);
    if shown(clip, spin_y) {
        Spinner::new(SPINNER_LIVE)
            .label("Waiting for the test runner")
            .frame(frame)
            .draw(ui, Rect::new(inner_x, spin_y, inner_w, 1));
    }
    let button_y = card.y.saturating_add(8);
    if !shown(clip, button_y) {
        return;
    }
    let restart = Button::new(RESTART, "Restart").variant(Variant::SECONDARY);
    let pause_label = if paused { "Resume" } else { "Pause" };
    let pause = Button::new(PAUSE, pause_label).variant(Variant::SECONDARY);
    let row = Rect::new(inner_x, button_y, inner_w, 1);
    let first = paint_button(ui, restart, row, "Restart");
    let next_x = first.right().saturating_add(2);
    if next_x < row.right() {
        paint_button(
            ui,
            pause,
            Rect::new(next_x, button_y, row.right().saturating_sub(next_x), 1),
            pause_label,
        );
    }
}

fn paint_tally(ui: &mut Ui<'_>, clip: Rect, x: u16, y: u16) {
    let parts = [
        ("2 of 7 done", SECONDARY),
        (" · ", FAINT),
        ("3 running", SECONDARY),
        (" · ", FAINT),
        ("1 paused", SECONDARY),
        (" · ", FAINT),
        ("1 failed", ERROR),
    ];
    let mut cursor = x;
    for (text, color) in parts {
        paint_clip(ui, clip, cursor, y, text, ink(color, SURFACE));
        cursor = cursor.saturating_add(width(text));
    }
}

fn paint_jobs(ui: &mut Ui<'_>, clip: Rect, card: Rect, frame: usize) {
    paint_card(
        ui,
        clip,
        card,
        PANEL_JOBS,
        "Deploy checkout-service",
        "one BarLayout",
    );
    let inner_x = card.x.saturating_add(2);
    let inner_w = card.width.saturating_sub(4);
    paint_tally(ui, clip, inner_x, card.y.saturating_add(2));
    let bar_w = inner_w.min(BAR_W);
    for (index, job) in JOBS.iter().enumerate() {
        let Ok(step) = u16::try_from(index) else {
            break;
        };
        paint_bar(
            ui,
            clip,
            row_at(card, step.saturating_add(4), inner_x, bar_w),
            BarSpec {
                id: BAR_JOBS.index(index),
                label: job.label,
                ratio: job.ratio,
                kind: job.kind,
                frame,
            },
        );
    }
}

/// The shared capacity meter paints the track, the level fill and the
/// value. The value ink stays explicit: every row carries its own state
/// color, and the meter has no per-row value token.
struct LineMeter<'a> {
    id: Id,
    ratio: f64,
    value: &'a str,
    tone: Option<MeterTone>,
    value_role: Role,
    suffix: u16,
}

fn paint_line_meter(ui: &mut Ui<'_>, clip: Rect, area: Rect, meter: LineMeter<'_>) {
    if !shown(clip, area.y) || area.width == 0 {
        return;
    }
    let label: &[(Part, StylePatch)] = &[(Part::LABEL, StylePatch::new().set_fg(meter.value_role))];
    let mut widget = Meter::new(meter.id)
        .ratio(meter.ratio)
        .value(meter.value)
        .suffix_width(meter.suffix)
        .patch_part(label);
    if let Some(tone) = meter.tone {
        widget = widget.tone(tone);
    }
    widget.draw(ui, area);
}

/// The shared block meter paints the value inside the bar and washes the
/// filled part with the tone. The two trailing cells stay blank.
fn paint_block_meter(
    ui: &mut Ui<'_>,
    clip: Rect,
    area: Rect,
    id: Id,
    ratio: f64,
    value: &str,
    tone: Option<MeterTone>,
) {
    if !shown(clip, area.y) || area.width == 0 {
        return;
    }
    let mut meter = Meter::new(id)
        .ratio(ratio)
        .value(value)
        .visual(MeterVisual::Block)
        .suffix_width(2);
    if let Some(tone) = tone {
        meter = meter.tone(tone);
    }
    meter.draw(ui, area);
}

struct WideLine<'a> {
    pct: u8,
    glyph: &'static str,
    color: u32,
    value: &'a str,
    value_color: u32,
    value_mod: Modifier,
    trail: &'a str,
    trail_color: u32,
    trail_mod: Modifier,
}

fn paint_wide_line(ui: &mut Ui<'_>, clip: Rect, area: Rect, line: WideLine<'_>) {
    if !shown(clip, area.y) || area.width == 0 {
        return;
    }
    let value_w = width(line.value);
    let trail_w = width(line.trail);
    let track_w = area
        .width
        .saturating_sub(value_w)
        .saturating_sub(trail_w)
        .saturating_sub(1);
    if track_w < 6 {
        paint_clip(
            ui,
            clip,
            area.x,
            area.y,
            line.value,
            face(line.value_color, SURFACE, line.value_mod),
        );
        return;
    }
    let filled = (f64::from(track_w) * f64::from(line.pct) / 100.0).round() as u16;
    paint_repeat(
        ui,
        clip,
        area.x,
        area.y,
        track_w,
        "─",
        face(RULE, SURFACE, Modifier::empty()),
    );
    paint_repeat(
        ui,
        clip,
        area.x,
        area.y,
        filled.min(track_w),
        line.glyph,
        face(line.color, SURFACE, Modifier::empty()),
    );
    let text_x = area.x.saturating_add(track_w).saturating_add(1);
    paint_clip(
        ui,
        clip,
        text_x,
        area.y,
        line.value,
        face(line.value_color, SURFACE, line.value_mod),
    );
    if !line.trail.is_empty() {
        paint_clip(
            ui,
            clip,
            text_x.saturating_add(value_w),
            area.y,
            line.trail,
            face(line.trail_color, SURFACE, line.trail_mod),
        );
    }
}

fn paint_wide_meters(ui: &mut Ui<'_>, clip: Rect, card: Rect, frame: usize) {
    let inner_x = card.x.saturating_add(2);
    let inner_w = card.width.saturating_sub(4);
    let col_w = inner_w.saturating_sub(12) / 2;
    let meter_w = col_w.min(METER_W);
    if meter_w == 0 {
        return;
    }
    let left_x = inner_x.saturating_add(11);
    let right_x = left_x.saturating_add(col_w).saturating_add(1);
    let plain = Modifier::empty();
    let row = |dy: u16| Rect::new(left_x, card.y.saturating_add(dy), meter_w, 1);
    let right = |dy: u16| Rect::new(right_x, card.y.saturating_add(dy), meter_w, 1);
    for (dy, label) in [
        (6, "Medium     "),
        (7, "High       "),
        (8, "Warning    "),
        (9, "Exhausted  "),
        (10, "Stale      "),
        (11, "Refreshing "),
        (12, "Error      "),
        (13, "Unknown    "),
    ] {
        paint_clip(
            ui,
            clip,
            inner_x,
            card.y.saturating_add(dy),
            label,
            face(WHITE, SURFACE, plain),
        );
    }
    paint_line_meter(
        ui,
        clip,
        row(6),
        LineMeter {
            id: METER_WIDE.index(0),
            ratio: 0.72,
            value: "72% used ▲",
            tone: None,
            value_role: Role::Warning,
            suffix: 0,
        },
    );
    if right_x < card.right() {
        paint_line_meter(
            ui,
            clip,
            right(6),
            LineMeter {
                id: METER_WIDE.index(1),
                ratio: 0.72,
                value: "72% disk, warns at 80",
                tone: None,
                value_role: Role::Fg(FgStep::Secondary),
                suffix: 2,
            },
        );
    }
    paint_line_meter(
        ui,
        clip,
        row(7),
        LineMeter {
            id: METER_WIDE.index(2),
            ratio: 0.91,
            value: "91% used !",
            tone: None,
            value_role: Role::Danger,
            suffix: 0,
        },
    );
    paint_line_meter(
        ui,
        clip,
        right(7),
        LineMeter {
            id: METER_WIDE.index(3),
            ratio: 0.88,
            value: "88% disk, critical at 95 ▲",
            // Below the critical mark, so the medium tone, not the ratio's high.
            tone: Some(MeterTone::Medium),
            value_role: Role::Warning,
            suffix: 0,
        },
    );
    paint_line_meter(
        ui,
        clip,
        row(8),
        LineMeter {
            id: METER_WIDE.index(4),
            ratio: 0.82,
            value: "82% used ▲",
            tone: None,
            value_role: Role::Warning,
            suffix: 0,
        },
    );
    paint_line_meter(
        ui,
        clip,
        right(8),
        LineMeter {
            id: METER_WIDE.index(5),
            ratio: 0.30,
            value: "30% · fixed high !",
            // The fixed-high override keeps the high tone despite the low level.
            tone: Some(MeterTone::High),
            value_role: Role::Danger,
            suffix: 0,
        },
    );
    paint_line_meter(
        ui,
        clip,
        row(9),
        LineMeter {
            id: METER_WIDE.index(6),
            ratio: 1.0,
            value: "100% used !",
            tone: None,
            value_role: Role::Danger,
            suffix: 0,
        },
    );
    paint_wide_line(
        ui,
        clip,
        right(9),
        WideLine {
            pct: 82,
            glyph: "╍",
            color: WARNING,
            value: "82% · stale warning",
            value_color: MUTED,
            value_mod: Modifier::ITALIC,
            trail: " ▲",
            trail_color: WARNING,
            trail_mod: plain,
        },
    );
    paint_wide_line(
        ui,
        clip,
        row(10),
        WideLine {
            pct: 76,
            glyph: "╍",
            color: WARNING,
            value: "76% used",
            value_color: MUTED,
            value_mod: Modifier::ITALIC,
            trail: " ▲",
            trail_color: WARNING,
            trail_mod: plain,
        },
    );
    paint_line_meter(
        ui,
        clip,
        right(10),
        LineMeter {
            id: METER_WIDE.index(7),
            ratio: 0.76,
            value: "76% · faint stale style",
            // The faint stale style keeps the unknown tone despite the level.
            tone: Some(MeterTone::Unknown),
            value_role: Role::Fg(FgStep::Muted),
            suffix: 2,
        },
    );
    let frames = ui.design().motion.spinner_frames;
    let spin = frames
        .get(frame.checked_rem(frames.len()).unwrap_or(0))
        .copied()
        .unwrap_or("");
    let refreshing = format!("{spin} refreshing  ");
    paint_line_meter(
        ui,
        clip,
        row(11),
        LineMeter {
            id: METER_WIDE.index(8),
            ratio: 0.54,
            value: &refreshing,
            tone: None,
            value_role: Role::Fg(FgStep::Muted),
            suffix: 0,
        },
    );
    let error_y = card.y.saturating_add(12);
    if shown(clip, error_y) {
        Meter::new(METER_WIDE.index(9))
            .value("quota read failed")
            .status(Status::Error)
            .patch_part(&[(Part::LABEL, StylePatch::new().set_fg(Role::Danger))])
            .draw(
                ui,
                Rect::new(left_x, error_y, card.right().saturating_sub(left_x), 1),
            );
    }
    let unknown_y = card.y.saturating_add(13);
    if shown(clip, unknown_y) {
        Meter::new(METER_WIDE.index(10))
            .value("—  ")
            .tone(MeterTone::Unknown)
            .patch_part(&[(
                Part::LABEL,
                StylePatch::new().set_fg(Role::Fg(FgStep::Faint)),
            )])
            .draw(
                ui,
                Rect::new(left_x, unknown_y, card.right().saturating_sub(left_x), 1),
            );
    }
}

fn paint_meters(ui: &mut Ui<'_>, clip: Rect, card: Rect) {
    paint_card(
        ui,
        clip,
        card,
        PANEL_METERS,
        "Meters",
        "capacity, not completion",
    );
    let inner_x = card.x.saturating_add(2);
    let inner_w = card.width.saturating_sub(4);
    for (index, note) in NOTES.iter().enumerate() {
        let Ok(step) = u16::try_from(index) else {
            break;
        };
        paint_clip(
            ui,
            clip,
            inner_x,
            card.y.saturating_add(step.saturating_add(2)),
            note,
            ink(MUTED, SURFACE),
        );
    }
    let y = card.y.saturating_add(5);
    paint_clip(ui, clip, inner_x, y, "Low       ", ink(WHITE, SURFACE));
    let col_w = inner_w.saturating_sub(12) / 2;
    let meter_w = col_w.min(METER_W);
    if meter_w == 0 {
        return;
    }
    let meter_x = inner_x.saturating_add(11);
    paint_line_meter(
        ui,
        clip,
        Rect::new(meter_x, y, meter_w, 1),
        LineMeter {
            id: METER_LOW_LINE,
            ratio: 0.38,
            value: "38% used",
            tone: None,
            value_role: Role::Fg(FgStep::Secondary),
            suffix: 2,
        },
    );
    let block_x = meter_x.saturating_add(col_w).saturating_add(1);
    if block_x < card.right() {
        paint_block_meter(
            ui,
            clip,
            Rect::new(block_x, y, meter_w, 1),
            METER_LOW_BLOCK,
            0.38,
            "38% used · block",
            None,
        );
    }
}

fn paint_narrow_blurb(ui: &mut Ui<'_>, body: Rect) {
    let title_w = width("Progress");
    let cols = body.width.saturating_sub(title_w).saturating_sub(3);
    if cols == 0 {
        return;
    }
    let _ = ui.paint_str(
        Rect::new(
            body.x.saturating_add(title_w).saturating_add(2),
            body.y.saturating_sub(2),
            cols,
            1,
        ),
        &termrock::truncate(BLURB, cols),
        settled(MUTED, CANVAS, false),
    );
}

fn paint_narrow_card(ui: &mut Ui<'_>, card: Rect, title: &str) {
    if card.is_empty() {
        return;
    }
    ui.fill(card, settled(WHITE, SURFACE, false));
    let span = card.width.saturating_sub(2);
    if span == 0 {
        return;
    }
    let x = card.x.saturating_add(2);
    let _ = ui.paint_str(
        Rect::new(x, card.y, span, 1),
        title,
        settled(SECONDARY, SURFACE, false),
    );
    if card.height < 2 {
        return;
    }
    let _ = ui.paint_str(
        Rect::new(x, card.y.saturating_add(1), span, 1),
        &"─".repeat(usize::from(span)),
        settled(RULE, SURFACE, false),
    );
}

fn paint_narrow_bar(
    ui: &mut Ui<'_>,
    x: u16,
    y: u16,
    track_w: u16,
    ratio: Option<f64>,
    kind: BarKind,
    frame: usize,
) {
    if track_w == 0 {
        return;
    }
    let _ = ui.paint_str(
        Rect::new(x, y, track_w, 1),
        &"─".repeat(usize::from(track_w)),
        settled(RULE, SURFACE, false),
    );
    let (from, to, color) = if let Some(ratio) = ratio {
        let filled = (f64::from(track_w) * ratio.clamp(0.0, 1.0)).round() as u16;
        (0, filled.min(track_w), fill_color(kind))
    } else {
        let track_i = i32::from(track_w);
        let seg = (track_i / 5).clamp(2, 8);
        let period = usize::try_from(track_i.saturating_add(seg))
            .unwrap_or(1)
            .max(1);
        let pos = i32::try_from(frame % period)
            .unwrap_or(0)
            .saturating_sub(seg);
        let mut from = 0u16;
        let mut to = 0u16;
        let mut started = false;
        for i in 0..track_i {
            let on = i >= pos && i < pos.saturating_add(seg);
            if on && !started {
                from = u16::try_from(i).unwrap_or(track_w);
                started = true;
            }
            if on {
                to = u16::try_from(i.saturating_add(1)).unwrap_or(track_w);
            }
        }
        (from, to, ACCENT)
    };
    if to > from {
        let len = to.saturating_sub(from);
        let _ = ui.paint_str(
            Rect::new(x.saturating_add(from), y, len, 1),
            &"━".repeat(usize::from(len)),
            settled(color, SURFACE, false),
        );
    }
}

fn paint_narrow_label_bar(
    ui: &mut Ui<'_>,
    card: Rect,
    dy: u16,
    label: &str,
    ratio: Option<f64>,
    kind: BarKind,
    frame: usize,
) {
    let y = card.y.saturating_add(dy);
    let x = card.x.saturating_add(2);
    let field = LABEL_W.saturating_add(2);
    let _ = ui.paint_str(
        Rect::new(x, y, LABEL_W, 1),
        label,
        settled(WHITE, SURFACE, false),
    );
    let track_x = x.saturating_add(field);
    let avail = card.right().saturating_sub(track_x);
    if let Some(ratio) = ratio
        && card.width >= 54
    {
        let pct_str = format!(" {}", pct_field(ratio));
        let pct_w = u16::try_from(pct_str.chars().count()).unwrap_or(0);
        let suf = suffix(kind);
        let suf_w = u16::try_from(suf.chars().count()).unwrap_or(0);
        let text_w = pct_w.saturating_add(suf_w);
        if avail > text_w {
            let bar_w = avail.saturating_sub(text_w);
            paint_narrow_bar(ui, track_x, y, bar_w, Some(ratio), kind, frame);
            let _ = ui.paint_str(
                Rect::new(track_x.saturating_add(bar_w), y, pct_w, 1),
                &pct_str,
                settled(SECONDARY, SURFACE, false),
            );
            if suf_w > 0 {
                let _ = ui.paint_str(
                    Rect::new(
                        track_x.saturating_add(bar_w).saturating_add(pct_w),
                        y,
                        suf_w,
                        1,
                    ),
                    suf,
                    settled(fill_color(kind), SURFACE, false),
                );
            }
        } else {
            paint_narrow_bar(ui, track_x, y, avail, Some(ratio), kind, frame);
        }
    } else {
        paint_narrow_bar(ui, track_x, y, avail, None, kind, frame);
    }
}

fn paint_narrow_button(ui: &mut Ui<'_>, button: Button<'_>, area: Rect, _label: &str) -> Rect {
    button.draw(ui, area)
}

fn paint_narrow_scroll(ui: &mut Ui<'_>, view: Rect) {
    let bar_h = view.height.saturating_sub(1);
    let (start, len) = shared_thumb(bar_h, CONTENT_H, bar_h);
    let (thumb_style, thumb_glyph) = scrollbar_cell(ui, Part::THUMB);
    let (track_style, track_glyph) = scrollbar_cell(ui, Part::TRACK);
    let x = view.right().saturating_sub(1);
    for row in 0..bar_h {
        let on = row >= start && row < start.saturating_add(len);
        let _ = ui.paint_str(
            Rect::new(x, view.y.saturating_add(row), 1, 1),
            if on { thumb_glyph } else { track_glyph },
            if on { thumb_style } else { track_style },
        );
    }
    let track = view.width.saturating_sub(1);
    let (_, filled) = shared_thumb(track, NARROW_CONTENT_W, track);
    let filled = filled.min(track);
    let y = view.bottom().saturating_sub(1);
    if filled > 0 {
        let _ = ui.paint_str(
            Rect::new(view.x, y, filled, 1),
            &"━".repeat(usize::from(filled)),
            thumb_style,
        );
    }
    let rest = track.saturating_sub(filled);
    if rest > 0 {
        let _ = ui.paint_str(
            Rect::new(view.x.saturating_add(filled), y, rest, 1),
            &"┄".repeat(usize::from(rest)),
            track_style,
        );
    }
}

fn paint_narrow(ui: &mut Ui<'_>, body: Rect, build: f64, paused: bool, frame: usize) {
    paint_narrow_blurb(ui, body);
    paint_section_strip(
        ui,
        Rect {
            height: 2.min(body.height),
            ..body
        },
    );
    let top = body.y.saturating_add(3);
    if top >= body.bottom() {
        return;
    }
    let view = Rect {
        y: top,
        height: body.bottom().saturating_sub(top),
        ..body
    };
    if view.height < 2 || view.width < 3 {
        return;
    }
    let cards = Rect {
        width: view.width.saturating_sub(2),
        ..view
    };
    let live = Rect {
        height: LIVE_H.min(view.height.saturating_sub(1)),
        ..cards
    };
    paint_narrow_card(ui, live, "Live");
    let kind = if build >= 1.0 {
        BarKind::Done
    } else if paused {
        BarKind::Paused
    } else {
        BarKind::Active
    };
    paint_narrow_label_bar(ui, live, 2, "Building", Some(build), kind, frame);
    paint_narrow_label_bar(ui, live, 4, "Resolving", None, BarKind::Sweep, frame);
    let spin_y = live.y.saturating_add(6);
    if spin_y < live.bottom() {
        let x = live.x.saturating_add(2);
        let _ = ui.paint_str(
            Rect::new(x, spin_y, 1, 1),
            SPINNER[frame % SPINNER.len()],
            settled(ACCENT, SURFACE, false),
        );
        let _ = ui.paint_str(
            Rect::new(
                x.saturating_add(2),
                spin_y,
                live.right().saturating_sub(x).saturating_sub(2),
                1,
            ),
            "Waiting for the test runner",
            settled(SECONDARY, SURFACE, false),
        );
    }
    let button_y = live.y.saturating_add(8);
    if button_y < live.bottom() {
        let pause_label = if paused { "Resume" } else { "Pause" };
        let row = Rect::new(
            live.x.saturating_add(2),
            button_y,
            live.width.saturating_sub(2),
            1,
        );
        let first = paint_narrow_button(ui, restart_button(), row, "Restart");
        let next_x = first.right().saturating_add(2);
        if next_x < row.right() {
            paint_narrow_button(
                ui,
                pause_button(paused),
                Rect::new(next_x, button_y, row.right().saturating_sub(next_x), 1),
                pause_label,
            );
        }
    }
    let jobs_y = cards.y.saturating_add(LIVE_H).saturating_add(1);
    let jobs_h = view.bottom().saturating_sub(1).saturating_sub(jobs_y);
    if jobs_y < view.bottom().saturating_sub(1) && jobs_h > 0 {
        paint_narrow_card(
            ui,
            Rect::new(cards.x, jobs_y, cards.width, jobs_h.min(2)),
            "Deploy checkout-service",
        );
    }
    paint_narrow_scroll(ui, view);
}

fn restart_button() -> Button<'static> {
    Button::new(RESTART, "Restart").variant(Variant::SECONDARY)
}

fn pause_button(paused: bool) -> Button<'static> {
    Button::new(PAUSE, if paused { "Resume" } else { "Pause" }).variant(Variant::SECONDARY)
}

#[derive(Debug)]
pub struct ProgressPage {
    frame: usize,
    build: f64,
    paused: bool,
    next_tick: Option<termrock::Moment>,
}

impl ProgressPage {
    pub fn new() -> Self {
        Self {
            frame: 0,
            build: 0.0,
            paused: false,
            next_tick: None,
        }
    }
}

impl Default for ProgressPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for ProgressPage {
    fn seek_paused(&mut self, frame: usize) {
        self.frame = frame;
        self.build = ((frame as f64) * 0.006).min(1.0);
        self.next_tick = None;
    }

    fn title(&self) -> &'static str {
        "Progress"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let mut response = Response::ignored();
        let mut status = None;
        let restart = restart_button().update(cx);
        if restart.activated() {
            self.build = 0.0;
        }
        response |= restart.erase();
        let pause = pause_button(self.paused).update(cx);
        if pause.activated() {
            self.paused = !self.paused;
        }
        response |= pause.erase();

        let now = cx.now();
        let interval = Duration::from_millis(80);
        let deadline = *self
            .next_tick
            .get_or_insert_with(|| now.saturating_add(interval));
        if cx.update_cause() == termrock::UpdateCause::Tick && now >= deadline {
            if !self.paused && self.build < 1.0 {
                self.build = (self.build + 0.006).min(1.0);
                if self.build >= 1.0 {
                    status = Some(PageStatus("Build finished ✓".to_owned()));
                }
            }
            self.frame = self.frame.wrapping_add(1);
            self.next_tick = Some(now.saturating_add(interval));
            response = response.repaint();
        }
        if cx.top_layer() == termrock::LayerId::PAGE
            && let Some(deadline) = self.next_tick
        {
            cx.request_repaint_at(deadline);
        }
        PageUpdate { response, status }
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(ui, area, self.title(), "", |ui, body| {
            // Shells up to 80×24 show the compact strip.
            if compact_page(area) {
                paint_narrow(ui, body, self.build, self.paused, self.frame);
                return;
            }
            paint_blurb(ui, body);
            paint_section_strip(
                ui,
                Rect {
                    height: 2.min(body.height),
                    ..body
                },
            );
            let top = body.y.saturating_add(3);
            if top >= body.bottom() {
                return;
            }
            let view = Rect {
                y: top,
                height: body.bottom().saturating_sub(top),
                ..body
            };
            let scrolling = CONTENT_H > view.height;
            let cards = Rect {
                width: if scrolling {
                    view.width.saturating_sub(2)
                } else {
                    view.width
                },
                ..view
            };
            if scrolling {
                paint_scrollbar(
                    ui,
                    Rect::new(view.right().saturating_sub(1), view.y, 1, view.height),
                    CONTENT_H,
                );
            }
            paint_live(
                ui,
                cards,
                Rect {
                    height: LIVE_H,
                    ..cards
                },
                self.build,
                self.paused,
                self.frame,
            );
            let jobs_y = cards.y.saturating_add(LIVE_H).saturating_add(1);
            paint_jobs(
                ui,
                cards,
                Rect::new(cards.x, jobs_y, cards.width, JOBS_H),
                self.frame,
            );
            let meters_y = jobs_y.saturating_add(JOBS_H).saturating_add(1);
            paint_meters(
                ui,
                cards,
                Rect::new(cards.x, meters_y, cards.width, METERS_H),
            );
            // The wide meter rows need the 160×50 shell width.
            if wide_page(area) {
                paint_wide_meters(
                    ui,
                    cards,
                    Rect::new(cards.x, meters_y, cards.width, METERS_H),
                    self.frame,
                );
            }
        });
    }

    fn hints(&self, _ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        &[("Enter", "Activate")]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use termrock::{App, Theme};

    struct Host;

    impl App for Host {
        fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            ProgressPage::new().draw(ui, ui.full());
        }
    }

    #[test]
    fn paused_seek_is_deterministic_and_caps_completed_progress() {
        let mut page = ProgressPage::new();
        page.seek_paused(80);
        assert_eq!(page.frame, 80);
        assert!((page.build - 0.48).abs() < f64::EPSILON);
        page.seek_paused(usize::MAX);
        assert!((page.build - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn playground_at_120_matches_the_progress_copy() {
        let h = termrock_test_support::Harness::new(Host, Theme::junie(), 94, 36);
        let text = h.text();
        for needle in [
            BLURB,
            "Playground",
            "the live widget",
            "ticks at 80 ms",
            "Waiting for the test runner",
            "Deploy checkout-service",
            "one BarLayout",
            "2 of 7 done",
            "1 failed",
            "Build image",
            "Warm caches",
            "capacity, not completion",
            "Stale keeps its level, dashed.",
            "38% used · block",
        ] {
            assert!(text.contains(needle), "missing {needle}\n{text}");
        }
        assert!(!text.contains("3 of 12 files"), "{text}");
        assert!(!text.contains("Queued"), "{text}");

        let track = h.cell(20, 7);
        assert_eq!(track.symbol(), "─");
        assert_eq!(track.fg, Color::Rgb(38, 38, 38));
        assert_eq!(track.bg, rgb(SURFACE));

        let done = h.cell(20, 21);
        assert_eq!(done.symbol(), "━");
        assert_eq!(done.fg, rgb(SUCCESS));

        let low = h.cell(13, 35);
        assert_eq!(low.symbol(), "━");
        assert_eq!(low.fg, Color::Rgb(179, 179, 179));
        assert_eq!(low.bg, rgb(SURFACE));

        let block = h.cell(53, 35);
        assert_eq!(block.symbol(), "3");
        assert_eq!(block.fg, Color::Rgb(25, 25, 28));
        assert_eq!(block.bg, Color::Rgb(179, 179, 179));
        assert!(!block.modifier.contains(Modifier::BOLD));

        let thumb = h.cell(93, 5);
        assert_eq!(thumb.symbol(), "┃");
        assert_eq!(thumb.fg, Color::Rgb(128, 128, 128));
        let track_bar = h.cell(93, 29);
        assert_eq!(track_bar.symbol(), "┆");
        assert_eq!(track_bar.fg, Color::Rgb(71, 71, 71));

        // The shared button keeps its focus bar: the harness focuses the
        // first control, and the old fill shim used to paint over it.
        let gutter = h.cell(2, 13);
        assert_eq!(gutter.symbol(), "▎");
        assert_eq!(gutter.fg, rgb(ACCENT));
        assert_eq!(gutter.bg, Color::Rgb(39, 39, 42));
    }

    #[test]
    fn narrow_page_matches_the_80x24_progress_strip() {
        let h = termrock_test_support::Harness::new(Host, Theme::junie(), 59, 20);
        let text = h.text();
        assert!(
            text.contains("Determinate, indeterminate, an aggregate tally,…"),
            "{text}"
        );
        assert!(!text.contains("ticks at 80 ms"), "{text}");
        assert!(!text.contains("one BarLayout"), "{text}");
        assert!(!text.contains("38% used"), "{text}");
        let track = h.cell(20, 7);
        assert_eq!(track.symbol(), "─");
        assert_eq!(track.fg, rgb(RULE));
        assert_eq!(track.bg, rgb(SURFACE));
        assert!(!track.modifier.contains(Modifier::BOLD));
        let thumb = h.cell(58, 5);
        assert_eq!(thumb.symbol(), "┃");
        assert_eq!(thumb.fg, Color::Rgb(128, 128, 128));
        assert_eq!(thumb.bg, rgb(CANVAS));
        let track_bar = h.cell(58, 9);
        assert_eq!(track_bar.symbol(), "┆");
        assert_eq!(track_bar.fg, Color::Rgb(71, 71, 71));
        assert_eq!(track_bar.bg, rgb(CANVAS));
        let hbar = h.cell(0, 19);
        assert_eq!(hbar.symbol(), "━");
        assert_eq!(hbar.fg, Color::Rgb(128, 128, 128));
        assert_eq!(hbar.bg, rgb(CANVAS));
        let htrack = h.cell(35, 19);
        assert_eq!(htrack.symbol(), "┄");
        assert_eq!(htrack.fg, Color::Rgb(71, 71, 71));
        let corner = h.cell(58, 19);
        assert_eq!(corner.symbol(), " ");
        assert_eq!(corner.bg, rgb(CANVAS));
    }

    #[test]
    fn compact_page_matches_the_72x20_progress_strip() {
        let h = termrock_test_support::Harness::new(Host, Theme::junie(), 51, 16);
        let text = h.text();
        assert!(text.contains("an aggregat…"), "{text}");
        assert!(!text.contains("ticks at 80 ms"), "{text}");
        assert!(!text.contains("0%"), "{text}");
        let ellipsis = h.cell(49, 0);
        assert_eq!(ellipsis.symbol(), "…");
        assert_eq!(ellipsis.fg, rgb(MUTED));
        assert_eq!(ellipsis.bg, rgb(CANVAS));
        assert!(!ellipsis.modifier.contains(Modifier::BOLD));
        let rule = h.cell(48, 6);
        assert_eq!(rule.symbol(), "─");
        assert_eq!(rule.fg, rgb(RULE));
        assert_eq!(rule.bg, rgb(SURFACE));
        let track = h.cell(48, 7);
        assert_eq!(track.symbol(), "─");
        assert_eq!(track.fg, rgb(RULE));
        assert_eq!(track.bg, rgb(SURFACE));
        assert!(!track.modifier.contains(Modifier::BOLD));
        let thumb = h.cell(50, 5);
        assert_eq!(thumb.symbol(), "┃");
        assert_eq!(thumb.fg, Color::Rgb(128, 128, 128));
        assert_eq!(thumb.bg, rgb(CANVAS));
        let track_bar = h.cell(50, 7);
        assert_eq!(track_bar.symbol(), "┆");
        assert_eq!(track_bar.fg, Color::Rgb(71, 71, 71));
        assert_eq!(track_bar.bg, rgb(CANVAS));
        let hbar = h.cell(0, 15);
        assert_eq!(hbar.symbol(), "━");
        assert_eq!(hbar.fg, Color::Rgb(128, 128, 128));
        assert_eq!(hbar.bg, rgb(CANVAS));
        let htrack = h.cell(26, 15);
        assert_eq!(htrack.symbol(), "┄");
        assert_eq!(htrack.fg, Color::Rgb(71, 71, 71));
        assert_eq!(htrack.bg, rgb(CANVAS));
        let corner = h.cell(50, 15);
        assert_eq!(corner.symbol(), " ");
        assert_eq!(corner.bg, rgb(CANVAS));
    }

    #[test]
    fn wide_page_matches_the_160x50_meter_rows() {
        let h = termrock_test_support::Harness::new(Host, Theme::junie(), 134, 46);
        let text = h.text();
        for needle in [
            "72% used ▲",
            "72% disk, warns at 80",
            "88% disk, critical at 95 ▲",
            "30% · fixed high !",
            "82% · stale warning",
            "76% · faint stale style",
            "⠋ refreshing",
            "quota read failed !",
        ] {
            assert!(text.contains(needle), "missing {needle}\n{text}");
        }
        let medium = h.cell(13, 36);
        assert_eq!(medium.symbol(), "━");
        assert_eq!(medium.fg, rgb(WARNING));
        assert_eq!(medium.bg, rgb(SURFACE));
        let stale = h.cell(37, 40);
        assert_eq!(stale.symbol(), "7");
        assert_eq!(stale.fg, rgb(MUTED));
        assert!(stale.modifier.contains(Modifier::ITALIC));
        assert!(!stale.modifier.contains(Modifier::BOLD));
        let bang = h.cell(46, 39);
        assert_eq!(bang.symbol(), "!");
        assert_eq!(bang.fg, rgb(ERROR));
        assert!(!bang.modifier.contains(Modifier::BOLD));
        let unknown = h.cell(13, 43);
        assert_eq!(unknown.symbol(), "—");
        assert_eq!(unknown.fg, rgb(FAINT));
        assert_eq!(unknown.bg, rgb(SURFACE));
    }
}
