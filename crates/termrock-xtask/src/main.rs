//! `termrock-xtask`: repository enforcement checks.
//!
//! `cargo run -p termrock-xtask -- check-ownership` enforces the FIX-008
//! consumer/author boundary (see `RULES.md` next to this crate).

use std::path::PathBuf;

use termrock_xtask::ownership::{check_workspace, find_workspace_root, render_human};

fn main() {
    let code = run(std::env::args().skip(1).collect());
    std::process::exit(code);
}

fn run(args: Vec<String>) -> i32 {
    let mut command: Option<String> = None;
    let mut root: Option<PathBuf> = None;
    let mut exceptions: Option<PathBuf> = None;
    let mut format = "human".to_string();

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--root" => {
                index += 1;
                root = args.get(index).map(PathBuf::from);
            }
            "--exceptions" => {
                index += 1;
                exceptions = args.get(index).map(PathBuf::from);
            }
            "--format" => {
                index += 1;
                if let Some(value) = args.get(index) {
                    format = value.clone();
                }
            }
            "--help" | "-h" => {
                print_usage();
                return 0;
            }
            other if command.is_none() && !other.starts_with('-') => {
                command = Some(other.to_string());
            }
            other => {
                eprintln!("unknown argument: {other}");
                print_usage();
                return 2;
            }
        }
        index += 1;
    }

    if command.as_deref() != Some("check-ownership") {
        print_usage();
        return 2;
    }

    let Ok(cwd) = std::env::current_dir() else {
        eprintln!("cannot determine the current directory");
        return 2;
    };
    let workspace = root.or_else(|| find_workspace_root(&cwd));
    let Some(workspace) = workspace else {
        eprintln!("cannot locate the workspace root (no [workspace] Cargo.toml above)");
        return 2;
    };
    let exceptions = exceptions.unwrap_or_else(|| {
        workspace
            .join("crates")
            .join("termrock-xtask")
            .join("exceptions")
            .join("v1.json")
    });

    let report = check_workspace(&workspace, &exceptions);
    match format.as_str() {
        "human" => print!("{}", render_human(&report)),
        "json" => print!("{}", render_json(&report)),
        other => {
            eprintln!("unknown --format: {other}");
            return 2;
        }
    }
    i32::from(!report.passed())
}

fn print_usage() {
    eprintln!(
        "usage: termrock-xtask check-ownership [--root DIR] [--exceptions FILE] [--format human|json]"
    );
}

/// Machine-readable report for review tooling (counts, not verdicts).
fn render_json(report: &termrock_xtask::ownership::Report) -> String {
    use std::fmt::Write as _;
    let mut out = String::from("{\"files_scanned\":");
    let _ = write!(
        out,
        "{},\"passed\":{},\"counts\":{{",
        report.files_scanned,
        report.passed()
    );
    let mut first = true;
    for ((file, rule), count) in report.counts() {
        if !first {
            out.push(',');
        }
        first = false;
        let _ = write!(out, "\"{file} {rule}\":{count}");
    }
    out.push_str("},\"failures\":[");
    first = true;
    for failure in &report.failures {
        if !first {
            out.push(',');
        }
        first = false;
        let message = failure.message.replace('\\', "\\\\").replace('"', "\\\"");
        let _ = write!(
            out,
            "{{\"kind\":\"{:?}\",\"message\":\"{message}\"}}",
            failure.kind
        );
    }
    out.push_str("],\"wrappers\":[");
    first = true;
    for wrapper in &report.wrappers {
        if !first {
            out.push(',');
        }
        first = false;
        let _ = write!(
            out,
            "{{\"member\":\"{}\",\"file\":\"{}\",\"name\":\"{}\"}}",
            wrapper.member, wrapper.file, wrapper.name
        );
    }
    out.push_str("]}");
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_xtask_binary_entry() {
        // Sanity test verifying xtask entry point compiles and links.
    }
}
