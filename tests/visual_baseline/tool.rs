//! Tool mode of the test binary for `cargo xtask visual`: when
//! `VISUAL_TOOL` is set, `main` runs one of these instead of the tests.
//! They reuse the scenario tables and the tuiscotti renderer, so xtask needs
//! neither.
//!
//! - `list`: one TSV row per scenario:
//!   `name  test_name  table  tier  pending  approval` where `approval` is
//!   `reapproved` (tuiscotti `.ansi`), `legacy` (tuisnap's four files) or
//!   `missing`.
//! - `render`: `VISUAL_NAME=<store name>` renders the approved `.ansi` to
//!   `target/tuiscotti/render/<name>.{png,html}` and prints both paths.
//! - `capture`: `VISUAL_NAME=<store name>` captures that scenario once,
//!   without a gate, and prints its `.ansi` (cursor trailer included). It
//!   runs as `probe/<VISUAL_AS>/<name>` so it never shares a `HOME` with
//!   the real scenario. Used by the `env_isolation` check.
//! - `env-probe`: spawns `/bin/sh` through the capture spawn path and prints
//!   the environment the child sees (`env | sort`).
//! - `harness-hash`: prints the oracle `harness_sha256` of `VISUAL_DIR`
//!   (default `tests/visual_baseline/`), for `seal` and `sync-oracle`.

use crate::scenarios;
use crate::support;

pub fn run(tool: &str) -> Result<(), String> {
    match tool {
        "list" => {
            list();
            Ok(())
        }
        "render" => {
            let name = std::env::var("VISUAL_NAME")
                .map_err(|_| "set VISUAL_NAME=<scenario name>".to_string())?;
            render(&name)
        }
        "capture" => {
            let name = std::env::var("VISUAL_NAME")
                .map_err(|_| "set VISUAL_NAME=<scenario name>".to_string())?;
            let tag = std::env::var("VISUAL_AS").unwrap_or_else(|_| "capture".into());
            capture(&name, &tag)
        }
        "env-probe" => {
            env_probe();
            Ok(())
        }
        "harness-hash" => {
            let dir = std::env::var_os("VISUAL_DIR")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(crate::oracle::harness_dir);
            if !dir.is_dir() {
                return Err(format!("{} is not a directory", dir.display()));
            }
            println!("{}", crate::oracle::harness_sha256(&dir));
            Ok(())
        }
        other => Err(format!(
            "unknown VISUAL_TOOL `{other}` (list, render, capture, env-probe, harness-hash)"
        )),
    }
}

fn approval(name: &str) -> &'static str {
    let root = support::approved_root();
    if support::is_reapproved(name) {
        "reapproved"
    } else if root.join(format!("{name}.ansi")).exists() {
        "legacy"
    } else {
        "missing"
    }
}

fn list() {
    for s in scenarios::all() {
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}",
            s.name,
            s.test_name(),
            s.kind.table(),
            s.tier,
            if s.pending { "pending" } else { "stored" },
            approval(&s.name),
        );
    }
}

fn render(name: &str) -> Result<(), String> {
    let name = name.replace("::", "/");
    if !scenarios::all().iter().any(|s| s.name == name) {
        return Err(format!(
            "`{name}` is not a scenario (cargo xtask visual list)"
        ));
    }
    let (png, html) = support::render_approved(&name)?;
    println!("{}", png.display());
    println!("{}", html.display());
    Ok(())
}

/// Marker the probe shell prints after the environment.
pub const ENV_PROBE_END: &str = "END-OF-ENV";

fn env_probe() {
    let case = support::Case::new(
        "probe/env/200x50/nocolor",
        "/bin/sh",
        &["-c", "env | sort; echo END-OF-ENV; sleep 30"],
        200,
        50,
        support::Color::NoColorEnv,
        ENV_PROBE_END,
    );
    let mut session = support::spawn_boot(&case);
    let frame = session
        .wait_stable(std::time::Duration::from_millis(100))
        .unwrap_or_else(|e| panic!("env probe never settled: {e}"));
    print!("{}", frame.text());
}

fn capture(name: &str, tag: &str) -> Result<(), String> {
    let name = name.replace("::", "/");
    let scenario = scenarios::all()
        .into_iter()
        .find(|s| s.name == name)
        .ok_or_else(|| format!("`{name}` is not a scenario (cargo xtask visual list)"))?;
    let frame = scenario.capture(&format!("probe/{tag}/{name}"))?;
    print!("{}", crate::ansi_cursor::encode(&frame));
    Ok(())
}
