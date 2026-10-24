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
    assert_eq!(
        h.focus(),
        Some(jackin_preview_app::screens::usage::TABS),
        "Usage opens with tab focus"
    );
    let selected = h.app().usage.selected().map(str::to_owned);

    h.key(KeyCode::Down);
    assert_eq!(
        h.app().usage.tab,
        jackin_preview_app::screens::usage::Tab::Registration
    );
    assert_eq!(h.app().usage.selected(), selected.as_deref());
}

#[test]
fn normal_usage_journey_tabs_into_accounts_and_opens_selected_work_detail() {
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

    h.key(KeyCode::Char('u'));
    assert_eq!(h.app().route(), Route::Usage);
    assert_eq!(h.app().usage.selected(), Some("acct-claude-personal"));
    h.key(KeyCode::Down);
    assert_eq!(
        h.app().usage.tab,
        jackin_preview_app::screens::usage::Tab::Registration
    );
    assert_eq!(h.app().usage.selected(), Some("acct-claude-personal"));

    h.key(KeyCode::Tab);
    assert_eq!(
        h.focus(),
        Some(jackin_preview_app::screens::usage::LIST),
        "Tab should transfer focus to the registered account list"
    );
    assert!(
        h.harness.diagnostics().is_empty(),
        "after entering account list: {:?}",
        h.harness.diagnostics()
    );
    h.key(KeyCode::Down);
    assert_eq!(
        h.app().usage.tab,
        jackin_preview_app::screens::usage::Tab::Registration,
        "moving in the account list must not change the Usage tab"
    );
    assert_eq!(h.app().usage.selected(), Some("acct-claude-work"));
    assert!(
        h.harness.diagnostics().is_empty(),
        "after moving account cursor: {:?}",
        h.harness.diagnostics()
    );

    h.key(KeyCode::Enter);
    assert!(h.app().usage.detail_open());
    assert_eq!(
        h.focus(),
        Some(jackin_preview_app::screens::usage::DETAIL),
        "Enter transfers focus to account detail"
    );
    assert_eq!(h.app().usage.selected(), Some("acct-claude-work"));
    let detail = h.text();
    assert!(detail.contains("Claude · Work"), "{detail}");
    assert!(detail.contains("Session · 5-hour"), "{detail}");
    assert!(detail.contains("76%"), "{detail}");
    assert!(detail.contains("stale · last good 47 min ago"), "{detail}");
    assert!(detail.contains("Retry        resets in 13 min"), "{detail}");
    assert!(
        h.harness.diagnostics().is_empty(),
        "{:?}",
        h.harness.diagnostics()
    );

    h.key(KeyCode::Esc);
    assert_eq!(h.app().route(), Route::Usage);
    assert!(!h.app().usage.detail_open());
    assert_eq!(h.app().usage.selected(), Some("acct-claude-work"));
    assert_eq!(h.focus(), Some(jackin_preview_app::screens::usage::LIST));
    assert!(
        h.harness.diagnostics().is_empty(),
        "after returning to list: {:?}",
        h.harness.diagnostics()
    );

    h.key(KeyCode::Esc);
    assert_eq!(h.app().route(), Route::Manager);
}

#[test]
fn usage_account_click_selects_the_keyed_row_without_opening_detail() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 120, 40);
    h.key(KeyCode::Char('u'));
    h.key(KeyCode::Down);
    h.key(KeyCode::Tab);
    assert_eq!(h.focus(), Some(jackin_preview_app::screens::usage::LIST));

    let (x, y) = h.find("Work").expect("visible Work account row");
    let list_area = h
        .harness
        .area_of(jackin_preview_app::screens::usage::LIST)
        .expect("registered Usage account list");
    assert!(list_area.contains(termrock::Position::new(x, y)));

    let (outside_x, outside_y) = h.find("Limits").expect("visible summary outside the list");
    let outside = termrock::Position::new(outside_x, outside_y);
    assert!(!list_area.contains(outside));
    h.click(outside_x, outside_y);
    assert_eq!(h.app().usage.selected(), Some("acct-claude-personal"));

    h.click(x, y);

    assert_eq!(
        h.app().usage.tab,
        jackin_preview_app::screens::usage::Tab::Registration
    );
    assert_eq!(h.app().usage.selected(), Some("acct-claude-work"));
    assert!(
        !h.app().usage.detail_open(),
        "a row click selects without opening"
    );
    assert_eq!(h.focus(), Some(jackin_preview_app::screens::usage::LIST));
    assert!(
        h.harness.diagnostics().is_empty(),
        "{:?}",
        h.harness.diagnostics()
    );
}

#[test]
fn usage_list_selection_uses_account_ids_for_duplicate_names_and_reordering() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 120, 40);
    {
        let accounts = &mut h.app_mut().world.accounts;
        accounts
            .get_mut("acct-claude-personal")
            .expect("personal account")
            .display_name = "Shared".into();
        accounts
            .get_mut("acct-claude-work")
            .expect("work account")
            .display_name = "Shared".into();
    }

    h.key(KeyCode::Char('u'));
    h.key(KeyCode::Down);
    h.key(KeyCode::Tab);
    h.key(KeyCode::Down);
    assert_eq!(h.app().usage.selected(), Some("acct-claude-work"));
    assert_eq!(
        h.app().usage.tab,
        jackin_preview_app::screens::usage::Tab::Registration
    );

    {
        let accounts = &mut h.app_mut().world.accounts;
        accounts
            .get_mut("acct-claude-personal")
            .expect("personal account")
            .default_for_provider = false;
        accounts
            .get_mut("acct-claude-work")
            .expect("work account")
            .default_for_provider = true;
        let last = accounts.accounts.len().saturating_sub(1);
        accounts.accounts.swap(0, last);
    }
    h.key(KeyCode::Up);
    assert_eq!(
        h.app().usage.selected(),
        Some("acct-claude-work"),
        "keyed selection survives a row reorder despite duplicate names"
    );
    assert_eq!(
        h.app().usage.tab,
        jackin_preview_app::screens::usage::Tab::Registration
    );
    assert!(
        h.harness.diagnostics().is_empty(),
        "{:?}",
        h.harness.diagnostics()
    );
}

#[test]
fn usage_list_reconciles_a_removed_selection_to_a_surviving_account() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 120, 40);
    h.key(KeyCode::Char('u'));
    h.key(KeyCode::Down);
    h.key(KeyCode::Tab);
    h.key(KeyCode::Down);
    assert_eq!(h.app().usage.selected(), Some("acct-claude-work"));

    h.app_mut()
        .world
        .accounts
        .accounts
        .retain(|account| account.id != "acct-claude-work");
    h.key(KeyCode::Down);

    let selected = h.app().usage.selected();
    assert_ne!(selected, Some("acct-claude-work"));
    assert!(selected.is_some_and(|id| h.app().world.accounts.get(id).is_some()));
    assert!(
        h.harness.diagnostics().is_empty(),
        "{:?}",
        h.harness.diagnostics()
    );
}

#[test]
fn usage_dialog_input_does_not_change_the_underlying_tab_or_account() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 120, 40);
    h.key(KeyCode::Char('u'));
    h.key(KeyCode::Down);
    h.key(KeyCode::Tab);
    h.key(KeyCode::Down);
    assert_eq!(h.app().usage.selected(), Some("acct-claude-work"));
    assert_eq!(
        h.app().usage.tab,
        jackin_preview_app::screens::usage::Tab::Registration
    );

    h.key(KeyCode::Char('q'));
    assert!(h.text().contains("Exit jackin?"), "{}", h.text());
    h.key(KeyCode::Down);
    assert_eq!(h.app().usage.selected(), Some("acct-claude-work"));
    assert_eq!(
        h.app().usage.tab,
        jackin_preview_app::screens::usage::Tab::Registration
    );
    h.key(KeyCode::Esc);

    assert!(!h.text().contains("Exit jackin?"));
    assert_eq!(h.app().usage.selected(), Some("acct-claude-work"));
    assert_eq!(
        h.app().usage.tab,
        jackin_preview_app::screens::usage::Tab::Registration
    );
    assert!(
        h.harness.diagnostics().is_empty(),
        "{:?}",
        h.harness.diagnostics()
    );
}

#[test]
fn empty_usage_list_has_no_stale_selection_or_keyboard_target() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 120, 40);
    h.key(KeyCode::Char('u'));
    h.key(KeyCode::Down);
    h.key(KeyCode::Tab);
    assert_eq!(h.app().usage.selected(), Some("acct-claude-personal"));

    h.app_mut().world.accounts.accounts.clear();
    h.key(KeyCode::Down);

    assert_eq!(h.app().usage.selected(), None);
    assert!(!h.app().usage.detail_open());
    assert!(
        h.harness.diagnostics().is_empty(),
        "{:?}",
        h.harness.diagnostics()
    );
}

#[test]
fn empty_usage_activation_keeps_focus_on_registered_controls() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 120, 40);
    h.key(KeyCode::Char('u'));
    h.key(KeyCode::Down);
    h.key(KeyCode::Tab);
    assert_eq!(h.focus(), Some(jackin_preview_app::screens::usage::LIST));
    h.key(KeyCode::Down);
    assert_eq!(h.app().usage.selected(), Some("acct-claude-work"));

    h.app_mut().world.accounts.accounts.clear();
    h.key(KeyCode::Down);
    assert_eq!(h.app().usage.selected(), None);
    assert!(!h.app().usage.detail_open());
    assert_eq!(h.focus(), Some(jackin_preview_app::screens::usage::LIST));

    for key in [KeyCode::Enter, KeyCode::Right] {
        h.key(key);
        assert_eq!(h.app().usage.selected(), None);
        assert!(!h.app().usage.detail_open());
        assert_eq!(h.focus(), Some(jackin_preview_app::screens::usage::LIST));
        assert!(
            h.harness.diagnostics().is_empty(),
            "{:?}",
            h.harness.diagnostics()
        );
    }

    // Tabs focus is tested from the route's valid initial focus, before the
    // empty row set changes which focus targets are registered.
    let mut tabs = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 120, 40);
    tabs.key(KeyCode::Char('u'));
    assert_eq!(tabs.focus(), Some(jackin_preview_app::screens::usage::TABS));
    tabs.app_mut().world.accounts.accounts.clear();
    tabs.app_mut().usage.select(None::<String>);
    assert_eq!(tabs.app().usage.selected(), None);
    tabs.key(KeyCode::Enter);
    assert_eq!(tabs.app().usage.selected(), None);
    assert!(!tabs.app().usage.detail_open());
    assert_eq!(tabs.focus(), Some(jackin_preview_app::screens::usage::TABS));
    assert!(
        tabs.harness.diagnostics().is_empty(),
        "{:?}",
        tabs.harness.diagnostics()
    );

    tabs.key(KeyCode::Down);
    assert_eq!(
        tabs.app().usage.tab,
        jackin_preview_app::screens::usage::Tab::Registration
    );
    assert_eq!(tabs.focus(), Some(jackin_preview_app::screens::usage::TABS));
    assert!(
        tabs.harness.diagnostics().is_empty(),
        "{:?}",
        tabs.harness.diagnostics()
    );
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

#[test]
fn short_usage_detail_scrolls_with_viewport_keys_and_reopens_at_top() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 72, 20);
    h.app_mut().usage.select(Some("acct-claude-work"));

    h.key(KeyCode::Char('u'));
    h.key(KeyCode::Down);
    assert_eq!(
        h.app().usage.tab,
        jackin_preview_app::screens::usage::Tab::Registration
    );
    h.key(KeyCode::Enter);

    assert!(h.app().usage.detail_open());
    assert_eq!(h.app().usage.selected(), Some("acct-claude-work"));
    let top = h.text();
    assert!(top.contains("Usage detail"), "{top}");
    assert!(top.contains("Claude · Work"), "{top}");
    assert!(top.contains("Provider"), "{top}");
    assert!(
        !top.contains("Retry"),
        "short viewport should start at the top: {top}"
    );

    for _ in 0..8 {
        h.key(KeyCode::PageDown);
    }
    let bottom = h.text();
    assert!(bottom.contains("Retry        resets in 13 min"), "{bottom}");
    assert_eq!(
        h.app().usage.tab,
        jackin_preview_app::screens::usage::Tab::Registration
    );
    assert_eq!(h.app().usage.selected(), Some("acct-claude-work"));
    assert_eq!(h.buffer().area().width, 72);
    assert_eq!(h.buffer().area().height, 20);

    h.key(KeyCode::Esc);
    assert!(!h.app().usage.detail_open());
    assert_eq!(h.app().usage.selected(), Some("acct-claude-work"));
    h.key(KeyCode::Enter);
    let reopened = h.text();
    assert!(reopened.contains("Usage detail"), "{reopened}");
    assert!(reopened.contains("Claude · Work"), "{reopened}");
    assert!(reopened.contains("Provider"), "{reopened}");
    assert!(
        !reopened.contains("Retry"),
        "reopened detail should reset to the top: {reopened}"
    );
    assert!(
        h.harness.diagnostics().is_empty(),
        "{:?}",
        h.harness.diagnostics()
    );
}

#[test]
fn selected_id_survives_account_reorder_and_missing_quota_stays_unknown() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 80, 24);
    {
        let app = h.app_mut();
        app.usage.select(Some("acct-claude-work"));
        let accounts = &mut app.world.accounts.accounts;
        let work_index = accounts
            .iter()
            .position(|account| account.id == "acct-claude-work")
            .expect("work account exists before reorder");
        let work = accounts.remove(work_index);
        accounts.push(work);
        assert_eq!(
            accounts.last().map(|account| account.id.as_str()),
            Some("acct-claude-work")
        );
        app.world
            .accounts
            .get_mut("acct-claude-work")
            .expect("reordered work account")
            .usage
            .windows
            .clear();
    }

    h.key(KeyCode::Char('u'));
    h.key(KeyCode::Enter);
    assert_eq!(h.app().usage.selected(), Some("acct-claude-work"));
    let detail = h.text();
    assert!(detail.contains("Claude · Work"), "{detail}");
    assert!(detail.contains("No reported quota values"), "{detail}");
    assert!(
        !detail.contains("0%"),
        "unknown quota must not become 0%: {detail}"
    );
    assert!(!detail.contains("Claude · Personal"), "{detail}");
    assert!(
        h.harness.diagnostics().is_empty(),
        "{:?}",
        h.harness.diagnostics()
    );
}

#[test]
fn removed_selected_account_shows_unavailable_detail_without_fallback() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 80, 24);
    h.app_mut().usage.select(Some("acct-claude-work"));
    h.app_mut()
        .world
        .accounts
        .remove("acct-claude-work")
        .expect("selected work account exists before removal");

    h.key(KeyCode::Char('u'));
    h.key(KeyCode::Enter);
    assert!(h.app().usage.detail_open());
    assert_eq!(h.app().usage.selected(), Some("acct-claude-work"));
    let detail = h.text();
    assert!(detail.contains("selected account unavailable"), "{detail}");
    assert!(!detail.contains("Claude · Work"), "{detail}");
    assert!(!detail.contains("Claude · Personal"), "{detail}");
    assert!(
        h.harness.diagnostics().is_empty(),
        "{:?}",
        h.harness.diagnostics()
    );
}
