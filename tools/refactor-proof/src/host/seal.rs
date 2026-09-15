//! `seal` rejects fixture tasks with empty `seal_products`.

use std::path::Path;

use super::authority::{load_authority, load_campaign, load_preparation};

pub(super) enum SealOutcome {
    Passed,
    Rejected(&'static str),
    Failed(String),
}

pub(super) fn run_seal(run_dir: &Path, product: &str) -> SealOutcome {
    let authority = match load_authority() {
        Ok(value) => value,
        Err(error) => return SealOutcome::Failed(error),
    };
    let Some(campaign_dir) = authority.campaign_path.parent() else {
        return SealOutcome::Failed("campaign path has no parent".into());
    };
    let campaign = match load_campaign(campaign_dir, &authority) {
        Ok(value) => value,
        Err(error) => return SealOutcome::Failed(error),
    };
    let (task_id, _parent) = match load_preparation(run_dir) {
        Ok(value) => value,
        Err(error) => return SealOutcome::Failed(error),
    };
    match seal_run(&campaign, &task_id, product) {
        Ok(()) => SealOutcome::Passed,
        Err(category) => SealOutcome::Rejected(category),
    }
}

fn seal_run(
    campaign: &super::authority::Campaign,
    task_id: &str,
    product: &str,
) -> Result<(), &'static str> {
    let task_spec = campaign.tasks.get(task_id).ok_or("integrity")?;
    if !task_spec.seal_products.iter().any(|allowed| allowed == product) {
        return Err("authority");
    }
    Err("integrity")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::authority::{Campaign, TaskSpec, TaskfmtPin, TrustedOverlay};

    #[test]
    fn rejects_unauthorized_product_for_fixture_task() {
        let campaign = Campaign {
            path: Default::default(),
            repository: Default::default(),
            integration_ref: "refs/heads/refactor/holla-parity".into(),
            ledger_root: Default::default(),
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
                fingerprint: String::new(),
                config: Default::default(),
                config_sha256: String::new(),
            },
            tasks: [(
                "task".into(),
                TaskSpec {
                    package: "task".into(),
                    seal_products: Vec::new(),
                    dependencies: Vec::new(),
                    check_context_templates: Vec::new(),
                },
            )]
            .into(),
        };
        assert!(matches!(
            seal_run(&campaign, "task", "oracle-showcase"),
            Err("authority")
        ));
    }
}
