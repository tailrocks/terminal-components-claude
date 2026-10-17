//! The Settings and Task runner screens (plan §7.5, SC27), driven through
//! the real app: the dirty mark on the Settings tab, and the quit guard
//! while the pipeline runs.

use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use crate::app::PageId;
use crate::app_tests_tabs::Harness;

#[test]
fn an_unsaved_settings_section_marks_its_tab_until_saved() {
    let mut h = Harness::on(120, 40, PageId::Settings);
    let dirty = junie_tui::theme::Theme::junie().glyphs.dirty;
    assert!(!h.text().contains(&format!("General {dirty}")));
    // the section strip, the screen's tabs, name, description, visibility,
    // then the auto-merge toggle
    for _ in 0..6 {
        h.key(KeyCode::Tab);
    }
    assert_eq!(
        h.app.shell.focus.current(),
        Some(junie_tui::core::id::WidgetId::of("settings").sub("automerge"))
    );
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
    // Ctrl+Q quits from anywhere (QuitKeys)
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

/// Settings on the template: its options open a section, rebuild the
/// members table and the environment list through their builders, drop the
/// dirty mark and the save confirmation; States draws the section tabs,
/// the members table and the environment list.
#[test]
fn the_settings_options_and_states() {
    use crate::pages::template::Section;
    let mut h = Harness::on(120, 40, PageId::Settings);
    h.opt("settings.tab=members");
    assert!(h.text().contains("Mira Okafor"), "{}", h.text());
    h.opt("settings.tab=environment");
    assert!(h.text().contains("DATABASE_URL"));
    // multi select: Space checks the cursor row
    let env = junie_tui::core::id::WidgetId::of("settings").sub("env");
    h.focus(env);
    h.key(KeyCode::Char(' '));
    assert!(h.text().contains("1 selected"), "{}", h.text());
    // single select: the list is rebuilt through its builder, no checks
    h.opt("settings.env_select=single");
    h.focus(env);
    h.key(KeyCode::Char(' '));
    assert!(h.text().contains("0 selected"), "{}", h.text());
    // without the confirmation, Ctrl+S on a dirty form saves at once
    let mut h = Harness::on(120, 40, PageId::Settings);
    h.opt("settings.save_confirm=off");
    h.opt("settings.tab_dirty=off");
    for _ in 0..6 {
        h.key(KeyCode::Tab);
    }
    h.key(KeyCode::Char(' '));
    let dirty = junie_tui::theme::Theme::junie().glyphs.dirty;
    assert!(h.text().contains("General · unsaved"));
    assert!(
        !h.text().contains(&format!("General {dirty}")),
        "no dirty mark on the tab"
    );
    let save = junie_tui::core::id::WidgetId::of("settings").sub("save");
    h.focus(save);
    h.key(KeyCode::Enter);
    assert!(!h.text().contains("Save settings?"), "no confirmation");
    assert!(!h.text().contains("General · unsaved"), "saved");
    h.section(Section::States);
    let text = h.text();
    for col in ["Section tabs", "Members", "Environment"] {
        assert!(text.contains(col), "{col}\n{text}");
    }
}

/// The Task runner on the template: its options rebuild the tree and the
/// log through their builders, r runs the pipeline from the section strip,
/// the pipeline keeps running while another section shows, and States
/// draws the tree, the log and a task's bar.
#[test]
fn the_task_runner_options_states_and_run_from_the_strip() {
    use crate::pages::template::Section;
    let mut h = Harness::on(120, 40, PageId::TaskRunner);
    assert!(h.text().contains("◆ production"), "{}", h.text());
    assert!(h.text().contains("integration"));
    h.opt("taskrunner.env_glyphs=off");
    assert!(!h.text().contains("◆ production"), "{}", h.text());
    h.opt("taskrunner.expanded=off");
    assert!(!h.text().contains("staging"), "folded: {}", h.text());
    h.opt("taskrunner.expanded=on");
    h.opt("taskrunner.tree_meta=on");
    let (_, y) = h.find("deploy").expect("the deploy folder");
    assert!(h.row(y).contains('2'), "its task count: {}", h.row(y));
    // r from the section strip runs the pipeline
    h.key(KeyCode::Tab);
    h.key(KeyCode::Char('r'));
    assert!(h.text().contains("Pipeline · running"), "{}", h.text());
    h.section(Section::States);
    for _ in 0..30 {
        junie_tui::runtime::Application::handle(&mut h.app, junie_tui::core::event::Input::Tick);
    }
    h.section(Section::Playground);
    assert!(h.text().contains("compile started"), "it ran meanwhile");
    h.section(Section::States);
    let text = h.text();
    for col in ["Targets", "Log", "Task bar"] {
        assert!(text.contains(col), "{col}\n{text}");
    }
}
