use std::{cell::Cell, time::Duration};

use termrock::navigation::TabAffixes;
use termrock::{
    ActionKey, Binding, Chord, Color, Cx, Focusability, FrameRead, Id, Intent, ItemKey, KeyCode,
    Modifier, PaintStyle, Part, PartRef, Rect, ReferenceState, ReferenceTarget, Response,
    ScrollRegion, ScrollState, Select, SelectState, StateFlags, Tabs, TabsState, TextViewport,
    Toggle, Ui, ViewportLine, ViewportState,
    author::VerticalWindow,
    core::{Cursor, Nav, RowFn, TypeAhead},
    truncate, width, wrap,
};

use showcase_data::{PROSE, SCROLL_ROWS, log_lines};
use termrock::collections::FollowCursor;

use super::{Page, PageUpdate, frame};

const LIST_VIEW: Id = termrock::Id::root("showcase_ui::pages::scrolling::scrolling.list");
const PAGE_TABS: Id = Id::root("showcase_ui::pages::scrolling.page_tabs");
const OPTIONS_SCROLL: Id = Id::root("showcase_ui::pages::scrolling.options_scroll");
const OPTIONS_VIEW: Id = Id::root("showcase_ui::pages::scrolling.options_view");
const CONTEXT_SCROLL: Id = Id::root("showcase_ui::pages::scrolling.context_scroll");
const STATES_SCROLL: Id = Id::root("showcase_ui::pages::scrolling.states_scroll");
const STATE_SAMPLE: Id = Id::root("showcase_ui::pages::scrolling.state_sample");
const CONTEXT_OPTION: Id = Id::root("showcase_ui::pages::scrolling.option.context");
const FOLLOW_OPTION: Id = Id::root("showcase_ui::pages::scrolling.option.follow");
const WRAP_OPTION: Id = Id::root("showcase_ui::pages::scrolling.option.wrap");
const CURSOR_WRAP_OPTION: Id = Id::root("showcase_ui::pages::scrolling.option.cursor_wrap");
const SKIP_DISABLED_OPTION: Id = Id::root("showcase_ui::pages::scrolling.option.skip_disabled");
const HORIZONTAL_PAN_OPTION: Id = Id::root("showcase_ui::pages::scrolling.option.horizontal_pan");
const PROFILE_OPTION: Id = Id::root("showcase_ui::pages::scrolling.option.profile");
const TYPE_AHEAD_OPTION: Id = Id::root("showcase_ui::pages::scrolling.option.type_ahead");
const TIMEOUT_OPTION: Id = Id::root("showcase_ui::pages::scrolling.option.timeout");
const PRECEDENCE_OPTION: Id = Id::root("showcase_ui::pages::scrolling.option.precedence");

const BLURB: &str = "Wheel under the pointer, keys on the focused container, thumb shows where you are, edges fade toward hidden rows";
const SECTIONS: &[&str] = &["Playground", "Options", "States"];
const SECTION_CAPTION: &str = "the live widget";
const SECTION_WIDE: u16 = 76;
const CONTEXT_ITEMS: &[&str] = &["0 rows", "1 row", "2 rows"];
const FOLLOW_ITEMS: &[&str] = &["On move", "Always"];
const PROFILE_ITEMS: &[&str] = &["Desktop", "Vim"];
const TYPE_AHEAD_ITEMS: &[&str] = &["Profile", "On", "Off"];
const TIMEOUT_ITEMS: &[&str] = &["0.5 s", "1 s", "2 s"];
const PRECEDENCE_ITEMS: &[&str] = &["Shortcuts", "Type-ahead"];
const TAB_HINTS: &[(&str, &str)] = &[("← →", "Section"), ("1–3", "Jump"), ("Enter", "Activate")];
const OPTIONS_HINTS: &[(&str, &str)] = &[
    ("↑ ↓", "Choose"),
    ("Enter", "Open / choose"),
    ("Space", "Toggle"),
    ("Wheel", "Scroll"),
];
const SELECT_HINTS: &[(&str, &str)] = &[
    ("↑ ↓", "Choose"),
    ("Enter", "Open / choose"),
    ("Wheel", "Scroll"),
];
const SELECT_OPEN_HINTS: &[(&str, &str)] = &[
    ("↑ ↓", "Choose"),
    ("Enter", "Select"),
    ("Esc", "Close"),
    ("Wheel", "Scroll"),
];
const TOGGLE_HINTS: &[(&str, &str)] = &[("Space", "Toggle"), ("Wheel", "Scroll")];
const SCROLL_HINTS: &[(&str, &str)] = &[
    ("↑ ↓", "Scroll"),
    ("PgUp PgDn", "Page"),
    ("Home End", "Ends"),
    ("Wheel", "Scroll"),
];
const PAN_SCROLL_HINTS: &[(&str, &str)] = &[
    ("↑ ↓", "Scroll"),
    ("← →", "Pan"),
    ("PgUp PgDn", "Page"),
    ("Home End", "Ends"),
    ("Wheel", "Scroll"),
];
const TYPE_AHEAD_HINTS: &[(&str, &str)] = &[
    ("↑ ↓", "Scroll"),
    ("Type", "Jump"),
    ("PgUp PgDn", "Page"),
    ("Wheel", "Scroll"),
];
const VIM_HINTS: &[(&str, &str)] = &[("↑ ↓", "Scroll"), ("j k", "Move"), ("g G", "Ends")];
const VIM_TYPE_AHEAD_HINTS: &[(&str, &str)] = &[
    ("↑ ↓", "Scroll"),
    ("j k", "Move"),
    ("g G", "Ends"),
    ("Type", "Jump"),
];
const OPTIONS_ROWS: usize = 31;
const OPTIONS_VIEW_START: usize = 14;
const OPTIONS_VIEW_ROWS: usize = 8;
const CONTEXT_VIEW_START: usize = 23;
const CONTEXT_VIEW_ROWS: usize = 8;
const STATE_CARD_ROWS: usize = 5;
const STATE_SAMPLE_ROWS: usize = 3;
const STATE_COUNT: usize = 6;
const ROUTE_REVISION: u64 = 1;
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
const ACCENT: u32 = 0x48e054;
const ACCENT_BG: u32 = 0x27272a;
const SECONDARY: u32 = 0xb3b3b3;
const MUTED: u32 = 0x8a8a8a;
const FAINT: u32 = 0x737373;
const RULE: u32 = 0x3a3a3a;
const THUMB: u32 = 0x808080;
const TRACK: u32 = 0x474747;
const WARNING: u32 = 0xf59e09;
const ERROR: u32 = 0xf05d5d;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScrollKey {
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
}

const fn scroll_binding(
    action: &'static str,
    chord: Chord,
    cmd: ScrollKey,
    label: &'static str,
    visible: bool,
) -> Binding<ScrollKey> {
    Binding {
        action: ActionKey::custom(action),
        chord: Some(chord),
        cmd,
        label,
        priority: if visible { 60 } else { 10 },
        visible,
    }
}

const SCROLL_BINDINGS: &[Binding<ScrollKey>] = &[
    scroll_binding(
        "scrolling.up",
        Chord::key(KeyCode::Up),
        ScrollKey::Up,
        "Up",
        true,
    ),
    scroll_binding(
        "scrolling.down",
        Chord::key(KeyCode::Down),
        ScrollKey::Down,
        "Down",
        true,
    ),
    scroll_binding(
        "scrolling.page-up",
        Chord::key(KeyCode::PageUp),
        ScrollKey::PageUp,
        "Page up",
        true,
    ),
    scroll_binding(
        "scrolling.page-down",
        Chord::key(KeyCode::PageDown),
        ScrollKey::PageDown,
        "Page down",
        true,
    ),
    scroll_binding(
        "scrolling.home",
        Chord::key(KeyCode::Home),
        ScrollKey::Home,
        "Start",
        false,
    ),
    scroll_binding(
        "scrolling.end",
        Chord::key(KeyCode::End),
        ScrollKey::End,
        "End",
        false,
    ),
];

const VIM_SCROLL_BINDINGS: &[Binding<ScrollKey>] = &[
    scroll_binding(
        "scrolling.up",
        Chord::key(KeyCode::Up),
        ScrollKey::Up,
        "Up",
        true,
    ),
    scroll_binding(
        "scrolling.down",
        Chord::key(KeyCode::Down),
        ScrollKey::Down,
        "Down",
        true,
    ),
    scroll_binding(
        "scrolling.page-up",
        Chord::key(KeyCode::PageUp),
        ScrollKey::PageUp,
        "Page up",
        true,
    ),
    scroll_binding(
        "scrolling.page-down",
        Chord::key(KeyCode::PageDown),
        ScrollKey::PageDown,
        "Page down",
        true,
    ),
    scroll_binding(
        "scrolling.home",
        Chord::key(KeyCode::Home),
        ScrollKey::Home,
        "Start",
        false,
    ),
    scroll_binding(
        "scrolling.end",
        Chord::key(KeyCode::End),
        ScrollKey::End,
        "End",
        false,
    ),
    scroll_binding(
        "scrolling.up-vim",
        Chord::key(KeyCode::Char('k')),
        ScrollKey::Up,
        "Up",
        false,
    ),
    scroll_binding(
        "scrolling.down-vim",
        Chord::key(KeyCode::Char('j')),
        ScrollKey::Down,
        "Down",
        false,
    ),
    scroll_binding(
        "scrolling.home-vim",
        Chord::key(KeyCode::Char('g')),
        ScrollKey::Home,
        "Start",
        false,
    ),
    scroll_binding(
        "scrolling.end-vim",
        Chord::key(KeyCode::Char('G')),
        ScrollKey::End,
        "End",
        false,
    ),
];

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

fn section_tab_affixes(_section: &&str) -> TabAffixes {
    TabAffixes::default().trailing_padding(1)
}

fn intersects_overflow_hit(layout: &termrock::navigation::tabs::TabLayout, area: Rect) -> bool {
    (0..SECTIONS.len()).any(|index| {
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
            ink(WHITE, ACCENT_BG).add_modifier(Modifier::BOLD)
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

fn thumb_len(viewport: usize, content: usize, track: usize) -> usize {
    if content <= viewport || track == 0 {
        return 0;
    }
    viewport
        .saturating_mul(track)
        .checked_div(content)
        .unwrap_or(0)
        .clamp(1, track)
}

fn thumb_start(len: usize, track: usize, at_end: bool) -> usize {
    if at_end { track.saturating_sub(len) } else { 0 }
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
    let len = thumb_len(viewport, content, usize::from(area.height));
    if len == 0 {
        return;
    }
    let origin = thumb_start(len, usize::from(area.height), at_end);
    for row in 0..area.height {
        let on = usize::from(row) >= origin && usize::from(row) < origin.saturating_add(len);
        let _ = ui.paint_str(
            Rect::new(area.x, area.y.saturating_add(row), 1, 1),
            if on { "┃" } else { "┆" },
            ink(if on { THUMB } else { TRACK }, SURFACE),
        );
    }
}

fn paint_hbar(ui: &mut Ui<'_>, area: Rect, viewport: usize, content: usize) {
    let len = thumb_len(viewport, content, usize::from(area.width));
    if len == 0 {
        return;
    }
    for col in 0..area.width {
        let on = usize::from(col) < len;
        let _ = ui.paint_str(
            Rect::new(area.x.saturating_add(col), area.y, 1, 1),
            if on { "━" } else { "┄" },
            ink(if on { THUMB } else { TRACK }, SURFACE),
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
    let viewport = usize::from(body_h);
    for index in 0..viewport {
        let y = area
            .y
            .saturating_add(1)
            .saturating_add(u16::try_from(index).unwrap_or(0));
        let edge = fade_row(index, viewport, 0, HAND_ROWS);
        let color = if index == 0 { WHITE } else { SECONDARY };
        let line = hand_visible(Some(index), text_w);
        paint_span(
            ui,
            Rect::new(area.x, y, text_w, 1),
            &line,
            faded(color, edge),
            SURFACE,
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
        HAND_ROWS,
        false,
    );
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
        for index in 0..viewport.min(total) {
            let y = body.y.saturating_add(u16::try_from(index).unwrap_or(0));
            let label = format!("item {}", index.saturating_add(1));
            let shown = if width(&label) <= text_w {
                label
            } else {
                truncate(&label, text_w)
            };
            paint_span(
                ui,
                Rect::new(body.x, y, text_w, 1),
                &shown,
                SECONDARY,
                SURFACE,
            );
            if bar {
                continue;
            }
            let _ = ui.paint_str(
                Rect::new(body.right().saturating_sub(1), y, 1, 1),
                "│",
                ink(RULE, SURFACE),
            );
        }
        if bar {
            paint_vbar(
                ui,
                Rect::new(body.right().saturating_sub(1), body.y, 1, body.height),
                viewport,
                total,
                false,
            );
        }
    }
}

fn paint_cursor(ui: &mut Ui<'_>, area: Rect) {
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
    let len = thumb_len(1, 3, track);
    if len == 0 {
        return;
    }
    let x = area.right().saturating_sub(1);
    for row in 0..area.height {
        let on = usize::from(row) < len;
        let _ = ui.paint_str(
            Rect::new(x, area.y.saturating_add(row), 1, 1),
            if on { "┃" } else { "┆" },
            ink(if on { THUMB } else { TRACK }, CANVAS),
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

fn scroll_window(view: Rect, logical_rows: usize, state: &ScrollState) -> Option<VerticalWindow> {
    if view.width == 0 || view.height == 0 || logical_rows == 0 {
        return None;
    }
    let first = ScrollRegion::view(state, view, logical_rows)
        .offset()
        .min(logical_rows);
    let visible_rows = usize::from(view.height).min(logical_rows.saturating_sub(first));
    if visible_rows == 0 {
        return None;
    }
    let body = Rect {
        height: u16::try_from(visible_rows).ok()?,
        ..view
    };
    VerticalWindow::new(body, logical_rows, first).ok()
}

fn nested_window(
    parent: VerticalWindow,
    source_start: usize,
    source_rows: usize,
    x: u16,
    width: u16,
) -> Option<VerticalWindow> {
    if source_rows == 0 || width == 0 {
        return None;
    }
    let visible = parent.visible_source_range();
    let start = visible.start.max(source_start);
    let end = visible.end.min(source_start.saturating_add(source_rows));
    if start >= end {
        return None;
    }
    let y = parent.physical_row_for_source(start)?;
    let height = u16::try_from(end.saturating_sub(start)).ok()?;
    VerticalWindow::new(
        Rect::new(x, y, width, height),
        source_rows,
        start.saturating_sub(source_start),
    )
    .ok()
}

fn nested_scroll_window(
    window: VerticalWindow,
    body: Rect,
    content_len: usize,
    state: &ScrollState,
) -> Option<VerticalWindow> {
    if body.is_empty() || window.logical_rows() == 0 || content_len == 0 {
        return None;
    }
    let logical_height = u16::try_from(window.logical_rows()).ok()?;
    let logical_view = Rect::new(body.x, body.y, body.width, logical_height);
    let first = ScrollRegion::view(state, logical_view, content_len)
        .offset()
        .saturating_add(window.first_source_row());
    let visible_rows = usize::from(body.height).min(content_len.saturating_sub(first));
    if visible_rows == 0 {
        return None;
    }
    let visible_body = Rect {
        height: u16::try_from(visible_rows).ok()?,
        ..body
    };
    VerticalWindow::new(visible_body, content_len, first).ok()
}

fn register_scroll_keys(ui: &mut Ui<'_>, id: Id) {
    ui.publish_bindings(id, ui.state(id), SCROLL_BINDINGS);
}

fn register_focus_only(ui: &mut Ui<'_>, id: Id) {
    ui.register_focus_only(id, Focusability::FocusableReadOnly);
}

fn update_scroll_keys(cx: &mut Cx<'_>, id: Id, state: &mut ScrollState) -> Response<()> {
    let mut response = Response::ignored();
    for intent in cx.intents(id) {
        let Intent::Binding(action) = intent else {
            continue;
        };
        match Binding::command(SCROLL_BINDINGS, action) {
            Some(ScrollKey::Up) => state.scroll_by(-1),
            Some(ScrollKey::Down) => state.scroll_by(1),
            Some(ScrollKey::PageUp) => state.page_up(),
            Some(ScrollKey::PageDown) => state.page_down(),
            Some(ScrollKey::Home) => state.jump_start(),
            Some(ScrollKey::End) => state.jump_end(),
            None => continue,
        }
        response |= Response::changed().for_id(id);
    }
    response
}

fn options_view(wrap_lines: bool, horizontal_pan: bool) -> TextViewport<'static> {
    TextViewport::new(OPTIONS_VIEW)
        .wrap(wrap_lines)
        .horizontal_pan(horizontal_pan)
}

fn option_log_lines(log: &[String]) -> Vec<ViewportLine<'_>> {
    PROSE
        .split('\n')
        .map(ViewportLine::Plain)
        .chain(log.iter().map(|line| ViewportLine::Plain(line.as_str())))
        .collect()
}

fn context_row_enabled(index: usize) -> bool {
    index % 10 != 9
}

fn context_row_label(index: usize) -> &'static str {
    SCROLL_ROWS.get(index).copied().unwrap_or("")
}

fn context_rows() -> RowFn<'static> {
    RowFn {
        len: LIST_LEN,
        enabled: &context_row_enabled,
        label: &context_row_label,
    }
}

fn type_ahead_timeout(index: usize) -> Duration {
    match index {
        0 => Duration::from_millis(500),
        2 => Duration::from_secs(2),
        _ => Duration::from_secs(1),
    }
}

#[derive(Debug)]
pub struct ScrollingPage {
    list: Vec<ViewportLine<'static>>,
    log: Vec<String>,
    list_state: ViewportState,
    last: &'static str,
    tabs: TabsState,
    options_scroll: ScrollState,
    options_view_state: ViewportState,
    context_scroll: ScrollState,
    context_cursor: usize,
    cursor_wrap: bool,
    skip_disabled: bool,
    profile: usize,
    type_ahead_mode: usize,
    timeout: usize,
    precedence: usize,
    context_type_ahead: TypeAhead,
    states_scroll: ScrollState,
    context_option: usize,
    context_option_state: SelectState,
    follow_option_state: SelectState,
    profile_option_state: SelectState,
    type_ahead_option_state: SelectState,
    timeout_option_state: SelectState,
    precedence_option_state: SelectState,
    wrap_lines: bool,
    horizontal_pan: bool,
    option_revision: Option<u64>,
    options_window: Cell<(usize, usize)>,
    options_window_valid: Cell<bool>,
}

impl ScrollingPage {
    pub fn new() -> Self {
        let mut list_state = ViewportState::default();
        list_state.set_follow(false);
        let mut options_view_state = ViewportState::default();
        options_view_state.set_follow_cursor(FollowCursor::OnMove);
        Self {
            list: list_lines(),
            log: log_lines(LOG_START),
            list_state,
            last: "top of document",
            tabs: TabsState::default(),
            options_scroll: ScrollState::default(),
            options_view_state,
            context_scroll: ScrollState::default()
                .with_context(1)
                .with_follow_cursor(FollowCursor::OnMove),
            context_cursor: 0,
            cursor_wrap: false,
            skip_disabled: true,
            profile: 0,
            type_ahead_mode: 0,
            timeout: 1,
            precedence: 0,
            context_type_ahead: TypeAhead::with_timeout(Duration::from_secs(1)),
            states_scroll: ScrollState::default(),
            context_option: 1,
            context_option_state: SelectState::default(),
            follow_option_state: SelectState::default(),
            profile_option_state: SelectState::default(),
            type_ahead_option_state: SelectState::default(),
            timeout_option_state: SelectState::default(),
            precedence_option_state: SelectState::default(),
            wrap_lines: false,
            horizontal_pan: false,
            option_revision: Some(ROUTE_REVISION),
            options_window: Cell::new((0, 0)),
            options_window_valid: Cell::new(false),
        }
    }

    fn section(&self) -> usize {
        match self.tabs.active().or(self.tabs.cursor()) {
            Some(termrock::ItemKey::Index(index)) => index.min(2),
            _ => 0,
        }
    }

    fn bump_option_revision(&mut self) {
        self.option_revision = self
            .option_revision
            .and_then(|revision| revision.checked_add(1));
    }

    fn option_popup_open(&self, cx: &Cx<'_>) -> bool {
        self.context_option_state.is_open()
            || self.follow_option_state.is_open()
            || self.profile_option_state.is_open()
            || self.type_ahead_option_state.is_open()
            || self.timeout_option_state.is_open()
            || self.precedence_option_state.is_open()
            || cx.is_open(CONTEXT_OPTION)
            || cx.is_open(FOLLOW_OPTION)
            || cx.is_open(PROFILE_OPTION)
            || cx.is_open(TYPE_AHEAD_OPTION)
            || cx.is_open(TIMEOUT_OPTION)
            || cx.is_open(PRECEDENCE_OPTION)
    }

    fn option_select_open(&self, id: Id) -> bool {
        match id {
            CONTEXT_OPTION => self.context_option_state.is_open(),
            FOLLOW_OPTION => self.follow_option_state.is_open(),
            PROFILE_OPTION => self.profile_option_state.is_open(),
            TYPE_AHEAD_OPTION => self.type_ahead_option_state.is_open(),
            TIMEOUT_OPTION => self.timeout_option_state.is_open(),
            PRECEDENCE_OPTION => self.precedence_option_state.is_open(),
            _ => false,
        }
    }

    fn hints_for_focused_owner(
        &self,
        focused: impl Fn(Id) -> bool,
    ) -> &'static [(&'static str, &'static str)] {
        if focused(PAGE_TABS) {
            return TAB_HINTS;
        }
        if focused(CONTEXT_SCROLL) {
            if self.profile == 1 && !(self.context_type_ahead_enabled() && self.precedence == 1) {
                return if self.context_type_ahead_enabled() {
                    VIM_TYPE_AHEAD_HINTS
                } else {
                    VIM_HINTS
                };
            }
            if self.context_type_ahead_enabled() {
                return TYPE_AHEAD_HINTS;
            }
            return SCROLL_HINTS;
        }
        if focused(OPTIONS_VIEW) {
            return if self.horizontal_pan && !self.wrap_lines {
                PAN_SCROLL_HINTS
            } else {
                SCROLL_HINTS
            };
        }
        if focused(OPTIONS_SCROLL) || focused(STATES_SCROLL) || focused(LIST_VIEW) {
            return SCROLL_HINTS;
        }
        for id in [
            CONTEXT_OPTION,
            FOLLOW_OPTION,
            PROFILE_OPTION,
            TYPE_AHEAD_OPTION,
            TIMEOUT_OPTION,
            PRECEDENCE_OPTION,
        ] {
            if focused(id) {
                return if self.option_select_open(id) {
                    SELECT_OPEN_HINTS
                } else {
                    SELECT_HINTS
                };
            }
        }
        if [
            WRAP_OPTION,
            CURSOR_WRAP_OPTION,
            SKIP_DISABLED_OPTION,
            HORIZONTAL_PAN_OPTION,
        ]
        .into_iter()
        .any(focused)
        {
            return TOGGLE_HINTS;
        }
        match self.section() {
            0 => &[("↑ ↓", "Scroll"), ("PgUp PgDn", "Page"), ("g G", "Ends")],
            1 => OPTIONS_HINTS,
            _ => SCROLL_HINTS,
        }
    }

    fn context_type_ahead_enabled(&self) -> bool {
        match self.type_ahead_mode {
            1 => true,
            2 => false,
            _ => self.profile == 0,
        }
    }

    fn context_bindings(&self) -> &'static [Binding<ScrollKey>] {
        if self.profile == 1 && !(self.context_type_ahead_enabled() && self.precedence == 1) {
            VIM_SCROLL_BINDINGS
        } else {
            SCROLL_BINDINGS
        }
    }

    fn option_visible(&self, row: usize) -> bool {
        if !self.options_window_valid.get() {
            return true;
        }
        let (start, end) = self.options_window.get();
        (start..end).contains(&row)
    }

    fn update_option_controls(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = Response::ignored();

        if self.option_visible(1)
            || self.context_option_state.is_open()
            || cx.is_open(CONTEXT_OPTION)
        {
            let mut context = self.context_option;
            let old = context;
            let result = Select::new(CONTEXT_OPTION).update_in_form(
                cx,
                &mut self.context_option_state,
                &mut context,
                CONTEXT_ITEMS,
                false,
            );
            response |= result.erase();
            if context != old {
                self.context_option = context.min(CONTEXT_ITEMS.len().saturating_sub(1));
                self.context_scroll = self.context_scroll.with_context(self.context_option);
            }
        }

        if self.option_visible(3) || self.follow_option_state.is_open() || cx.is_open(FOLLOW_OPTION)
        {
            let old = usize::from(self.options_view_state.follow_cursor() == FollowCursor::Always);
            let mut follow = old;
            let result = Select::new(FOLLOW_OPTION).update_in_form(
                cx,
                &mut self.follow_option_state,
                &mut follow,
                FOLLOW_ITEMS,
                false,
            );
            response |= result.erase();
            if follow != old {
                let mode = if follow == 0 {
                    FollowCursor::OnMove
                } else {
                    FollowCursor::Always
                };
                self.options_view_state.set_follow_cursor(mode);
                self.context_scroll = self.context_scroll.with_follow_cursor(mode);
            }
        }

        if self.option_visible(9)
            || self.profile_option_state.is_open()
            || cx.is_open(PROFILE_OPTION)
        {
            let mut profile = self.profile;
            let result = Select::new(PROFILE_OPTION).update_in_form(
                cx,
                &mut self.profile_option_state,
                &mut profile,
                PROFILE_ITEMS,
                false,
            );
            response |= result.erase();
            if profile != self.profile {
                self.profile = profile.min(PROFILE_ITEMS.len().saturating_sub(1));
                self.context_type_ahead.reset();
            }
        }

        if self.option_visible(10)
            || self.type_ahead_option_state.is_open()
            || cx.is_open(TYPE_AHEAD_OPTION)
        {
            let mut type_ahead_mode = self.type_ahead_mode;
            let result = Select::new(TYPE_AHEAD_OPTION).update_in_form(
                cx,
                &mut self.type_ahead_option_state,
                &mut type_ahead_mode,
                TYPE_AHEAD_ITEMS,
                false,
            );
            response |= result.erase();
            if type_ahead_mode != self.type_ahead_mode {
                self.type_ahead_mode =
                    type_ahead_mode.min(TYPE_AHEAD_ITEMS.len().saturating_sub(1));
                self.context_type_ahead.reset();
            }
        }

        if self.option_visible(11)
            || self.timeout_option_state.is_open()
            || cx.is_open(TIMEOUT_OPTION)
        {
            let mut timeout = self.timeout;
            let result = Select::new(TIMEOUT_OPTION).update_in_form(
                cx,
                &mut self.timeout_option_state,
                &mut timeout,
                TIMEOUT_ITEMS,
                false,
            );
            response |= result.erase();
            if timeout != self.timeout {
                self.timeout = timeout.min(TIMEOUT_ITEMS.len().saturating_sub(1));
                self.context_type_ahead.timeout = type_ahead_timeout(self.timeout);
                self.context_type_ahead.reset();
            }
        }

        if self.option_visible(12)
            || self.precedence_option_state.is_open()
            || cx.is_open(PRECEDENCE_OPTION)
        {
            let mut precedence = self.precedence;
            let result = Select::new(PRECEDENCE_OPTION).update_in_form(
                cx,
                &mut self.precedence_option_state,
                &mut precedence,
                PRECEDENCE_ITEMS,
                false,
            );
            response |= result.erase();
            if precedence != self.precedence {
                self.precedence = precedence.min(PRECEDENCE_ITEMS.len().saturating_sub(1));
                self.context_type_ahead.reset();
            }
        }

        if self.option_visible(4) {
            let mut wrap_lines = self.wrap_lines;
            let result = Toggle::new(WRAP_OPTION, "Wrap output lines")
                .on(wrap_lines)
                .update(cx, &mut wrap_lines);
            response |= result.erase();
            if wrap_lines != self.wrap_lines {
                self.wrap_lines = wrap_lines;
                self.bump_option_revision();
            }
        }

        if self.option_visible(5) {
            let mut cursor_wrap = self.cursor_wrap;
            let result = Toggle::new(CURSOR_WRAP_OPTION, "Wrap cursor at list ends")
                .on(cursor_wrap)
                .update(cx, &mut cursor_wrap);
            response |= result.erase();
            self.cursor_wrap = cursor_wrap;
        }

        if self.option_visible(6) {
            let mut skip_disabled = self.skip_disabled;
            let result = Toggle::new(SKIP_DISABLED_OPTION, "Skip disabled rows")
                .on(skip_disabled)
                .update(cx, &mut skip_disabled);
            response |= result.erase();
            if skip_disabled != self.skip_disabled {
                self.skip_disabled = skip_disabled;
                if skip_disabled && !context_row_enabled(self.context_cursor) {
                    self.context_cursor = self.context_cursor.saturating_sub(1);
                    self.context_scroll.follow(self.context_cursor);
                }
            }
        }

        if self.option_visible(7) {
            let mut horizontal_pan = self.horizontal_pan;
            let result = Toggle::new(HORIZONTAL_PAN_OPTION, "Allow sideways pan")
                .on(horizontal_pan)
                .update(cx, &mut horizontal_pan);
            response |= result.erase();
            if horizontal_pan != self.horizontal_pan {
                self.horizontal_pan = horizontal_pan;
                self.bump_option_revision();
            }
        }
        response
    }

    fn update_context_scroll(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if cx.layout(CONTEXT_SCROLL).is_none() {
            return Response::ignored();
        }
        let mut response = ScrollRegion::new(CONTEXT_SCROLL).update_routed(
            cx,
            &mut self.context_scroll,
            LIST_LEN,
            ROUTE_REVISION,
        );
        let rows = context_rows();
        let mut cursor = Cursor::new(self.context_cursor)
            .wrap(self.cursor_wrap)
            .skip_disabled(self.skip_disabled);
        for intent in cx.intents(CONTEXT_SCROLL) {
            let nav = match intent {
                Intent::Binding(action) => {
                    match Binding::command(self.context_bindings(), action) {
                        Some(ScrollKey::Up) => Some(Nav::Up),
                        Some(ScrollKey::Down) => Some(Nav::Down),
                        Some(ScrollKey::PageUp) => Some(Nav::PageUp),
                        Some(ScrollKey::PageDown) => Some(Nav::PageDown),
                        Some(ScrollKey::Home) => Some(Nav::Home),
                        Some(ScrollKey::End) => Some(Nav::End),
                        None => None,
                    }
                }
                Intent::Key(key) if self.context_type_ahead_enabled() => {
                    let Some(character) = TypeAhead::char_of(&key) else {
                        continue;
                    };
                    let vim_shortcut = self.profile == 1
                        && self.precedence == 0
                        && matches!(character, 'j' | 'k' | 'g' | 'G');
                    if vim_shortcut {
                        continue;
                    }
                    let before = cursor.index;
                    cursor.type_ahead(character, &mut self.context_type_ahead, &rows);
                    if cursor.index != before {
                        self.context_scroll.follow(cursor.index);
                        response |= Response::changed().for_id(CONTEXT_SCROLL);
                    } else {
                        response |= Response::consumed().for_id(CONTEXT_SCROLL);
                    }
                    None
                }
                _ => None,
            };
            let Some(nav) = nav else {
                continue;
            };
            let before = cursor.index;
            cursor.apply(nav, &rows, self.context_scroll.viewport_len().max(1));
            if cursor.index != before {
                self.context_scroll.follow(cursor.index);
                response |= Response::changed().for_id(CONTEXT_SCROLL);
            } else {
                response |= Response::consumed().for_id(CONTEXT_SCROLL);
            }
        }
        if cursor.index != self.context_cursor {
            self.context_cursor = cursor.index;
        }
        if cx
            .intents(CONTEXT_SCROLL)
            .any(|intent| matches!(intent, Intent::FocusIn { .. }))
        {
            self.context_scroll.follow(self.context_cursor);
        }
        response
    }

    fn update_options(&mut self, cx: &mut Cx<'_>, controls_updated: bool) -> Response<()> {
        let mut response = if controls_updated {
            Response::ignored()
        } else {
            self.update_option_controls(cx)
        };
        if cx.layout(OPTIONS_VIEW).is_some() {
            let lines = option_log_lines(&self.log);
            response |= match self.option_revision {
                Some(revision) => options_view(self.wrap_lines, self.horizontal_pan)
                    .update_routed(cx, &mut self.options_view_state, &lines, revision)
                    .erase(),
                None => options_view(self.wrap_lines, self.horizontal_pan)
                    .update(cx, &mut self.options_view_state, &lines)
                    .erase(),
            };
            self.context_scroll = self
                .context_scroll
                .with_follow_cursor(self.options_view_state.follow_cursor());
        }
        response |= self.update_context_scroll(cx);
        response |= ScrollRegion::new(OPTIONS_SCROLL).update_routed(
            cx,
            &mut self.options_scroll,
            OPTIONS_ROWS,
            ROUTE_REVISION,
        );
        response |= update_scroll_keys(cx, OPTIONS_SCROLL, &mut self.options_scroll);
        for (id, row) in [
            (CONTEXT_OPTION, 1usize),
            (FOLLOW_OPTION, 3),
            (WRAP_OPTION, 4),
            (CURSOR_WRAP_OPTION, 5),
            (SKIP_DISABLED_OPTION, 6),
            (HORIZONTAL_PAN_OPTION, 7),
            (PROFILE_OPTION, 9),
            (TYPE_AHEAD_OPTION, 10),
            (TIMEOUT_OPTION, 11),
            (PRECEDENCE_OPTION, 12),
            (OPTIONS_VIEW, OPTIONS_VIEW_START),
            (CONTEXT_SCROLL, CONTEXT_VIEW_START),
        ] {
            if cx
                .intents(id)
                .any(|intent| matches!(intent, Intent::FocusIn { .. }))
            {
                self.options_scroll.ensure_visible(row);
            }
        }
        response
    }

    fn update_states(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = ScrollRegion::new(STATES_SCROLL).update_routed(
            cx,
            &mut self.states_scroll,
            STATE_COUNT.saturating_mul(STATE_CARD_ROWS),
            ROUTE_REVISION,
        );
        response |= update_scroll_keys(cx, STATES_SCROLL, &mut self.states_scroll);
        response
    }

    fn draw_playground(&self, ui: &mut Ui<'_>, body: Rect, width: u16) {
        let list_offset = self.list_state.scroll().offset();
        let log_offset = self.log.len().saturating_sub(6);
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
        if width < COLS_80_MIN_W {
            if slots[0].height > 0 {
                let list_area = content_narrow(narrow_columns(slots[0])[1]);
                list_view().draw(ui, list_area, &self.list_state, &self.list);
                paint_scrollers_narrow(ui, slots[0], list_offset, &self.log);
            }
            paint_page_thumb(ui, playground);
            return;
        }
        if width < FULL_MIN_W {
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
    }

    fn draw_options(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        ui.register_control(OPTIONS_SCROLL, area, Focusability::FocusableReadOnly);
        register_scroll_keys(ui, OPTIONS_SCROLL);
        let (view, parent_route) = ScrollRegion::new(OPTIONS_SCROLL).draw_routed(
            ui,
            area,
            &self.options_scroll,
            OPTIONS_ROWS,
            ROUTE_REVISION,
        );
        let Some(window) = scroll_window(view, OPTIONS_ROWS, &self.options_scroll) else {
            self.options_window.set((0, 0));
            self.options_window_valid.set(true);
            return;
        };
        let visible = window.visible_source_range();
        self.options_window.set((visible.start, visible.end));
        self.options_window_valid.set(true);
        for (row, label) in [(0usize, "Cursor reveal context"), (2, "Cursor follow")] {
            if let Some(y) = window.physical_row_for_source(row) {
                let _ = ui.paint_str(
                    Rect::new(view.x, y, view.width, 1),
                    label,
                    ui.surface_style(),
                );
            }
        }
        if let Some(y) = window.physical_row_for_source(8) {
            let _ = ui.paint_str(
                Rect::new(view.x, y, view.width, 1),
                "List navigation",
                ui.surface_style(),
            );
        }
        if let Some(y) = window.physical_row_for_source(13) {
            let _ = ui.paint_str(
                Rect::new(view.x, y, view.width, 1),
                "Live log · TextViewport",
                ui.surface_style(),
            );
        }
        if let Some(y) = window.physical_row_for_source(1) {
            let _ = Select::new(CONTEXT_OPTION).draw(
                ui,
                Rect::new(view.x, y, view.width, 1),
                &self.context_option_state,
                CONTEXT_ITEMS,
            );
        } else {
            register_focus_only(ui, CONTEXT_OPTION);
        }
        if let Some(y) = window.physical_row_for_source(3) {
            let _ = Select::new(FOLLOW_OPTION).draw(
                ui,
                Rect::new(view.x, y, view.width, 1),
                &self.follow_option_state,
                FOLLOW_ITEMS,
            );
        } else {
            register_focus_only(ui, FOLLOW_OPTION);
        }
        if let Some(y) = window.physical_row_for_source(4) {
            Toggle::new(WRAP_OPTION, "Wrap output lines")
                .on(self.wrap_lines)
                .draw(ui, Rect::new(view.x, y, view.width, 1));
        } else {
            register_focus_only(ui, WRAP_OPTION);
        }
        if let Some(y) = window.physical_row_for_source(5) {
            Toggle::new(CURSOR_WRAP_OPTION, "Wrap cursor at list ends")
                .on(self.cursor_wrap)
                .draw(ui, Rect::new(view.x, y, view.width, 1));
        } else {
            register_focus_only(ui, CURSOR_WRAP_OPTION);
        }
        if let Some(y) = window.physical_row_for_source(6) {
            Toggle::new(SKIP_DISABLED_OPTION, "Skip disabled rows")
                .on(self.skip_disabled)
                .draw(ui, Rect::new(view.x, y, view.width, 1));
        } else {
            register_focus_only(ui, SKIP_DISABLED_OPTION);
        }
        if let Some(y) = window.physical_row_for_source(7) {
            Toggle::new(HORIZONTAL_PAN_OPTION, "Allow sideways pan")
                .on(self.horizontal_pan)
                .draw(ui, Rect::new(view.x, y, view.width, 1));
        } else {
            register_focus_only(ui, HORIZONTAL_PAN_OPTION);
        }
        if let Some(y) = window.physical_row_for_source(9) {
            let _ = Select::new(PROFILE_OPTION).draw(
                ui,
                Rect::new(view.x, y, view.width, 1),
                &self.profile_option_state,
                PROFILE_ITEMS,
            );
        } else {
            register_focus_only(ui, PROFILE_OPTION);
        }
        if let Some(y) = window.physical_row_for_source(10) {
            let _ = Select::new(TYPE_AHEAD_OPTION).draw(
                ui,
                Rect::new(view.x, y, view.width, 1),
                &self.type_ahead_option_state,
                TYPE_AHEAD_ITEMS,
            );
        } else {
            register_focus_only(ui, TYPE_AHEAD_OPTION);
        }
        if let Some(y) = window.physical_row_for_source(11) {
            let _ = Select::new(TIMEOUT_OPTION).draw(
                ui,
                Rect::new(view.x, y, view.width, 1),
                &self.timeout_option_state,
                TIMEOUT_ITEMS,
            );
        } else {
            register_focus_only(ui, TIMEOUT_OPTION);
        }
        if let Some(y) = window.physical_row_for_source(12) {
            let _ = Select::new(PRECEDENCE_OPTION).draw(
                ui,
                Rect::new(view.x, y, view.width, 1),
                &self.precedence_option_state,
                PRECEDENCE_ITEMS,
            );
        } else {
            register_focus_only(ui, PRECEDENCE_OPTION);
        }
        if let Some(sample) = nested_window(
            window,
            OPTIONS_VIEW_START,
            OPTIONS_VIEW_ROWS,
            view.x,
            view.width,
        ) {
            let lines = option_log_lines(&self.log);
            let child_route = match self.option_revision {
                Some(revision) => {
                    options_view(self.wrap_lines, self.horizontal_pan)
                        .draw_window_routed(ui, sample, &self.options_view_state, &lines, revision)
                        .1
                }
                None => {
                    options_view(self.wrap_lines, self.horizontal_pan).draw_window(
                        ui,
                        sample,
                        &self.options_view_state,
                        &lines,
                    );
                    None
                }
            };
            if let (Some(child), Some(parent)) = (child_route, parent_route) {
                let _ = ui.link_scroll_parent(child, parent);
            }
        } else {
            register_focus_only(ui, OPTIONS_VIEW);
        }

        if let Some(y) = window.physical_row_for_source(22) {
            let _ = ui.paint_str(
                Rect::new(view.x, y, view.width, 1),
                "Cursor reveal · ScrollRegion",
                ui.surface_style(),
            );
        }
        if let Some(sample) = nested_window(
            window,
            CONTEXT_VIEW_START,
            CONTEXT_VIEW_ROWS,
            view.x,
            view.width,
        ) {
            let sample_area = sample.visible_body();
            ui.register_control(CONTEXT_SCROLL, sample_area, Focusability::FocusableReadOnly);
            ui.publish_bindings(
                CONTEXT_SCROLL,
                ui.state(CONTEXT_SCROLL),
                self.context_bindings(),
            );
            let (inner, child_route) = ScrollRegion::new(CONTEXT_SCROLL).draw_window_routed(
                ui,
                sample,
                &self.context_scroll,
                LIST_LEN,
                ROUTE_REVISION,
            );
            if let (Some(child), Some(parent)) = (child_route, parent_route) {
                let _ = ui.link_scroll_parent(child, parent);
            }
            if let Some(rows) = nested_scroll_window(sample, inner, LIST_LEN, &self.context_scroll)
            {
                for logical_row in rows.visible_source_range() {
                    let Some(y) = rows.physical_row_for_source(logical_row) else {
                        continue;
                    };
                    let row_area = Rect::new(inner.x, y, inner.width, 1);
                    if logical_row == self.context_cursor {
                        ui.fill(row_area, ink(WHITE, ACCENT_BG));
                    }
                    let label = format!(
                        "{} Row {:03} · context {}{}",
                        if logical_row == self.context_cursor {
                            "›"
                        } else {
                            " "
                        },
                        logical_row.saturating_add(1),
                        self.context_scroll.context(),
                        if context_row_enabled(logical_row) {
                            ""
                        } else {
                            " · disabled"
                        },
                    );
                    let _ = ui.paint_str(
                        row_area,
                        &truncate(&label, row_area.width),
                        ink(
                            if logical_row == self.context_cursor {
                                WHITE
                            } else if context_row_enabled(logical_row) {
                                SECONDARY
                            } else {
                                MUTED
                            },
                            if logical_row == self.context_cursor {
                                ACCENT_BG
                            } else {
                                SURFACE
                            },
                        ),
                    );
                }
            }
        } else {
            register_focus_only(ui, CONTEXT_SCROLL);
        }
    }

    fn draw_states(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        const LABELS: [&str; STATE_COUNT] =
            ["Rest", "Hover", "Pressed", "Empty", "Overflow", "Narrow"];
        ui.register_control(STATES_SCROLL, area, Focusability::FocusableReadOnly);
        register_scroll_keys(ui, STATES_SCROLL);
        let rows = STATE_COUNT.saturating_mul(STATE_CARD_ROWS);
        let (view, _) = ScrollRegion::new(STATES_SCROLL).draw_routed(
            ui,
            area,
            &self.states_scroll,
            rows,
            ROUTE_REVISION,
        );
        let Some(window) = scroll_window(view, rows, &self.states_scroll) else {
            return;
        };
        for (index, label) in LABELS.into_iter().enumerate() {
            let start = index.saturating_mul(STATE_CARD_ROWS);
            if let Some(y) = window.physical_row_for_source(start) {
                let _ = ui.paint_str(
                    Rect::new(view.x, y, view.width, 1),
                    label,
                    ui.surface_style(),
                );
            }
            let sample_id = STATE_SAMPLE.index(index);
            let ref_state = match index {
                1 => Some(ReferenceState::HOVERED),
                2 => Some(ReferenceState::PRESSED),
                _ => None,
            };
            if let Some(sample) = nested_window(
                window,
                start.saturating_add(1),
                STATE_SAMPLE_ROWS,
                view.x,
                if index == 5 {
                    view.width.min(18)
                } else {
                    view.width
                },
            ) {
                let mut state = ScrollState::default();
                let content = if index == 3 { 0 } else { 18 };
                state.set_content(content);
                state.set_viewport(sample.logical_rows());
                state.scroll_to(if index == 2 { 10 } else { 4 });
                let target = ref_state.map(|flags| {
                    ReferenceTarget::new(sample_id, flags).part(PartRef::of(Part::THUMB))
                });
                ui.reference(target, |ui| {
                    let inner =
                        ScrollRegion::new(sample_id).draw_window(ui, sample, &state, content);
                    for y in inner.y..inner.bottom() {
                        if let Some(visible_row) = sample.source_row_for_physical(y) {
                            let line = if content == 0 {
                                "empty viewport".to_owned()
                            } else {
                                format!(
                                    "line {:02} of {content}",
                                    state.offset().saturating_add(visible_row).saturating_add(1)
                                )
                            };
                            let _ = ui.paint_str(
                                Rect::new(inner.x, y, inner.width, 1),
                                &truncate(&line, inner.width),
                                ink(SECONDARY, SURFACE),
                            );
                        }
                    }
                });
            }
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
        if self.log.len() != count {
            self.log = log_lines(count);
            self.bump_option_revision();
        }
    }

    fn title(&self) -> &'static str {
        "Scrolling"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let mut response = Response::ignored();
        let popup_open = self.option_popup_open(cx);
        if popup_open {
            response |= self.update_option_controls(cx);
        }
        response |= Tabs::new(PAGE_TABS)
            .item_affixes(section_tab_affixes)
            .update(cx, &mut self.tabs, SECTIONS)
            .erase();
        match self.section() {
            0 => {
                let before = self.list_state.scroll().offset();
                response |= list_view()
                    .update(cx, &mut self.list_state, &self.list)
                    .erase();
                if self.list_state.scroll().offset() != before {
                    self.last = "manual scroll";
                }
            }
            1 => response |= self.update_options(cx, popup_open),
            _ => response |= self.update_states(cx),
        }
        response.into()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(ui, area, self.title(), BLURB, |ui, body| {
            let content_top = body.y.saturating_add(3);
            let content = Rect {
                y: content_top,
                height: body.bottom().saturating_sub(content_top),
                ..body
            };
            match self.section() {
                0 => self.draw_playground(ui, body, area.width),
                1 => self.draw_options(ui, content),
                _ => self.draw_states(ui, content),
            }
            let tabs_area = Rect {
                height: 2.min(body.height),
                ..body
            };
            let tabs = Tabs::new(PAGE_TABS).item_affixes(section_tab_affixes);
            let previous =
                ui.layout(PAGE_TABS)
                    .map(|facts| termrock::navigation::tabs::TabWindowReceipt {
                        viewport_len: facts.viewport_len,
                        content_len: facts.content_len,
                    });
            let layout = tabs.layout_default(tabs_area, area, &self.tabs, SECTIONS, previous);
            if tabs.draw_layout(ui, &layout).is_err() {
                return;
            }
            let focused_cursor = ui
                .state(PAGE_TABS)
                .contains(StateFlags::FOCUSED)
                .then(|| self.tabs.cursor())
                .flatten()
                .and_then(|key| match key {
                    ItemKey::Index(index) => Some(index),
                    _ => None,
                });
            paint_section_strip(ui, tabs_area, self.section(), focused_cursor, &layout);
        });
    }

    fn hints(&self, ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        self.hints_for_focused_owner(|id| ui.state(id).contains(StateFlags::FOCUSED))
    }
}

#[cfg(test)]
mod motion_tests {
    use super::*;
    use termrock::{App, Axis, KeyCode, Theme};
    use termrock_test_support::Harness;

    struct Host {
        page: ScrollingPage,
        context_layout: Cell<Option<termrock::LayoutFacts>>,
    }

    impl App for Host {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            self.context_layout.set(cx.layout(CONTEXT_SCROLL));
            self.page.update(cx).response
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            self.page.draw(ui, ui.full());
        }
    }

    fn host(width: u16, height: u16) -> Harness<Host> {
        Harness::new(
            Host {
                page: ScrollingPage::new(),
                context_layout: Cell::new(None),
            },
            Theme::junie(),
            width,
            height,
        )
    }

    fn select_section(h: &mut Harness<Host>, index: char) {
        assert!(h.tab_to(PAGE_TABS));
        let _ = h.key(KeyCode::Char(index));
    }

    fn choose_next(h: &mut Harness<Host>, id: Id) {
        assert!(h.tab_to(id));
        let _ = h.key(KeyCode::Enter);
        let _ = h.key(KeyCode::Down);
        let _ = h.key(KeyCode::Enter);
    }

    fn wheel_options_parent(h: &mut Harness<Host>, outer: Rect, delta: i16, child: Id) {
        let track = h
            .area_of_part(OPTIONS_SCROLL, PartRef::of(Part::TRACK))
            .expect("the outer Options scrollbar track is a live target");
        let strip = Rect {
            x: outer.right().saturating_sub(1),
            width: 1,
            ..outer
        };
        assert_eq!(
            track, strip,
            "the parent track occupies its reserved rightmost strip"
        );
        if let Some(child_area) = h.area_of(child) {
            assert!(
                strip.intersection(child_area).is_empty(),
                "the parent track stays outside the child hit area"
            );
        }
        let page_before = h.app().page.options_scroll.offset();
        let child_offset_before = match child {
            OPTIONS_VIEW => Some(h.app().page.options_view_state.scroll().offset()),
            CONTEXT_SCROLL => Some(h.app().page.context_scroll.offset()),
            _ => None,
        };

        let _ = h.wheel(Axis::V, delta, strip.x, strip.y);

        let page_after = h.app().page.options_scroll.offset();
        if delta > 0 {
            assert!(page_after > page_before, "the outer page scrolls down");
        } else if delta < 0 {
            assert!(page_after < page_before, "the outer page scrolls up");
        }
        let child_offset_after = match child {
            OPTIONS_VIEW => Some(h.app().page.options_view_state.scroll().offset()),
            CONTEXT_SCROLL => Some(h.app().page.context_scroll.offset()),
            _ => None,
        };
        assert_eq!(
            child_offset_after, child_offset_before,
            "parent-only wheel preparation leaves the child offset unchanged"
        );
    }

    #[test]
    fn default_gallery_cells_and_focus_order_are_preserved() {
        let mut h = host(180, 60);
        let text = h.text();
        for cell in [
            "Wrapped text",
            "Long list",
            "Log",
            "Hand-drawn",
            "Divider or bar",
            "Cursor",
            "Type to jump · select, tree, table, grid",
        ] {
            assert!(text.contains(cell), "missing authored cell {cell}: {text}");
        }
        let entries = h.ring().entries();
        let list_index = entries
            .iter()
            .position(|entry| entry.id == LIST_VIEW)
            .expect("the original list remains in the focus ring");
        let tabs_index = entries
            .iter()
            .position(|entry| entry.id == PAGE_TABS)
            .expect("the new section tabs are focusable");
        assert!(
            list_index < tabs_index,
            "body controls register before tabs"
        );
        assert!(h.tab_to(LIST_VIEW));
        assert_eq!(h.focus(), Some(LIST_VIEW));
    }

    #[test]
    fn context_option_changes_native_scroll_state_and_keeps_viewport_offset() {
        let mut h = host(90, 48);
        select_section(&mut h, '2');
        assert_eq!(h.app().page.section(), 1);
        assert!(h.tab_to(OPTIONS_VIEW));
        let _ = h.key(KeyCode::End);
        let before = h.app().page.options_view_state.scroll().offset();
        assert!(before > 0);

        assert!(h.tab_to(CONTEXT_OPTION));
        let _ = h.key(KeyCode::Enter);
        let _ = h.key(KeyCode::Down);
        let _ = h.key(KeyCode::Enter);
        assert_eq!(h.app().page.context_option, 2);
        assert_eq!(h.app().page.context_scroll.context(), 2);
        assert_eq!(
            h.app().page.options_view_state.scroll().offset(),
            before,
            "an unrelated context change leaves the output viewport in place"
        );
    }

    #[test]
    fn wrap_and_follow_options_change_the_native_text_viewport() {
        let mut h = host(64, 48);
        select_section(&mut h, '2');
        assert!(h.tab_to(WRAP_OPTION));
        let unwrapped = h.app().page.options_view_state.scroll().content_len();
        let _ = h.key(KeyCode::Char(' '));
        assert!(h.app().page.wrap_lines);
        let wrapped = h.app().page.options_view_state.scroll().content_len();
        assert!(wrapped > unwrapped, "wrapping changes actual viewport rows");

        assert!(h.tab_to(FOLLOW_OPTION));
        let _ = h.key(KeyCode::Enter);
        let _ = h.key(KeyCode::Down);
        let _ = h.key(KeyCode::Enter);
        assert_eq!(
            h.app().page.context_scroll.follow_cursor(),
            FollowCursor::Always
        );
        h.app_mut().page.seek_paused(12);
        let _ = h.key(KeyCode::Tab);
        assert!(
            h.app().page.options_view_state.scroll().at_end(),
            "Always follows appended output"
        );
    }

    #[test]
    fn wheel_routes_to_nested_owner_without_page_snapback() {
        let mut h = host(100, 48);
        select_section(&mut h, '2');
        let child = h
            .area_of(CONTEXT_SCROLL)
            .expect("the nested native ScrollRegion is visible");
        let wheel_rows = usize::from(h.runtime().theme().design.motion.wheel_rows);
        assert!(wheel_rows > 0, "the active theme defines a wheel stride");
        let inner_before = h.app().page.context_scroll.offset();
        let page_before = h.app().page.options_scroll.offset();
        let _ = h.wheel(
            Axis::V,
            1,
            child.x.saturating_add(1),
            child.y.saturating_add(child.height / 2),
        );
        let layout = h
            .app()
            .context_layout
            .get()
            .expect("the nested scroll publishes committed layout facts");
        assert_eq!(layout.content_len, LIST_LEN);
        assert_eq!(layout.viewport_len, CONTEXT_VIEW_ROWS);
        let logical_rows = layout
            .vertical_window
            .map_or(layout.viewport_len, VerticalWindow::logical_rows);
        assert_eq!(logical_rows, CONTEXT_VIEW_ROWS);
        let max_offset = layout.content_len.saturating_sub(logical_rows);
        assert!(
            max_offset.saturating_sub(inner_before) >= wheel_rows,
            "the nested owner has enough headroom for one theme wheel stride"
        );
        assert!(h.app().page.context_scroll.offset() > inner_before);
        assert_eq!(h.app().page.options_scroll.offset(), page_before);
        assert_eq!(
            h.app().page.context_scroll.offset(),
            inner_before.saturating_add(wheel_rows),
            "the first wheel moves the nested owner by the active theme stride"
        );
        let moved = h.app().page.context_scroll.offset();
        let page_after_first = h.app().page.options_scroll.offset();
        let _ = h.wheel(
            Axis::V,
            1,
            child.x.saturating_add(1),
            child.y.saturating_add(child.height / 2),
        );
        let layout_before_second_wheel = h
            .app()
            .context_layout
            .get()
            .expect("the second wheel observes the committed nested layout before service");
        assert_eq!(layout_before_second_wheel.content_len, LIST_LEN);
        assert_eq!(layout_before_second_wheel.viewport_len, CONTEXT_VIEW_ROWS);
        let second_logical_rows = layout_before_second_wheel.vertical_window.map_or(
            layout_before_second_wheel.viewport_len,
            VerticalWindow::logical_rows,
        );
        assert_eq!(second_logical_rows, CONTEXT_VIEW_ROWS);
        let second_max_offset = layout_before_second_wheel
            .content_len
            .saturating_sub(second_logical_rows);
        assert!(
            second_max_offset.saturating_sub(moved) >= wheel_rows,
            "the nested owner retains enough headroom for the next theme wheel stride"
        );
        assert_eq!(
            h.app().page.context_scroll.offset(),
            moved.saturating_add(wheel_rows),
            "the next update does not reveal the old cursor over the wheel move"
        );
        assert_eq!(h.app().page.options_scroll.offset(), page_after_first);
        assert_eq!(h.app().page.options_scroll.offset(), page_before);

        assert!(h.tab_to(CONTEXT_SCROLL));
        let _ = h.key(KeyCode::End);
        assert_eq!(h.app().page.context_cursor, LIST_LEN - 2);
        assert!(h.app().page.context_scroll.offset() > moved);
    }

    #[test]
    fn cursor_wrap_and_disabled_options_change_real_row_navigation() {
        let mut h = host(100, 48);
        select_section(&mut h, '2');
        assert!(h.tab_to(CONTEXT_SCROLL));
        let _ = h.key(KeyCode::End);
        assert_eq!(h.app().page.context_cursor, LIST_LEN - 2);

        assert!(h.tab_to(CURSOR_WRAP_OPTION));
        let _ = h.key(KeyCode::Char(' '));
        assert!(h.app().page.cursor_wrap);
        assert!(h.tab_to(CONTEXT_SCROLL));
        let _ = h.key(KeyCode::Down);
        assert_eq!(
            h.app().page.context_cursor,
            0,
            "wrapping crosses the disabled final row"
        );

        assert!(h.tab_to(SKIP_DISABLED_OPTION));
        let _ = h.key(KeyCode::Char(' '));
        assert!(!h.app().page.skip_disabled);
        assert!(h.tab_to(CONTEXT_SCROLL));
        let _ = h.key(KeyCode::End);
        assert_eq!(h.app().page.context_cursor, LIST_LEN - 1);
    }

    #[test]
    fn profile_type_ahead_timeout_and_precedence_are_per_instance() {
        let mut h = host(100, 48);
        select_section(&mut h, '2');

        choose_next(&mut h, PROFILE_OPTION);
        assert_eq!(h.app().page.profile, 1);
        choose_next(&mut h, TYPE_AHEAD_OPTION);
        assert!(h.app().page.context_type_ahead_enabled());
        assert!(h.tab_to(CONTEXT_SCROLL));
        let _ = h.key(KeyCode::Char('j'));
        assert_eq!(h.app().page.context_cursor, 1, "Vim j moves this cursor");
        let _ = h.key(KeyCode::Char('k'));
        assert_eq!(h.app().page.context_cursor, 0, "Vim k moves this cursor");
        h.app_mut().page.context_cursor = 8;
        h.app_mut().page.context_scroll.scroll_to(8);
        h.app_mut().page.context_type_ahead.reset();
        h.draw();
        let _ = h.key(KeyCode::Char('g'));
        assert_eq!(h.app().page.context_cursor, 0, "Vim g returns to the start");
        let _ = h.key(KeyCode::Char('G'));
        assert_eq!(
            h.app().page.context_cursor,
            LIST_LEN - 2,
            "Vim G returns to the last enabled row"
        );

        choose_next(&mut h, PRECEDENCE_OPTION);
        assert_eq!(h.app().page.precedence, 1);
        assert!(h.tab_to(CONTEXT_SCROLL));
        h.app_mut().page.context_cursor = 5;
        h.app_mut().page.context_scroll.scroll_to(5);
        h.app_mut().page.context_type_ahead.reset();
        h.draw();
        let _ = h.key(KeyCode::Char('j'));
        assert_eq!(
            h.app().page.context_cursor,
            5,
            "type-ahead receives Vim j instead of moving the cursor"
        );
        h.app_mut().page.context_type_ahead.reset();
        let _ = h.key(KeyCode::Char('g'));
        assert_eq!(h.app().page.context_cursor, 5, "type-ahead wins over Vim g");
        h.app_mut().page.context_type_ahead.reset();
        let _ = h.key(KeyCode::Char('G'));
        assert_eq!(h.app().page.context_cursor, 5, "type-ahead wins over Vim G");

        assert!(h.tab_to(TIMEOUT_OPTION));
        let _ = h.key(KeyCode::Enter);
        let _ = h.key(KeyCode::Down);
        let _ = h.key(KeyCode::Enter);
        assert_eq!(
            h.app().page.context_type_ahead.timeout,
            Duration::from_secs(2)
        );
    }

    #[test]
    fn horizontal_pan_toggle_updates_the_native_viewport_configuration() {
        let mut h = host(60, 48);
        select_section(&mut h, '2');
        let revision = h.app().page.option_revision;
        assert!(h.tab_to(HORIZONTAL_PAN_OPTION));
        let _ = h.key(KeyCode::Char(' '));
        assert!(h.app().page.horizontal_pan);
        assert!(h.app().page.option_revision > revision);
        assert!(h.tab_to(OPTIONS_VIEW));
        assert_eq!(h.app().page.options_view_state.horizontal_offset(), 0);
        let _ = h.key(KeyCode::Right);
        assert!(
            h.app().page.options_view_state.horizontal_offset() > 0,
            "Right pans the actual unwrapped native TextViewport"
        );
    }

    #[test]
    fn select_outside_click_is_consumed_and_hidden_references_have_no_hits() {
        let mut h = host(100, 48);
        select_section(&mut h, '2');
        assert!(h.tab_to(CONTEXT_OPTION));
        let _ = h.key(KeyCode::Enter);
        assert!(h.is_open(CONTEXT_OPTION));
        let tabs = h.area_of(PAGE_TABS).expect("section tabs are visible");
        let _ = h.click(tabs.x.saturating_add(1), tabs.y);
        assert_eq!(
            h.app().page.section(),
            1,
            "dismissal consumes the first click"
        );
        assert!(!h.is_open(CONTEXT_OPTION));
        let _ = h.click(tabs.x.saturating_add(1), tabs.y);
        assert_eq!(h.app().page.section(), 0);

        let mut states = host(100, 48);
        select_section(&mut states, '3');
        assert_eq!(states.app().page.section(), 2);
        assert!(states.text().contains("Overflow"));
        for index in 0..STATE_COUNT {
            let id = STATE_SAMPLE.index(index);
            assert!(
                states.area_of(id).is_none(),
                "reference {index} has no hit area"
            );
            assert!(
                !states.ring().is_registered(id),
                "reference {index} is inert"
            );
        }
        assert!(states.area_of(CONTEXT_OPTION).is_none());
    }

    #[test]
    fn clipped_samples_keep_their_logical_origin_and_partial_rows() {
        let parent = VerticalWindow::new(Rect::new(4, 8, 20, 3), 10, 4).unwrap();
        let sample = nested_window(parent, 4, 4, 4, 20).unwrap();
        assert_eq!(sample.visible_source_range(), 0..3);
        assert_eq!(sample.first_source_row(), 0);
        assert_eq!(sample.physical_row_for_source(2), Some(10));
        assert_eq!(sample.physical_row_for_source(3), None);

        let mut h = host(80, 16);
        select_section(&mut h, '3');
        assert!(
            h.text().contains("Rest"),
            "the first clipped specimen stays visible"
        );
        assert!(h.tab_to(STATES_SCROLL));
        let area = h
            .area_of(STATES_SCROLL)
            .expect("compact states have a real scroll owner");
        let _ = h.key(KeyCode::PageDown);
        assert!(h.app().page.states_scroll.offset() > 0);
        let _ = h.wheel(Axis::V, 1, area.x, area.y);
        assert!(h.app().page.states_scroll.offset() > 0);
    }

    #[test]
    fn clipped_log_keeps_logical_window_origin_size_and_wheel_owner() {
        let mut h = host(80, 12);
        select_section(&mut h, '2');
        let outer = h
            .area_of(OPTIONS_SCROLL)
            .expect("the compact Options owner is visible");

        for _ in 0..OPTIONS_ROWS {
            let (start, _) = h.app().page.options_window.get();
            let first_source_row = start.saturating_sub(OPTIONS_VIEW_START);
            if matches!(first_source_row, 2 | 4 | 6) {
                break;
            }
            wheel_options_parent(&mut h, outer, 1, OPTIONS_VIEW);
        }

        let (outer_start, _) = h.app().page.options_window.get();
        let first_source_row = outer_start.saturating_sub(OPTIONS_VIEW_START);
        assert!(
            matches!(first_source_row, 2 | 4 | 6),
            "the outer owner clips the log at a visible nonblank source row"
        );
        let sample = h
            .area_of(OPTIONS_VIEW)
            .expect("the clipped TextViewport remains physically visible");
        assert_eq!(
            h.app().page.options_view_state.scroll().viewport_len(),
            OPTIONS_VIEW_ROWS,
            "the native TextViewport retains the eight-row logical viewport"
        );
        assert_eq!(
            h.app().page.options_view_state.scroll().offset(),
            0,
            "outer clipping does not advance the inner log"
        );
        let expected = match first_source_row {
            2 => "Each step is visible.",
            4 => "The design system in this prototype",
            6 => "Scroll with the mouse wheel",
            _ => "",
        };
        assert!(!expected.is_empty());
        assert!(
            h.text().contains(expected),
            "the first painted line follows source row {first_source_row}: {expected}"
        );
        assert!(
            !h.text().contains("Junie works through a task"),
            "rows above the visible window are not restarted at the physical clip"
        );

        let page_offset = h.app().page.options_scroll.offset();
        let log_offset = h.app().page.options_view_state.scroll().offset();
        let wheel_rows = usize::from(h.runtime().theme().design.motion.wheel_rows);
        assert!(wheel_rows > 0, "the active theme defines a wheel stride");
        assert!(
            h.app()
                .page
                .options_view_state
                .scroll()
                .max_offset()
                .saturating_sub(log_offset)
                >= wheel_rows,
            "the nested log has enough headroom for one theme wheel stride"
        );
        let _ = h.wheel(Axis::V, 1, sample.x.saturating_add(1), sample.y);
        assert_eq!(
            h.app().page.options_view_state.scroll().offset(),
            log_offset.saturating_add(wheel_rows),
            "the clipped nested log owns its routed wheel input"
        );
        assert_eq!(
            h.app().page.options_scroll.offset(),
            page_offset,
            "the nested log wheel does not move the page owner"
        );
    }

    #[test]
    fn compact_nested_samples_reveal_on_focus_and_wheel_hidden_cursor_is_inert() {
        let mut h = host(80, 12);
        select_section(&mut h, '2');
        assert!(h.ring().is_registered(OPTIONS_VIEW));
        assert!(h.tab_to(OPTIONS_VIEW));
        assert!(h.area_of(OPTIONS_VIEW).is_some());
        let before = h.app().page.options_view_state.scroll().offset();
        let _ = h.key(KeyCode::End);
        assert!(
            h.app().page.options_view_state.scroll().offset() > before,
            "focus reveal makes the compact native TextViewport respond to End"
        );

        assert!(h.tab_to(CONTEXT_SCROLL));
        assert!(h.area_of(CONTEXT_SCROLL).is_some());
        let _ = h.key(KeyCode::End);
        assert_eq!(h.app().page.context_cursor, LIST_LEN - 2);

        assert!(h.ring().is_registered(HORIZONTAL_PAN_OPTION));
        let outer = h
            .area_of(OPTIONS_SCROLL)
            .expect("the compact Options owner remains visible");
        for _ in 0..OPTIONS_ROWS {
            if h.area_of(CONTEXT_SCROLL).is_none() {
                break;
            }
            wheel_options_parent(&mut h, outer, -1, CONTEXT_SCROLL);
        }
        assert!(h.area_of(CONTEXT_SCROLL).is_none());
        let cursor = h.app().page.context_cursor;
        let offset = h.app().page.context_scroll.offset();
        let _ = h.key(KeyCode::End);
        assert_eq!(h.app().page.context_cursor, cursor);
        assert_eq!(h.app().page.context_scroll.offset(), offset);
    }

    #[test]
    fn wheel_hidden_toggle_has_no_hit_area_and_cannot_activate() {
        let mut h = host(80, 12);
        select_section(&mut h, '2');
        assert!(h.tab_to(HORIZONTAL_PAN_OPTION));
        assert!(h.area_of(HORIZONTAL_PAN_OPTION).is_some());
        assert!(!h.app().page.horizontal_pan);

        let outer = h
            .area_of(OPTIONS_SCROLL)
            .expect("the compact Options owner remains visible");
        for _ in 0..OPTIONS_ROWS {
            if h.area_of(HORIZONTAL_PAN_OPTION).is_none() {
                break;
            }
            wheel_options_parent(&mut h, outer, 1, HORIZONTAL_PAN_OPTION);
        }
        assert!(h.area_of(HORIZONTAL_PAN_OPTION).is_none());
        let _ = h.key(KeyCode::Char(' '));
        assert!(
            !h.app().page.horizontal_pan,
            "a focused but wheel-hidden toggle remains inert without layout"
        );
    }

    #[test]
    fn clipped_cursor_keeps_logical_rows_content_origin_and_thumb_coordinates() {
        let mut h = host(80, 12);
        select_section(&mut h, '2');
        assert!(h.tab_to(CONTEXT_SCROLL));
        let _ = h.key(KeyCode::Home);

        let outer = h
            .area_of(OPTIONS_SCROLL)
            .expect("the compact Options owner remains visible");
        for _ in 0..OPTIONS_ROWS {
            let (start, _) = h.app().page.options_window.get();
            if start.saturating_sub(CONTEXT_VIEW_START) > 0 {
                break;
            }
            wheel_options_parent(&mut h, outer, 1, CONTEXT_SCROLL);
        }

        let (outer_start, _) = h.app().page.options_window.get();
        let first_source_row = outer_start
            .saturating_sub(CONTEXT_VIEW_START)
            .min(CONTEXT_VIEW_ROWS);
        assert!(first_source_row > 0, "the outer page clips the sample top");
        h.app_mut()
            .page
            .context_scroll
            .scroll_to(first_source_row.saturating_mul(16));
        h.draw();
        let sample_area = h
            .area_of(CONTEXT_SCROLL)
            .expect("the clipped cursor sample still has a visible window");
        assert_eq!(
            h.app().page.context_scroll.viewport_len(),
            CONTEXT_VIEW_ROWS,
            "the nested owner retains its eight-row logical viewport"
        );

        let expected_content_row = h
            .app()
            .page
            .context_scroll
            .offset()
            .saturating_add(first_source_row)
            .saturating_add(1);
        assert!(h.text().contains(&format!(
            "Row {expected_content_row:03} · context {}",
            h.app().page.context_scroll.context()
        )));

        let (thumb_start, thumb_len) = h.app().page.context_scroll.thumb(CONTEXT_VIEW_ROWS);
        assert!(thumb_start >= first_source_row);
        assert!(thumb_start < first_source_row.saturating_add(usize::from(sample_area.height)));
        let thumb = h
            .area_of_part(CONTEXT_SCROLL, PartRef::of(Part::THUMB))
            .expect("the logical thumb intersects the clipped viewport");
        assert_eq!(
            thumb.y,
            sample_area.y.saturating_add(
                u16::try_from(thumb_start.saturating_sub(first_source_row)).unwrap()
            )
        );
        assert_eq!(thumb.height, u16::try_from(thumb_len).unwrap());
        let offset_before_thumb_press = h.app().page.context_scroll.offset();
        let _ = h.click_part(CONTEXT_SCROLL, PartRef::of(Part::THUMB));
        assert_eq!(
            h.app().page.context_scroll.offset(),
            offset_before_thumb_press,
            "pressing the visible thumb uses its logical track coordinate"
        );
    }

    #[test]
    fn compact_hints_follow_the_focused_option_or_scroll_owner() {
        let mut h = host(60, 16);
        select_section(&mut h, '2');

        assert!(h.tab_to(CONTEXT_OPTION));
        let focus = h.focus();
        assert_eq!(
            h.app().page.hints_for_focused_owner(|id| focus == Some(id)),
            SELECT_HINTS
        );
        let _ = h.key(KeyCode::Enter);
        assert!(h.is_open(CONTEXT_OPTION));
        let focus = h.focus();
        assert_eq!(
            h.app().page.hints_for_focused_owner(|id| focus == Some(id)),
            SELECT_OPEN_HINTS
        );
        let _ = h.key(KeyCode::Esc);

        assert!(h.tab_to(WRAP_OPTION));
        let focus = h.focus();
        assert_eq!(
            h.app().page.hints_for_focused_owner(|id| focus == Some(id)),
            TOGGLE_HINTS
        );

        assert!(h.tab_to(OPTIONS_SCROLL));
        let focus = h.focus();
        assert_eq!(
            h.app().page.hints_for_focused_owner(|id| focus == Some(id)),
            SCROLL_HINTS
        );

        assert!(h.tab_to(OPTIONS_VIEW));
        let focus = h.focus();
        assert_eq!(
            h.app().page.hints_for_focused_owner(|id| focus == Some(id)),
            SCROLL_HINTS
        );

        assert!(h.tab_to(HORIZONTAL_PAN_OPTION));
        let _ = h.key(KeyCode::Char(' '));
        assert!(h.tab_to(OPTIONS_VIEW));
        let focus = h.focus();
        assert_eq!(
            h.app().page.hints_for_focused_owner(|id| focus == Some(id)),
            PAN_SCROLL_HINTS
        );
    }

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
