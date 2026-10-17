//! Query safety policy and risk classification.

use tablepro_domain::{SafeMode, Table};

use crate::ast::Statement;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tier {
    Safe,
    Write,
    Destructive,
}

pub fn tier(stmt: &Statement) -> Tier {
    match stmt {
        Statement::Select(_) => Tier::Safe,
        Statement::Explain { inner, .. } => tier(inner),
        Statement::Drop { .. }
        | Statement::Truncate { .. }
        | Statement::Alter {
            destructive: true, ..
        } => Tier::Destructive,
        _ => Tier::Write,
    }
}

pub fn is_dangerous(stmt: &Statement) -> bool {
    match stmt {
        Statement::Explain { inner, .. } => is_dangerous(inner),
        _ => {
            tier(stmt) == Tier::Destructive
                || matches!(
                    stmt,
                    Statement::Delete {
                        has_where: false,
                        ..
                    }
                )
        }
    }
}

/// Result of applying the safety policy to a parsed statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// Execute the statement immediately.
    Run,
    /// Ask first. `deliberate` = Safe Mode levels: requires deliberate confirmation.
    Confirm { deliberate: bool },
    /// Read-only connection refuses writes.
    Deny,
}

/// Apply configured safety policy to a parsed statement.
pub fn gate(level: SafeMode, stmt: &Statement) -> Decision {
    let t = tier(stmt);
    let write = t != Tier::Safe;
    if level == SafeMode::ReadOnly && write {
        return Decision::Deny;
    }
    let confirm = is_dangerous(stmt)
        || (level.requires_confirmation() && (write || level.applies_to_all_queries()));
    if !confirm {
        return Decision::Run;
    }
    let deliberate = level.requires_authentication() && (write || level.applies_to_all_queries());
    Decision::Confirm { deliberate }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Risk {
    pub tier: Tier,
    pub dangerous: bool,
    pub action: String,
    pub scope: String,
    pub summary: String,
    pub reversible: &'static str,
}

pub fn fmt_rows(n: usize) -> String {
    if n >= 1_000_000 {
        format!("{:.1} M", n as f64 / 1e6)
    } else if n >= 1_000 {
        format!("{:.1} k", n as f64 / 1e3)
    } else {
        n.to_string()
    }
}

pub fn assess(stmt: &Statement, table: Option<&Table>) -> Risk {
    let rows = table.map_or(0, |t| t.row_count);
    let t = tier(stmt);
    let dangerous = is_dangerous(stmt);
    let (action, scope, risk, reversible) = match stmt {
        Statement::Update {
            table: tb,
            has_where: false,
        } => (
            "UPDATE without WHERE".to_owned(),
            format!("every row in {tb} ({} rows)", fmt_rows(rows)),
            "Overwrites the column values of all rows at once.".to_owned(),
            "Reversible only by a compensating UPDATE or a backup",
        ),
        Statement::Delete {
            table: tb,
            has_where: false,
        } => (
            "DELETE without WHERE".to_owned(),
            format!("every row in {tb} ({} rows)", fmt_rows(rows)),
            "Removes all rows; dependent rows may go with ON DELETE CASCADE.".to_owned(),
            "Not reversible without a backup",
        ),
        Statement::Update { table: tb, .. } => (
            "UPDATE".to_owned(),
            format!("matching rows in {tb}"),
            String::new(),
            "Reversible by a compensating UPDATE",
        ),
        Statement::Delete { table: tb, .. } => (
            "DELETE".to_owned(),
            format!("matching rows in {tb}"),
            String::new(),
            "Not reversible without a backup",
        ),
        Statement::Insert { table: tb } => (
            "INSERT".to_owned(),
            format!("new rows in {tb}"),
            String::new(),
            "Reversible by deleting the inserted rows",
        ),
        Statement::Drop { kind, name } => (
            format!("DROP {kind}"),
            if kind == "DATABASE" {
                format!("the whole database {name}")
            } else {
                format!(
                    "{name}, its {} rows, indexes and constraints",
                    fmt_rows(rows)
                )
            },
            "The object and everything stored in it disappears immediately.".to_owned(),
            "Not reversible",
        ),
        Statement::Truncate { table: tb } => (
            "TRUNCATE".to_owned(),
            format!("every row in {tb} ({} rows)", fmt_rows(rows)),
            "Removes all rows without firing row triggers; audit rows are not written.".to_owned(),
            "Not reversible without a backup",
        ),
        Statement::Alter {
            table: tb,
            destructive: true,
        } => (
            "ALTER TABLE with DROP".to_owned(),
            format!("{tb} structure"),
            "Dropping columns or constraints discards data and may break dependent views."
                .to_owned(),
            "Not reversible for dropped data",
        ),
        Statement::Alter { table: tb, .. } => (
            "ALTER TABLE".to_owned(),
            format!("{tb} structure"),
            "Schema changes lock the table while they run.".to_owned(),
            "Reversible by a compensating ALTER",
        ),
        Statement::Create { kind, name } => (
            format!("CREATE {kind}"),
            name.clone(),
            String::new(),
            "Reversible by DROP",
        ),
        Statement::Explain { inner, .. } => {
            let r = assess(inner, table);
            (
                format!("EXPLAIN ANALYZE runs {}", r.action),
                r.scope,
                r.summary,
                r.reversible,
            )
        }
        Statement::Select(s) => ("SELECT".to_owned(), s.table.clone(), String::new(), ""),
        Statement::Other(v) => (v.clone(), String::new(), String::new(), ""),
    };
    Risk {
        tier: t,
        dangerous,
        action,
        scope,
        summary: risk,
        reversible,
    }
}

pub fn risk_assessment(stmt: &Statement) -> Risk {
    assess(stmt, None)
}
