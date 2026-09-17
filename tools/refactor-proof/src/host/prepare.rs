//! `prepare` validates campaign authority and initializes run progress.

use std::path::Path;
use std::process::Command;

use super::authority::{
    Authority, Campaign, load_authority, load_campaign, load_receipt, write_preparation,
};
use crate::json_util::sha256_bytes;

pub(super) enum PrepareOutcome {
    Passed,
    Rejected(&'static str),
    Failed(String),
}

pub(super) fn run_prepare(
    campaign_dir: &Path,
    task: &str,
    parent: &str,
    run_dir: &Path,
) -> PrepareOutcome {
    let authority = match load_authority() {
        Ok(value) => value,
        Err(error) => return PrepareOutcome::Failed(error),
    };
    let campaign = match load_campaign(campaign_dir, &authority) {
        Ok(value) => value,
        Err(error) => return PrepareOutcome::Failed(error),
    };
    match prepare_run(&authority, &campaign, task, parent, run_dir) {
        Ok(()) => PrepareOutcome::Passed,
        Err(category) => PrepareOutcome::Rejected(category),
    }
}

fn prepare_run(
    authority: &Authority,
    campaign: &Campaign,
    task: &str,
    parent: &str,
    run_dir: &Path,
) -> Result<(), &'static str> {
    let task_spec = campaign.tasks.get(task).ok_or("integrity")?;
    load_receipt(
        authority
            .accepted_receipts
            .get(&campaign.harness_receipt_sha256)
            .ok_or("receipt")?,
        authority,
    )
    .map_err(|_| "receipt")?;
    for dependency in &task_spec.dependencies {
        let receipt_path = authority
            .accepted_receipts
            .get(&dependency.receipt_sha256)
            .ok_or("receipt")?;
        let receipt = load_receipt(receipt_path, authority).map_err(|_| "receipt")?;
        if receipt.producer != dependency.producer || receipt.product != dependency.product {
            return Err("dependency");
        }
        if !git_is_ancestor(&campaign.repository, &receipt.source_commit, parent)? {
            return Err("ancestry");
        }
    }
    validate_taskfmt(&campaign.taskfmt).map_err(|_| "integrity")?;
    let package_dir = campaign.catalog_root.join(&task_spec.package);
    if !package_dir.is_dir() {
        return Err("integrity");
    }
    let progress_path = run_dir.join("progress.md");
    run_taskfmt_init(&campaign.taskfmt, &package_dir, &progress_path).map_err(|_| "integrity")?;
    write_preparation(run_dir, task, parent).map_err(|_| "integrity")?;
    Ok(())
}

fn validate_taskfmt(pin: &super::authority::TaskfmtPin) -> Result<(), String> {
    let executable_bytes = std::fs::read(&pin.executable).map_err(|error| error.to_string())?;
    if sha256_bytes(&executable_bytes) != pin.sha256 {
        return Err("taskfmt executable hash mismatch".into());
    }
    let version =
        run_command(&pin.executable, &["--version"]).map_err(|error| error.to_string())?;
    if !version.contains(&pin.revision) {
        return Err("taskfmt revision mismatch".into());
    }
    Ok(())
}

fn run_taskfmt_init(
    pin: &super::authority::TaskfmtPin,
    package_dir: &Path,
    progress_path: &Path,
) -> Result<(), String> {
    if let Some(parent) = progress_path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let status = Command::new(&pin.executable)
        .args(["init", "--task-dir"])
        .arg(package_dir)
        .args(["--out"])
        .arg(progress_path)
        .status()
        .map_err(|error| error.to_string())?;
    if !status.success() || !progress_path.is_file() {
        return Err("taskfmt init failed".into());
    }
    Ok(())
}

fn run_command(program: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "{} {} failed: {}",
            program.display(),
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn git_is_ancestor(repository: &Path, ancestor: &str, commit: &str) -> Result<bool, &'static str> {
    let status = Command::new("git")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .arg("-C")
        .arg(repository)
        .args(["merge-base", "--is-ancestor", ancestor, commit])
        .status()
        .map_err(|_| "ancestry")?;
    if status.success() {
        Ok(true)
    } else if status.code() == Some(1) {
        Ok(false)
    } else {
        Err("ancestry")
    }
}
