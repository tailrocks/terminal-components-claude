//! Actual executable contracts: help/errors must terminate before terminal setup.
use std::io;
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

fn invoke(args: &[&str], no_motion: Option<&str>) -> io::Result<Output> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_jackin-preview"));
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env_remove("JACKIN_NO_MOTION")
        .env_remove("NO_COLOR")
        .env("TERM", "xterm-256color")
        .env("COLORTERM", "truecolor");
    if let Some(value) = no_motion {
        command.env("JACKIN_NO_MOTION", value);
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
fn help_aliases_exit_success_without_a_terminal() -> io::Result<()> {
    for alias in ["--help", "-h"] {
        let output = invoke(&[alias], None)?;
        assert_eq!(output.status.code(), Some(0));
        assert!(output.stderr.is_empty());
        let text = String::from_utf8_lossy(&output.stdout);
        assert!(text.contains("USAGE: jackin-preview"));
        assert!(text.contains("Everything is simulated in memory"));
        assert!(
            !output.stdout.contains(&0x1b),
            "help entered terminal protocol"
        );
    }
    Ok(())
}

#[test]
fn invalid_and_missing_reference_options_exit_two_before_terminal_setup() -> io::Result<()> {
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
            let output = invoke(&args, None)?;
            assert_eq!(output.status.code(), Some(2), "{option}");
            assert!(output.stdout.is_empty());
            assert!(!output.stderr.is_empty());
            assert!(
                !output.stderr.contains(&0x1b),
                "error entered terminal protocol"
            );
        }
    }
    Ok(())
}

#[test]
fn invalid_value_diagnostics_do_not_disclose_supplied_material() -> io::Result<()> {
    let sentinel = "SYNTHETIC_CREDENTIAL_NOT_FOR_DISCLOSURE_9bc3";
    for option in ["--scenario", "--motion", "--frame", "--color"] {
        let output = invoke(&[option, sentinel], None)?;
        assert_eq!(output.status.code(), Some(2));
        assert!(!String::from_utf8_lossy(&output.stderr).contains(sentinel));
        assert!(!String::from_utf8_lossy(&output.stdout).contains(sentinel));
    }
    Ok(())
}

#[test]
fn unknown_arguments_are_ignored_and_help_obeys_reference_order() -> io::Result<()> {
    let output = invoke(&["--unknown", "value", "--help"], None)?;
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        invoke(&["--help", "--scenario", "bad"], None)?
            .status
            .code(),
        Some(0)
    );
    assert_eq!(
        invoke(&["--scenario", "bad", "--help"], None)?
            .status
            .code(),
        Some(2)
    );
    Ok(())
}

#[test]
fn accepted_aliases_and_environment_do_not_prevent_help_exit() -> io::Result<()> {
    for no_motion in [None, Some(""), Some("0"), Some("1")] {
        let output = invoke(
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
            no_motion,
        )?;
        assert_eq!(output.status.code(), Some(0));
        assert!(output.stderr.is_empty());
    }
    Ok(())
}
