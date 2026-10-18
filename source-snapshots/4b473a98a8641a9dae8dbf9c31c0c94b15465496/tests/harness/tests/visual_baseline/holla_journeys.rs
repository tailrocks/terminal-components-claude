//! Holla app-journey suite, slice 1 (VB Phase-7c, §15).
//!
//! Multi-step PTY journeys over the real `holla` binary built from this
//! worktree's verified VB sources (`env!("CARGO_BIN_EXE_holla")`, the
//! harness `[[bin]]` compiled from `../../src/bin/holla/main.rs`).
//! Journeys assert needle state across steps — they never gate snapshots,
//! so every case here uses owned (dynamic) names: the approval-inventory
//! parser in [`crate::support::suite_capture_names`] only scans static
//! representative declarations, and journey names must stay out of the
//! committed `snapshots/` inventory.
//!
//! Per-test scratch (checkpoints + provenance) lands under this harness
//! crate's `target/tuiscotti/journeys/<test>/` (gitignored, unique per
//! test fn). Checkpoints are text frames (`.txt`); `provenance.txt`
//! records the binary path, size, mtime, and argv behind each journey.
//! Typed input is synthetic and in-memory only (simulation data). Journeys
//! never run real cleanup, never touch Git state, and never perform
//! destructive operations: the trust flow accepts a fixture-world prompt
//! and the error flow cancels its picker without jumping anywhere.

use std::path::{Path, PathBuf};
use std::time::Duration;

use tuiscotti::tui::Session;

use crate::support::state_waits as waits;
use crate::support::{self, Case, Color, HOLLA};

const BOOT: &str = "holla❯";
const COLS: u16 = 120;
const ROWS: u16 = 40;

/// Core routes visited in journey 1: argv, checkpoint slug, and the route
/// needles that prove each boot landed in its own fixture world (header
/// project label plus one content row, both read off the approved
/// `holla/parity/<route>/120x40/truecolor` frames these argvs reproduce).
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

/// Owned-name journey case at the canonical 120x40 truecolor geometry.
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

/// Unique scratch dir for one journey test (created, never shared).
fn journey_dir(test: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/tuiscotti/journeys")
        .join(test);
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("mkdir {}: {e}", dir.display()));
    dir
}

/// Record the binary path + identity behind a journey: absolute path,
/// byte size, mtime, and argv. Written to `provenance.txt`, echoed too.
fn write_provenance(dir: &Path, case: &Case) {
    write_provenance_named(dir, case, "provenance.txt");
}

/// [`write_provenance`] with an explicit filename, for multi-session
/// journeys where each session's argv deserves its own record.
fn write_provenance_named(dir: &Path, case: &Case, file: &str) {
    let meta = std::fs::metadata(case.bin).unwrap_or_else(|e| panic!("stat {}: {e}", case.bin));
    let modified = meta
        .modified()
        .map(|t| format!("{t:?}"))
        .unwrap_or_else(|_| "unknown".to_string());
    let argv = support::argv_for(case);
    let body = format!(
        "profile: tuiscotti-default\ntransport: pty\nbinary: {}\nsize_bytes: {}\nmodified: {modified}\nargv: {}\n",
        case.bin,
        meta.len(),
        argv.join(" "),
    );
    std::fs::write(dir.join(file), &body)
        .unwrap_or_else(|e| panic!("write {}: {e}", dir.join(file).display()));
    eprintln!("journey provenance ({file}): {body}");
}

/// Save the live screen text as `<name>.txt` in the journey dir.
fn checkpoint(s: &mut Session, dir: &Path, name: &str) {
    let obs = s
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

/// Slice 1, journey 1: enumerate the scenario definitions (the binary's
/// own `--help` must list the core route scenarios), then visit the core
/// routes — discovery, history, files — one session per scenario, proving
/// each with its header project label plus a content row.
#[test]
#[ignore = "holla journey; run with --ignored"]
fn journey_holla_core_routes() {
    let dir = journey_dir("journey_holla_core_routes");

    let help = std::process::Command::new(HOLLA)
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

    // By-value iteration copies each `&[&str]` out of the const's promoted
    // statics, so `args` already is `&'static [&'static str]`.
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

/// Slice 1, journey 2: the trust/confirmation flow on the monorepo-child
/// fixture with a synthetic query (`test`, in-memory only): Enter raises
/// the `Trust …mise.toml?` prompt, Right+Enter accepts it, and the
/// `Trusted …` status plus the closed prompt prove the accept. The accept
/// runs the fixture world's simulated task — no real command executes.
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

    support::drive(
        &mut s,
        &["type:test", "enter", &format!("wait:{PROMPT}")],
        case_timeout(&case),
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
            // The full prompt (with `?`) is checked for absence: it is a
            // prefix of the accepted line, so a short needle would lie.
            text.contains(ACCEPTED) && !text.contains(PROMPT)
        },
    );
    checkpoint(&mut s, &dir, "02-trusted");
    eprintln!("journey trust_accept: prompt raised, accepted, dialog closed");
}

/// Slice 1, journey 3: an error/empty-data case with assertion-proven
/// state. In the finished browser listing (`16 entries`), `g` opens the
/// Go-to-path picker, a synthetic bad path (`nope`) produces the
/// `no such file or directory` picker error (checkpointed while present),
/// then two Escapes recover (the first clears the query, the second
/// cancels the picker) — proven by the picker's non-vacuous disappearance
/// plus the intact listing behind it. Nothing jumps, nothing is trashed:
/// the bad path never leaves the picker.
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

    support::drive(
        &mut s,
        &["type:Browse ~/work/site", "enter", "wait:16 entries"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-listing");
    support::press_step(&s, "g");
    // The picker title "Go to path" also lives in the browser footer
    // (`g` hint), so it proves nothing in either direction. The footer
    // swap is the signal: picker footer says "Enter Switch", browser
    // footer says "Enter Open / preview" — each absent in the other.
    waits::wait_state(&mut s, "Enter Switch", "jump picker open");
    checkpoint(&mut s, &dir, "02-picker");
    support::drive(
        &mut s,
        &["type:nope", "enter", "wait:no such file or directory"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "03-error");
    // Recovery takes two Escapes by construction (`Picker::on_key`): the
    // first clears the non-empty query, the second cancels the picker.
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
