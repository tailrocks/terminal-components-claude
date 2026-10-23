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
    literal_cfg_test: bool,
    path: Option<String>,
    derives: BTreeSet<String>,
    test_attributes: BTreeSet<String>,
    complete: bool,
}

impl Default for Attributes {
    fn default() -> Self {
        Self {
            predicate: Predicate::ALWAYS,
            literal_cfg_test: false,
            path: None,
            derives: BTreeSet::new(),
            test_attributes: BTreeSet::new(),
            complete: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ContextKey {
    file: PathBuf,
    // The caller's module directory carries the included text's namespace.
    module_dir: PathBuf,
    // `include!` changes filesystem lookup to the included file's parent.
    file_module_dir: PathBuf,
    predicate: Predicate,
    expression_fragment: bool,
}

#[derive(Debug)]
struct Traversal {
    member_root: PathBuf,
    sources: BTreeMap<PathBuf, String>,
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

    fn queue_file(
        &mut self,
        file: PathBuf,
        module_dir: PathBuf,
        file_module_dir: PathBuf,
        predicate: Predicate,
        expression_fragment: bool,
    ) {
        let key = ContextKey {
            file,
            module_dir,
            file_module_dir,
            predicate,
            expression_fragment,
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
        file_module_dir: &Path,
        path_base: &Path,
        name: &str,
        path_override: Option<&str>,
        predicate: Predicate,
    ) {
        let candidates = if let Some(path) = path_override {
            vec![path_base.join(path)]
        } else {
            vec![
                file_module_dir.join(format!("{name}.rs")),
                file_module_dir.join(name).join("mod.rs"),
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
            // Rust treats a path-attributed external module file like a
            // `mod.rs` for child lookup. A normal `name.rs` instead gets its
            // stem appended to the source file's parent.
            let child_file_module_dir = if path_override.is_some() {
                file.parent()
                    .unwrap_or_else(|| Path::new("."))
                    .to_path_buf()
            } else {
                module_directory(&file)
            };
            if child_file_module_dir.as_os_str().is_empty() {
                self.complete = false;
                continue;
            }
            let child_module_dir = module_dir.join(name);
            self.mark_reachable(&file, predicate);
            self.queue_file(
                file,
                child_module_dir,
                child_file_module_dir,
                predicate,
                false,
            );
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
        self.mark_reachable(&file, include.predicate);
        let file_module_dir = file
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        self.queue_file(
            file,
            include.module_dir,
            file_module_dir,
            include.predicate,
            include.expression_fragment,
        );
    }

    fn analyze(mut self) -> Self {
        while let Some(context) = self.queue.pop_front() {
            let Some(source) = self.sources.get(&context.file).cloned() else {
                self.complete = false;
                continue;
            };
            let parsed = match if context.expression_fragment {
                parse_expression_fragment(
                    &source,
                    &context.file,
                    &context.module_dir,
                    &context.file_module_dir,
                    context.predicate,
                )
            } else {
                parse_source(
                    &source,
                    &context.file,
                    &context.module_dir,
                    &context.file_module_dir,
                    context.predicate,
                )
            } {
                Some(parsed) => parsed,
                None => {
                    self.complete = false;
                    continue;
                }
            };
            if !parsed.complete {
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
                    &module.file_module_dir,
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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ModuleDecl {
    name: Option<String>,
    path: Option<String>,
    predicate: Predicate,
    module_dir: PathBuf,
    file_module_dir: PathBuf,
    path_base: PathBuf,
    inline: Option<(usize, usize)>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct IncludeDecl {
    path: String,
    source_dir: PathBuf,
    module_dir: PathBuf,
    predicate: Predicate,
    origin: usize,
    expression_fragment: bool,
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
                    canonical.parent().unwrap_or(&member_root).to_path_buf(),
                    predicate,
                    false,
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
            file_module_dir: module_dir,
            path_base: module_dir,
        },
        Predicate::ALWAYS,
        ParseScopeMode {
            collect_ranges: true,
            collect_body_routes: false,
            expression_fragment: false,
        },
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
    file_module_dir: &Path,
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
            file_module_dir,
            path_base,
        },
        inherited,
        ParseScopeMode {
            collect_ranges: true,
            collect_body_routes: true,
            expression_fragment: false,
        },
    )
}

fn parse_expression_fragment(
    source: &str,
    file: &Path,
    module_dir: &Path,
    file_module_dir: &Path,
    inherited: Predicate,
) -> Option<ParsedSource> {
    let tokens = lex(source)?;
    let path_base = file.parent().unwrap_or_else(|| Path::new("."));
    let mut routes = ParsedSource {
        modules: Vec::new(),
        includes: Vec::new(),
        test_ranges: Vec::new(),
        complete: true,
    };
    let known_include_origins = BTreeSet::new();
    scan_body_route_groups(
        &tokens,
        0,
        tokens.values.len(),
        BodyRouteContext {
            source: ParsePath {
                file,
                module_dir,
                file_module_dir,
                path_base,
            },
            predicate: inherited,
            inline_scopes: &[],
            known_include_origins: &known_include_origins,
            expression_fragment: true,
            strict_unknown_macros: true,
            depth: 0,
        },
        &mut routes,
    );
    routes.modules.sort();
    routes.modules.dedup();
    routes.includes.sort();
    routes.includes.dedup();
    Some(routes)
}

#[derive(Clone, Copy)]
struct ParsePath<'a> {
    file: &'a Path,
    module_dir: &'a Path,
    file_module_dir: &'a Path,
    path_base: &'a Path,
}

#[derive(Clone, Copy)]
struct ParseScopeMode {
    collect_ranges: bool,
    collect_body_routes: bool,
    expression_fragment: bool,
}

fn parse_scope(
    tokens: &Tokens,
    start: usize,
    end: usize,
    source_context: ParsePath<'_>,
    inherited: Predicate,
    mode: ParseScopeMode,
) -> Option<ParsedSource> {
    let ParseScopeMode {
        collect_ranges,
        collect_body_routes,
        expression_fragment,
    } = mode;
    let mut parsed = ParsedSource {
        modules: Vec::new(),
        includes: Vec::new(),
        test_ranges: Vec::new(),
        complete: true,
    };
    let mut scope_predicate = inherited;
    let mut derived_names = BTreeSet::new();
    let mut imported_derive_names = BTreeSet::new();
    let mut imported_derive_glob = false;
    let mut test_attributes = BTreeSet::new();
    let mut imported_test_attributes = TestAttributeImports::default();
    let mut skipped_items = Vec::new();
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
            test_attributes.extend(attr.test_attributes.iter().cloned());
            attributes.derives.extend(attr.derives.iter().cloned());
            attributes
                .test_attributes
                .extend(attr.test_attributes.iter().cloned());
            attributes.predicate = attributes.predicate.and(attr.predicate);
            attributes.literal_cfg_test |= attr.literal_cfg_test;
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
                    file_module_dir: source_context.file_module_dir.to_path_buf(),
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
                if collect_ranges
                    && !collect_body_routes
                    && attributes.complete
                    && attributes.literal_cfg_test
                    && attributes.path.is_none()
                    && attributes.derives.is_empty()
                    && attributes.test_attributes.is_empty()
                    && effective.test_only()
                {
                    push_range(tokens, item_start, close + 1, &mut parsed.test_ranges);
                    index = close + 1;
                    saw_item = true;
                    continue;
                }
                let child_module_dir = source_context.module_dir.join(&name);
                let child_file_module_dir = attributes.path.as_deref().map_or_else(
                    || source_context.file_module_dir.join(&name),
                    |path| source_context.path_base.join(path),
                );
                let nested = parse_scope(
                    tokens,
                    delimiter + 1,
                    close,
                    ParsePath {
                        file: source_context.file,
                        module_dir: &child_module_dir,
                        file_module_dir: &child_file_module_dir,
                        path_base: &child_file_module_dir,
                    },
                    effective,
                    ParseScopeMode {
                        collect_ranges,
                        collect_body_routes,
                        expression_fragment: false,
                    },
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
                    module_dir: child_module_dir,
                    file_module_dir: child_file_module_dir,
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
        let include = parse_static_include(tokens, index, end, expression_fragment);
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
                &mut imported_derive_glob,
            );
            record_test_attribute_imports(tokens, index, item_end, &mut imported_test_attributes);
            push_range(tokens, item_start, item_end, &mut parsed.test_ranges);
            skipped_items.push((index, item_end, effective));
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
                    origin: index,
                    expression_fragment,
                });
            }
            index = item_end;
            saw_item = true;
            continue;
        }
        let Some(item_end) = skip_item(tokens, index, end) else {
            if expression_fragment {
                break;
            }
            parsed.complete = false;
            return Some(parsed);
        };
        if item_end <= index {
            parsed.complete = false;
            return Some(parsed);
        }
        skipped_items.push((index, item_end, effective));
        collect_use_derive_imports(
            tokens,
            index,
            item_end,
            &mut imported_derive_names,
            &mut imported_derive_glob,
        );
        record_test_attribute_imports(tokens, index, item_end, &mut imported_test_attributes);
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
                origin: index,
                expression_fragment,
            });
        }
        index = item_end;
        saw_item = true;
    }
    if (imported_derive_glob && !derived_names.is_empty())
        || derived_names
            .iter()
            .any(|name| imported_derive_names.contains(name))
        || test_attributes
            .iter()
            .any(|name| imported_test_attributes.may_shadow(name))
    {
        // Without resolving macro namespaces, a matching `use` alias or glob
        // could shadow a built-in derive or test attribute and generate
        // arbitrary module items.
        parsed.complete = false;
    }
    for (item_start, item_end, effective) in skipped_items {
        if !(collect_ranges && effective.test_only())
            && skipped_item_has_unmodeled_routes(
                tokens,
                item_start,
                item_end,
                &imported_test_attributes,
            )
        {
            parsed.complete = false;
        }
        if collect_body_routes {
            let routes = collect_body_routes_in_item(
                tokens,
                item_start,
                item_end,
                source_context,
                effective,
            );
            parsed.complete &= routes.complete;
            parsed.modules.extend(routes.modules);
            parsed.includes.extend(routes.includes);
        }
    }
    parsed.modules.sort();
    parsed.modules.dedup();
    parsed.includes.sort();
    parsed.includes.dedup();
    Some(parsed)
}

#[derive(Clone)]
struct InlineRouteContext {
    open: usize,
    close: usize,
    module_dir: PathBuf,
    file_module_dir: PathBuf,
    predicate: Predicate,
}

#[derive(Clone, Copy)]
struct BodyRouteContext<'a> {
    source: ParsePath<'a>,
    predicate: Predicate,
    inline_scopes: &'a [InlineRouteContext],
    known_include_origins: &'a BTreeSet<usize>,
    expression_fragment: bool,
    strict_unknown_macros: bool,
    depth: usize,
}

fn collect_body_routes_in_item(
    tokens: &Tokens,
    start: usize,
    end: usize,
    source_context: ParsePath<'_>,
    inherited: Predicate,
) -> ParsedSource {
    let mut routes = ParsedSource {
        modules: Vec::new(),
        includes: Vec::new(),
        test_ranges: Vec::new(),
        complete: true,
    };
    if !matches!(
        item_head(tokens, start, end),
        Some("fn" | "impl" | "trait" | "const" | "static")
    ) {
        return routes;
    }
    let known_include_origins = BTreeSet::new();
    let expression_fragment = matches!(
        item_head(tokens, start, end),
        Some("fn" | "const" | "static")
    );
    scan_body_route_groups(
        tokens,
        start,
        end,
        BodyRouteContext {
            source: source_context,
            predicate: inherited,
            inline_scopes: &[],
            known_include_origins: &known_include_origins,
            expression_fragment,
            strict_unknown_macros: false,
            depth: 0,
        },
        &mut routes,
    );
    routes.modules.sort();
    routes.modules.dedup();
    routes.includes.sort();
    routes.includes.dedup();
    routes
}

fn collect_scope_body_routes(
    tokens: &Tokens,
    start: usize,
    end: usize,
    context: BodyRouteContext<'_>,
    routes: &mut ParsedSource,
) {
    if context.depth >= 128 {
        routes.complete = false;
        return;
    }
    let Some(scope) = parse_scope(
        tokens,
        start,
        end,
        context.source,
        context.predicate,
        ParseScopeMode {
            collect_ranges: false,
            collect_body_routes: false,
            expression_fragment: context.expression_fragment,
        },
    ) else {
        routes.complete = false;
        return;
    };
    routes.complete &= scope.complete;
    let inline_scopes = scope
        .modules
        .iter()
        .filter_map(|module| {
            let (body_start, close) = module.inline?;
            Some(InlineRouteContext {
                open: body_start.checked_sub(1)?,
                close,
                module_dir: module.module_dir.clone(),
                file_module_dir: module.file_module_dir.clone(),
                predicate: module.predicate,
            })
        })
        .collect::<Vec<_>>();
    let known_include_origins = scope
        .includes
        .iter()
        .map(|include| include.origin)
        .collect::<BTreeSet<_>>();
    routes.modules.extend(scope.modules);
    routes.includes.extend(scope.includes);
    scan_body_route_groups(
        tokens,
        start,
        end,
        BodyRouteContext {
            source: context.source,
            predicate: context.predicate,
            inline_scopes: &inline_scopes,
            known_include_origins: &known_include_origins,
            expression_fragment: context.expression_fragment,
            strict_unknown_macros: context.strict_unknown_macros,
            depth: context.depth + 1,
        },
        routes,
    );
}

fn scan_body_route_groups(
    tokens: &Tokens,
    start: usize,
    end: usize,
    context: BodyRouteContext<'_>,
    routes: &mut ParsedSource,
) {
    if context.depth >= 128 {
        routes.complete = false;
        return;
    }
    let mut index = start;
    while index < end {
        if is_punct(tokens, index, '#') {
            if let Some((next, _, attribute)) = parse_attribute(tokens, index, end) {
                routes.complete &= attribute.complete;
                index = next;
            } else {
                routes.complete = false;
                index += 1;
            }
            continue;
        }
        if let Some((path, next)) = parse_include_invocation(tokens, index, end) {
            if context.known_include_origins.contains(&index) {
                index = next;
                continue;
            }
            if let Some(path) = path {
                let source_dir = context
                    .source
                    .file
                    .parent()
                    .unwrap_or_else(|| Path::new("."))
                    .to_path_buf();
                routes.includes.push(IncludeDecl {
                    path,
                    source_dir,
                    module_dir: context.source.module_dir.to_path_buf(),
                    predicate: context.predicate,
                    origin: index,
                    expression_fragment: true,
                });
            } else {
                routes.complete = false;
            }
            index = next;
            continue;
        }
        if let Some((_, close)) = macro_invocation_group(tokens, index, end) {
            if context.strict_unknown_macros || macro_starts_block_item(tokens, start, index) {
                routes.complete = false;
            }
            index = close + 1;
            if is_punct(tokens, index, ';') {
                index += 1;
            }
            continue;
        }
        if is_open(tokens, index) {
            let Some(close) = tokens.pairs.get(index).and_then(|value| *value) else {
                routes.complete = false;
                index += 1;
                continue;
            };
            if is_punct(tokens, index, '{') {
                if let Some(inline) = context
                    .inline_scopes
                    .iter()
                    .find(|inline| inline.open == index && inline.close == close)
                {
                    scan_body_route_groups(
                        tokens,
                        index + 1,
                        close,
                        BodyRouteContext {
                            source: ParsePath {
                                file: context.source.file,
                                module_dir: &inline.module_dir,
                                file_module_dir: &inline.file_module_dir,
                                path_base: &inline.file_module_dir,
                            },
                            predicate: inline.predicate,
                            inline_scopes: context.inline_scopes,
                            known_include_origins: context.known_include_origins,
                            expression_fragment: false,
                            strict_unknown_macros: context.strict_unknown_macros,
                            depth: context.depth + 1,
                        },
                        routes,
                    );
                } else {
                    collect_scope_body_routes(
                        tokens,
                        index + 1,
                        close,
                        BodyRouteContext {
                            depth: context.depth + 1,
                            ..context
                        },
                        routes,
                    );
                }
            } else {
                scan_body_route_groups(
                    tokens,
                    index + 1,
                    close,
                    BodyRouteContext {
                        depth: context.depth + 1,
                        ..context
                    },
                    routes,
                );
            }
            index = close + 1;
            continue;
        }
        index += 1;
    }
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
    let Some((name, input, qualified)) = attribute_path(tokens, bracket + 1, close) else {
        attr.complete = false;
        attr.predicate = Predicate::UNKNOWN;
        return Some((close + 1, inner, attr));
    };
    if qualified {
        // Built-in attribute meaning comes from the complete unqualified
        // name. A tool or proc-macro path with a familiar first segment is
        // still unresolved and must not prove test-only reachability.
        attr.complete = false;
        attr.predicate = Predicate::UNKNOWN;
        return Some((close + 1, inner, attr));
    }
    match name {
        "cfg" => {
            let open = input;
            let cfg_close = tokens.pairs.get(open).and_then(|value| *value);
            if !is_punct(tokens, open, '(') || cfg_close != Some(close - 1) {
                attr.complete = false;
                attr.predicate = Predicate::UNKNOWN;
            } else if let Some(predicate) = parse_cfg(tokens, open + 1, close - 1) {
                attr.predicate = predicate;
                attr.literal_cfg_test =
                    ident_at(tokens, open + 1) == Some("test") && cfg_close == open.checked_add(2);
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
            if is_punct(tokens, input, '=')
                && let Some(TokenKind::String(Some(path))) =
                    tokens.values.get(input + 1).map(|token| &token.kind)
                && input + 2 == close
                && !path.is_empty()
                && !Path::new(path).is_absolute()
            {
                attr.path = Some(path.clone());
            } else {
                attr.complete = false;
                attr.predicate = Predicate::UNKNOWN;
            }
        }
        "test" | "bench" if input == close => {
            attr.predicate = Predicate::TEST;
            attr.test_attributes.insert(name.to_string());
        }
        "test" | "bench" => {
            attr.complete = false;
            attr.predicate = Predicate::UNKNOWN;
        }
        "derive" => match builtin_derive_names(tokens, input, close - 1) {
            Some(names) => attr.derives = names,
            None => {
                attr.complete = false;
                attr.predicate = Predicate::UNKNOWN;
            }
        },
        name if known_inert_attribute(name, tokens, input, close) => {}
        _ => {
            attr.complete = false;
            attr.predicate = Predicate::UNKNOWN;
        }
    }
    Some((close + 1, inner, attr))
}

fn attribute_path(tokens: &Tokens, start: usize, end: usize) -> Option<(&str, usize, bool)> {
    let name = ident_at(tokens, start)?;
    let mut next = start.checked_add(1)?;
    let mut qualified = false;
    while is_punct(tokens, next, ':') {
        if !is_punct(tokens, next + 1, ':') || ident_at(tokens, next + 2).is_none() {
            return None;
        }
        qualified = true;
        next = next.checked_add(3)?;
        if next > end {
            return None;
        }
    }
    Some((name, next, qualified))
}

fn known_inert_attribute(name: &str, tokens: &Tokens, input: usize, end: usize) -> bool {
    if input == end {
        return matches!(
            name,
            "ignore"
                | "should_panic"
                | "inline"
                | "cold"
                | "track_caller"
                | "deprecated"
                | "must_use"
                | "non_exhaustive"
                | "no_mangle"
                | "used"
                | "macro_export"
                | "automatically_derived"
                | "global_allocator"
                | "panic_handler"
                | "proc_macro"
                | "proc_macro_attribute"
                | "no_std"
                | "no_main"
                | "allow_internal_unsafe"
        );
    }

    if is_punct(tokens, input, '(') {
        let Some(group_end) = tokens.pairs.get(input).and_then(|value| *value) else {
            return false;
        };
        if group_end != end - 1 || input + 1 == group_end {
            return false;
        }
        return match name {
            "doc"
            | "allow"
            | "warn"
            | "deny"
            | "forbid"
            | "expect"
            | "should_panic"
            | "deprecated"
            | "must_use"
            | "repr"
            | "link"
            | "target_feature"
            | "macro_export"
            | "proc_macro_derive"
            | "allow_internal_unstable"
            | "allow_internal_unsafe"
            | "stable"
            | "unstable"
            | "coverage" => true,
            "inline" => {
                matches!(ident_at(tokens, input + 1), Some("always" | "never"))
                    && (input + 2 == group_end
                        || (input + 3 == group_end && is_punct(tokens, input + 2, ',')))
            }
            _ => false,
        };
    }

    if !is_punct(tokens, input, '=') || !supported_string_value_attribute(name) {
        return false;
    }
    if matches!(
        tokens.values.get(input + 1).map(|token| &token.kind),
        Some(TokenKind::String(Some(_)))
    ) && input + 2 == end
    {
        return true;
    }

    name == "doc" && complete_macro_expression(tokens, input + 1, end)
}

fn supported_string_value_attribute(name: &str) -> bool {
    matches!(
        name,
        "doc"
            | "ignore"
            | "should_panic"
            | "deprecated"
            | "must_use"
            | "export_name"
            | "link_name"
            | "crate_type"
            | "crate_name"
            | "recursion_limit"
            | "type_length_limit"
    )
}

fn complete_macro_expression(tokens: &Tokens, start: usize, end: usize) -> bool {
    let Some((_, bang, _)) = attribute_path(tokens, start, end) else {
        return false;
    };
    if !is_punct(tokens, bang, '!') || !is_punct(tokens, bang + 1, '(') {
        return false;
    }
    tokens.pairs.get(bang + 1).and_then(|value| *value) == Some(end - 1)
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

#[derive(Default)]
struct TestAttributeImports {
    names: BTreeSet<String>,
    wildcard: bool,
    uncertain: bool,
}

impl TestAttributeImports {
    fn may_shadow(&self, name: &str) -> bool {
        self.wildcard || self.uncertain || self.names.contains(name)
    }
}

fn record_test_attribute_imports(
    tokens: &Tokens,
    start: usize,
    end: usize,
    imports: &mut TestAttributeImports,
) {
    if item_head(tokens, start, end) != Some("use") {
        return;
    }
    match parse_test_attribute_imports(tokens, start, end) {
        Some(parsed) => {
            imports.names.extend(parsed.names);
            imports.wildcard |= parsed.wildcard;
            imports.uncertain |= parsed.uncertain;
        }
        None => imports.uncertain = true,
    }
}

// Resolve only the local binding produced by a complete use tree. Path
// segments that happen to spell `test` or `bench` do not shadow an attribute;
// wildcard and unrecognized trees stay conservative at the containing scope.
fn parse_test_attribute_imports(
    tokens: &Tokens,
    start: usize,
    end: usize,
) -> Option<TestAttributeImports> {
    if end <= start + 2 || !is_punct(tokens, end - 1, ';') {
        return None;
    }
    let mut use_index = start;
    if ident_at(tokens, use_index) == Some("pub") {
        use_index += 1;
        if is_punct(tokens, use_index, '(') {
            use_index = tokens.pairs.get(use_index).and_then(|value| *value)? + 1;
        }
    }
    if ident_at(tokens, use_index) != Some("use") {
        return None;
    }
    let mut imports = TestAttributeImports::default();
    let mut prefix = Vec::new();
    let next = parse_use_tree_list(tokens, use_index + 1, end - 1, &mut prefix, &mut imports, 0)?;
    (next == end - 1).then_some(imports)
}

fn parse_use_tree_list(
    tokens: &Tokens,
    mut index: usize,
    end: usize,
    prefix: &mut Vec<String>,
    imports: &mut TestAttributeImports,
    depth: usize,
) -> Option<usize> {
    if depth > 64 || prefix.len() > 64 || index >= end {
        return None;
    }
    loop {
        index = parse_use_tree(tokens, index, end, prefix, imports, depth)?;
        if index == end {
            return Some(index);
        }
        if !is_punct(tokens, index, ',') {
            return None;
        }
        index += 1;
        if index == end {
            return Some(index);
        }
    }
}

fn parse_use_tree(
    tokens: &Tokens,
    mut index: usize,
    end: usize,
    prefix: &mut Vec<String>,
    imports: &mut TestAttributeImports,
    depth: usize,
) -> Option<usize> {
    if depth > 64 || index >= end {
        return None;
    }
    if is_punct(tokens, index, '*') {
        imports.wildcard = true;
        return Some(index + 1);
    }
    if is_punct(tokens, index, '{') {
        let group = index;
        let close = tokens.pairs.get(group).and_then(|value| *value)?;
        if close >= end {
            return None;
        }
        let next = parse_use_tree_list(tokens, group + 1, close, prefix, imports, depth + 1)?;
        return (next == close).then_some(close + 1);
    }
    if is_punct(tokens, index, ':') {
        if !is_punct(tokens, index + 1, ':') {
            return None;
        }
        index += 2;
    }
    let mut path = prefix.clone();
    loop {
        let segment = ident_at(tokens, index)?;
        index += 1;
        if is_punct(tokens, index, ':') {
            if !is_punct(tokens, index + 1, ':') {
                return None;
            }
            if is_punct(tokens, index + 2, '*') {
                if index + 3 != end {
                    return None;
                }
                imports.wildcard = true;
                return Some(index + 3);
            }
            if is_punct(tokens, index + 2, '{') {
                let group = index + 2;
                let close = tokens.pairs.get(group).and_then(|value| *value)?;
                if close >= end {
                    return None;
                }
                path.push(segment.to_string());
                if path.len() > 64 {
                    return None;
                }
                let next =
                    parse_use_tree_list(tokens, group + 1, close, &mut path, imports, depth + 1)?;
                if next != close || close + 1 > end {
                    return None;
                }
                return Some(close + 1);
            }
            path.push(segment.to_string());
            if path.len() > 64 {
                return None;
            }
            index += 2;
            if index >= end {
                return None;
            }
            continue;
        }
        if ident_at(tokens, index) == Some("as") {
            let alias = ident_at(tokens, index + 1)?;
            if alias == "_" {
                return Some(index + 2);
            }
            record_imported_attribute(alias, imports);
            return Some(index + 2);
        }
        let local_name = if segment == "self" {
            path.last().map(String::as_str)?
        } else {
            segment
        };
        record_imported_attribute(local_name, imports);
        return Some(index);
    }
}

fn record_imported_attribute(name: &str, imports: &mut TestAttributeImports) {
    if matches!(name, "test" | "bench") {
        imports.names.insert(name.to_string());
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
    macro_invocation_group(tokens, at, end).is_some()
}

fn macro_invocation_group(tokens: &Tokens, at: usize, end: usize) -> Option<(usize, usize)> {
    let mut index = at;
    if is_punct(tokens, index, ':') && is_punct(tokens, index + 1, ':') {
        index += 2;
    }
    ident_at(tokens, index)?;
    index += 1;
    while is_punct(tokens, index, ':') && is_punct(tokens, index + 1, ':') {
        index += 2;
        ident_at(tokens, index)?;
        index += 1;
    }
    if !is_punct(tokens, index, '!') || !is_open(tokens, index + 1) || index + 1 >= end {
        return None;
    }
    let close = tokens.pairs.get(index + 1).and_then(|value| *value)?;
    (close < end).then_some((index, close))
}

fn parse_static_include(
    tokens: &Tokens,
    at: usize,
    end: usize,
    expression_fragment: bool,
) -> Option<String> {
    let (path, next) = parse_include_invocation(tokens, at, end)?;
    if !is_punct(tokens, next.saturating_sub(1), ';') && !(expression_fragment && next == end) {
        return None;
    }
    path
}

fn parse_include_invocation(
    tokens: &Tokens,
    at: usize,
    end: usize,
) -> Option<(Option<String>, usize)> {
    if ident_at(tokens, at) != Some("include")
        || !is_punct(tokens, at + 1, '!')
        || !is_punct(tokens, at + 2, '(')
    {
        return None;
    }
    let close = tokens.pairs.get(at + 2).and_then(|value| *value)?;
    if close >= end {
        return None;
    }
    let path = if close == at + 4 {
        match &tokens.values.get(at + 3)?.kind {
            TokenKind::String(Some(path)) if !path.is_empty() => Some(path.clone()),
            _ => None,
        }
    } else {
        None
    };
    let mut next = close + 1;
    if is_punct(tokens, next, ';') {
        next += 1;
    }
    Some((path, next))
}

fn skipped_item_has_unmodeled_routes(
    tokens: &Tokens,
    start: usize,
    end: usize,
    enclosing_imports: &TestAttributeImports,
) -> bool {
    let mut index = start;
    let mut test_attributes = BTreeSet::new();
    let mut local_imports = TestAttributeImports::default();
    while index < end {
        if is_punct(tokens, index, '#') {
            let Some((next, inner, attribute)) = parse_attribute(tokens, index, end) else {
                return true;
            };
            let bracket = index + usize::from(inner) + 1;
            let is_derive = ident_at(tokens, bracket + 1) == Some("derive");
            if !attribute.complete || (!inner && is_derive) {
                return true;
            }
            if !inner {
                test_attributes.extend(attribute.test_attributes);
            }
            // Advance over the complete attribute, including its arguments. A
            // hash-like token inside known inert metadata is not a second
            // attached attribute.
            index = next;
            continue;
        }
        if item_head(tokens, index, end) == Some("use") {
            let Some(item_end) = skip_item(tokens, index, end) else {
                return true;
            };
            if item_end <= index {
                return true;
            }
            record_test_attribute_imports(tokens, index, item_end, &mut local_imports);
            index = item_end;
            continue;
        }
        if ident_at(tokens, index) == Some("mod") {
            if ident_at(tokens, index + 1).is_none() {
                return true;
            }
            if is_punct(tokens, index + 2, ';') {
                index += 3;
                continue;
            }
            if is_punct(tokens, index + 2, '{')
                && let Some(close) = tokens.pairs.get(index + 2).and_then(|value| *value)
                && close < end
            {
                index = close + 1;
                continue;
            }
            return true;
        }
        if let Some((path, next)) = parse_include_invocation(tokens, index, end) {
            if path.is_none() {
                return true;
            }
            index = next;
            continue;
        }
        if is_item_macro(tokens, index, end) && macro_starts_block_item(tokens, start, index) {
            return true;
        }
        index += 1;
    }
    test_attributes
        .iter()
        .any(|name| enclosing_imports.may_shadow(name) || local_imports.may_shadow(name))
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
    fn inline_cfg_test_range_does_not_resolve_descendant_attributes() {
        let source = "fn before() {} #[allow(dead_code)] #[cfg(test)] mod helpers {\n\
                      use super::*;\n\
                      #[test] fn nested_test() {}\n\
                      #[derive(Clone)] struct Derived;\n\
                      #[custom_rewrite] fn rewritten() {}\n\
                      } fn after() {}\n";
        let actual = ranges(source);
        assert_eq!(actual, [LineRange { start: 2, end: 5 }]);
        assert!(!actual.iter().any(|range| range.contains(1)));
        assert!(!actual.iter().any(|range| range.contains(6)));
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
            paths: BTreeSet::new(),
            reachable: BTreeMap::new(),
            visited: BTreeSet::new(),
            queue: VecDeque::new(),
            complete: true,
        };
        let file = PathBuf::from("/member/src/lib.rs");
        let module_dir = PathBuf::from("/member/src");
        traversal.queue_file(
            file.clone(),
            module_dir.clone(),
            module_dir.clone(),
            Predicate::ALWAYS,
            false,
        );
        traversal.queue_file(
            file,
            module_dir.clone(),
            module_dir.clone(),
            Predicate::ALWAYS,
            false,
        );
        assert_eq!(traversal.queue.len(), 1);

        for index in 0..=MAX_CONTEXTS {
            traversal.queue_file(
                PathBuf::from(format!("/member/src/{index}.rs")),
                PathBuf::from(format!("/member/src/{index}")),
                PathBuf::from(format!("/member/src/{index}")),
                Predicate::ALWAYS,
                false,
            );
        }
        assert!(!traversal.complete);
        assert_eq!(traversal.queue.len(), MAX_CONTEXTS);
    }

    #[test]
    fn expression_fragment_contexts_do_not_deduplicate_with_item_contexts() {
        let mut traversal = Traversal {
            member_root: PathBuf::from("/member"),
            sources: BTreeMap::new(),
            paths: BTreeSet::new(),
            reachable: BTreeMap::new(),
            visited: BTreeSet::new(),
            queue: VecDeque::new(),
            complete: true,
        };
        let file = PathBuf::from("/member/outside/shared.inc.rs");
        let module_dir = PathBuf::from("/member/src/caller");
        let file_module_dir = PathBuf::from("/member/outside");
        traversal.queue_file(
            file.clone(),
            module_dir.clone(),
            file_module_dir.clone(),
            Predicate::ALWAYS,
            false,
        );
        traversal.queue_file(file, module_dir, file_module_dir, Predicate::ALWAYS, true);
        assert_eq!(traversal.queue.len(), 2);
        assert_ne!(
            traversal.queue[0].expression_fragment,
            traversal.queue[1].expression_fragment
        );
    }

    #[test]
    fn token_limit_fails_closed() {
        let source = "#[cfg(test)]\nfn test_only() {}\n".repeat(MAX_TOKENS / 4);
        assert!(ranges(&source).is_empty());
    }
}
