//! Subprocess driver for the production `TablePro` CLI.

use std::ffi::OsStr;
use std::io;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

use junie_tui::ColorLevel;

/// Exact help bytes printed by `tablepro_app::run` on `--help` / `-h`.
pub const HELP: &str = "tablepro — TablePro's core workflow as a terminal application\n\n\
USAGE: tablepro [--color truecolor|256|16|none] [--connect NAME] [--theme junie|paper]\n\n\
Keys: Ctrl+O open quickly · Ctrl+T new query · Ctrl+R run · Ctrl+Y history · ? help · q quit\n";

/// Map a `--color` / `-c` value the same way `apps/tablepro/src/cli.rs` does.
#[must_use]
pub fn parse_color_flag(value: &str) -> Option<ColorLevel> {
    match value.to_ascii_lowercase().as_str() {
        "truecolor" | "24bit" => Some(ColorLevel::TrueColor),
        "256" | "ansi256" => Some(ColorLevel::Ansi256),
        "16" | "ansi16" => Some(ColorLevel::Ansi16),
        "none" | "mono" => Some(ColorLevel::Mono),
        _ => None,
    }
}

/// Run the production CLI binary with piped stdio so a live session cannot hide failures.
///
/// # Errors
///
/// Returns I/O errors from spawn/wait, or timed out if the child entered a session.
pub fn invoke_cli(bin: &Path, args: &[&str]) -> io::Result<Output> {
    invoke_cli_with_env(bin, args, &[])
}

/// Run the production CLI with extra child environment entries.
///
/// # Errors
///
/// Returns I/O errors from spawn/wait, or timed out if the child entered a session.
pub fn invoke_cli_with_env(
    bin: &Path,
    args: &[&str],
    env: &[(&str, &OsStr)],
) -> io::Result<Output> {
    let mut command = Command::new(bin);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env_remove("CLICOLOR_FORCE");
    for (key, value) in env {
        command.env(key, value);
    }
    let mut child = command.spawn()?;
    let deadline = Instant::now()
        .checked_add(Duration::from_secs(5))
        .ok_or_else(|| io::Error::other("five-second CLI deadline is not representable"))?;
    while child.try_wait()?.is_none() {
        if Instant::now() >= deadline {
            child.kill()?;
            let _ = child.wait();
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "CLI entered a live session",
            ));
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    child.wait_with_output()
}
