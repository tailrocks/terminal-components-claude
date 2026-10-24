use jackin_preview_app::screens::manager::ManagerRowKey;
use jackin_preview_app::{ACCOUNTS, ACCOUNTS_LIST, App, MANAGER, Motion, Route, Scenario};
use termrock::{ColorLevel, ItemKey, KeyCode, Rect, Theme};
use termrock_test_support::Harness;

const WIDTH: u16 = 120;
const HEIGHT: u16 = 40;

fn returning_paper(level: ColorLevel) -> Harness<App> {
    Harness::new(
        App::for_scenario(Scenario::Returning, Motion::Paused),
        Theme::paper(),
        WIDTH,
        HEIGHT,
    )
    .with_color(level)
}

fn cell_text(
    harness: &Harness<App>,
    x: u16,
    y: u16,
    width: u16,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut text = String::new();
    for column in x..x.saturating_add(width) {
        let cell = harness
            .buffer()
            .cell((column, y))
            .ok_or("expected an in-bounds frame cell")?;
        text.push_str(cell.symbol());
    }
    Ok(text)
}

fn assert_text_at(
    harness: &Harness<App>,
    x: u16,
    y: u16,
    expected: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let width = u16::try_from(expected.chars().count())?;
    assert_eq!(
        cell_text(harness, x, y, width)?,
        expected,
        "text at ({x}, {y})"
    );
    Ok(())
}

fn assert_clean_diagnostics(harness: &Harness<App>, action: &str) {
    assert!(
        harness.diagnostics().is_empty(),
        "{action} emitted diagnostics: {:?}",
        harness.diagnostics()
    );
}

fn assert_accounts_route_entry(level: ColorLevel, key: KeyCode, action: &str) {
    let mut harness = returning_paper(level);
    let account_count = harness.app().world.accounts.accounts.len();
    let account_revision = harness.app().world.accounts.revision;

    let response = harness.key(key);
    assert!(response.is_changed(), "{action} should change route");
    assert_eq!(harness.app().route(), Route::Accounts, "{action} route");
    assert_eq!(harness.focus(), Some(ACCOUNTS_LIST), "{action} focus");
    assert!(!harness.app().accounts.form_open, "{action} opened a form");
    assert_eq!(
        harness.app().world.accounts.accounts.len(),
        account_count,
        "{action} changed the account count"
    );
    assert_eq!(
        harness.app().world.accounts.revision,
        account_revision,
        "{action} changed account data"
    );
    assert_clean_diagnostics(&harness, action);

    for frame in 0..2 {
        harness.draw();
        assert_eq!(
            harness.app().route(),
            Route::Accounts,
            "{action} frame {frame}"
        );
        assert_eq!(
            harness.focus(),
            Some(ACCOUNTS_LIST),
            "{action} frame {frame} focus"
        );
        assert!(
            !harness.app().accounts.form_open,
            "{action} frame {frame} opened a form"
        );
        assert_eq!(
            harness.app().world.accounts.accounts.len(),
            account_count,
            "{action} frame {frame} changed the account count"
        );
        assert_eq!(
            harness.app().world.accounts.revision,
            account_revision,
            "{action} frame {frame} changed account data"
        );
        assert_clean_diagnostics(&harness, action);
    }
}

fn assert_current_saved_frame(
    harness: &mut Harness<App>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Match the retained normal observation: current App, Paper theme, fixed
    // capability, then an ordinary draw. This is not the historical renderer
    // or the repository visual-baseline target.
    harness.draw();
    assert_eq!(harness.app().route(), Route::Manager);
    assert_eq!(harness.focus(), Some(MANAGER));
    assert_eq!(harness.cursor(), None);
    assert!(!harness.cursor_bar());
    assert_eq!(harness.app().world.scenario, Scenario::Returning);
    assert_eq!(harness.app().world.running_count(), 2);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    assert_eq!(harness.buffer().area, Rect::new(0, 0, WIDTH, HEIGHT));

    // Brand, current saved-workspace card, separator, tree row, action buttons,
    // and every one of the nine currently published hints have literal cells.
    assert_text_at(harness, 73, 0, "Workspaces")?;
    assert_text_at(harness, 85, 0, "inside the Construct")?;
    assert_text_at(harness, 107, 0, "●")?;
    assert_text_at(harness, 109, 0, "2 running")?;
    assert_text_at(harness, 42, 2, "Current directory · payments-platform")?;
    assert_text_at(harness, 102, 2, "saved workspace")?;
    assert_eq!(cell_text(harness, 42, 3, 75)?, "─".repeat(75));
    assert_text_at(harness, 5, 4, "▸")?;
    assert_text_at(harness, 7, 4, "payments-platform")?;
    assert_text_at(harness, 42, 4, "Working dir")?;
    assert_text_at(harness, 56, 4, "/workspace/payments-platform")?;
    assert_text_at(harness, 41, 36, " ")?;
    assert_text_at(harness, 42, 36, "Launch")?;
    assert_text_at(harness, 51, 36, " ")?;
    assert_text_at(harness, 52, 36, "Edit")?;

    for (key, label, x) in [
        ("Enter", "Launch", 1),
        ("n", "New", 15),
        ("e", "Edit", 22),
        ("Tab", "Details", 30),
        ("c", "Accounts", 43),
        ("u", "Usage", 55),
        ("s", "Settings", 64),
        ("?", "Help", 76),
        ("q", "Quit", 84),
    ] {
        assert_text_at(harness, x, 39, key)?;
        let label_x = x
            .checked_add(u16::try_from(key.len())?)
            .and_then(|label_x| label_x.checked_add(1))
            .ok_or("footer label coordinate overflow")?;
        assert_text_at(harness, label_x, 39, label)?;
    }
    Ok(())
}

fn assert_world_backed_repaint(level: ColorLevel) -> Result<(), Box<dyn std::error::Error>> {
    let mut harness = returning_paper(level);
    let original = harness
        .app()
        .world
        .workspaces
        .iter()
        .find(|workspace| workspace.name == "payments-platform")
        .ok_or("Returning World lacks the pinned payments workspace")?
        .workdir
        .clone();
    assert_eq!(original, "/workspace/payments-platform");
    assert_text_at(&harness, 56, 4, &original)?;

    let changed = "/workspace/paper-live-model-probe";
    let workspace = harness
        .app_mut()
        .world
        .workspaces
        .iter_mut()
        .find(|workspace| workspace.name == "payments-platform")
        .ok_or("Returning World lacks the pinned payments workspace")?;
    changed.clone_into(&mut workspace.workdir);
    harness.draw();

    assert_eq!(harness.app().route(), Route::Manager);
    assert_text_at(&harness, 56, 4, changed)?;
    assert!(!cell_text(&harness, 56, 4, 64)?.contains(original.as_str()));
    assert_clean_diagnostics(&harness, "World-backed repaint");
    Ok(())
}

fn assert_launch_command(level: ColorLevel) -> Result<(), Box<dyn std::error::Error>> {
    // Enter remains a real launch/reconnect action after the normal tree setup.
    let mut launch = returning_paper(level);
    assert_eq!(
        launch.focus(),
        Some(MANAGER),
        "the shell owns initial focus"
    );
    let workspace = launch
        .app()
        .world
        .workspaces
        .first()
        .ok_or("Returning World has no saved workspace")?
        .id;
    let _ = launch.key(KeyCode::Home);
    assert_eq!(
        launch.focus(),
        Some(MANAGER),
        "Home leaves shell focus intact"
    );
    assert_eq!(launch.app().manager.selected(), Some(workspace));
    let expand = launch.key(KeyCode::Right);
    assert!(expand.is_changed(), "Right expands the selected workspace");
    assert!(launch.app().manager.is_expanded(workspace));
    let selected_key = ManagerRowKey::Workspace(workspace).stable_key();
    assert_eq!(
        launch.app().manager.list.cursor(),
        Some(ItemKey::text(&selected_key)),
        "the pre-Enter List cursor still identifies the selected workspace"
    );
    assert_eq!(launch.focus(), Some(MANAGER), "Enter starts at shell focus");
    let enter = launch.key(KeyCode::Enter);
    assert!(enter.is_changed(), "Enter opens the picker or reconnects");
    assert!(
        launch.app().route() == Route::Capsule || launch.text().contains("Launch · choose Agent"),
        "Enter did not reconnect or open the launch picker: {}",
        launch.text()
    );
    assert!(launch.app().manager.is_expanded(workspace));
    assert_clean_diagnostics(&launch, "Enter Launch");
    Ok(())
}

fn assert_new_and_edit_commands(level: ColorLevel) {
    let mut new = returning_paper(level);
    let workspace_count = new.app().world.workspaces.len();
    let _ = new.key(KeyCode::Char('n'));
    assert_eq!(new.app().route(), Route::Prelude);
    assert_eq!(new.app().world.workspaces.len(), workspace_count);
    assert_clean_diagnostics(&new, "n New");
    let _ = new.key(KeyCode::Esc);
    assert_eq!(new.app().route(), Route::Manager);

    let mut edit = returning_paper(level);
    let _ = edit.key(KeyCode::Char('e'));
    assert_eq!(edit.app().route(), Route::Editor);
    assert_eq!(edit.app().editor.pending.name, "payments-platform");
    assert_clean_diagnostics(&edit, "e Edit");
}

fn assert_manager_detail_command(level: ColorLevel) {
    let mut details = returning_paper(level);
    let _ = details.key(KeyCode::Tab);
    assert_eq!(details.app().route(), Route::Manager);
    assert_eq!(details.focus(), Some(ACCOUNTS));
    assert!(!details.app().manager.detail_open());
    assert!(details.text().contains("Tab Details"), "{}", details.text());
    assert!(
        !details.text().contains("Tab Actions"),
        "{}",
        details.text()
    );
    assert_clean_diagnostics(&details, "Tab Details");
    let _ = details.key(KeyCode::Esc);
    assert_eq!(details.app().route(), Route::Manager);
    assert!(details.app().should_quit());
    assert_clean_diagnostics(&details, "Esc from the Paper manager");
}

fn assert_accounts_and_usage_commands(level: ColorLevel) {
    assert_accounts_route_entry(level, KeyCode::Char('a'), "a Accounts");
    assert_accounts_route_entry(level, KeyCode::Char('c'), "c Accounts");

    let mut usage = returning_paper(level);
    let selected_account = usage
        .app()
        .world
        .accounts
        .sorted()
        .first()
        .map(|account| account.id.clone());
    let _ = usage.key(KeyCode::Char('u'));
    assert_eq!(usage.app().route(), Route::Usage);
    assert_eq!(
        usage.focus(),
        Some(jackin_preview_app::screens::usage::TABS)
    );
    assert_eq!(usage.app().usage.selected(), selected_account.as_deref());
    assert!(!usage.app().usage.detail_open());
    assert!(
        usage.text().contains("Usage · read-only"),
        "{}",
        usage.text()
    );
    assert_clean_diagnostics(&usage, "u Usage");
    for frame in 0..2 {
        usage.draw();
        assert_eq!(usage.app().route(), Route::Usage, "u Usage frame {frame}");
        assert_eq!(
            usage.focus(),
            Some(jackin_preview_app::screens::usage::TABS),
            "u Usage frame {frame} focus"
        );
        assert_eq!(usage.app().usage.selected(), selected_account.as_deref());
        assert!(!usage.app().usage.detail_open());
        assert_clean_diagnostics(&usage, "u Usage");
    }
}

fn assert_settings_command(level: ColorLevel) {
    let mut settings = returning_paper(level);
    let _ = settings.key(KeyCode::Char('s'));
    assert_eq!(settings.app().route(), Route::Settings);
    assert_eq!(settings.app().settings.tab, 0);
    assert!(!settings.app().settings.dirty);
    assert!(
        settings
            .text()
            .lines()
            .any(|line| line.trim() == "Settings"),
        "{}",
        settings.text()
    );
    assert!(
        settings
            .text()
            .contains("▸ General  Mounts  Environments  Agents  Trust"),
        "{}",
        settings.text()
    );
    assert!(
        settings.text().contains("Save settings"),
        "{}",
        settings.text()
    );
    assert!(
        settings.text().contains("Trust local incident role"),
        "{}",
        settings.text()
    );
    assert_clean_diagnostics(&settings, "s Settings");
}

fn assert_help_and_quit_commands(level: ColorLevel) {
    let mut help = returning_paper(level);
    let _ = help.key(KeyCode::Char('?'));
    assert_eq!(help.app().route(), Route::Manager);
    assert!(
        help.text().contains("Keyboard shortcuts"),
        "{}",
        help.text()
    );
    assert_clean_diagnostics(&help, "? Help");
    let _ = help.key(KeyCode::Esc);
    assert_eq!(help.app().route(), Route::Manager);
    assert!(
        !help.text().contains("Keyboard shortcuts"),
        "{}",
        help.text()
    );

    let mut quit = returning_paper(level);
    let _ = quit.key(KeyCode::Char('q'));
    assert_eq!(quit.app().route(), Route::Manager);
    assert!(!quit.app().should_quit());
    assert!(quit.text().contains("Exit jackin?"), "{}", quit.text());
    assert!(
        quit.text().contains("2 instances keep running"),
        "{}",
        quit.text()
    );
    assert_clean_diagnostics(&quit, "q Quit");
    let _ = quit.key(KeyCode::Esc);
    assert!(!quit.app().should_quit());
    assert!(!quit.text().contains("Exit jackin?"), "{}", quit.text());
}

fn assert_hint_commands(level: ColorLevel) -> Result<(), Box<dyn std::error::Error>> {
    assert_launch_command(level)?;
    assert_new_and_edit_commands(level);
    assert_manager_detail_command(level);
    assert_accounts_and_usage_commands(level);
    assert_settings_command(level);
    assert_help_and_quit_commands(level);
    Ok(())
}

fn returning120_paper_contract(level: ColorLevel) -> Result<(), Box<dyn std::error::Error>> {
    let mut harness = returning_paper(level);
    assert_current_saved_frame(&mut harness)?;
    assert_world_backed_repaint(level)?;
    assert_hint_commands(level)?;
    Ok(())
}

#[test]
fn returning120_paper_mono_current_normal_contract() -> Result<(), Box<dyn std::error::Error>> {
    returning120_paper_contract(ColorLevel::Mono)
}

#[test]
fn returning120_paper_truecolor_current_normal_contract() -> Result<(), Box<dyn std::error::Error>>
{
    returning120_paper_contract(ColorLevel::TrueColor)
}
