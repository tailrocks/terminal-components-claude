//! Expand remaining SC rows: default frames, Diff absence, executable keys.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "integration assertions"
)]

use oracle_showcase::{
    Action, ActionProgram, ColorSpec, OraclePageId, ShowcaseDriver, TerminalSize, capture_case,
    default_page_frames, expand, rows,
};

#[test]
fn every_sc_base_default_frame_is_observed() {
    for case in default_page_frames().unwrap() {
        let frame = capture_case(case.page, case.size, case.color, &case.program).unwrap();
        assert_eq!(frame.oracle_page, case.page);
        assert_eq!(frame.size, case.size);
        assert_eq!(frame.color, case.color);
        if case.page == OraclePageId::Diff {
            assert!(
                frame.production_route_absent,
                "Diff is oracle-only; do not fabricate a production frame"
            );
            assert!(frame.cells.is_empty());
            continue;
        }
        assert!(!frame.production_route_absent);
        assert_eq!(frame.production_page, Some(case.page));
        assert!(frame.is_complete());
    }
}

#[test]
fn sc_shell_nav_keys_replay_without_find() {
    let row = rows()
        .unwrap()
        .into_iter()
        .find(|row| row.scenario_id == "SC-SHELL-NAV")
        .unwrap();
    let mut driver = ShowcaseDriver::open(
        OraclePageId::Overview,
        TerminalSize::new(80, 24),
        ColorSpec::TrueColor,
    )
    .unwrap();
    for step in &row.program.steps {
        match step {
            Action::Key(key) => driver.key(*key).unwrap(),
            Action::FreshDraw => driver.present().unwrap(),
            Action::Tick => driver.tick().unwrap(),
            // Labeled hits stay fail-closed; keys already prove routing
            // without buffer text search.
            Action::Unfrozen { .. }
            | Action::Ticks(_)
            | Action::Time(_)
            | Action::TimeThenTick(_)
            | Action::Resize(_)
            | Action::Pointer { .. } => {}
        }
    }
    let frame = driver.observe().unwrap();
    assert!(!frame.quit, "Esc in SC-SHELL-NAV must never quit");
    assert!(frame.is_complete());
}

#[test]
fn action_parser_covers_real_tsv_token_shapes() {
    let program = ActionProgram::parse("time4000ms+1ns without Tick;time1800ms+1ns+Tick");
    assert_eq!(program.steps.len(), 2);
    assert!(matches!(program.steps[0], Action::Time(_)));
    assert!(matches!(program.steps[1], Action::TimeThenTick(_)));

    let repeats = ActionProgram::parse("Down*5;Shift+Down*3");
    assert_eq!(repeats.steps.len(), 8);
    assert!(
        repeats
            .steps
            .iter()
            .all(|step| matches!(step, Action::Key(_)))
    );

    let chain = ActionProgram::parse("resize80x24->100x30->120x40");
    assert_eq!(chain.steps.len(), 3);
    assert!(
        chain
            .steps
            .iter()
            .all(|step| matches!(step, Action::Resize(_)))
    );

    let fresh_keys = ActionProgram::parse("fresh Space;fresh-run j");
    assert_eq!(fresh_keys.steps.len(), 2);
    assert!(
        fresh_keys
            .steps
            .iter()
            .all(|step| matches!(step, Action::Key(_)))
    );

    let backspace = ActionProgram::parse("Backspace");
    assert_eq!(backspace.steps.len(), 1);
    assert!(matches!(backspace.steps[0], Action::Key(_)));

    let frozen = ActionProgram::parse("click(Run task);type(-persist);wheel(nav,+1)");
    assert!(
        frozen
            .steps
            .iter()
            .all(|step| matches!(step, Action::Unfrozen { .. }))
    );
}

#[test]
fn resize_chain_and_key_repeats_execute() {
    let mut driver = ShowcaseDriver::open(
        OraclePageId::Overview,
        TerminalSize::new(80, 24),
        ColorSpec::TrueColor,
    )
    .unwrap();
    driver
        .execute(&ActionProgram::parse(
            "resize80x24->100x30->120x40;Down*5;Up*5",
        ))
        .unwrap();
    let frame = driver.observe().unwrap();
    assert_eq!(frame.size, TerminalSize::new(120, 40));
    assert!(frame.is_complete());
    assert!(!frame.quit);
}

#[test]
fn expansion_retains_every_tsv_row() {
    let rows = rows().unwrap();
    let cases = expand().unwrap();
    for row in &rows {
        assert!(
            cases.iter().any(|case| case.scenario_id == row.scenario_id),
            "missing expansion for {}",
            row.scenario_id
        );
    }
}

#[test]
fn production_cli_page_parser_is_observed_not_repaired() {
    assert!(showcase_app::PageId::from_name("overview").is_some());
    assert!(showcase_app::PageId::from_name("diff").is_none());
    assert!(showcase_app::PageId::from_name("codeeditor").is_some());
    assert!(showcase_app::PageId::from_name("datagrid").is_some());
    assert!(showcase_app::PageId::from_name("chipsselects").is_some());
    assert!(showcase_app::PageId::from_name("editabletables").is_some());
    for page in OraclePageId::ALL {
        match page {
            OraclePageId::Diff => assert!(page.production().is_none()),
            _ => assert!(
                page.production().is_some(),
                "production missing {}",
                page.slug()
            ),
        }
    }
}
