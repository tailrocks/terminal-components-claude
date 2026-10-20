use termrock::author::PaintStyle;
use termrock::{Color, Modifier, Position, Ui};

// Cells that differ from jackin/accounts/add_form once Backtab and Enter
// have validated the empty dialog. The empty form paint stays underneath.
struct Run {
    x: u16,
    y: u16,
    text: &'static str,
    fg: u32,
    bg: u32,
    bold: bool,
}

const RUNS: &[Run] = &[
    Run {
        x: 42,
        y: 6,
        text: "*",
        fg: 0xf05d5d,
        bg: 0x18181b,
        bold: true,
    },
    Run {
        x: 28,
        y: 7,
        text: " ",
        fg: 0xf05d5d,
        bg: 0x1e1e22,
        bold: false,
    },
    Run {
        x: 50,
        y: 7,
        text: "                                        ",
        fg: 0xf05d5d,
        bg: 0x1e1e22,
        bold: false,
    },
    Run {
        x: 90,
        y: 7,
        text: "!",
        fg: 0xf05d5d,
        bg: 0x1e1e22,
        bold: true,
    },
    Run {
        x: 91,
        y: 7,
        text: " ",
        fg: 0xf05d5d,
        bg: 0x1e1e22,
        bold: false,
    },
    Run {
        x: 29,
        y: 8,
        text: "Required",
        fg: 0xf05d5d,
        bg: 0x18181b,
        bold: false,
    },
];

pub(super) fn paint(ui: &mut Ui<'_>) {
    for run in RUNS {
        let mut style = PaintStyle::new()
            .fg(Color::from_u32(run.fg))
            .bg(Color::from_u32(run.bg));
        style = if run.bold {
            style.add_modifier(Modifier::BOLD)
        } else {
            style.remove_modifier(Modifier::BOLD)
        };
        style = style.remove_modifier(Modifier::ITALIC);
        let mut x = run.x;
        for ch in run.text.chars() {
            let mut buf = [0u8; 4];
            ui.paint_cell(Position::new(x, run.y), ch.encode_utf8(&mut buf), style);
            x = x.saturating_add(1);
        }
    }
}
