//! Conservative source-level module provenance for ownership scanning.
//!
//! This is deliberately a bounded Rust recognizer, not a Rust parser. A
//! construct that can change module reachability and that this module does
//! not understand disables test-only pruning for the whole package.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

use super::scan::LineRange;

const MAX_TOKENS: usize = 250_000;
const MAX_CONTEXTS: usize = 50_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberAnalysis {
    /// Source paths kept in the ownership inventory, including module and
    /// explicitly configured target paths discovered outside `src`.
    pub sources: Vec<PathBuf>,
    /// Canonical source paths whose every proven incoming route is test-only.
    pub test_only: BTreeSet<PathBuf>,
    /// False means the caller must retain every candidate as production.
    pub complete: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TokenKind {
    Ident(String),
    String(Option<String>),
    Punct(char),
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Token {
    kind: TokenKind,
    line: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Truth {
    False,
    Unknown,
    True,
}

impl Truth {
    fn and(self, other: Self) -> Self {
        match (self, other) {
            (Self::False, _) | (_, Self::False) => Self::False,
            (Self::True, Self::True) => Self::True,
            _ => Self::Unknown,
        }
    }

    fn or(self, other: Self) -> Self {
        match (self, other) {
            (Self::True, _) | (_, Self::True) => Self::True,
            (Self::False, Self::False) => Self::False,
            _ => Self::Unknown,
        }
    }

    fn not(self) -> Self {
        match self {
            Self::False => Self::True,
            Self::Unknown => Self::Unknown,
            Self::True => Self::False,
        }
    }
}

/// Predicate values when the built-in `test` cfg is false and true.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Predicate(Truth, Truth);

impl Predicate {
    const ALWAYS: Self = Self(Truth::True, Truth::True);
    const TEST: Self = Self(Truth::False, Truth::True);
    const UNKNOWN: Self = Self(Truth::Unknown, Truth::Unknown);

    fn and(self, other: Self) -> Self {
        Self(self.0.and(other.0), self.1.and(other.1))
    }

    fn test_only(self) -> bool {
        self.0 == Truth::False && self.1 != Truth::False
    }
}

#[derive(Debug, Clone)]
struct Tokens {
    values: Vec<Token>,
    pairs: Vec<Option<usize>>,
}

#[derive(Debug, Clone)]
struct Attributes {
    predicate: Predicate,
    path: Option<String>,
    derives: BTreeSet<String>,
    complete: bool,
}

impl Default for Attributes {
    fn default() -> Self {
        Self {
            predicate: Predicate::ALWAYS,
            path: None,
            derives: BTreeSet::new(),
            complete: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ContextKey {
    file: PathBuf,
    module_dir: PathBuf,
    predicate: Predicate,
}

#[derive(Debug)]
struct Traversal {
    member_root: PathBuf,
    sources: BTreeMap<PathBuf, String>,
    included_sources: BTreeSet<PathBuf>,
    paths: BTreeSet<PathBuf>,
    reachable: BTreeMap<PathBuf, (bool, bool)>,
    visited: BTreeSet<ContextKey>,
    queue: VecDeque<ContextKey>,
    complete: bool,
}

impl Traversal {
    fn mark_reachable(&mut self, file: &Path, predicate: Predicate) {
        let state = self
            .reachable
            .entry(file.to_path_buf())
            .or_insert((false, false));
        if predicate.test_only() {
            state.1 = true;
        } else {
            state.0 = true;
        }
    }

    fn queue_file(&mut self, file: PathBuf, module_dir: PathBuf, predicate: Predicate) {
        let key = ContextKey {
            file,
            module_dir,
            predicate,
        };
        if self.visited.contains(&key) {
            return;
        }
        if self.visited.len() >= MAX_CONTEXTS {
            self.complete = false;
            return;
        }
        self.visited.insert(key.clone());
        self.queue.push_back(key);
    }

    fn resolve_module(
        &mut self,
        module_dir: &Path,
        path_base: &Path,
        name: &str,
        path_override: Option<&str>,
        predicate: Predicate,
    ) {
        let candidates = if let Some(path) = path_override {
            vec![path_base.join(path)]
        } else {
            vec![
                module_dir.join(format!("{name}.rs")),
                module_dir.join(name).join("mod.rs"),
            ]
        };
        let mut found = Vec::new();
        for candidate in candidates {
            if !candidate.exists() {
                continue;
            }
            if fs::symlink_metadata(&candidate)
                .is_ok_and(|metadata| metadata.file_type().is_symlink())
            {
                self.complete = false;
            }
            match fs::canonicalize(&candidate) {
                Ok(canonical) if canonical.starts_with(&self.member_root) => {
                    found.push(canonical);
                }
                _ => self.complete = false,
            }
        }
        found.sort();
        found.dedup();
        if found.is_empty() {
            self.complete = false;
            return;
        }
        if found.len() != 1 {
            self.complete = false;
        }
        for file in found {
            let Ok(text) = fs::read_to_string(&file) else {
                self.complete = false;
                continue;
            };
            self.paths.insert(file.clone());
            self.sources.entry(file.clone()).or_insert(text);
            let dir = module_directory(&file);
            if dir.as_os_str().is_empty() {
                self.complete = false;
                continue;
            }
            self.mark_reachable(&file, predicate);
            self.queue_file(file, dir, predicate);
        }
    }

    fn resolve_include(&mut self, include: IncludeDecl) {
        let candidate = include.source_dir.join(include.path);
        if fs::symlink_metadata(&candidate).is_ok_and(|metadata| metadata.file_type().is_symlink())
        {
            self.complete = false;
        }
        let Ok(file) = fs::canonicalize(&candidate) else {
            self.complete = false;
            return;
        };
        if !file.starts_with(&self.member_root) {
            self.complete = false;
            return;
        }
        let Ok(text) = fs::read_to_string(&file) else {
            self.complete = false;
            return;
        };
        self.paths.insert(file.clone());
        self.sources.entry(file.clone()).or_insert(text);
        self.included_sources.insert(file.clone());
        self.mark_reachable(&file, include.predicate);
        self.queue_file(file, include.module_dir, include.predicate);
    }

    fn analyze(mut self) -> Self {
        while let Some(context) = self.queue.pop_front() {
            let Some(source) = self.sources.get(&context.file).cloned() else {
                self.complete = false;
                continue;
            };
            let parsed = match parse_source(
                &source,
                &context.file,
                &context.module_dir,
                context.predicate,
            ) {
                Some(parsed) => parsed,
                None => {
                    self.complete = false;
                    continue;
                }
            };
            if !parsed.complete {
                self.complete = false;
            }
            if self.included_sources.contains(&context.file)
                && (!parsed.modules.is_empty() || !parsed.includes.is_empty())
            {
                // Included text participates in the caller's lexical module
                // path. Nested module/include resolution is retained as
                // production until its call-site/file provenance is modeled.
                self.complete = false;
            }
            for include in parsed.includes {
                self.resolve_include(include);
            }
            for module in parsed.modules {
                if module.inline.is_some() {
                    continue;
                }
                let Some(name) = module.name else {
                    self.complete = false;
                    continue;
                };
                self.resolve_module(
                    &module.module_dir,
                    &module.path_base,
                    &name,
                    module.path.as_deref(),
                    module.predicate,
                );
            }
        }
        self
    }
}

#[derive(Debug)]
struct ModuleDecl {
    name: Option<String>,
    path: Option<String>,
    predicate: Predicate,
    module_dir: PathBuf,
    path_base: PathBuf,
    inline: Option<(usize, usize)>,
}

#[derive(Debug)]
struct IncludeDecl {
    path: String,
    source_dir: PathBuf,
    module_dir: PathBuf,
    predicate: Predicate,
}

#[derive(Debug)]
struct ParsedSource {
    modules: Vec<ModuleDecl>,
    includes: Vec<IncludeDecl>,
    test_ranges: Vec<LineRange>,
    complete: bool,
}

/// Enumerates the current physical `src` inventory and source files reached
/// from known target roots. Unknown filesystem or manifest shapes retain all
/// candidates as production.
pub fn analyze_member(member: &Path) -> MemberAnalysis {
    let member_root = match fs::canonicalize(member) {
        Ok(path) => path,
        Err(_) => {
            return MemberAnalysis {
                sources: Vec::new(),
                test_only: BTreeSet::new(),
                complete: false,
            };
        }
    };
    let (mut source_paths, inventory_complete) = collect_src_sources(member);
    let (roots, manifest_complete) = target_roots(member);
    let mut traversal = Traversal {
        member_root: member_root.clone(),
        sources: BTreeMap::new(),
        included_sources: BTreeSet::new(),
        paths: BTreeSet::new(),
        reachable: BTreeMap::new(),
        visited: BTreeSet::new(),
        queue: VecDeque::new(),
        complete: inventory_complete && manifest_complete,
    };

    for source in &source_paths {
        match fs::canonicalize(source) {
            Ok(canonical) if canonical.starts_with(&member_root) => {
                if let Ok(text) = fs::read_to_string(source) {
                    traversal.sources.entry(canonical.clone()).or_insert(text);
                    traversal.paths.insert(canonical);
                } else {
                    traversal.complete = false;
                }
            }
            _ => traversal.complete = false,
        }
    }

    for root in roots {
        match fs::canonicalize(&root) {
            Ok(canonical) if canonical.starts_with(&member_root) => {
                if fs::symlink_metadata(&root)
                    .is_ok_and(|metadata| metadata.file_type().is_symlink())
                {
                    traversal.complete = false;
                }
                traversal.paths.insert(canonical.clone());
                let Ok(text) = fs::read_to_string(&canonical) else {
                    traversal.complete = false;
                    continue;
                };
                traversal.paths.insert(canonical.clone());
                traversal.sources.entry(canonical.clone()).or_insert(text);
                // Explicit Cargo target roots stay production inputs under
                // this scanner policy; cfg(test) items inside them are still
                // classified independently.
                let predicate = Predicate::ALWAYS;
                traversal.mark_reachable(&canonical, predicate);
                traversal.queue_file(
                    canonical.clone(),
                    canonical.parent().unwrap_or(&member_root).to_path_buf(),
                    predicate,
                );
            }
            _ => traversal.complete = false,
        }
    }

    traversal = traversal.analyze();
    source_paths.extend(traversal.paths.iter().cloned());
    source_paths.sort();
    source_paths.dedup();
    let test_only = if traversal.complete {
        traversal
            .reachable
            .into_iter()
            .filter_map(|(path, (production, test))| (!production && test).then_some(path))
            .collect()
    } else {
        BTreeSet::new()
    };
    MemberAnalysis {
        sources: source_paths,
        test_only,
        complete: traversal.complete,
    }
}

/// Returns line ranges for items proven to compile only when `test` is true.
/// Any uncertain syntax returns no ranges, retaining the source as production.
pub fn cfg_test_ranges(source: &str) -> Vec<LineRange> {
    let Some(tokens) = lex(source) else {
        return Vec::new();
    };
    let path = Path::new("<source>");
    let module_dir = Path::new(".");
    match parse_scope(
        &tokens,
        0,
        tokens.values.len(),
        ParsePath {
            file: path,
            module_dir,
            path_base: module_dir,
        },
        Predicate::ALWAYS,
        true,
    ) {
        Some(parsed) if parsed.complete => normalize_ranges(parsed.test_ranges),
        _ => Vec::new(),
    }
}

fn collect_src_sources(member: &Path) -> (Vec<PathBuf>, bool) {
    let root = member.join("src");
    let mut stack = vec![root.clone()];
    let mut visited = BTreeSet::new();
    let mut sources = Vec::new();
    let mut complete = true;
    while let Some(dir) = stack.pop() {
        let metadata = match fs::symlink_metadata(&dir) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound && dir == root => continue,
            Err(_) => {
                complete = false;
                continue;
            }
        };
        let was_symlink = metadata.file_type().is_symlink();
        if was_symlink {
            complete = false;
        }
        let target_metadata = if was_symlink {
            match fs::metadata(&dir) {
                Ok(metadata) => metadata,
                Err(_) => continue,
            }
        } else {
            metadata
        };
        if !target_metadata.is_dir() {
            if dir.extension().is_some_and(|ext| ext == "rs") {
                sources.push(dir);
            }
            continue;
        }
        if let Ok(canonical) = fs::canonicalize(&dir) {
            if !visited.insert(canonical) {
                complete = false;
                continue;
            }
        } else {
            complete = false;
            continue;
        }
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => {
                complete = false;
                continue;
            }
        };
        for entry in entries {
            let Ok(entry) = entry else {
                complete = false;
                continue;
            };
            let path = entry.path();
            let Ok(kind) = entry.file_type() else {
                complete = false;
                continue;
            };
            if kind.is_symlink() {
                complete = false;
                if fs::metadata(&path).is_ok_and(|metadata| metadata.is_dir()) {
                    stack.push(path);
                } else if path.extension().is_some_and(|ext| ext == "rs") {
                    sources.push(path);
                }
            } else if kind.is_dir() {
                stack.push(path);
            } else if kind.is_file() && path.extension().is_some_and(|ext| ext == "rs") {
                sources.push(path);
            }
        }
    }
    sources.sort();
    sources.dedup();
    (sources, complete)
}

fn target_roots(member: &Path) -> (Vec<PathBuf>, bool) {
    let mut roots = Vec::new();
    let src = member.join("src");
    for conventional in [src.join("lib.rs"), src.join("main.rs")] {
        if conventional.is_file() {
            roots.push(conventional);
        }
    }
    add_bin_roots(&src.join("bin"), &mut roots);

    let manifest = match fs::read_to_string(member.join("Cargo.toml")) {
        Ok(text) => text,
        Err(_) => return (roots, false),
    };
    let mut section = TargetKindSection::None;
    let mut complete = true;
    for raw_line in manifest.lines() {
        let line = strip_toml_comment(raw_line).trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') {
            section = match line {
                "[lib]" => TargetKindSection::Production,
                "[[bin]]" | "[[example]]" | "[[bench]]" => TargetKindSection::Production,
                "[[test]]" => TargetKindSection::Test,
                _ => TargetKindSection::None,
            };
            if (line.starts_with("[[lib")
                || line.starts_with("[[bin")
                || line.starts_with("[[example")
                || line.starts_with("[[test")
                || line.starts_with("[[bench"))
                && !matches!(line, "[[bin]]" | "[[example]]" | "[[test]]" | "[[bench]]")
            {
                complete = false;
            }
            continue;
        }
        if section == TargetKindSection::None {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            if line.contains("path") {
                complete = false;
            }
            continue;
        };
        if key.trim() != "path" {
            continue;
        }
        let Some(path) = parse_toml_path(value.trim()) else {
            complete = false;
            continue;
        };
        roots.push(member.join(path));
    }
    roots.sort();
    roots.dedup();
    (roots, complete)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TargetKindSection {
    None,
    Production,
    Test,
}

fn add_bin_roots(bin_dir: &Path, roots: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(bin_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if entry.file_type().is_ok_and(|kind| kind.is_file())
            && path.extension().is_some_and(|ext| ext == "rs")
        {
            roots.push(path);
        } else if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
            let main = path.join("main.rs");
            if main.is_file() {
                roots.push(main);
            }
        }
    }
}

fn strip_toml_comment(line: &str) -> &str {
    let mut quoted = false;
    let mut escaped = false;
    for (index, byte) in line.bytes().enumerate() {
        if escaped {
            escaped = false;
            continue;
        }
        match byte {
            b'\\' if quoted => escaped = true,
            b'"' => quoted = !quoted,
            b'#' if !quoted => return &line[..index],
            _ => {}
        }
    }
    line
}

fn parse_toml_path(value: &str) -> Option<String> {
    let value = value.strip_prefix('"')?.strip_suffix('"')?;
    if value.is_empty() || value.contains(['\\', '"']) || Path::new(value).is_absolute() {
        return None;
    }
    Some(value.to_string())
}

fn module_directory(file: &Path) -> PathBuf {
    let parent = file.parent().unwrap_or_else(|| Path::new("."));
    if file.file_name().is_some_and(|name| name == "mod.rs") {
        parent.to_path_buf()
    } else {
        parent.join(file.file_stem().unwrap_or_default())
    }
}

fn lex(source: &str) -> Option<Tokens> {
    let bytes = source.as_bytes();
    let mut values = Vec::new();
    let mut index = 0;
    let mut line = 1;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte.is_ascii_whitespace() {
            if byte == b'\n' {
                line += 1;
            }
            index += 1;
            continue;
        }
        if bytes[index..].starts_with(b"//") {
            while index < bytes.len() && bytes[index] != b'\n' {
                index += 1;
            }
            continue;
        }
        if bytes[index..].starts_with(b"/*") {
            index += 2;
            let mut depth = 1usize;
            while index < bytes.len() && depth > 0 {
                if bytes[index..].starts_with(b"/*") {
                    depth = depth.checked_add(1)?;
                    index += 2;
                } else if bytes[index..].starts_with(b"*/") {
                    depth -= 1;
                    index += 2;
                } else {
                    if bytes[index] == b'\n' {
                        line += 1;
                    }
                    index += char_len(bytes[index])?;
                }
            }
            if depth != 0 {
                return None;
            }
            continue;
        }

        let token_line = line;
        if let Some((prefix, hashes)) = raw_string_prefix(bytes, index) {
            index += prefix;
            let body_start = index;
            let close_len = 1 + hashes;
            let mut found = None;
            while index < bytes.len() {
                if bytes[index] == b'"'
                    && index + close_len <= bytes.len()
                    && bytes[index + 1..index + close_len]
                        .iter()
                        .all(|byte| *byte == b'#')
                {
                    found = Some(index);
                    break;
                }
                if bytes[index] == b'\n' {
                    line += 1;
                }
                index += char_len(bytes[index])?;
            }
            let close = found?;
            let content = std::str::from_utf8(&bytes[body_start..close]).ok()?;
            values.push(Token {
                kind: if prefix == 1 {
                    TokenKind::String(Some(content.to_string()))
                } else {
                    TokenKind::Other
                },
                line: token_line,
            });
            index = close + close_len;
        } else if let Some((prefix, is_rust_string)) = cooked_string_prefix(bytes, index) {
            index += prefix;
            let mut content = String::new();
            let mut simple = true;
            let mut closed = false;
            while index < bytes.len() {
                match bytes[index] {
                    b'"' => {
                        index += 1;
                        closed = true;
                        break;
                    }
                    b'\\' => {
                        simple = false;
                        index += 1;
                        if index >= bytes.len() {
                            return None;
                        }
                        if bytes[index] == b'\n' {
                            line += 1;
                            index += 1;
                            while index < bytes.len() && bytes[index].is_ascii_whitespace() {
                                if bytes[index] == b'\n' {
                                    line += 1;
                                }
                                index += 1;
                            }
                        } else {
                            index += char_len(bytes[index])?;
                        }
                    }
                    b'\n' => return None,
                    _ => {
                        let size = char_len(bytes[index])?;
                        content.push_str(std::str::from_utf8(&bytes[index..index + size]).ok()?);
                        index += size;
                    }
                }
            }
            if !closed {
                return None;
            }
            values.push(Token {
                kind: if is_rust_string {
                    TokenKind::String(simple.then_some(content))
                } else {
                    TokenKind::Other
                },
                line: token_line,
            });
        } else if let Some((ident, next)) = identifier(bytes, index) {
            values.push(Token {
                kind: TokenKind::Ident(ident),
                line: token_line,
            });
            index = next;
        } else if bytes[index].is_ascii_digit() {
            index += 1;
            while index < bytes.len()
                && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_')
            {
                index += 1;
            }
            values.push(Token {
                kind: TokenKind::Other,
                line: token_line,
            });
        } else if bytes[index] == b'\'' {
            if let Some(end) = char_literal_end(bytes, index) {
                values.push(Token {
                    kind: TokenKind::Other,
                    line: token_line,
                });
                index = end;
            } else {
                values.push(Token {
                    kind: TokenKind::Punct('\''),
                    line: token_line,
                });
                index += 1;
            }
        } else if bytes[index].is_ascii_punctuation() {
            values.push(Token {
                kind: TokenKind::Punct(bytes[index] as char),
                line: token_line,
            });
            index += 1;
        } else {
            // Non-ASCII identifiers and punctuation are not needed to prove
            // module provenance. Keep the package in the production set.
            return None;
        }
        if values.len() > MAX_TOKENS {
            return None;
        }
    }

    let mut pairs = vec![None; values.len()];
    let mut stack = Vec::new();
    for (position, token) in values.iter().enumerate() {
        let TokenKind::Punct(punct) = token.kind else {
            continue;
        };
        if matches!(punct, '(' | '[' | '{') {
            stack.push((punct, position));
        } else if matches!(punct, ')' | ']' | '}') {
            let (open, opening) = stack.pop()?;
            if !matches!((open, punct), ('(', ')') | ('[', ']') | ('{', '}')) {
                return None;
            }
            pairs[opening] = Some(position);
            pairs[position] = Some(opening);
        }
    }
    if !stack.is_empty() {
        return None;
    }
    Some(Tokens { values, pairs })
}

fn char_len(byte: u8) -> Option<usize> {
    if byte.is_ascii() {
        Some(1)
    } else if byte & 0b1110_0000 == 0b1100_0000 {
        Some(2)
    } else if byte & 0b1111_0000 == 0b1110_0000 {
        Some(3)
    } else if byte & 0b1111_1000 == 0b1111_0000 {
        Some(4)
    } else {
        None
    }
}

fn raw_string_prefix(bytes: &[u8], at: usize) -> Option<(usize, usize)> {
    let prefix = if bytes[at..].starts_with(b"br") || bytes[at..].starts_with(b"cr") {
        2
    } else if bytes[at..].starts_with(b"r") {
        1
    } else {
        return None;
    };
    let mut index = at + prefix;
    let mut hashes = 0;
    while bytes.get(index) == Some(&b'#') {
        hashes += 1;
        index += 1;
    }
    (bytes.get(index) == Some(&b'"')).then_some((index - at + 1, hashes))
}

fn cooked_string_prefix(bytes: &[u8], at: usize) -> Option<(usize, bool)> {
    if matches!(bytes.get(at), Some(b'b' | b'c')) && bytes.get(at + 1) == Some(&b'"') {
        return Some((2, false));
    }
    if bytes.get(at) == Some(&b'"') {
        Some((1, true))
    } else {
        None
    }
}

fn identifier(bytes: &[u8], at: usize) -> Option<(String, usize)> {
    let mut start = at;
    if bytes.get(at..at + 2) == Some(b"r#")
        && bytes
            .get(at + 2)
            .is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'_')
    {
        start += 2;
    }
    let first = *bytes.get(start)?;
    if !(first.is_ascii_alphabetic() || first == b'_') {
        return None;
    }
    let mut end = start + 1;
    while bytes
        .get(end)
        .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
    {
        end += 1;
    }
    Some((
        std::str::from_utf8(&bytes[start..end]).ok()?.to_string(),
        end,
    ))
}

fn char_literal_end(bytes: &[u8], at: usize) -> Option<usize> {
    if bytes.get(at) != Some(&b'\'') {
        return None;
    }
    let mut index = at + 1;
    if bytes.get(index) == Some(&b'\\') {
        index += 2;
    } else {
        let size = char_len(*bytes.get(index)?)?;
        index += size;
    }
    (bytes.get(index) == Some(&b'\'')).then_some(index + 1)
}

fn parse_source(
    source: &str,
    file: &Path,
    module_dir: &Path,
    inherited: Predicate,
) -> Option<ParsedSource> {
    let tokens = lex(source)?;
    let path_base = file.parent().unwrap_or_else(|| Path::new("."));
    parse_scope(
        &tokens,
        0,
        tokens.values.len(),
        ParsePath {
            file,
            module_dir,
            path_base,
        },
        inherited,
        true,
    )
}

#[derive(Clone, Copy)]
struct ParsePath<'a> {
    file: &'a Path,
    module_dir: &'a Path,
    path_base: &'a Path,
}

fn parse_scope(
    tokens: &Tokens,
    start: usize,
    end: usize,
    source_context: ParsePath<'_>,
    inherited: Predicate,
    collect_ranges: bool,
) -> Option<ParsedSource> {
    let mut parsed = ParsedSource {
        modules: Vec::new(),
        includes: Vec::new(),
        test_ranges: Vec::new(),
        complete: true,
    };
    let mut scope_predicate = inherited;
    let mut derived_names = BTreeSet::new();
    let mut imported_derive_names = BTreeSet::new();
    let mut imported_glob = false;
    let mut index = start;
    let mut saw_item = false;
    while index < end {
        if is_punct(tokens, index, ';') {
            index += 1;
            continue;
        }
        let item_start = index;
        let mut attributes = Attributes::default();
        while is_punct(tokens, index, '#') {
            let Some((next, inner, attr)) = parse_attribute(tokens, index, end) else {
                parsed.complete = false;
                return Some(parsed);
            };
            index = next;
            if inner {
                if saw_item {
                    parsed.complete = false;
                    continue;
                }
                if !attr.complete {
                    parsed.complete = false;
                }
                scope_predicate = scope_predicate.and(attr.predicate);
                continue;
            }
            derived_names.extend(attr.derives.iter().cloned());
            attributes.predicate = attributes.predicate.and(attr.predicate);
            if attributes.path.is_some() && attr.path.is_some() {
                parsed.complete = false;
            } else if attr.path.is_some() {
                attributes.path = attr.path;
            }
            attributes.complete &= attr.complete;
        }
        if index >= end {
            parsed.complete &= attributes.complete;
            break;
        }
        let effective = scope_predicate.and(attributes.predicate);
        if let Some(mod_start) = module_keyword(tokens, index, end) {
            if !attributes.complete {
                parsed.complete = false;
            }
            let Some(name) = ident_at(tokens, mod_start + 1).map(str::to_string) else {
                parsed.complete = false;
                return Some(parsed);
            };
            let Some(delimiter) = mod_start.checked_add(2) else {
                parsed.complete = false;
                return Some(parsed);
            };
            if is_punct(tokens, delimiter, ';') {
                let item_end = delimiter + 1;
                parsed.modules.push(ModuleDecl {
                    name: Some(name),
                    path: attributes.path,
                    predicate: effective,
                    module_dir: source_context.module_dir.to_path_buf(),
                    path_base: source_context.path_base.to_path_buf(),
                    inline: None,
                });
                if collect_ranges && effective.test_only() {
                    push_range(tokens, item_start, item_end, &mut parsed.test_ranges);
                }
                index = item_end;
                saw_item = true;
                continue;
            }
            if is_punct(tokens, delimiter, '{') {
                let Some(close) = tokens.pairs.get(delimiter).and_then(|value| *value) else {
                    parsed.complete = false;
                    return Some(parsed);
                };
                let child_dir = attributes.path.as_deref().map_or_else(
                    || source_context.module_dir.join(&name),
                    |path| source_context.path_base.join(path),
                );
                let nested = parse_scope(
                    tokens,
                    delimiter + 1,
                    close,
                    ParsePath {
                        file: source_context.file,
                        module_dir: &child_dir,
                        path_base: &child_dir,
                    },
                    effective,
                    collect_ranges,
                )?;
                parsed.complete &= nested.complete;
                parsed.modules.extend(nested.modules);
                parsed.includes.extend(nested.includes);
                parsed.test_ranges.extend(nested.test_ranges);
                if collect_ranges && effective.test_only() {
                    push_range(tokens, item_start, close + 1, &mut parsed.test_ranges);
                }
                parsed.modules.push(ModuleDecl {
                    name: Some(name),
                    path: attributes.path,
                    predicate: effective,
                    module_dir: child_dir,
                    path_base: source_context.path_base.to_path_buf(),
                    inline: Some((delimiter + 1, close)),
                });
                index = close + 1;
                saw_item = true;
                continue;
            }
            parsed.complete = false;
            return Some(parsed);
        }

        if !attributes.complete {
            parsed.complete = false;
        }
        let include = parse_static_include(tokens, index, end);
        if is_item_macro(tokens, index, end) && include.is_none() {
            parsed.complete = false;
        }
        if collect_ranges && effective.test_only() {
            let Some(item_end) = skip_item(tokens, index, end) else {
                parsed.complete = false;
                return Some(parsed);
            };
            collect_use_derive_imports(
                tokens,
                index,
                item_end,
                &mut imported_derive_names,
                &mut imported_glob,
            );
            push_range(tokens, item_start, item_end, &mut parsed.test_ranges);
            if let Some(path) = include {
                let source_dir = source_context
                    .file
                    .parent()
                    .unwrap_or_else(|| Path::new("."))
                    .to_path_buf();
                parsed.includes.push(IncludeDecl {
                    path,
                    source_dir,
                    module_dir: source_context.module_dir.to_path_buf(),
                    predicate: effective,
                });
            }
            index = item_end;
            saw_item = true;
            continue;
        }
        let Some(item_end) = skip_item(tokens, index, end) else {
            parsed.complete = false;
            return Some(parsed);
        };
        if item_end <= index {
            parsed.complete = false;
            return Some(parsed);
        }
        if !effective.test_only()
            && include.is_none()
            && skipped_item_has_unmodeled_routes(tokens, index, item_end)
        {
            parsed.complete = false;
            return Some(parsed);
        }
        collect_use_derive_imports(
            tokens,
            index,
            item_end,
            &mut imported_derive_names,
            &mut imported_glob,
        );
        if let Some(path) = include {
            let source_dir = source_context
                .file
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf();
            parsed.includes.push(IncludeDecl {
                path,
                source_dir,
                module_dir: source_context.module_dir.to_path_buf(),
                predicate: effective,
            });
        }
        index = item_end;
        saw_item = true;
    }
    if (imported_glob && !derived_names.is_empty())
        || derived_names
            .iter()
            .any(|name| imported_derive_names.contains(name))
    {
        // Without resolving macro namespaces, a matching `use` alias or glob
        // could shadow a built-in derive and generate arbitrary module items.
        parsed.complete = false;
    }
    Some(parsed)
}

fn parse_attribute(tokens: &Tokens, at: usize, end: usize) -> Option<(usize, bool, Attributes)> {
    let inner = is_punct(tokens, at + 1, '!');
    let bracket = at + usize::from(inner) + 1;
    if !is_punct(tokens, bracket, '[') {
        return None;
    }
    let close = tokens.pairs.get(bracket).and_then(|value| *value)?;
    if close >= end {
        return None;
    }
    let mut attr = Attributes::default();
    let first = ident_at(tokens, bracket + 1)?;
    match first {
        "cfg" => {
            let open = bracket + 2;
            let cfg_close = tokens.pairs.get(open).and_then(|value| *value);
            if !is_punct(tokens, open, '(') || cfg_close != Some(close - 1) {
                attr.complete = false;
                attr.predicate = Predicate::UNKNOWN;
            } else if let Some(predicate) = parse_cfg(tokens, open + 1, close - 1) {
                attr.predicate = predicate;
            } else {
                attr.complete = false;
                attr.predicate = Predicate::UNKNOWN;
            }
        }
        "cfg_attr" => {
            attr.complete = false;
            attr.predicate = Predicate::UNKNOWN;
        }
        "path" => {
            if is_punct(tokens, bracket + 2, '=')
                && let Some(TokenKind::String(Some(path))) =
                    tokens.values.get(bracket + 3).map(|token| &token.kind)
                && bracket + 4 == close
                && !path.is_empty()
                && !Path::new(path).is_absolute()
            {
                attr.path = Some(path.clone());
            } else {
                attr.complete = false;
            }
        }
        "test" | "bench" => attr.predicate = Predicate::TEST,
        "derive" => match builtin_derive_names(tokens, bracket + 2, close - 1) {
            Some(names) => attr.derives = names,
            None => attr.complete = false,
        },
        name if known_inert_attribute(name) => {}
        _ => attr.complete = false,
    }
    Some((close + 1, inner, attr))
}

fn known_inert_attribute(name: &str) -> bool {
    matches!(
        name,
        "doc"
            | "allow"
            | "warn"
            | "deny"
            | "forbid"
            | "expect"
            | "ignore"
            | "should_panic"
            | "inline"
            | "cold"
            | "track_caller"
            | "deprecated"
            | "must_use"
            | "non_exhaustive"
            | "repr"
            | "no_mangle"
            | "export_name"
            | "link"
            | "link_name"
            | "used"
            | "target_feature"
            | "macro_export"
            | "automatically_derived"
            | "global_allocator"
            | "panic_handler"
            | "proc_macro"
            | "proc_macro_attribute"
            | "proc_macro_derive"
            | "no_std"
            | "no_main"
            | "crate_type"
            | "crate_name"
            | "recursion_limit"
            | "type_length_limit"
            | "allow_internal_unstable"
            | "allow_internal_unsafe"
            | "stable"
            | "unstable"
            | "rustfmt"
            | "coverage"
    )
}

fn builtin_derive_names(tokens: &Tokens, start: usize, end: usize) -> Option<BTreeSet<String>> {
    if !is_punct(tokens, start, '(')
        || tokens.pairs.get(start).and_then(|value| *value) != Some(end)
    {
        return None;
    }
    let known = [
        "Clone",
        "Copy",
        "Debug",
        "Default",
        "Eq",
        "Hash",
        "Ord",
        "PartialEq",
        "PartialOrd",
    ];
    let mut index = start + 1;
    let mut names = BTreeSet::new();
    while index < end {
        let name = ident_at(tokens, index)?;
        if !known.contains(&name) {
            return None;
        }
        names.insert(name.to_string());
        index += 1;
        if index == end {
            break;
        }
        if !is_punct(tokens, index, ',') {
            return None;
        }
        index += 1;
    }
    (!names.is_empty()).then_some(names)
}

fn collect_use_derive_imports(
    tokens: &Tokens,
    start: usize,
    end: usize,
    imported: &mut BTreeSet<String>,
    imported_glob: &mut bool,
) {
    if item_head(tokens, start, end) != Some("use") {
        return;
    }
    const DERIVE_NAMES: &[&str] = &[
        "Clone",
        "Copy",
        "Debug",
        "Default",
        "Eq",
        "Hash",
        "Ord",
        "PartialEq",
        "PartialOrd",
    ];
    for index in start..end {
        if is_punct(tokens, index, '*') {
            *imported_glob = true;
        }
        if let Some(name) = ident_at(tokens, index)
            && DERIVE_NAMES.contains(&name)
        {
            // This is intentionally broader than resolving a use tree: any
            // matching name could be a derive macro import or alias.
            imported.insert(name.to_string());
        }
    }
}

fn parse_cfg(tokens: &Tokens, start: usize, end: usize) -> Option<Predicate> {
    let (predicate, next) = cfg_node(tokens, start, end)?;
    (next == end).then_some(predicate)
}

fn cfg_node(tokens: &Tokens, at: usize, end: usize) -> Option<(Predicate, usize)> {
    let name = ident_at(tokens, at)?;
    let open = at + 1;
    if matches!(name, "all" | "any" | "not") && is_punct(tokens, open, '(') {
        let close = tokens.pairs.get(open).and_then(|value| *value)?;
        if close >= end {
            return None;
        }
        let mut children = Vec::new();
        let mut index = open + 1;
        while index < close {
            let (child, next) = cfg_node(tokens, index, close)?;
            children.push(child);
            index = next;
            if index == close {
                break;
            }
            if !is_punct(tokens, index, ',') {
                return None;
            }
            index += 1;
            if index == close {
                break;
            }
        }
        let result = match name {
            "all" => children.into_iter().fold(Predicate::ALWAYS, Predicate::and),
            "any" => children
                .into_iter()
                .fold(Predicate(Truth::False, Truth::False), |left, right| {
                    Predicate(left.0.or(right.0), left.1.or(right.1))
                }),
            "not" if children.len() == 1 => {
                let child = children[0];
                Predicate(child.0.not(), child.1.not())
            }
            "not" => return None,
            _ => Predicate::UNKNOWN,
        };
        return Some((result, close + 1));
    }
    if is_punct(tokens, open, '=') {
        let value = tokens.values.get(open + 1)?;
        if !matches!(value.kind, TokenKind::String(_)) {
            return None;
        }
        return Some((Predicate::UNKNOWN, open + 2));
    }
    if name == "test" {
        return Some((Predicate::TEST, at + 1));
    }
    if name == "true" {
        return Some((Predicate(Truth::True, Truth::True), at + 1));
    }
    if name == "false" {
        return Some((Predicate(Truth::False, Truth::False), at + 1));
    }
    Some((Predicate::UNKNOWN, at + 1))
}

fn module_keyword(tokens: &Tokens, at: usize, end: usize) -> Option<usize> {
    let mut index = at;
    if ident_at(tokens, index) == Some("pub") {
        index += 1;
        if is_punct(tokens, index, '(') {
            index = tokens.pairs.get(index).and_then(|value| *value)? + 1;
        }
    }
    (index < end && ident_at(tokens, index) == Some("mod")).then_some(index)
}

fn is_item_macro(tokens: &Tokens, at: usize, end: usize) -> bool {
    let mut index = at;
    if is_punct(tokens, index, ':') && is_punct(tokens, index + 1, ':') {
        index += 2;
    }
    if ident_at(tokens, index).is_none() {
        return false;
    }
    index += 1;
    while is_punct(tokens, index, ':') && is_punct(tokens, index + 1, ':') {
        index += 2;
        if ident_at(tokens, index).is_none() {
            return false;
        }
        index += 1;
    }
    is_punct(tokens, index, '!') && is_open(tokens, index + 1) && index + 1 < end
}

fn parse_static_include(tokens: &Tokens, at: usize, end: usize) -> Option<String> {
    if ident_at(tokens, at) != Some("include")
        || !is_punct(tokens, at + 1, '!')
        || !is_punct(tokens, at + 2, '(')
        || !is_punct(tokens, at + 4, ')')
        || !is_punct(tokens, at + 5, ';')
        || at + 6 > end
    {
        return None;
    }
    match &tokens.values.get(at + 3)?.kind {
        TokenKind::String(Some(path)) if !path.is_empty() => Some(path.clone()),
        _ => None,
    }
}

fn skipped_item_has_unmodeled_routes(tokens: &Tokens, start: usize, end: usize) -> bool {
    for index in start..end {
        if ident_at(tokens, index) == Some("mod")
            && ident_at(tokens, index + 1).is_some()
            && (is_punct(tokens, index + 2, ';') || is_punct(tokens, index + 2, '{'))
        {
            return true;
        }
        if ident_at(tokens, index) == Some("include") && is_punct(tokens, index + 1, '!') {
            return true;
        }
        if is_item_macro(tokens, index, end) && macro_starts_block_item(tokens, start, index) {
            return true;
        }
    }
    false
}

fn macro_starts_block_item(tokens: &Tokens, item_start: usize, macro_start: usize) -> bool {
    let mut path_start = macro_start;
    while path_start >= item_start + 3
        && is_punct(tokens, path_start - 1, ':')
        && is_punct(tokens, path_start - 2, ':')
        && ident_at(tokens, path_start - 3).is_some()
    {
        path_start -= 3;
    }
    if path_start == item_start {
        return true;
    }
    matches!(
        tokens.values.get(path_start - 1).map(|token| &token.kind),
        Some(TokenKind::Punct(';' | '{' | '}' | ']'))
    )
}

fn skip_item(tokens: &Tokens, start: usize, end: usize) -> Option<usize> {
    let first = item_head(tokens, start, end);
    let has_braced_body = matches!(
        first,
        Some("fn" | "struct" | "enum" | "union" | "trait" | "impl" | "macro_rules" | "extern")
    );
    let is_use = first == Some("use");
    let mut angle_depth = 0usize;
    let mut index = start;
    while index < end {
        if is_punct(tokens, index, ';') {
            return Some(index + 1);
        }
        if is_punct(tokens, index, '<') {
            angle_depth = angle_depth.checked_add(1)?;
            index += 1;
            continue;
        }
        if is_punct(tokens, index, '>') && angle_depth > 0 {
            angle_depth -= 1;
            index += 1;
            continue;
        }
        if is_open(tokens, index) {
            let close = tokens.pairs.get(index).and_then(|value| *value)?;
            if is_punct(tokens, index, '{') && angle_depth == 0 && has_braced_body && !is_use {
                return Some(close + 1);
            }
            index = close + 1;
            continue;
        }
        index += 1;
    }
    None
}

fn item_head(tokens: &Tokens, start: usize, end: usize) -> Option<&str> {
    let mut index = start;
    if ident_at(tokens, index) == Some("pub") {
        index += 1;
        if is_punct(tokens, index, '(') {
            index = tokens.pairs.get(index).and_then(|value| *value)? + 1;
        }
    }
    loop {
        let word = ident_at(tokens, index)?;
        match word {
            "async" | "unsafe" | "default" | "auto" | "const" => index += 1,
            "extern" => {
                index += 1;
                if matches!(
                    tokens.values.get(index).map(|token| &token.kind),
                    Some(TokenKind::String(_))
                ) {
                    index += 1;
                }
                if is_punct(tokens, index, '{') {
                    return Some("extern");
                }
            }
            _ => break,
        }
        if index >= end {
            return None;
        }
    }
    ident_at(tokens, index)
}

fn push_range(tokens: &Tokens, start: usize, end: usize, ranges: &mut Vec<LineRange>) {
    let Some(first) = token_line(tokens, start) else {
        return;
    };
    let Some(last) = token_line(tokens, end.saturating_sub(1)) else {
        return;
    };
    let mut safe_lines = Vec::new();
    for line in first..=last {
        let line_start = tokens.values.partition_point(|token| token.line < line);
        let line_end = tokens.values.partition_point(|token| token.line <= line);
        if line_start < line_end && line_start >= start && line_end <= end {
            safe_lines.push(line);
        }
    }
    let mut start: Option<usize> = None;
    let mut end: Option<usize> = None;
    for line in safe_lines {
        match end {
            Some(previous) if previous.checked_add(1) == Some(line) => end = Some(line),
            Some(previous) => {
                ranges.push(LineRange {
                    start: start.take().unwrap_or(previous),
                    end: previous,
                });
                start = Some(line);
                end = Some(line);
            }
            None => {
                start = Some(line);
                end = Some(line);
            }
        }
    }
    if let (Some(start), Some(end)) = (start, end) {
        ranges.push(LineRange { start, end });
    }
}

fn normalize_ranges(mut ranges: Vec<LineRange>) -> Vec<LineRange> {
    ranges.sort_by_key(|range| (range.start, range.end));
    let mut normalized: Vec<LineRange> = Vec::new();
    for range in ranges {
        if let Some(previous) = normalized.last_mut()
            && range.start <= previous.end.saturating_add(1)
        {
            previous.end = previous.end.max(range.end);
        } else {
            normalized.push(range);
        }
    }
    normalized
}

fn token_line(tokens: &Tokens, index: usize) -> Option<usize> {
    tokens.values.get(index).map(|token| token.line)
}

fn ident_at(tokens: &Tokens, index: usize) -> Option<&str> {
    match &tokens.values.get(index)?.kind {
        TokenKind::Ident(ident) => Some(ident),
        _ => None,
    }
}

fn is_punct(tokens: &Tokens, index: usize, expected: char) -> bool {
    matches!(tokens.values.get(index).map(|token| &token.kind), Some(TokenKind::Punct(value)) if *value == expected)
}

fn is_open(tokens: &Tokens, index: usize) -> bool {
    is_punct(tokens, index, '(') || is_punct(tokens, index, '[') || is_punct(tokens, index, '{')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ranges(source: &str) -> Vec<LineRange> {
        cfg_test_ranges(source)
    }

    #[test]
    fn cfg_truth_table_requires_complete_predicate_proof() {
        let source = "#[cfg(all(test, feature = \"x\"))]\nfn yes() {}\n\
                      #[cfg(any(test, feature = \"x\"))]\nfn maybe() {}\n\
                      #[cfg(not(test))]\nfn no() {}\n\
                      #[cfg(not(not(test)))]\nfn double_not() {}\n";
        let actual = ranges(source);
        assert!(actual.iter().any(|range| range.contains(2)));
        assert!(!actual.iter().any(|range| range.contains(4)));
        assert!(!actual.iter().any(|range| range.contains(6)));
        assert!(actual.iter().any(|range| range.contains(8)));
    }

    #[test]
    fn repeated_and_enclosing_cfg_attributes_compose() {
        let source = "#[cfg(test)]\n#[cfg(not(feature = \"x\"))]\nmod tests {\n\
                      #[cfg(not(test))]\nfn impossible() {}\n\
                      fn helper() {}\n}\n\
                      #[cfg(test)]\n#[cfg(not(test))]\nfn impossible_outer() {}\n";
        let actual = ranges(source);
        assert!(actual.iter().any(|range| range.contains(6)));
        assert!(!actual.iter().any(|range| range.contains(9)));
    }

    #[test]
    fn test_names_comments_raw_strings_and_characters_never_mask_code() {
        let source = "// #[cfg(test)] mod phantom;\n\
                      const TEXT: &str = r###\"#[cfg(test)] mod phantom { }\"###;\n\
                      const BYTES: &[u8] = br##\"#[cfg(test)] mod phantom {}\"##;\n\
                      const CHAR: char = '{';\n\
                      fn lifetime<'a>(input: &'a str) { let _ = input; }\n\
                      mod tests { fn production() {} }\n\
                      #[cfg(test)]\nfn actual_test() {}\n";
        let actual = ranges(source);
        assert_eq!(actual.len(), 1);
        assert_eq!(actual[0], LineRange { start: 7, end: 8 });
    }

    #[test]
    fn cfg_values_do_not_match_test_by_substring_and_byte_strings_are_not_cfg_strings() {
        assert!(ranges("#[cfg(any(feature = \"test\", unix))]\nfn prod() {}\n").is_empty());
        assert!(ranges("#[cfg(all(test, feature = b\"x\"))]\nfn malformed() {}\n").is_empty());
        assert_eq!(
            ranges("#[cfg(all(test, feature = \"x\"))]\nfn test_only() {}\n"),
            [LineRange { start: 1, end: 2 }]
        );
    }

    #[test]
    fn malformed_lexing_and_unknown_attributes_keep_all_lines_production() {
        assert!(ranges("#[cfg(test)]\nfn x() {\n").is_empty());
        assert!(ranges("#[custom_rewrite]\n#[cfg(test)]\nfn x() {}\n").is_empty());
        assert!(ranges("#[cfg_attr(test, path = \"x.rs\")]\nmod x;\n").is_empty());
    }

    #[test]
    fn repeated_module_contexts_deduplicate_and_context_cap_fails_closed() {
        let mut traversal = Traversal {
            member_root: PathBuf::from("/member"),
            sources: BTreeMap::new(),
            included_sources: BTreeSet::new(),
            paths: BTreeSet::new(),
            reachable: BTreeMap::new(),
            visited: BTreeSet::new(),
            queue: VecDeque::new(),
            complete: true,
        };
        let file = PathBuf::from("/member/src/lib.rs");
        let module_dir = PathBuf::from("/member/src");
        traversal.queue_file(file.clone(), module_dir.clone(), Predicate::ALWAYS);
        traversal.queue_file(file, module_dir, Predicate::ALWAYS);
        assert_eq!(traversal.queue.len(), 1);

        for index in 0..=MAX_CONTEXTS {
            traversal.queue_file(
                PathBuf::from(format!("/member/src/{index}.rs")),
                PathBuf::from(format!("/member/src/{index}")),
                Predicate::ALWAYS,
            );
        }
        assert!(!traversal.complete);
        assert_eq!(traversal.queue.len(), MAX_CONTEXTS);
    }

    #[test]
    fn token_limit_fails_closed() {
        let source = "#[cfg(test)]\nfn test_only() {}\n".repeat(MAX_TOKENS / 4);
        assert!(ranges(&source).is_empty());
    }
}
