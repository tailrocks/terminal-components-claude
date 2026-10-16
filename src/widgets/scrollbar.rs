use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::Style;

use crate::core::id::WidgetId;
use crate::core::scroll::{ScrollState, TrackClick};
use crate::theme::{Glyphs, Kind, Plane};
use crate::ui::ctx::{RenderCtx, VisualState};

// The default glyphs. Scrollbars draw `Theme::glyphs`
// (`scroll_track`, `scroll_thumb`, `scroll_h_track`, `scroll_h_thumb`);
// these constants name the `Glyphs::UNICODE` set. The track is a dotted
// rail (`┆` `┄`) in `Theme::scroll_track` ink, so it never reads as a
// panel border or divider (`│` `─`). The thumb keeps its glyph in every
// state: rest, hover and held differ in colour only (`Ladders::handle`),
// plus a modifier in monochrome.
pub const TRACK: &str = Glyphs::UNICODE.scroll_track;
pub const THUMB: &str = Glyphs::UNICODE.scroll_thumb;
/// Horizontal track and thumb: the vertical glyphs turned on their side.
pub const H_TRACK: &str = Glyphs::UNICODE.scroll_h_track;
pub const H_THUMB: &str = Glyphs::UNICODE.scroll_h_thumb;

/// Id used for a container's scrollbar hit region.
pub fn id_for(container: WidgetId) -> WidgetId {
    container.sub("scrollbar")
}

/// Id used for a container's horizontal scrollbar hit region. Separate
/// from [`id_for`] so hovering or holding one bar never lights the other.
pub fn h_id_for(container: WidgetId) -> WidgetId {
    container.sub("hscrollbar")
}

/// Either of `container`'s scrollbars.
pub fn is_bar_of(id: WidgetId, container: WidgetId) -> bool {
    id == id_for(container) || id == h_id_for(container)
}

/// The thumb's style for the bar `sb_id`: rest (neutral), hover (lighter)
/// and pressed (the accent) are distinct colours of one glyph. Container
/// focus does not change it: the `▎` is the only focus indicator.
fn thumb_style(ctx: &RenderCtx, sb_id: WidgetId) -> Style {
    let s = VisualState {
        hovered: ctx.interaction.hovered(sb_id),
        pressed: ctx.interaction.pressed == Some(sb_id),
        ..Default::default()
    };
    ctx.theme.interactive(Kind::Handle, s, Plane::Canvas).style
}

/// A 1-column divider that is also the container's scrollbar. While the
/// content overflows it is a scrollbar: the dotted track, the thumb and
/// its click region, exactly as [`render_vertical`]. While the content
/// fits it is a plain divider: the solid [`Glyphs::divider`] in the
/// border ink, with no click region. The pane edge is there at every
/// size, and only a line that scrolls looks like a scrollbar.
pub fn render_divider(
    area: Rect,
    buf: &mut Buffer,
    ctx: &mut RenderCtx,
    container: WidgetId,
    scroll: &ScrollState,
    focused: bool,
) {
    let area = area.intersection(*buf.area());
    if area.is_empty() {
        return;
    }
    if scroll.overflows() {
        render_vertical(area, buf, ctx, container, scroll, focused);
        return;
    }
    let line = ctx.theme.border(false);
    let g = ctx.theme.glyphs;
    for y in area.top()..area.bottom() {
        buf.set_string(area.x, y, g.divider, line);
    }
}

/// Render a vertical scrollbar into a 1-column area. Draws nothing when the
/// content fits. Registers a clickable region for track clicks and drags.
/// `_focused` (the container's focus) does not change the bar: the thumb
/// shows rest, hover and pressed only.
pub fn render_vertical(
    area: Rect,
    buf: &mut Buffer,
    ctx: &mut RenderCtx,
    container: WidgetId,
    scroll: &ScrollState,
    _focused: bool,
) {
    let area = area.intersection(*buf.area());
    if area.is_empty() || !scroll.overflows() {
        return;
    }
    let sb_id = id_for(container);
    let thumb = thumb_style(ctx, sb_id);
    let track = ctx.theme.scrollbar_track();
    let g = ctx.theme.glyphs;
    let track_len = area.height as usize;
    let (start, len) = scroll.thumb(track_len);
    for i in 0..track_len {
        let y = area.y + i as u16;
        if i >= start && i < start + len {
            buf.set_string(area.x, y, g.scroll_thumb, thumb);
        } else {
            buf.set_string(area.x, y, g.scroll_track, track);
        }
    }
    ctx.scrollbar(sb_id, area, container, crate::core::hit::Axis::Vertical);
}

/// Render a horizontal scrollbar into a 1-row area: the mirror of
/// [`render_vertical`]. Draws nothing when the content fits. Registers a
/// clickable region under [`h_id_for`] for track clicks and drags.
pub fn render_horizontal(
    area: Rect,
    buf: &mut Buffer,
    ctx: &mut RenderCtx,
    container: WidgetId,
    scroll: &ScrollState,
    focused: bool,
) {
    render_horizontal_with(area, buf, ctx, container, scroll, focused, false);
}

/// [`render_horizontal`] that, with `always` (the
/// [`ScrollbarVisibility::Always`](crate::core::scroll::ScrollbarVisibility)
/// policy), also draws a full-length thumb when the content fits.
pub fn render_horizontal_with(
    area: Rect,
    buf: &mut Buffer,
    ctx: &mut RenderCtx,
    container: WidgetId,
    scroll: &ScrollState,
    _focused: bool,
    always: bool,
) {
    let area = area.intersection(*buf.area());
    if area.is_empty() || !(scroll.overflows() || always) {
        return;
    }
    let sb_id = h_id_for(container);
    let thumb = thumb_style(ctx, sb_id);
    let track = ctx.theme.scrollbar_track();
    let g = ctx.theme.glyphs;
    let track_len = area.width as usize;
    let (start, len) = scroll.thumb(track_len);
    for i in 0..track_len {
        let x = area.x + i as u16;
        if i >= start && i < start + len {
            buf.set_string(x, area.y, g.scroll_h_thumb, thumb);
        } else {
            buf.set_string(x, area.y, g.scroll_h_track, track);
        }
    }
    if scroll.overflows() {
        ctx.scrollbar(sb_id, area, container, crate::core::hit::Axis::Horizontal);
    }
}

/// Where a scrolled region's content and bars go. The vertical bar takes
/// the right column, the horizontal bar the bottom row; with both, the
/// horizontal track stops short of the corner cell, which stays blank.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bars {
    pub content: Rect,
    pub vertical: Option<Rect>,
    pub horizontal: Option<Rect>,
}

impl Bars {
    pub fn split(area: Rect, vertical: bool, horizontal: bool) -> Self {
        let v_w = u16::from(vertical && area.width > 1);
        let h_h = u16::from(horizontal && area.height > 1);
        let content = Rect::new(area.x, area.y, area.width - v_w, area.height - h_h);
        Self {
            content,
            vertical: (v_w == 1).then(|| Rect::new(area.right() - 1, area.y, 1, content.height)),
            horizontal: (h_h == 1).then(|| Rect::new(area.x, area.bottom() - 1, content.width, 1)),
        }
    }
}

/// Blank columns between scrolled content and its vertical bar, the
/// default for every widget's `scrollbar_gap`. One cell keeps text from
/// touching the track (`TypeScript┃`) without costing a narrow pane much.
pub const GAP: u16 = 1;

/// Columns a vertical bar takes from its content: the track and the gap.
pub const fn reserve(gap: u16) -> u16 {
    1 + gap
}

/// A region with a vertical bar on its right, split once so every widget
/// keeps the same distance between its text and the bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VSplit {
    /// Everything left of the track: row fills and cursor highlights
    /// reach up to the bar.
    pub body: Rect,
    /// Where text goes: the body less the gap. Equal to `body` when there
    /// is no bar.
    pub content: Rect,
    /// The bar's column, when the content overflows.
    pub track: Option<Rect>,
}

/// Split `area` for a vertical bar when `overflows`, leaving `gap` blank
/// columns between the content and the track. Too narrow to spare the gap,
/// the gap goes first and the track last.
pub fn split(area: Rect, overflows: bool, gap: u16) -> VSplit {
    if !overflows || area.width < 2 {
        return VSplit {
            body: area,
            content: area,
            track: None,
        };
    }
    let body = Rect::new(area.x, area.y, area.width - 1, area.height);
    let gap = gap.min(body.width.saturating_sub(1));
    VSplit {
        body,
        content: Rect::new(area.x, area.y, body.width - gap, area.height),
        track: Some(Rect::new(area.right() - 1, area.y, 1, area.height)),
    }
}

/// Map a pointer position on a vertical track to a scroll offset.
pub fn offset_for_click(track: Rect, pos: Position, scroll: &ScrollState) -> usize {
    let rel = pos.y.saturating_sub(track.y) as usize;
    scroll.offset_for_track_pos(rel, track.height as usize)
}

/// Row of `track` under `pos`, clamped to the track (a pointer above or
/// below the track still drags to the ends).
fn track_row(track: Rect, pos: Position) -> usize {
    pos.y
        .saturating_sub(track.y)
        .min(track.height.saturating_sub(1)) as usize
}

/// Column of a horizontal `track` under `pos`, clamped to the track.
fn track_col(track: Rect, pos: Position) -> usize {
    pos.x
        .saturating_sub(track.x)
        .min(track.width.saturating_sub(1)) as usize
}

/// The pointer went down on `track`: on the thumb this grabs it, on the
/// bare track it acts per the default [`TrackClick`] (one page toward the
/// pointer). Returns whether the offset changed. `track` must be the
/// rectangle the scrollbar was drawn in.
pub fn press(track: Rect, pos: Position, scroll: &mut ScrollState) -> bool {
    press_with(track, pos, scroll, TrackClick::default())
}

/// [`press`] under a track-click policy: with [`TrackClick::Page`] a
/// press on the bare track moves one page toward the pointer instead of
/// jumping the thumb under it (the macOS "next page" behaviour).
pub fn press_with(track: Rect, pos: Position, scroll: &mut ScrollState, click: TrackClick) -> bool {
    scroll.press_track_with(track_row(track, pos), track.height as usize, click)
}

/// The pointer moved while held after [`press`]: the thumb follows,
/// keeping the grabbed row under the pointer. Returns whether the offset
/// changed.
pub fn drag(track: Rect, pos: Position, scroll: &mut ScrollState) -> bool {
    scroll.drag_track(track_row(track, pos), track.height as usize)
}

/// [`press`] for a horizontal `track`; `click` decides what a press on the
/// bare track does.
pub fn press_h(track: Rect, pos: Position, scroll: &mut ScrollState, click: TrackClick) -> bool {
    scroll.press_track_with(track_col(track, pos), track.width as usize, click)
}

/// [`drag`] for a horizontal `track`.
pub fn drag_h(track: Rect, pos: Position, scroll: &mut ScrollState) -> bool {
    scroll.drag_track(track_col(track, pos), track.width as usize)
}

/// A pointer press or drag on a [`Bar`](crate::core::hit::Bar), routed by
/// the pointer layer: `scroll` is the owner's state on the bar's axis.
/// Returns whether the offset changed.
pub fn apply(
    bar: &crate::core::hit::Bar,
    pos: Position,
    pressed: bool,
    scroll: &mut ScrollState,
) -> bool {
    apply_with(bar, pos, pressed, scroll, TrackClick::default())
}

/// [`apply`] under a track-click policy (`click` decides what a press on
/// the bare track does, on either axis).
pub fn apply_with(
    bar: &crate::core::hit::Bar,
    pos: Position,
    pressed: bool,
    scroll: &mut ScrollState,
    click: TrackClick,
) -> bool {
    use crate::core::hit::Axis;
    match (bar.axis, pressed) {
        (Axis::Vertical, true) => press_with(bar.track, pos, scroll, click),
        (Axis::Vertical, false) => drag(bar.track, pos, scroll),
        (Axis::Horizontal, true) => press_h(bar.track, pos, scroll, click),
        (Axis::Horizontal, false) => drag_h(bar.track, pos, scroll),
    }
}

/// "12–24 of 120" style position label.
pub fn position_label(scroll: &ScrollState) -> String {
    if !scroll.overflows() {
        return String::new();
    }
    let r = scroll.visible_range();
    format!("{}–{} of {}", r.start + 1, r.end, scroll.content_len)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{focus::FocusRing, hit::HitRegistry};
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;

    fn draw(scroll: &ScrollState, interaction: Interaction, always: bool) -> (Buffer, HitRegistry) {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 1));
        {
            let mut ctx = RenderCtx::new(&theme, interaction, &mut hits, &mut ring);
            render_horizontal_with(
                Rect::new(0, 0, 10, 1),
                &mut buf,
                &mut ctx,
                WidgetId::of("c"),
                scroll,
                false,
                always,
            );
        }
        (buf, hits)
    }

    fn row(buf: &Buffer) -> String {
        (0..buf.area.width)
            .map(|x| buf[(x, 0)].symbol().to_string())
            .collect()
    }

    fn hover_on(id: WidgetId) -> Interaction {
        Interaction {
            hover: Some(id),
            ..Default::default()
        }
    }

    #[test]
    fn a_horizontal_bar_draws_a_proportional_thumb_only_on_overflow() {
        let mut s = ScrollState::new(40);
        s.set_viewport(10);
        let (buf, hits) = draw(&s, Interaction::default(), false);
        assert_eq!(row(&buf), "━━┄┄┄┄┄┄┄┄");
        assert_eq!(
            hits.hit(Position::new(5, 0)),
            Some(h_id_for(WidgetId::of("c"))),
            "the track is a hit region under the horizontal id"
        );
        s.scroll_to(30);
        let (buf, _) = draw(&s, Interaction::default(), false);
        assert_eq!(row(&buf), "┄┄┄┄┄┄┄┄━━", "at the end the thumb sits right");

        s.set_viewport(40);
        let (buf, hits) = draw(&s, Interaction::default(), false);
        assert_eq!(row(&buf).trim(), "", "content fits: nothing drawn");
        assert!(hits.is_empty());
        let (buf, hits) = draw(&s, Interaction::default(), true);
        assert_eq!(row(&buf), "━━━━━━━━━━", "always: a full thumb");
        assert!(hits.is_empty(), "nothing to scroll, nothing to press");
    }

    #[test]
    fn rest_hover_and_pressed_thumbs_are_distinct() {
        let mut s = ScrollState::new(40);
        s.set_viewport(10);
        let id = h_id_for(WidgetId::of("c"));
        let style = |i: Interaction| {
            let c = draw(&s, i, false).0[(0, 0)].clone();
            (c.fg, c.modifier)
        };
        let rest = style(Interaction::default());
        let hover = style(hover_on(id));
        let pressed = style(Interaction {
            hover: Some(id),
            pressed: Some(id),
            ..Default::default()
        });
        assert_ne!(rest, hover);
        assert_ne!(hover, pressed);
        assert_ne!(rest, pressed);
        assert_eq!(
            style(hover_on(id_for(WidgetId::of("c")))),
            rest,
            "hovering the vertical bar does not light the horizontal thumb"
        );
    }

    /// Rest, hover and pressed thumbs on both axes at `level`: (symbol,
    /// fg, modifiers) of the first thumb cell.
    fn thumb_cells(
        level: crate::theme::ColorLevel,
    ) -> Vec<
        [(
            String,
            Option<ratatui::style::Color>,
            ratatui::style::Modifier,
        ); 3],
    > {
        let theme = Theme::for_level(level);
        let mut s = ScrollState::new(40);
        s.set_viewport(10);
        let c = WidgetId::of("c");
        let mut out = vec![];
        for vertical in [true, false] {
            let id = if vertical { id_for(c) } else { h_id_for(c) };
            let states = [
                Interaction::default(),
                hover_on(id),
                Interaction {
                    hover: Some(id),
                    pressed: Some(id),
                    ..Default::default()
                },
            ];
            let cells = states.map(|i| {
                let mut hits = HitRegistry::default();
                let mut ring = FocusRing::default();
                let area = if vertical {
                    Rect::new(0, 0, 1, 10)
                } else {
                    Rect::new(0, 0, 10, 1)
                };
                let mut buf = Buffer::empty(area);
                {
                    let mut ctx = RenderCtx::new(&theme, i, &mut hits, &mut ring);
                    // a focused container: focus must not change the thumb
                    if vertical {
                        render_vertical(area, &mut buf, &mut ctx, c, &s, true);
                    } else {
                        render_horizontal(area, &mut buf, &mut ctx, c, &s, true);
                    }
                }
                let cell = &buf[(0, 0)];
                (cell.symbol().to_owned(), Some(cell.fg), cell.modifier)
            });
            out.push(cells);
        }
        out
    }

    #[test]
    fn the_thumb_keeps_one_glyph_and_colour_carries_rest_hover_and_pressed() {
        use crate::theme::ColorLevel;
        for level in ColorLevel::ALL {
            let t = Theme::for_level(level);
            for (axis, [rest, hover, pressed]) in thumb_cells(level).into_iter().enumerate() {
                let glyph = if axis == 0 {
                    t.glyphs.scroll_thumb
                } else {
                    t.glyphs.scroll_h_thumb
                };
                let ctx = format!("{level:?} axis {axis}");
                for (name, cell) in [("rest", &rest), ("hover", &hover), ("pressed", &pressed)] {
                    assert_eq!(cell.0, glyph, "{name} keeps the thumb glyph: {ctx}");
                }
                if level == ColorLevel::Mono {
                    // colour cannot carry state: a modifier does
                    assert_ne!(rest.2, hover.2, "{ctx}");
                    assert_ne!(hover.2, pressed.2, "{ctx}");
                    assert_ne!(rest.2, pressed.2, "{ctx}");
                } else {
                    assert_eq!(rest.1, Some(t.ladders.handle.rest), "{ctx}");
                    assert_eq!(hover.1, Some(t.ladders.handle.hover), "{ctx}");
                    assert_eq!(pressed.1, Some(t.accent), "pressed is the accent: {ctx}");
                    assert!(
                        rest.1 != hover.1 && hover.1 != pressed.1 && rest.1 != pressed.1,
                        "{ctx}"
                    );
                    assert_eq!(
                        (rest.2, hover.2, pressed.2),
                        (Default::default(), Default::default(), Default::default()),
                        "colour alone carries state: {ctx}"
                    );
                }
            }
        }
    }

    #[test]
    fn pressing_and_dragging_a_horizontal_track_moves_the_view() {
        let track = Rect::new(4, 9, 10, 1);
        let mut s = ScrollState::new(40);
        s.set_viewport(10);
        assert!(press_h(
            track,
            Position::new(13, 9),
            &mut s,
            TrackClick::Jump
        ));
        assert_eq!(
            s.offset,
            s.max_offset(),
            "a press at the right end jumps there"
        );
        s.release_track();
        assert!(press_h(
            track,
            Position::new(4, 9),
            &mut s,
            TrackClick::Page
        ));
        assert_eq!(s.offset, 20, "paging moves one viewport left");
        let (start, _) = s.thumb(10);
        let on_thumb = Position::new(4 + start as u16, 9);
        assert!(
            !press_h(track, on_thumb, &mut s, TrackClick::Jump),
            "grab only"
        );
        assert!(
            drag_h(track, Position::new(0, 9), &mut s),
            "dragged past the left end"
        );
        assert_eq!(s.offset, 0);
    }

    /// A cell's ink: foreground and modifiers.
    fn ink(s: Style) -> (Option<ratatui::style::Color>, ratatui::style::Modifier) {
        (s.fg, s.add_modifier)
    }

    /// A grey's lightness on the level's own scale: the RGB channel, the
    /// 256-colour grey ramp, the 16-colour greys.
    fn grey(c: ratatui::style::Color) -> u16 {
        use ratatui::style::Color::*;
        match c {
            Rgb(r, g, b) => {
                assert!(r == g && g == b, "{c:?} is a grey");
                u16::from(r)
            }
            Indexed(i @ 232..=255) => u16::from(i - 232) * 10 + 8,
            Indexed(16) | Black => 0,
            DarkGray => 85,
            Gray => 170,
            White => 255,
            other => panic!("not a grey: {other:?}"),
        }
    }

    /// A vertical and a horizontal bar over overflowing content at
    /// `theme`: every (symbol, style) cell of each, track first.
    fn both_bars(theme: &Theme) -> [Vec<(String, Style)>; 2] {
        let mut s = ScrollState::new(40);
        s.set_viewport(10);
        let c = WidgetId::of("c");
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut buf = Buffer::empty(Rect::new(0, 0, 11, 11));
        {
            let mut ctx = RenderCtx::new(theme, Interaction::default(), &mut hits, &mut ring);
            let bars = Bars::split(Rect::new(0, 0, 11, 11), true, true);
            render_vertical(bars.vertical.unwrap(), &mut buf, &mut ctx, c, &s, false);
            render_horizontal(bars.horizontal.unwrap(), &mut buf, &mut ctx, c, &s, false);
        }
        let cell = |x, y| {
            let c: &ratatui::buffer::Cell = &buf[(x, y)];
            (c.symbol().to_owned(), c.style())
        };
        assert_eq!(cell(10, 10).0, " ", "the corner cell stays blank");
        [
            (0..10).map(|y| cell(10, y)).collect(),
            (0..10).map(|x| cell(x, 10)).collect(),
        ]
    }

    #[test]
    fn both_tracks_are_one_dotted_pair_in_the_track_ink_never_a_border() {
        use crate::theme::{ColorLevel, Glyphs};
        for glyphs in [Glyphs::UNICODE, Glyphs::ASCII] {
            for level in ColorLevel::ALL {
                let mut t = Theme::for_level(level);
                t.glyphs = glyphs;
                let track = t.scrollbar_track();
                let rest = t.scrollbar_thumb(false, false);
                let border = t.glyphs.border;
                for (axis, cells) in both_bars(&t).into_iter().enumerate() {
                    let ctx = format!("{level:?} axis {axis} {:?}", glyphs.scroll_track);
                    let (track_glyph, thumb_glyph, line, edge) = if axis == 0 {
                        (
                            t.glyphs.scroll_track,
                            t.glyphs.scroll_thumb,
                            t.glyphs.divider,
                            border.vertical_left,
                        )
                    } else {
                        (
                            t.glyphs.scroll_h_track,
                            t.glyphs.scroll_h_thumb,
                            t.glyphs.h_divider,
                            border.horizontal_top,
                        )
                    };
                    let tracks: Vec<_> = cells.iter().filter(|c| c.0 != thumb_glyph).collect();
                    assert!(tracks.len() >= 5, "{ctx}: {cells:?}");
                    for (sym, style) in tracks {
                        assert_eq!(sym, track_glyph, "{ctx}");
                        assert_eq!(ink(*style), ink(track), "{ctx}: the track style");
                    }
                    // the shape alone tells a track from a border or a
                    // divider (WCAG 1.4.1), so it holds in monochrome too
                    assert_ne!(track_glyph, line, "{ctx}");
                    assert_ne!(track_glyph, edge, "{ctx}");
                    assert_ne!(track_glyph, thumb_glyph, "{ctx}");
                    let thumb = cells.iter().find(|c| c.0 == thumb_glyph).unwrap();
                    assert_eq!(
                        ink(thumb.1),
                        ink(rest),
                        "{ctx}: the thumb at rest is unchanged"
                    );
                    if level != ColorLevel::Mono {
                        let (tk, th) = (grey(t.scroll_track), grey(t.ladders.handle.rest));
                        assert!(tk < th, "{ctx}: the track is fainter than the thumb");
                        assert!(
                            grey(t.canvas) < tk,
                            "{ctx}: the track is visible on the canvas"
                        );
                    }
                    if matches!(level, ColorLevel::TrueColor | ColorLevel::Ansi256) {
                        assert_ne!(t.scroll_track, t.border_subtle, "{ctx}");
                        assert_ne!(t.scroll_track, t.border_strong, "{ctx}");
                    }
                }
            }
        }
    }

    #[test]
    fn the_glyph_sets_pair_their_tracks_and_thumbs() {
        use crate::theme::Glyphs;
        use unicode_width::UnicodeWidthStr;
        let u = Glyphs::UNICODE;
        // U+2506 / U+2504: light triple dash, vertical and horizontal
        assert_eq!((u.scroll_track, u.scroll_h_track), ("┆", "┄"));
        assert_eq!((u.scroll_thumb, u.scroll_h_thumb), ("┃", "━"));
        assert_eq!((u.divider, u.h_divider), ("│", "─"));
        let a = Glyphs::ASCII;
        assert_eq!((a.scroll_track, a.scroll_h_track), (":", "."));
        assert_eq!(a.scroll_thumb, a.scroll_h_thumb, "one ASCII thumb");
        assert_eq!((a.divider, a.h_divider), ("|", "-"));
        for g in [u, a] {
            for s in [
                g.scroll_track,
                g.scroll_h_track,
                g.scroll_thumb,
                g.scroll_h_thumb,
                g.divider,
                g.h_divider,
            ] {
                assert_eq!(s.width(), 1, "{s:?} is one cell");
            }
        }
    }

    #[test]
    fn a_divider_is_solid_while_it_fits_and_a_dotted_scrollbar_once_it_overflows() {
        let theme = Theme::junie();
        let c = WidgetId::of("c");
        let draw = |s: &ScrollState| {
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let mut buf = Buffer::empty(Rect::new(0, 0, 1, 6));
            {
                let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
                render_divider(Rect::new(0, 0, 1, 6), &mut buf, &mut ctx, c, s, false);
            }
            let cells: Vec<_> = (0..6)
                .map(|y| (buf[(0, y)].symbol().to_owned(), buf[(0, y)].style()))
                .collect();
            (cells, hits)
        };
        let mut s = ScrollState::new(4);
        s.set_viewport(6);
        let (cells, hits) = draw(&s);
        for (sym, style) in &cells {
            assert_eq!(sym, theme.glyphs.divider, "fits: a plain divider");
            assert_eq!(ink(*style), ink(theme.border(false)));
        }
        assert!(hits.is_empty(), "nothing to scroll, nothing to press");
        s.set_content(40);
        let (cells, hits) = draw(&s);
        assert_eq!(cells[0].0, theme.glyphs.scroll_thumb);
        assert!(
            cells[1..]
                .iter()
                .all(|(sym, st)| sym == theme.glyphs.scroll_track
                    && ink(*st) == ink(theme.scrollbar_track()))
        );
        assert_eq!(hits.hit(Position::new(0, 3)), Some(id_for(c)));
    }

    #[test]
    fn bars_leave_the_corner_blank() {
        let b = Bars::split(Rect::new(0, 0, 20, 10), true, true);
        assert_eq!(b.content, Rect::new(0, 0, 19, 9));
        assert_eq!(b.vertical, Some(Rect::new(19, 0, 1, 9)));
        assert_eq!(b.horizontal, Some(Rect::new(0, 9, 19, 1)));
        let b = Bars::split(Rect::new(0, 0, 20, 10), false, true);
        assert_eq!(b.horizontal, Some(Rect::new(0, 9, 20, 1)));
        assert_eq!(b.vertical, None);
    }
}
