//! The Splits & stacks page through the real app (plan §7 SC23): seams
//! drag, reset and pad their hit target as the options say, zoom hides a
//! pane, and the stack shrinks, scrolls or clips.

use junie_tui::core::event::MouseKind;
use junie_tui::core::id::WidgetId;

use crate::app::PageId;
use crate::app_tests_feedback::Harness;
use crate::pages::template::Section;

fn outer() -> WidgetId {
    WidgetId::of("splitsstacks").sub("outer")
}

/// Width of the Explorer pane: its frame from `╭` to `╮`.
fn explorer_width(h: &Harness) -> u16 {
    let (x, y) = h.at("╭─ Explorer");
    let row: Vec<String> = (x..h.term.backend().buffer().area.width)
        .map(|cx| h.cell(cx, y).symbol().to_owned())
        .collect();
    row.iter()
        .position(|s| s == "╮")
        .unwrap_or_else(|| panic!("pane frame:\n{}", h.text())) as u16
        + 1
}

fn drag(h: &mut Harness, from: (u16, u16), to: (u16, u16)) {
    h.mouse(MouseKind::Down, from.0, from.1);
    h.mouse(MouseKind::Drag, to.0, to.1);
    h.mouse(MouseKind::Up, to.0, to.1);
}

#[test]
fn splits_seam_drags_and_a_double_click_resets_unless_ignored() {
    for (policy, resets) in [("reset", true), ("ignore", false)] {
        let mut h = Harness::new(120, 40, PageId::SplitsStacks);
        h.opt(&format!("splitsstacks.double_click={policy}"));
        let before = explorer_width(&h);
        let seam = h.area(outer());
        drag(&mut h, (seam.x, seam.y + 2), (seam.x + 12, seam.y + 2));
        let dragged = explorer_width(&h);
        assert_eq!(dragged, before + 12, "{policy}");
        let seam = h.area(outer());
        h.click(seam.x, seam.y + 2);
        h.click(seam.x, seam.y + 2);
        assert_eq!(
            explorer_width(&h),
            if resets { before } else { dragged },
            "{policy}"
        );
    }
}

#[test]
fn splits_hit_pad_widens_the_press_target() {
    for (pad, moves) in [("1", true), ("0", false)] {
        let mut h = Harness::new(120, 40, PageId::SplitsStacks);
        h.opt(&format!("splitsstacks.hit_pad={pad}"));
        let before = explorer_width(&h);
        let seam = h.area(outer());
        // one cell right of the seam: the editor pane's frame
        drag(&mut h, (seam.x + 1, seam.y + 2), (seam.x + 9, seam.y + 2));
        assert_eq!(explorer_width(&h) != before, moves, "pad {pad}");
    }
}

#[test]
fn splits_minimum_clamps_the_default_ratio() {
    let mut h = Harness::new(120, 40, PageId::SplitsStacks);
    assert!(explorer_width(&h) < 40);
    h.opt("splitsstacks.min=40");
    assert_eq!(explorer_width(&h), 40, "{}", h.text());
    // a 70% explorer with the editor whole beside it wants more than 120
    // columns: wide enough that the page needs no sideways scroll
    let mut h = Harness::new(240, 40, PageId::SplitsStacks);
    h.opt("splitsstacks.min=40");
    let at_30 = explorer_width(&h);
    h.opt("splitsstacks.percent=70");
    let at_70 = explorer_width(&h);
    assert!(at_70 > 40 && at_70 > at_30, "{at_30} -> {at_70}");
}

#[test]
fn splits_zoom_hides_the_other_pane_and_its_seam() {
    let mut h = Harness::new(120, 40, PageId::SplitsStacks);
    let inner = WidgetId::of("splitsstacks").sub("inner");
    assert!(h.find("Output").is_some() && h.find("Editor").is_some());
    h.opt("splitsstacks.maximized=first");
    assert!(h.find("─ Output").is_none(), "{}", h.text());
    assert!(
        h.app.shell.hits.area_of(inner).is_none(),
        "no seam while zoomed"
    );
    h.opt("splitsstacks.maximized=none");
    let (x, y) = h.at("Zoom output");
    h.click(x + 1, y);
    assert!(h.find("─ Editor").is_none());
    assert!(h.text().contains("output zoomed"));
    let (x, y) = h.at("Restore");
    h.click(x + 1, y);
    assert!(h.find("─ Editor").is_some() && h.find("─ Output").is_some());
}

#[test]
fn splits_direction_stacks_the_panes() {
    let mut h = Harness::new(120, 40, PageId::SplitsStacks);
    let (_, ey) = h.at("─ Explorer");
    let (_, dy) = h.at("─ Editor");
    assert_eq!(ey, dy, "side by side");
    h.opt("splitsstacks.dir=vertical");
    let (_, ey) = h.at("─ Explorer");
    let (dx, dy) = h.at("─ Editor");
    let (ox, oy) = h.at("─ Output");
    assert!(dy > ey, "the editor under the explorer");
    assert!(
        oy == dy && ox > dx,
        "editor and output side by side under it"
    );
}

#[test]
fn stack_children_shrink_then_scroll_or_clip() {
    // tall enough for the stack card under the panes at their natural size
    let mut h = Harness::new(120, 50, PageId::SplitsStacks);
    assert!(h.text().contains("▸ Danger zone · 1 row"));
    assert!(!h.text().contains("not in view"));
    h.opt("splitsstacks.min_child_rows=3");
    assert!(h.text().contains("not in view"), "{}", h.text());
    let stack = WidgetId::of("splitsstacks").sub("stack");
    assert!(
        h.app
            .shell
            .hits
            .area_of(junie_tui::widgets::scrollbar::id_for(stack))
            .is_some()
    );
    let (x, y) = h.at("▸ Connection");
    h.mouse(MouseKind::WheelDown, x, y);
    assert!(
        h.find("▸ Connection").is_none(),
        "the wheel scrolls a child"
    );
    h.opt("splitsstacks.overflow=clip");
    assert!(h.text().contains("▸ Connection · 3 rows"));
    assert!(
        h.app
            .shell
            .hits
            .area_of(junie_tui::widgets::scrollbar::id_for(stack))
            .is_none(),
        "a clipped stack has no scrollbar"
    );
}

#[test]
fn stack_gap_puts_a_blank_row_between_children() {
    let mut h = Harness::new(120, 40, PageId::SplitsStacks);
    let (_, a) = h.at("▸ Connection");
    let (_, b) = h.at("▸ Credentials");
    assert_eq!(b, a + 3);
    h.opt("splitsstacks.gap=1");
    let (_, a) = h.at("▸ Connection");
    let (_, b) = h.at("▸ Credentials");
    assert_eq!(b, a + 4, "{}", h.text());
}

#[test]
fn splits_states_grid_shows_both_seams() {
    let mut h = Harness::new(120, 40, PageId::SplitsStacks);
    h.section(Section::States);
    let text = h.text();
    for label in [
        "Vertical seam",
        "Horizontal seam",
        "rest",
        "hover",
        "pressed",
    ] {
        assert!(text.contains(label), "{label}: {text}");
    }
}

/// Hovering or pressing a seam restyles only that seam's own line of
/// cells: no other seam or rule changes, the hover is the heavy rule in a
/// brighter grey without an underline (it drew a ladder of ticks on `┃`
/// and a double rule under `━`), and a held seam is the phosphor accent.
/// Checked on the Splits & stacks page and the Query workbench.
#[test]
fn hovering_or_pressing_a_seam_restyles_only_its_own_line() {
    let theme = junie_tui::theme::Theme::junie();
    let pages = [
        (PageId::SplitsStacks, "splitsstacks"),
        (PageId::QueryWorkbench, "queryworkbench"),
    ];
    for (page, slug) in pages {
        for (part, vertical) in [("outer", true), ("inner", false)] {
            let at = format!("{slug} {part}");
            let mut h = Harness::new(117, 33, page);
            h.section(Section::Playground);
            let id = WidgetId::of(slug).sub(part);
            let seam = h.area(id);
            let snap = |h: &Harness| h.term.backend().buffer().clone();
            let rest = snap(&h);
            let (x, y) = if vertical {
                (seam.x, seam.y + seam.height / 2)
            } else {
                (seam.x + seam.width / 2, seam.y)
            };
            let line = |cx: u16, cy: u16| if vertical { cx == x } else { cy == y };
            let check = |h: &Harness, heavy: &str, fg: ratatui::style::Color, what: &str| {
                let now = snap(h);
                let mut changed = 0;
                for pos in rest.area.positions() {
                    if rest[pos] != now[pos] {
                        assert!(
                            line(pos.x, pos.y),
                            "{at} {what}: {pos:?} is off the seam\n{}",
                            h.text()
                        );
                        changed += 1;
                    }
                }
                assert!(changed > 0, "{at} {what}: the seam reacts");
                let c = &now[(x, y)];
                assert_eq!(c.symbol(), heavy, "{at} {what}");
                assert_eq!(c.fg, fg, "{at} {what}");
                assert!(
                    !c.modifier.contains(ratatui::style::Modifier::UNDERLINED),
                    "{at} {what}: no underline on the rule"
                );
            };
            let (thin, heavy) = if vertical {
                (theme.glyphs.divider, theme.glyphs.scroll_thumb)
            } else {
                (theme.glyphs.h_divider, theme.glyphs.scroll_h_thumb)
            };
            assert_eq!(rest[(x, y)].symbol(), thin, "{at} rest");
            assert_eq!(rest[(x, y)].fg, theme.ladders.seam.rest, "{at} rest");
            h.mouse(MouseKind::Move, x, y);
            check(&h, heavy, theme.ladders.seam.hover, "hover");
            h.mouse(MouseKind::Down, x, y);
            check(&h, heavy, theme.ladders.handle.pressed, "pressed");
            h.mouse(MouseKind::Up, x, y);
        }
    }
}
