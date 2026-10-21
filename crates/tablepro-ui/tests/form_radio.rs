#[path = "common/tab_navigation.rs"]
mod tab_navigation;

use tablepro_ui::{TableProApp, form_fields};
use termrock::{GlyphRole, Id, KeyCode, Modifier, Theme};
use termrock_test_support::Harness;

const NAME: Id = Id::root("tablepro.connections.form.name");
const DATABASE: Id = Id::root("tablepro.connections.form.database");
const FORM: Id = Id::root("tablepro.connections.form");
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

fn focus_form_control(t: &mut Harness<TableProApp>, id: Id) {
    for _ in 0..96 {
        if t.focus() == Some(id) {
            return;
        }
        let _ = t.key(KeyCode::Tab);
    }
    assert_eq!(t.focus(), Some(id), "form control {id:?} must be reachable");
}

fn row_span(t: &Harness<TableProApp>, y: u16, x0: u16, x1: u16) -> String {
    (x0..=x1).map(|x| t.cell(x, y).symbol()).collect()
}

#[test]
fn form_radio_commit_changes_draft() -> Result<(), &'static str> {
    let mut t = open_new_form(120, 40);
    assert_eq!(drafts(&t)?, (0, 0), "fresh draft must start at index 0");
    assert!(
        tab_navigation::tab_to(&mut t, ENVIRONMENT),
        "ENVIRONMENT must be Tab-reachable"
    );
    let _ = t.key(KeyCode::Down);
    assert_eq!(drafts(&t)?.0, 1, "Down immediately chooses Development");
    let _ = t.key(KeyCode::Down);
    assert_eq!(drafts(&t)?.0, 2, "Down immediately chooses Staging");
    let _ = t.key(KeyCode::Char(' '));
    assert_eq!(drafts(&t)?.0, 2, "Space must commit the cursor");

    assert!(
        tab_navigation::tab_to(&mut t, SAFE_MODE),
        "SAFE_MODE must be Tab-reachable"
    );
    let _ = t.key(KeyCode::Down);
    assert_eq!(drafts(&t)?.1, 1, "Down immediately chooses Safe Mode");
    let _ = t.key(KeyCode::Char(' '));
    assert_eq!(drafts(&t)?.1, 1, "Space must commit the cursor");
    Ok(())
}

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

#[test]
fn form_radio_enter_chooses_without_submitting_and_ctrl_s_saves() -> Result<(), &'static str> {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    let _ = t.ctrl('n');
    assert!(
        t.app().connection_form_open(),
        "ctrl-n must open the connection form"
    );
    assert_eq!(t.focus(), Some(NAME), "fresh form must focus Name");
    assert_eq!(
        drafts(&t)?,
        (0, 0),
        "a new connection starts from the driver's first radio choices"
    );
    assert!(t.find("Name").is_some(), "the form must show Name");
    let _ = t.type_str("Radio contract probe");
    focus_form_control(&mut t, DATABASE);
    assert_eq!(t.focus(), Some(DATABASE), "focus must sit on DATABASE");
    assert!(
        t.area_of(DATABASE).is_some(),
        "DATABASE must be visibly rendered"
    );
    let _ = t.type_str("radio_probe");
    focus_form_control(&mut t, ENVIRONMENT);
    assert_eq!(t.focus(), Some(ENVIRONMENT));
    let draft = t
        .app()
        .connection_draft()
        .ok_or("form must remain open before radio Enter")?;
    assert_eq!(draft.name, "Radio contract probe");
    assert_eq!(draft.database, "radio_probe");
    assert!(
        draft.validate_all().is_ok(),
        "draft errors: {:?}",
        draft.validate_all()
    );
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
        (0, 0),
        "radio defaults remain unchanged after entering valid form data"
    );
    focus_form_control(&mut t, ENVIRONMENT);
    assert_eq!(
        t.focus(),
        Some(ENVIRONMENT),
        "focus must sit on ENVIRONMENT"
    );

    let _ = t.key(KeyCode::Down);
    assert_eq!(drafts(&t)?.0, 1, "Down immediately chooses Development");

    let before = t.app().connections_screen.connections.len();
    let _ = t.key(KeyCode::Enter);
    assert!(
        t.app().connection_form_open(),
        "radio Enter chooses without submitting or closing the form"
    );
    assert_eq!(
        t.app().connections_screen.connections.len(),
        before,
        "radio Enter does not append a connection"
    );
    let save = FORM.part(termrock::Part::ACTIONS).index(2);
    focus_form_control(&mut t, save);
    let _ = t.key(KeyCode::Enter);
    assert_eq!(
        t.app().connections_screen.connections.len(),
        before + 1,
        "Enter on the Save action reaches the typed Save action"
    );
    let _ = t.ctrl('s');
    assert!(
        t.app().connections_screen.connections.len() > before + 1,
        "Ctrl+S reaches the configured Save action"
    );
    let pushed = t
        .app()
        .connections_screen
        .connections
        .last()
        .ok_or("submit must push a connection")?;
    let pushed = format!("{pushed:?}");
    assert!(
        pushed.contains("name: \"Radio contract probe\""),
        "Save keeps the edited name, got {pushed}"
    );
    assert!(
        pushed.contains("database: \"radio_probe\""),
        "Save keeps the edited Database, got {pushed}"
    );
    assert!(
        pushed.contains("environment: Development"),
        "the chosen radio value is saved, got {pushed}"
    );
    Ok(())
}

#[test]
fn fresh_form_reaches_database_field() -> Result<(), &'static str> {
    let mut t = open_new_form(120, 40);
    assert!(
        form_fields().iter().any(|field| field.id == DATABASE),
        "DATABASE must stay declared as tablepro.connections.form.database"
    );
    assert!(
        tab_navigation::tab_to(&mut t, DATABASE),
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

fn option_row(t: &Harness<TableProApp>) -> Result<u16, &'static str> {
    t.find("Environment")
        .map(|(_, y)| y.saturating_add(1))
        .ok_or("Environment caption missing")
}

fn find_from(t: &Harness<TableProApp>, needle: &str, min_y: u16) -> Option<u16> {
    (min_y..t.buffer().area().height).find(|y| t.row(*y).contains(needle))
}

fn marker_x(t: &Harness<TableProApp>, y: u16) -> Result<u16, &'static str> {
    let width = t.buffer().area().width;
    (0..width)
        .find(|x| t.cell(*x, y).symbol() == "(")
        .ok_or("radio marker missing")
}

fn blob_beneath(symbol: &str, fg: &str, bg: &str) -> bool {
    symbol == " " && fg.contains("128, 128, 128") && bg.contains("17, 17, 17")
}

#[test]
fn form_radio_right_card_pins_miss_the_stock_form() -> Result<(), &'static str> {
    let mut t = open_new_form(120, 40);
    let (x, y) = t.find("Environment").ok_or("Environment caption missing")?;
    assert_ne!((x, y), (90, 7), "120x40 caption is not the card pin");
    assert_ne!(row_span(&t, 7, 90, 100), "Environment");
    let row = option_row(&t)?;
    let marker = marker_x(&t, row)?;
    assert_eq!(row_span(&t, row, marker, marker + 2), "(●)");
    assert_eq!(
        row_span(&t, row, marker.saturating_add(4), marker.saturating_add(8)),
        "Local",
        "the first option is title case"
    );
    let staging_y = find_from(&t, "Staging", row).ok_or("Staging option missing")?;
    assert!(staging_y > row, "Staging is a later option row");
    assert_ne!((marker, staging_y), (90, 16), "Staging is not the card pin");
    assert!(
        t.find("development").is_none(),
        "lowercase card labels are absent"
    );
    assert!(
        t.find("Writes run").is_none(),
        "the card description is absent"
    );
    assert!(
        tab_navigation::tab_to(&mut t, SAFE_MODE),
        "SAFE_MODE must be reachable below the environment options"
    );
    let safe = t.find("Safe Mode").ok_or("Safe Mode caption missing")?;
    assert_ne!(safe, (90, 16), "safe caption is not the card pin");
    assert_eq!(
        row_span(
            &t,
            safe.1.saturating_add(1),
            marker,
            marker.saturating_add(5)
        ),
        "(●) Si",
    );

    let wide = open_new_form(160, 50);
    let wide_caption = wide.find("Environment").ok_or("160x50 caption missing")?;
    assert_ne!(wide_caption.0, 95, "160x50 caption is not column 95");
    assert_ne!(row_span(&wide, 7, 95, 105), "Environment");
    assert!(wide.find("Writes run").is_none());
    Ok(())
}

#[test]
fn form_radio_narrow_frames_clip_the_environment_field() {
    for (w, h) in [(100u16, 30u16), (80, 24), (72, 20)] {
        let t = open_new_form(w, h);
        assert!(
            t.find("Environment").is_none(),
            "{w}x{h}: environment caption is below the fold"
        );
        assert!(
            t.find("Writes run").is_none(),
            "{w}x{h}: description is not on the form"
        );
        assert!(
            t.find("Safe Mode").is_none(),
            "{w}x{h}: safe-mode caption is below the fold"
        );
        assert!(t.find("Save").is_some(), "{w}x{h}: the action row stays");
    }
    let eighty = open_new_form(80, 24);
    assert!(
        (70..=75u16).any(|x| {
            let cell = eighty.cell(x, 7);
            !blob_beneath(
                cell.symbol(),
                &format!("{:?}", cell.fg),
                &format!("{:?}", cell.bg),
            )
        }),
        "80x24 caption row is not the muted-on-card tree residue"
    );
    let seventy = open_new_form(72, 20);
    assert!(
        (62..=67u16).any(|x| {
            let cell = seventy.cell(x, 7);
            !blob_beneath(
                cell.symbol(),
                &format!("{:?}", cell.fg),
                &format!("{:?}", cell.bg),
            )
        }),
        "72x20 caption row is not the muted-on-card tree residue"
    );
}

#[test]
fn form_radio_off_marker_uses_the_hollow_circle() -> Result<(), &'static str> {
    let theme = Theme::junie();
    assert_eq!(theme.design.glyphs.get(GlyphRole::RadioOff), "(○)");
    assert_eq!(theme.design.glyphs.get(GlyphRole::RadioOn), "(●)");
    let mut t = open_new_form(120, 40);
    assert!(
        tab_navigation::tab_to(&mut t, ENVIRONMENT),
        "ENVIRONMENT must be Tab-reachable"
    );
    let _ = t.key(KeyCode::Down);
    let _ = t.key(KeyCode::Down);
    let _ = t.key(KeyCode::Char(' '));
    assert_eq!(drafts(&t)?.0, 2, "Space commits Staging");
    let caption = t
        .find("Environment")
        .ok_or("Environment caption missing")?
        .1;
    let local_y = find_from(&t, "Local", caption).ok_or("Local option missing")?;
    let staging_y =
        find_from(&t, "Staging", local_y.saturating_add(1)).ok_or("Staging option missing")?;
    assert!(local_y < staging_y, "Staging is painted below Local");
    let local_marker = marker_x(&t, local_y)?;
    assert_eq!(row_span(&t, local_y, local_marker, local_marker + 2), "(○)");
    assert_eq!(
        t.cell(local_marker.saturating_add(1), local_y).symbol(),
        "○"
    );
    let staging_marker = marker_x(&t, staging_y)?;
    assert_eq!(
        row_span(&t, staging_y, staging_marker, staging_marker + 2),
        "(●)"
    );
    assert_eq!(
        row_span(
            &t,
            staging_y,
            staging_marker.saturating_add(4),
            staging_marker.saturating_add(10)
        ),
        "Staging"
    );
    assert_ne!(row_span(&t, 9, 89, 91), "( )");
    Ok(())
}

#[test]
fn form_radio_focus_bolds_the_painted_row() -> Result<(), &'static str> {
    let mut t = open_new_form(120, 40);
    let row = option_row(&t)?;
    let marker = marker_x(&t, row)?;
    assert!(
        !t.cell(marker, row).modifier.contains(Modifier::BOLD),
        "an unfocused marker is not bold"
    );
    assert!(
        tab_navigation::tab_to(&mut t, ENVIRONMENT),
        "ENVIRONMENT must be Tab-reachable"
    );
    for x in marker..marker.saturating_add(3) {
        assert!(
            t.cell(x, row).modifier.contains(Modifier::BOLD),
            "focused marker x={x} is bold"
        );
    }
    let label = marker.saturating_add(4);
    assert_eq!(t.cell(label, row).symbol(), "L");
    assert!(
        t.cell(label, row).modifier.contains(Modifier::BOLD),
        "focused label is bold"
    );
    let gutter = t.cell(marker.saturating_sub(1), row);
    assert_eq!(gutter.symbol(), "▎", "focus gutter is the focus bar");
    assert!(gutter.modifier.contains(Modifier::BOLD));
    assert_ne!(gutter.fg, gutter.bg, "focus gutter stays visible");
    let (caption_x, caption_y) = t.find("Environment").ok_or("caption")?;
    assert!(
        !t.cell(caption_x, caption_y)
            .modifier
            .contains(Modifier::BOLD),
        "the caption is painted without the field focus flag"
    );
    Ok(())
}

#[test]
fn form_radio_e_does_not_open_the_form() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    let _ = t.key(KeyCode::Char('e'));
    assert!(
        !t.app().connection_form_open(),
        "e must not open the connection form"
    );
}

#[test]
fn form_radio_blob_click_does_not_choose_staging() -> Result<(), &'static str> {
    let mut t = open_new_form(120, 40);
    assert_eq!(drafts(&t)?, (0, 0));
    let _ = t.click(95, 10);
    assert_eq!(drafts(&t)?.0, 0, "the card click must not commit Staging");
    let caption = t.find("Environment").map_or(0, |(_, y)| y);
    assert!(
        find_from(&t, "Staging", caption).is_some(),
        "Staging stays a painted option"
    );
    Ok(())
}

#[test]
fn form_radio_has_no_is_editing_gate() {
    let src = include_str!("../src/app.rs");
    assert!(
        !src.contains("fn is_editing"),
        "is_editing must not be added to satisfy the card tests"
    );
}
