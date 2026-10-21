use termrock::author::PaintStyle;
use termrock::{
    ChipBar, ChipBarAction, ChipBarState, Color, Cx, FrameRead, Id, ItemKey, Modifier, Rect,
    Response, RowUi, Select, SelectAction, SelectState, StateFlags, Ui, width,
};

use showcase_data::LANGUAGES;

use super::{Page, PageUpdate, chrome_ink};
use termrock::feedback::SectionStrip;

const CHIPS: Id = termrock::Id::root("showcase_ui::pages::chips::chips.filters");
const SELECT: Id = termrock::Id::root("showcase_ui::pages::chips::chips.language");
const FILTERS: &[&str] = &[
    "Open",
    "Assigned",
    "Needs review",
    "Blocked",
    "Mine",
    "Recent",
];

const BLURB: &str = "Removable chips, a popup select, and strips that drop what does not fit";
const SECTIONS: [&str; 3] = ["Playground", "Options", "States"];
const SECTION_CAPTION: &str = "the live widget";
const SECTION_WIDE: u16 = 76;

const CANVAS: u32 = 0x000000;
const SURFACE: u32 = 0x111111;
const WHITE: u32 = 0xffffff;
const SECONDARY: u32 = 0xb3b3b3;
const MUTED: u32 = 0x8a8a8a;
const FAINT: u32 = 0x737373;
const RULE: u32 = 0x3a3a3a;
const CHIP_BG: u32 = 0x27272a;
const FIELD_BG: u32 = 0x1e1e22;
const THUMB: u32 = 0x808080;
const TRACK: u32 = 0x474747;
const DISABLED: u32 = 0x4d4d4d;
const ERROR: u32 = 0xf05d5d;
const SUCCESS: u32 = 0x3fb950;
const WARNING: u32 = 0xf59e09;

const CONTENT_W: u16 = 118;
const CONTENT_H: u16 = 39;
const SCROLL_ROOM: u16 = 2;
const CARD_PAD: u16 = 2;

const STREAMS_META: &str = "not removable · 1\u{2013}9 switch a stream";
const STRIP_NOTE: &str =
    "the same strip at 44 columns: low-priority segments leave first, from the right";

fn chip_key(value: &&'static str) -> ItemKey {
    ItemKey::text(value)
}

fn chip_row(value: &&'static str, row: &mut RowUi<'_>) {
    row.label(value);
}

fn chips() -> ChipBar<
    'static,
    &'static str,
    impl Fn(&&'static str) -> ItemKey,
    impl Fn(&&'static str, &mut RowUi<'_>),
> {
    ChipBar::new(CHIPS)
        .key(chip_key)
        .row(chip_row)
        .select_mode(termrock::SelectMode::Multi)
        .closable(false)
}

fn select() -> Select<'static, &'static str> {
    Select::new(SELECT).placeholder("Choose language")
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

fn paint_in(ui: &mut Ui<'_>, view: Rect, x: u16, y: u16, text: &str, style: PaintStyle) {
    if y < view.y || y >= view.bottom() || x >= view.right() || text.is_empty() {
        return;
    }
    ui.paint_str(
        Rect::new(x, y, view.right().saturating_sub(x), 1),
        text,
        style,
    );
}

fn fill_in(ui: &mut Ui<'_>, view: Rect, rect: Rect, style: PaintStyle) {
    let rect = rect.intersection(view);
    if !rect.is_empty() {
        ui.fill(rect, style);
    }
}

struct ScrollSplit {
    view: Rect,
    vertical: bool,
    horizontal: bool,
}

fn scroll_split(area: Rect) -> ScrollSplit {
    let (mut vertical, mut horizontal) = (false, false);
    for _ in 0..3 {
        let view_w = area
            .width
            .saturating_sub(if vertical { SCROLL_ROOM } else { 0 });
        let view_h = area.height.saturating_sub(u16::from(horizontal));
        let next_v = CONTENT_H > view_h;
        let next_h = CONTENT_W > view_w;
        if (next_v, next_h) == (vertical, horizontal) {
            break;
        }
        vertical = next_v;
        horizontal = next_h;
    }
    let vertical = vertical && area.width > SCROLL_ROOM;
    let horizontal = horizontal && area.height > 1;
    let view = Rect {
        width: area
            .width
            .saturating_sub(if vertical { SCROLL_ROOM } else { 0 }),
        height: area.height.saturating_sub(u16::from(horizontal)),
        ..area
    };
    ScrollSplit {
        view,
        vertical,
        horizontal,
    }
}

fn thumb_len(viewport: u16, track: u16, content: u16) -> u16 {
    if content <= viewport || track == 0 {
        return track;
    }
    let len = u32::from(viewport).saturating_mul(u32::from(track)) / u32::from(content);
    u16::try_from(len.max(1)).unwrap_or(u16::MAX).min(track)
}

fn paint_scrollbars(ui: &mut Ui<'_>, area: Rect, split: &ScrollSplit) {
    if split.vertical {
        let bar = Rect::new(area.right().saturating_sub(1), area.y, 1, split.view.height);
        let thumb = thumb_len(split.view.height, bar.height, CONTENT_H);
        for row in 0..bar.height {
            let on = row < thumb;
            ui.paint_str(
                Rect::new(bar.x, bar.y.saturating_add(row), 1, 1),
                if on { "┃" } else { "┆" },
                if on {
                    ink(THUMB, CANVAS)
                } else {
                    ink(TRACK, CANVAS)
                },
            );
        }
    }
    if split.horizontal {
        let width = area.width.saturating_sub(u16::from(split.vertical));
        let bar = Rect::new(area.x, area.bottom().saturating_sub(1), width, 1);
        let thumb = thumb_len(split.view.width, bar.width, CONTENT_W);
        for col in 0..bar.width {
            let on = col < thumb;
            ui.paint_str(
                Rect::new(bar.x.saturating_add(col), bar.y, 1, 1),
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

fn paint_card(ui: &mut Ui<'_>, view: Rect, card: Rect, title: &str, meta: &str) {
    fill_in(ui, view, card, ink(WHITE, SURFACE));
    let text_x = card.x.saturating_add(CARD_PAD);
    let text_w = card.width.saturating_sub(CARD_PAD.saturating_mul(2));
    paint_in(ui, view, text_x, card.y, title, ink(SECONDARY, SURFACE));
    if !meta.is_empty() && text_w > width(meta) {
        let meta_x = text_x.saturating_add(text_w).saturating_sub(width(meta));
        paint_in(ui, view, meta_x, card.y, meta, ink(FAINT, SURFACE));
    }
    if card.height > 1 && text_w > 0 {
        paint_in(
            ui,
            view,
            text_x,
            card.y.saturating_add(1),
            &"─".repeat(usize::from(text_w)),
            ink(RULE, SURFACE),
        );
    }
}

struct ChipPaint {
    label: &'static str,
    removable: bool,
    enabled: bool,
    error: bool,
}

fn chip_width(chip: &ChipPaint) -> u16 {
    width(chip.label)
        .saturating_add(3)
        .saturating_add(if chip.removable { 2 } else { 0 })
}

fn paint_chip(ui: &mut Ui<'_>, view: Rect, x: u16, y: u16, chip: &ChipPaint) {
    let w = chip_width(chip);
    let fg = if chip.error {
        ERROR
    } else if chip.enabled {
        WHITE
    } else {
        SECONDARY
    };
    let body = if chip.enabled {
        bold(fg, CHIP_BG)
    } else {
        ink(fg, CHIP_BG)
    };
    fill_in(ui, view, Rect::new(x, y, w, 1), body);
    let gutter = if chip.enabled {
        bold(CHIP_BG, CHIP_BG)
    } else {
        ink(CHIP_BG, CHIP_BG)
    };
    paint_in(ui, view, x, y, " ", gutter);
    paint_in(ui, view, x.saturating_add(1), y, chip.label, body);
    if chip.removable {
        paint_in(
            ui,
            view,
            x.saturating_add(2).saturating_add(width(chip.label)),
            y,
            "×",
            ink(SECONDARY, CHIP_BG).remove_modifier(Modifier::BOLD),
        );
    }
}

fn paint_lead(ui: &mut Ui<'_>, view: Rect, x: u16, y: u16, lead: &str) -> u16 {
    let text = format!(" {lead} ");
    let w = width(&text);
    paint_in(ui, view, x, y, &text, ink(SECONDARY, SURFACE));
    w.saturating_add(1)
}

fn paint_filters(ui: &mut Ui<'_>, view: Rect, card: Rect, last: &str) {
    paint_card(ui, view, card, "Filters", "3 active");
    let y = card.y.saturating_add(2);
    let mut x = card.x.saturating_add(CARD_PAD);
    x = x.saturating_add(paint_lead(ui, view, x, y, "match all ▾"));
    for chip in [
        ChipPaint {
            label: "status = 'pending'",
            removable: true,
            enabled: true,
            error: false,
        },
        ChipPaint {
            label: "total > 100",
            removable: true,
            enabled: true,
            error: false,
        },
        ChipPaint {
            label: "country in (DE, FR)",
            removable: true,
            enabled: false,
            error: false,
        },
        ChipPaint {
            label: "amount > 'abc'",
            removable: true,
            enabled: true,
            error: true,
        },
    ] {
        paint_chip(ui, view, x, y, &chip);
        x = x.saturating_add(chip_width(&chip)).saturating_add(1);
    }
    let add = "+ Add filter";
    let add_w = width(add).saturating_add(2);
    let inner_right = card.x.saturating_add(card.width).saturating_sub(CARD_PAD);
    if x.saturating_add(add_w) <= inner_right {
        let body = ink(SECONDARY, SURFACE);
        fill_in(ui, view, Rect::new(x, y, add_w, 1), body);
        paint_in(ui, view, x, y, " ", ink(SURFACE, SURFACE));
        paint_in(ui, view, x.saturating_add(1), y, add, body);
    }
    paint_in(
        ui,
        view,
        card.x.saturating_add(CARD_PAD),
        y.saturating_add(2),
        &format!("last action: {last}"),
        ink(MUTED, SURFACE),
    );
}

fn paint_streams(ui: &mut Ui<'_>, view: Rect, card: Rect) {
    paint_card(ui, view, card, "Streams", STREAMS_META);
    let y = card.y.saturating_add(2);
    let mut x = card.x.saturating_add(CARD_PAD);
    x = x.saturating_add(paint_lead(ui, view, x, y, "streams"));
    for (label, enabled) in [
        ("api", true),
        ("worker", true),
        ("db", true),
        ("cron", false),
    ] {
        let chip = ChipPaint {
            label,
            removable: false,
            enabled,
            error: false,
        };
        paint_chip(ui, view, x, y, &chip);
        x = x.saturating_add(chip_width(&chip)).saturating_add(1);
    }
}

fn paint_field(
    ui: &mut Ui<'_>,
    view: Rect,
    cell: Rect,
    label: &str,
    value: &str,
    help: &str,
    disabled: bool,
) {
    let label_fg = if disabled { FAINT } else { SECONDARY };
    paint_in(
        ui,
        view,
        cell.x.saturating_add(2),
        cell.y,
        label,
        ink(label_fg, SURFACE),
    );
    if cell.height < 2 {
        return;
    }
    let field = Rect::new(cell.x, cell.y.saturating_add(1), cell.width, 1);
    let value_fg = if disabled { DISABLED } else { WHITE };
    fill_in(ui, view, field, ink(value_fg, FIELD_BG));
    paint_in(ui, view, field.x, field.y, " ", ink(FIELD_BG, FIELD_BG));
    paint_in(
        ui,
        view,
        field.x.saturating_add(2),
        field.y,
        value,
        ink(value_fg, FIELD_BG),
    );
    let chevron = field.right().saturating_sub(2);
    let chevron_fg = if disabled { DISABLED } else { SECONDARY };
    paint_in(ui, view, chevron, field.y, "▾", ink(chevron_fg, FIELD_BG));
    if cell.height >= 3 && !help.is_empty() {
        paint_in(
            ui,
            view,
            cell.x.saturating_add(2),
            cell.y.saturating_add(2),
            help,
            ink(MUTED, SURFACE),
        );
    }
}

fn paint_selects(ui: &mut Ui<'_>, view: Rect, card: Rect) {
    paint_card(ui, view, card, "Selects", "");
    let inner_x = card.x.saturating_add(CARD_PAD);
    let inner_w = card.width.saturating_sub(CARD_PAD.saturating_mul(2));
    let y = card.y.saturating_add(2);
    let third = inner_w / 3;
    let cells = [
        (inner_x, third.saturating_sub(2)),
        (inner_x.saturating_add(third), third.saturating_sub(2)),
        (
            inner_x.saturating_add(third.saturating_mul(2)),
            inner_w.saturating_sub(third.saturating_mul(2)),
        ),
    ];
    let specs = [
        ("Sort by", "created_at", "Applies to the next query", false),
        ("Page size", "50", "", false),
        ("Engine", "PostgreSQL", "Fixed by the connection", true),
    ];
    for ((x, w), (label, value, help, disabled)) in cells.into_iter().zip(specs) {
        paint_field(
            ui,
            view,
            Rect::new(x, y, w, 3),
            label,
            value,
            help,
            disabled,
        );
    }
}

struct Seg {
    text: &'static str,
    fg: u32,
    bold: bool,
    priority: u8,
}

fn paint_segments(ui: &mut Ui<'_>, view: Rect, area: Rect, left: &[Seg], right: &[Seg]) {
    if area.is_empty() {
        return;
    }
    let edge = 2u16;
    let sep = 2u16;
    let mut keep_l = vec![true; left.len()];
    let mut keep_r = vec![true; right.len()];
    let total = |keep_l: &[bool], keep_r: &[bool]| -> u16 {
        let left_w: u16 = left
            .iter()
            .zip(keep_l)
            .filter(|(_, keep)| **keep)
            .map(|(seg, _)| width(seg.text).saturating_add(sep))
            .sum();
        let right_w: u16 = right
            .iter()
            .zip(keep_r)
            .filter(|(_, keep)| **keep)
            .map(|(seg, _)| width(seg.text).saturating_add(sep))
            .sum();
        let hidden = keep_l.iter().chain(keep_r.iter()).any(|keep| !keep);
        let more = if hidden { 1 + sep } else { 0 };
        left_w
            .saturating_add(right_w)
            .saturating_add(more)
            .saturating_sub(sep)
            .saturating_add(edge.saturating_mul(2))
    };
    while total(&keep_l, &keep_r) > area.width {
        let mut best: Option<(u8, bool, usize)> = None;
        for (index, seg) in left.iter().enumerate() {
            if keep_l[index] && best.is_none_or(|found| seg.priority < found.0) {
                best = Some((seg.priority, true, index));
            }
        }
        for (index, seg) in right.iter().enumerate() {
            if keep_r[index] && best.is_none_or(|found| seg.priority <= found.0) {
                best = Some((seg.priority, false, index));
            }
        }
        match best {
            Some((_, true, index)) => keep_l[index] = false,
            Some((_, false, index)) => keep_r[index] = false,
            None => break,
        }
    }
    let style = |seg: &Seg| {
        if seg.bold {
            bold(seg.fg, SURFACE)
        } else {
            ink(seg.fg, SURFACE)
        }
    };
    let mut x = area.x.saturating_add(edge);
    for (seg, keep) in left.iter().zip(&keep_l) {
        if !keep {
            continue;
        }
        paint_in(ui, view, x, area.y, seg.text, style(seg));
        x = x.saturating_add(width(seg.text)).saturating_add(sep);
    }
    let mut rx = area.right().saturating_sub(edge);
    for (seg, keep) in right.iter().zip(&keep_r).rev() {
        if !keep {
            continue;
        }
        let seg_w = width(seg.text);
        rx = rx.saturating_sub(seg_w);
        paint_in(ui, view, rx, area.y, seg.text, style(seg));
        rx = rx.saturating_sub(sep);
    }
    let hidden = keep_l.iter().chain(keep_r.iter()).any(|keep| !keep);
    if hidden && area.width > edge.saturating_mul(2) {
        let at = if right.is_empty() {
            x.min(area.right().saturating_sub(edge).saturating_sub(1))
        } else {
            rx.saturating_sub(1).max(area.x.saturating_add(edge))
        };
        paint_in(ui, view, at, area.y, "…", ink(MUTED, SURFACE));
    }
}

fn strip_sides() -> ([Seg; 5], [Seg; 3]) {
    (
        [
            Seg {
                text: "▪",
                fg: SUCCESS,
                bold: false,
                priority: 9,
            },
            Seg {
                text: "Acme",
                fg: WHITE,
                bold: true,
                priority: 9,
            },
            Seg {
                text: "◆ production",
                fg: WHITE,
                bold: true,
                priority: 8,
            },
            Seg {
                text: "acme_prod › public",
                fg: SECONDARY,
                bold: false,
                priority: 6,
            },
            Seg {
                text: "safe",
                fg: WHITE,
                bold: true,
                priority: 7,
            },
        ],
        [
            Seg {
                text: "3 pending",
                fg: WARNING,
                bold: false,
                priority: 5,
            },
            Seg {
                text: "truecolor · 120×40",
                fg: MUTED,
                bold: false,
                priority: 2,
            },
            Seg {
                text: "? help",
                fg: MUTED,
                bold: false,
                priority: 3,
            },
        ],
    )
}

fn paint_strip(ui: &mut Ui<'_>, view: Rect, card: Rect) {
    paint_card(ui, view, card, "Segment strip", "");
    let inner = Rect {
        x: card.x.saturating_add(CARD_PAD),
        y: card.y.saturating_add(2),
        width: card.width.saturating_sub(CARD_PAD.saturating_mul(2)),
        height: card.height.saturating_sub(3),
    };
    let (left, right) = strip_sides();
    paint_segments(ui, view, Rect { height: 1, ..inner }, &left, &right);
    let narrow = inner.width.min(44);
    paint_segments(
        ui,
        view,
        Rect::new(inner.x, inner.y.saturating_add(2), narrow, 1),
        &left,
        &right,
    );
    paint_in(
        ui,
        view,
        inner.x,
        inner.y.saturating_add(3),
        STRIP_NOTE,
        ink(FAINT, SURFACE),
    );
}

fn place(origin: Rect, y: u16, height: u16, width: u16) -> Rect {
    Rect::new(origin.x, origin.y.saturating_add(y), width, height)
}

fn wrap_words(text: &str, columns: usize) -> Vec<String> {
    let columns = columns.max(1);
    let mut lines = Vec::new();
    let mut line = String::new();
    let mut line_w = 0u16;
    for word in text.split_whitespace() {
        let word_w = width(word);
        if line.is_empty() {
            line = word.to_owned();
            line_w = word_w;
            continue;
        }
        if usize::from(line_w.saturating_add(1).saturating_add(word_w)) <= columns {
            line.push(' ');
            line.push_str(word);
            line_w = line_w.saturating_add(1).saturating_add(word_w);
        } else {
            lines.push(std::mem::take(&mut line));
            line = word.to_owned();
            line_w = word_w;
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

fn pair_widths(total: u16) -> (u16, u16) {
    let share = total.saturating_sub(2) / 2;
    let right = total.saturating_sub(2).saturating_sub(share);
    (share, right)
}

fn paint_centered(ui: &mut Ui<'_>, view: Rect, area: Rect, y: u16, text: &str, style: PaintStyle) {
    let text_w = width(text);
    let x = area.x.saturating_add(area.width.saturating_sub(text_w) / 2);
    paint_in(ui, view, x, y, text, style);
}

fn paint_props(ui: &mut Ui<'_>, view: Rect, card: Rect) {
    paint_card(ui, view, card, "Properties", "");
    let content_x = card.x.saturating_add(CARD_PAD);
    let mut y = card.y.saturating_add(2);
    let content_w = card.width.saturating_sub(CARD_PAD.saturating_mul(2));
    if content_w == 0 {
        return;
    }
    let labels = ["Engine", "Host", "Environment", "Safe Mode", "Last used"];
    let mut label_w = 2u16;
    for label in labels {
        label_w = label_w.max(width(label).saturating_add(2));
    }
    let value_columns = usize::from(content_w.saturating_sub(label_w)).max(4);
    let facts = [
        ("Engine", "PostgreSQL 16.3", false, false),
        ("Host", "prod-db-1.acme.io:5432", false, false),
        ("Environment", "production", false, false),
        (
            "Safe Mode",
            "Writes ask for confirmation and a deliberate acknowledgement.",
            true,
            false,
        ),
        ("Last used", "1 hour ago", false, true),
    ];
    for (label, value, wraps, muted) in facts {
        let value_fg = if muted { MUTED } else { WHITE };
        let lines = if wraps {
            wrap_words(value, value_columns)
        } else {
            vec![value.to_owned()]
        };
        for (index, line) in lines.iter().enumerate() {
            if index == 0 {
                paint_in(ui, view, content_x, y, label, ink(MUTED, SURFACE));
            }
            paint_in(
                ui,
                view,
                content_x.saturating_add(label_w),
                y,
                line,
                ink(value_fg, SURFACE),
            );
            y = y.saturating_add(1);
        }
    }
}

fn paint_empty(ui: &mut Ui<'_>, view: Rect, card: Rect) {
    paint_card(ui, view, card, "Empty state", "");
    let content = Rect::new(
        card.x.saturating_add(CARD_PAD),
        card.y.saturating_add(2),
        card.width.saturating_sub(CARD_PAD.saturating_mul(2)),
        card.height.saturating_sub(3),
    );
    if content.is_empty() {
        return;
    }
    let title = "No results yet";
    let hint = "A title and one hint, centred in whatever is left";
    let hints = wrap_words(hint, usize::from(content.width.saturating_sub(4).max(8)));
    let total = 1u16.saturating_add(hints.len() as u16).saturating_add(1);
    let y0 = content
        .y
        .saturating_add(content.height.saturating_sub(total) / 2);
    paint_centered(ui, view, content, y0, title, ink(MUTED, SURFACE));
    for (index, line) in hints.iter().enumerate() {
        paint_centered(
            ui,
            view,
            content,
            y0.saturating_add(2).saturating_add(index as u16),
            line,
            ink(FAINT, SURFACE),
        );
    }
}

fn paint_blurb(ui: &mut Ui<'_>, area: Rect) {
    let title_w = width("Chips & selects");
    if area.width <= title_w.saturating_add(4) {
        return;
    }
    let meta = Rect::new(
        area.x.saturating_add(title_w).saturating_add(2),
        area.y,
        area.width.saturating_sub(title_w).saturating_sub(3),
        1,
    );
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

fn paint_playground(ui: &mut Ui<'_>, area: Rect, last: &str) {
    if area.is_empty() {
        return;
    }
    let split = scroll_split(area);
    let view = split.view;
    let card_w = if split.horizontal {
        CONTENT_W
    } else {
        view.width.max(CONTENT_W)
    };
    paint_filters(ui, view, place(view, 0, 6, card_w), last);
    paint_streams(ui, view, place(view, 7, 5, card_w));
    paint_selects(ui, view, place(view, 13, 8, card_w));
    paint_strip(ui, view, place(view, 22, 7, card_w));
    let (left_w, right_w) = pair_widths(card_w);
    paint_props(ui, view, place(view, 30, 9, left_w));
    if right_w > 0 {
        paint_empty(
            ui,
            view,
            Rect::new(
                view.x.saturating_add(left_w).saturating_add(2),
                view.y.saturating_add(30),
                right_w,
                9,
            ),
        );
    }
    paint_scrollbars(ui, area, &split);
}

#[derive(Debug, Default)]
pub struct ChipsPage {
    chip_state: ChipBarState,
    select_state: SelectState,
    last: &'static str,
}

impl ChipsPage {
    pub fn new() -> Self {
        let mut chip_state = ChipBarState::default();
        chip_state.checked_mut().insert(ItemKey::text("Open"));
        chip_state.checked_mut().insert(ItemKey::text("Assigned"));
        Self {
            chip_state,
            select_state: SelectState::default(),
            last: "no filter selected",
        }
    }
}

impl Page for ChipsPage {
    fn title(&self) -> &'static str {
        "Chips & selects"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let mut result = Response::ignored();
        let chips = chips().update(cx, &mut self.chip_state, FILTERS);
        if let Some(action) = chips.action_ref() {
            self.last = match action {
                ChipBarAction::Toggled(_) => "filter toggled",
                ChipBarAction::Activated(_) => "filter activated",
                ChipBarAction::Closed(_) => "filter closed",
                ChipBarAction::AddRequested => "filter add requested",
            };
        }
        result |= chips.erase();
        let select = select().update(cx, &mut self.select_state, LANGUAGES);
        if select
            .action_ref()
            .is_some_and(|action| matches!(action, SelectAction::Chose(_)))
        {
            self.last = "language selected";
        }
        result |= select.erase();
        result.into()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        ui.fill(area, ink(WHITE, CANVAS));
        if area.is_empty() {
            return;
        }
        ui.paint_str(
            Rect { height: 1, ..area },
            self.title(),
            bold(WHITE, CANVAS),
        );
        paint_blurb(ui, area);
        if area.height < 3 {
            return;
        }
        let body = Rect {
            y: area.y.saturating_add(2),
            height: area.height.saturating_sub(2),
            ..area
        };
        SectionStrip::new(&SECTIONS)
            .caption(SECTION_CAPTION)
            .caption_min_width(SECTION_WIDE)
            .ink(chrome_ink())
            .draw(
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
        let playground_area = Rect {
            y: top,
            height: body.bottom().saturating_sub(top),
            ..body
        };
        let (chip_area, rest) = termrock::layout::split_v(playground_area, 4);
        chips().draw(ui, chip_area, &self.chip_state, FILTERS);
        let (select_area, _) = termrock::layout::split_v(rest, 3);
        select().draw(ui, select_area, &self.select_state, LANGUAGES);
        let action = if self.last == "no filter selected" {
            "nothing yet"
        } else {
            self.last
        };
        paint_playground(ui, playground_area, action);
    }

    fn hints(&self, ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        if ui.state(CHIPS).contains(StateFlags::FOCUSED) {
            &[
                ("← →", "Move"),
                ("Space", "Toggle"),
                ("Enter", "Edit / add"),
                ("x", "Remove"),
                ("X", "Clear all"),
            ]
        } else if ui.state(SELECT).contains(StateFlags::FOCUSED) {
            &[("Enter", "Open"), ("↑ ↓", "Choose"), ("Esc", "Close")]
        } else {
            &[]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ChipsPage, Page};
    use termrock::{App, Color, Cx, Response, Theme, Ui};
    use termrock_test_support::Harness;

    struct Host;

    impl App for Host {
        fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            ChipsPage::new().draw(ui, ui.full());
        }
    }

    #[test]
    fn narrow_blurb_ends_on_the_ellipsis() {
        let h = Harness::new(Host, Theme::junie(), 51, 6);
        assert_eq!(h.cell(48, 0).symbol(), ",");
        assert_eq!(h.cell(49, 0).symbol(), "…");
        assert_eq!(h.cell(49, 0).fg, Color::Rgb(0x8a, 0x8a, 0x8a));
        assert_eq!(h.cell(49, 0).bg, Color::Rgb(0, 0, 0));
        assert_eq!(h.cell(50, 0).symbol(), " ");
    }

    #[test]
    fn wide_playground_shows_the_lower_cards_and_add_stop() {
        let h = Harness::new(Host, Theme::junie(), 134, 46);
        let text = h.text();
        assert!(text.contains("+ Add filter"), "{text}");
        assert!(text.contains("3 active"), "{text}");
        assert!(text.contains("Properties"), "{text}");
        assert!(text.contains("Empty state"), "{text}");
        assert!(text.contains("No results yet"), "{text}");
        assert!(text.contains("acknowledgement."), "{text}");
        let (x, y) = h.find("No results yet").expect("empty title");
        assert_eq!(h.cell(x, y).fg, Color::Rgb(0x8a, 0x8a, 0x8a));
        assert_eq!(h.cell(x, y).bg, Color::Rgb(0x11, 0x11, 0x11));
        let (px, py) = h.find("PostgreSQL").expect("engine value");
        let chevron = (px..134)
            .map(|x| h.cell(x, py))
            .find(|cell| cell.symbol() == "▾");
        let chevron = chevron.expect("disabled chevron");
        assert_eq!(chevron.fg, Color::Rgb(0x4d, 0x4d, 0x4d));
        assert_eq!(chevron.bg, Color::Rgb(0x1e, 0x1e, 0x22));
    }
}
