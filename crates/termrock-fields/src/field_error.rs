use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::{Color, Modifier};

use crate::text::width;
use crate::theme::{PaintStyle, Surface};
use crate::ui::Ui;

const GUTTER: u16 = 2;

pub struct FieldErrorChrome<'a> {
    pub label: &'a str,

    pub field: Rect,

    pub value: &'a str,
    pub message: &'a str,
    pub ink: Color,

    pub message_bg: Color,
}

pub fn paint_field_error(ui: &mut Ui<'_>, mark: FieldErrorChrome<'_>) {
    let field = mark.field;
    if field.is_empty() {
        return;
    }
    let gutter = GUTTER.min(field.width);
    let value_x = field.x.saturating_add(gutter);
    if field.y > 0 && !mark.label.is_empty() {
        let star_x = value_x.saturating_add(width(mark.label)).saturating_add(1);
        recolor_at(ui, star_x, field.y.saturating_sub(1), mark.ink);
    }
    if gutter == GUTTER {
        let gap_x = field.x.saturating_add(1);
        if gap_x < value_x {
            recolor_at(ui, gap_x, field.y, mark.ink);
        }
    }
    let pad_x = value_x.saturating_add(width(mark.value));
    if field.width >= 2 {
        let bang_x = field.right().saturating_sub(2);
        let tail_x = field.right().saturating_sub(1);
        if pad_x < bang_x {
            ui.recolor_fg(
                Rect {
                    x: pad_x,
                    y: field.y,
                    width: bang_x.saturating_sub(pad_x),
                    height: 1,
                },
                |_fg, _bg| Some(mark.ink),
            );
        }
        if bang_x >= field.x {
            let face = ui.theme_ref().bg(Surface::Field);
            ui.paint_cell(
                Position::new(bang_x, field.y),
                "!",
                PaintStyle::new()
                    .fg(mark.ink)
                    .bg(face)
                    .add_modifier(Modifier::BOLD),
            );
        }
        if tail_x > bang_x && tail_x >= pad_x {
            recolor_at(ui, tail_x, field.y, mark.ink);
        }
    }
    if mark.message.is_empty() {
        return;
    }
    let msg_y = field.y.saturating_add(1);
    ui.paint_str(
        Rect {
            x: value_x,
            y: msg_y,
            width: field.width.saturating_sub(gutter).max(1),
            height: 1,
        },
        mark.message,
        PaintStyle::new().fg(mark.ink).bg(mark.message_bg),
    );
}

fn recolor_at(ui: &mut Ui<'_>, x: u16, y: u16, ink: Color) {
    ui.recolor_fg(Rect::new(x, y, 1, 1), |_fg, _bg| Some(ink));
}
