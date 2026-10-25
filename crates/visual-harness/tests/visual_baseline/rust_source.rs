//! Small, dependency-free lexical helpers shared by test-only source guards.
//!
//! This is intentionally not a Rust parser: it masks comments and literals, recognizes only
//! exact inline `#[cfg(test)]` module items, and rejects malformed token structure. Unknown
//! source bindings remain in the projection.

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

pub(super) fn mask_non_code(text: &str) -> Result<Vec<u8>, String> {
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

#[derive(Clone, Copy, Debug)]
struct Token {
    start: usize,
    end: usize,
}

pub(super) struct PatternScan {
    pub(super) matches: Vec<Vec<std::ops::Range<usize>>>,
    pub(super) identifiers: Vec<std::ops::Range<usize>>,
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

fn module_item_range(
    code: &str,
    tokens: &[Token],
    matching: &[Option<usize>],
    mod_at: usize,
    start: usize,
) -> Option<std::ops::Range<usize>> {
    let mut name_at = mod_at + 1;
    if token_text(code, tokens, name_at) == "r"
        && tokens
            .get(name_at + 1)
            .is_some_and(|_| token_text(code, tokens, name_at + 1) == "#")
        && identifier_token(code, tokens, name_at + 2)
    {
        name_at += 3;
    } else {
        if !identifier_token(code, tokens, name_at) {
            return None;
        }
        name_at += 1;
    }
    let token = tokens.get(name_at)?;
    let end = match token_text(code, tokens, name_at) {
        ";" => token.end,
        "{" => tokens[matching[name_at]?].end,
        _ => return None,
    };
    Some(start..end)
}

fn module_after_attribute(
    code: &str,
    tokens: &[Token],
    matching: &[Option<usize>],
    after_attribute: usize,
    start: usize,
) -> Option<std::ops::Range<usize>> {
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
    module_item_range(code, tokens, matching, at, start)
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

fn test_module_ranges(text: &str) -> Result<Vec<std::ops::Range<usize>>, String> {
    let code = String::from_utf8(mask_non_code(text)?).expect("lexical mask preserves UTF-8");
    let (tokens, matching) = tokenize(&code)?;
    let macro_ranges = macro_token_ranges(&code, &tokens, &matching);
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
        let Some(attribute_end) = attribute_end(&code, &tokens, &matching, at) else {
            return Err(format!(
                "unmatched attribute group at byte {}",
                tokens[at].start
            ));
        };
        // Attribute arguments are arbitrary token streams. Only inspect an attribute at its
        // current item context; nested `#[cfg(test)]` tokens belong to the outer attribute.
        if let Some(attribute) = cfg_test_attribute(&code, &tokens, &matching, at)
            && let Some(range) =
                module_after_attribute(&code, &tokens, &matching, attribute, tokens[at].start)
        {
            test_ranges.push(range);
        }
        at = attribute_end + 1;
    }
    Ok(test_ranges)
}

/// Blanks only syntactically complete cfg(test) module items, preserving every newline and byte offset.
pub(super) fn non_test(text: &str) -> Result<String, String> {
    let test_ranges = test_module_ranges(text)?;
    let mut result = text.as_bytes().to_vec();
    for range in test_ranges {
        mask_range(&mut result, range);
    }
    String::from_utf8(result).map_err(|error| format!("masked Rust source is not UTF-8: {error}"))
}

/// Tokenize an opener prefix without requiring its intentionally unclosed delimiter groups.
fn tokenize_pattern_fragment(pattern: &str) -> Vec<Token> {
    let bytes = pattern.as_bytes();
    let mut tokens = Vec::new();
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
        tokens.push(Token { start, end: at });
    }
    tokens
}

fn matching_pattern_ranges(
    code: &str,
    tokens: &[Token],
    pattern: &str,
) -> Vec<std::ops::Range<usize>> {
    let pattern_tokens = tokenize_pattern_fragment(pattern);
    if pattern_tokens.is_empty() {
        return Vec::new();
    }
    let expected: Vec<_> = pattern_tokens
        .iter()
        .map(|token| &pattern[token.start..token.end])
        .collect();
    tokens
        .windows(expected.len())
        .enumerate()
        .filter_map(|(start, window)| {
            window
                .iter()
                .enumerate()
                .all(|(offset, token)| &code[token.start..token.end] == expected[offset])
                .then_some(tokens[start].start..window.last().expect("non-empty pattern").end)
        })
        .collect()
}

/// Match reviewed fragments against strictly tokenized source and retain byte ranges.
pub(super) fn matching_patterns_checked(
    code: &str,
    patterns: &[&str],
) -> Result<PatternScan, String> {
    let (tokens, _) = tokenize(code)?;
    let matches = patterns
        .iter()
        .map(|pattern| matching_pattern_ranges(code, &tokens, pattern))
        .collect();
    let identifiers = tokens
        .into_iter()
        .filter(|token| {
            let text = &code[token.start..token.end];
            text.as_bytes()
                .first()
                .is_some_and(|byte| identifier_byte(*byte) && !byte.is_ascii_digit())
        })
        .map(|token| token.start..token.end)
        .collect();
    Ok(PatternScan {
        matches,
        identifiers,
    })
}

/// Return a byte-stable production projection: only proven inline `cfg(test)` modules, comments,
/// and literal contents are masked. Unknown cfg and macro contexts remain visible to scanners.
pub(super) fn production_code(text: &str) -> Result<String, String> {
    let production = non_test(text)?;
    String::from_utf8(mask_non_code(&production)?)
        .map_err(|error| format!("masked Rust source is not UTF-8: {error}"))
}

const MAX_SHARED_DIALOG_SOURCE_BYTES: usize = 512 * 1024;
const MAX_SHARED_DIALOG_TOKENS: usize = 150_000;
const MAX_SHARED_DIALOG_PROOF_WORK: usize = 25_000_000;

#[derive(Clone, Copy)]
struct FunctionBody {
    body_open: usize,
    body_close: usize,
}

fn identifier_name_at<'a>(code: &'a str, tokens: &[Token], at: usize) -> Option<(&'a str, usize)> {
    tokens.get(at)?;
    if token_text(code, tokens, at) == "r"
        && tokens
            .get(at + 1)
            .is_some_and(|_| token_text(code, tokens, at + 1) == "#")
        && identifier_token(code, tokens, at + 2)
    {
        Some((token_text(code, tokens, at + 2), at + 3))
    } else if identifier_token(code, tokens, at) {
        Some((token_text(code, tokens, at), at + 1))
    } else {
        None
    }
}

fn token_is(code: &str, tokens: &[Token], at: usize, expected: &str) -> bool {
    tokens
        .get(at)
        .is_some_and(|_| token_text(code, tokens, at) == expected)
}

fn token_is_delimiter_open(code: &str, tokens: &[Token], at: usize) -> bool {
    ["(", "[", "{"]
        .iter()
        .any(|open| token_is(code, tokens, at, open))
}

fn path_separator_at(code: &str, tokens: &[Token], at: usize) -> bool {
    token_is(code, tokens, at, ":")
        && token_is(code, tokens, at + 1, ":")
        && tokens[at].end == tokens[at + 1].start
}

fn angle_close(
    code: &str,
    tokens: &[Token],
    matching: &[Option<usize>],
    open: usize,
) -> Option<usize> {
    if !token_is(code, tokens, open, "<") {
        return None;
    }
    let mut depth = 0usize;
    let mut at = open;
    while at < tokens.len() {
        let text = token_text(code, tokens, at);
        if matches!(text, "(" | "[" | "{") {
            at = matching.get(at).copied().flatten()? + 1;
            continue;
        }
        if text == "<" {
            depth = depth.checked_add(1)?;
        } else if text == ">" && !token_is(code, tokens, at.checked_sub(1)?, "-") {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(at);
            }
        }
        at += 1;
    }
    None
}

fn token_levels(code: &str, tokens: &[Token]) -> Vec<bool> {
    let mut levels = vec![false; tokens.len()];
    let mut groups = Vec::new();
    for (at, token) in tokens.iter().enumerate() {
        let text = &code[token.start..token.end];
        levels[at] = groups.is_empty();
        match text {
            "(" | "[" | "{" => groups.push(at),
            ")" | "]" | "}" => {
                groups.pop();
            }
            _ => {}
        }
    }
    levels
}

fn contains_dialog_token(code: &str, tokens: &[Token], start: usize, end: usize) -> bool {
    (start..end).any(|at| token_is(code, tokens, at, "Dialog"))
}

fn use_item_end(
    code: &str,
    tokens: &[Token],
    matching: &[Option<usize>],
    start: usize,
) -> Option<usize> {
    let mut at = start + 1;
    while at < tokens.len() {
        match token_text(code, tokens, at) {
            ";" => return Some(at),
            "(" | "[" | "{" => at = matching.get(at).copied().flatten()? + 1,
            _ => at += 1,
        }
    }
    None
}

fn root_use_has_attribute_before(
    code: &str,
    tokens: &[Token],
    matching: &[Option<usize>],
    use_at: usize,
) -> bool {
    let mut cursor = use_at;
    while cursor > 0 {
        let previous = cursor - 1;
        if token_is(code, tokens, previous, "]") {
            let Some(open) = matching.get(previous).copied().flatten() else {
                return true;
            };
            if open > 0 && token_is(code, tokens, open - 1, "#") {
                return true;
            }
            break;
        }
        if token_is(code, tokens, previous, ")") {
            let Some(open) = matching.get(previous).copied().flatten() else {
                return true;
            };
            if open > 0 && token_is(code, tokens, open - 1, "pub") {
                cursor = open - 1;
                continue;
            }
        }
        if token_is(code, tokens, previous, "pub") {
            cursor = previous;
            continue;
        }
        break;
    }
    false
}

fn direct_dialog_use_item(
    code: &str,
    tokens: &[Token],
    matching: &[Option<usize>],
    start: usize,
    end: usize,
) -> (usize, bool) {
    let path_start = start + 1;
    if token_is(code, tokens, path_start, "termrock")
        && path_separator_at(code, tokens, path_start + 1)
        && token_is(code, tokens, path_start + 3, "Dialog")
        && path_start + 4 == end
    {
        return (1, false);
    }

    if token_is(code, tokens, path_start, "termrock")
        && path_separator_at(code, tokens, path_start + 1)
        && token_is(code, tokens, path_start + 3, "{")
        && matching.get(path_start + 3).copied().flatten() == Some(end - 1)
    {
        let group_open = path_start + 3;
        let group_close = end - 1;
        let mut direct = 0usize;
        let mut conflict = false;
        let mut item_start = group_open + 1;
        let mut at = item_start;
        while at <= group_close {
            if at == group_close || token_is(code, tokens, at, ",") {
                if at == item_start + 1 && token_is(code, tokens, item_start, "Dialog") {
                    direct += 1;
                } else if contains_dialog_token(code, tokens, item_start, at) {
                    conflict = true;
                }
                item_start = at + 1;
                at += 1;
                continue;
            }
            if matches!(token_text(code, tokens, at), "(" | "[" | "{") {
                let Some(close) = matching.get(at).copied().flatten() else {
                    return (0, true);
                };
                at = close + 1;
            } else {
                at += 1;
            }
        }
        return (direct, conflict || direct > 1);
    }

    (0, contains_dialog_token(code, tokens, path_start, end))
}

fn has_unambiguous_root_dialog_import(
    code: &str,
    tokens: &[Token],
    matching: &[Option<usize>],
    root_level: &[bool],
) -> bool {
    let mut direct_imports = 0usize;
    let mut conflict = false;
    for (at, &is_root_level) in root_level.iter().enumerate() {
        if !is_root_level || !token_is(code, tokens, at, "use") {
            continue;
        }
        if root_use_has_attribute_before(code, tokens, matching, at) {
            return false;
        }
        let Some(end) = use_item_end(code, tokens, matching, at) else {
            return false;
        };
        if root_use_binds_termrock(code, tokens, at, end) {
            conflict = true;
        }
        let is_super_glob = is_super_glob_use(code, tokens, at, end);
        if (at + 1..end).any(|index| {
            (token_is(code, tokens, index, "*") && !is_super_glob)
                || standard_scope_macro(token_text(code, tokens, index))
                || built_in_derive(token_text(code, tokens, index))
        }) {
            conflict = true;
        }
        let (direct, other_dialog) = direct_dialog_use_item(code, tokens, matching, at, end);
        direct_imports = direct_imports.saturating_add(direct);
        conflict |= other_dialog;
    }
    direct_imports == 1 && !conflict
}

fn has_root_binding_named(code: &str, tokens: &[Token], root_level: &[bool], wanted: &str) -> bool {
    const ITEMS: [&str; 10] = [
        "struct",
        "enum",
        "union",
        "trait",
        "type",
        "mod",
        "fn",
        "const",
        "static",
        "macro_rules",
    ];
    for (at, &is_root_level) in root_level.iter().enumerate() {
        if token_is(code, tokens, at, "macro_use") {
            return true;
        }
        if !is_root_level {
            continue;
        }
        let item = token_text(code, tokens, at);
        if ITEMS.contains(&item) {
            let name_at = if item == "macro_rules" {
                if !token_is(code, tokens, at + 1, "!") {
                    continue;
                }
                at + 2
            } else if (item == "const" && token_is(code, tokens, at + 1, "fn"))
                || (item == "static" && token_is(code, tokens, at + 1, "mut"))
            {
                at + 2
            } else {
                at + 1
            };
            if identifier_name_at(code, tokens, name_at).is_some_and(|(name, _)| {
                name == wanted
                    || (item == "macro_rules"
                        && (standard_scope_macro(name) || built_in_derive(name)))
            }) {
                return true;
            }
        }
        if item == "extern" && token_is(code, tokens, at + 1, "crate") {
            let Some((crate_name, after_crate_name)) = identifier_name_at(code, tokens, at + 2)
            else {
                return true;
            };
            let bound_name = if token_is(code, tokens, after_crate_name, "as") {
                identifier_name_at(code, tokens, after_crate_name + 1)
                    .map(|(name, _)| name)
                    .unwrap_or("")
            } else {
                crate_name
            };
            if bound_name == wanted
                && !(wanted == "termrock" && crate_name == "termrock" && bound_name == crate_name)
            {
                return true;
            }
        }
    }
    false
}

fn root_use_binds_termrock(code: &str, tokens: &[Token], start: usize, end: usize) -> bool {
    (start + 1..end).any(|at| {
        (token_is(code, tokens, at, "as")
            && identifier_name_at(code, tokens, at + 1).is_some_and(|(name, _)| name == "termrock"))
            || (at > start + 1 && token_is(code, tokens, at, "termrock"))
    })
}

fn macro_ranges_contain(ranges: &[std::ops::Range<usize>], offset: usize) -> bool {
    ranges.iter().any(|range| range.contains(&offset))
}

fn action_scope_macro_ranges(
    code: &str,
    tokens: &[Token],
    matching: &[Option<usize>],
) -> Vec<std::ops::Range<usize>> {
    let mut ranges = Vec::new();
    for bang in 1..tokens.len() {
        if !token_is(code, tokens, bang, "!") {
            continue;
        }
        let open = if token_is(code, tokens, bang - 1, "macro_rules") {
            let Some((_, after_name)) = identifier_name_at(code, tokens, bang + 1) else {
                continue;
            };
            after_name
        } else {
            bang + 1
        };
        if token_is_delimiter_open(code, tokens, open)
            && let Some(close) = matching.get(open).copied().flatten()
        {
            ranges.push(tokens[open].start..tokens[close].end);
        }
    }
    ranges
}

fn inside_inline_module(
    code: &str,
    tokens: &[Token],
    matching: &[Option<usize>],
    macro_ranges: &[std::ops::Range<usize>],
    candidate: usize,
) -> bool {
    for at in 0..tokens.len() {
        if !token_is(code, tokens, at, "mod")
            || macro_ranges_contain(macro_ranges, tokens[at].start)
        {
            continue;
        }
        let Some((_, after_name)) = identifier_name_at(code, tokens, at + 1) else {
            continue;
        };
        if token_is(code, tokens, after_name, "{")
            && matching
                .get(after_name)
                .copied()
                .flatten()
                .is_some_and(|close| at < candidate && candidate < close)
        {
            return true;
        }
    }
    false
}

fn function_bodies(
    code: &str,
    tokens: &[Token],
    matching: &[Option<usize>],
    macro_ranges: &[std::ops::Range<usize>],
) -> Vec<FunctionBody> {
    let mut bodies = Vec::new();
    for fn_at in 0..tokens.len() {
        if !token_is(code, tokens, fn_at, "fn")
            || macro_ranges_contain(macro_ranges, tokens[fn_at].start)
        {
            continue;
        }
        let Some((_, mut at)) = identifier_name_at(code, tokens, fn_at + 1) else {
            continue;
        };
        if token_is(code, tokens, at, "<") {
            let Some(close) = angle_close(code, tokens, matching, at) else {
                continue;
            };
            at = close + 1;
        }
        if !token_is(code, tokens, at, "(") {
            continue;
        }
        let Some(params_close) = matching.get(at).copied().flatten() else {
            continue;
        };
        let mut angle_depth = 0usize;
        let mut header_at = params_close + 1;
        while header_at < tokens.len() {
            let text = token_text(code, tokens, header_at);
            if text == "where" {
                break;
            }
            if matches!(text, "(" | "[") {
                let Some(close) = matching.get(header_at).copied().flatten() else {
                    break;
                };
                header_at = close + 1;
                continue;
            }
            if text == "<" {
                angle_depth = angle_depth.saturating_add(1);
            } else if text == ">" && !token_is(code, tokens, header_at.saturating_sub(1), "-") {
                angle_depth = angle_depth.saturating_sub(1);
            } else if text == "{" {
                if angle_depth > 0 {
                    let Some(close) = matching.get(header_at).copied().flatten() else {
                        break;
                    };
                    header_at = close + 1;
                    continue;
                }
                if let Some(body_close) = matching.get(header_at).copied().flatten() {
                    bodies.push(FunctionBody {
                        body_open: header_at,
                        body_close,
                    });
                }
                break;
            } else if text == ";" && angle_depth == 0 {
                break;
            }
            header_at += 1;
        }
    }
    bodies
}

fn has_dialog_in_let_pattern(
    code: &str,
    tokens: &[Token],
    matching: &[Option<usize>],
    start: usize,
    end: usize,
) -> bool {
    let mut at = start + 1;
    while at < end {
        if token_is(code, tokens, at, "=") {
            return false;
        }
        if matches!(token_text(code, tokens, at), "(" | "[" | "{") {
            let Some(close) = matching.get(at).copied().flatten() else {
                return true;
            };
            if contains_dialog_token(code, tokens, at, close + 1) {
                return true;
            }
            at = close + 1;
        } else {
            if token_is(code, tokens, at, "Dialog") {
                return true;
            }
            at += 1;
        }
    }
    false
}

fn has_dialog_in_for_pattern(code: &str, tokens: &[Token], start: usize, end: usize) -> bool {
    (start + 1..end)
        .take_while(|at| !token_is(code, tokens, *at, "in"))
        .any(|at| token_is(code, tokens, at, "Dialog"))
}

fn statement_has_dialog(
    code: &str,
    tokens: &[Token],
    matching: &[Option<usize>],
    start: usize,
    end: usize,
) -> bool {
    let mut at = start;
    while at < end {
        if token_is(code, tokens, at, ";") {
            return contains_dialog_token(code, tokens, start, at)
                || (start..at).any(|index| token_is(code, tokens, index, "*"));
        }
        if matches!(token_text(code, tokens, at), "(" | "[" | "{") {
            let Some(close) = matching.get(at).copied().flatten() else {
                return true;
            };
            at = close + 1;
        } else {
            at += 1;
        }
    }
    contains_dialog_token(code, tokens, start, end)
}

fn standard_scope_macro(name: &str) -> bool {
    matches!(
        name,
        "assert"
            | "assert_eq"
            | "assert_ne"
            | "debug_assert"
            | "debug_assert_eq"
            | "debug_assert_ne"
            | "matches"
            | "format"
            | "vec"
            | "concat"
            | "stringify"
    )
}

fn built_in_derive(name: &str) -> bool {
    matches!(
        name,
        "Clone" | "Copy" | "Debug" | "Default" | "Eq" | "Hash" | "Ord" | "PartialEq" | "PartialOrd"
    )
}

const INERT_ATTRIBUTES: [&str; 16] = [
    "test",
    "cfg",
    "allow",
    "warn",
    "deny",
    "forbid",
    "expect",
    "doc",
    "inline",
    "cold",
    "must_use",
    "track_caller",
    "deprecated",
    "non_exhaustive",
    "path",
    "no_mangle",
];

fn attribute_is_known_at(
    code: &str,
    tokens: &[Token],
    matching: &[Option<usize>],
    at: usize,
) -> bool {
    if !token_is(code, tokens, at, "#") {
        return true;
    }
    let open = if token_is(code, tokens, at + 1, "!") {
        at + 2
    } else {
        at + 1
    };
    if !token_is(code, tokens, open, "[") {
        return true;
    }
    let Some(close) = matching.get(open).copied().flatten() else {
        return false;
    };
    let Some(name_token) = tokens.get(open + 1) else {
        return false;
    };
    let name = &code[name_token.start..name_token.end];
    if name == "derive" {
        // An external module can inherit macro bindings from its parent. Keep
        // derive resolution uncertain until that provenance is proven.
        return false;
    }
    INERT_ATTRIBUTES.contains(&name) && close > open + 1
}

fn is_super_glob_use(code: &str, tokens: &[Token], at: usize, end: usize) -> bool {
    token_is(code, tokens, at + 1, "super")
        && path_separator_at(code, tokens, at + 2)
        && token_is(code, tokens, at + 4, "*")
        && at + 5 == end
}

fn root_has_super_glob(
    code: &str,
    tokens: &[Token],
    matching: &[Option<usize>],
    root_level: &[bool],
) -> bool {
    (0..tokens.len()).any(|at| {
        if !root_level[at] || !token_is(code, tokens, at, "use") {
            return false;
        }
        use_item_end(code, tokens, matching, at)
            .is_some_and(|end| is_super_glob_use(code, tokens, at, end))
    })
}

fn root_has_unknown_attribute(
    code: &str,
    tokens: &[Token],
    matching: &[Option<usize>],
    root_level: &[bool],
    has_super_glob: bool,
) -> bool {
    (0..tokens.len()).any(|at| {
        if !root_level[at]
            || !token_is(code, tokens, at, "#")
            || !(token_is(code, tokens, at + 1, "[")
                || (token_is(code, tokens, at + 1, "!") && token_is(code, tokens, at + 2, "[")))
        {
            return false;
        }
        !attribute_is_known_at(code, tokens, matching, at)
            || (has_super_glob
                && token_is(code, tokens, at + 1, "[")
                && token_is(code, tokens, at + 2, "derive"))
    })
}

fn function_scope_is_unshadowed(
    code: &str,
    tokens: &[Token],
    matching: &[Option<usize>],
    macro_ranges: &[std::ops::Range<usize>],
    body: FunctionBody,
    has_super_glob: bool,
) -> bool {
    let mut at = body.body_open + 1;
    while at < body.body_close {
        let text = token_text(code, tokens, at);
        if macro_ranges_contain(macro_ranges, tokens[at].start) {
            at += 1;
            continue;
        }
        if text == "#"
            && (!attribute_is_known_at(code, tokens, matching, at)
                || (has_super_glob && token_is(code, tokens, at + 2, "derive")))
        {
            return false;
        }
        if text == "let" && has_dialog_in_let_pattern(code, tokens, matching, at, body.body_close) {
            return false;
        }
        if text == "for" && has_dialog_in_for_pattern(code, tokens, at, body.body_close) {
            return false;
        }
        if matches!(
            text,
            "struct" | "enum" | "union" | "trait" | "type" | "mod" | "fn" | "const" | "static"
        ) && identifier_name_at(code, tokens, at + 1).is_some_and(|(name, _)| name == "Dialog")
        {
            return false;
        }
        if text == "use" && statement_has_dialog(code, tokens, matching, at, body.body_close) {
            return false;
        }
        if text == "extern" && token_is(code, tokens, at + 1, "crate") {
            let Some((crate_name, after_crate_name)) = identifier_name_at(code, tokens, at + 2)
            else {
                return false;
            };
            let bound_name = if token_is(code, tokens, after_crate_name, "as") {
                identifier_name_at(code, tokens, after_crate_name + 1)
                    .map(|(name, _)| name)
                    .unwrap_or("")
            } else {
                crate_name
            };
            if bound_name == "Dialog" {
                return false;
            }
        }
        if text == "macro_rules"
            && token_is(code, tokens, at + 1, "!")
            && identifier_name_at(code, tokens, at + 2).is_some_and(|(name, _)| {
                name == "Dialog" || standard_scope_macro(name) || built_in_derive(name)
            })
        {
            return false;
        }
        if text == "macro_rules" && token_is(code, tokens, at + 1, "!") {
            let body_start = at + 2;
            if let Some(range) = macro_ranges.iter().find(|range| {
                tokens
                    .get(body_start)
                    .is_some_and(|token| token.start == range.start)
            }) && tokens.iter().enumerate().any(|(index, token)| {
                range.contains(&token.start) && token_is(code, tokens, index, "Dialog")
            }) {
                return false;
            }
        }
        if text == "|" && !token_is(code, tokens, at + 1, "|") {
            let mut close = at + 1;
            while close < body.body_close && !token_is(code, tokens, close, "|") {
                close += 1;
            }
            if close < body.body_close && contains_dialog_token(code, tokens, at + 1, close) {
                return false;
            }
        }
        if text == "!"
            && at > body.body_open + 1
            && !token_is(code, tokens, at - 1, "macro_rules")
            && token_is_delimiter_open(code, tokens, at + 1)
        {
            // A macro invocation can inject a local binding anywhere in this
            // function, including when its spelling looks like a standard macro.
            return false;
        }
        at += 1;
    }
    true
}

#[derive(Clone, Copy)]
struct LexicalFrame {
    segment_start: usize,
    is_brace: bool,
    bad_header: bool,
}

fn header_attributes_are_known(
    code: &str,
    tokens: &[Token],
    matching: &[Option<usize>],
    start: usize,
    end: usize,
) -> bool {
    let mut at = start;
    while at < end {
        if token_is(code, tokens, at, "#")
            && (token_is(code, tokens, at + 1, "[")
                || (token_is(code, tokens, at + 1, "!") && token_is(code, tokens, at + 2, "[")))
        {
            let open = if token_is(code, tokens, at + 1, "!") {
                at + 2
            } else {
                at + 1
            };
            let Some(close) = matching.get(open).copied().flatten() else {
                return false;
            };
            if !attribute_is_known_at(code, tokens, matching, at) {
                return false;
            }
            at = close + 1;
        } else {
            at += 1;
        }
    }
    true
}

fn bad_brace_header_context(code: &str, tokens: &[Token], matching: &[Option<usize>]) -> Vec<bool> {
    let mut context = vec![false; tokens.len()];
    let mut frames: Vec<LexicalFrame> = Vec::new();
    let mut root_segment_start = 0usize;
    let mut bad_headers = 0usize;

    for (at, context_at) in context.iter_mut().enumerate() {
        *context_at = bad_headers > 0;
        match token_text(code, tokens, at) {
            "(" | "[" => frames.push(LexicalFrame {
                segment_start: at + 1,
                is_brace: false,
                bad_header: false,
            }),
            "{" => {
                let segment_start = frames
                    .last()
                    .map_or(root_segment_start, |frame| frame.segment_start);
                let bad_header = contains_dialog_token(code, tokens, segment_start, at)
                    || (segment_start..at).any(|index| token_is(code, tokens, index, "where"))
                    || !header_attributes_are_known(code, tokens, matching, segment_start, at);
                frames.push(LexicalFrame {
                    segment_start: at + 1,
                    is_brace: true,
                    bad_header,
                });
                if bad_header {
                    bad_headers += 1;
                }
                *context_at = bad_headers > 0;
            }
            ")" | "]" | "}" => {
                if let Some(frame) = frames.pop()
                    && frame.is_brace
                {
                    if frame.bad_header {
                        bad_headers = bad_headers.saturating_sub(1);
                    }
                    if let Some(parent) = frames.last_mut() {
                        parent.segment_start = at + 1;
                    } else {
                        root_segment_start = at + 1;
                    }
                }
            }
            ";" => {
                if let Some(frame) = frames.last_mut() {
                    frame.segment_start = at + 1;
                } else {
                    root_segment_start = at + 1;
                }
            }
            _ => {}
        }
    }
    context
}

fn root_has_unknown_macro_invocation(code: &str, tokens: &[Token], root_level: &[bool]) -> bool {
    for (at, &is_root_level) in root_level.iter().enumerate().skip(1) {
        if !is_root_level
            || !token_is(code, tokens, at, "!")
            || token_is(code, tokens, at.saturating_sub(1), "#")
            || token_is(code, tokens, at - 1, "macro_rules")
            || !token_is_delimiter_open(code, tokens, at + 1)
        {
            continue;
        }
        return true;
    }
    false
}

fn projection_expression_prefix(code: &str, tokens: &[Token], at: usize) -> bool {
    if at == 0 {
        return true;
    }
    let previous = token_text(code, tokens, at - 1);
    if previous == "=" {
        return at >= 2
            && !matches!(
                token_text(code, tokens, at - 2),
                "=" | "!" | "<" | ">" | "+" | "-" | "*" | "/" | "%" | "&" | "|" | "^" | "."
            );
    }
    if matches!(previous, "(" | "[" | "{" | "," | ";" | "return" | "break") {
        return true;
    }
    previous == ">" && at >= 2 && token_is(code, tokens, at - 2, "=")
}

fn simple_constant_path(code: &str, tokens: &[Token], start: usize, end: usize) -> bool {
    if start >= end {
        return false;
    }
    let mut at = start;
    let Some((first, after_first)) = identifier_name_at(code, tokens, at) else {
        return false;
    };
    if first.is_empty() {
        return false;
    }
    at = after_first;
    while at < end {
        if !path_separator_at(code, tokens, at) {
            return false;
        }
        let Some((segment, after_segment)) = identifier_name_at(code, tokens, at + 2) else {
            return false;
        };
        if segment.is_empty() {
            return false;
        }
        at = after_segment;
    }
    at == end
}

fn decimal_integer_literal(code: &str, tokens: &[Token], start: usize, end: usize) -> bool {
    if start >= end {
        return false;
    }
    let mut at = start;
    let mut previous_end = None;
    while at < end {
        let token = token_text(code, tokens, at);
        if token.len() != 1 || !token.as_bytes()[0].is_ascii_digit() {
            return false;
        }
        if previous_end.is_some_and(|byte| byte != tokens[at].start) {
            return false;
        }
        previous_end = Some(tokens[at].end);
        at += 1;
    }
    true
}

/// Return only openers proven to be shared Dialog constructors reduced directly to an action Id.
/// Every uncertain binding, expression, or token structure stays visible to the overlay scanner.
pub(super) fn shared_dialog_action_id_projection_ranges(
    source: &str,
) -> Result<Vec<std::ops::Range<usize>>, String> {
    if source.len() > MAX_SHARED_DIALOG_SOURCE_BYTES {
        return Ok(Vec::new());
    }
    let code = production_code(source)?;
    let (tokens, matching) = tokenize(&code)?;
    if tokens.len() > MAX_SHARED_DIALOG_TOKENS {
        return Ok(Vec::new());
    }
    let root_level = token_levels(&code, &tokens);
    let macro_ranges = action_scope_macro_ranges(&code, &tokens, &matching);
    let bad_scope_headers = bad_brace_header_context(&code, &tokens, &matching);
    let has_super_glob = root_has_super_glob(&code, &tokens, &matching, &root_level);
    if !has_unambiguous_root_dialog_import(&code, &tokens, &matching, &root_level)
        || has_root_binding_named(&code, &tokens, &root_level, "Dialog")
        || has_root_binding_named(&code, &tokens, &root_level, "termrock")
        || root_has_unknown_attribute(&code, &tokens, &matching, &root_level, has_super_glob)
        || root_has_unknown_macro_invocation(&code, &tokens, &root_level)
    {
        return Ok(Vec::new());
    }
    let bodies = function_bodies(&code, &tokens, &matching, &macro_ranges);
    let constructors = matching_pattern_ranges(&code, &tokens, "Dialog::new(");
    let mut potential_projections = Vec::new();

    for constructor in constructors {
        let open = tokens.partition_point(|token| token.start < constructor.start);
        if tokens
            .get(open)
            .is_none_or(|token| token.start != constructor.start)
            || !projection_expression_prefix(&code, &tokens, open)
            || macro_ranges_contain(&macro_ranges, constructor.start)
            || bad_scope_headers.get(open).copied().unwrap_or(true)
        {
            continue;
        }

        let constructor_open = open + 4;
        if !token_is(&code, &tokens, constructor_open, "(")
            || tokens[constructor_open].start != constructor.end - 1
        {
            continue;
        }
        let Some(constructor_close) = matching.get(constructor_open).copied().flatten() else {
            continue;
        };
        if !simple_constant_path(&code, &tokens, constructor_open + 1, constructor_close) {
            continue;
        }
        let action_dot = constructor_close + 1;
        let action_name = action_dot + 1;
        let action_open = action_name + 1;
        if !token_is(&code, &tokens, action_dot, ".")
            || !token_is(&code, &tokens, action_name, "action_id")
            || !token_is(&code, &tokens, action_open, "(")
        {
            continue;
        }
        let Some(action_close) = matching.get(action_open).copied().flatten() else {
            continue;
        };
        if !decimal_integer_literal(&code, &tokens, action_open + 1, action_close)
            || ![";", ",", ")", "]", "}"]
                .iter()
                .any(|terminator| token_is(&code, &tokens, action_close + 1, terminator))
        {
            continue;
        }
        potential_projections.push((constructor, open));
    }

    let proof_work = tokens
        .len()
        .saturating_mul(potential_projections.len())
        .saturating_mul(
            bodies
                .len()
                .saturating_add(macro_ranges.len())
                .saturating_add(2),
        );
    if proof_work > MAX_SHARED_DIALOG_PROOF_WORK {
        return Ok(Vec::new());
    }

    let mut projections = Vec::new();

    for (constructor, open) in potential_projections {
        if inside_inline_module(&code, &tokens, &matching, &macro_ranges, open) {
            continue;
        }
        let enclosing_bodies: Vec<_> = bodies
            .iter()
            .copied()
            .filter(|body| body.body_open < open && open < body.body_close)
            .collect();
        if enclosing_bodies.is_empty() {
            continue;
        }
        if enclosing_bodies.iter().any(|body| {
            !function_scope_is_unshadowed(
                &code,
                &tokens,
                &matching,
                &macro_ranges,
                *body,
                has_super_glob,
            )
        }) {
            continue;
        }
        projections.push(constructor.start..constructor.end);
    }
    Ok(projections)
}
