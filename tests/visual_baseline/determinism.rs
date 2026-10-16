//! Determinism audit (plan §B3, task T6), as a cheap source check in the
//! default run. A capture is only reproducible if what the apps render
//! derives from tick counts, `--frame N` and fixture clocks (holla and
//! jackin `clock.rs`, tablepro's seeded data and demo clock), never from the
//! wall clock or a random source.
//!
//! - Wall-clock dates and random sources are forbidden in non-test code
//!   under `src/` ([`FORBIDDEN`]).
//! - `Instant::now` is allowed only where it times *when* something
//!   happens, never *what* is drawn, and only as often as listed in
//!   [`INSTANT_ALLOWED`]; a new use fails until it is reviewed and listed.
//!
//! String literals and comments are ignored (a fixture may show code that
//! calls `rand::`), as are test modules (`#[cfg(test)] mod …`) and files
//! whose name contains `tests`.

use std::collections::BTreeMap;
use std::path::Path;

/// Wall-clock and randomness APIs no rendered value may depend on.
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

/// Reviewed `Instant::now` uses per file, with why they cannot change a
/// frame's content.
const INSTANT_ALLOWED: [(&str, usize, &str); 4] = [
    (
        "src/runtime.rs",
        3,
        "tick timer and input-burst coalescing: when a frame is drawn, not what it shows",
    ),
    (
        "src/core/typeahead.rs",
        1,
        "type-ahead buffer reset after a pause between keys",
    ),
    (
        "src/bin/showcase/app.rs",
        2,
        "press-flash and status-message expiry; both only expire on a tick, which --motion paused stops",
    ),
    (
        "src/bin/tablepro/app.rs",
        2,
        "press-flash and status-message expiry; the flash clock is frozen under --motion paused",
    ),
];

/// `text` with comments removed and string/char literal contents blanked.
fn code_only(text: &str) -> String {
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
                // char literal ('x', '\n', '\u{..}') or a lifetime ('a)
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

/// `pat` in `line`, not as the tail of a longer identifier (`brand::`
/// is not `rand::`).
fn contains_word(line: &str, pat: &str) -> bool {
    line.match_indices(pat).any(|(at, _)| {
        !line[..at]
            .chars()
            .next_back()
            .is_some_and(|c| c.is_alphanumeric() || c == '_')
    })
}

/// Non-test part of a source file: everything before a `#[cfg(test)]`
/// that introduces a module.
fn non_test(text: &str) -> &str {
    let mut offset = 0;
    let mut lines = text.split_inclusive('\n').peekable();
    while let Some(line) = lines.next() {
        if line.trim() == "#[cfg(test)]"
            && lines
                .peek()
                .is_some_and(|next| next.trim_start().starts_with("mod "))
        {
            return &text[..offset];
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

/// `(forbidden hits, Instant::now count per file)` over `src/`.
fn scan(root: &Path) -> (Vec<String>, BTreeMap<String, usize>) {
    let mut files = Vec::new();
    sources(&root.join("src"), &mut files);
    files.sort();
    let mut hits = Vec::new();
    let mut instants = BTreeMap::new();
    for file in files {
        let rel = file
            .strip_prefix(root)
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
        let count = code.matches("Instant::now").count();
        if count > 0 {
            instants.insert(rel, count);
        }
    }
    (hits, instants)
}

/// The default-run check.
pub fn sources_are_deterministic() {
    let (hits, instants) = scan(Path::new(env!("CARGO_MANIFEST_DIR")));
    assert!(
        hits.is_empty(),
        "wall-clock or random sources in rendering code (derive from ticks, --frame or a fixture clock instead):\n  {}",
        hits.join("\n  ")
    );
    let allowed: BTreeMap<String, usize> = INSTANT_ALLOWED
        .iter()
        .map(|(f, n, _)| (f.to_string(), *n))
        .collect();
    assert_eq!(
        instants, allowed,
        "Instant::now uses changed: review that the new use cannot change a frame's content, then update determinism::INSTANT_ALLOWED"
    );
}

/// The scanner itself, on fixed text.
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
}
