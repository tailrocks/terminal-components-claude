//! [`ScrollView`] unit tests: measure and layout, both bars, focus
//! follow, scroll chaining, bar press and drag, keys, moved hits and
//! overlays.

use super::*;
use crate::core::focus::FocusRing;
use crate::core::hit::HitRegistry;
use crate::core::intent::Phase;
use crate::theme::{ColorLevel, Theme};
use crate::ui::ctx::Interaction;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};

const VIEW: WidgetId = WidgetId::of("test.view");

fn theme() -> Theme {
    Theme::for_level(ColorLevel::TrueColor)
}

/// Content filling `area`, each cell the letter of its row (a..z) so a
/// window's position reads from its cells, with a control at each rect of
/// `controls`.
fn content(area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, controls: &[(WidgetId, Rect)]) {
    for y in area.top()..area.bottom() {
        let ch = (b'a' + ((y - area.y) % 26) as u8) as char;
        for x in area.left()..area.right() {
            buf[(x, y)].set_char(ch);
        }
    }
    for &(id, r) in controls {
        ctx.control(id, r, false);
    }
}

struct Frame {
    buf: Buffer,
    hits: HitRegistry,
    ring: FocusRing,
}

/// One frame of `view` in `area` (on a 30x10 screen) over content of
/// `size`, with `focus` focused and `controls` placed relative to the
/// content's top-left.
fn frame(
    view: &mut ScrollView,
    area: Rect,
    size: Size,
    focus: Option<WidgetId>,
    controls: &[(WidgetId, Rect)],
) -> Frame {
    let t = theme();
    let mut hits = HitRegistry::default();
    let mut ring = FocusRing::default();
    let mut buf = Buffer::empty(Rect::new(0, 0, 30, 10));
    let interaction = Interaction {
        focus,
        ..Interaction::default()
    };
    {
        let mut ctx = RenderCtx::new(&t, interaction, &mut hits, &mut ring);
        view.render(area, size, &mut buf, &mut ctx, |inner, buf, ctx| {
            let placed: Vec<(WidgetId, Rect)> = controls
                .iter()
                .map(|&(id, r)| {
                    (
                        id,
                        Rect::new(inner.x + r.x, inner.y + r.y, r.width, r.height),
                    )
                })
                .collect();
            content(inner, buf, ctx, &placed)
        });
        ctx.flush_overlays(&mut buf);
    }
    Frame { buf, hits, ring }
}

fn row(buf: &Buffer, y: u16) -> String {
    (0..buf.area.width).map(|x| buf[(x, y)].symbol()).collect()
}

fn key(code: KeyCode, mods: KeyModifiers) -> Key {
    Key { code, mods }
}

fn ev(hits: &HitRegistry, phase: Phase, id: WidgetId, x: u16, y: u16) -> PointerEvent {
    PointerEvent::new(hits, phase, id, Position::new(x, y), 1, KeyModifiers::NONE)
}

fn wheel(hits: &HitRegistry, axis: Axis, delta: i32) -> PointerEvent {
    PointerEvent::wheel_on(
        hits,
        VIEW,
        axis,
        delta,
        Position::new(3, 3),
        KeyModifiers::NONE,
    )
}

#[test]
fn content_that_fits_draws_in_place_with_no_bars_and_no_wheel_region() {
    let mut v = ScrollView::new(VIEW);
    let area = Rect::new(0, 0, 20, 8);
    let f = frame(&mut v, area, Size::new(20, 8), None, &[]);
    assert_eq!(row(&f.buf, 0), format!("{}          ", "a".repeat(20)));
    assert_eq!(row(&f.buf, 7), format!("{}          ", "h".repeat(20)));
    assert!(f.hits.scroll_stack(Position::new(3, 3)).is_empty());
    assert!(!v.last_layout().overflows());
}

#[test]
fn layout_settles_both_bars_and_leaves_the_corner_blank() {
    let v = ScrollView::new(VIEW);
    let area = Rect::new(0, 0, 20, 8);
    // taller only: a vertical bar and its gap
    let l = v.layout(area, Size::new(18, 30));
    assert_eq!(l.viewport, Rect::new(0, 0, 18, 8));
    assert_eq!(l.v_track, Some(Rect::new(19, 0, 1, 8)));
    assert_eq!(l.h_track, None);
    // exactly as tall but wider: the horizontal bar takes a row, so the
    // rows overflow too and both bars show
    let l = v.layout(area, Size::new(40, 8));
    assert_eq!(l.viewport, Rect::new(0, 0, 18, 7));
    assert_eq!(l.v_track, Some(Rect::new(19, 0, 1, 7)));
    assert_eq!(l.h_track, Some(Rect::new(0, 7, 19, 1)));
    assert_eq!(l.corner, Some(Rect::new(19, 7, 1, 1)));
    assert_eq!(l.content, Rect::new(0, 0, 40, 8));
    // one axis only: the content gets the view's width
    let l = ScrollView::new(VIEW)
        .axes(Axes::VERTICAL)
        .layout(area, Size::new(40, 30));
    assert_eq!(l.h_track, None);
    assert_eq!(l.content.width, l.viewport.width);
    // never: no bar, the view still scrolls
    let l = ScrollView::new(VIEW)
        .bars(ScrollbarVisibility::Never)
        .layout(area, Size::new(40, 30));
    assert_eq!((l.v_track, l.h_track), (None, None));
    assert_eq!(l.viewport, area);
    assert!(l.overflows());
    // padding scrolls with the content
    let l = ScrollView::new(VIEW)
        .padding(Padding::new(1, 1, 1, 1))
        .layout(area, Size::new(18, 6));
    assert_eq!(l.content, Rect::new(0, 0, 20, 8));
    assert_eq!(l.inner, Rect::new(1, 1, 18, 6));
    assert!(!l.overflows());
}

#[test]
fn scrolling_to_the_end_shows_the_last_row_and_column_with_both_bars() {
    let mut v = ScrollView::new(VIEW);
    let area = Rect::new(0, 0, 20, 8);
    let size = Size::new(40, 30);
    let f = frame(&mut v, area, size, None, &[]);
    let t = theme();
    // the dotted tracks and the blank corner
    assert_eq!(f.buf[(19, 3)].symbol(), t.glyphs.scroll_track);
    assert_eq!(f.buf[(15, 7)].symbol(), t.glyphs.scroll_h_track);
    assert_eq!(f.buf[(19, 7)].symbol(), " ");
    assert_eq!(f.buf[(0, 0)].symbol(), "a");
    // to the end on each axis
    v.pane.v.jump_end();
    v.pane.h.scroll_to(usize::MAX);
    let f = frame(&mut v, area, size, None, &[]);
    assert_eq!(v.offset(), (40 - 18, 30 - 7));
    // the last content row ('d' is row 29) is the last row in view
    assert_eq!(f.buf[(17, 6)].symbol(), "d");
    assert_eq!(f.buf[(0, 0)].symbol(), "x");
}

#[test]
fn hits_move_with_the_window_and_pointer_events_read_content_positions() {
    let mut v = ScrollView::new(VIEW);
    let area = Rect::new(0, 0, 20, 8);
    let size = Size::new(40, 30);
    let near = WidgetId::of("near");
    let far = WidgetId::of("far");
    let controls = [
        (near, Rect::new(2, 10, 6, 1)),
        (far, Rect::new(2, 28, 6, 1)),
    ];
    // the offset is clamped against the layout: lay out once first
    frame(&mut v, area, size, None, &controls);
    v.pane.v.scroll_to(8);
    let f = frame(&mut v, area, size, None, &controls);
    assert_eq!(v.offset(), (0, 8));
    // in view: content row 10 is the third row of the view
    assert_eq!(f.hits.area_of(near), Some(Rect::new(2, 2, 6, 1)));
    // out of view: no region, but still a focus stop
    assert_eq!(f.hits.area_of(far), None);
    assert!(f.ring.contains(far));
    // a click on it reports where the control drew
    let e = ev(&f.hits, Phase::Click, near, 4, 2);
    assert_eq!(e.pos(), Position::new(4, 10));
}

#[test]
fn focus_moving_to_a_clipped_control_brings_it_into_view_with_context() {
    let mut v = ScrollView::new(VIEW);
    let area = Rect::new(0, 0, 20, 8);
    let size = Size::new(40, 30);
    let far = WidgetId::of("far");
    let wide = WidgetId::of("wide");
    let controls = [
        (far, Rect::new(2, 20, 6, 2)),
        (wide, Rect::new(30, 3, 6, 1)),
    ];
    let f = frame(&mut v, area, size, Some(far), &controls);
    // rows 20..22 and one row of context end the 7-row view
    assert_eq!(v.offset().1, 22 + 1 - 7);
    assert_eq!(f.hits.area_of(far), Some(Rect::new(2, 20 - 16, 6, 2)));
    // the wheel may scroll away again: focus did not move
    v.pane.v.scroll_to(0);
    frame(&mut v, area, size, Some(far), &controls);
    assert_eq!(v.offset().1, 0);
    // focus moves sideways out of view: the columns follow
    frame(&mut v, area, size, Some(wide), &controls);
    assert_eq!(v.offset().0, 36 + 1 - 18);
}

#[test]
fn a_list_cursor_inside_a_tall_control_is_followed_by_its_gutter() {
    let t = theme();
    let mut v = ScrollView::new(VIEW);
    let area = Rect::new(0, 0, 20, 8);
    let list = WidgetId::of("list");
    for cursor in [3u16, 25] {
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut buf = Buffer::empty(Rect::new(0, 0, 30, 10));
        let interaction = Interaction {
            focus: Some(list),
            ..Interaction::default()
        };
        let mut ctx = RenderCtx::new(&t, interaction, &mut hits, &mut ring);
        v.render(
            area,
            Size::new(18, 30),
            &mut buf,
            &mut ctx,
            |inner, buf, ctx| {
                ctx.control(list, inner, false);
                buf.set_string(
                    inner.x,
                    inner.y + cursor,
                    t.glyphs.focus_bar,
                    ratatui::style::Style::new(),
                );
            },
        );
    }
    // row 25 and one row of context end the 8-row view
    assert_eq!(v.offset().1, 25 + 1 + 1 - 8);
}

#[test]
fn the_wheel_scrolls_the_view_and_a_short_inner_scroller_passes_it_on() {
    let mut v = ScrollView::new(VIEW);
    let area = Rect::new(0, 0, 20, 8);
    let f = frame(&mut v, area, Size::new(18, 30), None, &[]);
    assert_eq!(f.hits.scroll_stack(Position::new(3, 3)), vec![VIEW]);
    let r = v.pointer(&wheel(&f.hits, Axis::Vertical, 3));
    assert!(r.is_some_and(|r| r.is_changed()));
    assert_eq!(v.offset().1, 3);
    // no columns to scroll: a sideways notch goes on to an outer owner
    let r = v.pointer(&wheel(&f.hits, Axis::Horizontal, 1));
    assert_eq!(r.map(|r| r.outcome()), Some(Outcome::Ignored));
    // an inner pane whose rows fit ignores the notch (chaining); one that
    // overflows keeps it, even at its end
    let mut fits = ScrollPane::new(5);
    fits.set_viewport(8);
    assert_eq!(fits.wheel(Axis::Vertical, 3).outcome(), Outcome::Ignored);
    let mut over = ScrollPane::new(20);
    over.set_viewport(8);
    over.jump_end();
    assert_eq!(over.wheel(Axis::Vertical, 3).outcome(), Outcome::Consumed);
}

#[test]
fn an_inner_scroller_is_offered_the_notch_before_the_view() {
    let t = theme();
    let mut v = ScrollView::new(VIEW);
    let inner = WidgetId::of("inner");
    let mut hits = HitRegistry::default();
    let mut ring = FocusRing::default();
    let mut buf = Buffer::empty(Rect::new(0, 0, 30, 10));
    {
        let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
        v.render(
            Rect::new(0, 0, 20, 8),
            Size::new(18, 30),
            &mut buf,
            &mut ctx,
            |area, _buf, ctx| ctx.scrollable(inner, Rect::new(area.x, area.y, 10, 4)),
        );
    }
    assert_eq!(hits.scroll_stack(Position::new(3, 3)), vec![inner, VIEW]);
}

#[test]
fn pressing_and_dragging_either_bar_scrolls_its_axis() {
    let mut v = ScrollView::new(VIEW).track_click(TrackClick::Jump);
    let area = Rect::new(0, 0, 20, 8);
    let f = frame(&mut v, area, Size::new(40, 30), None, &[]);
    let vbar = crate::widgets::scrollbar::id_for(VIEW);
    let hbar = crate::widgets::scrollbar::h_id_for(VIEW);
    // grab the vertical thumb (at the top) and drag it past the bottom
    let press = ev(&f.hits, Phase::Press, vbar, 19, 0);
    assert!(press.bar.is_some());
    let _ = v.pointer(&press);
    let _ = v.pointer(&ev(&f.hits, Phase::Drag, vbar, 19, 9));
    assert_eq!(v.offset().1, 30 - 7);
    let _ = v.pointer(&ev(&f.hits, Phase::Release, vbar, 19, 9));
    // a press on the far end of the horizontal track jumps there
    let _ = v.pointer(&ev(&f.hits, Phase::Press, hbar, 18, 7));
    assert_eq!(v.offset().0, 40 - 18);
}

#[test]
fn unused_page_keys_scroll_the_view() {
    let mut v = ScrollView::new(VIEW);
    let area = Rect::new(0, 0, 20, 8);
    frame(&mut v, area, Size::new(40, 30), None, &[]);
    let none = KeyModifiers::NONE;
    assert_eq!(
        v.on_key(&key(KeyCode::PageDown, none)),
        Some(Outcome::Changed)
    );
    assert_eq!(v.offset().1, 7);
    assert_eq!(
        v.on_key(&key(KeyCode::PageDown, KeyModifiers::ALT)),
        Some(Outcome::Changed)
    );
    assert_eq!(v.offset().0, 18);
    assert_eq!(v.on_key(&key(KeyCode::End, none)), Some(Outcome::Changed));
    assert_eq!(v.offset().1, 23);
    assert_eq!(v.on_key(&key(KeyCode::Home, none)), Some(Outcome::Changed));
    assert_eq!(v.offset(), (0, 0));
    assert_eq!(v.on_key(&key(KeyCode::Char('x'), none)), None);
    // content that fits leaves the keys alone
    let mut fits = ScrollView::new(VIEW);
    frame(&mut fits, area, Size::new(10, 5), None, &[]);
    assert_eq!(fits.on_key(&key(KeyCode::PageDown, none)), None);
}

#[test]
fn an_overlay_queued_by_scrolled_content_lands_over_its_anchor() {
    let t = theme();
    let mut v = ScrollView::new(VIEW);
    let area = Rect::new(0, 0, 20, 8);
    let pop = WidgetId::of("pop");
    // the offset is clamped against the layout: lay out once first
    frame(&mut v, area, Size::new(18, 30), None, &[]);
    v.pane.v.scroll_to(10);
    let mut hits = HitRegistry::default();
    let mut ring = FocusRing::default();
    let mut buf = Buffer::empty(Rect::new(0, 0, 30, 10));
    {
        let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
        v.render(
            area,
            Size::new(18, 30),
            &mut buf,
            &mut ctx,
            |inner, _buf, ctx| {
                // a popup on content row 13
                let at = Rect::new(inner.x + 1, inner.y + 13, 4, 1);
                ctx.overlay(move |buf, ctx| {
                    buf.set_string(at.x, at.y, "POP!", ratatui::style::Style::new());
                    ctx.clickable(pop, at);
                });
            },
        );
        ctx.flush_overlays(&mut buf);
    }
    assert_eq!(v.offset().1, 10);
    assert_eq!(row(&buf, 3).get(1..5), Some("POP!"));
    assert_eq!(hits.area_of(pop), Some(Rect::new(1, 3, 4, 1)));
    let e = ev(&hits, Phase::Click, pop, 2, 3);
    assert_eq!(e.pos(), Position::new(2, 13));
}

#[test]
fn reveal_span_brings_a_span_in_with_context() {
    let mut s = crate::core::scroll::ScrollState::new(30);
    s.set_viewport(8);
    assert!(s.reveal_span(20, 2));
    assert_eq!(s.offset, 20 + 2 + 1 - 8);
    assert!(s.reveal_span(3, 1));
    assert_eq!(s.offset, 2);
    // taller than the view: its start
    assert!(s.reveal_span(10, 12));
    assert_eq!(s.offset, 10);
}
