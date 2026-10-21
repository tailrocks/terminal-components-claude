use termrock::author::PaintStyle;
use termrock::{Brand, Color, Cx, Id, Modifier, Rect, Ui, width, wrap};

use super::{Page, PageStatus, PageUpdate, chrome_ink};
use termrock::feedback::{PageFrame, SectionStrip};

const BRAND: Id = termrock::Id::root("showcase_ui::pages::chrome::chrome.brand");

const BLURB: &str = "Brand lockup · menu bar with anchored menus · status bar planes and priorities · context menu · hint layers";
const SECTIONS: [&str; 3] = ["Playground", "Options", "States"];
const SECTION_CAPTION: &str = "the live widget";
const SECTION_WIDE: u16 = 76;

/// The playground has a natural width. A narrower page scrolls it sideways
/// and stacks the card notes under the session list.
const NATURAL_W: u16 = 91;
/// The card's inner padding and the row the card starts on.
const CARD_PAD: u16 = 2;
const CARD_TOP: u16 = 2;
/// The session list column and the notes column minimum.
const LIST_W: u16 = 48;
const NOTES_MIN: u16 = 36;
/// The gap between the card's columns and between its stacked rows.
const TILE_GAP: u16 = 4;
const TILE_ROW_GAP: u16 = 1;
/// The session list rows the card reserves.
const LIST_ROWS: u16 = 6;
/// The narrow status bar row the page shows below the wide one.
const NARROW_STATUS: u16 = 64;
/// The room the scrollbars need: the bar and its gap.
const SCROLL_ROOM: u16 = 2;

const NOTES_TEXT: &str = "The status bar below sits on its own plane: three groups, no separator glyphs, and items that do not fit collapse into a … that shows them on hover, by priority — the 64-column row shows the center going first.";
const BRAND_NOTE: &str = "Brand: one lockup, accent-filled, the only accent-filled control.";
const READOUT: [&str; 3] = [
    "MenuBar · open_on Press · overflow Hide · hover_switch on · press_drag on · hide_owner_focus on",
    "wrap on · type_ahead off · nav default · dismiss light · brand lockup",
    "last MenuBarAction: —   (Opened(menu) · Chosen(menu, item) · Closed · Brand)",
];
const HINTS: [(&str, &str); 4] = [
    ("↑↓", "Move"),
    ("Shift+F10", "Context menu"),
    ("right-click", "Context menu"),
    ("Tab", "Next"),
];

const CANVAS: u32 = 0x000000;
const SURFACE: u32 = 0x111111;
const ELEVATED: u32 = 0x18181b;
const WHITE: u32 = 0xffffff;
const SECONDARY: u32 = 0xb3b3b3;
const MUTED: u32 = 0x8a8a8a;
const FAINT: u32 = 0x737373;
const RULE: u32 = 0x3a3a3a;
const ACCENT: u32 = 0x48e054;
const THUMB: u32 = 0x808080;
const TRACK: u32 = 0x474747;
const SUCCESS: u32 = 0x3fb950;
const ERROR: u32 = 0xf05d5d;
const WARNING: u32 = 0xf59e09;
const DANGER: u32 = 0xff6b6b;
const CHIP: u32 = 0x27272a;
const BLUE: u32 = 0x79b8ff;

const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// The sessions the card lists.
const SESSIONS: [(&str, &str); 4] = [
    ("  1 Claude Code (Work)", "working"),
    ("  2 Codex (Primary)", "idle"),
    ("  3 Shell", ""),
    ("  4 docs", "blocked"),
];

fn brand() -> Brand<'static> {
    Brand::new(BRAND, "app❯").clickable(true)
}

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

fn bold(fg: u32, bg: u32) -> PaintStyle {
    ink(fg, bg).add_modifier(Modifier::BOLD)
}

/// The card tiles: the session list and the notes, beside each other when
/// the card is wide enough and stacked when it is not.
struct Card {
    inner: u16,
    stacked: bool,
    notes_x: u16,
    notes_w: u16,
    lines: Vec<String>,
    content_h: u16,
}

impl Card {
    fn layout(width: u16) -> Self {
        let inner = width
            .saturating_sub(CARD_PAD.saturating_mul(2))
            .max(LIST_W.saturating_add(NOTES_MIN));
        let stacked = LIST_W.saturating_add(TILE_GAP).saturating_add(NOTES_MIN) > inner;
        let notes_w = if stacked {
            inner
        } else {
            inner.saturating_sub(LIST_W).saturating_sub(TILE_GAP)
        };
        let lines = notes_lines(notes_w);
        let notes_h = lines.len() as u16 + 3;
        let content_h = if stacked {
            LIST_ROWS + TILE_ROW_GAP + notes_h
        } else {
            LIST_ROWS.max(notes_h)
        };
        Self {
            inner,
            stacked,
            notes_x: if stacked { 0 } else { LIST_W + TILE_GAP },
            notes_w,
            lines,
            content_h,
        }
    }

    /// The card's height: the title, the rule, the content and a padding row.
    fn height(&self) -> u16 {
        3 + self.content_h
    }
}

/// The notes wrap one column short of the tile so a word never touches the
/// next column.
fn notes_lines(width: u16) -> Vec<String> {
    let width = width.max(1);
    let mut lines = wrap(NOTES_TEXT, width);
    lines.push(String::new());
    lines.extend(wrap(BRAND_NOTE, width));
    lines
}

/// The menu bar readout wraps at the playground width.
fn readout_lines(width: u16) -> Vec<String> {
    let mut lines = Vec::new();
    for line in READOUT {
        lines.extend(wrap(line, width.max(1)));
    }
    lines
}

/// One responsive playground: its width, its height, the card inside and the
/// row every following block starts on.
struct Playground {
    width: u16,
    height: u16,
    card: Card,
    readout: Vec<String>,
    readout_row: u16,
    strip_row: u16,
    status_row: u16,
    narrow_row: u16,
    hint_row: u16,
}

impl Playground {
    fn layout(width: u16) -> Self {
        let card = Card::layout(width);
        let readout = readout_lines(width);
        let readout_row = 3 + card.height();
        let strip_row = readout_row + readout.len() as u16 + 2;
        let status_row = strip_row + 3;
        let narrow_row = status_row + 3;
        let hint_row = narrow_row + 3;
        let height = hint_row + 1;
        Self {
            width,
            height,
            card,
            readout,
            readout_row,
            strip_row,
            status_row,
            narrow_row,
            hint_row,
        }
    }
}

/// The scroll view: the visible part of the playground and the room the
/// bars take.
struct Scroll {
    area: Rect,
    view: Rect,
    content: Rect,
    v_bar: bool,
    h_bar: bool,
}

impl Scroll {
    /// The content width the playground needs, and the height it needs.
    fn content_size(area: Rect) -> (u16, u16) {
        let natural = NATURAL_W.max(area.width);
        let wide = Playground::layout(natural);
        if wide.height > area.height && natural <= area.width {
            let narrow = area.width.saturating_sub(SCROLL_ROOM).max(NATURAL_W);
            if narrow <= area.width.saturating_sub(SCROLL_ROOM) {
                let narrow = Playground::layout(narrow);
                return (narrow.width, narrow.height);
            }
        }
        (wide.width, wide.height)
    }

    fn layout(area: Rect) -> Self {
        let (content_w, content_h) = Self::content_size(area);
        let (mut v, mut h) = (false, false);
        for _ in 0..3 {
            let vw = area.width.saturating_sub(if v { SCROLL_ROOM } else { 0 });
            let vh = area.height.saturating_sub(u16::from(h));
            let nv = content_h > vh;
            let nh = content_w > vw;
            if nv == v && nh == h {
                break;
            }
            (v, h) = (nv, nh);
        }
        let v = v && area.width > SCROLL_ROOM;
        let h = h && area.height > 1;
        let vw = area.width.saturating_sub(if v { SCROLL_ROOM } else { 0 });
        let vh = area.height.saturating_sub(u16::from(h));
        Self {
            area,
            view: Rect::new(area.x, area.y, vw, vh),
            content: Rect::new(area.x, area.y, content_w.max(vw), content_h.max(vh)),
            v_bar: v,
            h_bar: h,
        }
    }
}

fn paint_at(ui: &mut Ui<'_>, view: Rect, col: u16, row: u16, text: &str, style: PaintStyle) {
    let y = view.y.saturating_add(row);
    if y >= view.bottom() || col >= view.width || text.is_empty() {
        return;
    }
    let x = view.x.saturating_add(col);
    ui.paint_str(
        Rect::new(x, y, view.right().saturating_sub(x), 1),
        text,
        style,
    );
}

fn fill_row(ui: &mut Ui<'_>, view: Rect, row: u16, width: u16, style: PaintStyle) {
    let y = view.y.saturating_add(row);
    if y >= view.bottom() || width == 0 {
        return;
    }
    ui.fill(Rect::new(view.x, y, width.min(view.width), 1), style);
}

fn paint_blurb(ui: &mut Ui<'_>, area: Rect) {
    let title_w = width("Chrome");
    ui.paint_str(
        Rect {
            width: title_w.min(area.width),
            height: 1,
            ..area
        },
        "Chrome",
        bold(WHITE, CANVAS),
    );
    if area.width <= title_w.saturating_add(4) {
        return;
    }
    let meta = Rect {
        x: area.x.saturating_add(title_w).saturating_add(2),
        width: area.width.saturating_sub(title_w).saturating_sub(3),
        height: 1,
        ..area
    };
    let style = ink(MUTED, CANVAS);
    if width(BLURB) <= meta.width {
        ui.paint_str(meta, BLURB, style);
        return;
    }
    let budget = Rect {
        width: meta.width.saturating_sub(1),
        ..meta
    };
    let used = ui.paint_str(budget, BLURB, style);
    ui.paint_str(
        Rect::new(meta.x.saturating_add(used), meta.y, 1, 1),
        "…",
        style,
    );
}

fn paint_menu(ui: &mut Ui<'_>, view: Rect) {
    let _ = brand().draw(
        ui,
        Rect::new(
            view.x.saturating_add(1),
            view.y,
            view.width.saturating_sub(1),
            1,
        ),
    );
    let label = ink(SECONDARY, CANVAS);
    for (col, text) in [
        (9, " File "),
        (16, " Edit "),
        (23, " View "),
        (30, " Session "),
        (40, " Help "),
    ] {
        paint_at(ui, view, col, 0, text, label);
    }
}

/// The sessions card: the title, its meta, the list and the notes.
fn paint_card(ui: &mut Ui<'_>, view: Rect, card: &Card, scrolls: bool) {
    let top = view.y.saturating_add(2);
    if top >= view.bottom() {
        return;
    }
    let height = card
        .height()
        .min(view.bottom().saturating_sub(top))
        .min(view.height.saturating_sub(CARD_TOP));
    ui.fill(
        Rect::new(view.x, top, view.width, height),
        ink(WHITE, SURFACE),
    );
    let title = ink(SECONDARY, SURFACE);
    let faint = ink(FAINT, SURFACE);
    let rule = ink(RULE, SURFACE);
    let text = ink(WHITE, SURFACE);
    let muted = ink(MUTED, SURFACE);
    let gutter = ink(SURFACE, SURFACE);
    paint_at(ui, view, 2, 2, "Sessions", title);
    let meta = "right-click or Shift+F10 for the tab menu";
    let meta_col = card
        .inner
        .saturating_add(CARD_PAD)
        .saturating_sub(width(meta))
        .saturating_sub(u16::from(scrolls));
    if card.inner > width(meta).saturating_add(8) {
        paint_at(ui, view, meta_col, 2, meta, faint);
    }
    paint_at(ui, view, 2, 3, &"─".repeat(usize::from(card.inner)), rule);
    for (row, (label, meta)) in SESSIONS.iter().enumerate() {
        let row = row as u16 + 4;
        paint_at(ui, view, 2, row, " ", gutter);
        paint_at(ui, view, 3, row, label, text);
        if !meta.is_empty() {
            let col = CARD_PAD
                .saturating_add(LIST_W)
                .saturating_sub(width(meta))
                .saturating_sub(1);
            paint_at(ui, view, col, row, meta, muted);
        }
    }
    let notes_x = CARD_PAD.saturating_add(card.notes_x);
    let notes_y = if card.stacked {
        4 + LIST_ROWS + TILE_ROW_GAP
    } else {
        4
    };
    for (index, line) in card.lines.iter().enumerate() {
        paint_at(ui, view, notes_x, notes_y + index as u16, line, muted);
    }
    let checks_y = notes_y + card.lines.len() as u16 + 1;
    paint_at(ui, view, notes_x, checks_y, "checks ", muted);
    paint_at(
        ui,
        view,
        notes_x + 7,
        checks_y,
        "2 ok",
        ink(SUCCESS, SURFACE),
    );
    paint_at(ui, view, notes_x + 11, checks_y, " · ", muted);
    paint_at(
        ui,
        view,
        notes_x + 14,
        checks_y,
        "1 failed",
        bold(ERROR, SURFACE),
    );
    paint_at(ui, view, notes_x + 22, checks_y, " · ", muted);
    paint_at(ui, view, notes_x + 25, checks_y, "1 cancelled", muted);
    let save_y = checks_y + 1;
    paint_at(
        ui,
        view,
        notes_x + 1,
        save_y,
        "Ctrl+S",
        bold(WHITE, SURFACE),
    );
    paint_at(ui, view, notes_x + 8, save_y, "Save", muted);
    paint_at(
        ui,
        view,
        notes_x + card.notes_w.min(44).saturating_sub(8),
        save_y,
        "✓ Saved",
        ink(SUCCESS, SURFACE),
    );
}

/// The identity strip: the segments that drop by priority.
fn paint_strip(ui: &mut Ui<'_>, view: Rect, row: u16, spinner: &str, content: u16) {
    fill_row(ui, view, row, view.width, ink(WHITE, SURFACE));
    paint_at(ui, view, 2, row, "payments-platform", bold(WHITE, SURFACE));
    paint_at(ui, view, 20, row, " ◆ production ", ink(DANGER, SURFACE));
    paint_at(ui, view, 35, row, "◇ staging", ink(WARNING, SURFACE));
    paint_at(ui, view, 46, row, spinner, ink(ACCENT, SURFACE));
    paint_at(ui, view, 47, row, " deploying", ink(SECONDARY, SURFACE));
    // The right group keeps the playground's edge.
    let right = content.saturating_sub(2);
    paint_at(
        ui,
        view,
        right.saturating_sub(21),
        row,
        " ",
        ink(SECONDARY, SURFACE),
    );
    paint_at(
        ui,
        view,
        right.saturating_sub(20),
        row,
        "·",
        ink(WARNING, SURFACE),
    );
    paint_at(
        ui,
        view,
        right.saturating_sub(19),
        row,
        " 3 pending ",
        ink(SECONDARY, SURFACE),
    );
    paint_at(
        ui,
        view,
        right.saturating_sub(7),
        row,
        "main ↑2",
        ink(MUTED, SURFACE),
    );
}

/// One status bar item. The meters carry their own glyphs.
struct StatusItem {
    text: &'static str,
    priority: u8,
    group: StatusGroup,
    style: StatusStyle,
    meter: Option<Meter>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum StatusGroup {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy)]
enum StatusStyle {
    Strong,
    Secondary,
    Chip,
    Muted,
    Faint,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MeterTone {
    Level,
    Stale,
}

#[derive(Clone, Copy)]
struct Meter {
    percent: u8,
    tone: MeterTone,
}

impl StatusItem {
    fn width(&self) -> u16 {
        let text = width(self.text);
        let text = match self.style {
            StatusStyle::Chip => text + 2,
            _ => text,
        };
        if self.meter.is_some() {
            text.saturating_add(1)
                .saturating_add(METER_TRACK)
                .saturating_add(7)
        } else {
            text
        }
    }
}

const STATUS_ITEMS: [StatusItem; 8] = [
    StatusItem {
        text: "payments-platform",
        priority: 9,
        group: StatusGroup::Left,
        style: StatusStyle::Strong,
        meter: None,
    },
    StatusItem {
        text: "PR #482 · settlement backoff",
        priority: 7,
        group: StatusGroup::Left,
        style: StatusStyle::Secondary,
        meter: None,
    },
    StatusItem {
        text: "Claude Code · working · 2 tabs",
        priority: 4,
        group: StatusGroup::Center,
        style: StatusStyle::Secondary,
        meter: None,
    },
    StatusItem {
        text: "Weekly 59%",
        priority: 6,
        group: StatusGroup::Right,
        style: StatusStyle::Chip,
        meter: None,
    },
    StatusItem {
        text: "disk",
        priority: 5,
        group: StatusGroup::Right,
        style: StatusStyle::Muted,
        meter: Some(Meter {
            percent: 72,
            tone: MeterTone::Level,
        }),
    },
    StatusItem {
        text: "quota",
        priority: 5,
        group: StatusGroup::Right,
        style: StatusStyle::Muted,
        meter: Some(Meter {
            percent: 88,
            tone: MeterTone::Stale,
        }),
    },
    StatusItem {
        text: "jackin-payments-7f3a",
        priority: 3,
        group: StatusGroup::Right,
        style: StatusStyle::Chip,
        meter: None,
    },
    StatusItem {
        text: "run 9c41",
        priority: 2,
        group: StatusGroup::Right,
        style: StatusStyle::Faint,
        meter: None,
    },
];

/// The meter's track width and the room its label needs.
const METER_TRACK: u16 = 10;
const STATUS_GAP: u16 = 3;
const STATUS_EDGE: u16 = 1;

/// The status bar places its items by priority and drops the ones that do
/// not fit. The caption reports what it placed.
fn status_plan(width: u16) -> (Vec<(usize, u16)>, Option<u16>, usize) {
    let mut keep = [true; STATUS_ITEMS.len()];
    let group_width = |keep: &[bool; STATUS_ITEMS.len()]| -> [u16; 3] {
        let mut widths = [0u16; 3];
        for (index, item) in STATUS_ITEMS.iter().enumerate() {
            if !keep[index] {
                continue;
            }
            let group = match item.group {
                StatusGroup::Left => 0,
                StatusGroup::Center => 1,
                StatusGroup::Right => 2,
            };
            widths[group] = widths[group].saturating_add(item.width());
        }
        let mut counts = [0u16; 3];
        for (index, item) in STATUS_ITEMS.iter().enumerate() {
            if !keep[index] {
                continue;
            }
            let group = match item.group {
                StatusGroup::Left => 0,
                StatusGroup::Center => 1,
                StatusGroup::Right => 2,
            };
            counts[group] = counts[group].saturating_add(1);
        }
        for group in 0..3 {
            widths[group] =
                widths[group].saturating_add(counts[group].saturating_sub(1) * STATUS_GAP);
        }
        widths
    };
    let needed = |keep: &[bool; STATUS_ITEMS.len()]| -> u16 {
        let widths = group_width(keep);
        let present = widths.iter().filter(|w| **w > 0).count() as u16;
        let gaps = present.saturating_sub(1) * STATUS_GAP;
        let gone = keep.iter().any(|k| !*k);
        let more = if gone { 1 + STATUS_GAP } else { 0 };
        widths
            .iter()
            .sum::<u16>()
            .saturating_add(gaps)
            .saturating_add(more)
            .saturating_add(2 * STATUS_EDGE)
    };
    while needed(&keep) > width {
        let mut victim: Option<(u8, usize)> = None;
        for group in [1usize, 2, 0] {
            let alive: Vec<usize> = STATUS_ITEMS
                .iter()
                .enumerate()
                .filter(|(index, item)| {
                    keep[*index]
                        && matches!(
                            (item.group, group),
                            (StatusGroup::Center, 1)
                                | (StatusGroup::Right, 2)
                                | (StatusGroup::Left, 0)
                        )
                })
                .map(|(index, _)| index)
                .collect();
            if alive.is_empty() || (group == 0 && alive.len() == 1) {
                continue;
            }
            let lowest = alive
                .iter()
                .copied()
                .min_by_key(|index| (STATUS_ITEMS[*index].priority, usize::MAX - *index))
                .unwrap();
            let priority = STATUS_ITEMS[lowest].priority;
            if victim.is_none_or(|(best, _)| priority < best) {
                victim = Some((priority, lowest));
            }
        }
        match victim {
            Some((_, index)) => keep[index] = false,
            None => break,
        }
    }
    let hidden: Vec<usize> = keep
        .iter()
        .enumerate()
        .filter(|(_, k)| !**k)
        .map(|(index, _)| index)
        .collect();
    let placed = keep.iter().filter(|k| **k).count();
    let mut result = Vec::new();
    let mut x = STATUS_EDGE;
    for (index, item) in STATUS_ITEMS.iter().enumerate() {
        if !keep[index] || item.group != StatusGroup::Left {
            continue;
        }
        result.push((index, x));
        x = x.saturating_add(item.width()).saturating_add(STATUS_GAP);
    }
    let mut right = Vec::new();
    let mut rx = width.saturating_sub(STATUS_EDGE);
    for (index, item) in STATUS_ITEMS.iter().enumerate().rev() {
        if !keep[index] || item.group != StatusGroup::Right {
            continue;
        }
        rx = rx.saturating_sub(item.width());
        right.push((index, rx));
        rx = rx.saturating_sub(STATUS_GAP);
    }
    right.reverse();
    let right_start = if right.is_empty() {
        width.saturating_sub(STATUS_EDGE)
    } else {
        rx.saturating_add(STATUS_GAP)
    };
    let more = if hidden.is_empty() {
        None
    } else {
        let at = if right.is_empty() {
            right_start.saturating_sub(1)
        } else {
            right_start.saturating_sub(STATUS_GAP + 1)
        };
        Some(at)
    };
    result.extend(right);
    (result, more, placed)
}

/// The caption the status bar explains itself with.
fn status_caption(width: u16) -> String {
    let (placed, _, count) = status_plan(width);
    let mut groups = [0u16; 3];
    for (index, _) in &placed {
        let group = match STATUS_ITEMS[*index].group {
            StatusGroup::Left => 0,
            StatusGroup::Center => 1,
            StatusGroup::Right => 2,
        };
        groups[group] = groups[group].saturating_add(1);
    }
    format!(
        "places {count} of {} · Left {}, Center {}, Right {}",
        STATUS_ITEMS.len(),
        groups[0],
        groups[1],
        groups[2]
    )
}

/// The meter's track: the filled part, the empty part and the value.
fn paint_meter(ui: &mut Ui<'_>, view: Rect, row: u16, x: u16, meter: Meter) {
    const TRACK: u16 = 11;
    let filled = ((u32::from(meter.percent) * u32::from(TRACK) + 50) / 100) as u16;
    let filled = filled.min(TRACK);
    let (glyph, tone, value) = match meter.tone {
        MeterTone::Level => ("━", BLUE, WHITE),
        MeterTone::Stale => ("╍", ERROR, MUTED),
    };
    paint_at(
        ui,
        view,
        x,
        row,
        &glyph.repeat(usize::from(filled)),
        ink(tone, ELEVATED),
    );
    paint_at(
        ui,
        view,
        x + filled,
        row,
        &"─".repeat(usize::from(TRACK - filled)),
        ink(RULE, ELEVATED),
    );
    paint_at(ui, view, x + TRACK, row, " ", ink(WHITE, ELEVATED));
    paint_at(
        ui,
        view,
        x + TRACK + 1,
        row,
        &format!("{}%", meter.percent),
        ink(value, ELEVATED),
    );
    let tail = match meter.tone {
        MeterTone::Level => "  ",
        MeterTone::Stale => " !",
    };
    paint_at(ui, view, x + TRACK + 4, row, tail, ink(tone, ELEVATED));
}

/// The status bar row and its caption.
fn paint_status(ui: &mut Ui<'_>, view: Rect, row: u16, bar_width: u16) {
    fill_row(ui, view, row, bar_width, ink(WHITE, ELEVATED));
    let (placed, more, _) = status_plan(bar_width);
    for (index, x) in placed {
        let item = &STATUS_ITEMS[index];
        let style = match item.style {
            StatusStyle::Strong => bold(WHITE, ELEVATED),
            StatusStyle::Secondary => ink(SECONDARY, ELEVATED),
            StatusStyle::Chip => ink(WARNING, CHIP),
            StatusStyle::Muted => ink(MUTED, ELEVATED),
            StatusStyle::Faint => ink(FAINT, ELEVATED),
        };
        match item.meter {
            Some(meter) => {
                paint_at(ui, view, x, row, item.text, style);
                paint_at(
                    ui,
                    view,
                    x + termrock::width(item.text),
                    row,
                    " ",
                    ink(WHITE, ELEVATED),
                );
                paint_meter(ui, view, row, x + termrock::width(item.text) + 1, meter);
            }
            None => match item.style {
                StatusStyle::Chip => {
                    let text = format!(" {} ", item.text);
                    let chip = Rect::new(
                        view.x.saturating_add(x),
                        view.y.saturating_add(row),
                        termrock::width(&text),
                        1,
                    );
                    ui.fill(chip, ink(WARNING, CHIP));
                    paint_at(ui, view, x + 1, row, item.text, style);
                }
                _ => paint_at(ui, view, x, row, item.text, style),
            },
        }
    }
    if let Some(x) = more {
        paint_at(ui, view, x, row, "…", ink(MUTED, ELEVATED));
    }
}

/// The hint bar: the keys with their labels and the page's status.
fn paint_hints(ui: &mut Ui<'_>, view: Rect, row: u16) {
    let mut x = 1u16;
    for (key, label) in HINTS {
        paint_at(ui, view, x, row, key, bold(WHITE, CANVAS));
        x = x.saturating_add(width(key)).saturating_add(1);
        paint_at(ui, view, x, row, label, ink(MUTED, CANVAS));
        x = x.saturating_add(width(label)).saturating_add(2);
    }
    // The page status follows the hints with a wider gap.
    x = x.saturating_add(2);
    paint_at(ui, view, x, row, "last: ", ink(MUTED, CANVAS));
    paint_at(
        ui,
        view,
        x.saturating_add(6),
        row,
        "nothing yet",
        ink(SECONDARY, CANVAS),
    );
}

fn paint_playground(ui: &mut Ui<'_>, view: Rect, page: &Playground, spinner: &str, scrolls: bool) {
    paint_menu(ui, view);
    paint_card(ui, view, &page.card, scrolls);
    for (index, line) in page.readout.iter().enumerate() {
        paint_at(
            ui,
            view,
            0,
            page.readout_row + index as u16,
            line,
            ink(FAINT, CANVAS),
        );
    }
    paint_at(
        ui,
        view,
        0,
        page.strip_row - 1,
        "identity strip · segments drop by priority · ◆ opens the app menu",
        ink(FAINT, CANVAS),
    );
    paint_strip(ui, view, page.strip_row, spinner, page.width);
    paint_at(
        ui,
        view,
        1,
        page.status_row - 1,
        &format!(
            "status bar · StatusBar::layout {}",
            status_caption(page.width)
        ),
        ink(FAINT, CANVAS),
    );
    paint_status(ui, view, page.status_row, page.width);
    paint_at(
        ui,
        view,
        1,
        page.narrow_row - 1,
        &format!(
            "at {NARROW_STATUS} columns · StatusBar::layout {}",
            status_caption(NARROW_STATUS)
        ),
        ink(FAINT, CANVAS),
    );
    paint_status(ui, view, page.narrow_row, NARROW_STATUS);
    paint_at(
        ui,
        view,
        1,
        page.hint_row - 1,
        "hint bar · topmost layer wins:",
        ink(FAINT, CANVAS),
    );
    paint_hints(ui, view, page.hint_row);
}

/// The scrollbars: the vertical one on the right, the horizontal one at the
/// bottom. Both show how much of the playground is visible.
fn paint_scrollbars(ui: &mut Ui<'_>, scroll: &Scroll, content: &Playground) {
    if scroll.v_bar {
        let track = Rect::new(
            scroll.area.right().saturating_sub(1),
            scroll.area.y,
            1,
            scroll.view.height,
        );
        let len = thumb_len(track.height, content.height);
        for row in 0..track.height {
            let on = row < len;
            ui.paint_str(
                Rect::new(track.x, track.y.saturating_add(row), 1, 1),
                if on { "┃" } else { "┆" },
                if on {
                    ink(THUMB, CANVAS)
                } else {
                    ink(TRACK, CANVAS)
                },
            );
        }
    }
    if scroll.h_bar {
        let track = Rect::new(
            scroll.area.x,
            scroll.area.bottom().saturating_sub(1),
            scroll.area.width.saturating_sub(1),
            1,
        );
        let len = thumb_len(track.width, content.width);
        for col in 0..track.width {
            let on = col < len;
            ui.paint_str(
                Rect::new(track.x.saturating_add(col), track.y, 1, 1),
                if on { "━" } else { "┄" },
                if on {
                    ink(THUMB, CANVAS)
                } else {
                    ink(TRACK, CANVAS)
                },
            );
        }
    }
}

/// The thumb covers the share of the track the viewport shows.
fn thumb_len(track: u16, content: u16) -> u16 {
    if track == 0 || content == 0 {
        return track;
    }
    let len = (u32::from(track) * u32::from(track) / u32::from(content)) as u16;
    len.max(1).min(track)
}

#[derive(Debug, Default)]
pub struct ChromePage {
    brand_clicks: u32,
    frame: usize,
}

impl ChromePage {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Page for ChromePage {
    fn seek_paused(&mut self, frame: usize) {
        self.frame = frame;
    }

    fn title(&self) -> &'static str {
        "Chrome"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let brand = brand().update(cx);
        if brand.activated() {
            self.brand_clicks = self.brand_clicks.saturating_add(1);
        }
        let mut update = PageUpdate::from(brand.erase());
        if self.brand_clicks > 0 {
            update.status = Some(PageStatus(format!(
                "brand activations: {}",
                self.brand_clicks
            )));
        }
        update
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        PageFrame::new(self.title())
            .meta("")
            .draw_with(ui, area, |ui, body| {
                paint_blurb(ui, area);
                SectionStrip::new(&SECTIONS)
                    .caption(SECTION_CAPTION)
                    .caption_min_width(SECTION_WIDE)
                    .ink(chrome_ink())
                    .draw(ui, body);
                let playground = Rect {
                    y: body.y.saturating_add(3),
                    height: body.height.saturating_sub(3),
                    ..body
                };
                let scroll = Scroll::layout(playground);
                let page = Playground::layout(scroll.content.width);
                let spinner = SPINNER[self.frame % SPINNER.len()];
                paint_playground(
                    ui,
                    scroll.view,
                    &page,
                    spinner,
                    scroll.content.width > scroll.view.width,
                );
                paint_scrollbars(ui, &scroll, &page);
            });
    }

    fn hints(&self, _ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        &[
            ("↑↓", "Move"),
            ("m", "Context menu"),
            ("right-click", "Context menu"),
        ]
    }
}
