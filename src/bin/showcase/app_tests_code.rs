//! The Text & code pages through the real app (plan §7 SC24 Code editor,
//! SC25 Diff and Terminal): options set through `--opt` change the
//! widget, the States rows render, and the key interactions work.

use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use junie_tui::core::id::WidgetId;
use junie_tui::theme::Theme;

use crate::app::PageId;
use crate::app_tests_feedback::Harness;
use crate::pages::template::Section;

// ------------------------------------------------------------- Code editor

fn code() -> WidgetId {
    WidgetId::of("editor").sub("code")
}

fn editor(opts: &[&str]) -> Harness {
    let mut h = Harness::new(140, 45, PageId::Editor);
    for o in opts {
        h.opt(o);
    }
    // focus starts an edit (the theme's `EditEntry::Immediate`); Esc ends
    // it, so each test starts reading
    h.focus(code());
    h.key(KeyCode::Esc);
    h
}

fn text(h: &Harness) -> String {
    h.app.pages[PageId::Editor.index()]
        .as_any()
        .and_then(|a| a.downcast_ref::<crate::pages::editor::EditorPage>())
        .map(|p| p.text().to_owned())
        .expect("the editor page")
}

#[test]
fn code_read_only_never_starts_an_edit() {
    let mut h = editor(&["codeeditor.read_only=on"]);
    h.key(KeyCode::Char('i'));
    assert!(h.text().contains("Mode         navigating"), "{}", h.text());
    assert!(h.text().contains("read-only"));
    let mut h = editor(&[]);
    h.key(KeyCode::Char('i'));
    assert!(h.text().contains("Mode         editing"));
}

#[test]
fn code_tab_leaves_moves_focus_out_while_editing() {
    for (value, leaves) in [("off", false), ("on", true)] {
        let mut h = editor(&[&format!("codeeditor.tab_leaves={value}")]);
        h.key(KeyCode::Char('i'));
        h.key(KeyCode::Tab);
        assert_eq!(h.app.focus.current() != Some(code()), leaves, "{value}");
    }
}

#[test]
fn code_esc_reverts_drops_the_edit() {
    for (value, kept) in [("off", true), ("on", false)] {
        let mut h = editor(&[&format!("codeeditor.esc_reverts={value}")]);
        h.key(KeyCode::Char('i'));
        h.key(KeyCode::Char('x'));
        h.key(KeyCode::Esc);
        assert_eq!(text(&h).starts_with("x//"), kept, "{value}");
    }
}

#[test]
fn code_custom_keymap_moves_find_to_ctrl_f() {
    let mut h = editor(&[]);
    h.key(KeyCode::Char('/'));
    assert!(h.text().contains("type to find"), "{}", h.text());
    let mut h = editor(&["codeeditor.keymap=custom"]);
    h.key(KeyCode::Char('/'));
    assert!(h.text().contains("Find         closed"));
    h.key_mod(KeyCode::Char('f'), KeyModifiers::CONTROL);
    assert!(h.text().contains("type to find"), "{}", h.text());
}

#[test]
fn code_nav_vim_moves_with_j() {
    let mut h = editor(&[]);
    h.key(KeyCode::Char('j'));
    assert!(h.text().contains("ln 1 · col 1"));
    let mut h = editor(&["codeeditor.nav=vim"]);
    h.key(KeyCode::Char('j'));
    assert!(h.text().contains("ln 2 · col 1"), "{}", h.text());
}

#[test]
fn code_highlighter_and_segmenter_options_switch_tones_and_blocks() {
    let t = Theme::junie();
    let mut h = editor(&[]);
    let (x, y) = h.at("pub async fn fetch");
    let toned = h.cell(x, y).fg;
    assert!(h.text().contains("3 blocks"));
    h.opt("codeeditor.highlighter=off");
    let (x, y) = h.at("pub async fn fetch");
    assert_ne!(h.cell(x, y).fg, toned, "the keyword loses its tone");
    assert_eq!(h.cell(x, y).fg, t.text_primary);
    h.opt("codeeditor.segmenter=off");
    assert!(h.text().contains("0 blocks"), "{}", h.text());
}

#[test]
fn code_empty_document_shows_the_placeholder() {
    let h = editor(&["codeeditor.document=empty"]);
    assert!(h.text().contains("Type code. i edits"), "{}", h.text());
}

#[test]
fn code_completion_pairs_a_call_and_follows_its_options() {
    let mut h = editor(&["codeeditor.document=empty"]);
    h.key(KeyCode::Char('i'));
    h.key(KeyCode::Char('f'));
    h.key(KeyCode::Char('e'));
    assert!(h.text().contains("ƒ fetch("), "{}", h.text());
    h.key(KeyCode::Enter);
    assert_eq!(text(&h), "fetch()");
    // four rows at most, and above the line when asked
    let mut h = editor(&[
        "codeeditor.document=empty",
        "codeeditor.completion_rows=4",
        "codeeditor.completion_placement=above",
    ]);
    h.key(KeyCode::Char('i'));
    h.key_mod(KeyCode::Char(' '), KeyModifiers::CONTROL);
    assert!(h.find("T Option<T>").is_some(), "{}", h.text());
    assert!(
        h.find("T Result<T, E>").is_none(),
        "four rows: {}",
        h.text()
    );
    let (_, caret_y) = h.at("1 Type code");
    let (_, popup_y) = h.at("T Body");
    assert!(popup_y < caret_y, "above the line: {}", h.text());
}

#[test]
fn code_find_button_opens_find_and_counts_matches() {
    let mut h = editor(&[]);
    // from elsewhere on the page: the button hands focus to the editor
    h.focus(WidgetId::of("editor").sub("run"));
    let (x, y) = h.at("Find ");
    h.click(x + 1, y);
    assert_eq!(h.app.focus.current(), Some(code()));
    for c in "delay".chars() {
        h.key(KeyCode::Char(c));
    }
    assert!(h.text().contains("“delay” · 1 of 3"), "{}", h.text());
}

#[test]
fn code_run_button_marks_the_running_block() {
    let mut h = editor(&[]);
    h.key(KeyCode::Char('}'));
    let (x, y) = h.at("Run block");
    h.click(x + 1, y);
    assert!(h.text().contains("running"), "{}", h.text());
    for _ in 0..12 {
        h.tick();
    }
    assert!(h.status().starts_with("Block ran in"), "{}", h.status());
}

#[test]
fn code_states_grid_shows_every_editor_state() {
    let mut h = Harness::new(140, 45, PageId::Editor);
    h.section(Section::States);
    let text = h.text();
    for label in [
        "Tones",
        "Plain",
        "Warning",
        "rest",
        "editing",
        "disabled",
        "busy",
        "error",
        "empty",
        "overflow",
        "unused variable",
        "expected an item",
    ] {
        assert!(text.contains(label), "{label}: {text}");
    }
}

// -------------------------------------------------------------------- Diff

#[test]
fn diff_follows_the_changes_list_cursor() {
    let mut h = Harness::new(140, 45, PageId::Diff);
    assert!(h.text().contains("- attempts = 3"));
    h.focus(WidgetId::of("diff").sub("files"));
    h.key(KeyCode::Down);
    assert!(h.text().contains("+ # Retries"), "{}", h.text());
    let (x, y) = h.at("D src/legacy");
    h.click(x + 2, y);
    assert!(h.text().contains("- pub fn linear"), "{}", h.text());
    h.key(KeyCode::Down);
    assert!(
        h.text().contains("+ use super::exponential;"),
        "{}",
        h.text()
    );
}

#[test]
fn diff_options_switch_mode_file_and_width() {
    let mut h = Harness::new(140, 45, PageId::Diff);
    assert!(!h.text().contains("● Review"));
    h.opt("diff.mode=review");
    assert!(h.text().contains("● Review"), "{}", h.text());
    h.opt("diff.file=none");
    assert!(h.text().contains("No file selected"));
    h.opt("diff.file=changes");
    let (cx, cy) = h.at("─ Changes");
    let (dx, dy) = h.at("─ Diff");
    assert!(dx > cx && dy == cy, "side by side");
    h.opt("diff.width=narrow");
    let (cx, cy) = h.at("─ Changes");
    let (dx, dy) = h.at("─ Diff");
    assert!(dy > cy && dx == cx, "stacked when narrow: {}", h.text());
}

#[test]
fn diff_states_grid_shows_unified_and_review() {
    let mut h = Harness::new(140, 45, PageId::Diff);
    h.section(Section::States);
    let text = h.text();
    for label in [
        "Unified",
        "Review",
        "rest",
        "focused",
        "empty",
        "narrow",
        "No file selected",
    ] {
        assert!(text.contains(label), "{label}: {text}");
    }
}

// ---------------------------------------------------------------- Terminal

fn term_id() -> WidgetId {
    WidgetId::of("terminal").sub("term")
}

#[test]
fn terminal_spans_carry_their_styles() {
    use ratatui::style::Modifier;
    let h = Harness::new(140, 45, PageId::Terminal);
    let has = |needle: &str, m: Modifier| {
        let (x, y) = h.at(needle);
        h.cell(x, y).modifier.contains(m)
    };
    assert!(has("payments-platform ❯", Modifier::BOLD));
    assert!(has("dev ·", Modifier::ITALIC));
    assert!(has("jackin.toml", Modifier::UNDERLINED));
    // reversed: canvas ink on the primary text colour, a cursor block
    let (x, y) = h.at("LIVE");
    let t = Theme::junie();
    assert_eq!(
        (h.cell(x, y).fg, h.cell(x, y).bg),
        (t.canvas, t.text_primary)
    );
}

#[test]
fn terminal_retention_keeps_the_last_lines() {
    let mut h = Harness::new(140, 45, PageId::Terminal);
    h.opt("terminal.max_lines=20");
    for _ in 0..200 {
        h.tick();
    }
    assert!(h.text().contains("last 20 kept"), "{}", h.text());
    assert!(!h.text().contains("payments-platform ❯ jackin launch"));
}

#[test]
fn terminal_find_marks_highlight_the_match() {
    let mut h = Harness::new(140, 45, PageId::Terminal);
    let (x, y) = h.at("jackin launch");
    let plain = h.cell(x, y).bg;
    h.opt("terminal.mark=on");
    assert_ne!(h.cell(x, y).bg, plain, "the mark fills the match");
}

#[test]
fn terminal_custom_keymap_moves_follow_to_shift_f() {
    let mut h = Harness::new(140, 45, PageId::Terminal);
    h.focus(term_id());
    h.key(KeyCode::Char('f'));
    assert_eq!(h.status(), "Paused at the scrollback position");
    let mut h = Harness::new(140, 45, PageId::Terminal);
    h.opt("terminal.keymap=custom");
    h.focus(term_id());
    h.key(KeyCode::Char('f'));
    assert_eq!(h.status(), "");
    h.key(KeyCode::Char('F'));
    assert_eq!(h.status(), "Paused at the scrollback position");
}

#[test]
fn terminal_rail_numbering_follows_the_option() {
    let mut h = Harness::new(140, 45, PageId::Terminal);
    assert!(h.text().contains("01 Resolve workspace"));
    h.opt("terminal.numbered=off");
    assert!(!h.text().contains("01 Resolve workspace"));
    assert!(h.text().contains("Resolve workspace"));
}

#[test]
fn terminal_states_grid_shows_viewports_and_the_rail() {
    let mut h = Harness::new(140, 45, PageId::Terminal);
    h.section(Section::States);
    let text = h.text();
    for label in [
        "Viewport",
        "Wrapped",
        "Step rail",
        "rest",
        "selected",
        "empty",
        "overflow",
        "narrow",
        "skipped",
        "blocked",
    ] {
        assert!(text.contains(label), "{label}: {text}");
    }
}

