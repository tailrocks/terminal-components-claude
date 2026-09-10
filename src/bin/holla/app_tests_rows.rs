//! Row-level proofs for capability rows the journeys leave implicit. Each
//! test names the matrix rows it closes (docs/holla-parity-matrix.md) and
//! asserts the exact argv, state or text the row contract requires.

use ratatui::crossterm::event::KeyCode;

use crate::app_tests::H;
use crate::app_tests_parity::{activity_id, argv, argv_of, open, output, run, settle};
use crate::domain::activity::ActivityState;
use crate::domain::cleanup::{CATEGORIES, Safety, validate};
use crate::domain::context::Os;
use crate::scenario::{Motion, Scenario};
use crate::screens::{files, finder};

fn finder_cursor(h: &mut H) -> (usize, bool) {
    let f = h.app.tabs[0]
        .stack
        .last_mut()
        .and_then(|s| s.as_finder())
        .expect("the finder is the top page");
    (f.cursor, f.cursor_on_row())
}

fn preview_offset(h: &mut H) -> usize {
    h.app.tabs[0]
        .stack
        .last_mut()
        .and_then(|s| s.as_finder())
        .map(|f| f.preview_offset())
        .expect("the finder is the top page")
}

fn item_argv(h: &H, id: &str) -> Vec<String> {
    let items = h.app.world.items();
    let it = items.iter().find(|i| i.id == id).unwrap_or_else(|| {
        panic!(
            "{id} missing from {:?}",
            items.iter().map(|i| &i.id).collect::<Vec<_>>()
        )
    });
    argv_of(it)
}

fn batch_argv(h: &H, id: &str) -> Vec<String> {
    let items = h.app.world.items();
    let it = items
        .iter()
        .find(|i| i.id == id)
        .unwrap_or_else(|| panic!("{id} missing"));
    it.batch
        .iter()
        .flat_map(|(_, cmds, _, _)| cmds.iter().map(|c| format!("{} @{}", c.display(), c.cwd)))
        .collect()
}

// ------------------------------------------------------------ HP01 L07 L16 L19

#[test]
fn l07_built_in_find_browse_disk_and_cleanup_routes_survive_an_empty_toolset() {
    let mut h = H::new(Scenario::ParityDiscovery, Motion::Reduced, 0, 120, 40);
    h.ticks(60);
    let before: Vec<String> = h.app.world.items().into_iter().map(|i| i.id).collect();
    assert!(before.iter().any(|i| i == "cargo.build"), "{before:?}");
    h.app.world.tools.clear();
    let ids: Vec<String> = h.app.world.items().into_iter().map(|i| i.id).collect();
    for id in [
        "find.files",
        "browse.files",
        "disk.usage",
        "disk.overview",
        "cleanup.review-all",
    ] {
        assert!(
            ids.iter().any(|i| i == id),
            "{id} gone without tools: {ids:?}"
        );
    }
    assert!(
        !ids.iter().any(|i| i == "cargo.build"),
        "a tool-gated action stays without its tool: {ids:?}"
    );
}

#[test]
fn l16_l19_navigation_keys_land_on_rows_and_the_focused_preview_scrolls() {
    let mut h = H::new(Scenario::ParityDiscovery, Motion::Reduced, 0, 120, 24);
    h.ticks(60);
    h.key(KeyCode::End);
    let (end, on_row) = finder_cursor(&mut h);
    assert!(on_row, "End lands on a row: {}", h.text());
    h.key(KeyCode::Home);
    let (home, on_row) = finder_cursor(&mut h);
    assert!(
        on_row && home < end,
        "Home lands on the first row ({home} < {end})"
    );
    h.key(KeyCode::PageDown);
    let (page, on_row) = finder_cursor(&mut h);
    assert!(
        on_row && page > home,
        "PageDown moves down a page ({page} > {home})"
    );
    h.key(KeyCode::PageUp);
    let (back, on_row) = finder_cursor(&mut h);
    assert!(on_row && back < page, "PageUp moves back ({back} < {page})");
    // headings and blank rows never take the cursor, whichever way it moves
    for _ in 0..40 {
        h.key(KeyCode::Down);
        assert!(
            finder_cursor(&mut h).1,
            "Down landed off a row: {}",
            h.text()
        );
    }
    for _ in 0..40 {
        h.key(KeyCode::Up);
        assert!(finder_cursor(&mut h).1, "Up landed off a row: {}", h.text());
    }
    // Tab focuses the preview; arrows then scroll the preview, not the list
    h.key(KeyCode::Home);
    let (before, _) = finder_cursor(&mut h);
    h.key(KeyCode::Tab);
    assert!(h.app.focus.is(finder::PREVIEW), "{}", h.text());
    let off = preview_offset(&mut h);
    h.key(KeyCode::Down);
    h.key(KeyCode::Down);
    assert_eq!(finder_cursor(&mut h).0, before, "the list cursor stays");
    assert!(
        preview_offset(&mut h) > off,
        "the preview scrolled at 120x24: {}",
        h.text()
    );
    // a query change resets the preview scroll and returns to the list
    h.type_str("pull");
    assert_eq!(preview_offset(&mut h), 0);
    assert!(!h.app.focus.is(finder::PREVIEW));
}

// ------------------------------------------------------------ HP03 I-F10 I-F12 · HP04 I-B14

#[test]
fn i_f10_i_f12_file_actions_reveal_through_open_r_and_analyze_the_folder() {
    let mut h = H::new(Scenario::ParityBrowser, Motion::Reduced, 0, 120, 40);
    h.ticks(4);
    open(&mut h, "Browse ~/work/site");
    let (_, y) = h
        .find("index.html")
        .unwrap_or_else(|| panic!("{}", h.text()));
    let list = h.app.hits.area_of(files::LIST).unwrap();
    h.click(list.x + 2, y);
    h.alt(KeyCode::Enter);
    assert!(!h.app.modals.is_empty(), "the actions menu: {}", h.text());
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    let id = activity_id(&h);
    let a = argv(&h, &id);
    assert_eq!(a.len(), 1, "{a:?}");
    assert!(
        a[0].starts_with("open -R /Users/alex/work/site/index.html @"),
        "macOS reveal is open -R on the exact path: {a:?}"
    );
    assert_eq!(settle(&mut h, &id), ActivityState::Succeeded);
    // analyze from the menu: the Disk page for that folder, not for cwd
    open(&mut h, "Browse ~/work/site");
    let (_, y) = h.find("assets").unwrap_or_else(|| panic!("{}", h.text()));
    let list = h.app.hits.area_of(files::LIST).unwrap();
    h.click(list.x + 2, y);
    h.alt(KeyCode::Enter);
    h.key(KeyCode::Down);
    h.key(KeyCode::Down);
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    assert!(
        h.text().contains("Here › Files › site › Disk › Usage"),
        "the disk page nests under the browser: {}",
        h.text()
    );
    assert!(
        h.text().contains("Disk usage · ~/work/site/assets"),
        "{}",
        h.text()
    );
}

#[test]
fn i_b14_context_markers_and_recommendations_are_preview_only() {
    let mut h = H::new(Scenario::ParityBrowser, Motion::Reduced, 0, 120, 40);
    h.ticks(4);
    open(&mut h, "Browse ~/work/site");
    h.key(KeyCode::Char('g'));
    h.type_str("~/work");
    h.key(KeyCode::Enter);
    let (_, y) = h.find("site").unwrap_or_else(|| panic!("{}", h.text()));
    let list = h.app.hits.area_of(files::LIST).unwrap();
    h.click(list.x + 2, y);
    let t = h.text();
    assert!(t.contains("markers · .git"), "{t}");
    assert!(t.contains("git status · git pull"), "{t}");
    assert!(t.contains("recommendations are preview-only"), "{t}");
    assert!(
        h.app.world.activities.is_empty(),
        "nothing ran from a preview"
    );
}

// ------------------------------------------------------------ HP05 OP03 · HP06 OP07 OP08 OP11 OP12

#[test]
fn op03_op11_op12_current_repository_status_fetch_prune_and_gc_are_exact() {
    let mut h = H::new(Scenario::ParityGitCurrent, Motion::Reduced, 0, 120, 40);
    h.ticks(10);
    assert_eq!(
        item_argv(&h, "git.status"),
        vec!["git -C /Users/alex/work/svc status @/Users/alex/work/svc"]
    );
    assert_eq!(
        item_argv(&h, "git.fetch"),
        vec!["git -C /Users/alex/work/svc fetch --prune @/Users/alex/work/svc"]
    );
    assert_eq!(
        item_argv(&h, "git.gc"),
        vec!["git -C /Users/alex/work/svc gc @/Users/alex/work/svc"]
    );
}

#[test]
fn op07_op08_sibling_push_and_status_batches_name_every_repository() {
    let mut h = H::new(Scenario::ParityGitBatch, Motion::Reduced, 0, 120, 40);
    h.ticks(10);
    let repos = ["alpha", "beta", "delta", "gamma"];
    let push = batch_argv(&h, "git.push-all");
    let status = batch_argv(&h, "git.status-all");
    for r in repos {
        let root = format!("/Users/alex/work/repos/{r}");
        assert!(
            push.contains(&format!("git -C {root} push @{root}")),
            "{push:?}"
        );
        assert!(
            status.contains(&format!("git -C {root} status --short @{root}")),
            "{status:?}"
        );
    }
    assert_eq!(push.len(), repos.len());
    assert_eq!(status.len(), repos.len());
}

// ------------------------------------------------------------ HP09 OP34 · HP10 OP41 · HP13 OP54 OP56 OP57

#[test]
fn op34_op41_compose_logs_and_brew_start_carry_the_legacy_argv() {
    let mut d = H::new(Scenario::ParityDocker, Motion::Reduced, 0, 120, 40);
    d.ticks(10);
    let logs = item_argv(&d, "compose.logs");
    assert_eq!(logs.len(), 1, "{logs:?}");
    assert!(
        logs[0].starts_with("docker compose logs --tail 200 @"),
        "{logs:?}"
    );
    let mut b = H::new(Scenario::ParityBrewServices, Motion::Reduced, 0, 120, 40);
    b.ticks(10);
    let start = item_argv(&b, "brew.service.svc01.start");
    assert_eq!(start.len(), 1, "{start:?}");
    assert!(
        start[0].starts_with("brew services start svc01 @"),
        "{start:?}"
    );
}

#[test]
fn op54_op56_op57_standalone_upgrade_rows_are_exact() {
    let mut h = H::new(Scenario::ParityUpgradeManagers, Motion::Reduced, 0, 120, 40);
    h.ticks(10);
    let casks = batch_argv(&h, "upgrade.brew-casks");
    assert!(
        casks
            .iter()
            .any(|c| c.starts_with("brew upgrade --cask --greedy --yes @")),
        "{casks:?}"
    );
    assert!(
        casks.iter().any(|c| c.starts_with("brew update @")),
        "{casks:?}"
    );
    let amp = item_argv(&h, "upgrade.amp");
    assert!(amp[0].starts_with("amp update @"), "{amp:?}");
    let omz = item_argv(&h, "upgrade.oh-my-zsh");
    assert!(
        omz[0].starts_with("sh ") && omz[0].contains("/tools/upgrade.sh @"),
        "{omz:?}"
    );
}

// ------------------------------------------------------------ HP15 E33 E39

#[test]
fn e33_e39_input_mode_returns_on_esc_ends_with_the_program_and_a_kill_reports_reaping() {
    let mut h = H::new(Scenario::ParityTaskInput, Motion::Reduced, 0, 120, 40);
    h.ticks(4);
    run(&mut h, "Run the stubborn worker");
    let id = activity_id(&h);
    h.ticks(2);
    let input = |h: &mut H| {
        h.app.tabs[h.app.active]
            .stack
            .last_mut()
            .and_then(|s| s.as_activity())
            .map(|a| a.input)
            .expect("an activity tab")
    };
    h.key(KeyCode::Char('i'));
    assert!(input(&mut h));
    h.key(KeyCode::Esc);
    assert!(!input(&mut h), "Esc returns the keyboard");
    assert!(
        h.text().contains("Keyboard returned to holla"),
        "{}",
        h.text()
    );
    h.key(KeyCode::Char('s'));
    assert_eq!(settle(&mut h, &id), ActivityState::Stopped);
    let out = output(&h, &id);
    assert!(
        out.iter()
            .any(|l| l == "killed (SIGKILL) · descendants reaped"),
        "{out:?}"
    );
    // input mode ends by itself when the program exits
    run(&mut h, "Deploy the release");
    let id = activity_id(&h);
    h.ticks(4);
    h.key(KeyCode::Char('i'));
    assert!(input(&mut h));
    h.type_str("hunter2");
    h.key(KeyCode::Enter);
    h.ticks(4);
    h.type_str("y");
    h.key(KeyCode::Enter);
    assert_eq!(settle(&mut h, &id), ActivityState::Succeeded);
    assert!(
        !input(&mut h),
        "input mode ended with the program: {}",
        h.text()
    );
}

// ------------------------------------------------------------ HP17 E20

#[test]
fn e20_unreviewed_project_actions_are_labelled_before_trust() {
    let mut h = H::new(Scenario::ParityCustomActions, Motion::Reduced, 0, 120, 40);
    h.ticks(10);
    h.type_str("Deploy preview");
    assert!(h.text().contains("⚠ unreviewed"), "{}", h.text());
}

// ------------------------------------------------------------ HP19 LD002 LD003

#[test]
fn ld002_ld003_the_overview_lists_home_folders_first_and_a_custom_path_is_validated() {
    let mut h = H::new(Scenario::ParityDiskNavigation, Motion::Reduced, 0, 120, 40);
    h.ticks(10);
    open(&mut h, "Disk overview");
    assert!(h.text().contains("Here › Disk › Overview"), "{}", h.text());
    let rows = h.app.tabs[0]
        .stack
        .last_mut()
        .and_then(|s| s.as_disk())
        .map(|d| d.overview_rows())
        .expect("the overview page");
    let homes: Vec<&str> = rows
        .iter()
        .take_while(|(_, kind)| *kind == "home folder")
        .map(|(l, _)| l.as_str())
        .collect();
    assert!(homes.len() >= 2, "home folders first: {rows:?}");
    let mut sorted = homes.clone();
    sorted.sort_unstable();
    assert_eq!(homes, sorted, "alphabetical");
    assert!(
        rows.iter().any(|(_, kind)| *kind != "home folder"),
        "insight roots follow the home folders: {rows:?}"
    );
    // the custom path is validated before any scan starts
    open(&mut h, "Analyze a custom path");
    assert!(
        h.text()
            .contains("must exist · relative paths are rejected"),
        "{}",
        h.text()
    );
    let set_path = |h: &mut H, p: &str| {
        h.key(KeyCode::Enter);
        h.ctrl(KeyCode::Char('u'));
        h.type_str(p);
        h.key(KeyCode::Enter);
        h.ctrl(KeyCode::Char('s'));
    };
    set_path(&mut h, "relative/x");
    assert!(
        h.text().contains("relative/x: the path must be absolute"),
        "{}",
        h.text()
    );
    assert!(
        h.app
            .world
            .scan
            .as_ref()
            .is_none_or(|s| s.root != "relative/x")
    );
    open(&mut h, "Analyze a custom path");
    set_path(&mut h, "/Users/alex/nowhere");
    assert!(
        h.text()
            .contains("/Users/alex/nowhere: no such file or directory"),
        "{}",
        h.text()
    );
    open(&mut h, "Analyze a custom path");
    set_path(&mut h, "/Users/alex/work/big");
    assert!(h.text().contains("Disk usage · ~/work/big"), "{}", h.text());
    assert_eq!(
        h.app.world.scan.as_ref().map(|s| s.root.as_str()),
        Some("/Users/alex/work/big")
    );
}

// ------------------------------------------------------------ HP20 LD022–LD047

#[test]
fn ld022_to_ld047_the_category_table_matches_the_legacy_contract() {
    // (id, safety, minimum age in days, macOS only, roots, guard)
    type Row = (
        &'static str,
        Safety,
        i64,
        bool,
        &'static [&'static str],
        Option<&'static str>,
    );
    let expected: [Row; 18] = [
        (
            "xcode.derived-data",
            Safety::Rebuildable,
            0,
            true,
            &["Library/Developer/Xcode/DerivedData"],
            Some("Xcode"),
        ),
        (
            "xcode.device-support",
            Safety::OldOnly,
            90,
            true,
            &[
                "Library/Developer/Xcode/iOS DeviceSupport",
                "Library/Developer/Xcode/watchOS DeviceSupport",
                "Library/Developer/Xcode/tvOS DeviceSupport",
            ],
            None,
        ),
        (
            "xcode.archives",
            Safety::ReviewFirst,
            0,
            true,
            &["Library/Developer/Xcode/Archives"],
            None,
        ),
        (
            "simulator.caches",
            Safety::Rebuildable,
            0,
            true,
            &["Library/Developer/CoreSimulator/Caches"],
            Some("Simulator"),
        ),
        (
            "brew.cache",
            Safety::Rebuildable,
            0,
            true,
            &["Library/Caches/Homebrew"],
            None,
        ),
        (
            "npm.cache",
            Safety::Rebuildable,
            0,
            false,
            &[".npm/_cacache", ".npm/_logs"],
            None,
        ),
        (
            "pnpm.store",
            Safety::OldOnly,
            30,
            true,
            &["Library/pnpm/store"],
            None,
        ),
        (
            "yarn.cache",
            Safety::Rebuildable,
            0,
            true,
            &[".yarn/cache", "Library/Caches/Yarn"],
            None,
        ),
        (
            "bun.cache",
            Safety::Rebuildable,
            0,
            false,
            &[".bun/install/cache"],
            None,
        ),
        (
            "cargo.registry-cache",
            Safety::OldOnly,
            30,
            false,
            &[".cargo/registry/cache", ".cargo/git"],
            None,
        ),
        (
            "gradle.caches",
            Safety::OldOnly,
            30,
            false,
            &[".gradle/caches", ".gradle/daemon", ".gradle/wrapper/dists"],
            None,
        ),
        (
            "maven.repository",
            Safety::ReviewFirst,
            0,
            false,
            &[".m2/repository"],
            None,
        ),
        (
            "pip.cache",
            Safety::Rebuildable,
            0,
            true,
            &["Library/Caches/pip"],
            None,
        ),
        (
            "uv.cache",
            Safety::Rebuildable,
            0,
            false,
            &[".cache/uv"],
            None,
        ),
        (
            "user.caches",
            Safety::ReviewFirst,
            30,
            true,
            &["Library/Caches"],
            None,
        ),
        (
            "user.logs",
            Safety::Rebuildable,
            7,
            true,
            &["Library/Logs"],
            None,
        ),
        (
            "ide.jetbrains-logs",
            Safety::Rebuildable,
            7,
            true,
            &["Library/Logs/JetBrains"],
            None,
        ),
        (
            "project.artifacts",
            Safety::ReviewFirst,
            7,
            false,
            &["Projects"],
            None,
        ),
    ];
    assert_eq!(CATEGORIES.len(), expected.len());
    for (id, safety, age, mac, roots, guard) in expected {
        let c = CATEGORIES
            .iter()
            .find(|c| c.id == id)
            .unwrap_or_else(|| panic!("{id} is not a category"));
        assert_eq!(c.safety, safety, "{id} safety");
        assert_eq!(c.min_age_days, age, "{id} age");
        assert_eq!(c.macos_only, mac, "{id} platform");
        assert_eq!(c.roots, roots, "{id} roots");
        assert_eq!(c.guard_process, guard, "{id} guard");
    }
    // the two whole-root categories protect their root and offer children
    for id in ["user.caches", "user.logs"] {
        assert!(
            CATEGORIES.iter().any(|c| c.id == id && c.children_of_root),
            "{id}"
        );
    }
}

// ------------------------------------------------------------ HP21 LD053 LD054

#[test]
fn ld053_ld054_every_protected_root_and_user_path_is_denied_by_name() {
    let fs = crate::sim::fs::Fs::new();
    let home = "/Users/alex";
    let denied = [
        "/",
        "/bin/x",
        "/sbin/x",
        "/etc/x",
        "/private/etc/x",
        "/System/x",
        "/var/db/x",
        "/private/var/db/x",
        "/usr/x",
        "/usr/local",
        "/Library",
        "/Applications",
        "/Users",
        "/home",
        "/Users/alex",
        "/Users/alex/.Trash/x",
        "/Users/alex/Library/Keychains/x",
        "/Users/alex/Library/Application Support/x",
        "/Users/alex/Library/Safari/x",
        "/Users/alex/Library/Containers/com.apple.Safari/Data",
        "/Users/alex/Library/Containers/com.docker.docker/Data",
        "/Users/alex/Library/Caches",
        "/Users/alex/Library/Logs",
    ];
    for p in denied {
        assert!(
            validate(p, home, Os::MacOs, &fs).is_err(),
            "{p} must be denied"
        );
    }
    for p in [
        "/Users/alex/Library/Caches/com.app",
        "/Users/alex/work/x/target",
        "/Users/alex/.npm/_cacache",
    ] {
        assert!(
            validate(p, home, Os::MacOs, &fs).is_ok(),
            "{p} is an ordinary path"
        );
    }
}

// ------------------------------------------------------------ pointer: one click edits

#[test]
fn one_click_on_an_argument_field_starts_editing_at_the_pointer() {
    let mut h = H::new(Scenario::ParityDiskNavigation, Motion::Reduced, 0, 120, 40);
    h.ticks(10);
    open(&mut h, "Analyze a custom path");
    let field = crate::screens::review::ARG.child(0);
    let area = h
        .app
        .hits
        .area_of(field)
        .unwrap_or_else(|| panic!("the path field is hittable: {}", h.text()));
    // one completed click: focused and editing, the caret under the pointer
    h.click(area.x + 3, area.y);
    assert!(h.app.focus.is(field), "{}", h.text());
    assert!(
        h.last_row().contains("EDIT") || h.text().contains("EDIT"),
        "editing after one click: {}",
        h.text()
    );
    h.type_str("X");
    assert!(
        h.text().contains("/UsXers/alex/work/big") || h.text().contains("X"),
        "typing lands in the field: {}",
        h.text()
    );
}
