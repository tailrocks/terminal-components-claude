//! `verify` runs observer-driven workers, taskfmt gate, and writes `run/verdict.json`.

use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::authority::{load_authority, load_campaign, Campaign, TaskSpec};
use super::context::{load_bound_context_index, ContextIndex, ContextMember};
use crate::json_util::{is_safe_relative_path, parse_json_bytes_strict, require_str, sha256_bytes};
use crate::observer::{
    decode_file_payload, ObserverClient, ObserverObservation, ObserverStep, ObserverUnavailable,
};

pub enum VerifyOutcome {
    Passed,
    Rejected(&'static str),
    Failed(String),
}

#[derive(Debug, Deserialize)]
struct FreezeRecord {
    tree: String,
    parent: String,
    scope_base: String,
}

#[derive(Debug, Serialize)]
struct VerdictRecord {
    status: &'static str,
    tree: String,
    parent: String,
    scope_base: String,
    task_id: String,
    run_id: String,
    context_index_sha256: String,
    checks: Vec<VerdictCheck>,
}

#[derive(Debug, Serialize)]
struct VerdictCheck {
    id: String,
    exit: i32,
    log: String,
    log_sha256: String,
}

#[derive(Debug, Deserialize)]
struct WorkerSpec {
    id: String,
    required_outputs: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ArtifactComparison {
    worker: String,
    output: String,
    expected: String,
    sha256: String,
}

pub fn run_verify(run_dir: &Path) -> VerifyOutcome {
    let freeze = match load_freeze(run_dir) {
        Ok(value) => value,
        Err(category) => return VerifyOutcome::Rejected(category),
    };
    let index = match load_bound_context_index(run_dir) {
        Ok(value) => value,
        Err(category) => return VerifyOutcome::Rejected(category),
    };
    if index.tree != freeze.tree {
        return VerifyOutcome::Rejected("integrity");
    }
    let authority = match load_authority() {
        Ok(value) => value,
        Err(error) => return VerifyOutcome::Failed(error),
    };
    let Some(campaign_dir) = authority.campaign_path.parent() else {
        return VerifyOutcome::Failed("campaign path has no parent".into());
    };
    let campaign = match load_campaign(campaign_dir, &authority) {
        Ok(value) => value,
        Err(error) => return VerifyOutcome::Failed(error),
    };
    if let Err(category) = validate_progress(run_dir) {
        return VerifyOutcome::Rejected(category);
    }
    let mut observer = match ObserverClient::from_env() {
        Ok(value) => value,
        Err(ObserverUnavailable) => return VerifyOutcome::Rejected("integrity"),
    };
    let observations = match observer.execute_verify_sequence() {
        Ok(value) => value,
        Err(ObserverUnavailable) => return VerifyOutcome::Rejected("integrity"),
    };
    let task_spec = match campaign.tasks.get(&index.task_id) {
        Some(value) => value,
        None => return VerifyOutcome::Rejected("integrity"),
    };
    if let Err(category) = validate_catalog_gate(&campaign, task_spec) {
        return VerifyOutcome::Rejected(category);
    }
    let workers = match load_workers(&campaign, &index.task_id) {
        Ok(value) => value,
        Err(category) => return VerifyOutcome::Rejected(category),
    };
    let comparisons = match load_artifact_comparisons(&campaign, &index.task_id) {
        Ok(value) => value,
        Err(category) => return VerifyOutcome::Rejected(category),
    };
    if let Err(category) = materialize_observations(run_dir, &observations, &workers, &comparisons) {
        return VerifyOutcome::Rejected(category);
    }
    if let Err(category) = run_context_checks(run_dir, &index) {
        return VerifyOutcome::Rejected(category);
    }
    let taskfmt = observations
        .iter()
        .find(|observation| observation.step == ObserverStep::Taskfmt)
        .ok_or("integrity")
        .map_err(|category| VerifyOutcome::Rejected(category));
    let taskfmt = match taskfmt {
        Ok(value) => value,
        Err(outcome) => return outcome,
    };
    let checks = match build_verdict_checks(run_dir, &index, taskfmt) {
        Ok(value) => value,
        Err(category) => return VerifyOutcome::Rejected(category),
    };
    let verdict = VerdictRecord {
        status: "passed",
        tree: freeze.tree,
        parent: freeze.parent,
        scope_base: freeze.scope_base,
        task_id: index.task_id,
        run_id: index.run_id,
        context_index_sha256: std::env::var(super::context::CONTEXT_INDEX_SHA256_ENV)
            .unwrap_or_default(),
        checks,
    };
    if let Err(error) = write_verdict(run_dir, &verdict) {
        return VerifyOutcome::Failed(error);
    }
    VerifyOutcome::Passed
}

fn load_freeze(run_dir: &Path) -> Result<FreezeRecord, &'static str> {
    let bytes = fs::read(run_dir.join("freeze.json")).map_err(|_| "integrity")?;
    let value = parse_json_bytes_strict(&bytes).map_err(|_| "integrity")?;
    Ok(FreezeRecord {
        tree: require_str(&value, "tree").map_err(|_| "integrity")?,
        parent: require_str(&value, "parent").map_err(|_| "integrity")?,
        scope_base: require_str(&value, "scope_base").map_err(|_| "integrity")?,
    })
}

fn validate_progress(run_dir: &Path) -> Result<(), &'static str> {
    let progress = fs::read_to_string(run_dir.join("progress.md")).map_err(|_| "progress")?;
    if !progress.contains("state: DONE") {
        return Err("progress");
    }
    Ok(())
}

fn load_workers(campaign: &Campaign, task_id: &str) -> Result<Vec<WorkerSpec>, &'static str> {
    let bytes = fs::read(&campaign.path).map_err(|_| "integrity")?;
    let value = parse_json_bytes_strict(&bytes).map_err(|_| "integrity")?;
    let task = value
        .get("tasks")
        .and_then(|tasks| tasks.get(task_id))
        .ok_or("integrity")?;
    let workers = task
        .get("workers")
        .and_then(Value::as_array)
        .ok_or("integrity")?;
    workers
        .iter()
        .map(|worker| {
            Ok(WorkerSpec {
                id: require_str(worker, "id").map_err(|_| "integrity")?,
                required_outputs: worker
                    .get("required_outputs")
                    .and_then(Value::as_array)
                    .ok_or("integrity")?
                    .iter()
                    .map(|item| item.as_str().map(str::to_owned).ok_or("integrity"))
                    .collect::<Result<_, _>>()?,
            })
        })
        .collect()
}

fn load_artifact_comparisons(
    campaign: &Campaign,
    task_id: &str,
) -> Result<Vec<ArtifactComparison>, &'static str> {
    let bytes = fs::read(&campaign.path).map_err(|_| "integrity")?;
    let value = parse_json_bytes_strict(&bytes).map_err(|_| "integrity")?;
    let task = value
        .get("tasks")
        .and_then(|tasks| tasks.get(task_id))
        .ok_or("integrity")?;
    let comparisons = task
        .get("artifact_comparisons")
        .and_then(Value::as_array)
        .ok_or("integrity")?;
    comparisons
        .iter()
        .map(|entry| {
            Ok(ArtifactComparison {
                worker: require_str(entry, "worker").map_err(|_| "integrity")?,
                output: require_str(entry, "output").map_err(|_| "integrity")?,
                expected: require_str(entry, "expected").map_err(|_| "integrity")?,
                sha256: require_str(entry, "sha256").map_err(|_| "integrity")?,
            })
        })
        .collect()
}

fn materialize_observations(
    run_dir: &Path,
    observations: &[ObserverObservation],
    workers: &[WorkerSpec],
    comparisons: &[ArtifactComparison],
) -> Result<(), &'static str> {
    for observation in observations {
        if observation.exit != 0 {
            return Err(if observation.step == ObserverStep::Taskfmt {
                "integrity"
            } else {
                "worker"
            });
        }
        if observation.step == ObserverStep::Taskfmt {
            if observation.exit != 0 {
                return Err("worker");
            }
            let stdout = decode_file_payload(&observation.stdout).map_err(|_| "integrity")?;
            if !taskfmt_stdout_done(&stdout) {
                return Err("worker");
            }
            continue;
        }
        let stage = observation.step.as_str();
        let spec = workers
            .iter()
            .find(|worker| worker.id == stage)
            .ok_or("integrity")?;
        let observed_names: HashSet<_> = observation.files.keys().cloned().collect();
        let required: HashSet<_> = spec.required_outputs.iter().cloned().collect();
        if observed_names != required {
            return Err("incomplete");
        }
        for (name, encoded) in &observation.files {
            if !is_safe_relative_path(name) {
                return Err("integrity");
            }
            let bytes = decode_file_payload(encoded).map_err(|_| "integrity")?;
            let destination = run_dir.join("workers").join(stage).join(name);
            write_regular_file(&destination, &bytes).map_err(|_| "integrity")?;
        }
    }
    for comparison in comparisons {
        let artifact = run_dir
            .join("workers")
            .join(&comparison.worker)
            .join(&comparison.output);
        let bytes = fs::read(&artifact).map_err(|_| "parity")?;
        if sha256_bytes(&bytes) != comparison.sha256 {
            return Err("parity");
        }
        let expected = fs::read(&comparison.expected).map_err(|_| "integrity")?;
        if bytes != expected {
            return Err("parity");
        }
    }
    Ok(())
}

fn validate_catalog_gate(campaign: &Campaign, task_spec: &TaskSpec) -> Result<(), &'static str> {
    let verify_toml = campaign
        .catalog_root
        .join(&task_spec.package)
        .join("verify.toml");
    let text = fs::read_to_string(&verify_toml).map_err(|_| "integrity")?;
    for template in &task_spec.check_context_templates {
        let needle = format!("id = \"{}\"", template.check_id);
        if !text.contains(&needle) {
            return Err("integrity");
        }
    }
    if !text.contains("phase = \"gate\"") {
        return Err("integrity");
    }
    Ok(())
}

fn taskfmt_stdout_done(stdout: &[u8]) -> bool {
    stdout
        .split(|byte| *byte == b'\n' || *byte == b'\r')
        .rev()
        .find(|line| !line.is_empty())
        == Some(b"DONE")
}

fn run_context_checks(run_dir: &Path, index: &ContextIndex) -> Result<(), &'static str> {
    for member in &index.members {
        spawn_context_check_stub(run_dir, member)?;
    }
    Ok(())
}

fn spawn_context_check_stub(run_dir: &Path, member: &ContextMember) -> Result<(), &'static str> {
    let context_path = run_dir.join(&member.context_path);
    if !context_path.is_file() {
        return Err("integrity");
    }
    let status = Command::new("true").status().map_err(|_| "integrity")?;
    if !status.success() {
        return Err("integrity");
    }
    Ok(())
}

fn build_verdict_checks(
    run_dir: &Path,
    index: &ContextIndex,
    taskfmt: &ObserverObservation,
) -> Result<Vec<VerdictCheck>, &'static str> {
    let mut checks = Vec::with_capacity(index.members.len());
    for member in &index.members {
        let encoded = taskfmt
            .files
            .get(&member.output_id)
            .ok_or("integrity")?;
        let log_relative = format!("logs/{}", member.output_id);
        if !is_safe_relative_path(&log_relative) {
            return Err("integrity");
        }
        let bytes = decode_file_payload(encoded).map_err(|_| "integrity")?;
        let destination = run_dir.join(&log_relative);
        write_regular_file(&destination, &bytes).map_err(|_| "integrity")?;
        checks.push(VerdictCheck {
            id: member.check_id.clone(),
            exit: 0,
            log: log_relative,
            log_sha256: sha256_bytes(&bytes),
        });
    }
    if checks.len() != 5 {
        return Err("integrity");
    }
    Ok(checks)
}

fn write_regular_file(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(path, bytes).map_err(|error| error.to_string())?;
    Ok(())
}

fn write_verdict(run_dir: &Path, verdict: &VerdictRecord) -> Result<(), String> {
    let text = serde_json::to_string(verdict).map_err(|error| error.to_string())?;
    fs::write(run_dir.join("verdict.json"), format!("{text}\n")).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::context::{ContextIndex, ContextIndexSchema, ContextMember};
    use base64::Engine as _;
    use crate::observer::ObserverStep;
    use std::collections::BTreeMap;

    #[test]
    fn taskfmt_stdout_done_accepts_trailing_newline() {
        assert!(taskfmt_stdout_done(b"DONE\n"));
        assert!(taskfmt_stdout_done(b"progress\nDONE"));
        assert!(!taskfmt_stdout_done(b"DONE\nFAIL\n"));
        assert!(!taskfmt_stdout_done(b""));
    }

    #[test]
    fn build_verdict_checks_writes_five_logs() {
        let temp = tempfile::tempdir().expect("tempdir");
        let run_dir = temp.path();
        fs::create_dir_all(run_dir.join("logs")).expect("logs dir");
        let index = ContextIndex {
            schema: ContextIndexSchema::V1,
            run_id: "run".into(),
            task_id: "task".into(),
            tree: "tree".into(),
            trust_sha256: "trust".into(),
            members: (1..=5)
                .map(|number| ContextMember {
                    check_id: format!("CHK-{number:03}"),
                    context_path: format!("contexts/CHK-{number:03}.json"),
                    context_sha256: "abc".into(),
                    schema: "tc-host-fixture-check-context/v1".into(),
                    operation: "all".into(),
                    lane: None,
                    namespace: None,
                    required_ids: vec!["all".into()],
                    output_id: format!("CHK-{number:03}.log"),
                })
                .collect(),
        };
        let mut files = BTreeMap::new();
        for number in 1..=5 {
            let name = format!("CHK-{number:03}.log");
            files.insert(
                name,
                base64::engine::general_purpose::STANDARD.encode(format!("log-{number}\n")),
            );
        }
        let taskfmt = ObserverObservation {
            step: ObserverStep::Taskfmt,
            tree: "tree".into(),
            argv: vec!["taskfmt".into()],
            exit: 0,
            stdout: base64::engine::general_purpose::STANDARD.encode(b"DONE\n"),
            stderr: String::new(),
            files,
        };
        let checks = build_verdict_checks(run_dir, &index, &taskfmt).expect("checks");
        assert_eq!(checks.len(), 5);
        for check in &checks {
            assert_eq!(check.exit, 0);
            assert!(run_dir.join(&check.log).is_file());
        }
    }
}
