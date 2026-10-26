#![cfg(test)]
//! Small, dependency-free lexical helpers shared by test-only source guards.
//!
//! This is intentionally not a Rust parser: it masks comments and literals, recognizes only
//! exact first-root cfg(test) files and inline cfg(test) module items, and rejects malformed token
//! structure. Unknown source bindings remain in the projection.

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
pub(super) struct Token {
    pub(super) start: usize,
    pub(super) end: usize,
}

pub(super) struct PatternScan {
    pub(super) matches: Vec<Vec<std::ops::Range<usize>>>,
    pub(super) identifiers: Vec<std::ops::Range<usize>>,
}

pub(super) fn tokenize(code: &str) -> Result<(Vec<Token>, Vec<Option<usize>>), String> {
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

pub(super) fn token_text<'a>(code: &'a str, tokens: &[Token], index: usize) -> &'a str {
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

/// Recognize only the exact first-root test condition; mixed cfg expressions stay visible.
fn exact_root_cfg_test_attribute(code: &str, tokens: &[Token], matching: &[Option<usize>]) -> bool {
    const ROOT_CFG_TEST: [&str; 8] = ["#", "!", "[", "cfg", "(", "test", ")", "]"];

    tokens.len() >= ROOT_CFG_TEST.len()
        && ROOT_CFG_TEST
            .iter()
            .enumerate()
            .all(|(index, expected)| token_text(code, tokens, index) == *expected)
        && matching.get(2).is_some_and(|close| *close == Some(7))
        && matching.get(4).is_some_and(|close| *close == Some(6))
}

pub(super) fn identifier_token(code: &str, tokens: &[Token], at: usize) -> bool {
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
    if exact_root_cfg_test_attribute(&code, &tokens, &matching) {
        return Ok(std::iter::once(0..text.len()).collect());
    }
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

/// Blanks exact root cfg(test) files and complete inline test modules, preserving byte offsets.
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

pub(super) fn matching_pattern_ranges(
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

/// Return a byte-stable production projection: exact root test files and proven inline test
/// modules, comments, and literal contents are masked. Unknown cfg and macro contexts stay visible.
pub(super) fn production_code(text: &str) -> Result<String, String> {
    let production = non_test(text)?;
    String::from_utf8(mask_non_code(&production)?)
        .map_err(|error| format!("masked Rust source is not UTF-8: {error}"))
}
