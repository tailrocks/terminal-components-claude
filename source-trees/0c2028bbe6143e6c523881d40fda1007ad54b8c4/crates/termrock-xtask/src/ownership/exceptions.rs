//! Versioned ownership exceptions with ratchet semantics.
//!
//! The G1 corpus (historical painters pending FIX-002…FIX-005) is admitted
//! file by file, each entry pinning the observed finding count as a
//! maximum: the corpus may shrink as fixes land, never grow. An entry
//! whose count reaches zero is stale and must be removed; an unknown
//! schema version fails the check outright.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

/// The only schema version this check accepts.
pub const SCHEMA_VERSION: &str = "termrock-ownership-exceptions/v1";

/// One ratcheted admission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exception {
    /// Workspace-relative file path.
    pub path: String,
    /// Stable rule id (`OWN-01` …).
    pub rule: String,
    /// Maximum admitted findings for this file and rule.
    pub max: usize,
    /// What the admitted corpus is and why it stands.
    pub reason: String,
    /// Review receipt or work item that admitted it.
    pub review: String,
}

/// The versioned exceptions document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExceptionsFile {
    pub schema: String,
    #[serde(default)]
    pub exceptions: Vec<Exception>,
}

/// A failure to report: over-max, stale, unexcepted, unknown rule, or a
/// structural problem with the exceptions document itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    pub kind: FailureKind,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureKind {
    /// A finding no exception admits (or admitted count exceeded).
    Unexcepted,
    /// Growth beyond the pinned ratchet maximum.
    Growth,
    /// An exception that matches nothing anymore.
    Stale,
    /// Unknown rule id or schema version.
    Unknown,
}

impl Failure {
    fn new(kind: FailureKind, message: String) -> Self {
        Failure { kind, message }
    }
}

/// Parse and validate the exceptions document.
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

/// Reconcile finding counts against exceptions.
///
/// `counts` maps `(file, rule)` to the observed finding count.
/// `known_files` is the set of scanned workspace-relative paths, used to
/// flag exceptions pointing at files that no longer exist.
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

#[cfg(test)]
mod tests {
    use super::*;

    fn counts(pairs: &[(&str, &str, usize)]) -> BTreeMap<(String, String), usize> {
        pairs
            .iter()
            .map(|(f, r, n)| ((f.to_string(), r.to_string()), *n))
            .collect()
    }

    fn files(names: &[&str]) -> BTreeSet<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    fn doc(pairs: &[(&str, &str, usize)]) -> ExceptionsFile {
        ExceptionsFile {
            schema: SCHEMA_VERSION.to_string(),
            exceptions: pairs
                .iter()
                .map(|(path, rule, max)| Exception {
                    path: path.to_string(),
                    rule: rule.to_string(),
                    max: *max,
                    reason: "test".to_string(),
                    review: "test".to_string(),
                })
                .collect(),
        }
    }

    #[test]
    fn ratchet_admits_shrinkage_and_rejects_growth() {
        let exceptions = doc(&[("a.rs", "OWN-02", 5)]);
        let known = files(&["a.rs"]);
        assert!(reconcile(&counts(&[("a.rs", "OWN-02", 5)]), &known, &exceptions).is_empty());
        assert!(reconcile(&counts(&[("a.rs", "OWN-02", 3)]), &known, &exceptions).is_empty());
        let failures = reconcile(&counts(&[("a.rs", "OWN-02", 6)]), &known, &exceptions);
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].kind, FailureKind::Growth);
    }

    #[test]
    fn unexcepted_findings_and_stale_entries_fail() {
        let exceptions = doc(&[("a.rs", "OWN-02", 5)]);
        let known = files(&["a.rs", "b.rs"]);
        let failures = reconcile(
            &counts(&[("a.rs", "OWN-02", 5), ("b.rs", "OWN-02", 1)]),
            &known,
            &exceptions,
        );
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].kind, FailureKind::Unexcepted);

        let failures = reconcile(&counts(&[]), &known, &exceptions);
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].kind, FailureKind::Stale);
    }

    #[test]
    fn unknown_schema_and_rules_fail_closed() {
        assert!(load_exceptions(r#"{"schema":"nope","exceptions":[]}"#).is_err());
        assert!(load_exceptions(r#"{"schema":"#,).is_err());
        let bad_rule = format!(
            r#"{{"schema":"{SCHEMA_VERSION}","exceptions":[{{"path":"a","rule":"OWN-99","max":1,"reason":"r","review":"v"}}]}}"#
        );
        assert!(load_exceptions(&bad_rule).is_err());
    }
}
