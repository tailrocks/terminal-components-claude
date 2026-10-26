use std::collections::BTreeMap;

use super::scan::{ScannedFile, is_raw_buffer_path, receiver_before, words};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub rule: &'static str,
    pub file: String,
    pub line: usize,
    pub symbol: Option<String>,
    pub detail: String,
    pub generated: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Preview,
    Library,
    Production,
}

#[derive(Debug, Clone, Copy)]
pub struct Rule {
    pub id: &'static str,
    pub name: &'static str,
    pub scope: Scope,
    pub summary: &'static str,
}

pub const RULES: &[Rule] = &[
    Rule {
        id: "OWN-01",
        name: "preview-raw-buffer",
        scope: Scope::Preview,
        summary: "preview code imports a raw ratatui buffer/cell type",
    },
    Rule {
        id: "OWN-02",
        name: "preview-paint-call",
        scope: Scope::Preview,
        summary: "preview code calls a cell-writing paint method",
    },
    Rule {
        id: "OWN-03",
        name: "preview-raw-buffer-escape",
        scope: Scope::Preview,
        summary: "preview code escapes to the raw buffer via Ui::raw/peek",
    },
    Rule {
        id: "OWN-04",
        name: "preview-paint-wrapper-call",
        scope: Scope::Preview,
        summary: "preview code calls a local paint-wrapper fn",
    },
    Rule {
        id: "OWN-05",
        name: "preview-paint-macro",
        scope: Scope::Preview,
        summary: "a preview macro paints cells or is invoked",
    },
    Rule {
        id: "OWN-06",
        name: "preview-stored-frame-include",
        scope: Scope::Preview,
        summary: "preview code includes a stored frame/asset file",
    },
    Rule {
        id: "OWN-07",
        name: "preview-screen-literal",
        scope: Scope::Preview,
        summary: "preview code embeds a screen-shaped string literal",
    },
    Rule {
        id: "OWN-08",
        name: "preview-fixture-render-gate",
        scope: Scope::Preview,
        summary: "preview rendering branches on test/debug/env input",
    },
    Rule {
        id: "OWN-09",
        name: "preview-paused-bypass",
        scope: Scope::Preview,
        summary: "preview code keys behavior off paused state",
    },
    Rule {
        id: "OWN-10",
        name: "preview-scenario-render-switch",
        scope: Scope::Preview,
        summary: "preview code selects behavior by scenario id",
    },
    Rule {
        id: "OWN-11",
        name: "preview-size-render-switch",
        scope: Scope::Preview,
        summary: "preview code selects behavior by width/height comparison",
    },
    Rule {
        id: "OWN-12",
        name: "library-preview-dependency",
        scope: Scope::Library,
        summary: "library code depends on a preview crate",
    },
    Rule {
        id: "OWN-13",
        name: "library-stored-frame",
        scope: Scope::Library,
        summary: "production library code references stored expected frames",
    },
    Rule {
        id: "OWN-14",
        name: "expect-in-production",
        scope: Scope::Production,
        summary: "production code names expect outside the allowed place",
    },
];

const UI_PAINT_METHODS: &[&str] = &[
    "paint_cell",
    "paint_str",
    "paint_matched",
    "paint_middle",
    "paint_spans",
    "paint_style",
    "fill",
    "rule",
    "frame",
    "glyph",
    "dim_layer",
    "scroll_edges",
    "scroll_edges_except",
];

const BUFFER_PAINT_METHODS: &[&str] = &[
    "set_string",
    "set_stringn",
    "set_span",
    "set_style",
    "set_line",
    "cell_mut",
];

pub const TOOLING_MEMBERS: &[&str] = &["termrock-xtask"];

pub fn is_library_member(member: &str) -> bool {
    member == "crates/termrock"
        || member.starts_with("crates/termrock-")
        || member.starts_with("crates/termrock/")
}

pub fn is_production_member(member: &str) -> bool {
    let name = member.rsplit('/').next().unwrap_or(member);
    !TOOLING_MEMBERS.contains(&name)
}

pub fn preview_crate_names(members: &[String]) -> Vec<String> {
    members
        .iter()
        .filter(|m| !is_library_member(m))
        .map(|m| m.rsplit('/').next().unwrap_or(m).replace('-', "_"))
        .collect()
}

pub fn match_file(file: &ScannedFile, preview_crates: &[String]) -> Vec<Finding> {
    let mut findings = Vec::new();
    let preview = !is_library_member(&file.member);
    if preview {
        match_imports(file, &mut findings);
        match_paint_calls(file, &mut findings);
        match_raw_escapes(file, &mut findings);
        match_macros(file, &mut findings);
        match_includes(file, &mut findings);
        match_screen_literals(file, &mut findings);
        match_fixture_gates(file, &mut findings);
        match_paused(file, &mut findings);
        match_scenario_switches(file, &mut findings);
        match_size_switches(file, &mut findings);
    } else {
        match_library_preview_deps(file, preview_crates, &mut findings);

        if is_production_member(&file.member) {
            match_library_stored_frames(file, &mut findings);
        }
    }
    if is_production_member(&file.member) {
        match_expects(file, &mut findings);
    }
    findings
}

fn production_lines(file: &ScannedFile) -> impl Iterator<Item = &super::scan::CodeLine> {
    file.lines
        .iter()
        .filter(|line| !file.is_test_line(line.number))
}

fn finding(file: &ScannedFile, rule: &'static str, line: usize, detail: String) -> Finding {
    Finding {
        rule,
        file: file.path.clone(),
        line,
        symbol: file.enclosing_fn(line).map(str::to_string),
        detail,
        generated: file.generated,
    }
}

fn match_imports(file: &ScannedFile, findings: &mut Vec<Finding>) {
    for (local, path) in &file.aliases {
        if !is_raw_buffer_path(path) {
            continue;
        }
        let line = import_line(file, local).unwrap_or(1);
        if file.is_test_line(line) {
            continue;
        }
        findings.push(finding(
            file,
            "OWN-01",
            line,
            format!("imports raw buffer type {path} as {local}"),
        ));
    }
}

fn import_line(file: &ScannedFile, local: &str) -> Option<usize> {
    production_lines(file)
        .filter(|line| line.code.contains("use "))
        .find(|line| words(&line.code).contains(&local))
        .map(|line| line.number)
}

fn match_paint_calls(file: &ScannedFile, findings: &mut Vec<Finding>) {
    for line in production_lines(file) {
        for method in UI_PAINT_METHODS.iter().chain(BUFFER_PAINT_METHODS.iter()) {
            for dot in call_dots(&line.code, method) {
                let Some(receiver) = receiver_before(&line.code, dot) else {
                    continue;
                };
                let ui_method = UI_PAINT_METHODS.contains(method);
                if ui_method && !file.is_ui_ident(&receiver) {
                    continue;
                }
                findings.push(finding(
                    file,
                    "OWN-02",
                    line.number,
                    format!("{receiver}.{method} paints cells"),
                ));
            }
        }
    }
}

fn match_raw_escapes(file: &ScannedFile, findings: &mut Vec<Finding>) {
    for line in production_lines(file) {
        for method in ["raw", "peek"] {
            for dot in call_dots(&line.code, method) {
                let Some(receiver) = receiver_before(&line.code, dot) else {
                    continue;
                };
                if !file.is_ui_ident(&receiver) {
                    continue;
                }
                findings.push(finding(
                    file,
                    "OWN-03",
                    line.number,
                    format!("{receiver}.{method} escapes to the raw buffer"),
                ));
            }
        }
    }
}

fn macro_body_paints(body: &str, method: &str) -> bool {
    for receiver in ["ui.", "$ui."] {
        let needle = format!("{receiver}{method}");
        let mut search = 0;
        while let Some(offset) = body[search..].find(&needle) {
            let at = search + offset;
            let before_ok = body[..at]
                .chars()
                .next_back()
                .is_none_or(|c| !c.is_alphanumeric() && c != '_' && c != '$');
            let after = &body[at + needle.len()..];
            if before_ok && (after.starts_with('(') || after.starts_with("::")) {
                return true;
            }
            search = at + needle.len();
        }
    }
    false
}

fn call_dots(code: &str, method: &str) -> Vec<usize> {
    let mut dots = Vec::new();
    let mut search = 0;
    let needle = format!(".{method}");
    while let Some(offset) = code[search..].find(&needle) {
        let dot = search + offset;
        let after = dot + needle.len();
        let rest = &code[after..];

        let is_call = rest.starts_with('(') || rest.starts_with("::");
        let boundary = rest
            .chars()
            .next()
            .is_none_or(|c| !c.is_alphanumeric() && c != '_');
        if is_call && boundary {
            dots.push(dot);
        }
        search = after;
    }
    dots
}

fn match_macros(file: &ScannedFile, findings: &mut Vec<Finding>) {
    let mut painted = Vec::new();
    for (name, range) in &file.macros {
        let body: String = file
            .lines
            .iter()
            .filter(|line| range.contains(line.number) && !file.is_test_line(line.number))
            .map(|line| line.code.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let paints = UI_PAINT_METHODS
            .iter()
            .any(|method| macro_body_paints(&body, method));
        if paints {
            painted.push(name.clone());
            findings.push(finding(
                file,
                "OWN-05",
                range.start,
                format!("macro {name} paints cells"),
            ));
        }
    }
    for name in &painted {
        let bang = format!("{name}!");
        for line in production_lines(file) {
            if line.code.contains(&bang) && !line.code.contains("macro_rules") {
                findings.push(finding(
                    file,
                    "OWN-05",
                    line.number,
                    format!("invokes paint macro {name}"),
                ));
            }
        }
    }
}

fn match_includes(file: &ScannedFile, findings: &mut Vec<Finding>) {
    for line in production_lines(file) {
        if line.code.contains("include_str!") || line.code.contains("include_bytes!") {
            findings.push(finding(
                file,
                "OWN-06",
                line.number,
                "includes a stored file as application output".to_string(),
            ));
        }
    }
}

fn match_screen_literals(file: &ScannedFile, findings: &mut Vec<Finding>) {
    for span in &file.strings {
        if span.end_line - span.start_line < 2 {
            continue;
        }
        let boxed = span
            .text
            .lines()
            .filter(|line| line.chars().any(is_box_char))
            .count();
        if boxed >= 2 {
            findings.push(finding(
                file,
                "OWN-07",
                span.start_line,
                "embeds a screen-shaped string literal".to_string(),
            ));
        }
    }
}

fn is_box_char(char: char) -> bool {
    ('\u{2500}'..='\u{257f}').contains(&char)
}

fn match_fixture_gates(file: &ScannedFile, findings: &mut Vec<Finding>) {
    for line in production_lines(file) {
        let code = &line.code;
        let gated = code.contains("cfg!(test)")
            || code.contains("cfg!(debug_assertions)")
            || code.contains("option_env!")
            || code.contains("env!(");
        if gated {
            findings.push(finding(
                file,
                "OWN-08",
                line.number,
                "gates rendering on test/debug/env input".to_string(),
            ));
        }
    }
}

fn match_paused(file: &ScannedFile, findings: &mut Vec<Finding>) {
    for line in production_lines(file) {
        if words(&line.code).contains(&"paused") {
            findings.push(finding(
                file,
                "OWN-09",
                line.number,
                "keys behavior off paused state".to_string(),
            ));
        }
    }
}

fn match_scenario_switches(file: &ScannedFile, findings: &mut Vec<Finding>) {
    for line in production_lines(file) {
        let tokens = words(&line.code);
        let has_scenario = tokens.contains(&"scenario");
        if !has_scenario {
            continue;
        }
        let code = &line.code;
        let switched = tokens.contains(&"match")
            || code.contains("scenario ==")
            || code.contains("== scenario")
            || code.contains("scenario !=")
            || code.contains("!= scenario");
        if switched {
            findings.push(finding(
                file,
                "OWN-10",
                line.number,
                "selects behavior by scenario id".to_string(),
            ));
        }
    }
}

const MAX_SIGNATURE_BYTES: usize = 8 * 1024;
const MAX_DELIMITER_DEPTH: usize = 256;
const MAX_PARSE_DEPTH: usize = 64;
const MAX_RETURN_TYPE_STEPS: usize = 1024;

fn return_type_generic_angle_masks(file: &ScannedFile) -> Vec<Vec<usize>> {
    let mut source = String::new();
    let mut line_starts = Vec::with_capacity(file.lines.len());
    for line in &file.lines {
        line_starts.push(source.len());
        source.push_str(&line.code);
        source.push('\n');
    }

    let mut masks = vec![Vec::new(); file.lines.len()];
    let mut delimiters = Vec::new();
    let mut uncertain = false;
    for (line_index, line) in file.lines.iter().enumerate() {
        if !uncertain
            && delimiters.is_empty()
            && let Some(fn_offset) = ordinary_fn_offset(&line.code)
        {
            let fn_start = line_starts[line_index] + fn_offset;
            if let Some(angles) = named_function_return_type_angles(&source, fn_start) {
                for angle in angles {
                    let angle_line = line_starts
                        .partition_point(|start| *start <= angle)
                        .saturating_sub(1);
                    let local_offset = angle - line_starts[angle_line];
                    if local_offset < file.lines[angle_line].code.len() {
                        masks[angle_line].push(local_offset);
                    }
                }
            }
        }

        if uncertain {
            continue;
        }
        for byte in line.code.bytes() {
            match byte {
                b'(' | b'[' | b'{' => {
                    if delimiters.len() == MAX_DELIMITER_DEPTH {
                        uncertain = true;
                        break;
                    }
                    delimiters.push(match byte {
                        b'(' => b')',
                        b'[' => b']',
                        _ => b'}',
                    });
                }
                b')' | b']' | b'}' if delimiters.pop() != Some(byte) => {
                    uncertain = true;
                    delimiters.clear();
                    break;
                }
                _ => {}
            }
        }
    }

    for mask in &mut masks {
        mask.sort_unstable();
        mask.dedup();
    }
    masks
}

fn ordinary_fn_offset(code: &str) -> Option<usize> {
    let (offset, _) = code
        .char_indices()
        .find(|(_, character)| !character.is_whitespace())?;
    let bytes = code.as_bytes();
    if ordinary_fn_keyword_at(bytes, offset) {
        return Some(offset);
    }

    let visibility_end = offset.checked_add(3)?;
    if !bytes
        .get(offset..visibility_end)
        .is_some_and(|visibility| visibility == b"pub")
        || !bytes
            .get(visibility_end)
            .is_some_and(|next| next.is_ascii_whitespace() || *next == b'(')
    {
        return None;
    }

    let mut cursor = visibility_end;
    skip_ascii_whitespace(bytes, &mut cursor, bytes.len());
    if bytes.get(cursor) == Some(&b'(') {
        cursor += 1;
        skip_ascii_whitespace(bytes, &mut cursor, bytes.len());
        let crate_end = cursor.checked_add(b"crate".len())?;
        if !bytes
            .get(cursor..crate_end)
            .is_some_and(|visibility| visibility == b"crate")
            || !bytes
                .get(crate_end)
                .is_some_and(|next| next.is_ascii_whitespace() || *next == b')')
        {
            return None;
        }
        cursor = crate_end;
        skip_ascii_whitespace(bytes, &mut cursor, bytes.len());
        if bytes.get(cursor) != Some(&b')') {
            return None;
        }
        cursor += 1;
    }
    skip_ascii_whitespace(bytes, &mut cursor, bytes.len());
    ordinary_fn_keyword_at(bytes, cursor).then_some(cursor)
}

fn ordinary_fn_keyword_at(bytes: &[u8], offset: usize) -> bool {
    let Some(keyword_end) = offset.checked_add(2) else {
        return false;
    };
    bytes
        .get(offset..keyword_end)
        .is_some_and(|keyword| keyword == b"fn")
        && bytes.get(keyword_end).is_some_and(u8::is_ascii_whitespace)
}

fn named_function_return_type_angles(source: &str, start: usize) -> Option<Vec<usize>> {
    let bytes = source.as_bytes();
    let limit = start.saturating_add(MAX_SIGNATURE_BYTES).min(bytes.len());
    if !bytes.get(start..limit)?.starts_with(b"fn")
        || !bytes.get(start + 2).is_some_and(u8::is_ascii_whitespace)
    {
        return None;
    }

    let mut cursor = start + 2;
    skip_ascii_whitespace(bytes, &mut cursor, limit);
    parse_ascii_identifier(bytes, &mut cursor, limit)?;
    skip_ascii_whitespace(bytes, &mut cursor, limit);
    if bytes.get(cursor) != Some(&b'(') {
        return None;
    }
    cursor = skip_balanced_parameters(bytes, cursor, limit)?;
    skip_ascii_whitespace(bytes, &mut cursor, limit);
    if !bytes.get(cursor..limit)?.starts_with(b"->") {
        return None;
    }
    cursor += 2;

    let mut parser = SimpleReturnTypeParser {
        bytes,
        cursor,
        limit,
        angles: Vec::new(),
        depth: 0,
        steps: 0,
    };
    parser.parse_type()?;
    parser.skip_whitespace();
    (parser.peek() == Some(b'{')).then_some(parser.angles)
}

fn skip_ascii_whitespace(bytes: &[u8], cursor: &mut usize, limit: usize) {
    while *cursor < limit && bytes[*cursor].is_ascii_whitespace() {
        *cursor += 1;
    }
}

fn parse_ascii_identifier(bytes: &[u8], cursor: &mut usize, limit: usize) -> Option<()> {
    if *cursor >= limit {
        return None;
    }
    let first = bytes[*cursor];
    if !(first.is_ascii_alphabetic() || first == b'_') {
        return None;
    }
    *cursor += 1;
    while *cursor < limit && (bytes[*cursor].is_ascii_alphanumeric() || bytes[*cursor] == b'_') {
        *cursor += 1;
    }
    Some(())
}

fn skip_balanced_parameters(bytes: &[u8], start: usize, limit: usize) -> Option<usize> {
    if bytes.get(start) != Some(&b'(') {
        return None;
    }
    let mut closing = vec![b')'];
    let mut cursor = start + 1;
    while cursor < limit {
        let byte = bytes[cursor];
        match byte {
            b'(' | b'[' | b'{' => {
                if closing.len() == MAX_PARSE_DEPTH {
                    return None;
                }
                closing.push(match byte {
                    b'(' => b')',
                    b'[' => b']',
                    _ => b'}',
                });
            }
            b')' | b']' | b'}' => {
                if closing.pop() != Some(byte) {
                    return None;
                }
                if closing.is_empty() {
                    return Some(cursor + 1);
                }
            }
            _ => {}
        }
        cursor += 1;
    }
    None
}

struct SimpleReturnTypeParser<'a> {
    bytes: &'a [u8],
    cursor: usize,
    limit: usize,
    angles: Vec<usize>,
    depth: usize,
    steps: usize,
}

impl SimpleReturnTypeParser<'_> {
    fn parse_type(&mut self) -> Option<()> {
        self.skip_whitespace();
        if self.depth == MAX_PARSE_DEPTH || self.step().is_none() {
            return None;
        }
        self.depth += 1;
        let result = self.parse_type_inner();
        self.depth -= 1;
        result
    }

    fn parse_type_inner(&mut self) -> Option<()> {
        match self.peek()? {
            b'&' => self.parse_reference(),
            b'(' => self.parse_tuple_or_group(),
            _ => self.parse_path(),
        }
    }

    fn parse_reference(&mut self) -> Option<()> {
        self.cursor += 1;
        self.skip_whitespace();
        if self.peek() == Some(b'\'') {
            self.parse_lifetime()?;
            self.skip_whitespace();
        }
        if self.consume_keyword(b"mut") {
            self.skip_whitespace();
        }
        self.parse_type()
    }

    fn parse_tuple_or_group(&mut self) -> Option<()> {
        self.cursor += 1;
        self.skip_whitespace();
        if self.consume(b')') {
            return Some(());
        }
        self.parse_type()?;
        loop {
            self.skip_whitespace();
            if self.consume(b')') {
                return Some(());
            }
            if !self.consume(b',') {
                return None;
            }
            self.skip_whitespace();
            if self.consume(b')') {
                return Some(());
            }
            self.parse_type()?;
        }
    }

    fn parse_path(&mut self) -> Option<()> {
        self.parse_identifier()?;
        self.parse_optional_generics()?;
        loop {
            self.skip_whitespace();
            if !self.consume_double_colon() {
                return Some(());
            }
            self.step()?;
            self.skip_whitespace();
            self.parse_identifier()?;
            self.parse_optional_generics()?;
        }
    }

    fn parse_optional_generics(&mut self) -> Option<()> {
        self.skip_whitespace();
        if self.peek() != Some(b'<') {
            return Some(());
        }
        self.angles.push(self.cursor);
        self.cursor += 1;
        self.skip_whitespace();
        if self.peek() == Some(b'>') {
            return None;
        }
        loop {
            self.step()?;
            if self.peek() == Some(b'\'') {
                self.parse_lifetime()?;
            } else {
                self.parse_type()?;
            }
            self.skip_whitespace();
            if self.consume(b'>') {
                self.angles.push(self.cursor - 1);
                return Some(());
            }
            if !self.consume(b',') {
                return None;
            }
            self.skip_whitespace();
            if self.consume(b'>') {
                self.angles.push(self.cursor - 1);
                return Some(());
            }
        }
    }

    fn parse_lifetime(&mut self) -> Option<()> {
        if !self.consume(b'\'') {
            return None;
        }
        self.parse_identifier()
    }

    fn parse_identifier(&mut self) -> Option<()> {
        parse_ascii_identifier(self.bytes, &mut self.cursor, self.limit)
    }

    fn consume_double_colon(&mut self) -> bool {
        if self
            .bytes
            .get(self.cursor..self.limit)
            .is_some_and(|rest| rest.starts_with(b"::"))
        {
            self.cursor += 2;
            true
        } else {
            false
        }
    }

    fn consume_keyword(&mut self, keyword: &[u8]) -> bool {
        let end = self.cursor.saturating_add(keyword.len());
        let matches = self
            .bytes
            .get(self.cursor..self.limit)
            .is_some_and(|rest| rest.starts_with(keyword));
        let boundary = end >= self.limit
            || !self.bytes[end].is_ascii_alphanumeric() && self.bytes[end] != b'_';
        if matches && boundary {
            self.cursor = end;
            true
        } else {
            false
        }
    }

    fn step(&mut self) -> Option<()> {
        if self.steps == MAX_RETURN_TYPE_STEPS {
            return None;
        }
        self.steps += 1;
        Some(())
    }

    fn skip_whitespace(&mut self) {
        skip_ascii_whitespace(self.bytes, &mut self.cursor, self.limit);
    }

    fn peek(&self) -> Option<u8> {
        (self.cursor < self.limit).then(|| self.bytes[self.cursor])
    }

    fn consume(&mut self, byte: u8) -> bool {
        if self.peek() == Some(byte) {
            self.cursor += 1;
            true
        } else {
            false
        }
    }
}

fn match_size_switches(file: &ScannedFile, findings: &mut Vec<Finding>) {
    let generic_angles = return_type_generic_angle_masks(file);
    for (line_index, line) in file.lines.iter().enumerate() {
        if file.is_test_line(line.number) {
            continue;
        }
        let tokens = words(&line.code);
        let sized = tokens.iter().any(|w| *w == "width" || *w == "height");
        if !sized {
            continue;
        }
        let mut normalized = line.code.replace("->", "  ").replace("=>", "  ");
        for angle in &generic_angles[line_index] {
            normalized.replace_range(*angle..*angle + 1, " ");
        }
        let compared = ["<=", ">=", "==", "!=", "<", ">"]
            .iter()
            .any(|op| normalized.contains(op));
        if compared {
            findings.push(finding(
                file,
                "OWN-11",
                line.number,
                "selects behavior by width/height comparison".to_string(),
            ));
        }
    }
}

fn match_library_preview_deps(
    file: &ScannedFile,
    preview_crates: &[String],
    findings: &mut Vec<Finding>,
) {
    for line in production_lines(file) {
        for krate in preview_crates {
            let qualified = format!("{krate}::");
            if line.code.contains(&qualified) {
                findings.push(finding(
                    file,
                    "OWN-12",
                    line.number,
                    format!("library code depends on preview crate {krate}"),
                ));
                break;
            }
        }
    }
}

fn match_library_stored_frames(file: &ScannedFile, findings: &mut Vec<Finding>) {
    for span in &file.strings {
        let fully_certified_test_span = span.start_line <= span.end_line
            && (span.start_line..=span.end_line).all(|line| file.is_test_line(line));
        if span.text.contains("baselines/") && !fully_certified_test_span {
            findings.push(finding(
                file,
                "OWN-13",
                span.start_line,
                "library code references stored expected frames".to_string(),
            ));
        }
    }
}

fn match_expects(file: &ScannedFile, findings: &mut Vec<Finding>) {
    for line in production_lines(file) {
        for (offset, _) in line.code.match_indices("expect") {
            let after = &line.code[offset + "expect".len()..];
            let before = line.code[..offset].trim_end();
            let method_call = after.starts_with('(') && before.ends_with('.');
            let macro_call = after.starts_with('!');
            if method_call || macro_call {
                findings.push(finding(
                    file,
                    "OWN-14",
                    line.number,
                    "names expect in production code".to_string(),
                ));
                break;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Wrapper {
    pub member: String,
    pub file: String,
    pub name: String,
}

pub fn collect_wrappers(
    files: &BTreeMap<String, ScannedFile>,
    findings: &[Finding],
) -> Vec<Wrapper> {
    let mut wrappers = Vec::new();
    for finding in findings {
        if finding.rule != "OWN-02" && finding.rule != "OWN-03" {
            continue;
        }
        let Some(symbol) = &finding.symbol else {
            continue;
        };
        let Some(file) = files.get(&finding.file) else {
            continue;
        };
        if !takes_ui_param(file, symbol) {
            continue;
        }
        if wrappers
            .iter()
            .any(|w: &Wrapper| w.member == file.member && w.name == *symbol)
        {
            continue;
        }
        wrappers.push(Wrapper {
            member: file.member.clone(),
            file: file.path.clone(),
            name: symbol.clone(),
        });
    }
    wrappers
}

fn takes_ui_param(file: &ScannedFile, name: &str) -> bool {
    let Some((_, range)) = file.functions.iter().find(|(n, _)| n == name) else {
        return false;
    };
    let mut signature = String::new();
    for line in &file.lines {
        if line.number < range.start || line.number > range.end {
            continue;
        }
        signature.push_str(&line.code);
        if line.code.contains('{') {
            break;
        }
    }
    signature.contains("&mut Ui")
        || signature.contains("&Ui")
        || file.aliases.keys().any(|alias| {
            file.is_ui_ident(alias)
                && (signature.contains(&format!("&mut {alias}"))
                    || signature.contains(&format!("&{alias}")))
        })
}

pub fn match_wrapper_calls(
    files: &BTreeMap<String, ScannedFile>,
    wrappers: &[Wrapper],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for wrapper in wrappers {
        for file in files.values().filter(|f| f.member == wrapper.member) {
            for line in production_lines(file) {
                if is_wrapper_definition(&line.code, &wrapper.name) {
                    continue;
                }
                if calls_wrapper(&line.code, &wrapper.name) {
                    findings.push(finding(
                        file,
                        "OWN-04",
                        line.number,
                        format!("calls paint wrapper {}", wrapper.name),
                    ));
                }
            }
        }
    }
    findings
}

fn is_wrapper_definition(code: &str, name: &str) -> bool {
    let tokens = words(code);
    tokens
        .windows(2)
        .any(|pair| pair[0] == "fn" && pair[1] == name)
}

fn calls_wrapper(code: &str, name: &str) -> bool {
    let mut search = 0;
    while let Some(offset) = code[search..].find(name) {
        let at = search + offset;
        let before = &code[..at];
        let after = &code[at + name.len()..];
        search = at + name.len();
        let before_ok = before
            .chars()
            .next_back()
            .is_none_or(|c| !c.is_alphanumeric() && c != '_' && c != '.');
        let after_ok = after.starts_with('(');
        if before_ok && after_ok {
            return true;
        }
    }
    false
}
