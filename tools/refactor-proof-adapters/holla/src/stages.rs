//! Stage contribution and audit membership frozen from the protected TSVs.
//!
//! Sources under `data/` are byte copies of the TASK-003 trusted inputs:
//! `stage-contributions.tsv` (23 nonempty early contributions),
//! `holla-stage-audit.tsv` (135 primary-scenario rows),
//! `shell-contributions.tsv` (3 semantic contributions), and
//! `shell-frame-contributions.tsv` (1 whole-frame contribution).

use crate::tsv::Table;

const CONTRIBUTIONS_TSV: &str = include_str!("../data/stage-contributions.tsv");
const AUDIT_TSV: &str = include_str!("../data/holla-stage-audit.tsv");
const SHELL_TSV: &str = include_str!("../data/shell-contributions.tsv");
const SHELL_FRAME_TSV: &str = include_str!("../data/shell-frame-contributions.tsv");

/// One nonempty early contribution identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Contribution {
    /// Parent HO-* scenario.
    pub parent_scenario: String,
    /// Immutable contribution id.
    pub contribution_id: String,
    /// Early task owner.
    pub task_owner: String,
    /// `frame` or `state`.
    pub proof_kind: String,
    /// Minimum checkpoint count.
    pub minimum_checkpoint_count: u32,
    /// Full parent owner task.
    pub full_parent_owner: String,
}

/// Load stage contributions.
#[must_use]
pub fn contributions() -> Vec<Contribution> {
    let Some(table) = Table::parse(CONTRIBUTIONS_TSV) else {
        return Vec::new();
    };
    let mut out = Vec::with_capacity(table.len());
    let mut index = 0;
    while index < table.len() {
        out.push(Contribution {
            parent_scenario: table.get(index, "parent_scenario").unwrap_or("").to_owned(),
            contribution_id: table.get(index, "contribution_id").unwrap_or("").to_owned(),
            task_owner: table.get(index, "task_owner").unwrap_or("").to_owned(),
            proof_kind: table.get(index, "proof_kind").unwrap_or("").to_owned(),
            minimum_checkpoint_count: table
                .get(index, "minimum_checkpoint_count")
                .and_then(|value| value.parse().ok())
                .unwrap_or(1),
            full_parent_owner: table
                .get(index, "full_parent_owner")
                .unwrap_or("")
                .to_owned(),
        });
        index = index.saturating_add(1);
    }
    out
}

/// Stage audit parent count (one row per primary scenario).
#[must_use]
pub fn audit_row_count() -> usize {
    Table::parse(AUDIT_TSV).map_or(0, |table| table.len())
}

/// Shell semantic contribution count.
#[must_use]
pub fn shell_contribution_count() -> usize {
    Table::parse(SHELL_TSV).map_or(0, |table| table.len())
}

/// Shell whole-frame contribution count.
#[must_use]
pub fn shell_frame_contribution_count() -> usize {
    Table::parse(SHELL_FRAME_TSV).map_or(0, |table| table.len())
}
