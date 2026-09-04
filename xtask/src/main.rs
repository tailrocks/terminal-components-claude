//! Workspace checks (`COMPONENT_ARCHITECTURE.md` §16.5, §21 item 34, §22.7).
//!
//! ```text
//! cargo run -p xtask -- doc-check                # §3–§17 and §21–§23 references resolve
//! cargo run -p xtask -- boundary                 # every §16.5 / §22.7 grep and metadata check
//! cargo run -p xtask -- boundary --check <name>  # one named check
//! cargo run -p xtask -- bless-guard              # §16.3 / §36.5 baseline bless guard
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
        // §16.3 / §20.10 / §36.5. A named `boundary` check, so it inherits the
        // `ok`/`FAIL` formatting, the `N check(s) failed` tail and `xtask list`.
        Some("bless-guard") => boundary(Some("baseline_moves_are_classified")),
        Some("list") => {
            for name in CHECKS.iter().map(|c| c.0) {
                println!("{name}");
            }
            Ok(())
        }
        _ => {
            eprintln!("usage: xtask <doc-check | boundary [--check NAME] | bless-guard | list>");
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

/// Lines of a file with **each** `#[cfg(test)]` item removed.
///
/// The old version `break`ed at the first `#[cfg(test)]`, so a mid-file
/// `#[cfg(test)] pub(crate) const fn stats` left the whole rest of the file
/// unscanned by all 26 forbidden-pattern rules (MA-2). This skips exactly the
/// attributed item by brace matching and keeps scanning afterwards.
fn non_test_lines(text: &str) -> Vec<(usize, &str)> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < lines.len() {
        let Some(line) = lines.get(i) else { break };
        if line.trim_start().starts_with("#[cfg(test)]") {
            i = skip_item(&lines, i.saturating_add(1));
            continue;
        }
        out.push((i.saturating_add(1), *line));
        i = i.saturating_add(1);
    }
    out
}

/// The index just past the item starting at `from`: any further attributes,
/// then either a braced body (matched, ignoring braces inside `"` strings and
/// `//` comments) or a single `;`-terminated line.
fn skip_item(lines: &[&str], from: usize) -> usize {
    let mut i = from;
    let mut depth = 0usize;
    let mut opened = false;
    while i < lines.len() {
        let Some(line) = lines.get(i) else { break };
        let code = code_line(line);
        let mut in_str = false;
        let mut prev_escape = false;
        for c in code.chars() {
            if in_str {
                if prev_escape {
                    prev_escape = false;
                } else if c == '\\' {
                    prev_escape = true;
                } else if c == '"' {
                    in_str = false;
                }
                continue;
            }
            match c {
                '"' => in_str = true,
                '{' => {
                    depth = depth.saturating_add(1);
                    opened = true;
                }
                '}' => depth = depth.saturating_sub(1),
                _ => {}
            }
        }
        i = i.saturating_add(1);
        if opened && depth == 0 {
            return i;
        }
        if !opened && code.trim_end().ends_with(';') {
            return i;
        }
    }
    i
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
    ("no_unreachable_spin_loops", no_unreachable_spin_loops),
    (
        "ratatui_crossterm_is_named_in_exactly_two_files",
        ratatui_crossterm_is_named_in_exactly_two_files,
    ),
    ("every_named_test_exists", every_named_test_exists),
    (
        "conformance_covers_every_public_component",
        conformance_covers_every_public_component,
    ),
    (
        "state_override_is_used_only_in_apps_and_fixtures",
        state_override_is_used_only_in_apps_and_fixtures,
    ),
    (
        "examples_are_external_consumers",
        examples_are_external_consumers,
    ),
    (
        "inherit_forced_stays_crate_internal",
        inherit_forced_stays_crate_internal,
    ),
    ("binary_names_are_preserved", binary_names_are_preserved),
    (
        "app_libs_are_not_published_and_are_not_depended_on_by_the_library",
        app_libs_are_not_published_and_are_not_depended_on_by_the_library,
    ),
    (
        "applications_depend_only_on_the_library_facade",
        applications_depend_only_on_the_library_facade,
    ),
    (
        "baseline_moves_are_classified",
        baseline_moves_are_classified,
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
    // NOTE (§4(j)-3): this rule cannot distinguish *construction*
    // (`st.fg = c` while building a `Style`) from *layering* (which R-9
    // forbids); the two occurrences in `ui/paint.rs` are construction and the
    // file is allow-listed by path for that reason.
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
        // `ui/derived.rs` is the single documented exception: `dyn
        // Any::downcast_mut` returns `Option` and safe Rust cannot express
        // "keyed by TypeId::of::<T>() ⇒ holds Box<T>". A named path shows the
        // exception; the alternative was a livelock (BL-2).
        allowed: &[
            "crates/tui-testing/",
            "xtask/",
            "crates/tui/src/ui/derived.rs",
        ],
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
        // §22.7's broad regex: a narrowed one lets `Color::Rgb(r, g, b)` from
        // computed values through anywhere (D-10). The exceptions are named
        // *paths*, which are printed and reviewable; path exceptions do not
        // feed the "legacy_api.txt must be empty" condition.
        re: r"Color::Rgb\(|Color::from_u32\(|#[0-9a-fA-F]{6}\b",
        allowed: &[
            "crates/tui/src/theme/builtin/junie.rs",
            "crates/tui/src/theme/builtin/paper.rs",
            // derives a colour arithmetically from the seeds (L* ladders)
            "crates/tui/src/theme/builder.rs",
            // reconstructs `(r, g, b)` when downgrading to a palette
            "crates/tui/src/theme/downgrade.rs",
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
    Rule {
        n: 27,
        re: r"CrosstermBackend|ratatui_crossterm::crossterm::(?:terminal|execute|cursor|style)",
        allowed: &["crates/tui/src/runtime/session.rs"],
        why: "§22.1: the backend lives in runtime/session.rs only",
        only_if: None,
        under: None,
    },
    Rule {
        n: 28,
        re: r"spin_loop",
        allowed: &[],
        why: "BL-2: a livelock is strictly worse than a panic",
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

// ─────────────────────── the named-test inventory (§21 item 28) ───────────────────────

/// The section of `COMPONENT_ARCHITECTURE.md` between two headings.
fn doc_section(doc: &str, from: &str, to: &str) -> String {
    let Some(a) = doc.find(from) else {
        return String::new();
    };
    let rest = doc.get(a..).unwrap_or_default();
    match rest.find(to) {
        Some(b) => rest.get(..b).unwrap_or_default().to_owned(),
        None => rest.to_owned(),
    }
}

/// Backticked snake-case identifiers at parenthesis depth 0, with HTML
/// comments removed. The listed names are at depth 0; the prose that explains
/// them is inside `(…)`, which is exactly how §16.1 is written.
fn doc_test_names(text: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut depth = 0i32;
    let bytes = text.as_bytes();
    let mut i = 0usize;
    while i < text.len() {
        let rest = text.get(i..).unwrap_or_default();
        if rest.starts_with("<!--") {
            i = rest.find("-->").map_or(text.len(), |j| i + j + 3);
            continue;
        }
        let c = bytes.get(i).copied().unwrap_or(b' ');
        match c {
            b'(' => depth = depth.saturating_add(1),
            b')' => depth = depth.saturating_sub(1),
            b'`' => {
                let body = rest.get(1..).unwrap_or_default();
                let Some(end) = body.find('`') else {
                    break;
                };
                let token = body.get(..end).unwrap_or_default();
                if depth == 0 {
                    let last = token.rsplit("::").next().unwrap_or(token);
                    let snake = last
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
                    if snake
                        && last.matches('_').count() >= 2
                        && last.starts_with(|c: char| c.is_ascii_lowercase())
                    {
                        out.insert(last.to_owned());
                    }
                }
                i = i + 1 + end + 1;
                continue;
            }
            _ => {}
        }
        i = i.saturating_add(
            text.get(i..)
                .and_then(|r| r.chars().next())
                .map_or(1, char::len_utf8),
        );
    }
    out
}

/// The first cell of every `| \`name\` | … |` row.
fn doc_table_names(text: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for line in text.lines() {
        let line = line.trim();
        if !line.starts_with('|') {
            continue;
        }
        let cell = line.trim_start_matches('|').trim();
        let Some(rest) = cell.strip_prefix('`') else {
            continue;
        };
        let Some(end) = rest.find('`') else { continue };
        let name = rest.get(..end).unwrap_or_default();
        let last = name.rsplit("::").next().unwrap_or(name);
        if last
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
            && last.matches('_').count() >= 2
        {
            out.insert(last.to_owned());
        }
    }
    out
}

/// Every `#[test]`-annotated function name in the workspace's sources.
///
/// §21 item 28 words this as `cargo test --workspace -- --list`. This scans
/// the sources for the same thing — every `#[test] fn name` — because the
/// check runs *inside* `cargo test --test architecture` and a nested
/// `cargo test --workspace --test perf --release -- --list` would rebuild the
/// world in a second profile on every architecture run. The two enumerate the
/// same set; source scanning additionally sees `cfg`-gated tests, which for a
/// one-directional "the name exists" check is the safer direction.
fn declared_test_names() -> BTreeSet<String> {
    let r = root();
    let mut dirs = vec![
        r.join("crates"),
        r.join("src"),
        r.join("tests"),
        r.join("xtask/src"),
    ];
    if r.join("apps").exists() {
        dirs.push(r.join("apps"));
    }
    let Ok(re) = Regex::new(r"\bfn\s+([a-z_][a-z0-9_]*)\s*\(") else {
        return BTreeSet::new();
    };
    let mut out = BTreeSet::new();
    for dir in dirs {
        for file in rust_files(&dir) {
            if rel(&file).contains("/target/") {
                continue;
            }
            let text = read(&file);
            let lines: Vec<&str> = text.lines().collect();
            for (i, line) in lines.iter().enumerate() {
                if !line.trim_start().starts_with("#[test]") {
                    continue;
                }
                for l in lines.iter().skip(i.saturating_add(1)).take(4) {
                    if let Some(c) = re.captures(l) {
                        if let Some(m) = c.get(1) {
                            out.insert(m.as_str().to_owned());
                        }
                        break;
                    }
                }
            }
        }
    }
    // `trybuild` cases are file names, not `#[test]` functions
    if let Ok(entries) = std::fs::read_dir(r.join("crates/tui/tests/ui")) {
        for e in entries.filter_map(Result::ok) {
            if let Some(stem) = e.path().file_stem().and_then(|s| s.to_str()) {
                out.insert(stem.to_owned());
            }
        }
    }
    out
}

/// The names deferred to a later slice, with the slice that owns each.
fn named_tests_allow() -> BTreeMap<String, String> {
    let p = root().join("xtask/named_tests_allow.txt");
    let mut out = BTreeMap::new();
    for line in read(&p).lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (name, why) = line.split_once("  #").unwrap_or((line, ""));
        out.insert(name.trim().to_owned(), why.trim().to_owned());
    }
    out
}

/// Benchmarks §16.6 records as **deleted**: their deletion is asserted by
/// line-absence in `perf_baseline.txt` (§21 item 28), so the name must not
/// appear as the first field of any data row of any baseline in the tree.
const DELETED_PERF_ROWS: [(&str, &str); 1] = [(
    "capsule_pane_clone_4x2000",
    "§21 item 10 / §16.6: `Capsule` clones no viewport per frame, so the \
     benchmark is deleted (Slice 7, apps/jackin-preview)",
)];

/// Every `perf_baseline.txt` in the tree, build output excluded.
///
/// There are two, holding disjoint rows: the root `tests/perf_baseline.txt` —
/// the WP-0 pre-refactor baseline tagged `perf/baseline`, still carrying the
/// application benchmarks that Slices 5–7 move out — and
/// `crates/tui/tests/perf_baseline.txt`, the new library baseline written by
/// `crates/tui-testing`. A deletion assertion aimed at one file alone passes by
/// construction when the row lives in the other, and stops meaning anything
/// the moment a slice moves a benchmark; scanning all of them is invariant
/// under those moves.
fn perf_baselines() -> Vec<PathBuf> {
    WalkDir::new(root())
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !(e.file_type().is_dir() && (name == "target" || name == ".git"))
        })
        .filter_map(Result::ok)
        .filter(|e| e.file_name() == "perf_baseline.txt")
        .map(|e| e.path().to_path_buf())
        .collect()
}

/// The §21 item 28 deletion assertion: one message per `DELETED_PERF_ROWS`
/// name still present in a baseline, naming the file and line so the failure
/// is actionable rather than a bare "must be ABSENT".
fn surviving_deleted_perf_rows() -> Vec<String> {
    let mut hits = Vec::new();
    for path in perf_baselines() {
        for (i, line) in read(&path).lines().enumerate() {
            let row = line.trim();
            if row.is_empty() || row.starts_with('#') {
                continue;
            }
            let name = row.split_whitespace().next().unwrap_or_default();
            for (deleted, why) in DELETED_PERF_ROWS {
                if name == deleted {
                    hits.push(format!(
                        "{}:{}: {deleted} must be ABSENT from perf_baseline.txt — {why}",
                        rel(&path),
                        i.saturating_add(1)
                    ));
                }
            }
        }
    }
    hits
}

fn every_named_test_exists() -> Result<(), String> {
    let doc = read(&root().join("COMPONENT_ARCHITECTURE.md"));
    if doc.is_empty() {
        return Err("COMPONENT_ARCHITECTURE.md not found".to_owned());
    }
    let mut want = BTreeSet::new();
    want.extend(doc_test_names(&doc_section(&doc, "### 16.1", "### 16.2")));
    // §16.2: the suite-level bullets only (the per-case names are generated)
    for line in doc_section(&doc, "Suite-level tests (emitted once", "---").lines() {
        if line.trim_start().starts_with("* `") {
            want.extend(doc_test_names(line));
        }
    }
    // §16.4: only the "New application coverage" list names tests; the rest
    // of the section is the `Harness` API contract
    want.extend(doc_test_names(&doc_section(
        &doc,
        "**New application coverage**",
        "### 16.5",
    )));
    // §16.5: the architecture-check table. Its absence was a blind spot
    // covering EVERY boundary check (§47.4): all nine names that follow are
    // listed only here and in Appendix B, so until this line existed their
    // absence could not fail any gate — which is how three missing `apps/`
    // guards came to be found by a person reading the document.
    want.extend(doc_table_names(&doc_section(&doc, "### 16.5", "### 16.6")));
    want.extend(doc_table_names(&doc_section(&doc, "### 16.6", "## 17.")));

    let have = declared_test_names();
    let allow = named_tests_allow();
    let mut missing = Vec::new();
    let mut deferred = 0usize;
    for name in &want {
        if have.contains(name) {
            continue;
        }
        if allow.contains_key(name) {
            deferred = deferred.saturating_add(1);
            continue;
        }
        missing.push(name.clone());
    }
    println!(
        "every_named_test_exists: {} names in §16.1/§16.2/§16.4/§16.5/§16.6, {} present, {deferred} deferred (xtask/named_tests_allow.txt)",
        want.len(),
        want.len()
            .saturating_sub(missing.len())
            .saturating_sub(deferred)
    );
    // the §21 item 28 deletion assertion
    missing.extend(surviving_deleted_perf_rows());
    // an allow-list entry that is now satisfied must be removed
    let stale: Vec<&String> = allow.keys().filter(|n| have.contains(*n)).collect();
    if !stale.is_empty() {
        missing.push(format!("stale entries in named_tests_allow.txt: {stale:?}"));
    }
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "named tests missing (add the test, or defer it in xtask/named_tests_allow.txt with the owning slice):\n  {}",
            missing.join("\n  ")
        ))
    }
}

/// One `name => NameCase` entry of a `conformance_suite!` invocation.
struct SuiteEntry {
    module: syn::Ident,
    case: syn::Type,
}

impl syn::parse::Parse for SuiteEntry {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let module: syn::Ident = input.parse()?;
        input.parse::<syn::Token![=>]>()?;
        let case: syn::Type = input.parse()?;
        Ok(Self { module, case })
    }
}

/// The case type names **registered** in `conformance.rs`'s
/// `conformance_suite!` invocation, read from the macro's token stream.
///
/// Never a substring search over the file text. Incident 2: `select =>
/// SelectCase,` is commented out of the list while the identifier
/// `SelectCase` occurs nine more times in the same file — in prose, in
/// `struct SelectCase;`, in its `impl Conformance` block and in three
/// standalone tests — so `text.contains("SelectCase")` reported a withheld
/// case as registered and the §16.2 coverage gate could not fail. Comments
/// are not tokens, so nothing outside the list is visible here.
fn registered_conformance_cases(path: &Path) -> Result<BTreeSet<String>, String> {
    let text = read(path);
    if text.is_empty() {
        return Err(format!("{} not found", rel(path)));
    }
    let ast = syn::parse_file(&text).map_err(|e| format!("{} does not parse: {e}", rel(path)))?;
    let mut cases = BTreeSet::new();
    let mut invocations = 0usize;
    collect_suite_entries(&ast.items, &mut cases, &mut invocations)?;
    if invocations == 0 {
        return Err(format!(
            "{}: no conformance_suite! invocation — the §16.2 coverage gate has nothing to read",
            rel(path)
        ));
    }
    Ok(cases)
}

/// Walks items (and inline modules) for `conformance_suite!` invocations and
/// parses each body as `name => NameCase,` entries.
fn collect_suite_entries(
    items: &[syn::Item],
    cases: &mut BTreeSet<String>,
    invocations: &mut usize,
) -> Result<(), String> {
    for item in items {
        match item {
            syn::Item::Mod(m) => {
                if let Some((_, inner)) = &m.content {
                    collect_suite_entries(inner, cases, invocations)?;
                }
            }
            syn::Item::Macro(m)
                if m.mac
                    .path
                    .segments
                    .last()
                    .is_some_and(|s| s.ident == "conformance_suite") =>
            {
                *invocations = invocations.saturating_add(1);
                let entries = m
                    .mac
                    .parse_body_with(
                        syn::punctuated::Punctuated::<SuiteEntry, syn::Token![,]>::parse_terminated,
                    )
                    .map_err(|e| format!("conformance_suite! body does not parse: {e}"))?;
                for entry in entries {
                    let syn::Type::Path(tp) = &entry.case else {
                        return Err(format!(
                            "conformance_suite!: entry `{}` registers a non-path case type",
                            entry.module
                        ));
                    };
                    let seg = tp.path.segments.last().ok_or_else(|| {
                        format!(
                            "conformance_suite!: entry `{}` registers an empty path",
                            entry.module
                        )
                    })?;
                    cases.insert(seg.ident.to_string());
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// §16.2: `conformance.rs`'s `conformance_suite!` must list a case for every
/// public component, so adding a component without registering it fails CI.
fn conformance_covers_every_public_component() -> Result<(), String> {
    let dir = root().join("crates/tui/src/components");
    let registered = registered_conformance_cases(&root().join("crates/tui/tests/conformance.rs"))?;
    let mut missing = Vec::new();
    let mut covered = 0usize;
    for (path, ast) in parse_files(&dir) {
        if path.ends_with("/mod.rs") {
            continue;
        }
        // the component type is the public struct whose inherent impl has
        // `pub fn draw(&self, ui: &mut Ui<'_>, …)`
        let mut drawn: BTreeSet<String> = BTreeSet::new();
        for item in &ast.items {
            if let syn::Item::Impl(im) = item
                && im.trait_.is_none()
                && let syn::Type::Path(tp) = im.self_ty.as_ref()
                && let Some(seg) = tp.path.segments.last()
            {
                for it in &im.items {
                    if let syn::ImplItem::Fn(f) = it
                        && f.sig.ident == "draw"
                        && matches!(f.vis, syn::Visibility::Public(_))
                    {
                        drawn.insert(seg.ident.to_string());
                    }
                }
            }
        }
        let public: BTreeSet<String> = ast
            .items
            .iter()
            .filter_map(|i| match i {
                syn::Item::Struct(s) if matches!(s.vis, syn::Visibility::Public(_)) => {
                    Some(s.ident.to_string())
                }
                _ => None,
            })
            .collect();
        for name in drawn.intersection(&public) {
            let case = format!("{name}Case");
            if registered.contains(&case) {
                covered = covered.saturating_add(1);
            } else {
                missing.push(format!(
                    "{path}: {name} is not certified — the conformance_suite! list in \
                     crates/tui/tests/conformance.rs has no `=> {case}` entry \
                     (mentioning {case} elsewhere in that file does not register it)"
                ));
            }
        }
    }
    println!(
        "conformance_covers_every_public_component: {covered} component(s) registered, \
         {} entr(y/ies) in conformance_suite!",
        registered.len()
    );
    if missing.is_empty() {
        Ok(())
    } else {
        Err(missing.join("\n"))
    }
}

/// §21 item 30: `.state_override(` is a fixture affordance, never a library
/// or application-logic call.
fn state_override_is_used_only_in_apps_and_fixtures() -> Result<(), String> {
    let mut dirs = vec![root().join("crates")];
    if root().join("apps").exists() {
        dirs.push(root().join("apps"));
    }
    let re = Regex::new(r"\.state_override\(").map_err(|e| e.to_string())?;
    // the builder's own forwarding (`self.ov = self.ov.state_override(s)`) is
    // the *definition* of the affordance, not a use of it
    let own =
        Regex::new(r"self\.\w+\s*=\s*self\.\w+\.state_override\(|pub (const )?fn state_override")
            .map_err(|e| e.to_string())?;
    let mut hits = Vec::new();
    for dir in dirs {
        for file in rust_files(&dir) {
            let path = rel(&file);
            if path.contains("/target/")
                || path.starts_with("apps/")
                // every test target is a fixture context: `tests/fixtures/**`
                // is the documented home, and the conformance cases are the
                // same thing under another name
                || path.contains("crates/tui/tests/")
                || path.contains("crates/tui-testing/")
            {
                continue;
            }
            // A `#[cfg(test)]` item in library source is a fixture context
            // for the same reason `crates/tui/tests/**` is: it is compiled
            // only for `cargo test`, it ships in no binary, and §16.1 puts a
            // component's own unit tests beside the component
            // (`dialog::a_forced_dialog_registers_no_control` asserts that a
            // *forced* dialog registers nothing, which it cannot do without
            // forcing one). The rule this check protects — no component
            // forces its own visual state where behaviour depends on it —
            // is about the code that runs in an application.
            let mut depth: i32 = 0;
            let mut pending_cfg_test = false;
            let mut test_at: Option<i32> = None;
            for (i, line) in read(&file).lines().enumerate() {
                let code = code_line(line);
                if test_at.is_none() && re.is_match(code) && !own.is_match(code) {
                    hits.push(format!("{path}:{}: {}", i.saturating_add(1), line.trim()));
                }
                if code.contains("#[cfg(test)]") {
                    pending_cfg_test = true;
                }
                let opens = i32::try_from(code.matches('{').count()).unwrap_or(0);
                let closes = i32::try_from(code.matches('}').count()).unwrap_or(0);
                if pending_cfg_test && opens > 0 {
                    test_at = Some(depth);
                    pending_cfg_test = false;
                }
                depth = depth.saturating_add(opens).saturating_sub(closes);
                if test_at.is_some_and(|d| depth <= d) {
                    test_at = None;
                }
            }
        }
    }
    if hits.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "`.state_override(` outside apps/**, crates/tui/tests/**, crates/tui-testing/** and \
             a component's own builder forwarding:\n{}",
            hits.join("\n")
        ))
    }
}

/// §12.1 / §21 item 30: `inherit_forced` is the **crate-internal**
/// composition half of A11 — a container forcing a component it owns. It is
/// deliberately not spelled `.state_override(`, so
/// `state_override_is_used_only_in_apps_and_fixtures` still sees every
/// *caller* use of the public affordance. That only holds while the hook
/// itself cannot be reached from outside: if a later slice makes it `pub`
/// anywhere but the `FieldControl` trait default, an application can force a
/// component's state without ever writing `.state_override(`, and the A11
/// boundary check becomes decorative.
fn inherit_forced_stays_crate_internal() -> Result<(), String> {
    let mut dirs = vec![root().join("crates")];
    if root().join("apps").exists() {
        dirs.push(root().join("apps"));
    }
    let re = Regex::new(r"\binherit_forced\b").map_err(|e| e.to_string())?;
    let public = Regex::new(r"pub (const )?fn inherit_forced").map_err(|e| e.to_string())?;
    // the trait declaration and its defaulted body
    let trait_home = "crates/tui/src/field_control.rs";
    let mut hits = Vec::new();
    let mut seen = 0usize;
    for dir in dirs {
        for file in rust_files(&dir) {
            let path = rel(&file);
            if path.contains("/target/") {
                continue;
            }
            let component = path.starts_with("crates/tui/src/components/");
            let is_trait = path == trait_home;
            for (i, line) in read(&file).lines().enumerate() {
                let code = code_line(line);
                if !re.is_match(code) {
                    continue;
                }
                seen = seen.saturating_add(1);
                let at = format!("{path}:{}: {}", i.saturating_add(1), line.trim());
                if !component && !is_trait {
                    hits.push(at);
                } else if public.is_match(code) && !is_trait {
                    hits.push(format!("{at}  (public outside the trait default)"));
                }
            }
        }
    }
    println!("inherit_forced_stays_crate_internal: {seen} mention(s)");
    if hits.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "`inherit_forced` may appear only under crates/tui/src/components/** and \
             {trait_home}, and may be `pub` only as the trait default:\n{}",
            hits.join("\n")
        ))
    }
}

/// §16.5: the examples are external consumers — they compile against the
/// public facade only, with no `#[path]`, no `include!` and no private paths.
fn examples_are_external_consumers() -> Result<(), String> {
    let dir = root().join("crates/tui/examples");
    let mut hits = Vec::new();
    let mut n = 0usize;
    for file in rust_files(&dir) {
        n = n.saturating_add(1);
        let path = rel(&file);
        let text = read(&file);
        for (i, line) in text.lines().enumerate() {
            let code = code_line(line);
            if code.contains("#[path") || code.contains("include!") {
                hits.push(format!("{path}:{}: {}", i.saturating_add(1), line.trim()));
            }
        }
    }
    if n == 0 {
        return Err("no examples found under crates/tui/examples".to_owned());
    }
    let status = Command::new("cargo")
        .args(["build", "-p", LIB, "--examples", "-q"])
        .current_dir(root())
        .status()
        .map_err(|e| e.to_string())?;
    if !status.success() {
        hits.push("cargo build -p tui-next --examples failed".to_owned());
    }
    println!("examples_are_external_consumers: {n} example(s)");
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
/// §22.7 (2a): absent from the **entire** normal closure.
const FORBIDDEN_ANYWHERE: [&str; 5] = [
    "ratatui",
    "ratatui-widgets",
    "ratatui-macros",
    // (2b): these can only arrive through `ratatui-core` features we disable
    "critical-section",
    "palette",
];

/// §22.7 (2c): crossterm's own internals. They may appear **only beneath
/// `ratatui-crossterm`** — they are crossterm's choice, not ours, and §22.4's
/// decision is about *our* containers (enforced by forbidden-pattern rule 26
/// over our source).
const ONLY_UNDER_CROSSTERM: [&str; 8] = [
    "smallvec",
    "parking_lot",
    "parking_lot_core",
    "lock_api",
    "scopeguard",
    "libc",
    "mio",
    "signal-hook",
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
    let full_names: BTreeSet<String> = full.iter().map(|(n, _, _)| n.clone()).collect();
    if !full_names.contains("crossterm") {
        errors
            .push("crossterm is not in the closure at all (ratatui-crossterm missing?)".to_owned());
    }
    // (2a) + (2b): absent from the ENTIRE closure, not merely the pruned one
    for f in FORBIDDEN_ANYWHERE {
        if full_names.contains(f) {
            errors.push(format!("{f} is in the entire normal closure (2a/2b)"));
        }
    }
    // (2d): no direct `smallvec`, no direct `crossterm`
    if direct.contains("crossterm") {
        errors.push("direct crossterm dependency (2d)".to_owned());
    }
    if direct.contains("smallvec") {
        errors.push("direct smallvec dependency (2d)".to_owned());
    }
    if closure.contains("smallvec") {
        errors.push("smallvec survives the crossterm prune (2c/2d)".to_owned());
    }
    // (2c): every path to each of crossterm's internals passes through
    // `ratatui-crossterm`. Printed on success so the exception is visible.
    for name in ONLY_UNDER_CROSSTERM {
        if !full_names.iter().any(|n| n == name || n.starts_with(name)) {
            continue; // not in the closure at all: nothing to prove
        }
        match inverted_paths(name) {
            Err(e) => errors.push(format!("cargo tree --invert {name}: {e}")),
            Ok(paths) => {
                if paths.trim().is_empty() {
                    continue;
                }
                if !paths.contains("ratatui-crossterm") {
                    errors.push(format!(
                        "{name} is reachable without ratatui-crossterm (2c):\n{paths}"
                    ));
                } else {
                    println!("2c: {name} is reachable only beneath ratatui-crossterm");
                }
            }
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

/// `cargo tree -p tui-next -e normal --invert <crate>`, for §22.7 (2c).
fn inverted_paths(name: &str) -> Result<String, String> {
    let out = Command::new("cargo")
        .args([
            "tree", "-p", LIB, "-e", "normal", "--invert", name, "--prefix", "none",
        ])
        .current_dir(root())
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        // an absent crate is not an error for this assertion
        return Ok(String::new());
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
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

// ─────────────── the `apps/` boundary guards (§16.5, §47.5) ───────────────

/// One application package: its `[[bin]]` name, its `apps/` directory, its
/// `[lib]` target name and the slice that owns the migration (§47.1).
///
/// The binary name doubles as the package name: Appendix B.3 item 7 gives each
/// app a package with a `[lib]` (`showcase_app`, …) and a thin `[[bin]]` whose
/// `main` calls `<app>::run()`.
struct AppPackage {
    bin: &'static str,
    dir: &'static str,
    lib: &'static str,
    slice: &'static str,
}

/// The three applications, in the order §47.1 migrates them.
const APPS: [AppPackage; 3] = [
    AppPackage {
        bin: "showcase",
        dir: "apps/showcase",
        lib: "showcase_app",
        slice: "Slice 5",
    },
    AppPackage {
        bin: "tablepro",
        dir: "apps/tablepro",
        lib: "tablepro_app",
        slice: "Slice 6",
    },
    AppPackage {
        bin: "jackin-preview",
        dir: "apps/jackin-preview",
        lib: "jackin_app",
        slice: "Slice 7",
    },
];

/// The workspace's tooling package. Its binary is the checker itself, not a
/// shipped application, and is excluded from `binary_names_are_preserved` by
/// **package name** so that a second tooling binary still fails.
const TOOLING: &str = "xtask";

/// The `bin` target names of the **legacy root package**, as cargo resolves
/// them — which includes autodiscovery, not only explicit `[[bin]]` sections.
///
/// That distinction is load-bearing and was verified rather than assumed:
/// deleting `[[bin]] showcase` while `src/bin/showcase/main.rs` remains leaves
/// the target in place, so the slice index below correctly reports the
/// migration as *not started*. §47.1 requires the section **and** the
/// `src/bin/<app>/**` tree to go in the same commit, and this reads the same
/// thing cargo does.
///
/// Empty once the root manifest goes virtual (§47.1, the commit between
/// Slice 7 and Slice 8), which is the correct reading: no root package
/// declares no binaries, so every application is due.
fn root_package_bins(md: &cargo_metadata::Metadata) -> BTreeSet<String> {
    md.packages
        .iter()
        .filter(|p| p.manifest_path.parent() == Some(md.workspace_root.as_path()))
        .flat_map(|p| p.targets.iter())
        .filter(|t| t.kind.contains(&cargo_metadata::TargetKind::Bin))
        .map(|t| t.name.clone())
        .collect()
}

/// The applications whose migration has **started**, and which must therefore
/// be present in full.
///
/// The slice index is read off the tree rather than off a constant a builder
/// has to bump: §47.1 binds the root package to lose `[[bin]] X` **in the same
/// commit** that adds `apps/X`, so "the root no longer declares `[[bin]] X`"
/// is exactly "`apps/X` is due". A builder who drops the root binary without
/// adding the package fails here; one who adds the package without dropping
/// the root binary fails here *and* in `binary_names_are_preserved`, which
/// sees the duplicate.
fn due_apps(md: &cargo_metadata::Metadata) -> Vec<&'static AppPackage> {
    let root_bins = root_package_bins(md);
    APPS.iter().filter(|a| !root_bins.contains(a.bin)).collect()
}

/// §16.5 / §47.5. The multiset of `bin` target names across every workspace
/// member equals `{showcase, tablepro, jackin-preview}`.
///
/// **An equality over a multiset, deliberately.** Containment would miss a
/// rename; a set would miss the migration's real hazard, which §47.5 states
/// as: the instant the root package and `apps/showcase` both declare
/// `[[bin]] showcase`, `target/debug/showcase` is whichever built last and the
/// capture harness silently captures the wrong program. That shows up only as
/// a *count* of two.
///
/// This check is meaningful on today's tree — the three names are the root
/// package's binaries — and stays meaningful after each migration, because the
/// name moves rather than changes.
fn binary_names_are_preserved() -> Result<(), String> {
    let md = metadata()?;
    let mut found: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut tooling: Vec<String> = Vec::new();
    for p in md.workspace_packages() {
        for t in &p.targets {
            if !t.kind.contains(&cargo_metadata::TargetKind::Bin) {
                continue;
            }
            if p.name.as_str() == TOOLING {
                tooling.push(t.name.clone());
                continue;
            }
            found
                .entry(t.name.clone())
                .or_default()
                .push(p.name.as_str().to_owned());
        }
    }
    let mut errors = Vec::new();
    // the tooling exclusion is by package, and the package is pinned to one bin
    if tooling != vec![TOOLING.to_owned()] {
        errors.push(format!(
            "the `{TOOLING}` package declares bins {tooling:?}, expected exactly [\"{TOOLING}\"] \
             — the tooling exclusion covers that one binary and nothing else"
        ));
    }
    for a in &APPS {
        match found.get(a.bin) {
            None => errors.push(format!(
                "`[[bin]] {}` is missing from the workspace (owner {}): goal §21 preserves all \
                 three binary names across the split",
                a.bin, a.slice
            )),
            Some(pkgs) if pkgs.len() > 1 => errors.push(format!(
                "`[[bin]] {}` is declared by {} packages {pkgs:?} — `target/debug/{}` is whichever \
                 built last and the capture harness captures the wrong program (§47.5); §47.1 \
                 drops the root binary in the same commit that adds `{}`",
                a.bin,
                pkgs.len(),
                a.bin,
                a.dir
            )),
            Some(_) => {}
        }
    }
    let want: BTreeSet<&str> = APPS.iter().map(|a| a.bin).collect();
    for (name, pkgs) in &found {
        if !want.contains(name.as_str()) {
            errors.push(format!(
                "unexpected `[[bin]] {name}` in {pkgs:?}: the workspace ships exactly {want:?}"
            ));
        }
    }
    if errors.is_empty() {
        let where_from: Vec<String> = APPS
            .iter()
            .map(|a| {
                let pkg = found
                    .get(a.bin)
                    .and_then(|p| p.first())
                    .map_or("?", String::as_str);
                format!("{}({pkg})", a.bin)
            })
            .collect();
        println!(
            "binary_names_are_preserved: {} — plus the `{TOOLING}` tooling binary",
            where_from.join(", ")
        );
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

/// §16.5 / §21 item 23 / §47.5. Every **due** application lib exists, is
/// `publish = false`, and no workspace member other than the library itself is
/// in the library's normal dependency closure.
///
/// **The expected set is slice-indexed and a missing member is a failure**
/// (§47.5): `{showcase_app}` from Slice 5, `+ tablepro_app` from 6,
/// `+ jackin_app` from 7, with `due_apps` reading the index off the root
/// package's remaining `[[bin]]`s rather than off a hand-maintained constant.
///
/// **Honest statement of what is vacuous today.** No `apps/` package exists,
/// so the expected set is empty and the *publish* and *lib-target* assertions
/// have nothing to run against; they are **vacuous until Slice 5**. Two halves
/// are not: the pairing assertion (a root `[[bin]]` dropped without its
/// package, or a package added while the root binary survives) is checkable on
/// today's tree, and the absent-from-closure half is asserted against **every
/// workspace member**, not only the three app libs — so the day the library
/// takes a path dependency on any member other than itself, this fails. That
/// generalisation is what makes the closure half demonstrable red today
/// instead of waiting for a package that does not exist.
fn app_libs_are_not_published_and_are_not_depended_on_by_the_library() -> Result<(), String> {
    let md = metadata()?;
    let root_bins = root_package_bins(&md);
    let members = md.workspace_packages();
    let mut errors = Vec::new();
    let mut present: Vec<&str> = Vec::new();
    for a in &APPS {
        let due = !root_bins.contains(a.bin);
        let pkg = members.iter().find(|p| p.name.as_str() == a.bin);
        match (due, pkg) {
            (true, None) => errors.push(format!(
                "{}: the root package no longer declares `[[bin]] {}`, so `{}` with `[lib] {}` is \
                 DUE and must exist — a missing expected member is a failure, not a pass (§47.5)",
                a.slice, a.bin, a.dir, a.lib
            )),
            (false, Some(_)) => errors.push(format!(
                "package `{}` exists while the root package still declares `[[bin]] {}`: §47.1 \
                 drops the root binary in the same commit that adds `{}`",
                a.bin, a.bin, a.dir
            )),
            (false, None) => {}
            (true, Some(p)) => {
                present.push(a.lib);
                if !p
                    .targets
                    .iter()
                    .any(|t| t.kind.contains(&cargo_metadata::TargetKind::Lib) && t.name == a.lib)
                {
                    let libs: Vec<&str> = p
                        .targets
                        .iter()
                        .filter(|t| t.kind.contains(&cargo_metadata::TargetKind::Lib))
                        .map(|t| t.name.as_str())
                        .collect();
                    errors.push(format!(
                        "package `{}` has lib target(s) {libs:?}, expected `[lib] {}` (§21 item \
                         23: the tests link the lib, so a binary-only package cannot host them)",
                        a.bin, a.lib
                    ));
                }
                if !p.publish.as_ref().is_some_and(Vec::is_empty) {
                    errors.push(format!(
                        "package `{}` has publish = {:?}, expected `publish = false` (§21 item 23)",
                        a.bin, p.publish
                    ));
                }
            }
        }
    }
    // the absent-from-closure half, generalised past the three app libs
    let closure: BTreeSet<String> = lib_tree(false)?.into_iter().map(|(n, _, _)| n).collect();
    let mut intruders: Vec<String> = Vec::new();
    for p in &members {
        let name = p.name.as_str();
        if name != LIB && closure.contains(name) {
            intruders.push(name.to_owned());
        }
    }
    for a in &APPS {
        if closure.contains(a.lib) {
            intruders.push(a.lib.to_owned());
        }
    }
    if !intruders.is_empty() {
        intruders.sort_unstable();
        intruders.dedup();
        errors.push(format!(
            "workspace member(s) {intruders:?} are in `{LIB}`'s normal dependency closure: the \
             library depends on no application and on no other workspace member (§16.5, §21 item \
             23)"
        ));
    }
    if errors.is_empty() {
        println!(
            "app_libs_are_not_published_and_are_not_depended_on_by_the_library: {} of {} app \
             lib(s) due and present {present:?}; {} workspace member(s) absent from `{LIB}`'s \
             normal closure",
            present.len(),
            APPS.len(),
            members.len().saturating_sub(1)
        );
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

/// Every root-level public name of the library: `pub mod` idents, the leaves
/// of the root's `pub use` trees, and every `#[macro_export]` macro (which
/// lives at the crate root wherever it is written).
///
/// Derived from the source rather than hard-coded, so the facade cannot grow a
/// module that this check silently blesses or forbids.
fn library_facade_roots() -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let lib = root().join("crates/tui/src/lib.rs");
    if let Ok(file) = syn::parse_file(&read(&lib)) {
        for item in &file.items {
            match item {
                syn::Item::Mod(m) if matches!(m.vis, syn::Visibility::Public(_)) => {
                    out.insert(m.ident.to_string());
                }
                syn::Item::Use(u) if matches!(u.vis, syn::Visibility::Public(_)) => {
                    use_leaves(&u.tree, &mut out);
                }
                _ => {}
            }
        }
    }
    let Ok(re) = Regex::new(r"macro_rules!\s*([A-Za-z_][A-Za-z0-9_]*)") else {
        return out;
    };
    for file in rust_files(&root().join("crates/tui/src")) {
        let text = read(&file);
        let lines: Vec<&str> = text.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if !line.trim_start().starts_with("#[macro_export]") {
                continue;
            }
            for l in lines.iter().skip(i.saturating_add(1)).take(4) {
                if let Some(c) = re.captures(l) {
                    if let Some(m) = c.get(1) {
                        out.insert(m.as_str().to_owned());
                    }
                    break;
                }
            }
        }
    }
    out
}

/// The names a `pub use` tree brings into the crate root.
fn use_leaves(tree: &syn::UseTree, out: &mut BTreeSet<String>) {
    match tree {
        syn::UseTree::Path(p) => use_leaves(&p.tree, out),
        syn::UseTree::Name(n) => {
            out.insert(n.ident.to_string());
        }
        syn::UseTree::Rename(r) => {
            out.insert(r.rename.to_string());
        }
        syn::UseTree::Group(g) => {
            for t in &g.items {
                use_leaves(t, out);
            }
        }
        syn::UseTree::Glob(_) => {}
    }
}

/// Collects the segment an application names **immediately after** the library
/// crate: `junie_tui::author::raw::Line` yields `author`, `use junie_tui::{Id,
/// author::Foo}` yields `Id` and `author`.
///
/// Token-based, never a regex: `use junie_tui::{A, b::C}` puts the interesting
/// idents inside a brace group that a regex over `crate::(\w+)` cannot see at
/// all.
struct FacadeUse {
    krate: &'static [&'static str],
    seen: BTreeSet<String>,
}

impl FacadeUse {
    fn use_tree(&mut self, tree: &syn::UseTree) {
        match tree {
            syn::UseTree::Path(p) => {
                if self.krate.contains(&p.ident.to_string().as_str()) {
                    self.after_crate(&p.tree);
                } else {
                    self.use_tree(&p.tree);
                }
            }
            syn::UseTree::Group(g) => {
                for t in &g.items {
                    self.use_tree(t);
                }
            }
            syn::UseTree::Name(_) | syn::UseTree::Rename(_) | syn::UseTree::Glob(_) => {}
        }
    }

    fn after_crate(&mut self, tree: &syn::UseTree) {
        match tree {
            syn::UseTree::Path(p) => {
                self.seen.insert(p.ident.to_string());
            }
            syn::UseTree::Name(n) => {
                self.seen.insert(n.ident.to_string());
            }
            syn::UseTree::Rename(r) => {
                self.seen.insert(r.ident.to_string());
            }
            syn::UseTree::Glob(_) => {
                self.seen.insert("*".to_owned());
            }
            syn::UseTree::Group(g) => {
                for t in &g.items {
                    self.after_crate(t);
                }
            }
        }
    }
}

impl syn::visit::Visit<'_> for FacadeUse {
    fn visit_item_use(&mut self, node: &syn::ItemUse) {
        self.use_tree(&node.tree);
        syn::visit::visit_item_use(self, node);
    }

    fn visit_path(&mut self, node: &syn::Path) {
        let mut it = node.segments.iter();
        if let (Some(first), Some(second)) = (it.next(), it.next())
            && self.krate.contains(&first.ident.to_string().as_str())
        {
            self.seen.insert(second.ident.to_string());
        }
        syn::visit::visit_path(self, node);
    }
}

/// The crate names the library answers to: the Slice 3–4 temporary name and
/// the name it takes at the rename commit (§21 item 31, §47.1). Both are
/// scanned so this check does not go silently blind for one commit.
const LIB_CRATE_IDENTS: &[&str] = &["tui_next", "junie_tui"];

/// The first line of `text` naming `<crate>::<segment>`, 1-based.
fn first_facade_line(text: &str, segment: &str) -> usize {
    for ident in LIB_CRATE_IDENTS {
        let needle = format!("{ident}::{segment}");
        for (i, line) in text.lines().enumerate() {
            if line.contains(&needle) {
                return i.saturating_add(1);
            }
        }
    }
    for (i, line) in text.lines().enumerate() {
        if line.contains(segment) {
            return i.saturating_add(1);
        }
    }
    0
}

/// §16.5 / §47.5 / §47.8. The two thirds of "applications reach only the
/// public facade" that no other check performs.
///
/// **The third that is not here.** `cargo tree -p <app> -e normal --depth 1`
/// prints `junie-tui` and nothing else — that is asserted by
/// `dependency_graph_is_exactly_the_declared_set` item (3), whose app-package
/// loop already requires each app's direct normal dependencies to be exactly
/// the library. It is *not* duplicated here; duplicating it would produce two
/// checks that fail together and one that could quietly stop being run.
///
/// **What is here**: every path an application names under the library
/// resolves against the library's *root* public surface (`pub mod`s, root
/// `pub use` leaves, `#[macro_export]` macros — `author` and `author::raw`
/// among them), and no `#[path]` attribute or `include!` smuggles library
/// source into an application. §16.5 records that the *enforcement* is
/// structural — a separate crate cannot name a `pub(crate)` item — so this is
/// the belt-and-braces report that names the offender.
///
/// **Honest statement of what is vacuous today.** `apps/` does not exist, so
/// the path scan and the `#[path]`/`include!` prohibition have no input and
/// are **vacuous until Slice 5**; they were demonstrated red on a scratch
/// `apps/showcase` tree outside the repository, per `COORDINATION.md`. What is
/// not vacuous today is the due-set assertion: an application whose root
/// `[[bin]]` has been dropped must have an `apps/<app>/src` that this scan
/// actually read, so the check cannot pass by scanning nothing.
fn applications_depend_only_on_the_library_facade() -> Result<(), String> {
    let md = metadata()?;
    let due = due_apps(&md);
    let facade = library_facade_roots();
    let mut errors = Vec::new();
    let mut scanned = 0usize;
    let Ok(path_attr) = Regex::new(r"#\s*\[\s*path\s*=") else {
        return Err("bad #[path] regex".to_owned());
    };
    let Ok(include) = Regex::new(r"\binclude!\s*\(") else {
        return Err("bad include! regex".to_owned());
    };
    let apps_dir = root().join("apps");
    let mut app_dirs: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&apps_dir) {
        for e in entries.filter_map(Result::ok) {
            let src = e.path().join("src");
            if src.is_dir() {
                app_dirs.push(src);
            }
        }
    }
    app_dirs.sort();
    for a in &due {
        let src = root().join(a.dir).join("src");
        if !app_dirs.contains(&src) {
            errors.push(format!(
                "{}/src is missing, yet `{}` is due at {} (the root package no longer declares \
                 `[[bin]] {}`): this check must never pass by scanning nothing (§47.5)",
                a.dir, a.bin, a.slice, a.bin
            ));
        }
    }
    for dir in &app_dirs {
        for file in rust_files(dir) {
            let text = read(&file);
            let Ok(ast) = syn::parse_file(&text) else {
                errors.push(format!("{} does not parse", rel(&file)));
                continue;
            };
            scanned = scanned.saturating_add(1);
            let mut visitor = FacadeUse {
                krate: LIB_CRATE_IDENTS,
                seen: BTreeSet::new(),
            };
            syn::visit::Visit::visit_file(&mut visitor, &ast);
            for segment in &visitor.seen {
                if facade.contains(segment) {
                    continue;
                }
                errors.push(format!(
                    "{}:{}: `{segment}` is not part of the library's root facade — an application \
                     names only the crate root or `author` (§16.5, §22 §1.2)",
                    rel(&file),
                    first_facade_line(&text, segment)
                ));
            }
            for (n, line) in non_test_lines(&text) {
                let code = code_line(line);
                if path_attr.is_match(code) {
                    errors.push(format!(
                        "{}:{n}: `#[path]` is forbidden in an application — a module reached by \
                         path is not a facade consumer (§16.5)",
                        rel(&file)
                    ));
                }
                if include.is_match(code) {
                    errors.push(format!(
                        "{}:{n}: `include!` is forbidden in an application — included source \
                         bypasses the crate boundary the workspace exists to create (§16.5)",
                        rel(&file)
                    ));
                }
            }
        }
    }
    if errors.is_empty() {
        println!(
            "applications_depend_only_on_the_library_facade: {} application source file(s) \
             scanned across {} due app(s) against {} facade root(s); the `cargo tree` third is \
             asserted by dependency_graph_is_exactly_the_declared_set (3), not duplicated here",
            scanned,
            due.len(),
            facade.len()
        );
        Ok(())
    } else {
        Err(errors.join("\n"))
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

/// Scans **code lines only**, deliberately (§4(j)-5): `\bworkspace\b` and
/// `\binstance\b` appear in ordinary architectural prose ("per-instance
/// patch"), and a reflowed `///` line must not fire the check. `grep_check`
/// strips comments through `code_line`.
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
        // §22.7's broad regex (D-10): the narrowed form let a computed
        // `Color::Rgb(r, g, b)` through anywhere.
        r"Color::Rgb\(|Color::from_u32\(|#[0-9a-fA-F]{6}\b",
        &[
            "theme/builtin/junie.rs",
            "theme/builtin/paper.rs",
            "theme/builder.rs",
            "theme/downgrade.rs",
        ],
        None,
        "colour literals outside theme/builtin (R-10)",
    )
}

/// BL-2: an "unreachable" arm implemented as `loop { spin_loop() }` hangs the
/// process with raw mode on and the alternate screen entered — strictly worse
/// than a panic, because `TerminalSession`'s hook can restore a panic but not
/// a livelock.
fn no_unreachable_spin_loops() -> Result<(), String> {
    grep_check(
        &[root().join("crates/tui/src")],
        r"spin_loop|loop\s*\{\s*\}",
        &[],
        None,
        "a livelock is not an `unreachable` arm (BL-2)",
    )
}

/// §22.1 as amended: `ratatui-crossterm` is a normal, non-optional dependency
/// taken for its version-unified `crossterm` **event vocabulary**, never for
/// `CrosstermBackend`. Exactly two files may name it: `event.rs` (the
/// vocabulary) and `runtime/session.rs` (the backend).
fn ratatui_crossterm_is_named_in_exactly_two_files() -> Result<(), String> {
    let re = Regex::new(r"ratatui_crossterm\b").map_err(|e| e.to_string())?;
    let mut files: Vec<String> = Vec::new();
    for file in rust_files(&root().join("crates/tui/src")) {
        if read(&file).lines().any(|l| re.is_match(l)) {
            files.push(rel(&file));
        }
    }
    files.sort();
    let want = [
        "crates/tui/src/event.rs".to_owned(),
        "crates/tui/src/runtime/session.rs".to_owned(),
    ];
    if files == want {
        println!("ratatui-crossterm is named in {files:?}");
        Ok(())
    } else {
        Err(format!(
            "ratatui_crossterm named in {files:?}, want {want:?}"
        ))
    }
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

/// Whether a file carries the inner attribute `#![<level>(… <lint> …)]`.
///
/// Structural, because the substring form this replaced (`text.contains(
/// "#![forbid(unsafe_code)]")`) also matched the same characters inside a
/// line comment, a doc comment, a doc example or a string literal — a crate
/// that merely *documented* the attribute passed the gate. A `reason = "…"`
/// argument is consumed rather than aborting the walk, so it may precede the
/// lint.
fn file_has_inner_lint(path: &Path, level: &str, lint: &str) -> Result<bool, String> {
    let ast =
        syn::parse_file(&read(path)).map_err(|e| format!("{} does not parse: {e}", rel(path)))?;
    Ok(ast.attrs.iter().any(|a| {
        if !matches!(a.style, syn::AttrStyle::Inner(_)) || !a.path().is_ident(level) {
            return false;
        }
        let mut hit = false;
        let _ = a.parse_nested_meta(|m| {
            if m.path.is_ident(lint) {
                hit = true;
            }
            if m.input.peek(syn::Token![=]) {
                m.value()?.parse::<syn::Expr>()?;
            }
            Ok(())
        });
        hit
    }))
}

/// Whether any item — including items in inline modules — is an `unsafe impl`.
fn has_unsafe_impl(items: &[syn::Item]) -> bool {
    items.iter().any(|i| match i {
        syn::Item::Impl(im) => im.unsafety.is_some(),
        syn::Item::Mod(m) => m
            .content
            .as_ref()
            .is_some_and(|(_, inner)| has_unsafe_impl(inner)),
        _ => false,
    })
}

fn no_unsafe() -> Result<(), String> {
    let lib = root().join("crates/tui/src/lib.rs");
    if !file_has_inner_lint(&lib, "forbid", "unsafe_code")? {
        return Err("crates/tui/src/lib.rs lacks #![forbid(unsafe_code)]".to_owned());
    }
    let testing = root().join("crates/tui-testing/src/lib.rs");
    if !file_has_inner_lint(&testing, "deny", "unsafe_code")? {
        return Err("crates/tui-testing/src/lib.rs lacks #![deny(unsafe_code)]".to_owned());
    }
    let mut unsafe_files: Vec<String> = Vec::new();
    for f in rust_files(&root().join("crates/tui-testing/src")) {
        let ast =
            syn::parse_file(&read(&f)).map_err(|e| format!("{} does not parse: {e}", rel(&f)))?;
        if has_unsafe_impl(&ast.items) {
            unsafe_files.push(rel(&f));
        }
    }
    unsafe_files.sort();
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
    // constant pattern: built once, borrowed by the (cache type × file) loop below
    let state_struct =
        Regex::new(r"pub struct (\w+State)\s*\{([^}]*)\}").map_err(|e| e.to_string())?;
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
            for m in state_struct.captures_iter(t) {
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

/// Finds a declared function of a given name anywhere in a file — free,
/// inherent, trait item or trait impl item.
struct FnNamed<'a> {
    want: &'a str,
    found: bool,
}

impl<'ast> syn::visit::Visit<'ast> for FnNamed<'_> {
    fn visit_signature(&mut self, sig: &'ast syn::Signature) {
        if sig.ident == self.want {
            self.found = true;
        }
        syn::visit::visit_signature(self, sig);
    }
}

/// Whether any item — including items in inline modules — declares the named
/// trait.
fn declares_trait(items: &[syn::Item], want: &str) -> bool {
    items.iter().any(|i| match i {
        syn::Item::Trait(t) => t.ident == want,
        syn::Item::Mod(m) => m
            .content
            .as_ref()
            .is_some_and(|(_, inner)| declares_trait(inner, want)),
        _ => false,
    })
}

fn no_boolean_capability_parameter_on_grid() -> Result<(), String> {
    let grid = root().join("crates/tui/src/components/grid.rs");
    if !grid.exists() {
        return Ok(());
    }
    // parsed, not grepped: `text.contains("fn editable(")` missed
    // `fn editable (`, `fn editable<T>(` and a signature broken across lines,
    // and matched a comment naming the forbidden shape
    let ast =
        syn::parse_file(&read(&grid)).map_err(|e| format!("{} does not parse: {e}", rel(&grid)))?;
    let mut hunt = FnNamed {
        want: "editable",
        found: false,
    };
    syn::visit::visit_file(&mut hunt, &ast);
    if hunt.found {
        return Err("crates/tui/src/components/grid.rs declares fn editable".to_owned());
    }
    for f in rust_files(&root().join("crates/tui/src")) {
        let ast =
            syn::parse_file(&read(&f)).map_err(|e| format!("{} does not parse: {e}", rel(&f)))?;
        if declares_trait(&ast.items, "GridCellActions") {
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
    // constant pattern: built once, borrowed by the `bitflags! {` scan below
    let struct_re = Regex::new(r"pub struct (\w+)")
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
            let ty = struct_re
                .captures(body)
                .map(|c| c[1].to_owned())
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

/// Well-known **foreign** members reachable through the facade's re-exports.
///
/// This table is for ratatui / crossterm API only. Legacy pre-refactor names
/// (`Theme::row`, `Theme::gutter`, `Interaction::pressed`,
/// `Interaction::focus_hidden`) used to be listed here as if they were
/// foreign API, which hid them; they are explicit entries in
/// `xtask/doc_check_allow.txt` instead (MA-14).
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
                "set_span",
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
        ("Constraint", &["Length", "Ratio"][..]),
        ("Margin", &["new"][..]),
        ("CellWidth", &["cell_width"][..]),
        ("Backend", &["Error", "size"][..]),
        ("Layout", &["split", "horizontal", "vertical"][..]),
        ("Terminal", &["insert_before"][..]),
        ("Style", &["underline_color"][..]),
        ("Self", &["PARTS", "State", "Action", "Cmd"][..]),
        ("Ident", &["method"][..]),
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
    // §3–§17 and §21–§26
    let mut out = String::new();
    let mut keep = false;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("## ") {
            let n: Option<u32> = rest.split('.').next().and_then(|s| s.trim().parse().ok());
            // §24–§26 carry the M-, K- and correction-pass amendments; the
            // range stopped at §23 and left them unchecked (MA-14, F23)
            keep = matches!(n, Some(3..=17 | 21..=26));
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

// ─────────────── bless-guard (§16.3, §20.10, §36.5) ───────────────

/// The kind of baseline a path is, which fixes how its lines become keys.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum BaselineKind {
    /// `name w h theme color hash`. The key is the line minus its **last**
    /// whitespace field — exactly `Baseline::parse`'s rule in
    /// `crates/tui-testing/src/digest.rs`. Any other rule makes the guard and
    /// the writer disagree about what a key is.
    Digest,
    /// `name ns allocs bytes [hits ring]`. The key is the name and the compared
    /// value **excludes the `ns` column**: `crates/tui/tests/perf_baseline.txt`
    /// records that timings are re-measured per machine and asserted only under
    /// `PERF_STRICT`, so a guard that fired on timing noise would be suppressed,
    /// and a suppressed guard is a dead one.
    Perf,
    /// Frozen pre-refactor evidence (`baseline/before/**`, `tests/baselines/**`,
    /// `tests/showcase_baseline.txt`, the root `tests/perf_baseline.txt`). Never
    /// parsed into keys: any change fails, and the remedy is `git checkout --`,
    /// not a ledger entry. Much of it is binary, so change is read from git's
    /// name-status rather than from a text comparison.
    Frozen,
}

/// Which baseline rule a repository-relative path obeys, or `None` if it is not
/// a baseline at all. Patterns, never a hard-coded file list:
/// `apps/*/tests/baselines/*.txt` do not exist yet and arrive in Slices 5–7.
fn classify_baseline(rel: &str) -> Option<BaselineKind> {
    let frozen = rel.starts_with("baseline/before/")
        || rel == "tests/showcase_baseline.txt"
        || rel == "tests/perf_baseline.txt"
        || (rel.starts_with("tests/baselines/") && rel.ends_with(".txt"));
    if frozen {
        return Some(BaselineKind::Frozen);
    }
    if rel.ends_with("/tests/perf_baseline.txt") {
        return Some(BaselineKind::Perf);
    }
    if rel.ends_with(".txt") && rel.contains("/tests/baselines/") {
        return Some(BaselineKind::Digest);
    }
    None
}

/// A baseline file's `key -> compared value` map under its own rule. Blank and
/// `#` lines are skipped, as `Baseline::parse` skips them.
fn parse_baseline_entries(kind: BaselineKind, text: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        match kind {
            BaselineKind::Digest => {
                if let Some((key, hash)) = line.rsplit_once(' ') {
                    out.insert(key.to_owned(), hash.to_owned());
                }
            }
            BaselineKind::Perf => {
                let mut fields = line.split_whitespace();
                let Some(name) = fields.next() else { continue };
                let _ns = fields.next(); // re-measured per machine; never compared
                out.insert(name.to_owned(), fields.collect::<Vec<&str>>().join(" "));
            }
            // frozen evidence is compared as a whole file, never by key
            BaselineKind::Frozen => {}
        }
    }
    out
}

/// A baseline key whose recorded value changed.
#[derive(Debug, PartialEq, Eq)]
struct Movement {
    file: String,
    key: String,
    old: String,
    new: String,
}

/// A baseline key recorded for the first time.
#[derive(Debug, PartialEq, Eq)]
struct Addition {
    file: String,
    key: String,
}

/// The moved and added sets of one baseline file. A key present in the base and
/// absent now is a *removal*, which §36.5 does not make classifiable and which
/// this therefore does not report.
fn diff_baseline(
    file: &str,
    kind: BaselineKind,
    base: &str,
    work: &str,
) -> (Vec<Movement>, Vec<Addition>) {
    let before = parse_baseline_entries(kind, base);
    let after = parse_baseline_entries(kind, work);
    let mut moved = Vec::new();
    let mut added = Vec::new();
    for (key, value) in &after {
        match before.get(key) {
            Some(old) if old == value => {}
            Some(old) => moved.push(Movement {
                file: file.to_owned(),
                key: key.clone(),
                old: old.clone(),
                new: value.clone(),
            }),
            None => added.push(Addition {
                file: file.to_owned(),
                key: key.clone(),
            }),
        }
    }
    (moved, added)
}

/// What a §20.10 row's `{scope: …}` tag says the item is allowed to account
/// for (§49.3).
///
/// **Fail-closed.** The absence of a tag is `MonoOnly`, so items 1–19 keep
/// exactly the behaviour §36.5 shipped and every widening is an explicit,
/// reviewable edit in `COMPONENT_ARCHITECTURE.md`. There is deliberately no
/// flag, allow-list or environment variable that reaches this decision: §35.2
/// named the suppression class and §49.3 rejected reintroducing it.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
enum ItemScope {
    /// No `{scope: …}` tag. The item may account for mono movements only; a
    /// moved `truecolor` key citing it is refused.
    #[default]
    MonoOnly,
    /// `{scope: truecolor}` — the item may account for a moved `truecolor`
    /// key. Items 7, 11, 16 and 17 each anticipate truecolor movement and
    /// could not otherwise ever be discharged.
    TrueColor,
    /// `{scope: first-generation}` — the item covers the **first** recording
    /// of a key only. A *moved* key citing it is refused outright, which is
    /// what makes item 19's "may not be cited again for the same key"
    /// machine-checked rather than merely read.
    FirstGeneration,
    /// A `{scope: …}` tag whose value this guard does not know. Treated as
    /// `MonoOnly` for every refusal and additionally reported, so a typo
    /// widens nothing.
    Unrecognised,
}

/// Every item number declared in `COMPONENT_ARCHITECTURE.md` §20.10, with the
/// scope its row declares.
///
/// §20.10 is **five** tables — items 1–16, then 17, 18, 19 and 20 as separate
/// one-row tables. A parser that takes "the first table after the heading"
/// silently gets 1–16 and then rejects a correct citation of item 19, the item
/// that governs the first-generation digests. This scans the whole section for
/// table rows whose first cell is a number, so a further split costs nothing.
///
/// The scope tag is read from the whole row, not from a fixed cell, because
/// the tag belongs to the sentence that describes the change and the row's
/// cell boundaries are not stable.
fn visual_change_items(doc: &str) -> BTreeMap<u32, ItemScope> {
    let scope_re = Regex::new(r"\{\s*scope\s*:\s*([A-Za-z-]+)\s*\}").ok();
    let mut out = BTreeMap::new();
    let mut inside = false;
    for line in doc.lines() {
        if line.starts_with("### 20.10") {
            inside = true;
            continue;
        }
        if !inside {
            continue;
        }
        if line.starts_with("## ") || line.starts_with("### ") {
            break;
        }
        let trimmed = line.trim();
        if !trimmed.starts_with('|') {
            continue;
        }
        let cell = trimmed
            .trim_start_matches('|')
            .split('|')
            .next()
            .unwrap_or("")
            .trim();
        if let Ok(n) = cell.parse::<u32>() {
            let scope = scope_re
                .as_ref()
                .and_then(|re| re.captures(trimmed))
                .and_then(|c| c.get(1))
                .map_or(ItemScope::MonoOnly, |m| match m.as_str() {
                    "truecolor" => ItemScope::TrueColor,
                    "first-generation" => ItemScope::FirstGeneration,
                    _ => ItemScope::Unrecognised,
                });
            out.insert(n, scope);
        }
    }
    out
}

/// Every **declared** blocking marker in `COMPONENT_ARCHITECTURE.md`, as
/// `(1-based line number, the line)` (§49.4).
///
/// A marker is a section `**Status:` line that reads `BLOCKS the … bless`.
/// Only status lines are scanned: §49.4 itself quotes the marker in prose and
/// names its own pattern in backticks, and a scan of every line would match
/// those two and could never be discharged — a check that cannot go green is
/// as useless as one that cannot go red.
///
/// **What this proves, exactly.** It proves a blocker was *declared* and is
/// still standing. It does **not** prove that an undeclared blocker was
/// respected, and nothing here should be read as evidence of ordering: §36.5
/// is right that a committed tree is a state and not a history. A lane that
/// never writes the marker defeats this check completely; what the check buys
/// is that a blocker someone did write down cannot then be walked past
/// silently, because discharging it means editing one sentence in a
/// single-writer file, which is visible in a diff.
fn blocking_bless_markers(doc: &str) -> Result<Vec<(usize, String)>, String> {
    // `\bBLOCKS\b` is case-sensitive on purpose: "Unblocks the §39 re-bless"
    // is the *discharge* wording used by the same headers and must not match.
    // The `?` is why this returns a Result rather than an empty vector: a
    // refusal that degrades to "found nothing" when its own pattern fails to
    // build is a gate that cannot fail, which is the class this guard exists
    // inside.
    let marker = Regex::new(r"\bBLOCKS\b.*\bbless\b")
        .map_err(|e| format!("blocking-marker pattern does not compile: {e}"))?;
    let mut out = Vec::new();
    for (i, line) in doc.lines().enumerate() {
        let trimmed = line.trim();
        if !trimmed.trim_start_matches('*').starts_with("Status:") {
            continue;
        }
        if marker.is_match(&trimmed.replace('*', "")) {
            out.push((i.saturating_add(1), trimmed.to_owned()));
        }
    }
    Ok(out)
}

/// Expands `{a,b,c}` alternation groups into the full cross product, so one
/// matrix-generated key set is one ledger claim instead of 896 transcribed
/// lines. Whitespace inside a produced key is normalised to single spaces,
/// which is the form a baseline line uses.
fn expand_key_pattern(pattern: &str) -> Result<Vec<String>, String> {
    let mut out = vec![String::new()];
    let mut rest = pattern;
    loop {
        let Some(open) = rest.find('{') else {
            for s in &mut out {
                s.push_str(rest);
            }
            break;
        };
        let Some(close) = rest.get(open..).and_then(|r| r.find('}')).map(|i| open + i) else {
            return Err(format!("unbalanced `{{` in key pattern `{pattern}`"));
        };
        let head = rest.get(..open).unwrap_or_default();
        let alts: Vec<&str> = rest
            .get(open.saturating_add(1)..close)
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .collect();
        let mut next = Vec::new();
        for prefix in &out {
            for alt in &alts {
                let mut s = prefix.clone();
                s.push_str(head);
                s.push_str(alt);
                next.push(s);
            }
        }
        if next.len() > 100_000 {
            return Err(format!(
                "key pattern `{pattern}` expands past 100000 keys; write the keys out"
            ));
        }
        out = next;
        rest = rest.get(close.saturating_add(1)..).unwrap_or_default();
    }
    Ok(out
        .into_iter()
        .map(|s| s.split_whitespace().collect::<Vec<&str>>().join(" "))
        .filter(|s| !s.is_empty())
        .collect())
}

/// One classified entry of `docs/visual-changes.md`: a fenced block beneath a
/// `## Item <n>` heading. The entry-format block at the head of the ledger sits
/// **above** the first such heading and is therefore not an entry.
#[derive(Debug, Default)]
struct LedgerEntry {
    /// `Item 18 / 18b` — for messages only.
    label: String,
    /// The numbered §20.10 item this entry cites.
    cited: Option<u32>,
    class: Option<String>,
    /// The text left of the arrow on each `- moved:` claim line.
    moved: Vec<String>,
    moved_declared: Option<usize>,
    /// The expanded key claims of `- added:`.
    added: Vec<String>,
    added_declared: Option<usize>,
    /// Defects in the entry's own shape, reported only when it is engaged.
    defects: Vec<String>,
}

/// The `- name:` fields of one fenced entry block. A line that does not start a
/// field continues the previous one, which is how the ledger writes its
/// multi-line `- moved:` lists.
fn block_fields(block: &str) -> BTreeMap<String, String> {
    let mut out: BTreeMap<String, String> = BTreeMap::new();
    let mut current = String::new();
    for line in block.lines() {
        let mut started = false;
        if let Some(rest) = line.trim_start().strip_prefix("- ")
            && let Some((name, value)) = rest.split_once(':')
            && !name.is_empty()
            && name.chars().all(|c| c.is_ascii_lowercase() || c == '_')
        {
            current = name.to_owned();
            let field = out.entry(current.clone()).or_default();
            field.push_str(value.trim());
            field.push('\n');
            started = true;
        }
        if !started && !current.is_empty() {
            let field = out.entry(current.clone()).or_default();
            field.push_str(line.trim());
            field.push('\n');
        }
    }
    out
}

/// Whether a field's first line declares the empty set.
fn declares_none(field: &str) -> bool {
    field
        .lines()
        .next()
        .is_some_and(|l| l.trim_start().starts_with("none"))
}

/// The `N keys` / `N lines` count a field declares about itself, if any.
fn declared_count(field: &str) -> Option<usize> {
    let re = Regex::new(r"\b(\d+)\s+(?:keys?|lines?|entries|entry)\b").ok()?;
    re.captures(field)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse().ok())
}

/// The `<key> <old> → <new>` claim lines of a `- moved:` field: the text left
/// of the arrow, which is `<key> <old>` under the same "line minus its last
/// field" rule the writer uses. A line without an arrow is prose.
fn moved_claims(field: &str) -> Vec<String> {
    field
        .lines()
        .filter_map(|l| {
            let (left, right) = l.split_once('→').or_else(|| l.split_once("->"))?;
            let left = left.trim();
            if left.is_empty() || right.trim().is_empty() {
                return None;
            }
            Some(left.to_owned())
        })
        .collect()
}

/// The key claims of an `- added:` field: every backticked span, expanded
/// through `{a,b}` alternation. Backticks mark the claim so prose in the same
/// field cannot be mistaken for a key.
fn added_claims(field: &str) -> (Vec<String>, Vec<String>) {
    let mut keys = Vec::new();
    let mut defects = Vec::new();
    let mut rest = field;
    while let Some(open) = rest.find('`') {
        let after = rest.get(open.saturating_add(1)..).unwrap_or_default();
        let Some(end) = after.find('`') else { break };
        let span = after.get(..end).unwrap_or_default().trim();
        match expand_key_pattern(span) {
            Ok(expanded) => keys.extend(expanded),
            Err(e) => defects.push(e),
        }
        rest = after.get(end.saturating_add(1)..).unwrap_or_default();
    }
    (keys, defects)
}

/// Every entry of `docs/visual-changes.md`.
fn parse_ledger(ledger: &str) -> Vec<LedgerEntry> {
    let reason_re = Regex::new(r"§20\.10 item (\d+)").ok();
    let mut out = Vec::new();
    let mut item: Option<u32> = None;
    let mut sub = String::new();
    let mut in_fence = false;
    let mut block = String::new();
    for line in ledger.lines() {
        if line.trim_start().starts_with("```") {
            if in_fence {
                if let Some(n) = item {
                    let label = if sub.is_empty() {
                        format!("Item {n}")
                    } else {
                        format!("Item {n} / {sub}")
                    };
                    out.push(entry_from_block(&block, n, &label, reason_re.as_ref()));
                }
                block.clear();
            }
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            block.push_str(line);
            block.push('\n');
            continue;
        }
        if let Some(rest) = line.strip_prefix("## Item ") {
            let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
            item = digits.parse().ok();
            sub.clear();
        } else if line.starts_with("## ") {
            item = None;
            sub.clear();
        } else if let Some(rest) = line.strip_prefix("### ") {
            sub = rest.split('—').next().unwrap_or(rest).trim().to_owned();
        }
    }
    out
}

/// One entry, from its fenced block and the heading it sits beneath.
fn entry_from_block(
    block: &str,
    heading_item: u32,
    label: &str,
    reason_re: Option<&Regex>,
) -> LedgerEntry {
    let fields = block_fields(block);
    let empty = String::new();
    let moved_field = fields.get("moved").unwrap_or(&empty);
    let added_field = fields.get("added").unwrap_or(&empty);
    let reason = fields.get("reason").unwrap_or(&empty);
    let (added, mut defects) = added_claims(added_field);
    let moved = moved_claims(moved_field);
    let moved_none = declares_none(moved_field);
    let added_none = declares_none(added_field);
    if !moved_none && moved.is_empty() {
        defects.push(
            "`- moved:` is neither `none` nor one or more `<key> <old> → <new>` lines".to_owned(),
        );
    }
    if !added_none && added.is_empty() {
        defects
            .push("`- added:` is neither `none` nor one or more backticked key claims".to_owned());
    }
    let class = fields
        .get("class")
        .and_then(|f| f.split_whitespace().next())
        .map(str::to_ascii_lowercase);
    let cited = reason_re
        .and_then(|re| re.captures(reason))
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse().ok())
        .or(Some(heading_item));
    LedgerEntry {
        label: label.to_owned(),
        cited,
        class,
        moved,
        moved_declared: declared_count(moved_field),
        added,
        added_declared: declared_count(added_field),
        defects,
    }
}

/// Whether a `- moved:` claim accounts for `key`: the claim is `<key> <old>`,
/// so it either equals the key or is the key followed by its old value.
fn claim_covers(claim: &str, key: &str) -> bool {
    claim == key || claim.strip_prefix(key).is_some_and(|r| r.starts_with(' '))
}

/// The classification report. Pure: everything it needs arrives as strings.
///
/// **Ordering is not checked here and cannot be checked anywhere.** §36.5: a
/// committed tree is a state, not a history, so nothing in this guard proves
/// the classification was written before the bless. Do not read a green
/// `baseline_moves_are_classified` as evidence of the fixed order
/// change → capture → classify → bless. What it does prove is *completeness* —
/// the accounted key set equals the moved-and-added key set — plus the citation
/// and the two refusals.
fn report_baseline_moves(
    items: &BTreeMap<u32, ItemScope>,
    entries: &[LedgerEntry],
    moved: &[Movement],
    added: &[Addition],
) -> Result<(), String> {
    let mut problems: Vec<String> = Vec::new();
    if items.is_empty() {
        return Err(
            "COMPONENT_ARCHITECTURE.md §20.10 declares no numbered items; the citation check has \
             nothing to resolve against, which would let any citation pass"
                .to_owned(),
        );
    }

    // which entries account for which keys
    let mut engaged: Vec<bool> = vec![false; entries.len()];
    let mut unaccounted: Vec<String> = Vec::new();
    for m in moved {
        let mut covered = false;
        for (i, e) in entries.iter().enumerate() {
            if e.moved.iter().any(|c| claim_covers(c, &m.key)) {
                covered = true;
                if let Some(slot) = engaged.get_mut(i) {
                    *slot = true;
                }
            }
        }
        if !covered {
            unaccounted.push(format!(
                "  moved, unaccounted: {} :: {} ({} → {})",
                m.file, m.key, m.old, m.new
            ));
        }
    }
    for a in added {
        let mut covered = false;
        for (i, e) in entries.iter().enumerate() {
            if e.added.iter().any(|c| c == &a.key) {
                covered = true;
                if let Some(slot) = engaged.get_mut(i) {
                    *slot = true;
                }
            }
        }
        if !covered {
            unaccounted.push(format!("  added, unaccounted: {} :: {}", a.file, a.key));
        }
    }

    let moved_keys: BTreeSet<&str> = moved.iter().map(|m| m.key.as_str()).collect();
    let added_keys: BTreeSet<&str> = added.iter().map(|a| a.key.as_str()).collect();
    for (i, e) in entries.iter().enumerate() {
        if !engaged.get(i).copied().unwrap_or(false) {
            continue;
        }
        for d in &e.defects {
            problems.push(format!("  {}: {d}", e.label));
        }
        match e.cited {
            None => problems.push(format!("  {}: cites no numbered §20.10 item", e.label)),
            Some(n) if !items.contains_key(&n) => problems.push(format!(
                "  {}: cites §20.10 item {n}, which does not exist (§20.10 declares {:?})",
                e.label,
                items.keys().copied().collect::<Vec<u32>>()
            )),
            Some(n) if items.get(&n) == Some(&ItemScope::Unrecognised) => problems.push(format!(
                "  {}: cites §20.10 item {n}, whose row declares a `{{scope: …}}` tag that is \
                 neither `truecolor` nor `first-generation`. An unknown tag widens nothing and \
                 is reported rather than ignored (§49.3, fail-closed).",
                e.label
            )),
            Some(_) => {}
        }
        match e.class.as_deref() {
            Some("intended" | "fix") => {}
            Some("regression") => problems.push(format!(
                "  {}: `- class: regression` — a regression must be fixed, never blessed",
                e.label
            )),
            Some(other) => problems.push(format!(
                "  {}: `- class: {other}` is not one of intended | fix | regression",
                e.label
            )),
            None => problems.push(format!("  {}: has no `- class:` field", e.label)),
        }
        for c in &e.moved {
            if !moved_keys.iter().any(|k| claim_covers(c, k)) {
                problems.push(format!(
                    "  {}: `- moved:` claims `{c}`, which the diff did not move",
                    e.label
                ));
            }
        }
        for c in &e.added {
            if !added_keys.contains(c.as_str()) {
                problems.push(format!(
                    "  {}: `- added:` claims `{c}`, which the diff did not add",
                    e.label
                ));
            }
        }
        if let Some(n) = e.moved_declared
            && n != e.moved.len()
        {
            problems.push(format!(
                "  {}: `- moved:` declares {n} but lists {}",
                e.label,
                e.moved.len()
            ));
        }
        if let Some(n) = e.added_declared
            && n != e.added.len()
        {
            problems.push(format!(
                "  {}: `- added:` declares {n} but its key patterns expand to {}",
                e.label,
                e.added.len()
            ));
        }
    }

    if unaccounted.is_empty() && problems.is_empty() {
        return Ok(());
    }
    let mut msg = String::from(
        "docs/visual-changes.md does not account for this tree's baseline diff (§16.3, §20.10, \
         §36.5). Every moved and added key must be listed by an entry that cites a numbered \
         §20.10 item; co-presence of an entry is not enough.\n",
    );
    let shown = unaccounted.len().min(20);
    for line in unaccounted.iter().take(shown) {
        msg.push_str(line);
        msg.push('\n');
    }
    if unaccounted.len() > shown {
        msg.push_str(&format!(
            "  … and {} more unaccounted key(s)\n",
            unaccounted.len().saturating_sub(shown)
        ));
    }
    for line in &problems {
        msg.push_str(line);
        msg.push('\n');
    }
    Err(msg)
}

/// The whole guard over in-memory strings.
///
/// `files` is `(repository-relative path, base text, working text)` for every
/// non-frozen baseline; `frozen_changed` is the frozen evidence git reports as
/// touched, which is read from name-status rather than content because
/// `baseline/before/**` is binary.
fn evaluate_bless_guard(
    doc: &str,
    ledger: &str,
    files: &[(String, String, String)],
    frozen_changed: &[String],
) -> Result<(), String> {
    let mut refusals: Vec<String> = frozen_changed
        .iter()
        .map(|p| {
            format!(
                "  {p}: frozen pre-refactor evidence changed. Revert it (`git checkout -- {p}`); \
                 do not classify it."
            )
        })
        .collect();
    let mut moved: Vec<Movement> = Vec::new();
    let mut added: Vec<Addition> = Vec::new();
    let mut digest_files: BTreeSet<String> = BTreeSet::new();
    for (path, base, work) in files {
        let Some(kind) = classify_baseline(path) else {
            continue;
        };
        if kind == BaselineKind::Frozen {
            continue;
        }
        if kind == BaselineKind::Digest {
            digest_files.insert(path.clone());
        }
        let (m, a) = diff_baseline(path, kind, base, work);
        moved.extend(m);
        added.extend(a);
    }
    println!(
        "baseline_moves_are_classified: {} moved key(s), {} added key(s) across {} baseline file(s)",
        moved.len(),
        added.len(),
        files.len()
    );

    let items = visual_change_items(doc);
    let entries = parse_ledger(ledger);
    refusals.extend(refuse_while_a_bless_is_blocked(
        doc,
        &moved,
        &added,
        &digest_files,
    )?);
    refusals.extend(refuse_moved_digest_keys(
        &items,
        &entries,
        &moved,
        &digest_files,
    ));

    if !refusals.is_empty() {
        let mut msg = String::from("baseline changes refused outright (§36.5, §49.3, §49.4):\n");
        for r in &refusals {
            msg.push_str(r);
            msg.push('\n');
        }
        return Err(msg);
    }
    report_baseline_moves(&items, &entries, &moved, &added)
}

/// §49.4: refuse **any** digest addition or movement while
/// `COMPONENT_ARCHITECTURE.md` declares a live `BLOCKS the … bless` marker.
///
/// Coarse on purpose. A blocker on the bless is a blocker on the bless, so no
/// attempt is made to decide which keys a particular blocker covers — §36.5 is
/// right that ordering is not provable on a committed tree, and a check that
/// tried to scope the marker would be inventing the history it cannot read.
///
/// **What a green result proves, and only this: that no blocker was left
/// standing in the document.** It does not prove that an undeclared blocker
/// was respected, and it is not evidence about the order in which the tree
/// reached its state. Its whole force is that discharging it means editing one
/// sentence in a single-writer file, which a reviewer sees in the diff, rather
/// than walking past a marker silently — which is what happened in the
/// incident §49.4 records.
fn refuse_while_a_bless_is_blocked(
    doc: &str,
    moved: &[Movement],
    added: &[Addition],
    digest_files: &BTreeSet<String>,
) -> Result<Vec<String>, String> {
    let touched = moved.iter().any(|m| digest_files.contains(&m.file))
        || added.iter().any(|a| digest_files.contains(&a.file));
    if !touched {
        return Ok(Vec::new());
    }
    Ok(blocking_bless_markers(doc)?
        .into_iter()
        .map(|(line, text)| {
            // the status line carries a whole paragraph after the marker; quote
            // enough to identify the sentence and no more
            let quoted: String = text.chars().take(120).collect();
            let ellipsis = if text.chars().count() > 120 {
                "…"
            } else {
                ""
            };
            format!(
                "  COMPONENT_ARCHITECTURE.md:{line}: a blocking marker is still declared — \
                 \"{quoted}{ellipsis}\". No digest baseline key may be added or moved while it \
                 stands (§49.4). Discharge it by editing that `Status:` line in the same commit \
                 that lands the blocking change."
            )
        })
        .collect())
}

/// §49.3's two refusals over the moved digest keys.
///
/// 1. A moved `truecolor` key is refused **unless** an entry accounting for it
///    cites a §20.10 item whose row declares `{scope: truecolor}`. §36.5
///    justified the old unconditional refusal by §20.10's closing clause, but
///    that clause's predicate is *not on this list*, not *truecolor*; the two
///    coincide only while every numbered item is mono-only, and items 7, 11,
///    16 and 17 each anticipate truecolor movement. Refusing unconditionally
///    refuses those four items' own discharge, permanently.
/// 2. A moved key whose entry cites a `{scope: first-generation}` item is
///    refused outright, whatever its colour. That is item 19's "may not be
///    cited again for the same key", machine-checked instead of read.
///
/// Untagged items stay mono-only, so items 1–19 keep exactly today's
/// behaviour and widening one is an explicit edit in a single-writer file.
fn refuse_moved_digest_keys(
    items: &BTreeMap<u32, ItemScope>,
    entries: &[LedgerEntry],
    moved: &[Movement],
    digest_files: &BTreeSet<String>,
) -> Vec<String> {
    let mut out = Vec::new();
    for mv in moved {
        if !digest_files.contains(&mv.file) {
            continue;
        }
        let cited: Vec<(u32, ItemScope)> = entries
            .iter()
            .filter(|e| e.moved.iter().any(|c| claim_covers(c, &mv.key)))
            .filter_map(|e| e.cited)
            .map(|n| (n, items.get(&n).copied().unwrap_or_default()))
            .collect();
        for (n, scope) in &cited {
            if *scope == ItemScope::FirstGeneration {
                out.push(format!(
                    "  {}: `{}` MOVED ({} → {}) and the entry accounting for it cites §20.10 \
                     item {n}, whose row declares `{{scope: first-generation}}`. That item covers \
                     the FIRST recording of a key only and may not be cited again for the same \
                     key (§20.10 item 19, §49.2), so the movement is refused outright.",
                    mv.file, mv.key, mv.old, mv.new
                ));
            }
        }
        if mv.key.rsplit(' ').next() != Some("truecolor") {
            continue;
        }
        if cited.iter().any(|(_, s)| *s == ItemScope::TrueColor) {
            continue;
        }
        let who = if cited.is_empty() {
            "no ledger entry accounts for it".to_owned()
        } else {
            format!(
                "the entry accounting for it cites §20.10 item(s) {:?}, none of which declares \
                 that tag",
                cited.iter().map(|(n, _)| *n).collect::<Vec<u32>>()
            )
        };
        out.push(format!(
            "  {}: `{}` MOVED ({} → {}) and its colour field is `truecolor`. A moved truecolor \
             key is refused unless the entry accounting for it cites a §20.10 item whose row \
             declares `{{scope: truecolor}}`; {who}. An item with no `{{scope: …}}` tag is \
             mono-only, and §20.10's closing clause makes anything not on the list a regression \
             by construction (§49.3).",
            mv.file, mv.key, mv.old, mv.new
        ));
    }
    out
}

// ── the git and filesystem half ──

fn git(args: &[&str]) -> Result<std::process::Output, String> {
    Command::new("git")
        .args(args)
        .current_dir(root())
        .output()
        .map_err(|e| format!("git {}: {e}", args.join(" ")))
}

/// `git show <rev>:<path>`, or the empty string when the path did not exist.
fn git_show(rev: &str, path: &str) -> String {
    match git(&["show", &format!("{rev}:{path}")]) {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).into_owned(),
        _ => String::new(),
    }
}

fn resolve_rev(rev: &str, source: &str) -> Result<String, String> {
    let out = git(&[
        "rev-parse",
        "--verify",
        "--quiet",
        &format!("{rev}^{{commit}}"),
    ])?;
    if out.status.success() {
        return Ok(rev.to_owned());
    }
    Err(format!(
        "bless-guard base revision `{rev}` (from {source}) does not resolve. Falling back to HEAD \
         here would compare the tree with itself and pass vacuously, which is the failure this \
         gate exists to prevent, so the guard stops instead. In CI the checkout needs \
         `fetch-depth: 0`; locally set BLESS_GUARD_BASE to a revision that exists."
    ))
}

/// Resolve the guard base from the explicit CI variable, or from the pull
/// request base ref. Missing both is an error: falling back to `HEAD` would
/// compare the tree with itself and pass vacuously.
fn bless_guard_base_from(
    explicit: Option<&str>,
    github_base_ref: Option<&str>,
) -> Result<String, String> {
    if let Some(v) = explicit.filter(|v| !v.trim().is_empty()) {
        return resolve_rev(v.trim(), "BLESS_GUARD_BASE");
    }
    if let Some(v) = github_base_ref.filter(|v| !v.trim().is_empty()) {
        return resolve_rev(&format!("origin/{}", v.trim()), "GITHUB_BASE_REF");
    }
    Err(
        "bless-guard has no base revision. Set BLESS_GUARD_BASE explicitly (or provide GITHUB_BASE_REF on a pull request); comparing against HEAD is refused because it passes vacuously. CI checkouts must use `fetch-depth: 0`."
            .to_owned(),
    )
}

fn bless_guard_base() -> Result<String, String> {
    let explicit = std::env::var("BLESS_GUARD_BASE").ok();
    let github_base_ref = std::env::var("GITHUB_BASE_REF").ok();
    bless_guard_base_from(explicit.as_deref(), github_base_ref.as_deref())
}

/// `git diff -M --name-status <base>`: `(new path -> base path)` for renames,
/// and every baseline path the diff touches. `-M` is why Slice 5's move of the
/// app baselines into `apps/` does not fire hundreds of spurious entries.
fn diff_name_status(base: &str) -> Result<(BTreeMap<String, String>, BTreeSet<String>), String> {
    let out = git(&["diff", "-M", "--name-status", base, "--"])?;
    if !out.status.success() {
        return Err(format!(
            "git diff -M --name-status {base} failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    let mut renames = BTreeMap::new();
    let mut touched = BTreeSet::new();
    for line in text.lines() {
        let mut fields = line.split('\t');
        let Some(status) = fields.next() else {
            continue;
        };
        let Some(first) = fields.next() else { continue };
        match fields.next() {
            Some(second) if status.starts_with('R') || status.starts_with('C') => {
                renames.insert(second.to_owned(), first.to_owned());
                touched.insert(second.to_owned());
                touched.insert(first.to_owned());
            }
            _ => {
                touched.insert(first.to_owned());
            }
        }
    }
    Ok((renames, touched))
}

/// Untracked files, so a baseline that has never been committed is still seen.
fn untracked_files() -> Result<BTreeSet<String>, String> {
    let out = git(&["ls-files", "--others", "--exclude-standard"])?;
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::to_owned)
        .collect())
}

/// Every baseline file on disk, by pattern rather than by name. A discovery
/// that stopped matching would find nothing to compare and the guard would exit
/// 0 forever; `the_baseline_inventory_is_not_empty` is the defence against that.
fn discover_baselines() -> BTreeSet<String> {
    WalkDir::new(root())
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            name != "target" && name != ".git"
        })
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .map(|e| rel(e.path()))
        .filter(|r| classify_baseline(r).is_some())
        .collect()
}

/// §16.3 as amended by §36, §20.10 and §36.5: every moved or added baseline key
/// is accounted for by a `docs/visual-changes.md` entry citing a numbered
/// §20.10 item, frozen evidence is never touched, a moved `truecolor` key needs
/// an item explicitly scoped for truecolor, and a first-generation item cannot
/// account for a moved key.
fn baseline_moves_are_classified() -> Result<(), String> {
    let base = bless_guard_base()?;
    let (renames, touched) = diff_name_status(&base)?;
    let untracked = untracked_files()?;
    let mut paths = discover_baselines();
    paths.extend(touched.iter().cloned());
    paths.extend(untracked.iter().cloned());
    paths.retain(|p| classify_baseline(p).is_some());

    let mut frozen_changed: Vec<String> = Vec::new();
    let mut files: Vec<(String, String, String)> = Vec::new();
    for path in &paths {
        if classify_baseline(path) == Some(BaselineKind::Frozen) {
            if touched.contains(path) || untracked.contains(path) {
                frozen_changed.push(path.clone());
            }
            continue;
        }
        let base_path = renames.get(path).cloned().unwrap_or_else(|| path.clone());
        files.push((
            path.clone(),
            git_show(&base, &base_path),
            read(&root().join(path)),
        ));
    }
    println!("baseline_moves_are_classified: base {base}");
    evaluate_bless_guard(
        &read(&root().join("COMPONENT_ARCHITECTURE.md")),
        &read(&root().join("docs/visual-changes.md")),
        &files,
        &frozen_changed,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// MA-2: the scan used to stop at the first `#[cfg(test)]`, so everything
    /// after a mid-file test helper was invisible to all forbidden-pattern
    /// rules. It must skip exactly the attributed item and carry on.
    #[test]
    fn non_test_lines_skips_only_the_cfg_test_item() {
        let src = "\
a();
#[cfg(test)]
fn helper() {
    let s = \"}\";
    inner();
}
b();
#[cfg(test)]
const K: u8 = 1;
c();
#[cfg(test)]
mod tests {
    fn t() {}
}
";
        let kept: Vec<&str> = non_test_lines(src).into_iter().map(|(_, l)| l).collect();
        assert_eq!(kept, vec!["a();", "b();", "c();"]);
        let lines: Vec<usize> = non_test_lines(src).into_iter().map(|(n, _)| n).collect();
        assert_eq!(lines, vec![1, 7, 10]);
    }

    // ── bless-guard (§16.3, §20.10, §36.5) ──

    /// A §20.10 stand-in that reproduces the real document's **five** tables.
    const DOC: &str = "\
### 20.10 Intentional visual changes

| # | Change | Why | How it is reviewed |
|---|---|---|---|
| 1 | **Mono legibility fallbacks** | … | … |
| 2 | **Layer compositing order** | … | … |

| 17 | **`Anchor::Point` flips** | … | … |


| 18 | **Mono `DISABLED` gains `DIM`** | … | … |

| 19 | **First-generation digests**; may not be cited again for the same key. \
`{scope: first-generation}` | … | … |

| 20 | **Forcing stops erasing the props half.** `{scope: truecolor}` | … | … |

## Appendix A — Slice plan
";

    /// The same §20.10, in a document that still declares a live blocking
    /// marker on its own status line — the shape §49.4 records the bless
    /// commit's tree as having had.
    const DOC_BLOCKED: &str = "\
## §39 Adjudication — forcing substitutes for the runtime

**Status: accepted. BLOCKS the §36 first-generation bless — see §39.4.** Fresh adjudication.

### 20.10 Intentional visual changes

| # | Change | Why | How it is reviewed |
|---|---|---|---|
| 1 | **Mono legibility fallbacks** | … | … |

## Appendix A — Slice plan
";

    const BASE_BASELINE: &str = "\
# digest baseline: name w h theme color hash
render::components::tabs::pressed 120 40 junie mono 5517de00b23ac747
render::components::tabs::pressed 120 40 junie truecolor aaaaaaaaaaaaaaaa
";

    const WORK_BASELINE: &str = "\
# digest baseline: name w h theme color hash
render::components::tabs::pressed 120 40 junie mono 8531aef99ed82a7c
render::components::tabs::pressed 120 40 junie truecolor aaaaaaaaaaaaaaaa
";

    const LEDGER_WITHOUT_ENTRY: &str = "\
# Visual changes ledger

## Item 1 — Mono legibility fallbacks

captures / classification: `(pending — filled when the change lands)`
";

    const LEDGER_WITH_ENTRY: &str = "\
# Visual changes ledger

## Item 1 — Mono legibility fallbacks

### 1a — `Tabs` paints §11.4's mono `PRESSED` bracket

```
- surface:   tui-next/tabs/pressed @ 120x40 / junie / mono
- captures:  none under `shots/` — headless `Scene` matrix; frame-text dump attached
- tests:     crates/tui/tests/baselines/components.txt
- moved:     1 line, every one `mono`:
  render::components::tabs::pressed 120 40 junie mono 5517de00b23ac747 → 8531aef99ed82a7c
- added:     none
- class:     fix
- reason:    §20.10 item 1 (mono legibility fallbacks).
```
";

    /// An entry that cites the `{scope: truecolor}` item and claims both the
    /// mono and the truecolor movement of the same key.
    const LEDGER_ITEM_20_TRUECOLOR: &str = "\
# Visual changes ledger

## Item 20 — forcing stops erasing the props half

```
- surface:   tui-next/tabs/pressed @ 120x40 / junie
- captures:  none under `shots/` — headless `Scene` matrix; frame-text dump attached
- tests:     crates/tui/tests/baselines/components.txt
- moved:     2 lines:
  render::components::tabs::pressed 120 40 junie mono 5517de00b23ac747 → 8531aef99ed82a7c
  render::components::tabs::pressed 120 40 junie truecolor aaaaaaaaaaaaaaaa → bbbbbbbbbbbbbbbb
- added:     none
- class:     fix
- reason:    §20.10 item 20 (forcing stops erasing the props half).
```
";

    /// An entry that cites the `{scope: first-generation}` item for a key the
    /// diff **moved** — the second citation item 19 forbids.
    const LEDGER_ITEM_19_FIRST_GEN: &str = "\
# Visual changes ledger

## Item 19 — first-generation component digests

```
- surface:   tui-next/tabs/pressed @ 120x40 / junie / mono
- captures:  none under `shots/` — headless `Scene` matrix; frame-text dump attached
- tests:     crates/tui/tests/baselines/components.txt
- moved:     1 line:
  render::components::tabs::pressed 120 40 junie mono 5517de00b23ac747 → 8531aef99ed82a7c
- added:     none
- class:     intended
- reason:    §20.10 item 19 (first-generation digests).
```
";

    fn one_file(base: &str, work: &str) -> Vec<(String, String, String)> {
        vec![(
            "crates/tui/tests/baselines/components.txt".to_owned(),
            base.to_owned(),
            work.to_owned(),
        )]
    }

    /// The red half of the COORDINATION.md demonstration: a moved key with no
    /// ledger entry accounting for it must fail. A guard hard-wired to `Ok(())`
    /// fails this test.
    #[test]
    fn a_moved_baseline_without_a_ledger_entry_fails() {
        let err = evaluate_bless_guard(
            DOC,
            LEDGER_WITHOUT_ENTRY,
            &one_file(BASE_BASELINE, WORK_BASELINE),
            &[],
        )
        .expect_err("an unclassified move must fail");
        assert!(
            err.contains("render::components::tabs::pressed 120 40 junie mono"),
            "the failure must name the key: {err}"
        );
    }

    /// The green half: the same input with the entry present passes. A guard
    /// hard-wired to `Err` fails this test.
    #[test]
    fn the_same_move_with_a_ledger_entry_passes() {
        evaluate_bless_guard(
            DOC,
            LEDGER_WITH_ENTRY,
            &one_file(BASE_BASELINE, WORK_BASELINE),
            &[],
        )
        .expect("the same move, classified, must pass");
    }

    #[test]
    fn an_unchanged_tree_reports_no_moves() {
        let (moved, added) = diff_baseline(
            "crates/tui/tests/baselines/components.txt",
            BaselineKind::Digest,
            BASE_BASELINE,
            BASE_BASELINE,
        );
        assert!(moved.is_empty() && added.is_empty(), "{moved:?} {added:?}");
        evaluate_bless_guard(
            DOC,
            LEDGER_WITHOUT_ENTRY,
            &one_file(BASE_BASELINE, BASE_BASELINE),
            &[],
        )
        .expect("an unchanged tree has nothing to classify");
    }

    /// §20.10 is five tables. A parser that takes the first one gets 1–16 and
    /// then rejects a correct citation of item 19.
    #[test]
    fn the_2010_item_list_survives_the_split_tables() {
        let doc = read(&root().join("COMPONENT_ARCHITECTURE.md"));
        let items = visual_change_items(&doc);
        let want: BTreeSet<u32> = (1..=20).collect();
        assert_eq!(
            items.keys().copied().collect::<BTreeSet<u32>>(),
            want,
            "§20.10 item numbers"
        );
    }

    /// §49.3's two scope tags must actually be present on the rows the guard
    /// resolves them from, or the refusals it derives from them are decoration.
    ///
    /// Item 19 is the first-generation item: without `{scope: first-generation}`
    /// its "may not be cited again for the same key" clause is read by people
    /// and enforced by nothing. Item 20 is the first `{scope: truecolor}` item:
    /// without the tag it cannot discharge its own truecolor movement.
    #[test]
    fn the_2010_scope_tags_are_declared_where_the_guard_needs_them() {
        let doc = read(&root().join("COMPONENT_ARCHITECTURE.md"));
        let items = visual_change_items(&doc);
        assert_eq!(
            items.get(&19),
            Some(&ItemScope::FirstGeneration),
            "§20.10 item 19's row must end its `Change` cell with the literal text \
             `{{scope: first-generation}}` (COMPONENT_ARCHITECTURE.md is single-writer, so this \
             is Lane A's edit). Without it, item 19 is treated as mono-only and a second \
             movement of a first-generation key citing item 19 is refused only when the key is \
             truecolor."
        );
        assert_eq!(
            items.get(&20),
            Some(&ItemScope::TrueColor),
            "§20.10 item 20's row must carry the literal text `{{scope: truecolor}}`"
        );
    }

    /// The tag is read from the row text, wherever in the row it sits, and an
    /// untagged row is mono-only. This is the green half of
    /// `the_2010_scope_tags_are_declared_where_the_guard_needs_them`, proven
    /// on a fixture because `COMPONENT_ARCHITECTURE.md` is single-writer.
    #[test]
    fn the_scope_tag_is_read_from_the_row_text() {
        let items = visual_change_items(DOC);
        assert_eq!(items.get(&1), Some(&ItemScope::MonoOnly), "untagged");
        assert_eq!(items.get(&18), Some(&ItemScope::MonoOnly), "untagged");
        assert_eq!(items.get(&19), Some(&ItemScope::FirstGeneration));
        assert_eq!(items.get(&20), Some(&ItemScope::TrueColor));
    }

    /// An unrecognised tag must widen nothing and must be reported, not
    /// silently read as "untagged".
    #[test]
    fn an_unrecognised_scope_tag_is_mono_only_and_reported() {
        let doc = DOC.replace("{scope: truecolor}", "{scope: everything}");
        assert_eq!(
            visual_change_items(&doc).get(&20),
            Some(&ItemScope::Unrecognised)
        );
        // it does not widen: the truecolor movement is still refused
        let work = WORK_BASELINE.replace("aaaaaaaaaaaaaaaa", "bbbbbbbbbbbbbbbb");
        let err = evaluate_bless_guard(
            &doc,
            LEDGER_ITEM_20_TRUECOLOR,
            &one_file(BASE_BASELINE, &work),
            &[],
        )
        .expect_err("an unknown tag may not discharge the truecolor refusal");
        assert!(
            err.contains("refused unless the entry accounting for it cites"),
            "{err}"
        );
        // and it is named, rather than being read as an untagged row
        let mono_only = LEDGER_ITEM_20_TRUECOLOR
            .replace(
                "  render::components::tabs::pressed 120 40 junie truecolor aaaaaaaaaaaaaaaa →                  bbbbbbbbbbbbbbbb\n",
                "",
            )
            .replace("2 lines", "1 line");
        let err = evaluate_bless_guard(
            &doc,
            &mono_only,
            &one_file(BASE_BASELINE, WORK_BASELINE),
            &[],
        )
        .expect_err("an unknown tag is reported");
        assert!(
            err.contains("neither `truecolor` nor `first-generation`"),
            "{err}"
        );
    }

    /// §49.3, half one: a `{scope: truecolor}` item discharges its own
    /// truecolor movement. Without this the four already-numbered items that
    /// anticipate truecolor movement — 7, 11, 16 and 17 — could never be
    /// discharged at all.
    #[test]
    fn a_moved_truecolor_key_is_allowed_by_an_item_tagged_truecolor() {
        let work = WORK_BASELINE.replace("aaaaaaaaaaaaaaaa", "bbbbbbbbbbbbbbbb");
        evaluate_bless_guard(
            DOC,
            LEDGER_ITEM_20_TRUECOLOR,
            &one_file(BASE_BASELINE, &work),
            &[],
        )
        .expect("an item declaring `{scope: truecolor}` accounts for its own truecolor movement");
    }

    /// §49.3, half two: item 19's "may not be cited again for the same key",
    /// machine-checked. The key here is `mono`, so nothing but the
    /// first-generation tag can refuse it.
    #[test]
    fn a_moved_key_citing_a_first_generation_item_is_refused() {
        let err = evaluate_bless_guard(
            DOC,
            LEDGER_ITEM_19_FIRST_GEN,
            &one_file(BASE_BASELINE, WORK_BASELINE),
            &[],
        )
        .expect_err("a first-generation item may not account for a movement");
        assert!(err.contains("first-generation"), "{err}");
        assert!(err.contains("item 19"), "{err}");
    }

    /// §49.4: a declared blocker refuses every digest change while it stands.
    #[test]
    fn a_live_blocking_marker_refuses_every_digest_change() {
        let err = evaluate_bless_guard(
            DOC_BLOCKED,
            LEDGER_WITH_ENTRY,
            &one_file(BASE_BASELINE, WORK_BASELINE),
            &[],
        )
        .expect_err("a declared blocker refuses the bless");
        assert!(err.contains("blocking marker is still declared"), "{err}");
        assert!(
            err.contains("BLOCKS the §36 first-generation bless"),
            "{err}"
        );
    }

    /// …and stops refusing the moment that sentence is edited. Discharge is an
    /// edit to one line of a single-writer file, visible in the diff.
    #[test]
    fn a_discharged_blocking_marker_stops_refusing() {
        let doc = DOC_BLOCKED.replace(
            "BLOCKS the §36 first-generation bless — see §39.4.",
            "Landed; the §36 first-generation bless is unblocked (§49).",
        );
        assert!(
            blocking_bless_markers(&doc)
                .expect("pattern compiles")
                .is_empty(),
            "the discharged wording must not match"
        );
        evaluate_bless_guard(
            &doc,
            LEDGER_WITH_ENTRY,
            &one_file(BASE_BASELINE, WORK_BASELINE),
            &[],
        )
        .expect("a discharged marker refuses nothing");
    }

    /// The refusal is on the *change*, not on the marker: a tree that moves no
    /// digest key passes with the marker standing, so the check cannot be
    /// mistaken for a blanket veto on committing while a blocker exists.
    #[test]
    fn a_blocking_marker_alone_refuses_nothing() {
        evaluate_bless_guard(
            DOC_BLOCKED,
            LEDGER_WITHOUT_ENTRY,
            &one_file(BASE_BASELINE, BASE_BASELINE),
            &[],
        )
        .expect("no digest key moved or was added");
    }

    /// §49.6: the guard's base falls back to `HEAD` when nothing sets one, and
    /// CI runs on **push to `main`** as well as on pull requests. On the push
    /// leg there is no `GITHUB_BASE_REF`, so without an explicit base a clean
    /// checkout is diffed against itself and the guard reports `0 moved,
    /// 0 added` on every direct commit — which is what it did for this whole
    /// session. The push leg must therefore name a base explicitly.
    #[test]
    fn the_ci_push_leg_gives_the_bless_guard_a_base() {
        let ci = read(&root().join(".github/workflows/ci.yml"));
        assert!(
            ci.contains("  push:"),
            "the workflow no longer has a push leg; this check is about that leg"
        );
        let lines: Vec<&str> = ci.lines().collect();
        // the `run:` step, not the gate→requirement comment block at the head
        // of the file, which names the same command
        let run = lines
            .iter()
            .position(|l| {
                !l.trim_start().starts_with('#')
                    && l.contains("run: cargo run -p xtask -- bless-guard")
            })
            .expect("ci.yml has a step that runs the bless guard");
        let start = run.saturating_sub(8);
        let window = lines.get(start..run).unwrap_or_default().join("\n");
        assert!(
            window.contains("BLESS_GUARD_BASE:"),
            "the bless-guard step must set BLESS_GUARD_BASE; without it the push leg diffs \
             the tree against itself. Step context:\n{window}"
        );
        assert!(
            window.contains("github.event.before"),
            "BLESS_GUARD_BASE on the push leg must be `${{{{ github.event.before }}}}` — the \
             commit the push moved `main` off. Step context:\n{window}"
        );
    }

    #[test]
    fn missing_guard_base_is_refused_instead_of_defaulting_to_head() {
        let err = bless_guard_base_from(None, None)
            .expect_err("a guard without an explicit or pull-request base must stop");
        assert!(err.contains("no base revision"), "{err}");
        assert!(err.contains("BLESS_GUARD_BASE"), "{err}");
    }

    #[test]
    fn invalid_explicit_guard_base_does_not_fall_back_to_pull_request_base() {
        let err = bless_guard_base_from(Some("not-a-real-revision"), Some("main"))
            .expect_err("an invalid explicit base must stop");
        assert!(err.contains("not-a-real-revision"), "{err}");
        assert!(err.contains("does not resolve"), "{err}");
    }

    /// The defence against the third vacuous-pass mode: discovery patterns that
    /// stop matching find nothing to compare and the guard exits 0 forever.
    #[test]
    fn the_baseline_inventory_is_not_empty() {
        let found = discover_baselines();
        for want in [
            "crates/tui/tests/baselines/components.txt",
            "crates/tui/tests/perf_baseline.txt",
        ] {
            assert!(found.contains(want), "{want} not discovered: {found:?}");
        }
    }

    /// A moved `truecolor` key whose entry cites an **untagged** item is
    /// refused, even when the ledger otherwise accounts for it. No item before
    /// 20 carries `{scope: truecolor}`, so this is §36.5's behaviour unchanged.
    ///
    /// The assertion names the refusal sentence, not merely the word
    /// `truecolor`: with the refusal removed this input still fails the
    /// *completeness* check, so a weaker assertion would pass on a guard that
    /// had stopped refusing anything.
    #[test]
    fn a_moved_truecolor_key_is_refused_outright() {
        let work = WORK_BASELINE.replace("aaaaaaaaaaaaaaaa", "bbbbbbbbbbbbbbbb");
        let err =
            evaluate_bless_guard(DOC, LEDGER_WITH_ENTRY, &one_file(BASE_BASELINE, &work), &[])
                .expect_err("a truecolor movement is a regression by construction");
        assert!(
            err.contains("refused unless the entry accounting for it cites"),
            "{err}"
        );
    }

    /// `regression` is the ledger's own "must be fixed, never blessed".
    #[test]
    fn a_regression_class_fails_the_guard() {
        let ledger = LEDGER_WITH_ENTRY.replace("- class:     fix", "- class:     regression");
        let err = evaluate_bless_guard(DOC, &ledger, &one_file(BASE_BASELINE, WORK_BASELINE), &[])
            .expect_err("a regression may not be blessed");
        assert!(err.contains("regression"), "{err}");
    }

    /// A citation of an item §20.10 does not declare is not a citation.
    #[test]
    fn a_citation_of_a_nonexistent_2010_item_fails() {
        let ledger = LEDGER_WITH_ENTRY.replace("§20.10 item 1 ", "§20.10 item 42 ");
        let err = evaluate_bless_guard(DOC, &ledger, &one_file(BASE_BASELINE, WORK_BASELINE), &[])
            .expect_err("item 42 does not exist");
        assert!(err.contains("item 42"), "{err}");
    }

    #[test]
    fn frozen_evidence_fails_without_classification() {
        let err = evaluate_bless_guard(
            DOC,
            LEDGER_WITH_ENTRY,
            &[],
            &["tests/baselines/tablepro.txt".to_owned()],
        )
        .expect_err("frozen evidence may not change");
        assert!(err.contains("Revert it"), "{err}");
    }

    /// The perf rule: the `ns` column is re-measured per machine, so a timing
    /// difference is not a movement — but an allocation or hit-count one is.
    #[test]
    fn perf_baselines_ignore_the_ns_column_and_see_the_rest() {
        let base = "frame_showcase_lists_120x40 8275 0 0 9 7\n";
        let noise = "frame_showcase_lists_120x40 9111 0 0 9 7\n";
        let real = "frame_showcase_lists_120x40 8275 0 0 10 7\n";
        let (moved, _) =
            diff_baseline("a/tests/perf_baseline.txt", BaselineKind::Perf, base, noise);
        assert!(
            moved.is_empty(),
            "timing noise is not a movement: {moved:?}"
        );
        let (moved, _) = diff_baseline("a/tests/perf_baseline.txt", BaselineKind::Perf, base, real);
        assert_eq!(moved.len(), 1, "a hit-count change is a movement");
    }

    /// Frozen paths win over the digest and perf patterns.
    #[test]
    fn baseline_paths_are_classified_by_pattern() {
        use BaselineKind::{Digest, Frozen, Perf};
        for (path, want) in [
            ("crates/tui/tests/baselines/components.txt", Some(Digest)),
            ("apps/showcase/tests/baselines/showcase.txt", Some(Digest)),
            ("crates/tui/tests/perf_baseline.txt", Some(Perf)),
            ("tests/perf_baseline.txt", Some(Frozen)),
            ("tests/showcase_baseline.txt", Some(Frozen)),
            ("tests/baselines/tablepro.txt", Some(Frozen)),
            (
                "baseline/before/showcase_forms_default_120x40.png",
                Some(Frozen),
            ),
            ("crates/tui/src/lib.rs", None),
        ] {
            assert_eq!(classify_baseline(path), want, "{path}");
        }
    }

    /// One matrix-generated set is one claim; the declared count must match.
    #[test]
    fn a_key_pattern_expands_to_the_matrix_it_names() {
        let keys = expand_key_pattern(
            "render::components::{meter,brand}::{default,focused} {120 40,40 10} {junie} {mono}",
        )
        .expect("expands");
        assert_eq!(keys.len(), 8);
        assert!(keys.contains(&"render::components::meter::default 120 40 junie mono".to_owned()));
    }
}
