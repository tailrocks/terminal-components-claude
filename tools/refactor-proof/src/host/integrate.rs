//! `integrate` performs CAS ref updates and appends acceptance ledger records.

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::authority::{
        Campaign, DependencySpec, TaskSpec, TaskfmtPin, TrustedOverlay, write_preparation,
    };
    use crate::json_util::canonical_json_line;
    use serde_json::json;
    use std::process::Command;

    fn fixture_campaign(repo: &Path, integration_ref: &str, ledger_root: &Path) -> Campaign {
        Campaign {
            path: Default::default(),
            repository: repo.to_path_buf(),
            integration_ref: integration_ref.into(),
            ledger_root: ledger_root.to_path_buf(),
            catalog_root: Default::default(),
            catalog_sha256: String::new(),
            harness_receipt_sha256: String::new(),
            run_id: String::new(),
            trusted_overlay: TrustedOverlay {
                scope_base: String::new(),
                parent: String::new(),
                paths: Default::default(),
            },
            taskfmt: TaskfmtPin {
                executable: Default::default(),
                sha256: String::new(),
                revision: String::new(),
            },
            tasks: [(
                "task".into(),
                TaskSpec {
                    package: "task".into(),
                    seal_products: Vec::new(),
                    dependencies: vec![DependencySpec {
                        producer: "p".into(),
                        product: "q".into(),
                        receipt_sha256: "0".repeat(64),
                    }],
                    check_context_templates: Vec::new(),
                },
            )]
            .into(),
        }
    }

    fn init_repo(root: &Path) -> String {
        Command::new("git")
            .args(["init", "-q", "--initial-branch=main"])
            .current_dir(root)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .status()
            .expect("git init");
        fs::write(root.join("README"), b"fixture\n").expect("write");
        Command::new("git")
            .args(["add", "-A"])
            .current_dir(root)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .status()
            .expect("git add");
        Command::new("git")
            .args([
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.invalid",
                "commit",
                "-q",
                "-m",
                "base",
            ])
            .current_dir(root)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .status()
            .expect("git commit");
        String::from_utf8_lossy(
            &Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(root)
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env("GIT_CONFIG_GLOBAL", "/dev/null")
                .output()
                .expect("rev-parse")
                .stdout,
        )
        .trim()
        .to_owned()
    }

    fn write_run_artifacts(run_dir: &Path, parent: &str) {
        write_preparation(run_dir, "task", parent).expect("preparation");
        fs::write(
            run_dir.join("freeze.json"),
            canonical_json_line(
                &json!({"tree": "0".repeat(40), "parent": parent, "scope_base": parent}),
            ),
        )
        .expect("freeze");
        fs::write(
            run_dir.join("verdict.json"),
            canonical_json_line(&json!({
                "status": "passed",
                "tree": "0".repeat(40),
                "parent": parent,
                "scope_base": parent,
                "task_id": "task",
            })),
        )
        .expect("verdict");
    }

    #[test]
    fn rejects_wrong_integration_ref_before_git_mutation() {
        let temp = tempfile::tempdir().expect("tempdir");
        let repo = temp.path().join("repo");
        fs::create_dir_all(&repo).expect("mkdir");
        let parent = init_repo(&repo);
        let ledger_root = temp.path().join("ledger");
        fs::create_dir_all(&ledger_root).expect("ledger");
        let campaign = fixture_campaign(&repo, "refs/heads/refactor/holla-parity", &ledger_root);
        let run_dir = temp.path().join("run");
        write_run_artifacts(&run_dir, &parent);
        assert!(matches!(
            integrate_run(
                &campaign,
                &run_dir,
                "task",
                &parent,
                "refs/heads/main",
                &parent
            ),
            Err("authority")
        ));
    }

    #[test]
    fn rejects_wrong_expected_parent() {
        let temp = tempfile::tempdir().expect("tempdir");
        let repo = temp.path().join("repo");
        fs::create_dir_all(&repo).expect("mkdir");
        let parent = init_repo(&repo);
        let ledger_root = temp.path().join("ledger");
        fs::create_dir_all(&ledger_root).expect("ledger");
        let integration_ref = "refs/heads/refactor/holla-parity";
        let campaign = fixture_campaign(&repo, integration_ref, &ledger_root);
        let run_dir = temp.path().join("run");
        write_run_artifacts(&run_dir, &parent);
        assert!(matches!(
            integrate_run(
                &campaign,
                &run_dir,
                "task",
                &parent,
                integration_ref,
                &"0".repeat(40),
            ),
            Err("parent")
        ));
    }

    #[test]
    fn rejects_stale_parent_cas() {
        let temp = tempfile::tempdir().expect("tempdir");
        let repo = temp.path().join("repo");
        fs::create_dir_all(&repo).expect("mkdir");
        let parent = init_repo(&repo);
        fs::write(repo.join("advance.txt"), b"advanced\n").expect("write");
        Command::new("git")
            .args(["add", "-A"])
            .current_dir(&repo)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .status()
            .expect("add advance");
        Command::new("git")
            .args([
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.invalid",
                "commit",
                "-q",
                "-m",
                "advance",
            ])
            .current_dir(&repo)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .status()
            .expect("commit");
        let advanced = String::from_utf8_lossy(
            &Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(&repo)
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env("GIT_CONFIG_GLOBAL", "/dev/null")
                .output()
                .expect("rev-parse")
                .stdout,
        )
        .trim()
        .to_owned();
        let integration_ref = "refs/heads/refactor/holla-parity";
        GitCommand::new(&repo)
            .update_ref(
                integration_ref,
                &parent,
                "0000000000000000000000000000000000000000",
            )
            .expect("seed ref");
        GitCommand::new(&repo)
            .update_ref(integration_ref, &advanced, &parent)
            .expect("advance ref");
        let ledger_root = temp.path().join("ledger");
        fs::create_dir_all(&ledger_root).expect("ledger");
        let campaign = fixture_campaign(&repo, integration_ref, &ledger_root);
        let run_dir = temp.path().join("run");
        write_run_artifacts(&run_dir, &parent);
        assert!(matches!(
            integrate_run(
                &campaign,
                &run_dir,
                "task",
                &parent,
                integration_ref,
                &parent
            ),
            Err("parent")
        ));
    }
}
