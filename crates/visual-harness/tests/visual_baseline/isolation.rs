use std::collections::BTreeMap;
use std::process::Command;

use crate::support;
use crate::tool::{CLIENT_PLATFORM_PROBE_END, ENV_PROBE_END};
use termrock_core::platform::ClientPlatform;

const LEAKS: [(&str, &str); 17] = [
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
    ("TERM_PROGRAM", "Apple_Terminal"),
    ("LC_TERMINAL", "iTerm2"),
    ("HOME", "/nonexistent-leaked-home"),
    ("JUNIE_TUI_CLIENT_PLATFORM", "macos"),
    ("TERMROCK_CLIENT_PLATFORM", "other"),
    ("JUNIE_TUI_GLYPHS", "ascii"),
    ("JUNIE_LEAK_CANARY", "leaked"),
];

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

fn app_capture_ignores_parent_env() {
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

fn platform_child(parent_env: &[(&str, &str)], child_platform: Option<&str>) -> String {
    let exe = std::env::current_exe().expect("test binary path");
    let mut command = Command::new(exe);
    command
        .env("VISUAL_TOOL", "client-platform-proxy")
        .env_remove("VISUAL_CHILD_CLIENT_PLATFORM")
        .envs(parent_env.iter().copied());
    if let Some(platform) = child_platform {
        command.env("VISUAL_CHILD_CLIENT_PLATFORM", platform);
    }
    let out = command
        .output()
        .unwrap_or_else(|e| panic!("launch platform proxy child: {e}"));
    assert!(
        out.status.success(),
        "platform proxy failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("platform probe output is UTF-8")
}

pub fn client_platform_override_child() {
    let pinned = platform_child(&LEAKS, None);
    assert!(
        pinned.contains("CLIENT_PLATFORM=macos"),
        "capture child must detect its pinned macOS platform:\n{pinned}"
    );
    assert!(
        pinned.contains("CLIENT_PLATFORM_OVERRIDE=macos"),
        "capture child must receive the new typed platform override, not rely on host fallback:\n{pinned}"
    );
    assert!(
        pinned.contains(CLIENT_PLATFORM_PROBE_END),
        "capture child did not complete its typed platform probe:\n{pinned}"
    );

    let other = platform_child(&LEAKS, Some("other"));
    assert!(
        other.contains("CLIENT_PLATFORM=other"),
        "explicit `other` override must defeat macOS terminal hints:\n{other}"
    );
    assert!(
        other.contains("CLIENT_PLATFORM_OVERRIDE=other"),
        "the isolated child must receive the explicit `other` override:\n{other}"
    );

    let macos = platform_child(&LEAKS, Some("macos"));
    assert!(
        macos.contains("CLIENT_PLATFORM=macos"),
        "explicit `macos` override must survive contrary terminal hints:\n{macos}"
    );
    assert!(
        macos.contains("CLIENT_PLATFORM_OVERRIDE=macos"),
        "the isolated child must receive the explicit `macos` override:\n{macos}"
    );
}

pub fn invalid_platform_override_keeps_host_fallback() {
    for value in ["", "invalid"] {
        let platform = ClientPlatform::from_env(|key| {
            (key == ClientPlatform::OVERRIDE_ENV).then(|| value.to_string())
        });
        assert_eq!(
            platform,
            ClientPlatform::build_target(),
            "invalid override {value:?} must keep live host fallback"
        );
    }
}

pub fn env_isolation() {
    child_env_is_pinned();
    app_capture_ignores_parent_env();
}
