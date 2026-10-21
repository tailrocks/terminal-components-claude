//! Production-source audit for Escape hints.
//!
//! The frozen reference has one intentional exception: Jackin's cancel
//! action is labelled `Keep launching`. The baseline Dialog contract tests
//! that exact visible label at `src/widgets/dialog.rs:1987-1997` in source
//! commit `4169783ae883788186d161af09ba9c481bd84e5c`. The current Jackin
//! presentation source carries two matching Escape hints. Keep this exception
//! tied to that source file and label; every other non-verb label fails.

use std::path::{Path, PathBuf};

const VERBS: [&str; 9] = [
    "cancel", "close", "back", "clear", "done", "discard", "exit", "release", "restore",
];

const BASELINE_EXCEPTION_SOURCE: &str = "jackin-preview-app/src/app/launch_cockpit.rs";
const BASELINE_EXCEPTION_LABEL: &str = "Keep launching";
const BASELINE_EXCEPTION_COUNT: usize = 2;
const BASELINE_COMMIT: &str = "4169783ae883788186d161af09ba9c481bd84e5c";
const BASELINE_CANCEL_LABEL_ASSERTION: &str =
    "←→ Choose · Enter Keep launching · Esc Keep launching";

#[derive(Debug, PartialEq, Eq)]
struct EscLabel {
    path: PathBuf,
    line: usize,
    label: String,
}

fn source_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            if path.file_name().is_some_and(|name| name == "src") {
                rust_sources_in(&path, out);
            } else if path
                .file_name()
                .is_some_and(|name| !matches!(name.to_str(), Some("target" | "tests" | "examples")))
            {
                source_files(&path, out);
            }
        }
    }
}

fn rust_sources_in(src: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(src).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            if path.file_name().is_some_and(|name| name != "tests") {
                rust_sources_in(&path, out);
            }
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

/// Preserve Rust string literals while blanking line and block comments.
/// This keeps doc examples and comments from becoming apparent call sites.
fn without_comments(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut out = bytes.to_vec();
    let mut i = 0;
    while i < bytes.len() {
        if let Some(end) = raw_string_end(bytes, i) {
            i = end;
        } else if bytes[i] == b'"' {
            i = quoted_end(bytes, i, b'"');
        } else if let Some(end) = char_literal_end(bytes, i) {
            i = end;
        } else if bytes[i..].starts_with(b"//") {
            let start = i;
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            blank(&mut out, start, i);
        } else if bytes[i..].starts_with(b"/*") {
            let start = i;
            i += 2;
            let mut depth = 1usize;
            while i < bytes.len() && depth > 0 {
                if bytes[i..].starts_with(b"/*") {
                    depth += 1;
                    i += 2;
                } else if bytes[i..].starts_with(b"*/") {
                    depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            blank(&mut out, start, i);
        } else {
            i += 1;
        }
    }
    String::from_utf8(out).expect("comment masking preserves valid UTF-8")
}

fn blank(bytes: &mut [u8], start: usize, end: usize) {
    for byte in &mut bytes[start..end] {
        if *byte != b'\n' {
            *byte = b' ';
        }
    }
}

fn raw_string_end(bytes: &[u8], start: usize) -> Option<usize> {
    if bytes.get(start) != Some(&b'r') {
        return None;
    }
    let mut quote = start + 1;
    while bytes.get(quote) == Some(&b'#') {
        quote += 1;
    }
    if bytes.get(quote) != Some(&b'"') {
        return None;
    }
    let hashes = quote - start - 1;
    let closing_hashes = vec![b'#'; hashes];
    let mut cursor = quote + 1;
    while cursor < bytes.len() {
        if bytes[cursor] == b'"'
            && bytes
                .get(cursor + 1..)
                .is_some_and(|suffix| suffix.starts_with(&closing_hashes))
        {
            return Some(cursor + hashes + 1);
        }
        cursor += 1;
    }
    Some(bytes.len())
}

fn quoted_end(bytes: &[u8], start: usize, quote: u8) -> usize {
    let mut cursor = start + 1;
    let mut escaped = false;
    while cursor < bytes.len() {
        let byte = bytes[cursor];
        cursor += 1;
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == quote {
            break;
        }
    }
    cursor
}

fn char_literal_end(bytes: &[u8], start: usize) -> Option<usize> {
    if bytes.get(start) != Some(&b'\'') {
        return None;
    }
    let mut cursor = start + 1;
    match bytes.get(cursor).copied()? {
        b'\\' => {
            cursor += 1;
            match bytes.get(cursor).copied()? {
                b'x' => cursor += 3,
                b'u' if bytes.get(cursor + 1) == Some(&b'{') => {
                    cursor += 2;
                    while bytes.get(cursor).is_some_and(|byte| *byte != b'}') {
                        cursor += 1;
                    }
                    cursor += 1;
                }
                _ => cursor += 1,
            }
        }
        _ => {
            let tail = std::str::from_utf8(bytes.get(cursor..)?).ok()?;
            cursor += tail.chars().next()?.len_utf8();
        }
    }
    (bytes.get(cursor) == Some(&b'\'')).then_some(cursor + 1)
}

/// Blank `#[cfg(test)]` modules before scanning so their fixtures cannot
/// satisfy or fail the production vocabulary gate.
fn without_test_modules(source: &str) -> String {
    let comments_removed = without_comments(source);
    let code = code_only(&comments_removed);
    let mut out = comments_removed.into_bytes();
    let mut search_from = 0;
    while let Some(relative) = code[search_from..].find("#[cfg(test)]") {
        let attribute = search_from + relative;
        let Some((inline, end)) = test_module_end(&code, attribute) else {
            search_from = attribute + "#[cfg(test)]".len();
            continue;
        };
        if inline {
            let Some(close) = matching_brace(&code, end) else {
                search_from = end + 1;
                continue;
            };
            blank(&mut out, attribute, close + 1);
            search_from = close + 1;
        } else {
            blank(&mut out, attribute, end + 1);
            search_from = end + 1;
        }
    }
    String::from_utf8(out).expect("test module masking preserves valid UTF-8")
}

/// Return whether the cfg(test) module is inline and the `{` or `;` ending
/// its declaration header. Only inspect that header; never search into the
/// next Rust item when an external module ends with `;`.
fn test_module_end(code: &str, attribute: usize) -> Option<(bool, usize)> {
    let bytes = code.as_bytes();
    let mut item = attribute + "#[cfg(test)]".len();
    loop {
        while bytes.get(item).is_some_and(u8::is_ascii_whitespace) {
            item += 1;
        }
        if bytes.get(item..item + 2) != Some(b"#[") {
            break;
        }
        item = matching_delimiter(code, item + 1, b'[', b']')? + 1;
    }

    while bytes.get(item).is_some_and(u8::is_ascii_whitespace) {
        item += 1;
    }
    if code[item..].starts_with("pub")
        && bytes
            .get(item + 3)
            .is_none_or(|byte| byte.is_ascii_whitespace() || *byte == b'(')
    {
        item += 3;
        while bytes.get(item).is_some_and(u8::is_ascii_whitespace) {
            item += 1;
        }
        if bytes.get(item) == Some(&b'(') {
            item = matching_delimiter(code, item, b'(', b')')? + 1;
            while bytes.get(item).is_some_and(u8::is_ascii_whitespace) {
                item += 1;
            }
        }
    }
    if !code[item..].starts_with("mod") || !bytes.get(item + 3).is_some_and(u8::is_ascii_whitespace)
    {
        return None;
    }
    let mut cursor = item + 3;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'{' => return Some((true, cursor)),
            b';' => return Some((false, cursor)),
            _ => cursor += 1,
        }
    }
    None
}

fn matching_delimiter(code: &str, open: usize, left: u8, right: u8) -> Option<usize> {
    let bytes = code.as_bytes();
    if bytes.get(open) != Some(&left) {
        return None;
    }
    let mut depth = 0usize;
    for (index, byte) in bytes.iter().copied().enumerate().skip(open) {
        match byte {
            byte if byte == left => depth += 1,
            byte if byte == right => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

/// Return code punctuation with comments and string/character literals blanked.
fn code_only(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut out = bytes.to_vec();
    let mut i = 0;
    while i < bytes.len() {
        if let Some(end) = raw_string_end(bytes, i) {
            blank(&mut out, i, end);
            i = end;
        } else if bytes[i] == b'"' {
            let end = quoted_end(bytes, i, b'"');
            blank(&mut out, i, end);
            i = end;
        } else if let Some(end) = char_literal_end(bytes, i) {
            blank(&mut out, i, end);
            i = end;
        } else {
            i += 1;
        }
    }
    String::from_utf8(out).expect("code masking preserves valid UTF-8")
}

fn matching_brace(code: &str, open: usize) -> Option<usize> {
    let bytes = code.as_bytes();
    let mut depth = 0usize;
    for (index, byte) in bytes.iter().copied().enumerate().skip(open) {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn esc_labels(path: &Path, text: &str) -> Vec<EscLabel> {
    let text = without_test_modules(text);
    let mut found = vec![];
    let mut rest = text.as_str();
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
            found.push(EscLabel {
                path: path.to_owned(),
                line,
                label: lit[..end].to_owned(),
            });
        }
        offset += i + 6;
        rest = &rest[i + 6..];
    }
    found
}

fn is_verb(label: &str) -> bool {
    let first = label.split_whitespace().next().unwrap_or("").to_lowercase();
    VERBS.contains(&first.as_str())
}

fn is_frozen_exception(root: &Path, hit: &EscLabel) -> bool {
    hit.path
        .strip_prefix(root)
        .is_ok_and(|relative| relative == Path::new(BASELINE_EXCEPTION_SOURCE))
        && hit.label == BASELINE_EXCEPTION_LABEL
}

fn invalid_labels(root: &Path, labels: &[EscLabel]) -> Vec<String> {
    labels
        .iter()
        .filter(|hit| !is_verb(&hit.label) && !is_frozen_exception(root, hit))
        .map(|hit| format!("{}:{}: Esc {:?}", hit.path.display(), hit.line, hit.label))
        .collect()
}

fn production_labels(root: &Path) -> Vec<EscLabel> {
    let mut files = vec![];
    source_files(root, &mut files);
    files
        .into_iter()
        .flat_map(|path| {
            let text = std::fs::read_to_string(&path).unwrap();
            esc_labels(&path, &text)
        })
        .collect()
}

fn workspace_crates() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates")
        .canonicalize()
        .unwrap()
}

#[test]
fn scan_reads_hint_forms_but_ignores_docs_and_test_fixtures() {
    let path = Path::new("demo/src/hints.rs");
    let source = r##"
        /// Example: ("Esc", "Document only")
        hint("Esc", "Back to list"), ("Esc", "Outline"), hint(
            "Esc",
            "Close"), keys: &["Esc", "?"]
        #[cfg(test)]
        #[allow(dead_code)]
        pub (crate)
        mod tests
        {
            const FIXTURE: &str = r#"("Esc", "Test only")"#;
        }
        fn after_tests() { hint("Esc", "Restore"); }
    "##;
    let labels: Vec<String> = esc_labels(path, source)
        .into_iter()
        .map(|hit| hit.label)
        .collect();
    assert_eq!(labels, ["Back to list", "Outline", "Close", "Restore"]);
}

#[test]
fn external_test_module_declaration_does_not_hide_following_production_code() {
    let path = Path::new("demo/src/hints.rs");
    let source = r#"
        #[cfg(test)]
        #[allow(dead_code)]
        pub (crate) mod tests;
        fn production() { hint("Esc", "Unexpected label"); }
    "#;
    let hits = esc_labels(path, source);
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].label, "Unexpected label");
    assert_eq!(invalid_labels(Path::new("demo"), &hits).len(), 1);
}

#[test]
fn frozen_cancel_label_exception_is_exact_and_evidence_pinned() {
    assert_eq!(BASELINE_COMMIT, "4169783ae883788186d161af09ba9c481bd84e5c");
    assert_eq!(
        BASELINE_CANCEL_LABEL_ASSERTION,
        "←→ Choose · Enter Keep launching · Esc Keep launching"
    );
    let root = Path::new("crates");
    let source = Path::new("crates/jackin-preview-app/src/app/launch_cockpit.rs");
    let frozen_label = EscLabel {
        path: source.to_owned(),
        line: 52,
        label: BASELINE_EXCEPTION_LABEL.to_owned(),
    };
    assert!(is_frozen_exception(root, &frozen_label));
    assert!(invalid_labels(root, std::slice::from_ref(&frozen_label)).is_empty());
}

#[test]
fn a_new_nonverb_label_fails_even_at_the_allowlisted_source_path() {
    let root = Path::new("crates");
    let path = root.join(BASELINE_EXCEPTION_SOURCE);
    let fresh_fixture = r#"fn fresh_hint() { ("Esc", "Keep retrying"); }"#;
    let fresh_labels = esc_labels(&path, fresh_fixture);
    assert_eq!(fresh_labels.len(), 1, "negative fixture must be scanned");
    assert_eq!(
        invalid_labels(root, &fresh_labels),
        [format!("{}:1: Esc {:?}", path.display(), "Keep retrying")]
    );
}

#[test]
fn a_frozen_label_at_an_unreviewed_path_fails() {
    let root = Path::new("crates");
    let path = root.join("showcase-ui/src/page.rs");
    let source = r#"fn hint() { ("Esc", "Keep launching"); }"#;
    let hits = esc_labels(&path, source);
    assert_eq!(invalid_labels(root, &hits).len(), 1);
}

#[test]
fn every_production_esc_label_uses_a_verb_or_the_frozen_contract() {
    let root = workspace_crates();
    let labels = production_labels(&root);
    let exceptions: Vec<_> = labels
        .iter()
        .filter(|hit| is_frozen_exception(&root, hit))
        .collect();
    assert_eq!(
        exceptions.len(),
        BASELINE_EXCEPTION_COUNT,
        "the frozen cancel-label contract should remain at its two reviewed sites: {exceptions:#?}"
    );
    let bad = invalid_labels(&root, &labels);
    assert!(
        bad.is_empty(),
        "production Esc labels without verbs or the pinned frozen exception:\n{}",
        bad.join("\n")
    );
}
