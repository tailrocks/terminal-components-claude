#![allow(
    dead_code,
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    clippy::arithmetic_side_effects,
    clippy::doc_markdown,
    clippy::explicit_iter_loop,
    clippy::indexing_slicing,
    clippy::many_single_char_names,
    clippy::missing_panics_doc,
    clippy::panic,
    clippy::too_many_lines,
    clippy::unwrap_used,
    clippy::expect_used
)]

use termrock::{GlyphRole, KeyCode, MouseKind, Part, PartRef, Theme};

use jackin_preview_app::Route;
use jackin_preview_app::app::{ACCOUNT_FORM_LAYER, ACCOUNTS_LIST};
use jackin_preview_app::{Motion, Scenario};
mod support;
use support::H;

#[test]
fn first_use_plays_intro_then_manager_and_no_replay_when_returning() {
    let mut h = H::new(Scenario::FirstUse, Motion::Full, 0, 120, 40);
    assert_eq!(h.app().route(), Route::Intro);
    h.ticks(45);
    assert!(h.text().contains("Stand up, operator…"), "{}", h.text());
    assert!(h.text().contains("jackin❯"));

    h.ticks(3);
    h.key(KeyCode::Enter);
    assert_eq!(h.app().route(), Route::Intro);
    h.key(KeyCode::Enter);
    assert_eq!(h.app().route(), Route::Manager);
    assert!(h.text().contains("Current directory"));
    assert!(h.text().contains("+ New workspace"));
    let r = H::new(Scenario::Returning, Motion::Full, 0, 120, 40);
    assert_eq!(
        r.app().route(),
        Route::Manager,
        "an active Construct joins without replay"
    );
    assert!(r.text().contains("2 running"));
}

#[test]
fn paused_intro_outro_and_failed_launch_match_the_baseline_first_frame() {
    let intro = H::new(Scenario::FirstUse, Motion::Paused, 300, 120, 40);
    let intro_text = intro.text();
    assert!(
        intro_text.contains("Enter Skip"),
        "intro warp hint missing: {intro_text}"
    );
    assert!(
        intro_text.contains('─') || intro_text.contains('+'),
        "intro warp rain missing: {intro_text}"
    );
    let outro = H::new(Scenario::OutroLast, Motion::Paused, 40, 120, 40);
    let outro_text = outro.text();
    assert!(
        outro_text.contains("Enter Skip"),
        "outro warp hint missing: {outro_text}"
    );
    assert!(
        outro_text.contains('─') || outro_text.contains('+'),
        "outro warp rain missing: {outro_text}"
    );
    let phrases = H::new(Scenario::FirstUse, Motion::Paused, 40, 120, 40);
    assert!(phrases.text().contains("Enter Skip"), "{}", phrases.text());
    assert!(phrases.text().contains("jackin❯"), "{}", phrases.text());
    let failed = H::new(Scenario::LaunchFailure, Motion::Paused, 240, 120, 40);
    assert!(failed.text().contains("Launch failed"), "{}", failed.text());
}

#[test]
fn reduced_motion_and_paused_frames_are_deterministic() {
    let mut h = H::new(Scenario::FirstUse, Motion::Reduced, 0, 80, 24);
    assert_eq!(h.app().route(), Route::Intro);
    assert!(h.text().contains("Enter Continue"));
    h.ticks(3);
    h.key(KeyCode::Enter);
    assert_eq!(h.app().route(), Route::Manager);
    let a = H::new(Scenario::FirstUse, Motion::Paused, 282, 100, 30);
    let b = H::new(Scenario::FirstUse, Motion::Paused, 282, 100, 30);
    assert_eq!(a.text(), b.text());
    let mut p = H::new(Scenario::FirstUse, Motion::Paused, 45, 80, 24);
    p.ticks(5);
    assert!(
        p.text().contains("Stand up, operator…"),
        "paused frames never advance"
    );
}

#[test]
fn manager_navigation_expand_and_detail_focus() {
    let mut h = H::new(Scenario::Returning, Motion::Full, 0, 120, 40);
    h.key(KeyCode::Home);
    let workspace = h.app().world.workspaces.first().unwrap().id;
    assert_eq!(h.app().manager.selected(), Some(workspace));
    h.key(KeyCode::Right);
    assert!(
        h.text().contains("7f3a"),
        "instance children visible after expand"
    );
    h.key(KeyCode::Down);
    let (x, y) = h.find("7f3a").unwrap();
    h.click(x, y);
    h.key(KeyCode::Tab);
    assert!(h.text().contains("Live topology"));
    h.key(KeyCode::Esc);
    assert_eq!(h.focus(), Some(jackin_preview_app::screens::manager::TREE));

    let (x, y) = h.find("infra-control-plane").unwrap();
    h.click(x, y);
    assert!(h.text().contains("Workspaces › infra-control-plane"));
}

#[test]
fn launch_runs_all_stages_and_hands_off_to_the_capsule() {
    let mut h = H::new(Scenario::LaunchRunning, Motion::Full, 0, 120, 40);
    assert_eq!(h.app().route(), Route::Cockpit);
    for _ in 0..40 {
        h.ticks(10);
        if h.app().route() != Route::Cockpit {
            break;
        }
    }
    assert!(
        matches!(h.app().route(), Route::Handoff | Route::Capsule),
        "route {:?}",
        h.app().route()
    );
    h.ticks(15);
    assert_eq!(h.app().route(), Route::Capsule);
    assert!(h.text().contains("jackin❯"));

    h.ticks(60);
    h.type_str("hello");
    assert!(h.text().contains("hello"));
}

#[test]
fn launch_failure_returns_to_the_construct_when_another_instance_runs() {
    let mut h = H::new(Scenario::LaunchFailure, Motion::Full, 0, 120, 40);
    for _ in 0..60 {
        h.ticks(10);
        if h.text().contains("Launch failed") {
            break;
        }
    }
    assert!(h.text().contains("Launch failed"), "{}", h.text());
    assert!(h.text().contains("Network"));
    h.key(KeyCode::Esc);
    assert_eq!(h.app().route(), Route::Manager);
    assert!(h.text().contains("still running"));
}

#[test]
fn detach_reconnect_and_final_exit_plays_one_outro() {
    let mut h = H::new(Scenario::OutroLast, Motion::Full, 0, 120, 40);
    assert_eq!(h.app().route(), Route::Capsule);
    h.ctrl('b');
    h.key(KeyCode::Char('d'));
    assert_eq!(h.app().route(), Route::Manager);
    assert!(h.text().contains("Detached"));
    h.key(KeyCode::Enter);
    assert_eq!(
        h.app().route(),
        Route::Capsule,
        "reconnect restores the Capsule"
    );
    h.ctrl('q');
    assert!(h.text().contains("Unsaved work"));
    h.key(KeyCode::Down);
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    assert_eq!(h.app().route(), Route::Outro);
    h.key(KeyCode::Enter);
    h.ticks(25);
    assert!(
        h.text()
            .contains("You were in the Construct for 2 hours 14 minutes"),
        "{}",
        h.text()
    );
    h.key(KeyCode::Enter);
    assert!(h.app().should_quit());
}

#[test]
fn still_inside_feedback_when_other_instances_remain() {
    let mut h = H::new(Scenario::CapsuleMulti, Motion::Full, 0, 120, 40);
    assert_eq!(h.app().route(), Route::Capsule);
    h.ctrl('q');
    h.key(KeyCode::Down);
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    assert_eq!(h.app().route(), Route::Manager);
    assert!(h.text().contains("Still inside the Construct"));
    assert_eq!(h.app().world.running_count(), 1);
}

#[test]
fn too_small_state_and_resize_recover() {
    let mut h = H::new(Scenario::Returning, Motion::Full, 0, 120, 40);
    h.resize(60, 18);
    assert!(h.text().contains("Terminal too small"));
    h.resize(80, 24);
    assert!(h.text().contains("Workspaces"));
}

#[test]
fn accounts_register_with_a_1password_reference_and_never_render_the_secret() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Reduced, 0, 120, 40);
    assert_eq!(h.app().route(), Route::Accounts);
    assert!(h.text().contains("Overview"));
    h.key(KeyCode::Char('a'));
    assert!(h.text().contains("New account"));
    assert!(
        h.text().contains("Tab Next field") && h.text().contains("Edit / Save"),
        "{}",
        h.text()
    );
    h.key(KeyCode::Enter);
    h.type_str("Team");
    for _ in 0..4 {
        h.key(KeyCode::Tab);
    }
    h.key(KeyCode::Enter);
    h.ticks(4);
    assert!(h.text().contains("chainargos"), "{}", h.text());
    h.key(KeyCode::Enter);
    h.ticks(4);
    assert!(h.text().contains("Engineering"), "{}", h.text());
    h.key(KeyCode::Enter);
    h.ticks(4);
    h.type_str("Anthropic");
    h.ticks(4);
    h.key(KeyCode::Enter);
    h.ticks(4);
    assert!(h.text().contains("credential"), "{}", h.text());
    h.key(KeyCode::Enter);
    h.ticks(2);
    assert!(
        h.text().contains("Anthropic · Work › credential"),
        "{}",
        h.text()
    );
    assert!(
        !h.text().contains("valid-ant01"),
        "secret leaked into the frame"
    );

    h.tab_to(jackin_preview_app::screens::accounts::FORM.sub("save"));
    h.key(KeyCode::Enter);
    assert!(
        h.text()
            .contains("Already registered: this source is used by Claude · Work"),
        "{}",
        h.text()
    );
    assert!(h.app().world.accounts.get("acct-anthropic-team").is_none());

    h.tab_to(jackin_preview_app::screens::accounts::FORM.sub("provider"));
    h.key(KeyCode::Down);
    assert!(h.text().contains("Codex · OpenAI"), "{}", h.text());
    h.tab_to(jackin_preview_app::screens::accounts::FORM.sub("op"));
    h.key(KeyCode::Enter);
    h.ticks(4);
    h.key(KeyCode::Enter);
    h.ticks(4);
    h.key(KeyCode::Enter);
    h.ticks(4);
    h.type_str("Throttled");
    h.ticks(4);
    h.key(KeyCode::Enter);
    h.ticks(4);
    h.key(KeyCode::Enter);
    h.ticks(2);
    assert!(
        h.text().contains("OpenAI · Throttled sandbox"),
        "{}",
        h.text()
    );
    h.tab_to(jackin_preview_app::screens::accounts::FORM.sub("save"));
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Saved Codex · Team"), "{}", h.text());
    assert!(h.text().contains("Rate limited"), "{}", h.text());
    assert!(!h.text().contains("throttled-thr01"));
    assert!(h.app().world.accounts.get("acct-openai-team").is_some());

    h.key(KeyCode::Char('r'));
    assert!(h.text().contains("Refreshing"), "{}", h.text());
    h.ticks(60);
    assert!(
        h.text()
            .contains("Refreshed Codex · Team · still rate limited"),
        "{}",
        h.text()
    );
}

#[test]
fn one_password_picker_uses_active_items_and_stable_stage_two_keys() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Reduced, 0, 120, 40);
    h.key(KeyCode::Char('a'));
    h.key(KeyCode::Enter);
    h.type_str("Team");
    h.tab_to(jackin_preview_app::screens::accounts::PROVIDER);
    h.key(KeyCode::Down);
    h.tab_to(jackin_preview_app::screens::accounts::OP);
    h.key(KeyCode::Enter);
    h.ticks(4);

    let picker = h
        .harness
        .layer_area(jackin_preview_app::ACCOUNT_PICKER)
        .expect("1Password account picker should be open");
    assert_eq!(
        picker.height, 6,
        "one option needs one visible list row plus picker chrome: {picker:?}"
    );
    assert!(h.text().contains("chainargos.1pa"), "{}", h.text());
    assert!(!h.text().contains("Codex Primary"), "{}", h.text());

    h.key(KeyCode::Enter);
    h.ticks(4);
    let picker = h
        .harness
        .layer_area(jackin_preview_app::ACCOUNT_PICKER)
        .expect("vault picker should remain open");
    assert_eq!(
        picker.height, 6,
        "one vault needs one visible list row plus picker chrome: {picker:?}"
    );

    h.key(KeyCode::Enter);
    h.ticks(4);
    assert!(h.text().contains("OpenAI · Codex Primary"), "{}", h.text());
    h.type_str("Throttled");
    h.ticks(4);
    h.key(KeyCode::Enter);
    h.ticks(4);
    assert!(
        h.text().contains("OpenAI · Throttled sandbox"),
        "{}",
        h.text()
    );

    h.key(KeyCode::Enter);
    h.ticks(2);
    let selected = h
        .app()
        .accounts
        .selected_op
        .as_ref()
        .expect("stage three should resolve a reference");
    assert_eq!(selected.item_id, "it_thr01");
    assert!(h.text().contains("OpenAI · Throttled sandbox › credential"));
}

#[test]
fn form_dialog_secret_never_reaches_the_screen_as_a_string() {
    accounts_register_with_a_1password_reference_and_never_render_the_secret();
}

#[test]
fn accounts_plain_key_is_masked_everywhere_and_remove_asks_first() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Reduced, 0, 120, 40);
    h.key(KeyCode::Char('a'));
    h.key(KeyCode::Enter);
    h.type_str("Spare");
    for _ in 0..3 {
        h.key(KeyCode::Tab);
    }
    h.key(KeyCode::Down);
    h.key(KeyCode::Down);
    assert!(h.text().contains("API key"), "{}", h.text());
    h.key(KeyCode::Tab);
    h.key(KeyCode::Enter);
    h.type_str("sk-ant-valid-abcdef1234");
    assert!(
        !h.text().contains("sk-ant-valid"),
        "raw key rendered while typing"
    );
    h.key(KeyCode::Tab);
    let t = h.text();
    assert!(!t.contains("sk-ant-valid"), "raw key rendered: {t}");
    assert!(t.contains("1234"), "tail hint missing: {t}");
    h.tab_to(jackin_preview_app::screens::accounts::FORM.sub("save"));
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Saved Claude · Spare"), "{}", h.text());
    assert!(!h.text().contains("abcdef"));
    let a = h.app().world.accounts.get("acct-anthropic-spare").unwrap();
    assert!(
        matches!(&a.source, jackin_preview_app::domain::account::CredentialSource::PlainApiKey { tail, .. } if tail == "1234")
    );
    assert!(
        !format!("{:?}", a.source).contains("abcdef"),
        "fingerprint must not embed the key"
    );
    h.key(KeyCode::Char('x'));
    assert!(h.text().contains("Remove account Spare?"), "{}", h.text());
    h.key(KeyCode::Esc);
    assert!(h.app().world.accounts.get("acct-anthropic-spare").is_some());
}

#[test]
fn usage_overlay_is_read_only_and_hands_off_to_accounts() {
    let mut h = H::new(Scenario::Returning, Motion::Full, 0, 120, 40);
    h.key(KeyCode::Char('u'));
    assert_eq!(h.app().route(), Route::Usage);
    assert!(h.text().contains("Usage · read-only"));
    assert!(h.text().contains("Usage › Overview"));
    assert!(h.text().contains("Overview"));
    h.key(KeyCode::Down);
    assert!(h.text().contains("Limits"), "{}", h.text());
    h.key(KeyCode::Char('m'));
    assert_eq!(h.app().route(), Route::Accounts);
    assert!(h.text().contains("Accounts › "));
    h.key(KeyCode::Esc);
    assert_eq!(h.app().route(), Route::Manager);
}

#[test]
fn prelude_creates_a_pending_workspace_and_opens_the_editor() {
    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    h.key(KeyCode::End);
    h.key(KeyCode::Enter);
    assert_eq!(h.app().route(), Route::Prelude);
    assert!(h.text().contains("step 1 of 5"), "{}", h.text());
    assert!(h.text().contains("~/src/payments-platform"));

    h.key(KeyCode::Backspace);
    assert!(h.text().contains("▸ payments-platform/"), "{}", h.text());
    assert!(h.text().contains("customer-portal/"), "{}", h.text());
    h.key(KeyCode::Down);
    assert!(h.text().contains("▸ customer-portal/"), "{}", h.text());
    h.key(KeyCode::Down);
    assert!(h.text().contains("▸ data-pipeline/"), "{}", h.text());
    h.key(KeyCode::Char(' '));
    assert!(h.text().contains("step 2 of 5"), "{}", h.text());
    assert!(
        h.text()
            .contains("Same path   /Users/alexey/src/data-pipeline"),
        "{}",
        h.text()
    );
    assert!(h.text().contains("✓ Source"));

    h.key(KeyCode::Esc);
    assert!(h.text().contains("step 1 of 5"), "{}", h.text());
    assert!(h.text().contains("~/src"), "{}", h.text());
    h.key(KeyCode::Down);
    h.key(KeyCode::Down);
    h.key(KeyCode::Char(' '));
    h.key(KeyCode::Enter);
    assert!(h.text().contains("step 4 of 5"), "{}", h.text());
    assert!(h.text().contains("destination"), "{}", h.text());
    h.key(KeyCode::Enter);
    assert!(h.text().contains("step 5 of 5"), "{}", h.text());
    assert!(h.text().contains("data-pipeline"), "{}", h.text());
    h.key(KeyCode::Enter);
    assert_eq!(h.app().route(), Route::Editor, "{}", h.text());
    let ed = &h.app().editor;
    assert_eq!(ed.pending.name, "data-pipeline");
    assert_eq!(ed.pending.workdir, "/Users/alexey/src/data-pipeline");
    assert_eq!(ed.pending.mounts.len(), 1);
    assert_eq!(
        ed.pending.mounts[0].destination,
        "/Users/alexey/src/data-pipeline"
    );
}

#[test]
fn prelude_refuses_a_duplicate_name_and_cancels_cleanly() {
    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    h.key(KeyCode::End);
    h.key(KeyCode::Enter);
    h.key(KeyCode::Backspace);
    h.key(KeyCode::Down);
    h.key(KeyCode::Char(' '));
    h.key(KeyCode::Enter);
    h.key(KeyCode::Enter);
    assert!(h.text().contains("step 5 of 5"), "{}", h.text());
    h.key(KeyCode::Enter);
    assert!(
        h.text()
            .contains("A workspace named customer-portal already exists"),
        "{}",
        h.text()
    );
    assert_eq!(h.app().route(), Route::Prelude);

    for _ in 0..8 {
        h.key(KeyCode::Esc);
        if h.app().route() != Route::Prelude {
            break;
        }
    }
    assert_eq!(h.app().route(), Route::Manager);
    assert!(
        h.text().contains("Cancelled · nothing created"),
        "{}",
        h.text()
    );
}

#[test]
fn editor_edits_count_once_preview_then_saves_and_returns() {
    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    h.key(KeyCode::Down);
    h.key(KeyCode::Char('e'));
    assert_eq!(h.app().route(), Route::Editor);
    assert!(h.text().contains("payments-platform › edit"));
    h.key(KeyCode::Char(']'));
    h.key(KeyCode::Enter);
    h.key(KeyCode::Char('r'));
    h.key(KeyCode::Char('i'));
    assert!(h.text().contains("• 1 change"), "{}", h.text());
    assert!(h.text().contains("Mounts •"));

    h.key(KeyCode::Esc);
    h.key(KeyCode::Esc);
    assert!(
        h.text().contains("Save changes before leaving?"),
        "{}",
        h.text()
    );
    h.key(KeyCode::Esc);
    assert_eq!(h.app().route(), Route::Editor);

    h.ctrl('s');
    assert!(h.text().contains("Save workspace"));
    assert!(h.text().contains("1 modified"));
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Saving"), "{}", h.text());
    h.ticks(20);
    assert_eq!(h.app().route(), Route::Manager, "{}", h.text());
    assert!(h.text().contains("Workspace payments-platform saved"));
    let ws = h.app().world.workspace(1).unwrap();
    assert!(ws.mounts[0].readonly);
    assert_eq!(
        ws.mounts[0].isolation,
        jackin_preview_app::domain::workspace::Isolation::Clone
    );
}

#[test]
fn editor_env_plain_value_stays_masked() {
    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    h.key(KeyCode::Down);
    h.key(KeyCode::Char('e'));
    h.key(KeyCode::Char('4'));
    h.key(KeyCode::Enter);
    let t = h.text();
    assert!(t.contains("DATABASE_URL"));
    assert!(!t.contains("pw-fixture-only"), "plain value leaked: {t}");
    h.key(KeyCode::Char('m'));
    assert_eq!(h.app().route(), Route::Editor);
    assert!(!h.text().contains("postgres://"), "{}", h.text());
    assert!(!h.text().contains("pw-fixture-only"), "{}", h.text());
    assert!(
        h.text().contains("plain values stay masked"),
        "{}",
        h.text()
    );

    h.key(KeyCode::Char('a'));
    assert!(
        h.text().contains("New workspace environment key"),
        "{}",
        h.text()
    );
    h.key(KeyCode::Enter);
    h.type_str("NEW_SECRET");
    h.key(KeyCode::Tab);
    h.key(KeyCode::Tab);
    h.key(KeyCode::Enter);
    h.type_str("sk-live-abcdefghijklmnop1234");
    h.key(KeyCode::Tab);
    assert!(!h.text().contains("abcdefghijklmnop"), "{}", h.text());
    h.tab_to(termrock::Id::root("editor.cfg").sub("form").sub("save"));
    h.key(KeyCode::Enter);
    let t = h.text();
    assert!(t.contains("NEW_SECRET"), "{t}");
    assert!(t.contains("************1234"), "{t}");
    assert!(!t.contains("abcdefghijklmnop"));
    assert!(t.contains("• 1 change"), "{t}");
    assert!(
        h.app()
            .world
            .workspace(1)
            .unwrap()
            .env
            .iter()
            .all(|env| env.key != "NEW_SECRET"),
        "draft environment leaked before workspace save: {t}"
    );
    h.ctrl('s');
    h.tab_to(termrock::Id::root("editor.cfg").sub("form").sub("save"));
    h.key(KeyCode::Enter);
    h.ticks(20);
    assert_eq!(h.app().route(), Route::Manager, "{}", h.text());
    assert!(
        h.app()
            .world
            .workspace(1)
            .unwrap()
            .env
            .iter()
            .any(|env| env.key == "NEW_SECRET"),
        "saved environment draft was not committed: {}",
        h.text()
    );
}

#[test]
fn form_dialog_toggles_visibility_and_keeps_drafts() {
    editor_env_plain_value_stays_masked();
}

#[test]
fn editor_env_rejects_invalid_key_before_staging() {
    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    h.key(KeyCode::Down);
    h.key(KeyCode::Char('e'));
    h.key(KeyCode::Char('4'));
    h.key(KeyCode::Enter);
    h.key(KeyCode::Char('a'));
    h.key(KeyCode::Enter);
    h.type_str("BAD-NAME");
    h.tab_to(termrock::Id::root("editor.cfg").sub("form").sub("save"));
    h.key(KeyCode::Enter);
    assert!(h.app().editor.env_form_open);
    assert!(
        h.text().contains("letters, digits and underscores"),
        "{}",
        h.text()
    );
    assert!(
        h.app()
            .editor
            .pending
            .env
            .iter()
            .all(|env| env.key != "BAD-NAME")
    );
    assert!(
        h.app()
            .world
            .workspace(1)
            .unwrap()
            .env
            .iter()
            .all(|env| env.key != "BAD-NAME")
    );
}

#[test]
fn settings_trust_toggle_and_failed_save_keep_edits() {
    let mut h = H::new(Scenario::HardCases, Motion::Reduced, 0, 120, 40);
    for _ in 0..8 {
        h.ticks(3);
        if h.app().route() == Route::Manager {
            break;
        }
        h.key(KeyCode::Enter);
    }
    assert_eq!(h.app().route(), Route::Manager, "{}", h.text());
    h.key(KeyCode::Char('s'));
    assert_eq!(h.app().route(), Route::Settings);
    h.key(KeyCode::Char('5'));
    h.key(KeyCode::Enter);
    h.key(KeyCode::Char(' '));
    assert!(h.text().contains("• 1 change"), "{}", h.text());
    // `5` then Enter only selects Trust. Space toggles. Toggle back off
    // so the saved draft stays untrusted.
    h.key(KeyCode::Char(' '));
    h.ctrl('s');
    assert!(h.text().contains("Save settings"));
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    h.ticks(20);
    assert!(h.text().contains("Settings error"), "{}", h.text());
    h.key(KeyCode::Esc);
    assert_eq!(h.app().route(), Route::Settings);
    assert!(h.text().contains("• 1 change"));

    h.ctrl('s');
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    h.ticks(20);
    assert_eq!(h.app().route(), Route::Manager, "{}", h.text());

    assert!(!h.app().world.global.trust[0].trusted);
}

#[test]
fn hard_cases_refresh_keeps_last_good_and_help_opens_everywhere() {
    let mut h = H::new(Scenario::HardCases, Motion::Reduced, 0, 120, 40);
    for _ in 0..8 {
        h.ticks(3);
        if h.app().route() == Route::Manager {
            break;
        }
        h.key(KeyCode::Enter);
    }
    h.key(KeyCode::Char('c'));
    assert_eq!(h.app().route(), Route::Accounts);
    h.key(KeyCode::Char('?'));
    assert!(h.text().contains("Credential sources"), "{}", h.text());
    h.key(KeyCode::Esc);
    h.key(KeyCode::Down);
    h.key(KeyCode::Down);
    h.key(KeyCode::Char('r'));
    h.ticks(60);
    assert!(h.text().contains("broker unreachable"), "{}", h.text());
    h.key(KeyCode::Char('u'));
    assert_eq!(h.app().route(), Route::Usage);
    h.key(KeyCode::Char('?'));
    assert!(h.text().contains("Reading meters"));
}

#[test]
fn complete_jackin_flow_keyboard_first() {
    use termrock::Id;
    let form_save = jackin_preview_app::screens::accounts::FORM.sub("save");
    let cfg_save = Id::root("editor.cfg").sub("form").sub("save");
    let mut h = H::new(Scenario::FirstUse, Motion::Reduced, 0, 120, 40);

    assert_eq!(h.app().route(), Route::Intro);
    h.ticks(3);
    h.key(KeyCode::Enter);
    assert_eq!(h.app().route(), Route::Manager);
    assert_eq!(h.app().world.running_count(), 0);

    h.key(KeyCode::Char('c'));
    assert_eq!(h.app().route(), Route::Accounts);

    for (name, folder) in [("Personal", "~/.claude"), ("Work", "~/.claude-work")] {
        h.key(KeyCode::Char('a'));
        h.key(KeyCode::Enter);
        h.type_str(name);
        for _ in 0..3 {
            h.key(KeyCode::Tab);
        }
        h.key(KeyCode::Down);
        assert!(h.text().contains("Local agent folder"), "{}", h.text());
        h.key(KeyCode::Tab);
        h.key(KeyCode::Enter);
        h.type_str(folder);
        h.key(KeyCode::Tab);
        h.tab_to(form_save);
        h.key(KeyCode::Enter);
        assert!(
            h.text().contains(&format!("Saved Claude · {name}")),
            "{}",
            h.text()
        );
    }

    for (name, provider_steps, item) in [("Primary", 1, "Codex Primary"), ("Team", 2, "Grok Team")]
    {
        h.key(KeyCode::Char('a'));
        h.key(KeyCode::Enter);
        h.type_str(name);
        h.key(KeyCode::Tab);
        h.key(KeyCode::Tab);
        for _ in 0..provider_steps {
            h.key(KeyCode::Down);
        }
        h.key(KeyCode::Tab);
        h.key(KeyCode::Tab);
        h.key(KeyCode::Enter);
        h.ticks(4);
        h.key(KeyCode::Enter);
        h.ticks(4);
        h.key(KeyCode::Enter);
        h.ticks(4);
        h.type_str(item);
        h.ticks(4);
        h.key(KeyCode::Enter);
        h.ticks(4);
        h.key(KeyCode::Enter);
        h.ticks(2);
        assert!(h.text().contains(item), "{}", h.text());
        h.tab_to(form_save);
        h.key(KeyCode::Enter);
        assert!(h.text().contains(&format!("· {name}")), "{}", h.text());
        assert!(!h.text().contains("valid-"), "secret leaked: {}", h.text());
    }
    assert!(
        h.app()
            .world
            .accounts
            .get("acct-xai-team")
            .is_some_and(|a| a.endpoint.is_some()),
        "Grok keeps its endpoint"
    );

    h.key(KeyCode::Char('a'));
    h.key(KeyCode::Enter);
    h.type_str("Go");
    h.key(KeyCode::Tab);
    h.key(KeyCode::Tab);
    for _ in 0..3 {
        h.key(KeyCode::Down);
    }
    h.key(KeyCode::Tab);
    h.key(KeyCode::Down);
    h.key(KeyCode::Down);
    h.key(KeyCode::Tab);
    h.key(KeyCode::Enter);
    h.type_str("oc_valid_abcdefghijklmn1234");
    h.key(KeyCode::Tab);
    assert!(!h.text().contains("abcdefghijklmn"), "{}", h.text());
    h.tab_to(form_save);
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Saved OpenCode · Go"), "{}", h.text());
    assert_eq!(h.app().world.accounts.accounts.len(), 5);

    h.key(KeyCode::Char('v'));
    h.ticks(20);
    assert!(
        h.text().contains("fingerprint") && h.text().contains("matches"),
        "{}",
        h.text()
    );
    h.key(KeyCode::Home);
    for _ in 0..3 {
        h.key(KeyCode::Down);
    }
    assert!(
        h.text().contains("Accounts › Claude › Work"),
        "{}",
        h.text()
    );
    h.key(KeyCode::Char(' '));
    assert!(h.text().contains("Default set"), "{}", h.text());

    h.key(KeyCode::Home);
    assert!(h.text().contains("Health"));
    h.key(KeyCode::Down);
    assert!(h.text().contains("Registration"), "{}", h.text());
    h.key(KeyCode::Down);
    assert!(h.text().contains("Quota"), "{}", h.text());

    h.key(KeyCode::Esc);
    assert_eq!(h.app().route(), Route::Manager);
    assert_eq!(h.focus(), Some(jackin_preview_app::screens::manager::TREE));

    h.key(KeyCode::End);
    h.key(KeyCode::Enter);
    assert_eq!(h.app().route(), Route::Prelude);
    h.key(KeyCode::Char(' '));
    assert!(h.text().contains("step 2 of 5"), "{}", h.text());
    h.key(KeyCode::Enter);
    h.key(KeyCode::Enter);
    assert!(h.text().contains("step 5 of 5"), "{}", h.text());
    h.key(KeyCode::Enter);
    assert_eq!(h.app().route(), Route::Editor, "{}", h.text());
    assert!(h.text().contains("new workspace › edit"));

    h.key(KeyCode::Char(']'));
    h.key(KeyCode::Enter);
    h.key(KeyCode::Char('i'));
    assert!(h.text().contains("worktree"), "{}", h.text());
    h.key(KeyCode::Esc);
    h.key(KeyCode::Char(']'));
    h.key(KeyCode::Enter);
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Default role ★"), "{}", h.text());
    h.key(KeyCode::Esc);
    h.key(KeyCode::Char(']'));
    h.key(KeyCode::Enter);
    h.key(KeyCode::Char('a'));
    h.key(KeyCode::Enter);
    h.type_str("API_BASE");
    h.key(KeyCode::Tab);
    h.key(KeyCode::Tab);
    h.key(KeyCode::Enter);
    h.type_str("https://api.internal");
    h.key(KeyCode::Tab);
    h.tab_to(cfg_save);
    h.key(KeyCode::Enter);
    assert!(h.text().contains("API_BASE"), "{}", h.text());
    h.key(KeyCode::Esc);
    h.key(KeyCode::Char(']'));
    h.key(KeyCode::Enter);

    assert!(h.text().contains("Active accounts"), "{}", h.text());
    let (_, py) = h.find("Personal").expect("Personal row");
    let (_, wy) = h.find("Work").expect("Work row");
    assert!(h.text().contains("inherited default"), "{}", h.text());

    if py > wy {
        h.key(KeyCode::Down);
    }
    h.key(KeyCode::Char(' '));
    assert!(h.text().contains("enabled here"), "{}", h.text());
    assert!(
        h.text()
            .contains("Claude · Personal · active for this Workspace"),
        "{}",
        h.text()
    );
    h.key(KeyCode::Char('p'));
    assert!(h.text().contains("Preferred for"), "{}", h.text());
    assert!(
        h.text()
            .contains("5 effective · 4 inherited · 1 enabled here"),
        "{}",
        h.text()
    );

    h.ctrl('s');
    assert!(h.text().contains("Create workspace"), "{}", h.text());
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    h.ticks(20);
    assert_eq!(h.app().route(), Route::Manager, "{}", h.text());
    assert_eq!(h.app().world.workspaces.len(), 1);

    h.key(KeyCode::Home);
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Launch · choose Agent"), "{}", h.text());
    h.key(KeyCode::Enter);
    assert_eq!(h.app().route(), Route::Cockpit, "{}", h.text());
    h.ticks(40);

    h.key(KeyCode::Char('b'));
    assert!(h.text().contains("Docker build"), "{}", h.text());
    h.key(KeyCode::PageUp);
    h.key(KeyCode::End);
    h.key(KeyCode::Esc);
    for _ in 0..60 {
        h.ticks(10);
        if h.app().route() != Route::Cockpit {
            break;
        }
    }
    h.ticks(15);

    assert_eq!(h.app().route(), Route::Capsule, "{}", h.text());
    h.ticks(40);
    h.type_str("hello");
    assert!(h.text().contains("hello"));
    let inst = h
        .app()
        .world
        .instances
        .iter()
        .find(|instance| {
            instance.status == jackin_preview_app::domain::instance::InstanceStatus::Running
        })
        .map(|instance| instance.id.clone())
        .expect("running instance");

    h.ctrl('b');
    h.key(KeyCode::Char('c'));
    assert!(h.text().contains("New tab"), "{}", h.text());
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Account for Claude Code"), "{}", h.text());
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    assert_eq!(h.app().world.daemons[&inst].tabs.len(), 2);
    assert!(h.text().contains("(Work)"), "{}", h.text());

    h.ctrl('b');
    h.key(KeyCode::Char('%'));
    h.key(KeyCode::Enter);
    if h.text().contains("Account for") {
        h.key(KeyCode::Enter);
    }
    assert_eq!(h.app().world.daemons[&inst].panes.len(), 3);
    h.ctrl('b');
    h.key(KeyCode::Char('h'));
    h.key_mod(
        KeyCode::Right,
        termrock::KeyModifiers::ALT | termrock::KeyModifiers::SHIFT,
    );
    h.draw();
    h.ctrl('b');
    h.key(KeyCode::Char('z'));
    assert!(h.text().contains("zoom"), "{}", h.text());
    h.ctrl('b');
    h.key(KeyCode::Char('z'));

    h.ticks(60);
    h.key(KeyCode::PageUp);
    h.key(KeyCode::End);
    let (x, y) = h.find("Refactor").expect("transcript line visible");
    h.mouse(MouseKind::Down, x, y);
    h.mouse(MouseKind::Drag, x + 8, y);
    h.mouse(MouseKind::Up, x + 8, y);
    assert_eq!(
        h.app().world.clipboard.as_deref(),
        Some("Refactor"),
        "{}",
        h.text()
    );

    h.app_mut().world.clipboard = None;
    h.mouse(MouseKind::Down, x + 2, y);
    h.mouse(MouseKind::Up, x + 2, y);
    assert_eq!(
        h.app().world.clipboard.as_deref(),
        Some("Refactor"),
        "{}",
        h.text()
    );
    h.app_mut().world.clipboard = None;
    h.key(KeyCode::Char('y'));
    assert_eq!(
        h.app().world.clipboard.as_deref(),
        Some("Refactor"),
        "{}",
        h.text()
    );
    h.key(KeyCode::End);

    h.ctrl('\\');
    assert!(
        h.text().contains("palette") || h.text().contains("Palette"),
        "{}",
        h.text()
    );
    h.key(KeyCode::Esc);

    h.ctrl('b');
    h.key(KeyCode::Char('u'));
    assert!(h.text().contains("Usage"), "{}", h.text());
    h.key(KeyCode::Esc);

    h.ctrl('b');
    h.key(KeyCode::Char('d'));
    assert_eq!(h.app().route(), Route::Manager);
    h.key(KeyCode::Enter);
    assert_eq!(h.app().route(), Route::Capsule);
    assert_eq!(h.app().world.daemons[&inst].tabs.len(), 2);

    h.ctrl('b');
    h.key(KeyCode::Char('d'));
    h.key(KeyCode::Home);
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    h.key(KeyCode::Enter);
    assert_eq!(h.app().route(), Route::Cockpit, "{}", h.text());
    for _ in 0..80 {
        h.ticks(10);
        if h.app().route() == Route::Capsule {
            break;
        }
    }
    h.ticks(15);
    assert_eq!(h.app().route(), Route::Capsule);
    assert_eq!(h.app().world.running_count(), 2);

    h.ctrl('q');
    if h.text().contains("Unsaved work") {
        h.key(KeyCode::Down);
        h.key(KeyCode::Down);
        h.key(KeyCode::Enter);
    } else {
        h.key(KeyCode::Right);
        h.key(KeyCode::Enter);
    }
    assert_eq!(h.app().route(), Route::Manager, "{}", h.text());
    assert!(
        h.text().contains("Still inside the Construct"),
        "{}",
        h.text()
    );
    assert_eq!(h.app().world.running_count(), 1);

    h.key(KeyCode::Home);
    h.key(KeyCode::Right);
    let running_id = h
        .app()
        .world
        .instances
        .iter()
        .find(|instance| {
            instance.status == jackin_preview_app::domain::instance::InstanceStatus::Running
        })
        .expect("one instance remains running")
        .id
        .clone();
    let running_key = jackin_preview_app::screens::manager::ManagerRowKey::Instance(running_id);
    h.key(KeyCode::Home);
    for _ in 0..=(h.app().world.instances.len() + h.app().world.workspaces.len()) {
        if h.app().manager.selected_row() == &running_key {
            break;
        }
        h.key(KeyCode::Down);
    }
    assert_eq!(h.app().manager.selected_row(), &running_key, "{}", h.text());
    h.key(KeyCode::Enter);
    assert_eq!(h.app().route(), Route::Capsule, "{}", h.text());
    h.ctrl('q');
    if h.text().contains("Unsaved work") {
        h.key(KeyCode::Down);
        h.key(KeyCode::Down);
        h.key(KeyCode::Enter);
    } else {
        h.key(KeyCode::Right);
        h.key(KeyCode::Enter);
    }

    assert_eq!(h.app().route(), Route::Outro, "{}", h.text());
    h.ticks(5);
    if !h.text().contains("You were in the Construct for") {
        h.key(KeyCode::Enter);
        h.ticks(25);
    }
    assert!(
        h.text().contains("You were in the Construct for"),
        "{}",
        h.text()
    );
    h.key(KeyCode::Enter);
    assert!(h.app().should_quit());
}

#[test]
fn complete_flow_keyboard_first() {
    complete_jackin_flow_keyboard_first();
}

#[test]
fn editor_accounts_tab_switches_inherited_defaults_off_and_extra_accounts_on() {
    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    h.key(KeyCode::Down);
    h.key(KeyCode::Char('e'));
    h.key(KeyCode::Char('5'));
    h.key(KeyCode::Enter);
    let t = h.text();
    assert!(t.contains("Active accounts"), "{t}");
    assert!(t.contains("inherited default"), "{t}");
    assert!(t.contains("enabled here"), "{t}");

    h.key(KeyCode::Char(' '));
    assert!(h.text().contains("off for this Workspace"), "{}", h.text());
    assert!(h.text().contains("disabled here"), "{}", h.text());
    {
        let ed = &h.app().editor;
        assert!(
            ed.pending
                .accounts
                .disabled_defaults
                .contains("acct-claude-personal")
        );
        let set = ed.pending.effective_accounts(&h.app().world.accounts);
        assert!(set.iter().all(|e| e.id != "acct-claude-personal"));
        assert!(
            set.iter()
                .any(|e| e.id == "acct-claude-work" && e.preferred)
        );
    }

    h.key(KeyCode::Char(' '));
    assert!(
        h.text().contains("active for this Workspace"),
        "{}",
        h.text()
    );

    let (x, y) = h.find("Experiments").expect("Experiments row");
    h.click(x, y);
    h.key(KeyCode::Char(' '));
    assert!(
        h.text()
            .contains("Codex · Experiments · active for this Workspace"),
        "{}",
        h.text()
    );
    {
        let ed = &h.app().editor;
        let codex: Vec<_> = ed
            .pending
            .effective_accounts(&h.app().world.accounts)
            .into_iter()
            .filter(|e| e.provider == jackin_preview_app::domain::agent::Provider::OpenAi)
            .collect();
        assert_eq!(codex.len(), 2);
    }
    assert!(h.text().contains("• 1 change"), "{}", h.text());

    h.key(KeyCode::Char('p'));
    assert!(h.text().contains("Preferred for OpenAI"), "{}", h.text());
    h.ctrl('s');
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    h.ticks(20);
    assert_eq!(h.app().route(), Route::Manager);
    let ws = h.app().world.workspace(1).unwrap();
    assert!(ws.accounts.enabled.contains("acct-codex-experiments"));
    assert_eq!(
        ws.accounts
            .preferred
            .get(&jackin_preview_app::domain::agent::Provider::OpenAi)
            .map(String::as_str),
        Some("acct-codex-experiments")
    );
    let r = h.app().world.account_for(
        jackin_preview_app::domain::agent::Provider::OpenAi,
        Some(ws),
        None,
        None,
    );
    assert_eq!(r.account.as_deref(), Some("acct-codex-experiments"));
}

#[test]
fn editor_accounts_enter_toggles_the_selected_account_after_tab_transition() {
    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    h.key(KeyCode::Down);
    h.key(KeyCode::Char('e'));
    h.key(KeyCode::Char('5'));
    h.key(KeyCode::Enter); // Clears the intentional Editor-tab transition.

    let account_index = h
        .app()
        .world
        .accounts
        .sorted()
        .iter()
        .position(|account| account.id == "acct-claude-personal")
        .expect("fixture has the inherited Claude account");
    let list_id = jackin_preview_app::screens::editor::ROOT.sub("accounts-list");
    let row = h
        .harness
        .area_of_part(
            list_id,
            PartRef::item(Part::ROW, termrock::ItemKey::index(account_index)),
        )
        .expect("the selected account has a real row target");
    assert!(row.width > 1 && row.height > 0, "account row: {row:?}");
    assert_eq!(h.focus(), Some(list_id));

    let before_disabled = h.app().editor.pending.accounts.disabled_defaults.clone();
    assert!(!before_disabled.contains("acct-claude-personal"));
    assert!(
        h.app()
            .editor
            .pending
            .effective_accounts(&h.app().world.accounts)
            .iter()
            .any(|account| account.id == "acct-claude-personal")
    );

    h.key(KeyCode::Enter);

    let mut expected_disabled = before_disabled;
    expected_disabled.insert("acct-claude-personal".to_owned());
    assert_eq!(
        h.app().editor.pending.accounts.disabled_defaults,
        expected_disabled,
        "one real Enter toggles only the selected inherited account"
    );
    let effective = h
        .app()
        .editor
        .pending
        .effective_accounts(&h.app().world.accounts);
    assert!(
        effective
            .iter()
            .all(|account| account.id != "acct-claude-personal")
    );
    assert!(
        effective
            .iter()
            .any(|account| account.id == "acct-claude-work")
    );
    assert!(h.text().contains("off for this Workspace"), "{}", h.text());

    let chosen = Theme::junie().design.glyphs.get(GlyphRole::Chosen);
    assert_eq!(
        h.harness.cell(row.x.saturating_add(1), row.y).symbol(),
        chosen,
        "Enter retains the chosen marker on the domain row"
    );
    assert!(
        h.harness.diagnostics().is_empty(),
        "{:?}",
        h.harness.diagnostics()
    );
}

#[test]
fn manager_launch_picker_hides_agents_without_an_account() {
    let mut h = H::new(Scenario::FirstUse, Motion::Reduced, 0, 120, 40);
    h.ticks(3);
    h.key(KeyCode::Enter);
    assert_eq!(h.app().route(), Route::Manager);

    h.key(KeyCode::Enter);
    assert_eq!(h.app().route(), Route::Manager);
    assert!(!h.harness.is_open(jackin_preview_app::LAUNCH_DIALOG));
    h.app_mut()
        .world
        .workspaces
        .push(jackin_preview_app::domain::fixtures::fixture_workspace());
    let mut a = jackin_preview_app::domain::account::Account::registered(
        "acct-only",
        "Only",
        jackin_preview_app::domain::agent::Provider::Anthropic,
        jackin_preview_app::domain::account::CredentialSource::LocalFolder {
            path: "~/.claude".into(),
            detected: jackin_preview_app::domain::account::DetectedKind::ClaudeOAuthProfile,
        },
    );
    a.default_for_provider = true;
    h.app_mut().world.accounts.insert(a);
    h.draw();
    let _ = h.harness.click_id(jackin_preview_app::LAUNCH);
    let t = h.text();
    assert!(t.contains("Launch · choose Agent"), "{t}");
    assert!(t.contains("Claude Code"), "{t}");
    assert!(!t.contains("Codex"), "unconfigured agent offered: {t}");
    assert!(!t.contains("Grok Build"), "unconfigured agent offered: {t}");
    assert!(!t.contains("needs account"), "{t}");
    assert!(!t.contains("no account"), "{t}");
}

#[test]
fn environments_stay_readable_with_a_hundred_roles() {
    let mut h = H::new(Scenario::HardCases, Motion::Reduced, 0, 120, 40);
    for _ in 0..8 {
        h.ticks(3);
        if h.app().route() == Route::Manager {
            break;
        }
        h.key(KeyCode::Enter);
    }
    assert!(h.app().world.roles.len() > 100);
    h.key(KeyCode::Down);
    h.key(KeyCode::Char('e'));
    h.key(KeyCode::Char('4'));
    h.key(KeyCode::Enter);
    let t = h.text();
    assert!(t.contains("Role overrides"), "{t}");
    assert!(t.contains("1 configured · "), "{t}");
    assert!(t.contains("Role: backend"), "{t}");
    assert!(
        !t.contains("Role: svc-"),
        "empty role sections rendered: {t}"
    );
    assert!(
        t.lines().filter(|l| l.contains("Role: ")).count() <= 2,
        "{t}"
    );
    assert!(t.contains("+ Add role override…"), "{t}");

    h.key(KeyCode::End);
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Add role override"), "{}", h.text());
    h.type_str("svc-01");
    h.key(KeyCode::Enter);
    assert!(
        h.text().contains("New svc-010 environment key"),
        "{}",
        h.text()
    );
    h.key(KeyCode::Enter);
    h.type_str("SVC_FLAG");
    h.key(KeyCode::Tab);
    h.key(KeyCode::Tab);
    h.key(KeyCode::Enter);
    h.type_str("on");
    h.key(KeyCode::Tab);
    h.tab_to(termrock::Id::root("editor.cfg").sub("form").sub("save"));
    h.key(KeyCode::Enter);
    let t = h.text();
    assert!(t.contains("Role: svc-010"), "{t}");
    assert!(t.contains("SVC_FLAG"), "{t}");
    assert!(t.contains("2 configured · "), "{t}");

    h.key(KeyCode::Esc);
    h.key(KeyCode::Char('3'));
    h.key(KeyCode::Enter);
    h.key(KeyCode::End);
    assert!(h.text().contains("+ Load role…"), "{}", h.text());
}

#[test]
fn cockpit_resolves_every_effective_account_for_the_container() {
    let mut h = H::new(Scenario::LaunchRunning, Motion::Reduced, 0, 120, 40);
    assert_eq!(h.app().route(), Route::Cockpit);
    let c = h.app().launch().expect("launch run");
    assert_eq!(
        c.agent,
        jackin_preview_app::domain::agent::Agent::ClaudeCode
    );
    let mut seen = false;
    for _ in 0..80 {
        h.ticks(5);
        let t = h.text();
        if t.contains("accounts · Claude ·")
            && t.contains("Claude · Work")
            && t.contains("Claude · Personal")
        {
            seen = true;
            break;
        }
        if h.app().route() != Route::Cockpit {
            break;
        }
    }
    assert!(
        seen,
        "credentials line never listed the second account: {}",
        h.text()
    );
    for _ in 0..60 {
        h.ticks(10);
        if h.app().route() != Route::Cockpit {
            break;
        }
    }
    h.ticks(15);
    assert_eq!(h.app().route(), Route::Capsule);
    let inst = h
        .app()
        .world
        .instances
        .iter()
        .find(|instance| {
            instance.status == jackin_preview_app::domain::instance::InstanceStatus::Running
        })
        .map(|instance| instance.id.clone())
        .expect("running instance");
    let i = h.app().world.instance(&inst).unwrap();
    assert!(i.accounts.len() >= 2, "{:?}", i.accounts);
}

#[test]
fn prelude_up_follows_one_source_row_and_ignores_a_jump() {
    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    h.key(KeyCode::End);
    h.key(KeyCode::Enter);
    assert_eq!(h.app().route(), Route::Prelude);
    assert!(h.text().contains("▸ payments-platform/"), "{}", h.text());
    h.key(KeyCode::Down);
    assert!(h.text().contains("▸ customer-portal/"), "{}", h.text());
    h.key(KeyCode::Up);
    assert!(h.text().contains("▸ payments-platform/"), "{}", h.text());
    assert!(
        !h.text().contains("▸ customer-portal/"),
        "up moved more than one row: {}",
        h.text()
    );
    assert_eq!(h.app().prelude.selection(), 0);
}

#[test]
fn usage_enter_opens_the_read_only_detail_and_escape_closes_it() {
    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    h.key(KeyCode::Char('u'));
    assert_eq!(h.app().route(), Route::Usage);
    assert!(h.text().contains("Usage · read-only"));
    assert!(!h.app().usage.detail_open());
    h.key(KeyCode::Enter);
    assert!(h.app().usage.detail_open());
    assert!(
        h.text().contains("Usage detail · read-only"),
        "{}",
        h.text()
    );
    assert!(h.text().contains("Surface ·"), "{}", h.text());
    h.key(KeyCode::Esc);
    assert!(!h.app().usage.detail_open());
    assert!(h.text().contains("Usage · read-only"), "{}", h.text());
    h.key(KeyCode::Down);
    assert!(h.text().contains("Limits"), "{}", h.text());
    assert_eq!(h.app().route(), Route::Usage);
}

#[test]
fn settings_route_s_paints_the_approved_settings_frame() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 120, 40);
    assert!(
        !h.text().contains("Settings › global"),
        "accounts list must stay off the settings frame: {}",
        h.text()
    );
    h.key(KeyCode::Char('s'));
    assert_eq!(h.app().route(), Route::Settings);
    assert_eq!(h.app().settings.tab, 0);
    assert!(!h.app().settings.dirty);
    assert!(h.text().contains("Settings › global"), "{}", h.text());
    let crumb = h.buffer().cell((56, 0)).expect("settings breadcrumb");
    assert_eq!(crumb.symbol(), "S");
    assert_eq!(crumb.fg, termrock::Color::from_u32(0xb3_b3_b3));
    assert_eq!(crumb.bg, termrock::Color::from_u32(0x00_00_00));
    assert!(!crumb.modifier.contains(termrock::Modifier::BOLD));
    let mark = h.buffer().cell((2, 0)).expect("jackin prompt");
    assert_eq!(mark.symbol(), "j");
    assert_eq!(mark.fg, termrock::Color::from_u32(0x19_19_1c));
    assert_eq!(mark.bg, termrock::Color::from_u32(0x48_e0_54));
    assert!(mark.modifier.contains(termrock::Modifier::BOLD));
    let tab = h.buffer().cell((2, 3)).expect("general tab mark");
    assert_eq!(tab.symbol(), "▎");
    assert_eq!(tab.fg, termrock::Color::from_u32(0x48_e0_54));
    assert_eq!(tab.bg, termrock::Color::from_u32(0x27_27_2a));
    assert!(tab.modifier.contains(termrock::Modifier::BOLD));
    assert_eq!(h.harness.cursor(), None);
    h.key(KeyCode::Char('5'));
    assert_eq!(h.app().settings.tab, 0);
    h.key(KeyCode::Enter);
    assert_eq!(h.app().settings.tab, 4);
    let general = h.buffer().cell((2, 3)).expect("left of general");
    assert_ne!(
        general.symbol(),
        "▎",
        "Trust must not keep the General gutter: {}",
        h.text()
    );
}

#[test]
fn settings_route_80_paints_the_approved_breadcrumb_cell() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 80, 24);
    h.key(KeyCode::Char('s'));
    assert_eq!(h.app().route(), Route::Settings);
    assert_eq!(h.app().settings.tab, 0);
    let crumb = h.buffer().cell((51, 0)).expect("settings breadcrumb");
    assert_eq!(crumb.symbol(), "S");
    assert_eq!(crumb.fg, termrock::Color::from_u32(0xb3_b3_b3));
    assert_eq!(crumb.bg, termrock::Color::from_u32(0x00_00_00));
    assert!(!crumb.modifier.contains(termrock::Modifier::BOLD));
    assert!(!crumb.modifier.contains(termrock::Modifier::ITALIC));
    assert_eq!(h.harness.cursor(), None);
}

#[test]
fn settings_mounts_80_paints_the_approved_destination_cell() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 80, 24);
    h.key(KeyCode::Char('s'));
    assert_eq!(h.app().settings.tab, 0);
    let general = h.buffer().cell((8, 6)).expect("general body");
    assert_ne!(general.symbol(), "D", "General must not show Destination");
    h.key(KeyCode::Char('2'));
    assert_eq!(h.app().settings.tab, 0);
    h.key(KeyCode::Enter);
    assert_eq!(h.app().settings.tab, 1);
    let dest = h.buffer().cell((8, 6)).expect("destination heading");
    assert_eq!(dest.symbol(), "D");
    assert_eq!(dest.fg, termrock::Color::from_u32(0x8a_8a_8a));
    assert_eq!(dest.bg, termrock::Color::from_u32(0x00_00_00));
    assert!(!dest.modifier.contains(termrock::Modifier::BOLD));
    assert!(!dest.modifier.contains(termrock::Modifier::ITALIC));
    assert_eq!(h.harness.cursor(), None);
}

#[test]
fn settings_env_three_enter_paints_the_approved_env_frame() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 120, 40);
    h.key(KeyCode::Char('s'));
    assert_eq!(h.app().settings.tab, 0);
    let general = h.buffer().cell((8, 7)).expect("general body");
    assert_ne!(general.symbol(), "G", "General must not show GH_TOKEN");
    h.key(KeyCode::Char('3'));
    assert_eq!(h.app().settings.tab, 0);
    let armed = h.buffer().cell((8, 7)).expect("armed general body");
    assert_ne!(armed.symbol(), "G", "`3` alone must stay on General");
    h.key(KeyCode::Enter);
    assert_eq!(h.app().settings.tab, 2);
    let token = h.buffer().cell((8, 7)).expect("GH_TOKEN");
    assert_eq!(token.symbol(), "G");
    assert_eq!(token.fg, termrock::Color::from_u32(0xff_ff_ff));
    assert_eq!(token.bg, termrock::Color::from_u32(0x17_3f_1c));
    assert!(token.modifier.contains(termrock::Modifier::BOLD));
    assert!(h.text().contains("GH_TOKEN"), "{}", h.text());
    assert_eq!(h.harness.cursor(), None);

    let mut mounts = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 120, 40);
    mounts.key(KeyCode::Char('s'));
    mounts.key(KeyCode::Char('2'));
    mounts.key(KeyCode::Enter);
    assert_eq!(mounts.app().settings.tab, 1);
    let mount_cell = mounts.buffer().cell((8, 7)).expect("mounts body");
    assert_ne!(mount_cell.symbol(), "G", "Mounts must not show GH_TOKEN");
    assert!(
        !mounts.text().contains("GH_TOKEN"),
        "Mounts must stay off the environment table: {}",
        mounts.text()
    );
}

#[test]
fn settings_trust_five_enter_paints_the_approved_trust_frame() {
    let closed = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 120, 40);
    assert!(
        !closed.text().contains("github.com/chainargos/roles"),
        "closed settings must stay off the trust frame: {}",
        closed.text()
    );

    let mut agents = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 120, 40);
    agents.key(KeyCode::Char('s'));
    agents.key(KeyCode::Char('4'));
    agents.key(KeyCode::Enter);
    assert_eq!(agents.app().settings.tab, 3);
    assert!(
        !agents.text().contains("github.com/chainargos/roles"),
        "Agents must stay off the trust frame: {}",
        agents.text()
    );

    let mut h = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 120, 40);
    h.key(KeyCode::Char('s'));
    assert_eq!(h.app().settings.tab, 0);
    h.key(KeyCode::Char('5'));
    assert_eq!(h.app().settings.tab, 0);
    assert!(
        !h.text().contains("github.com/chainargos/roles"),
        "`5` alone must stay on General: {}",
        h.text()
    );
    h.key(KeyCode::Enter);
    assert_eq!(h.app().settings.tab, 4);
    let role = h.buffer().cell((7, 7)).expect("chainargos role");
    assert_eq!(role.symbol(), "g");
    assert_eq!(role.fg, termrock::Color::from_u32(0xff_ff_ff));
    assert_eq!(role.bg, termrock::Color::from_u32(0x17_3f_1c));
    assert!(role.modifier.contains(termrock::Modifier::BOLD));
    assert!(
        h.text().contains("github.com/chainargos/roles"),
        "{}",
        h.text()
    );
    assert_eq!(h.harness.cursor(), None);
}

#[test]
fn settings_save_preview_keys_paint_the_approved_dialog() {
    let apply_save = |h: &H| {
        let cell = h.buffer().cell((85, 24)).expect("apply save");
        cell.symbol() == "S"
            && cell.fg == termrock::Color::from_u32(0x19_19_1c)
            && cell.bg == termrock::Color::from_u32(0x48_e0_54)
            && cell.modifier.contains(termrock::Modifier::BOLD)
            && cell.modifier.contains(termrock::Modifier::UNDERLINED)
    };

    let closed = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 120, 40);
    assert!(
        !apply_save(&closed),
        "closed settings must stay off the save preview: {}",
        closed.text()
    );

    let mut trust = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 120, 40);
    trust.key(KeyCode::Char('s'));
    trust.key(KeyCode::Char('5'));
    trust.key(KeyCode::Enter);
    assert_eq!(trust.app().settings.tab, 4);
    assert!(
        !apply_save(&trust),
        "clean Trust must stay off the save preview: {}",
        trust.text()
    );

    let mut h = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 120, 40);
    h.key(KeyCode::Char('s'));
    h.key(KeyCode::Char('5'));
    assert_eq!(h.app().settings.tab, 0);
    h.key(KeyCode::Enter);
    h.key(KeyCode::Char(' '));
    h.key(KeyCode::Down);
    h.key(KeyCode::Char(' '));
    h.ctrl('s');
    assert_eq!(h.app().settings.tab, 4);
    let cell = h.buffer().cell((85, 24)).expect("apply save");
    assert_eq!(cell.symbol(), "S");
    assert_eq!(cell.fg, termrock::Color::from_u32(0x19_19_1c));
    assert_eq!(cell.bg, termrock::Color::from_u32(0x48_e0_54));
    assert!(cell.modifier.contains(termrock::Modifier::BOLD));
    assert!(cell.modifier.contains(termrock::Modifier::UNDERLINED));
    assert!(apply_save(&h));
    assert!(h.text().contains("Save settings"), "{}", h.text());
    assert_eq!(h.harness.cursor(), None);
}

#[test]
fn settings_agents_four_enter_paints_the_approved_agents_frame() {
    let closed = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 120, 40);
    assert!(
        !closed.text().contains("Agent runtime mode"),
        "closed settings must stay off the agents frame: {}",
        closed.text()
    );

    let mut env = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 120, 40);
    env.key(KeyCode::Char('s'));
    env.key(KeyCode::Char('3'));
    env.key(KeyCode::Enter);
    assert_eq!(env.app().settings.tab, 2);
    assert!(
        !env.text().contains("Agent runtime mode"),
        "Environments must stay off the agents frame: {}",
        env.text()
    );

    let mut h = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 120, 40);
    h.key(KeyCode::Char('s'));
    assert_eq!(h.app().settings.tab, 0);
    let general = h.buffer().cell((4, 6)).expect("general body");
    assert_ne!(
        general.symbol(),
        "A",
        "General must not show Agent runtime mode"
    );
    h.key(KeyCode::Char('4'));
    assert_eq!(h.app().settings.tab, 0);
    let armed = h.buffer().cell((4, 6)).expect("armed general body");
    assert_ne!(armed.symbol(), "A", "`4` alone must stay on General");
    h.key(KeyCode::Enter);
    assert_eq!(h.app().settings.tab, 3);
    let mode = h.buffer().cell((4, 6)).expect("agent runtime mode");
    assert_eq!(mode.symbol(), "A");
    assert_eq!(mode.fg, termrock::Color::from_u32(0xb3_b3_b3));
    assert_eq!(mode.bg, termrock::Color::from_u32(0x00_00_00));
    assert!(mode.modifier.contains(termrock::Modifier::BOLD));
    assert!(h.text().contains("Agent runtime mode"), "{}", h.text());
    assert_eq!(h.harness.cursor(), None);
}

#[test]
fn settings_mounts_two_enter_paints_the_approved_mounts_frame() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 120, 40);
    h.key(KeyCode::Char('s'));
    assert_eq!(h.app().settings.tab, 0);
    let general = h.buffer().cell((8, 6)).expect("general body");
    assert_ne!(general.symbol(), "D", "General must not show Destination");
    h.key(KeyCode::Char('2'));
    assert_eq!(h.app().settings.tab, 0);
    let armed = h.buffer().cell((8, 6)).expect("armed general body");
    assert_ne!(armed.symbol(), "D", "`2` alone must stay on General");
    h.key(KeyCode::Enter);
    assert_eq!(h.app().settings.tab, 1);
    let dest = h.buffer().cell((8, 6)).expect("destination heading");
    assert_eq!(dest.symbol(), "D");
    assert_eq!(dest.fg, termrock::Color::from_u32(0x8a_8a_8a));
    assert_eq!(dest.bg, termrock::Color::from_u32(0x00_00_00));
    assert!(!dest.modifier.contains(termrock::Modifier::BOLD));
    assert!(h.text().contains("/home/agent/.gitconfig"), "{}", h.text());
    assert_eq!(h.harness.cursor(), None);
}

#[test]
fn accounts_detail_downs_paint_the_approved_work_frame() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 120, 40);
    assert!(
        !h.text().contains("Accounts › Claude › Work"),
        "closed list must stay off the detail frame: {}",
        h.text()
    );
    for _ in 0..4 {
        h.key(KeyCode::Down);
    }
    assert_eq!(
        h.app().accounts.selected_id.as_deref(),
        Some("acct-claude-work")
    );
    assert_eq!(
        h.app().accounts.list.cursor(),
        Some(termrock::ItemKey::index(4))
    );
    assert!(
        h.text().contains("Accounts › Claude › Work"),
        "{}",
        h.text()
    );
    let (crumb_x, crumb_y) = h.find("Accounts › Claude › Work").expect("work breadcrumb");
    let crumb = h
        .buffer()
        .cell((crumb_x, crumb_y))
        .expect("work breadcrumb");
    assert_eq!(crumb.symbol(), "A");
    assert_eq!(crumb.fg, termrock::Color::from_u32(0xb3_b3_b3));
    assert_eq!(crumb.bg, termrock::Color::from_u32(0x00_00_00));
    assert!(!crumb.modifier.contains(termrock::Modifier::BOLD));
    let mark = h.buffer().cell((2, 0)).expect("jackin prompt");
    assert_eq!(mark.symbol(), "j");
    assert_eq!(mark.fg, termrock::Color::from_u32(0x19_19_1c));
    assert_eq!(mark.bg, termrock::Color::from_u32(0x48_e0_54));
    assert!(mark.modifier.contains(termrock::Modifier::BOLD));
    let title = h.buffer().cell((44, 2)).expect("work title");
    assert_eq!(title.symbol(), "C");
    assert_eq!(title.fg, termrock::Color::from_u32(0xb3_b3_b3));
    assert_eq!(title.bg, termrock::Color::from_u32(0x11_11_11));
    assert_eq!(h.harness.cursor(), None);
    h.key(KeyCode::Down);
    assert_ne!(
        h.app().accounts.selected_id.as_deref(),
        Some("acct-claude-work")
    );
    assert!(
        !h.text().contains("Accounts › Claude › Work"),
        "the next row must keep the live list: {}",
        h.text()
    );
}

#[test]
fn accounts_detail_downs_paint_the_approved_80_work_cell() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 80, 24);
    let closed = h.buffer().cell((38, 0)).expect("overview crumb");
    assert_ne!(
        closed.symbol(),
        "A",
        "overview must stay off the detail crumb"
    );
    for _ in 0..4 {
        h.key(KeyCode::Down);
    }
    assert_eq!(
        h.app().accounts.selected_id.as_deref(),
        Some("acct-claude-work")
    );
    let crumb = h.buffer().cell((38, 0)).expect("work breadcrumb");
    assert_eq!(crumb.symbol(), "A");
    assert_eq!(crumb.fg, termrock::Color::from_u32(0xb3_b3_b3));
    assert_eq!(crumb.bg, termrock::Color::from_u32(0x00_00_00));
    assert!(!crumb.modifier.contains(termrock::Modifier::BOLD));
    assert_eq!(h.harness.cursor(), None);
}

#[test]
fn accounts_drawer_enter_paints_the_approved_work_drawer() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 120, 40);
    for _ in 0..4 {
        h.key(KeyCode::Down);
    }
    let defaults_before = h
        .app()
        .world
        .accounts
        .accounts
        .iter()
        .filter(|account| {
            account.provider == jackin_preview_app::domain::agent::Provider::Anthropic
        })
        .filter(|account| account.default_for_provider)
        .map(|account| account.id.clone())
        .collect::<Vec<_>>();
    assert!(!h.app().accounts.drawer_open);
    assert!(
        !h.text().contains("Tab Actions"),
        "detail must stay off the drawer footer: {}",
        h.text()
    );
    h.key(KeyCode::Enter);
    assert!(h.app().accounts.drawer_open);
    assert_eq!(
        h.app().accounts.selected_id.as_deref(),
        Some("acct-claude-work")
    );
    assert_eq!(
        h.app()
            .world
            .accounts
            .accounts
            .iter()
            .filter(|account| account.provider
                == jackin_preview_app::domain::agent::Provider::Anthropic)
            .filter(|account| account.default_for_provider)
            .map(|account| account.id.clone())
            .collect::<Vec<_>>(),
        defaults_before,
        "Enter opens the drawer without changing the provider default"
    );
    assert!(h.text().contains("Tab Actions"), "{}", h.text());
    // The open drawer keeps the crumb: the selection is still the account.
    assert!(
        h.text().contains("Accounts › Claude › Work"),
        "{}",
        h.text()
    );
    let mark = h.buffer().cell((2, 0)).expect("jackin prompt");
    assert_eq!(mark.symbol(), "j");
    assert_eq!(mark.fg, termrock::Color::from_u32(0x19_19_1c));
    assert_eq!(mark.bg, termrock::Color::from_u32(0x48_e0_54));
    assert!(mark.modifier.contains(termrock::Modifier::BOLD));
    let title = h.buffer().cell((44, 2)).expect("drawer title");
    assert_eq!(title.symbol(), "C");
    assert_eq!(title.fg, termrock::Color::from_u32(0xff_ff_ff));
    assert_eq!(title.bg, termrock::Color::from_u32(0x11_11_11));
    assert!(title.modifier.contains(termrock::Modifier::BOLD));
    let tab = h.buffer().cell((12, 39)).expect("drawer tab hint");
    assert_eq!(tab.symbol(), "T");
    assert_eq!(tab.fg, termrock::Color::from_u32(0xff_ff_ff));
    assert_eq!(tab.bg, termrock::Color::from_u32(0x00_00_00));
    assert!(tab.modifier.contains(termrock::Modifier::BOLD));
    assert_eq!(h.harness.cursor(), None);
    h.key(KeyCode::Esc);
    assert!(!h.app().accounts.drawer_open);
    assert!(
        !h.text().contains("Tab Actions"),
        "escape must leave the drawer frame: {}",
        h.text()
    );
}

#[test]
fn accounts_row_click_selects_without_opening_drawer_or_mutating_account() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 120, 40);
    let account_id = "acct-claude-personal";
    let account = h
        .app()
        .world
        .accounts
        .get(account_id)
        .expect("fixture account");
    let source_before = account.source.clone();
    let enabled_before = account.enabled;
    let defaults_before = h
        .app()
        .world
        .accounts
        .accounts
        .iter()
        .filter(|account| account.default_for_provider)
        .map(|account| account.id.clone())
        .collect::<Vec<_>>();
    let registry_revision = h.app().world.accounts.revision;
    let account_count = h.app().world.accounts.accounts.len();

    let (x, y) = h.find("★ Personal").expect("painted account row");
    assert!(x > 0 && x < 40, "click must hit the left tree: ({x}, {y})");
    h.click(x, y);

    assert_eq!(
        h.app().accounts.list.cursor(),
        Some(termrock::ItemKey::index(2)),
        "the click must select the painted account row"
    );
    assert_eq!(h.app().accounts.selected_id.as_deref(), Some(account_id));
    assert!(
        !h.app().accounts.drawer_open,
        "a row click selects the account without opening its drawer"
    );
    assert!(!h.text().contains("Tab Actions"), "{}", h.text());
    let selected = h
        .app()
        .world
        .accounts
        .get(account_id)
        .expect("selected fixture account");
    assert!(
        selected.source == source_before,
        "click must preserve credentials"
    );
    assert_eq!(selected.enabled, enabled_before);
    assert_eq!(
        h.app()
            .world
            .accounts
            .accounts
            .iter()
            .filter(|account| account.default_for_provider)
            .map(|account| account.id.clone())
            .collect::<Vec<_>>(),
        defaults_before,
        "click must not change a provider default"
    );
    assert_eq!(h.app().world.accounts.revision, registry_revision);
    assert_eq!(h.app().world.accounts.accounts.len(), account_count);
    assert!(
        h.harness.diagnostics().is_empty(),
        "{:?}",
        h.harness.diagnostics()
    );
}

#[test]
fn accounts_click_closes_open_drawer_for_same_and_different_rows() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 120, 40);
    let (personal_x, personal_y) = h.find("★ Personal").expect("Personal row");
    assert!(
        personal_x < 40,
        "Personal must be in the tree: {personal_x}"
    );
    h.click(personal_x, personal_y);
    assert_eq!(
        h.app().accounts.selected_id.as_deref(),
        Some("acct-claude-personal")
    );
    assert!(!h.app().accounts.drawer_open);

    h.key(KeyCode::Enter);
    assert!(h.app().accounts.drawer_open, "root Enter opens the drawer");
    assert!(h.text().contains("Tab Actions"), "{}", h.text());
    advance_past_junie_double_click_window(&mut h);

    let (same_x, same_y) = h.find("★ Personal").expect("Personal row while open");
    assert!(same_x < 40, "same-row click must target the tree: {same_x}");
    h.click(same_x, same_y);
    assert_eq!(
        h.app().accounts.selected_id.as_deref(),
        Some("acct-claude-personal")
    );
    assert!(
        !h.app().accounts.drawer_open,
        "same-row click closes the drawer; cursor={:?}, chosen={:?}, focus={:?}, diagnostics={:?}",
        h.app().accounts.list.cursor(),
        h.app().accounts.list.chosen(),
        h.focus(),
        h.harness.diagnostics()
    );
    assert!(!h.text().contains("Tab Actions"), "{}", h.text());
    assert!(
        h.text().contains("Enter Details"),
        "detail hints remain visible: {}",
        h.text()
    );
    assert_eq!(h.focus(), Some(ACCOUNTS_LIST));

    h.key(KeyCode::Enter);
    assert!(h.app().accounts.drawer_open);
    advance_past_junie_double_click_window(&mut h);
    let (work_x, work_y) = h.find("Work").expect("Work row while drawer is open");
    assert!(
        work_x < 40,
        "different-row click must target the tree: {work_x}"
    );
    h.click(work_x, work_y);
    assert_eq!(
        h.app().accounts.selected_id.as_deref(),
        Some("acct-claude-work")
    );
    assert!(
        !h.app().accounts.drawer_open,
        "different-row click closes the drawer"
    );
    assert!(!h.text().contains("Tab Actions"), "{}", h.text());
    assert!(
        h.text().contains("Enter Details"),
        "detail hints remain visible: {}",
        h.text()
    );
    assert_eq!(h.focus(), Some(ACCOUNTS_LIST));
    assert!(
        h.harness.diagnostics().is_empty(),
        "{:?}",
        h.harness.diagnostics()
    );
}

fn advance_past_junie_double_click_window(h: &mut H) {
    let threshold = Theme::junie().design.motion.double_click_ms;
    let _ = h.harness.advance(std::time::Duration::from_millis(
        threshold.saturating_add(1),
    ));
}

#[test]
fn accounts_double_click_activation_does_not_open_the_drawer() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 120, 40);
    let (x, y) = h.find("★ Personal").expect("Personal row");
    assert!(x > 0 && x < 40);

    let _ = h.harness.double_click(x, y);

    assert_eq!(
        h.app().accounts.selected_id.as_deref(),
        Some("acct-claude-personal")
    );
    assert!(!h.app().accounts.drawer_open);
    assert!(!h.text().contains("Tab Actions"), "{}", h.text());
    assert!(
        h.harness.diagnostics().is_empty(),
        "{:?}",
        h.harness.diagnostics()
    );
}

#[test]
fn accounts_row_click_keeps_selection_only_at_supported_narrow_size() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 80, 24);
    let (x, y) = h.find("★ Personal").expect("painted Personal row at 80x24");
    assert!(x > 0 && x < 40, "click must hit the left tree: ({x}, {y})");
    h.click(x, y);
    assert_eq!(
        h.app().accounts.selected_id.as_deref(),
        Some("acct-claude-personal")
    );
    assert!(!h.app().accounts.drawer_open);
    assert!(!h.text().contains("Tab Actions"), "{}", h.text());
    assert_eq!(h.focus(), Some(ACCOUNTS_LIST));
    assert!(
        h.harness.diagnostics().is_empty(),
        "{:?}",
        h.harness.diagnostics()
    );
}

#[test]
fn accounts_space_sets_work_default_once_without_opening_drawer() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 120, 40);
    for _ in 0..4 {
        h.key(KeyCode::Down);
    }
    assert_eq!(
        h.app().accounts.selected_id.as_deref(),
        Some("acct-claude-work")
    );
    assert!(!h.app().accounts.drawer_open);
    let revision_before = h.app().world.accounts.revision;

    h.key(KeyCode::Char(' '));

    assert!(!h.app().accounts.drawer_open);
    assert_eq!(h.app().world.accounts.revision, revision_before + 1);
    assert!(
        h.app()
            .world
            .accounts
            .get("acct-claude-work")
            .unwrap()
            .default_for_provider
    );
    assert!(
        !h.app()
            .world
            .accounts
            .get("acct-claude-personal")
            .unwrap()
            .default_for_provider
    );
    assert!(h.text().contains("Default set"), "{}", h.text());
    h.ticks(3);
    assert_eq!(h.app().world.accounts.revision, revision_before + 1);
    assert!(!h.app().accounts.drawer_open);
}

#[test]
fn accounts_filter_click_dismisses_input_and_selects_without_opening_drawer() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 120, 40);
    h.key(KeyCode::Char('/'));
    assert!(h.app().accounts.filter_input.is_editing());
    assert_eq!(
        h.focus(),
        Some(jackin_preview_app::screens::accounts::FILTER)
    );
    assert!(h.harness.cursor().is_some(), "filter owns a visible caret");
    h.type_str("work");
    assert_eq!(h.app().accounts.filter_query, "work");
    assert!(h.app().accounts.filter_input.is_editing());

    let (x, y) = h.find("Work").expect("filtered Work result");
    assert!(
        x > 0 && x < 40,
        "click must hit the filtered tree: ({x}, {y})"
    );
    h.click(x, y);

    assert_eq!(
        h.app().accounts.selected_id.as_deref(),
        Some("acct-claude-work")
    );
    assert!(!h.app().accounts.drawer_open);
    assert!(h.app().accounts.filtering);
    assert!(!h.app().accounts.filter_input.is_editing());
    assert_eq!(h.app().accounts.filter_query, "work");
    assert_eq!(h.focus(), Some(ACCOUNTS_LIST));
    assert!(h.text().contains("Accounts · filt"), "{}", h.text());
    assert!(!h.text().contains("Tab Actions"), "{}", h.text());
    assert!(
        h.harness.diagnostics().is_empty(),
        "{:?}",
        h.harness.diagnostics()
    );
}

#[test]
fn accounts_form_owns_enter_and_space_without_mutating_the_underlay() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Reduced, 0, 120, 40);
    let selected_before = h.app().accounts.selected_id.clone();
    let registry_before = h.app().world.accounts.clone();
    let defaults_before = h
        .app()
        .world
        .accounts
        .accounts
        .iter()
        .filter(|account| account.default_for_provider)
        .map(|account| account.id.clone())
        .collect::<Vec<_>>();

    start_live_account_form(&mut h);
    // The inline form owns the route input while its fields are active. Its
    // underlying account tree is intentionally not addressable through the
    // form's painted controls, so verify the real form-owned key path here.
    h.key(KeyCode::Enter);
    h.key(KeyCode::Char(' '));

    assert_eq!(h.app().route(), Route::Accounts);
    assert!(h.app().accounts.form_open);
    assert!(!h.app().accounts.drawer_open);
    assert_eq!(h.app().accounts.selected_id, selected_before);
    assert_eq!(h.app().world.accounts, registry_before);
    assert_eq!(
        h.app()
            .world
            .accounts
            .accounts
            .iter()
            .filter(|account| account.default_for_provider)
            .map(|account| account.id.clone())
            .collect::<Vec<_>>(),
        defaults_before
    );
    assert!(h.harness.is_open(ACCOUNT_FORM_LAYER));
    assert!(
        h.harness.diagnostics().is_empty(),
        "{:?}",
        h.harness.diagnostics()
    );
}

#[test]
fn usage_detail_80_paints_back_to_list() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 80, 24);
    h.key(KeyCode::Char('u'));
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    assert!(h.app().usage.detail_open());
    assert!(h.text().contains("Back to list"), "{}", h.text());
    assert!(h.text().contains("Session · 5-hour"), "{}", h.text());
    let back = h.buffer().cell((49, 23)).expect("back to list");
    assert_eq!(back.symbol(), "B");
    assert_eq!(back.fg, termrock::Color::Rgb(0x8a, 0x8a, 0x8a));
}

#[test]
fn usage_overview_80_paints_the_narrow_account_tree() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 80, 24);
    h.key(KeyCode::Char('u'));
    assert_eq!(h.app().route(), Route::Usage);
    let ellipsis = h.buffer().cell((37, 0)).expect("crumb ellipsis");
    assert_eq!(ellipsis.symbol(), "…");
    assert_eq!(ellipsis.fg, termrock::Color::Rgb(0x8a, 0x8a, 0x8a));
    let gutter = h.buffer().cell((4, 3)).expect("overview gutter");
    assert_eq!(gutter.symbol(), "▎");
    assert_eq!(gutter.fg, termrock::Color::Rgb(0x48, 0xe0, 0x54));
    assert_eq!(gutter.bg, termrock::Color::Rgb(0x17, 0x3f, 0x1c));
    let title = h.buffer().cell((7, 3)).expect("overview label");
    assert_eq!(title.symbol(), "O");
    assert!(h.text().contains("disabled"), "{}", h.text());
    assert!(h.text().contains("Esc Close"), "{}", h.text());
    assert!(
        !h.text().contains("Jackin Preview"),
        "the narrow usage page keeps the jackin menu: {}",
        h.text()
    );
}

#[test]
fn usage_overview_u_paints_the_approved_overview_frame() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 120, 40);
    assert!(
        !h.text().contains("Usage › Overview"),
        "accounts list must stay off the usage frame: {}",
        h.text()
    );
    h.key(KeyCode::Char('u'));
    assert_eq!(h.app().route(), Route::Usage);
    assert_eq!(
        h.app().usage.tab,
        jackin_preview_app::screens::usage::Tab::Overview
    );
    assert!(!h.app().usage.detail_open());
    assert!(h.text().contains("Usage › Overview"), "{}", h.text());
    let crumb = h.buffer().cell((67, 0)).expect("usage breadcrumb");
    assert_eq!(crumb.symbol(), "U");
    assert_eq!(crumb.fg, termrock::Color::from_u32(0xb3_b3_b3));
    assert_eq!(crumb.bg, termrock::Color::from_u32(0x00_00_00));
    assert!(!crumb.modifier.contains(termrock::Modifier::BOLD));
    let mark = h.buffer().cell((2, 0)).expect("jackin prompt");
    assert_eq!(mark.symbol(), "j");
    assert_eq!(mark.fg, termrock::Color::from_u32(0x19_19_1c));
    assert_eq!(mark.bg, termrock::Color::from_u32(0x48_e0_54));
    assert!(mark.modifier.contains(termrock::Modifier::BOLD));
    let tab = h.buffer().cell((4, 3)).expect("overview tab mark");
    assert_eq!(tab.symbol(), "▎");
    assert_eq!(tab.fg, termrock::Color::from_u32(0x48_e0_54));
    assert_eq!(tab.bg, termrock::Color::from_u32(0x17_3f_1c));
    assert!(tab.modifier.contains(termrock::Modifier::BOLD));
    assert_eq!(h.harness.cursor(), None);
    h.key(KeyCode::Down);
    assert_ne!(
        h.app().usage.tab,
        jackin_preview_app::screens::usage::Tab::Overview
    );
    assert!(
        !h.text().contains("Usage › Overview"),
        "the next tab must keep the live usage page: {}",
        h.text()
    );
}

#[test]
fn usage_detail_down_enter_paints_the_approved_detail_frame() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 120, 40);
    let back_before = h.buffer().cell((49, 39)).expect("accounts footer");
    assert_ne!(
        back_before.symbol(),
        "B",
        "accounts list must stay off the detail footer"
    );
    h.key(KeyCode::Char('u'));
    assert_eq!(h.app().route(), Route::Usage);
    assert_eq!(
        h.app().usage.tab,
        jackin_preview_app::screens::usage::Tab::Overview
    );
    assert!(!h.app().usage.detail_open());
    let back_overview = h.buffer().cell((49, 39)).expect("overview footer");
    assert_ne!(
        back_overview.symbol(),
        "B",
        "overview host must stay off the detail footer: {}",
        h.text()
    );
    assert!(h.text().contains("Usage › Overview"), "{}", h.text());
    h.key(KeyCode::Down);
    assert_eq!(
        h.app().usage.tab,
        jackin_preview_app::screens::usage::Tab::Registration
    );
    assert!(!h.app().usage.detail_open());
    let back_tab = h.buffer().cell((49, 39)).expect("registration footer");
    assert_ne!(back_tab.symbol(), "B", "{}", h.text());
    h.key(KeyCode::Enter);
    assert!(h.app().usage.detail_open());
    assert!(h.text().contains("Back to list"), "{}", h.text());
    assert!(
        h.text().contains("Usage › Claude › Personal"),
        "{}",
        h.text()
    );
    let back = h.buffer().cell((49, 39)).expect("detail back hint");
    assert_eq!(back.symbol(), "B");
    assert_eq!(back.fg, termrock::Color::from_u32(0x8a_8a_8a));
    assert_eq!(back.bg, termrock::Color::from_u32(0x00_00_00));
    assert!(!back.modifier.contains(termrock::Modifier::BOLD));
    let mark = h.buffer().cell((2, 0)).expect("jackin prompt");
    assert_eq!(mark.symbol(), "j");
    assert_eq!(mark.fg, termrock::Color::from_u32(0x19_19_1c));
    assert_eq!(mark.bg, termrock::Color::from_u32(0x48_e0_54));
    assert!(mark.modifier.contains(termrock::Modifier::BOLD));
    assert_eq!(h.harness.cursor(), None);
}

#[test]
fn settings_tabs_and_save_preview_stay_on_the_settings_route() {
    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    h.key(KeyCode::Char('s'));
    assert_eq!(h.app().route(), Route::Settings);
    assert_eq!(h.app().settings.tab, 0);
    assert!(h.text().contains("▸ General"), "{}", h.text());
    h.key(KeyCode::Char('5'));
    h.key(KeyCode::Enter);
    assert_eq!(h.app().settings.tab, 4);
    assert!(h.text().contains("▸ Trust"), "{}", h.text());
    assert!(!h.text().contains("▸ General"), "{}", h.text());
    h.key(KeyCode::Char(' '));
    h.ctrl('s');
    let preview = h.text();
    assert!(preview.contains("Save settings"), "{preview}");
    assert!(preview.contains("Scope · global config"), "{preview}");
    assert!(preview.contains("Changes · trust draft"), "{preview}");
    assert_eq!(h.app().route(), Route::Settings);
}

#[test]
fn accounts_filter_enter_paints_the_approved_filter_frame() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 120, 40);
    assert!(
        !h.text().contains("Accounts · filt"),
        "closed list must stay off the filter frame: {}",
        h.text()
    );
    h.key(KeyCode::Char('/'));
    h.type_str("work");
    h.key(KeyCode::Enter);
    assert!(
        !h.app().accounts.drawer_open,
        "filter Enter is owned by its field"
    );
    assert!(h.app().accounts.filtering);
    assert!(!h.app().accounts.filter_input.is_editing());
    assert_eq!(h.app().accounts.filter_query, "work");
    assert!(h.text().contains("Accounts · filt"), "{}", h.text());
    let title = h.buffer().cell((4, 2)).expect("filter title");
    assert_eq!(title.symbol(), "A");
    assert_eq!(title.fg, termrock::Color::from_u32(0xff_ff_ff));
    assert_eq!(title.bg, termrock::Color::from_u32(0x00_00_00));
    assert!(title.modifier.contains(termrock::Modifier::BOLD));
    let mark = h.buffer().cell((2, 0)).expect("jackin prompt");
    assert_eq!(mark.symbol(), "j");
    assert_eq!(mark.fg, termrock::Color::from_u32(0x19_19_1c));
    assert_eq!(mark.bg, termrock::Color::from_u32(0x48_e0_54));
    assert_eq!(h.harness.cursor(), None);
}

#[test]
fn accounts_filter_enter_paints_the_approved_80_filter_cell() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 80, 24);
    assert!(
        !h.text().contains("Accounts · filt"),
        "closed list must stay off the filter frame: {}",
        h.text()
    );
    h.key(KeyCode::Char('/'));
    h.type_str("work");
    h.key(KeyCode::Enter);
    assert!(
        !h.app().accounts.drawer_open,
        "filter Enter is owned by its field"
    );
    assert!(h.app().accounts.filtering);
    assert!(!h.app().accounts.filter_input.is_editing());
    assert_eq!(h.app().accounts.filter_query, "work");
    let title = h.buffer().cell((4, 2)).expect("filter title");
    assert_eq!(title.symbol(), "A");
    assert_eq!(title.fg, termrock::Color::from_u32(0xff_ff_ff));
    assert_eq!(title.bg, termrock::Color::from_u32(0x00_00_00));
    assert!(title.modifier.contains(termrock::Modifier::BOLD));
    assert_eq!(h.harness.cursor(), None);
}

#[test]
fn accounts_filter_and_inspector_open_without_writing_a_credential() {
    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    h.key(KeyCode::Char('a'));
    assert_eq!(h.app().route(), Route::Accounts);
    assert!(!h.app().accounts.filtering);
    assert!(!h.app().accounts.drawer_open);
    let before = h.app().world.accounts.accounts.len();
    let visible_name = h
        .app()
        .world
        .accounts
        .sorted()
        .iter()
        .map(|account| account.display_name.clone())
        .find(|name| h.text().contains(name))
        .expect("closed accounts list paints an account name");
    h.key(KeyCode::Char('/'));
    assert!(h.app().accounts.filtering);
    assert!(h.text().contains("Filter accounts"), "{}", h.text());
    for ch in ['z', 'z', 'z'] {
        h.key(KeyCode::Char(ch));
    }
    assert_eq!(h.app().accounts.filter_query, "zzz");
    assert!(
        !h.text().contains(&visible_name),
        "filter left {visible_name} visible: {}",
        h.text()
    );
    h.key(KeyCode::Esc);
    assert!(!h.app().accounts.filtering);
    assert!(!h.text().contains("Filter accounts"), "{}", h.text());
    h.key(KeyCode::Char('i'));
    assert!(h.app().accounts.drawer_open);
    assert!(h.text().contains("Tab Actions"), "{}", h.text());
    h.key(KeyCode::Esc);
    assert!(!h.app().accounts.drawer_open);
    assert_eq!(h.app().route(), Route::Accounts);
    assert_eq!(h.app().world.accounts.accounts.len(), before);
    assert!(h.app().world.clipboard.is_none());
}

#[test]
fn manager_help_overlay_paints_the_approved_shortcuts_cell() {
    let mut h = H::new(Scenario::Returning, Motion::Paused, 40, 120, 40);
    let closed = h.buffer().cell((15, 4)).expect("manager cell");
    assert_ne!(closed.symbol(), "K");
    assert!(
        !h.text().contains("Keyboard shortcuts"),
        "manager without ? must stay off the help frame: {}",
        h.text()
    );
    h.key(KeyCode::Char('?'));
    let title = h.buffer().cell((15, 4)).expect("keyboard shortcuts");
    assert_eq!(title.symbol(), "K");
    assert_eq!(title.fg, termrock::Color::from_u32(0x00_ff_ff_ff));
    assert_eq!(title.bg, termrock::Color::from_u32(0x00_18_18_1b));
    assert!(title.modifier.contains(termrock::Modifier::BOLD));
    assert!(h.text().contains("Keyboard shortcuts"), "{}", h.text());
    assert_eq!(h.harness.cursor(), None);
}

#[test]
fn manager_launch_picker_paints_the_approved_agent_cell() {
    let mut h = H::new(Scenario::Returning, Motion::Paused, 40, 120, 40);
    let closed = h.buffer().cell((20, 16)).expect("manager cell");
    assert_ne!(
        (closed.symbol(), closed.bg),
        ("L", termrock::Color::from_u32(0x00_18_18_1b)),
        "closed manager must stay off the launch picker frame: {}",
        h.text()
    );
    h.key(KeyCode::Enter);
    let title = h.buffer().cell((20, 16)).expect("launch picker title");
    assert_eq!(title.symbol(), "L");
    assert_eq!(title.fg, termrock::Color::from_u32(0x00_ff_ff_ff));
    assert_eq!(title.bg, termrock::Color::from_u32(0x00_18_18_1b));
    assert!(title.modifier.contains(termrock::Modifier::BOLD));
    assert!(!title.modifier.contains(termrock::Modifier::ITALIC));
    assert!(h.text().contains("Launch · choose Agent"), "{}", h.text());
    assert_eq!(h.harness.cursor(), None);
}

#[test]
fn manager_file_menu_paints_the_approved_menu_frame() {
    let mut h = H::new(Scenario::Returning, Motion::Paused, 40, 120, 40);
    assert!(
        !h.text().contains("New workspace…"),
        "closed manager must stay off the menu frame: {}",
        h.text()
    );
    h.key(KeyCode::F(10));
    assert!(h.text().contains("New workspace…"), "{}", h.text());
    let item = h.buffer().cell((15, 2)).expect("new workspace");
    assert_eq!(item.symbol(), "N");
    assert_eq!(item.fg, termrock::Color::from_u32(0x00_ff_ff_ff));
    assert_eq!(item.bg, termrock::Color::from_u32(0x00_2f_5a_a8));
    assert!(item.modifier.contains(termrock::Modifier::BOLD));
    let mark = h.buffer().cell((2, 0)).expect("jackin prompt");
    assert_eq!(mark.symbol(), "j");
    assert_eq!(mark.fg, termrock::Color::from_u32(0x00_19_19_1c));
    assert_eq!(mark.bg, termrock::Color::from_u32(0x00_48_e0_54));
    assert_eq!(h.harness.cursor(), None);
}

#[test]
fn paused_returning_down_paints_the_expanded_payments_tree() {
    let mut h = H::new(Scenario::Returning, Motion::Paused, 40, 120, 40);
    assert!(h.text().contains("▸ payments-platform"), "{}", h.text());
    assert!(!h.text().contains("Prewarm"), "{}", h.text());
    h.key(KeyCode::Down);
    h.key(KeyCode::Char(' '));
    let text = h.text();
    assert!(text.contains("▾"), "{text}");
    assert!(text.contains("the-archite…ude Code"), "{text}");
    assert!(text.contains("saved workspace · 1 running"), "{text}");
    assert!(text.contains("Prewarm"), "{text}");
    assert!(text.contains("→"), "{text}");
    assert!(text.contains("Workspaces › payments-platform"), "{text}");
}

fn start_live_account_form(h: &mut H) {
    h.key(KeyCode::Char('a'));
    assert_eq!(h.app().route(), Route::Accounts);
    assert!(h.app().accounts.form_open);
    assert_eq!(
        h.focus(),
        Some(jackin_preview_app::screens::accounts::START)
    );

    h.key(KeyCode::Enter);
    assert!(h.app().accounts.started);
    assert_eq!(h.focus(), Some(jackin_preview_app::screens::accounts::NAME));
}

#[test]
fn account_name_escape_reverts_the_field_without_closing_the_form() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Reduced, 0, 120, 40);
    start_live_account_form(&mut h);

    h.type_str("discard this field edit");
    assert!(h.app().accounts.name_input.is_editing());
    assert!(h.text().contains("discard this field edit"), "{}", h.text());

    h.key(KeyCode::Esc);

    assert_eq!(h.app().route(), Route::Accounts);
    assert!(
        h.app().accounts.form_open,
        "field Esc must not cancel the form"
    );
    assert!(!h.app().accounts.name_input.is_editing());
    assert_eq!(h.app().accounts.draft_name, "");
    assert!(h.text().contains("New account · register"), "{}", h.text());
}

#[test]
fn account_form_cancel_preserves_the_selected_account_and_registry() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Reduced, 0, 120, 40);
    for _ in 0..4 {
        h.key(KeyCode::Down);
    }
    let selected_id = h
        .app()
        .accounts
        .selected_id
        .clone()
        .expect("four Down inputs select Claude · Work");
    assert_eq!(selected_id, "acct-claude-work");
    let list_before = h.app().accounts.list.clone();
    let registry_before = h.app().world.accounts.clone();

    start_live_account_form(&mut h);
    h.type_str("temporary account");
    h.key(KeyCode::Enter);
    assert_eq!(h.app().accounts.draft_name, "temporary account");
    h.key(KeyCode::Esc);

    assert_eq!(h.app().route(), Route::Accounts);
    assert!(!h.app().accounts.form_open);
    assert_eq!(
        h.app().accounts.selected_id.as_deref(),
        Some(selected_id.as_str())
    );
    assert_eq!(h.app().accounts.list, list_before);
    assert_eq!(h.app().world.accounts, registry_before);
    assert_eq!(h.focus(), Some(ACCOUNTS_LIST));
}

#[test]
fn account_form_cancel_clears_name_and_credential_draft_immediately() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Reduced, 0, 120, 40);
    start_live_account_form(&mut h);

    h.type_str("temporary account");
    h.key(KeyCode::Enter);
    h.tab_to(jackin_preview_app::screens::accounts::SOURCE);
    h.key(KeyCode::Down);
    h.key(KeyCode::Down);
    assert_eq!(h.app().accounts.source_index, 2);
    h.tab_to(jackin_preview_app::screens::accounts::SECRET);
    h.key(KeyCode::Enter);
    h.type_str("sk-cancel-secret-1234");
    assert!(!h.text().contains("sk-cancel-secret-1234"));
    h.key(KeyCode::Enter);
    assert_eq!(h.app().accounts.masked_input, "sk-cancel-secret-1234");

    h.key(KeyCode::Esc);

    assert!(!h.app().accounts.form_open);
    assert_eq!(h.app().accounts.draft_name, "");
    assert_eq!(h.app().accounts.masked_input, "");
    assert!(h.app().accounts.selected_op.is_none());
    assert_eq!(h.app().accounts.op_item, "");
    assert_eq!(h.app().accounts.op_stage, 0);
}

#[test]
fn account_picker_escape_closes_only_the_nested_picker() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Reduced, 0, 120, 40);
    start_live_account_form(&mut h);
    h.tab_to(jackin_preview_app::screens::accounts::OP);
    h.key(KeyCode::Enter);
    assert!(
        h.harness
            .layer_area(jackin_preview_app::ACCOUNT_PICKER)
            .is_some(),
        "the real 1Password picker should own input above the form"
    );

    h.key(KeyCode::Esc);

    assert!(
        h.harness
            .layer_area(jackin_preview_app::ACCOUNT_PICKER)
            .is_none(),
        "Esc should close the nested picker"
    );
    assert_eq!(h.app().route(), Route::Accounts);
    assert!(
        h.app().accounts.form_open,
        "Esc must leave the parent form open"
    );
    assert_eq!(h.app().accounts.op_stage, 0);
    assert!(h.app().accounts.selected_op.is_none());
    assert!(h.app().world.accounts.get("acct-claude-work").is_some());
}

#[test]
fn account_form_ignores_clicks_on_blank_content_outside_its_controls() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Reduced, 0, 120, 40);
    start_live_account_form(&mut h);
    h.type_str("draft survives an outside click");
    assert_eq!(
        h.app().accounts.name_input.draft_text(),
        Some("draft survives an outside click")
    );
    let focus = h.focus();
    let registry = h.app().world.accounts.clone();

    // The live inline form has no control at this blank lower-right content cell.
    let _ = h.harness.click(118, 30);

    assert_eq!(h.app().route(), Route::Accounts);
    assert!(h.app().accounts.form_open);
    assert_eq!(
        h.app().accounts.name_input.draft_text(),
        Some("draft survives an outside click")
    );
    assert_eq!(h.focus(), focus);
    assert_eq!(h.app().world.accounts, registry);
}

#[test]
fn account_form_does_not_let_an_underlying_navigation_click_change_route() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Reduced, 0, 120, 40);
    start_live_account_form(&mut h);
    h.type_str("draft stays owned by the form");
    assert_eq!(
        h.app().accounts.name_input.draft_text(),
        Some("draft stays owned by the form")
    );
    let registry = h.app().world.accounts.clone();
    let (manager_x, manager_y) = h.find("Manager").expect("underlay Manager navigation");
    let _ = h.harness.click(manager_x, manager_y);

    assert_eq!(h.app().route(), Route::Accounts);
    assert!(h.app().accounts.form_open);
    assert_eq!(
        h.app().accounts.name_input.draft_text(),
        Some("draft stays owned by the form")
    );
    assert_eq!(h.app().world.accounts, registry);
}

#[test]
fn account_form_consumes_every_underlying_route_key() {
    for route_key in ['m', 'a', 'u', 's', 'c'] {
        let mut h = H::new(Scenario::AccountsMixed, Motion::Reduced, 0, 120, 40);
        h.key(KeyCode::Char('a'));
        assert!(h.app().accounts.form_open);
        assert!(!h.app().accounts.started);
        let registry = h.app().world.accounts.clone();

        h.key(KeyCode::Char(route_key));

        assert_eq!(h.app().route(), Route::Accounts, "route key {route_key:?}");
        assert!(h.app().accounts.form_open, "route key {route_key:?}");
        assert!(!h.app().accounts.started, "route key {route_key:?}");
        assert_eq!(h.app().world.accounts, registry, "route key {route_key:?}");
        assert!(
            h.harness.layer_area(ACCOUNT_FORM_LAYER).is_some(),
            "route key {route_key:?} must leave the host open"
        );
    }
}

#[test]
fn nested_account_picker_keeps_the_parent_form_route_keys_blocked() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Reduced, 0, 120, 40);
    start_live_account_form(&mut h);
    h.tab_to(jackin_preview_app::screens::accounts::OP);
    h.key(KeyCode::Enter);
    assert!(h.harness.is_open(jackin_preview_app::ACCOUNT_PICKER));

    h.key(KeyCode::Char('m'));

    assert_eq!(h.app().route(), Route::Accounts);
    assert!(h.app().accounts.form_open);
    assert!(h.harness.is_open(jackin_preview_app::ACCOUNT_PICKER));
    assert!(h.harness.is_open(ACCOUNT_FORM_LAYER));
}

#[test]
fn account_add_save_does_not_overwrite_the_selected_existing_account() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Reduced, 0, 120, 40);
    for _ in 0..4 {
        h.key(KeyCode::Down);
    }
    let selected_id = h.app().accounts.selected_id.clone().unwrap();
    assert_eq!(selected_id, "acct-claude-work");
    let existing = h.app().world.accounts.get(&selected_id).unwrap().clone();
    let account_count = h.app().world.accounts.accounts.len();

    start_live_account_form(&mut h);
    h.type_str("Fresh");
    h.tab_to(jackin_preview_app::screens::accounts::SOURCE);
    h.key(KeyCode::Down);
    h.key(KeyCode::Down);
    h.tab_to(jackin_preview_app::screens::accounts::SECRET);
    h.key(KeyCode::Enter);
    h.type_str("sk-ant-new-account-abcdef1234");
    h.key(KeyCode::Enter);
    h.tab_to(jackin_preview_app::screens::accounts::FORM.sub("save"));
    h.key(KeyCode::Enter);

    assert!(!h.app().accounts.form_open);
    assert_eq!(h.app().world.accounts.accounts.len(), account_count + 1);
    assert_eq!(h.app().world.accounts.get(&selected_id), Some(&existing));
    assert!(h.app().world.accounts.get("acct-anthropic-fresh").is_some());
    assert_eq!(
        h.app().accounts.selected_id.as_deref(),
        Some("acct-anthropic-fresh")
    );
}

#[test]
fn account_add_cancel_reconciles_when_the_return_account_disappears() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Reduced, 0, 120, 40);
    for _ in 0..4 {
        h.key(KeyCode::Down);
    }
    let removed_id = h.app().accounts.selected_id.clone().unwrap();
    start_live_account_form(&mut h);
    h.app_mut()
        .world
        .accounts
        .remove(&removed_id)
        .expect("selected account exists before removal");
    h.key(KeyCode::Enter);

    h.key(KeyCode::Esc);

    assert!(!h.app().accounts.form_open);
    assert_ne!(
        h.app().accounts.selected_id.as_deref(),
        Some(removed_id.as_str())
    );
    assert!(
        h.app()
            .accounts
            .selected_id
            .as_ref()
            .is_none_or(|id| { h.app().world.accounts.get(id).is_some() })
    );
}

#[test]
fn account_form_focus_continuation_is_acknowledged_across_repeated_open_close() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Reduced, 0, 120, 40);
    for _ in 0..4 {
        h.key(KeyCode::Down);
    }
    let selected_id = h.app().accounts.selected_id.clone();
    let list_cursor = h.app().accounts.list.cursor();
    assert_eq!(selected_id.as_deref(), Some("acct-claude-work"));

    for cycle in 0..36 {
        h.key(KeyCode::Char('a'));
        assert!(h.app().accounts.form_open, "open cycle {cycle}");
        assert!(h.harness.is_open(ACCOUNT_FORM_LAYER), "layer cycle {cycle}");
        assert!(
            h.harness.diagnostics().is_empty(),
            "open cycle {cycle}: {:?}",
            h.harness.diagnostics()
        );

        h.key(KeyCode::Esc);
        assert!(!h.app().accounts.form_open, "close cycle {cycle}");
        assert_eq!(h.focus(), Some(ACCOUNTS_LIST), "focus cycle {cycle}");
        assert_eq!(
            h.app().accounts.selected_id,
            selected_id,
            "selection cycle {cycle}"
        );
        assert_eq!(
            h.app().accounts.list.cursor(),
            list_cursor,
            "cursor cycle {cycle}"
        );
        assert!(
            h.harness.diagnostics().is_empty(),
            "close cycle {cycle}: {:?}",
            h.harness.diagnostics()
        );
    }

    // Runtime continuation receipts expire after 32 presented frames unless the
    // owner applies and acknowledges the settled focus result.
    h.ticks(40);
    assert!(
        h.harness.diagnostics().is_empty(),
        "post-close continuation expiry: {:?}",
        h.harness.diagnostics()
    );
}

#[test]
fn account_add_invalid_save_keeps_the_same_form_open_for_correction() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Reduced, 0, 120, 40);
    start_live_account_form(&mut h);
    let registry = h.app().world.accounts.clone();

    h.tab_to(jackin_preview_app::screens::accounts::FORM.sub("save"));
    h.key(KeyCode::Enter);

    assert!(h.app().accounts.form_open);
    assert!(h.app().accounts.started);
    assert!(h.text().contains("Choose a 1Password reference first"));
    assert_eq!(h.app().world.accounts, registry);
    assert!(h.harness.is_open(ACCOUNT_FORM_LAYER));
    assert!(
        h.harness.diagnostics().is_empty(),
        "{:?}",
        h.harness.diagnostics()
    );
}

#[test]
fn account_edit_save_and_cancel_use_the_live_host_without_touching_credentials() {
    let mut save = H::new(Scenario::AccountsMixed, Motion::Reduced, 0, 120, 40);
    for _ in 0..4 {
        save.key(KeyCode::Down);
    }
    save.key(KeyCode::Char('e'));
    assert!(save.app().accounts.form_open);
    assert!(save.harness.is_open(ACCOUNT_FORM_LAYER));
    let id = save
        .app()
        .accounts
        .editing
        .clone()
        .expect("selected account edit");
    let before = save.app().world.accounts.get(&id).unwrap().clone();
    save.type_str("-updated");
    save.tab_to(jackin_preview_app::screens::accounts::FORM.sub("save"));
    save.key(KeyCode::Enter);

    assert!(!save.app().accounts.form_open);
    let updated = save
        .app()
        .world
        .accounts
        .get(&id)
        .expect("edited account remains");
    assert_eq!(updated.display_name, "Work-updated");
    assert_eq!(updated.source, before.source);
    assert_eq!(updated.id, before.id);
    assert!(!save.harness.is_open(ACCOUNT_FORM_LAYER));
    assert_eq!(save.focus(), Some(ACCOUNTS_LIST));
    assert!(
        save.harness.diagnostics().is_empty(),
        "{:?}",
        save.harness.diagnostics()
    );

    let mut cancel = H::new(Scenario::AccountsMixed, Motion::Reduced, 0, 120, 40);
    for _ in 0..4 {
        cancel.key(KeyCode::Down);
    }
    let id = cancel.app().accounts.selected_id.clone().unwrap();
    let registry = cancel.app().world.accounts.clone();
    cancel.key(KeyCode::Char('e'));
    cancel.type_str("-discarded");
    cancel.key(KeyCode::Esc);
    assert!(
        cancel.app().accounts.form_open,
        "first Esc reverts the field"
    );
    assert_eq!(cancel.app().accounts.draft_name, "Work");
    assert!(cancel.harness.is_open(ACCOUNT_FORM_LAYER));
    assert!(
        cancel.harness.diagnostics().is_empty(),
        "field revert frame: {:?}",
        cancel.harness.diagnostics()
    );
    cancel.key(KeyCode::Esc);

    assert!(!cancel.app().accounts.form_open);
    assert_eq!(cancel.app().world.accounts, registry);
    assert_eq!(
        cancel.app().accounts.selected_id.as_deref(),
        Some(id.as_str())
    );
    assert!(!cancel.harness.is_open(ACCOUNT_FORM_LAYER));
    assert_eq!(cancel.focus(), Some(ACCOUNTS_LIST));
    assert!(
        cancel.harness.diagnostics().is_empty(),
        "{:?}",
        cancel.harness.diagnostics()
    );
}
