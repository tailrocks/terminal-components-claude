use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: &str = "termrock-ownership-exceptions/v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exception {
    pub path: String,
    pub rule: String,
    pub max: usize,
    pub reason: String,
    pub review: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExceptionsFile {
    pub schema: String,
    #[serde(default)]
    pub exceptions: Vec<Exception>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    pub kind: FailureKind,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureKind {
    Unexcepted,
    Growth,
    Stale,
    Unknown,
}

impl Failure {
    fn new(kind: FailureKind, message: String) -> Self {
        Failure { kind, message }
    }
}

pub fn load_exceptions(text: &str) -> Result<ExceptionsFile, Failure> {
    let parsed: ExceptionsFile = serde_json::from_str(text).map_err(|err| {
        Failure::new(
            FailureKind::Unknown,
            format!("exceptions file does not parse: {err}"),
        )
    })?;
    if parsed.schema != SCHEMA_VERSION {
        return Err(Failure::new(
            FailureKind::Unknown,
            format!(
                "unsupported exceptions schema {:?}; this check accepts {:?}",
                parsed.schema, SCHEMA_VERSION
            ),
        ));
    }
    let mut seen = BTreeSet::new();
    for exception in &parsed.exceptions {
        if !super::rules::RULES
            .iter()
            .any(|rule| rule.id == exception.rule)
        {
            return Err(Failure::new(
                FailureKind::Unknown,
                format!(
                    "exception for {} names unknown rule {}",
                    exception.path, exception.rule
                ),
            ));
        }
        if !seen.insert((exception.path.clone(), exception.rule.clone())) {
            return Err(Failure::new(
                FailureKind::Unknown,
                format!(
                    "duplicate exception for {} rule {}",
                    exception.path, exception.rule
                ),
            ));
        }
    }
    Ok(parsed)
}

pub fn reconcile(
    counts: &BTreeMap<(String, String), usize>,
    known_files: &BTreeSet<String>,
    exceptions: &ExceptionsFile,
) -> Vec<Failure> {
    let mut failures = Vec::new();
    let admitted: BTreeMap<(&str, &str), usize> = exceptions
        .exceptions
        .iter()
        .map(|e| (e.path.as_str(), e.rule.as_str(), e.max))
        .map(|(path, rule, max)| ((path, rule), max))
        .collect();

    for ((file, rule), count) in counts {
        match admitted.get(&(file.as_str(), rule.as_str())) {
            None => failures.push(Failure::new(
                FailureKind::Unexcepted,
                format!("{file}: {count} unexcepted {rule} finding(s)"),
            )),
            Some(max) if count > max => failures.push(Failure::new(
                FailureKind::Growth,
                format!("{file}: {rule} grew to {count} findings (max {max})"),
            )),
            Some(_) => {}
        }
    }

    for exception in &exceptions.exceptions {
        if !known_files.contains(&exception.path) {
            failures.push(Failure::new(
                FailureKind::Stale,
                format!(
                    "stale exception: {} no longer exists (rule {})",
                    exception.path, exception.rule
                ),
            ));
            continue;
        }
        let count = counts
            .get(&(exception.path.clone(), exception.rule.clone()))
            .copied()
            .unwrap_or(0);
        if count == 0 {
            failures.push(Failure::new(
                FailureKind::Stale,
                format!(
                    "stale exception: {} rule {} matches nothing; remove it",
                    exception.path, exception.rule
                ),
            ));
        }
    }
    failures
}
