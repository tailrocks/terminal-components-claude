//! Execute help/error paths with pipes so terminal setup cannot hide CLI failures.

use std::io;
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

fn invoke(args: &[&str]) -> io::Result<Output> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tablepro"))
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let deadline = Instant::now() + Duration::from_secs(5);
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

#[test]
fn help_exits_successfully_without_terminal_control_bytes() -> io::Result<()> {
    for args in [&["--help"][..], &["-h"], &["--unknown", "value", "--help"]] {
        let output = invoke(args)?;
        assert_eq!(output.status.code(), Some(0));
        assert!(output.stderr.is_empty());
        let text = String::from_utf8_lossy(&output.stdout);
        assert!(text.starts_with("tablepro — TablePro's core workflow"));
        assert!(text.contains("Ctrl+O open quickly"));
        assert!(text.contains("--theme junie|paper"));
        assert!(!output.stdout.contains(&0x1b));
    }
    Ok(())
}

#[test]
fn argument_errors_use_exit_two_without_echoing_values() -> io::Result<()> {
    let secret = "synthetic-secret-sentinel";
    for args in [
        &["--color", secret][..],
        &["-c", secret],
        &["--color"],
        &["--theme", secret],
        &["--connect", secret],
    ] {
        let output = invoke(args)?;
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
        assert!(!String::from_utf8_lossy(&output.stderr).contains(secret));
        assert!(!output.stderr.contains(&0x1b));
    }
    Ok(())
}

#[test]
fn accepted_color_aliases_reach_help_in_reference_order() -> io::Result<()> {
    for flag in ["--color", "-c"] {
        for color in [
            "truecolor",
            "24bit",
            "256",
            "16",
            "none",
            "mono",
            "ansi256",
            "ansi16",
        ] {
            let output = invoke(&[flag, color, "--help"])?;
            assert_eq!(output.status.code(), Some(0), "{flag} {color}");
            assert!(output.stderr.is_empty());
        }
    }
    let output = invoke(&["--help", "--color", "invalid"])?;
    assert_eq!(output.status.code(), Some(0));
    Ok(())
}
