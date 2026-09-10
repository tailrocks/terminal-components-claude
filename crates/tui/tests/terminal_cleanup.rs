//! Explicit real-PTY fixture for session input-mode restoration.
#![cfg(feature = "crossterm")]
#![allow(clippy::unwrap_used, reason = "explicit terminal fixture assertions")]

use junie_tui::TerminalSession;
use std::io::{self, Read};

/// The external driver controls stdout failure and compares PTY termios.
#[test]
#[ignore = "real PTY fixture; external driver supplies terminal and output failure"]
#[expect(clippy::print_stderr, reason = "explicit PTY synchronization markers")]
fn terminal_cleanup_driver_fixture() {
    let case = std::env::var("TERMINAL_CLEANUP_CASE").unwrap();
    if case == "entry_error" {
        eprintln!("TERMINAL_ENTRY_READY");
        io::stdin().read_exact(&mut [0]).unwrap();
        let result = TerminalSession::enter();
        assert!(result.is_err());
        eprintln!("TERMINAL_ENTRY_ERROR_RESTORED");
        // stdout is deliberately broken; bypass only libtest's final report,
        // after the session error path and assertions have completed.
        std::process::exit(0);
    }
    let mut session = TerminalSession::enter().unwrap();
    session.terminal().hide_cursor().unwrap();
    eprintln!("TERMINAL_CLEANUP_READY");
    let mut input = [0];
    io::stdin().read_exact(&mut input).unwrap();
    let result = session.leave();
    if case == "leave_error" {
        assert!(result.is_err());
    } else {
        assert!(result.is_ok());
        assert!(session.leave().is_ok());
    }
    drop(session);
    eprintln!("TERMINAL_CLEANUP_RESTORED");
    if case == "leave_error" {
        std::process::exit(0);
    }
}
