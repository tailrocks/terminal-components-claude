//! JA-069: the complete keyboard-first journey (steps 1-40).
//!
//! A verbatim replay of the pinned `complete_jackin_flow_keyboard_first`
//! transcript: intro to manager, five accounts across local folders,
//! 1Password references, and the masked fallback, validation and provider
//! defaults, workspace creation through the prelude and editor, launch to
//! the cockpit and capsule, typing, a second tab, split/focus/resize/zoom,
//! scrollback mouse selection and copy, palette, capsule usage, detach and
//! reconnect with retained tabs, a second instance, still-inside exit, and
//! the outro with its elapsed caption through to quit. One frame per
//! transcript group plus a milestone ledger.

use jackin_app::{Motion, Route, Scenario};
use junie_tui::{Id, KeyCode, KeyModifiers, MouseKind};

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA069_ID: &str = "JA-069";
/// JA-069 sizes.
pub const JA069_SIZES: [Viewport; 1] = [Viewport::new(120, 40)];

/// Milestones the replay must hit, in transcript order.
pub const JA069_MILESTONES: [&str; 34] = [
    "local-folder",
    "codex-primary",
    "grok-team",
    "secret-masked",
    "fingerprint",
    "default-set",
    "health",
    "registration",
    "quota",
    "focus-tree",
    "step-2",
    "step-5",
    "worktree",
    "default-role",
    "api-base",
    "enabled-here",
    "preferred",
    "create-workspace",
    "launch-choose",
    "docker-build",
    "hello",
    "new-tab",
    "account-for",
    "zoom",
    "clipboard-drag",
    "clipboard-dclick",
    "clipboard-y",
    "palette",
    "usage",
    "reattached-tabs2",
    "second-running2",
    "still-inside",
    "outro-caption",
    "quit",
];

/// The complete journey capture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja069Capture {
    /// Milestones hit, in transcript order.
    pub milestones: Vec<String>,
    /// Accounts after step 9.
    pub accounts_len: usize,
    /// Workspaces after step 17.
    pub workspaces_len: usize,
    /// Tabs after the step-24 new tab.
    pub tabs_after_new: usize,
    /// Panes after the step-25 split.
    pub panes_after_split: usize,
    /// Running count after the second instance attaches.
    pub running_after_second: usize,
    /// Running count after the still-inside exit.
    pub running_after_exit: usize,
    /// Quit was requested at the end of the outro.
    pub final_quit: bool,
    /// One frame per transcript group, in order.
    pub frames: Vec<ObservedFrame>,
}

fn note(milestones: &mut Vec<String>, name: &str, hit: bool) {
    if hit {
        milestones.push(name.to_owned());
    }
}

fn running_instance(session: &DirectSession) -> Option<String> {
    session
        .app()
        .world
        .instances
        .iter()
        .find(|instance| instance.status == jackin_app::domain::instance::InstanceStatus::Running)
        .map(|instance| instance.id.clone())
}

fn tab_count(session: &DirectSession, inst: &str) -> usize {
    session
        .app()
        .world
        .daemons
        .get(inst)
        .map_or(0, |daemon| daemon.tabs.len())
}

fn pane_count(session: &DirectSession, inst: &str) -> usize {
    session
        .app()
        .world
        .daemons
        .get(inst)
        .map_or(0, |daemon| daemon.panes.len())
}

fn exit_or_keep(session: &mut DirectSession) {
    session.ctrl('q');
    if session.observe("exit-check").text.contains("Unsaved work") {
        session.key(KeyCode::Down);
        session.key(KeyCode::Down);
        session.key(KeyCode::Enter);
    } else {
        session.key(KeyCode::Right);
        session.key(KeyCode::Enter);
    }
}

fn capture_size(viewport: Viewport) -> Ja069Capture {
    let mut milestones = Vec::new();
    let mut frames = Vec::new();
    let form_save = jackin_app::screens::accounts::FORM.sub("save");
    let cfg_save = Id::root("editor.cfg").sub("form").sub("save");
    let mut session = DirectSession::fresh(
        JA069_ID,
        Scenario::FirstUse,
        Motion::Reduced,
        0,
        viewport,
        CaptureColor::TrueColor,
    );

    // 1-3 intro to the manager with zero instances.
    session.ticks(3);
    session.key(KeyCode::Enter);
    frames.push(session.observe("s01-manager"));

    // 4 Account & Usage Center.
    session.key(KeyCode::Char('c'));
    frames.push(session.observe("s04-accounts"));

    // 5-6 two Claude Code local-folder accounts.
    for (name, folder) in [("Personal", "~/.claude"), ("Work", "~/.claude-work")] {
        session.key(KeyCode::Char('a'));
        session.key(KeyCode::Enter);
        session.type_str(name);
        for _ in 0..3 {
            session.key(KeyCode::Tab);
        }
        session.key(KeyCode::Down);
        note(
            &mut milestones,
            "local-folder",
            session.observe("probe").text.contains("Local agent folder") && name == "Personal",
        );
        session.key(KeyCode::Tab);
        session.key(KeyCode::Enter);
        session.type_str(folder);
        session.key(KeyCode::Tab);
        session.tab_to(form_save);
        session.key(KeyCode::Enter);
    }
    frames.push(session.observe("s06-claude"));

    // 7-8 Codex and Grok Build through 1Password references.
    for (name, provider_steps, item) in [("Primary", 1, "Codex Primary"), ("Team", 2, "Grok Team")]
    {
        session.key(KeyCode::Char('a'));
        session.key(KeyCode::Enter);
        session.type_str(name);
        session.key(KeyCode::Tab);
        session.key(KeyCode::Tab);
        for _ in 0..provider_steps {
            session.key(KeyCode::Down);
        }
        session.key(KeyCode::Tab);
        session.key(KeyCode::Tab);
        session.key(KeyCode::Enter);
        session.ticks(4);
        session.key(KeyCode::Enter);
        session.ticks(4);
        session.key(KeyCode::Enter);
        session.ticks(4);
        session.type_str(item);
        session.ticks(4);
        session.key(KeyCode::Enter);
        session.ticks(4);
        session.key(KeyCode::Enter);
        session.ticks(2);
        let milestone = if name == "Primary" {
            "codex-primary"
        } else {
            "grok-team"
        };
        note(
            &mut milestones,
            milestone,
            session.observe("probe").text.contains(item),
        );
        session.tab_to(form_save);
        session.key(KeyCode::Enter);
    }
    frames.push(session.observe("s08-op"));

    // 9 OpenCode through the masked plain-text fallback.
    session.key(KeyCode::Char('a'));
    session.key(KeyCode::Enter);
    session.type_str("Go");
    session.key(KeyCode::Tab);
    session.key(KeyCode::Tab);
    for _ in 0..3 {
        session.key(KeyCode::Down);
    }
    session.key(KeyCode::Tab);
    session.key(KeyCode::Down);
    session.key(KeyCode::Down);
    session.key(KeyCode::Tab);
    session.key(KeyCode::Enter);
    session.type_str("oc_valid_abcdefghijklmn1234");
    session.key(KeyCode::Tab);
    note(
        &mut milestones,
        "secret-masked",
        !session.observe("probe").text.contains("abcdefghijklmn"),
    );
    session.tab_to(form_save);
    session.key(KeyCode::Enter);
    let accounts_len = session.app().world.accounts.accounts.len();
    frames.push(session.observe("s09-opencode"));

    // 10 validate one, set a provider default.
    session.key(KeyCode::Char('v'));
    session.ticks(20);
    {
        let text = session.observe("probe").text;
        note(
            &mut milestones,
            "fingerprint",
            text.contains("fingerprint") && text.contains("matches"),
        );
    }
    session.key(KeyCode::Home);
    for _ in 0..3 {
        session.key(KeyCode::Down);
    }
    session.key(KeyCode::Char(' '));
    note(
        &mut milestones,
        "default-set",
        session.observe("probe").text.contains("Default set"),
    );
    frames.push(session.observe("s10-default"));

    // 11-12 overview, one provider, one account.
    session.key(KeyCode::Home);
    note(
        &mut milestones,
        "health",
        session.observe("probe").text.contains("Health"),
    );
    session.key(KeyCode::Down);
    note(
        &mut milestones,
        "registration",
        session.observe("probe").text.contains("Registration"),
    );
    session.key(KeyCode::Down);
    note(
        &mut milestones,
        "quota",
        session.observe("probe").text.contains("Quota"),
    );
    frames.push(session.observe("s12-overview"));

    // 13 back to the manager, focus on the tree.
    session.key(KeyCode::Esc);
    {
        let frame = session.observe("s13-manager");
        let tree = format!("{:?}", jackin_app::screens::manager::TREE);
        note(
            &mut milestones,
            "focus-tree",
            frame.focus.as_deref() == Some(tree.as_str()),
        );
        frames.push(frame);
    }

    // 14 create a workspace through the prelude.
    session.key(KeyCode::End);
    session.key(KeyCode::Enter);
    session.key(KeyCode::Char(' '));
    note(
        &mut milestones,
        "step-2",
        session.observe("probe").text.contains("step 2 of 5"),
    );
    session.key(KeyCode::Enter);
    session.key(KeyCode::Enter);
    note(
        &mut milestones,
        "step-5",
        session.observe("probe").text.contains("step 5 of 5"),
    );
    session.key(KeyCode::Enter);
    frames.push(session.observe("s14-editor"));

    // 15 configure every tab.
    session.key(KeyCode::Char(']'));
    session.key(KeyCode::Enter);
    session.key(KeyCode::Char('i'));
    note(
        &mut milestones,
        "worktree",
        session.observe("probe").text.contains("worktree"),
    );
    session.key(KeyCode::Esc);
    session.key(KeyCode::Char(']'));
    session.key(KeyCode::Enter);
    session.key(KeyCode::Enter);
    note(
        &mut milestones,
        "default-role",
        session.observe("probe").text.contains("Default role ★"),
    );
    session.key(KeyCode::Esc);
    session.key(KeyCode::Char(']'));
    session.key(KeyCode::Enter);
    session.key(KeyCode::Char('a'));
    session.key(KeyCode::Enter);
    session.type_str("API_BASE");
    session.key(KeyCode::Tab);
    session.key(KeyCode::Tab);
    session.key(KeyCode::Enter);
    session.type_str("https://api.internal");
    session.key(KeyCode::Tab);
    session.tab_to(cfg_save);
    session.key(KeyCode::Enter);
    note(
        &mut milestones,
        "api-base",
        session.observe("probe").text.contains("API_BASE"),
    );
    frames.push(session.observe("s15-configured"));
    session.key(KeyCode::Esc);
    session.key(KeyCode::Char(']'));
    session.key(KeyCode::Enter);

    // 16 activate and prefer the non-default Claude account.
    if let (Some((_, py)), Some((_, wy))) = (session.find("Personal"), session.find("Work"))
        && py > wy
    {
        session.key(KeyCode::Down);
    }
    session.key(KeyCode::Char(' '));
    note(
        &mut milestones,
        "enabled-here",
        session.observe("probe").text.contains("enabled here"),
    );
    session.key(KeyCode::Char('p'));
    note(
        &mut milestones,
        "preferred",
        session.observe("probe").text.contains("Preferred for"),
    );
    frames.push(session.observe("s16-accounts"));

    // 17 preview and save.
    session.ctrl('s');
    note(
        &mut milestones,
        "create-workspace",
        session.observe("probe").text.contains("Create workspace"),
    );
    session.key(KeyCode::Right);
    session.key(KeyCode::Enter);
    session.ticks(20);
    let workspaces_len = session.app().world.workspaces.len();
    frames.push(session.observe("s17-saved"));

    // 18-20 launch straight to the cockpit.
    session.key(KeyCode::Home);
    session.key(KeyCode::Down);
    session.key(KeyCode::Enter);
    note(
        &mut milestones,
        "launch-choose",
        session
            .observe("probe")
            .text
            .contains("Launch · choose Agent"),
    );
    session.key(KeyCode::Enter);
    session.ticks(40);
    frames.push(session.observe("s20-cockpit"));

    // 21 build log.
    session.key(KeyCode::Char('b'));
    note(
        &mut milestones,
        "docker-build",
        session.observe("probe").text.contains("Docker build"),
    );
    session.key(KeyCode::PageUp);
    session.key(KeyCode::End);
    session.key(KeyCode::Esc);
    for _ in 0..60 {
        session.ticks(10);
        if session.app().route() != Route::Cockpit {
            break;
        }
    }
    session.ticks(15);
    frames.push(session.observe("s21-log"));

    // 22-23 capsule, typing.
    session.ticks(40);
    session.type_str("hello");
    note(
        &mut milestones,
        "hello",
        session.observe("probe").text.contains("hello"),
    );
    let inst = running_instance(&session).unwrap_or_default();
    frames.push(session.observe("s23-capsule"));

    // 24 second session with a different account.
    session.ctrl('b');
    session.key(KeyCode::Char('c'));
    note(
        &mut milestones,
        "new-tab",
        session.observe("probe").text.contains("New tab"),
    );
    session.key(KeyCode::Enter);
    note(
        &mut milestones,
        "account-for",
        session
            .observe("probe")
            .text
            .contains("Account for Claude Code"),
    );
    session.key(KeyCode::Down);
    session.key(KeyCode::Enter);
    let tabs_after_new = tab_count(&session, &inst);
    frames.push(session.observe("s24-second-tab"));

    // 25-27 split, focus, resize, zoom.
    session.ctrl('b');
    session.key(KeyCode::Char('%'));
    session.key(KeyCode::Enter);
    if session.observe("probe").text.contains("Account for") {
        session.key(KeyCode::Enter);
    }
    let panes_after_split = pane_count(&session, &inst);
    session.ctrl('b');
    session.key(KeyCode::Char('h'));
    session.key_mod(KeyCode::Right, KeyModifiers::ALT | KeyModifiers::SHIFT);
    session.draw();
    session.ctrl('b');
    session.key(KeyCode::Char('z'));
    note(
        &mut milestones,
        "zoom",
        session.observe("probe").text.contains("zoom"),
    );
    session.ctrl('b');
    session.key(KeyCode::Char('z'));
    frames.push(session.observe("s27-zoom"));

    // 28 scrollback, selection by mouse, copy, live again.
    session.ticks(60);
    session.key(KeyCode::PageUp);
    session.key(KeyCode::End);
    if let Some((x, y)) = session.find("Refactor") {
        session.mouse(MouseKind::Down, x, y);
        session.mouse(MouseKind::Drag, x.saturating_add(8), y);
        session.mouse(MouseKind::Up, x.saturating_add(8), y);
        note(
            &mut milestones,
            "clipboard-drag",
            session.app().world.clipboard.as_deref() == Some("Refactor"),
        );
        session.app_mut().world.clipboard = None;
        session.mouse(MouseKind::Down, x.saturating_add(2), y);
        session.mouse(MouseKind::Up, x.saturating_add(2), y);
        note(
            &mut milestones,
            "clipboard-dclick",
            session.app().world.clipboard.as_deref() == Some("Refactor"),
        );
        session.app_mut().world.clipboard = None;
        session.key(KeyCode::Char('y'));
        note(
            &mut milestones,
            "clipboard-y",
            session.app().world.clipboard.as_deref() == Some("Refactor"),
        );
    }
    session.key(KeyCode::End);
    frames.push(session.observe("s28-copy"));

    // 29 palette.
    session.ctrl('\\');
    {
        let text = session.observe("probe").text;
        note(
            &mut milestones,
            "palette",
            text.contains("palette") || text.contains("Palette"),
        );
    }
    session.key(KeyCode::Esc);
    frames.push(session.observe("s29-palette"));

    // 30-31 capsule Usage.
    session.ctrl('b');
    session.key(KeyCode::Char('u'));
    note(
        &mut milestones,
        "usage",
        session.observe("probe").text.contains("Usage"),
    );
    session.key(KeyCode::Esc);
    frames.push(session.observe("s31-usage"));

    // 32-33 detach, reconnect with retained tabs.
    session.ctrl('b');
    session.key(KeyCode::Char('d'));
    session.key(KeyCode::Enter);
    note(
        &mut milestones,
        "reattached-tabs2",
        session.app().route() == Route::Capsule && tab_count(&session, &inst) == 2,
    );
    frames.push(session.observe("s33-reattached"));

    // 34 a second instance of the same Workspace.
    session.ctrl('b');
    session.key(KeyCode::Char('d'));
    session.key(KeyCode::Home);
    session.key(KeyCode::Down);
    session.key(KeyCode::Enter);
    session.key(KeyCode::Enter);
    for _ in 0..80 {
        session.ticks(10);
        if session.app().route() == Route::Capsule {
            break;
        }
    }
    session.ticks(15);
    let running_after_second = session.app().world.running_count();
    note(
        &mut milestones,
        "second-running2",
        running_after_second == 2,
    );
    frames.push(session.observe("s34-second"));

    // 35-36 exit this one, stay inside.
    exit_or_keep(&mut session);
    let running_after_exit = session.app().world.running_count();
    {
        let frame = session.observe("s36-still-inside");
        note(
            &mut milestones,
            "still-inside",
            frame.route == "manager"
                && frame.text.contains("Still inside the Construct")
                && running_after_exit == 1,
        );
        frames.push(frame);
    }

    // 37 reconnect the first (still running) instance.
    session.key(KeyCode::Home);
    session.key(KeyCode::Right);
    let running_id = running_instance(&session).unwrap_or_default();
    let running_key = jackin_app::screens::manager::ManagerRowKey::Instance(running_id);
    session.key(KeyCode::Home);
    let bound = session.app().world.instances.len() + session.app().world.workspaces.len();
    for _ in 0..=bound {
        if session.app().manager.selected_row() == &running_key {
            break;
        }
        session.key(KeyCode::Down);
    }
    session.key(KeyCode::Enter);
    frames.push(session.observe("s37-reconnected"));

    // 38-40 outro with the elapsed caption, then quit.
    exit_or_keep(&mut session);
    session.ticks(5);
    if !session
        .observe("probe")
        .text
        .contains("You were in the Construct for")
    {
        session.key(KeyCode::Enter);
        session.ticks(25);
    }
    note(
        &mut milestones,
        "outro-caption",
        session
            .observe("probe")
            .text
            .contains("You were in the Construct for"),
    );
    session.key(KeyCode::Enter);
    let end = session.observe("s40-quit");
    let final_quit = end.quit;
    note(&mut milestones, "quit", final_quit);
    frames.push(end);

    Ja069Capture {
        milestones,
        accounts_len,
        workspaces_len,
        tabs_after_new,
        panes_after_split,
        running_after_second,
        running_after_exit,
        final_quit,
        frames,
    }
}

/// Capture JA-069 at all listed sizes.
#[must_use]
pub fn ja069_complete_journey() -> Vec<Ja069Capture> {
    JA069_SIZES.iter().map(|size| capture_size(*size)).collect()
}
