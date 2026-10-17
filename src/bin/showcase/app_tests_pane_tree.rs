//! The Pane tree page through the real app (plan §7 SC45): nested panes
//! with frames, focus by click and F6, the resize chords, zoom by key and
//! button, split and close, and the options that drive the `PaneTree`.

use junie_tui::core::event::MouseKind;
use junie_tui::core::id::WidgetId;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use crate::app::PageId;
use crate::app_tests_feedback::Harness;
use crate::pages::template::Section;

const TREE: WidgetId = WidgetId::of("panetree").sub("tree");

fn button(name: &str) -> WidgetId {
    WidgetId::of("panetree").sub(name)
}

/// Width of a pane's frame from its `╭─ <name>` to the next `╮`.
fn pane_width(h: &Harness, name: &str) -> u16 {
    let (x, y) = h.at(&format!("╭─ {name}"));
    let row: Vec<String> = (x..h.term.backend().buffer().area.width)
        .map(|cx| h.cell(cx, y).symbol().to_owned())
        .collect();
    row.iter().position(|s| s == "╮").expect("pane frame") as u16 + 1
}

#[test]
fn pane_tree_shows_three_framed_panes_and_no_stub() {
    let h = Harness::new(120, 40, PageId::PaneTree);
    let text = h.text();
    assert!(!text.contains("arrives with"), "{text}");
    for name in ["╭─ shell", "╭─ build", "╭─ logs"] {
        assert!(text.contains(name), "{name}: {text}");
    }
    assert!(text.contains("3 panes · focus shell"), "{text}");
    assert!(text.contains("F6 next pane"), "{text}");
}

#[test]
fn pane_tree_f6_cycles_and_a_click_focuses_a_pane() {
    let mut h = Harness::new(120, 40, PageId::PaneTree);
    h.focus(TREE);
    h.key(KeyCode::F(6));
    assert!(h.text().contains("focus build"), "{}", h.text());
    let (x, y) = h.at("╭─ logs");
    h.click(x + 4, y + 2);
    assert!(h.text().contains("focus logs"), "{}", h.text());
}

#[test]
fn pane_tree_zoom_takes_the_whole_tree_and_restores() {
    let mut h = Harness::new(120, 40, PageId::PaneTree);
    h.focus(TREE);
    h.key_mod(KeyCode::Char('z'), KeyModifiers::ALT);
    let text = h.text();
    assert!(text.contains("zoomed"), "{text}");
    assert!(!text.contains("╭─ build"), "only the zoomed pane: {text}");
    // focus does not move while zoomed: the status says why
    h.key_mod(KeyCode::Right, KeyModifiers::ALT);
    assert!(
        h.status().contains("Unzoom to move focus"),
        "{}",
        h.status()
    );
    h.click_id(button("zoom"));
    assert!(h.text().contains("╭─ build"), "restored: {}", h.text());
}

#[test]
fn pane_tree_resize_chords_and_seam_drag_move_the_seam() {
    let mut h = Harness::new(120, 40, PageId::PaneTree);
    let w = pane_width(&h, "shell");
    h.focus(TREE);
    h.key_mod(KeyCode::Right, KeyModifiers::ALT | KeyModifiers::SHIFT);
    assert_eq!(pane_width(&h, "shell"), w + 2);
    let seam = h.area(TREE.sub("seam").child(0));
    h.mouse(MouseKind::Down, seam.x, seam.y + 3);
    h.mouse(MouseKind::Drag, seam.x + 5, seam.y + 3);
    h.mouse(MouseKind::Up, seam.x + 5, seam.y + 3);
    assert_eq!(pane_width(&h, "shell"), w + 7);
}

#[test]
fn pane_tree_split_and_close_change_the_panes() {
    let mut h = Harness::new(120, 40, PageId::PaneTree);
    h.click_id(button("split-right"));
    assert!(h.text().contains("4 panes · focus editor"), "{}", h.text());
    h.click_id(button("close"));
    assert!(h.text().contains("3 panes"), "{}", h.text());
    h.opt("panetree.layout=single");
    h.click_id(button("close"));
    assert!(h.status().contains("one pane"), "{}", h.status());
}

#[test]
fn pane_tree_options_drive_the_tree() {
    let mut h = Harness::new(120, 40, PageId::PaneTree);
    h.opt("panetree.layout=grid");
    assert!(h.text().contains("4 panes"), "{}", h.text());
    h.opt("panetree.frames=never");
    assert!(!h.text().contains("╭─ shell"), "{}", h.text());
    h.opt("panetree.frames=when-split");
    h.opt("panetree.layout=single");
    assert!(!h.text().contains("╭─ shell"), "a lone pane is unframed");
    h.opt("panetree.frames=always");
    assert!(h.text().contains("╭─ shell"));
    h.opt("panetree.keys=custom");
    assert!(h.text().contains("Ctrl+N next pane"), "{}", h.text());
    h.opt("panetree.keys=none");
    assert!(h.text().contains("no pane keys"), "{}", h.text());
}

#[test]
fn pane_tree_states_grid_lists_its_rows() {
    let mut h = Harness::new(120, 40, PageId::PaneTree);
    h.section(Section::States);
    let text = h.text();
    for row in ["rest", "hover", "focused", "selected"] {
        assert!(text.contains(row), "{row}: {text}");
    }
}

#[test]
fn pane_tree_reads_at_80x24() {
    let h = Harness::new(80, 24, PageId::PaneTree);
    let text = h.text();
    assert!(
        text.contains("╭─ shell") && text.contains("Split right"),
        "{text}"
    );
}
