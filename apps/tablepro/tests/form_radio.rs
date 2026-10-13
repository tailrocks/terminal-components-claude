//! Connection-form radio assertions adapted to the stock `Form`.
//!
//! The approved geometry, description, focus-recipe, and click pins are not
//! retargeted. This file keeps the checks the public form can satisfy:
//! legacy painter absence, draft commit versus cursor movement, Enter
//! submit, and a fresh form reaching `DATABASE`.

use junie_tui::{Id, KeyCode, Theme};
use junie_tui_testing::Harness;
use tablepro_app::{TableProApp, form_fields};

const NAME: Id = Id::root("tablepro.connections.form.name");
const DATABASE: Id = Id::root("tablepro.connections.form.database");
const ENVIRONMENT: Id = Id::root("tablepro.connections.form.environment");
const GROUP: Id = Id::root("tablepro.connections.form.group");
const SAFE_MODE: Id = Id::root("tablepro.connections.form.safe-mode");

fn open_new_form(w: u16, h: u16) -> Harness<TableProApp> {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), w, h);
    let _ = t.ctrl('n');
    assert!(
        t.app().connection_form_open(),
        "{w}x{h}: ctrl-n must open the connection form"
    );
    t
}

fn drafts(t: &Harness<TableProApp>) -> Result<(usize, usize), &'static str> {
    let draft = t.app().connection_draft().ok_or("form must be open")?;
    Ok((draft.environment, draft.safe_mode))
}

fn production_index(app: &TableProApp) -> Option<usize> {
    app.connections_screen
        .connections
        .iter()
        .position(|connection| format!("{connection:?}").contains("name: \"Production\""))
}

/// Fresh ctrl-n form: arrows move the radio cursor without a draft write,
/// and Space commits that cursor for environment then safe mode.
#[test]
fn form_radio_commit_changes_draft() -> Result<(), &'static str> {
    let mut t = open_new_form(120, 40);
    assert_eq!(drafts(&t)?, (0, 0), "fresh draft must start at index 0");
    assert!(t.tab_to(ENVIRONMENT), "ENVIRONMENT must be Tab-reachable");
    let _ = t.key(KeyCode::Down);
    let _ = t.key(KeyCode::Down);
    assert_eq!(drafts(&t)?.0, 0, "arrows must not write the draft");
    let _ = t.key(KeyCode::Char(' '));
    assert_eq!(drafts(&t)?.0, 2, "Space must commit the cursor");

    assert!(t.tab_to(SAFE_MODE), "SAFE_MODE must be Tab-reachable");
    let _ = t.key(KeyCode::Down);
    assert_eq!(drafts(&t)?.1, 0, "arrows must not write the draft");
    let _ = t.key(KeyCode::Char(' '));
    assert_eq!(drafts(&t)?.1, 1, "Space must commit the cursor");
    Ok(())
}

/// `draw_form_radio` is not part of the app shell.
#[test]
fn form_radio_no_legacy_painter() {
    let src = include_str!("../src/app.rs");
    assert!(
        !src.contains("fn draw_form_radio"),
        "legacy draw_form_radio definition survived"
    );
    assert!(
        !src.contains("draw_form_radio("),
        "legacy draw_form_radio call survived"
    );
}

/// Corpus Production draft: ring order, Space commit, Enter submits the
/// last committed environment and discards the later cursor.
///
/// The shell has no `e` edit binding. Connecting to Production and pressing
/// ctrl-n opens that row through the public form command.
#[test]
fn form_radio_enter_submits() -> Result<(), &'static str> {
    let mut app = TableProApp::default();
    let index = production_index(&app).ok_or("Production connection missing from the demo list")?;
    assert!(app.connect(index), "Production must connect");
    let mut t = Harness::new(app, Theme::junie(), 120, 40);
    let _ = t.ctrl('n');
    assert!(
        t.app().connection_form_open(),
        "ctrl-n must open the connection form"
    );
    assert!(t.find("Name").is_some(), "the form must show Name");
    assert!(t.tab_to(NAME), "NAME must be Tab-reachable");

    let entries = t.ring().entries();
    let env = entries.iter().position(|entry| entry.id == ENVIRONMENT);
    let group = entries.iter().position(|entry| entry.id == GROUP);
    let safe = entries.iter().position(|entry| entry.id == SAFE_MODE);
    assert!(env.is_some(), "ENVIRONMENT must be registered");
    assert!(group.is_some(), "GROUP must be registered");
    assert!(safe.is_some(), "SAFE_MODE must be registered");
    let (Some(env), Some(group), Some(safe)) = (env, group, safe) else {
        return Err("radio stops must be registered");
    };
    assert!(
        entries.get(env).is_some_and(|entry| !entry.disabled),
        "ENVIRONMENT stop must be enabled"
    );
    assert!(
        entries.get(safe).is_some_and(|entry| !entry.disabled),
        "SAFE_MODE stop must be enabled"
    );
    assert!(
        env < group && group < safe,
        "stops must register in draw order ENVIRONMENT < GROUP < SAFE_MODE"
    );
    assert_eq!(
        drafts(&t)?,
        (3, 3),
        "the corpus pick starts Production/Safe"
    );
    assert!(t.tab_to(ENVIRONMENT), "ENVIRONMENT must be Tab-reachable");
    assert_eq!(
        t.focus(),
        Some(ENVIRONMENT),
        "focus must sit on ENVIRONMENT"
    );

    let _ = t.key(KeyCode::Up);
    assert_eq!(drafts(&t)?.0, 3, "Up must not write the draft");
    let _ = t.key(KeyCode::Up);
    assert_eq!(drafts(&t)?.0, 3, "Up must not write the draft");
    let _ = t.key(KeyCode::Char(' '));
    assert_eq!(drafts(&t)?.0, 1, "Space must commit development");
    let _ = t.key(KeyCode::Down);
    assert_eq!(drafts(&t)?.0, 1, "Down must not write the draft");

    let before = t.app().connections_screen.connections.len();
    let _ = t.key(KeyCode::Enter);
    assert!(
        !t.app().connection_form_open(),
        "Enter must submit and close the form"
    );
    assert!(
        t.app().connections_screen.connections.len() > before,
        "submit must push a connection"
    );
    let pushed = t
        .app()
        .connections_screen
        .connections
        .last()
        .ok_or("submit must push a connection")?;
    let pushed = format!("{pushed:?}");
    assert!(
        pushed.contains("name: \"Production\""),
        "pushed connection keeps the corpus name, got {pushed}"
    );
    assert!(
        pushed.contains("environment: Development"),
        "the pushed connection keeps the last Space-committed value, got {pushed}"
    );
    Ok(())
}

/// A fresh form can Tab to the declared DATABASE field and commit into it.
#[test]
fn fresh_form_reaches_database_field() -> Result<(), &'static str> {
    let mut t = open_new_form(120, 40);
    assert!(
        form_fields().iter().any(|field| field.id == DATABASE),
        "DATABASE must stay declared as tablepro.connections.form.database"
    );
    assert!(
        t.tab_to(DATABASE),
        "a fresh form must reach the DATABASE field"
    );
    assert_eq!(t.focus(), Some(DATABASE), "focus must sit on DATABASE");
    assert!(
        t.ring()
            .entries()
            .iter()
            .any(|entry| entry.id == DATABASE && !entry.disabled),
        "DATABASE stop must be enabled"
    );
    let before = t
        .app()
        .connection_draft()
        .map(|draft| draft.database.clone())
        .ok_or("form must be open")?;
    assert!(
        !before.contains('Z'),
        "fixture database must not already contain the probe"
    );
    let _ = t.type_str("Z");
    let _ = t.key(KeyCode::Enter);
    assert!(
        t.app().connection_form_open(),
        "committing DATABASE must not submit the form"
    );
    let after = t
        .app()
        .connection_draft()
        .map(|draft| draft.database.clone())
        .ok_or("form must stay open")?;
    assert!(
        after.contains('Z'),
        "DATABASE must accept input, before {before:?} after {after:?}"
    );
    Ok(())
}
