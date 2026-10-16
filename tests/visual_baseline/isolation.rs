//! Environment isolation (plan §B3, task T3): a capture must not see the
//! environment of whoever runs the suite. The parent here is a re-executed
//! copy of this test binary in tool mode, started with variables that
//! would change the apps' output if they leaked (`NO_COLOR`, `FORCE_COLOR`,
//! `CLICOLOR_FORCE`, `*_NO_MOTION`, `COLORTERM`, `TERM`, `LANG`, `HOME`) and a
//! canary. No variable is set in this process, so tests running beside it
//! are unaffected.

use std::collections::BTreeMap;
use std::process::Command;

use crate::support;
use crate::tool::ENV_PROBE_END;

/// What a careless developer or CI shell might export.
const LEAKS: [(&str, &str); 12] = [
    ("SHELL", "/leaked/shell"),
    ("NO_COLOR", "1"),
    ("FORCE_COLOR", "3"),
    ("CLICOLOR_FORCE", "1"),
    ("HOLLA_NO_MOTION", "1"),
    ("JACKIN_NO_MOTION", "1"),
    ("COLORTERM", "24bit"),
    ("TERM", "dumb"),
    ("LANG", "fr_FR.ISO8859-1"),
    ("TZ", "Asia/Bangkok"),
    ("HOME", "/nonexistent-leaked-home"),
    ("JUNIE_LEAK_CANARY", "leaked"),
];

/// Variables a POSIX shell sets for itself; not part of the spawn env.
const SHELL_OWN: [&str; 4] = ["PWD", "OLDPWD", "SHLVL", "_"];

fn tool(tool: &str, envs: &[(&str, &str)], extra: &[(&str, &str)]) -> String {
    let exe = std::env::current_exe().expect("test binary path");
    let out = Command::new(exe)
        .env("VISUAL_TOOL", tool)
        .envs(envs.iter().copied())
        .envs(extra.iter().copied())
        .output()
        .unwrap_or_else(|e| panic!("re-run the test binary as VISUAL_TOOL={tool}: {e}"));
    assert!(
        out.status.success(),
        "VISUAL_TOOL={tool} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("tool output is UTF-8")
}

/// The spawned child's environment is exactly the pinned set plus the
/// per-scenario `HOME` (also its cwd) and `NO_COLOR` for the `nocolor`
/// level, whatever the parent exports.
fn child_env_is_pinned() {
    let text = tool("env-probe", &LEAKS, &[]);
    let (vars, _) = text
        .split_once(ENV_PROBE_END)
        .unwrap_or_else(|| panic!("probe printed no end marker:\n{text}"));
    let seen: BTreeMap<&str, &str> = vars
        .lines()
        .filter_map(|l| l.split_once('='))
        .filter(|(k, _)| !SHELL_OWN.contains(k))
        .collect();
    let home = support::scenario_home("probe/env/200x50/nocolor");
    let home = home.to_str().expect("UTF-8 home");
    let mut expected: BTreeMap<&str, &str> = support::PINNED_ENV.into_iter().collect();
    expected.insert("HOME", home);
    expected.insert("NO_COLOR", "1");
    assert_eq!(
        seen, expected,
        "the capture child saw a different environment"
    );
    let pwd = vars
        .lines()
        .find_map(|l| l.strip_prefix("PWD="))
        .expect("the shell reports its cwd");
    assert_eq!(
        std::fs::canonicalize(pwd).ok(),
        std::fs::canonicalize(home).ok(),
        "the capture cwd is its isolated HOME"
    );
}

/// A real app capture is byte-identical with and without the leaks.
fn app_capture_ignores_parent_env() {
    // Truecolor: a leaked NO_COLOR, TERM or COLORTERM would drop colours.
    let name = "showcase/pages/overview/72x20/truecolor";
    let clean = tool(
        "capture",
        &[],
        &[("VISUAL_NAME", name), ("VISUAL_AS", "clean")],
    );
    let leaky = tool(
        "capture",
        &LEAKS,
        &[("VISUAL_NAME", name), ("VISUAL_AS", "leaky")],
    );
    assert!(
        clean.contains("\x1b[38;2;"),
        "truecolor capture has RGB colours"
    );
    assert_eq!(clean, leaky, "the parent environment changed `{name}`");
}

pub fn env_isolation() {
    child_env_is_pinned();
    app_capture_ignores_parent_env();
}
