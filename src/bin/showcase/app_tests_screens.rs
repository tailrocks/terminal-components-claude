use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use crate::app::PageId;
use crate::app_tests_tabs::Harness;

#[test]
fn an_unsaved_settings_section_marks_its_tab_until_saved() {
    let mut h = Harness::on(120, 40, PageId::Settings);
    let dirty = junie_tui::theme::Theme::junie().glyphs.dirty;
    assert!(!h.text().contains(&format!("General {dirty}")));
    for _ in 0..5 {
        h.key(KeyCode::Tab);
    }
    h.key(KeyCode::Char(' '));
    assert!(h.text().contains("General · unsaved"), "{}", h.text());
    assert!(
        h.text().contains(&format!("General {dirty}")),
        "the tab carries the dirty mark"
    );
    h.key_mod(KeyCode::Char('s'), KeyModifiers::CONTROL);
    assert!(!h.text().contains(&format!("General {dirty}")), "saved");
}

#[test]
fn quitting_while_the_pipeline_runs_asks_first() {
    let mut h = Harness::on(120, 40, PageId::TaskRunner);
    let (x, y) = h.find("Run pipeline").expect("the Run button");
    h.click(x + 1, y);

    h.key_mod(KeyCode::Char('q'), KeyModifiers::CONTROL);
    assert!(!h.app.shell.quit, "a running pipeline holds the quit");
    assert!(h.text().contains("Still running"), "{}", h.text());
    h.key(KeyCode::Esc);
    assert!(!h.app.shell.quit, "Esc stays");
    h.key_mod(KeyCode::Char('q'), KeyModifiers::CONTROL);
    let (x, y) = h.find("Stop and leave").expect("the stop button");
    h.click(x + 1, y);
    assert!(h.app.shell.quit, "Stop and leave quits");
}
