#![allow(
    dead_code,
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    clippy::arithmetic_side_effects,
    clippy::doc_markdown,
    clippy::indexing_slicing,
    clippy::many_single_char_names,
    clippy::missing_panics_doc,
    clippy::panic,
    clippy::too_many_lines,
    clippy::unwrap_used,
    clippy::expect_used
)]

use jackin_preview_app::domain::usage::Freshness;
use jackin_preview_app::{Motion, Route, Scenario};
use support::H;
use termrock::KeyCode;

mod support;

#[test]
fn usage_initial_down_advances_tab_without_changing_selected_account() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 120, 40);
    let personal = h
        .app()
        .world
        .accounts
        .get("acct-claude-personal")
        .expect("normal fixture personal account");
    assert!(personal.default_for_provider);

    h.key(KeyCode::Char('u'));
    assert_eq!(h.app().route(), Route::Usage);
    assert_eq!(h.app().usage.selected(), Some("acct-claude-personal"));
    let selected = h.app().usage.selected().map(str::to_owned);

    h.key(KeyCode::Down);
    assert_eq!(
        h.app().usage.tab,
        jackin_preview_app::screens::usage::Tab::Registration
    );
    assert_eq!(h.app().usage.selected(), selected.as_deref());
}

#[test]
fn selected_nonfirst_work_account_detail_includes_its_window_and_freshness() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 120, 40);
    let personal = h
        .app()
        .world
        .accounts
        .get("acct-claude-personal")
        .expect("normal fixture personal account");
    let work = h
        .app()
        .world
        .accounts
        .get("acct-claude-work")
        .expect("normal fixture work account");
    assert!(personal.default_for_provider);
    assert!(!work.default_for_provider);
    assert_eq!(
        h.app()
            .world
            .accounts
            .sorted()
            .first()
            .map(|account| account.id.as_str()),
        Some("acct-claude-personal")
    );

    // This isolates the Stage 1 renderer from the separately tested account
    // selection journey. Route, tab, detail and viewport use real key input.
    h.app_mut().usage.select(Some("acct-claude-work"));
    h.key(KeyCode::Char('u'));
    assert_eq!(h.app().route(), Route::Usage);
    assert_eq!(h.app().usage.selected(), Some("acct-claude-work"));
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);

    assert!(h.app().usage.detail_open());
    assert_eq!(h.app().usage.selected(), Some("acct-claude-work"));
    let account = h
        .app()
        .world
        .accounts
        .get("acct-claude-work")
        .expect("work account remains in the model");
    let session = account
        .usage
        .windows
        .iter()
        .find(|window| window.id == "session")
        .expect("work account Session window");
    assert_eq!(session.label, "Session · 5-hour");
    assert_eq!(session.used_pct, Some(76));
    assert_eq!(account.usage.freshness.phase, Freshness::Stale);
    assert_eq!(
        h.app().world.now_secs()
            - account
                .usage
                .freshness
                .last_good_secs
                .expect("last-good timestamp"),
        47 * 60
    );

    let detail = h.text();
    assert!(detail.contains("Claude · Work"), "{detail}");
    assert!(detail.contains("Session · 5-hour"), "{detail}");
    assert!(detail.contains("76%"), "{detail}");
    assert!(detail.contains("stale · last good 47 min ago"), "{detail}");
    assert!(detail.contains("Retry        resets in 13 min"), "{detail}");
}
