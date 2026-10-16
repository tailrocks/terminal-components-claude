//! The committed snapshot format: tuiscotti's normalized `.ansi` dump
//! (`render::ansi_dump`, byte-identical) plus ONE trailer carrying the
//! cursor as real VT sequences, and its exact inverse (`.ansi` → `Frame`)
//! so PNG/HTML review artifacts can be rebuilt from an approved file.
//!
//! Trailer: `ESC[?25l` when the cursor is hidden (an invisible cursor's
//! position and shape are not observable), else
//! `ESC[?25h ESC[<y>;<x>H ESC[<Ps> q` (DECSCUSR shape/blink).
//!
//! Upstream gap: `ansi_dump` writes no cursor at all, so a plain `.ansi`
//! gate misses caret moves; see docs/visual-testing/tuiscotti-migration.md.

use tuiscotti::frame::{
    Cell, Color, Cursor, CursorStyle, FRAME_VERSION, Frame, Mods, Provenance, Rgb, UnderlineStyle,
};
use tuiscotti::render::ansi_dump;
use unicode_segmentation::UnicodeSegmentation;

/// `ansi_dump(frame)` + cursor trailer.
pub fn encode(frame: &Frame) -> String {
    let mut s = ansi_dump(frame);
    let c = frame.cursor;
    if c.visible {
        let ps = match (c.style, c.blinking) {
            (CursorStyle::Block, true) => 1,
            (CursorStyle::Block, false) => 2,
            (CursorStyle::Underline, true) => 3,
            (CursorStyle::Underline, false) => 4,
            (CursorStyle::Bar, true) => 5,
            (CursorStyle::Bar, false) => 6,
        };
        s.push_str(&format!(
            "\x1b[?25h\x1b[{};{}H\x1b[{ps} q",
            c.y + 1,
            c.x + 1
        ));
    } else {
        s.push_str("\x1b[?25l");
    }
    s
}

#[derive(Clone, Copy, Default)]
struct Style {
    fg: Color,
    bg: Color,
    ul: Color,
    mods: Mods,
}

fn parse_color(toks: &[&str], i: &mut usize) -> Option<Color> {
    let kind = *toks.get(*i + 1)?;
    if kind == "5" {
        let c = Color::Indexed(toks.get(*i + 2)?.parse().ok()?);
        *i += 3;
        Some(c)
    } else {
        let c = Color::Rgb(Rgb::new(
            toks.get(*i + 2)?.parse().ok()?,
            toks.get(*i + 3)?.parse().ok()?,
            toks.get(*i + 4)?.parse().ok()?,
        ));
        *i += 5;
        Some(c)
    }
}

fn parse_sgr(params: &str) -> Option<Style> {
    let mut st = Style::default();
    let toks: Vec<&str> = params.split(';').collect();
    let mut i = 0;
    while i < toks.len() {
        match toks[i] {
            "1" => st.mods.bold = true,
            "2" => st.mods.dim = true,
            "3" => st.mods.italic = true,
            "5" => st.mods.blink = true,
            "7" => st.mods.reverse = true,
            "8" => st.mods.hidden = true,
            "9" => st.mods.strikethrough = true,
            "4" | "4:2" | "4:3" | "4:4" | "4:5" => {
                st.mods.underline = true;
                st.mods.underline_style = match toks[i] {
                    "4" => UnderlineStyle::Single,
                    "4:2" => UnderlineStyle::Double,
                    "4:3" => UnderlineStyle::Curly,
                    "4:4" => UnderlineStyle::Dotted,
                    _ => UnderlineStyle::Dashed,
                };
            }
            "38" => {
                st.fg = parse_color(&toks, &mut i)?;
                continue;
            }
            "48" => {
                st.bg = parse_color(&toks, &mut i)?;
                continue;
            }
            "58" => {
                st.ul = parse_color(&toks, &mut i)?;
                continue;
            }
            _ => return None,
        }
        i += 1;
    }
    Some(st)
}

/// Inverse of [`encode`] (review artifacts only, never a gate input).
/// Grid width is recovered from row 0's cell widths. `None` when the text is
/// not a well-formed encoded frame.
pub fn decode(text: &str, provenance: Provenance) -> Option<Frame> {
    let (body, trailer) = match text.rfind("\x1b[?25") {
        Some(i) => (&text[..i], &text[i..]),
        None => (text, ""),
    };
    let mut rows: Vec<Vec<Cell>> = Vec::new();
    for line in body.split_terminator('\n') {
        let y = u16::try_from(rows.len()).ok()?;
        let mut cells: Vec<Cell> = Vec::new();
        let mut st = Style::default();
        let mut rest = line;
        while !rest.is_empty() {
            if let Some(after) = rest.strip_prefix("\x1b[") {
                let m = after.find('m')?;
                let p = &after[..m];
                st = if p == "0" {
                    Style::default()
                } else {
                    parse_sgr(p)?
                };
                rest = &after[m + 1..];
                continue;
            }
            let end = rest.find('\x1b').unwrap_or(rest.len());
            for g in rest[..end].graphemes(true) {
                let wide = unicode_width::UnicodeWidthStr::width(g) == 2;
                let lead = Cell {
                    x: u16::try_from(cells.len()).ok()?,
                    y,
                    symbol: g.to_string(),
                    width: if wide { 2 } else { 1 },
                    continuation: false,
                    fg: st.fg,
                    bg: st.bg,
                    mods: st.mods,
                    underline_color: st.ul,
                };
                let cont = Cell {
                    x: lead.x + 1,
                    symbol: String::new(),
                    width: 0,
                    continuation: true,
                    ..lead.clone()
                };
                cells.push(lead);
                if wide {
                    cells.push(cont);
                }
            }
            rest = &rest[end..];
        }
        rows.push(cells);
    }
    let cols = u16::try_from(rows.first()?.len()).ok()?;
    let nrows = u16::try_from(rows.len()).ok()?;
    let mut cursor = Cursor::default();
    if let Some(t) = trailer.strip_prefix("\x1b[?25h\x1b[") {
        let h = t.find('H')?;
        let (y, x) = t[..h].split_once(';')?;
        let ps: u8 = t.get(h + 3..t.len().checked_sub(2)?)?.parse().ok()?;
        cursor = Cursor {
            x: x.parse::<u16>().ok()?.checked_sub(1)?,
            y: y.parse::<u16>().ok()?.checked_sub(1)?,
            visible: true,
            style: match ps {
                1 | 2 => CursorStyle::Block,
                3 | 4 => CursorStyle::Underline,
                _ => CursorStyle::Bar,
            },
            blinking: ps % 2 == 1,
        };
    }
    Some(Frame {
        version: FRAME_VERSION,
        cols,
        rows: nrows,
        cells: rows.into_iter().flatten().collect(),
        cursor,
        provenance,
    })
}

#[test]
fn ansi_cursor_round_trips_cells_and_visible_cursor() {
    let provenance = Provenance {
        tool: "tuiscotti".into(),
        tool_version: "test".into(),
        profile: "tuiscotti-default".into(),
        source: "screen".into(),
        argv: Vec::new(),
        created_unix: 0,
    };
    let mut frame = decode(
        "\x1b[1;38;5;3mab\x1b[0m界\nxy  \x1b[?25l",
        provenance.clone(),
    )
    .expect("decode fixture");
    assert_eq!((frame.cols, frame.rows), (4, 2));
    frame.cursor = Cursor {
        x: 2,
        y: 1,
        visible: true,
        style: CursorStyle::Bar,
        blinking: true,
    };
    let encoded = encode(&frame);
    assert!(
        encoded.ends_with("\x1b[?25h\x1b[2;3H\x1b[5 q"),
        "{encoded:?}"
    );
    let back = decode(&encoded, provenance).expect("decode encoded");
    assert_eq!(back.cells, frame.cells);
    assert_eq!(back.cursor, frame.cursor);
    assert_eq!(encode(&back), encoded);
}
