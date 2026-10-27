use std::fs;
use std::path::Path;

use serde_json::json;

use super::authority::{Campaign, load_authority, load_campaign, load_preparation};
use super::git::GitCommand;
use crate::json_util::{canonical_json_line, parse_json_bytes_strict, require_str, sha256_bytes};

pub(super) enum IntegrateOutcome {
    Passed,
    Rejected(&'static str),
    Failed(String),
}

struct FreezeRecord {
    tree: String,
    parent: String,
    scope_base: String,
}

struct VerdictRecord {
    status: String,
    tree: String,
    parent: String,
    scope_base: String,
    task_id: String,
}

pub(super) fn run_integrate(
    run_dir: &Path,
    integration_ref: &str,
    expected_parent: &str,
) -> IntegrateOutcome {
    let authority = match load_authority() {
        Ok(value) => value,
        Err(error) => return IntegrateOutcome::Failed(error),
    };
    let Some(campaign_dir) = authority.campaign_path.parent() else {
        return IntegrateOutcome::Failed("campaign path has no parent".into());
    };
    let campaign = match load_campaign(campaign_dir, &authority) {
        Ok(value) => value,
        Err(error) => return IntegrateOutcome::Failed(error),
    };
    let (task_id, prepared_parent) = match load_preparation(run_dir) {
        Ok(value) => value,
        Err(error) => return IntegrateOutcome::Failed(error),
    };
    match integrate_run(
        &campaign,
        run_dir,
        &task_id,
        &prepared_parent,
        integration_ref,
        expected_parent,
    ) {
        Ok(()) => IntegrateOutcome::Passed,
        Err(category) => IntegrateOutcome::Rejected(category),
    }
}

fn integrate_run(
    campaign: &Campaign,
    run_dir: &Path,
    task_id: &str,
    prepared_parent: &str,
    integration_ref: &str,
    expected_parent: &str,
) -> Result<(), &'static str> {
    if integration_ref != campaign.integration_ref {
        return Err("authority");
    }
    if expected_parent != prepared_parent {
        return Err("parent");
    }
    let freeze = load_freeze(run_dir)?;
    let verdict = load_verdict(run_dir)?;
    if verdict.status != "passed" {
        return Err("integrity");
    }
    if verdict.task_id != task_id
        || verdict.tree != freeze.tree
        || verdict.parent != freeze.parent
        || verdict.scope_base != freeze.scope_base
    {
        return Err("integrity");
    }
    if freeze.parent != prepared_parent {
        return Err("parent");
    }
    let git = GitCommand::new(&campaign.repository);
    let current_ref = git.rev_parse(integration_ref).map_err(|_| "parent")?;
    if current_ref != expected_parent {
        return Err("parent");
    }
    let task_spec = campaign.tasks.get(task_id).ok_or("integrity")?;
    let predecessor = task_spec.dependencies.first().ok_or("integrity")?;
    let message = format!(
        "Integrate {task_id}\n\nSigned-off-by: tc-proof-host <host@example.invalid>\nCo-authored-by: Codex <codex@openai.com>"
    );
    let alternate_objects = run_dir.join("tree-build").join(".git").join("objects");
    if alternate_objects.is_dir() {
        git.register_alternate_object_directory(&alternate_objects)
            .map_err(|_| "integrity")?;
    }
    let commit = git
        .commit_tree(&freeze.tree, &freeze.parent, &message)
        .map_err(|_| "integrity")?;
    git.update_ref(integration_ref, &commit, expected_parent)
        .map_err(|_| "parent")?;
    let verdict_bytes = fs::read(run_dir.join("verdict.json")).map_err(|_| "integrity")?;
    let acceptance = json!({
        "schema": "tc-proof-host-acceptance/v1",
        "task": task_id,
        "commit": commit,
        "tree": freeze.tree,
        "parent": freeze.parent,
        "scope_base": freeze.scope_base,
        "verdict_sha256": sha256_bytes(&verdict_bytes),
        "dependencies": [predecessor.receipt_sha256.clone()],
    });
    let ledger_path = campaign.ledger_root.join(format!("{commit}.json"));
    fs::write(&ledger_path, canonical_json_line(&acceptance)).map_err(|_| "integrity")?;
    Ok(())
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

fn load_verdict(run_dir: &Path) -> Result<VerdictRecord, &'static str> {
    let bytes = fs::read(run_dir.join("verdict.json")).map_err(|_| "integrity")?;
    let value = parse_json_bytes_strict(&bytes).map_err(|_| "integrity")?;
    Ok(VerdictRecord {
        status: require_str(&value, "status").map_err(|_| "integrity")?,
        tree: require_str(&value, "tree").map_err(|_| "integrity")?,
        parent: require_str(&value, "parent").map_err(|_| "integrity")?,
        scope_base: require_str(&value, "scope_base").map_err(|_| "integrity")?,
        task_id: require_str(&value, "task_id").map_err(|_| "integrity")?,
    })
}
