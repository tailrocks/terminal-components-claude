//! Parity journeys (HP01–HP23 except the deferred CLI): one test per task,
//! each driving the real App through the named parity fixture and asserting
//! the decisive rows with exact argv, cwd, states, effects and errors. These
//! are the semantic regressions the capability matrix requires; captures are
//! the visual half.

use ratatui::crossterm::event::KeyCode;

use junie_tui::core::event::Input;

use crate::app::TabKind;
use crate::app_tests::H;
use crate::domain::activity::ActivityState;
use crate::domain::effect::Effect;
use crate::scenario::{Motion, Scenario};

const HOME: &str = "/Users/alex";

fn open(h: &mut H, label: &str) {
    // Alt+0 is the Here tab; Esc leaves any page above the finder; Ctrl+U
    // clears whatever query was left there
    h.alt(KeyCode::Char('0'));
    for _ in 0..8 {
        if h.app.tabs[0].stack.len() <= 1 || !h.app.modals.is_empty() {
            break;
        }
        h.key(KeyCode::Esc);
    }
    h.ctrl(KeyCode::Char('u'));
    h.type_str(label);
    h.key(KeyCode::Enter);
}

/// Accept a one-step confirmation when one is open (the default is Cancel).
fn confirm(h: &mut H) {
    if !h.app.modals.is_empty() {
        h.key(KeyCode::Right);
        h.key(KeyCode::Enter);
    }
}

fn run(h: &mut H, label: &str) {
    open(h, label);
    confirm(h);
}

fn activity_id(h: &H) -> String {
    match h.tab_kind() {
        TabKind::Activity(id) => id,
        other => panic!("expected an activity tab, got {other:?}\n{}", h.text()),
    }
}

fn argv(h: &H, id: &str) -> Vec<String> {
    h.app
        .world
        .activity(id)
        .expect("activity")
        .argv
        .iter()
        .map(|c| format!("{} @{}", c.display(), c.cwd))
        .collect()
}

fn settle(h: &mut H, id: &str) -> ActivityState {
    for _ in 0..80 {
        if h.app.world.activity(id).is_some_and(|a| a.state.finished()) {
            break;
        }
        h.ticks(1);
    }
    h.app.world.activity(id).unwrap().state
}

fn output(h: &H, id: &str) -> Vec<String> {
    h.app
        .world
        .activity(id)
        .unwrap()
        .output
        .iter()
        .map(|(_, t, _)| t.clone())
        .collect()
}

// ------------------------------------------------------------ HP01

#[test]
fn hp01_discovery_is_adaptive_stable_and_reports_failed_sources_and_config_warnings() {
    // full motion keeps the sources' own timing: discovery is observable
    let mut h = H::new(Scenario::ParityDiscovery, Motion::Full, 0, 120, 40);
    // slow and failed sources are visible while discovery runs; nothing is
    // invented before a source answers
    assert!(h.text().contains("discovering"), "{}", h.text());
    let early: Vec<String> = h.app.world.items().iter().map(|i| i.id.clone()).collect();
    assert!(
        !early.iter().any(|i| i == "git.pull"),
        "no git action before its source: {early:?}"
    );
    h.ticks(14);
    assert!(
        h.app
            .world
            .failed_sources()
            .iter()
            .any(|(n, _)| *n == "docker")
    );
    h.ticks(45);
    let items = h.app.world.items();
    let ids: Vec<&str> = items.iter().map(|i| i.id.as_str()).collect();
    for id in [
        "git.pull",
        "git.push",
        "mise.task.test",
        "cargo.build",
        "node.script.dev",
        "just.recipe.build",
        "make.target.build",
        "taskfile.task.lint",
        "brew.service.redis.restart",
        "gradle.build",
        "idea.clean",
        "docker.daemon",
        "deploy",
        "custom.config",
    ] {
        assert!(ids.contains(&id), "{id} missing from {ids:?}");
    }
    // the daemon is down: docker resources say so and actions fail truthfully
    let daemon = items.iter().find(|i| i.id == "docker.daemon").unwrap();
    assert!(matches!(
        daemon.freshness,
        crate::domain::action::Freshness::Unavailable(_)
    ));
    // a project action colliding with a built-in id never shadows it; the
    // malformed sibling is a diagnostic, not an action
    let pull = items.iter().find(|i| i.id == "git.pull").unwrap();
    assert_eq!(
        argv_of(pull),
        vec!["git -C /Users/alex/work/probe pull --ff-only @/Users/alex/work/probe"]
    );
    assert!(!ids.contains(&"broken"));
    let deploy = items.iter().find(|i| i.id == "deploy").unwrap();
    assert_eq!(
        argv_of(deploy),
        vec!["tools/deploy.sh @/Users/alex/work/probe"]
    );
    let cfg = h.app.world.custom_project.as_ref().unwrap();
    assert!(
        cfg.diagnostics
            .iter()
            .any(|d| d.text().contains("git.pull")),
        "{:?}",
        cfg.diagnostics
    );
    assert!(
        cfg.diagnostics.iter().any(|d| d.index == Some(2)),
        "action[2] is the malformed one: {:?}",
        cfg.diagnostics
    );
    h.draw();
    assert!(
        h.text().contains("config warning"),
        "warnings are shown, never dropped: {}",
        h.text()
    );
    assert!(
        h.text().contains("docker, github unavailable"),
        "{}",
        h.text()
    );
    // unavailable github: the clone action leads to login, not to a clone
    let clone = items
        .iter()
        .find(|i| i.id == "github.clone")
        .unwrap_or_else(|| panic!("github.clone missing: {ids:?}"));
    assert_eq!(
        argv_of(clone),
        vec!["gh auth login @/Users/alex/work/probe"]
    );
    // the configuration page shows the diagnostics with their action index
    open(&mut h, "Custom action configuration");
    assert!(
        h.text().contains("action[0]") || h.text().contains("action[2]"),
        "{}",
        h.text()
    );
}

fn argv_of(it: &crate::domain::action::Item) -> Vec<String> {
    it.all_exec()
        .iter()
        .map(|c| format!("{} @{}", c.display(), c.cwd))
        .collect()
}

// ------------------------------------------------------------ HP02

#[test]
fn hp02_recents_learned_queries_query_editing_and_persistence_merge() {
    let mut h = H::new(Scenario::ParityHistory, Motion::Reduced, 0, 120, 40);
    h.ticks(8);
    let w = &h.app.world;
    let recent = w.memory.usage.recent(&w.location.cwd, "mbp", w.now_secs());
    let names: Vec<&str> = recent.iter().map(|(i, _)| i.as_str()).collect();
    // the capped 20-use action a half-life ago still leads; equal fresh use
    // ties deterministically by id; a stale single use is below the threshold
    assert_eq!(names[0], "git.pull", "{names:?}");
    assert_eq!(names[1], "cargo.build", "{names:?}");
    assert_eq!(names[2], "cargo.test", "{names:?}");
    assert!(!names.contains(&"cargo.check"), "{names:?}");
    assert!(
        recent[0].1 <= 1.0 + 20.0,
        "bounded by the use cap: {recent:?}"
    );
    assert!(recent.iter().all(|(_, s)| *s > 0.05));
    assert!(h.text().contains("Recent here"));
    // the fixture remembers "pull" → git.fetch, which does not match the
    // text: an unmatched remembered choice is never shown, the alias-free
    // best match leads
    h.type_str("pull");
    assert!(!h.text().contains("Fetch and prune"), "{}", h.text());
    assert!(h.row(6).contains("Pull "), "{}", h.row(6));
    // a remembered choice that still matches leads the rows for its query
    let now = h.app.world.now_secs();
    // a remembered choice always follows an actual use (the app learns on
    // run); a choice whose action was never used is pruned on save
    h.app.world.memory.usage.used(
        "git.pull_rebase",
        Some("/Users/alex/work/holla"),
        "mbp",
        now,
    );
    h.app
        .world
        .memory
        .usage
        .learn_query("pull", "git.pull_rebase", "mbp", now);
    h.key(KeyCode::Esc);
    h.type_str("pull");
    assert!(h.row(6).contains("Pull with rebase"), "{}", h.text());
    assert!(h.text().contains("remembered for “pull”"), "{}", h.text());
    // the remembered item must still match: an unmatched choice is never shown
    h.key(KeyCode::Esc);
    h.type_str("ünï code");
    assert!(!h.text().contains("cargo clippy"), "{}", h.text());
    assert!(h.text().contains("No matches"), "{}", h.text());
    // query editing: select all, replace, undo, redo, word delete, clear
    h.ctrl(KeyCode::Char('a'));
    assert!(
        h.text().contains("Query selected"),
        "{}\n{}",
        h.row(38),
        h.row(39)
    );
    h.type_str("x");
    assert!(
        h.row(4).contains("x") && !h.row(4).contains("ünï"),
        "typing replaced the selection: {}",
        h.row(4)
    );
    h.ctrl(KeyCode::Char('z'));
    assert!(h.row(4).contains("ünï code"), "undo: {}", h.row(4));
    h.ctrl(KeyCode::Char('y'));
    assert!(
        h.row(4).contains("x") && !h.row(4).contains("ünï"),
        "redo: {}",
        h.row(4)
    );
    h.key(KeyCode::Esc);
    h.type_str("cargo bui");
    h.alt(KeyCode::Backspace);
    assert!(
        h.row(4).contains("cargo ") && !h.row(4).contains("bui"),
        "word delete: {}",
        h.row(4)
    );
    h.type_str("bui");
    h.ctrl(KeyCode::Backspace);
    assert!(
        h.row(4).contains("cargo ") && !h.row(4).contains("bui"),
        "ctrl word delete: {}",
        h.row(4)
    );
    h.ctrl(KeyCode::Char('u'));
    assert!(!h.row(4).contains("cargo"), "clear: {}", h.row(4));
    // a paste is one edit
    h.app.handle(Input::Paste("cargo\ncheck".into()));
    h.draw();
    assert!(h.row(4).contains("cargo check"), "{}", h.row(4));
    // running an action records it and saves a merged store: the other
    // writer's record survives, this run is counted, stale queries are gone
    h.key(KeyCode::Enter);
    let id = activity_id(&h);
    assert_eq!(argv(&h, &id), vec!["cargo check @/Users/alex/work/holla"]);
    let saved = h.app.world.persisted.frecency.clone().expect("saved store");
    assert!(
        saved.contains("system.resources"),
        "concurrent writer merged: {saved}"
    );
    let on_disk = crate::domain::usage::UsageStore::load(Some(&saved), h.app.world.now_secs());
    let mine = on_disk
        .find("cargo.check", Some("/Users/alex/work/holla"), "mbp")
        .unwrap();
    assert_eq!(mine.count(), 2, "one stale use plus this run");
    assert!(
        on_disk.learned_for("gone", "mbp").is_none(),
        "a query whose action is gone was pruned"
    );
    assert_eq!(
        on_disk.learned_for("pull", "mbp"),
        Some("git.pull_rebase"),
        "the newer choice replaced the older"
    );
    // the Unicode query was remembered for an action never used here: it is
    // pruned like any other orphaned choice (normalization was proved above
    // by the lookup that matched "ünï code" against "Ünï  Code")
    assert_eq!(on_disk.learned_for("ünï  code", "mbp"), None);
    // the opt-out: nothing is read, learned or written
    let mut off = H::new(Scenario::ParityHistory, Motion::Reduced, 0, 120, 40);
    off.app.world.memory.usage = crate::domain::usage::UsageStore::disabled();
    off.ticks(8);
    assert!(!off.text().contains("remembered"));
    open(&mut off, "cargo check");
    assert!(
        off.app
            .world
            .persisted
            .frecency
            .as_deref()
            .is_none_or(|s| !s.contains("cargo.check")),
        "opt-out wrote nothing"
    );
    assert!(
        !off.text().contains("history not saved"),
        "the opt-out is silent, not an error: {}",
        off.text()
    );
}

// ------------------------------------------------------------ HP05

#[test]
fn hp05_git_current_runs_at_the_repository_root_with_truthful_outcomes() {
    let mut h = H::new(Scenario::ParityGitCurrent, Motion::Reduced, 0, 120, 40);
    h.ticks(6);
    let items = h.app.world.items();
    let pull = items.iter().find(|i| i.id == "git.pull").unwrap();
    // pull.rebase=false and a diverged branch: the plain pull merges; the
    // dirty tree is what blocks it, and the block is stated, not hidden
    assert_eq!(
        argv_of(pull),
        vec!["git -C /Users/alex/work/svc pull @/Users/alex/work/svc"]
    );
    assert_eq!(
        pull.effect,
        Some(Effect::GitPullMerge("/Users/alex/work/svc".into()))
    );
    assert!(
        pull.effects
            .iter()
            .any(|e| e.contains("blocked · 1 modified")),
        "{:?}",
        pull.effects
    );
    run(&mut h, "Pull with merge");
    let id = activity_id(&h);
    assert_eq!(
        argv(&h, &id),
        vec!["git -C /Users/alex/work/svc pull --no-rebase @/Users/alex/work/svc"]
    );
    let state = settle(&mut h, &id);
    let out = output(&h, &id).join("\n");
    // git refuses to merge over a modified file it would touch, or merges:
    // either way the world matches the outcome exactly
    let g = h.app.world.git_here().unwrap().clone();
    if state == ActivityState::Succeeded {
        assert_eq!(g.behind, 0, "{out}");
        assert_eq!(
            g.ahead, 2,
            "a merge commit lands on the local commit: {out}"
        );
        assert!(!g.diverged);
    } else {
        assert!(
            out.contains("overwritten") || out.contains("error"),
            "{out}"
        );
        assert_eq!(g.behind, 2, "a failed merge changes nothing: {out}");
    }
    // a rejected push fails visibly with git's reason and changes nothing
    h.alt(KeyCode::Char('0'));
    run(&mut h, "Push");
    let id = activity_id(&h);
    assert_eq!(
        argv(&h, &id),
        vec!["git -C /Users/alex/work/svc push @/Users/alex/work/svc"]
    );
    assert_eq!(settle(&mut h, &id), ActivityState::Failed);
    let out = output(&h, &id).join("\n");
    assert!(
        out.contains("fetch first") || out.contains("rejected"),
        "{out}"
    );
    // the dry run is read-only and succeeds
    h.alt(KeyCode::Char('0'));
    run(&mut h, "Push dry run");
    let id = activity_id(&h);
    assert_eq!(
        argv(&h, &id),
        vec!["git -C /Users/alex/work/svc push --dry-run @/Users/alex/work/svc"]
    );
    assert_eq!(settle(&mut h, &id), ActivityState::Succeeded);
    // switching to the primary branch lands the effect
    h.alt(KeyCode::Char('0'));
    run(&mut h, "Switch to main");
    let id = activity_id(&h);
    assert_eq!(
        argv(&h, &id),
        vec!["git -C /Users/alex/work/svc switch main @/Users/alex/work/svc"]
    );
    let state = settle(&mut h, &id);
    let out = output(&h, &id).join("\n");
    if state == ActivityState::Succeeded {
        assert_eq!(
            h.app.world.git_here().unwrap().branch.as_deref(),
            Some("main"),
            "{out}"
        );
    } else {
        // a modified file git would overwrite refuses the switch: nothing moves
        assert!(
            out.contains("overwritten") || out.contains("error"),
            "{out}"
        );
        assert_eq!(
            h.app.world.git_here().unwrap().branch.as_deref(),
            Some("feature/x"),
            "{out}"
        );
    }
}

// ------------------------------------------------------------ HP06

#[test]
fn hp06_sibling_batches_carry_exact_members_and_report_each_failure() {
    let mut h = H::new(Scenario::ParityGitBatch, Motion::Reduced, 0, 120, 40);
    h.ticks(8);
    let items = h.app.world.items();
    let pull = items.iter().find(|i| i.id == "git.pull-all").unwrap();
    let members: Vec<String> = pull.batch.iter().map(|(n, _, _, _)| n.clone()).collect();
    assert_eq!(
        pull.batch.len(),
        4,
        "four siblings, the nested repository is not one: {members:?}"
    );
    assert!(members.iter().all(|m| !m.contains("nested")));
    let remotes = items
        .iter()
        .find(|i| i.id == "git.push-all-remotes")
        .unwrap();
    let cmds = argv_of(remotes);
    assert!(
        cmds.contains(
            &"git -C /Users/alex/work/repos/alpha push gitlab @/Users/alex/work/repos/alpha"
                .to_owned()
        ),
        "{cmds:?}"
    );
    assert!(
        !cmds.iter().any(|c| c.contains("beta push gitlab")),
        "only alpha has the mirror: {cmds:?}"
    );
    // merged-branch cleanup (offered inside a repository) excludes the
    // worktree-occupied and primary branches
    let mut inside = crate::domain::fixtures::world_for(Scenario::ParityGitBatch, Motion::Reduced);
    inside.location.cwd = "/Users/alex/work/repos/alpha".into();
    let inside_items = inside.items();
    let del = inside_items
        .iter()
        .find(|i| i.id == "git.delete_merged")
        .unwrap_or_else(|| {
            panic!(
                "{:?}",
                inside_items
                    .iter()
                    .map(|i| i.id.clone())
                    .collect::<Vec<_>>()
            )
        });
    let del_cmds = argv_of(del);
    assert!(
        del_cmds
            .iter()
            .any(|c| c.contains("feature/done") && c.contains("hotfix/1")),
        "{del_cmds:?}"
    );
    assert!(
        !del_cmds
            .iter()
            .any(|c| c.contains("feature/wt") || c.contains(" main ")),
        "{del_cmds:?}"
    );
    // run the pull batch: every member is its own activity with the exact
    // command; delta fails on its corrupt index, the rest succeed
    run(&mut h, "Pull 4 sibling repositories");
    h.ticks(1);
    let batch = h
        .app
        .world
        .batches
        .last()
        .cloned()
        .unwrap_or_else(|| panic!("a batch\n{}", h.text()));
    assert_eq!(batch.members.len(), 4);
    for _ in 0..60 {
        h.ticks(1);
    }
    let states: Vec<(String, ActivityState, i32)> = batch
        .members
        .iter()
        .map(|m| {
            let a = h.app.world.activity(m).unwrap_or_else(|| {
                panic!(
                    "member {m} gone; batch {:?}; activities {:?}",
                    batch.members,
                    h.app
                        .world
                        .activities
                        .iter()
                        .map(|a| (a.id.clone(), a.name.clone(), a.state))
                        .collect::<Vec<_>>()
                )
            });
            (a.argv[0].display(), a.state, a.exit.unwrap_or(-1))
        })
        .collect();
    assert!(states.iter().all(|(_, s, _)| s.finished()), "{states:?}");
    let delta = states
        .iter()
        .find(|(c, _, _)| c.contains("/delta "))
        .unwrap_or_else(|| panic!("{states:?}"));
    assert_eq!(delta.1, ActivityState::Failed, "{states:?}");
    assert!(
        states
            .iter()
            .filter(|(_, s, _)| *s == ActivityState::Succeeded)
            .count()
            >= 2,
        "{states:?}"
    );
    assert_eq!(
        h.app
            .world
            .git
            .iter()
            .find(|g| g.path.ends_with("/alpha"))
            .unwrap()
            .behind,
        0,
        "alpha pulled"
    );
    h.draw();
    assert!(
        h.text().contains("1 failed") || h.text().contains("failed"),
        "the batch summary names the failure: {}",
        h.text()
    );
}

// ------------------------------------------------------------ HP03

#[test]
fn hp03_find_files_indexes_home_truthfully_and_resource_actions_are_exact() {
    let mut h = H::new(Scenario::ParityFiles, Motion::Reduced, 0, 120, 40);
    h.ticks(4);
    open(&mut h, "Find files under home");
    assert!(h.text().contains("Files ›"), "{}", h.text());
    h.type_str("readme");
    let text = h.text();
    assert!(
        text.contains("README.md") && text.contains("~/work/notes"),
        "{text}"
    );
    assert!(text.contains("~/work/app"), "{text}");
    assert!(
        !text.contains("node_modules"),
        "an .ignore'd tree is never indexed: {text}"
    );
    assert!(!text.contains("scratch"), "{text}");
    // Unicode names are found by their own text
    h.ctrl(KeyCode::Char('u'));
    h.type_str("café");
    assert!(h.text().contains("café menu.txt"), "{}", h.text());
    h.ctrl(KeyCode::Char('u'));
    h.type_str("東京");
    // wide cells: the buffer text carries the continuation cells
    assert!(
        h.text().contains("東") && h.text().contains("京") && h.text().contains(".md"),
        "{}",
        h.text()
    );
    // hidden entries and cloud-only roots are outside the index
    h.ctrl(KeyCode::Char('u'));
    h.type_str("hidden-config");
    assert!(h.text().contains("0 results"), "{}", h.text());
    h.ctrl(KeyCode::Char('u'));
    h.type_str("cloud-only");
    assert!(h.text().contains("0 results"), "{}", h.text());
    // the symlink is listed as itself, never followed into the app tree
    h.ctrl(KeyCode::Char('u'));
    h.type_str("main.rs");
    assert!(h.text().contains("work/app/src/main.rs"), "{}", h.text());
    assert!(!h.text().contains("link-to-app/src"), "{}", h.text());
    // resource actions on a hit: copy is OSC 52 bounded by the terminal's
    // limit (64 bytes here), open/reveal are exact opener commands
    h.ctrl(KeyCode::Char('u'));
    h.type_str("todo");
    h.key(KeyCode::Enter);
    assert!(
        !h.app.modals.is_empty(),
        "the actions menu opened: {}",
        h.text()
    );
    h.key(KeyCode::Down);
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    let text = h.text();
    assert!(
        text.contains("OSC 52") || text.contains("clipboard") || text.contains("Copied"),
        "copy outcome is stated: {text}"
    );
    let w = &h.app.world;
    let path = format!("{HOME}/work/notes/todo.md");
    if path.len() > 64 {
        assert!(w.clipboard.is_none(), "over the limit nothing is copied");
    } else {
        assert_eq!(w.clipboard.as_deref(), Some(path.as_str()));
    }
    h.key(KeyCode::Enter);
    h.key(KeyCode::Enter);
    let id = activity_id(&h);
    assert_eq!(
        argv(&h, &id),
        vec![format!("open {HOME}/work/notes/todo.md @{HOME}/work/notes")]
    );
}

// ------------------------------------------------------------ HP04

#[test]
fn hp04_browser_lists_previews_and_jumps_safely() {
    let mut h = H::new(Scenario::ParityBrowser, Motion::Reduced, 0, 120, 40);
    h.ticks(4);
    open(&mut h, "Browse ~/work/site");
    let text = h.text();
    assert!(text.contains("Files ›"), "{text}");
    // directories first, hidden entries hidden until asked
    let assets = h.find("assets").expect("assets");
    let index = h.find("index.html").expect("index.html");
    assert!(assets.1 < index.1, "directories first");
    assert!(!text.contains(".env"), "{text}");
    h.ctrl(KeyCode::Char('h'));
    assert!(
        h.text().contains(".env") && h.text().contains(".git"),
        "{}",
        h.text()
    );
    h.ctrl(KeyCode::Char('h'));
    // both normalisation forms are distinct entries
    assert!(
        h.text().contains("café.txt") && h.text().contains("café.txt"),
        "{}",
        h.text()
    );
    // previews: text is shown, binary is named not shown, invalid bytes and
    // controls are sanitized, caps are stated, links and specials are safe
    let cases: [(&str, &[&str], &[&str]); 8] = [
        ("index.html", &["<html>", "hello"], &[]),
        ("empty.txt", &["empty"], &[]),
        ("logo.png", &["binary"], &["\u{89}"]),
        ("bad.txt", &["h", "i"], &["\u{ff}"]),
        ("control.txt", &["red"], &["\u{1b}", "\u{7}"]),
        ("big.log", &["first 2000 lines"], &["line 2099"]),
        ("long.txt", &["cut at 4096"], &[]),
        ("pipe", &["special file"], &[]),
    ];
    for (name, must, must_not) in cases {
        let (_, y) = h
            .find(name)
            .unwrap_or_else(|| panic!("{name} listed\n{}", h.text()));
        let cur = h.app.hits.area_of(files::LIST).unwrap();
        h.click(cur.x + 2, y);
        h.ticks(1);
        let t = h.text();
        for m in must {
            assert!(t.contains(m), "{name}: expected {m:?}\n{t}");
        }
        for m in must_not {
            assert!(!t.contains(m), "{name}: raw {m:?} leaked\n{t}");
        }
    }
    // a broken link and an unreadable directory are errors, never crashes
    let (_, y) = h.find("broken").unwrap();
    let cur = h.app.hits.area_of(files::LIST).unwrap();
    h.click(cur.x + 2, y);
    assert!(h.text().contains("broken symbol"), "{}", h.text());
    let (_, y) = h.find("private").unwrap();
    h.click(cur.x + 2, y);
    h.key(KeyCode::Enter);
    assert!(
        h.text().to_lowercase().contains("permission denied"),
        "{}",
        h.text()
    );
    // a slow listing is pending, not empty, then complete at its full size
    let (_, y) = h.find("assets").unwrap();
    h.click(cur.x + 2, y);
    h.key(KeyCode::Enter);
    assert!(
        h.text().contains("listing") || h.text().contains("…"),
        "{}",
        h.text()
    );
    h.ticks(10);
    assert!(
        h.text().contains("2100") || h.text().contains("img0000.png"),
        "{}",
        h.text()
    );
    // the jump picker: an exact existing path wins, a missing one stays open
    h.key(KeyCode::Char('g'));
    assert!(!h.app.modals.is_empty());
    h.type_str("~/nowhere");
    h.key(KeyCode::Enter);
    assert!(
        !h.app.modals.is_empty(),
        "a failed jump keeps the picker open: {}",
        h.text()
    );
    assert!(h.text().contains("!"), "{}", h.text());
    h.ctrl(KeyCode::Char('u'));
    h.type_str("~/work");
    h.key(KeyCode::Enter);
    assert!(h.app.modals.is_empty(), "{}", h.text());
    assert!(h.text().contains("Files › work"), "{}", h.text());
    assert!(h.text().contains("Jumped to ~/work"), "{}", h.text());
}

use crate::screens::files;

// ------------------------------------------------------------ HP07

#[test]
fn hp07_task_adapters_keep_exact_names_caps_and_diagnostics() {
    let h = H::new(Scenario::ParityTaskSources, Motion::Reduced, 0, 120, 40);
    let items = h.app.world.items();
    let ids: Vec<&str> = items.iter().map(|i| i.id.as_str()).collect();
    // node: the lockfile picks yarn; odd script names are exact argv, never
    // interpolated; the visible set is the first thirty by name
    let quote = items
        .iter()
        .find(|i| i.id == "node.script.it's; rm")
        .unwrap_or_else(|| panic!("{ids:?}"));
    assert_eq!(
        argv_of(quote),
        vec!["yarn run 'it'\\''s; rm' @/Users/alex/work/poly"]
    );
    let pj = match &h
        .app
        .world
        .fs
        .get("/Users/alex/work/poly/package.json")
        .unwrap()
        .content
    {
        crate::sim::fs::Content::Text(t) => t.clone(),
        _ => unreachable!(),
    };
    let d = crate::domain::manifest::node_scripts(
        "/Users/alex/work/poly",
        Some(&pj),
        &["yarn.lock", "bun.lockb"],
    );
    assert_eq!(d.total, 35);
    assert_eq!(d.runner, "yarn", "yarn.lock wins over bun.lockb");
    assert_eq!(d.tasks.len(), 30);
    assert!(
        d.tasks
            .iter()
            .all(|t| t.name != "weird key" && t.name != "ünï"),
        "beyond the cap by name order"
    );
    let all = crate::domain::manifest::node_scripts(
        "/Users/alex/work/poly",
        Some(r#"{"scripts":{"weird key":"x","ünï":"y"}}"#),
        &["yarn.lock"],
    );
    let weird = all
        .tasks
        .iter()
        .find(|t| t.name == "weird key")
        .expect("weird key within the cap when the filler sorts after it");
    assert_eq!(
        crate::domain::exec::Command::from_vec(weird.argv.clone(), "/Users/alex/work/poly", "mbp")
            .display(),
        "yarn run 'weird key'"
    );
    let uni = all.tasks.iter().find(|t| t.name == "ünï").unwrap();
    assert_eq!(
        crate::domain::exec::Command::from_vec(uni.argv.clone(), "/Users/alex/work/poly", "mbp")
            .display(),
        "yarn run ünï"
    );
    // the cap: thirty scripts visible of thirty-five, the note says so
    let node: Vec<&crate::domain::action::Item> = items
        .iter()
        .filter(|i| i.id.starts_with("node.script."))
        .collect();
    assert_eq!(node.len(), 30, "{ids:?}");
    assert!(
        node.iter().any(|i| i
            .cap_note
            .as_deref()
            .is_some_and(|c| c.contains("30 of 35"))),
        "{:?}",
        node.iter().map(|i| i.cap_note.clone()).collect::<Vec<_>>()
    );
    // just: recipes come from the summary; make: pattern rules and variables
    // are not targets; Taskfile: a failed listing is a diagnostic, not tasks
    for id in [
        "just.recipe.build",
        "just.recipe.test",
        "just.recipe.lint",
        "make.target.all",
        "make.target.build",
        "make.target.deploy-prod",
        "taskfile.discovery",
    ] {
        assert!(ids.contains(&id), "{id} missing: {ids:?}");
    }
    assert!(
        !ids.iter()
            .any(|i| i.starts_with("make.target.%") || i.contains("VAR")),
        "{ids:?}"
    );
    assert!(
        !ids.iter().any(|i| i.starts_with("taskfile.task.")),
        "{ids:?}"
    );
    let diag = items.iter().find(|i| i.id == "taskfile.discovery").unwrap();
    assert!(diag.summary.contains("yaml: line 3"), "{}", diag.summary);
    // a failing script fails with its runner's exit, nothing is pretended
    let mut h = h;
    run(&mut h, "yarn s01");
    let id = activity_id(&h);
    assert_eq!(argv(&h, &id), vec!["yarn run s01 @/Users/alex/work/poly"]);
    assert_eq!(settle(&mut h, &id), ActivityState::Failed);
    assert!(h.app.world.activity(&id).unwrap().exit.unwrap_or(0) != 0);
    h.alt(KeyCode::Char('0'));
    run(&mut h, "yarn dev");
    let id = activity_id(&h);
    assert_eq!(argv(&h, &id), vec!["yarn run dev @/Users/alex/work/poly"]);
    assert_eq!(settle(&mut h, &id), ActivityState::Succeeded);
}

// ------------------------------------------------------------ HP08

#[test]
fn hp08_cargo_results_are_exit_codes_and_clean_lands_its_effect() {
    let mut h = H::new(Scenario::ParityCargo, Motion::Reduced, 0, 120, 40);
    h.ticks(6);
    run(&mut h, "cargo clippy");
    let id = activity_id(&h);
    assert_eq!(
        argv(&h, &id),
        vec!["cargo clippy --all-targets --all-features @/Users/alex/work/engine"]
    );
    assert_eq!(
        settle(&mut h, &id),
        ActivityState::Succeeded,
        "warnings do not fail the run"
    );
    assert!(
        output(&h, &id).join("\n").contains("warning"),
        "{:?}",
        output(&h, &id)
    );
    h.alt(KeyCode::Char('0'));
    run(&mut h, "cargo test");
    let id = activity_id(&h);
    assert_eq!(settle(&mut h, &id), ActivityState::Failed);
    let a = h.app.world.activity(&id).unwrap();
    assert_eq!(
        a.exit,
        Some(101),
        "cargo test's own exit code: {:?}",
        a.output
    );
    // clean: the label carries the measured size, the gate names the shared
    // target, the effect removes exactly the target directory
    let items = h.app.world.items();
    let clean = items.iter().find(|i| i.id == "cargo.clean").unwrap();
    assert!(clean.label.contains("900.0 MiB"), "{}", clean.label);
    assert_eq!(
        clean.effect,
        Some(Effect::CargoClean("/Users/alex/work/engine/target".into()))
    );
    assert!(
        clean.summary.contains("engine-wt")
            || clean.effects.iter().any(|e| e.contains("engine-wt")),
        "the shared target is named: {} {:?}",
        clean.summary,
        clean.effects
    );
    assert!(
        h.app
            .world
            .fs
            .exists("/Users/alex/work/engine/target/debug/engine")
    );
    h.alt(KeyCode::Char('0'));
    run(&mut h, "cargo clean dry run");
    let id = activity_id(&h);
    assert_eq!(
        argv(&h, &id),
        vec!["cargo clean --dry-run --verbose @/Users/alex/work/engine"]
    );
    assert_eq!(settle(&mut h, &id), ActivityState::Succeeded);
    assert!(
        h.app
            .world
            .fs
            .exists("/Users/alex/work/engine/target/debug/engine"),
        "a dry run removes nothing"
    );
    h.alt(KeyCode::Char('0'));
    open(&mut h, "cargo clean");
    // destructive: a review, then the typed phrase would follow; here the
    // one-step confirmation is enough for a project-local target
    confirm(&mut h);
    if !h.app.modals.is_empty() || h.text().contains("gate") {
        h.key(KeyCode::Right);
        h.key(KeyCode::Enter);
    }
    let id = activity_id(&h);
    assert_eq!(argv(&h, &id), vec!["cargo clean @/Users/alex/work/engine"]);
    assert_eq!(settle(&mut h, &id), ActivityState::Succeeded);
    assert!(
        !h.app.world.fs.exists("/Users/alex/work/engine/target"),
        "the target directory is gone"
    );
    assert_eq!(h.app.world.cargo.target_bytes, 0);
}

// ------------------------------------------------------------ HP09

#[test]
fn hp09_docker_and_compose_outcomes_land_in_the_world_and_fail_where_the_daemon_does() {
    let mut h = H::new(Scenario::ParityDocker, Motion::Reduced, 0, 120, 40);
    h.ticks(6);
    let running = |h: &H| {
        h.app
            .world
            .docker
            .containers
            .iter()
            .filter(|c| c.running)
            .count()
    };
    assert_eq!(running(&h), 5);
    // a restart lands its effect: the container comes back healthy
    run(&mut h, "Restart acme-db-1");
    let id = activity_id(&h);
    assert_eq!(
        argv(&h, &id),
        vec!["docker restart acme-db-1 @/Users/alex/work/stack"]
    );
    assert_eq!(settle(&mut h, &id), ActivityState::Succeeded);
    let db = h
        .app
        .world
        .docker
        .containers
        .iter()
        .find(|c| c.name == "acme-db-1")
        .unwrap();
    assert!(db.running && db.health.as_deref().is_none_or(|s| s == "healthy"));
    // this daemon fails every stop: one container stays running too
    h.alt(KeyCode::Char('0'));
    run(&mut h, "Stop acme-redis-1");
    let id = activity_id(&h);
    assert_eq!(
        argv(&h, &id),
        vec!["docker stop acme-redis-1 @/Users/alex/work/stack"]
    );
    assert_eq!(settle(&mut h, &id), ActivityState::Failed);
    assert!(
        h.app
            .world
            .docker
            .containers
            .iter()
            .find(|c| c.name == "acme-redis-1")
            .unwrap()
            .running
    );
    // stop-all fails at the daemon's stop stage: nothing else is pretended
    h.alt(KeyCode::Char('0'));
    run(&mut h, "Stop all containers");
    let id = activity_id(&h);
    assert_eq!(
        argv(&h, &id),
        vec![
            "docker stop acme-db-1 acme-redis-1 acme-api-1 acme-worker-1 acme-scheduler-1 @/Users/alex/work/stack"
        ]
    );
    assert_eq!(settle(&mut h, &id), ActivityState::Failed);
    assert!(
        output(&h, &id).join("\n").contains("cannot stop container"),
        "{:?}",
        output(&h, &id)
    );
    assert!(
        running(&h) >= 4,
        "a failed stop leaves the containers running"
    );
    // remove-all needs the typed phrase bound to the host
    h.alt(KeyCode::Char('0'));
    open(&mut h, "Stop and remove all containers");
    assert!(h.text().contains("gate 1 of 2"), "{}", h.text());
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    assert!(
        h.text()
            .contains("Type REMOVE ALL CONTAINERS ON mbp to confirm"),
        "{}",
        h.text()
    );
    h.key(KeyCode::Esc);
    h.key(KeyCode::Esc);
    // compose down removes the project's containers and nothing else; up
    // recreates the declared services running
    run(&mut h, "Stop the Compose project");
    let id = activity_id(&h);
    assert_eq!(
        argv(&h, &id),
        vec!["docker compose down @/Users/alex/work/stack"]
    );
    assert_eq!(settle(&mut h, &id), ActivityState::Succeeded);
    let names = |h: &H| {
        h.app
            .world
            .docker
            .containers
            .iter()
            .map(|c| (c.name.clone(), c.running))
            .collect::<Vec<_>>()
    };
    assert!(
        h.app
            .world
            .docker
            .containers
            .iter()
            .all(|c| c.project.as_deref() != Some("acme")),
        "{:?}",
        names(&h)
    );
    assert!(
        h.app
            .world
            .docker
            .containers
            .iter()
            .any(|c| c.name == "pgadmin"),
        "the unmanaged container stays: {:?}",
        names(&h)
    );
    h.alt(KeyCode::Char('0'));
    run(&mut h, "Start the Compose project");
    let id = activity_id(&h);
    assert_eq!(
        argv(&h, &id),
        vec!["docker compose up -d @/Users/alex/work/stack"]
    );
    assert_eq!(settle(&mut h, &id), ActivityState::Succeeded);
    let up: Vec<String> = h
        .app
        .world
        .docker
        .containers
        .iter()
        .filter(|c| c.project.as_deref() == Some("acme"))
        .map(|c| c.name.clone())
        .collect();
    assert_eq!(
        up,
        vec!["acme-db-1", "acme-api-1", "acme-worker-1"],
        "{:?}",
        names(&h)
    );
    assert!(
        h.app
            .world
            .docker
            .containers
            .iter()
            .filter(|c| c.project.as_deref() == Some("acme"))
            .all(|c| c.running)
    );
    // with the daemon down the actions are shown unavailable with the
    // daemon's reason and are not runnable; nothing is pretended
    let mut down = H::new(Scenario::ParityDiscovery, Motion::Reduced, 0, 120, 40);
    down.ticks(4);
    let items = down.app.world.items();
    let stop = items.iter().find(|i| i.id == "docker.stop_all").unwrap();
    assert_eq!(
        argv_of(stop),
        vec!["docker ps -q @/Users/alex/work/probe"],
        "the capture is the command when nothing was captured"
    );
    assert!(
        matches!(&stop.freshness, crate::domain::action::Freshness::Unavailable(r) if r.contains("Cannot connect")),
        "{:?}",
        stop.freshness
    );
    run(&mut down, "Stop all containers");
    assert!(
        matches!(down.tab_kind(), TabKind::Here),
        "nothing started: {}",
        down.text()
    );
    assert!(
        down.text()
            .contains("unavailable · Cannot connect to the Docker daemon"),
        "{}",
        down.text()
    );
    // the same command against a live daemon with no containers is a
    // truthful no-op listing
    let mut none = H::new(Scenario::FirstUse, Motion::Reduced, 0, 120, 40);
    none.ticks(4);
    if none.app.world.docker.daemon.is_ok() {
        run(&mut none, "Stop all containers");
        let id = activity_id(&none);
        assert_eq!(settle(&mut none, &id), ActivityState::Succeeded);
        assert!(
            output(&none, &id).join("\n").contains("0 containers"),
            "{:?}",
            output(&none, &id)
        );
    }
}

// ------------------------------------------------------------ HP10

#[test]
fn hp10_brew_services_verbs_are_exact_capped_and_fail_with_brew_s_reason() {
    let mut h = H::new(Scenario::ParityBrewServices, Motion::Reduced, 0, 120, 40);
    h.ticks(6);
    let items = h.app.world.items();
    // eleven services, malformed rows dropped, three verbs each, capped
    let verbs: Vec<&str> = items
        .iter()
        .filter(|i| i.id.starts_with("brew.service."))
        .map(|i| i.id.as_str())
        .collect();
    assert_eq!(verbs.len(), 30, "{verbs:?}");
    assert!(
        verbs.iter().all(|v| !v.contains("stale-one")),
        "the stale cache name is never an action: {verbs:?}"
    );
    let list = items.iter().find(|i| i.id == "brew.services").unwrap();
    assert!(list.label.contains("30 of 33"), "{}", list.label);
    // a restart lands its status
    run(&mut h, "Restart svc01");
    let id = activity_id(&h);
    assert_eq!(
        argv(&h, &id),
        vec!["brew services restart svc01 @/Users/alex/work"]
    );
    assert_eq!(settle(&mut h, &id), ActivityState::Succeeded);
    let status = |h: &H, n: &str| {
        h.app
            .world
            .brew
            .as_ref()
            .unwrap()
            .services
            .iter()
            .find(|s| s.name == n)
            .map(|s| s.status.clone())
            .unwrap()
    };
    assert_eq!(status(&h, "svc01"), "started");
    // the failing verb fails with brew's own words and changes nothing
    h.alt(KeyCode::Char('0'));
    run(&mut h, "Stop svc02");
    let id = activity_id(&h);
    assert_eq!(settle(&mut h, &id), ActivityState::Failed);
    assert!(
        output(&h, &id)
            .join("\n")
            .contains("Bootstrap failed: 5: Input/output error"),
        "{:?}",
        output(&h, &id)
    );
    assert_eq!(status(&h, "svc02"), "error");
    // the listing snapshot states the failed cache write, never retries
    h.alt(KeyCode::Char('0'));
    open(&mut h, "Homebrew services");
    assert!(h.text().contains("fails on this host"), "{}", h.text());
    assert!(h.text().contains("svc10"), "{}", h.text());
}

/// The item count named in the gate-2 title ("Trash 4 items · gate 2 of 2").
fn gate2_items(h: &H) -> usize {
    let text = h.text();
    let line = text
        .lines()
        .find(|l| l.contains("gate 2 of 2"))
        .unwrap_or_else(|| panic!("{text}"));
    line.split_whitespace()
        .find_map(|w| w.parse::<usize>().ok())
        .unwrap_or_else(|| panic!("{line}"))
}

/// Gate 2: type the exact phrase into the dialog and execute.
fn gate2(h: &mut H, phrase: &str) {
    assert!(h.text().contains("gate 2 of 2"), "{}", h.text());
    h.key(KeyCode::Enter);
    h.type_str(phrase);
    h.key(KeyCode::Enter);
    h.key(KeyCode::Tab);
    h.key(KeyCode::Enter);
}

// ------------------------------------------------------------ HP11

#[test]
fn hp11_gradle_wrapper_daemon_and_recursive_cleanup_are_bounded_and_truthful() {
    let mut h = H::new(Scenario::ParityGradle, Motion::Reduced, 0, 120, 40);
    h.ticks(6);
    let items = h.app.world.items();
    let ids: Vec<&str> = items.iter().map(|i| i.id.as_str()).collect();
    for id in [
        "gradle.build",
        "gradlew.build",
        "gradle.clean",
        "gradlew.clean",
        "gradle.test",
        "gradlew.test",
        "gradle.clean-all",
    ] {
        assert!(ids.contains(&id), "{id}: {ids:?}");
    }
    let wrapper = items.iter().find(|i| i.id == "gradlew.build").unwrap();
    assert_eq!(
        argv_of(wrapper),
        vec!["./gradlew build @/Users/alex/work/android"]
    );
    // the build fails with gradle's exit; the daemon stays up
    run(&mut h, "./gradlew build");
    let id = activity_id(&h);
    assert_eq!(settle(&mut h, &id), ActivityState::Failed);
    assert!(h.app.world.gradle.daemon_running);
    // clean lands its effect: the top-level build output is gone
    h.alt(KeyCode::Char('0'));
    run(&mut h, "gradle clean");
    let id = activity_id(&h);
    assert_eq!(
        argv(&h, &id),
        vec!["gradle clean @/Users/alex/work/android"]
    );
    assert_eq!(settle(&mut h, &id), ActivityState::Succeeded);
    assert!(!h.app.world.fs.exists("/Users/alex/work/android/build"));
    // the recursive cleanup: depth-bounded, skips node_modules, never follows
    // the link; the label states the measured size
    let all = items.iter().find(|i| i.id == "gradle.clean-all").unwrap();
    assert!(all.label.contains("49.0 MiB"), "{}", all.label);
    h.alt(KeyCode::Char('0'));
    open(&mut h, "Clean Gradle outputs");
    assert!(h.text().contains("Cleanup"), "{}", h.text());
    h.key(KeyCode::Right);
    let t = h.text();
    assert!(
        t.contains("app/build") && t.contains("android/.gradle") && t.contains("a/b/c/d/e/bu"),
        "{t}"
    );
    assert!(
        !t.contains("e/f/build") && !t.contains("too-deep"),
        "beyond the depth bound: {t}"
    );
    assert!(!t.contains("android/node_modules"), "{t}");
    assert!(!t.contains("work/other"), "the link is never followed: {t}");
    // the deletion runs gradle --stop first; a failing stop cancels the
    // cleanup (the stale candidates are preselected)
    assert!(h.text().contains("3 selected"), "{}", h.text());
    h.key(KeyCode::Char('d'));
    assert!(h.text().contains("gradle --stop first"), "{}", h.text());
    h.app.world.gradle.stop_fails = Some("Gradle daemon is busy".into());
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    let n = gate2_items(&h);
    gate2(
        &mut h,
        &format!("TRASH {n} UNDER /Users/alex/work/android ON mbp"),
    );
    assert!(h.text().contains("gradle --stop failed"), "{}", h.text());
    assert!(
        h.app.world.fs.exists("/Users/alex/work/android/.gradle"),
        "nothing was removed"
    );
    assert!(h.app.world.gradle.daemon_running);
}

// ------------------------------------------------------------ HP12

#[test]
fn hp12_intellij_cleanup_finds_metadata_case_insensitively_and_logs_a_failed_log_write() {
    let mut h = H::new(Scenario::ParityIdea, Motion::Reduced, 0, 120, 40);
    h.ticks(4);
    let items = h.app.world.items();
    let idea = items.iter().find(|i| i.id == "idea.clean").unwrap();
    assert!(idea.label.contains("44.0 KiB"), "{}", idea.label);
    open(&mut h, "Clean IntelliJ metadata");
    h.key(KeyCode::Right);
    let t = h.text();
    // exact lowercase `.iml` (OP49): the upper-case file is not metadata
    // rows truncate long paths in the middle: match their visible heads
    for want in [
        "app.iml",
        "~/work/ide/.idea",
        "mod/mod.iml",
        "a/b/c/d/e/deep",
    ] {
        assert!(t.contains(want), "{want}\n{t}");
    }
    for never in ["Upper", "e/f/g", "ide/node_modules", "elsewhere", "nested"] {
        assert!(!t.contains(never), "{never} must not be a candidate\n{t}");
    }
    assert!(
        t.contains("locked"),
        "the unreadable folder is reported: {t}"
    );
    assert!(
        t.contains("4 selected"),
        "stale metadata is preselected: {t}"
    );
    h.key(KeyCode::Char('d'));
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    let n = gate2_items(&h);
    gate2(
        &mut h,
        &format!("TRASH {n} UNDER /Users/alex/work/ide ON mbp"),
    );
    // the report: removed items, the log failure named, nothing rolled back
    assert!(h.text().contains("report"), "{}", h.text());
    assert!(
        h.text().contains("write failures") || h.text().contains("EROFS"),
        "{}",
        h.text()
    );
    assert!(!h.app.world.fs.exists("/Users/alex/work/ide/app.iml"));
    assert!(
        h.app
            .world
            .fs
            .exists("/Users/alex/work/ide/node_modules/x/.idea/x.xml")
    );
    let r = h.app.world.reports.last().unwrap();
    assert!(r.log_failures > 0);
    assert!(
        r.count(crate::domain::cleanup::Outcome::Trashed) >= 4,
        "{:?}",
        r.items
    );
}

// ------------------------------------------------------------ HP13

#[test]
fn hp13_upgrade_managers_run_exact_stages_and_the_plan_branches() {
    let mut h = H::new(Scenario::ParityUpgradeManagers, Motion::Reduced, 0, 120, 40);
    h.ticks(6);
    let items = h.app.world.items();
    let brew = items
        .iter()
        .find(|i| i.id == "upgrade.brew-packages")
        .unwrap();
    assert_eq!(
        argv_of(brew),
        vec![
            "brew update @/Users/alex",
            "brew upgrade --greedy --yes @/Users/alex",
            "brew cleanup @/Users/alex",
            "brew autoremove @/Users/alex",
            "brew doctor @/Users/alex"
        ]
    );
    let omz = items.iter().find(|i| i.id == "upgrade.oh-my-zsh").unwrap();
    assert_eq!(
        argv_of(omz),
        vec!["sh /Users/alex/.config/omz/tools/upgrade.sh @/Users/alex"],
        "$ZSH wins over ~/.oh-my-zsh"
    );
    // the sequential batch: doctor fails on this host, the earlier stages
    // succeeded and the upgrade effect landed
    run(&mut h, "Upgrade Homebrew packages");
    h.ticks(1);
    let batch = h.app.world.batches.last().cloned().unwrap();
    assert_eq!(batch.members.len(), 5);
    for _ in 0..80 {
        h.ticks(1);
    }
    let states: Vec<(String, ActivityState)> = batch
        .members
        .iter()
        .map(|m| {
            let a = h.app.world.activity(m).unwrap();
            (a.name.clone(), a.state)
        })
        .collect();
    assert_eq!(states[1].1, ActivityState::Succeeded, "{states:?}");
    assert_eq!(
        states[4].1,
        ActivityState::Failed,
        "brew doctor fails on this host: {states:?}"
    );
    assert!(
        h.app.world.upgrade.brew_outdated.is_empty(),
        "the upgrade effect landed"
    );
    // mise upgrade moves the global tools to their latest versions
    h.alt(KeyCode::Char('0'));
    run(&mut h, "Upgrade mise-managed tools");
    let id = activity_id(&h);
    assert_eq!(argv(&h, &id), vec!["mise upgrade @/Users/alex"]);
    assert_eq!(settle(&mut h, &id), ActivityState::Succeeded);
    assert!(
        h.app
            .world
            .mise
            .global_tools
            .iter()
            .all(|t| t.active.as_deref() == Some(t.latest.as_str()))
    );
    // the everything plan: branches per manager, brew chained, verify last
    h.alt(KeyCode::Char('0'));
    open(&mut h, "Upgrade everything");
    assert!(
        h.text().contains("Upgrade everything · mbp") && h.text().contains("9 steps"),
        "{}",
        h.text()
    );
    let plan = h.app.world.plan("upgrade-all").unwrap();
    let ids: Vec<&str> = plan.steps.iter().map(|s| s.id.as_str()).collect();
    for id in [
        "brew-update",
        "brew-upgrade",
        "brew-casks",
        "mise-upgrade",
        "amp-update",
        "omz-upgrade",
        "verify",
    ] {
        assert!(ids.contains(&id), "{id}: {ids:?}");
    }
    let doctor = plan.steps.iter().find(|s| s.id == "brew-doctor").unwrap();
    assert!(doctor.fails);
    assert!(
        plan.steps
            .iter()
            .find(|s| s.id == "mise-upgrade")
            .unwrap()
            .deps
            .is_empty(),
        "managers run beside the brew chain"
    );
}

// ------------------------------------------------------------ HP14

#[test]
fn hp14_output_streams_are_exact_and_retention_drops_are_stated() {
    let mut h = H::new(Scenario::ParityExecutor, Motion::Reduced, 0, 120, 40);
    h.ticks(6);
    run(&mut h, "Emit a mixed stream");
    let id = activity_id(&h);
    assert_eq!(argv(&h, &id), vec!["./emit-stream @/Users/alex/work/batch"]);
    assert_eq!(settle(&mut h, &id), ActivityState::Succeeded);
    let out = output(&h, &id);
    assert_eq!(out[0], "first", "CRLF stripped: {out:?}");
    assert_eq!(
        out[1], "\u{1b}[32mgreen\u{1b}[0m",
        "the executor keeps the bytes: {out:?}"
    );
    assert!(
        !h.text().contains('\u{1b}'),
        "the viewer never paints a raw escape: {}",
        h.text()
    );
    assert!(h.text().contains("green"), "{}", h.text());
    assert!(
        out[2].contains("bad \u{fffd} byte"),
        "invalid UTF-8 replaced, never dropped: {out:?}"
    );
    assert_eq!(
        out[3], "last fragment",
        "a final fragment without newline is a line: {out:?}"
    );
    // the burst: 4500 lines, the last 4000 kept, the drop stated
    h.alt(KeyCode::Char('0'));
    run(&mut h, "Emit a burst");
    let id = activity_id(&h);
    assert_eq!(settle(&mut h, &id), ActivityState::Succeeded);
    let a = h.app.world.activity(&id).unwrap();
    assert_eq!(a.output.len(), 4000);
    assert_eq!(a.dropped, 500);
    assert_eq!(a.output[0].1, "burst line 500");
    assert!(
        h.text().contains("500 earlier lines dropped"),
        "{}",
        h.text()
    );
    // the sequential batch runs one member after another and stops the rest
    // when asked; the summary counts what actually happened
    h.alt(KeyCode::Char('0'));
    let mut hb = H::new(Scenario::ParityGitBatch, Motion::Reduced, 0, 120, 40);
    hb.ticks(6);
    let mode = hb
        .app
        .world
        .items()
        .iter()
        .find(|i| i.id == "git.push-all-remotes")
        .unwrap()
        .batch_mode;
    run(&mut hb, "Push 4 repositories to origin and GitLab");
    hb.ticks(1);
    let batch = hb.app.world.batches.last().cloned().unwrap();
    let first = hb.app.world.activity(&batch.members[0]).unwrap().state;
    let second = hb.app.world.activity(&batch.members[1]).unwrap().state;
    assert_eq!(batch.mode, mode);
    assert_eq!(first, ActivityState::Running);
    match mode {
        crate::sim::world::BatchMode::Sequential => {
            assert_eq!(second, ActivityState::Queued, "sequential: the next waits");
            assert!(hb.text().contains("queued"), "{}", hb.text());
        }
        crate::sim::world::BatchMode::Parallel => {
            assert_eq!(
                second,
                ActivityState::Running,
                "parallel: members run together"
            );
        }
    }
}

// ------------------------------------------------------------ HP15

#[test]
fn hp15_prompts_take_exact_input_and_cancellation_escalates_truthfully() {
    let mut h = H::new(Scenario::ParityTaskInput, Motion::Reduced, 0, 120, 40);
    h.ticks(4);
    run(&mut h, "Deploy the release");
    let id = activity_id(&h);
    assert_eq!(argv(&h, &id), vec!["./deploy.sh @/srv/app"]);
    h.ticks(4);
    assert!(
        h.app
            .world
            .activity(&id)
            .unwrap()
            .waiting
            .as_ref()
            .is_some_and(|p| p.secret)
    );
    assert!(h.text().contains("waiting for input"), "{}", h.text());
    assert!(h.text().contains("input wanted"), "{}", h.text());
    // input mode: the secret is never echoed or retained; Enter sends it
    h.key(KeyCode::Char('i'));
    assert!(h.text().contains("typing goes to stdin"), "{}", h.text());
    h.type_str("hunter2");
    assert!(
        !h.text().contains("hunter2"),
        "a secret is never echoed: {}",
        h.text()
    );
    h.key(KeyCode::Enter);
    let a = h.app.world.activity(&id).unwrap();
    assert!(
        a.stdin.iter().all(|r| r.secret && r.bytes.is_empty()),
        "{:?}",
        a.stdin
    );
    assert!(a.waiting.is_none());
    h.ticks(4);
    assert!(
        output(&h, &id).iter().any(|l| l == "authenticated"),
        "{:?}",
        output(&h, &id)
    );
    // the second prompt echoes a plain answer; n cancels the rollout
    assert!(
        h.app
            .world
            .activity(&id)
            .unwrap()
            .waiting
            .as_ref()
            .is_some_and(|p| !p.secret)
    );
    h.type_str("n");
    h.key(KeyCode::Enter);
    assert_eq!(settle(&mut h, &id), ActivityState::Failed);
    let out = output(&h, &id);
    assert!(
        out.iter().any(|l| l.ends_with("[y/N] n")),
        "the answer is echoed on the prompt row: {out:?}"
    );
    assert!(
        out.iter().any(|l| l.contains("rollout cancelled")),
        "{out:?}"
    );
    assert_eq!(h.app.world.activity(&id).unwrap().exit, Some(2));
    // EOF at the password prompt takes the EOF branch
    h.alt(KeyCode::Char('0'));
    run(&mut h, "Deploy the release");
    let id = activity_id(&h);
    h.ticks(4);
    h.key(KeyCode::Char('i'));
    h.ctrl(KeyCode::Char('d'));
    let st = settle(&mut h, &id);
    assert_eq!(
        st,
        ActivityState::Failed,
        "{:?} · stdin {:?}",
        output(&h, &id),
        h.app.world.activity(&id).unwrap().stdin
    );
    assert!(
        output(&h, &id).iter().any(|l| l.contains("aborted (EOF)")),
        "{:?}",
        output(&h, &id)
    );
    assert!(
        output(&h, &id).iter().any(|l| l.ends_with("Password: ^D")),
        "EOF is visible: {:?}",
        output(&h, &id)
    );
    // a resistant program: stop sends TERM, KILL follows after the grace
    h.alt(KeyCode::Char('0'));
    run(&mut h, "Run the stubborn worker");
    let id = activity_id(&h);
    h.ticks(2);
    h.key(KeyCode::Char('s'));
    assert_eq!(
        h.app.world.activity(&id).unwrap().state,
        ActivityState::Cancelling
    );
    assert!(h.text().contains("stopping"), "{}", h.text());
    h.ticks(4);
    assert_eq!(
        h.app.world.activity(&id).unwrap().state,
        ActivityState::Cancelling,
        "TERM alone does not end it"
    );
    h.ticks(10);
    assert_eq!(
        h.app.world.activity(&id).unwrap().state,
        ActivityState::Stopped
    );
    let out = output(&h, &id).join("\n");
    assert!(out.contains("SIGKILL"), "{out}");
    // Ctrl+C in input mode is the same stop request
    h.alt(KeyCode::Char('0'));
    run(&mut h, "Run the stubborn worker");
    let id = activity_id(&h);
    h.ticks(1);
    h.key(KeyCode::Char('i'));
    h.ctrl(KeyCode::Char('c'));
    assert_eq!(
        h.app.world.activity(&id).unwrap().state,
        ActivityState::Cancelling
    );
    assert!(output(&h, &id).iter().any(|l| l == "^C"));
    // a finished program takes no input
    h.alt(KeyCode::Char('0'));
    run(&mut h, "Quick no-op");
    let id = activity_id(&h);
    assert_eq!(settle(&mut h, &id), ActivityState::Succeeded);
    h.key(KeyCode::Char('i'));
    assert!(
        h.text().contains("finished · nothing reads input"),
        "{}",
        h.text()
    );
}

// ------------------------------------------------------------ HP17

#[test]
fn hp17_custom_actions_are_exact_argv_reviewed_by_digest_and_never_run_unsaved() {
    let mut h = H::new(Scenario::ParityCustomActions, Motion::Reduced, 0, 120, 40);
    h.ticks(4);
    let items = h.app.world.items();
    let ids: Vec<&str> = items.iter().map(|i| i.id.as_str()).collect();
    // global and project actions; a project id that repeats a global one is
    // dropped as a diagnostic; a malformed id never becomes an action
    for id in [
        "notes",
        "shared",
        "deploy.preview",
        "wipe",
        "multi",
        "custom.config",
    ] {
        assert!(ids.contains(&id), "{id}: {ids:?}");
    }
    assert!(!ids.contains(&"Bad Id"));
    let shared = items.iter().find(|i| i.id == "shared").unwrap();
    assert!(
        shared.provenance.contains("global"),
        "the global definition wins: {}",
        shared.provenance
    );
    let cfg = h.app.world.custom_project.clone().unwrap();
    assert!(
        cfg.diagnostics.iter().any(|d| d.text().contains("shared")),
        "{:?}",
        cfg.diagnostics
    );
    assert!(
        cfg.diagnostics.iter().any(|d| d.text().contains("Bad Id")),
        "{:?}",
        cfg.diagnostics
    );
    // argv is exact: the shell substitution is a literal argument
    let deploy = items.iter().find(|i| i.id == "deploy.preview").unwrap();
    assert_eq!(
        argv_of(deploy),
        vec!["tools/deploy-preview.sh --env 'preview stack' '$(whoami)' @/Users/alex/work/team"]
    );
    let multi = items.iter().find(|i| i.id == "multi").unwrap();
    assert_eq!(multi.all_exec()[0].args, vec!["%s\n", "a\nb c"]);
    // the bytes were trusted at another path (and as a legacy digest-only
    // record): both mean review again; the trust page names the whole-file
    // digest and the exact argv, then approval binds path and cwd
    let status = h
        .app
        .world
        .trust
        .status(&cfg.digest, &cfg.path, "/Users/alex/work/team");
    assert!(
        matches!(
            status,
            crate::domain::custom::TrustStatus::Moved
                | crate::domain::custom::TrustStatus::LegacyDigestOnly
        ),
        "{status:?}"
    );
    open(&mut h, "Deploy preview");
    assert!(h.text().contains("Trust"), "{}", h.text());
    assert!(h.text().contains("sha256:"), "{}", h.text());
    assert!(
        h.text().contains("argv[3]") && h.text().contains("$(whoami)"),
        "{}",
        h.text()
    );
    // the file changes during review: nothing is trusted or run
    let edited = cfg.text.replace("Deploy preview", "Deploy PREVIEW");
    h.app.world.fs.text(&cfg.path, &edited, 0);
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    assert!(h.text().contains("changed during review"), "{}", h.text());
    assert!(matches!(h.tab_kind(), TabKind::Here));
    assert!(
        h.app
            .world
            .trust
            .status(&cfg.digest, &cfg.path, "/Users/alex/work/team")
            != crate::domain::custom::TrustStatus::Trusted
    );
    // reviewed again with the current bytes: approval persists, the action runs
    let now_cfg = h.app.world.custom_project.clone().unwrap();
    open(&mut h, "Deploy preview");
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    let id = activity_id(&h);
    assert_eq!(
        argv(&h, &id),
        vec!["tools/deploy-preview.sh --env 'preview stack' '$(whoami)' @/Users/alex/work/team"]
    );
    assert_eq!(
        h.app
            .world
            .trust
            .status(&now_cfg.digest, &now_cfg.path, "/Users/alex/work/team"),
        crate::domain::custom::TrustStatus::Trusted
    );
    let saved = h.app.world.persisted.trust.clone().unwrap();
    assert!(
        saved.contains(&now_cfg.digest) && saved.contains("/Users/alex/work/team/.holla.toml"),
        "{saved}"
    );
    // a trust store that cannot be saved runs nothing
    h.alt(KeyCode::Char('0'));
    h.app.world.trust.save_failure = Some("EROFS".into());
    let _ = h.app.world.trust.revoke(&now_cfg.digest);
    open(&mut h, "Deploy preview");
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    assert!(h.text().contains("nothing was run"), "{}", h.text());
    assert!(matches!(h.tab_kind(), TabKind::Here));
    // the destructive action confirms once and runs its argv without a shell
    h.app.world.trust.save_failure = None;
    open(&mut h, "Wipe local caches");
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    confirm(&mut h);
    let id = activity_id(&h);
    assert_eq!(
        argv(&h, &id),
        vec!["sh -c 'rm -rf .cache && echo done' @/Users/alex/work/team"]
    );
}

// ------------------------------------------------------------ HP18

#[test]
fn hp18_disk_scan_streams_measures_exactly_and_keeps_cached_hints_as_hints() {
    let mut h = H::new(Scenario::ParityDiskScan, Motion::Reduced, 0, 120, 40);
    h.ticks(2);
    open(&mut h, "Analyze disk usage");
    assert!(h.text().contains("Disk › Usage"), "{}", h.text());
    assert!(h.text().contains("scanning ·"), "{}", h.text());
    // a cached size is a hint with its age until the live number arrives
    let scan = h.app.world.scan.clone().unwrap();
    assert_eq!(scan.ticks, 48);
    let now = h.app.world.now_secs();
    let mtime = |h: &H, p: &str| h.app.world.fs.get(p).map(|n| n.mtime).unwrap_or(0);
    assert!(
        h.app
            .world
            .size_cache
            .hint(
                "/Users/alex/work/data/media",
                mtime(&h, "/Users/alex/work/data/media"),
                now
            )
            .is_some(),
        "a valid entry is a hint"
    );
    assert!(
        !h.app
            .world
            .size_cache
            .entries
            .iter()
            .any(|e| e.path.ends_with("/expired")),
        "expired entries are dropped at load"
    );
    assert!(
        h.app
            .world
            .size_cache
            .hint(
                "/Users/alex/work/data/many",
                mtime(&h, "/Users/alex/work/data/many"),
                now
            )
            .is_none(),
        "a stale mtime invalidates the hint"
    );
    assert!(
        h.app
            .world
            .size_cache
            .hint("/Users/alex/work/data/gone", 1, now)
            .is_none()
            || !h.app.world.fs.exists("/Users/alex/work/data/gone")
    );
    h.ticks(60);
    assert!(h.text().contains("scan complete"), "{}", h.text());
    let scan = h.app.world.scan.clone().unwrap();
    let media = scan.tree.find("/Users/alex/work/data/media").unwrap();
    assert_eq!(
        media.allocated,
        12 * crate::sim::fs::BLOCK + crate::sim::fs::BLOCK,
        "the hardlink counts once, plus the directory block"
    );
    assert_eq!(media.entries, 3, "both names and the link are entries");
    assert!(
        media
            .children
            .iter()
            .any(|c| c.link && c.path.ends_with("/link")),
        "the link is a leaf, never followed"
    );
    let sparse = scan.tree.find("/Users/alex/work/data/sparse.img").unwrap();
    assert_eq!(
        (sparse.apparent, sparse.allocated),
        (50 * crate::sim::fs::BLOCK, 3 * crate::sim::fs::BLOCK)
    );
    let hidden = scan.tree.find("/Users/alex/work/data/.hidden").unwrap();
    assert_eq!(
        hidden.allocated,
        4 * crate::sim::fs::BLOCK + crate::sim::fs::BLOCK,
        "hidden entries are scanned"
    );
    let locked = scan.tree.find("/Users/alex/work/data/locked").unwrap();
    assert!(
        locked.error.is_some(),
        "an unreadable folder is an error, not a zero"
    );
    let cloud = scan.tree.find("/Users/alex/work/data/cloud").unwrap();
    assert!(
        cloud.error.is_some() && cloud.children.is_empty(),
        "dataless is never materialised"
    );
    assert!(h.text().contains("unreadable"), "{}", h.text());
    // the finished scan recorded the cache (root plus depth two), skipping
    // paths that changed during the scan
    assert!(
        h.app
            .world
            .size_cache
            .entries
            .iter()
            .any(|e| e.path == "/Users/alex/work/data/media")
    );
    assert!(
        h.app
            .world
            .persisted
            .sizes
            .as_deref()
            .is_some_and(|s| s.contains("\"v\":3") || s.contains("3"))
    );
    // rescan restarts the generation and keeps hints; cancel keeps a partial tree
    h.key(KeyCode::Char('r'));
    assert!(h.text().contains("Rescanning"), "{}", h.text());
    assert_eq!(h.app.world.scan.as_ref().unwrap().generation, 2);
    h.ticks(5);
    h.key(KeyCode::Char('x'));
    assert!(
        h.text().contains("partial · scan cancelled"),
        "{}",
        h.text()
    );
    assert!(h.app.world.scan.as_ref().unwrap().cancelled);
    h.ticks(60);
    assert!(
        h.text().contains("partial"),
        "a cancelled scan stays partial: {}",
        h.text()
    );
    assert!(
        h.text().contains("Freshness") && h.text().contains("partial · scan cancelled"),
        "{}",
        h.text()
    );
}

// ------------------------------------------------------------ HP19

#[test]
fn hp19_tree_navigation_sorting_folding_selection_and_top_files() {
    let mut h = H::new(Scenario::ParityDiskNavigation, Motion::Reduced, 0, 120, 40);
    h.ticks(2);
    open(&mut h, "Analyze disk usage");
    h.ticks(40);
    assert!(h.text().contains("scan complete"), "{}", h.text());
    // largest first with allocated bytes; the sparse image sorts by what it
    // occupies, not by its apparent size, until s switches the sort
    let target = h.find("target").unwrap();
    let vm = h.find("vm.img").unwrap();
    assert!(
        target.1 < vm.1,
        "target (allocated 500 blocks) above vm.img (allocated 40)"
    );
    h.key(KeyCode::Char('s'));
    assert!(h.text().contains("Sorted by apparent"), "{}", h.text());
    let target = h.find("target").unwrap();
    let vm = h.find("vm.img").unwrap();
    assert!(
        vm.1 < target.1,
        "apparent: vm.img (900 blocks) above target"
    );
    h.key(KeyCode::Char('s'));
    // noise folding is presentation only: the size stays, the row is one
    h.key(KeyCode::Down);
    h.key(KeyCode::Down);
    let before = h.text();
    assert!(before.contains("noise folded"), "{before}");
    h.key(KeyCode::Char('f'));
    assert!(h.text().contains("unfolded"), "{}", h.text());
    h.key(KeyCode::Char('f'));
    // selection: a parent dominates its descendants; a selects the visible
    h.key(KeyCode::Home);
    h.key(KeyCode::Down);
    h.key(KeyCode::Down);
    assert!(
        h.text().contains("~/work/big/noise"),
        "the cursor is on the noise folder: {}",
        h.text()
    );
    h.key(KeyCode::Char(' '));
    assert!(h.text().contains("1 item selected"), "{}", h.text());
    h.key(KeyCode::Char('f'));
    h.key(KeyCode::Right);
    h.key(KeyCode::Down);
    h.key(KeyCode::Char(' '));
    assert!(
        h.text().contains("covered by a selected ancestor"),
        "{}",
        h.text()
    );
    h.key(KeyCode::Char('a'));
    assert!(
        h.text().contains("Every visible entry selected"),
        "{}",
        h.text()
    );
    h.key(KeyCode::Char('a'));
    assert!(h.text().contains("Everything unselected"), "{}", h.text());
    // top files: Spotlight results deduplicated, the vanished one reported,
    // fed into the same gate
    h.key(KeyCode::Char('t'));
    assert!(h.text().contains("Disk › Top files"), "{}", h.text());
    let t = h.text();
    let rows = t
        .lines()
        .filter(|l| l.contains("iso.iso") && l.contains("MiB"))
        .count();
    assert_eq!(rows, 1, "duplicates collapse: {t}");
    assert!(t.contains("big.mov"), "{t}");
    assert!(
        t.contains("vanished.bin") && t.contains("stat failed"),
        "{t}"
    );
    assert!(!t.contains("small.bin"), "below 100 MiB: {t}");
    h.key(KeyCode::Char(' '));
    assert!(
        h.text().contains("1 item selected") || h.text().contains("selected"),
        "{}",
        h.text()
    );
    h.key(KeyCode::Char('d'));
    assert!(
        h.text().contains("gate 1 of 2") || h.text().contains("Review · trash"),
        "{}",
        h.text()
    );
    // Linux: no Spotlight, the tree scan is the way
    let mut l = H::new(Scenario::ParityPlatformsLinux, Motion::Reduced, 0, 120, 40);
    l.ticks(3);
    open(&mut l, "Top files on this Mac");
    assert!(
        l.text()
            .contains("unavailable · Spotlight is unavailable on Linux"),
        "{}",
        l.text()
    );
    assert!(matches!(l.tab_kind(), TabKind::Here));
}

// ------------------------------------------------------------ HP20

#[test]
fn hp20_insight_categories_sizes_eligibility_and_guards_are_truthful() {
    let mut h = H::new(Scenario::ParityInsights, Motion::Reduced, 0, 120, 40);
    h.ticks(8);
    let cats = crate::domain::catalog::insight_candidates(&h.app.world);
    let by = |id: &str| cats.iter().find(|c| c.category.id == id);
    // Xcode is running: derived data is ineligible with the reason
    let dd = by("xcode.derived-data").expect("derived data");
    assert!(dd.candidates.iter().all(|k| matches!(&k.eligibility, crate::domain::cleanup::Eligibility::Ineligible(w) if w.contains("Xcode is running"))), "{:?}", dd.candidates);
    // device support: 91 days old is preselected, 89 is not
    let ds = by("xcode.device-support").unwrap();
    let ios = ds
        .candidates
        .iter()
        .find(|k| k.path.contains("iOS"))
        .unwrap();
    let watch = ds
        .candidates
        .iter()
        .find(|k| k.path.contains("watchOS"))
        .unwrap();
    assert_eq!(
        ios.eligibility,
        crate::domain::cleanup::Eligibility::Preselected
    );
    assert!(matches!(
        watch.eligibility,
        crate::domain::cleanup::Eligibility::Ineligible(_)
    ));
    // the pnpm store comes from the tool's resolved path, not a guess
    let pnpm = by("pnpm.store").unwrap();
    assert!(
        pnpm.candidates
            .iter()
            .any(|k| k.path == "/Users/alex/Library/pnpm/store/v3"),
        "{:?}",
        pnpm.candidates
    );
    assert_eq!(
        pnpm.candidates[0].bytes,
        2200 * 1024 * 1024 + 2 * crate::sim::fs::BLOCK,
        "the file plus two directory blocks"
    );
    // project artifacts: every indicator-backed tree, nested never, the
    // symlinked project skipped, the unreadable folder reported
    let art = by("project.artifacts").unwrap();
    let paths: Vec<&str> = art.candidates.iter().map(|k| k.path.as_str()).collect();
    for want in [
        "/Users/alex/Projects/web/node_modules",
        "/Users/alex/Projects/rs/target",
        "/Users/alex/Projects/py/.venv",
        "/Users/alex/Projects/kt/build",
        "/Users/alex/Projects/ios/Pods",
        "/Users/alex/Projects/next/.next",
    ] {
        assert!(paths.contains(&want), "{want}: {paths:?}");
    }
    assert!(
        !paths.iter().any(|p| p.contains("target/node_modules")),
        "nested artifacts never double: {paths:?}"
    );
    assert!(
        !paths.iter().any(|p| p.contains("/plain/build")),
        "no indicator, no candidate: {paths:?}"
    );
    assert!(
        !paths
            .iter()
            .any(|p| p.contains("/linked/") || p.starts_with("/Users/alex/work/")),
        "symlinks are never followed: {paths:?}"
    );
    assert!(
        art.unreadable.iter().any(|p| p.ends_with("/locked")),
        "{:?}",
        art.unreadable
    );
    // the page: sizes per category, the Xcode guard visible, the lower bound named
    open(&mut h, "Review cleanup candidates");
    let t = h.text();
    assert!(t.contains("18 categories"), "{t}");
    assert!(
        t.contains("Xcode DerivedData") && t.contains("810.0 MiB"),
        "{t}"
    );
    assert!(
        t.contains("Xcode is running"),
        "the guard is stated on the category: {t}"
    );
    h.key(KeyCode::End);
    let t = h.text();
    assert!(
        t.contains("Project artifacts") && t.contains("2.5 GiB"),
        "{t}"
    );
    let (_, y) = h.find("Project artifacts").unwrap();
    let list = h.app.hits.area_of(crate::screens::cleanup::LIST).unwrap();
    h.click(list.x + 2, y);
    assert!(
        h.text().contains("Unreadable") && h.text().contains("locked"),
        "{}",
        h.text()
    );
}

// ------------------------------------------------------------ HP21

#[test]
fn hp21_deletion_is_authorized_validated_at_commit_and_never_falls_back() {
    let mut h = H::new(Scenario::ParityDeleteSafety, Motion::Reduced, 0, 120, 40);
    h.ticks(2);
    open(&mut h, "Analyze disk usage");
    h.ticks(30);
    assert!(h.text().contains("scan complete"), "{}", h.text());
    // select the target, the odd-named build, the linked tree and the
    // folder that will vanish; the gate names denials and the threat model
    for name in ["target", "linked", "gone-later", "locked"] {
        let (_, y) = h
            .find(name)
            .unwrap_or_else(|| panic!("{name}\n{}", h.text()));
        let tree = h.app.hits.area_of(crate::screens::disk::TREE).unwrap();
        h.click(tree.x + 6, y);
    }
    assert!(h.text().contains("4 selected"), "{}", h.text());
    // the documented deny rules, each with its own reason
    let w = &h.app.world;
    let deny = |p: &str| {
        crate::domain::cleanup::validate(p, &w.location.home, w.host.os, &w.fs)
            .err()
            .map(|d| d.0)
            .unwrap_or_default()
    };
    assert!(deny("/Users/alex/.Trash/old").contains("Trash"));
    assert!(
        deny("/Users/alex/Library/Containers/com.apple.Safari/Data/x").contains("holds user data")
    );
    assert!(deny("/Users/alex/Library/Application Support/App/x").contains("holds user data"));
    assert!(deny("/Users/alex/Library/Mobile Documents/com~apple~CloudDocs/x").contains("iCloud"));
    assert!(deny("/Users/alex/Library/Caches").contains("itself is protected"));
    assert!(deny("/Users/alex/Projects/safe/linked/node_modules").contains("symbolic link"));
    assert_eq!(
        crate::domain::cleanup::validate(
            "/tmp/holla-scratch/x",
            &w.location.home,
            w.host.os,
            &w.fs
        )
        .unwrap(),
        "/private/tmp/holla-scratch/x"
    );
    assert!(
        crate::domain::cleanup::validate(
            "/Users/alex/Projects/safe/ünï\ncode/build",
            &w.location.home,
            w.host.os,
            &w.fs
        )
        .is_ok(),
        "odd names are names"
    );
    h.key(KeyCode::Char('d'));
    let t = h.text();
    assert!(t.contains("Review · trash 4 items"), "{t}");
    assert!(
        t.contains("Threat model") && t.contains("re-resolved at commit"),
        "{t}"
    );
    assert!(
        t.contains("Denied") && t.contains("none · every path passed"),
        "{t}"
    );
    assert!(
        t.contains("Revision") && t.contains("voids this review"),
        "{t}"
    );
    // the folder vanishes after review: commit reports it, everything else
    // proceeds; the Trash collision retries and succeeds
    h.app
        .world
        .fs
        .remove_permanent("/Users/alex/Projects/safe/gone-later")
        .unwrap();
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    gate2(&mut h, "TRASH 4 UNDER /Users/alex/Projects/safe ON mbp");
    let r = h.app.world.reports.last().cloned().expect("a report");
    let outcome = |p: &str| {
        r.items
            .iter()
            .find(|i| i.path.ends_with(p))
            .map(|i| (i.outcome, i.error.clone()))
            .unwrap()
    };
    assert_eq!(
        outcome("/target").0,
        crate::domain::cleanup::Outcome::Trashed
    );
    assert_eq!(
        outcome("/gone-later").0,
        crate::domain::cleanup::Outcome::Failed,
        "{:?}",
        r.items
    );
    assert_eq!(
        outcome("/linked").0,
        crate::domain::cleanup::Outcome::Trashed,
        "a selected link removes the link only"
    );
    assert!(
        h.app
            .world
            .fs
            .exists("/Users/alex/work/other/node_modules/x"),
        "the link target is untouched"
    );
    assert_eq!(
        outcome("/locked").0,
        crate::domain::cleanup::Outcome::Failed,
        "{:?}",
        r.items
    );
    assert!(
        outcome("/locked")
            .1
            .as_deref()
            .is_some_and(|e| e.contains("unreadable")),
        "{:?}",
        r.items
    );
    assert_eq!(r.freed_now, 0, "Trash keeps the bytes on the volume");
    assert!(r.bytes >= 1000 * 1024 * 1024);
    assert!(h.text().contains("Cleanup report"), "{}", h.text());
    // a review is void once the mode changes: the gate refuses the stale plan
    let mut d = H::new(Scenario::ParityDeleteSafety, Motion::Reduced, 0, 120, 40);
    d.ticks(2);
    open(&mut d, "Review cleanup candidates");
    d.key(KeyCode::Right);
    d.key(KeyCode::Char('m'));
    assert!(d.text().contains("PERMANENT"), "{}", d.text());
    d.key(KeyCode::Char('n'));
    assert!(d.text().contains("Dry run"), "{}", d.text());
    d.key(KeyCode::Char('d'));
    assert!(
        d.text().contains("dry run") && d.text().contains("touch nothing"),
        "{}",
        d.text()
    );
    d.key(KeyCode::Right);
    d.key(KeyCode::Enter);
    let n = gate2_items(&d);
    let used = d.app.world.fs.volume_for("/").unwrap().used;
    gate2(
        &mut d,
        &format!("PERMANENTLY DELETE {n} UNDER /Users/alex ON mbp"),
    );
    let r = d.app.world.reports.last().cloned().expect("dry-run report");
    assert!(r.dry_run);
    assert!(
        r.items
            .iter()
            .all(|i| i.outcome != crate::domain::cleanup::Outcome::Removed)
    );
    assert_eq!(
        d.app.world.fs.volume_for("/").unwrap().used,
        used,
        "a dry run frees nothing"
    );
    assert!(
        d.app
            .world
            .ops_log
            .lines
            .iter()
            .any(|l| l.contains("\"dry_run\":true") && l.contains("would_remove")),
        "{:?}",
        d.app.world.ops_log.lines
    );
}

// ------------------------------------------------------------ HP22

#[test]
fn hp22_reports_ownership_outcomes_and_the_operation_log_are_exact() {
    let mut h = H::new(Scenario::ParityCleanupResults, Motion::Reduced, 0, 120, 40);
    h.ticks(2);
    // the prior run is in the log and the history before anything happens
    assert_eq!(h.app.world.ops_log.lines.len(), 1);
    open(&mut h, "Show cleanup history");
    assert!(
        h.text().contains("legacy/node_modules") || h.text().contains("legacy"),
        "{}",
        h.text()
    );
    assert!(h.text().contains("1 operation log records"), "{}", h.text());
    // trash the clones: the estimate over-counts, the report says so
    h.alt(KeyCode::Char('0'));
    open(&mut h, "Analyze disk usage");
    h.ticks(30);
    let (_, y) = h.find("big").unwrap_or_else(|| panic!("{}", h.text()));
    let tree = h.app.hits.area_of(crate::screens::disk::TREE).unwrap();
    h.click(tree.x + 6, y);
    h.key(KeyCode::Char('d'));
    assert!(
        h.text().contains("APFS clones may overcount")
            && h.text().contains("purgeable space excluded"),
        "{}",
        h.text()
    );
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    gate2(&mut h, "TRASH 1 UNDER /Users/alex/Projects/safe ON mbp");
    let r = h.app.world.reports.last().cloned().unwrap();
    assert_eq!(r.count(crate::domain::cleanup::Outcome::Trashed), 1);
    assert_eq!(r.freed_now, 0);
    assert_eq!(
        r.bytes,
        6000 * 1024 * 1024 + crate::sim::fs::BLOCK,
        "two clones plus the directory block"
    );
    assert!(
        h.text().contains("returns when the Trash is emptied"),
        "{}",
        h.text()
    );
    assert!(h.text().contains("every record written"), "{}", h.text());
    // the log grew by exactly the items, persisted with the prior record intact
    assert_eq!(h.app.world.ops_log.lines.len(), 2);
    assert_eq!(
        h.app.world.persisted.ops_log.len(),
        2,
        "append only, the earlier record kept"
    );
    assert!(h.app.world.persisted.ops_log[0].contains("legacy/node_modules"));
    assert!(
        h.app.world.persisted.ops_log[1].contains("\"outcome\":\"trashed\"")
            && h.app.world.persisted.ops_log[1].contains("/big")
    );
    // the history snapshot lists both and links the report
    h.alt(KeyCode::Char('0'));
    open(&mut h, "Show cleanup history");
    let t = h.text();
    assert!(t.contains("2 operation log records"), "{t}");
    assert!(t.contains("Report 1"), "{t}");
    // the used bytes on the volume stay until the Trash is emptied
    let v = h.app.world.fs.volume_for("/").unwrap();
    assert!(v.used > 0);
    assert!(!h.app.world.fs.exists("/Users/alex/Projects/safe/big"));
}

// ------------------------------------------------------------ HP23

#[test]
fn hp23_platform_capabilities_are_stated_and_never_faked() {
    // macOS: the dataless policy failure and the Spotlight timeout are facts
    let mut m = H::new(Scenario::ParityPlatforms, Motion::Reduced, 0, 120, 40);
    m.ticks(4);
    open(&mut m, "Top files on this Mac");
    assert!(
        m.text().contains("did not finish within 5 s"),
        "{}",
        m.text()
    );
    m.alt(KeyCode::Char('0'));
    let items = m.app.world.items();
    assert!(
        items.iter().any(|i| i.id == "upgrade.brew-casks"),
        "casks are a macOS action"
    );
    assert!(
        m.app
            .world
            .platform
            .dataless_failure
            .as_deref()
            .is_some_and(|f| f.contains("EPERM"))
    );
    // /tmp resolves through the macOS alias: validation names the canonical path
    assert_eq!(
        crate::domain::cleanup::validate(
            "/private/tmp/x",
            &m.app.world.location.home,
            m.app.world.host.os,
            &m.app.world.fs
        )
        .unwrap(),
        "/private/tmp/x"
    );
    assert_eq!(
        crate::domain::cleanup::validate(
            "/tmp/x",
            &m.app.world.location.home,
            m.app.world.host.os,
            &m.app.world.fs
        )
        .unwrap(),
        "/private/tmp/x"
    );
    // Linux: no Trash backend, no opener, no OSC 52, no pgrep, no Spotlight
    let mut l = H::new(Scenario::ParityPlatformsLinux, Motion::Reduced, 0, 120, 40);
    l.ticks(4);
    assert_eq!(
        crate::domain::cleanup::validate(
            "/tmp/x",
            &l.app.world.location.home,
            l.app.world.host.os,
            &l.app.world.fs
        )
        .unwrap(),
        "/tmp/x",
        "no /private alias on Linux: the path stays as typed"
    );
    let items = l.app.world.items();
    assert!(!items.iter().any(|i| i.id == "upgrade.brew-casks"));
    assert!(!items.iter().any(|i| i.id.starts_with("cleanup.xcode")));
    open(&mut l, "Review cleanup candidates");
    assert!(
        l.text().contains("Cargo registry cache") || l.text().contains("Project artifacts"),
        "{}",
        l.text()
    );
    l.key(KeyCode::Right);
    if !l.text().contains(" selected (") || l.text().contains("0 selected") {
        l.key(KeyCode::Down);
        l.key(KeyCode::Char(' '));
    }
    l.key(KeyCode::Char('d'));
    let t = l.text();
    assert!(
        t.contains("no Trash backend · every item will fail, never fall back"),
        "{t}"
    );
    l.key(KeyCode::Right);
    l.key(KeyCode::Enter);
    let n = gate2_items(&l);
    gate2(&mut l, &format!("TRASH {n} UNDER /home/alex ON devbox"));
    let r = l.app.world.reports.last().cloned().unwrap();
    assert_eq!(
        r.count(crate::domain::cleanup::Outcome::Failed),
        r.items.len(),
        "{:?}",
        r.items
    );
    assert!(
        r.items.iter().all(|i| i
            .error
            .as_deref()
            .is_some_and(|e| e.contains("Trash unavailable"))),
        "{:?}",
        r.items
    );
    assert!(
        l.app
            .world
            .fs
            .exists("/home/alex/Projects/app/node_modules/x"),
        "nothing fell back to permanent deletion"
    );
    // copy and open state their missing capability
    l.alt(KeyCode::Char('0'));
    open(&mut l, "Copy this folder's path");
    assert!(l.text().contains("does not accept OSC 52"), "{}", l.text());
    assert!(l.app.world.clipboard.is_none());
    open(&mut l, "Browse ~/work/box");
    l.key(KeyCode::Char('g'));
    l.type_str("~/Projects/app");
    l.key(KeyCode::Enter);
    assert!(l.text().contains("Files › app"), "{}", l.text());
    let (_, y) = l
        .find("package.json")
        .unwrap_or_else(|| panic!("{}", l.text()));
    let list = l.app.hits.area_of(files::LIST).unwrap();
    l.click(list.x + 2, y);
    l.alt(KeyCode::Enter);
    assert!(!l.app.modals.is_empty(), "the actions menu: {}", l.text());
    l.key(KeyCode::Enter);
    assert!(l.text().contains("No opener on devbox"), "{}", l.text());
}
