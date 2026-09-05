//! End-to-end interaction tests through the public runtime harness.

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

use junie_tui::{Id, KeyCode, MouseKind};

use jackin_app::Route;
use jackin_app::{Motion, Scenario};
mod support;
use support::H;

#[test]
fn first_use_plays_intro_then_manager_and_no_replay_when_returning() {
    let mut h = H::new(Scenario::FirstUse, Motion::Full, 0, 120, 40);
    assert_eq!(h.app().route(), Route::Intro);
    h.ticks(45);
    assert!(h.text().contains("Stand up, operator…"), "{}", h.text());
    assert!(h.text().contains("jackin❯"));
    // skip during phrases jumps to the warp, then finishes into the manager
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
    h.key(KeyCode::Down);
    assert!(h.text().contains("payments-platform"));
    h.key(KeyCode::Right);
    assert!(
        h.text().contains("7f3a"),
        "instance children visible after expand"
    );
    h.key(KeyCode::Down);
    h.key(KeyCode::Tab);
    assert!(h.text().contains("Live topology"));
    h.key(KeyCode::Esc);
    assert_eq!(h.focus(), Some(jackin_app::screens::manager::TREE));
    // mouse: click the row of infra-control-plane
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
    // type into the pane and see the echo
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
    h.key(KeyCode::Enter); // exit & keep
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
    // the same reference already backs Claude · Work: duplicate protection refuses
    h.tab_to(jackin_app::screens::accounts::FORM.sub("save"));
    h.key(KeyCode::Enter);
    assert!(
        h.text()
            .contains("Already registered: this source is used by Claude · Work"),
        "{}",
        h.text()
    );
    assert!(h.app().world.accounts.get("acct-anthropic-team").is_none());
    // switch to Codex and pick the throttled sandbox item instead
    h.tab_to(jackin_app::screens::accounts::FORM.sub("provider"));
    h.key(KeyCode::Down);
    assert!(h.text().contains("Codex · OpenAI"), "{}", h.text());
    h.tab_to(jackin_app::screens::accounts::FORM.sub("op"));
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
    h.tab_to(jackin_app::screens::accounts::FORM.sub("save"));
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Saved Codex · Team"), "{}", h.text());
    assert!(h.text().contains("Rate limited"), "{}", h.text());
    assert!(!h.text().contains("throttled-thr01"));
    assert!(h.app().world.accounts.get("acct-openai-team").is_some());
    // refresh the new account: the job completes and the status reports it honestly
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

/// §16.4's stable name for the secret-frame acceptance journey.  Keep the
/// product-specific test above as the readable inventory entry while exposing
/// the architecture name used by the cross-workspace gate.
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
    h.tab_to(jackin_app::screens::accounts::FORM.sub("save"));
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Saved Claude · Spare"), "{}", h.text());
    assert!(!h.text().contains("abcdef"));
    let a = h.app().world.accounts.get("acct-anthropic-spare").unwrap();
    assert!(
        matches!(&a.source, jackin_app::domain::account::CredentialSource::PlainApiKey { tail, .. } if tail == "1234")
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
    // up to ~/src, choose data-pipeline (second folder)
    h.key(KeyCode::Backspace);
    assert!(h.text().contains("customer-portal/"), "{}", h.text());
    h.key(KeyCode::Down);
    h.key(KeyCode::Down);
    h.key(KeyCode::Char(' '));
    assert!(h.text().contains("step 2 of 5"), "{}", h.text());
    assert!(
        h.text()
            .contains("Same path   /Users/alexey/src/data-pipeline"),
        "{}",
        h.text()
    );
    assert!(h.text().contains("✓ Source"));
    // Esc rewinds to the browser at the same folder, Space re-chooses
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
    h.key(KeyCode::Char(' ')); // customer-portal: an existing workspace name
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
    // rewind all the way out
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
    // leaving asks first
    h.key(KeyCode::Esc);
    h.key(KeyCode::Esc);
    assert!(
        h.text().contains("Save changes before leaving?"),
        "{}",
        h.text()
    );
    h.key(KeyCode::Esc);
    assert_eq!(h.app().route(), Route::Editor);
    // preview lists the modified mount, then the save job completes
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
        jackin_app::domain::workspace::Isolation::Clone
    );
}

#[test]
fn editor_env_plain_value_stays_masked_and_can_be_shown() {
    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    h.key(KeyCode::Down);
    h.key(KeyCode::Char('e'));
    h.key(KeyCode::Char('4'));
    h.key(KeyCode::Enter);
    let t = h.text();
    assert!(t.contains("DATABASE_URL"));
    assert!(!t.contains("pw-fixture-only"), "plain value leaked: {t}");
    h.key(KeyCode::Char('m'));
    assert!(h.text().contains("postgres://"), "{}", h.text());
    assert!(h.text().contains("plain · shown"));
    h.key(KeyCode::Char('m'));
    assert!(!h.text().contains("pw-fixture-only"));
    // add a variable through the form
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
    h.tab_to(junie_tui::Id::root("editor.cfg").sub("form").sub("save"));
    h.key(KeyCode::Enter);
    let t = h.text();
    assert!(t.contains("NEW_SECRET"), "{t}");
    assert!(t.contains("************1234"), "{t}");
    assert!(!t.contains("abcdefghijklmnop"));
    assert!(t.contains("• 1 change"), "{t}");
}

/// §16.4's stable name for the controlled secret draft journey.
#[test]
fn form_dialog_toggles_visibility_and_keeps_drafts() {
    editor_env_plain_value_stays_masked_and_can_be_shown();
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
    h.ctrl('s');
    assert!(h.text().contains("Save settings"));
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    h.ticks(20);
    assert!(h.text().contains("Settings error"), "{}", h.text());
    h.key(KeyCode::Esc);
    assert_eq!(h.app().route(), Route::Settings);
    assert!(h.text().contains("• 1 change"));
    // second attempt succeeds
    h.ctrl('s');
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    h.ticks(20);
    assert_eq!(h.app().route(), Route::Manager, "{}", h.text());
    // the manager's own refresh may overwrite the status in the hard cases;
    // the persisted config is the proof
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

/// Section 34 of the goal: the connected keyboard-first journey from a
/// fresh Construct with zero instances to the final outro.
#[test]
fn complete_jackin_flow_keyboard_first() {
    use junie_tui::Id;
    let form_save = jackin_app::screens::accounts::FORM.sub("save");
    let cfg_save = Id::root("editor.cfg").sub("form").sub("save");
    let mut h = H::new(Scenario::FirstUse, Motion::Reduced, 0, 120, 40);
    // 1–3 intro → manager with zero instances
    assert_eq!(h.app().route(), Route::Intro);
    h.ticks(3);
    h.key(KeyCode::Enter);
    assert_eq!(h.app().route(), Route::Manager);
    assert_eq!(h.app().world.running_count(), 0);
    // 4 Account & Usage Center
    h.key(KeyCode::Char('c'));
    assert_eq!(h.app().route(), Route::Accounts);
    // 5–6 two Claude Code local-folder accounts
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
    // 7–8 Codex and Grok Build through 1Password references
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
    // 9 OpenCode through the masked plain-text fallback
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
    // 10 validate one, set a provider default
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
    // 11–12 overview, one provider, one account
    h.key(KeyCode::Home);
    assert!(h.text().contains("Health"));
    h.key(KeyCode::Down);
    assert!(h.text().contains("Registration"), "{}", h.text());
    h.key(KeyCode::Down);
    assert!(h.text().contains("Quota"), "{}", h.text());
    // 13 back to the manager, focus on the tree
    h.key(KeyCode::Esc);
    assert_eq!(h.app().route(), Route::Manager);
    assert_eq!(h.focus(), Some(jackin_app::screens::manager::TREE));
    // 14 create a workspace through the prelude (current directory as source)
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
    // 15 configure every tab
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
    // 16 activate and prefer the non-default Claude account for this Workspace
    assert!(h.text().contains("Active accounts"), "{}", h.text());
    let (_, py) = h.find("Personal").expect("Personal row");
    let (_, wy) = h.find("Work").expect("Work row");
    assert!(h.text().contains("inherited default"), "{}", h.text());
    // move onto the Personal row (rows are provider-grouped; the default sorts first)
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
    // 17 preview and save
    h.ctrl('s');
    assert!(h.text().contains("Create workspace"), "{}", h.text());
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    h.ticks(20);
    assert_eq!(h.app().route(), Route::Manager, "{}", h.text());
    assert_eq!(h.app().world.workspaces.len(), 1);
    // 18–20 launch: already inside the Construct, straight to the cockpit
    h.key(KeyCode::Home);
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Launch · choose Agent"), "{}", h.text());
    h.key(KeyCode::Enter);
    assert_eq!(h.app().route(), Route::Cockpit, "{}", h.text());
    h.ticks(40);
    // 21 build log
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
    // 22–23 capsule, typing
    assert_eq!(h.app().route(), Route::Capsule, "{}", h.text());
    h.ticks(40);
    h.type_str("hello");
    assert!(h.text().contains("hello"));
    let inst = h
        .app()
        .world
        .instances
        .iter()
        .find(|instance| instance.status == jackin_app::domain::instance::InstanceStatus::Running)
        .map(|instance| instance.id.clone())
        .expect("running instance");
    // 24 second session with a different account
    h.ctrl('b');
    h.key(KeyCode::Char('c'));
    assert!(h.text().contains("New tab"), "{}", h.text());
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Account for Claude Code"), "{}", h.text());
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    assert_eq!(h.app().world.daemons[&inst].tabs.len(), 2);
    assert!(h.text().contains("(Work)"), "{}", h.text());
    // 25–27 split, focus, resize, zoom
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
        junie_tui::KeyModifiers::ALT | junie_tui::KeyModifiers::SHIFT,
    );
    h.draw();
    h.ctrl('b');
    h.key(KeyCode::Char('z'));
    assert!(h.text().contains("zoom"), "{}", h.text());
    h.ctrl('b');
    h.key(KeyCode::Char('z'));
    // 28 scrollback, selection by mouse, copy, live again
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
    // a second press within the double-click window selects the word
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
    // 29 palette
    h.ctrl('\\');
    assert!(
        h.text().contains("palette") || h.text().contains("Palette"),
        "{}",
        h.text()
    );
    h.key(KeyCode::Esc);
    // 30–31 capsule Usage
    h.ctrl('b');
    h.key(KeyCode::Char('u'));
    assert!(h.text().contains("Usage"), "{}", h.text());
    h.key(KeyCode::Esc);
    // 32–33 detach, reconnect with retained tabs
    h.ctrl('b');
    h.key(KeyCode::Char('d'));
    assert_eq!(h.app().route(), Route::Manager);
    h.key(KeyCode::Enter);
    assert_eq!(h.app().route(), Route::Capsule);
    assert_eq!(h.app().world.daemons[&inst].tabs.len(), 2);
    // 34 a second instance of the same Workspace
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
    // 35–36 exit this one, stay inside
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
    // 37 reconnect the first (still running) instance and leave through the exit flow
    h.key(KeyCode::Home);
    h.key(KeyCode::Down);
    h.key(KeyCode::Right);
    for _ in 0..4 {
        if h.text().contains("instance · running") {
            break;
        }
        h.key(KeyCode::Down);
    }
    assert!(h.text().contains("instance · running"), "{}", h.text());
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
    // 38–40 outro with the elapsed caption, then the terminal is restored
    assert_eq!(h.app().route(), Route::Outro, "{}", h.text());
    h.ticks(5);
    if !h.text().contains("You were in the Construct for") {
        // full motion: skip the warp to reach the caption
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

/// §16.4's stable name for the complete keyboard-first journey.  The retained
/// product test above carries the detailed step inventory.
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
    // the first account row is the Anthropic default: switch it off here
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
    // and back on
    h.key(KeyCode::Char(' '));
    assert!(
        h.text().contains("active for this Workspace"),
        "{}",
        h.text()
    );
    // enable a second Codex account: two accounts of one provider coexist
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
            .filter(|e| e.provider == jackin_app::domain::agent::Provider::OpenAi)
            .collect();
        assert_eq!(codex.len(), 2);
    }
    assert!(h.text().contains("• 1 change"), "{}", h.text());
    // prefer it, then save
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
            .get(&jackin_app::domain::agent::Provider::OpenAi)
            .map(String::as_str),
        Some("acct-codex-experiments")
    );
    let r = h.app().world.account_for(
        jackin_app::domain::agent::Provider::OpenAi,
        Some(ws),
        None,
        None,
    );
    assert_eq!(r.account.as_deref(), Some("acct-codex-experiments"));
}

#[test]
fn manager_launch_picker_hides_agents_without_an_account() {
    let mut h = H::new(Scenario::FirstUse, Motion::Reduced, 0, 120, 40);
    h.ticks(3);
    h.key(KeyCode::Enter);
    assert_eq!(h.app().route(), Route::Manager);
    let mut a = jackin_app::domain::account::Account::registered(
        "acct-only",
        "Only",
        jackin_app::domain::agent::Provider::Anthropic,
        jackin_app::domain::account::CredentialSource::LocalFolder {
            path: "~/.claude".into(),
            detected: jackin_app::domain::account::DetectedKind::ClaudeOAuthProfile,
        },
    );
    a.default_for_provider = true;
    h.app_mut().world.accounts.insert(a);
    h.key(KeyCode::Enter);
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
    // add an override for a Role that has none through the searchable picker
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
    h.tab_to(junie_tui::Id::root("editor.cfg").sub("form").sub("save"));
    h.key(KeyCode::Enter);
    let t = h.text();
    assert!(t.contains("Role: svc-010"), "{t}");
    assert!(t.contains("SVC_FLAG"), "{t}");
    assert!(t.contains("2 configured · "), "{t}");
    // the Roles tab scrolls and filters instead of overflowing
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
    assert_eq!(c.agent, jackin_app::domain::agent::Agent::ClaudeCode);
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
        .find(|instance| instance.status == jackin_app::domain::instance::InstanceStatus::Running)
        .map(|instance| instance.id.clone())
        .expect("running instance");
    let i = h.app().world.instance(&inst).unwrap();
    assert!(i.accounts.len() >= 2, "{:?}", i.accounts);
}
