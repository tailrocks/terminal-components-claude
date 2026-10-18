//! WI-HOLLA-RUNTIME-01: Holla scroll geometry runs on the shared ScrollState.
//!
//! Contract `docs/components/scroll-region.md` (W45), case W45-03: thumb drag
//! with a nonzero grab offset reaches exact ends. Visual authority is the
//! frozen `visual-baseline` tag (`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
//! The journey below drives a real Holla `ScrollPanel` through its real
//! render + scrollbar press/drag handlers; the panel's scroll model must be
//! the shared `termrock::ScrollState`, not an app-local copy.

use holla_ui::tui::core::event::Outcome;
use holla_ui::tui::core::focus::FocusRing;
use holla_ui::tui::core::hit::HitRegistry;
use holla_ui::tui::core::id::WidgetId;
use holla_ui::tui::theme::{ColorLevel, Theme};
use holla_ui::tui::ui::ctx::{Interaction, RenderCtx};
use holla_ui::tui::widgets::panel::ScrollPanel;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::Style;

/// Compile-time proof: the Holla panel owns the shared scroll model.
fn assert_shared(_: &termrock::ScrollState) {}

fn style_line(t: &Theme, _: &str) -> Style {
    t.secondary()
}

fn render(panel: &mut ScrollPanel, theme: &Theme, w: u16, h: u16) -> Buffer {
    let mut hits = HitRegistry::default();
    let mut ring = FocusRing::default();
    let mut ctx = RenderCtx::new(theme, Interaction::default(), &mut hits, &mut ring);
    let mut buf = Buffer::empty(Rect::new(0, 0, w, h));
    panel.render(
        Rect::new(0, 0, w, h),
        &mut buf,
        &mut ctx,
        theme.canvas,
        style_line,
    );
    buf
}

fn row(buf: &Buffer, y: u16) -> String {
    (0..buf.area.width)
        .map(|x| buf[(x, y)].symbol().to_owned())
        .collect::<String>()
}

#[test]
fn holla_panel_thumb_drag_uses_shared_scroll_state_with_nonzero_grab() {
    let theme = Theme::for_level(ColorLevel::TrueColor);
    let lines: Vec<String> = (1..=30).map(|i| format!("line {i}")).collect();
    let mut panel = ScrollPanel::new(WidgetId::of("log"), lines);
    assert_shared(&panel.scroll);

    // Real geometry: 30 content rows in a 10-row viewport overflow, so the
    // panel draws a scrollbar track in column 19 with a 3-row thumb at 0..3.
    render(&mut panel, &theme, 20, 10);
    assert!(panel.scroll.overflows());
    assert_eq!(panel.scroll.offset(), 0);
    assert_eq!(panel.scroll.thumb(10), (0, 3));

    // A press inside the thumb grabs row 2 without jumping.
    assert_eq!(panel.on_scrollbar(Position::new(19, 2)), Outcome::Consumed);
    assert_eq!(panel.scroll.offset(), 0);

    // The drag keeps the grabbed row under the pointer and lands exactly
    // on the last offset; the shared model owns that math now.
    assert_eq!(
        panel.on_scrollbar_drag(Position::new(19, 9)),
        Outcome::Changed
    );
    assert_eq!(panel.scroll.offset(), panel.scroll.max_offset());
    assert_eq!(panel.scroll.offset(), 20);
    assert!(panel.scroll.at_end());

    // The end-to-end journey shows the final content row on screen.
    let buf = render(&mut panel, &theme, 20, 10);
    assert!(
        row(&buf, 9).contains("line 30"),
        "tail row visible: {}",
        row(&buf, 9)
    );
}
