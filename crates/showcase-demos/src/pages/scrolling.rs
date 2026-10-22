use termrock::{
    Color, Cx, Family, FgStep, Id, Modifier, PaintStyle, Part, Rect, Role, ScrollState, Slot, Span,
    StateFlags, StylePatch, Surface, TextViewport, Ui, Variant, ViewportLine, ViewportState,
    truncate, width, wrap,
};

use showcase_data::{PROSE, SCROLL_ROWS, log_lines};

use super::{Page, PageUpdate, frame};

const LIST_VIEW: Id = termrock::Id::root("showcase_ui::pages::scrolling::scrolling.list");
const HAND_VIEW: Id = termrock::Id::root("showcase_ui::pages::scrolling::scrolling.hand");
const DIV_FITS_VIEW: Id =
    termrock::Id::root("showcase_ui::pages::scrolling::scrolling.divider.fits");
const DIV_OVER_VIEW: Id =
    termrock::Id::root("showcase_ui::pages::scrolling::scrolling.divider.over");

const BLURB: &str = "Wheel under the pointer, keys on the focused container, thumb shows where you are, edges fade toward hidden rows";
const SECTIONS: [&str; 3] = ["Playground", "Options", "States"];
const SECTION_CAPTION: &str = "the live widget";
const SECTION_WIDE: u16 = 76;
const LOG_START: usize = 400;
const LOG_CAP: usize = 2_000;
const LIST_LEN: usize = 120;
const HAND_ROWS: usize = 120;
const HAND_W: usize = 14;
const FADE_OUTER: f32 = 0.55;
const FADE_MIN: u16 = 4;
const TITLE_MIN: u16 = 4;
const CARD_PAD: u16 = 2;

const CANVAS: u32 = 0x000000;
const SURFACE: u32 = 0x111111;
const FIELD: u32 = 0x1e1e22;
const WHITE: u32 = 0xffffff;
const SECONDARY: u32 = 0xb3b3b3;
const MUTED: u32 = 0x8a8a8a;
const FAINT: u32 = 0x737373;
const RULE: u32 = 0x3a3a3a;
const ACCENT: u32 = 0x48e054;
const TAB_ACTIVE: u32 = 0x27272a;
// Gap: the mechanics panes ride TextViewport, whose bar is wired to the
// viewport family. These pins replay the scrollbar theme tokens until the
// viewport bar can select the scrollbar family.
const THUMB: u32 = 0x808080;
const TRACK: u32 = 0x474747;
const WARNING: u32 = 0xf59e09;
const ERROR: u32 = 0xf05d5d;

const SERVICES: &[&str] = &[
    "api", "auth", "billing", "cache", "db", "edge", "feeds", "gateway", "jobs", "logs",
];
const REGIONS: &[&str] = &[
    "ap-south-1",
    "ap-southeast-2",
    "ca-central-1",
    "eu-central-1",
    "eu-north-1",
    "eu-west-1",
    "eu-west-2",
    "me-south-1",
    "sa-east-1",
    "us-east-1",
    "us-east-2",
    "us-west-2",
];
const HAND_COLS: &[&str] = &["Host", "Region", "Latency", "CPU", "Memory", "Uptime"];

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

fn mix(fg: u32, bg: u32, keep: f32) -> u32 {
    let channel = |shift: u32| {
        let from = f32::from(((fg >> shift) & 0xff) as u8);
        let into = f32::from(((bg >> shift) & 0xff) as u8);
        (into + (from - into) * keep).round().clamp(0.0, 255.0) as u32
    };
    (channel(16) << 16) | (channel(8) << 8) | channel(0)
}

fn list_view() -> TextViewport<'static> {
    TextViewport::new(LIST_VIEW)
}

fn list_lines() -> Vec<ViewportLine<'static>> {
    SCROLL_ROWS
        .iter()
        .copied()
        .map(ViewportLine::Plain)
        .collect()
}

fn tab_width(label: &str) -> u16 {
    width(label).saturating_add(3)
}

fn paint_blurb(ui: &mut Ui<'_>, body: Rect) {
    let title_w = width("Scrolling");
    if body.width <= title_w.saturating_add(4) {
        return;
    }
    let area = Rect::new(
        body.x.saturating_add(title_w).saturating_add(2),
        body.y.saturating_sub(2),
        body.width.saturating_sub(title_w).saturating_sub(3),
        1,
    );
    let shown = if width(BLURB) <= area.width {
        BLURB.to_owned()
    } else {
        truncate(BLURB, area.width)
    };
    let _ = ui.paint_str(area, &shown, ink(MUTED, CANVAS));
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

fn thirds(area: Rect) -> [Rect; 3] {
    let third = area.width / 3;
    [
        Rect::new(area.x, area.y, third.saturating_sub(1), area.height),
        Rect::new(
            area.x.saturating_add(third).saturating_add(1),
            area.y,
            third.saturating_sub(1),
            area.height,
        ),
        Rect::new(
            area.x
                .saturating_add(third.saturating_mul(2))
                .saturating_add(2),
            area.y,
            area.width
                .saturating_sub(third.saturating_mul(2).saturating_add(2)),
            area.height,
        ),
    ]
}

fn bands(area: Rect) -> [Rect; 3] {
    let top_h = area.height.saturating_sub(10 + 8 + 3).max(10);
    let heights = [top_h, 10, 8];
    let mut y = area.y;
    let mut out = [Rect::new(0, 0, 0, 0); 3];
    for (index, height) in heights.into_iter().enumerate() {
        if y >= area.bottom() {
            break;
        }
        let height = height.min(area.bottom().saturating_sub(y));
        out[index] = Rect::new(area.x, y, area.width, height);
        y = y.saturating_add(height).saturating_add(1);
    }
    out
}

fn content_of(card: Rect) -> Rect {
    Rect::new(
        card.x.saturating_add(CARD_PAD),
        card.y.saturating_add(CARD_PAD),
        card.width.saturating_sub(CARD_PAD.saturating_mul(2)),
        card.height.saturating_sub(3),
    )
}

fn fit_meta(title: &str, meta: &str, text_w: u16) -> String {
    if meta.is_empty() || text_w == 0 {
        return String::new();
    }
    let title_min = width(title).min(TITLE_MIN);
    let room = text_w.saturating_sub(title_min.saturating_add(u16::from(title_min > 0)));
    let fits = |value: &str| width(value) <= room;
    if fits(meta) {
        return meta.to_owned();
    }
    let short = meta.replace(" of ", "/");
    let head = short.split(" · ").next().unwrap_or("");
    if fits(&short) {
        short
    } else if fits(head) && !head.is_empty() {
        head.to_owned()
    } else {
        truncate(&short, room)
    }
}

fn position(offset: usize, viewport: usize, total: usize) -> String {
    if viewport == 0 || total <= viewport {
        return String::new();
    }
    let end = offset.saturating_add(viewport).min(total);
    format!("{}–{} of {}", offset.saturating_add(1), end, total)
}

/// The shared scrollbar geometry: the thumb covers the share of the track
/// the viewport shows, snapped to the end once the view rests there. A
/// viewport that fits its content shows no bar at all.
fn bar_state(viewport: usize, content: usize, at_end: bool) -> Option<ScrollState> {
    let mut state = ScrollState::new(content);
    state.set_viewport(viewport);
    if !state.overflows() {
        return None;
    }
    if at_end {
        state.jump_end();
    }
    Some(state)
}

/// A shared scrollbar ink: the foreground comes from the scrollbar theme,
/// the background stays the card surface.
fn bar_ink(ui: &Ui<'_>, part: Part) -> PaintStyle {
    let surface = ui.theme_ref().bg(Surface::Surface);
    ui.resolve(
        Family::SCROLLBAR,
        Variant::DEFAULT,
        part,
        StateFlags::empty(),
    )
    .style
    .bg(surface)
}

/// A shared scrollbar glyph: the vertical pair comes from the scrollbar
/// theme.
fn bar_glyph(ui: &Ui<'_>, part: Part) -> &'static str {
    let resolved = ui.resolve(
        Family::SCROLLBAR,
        Variant::DEFAULT,
        part,
        StateFlags::empty(),
    );
    match resolved.glyph {
        Slot::Set(glyph) => ui.glyph_str(glyph),
        Slot::Inherit | Slot::Clear => {
            if part == Part::THUMB {
                "┃"
            } else {
                "┆"
            }
        }
    }
}

fn fade_row(index: usize, viewport: usize, offset: usize, total: usize) -> bool {
    if viewport < usize::from(FADE_MIN) || total <= viewport {
        return false;
    }
    let above = offset > 0 && index == 0;
    let below = offset.saturating_add(viewport) < total && index.saturating_add(1) == viewport;
    above || below
}

fn faded(color: u32, on: bool) -> u32 {
    if on {
        mix(color, SURFACE, FADE_OUTER)
    } else {
        color
    }
}

fn paint_span(ui: &mut Ui<'_>, area: Rect, text: &str, fg: u32, bg: u32) {
    if area.is_empty() {
        return;
    }
    let style = ink(fg, bg);
    ui.fill(area, style);
    let _ = ui.paint_str(area, text, style);
}

fn paint_card(ui: &mut Ui<'_>, card: Rect, title: &str, meta: &str) {
    if card.is_empty() {
        return;
    }
    ui.fill(card, ink(WHITE, SURFACE));
    let text_w = card.width.saturating_sub(CARD_PAD.saturating_mul(2));
    if text_w == 0 || card.height == 0 {
        return;
    }
    let text_x = card.x.saturating_add(CARD_PAD);
    let _ = ui.paint_str(
        Rect::new(text_x, card.y, text_w, 1),
        title,
        ink(SECONDARY, SURFACE),
    );
    let meta = fit_meta(title, meta, text_w);
    let meta_w = width(&meta);
    if meta_w > 0 && meta_w < text_w {
        let _ = ui.paint_str(
            Rect::new(
                text_x.saturating_add(text_w.saturating_sub(meta_w)),
                card.y,
                meta_w,
                1,
            ),
            &meta,
            ink(FAINT, SURFACE),
        );
    }
    if card.height > 1 {
        let _ = ui.paint_str(
            Rect::new(text_x, card.y.saturating_add(1), text_w, 1),
            &"─".repeat(usize::from(text_w)),
            ink(RULE, SURFACE),
        );
    }
}

fn paint_vbar(ui: &mut Ui<'_>, area: Rect, viewport: usize, content: usize, at_end: bool) {
    let Some(state) = bar_state(viewport, content, at_end) else {
        return;
    };
    let (start, len) = state.thumb(usize::from(area.height));
    let thumb = bar_ink(ui, Part::THUMB);
    let track = bar_ink(ui, Part::TRACK);
    let thumb_glyph = bar_glyph(ui, Part::THUMB);
    let track_glyph = bar_glyph(ui, Part::TRACK);
    for row in 0..area.height {
        let on = usize::from(row) >= start && usize::from(row) < start.saturating_add(len);
        let _ = ui.paint_str(
            Rect::new(area.x, area.y.saturating_add(row), 1, 1),
            if on { thumb_glyph } else { track_glyph },
            if on { thumb } else { track },
        );
    }
}

fn paint_hbar(ui: &mut Ui<'_>, area: Rect, viewport: usize, content: usize) {
    let Some(state) = bar_state(viewport, content, false) else {
        return;
    };
    let (start, len) = state.thumb(usize::from(area.width));
    // The horizontal pair has no theme role; the viewport's own overflow
    // bar paints the same two literals.
    let thumb = bar_ink(ui, Part::THUMB);
    let track = bar_ink(ui, Part::TRACK);
    for col in 0..area.width {
        let on = usize::from(col) >= start && usize::from(col) < start.saturating_add(len);
        let _ = ui.paint_str(
            Rect::new(area.x.saturating_add(col), area.y, 1, 1),
            if on { "━" } else { "┄" },
            if on { thumb } else { track },
        );
    }
}

fn wrapped_prose(columns: u16) -> Vec<String> {
    let mut stored = Vec::new();
    for _ in 0..3 {
        stored.extend(PROSE.split('\n').map(str::to_owned));
        stored.push(String::new());
    }
    let mut lines = Vec::new();
    for line in stored {
        if line.is_empty() {
            lines.push(String::new());
        } else {
            lines.extend(wrap(&line, columns.max(1)));
        }
    }
    lines
}

fn log_fg(line: &str) -> u32 {
    if line.contains(" error ") {
        ERROR
    } else if line.contains(" warn ") {
        WARNING
    } else {
        SECONDARY
    }
}

fn fit_cell(text: &str, columns: usize) -> String {
    let columns = u16::try_from(columns).unwrap_or(u16::MAX);
    let shown = if width(text) <= columns {
        text.to_owned()
    } else {
        truncate(text, columns)
    };
    let pad = usize::from(columns.saturating_sub(width(&shown)));
    format!("{shown}{}", " ".repeat(pad))
}

fn hand_cells(row: Option<usize>) -> String {
    let parts: Vec<String> = if let Some(index) = row {
        hand_row(index)
            .into_iter()
            .map(|cell| fit_cell(&cell, HAND_W))
            .collect()
    } else {
        HAND_COLS
            .iter()
            .copied()
            .map(|cell| fit_cell(cell, HAND_W))
            .collect()
    };
    parts.concat()
}

fn hand_visible(row: Option<usize>, columns: u16) -> String {
    let full = hand_cells(row);
    if width(&full) <= columns {
        let pad = usize::from(columns.saturating_sub(width(&full)));
        format!("{full}{}", " ".repeat(pad))
    } else {
        truncate(&full, columns)
    }
}

fn hand_row(index: usize) -> [String; 6] {
    [
        format!(
            "{}-{:02}",
            SERVICES[index % SERVICES.len()],
            index / SERVICES.len() + 1
        ),
        REGIONS[index % REGIONS.len()].to_owned(),
        format!("{} ms", 8 + (index * 37) % 90),
        format!("{}%", (index * 53) % 100),
        format!("{:.1} GiB", 1.0 + ((index * 29) % 60) as f32 / 10.0),
        format!("{} d", 1 + (index * 17) % 300),
    ]
}

fn paint_text_rows(
    ui: &mut Ui<'_>,
    area: Rect,
    rows: &[String],
    offset: usize,
    color_of: impl Fn(&str) -> u32,
    gap: bool,
) {
    if area.height == 0 || area.width < 2 {
        return;
    }
    let bar_w = 1u16;
    let gap_w = u16::from(gap);
    let text_w = area.width.saturating_sub(bar_w.saturating_add(gap_w));
    let viewport = usize::from(area.height);
    let total = rows.len();
    for index in 0..viewport {
        let source = offset.saturating_add(index);
        let Some(line) = rows.get(source) else {
            break;
        };
        let y = area
            .y
            .saturating_add(u16::try_from(index).unwrap_or(u16::MAX));
        let edge = fade_row(index, viewport, offset, total);
        let shown = if width(line) <= text_w {
            line.clone()
        } else {
            truncate(line, text_w)
        };
        paint_span(
            ui,
            Rect::new(area.x, y, text_w, 1),
            &shown,
            faded(color_of(line), edge),
            SURFACE,
        );
        if gap {
            paint_span(
                ui,
                Rect::new(area.x.saturating_add(text_w), y, 1, 1),
                " ",
                faded(WHITE, edge),
                SURFACE,
            );
        }
    }
    let at_end = offset.saturating_add(viewport) >= total && offset > 0;
    paint_vbar(
        ui,
        Rect::new(area.right().saturating_sub(1), area.y, 1, area.height),
        viewport,
        total,
        at_end,
    );
}

fn log_window(area: Rect, lines: &[String]) -> (usize, usize) {
    if area.height == 0 || area.width < 2 {
        return (0, 0);
    }
    let text_w = area.width.saturating_sub(2);
    let widest = lines
        .iter()
        .map(|line| usize::from(width(line)))
        .max()
        .unwrap_or(0);
    let h_bar = area.height > 1 && widest > usize::from(text_w);
    let viewport = usize::from(area.height.saturating_sub(u16::from(h_bar)));
    (lines.len().saturating_sub(viewport), viewport)
}

fn paint_log(ui: &mut Ui<'_>, area: Rect, lines: &[String]) -> (usize, usize) {
    if area.height == 0 || area.width < 2 {
        return (0, 0);
    }
    let text_w = area.width.saturating_sub(2);
    let widest = lines
        .iter()
        .map(|line| usize::from(width(line)))
        .max()
        .unwrap_or(0);
    let h_bar = area.height > 1 && widest > usize::from(text_w);
    let rows_h = area.height.saturating_sub(u16::from(h_bar));
    let viewport = usize::from(rows_h);
    let offset = lines.len().saturating_sub(viewport);
    paint_text_rows(
        ui,
        Rect {
            height: rows_h,
            ..area
        },
        lines,
        offset,
        log_fg,
        true,
    );
    if h_bar {
        paint_hbar(
            ui,
            Rect::new(
                area.x,
                area.y.saturating_add(rows_h),
                text_w.saturating_add(1),
                1,
            ),
            usize::from(text_w),
            widest,
        );
    }
    (offset, viewport)
}

fn paint_list_rows(ui: &mut Ui<'_>, area: Rect, offset: usize) {
    if area.height == 0 || area.width < 3 {
        return;
    }
    let text_w = area.width.saturating_sub(2);
    let viewport = usize::from(area.height);
    for index in 0..viewport {
        let source = offset.saturating_add(index);
        if source >= LIST_LEN {
            break;
        }
        let y = area
            .y
            .saturating_add(u16::try_from(index).unwrap_or(u16::MAX));
        let edge = fade_row(index, viewport, offset, LIST_LEN);
        paint_span(ui, Rect::new(area.x, y, 1, 1), " ", SURFACE, SURFACE);
        let label = format!("  {}", SCROLL_ROWS.get(source).copied().unwrap_or(""));
        paint_span(
            ui,
            Rect::new(area.x.saturating_add(1), y, text_w, 1),
            &label,
            faded(WHITE, edge),
            SURFACE,
        );
    }
    let at_end = offset.saturating_add(viewport) >= LIST_LEN && offset > 0;
    paint_vbar(
        ui,
        Rect::new(area.right().saturating_sub(1), area.y, 1, area.height),
        viewport,
        LIST_LEN,
        at_end,
    );
}

fn hand_view() -> TextViewport<'static> {
    TextViewport::new(HAND_VIEW).reserve_gutter(false)
}

/// The bar behind the hand table keeps the frozen demo inks: the viewport
/// paints its own bar family and cannot select the scrollbar one.
const HAND_BAR: &[(Part, StylePatch)] = &[
    (
        Part::CONTAINER,
        StylePatch::new().set_bg(Role::Surface(Surface::Surface)),
    ),
    (
        Part::TEXT,
        StylePatch::new().set_bg(Role::Surface(Surface::Surface)),
    ),
    (
        Part::TRACK,
        StylePatch::new().set_fg(Role::Custom(Color::Rgb(
            ((TRACK >> 16) & 0xff) as u8,
            ((TRACK >> 8) & 0xff) as u8,
            (TRACK & 0xff) as u8,
        ))),
    ),
    (
        Part::THUMB,
        StylePatch::new().set_fg(Role::Custom(Color::Rgb(
            ((THUMB >> 16) & 0xff) as u8,
            ((THUMB >> 8) & 0xff) as u8,
            (THUMB & 0xff) as u8,
        ))),
    ),
];

fn paint_hand(ui: &mut Ui<'_>, area: Rect) {
    if area.height < 3 || area.width < 2 {
        return;
    }
    let text_w = area.width.saturating_sub(1);
    let header = hand_visible(None, text_w);
    paint_span(
        ui,
        Rect::new(area.x, area.y, text_w, 1),
        &header,
        MUTED,
        SURFACE,
    );
    let body_h = area.height.saturating_sub(2);
    // The hand table scrolls through the shared viewport: the rows keep
    // their demo inks as span roles, the viewport owns the edge fade and
    // the bar. The lines stay padded so the fill matches the hand fill.
    // All rows ride along so the bar sees the full content height.
    let mut texts = Vec::with_capacity(HAND_ROWS);
    for index in 0..HAND_ROWS {
        texts.push(hand_visible(Some(index), text_w));
    }
    let spans: Vec<Span<'_>> = texts
        .iter()
        .enumerate()
        .map(|(index, text)| {
            let role = if index == 0 {
                Role::Fg(FgStep::Primary)
            } else {
                Role::Fg(FgStep::Secondary)
            };
            Span::new(text).role(role)
        })
        .collect();
    let lines: Vec<ViewportLine<'_>> = spans
        .iter()
        .map(|span| ViewportLine::Spans(std::slice::from_ref(span)))
        .collect();
    let body = Rect::new(
        area.x,
        area.y.saturating_add(1),
        text_w.saturating_add(1),
        body_h,
    );
    // The demo rows rest at the top: following would pin the tail.
    let mut state = ViewportState::default();
    state.set_follow(false);
    let _ = hand_view()
        .patch_part(HAND_BAR)
        .draw(ui, body, &state, &lines);
    let cols = (usize::from(text_w) / HAND_W).max(1);
    paint_hbar(
        ui,
        Rect::new(area.x, area.bottom().saturating_sub(1), text_w, 1),
        cols,
        HAND_COLS.len(),
    );
}

fn paint_dividers(ui: &mut Ui<'_>, area: Rect) {
    if area.height < 2 || area.width < 4 {
        return;
    }
    let half = area.width / 2;
    let rows = area.height.saturating_sub(1);
    let panes = [
        (
            Rect::new(area.x, area.y, half.saturating_sub(1), area.height),
            "fits",
            usize::from(rows),
            false,
        ),
        (
            Rect::new(
                area.x.saturating_add(half).saturating_add(1),
                area.y,
                area.width.saturating_sub(half).saturating_sub(1),
                area.height,
            ),
            "over",
            30,
            true,
        ),
    ];
    for (rect, caption, total, bar) in panes {
        if rect.width < 3 {
            continue;
        }
        let _ = ui.paint_str(
            Rect::new(rect.x, rect.y, rect.width, 1),
            caption,
            ink(MUTED, SURFACE),
        );
        let body = Rect::new(rect.x, rect.y.saturating_add(1), rect.width, rows);
        let text_w = body.width.saturating_sub(2);
        let viewport = usize::from(body.height);
        // Each pane scrolls through the shared viewport: the rows keep the
        // demo ink as a span role, the viewport owns the edge fade and the
        // bar. The fitting pane rides its exact row count so no bar appears.
        let count = if bar { total } else { viewport.min(total) };
        let mut texts = Vec::with_capacity(count);
        for index in 0..count {
            let label = format!("item {}", index.saturating_add(1));
            let shown = if width(&label) <= text_w {
                label
            } else {
                truncate(&label, text_w)
            };
            let pad = usize::from(text_w.saturating_sub(width(&shown)));
            texts.push(format!("{shown}{}", " ".repeat(pad)));
        }
        let spans: Vec<Span<'_>> = texts
            .iter()
            .map(|text| Span::new(text).role(Role::Fg(FgStep::Secondary)))
            .collect();
        let lines: Vec<ViewportLine<'_>> = spans
            .iter()
            .map(|span| ViewportLine::Spans(std::slice::from_ref(span)))
            .collect();
        let view = Rect::new(body.x, body.y, body.width, body.height);
        let mut state = ViewportState::default();
        state.set_follow(false);
        let pane = TextViewport::new(if bar { DIV_OVER_VIEW } else { DIV_FITS_VIEW })
            .reserve_gutter(false)
            .patch_part(HAND_BAR);
        let _ = pane.draw(ui, view, &state, &lines);
        if !bar {
            for index in 0..count {
                let y = body.y.saturating_add(u16::try_from(index).unwrap_or(0));
                let _ = ui.paint_str(
                    Rect::new(body.right().saturating_sub(1), y, 1, 1),
                    "│",
                    ink(RULE, SURFACE),
                );
            }
        }
    }
}

fn paint_cursor(ui: &mut Ui<'_>, area: Rect) {
    // Gap: the cursor readout is a static info list with no overflow. The
    // shared viewport always reserves a bar column, which would clip the
    // longest line by one cell, so these rows stay hand-painted.
    let open = (0..LIST_LEN).filter(|index| index % 10 != 9).count();
    let lines = [
        ("row 1 · vertical".to_owned(), SECONDARY),
        ("wrap off · skip disabled on".to_owned(), SECONDARY),
        (format!("RowFn: {open} of {LIST_LEN} open"), SECONDARY),
        ("Last: no key yet".to_owned(), SECONDARY),
        ("KeyStep: Nav · Type".to_owned(), FAINT),
        ("Moved: To · Stayed · Ignored".to_owned(), FAINT),
        ("Desktop · 1s · shortcuts first".to_owned(), SECONDARY),
    ];
    for (index, (line, color)) in lines.iter().enumerate() {
        let Ok(offset) = u16::try_from(index) else {
            break;
        };
        if offset >= area.height {
            break;
        }
        let shown = if width(line) <= area.width {
            line.clone()
        } else {
            truncate(line, area.width)
        };
        let _ = ui.paint_str(
            Rect::new(area.x, area.y.saturating_add(offset), area.width, 1),
            &shown,
            ink(*color, SURFACE),
        );
    }
}

fn paint_mechanics(ui: &mut Ui<'_>, area: Rect) {
    let hand = area.width.saturating_mul(3) / 8;
    let div = area.width / 4;
    let x1 = area.x.saturating_add(hand).saturating_add(1);
    let x2 = x1.saturating_add(div).saturating_add(1);
    let hand_card = Rect::new(area.x, area.y, hand.saturating_sub(1), area.height);
    let div_card = Rect::new(x1, area.y, div.saturating_sub(1), area.height);
    let cursor_card = Rect::new(x2, area.y, area.right().saturating_sub(x2), area.height);
    paint_card(ui, hand_card, "Hand-drawn", "");
    paint_hand(ui, content_of(hand_card));
    paint_card(ui, div_card, "Divider or bar", "");
    paint_dividers(ui, content_of(div_card));
    paint_card(ui, cursor_card, "Cursor", "");
    paint_cursor(ui, content_of(cursor_card));
}

fn paint_bar_rows(
    ui: &mut Ui<'_>,
    area: Rect,
    labels: &[&str],
    header: Option<&str>,
    header_at: u16,
    header_width: u16,
) {
    if area.is_empty() {
        return;
    }
    let body_y = if header.is_some() { 1 } else { 0 };
    if let Some(title) = header {
        let shown = fit_cell(title, usize::from(header_width));
        paint_span(
            ui,
            Rect::new(
                area.x.saturating_add(header_at),
                area.y,
                header_width.min(area.width.saturating_sub(header_at)),
                1,
            ),
            &shown,
            MUTED,
            SURFACE,
        );
    }
    let body_h = area.height.saturating_sub(body_y);
    let viewport = usize::from(body_h);
    let text_w = area.width.saturating_sub(2);
    for index in 0..viewport {
        let Some(label) = labels.get(index).copied() else {
            break;
        };
        let y = area
            .y
            .saturating_add(body_y)
            .saturating_add(u16::try_from(index).unwrap_or(0));
        let edge = fade_row(index, viewport, 0, labels.len());
        paint_span(ui, Rect::new(area.x, y, 1, 1), " ", SURFACE, SURFACE);
        paint_span(
            ui,
            Rect::new(area.x.saturating_add(1), y, text_w, 1),
            &format!("  {label}"),
            faded(WHITE, edge),
            SURFACE,
        );
    }
    if viewport > 0 && labels.len() > viewport {
        paint_vbar(
            ui,
            Rect::new(
                area.right().saturating_sub(1),
                area.y.saturating_add(body_y),
                1,
                body_h,
            ),
            viewport,
            labels.len(),
            false,
        );
    }
}

fn paint_grid(ui: &mut Ui<'_>, area: Rect) {
    if area.height < 2 || area.width < 8 {
        return;
    }
    let header_w = width("Service").saturating_add(2);
    paint_span(
        ui,
        Rect::new(area.x.saturating_add(6), area.y, header_w, 1),
        &fit_cell("Service", usize::from(header_w)),
        MUTED,
        SURFACE,
    );
    let body_h = area.height.saturating_sub(1);
    let viewport = usize::from(body_h);
    for (index, name) in SERVICES.iter().copied().enumerate().take(viewport) {
        let y = area
            .y
            .saturating_add(1)
            .saturating_add(u16::try_from(index).unwrap_or(0));
        let edge = fade_row(index, viewport, 0, SERVICES.len());
        paint_span(ui, Rect::new(area.x, y, 1, 1), " ", SURFACE, SURFACE);
        paint_span(
            ui,
            Rect::new(area.x.saturating_add(1), y, area.width.saturating_sub(2), 1),
            " ",
            faded(WHITE, edge),
            SURFACE,
        );
        paint_span(
            ui,
            Rect::new(area.x.saturating_add(3), y, 2, 1),
            &format!("{:>2}", index.saturating_add(1)),
            faded(FAINT, edge),
            SURFACE,
        );
        let _ = ui.paint_str(
            Rect::new(area.x.saturating_add(6), y, area.width.saturating_sub(7), 1),
            name,
            ink(faded(WHITE, edge), SURFACE),
        );
    }
    paint_vbar(
        ui,
        Rect::new(
            area.right().saturating_sub(1),
            area.y.saturating_add(1),
            1,
            body_h,
        ),
        viewport,
        SERVICES.len(),
        false,
    );
}

fn paint_select(ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    let _ = ui.paint_str(
        Rect::new(
            area.x.saturating_add(2),
            area.y,
            area.width.saturating_sub(2),
            1,
        ),
        "Region",
        ink(SECONDARY, SURFACE),
    );
    if area.height < 2 {
        return;
    }
    let field = Rect::new(area.x, area.y.saturating_add(1), area.width, 1);
    ui.fill(field, ink(WHITE, FIELD));
    paint_span(ui, Rect::new(field.x, field.y, 1, 1), " ", FIELD, FIELD);
    let value = REGIONS.get(5).copied().unwrap_or("");
    let _ = ui.paint_str(
        Rect::new(
            field.x.saturating_add(2),
            field.y,
            field.width.saturating_sub(4),
            1,
        ),
        value,
        ink(WHITE, FIELD),
    );
    if field.width >= 2 {
        let _ = ui.paint_str(
            Rect::new(field.right().saturating_sub(2), field.y, 1, 1),
            "▾",
            ink(SECONDARY, FIELD),
        );
    }
}

fn paint_collections(ui: &mut Ui<'_>, card: Rect) {
    paint_card(ui, card, "Type to jump · select, tree, table, grid", "");
    let inner = content_of(card);
    if inner.is_empty() {
        return;
    }
    let quarter = inner.width / 4;
    let cell = |index: u16| {
        Rect::new(
            inner.x.saturating_add(index.saturating_mul(quarter)),
            inner.y,
            quarter.saturating_sub(2),
            inner.height,
        )
    };
    paint_select(ui, cell(0));
    paint_bar_rows(ui, cell(1), SERVICES, None, 0, 0);
    let region_w = REGIONS
        .iter()
        .map(|region| width(region))
        .max()
        .unwrap_or(0);
    paint_bar_rows(ui, cell(2), REGIONS, Some("Region"), 3, region_w);
    let grid = Rect::new(
        inner.x.saturating_add(quarter.saturating_mul(3)),
        inner.y,
        inner
            .right()
            .saturating_sub(inner.x.saturating_add(quarter.saturating_mul(3))),
        inner.height,
    );
    paint_grid(ui, grid);
}

// Compact shells show the scroller band only; column widths follow breakpoints.
const FULL_MIN_W: u16 = 60;
const COLS_80_MIN_W: u16 = 59;

fn content_narrow(card: Rect) -> Rect {
    Rect::new(
        card.x.saturating_add(1),
        card.y.saturating_add(CARD_PAD),
        card.width.saturating_sub(2),
        card.height.saturating_sub(3),
    )
}

fn narrow_columns(area: Rect) -> [Rect; 3] {
    let card_w: u16 = 15;
    let gap: u16 = 2;
    let step = card_w.saturating_add(gap);
    [
        Rect::new(area.x, area.y, card_w, area.height),
        Rect::new(area.x.saturating_add(step), area.y, card_w, area.height),
        Rect::new(
            area.x.saturating_add(step.saturating_mul(2)),
            area.y,
            card_w,
            area.height,
        ),
    ]
}

fn narrow_meta(title: &str, meta: &str, text_w: u16) -> String {
    let fitted = fit_meta(title, meta, text_w);
    if fitted.contains('…') {
        String::new()
    } else {
        fitted
    }
}

fn paint_card_narrow(ui: &mut Ui<'_>, card: Rect, title: &str, meta: &str) {
    if card.is_empty() {
        return;
    }
    ui.fill(card, ink(WHITE, SURFACE));
    let text_w = card.width.saturating_sub(2);
    if text_w == 0 || card.height == 0 {
        return;
    }
    let text_x = card.x.saturating_add(1);
    let meta = narrow_meta(title, meta, text_w);
    let meta_w = width(&meta);
    let title_w = if meta_w > 0 {
        text_w.saturating_sub(meta_w).saturating_sub(1)
    } else {
        text_w
    };
    let shown = if width(title) <= title_w {
        title.to_owned()
    } else {
        truncate(title, title_w)
    };
    let _ = ui.paint_str(
        Rect::new(text_x, card.y, title_w, 1),
        &shown,
        ink(SECONDARY, SURFACE),
    );
    if meta_w > 0 {
        let _ = ui.paint_str(
            Rect::new(
                text_x.saturating_add(text_w.saturating_sub(meta_w).saturating_sub(1)),
                card.y,
                1,
                1,
            ),
            " ",
            ink(SECONDARY, SURFACE),
        );
        let _ = ui.paint_str(
            Rect::new(
                text_x.saturating_add(text_w.saturating_sub(meta_w)),
                card.y,
                meta_w,
                1,
            ),
            &meta,
            ink(FAINT, SURFACE),
        );
    }
    if card.height > 1 {
        let _ = ui.paint_str(
            Rect::new(text_x, card.y.saturating_add(1), text_w, 1),
            &"─".repeat(usize::from(text_w)),
            ink(RULE, SURFACE),
        );
    }
}

fn paint_page_thumb(ui: &mut Ui<'_>, area: Rect) {
    if area.height == 0 || area.width < 2 {
        return;
    }
    let track = usize::from(area.height);
    let Some(state) = bar_state(1, 3, false) else {
        return;
    };
    let (start, len) = state.thumb(track);
    let canvas = ui.theme_ref().bg(Surface::Canvas);
    let resolve = |part| {
        let hit = ui.resolve(
            Family::SCROLLBAR,
            Variant::DEFAULT,
            part,
            StateFlags::empty(),
        );
        let glyph = match hit.glyph {
            Slot::Set(glyph) => ui.glyph_str(glyph),
            Slot::Inherit | Slot::Clear => {
                if part == Part::THUMB {
                    "┃"
                } else {
                    "┆"
                }
            }
        };
        (hit.style.bg(canvas), glyph)
    };
    let (thumb_style, thumb_glyph) = resolve(Part::THUMB);
    let (track_style, track_glyph) = resolve(Part::TRACK);
    let x = area.right().saturating_sub(1);
    for row in 0..area.height {
        let on = usize::from(row) >= start && usize::from(row) < start.saturating_add(len);
        let _ = ui.paint_str(
            Rect::new(x, area.y.saturating_add(row), 1, 1),
            if on { thumb_glyph } else { track_glyph },
            if on { thumb_style } else { track_style },
        );
    }
}

fn columns_80(area: Rect) -> [Rect; 3] {
    let gap = 2u16;
    let widths = [18u16, 18, 17];
    let mut x = area.x;
    let mut cols = [Rect::new(0, 0, 0, 0); 3];
    for (index, width) in widths.into_iter().enumerate() {
        cols[index] = Rect::new(x, area.y, width, area.height);
        x = x.saturating_add(width).saturating_add(gap);
    }
    cols
}

fn paint_scrollers_80(ui: &mut Ui<'_>, area: Rect, list_offset: usize, log: &[String]) {
    let cols = columns_80(area);
    let prose_area = content_narrow(cols[0]);
    let prose = wrapped_prose(prose_area.width.saturating_sub(2));
    let prose_view = usize::from(prose_area.height);
    paint_card_narrow(
        ui,
        cols[0],
        "Wrapped text",
        &position(0, prose_view, prose.len()),
    );
    paint_text_rows(ui, prose_area, &prose, 0, |_| SECONDARY, true);
    let list_area = content_narrow(cols[1]);
    let list_view_h = usize::from(list_area.height);
    paint_card_narrow(
        ui,
        cols[1],
        "Long list",
        &position(list_offset, list_view_h, LIST_LEN),
    );
    paint_list_rows(ui, list_area, list_offset);
    let log_area = content_narrow(cols[2]);
    let (log_offset, log_view) = log_window(log_area, log);
    let mut meta = position(log_offset, log_view, log.len());
    if !meta.is_empty() {
        meta = format!("{meta} · following");
    }
    paint_card_narrow(ui, cols[2], "Log", &meta);
    let _ = paint_log(ui, log_area, log);
}

fn paint_scrollers_narrow(ui: &mut Ui<'_>, area: Rect, list_offset: usize, log: &[String]) {
    let cols = narrow_columns(area);
    let prose_area = content_narrow(cols[0]);
    let prose = wrapped_prose(prose_area.width.saturating_sub(2));
    let prose_view = usize::from(prose_area.height);
    paint_card_narrow(
        ui,
        cols[0],
        "Wrapped text",
        &position(0, prose_view, prose.len()),
    );
    paint_text_rows(ui, prose_area, &prose, 0, |_| SECONDARY, true);
    let list_area = content_narrow(cols[1]);
    let list_view_h = usize::from(list_area.height);
    paint_card_narrow(
        ui,
        cols[1],
        "Long list",
        &position(list_offset, list_view_h, LIST_LEN),
    );
    paint_list_rows(ui, list_area, list_offset);
    let log_area = content_narrow(cols[2]);
    let (log_offset, log_view) = log_window(log_area, log);
    let mut meta = position(log_offset, log_view, log.len());
    if !meta.is_empty() {
        meta = format!("{meta} · following");
    }
    paint_card_narrow(ui, cols[2], "Log", &meta);
    let _ = paint_log(ui, log_area, log);
}

fn paint_scrollers(ui: &mut Ui<'_>, area: Rect, list_offset: usize, log: &[String]) {
    let cols = thirds(area);
    let prose = wrapped_prose(cols[0].width.saturating_sub(6));
    let prose_view = usize::from(content_of(cols[0]).height);
    paint_card(
        ui,
        cols[0],
        "Wrapped text",
        &position(0, prose_view, prose.len()),
    );
    paint_text_rows(ui, content_of(cols[0]), &prose, 0, |_| SECONDARY, true);
    let list_view_h = usize::from(content_of(cols[1]).height);
    paint_card(
        ui,
        cols[1],
        "Long list",
        &position(list_offset, list_view_h, LIST_LEN),
    );
    paint_list_rows(ui, content_of(cols[1]), list_offset);
    let log_area = content_of(cols[2]);
    let (log_offset, log_view) = log_window(log_area, log);
    let mut meta = position(log_offset, log_view, log.len());
    if !meta.is_empty() {
        meta = format!("{meta} · following");
    }
    paint_card(ui, cols[2], "Log", &meta);
    let _ = paint_log(ui, log_area, log);
}

#[derive(Debug)]
pub struct ScrollingPage {
    list: Vec<ViewportLine<'static>>,
    log: Vec<String>,
    list_state: ViewportState,
    last: &'static str,
}

impl ScrollingPage {
    pub fn new() -> Self {
        let mut list_state = ViewportState::default();
        list_state.set_follow(false);
        Self {
            list: list_lines(),
            log: log_lines(LOG_START),
            list_state,
            last: "top of document",
        }
    }
}

impl Default for ScrollingPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for ScrollingPage {
    fn seek_paused(&mut self, frame: usize) {
        let count = LOG_START.saturating_add(frame).min(LOG_CAP);
        self.log = log_lines(count);
    }

    fn title(&self) -> &'static str {
        "Scrolling"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let before = self.list_state.scroll().offset();
        let list = list_view().update(cx, &mut self.list_state, &self.list);
        if self.list_state.scroll().offset() != before {
            self.last = "manual scroll";
        }
        list.erase().into()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        let list_offset = self.list_state.scroll().offset();
        let log_offset = self.log.len().saturating_sub(6);
        frame(ui, area, self.title(), "", |ui, body| {
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
            let playground = Rect {
                y: top,
                height: body.bottom().saturating_sub(top),
                ..body
            };
            let slots = bands(playground);
            if area.width < COLS_80_MIN_W {
                if slots[0].height > 0 {
                    let list_area = content_narrow(narrow_columns(slots[0])[1]);
                    list_view().draw(ui, list_area, &self.list_state, &self.list);
                    paint_scrollers_narrow(ui, slots[0], list_offset, &self.log);
                }
                paint_page_thumb(ui, playground);
                return;
            }
            if area.width < FULL_MIN_W {
                if slots[0].height > 0 {
                    let list_area = content_narrow(columns_80(slots[0])[1]);
                    list_view().draw(ui, list_area, &self.list_state, &self.list);
                    paint_scrollers_80(ui, slots[0], list_offset, &self.log);
                }
                paint_page_thumb(ui, playground);
                return;
            }
            if slots[0].height > 0 {
                let list_area = content_of(thirds(slots[0])[1]);
                list_view().draw(ui, list_area, &self.list_state, &self.list);
                paint_scrollers(ui, slots[0], list_offset, &self.log);
            }
            if slots[1].height > 0 {
                paint_mechanics(ui, slots[1]);
            }
            if slots[2].height > 0 {
                paint_collections(ui, slots[2]);
            }
            if self.last != "top of document" {
                let _ = ui.paint_str(
                    Rect {
                        y: body.bottom().saturating_sub(1),
                        height: 1,
                        ..body
                    },
                    &format!(
                        "prose=0 · list={list_offset} · log={log_offset} · {}",
                        self.last
                    ),
                    ink(WHITE, CANVAS),
                );
            }
        });
    }

    fn hints(&self, _ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        &[("↑ ↓", "Scroll"), ("PgUp PgDn", "Page"), ("g G", "Ends")]
    }
}

#[cfg(test)]
mod motion_tests {
    use super::*;

    #[test]
    fn paused_seek_stops_the_log_at_two_thousand() {
        let mut page = ScrollingPage::new();
        assert_eq!(page.log.len(), 400);
        page.seek_paused(12);
        assert_eq!(page.log.len(), 412);
        page.seek_paused(1_600);
        assert_eq!(page.log.len(), 2_000);
        page.seek_paused(usize::MAX);
        assert_eq!(page.log.len(), 2_000);
    }

    #[test]
    fn wrapped_prose_matches_the_open_viewport() {
        let lines = wrapped_prose(24);
        assert_eq!(lines.len(), 120);
        assert_eq!(lines[0], "Junie works through a");
        assert_eq!(lines[6], "and reports back with a");
    }

    #[test]
    fn tail_log_and_hand_slice_match_the_snapshot() {
        let log = log_lines(2_000);
        assert_eq!(truncate(&log[1994], 24), " 737.78s  info   Compil…");
        assert_eq!(truncate(&log[1999], 24), " 739.63s  info   test a…");
        assert_eq!(hand_visible(None, 29), "Host          Region        …");
        assert_eq!(hand_visible(Some(0), 29), "api-01        ap-south-1    …");
        assert_eq!(
            fit_meta("Log", "1995–2000 of 2000 · following", 26),
            "1995–2000/2000"
        );
        assert_eq!(fit_meta("Wrapped text", "1–7 of 120", 26), "1–7 of 120");
    }
}
