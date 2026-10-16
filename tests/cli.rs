//! The command-line contract of the four binaries (visual-testing plan F2):
//! `--help` prints usage with the shared options and exits 0; an unknown
//! option or an invalid value prints an error and exits non-zero, before
//! the terminal is touched.

use std::process::{Command, Output};

const BINS: [(&str, &str); 4] = [
    ("showcase", env!("CARGO_BIN_EXE_showcase")),
    ("tablepro", env!("CARGO_BIN_EXE_tablepro")),
    ("holla", env!("CARGO_BIN_EXE_holla")),
    ("jackin-preview", env!("CARGO_BIN_EXE_jackin-preview")),
];

fn run(bin: &str, args: &[&str]) -> Output {
    Command::new(bin)
        .args(args)
        .env_remove("NO_COLOR")
        .output()
        .expect("the binary starts")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn help_lists_the_shared_options_and_exits_zero() {
    for (name, bin) in BINS {
        let out = run(bin, &["--help"]);
        let stdout = text(&out.stdout);
        assert!(out.status.success(), "{name} --help exits 0:\n{stdout}");
        for needle in [
            "Usage:",
            "-c, --color <LEVEL>",
            "-m, --motion <MOTION>",
            "-f, --frame <N>",
            "-h, --help",
            "Motion:",
            "Keys:",
        ] {
            assert!(
                stdout.contains(needle),
                "{name} --help shows {needle:?}:\n{stdout}"
            );
        }
        for value in [
            "truecolor",
            "256",
            "16",
            "none",
            "full",
            "reduced",
            "paused",
        ] {
            assert!(
                stdout.contains(value),
                "{name} --help names {value:?}:\n{stdout}"
            );
        }
    }
}

#[test]
fn short_help_exits_zero_too() {
    for (name, bin) in BINS {
        let out = run(bin, &["-h"]);
        assert!(out.status.success(), "{name} -h exits 0");
        assert!(text(&out.stdout).contains("--motion"), "{name} -h");
    }
}

#[test]
fn an_unknown_option_or_value_exits_non_zero() {
    for (name, bin) in BINS {
        for args in [
            &["--bogus"][..],
            &["--color", "bogus"],
            &["--motion", "bogus"],
            &["--frame", "-3"],
            &["--frame"],
        ] {
            let out = run(bin, args);
            assert!(
                !out.status.success(),
                "{name} {args:?} exits non-zero, got {:?}",
                out.status
            );
            let stderr = text(&out.stderr);
            assert!(
                stderr.contains("error:"),
                "{name} {args:?} explains:\n{stderr}"
            );
            assert!(out.stdout.is_empty(), "{name} {args:?} draws nothing");
        }
    }
}

#[test]
fn app_specific_values_are_validated() {
    let cases: [(&str, &[&str], &str); 4] = [
        (
            env!("CARGO_BIN_EXE_showcase"),
            &["--page", "no-such-page"],
            "unknown page",
        ),
        (
            env!("CARGO_BIN_EXE_tablepro"),
            &["--connect", "not-configured"],
            "no connection named",
        ),
        (
            env!("CARGO_BIN_EXE_holla"),
            &["--scenario", "no-such"],
            "unknown scenario",
        ),
        (
            env!("CARGO_BIN_EXE_jackin-preview"),
            &["--scenario", "no-such"],
            "unknown scenario",
        ),
    ];
    for (bin, args, needle) in cases {
        let out = run(bin, args);
        assert!(!out.status.success(), "{bin} {args:?} exits non-zero");
        let stderr = text(&out.stderr);
        assert!(stderr.contains(needle), "{bin} {args:?}:\n{stderr}");
    }
}
