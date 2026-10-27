use std::cell::Cell;
use std::time::Duration;

use termrock::navigation::TabAffixes;
use termrock::{
    ActionKey, Binding, Button, Chord, Color, Cx, FrameRead, GlyphRole, Id, ItemKey, Meter,
    MeterTone, MeterVisual, Modifier, PaintStyle, Part, PartRef, ProgressBar, Rect, Response, Role,
    ScrollRegion, ScrollState, Select, SelectState, Spinner, StateFlags, Status, StylePatch,
    Surface, Tabs, TabsState, Ui, Variant, author::VerticalWindow, truncate, width,
};

use super::{Page, PageStatus, PageUpdate, frame, paint_boundary_meta};

const RESTART: Id = termrock::Id::root("showcase_ui::pages::progress::progress.restart");
const PAUSE: Id = termrock::Id::root("showcase_ui::pages::progress::progress.pause");
const PAGE_TABS: Id = Id::root("showcase_ui::pages::progress.tabs");
const PAGE_SCROLL_UP: ActionKey = ActionKey::application("showcase.progress.page_scroll_up");
const PAGE_SCROLL_DOWN: ActionKey = ActionKey::application("showcase.progress.page_scroll_down");
const PAGE_SCROLL_BINDINGS: &[Binding<()>] = &[
    Binding {
        action: PAGE_SCROLL_UP,
        chord: Some(Chord::key(termrock::KeyCode::PageUp)),
        cmd: (),
        label: "Scroll up",
        priority: 70,
        visible: true,
    },
    Binding {
        action: PAGE_SCROLL_DOWN,
        chord: Some(Chord::key(termrock::KeyCode::PageDown)),
        cmd: (),
        label: "Scroll down",
        priority: 70,
        visible: true,
    },
];
const PLAY_SCROLL: Id = Id::root("showcase_ui::pages::progress.play_scroll");
const OPTIONS_SCROLL: Id = Id::root("showcase_ui::pages::progress.options_scroll");
const STATES_SCROLL: Id = Id::root("showcase_ui::pages::progress.states_scroll");
const LABEL_WIDTH_SELECT: Id = Id::root("showcase_ui::pages::progress.options.label_width");
const SWEEP_RESERVE_SELECT: Id = Id::root("showcase_ui::pages::progress.options.sweep_value");
const METER_VISUAL_SELECT: Id = Id::root("showcase_ui::pages::progress.options.meter_visual");
const STALE_STYLE_SELECT: Id = Id::root("showcase_ui::pages::progress.options.stale_style");
const PB_QUEUED: Id = Id::root("showcase_ui::pages::progress.states.progress.queued");
const PB_HALF: Id = Id::root("showcase_ui::pages::progress.states.progress.half");
const PB_COMPLETE: Id = Id::root("showcase_ui::pages::progress.states.progress.complete");
const PB_ERROR: Id = Id::root("showcase_ui::pages::progress.states.progress.error");
const PB_PAUSED: Id = Id::root("showcase_ui::pages::progress.states.progress.paused");
const PB_SWEEP: Id = Id::root("showcase_ui::pages::progress.states.progress.sweep");
const SPINNER_FILES: Id = Id::root("showcase_ui::pages::progress.states.spinner.files");
const METER_CPU: Id = Id::root("showcase_ui::pages::progress.states.meter.cpu");
const METER_LATENCY: Id = Id::root("showcase_ui::pages::progress.states.meter.latency");
const METER_SYNC: Id = Id::root("showcase_ui::pages::progress.states.meter.sync");
const METER_UNKNOWN: Id = Id::root("showcase_ui::pages::progress.states.meter.unknown");
const METER_ZERO: Id = Id::root("showcase_ui::pages::progress.states.meter.zero");
const METER_FULL: Id = Id::root("showcase_ui::pages::progress.states.meter.full");
const METER_REFRESH: Id = Id::root("showcase_ui::pages::progress.states.meter.refresh");
const METER_FAILED: Id = Id::root("showcase_ui::pages::progress.states.meter.failed");
const METER_NARROW: Id = Id::root("showcase_ui::pages::progress.states.meter.narrow");
const METER_LONG: Id = Id::root("showcase_ui::pages::progress.states.meter.long");

const SECTIONS: [&str; 3] = ["Playground", "Options", "States"];
const LABEL_WIDTHS: &[&str] = &["16 cells", "Measured", "10 cells"];
const SWEEP_VALUE_COLUMNS: &[&str] = &["Reserve", "Use full track"];
const METER_VISUALS: &[&str] = &["Line", "Block"];
const STALE_STYLES: &[&str] = &["Dashed", "Faint"];
const OPTIONS_ROWS: usize = 6;
const STATES_ROWS: usize = 25;
const NARROW_METER_W: u16 = 11;
const PLAY_FOCUS_ROWS: &[(Id, usize)] = &[(RESTART, 8), (PAUSE, 8)];
const OPTION_FOCUS_ROWS: &[(Id, usize)] = &[
    (LABEL_WIDTH_SELECT, 1),
    (SWEEP_RESERVE_SELECT, 2),
    (METER_VISUAL_SELECT, 3),
    (STALE_STYLE_SELECT, 4),
];
const TAB_HINTS: &[(&str, &str)] = &[
    ("← →", "Section"),
    ("1–3", "Jump"),
    ("Enter", "Activate"),
    ("PgUp PgDn", "Scroll"),
];
const OPTION_HINTS: &[(&str, &str)] = &[
    ("↑ ↓", "Choose"),
    ("Enter", "Open / choose"),
    ("Wheel", "Scroll"),
];
const SCROLL_HINTS: &[(&str, &str)] = &[("Wheel", "Scroll")];
const SECTION_CAPTION: &str = "the live widget";
const SECTION_WIDE: u16 = 76;
const BLURB: &str = "Determinate, indeterminate, an aggregate tally, capacity meters";

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
const INFO: u32 = 0x79b8ff;
const TAB_ACTIVE: u32 = 0x27272a;
const BUTTON_BG: u32 = 0x27272a;
const PROGRESS_BUTTON_IDLE: StylePatch = StylePatch::new()
    .set_fg(Role::Custom(Color::from_u32(WHITE)))
    .set_bg(Role::Custom(Color::from_u32(BUTTON_BG)));
const PROGRESS_BUTTON_HOVER: &[(Part, StylePatch)] = &[(
    Part::CONTAINER,
    StylePatch::new().set_bg(Role::Surface(Surface::Popover)),
)];
const PROGRESS_TABS_CONTAINER: &[(Part, StylePatch)] = &[(
    Part::CONTAINER,
    StylePatch::new()
        .set_fg(Role::Custom(Color::from_u32(CANVAS)))
        .set_bg(Role::Custom(Color::from_u32(CANVAS))),
)];
const THUMB: u32 = 0x808080;
const TRACK: u32 = 0x474747;
const BLOCK_REST: u32 = 0x26262a;

const LABEL_W: u16 = 16;
const BAR_W: u16 = 70;
const VALUE_W: u16 = 7;
const METER_W: u16 = 34;
const LIVE_H: u16 = 11;
const JOBS_H: u16 = 12;
const METERS_H: u16 = 15;
const CONTENT_H: u16 = LIVE_H + 1 + JOBS_H + 1 + METERS_H;

const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

const NOTES: [&str; 2] = [
    "Capacity meters are never green: low ≤ 59 % blue, medium ≤ 84 % warning ▲, high error !.",
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

#[derive(Clone, Copy)]
struct BarOptions {
    frame: usize,
    label_width: usize,
    reserve_sweep_value: bool,
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

fn section_tab_affixes(_section: &&str) -> TabAffixes {
    TabAffixes::default().trailing_padding(1)
}

fn focus_row(owner: Id, rows: &[(Id, usize)]) -> Option<usize> {
    rows.iter()
        .find_map(|(candidate, row)| (*candidate == owner).then_some(*row))
}

fn focused_owner(ui: &Ui<'_>, rows: &[(Id, usize)]) -> Option<Id> {
    rows.iter()
        .map(|(owner, _)| *owner)
        .find(|owner| ui.state(*owner).contains(termrock::StateFlags::FOCUSED))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ScrollFocusFrame {
    area: Rect,
    focused: Option<Id>,
    content_len: usize,
}

fn scroll_view_revealing_focus(
    state: &ScrollState,
    area: Rect,
    content_len: usize,
    focused: Option<Id>,
    previous: Option<ScrollFocusFrame>,
    requested: Option<Id>,
    focus_rows: &[(Id, usize)],
) -> (ScrollState, Option<Id>, ScrollFocusFrame) {
    let transitioned = previous.is_none_or(|frame| {
        frame.area != area || frame.focused != focused || frame.content_len != content_len
    });
    let requested = requested.filter(|owner| Some(*owner) == focused);
    let owner = requested.or_else(|| transitioned.then_some(focused).flatten());
    let mut view = ScrollRegion::view(state, area, content_len);
    if let Some(row) = owner.and_then(|owner| focus_row(owner, focus_rows)) {
        let _ = view.reveal_span(row, 1);
    }
    let pending = owner.filter(|_| view.offset() != state.offset());
    (
        view,
        pending,
        ScrollFocusFrame {
            area,
            focused,
            content_len,
        },
    )
}

fn update_focus_scroll(
    cx: &mut Cx<'_>,
    scroll_id: Id,
    state: &mut ScrollState,
    content_len: usize,
    frame: &Cell<Option<ScrollFocusFrame>>,
    requested: &Cell<Option<Id>>,
    focus_rows: &[(Id, usize)],
) -> Response<()> {
    let wheel = cx
        .intents(scroll_id)
        .any(|intent| matches!(intent, termrock::Intent::Wheel { .. }));
    let mut response = ScrollRegion::new(scroll_id).update(cx, state, content_len);
    let focused = cx
        .focused_in_scope(termrock::ScopeId::ROOT)
        .filter(|owner| focus_row(*owner, focus_rows).is_some());
    let previous = frame.get();

    if wheel {
        requested.set(None);
        if let Some(mut previous) = previous {
            previous.focused = focused;
            previous.content_len = content_len;
            frame.set(Some(previous));
        }
        return response;
    }

    let pending = requested.take().filter(|owner| Some(*owner) == focused);
    let focus_changed = previous.map_or(focused.is_some(), |frame| frame.focused != focused);
    let owner = pending.or_else(|| focus_changed.then_some(focused).flatten());
    if let Some(owner) = owner
        && let Some(row) = focus_row(owner, focus_rows)
    {
        let before = state.offset();
        let _ = ScrollRegion::new(scroll_id).prepare(cx, state, content_len);
        let _ = state.reveal_span(row, 1);
        if state.offset() != before {
            response |= Response::changed();
        }
    }
    response
}

fn paint_section_strip(
    ui: &mut Ui<'_>,
    area: Rect,
    active_section: usize,
    focused_cursor: Option<usize>,
) {
    if area.is_empty() {
        return;
    }
    ui.fill(area, ink(CANVAS, CANVAS));
    let active_section = active_section.min(SECTIONS.len().saturating_sub(1));
    let mut x = area.x;
    let mut tab_end = area.x;
    for (index, label) in SECTIONS.iter().enumerate() {
        let w = tab_width(label);
        if x.saturating_add(w) > area.right() {
            break;
        }
        let style = if index == active_section {
            ink(WHITE, TAB_ACTIVE).add_modifier(Modifier::BOLD)
        } else if focused_cursor == Some(index) {
            ink(WHITE, CANVAS).add_modifier(Modifier::BOLD)
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
            let caption = match active_section {
                1 => "layout controls",
                2 => "native component states",
                _ => SECTION_CAPTION,
            };
            let caption_width = width(caption);
            let caption_x = area.right().saturating_sub(caption_width.saturating_add(1));
            if caption_x > tab_end.saturating_add(2) {
                let _ = ui.paint_str(
                    Rect::new(caption_x, area.y, caption_width, 1),
                    caption,
                    ink(FAINT, CANVAS),
                );
            }
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
    let mut active_x = area.x;
    for label in SECTIONS.iter().take(active_section) {
        active_x = active_x.saturating_add(tab_width(label)).saturating_add(1);
    }
    let active = tab_width(SECTIONS[active_section]).min(area.right().saturating_sub(active_x));
    let _ = ui.paint_str(
        Rect::new(active_x, rule_y, active, 1),
        &"━".repeat(usize::from(active)),
        ink(ACCENT, CANVAS),
    );
}

fn intersects_overflow_hit(layout: &termrock::navigation::tabs::TabLayout, area: Rect) -> bool {
    (0..=2).any(|index| {
        layout
            .part_bounds(PartRef::item(Part::OVERFLOW, ItemKey::index(index)))
            .is_some_and(|(_, visible)| !visible.intersection(area).is_empty())
    })
}

fn paint_tab_layout_strip(
    ui: &mut Ui<'_>,
    area: Rect,
    active_section: usize,
    focused_cursor: Option<usize>,
    layout: &termrock::navigation::tabs::TabLayout,
) {
    if area.is_empty() {
        return;
    }
    let active_section = active_section.min(SECTIONS.len().saturating_sub(1));
    let mut tab_end = area.x;
    for row in layout.rows() {
        let Some(label) = SECTIONS.get(row.index).copied() else {
            continue;
        };
        let tab = row.visible_tab.intersection(area);
        if row.logical_tab.width != tab_width(label)
            || row.logical_tab != row.visible_tab
            || tab != row.visible_tab
            || intersects_overflow_hit(layout, tab)
        {
            continue;
        }
        let style = if row.index == active_section {
            ink(WHITE, TAB_ACTIVE).add_modifier(Modifier::BOLD)
        } else if focused_cursor == Some(row.index) {
            ink(WHITE, CANVAS).add_modifier(Modifier::BOLD)
        } else {
            ink(SECONDARY, CANVAS)
        };
        ui.fill(tab, style);
        let _ = ui.paint_str(
            Rect::new(
                tab.x.saturating_add(1),
                tab.y,
                tab.width.saturating_sub(1),
                1,
            ),
            label,
            style,
        );
        tab_end = tab.right();
    }

    if area.width >= SECTION_WIDE {
        let caption = match active_section {
            1 => "layout controls",
            2 => "native component states",
            _ => SECTION_CAPTION,
        };
        let caption_width = width(caption);
        let caption_x = area.right().saturating_sub(caption_width.saturating_add(1));
        let caption_area = Rect::new(caption_x, area.y, caption_width, 1);
        let overlaps_tab = layout
            .rows()
            .iter()
            .any(|row| !row.visible_tab.intersection(caption_area).is_empty());
        if caption_x > tab_end.saturating_add(2)
            && !overlaps_tab
            && !intersects_overflow_hit(layout, caption_area)
        {
            let _ = ui.paint_str(caption_area, caption, ink(FAINT, CANVAS));
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
    if let Some(active_tab) = layout.rows().iter().find(|row| row.index == active_section) {
        let visible = active_tab.visible_tab.intersection(area);
        if !visible.is_empty() && !intersects_overflow_hit(layout, visible) {
            let _ = ui.paint_str(
                Rect::new(visible.x, rule_y, visible.width, 1),
                &"━".repeat(usize::from(visible.width)),
                ink(ACCENT, CANVAS),
            );
        }
    }
}

fn thumb_len(viewport: u16, content: u16) -> u16 {
    if content <= viewport || viewport == 0 {
        return viewport;
    }
    let len = u32::from(viewport).saturating_mul(u32::from(viewport)) / u32::from(content);
    u16::try_from(len).unwrap_or(u16::MAX).max(1).min(viewport)
}

fn thumb_start(viewport: u16, content: u16, offset: usize, thumb: u16) -> u16 {
    let travel = viewport.saturating_sub(thumb);
    let max_offset = usize::from(content.saturating_sub(viewport));
    if max_offset == 0 {
        return 0;
    }
    let numerator = offset.min(max_offset).saturating_mul(usize::from(travel));
    u16::try_from(numerator / max_offset).unwrap_or(travel)
}

fn paint_scrollbar(ui: &mut Ui<'_>, bar: Rect, content_h: u16, offset: usize) {
    let thumb = thumb_len(bar.height, content_h);
    let start = thumb_start(bar.height, content_h, offset, thumb);
    for row in 0..bar.height {
        let on_thumb = row >= start && row < start.saturating_add(thumb);
        let _ = ui.paint_str(
            Rect::new(bar.x, bar.y.saturating_add(row), 1, 1),
            if on_thumb { "┃" } else { "┆" },
            if on_thumb {
                ink(THUMB, CANVAS)
            } else {
                ink(TRACK, CANVAS)
            },
        );
    }
}

#[derive(Clone, Copy)]
struct RowClip {
    area: Rect,
    source_origin: u16,
    offset: usize,
}

impl RowClip {
    fn identity(area: Rect) -> Self {
        Self {
            area,
            source_origin: area.y,
            offset: 0,
        }
    }

    fn map_y(self, source_y: u16) -> Option<u16> {
        let relative = source_y.checked_sub(self.source_origin)?;
        let visible = relative.checked_sub(u16::try_from(self.offset).ok()?)?;
        if visible >= self.area.height {
            return None;
        }
        self.area.y.checked_add(visible)
    }

    fn contains(self, source_y: u16) -> bool {
        self.map_y(source_y).is_some()
    }

    fn right(self) -> u16 {
        self.area.right()
    }

    fn map_rect(self, rect: Rect) -> Rect {
        let source_start = self
            .source_origin
            .saturating_add(u16::try_from(self.offset).unwrap_or(u16::MAX));
        let source_end = source_start.saturating_add(self.area.height);
        let top = rect.y.max(source_start);
        let bottom = rect.bottom().min(source_end);
        let left = rect.x.max(self.area.x);
        let right = rect.right().min(self.area.right());
        if top >= bottom || left >= right {
            return Rect::default();
        }
        let Some(y_delta) = top.checked_sub(source_start) else {
            return Rect::default();
        };
        Rect::new(
            left,
            self.area.y.saturating_add(y_delta),
            right.saturating_sub(left),
            bottom.saturating_sub(top),
        )
    }
}

fn paint_clip(
    ui: &mut Ui<'_>,
    clip: RowClip,
    x: u16,
    source_y: u16,
    text: &str,
    style: PaintStyle,
) {
    let Some(y) = clip.map_y(source_y) else {
        return;
    };
    if x < clip.area.x || x >= clip.right() || text.is_empty() {
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
    clip: RowClip,
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

fn paint_card(ui: &mut Ui<'_>, clip: RowClip, rect: Rect, title: &str, meta: &str) {
    let visible = clip.map_rect(rect);
    if visible.is_empty() {
        return;
    }
    ui.fill(visible, ink(WHITE, SURFACE));
    let inner_x = rect.x.saturating_add(2);
    let span = rect.width.saturating_sub(4);
    if clip.contains(rect.y) && span > 0 {
        paint_clip(ui, clip, inner_x, rect.y, title, ink(SECONDARY, SURFACE));
        let meta_w = width(meta);
        if !meta.is_empty() && span > meta_w {
            paint_clip(
                ui,
                clip,
                inner_x.saturating_add(span.saturating_sub(meta_w)),
                rect.y,
                meta,
                ink(FAINT, SURFACE),
            );
        }
    }
    let rule_y = rect.y.saturating_add(1);
    if clip.contains(rule_y) && span > 0 {
        paint_repeat(ui, clip, inner_x, rule_y, span, "─", ink(RULE, SURFACE));
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

fn paint_bar(
    ui: &mut Ui<'_>,
    clip: RowClip,
    area: Rect,
    label: &str,
    ratio: Option<f64>,
    kind: BarKind,
    options: BarOptions,
) {
    if area.height == 0 || area.width == 0 || !clip.contains(area.y) {
        return;
    }
    let requested_label_width = match options.label_width {
        1 => width(label),
        2 => 10,
        _ => LABEL_W,
    }
    .min(area.width);
    let has_label = !label.is_empty() && area.width > requested_label_width.saturating_add(8);
    let mut x = area.x;
    if has_label {
        let label = truncate(label, requested_label_width);
        paint_clip(ui, clip, x, area.y, &label, ink(WHITE, SURFACE));
        x = x.saturating_add(requested_label_width).saturating_add(2);
    }
    let room = area.right().saturating_sub(x);
    let reserve = match ratio {
        Some(_) => VALUE_W,
        None if options.reserve_sweep_value && room >= VALUE_W.saturating_add(6) => VALUE_W,
        None => 0,
    };
    let track_w = room.saturating_sub(reserve);
    if track_w < 6 {
        if let Some(ratio) = ratio {
            paint_clip(
                ui,
                clip,
                x,
                area.y,
                pct_field(ratio).trim_start(),
                ink(SECONDARY, SURFACE),
            );
        }
        return;
    }
    let (filled, color) = if let Some(ratio) = ratio {
        let filled = (f64::from(track_w) * ratio.clamp(0.0, 1.0)).round() as u16;
        (filled.min(track_w), fill_color(kind))
    } else {
        let track_i = i32::from(track_w);
        let seg = (track_i / 5).clamp(2, 8);
        let period = usize::try_from(track_i.saturating_add(seg))
            .unwrap_or(1)
            .max(1);
        let pos = i32::try_from(options.frame % period)
            .unwrap_or(0)
            .saturating_sub(seg);
        let mut filled_from = 0u16;
        let mut filled_to = 0u16;
        for i in 0..track_i {
            let on = i >= pos && i < pos.saturating_add(seg);
            if on && filled_to == filled_from && i >= 0 {
                filled_from = u16::try_from(i).unwrap_or(track_w);
                filled_to = filled_from;
            }
            if on {
                filled_to = u16::try_from(i.saturating_add(1)).unwrap_or(track_w);
            }
        }
        paint_repeat(ui, clip, x, area.y, track_w, "─", ink(RULE, SURFACE));
        if filled_to > filled_from {
            paint_repeat(
                ui,
                clip,
                x.saturating_add(filled_from),
                area.y,
                filled_to.saturating_sub(filled_from),
                "━",
                ink(ACCENT, SURFACE),
            );
        }
        return;
    };
    paint_repeat(ui, clip, x, area.y, track_w, "─", ink(RULE, SURFACE));
    paint_repeat(ui, clip, x, area.y, filled, "━", ink(color, SURFACE));
    let value = format!(" {}", pct_field(ratio.unwrap_or(0.0)));
    paint_clip(
        ui,
        clip,
        x.saturating_add(track_w),
        area.y,
        &value,
        ink(SECONDARY, SURFACE),
    );
    paint_clip(
        ui,
        clip,
        x.saturating_add(track_w).saturating_add(5),
        area.y,
        suffix(kind),
        ink(color, SURFACE),
    );
}

fn progress_button<'a>(ui: &Ui<'_>, id: Id, button: Button<'a>) -> Button<'a> {
    let state = ui.state(id);
    if state.contains(StateFlags::PRESSED) || state.contains(StateFlags::DISABLED) {
        button
    } else if state.contains(StateFlags::HOVERED) {
        button.patch_part(PROGRESS_BUTTON_HOVER)
    } else if state.contains(StateFlags::FOCUSED) || state.contains(StateFlags::FOCUS_VISIBLE) {
        button
    } else {
        button.patch(&PROGRESS_BUTTON_IDLE)
    }
}

fn paint_button(ui: &mut Ui<'_>, button: Button<'_>, id: Id, area: Rect) -> Rect {
    progress_button(ui, id, button).draw(ui, area)
}

fn row_at(card: Rect, dy: u16, x: u16, w: u16) -> Rect {
    Rect::new(x, card.y.saturating_add(dy), w, 1)
}

fn paint_live(
    ui: &mut Ui<'_>,
    clip: RowClip,
    card: Rect,
    build: f64,
    paused: bool,
    options: BarOptions,
) {
    paint_card(ui, clip, card, "Live", "ticks at 80 ms");
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
        "Building",
        Some(build),
        kind,
        options,
    );
    paint_bar(
        ui,
        clip,
        row_at(card, 4, inner_x, bar_w),
        "Resolving",
        None,
        BarKind::Sweep,
        options,
    );
    let spin_y = card.y.saturating_add(6);
    let glyph = SPINNER[options.frame % SPINNER.len()];
    paint_clip(ui, clip, inner_x, spin_y, glyph, ink(ACCENT, SURFACE));
    paint_clip(
        ui,
        clip,
        inner_x.saturating_add(2),
        spin_y,
        "Waiting for the test runner",
        ink(SECONDARY, SURFACE),
    );
    let button_source_y = card.y.saturating_add(8);
    let Some(button_y) = clip.map_y(button_source_y) else {
        ui.register_focus_only(RESTART, termrock::Focusability::Focusable);
        ui.register_focus_only(PAUSE, termrock::Focusability::Focusable);
        return;
    };
    let restart = Button::new(RESTART, "Restart").variant(Variant::SECONDARY);
    let pause_label = if paused { "Resume" } else { "Pause" };
    let pause = Button::new(PAUSE, pause_label).variant(Variant::SECONDARY);
    let row = Rect::new(inner_x, button_y, inner_w, 1);
    let first = paint_button(ui, restart, RESTART, row);
    let next_x = first.right().saturating_add(2);
    if next_x < row.right() {
        paint_button(
            ui,
            pause,
            PAUSE,
            Rect::new(next_x, button_y, row.right().saturating_sub(next_x), 1),
        );
    } else {
        ui.register_focus_only(PAUSE, termrock::Focusability::Focusable);
    }
}

fn paint_tally(ui: &mut Ui<'_>, clip: RowClip, x: u16, y: u16) {
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

fn paint_jobs(ui: &mut Ui<'_>, clip: RowClip, card: Rect, options: BarOptions) {
    paint_card(ui, clip, card, "Deploy checkout-service", "one BarLayout");
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
            job.label,
            job.ratio,
            job.kind,
            options,
        );
    }
}

fn paint_line_meter(ui: &mut Ui<'_>, clip: RowClip, area: Rect, pct: u8, value: &str, color: u32) {
    if !clip.contains(area.y) || area.width == 0 {
        return;
    }
    let vw = width(value);
    let track_w = area.width.saturating_sub(vw.saturating_add(3));
    if track_w < 6 {
        paint_clip(ui, clip, area.x, area.y, value, ink(WHITE, SURFACE));
        return;
    }
    let filled = (f64::from(track_w) * f64::from(pct) / 100.0).round() as u16;
    paint_repeat(ui, clip, area.x, area.y, track_w, "─", ink(RULE, SURFACE));
    paint_repeat(
        ui,
        clip,
        area.x,
        area.y,
        filled.min(track_w),
        "━",
        ink(color, SURFACE),
    );
    let text_x = area.x.saturating_add(track_w).saturating_add(1);
    paint_clip(ui, clip, text_x, area.y, value, ink(WHITE, SURFACE));
    paint_clip(
        ui,
        clip,
        text_x.saturating_add(vw),
        area.y,
        "  ",
        ink(color, SURFACE),
    );
}

fn paint_block_meter(ui: &mut Ui<'_>, clip: RowClip, area: Rect, pct: u8, value: &str, color: u32) {
    if !clip.contains(area.y) || area.width == 0 {
        return;
    }
    let bar_w = area.width.saturating_sub(2);
    if bar_w < 4 {
        paint_clip(ui, clip, area.x, area.y, value, ink(WHITE, SURFACE));
        return;
    }
    let filled = (f64::from(bar_w) * f64::from(pct) / 100.0).round() as u16;
    let chars: Vec<char> = value.chars().collect();
    for i in 0..bar_w {
        let x = area.x.saturating_add(i);
        if x >= clip.right() {
            break;
        }
        let on_fill = i < filled;
        let mut style = if on_fill {
            ink(CANVAS, color).add_modifier(Modifier::BOLD)
        } else {
            ink(WHITE, BLOCK_REST)
        };
        let symbol = if i == 0 {
            " ".to_owned()
        } else {
            chars
                .get(usize::from(i.saturating_sub(1)))
                .map(|ch| ch.to_string())
                .unwrap_or_else(|| " ".to_owned())
        };
        if !on_fill {
            style = ink(WHITE, BLOCK_REST);
        }
        paint_clip(ui, clip, x, area.y, &symbol, style);
    }
    paint_clip(
        ui,
        clip,
        area.x.saturating_add(bar_w),
        area.y,
        "  ",
        ink(color, SURFACE),
    );
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

fn paint_wide_line(ui: &mut Ui<'_>, clip: RowClip, area: Rect, line: WideLine<'_>) {
    if !clip.contains(area.y) || area.width == 0 {
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

fn paint_wide_meters(ui: &mut Ui<'_>, clip: RowClip, card: Rect, frame: usize, stale_style: usize) {
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
    paint_wide_line(
        ui,
        clip,
        row(6),
        WideLine {
            pct: 72,
            glyph: "━",
            color: WARNING,
            value: "72% used ▲",
            value_color: WARNING,
            value_mod: plain,
            trail: "",
            trail_color: WARNING,
            trail_mod: plain,
        },
    );
    if right_x < card.right() {
        paint_line_meter(ui, clip, right(6), 72, "72% disk, warns at 80", INFO);
    }
    paint_wide_line(
        ui,
        clip,
        row(7),
        WideLine {
            pct: 91,
            glyph: "━",
            color: ERROR,
            value: "91% used !",
            value_color: ERROR,
            value_mod: plain,
            trail: "",
            trail_color: ERROR,
            trail_mod: plain,
        },
    );
    paint_wide_line(
        ui,
        clip,
        right(7),
        WideLine {
            pct: 88,
            glyph: "━",
            color: WARNING,
            value: "88% disk, critical at 95 ▲",
            value_color: WARNING,
            value_mod: plain,
            trail: "",
            trail_color: WARNING,
            trail_mod: plain,
        },
    );
    paint_wide_line(
        ui,
        clip,
        row(8),
        WideLine {
            pct: 82,
            glyph: "━",
            color: WARNING,
            value: "82% used ▲",
            value_color: WARNING,
            value_mod: plain,
            trail: "",
            trail_color: WARNING,
            trail_mod: plain,
        },
    );
    paint_wide_line(
        ui,
        clip,
        right(8),
        WideLine {
            pct: 30,
            glyph: "━",
            color: ERROR,
            value: "30% · fixed high !",
            value_color: ERROR,
            value_mod: plain,
            trail: "",
            trail_color: ERROR,
            trail_mod: plain,
        },
    );
    paint_wide_line(
        ui,
        clip,
        row(9),
        WideLine {
            pct: 100,
            glyph: "━",
            color: ERROR,
            value: "100% used",
            value_color: ERROR,
            value_mod: plain,
            trail: " !",
            trail_color: ERROR,
            trail_mod: Modifier::BOLD,
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
    let selected_stale = if stale_style == 1 {
        WideLine {
            pct: 76,
            glyph: "━",
            color: FAINT,
            value: "76% used",
            value_color: MUTED,
            value_mod: plain,
            trail: "  ",
            trail_color: FAINT,
            trail_mod: plain,
        }
    } else {
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
        }
    };
    paint_wide_line(ui, clip, row(10), selected_stale);
    paint_wide_line(
        ui,
        clip,
        right(10),
        WideLine {
            pct: 76,
            glyph: "━",
            color: FAINT,
            value: "76% · faint stale style",
            value_color: MUTED,
            value_mod: plain,
            trail: "  ",
            trail_color: FAINT,
            trail_mod: plain,
        },
    );
    let refreshing = format!("{} refreshing  ", SPINNER[frame % SPINNER.len()]);
    paint_wide_line(
        ui,
        clip,
        row(11),
        WideLine {
            pct: 54,
            glyph: "━",
            color: MUTED,
            value: &refreshing,
            value_color: MUTED,
            value_mod: plain,
            trail: "",
            trail_color: MUTED,
            trail_mod: plain,
        },
    );
    paint_clip(
        ui,
        clip,
        left_x,
        card.y.saturating_add(12),
        "quota read failed !",
        face(ERROR, SURFACE, plain),
    );
    paint_clip(
        ui,
        clip,
        left_x,
        card.y.saturating_add(13),
        "—  ",
        face(FAINT, SURFACE, plain),
    );
}

fn paint_meters(
    ui: &mut Ui<'_>,
    clip: RowClip,
    card: Rect,
    meter_visual: usize,
    stale_style: usize,
) {
    paint_card(ui, clip, card, "Meters", "capacity, not completion");
    let inner_x = card.x.saturating_add(2);
    let inner_w = card.width.saturating_sub(4);
    for (index, note) in NOTES.iter().enumerate() {
        let Ok(step) = u16::try_from(index) else {
            break;
        };
        let note = if index == 1 && stale_style == 1 {
            "Stale keeps its level, faint."
        } else {
            *note
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
    let low_meter = Rect::new(meter_x, y, meter_w, 1);
    if meter_visual == 1 {
        paint_block_meter(ui, clip, low_meter, 38, "38% used · block", INFO);
    } else {
        paint_line_meter(ui, clip, low_meter, 38, "38% used", INFO);
    }
    let block_x = meter_x.saturating_add(col_w).saturating_add(1);
    if block_x < card.right() {
        paint_block_meter(
            ui,
            clip,
            Rect::new(block_x, y, meter_w, 1),
            38,
            "38% used · block",
            INFO,
        );
    }
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

fn paint_narrow_button(ui: &mut Ui<'_>, button: Button<'_>, id: Id, area: Rect) -> Rect {
    progress_button(ui, id, button).draw(ui, area)
}

fn paint_narrow_scroll(ui: &mut Ui<'_>, view: Rect) {
    let bar_h = view.height.saturating_sub(1);
    let thumb = thumb_len(bar_h, CONTENT_H);
    let x = view.right().saturating_sub(1);
    for row in 0..bar_h {
        let on = row < thumb;
        let _ = ui.paint_str(
            Rect::new(x, view.y.saturating_add(row), 1, 1),
            if on { "┃" } else { "┆" },
            settled(if on { THUMB } else { TRACK }, CANVAS, false),
        );
    }
    let track = view.width.saturating_sub(1);
    let filled = thumb_len(track, NARROW_CONTENT_W).min(track);
    let y = view.bottom().saturating_sub(1);
    if filled > 0 {
        let _ = ui.paint_str(
            Rect::new(view.x, y, filled, 1),
            &"━".repeat(usize::from(filled)),
            settled(THUMB, CANVAS, false),
        );
    }
    let rest = track.saturating_sub(filled);
    if rest > 0 {
        let _ = ui.paint_str(
            Rect::new(view.x.saturating_add(filled), y, rest, 1),
            &"┄".repeat(usize::from(rest)),
            settled(TRACK, CANVAS, false),
        );
    }
}

fn paint_narrow(ui: &mut Ui<'_>, body: Rect, build: f64, paused: bool, frame: usize) {
    paint_section_strip(
        ui,
        Rect {
            height: 2.min(body.height),
            ..body
        },
        0,
        None,
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
        let row = Rect::new(
            live.x.saturating_add(2),
            button_y,
            live.width.saturating_sub(2),
            1,
        );
        let first = paint_narrow_button(ui, restart_button(), RESTART, row);
        let next_x = first.right().saturating_add(2);
        if next_x < row.right() {
            paint_narrow_button(
                ui,
                pause_button(paused),
                PAUSE,
                Rect::new(next_x, button_y, row.right().saturating_sub(next_x), 1),
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
    tabs: TabsState,
    label_width: usize,
    reserve_sweep_value: usize,
    meter_visual: usize,
    stale_style: usize,
    label_width_state: SelectState,
    reserve_sweep_state: SelectState,
    meter_visual_state: SelectState,
    stale_style_state: SelectState,
    play_scroll: ScrollState,
    options_scroll: ScrollState,
    states_scroll: ScrollState,
    play_scroll_frame: Cell<Option<ScrollFocusFrame>>,
    play_reveal_request: Cell<Option<Id>>,
    options_scroll_frame: Cell<Option<ScrollFocusFrame>>,
    options_reveal_request: Cell<Option<Id>>,
    last_page_width: Cell<Option<u16>>,
}

impl ProgressPage {
    pub fn new() -> Self {
        Self {
            frame: 0,
            build: 0.0,
            paused: false,
            next_tick: None,
            tabs: TabsState::default(),
            label_width: 0,
            reserve_sweep_value: 0,
            meter_visual: 0,
            stale_style: 0,
            label_width_state: SelectState::default(),
            reserve_sweep_state: SelectState::default(),
            meter_visual_state: SelectState::default(),
            stale_style_state: SelectState::default(),
            play_scroll: ScrollState::default(),
            options_scroll: ScrollState::default(),
            states_scroll: ScrollState::default(),
            play_scroll_frame: Cell::new(None),
            play_reveal_request: Cell::new(None),
            options_scroll_frame: Cell::new(None),
            options_reveal_request: Cell::new(None),
            last_page_width: Cell::new(None),
        }
    }
}

impl ProgressPage {
    fn section(&self) -> usize {
        match self.tabs.active().or(self.tabs.cursor()) {
            Some(ItemKey::Index(index)) => index.min(SECTIONS.len().saturating_sub(1)),
            _ => 0,
        }
    }

    fn effective_section(&self, width: u16) -> usize {
        if width < FULL_MIN_W {
            0
        } else {
            self.section()
        }
    }

    fn update_layout_width(&self, cx: &Cx<'_>) -> u16 {
        let viewport_width = cx.viewport().width;
        if viewport_width < FULL_MIN_W {
            viewport_width
        } else {
            self.last_page_width.get().unwrap_or(viewport_width)
        }
    }

    fn scroll_window(
        view: Rect,
        state: &ScrollState,
        content_rows: usize,
    ) -> Option<VerticalWindow> {
        let offset = ScrollRegion::view(state, view, content_rows).offset();
        let height = usize::from(view.height).min(content_rows.saturating_sub(offset));
        let visible = Rect {
            height: u16::try_from(height).ok()?,
            ..view
        };
        VerticalWindow::new(visible, content_rows, offset).ok()
    }

    fn update_options(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = Response::ignored();
        response |= Select::new(LABEL_WIDTH_SELECT)
            .popup_rows(3)
            .update_in_form(
                cx,
                &mut self.label_width_state,
                &mut self.label_width,
                LABEL_WIDTHS,
                false,
            )
            .erase();
        response |= Select::new(SWEEP_RESERVE_SELECT)
            .popup_rows(2)
            .update_in_form(
                cx,
                &mut self.reserve_sweep_state,
                &mut self.reserve_sweep_value,
                SWEEP_VALUE_COLUMNS,
                false,
            )
            .erase();
        response |= Select::new(METER_VISUAL_SELECT)
            .popup_rows(2)
            .update_in_form(
                cx,
                &mut self.meter_visual_state,
                &mut self.meter_visual,
                METER_VISUALS,
                false,
            )
            .erase();
        response |= Select::new(STALE_STYLE_SELECT)
            .popup_rows(2)
            .update_in_form(
                cx,
                &mut self.stale_style_state,
                &mut self.stale_style,
                STALE_STYLES,
                false,
            )
            .erase();
        response
    }

    fn draw_playground(&self, ui: &mut Ui<'_>, area: Rect) {
        let content_len = usize::from(CONTENT_H);
        let focused = focused_owner(ui, PLAY_FOCUS_ROWS);
        let (view_state, request, focus_frame) = scroll_view_revealing_focus(
            &self.play_scroll,
            area,
            content_len,
            focused,
            self.play_scroll_frame.get(),
            self.play_reveal_request.get(),
            PLAY_FOCUS_ROWS,
        );
        self.play_reveal_request.set(request);
        self.play_scroll_frame.set(Some(focus_frame));
        let scroll_content = ScrollRegion::new(PLAY_SCROLL).preserve_underlay(true).draw(
            ui,
            area,
            &view_state,
            content_len,
        );
        let offset = view_state.offset();
        let scrolling = view_state.overflows();
        let cards = Rect {
            width: if scrolling {
                scroll_content.width.saturating_sub(1)
            } else {
                scroll_content.width
            },
            ..scroll_content
        };
        let clip = RowClip {
            area: cards,
            source_origin: scroll_content.y,
            offset,
        };
        let live = Rect {
            y: scroll_content.y,
            height: LIVE_H,
            ..cards
        };
        let bar_options = BarOptions {
            frame: self.frame,
            label_width: self.label_width,
            reserve_sweep_value: self.reserve_sweep_value == 0,
        };
        paint_live(ui, clip, live, self.build, self.paused, bar_options);
        let jobs_y = scroll_content.y.saturating_add(LIVE_H).saturating_add(1);
        paint_jobs(
            ui,
            clip,
            Rect::new(cards.x, jobs_y, cards.width, JOBS_H),
            bar_options,
        );
        let meters_y = jobs_y.saturating_add(JOBS_H).saturating_add(1);
        let meters = Rect::new(cards.x, meters_y, cards.width, METERS_H);
        paint_meters(ui, clip, meters, self.meter_visual, self.stale_style);
        if wide_page(area) {
            paint_wide_meters(ui, clip, meters, self.frame, self.stale_style);
        }
        if scrolling {
            paint_scrollbar(
                ui,
                Rect::new(
                    scroll_content.right(),
                    scroll_content.y,
                    1,
                    scroll_content.height,
                ),
                CONTENT_H,
                offset,
            );
        }
    }

    fn register_hidden_option(ui: &mut Ui<'_>, id: Id) {
        match id {
            LABEL_WIDTH_SELECT => {
                Select::new(id).register_focus_only_in_form(ui, false);
            }
            SWEEP_RESERVE_SELECT => {
                Select::new(id).register_focus_only_in_form(ui, false);
            }
            METER_VISUAL_SELECT => {
                Select::new(id).register_focus_only_in_form(ui, false);
            }
            STALE_STYLE_SELECT => {
                Select::new(id).register_focus_only_in_form(ui, false);
            }
            _ => {}
        }
    }

    fn draw_options(&self, ui: &mut Ui<'_>, area: Rect) {
        let focused = focused_owner(ui, OPTION_FOCUS_ROWS);
        let (view_state, request, focus_frame) = scroll_view_revealing_focus(
            &self.options_scroll,
            area,
            OPTIONS_ROWS,
            focused,
            self.options_scroll_frame.get(),
            self.options_reveal_request.get(),
            OPTION_FOCUS_ROWS,
        );
        self.options_reveal_request.set(request);
        self.options_scroll_frame.set(Some(focus_frame));
        let view = ScrollRegion::new(OPTIONS_SCROLL)
            .preserve_underlay(true)
            .draw(ui, area, &view_state, OPTIONS_ROWS);
        let Some(window) = Self::scroll_window(view, &view_state, OPTIONS_ROWS) else {
            for (id, _) in OPTION_FOCUS_ROWS {
                Self::register_hidden_option(ui, *id);
            }
            return;
        };
        let label_w = 22u16.min(view.width.saturating_sub(8));
        let field_x = view.x.saturating_add(label_w).saturating_add(2);
        let field_w = view.right().saturating_sub(field_x);
        let rows = [
            (1usize, "Label width", LABEL_WIDTH_SELECT),
            (2usize, "Sweep value column", SWEEP_RESERVE_SELECT),
            (3usize, "Low meter visual", METER_VISUAL_SELECT),
            (4usize, "Stale style", STALE_STYLE_SELECT),
        ];
        ui.with_area(view, |ui| {
            if let Some(y) = window.physical_row_for_source(0) {
                let _ = ui.paint_str(
                    Rect::new(view.x, y, view.width, 1),
                    "Changes apply to Playground and selected States samples.",
                    ink(SECONDARY, CANVAS),
                );
            }
            for (row, label, id) in rows {
                let Some(y) = window.physical_row_for_source(row) else {
                    Self::register_hidden_option(ui, id);
                    continue;
                };
                let _ = ui.paint_str(Rect::new(view.x, y, label_w, 1), label, ink(WHITE, CANVAS));
                let area = Rect::new(field_x, y, field_w, 1);
                match id {
                    LABEL_WIDTH_SELECT => {
                        let _ =
                            Select::new(id).draw(ui, area, &self.label_width_state, LABEL_WIDTHS);
                    }
                    SWEEP_RESERVE_SELECT => {
                        let _ = Select::new(id).draw(
                            ui,
                            area,
                            &self.reserve_sweep_state,
                            SWEEP_VALUE_COLUMNS,
                        );
                    }
                    METER_VISUAL_SELECT => {
                        let _ =
                            Select::new(id).draw(ui, area, &self.meter_visual_state, METER_VISUALS);
                    }
                    STALE_STYLE_SELECT => {
                        let _ =
                            Select::new(id).draw(ui, area, &self.stale_style_state, STALE_STYLES);
                    }
                    _ => {}
                }
            }
            if let Some(y) = window.physical_row_for_source(5) {
                let summary = format!(
                    "Bars: {}; sweep: {}; meter: {}; stale: {}.",
                    LABEL_WIDTHS[self.label_width.min(LABEL_WIDTHS.len() - 1)],
                    SWEEP_VALUE_COLUMNS
                        [self.reserve_sweep_value.min(SWEEP_VALUE_COLUMNS.len() - 1)],
                    METER_VISUALS[self.meter_visual.min(METER_VISUALS.len() - 1)],
                    STALE_STYLES[self.stale_style.min(STALE_STYLES.len() - 1)],
                );
                let _ = ui.paint_str(
                    Rect::new(view.x, y, view.width, 1),
                    &summary,
                    ink(MUTED, CANVAS),
                );
            }
        });
    }

    fn state_row_areas(
        view: Rect,
        window: VerticalWindow,
        source_row: usize,
    ) -> Option<(Rect, Rect)> {
        let y = window.physical_row_for_source(source_row)?;
        let label_w = 23u16.min(view.width.saturating_sub(7));
        let widget_x = view
            .x
            .saturating_add(label_w)
            .saturating_add(2)
            .min(view.right());
        Some((
            Rect::new(view.x, y, label_w, 1),
            Rect::new(widget_x, y, view.right().saturating_sub(widget_x), 1),
        ))
    }

    fn draw_state_row(
        ui: &mut Ui<'_>,
        window: VerticalWindow,
        view: Rect,
        source_row: usize,
        label: &str,
        draw_widget: impl FnOnce(&mut Ui<'_>, Rect),
    ) {
        let Some((label_area, widget_area)) = Self::state_row_areas(view, window, source_row)
        else {
            return;
        };
        let _ = ui.paint_str(label_area, label, ink(SECONDARY, CANVAS));
        if !widget_area.is_empty() {
            draw_widget(ui, widget_area);
        }
    }

    fn draw_states(&self, ui: &mut Ui<'_>, area: Rect) {
        let view = ScrollRegion::new(STATES_SCROLL)
            .preserve_underlay(true)
            .draw(ui, area, &self.states_scroll, STATES_ROWS);
        let Some(window) = Self::scroll_window(view, &self.states_scroll, STATES_ROWS) else {
            return;
        };
        ui.with_area(view, |ui| {
            for (row, heading) in [
                (
                    0usize,
                    "Native component states · wheel to inspect the full list",
                ),
                (1, "ProgressBar"),
                (9, "Meter and Spinner"),
                (16, "Retained stale appearance"),
                (20, "Clipping and long values"),
            ] {
                if let Some(y) = window.physical_row_for_source(row) {
                    let _ = ui.paint_str(
                        Rect::new(view.x, y, view.width, 1),
                        heading,
                        ink(WHITE, CANVAS).add_modifier(Modifier::BOLD),
                    );
                }
            }

            Self::draw_state_row(ui, window, view, 2, "Queued · 0%", |ui, row| {
                ProgressBar::new(PB_QUEUED)
                    .label("Queued")
                    .ratio(0.0)
                    .draw(ui, row);
            });
            Self::draw_state_row(ui, window, view, 3, "Halfway · 50%", |ui, row| {
                ProgressBar::new(PB_HALF)
                    .label("Halfway")
                    .ratio(0.5)
                    .draw(ui, row);
            });
            Self::draw_state_row(ui, window, view, 4, "Complete · 100%", |ui, row| {
                ProgressBar::new(PB_COMPLETE)
                    .label("Complete")
                    .ratio(1.0)
                    .done(true)
                    .draw(ui, row);
            });
            Self::draw_state_row(ui, window, view, 5, "Failed · error", |ui, row| {
                ProgressBar::new(PB_ERROR)
                    .label("Build image")
                    .ratio(0.64)
                    .status(Status::Error)
                    .draw(ui, row);
            });
            Self::draw_state_row(ui, window, view, 6, "Paused", |ui, row| {
                ProgressBar::new(PB_PAUSED)
                    .label("Migrate schema")
                    .ratio(0.3)
                    .icon(GlyphRole::ProgressPaused)
                    .draw(ui, row);
            });
            Self::draw_state_row(ui, window, view, 7, "Indeterminate", |ui, row| {
                ProgressBar::new(PB_SWEEP)
                    .label("Warm caches")
                    .status(Status::Busy)
                    .frame(self.frame)
                    .draw(ui, row);
            });
            Self::draw_state_row(ui, window, view, 8, "Files in flight", |ui, row| {
                Spinner::new(SPINNER_FILES)
                    .label("3 of 12 files")
                    .frame(self.frame)
                    .draw(ui, row);
            });
            Self::draw_state_row(ui, window, view, 10, "CPU quota · Auto", |ui, row| {
                Meter::new(METER_CPU)
                    .ratio(0.72)
                    .value("72% of 8 vCPU")
                    .visual(self.meter_visual())
                    .draw(ui, row);
            });
            Self::draw_state_row(ui, window, view, 11, "Busy latency · 142 ms", |ui, row| {
                Meter::new(METER_LATENCY)
                    .ratio(0.42)
                    .value("142 ms")
                    .tone(MeterTone::Medium)
                    .status(Status::Busy)
                    .leading_activity(true)
                    .frame(self.frame)
                    .draw(ui, row);
            });
            Self::draw_state_row(ui, window, view, 12, "Last sync · suffix 4", |ui, row| {
                Meter::new(METER_SYNC)
                    .value("last sync 14:02")
                    .tone(MeterTone::Stale)
                    .suffix_width(4)
                    .draw(ui, row);
            });
            Self::draw_state_row(ui, window, view, 13, "Unknown reading", |ui, row| {
                Meter::new(METER_UNKNOWN)
                    .value("unknown")
                    .tone(MeterTone::Unknown)
                    .visual(self.meter_visual())
                    .draw(ui, row);
            });
            Self::draw_state_row(ui, window, view, 14, "Zero reading", |ui, row| {
                Meter::new(METER_ZERO)
                    .ratio(0.0)
                    .value("0%")
                    .tone(MeterTone::Low)
                    .visual(self.meter_visual())
                    .draw(ui, row);
            });
            Self::draw_state_row(ui, window, view, 15, "Full reading", |ui, row| {
                Meter::new(METER_FULL)
                    .ratio(1.0)
                    .value("100%")
                    .tone(MeterTone::High)
                    .visual(self.meter_visual())
                    .draw(ui, row);
            });
            Self::draw_state_row(ui, window, view, 17, "Selected stale style", |ui, row| {
                let selected = if self.stale_style == 1 {
                    WideLine {
                        pct: 76,
                        glyph: "━",
                        color: FAINT,
                        value: "76% stale",
                        value_color: MUTED,
                        value_mod: Modifier::empty(),
                        trail: "  ",
                        trail_color: FAINT,
                        trail_mod: Modifier::empty(),
                    }
                } else {
                    WideLine {
                        pct: 76,
                        glyph: "╍",
                        color: WARNING,
                        value: "76% stale",
                        value_color: MUTED,
                        value_mod: Modifier::ITALIC,
                        trail: " ▲",
                        trail_color: WARNING,
                        trail_mod: Modifier::empty(),
                    }
                };
                paint_wide_line(ui, RowClip::identity(row), row, selected);
            });
            Self::draw_state_row(ui, window, view, 18, "Faint stale comparison", |ui, row| {
                paint_wide_line(
                    ui,
                    RowClip::identity(row),
                    row,
                    WideLine {
                        pct: 76,
                        glyph: "━",
                        color: FAINT,
                        value: "76% stale",
                        value_color: MUTED,
                        value_mod: Modifier::empty(),
                        trail: "  ",
                        trail_color: FAINT,
                        trail_mod: Modifier::empty(),
                    },
                );
            });
            Self::draw_state_row(ui, window, view, 19, "Refreshing · loading", |ui, row| {
                Meter::new(METER_REFRESH)
                    .ratio(0.54)
                    .value("refreshing")
                    .tone(MeterTone::Medium)
                    .status(Status::Loading)
                    .frame(self.frame)
                    .visual(self.meter_visual())
                    .draw(ui, row);
            });
            Self::draw_state_row(ui, window, view, 21, "Meter error", |ui, row| {
                Meter::new(METER_FAILED)
                    .ratio(0.64)
                    .value("quota read failed")
                    .tone(MeterTone::High)
                    .status(Status::Error)
                    .visual(self.meter_visual())
                    .draw(ui, row);
            });
            Self::draw_state_row(ui, window, view, 22, "Narrow value preview", |ui, row| {
                let narrow = Rect {
                    width: row.width.min(NARROW_METER_W),
                    ..row
                };
                Meter::new(METER_NARROW)
                    .ratio(0.72)
                    .value("72% of 8 vCPU")
                    .tone(MeterTone::Medium)
                    .visual(self.meter_visual())
                    .draw(ui, narrow);
            });
            Self::draw_state_row(ui, window, view, 23, "Long label", |ui, row| {
                ProgressBar::new(PB_SWEEP)
                    .label("Archive production artifacts")
                    .ratio(0.32)
                    .draw(ui, row);
            });
            Self::draw_state_row(ui, window, view, 24, "Long readout", |ui, row| {
                Meter::new(METER_LONG)
                    .ratio(0.82)
                    .value("82% · stale warning")
                    .tone(MeterTone::Stale)
                    .suffix_width(4)
                    .visual(self.meter_visual())
                    .draw(ui, row);
            });
        });
    }

    fn meter_visual(&self) -> MeterVisual {
        if self.meter_visual == 1 {
            MeterVisual::Block
        } else {
            MeterVisual::Line
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
        let page_width = self.update_layout_width(cx);
        let compact = page_width < FULL_MIN_W;
        let mut reconcile_option_popup = false;
        if compact {
            for id in [
                LABEL_WIDTH_SELECT,
                SWEEP_RESERVE_SELECT,
                METER_VISUAL_SELECT,
                STALE_STYLE_SELECT,
            ] {
                let layer_open = cx.is_open(id);
                let state_open = match id {
                    LABEL_WIDTH_SELECT => self.label_width_state.is_open(),
                    SWEEP_RESERVE_SELECT => self.reserve_sweep_state.is_open(),
                    METER_VISUAL_SELECT => self.meter_visual_state.is_open(),
                    STALE_STYLE_SELECT => self.stale_style_state.is_open(),
                    _ => false,
                };
                if layer_open || state_open {
                    if layer_open {
                        cx.close_layer(id, None);
                    }

                    reconcile_option_popup = true;
                }
            }
        }
        let tabs_ready = !compact && cx.layout(PAGE_TABS).is_some();
        let page_up = tabs_ready
            && cx.intents(PAGE_TABS).any(|intent| {
                matches!(intent, termrock::Intent::Binding(action) if action == PAGE_SCROLL_UP)
            });
        let page_down = tabs_ready
            && cx.intents(PAGE_TABS).any(|intent| {
                matches!(intent, termrock::Intent::Binding(action) if action == PAGE_SCROLL_DOWN)
            });
        let mut response = if tabs_ready {
            Tabs::new(PAGE_TABS)
                .item_affixes(section_tab_affixes)
                .patch_part(PROGRESS_TABS_CONTAINER)
                .update(cx, &mut self.tabs, &SECTIONS)
                .erase()
        } else {
            Response::ignored()
        };
        if reconcile_option_popup {
            response |= self.update_options(cx);
        }
        let mut status = None;
        let section = self.effective_section(page_width);
        match section {
            0 if compact => {
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
            }
            0 if cx.layout(PLAY_SCROLL).is_some() => {
                let content_len = cx
                    .layout(PLAY_SCROLL)
                    .map_or(usize::from(CONTENT_H), |layout| layout.content_len);
                response |= update_focus_scroll(
                    cx,
                    PLAY_SCROLL,
                    &mut self.play_scroll,
                    content_len,
                    &self.play_scroll_frame,
                    &self.play_reveal_request,
                    PLAY_FOCUS_ROWS,
                );
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
            }
            1 if tabs_ready && cx.layout(OPTIONS_SCROLL).is_some() => {
                let content_len = cx
                    .layout(OPTIONS_SCROLL)
                    .map_or(OPTIONS_ROWS, |layout| layout.content_len);
                response |= update_focus_scroll(
                    cx,
                    OPTIONS_SCROLL,
                    &mut self.options_scroll,
                    content_len,
                    &self.options_scroll_frame,
                    &self.options_reveal_request,
                    OPTION_FOCUS_ROWS,
                );
                response |= self.update_options(cx);
            }
            2 if tabs_ready && cx.layout(STATES_SCROLL).is_some() => {
                let content_len = cx
                    .layout(STATES_SCROLL)
                    .map_or(STATES_ROWS, |layout| layout.content_len);
                response |= ScrollRegion::new(STATES_SCROLL).update(
                    cx,
                    &mut self.states_scroll,
                    content_len,
                );
            }
            _ => {}
        }

        if page_up || page_down {
            let section = self.effective_section(page_width);
            let before = match section {
                0 => self.play_scroll.offset(),
                1 => self.options_scroll.offset(),
                _ => self.states_scroll.offset(),
            };
            match section {
                0 if page_up => self.play_scroll.page_up(),
                0 => self.play_scroll.page_down(),
                1 if page_up => self.options_scroll.page_up(),
                1 => self.options_scroll.page_down(),
                _ if page_up => self.states_scroll.page_up(),
                _ => self.states_scroll.page_down(),
            }
            let after = match section {
                0 => self.play_scroll.offset(),
                1 => self.options_scroll.offset(),
                _ => self.states_scroll.offset(),
            };
            response |= if before == after {
                Response::consumed()
            } else {
                Response::changed()
            };
        }

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
        self.last_page_width.set(Some(area.width));
        frame(ui, area, self.title(), BLURB, |ui, body| {
            if compact_page(area) {
                paint_narrow(ui, body, self.build, self.paused, self.frame);
                return;
            }
            let tabs_area = Rect {
                height: 2.min(body.height),
                ..body
            };
            let tabs = Tabs::new(PAGE_TABS)
                .item_affixes(section_tab_affixes)
                .patch_part(PROGRESS_TABS_CONTAINER);
            let layout = tabs.layout_default(
                tabs_area,
                tabs_area.intersection(ui.full()),
                &self.tabs,
                &SECTIONS,
                None,
            );
            if tabs.draw_layout(ui, &layout).is_err() {
                return;
            }
            ui.publish_additional_bindings(PAGE_TABS, ui.state(PAGE_TABS), PAGE_SCROLL_BINDINGS);
            let focused_cursor = ui
                .state(PAGE_TABS)
                .contains(termrock::StateFlags::FOCUSED)
                .then(|| self.tabs.cursor())
                .flatten()
                .and_then(|key| match key {
                    ItemKey::Index(index) => Some(index),
                    _ => None,
                });
            let section = self.effective_section(area.width);
            paint_tab_layout_strip(ui, tabs_area, section, focused_cursor, &layout);
            let content_top = body.y.saturating_add(3);
            if content_top >= body.bottom() {
                return;
            }
            let content = Rect {
                y: content_top,
                height: body.bottom().saturating_sub(content_top),
                ..body
            };
            match section {
                0 => self.draw_playground(ui, content),
                1 => self.draw_options(ui, content),
                _ => self.draw_states(ui, content),
            }
        });
        paint_boundary_meta(ui, area, self.title(), BLURB);
    }

    fn hints(&self, ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        let width = self.last_page_width.get().unwrap_or(ui.full().width);
        if width < FULL_MIN_W {
            return SCROLL_HINTS;
        }
        if ui.state(PAGE_TABS).contains(termrock::StateFlags::FOCUSED) {
            return TAB_HINTS;
        }
        if [
            LABEL_WIDTH_SELECT,
            SWEEP_RESERVE_SELECT,
            METER_VISUAL_SELECT,
            STALE_STYLE_SELECT,
        ]
        .iter()
        .any(|id| ui.state(*id).contains(termrock::StateFlags::FOCUSED))
        {
            return OPTION_HINTS;
        }
        match self.effective_section(width) {
            0 => SCROLL_HINTS,
            1 => OPTION_HINTS,
            _ => SCROLL_HINTS,
        }
    }
}
