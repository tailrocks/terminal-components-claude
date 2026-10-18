//! Ownership enforcement check (FIX-008 boundary G1–G2).
//!
//! Scans workspace `src/` trees for raw preview painting and boundary
//! violations per `docs/verification/ownership-and-parity.md` VER-006,
//! resolves import aliases, paint wrappers, paint macros, and generated
//! files, and reconciles every finding against the versioned exceptions
//! file. The grandfathered G1 corpus may only shrink.

pub mod exceptions;
pub mod rules;
pub mod scan;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use exceptions::{Failure, FailureKind, reconcile};
use rules::{Finding, Wrapper, collect_wrappers, match_file, match_wrapper_calls};
use scan::{ScannedFile, scan_file};

/// The outcome of one check run.
#[derive(Debug, Clone)]
pub struct Report {
    /// Workspace root the check ran against.
    pub root: PathBuf,
    /// Every finding, in scan order.
    pub findings: Vec<Finding>,
    /// Every failure: unexcepted, growth, stale, or structural.
    pub failures: Vec<Failure>,
    /// Paint wrappers resolved across preview crates.
    pub wrappers: Vec<Wrapper>,
    /// Number of files scanned.
    pub files_scanned: usize,
}

impl Report {
    /// True when the tree satisfies the ownership boundary.
    pub fn passed(&self) -> bool {
        self.failures.is_empty()
    }

    /// Finding counts keyed by `(file, rule)`.
    pub fn counts(&self) -> BTreeMap<(String, String), usize> {
        let mut counts = BTreeMap::new();
        for finding in &self.findings {
            *counts
                .entry((finding.file.clone(), finding.rule.to_string()))
                .or_insert(0) += 1;
        }
        counts
    }
}

/// Run the ownership check against `root` with exceptions at
/// `exceptions_path`. Fails closed: unreadable manifests, sources, or
/// exceptions are failures, never silent skips.
pub fn check_workspace(root: &Path, exceptions_path: &Path) -> Report {
    let mut failures = Vec::new();
    let members = match workspace_members(root) {
        Ok(members) => members,
        Err(message) => {
            failures.push(Failure {
                kind: FailureKind::Unknown,
                message,
            });
            return Report {
                root: root.to_path_buf(),
                findings: Vec::new(),
                failures,
                wrappers: Vec::new(),
                files_scanned: 0,
            };
        }
    };

    let exceptions_text = match std::fs::read_to_string(exceptions_path) {
        Ok(text) => text,
        Err(err) => {
            failures.push(Failure {
                kind: FailureKind::Unknown,
                message: format!(
                    "cannot read exceptions file {}: {err}",
                    exceptions_path.display()
                ),
            });
            return Report {
                root: root.to_path_buf(),
                findings: Vec::new(),
                failures,
                wrappers: Vec::new(),
                files_scanned: 0,
            };
        }
    };
    let exceptions = match exceptions::load_exceptions(&exceptions_text) {
        Ok(parsed) => parsed,
        Err(failure) => {
            failures.push(failure);
            return Report {
                root: root.to_path_buf(),
                findings: Vec::new(),
                failures,
                wrappers: Vec::new(),
                files_scanned: 0,
            };
        }
    };

    let mut files: BTreeMap<String, ScannedFile> = BTreeMap::new();
    for member in &members {
        for source in member_sources(root, member) {
            let rel = match source.strip_prefix(root) {
                Ok(rel) => rel,
                Err(_) => continue,
            };
            let rel_string = rel.to_string_lossy().replace('\\', "/");
            match std::fs::read_to_string(&source) {
                Ok(text) => {
                    files.insert(rel_string.clone(), scan_file(&rel_string, member, &text));
                }
                Err(err) => failures.push(Failure {
                    kind: FailureKind::Unknown,
                    message: format!("cannot read {rel_string}: {err}"),
                }),
            }
        }
    }

    let preview_crates = rules::preview_crate_names(&members);
    let mut findings = Vec::new();
    for file in files.values() {
        findings.extend(match_file(file, &preview_crates));
    }
    let wrappers = collect_wrappers(&files, &findings);
    findings.extend(match_wrapper_calls(&files, &wrappers));

    let known_files: BTreeSet<String> = files.keys().cloned().collect();
    let mut counts: BTreeMap<(String, String), usize> = BTreeMap::new();
    for finding in &findings {
        *counts
            .entry((finding.file.clone(), finding.rule.to_string()))
            .or_insert(0) += 1;
    }
    let files_scanned = files.len();
    failures.extend(reconcile(&counts, &known_files, &exceptions));

    Report {
        root: root.to_path_buf(),
        findings,
        failures,
        wrappers,
        files_scanned,
    }
}

/// Workspace member paths from the root `Cargo.toml` `members` array.
///
/// This parses the manifest's member list only (comments and trailing
/// commas tolerated). Zero members is a hard error: the check fails
/// closed rather than scanning nothing.
pub fn workspace_members(root: &Path) -> Result<Vec<String>, String> {
    let manifest = root.join("Cargo.toml");
    let text = std::fs::read_to_string(&manifest)
        .map_err(|err| format!("cannot read {}: {err}", manifest.display()))?;
    let mut members = Vec::new();
    let mut in_members = false;
    for raw in text.lines() {
        let line = raw.split('#').next().unwrap_or("");
        if !in_members {
            let trimmed = line.trim_start();
            if trimmed.starts_with("members") && trimmed.contains('=') && trimmed.contains('[') {
                in_members = true;
                let after = line.split_once('[').map(|(_, rest)| rest).unwrap_or("");
                members.extend(quoted_strings(after));
                if after.contains(']') {
                    break;
                }
            }
            continue;
        }
        members.extend(quoted_strings(line));
        if line.contains(']') {
            break;
        }
    }
    if members.is_empty() {
        return Err(format!(
            "no workspace members parsed from {}",
            manifest.display()
        ));
    }
    Ok(members)
}

/// Double-quoted strings on one manifest line.
fn quoted_strings(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = line;
    while let Some(open) = rest.find('"') {
        let tail = &rest[open + 1..];
        let Some(close) = tail.find('"') else {
            break;
        };
        out.push(tail[..close].to_string());
        rest = &tail[close + 1..];
    }
    out
}

/// Every `.rs` file under `<member>/src`, recursively.
fn member_sources(root: &Path, member: &str) -> Vec<PathBuf> {
    let mut sources = Vec::new();
    let mut stack = vec![root.join(member).join("src")];
    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                sources.push(path);
            }
        }
    }
    sources.sort();
    sources
}

/// Walk up from `start` to the workspace root (the dir whose `Cargo.toml`
/// declares `[workspace]`).
pub fn find_workspace_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        let manifest = current.join("Cargo.toml");
        if let Ok(text) = std::fs::read_to_string(&manifest)
            && text.lines().any(|line| line.trim() == "[workspace]")
        {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

/// Render a human-readable report.
pub fn render_human(report: &Report) -> String {
    let mut text = String::new();
    text.push_str(&format!(
        "ownership check: {} file(s) scanned, {} finding(s), {} wrapper(s)\n",
        report.files_scanned,
        report.findings.len(),
        report.wrappers.len()
    ));
    if report.failures.is_empty() {
        text.push_str("PASS: every finding is admitted and no exception is stale\n");
    } else {
        text.push_str(&format!("FAIL: {} problem(s)\n", report.failures.len()));
        for failure in &report.failures {
            text.push_str(&format!("  [{:?}] {}\n", failure.kind, failure.message));
        }
    }
    text
}
