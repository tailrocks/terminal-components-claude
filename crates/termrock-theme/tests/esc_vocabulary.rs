//! Port of the PR #26 reference `tests/esc_vocabulary.rs`
//! (f4867e9b5f8a08a3bb61ce80d644198a29f51984): every `("Esc", "<label>")`
//! hint starts with an Esc verb.
//!
//! On the parity branch the hint call sites live in the app and demo crates,
//! which this workstream does not own, so the workspace-wide scan is
//! `#[ignore]`d with the two known violations recorded; unignore it when the
//! apps adopt the vocabulary. The scanner itself is pinned by the same tests
//! the reference used.

use std::path::{Path, PathBuf};

const VERBS: [&str; 9] = [
    "cancel", "close", "back", "clear", "done", "discard", "exit", "release", "restore",
];

/// Crates with known non-verb Esc labels, recorded for the owner decision
/// (frozen output forbids editing the apps in this workstream):
/// - jackin-preview-app/src/app/launch_cockpit.rs: "Keep launching" (x2)
const KNOWN_VIOLATIONS: usize = 2;

fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

fn esc_labels(text: &str) -> Vec<(usize, String)> {
    let mut found = vec![];
    let mut rest = text;
    let mut offset = 0;
    while let Some(i) = rest.find("\"Esc\",") {
        let after = &rest[i + 6..];
        let trimmed = after.trim_start();

        let opens_pair = rest[..i].trim_end().ends_with('(');
        if opens_pair
            && let Some(lit) = trimmed.strip_prefix('"')
            && let Some(end) = lit.find('"')
        {
            let line = text[..offset + i].matches('\n').count() + 1;
            found.push((line, lit[..end].to_owned()));
        }
        offset += i + 6;
        rest = &rest[i + 6..];
    }
    found
}

#[test]
fn the_scan_reads_both_hint_forms() {
    let src = r#"hint("Esc", "Back to list"), ("Esc", "Outline"), hint(
        "Esc",
        "Close"), keys: &["Esc", "?"]"#;
    let labels: Vec<String> = esc_labels(src).into_iter().map(|(_, l)| l).collect();
    assert_eq!(labels, ["Back to list", "Outline", "Close"]);
}

#[test]
fn workspace_esc_labels_start_with_verbs_up_to_the_recorded_violations() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates")
        .canonicalize()
        .unwrap();
    let mut files = vec![];
    sources(&root, &mut files);
    let mut bad = vec![];
    let mut seen = 0;
    for file in files {
        // The reference scans `src/` only; keep that scope so the scanner's
        // own fixtures and doc comments never count as call sites.
        if file.components().any(|c| c.as_os_str() == "tests") {
            continue;
        }
        let text = std::fs::read_to_string(&file).unwrap();
        for (line, label) in esc_labels(&text) {
            seen += 1;
            let first = label.split_whitespace().next().unwrap_or("").to_lowercase();
            if !VERBS.contains(&first.as_str()) {
                bad.push(format!("{}:{line}: Esc {label:?}", file.display()));
            }
        }
    }
    assert!(seen > 20, "the scan found the hint sources ({seen})");
    assert_eq!(
        bad.len(),
        KNOWN_VIOLATIONS,
        "Esc labels without a verb (fix apps or update KNOWN_VIOLATIONS):\n{}",
        bad.join("\n")
    );
}

/// The reference gate. Runs green once the recorded app violations adopt the
/// vocabulary; this workstream may not edit the app crates.
#[test]
#[ignore = "2 known violations in jackin-preview-app launch_cockpit (\"Keep launching\"); unignore after the apps adopt the Esc vocabulary"]
fn every_esc_label_starts_with_a_verb() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates")
        .canonicalize()
        .unwrap();
    let mut files = vec![];
    sources(&root, &mut files);
    let mut bad = vec![];
    for file in files {
        let text = std::fs::read_to_string(&file).unwrap();
        for (line, label) in esc_labels(&text) {
            let first = label.split_whitespace().next().unwrap_or("").to_lowercase();
            if !VERBS.contains(&first.as_str()) {
                bad.push(format!("{}:{line}: Esc {label:?}", file.display()));
            }
        }
    }
    assert!(
        bad.is_empty(),
        "Esc labels without a verb:\n{}",
        bad.join("\n")
    );
}
