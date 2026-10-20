use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier};

use crate::core::scroll::ScrollState;
use crate::ui::ctx::RenderCtx;

pub const OUTER_KEEP: f32 = 0.55;

pub const INNER_KEEP: f32 = 0.8;

pub const DEEP_FROM: u16 = 12;

pub const MIN_ROWS: u16 = 4;

pub fn scroll_edges(buf: &mut Buffer, ctx: &RenderCtx, area: Rect, scroll: &ScrollState) {
    scroll_edges_except(buf, ctx, area, scroll, &[]);
}

pub fn scroll_edges_except(
    buf: &mut Buffer,
    ctx: &RenderCtx,
    area: Rect,
    scroll: &ScrollState,
    keep: &[u16],
) {
    let area = area.intersection(*buf.area());
    if area.is_empty() || area.height < MIN_ROWS {
        return;
    }
    let up = scroll.offset > 0;
    let down = scroll.viewport_len > 0 && scroll.offset + scroll.viewport_len < scroll.content_len;
    if !up && !down {
        return;
    }
    let depth: u16 = if area.height >= DEEP_FROM { 2 } else { 1 };
    let container = majority_bg(buf, area);
    let cursor_row = ctx.cursor.filter(|p| area.contains(*p)).map(|p| p.y);
    let mut rows: Vec<(u16, f32)> = vec![];
    if up {
        rows.push((area.y, OUTER_KEEP));
        if depth == 2 {
            rows.push((area.y + 1, INNER_KEEP));
        }
    }
    if down {
        rows.push((area.bottom() - 1, OUTER_KEEP));
        if depth == 2 {
            rows.push((area.bottom() - 2, INNER_KEEP));
        }
    }
    for (y, strength) in rows {
        if Some(y) == cursor_row || keep.contains(&y) {
            continue;
        }
        fade_row(buf, area, y, strength, container);
    }
}

fn majority_bg(buf: &Buffer, area: Rect) -> Color {
    let mut counts: Vec<(Color, usize)> = vec![];
    for pos in area.positions() {
        let bg = buf[pos].bg;
        match counts.iter_mut().find(|(c, _)| *c == bg) {
            Some((_, n)) => *n += 1,
            None => counts.push((bg, 1)),
        }
    }
    counts
        .into_iter()
        .max_by_key(|(_, n)| *n)
        .map(|(c, _)| c)
        .unwrap_or(Color::Reset)
}

fn fade_row(buf: &mut Buffer, area: Rect, y: u16, keep: f32, container: Color) {
    let outer = keep <= OUTER_KEEP;
    for x in area.x..area.right() {
        let Some(cell) = buf.cell_mut((x, y)) else {
            continue;
        };
        if cell.bg != container || cell.modifier.contains(Modifier::REVERSED) {
            continue;
        }
        match (cell.fg, container) {
            (Color::Rgb(fr, fg, fb), Color::Rgb(br, bgg, bb)) => {
                cell.fg = Color::Rgb(mix(fr, br, keep), mix(fg, bgg, keep), mix(fb, bb, keep));
            }

            _ if outer => {
                cell.modifier |= Modifier::DIM;
            }
            _ => {}
        }
    }
}

fn mix(fg: u8, bg: u8, keep: f32) -> u8 {
    (bg as f32 + (fg as f32 - bg as f32) * keep)
        .round()
        .clamp(0.0, 255.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::theme::{ColorLevel, Theme};
    use crate::ui::ctx::Interaction;
    use ratatui::layout::Position;
    use ratatui::style::Style;

    const BG: Color = Color::Rgb(0, 0, 0);
    const FG: Color = Color::Rgb(200, 200, 200);

    fn painted(rows: u16) -> Buffer {
        let mut buf = Buffer::empty(Rect::new(0, 0, 10, rows));
        for y in 0..rows {
            buf.set_string(0, y, "abcdefghij", Style::new().fg(FG).bg(BG));
        }
        buf
    }

    fn fade(buf: &mut Buffer, level: ColorLevel, scroll: ScrollState, cursor: Option<Position>) {
        let theme = Theme::for_level(level);
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        if let Some(c) = cursor {
            ctx.set_cursor(c);
        }
        let area = *buf.area();
        scroll_edges(buf, &ctx, area, &scroll);
    }

    fn grey(buf: &Buffer, y: u16) -> u8 {
        match buf[(0, y)].fg {
            Color::Rgb(r, _, _) => r,
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn fades_only_the_edges_that_hide_more_content_and_clears_at_the_boundary() {
        let s = |offset: usize| ScrollState {
            offset,
            content_len: 30,
            viewport_len: 6,
            ..Default::default()
        };

        let mut b = painted(6);
        fade(&mut b, ColorLevel::TrueColor, s(0), None);
        assert_eq!(grey(&b, 0), 200);
        assert_eq!(grey(&b, 5), 110, "outer row keeps 55%");
        assert_eq!(grey(&b, 4), 200, "one row only below twelve rows");

        let mut b = painted(6);
        fade(&mut b, ColorLevel::TrueColor, s(10), None);
        assert_eq!(grey(&b, 0), 110);
        assert_eq!(grey(&b, 5), 110);

        let mut b = painted(6);
        fade(&mut b, ColorLevel::TrueColor, s(24), None);
        assert_eq!(grey(&b, 0), 110);
        assert_eq!(grey(&b, 5), 200);

        let mut b = painted(6);
        fade(
            &mut b,
            ColorLevel::TrueColor,
            ScrollState {
                offset: 0,
                content_len: 6,
                viewport_len: 6,
                ..Default::default()
            },
            None,
        );
        assert_eq!(grey(&b, 0), 200);
        assert_eq!(grey(&b, 5), 200);
    }

    #[test]
    fn tall_viewports_fade_two_rows_and_short_ones_none() {
        let mut b = painted(12);
        fade(
            &mut b,
            ColorLevel::TrueColor,
            ScrollState {
                offset: 5,
                content_len: 100,
                viewport_len: 12,
                ..Default::default()
            },
            None,
        );
        assert_eq!(grey(&b, 0), 110);
        assert_eq!(grey(&b, 1), 160, "inner row keeps 80%");
        assert_eq!(grey(&b, 2), 200);
        assert_eq!(grey(&b, 11), 110);
        assert_eq!(grey(&b, 10), 160);
        let mut b = painted(3);
        fade(
            &mut b,
            ColorLevel::TrueColor,
            ScrollState {
                offset: 5,
                content_len: 100,
                viewport_len: 3,
                ..Default::default()
            },
            None,
        );
        assert_eq!(grey(&b, 0), 200, "three rows are all content");
    }

    #[test]
    fn emphasised_cells_and_the_cursor_row_are_never_faded() {
        let mut b = painted(6);

        b.set_string(
            0,
            0,
            "abcdefghij",
            Style::new().fg(FG).bg(Color::Rgb(20, 60, 20)),
        );
        b.set_string(
            0,
            5,
            "ab",
            Style::new().fg(FG).bg(BG).add_modifier(Modifier::REVERSED),
        );
        b.set_string(2, 5, "c", Style::new().fg(FG).bg(Color::Rgb(40, 40, 0)));
        fade(
            &mut b,
            ColorLevel::TrueColor,
            ScrollState {
                offset: 10,
                content_len: 30,
                viewport_len: 6,
                ..Default::default()
            },
            None,
        );
        assert_eq!(grey(&b, 0), 200, "selected row untouched");
        assert_eq!(grey(&b, 5), 200, "reversed cell untouched");
        assert!(
            matches!(b[(2, 5)].fg, Color::Rgb(200, ..)),
            "marked cell untouched"
        );
        assert!(
            matches!(b[(3, 5)].fg, Color::Rgb(110, ..)),
            "the plain neighbour fades"
        );

        let mut b = painted(6);
        fade(
            &mut b,
            ColorLevel::TrueColor,
            ScrollState {
                offset: 10,
                content_len: 30,
                viewport_len: 6,
                ..Default::default()
            },
            Some(Position::new(4, 5)),
        );
        assert_eq!(grey(&b, 5), 200);
        assert_eq!(grey(&b, 0), 110);
    }

    #[test]
    fn protected_rows_stay_whole_on_the_container_plane() {
        let mut b = painted(6);
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        let area = *b.area();
        scroll_edges_except(
            &mut b,
            &ctx,
            area,
            &ScrollState {
                offset: 10,
                content_len: 30,
                viewport_len: 6,
                ..Default::default()
            },
            &[5],
        );
        assert_eq!(grey(&b, 0), 110);
        assert_eq!(grey(&b, 5), 200, "the kept row is untouched");
    }

    #[test]
    fn palettes_without_rgb_dim_the_outer_row_only() {
        for level in [ColorLevel::Ansi256, ColorLevel::Ansi16, ColorLevel::Mono] {
            let mut b = Buffer::empty(Rect::new(0, 0, 10, 12));
            for y in 0..12 {
                b.set_string(
                    0,
                    y,
                    "abcdefghij",
                    Style::new().fg(Color::Gray).bg(Color::Black),
                );
            }
            fade(
                &mut b,
                level,
                ScrollState {
                    offset: 5,
                    content_len: 100,
                    viewport_len: 12,
                    ..Default::default()
                },
                None,
            );
            assert!(b[(0, 0)].modifier.contains(Modifier::DIM), "{level:?}");
            assert!(
                !b[(0, 1)].modifier.contains(Modifier::DIM),
                "{level:?}: inner row stays"
            );
            assert!(b[(0, 11)].modifier.contains(Modifier::DIM), "{level:?}");
            assert_eq!(b[(0, 0)].fg, Color::Gray, "{level:?}: no colour invented");
        }
    }
}
