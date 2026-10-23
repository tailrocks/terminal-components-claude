#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "literal public API and model-state assertions"
)]

use termrock::{ScrollState, position_label};

#[test]
fn position_label_is_empty_without_nonzero_overflow() {
    assert_eq!(position_label(&ScrollState::default()), "");
    assert_eq!(position_label(&ScrollState::new(0)), "");

    let no_viewport = ScrollState::new(120);
    assert_eq!(position_label(&no_viewport), "");

    let mut underflow = ScrollState::new(5);
    underflow.set_viewport(10);
    assert_eq!(position_label(&underflow), "");

    let mut exact_fit = ScrollState::new(13);
    exact_fit.set_viewport(13);
    assert_eq!(position_label(&exact_fit), "");
}

#[test]
fn position_label_reports_top_middle_and_clamped_end_ranges() {
    let mut scroll = ScrollState::new(120);
    scroll.set_viewport(13);
    assert_eq!(position_label(&scroll), "1–13 of 120");

    scroll.scroll_to(11);
    assert_eq!(position_label(&scroll), "12–24 of 120");

    scroll.jump_end();
    assert_eq!(position_label(&scroll), "108–120 of 120");

    scroll.scroll_to(usize::MAX);
    assert_eq!(position_label(&scroll), "108–120 of 120");
}

#[test]
fn position_label_handles_usize_max_at_the_end() {
    let mut scroll = ScrollState::new(usize::MAX);
    scroll.set_viewport(13);
    scroll.scroll_to(usize::MAX);

    let expected = match usize::BITS {
        32 => "4294967283–4294967295 of 4294967295",
        64 => "18446744073709551603–18446744073709551615 of 18446744073709551615",
        bits => panic!("unsupported usize width for literal MAX control: {bits}"),
    };
    assert_eq!(position_label(&scroll), expected);
}

#[test]
fn position_label_observes_layout_clamping_after_resize_and_content_shrink() {
    let mut scroll = ScrollState::new(120);
    scroll.set_viewport(13);
    scroll.scroll_to(usize::MAX);
    assert_eq!(position_label(&scroll), "108–120 of 120");

    scroll.apply_layout(5, 80);
    assert_eq!(scroll.offset(), 75);
    assert_eq!(position_label(&scroll), "76–80 of 80");
}

#[test]
fn position_label_leaves_pending_reveal_follow_and_thumb_grab_unchanged() {
    let mut scroll = ScrollState::new(120);
    scroll.set_viewport(13);
    scroll.scroll_to(11);
    assert!(!scroll.follow(17));
    scroll.ensure_visible_on_next_layout(80);

    let (thumb_start, _) = scroll.thumb(20);
    assert!(!scroll.press_track(thumb_start, 20));
    assert_eq!(scroll.pending_reveal(), Some(80));
    let before = scroll;

    assert_eq!(position_label(&scroll), "12–24 of 120");
    assert_eq!(scroll, before);

    let mut drag_probe = scroll;
    assert!(drag_probe.drag_track(thumb_start.saturating_add(1), 20));
    assert_ne!(drag_probe.offset(), before.offset());
}
