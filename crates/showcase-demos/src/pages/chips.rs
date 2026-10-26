use std::cell::Cell;

use termrock::author::PaintStyle;
use termrock::navigation::TabAffixes;
use termrock::{
    ChipBar, ChipBarAction, ChipBarState, Color, Cx, Focusability, FrameRead, Id, Intent, ItemKey,
    KeyCode, Modifier, Part, PartRef, Rect, ReferenceState, ReferenceTarget, Response, Role, RowUi,
    ScopeId, ScrollRegion, ScrollState, Select, SelectAction, SelectMode, SelectState, StateFlags,
    StylePatch, Tabs, TabsState, Toggle, Ui, author::VerticalWindow, width,
};

use showcase_data::LANGUAGES;

use super::{Page, PageUpdate, frame};

const CHIPS: Id = termrock::Id::root("showcase_ui::pages::chips::chips.filters");
const SELECT: Id = termrock::Id::root("showcase_ui::pages::chips::chips.language");
const PAGE_TABS: Id = termrock::Id::root("showcase_ui::pages::chips.page_tabs");
const OPTIONS_SCROLL: Id = termrock::Id::root("showcase_ui::pages::chips.options_scroll");
const STATES_SCROLL: Id = termrock::Id::root("showcase_ui::pages::chips.states_scroll");
const CHIP_MODE: Id = termrock::Id::root("showcase_ui::pages::chips.option.chip_mode");
const CHIP_WIDTH: Id = termrock::Id::root("showcase_ui::pages::chips.option.chip_width");
const CHIP_CLOSE: Id = termrock::Id::root("showcase_ui::pages::chips.option.chip_close");
const POPUP_ROWS: Id = termrock::Id::root("showcase_ui::pages::chips.option.popup_rows");
const SELECT_DISABLED: Id = termrock::Id::root("showcase_ui::pages::chips.option.select_disabled");
const OPTION_CONTROLS: [Id; 5] = [
    CHIP_MODE,
    CHIP_WIDTH,
    CHIP_CLOSE,
    POPUP_ROWS,
    SELECT_DISABLED,
];
const PAGE_SECTIONS: &[&str] = &["Playground", "Options", "States"];
const SECTION_CAPTION: &str = "the live widget";
const SECTION_WIDE: u16 = 76;
const CHIP_MODE_ITEMS: &[&str] = &["Multi toggle", "Single activate"];
const CHIP_WIDTH_ITEMS: &[&str] = &["Full width", "60 columns", "40 columns"];
const POPUP_ROWS_ITEMS: &[&str] = &["Theme default", "4 rows", "8 rows", "12 rows"];
const STATE_CHIPS: &[&str] = &[
    "Open",
    "Assigned",
    "Needs review",
    "Blocked",
    "Mine",
    "Recent",
];
const ERROR_CHIP: &str = "amount > 'abc'";
const ERROR_CHIPS: &[&str] = &[ERROR_CHIP];
const UNICODE_CHIPS: &[&str] = &["北京", "e\u{301}", "👩‍👩‍👧‍👦"];
const OPTIONS_ROWS: usize = OPTION_CONTROLS.len();
const STATES_ROWS: usize = 18;
const ERROR_LABEL_PATCH: StylePatch =
    StylePatch::new().set_fg(Role::Custom(Color::Rgb(240, 93, 93)));
const FILTERS: &[&str] = &[
    "Open",
    "Assigned",
    "Needs review",
    "Blocked",
    "Mine",
    "Recent",
];

const BLURB: &str = "Removable chips, a popup select, and strips that drop what does not fit";

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
const ACCENT: u32 = 0x48e054;
const TAB_ACTIVE: u32 = 0x27272a;

#[derive(Clone, Copy, Debug, Default)]
struct DrawVisibility {
    page_tabs: bool,
    chips: bool,
    select: bool,
    options_scroll: bool,
    states_scroll: bool,
    option_controls: [bool; OPTION_CONTROLS.len()],
}

impl DrawVisibility {
    fn contains(self, id: Id) -> bool {
        if id == PAGE_TABS {
            return self.page_tabs;
        }
        if id == CHIPS {
            return self.chips;
        }
        if id == SELECT {
            return self.select;
        }
        if id == OPTIONS_SCROLL {
            return self.options_scroll;
        }
        if id == STATES_SCROLL {
            return self.states_scroll;
        }
        options_control_row(id).is_some_and(|row| self.option_controls[row])
    }

    fn set(&mut self, id: Id, visible: bool) {
        if id == PAGE_TABS {
            self.page_tabs = visible;
        } else if id == CHIPS {
            self.chips = visible;
        } else if id == SELECT {
            self.select = visible;
        } else if id == OPTIONS_SCROLL {
            self.options_scroll = visible;
        } else if id == STATES_SCROLL {
            self.states_scroll = visible;
        } else if let Some(row) = options_control_row(id) {
            self.option_controls[row] = visible;
        }
    }
}

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

fn error_chip_row(value: &&'static str, row: &mut RowUi<'_>) {
    if *value == ERROR_CHIP {
        row.label_patched(value, &ERROR_LABEL_PATCH);
    } else {
        row.label(value);
    }
}

fn chips(
    mode: SelectMode,
    closable: bool,
) -> ChipBar<
    'static,
    &'static str,
    impl Fn(&&'static str) -> ItemKey,
    impl Fn(&&'static str, &mut RowUi<'_>),
> {
    ChipBar::new(CHIPS)
        .key(chip_key)
        .row(chip_row)
        .select_mode(mode)
        .closable(closable)
}

fn language_select(popup_rows: Option<u16>, disabled: bool) -> Select<'static, &'static str> {
    let select = Select::new(SELECT)
        .placeholder("Choose language")
        .disabled(disabled);
    match popup_rows {
        Some(rows) => select.popup_rows(rows),
        None => select,
    }
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

fn tab_width(label: &str) -> u16 {
    width(label).saturating_add(3)
}

fn section_tab_affixes(_section: &&str) -> TabAffixes {
    TabAffixes::default().trailing_padding(1)
}

fn intersects_overflow_hit(layout: &termrock::navigation::tabs::TabLayout, area: Rect) -> bool {
    (0..=2).any(|index| {
        layout
            .part_bounds(PartRef::item(Part::OVERFLOW, ItemKey::index(index)))
            .is_some_and(|(_, visible)| !visible.intersection(area).is_empty())
    })
}

fn paint_section_strip(
    ui: &mut Ui<'_>,
    area: Rect,
    active_section: usize,
    focused_cursor: Option<usize>,
    layout: &termrock::navigation::tabs::TabLayout,
) {
    if area.is_empty() {
        return;
    }
    let active_section = active_section.min(PAGE_SECTIONS.len().saturating_sub(1));
    let mut tab_end = area.x;
    for row in layout.rows() {
        let Some(label) = PAGE_SECTIONS.get(row.index).copied() else {
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
        let caption_width = width(SECTION_CAPTION);
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
            let _ = ui.paint_str(caption_area, SECTION_CAPTION, ink(FAINT, CANVAS));
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
        if !visible.is_empty() {
            let _ = ui.paint_str(
                Rect::new(visible.x, rule_y, visible.width, 1),
                &"━".repeat(usize::from(visible.width)),
                ink(ACCENT, CANVAS),
            );
        }
    }
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

fn option_select_index(state: &SelectState) -> usize {
    match state.value() {
        Some(ItemKey::Index(index)) => index,
        _ => 0,
    }
}

fn selected_option(index: usize) -> SelectState {
    let mut state = SelectState::default();
    state.set_value(Some(ItemKey::index(index)));
    state
}

fn options_control_row(id: Id) -> Option<usize> {
    OPTION_CONTROLS
        .iter()
        .position(|candidate| *candidate == id)
}

fn drain_hidden_option_intents(cx: &mut Cx<'_>, id: Id) {
    for _ in cx.intents(id) {}
}

fn page_scroll_keys(cx: &mut Cx<'_>, id: Id, scroll: &mut ScrollState) -> Response<()> {
    let mut response = Response::ignored();
    for intent in cx.intents(id) {
        let Intent::Key(key) = intent else {
            continue;
        };
        let before = scroll.offset();
        match key.code {
            KeyCode::PageUp => scroll.page_up(),
            KeyCode::PageDown => scroll.page_down(),
            _ => continue,
        }
        response |= if scroll.offset() == before {
            Response::consumed()
        } else {
            Response::changed()
        };
    }
    response
}

#[derive(Debug)]
pub struct ChipsPage {
    chip_state: ChipBarState,
    select_state: SelectState,
    page_tabs: TabsState,
    active_filters: Vec<&'static str>,
    chip_mode_state: SelectState,
    chip_width_state: SelectState,
    popup_rows_state: SelectState,
    close_chips: bool,
    select_disabled: bool,
    options_scroll: ScrollState,
    states_scroll: ScrollState,
    options_focus: Option<Id>,
    last: &'static str,
    // Page hints run before this draw's hit registry commits.
    draw_visibility: Cell<DrawVisibility>,
}

impl ChipsPage {
    pub fn new() -> Self {
        let mut chip_state = ChipBarState::default();
        chip_state.checked_mut().insert(ItemKey::text("Open"));
        chip_state.checked_mut().insert(ItemKey::text("Assigned"));
        Self {
            chip_state,
            select_state: SelectState::default(),
            page_tabs: TabsState::default(),
            active_filters: FILTERS.to_vec(),
            chip_mode_state: selected_option(0),
            chip_width_state: selected_option(0),
            popup_rows_state: selected_option(0),
            close_chips: false,
            select_disabled: false,
            options_scroll: ScrollState::default(),
            states_scroll: ScrollState::default(),
            options_focus: None,
            last: "no filter selected",
            draw_visibility: Cell::new(DrawVisibility::default()),
        }
    }

    fn mark_draw_visible(&self, id: Id, area: Rect, clip: Rect) {
        let mut visible = self.draw_visibility.get();
        visible.set(id, !area.intersection(clip).is_empty());
        self.draw_visibility.set(visible);
    }

    fn section(&self) -> usize {
        match self.page_tabs.active().or(self.page_tabs.cursor()) {
            Some(ItemKey::Index(index)) => index.min(2),
            _ => 0,
        }
    }

    fn chip_mode(&self) -> SelectMode {
        if option_select_index(&self.chip_mode_state) == 1 {
            SelectMode::Single
        } else {
            SelectMode::Multi
        }
    }

    fn chip_width(&self, available: u16) -> u16 {
        match option_select_index(&self.chip_width_state) {
            1 => available.min(60),
            2 => available.min(40),
            _ => available,
        }
    }

    fn popup_rows(&self) -> Option<u16> {
        match option_select_index(&self.popup_rows_state) {
            1 => Some(4),
            2 => Some(8),
            3 => Some(12),
            _ => None,
        }
    }

    fn language_select(&self) -> Select<'static, &'static str> {
        language_select(self.popup_rows(), self.select_disabled)
    }

    fn update_options(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = Response::ignored();
        if cx.area(CHIP_MODE).is_some() || self.chip_mode_state.is_open() || cx.is_open(CHIP_MODE) {
            response |= Select::new(CHIP_MODE)
                .popup_rows(4)
                .update(cx, &mut self.chip_mode_state, CHIP_MODE_ITEMS)
                .erase();
        } else {
            drain_hidden_option_intents(cx, CHIP_MODE);
        }
        if cx.area(CHIP_WIDTH).is_some()
            || self.chip_width_state.is_open()
            || cx.is_open(CHIP_WIDTH)
        {
            response |= Select::new(CHIP_WIDTH)
                .popup_rows(3)
                .update(cx, &mut self.chip_width_state, CHIP_WIDTH_ITEMS)
                .erase();
        } else {
            drain_hidden_option_intents(cx, CHIP_WIDTH);
        }
        if cx.area(CHIP_CLOSE).is_some() {
            response |= Toggle::new(CHIP_CLOSE, "Enabled")
                .on(self.close_chips)
                .update(cx, &mut self.close_chips)
                .erase();
        } else {
            drain_hidden_option_intents(cx, CHIP_CLOSE);
        }
        if cx.area(POPUP_ROWS).is_some()
            || self.popup_rows_state.is_open()
            || cx.is_open(POPUP_ROWS)
        {
            response |= Select::new(POPUP_ROWS)
                .popup_rows(4)
                .update(cx, &mut self.popup_rows_state, POPUP_ROWS_ITEMS)
                .erase();
        } else {
            drain_hidden_option_intents(cx, POPUP_ROWS);
        }
        if cx.area(SELECT_DISABLED).is_some() {
            response |= Toggle::new(SELECT_DISABLED, "Disabled")
                .on(self.select_disabled)
                .update(cx, &mut self.select_disabled)
                .erase();
        } else {
            drain_hidden_option_intents(cx, SELECT_DISABLED);
        }
        response
    }

    fn draw_playground(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let (mut chip_area, rest) = termrock::layout::split_v(area, 4);
        chip_area.width = self.chip_width(chip_area.width);
        chips(self.chip_mode(), self.close_chips).draw(
            ui,
            chip_area,
            &self.chip_state,
            &self.active_filters,
        );
        self.mark_draw_visible(CHIPS, chip_area, ui.full());
        let (select_area, _) = termrock::layout::split_v(rest, 3);
        self.language_select()
            .draw(ui, select_area, &self.select_state, LANGUAGES);
        self.mark_draw_visible(SELECT, select_area, ui.full());
        let action = if self.last == "no filter selected" {
            "nothing yet"
        } else {
            self.last
        };
        paint_playground(ui, area, action);
    }

    fn draw_options(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let sample_height = area.height.min(3);
        let sample_area = Rect {
            height: sample_height,
            ..area
        };
        if sample_height > 0 {
            let _ = ui.paint_str(
                Rect::new(area.x, area.y, area.width, 1),
                "Live sample · options below apply here",
                ui.surface_style(),
            );
        }
        if sample_height > 1 {
            let chip_area = Rect::new(
                area.x,
                area.y.saturating_add(1),
                self.chip_width(area.width),
                1,
            );
            chips(self.chip_mode(), self.close_chips).draw(
                ui,
                chip_area,
                &self.chip_state,
                &self.active_filters,
            );
            self.mark_draw_visible(CHIPS, chip_area, ui.full());
        }
        if sample_height > 2 {
            let select_area = Rect::new(area.x, area.y.saturating_add(2), area.width, 1);
            self.language_select()
                .draw(ui, select_area, &self.select_state, LANGUAGES);
            self.mark_draw_visible(SELECT, select_area, ui.full());
        }
        let options_area = Rect {
            y: sample_area.bottom(),
            height: area.bottom().saturating_sub(sample_area.bottom()),
            ..area
        };
        if options_area.is_empty() {
            return;
        }
        let view = ScrollRegion::new(OPTIONS_SCROLL).draw(
            ui,
            options_area,
            &self.options_scroll,
            OPTIONS_ROWS,
        );
        self.mark_draw_visible(OPTIONS_SCROLL, view, ui.full());
        if view.is_empty() {
            return;
        }
        ui.register_focus_only(OPTIONS_SCROLL, Focusability::Focusable);
        let projected = ScrollRegion::view(&self.options_scroll, view, OPTIONS_ROWS);
        let first_row = projected.offset().min(OPTIONS_ROWS);
        let logical_height = u16::try_from(OPTIONS_ROWS).unwrap_or(u16::MAX);
        let visible = Rect {
            height: view.height.min(logical_height),
            ..view
        };
        let Ok(window) = VerticalWindow::new(visible, OPTIONS_ROWS, first_row) else {
            return;
        };
        let source_rows = window.visible_source_range();
        let labels = [
            "Chip mode",
            "Chip width",
            "Close chips",
            "Popup rows",
            "Language Select",
        ];
        let label_width = view.width.saturating_sub(22).min(20);
        let control_x = view.x.saturating_add(label_width).saturating_add(1);
        let control_width = view.right().saturating_sub(control_x);
        for (source_y, label) in labels.into_iter().enumerate() {
            if source_y < source_rows.start || source_y >= source_rows.end {
                if control_width > 0 {
                    ui.register_focus_only(OPTION_CONTROLS[source_y], Focusability::Focusable);
                }
                continue;
            }
            let Some(y) = window.physical_row_for_source(source_y) else {
                continue;
            };
            if label_width > 0 {
                let _ = ui.paint_str(
                    Rect::new(view.x, y, label_width, 1),
                    label,
                    ui.surface_style(),
                );
            }
            let control_area = Rect::new(control_x, y, control_width, 1);
            if control_area.is_empty() {
                continue;
            }
            match source_y {
                0 => {
                    let _ = Select::new(CHIP_MODE).popup_rows(2).draw(
                        ui,
                        control_area,
                        &self.chip_mode_state,
                        CHIP_MODE_ITEMS,
                    );
                }
                1 => {
                    let _ = Select::new(CHIP_WIDTH).popup_rows(3).draw(
                        ui,
                        control_area,
                        &self.chip_width_state,
                        CHIP_WIDTH_ITEMS,
                    );
                }
                2 => {
                    let _ = Toggle::new(CHIP_CLOSE, "Enabled")
                        .on(self.close_chips)
                        .draw(ui, control_area);
                }
                3 => {
                    let _ = Select::new(POPUP_ROWS).popup_rows(4).draw(
                        ui,
                        control_area,
                        &self.popup_rows_state,
                        POPUP_ROWS_ITEMS,
                    );
                }
                4 => {
                    let _ = Toggle::new(SELECT_DISABLED, "Disabled")
                        .on(self.select_disabled)
                        .draw(ui, control_area);
                }
                _ => (),
            }
            self.mark_draw_visible(OPTION_CONTROLS[source_y], control_area, ui.full());
        }
    }

    fn draw_states(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let view =
            ScrollRegion::new(STATES_SCROLL).draw(ui, area, &self.states_scroll, STATES_ROWS);
        self.mark_draw_visible(STATES_SCROLL, view, ui.full());
        if view.is_empty() {
            return;
        }
        ui.register_focus_only(STATES_SCROLL, Focusability::Focusable);
        let projected = ScrollRegion::view(&self.states_scroll, view, STATES_ROWS);
        let first_row = projected.offset().min(STATES_ROWS);
        let logical_height = u16::try_from(STATES_ROWS).unwrap_or(u16::MAX);
        let visible = Rect {
            height: view.height.min(logical_height),
            ..view
        };
        let Ok(window) = VerticalWindow::new(visible, STATES_ROWS, first_row) else {
            return;
        };
        let source_rows = window.visible_source_range();
        let label_width = view.width.saturating_sub(28).min(18);
        let control_x = view.x.saturating_add(label_width).saturating_add(1);
        let control_width = view.right().saturating_sub(control_x);
        let labels = [
            "Chip rest",
            "Chip hovered",
            "Chip pressed",
            "Chip focused",
            "Chip focus + hover",
            "Chip selected",
            "Chip disabled",
            "Chip error",
            "Chip empty",
            "Overflow",
            "Chip narrow",
            "Unicode",
            "Select rest",
            "Select chosen",
            "Select hovered",
            "Select pressed",
            "Select focused",
            "Select disabled",
        ];
        let mut chip_state = ChipBarState::default();
        chip_state.set_cursor(0, ItemKey::text(STATE_CHIPS[0]));
        let mut selected_state = chip_state.clone();
        selected_state
            .checked_mut()
            .insert(ItemKey::text(STATE_CHIPS[0]));
        let select_state = selected_option(0);
        let empty_select_state = SelectState::default();
        let empty_chip_state = ChipBarState::default();
        for (source_y, label) in labels.into_iter().enumerate() {
            if source_y < source_rows.start || source_y >= source_rows.end {
                continue;
            }
            let Some(y) = window.physical_row_for_source(source_y) else {
                continue;
            };
            if label_width > 0 {
                let _ = ui.paint_str(
                    Rect::new(view.x, y, label_width, 1),
                    label,
                    ui.surface_style(),
                );
            }
            let mut sample_area = Rect::new(control_x, y, control_width, 1);
            if sample_area.is_empty() {
                continue;
            }
            match source_y {
                0..=6 => {
                    let (state, target, disabled) = match source_y {
                        1 => (
                            &chip_state,
                            Some(ReferenceTarget::new(CHIPS, ReferenceState::HOVERED)),
                            false,
                        ),
                        2 => (
                            &chip_state,
                            Some(ReferenceTarget::new(CHIPS, ReferenceState::PRESSED)),
                            false,
                        ),
                        3 => (
                            &chip_state,
                            Some(ReferenceTarget::new(CHIPS, ReferenceState::FOCUS_VISIBLE)),
                            false,
                        ),
                        4 => (
                            &chip_state,
                            Some(ReferenceTarget::new(
                                CHIPS,
                                ReferenceState::FOCUS_VISIBLE | ReferenceState::HOVERED,
                            )),
                            false,
                        ),
                        5 => (&selected_state, None, false),
                        6 => (&chip_state, None, true),
                        _ => (&chip_state, None, false),
                    };
                    ui.reference(target, |ui| {
                        chips(SelectMode::Multi, true).disabled(disabled).draw(
                            ui,
                            sample_area,
                            state,
                            STATE_CHIPS,
                        );
                    });
                }
                7 => {
                    let items = ERROR_CHIPS;
                    let mut specimen = ChipBarState::default();
                    specimen.set_cursor(0, ItemKey::text(ERROR_CHIP));
                    ui.reference(None, |ui| {
                        ChipBar::new(CHIPS)
                            .key(chip_key)
                            .row(error_chip_row)
                            .select_mode(SelectMode::Multi)
                            .closable(true)
                            .draw(ui, sample_area, &specimen, items);
                    });
                }
                8 => {
                    ui.reference(None, |ui| {
                        chips(SelectMode::Multi, true).draw(
                            ui,
                            sample_area,
                            &empty_chip_state,
                            &[],
                        );
                    });
                }
                9..=11 => {
                    sample_area.width = sample_area.width.min(28);
                    if source_y == 10 {
                        sample_area.width = sample_area.width.min(8);
                    }
                    let items = if source_y == 11 {
                        UNICODE_CHIPS
                    } else {
                        STATE_CHIPS
                    };
                    let mut specimen = ChipBarState::default();
                    if !items.is_empty() {
                        specimen.set_cursor(0, ItemKey::text(items[0]));
                    }
                    ui.reference(None, |ui| {
                        chips(SelectMode::Multi, true).draw(ui, sample_area, &specimen, items);
                    });
                }
                12..=17 => {
                    let target = match source_y {
                        14 => Some(ReferenceTarget::new(SELECT, ReferenceState::HOVERED)),
                        15 => Some(ReferenceTarget::new(SELECT, ReferenceState::PRESSED)),
                        16 => Some(ReferenceTarget::new(SELECT, ReferenceState::FOCUS_VISIBLE)),
                        _ => None,
                    };
                    let state = if matches!(source_y, 13 | 16 | 17) {
                        &select_state
                    } else {
                        &empty_select_state
                    };
                    let disabled = source_y == 17;
                    ui.reference(target, |ui| {
                        language_select(None, disabled).draw(ui, sample_area, state, LANGUAGES);
                    });
                }
                _ => (),
            }
        }
    }
}

impl Default for ChipsPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for ChipsPage {
    fn title(&self) -> &'static str {
        "Chips & selects"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let old_section = self.section();
        let mut response = Tabs::new(PAGE_TABS)
            .item_affixes(section_tab_affixes)
            .update(cx, &mut self.page_tabs, PAGE_SECTIONS)
            .erase();
        let section = self.section();
        if section != old_section {
            self.options_scroll.scroll_to(0);
            self.states_scroll.scroll_to(0);
            self.options_focus = None;
        }

        if section <= 1 {
            let chips = chips(self.chip_mode(), self.close_chips).update(
                cx,
                &mut self.chip_state,
                &self.active_filters,
            );
            let action = chips.action_ref().copied();
            response |= chips.erase();
            if let Some(action) = action {
                self.last = match action {
                    ChipBarAction::Toggled(_) => "filter toggled",
                    ChipBarAction::Activated(_) => "filter activated",
                    ChipBarAction::Closed(_) => "filter closed",
                    ChipBarAction::AddRequested => "filter add requested",
                };
                if let ChipBarAction::Closed(key) = action {
                    let previous_len = self.active_filters.len();
                    self.active_filters
                        .retain(|filter| ItemKey::text(filter) != key);
                    if previous_len != self.active_filters.len()
                        && self.chip_state.checked().contains(key)
                    {
                        self.chip_state.checked_mut().toggle(key);
                    }
                }
            }
        }

        if section <= 1 || self.select_state.is_open() || cx.is_open(SELECT) {
            let select = self
                .language_select()
                .update(cx, &mut self.select_state, LANGUAGES);
            if select
                .action_ref()
                .is_some_and(|action| matches!(action, SelectAction::Chose(_)))
            {
                self.last = "language selected";
            }
            response |= select.erase();
        }

        response |= self.update_options(cx);

        let options_len = cx
            .layout(OPTIONS_SCROLL)
            .map_or(0, |layout| layout.content_len);
        response |=
            ScrollRegion::new(OPTIONS_SCROLL).update(cx, &mut self.options_scroll, options_len);
        response |= page_scroll_keys(cx, OPTIONS_SCROLL, &mut self.options_scroll);
        let focused_option = (section == 1)
            .then(|| cx.focused_in_scope(ScopeId::ROOT))
            .flatten()
            .filter(|id| options_control_row(*id).is_some());
        let focus_changed = focused_option != self.options_focus;
        self.options_focus = focused_option;
        let wheel_delta = cx
            .intents(OPTIONS_SCROLL)
            .filter_map(|intent| match intent {
                Intent::Wheel { delta, .. } => Some(i32::from(delta)),
                _ => None,
            })
            .fold(0i32, i32::saturating_add);
        if wheel_delta == 0
            && focus_changed
            && let Some(id) = focused_option
            && let Some(row) = options_control_row(id)
        {
            let _ = ScrollRegion::new(OPTIONS_SCROLL).prepare(
                cx,
                &mut self.options_scroll,
                OPTIONS_ROWS,
            );
            let _ = self.options_scroll.reveal_span(row, 1);
        }

        let states_len = cx
            .layout(STATES_SCROLL)
            .map_or(0, |layout| layout.content_len);
        response |=
            ScrollRegion::new(STATES_SCROLL).update(cx, &mut self.states_scroll, states_len);
        response |= page_scroll_keys(cx, STATES_SCROLL, &mut self.states_scroll);
        response.into()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        self.draw_visibility.set(DrawVisibility::default());
        frame(ui, area, self.title(), BLURB, |ui, body| {
            if area.height >= 2 {
                ui.fill(
                    Rect {
                        y: area.y.saturating_add(1),
                        height: 1,
                        ..area
                    },
                    ink(WHITE, CANVAS),
                );
            }
            ui.fill(body, ink(WHITE, CANVAS));
            if body.is_empty() {
                return;
            }
            let tabs_area = Rect {
                height: body.height.min(2),
                ..body
            };
            let content_top = body.y.saturating_add(3);
            let content = Rect {
                y: content_top,
                height: body.bottom().saturating_sub(content_top),
                ..body
            };
            match self.section() {
                0 => self.draw_playground(ui, content),
                1 => self.draw_options(ui, content),
                _ => self.draw_states(ui, content),
            }

            let tabs = Tabs::new(PAGE_TABS).item_affixes(section_tab_affixes);
            let previous =
                ui.layout(PAGE_TABS)
                    .map(|facts| termrock::navigation::tabs::TabWindowReceipt {
                        viewport_len: facts.viewport_len,
                        content_len: facts.content_len,
                    });
            let layout =
                tabs.layout_default(tabs_area, area, &self.page_tabs, PAGE_SECTIONS, previous);
            if tabs.draw_layout(ui, &layout).is_err() {
                return;
            }
            self.mark_draw_visible(PAGE_TABS, layout.visible_area, ui.full());
            let focused_cursor = ui
                .state(PAGE_TABS)
                .contains(StateFlags::FOCUSED)
                .then(|| self.page_tabs.cursor())
                .flatten()
                .and_then(|key| match key {
                    ItemKey::Index(index) => Some(index),
                    _ => None,
                });
            paint_section_strip(ui, tabs_area, self.section(), focused_cursor, &layout);
        });
    }

    fn hints(&self, ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        let visible = self.draw_visibility.get();
        if ui.state(PAGE_TABS).contains(StateFlags::FOCUSED) && visible.contains(PAGE_TABS) {
            return &[("← →", "Section"), ("1–3", "Jump"), ("Enter", "Activate")];
        }
        match self.section() {
            0 if ui.state(CHIPS).contains(StateFlags::FOCUSED) && visible.contains(CHIPS) => {
                if self.chip_mode() == SelectMode::Multi {
                    if self.close_chips {
                        &[
                            ("← →", "Move"),
                            ("Space", "Toggle"),
                            ("Enter", "Activate"),
                            ("Delete", "Remove"),
                        ]
                    } else {
                        &[("← →", "Move"), ("Space", "Toggle"), ("Enter", "Activate")]
                    }
                } else if self.close_chips {
                    &[("← →", "Move"), ("Enter", "Activate"), ("Delete", "Remove")]
                } else {
                    &[("← →", "Move"), ("Enter", "Activate")]
                }
            }
            0 if ui.state(SELECT).contains(StateFlags::FOCUSED) && visible.contains(SELECT) => &[
                ("Enter", "Open / choose"),
                ("↑ ↓", "Choose"),
                ("Esc", "Close"),
            ],
            1 if [CHIP_MODE, CHIP_WIDTH, POPUP_ROWS]
                .iter()
                .any(|id| ui.state(*id).contains(StateFlags::FOCUSED) && visible.contains(*id)) =>
            {
                &[("Enter", "Open / choose"), ("↑ ↓", "Choose")]
            }
            1 if [CHIP_CLOSE, SELECT_DISABLED]
                .iter()
                .any(|id| ui.state(*id).contains(StateFlags::FOCUSED) && visible.contains(*id)) =>
            {
                &[("Space", "Toggle"), ("Enter", "Toggle")]
            }
            1 if ui.state(OPTIONS_SCROLL).contains(StateFlags::FOCUSED)
                && visible.contains(OPTIONS_SCROLL) =>
            {
                &[("Wheel", "Scroll"), ("PgUp / PgDn", "Page")]
            }
            2 if ui.state(STATES_SCROLL).contains(StateFlags::FOCUSED)
                && visible.contains(STATES_SCROLL) =>
            {
                &[("Wheel", "Scroll"), ("PgUp / PgDn", "Page")]
            }
            _ => &[],
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::{
        CHIP_CLOSE, CHIP_MODE, CHIP_WIDTH, CHIPS, ChipsPage, OPTIONS_SCROLL, PAGE_TABS, POPUP_ROWS,
        Page, SELECT, SELECT_DISABLED, STATES_SCROLL,
    };
    use termrock::{
        App, Axis, Color, Cx, GlyphRole, ItemKey, KeyCode, Modifier, Part, PartRef, Rect, Response,
        StateFlags, Theme, Ui,
    };
    use termrock_test_support::Harness;

    struct Host {
        page: ChipsPage,
        area: Option<Rect>,
        captured_hints: Cell<&'static [(&'static str, &'static str)]>,
    }

    impl Host {
        fn new(area: Option<Rect>) -> Self {
            Self {
                page: ChipsPage::new(),
                area,
                captured_hints: Cell::new(&[]),
            }
        }
    }

    impl App for Host {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            self.page.update(cx).response
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let area = self.area.unwrap_or_else(|| ui.full());
            self.page.draw(ui, area);
            self.captured_hints.set(self.page.hints(ui));
        }
    }

    fn host(width: u16, height: u16) -> Harness<Host> {
        Harness::new(Host::new(None), Theme::junie(), width, height)
    }

    fn compact_host(area: Rect) -> Harness<Host> {
        Harness::new(Host::new(Some(area)), Theme::junie(), 120, 40)
    }

    fn click_tab(h: &mut Harness<Host>, label: &str) {
        let (x, y) = h.find(label).expect("section tab is visible");
        let _ = h.click(x, y);
    }

    fn open_section(h: &mut Harness<Host>, section: usize) {
        assert!(h.tab_to(PAGE_TABS));
        let current = h.app().page.section();
        if section > current {
            for _ in current..section {
                let _ = h.key(KeyCode::Right);
            }
        } else {
            for _ in section..current {
                let _ = h.key(KeyCode::Left);
            }
        }
        assert_eq!(h.app().page.section(), section);
    }

    #[test]
    fn narrow_blurb_ends_on_the_ellipsis() {
        let h = host(51, 6);
        assert_eq!(h.cell(48, 0).symbol(), ",");
        assert_eq!(h.cell(49, 0).symbol(), "…");
        assert_eq!(h.cell(49, 0).fg, Color::Rgb(0x8a, 0x8a, 0x8a));
        assert_eq!(h.cell(49, 0).bg, Color::Rgb(0, 0, 0));
        assert_eq!(h.cell(50, 0).symbol(), " ");
    }

    #[test]
    fn wide_playground_shows_the_lower_cards_and_add_stop() {
        let h = host(134, 46);
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

    #[test]
    fn defaults_keep_playground_selection_callbacks_and_first_focus() {
        let mut h = host(134, 46);
        let page = &h.app().page;
        assert_eq!(h.focus(), Some(CHIPS));
        assert_eq!(page.section(), 0);
        assert_eq!(page.chip_state.cursor(), Some(ItemKey::text("Open")));
        assert!(page.chip_state.checked().contains(ItemKey::text("Open")));
        assert!(
            page.chip_state
                .checked()
                .contains(ItemKey::text("Assigned"))
        );
        assert_eq!(page.active_filters.as_slice(), super::FILTERS);
        assert!(!page.close_chips);
        assert_eq!(page.chip_mode(), termrock::SelectMode::Multi);
        assert_eq!(page.select_state.value(), None);
        assert!(!page.select_state.is_open());
        assert!(h.text().contains("+ Add filter"));
        assert!(h.text().contains("3 active"));
        assert!(h.area_of(SELECT).is_some());

        let _ = h.click_part(CHIPS, PartRef::item(Part::LABEL, ItemKey::text("Blocked")));
        assert_eq!(h.app().page.last, "filter toggled");
        assert!(
            h.app()
                .page
                .chip_state
                .checked()
                .contains(ItemKey::text("Blocked"))
        );
    }

    #[test]
    fn authored_section_strip_keeps_caption_geometry_and_body_offset() {
        let mut h = host(134, 46);
        let tabs = h.area_of(PAGE_TABS).expect("native Tabs area");
        let (playground_x, playground_y) = h.find("Playground").expect("first section tab");
        let (options_x, options_y) = h.find("Options").expect("second section tab");
        let (states_x, states_y) = h.find("States").expect("third section tab");
        let (caption_x, caption_y) = h.find("the live widget").expect("authored caption");

        assert_eq!(playground_y, tabs.y);
        assert_eq!(options_y, tabs.y);
        assert_eq!(states_y, tabs.y);
        assert_eq!(caption_y, tabs.y);
        assert_eq!(options_x, playground_x + super::tab_width("Playground") + 1);
        assert_eq!(states_x, options_x + super::tab_width("Options") + 1);
        assert!(caption_x > states_x + super::tab_width("States") + 2);
        assert_eq!(
            h.cell(playground_x - 1, playground_y).bg,
            Color::Rgb(0x27, 0x27, 0x2a)
        );
        assert!(
            h.cell(playground_x, playground_y)
                .modifier
                .contains(Modifier::BOLD)
        );
        assert_eq!(h.cell(tabs.x, tabs.y + 1).symbol(), "━");
        assert_eq!(
            h.area_of(CHIPS).expect("Playground body remains live").y,
            tabs.y + 3
        );

        click_tab(&mut h, "Options");
        let (options_x, options_y) = h.find("Options").expect("selected Options tab");
        assert_eq!(
            h.cell(options_x - 1, options_y).bg,
            Color::Rgb(0x27, 0x27, 0x2a)
        );
        assert_eq!(h.app().page.section(), 1);
        assert!(h.diagnostics().is_empty());
    }

    #[test]
    fn clipped_section_strip_keeps_native_overflow_hit_and_glyph() {
        let mut h = compact_host(Rect::new(7, 5, 27, 12));
        let right_overflow = PartRef::item(Part::OVERFLOW, ItemKey::index(1));
        let overflow_area = h
            .area_of_part(PAGE_TABS, right_overflow)
            .expect("clipped native strip keeps its right overflow target");
        let overflow_glyph = Theme::junie().design.glyphs.get(GlyphRole::OverflowRight);
        assert_eq!(
            h.cell(overflow_area.right() - 1, overflow_area.y).symbol(),
            overflow_glyph
        );

        let _ = h.click_part(PAGE_TABS, right_overflow);
        assert!(
            h.area_of_part(PAGE_TABS, PartRef::item(Part::TAB, ItemKey::index(2)))
                .is_some(),
            "the native overflow action reveals the clipped States tab"
        );
        assert!(h.diagnostics().is_empty());
    }

    #[test]
    fn native_chip_options_change_mode_and_close_without_losing_other_selection() {
        let mut h = host(134, 46);
        open_section(&mut h, 1);

        let _ = h.click_id(CHIP_MODE);
        let _ = h.key(KeyCode::Down);
        let _ = h.key(KeyCode::Enter);
        assert_eq!(h.app().page.chip_mode(), termrock::SelectMode::Single);

        let _ = h.click_id(CHIP_WIDTH);
        let _ = h.key(KeyCode::Down);
        let _ = h.key(KeyCode::Down);
        let _ = h.key(KeyCode::Enter);
        assert_eq!(h.app().page.chip_width(100), 40);
        assert_eq!(h.area_of(CHIPS).map(|area| area.width), Some(40));

        let _ = h.click_id(CHIP_CLOSE);
        assert!(h.app().page.close_chips);

        assert!(h.area_of(CHIPS).is_some(), "the options sample stays live");
        let before = h.app().page.chip_state.checked().clone();
        let _ = h.click_part(CHIPS, PartRef::item(Part::LABEL, ItemKey::text("Open")));
        assert_eq!(h.app().page.last, "filter activated");
        assert_eq!(h.app().page.chip_state.checked(), &before);

        let close = h.area_of_part(CHIPS, PartRef::item(Part::CLOSE, ItemKey::text("Assigned")));
        let label = h.area_of_part(CHIPS, PartRef::item(Part::LABEL, ItemKey::text("Assigned")));
        let _ = h.click_part(CHIPS, PartRef::item(Part::CLOSE, ItemKey::text("Assigned")));
        assert_eq!(
            h.app().page.last,
            "filter closed",
            "close={close:?}, label={label:?}, frame={}",
            h.text()
        );
        assert!(!h.app().page.active_filters.contains(&"Assigned"));
        assert!(
            h.app()
                .page
                .chip_state
                .checked()
                .contains(ItemKey::text("Open"))
        );
        assert!(
            !h.app()
                .page
                .chip_state
                .checked()
                .contains(ItemKey::text("Assigned"))
        );
        assert!(h.diagnostics().is_empty(), "{:#?}", h.diagnostics());
    }

    #[test]
    fn language_select_dismisses_before_the_outside_click_acts_and_keeps_value() {
        let mut h = host(134, 46);
        let _ = h.click_id(SELECT);
        assert!(h.is_open(SELECT));
        let _ = h.click_part(SELECT, PartRef::item(Part::ROW, ItemKey::index(2)));
        assert_eq!(h.app().page.select_state.value(), Some(ItemKey::index(2)));
        assert!(!h.is_open(SELECT));

        let _ = h.click_id(SELECT);
        click_tab(&mut h, "Options");
        assert!(!h.is_open(SELECT));
        assert!(!h.app().page.select_state.is_open());
        assert_eq!(
            h.app().page.section(),
            0,
            "first outside click only dismisses"
        );
        click_tab(&mut h, "Options");
        assert_eq!(
            h.app().page.section(),
            1,
            "next outside click activates tab"
        );
        assert_eq!(h.app().page.select_state.value(), Some(ItemKey::index(2)));

        let _ = h.click_id(POPUP_ROWS);
        let _ = h.key(KeyCode::Down);
        let _ = h.key(KeyCode::Enter);
        assert_eq!(h.app().page.popup_rows(), Some(4));
        click_tab(&mut h, "Playground");
        assert_eq!(h.app().page.section(), 0);
        let _ = h.click_id(SELECT);
        assert!(h.is_open(SELECT));
        assert_eq!(h.app().page.select_state.value(), Some(ItemKey::index(2)));
        assert_eq!(h.layer_area(SELECT).map(|area| area.height), Some(6));
        assert!(h.diagnostics().is_empty());
    }

    #[test]
    fn disabled_language_select_keeps_native_disabled_behavior() {
        let mut h = host(134, 46);
        open_section(&mut h, 1);
        let _ = h.click_id(SELECT_DISABLED);
        assert!(h.app().page.select_disabled);
        open_section(&mut h, 0);
        assert!(h.state_of(SELECT).contains(StateFlags::DISABLED));
        let _ = h.click_id(SELECT);
        assert!(!h.is_open(SELECT));
    }

    #[test]
    fn compact_options_focus_reveals_hidden_controls_without_wheel_snapback() {
        let area = Rect::new(7, 5, 104, 9);
        let mut h = compact_host(area);
        open_section(&mut h, 1);
        assert!(h.ring().contains(SELECT_DISABLED));
        assert!(h.area_of(SELECT_DISABLED).is_none());
        assert!(h.tab_to(SELECT_DISABLED));
        assert_eq!(h.app().page.options_scroll.offset(), 4);
        assert!(h.area_of(SELECT_DISABLED).is_some());

        let scroll = h
            .area_of(OPTIONS_SCROLL)
            .expect("compact options scroll region is live");
        let expected_after_wheel =
            4usize.saturating_sub(usize::from(Theme::junie().design.motion.wheel_rows));
        let _ = h.wheel(Axis::V, -1, scroll.x, scroll.y);
        assert!(
            h.app()
                .captured_hints
                .get()
                .iter()
                .all(|(_, action)| *action != "Toggle"),
            "the first returned wheel frame hides Toggle hints"
        );
        assert_eq!(
            h.app().page.options_scroll.offset(),
            expected_after_wheel,
            "wheel motion follows the active theme token and clamps at zero"
        );
        assert_eq!(h.focus(), Some(SELECT_DISABLED));
        assert!(h.area_of(SELECT_DISABLED).is_none());
        let _ = h.key(KeyCode::Enter);
        let _ = h.key(KeyCode::Char(' '));
        assert!(
            !h.app().page.select_disabled,
            "hidden Toggle ignores activation"
        );
        assert_eq!(h.app().page.options_scroll.offset(), expected_after_wheel);

        let _ = h.key(KeyCode::Tab);
        assert_ne!(h.focus(), Some(SELECT_DISABLED));
        let _ = h.key(KeyCode::BackTab);
        assert_eq!(h.focus(), Some(SELECT_DISABLED));
        assert_eq!(h.app().page.options_scroll.offset(), 4);
        assert!(h.area_of(SELECT_DISABLED).is_some());
        assert!(
            h.app().captured_hints.get().contains(&("Space", "Toggle")),
            "the first transition-driven reveal frame restores Toggle hints"
        );
        let _ = h.key(KeyCode::Char(' '));
        assert!(h.app().page.select_disabled);
        assert!(h.diagnostics().is_empty());
    }

    #[test]
    fn clipped_page_width_clears_stale_option_hints_in_the_same_draw() {
        let mut h = compact_host(Rect::new(7, 5, 104, 9));
        open_section(&mut h, 1);
        assert!(h.tab_to(CHIP_MODE));
        assert!(
            h.app()
                .captured_hints
                .get()
                .contains(&("Enter", "Open / choose"))
        );

        h.app_mut().area = Some(Rect::new(7, 5, 1, 9));
        h.draw();
        assert!(
            h.app()
                .captured_hints
                .get()
                .iter()
                .all(|(_, action)| *action != "Open / choose"),
            "a narrow page draw cannot reuse the preceding live control area"
        );
        assert_ne!(h.focus(), Some(CHIP_MODE));
        let _ = h.tick();
        assert_ne!(h.focus(), Some(CHIP_MODE));
        assert!(h.diagnostics().is_empty(), "{:#?}", h.diagnostics());
    }

    #[test]
    fn hidden_chip_mode_select_reconciles_then_reopens_and_selects() {
        let mut h = compact_host(Rect::new(7, 5, 104, 9));
        open_section(&mut h, 1);
        assert!(h.area_of(CHIP_MODE).is_some());
        let _ = h.click_id(CHIP_MODE);
        assert!(h.is_open(CHIP_MODE));
        assert!(h.app().page.chip_mode_state.is_open());

        h.app_mut().page.options_scroll.scroll_to(1);
        h.draw();
        assert!(h.area_of(CHIP_MODE).is_none(), "the anchor row is clipped");
        assert!(
            h.is_open(CHIP_MODE),
            "runtime dismissal awaits an outside click"
        );
        click_tab(&mut h, "Playground");
        assert_eq!(
            h.app().page.section(),
            1,
            "first outside click only dismisses"
        );
        assert!(!h.is_open(CHIP_MODE));
        assert!(!h.app().page.chip_mode_state.is_open());

        click_tab(&mut h, "Playground");
        open_section(&mut h, 1);
        assert!(
            h.area_of(CHIP_MODE).is_some(),
            "the option row is revealed again"
        );
        let _ = h.click_id(CHIP_MODE);
        let _ = h.key(KeyCode::Down);
        let _ = h.key(KeyCode::Enter);
        assert_eq!(h.app().page.chip_mode(), termrock::SelectMode::Single);
        assert!(!h.is_open(CHIP_MODE));
        assert!(!h.app().page.chip_mode_state.is_open());
        assert!(h.diagnostics().is_empty());
    }

    #[test]
    fn hidden_chip_width_select_reconciles_after_leaving_options() {
        let mut h = compact_host(Rect::new(7, 5, 104, 9));
        open_section(&mut h, 1);
        assert!(h.tab_to(CHIP_WIDTH), "focus reveals the clipped width row");
        assert!(h.area_of(CHIP_WIDTH).is_some());
        let _ = h.click_id(CHIP_WIDTH);
        assert!(h.is_open(CHIP_WIDTH));
        assert!(h.app().page.chip_width_state.is_open());

        h.app_mut().page.options_scroll.scroll_to(0);
        h.draw();
        assert!(h.area_of(CHIP_WIDTH).is_none(), "the anchor row is clipped");
        h.app_mut().page.page_tabs.set_active(2, ItemKey::index(2));
        let _ = h.click(0, 0);
        assert_eq!(h.app().page.section(), 2);
        assert!(!h.is_open(CHIP_WIDTH));
        assert!(
            !h.app().page.chip_width_state.is_open(),
            "dismissal is reconciled after the Options section leaves view"
        );
        assert_ne!(h.focus(), Some(CHIP_WIDTH));
        let _ = h.tick();
        assert_ne!(h.focus(), Some(CHIP_WIDTH));
        assert!(h.diagnostics().is_empty());

        h.app_mut().page.page_tabs.set_active(1, ItemKey::index(1));
        let _ = h.tick();
        h.app_mut().page.options_scroll.scroll_to(1);
        h.draw();
        assert!(
            h.area_of(CHIP_WIDTH).is_some(),
            "the width row is revealed again"
        );
        let _ = h.click_id(CHIP_WIDTH);
        let _ = h.key(KeyCode::Down);
        let _ = h.key(KeyCode::Enter);
        assert_eq!(h.app().page.chip_width(100), 60);
        assert_eq!(h.area_of(CHIPS).map(|area| area.width), Some(60));
        assert!(!h.is_open(CHIP_WIDTH));
        assert!(!h.app().page.chip_width_state.is_open());
        assert!(h.diagnostics().is_empty());
    }

    #[test]
    fn states_are_native_inert_specimens_with_unicode_and_overflow() {
        let mut h = host(100, 30);
        open_section(&mut h, 2);
        let text = h.text();
        assert!(text.contains("Overflow"), "{text}");
        assert!(text.contains("北京"), "{text}");
        assert!(text.contains("Chip disabled"), "{text}");
        assert!(text.contains("Select disabled"), "{text}");
        assert!(text.contains("…"), "{text}");
        assert!(h.area_of(CHIPS).is_none());
        assert!(h.area_of(SELECT).is_none());
        assert!(h.area_of(OPTIONS_SCROLL).is_none());
        assert!(h.area_of(STATES_SCROLL).is_some());
        assert!(h.diagnostics().is_empty());
    }
}
