use std::path::{Path, PathBuf};

use crate::support::Session;
use crate::support::state_waits as waits;
use crate::support::{self, Case, Color, HOLLA};

const BOOT: &str = "holla❯";
const COLS: u16 = 120;
const ROWS: u16 = 40;

const ROUTES: [(&[&str], &str, &[&str]); 3] = [
    (
        &[
            "--scenario",
            "parity-discovery",
            "--motion",
            "paused",
            "--frame",
            "40",
        ],
        "discovery",
        &["probe · Rust project", "Suggested here"],
    ),
    (
        &[
            "--scenario",
            "parity-history",
            "--motion",
            "paused",
            "--frame",
            "40",
        ],
        "history",
        &["holla · Rust project", "Run check"],
    ),
    (
        &[
            "--scenario",
            "parity-files",
            "--motion",
            "paused",
            "--frame",
            "40",
        ],
        "files",
        &["no project", "~/work/notes"],
    ),
];

fn journey_case(name: &str, args: &'static [&'static str]) -> Case {
    Case::dynamic(
        format!("journeys/holla/{name}"),
        HOLLA,
        args,
        COLS,
        ROWS,
        Color::Truecolor,
        BOOT,
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
        "profile: tuiscotti-default\ntransport: pty\nport_of: a91d0ebf4bef97fa5138dafad54ed9fd57e3362e\nbinary: {}\nsha256: {}\nsize_bytes: {}\nmodified_unix: {modified}\nargv: {}\n",
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
#[ignore = "holla journey; run with --ignored"]
fn journey_holla_core_routes() {
    let dir = journey_dir("journey_holla_core_routes");

    let holla = support::try_resolve_bin(HOLLA).unwrap_or_else(|e| panic!("resolve {HOLLA}: {e}"));
    let help = std::process::Command::new(&holla.path)
        .arg("--help")
        .output()
        .unwrap_or_else(|e| panic!("holla --help failed: {e}"));
    assert!(
        help.status.success(),
        "holla --help exits 0 (got {})",
        help.status
    );
    let text = String::from_utf8_lossy(&help.stdout);
    std::fs::write(dir.join("scenario-help.txt"), text.as_bytes())
        .unwrap_or_else(|e| panic!("write scenario-help.txt: {e}"));
    for scenario in ["parity-discovery", "parity-history", "parity-files"] {
        assert!(
            text.contains(scenario),
            "scenario registry lists `{scenario}` ({}/scenario-help.txt)",
            dir.display()
        );
    }
    eprintln!("enumeration: parity-discovery, parity-history, parity-files all listed");

    for (i, (args, slug, needles)) in ROUTES.into_iter().enumerate() {
        let case = journey_case(&format!("core_routes_{slug}"), args);
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
#[ignore = "holla journey; run with --ignored"]
fn journey_holla_trust_accept() {
    const PROMPT: &str = "Trust ~/work/acme/apps/frontend/mise.toml?";
    const ACCEPTED: &str = "Trusted ~/work/acme/apps/frontend/mise.toml";

    let dir = journey_dir("journey_holla_trust_accept");
    let case = journey_case(
        "trust_accept",
        &["--scenario", "monorepo-child", "--motion", "reduced"],
    );
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &["type:test", "enter", &format!("wait:{PROMPT}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-trust-prompt");
    support::press_step(&s, "right");
    support::press_step(&s, "enter");
    support::wait_screen(
        &mut s,
        support::DEFAULT_WAIT,
        "trust accepted and dialog closed",
        |screen| {
            let text = support::screen_text(screen);

            text.contains(ACCEPTED) && !text.contains(PROMPT)
        },
    );
    checkpoint(&mut s, &dir, "02-trusted");
    eprintln!("journey trust_accept: prompt raised, accepted, dialog closed");
}

#[test]
#[ignore = "holla journey; run with --ignored"]
fn journey_holla_jump_error_recovery() {
    let dir = journey_dir("journey_holla_jump_error_recovery");
    let case = journey_case(
        "jump_error_recovery",
        &["--scenario", "parity-browser", "--motion", "reduced"],
    );
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &["type:Browse ~/work/site", "enter", "wait:16 entries"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-listing");
    support::press_step(&s, "g");

    waits::wait_state(&mut s, "Enter Switch", "jump picker open");
    checkpoint(&mut s, &dir, "02-picker");
    support::drive_with_timeout(
        &mut s,
        &["type:nope", "enter", "wait:no such file or directory"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "03-error");

    support::press_step(&s, "escape");
    support::wait_screen(&mut s, support::DEFAULT_WAIT, "query cleared", |screen| {
        !support::screen_text(screen).contains("▎ nope")
    });
    checkpoint(&mut s, &dir, "04-query-cleared");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Enter Switch", "picker dismissed");
    waits::wait_state(&mut s, "Open / preview", "browser footer back");
    waits::wait_state(&mut s, "16 entries", "listing intact");
    checkpoint(&mut s, &dir, "05-recovered");
    eprintln!("journey jump_error_recovery: error seen, picker cancelled, listing intact");
}
