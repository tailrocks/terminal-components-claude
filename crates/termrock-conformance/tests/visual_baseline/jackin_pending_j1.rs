use std::path::{Path, PathBuf};
use std::time::Duration;

use tuiscotti::tui::CancelToken;

use crate::support::Session;
use crate::support::state_waits as waits;
use crate::support::{self, Case, Color, JACKIN};

const BOOT: &str = "jackin❯";
const BOOT_FAILURE: &str = "Launch failed";
const BOOT_SKIP: &str = "Enter Skip";

const RETURNING: &[&str] = &[
    "--scenario",
    "returning",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const ACCOUNTS: &[&str] = &[
    "--scenario",
    "accounts-mixed",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const LAUNCH: &[&str] = &[
    "--scenario",
    "launch-running",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const FAILURE: &[&str] = &[
    "--scenario",
    "launch-failure",
    "--motion",
    "paused",
    "--frame",
    "240",
];
const HARD: &[&str] = &[
    "--scenario",
    "hard-cases",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const FIRSTUSE40: &[&str] = &[
    "--scenario",
    "first-use",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const FIRSTUSE300: &[&str] = &[
    "--scenario",
    "first-use",
    "--motion",
    "paused",
    "--frame",
    "300",
];
const FIRST400: &[&str] = &[
    "--scenario",
    "first-use",
    "--motion",
    "paused",
    "--frame",
    "400",
];
const OUTRO40: &[&str] = &[
    "--scenario",
    "outro-last",
    "--motion",
    "paused",
    "--frame",
    "40",
];

const CRUMB_EDIT: &str = "Workspaces › payments-platform › edit";
const TREE_FOOTER: &str = "Tab Details";
const DRAWER_FOOTER: &str = "Tab Actions";
const PICKER_TITLE: &str = "Launch · choose Agent";
const PRELUDE_S1: &str = "step 1 of 5 · Source";

fn j1_case(slug: &str, args: &'static [&'static str], boot: &'static str) -> Case {
    Case::dynamic(
        format!("journeys/jackin/j1/{slug}"),
        JACKIN,
        args,
        120,
        40,
        Color::Truecolor,
        boot,
    )
}

fn j1_dir(test: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/tuiscotti/journeys")
        .join(test);
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("mkdir {}: {e}", dir.display()));
    dir
}

fn write_provenance(dir: &Path, case: &Case) {
    write_provenance_named(dir, case, "provenance.txt");
}

fn write_provenance_named(dir: &Path, case: &Case, file: &str) {
    let subject =
        support::try_resolve_bin(case.bin).unwrap_or_else(|e| panic!("resolve {}: {e}", case.bin));
    let modified = subject
        .mtime_unix
        .map_or_else(|| "unknown".to_string(), |m| m.to_string());
    let argv = support::argv_for(case);
    let body = format!(
        "profile: tuiscotti-default\ntransport: pty\nport_of: 3f167a6ea13527aa431a167d6bc7806ddd0de83e\nbinary: {}\nsha256: {}\nsize_bytes: {}\nmodified_unix: {modified}\nargv: {}\n",
        subject.path.display(),
        subject.sha256,
        subject.len,
        argv.join(" "),
    );
    std::fs::write(dir.join(file), &body)
        .unwrap_or_else(|e| panic!("write {}: {e}", dir.join(file).display()));
    eprintln!("j1 provenance ({file}): {body}");
}

fn checkpoint(s: &mut Session, dir: &Path, name: &str) {
    let obs = s
        .inner
        .observe_now()
        .unwrap_or_else(|e| panic!("checkpoint `{name}` sample failed: {e:#}"));
    let text = support::screen_text(&obs.screen);
    let body = format!(
        "# checkpoint {name} {}x{}\n{text}\n",
        obs.screen.cols(),
        obs.screen.rows()
    );
    let path = dir.join(format!("{name}.txt"));
    std::fs::write(&path, body).unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
    eprintln!("checkpoint {name} ({} bytes visible text)", text.len());
}

fn case_timeout(case: &Case) -> Duration {
    Duration::from_millis(case.timeout_ms)
}

fn live_text(s: &mut Session) -> String {
    let obs = s
        .inner
        .observe_now()
        .unwrap_or_else(|e| panic!("live sample failed: {e:#}"));
    support::screen_text(&obs.screen)
}

fn expect_exit_text(s: &Session, timeout: Duration, what: &str) -> String {
    let cancel = CancelToken::new();
    let exit = s
        .inner
        .expect_exit(std::time::Instant::now() + timeout, &cancel)
        .unwrap_or_else(|e| panic!("{what}: session never exited: {e:#}"));
    let obs = exit
        .success()
        .unwrap_or_else(|e| panic!("{what}: session did not exit cleanly: {e:#}"));
    support::screen_text(&obs.screen)
}

fn assert_line_has(text: &str, needle: &str, also: &str, what: &str) {
    assert!(
        text.lines().any(|l| l.contains(needle) && l.contains(also)),
        "{what}: no line contains both `{needle}` and `{also}`"
    );
}

fn assert_line_lacks(text: &str, needle: &str, also: &str, what: &str) {
    for line in text.lines().filter(|l| l.contains(needle)) {
        assert!(
            !line.contains(also),
            "{what}: `{needle}` line must not contain `{also}`: {line:?}"
        );
    }
}

#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_accounts_detail() {
    const CRUMB: &str = "Accounts › Claude › Work";
    let dir = j1_dir("j1_accounts_detail");
    let case = j1_case("accounts_detail", ACCOUNTS, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &["down", "down", "down", "down", &format!("wait:{CRUMB}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-detail");
    let detail = live_text(&mut s);

    assert!(detail.contains(CRUMB), "V1: crumb lacks `{CRUMB}`");
    assert_line_has(&detail, "Claude · Work", "exhausted", "V1 header");

    assert_line_has(&detail, "Provider", "Anthropic / Claude", "V2 provider");
    assert_line_has(&detail, "Quota", "stale · last good 47 min ago", "V2 quota");

    assert_line_has(&detail, "▎", "Work", "S1 cursor");

    assert!(detail.contains("Enter Details"), "S2: no Enter Details");
    assert!(detail.contains("m Usage"), "S2: no Usage hint");

    support::drive_with_timeout(
        &mut s,
        &["enter", &format!("wait:{DRAWER_FOOTER}")],
        case.timeout_ms,
    );
    let drawer = live_text(&mut s);
    assert!(
        drawer.contains("▎Claude · Work"),
        "A1: drawer title unfocused"
    );

    assert!(
        !drawer.contains("Enter Details"),
        "N1: tree footer visible in drawer"
    );
    checkpoint(&mut s, &dir, "02-drawer");

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, DRAWER_FOOTER, "drawer closed");
    waits::wait_state(&mut s, "Enter Details", "tree footer back");

    support::drive_with_timeout(&mut s, &["a", "wait:New account"], case.timeout_ms);
    checkpoint(&mut s, &dir, "03-add-presence");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "New account", "add form cancelled");
    let back = live_text(&mut s);
    assert!(
        !back.contains("New account"),
        "N2: add form stuck open after Esc"
    );
    assert_line_has(&back, "▎", "Work", "selection intact");
    checkpoint(&mut s, &dir, "04-tree");
    eprintln!("j1 accounts_detail: detail, drawer roundtrip, add-form roundtrip");
}

#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_accounts_drawer() {
    const CRUMB: &str = "Accounts › Claude › Work";
    let dir = j1_dir("j1_accounts_drawer");
    let case = j1_case("accounts_drawer", ACCOUNTS, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "down",
            "down",
            "down",
            "down",
            "enter",
            &format!("wait:{DRAWER_FOOTER}"),
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-drawer");
    let drawer = live_text(&mut s);

    assert_line_has(&drawer, "▎Claude · Work", "exhausted", "V1 title");

    assert!(
        drawer.contains("op://v_eng01/it_ant01/credential"),
        "V2: no op locator"
    );
    assert!(drawer.contains("••••••••…3c9e"), "V2: no redacted tail");

    assert!(drawer.contains(DRAWER_FOOTER), "S1: no Tab Actions");
    assert!(drawer.contains("Esc Back"), "S1: no Esc Back");

    assert_line_has(&drawer, "Refresh", "Remove…", "S2 actions");

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, DRAWER_FOOTER, "drawer closed");
    waits::wait_state(&mut s, "Enter Details", "tree footer back");
    checkpoint(&mut s, &dir, "02-tree");
    let tree = live_text(&mut s);

    assert!(!tree.contains(DRAWER_FOOTER), "N1: Tab Actions stuck");
    assert!(!tree.contains("▎Claude · Work"), "N2: drawer title stuck");

    support::drive_with_timeout(
        &mut s,
        &["enter", &format!("wait:{DRAWER_FOOTER}")],
        case.timeout_ms,
    );
    waits::wait_state(&mut s, "▎Claude · Work", "drawer title refocused");
    assert!(live_text(&mut s).contains(CRUMB), "A2: crumb lost");
    checkpoint(&mut s, &dir, "03-drawer-again");
    eprintln!("j1 accounts_drawer: drawer, tree roundtrip, drawer again");
}

#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_accounts_filter() {
    const TITLE: &str = "Accounts · filt";
    let dir = j1_dir("j1_accounts_filter");
    let case = j1_case("accounts_filter", ACCOUNTS, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);

    assert!(boot.contains("★ Personal"), "S2: boot lacks ★ Personal");

    support::drive_with_timeout(
        &mut s,
        &["/", "enter", "type:work", "enter", &format!("wait:{TITLE}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-filtered");
    let filtered = live_text(&mut s);

    assert!(filtered.contains(TITLE), "V1: no filtered title");
    assert!(filtered.contains("12 · 4 ▲ · 4 !"), "V1: no tree counts");

    assert!(filtered.contains("▾ Claude"), "V2: no Claude parent");
    assert_line_has(&filtered, "▎", "Overview", "V2 cursor");
    assert!(
        !filtered.contains("★ Personal"),
        "V2: Personal survives the work filter"
    );

    assert!(filtered.contains("Enter Details"), "S1: no Enter Details");
    assert!(filtered.contains("/ Filter"), "S1: no Filter hint");

    support::press_step(&s, "escape");
    waits::wait_state(&mut s, "★ Personal", "full tree restored");
    checkpoint(&mut s, &dir, "02-cleared");
    let cleared = live_text(&mut s);

    assert!(!cleared.contains(TITLE), "N1: filtered title stuck");

    support::drive_with_timeout(&mut s, &["a", "wait:New account"], case.timeout_ms);
    checkpoint(&mut s, &dir, "03-add-presence");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "New account", "add form cancelled");
    assert!(
        !live_text(&mut s).contains("New account"),
        "N2: add form stuck open"
    );
    checkpoint(&mut s, &dir, "04-tree");
    eprintln!("j1 accounts_filter: filtered, cleared, add-form roundtrip");
}

#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_audit_accounts_boot() {
    let dir = j1_dir("j1_audit_accounts_boot");
    let case = j1_case("audit_accounts_boot", ACCOUNTS, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);

    assert!(
        boot.contains("Accounts › Overview"),
        "V1: no overview crumb"
    );
    assert!(
        boot.contains("12 accounts · 8 providers"),
        "V1: no registry totals"
    );

    assert_line_has(&boot, "Health", "degraded · 4 warnings", "V2 health");
    assert!(boot.contains("1 exhausted"), "V2: no exhausted count");

    assert_line_has(&boot, "▎", "Overview", "S1 cursor");

    assert!(boot.contains("Enter Details"), "S2: no Enter Details");
    assert!(boot.contains("r Refresh all"), "S2: no Refresh all");

    support::drive_with_timeout(
        &mut s,
        &["enter", &format!("wait:{DRAWER_FOOTER}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-drawer-presence");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, DRAWER_FOOTER, "drawer closed");
    waits::wait_state(&mut s, "Enter Details", "tree footer back");
    assert!(
        !live_text(&mut s).contains(DRAWER_FOOTER),
        "N1: Tab Actions stuck"
    );

    support::drive_with_timeout(&mut s, &["a", "wait:New account"], case.timeout_ms);
    checkpoint(&mut s, &dir, "02-add-presence");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "New account", "add form cancelled");
    let back = live_text(&mut s);
    assert!(!back.contains("New account"), "N2: add form stuck open");
    assert_line_has(&back, "▎", "Overview", "cursor intact");
    checkpoint(&mut s, &dir, "03-boot");
    eprintln!("j1 audit_accounts_boot: boot regions, drawer+form roundtrips");
}

#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_editor_general() {
    let dir = j1_dir("j1_editor_general");
    let case = j1_case("editor_general", RETURNING, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &["e", &format!("wait:{CRUMB_EDIT}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-general");
    let general = live_text(&mut s);

    assert!(general.contains(CRUMB_EDIT), "V1: no editor crumb");
    assert!(general.contains("Name *"), "V1: no Name row");
    assert!(
        general.contains("payments-platform"),
        "V1: no workspace name"
    );

    assert!(general.contains("[✓] Keep awake"), "V2: no Keep awake");
    assert!(
        general.contains("[✓] Git pull before launch"),
        "V2: no Git pull"
    );

    assert!(general.contains("1–5 Jump"), "S1: no 1–5 Jump");
    assert!(general.contains("Enter Body"), "S1: no Enter Body");

    assert_line_has(&general, "Cancel", "Save…", "S2 buttons");

    support::drive_with_timeout(&mut s, &["2", "enter", "wait:+ Add mount"], case.timeout_ms);
    checkpoint(&mut s, &dir, "02-mounts");
    let mounts = live_text(&mut s);
    assert!(mounts.contains("r Read-only"), "A1: no mounts footer");

    assert!(!mounts.contains("Keep awake"), "N1: General body stuck");

    support::press_step(&s, "escape");
    waits::wait_state(&mut s, "1–5 Jump", "TABS refocused");
    assert!(
        live_text(&mut s).contains("+ Add mount"),
        "A2: mounts body lost on refocus"
    );
    checkpoint(&mut s, &dir, "03-tabs");
    support::press_step(&s, "escape");
    waits::wait_state(&mut s, "Current directory", "manager back");
    checkpoint(&mut s, &dir, "04-manager");
    let manager = live_text(&mut s);

    assert!(!manager.contains("› edit"), "N2: editor crumb stuck");
    assert!(manager.contains("Enter Launch"), "manager footer back");
    eprintln!("j1 editor_general: general, mounts jump, clean leave");
}

#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_editor_mounts() {
    let dir = j1_dir("j1_editor_mounts");
    let case = j1_case("editor_mounts", RETURNING, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "e",
            &format!("wait:{CRUMB_EDIT}"),
            "2",
            "enter",
            "wait:+ Add mount",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-mounts");
    let mounts = live_text(&mut s);

    assert_line_has(&mounts, "Destination", "Isolation", "V1 header");
    assert_line_has(
        &mounts,
        "/workspace/payments-platform",
        "worktree",
        "V1 first row",
    );

    assert!(mounts.contains("/workspace/libs"), "V2: no libs row");
    assert_line_has(&mounts, "/workspace/libs", "ro", "V2 libs mode");

    assert!(mounts.contains("▎›"), "S1: no body cursor");

    assert!(mounts.contains("r Read-only"), "S2: no Read-only hint");
    assert!(mounts.contains("i Isolation"), "S2: no Isolation hint");

    support::drive_with_timeout(&mut s, &["[", "wait:Keep awake"], case.timeout_ms);
    checkpoint(&mut s, &dir, "02-general");
    let general = live_text(&mut s);
    assert!(general.contains("1–5 Jump"), "A1: no TABS footer");

    assert!(!general.contains("+ Add mount"), "N1: mounts body stuck");
    assert!(!general.contains("r Read-only"), "N2: mounts footer stuck");

    support::drive_with_timeout(&mut s, &["]", "wait:+ Add mount"], case.timeout_ms);
    waits::wait_state(&mut s, "1–5 Jump", "TABS footer persists");
    assert!(
        live_text(&mut s).contains("/workspace/payments-platform"),
        "A2: mounts body lost"
    );
    checkpoint(&mut s, &dir, "03-mounts");
    eprintln!("j1 editor_mounts: mounts, general jump, mounts back");
}

#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_editor_mounts_dirty() {
    let dir = j1_dir("j1_editor_mounts_dirty");
    let case = j1_case("editor_mounts_dirty", RETURNING, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "e",
            &format!("wait:{CRUMB_EDIT}"),
            "2",
            "enter",
            "wait:+ Add mount",
            "r",
            "i",
            "wait:• 1 change",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-dirty");
    let dirty = live_text(&mut s);

    assert!(dirty.contains("• 1 change"), "V1: no change badge");
    assert!(dirty.contains("Mounts •"), "V1: no dotted tab");

    assert_line_has(&dirty, "/workspace/payments-platform", "clone", "V2 clone");
    assert_line_has(&dirty, "› •", "/workspace/payments-platform", "V2 marker");

    assert!(dirty.contains("isolation clone"), "S1: no isolation hint");

    assert_line_has(&dirty, "/workspace/libs", "shared", "S2 libs intact");

    support::press_step(&s, "escape");
    waits::wait_state(&mut s, "1–5 Jump", "TABS refocused");
    support::press_step(&s, "escape");
    waits::wait_state(&mut s, "Unsaved changes", "exit dialog opened");
    checkpoint(&mut s, &dir, "02-exit-dialog");
    let dialog = live_text(&mut s);
    assert!(
        dialog.contains("Save changes before leaving?"),
        "A1: no exit question"
    );
    for button in ["Cancel", "Discard", "Save"] {
        assert!(dialog.contains(button), "A1: no {button} button");
    }

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Unsaved changes", "exit dialog cancelled");
    checkpoint(&mut s, &dir, "03-dirty");
    let back = live_text(&mut s);
    assert!(!back.contains("Unsaved changes"), "N1: exit dialog stuck");
    assert!(!back.contains("Discard"), "N2: Discard action stuck");
    assert!(back.contains("• 1 change"), "edits lost on cancel");
    assert_line_has(&back, "/workspace/payments-platform", "clone", "row lost");
    eprintln!("j1 editor_mounts_dirty: dirty, exit dialog, cancelled intact");
}

#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_editor_env() {
    let dir = j1_dir("j1_editor_env");
    let case = j1_case("editor_env", RETURNING, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "e",
            &format!("wait:{CRUMB_EDIT}"),
            "4",
            "enter",
            "wait:DATABASE_URL",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-env");
    let env = live_text(&mut s);

    assert!(env.contains("4 vars"), "V1: no var count");
    assert_line_has(&env, "DATABASE_URL", "plain", "V1 first var");

    assert_line_has(&env, "GH_TOKEN", "host env", "V2 host env");
    assert!(
        env.contains("[op] Engineering › Stripe"),
        "V2: no 1Password row"
    );

    assert!(env.contains("Role overrides"), "S1: no overrides section");
    assert!(env.contains("▾ Role: backend"), "S1: no backend role");

    assert!(env.contains("p 1Password"), "S2: no 1Password hint");
    assert!(env.contains("s Scope"), "S2: no Scope hint");

    support::drive_with_timeout(&mut s, &["[", "wait:Allowed roles"], case.timeout_ms);
    checkpoint(&mut s, &dir, "02-roles");
    let roles = live_text(&mut s);
    assert!(roles.contains("1–5 Jump"), "A1: no TABS footer");

    assert!(!roles.contains("DATABASE_URL"), "N1: env body stuck");
    assert!(!roles.contains("p 1Password"), "N2: env footer stuck");

    support::drive_with_timeout(&mut s, &["]", "wait:DATABASE_URL"], case.timeout_ms);
    assert!(
        live_text(&mut s).contains("Role overrides"),
        "A2: env body lost"
    );
    checkpoint(&mut s, &dir, "03-env");
    eprintln!("j1 editor_env: env, roles jump, env back");
}

#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_editor_auth() {
    let dir = j1_dir("j1_editor_auth");
    let case = j1_case("editor_auth", RETURNING, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "e",
            &format!("wait:{CRUMB_EDIT}"),
            "5",
            "enter",
            "wait:Active accounts",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-auth");
    let auth = live_text(&mut s);

    assert!(auth.contains("Active accounts"), "V1: no Active accounts");
    assert!(
        auth.contains("5 effective · 4 inherited · 1 enabled here"),
        "V1: no effective counts"
    );

    assert_line_has(&auth, "[✓] Work", "enabled here", "V2 Work row");
    assert_line_has(
        &auth,
        "Archived contractor laptop",
        "disabled globally",
        "V2 disabled row",
    );

    assert!(
        auth.contains("registry in Accounts (c)"),
        "S1: no registry hint"
    );

    assert!(auth.contains("p Prefer"), "S2: no Prefer hint");
    assert!(auth.contains("/ Filter"), "S2: no Filter hint");

    support::drive_with_timeout(&mut s, &["[", "wait:DATABASE_URL"], case.timeout_ms);
    checkpoint(&mut s, &dir, "02-env");
    let env = live_text(&mut s);
    assert!(env.contains("1–5 Jump"), "A1: no TABS footer");

    assert!(!env.contains("Active accounts"), "N1: auth body stuck");
    assert!(!env.contains("p Prefer"), "N2: auth footer stuck");

    support::drive_with_timeout(&mut s, &["]", "wait:Active accounts"], case.timeout_ms);
    assert!(
        live_text(&mut s).contains("5 effective"),
        "A2: auth body lost"
    );
    checkpoint(&mut s, &dir, "03-auth");
    eprintln!("j1 editor_auth: auth, env jump, auth back");
}

#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_editor_roles() {
    let dir = j1_dir("j1_editor_roles");
    let case = j1_case("editor_roles", RETURNING, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "e",
            &format!("wait:{CRUMB_EDIT}"),
            "3",
            "enter",
            "wait:Allowed roles",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-roles");
    let roles = live_text(&mut s);

    assert!(roles.contains("Allowed roles"), "V1: no Allowed roles");
    assert!(roles.contains("3 of 46"), "V1: no allow counts");
    assert!(roles.contains("default ★ the-architect"), "V1: no default");

    assert_line_has(
        &roles,
        "[✓] the-architect ★",
        "Full-stack design",
        "V2 architect row",
    );
    assert_line_has(
        &roles,
        "[ ] data-eng",
        "! load error · trust required",
        "V2 blocked row",
    );

    assert!(roles.contains("▎›"), "S1: no body cursor");

    assert!(roles.contains("Space Allow"), "S2: no Allow hint");
    assert!(
        roles.contains("Enter Set default"),
        "S2: no Set default hint"
    );

    support::drive_with_timeout(&mut s, &["[", "wait:+ Add mount"], case.timeout_ms);
    checkpoint(&mut s, &dir, "02-mounts");
    let mounts = live_text(&mut s);
    assert!(mounts.contains("1–5 Jump"), "A1: no TABS footer");

    assert!(!mounts.contains("Allowed roles"), "N1: roles body stuck");
    assert!(
        !mounts.contains("Enter Set default"),
        "N2: roles footer stuck"
    );

    support::drive_with_timeout(&mut s, &["]", "wait:Allowed roles"], case.timeout_ms);
    assert!(live_text(&mut s).contains("3 of 46"), "A2: roles body lost");
    checkpoint(&mut s, &dir, "03-roles");
    eprintln!("j1 editor_roles: roles, mounts jump, roles back");
}

#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_editor_save_preview() {
    const DIALOG: &str = "Save workspace";
    const DIFF: &str = "~ keep_awake true → false";
    let dir = j1_dir("j1_editor_save_preview");
    let case = j1_case("editor_save_preview", RETURNING, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "e",
            &format!("wait:{CRUMB_EDIT}"),
            "down",
            "down",
            "down",
            "space",
            "wait:• 1 change",
        ],
        case.timeout_ms,
    );
    support::press_step(&s, "ctrl-s");
    waits::wait_state(&mut s, DIALOG, "save preview open");
    waits::wait_state(&mut s, DIFF, "preview diff line");
    checkpoint(&mut s, &dir, "01-preview");
    let preview = live_text(&mut s);

    assert!(preview.contains(DIALOG), "V1: no dialog title");
    assert!(
        preview.contains("payments-platform"),
        "V1: no workspace name"
    );
    assert!(preview.contains("1 change"), "V1: no change count");

    assert!(preview.contains(DIFF), "V2: no diff line");

    assert!(preview.contains("• 1 change"), "S1: no change badge");
    assert!(preview.contains("General •"), "S1: no dotted tab");

    assert_line_has(&preview, "▎Cancel", "Save", "S2 actions");

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, DIALOG, "preview cancelled");
    waits::wait_state(&mut s, "Not saved · keep editing", "cancel status");
    waits::wait_state(&mut s, "• 1 change", "edits intact");
    checkpoint(&mut s, &dir, "02-cancelled");

    support::press_step(&s, "ctrl-s");
    waits::wait_state(&mut s, DIALOG, "preview reopened");
    waits::wait_state(&mut s, DIFF, "diff line back");
    checkpoint(&mut s, &dir, "03-reopened");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, DIALOG, "preview cancelled again");
    checkpoint(&mut s, &dir, "04-cancelled");
    let back = live_text(&mut s);

    assert!(!back.contains(DIALOG), "N1: preview stuck open");
    assert!(!back.contains(DIFF), "N2: diff line stuck");
    assert!(back.contains("• 1 change"), "edits lost on cancel");
    assert!(back.contains(CRUMB_EDIT), "editor left on cancel");
    eprintln!("j1 editor_save_preview: preview, cancel, reopen, cancel");
}

#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_intro_f300() {
    let dir = j1_dir("j1_intro_f300");
    let case = j1_case("intro_f300", FIRSTUSE300, BOOT_SKIP);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-rain");
    let rain = live_text(&mut s);

    assert!(rain.contains("Enter Skip"), "V1: no skip footer");

    for glyph in ["─", "+", "·"] {
        assert!(rain.contains(glyph), "V2: no rain glyph `{glyph}`");
    }
    let first = rain.lines().next().unwrap_or_default();
    assert!(!first.trim().is_empty(), "V2: top line blank");

    assert!(!rain.contains("File"), "S1: menubar on intro");

    assert!(!rain.contains("Current directory"), "S2: tree on intro");

    support::drive_with_timeout(
        &mut s,
        &["enter", "wait:Current directory"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-manager");
    let manager = live_text(&mut s);
    assert!(
        manager.contains("+ New workspace"),
        "A1: no New workspace row"
    );
    assert!(
        manager.contains("no instances"),
        "A1: instances in fresh world"
    );
    assert!(manager.contains("File"), "A1: no menubar (S1 presence)");
    assert!(manager.contains("Workspaces"), "A1: no workspace chrome");
    assert!(!manager.contains(PICKER_TITLE), "A1: picker leaked open");

    assert!(!manager.contains("Enter Skip"), "N2: skip footer stuck");

    support::drive_with_timeout(
        &mut s,
        &["n", &format!("wait:{PRELUDE_S1}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "02-prelude");
    let prelude = live_text(&mut s);
    assert!(
        !prelude.contains("Current directory"),
        "A2: manager tree visible in prelude"
    );

    assert!(!prelude.contains("+ New workspace"), "N1: tree rows stuck");
    eprintln!("j1 intro_f300: rain, skip to manager, prelude");
}

#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_intro_f400() {
    let dir = j1_dir("j1_intro_f400");
    let case = j1_case("intro_f400", FIRST400, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-manager");
    let manager = live_text(&mut s);

    assert!(manager.contains("Workspaces"), "V1: no Workspaces chrome");
    assert!(manager.contains("no instances"), "V1: no empty status");
    assert!(
        manager.contains("+ New workspace"),
        "V1: no New workspace row"
    );

    assert!(manager.contains("Current directory"), "V2: no tree root");
    assert!(manager.contains("not saved"), "V2: no not-saved badge");
    assert!(
        manager.contains("Create a workspace from this directory."),
        "V2: no empty hint"
    );

    assert_line_has(&manager, "▎", "Current directory", "S1 cursor");

    assert!(manager.contains("Enter Launch"), "S2: no Launch hint");
    assert!(manager.contains("n New"), "S2: no New hint");

    support::drive_with_timeout(
        &mut s,
        &["n", &format!("wait:{PRELUDE_S1}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-prelude");
    let prelude = live_text(&mut s);

    assert!(!prelude.contains("Current directory"), "N1: tree stuck");

    support::press_step(&s, "escape");
    waits::wait_state(&mut s, "Cancelled · nothing created", "prelude cancelled");
    waits::wait_state(&mut s, "Current directory", "manager back");
    checkpoint(&mut s, &dir, "02-manager");
    assert!(
        !live_text(&mut s).contains(PRELUDE_S1),
        "N2: prelude stuck open"
    );
    eprintln!("j1 intro_f400: settled manager, prelude roundtrip");
}

#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_intro_skipped() {
    let dir = j1_dir("j1_intro_skipped");
    let case = j1_case("intro_skipped", FIRSTUSE40, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-intro");
    let intro = live_text(&mut s);

    assert!(intro.contains("Enter Skip"), "boot is not the intro");

    support::drive_with_timeout(
        &mut s,
        &["enter", "enter", "wait:Current directory"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-manager");
    let manager = live_text(&mut s);

    assert!(manager.contains("Workspaces"), "V1: no Workspaces chrome");
    assert!(manager.contains("no instances"), "V1: no empty status");
    assert!(
        manager.contains("+ New workspace"),
        "V1: no New workspace row"
    );

    assert!(manager.contains("not saved"), "V2: no not-saved badge");
    assert!(
        manager.contains("Enter launches with defaults"),
        "V2: no launch hint"
    );

    assert_line_has(&manager, "▎", "Current directory", "S1 cursor");

    assert!(manager.contains("Enter Launch"), "S2: no Launch hint");
    assert!(manager.contains("q Quit"), "S2: no Quit hint");

    assert!(!manager.contains("Enter Skip"), "N1: intro footer stuck");

    support::drive_with_timeout(
        &mut s,
        &["n", &format!("wait:{PRELUDE_S1}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-prelude");

    support::press_step(&s, "escape");
    waits::wait_state(&mut s, "Cancelled · nothing created", "prelude cancelled");
    waits::wait_state(&mut s, "Current directory", "manager back");
    checkpoint(&mut s, &dir, "02-manager");
    assert!(
        !live_text(&mut s).contains(PRELUDE_S1),
        "N2: prelude stuck open"
    );
    eprintln!("j1 intro_skipped: skip to manager, prelude roundtrip");
}

#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_manager_detail_drawer() {
    let dir = j1_dir("j1_manager_detail_drawer");
    let case = j1_case("manager_detail_drawer", RETURNING, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &["tab", &format!("wait:{DRAWER_FOOTER}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-drawer");
    let drawer = live_text(&mut s);

    assert_line_has(
        &drawer,
        "▎Current directory · payments-platform",
        "saved workspace",
        "V1 title",
    );

    assert_line_has(
        &drawer,
        "Working dir",
        "/workspace/payments-platform",
        "V2 workdir",
    );
    assert_line_has(&drawer, "◉ 7f3a", "running", "V2 running row");

    assert!(drawer.contains("↑↓ Move"), "S1: no Move hint");
    assert!(drawer.contains("Esc Back"), "S1: no Esc Back");

    assert_line_has(&drawer, "Launch", "Edit", "S2 actions");

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, DRAWER_FOOTER, "drawer closed");
    waits::wait_state(&mut s, TREE_FOOTER, "tree footer back");
    checkpoint(&mut s, &dir, "02-tree");
    let tree = live_text(&mut s);

    assert!(!tree.contains(DRAWER_FOOTER), "N1: Tab Actions stuck");
    assert!(tree.contains("Enter Launch"), "tree footer back");

    support::drive_with_timeout(
        &mut s,
        &["e", &format!("wait:{CRUMB_EDIT}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "03-editor-presence");
    support::press_step(&s, "escape");
    waits::wait_state(&mut s, TREE_FOOTER, "manager back");
    assert!(
        !live_text(&mut s).contains("› edit"),
        "N2: editor crumb stuck"
    );
    checkpoint(&mut s, &dir, "04-tree");
    eprintln!("j1 manager_detail_drawer: drawer, tree, editor roundtrip");
}

#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_manager_hard_launch_locked() {
    let dir = j1_dir("j1_manager_hard_launch_locked");
    let case = j1_case("manager_hard_launch_locked", HARD, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &["enter", &format!("wait:{PICKER_TITLE}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "00-picker");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, PICKER_TITLE, "picker cancelled");
    waits::wait_state(&mut s, "Enter Launch", "tree footer back");

    support::drive_with_timeout(
        &mut s,
        &[
            "down",
            "down",
            "down",
            "down",
            "down",
            "space",
            "down",
            "enter",
            "wait:▎ File",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-attached");
    let attached = live_text(&mut s);

    assert!(attached.contains("▎ File"), "V1: no focused menubar");
    assert!(
        attached.contains("jackin-data-pipeline-e0e0"),
        "V1: no instance id"
    );

    assert_line_has(
        &attached,
        "Attached to",
        "tabs and panes restored",
        "V2 attach",
    );

    assert!(attached.contains("main"), "S1: no branch");
    assert!(
        attached.contains("no account · shell"),
        "S1: no shell status"
    );

    assert!(attached.contains("Ctrl+B Prefix"), "S2: no Prefix hint");
    assert!(attached.contains("F10 Menu"), "S2: no Menu hint");

    assert!(
        !attached.contains(PICKER_TITLE),
        "N2: picker opened on attach"
    );

    support::drive_with_timeout(&mut s, &["ctrl-b", "d", "wait:Detached ·"], case.timeout_ms);
    waits::wait_state(&mut s, "Current directory", "manager back");
    checkpoint(&mut s, &dir, "02-detached");
    let manager = live_text(&mut s);
    assert!(manager.contains("keeps running"), "A1: no detach status");
    assert!(!manager.contains("Attached to"), "N1: capsule chrome stuck");
    eprintln!("j1 manager_hard_launch_locked: attach, detach to manager");
}

#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_manager_hard_launch_picker() {
    let dir = j1_dir("j1_manager_hard_launch_picker");
    let case = j1_case("manager_hard_launch_picker", HARD, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &["enter", &format!("wait:{PICKER_TITLE}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-picker");
    let picker = live_text(&mut s);

    assert_line_has(
        &picker,
        PICKER_TITLE,
        "payments-platform › the-architect",
        "V1 title",
    );

    assert_line_has(&picker, "Claude Code", "choose at start", "V2 choice");
    assert!(picker.contains("▎▪ Claude Code"), "V2: choice unfocused");

    assert!(
        picker.contains("! instance index unreadable"),
        "S1: index badge lost"
    );
    assert!(picker.contains("▲ daemon stale"), "S1: daemon badge lost");

    assert!(picker.contains("↑↓ Move"), "S2: no Move hint");
    assert!(
        picker.contains("Could not confirm running instances"),
        "S2: no footer warning"
    );

    support::drive_with_timeout(&mut s, &["down"], case.timeout_ms);
    std::thread::sleep(std::time::Duration::from_millis(300));
    checkpoint(&mut s, &dir, "02-moved");
    let moved = live_text(&mut s);
    assert_line_has(&moved, "▎", "Codex", "A1: choice did not move");
    assert_line_lacks(&moved, "Claude Code", "▎", "A1: old choice focused");

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, PICKER_TITLE, "picker cancelled");
    waits::wait_state(&mut s, "Enter Launch", "tree footer back");
    checkpoint(&mut s, &dir, "03-tree");
    let tree = live_text(&mut s);

    assert!(!tree.contains(PICKER_TITLE), "N1: picker stuck open");
    assert!(
        tree.contains("! instance index unreadable"),
        "badges lost on cancel"
    );

    support::drive_with_timeout(
        &mut s,
        &["down", "space", "wait:▾ payments-platform"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "04-expanded");
    support::drive_with_timeout(
        &mut s,
        &["space", "wait:▸ payments-platform"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "05-tree");
    assert!(
        !live_text(&mut s).contains("▾ payments-platform"),
        "N2: tree stuck expanded"
    );
    eprintln!("j1 manager_hard_launch_picker: picker, move, cancel, toggle");
}

#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_manager_launch_picker() {
    let dir = j1_dir("j1_manager_launch_picker");
    let case = j1_case("manager_launch_picker", RETURNING, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &["enter", &format!("wait:{PICKER_TITLE}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-picker");
    let picker = live_text(&mut s);

    assert_line_has(
        &picker,
        PICKER_TITLE,
        "payments-platform › the-architect",
        "V1 title",
    );

    assert_line_has(&picker, "Claude Code", "choose at start", "V2 choice");
    assert_line_has(&picker, "▪ Codex", "ready", "V2 alternate");
    assert!(picker.contains("▎▪ Claude Code"), "V2: choice unfocused");

    assert!(picker.contains("▸ payments-platform"), "S1: tree lost");

    assert!(picker.contains("Enter Choose"), "S2: no Choose hint");
    assert!(picker.contains("Esc Cancel"), "S2: no Cancel hint");

    support::drive_with_timeout(&mut s, &["down"], case.timeout_ms);
    std::thread::sleep(std::time::Duration::from_millis(300));
    checkpoint(&mut s, &dir, "02-moved");
    let moved = live_text(&mut s);
    assert_line_has(&moved, "▎", "Codex", "A1: choice did not move");
    assert_line_lacks(&moved, "Claude Code", "▎", "A1: old choice focused");

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, PICKER_TITLE, "picker cancelled");
    waits::wait_state(&mut s, "Enter Launch", "tree footer back");
    checkpoint(&mut s, &dir, "03-tree");
    let tree = live_text(&mut s);

    assert!(!tree.contains(PICKER_TITLE), "N1: picker stuck open");
    assert!(tree.contains("▸ payments-platform"), "tree lost on cancel");

    support::drive_with_timeout(
        &mut s,
        &["down", "space", "wait:▾ payments-platform"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "04-expanded");
    support::drive_with_timeout(
        &mut s,
        &["space", "wait:▸ payments-platform"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "05-tree");
    assert!(
        !live_text(&mut s).contains("▾ payments-platform"),
        "N2: tree stuck expanded"
    );
    eprintln!("j1 manager_launch_picker: picker, move, cancel, toggle");
}

#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_manager_tree_expanded() {
    const TREE_ROW: &str = "the-archite…ude Code";
    let dir = j1_dir("j1_manager_tree_expanded");
    let case = j1_case("manager_tree_expanded", RETURNING, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &["down", "space", "wait:▾ payments-platform"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-expanded");
    let expanded = live_text(&mut s);

    assert!(expanded.contains("▾ payments-platform"), "V1: not expanded");
    assert!(
        expanded.contains("Workspaces › payments-platform"),
        "V1: no crumb"
    );

    assert_line_has(&expanded, "◉ 7f3a", TREE_ROW, "V2 running row");
    assert_line_has(&expanded, "◌ c41e", "•", "V2 superseded row");

    assert_line_has(&expanded, "▎", "payments-platform", "S1 cursor");

    assert!(expanded.contains("→ Expand"), "S2: no Expand hint");
    assert!(expanded.contains(TREE_FOOTER), "S2: no Details hint");

    support::drive_with_timeout(
        &mut s,
        &["space", "wait:▸ payments-platform"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "02-collapsed");
    let collapsed = live_text(&mut s);
    assert!(
        !collapsed.contains(TREE_ROW),
        "A1: tree instance rows survive collapse"
    );

    assert!(
        collapsed.contains("the-architect · Claude Code · running"),
        "A1: detail rows lost on collapse"
    );

    support::drive_with_timeout(
        &mut s,
        &["space", "wait:▾ payments-platform"],
        case.timeout_ms,
    );
    assert!(
        live_text(&mut s).contains(TREE_ROW),
        "A2: instance rows lost on re-expand"
    );
    checkpoint(&mut s, &dir, "03-expanded");

    support::drive_with_timeout(&mut s, &["ctrl-q", "wait:Exit jackin❯?"], case.timeout_ms);
    checkpoint(&mut s, &dir, "04-quit-presence");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Exit jackin❯?", "quit cancelled");
    checkpoint(&mut s, &dir, "05-tree");
    let back = live_text(&mut s);
    assert!(!back.contains("Exit jackin❯?"), "N1: quit dialog stuck");
    assert!(!back.contains("▸ payments-platform"), "N2: tree collapsed");
    assert!(back.contains("▾ payments-platform"), "tree lost on cancel");
    eprintln!("j1 manager_tree_expanded: expand, collapse, expand, quit roundtrip");
}

#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_scenarios_first_use() {
    const PHRASE: &str = "Stand up, operator…";
    let dir = j1_dir("j1_scenarios_first_use");
    let case = j1_case("scenarios_first_use", FIRSTUSE40, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-phrases");
    let phrases = live_text(&mut s);

    assert!(phrases.contains(PHRASE), "V1: no intro phrase");
    assert!(phrases.contains(BOOT), "V1: no brand");

    assert!(phrases.contains("Enter Skip"), "V2: no skip footer");

    assert!(!phrases.contains("File"), "S1: menubar on intro");

    assert!(!phrases.contains("Current directory"), "S2: tree on intro");

    support::drive_with_timeout(
        &mut s,
        &["enter", "enter", "wait:Current directory"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-manager");
    let manager = live_text(&mut s);
    assert!(manager.contains("File"), "A1: no menubar (S1 presence)");
    assert!(
        manager.contains("no instances"),
        "A1: instances in fresh world"
    );

    assert!(!manager.contains(PHRASE), "N1: phrase stuck");
    assert!(!manager.contains("Enter Skip"), "N2: skip footer stuck");

    support::drive_with_timeout(
        &mut s,
        &["n", &format!("wait:{PRELUDE_S1}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "02-prelude");
    assert!(
        !live_text(&mut s).contains("Current directory"),
        "A2: manager tree visible in prelude"
    );
    eprintln!("j1 scenarios_first_use: phrases, skip to manager, prelude");
}

#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_scenarios_hard_cases() {
    let dir = j1_dir("j1_scenarios_hard_cases");
    let case = j1_case("scenarios_hard_cases", HARD, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);

    assert!(
        boot.contains("! instance index unreadable"),
        "V1: no index badge"
    );
    assert!(boot.contains("▲ daemon stale"), "V1: no daemon badge");

    assert!(boot.contains("▸ data-pipeline"), "V2: no data-pipeline row");
    assert!(boot.contains("3 running"), "V2: no running count");

    assert!(
        boot.contains("Could not confirm running instances"),
        "S1: no footer warning"
    );

    assert_line_has(&boot, "Instances", "▲ daemon stale · 3 s ago", "S2 stamp");

    support::drive_with_timeout(
        &mut s,
        &["enter", &format!("wait:{PICKER_TITLE}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-picker-presence");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, PICKER_TITLE, "picker cancelled");
    waits::wait_state(&mut s, "Enter Launch", "tree footer back");
    assert!(
        !live_text(&mut s).contains(PICKER_TITLE),
        "N1: picker stuck open"
    );

    support::drive_with_timeout(
        &mut s,
        &["down", "space", "wait:▾ payments-platform"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "02-expanded");
    support::drive_with_timeout(
        &mut s,
        &["space", "wait:▸ payments-platform"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "03-tree");
    let back = live_text(&mut s);
    assert!(
        !back.contains("▾ payments-platform"),
        "N2: tree stuck expanded"
    );
    assert!(
        back.contains("! instance index unreadable"),
        "badges lost on toggle"
    );
    eprintln!("j1 scenarios_hard_cases: degraded boot, picker+toggle roundtrips");
}

#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_scenarios_launch_failure() {
    const SUMMARY: &str = "The Construct network could not be attached";
    let dir = j1_dir("j1_scenarios_launch_failure");
    let case = j1_case("scenarios_launch_failure", FAILURE, BOOT_FAILURE);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    waits::wait_state(&mut s, SUMMARY, "failure summary");
    checkpoint(&mut s, &dir, "00-failure");
    let failure = live_text(&mut s);

    assert!(failure.contains("! Launch failed"), "V1: no dialog title");
    assert!(failure.contains(SUMMARY), "V1: no summary");

    assert!(failure.contains("run-202609030914-b5df"), "V2: no run id");
    assert!(
        failure.contains("Check that Docker's jackin-net bridge exists"),
        "V2: no next step"
    );

    assert!(
        failure.contains("retry 3/3 after 1600 ms"),
        "S1: no retry tail"
    );
    assert!(
        failure.contains("44 lines · b build log"),
        "S1: no log counter"
    );

    assert!(failure.contains("y Copy"), "S2: no Copy hint");
    assert!(failure.contains("Esc Close"), "S2: no Close hint");

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, SUMMARY, "failure dialog closed");
    waits::wait_state(&mut s, "still running in the Construct", "ack status");
    waits::wait_state(&mut s, "Current directory", "manager tree back");
    checkpoint(&mut s, &dir, "01-acknowledged");
    let acked = live_text(&mut s);

    assert!(!acked.contains(SUMMARY), "N1: summary stuck");
    assert!(!acked.contains("Run id"), "N2: dialog rows stuck");

    support::drive_with_timeout(
        &mut s,
        &["tab", &format!("wait:{DRAWER_FOOTER}")],
        case.timeout_ms,
    );
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, DRAWER_FOOTER, "drawer closed");
    waits::wait_state(&mut s, TREE_FOOTER, "tree footer back");
    checkpoint(&mut s, &dir, "02-tree");
    eprintln!("j1 scenarios_launch_failure: failure, ack, drawer roundtrip");
}

#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_scenarios_launch_running() {
    let dir = j1_dir("j1_scenarios_launch_running");
    let case = j1_case("scenarios_launch_running", LAUNCH, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    waits::wait_state(&mut s, "stage 3 of 11 · Credentials", "cockpit stage");
    checkpoint(&mut s, &dir, "00-cockpit");
    let cockpit = live_text(&mut s);

    assert!(
        cockpit.contains("Launch › payments-platform › the-architect"),
        "V1: no launch header"
    );
    assert!(
        cockpit.contains("stage 3 of 11 · Credentials"),
        "V1: no stage line"
    );

    assert!(cockpit.contains("✓ 01 Identity"), "V2: no stage 01");
    assert!(cockpit.contains("✓ 02 Role"), "V2: no stage 02");
    assert!(cockpit.contains("⠋ 03 Credentials"), "V2: no stage 03");

    assert!(
        cockpit.contains("account Claude · Work (session choice)"),
        "S1: no session choice"
    );

    assert!(cockpit.contains("i Container info"), "S2: no info hint");
    assert!(cockpit.contains("c Cancel"), "S2: no Cancel hint");

    support::drive_with_timeout(&mut s, &["d", "wait:run-2026"], case.timeout_ms);
    checkpoint(&mut s, &dir, "01-chip");
    support::press_step(&s, "d");
    waits::wait_gone(&mut s, "run-2026", "debug chip off");
    checkpoint(&mut s, &dir, "02-nochip");

    assert!(
        !live_text(&mut s).contains("run-2026"),
        "N1: debug chip stuck"
    );

    support::drive_with_timeout(&mut s, &["i", "wait:Debug info"], case.timeout_ms);
    checkpoint(&mut s, &dir, "03-info-presence");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Debug info", "info dialog closed");
    let back = live_text(&mut s);
    assert!(!back.contains("Debug info"), "N2: info dialog stuck");
    assert!(
        back.contains("stage 3 of 11 · Credentials"),
        "cockpit lost on close"
    );
    checkpoint(&mut s, &dir, "04-cockpit");
    eprintln!("j1 scenarios_launch_running: cockpit, chip toggle, info roundtrip");
}

#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_scenarios_outro_last() {
    const PHRASE: &str = "Stand up, operator…";
    let dir = j1_dir("j1_scenarios_outro_last");
    let case = j1_case("scenarios_outro_last", OUTRO40, BOOT_SKIP);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-warp");
    let warp = live_text(&mut s);

    assert!(warp.contains("Enter Skip"), "V1: no skip footer");

    for glyph in ["*", "+", "·"] {
        assert!(warp.contains(glyph), "V2: no outro glyph `{glyph}`");
    }

    let before = live_text(&mut s);
    support::press_step(&s, "x");
    std::thread::sleep(std::time::Duration::from_millis(300));
    let after = live_text(&mut s);
    assert_eq!(before, after, "A1/S1: x changed the warp frame");

    support::drive_with_timeout(&mut s, &["enter", "wait:Enter Close"], case.timeout_ms);
    checkpoint(&mut s, &dir, "01-caption");

    assert!(
        !live_text(&mut s).contains("Enter Skip"),
        "S2: caption keeps the warp hint"
    );
    support::press_step(&s, "enter");
    let exit = expect_exit_text(&s, case_timeout(&case), "outro quit");
    assert!(
        !exit.contains("panic"),
        "outro exit frame mentions panic: {exit:?}"
    );
    eprintln!("outro exit frame ({} bytes)", exit.len());

    let case2 = j1_case("outro_presence", FIRSTUSE40, BOOT);
    write_provenance_named(&dir, &case2, "provenance-s2.txt");
    let mut s2 = support::spawn_boot(&case2);
    waits::wait_state(&mut s2, PHRASE, "N1 presence: intro phrase renders");

    assert!(!warp.contains(PHRASE), "N1: intro phrase on outro");
    support::drive_with_timeout(
        &mut s2,
        &["enter", "enter", "wait:Current directory"],
        case2.timeout_ms,
    );

    assert!(!warp.contains("Current directory"), "N2: tree on outro");
    checkpoint(&mut s2, &dir, "02-presence");
    eprintln!("j1 scenarios_outro_last: warp, ignored x, caption, clean exit");
}

#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_scenarios_returning() {
    let dir = j1_dir("j1_scenarios_returning");
    let case = j1_case("scenarios_returning", RETURNING, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);

    assert!(boot.contains("Workspaces"), "V1: no Workspaces chrome");
    assert!(boot.contains("2 running"), "V1: no running count");
    assert!(
        boot.contains("Current directory · payments-platform"),
        "V1: no detail title"
    );

    assert_line_has(&boot, "◉ 7f3a", "running", "V2 running row");
    assert_line_has(&boot, "◌ c41e", "preserved · dirty", "V2 dirty row");

    assert_line_has(&boot, "▎", "Current directory", "S1 cursor");

    assert!(boot.contains("Enter Launch"), "S2: no Launch hint");
    assert!(boot.contains(TREE_FOOTER), "S2: no Details hint");
    assert!(boot.contains("q Quit"), "S2: no Quit hint");

    support::drive_with_timeout(
        &mut s,
        &["enter", &format!("wait:{PICKER_TITLE}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-picker-presence");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, PICKER_TITLE, "picker cancelled");
    waits::wait_state(&mut s, "Enter Launch", "tree footer back");
    assert!(
        !live_text(&mut s).contains(PICKER_TITLE),
        "N1: picker stuck open"
    );

    support::drive_with_timeout(
        &mut s,
        &["tab", &format!("wait:{DRAWER_FOOTER}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "02-drawer-presence");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, DRAWER_FOOTER, "drawer closed");
    waits::wait_state(&mut s, TREE_FOOTER, "tree footer back");
    let back = live_text(&mut s);
    assert!(!back.contains(DRAWER_FOOTER), "N2: drawer stuck open");
    assert_line_has(&back, "▎", "Current directory", "cursor moved");
    checkpoint(&mut s, &dir, "03-boot");
    eprintln!("j1 scenarios_returning: boot, picker+drawer roundtrips");
}

#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_settings_agents() {
    let dir = j1_dir("j1_settings_agents");
    let case = j1_case("settings_agents", ACCOUNTS, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "s",
            "wait:Settings › global",
            "4",
            "enter",
            "wait:Agent runtime mode",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-agents");
    let agents = live_text(&mut s);

    assert!(agents.contains("Agent runtime mode"), "V1: no mode header");
    assert_line_has(&agents, "Claude Code", "sync", "V1 Claude row");

    assert_line_has(&agents, "Kimi Code", "ignore", "V2 Kimi row");
    assert!(
        agents.contains("no credentials handed to the container"),
        "V2: no ignore note"
    );

    assert!(
        agents.contains("accounts are registered in Accounts (c)"),
        "S1: no registry hint"
    );

    assert!(agents.contains("Space Cycle mode"), "S2: no Cycle hint");
    assert!(agents.contains("d Reset to sync"), "S2: no Reset hint");

    support::drive_with_timeout(&mut s, &["[", "wait:GH_TOKEN"], case.timeout_ms);
    checkpoint(&mut s, &dir, "02-env");
    let env = live_text(&mut s);
    assert!(env.contains("1–5 Jump"), "A1: no TABS footer");

    assert!(!env.contains("Agent runtime mode"), "N1: agents body stuck");
    assert!(!env.contains("Space Cycle mode"), "N2: agents footer stuck");

    support::drive_with_timeout(&mut s, &["]", "wait:Agent runtime mode"], case.timeout_ms);
    assert!(
        live_text(&mut s).contains("Kimi Code"),
        "A2: agents body lost"
    );
    checkpoint(&mut s, &dir, "03-agents");
    eprintln!("j1 settings_agents: agents, env jump, agents back");
}

#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_settings_mounts() {
    const ROW: &str = "/home/agent/.gitconfig";
    let dir = j1_dir("j1_settings_mounts");
    let case = j1_case("settings_mounts", ACCOUNTS, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "s",
            "wait:Settings › global",
            "2",
            "enter",
            &format!("wait:{ROW}"),
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-mounts");
    let mounts = live_text(&mut s);

    assert_line_has(&mounts, "Destination", "Scope", "V1 header");
    assert_line_has(&mounts, ROW, "global", "V1 gitconfig row");

    assert_line_has(&mounts, "/home/agent/.kube", "role sre", "V2 kube row");

    assert!(mounts.contains("▎›"), "S1: no body cursor");

    assert!(mounts.contains("s Scope"), "S2: no Scope hint");
    assert!(mounts.contains("a Add mount"), "S2: no Add hint");

    support::drive_with_timeout(
        &mut s,
        &["[", "wait:Sign off commits (DCO)"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "02-general");
    let general = live_text(&mut s);
    assert!(general.contains("1–5 Jump"), "A1: no TABS footer");

    assert!(!general.contains(ROW), "N1: mounts body stuck");
    assert!(!general.contains("s Scope"), "N2: mounts footer stuck");

    support::drive_with_timeout(&mut s, &["]", &format!("wait:{ROW}")], case.timeout_ms);
    assert!(
        live_text(&mut s).contains("/home/agent/.kube"),
        "A2: mounts body lost"
    );
    checkpoint(&mut s, &dir, "03-mounts");
    eprintln!("j1 settings_mounts: mounts, general jump, mounts back");
}

#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_settings_route() {
    const DCO: &str = "Sign off commits (DCO)";
    let dir = j1_dir("j1_settings_route");
    let case = j1_case("settings_route", ACCOUNTS, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(&mut s, &["s", "wait:Settings › global"], case.timeout_ms);
    checkpoint(&mut s, &dir, "01-general");
    let general = live_text(&mut s);

    assert!(
        general.contains("Settings › global › General"),
        "V1: no crumb"
    );
    assert!(general.contains("Commits"), "V1: no Commits section");

    assert!(
        general.contains("[✓] Add Co-authored-by trailer"),
        "V2: no trailer flag"
    );
    assert!(
        general.contains("[✓] Sign off commits (DCO)"),
        "V2: no DCO flag"
    );

    assert!(general.contains("1–5 Jump"), "S1: no 1–5 Jump");
    assert!(general.contains("Enter Body"), "S1: no Enter Body");

    assert!(general.contains("Co-authored-by:"), "S2: no trailer line");
    assert!(
        general.contains("Signed-off-by: Alexey Zhokhov"),
        "S2: no signoff line"
    );

    support::drive_with_timeout(
        &mut s,
        &["]", "wait:/home/agent/.gitconfig"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "02-mounts");
    let mounts = live_text(&mut s);
    assert!(mounts.contains("1–5 Jump"), "A1: TABS focus lost");

    assert!(!mounts.contains(DCO), "N1: General body stuck");
    assert!(!mounts.contains("Co-authored-by:"), "N2: trailers stuck");

    support::drive_with_timeout(&mut s, &["[", &format!("wait:{DCO}")], case.timeout_ms);
    assert!(
        live_text(&mut s).contains("Co-authored-by:"),
        "A2: General body lost"
    );
    checkpoint(&mut s, &dir, "03-general");
    eprintln!("j1 settings_route: general, mounts jump, general back");
}

#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_settings_save_preview() {
    const DIALOG: &str = "Save settings";
    const DIFF1: &str = "~ trust github.com/chainargos/roles true → false";
    const DIFF2: &str = "~ trust github.com/acme-labs/roles-experimental false → true";
    let dir = j1_dir("j1_settings_save_preview");
    let case = j1_case("settings_save_preview", ACCOUNTS, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "s",
            "wait:Settings › global",
            "5",
            "enter",
            "wait:Role sources",
            "space",
            "down",
            "space",
            "wait:• 2 changes",
        ],
        case.timeout_ms,
    );
    support::press_step(&s, "ctrl-s");
    waits::wait_state(&mut s, DIALOG, "save preview open");
    waits::wait_state(&mut s, DIFF1, "first diff line");
    waits::wait_state(&mut s, DIFF2, "second diff line");
    checkpoint(&mut s, &dir, "01-preview");
    let preview = live_text(&mut s);

    assert!(preview.contains(DIALOG), "V1: no dialog title");
    assert!(preview.contains("global config"), "V1: no scope");
    assert!(preview.contains("config.toml"), "V1: no config path");
    assert!(preview.contains("2 changes"), "V1: no change count");

    assert!(preview.contains(DIFF1), "V2: no chainargos diff");
    assert!(preview.contains(DIFF2), "V2: no acme-labs diff");

    assert!(preview.contains("• 2 changes"), "S1: no change badge");
    assert!(preview.contains("Trust •"), "S1: no dotted tab");

    assert_line_has(&preview, "▎Cancel", "Save", "S2 actions");

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, DIALOG, "preview cancelled");
    waits::wait_state(&mut s, "Save aborted · settings unchanged", "cancel status");
    waits::wait_state(&mut s, "• 2 changes", "edits intact");
    checkpoint(&mut s, &dir, "02-cancelled");

    support::press_step(&s, "ctrl-s");
    waits::wait_state(&mut s, DIALOG, "preview reopened");
    waits::wait_state(&mut s, DIFF1, "diff line back");
    checkpoint(&mut s, &dir, "03-reopened");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, DIALOG, "preview cancelled again");
    checkpoint(&mut s, &dir, "04-cancelled");
    let back = live_text(&mut s);

    assert!(!back.contains(DIALOG), "N1: preview stuck open");
    assert!(!back.contains(DIFF1), "N2: diff line stuck");
    assert!(back.contains("• 2 changes"), "edits lost on cancel");
    assert!(back.contains("Role sources"), "trust body lost on cancel");
    eprintln!("j1 settings_save_preview: preview, cancel, reopen, cancel");
}

#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_settings_trust() {
    let dir = j1_dir("j1_settings_trust");
    let case = j1_case("settings_trust", ACCOUNTS, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "s",
            "wait:Settings › global",
            "5",
            "enter",
            "wait:Role sources",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-trust");
    let trust = live_text(&mut s);

    assert!(trust.contains("Role sources"), "V1: no Role sources");
    assert!(
        trust.contains("3 trusted · 1 untrusted"),
        "V1: no trust counts"
    );

    assert_line_has(
        &trust,
        "github.com/chainargos/roles",
        "[✓] trusted",
        "V2 chainargos row",
    );
    assert_line_has(
        &trust,
        "github.com/acme-labs/roles-experimental",
        "[ ] untrusted",
        "V2 acme-labs row",
    );

    assert!(
        trust.contains("An untrusted source blocks + Load role"),
        "S1: no trust warning"
    );

    assert!(trust.contains("Space Toggle trust"), "S2: no Toggle hint");
    assert!(trust.contains("o Open source"), "S2: no Open hint");

    support::drive_with_timeout(&mut s, &["[", "wait:Agent runtime mode"], case.timeout_ms);
    checkpoint(&mut s, &dir, "02-agents");
    let agents = live_text(&mut s);
    assert!(agents.contains("1–5 Jump"), "A1: no TABS footer");

    assert!(!agents.contains("Role sources"), "N1: trust body stuck");
    assert!(
        !agents.contains("Space Toggle trust"),
        "N2: trust footer stuck"
    );

    support::drive_with_timeout(&mut s, &["]", "wait:Role sources"], case.timeout_ms);
    assert!(
        live_text(&mut s).contains("3 trusted · 1 untrusted"),
        "A2: trust body lost"
    );
    checkpoint(&mut s, &dir, "03-trust");
    eprintln!("j1 settings_trust: trust, agents jump, trust back");
}
