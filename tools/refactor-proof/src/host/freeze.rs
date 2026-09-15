//! `freeze` reconstructs the tested tree and seals operation contexts.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use super::authority::{
    load_authority, load_campaign, load_preparation, Campaign, CheckContextTemplate,
};
use super::git::GitCommand;
use super::scope::{load_verify_scope, validate_candidate};
use crate::json_util::{canonical_json_line, sha256_canonical_line};

pub enum FreezeOutcome {
    Passed,
    Rejected(&'static str),
    Failed(String),
}

pub fn run_freeze(run_dir: &Path, candidate: &Path) -> FreezeOutcome {
    let authority = match load_authority() {
        Ok(value) => value,
        Err(error) => return FreezeOutcome::Failed(error),
    };
    let campaign_dir = authority
        .campaign_path
        .parent()
        .ok_or_else(|| "invalid campaign path".to_string());
    let campaign_dir = match campaign_dir {
        Ok(value) => value,
        Err(error) => return FreezeOutcome::Failed(error),
    };
    let campaign = match load_campaign(campaign_dir, &authority) {
        Ok(value) => value,
        Err(error) => return FreezeOutcome::Failed(error),
    };
    let (task_id, parent) = match load_preparation(run_dir) {
        Ok(value) => value,
        Err(error) => return FreezeOutcome::Failed(error),
    };
    match freeze_run(&campaign, &task_id, &parent, run_dir, candidate) {
        Ok(()) => FreezeOutcome::Passed,
        Err(category) => FreezeOutcome::Rejected(category),
    }
}

fn freeze_run(
    campaign: &Campaign,
    task_id: &str,
    parent: &str,
    run_dir: &Path,
    candidate: &Path,
) -> Result<(), &'static str> {
    if parent != campaign.trusted_overlay.parent {
        return Err("integrity");
    }
    let task_spec = campaign.tasks.get(task_id).ok_or("integrity")?;
    let verify_toml = campaign
        .catalog_root
        .join(&task_spec.package)
        .join("verify.toml");
    let verify_scope = load_verify_scope(&verify_toml).map_err(|_| "integrity")?;
    validate_candidate(
        candidate,
        &campaign.trusted_overlay.scope_base,
        &campaign.trusted_overlay,
        &verify_scope,
    )?;
    let tree = GitCommand::new(candidate)
        .write_tree_from_worktree(candidate)
        .map_err(|_| "integrity")?;
    write_freeze_record(
        run_dir,
        &tree,
        parent,
        &campaign.trusted_overlay.scope_base,
    )
    .map_err(|_| "integrity")?;
    write_contexts(run_dir, campaign, task_id, &tree, task_spec).map_err(|_| "integrity")?;
    Ok(())
}

fn write_freeze_record(
    run_dir: &Path,
    tree: &str,
    parent: &str,
    scope_base: &str,
) -> Result<(), String> {
    fs::create_dir_all(run_dir).map_err(|error| error.to_string())?;
    let document = json!({
        "tree": tree,
        "parent": parent,
        "scope_base": scope_base,
    });
    fs::write(
        run_dir.join("freeze.json"),
        canonical_json_line(&document),
    )
    .map_err(|error| error.to_string())
}

fn write_contexts(
    run_dir: &Path,
    campaign: &Campaign,
    task_id: &str,
    tree: &str,
    task_spec: &super::authority::TaskSpec,
) -> Result<(), String> {
    let predecessor = task_spec
        .dependencies
        .first()
        .ok_or_else(|| "missing predecessor dependency".to_string())?;
    let trust = json!({
        "catalog_sha256": campaign.catalog_sha256,
        "harness_receipt_sha256": campaign.harness_receipt_sha256,
        "dependencies": [predecessor.receipt_sha256.clone()],
    });
    let mut members = Vec::new();
    let contexts_dir = run_dir.join("contexts");
    fs::create_dir_all(&contexts_dir).map_err(|error| error.to_string())?;
    for template in &task_spec.check_context_templates {
        let (member, child) = build_context_member(
            campaign,
            task_id,
            tree,
            template,
        )?;
        let context_path = PathBuf::from(&member["context_path"].as_str().unwrap());
        fs::write(contexts_dir.join(context_path.file_name().unwrap()), canonical_json_line(&child))
            .map_err(|error| error.to_string())?;
        members.push(member);
    }
    let index = json!({
        "schema": "tc-proof-context-index/v1",
        "run_id": campaign.run_id,
        "task_id": task_id,
        "tree": tree,
        "trust_sha256": sha256_canonical_line(&trust),
        "members": members,
    });
    fs::write(
        run_dir.join("context-index.json"),
        canonical_json_line(&index),
    )
    .map_err(|error| error.to_string())
}

fn build_context_member(
    campaign: &Campaign,
    task_id: &str,
    tree: &str,
    template: &CheckContextTemplate,
) -> Result<(Value, Value), String> {
    let context_path = format!("contexts/{}.json", template.check_id);
    let mut child = json!({
        "check_id": template.check_id,
        "schema": template.schema,
        "operation": template.operation,
        "required_ids": template.required_ids,
        "output_id": template.output_id,
        "run_id": campaign.run_id,
        "task_id": task_id,
        "tree": tree,
    });
    if let Some(lane) = &template.lane {
        child["lane"] = json!(lane);
    } else {
        child["lane"] = Value::Null;
    }
    if let Some(namespace) = &template.namespace {
        child["namespace"] = json!(namespace);
    } else {
        child["namespace"] = Value::Null;
    }
    let member = json!({
        "check_id": template.check_id,
        "context_path": context_path,
        "context_sha256": sha256_canonical_line(&child),
        "schema": template.schema,
        "operation": template.operation,
        "lane": template.lane.clone().map(Value::String).unwrap_or(Value::Null),
        "namespace": template.namespace.clone().map(Value::String).unwrap_or(Value::Null),
        "required_ids": template.required_ids,
        "output_id": template.output_id,
    });
    Ok((member, child))
}
