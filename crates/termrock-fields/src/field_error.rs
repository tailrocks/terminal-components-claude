//! Error chrome for a field that is already on the buffer.
//!
//! Baseline `TextInput` paints this when a required field fails: the mark
//! one column after the label, danger ink on the field padding, `!` in the
//! trailing slot, and the message under the value. The value or placeholder
//! stays as the caller painted it.

use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::{Color, Modifier};

use crate::text::width;
use crate::theme::{PaintStyle, Surface};
use crate::ui::Ui;

/// Two columns: the focus gutter, then the pad before the value.
const GUTTER: u16 = 2;

pub struct FieldErrorChrome<'a> {
    /// Label text. The required mark sits one column after it.
    pub label: &'a str,
    /// Control row, focus gutter included. The value starts `GUTTER` columns in.
    pub field: Rect,
    /// Value or placeholder already painted at the value origin.
    pub value: &'a str,
    pub message: &'a str,
    pub ink: Color,
    /// Background of the message row. Field padding keeps the background it has.
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

#[cfg(test)]
mod tests {
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::{Position, Rect};
    use ratatui_core::style::{Color, Modifier};

    use super::{FieldErrorChrome, paint_field_error};
    use crate::runtime::Runtime;
    use crate::runtime::stub::Stub;
    use crate::theme::{PaintStyle, Surface, Theme};

    #[test]
    fn error_chrome_recolors_padding_and_keeps_the_value() {
        let theme = Theme::junie();
        let ink = Color::Rgb(0xf0, 0x5d, 0x5d);
        let face = theme.bg(Surface::Field);
        let card = theme.bg(Surface::Elevated);
        let muted = Color::Rgb(0x8a, 0x8a, 0x8a);
        let white = Color::Rgb(0xff, 0xff, 0xff);
        let green = Color::Rgb(0x48, 0xe0, 0x54);
        let area = Rect::new(0, 0, 40, 12);
        let mut runtime = Runtime::new(Stub::default(), theme);
        let mut buffer = Buffer::empty(area);
        runtime
            .draw_scene(area, &mut buffer, |ui, _area| {
                ui.paint_cell(
                    Position::new(9, 1),
                    "*",
                    PaintStyle::new()
                        .fg(muted)
                        .bg(card)
                        .add_modifier(Modifier::BOLD),
                );
                ui.paint_cell(
                    Position::new(2, 2),
                    "▎",
                    PaintStyle::new().fg(green).bg(face),
                );
                ui.paint_cell(
                    Position::new(3, 2),
                    " ",
                    PaintStyle::new().fg(white).bg(face),
                );
                ui.paint_str(
                    Rect::new(4, 2, 2, 1),
                    "ab",
                    PaintStyle::new().fg(muted).bg(face),
                );
                for x in 6..22 {
                    ui.paint_cell(
                        Position::new(x, 2),
                        " ",
                        PaintStyle::new().fg(white).bg(face),
                    );
                }
                for x in 4..12 {
                    ui.paint_cell(
                        Position::new(x, 3),
                        " ",
                        PaintStyle::new().fg(muted).bg(card),
                    );
                }
                paint_field_error(
                    ui,
                    FieldErrorChrome {
                        label: "Name",
                        field: Rect::new(2, 2, 20, 1),
                        value: "ab",
                        message: "Required",
                        ink,
                        message_bg: card,
                    },
                );
            })
            .commit_presented();

        let star = buffer.cell((9, 1)).expect("star");
        assert_eq!(star.symbol(), "*");
        assert_eq!(star.fg, ink);
        assert_eq!(star.bg, card);
        assert!(star.modifier.contains(Modifier::BOLD));

        let gutter = buffer.cell((2, 2)).expect("gutter");
        assert_eq!(gutter.symbol(), "▎");
        assert_eq!(gutter.fg, green);
        assert_eq!(gutter.bg, face);

        let gap = buffer.cell((3, 2)).expect("gap");
        assert_eq!(gap.symbol(), " ");
        assert_eq!(gap.fg, ink);
        assert_eq!(gap.bg, face);
        assert!(!gap.modifier.contains(Modifier::BOLD));

        let value = buffer.cell((4, 2)).expect("value");
        assert_eq!(value.symbol(), "a");
        assert_eq!(value.fg, muted);
        let value_b = buffer.cell((5, 2)).expect("value");
        assert_eq!(value_b.symbol(), "b");
        assert_eq!(value_b.fg, muted);

        let pad = buffer.cell((6, 2)).expect("pad");
        assert_eq!(pad.symbol(), " ");
        assert_eq!(pad.fg, ink);
        assert_eq!(pad.bg, face);

        let bang = buffer.cell((20, 2)).expect("bang");
        assert_eq!(bang.symbol(), "!");
        assert_eq!(bang.fg, ink);
        assert_eq!(bang.bg, face);
        assert!(bang.modifier.contains(Modifier::BOLD));

        let tail = buffer.cell((21, 2)).expect("tail");
        assert_eq!(tail.symbol(), " ");
        assert_eq!(tail.fg, ink);
        assert_eq!(tail.bg, face);
        assert!(!tail.modifier.contains(Modifier::BOLD));

        let message = buffer.cell((4, 3)).expect("message");
        assert_eq!(message.symbol(), "R");
        assert_eq!(message.fg, ink);
        assert_eq!(message.bg, card);
        assert!(!message.modifier.contains(Modifier::BOLD));
    }
}
