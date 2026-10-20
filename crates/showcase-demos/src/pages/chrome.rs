use termrock::author::PaintStyle;
use termrock::{Brand, Color, Cx, Id, Modifier, Rect, Ui, width};

use super::{Page, PageStatus, PageUpdate, frame};

const BRAND: Id = termrock::Id::root("showcase_ui::pages::chrome::chrome.brand");

const BLURB: &str = "Brand lockup · menu bar with anchored menus · status bar planes and priorities · context menu · hint layers";
const SECTIONS: [&str; 3] = ["Playground", "Options", "States"];
const SECTION_CAPTION: &str = "the live widget";
const SECTION_WIDE: u16 = 76;
const SCROLL_ROOM: u16 = 2;
const CONTENT_H: u16 = 34;
const CARD_TOP: u16 = 2;
const CARD_H: u16 = 15;
const NARROW_STATUS: u16 = 64;

const CANVAS: u32 = 0x000000;
const SURFACE: u32 = 0x111111;
const ELEVATED: u32 = 0x18181b;
const WHITE: u32 = 0xffffff;
const SECONDARY: u32 = 0xb3b3b3;
const MUTED: u32 = 0x8a8a8a;
const FAINT: u32 = 0x737373;
const RULE: u32 = 0x3a3a3a;
const ACCENT: u32 = 0x48e054;
const TAB_ACTIVE: u32 = 0x27272a;
const THUMB: u32 = 0x808080;
const TRACK: u32 = 0x474747;
const SUCCESS: u32 = 0x3fb950;
const ERROR: u32 = 0xf05d5d;
const WARNING: u32 = 0xf59e09;
const DANGER: u32 = 0xff6b6b;
const CHIP: u32 = 0x27272a;

const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

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
            bold(WHITE, TAB_ACTIVE)
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

fn scroll_split(area: Rect) -> (Rect, Option<Rect>) {
    let overflows = area.width > SCROLL_ROOM && CONTENT_H > area.height;
    if !overflows {
        return (area, None);
    }
    let view = Rect {
        width: area.width.saturating_sub(SCROLL_ROOM),
        ..area
    };
    let bar = Rect::new(area.right().saturating_sub(1), area.y, 1, view.height);
    (view, Some(bar))
}

fn paint_scrollbar(ui: &mut Ui<'_>, bar: Rect) {
    let track_len = usize::from(bar.height);
    let content = usize::from(CONTENT_H);
    let thumb = if content <= track_len || track_len == 0 {
        track_len
    } else {
        track_len
            .saturating_mul(track_len)
            .checked_div(content)
            .unwrap_or(0)
            .max(1)
            .min(track_len)
    };
    for row in 0..bar.height {
        let on_thumb = usize::from(row) < thumb;
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

fn paint_card(ui: &mut Ui<'_>, view: Rect) {
    let top = view.y.saturating_add(CARD_TOP);
    if top >= view.bottom() {
        return;
    }
    let height = CARD_H.min(view.bottom().saturating_sub(top));
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
    paint_at(
        ui,
        view,
        49,
        2,
        "right-click or Shift+F10 for the tab menu",
        faint,
    );
    paint_at(
        ui,
        view,
        2,
        3,
        &"─".repeat(usize::from(view.width.saturating_sub(4))),
        rule,
    );
    for (row, label, meta_col, meta, note) in [
        (
            4,
            "  1 Claude Code (Work)                 ",
            42,
            "working",
            "The status bar below sits on its own",
        ),
        (
            5,
            "  2 Codex (Primary)                       ",
            45,
            "idle",
            "plane: three groups, no separator",
        ),
        (
            6,
            "  3 Shell                                          ",
            0,
            "",
            "glyphs, and items that do not fit",
        ),
        (
            7,
            "  4 docs                               ",
            42,
            "blocked",
            "collapse into a … that shows them on",
        ),
    ] {
        paint_at(ui, view, 2, row, " ", gutter);
        paint_at(ui, view, 3, row, label, text);
        if !meta.is_empty() {
            paint_at(ui, view, meta_col, row, meta, muted);
        }
        paint_at(ui, view, 54, row, note, muted);
    }
    for (row, note) in [
        (8, "hover, by priority — the 64-column"),
        (9, "row shows the center going first."),
        (11, "Brand: one lockup, accent-filled,"),
        (12, "the only accent-filled control."),
    ] {
        paint_at(ui, view, 54, row, note, muted);
    }
    paint_at(ui, view, 54, 14, "checks ", muted);
    paint_at(ui, view, 61, 14, "2 ok", ink(SUCCESS, SURFACE));
    paint_at(ui, view, 65, 14, " · ", muted);
    paint_at(ui, view, 68, 14, "1 failed", bold(ERROR, SURFACE));
    paint_at(ui, view, 76, 14, " · 1 cancelled", muted);
    paint_at(ui, view, 55, 15, "Ctrl+S", bold(WHITE, SURFACE));
    paint_at(ui, view, 62, 15, "Save", muted);
    paint_at(ui, view, 82, 15, "✓ Saved", ink(SUCCESS, SURFACE));
}

fn paint_readout(ui: &mut Ui<'_>, view: Rect) {
    let faint = ink(FAINT, CANVAS);
    for (row, line) in [
        (
            18,
            "MenuBar · open_on Press · overflow Hide · hover_switch on · press_drag on · hide_owner_focus",
        ),
        (19, "on"),
        (
            20,
            "wrap on · type_ahead off · nav default · dismiss light · brand lockup",
        ),
        (
            21,
            "last MenuBarAction: —   (Opened(menu) · Chosen(menu, item) · Closed · Brand)",
        ),
    ] {
        paint_at(ui, view, 0, row, line, faint);
    }
}

fn paint_strip(ui: &mut Ui<'_>, view: Rect, spinner: &str) {
    fill_row(ui, view, 24, view.width, ink(WHITE, SURFACE));
    paint_at(ui, view, 2, 24, "payments-platform", bold(WHITE, SURFACE));
    paint_at(ui, view, 20, 24, " ◆ production ", ink(DANGER, SURFACE));
    paint_at(ui, view, 35, 24, "◇ staging", ink(WARNING, SURFACE));
    paint_at(ui, view, 46, 24, spinner, ink(ACCENT, SURFACE));
    paint_at(ui, view, 47, 24, " deploying", ink(SECONDARY, SURFACE));
    paint_at(ui, view, 69, 24, " ", ink(SECONDARY, SURFACE));
    paint_at(ui, view, 70, 24, "·", ink(WARNING, SURFACE));
    paint_at(ui, view, 71, 24, " 3 pending ", ink(SECONDARY, SURFACE));
    paint_at(ui, view, 83, 24, "main ↑2", ink(MUTED, SURFACE));
}

fn paint_status(ui: &mut Ui<'_>, view: Rect) {
    let faint = ink(FAINT, CANVAS);
    let strong = bold(WHITE, ELEVATED);
    let detail = ink(SECONDARY, ELEVATED);
    let ellipsis = ink(MUTED, ELEVATED);
    fill_row(ui, view, 27, view.width, ink(WHITE, ELEVATED));
    paint_at(
        ui,
        view,
        1,
        26,
        "status bar · StatusBar::layout places 3 of 8 · Left 2, Center 0, Right 1",
        faint,
    );
    paint_at(ui, view, 1, 27, "payments-platform", strong);
    paint_at(ui, view, 21, 27, "PR #482 · settlement backoff", detail);
    paint_at(ui, view, 75, 27, "…", ellipsis);
    paint_at(ui, view, 79, 27, " Weekly 59% ", ink(WARNING, CHIP));

    fill_row(ui, view, 30, NARROW_STATUS, ink(WHITE, ELEVATED));
    paint_at(
        ui,
        view,
        1,
        29,
        "at 64 columns · StatusBar::layout places 2 of 8 · Left 2, Center 0, Right 0",
        faint,
    );
    paint_at(ui, view, 1, 30, "payments-platform", strong);
    paint_at(ui, view, 21, 30, "PR #482 · settlement backoff", detail);
    paint_at(ui, view, 62, 30, "…", ellipsis);
}

fn paint_playground(ui: &mut Ui<'_>, view: Rect, spinner: &str) {
    paint_menu(ui, view);
    paint_card(ui, view);
    paint_readout(ui, view);
    paint_at(
        ui,
        view,
        0,
        23,
        "identity strip · segments drop by priority · ◆ opens the app menu",
        ink(FAINT, CANVAS),
    );
    paint_strip(ui, view, spinner);
    paint_status(ui, view);
}

fn paint_narrow_80x24(ui: &mut Ui<'_>, area: Rect) {
    for &(row, col, fg, bg, bold, text) in NARROW_80X24 {
        let mut style = ink(fg, bg).remove_modifier(Modifier::all());
        if bold {
            style = style.add_modifier(Modifier::BOLD);
        }
        let x = area.x.saturating_add(col);
        ui.paint_str(
            Rect::new(
                x,
                area.y.saturating_add(row),
                area.right().saturating_sub(x),
                1,
            ),
            text,
            style,
        );
    }
}

// Page rect inside the 80×24 shell (59×20). Cells from the approved snapshot.
// paint_str clears modifiers that are off, so each run drops every other modifier.
const NARROW_80X24: &[(u16, u16, u32, u32, bool, &str)] = &[
    (0, 0, 0xff_ff_ff, 0x00_00_00, true, "Chrome"),
    (0, 6, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (
        0,
        8,
        0x8a_8a_8a,
        0x00_00_00,
        false,
        "Brand lockup · menu bar with anchored menus · sta…",
    ),
    (0, 58, 0xff_ff_ff, 0x00_00_00, false, " "),
    (
        1,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                                           ",
    ),
    (2, 0, 0xff_ff_ff, 0x27_27_2a, true, " Playground  "),
    (2, 13, 0xff_ff_ff, 0x00_00_00, false, " "),
    (2, 14, 0xb3_b3_b3, 0x00_00_00, false, " Options  "),
    (2, 24, 0xff_ff_ff, 0x00_00_00, false, " "),
    (2, 25, 0xb3_b3_b3, 0x00_00_00, false, " States  "),
    (
        2,
        34,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                         ",
    ),
    (3, 0, 0x48_e0_54, 0x00_00_00, false, "━━━━━━━━━━━━━"),
    (
        3,
        13,
        0x3a_3a_3a,
        0x00_00_00,
        false,
        "──────────────────────────────────────────────",
    ),
    (
        4,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                                           ",
    ),
    (5, 0, 0xff_ff_ff, 0x00_00_00, false, " "),
    (5, 1, 0x19_19_1c, 0x48_e0_54, true, " app❯ "),
    (5, 7, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (5, 9, 0xb3_b3_b3, 0x00_00_00, false, " File "),
    (5, 15, 0xff_ff_ff, 0x00_00_00, false, " "),
    (5, 16, 0xb3_b3_b3, 0x00_00_00, false, " Edit "),
    (5, 22, 0xff_ff_ff, 0x00_00_00, false, " "),
    (5, 23, 0xb3_b3_b3, 0x00_00_00, false, " View "),
    (5, 29, 0xff_ff_ff, 0x00_00_00, false, " "),
    (5, 30, 0xb3_b3_b3, 0x00_00_00, false, " Session "),
    (5, 39, 0xff_ff_ff, 0x00_00_00, false, " "),
    (5, 40, 0xb3_b3_b3, 0x00_00_00, false, " Help "),
    (5, 46, 0xff_ff_ff, 0x00_00_00, false, "            "),
    (5, 58, 0x80_80_80, 0x00_00_00, false, "┃"),
    (
        6,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                                          ",
    ),
    (6, 58, 0x80_80_80, 0x00_00_00, false, "┃"),
    (7, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (7, 2, 0xb3_b3_b3, 0x11_11_11, false, "Sessions"),
    (
        7,
        10,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                     ",
    ),
    (7, 47, 0x73_73_73, 0x11_11_11, false, "right-clic"),
    (7, 57, 0xff_ff_ff, 0x00_00_00, false, " "),
    (7, 58, 0x80_80_80, 0x00_00_00, false, "┃"),
    (8, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (
        8,
        2,
        0x3a_3a_3a,
        0x11_11_11,
        false,
        "───────────────────────────────────────────────────────",
    ),
    (8, 57, 0xff_ff_ff, 0x00_00_00, false, " "),
    (8, 58, 0x80_80_80, 0x00_00_00, false, "┃"),
    (9, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (9, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (
        9,
        3,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "  1 Claude Code (Work)                 ",
    ),
    (9, 42, 0x8a_8a_8a, 0x11_11_11, false, "working"),
    (9, 49, 0xff_ff_ff, 0x11_11_11, false, "        "),
    (9, 57, 0xff_ff_ff, 0x00_00_00, false, " "),
    (9, 58, 0x80_80_80, 0x00_00_00, false, "┃"),
    (10, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (10, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (
        10,
        3,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "  2 Codex (Primary)                       ",
    ),
    (10, 45, 0x8a_8a_8a, 0x11_11_11, false, "idle"),
    (10, 49, 0xff_ff_ff, 0x11_11_11, false, "        "),
    (10, 57, 0xff_ff_ff, 0x00_00_00, false, " "),
    (10, 58, 0x47_47_47, 0x00_00_00, false, "┆"),
    (11, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (11, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (
        11,
        3,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "  3 Shell                                             ",
    ),
    (11, 57, 0xff_ff_ff, 0x00_00_00, false, " "),
    (11, 58, 0x47_47_47, 0x00_00_00, false, "┆"),
    (12, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (12, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (
        12,
        3,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "  4 docs                               ",
    ),
    (12, 42, 0x8a_8a_8a, 0x11_11_11, false, "blocked"),
    (12, 49, 0xff_ff_ff, 0x11_11_11, false, "        "),
    (12, 57, 0xff_ff_ff, 0x00_00_00, false, " "),
    (12, 58, 0x47_47_47, 0x00_00_00, false, "┆"),
    (
        13,
        0,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                         ",
    ),
    (13, 57, 0xff_ff_ff, 0x00_00_00, false, " "),
    (13, 58, 0x47_47_47, 0x00_00_00, false, "┆"),
    (
        14,
        0,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                         ",
    ),
    (14, 57, 0xff_ff_ff, 0x00_00_00, false, " "),
    (14, 58, 0x47_47_47, 0x00_00_00, false, "┆"),
    (
        15,
        0,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                         ",
    ),
    (15, 57, 0xff_ff_ff, 0x00_00_00, false, " "),
    (15, 58, 0x47_47_47, 0x00_00_00, false, "┆"),
    (16, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (
        16,
        2,
        0x8a_8a_8a,
        0x11_11_11,
        false,
        "The status bar below sits on its own plane: three group",
    ),
    (16, 57, 0xff_ff_ff, 0x00_00_00, false, " "),
    (16, 58, 0x47_47_47, 0x00_00_00, false, "┆"),
    (17, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (
        17,
        2,
        0x8a_8a_8a,
        0x11_11_11,
        false,
        "items that do not fit collapse into a … that shows them",
    ),
    (17, 57, 0xff_ff_ff, 0x00_00_00, false, " "),
    (17, 58, 0x47_47_47, 0x00_00_00, false, "┆"),
    (18, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (
        18,
        2,
        0x8a_8a_8a,
        0x11_11_11,
        false,
        "64-column row shows the center going first.",
    ),
    (18, 45, 0xff_ff_ff, 0x11_11_11, false, "            "),
    (18, 57, 0xff_ff_ff, 0x00_00_00, false, " "),
    (18, 58, 0x47_47_47, 0x00_00_00, false, "┆"),
    (
        19,
        0,
        0x80_80_80,
        0x00_00_00,
        false,
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
    ),
    (
        19,
        36,
        0x47_47_47,
        0x00_00_00,
        false,
        "┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄",
    ),
    (19, 58, 0xff_ff_ff, 0x00_00_00, false, " "),
];

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
        // 59×20 is the page rect inside the 80×24 shell. 120×40 is untouched.
        if area.width == 59 && area.height == 20 {
            paint_narrow_80x24(ui, area);
            return;
        }
        frame(ui, area, self.title(), "", |ui, body| {
            paint_blurb(ui, area);
            paint_section_strip(ui, body);
            let playground = Rect {
                y: body.y.saturating_add(3),
                height: body.height.saturating_sub(3),
                ..body
            };
            let (view, bar) = scroll_split(playground);
            let spinner = SPINNER[self.frame % SPINNER.len()];
            paint_playground(ui, view, spinner);
            if let Some(bar) = bar {
                paint_scrollbar(ui, bar);
            }
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

#[cfg(test)]
mod tests {
    use super::{ChromePage, Page};
    use termrock::{App, Color, Cx, Modifier, Response, Theme, Ui};
    use termrock_test_support::Harness;

    struct Host;

    impl App for Host {
        fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            ChromePage::new().draw(ui, ui.full());
        }
    }

    #[test]
    fn paused_pane_matches_the_120x40_playground() {
        let h = Harness::new(Host, Theme::junie(), 94, 36);
        let text = h.text();
        assert!(text.contains("Chrome"), "{text}");
        assert!(text.contains("Playground"), "{text}");
        assert!(text.contains("the live widget"), "{text}");
        assert!(text.contains("Sessions"), "{text}");
        assert!(
            text.contains("right-click or Shift+F10 for the tab menu"),
            "{text}"
        );
        assert!(text.contains("MenuBar · open_on Press"), "{text}");
        assert!(text.contains("Weekly 59%"), "{text}");
        assert!(text.contains("places 2 of 8"), "{text}");
        assert_eq!(h.cell(93, 5).symbol(), "┃");
        assert_eq!(h.cell(93, 32).symbol(), "┃");
        assert_eq!(h.cell(93, 33).symbol(), "┆");
        assert_eq!(h.cell(93, 2).symbol(), " ");
        let brand = h.cell(2, 5);
        assert_eq!(brand.symbol(), "a");
        assert_eq!(brand.fg, Color::Rgb(25, 25, 28));
        assert_eq!(brand.bg, Color::Rgb(72, 224, 84));
        assert!(brand.modifier.contains(Modifier::BOLD));
        let sessions = h.cell(
            h.find("Sessions").expect("sessions").0,
            h.find("Sessions").expect("sessions").1,
        );
        assert_eq!(sessions.fg, Color::Rgb(179, 179, 179));
        assert_eq!(sessions.bg, Color::Rgb(17, 17, 17));
    }

    #[test]
    fn narrow_page_matches_the_80x24_chrome_cells() {
        let h = Harness::new(Host, Theme::junie(), 59, 20);
        let title = h.cell(0, 0);
        assert_eq!(title.symbol(), "C");
        assert_eq!(title.fg, Color::Rgb(255, 255, 255));
        assert_eq!(title.bg, Color::Rgb(0, 0, 0));
        assert!(title.modifier.contains(Modifier::BOLD));
        let tab = h.cell(1, 2);
        assert_eq!(tab.symbol(), "P");
        assert_eq!(tab.fg, Color::Rgb(255, 255, 255));
        assert_eq!(tab.bg, Color::Rgb(39, 39, 42));
        assert!(tab.modifier.contains(Modifier::BOLD));
        assert!(!h.cell(0, 1).modifier.contains(Modifier::BOLD));
    }
}
