use std::collections::BTreeSet;
use std::ops::Range;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};

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

    let mut rust_inputs = rust_paths.into_iter().collect::<BTreeSet<_>>();
    rust_inputs.extend(scan_cargo_inventory(root, &tracked, &mut report));
    scan_rust_inputs(root, rust_inputs, &tracked, &mut report);
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

fn scan_cargo_inventory(
    root: &Path,
    tracked: &BTreeSet<String>,
    report: &mut Report,
) -> BTreeSet<String> {
    let mut source_paths = BTreeSet::new();
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
            return source_paths;
        }
        Err(error) => {
            report.issues.push(issue(
                "Cargo.toml",
                1,
                format!("cannot execute cargo metadata: {error}"),
            ));
            return source_paths;
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
            return source_paths;
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
        return source_paths;
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
                let target_path = match cargo_target_src_path(target) {
                    Ok(path) => path,
                    Err(detail) => {
                        report.issues.push(issue(
                            &manifest,
                            1,
                            format!("Cargo target source metadata is invalid: {detail}"),
                        ));
                        continue;
                    }
                };
                match resolve_cargo_target_source(root, target_path, tracked) {
                    Ok(paths) => {
                        source_paths.insert(paths.declared);
                        source_paths.insert(paths.physical);
                    }
                    Err(detail) => report.issues.push(issue(
                        &manifest,
                        1,
                        format!("Cargo target source `{target_path}` is invalid: {detail}"),
                    )),
                }
            }
        }
        match package.get("readme") {
            Some(serde_json::Value::String(path)) => {
                report.package_readmes += 1;
                if let Err(detail) = resolve_package_readme(root, &manifest, path, tracked) {
                    report.issues.push(issue(
                        &manifest,
                        1,
                        format!("package README `{path}` is invalid: {detail}"),
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
    source_paths
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CargoTargetPaths {
    declared: String,
    physical: String,
}

fn cargo_target_src_path(target: &serde_json::Value) -> Result<&str, String> {
    match target.get("src_path") {
        Some(serde_json::Value::String(path)) => Ok(path),
        Some(serde_json::Value::Null) => Err("`src_path` is null".to_string()),
        Some(_) => Err("`src_path` is not a string".to_string()),
        None => Err("`src_path` is missing".to_string()),
    }
}

fn resolve_cargo_target_source(
    root: &Path,
    source: &str,
    tracked: &BTreeSet<String>,
) -> Result<CargoTargetPaths, String> {
    let canonical_root = root
        .canonicalize()
        .map_err(|error| format!("workspace root cannot be resolved: {error}"))?;
    let source_path = Path::new(source);
    if !source_path.is_absolute() {
        return Err("`src_path` is not absolute".to_string());
    }
    let relative = source_path
        .strip_prefix(root)
        .or_else(|_| source_path.strip_prefix(&canonical_root))
        .map_err(|_| "absolute path is outside the workspace root".to_string())?;
    let normalized_relative = normalize_workspace_relative(relative)?;
    let declared = workspace_git_key(&normalized_relative)?;
    if !tracked.contains(&declared) {
        return Err(format!("declared source `{declared}` is not tracked"));
    }

    let physical_path = source_path
        .canonicalize()
        .map_err(|error| format!("declared source `{declared}` cannot be resolved: {error}"))?;
    if !physical_path.starts_with(&canonical_root) {
        return Err(format!(
            "declared source `{declared}` resolves outside the workspace root to `{}`",
            physical_path.display()
        ));
    }
    if !physical_path.is_file() {
        return Err(format!(
            "declared source `{declared}` is not a regular file"
        ));
    }

    let normalized_path = canonical_root.join(&normalized_relative);
    let normalized_physical = normalized_path
        .canonicalize()
        .map_err(|error| format!("normalized source `{declared}` cannot be resolved: {error}"))?;
    if normalized_physical != physical_path {
        return Err(format!(
            "original path resolves to `{}` but normalized path resolves to `{}`; source identity is ambiguous",
            physical_path.display(),
            normalized_physical.display()
        ));
    }

    let physical_relative = physical_path.strip_prefix(&canonical_root).map_err(|_| {
        format!(
            "physical source `{}` is outside the workspace",
            physical_path.display()
        )
    })?;
    let physical = workspace_git_key(physical_relative)?;
    if !tracked.contains(&physical) {
        return Err(format!("physical source `{physical}` is not tracked"));
    }
    std::fs::File::open(source_path)
        .map_err(|error| format!("declared source `{declared}` cannot be read: {error}"))?;

    Ok(CargoTargetPaths { declared, physical })
}

fn resolve_rust_input(
    root: &Path,
    canonical_root: &Path,
    relative: &str,
    tracked: &BTreeSet<String>,
) -> Result<CargoTargetPaths, String> {
    let relative_path = Path::new(relative);
    if relative_path.is_absolute() {
        return Err("Rust source root is not workspace-relative".to_string());
    }
    let normalized_relative = normalize_workspace_relative(relative_path)?;
    let declared = workspace_git_key(&normalized_relative)?;
    if declared != relative {
        return Err(format!(
            "Rust source root `{relative}` is not in normalized workspace form"
        ));
    }
    if !tracked.contains(&declared) {
        return Err(format!("declared source `{declared}` is not tracked"));
    }

    let source_path = root.join(&normalized_relative);
    let physical_path = source_path
        .canonicalize()
        .map_err(|error| format!("declared source `{declared}` cannot be resolved: {error}"))?;
    if !physical_path.starts_with(canonical_root) {
        return Err(format!(
            "declared source `{declared}` resolves outside the workspace root to `{}`",
            physical_path.display()
        ));
    }
    if !physical_path.is_file() {
        return Err(format!(
            "declared source `{declared}` is not a regular file"
        ));
    }
    let normalized_physical = canonical_root
        .join(&normalized_relative)
        .canonicalize()
        .map_err(|error| format!("normalized source `{declared}` cannot be resolved: {error}"))?;
    if normalized_physical != physical_path {
        return Err(format!(
            "original path resolves to `{}` but normalized path resolves to `{}`; source identity is ambiguous",
            physical_path.display(),
            normalized_physical.display()
        ));
    }
    let physical_relative = physical_path.strip_prefix(canonical_root).map_err(|_| {
        format!(
            "physical source `{}` is outside the workspace",
            physical_path.display()
        )
    })?;
    let physical = workspace_git_key(physical_relative)?;
    if !tracked.contains(&physical) {
        return Err(format!("physical source `{physical}` is not tracked"));
    }
    std::fs::File::open(source_path)
        .map_err(|error| format!("declared source `{declared}` cannot be read: {error}"))?;

    Ok(CargoTargetPaths { declared, physical })
}

fn workspace_git_key(path: &Path) -> Result<String, String> {
    let mut components = Vec::new();
    for component in path.components() {
        let Component::Normal(part) = component else {
            return Err("path is not a normalized workspace-relative path".to_string());
        };
        components.push(
            part.to_str()
                .ok_or_else(|| "workspace path is not valid UTF-8".to_string())?,
        );
    }
    if components.is_empty() {
        return Err("path does not name a file".to_string());
    }
    Ok(components.join("/"))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PackageReadmePaths {
    declared: String,
    physical: String,
}

fn resolve_package_readme(
    root: &Path,
    manifest: &str,
    readme: &str,
    tracked: &BTreeSet<String>,
) -> Result<PackageReadmePaths, String> {
    let canonical_root = root
        .canonicalize()
        .map_err(|error| format!("workspace root cannot be resolved: {error}"))?;
    let manifest = Path::new(manifest);
    if manifest.is_absolute() {
        return Err("package manifest path is not workspace-relative".to_string());
    }
    let package_directory = manifest
        .parent()
        .ok_or_else(|| "package manifest has no parent directory".to_string())?;
    let readme_path = Path::new(readme);

    let declared_path = if readme_path.is_absolute() {
        readme_path
            .strip_prefix(root)
            .or_else(|_| readme_path.strip_prefix(&canonical_root))
            .map_err(|_| "absolute path is outside the workspace root".to_string())?
            .to_path_buf()
    } else {
        package_directory.join(readme_path)
    };
    let declared_relative = normalize_workspace_relative(&declared_path)?;
    let declared = slash_path(&declared_relative);
    if !is_markdown(&declared) {
        return Err(format!("declared path `{declared}` is not Markdown"));
    }
    if !tracked.contains(&declared) {
        return Err(format!("declared path `{declared}` is not tracked"));
    }

    let candidate = if readme_path.is_absolute() {
        readme_path.to_path_buf()
    } else {
        canonical_root.join(package_directory).join(readme_path)
    };
    let physical_path = candidate
        .canonicalize()
        .map_err(|error| format!("declared path `{declared}` cannot be resolved: {error}"))?;
    if !physical_path.starts_with(&canonical_root) {
        return Err(format!(
            "declared path `{declared}` resolves to `{}` outside the workspace root",
            physical_path.display()
        ));
    }
    if !physical_path.is_file() {
        return Err(format!("declared path `{declared}` is not a file"));
    }
    let physical_relative = physical_path
        .strip_prefix(&canonical_root)
        .map_err(|_| format!("declared path `{declared}` resolves outside the workspace root"))?;
    let physical = slash_path(physical_relative);
    if !is_markdown(&physical) {
        return Err(format!("physical target `{physical}` is not Markdown"));
    }
    if !tracked.contains(&physical) {
        return Err(format!("physical target `{physical}` is not tracked"));
    }
    std::fs::File::open(&candidate)
        .map_err(|error| format!("declared path `{declared}` cannot be read: {error}"))?;

    Ok(PackageReadmePaths { declared, physical })
}

fn normalize_workspace_relative(path: &Path) -> Result<PathBuf, String> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(part) => normalized.push(part),
            Component::ParentDir if !normalized.pop() => {
                return Err("path escapes the workspace root".to_string());
            }
            Component::ParentDir => {}
            Component::RootDir | Component::Prefix(_) => {
                return Err("path is not workspace-relative".to_string());
            }
        }
    }
    if normalized.as_os_str().is_empty() {
        return Err("path does not name a file".to_string());
    }
    Ok(normalized)
}

fn scan_rust_inputs(
    root: &Path,
    roots: BTreeSet<String>,
    tracked: &BTreeSet<String>,
    report: &mut Report,
) {
    let canonical_root = match root.canonicalize() {
        Ok(root) => root,
        Err(error) => {
            report.issues.push(issue(
                ".",
                1,
                format!("workspace root cannot be resolved: {error}"),
            ));
            return;
        }
    };
    let mut pending = roots.into_iter().collect::<Vec<_>>();
    let mut scanned = BTreeSet::new();
    while let Some(relative) = pending.pop() {
        if !scanned.insert(relative.clone()) {
            continue;
        }
        if !tracked.contains(&relative) {
            report.issues.push(issue(
                &relative,
                1,
                "Rust source or include input is not tracked",
            ));
            continue;
        }
        let paths = match resolve_rust_input(root, &canonical_root, &relative, tracked) {
            Ok(paths) => paths,
            Err(detail) => {
                report.issues.push(issue(
                    &relative,
                    1,
                    format!("Rust source root is invalid: {detail}"),
                ));
                continue;
            }
        };
        let source = match std::fs::read_to_string(root.join(&relative)) {
            Ok(source) => source,
            Err(error) => {
                report.issues.push(issue(
                    &relative,
                    1,
                    format!("tracked Rust source cannot be read: {error}"),
                ));
                continue;
            }
        };
        if paths.physical != relative {
            pending.push(paths.physical);
        }
        scan_rust_source(&relative, &source, report);
        for include in include_literals(&source) {
            let (line, included) = match include {
                Ok(include) => include,
                Err((line, detail)) => {
                    report.issues.push(issue(&relative, line, detail));
                    continue;
                }
            };
            let parent = Path::new(&relative)
                .parent()
                .unwrap_or_else(|| Path::new(""));
            let joined = root.join(parent).join(&included);
            let canonical = match joined.canonicalize() {
                Ok(path) => path,
                Err(error) => {
                    report.issues.push(issue(
                        &relative,
                        line,
                        format!("include! input {included:?} cannot be resolved: {error}"),
                    ));
                    continue;
                }
            };
            let Ok(canonical_relative) = canonical.strip_prefix(&canonical_root) else {
                report.issues.push(issue(
                    &relative,
                    line,
                    format!("include! input {included:?} resolves outside the workspace"),
                ));
                continue;
            };
            if !canonical.is_file() {
                report.issues.push(issue(
                    &relative,
                    line,
                    format!("include! input {included:?} is not a regular file"),
                ));
                continue;
            }
            let canonical_relative = match workspace_git_key(canonical_relative) {
                Ok(path) => path,
                Err(detail) => {
                    report.issues.push(issue(
                        &relative,
                        line,
                        format!(
                            "include! input {included:?} has invalid workspace identity: {detail}"
                        ),
                    ));
                    continue;
                }
            };
            if !tracked.contains(&canonical_relative) {
                report.issues.push(issue(
                    &canonical_relative,
                    1,
                    "include! input is not tracked",
                ));
                continue;
            }
            pending.push(canonical_relative);
        }
    }
}

fn include_literals(source: &str) -> Vec<Result<(usize, String), (usize, String)>> {
    let tokens = rust_tokens(source);
    let mut includes = Vec::new();
    let mut index = 0;
    while index + 2 < tokens.len() {
        if tokens[index].text != "include" || tokens[index + 1].text != "!" {
            index += 1;
            continue;
        }
        let line = tokens[index].line;
        if tokens[index + 2].text != "(" {
            includes.push(Err((line, "malformed include! invocation".to_string())));
            index += 2;
            continue;
        }
        match tokens.get(index + 3..index + 5) {
            Some([path, close]) if path.string && close.text == ")" => {
                if path.text.contains('\\') {
                    includes.push(Err((
                        line,
                        "escaped include! paths require explicit inventory review".to_string(),
                    )));
                } else {
                    includes.push(Ok((line, path.text.clone())));
                }
            }
            _ => includes.push(Err((
                line,
                "include! path must be a direct string literal so its source can be inventoried"
                    .to_string(),
            ))),
        }
        index += 3;
    }
    includes
}

#[derive(Debug)]
struct RustToken {
    text: String,
    line: usize,
    string: bool,
}

fn rust_tokens(source: &str) -> Vec<RustToken> {
    let bytes = source.as_bytes();
    let mut tokens = Vec::new();
    let (mut index, mut line) = (0, 1);
    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b'\n' {
            line += 1;
            index += 1;
            continue;
        }
        if byte.is_ascii_whitespace() {
            index += 1;
            continue;
        }
        if bytes.get(index..index + 2) == Some(b"//") {
            while index < bytes.len() && bytes[index] != b'\n' {
                index += 1;
            }
            continue;
        }
        if bytes.get(index..index + 2) == Some(b"/*") {
            index += 2;
            let mut depth = 1usize;
            while index < bytes.len() && depth > 0 {
                if bytes.get(index..index + 2) == Some(b"/*") {
                    depth += 1;
                    index += 2;
                } else if bytes.get(index..index + 2) == Some(b"*/") {
                    depth -= 1;
                    index += 2;
                } else {
                    if bytes[index] == b'\n' {
                        line += 1;
                    }
                    index += 1;
                }
            }
            continue;
        }
        let token_line = line;
        if byte == b'r' {
            let mut quote = index + 1;
            while bytes.get(quote) == Some(&b'#') {
                quote += 1;
            }
            if bytes.get(quote) == Some(&b'"') {
                let hashes = quote - index - 1;
                let mut closing = vec![b'"'];
                closing.extend(std::iter::repeat_n(b'#', hashes));
                let start = quote + 1;
                let mut end = start;
                while end + closing.len() <= bytes.len()
                    && bytes.get(end..end + closing.len()) != Some(closing.as_slice())
                {
                    if bytes[end] == b'\n' {
                        line += 1;
                    }
                    end += 1;
                }
                let value = String::from_utf8_lossy(&bytes[start..end]).into_owned();
                index = (end + closing.len()).min(bytes.len());
                tokens.push(RustToken {
                    text: value,
                    line: token_line,
                    string: true,
                });
                continue;
            }
        }
        if byte == b'"' {
            index += 1;
            let start = index;
            let mut valid = true;
            while index < bytes.len() && bytes[index] != b'"' {
                if bytes[index] == b'\\' {
                    valid = false;
                    index = (index + 2).min(bytes.len());
                } else {
                    if bytes[index] == b'\n' {
                        line += 1;
                    }
                    index += 1;
                }
            }
            let end = index;
            if index < bytes.len() {
                index += 1;
            }
            let value = if valid {
                String::from_utf8_lossy(&bytes[start..end]).into_owned()
            } else {
                "\\\\".into()
            };
            tokens.push(RustToken {
                text: value,
                line: token_line,
                string: true,
            });
            continue;
        }
        if byte.is_ascii_alphabetic() || byte == b'_' {
            let start = index;
            index += 1;
            while bytes
                .get(index)
                .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_')
            {
                index += 1;
            }
            tokens.push(RustToken {
                text: String::from_utf8_lossy(&bytes[start..index]).into_owned(),
                line: token_line,
                string: false,
            });
            continue;
        }
        tokens.push(RustToken {
            text: char::from(byte).to_string(),
            line: token_line,
            string: false,
        });
        index += 1;
    }
    tokens
}

fn scan_rust_source(path: &str, source: &str, report: &mut Report) {
    let mut doc_lines = Vec::new();
    let mut doc_block_depth = 0usize;
    let mut regular_block_depth = 0usize;
    let mut raw_string_hashes = None;
    let mut in_cooked_string = false;
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
        if in_cooked_string {
            if cooked_string_closes(line) {
                in_cooked_string = false;
            }
            continue;
        }
        if regular_block_depth > 0 {
            regular_block_depth = advance_block_depth(trimmed, regular_block_depth);
            continue;
        }
        if doc_block_depth > 0 {
            let content = trimmed
                .strip_prefix('*')
                .map(str::trim_start)
                .unwrap_or(trimmed);
            doc_lines.push((line_number, content.to_string()));
            doc_block_depth = advance_block_depth(trimmed, doc_block_depth);
            if doc_block_depth == 0 {
                doc_lines.push((line_number, String::new()));
            }
            continue;
        }
        if trimmed.starts_with("/*") && !trimmed.starts_with("/**") && !trimmed.starts_with("/*!") {
            regular_block_depth = advance_block_depth(trimmed, 0);
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
        let doc_prefix = if trimmed.starts_with("/**") && !trimmed.starts_with("/***") {
            Some("/**")
        } else if trimmed.starts_with("/*!") {
            Some("/*!")
        } else {
            None
        };
        if let Some(prefix) = doc_prefix {
            doc_block_depth = advance_block_depth(trimmed, 0);
            let mut content = trimmed
                .strip_prefix(prefix)
                .unwrap_or_default()
                .trim_start_matches('*')
                .trim();
            if doc_block_depth == 0 {
                content = content.strip_suffix("*/").unwrap_or(content).trim_end();
            }
            if !content.is_empty() {
                doc_lines.push((line_number, content.to_string()));
            }
            if doc_block_depth == 0 {
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
        if cooked_string_continues(line) {
            in_cooked_string = true;
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

fn advance_block_depth(source: &str, mut depth: usize) -> usize {
    let bytes = source.as_bytes();
    let mut index = 0;
    while index + 1 < bytes.len() {
        match &bytes[index..index + 2] {
            b"/*" => {
                depth += 1;
                index += 2;
            }
            b"*/" => {
                depth = depth.saturating_sub(1);
                index += 2;
            }
            _ => index += 1,
        }
    }
    depth
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

fn cooked_string_continues(line: &str) -> bool {
    let mut in_string = false;
    let mut escaped = false;
    for character in line.chars() {
        if !in_string {
            if character == '"' {
                in_string = true;
            }
        } else if escaped {
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == '"' {
            in_string = false;
        }
    }
    in_string && line.trim_end().ends_with('\\')
}

fn cooked_string_closes(line: &str) -> bool {
    let mut escaped = false;
    for character in line.chars() {
        if escaped {
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == '"' {
            return true;
        }
    }
    false
}

fn looks_like_doc_attribute(line: &str) -> bool {
    let tokens = rust_tokens(line);
    tokens.windows(2).any(|pair| {
        !pair[0].string && !pair[1].string && pair[0].text == "doc" && pair[1].text == "="
    })
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

struct MarkdownCodeBlock {
    start: usize,
    info: String,
    indented: bool,
    body: String,
}

fn inspect_examples(path: &str, text: &str, base_line: usize, report: &mut Report) {
    let mut active_code_block = None;
    let mut html_ranges = Vec::new();

    for (event, range) in Parser::new(text).into_offset_iter() {
        match event {
            Event::Start(Tag::CodeBlock(kind)) => {
                if let Some(block) = active_code_block.take() {
                    inspect_code_block(path, text, base_line, block, report);
                    report_parser_event_error(path, text, range.start, base_line, report);
                }
                let (info, indented) = match kind {
                    CodeBlockKind::Fenced(info) => (info.into_string(), false),
                    CodeBlockKind::Indented => (String::new(), true),
                };
                active_code_block = Some(MarkdownCodeBlock {
                    start: range.start,
                    info,
                    indented,
                    body: String::new(),
                });
            }
            Event::Text(content) => {
                if let Some(block) = active_code_block.as_mut() {
                    block.body.push_str(&content);
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if let Some(block) = active_code_block.as_mut() {
                    block.body.push('\n');
                }
            }
            Event::End(TagEnd::CodeBlock) => {
                if let Some(block) = active_code_block.take() {
                    inspect_code_block(path, text, base_line, block, report);
                } else {
                    report_parser_event_error(path, text, range.start, base_line, report);
                }
            }
            Event::Html(_) | Event::InlineHtml(_) => {
                if active_code_block.is_some() {
                    report_parser_event_error(path, text, range.start, base_line, report);
                } else {
                    html_ranges.push(range);
                }
            }
            _ => {
                if active_code_block.is_some() {
                    report_parser_event_error(path, text, range.start, base_line, report);
                }
            }
        }
    }

    if let Some(block) = active_code_block {
        inspect_code_block(path, text, base_line, block, report);
    }

    html_ranges.sort_by_key(|range| range.start);
    let mut merged_html_ranges: Vec<Range<usize>> = Vec::new();
    for range in html_ranges {
        if let Some(previous) = merged_html_ranges.last_mut() {
            let gap_is_whitespace = text
                .get(previous.end..range.start)
                .is_some_and(|gap| gap.trim().is_empty());
            if range.start <= previous.end || gap_is_whitespace {
                previous.end = previous.end.max(range.end);
                continue;
            }
        }
        merged_html_ranges.push(range);
    }
    for range in merged_html_ranges {
        inspect_html_fences(path, text, range, base_line, report);
    }
}

fn inspect_code_block(
    path: &str,
    source: &str,
    base_line: usize,
    block: MarkdownCodeBlock,
    report: &mut Report,
) {
    let Some(line) = source_line_at(source, block.start, base_line) else {
        report.issues.push(issue(
            path,
            base_line,
            "Markdown parser returned an invalid source offset",
        ));
        return;
    };
    let info = if block.indented {
        "indented"
    } else {
        block.info.as_str()
    };
    if block.indented {
        if !looks_like_rust(&block.body) {
            return;
        }
    } else if !rust_info(info) && !(info.trim().is_empty() && looks_like_rust(&block.body)) {
        return;
    }

    let body = block.body.lines().collect::<Vec<_>>();
    record_example(path, line, info, &body, report);
}

fn inspect_html_fences(
    path: &str,
    source: &str,
    range: Range<usize>,
    base_line: usize,
    report: &mut Report,
) {
    let Some(html) = source.get(range.clone()) else {
        report.issues.push(issue(
            path,
            base_line,
            "Markdown parser returned an invalid HTML source range",
        ));
        return;
    };

    let mut active_fence: Option<(char, usize, String, usize, Vec<&str>)> = None;
    let mut relative_start = 0;
    for raw_line in html.split_inclusive('\n') {
        let line = raw_line.strip_suffix('\n').unwrap_or(raw_line);
        let line = line.strip_suffix('\r').unwrap_or(line);
        let trimmed = line.trim_start();
        let closes_fence = active_fence
            .as_ref()
            .is_some_and(|(marker, minimum, _, _, _)| fence_close(trimmed, *marker, *minimum));
        if closes_fence {
            if let Some((_, _, info, start, body)) = active_fence.take() {
                record_html_fence(path, source, base_line, start, &info, &body, report);
            }
        } else if let Some((_, _, _, _, body)) = active_fence.as_mut() {
            body.push(line);
        } else if let Some((marker, length, info)) = fence_open(trimmed) {
            active_fence = Some((
                marker,
                length,
                info.to_string(),
                range.start + relative_start,
                Vec::new(),
            ));
        }
        relative_start += raw_line.len();
    }

    if let Some((_, _, info, start, body)) = active_fence {
        record_html_fence(path, source, base_line, start, &info, &body, report);
    }
}

fn record_html_fence(
    path: &str,
    source: &str,
    base_line: usize,
    start: usize,
    info: &str,
    body: &[&str],
    report: &mut Report,
) {
    if !rust_info(info) && !(info.trim().is_empty() && looks_like_rust(&body.join("\n"))) {
        return;
    }
    let Some(line) = source_line_at(source, start, base_line) else {
        report.issues.push(issue(
            path,
            base_line,
            "Markdown parser returned an invalid HTML fence offset",
        ));
        return;
    };
    record_example(path, line, info, body, report);
}

fn report_parser_event_error(
    path: &str,
    source: &str,
    offset: usize,
    base_line: usize,
    report: &mut Report,
) {
    let line = source_line_at(source, offset, base_line).unwrap_or(base_line);
    report.issues.push(issue(
        path,
        line,
        "CommonMark parser emitted an invalid code-block event sequence",
    ));
}

fn source_line_at(source: &str, offset: usize, base_line: usize) -> Option<usize> {
    let prefix = source.get(..offset)?;
    base_line.checked_add(prefix.bytes().filter(|byte| *byte == b'\n').count())
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
        || trimmed.lines().any(|line| {
            let line = line.trim();
            let expression = line.contains('(')
                && line.contains(')')
                && (line.ends_with(';') || line.contains("::") || line.contains('.'));
            expression
                && line
                    .chars()
                    .next()
                    .is_some_and(|character| character.is_ascii_alphabetic() || character == '_')
        })
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

pub fn inspect_fixture(path: &str, source: &str) -> Vec<Issue> {
    let mut report = Report::default();
    if path.ends_with(".rs") {
        scan_rust_source(path, source, &mut report);
    } else {
        scan_markdown(path, source, &mut report);
    }
    report.issues
}
