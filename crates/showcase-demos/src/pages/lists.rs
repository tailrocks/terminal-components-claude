use termrock::{
    Color, Cx, EmptyState, Focusability, FrameRead, GlyphRole, Id, ItemKey, List, ListAction,
    ListState, Modifier, PaintStyle, Part, PartRef, Rect, Response, SelectMode, StateFlags,
    StylePatch, Ui, width,
};

use showcase_data::LANGUAGES;

use super::{Page, PageUpdate, frame};

const SINGLE: Id = termrock::Id::root("showcase_ui::pages::lists::lists.single");
const MULTI: Id = termrock::Id::root("showcase_ui::pages::lists::lists.multi");
const EMPTY: Id = termrock::Id::root("showcase_ui::pages::lists::lists.empty");
const LIST_GUTTER: &[(Part, StylePatch)] = &[(
    Part::GUTTER,
    StylePatch::new().set_glyph(GlyphRole::FocusBar),
)];

const SECTIONS: [&str; 3] = ["Playground", "Options", "States"];
const SECTION_CAPTION: &str = "the live widget";
const SECTION_WIDE: u16 = 76;
const CARD_H: u16 = 18;
const CARD_GAP_X: u16 = 2;
const CARD_GAP_Y: u16 = 1;
const SCROLL_ROOM: u16 = 2;
const LANG_MIN: u16 = 22;
const FILES_MIN: u16 = 40;
const SEARCH_MIN: u16 = 34;
const FADE_OUTER: f32 = 0.55;
const FADE_INNER: f32 = 0.8;

const CANVAS: u32 = 0x000000;
const SURFACE: u32 = 0x111111;
const WHITE: u32 = 0xffffff;
const SECONDARY: u32 = 0xb3b3b3;
const MUTED: u32 = 0x8a8a8a;
const FAINT: u32 = 0x737373;
const RULE: u32 = 0x3a3a3a;
const ACCENT: u32 = 0x48e054;
const ACCENT_BG: u32 = 0x173f1c;
const TAB_ACTIVE: u32 = 0x27272a;
const THUMB: u32 = 0x808080;
const TRACK: u32 = 0x474747;
const DISABLED: u32 = 0x4d4d4d;
const WARNING: u32 = 0xf59e09;
const SUCCESS: u32 = 0x3fb950;

const READOUT: [&str; 2] = [
    "nav default · type-ahead on (TypeAhead, 1000 ms pause) · click ClickPolicy { activate: Double, edit_on: Double, check: BoxOnly }, check BoxOnly · wrap false · skip_disabled true",
    "toggle platform+click · scrollbar_gap 1 · track_click Page · reveal tooltip · a ListCmd::ToggleAll · ⌘A ListCmd::SelectAll",
];
// Equal columns and this readout kick in at the 160×50 shell width; any height works.
const EQUAL_COLUMNS_MIN_W: u16 = 134;
const WIDE_READOUT: [&str; 2] = [
    "nav default · type-ahead on (TypeAhead, 1000 ms pause) · click Double, check BoxOnly · wrap false · skip_disabled true",
    "toggle platform+click · scrollbar_gap 1 · track_click Page · reveal tooltip · a ListCmd::ToggleAll · Ctrl+A ListCmd::SelectAll",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FileRow {
    key: u8,
    label: &'static str,
    meta: &'static str,
    disabled: bool,
}

const FILES: &[FileRow] = &[
    FileRow {
        key: 1,
        label: "src/api/auth.rs",
        meta: "modified",
        disabled: false,
    },
    FileRow {
        key: 2,
        label: "src/api/billing.rs",
        meta: "modified",
        disabled: false,
    },
    FileRow {
        key: 3,
        label: "src/db/schema.rs",
        meta: "generated",
        disabled: true,
    },
    FileRow {
        key: 4,
        label: "tests/checkout.rs",
        meta: "new",
        disabled: false,
    },
    FileRow {
        key: 5,
        label: "Cargo.lock",
        meta: "locked",
        disabled: true,
    },
    FileRow {
        key: 6,
        label: "docs/webhooks.md",
        meta: "modified",
        disabled: false,
    },
    FileRow {
        key: 7,
        label: "src/workers/mailer.rs",
        meta: "modified",
        disabled: false,
    },
    FileRow {
        key: 8,
        label: "src/config.rs",
        meta: "modified",
        disabled: false,
    },
    FileRow {
        key: 9,
        label: "README.md",
        meta: "modified",
        disabled: false,
    },
    FileRow {
        key: 10,
        label: "src/main.rs",
        meta: "modified",
        disabled: false,
    },
    FileRow {
        key: 11,
        label: "tests/auth_flow.rs",
        meta: "new",
        disabled: false,
    },
    FileRow {
        key: 12,
        label: "src/db/pool.rs",
        meta: "modified",
        disabled: false,
    },
];

fn language_key(value: &&'static str) -> ItemKey {
    ItemKey::text(value)
}

fn language_row(value: &&'static str, row: &mut termrock::RowUi<'_>) {
    row.label(value);
}

fn file_key(value: &FileRow) -> ItemKey {
    ItemKey::num(u64::from(value.key))
}

fn file_row(value: &FileRow, row: &mut termrock::RowUi<'_>) {
    row.label(value.label);
    row.meta(value.meta);
}

fn file_disabled(value: &FileRow) -> bool {
    value.disabled
}

fn single_list() -> List<
    'static,
    &'static str,
    impl Fn(&&'static str) -> ItemKey,
    impl Fn(&&'static str, &mut termrock::RowUi<'_>),
> {
    List::new(SINGLE)
        .key(language_key)
        .row(language_row)
        .patch_part(LIST_GUTTER)
}

fn multi_list()
-> List<'static, FileRow, impl Fn(&FileRow) -> ItemKey, impl Fn(&FileRow, &mut termrock::RowUi<'_>)>
{
    List::new(MULTI)
        .key(file_key)
        .row(file_row)
        .select_mode(SelectMode::Multi)
        .patch_part(LIST_GUTTER)
        .disabled_item(&file_disabled)
}

fn empty_list() -> List<'static, &'static str> {
    List::new(EMPTY).empty(EmptyState::Empty {
        title: "No results for “retry”",
        hint: None,
    })
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

fn mix(fg: u32, bg: u32, keep: f32) -> u32 {
    let channel = |shift: u32| {
        let from = f32::from(((fg >> shift) & 0xff) as u8);
        let into = f32::from(((bg >> shift) & 0xff) as u8);
        (into + (from - into) * keep).round().clamp(0.0, 255.0) as u32
    };
    (channel(16) << 16) | (channel(8) << 8) | channel(0)
}

fn paint_blurb(ui: &mut Ui<'_>, body: Rect) {
    let blurb = "Single and multiple selection, disabled items, scrolling, empty state";
    let title_w = width("Lists");
    if body.width <= title_w.saturating_add(4) {
        return;
    }
    let meta = Rect::new(
        body.x.saturating_add(title_w).saturating_add(2),
        body.y.saturating_sub(2),
        body.width.saturating_sub(title_w).saturating_sub(3),
        1,
    );
    let style = ink(MUTED, CANVAS);
    if width(blurb) <= meta.width {
        ui.paint_str(meta, blurb, style);
        return;
    }
    if meta.width == 0 {
        return;
    }
    let budget = Rect {
        width: meta.width.saturating_sub(1),
        ..meta
    };
    let used = ui.paint_str(budget, blurb, style);
    ui.paint_str(
        Rect::new(meta.x.saturating_add(used), meta.y, 1, 1),
        "…",
        style,
    );
}

fn tab_width(label: &str) -> u16 {
    width(label).saturating_add(3)
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
        ui.paint_str(
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
            ui.paint_str(
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
    ui.paint_str(
        Rect {
            y: rule_y,
            height: 1,
            ..area
        },
        &"─".repeat(usize::from(area.width)),
        ink(RULE, CANVAS),
    );
    let active = tab_width(SECTIONS[0]).min(area.width);
    ui.paint_str(
        Rect::new(area.x, rule_y, active, 1),
        &"━".repeat(usize::from(active)),
        ink(ACCENT, CANVAS),
    );
}

struct CardLayout {
    rects: [Rect; 3],
    height: u16,
    width: u16,
}

fn card_layout(view_width: u16) -> CardLayout {
    let mins = [LANG_MIN, FILES_MIN, SEARCH_MIN];
    let width = view_width.max(mins.into_iter().max().unwrap_or(0));
    let sum = mins
        .iter()
        .copied()
        .sum::<u16>()
        .saturating_add(CARD_GAP_X.saturating_mul(2));
    let mut rects = [Rect::ZERO; 3];
    if sum <= width && width > 0 {
        let spare = width.saturating_sub(sum);
        let share = spare / 3;
        let mut extra = spare % 3;
        let mut x = 0u16;
        for (index, min) in mins.into_iter().enumerate() {
            let mut w = min.saturating_add(share);
            if extra > 0 {
                w = w.saturating_add(1);
                extra -= 1;
            }
            if index + 1 == mins.len() {
                w = width.saturating_sub(x);
            }
            rects[index] = Rect::new(x, 0, w, CARD_H);
            x = x.saturating_add(w).saturating_add(CARD_GAP_X);
        }
        CardLayout {
            rects,
            height: CARD_H,
            width,
        }
    } else {
        let mut y = 0u16;
        for (index, _) in mins.iter().enumerate() {
            rects[index] = Rect::new(0, y, width, CARD_H);
            y = y.saturating_add(CARD_H);
            if index + 1 != mins.len() {
                y = y.saturating_add(CARD_GAP_Y);
            }
        }
        CardLayout {
            rects,
            height: y,
            width,
        }
    }
}

fn equal_card_layout(view_width: u16) -> CardLayout {
    let usable = view_width.saturating_sub(CARD_GAP_X.saturating_mul(2));
    let base = usable / 3;
    let widths = [base, base, usable.saturating_sub(base.saturating_mul(2))];
    let mut rects = [Rect::ZERO; 3];
    let mut x = 0u16;
    for (index, card_w) in widths.into_iter().enumerate() {
        rects[index] = Rect::new(x, 0, card_w, CARD_H);
        x = x.saturating_add(card_w).saturating_add(CARD_GAP_X);
    }
    CardLayout {
        rects,
        height: CARD_H,
        width: view_width,
    }
}

fn wrap_lines(text: &str, columns: usize) -> usize {
    let columns = columns.max(1);
    let mut lines = 1usize;
    let mut used = 0usize;
    for word in text.split(' ') {
        let word_w = width(word) as usize;
        if used == 0 {
            used = word_w;
        } else if used.saturating_add(1).saturating_add(word_w) <= columns {
            used = used.saturating_add(1).saturating_add(word_w);
        } else {
            lines = lines.saturating_add(1);
            used = word_w;
        }
    }
    lines
}

fn readout_lines(columns: u16) -> u16 {
    READOUT
        .iter()
        .map(|line| u16::try_from(wrap_lines(line, usize::from(columns))).unwrap_or(u16::MAX))
        .fold(0u16, u16::saturating_add)
}

fn content_height(view_width: u16) -> u16 {
    let cards = card_layout(view_width);
    cards
        .height
        .saturating_add(1)
        .saturating_add(readout_lines(cards.width))
}

fn thumb_len(viewport: u16, content: u16) -> u16 {
    if content <= viewport || viewport == 0 {
        return viewport;
    }
    let len = u32::from(viewport).saturating_mul(u32::from(viewport)) / u32::from(content);
    u16::try_from(len).unwrap_or(u16::MAX).max(1).min(viewport)
}

fn paint_scrollbar(ui: &mut Ui<'_>, bar: Rect, content_h: u16) {
    let thumb = thumb_len(bar.height, content_h);
    for row in 0..bar.height {
        let on_thumb = row < thumb;
        ui.paint_str(
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

fn paint_card_chrome(ui: &mut Ui<'_>, card: Rect, title: &str, meta: &str) {
    ui.fill(card, ink(WHITE, SURFACE));
    if card.height == 0 || card.width < 3 {
        return;
    }
    let text_x = card.x.saturating_add(2);
    let text_w = card.width.saturating_sub(4);
    ui.paint_str(
        Rect::new(text_x, card.y, text_w, 1),
        title,
        ink(SECONDARY, SURFACE),
    );
    if !meta.is_empty() {
        let meta_w = width(meta);
        if meta_w < text_w {
            ui.paint_str(
                Rect::new(
                    text_x.saturating_add(text_w.saturating_sub(meta_w)),
                    card.y,
                    meta_w,
                    1,
                ),
                meta,
                ink(FAINT, SURFACE),
            );
        }
    }
    if card.height > 1 && text_w > 0 {
        ui.paint_str(
            Rect::new(text_x, card.y.saturating_add(1), text_w, 1),
            &"─".repeat(usize::from(text_w)),
            ink(RULE, SURFACE),
        );
    }
}

fn card_inner(card: Rect) -> Rect {
    Rect {
        x: card.x.saturating_add(2),
        y: card.y.saturating_add(2),
        width: card.width.saturating_sub(4),
        height: card.height.saturating_sub(3),
    }
}

struct RowPaint {
    label: &'static str,
    meta: &'static str,
    selected: bool,
    disabled: bool,
}

fn meta_fg(row: &RowPaint) -> u32 {
    if row.disabled {
        DISABLED
    } else {
        match row.meta {
            "new" => SUCCESS,
            "modified" => WARNING,
            _ => MUTED,
        }
    }
}

fn paint_list_row(ui: &mut Ui<'_>, area: Rect, row: RowPaint, fade: Option<f32>) {
    if area.is_empty() {
        return;
    }
    let bg = if row.selected { ACCENT_BG } else { SURFACE };
    let mut label_fg = if row.disabled { DISABLED } else { WHITE };
    let mut trail_fg = meta_fg(&row);
    if let Some(keep) = fade.filter(|_| !row.selected) {
        label_fg = mix(label_fg, bg, keep);
        trail_fg = mix(trail_fg, bg, keep);
    }
    let mut style = ink(label_fg, bg);
    if row.selected {
        style = style.add_modifier(Modifier::BOLD);
    }
    ui.fill(area, style);
    ui.fill(Rect::new(area.x, area.y, 1, 1), ink(bg, bg));
    if row.selected {
        let mark = if row.meta.is_empty() { "›" } else { "✓" };
        let mut mark_style = ink(ACCENT, bg).add_modifier(Modifier::BOLD);
        if let Some(keep) = fade {
            mark_style = ink(mix(ACCENT, bg, keep), bg).add_modifier(Modifier::BOLD);
        }
        ui.paint_str(
            Rect::new(area.x.saturating_add(1), area.y, 1, 1),
            mark,
            mark_style,
        );
    }
    if area.width > 3 {
        ui.paint_str(
            Rect::new(
                area.x.saturating_add(3),
                area.y,
                area.width.saturating_sub(4),
                1,
            ),
            row.label,
            style,
        );
    }
    let meta_w = width(row.meta);
    if meta_w > 0 && area.width > meta_w.saturating_add(2) {
        let mut meta_style = ink(trail_fg, bg);
        if row.selected {
            meta_style = meta_style.add_modifier(Modifier::BOLD);
        }
        ui.paint_str(
            Rect::new(
                area.right().saturating_sub(1).saturating_sub(meta_w),
                area.y,
                meta_w,
                1,
            ),
            row.meta,
            meta_style,
        );
    }
}

fn paint_list_scrollbar(ui: &mut Ui<'_>, area: Rect, rows: usize, offset: usize) {
    let viewport = usize::from(area.height);
    if viewport == 0 || rows <= viewport {
        return;
    }
    let len = viewport.saturating_mul(viewport) / rows;
    let thumb = len.max(1).min(viewport);
    let max_offset = rows.saturating_sub(viewport);
    let top = offset
        .saturating_mul(viewport.saturating_sub(thumb))
        .checked_div(max_offset)
        .unwrap_or(0);
    let bottom = top.saturating_add(thumb);
    let x = area.right().saturating_sub(1);
    for row in 0..area.height {
        let r = usize::from(row);
        let on_thumb = r >= top && r < bottom;
        let style = if on_thumb {
            ink(THUMB, SURFACE)
        } else {
            ink(TRACK, SURFACE)
        };
        ui.paint_str(
            Rect::new(x, area.y.saturating_add(row), 1, 1),
            if on_thumb { "┃" } else { "┆" },
            style.remove_modifier(Modifier::BOLD),
        );
    }
}

fn fade_keep(index: usize, viewport: usize, total: usize) -> Option<f32> {
    if viewport < 4 || total <= viewport {
        return None;
    }
    let depth = usize::from(viewport >= 12) + 1;
    if index + 1 == viewport {
        Some(FADE_OUTER)
    } else if depth == 2 && index + 2 == viewport {
        Some(FADE_INNER)
    } else {
        None
    }
}

fn position_label(viewport: usize, total: usize, offset: usize) -> String {
    if viewport == 0 || total <= viewport {
        return String::new();
    }
    let start = offset.saturating_add(1);
    let end = offset.saturating_add(viewport).min(total);
    format!("{start}–{end} of {total}")
}

fn paint_language(ui: &mut Ui<'_>, card: Rect, chosen: &str, single: &ListState) {
    let inner = card_inner(card);
    let list_h = inner.height.saturating_sub(2);
    let scroll_offset = single.scroll().offset();
    let label = position_label(usize::from(list_h), LANGUAGES.len(), scroll_offset);
    paint_card_chrome(ui, card, "Language", &label);
    if inner.is_empty() {
        return;
    }
    ui.paint_str(
        Rect { height: 1, ..inner },
        &format!("Chosen: {chosen}"),
        ink(MUTED, SURFACE),
    );
    let list = Rect {
        y: inner.y.saturating_add(2),
        height: list_h,
        ..inner
    };
    ui.register_control(SINGLE, list, Focusability::Focusable);
    let viewport = usize::from(list.height);
    ui.register_scroll(
        SINGLE,
        list,
        termrock::Axes::V,
        termrock::Headroom {
            up: u16::try_from(scroll_offset).unwrap_or(u16::MAX),
            down: u16::try_from(
                LANGUAGES
                    .len()
                    .saturating_sub(viewport)
                    .saturating_sub(scroll_offset),
            )
            .unwrap_or(u16::MAX),
            left: 0,
            right: 0,
        },
    );
    ui.report_layout(
        SINGLE,
        termrock::LayoutFacts::new(viewport, LANGUAGES.len(), list.height, list.width),
    );
    if !ui.is_inert() {
        ui.publish_bindings(SINGLE, ui.state(SINGLE), single_list().table());
    }
    for (row_idx, (_, name)) in LANGUAGES
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(viewport)
        .enumerate()
    {
        let Ok(offset) = u16::try_from(row_idx) else {
            break;
        };
        let row_rect = Rect {
            y: list.y.saturating_add(offset),
            height: 1,
            ..list
        };
        ui.register_part(
            SINGLE,
            PartRef::item(Part::ROW, language_key(name)),
            row_rect,
        );
        paint_list_row(
            ui,
            row_rect,
            RowPaint {
                label: name,
                meta: "",
                selected: name == &chosen,
                disabled: false,
            },
            fade_keep(row_idx, viewport, LANGUAGES.len()),
        );
    }
    paint_list_scrollbar(ui, list, LANGUAGES.len(), scroll_offset);
}

fn paint_files(ui: &mut Ui<'_>, card: Rect, multi: &ListState) {
    let checked = multi.checked();
    let count = checked.len_in(FILES.len());
    let inner = card_inner(card);
    let list_h = inner.height.saturating_sub(2);
    let scroll_offset = multi.scroll().offset();
    paint_card_chrome(ui, card, "Files to include", &format!("{count} selected"));
    if inner.is_empty() {
        return;
    }
    ui.paint_str(
        Rect { height: 1, ..inner },
        "Space toggle · a all · Shift+↓ range",
        ink(MUTED, SURFACE),
    );
    let list = Rect {
        y: inner.y.saturating_add(2),
        height: list_h,
        ..inner
    };
    ui.register_control(MULTI, list, Focusability::Focusable);
    let viewport = usize::from(list.height);
    ui.register_scroll(
        MULTI,
        list,
        termrock::Axes::V,
        termrock::Headroom {
            up: u16::try_from(scroll_offset).unwrap_or(u16::MAX),
            down: u16::try_from(
                FILES
                    .len()
                    .saturating_sub(viewport)
                    .saturating_sub(scroll_offset),
            )
            .unwrap_or(u16::MAX),
            left: 0,
            right: 0,
        },
    );
    ui.report_layout(
        MULTI,
        termrock::LayoutFacts::new(viewport, FILES.len(), list.height, list.width),
    );
    if !ui.is_inert() {
        ui.publish_bindings(MULTI, ui.state(MULTI), multi_list().table());
    }
    for (row_idx, (_, file)) in FILES
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(viewport)
        .enumerate()
    {
        let Ok(offset) = u16::try_from(row_idx) else {
            break;
        };
        let row_rect = Rect {
            y: list.y.saturating_add(offset),
            height: 1,
            ..list
        };
        ui.register_part(MULTI, PartRef::item(Part::ROW, file_key(file)), row_rect);
        paint_list_row(
            ui,
            row_rect,
            RowPaint {
                label: file.label,
                meta: file.meta,
                selected: checked.contains(file_key(file)),
                disabled: file.disabled,
            },
            fade_keep(row_idx, viewport, FILES.len()),
        );
    }
    paint_list_scrollbar(ui, list, FILES.len(), scroll_offset);
}

fn paint_search(ui: &mut Ui<'_>, card: Rect, center_title: bool) {
    paint_card_chrome(ui, card, "Search results", "");
    let inner = card_inner(card);
    if inner.height < 2 || inner.width == 0 {
        return;
    }
    let title = "No results for “retry”";
    let title_w = width(title);
    let (text_x, text_w) = if center_title && title_w < inner.width {
        (
            inner
                .x
                .saturating_add(inner.width.saturating_sub(title_w) / 2),
            title_w,
        )
    } else {
        (inner.x, inner.width)
    };
    let y = inner.y.saturating_add(inner.height / 2);
    ui.paint_str(Rect::new(text_x, y, text_w, 1), title, ink(MUTED, SURFACE));
}

fn paint_wide_readout(ui: &mut Ui<'_>, area: Rect, cards_h: u16) {
    let y = area.y.saturating_add(cards_h).saturating_add(1);
    for (index, line) in WIDE_READOUT.iter().enumerate() {
        let Ok(offset) = u16::try_from(index) else {
            break;
        };
        let row = y.saturating_add(offset);
        if row >= area.bottom() {
            break;
        }
        ui.paint_str(
            Rect::new(area.x, row, area.width, 1),
            line,
            ink(FAINT, CANVAS),
        );
    }
}

fn paint_playground(
    ui: &mut Ui<'_>,
    area: Rect,
    chosen: &str,
    single: &ListState,
    multi: &ListState,
    wide: bool,
) {
    if area.is_empty() {
        return;
    }
    let content_h = content_height(area.width);
    let overflows = area.width > SCROLL_ROOM && content_h > area.height;
    let view = if overflows {
        Rect {
            width: area.width.saturating_sub(SCROLL_ROOM),
            ..area
        }
    } else {
        area
    };
    let cards = if wide {
        equal_card_layout(view.width)
    } else {
        card_layout(view.width)
    };
    ui.with_area(view, |ui| {
        let language = offset_card(view, cards.rects[0]);
        let files = offset_card(view, cards.rects[1]);
        let search = offset_card(view, cards.rects[2]);
        paint_language(ui, language, chosen, single);
        paint_files(ui, files, multi);
        paint_search(ui, search, wide);
    });
    if wide {
        paint_wide_readout(ui, view, cards.height);
    }
    if overflows {
        paint_scrollbar(
            ui,
            Rect::new(area.right().saturating_sub(1), area.y, 1, view.height),
            content_h,
        );
    }
}

fn offset_card(origin: Rect, local: Rect) -> Rect {
    Rect::new(
        origin.x.saturating_add(local.x),
        origin.y.saturating_add(local.y),
        local.width,
        local.height,
    )
}

#[derive(Debug, Default)]
pub struct ListsPage {
    single: ListState,
    multi: ListState,
    empty: ListState,
    chosen: Option<ItemKey>,
}

impl ListsPage {
    pub fn new() -> Self {
        let mut single = ListState::default();
        let mut multi = ListState::default();
        if let Some(file) = FILES.first() {
            multi.checked_mut().insert(file_key(file));
        }
        if let Some(file) = FILES.get(1) {
            multi.checked_mut().insert(file_key(file));
        }
        let chosen = LANGUAGES.first().map(language_key);
        single.choose(chosen);
        Self {
            single,
            multi,
            empty: ListState::default(),
            chosen,
        }
    }

    fn chosen_label(&self) -> &'static str {
        self.chosen
            .and_then(|key| {
                LANGUAGES
                    .iter()
                    .copied()
                    .find(|value| language_key(value) == key)
            })
            .unwrap_or("")
    }
}

impl Page for ListsPage {
    fn title(&self) -> &'static str {
        "Lists"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let mut response = Response::ignored();
        let one = single_list().update(cx, &mut self.single, LANGUAGES);
        if let Some(ListAction::Chose(key) | ListAction::Activated(key)) = one.action_ref() {
            self.chosen = Some(*key);
        }
        response |= one.erase();
        let many = multi_list().update(cx, &mut self.multi, FILES);
        if matches!(many.action_ref(), Some(ListAction::ToggledAll)) {
            for file in FILES.iter().filter(|file| file.disabled) {
                self.multi.checked_mut().remove(file_key(file));
            }
        }
        response |= many.erase();
        let empty = empty_list().update(cx, &mut self.empty, &[] as &[&str]);
        response |= empty.erase();
        response.into()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        let chosen = self.chosen_label();
        let wide = area.width >= EQUAL_COLUMNS_MIN_W;
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
            paint_playground(
                ui,
                Rect {
                    y: top,
                    height: body.bottom().saturating_sub(top),
                    ..body
                },
                chosen,
                &self.single,
                &self.multi,
                wide,
            );
        });
    }

    fn hints(&self, ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        if ui.state(MULTI).contains(StateFlags::FOCUSED) {
            &[
                ("↑ ↓", "Move"),
                ("Space", "Toggle"),
                ("a", "All / none"),
                ("Shift+↓", "Range"),
            ]
        } else {
            &[("↑ ↓", "Move"), ("Enter", "Choose"), ("g G", "Ends")]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ListsPage, Page, content_height, thumb_len};
    use termrock::{App, Color, Cx, Modifier, Response, Theme, Ui};
    use termrock_test_support::Harness;

    struct Host;

    impl App for Host {
        fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            ListsPage::new().draw(ui, ui.full());
        }
    }

    #[test]
    fn stacked_playground_matches_the_120_column_body() {
        let view_h = 31u16;
        let content = content_height(92);
        assert_eq!(thumb_len(view_h, content), 15, "content={content}");

        let h = Harness::new(Host, Theme::junie(), 94, 36);
        let text = h.text();
        assert!(text.contains("Playground"), "{text}");
        assert!(text.contains("the live widget"), "{text}");
        assert!(text.contains("Chosen: Rust"), "{text}");
        assert!(text.contains("1–13 of 20"), "{text}");
        assert!(text.contains("Files to include"), "{text}");
        assert!(text.contains("2 selected"), "{text}");
        assert!(
            text.contains("Space toggle · a all · Shift+↓ range"),
            "{text}"
        );
        assert!(text.contains("Haskell"), "{text}");
        assert!(text.contains("Zig"), "{text}");
        assert!(text.contains("src/api/auth.rs"), "{text}");
        assert!(text.contains("src/config.rs"), "{text}");
        assert!(!text.contains("Search results"), "{text}");
        assert!(!text.contains("Dart"), "{text}");

        let (x, y) = h.find("Playground").expect("tab");
        let tab = h.cell(x, y);
        assert_eq!(tab.bg, Color::Rgb(0x27, 0x27, 0x2a));
        assert!(tab.modifier.contains(Modifier::BOLD));
        assert_eq!(h.cell(x.saturating_sub(1), y + 1).symbol(), "━");

        let (tx, ty) = h.find("TypeScript").expect("second language");
        let (lx, ly) = (tx, ty.saturating_sub(1));
        assert_eq!(h.cell(lx.saturating_sub(2), ly).symbol(), "›");
        assert_eq!(h.cell(lx, ly).symbol(), "R");
        assert_eq!(h.cell(lx, ly).bg, Color::Rgb(0x17, 0x3f, 0x1c));
        assert_eq!(
            h.cell(lx, ly.saturating_add(11)).fg,
            Color::Rgb(207, 207, 207)
        );
        assert_eq!(
            h.cell(lx, ly.saturating_add(12)).fg,
            Color::Rgb(148, 148, 148)
        );

        let (fx, fy) = h.find("src/api/auth.rs").expect("checked file");
        assert_eq!(h.cell(fx.saturating_sub(2), fy).symbol(), "✓");
        assert!(h.cell(fx, fy).modifier.contains(Modifier::BOLD));
        let (mx, my) = h.find("generated").expect("disabled meta");
        assert_eq!(h.cell(mx, my).fg, Color::Rgb(0x4d, 0x4d, 0x4d));
        let (nx, ny) = h.find("new").expect("new meta");
        assert_eq!(h.cell(nx, ny).fg, Color::Rgb(0x3f, 0xb9, 0x50));
    }
}
