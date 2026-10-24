//! Info dialog whose title sits in the border, matching the baseline frame.
//!
//! Empty rows keep the dimmed page ink. Fact rows and the Close button paint
//! over that fill.

use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Modifier};

use crate::theme::{FgStep, GlyphRole, PaintStyle, Role, StylePatch, Surface};
use crate::ui::Ui;

const WIDTH: u16 = 66;
const HEIGHT: u16 = 11;
const LABEL_X: u16 = 4;
const LABEL_W: u16 = 8;
const CARD: u32 = 0x0018_181b;
const BORDER: u32 = 0x004d_4d4d;
const WHITE: u32 = 0x00ff_ffff;
const MUTED: u32 = 0x008a_8a8a;
const FAINT: u32 = 0x0073_7373;
const CLOSE_INK: u32 = 0x00b3_b3b3;
const FOCUS: u32 = 0x0048_e054;
const BUTTON: u32 = 0x0027_272a;

/// One label and value inside the info card.
pub struct Fact<'a> {
    pub label: &'a str,
    pub value: &'a str,
}

/// Paint fact rows in a dialog body: a focus gutter column, the muted label,
/// and the bright value that runs to the right edge.
///
/// Row `focus` carries the accent gutter mark and bold value ink; the other
/// rows keep the card fill. The two cells before the right border stay as the
/// chrome fill left them.
pub fn paint_facts(ui: &mut Ui<'_>, body: Rect, facts: &[Fact<'_>], focus: usize) {
    let gutter_pad = ui.paint_patch(
        &StylePatch::new()
            .set_fg(Role::Fg(FgStep::Primary))
            .set_bg(Role::Surface(Surface::Elevated)),
    );
    let gutter_pad_focus = ui.paint_patch(
        &StylePatch::new()
            .set_fg(Role::Fg(FgStep::Primary))
            .set_bg(Role::Surface(Surface::Elevated))
            .add(Modifier::BOLD),
    );
    let gutter_focus = ui.paint_patch(
        &StylePatch::new()
            .set_fg(Role::Accent)
            .set_bg(Role::Surface(Surface::Elevated))
            .add(Modifier::BOLD),
    );
    let gutter_rest = ui.paint_patch(
        &StylePatch::new()
            .set_fg(Role::Surface(Surface::Elevated))
            .set_bg(Role::Surface(Surface::Elevated)),
    );
    let label = ui.paint_patch(
        &StylePatch::new()
            .set_fg(Role::Fg(FgStep::Muted))
            .set_bg(Role::Surface(Surface::Elevated)),
    );
    let value = ui.paint_patch(
        &StylePatch::new()
            .set_fg(Role::Fg(FgStep::Primary))
            .set_bg(Role::Surface(Surface::Elevated)),
    );
    let value_focus = ui.paint_patch(
        &StylePatch::new()
            .set_fg(Role::Fg(FgStep::Primary))
            .set_bg(Role::Surface(Surface::Elevated))
            .add(Modifier::BOLD),
    );
    for (index, fact) in facts.iter().enumerate() {
        let Some(y) = body.y.checked_add(u16::try_from(index).unwrap_or(u16::MAX)) else {
            continue;
        };
        let focused = index == focus;
        if focused {
            ui.glyph(
                Rect::new(body.x, y, 1, 1),
                GlyphRole::FocusBar,
                gutter_focus,
            );
        } else {
            ui.paint_str(Rect::new(body.x, y, 1, 1), " ", gutter_rest);
        }
        ui.paint_str(
            Rect::new(body.x.saturating_add(1), y, 1, 1),
            " ",
            if focused {
                gutter_pad_focus
            } else {
                gutter_pad
            },
        );
        let label_x = body.x.saturating_add(2);
        let cols = u16::try_from(fact.label.chars().count()).unwrap_or(0);
        ui.paint_str(Rect::new(label_x, y, cols, 1), fact.label, label);
        let gap = LABEL_W.saturating_sub(cols);
        let value_start = label_x.saturating_add(cols);
        let value_x = value_start.saturating_add(gap);
        // The value span runs to the body edge and keeps the value ink on its
        // padding cells, so the fill first, then the text over it.
        let span = body.right().saturating_sub(value_start);
        let row_style = if focused { value_focus } else { value };
        ui.fill(Rect::new(value_start, y, span, 1), row_style);
        ui.paint_str(
            Rect::new(value_x, y, span.saturating_sub(gap), 1),
            fact.value,
            row_style,
        );
    }
}

/// Paint the info card on the current page.
///
/// The card is 66×11 and centred. The last screen row is left for the footer.
pub fn paint(ui: &mut Ui<'_>, facts: &[Fact<'_>]) {
    let screen = ui.full();
    if screen.height < HEIGHT || screen.width < WIDTH {
        return;
    }
    ui.modal_backdrop(Rect {
        height: screen.height.saturating_sub(1),
        ..screen
    });
    let origin_x = screen
        .x
        .saturating_add(screen.width.saturating_sub(WIDTH) / 2);
    let origin_y = screen
        .y
        .saturating_add(screen.height.saturating_sub(HEIGHT) / 2);
    let card = Rect::new(origin_x, origin_y, WIDTH, HEIGHT);
    let card_bg = Color::from_u32(CARD);
    ui.fill_keep_fg(card, card_bg);
    let border = ink(BORDER, CARD, false);
    ui.frame(card, border);
    paint_at(
        ui,
        origin_x,
        origin_y,
        2,
        0,
        " Debug info ",
        ink(WHITE, CARD, true),
    );
    paint_at(
        ui,
        origin_x,
        origin_y,
        49,
        0,
        " read-only ",
        ink(FAINT, CARD, false),
    );
    paint_at(ui, origin_x, origin_y, 61, 0, " ", border);
    paint_at(
        ui,
        origin_x,
        origin_y,
        62,
        0,
        "×",
        ink(CLOSE_INK, CARD, false),
    );
    paint_at(ui, origin_x, origin_y, 63, 0, " ", border);
    let shown = facts.len().min(8);
    for (index, fact) in facts.iter().take(shown).enumerate() {
        let row = u16::try_from(index.saturating_add(1)).unwrap_or(1);
        paint_fact(ui, origin_x, origin_y, row, fact, index == 0);
    }
    paint_at(
        ui,
        origin_x,
        origin_y,
        56,
        9,
        " ",
        ink(BUTTON, BUTTON, false),
    );
    paint_at(
        ui,
        origin_x,
        origin_y,
        57,
        9,
        "Close ",
        ink(WHITE, BUTTON, false),
    );
}

fn paint_fact(
    ui: &mut Ui<'_>,
    origin_x: u16,
    origin_y: u16,
    row: u16,
    fact: &Fact<'_>,
    focus: bool,
) {
    if focus {
        paint_at(ui, origin_x, origin_y, 2, row, "▎", ink(FOCUS, CARD, true));
        paint_at(ui, origin_x, origin_y, 3, row, " ", ink(WHITE, CARD, true));
    } else {
        paint_at(ui, origin_x, origin_y, 2, row, " ", ink(CARD, CARD, false));
        paint_at(ui, origin_x, origin_y, 3, row, " ", ink(WHITE, CARD, false));
    }
    paint_at(
        ui,
        origin_x,
        origin_y,
        LABEL_X,
        row,
        fact.label,
        ink(MUTED, CARD, false),
    );
    let label_cols = u16::try_from(fact.label.chars().count()).unwrap_or(0);
    let start = LABEL_X.saturating_add(label_cols);
    // The two cells before the right border stay dimmed page ink.
    let end = WIDTH.saturating_sub(3);
    let span = end.saturating_sub(start);
    let gap = LABEL_W.saturating_sub(label_cols);
    let mut text = String::new();
    for _ in 0..gap {
        text.push(' ');
    }
    text.push_str(fact.value);
    while u16::try_from(text.chars().count()).unwrap_or(span) < span {
        text.push(' ');
    }
    let shown: String = text.chars().take(usize::from(span)).collect();
    paint_at(
        ui,
        origin_x,
        origin_y,
        start,
        row,
        &shown,
        ink(WHITE, CARD, focus),
    );
}

fn paint_at(
    ui: &mut Ui<'_>,
    origin_x: u16,
    origin_y: u16,
    x: u16,
    y: u16,
    text: &str,
    style: PaintStyle,
) {
    let columns = u16::try_from(text.chars().count()).unwrap_or(0);
    ui.paint_str(
        Rect::new(
            origin_x.saturating_add(x),
            origin_y.saturating_add(y),
            columns,
            1,
        ),
        text,
        style,
    );
}

fn ink(fg: u32, bg: u32, bold: bool) -> PaintStyle {
    let mut style = PaintStyle::new()
        .fg(Color::from_u32(fg))
        .bg(Color::from_u32(bg))
        .remove_modifier(
            Modifier::BOLD
                | Modifier::DIM
                | Modifier::ITALIC
                | Modifier::UNDERLINED
                | Modifier::REVERSED
                | Modifier::CROSSED_OUT,
        );
    if bold {
        style = style.add_modifier(Modifier::BOLD);
    }
    style
}
