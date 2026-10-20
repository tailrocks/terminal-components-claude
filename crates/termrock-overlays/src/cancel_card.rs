//! Cancel-launch dialog. Title and body sit inside the border.
//!
//! Empty rows keep the dimmed page ink. The card is 54×11, one row below centre.

use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Modifier};

use crate::theme::PaintStyle;
use crate::ui::Ui;

const WIDTH: u16 = 54;
const HEIGHT: u16 = 11;
const Y_BIAS: u16 = 1;
const TEXT_X: u16 = 3;
const WRAP: usize = 50;
const CARD: u32 = 0x0018_181b;
const BORDER: u32 = 0x004d_4d4d;
const WHITE: u32 = 0x00ff_ffff;
const BODY: u32 = 0x00b3_b3b3;
const CLOSE_INK: u32 = 0x00b3_b3b3;
const FOCUS: u32 = 0x0048_e054;
const BUTTON: u32 = 0x0027_272a;
const DANGER: u32 = 0x00ff_6b6b;

/// Paint the cancel card on the current page.
///
/// `body` is wrapped to the baseline line width. The keep button takes focus.
pub fn paint(ui: &mut Ui<'_>, title: &str, body: &str, keep: &str, cancel: &str) {
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
        .saturating_add(screen.height.saturating_sub(HEIGHT) / 2)
        .saturating_add(Y_BIAS);
    let card = Rect::new(origin_x, origin_y, WIDTH, HEIGHT);
    ui.fill_keep_fg(card, Color::from_u32(CARD));
    let border = ink(BORDER, CARD, false);
    ui.frame(card, border);
    paint_at(ui, origin_x, origin_y, 49, 0, " ", border);
    paint_at(
        ui,
        origin_x,
        origin_y,
        50,
        0,
        "×",
        ink(CLOSE_INK, CARD, false),
    );
    paint_at(ui, origin_x, origin_y, 51, 0, " ", border);
    paint_at(
        ui,
        origin_x,
        origin_y,
        TEXT_X,
        2,
        title,
        ink(WHITE, CARD, true),
    );
    for (index, line) in wrap(body).into_iter().take(3).enumerate() {
        let row = u16::try_from(index.saturating_add(4)).unwrap_or(4);
        paint_at(
            ui,
            origin_x,
            origin_y,
            TEXT_X,
            row,
            &line,
            ink(BODY, CARD, false),
        );
    }
    paint_at(
        ui,
        origin_x,
        origin_y,
        18,
        8,
        "▎",
        ink(FOCUS, BUTTON, false),
    );
    paint_mnemonic(ui, origin_x, origin_y, 19, keep, ink(WHITE, BUTTON, true));
    paint_at(
        ui,
        origin_x,
        origin_y,
        36,
        8,
        " ",
        ink(BUTTON, BUTTON, false),
    );
    paint_mnemonic(
        ui,
        origin_x,
        origin_y,
        37,
        cancel,
        ink(DANGER, BUTTON, false),
    );
}

fn paint_mnemonic(
    ui: &mut Ui<'_>,
    origin_x: u16,
    origin_y: u16,
    x: u16,
    label: &str,
    style: PaintStyle,
) {
    let mut chars = label.chars();
    let Some(first) = chars.next() else {
        return;
    };
    let rest: String = chars.collect();
    let marked = style.add_modifier(Modifier::UNDERLINED);
    paint_at(ui, origin_x, origin_y, x, 8, &first.to_string(), marked);
    let tail = format!("{rest} ");
    paint_at(ui, origin_x, origin_y, x.saturating_add(1), 8, &tail, style);
}

fn wrap(body: &str) -> Vec<String> {
    let mut rows = Vec::new();
    let mut cur = String::new();
    for word in body.split_whitespace() {
        let trial = if cur.is_empty() {
            word.to_owned()
        } else {
            format!("{cur} {word}")
        };
        if trial.chars().count() <= WRAP {
            cur = trial;
        } else {
            if !cur.is_empty() {
                rows.push(cur);
            }
            cur = word.to_owned();
        }
    }
    if !cur.is_empty() {
        rows.push(cur);
    }
    rows
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
