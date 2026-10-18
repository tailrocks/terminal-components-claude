//! Source scanning: comment/string stripping, test-region exclusion,
//! import-alias resolution, and item-span tracking.
//!
//! The scanner is line-oriented and deliberately small. It resolves what a
//! `#[cfg(test)]` region covers, which local identifiers name the
//! component-author `Ui` or a raw `Buffer`/`Cell`, and which `fn` spans each
//! finding falls in, so rules stay contextual instead of name-based.

use std::collections::{BTreeMap, BTreeSet};

/// One physical source line with comments and string/char literals blanked.
///
/// Blanking preserves line numbers and keeps every other byte in place, so
/// downstream matchers see real code only: doc comments that mention app
/// names and string literals such as `"jackin.quit"` never match.
#[derive(Debug, Clone)]
pub struct CodeLine {
    /// 1-based line number.
    pub number: usize,
    /// The line with comments, strings, and char literals replaced by spaces.
    pub code: String,
}

/// A string literal occurrence, tracked so rules can classify literals by
/// shape (screen-shaped literals, `baselines/` references).
#[derive(Debug, Clone)]
pub struct StringSpan {
    /// 1-based start line.
    pub start_line: usize,
    /// 1-based end line (inclusive).
    pub end_line: usize,
    /// The literal bytes, without quotes or `r#` sigils.
    pub text: String,
}

/// An inclusive 1-based line range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineRange {
    pub start: usize,
    pub end: usize,
}

impl LineRange {
    pub fn contains(self, line: usize) -> bool {
        self.start <= line && line <= self.end
    }
}

/// A scanned source file.
#[derive(Debug, Clone)]
pub struct ScannedFile {
    /// Workspace-relative path with `/` separators.
    pub path: String,
    /// Crate member path (e.g. `crates/showcase-ui`).
    pub member: String,
    /// Every source line, blanked.
    pub lines: Vec<CodeLine>,
    /// String literals in source order.
    pub strings: Vec<StringSpan>,
    /// Regions excluded as test code.
    pub test_ranges: Vec<LineRange>,
    /// Production `fn` spans as `(name, range)`.
    pub functions: Vec<(String, LineRange)>,
    /// Local identifier to canonical import path (`Ui` aliases included).
    pub aliases: BTreeMap<String, String>,
    /// Value identifiers declared with a `Ui` type (`canvas: &mut Canvas`).
    pub ui_values: BTreeSet<String>,
    /// `macro_rules!` definitions as `(name, range)`.
    pub macros: Vec<(String, LineRange)>,
    /// True when the file carries a generator marker. Generated files are
    /// still scanned; the flag only attributes the findings.
    pub generated: bool,
}

impl ScannedFile {
    /// True when `line` is inside test-gated code.
    pub fn is_test_line(&self, line: usize) -> bool {
        self.test_ranges.iter().any(|range| range.contains(line))
    }

    /// The innermost production `fn` enclosing `line`, if any.
    pub fn enclosing_fn(&self, line: usize) -> Option<&str> {
        self.functions
            .iter()
            .filter(|(_, range)| range.contains(line))
            .max_by_key(|(_, range)| range.start)
            .map(|(name, _)| name.as_str())
    }

    /// True when `ident` names the component-author `Ui` in this file: the
    /// conventional `ui` receiver, a value declared with a `Ui` type, or a
    /// local import alias of `Ui`.
    pub fn is_ui_ident(&self, ident: &str) -> bool {
        ident == "ui"
            || self.ui_values.contains(ident)
            || self.aliases.get(ident).is_some_and(|path| is_ui_path(path))
    }
}

/// True for canonical paths that name the component-author draw context.
pub fn is_ui_path(path: &str) -> bool {
    path.rsplit("::").next() == Some("Ui")
}

/// True for canonical paths that name a raw ratatui buffer or cell type.
///
/// Preview-local `Cell`/`Buffer` lookalikes (holla-ui widget models) must
/// not match, so a ratatui/termrock path prefix is required.
pub fn is_raw_buffer_path(path: &str) -> bool {
    let last = path.rsplit("::").next().unwrap_or("");
    if last != "Buffer" && last != "Cell" {
        return false;
    }
    path.contains("ratatui") || path.contains("termrock") || path.contains("buffer")
}

/// Scan one source file.
pub fn scan_file(path: &str, member: &str, text: &str) -> ScannedFile {
    let mut out = ScannedFile {
        path: path.to_string(),
        member: member.to_string(),
        lines: Vec::new(),
        strings: Vec::new(),
        test_ranges: Vec::new(),
        functions: Vec::new(),
        aliases: BTreeMap::new(),
        ui_values: BTreeSet::new(),
        macros: Vec::new(),
        generated: is_generated_marker(text),
    };
    strip_lines(text, &mut out.lines, &mut out.strings);
    out.test_ranges = test_ranges(&out.lines);
    out.functions = fn_spans(&out.lines, &out.test_ranges);
    out.aliases = import_aliases(&out.lines);
    out.macros = macro_spans(&out.lines);
    out.ui_values = ui_value_idents(&out.lines, &out.test_ranges, &out.aliases);
    out
}

fn is_generated_marker(text: &str) -> bool {
    let lower = text.to_lowercase();
    lower.contains("do not edit")
        || lower.contains("@generated")
        || lower.contains("generated by")
        || lower.contains("auto-generated")
        || lower.contains("automatically generated")
}

/// Split source into blanked code lines plus string spans.
///
/// Handles `//` and nestable `/* */` comments, `"..."` with escapes,
/// `r"..."`/`r#"..."#` raw strings (multi-line), and `'x'` char literals.
/// A lone `'` (lifetimes) is left in place.
fn strip_lines(text: &str, lines: &mut Vec<CodeLine>, strings: &mut Vec<StringSpan>) {
    let chars: Vec<char> = text.chars().collect();
    let mut index = 0;
    let mut number = 1;
    let mut current = String::new();
    let mut block_depth: usize = 0;
    // (hashes, start line, collected text) while inside a raw string.
    let mut raw: Option<(usize, usize, String)> = None;
    // (start line, collected text) while inside a cooked string.
    let mut cooked: Option<(usize, String)> = None;

    let finish_line = |number: usize,
                       current: &mut String,
                       lines: &mut Vec<CodeLine>,
                       raw: &mut Option<(usize, usize, String)>,
                       cooked: &mut Option<(usize, String)>| {
        lines.push(CodeLine {
            number,
            code: std::mem::take(current),
        });
        if let Some((_, _, text)) = raw {
            text.push('\n');
        }
        if let Some((_, text)) = cooked {
            text.push('\n');
        }
    };

    while index < chars.len() {
        let rest = &chars[index..];
        // Inside a block comment: blank everything, track nesting and lines.
        if block_depth > 0 {
            if rest.starts_with(&['/', '*']) {
                block_depth += 1;
                current.push_str("  ");
                index += 2;
            } else if rest.starts_with(&['*', '/']) {
                block_depth -= 1;
                current.push_str("  ");
                index += 2;
            } else if rest[0] == '\n' {
                finish_line(number, &mut current, lines, &mut raw, &mut cooked);
                number += 1;
                index += 1;
            } else {
                current.push(' ');
                index += 1;
            }
            continue;
        }
        // Inside a raw string: blank, watch for the closing sigil.
        if let Some((hashes, start, text)) = raw.as_mut() {
            let closes = rest[0] == '"' && (0..*hashes).all(|h| rest.get(1 + h) == Some(&'#'));
            if closes {
                let skip = 1 + *hashes;
                for _ in 0..skip {
                    current.push(' ');
                }
                strings.push(StringSpan {
                    start_line: *start,
                    end_line: number,
                    text: std::mem::take(text),
                });
                raw = None;
                index += skip;
            } else if rest[0] == '\n' {
                finish_line(number, &mut current, lines, &mut raw, &mut cooked);
                number += 1;
                index += 1;
            } else {
                text.push(rest[0]);
                current.push(' ');
                index += 1;
            }
            continue;
        }
        // Inside a cooked string: blank, honor escapes.
        if let Some((start, text)) = cooked.as_mut() {
            if rest[0] == '\\' && rest.len() > 1 {
                if rest[1] == '\n' {
                    // Line-continuation escape: the newline is not content.
                    text.pop();
                    finish_line(number, &mut current, lines, &mut raw, &mut cooked);
                    number += 1;
                } else {
                    text.push(rest[0]);
                    text.push(rest[1]);
                    current.push_str("  ");
                }
                index += 2;
            } else if rest[0] == '"' {
                strings.push(StringSpan {
                    start_line: *start,
                    end_line: number,
                    text: std::mem::take(text),
                });
                cooked = None;
                current.push(' ');
                index += 1;
            } else if rest[0] == '\n' {
                finish_line(number, &mut current, lines, &mut raw, &mut cooked);
                number += 1;
                index += 1;
            } else {
                text.push(rest[0]);
                current.push(' ');
                index += 1;
            }
            continue;
        }
        // Plain code.
        if rest.starts_with(&['/', '/']) {
            while index < chars.len() && chars[index] != '\n' {
                current.push(' ');
                index += 1;
            }
        } else if rest.starts_with(&['/', '*']) {
            block_depth = 1;
            current.push_str("  ");
            index += 2;
        } else if rest[0] == '\n' {
            finish_line(number, &mut current, lines, &mut raw, &mut cooked);
            number += 1;
            index += 1;
        } else if rest[0] == '"' {
            cooked = Some((number, String::new()));
            current.push(' ');
            index += 1;
        } else if rest[0] == 'r' && matches!(rest.get(1), Some('"') | Some('#')) {
            let mut hashes = 0;
            while rest.get(1 + hashes) == Some(&'#') {
                hashes += 1;
            }
            if rest.get(1 + hashes) == Some(&'"') {
                for _ in 0..(2 + hashes) {
                    current.push(' ');
                }
                raw = Some((hashes, number, String::new()));
                index += 2 + hashes;
            } else {
                current.push(rest[0]);
                index += 1;
            }
        } else if rest[0] == '\'' && is_char_literal(rest) {
            // Blank `'x'` / `'\n'`; lifetimes stay as code.
            let mut taken = 1; // opening quote
            if rest.get(1) == Some(&'\\') {
                taken += 2;
            } else {
                taken += 1;
            }
            taken += 1; // closing quote
            for _ in 0..taken {
                current.push(' ');
            }
            index += taken;
        } else {
            current.push(rest[0]);
            index += 1;
        }
    }
    if !current.is_empty() || lines.is_empty() {
        lines.push(CodeLine {
            number,
            code: current,
        });
    }
}

/// True when `rest` starts a `'x'`-shaped char literal rather than a lifetime.
fn is_char_literal(rest: &[char]) -> bool {
    if rest.len() < 3 || rest[0] != '\'' {
        return false;
    }
    if rest[1] == '\\' {
        return rest.len() > 3 && rest[3] == '\'';
    }
    rest[2] == '\''
}

/// Line ranges excluded as test code: `#[cfg(test)]`-style items and any
/// `mod tests`/`mod test` block (defensive: all in-tree ones carry the cfg).
fn test_ranges(lines: &[CodeLine]) -> Vec<LineRange> {
    let mut ranges = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let code = lines[index].code.trim();
        if let Some(cfg) = cfg_attribute(code)
            && cfg_tests(cfg)
            && let Some(end) = skip_item(lines, index + 1)
        {
            ranges.push(LineRange {
                start: lines[index].number,
                end,
            });
            index = end;
            continue;
        } else if is_test_mod(code)
            && let Some(end) = brace_close(lines, index)
        {
            ranges.push(LineRange {
                start: lines[index].number,
                end,
            });
        }
        index += 1;
    }
    ranges
}

/// The `cfg(...)` body when the line holds a `#[cfg(...)]` attribute.
fn cfg_attribute(code: &str) -> Option<&str> {
    let start = code.find("#[cfg(")?;
    let after = &code[start + "#[cfg(".len()..];
    let mut depth = 1;
    for (offset, char) in after.char_indices() {
        if char == '(' {
            depth += 1;
        } else if char == ')' {
            depth -= 1;
            if depth == 0 {
                return Some(&after[..offset]);
            }
        }
    }
    None
}

/// True for cfg bodies that select test code: a `test` word that is not
/// negated via `not(test)`.
fn cfg_tests(body: &str) -> bool {
    let mut cleaned = body.to_string();
    while let Some(at) = cleaned.find("not(test)") {
        cleaned.replace_range(at..at + "not(test)".len(), "");
    }
    cleaned
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .any(|word| word == "test")
}

fn is_test_mod(code: &str) -> bool {
    code.split(|c: char| !c.is_alphanumeric() && c != '_')
        .collect::<Vec<_>>()
        .windows(2)
        .any(|pair| pair[0] == "mod" && (pair[1] == "tests" || pair[1] == "test"))
}

/// End line (inclusive) of the item starting at or after `from`, skipping
/// over intermediate attributes. Attributes on their own lines are part of
/// the skipped item.
fn skip_item(lines: &[CodeLine], from: usize) -> Option<usize> {
    let mut index = from;
    while index < lines.len() {
        let code = lines[index].code.trim();
        if code.is_empty() || code.starts_with("#[") {
            index += 1;
            continue;
        }
        return item_end(lines, index);
    }
    None
}

/// End line (inclusive) of the item starting at `at`: the matching close
/// brace when the item opens one, else the terminating `;`.
fn item_end(lines: &[CodeLine], at: usize) -> Option<usize> {
    brace_close(lines, at).or_else(|| {
        lines[at..]
            .iter()
            .find(|line| line.code.contains(';'))
            .map(|line| line.number)
    })
}

/// End line (inclusive) of the brace block opened at or after `at`.
fn brace_close(lines: &[CodeLine], at: usize) -> Option<usize> {
    let mut depth: i32 = 0;
    let mut opened = false;
    for line in &lines[at..] {
        for char in line.code.chars() {
            if char == '{' {
                depth += 1;
                opened = true;
            } else if char == '}' {
                depth -= 1;
                if opened && depth == 0 {
                    return Some(line.number);
                }
            }
        }
    }
    None
}

/// Production `fn` spans as `(name, range)`, outermost and nested.
fn fn_spans(lines: &[CodeLine], test_ranges: &[LineRange]) -> Vec<(String, LineRange)> {
    let mut spans = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        if test_ranges.iter().any(|r| r.contains(line.number)) {
            continue;
        }
        if let Some(name) = fn_name(&line.code) {
            let end = item_end(lines, index).unwrap_or(line.number);
            spans.push((
                name,
                LineRange {
                    start: line.number,
                    end,
                },
            ));
        }
    }
    spans
}

/// The declared name when the line starts a `fn` item.
fn fn_name(code: &str) -> Option<String> {
    let words: Vec<&str> = code
        .split(|c: char| !c.is_alphanumeric() && c != '_' && c != ':')
        .filter(|w| !w.is_empty())
        .collect();
    let mut index = 0;
    while index < words.len() {
        if words[index] == "fn" && index + 1 < words.len() {
            let name = words[index + 1].trim_matches(':');
            if !name.is_empty() && name != "_" {
                return Some(name.to_string());
            }
        }
        index += 1;
    }
    None
}

/// `macro_rules!` spans as `(name, range)`, delimiter-aware.
fn macro_spans(lines: &[CodeLine]) -> Vec<(String, LineRange)> {
    let mut spans = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let Some(name) = macro_def_name(&line.code) else {
            continue;
        };
        if let Some(end) = macro_end(lines, index) {
            spans.push((
                name,
                LineRange {
                    start: line.number,
                    end,
                },
            ));
        }
    }
    spans
}

/// The macro name when the line opens a `macro_rules!` definition.
fn macro_def_name(code: &str) -> Option<String> {
    let at = code.find("macro_rules!")?;
    let after = code[at + "macro_rules!".len()..].trim_start();
    let name: String = after
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if name.is_empty() {
        return None;
    }
    let tail = after[name.len()..].trim_start();
    if tail.starts_with('{') || tail.starts_with('(') || tail.starts_with('[') {
        Some(name)
    } else {
        None
    }
}

/// End line (inclusive) of the `macro_rules!` body: the delimiter opened
/// after the name can be `{...}`, `(...)`, or `[...]`.
fn macro_end(lines: &[CodeLine], at: usize) -> Option<usize> {
    let mut open: Option<char> = None;
    let mut close = '}';
    let mut depth = 0;
    let mut opened = false;
    for line in &lines[at..] {
        for char in line.code.chars() {
            if open.is_none() {
                if char == '{' || char == '(' || char == '[' {
                    open = Some(char);
                    close = match char {
                        '{' => '}',
                        '(' => ')',
                        _ => ']',
                    };
                    depth = 1;
                    opened = true;
                }
                continue;
            }
            if Some(char) == open {
                depth += 1;
            } else if char == close {
                depth -= 1;
                if opened && depth == 0 {
                    return Some(line.number);
                }
            }
        }
    }
    None
}

/// Value identifiers declared with a `Ui` type in production code:
/// `canvas: &mut Canvas` (params, `let` bindings) where `Canvas` is `Ui`
/// or a file-local `Ui` alias. Single `:` annotations only: `::` paths,
/// struct literals, labels, and bounds never match.
fn ui_value_idents(
    lines: &[CodeLine],
    test_ranges: &[LineRange],
    aliases: &BTreeMap<String, String>,
) -> BTreeSet<String> {
    let mut idents = BTreeSet::new();
    for line in lines {
        if test_ranges.iter().any(|r| r.contains(line.number)) {
            continue;
        }
        let code = line.code.as_bytes();
        let mut index = 0;
        while index < code.len() {
            if code[index] != b':' {
                index += 1;
                continue;
            }
            if code.get(index + 1) == Some(&b':') || (index > 0 && code[index - 1] == b':') {
                index += 1;
                continue;
            }
            if let Some(type_name) = ref_type_name(&line.code[index + 1..]) {
                let is_ui = type_name == "Ui"
                    || aliases.get(&type_name).is_some_and(|path| is_ui_path(path));
                if is_ui && let Some(name) = ident_before(&line.code[..index]) {
                    idents.insert(name);
                }
            }
            index += 1;
        }
    }
    idents
}

/// The referenced type name at the start of an annotation: `Canvas` in
/// `&mut Canvas<'_>`, `Ui` in `&termrock::author::Ui`. Non-reference
/// types return `None`: `Ui` travels by reference in this codebase.
fn ref_type_name(after: &str) -> Option<String> {
    let after = after.trim_start();
    let bytes = after.as_bytes();
    if bytes.first() != Some(&b'&') {
        return None;
    }
    let mut index = 1;
    while bytes.get(index).is_some_and(|b| b.is_ascii_whitespace()) {
        index += 1;
    }
    if after[index..].starts_with("mut")
        && after[index + 3..]
            .chars()
            .next()
            .is_none_or(|c| !c.is_alphanumeric() && c != '_')
    {
        index += 3;
        while bytes.get(index).is_some_and(|b| b.is_ascii_whitespace()) {
            index += 1;
        }
    }
    let mut segments = Vec::new();
    loop {
        let start = index;
        while bytes
            .get(index)
            .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_')
        {
            index += 1;
        }
        if start == index {
            break;
        }
        segments.push(after[start..index].to_string());
        if after[index..].starts_with("::") {
            index += 2;
        } else {
            break;
        }
    }
    segments.into_iter().next_back()
}

/// The identifier immediately before a type-annotation colon.
fn ident_before(before: &str) -> Option<String> {
    let ident: String = before
        .trim_end()
        .chars()
        .rev()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    if ident.is_empty()
        || ident == "mut"
        || ident == "self"
        || ident.chars().next().is_some_and(|c| c.is_numeric())
    {
        return None;
    }
    Some(ident)
}

/// Local-identifier to canonical-path map from `use` items, following
/// `as` renames and `{...}` groups across line breaks.
fn import_aliases(lines: &[CodeLine]) -> BTreeMap<String, String> {
    let mut aliases = BTreeMap::new();
    let mut index = 0;
    while index < lines.len() {
        let code = lines[index].code.clone();
        let trimmed = code.trim();
        if trimmed.starts_with("use ") || trimmed == "use" {
            let mut joined = code.clone();
            while !joined.contains(';') && index + 1 < lines.len() {
                index += 1;
                joined.push(' ');
                joined.push_str(&lines[index].code);
            }
            for (local, path) in parse_use(&joined) {
                aliases.insert(local, path);
            }
        }
        index += 1;
    }
    aliases
}

/// Parse one `use ...;` statement into `(local, canonical)` pairs.
///
/// `use termrock::author::Ui as Canvas;` binds `Canvas` to the true
/// canonical path `termrock::author::Ui`, so alias resolution keeps working
/// after a rename.
fn parse_use(statement: &str) -> Vec<(String, String)> {
    let mut text = statement.trim().to_string();
    if let Some(stripped) = text.strip_prefix("pub use") {
        text = stripped.to_string();
    } else if let Some(stripped) = text.strip_prefix("use") {
        text = stripped.to_string();
    }
    text = text.trim().trim_end_matches(';').trim().to_string();
    expand_use_path(&text)
}

/// Expand a `use` tree (with `{...}` groups and `as` renames) into
/// `(local, canonical)` pairs. Glob imports expand to nothing: a glob
/// binds no single name, so it can neither create nor hide an alias.
fn expand_use_path(tree: &str) -> Vec<(String, String)> {
    let tree = tree.trim();
    if tree.is_empty() || tree == "*" {
        return Vec::new();
    }
    if let Some((head, group)) = split_group(tree) {
        let mut bound = Vec::new();
        for item in split_top_level(group, ',') {
            let item = item.trim();
            if item.is_empty() {
                continue;
            }
            for (local, path) in expand_use_path(item) {
                if path == "self" {
                    let local = head.rsplit("::").next().unwrap_or(head.as_str());
                    bound.push((local.to_string(), head.clone()));
                } else {
                    bound.push((local, format!("{head}::{path}")));
                }
            }
        }
        return bound;
    }
    if let Some((path, alias)) = split_alias(tree) {
        return vec![(alias.to_string(), path.to_string())];
    }
    let local = tree.rsplit("::").next().unwrap_or(tree);
    vec![(local.to_string(), tree.to_string())]
}

/// Split `head::{...}` into head and group body.
fn split_group(tree: &str) -> Option<(String, &str)> {
    let open = tree.find('{')?;
    let head = tree[..open].trim().trim_end_matches("::").to_string();
    let mut depth = 0;
    for (offset, char) in tree[open..].char_indices() {
        if char == '{' {
            depth += 1;
        } else if char == '}' {
            depth -= 1;
            if depth == 0 {
                return Some((head, &tree[open + 1..open + offset]));
            }
        }
    }
    None
}

/// Split on a delimiter, ignoring delimiters nested in braces.
fn split_top_level(text: &str, delimiter: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0;
    let mut start = 0;
    for (offset, char) in text.char_indices() {
        if char == '{' {
            depth += 1;
        } else if char == '}' {
            depth -= 1;
        } else if char == delimiter && depth == 0 {
            parts.push(&text[start..offset]);
            start = offset + char.len_utf8();
        }
    }
    parts.push(&text[start..]);
    parts
}

/// Split a top-level `path as alias` rename.
fn split_alias(tree: &str) -> Option<(&str, &str)> {
    let mut depth = 0;
    let bytes = tree.as_bytes();
    let mut index = 0;
    while index + 4 <= bytes.len() {
        match bytes[index] {
            b'{' => depth += 1,
            b'}' => depth -= 1,
            _ => {}
        }
        // The leading space in " as " is the boundary: `use` paths never
        // contain spaces, so no longer word can match.
        if depth == 0 && tree[index..].starts_with(" as ") {
            let path = tree[..index].trim();
            let alias = tree[index + 4..].trim();
            if !path.is_empty() && !alias.is_empty() {
                return Some((path, alias));
            }
        }
        index += 1;
    }
    None
}

/// Word tokens of blanked code.
pub fn words(code: &str) -> Vec<&str> {
    code.split(|c: char| !is_word_char(c))
        .filter(|w| !w.is_empty())
        .collect()
}

fn is_word_char(char: char) -> bool {
    char.is_alphanumeric() || char == '_'
}

/// The receiver identifier of a `.method` call at `dot`, or `None` when
/// the call is qualified (`Type::method`), chained past a call (`f().m`),
/// or has no plain identifier receiver.
pub fn receiver_before(code: &str, dot: usize) -> Option<String> {
    let before = code[..dot].trim_end();
    if before.ends_with(')') || before.ends_with(']') {
        return None;
    }
    if before.ends_with(':') {
        return None;
    }
    let ident: String = before
        .chars()
        .rev()
        .take_while(|c| is_word_char(*c))
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    if ident.is_empty() || ident.chars().next().is_some_and(|c| c.is_numeric()) {
        return None;
    }
    Some(ident)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn striping_removes_comments_and_strings() {
        let file = scan_file(
            "crates/demo/src/lib.rs",
            "crates/demo",
            "//! showcase module docs stay out.\nuse termrock::Ui; // trailing Ui\nfn f() {\n    let s = \"Ui\";\n    /* block\n    Ui */\n    ui.fill(area, style);\n}\n",
        );
        assert_eq!(file.lines.len(), 8);
        assert!(!file.lines[0].code.contains("showcase"));
        assert!(!file.lines[3].code.contains("Ui"));
        assert!(file.lines[6].code.contains("ui.fill"));
    }

    #[test]
    fn multiline_raw_strings_span_lines() {
        let file = scan_file(
            "crates/demo/src/lib.rs",
            "crates/demo",
            "const S: &str = r#\"\nline one\nline two\n\"#;\n",
        );
        assert_eq!(file.strings.len(), 1);
        assert_eq!(file.strings[0].start_line, 1);
        assert_eq!(file.strings[0].end_line, 4);
    }

    #[test]
    fn cfg_test_regions_are_excluded() {
        let file = scan_file(
            "crates/demo/src/lib.rs",
            "crates/demo",
            "fn prod() {}\n#[cfg(test)]\nmod tests {\n    fn helper() {}\n}\n#[cfg(any(test, feature = \"testing\"))]\npub mod stub {\n    fn harness() {}\n}\n",
        );
        assert!(!file.is_test_line(1));
        assert!(file.is_test_line(4));
        assert!(file.is_test_line(8));
        assert!(file.enclosing_fn(1).is_some());
        assert!(file.enclosing_fn(4).is_none());
    }

    #[test]
    fn cfg_not_test_is_production() {
        let file = scan_file(
            "crates/demo/src/lib.rs",
            "crates/demo",
            "#[cfg(not(test))]\nfn prod() {}\n",
        );
        assert!(!file.is_test_line(2));
        assert_eq!(file.enclosing_fn(2), Some("prod"));
    }

    #[test]
    fn use_groups_expand_with_aliases() {
        let expanded = expand_use_path("termrock::{Buffer, Ui as Canvas}");
        assert!(expanded.contains(&("Buffer".to_string(), "termrock::Buffer".to_string())));
        assert!(expanded.contains(&("Canvas".to_string(), "termrock::Ui".to_string())));
    }

    #[test]
    fn ui_alias_resolves_through_rename() {
        let file = scan_file(
            "crates/demo/src/lib.rs",
            "crates/demo",
            "use termrock::author::Ui as Canvas;\nfn draw(c: &mut Canvas) {}\n",
        );
        assert!(file.is_ui_ident("Canvas"));
        assert!(file.is_ui_ident("ui"));
        assert!(!file.is_ui_ident("other"));
    }

    #[test]
    fn raw_buffer_paths_need_a_buffer_prefix() {
        assert!(is_raw_buffer_path("ratatui::buffer::Buffer"));
        assert!(is_raw_buffer_path("ratatui_core::buffer::Cell"));
        assert!(is_raw_buffer_path("termrock::Buffer"));
        assert!(!is_raw_buffer_path("crate::widgets::Cell"));
        assert!(!is_raw_buffer_path("super::Buffer"));
    }

    #[test]
    fn generated_markers_flag_without_skipping() {
        let file = scan_file(
            "crates/demo/src/gen.rs",
            "crates/demo",
            "// DO NOT EDIT: generated by build.\nuse termrock::Ui;\nfn f(ui: &mut Ui) {}\n",
        );
        assert!(file.generated);
        assert_eq!(file.enclosing_fn(3), Some("f"));
    }

    #[test]
    fn receivers_follow_ui_annotations_through_aliases() {
        let file = scan_file(
            "crates/demo/src/lib.rs",
            "crates/demo",
            "use termrock::author::Ui as Canvas;\n\
             fn draw(canvas: &mut Canvas<'_>, other: &mut Model) {\n\
             }\n",
        );
        assert!(file.is_ui_ident("canvas"));
        assert!(!file.is_ui_ident("other"));
    }

    #[test]
    fn receivers_reject_chains_and_paths() {
        assert_eq!(
            receiver_before("    ui.fill(area, s);", 6),
            Some("ui".to_string())
        );
        assert_eq!(receiver_before("    a.fill(x).flush();", 13), None);
        assert_eq!(receiver_before("    Buffer::empty(area)", 12), None);
    }
}
