use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::*;
use crate::core::focus::FocusRing;
use crate::core::hit::HitRegistry;
use crate::theme::Theme;
use crate::ui::ctx::Interaction;

fn row(buf: &Buffer) -> String {
    let a = buf.area;
    (a.x..a.right())
        .map(|x| buf[(x, a.y)].symbol().to_owned())
        .collect::<String>()
}

fn draw(footer: Footer, width: u16) -> String {
    let theme = Theme::junie();
    let mut hits = HitRegistry::default();
    let mut ring = FocusRing::default();
    let ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
    let area = Rect::new(0, 0, width, 1);
    let mut buf = Buffer::empty(area);
    let mut shell: Shell = Shell::new();
    footer.render(area, &mut buf, &ctx, &mut shell);
    row(&buf)
}

fn keys(h: &[Hint]) -> Vec<String> {
    h.iter()
        .map(|h| format!("{} {}", h.key, h.action))
        .collect()
}

#[test]
fn the_topmost_context_with_hints_wins_then_the_fallback() {
    let f = Footer::new()
        .layer(None)
        .layer(Some(vec![]))
        .layer(Some(vec![hint("Enter", "Open")]))
        .layer(Some(vec![hint("d", "Delete")]))
        .fallback(vec![hint("F1", "Help")]);
    assert_eq!(keys(&f.compose(None)), ["Enter Open"]);
    let f = Footer::new().layer(None).fallback(vec![hint("F1", "Help")]);
    assert_eq!(keys(&f.compose(None)), ["F1 Help"]);
}

#[test]
fn tab_next_is_added_once_and_never_while_editing() {
    let f = Footer::new()
        .layer(Some(vec![hint("d", "Delete")]))
        .tab_next(true);
    assert_eq!(keys(&f.compose(None)), ["d Delete", "Tab Next"]);
    let named = Footer::new()
        .layer(Some(vec![hint("Tab", "Into page")]))
        .tab_next(true);
    assert_eq!(keys(&named.compose(None)), ["Tab Into page"]);
    let editing = Footer::new()
        .layer(Some(vec![hint("Enter", "Apply")]))
        .tab_next(true)
        .editing(true);
    assert_eq!(keys(&editing.compose(None)), ["Enter Apply"]);
}

#[test]
fn hints_never_name_shift_click_or_an_undelivered_cmd_chord() {
    crate::core::keyboard::set_super_delivered(false);
    let f = Footer::new().layer(Some(vec![
        hint("Shift+click", "Extend"),
        hint("⌘A", "Select all"),
        hint("Ctrl+A", "Select all"),
    ]));
    assert_eq!(keys(&f.compose(None)), ["Ctrl+A Select all"]);
}

#[test]
fn an_error_wins_over_every_hint_and_info_leaves_the_way_out_room() {
    let hints = || {
        vec![
            hint("Enter", "Open"),
            hint("d", "Delete"),
            hint("Esc", "Back"),
        ]
    };
    let msg = "Connection refused by the server at db.example";
    let info = draw(
        Footer::new()
            .layer(Some(hints()))
            .status(Some((msg, Tone::Secondary))),
        50,
    );
    assert!(
        info.contains("Esc Back"),
        "info keeps the way out: {info:?}"
    );
    let error = draw(
        Footer::new()
            .layer(Some(hints()))
            .status(Some((msg, Tone::Error))),
        50,
    );
    assert!(error.contains(msg), "the error is not cut: {error:?}");
    assert!(!error.contains("Esc Back"), "{error:?}");
}

#[test]
fn the_commit_key_stays_pinned_while_dirty_only_when_asked() {
    let hints = || {
        vec![
            hint("Enter", "Edit"),
            hint("d", "Delete"),
            hint("Ctrl+S", "Save"),
            hint("Esc", "Back"),
        ]
    };
    let plain = draw(Footer::new().layer(Some(hints())).dirty(true), 24);
    assert!(!plain.contains("Ctrl+S"), "{plain:?}");
    let pinned = draw(
        Footer::new()
            .layer(Some(hints()))
            .dirty(true)
            .pin_dirty(Some("Ctrl+S")),
        24,
    );
    assert!(pinned.contains("Ctrl+S Save"), "{pinned:?}");
}

#[test]
fn the_edit_badge_leads_the_row() {
    let r = draw(
        Footer::new()
            .layer(Some(vec![hint("Enter", "Apply")]))
            .editing(true),
        40,
    );
    assert!(r.trim_start().starts_with("EDIT"), "{r:?}");
}

#[test]
fn the_peek_line_shows_the_focused_cut_value_in_full() {
    let theme = Theme::junie();
    let draw_peek = |footer: Footer| {
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        crate::ui::reveal::peek(&mut ctx, "a value\nfar too long for its cell");
        let area = Rect::new(0, 0, 80, 1);
        let mut buf = Buffer::empty(area);
        let mut shell: Shell = Shell::new();
        footer.render(area, &mut buf, &ctx, &mut shell);
        row(&buf)
    };
    let shown = draw_peek(Footer::new().fallback(vec![hint("F1", "Help")]));
    assert!(
        shown.contains("a value far too long for its cell"),
        "{shown}"
    );
    let status = draw_peek(Footer::new().status(Some(("Saved", Tone::Success))));
    assert!(!status.contains("far too long"), "{status}");
    let off = draw_peek(Footer::new().peek(false));
    assert!(!off.contains("far too long"), "{off}");
}
