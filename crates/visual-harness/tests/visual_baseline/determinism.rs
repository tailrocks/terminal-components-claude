use std::collections::BTreeMap;
use std::path::Path;

const FORBIDDEN: [&str; 13] = [
    "SystemTime",
    "UNIX_EPOCH",
    "chrono::",
    "Local::now",
    "Utc::now",
    "OffsetDateTime",
    "thread_rng",
    "rand::",
    "getrandom",
    "RandomState::new",
    "process::id",
    "new_v4",
    "fastrand",
];

const INSTANT_ALLOWED: [(&str, usize, &str); 1] = [(
    "crates/termrock-session/src/session.rs",
    1,
    "session loop scheduling; controls when another frame is drawn, not frame content",
)];

const REVIEWED_PATTERNS: [&str; 2] = ["Instant::now", "target_os"];

const TARGET_OS_ALLOWED: [(&str, usize, &str); 0] = [];

pub fn code_only(text: &str) -> String {
    let c: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < c.len() {
        match c[i] {
            '/' if c.get(i + 1) == Some(&'/') => {
                while i < c.len() && c[i] != '\n' {
                    i += 1;
                }
            }
            '/' if c.get(i + 1) == Some(&'*') => {
                i += 2;
                while i < c.len() && !(c[i] == '*' && c.get(i + 1) == Some(&'/')) {
                    i += 1;
                }
                i += 2;
            }
            'r' if matches!(c.get(i + 1), Some('"' | '#'))
                && (i == 0 || !(c[i - 1].is_alphanumeric() || c[i - 1] == '_')) =>
            {
                let mut j = i + 1;
                let mut hashes = 0;
                while c.get(j) == Some(&'#') {
                    hashes += 1;
                    j += 1;
                }
                if c.get(j) != Some(&'"') {
                    out.push(c[i]);
                    i += 1;
                    continue;
                }
                j += 1;
                loop {
                    if j >= c.len() {
                        break;
                    }
                    if c[j] == '"' && (1..=hashes).all(|k| c.get(j + k) == Some(&'#')) {
                        j += 1 + hashes;
                        break;
                    }
                    j += 1;
                }
                out.push_str("\"\"");
                i = j;
            }
            '"' => {
                i += 1;
                while i < c.len() && c[i] != '"' {
                    if c[i] == '\\' {
                        i += 1;
                    }
                    i += 1;
                }
                i += 1;
                out.push_str("\"\"");
            }
            '\'' => {
                if c.get(i + 1) == Some(&'\\') {
                    let mut j = i + 2;
                    while j < c.len() && c[j] != '\'' {
                        j += 1;
                    }
                    i = j + 1;
                    out.push_str("' '");
                } else if c.get(i + 2) == Some(&'\'') {
                    i += 3;
                    out.push_str("' '");
                } else {
                    out.push('\'');
                    i += 1;
                }
            }
            ch => {
                out.push(ch);
                i += 1;
            }
        }
    }
    out
}

fn contains_word(line: &str, pat: &str) -> bool {
    line.match_indices(pat).any(|(at, _)| {
        !line[..at]
            .chars()
            .next_back()
            .is_some_and(|c| c.is_alphanumeric() || c == '_')
    })
}

pub fn non_test(text: &str) -> &str {
    let lines: Vec<_> = text.split_inclusive('\n').collect();
    let mut offset = 0;
    for (index, line) in lines.iter().enumerate() {
        if line.trim() == "#[cfg(test)]" {
            let mut depth = 0isize;
            let mut found_item = false;
            for candidate in lines.iter().skip(index + 1) {
                let trimmed = candidate.trim();
                if depth > 0 {
                    depth += trimmed.matches('(').count() as isize;
                    depth -= trimmed.matches(')').count() as isize;
                    continue;
                }
                if trimmed.is_empty() || trimmed.starts_with("//") {
                    continue;
                }
                if trimmed.starts_with("#[") {
                    depth += trimmed.matches('(').count() as isize;
                    depth -= trimmed.matches(')').count() as isize;
                    continue;
                }
                found_item = trimmed.starts_with("mod ");
                break;
            }
            if found_item {
                return &text[..offset];
            }
        }
        offset += line.len();
    }
    text
}

fn sources(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap_or_else(|e| panic!("list {}: {e}", dir.display())) {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs")
            && !path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.contains("tests"))
        {
            out.push(path);
        }
    }
}

pub(super) fn source_roots(workspace: &Path) -> Vec<std::path::PathBuf> {
    let crates = workspace.join("crates");
    let entries =
        std::fs::read_dir(&crates).unwrap_or_else(|e| panic!("list {}: {e}", crates.display()));
    // Scan every production crate by default. Test and repository-tool crates are listed here;
    // docs/audits/2026-10-20-harness-paths.md records why each is outside render-source scope.
    let excluded = [
        "termrock-conformance",
        "termrock-e2e",
        "termrock-test-support",
        "termrock-visibility-publisher",
        "termrock-visibility-tests",
        "termrock-xtask",
        "visual-harness",
    ];
    let mut roots = Vec::new();
    for entry in entries {
        let path = entry.expect("crate entry").path();
        if !path.is_dir() || !path.join("Cargo.toml").is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if excluded.contains(&name) {
            continue;
        }
        let src = path.join("src");
        if src.is_dir() {
            roots.push(src);
        }
    }
    roots.sort();
    roots
}

fn scan_roots(
    workspace: &Path,
    roots: &[std::path::PathBuf],
) -> (Vec<String>, BTreeMap<(String, String), usize>, usize) {
    assert!(
        !roots.is_empty(),
        "no application or termrock source roots found under {}",
        workspace.display()
    );
    let mut files = Vec::new();
    for root in roots {
        sources(root, &mut files);
    }
    files.sort();
    let source_count = files.len();
    let mut hits = Vec::new();
    let mut instants = BTreeMap::new();
    for file in files {
        let rel = file
            .strip_prefix(workspace)
            .expect("under root")
            .to_string_lossy()
            .replace('\\', "/");
        let text = std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("read {rel}: {e}"));
        let code = code_only(non_test(&text));
        for (n, line) in code.lines().enumerate() {
            for pat in FORBIDDEN {
                if contains_word(line, pat) {
                    hits.push(format!("{rel}:{}: `{pat}`", n + 1));
                }
            }
        }
        for pat in REVIEWED_PATTERNS {
            let count = code.matches(pat).count();
            if count > 0 {
                instants.insert((pat.to_string(), rel.clone()), count);
            }
        }
    }
    (hits, instants, source_count)
}

fn scan(workspace: &Path) -> (Vec<String>, BTreeMap<(String, String), usize>, usize) {
    let roots = source_roots(workspace);
    scan_roots(workspace, &roots)
}

pub fn sources_are_deterministic() {
    let workspace = crate::support::manifest_path("");
    let roots = source_roots(&workspace);
    for required in [
        "crates/termrock/src",
        "crates/jackin-preview-presentation/src",
        "crates/jackin-preview-host-ui/src",
        "crates/jackin-preview-capsule-ui/src",
    ] {
        assert!(
            roots.iter().any(|root| root
                .strip_prefix(&workspace)
                .is_ok_and(|path| path == required)),
            "production source root `{required}` is missing from deterministic scan"
        );
    }
    let (hits, instants, source_count) = scan(&workspace);
    assert!(
        source_count > 0,
        "no production Rust source files scanned under {}",
        workspace.display()
    );
    assert!(
        hits.is_empty(),
        "wall-clock or random sources in rendering code (derive from ticks, --frame or a fixture clock instead):\n  {}",
        hits.join("\n  ")
    );
    let allowed: BTreeMap<(String, String), usize> = INSTANT_ALLOWED
        .iter()
        .map(|(f, n, _)| (("Instant::now".to_string(), f.to_string()), *n))
        .chain(
            TARGET_OS_ALLOWED
                .iter()
                .map(|(f, n, _)| (("target_os".to_string(), f.to_string()), *n)),
        )
        .collect();
    assert_eq!(
        instants, allowed,
        "Instant::now / target_os uses changed: review that the new use cannot change a frame's content (or differ between macOS and the ubuntu CI runner), then update determinism::INSTANT_ALLOWED / TARGET_OS_ALLOWED"
    );
}

pub fn scanner_ignores_strings_comments_and_tests() {
    let src = concat!(
        "fn a() { let s = \"rand::thread_rng()\"; } // SystemTime\n",
        "fn b() { let r = r#\"UNIX_EPOCH\"#; let c = '\"'; let l: &'static str = \"\"; }\n",
        "/* chrono:: */ fn c() { Instant::now(); }\n",
        "#[cfg(test)]\n",
        "mod tests { fn d() { SystemTime::now(); } }\n",
    );
    let code = code_only(non_test(src));
    for pat in FORBIDDEN {
        assert!(!code.contains(pat), "`{pat}` survived in {code:?}");
    }
    assert_eq!(code.matches("Instant::now").count(), 1);
    assert!(!contains_word(
        "use crate::widgets::brand::Lockup;",
        "rand::"
    ));
    assert!(contains_word("let r = rand::random();", "rand::"));
    assert_eq!(
        code_only("let x = SystemTime::now();")
            .matches("SystemTime")
            .count(),
        1,
        "code outside literals is kept"
    );
    scanner_detects_forbidden_source_injected_under_root();
}

pub fn scanner_detects_forbidden_source_injected_under_root() {
    let root =
        std::env::temp_dir().join(format!("visual-harness-determinism-{}", std::process::id()));
    let package = root.join("crates/example");
    let source = package.join("src");
    std::fs::create_dir_all(&source).expect("create source root");
    std::fs::write(
        package.join("Cargo.toml"),
        "[package]\nname = \"example\"\nversion = \"0.1.0\"\n",
    )
    .expect("write package manifest");
    std::fs::write(
        source.join("lib.rs"),
        "fn render() { SystemTime::now(); }\n",
    )
    .expect("write source fixture");
    let (hits, _, count) = scan(&root);
    std::fs::remove_dir_all(&root).expect("remove source fixture");
    assert_eq!(count, 1, "injected source was scanned");
    assert_eq!(hits.len(), 1, "forbidden source injection is detected");
    assert!(hits[0].contains("SystemTime"));
}
