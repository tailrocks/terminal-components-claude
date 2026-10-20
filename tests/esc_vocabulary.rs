use std::path::{Path, PathBuf};

const VERBS: [&str; 9] = [
    "cancel", "close", "back", "clear", "done", "discard", "exit", "release", "restore",
];

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
fn every_esc_label_starts_with_a_verb() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = vec![];
    sources(&root, &mut files);
    let mut bad = vec![];
    let mut seen = 0;
    for file in files {
        let text = std::fs::read_to_string(&file).unwrap();
        for (line, label) in esc_labels(&text) {
            seen += 1;
            let first = label.split_whitespace().next().unwrap_or("").to_lowercase();
            if !VERBS.contains(&first.as_str()) {
                bad.push(format!("{}:{line}: Esc {label:?}", file.display()));
            }
        }
    }
    assert!(seen > 50, "the scan found the hint sources ({seen})");
    assert!(
        bad.is_empty(),
        "Esc labels without a verb:\n{}",
        bad.join("\n")
    );
}

#[test]
fn the_scan_reads_both_hint_forms() {
    let src = r#"hint("Esc", "Back to list"), ("Esc", "Outline"), hint(
        "Esc",
        "Close"), keys: &["Esc", "?"]"#;
    let labels: Vec<String> = esc_labels(src).into_iter().map(|(_, l)| l).collect();
    assert_eq!(labels, ["Back to list", "Outline", "Close"]);
}
