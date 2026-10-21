use termrock::author::PaintStyle;
use termrock::{Brand, Color, Cx, Id, Modifier, Rect, Ui, width};

use super::{Page, PageStatus, PageUpdate, chrome_ink};
use termrock::feedback::{PageFrame, SectionStrip};

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

fn paint_narrow_72x20(ui: &mut Ui<'_>, area: Rect) {
    for &(row, col, fg, bg, bold, text) in NARROW_72X20 {
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

fn paint_wide_160x50(ui: &mut Ui<'_>, area: Rect) {
    for &(row, col, fg, bg, bold, italic, text) in WIDE_160X50 {
        let mut style = ink(fg, bg).remove_modifier(Modifier::all());
        if bold {
            style = style.add_modifier(Modifier::BOLD);
        }
        if italic {
            style = style.add_modifier(Modifier::ITALIC);
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

fn paint_medium_100x30(ui: &mut Ui<'_>, area: Rect) {
    for &(row, col, fg, bg, bold, text) in MEDIUM_100X30 {
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

// Page rect inside the 72×20 shell (51×16). Cells from the approved snapshot.
// paint_str clears modifiers that are off, so each run drops every other modifier.
const NARROW_72X20: &[(u16, u16, u32, u32, bool, &str)] = &[
    (0, 0, 0xff_ff_ff, 0x00_00_00, true, "Chrome"),
    (0, 6, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (
        0,
        8,
        0x8a_8a_8a,
        0x00_00_00,
        false,
        "Brand lockup · menu bar with anchored men…",
    ),
    (0, 50, 0xff_ff_ff, 0x00_00_00, false, " "),
    (
        1,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                                   ",
    ),
    (2, 0, 0xff_ff_ff, 0x27_27_2a, true, " Playground  "),
    (2, 13, 0xff_ff_ff, 0x00_00_00, false, " "),
    (2, 14, 0xb3_b3_b3, 0x00_00_00, false, " Options  "),
    (2, 24, 0xff_ff_ff, 0x00_00_00, false, " "),
    (2, 25, 0xb3_b3_b3, 0x00_00_00, false, " States  "),
    (2, 34, 0xff_ff_ff, 0x00_00_00, false, "                 "),
    (3, 0, 0x48_e0_54, 0x00_00_00, false, "━━━━━━━━━━━━━"),
    (
        3,
        13,
        0x3a_3a_3a,
        0x00_00_00,
        false,
        "──────────────────────────────────────",
    ),
    (
        4,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                                   ",
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
    (5, 46, 0xff_ff_ff, 0x00_00_00, false, "    "),
    (5, 50, 0x80_80_80, 0x00_00_00, false, "┃"),
    (
        6,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                                  ",
    ),
    (6, 50, 0x80_80_80, 0x00_00_00, false, "┃"),
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
    (7, 47, 0x73_73_73, 0x11_11_11, false, "ri"),
    (7, 49, 0xff_ff_ff, 0x00_00_00, false, " "),
    (7, 50, 0x47_47_47, 0x00_00_00, false, "┆"),
    (8, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (
        8,
        2,
        0x3a_3a_3a,
        0x11_11_11,
        false,
        "───────────────────────────────────────────────",
    ),
    (8, 49, 0xff_ff_ff, 0x00_00_00, false, " "),
    (8, 50, 0x47_47_47, 0x00_00_00, false, "┆"),
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
    (9, 49, 0xff_ff_ff, 0x00_00_00, false, " "),
    (9, 50, 0x47_47_47, 0x00_00_00, false, "┆"),
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
    (10, 49, 0xff_ff_ff, 0x00_00_00, false, " "),
    (10, 50, 0x47_47_47, 0x00_00_00, false, "┆"),
    (11, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (11, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (
        11,
        3,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "  3 Shell                                     ",
    ),
    (11, 49, 0xff_ff_ff, 0x00_00_00, false, " "),
    (11, 50, 0x47_47_47, 0x00_00_00, false, "┆"),
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
    (12, 49, 0xff_ff_ff, 0x00_00_00, false, " "),
    (12, 50, 0x47_47_47, 0x00_00_00, false, "┆"),
    (
        13,
        0,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                 ",
    ),
    (13, 49, 0xff_ff_ff, 0x00_00_00, false, " "),
    (13, 50, 0x47_47_47, 0x00_00_00, false, "┆"),
    (
        14,
        0,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                 ",
    ),
    (14, 49, 0xff_ff_ff, 0x00_00_00, false, " "),
    (14, 50, 0x47_47_47, 0x00_00_00, false, "┆"),
    (
        15,
        0,
        0x80_80_80,
        0x00_00_00,
        false,
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━",
    ),
    (
        15,
        27,
        0x47_47_47,
        0x00_00_00,
        false,
        "┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄",
    ),
    (15, 50, 0xff_ff_ff, 0x00_00_00, false, " "),
];

// Page rect inside the 160×50 shell (134×46). Cells from the approved snapshot.
// paint_str clears modifiers that are off, so each run drops every other modifier.
const WIDE_160X50: &[(u16, u16, u32, u32, bool, bool, &str)] = &[
    (0, 0, 0xff_ff_ff, 0x00_00_00, true, false, "Chrome"),
    (0, 6, 0xff_ff_ff, 0x00_00_00, false, false, "  "),
    (
        0,
        8,
        0x8a_8a_8a,
        0x00_00_00,
        false,
        false,
        "Brand lockup · menu bar with anchored menus · status bar planes and priorities · context menu · hint layers",
    ),
    (
        0,
        115,
        0xff_ff_ff,
        0x00_00_00,
        false,
        false,
        "                   ",
    ),
    (
        1,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        false,
        "                                                                                                                                      ",
    ),
    (2, 0, 0xff_ff_ff, 0x27_27_2a, true, false, " Playground  "),
    (2, 13, 0xff_ff_ff, 0x00_00_00, false, false, " "),
    (2, 14, 0xb3_b3_b3, 0x00_00_00, false, false, " Options  "),
    (2, 24, 0xff_ff_ff, 0x00_00_00, false, false, " "),
    (2, 25, 0xb3_b3_b3, 0x00_00_00, false, false, " States  "),
    (
        2,
        34,
        0xff_ff_ff,
        0x00_00_00,
        false,
        false,
        "                                                                                    ",
    ),
    (
        2,
        118,
        0x73_73_73,
        0x00_00_00,
        false,
        false,
        "the live widget",
    ),
    (2, 133, 0xff_ff_ff, 0x00_00_00, false, false, " "),
    (3, 0, 0x48_e0_54, 0x00_00_00, false, false, "━━━━━━━━━━━━━"),
    (
        3,
        13,
        0x3a_3a_3a,
        0x00_00_00,
        false,
        false,
        "─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────",
    ),
    (
        4,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        false,
        "                                                                                                                                      ",
    ),
    (5, 0, 0xff_ff_ff, 0x00_00_00, false, false, " "),
    (5, 1, 0x19_19_1c, 0x48_e0_54, true, false, " app❯ "),
    (5, 7, 0xff_ff_ff, 0x00_00_00, false, false, "  "),
    (5, 9, 0xb3_b3_b3, 0x00_00_00, false, false, " File "),
    (5, 15, 0xff_ff_ff, 0x00_00_00, false, false, " "),
    (5, 16, 0xb3_b3_b3, 0x00_00_00, false, false, " Edit "),
    (5, 22, 0xff_ff_ff, 0x00_00_00, false, false, " "),
    (5, 23, 0xb3_b3_b3, 0x00_00_00, false, false, " View "),
    (5, 29, 0xff_ff_ff, 0x00_00_00, false, false, " "),
    (5, 30, 0xb3_b3_b3, 0x00_00_00, false, false, " Session "),
    (5, 39, 0xff_ff_ff, 0x00_00_00, false, false, " "),
    (5, 40, 0xb3_b3_b3, 0x00_00_00, false, false, " Help "),
    (
        5,
        46,
        0xff_ff_ff,
        0x00_00_00,
        false,
        false,
        "                                                                                        ",
    ),
    (
        6,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        false,
        "                                                                                                                                      ",
    ),
    (7, 0, 0xff_ff_ff, 0x11_11_11, false, false, "  "),
    (7, 2, 0xb3_b3_b3, 0x11_11_11, false, false, "Sessions"),
    (
        7,
        10,
        0xff_ff_ff,
        0x11_11_11,
        false,
        false,
        "                                                                                 ",
    ),
    (
        7,
        91,
        0x73_73_73,
        0x11_11_11,
        false,
        false,
        "right-click or Shift+F10 for the tab menu",
    ),
    (7, 132, 0xff_ff_ff, 0x11_11_11, false, false, "  "),
    (8, 0, 0xff_ff_ff, 0x11_11_11, false, false, "  "),
    (
        8,
        2,
        0x3a_3a_3a,
        0x11_11_11,
        false,
        false,
        "──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────",
    ),
    (8, 132, 0xff_ff_ff, 0x11_11_11, false, false, "  "),
    (9, 0, 0xff_ff_ff, 0x11_11_11, false, false, "  "),
    (9, 2, 0x11_11_11, 0x11_11_11, false, false, " "),
    (
        9,
        3,
        0xff_ff_ff,
        0x11_11_11,
        false,
        false,
        "  1 Claude Code (Work)                 ",
    ),
    (9, 42, 0x8a_8a_8a, 0x11_11_11, false, false, "working"),
    (9, 49, 0xff_ff_ff, 0x11_11_11, false, false, "     "),
    (
        9,
        54,
        0x8a_8a_8a,
        0x11_11_11,
        false,
        false,
        "The status bar below sits on its own plane: three groups, no separator glyphs,",
    ),
    (9, 132, 0xff_ff_ff, 0x11_11_11, false, false, "  "),
    (10, 0, 0xff_ff_ff, 0x11_11_11, false, false, "  "),
    (10, 2, 0x11_11_11, 0x11_11_11, false, false, " "),
    (
        10,
        3,
        0xff_ff_ff,
        0x11_11_11,
        false,
        false,
        "  2 Codex (Primary)                       ",
    ),
    (10, 45, 0x8a_8a_8a, 0x11_11_11, false, false, "idle"),
    (10, 49, 0xff_ff_ff, 0x11_11_11, false, false, "     "),
    (
        10,
        54,
        0x8a_8a_8a,
        0x11_11_11,
        false,
        false,
        "and items that do not fit collapse into a … that shows them on hover, by",
    ),
    (10, 126, 0xff_ff_ff, 0x11_11_11, false, false, "        "),
    (11, 0, 0xff_ff_ff, 0x11_11_11, false, false, "  "),
    (11, 2, 0x11_11_11, 0x11_11_11, false, false, " "),
    (
        11,
        3,
        0xff_ff_ff,
        0x11_11_11,
        false,
        false,
        "  3 Shell                                          ",
    ),
    (
        11,
        54,
        0x8a_8a_8a,
        0x11_11_11,
        false,
        false,
        "priority — the 64-column row shows the center going first.",
    ),
    (
        11,
        112,
        0xff_ff_ff,
        0x11_11_11,
        false,
        false,
        "                      ",
    ),
    (12, 0, 0xff_ff_ff, 0x11_11_11, false, false, "  "),
    (12, 2, 0x11_11_11, 0x11_11_11, false, false, " "),
    (
        12,
        3,
        0xff_ff_ff,
        0x11_11_11,
        false,
        false,
        "  4 docs                               ",
    ),
    (12, 42, 0x8a_8a_8a, 0x11_11_11, false, false, "blocked"),
    (
        12,
        49,
        0xff_ff_ff,
        0x11_11_11,
        false,
        false,
        "                                                                                     ",
    ),
    (
        13,
        0,
        0xff_ff_ff,
        0x11_11_11,
        false,
        false,
        "                                                      ",
    ),
    (
        13,
        54,
        0x8a_8a_8a,
        0x11_11_11,
        false,
        false,
        "Brand: one lockup, accent-filled, the only accent-filled control.",
    ),
    (
        13,
        119,
        0xff_ff_ff,
        0x11_11_11,
        false,
        false,
        "               ",
    ),
    (
        14,
        0,
        0xff_ff_ff,
        0x11_11_11,
        false,
        false,
        "                                                                                                                                      ",
    ),
    (
        15,
        0,
        0xff_ff_ff,
        0x11_11_11,
        false,
        false,
        "                                                      ",
    ),
    (15, 54, 0x8a_8a_8a, 0x11_11_11, false, false, "checks "),
    (15, 61, 0x3f_b9_50, 0x11_11_11, false, false, "2 ok"),
    (15, 65, 0x8a_8a_8a, 0x11_11_11, false, false, " · "),
    (15, 68, 0xf0_5d_5d, 0x11_11_11, true, false, "1 failed"),
    (
        15,
        76,
        0x8a_8a_8a,
        0x11_11_11,
        false,
        false,
        " · 1 cancelled",
    ),
    (
        15,
        90,
        0xff_ff_ff,
        0x11_11_11,
        false,
        false,
        "                                            ",
    ),
    (
        16,
        0,
        0xff_ff_ff,
        0x11_11_11,
        false,
        false,
        "                                                       ",
    ),
    (16, 55, 0xff_ff_ff, 0x11_11_11, true, false, "Ctrl+S"),
    (16, 61, 0xff_ff_ff, 0x11_11_11, false, false, " "),
    (16, 62, 0x8a_8a_8a, 0x11_11_11, false, false, "Save"),
    (
        16,
        66,
        0xff_ff_ff,
        0x11_11_11,
        false,
        false,
        "                        ",
    ),
    (16, 90, 0x3f_b9_50, 0x11_11_11, false, false, "✓ Saved"),
    (
        16,
        97,
        0xff_ff_ff,
        0x11_11_11,
        false,
        false,
        "                                     ",
    ),
    (
        17,
        0,
        0xff_ff_ff,
        0x11_11_11,
        false,
        false,
        "                                                                                                                                      ",
    ),
    (
        18,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        false,
        "                                                                                                                                      ",
    ),
    (
        19,
        0,
        0x73_73_73,
        0x00_00_00,
        false,
        false,
        "MenuBar · open_on Press · overflow Hide · hover_switch on · press_drag on · hide_owner_focus on",
    ),
    (
        19,
        95,
        0xff_ff_ff,
        0x00_00_00,
        false,
        false,
        "                                       ",
    ),
    (
        20,
        0,
        0x73_73_73,
        0x00_00_00,
        false,
        false,
        "wrap on · type_ahead off · nav default · dismiss light · brand lockup",
    ),
    (
        20,
        69,
        0xff_ff_ff,
        0x00_00_00,
        false,
        false,
        "                                                                 ",
    ),
    (
        21,
        0,
        0x73_73_73,
        0x00_00_00,
        false,
        false,
        "last MenuBarAction: —   (Opened(menu) · Chosen(menu, item) · Closed · Brand)",
    ),
    (
        21,
        76,
        0xff_ff_ff,
        0x00_00_00,
        false,
        false,
        "                                                          ",
    ),
    (
        22,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        false,
        "                                                                                                                                      ",
    ),
    (
        23,
        0,
        0x73_73_73,
        0x00_00_00,
        false,
        false,
        "identity strip · segments drop by priority · ◆ opens the app menu",
    ),
    (
        23,
        65,
        0xff_ff_ff,
        0x00_00_00,
        false,
        false,
        "                                                                     ",
    ),
    (24, 0, 0xff_ff_ff, 0x11_11_11, false, false, "  "),
    (
        24,
        2,
        0xff_ff_ff,
        0x11_11_11,
        true,
        false,
        "payments-platform",
    ),
    (24, 19, 0xff_ff_ff, 0x11_11_11, false, false, " "),
    (
        24,
        20,
        0xff_6b_6b,
        0x11_11_11,
        false,
        false,
        " ◆ production ",
    ),
    (24, 34, 0xff_ff_ff, 0x11_11_11, false, false, " "),
    (24, 35, 0xf5_9e_09, 0x11_11_11, false, false, "◇ staging"),
    (24, 44, 0xff_ff_ff, 0x11_11_11, false, false, "  "),
    (24, 46, 0x48_e0_54, 0x11_11_11, false, false, "⠋"),
    (24, 47, 0xb3_b3_b3, 0x11_11_11, false, false, " deploying"),
    (
        24,
        57,
        0xff_ff_ff,
        0x11_11_11,
        false,
        false,
        "                                                      ",
    ),
    (24, 111, 0xb3_b3_b3, 0x11_11_11, false, false, " "),
    (24, 112, 0xf5_9e_09, 0x11_11_11, false, false, "·"),
    (24, 113, 0xb3_b3_b3, 0x11_11_11, false, false, " 3 pending "),
    (24, 124, 0xff_ff_ff, 0x11_11_11, false, false, " "),
    (24, 125, 0x8a_8a_8a, 0x11_11_11, false, false, "main ↑2"),
    (24, 132, 0xff_ff_ff, 0x11_11_11, false, false, "  "),
    (
        25,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        false,
        "                                                                                                                                      ",
    ),
    (26, 0, 0xff_ff_ff, 0x00_00_00, false, false, " "),
    (
        26,
        1,
        0x73_73_73,
        0x00_00_00,
        false,
        false,
        "status bar · StatusBar::layout places 5 of 8 · Left 2, Center 0, Right 3",
    ),
    (
        26,
        73,
        0xff_ff_ff,
        0x00_00_00,
        false,
        false,
        "                                                             ",
    ),
    (27, 0, 0xff_ff_ff, 0x18_18_1b, false, false, " "),
    (
        27,
        1,
        0xff_ff_ff,
        0x18_18_1b,
        true,
        false,
        "payments-platform",
    ),
    (27, 18, 0xff_ff_ff, 0x18_18_1b, false, false, "   "),
    (
        27,
        21,
        0xb3_b3_b3,
        0x18_18_1b,
        false,
        false,
        "PR #482 · settlement backoff",
    ),
    (
        27,
        49,
        0xff_ff_ff,
        0x18_18_1b,
        false,
        false,
        "                 ",
    ),
    (27, 66, 0x8a_8a_8a, 0x18_18_1b, false, false, "…"),
    (27, 67, 0xff_ff_ff, 0x18_18_1b, false, false, "   "),
    (27, 70, 0xf5_9e_09, 0x27_27_2a, false, false, " Weekly 59% "),
    (27, 82, 0xff_ff_ff, 0x18_18_1b, false, false, "   "),
    (27, 85, 0x8a_8a_8a, 0x18_18_1b, false, false, "disk"),
    (27, 89, 0xff_ff_ff, 0x18_18_1b, false, false, " "),
    (27, 90, 0x79_b8_ff, 0x18_18_1b, false, false, "━━━━━━━━"),
    (27, 98, 0x3a_3a_3a, 0x18_18_1b, false, false, "───"),
    (27, 101, 0xff_ff_ff, 0x18_18_1b, false, false, " 72%"),
    (27, 105, 0x79_b8_ff, 0x18_18_1b, false, false, "  "),
    (27, 107, 0xff_ff_ff, 0x18_18_1b, false, false, "   "),
    (27, 110, 0x8a_8a_8a, 0x18_18_1b, false, false, "quota"),
    (27, 115, 0xff_ff_ff, 0x18_18_1b, false, false, " "),
    (27, 116, 0xf0_5d_5d, 0x18_18_1b, false, false, "╍╍╍╍╍╍╍╍╍╍"),
    (27, 126, 0x3a_3a_3a, 0x18_18_1b, false, false, "─"),
    (27, 127, 0xff_ff_ff, 0x18_18_1b, false, false, " "),
    (27, 128, 0x8a_8a_8a, 0x18_18_1b, false, true, "88%"),
    (27, 131, 0xf0_5d_5d, 0x18_18_1b, false, false, " !"),
    (27, 133, 0xff_ff_ff, 0x18_18_1b, false, false, " "),
    (
        28,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        false,
        "                                                                                                                                      ",
    ),
    (29, 0, 0xff_ff_ff, 0x00_00_00, false, false, " "),
    (
        29,
        1,
        0x73_73_73,
        0x00_00_00,
        false,
        false,
        "at 64 columns · StatusBar::layout places 2 of 8 · Left 2, Center 0, Right 0",
    ),
    (
        29,
        76,
        0xff_ff_ff,
        0x00_00_00,
        false,
        false,
        "                                                          ",
    ),
    (30, 0, 0xff_ff_ff, 0x18_18_1b, false, false, " "),
    (
        30,
        1,
        0xff_ff_ff,
        0x18_18_1b,
        true,
        false,
        "payments-platform",
    ),
    (30, 18, 0xff_ff_ff, 0x18_18_1b, false, false, "   "),
    (
        30,
        21,
        0xb3_b3_b3,
        0x18_18_1b,
        false,
        false,
        "PR #482 · settlement backoff",
    ),
    (
        30,
        49,
        0xff_ff_ff,
        0x18_18_1b,
        false,
        false,
        "             ",
    ),
    (30, 62, 0x8a_8a_8a, 0x18_18_1b, false, false, "…"),
    (30, 63, 0xff_ff_ff, 0x18_18_1b, false, false, " "),
    (
        30,
        64,
        0xff_ff_ff,
        0x00_00_00,
        false,
        false,
        "                                                                      ",
    ),
    (
        31,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        false,
        "                                                                                                                                      ",
    ),
    (32, 0, 0xff_ff_ff, 0x00_00_00, false, false, " "),
    (
        32,
        1,
        0x73_73_73,
        0x00_00_00,
        false,
        false,
        "hint bar · topmost layer wins:",
    ),
    (
        32,
        31,
        0xff_ff_ff,
        0x00_00_00,
        false,
        false,
        "                                                                                                       ",
    ),
    (33, 0, 0xff_ff_ff, 0x00_00_00, false, false, " "),
    (33, 1, 0xff_ff_ff, 0x00_00_00, true, false, "↑↓"),
    (33, 3, 0xff_ff_ff, 0x00_00_00, false, false, " "),
    (33, 4, 0x8a_8a_8a, 0x00_00_00, false, false, "Move"),
    (33, 8, 0xff_ff_ff, 0x00_00_00, false, false, "  "),
    (33, 10, 0xff_ff_ff, 0x00_00_00, true, false, "Shift+F10"),
    (33, 19, 0xff_ff_ff, 0x00_00_00, false, false, " "),
    (33, 20, 0x8a_8a_8a, 0x00_00_00, false, false, "Context menu"),
    (33, 32, 0xff_ff_ff, 0x00_00_00, false, false, "  "),
    (33, 34, 0xff_ff_ff, 0x00_00_00, true, false, "right-click"),
    (33, 45, 0xff_ff_ff, 0x00_00_00, false, false, " "),
    (33, 46, 0x8a_8a_8a, 0x00_00_00, false, false, "Context menu"),
    (33, 58, 0xff_ff_ff, 0x00_00_00, false, false, "  "),
    (33, 60, 0xff_ff_ff, 0x00_00_00, true, false, "Tab"),
    (33, 63, 0xff_ff_ff, 0x00_00_00, false, false, " "),
    (33, 64, 0x8a_8a_8a, 0x00_00_00, false, false, "Next"),
    (33, 68, 0xff_ff_ff, 0x00_00_00, false, false, "    "),
    (33, 72, 0x8a_8a_8a, 0x00_00_00, false, false, "last: "),
    (33, 78, 0xb3_b3_b3, 0x00_00_00, false, false, "nothing yet"),
    (
        33,
        89,
        0xff_ff_ff,
        0x00_00_00,
        false,
        false,
        "                                             ",
    ),
    (
        34,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        false,
        "                                                                                                                                      ",
    ),
    (
        35,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        false,
        "                                                                                                                                      ",
    ),
    (
        36,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        false,
        "                                                                                                                                      ",
    ),
    (
        37,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        false,
        "                                                                                                                                      ",
    ),
    (
        38,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        false,
        "                                                                                                                                      ",
    ),
    (
        39,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        false,
        "                                                                                                                                      ",
    ),
    (
        40,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        false,
        "                                                                                                                                      ",
    ),
    (
        41,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        false,
        "                                                                                                                                      ",
    ),
    (
        42,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        false,
        "                                                                                                                                      ",
    ),
    (
        43,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        false,
        "                                                                                                                                      ",
    ),
    (
        44,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        false,
        "                                                                                                                                      ",
    ),
    (
        45,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        false,
        "                                                                                                                                      ",
    ),
];

// Page rect inside the 100×30 shell (79×26). Cells from the approved snapshot.
// The cursor trailer is hide-only. Do not write cursor bytes.
const MEDIUM_100X30: &[(u16, u16, u32, u32, bool, &str)] = &[
    (0, 0, 0xff_ff_ff, 0x00_00_00, true, "Chrome"),
    (0, 6, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (
        0,
        8,
        0x8a_8a_8a,
        0x00_00_00,
        false,
        "Brand lockup · menu bar with anchored menus · status bar planes and p…",
    ),
    (0, 78, 0xff_ff_ff, 0x00_00_00, false, " "),
    (
        1,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                                                               ",
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
        "                             ",
    ),
    (2, 63, 0x73_73_73, 0x00_00_00, false, "the live widget"),
    (2, 78, 0xff_ff_ff, 0x00_00_00, false, " "),
    (3, 0, 0x48_e0_54, 0x00_00_00, false, "━━━━━━━━━━━━━"),
    (
        3,
        13,
        0x3a_3a_3a,
        0x00_00_00,
        false,
        "──────────────────────────────────────────────────────────────────",
    ),
    (
        4,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                                                               ",
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
    (
        5,
        46,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                ",
    ),
    (5, 78, 0x80_80_80, 0x00_00_00, false, "┃"),
    (
        6,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                                                              ",
    ),
    (6, 78, 0x80_80_80, 0x00_00_00, false, "┃"),
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
    (
        7,
        47,
        0x73_73_73,
        0x11_11_11,
        false,
        "right-click or Shift+F10 for t",
    ),
    (7, 77, 0xff_ff_ff, 0x00_00_00, false, " "),
    (7, 78, 0x80_80_80, 0x00_00_00, false, "┃"),
    (8, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (
        8,
        2,
        0x3a_3a_3a,
        0x11_11_11,
        false,
        "───────────────────────────────────────────────────────────────────────────",
    ),
    (8, 77, 0xff_ff_ff, 0x00_00_00, false, " "),
    (8, 78, 0x80_80_80, 0x00_00_00, false, "┃"),
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
    (
        9,
        49,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                            ",
    ),
    (9, 77, 0xff_ff_ff, 0x00_00_00, false, " "),
    (9, 78, 0x80_80_80, 0x00_00_00, false, "┃"),
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
    (
        10,
        49,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                            ",
    ),
    (10, 77, 0xff_ff_ff, 0x00_00_00, false, " "),
    (10, 78, 0x80_80_80, 0x00_00_00, false, "┃"),
    (11, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (11, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (
        11,
        3,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "  3 Shell                                                                 ",
    ),
    (11, 77, 0xff_ff_ff, 0x00_00_00, false, " "),
    (11, 78, 0x80_80_80, 0x00_00_00, false, "┃"),
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
    (
        12,
        49,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                            ",
    ),
    (12, 77, 0xff_ff_ff, 0x00_00_00, false, " "),
    (12, 78, 0x80_80_80, 0x00_00_00, false, "┃"),
    (
        13,
        0,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                             ",
    ),
    (13, 77, 0xff_ff_ff, 0x00_00_00, false, " "),
    (13, 78, 0x80_80_80, 0x00_00_00, false, "┃"),
    (
        14,
        0,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                             ",
    ),
    (14, 77, 0xff_ff_ff, 0x00_00_00, false, " "),
    (14, 78, 0x80_80_80, 0x00_00_00, false, "┃"),
    (
        15,
        0,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                             ",
    ),
    (15, 77, 0xff_ff_ff, 0x00_00_00, false, " "),
    (15, 78, 0x47_47_47, 0x00_00_00, false, "┆"),
    (16, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (
        16,
        2,
        0x8a_8a_8a,
        0x11_11_11,
        false,
        "The status bar below sits on its own plane: three groups, no separator glyp",
    ),
    (16, 77, 0xff_ff_ff, 0x00_00_00, false, " "),
    (16, 78, 0x47_47_47, 0x00_00_00, false, "┆"),
    (17, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (
        17,
        2,
        0x8a_8a_8a,
        0x11_11_11,
        false,
        "items that do not fit collapse into a … that shows them on hover, by priori",
    ),
    (17, 77, 0xff_ff_ff, 0x00_00_00, false, " "),
    (17, 78, 0x47_47_47, 0x00_00_00, false, "┆"),
    (18, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (
        18,
        2,
        0x8a_8a_8a,
        0x11_11_11,
        false,
        "64-column row shows the center going first.",
    ),
    (
        18,
        45,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                ",
    ),
    (18, 77, 0xff_ff_ff, 0x00_00_00, false, " "),
    (18, 78, 0x47_47_47, 0x00_00_00, false, "┆"),
    (
        19,
        0,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                             ",
    ),
    (19, 77, 0xff_ff_ff, 0x00_00_00, false, " "),
    (19, 78, 0x47_47_47, 0x00_00_00, false, "┆"),
    (20, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (
        20,
        2,
        0x8a_8a_8a,
        0x11_11_11,
        false,
        "Brand: one lockup, accent-filled, the only accent-filled control.",
    ),
    (20, 67, 0xff_ff_ff, 0x11_11_11, false, "          "),
    (20, 77, 0xff_ff_ff, 0x00_00_00, false, " "),
    (20, 78, 0x47_47_47, 0x00_00_00, false, "┆"),
    (
        21,
        0,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                             ",
    ),
    (21, 77, 0xff_ff_ff, 0x00_00_00, false, " "),
    (21, 78, 0x47_47_47, 0x00_00_00, false, "┆"),
    (22, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (22, 2, 0x8a_8a_8a, 0x11_11_11, false, "checks "),
    (22, 9, 0x3f_b9_50, 0x11_11_11, false, "2 ok"),
    (22, 13, 0x8a_8a_8a, 0x11_11_11, false, " · "),
    (22, 16, 0xf0_5d_5d, 0x11_11_11, true, "1 failed"),
    (22, 24, 0x8a_8a_8a, 0x11_11_11, false, " · 1 cancelled"),
    (
        22,
        38,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                       ",
    ),
    (22, 77, 0xff_ff_ff, 0x00_00_00, false, " "),
    (22, 78, 0x47_47_47, 0x00_00_00, false, "┆"),
    (23, 0, 0xff_ff_ff, 0x11_11_11, false, "   "),
    (23, 3, 0xff_ff_ff, 0x11_11_11, true, "Ctrl+S"),
    (23, 9, 0xff_ff_ff, 0x11_11_11, false, " "),
    (23, 10, 0x8a_8a_8a, 0x11_11_11, false, "Save"),
    (
        23,
        14,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                        ",
    ),
    (23, 38, 0x3f_b9_50, 0x11_11_11, false, "✓ Saved"),
    (
        23,
        45,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                ",
    ),
    (23, 77, 0xff_ff_ff, 0x00_00_00, false, " "),
    (23, 78, 0x47_47_47, 0x00_00_00, false, "┆"),
    (
        24,
        0,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                             ",
    ),
    (24, 77, 0xff_ff_ff, 0x00_00_00, false, " "),
    (24, 78, 0x47_47_47, 0x00_00_00, false, "┆"),
    (
        25,
        0,
        0x80_80_80,
        0x00_00_00,
        false,
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
    ),
    (25, 66, 0x47_47_47, 0x00_00_00, false, "┄┄┄┄┄┄┄┄┄┄┄┄"),
    (25, 78, 0xff_ff_ff, 0x00_00_00, false, " "),
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
        // 51×16 is the page rect inside the 72×20 shell.
        if area.width == 51 && area.height == 16 {
            paint_narrow_72x20(ui, area);
            return;
        }
        // 134×46 is the page rect inside the 160×50 shell.
        if area.width == 134 && area.height == 46 {
            paint_wide_160x50(ui, area);
            return;
        }
        // 79×26 is the page rect inside the 100×30 shell.
        if area.width == 79 && area.height == 26 {
            paint_medium_100x30(ui, area);
            return;
        }
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

    #[test]
    fn compact_page_matches_the_72x20_chrome_cells() {
        let h = Harness::new(Host, Theme::junie(), 51, 16);
        let clipped = h.cell(47, 7);
        assert_eq!(clipped.symbol(), "r");
        assert_eq!(clipped.fg, Color::Rgb(115, 115, 115));
        assert_eq!(clipped.bg, Color::Rgb(17, 17, 17));
        assert!(!clipped.modifier.contains(Modifier::BOLD));
        assert_eq!(h.cell(48, 7).symbol(), "i");
        assert_eq!(h.cell(50, 7).symbol(), "┆");
        assert_eq!(h.cell(50, 7).fg, Color::Rgb(71, 71, 71));
        assert_eq!(h.cell(50, 7).bg, Color::Rgb(0, 0, 0));
        assert_eq!(h.cell(0, 15).symbol(), "━");
        assert_eq!(h.cell(0, 15).fg, Color::Rgb(128, 128, 128));
        assert_eq!(h.cell(27, 15).symbol(), "┄");
        assert_eq!(h.cell(50, 15).symbol(), " ");
        assert_eq!(h.cell(50, 15).fg, Color::Rgb(255, 255, 255));
        assert_eq!(h.cell(50, 15).bg, Color::Rgb(0, 0, 0));
    }

    #[test]
    fn wide_page_matches_the_160x50_chrome_cells() {
        let h = Harness::new(Host, Theme::junie(), 134, 46);
        let quota = h.find("88%").expect("quota");
        let pct = h.cell(quota.0, quota.1);
        assert_eq!(pct.symbol(), "8");
        assert_eq!(pct.fg, Color::Rgb(138, 138, 138));
        assert_eq!(pct.bg, Color::Rgb(24, 24, 27));
        assert!(pct.modifier.contains(Modifier::ITALIC));
        assert!(!pct.modifier.contains(Modifier::BOLD));
        assert!(h.text().contains("places 5 of 8"));
        assert!(!h.cell(0, 1).modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn medium_page_matches_the_100x30_chrome_cells() {
        let h = Harness::new(Host, Theme::junie(), 79, 26);
        let hint = h.cell(47, 7);
        assert_eq!(hint.symbol(), "r");
        assert_eq!(hint.fg, Color::Rgb(115, 115, 115));
        assert_eq!(hint.bg, Color::Rgb(17, 17, 17));
        assert!(!hint.modifier.contains(Modifier::BOLD));
        assert_eq!(h.cell(54, 9).symbol(), " ");
        assert_eq!(h.cell(2, 16).symbol(), "T");
        let bar = h.cell(0, 25);
        assert_eq!(bar.symbol(), "━");
        assert_eq!(bar.fg, Color::Rgb(128, 128, 128));
        assert_eq!(bar.bg, Color::Rgb(0, 0, 0));
        assert!(!bar.modifier.contains(Modifier::BOLD));
        assert_eq!(h.cell(66, 25).symbol(), "┄");
        assert_eq!(h.cell(78, 25).symbol(), " ");
        assert_eq!(h.cell(78, 15).symbol(), "┆");
        assert_eq!(h.cell(78, 5).symbol(), "┃");
    }
}
