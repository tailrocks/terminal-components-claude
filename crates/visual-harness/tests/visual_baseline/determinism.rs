#![cfg(test)]
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

const INSTANT_ALLOWED: [(&str, usize, &str); 5] = [
    (
        "crates/termrock-session/src/session.rs",
        1,
        "session loop event scheduling; timed controls still use explicit fixture clocks",
    ),
    (
        "crates/tablepro-domain/src/tabs.rs",
        1,
        "public execute records visible duration; execute_with_duration bypasses the clock",
    ),
    (
        "crates/tablepro-ui/src/app.rs",
        1,
        "private QueryTiming::Live starts the live query clock; Explicit installs fixture duration before the initial SELECT",
    ),
    (
        "crates/termrock-core/src/typeahead.rs",
        1,
        "public live feed expiry; feed_at is the deterministic explicit-time entrypoint",
    ),
    (
        "crates/termrock-session/src/esc_bridge.rs",
        4,
        "live poll/split-ESC timing plus three keyboard-probe deadlines (capability acceptance/transport readiness); probe/setup clocks are injectable; captures use fixture render clocks",
    ),
];

const REVIEWED_PATTERNS: [&str; 2] = ["Instant::now", "target_os"];

const TARGET_OS_ALLOWED: [(&str, usize, &str); 1] = [(
    "crates/termrock-core/src/platform.rs",
    1,
    "capture children pin the typed platform override before detection; production retains host fallback",
)];

type ReviewedPatternLocations = BTreeMap<(String, String), Vec<usize>>;
type ScanResult = (Vec<String>, ReviewedPatternLocations, usize);

#[path = "rust_source.rs"]
mod rust_source;

#[path = "shared_dialog_action_id.rs"]
mod shared_dialog_action_id;

use rust_source::matching_patterns_checked;

pub(super) fn non_test(text: &str) -> Result<String, String> {
    rust_source::non_test(text)
}

pub(super) fn code_only(text: &str) -> String {
    String::from_utf8(
        rust_source::mask_non_code(text).unwrap_or_else(|error| panic!("lex Rust source: {error}")),
    )
    .expect("lexical mask preserves UTF-8")
}

pub(super) fn shared_dialog_action_id_projection_ranges(
    source: &str,
) -> Result<Vec<std::ops::Range<usize>>, String> {
    shared_dialog_action_id::shared_dialog_action_id_projection_ranges(source)
}

fn matching_pattern_offsets(code: &str, pattern: &str) -> Vec<usize> {
    matching_patterns_checked(code, &[pattern])
        .expect("scanner source has balanced delimiters")
        .matches
        .remove(0)
        .into_iter()
        .map(|range| range.start)
        .collect()
}

pub(super) fn matching_patterns(code: &str, patterns: &[&str]) -> Vec<Vec<std::ops::Range<usize>>> {
    matching_patterns_checked(code, patterns)
        .expect("overlay source has balanced delimiters")
        .matches
}

fn source_line_at(code: &str, offset: usize) -> usize {
    code.as_bytes()[..offset]
        .iter()
        .filter(|byte| **byte == b'\n')
        .count()
        + 1
}

fn collect_rs_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap_or_else(|e| panic!("list {}: {e}", dir.display())) {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

pub(super) fn sources(dir: &Path) -> Vec<std::path::PathBuf> {
    // Keep every Rust file below src. An external source can have multiple or
    // conditional production bindings, so a test-only declaration alone is
    // not proof that the file is absent from production.
    let mut files = Vec::new();
    collect_rs_files(dir, &mut files);
    files.sort();
    files
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

fn scan_roots(workspace: &Path, roots: &[std::path::PathBuf]) -> ScanResult {
    assert!(
        !roots.is_empty(),
        "no application or termrock source roots found under {}",
        workspace.display()
    );
    let mut files = Vec::new();
    for root in roots {
        files.extend(sources(root));
    }
    files.sort();
    let source_count = files.len();
    let mut hits = Vec::new();
    let mut reviewed_locations = BTreeMap::new();
    for file in files {
        let rel = file
            .strip_prefix(workspace)
            .expect("under root")
            .to_string_lossy()
            .replace('\\', "/");
        let text = std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("read {rel}: {e}"));
        let code = rust_source::production_code(&text)
            .unwrap_or_else(|error| panic!("scan {rel}: {error}"));
        let patterns: Vec<_> = FORBIDDEN
            .iter()
            .chain(REVIEWED_PATTERNS.iter())
            .copied()
            .collect();
        let mut scan = matching_patterns_checked(&code, &patterns)
            .unwrap_or_else(|error| panic!("tokenize {rel}: {error}"));
        let reviewed_matches = scan.matches.split_off(FORBIDDEN.len());
        for (pat, pattern_matches) in FORBIDDEN.iter().zip(scan.matches) {
            for matched in pattern_matches {
                let offset = matched.start;
                hits.push(format!("{rel}:{}: `{pat}`", source_line_at(&code, offset)));
            }
        }
        for (pat, pattern_matches) in REVIEWED_PATTERNS.iter().zip(reviewed_matches) {
            let lines: Vec<_> = if *pat == "target_os" {
                scan.identifiers
                    .iter()
                    .filter(|range| &code[(*range).clone()] == "target_os")
                    .map(|range| source_line_at(&code, range.start))
                    .collect()
            } else {
                pattern_matches
                    .into_iter()
                    .map(|matched| source_line_at(&code, matched.start))
                    .collect()
            };
            if !lines.is_empty() {
                reviewed_locations.insert((pat.to_string(), rel.clone()), lines);
            }
        }
    }
    (hits, reviewed_locations, source_count)
}

fn scan(workspace: &Path) -> ScanResult {
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
    let (hits, reviewed_locations, source_count) = scan(&workspace);
    assert!(
        source_count > 0,
        "no production Rust source files scanned under {}",
        workspace.display()
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
    let reviewed_counts: BTreeMap<_, _> = reviewed_locations
        .iter()
        .map(|(key, lines)| (key.clone(), lines.len()))
        .collect();
    if !hits.is_empty() || reviewed_counts != allowed {
        eprintln!(
            "determinism inventory: source_files={source_count}, forbidden={hits:?}, reviewed_pattern_lines={reviewed_locations:?}, expected_patterns={allowed:?}"
        );
    }
    assert!(
        hits.is_empty(),
        "wall-clock or random sources in rendering code (derive from ticks, --frame or a fixture clock instead):\n  {}",
        hits.join("\n  ")
    );
    assert_eq!(
        reviewed_counts, allowed,
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
    let filtered = non_test(src).expect("valid test fixture");
    let code = code_only(&filtered);
    for pat in FORBIDDEN {
        assert!(!code.contains(pat), "`{pat}` survived in {code:?}");
    }
    assert_eq!(code.matches("Instant::now").count(), 1);
    let path_code = code_only("use crate::widgets::brand::Lockup;");
    assert!(matching_pattern_offsets(&path_code, "rand::").is_empty());
    let path_code = code_only("let r = rand::random();");
    let matches = matching_pattern_offsets(&path_code, "rand::");
    assert_eq!(
        matches.len(),
        1,
        "code={path_code:?}, matching offsets={matches:?}"
    );
    assert_eq!(
        code_only("let x = SystemTime::now();")
            .matches("SystemTime")
            .count(),
        1,
        "code outside literals is kept"
    );
    assert_eq!(code.lines().count(), src.lines().count());
    assert!(
        non_test("#[cfg(test)] mod tests {").is_err(),
        "unclosed Rust delimiters fail closed"
    );
    assert!(
        non_test("fn f() { let s = \"unterminated; }").is_err(),
        "unterminated literals fail closed"
    );
    assert!(
        non_test(r"fn f() { let c = '\z'; }").is_err(),
        "malformed character escapes fail closed"
    );
}

pub fn scanner_masks_only_exact_test_module_items() {
    let src = concat!(
        "fn before() { let s = r###\"#[cfg(test)] mod fake {\n",
        "    SystemTime::now();\n",
        "}\"###; }\n",
        "#[cfg(test)]\n",
        "#[allow(dead_code)]\n",
        "pub(crate) mod early {\n",
        "    const BRACES: &str = r#\"} {\"#;\n",
        "    /* outer {\n",
        "       /* nested } */ remains { */\n",
        "    fn only_test() { SystemTime::now(); }\n",
        "    #[cfg(test)] mod nested { fn only_test() { UNIX_EPOCH; } }\n",
        "}\n",
        "fn between() { Instant::now(); }\n",
        "#[cfg(\n",
        "    test\n",
        ")]\n",
        "pub mod external;\n",
        "fn after() { SystemTime::now(); }\n",
        "#[cfg(any(test, feature = \"extra\"))]\n",
        "mod conditional { fn check() { SystemTime::now(); } }\n",
        "#[cfg(not(test))]\n",
        "mod not_test { fn check() { SystemTime::now(); } }\n",
        "fn lifetime<'a>(value: &'a str) { let ch = '}'; let escaped = '\\''; }\n",
    );
    let filtered = non_test(src).expect("valid Rust source");
    assert_eq!(filtered.len(), src.len(), "byte offsets remain stable");
    assert_eq!(filtered.lines().count(), src.lines().count());
    let code = code_only(&filtered);
    assert_eq!(code.lines().count(), src.lines().count());
    assert!(!code.contains("only_test"));
    assert_eq!(
        code.matches("SystemTime::now").count(),
        3,
        "production, cfg(any(...)), and cfg(not(...)) bodies remain scanned"
    );
    assert_eq!(code.matches("UNIX_EPOCH").count(), 0);
    assert_eq!(code.matches("Instant::now").count(), 1);
    assert!(code.contains("fn before"));
    assert!(code.contains("fn between"));
    assert!(code.contains("fn after"));
    assert!(code.contains("fn lifetime"));
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
        concat!(
            "#[cfg(test)]\n",
            "mod early { fn ignored() { SystemTime::now(); } }\n",
            "#[cfg(test)]\n",
            "mod historical_checks;\n",
            "fn render() { SystemTime::now(); }\n",
        ),
    )
    .expect("write source fixture");
    std::fs::write(
        source.join("historical_checks.rs"),
        "fn ignored() { SystemTime::now(); }\n",
    )
    .expect("write out-of-line test fixture");
    std::fs::write(
        source.join("production_tests_helper.rs"),
        "fn included() { UNIX_EPOCH; }\n",
    )
    .expect("write production helper fixture");
    let roots = source_roots(&root);
    let (hits, _, count) = scan_roots(&root, &roots);
    std::fs::remove_dir_all(&root).expect("remove source fixture");
    assert_eq!(
        count, 3,
        "external Rust files remain in the conservative inventory"
    );
    assert_eq!(
        hits.len(),
        3,
        "both production files and the test-only external file are visible"
    );
    assert!(
        hits.iter()
            .any(|hit| hit.contains("lib.rs:5") && hit.contains("SystemTime"))
    );
    assert!(
        hits.iter()
            .any(|hit| hit.contains("historical_checks.rs:1") && hit.contains("SystemTime"))
    );
    assert!(
        hits.iter().any(|hit| {
            hit.contains("production_tests_helper.rs:1") && hit.contains("UNIX_EPOCH")
        })
    );
}

pub fn scanner_matches_qualified_paths_across_comments_and_whitespace() {
    let root = std::env::temp_dir().join(format!(
        "visual-harness-determinism-qualified-paths-{}",
        std::process::id()
    ));
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
        concat!(
            "fn calls() {\n",
            "    std::time::Instant /* separator */\n",
            "        :: /* separator */ now();\n",
            "    rand /* separator */ :: /* separator */ random();\n",
            "    chrono\n",
            "        /* separator */ :: Local;\n",
            "    Local /* separator */ ::\n",
            "        now();\n",
            "    Utc /* separator */\n",
            "        :: now();\n",
            "    std::collections::hash_map::RandomState /* separator */\n",
            "        :: new();\n",
            "    std::process /* separator */ :: id();\n",
            "    let text = \"Instant::now rand::random chrono::Local\";\n",
            "    // Instant::now rand::random chrono::Local\n",
            "}\n",
        ),
    )
    .expect("write qualified-path source fixture");

    let roots = source_roots(&root);
    let (hits, reviewed, count) = scan_roots(&root, &roots);
    std::fs::remove_dir_all(&root).expect("remove source fixture");

    assert_eq!(count, 1);
    for expected in [
        "crates/example/src/lib.rs:4: `rand::`",
        "crates/example/src/lib.rs:5: `chrono::`",
        "crates/example/src/lib.rs:7: `Local::now`",
        "crates/example/src/lib.rs:9: `Utc::now`",
        "crates/example/src/lib.rs:11: `RandomState::new`",
        "crates/example/src/lib.rs:13: `process::id`",
    ] {
        assert!(
            hits.iter().any(|hit| hit == expected),
            "missing {expected}: {hits:?}"
        );
    }
    assert_eq!(
        reviewed.get(&(
            "Instant::now".to_string(),
            "crates/example/src/lib.rs".to_string(),
        )),
        Some(&vec![2]),
        "reviewed path counting uses the same token-aware matcher"
    );
}

pub fn scanner_retains_uncertain_external_bindings_in_real_roots() {
    let root = std::env::temp_dir().join(format!(
        "visual-harness-determinism-bindings-{}",
        std::process::id()
    ));
    let ambiguous_package = root.join("crates/ambiguous");
    let ambiguous_src = ambiguous_package.join("src");
    let explicit_package = root.join("crates/explicit");
    let explicit_src = explicit_package.join("src");
    for package in [&ambiguous_package, &explicit_package] {
        std::fs::create_dir_all(package.join("src")).expect("create package source root");
        std::fs::write(
            package.join("Cargo.toml"),
            format!(
                "[package]\nname = \"{}\"\nversion = \"0.1.0\"\n",
                package.file_name().unwrap().to_string_lossy()
            ),
        )
        .expect("write package manifest");
    }
    std::fs::create_dir_all(ambiguous_src.join("nested")).expect("create nested module root");
    let write = |path: &Path, content: &str| {
        std::fs::write(path, content)
            .unwrap_or_else(|error| panic!("write {}: {error}", path.display()));
    };
    write(
        &ambiguous_src.join("lib.rs"),
        concat!(
            "#[cfg(test)] mod shared;\n",
            "#[cfg_attr(all(), path = \"shared.rs\")] mod production_alias;\n",
            "#[cfg(test)] mod conditional;\n",
            "#[cfg_attr(feature = \"production\", path = \"conditional.rs\")] mod conditional_alias;\n",
            "#[cfg(test)] mod included;\n",
            "include!(\"included.rs\");\n",
            "#[cfg(test)] mod unresolved;\n",
            "macro_rules! declare_production { () => { mod unresolved; } }\n",
            "declare_production!();\n",
            "#[cfg(test)] mod historical_checks;\n",
            "mod nested;\n",
        ),
    );
    write(
        &ambiguous_src.join("shared.rs"),
        "pub fn shared_render() { rand::random(); }\n",
    );
    write(
        &ambiguous_src.join("conditional.rs"),
        "pub fn conditional_render() { chrono::Local::now(); }\n",
    );
    write(
        &ambiguous_src.join("included.rs"),
        "pub fn included_render() { UNIX_EPOCH; }\n",
    );
    write(
        &ambiguous_src.join("unresolved.rs"),
        "pub fn unresolved_render() { getrandom(); }\n",
    );
    write(
        &ambiguous_src.join("historical_checks.rs"),
        "fn test_only() { Instant::now(); }\n",
    );
    write(
        &ambiguous_src.join("production_tests_helper.rs"),
        "pub fn helper_render() { fastrand::usize(..); }\n",
    );
    write(
        &ambiguous_src.join("nested/mod.rs"),
        "mod child; pub fn nested_render() { Local::now(); }\n",
    );
    write(
        &ambiguous_src.join("nested/child.rs"),
        "pub fn nested_child_render() { process::id(); }\n",
    );
    write(
        &explicit_src.join("lib.rs"),
        "#[path = \"direct.rs\"] mod direct;\n",
    );
    write(
        &explicit_src.join("direct.rs"),
        "pub fn direct_render() { OffsetDateTime::now_utc(); }\n",
    );

    let roots = source_roots(&root);
    let (hits, reviewed, count) = scan_roots(&root, &roots);
    std::fs::remove_dir_all(&root).expect("remove source fixture");

    assert_eq!(count, 11, "all source files remain in inventory");
    for expected in [
        "crates/ambiguous/src/shared.rs:1: `rand::`",
        "crates/ambiguous/src/conditional.rs:1: `chrono::`",
        "crates/ambiguous/src/included.rs:1: `UNIX_EPOCH`",
        "crates/ambiguous/src/unresolved.rs:1: `getrandom`",
        "crates/ambiguous/src/production_tests_helper.rs:1: `fastrand`",
        "crates/ambiguous/src/nested/mod.rs:1: `Local::now`",
        "crates/ambiguous/src/nested/child.rs:1: `process::id`",
        "crates/explicit/src/direct.rs:1: `OffsetDateTime`",
    ] {
        assert!(
            hits.iter().any(|hit| hit == expected),
            "missing {expected}: {hits:?}"
        );
    }
    assert_eq!(
        reviewed.get(&(
            "Instant::now".to_string(),
            "crates/ambiguous/src/historical_checks.rs".to_string(),
        )),
        Some(&vec![1]),
        "the retained external test source is visible as an attributed, non-waived inventory hit"
    );
}

pub fn scanner_preserves_attribute_arguments_and_masks_real_test_items() {
    let root = std::env::temp_dir().join(format!(
        "visual-harness-determinism-attribute-arguments-{}",
        std::process::id()
    ));
    let package = root.join("crates/example");
    let source = package.join("src");
    std::fs::create_dir_all(&source).expect("create source root");
    std::fs::write(
        package.join("Cargo.toml"),
        "[package]\nname = \"example\"\nversion = \"0.1.0\"\n",
    )
    .expect("write package manifest");
    let source_text = concat!(
        "#[emit(#[cfg(test)] mod quoted { fn generated() { SystemTime::now(); } })] fn marker() {}\r\n",
        "#[emit(#[cfg(test)] mod quoted_utf8 { const LABEL: &str = \"café\"; })]\r\n",
        "#[cfg(test)] #[allow(dead_code)] pub(crate) mod actual_tests { SystemTime::now(); }\r\n",
        "#[cfg(test)] mod only_tests { fn generated() { SystemTime::now(); } } fn render_inline() { SystemTime::now(); }\r\n",
        "fn render() { SystemTime::now(); }\r\n",
        "fn later() { SystemTime::now(); }\r\n",
    );
    assert_eq!(source_text.find("SystemTime"), Some(50));
    std::fs::write(source.join("lib.rs"), source_text).expect("write attribute source fixture");

    let filtered = non_test(source_text).expect("mask only proven test modules");
    assert_eq!(
        filtered.len(),
        source_text.len(),
        "preserve original byte offsets"
    );
    assert_eq!(filtered.as_bytes().get(50..60), Some(&b"SystemTime"[..]));
    assert_eq!(
        filtered
            .as_bytes()
            .windows(2)
            .filter(|pair| *pair == b"\r\n")
            .count(),
        source_text
            .as_bytes()
            .windows(2)
            .filter(|pair| *pair == b"\r\n")
            .count(),
        "preserve CRLF line offsets"
    );
    assert!(filtered.contains("fn marker() {}"));
    assert!(!filtered.contains("actual_tests"));
    assert_eq!(code_only(&filtered).matches("SystemTime").count(), 4);

    let roots = source_roots(&root);
    let (hits, _, count) = scan_roots(&root, &roots);
    std::fs::remove_dir_all(&root).expect("remove source fixture");

    assert_eq!(count, 1);
    assert_eq!(
        hits,
        [
            "crates/example/src/lib.rs:1: `SystemTime`",
            "crates/example/src/lib.rs:4: `SystemTime`",
            "crates/example/src/lib.rs:5: `SystemTime`",
            "crates/example/src/lib.rs:6: `SystemTime`",
        ],
        "retain quoted attribute tokens and production while masking actual test items"
    );
}
