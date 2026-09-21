//! JA-068: CLI contract (policy + real-binary spawn smoke).

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::zombie_processes
)]

use std::io;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

use jackin_adapter::{JA068_ID, JA068_PINNED_FRAME, JA068_SIZES, MOTION_SEED, ja068_cli_contract};

/// Locate the real binary next to the test executable, building it on
/// demand through the parent workspace with the shared target dir.
fn binary() -> PathBuf {
    let exe = std::env::current_exe().unwrap();
    let bin = exe
        .parent()
        .and_then(|deps| deps.parent())
        .unwrap()
        .join("jackin-preview");
    if bin.exists() {
        return bin;
    }
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
        .join("Cargo.toml");
    let mut build = Command::new("cargo");
    build
        .args([
            "build",
            "--locked",
            "--offline",
            "--bin",
            "jackin-preview",
            "--manifest-path",
        ])
        .arg(&workspace);
    if let Ok(target) = std::env::var("CARGO_TARGET_DIR") {
        build.env("CARGO_TARGET_DIR", target);
    }
    let status = build.status().unwrap();
    assert!(status.success(), "could not build jackin-preview: {status}");
    assert!(
        bin.exists(),
        "binary missing after build: {}",
        bin.display()
    );
    bin
}

fn invoke(bin: &std::path::Path, args: &[&str], extra_env: &[(&str, &str)]) -> io::Result<Output> {
    let mut command = Command::new(bin);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env_remove("JACKIN_NO_MOTION")
        .env_remove("NO_COLOR")
        .env("TERM", "xterm-256color")
        .env("COLORTERM", "truecolor");
    for (key, value) in extra_env {
        command.env(key, value);
    }
    let mut child = command.spawn()?;
    let start = Instant::now();
    loop {
        if child.try_wait()?.is_some() {
            break;
        }
        if start.elapsed() >= Duration::from_secs(5) {
            child.kill()?;
            let _ = child.wait();
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "CLI did not exit before terminal setup",
            ));
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    child.wait_with_output()
}

#[test]
fn ja068_policy_tables_and_frame() {
    let captures = ja068_cli_contract();
    assert_eq!(captures.len(), JA068_SIZES.len());
    for capture in &captures {
        assert!(capture.scenario_names);
        assert!(capture.motion_names);
        assert!(capture.default_construction);
        assert_eq!(capture.pinned_frame, JA068_PINNED_FRAME);
        assert_eq!(capture.frames.len(), 2);
        for frame in &capture.frames {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA068_ID), "{}", frame.identity);
        }
    }
}

#[test]
fn ja068_help_exits_success_without_a_terminal() {
    let bin = binary();
    for alias in ["--help", "-h"] {
        let output = invoke(&bin, &[alias], &[]).unwrap();
        assert_eq!(output.status.code(), Some(0));
        assert!(output.stderr.is_empty());
        let text = String::from_utf8_lossy(&output.stdout);
        assert!(text.contains("USAGE: jackin-preview"));
        assert!(text.contains("Everything is simulated in memory"));
        assert!(!output.stdout.contains(&0x1b));
    }
}

#[test]
fn ja068_invalid_reference_options_exit_two() {
    let bin = binary();
    for option in [
        "--scenario",
        "-s",
        "--motion",
        "-m",
        "--frame",
        "-f",
        "--color",
        "-c",
    ] {
        for args in [vec![option], vec![option, "invalid-value"]] {
            let output = invoke(&bin, &args, &[]).unwrap();
            assert_eq!(output.status.code(), Some(2), "{option}");
            assert!(output.stdout.is_empty());
            assert!(!output.stderr.is_empty());
            assert!(!output.stderr.contains(&0x1b));
        }
    }
    let sentinel = "SYNTHETIC_CREDENTIAL_NOT_FOR_DISCLOSURE_9bc3";
    for option in ["--scenario", "--motion", "--frame", "--color"] {
        let output = invoke(&bin, &[option, sentinel], &[]).unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(!String::from_utf8_lossy(&output.stderr).contains(sentinel));
        assert!(!String::from_utf8_lossy(&output.stdout).contains(sentinel));
    }
}

#[test]
fn ja068_unknown_and_precedence_and_aliases() {
    let bin = binary();
    let output = invoke(&bin, &["--unknown", "value", "--help"], &[]).unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        invoke(&bin, &["--help", "--scenario", "bad"], &[])
            .unwrap()
            .status
            .code(),
        Some(0)
    );
    assert_eq!(
        invoke(&bin, &["--scenario", "bad", "--help"], &[])
            .unwrap()
            .status
            .code(),
        Some(2)
    );
    for no_motion in [None, Some(""), Some("0"), Some("1")] {
        let env: Vec<(&str, &str)> = no_motion
            .map(|v| vec![("JACKIN_NO_MOTION", v)])
            .unwrap_or_default();
        let output = invoke(
            &bin,
            &[
                "-s",
                "first-use",
                "-m",
                "paused",
                "-f",
                "42",
                "-c",
                "mono",
                "-h",
            ],
            &env,
        )
        .unwrap();
        assert_eq!(output.status.code(), Some(0));
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn ja068_color_flag_and_no_color_equivalence() {
    let bin = binary();
    for color in ["mono", "none"] {
        let output = invoke(&bin, &["-c", color, "--help"], &[]).unwrap();
        assert_eq!(output.status.code(), Some(0));
    }
    let plain = invoke(&bin, &["--help"], &[]).unwrap();
    let noncolor = invoke(&bin, &["--help"], &[("NO_COLOR", "1")]).unwrap();
    assert_eq!(plain.stdout, noncolor.stdout);
    assert_eq!(plain.status.code(), noncolor.status.code());
    let plain_err = invoke(&bin, &["--scenario", "bad"], &[]).unwrap();
    let noncolor_err = invoke(&bin, &["--scenario", "bad"], &[("NO_COLOR", "1")]).unwrap();
    assert_eq!(plain_err.stderr, noncolor_err.stderr);
    assert_eq!(plain_err.status.code(), noncolor_err.status.code());
}
