//! Source scan behind the showcase coverage gate (`app_tests_catalog.rs`).
//!
//! A small token scanner, not a Rust parser: it strips comments, strings
//! and character literals, splits the rest into identifiers and
//! punctuation, and walks item headers with balanced-bracket skipping. It
//! finds exactly what `junie_tui::catalog` must list:
//!
//! - `pub struct` and `pub enum` (not `pub(crate)`), and every variant of
//!   a public enum;
//! - builders: `pub fn`/`pub const fn` in an inherent `impl`, taking
//!   `self` or `mut self` and returning `Self` (or the impl's own type);
//! - the public fields of `theme::Theme` (theme tokens).
//!
//! Private inline modules, `#[cfg(test)]` items, function bodies, trait
//! impls and macro bodies are skipped. Out-of-line `pub mod x;` children
//! are followed from the roots (`core`, `ui`, `widgets`, `theme`,
//! `runtime`).

use std::collections::HashMap;
use std::path::Path;

use junie_tui::catalog::CapKind;

/// One public item found in the source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub id: String,
    pub kind: CapKind,
    /// `file:line`, for messages.
    pub at: String,
}

/// Structs whose public fields are capabilities (theme tokens).
const TOKEN_STRUCTS: &[&str] = &["theme::Theme"];

/// Module roots the gate scans (below `src/`).
pub const ROOTS: &[&str] = &["core", "ui", "widgets", "theme", "runtime"];

/// Scans the library source under `src_dir` from [`ROOTS`].
pub fn scan_tree(src_dir: &Path) -> Vec<Item> {
    let mut raw = Vec::new();
    for root in ROOTS {
        scan_module(src_dir, &[root.to_string()], &mut raw);
    }
    resolve(raw)
}

fn scan_module(dir: &Path, module: &[String], out: &mut Vec<RawItem>) {
    let rel = module.join("/");
    let flat = dir.join(format!("{rel}.rs"));
    let nested = dir.join(format!("{rel}/mod.rs"));
    let file = if flat.exists() { flat } else { nested };
    let src = std::fs::read_to_string(&file)
        .unwrap_or_else(|e| panic!("catalog scan: cannot read {}: {e}", file.display()));
    let label = file
        .strip_prefix(dir.parent().unwrap_or(dir))
        .unwrap_or(&file)
        .display()
        .to_string();
    let children = scan_raw(module, &src, &label, out);
    for child in children {
        let mut path = module.to_vec();
        path.push(child);
        scan_module(dir, &path, out);
    }
}

/// Scans one source text as module `module` (fixtures use this directly).
pub fn scan_source(module: &[&str], src: &str) -> Vec<Item> {
    let module: Vec<String> = module.iter().map(|s| s.to_string()).collect();
    let mut raw = Vec::new();
    scan_raw(&module, src, "fixture.rs", &mut raw);
    resolve(raw)
}

#[derive(Debug, Clone)]
struct RawItem {
    module: Vec<String>,
    /// For members: the impl or enum type; `None` for types.
    owner: Option<String>,
    name: String,
    kind: CapKind,
    at: String,
}

/// Builds ids. A builder in an `impl` outside the type's own module is
/// filed under the module that defines the type when that is unambiguous.
fn resolve(raw: Vec<RawItem>) -> Vec<Item> {
    let mut homes: HashMap<&str, Vec<&[String]>> = HashMap::new();
    for item in raw.iter().filter(|i| i.kind == CapKind::Type) {
        homes.entry(&item.name).or_default().push(&item.module);
    }
    let mut out: Vec<Item> = Vec::new();
    for item in &raw {
        let module = match (&item.owner, item.kind) {
            (Some(owner), CapKind::Builder) => {
                let here = homes
                    .get(owner.as_str())
                    .is_some_and(|m| m.contains(&item.module.as_slice()));
                match homes.get(owner.as_str()) {
                    Some(m) if !here && m.len() == 1 => m[0].join("::"),
                    _ => item.module.join("::"),
                }
            }
            _ => item.module.join("::"),
        };
        let id = match &item.owner {
            Some(owner) => format!("{module}::{owner}::{}", item.name),
            None => format!("{module}::{}", item.name),
        };
        if !out.iter().any(|o| o.id == id) {
            out.push(Item {
                id,
                kind: item.kind,
                at: item.at.clone(),
            });
        }
    }
    out
}

// ---------------------------------------------------------------- lexing

#[derive(Debug, Clone, PartialEq, Eq)]
enum Tok {
    Ident(String),
    Punct(char),
}

#[derive(Debug, Clone)]
struct Token {
    tok: Tok,
    line: usize,
}

/// Replaces comments, string and char literals with spaces (keeping
/// newlines), so braces inside them never count.
fn strip(src: &str) -> String {
    let c: Vec<char> = src.chars().collect();
    let mut out = String::with_capacity(src.len());
    let mut i = 0;
    let blank = |out: &mut String, ch: char| out.push(if ch == '\n' { '\n' } else { ' ' });
    while i < c.len() {
        let ch = c[i];
        let next = c.get(i + 1).copied();
        if ch == '/' && next == Some('/') {
            while i < c.len() && c[i] != '\n' {
                i += 1;
                out.push(' ');
            }
        } else if ch == '/' && next == Some('*') {
            let mut depth = 0;
            while i < c.len() {
                if c[i] == '/' && c.get(i + 1) == Some(&'*') {
                    depth += 1;
                    out.push_str("  ");
                    i += 2;
                } else if c[i] == '*' && c.get(i + 1) == Some(&'/') {
                    depth -= 1;
                    out.push_str("  ");
                    i += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    blank(&mut out, c[i]);
                    i += 1;
                }
            }
        } else if ch == 'r'
            && (next == Some('"') || next == Some('#'))
            && !prev_is_ident(&c, i)
            && raw_string_start(&c, i + 1).is_some()
        {
            let hashes = raw_string_start(&c, i + 1).unwrap_or(0);
            // r, hashes, opening quote
            for _ in 0..(2 + hashes) {
                out.push(' ');
            }
            i += 2 + hashes;
            while i < c.len() {
                if c[i] == '"' && (0..hashes).all(|k| c.get(i + 1 + k) == Some(&'#')) {
                    for _ in 0..(1 + hashes) {
                        out.push(' ');
                    }
                    i += 1 + hashes;
                    break;
                }
                blank(&mut out, c[i]);
                i += 1;
            }
        } else if ch == '"' {
            out.push(' ');
            i += 1;
            while i < c.len() {
                if c[i] == '\\' {
                    out.push(' ');
                    if let Some(&e) = c.get(i + 1) {
                        blank(&mut out, e);
                    }
                    i += 2;
                } else if c[i] == '"' {
                    out.push(' ');
                    i += 1;
                    break;
                } else {
                    blank(&mut out, c[i]);
                    i += 1;
                }
            }
        } else if ch == '\'' {
            // Char literal (`'x'`, `'\n'`, `'\u{..}'`) or a lifetime.
            if next == Some('\\') {
                out.push(' ');
                i += 1;
                while i < c.len() && c[i] != '\'' {
                    out.push(' ');
                    i += if c[i] == '\\' { 2 } else { 1 };
                }
                out.push(' ');
                i += 1;
            } else if c.get(i + 2) == Some(&'\'') {
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

fn prev_is_ident(c: &[char], i: usize) -> bool {
    i > 0 && (c[i - 1].is_alphanumeric() || c[i - 1] == '_')
}

/// At `c[i]` (just after an `r`): `#…#"` or `"` opens a raw string;
/// returns the number of hashes.
fn raw_string_start(c: &[char], i: usize) -> Option<usize> {
    let mut n = 0;
    while c.get(i + n) == Some(&'#') {
        n += 1;
    }
    (c.get(i + n) == Some(&'"')).then_some(n)
}

fn tokenize(src: &str) -> Vec<Token> {
    let clean = strip(src);
    let mut out = Vec::new();
    let mut line = 1;
    let mut chars = clean.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\n' {
            line += 1;
        } else if ch.is_alphabetic() || ch == '_' {
            let mut s = String::from(ch);
            while let Some(&n) = chars.peek() {
                if n.is_alphanumeric() || n == '_' {
                    s.push(n);
                    chars.next();
                } else {
                    break;
                }
            }
            out.push(Token {
                tok: Tok::Ident(s),
                line,
            });
        } else if ch.is_ascii_digit() {
            while chars
                .peek()
                .is_some_and(|n| n.is_alphanumeric() || *n == '_')
            {
                chars.next();
            }
        } else if !ch.is_whitespace() {
            out.push(Token {
                tok: Tok::Punct(ch),
                line,
            });
        }
    }
    out
}

// ---------------------------------------------------------------- parsing

fn is_punct(t: &Token, c: char) -> bool {
    t.tok == Tok::Punct(c)
}

fn ident(t: &Token) -> Option<&str> {
    match &t.tok {
        Tok::Ident(s) => Some(s),
        Tok::Punct(_) => None,
    }
}

/// Index just past the bracket group opening at `toks[i]`.
fn skip_group(toks: &[Token], i: usize) -> usize {
    let mut depth = 0i32;
    let mut j = i;
    while j < toks.len() {
        match toks[j].tok {
            Tok::Punct('(' | '[' | '{') => depth += 1,
            Tok::Punct(')' | ']' | '}') => {
                depth -= 1;
                if depth == 0 {
                    return j + 1;
                }
            }
            _ => {}
        }
        j += 1;
    }
    toks.len()
}

/// Index just past an angle-bracket group opening at `toks[i]` (`<`).
fn skip_angles(toks: &[Token], i: usize) -> usize {
    let mut depth = 0i32;
    let mut j = i;
    while j < toks.len() {
        if is_punct(&toks[j], '-') && toks.get(j + 1).is_some_and(|t| is_punct(t, '>')) {
            j += 2;
            continue;
        }
        match toks[j].tok {
            Tok::Punct('<') => depth += 1,
            Tok::Punct('>') => {
                depth -= 1;
                if depth == 0 {
                    return j + 1;
                }
            }
            Tok::Punct('(' | '[') => {
                j = skip_group(toks, j);
                continue;
            }
            Tok::Punct('{' | ';') => return j,
            _ => {}
        }
        j += 1;
    }
    toks.len()
}

/// The header of an item with its outer attributes removed, and whether
/// one of them was `#[cfg(test)]`.
fn split_attrs(header: &[Token]) -> (Vec<Token>, bool) {
    let mut out = Vec::new();
    let mut test = false;
    let mut i = 0;
    while i < header.len() {
        if is_punct(&header[i], '#') {
            let mut j = i + 1;
            if header.get(j).is_some_and(|t| is_punct(t, '!')) {
                j += 1;
            }
            if header.get(j).is_some_and(|t| is_punct(t, '[')) {
                let end = skip_group(header, j);
                let inner: Vec<&str> = header[j..end].iter().filter_map(ident).collect();
                if inner == ["cfg", "test"] {
                    test = true;
                }
                i = end;
                continue;
            }
        }
        out.push(header[i].clone());
        i += 1;
    }
    (out, test)
}

#[derive(Debug, PartialEq, Eq)]
enum Vis {
    Pub,
    Restricted,
    Private,
}

/// Visibility and the index of the first token after it.
fn visibility(h: &[Token]) -> (Vis, usize) {
    match h.first().and_then(ident) {
        Some("pub") if h.get(1).is_some_and(|t| is_punct(t, '(')) => {
            (Vis::Restricted, skip_group(h, 1))
        }
        Some("pub") => (Vis::Pub, 1),
        _ => (Vis::Private, 0),
    }
}

/// Skips the `const`/`async`/`unsafe`/`extern`/`default` qualifiers of a
/// `fn` or `impl` starting at `i`.
fn skip_quals(h: &[Token], mut i: usize) -> usize {
    loop {
        let here = h.get(i).and_then(ident);
        let next = h.get(i + 1).and_then(ident);
        match here {
            Some("const") if matches!(next, Some("fn" | "unsafe" | "async" | "extern")) => i += 1,
            Some("async" | "unsafe" | "extern" | "default") => i += 1,
            _ => return i,
        }
    }
}

struct Ctx<'a> {
    label: &'a str,
    out: &'a mut Vec<RawItem>,
    children: Vec<String>,
}

impl Ctx<'_> {
    fn push(
        &mut self,
        module: &[String],
        owner: Option<&str>,
        name: &str,
        kind: CapKind,
        line: usize,
    ) {
        self.out.push(RawItem {
            module: module.to_vec(),
            owner: owner.map(str::to_string),
            name: name.to_string(),
            kind,
            at: format!("{}:{line}", self.label),
        });
    }
}

fn scan_raw(module: &[String], src: &str, label: &str, out: &mut Vec<RawItem>) -> Vec<String> {
    let toks = tokenize(src);
    let mut ctx = Ctx {
        label,
        out,
        children: Vec::new(),
    };
    walk_items(&toks, 0, toks.len(), module, true, &mut ctx);
    ctx.children
}

/// The item headers in `toks[start..end]`: yields (header range, the index
/// of the `{` or `;` that ends it).
fn next_header(toks: &[Token], start: usize, end: usize) -> Option<(usize, usize)> {
    let mut j = start;
    while j < end {
        match toks[j].tok {
            Tok::Punct('{' | ';') => return Some((start, j)),
            Tok::Punct('(' | '[') => j = skip_group(toks, j),
            _ => j += 1,
        }
    }
    None
}

/// Walks module-level items. `top` is false inside an inline module (whose
/// out-of-line children would be relative to it; we do not follow those).
fn walk_items(
    toks: &[Token],
    start: usize,
    end: usize,
    module: &[String],
    top: bool,
    ctx: &mut Ctx,
) {
    let mut i = start;
    while let Some((h0, stop)) = next_header(toks, i, end) {
        let (header, test) = split_attrs(&toks[h0..stop]);
        let at_brace = is_punct(&toks[stop], '{');
        let body_end = if at_brace {
            skip_group(toks, stop)
        } else {
            stop + 1
        };
        i = body_end;
        if test || header.is_empty() {
            continue;
        }
        let (vis, k) = visibility(&header);
        let k = skip_quals(&header, k);
        let kw = header.get(k).and_then(ident);
        let name = header.get(k + 1).and_then(ident);
        let line = header.get(k).map_or(0, |t| t.line);
        match (kw, at_brace) {
            (Some("mod"), false) if vis == Vis::Pub && top => {
                if let Some(n) = name {
                    ctx.children.push(n.to_string());
                }
            }
            (Some("mod"), true) if vis == Vis::Pub => {
                if let Some(n) = name {
                    let mut inner = module.to_vec();
                    inner.push(n.to_string());
                    walk_items(toks, stop + 1, body_end - 1, &inner, false, ctx);
                }
            }
            (Some("struct"), _) if vis == Vis::Pub => {
                if let Some(n) = name {
                    ctx.push(module, None, n, CapKind::Type, line);
                    let full = format!("{}::{n}", module.join("::"));
                    if at_brace && TOKEN_STRUCTS.contains(&full.as_str()) {
                        for (field, fl) in fields(toks, stop + 1, body_end - 1) {
                            ctx.push(module, Some(n), &field, CapKind::Token, fl);
                        }
                    }
                }
            }
            (Some("enum"), true) if vis == Vis::Pub => {
                if let Some(n) = name {
                    ctx.push(module, None, n, CapKind::Type, line);
                    for (variant, vl) in variants(toks, stop + 1, body_end - 1) {
                        ctx.push(module, Some(n), &variant, CapKind::Variant, vl);
                    }
                }
            }
            (Some("impl"), true) => {
                if let Some(ty) = impl_type(&header[k..]) {
                    walk_impl(toks, stop + 1, body_end - 1, module, &ty, ctx);
                }
            }
            // `const X: T = T { .. };` and the like: the brace was an
            // initializer; the `;` after it reads as an empty header.
            _ => {}
        }
    }
}

/// The self type of an inherent impl header (`impl<..> Type<..>`); `None`
/// for a trait impl.
fn impl_type(h: &[Token]) -> Option<String> {
    let mut i = 1;
    if h.get(i).is_some_and(|t| is_punct(t, '<')) {
        i = skip_angles(h, i);
    }
    let mut last = None;
    while i < h.len() {
        match &h[i].tok {
            Tok::Ident(s) if s == "for" => return None,
            Tok::Ident(s) if s == "where" => break,
            Tok::Ident(s) => {
                last = Some(s.clone());
                i += 1;
            }
            Tok::Punct('<') => i = skip_angles(h, i),
            _ => i += 1,
        }
    }
    last
}

fn walk_impl(toks: &[Token], start: usize, end: usize, module: &[String], ty: &str, ctx: &mut Ctx) {
    let mut i = start;
    while let Some((h0, stop)) = next_header(toks, i, end) {
        let (header, test) = split_attrs(&toks[h0..stop]);
        let at_brace = is_punct(&toks[stop], '{');
        i = if at_brace {
            skip_group(toks, stop)
        } else {
            stop + 1
        };
        if test {
            continue;
        }
        let (vis, k) = visibility(&header);
        let k = skip_quals(&header, k);
        if vis != Vis::Pub || header.get(k).and_then(ident) != Some("fn") {
            continue;
        }
        if let Some(name) = header.get(k + 1).and_then(ident)
            && is_builder(&header[k + 2..], ty)
        {
            ctx.push(module, Some(ty), name, CapKind::Builder, header[k].line);
        }
    }
}

/// `<generics>? (self | mut self, ..) -> Self | Type<..>`.
fn is_builder(sig: &[Token], ty: &str) -> bool {
    let mut i = 0;
    if sig.first().is_some_and(|t| is_punct(t, '<')) {
        i = skip_angles(sig, 0);
    }
    if !sig.get(i).is_some_and(|t| is_punct(t, '(')) {
        return false;
    }
    let close = skip_group(sig, i);
    let params: Vec<&Token> = sig[i + 1..close.saturating_sub(1)].iter().collect();
    let takes_self = match params.first().and_then(|t| ident(t)) {
        Some("self") => true,
        Some("mut") => params.get(1).and_then(|t| ident(t)) == Some("self"),
        _ => false,
    };
    if !takes_self || params.get(1).is_some_and(|t| is_punct(t, ':')) {
        // `self: Box<Self>` and friends are not builders.
        return false;
    }
    let ret = &sig[close..];
    if !(ret.first().is_some_and(|t| is_punct(t, '-'))
        && ret.get(1).is_some_and(|t| is_punct(t, '>')))
    {
        return false;
    }
    let ret = &ret[2..];
    let ret_end = ret
        .iter()
        .position(|t| ident(t) == Some("where"))
        .unwrap_or(ret.len());
    let ret = &ret[..ret_end];
    match ret.first().and_then(ident) {
        Some("Self") => ret.len() == 1,
        Some(name) if name == ty => {
            ret.len() == 1 || (is_punct(&ret[1], '<') && skip_angles(ret, 1) == ret.len())
        }
        _ => false,
    }
}

/// Splits `toks[start..end]` on top-level commas.
fn segments(toks: &[Token], start: usize, end: usize, angles: bool) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut seg = start;
    let mut j = start;
    let mut angle = 0i32;
    while j < end {
        match toks[j].tok {
            Tok::Punct('(' | '[' | '{') => {
                j = skip_group(toks, j);
                continue;
            }
            Tok::Punct('-') if toks.get(j + 1).is_some_and(|t| is_punct(t, '>')) => {
                j += 2;
                continue;
            }
            Tok::Punct('<') if angles => angle += 1,
            Tok::Punct('>') if angles => angle -= 1,
            Tok::Punct(',') if angle <= 0 => {
                out.push((seg, j));
                seg = j + 1;
            }
            _ => {}
        }
        j += 1;
    }
    if seg < end {
        out.push((seg, end));
    }
    out
}

fn variants(toks: &[Token], start: usize, end: usize) -> Vec<(String, usize)> {
    segments(toks, start, end, false)
        .into_iter()
        .filter_map(|(a, b)| {
            let (seg, test) = split_attrs(&toks[a..b]);
            if test {
                return None;
            }
            seg.first()
                .and_then(|t| ident(t).map(|s| (s.to_string(), t.line)))
        })
        .collect()
}

fn fields(toks: &[Token], start: usize, end: usize) -> Vec<(String, usize)> {
    segments(toks, start, end, true)
        .into_iter()
        .filter_map(|(a, b)| {
            let (seg, test) = split_attrs(&toks[a..b]);
            if test {
                return None;
            }
            let (vis, k) = visibility(&seg);
            if vis != Vis::Pub || !seg.get(k + 1).is_some_and(|t| is_punct(t, ':')) {
                return None;
            }
            seg.get(k)
                .and_then(|t| ident(t).map(|s| (s.to_string(), t.line)))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(src: &str) -> Vec<String> {
        scan_source(&["widgets", "demo"], src)
            .into_iter()
            .map(|i| format!("{}:{}", i.kind.name(), i.id))
            .collect()
    }

    #[test]
    fn finds_types_variants_and_builders() {
        let src = r#"
            /// Doc with a { brace.
            pub struct Demo<'a> { label: &'a str }
            pub(crate) struct Hidden;
            struct Private;
            pub enum Mode { #[default] A, B(u8), C { x: Vec<(u8, u8)> }, D = 4 }
            impl<'a> Demo<'a> {
                pub fn new() -> Self { Self { label: "}" } }
                pub fn label(mut self, l: &'a str) -> Self { self.label = l; self }
                pub const fn wide(self) -> Demo<'a> { self }
                pub fn get(&self) -> &str { self.label }
                pub fn into_mode(self) -> Mode { Mode::A }
                pub(crate) fn hidden(self) -> Self { self }
                fn private(self) -> Self { let c = '{'; self }
            }
            impl Default for Mode { fn default() -> Self { Mode::A } }
            mod inner { pub struct NotPublic; }
            pub mod open { pub struct Seen; }
            #[cfg(test)]
            mod tests { pub struct InTest; }
        "#;
        assert_eq!(
            ids(src),
            [
                "type:widgets::demo::Demo",
                "type:widgets::demo::Mode",
                "variant:widgets::demo::Mode::A",
                "variant:widgets::demo::Mode::B",
                "variant:widgets::demo::Mode::C",
                "variant:widgets::demo::Mode::D",
                "builder:widgets::demo::Demo::label",
                "builder:widgets::demo::Demo::wide",
                "type:widgets::demo::open::Seen",
            ]
        );
    }

    #[test]
    fn const_initializer_braces_do_not_end_the_item() {
        let src = "pub const X: Foo = Foo { a: 1 }; pub struct After;";
        assert_eq!(ids(src), ["type:widgets::demo::After"]);
    }

    #[test]
    fn theme_fields_are_tokens() {
        let src = "pub struct Theme { pub accent: Style, pub(crate) x: u8, \
                   pub map: HashMap<u8, u8>, pub last: Color }";
        let found: Vec<String> = scan_source(&["theme"], src)
            .into_iter()
            .map(|i| format!("{}:{}", i.kind.name(), i.id))
            .collect();
        assert_eq!(
            found,
            [
                "type:theme::Theme",
                "token:theme::Theme::accent",
                "token:theme::Theme::map",
                "token:theme::Theme::last",
            ]
        );
    }
}
