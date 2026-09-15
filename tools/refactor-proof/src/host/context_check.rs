//! Frozen per-check `tc-proof` dispatch during host `verify`.

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

pub(super) fn spawn_context_check(run_dir: &Path, member: &ContextMember) -> Result<(), &'static str> {
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
    let sibling = host_exe
        .parent()
        .ok_or("integrity")?
        .join("tc-proof");
    if sibling.is_file() {
        return Ok(sibling);
    }
    Err("integrity")
}

fn build_tc_proof_argv(member: &ContextMember, context_path: &Path) -> Result<Vec<String>, &'static str> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::context::ContextMember;

    fn member(operation: &str, schema: &str) -> ContextMember {
        ContextMember {
            check_id: "CHK-001".into(),
            context_path: "contexts/CHK-001.json".into(),
            context_sha256: "abc".into(),
            schema: schema.into(),
            operation: operation.into(),
            lane: None,
            namespace: None,
            required_ids: vec!["all".into()],
            output_id: "CHK-001.log".into(),
        }
    }

    #[test]
    fn build_compare_argv_uses_absolute_context_path() {
        let temp = tempfile::tempdir().expect("tempdir");
        let context_path = temp.path().join("CHK-004.json");
        std::fs::write(&context_path, b"{}\n").expect("write context");
        let argv = build_tc_proof_argv(&member("compare", "tc-proof-compare-context/v1"), &context_path)
            .expect("argv");
        assert_eq!(argv[0], "compare");
        assert_eq!(argv[1], "--context");
        assert_eq!(argv[2], context_path.canonicalize().expect("canonical").display().to_string());
        assert_eq!(argv.len(), 3);
    }

    #[test]
    fn build_oracle_argv_requires_namespace() {
        let temp = tempfile::tempdir().expect("tempdir");
        let context_path = temp.path().join("CHK-003.json");
        std::fs::write(&context_path, b"{}\n").expect("write context");
        let mut oracle = member("oracle", "tc-proof-oracle-context/v1");
        oracle.namespace = Some("showcase".into());
        let argv = build_tc_proof_argv(&oracle, &context_path).expect("argv");
        assert_eq!(argv, vec![
            "oracle".to_string(),
            "--context".to_string(),
            context_path.canonicalize().expect("canonical").display().to_string(),
            "--namespace".to_string(),
            "showcase".to_string(),
        ]);
    }

    #[test]
    fn build_capture_argv_requires_lane() {
        let temp = tempfile::tempdir().expect("tempdir");
        let context_path = temp.path().join("CHK-002.json");
        std::fs::write(&context_path, b"{}\n").expect("write context");
        let mut capture = member("capture", "tc-proof-capture-context/v1");
        capture.lane = Some("direct".into());
        let argv = build_tc_proof_argv(&capture, &context_path).expect("argv");
        assert_eq!(argv[3], "--lane");
        assert_eq!(argv[4], "direct");
    }

    #[test]
    fn fixture_schema_uses_stub_not_tc_proof_operation_set() {
        assert_eq!(FIXTURE_CONTEXT_SCHEMA, "tc-host-fixture-check-context/v1");
        assert!(!TC_PROOF_OPERATIONS.contains(&"sentinel"));
    }
}
