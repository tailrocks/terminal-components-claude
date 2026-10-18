



use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use crate::app::PageId;
use crate::app_tests_tabs::Harness;

#[test]
fn an_unsaved_settings_section_marks_its_tab_until_saved() {
    let mut h = Harness::on(120, 40, PageId::Settings);
    let dirty = junie_tui::theme::Theme::junie().glyphs.dirty;
    assert!(!h.text().contains(&format!("General {dirty}")));


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





#[test]
fn the_settings_options_and_states() {
    use crate::pages::template::Section;
    let mut h = Harness::on(120, 40, PageId::Settings);
    h.opt("settings.tab=members");
    assert!(h.text().contains("Mira Okafor"), "{}", h.text());
    h.opt("settings.tab=environment");
    assert!(h.text().contains("DATABASE_URL"));

    let env = junie_tui::core::id::WidgetId::of("settings").sub("env");
    h.focus(env);
    h.key(KeyCode::Char(' '));
    assert!(h.text().contains("1 selected"), "{}", h.text());

    h.opt("settings.env_select=single");
    h.focus(env);
    h.key(KeyCode::Char(' '));
    assert!(h.text().contains("0 selected"), "{}", h.text());

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
