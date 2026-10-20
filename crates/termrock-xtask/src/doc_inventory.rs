//! Fail-closed inventory of Rust documentation examples in tracked workspace inputs.
//!
//! This guard runs as an ordinary `termrock-xtask` nextest integration test. A new
//! executable example must first move into a compiled public-consumer target; until
//! that target exists, the guard rejects the documentation input.

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

const EXPECTED_IGNORED_PATH: &str = "crates/termrock-runtime/src/ui/paint.rs";
const EXPECTED_IGNORED_EXAMPLE: &str = "let view = ScrollRegion::view(st.core.scroll(), content, visible_len);\n\
     ui.fade_picker_overflow(content, &view);";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    pub path: String,
    pub line: usize,
    pub detail: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Report {
    pub tracked_files: usize,
    pub rust_files: usize,
    pub markdown_files: usize,
    pub package_count: usize,
    pub target_count: usize,
    pub package_readmes: usize,
    pub ignored_examples: usize,
    pub issues: Vec<Issue>,
}

impl Report {
    pub fn passed(&self) -> bool {
        self.issues.is_empty()
            && self.rust_files >= 300
            && self.markdown_files >= 40
            && self.package_count >= 40
            && self.target_count >= 150
            && self.ignored_examples == 1
    }

    pub fn describe(&self) -> String {
        let mut output = format!(
            "tracked={} rust={} markdown={} packages={} targets={} readmes={} expected_ignored={}",
            self.tracked_files,
            self.rust_files,
            self.markdown_files,
            self.package_count,
            self.target_count,
            self.package_readmes,
            self.ignored_examples
        );
        for issue in &self.issues {
            output.push_str(&format!(
                "\n{}:{}: {}",
                issue.path, issue.line, issue.detail
            ));
        }
        if !self.passed() && self.issues.is_empty() {
            output.push_str("\ninventory is incomplete or its explicit ignore changed");
        }
        output
    }
}

/// Scan every tracked Rust and Markdown input and validate Cargo's workspace targets.
pub fn scan_workspace(root: &Path) -> Report {
    let mut report = Report::default();
    if !root.join("Cargo.toml").is_file() {
        report
            .issues
            .push(issue("Cargo.toml", 1, "workspace manifest is missing"));
        return report;
    }

    let tracked = match tracked_files(root) {
        Ok(files) => files,
        Err(detail) => {
            report.issues.push(issue(".git", 1, detail));
            return report;
        }
    };
    report.tracked_files = tracked.len();
    if !tracked.contains("Cargo.toml") {
        report.issues.push(issue(
            "Cargo.toml",
            1,
            "workspace manifest is not part of the tracked inventory",
        ));
    }
    let rust_paths = tracked
        .iter()
        .filter(|path| path.ends_with(".rs"))
        .cloned()
        .collect::<Vec<_>>();
    let markdown_paths = tracked
        .iter()
        .filter(|path| is_markdown(path))
        .cloned()
        .collect::<Vec<_>>();
    report.rust_files = rust_paths.len();
    report.markdown_files = markdown_paths.len();

    for relative in &rust_paths {
        match std::fs::read_to_string(root.join(relative)) {
            Ok(source) => scan_rust_source(relative, &source, &mut report),
            Err(error) => report.issues.push(issue(
                relative,
                1,
                format!("tracked Rust source cannot be read: {error}"),
            )),
        }
    }
    for relative in &markdown_paths {
        match std::fs::read_to_string(root.join(relative)) {
            Ok(source) => scan_markdown(relative, &source, &mut report),
            Err(error) => report.issues.push(issue(
                relative,
                1,
                format!("tracked Markdown input cannot be read: {error}"),
            )),
        }
    }

    scan_cargo_inventory(root, &tracked, &mut report);
    if report.ignored_examples != 1 {
        report.issues.push(issue(
            EXPECTED_IGNORED_PATH,
            1,
            "the one reviewed illustrative ignore must remain at its documented source location",
        ));
    }
    report
}

fn tracked_files(root: &Path) -> Result<BTreeSet<String>, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "-z"])
        .output()
        .map_err(|error| format!("cannot execute git ls-files: {error}"))?;
    if !output.status.success() {
        return Err(format!("git ls-files failed with status {}", output.status));
    }
    let mut paths = BTreeSet::new();
    for path in output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
    {
        let path = std::str::from_utf8(path)
            .map_err(|_| "tracked path is not valid UTF-8; inventory is incomplete".to_string())?;
        paths.insert(path.to_string());
    }
    if paths.is_empty() {
        return Err("git returned an empty tracked-file inventory".to_string());
    }
    Ok(paths)
}

fn scan_cargo_inventory(root: &Path, tracked: &BTreeSet<String>, report: &mut Report) {
    let output = Command::new("cargo")
        .arg("metadata")
        .args([
            "--locked",
            "--offline",
            "--no-deps",
            "--format-version",
            "1",
        ])
        .current_dir(root)
        .output();
    let output = match output {
        Ok(output) if output.status.success() => output,
        Ok(output) => {
            report.issues.push(issue(
                "Cargo.toml",
                1,
                format!("cargo metadata failed: {}", output.status),
            ));
            return;
        }
        Err(error) => {
            report.issues.push(issue(
                "Cargo.toml",
                1,
                format!("cannot execute cargo metadata: {error}"),
            ));
            return;
        }
    };
    let metadata: serde_json::Value = match serde_json::from_slice(&output.stdout) {
        Ok(metadata) => metadata,
        Err(error) => {
            report.issues.push(issue(
                "Cargo.toml",
                1,
                format!("cargo metadata returned invalid JSON: {error}"),
            ));
            return;
        }
    };
    let Some(packages) = metadata
        .get("packages")
        .and_then(serde_json::Value::as_array)
    else {
        report.issues.push(issue(
            "Cargo.toml",
            1,
            "cargo metadata omitted workspace package inventory",
        ));
        return;
    };
    report.package_count = packages.len();
    for package in packages {
        let manifest = package
            .get("manifest_path")
            .and_then(serde_json::Value::as_str)
            .and_then(|path| Path::new(path).strip_prefix(root).ok());
        let Some(manifest) = manifest else {
            report.issues.push(issue(
                "Cargo.toml",
                1,
                "workspace package manifest is missing or outside the workspace root",
            ));
            continue;
        };
        let manifest = slash_path(manifest);
        if !tracked.contains(&manifest) {
            report.issues.push(issue(
                &manifest,
                1,
                "workspace package manifest is not tracked",
            ));
        }
        if let Some(targets) = package.get("targets").and_then(serde_json::Value::as_array) {
            report.target_count += targets.len();
            for target in targets {
                let target_path = target
                    .get("src_path")
                    .and_then(serde_json::Value::as_str)
                    .and_then(|path| Path::new(path).strip_prefix(root).ok());
                let Some(target_path) = target_path else {
                    report.issues.push(issue(
                        &manifest,
                        1,
                        "Cargo target source is missing or outside the workspace root",
                    ));
                    continue;
                };
                let target_path = slash_path(target_path);
                if !tracked.contains(&target_path) || !root.join(&target_path).is_file() {
                    report.issues.push(issue(
                        &target_path,
                        1,
                        "Cargo target source is absent from the tracked Rust inventory",
                    ));
                }
            }
        }
        match package.get("readme") {
            Some(serde_json::Value::String(path)) => {
                report.package_readmes += 1;
                let Some(relative) = Path::new(path).strip_prefix(root).ok() else {
                    report.issues.push(issue(
                        &manifest,
                        1,
                        "package README resolves outside the workspace root",
                    ));
                    continue;
                };
                let relative = slash_path(relative);
                if !is_markdown(&relative) || !tracked.contains(&relative) {
                    report.issues.push(issue(
                        &relative,
                        1,
                        "package README is not a tracked Markdown input",
                    ));
                }
            }
            Some(serde_json::Value::Null) | None => {
                if let Some(package_root) = Path::new(&manifest).parent() {
                    for name in ["README.md", "README.markdown", "readme.md"] {
                        let candidate = package_root.join(name);
                        if !root.join(&candidate).exists() {
                            continue;
                        }
                        let candidate = slash_path(&candidate);
                        report.package_readmes += 1;
                        if !tracked.contains(&candidate) || !is_markdown(&candidate) {
                            report.issues.push(issue(
                                candidate,
                                1,
                                "implicit package README is not a tracked Markdown input",
                            ));
                        }
                    }
                }
            }
            Some(_) => report.issues.push(issue(
                &manifest,
                1,
                "package README metadata has an unknown shape",
            )),
        }
    }
    if report.package_count == 0 || report.target_count == 0 {
        report.issues.push(issue(
            "Cargo.toml",
            1,
            "Cargo metadata returned an empty workspace source inventory",
        ));
    }
}

fn scan_rust_source(path: &str, source: &str, report: &mut Report) {
    let mut doc_lines = Vec::new();
    let mut in_block = false;
    let mut in_regular_block = false;
    let mut raw_string_hashes = None;
    let mut attribute: Option<(usize, String, i32)> = None;
    for (index, line) in source.lines().enumerate() {
        let line_number = index + 1;
        let trimmed = line.trim_start();
        if let Some(hashes) = raw_string_hashes {
            let close = format!("\"{}", "#".repeat(hashes));
            let Some(position) = line.find(&close) else {
                continue;
            };
            raw_string_hashes = None;
            if position + close.len() < line.len() {
                continue;
            }
        }
        if in_regular_block {
            if trimmed.contains("*/") {
                in_regular_block = false;
            }
            continue;
        }
        if trimmed.starts_with("/*") && !trimmed.starts_with("/**") {
            in_regular_block = !trimmed.contains("*/");
            continue;
        }
        if in_block {
            let content = trimmed
                .strip_prefix('*')
                .map(str::trim_start)
                .unwrap_or(trimmed);
            doc_lines.push((line_number, content.to_string()));
            if trimmed.contains("*/") {
                in_block = false;
                doc_lines.push((line_number, String::new()));
            }
            continue;
        }
        if trimmed.starts_with("///") && !trimmed.starts_with("////") {
            doc_lines.push((
                line_number,
                trimmed
                    .strip_prefix("///")
                    .unwrap_or_default()
                    .trim_start()
                    .to_string(),
            ));
            continue;
        }
        if trimmed.starts_with("//!") {
            doc_lines.push((
                line_number,
                trimmed
                    .strip_prefix("//!")
                    .unwrap_or_default()
                    .trim_start()
                    .to_string(),
            ));
            continue;
        }
        if trimmed.starts_with("/**") && !trimmed.starts_with("/***") {
            in_block = !trimmed.contains("*/");
            let content = trimmed
                .strip_prefix("/**")
                .unwrap_or_default()
                .trim_start_matches('*')
                .trim();
            if !content.is_empty() {
                doc_lines.push((line_number, content.to_string()));
            }
            if !in_block {
                doc_lines.push((line_number, String::new()));
            }
            continue;
        }
        if !doc_lines.is_empty() {
            inspect_doc_lines(path, &doc_lines, report);
            doc_lines.clear();
        }
        if let Some((hashes, quote)) = raw_string_start(trimmed) {
            let close = format!("\"{}", "#".repeat(hashes));
            if !trimmed
                .get(quote.saturating_add(1)..)
                .unwrap_or_default()
                .contains(&close)
            {
                raw_string_hashes = Some(hashes);
            }
            continue;
        }
        if let Some((start_line, mut content, mut depth)) = attribute.take() {
            content.push('\n');
            content.push_str(line);
            depth += bracket_delta(&mask_literals(line));
            if depth > 0 {
                attribute = Some((start_line, content, depth));
            } else if looks_like_doc_attribute(&content) {
                report.issues.push(issue(
                    path,
                    start_line,
                    "Rust doc attribute requires explicit inventory review; literal and include inputs must be registered",
                ));
            }
            continue;
        }
        if let Some(start) = attribute_start(&mask_literals(line)) {
            let content = line.get(start..).unwrap_or_default().to_string();
            let depth = bracket_delta(&mask_literals(&content));
            if depth > 0 {
                attribute = Some((line_number, content, depth));
                continue;
            }
            if looks_like_doc_attribute(&content) {
                report.issues.push(issue(
                    path,
                    line_number,
                    "Rust doc attribute requires explicit inventory review; literal and include inputs must be registered",
                ));
                continue;
            }
        }
    }
    if !doc_lines.is_empty() {
        inspect_doc_lines(path, &doc_lines, report);
    }
}

fn raw_string_start(line: &str) -> Option<(usize, usize)> {
    let bytes = line.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes.get(index) != Some(&b'r') {
            index += 1;
            continue;
        }
        let mut cursor = index + 1;
        while bytes.get(cursor) == Some(&b'#') {
            cursor += 1;
        }
        if bytes.get(cursor) == Some(&b'"') {
            return Some((cursor.saturating_sub(index + 1), cursor));
        }
        index += 1;
    }
    None
}

fn looks_like_doc_attribute(line: &str) -> bool {
    let code = mask_literals(line)
        .replace("#[doc(hidden)]", "")
        .replace("#![doc(hidden)]", "");
    code.contains("#[doc")
        || code.contains("#![doc")
        || (code.contains("#[cfg_attr") || code.contains("#![cfg_attr"))
            && (code.contains("doc =") || code.contains("doc="))
}

fn attribute_start(line: &str) -> Option<usize> {
    ["#![", "#["]
        .iter()
        .filter_map(|needle| line.find(needle))
        .min()
}

fn bracket_delta(line: &str) -> i32 {
    let mut depth = 0_i32;
    for character in line.chars() {
        if character == '[' {
            depth += 1;
        } else if character == ']' {
            depth -= 1;
        }
    }
    depth
}

fn mask_literals(line: &str) -> String {
    let chars = line.chars().collect::<Vec<_>>();
    let mut masked = chars.clone();
    let mut index = 0;
    while index < chars.len() {
        if chars[index] == 'r' {
            let mut quote = index + 1;
            while chars.get(quote) == Some(&'#') {
                quote += 1;
            }
            if chars.get(quote) == Some(&'"') {
                let hashes = quote.saturating_sub(index + 1);
                let close = format!("\"{}", "#".repeat(hashes));
                let remainder = chars
                    .get(quote + 1..)
                    .unwrap_or_default()
                    .iter()
                    .collect::<String>();
                let end = remainder.find(&close).map_or(chars.len(), |offset| {
                    quote + 1 + offset + close.chars().count()
                });
                for (position, value) in masked.iter_mut().enumerate().take(end).skip(index) {
                    if chars.get(position).is_some_and(char::is_ascii) {
                        *value = ' ';
                    }
                }
                index = end;
                continue;
            }
        }
        if chars[index] == '"' {
            let start = index;
            index += 1;
            while index < chars.len() {
                if chars[index] == '\\' {
                    index = index.saturating_add(2);
                } else if chars[index] == '"' {
                    index += 1;
                    break;
                } else {
                    index += 1;
                }
            }
            for (position, value) in masked
                .iter_mut()
                .enumerate()
                .take(index.min(chars.len()))
                .skip(start)
            {
                if chars.get(position).is_some_and(char::is_ascii) {
                    *value = ' ';
                }
            }
            continue;
        }
        if chars[index] == '\''
            && let Some(close) = chars
                .iter()
                .enumerate()
                .skip(index + 1)
                .take(4)
                .find_map(|(position, character)| (*character == '\'').then_some(position))
        {
            for (position, value) in masked.iter_mut().enumerate().take(close + 1).skip(index) {
                if chars.get(position).is_some_and(char::is_ascii) {
                    *value = ' ';
                }
            }
            index = close + 1;
            continue;
        }
        if chars[index] == '/' && chars.get(index + 1) == Some(&'/') {
            for (position, value) in masked.iter_mut().enumerate().skip(index) {
                if chars.get(position).is_some_and(char::is_ascii) {
                    *value = ' ';
                }
            }
            break;
        }
        index += 1;
    }
    masked.into_iter().collect()
}

fn inspect_doc_lines(path: &str, lines: &[(usize, String)], report: &mut Report) {
    let text = lines
        .iter()
        .map(|(_, line)| line.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    inspect_examples(
        path,
        &text,
        lines.first().map_or(1, |(line, _)| *line),
        report,
    );
}

fn scan_markdown(path: &str, source: &str, report: &mut Report) {
    inspect_examples(path, source, 1, report);
}

fn inspect_examples(path: &str, text: &str, base_line: usize, report: &mut Report) {
    let lines = text.lines().collect::<Vec<_>>();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim_start();
        if let Some((marker, marker_len, info)) = fence_open(trimmed) {
            let start = index;
            index += 1;
            let mut body = Vec::new();
            while index < lines.len() && !fence_close(lines[index].trim_start(), marker, marker_len)
            {
                body.push(lines[index]);
                index += 1;
            }
            let rust =
                rust_info(info) || (info.trim().is_empty() && looks_like_rust(&body.join("\n")));
            if rust {
                record_example(path, base_line + start, info, &body, report);
            }
            index = index.saturating_add(1);
            continue;
        }
        if line.starts_with("    ") || line.starts_with('\t') {
            let start = index;
            let mut body = Vec::new();
            while index < lines.len()
                && (lines[index].starts_with("    ") || lines[index].starts_with('\t'))
            {
                body.push(
                    lines[index]
                        .strip_prefix("    ")
                        .unwrap_or(lines[index].trim_start_matches('\t')),
                );
                index += 1;
            }
            if looks_like_rust(&body.join("\n")) {
                record_example(path, base_line + start, "indented", &body, report);
            }
            continue;
        }
        index += 1;
    }
}

fn fence_open(line: &str) -> Option<(char, usize, &str)> {
    let marker = line.chars().next()?;
    if marker != '`' && marker != '~' {
        return None;
    }
    let length = line.chars().take_while(|ch| *ch == marker).count();
    (length >= 3).then(|| (marker, length, line[length..].trim()))
}

fn fence_close(line: &str, marker: char, minimum: usize) -> bool {
    let prefix = line.chars().take_while(|ch| *ch == marker).count();
    prefix >= minimum && line[prefix..].trim().is_empty()
}

fn rust_info(info: &str) -> bool {
    info.split(|character: char| character == ',' || character.is_whitespace())
        .map(|part| {
            part.trim_matches('{')
                .trim_matches('}')
                .to_ascii_lowercase()
        })
        .any(|part| {
            matches!(
                part.as_str(),
                "rust" | "ignore" | "no_run" | "compile_fail" | "should_panic"
            ) || part.starts_with("edition")
        })
}

fn looks_like_rust(body: &str) -> bool {
    let trimmed = body.trim();
    if trimmed.is_empty() || trimmed.starts_with('$') || trimmed.starts_with("cargo ") {
        return false;
    }
    [
        "fn ",
        "pub fn ",
        "let ",
        "use ",
        "impl ",
        "struct ",
        "enum ",
        "trait ",
        "mod ",
        "match ",
        "for ",
        "if ",
        "const ",
        "type ",
        "assert!(",
        "assert_eq!(",
        "async fn ",
        "#![",
        "#[derive",
    ]
    .iter()
    .any(|token| trimmed.contains(token))
        || trimmed
            .lines()
            .any(|line| line.trim_start().starts_with("///"))
}

fn record_example(path: &str, line: usize, info: &str, body: &[&str], report: &mut Report) {
    let normalized = body
        .iter()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    if path == EXPECTED_IGNORED_PATH
        && info.trim() == "ignore"
        && normalized == EXPECTED_IGNORED_EXAMPLE
    {
        report.ignored_examples += 1;
        return;
    }
    report.issues.push(issue(
        path,
        line,
        format!(
            "unregistered Rust documentation example ({}) must move to a compiled nextest consumer",
            if info.trim().is_empty() {
                "unlabeled"
            } else {
                info.trim()
            }
        ),
    ));
}

fn is_markdown(path: &str) -> bool {
    path.ends_with(".md") || path.ends_with(".markdown")
}

fn slash_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn issue(path: impl AsRef<str>, line: usize, detail: impl Into<String>) -> Issue {
    Issue {
        path: path.as_ref().to_string(),
        line,
        detail: detail.into(),
    }
}

/// Exercise the parser without invoking Git, Cargo metadata, or the filesystem.
pub fn inspect_fixture(path: &str, source: &str) -> Vec<Issue> {
    let mut report = Report::default();
    if path.ends_with(".rs") {
        scan_rust_source(path, source, &mut report);
    } else {
        scan_markdown(path, source, &mut report);
    }
    report.issues
}
