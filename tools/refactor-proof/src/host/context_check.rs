use std::path::{Path, PathBuf};
use std::process::Command;

use super::context::ContextMember;

const FIXTURE_CONTEXT_SCHEMA: &str = "tc-host-fixture-check-context/v1";

const TC_PROOF_OPERATIONS: &[&str] = &[
    "preflight",
    "required",
    "oracle",
    "capture",
    "compare",
    "account-tests",
    "architecture",
    "close",
];

pub(super) fn spawn_context_check(
    run_dir: &Path,
    member: &ContextMember,
) -> Result<(), &'static str> {
    let context_path = run_dir.join(&member.context_path);
    if !context_path.is_file() {
        return Err("integrity");
    }
    if member.schema == FIXTURE_CONTEXT_SCHEMA {
        return spawn_fixture_context_check_stub();
    }
    spawn_tc_proof_context_check(member, &context_path)
}

fn spawn_fixture_context_check_stub() -> Result<(), &'static str> {
    let status = Command::new("true").status().map_err(|_| "integrity")?;
    if !status.success() {
        return Err("integrity");
    }
    Ok(())
}

fn spawn_tc_proof_context_check(
    member: &ContextMember,
    context_path: &Path,
) -> Result<(), &'static str> {
    if !TC_PROOF_OPERATIONS.contains(&member.operation.as_str()) {
        return Err("integrity");
    }
    let executable = resolve_tc_proof_executable()?;
    let argv = build_tc_proof_argv(member, context_path)?;
    let status = Command::new(&executable)
        .args(&argv)
        .status()
        .map_err(|_| "integrity")?;
    if !status.success() {
        return Err("integrity");
    }
    Ok(())
}

fn resolve_tc_proof_executable() -> Result<PathBuf, &'static str> {
    let host_exe = std::env::current_exe().map_err(|_| "integrity")?;
    let sibling = host_exe.parent().ok_or("integrity")?.join("tc-proof");
    if sibling.is_file() {
        return Ok(sibling);
    }
    Err("integrity")
}

fn build_tc_proof_argv(
    member: &ContextMember,
    context_path: &Path,
) -> Result<Vec<String>, &'static str> {
    let absolute_context = context_path
        .canonicalize()
        .map_err(|_| "integrity")?
        .to_string_lossy()
        .into_owned();
    let mut argv = vec![
        member.operation.clone(),
        "--context".into(),
        absolute_context,
    ];
    match member.operation.as_str() {
        "oracle" => {
            let namespace = member.namespace.as_deref().ok_or("integrity")?;
            argv.push("--namespace".into());
            argv.push(namespace.into());
        }
        "capture" => {
            let lane = member.lane.as_deref().ok_or("integrity")?;
            argv.push("--lane".into());
            argv.push(lane.into());
        }
        _ => {
            if member.namespace.is_some() || member.lane.is_some() {
                return Err("integrity");
            }
        }
    }
    Ok(argv)
}
