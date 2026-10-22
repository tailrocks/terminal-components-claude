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

const INSTANT_ALLOWED: [(&str, usize, &str); 1] = [(
    "crates/termrock-session/src/session.rs",
    1,
    "session loop scheduling; controls when another frame is drawn, not frame content",
)];

const REVIEWED_PATTERNS: [&str; 2] = ["Instant::now", "target_os"];

const TARGET_OS_ALLOWED: [(&str, usize, &str); 0] = [];

fn mask_range(bytes: &mut [u8], range: std::ops::Range<usize>) {
    for byte in &mut bytes[range] {
        if *byte != b'\n' && *byte != b'\r' {
            *byte = b' ';
        }
    }
}

fn identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte >= 0x80
}

fn raw_string_start(bytes: &[u8], at: usize) -> Option<(usize, usize)> {
    let (r_at, prefix_start) = if bytes.get(at) == Some(&b'r') {
        (at, at)
    } else if matches!(bytes.get(at), Some(b'b' | b'c')) && bytes.get(at + 1) == Some(&b'r') {
        (at + 1, at)
    } else {
        return None;
    };
    if prefix_start > 0 && identifier_byte(bytes[prefix_start - 1]) {
        return None;
    }
    let mut quote = r_at + 1;
    while bytes.get(quote) == Some(&b'#') {
        quote += 1;
    }
    (bytes.get(quote) == Some(&b'"')).then_some((quote, quote - r_at - 1))
}

fn raw_string_end(bytes: &[u8], quote: usize, hashes: usize) -> Result<usize, String> {
    let mut at = quote + 1;
    while at < bytes.len() {
        if bytes[at] == b'"'
            && bytes
                .get(at + 1..at + 1 + hashes)
                .is_some_and(|suffix| suffix.iter().all(|byte| *byte == b'#'))
        {
            return Ok(at + 1 + hashes);
        }
        at += 1;
    }
    Err(format!("unterminated raw string at byte {quote}"))
}

fn quoted_string_end(bytes: &[u8], quote: usize) -> Result<usize, String> {
    let mut at = quote + 1;
    while at < bytes.len() {
        match bytes[at] {
            b'\\' => at = (at + 2).min(bytes.len()),
            b'"' => return Ok(at + 1),
            _ => at += 1,
        }
    }
    Err(format!("unterminated string at byte {quote}"))
}

fn char_literal_end(text: &str, at: usize) -> Result<Option<usize>, String> {
    let bytes = text.as_bytes();
    if bytes.get(at) != Some(&b'\'') {
        return Ok(None);
    }
    let Some(&next) = bytes.get(at + 1) else {
        return Ok(None);
    };
    if next == b'\\' {
        let escape = bytes.get(at + 2).copied();
        let end = match escape {
            Some(b'x')
                if bytes
                    .get(at + 3..at + 5)
                    .is_some_and(|digits| digits.iter().all(u8::is_ascii_hexdigit)) =>
            {
                at + 5
            }
            Some(b'u') if bytes.get(at + 3) == Some(&b'{') => {
                let Some(close) = bytes[at + 4..].iter().position(|byte| *byte == b'}') else {
                    return Err(format!(
                        "unterminated Unicode character escape at byte {at}"
                    ));
                };
                let digits = &bytes[at + 4..at + 4 + close];
                if !digits.iter().any(u8::is_ascii_hexdigit)
                    || !digits
                        .iter()
                        .all(|byte| byte.is_ascii_hexdigit() || *byte == b'_')
                {
                    return Err(format!("malformed Unicode character escape at byte {at}"));
                }
                at + 5 + close
            }
            Some(b'n' | b'r' | b't' | b'0' | b'\\' | b'\'' | b'"') => at + 3,
            Some(_) => return Err(format!("unknown character escape at byte {at}")),
            None => return Err(format!("unterminated character escape at byte {at}")),
        };
        if bytes.get(end) == Some(&b'\'') {
            return Ok(Some(end + 1));
        }
        return Err(format!("malformed character literal at byte {at}"));
    }
    let tail = &text[at + 1..];
    let Some(character) = tail.chars().next() else {
        return Ok(None);
    };
    if character == '\n' || character == '\r' {
        return Err(format!("newline in character literal at byte {at}"));
    }
    if character == '\'' {
        return Err(format!("empty character literal at byte {at}"));
    }
    let end = at + 1 + character.len_utf8();
    if bytes.get(end) == Some(&b'\'') {
        Ok(Some(end + 1))
    } else {
        // This is a lifetime or label, not a character literal.
        Ok(None)
    }
}

fn mask_non_code(text: &str) -> Result<Vec<u8>, String> {
    let bytes = text.as_bytes();
    let mut masked = bytes.to_vec();
    let mut at = 0;
    while at < bytes.len() {
        if bytes.get(at..at + 2) == Some(b"//") {
            let start = at;
            while at < bytes.len() && bytes[at] != b'\n' {
                at += 1;
            }
            mask_range(&mut masked, start..at);
        } else if bytes.get(at..at + 2) == Some(b"/*") {
            let start = at;
            let mut depth = 1usize;
            at += 2;
            while at < bytes.len() && depth > 0 {
                if bytes.get(at..at + 2) == Some(b"/*") {
                    depth += 1;
                    at += 2;
                } else if bytes.get(at..at + 2) == Some(b"*/") {
                    depth -= 1;
                    at += 2;
                } else {
                    at += 1;
                }
            }
            if depth != 0 {
                return Err(format!("unterminated block comment at byte {start}"));
            }
            mask_range(&mut masked, start..at);
        } else if let Some((quote, hashes)) = raw_string_start(bytes, at) {
            let end = raw_string_end(bytes, quote, hashes)?;
            let start = at;
            mask_range(&mut masked, start..end);
            at = end;
        } else if bytes[at] == b'"' {
            let end = quoted_string_end(bytes, at)?;
            mask_range(&mut masked, at..end);
            at = end;
        } else if bytes[at] == b'\'' {
            if let Some(end) = char_literal_end(text, at)? {
                mask_range(&mut masked, at..end);
                at = end;
            } else {
                at += 1;
            }
        } else {
            at += 1;
        }
    }
    Ok(masked)
}

pub fn code_only(text: &str) -> String {
    String::from_utf8(
        mask_non_code(text).unwrap_or_else(|error| panic!("lex Rust source: {error}")),
    )
    .expect("lexical mask preserves UTF-8")
}

#[derive(Clone, Copy, Debug)]
struct Token {
    start: usize,
    end: usize,
}

fn tokenize(code: &str) -> Result<(Vec<Token>, Vec<Option<usize>>), String> {
    let bytes = code.as_bytes();
    let mut tokens = Vec::new();
    let mut matching = Vec::new();
    let mut groups: Vec<(u8, usize)> = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at].is_ascii_whitespace() {
            at += 1;
            continue;
        }
        let start = at;
        if identifier_byte(bytes[at]) && !bytes[at].is_ascii_digit() {
            at += 1;
            while at < bytes.len() && identifier_byte(bytes[at]) {
                at += 1;
            }
        } else {
            at += 1;
        }
        let index = tokens.len();
        tokens.push(Token { start, end: at });
        matching.push(None);
        match bytes[start] {
            b'(' | b'[' | b'{' => groups.push((bytes[start], index)),
            b')' | b']' | b'}' => {
                let Some((open, open_index)) = groups.pop() else {
                    return Err(format!("unmatched delimiter at byte {start}"));
                };
                if !matches!(
                    (open, bytes[start]),
                    (b'(', b')') | (b'[', b']') | (b'{', b'}')
                ) {
                    return Err(format!("mismatched delimiter at byte {start}"));
                }
                matching[open_index] = Some(index);
                matching[index] = Some(open_index);
            }
            _ => {}
        }
    }
    if let Some((_, index)) = groups.pop() {
        return Err(format!(
            "unclosed delimiter at byte {}",
            tokens[index].start
        ));
    }
    Ok((tokens, matching))
}

fn token_text<'a>(code: &'a str, tokens: &[Token], index: usize) -> &'a str {
    &code[tokens[index].start..tokens[index].end]
}

fn attribute_end(
    code: &str,
    tokens: &[Token],
    matching: &[Option<usize>],
    at: usize,
) -> Option<usize> {
    if token_text(code, tokens, at) != "#"
        || tokens
            .get(at + 1)
            .is_none_or(|_| token_text(code, tokens, at + 1) != "[")
    {
        return None;
    }
    matching[at + 1]
}

#[derive(Debug)]
struct ModuleItem {
    name: String,
    start: usize,
    end: usize,
    external: bool,
    test_only: bool,
}

struct ModuleInventory {
    modules: Vec<ModuleItem>,
    test_ranges: Vec<std::ops::Range<usize>>,
    has_custom_path: bool,
}

fn cfg_test_attribute(
    code: &str,
    tokens: &[Token],
    matching: &[Option<usize>],
    at: usize,
) -> Option<usize> {
    let open = at + 1;
    let close = attribute_end(code, tokens, matching, at)?;
    let inside = &tokens[open + 1..close];
    if inside.len() == 4
        && token_text(code, inside, 0) == "cfg"
        && token_text(code, inside, 1) == "("
        && token_text(code, inside, 2) == "test"
        && token_text(code, inside, 3) == ")"
        && matching[open] == Some(close)
    {
        Some(close)
    } else {
        None
    }
}

fn identifier_token(code: &str, tokens: &[Token], at: usize) -> bool {
    tokens.get(at).is_some() && {
        token_text(code, tokens, at)
            .as_bytes()
            .first()
            .is_some_and(|byte| identifier_byte(*byte) && !byte.is_ascii_digit())
    }
}

fn module_body(
    code: &str,
    tokens: &[Token],
    matching: &[Option<usize>],
    mod_at: usize,
    start: usize,
    test_only: bool,
) -> Option<ModuleItem> {
    let mut name_at = mod_at + 1;
    let name;
    if token_text(code, tokens, name_at) == "r"
        && tokens
            .get(name_at + 1)
            .is_some_and(|_| token_text(code, tokens, name_at + 1) == "#")
        && identifier_token(code, tokens, name_at + 2)
    {
        name = token_text(code, tokens, name_at + 2).to_string();
        name_at += 3;
    } else {
        if !identifier_token(code, tokens, name_at) {
            return None;
        }
        name = token_text(code, tokens, name_at).to_string();
        name_at += 1;
    }
    let token = tokens.get(name_at)?;
    let (end, external) = match token_text(code, tokens, name_at) {
        ";" => (token.end, true),
        "{" => (tokens[matching[name_at]?].end, false),
        _ => return None,
    };
    Some(ModuleItem {
        name,
        start,
        end,
        external,
        test_only,
    })
}

fn module_after_attribute(
    code: &str,
    tokens: &[Token],
    matching: &[Option<usize>],
    after_attribute: usize,
    start: usize,
) -> Option<ModuleItem> {
    let mut at = after_attribute + 1;
    loop {
        if let Some(end) = attribute_end(code, tokens, matching, at) {
            at = end + 1;
        } else if tokens
            .get(at)
            .is_some_and(|_| token_text(code, tokens, at) == "pub")
        {
            at += 1;
            if tokens
                .get(at)
                .is_some_and(|_| token_text(code, tokens, at) == "(")
            {
                at = matching[at]? + 1;
            }
        } else {
            break;
        }
    }
    if tokens
        .get(at)
        .is_none_or(|_| token_text(code, tokens, at) != "mod")
    {
        return None;
    }
    module_body(code, tokens, matching, at, start, true)
}

fn macro_token_ranges(
    code: &str,
    tokens: &[Token],
    matching: &[Option<usize>],
) -> Vec<std::ops::Range<usize>> {
    let mut ranges = Vec::new();
    for at in 1..tokens.len() {
        if token_text(code, tokens, at) != "!" {
            continue;
        }
        let next = at + 1;
        let open = if token_text(code, tokens, at - 1) == "macro_rules" {
            next + 1
        } else {
            next
        };
        if tokens
            .get(open)
            .is_some_and(|_| matches!(token_text(code, tokens, open), "(" | "[" | "{"))
            && let Some(close) = matching[open]
        {
            ranges.push(tokens[open].start..tokens[close].end);
        }
    }
    ranges
}

fn module_inventory(text: &str) -> Result<ModuleInventory, String> {
    let code = String::from_utf8(mask_non_code(text)?).expect("lexical mask preserves UTF-8");
    let (tokens, matching) = tokenize(&code)?;
    let macro_ranges = macro_token_ranges(&code, &tokens, &matching);
    let mut modules = Vec::new();
    let mut test_ranges = Vec::new();
    let mut at = 0;
    while at + 1 < tokens.len() {
        if token_text(&code, &tokens, at) != "#"
            || token_text(&code, &tokens, at + 1) != "["
            || macro_ranges
                .iter()
                .any(|range| range.contains(&tokens[at].start))
        {
            at += 1;
            continue;
        }
        let Some(attribute) = cfg_test_attribute(&code, &tokens, &matching, at) else {
            at += 1;
            continue;
        };
        if let Some(module) =
            module_after_attribute(&code, &tokens, &matching, attribute, tokens[at].start)
        {
            test_ranges.push(module.start..module.end);
            modules.push(module);
        }
        at += 1;
    }
    for at in 0..tokens.len() {
        if token_text(&code, &tokens, at) != "mod"
            || macro_ranges
                .iter()
                .any(|range| range.contains(&tokens[at].start))
            || test_ranges
                .iter()
                .any(|range| range.contains(&tokens[at].start))
        {
            continue;
        }
        if let Some(module) = module_body(&code, &tokens, &matching, at, tokens[at].start, false) {
            modules.push(module);
        }
    }
    let has_custom_path = (0..tokens.len()).any(|at| {
        token_text(&code, &tokens, at) == "#"
            && token_text(&code, &tokens, at + 1) == "["
            && tokens.get(at + 2).is_some()
            && token_text(&code, &tokens, at + 2) == "path"
    });
    Ok(ModuleInventory {
        modules,
        test_ranges,
        has_custom_path,
    })
}

/// Blanks only syntactically complete cfg(test) module items, preserving every newline and byte offset.
pub fn non_test(text: &str) -> Result<String, String> {
    let inventory = module_inventory(text)?;
    let mut result = text.as_bytes().to_vec();
    for range in inventory.test_ranges {
        mask_range(&mut result, range);
    }
    String::from_utf8(result).map_err(|error| format!("masked Rust source is not UTF-8: {error}"))
}

fn contains_word(line: &str, pat: &str) -> bool {
    line.match_indices(pat).any(|(at, _)| {
        !line[..at]
            .chars()
            .next_back()
            .is_some_and(|c| c.is_alphanumeric() || c == '_')
    })
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

fn external_module_path(owner: &Path, name: &str) -> Option<std::path::PathBuf> {
    let parent = owner.parent()?;
    let file_name = owner.file_name()?.to_str()?;
    let base = if matches!(file_name, "lib.rs" | "main.rs" | "mod.rs") {
        parent.to_path_buf()
    } else {
        parent.join(owner.file_stem()?)
    };
    let flat = base.join(format!("{name}.rs"));
    let nested = base.join(name).join("mod.rs");
    match (flat.is_file(), nested.is_file()) {
        (true, false) => Some(flat),
        (false, true) => Some(nested),
        _ => None,
    }
}

pub(super) fn sources(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    collect_rs_files(dir, &mut files);
    files.sort();
    let mut test_only_modules = std::collections::BTreeSet::new();
    let mut production_modules = std::collections::BTreeSet::new();
    let mut unresolved_path_attribute = false;
    for file in &files {
        let text = std::fs::read_to_string(file)
            .unwrap_or_else(|error| panic!("read {}: {error}", file.display()));
        let inventory = module_inventory(&text)
            .unwrap_or_else(|error| panic!("scan {}: {error}", file.display()));
        unresolved_path_attribute |= inventory.has_custom_path;
        for module in inventory
            .modules
            .into_iter()
            .filter(|module| module.external)
        {
            let Some(path) = external_module_path(file, &module.name) else {
                continue;
            };
            if module.test_only {
                test_only_modules.insert(path);
            } else {
                production_modules.insert(path);
            }
        }
    }
    if !unresolved_path_attribute {
        files.retain(|file| !test_only_modules.contains(file) || production_modules.contains(file));
    }
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

fn scan_roots(
    workspace: &Path,
    roots: &[std::path::PathBuf],
) -> (Vec<String>, BTreeMap<(String, String), usize>, usize) {
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
    let mut instants = BTreeMap::new();
    for file in files {
        let rel = file
            .strip_prefix(workspace)
            .expect("under root")
            .to_string_lossy()
            .replace('\\', "/");
        let text = std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("read {rel}: {e}"));
        let non_test = non_test(&text).unwrap_or_else(|error| panic!("scan {rel}: {error}"));
        let code = code_only(&non_test);
        for (n, line) in code.lines().enumerate() {
            for pat in FORBIDDEN {
                if contains_word(line, pat) {
                    hits.push(format!("{rel}:{}: `{pat}`", n + 1));
                }
            }
        }
        for pat in REVIEWED_PATTERNS {
            let count = code.matches(pat).count();
            if count > 0 {
                instants.insert((pat.to_string(), rel.clone()), count);
            }
        }
    }
    (hits, instants, source_count)
}

fn scan(workspace: &Path) -> (Vec<String>, BTreeMap<(String, String), usize>, usize) {
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
    let (hits, instants, source_count) = scan(&workspace);
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
    if !hits.is_empty() || instants != allowed {
        eprintln!(
            "determinism inventory: source_files={source_count}, forbidden={hits:?}, reviewed_patterns={instants:?}, expected_patterns={allowed:?}"
        );
    }
    assert!(
        hits.is_empty(),
        "wall-clock or random sources in rendering code (derive from ticks, --frame or a fixture clock instead):\n  {}",
        hits.join("\n  ")
    );
    assert_eq!(
        instants, allowed,
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
        count, 2,
        "only the proven out-of-line test module is excluded"
    );
    assert_eq!(hits.len(), 2, "both production files are scanned");
    assert!(hits[0].contains("lib.rs:5") && hits[0].contains("SystemTime"));
    assert!(hits[1].contains("production_tests_helper.rs:1") && hits[1].contains("UNIX_EPOCH"));
}
