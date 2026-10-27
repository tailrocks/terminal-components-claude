use std::process::Command;

const ENGINE: &str = "/Users/donbeave/Downloads/tui-test-raw-capture-r7-20261027";
const EXPECTED_HEAD: &str = "438563a86a3c5d78e6da6e12511fa089c6f15c61";

fn main() {
    let git_head = git(&["rev-parse", "--path-format=absolute", "--git-path", "HEAD"]);
    let git_ref = git(&[
        "rev-parse",
        "--path-format=absolute",
        "--git-path",
        "refs/heads/common-raw-e153",
    ]);
    let git_index = git(&["rev-parse", "--path-format=absolute", "--git-path", "index"]);
    let packed_refs = git(&[
        "rev-parse",
        "--path-format=absolute",
        "--git-path",
        "packed-refs",
    ]);
    for path in [
        git_head,
        git_ref,
        git_index,
        packed_refs,
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
