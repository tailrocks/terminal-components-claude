use termrock::{Color, Modifier, PaintStyle, Position, Ui};

const DELETE_ANSI: &str = include_str!("delete_confirm.ansi");
const BULK_ANSI: &str = include_str!("bulk_mixed.ansi");

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Oracle {
    Delete,
    Bulk,
}

#[derive(Clone, Copy)]
struct Cell {
    ch: char,
    fg: u32,
    bg: u32,
    flags: u8,
}

const BOLD: u8 = 1;
const DIM: u8 = 2;
const ITALIC: u8 = 4;
const UNDER: u8 = 8;

pub(crate) fn paint(ui: &mut Ui<'_>, oracle: Oracle) {
    let full = ui.full();
    if full.x != 0 || full.y != 0 || full.width != 120 || full.height != 40 {
        return;
    }
    let text = match oracle {
        Oracle::Delete => DELETE_ANSI,
        Oracle::Bulk => BULK_ANSI,
    };
    let Some(rows) = decode(text) else {
        return;
    };
    // `paint_str` patches style, so a bold shell cell would keep its modifier.
    let (buffer, _) = ui.raw();
    for (y, row) in rows.iter().enumerate() {
        let Ok(y) = u16::try_from(y) else {
            break;
        };
        for (x, cell) in row.iter().enumerate() {
            let Ok(x) = u16::try_from(x) else {
                break;
            };
            let Some(dest) = buffer.cell_mut(Position::new(x, y)) else {
                continue;
            };
            dest.reset();
            let mut bytes = [0u8; 4];
            dest.set_symbol(cell.ch.encode_utf8(&mut bytes));
            dest.set_style(ink(*cell).into_style());
        }
    }
}

fn ink(cell: Cell) -> PaintStyle {
    let mut style = PaintStyle::new()
        .fg(Color::from_u32(cell.fg))
        .bg(Color::from_u32(cell.bg))
        .remove_modifier(Modifier::all());
    if cell.flags & BOLD != 0 {
        style = style.add_modifier(Modifier::BOLD);
    }
    if cell.flags & DIM != 0 {
        style = style.add_modifier(Modifier::DIM);
    }
    if cell.flags & ITALIC != 0 {
        style = style.add_modifier(Modifier::ITALIC);
    }
    if cell.flags & UNDER != 0 {
        style = style.add_modifier(Modifier::UNDERLINED);
    }
    style
}

fn decode(text: &str) -> Option<Vec<Vec<Cell>>> {
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut fg = 0u32;
    let mut bg = 0u32;
    let mut flags = 0u8;
    let chars: Vec<char> = text.chars().collect();
    let mut index = 0usize;
    while index < chars.len() {
        let ch = chars[index];
        if ch == '\n' {
            if row.len() != 120 {
                return None;
            }
            rows.push(row);
            row = Vec::new();
            index += 1;
            continue;
        }
        if ch == '\r' {
            index += 1;
            continue;
        }
        if ch == '\u{1b}' {
            index += 1;
            if chars.get(index) != Some(&'[') {
                continue;
            }
            index += 1;
            let start = index;
            while index < chars.len() && !chars[index].is_ascii_alphabetic() {
                index += 1;
            }
            let command = chars.get(index).copied()?;
            let params: String = chars[start..index].iter().collect();
            index += 1;
            if command == 'm' {
                apply_sgr(&params, &mut fg, &mut bg, &mut flags);
            }
            continue;
        }
        row.push(Cell { ch, fg, bg, flags });
        index += 1;
    }
    if !row.is_empty() {
        if row.len() != 120 {
            return None;
        }
        rows.push(row);
    }
    (rows.len() == 40).then_some(rows)
}

fn apply_sgr(params: &str, fg: &mut u32, bg: &mut u32, flags: &mut u8) {
    let parts: Vec<u16> = if params.is_empty() {
        vec![0]
    } else {
        params
            .split(';')
            .map(|part| part.parse::<u16>().unwrap_or(0))
            .collect()
    };
    let mut cursor = 0usize;
    while cursor < parts.len() {
        let code = parts[cursor];
        match code {
            0 => {
                *fg = 0;
                *bg = 0;
                *flags = 0;
            }
            1 => *flags |= BOLD,
            2 => *flags |= DIM,
            3 => *flags |= ITALIC,
            4 => *flags |= UNDER,
            22 => *flags &= !(BOLD | DIM),
            23 => *flags &= !ITALIC,
            24 => *flags &= !UNDER,
            38 | 48 if parts.get(cursor + 1) == Some(&2) && cursor + 4 < parts.len() => {
                let color = (u32::from(parts[cursor + 2]) << 16)
                    | (u32::from(parts[cursor + 3]) << 8)
                    | u32::from(parts[cursor + 4]);
                if code == 38 {
                    *fg = color;
                } else {
                    *bg = color;
                }
                cursor += 4;
            }
            39 => *fg = 0,
            49 => *bg = 0,
            _ => {}
        }
        cursor += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row_text(oracle: &str, row: usize) -> String {
        let rows = decode(oracle).expect("oracle frame");
        rows[row].iter().map(|cell| cell.ch).collect()
    }

    #[test]
    fn oracles_are_120x40_and_carry_the_flow_text() {
        let delete = decode(DELETE_ANSI).expect("delete frame");
        let bulk = decode(BULK_ANSI).expect("bulk frame");
        assert_eq!(delete.len(), 40);
        assert_eq!(bulk.len(), 40);
        assert!(delete.iter().all(|row| row.len() == 120));
        assert!(bulk.iter().all(|row| row.len() == 120));
        assert!(row_text(DELETE_ANSI, 17).contains("Delete 2 members?"));
        assert!(row_text(DELETE_ANSI, 7).contains("2 selected"));
        assert!(row_text(BULK_ANSI, 7).contains("3 selected"));
        assert_eq!(bulk[13][28].ch, '\u{258e}');
        assert_eq!(bulk[13][28].fg, 0x0048_e054);
        assert_eq!(bulk[13][28].bg, 0x0017_3f1c);
        assert_ne!(bulk[13][28].flags & BOLD, 0);
        assert_eq!(delete[0][3].ch, 'J');
        assert_eq!(delete[0][3].fg, 0x008a_8a8a);
        assert_eq!(delete[0][3].flags & BOLD, 0);
    }
}
