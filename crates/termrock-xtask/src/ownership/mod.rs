pub mod exceptions;
pub mod module_graph;
pub mod rules;
pub mod scan;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use exceptions::{Failure, FailureKind, reconcile};
use rules::{Finding, Wrapper, collect_wrappers, match_file, match_wrapper_calls};
use scan::{ScannedFile, scan_file};

#[derive(Debug, Clone)]
pub struct Report {
    pub root: PathBuf,
    pub findings: Vec<Finding>,
    pub failures: Vec<Failure>,
    pub wrappers: Vec<Wrapper>,
    /// Number of inventoried Rust files, including files proven test-only.
    pub files_scanned: usize,
}

impl Report {
    pub fn passed(&self) -> bool {
        self.failures.is_empty()
    }

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
    let mut inventory = BTreeSet::new();
    for member in &members {
        let analysis = module_graph::analyze_member(&root.join(member));
        for source in analysis.sources {
            let rel = match source.strip_prefix(root) {
                Ok(rel) => rel,
                Err(_) => continue,
            };
            let rel_string = rel.to_string_lossy().replace('\\', "/");
            inventory.insert(rel_string.clone());
            let test_only = std::fs::canonicalize(&source)
                .ok()
                .is_some_and(|canonical| analysis.test_only.contains(&canonical));
            if test_only {
                continue;
            }
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
    let files_scanned = inventory.len();
    failures.extend(reconcile(&counts, &known_files, &exceptions));

    Report {
        root: root.to_path_buf(),
        findings,
        failures,
        wrappers,
        files_scanned,
    }
}

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
