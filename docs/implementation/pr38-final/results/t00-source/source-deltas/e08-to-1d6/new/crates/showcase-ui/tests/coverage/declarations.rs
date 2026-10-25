//! Bounded source inventory, page leads, and explicit behavioral examples.
//!
//! A page/title association is a navigation hint only. A behavioral proof is
//! recorded separately and covers only the listed capability and control.

use std::collections::HashMap;

use crate::coverage::catalog::{CAPABILITIES, CapKind};

/// Explicit inventory dispositions and pending review states.
pub const ALLOWLIST: &str = include_str!("allowlist.txt");

/// A retained page/title association; this is a lead, never a behavior proof.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageLead {
    /// Current showcase page source slug.
    pub page: &'static str,
    /// Historical title hint retained for navigation only.
    pub title_hint: &'static str,
}

/// Behavioral evidence forms used by a small set of current-head controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProofKind {
    Demo,
    Option,
    StateRow,
    Via,
}

/// One explicit, bounded behavioral example. It says nothing about unlisted IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BehaviorProof {
    /// Inventory ID touched by the example.
    pub capability: &'static str,
    /// Showcase source page containing the example.
    pub page: &'static str,
    /// Current navigation label used to open the page.
    pub route: &'static str,
    /// Example form.
    pub kind: ProofKind,
    /// Observable action or output.
    pub probe: &'static str,
    /// Demonstrated owner for a `Via` method claim.
    pub owner: Option<&'static str>,
}

#[path = "declarations_data.rs"]
mod declarations_data;

/// Historical page/title associations retained as non-proving leads.
pub use declarations_data::PAGE_LEADS;
/// Small current-head behavioral examples.
pub use declarations_data::PROOFS;

/// The slug of the shell pseudo page: the app chrome every page draws.
pub const SHELL_SLUG: &str = "shell";

/// The showcase page sources, read once, with strings and comments blanked.
///
/// The `shell` pseudo page is the app chrome: the source of the app that
/// draws the navigation, the brand, the status bar and the shell dialogs.
fn page_code() -> &'static HashMap<String, String> {
    use std::sync::OnceLock;
    static SOURCES: OnceLock<HashMap<String, String>> = OnceLock::new();
    SOURCES.get_or_init(|| {
        let root = crate::coverage::scan::workspace_root();
        let dir = root
            .join("crates")
            .join("showcase-demos")
            .join("src")
            .join("pages");
        let mut out = HashMap::new();
        let Ok(entries) = std::fs::read_dir(&dir) else {
            return out;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            if let Ok(text) = std::fs::read_to_string(&path) {
                out.insert(stem.to_string(), strip_literals(&text));
            }
        }
        if let Ok(text) = std::fs::read_to_string(
            root.join("crates")
                .join("showcase-ui")
                .join("src")
                .join("app.rs"),
        ) {
            out.insert(SHELL_SLUG.to_string(), strip_literals(&text));
        }
        out
    })
}

/// The code of one showcase page, with strings and comments blanked.
pub fn page_source(page: &str) -> Option<&'static str> {
    page_code().get(page).map(String::as_str)
}

/// Blanks out string literals, char literals and comments, so a name is
/// only found where the code uses it.
pub fn strip_literals(src: &str) -> String {
    let chars: Vec<char> = src.chars().collect();
    let mut out = String::with_capacity(src.len());
    let mut i = 0;
    let blank = |out: &mut String, ch: char| out.push(if ch == '\n' { '\n' } else { ' ' });
    while i < chars.len() {
        let ch = chars[i];
        let next = chars.get(i + 1).copied();
        if ch == '/' && next == Some('/') {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
                out.push(' ');
            }
        } else if ch == '/' && next == Some('*') {
            let mut depth = 0;
            while i < chars.len() {
                if chars[i] == '/' && chars.get(i + 1) == Some(&'*') {
                    depth += 1;
                    out.push_str("  ");
                    i += 2;
                } else if chars[i] == '*' && chars.get(i + 1) == Some(&'/') {
                    depth -= 1;
                    out.push_str("  ");
                    i += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    blank(&mut out, chars[i]);
                    i += 1;
                }
            }
        } else if ch == '"' {
            out.push(' ');
            i += 1;
            while i < chars.len() {
                if chars[i] == '\\' {
                    out.push(' ');
                    if let Some(&e) = chars.get(i + 1) {
                        blank(&mut out, e);
                    }
                    i += 2;
                } else if chars[i] == '"' {
                    out.push(' ');
                    i += 1;
                    break;
                } else {
                    blank(&mut out, chars[i]);
                    i += 1;
                }
            }
        } else if ch == '\'' {
            if next == Some('\\') {
                out.push(' ');
                i += 1;
                while i < chars.len() && chars[i] != '\'' {
                    out.push(' ');
                    i += 1;
                }
                out.push(' ');
                i += 1;
            } else if chars.get(i + 2) == Some(&'\'') {
                out.push_str("   ");
                i += 3;
            } else {
                out.push(ch);
                i += 1;
            }
        } else {
            out.push(ch);
            i += 1;
        }
    }
    out
}

/// The offsets where the name appears as a whole word.
fn word_offsets(src: &str, name: &str) -> Vec<usize> {
    let bytes = src.as_bytes();
    let needle = name.as_bytes();
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(offset) = src[from..].find(name) {
        let start = from + offset;
        let end = start + needle.len();
        let before = start.checked_sub(1).map_or(b' ', |i| bytes[i]);
        let after = bytes.get(end).copied().unwrap_or(b' ');
        let word_before = before.is_ascii_alphanumeric() || before == b'_';
        let word_after = after.is_ascii_alphanumeric() || after == b'_';
        if !word_before && !word_after {
            out.push(start);
        }
        from = end;
    }
    out
}

/// The names a source imports with a `use` statement.
fn use_names(src: &str) -> HashMap<String, ()> {
    let mut out = HashMap::new();
    let bytes = src.as_bytes();
    let mut from = 0;
    while let Some(offset) = src[from..].find("use ") {
        let start = from + offset;
        let before = start.checked_sub(1).map_or(b' ', |i| bytes[i]);
        if before.is_ascii_alphanumeric() || before == b'_' {
            from = start + 4;
            continue;
        }
        let mut end = start + 4;
        let mut depth = 0i32;
        while end < bytes.len() {
            match bytes[end] {
                b'{' => depth += 1,
                b'}' => depth -= 1,
                b';' if depth <= 0 => break,
                _ => {}
            }
            end += 1;
        }
        let statement = &src[start + 4..end.min(src.len())];
        for segment in statement.split([',', '{', '}']) {
            let segment = segment.trim();
            if segment.is_empty() || segment.starts_with('*') {
                continue;
            }
            let mut parts = segment.split_whitespace();
            let path = parts.next().unwrap_or_default();
            if path == "as" {
                continue;
            }
            let mut name = path.rsplit("::").next().unwrap_or(path);
            if let Some("as") = parts.next() {
                name = parts.next().unwrap_or(name);
            }
            if name.is_empty() || name == "self" || name == "crate" || name == "super" {
                continue;
            }
            out.insert(name.to_string(), ());
        }
        from = end.max(start + 4);
    }
    out
}

/// Whether the name appears in the source as a type: in a path, in a type
/// position, or in a use list.
fn mentions_type(src: &str, name: &str) -> bool {
    let bytes = src.as_bytes();
    for start in word_offsets(src, name) {
        let end = start + name.len();
        let before = start.checked_sub(1).map_or(b' ', |i| bytes[i]);
        let after = bytes.get(end).copied().unwrap_or(b' ');
        let next_is_path = after == b':';
        let next_is_generic = after == b'<';
        let next_is_literal = after == b'{';
        let prev_is_type_position = matches!(
            before,
            b':' | b',' | b'<' | b'(' | b'=' | b'&' | b'>' | b'|' | b';'
        );
        if next_is_path || next_is_generic || next_is_literal || prev_is_type_position {
            return true;
        }
    }
    use_names(src).contains_key(name)
}

/// Whether the name appears in the source as a call.
fn mentions_call(src: &str, name: &str) -> bool {
    let bytes = src.as_bytes();
    for start in word_offsets(src, name) {
        let end = start + name.len();
        let rest = src[end..].trim_start();
        if rest.starts_with('(') {
            let before = start.checked_sub(1).map_or(b' ', |i| bytes[i]);
            if before != b':' || bytes.get(end.wrapping_sub(2)) == Some(&b':') {
                return true;
            }
        }
    }
    false
}

/// The catalogue types, by id.
fn catalogue_types() -> &'static HashMap<&'static str, &'static str> {
    use std::sync::OnceLock;
    static TYPES: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    TYPES.get_or_init(|| {
        CAPABILITIES
            .iter()
            .filter(|cap| cap.kind == CapKind::Type)
            .map(|cap| (cap.id, cap.id.rsplit("::").next().unwrap_or(cap.id)))
            .collect()
    })
}

/// Lexical source-use hint only; this does not prove rendered behavior.
pub fn page_uses(page: &str, id: &str, kind: CapKind) -> bool {
    let Some(src) = page_source(page) else {
        return false;
    };
    let (owner, name) = match id.rsplit_once("::") {
        Some((owner, name)) => (owner, name),
        None => ("", id),
    };
    let owner_short = owner.rsplit("::").next().unwrap_or(owner);
    match kind {
        CapKind::Type => mentions_type(src, name),
        CapKind::Variant => src.contains(&format!("{owner_short}::{name}")),
        CapKind::Builder | CapKind::Function => {
            mentions_call(src, name)
                && match owner_is_type(owner) {
                    Some(owner_name) => mentions_type(src, owner_name),
                    None => true,
                }
        }
        CapKind::Token => {
            mentions_type(src, name)
                || src.contains(&format!(".{name}"))
                || src.contains(&format!("::{name}"))
        }
    }
}

/// The type name of the owner, when the owner is a catalogue type.
fn owner_is_type(owner: &str) -> Option<&'static str> {
    catalogue_types().get(owner).copied()
}

/// One allowlist line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Allow<'a> {
    /// The capability id.
    pub id: &'a str,
    /// Why the showcase does not show it.
    pub reason: &'a str,
    /// An existing roadmap step (`SCn`) or explicit pending `review`.
    pub step: &'a str,
}

/// Parses the allowlist file.
pub fn parse_allowlist(text: &str) -> (Vec<Allow<'_>>, Vec<String>) {
    let mut out = Vec::new();
    let mut problems = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split('|').map(str::trim).collect();
        let [id, reason, step] = parts[..] else {
            problems.push(format!(
                "allowlist line {}: expected `id | reason | SCn|review`: {line}",
                n + 1
            ));
            continue;
        };
        if reason.is_empty() {
            problems.push(format!("allowlist line {}: {id} has no reason", n + 1));
        }
        let valid_step = step == "review"
            || step
                .strip_prefix("SC")
                .is_some_and(|d| !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit()));
        if !valid_step {
            problems.push(format!(
                "allowlist line {}: {id} has an unknown disposition (got {step:?}); \
                 use `SCn` or `review`",
                n + 1
            ));
        }
        out.push(Allow { id, reason, step });
    }
    (out, problems)
}
