//! Workspace checks (`COMPONENT_ARCHITECTURE.md` §16.5, §21 item 34, §22.7).
//!
//! ```text
//! cargo run -p xtask -- doc-check                # §3–§17 and §21–§23 references resolve
//! cargo run -p xtask -- boundary                 # every §16.5 / §22.7 grep and metadata check
//! cargo run -p xtask -- boundary --check <name>  # one named check
//! cargo run -p xtask -- list                     # the check names
//! ```
//!
//! Exit status is non-zero on any failure; `crates/tui/tests/architecture.rs`
//! runs the named checks through this binary.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use regex::Regex;
use walkdir::WalkDir;

fn root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest.parent().map(Path::to_path_buf).unwrap_or(manifest)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("doc-check") => doc_check(),
        Some("boundary") => {
            let only = args
                .iter()
                .position(|a| a == "--check")
                .and_then(|i| args.get(i + 1).cloned());
            boundary(only.as_deref())
        }
        Some("list") => {
            for name in CHECKS.iter().map(|c| c.0) {
                println!("{name}");
            }
            Ok(())
        }
        _ => {
            eprintln!("usage: xtask <doc-check | boundary [--check NAME] | list>");
            Err("no command".to_owned())
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("xtask: {e}");
            ExitCode::FAILURE
        }
    }
}

// ───────────────────────────── source scanning ─────────────────────────────

fn rust_files(dir: &Path) -> Vec<PathBuf> {
    if !dir.exists() {
        return Vec::new();
    }
    WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|x| x == "rs"))
        .map(|e| e.path().to_path_buf())
        .collect()
}

fn read(p: &Path) -> String {
    std::fs::read_to_string(p).unwrap_or_default()
}

fn rel(p: &Path) -> String {
    p.strip_prefix(root())
        .unwrap_or(p)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Lines of a file with `#[cfg(test)]` tails removed (test modules sit at
/// the bottom of every library file).
fn non_test_lines(text: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if line.trim_start().starts_with("#[cfg(test)]") {
            break;
        }
        out.push((i + 1, line));
    }
    out
}

fn code_line(line: &str) -> &str {
    // strip line comments so a documented rule can name a forbidden pattern
    let trimmed = line.trim_start();
    if trimmed.starts_with("//") {
        return "";
    }
    match line.find("//") {
        Some(i) if !line[..i].contains('"') => &line[..i],
        _ => line,
    }
}

// ───────────────────────────── boundary checks ─────────────────────────────

type Check = (&'static str, fn() -> Result<(), String>);

const CHECKS: &[Check] = &[
    (
        "no_deprecated_or_legacy_api_usage",
        no_deprecated_or_legacy_api_usage,
    ),
    (
        "dependency_graph_is_exactly_the_declared_set",
        dependency_graph_is_exactly_the_declared_set,
    ),
    (
        "library_has_no_application_dependency",
        library_has_no_application_dependency,
    ),
    (
        "no_domain_vocabulary_in_the_library",
        no_domain_vocabulary_in_the_library,
    ),
    (
        "palette_literals_are_confined_to_theme_builtins",
        palette_literals_are_confined_to_theme_builtins,
    ),
    ("no_raw_background_parameter", no_raw_background_parameter),
    ("no_public_geometry_or_cache", no_public_geometry_or_cache),
    (
        "no_fn_pointer_extension_points",
        no_fn_pointer_extension_points,
    ),
    ("no_todo_or_unimplemented", no_todo_or_unimplemented),
    ("no_unsafe", no_unsafe),
    (
        "no_static_bound_in_component_surface",
        no_static_bound_in_component_surface,
    ),
    ("draw_takes_shared_self", draw_takes_shared_self),
    ("cache_types_are_derived_only", cache_types_are_derived_only),
    (
        "capability_has_no_unicode_field",
        capability_has_no_unicode_field,
    ),
    (
        "no_boolean_capability_parameter_on_grid",
        no_boolean_capability_parameter_on_grid,
    ),
    ("core_is_backend_free", core_is_backend_free),
    (
        "msrv_and_edition_are_unchanged",
        msrv_and_edition_are_unchanged,
    ),
];

fn boundary(only: Option<&str>) -> Result<(), String> {
    let mut failures = Vec::new();
    for (name, f) in CHECKS {
        if only.is_some_and(|o| o != *name) {
            continue;
        }
        match f() {
            Ok(()) => println!("ok   {name}"),
            Err(e) => {
                println!("FAIL {name}\n{e}");
                failures.push(*name);
            }
        }
    }
    if only.is_some() && !CHECKS.iter().any(|c| Some(c.0) == only) {
        return Err(format!("unknown check {}", only.unwrap_or("")));
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{} check(s) failed: {}",
            failures.len(),
            failures.join(", ")
        ))
    }
}

/// A forbidden pattern: regex, allowed path substrings, reason.
struct Rule {
    n: u8,
    re: &'static str,
    allowed: &'static [&'static str],
    why: &'static str,
    /// Only lines that also match this (a `pub fn` signature, …).
    only_if: Option<&'static str>,
    /// Only under this path.
    under: Option<&'static str>,
}

const RULES: &[Rule] = &[
    Rule {
        n: 1,
        re: r"Buffer::get\b|Buffer::get_mut\b",
        allowed: &[],
        why: "deprecated",
        only_if: None,
        under: None,
    },
    Rule {
        n: 2,
        re: r"enable_raw_mode|disable_raw_mode",
        allowed: &["crates/tui/src/runtime/session.rs"],
        why: "R-18",
        only_if: None,
        under: None,
    },
    Rule {
        n: 3,
        re: r"EnterAlternateScreen|LeaveAlternateScreen|Enable(Mouse|Bracketed)|Disable(Mouse|Bracketed)",
        allowed: &["crates/tui/src/runtime/session.rs"],
        why: "R-18",
        only_if: None,
        under: None,
    },
    Rule {
        n: 4,
        re: r"\\x1b\[|\\u\{1b\}\[",
        allowed: &[],
        why: "raw ANSI",
        only_if: None,
        under: None,
    },
    Rule {
        n: 5,
        re: r"KeyboardEnhancementFlags",
        allowed: &[],
        why: "R-17",
        only_if: None,
        under: None,
    },
    Rule {
        n: 6,
        re: r"for\s+\w+\s+in\s+\w+\.top\(\)\.\.|\.left\(\)\.\.",
        allowed: &["crates/tui/src/ui/paint.rs"],
        why: "R-4",
        only_if: None,
        under: None,
    },
    Rule {
        n: 7,
        re: r"Rect::new\(",
        allowed: &[
            "crates/tui/src/layout.rs",
            "crates/tui/src/ui/",
            "crates/tui-testing/",
            "/tests/",
        ],
        why: "components receive rects",
        only_if: None,
        under: Some("crates/tui/src/components/"),
    },
    Rule {
        n: 8,
        re: r"Style::default\(\)",
        allowed: &[],
        why: "R-8, one spelling",
        only_if: None,
        under: None,
    },
    Rule {
        n: 9,
        re: r"Style::new\(\)\s*\.(fg|bg)\(|style\.(fg|bg)\(|\.add_modifier\(|\.remove_modifier\(|\.underline_color\(",
        allowed: &[
            "crates/tui/src/theme/",
            "crates/tui/src/ui/paint.rs",
            "crates/tui-testing/",
        ],
        why: "R-8",
        only_if: None,
        under: None,
    },
    Rule {
        n: 10,
        re: r"style::Stylize|\.(red|green|blue|yellow|magenta|cyan|white|black|gray|on_[a-z]+)\(\)",
        allowed: &["crates/tui-testing/"],
        why: "R-8",
        only_if: None,
        under: None,
    },
    Rule {
        n: 11,
        re: r"\bMasked\b",
        allowed: &[],
        why: "R-19",
        only_if: None,
        under: None,
    },
    Rule {
        n: 12,
        re: r"unicode_width::|UnicodeWidth(Str|Char)",
        allowed: &["crates/tui/src/text/measure.rs"],
        why: "R-1",
        only_if: None,
        under: None,
    },
    Rule {
        n: 13,
        re: r"unicode_segmentation::",
        allowed: &["crates/tui/src/text/"],
        why: "one segmentation site",
        only_if: None,
        under: None,
    },
    Rule {
        n: 14,
        re: r"\bratatui::|ratatui_widgets::|ratatui_macros::",
        allowed: &[],
        why: "R-20",
        only_if: None,
        under: None,
    },
    Rule {
        n: 15,
        re: r"\bScrollbar\b|ScrollbarState|ScrollbarOrientation|ScrollDirection",
        allowed: &[],
        why: "§22.2 item 13",
        only_if: None,
        under: None,
    },
    Rule {
        n: 16,
        re: r"\b(Block|Paragraph|Padding|BorderType|Borders|Shadow|Clear)::|widgets::Clear\b|Fill::new|\bDimmed\b",
        allowed: &[],
        why: "R-20",
        only_if: None,
        under: None,
    },
    Rule {
        n: 17,
        re: r"#\[allow\(",
        allowed: &["crates/tui-testing/"],
        why: "#[expect(…, reason)] only",
        only_if: None,
        under: None,
    },
    Rule {
        n: 18,
        re: r"LazyLock|OnceLock|static mut|thread_local!",
        allowed: &["crates/tui-testing/", "xtask/"],
        why: "no process-global state",
        only_if: None,
        under: None,
    },
    Rule {
        n: 19,
        re: r"\.unwrap\(\)|\.expect\(|panic!|todo!|unimplemented!",
        allowed: &["crates/tui-testing/", "xtask/"],
        why: "goal §10",
        only_if: None,
        under: None,
    },
    Rule {
        n: 20,
        re: r"fn render\(&mut self|fn draw\(&mut self",
        allowed: &[],
        why: "G2",
        only_if: None,
        under: Some("crates/tui/src/components/"),
    },
    Rule {
        n: 21,
        re: r"\bbg:\s*(ratatui_core::style::)?Color\b",
        allowed: &[],
        why: "§16.5 no raw background parameter",
        only_if: Some(r"\bpub (const )?fn\b"),
        under: None,
    },
    Rule {
        n: 22,
        re: r"Color::Rgb\(\s*\d|Color::from_u32\(\s*0x|#[0-9a-fA-F]{6}\b",
        allowed: &[
            "crates/tui/src/theme/builtin/junie.rs",
            "crates/tui/src/theme/builtin/paper.rs",
            "/tests/fixtures/",
            "crates/tui-testing/",
        ],
        why: "R-10 colour literals only in builtins",
        only_if: None,
        under: None,
    },
    Rule {
        n: 23,
        re: r"set_cursor_position",
        allowed: &["crates/tui/src/runtime.rs", "crates/tui-testing/"],
        why: "R-7",
        only_if: None,
        under: None,
    },
    Rule {
        n: 24,
        re: r"Layout::|Constraint::|Flex::|Spacing::",
        allowed: &[],
        why: "R-13",
        only_if: None,
        under: Some("crates/tui/src/components/"),
    },
    Rule {
        n: 25,
        re: r"\.child\(|\.owns\(|\.locate\b|scrollbar::id_for|WidgetId",
        allowed: &[],
        why: "§16.5 dispatch",
        only_if: None,
        under: None,
    },
    Rule {
        n: 26,
        re: r"SmallVec|smallvec::",
        allowed: &[],
        why: "§22.4",
        only_if: None,
        under: None,
    },
];

fn scan_roots() -> Vec<PathBuf> {
    let r = root();
    let mut v = vec![r.join("crates/tui/src"), r.join("crates/tui-testing/src")];
    if let Ok(apps) = std::fs::read_dir(r.join("apps")) {
        for a in apps.filter_map(Result::ok) {
            v.push(a.path().join("src"));
        }
    }
    v
}

fn read_allow(name: &str) -> BTreeMap<String, String> {
    let p = root().join("crates/tui/tests/allow").join(name);
    let mut out = BTreeMap::new();
    for line in read(&p).lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (entry, why) = line.split_once("  #").unwrap_or((line, ""));
        out.insert(entry.trim().to_owned(), why.trim().to_owned());
    }
    out
}

fn no_deprecated_or_legacy_api_usage() -> Result<(), String> {
    let allow = read_allow("legacy_api.txt");
    for (entry, why) in &allow {
        println!("legacy_api allow-list: {entry}  # {why}");
    }
    let mut hits = Vec::new();
    for rule in RULES {
        let re = Regex::new(rule.re).map_err(|e| e.to_string())?;
        let only_if = rule
            .only_if
            .map(|r| Regex::new(r).map_err(|e| e.to_string()))
            .transpose()?;
        for dir in scan_roots() {
            for file in rust_files(&dir) {
                let path = rel(&file);
                if rule.allowed.iter().any(|a| path.contains(a)) {
                    continue;
                }
                if let Some(u) = rule.under
                    && !path.contains(u)
                {
                    continue;
                }
                let text = read(&file);
                for (ln, line) in non_test_lines(&text) {
                    let code = code_line(line);
                    if !re.is_match(code) {
                        continue;
                    }
                    if only_if.as_ref().is_some_and(|o| !o.is_match(code)) {
                        continue;
                    }
                    let key = format!("{path}:{ln}");
                    if allow.contains_key(&key) {
                        continue;
                    }
                    hits.push(format!(
                        "rule {} ({}): {key}: {}",
                        rule.n,
                        rule.why,
                        line.trim()
                    ));
                }
            }
        }
    }
    if !allow.is_empty() {
        hits.push(format!(
            "the allow-list must be empty; it has {} entries",
            allow.len()
        ));
    }
    if hits.is_empty() {
        Ok(())
    } else {
        Err(hits.join("\n"))
    }
}

fn metadata() -> Result<cargo_metadata::Metadata, String> {
    cargo_metadata::MetadataCommand::new()
        .manifest_path(root().join("Cargo.toml"))
        .exec()
        .map_err(|e| e.to_string())
}

const LIB: &str = "tui-next";
const DECLARED: [&str; 5] = [
    "ratatui-core",
    "ratatui-crossterm",
    "unicode-width",
    "unicode-segmentation",
    "bitflags",
];
const FORBIDDEN: [&str; 7] = [
    "ratatui",
    "ratatui-widgets",
    "ratatui-macros",
    "smallvec",
    "crossterm",
    "critical-section",
    "palette",
];

/// `cargo tree -p tui-next -e normal` lines: `(name, version, features)`.
/// With `prune_crossterm`, the backend's own `crossterm` subtree is left out:
/// the architecture mandates `ratatui-crossterm`, and what crossterm pulls
/// beneath itself (`parking_lot` → `smallvec`, …) is not a choice of ours.
fn lib_tree(prune_crossterm: bool) -> Result<Vec<(String, String, BTreeSet<String>)>, String> {
    let mut args = vec![
        "tree", "-p", LIB, "-e", "normal", "--prefix", "none", "-f", "{p}\t{f}",
    ];
    if prune_crossterm {
        args.extend(["--prune", "crossterm"]);
    }
    let out = Command::new("cargo")
        .args(args)
        .current_dir(root())
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(format!(
            "cargo tree failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut rows = Vec::new();
    for line in text.lines() {
        let (pkg, feats) = line.split_once('\t').unwrap_or((line, ""));
        let mut parts = pkg.split_whitespace();
        let (Some(name), Some(version)) = (parts.next(), parts.next()) else {
            continue;
        };
        let feats: BTreeSet<String> = feats
            .split(',')
            .filter(|f| !f.is_empty())
            .map(str::to_owned)
            .collect();
        rows.push((
            name.to_owned(),
            version.trim_start_matches('v').to_owned(),
            feats,
        ));
    }
    Ok(rows)
}

fn dependency_graph_is_exactly_the_declared_set() -> Result<(), String> {
    let md = metadata()?;
    let lib = md
        .packages
        .iter()
        .find(|p| p.name.as_str() == LIB)
        .ok_or("no tui-next package")?;
    let mut errors = Vec::new();
    // (1) direct normal deps
    let direct: BTreeSet<String> = lib
        .dependencies
        .iter()
        .filter(|d| d.kind == cargo_metadata::DependencyKind::Normal)
        .map(|d| d.name.clone())
        .collect();
    let want: BTreeSet<String> = DECLARED.iter().map(|s| (*s).to_owned()).collect();
    if direct != want {
        errors.push(format!("direct normal deps {direct:?} != {want:?}"));
    }
    // (2) the normal closure, resolved for this package alone (features are
    // unified per selected package, never across the legacy root package);
    // crossterm's own subtree is pruned, and crossterm must be reachable
    // only through ratatui-crossterm
    let tree = lib_tree(true)?;
    let closure: BTreeSet<String> = tree.iter().map(|(n, _, _)| n.clone()).collect();
    let full = lib_tree(false)?;
    if !full.iter().any(|(n, _, _)| n == "crossterm") {
        errors
            .push("crossterm is not in the closure at all (ratatui-crossterm missing?)".to_owned());
    }
    for f in FORBIDDEN {
        if f == "crossterm" {
            if direct.contains("crossterm") {
                errors.push("direct crossterm dependency".to_owned());
            }
            continue;
        }
        if closure.contains(f) {
            errors.push(format!("{f} is in the normal closure"));
        }
    }
    // (3) apps
    for p in &md.packages {
        if !["showcase", "tablepro", "jackin-preview"].contains(&p.name.as_str()) {
            continue;
        }
        let d: BTreeSet<String> = p
            .dependencies
            .iter()
            .filter(|d| d.kind == cargo_metadata::DependencyKind::Normal)
            .map(|d| d.name.clone())
            .collect();
        if d.len() != 1 || !(d.contains("junie-tui") || d.contains(LIB)) {
            errors.push(format!("{}: direct normal deps {d:?}", p.name));
        }
    }
    // (4) single versions inside the closure
    for name in ["unicode-width", "unicode-segmentation", "bitflags"] {
        let versions: BTreeSet<&str> = tree
            .iter()
            .filter(|(n, _, _)| n == name)
            .map(|(_, v, _)| v.as_str())
            .collect();
        if versions.len() > 1 {
            errors.push(format!(
                "{name} resolves to {versions:?} inside tui-next's closure"
            ));
        }
    }
    // (5) ratatui-core features
    if let Some((_, _, feats)) = tree.iter().find(|(n, _, _)| n == "ratatui-core") {
        // `default` is ratatui-core's empty default set and carries nothing
        let feats: BTreeSet<String> = feats.iter().filter(|f| *f != "default").cloned().collect();
        let want: BTreeSet<String> = ["std", "underline-color"]
            .iter()
            .map(|s| (*s).to_owned())
            .collect();
        if feats != want {
            errors.push(format!("ratatui-core features {feats:?} != {want:?}"));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

fn library_has_no_application_dependency() -> Result<(), String> {
    let md = metadata()?;
    let lib = md
        .packages
        .iter()
        .find(|p| p.name.as_str() == LIB)
        .ok_or("no tui-next package")?;
    let bad: Vec<String> = lib
        .dependencies
        .iter()
        .filter(|d| {
            ["showcase", "tablepro", "jackin-preview", "junie-tui"].contains(&d.name.as_str())
        })
        .map(|d| d.name.clone())
        .collect();
    if bad.is_empty() {
        Ok(())
    } else {
        Err(format!("library depends on {bad:?}"))
    }
}

fn grep_check(
    dirs: &[PathBuf],
    re: &str,
    allowed: &[&str],
    allow_file: Option<&str>,
    why: &str,
) -> Result<(), String> {
    let re = Regex::new(re).map_err(|e| e.to_string())?;
    let allow = allow_file.map(read_allow).unwrap_or_default();
    let mut hits = Vec::new();
    for dir in dirs {
        for file in rust_files(dir) {
            let path = rel(&file);
            if allowed.iter().any(|a| path.contains(a)) {
                continue;
            }
            for (ln, line) in non_test_lines(&read(&file)) {
                if re.is_match(code_line(line)) && !allow.contains_key(&format!("{path}:{ln}")) {
                    hits.push(format!("{path}:{ln}: {}", line.trim()));
                }
            }
        }
    }
    if hits.is_empty() {
        Ok(())
    } else {
        Err(format!("{why}\n{}", hits.join("\n")))
    }
}

fn no_domain_vocabulary_in_the_library() -> Result<(), String> {
    grep_check(
        &[root().join("crates/tui/src")],
        r"(?i)\b(sql|schema|primary key|nullable|foreign|references|not null|tablepro|jackin|workspace|instance|daemon|capsule|construct|catalog)\b",
        &[],
        Some("domain.txt"),
        "domain vocabulary in the library (DOM §7)",
    )
}

fn palette_literals_are_confined_to_theme_builtins() -> Result<(), String> {
    grep_check(
        &[root().join("crates/tui/src")],
        r"Color::Rgb\(\s*\d|Color::from_u32\(\s*0x|#[0-9a-fA-F]{6}\b",
        &["theme/builtin/junie.rs", "theme/builtin/paper.rs"],
        None,
        "colour literals outside theme/builtin (R-10)",
    )
}

fn no_raw_background_parameter() -> Result<(), String> {
    let re = Regex::new(r"\bbg:\s*(ratatui_core::style::)?Color\b").map_err(|e| e.to_string())?;
    let sig = Regex::new(r"\bpub (const )?fn\b").map_err(|e| e.to_string())?;
    let mut hits = Vec::new();
    for file in rust_files(&root().join("crates/tui/src")) {
        let path = rel(&file);
        for (ln, line) in non_test_lines(&read(&file)) {
            let code = code_line(line);
            if sig.is_match(code) && re.is_match(code) {
                hits.push(format!("{path}:{ln}: {}", line.trim()));
            }
        }
    }
    if hits.is_empty() {
        Ok(())
    } else {
        Err(hits.join("\n"))
    }
}

fn no_public_geometry_or_cache() -> Result<(), String> {
    grep_check(
        &[root().join("crates/tui/src/components")],
        r"\bpub (area|areas|anchor|scroll)\b|\bpub \w+_rects\b",
        &[],
        None,
        "public geometry on a component (S1)",
    )
}

fn no_fn_pointer_extension_points() -> Result<(), String> {
    grep_check(
        &[root().join("crates/tui/src")],
        r": fn\(|Option<fn\(|type \w+ = fn\(",
        &[],
        None,
        "fn-pointer extension point (API §3.12)",
    )
}

fn no_todo_or_unimplemented() -> Result<(), String> {
    let mut dirs = vec![root().join("crates")];
    if root().join("apps").exists() {
        dirs.push(root().join("apps"));
    }
    let re = Regex::new(r"todo!|unimplemented!|\bTODO\b|\bFIXME\b").map_err(|e| e.to_string())?;
    let mut hits = Vec::new();
    for dir in dirs {
        for file in rust_files(&dir) {
            let path = rel(&file);
            if path.contains("/target/") {
                continue;
            }
            for (i, line) in read(&file).lines().enumerate() {
                if re.is_match(line) && !line.contains("no_todo_or_unimplemented") {
                    hits.push(format!("{path}:{}: {}", i + 1, line.trim()));
                }
            }
        }
    }
    if hits.is_empty() {
        Ok(())
    } else {
        Err(hits.join("\n"))
    }
}

fn no_unsafe() -> Result<(), String> {
    let lib = read(&root().join("crates/tui/src/lib.rs"));
    if !lib.contains("#![forbid(unsafe_code)]") {
        return Err("crates/tui/src/lib.rs lacks #![forbid(unsafe_code)]".to_owned());
    }
    let testing = read(&root().join("crates/tui-testing/src/lib.rs"));
    if !testing.contains("#![deny(unsafe_code)]") {
        return Err("crates/tui-testing/src/lib.rs lacks #![deny(unsafe_code)]".to_owned());
    }
    let unsafe_files: Vec<String> = rust_files(&root().join("crates/tui-testing/src"))
        .into_iter()
        .filter(|f| read(f).contains("unsafe impl"))
        .map(|f| rel(&f))
        .collect();
    if unsafe_files == ["crates/tui-testing/src/perf.rs"] {
        Ok(())
    } else {
        Err(format!("unsafe impl in {unsafe_files:?}"))
    }
}

/// `syn`-based scans over `crates/tui/src` (and `components/**` when it exists).
fn parse_files(dir: &Path) -> Vec<(String, syn::File)> {
    rust_files(dir)
        .into_iter()
        .filter_map(|f| syn::parse_file(&read(&f)).ok().map(|ast| (rel(&f), ast)))
        .collect()
}

fn no_static_bound_in_component_surface() -> Result<(), String> {
    let dir = root().join("crates/tui/src/components");
    let mut hits = Vec::new();
    for (path, ast) in parse_files(&dir) {
        for item in &ast.items {
            if let syn::Item::Struct(s) = item
                && matches!(s.vis, syn::Visibility::Public(_))
            {
                let generics = quote_generics(&s.generics);
                if generics.contains("'static") {
                    hits.push(format!("{path}: {} has a 'static bound", s.ident));
                }
            }
        }
    }
    if hits.is_empty() {
        Ok(())
    } else {
        Err(hits.join("\n"))
    }
}

fn quote_generics(g: &syn::Generics) -> String {
    let mut s = String::new();
    for p in &g.params {
        if let syn::GenericParam::Type(t) = p {
            for b in &t.bounds {
                if let syn::TypeParamBound::Lifetime(l) = b {
                    s.push_str(&format!("'{}", l.ident));
                }
            }
        }
    }
    if let Some(w) = &g.where_clause {
        for pred in &w.predicates {
            if let syn::WherePredicate::Type(t) = pred {
                for b in &t.bounds {
                    if let syn::TypeParamBound::Lifetime(l) = b {
                        s.push_str(&format!("'{}", l.ident));
                    }
                }
            }
        }
    }
    s
}

fn draw_takes_shared_self() -> Result<(), String> {
    let dir = root().join("crates/tui/src/components");
    let mut hits = Vec::new();
    for (path, ast) in parse_files(&dir) {
        for item in &ast.items {
            if let syn::Item::Impl(im) = item {
                for it in &im.items {
                    if let syn::ImplItem::Fn(f) = it
                        && f.sig.ident == "draw"
                        && let Some(syn::FnArg::Receiver(r)) = f.sig.inputs.first()
                        && r.mutability.is_some()
                    {
                        hits.push(format!("{path}: fn draw takes &mut self"));
                    }
                }
            }
        }
    }
    if hits.is_empty() {
        Ok(())
    } else {
        Err(hits.join("\n"))
    }
}

fn cache_types_are_derived_only() -> Result<(), String> {
    let re = Regex::new(r"cache::<(\w+)>").map_err(|e| e.to_string())?;
    let src = root().join("crates/tui/src");
    let mut cache_types = BTreeSet::new();
    let mut texts = Vec::new();
    for file in rust_files(&src) {
        let t = read(&file);
        for cap in re.captures_iter(&t) {
            cache_types.insert(cap[1].to_owned());
        }
        texts.push((rel(&file), t));
    }
    let mut hits = Vec::new();
    for ty in &cache_types {
        let in_response = Regex::new(&format!(r"Response<{ty}>")).map_err(|e| e.to_string())?;
        let in_state = Regex::new(&format!(r"\b{ty}\b")).map_err(|e| e.to_string())?;
        for (path, t) in &texts {
            if in_response.is_match(t) {
                hits.push(format!("{path}: {ty} appears in a Response"));
            }
            // a struct named `*State` holding the cache type
            for m in Regex::new(r"pub struct (\w+State)\s*\{([^}]*)\}")
                .map_err(|e| e.to_string())?
                .captures_iter(t)
            {
                if in_state.is_match(&m[2]) {
                    hits.push(format!("{path}: {} holds cache type {ty}", &m[1]));
                }
            }
        }
    }
    if hits.is_empty() {
        Ok(())
    } else {
        Err(hits.join("\n"))
    }
}

fn capability_has_no_unicode_field() -> Result<(), String> {
    let text = read(&root().join("crates/tui/src/theme/tokens.rs"));
    let ast = syn::parse_file(&text).map_err(|e| e.to_string())?;
    for item in ast.items {
        if let syn::Item::Struct(s) = item
            && s.ident == "Capability"
        {
            let fields: Vec<String> = s
                .fields
                .iter()
                .filter_map(|f| f.ident.as_ref().map(ToString::to_string))
                .collect();
            return if fields == ["color"] {
                Ok(())
            } else {
                Err(format!("Capability fields {fields:?}"))
            };
        }
    }
    Err("Capability not found".to_owned())
}

fn no_boolean_capability_parameter_on_grid() -> Result<(), String> {
    let grid = root().join("crates/tui/src/components/grid.rs");
    if !grid.exists() {
        return Ok(());
    }
    let t = read(&grid);
    if t.contains("fn editable(") {
        return Err("grid.rs has fn editable(".to_owned());
    }
    for f in rust_files(&root().join("crates/tui/src")) {
        if read(&f).contains("trait GridCellActions") {
            return Err(format!("{}: trait GridCellActions", rel(&f)));
        }
    }
    Ok(())
}

fn core_is_backend_free() -> Result<(), String> {
    let status = Command::new("cargo")
        .args(["check", "-p", LIB, "--no-default-features", "-q"])
        .current_dir(root())
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err("cargo check --no-default-features failed".to_owned())
    }
}

fn msrv_and_edition_are_unchanged() -> Result<(), String> {
    let md = metadata()?;
    let mut errors = Vec::new();
    for p in &md.packages {
        if !md.workspace_members.contains(&p.id) {
            continue;
        }
        if p.edition != cargo_metadata::Edition::E2024 {
            errors.push(format!("{}: edition {:?}", p.name, p.edition));
        }
        let msrv = p
            .rust_version
            .as_ref()
            .map(|v| format!("{}.{}", v.major, v.minor));
        if msrv.as_deref() != Some("1.88") {
            errors.push(format!("{}: rust-version {:?}", p.name, p.rust_version));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

// ───────────────────────────── doc-check ─────────────────────────────

/// Public items of the library: type/trait/fn/const/macro names, and
/// `Type::member` associations.
#[derive(Default, Debug)]
struct Api {
    items: BTreeSet<String>,
    members: BTreeSet<(String, String)>,
}

fn collect_api() -> Api {
    let mut api = Api::default();
    for (_, ast) in parse_files(&root().join("crates/tui/src")) {
        collect_items(&ast.items, &mut api);
    }
    collect_macro_generated(&mut api);
    api.items.insert("id".to_owned());
    api
}

/// Constants produced by the crate's declarative macros (`parts!`,
/// `action_keys!`, `newtype_u16!`, `bitflags!`), invisible to `syn`.
fn collect_macro_generated(api: &mut Api) {
    let const_re = Regex::new(r"(?m)^\s*([A-Z][A-Z0-9_]*)\s*=\s*[0-9]")
        .unwrap_or_else(|_| Regex::new("$^").unwrap_or_else(|_| unreachable!()));
    let flag_re = Regex::new(r"const ([A-Z][A-Z0-9_]*)\s*=")
        .unwrap_or_else(|_| Regex::new("$^").unwrap_or_else(|_| unreachable!()));
    for file in rust_files(&root().join("crates/tui/src")) {
        let text = read(&file);
        for (needle, ty) in [
            ("parts! {", Some("Part")),
            ("action_keys! {", Some("ActionKey")),
            ("newtype_u16! {", None),
        ] {
            let mut from = 0;
            while let Some(i) = text[from..].find(needle) {
                let start = from + i + needle.len();
                let end = text[start..].find("\n}").map_or(text.len(), |e| start + e);
                let body = &text[start..end];
                let ty = match ty {
                    Some(t) => t.to_owned(),
                    None => body
                        .lines()
                        .map(str::trim)
                        .find(|l| !l.starts_with("///") && !l.is_empty())
                        .and_then(|l| l.split(',').next())
                        .unwrap_or("")
                        .trim()
                        .to_owned(),
                };
                for cap in const_re.captures_iter(body) {
                    api.members.insert((ty.clone(), cap[1].to_owned()));
                }
                api.members.insert((ty.clone(), "ALL".to_owned()));
                api.members.insert((ty.clone(), "custom".to_owned()));
                api.members.insert((ty.clone(), "name".to_owned()));
                api.members.insert((ty.clone(), "raw".to_owned()));
                from = end;
            }
        }
        let mut from = 0;
        while let Some(i) = text[from..].find("bitflags! {") {
            let start = from + i;
            let end = text[start..].find("\n}").map_or(text.len(), |e| start + e);
            let body = &text[start..end];
            let ty = Regex::new(r"pub struct (\w+)")
                .ok()
                .and_then(|r| r.captures(body).map(|c| c[1].to_owned()))
                .unwrap_or_default();
            for cap in flag_re.captures_iter(body) {
                api.members.insert((ty.clone(), cap[1].to_owned()));
            }
            for m in [
                "empty",
                "all",
                "bits",
                "contains",
                "iter",
                "iter_names",
                "from_bits_truncate",
                "union",
                "difference",
            ] {
                api.members.insert((ty.clone(), m.to_owned()));
            }
            from = end;
        }
    }
}

fn collect_items(items: &[syn::Item], api: &mut Api) {
    for item in items {
        match item {
            syn::Item::Struct(s) => {
                api.items.insert(s.ident.to_string());
                for f in &s.fields {
                    if let Some(id) = &f.ident {
                        api.members.insert((s.ident.to_string(), id.to_string()));
                    }
                }
            }
            syn::Item::Enum(e) => {
                api.items.insert(e.ident.to_string());
                for v in &e.variants {
                    api.members
                        .insert((e.ident.to_string(), v.ident.to_string()));
                }
            }
            syn::Item::Trait(t) => {
                api.items.insert(t.ident.to_string());
                for it in &t.items {
                    match it {
                        syn::TraitItem::Fn(f) => {
                            api.members
                                .insert((t.ident.to_string(), f.sig.ident.to_string()));
                        }
                        syn::TraitItem::Const(c) => {
                            api.members
                                .insert((t.ident.to_string(), c.ident.to_string()));
                        }
                        syn::TraitItem::Type(ty) => {
                            api.members
                                .insert((t.ident.to_string(), ty.ident.to_string()));
                        }
                        _ => {}
                    }
                }
            }
            syn::Item::Fn(f) => {
                api.items.insert(f.sig.ident.to_string());
            }
            syn::Item::Const(c) => {
                api.items.insert(c.ident.to_string());
            }
            syn::Item::Type(t) => {
                api.items.insert(t.ident.to_string());
            }
            syn::Item::Mod(m) => {
                api.items.insert(m.ident.to_string());
                if let Some((_, items)) = &m.content {
                    collect_items(items, api);
                }
            }
            syn::Item::Macro(m) => {
                if let Some(id) = &m.ident {
                    api.items.insert(id.to_string());
                }
            }
            syn::Item::Impl(im) => {
                let ty = match &*im.self_ty {
                    syn::Type::Path(p) => p.path.segments.last().map(|s| s.ident.to_string()),
                    _ => None,
                };
                let Some(ty) = ty else { continue };
                for it in &im.items {
                    match it {
                        syn::ImplItem::Fn(f) => {
                            api.members.insert((ty.clone(), f.sig.ident.to_string()));
                        }
                        syn::ImplItem::Const(c) => {
                            api.members.insert((ty.clone(), c.ident.to_string()));
                        }
                        _ => {}
                    }
                }
                // trait impls: the trait's methods become members of the type
                if let Some((_, path, _)) = &im.trait_
                    && let Some(seg) = path.segments.last()
                {
                    let tr = seg.ident.to_string();
                    for (t, m) in api.members.clone() {
                        if t == tr {
                            api.members.insert((ty.clone(), m));
                        }
                    }
                }
            }
            syn::Item::Use(u) => collect_use(&u.tree, api),
            _ => {}
        }
    }
}

fn collect_use(tree: &syn::UseTree, api: &mut Api) {
    match tree {
        syn::UseTree::Path(p) => collect_use(&p.tree, api),
        syn::UseTree::Name(n) => {
            api.items.insert(n.ident.to_string());
        }
        syn::UseTree::Rename(r) => {
            api.items.insert(r.rename.to_string());
        }
        syn::UseTree::Group(g) => {
            for t in &g.items {
                collect_use(t, api);
            }
        }
        syn::UseTree::Glob(_) => {}
    }
}

/// Well-known foreign members reachable through the facade's re-exports.
fn foreign_members() -> BTreeSet<(String, String)> {
    let mut m = BTreeSet::new();
    for (t, ms) in [
        (
            "Rect",
            &[
                "new",
                "clamp",
                "centered",
                "centered_horizontally",
                "centered_vertically",
                "inner",
                "outer",
                "intersection",
                "union",
                "offset",
                "is_empty",
                "ZERO",
                "positions",
                "rows",
                "columns",
            ][..],
        ),
        (
            "Color",
            &["from_u32", "Rgb", "Reset", "Black", "White", "Indexed"][..],
        ),
        ("Style", &["new", "patch", "default"][..]),
        (
            "Buffer",
            &[
                "set_stringn",
                "set_line",
                "set_string",
                "set_style",
                "cell",
                "cell_mut",
                "get",
                "get_mut",
                "empty",
            ][..],
        ),
        ("Frame", &["set_cursor_position", "area", "buffer_mut"][..]),
        (
            "KeyCode",
            &[
                "Char",
                "Enter",
                "Esc",
                "Tab",
                "Left",
                "Right",
                "Up",
                "Down",
                "BackTab",
                "Backspace",
                "Delete",
                "Home",
                "End",
                "PageUp",
                "PageDown",
                "F",
            ][..],
        ),
        ("KeyModifiers", &["CONTROL", "SHIFT", "ALT", "NONE"][..]),
        (
            "Modifier",
            &[
                "BOLD",
                "ITALIC",
                "UNDERLINED",
                "DIM",
                "REVERSED",
                "empty",
                "all",
            ][..],
        ),
        ("Position", &["new"][..]),
        ("Cell", &["symbol", "set_symbol", "set_style", "reset"][..]),
        ("Terminal", &["new", "draw", "try_draw"][..]),
        ("TestBackend", &["new"][..]),
        ("Vec", &["new"][..]),
        ("String", &["new"][..]),
        ("Option", &["None", "Some"][..]),
        ("Result", &["Ok", "Err"][..]),
        ("Default", &["default"][..]),
        ("mem", &["take"][..]),
        ("std", &["mem"][..]),
        ("core", &["fmt", "error"][..]),
        ("fmt", &["Debug", "Display", "Arguments"][..]),
        ("symbols", &["border", "line", "scrollbar"][..]),
        (
            "border",
            &["PLAIN", "ROUNDED", "DOUBLE", "ASCII", "Set"][..],
        ),
        ("scrollbar", &["Set", "VERTICAL"][..]),
        ("line", &["Set", "NORMAL", "THICK"][..]),
        (
            "ratatui_core",
            &["symbols", "buffer", "layout", "style", "text", "terminal"][..],
        ),
        ("ratatui_crossterm", &["crossterm", "CrosstermBackend"][..]),
        ("crossterm", &["event", "terminal"][..]),
        (
            "event",
            &[
                "KeyCode",
                "KeyModifiers",
                "EnableMouseCapture",
                "EnableBracketedPaste",
            ][..],
        ),
        (
            "terminal",
            &[
                "enable_raw_mode",
                "EnterAlternateScreen",
                "EnableLineWrap",
                "DisableLineWrap",
            ][..],
        ),
        ("Go", &["Manager", "Editor"][..]),
        ("Duration", &["from_millis"][..]),
        ("Cow", &["Borrowed", "Owned"][..]),
        (
            "MouseEventKind",
            &[
                "Moved",
                "Down",
                "Up",
                "Drag",
                "ScrollUp",
                "ScrollDown",
                "ScrollLeft",
                "ScrollRight",
            ][..],
        ),
        ("Event", &["as_key_press_event", "Key", "Mouse"][..]),
        ("KeyEvent", &["is_press", "is_repeat", "is_release"][..]),
        ("Theme", &["row", "gutter"][..]),
        ("Constraint", &["Length", "Ratio"][..]),
        ("Margin", &["new"][..]),
        ("CellWidth", &["cell_width"][..]),
        ("Backend", &["Error"][..]),
        ("Layout", &["split", "horizontal", "vertical"][..]),
        ("Terminal", &["insert_before"][..]),
        ("Style", &["underline_color"][..]),
        ("Self", &["PARTS", "State", "Action", "Cmd"][..]),
        ("Ident", &["method"][..]),
        ("Interaction", &["pressed", "focus_hidden"][..]),
        ("PartEdit", &["when"][..]),
    ] {
        for x in ms {
            m.insert(((*t).to_owned(), (*x).to_owned()));
        }
    }
    m
}

/// Names of `junie_tui::` imports used in the document's examples that are
/// not yet built (Slice 4 components and their vocabulary).
fn doc_allow() -> BTreeSet<String> {
    read(&root().join("xtask/doc_check_allow.txt"))
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.split_whitespace().next().unwrap_or("").to_owned())
        .collect()
}

fn doc_sections(text: &str) -> String {
    // §3–§17 and §21–§23: from "## 3." to "## 18." plus "## 21." to the end
    let mut out = String::new();
    let mut keep = false;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("## ") {
            let n: Option<u32> = rest.split('.').next().and_then(|s| s.trim().parse().ok());
            keep = matches!(n, Some(3..=17 | 21..=23));
        }
        if keep {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

fn doc_check() -> Result<(), String> {
    let text = read(&root().join("COMPONENT_ARCHITECTURE.md"));
    let scoped = doc_sections(&text);
    let api = collect_api();
    let foreign = foreign_members();
    let allow = doc_allow();
    let member_re = Regex::new(r"`([A-Z][A-Za-z0-9_]*)::([A-Za-z_][A-Za-z0-9_]*)`")
        .map_err(|e| e.to_string())?;
    let mut unresolved: BTreeMap<String, usize> = BTreeMap::new();
    let mut allowed_hits: BTreeMap<String, usize> = BTreeMap::new();
    let mut resolved = 0usize;
    // `Type::member` references in prose
    for cap in member_re.captures_iter(&scoped) {
        let (t, m) = (cap[1].to_owned(), cap[2].to_owned());
        let key = format!("{t}::{m}");
        let ok = api.members.contains(&(t.clone(), m.clone()))
            || foreign.contains(&(t.clone(), m.clone()))
            || (api.items.contains(&t)
                && api.items.contains(&m)
                && t.chars().next().is_some_and(char::is_lowercase));
        if ok {
            resolved += 1;
        } else if allow.contains(&t) || allow.contains(&key) {
            *allowed_hits.entry(key).or_insert(0) += 1;
        } else {
            *unresolved.entry(key).or_insert(0) += 1;
        }
    }
    // `use junie_tui::{…}` lists in rust blocks
    let block_re = Regex::new(r"(?s)```rust\n(.*?)```").map_err(|e| e.to_string())?;
    let use_re = Regex::new(r"use (?:junie_tui|tui_next)(?:::author)?::\{([^}]*)\}")
        .map_err(|e| e.to_string())?;
    let use_one = Regex::new(r"use (?:junie_tui|tui_next)(?:::author)?::([A-Za-z_][A-Za-z0-9_]*);")
        .map_err(|e| e.to_string())?;
    let mut blocks = 0usize;
    for b in block_re.captures_iter(&scoped) {
        blocks += 1;
        let body = &b[1];
        let mut names: Vec<String> = Vec::new();
        for u in use_re.captures_iter(body) {
            names.extend(
                u[1].split(',')
                    .map(|s| s.trim().trim_start_matches("self").trim_start_matches("::"))
                    .filter(|s| !s.is_empty())
                    .map(|s| s.split("::").last().unwrap_or(s).to_owned()),
            );
        }
        for u in use_one.captures_iter(body) {
            names.push(u[1].to_owned());
        }
        for n in names {
            if n == "id" || n == "self" || api.items.contains(&n) {
                resolved += 1;
            } else if allow.contains(&n) {
                *allowed_hits.entry(n).or_insert(0) += 1;
            } else {
                *unresolved.entry(n).or_insert(0) += 1;
            }
        }
    }
    println!(
        "doc-check: {blocks} rust blocks, {resolved} references resolved against crates/tui/src"
    );
    if !allowed_hits.is_empty() {
        println!("doc-check: not yet built (allow-listed, xtask/doc_check_allow.txt):");
        for (k, n) in &allowed_hits {
            println!("  {k} ({n})");
        }
    }
    if unresolved.is_empty() {
        Ok(())
    } else {
        let mut msg =
            String::from("doc-check: unresolved references (build them or allow-list them):\n");
        for (k, n) in &unresolved {
            msg.push_str(&format!("  {k} ({n})\n"));
        }
        Err(msg)
    }
}
