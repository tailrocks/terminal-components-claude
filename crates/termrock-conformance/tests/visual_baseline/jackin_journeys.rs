use std::path::{Path, PathBuf};

use crate::support::Session;
use crate::support::state_waits as waits;
use crate::support::{self, Case, Color, JACKIN};

const BOOT: &str = "jackin❯";
const BOOT_FAILURE: &str = "Launch failed";
const COLS: u16 = 120;
const ROWS: u16 = 40;

const ROUTES: [(&[&str], &str, &[&str]); 2] = [
    (
        &[
            "--scenario",
            "returning",
            "--motion",
            "paused",
            "--frame",
            "40",
        ],
        "manager",
        &["Current directory · payments-platform", "Working dir"],
    ),
    (
        &[
            "--scenario",
            "launch-running",
            "--motion",
            "paused",
            "--frame",
            "40",
        ],
        "cockpit",
        &[
            "Launch › payments-platform › the-architect",
            "stage 3 of 11 · Credentials",
        ],
    ),
];

fn journey_case(name: &str, args: &'static [&'static str], boot: &'static str) -> Case {
    Case::dynamic(
        format!("journeys/jackin/{name}"),
        JACKIN,
        args,
        COLS,
        ROWS,
        Color::Truecolor,
        boot,
    )
}

fn journey_dir(test: &str) -> PathBuf {
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
        "profile: tuiscotti-default\ntransport: pty\nport_of: 41ef28cca744998c8af936e758856ecb2d4b48e3\nbinary: {}\nsha256: {}\nsize_bytes: {}\nmodified_unix: {modified}\nargv: {}\n",
        subject.path.display(),
        subject.sha256,
        subject.len,
        argv.join(" "),
    );
    std::fs::write(dir.join(file), &body)
        .unwrap_or_else(|e| panic!("write {}: {e}", dir.join(file).display()));
    eprintln!("journey provenance ({file}): {body}");
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

#[test]
#[ignore = "jackin journey; run with --ignored"]
fn journey_jackin_core_routes() {
    let dir = journey_dir("journey_jackin_core_routes");

    let jackin =
        support::try_resolve_bin(JACKIN).unwrap_or_else(|e| panic!("resolve {JACKIN}: {e}"));
    let help = std::process::Command::new(&jackin.path)
        .arg("--help")
        .output()
        .unwrap_or_else(|e| panic!("jackin-preview --help failed: {e}"));
    assert!(
        help.status.success(),
        "jackin-preview --help exits 0 (got {})",
        help.status
    );
    let text = String::from_utf8_lossy(&help.stdout);
    std::fs::write(dir.join("scenario-help.txt"), text.as_bytes())
        .unwrap_or_else(|e| panic!("write scenario-help.txt: {e}"));
    for scenario in [
        "returning",
        "accounts-mixed",
        "launch-running",
        "launch-failure",
    ] {
        assert!(
            text.contains(scenario),
            "scenario registry lists `{scenario}` ({}/scenario-help.txt)",
            dir.display()
        );
    }
    eprintln!("enumeration: returning, accounts-mixed, launch-running, launch-failure all listed");

    for (i, (args, slug, needles)) in ROUTES.into_iter().enumerate() {
        let case = journey_case(&format!("core_routes_{slug}"), args, BOOT);
        write_provenance_named(&dir, &case, &format!("provenance-{slug}.txt"));
        let mut s = support::spawn_boot(&case);
        for needle in needles.iter() {
            waits::wait_state(&mut s, needle, &format!("route {slug}"));
        }
        checkpoint(&mut s, &dir, &format!("{i:02}-{slug}"));
    }
    eprintln!("journey core_routes: visited {} routes", ROUTES.len());
}

#[test]
#[ignore = "jackin journey; run with --ignored"]
fn journey_jackin_editor_mounts_tab() {
    const CRUMB: &str = "Workspaces › payments-platform › edit";

    let dir = journey_dir("journey_jackin_editor_mounts_tab");
    let case = journey_case(
        "editor_mounts_tab",
        &[
            "--scenario",
            "returning",
            "--motion",
            "paused",
            "--frame",
            "40",
        ],
        BOOT,
    );
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-manager");

    support::drive_with_timeout(&mut s, &["e", &format!("wait:{CRUMB}")], case.timeout_ms);
    waits::wait_state(&mut s, "Keep awake", "general tab body");
    checkpoint(&mut s, &dir, "01-general");
    support::drive_with_timeout(&mut s, &["2", "enter", "wait:+ Add mount"], case.timeout_ms);
    waits::wait_state(&mut s, "r Read-only", "mounts tab footer");
    waits::wait_state(&mut s, CRUMB, "editor crumb still up");
    checkpoint(&mut s, &dir, "02-mounts");
    eprintln!("journey editor_mounts_tab: general then mounts, both proven");
}

#[test]
#[ignore = "jackin journey; run with --ignored"]
fn journey_jackin_settings_env_tab() {
    const CRUMB_ENV: &str = "Settings › global › Environments";

    let dir = journey_dir("journey_jackin_settings_env_tab");
    let case = journey_case(
        "settings_env_tab",
        &[
            "--scenario",
            "accounts-mixed",
            "--motion",
            "paused",
            "--frame",
            "40",
        ],
        BOOT,
    );
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-accounts");

    support::drive_with_timeout(&mut s, &["s", "wait:Settings › global"], case.timeout_ms);
    waits::wait_state(&mut s, "Sign off commits (DCO)", "general tab body");
    checkpoint(&mut s, &dir, "01-general");
    support::drive_with_timeout(&mut s, &["3", "enter", "wait:GH_TOKEN"], case.timeout_ms);
    waits::wait_state(&mut s, CRUMB_ENV, "environments crumb");
    checkpoint(&mut s, &dir, "02-env");
    eprintln!("journey settings_env_tab: general then environments, both proven");
}

#[test]
#[ignore = "jackin journey; run with --ignored"]
fn journey_jackin_editor_save_cancel() {
    const CRUMB: &str = "Workspaces › payments-platform › edit";
    const DIALOG: &str = "Save workspace";

    let dir = journey_dir("journey_jackin_editor_save_cancel");
    let case = journey_case(
        "editor_save_cancel",
        &[
            "--scenario",
            "returning",
            "--motion",
            "paused",
            "--frame",
            "40",
        ],
        BOOT,
    );
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-manager");

    support::drive_with_timeout(&mut s, &["e", &format!("wait:{CRUMB}")], case.timeout_ms);
    checkpoint(&mut s, &dir, "01-editor");
    support::drive_with_timeout(
        &mut s,
        &["down", "down", "down", "space", "wait:• 1 change"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "02-dirty");
    support::press_step(&s, "ctrl-s");
    waits::wait_state(&mut s, DIALOG, "save preview open");
    waits::wait_state(&mut s, "~ keep_awake true → false", "preview diff line");
    checkpoint(&mut s, &dir, "03-save-preview");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, DIALOG, "preview cancelled");
    waits::wait_state(&mut s, "Not saved · keep editing", "cancel status");
    waits::wait_state(&mut s, "• 1 change", "edits intact");
    waits::wait_state(&mut s, CRUMB, "editor still up");
    checkpoint(&mut s, &dir, "04-cancelled");
    eprintln!("journey editor_save_cancel: dirty, preview, cancelled, edits intact");
}

#[test]
#[ignore = "jackin journey; run with --ignored"]
fn journey_jackin_launch_failure_dialog() {
    const SUMMARY: &str = "The Construct network could not be attached";

    let dir = journey_dir("journey_jackin_launch_failure_dialog");
    let case = journey_case(
        "launch_failure_dialog",
        &[
            "--scenario",
            "launch-failure",
            "--motion",
            "paused",
            "--frame",
            "240",
        ],
        BOOT_FAILURE,
    );
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    waits::wait_state(&mut s, SUMMARY, "failure summary");
    waits::wait_state(
        &mut s,
        "Check that Docker's jackin-net bridge exists",
        "failure next step",
    );
    checkpoint(&mut s, &dir, "00-failure-dialog");

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, SUMMARY, "failure dialog closed");
    waits::wait_state(
        &mut s,
        "still running in the Construct",
        "ack status on manager",
    );

    waits::wait_state(&mut s, "Current directory", "manager tree back");
    waits::wait_state(&mut s, "Tab Details", "manager footer back");
    checkpoint(&mut s, &dir, "01-acknowledged");
    eprintln!("journey launch_failure_dialog: failure proven, acknowledged, manager back");
}
