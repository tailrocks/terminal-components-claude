use std::process::Command;

const ENGINE: &str = "/Users/donbeave/Downloads/tui-test-raw-capture-20261027";
const EXPECTED_HEAD: &str = "9ff17bb7a7b2dc446ef1064cc8e4c0a8307934ff";

fn main() {
    for path in [
        format!("{ENGINE}/.git/HEAD"),
        format!("{ENGINE}/.git/refs/heads/main"),
        format!("{ENGINE}/.git/index"),
        format!("{ENGINE}/crates/tui-test"),
    ] {
        println!("cargo:rerun-if-changed={path}");
    }

    let head = git(&["rev-parse", "HEAD"]);
    assert_eq!(head, EXPECTED_HEAD, "raw capture engine pin changed");
    let status = git(&["status", "--porcelain", "--untracked-files=all"]);
    assert!(
        status.is_empty(),
        "raw capture engine worktree is not clean: {status}"
    );
}

fn git(args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(ENGINE)
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("cannot run git for frozen engine: {error}"));
    assert!(
        output.status.success(),
        "git {:?} failed for frozen engine: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .unwrap_or_else(|error| panic!("git output is not UTF-8: {error}"))
        .trim()
        .to_string()
}
