//! Port of the PR #26 reference backdrop tests (`src/theme.rs`
//! f4867e9b5f8a08a3bb61ce80d644198a29f51984, `backdrop` → parity
//! `Theme::backdrop_style`), including the per-tier checks that pin the
//! modal-backdrop mapping at every color level.

use ratatui_core::style::{Color, Style};
use termrock_theme::{COLOR_LEVELS, ColorLevel, Surface, Theme, contrast_ratio};

fn planes(theme: &Theme) -> [Color; 7] {
    [
        theme.bg(Surface::Canvas),
        theme.bg(Surface::Surface),
        theme.bg(Surface::Elevated),
        theme.color.field,
        theme.color.field_hover,
        theme.bg(Surface::Overlay),
        theme.bg(Surface::Popover),
    ]
}

#[test]
fn the_modal_backdrop_keeps_reversed_text_readable() {
    let theme = Theme::junie();
    let canvas = theme.bg(Surface::Canvas);
    let primary = theme.color.fg[0];

    let st = theme.backdrop_style(Style::new().fg(canvas).bg(primary));
    assert_ne!(st.fg, st.bg);

    let st = theme.backdrop_style(Style::new().fg(canvas).bg(canvas));
    assert_eq!(st.fg, st.bg);
}

#[test]
fn the_modal_backdrop_dims_text_without_erasing_it_at_every_level() {
    for level in COLOR_LEVELS {
        let theme = Theme::junie().for_level(level);
        let ghost = theme.color.fg[4];
        for bg in planes(&theme) {
            for fg in [
                theme.bg(Surface::Canvas),
                ghost,
                theme.color.fg[3],
                theme.color.fg[2],
                theme.color.border_strong,
            ] {
                if contrast_ratio(fg, bg).is_none_or(|r| r < 3.0) {
                    continue;
                }
                let st = theme.backdrop_style(Style::new().fg(fg).bg(bg));
                let (Some(ink), Some(plane)) = (st.fg, st.bg) else {
                    panic!("{level:?}: backdrop dropped a channel for {fg:?} on {bg:?}");
                };
                let r = contrast_ratio(ink, plane).unwrap();
                assert!(
                    r >= 3.0,
                    "{level:?}: {fg:?} on {bg:?} dims to {ink:?} on {plane:?} ({r:.2})"
                );
            }
        }

        let st = theme.backdrop_style(Style::new().fg(ghost).bg(theme.bg(Surface::Canvas)));
        if level == ColorLevel::TrueColor {
            assert_eq!(st.fg, Some(ghost));
        }
    }
}

#[test]
fn backdrop_text_never_lands_on_the_backdrop_grey_at_reduced_levels() {
    for level in COLOR_LEVELS {
        let theme = Theme::junie().for_level(level);
        let mut planes = [
            theme.bg(Surface::Canvas),
            theme.bg(Surface::Elevated),
            theme.bg(Surface::Overlay),
            theme.bg(Surface::Popover),
            theme.color.accent,
        ];
        if let Color::Reset = planes[0] {
            planes[0] = theme.color.fg[0];
        }
        for bg in planes {
            for fg in [
                theme.color.fg[0],
                theme.color.fg[1],
                theme.color.fg[2],
                theme.color.fg[3],
                theme.color.fg[4],
                theme.color.accent,
            ] {
                if fg == bg {
                    continue;
                }
                let st = theme.backdrop_style(Style::new().fg(fg).bg(bg));
                assert_ne!(
                    st.fg, st.bg,
                    "{level:?}: {fg:?} on {bg:?} lands on the backdrop"
                );
            }
        }
    }
}
